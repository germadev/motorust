//! How much detail the world is drawn with. The sun's shadows have a
//! [`Level`], chosen in Options > Graphics or, automatically, lowered while
//! the frame rate falls short of its target and raised again once it keeps
//! up ([`AutoQuality`]): how far they reach, in how many cascades and how
//! fine. They fade out at the end of their reach (see [`super::shadow_fade`]),
//! so a change of level shows as little as a shadow coming near. Nothing
//! else changes with the level, so nothing pops in or out as it does. The
//! edges are smoothed as Options > Graphics says, fixed: changing it would
//! make every pipeline again, and what they draw would vanish meanwhile.
//!
//! With two players the world is drawn twice a frame, once for each half of
//! the screen, each with its own shadows, so (unless Options > Graphics says
//! otherwise) it is drawn lighter: the shadows at the lowest level, the
//! grass, flowers and petals nearer and the trees lighter sooner.

use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::visibility::VisibilityRange;
use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::sky::Sunlight;
use crate::menu::GameState;
use crate::pause::Pause;
use crate::settings::{Antialiasing, Settings, ShadowQuality};
use crate::welcome::GameMode;

/// How much detail the shadows have, from the most to the least.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Four cascades out to 250 m, 2048 texels each: as they always were.
    #[default]
    High,
    /// Three out to 150 m.
    Medium,
    /// Two out to 90 m, 1024 texels each.
    Low,
}

impl Level {
    /// The next one down, or up, if there is one.
    pub fn lower(self) -> Option<Self> {
        match self {
            Level::High => Some(Level::Medium),
            Level::Medium => Some(Level::Low),
            Level::Low => None,
        }
    }

    pub fn higher(self) -> Option<Self> {
        match self {
            Level::High => None,
            Level::Medium => Some(Level::High),
            Level::Low => Some(Level::Medium),
        }
    }

    /// The level a quality chosen fixes, if it does.
    pub fn of(quality: ShadowQuality) -> Option<Self> {
        match quality {
            ShadowQuality::Auto => None,
            ShadowQuality::High => Some(Level::High),
            ShadowQuality::Medium => Some(Level::Medium),
            ShadowQuality::Low => Some(Level::Low),
        }
    }
}

/// The detail the world is drawn with now: the shadows' level, and whether
/// the screen is split (and drawn lighter for it).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Detail {
    pub level: Level,
    pub split: bool,
}

impl Detail {
    /// The share of their full reach the grass, flowers and petals are
    /// drawn to.
    pub fn reach(self) -> f32 {
        if self.split { 0.55 } else { 1.0 }
    }

    /// From how far from the nearest camera (m) the trees are drawn with
    /// their first lighter mesh, and with their second (see
    /// [`super::lod::level_at`]).
    pub fn tree_levels(self) -> [f32; 2] {
        if self.split {
            [50.0, 150.0]
        } else {
            [100.0, 280.0]
        }
    }

    /// The level the shadows are drawn at: with the screen split, the
    /// lowest.
    fn shadows(self) -> Level {
        if self.split { Level::Low } else { self.level }
    }

    /// The sun's shadow cascades (see [`Level`]).
    pub fn cascades(self) -> CascadeShadowConfig {
        let (num_cascades, first_cascade_far_bound, maximum_distance) = match self.shadows() {
            Level::High => (4, 25.0, 250.0),
            Level::Medium => (3, 20.0, 150.0),
            Level::Low => (2, 20.0, 90.0),
        };
        CascadeShadowConfigBuilder {
            num_cascades,
            first_cascade_far_bound,
            maximum_distance,
            ..default()
        }
        .build()
    }

    /// How many texels wide each shadow cascade is.
    pub fn shadow_map(self) -> usize {
        match self.shadows() {
            Level::Low => 1024,
            _ => 2048,
        }
    }
}

/// How long (s) each look at the frame rate takes, how long after a change
/// (or the start of a race, or the pause) the frames are not looked at, and
/// how soon after a step up a step down undoes it.
const WINDOW: f32 = 2.0;
const SETTLE: f32 = 3.0;
const UNDONE: f32 = 20.0;
/// How long the frame rate must keep up before a step up (s), at first and
/// at most: each step up soon undone doubles it.
const HOLD: (f32, f32) = (10.0, 300.0);
/// Under this share of the target the level goes down; from this one it
/// keeps up.
const FALLS_SHORT: f32 = 0.9;
const KEEPS_UP: f32 = 0.97;

/// The automatic quality of the shadows: the level it has come to, and how
/// the frame rate has been going.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct AutoQuality {
    pub level: Level,
    /// The frames of the look under way (s), and when it began.
    frames: Vec<f32>,
    began: f32,
    /// Since when the frame rate has kept up.
    steady: Option<f32>,
    hold: f32,
    /// When the last step up was, and the last change (or the end of a
    /// rest), whose frames are not looked at.
    stepped_up: Option<f32>,
    changed: f32,
}

impl Default for AutoQuality {
    fn default() -> Self {
        Self {
            level: Level::High,
            frames: Vec::new(),
            began: 0.0,
            steady: None,
            hold: HOLD.0,
            stepped_up: None,
            changed: f32::NEG_INFINITY,
        }
    }
}

impl AutoQuality {
    /// Not looking (in the menus, or paused): what was being looked at
    /// goes, and the first frames after are let settle.
    pub fn rest(&mut self, now: f32) {
        self.frames.clear();
        self.steady = None;
        self.changed = now;
    }

    /// A frame of `dt` s, ending at `now` (s), aiming at `target` frames a
    /// second: the level to draw with. Every [`WINDOW`] s it looks at the
    /// rate: short of the target, a level down; keeping up for long enough,
    /// a level up.
    pub fn frame(&mut self, now: f32, dt: f32, target: f32) -> Level {
        if now - self.changed < SETTLE {
            return self.level;
        }
        if self.frames.is_empty() {
            self.began = now - dt;
        }
        self.frames.push(dt);
        if now - self.began < WINDOW {
            return self.level;
        }
        let rate = self.frames.len() as f32 / self.frames.iter().sum::<f32>().max(1e-6);
        let began = self.began;
        self.frames.clear();
        if rate < target * FALLS_SHORT {
            self.steady = None;
            if let Some(lower) = self.level.lower() {
                // A step up soon undone: wait longer before the next one.
                if self.stepped_up.is_some_and(|up| now - up < UNDONE) {
                    self.hold = (self.hold * 2.0).min(HOLD.1);
                }
                self.level = lower;
                self.changed = now;
            }
        } else if rate >= target * KEEPS_UP {
            let steady = *self.steady.get_or_insert(began);
            if now - steady >= self.hold
                && let Some(higher) = self.level.higher()
            {
                self.level = higher;
                self.stepped_up = Some(now);
                self.changed = now;
                self.steady = None;
            }
        } else {
            self.steady = None;
        }
        self.level
    }
}

pub struct DetailPlugin;

impl Plugin for DetailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Detail>()
            .init_resource::<AutoQuality>()
            .add_systems(
                PostUpdate,
                (
                    adapt_quality,
                    choose_detail,
                    (shadows_for_detail, reach_for_detail, antialias),
                )
                    .chain(),
            );
    }
}

/// How far an entity with a [`VisibilityRange`] is drawn with one view: its
/// range is this one shortened by [`Detail::reach`].
#[derive(Component, Clone, PartialEq)]
struct FullReach(VisibilityRange);

/// With the shadows automatic, the frame rate looked at during a race (not
/// paused).
fn adapt_quality(
    time: Res<Time<Real>>,
    settings: Option<Res<Settings>>,
    state: Option<Res<State<GameState>>>,
    pause: Option<Res<Pause>>,
    mut auto: ResMut<AutoQuality>,
) {
    let now = time.elapsed_secs();
    let automatic = settings
        .as_deref()
        .is_some_and(|settings| settings.shadows == ShadowQuality::Auto);
    let racing = state.is_some_and(|state| *state.get() == GameState::Race)
        && pause.is_none_or(|pause| !pause.is_open());
    if !(automatic && racing) {
        auto.rest(now);
        return;
    }
    let target = settings.map_or(60.0, |settings| settings.fps.target());
    auto.frame(now, time.delta_secs(), target);
}

/// The detail for the players at play: the shadows' level chosen, or come
/// to automatically; split when two share the screen, unless Options >
/// Graphics says to draw it all anyway.
fn choose_detail(
    game_mode: Option<Res<GameMode>>,
    settings: Option<Res<Settings>>,
    auto: Res<AutoQuality>,
    mut detail: ResMut<Detail>,
) {
    let lighter = settings
        .as_deref()
        .is_none_or(|settings| settings.lighter_split_screen);
    let level = settings
        .and_then(|settings| Level::of(settings.shadows))
        .unwrap_or(auto.level);
    let wanted = Detail {
        level,
        split: lighter && game_mode.is_some_and(|mode| *mode == GameMode::SplitScreen),
    };
    detail.set_if_neq(wanted);
}

/// The sun's shadows as the detail says.
fn shadows_for_detail(
    detail: Res<Detail>,
    shadow_map: Option<ResMut<DirectionalLightShadowMap>>,
    mut sunlight: Query<&mut CascadeShadowConfig, With<Sunlight>>,
    mut done: Local<Option<Detail>>,
) {
    if *done == Some(*detail) || sunlight.is_empty() {
        return;
    }
    *done = Some(*detail);
    for mut cascades in &mut sunlight {
        *cascades = detail.cascades();
    }
    if let Some(mut shadow_map) = shadow_map
        && shadow_map.size != detail.shadow_map()
    {
        shadow_map.size = detail.shadow_map();
    }
}

/// `range` shortened to `share` of it.
pub fn shortened(range: &VisibilityRange, share: f32) -> VisibilityRange {
    VisibilityRange {
        start_margin: range.start_margin.start * share..range.start_margin.end * share,
        end_margin: range.end_margin.start * share..range.end_margin.end * share,
        use_aabb: range.use_aabb,
    }
}

/// Everything drawn only so far from the cameras (the grass, the flowers,
/// the petals) as far as the detail says: as it comes, and all of it when
/// the detail changes.
fn reach_for_detail(
    mut commands: Commands,
    detail: Res<Detail>,
    mut ranges: Query<(Entity, &mut VisibilityRange, Option<&FullReach>)>,
) {
    let share = detail.reach();
    for (entity, mut range, full) in &mut ranges {
        let full = match full {
            Some(full) if detail.is_changed() => full.0.clone(),
            Some(_) => continue,
            None => {
                commands.entity(entity).insert(FullReach(range.clone()));
                range.clone()
            }
        };
        let wanted = shortened(&full, share);
        if *range != wanted {
            *range = wanted;
        }
    }
}

/// The edges smoothed as Options > Graphics says, on every camera as it
/// comes: all of them (the interface's too) multisampled alike, four
/// samples a pixel or one, and FXAA on the world's if chosen.
#[allow(clippy::type_complexity)]
fn antialias(
    mut commands: Commands,
    settings: Option<Res<Settings>>,
    cameras: Query<(Entity, Option<&Msaa>, Has<Fxaa>, Has<Camera3d>), With<Camera>>,
) {
    let chosen = settings.map_or(Antialiasing::default(), |settings| settings.antialiasing);
    let (msaa, fxaa) = match chosen {
        Antialiasing::Msaa => (Msaa::Sample4, false),
        Antialiasing::Fxaa => (Msaa::Off, true),
        Antialiasing::Off => (Msaa::Off, false),
    };
    for (camera, current, has_fxaa, world) in &cameras {
        if current != Some(&msaa) {
            commands.entity(camera).insert(msaa);
        }
        if world && fxaa && !has_fxaa {
            commands.entity(camera).insert(Fxaa::default());
        } else if has_fxaa && !(world && fxaa) {
            commands.entity(camera).remove::<Fxaa>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_level_draws_less_and_two_views_lighter() {
        let at = |level, split| Detail { level, split };
        let levels = [Level::High, Level::Medium, Level::Low];
        for pair in levels.windows(2) {
            let (more, less) = (at(pair[0], false), at(pair[1], false));
            assert!(less.cascades().bounds.last() < more.cascades().bounds.last());
            assert!(less.cascades().bounds.len() < more.cascades().bounds.len());
            assert_eq!(pair[0].lower(), Some(pair[1]));
            assert_eq!(pair[1].higher(), Some(pair[0]));
            // Nothing else changes with the level: nothing pops in or out.
            assert_eq!(less.reach(), more.reach());
            assert_eq!(less.tree_levels(), more.tree_levels());
        }
        // High as they always were: four cascades out to 250 m, 2048 texels.
        let high = at(Level::High, false);
        assert_eq!(high.cascades().bounds.len(), 4);
        assert_eq!(high.cascades().bounds.last(), Some(&250.0));
        assert_eq!(high.shadow_map(), 2048);
        assert_eq!(high.reach(), 1.0);
        // Split, lighter whatever the level.
        let split = at(Level::High, true);
        assert!(split.reach() < high.reach());
        assert_eq!(split.shadow_map(), 1024);
        assert_eq!(split.cascades().bounds.len(), 2);
        assert!(split.tree_levels()[0] < high.tree_levels()[0]);
    }

    /// `seconds` of frames at `fps` from `from` (s), aiming at 60.
    fn run(auto: &mut AutoQuality, from: f32, seconds: f32, fps: f32) -> f32 {
        let dt = 1.0 / fps;
        let mut now = from;
        while now < from + seconds {
            now += dt;
            auto.frame(now, dt, 60.0);
        }
        now
    }

    #[test]
    fn the_automatic_quality_goes_down_while_short_and_back_up_when_it_keeps_up() {
        let mut auto = AutoQuality::default();
        auto.rest(0.0);
        assert_eq!(auto.level, Level::High);
        // At 45 fps: a level down after the settling and a look, then
        // another once that one has settled; and no lower.
        let now = run(&mut auto, 0.0, 5.5, 45.0);
        assert_eq!(auto.level, Level::Medium);
        let now = run(&mut auto, now, 5.5, 45.0);
        assert_eq!(auto.level, Level::Low);
        let now = run(&mut auto, now, 10.0, 45.0);
        assert_eq!(auto.level, Level::Low);
        // Keeping up at 60: not at once, but after a while, back up.
        let now = run(&mut auto, now, 8.0, 60.0);
        assert_eq!(auto.level, Level::Low);
        let now = run(&mut auto, now, 10.0, 60.0);
        assert_eq!(auto.level, Level::Medium);
        // That was too much: down again, and the next step up waits twice
        // as long.
        let now = run(&mut auto, now, 5.5, 45.0);
        assert_eq!(auto.level, Level::Low);
        let now = run(&mut auto, now, 16.0, 60.0);
        assert_eq!(auto.level, Level::Low, "not after the first hold");
        run(&mut auto, now, 10.0, 60.0);
        assert_eq!(auto.level, Level::Medium);
    }

    #[test]
    fn resting_lets_the_first_frames_after_settle() {
        let mut auto = AutoQuality::default();
        auto.rest(0.0);
        // A long frame or two as the race starts (pipelines being made)
        // count for nothing.
        auto.frame(0.5, 0.5, 60.0);
        auto.frame(1.5, 1.0, 60.0);
        run(&mut auto, 1.5, 1.4, 60.0);
        assert_eq!(auto.level, Level::High);
        assert!(auto.frames.is_empty(), "still settling");
    }

    #[test]
    fn what_is_drawn_so_far_comes_nearer_with_two_views_and_back() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameMode>()
            .init_resource::<DirectionalLightShadowMap>()
            .insert_resource(Settings::default())
            .add_plugins(DetailPlugin);
        let range = VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 100.0..120.0,
            use_aabb: true,
        };
        let grass = app.world_mut().spawn(range.clone()).id();
        let sun = app
            .world_mut()
            .spawn((Sunlight, CascadeShadowConfig::default()))
            .id();
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Msaa::Sample4))
            .id();
        app.update();
        // (Its margins: a range does not print.)
        let margins =
            |range: &VisibilityRange| (range.start_margin.clone(), range.end_margin.clone());
        let reach = |app: &App| margins(app.world().get::<VisibilityRange>(grass).unwrap());
        let cascades = |app: &App| {
            app.world()
                .get::<CascadeShadowConfig>(sun)
                .unwrap()
                .bounds
                .clone()
        };
        let high = Detail::default();
        let split = Detail {
            split: true,
            ..high
        };
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), high.cascades().bounds);

        *app.world_mut().resource_mut::<GameMode>() = GameMode::SplitScreen;
        app.update();
        assert_eq!(reach(&app), margins(&shortened(&range, split.reach())));
        assert_eq!(cascades(&app), split.cascades().bounds);
        assert_eq!(
            app.world().resource::<DirectionalLightShadowMap>().size,
            split.shadow_map()
        );
        // Something new comes in shortened too.
        let flowers = app.world_mut().spawn(range.clone()).id();
        app.update();
        assert_eq!(
            app.world()
                .get::<VisibilityRange>(flowers)
                .unwrap()
                .end_margin,
            shortened(&range, split.reach()).end_margin
        );

        // With the option off, two players see it all.
        app.world_mut()
            .resource_mut::<Settings>()
            .lighter_split_screen = false;
        app.update();
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), high.cascades().bounds);
        app.world_mut()
            .resource_mut::<Settings>()
            .lighter_split_screen = true;
        app.update();
        assert_eq!(cascades(&app), split.cascades().bounds);

        *app.world_mut().resource_mut::<GameMode>() = GameMode::SinglePlayer;
        app.update();
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), high.cascades().bounds);
        assert_eq!(
            app.world().resource::<DirectionalLightShadowMap>().size,
            2048
        );

        // Shadows chosen low: only the shadows change.
        app.world_mut().resource_mut::<Settings>().shadows = ShadowQuality::Low;
        app.update();
        let low = Detail {
            level: Level::Low,
            split: false,
        };
        assert_eq!(cascades(&app), low.cascades().bounds);
        assert_eq!(reach(&app), margins(&range));

        // FXAA instead of multisampling; and back.
        let msaa = |app: &App| *app.world().get::<Msaa>(camera).unwrap();
        let fxaa = |app: &App| app.world().get::<Fxaa>(camera).is_some();
        assert_eq!((msaa(&app), fxaa(&app)), (Msaa::Sample4, false));
        app.world_mut().resource_mut::<Settings>().antialiasing = Antialiasing::Fxaa;
        app.update();
        assert_eq!((msaa(&app), fxaa(&app)), (Msaa::Off, true));
        app.world_mut().resource_mut::<Settings>().antialiasing = Antialiasing::Off;
        app.update();
        assert_eq!((msaa(&app), fxaa(&app)), (Msaa::Off, false));
        app.world_mut().resource_mut::<Settings>().antialiasing = Antialiasing::Msaa;
        app.update();
        assert_eq!((msaa(&app), fxaa(&app)), (Msaa::Sample4, false));
    }
}
