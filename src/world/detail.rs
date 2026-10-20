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
//!
//! The performance forced (Options > Graphics > "Force performance") goes
//! further down the same way, quicker, to keep 60 or 120 frames a second
//! (no more than the frame rate cap or the screen let through): the
//! parallax first, if chosen (the dearest of the relief, on the most
//! pixels, the grass, and the least seen in a race: the normal maps stay),
//! then the shadows and the share of the window's resolution the world is
//! drawn at (see [`crate::render_scale`]) by turns, down a ladder of
//! [`Rung`]s ([`ladder`]). None makes a pipeline again, so none stutters.

use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::VisibilityRange;
use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::realm::PortalView;
use super::sky::Sunlight;
use crate::menu::GameState;
use crate::pause::Pause;
use crate::settings::{Antialiasing, Relief, ScreenRate, Settings, ShadowQuality, WEB};
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

/// The detail the world is drawn with now: the shadows' level, whether the
/// screen is split (and drawn lighter for it), the share of the window's
/// resolution the world is drawn at (%, see [`crate::render_scale`]), and
/// whether the parallax may be drawn, if chosen (see
/// [`super::relief::ReliefShown`]).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Detail {
    pub level: Level,
    pub split: bool,
    pub scale: u32,
    pub parallax: bool,
}

impl Default for Detail {
    fn default() -> Self {
        Self {
            level: Level::High,
            split: false,
            scale: 100,
            parallax: true,
        }
    }
}

/// A rung of an automatic quality's ladder: the shadows' level, the share
/// of the window's resolution the world is drawn at (%), and whether the
/// parallax may be drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rung {
    pub level: Level,
    pub scale: u32,
    pub parallax: bool,
}

/// A rung without the parallax.
const fn rung(level: Level, scale: u32) -> Rung {
    Rung {
        level,
        scale,
        parallax: false,
    }
}

/// A rung with the parallax, if chosen.
const fn with_parallax(level: Level, scale: u32) -> Rung {
    Rung {
        level,
        scale,
        parallax: true,
    }
}

/// The shadows alone, automatic (Options > Graphics > Shadows): their three
/// levels, the world at the render scale chosen, the relief as chosen.
const SHADOWS: [Rung; 3] = [
    with_parallax(Level::High, 100),
    with_parallax(Level::Medium, 100),
    with_parallax(Level::Low, 100),
];
/// The performance forced with one view: the shadows and the resolution down
/// by turns, the shadows first, as they show least. With the parallax
/// chosen, it goes first ([`PERFORMANCE_PARALLAX`]).
const PERFORMANCE: [Rung; 8] = [
    rung(Level::High, 100),
    rung(Level::Medium, 100),
    rung(Level::Medium, 90),
    rung(Level::Low, 90),
    rung(Level::Low, 80),
    rung(Level::Low, 70),
    rung(Level::Low, 60),
    rung(Level::Low, 50),
];
/// The same, with the parallax chosen: first without it, the rest after.
const PERFORMANCE_PARALLAX: [Rung; 9] = [
    with_parallax(Level::High, 100),
    rung(Level::High, 100),
    rung(Level::Medium, 100),
    rung(Level::Medium, 90),
    rung(Level::Low, 90),
    rung(Level::Low, 80),
    rung(Level::Low, 70),
    rung(Level::Low, 60),
    rung(Level::Low, 50),
];
/// With two views, whose shadows are at their lowest already: the
/// resolution alone.
const PERFORMANCE_SPLIT: [Rung; 6] = [
    rung(Level::Low, 100),
    rung(Level::Low, 90),
    rung(Level::Low, 80),
    rung(Level::Low, 70),
    rung(Level::Low, 60),
    rung(Level::Low, 50),
];
/// The same, with the parallax chosen: first without it.
const PERFORMANCE_SPLIT_PARALLAX: [Rung; 7] = [
    with_parallax(Level::Low, 100),
    rung(Level::Low, 100),
    rung(Level::Low, 90),
    rung(Level::Low, 80),
    rung(Level::Low, 70),
    rung(Level::Low, 60),
    rung(Level::Low, 50),
];

/// Which ladder an automatic quality is on: the performance forced or the
/// shadows alone, two views drawn lighter (`split`) or one, and the
/// parallax chosen or not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ladder {
    pub performance: bool,
    pub split: bool,
    pub parallax: bool,
}

/// The ladder an automatic quality goes down and up: the performance
/// forced's (for two views or one, with a rung for the parallax first if
/// it is chosen: without it, a rung would change nothing), or the shadows'
/// alone.
pub fn ladder(on: Ladder) -> &'static [Rung] {
    match (on.performance, on.split, on.parallax) {
        (true, false, false) => &PERFORMANCE,
        (true, false, true) => &PERFORMANCE_PARALLAX,
        (true, true, false) => &PERFORMANCE_SPLIT,
        (true, true, true) => &PERFORMANCE_SPLIT_PARALLAX,
        (false, ..) => &SHADOWS,
    }
}

/// The frame rate the performance forced keeps up, if it is on: 60 or 120,
/// or fewer if the frame rate cap lets fewer through (at the screen's rate,
/// the screen at `screen` Hz, once known): the most it could ever reach.
pub fn performance_target(settings: &Settings, screen: Option<f32>) -> Option<f32> {
    let forced = settings.force_performance.fps()?;
    Some(
        settings
            .fps
            .bound(screen)
            .map_or(forced, |bound| forced.min(bound)),
    )
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

/// How an automatic quality looks at the frame rate and steps: how long
/// (s) each look takes; how long after a change (or the start of a race,
/// or the pause) the frames are not looked at; under what share of the
/// target it goes down, and from what share it keeps up; and whether it
/// goes down several rungs at once when far short.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pace {
    window: f32,
    settle: f32,
    falls_short: f32,
    keeps_up: f32,
    jumps: bool,
}

/// The shadows alone: unhurried.
pub const SHADOWS_PACE: Pace = Pace {
    window: 2.0,
    settle: 3.0,
    falls_short: 0.9,
    keeps_up: 0.97,
    jumps: false,
};
/// The performance forced: a look every second, down as soon as the rate
/// falls a little short, as many rungs as it falls short by.
pub const PERFORMANCE_PACE: Pace = Pace {
    window: 1.0,
    settle: 1.0,
    falls_short: 0.95,
    keeps_up: 0.98,
    jumps: true,
};
/// How much faster each rung down is reckoned to draw, to tell how many
/// to go down at once; and the most at once.
const RUNG_GAIN: f32 = 1.1;
const MOST_AT_ONCE: usize = 3;
/// How soon after a step up a step down undoes it (s).
const UNDONE: f32 = 20.0;
/// How long the frame rate must keep up before a step up (s), at first and
/// at most: each step up soon undone doubles it.
const HOLD: (f32, f32) = (10.0, 300.0);

/// An automatic quality: how far down its ladder it has come, and how the
/// frame rate has been going.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct AutoQuality {
    /// The rung (0, the most detail; see [`ladder`]).
    pub step: usize,
    /// The ladder it is on (see [`ladder`]).
    ladder: Ladder,
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
            step: 0,
            ladder: Ladder::default(),
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

    /// On `ladder` (see [`ladder`]): if it was on another, back to its top,
    /// with everything it had learnt forgotten.
    pub fn on(&mut self, ladder: Ladder, now: f32) {
        if self.ladder != ladder {
            *self = Self {
                ladder,
                ..Self::default()
            };
            self.rest(now);
        }
    }

    /// A frame of `dt` s, ending at `now` (s), aiming at `target` frames a
    /// second, at `pace`, on a ladder of `rungs`: the rung to draw with.
    /// Every look at the rate (see [`Pace`]): short of the target, a rung
    /// down (or as many as it is short by, if the pace jumps); keeping up
    /// for long enough, a rung up.
    pub fn frame(&mut self, now: f32, dt: f32, target: f32, pace: &Pace, rungs: usize) -> usize {
        let last = rungs.saturating_sub(1);
        self.step = self.step.min(last);
        if now - self.changed < pace.settle {
            return self.step;
        }
        if self.frames.is_empty() {
            self.began = now - dt;
        }
        self.frames.push(dt);
        if now - self.began < pace.window {
            return self.step;
        }
        let rate = self.frames.len() as f32 / self.frames.iter().sum::<f32>().max(1e-6);
        let began = self.began;
        self.frames.clear();
        if rate < target * pace.falls_short {
            self.steady = None;
            if self.step < last {
                // A step up soon undone: wait longer before the next one.
                if self.stepped_up.is_some_and(|up| now - up < UNDONE) {
                    self.hold = (self.hold * 2.0).min(HOLD.1);
                }
                let down = if pace.jumps {
                    let short = (target / rate.max(1.0)).ln() / RUNG_GAIN.ln();
                    (short.ceil() as usize).clamp(1, MOST_AT_ONCE)
                } else {
                    1
                };
                self.step = (self.step + down).min(last);
                self.changed = now;
            }
        } else if rate >= target * pace.keeps_up {
            let steady = *self.steady.get_or_insert(began);
            if now - steady >= self.hold && self.step > 0 {
                self.step -= 1;
                self.stepped_up = Some(now);
                self.changed = now;
                self.steady = None;
            }
        } else {
            self.steady = None;
        }
        self.step
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

/// Whether the world is drawn lighter for two views now: two share the
/// screen, and Options > Graphics does not say to draw it all anyway.
fn split(settings: Option<&Settings>, game_mode: Option<&GameMode>) -> bool {
    settings.is_none_or(|settings| settings.lighter_split_screen)
        && game_mode.is_some_and(|mode| *mode == GameMode::SplitScreen)
}

/// With the performance forced or the shadows automatic, the frame rate
/// looked at during a race (not paused), on the ladder that goes with them.
#[allow(clippy::too_many_arguments)]
fn adapt_quality(
    time: Res<Time<Real>>,
    settings: Option<Res<Settings>>,
    screen: Option<Res<ScreenRate>>,
    game_mode: Option<Res<GameMode>>,
    state: Option<Res<State<GameState>>>,
    pause: Option<Res<Pause>>,
    mut auto: ResMut<AutoQuality>,
) {
    let now = time.elapsed_secs();
    let settings = settings.as_deref();
    let on = ladder_on(settings, game_mode.as_deref());
    auto.on(on, now);
    let automatic =
        on.performance || settings.is_some_and(|settings| settings.shadows == ShadowQuality::Auto);
    let racing = state.is_some_and(|state| *state.get() == GameState::Race)
        && pause.is_none_or(|pause| !pause.is_open());
    let (Some(settings), true) = (settings, automatic && racing) else {
        auto.rest(now);
        return;
    };
    let screen = screen.and_then(|screen| screen.0);
    let (target, pace) = match performance_target(settings, screen) {
        Some(target) => (target, &PERFORMANCE_PACE),
        None => (settings.fps.target(), &SHADOWS_PACE),
    };
    let rungs = ladder(on).len();
    auto.frame(now, time.delta_secs(), target, pace, rungs);
}

/// The ladder the settings and the players at play call for.
fn ladder_on(settings: Option<&Settings>, game_mode: Option<&GameMode>) -> Ladder {
    Ladder {
        performance: settings.is_some_and(|settings| settings.force_performance.is_on()),
        split: split(settings, game_mode),
        parallax: settings.is_some_and(|settings| settings.relief == Relief::Parallax),
    }
}

/// The detail for the players at play: with the performance forced, as far
/// down its ladder as it has come; otherwise the shadows' level chosen, or
/// come to automatically, and the render scale chosen. Split when two share
/// the screen, unless Options > Graphics says to draw it all anyway.
pub fn choose_detail(
    game_mode: Option<Res<GameMode>>,
    settings: Option<Res<Settings>>,
    auto: Res<AutoQuality>,
    mut detail: ResMut<Detail>,
) {
    let settings = settings.as_deref();
    let on = ladder_on(settings, game_mode.as_deref());
    let rungs = ladder(on);
    let reached = rungs[auto.step.min(rungs.len() - 1)];
    let (level, scale, parallax) = match settings {
        Some(_) if on.performance => (reached.level, reached.scale, reached.parallax),
        Some(settings) => (
            Level::of(settings.shadows).unwrap_or(reached.level),
            settings.render_scale,
            true,
        ),
        None => (reached.level, 100, true),
    };
    detail.set_if_neq(Detail {
        level,
        split: on.split,
        scale,
        parallax,
    });
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

/// Drawn up to `reach` m from the cameras and fading out over `fade` m
/// more, as the grass, the flowers and the petals are: dithered away, as
/// Bevy fades the end of a [`VisibilityRange`]. On the web they end at
/// once instead, halfway through the fade (see [`fading_out_on`]).
pub fn fading_out(reach: f32, fade: f32, use_aabb: bool) -> VisibilityRange {
    fading_out_on(WEB, reach, fade, use_aabb)
}

/// [`fading_out`], with no fade on the `web`: on WebGL2, Bevy (0.19) binds
/// the table of ranges the dithering reads (64 of them, 1024 bytes in its
/// shader) as if it held only one (16 bytes), so the pipeline of every mesh
/// that fades out fails to be made and the game quits before it shows.
/// Ending at once, they read none of it.
fn fading_out_on(web: bool, reach: f32, fade: f32, use_aabb: bool) -> VisibilityRange {
    let end_margin = if web {
        reach + fade / 2.0..reach + fade / 2.0
    } else {
        reach..reach + fade
    };
    VisibilityRange {
        start_margin: 0.0..0.0,
        end_margin,
        use_aabb,
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
/// comes: all of them (the interface's too, while it draws over what the
/// world's drew on the window) multisampled alike, four samples a pixel or
/// one, and FXAA on the world's if chosen. With the world drawn smaller,
/// into an image (see [`crate::render_scale`]), the interface's camera
/// draws on its own over the window, with one sample a pixel: told from
/// where the world's cameras draw this frame, so that both change together.
#[allow(clippy::type_complexity)]
fn antialias(
    mut commands: Commands,
    settings: Option<Res<Settings>>,
    cameras: Query<
        (
            Entity,
            Option<&Msaa>,
            Has<Fxaa>,
            Has<Camera3d>,
            Option<&RenderTarget>,
        ),
        // A portal view takes its player's camera's (see `super::realm`).
        (With<Camera>, Without<PortalView>),
    >,
) {
    let chosen = settings
        .as_deref()
        .map_or(Antialiasing::default(), |settings| settings.antialiasing);
    let (msaa, fxaa) = match chosen {
        Antialiasing::Msaa => (Msaa::Sample4, false),
        Antialiasing::Fxaa => (Msaa::Off, true),
        Antialiasing::Off => (Msaa::Off, false),
    };
    let scaled = cameras
        .iter()
        .any(|(.., world, target)| world && matches!(target, Some(RenderTarget::Image(_))));
    for (camera, current, has_fxaa, world, _) in &cameras {
        let msaa = if scaled && !world { Msaa::Off } else { msaa };
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
    fn what_fades_out_ends_at_once_on_the_web() {
        let desktop = fading_out_on(false, 100.0, 20.0, true);
        assert_eq!(desktop.end_margin, 100.0..120.0);
        assert!(!desktop.is_abrupt());
        // On the web, as far on average, but with no dithering.
        let web = fading_out_on(true, 100.0, 20.0, true);
        assert_eq!(web.end_margin, 110.0..110.0);
        assert!(web.is_abrupt());
        assert!(web.use_aabb);
        // And so it stays with two views.
        assert!(shortened(&web, 0.55).is_abrupt());
        assert_eq!(fading_out(100.0, 20.0, false).is_abrupt(), WEB);
    }

    #[test]
    fn each_level_draws_less_and_two_views_lighter() {
        let at = |level, split| Detail {
            level,
            split,
            scale: 100,
            parallax: true,
        };
        let levels = [Level::High, Level::Medium, Level::Low];
        for pair in levels.windows(2) {
            let (more, less) = (at(pair[0], false), at(pair[1], false));
            assert!(less.cascades().bounds.last() < more.cascades().bounds.last());
            assert!(less.cascades().bounds.len() < more.cascades().bounds.len());
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
            auto.frame(now, dt, 60.0, &SHADOWS_PACE, SHADOWS.len());
        }
        now
    }

    /// The shadows' level the automatic quality of the shadows has come to.
    fn level(auto: &AutoQuality) -> Level {
        SHADOWS[auto.step].level
    }

    #[test]
    fn the_automatic_quality_goes_down_while_short_and_back_up_when_it_keeps_up() {
        let mut auto = AutoQuality::default();
        auto.rest(0.0);
        assert_eq!(level(&auto), Level::High);
        // At 45 fps: a level down after the settling and a look, then
        // another once that one has settled; and no lower.
        let now = run(&mut auto, 0.0, 5.5, 45.0);
        assert_eq!(level(&auto), Level::Medium);
        let now = run(&mut auto, now, 5.5, 45.0);
        assert_eq!(level(&auto), Level::Low);
        let now = run(&mut auto, now, 10.0, 45.0);
        assert_eq!(level(&auto), Level::Low);
        // Keeping up at 60: not at once, but after a while, back up.
        let now = run(&mut auto, now, 8.0, 60.0);
        assert_eq!(level(&auto), Level::Low);
        let now = run(&mut auto, now, 10.0, 60.0);
        assert_eq!(level(&auto), Level::Medium);
        // That was too much: down again, and the next step up waits twice
        // as long.
        let now = run(&mut auto, now, 5.5, 45.0);
        assert_eq!(level(&auto), Level::Low);
        let now = run(&mut auto, now, 16.0, 60.0);
        assert_eq!(level(&auto), Level::Low, "not after the first hold");
        run(&mut auto, now, 10.0, 60.0);
        assert_eq!(level(&auto), Level::Medium);
    }

    /// `seconds` of frames at `fps` from `from` (s) with the performance
    /// forced, one view, aiming at 60.
    fn perform(auto: &mut AutoQuality, from: f32, seconds: f32, fps: f32) -> f32 {
        let dt = 1.0 / fps;
        let mut now = from;
        while now < from + seconds {
            now += dt;
            auto.frame(now, dt, 60.0, &PERFORMANCE_PACE, PERFORMANCE.len());
        }
        now
    }

    #[test]
    fn the_performance_forced_goes_down_quickly_as_far_as_it_falls_short() {
        let mut auto = AutoQuality::default();
        let one = Ladder {
            performance: true,
            ..default()
        };
        auto.on(one, 0.0);
        // At 45 fps: after a second settling and a second looking, three
        // rungs down at once (as many as it falls short by, three at most).
        let now = perform(&mut auto, 0.0, 2.1, 45.0);
        assert_eq!(auto.step, 3);
        // A little short: one more, two seconds later.
        let now = perform(&mut auto, now, 2.1, 56.0);
        assert_eq!(auto.step, 4);
        // Keeping up at 60 (a screen's rate): it stays, and after a while
        // goes back up a rung.
        let now = perform(&mut auto, now, 9.0, 60.0);
        assert_eq!(auto.step, 4);
        let now = perform(&mut auto, now, 3.0, 60.0);
        assert_eq!(auto.step, 3);
        // Far short for long: down to the last rung, and no further.
        perform(&mut auto, now, 20.0, 20.0);
        assert_eq!(auto.step, PERFORMANCE.len() - 1);
        // Another ladder (two views): back to its top.
        auto.on(Ladder { split: true, ..one }, 100.0);
        assert_eq!(auto.step, 0);
    }

    #[test]
    fn the_performance_ladder_drops_the_parallax_then_the_shadows_and_the_resolution_by_turns() {
        let ladders = [
            &PERFORMANCE[..],
            &PERFORMANCE_PARALLAX[..],
            &PERFORMANCE_SPLIT[..],
            &PERFORMANCE_SPLIT_PARALLAX[..],
            &SHADOWS[..],
        ];
        for rungs in ladders {
            assert_eq!(rungs[0].scale, 100);
            for pair in rungs.windows(2) {
                let (above, below) = (pair[0], pair[1]);
                // Each one down a little less: one thing at a time.
                let lower = below.level > above.level;
                let smaller = below.scale < above.scale;
                let flatter = above.parallax && !below.parallax;
                let changes = [lower, smaller, flatter].iter().filter(|&&c| c).count();
                assert_eq!(changes, 1, "{above:?} {below:?}");
                assert!(below.level >= above.level && below.scale <= above.scale);
                assert!(above.parallax || !below.parallax);
            }
        }
        // The parallax, if chosen, first; then the ladder as without it.
        for (with, without) in [
            (&PERFORMANCE_PARALLAX[..], &PERFORMANCE[..]),
            (&PERFORMANCE_SPLIT_PARALLAX[..], &PERFORMANCE_SPLIT[..]),
        ] {
            assert!(with[0].parallax);
            assert_eq!(&with[1..], without);
        }
        assert!(PERFORMANCE.iter().all(|rung| !rung.parallax));
        // The shadows alone leave the relief as chosen.
        assert!(SHADOWS.iter().all(|rung| rung.parallax));
        // As the game starts, at the top; down to half the resolution.
        assert_eq!(PERFORMANCE[0], rung(Level::High, 100));
        assert_eq!(PERFORMANCE.last().unwrap().scale, 50);
        // Two views: the resolution alone, the shadows at their lowest.
        assert!(
            PERFORMANCE_SPLIT
                .iter()
                .all(|rung| rung.level == Level::Low)
        );
        assert!(SHADOWS.iter().all(|rung| rung.scale == 100));
        let on = |performance, split, parallax| Ladder {
            performance,
            split,
            parallax,
        };
        assert_eq!(ladder(on(true, true, false)).len(), PERFORMANCE_SPLIT.len());
        assert_eq!(
            ladder(on(true, false, true)).len(),
            PERFORMANCE_PARALLAX.len()
        );
        assert_eq!(ladder(on(false, true, true)).len(), SHADOWS.len());
    }

    #[test]
    fn the_performance_forced_keeps_60_or_120_as_far_as_the_cap_and_the_screen_let_it() {
        use crate::settings::{ForcePerformance, FpsLimit};
        let mut settings = Settings::default();
        // Off: nothing kept.
        assert_eq!(performance_target(&settings, Some(60.0)), None);
        // 60, or the cap if lower.
        settings.force_performance = ForcePerformance::Fps60;
        settings.fps = FpsLimit::Unlimited;
        assert_eq!(performance_target(&settings, Some(60.0)), Some(60.0));
        settings.fps = FpsLimit::Cap(30);
        assert_eq!(performance_target(&settings, None), Some(30.0));
        settings.fps = FpsLimit::Cap(144);
        assert_eq!(performance_target(&settings, None), Some(60.0));
        // 120, unless the cap or the screen, at its rate, let fewer through
        // (it would never get there, and come down to the last rung).
        settings.force_performance = ForcePerformance::Fps120;
        settings.fps = FpsLimit::Cap(120);
        assert_eq!(performance_target(&settings, Some(60.0)), Some(120.0));
        settings.fps = FpsLimit::Unlimited;
        assert_eq!(performance_target(&settings, Some(60.0)), Some(120.0));
        settings.fps = FpsLimit::Screen;
        assert_eq!(performance_target(&settings, Some(60.0)), Some(60.0));
        assert_eq!(performance_target(&settings, Some(180.0)), Some(120.0));
        assert_eq!(performance_target(&settings, None), Some(120.0));
    }

    #[test]
    fn the_detail_comes_from_the_performance_forced_or_from_the_options() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameMode>()
            .init_resource::<DirectionalLightShadowMap>()
            .insert_resource(Settings {
                shadows: ShadowQuality::Medium,
                render_scale: 70,
                ..default()
            })
            .add_plugins(DetailPlugin);
        app.update();
        let detail = |app: &App| *app.world().resource::<Detail>();
        assert_eq!(
            (detail(&app).level, detail(&app).scale),
            (Level::Medium, 70)
        );
        // With the performance forced, as far down its ladder as it has
        // come.
        app.world_mut().resource_mut::<Settings>().force_performance =
            crate::settings::ForcePerformance::Fps120;
        app.update();
        app.world_mut().resource_mut::<AutoQuality>().step = 4;
        app.update();
        assert_eq!((detail(&app).level, detail(&app).scale), (Level::Low, 80));
        assert!(!detail(&app).parallax);
        // With the parallax chosen, it is the first to go (another ladder:
        // from its top again).
        app.world_mut().resource_mut::<Settings>().relief = Relief::Parallax;
        app.update();
        assert_eq!(app.world().resource::<AutoQuality>().step, 0);
        assert!(detail(&app).parallax);
        app.world_mut().resource_mut::<AutoQuality>().step = 1;
        app.update();
        let now = detail(&app);
        assert_eq!(
            (now.level, now.scale, now.parallax),
            (Level::High, 100, false)
        );
        // Not forced, the relief as chosen.
        app.world_mut().resource_mut::<Settings>().force_performance =
            crate::settings::ForcePerformance::Off;
        app.update();
        assert!(detail(&app).parallax);
    }

    #[test]
    fn resting_lets_the_first_frames_after_settle() {
        let mut auto = AutoQuality::default();
        auto.rest(0.0);
        // A long frame or two as the race starts (pipelines being made)
        // count for nothing.
        auto.frame(0.5, 0.5, 60.0, &SHADOWS_PACE, 3);
        auto.frame(1.5, 1.0, 60.0, &SHADOWS_PACE, 3);
        run(&mut auto, 1.5, 1.4, 60.0);
        assert_eq!(level(&auto), Level::High);
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
            ..Detail::default()
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
