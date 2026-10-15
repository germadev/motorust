//! How much detail the world is drawn with. With one player, all of it.
//! With two, the world is drawn twice a frame, once for each half of the
//! screen, each with its own shadows, and frame rate comes first: the
//! shadows reach less far, in fewer and coarser cascades; the grass,
//! flowers and petals are drawn less far; and the trees turn lighter
//! sooner ([`super::trees`]).

use bevy::camera::visibility::VisibilityRange;
use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::sky::Sunlight;
use crate::settings::Settings;
use crate::welcome::GameMode;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Detail {
    /// One view: everything.
    #[default]
    Full,
    /// Two views, the screen split: lighter.
    Split,
}

impl Detail {
    pub fn of(game_mode: GameMode) -> Self {
        match game_mode {
            GameMode::SinglePlayer => Detail::Full,
            GameMode::SplitScreen => Detail::Split,
        }
    }

    /// How far the grass, flowers and petals are drawn, as a share of how
    /// far they are with one view.
    pub fn reach(self) -> f32 {
        match self {
            Detail::Full => 1.0,
            Detail::Split => 0.55,
        }
    }

    /// From how far from the nearest camera (m) the trees are drawn with
    /// their first lighter mesh, and with their second (see
    /// [`super::lod::level_at`]).
    pub fn tree_levels(self) -> [f32; 2] {
        match self {
            Detail::Full => [100.0, 280.0],
            Detail::Split => [50.0, 150.0],
        }
    }

    /// The sun's shadow cascades: with one view, four, from 25 m out to
    /// 250 m; with two, each view has its own, so two, out to 90 m.
    pub fn cascades(self) -> CascadeShadowConfig {
        match self {
            Detail::Full => CascadeShadowConfigBuilder {
                first_cascade_far_bound: 25.0,
                maximum_distance: 250.0,
                ..default()
            },
            Detail::Split => CascadeShadowConfigBuilder {
                num_cascades: 2,
                first_cascade_far_bound: 20.0,
                maximum_distance: 90.0,
                ..default()
            },
        }
        .build()
    }

    /// How many texels wide each shadow cascade is: each half of the screen
    /// is half as tall, and needs no more.
    pub fn shadow_map(self) -> usize {
        match self {
            Detail::Full => 2048,
            Detail::Split => 1024,
        }
    }
}

pub struct DetailPlugin;

impl Plugin for DetailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Detail>().add_systems(
            PostUpdate,
            (choose_detail, (shadows_for_detail, reach_for_detail)).chain(),
        );
    }
}

/// How far an entity with a [`VisibilityRange`] is drawn with one view: its
/// range is this one shortened by [`Detail::reach`].
#[derive(Component, Clone, PartialEq)]
struct FullReach(VisibilityRange);

/// The detail for the players at play: split when two share the screen,
/// unless Options > Graphics says to draw it all anyway.
fn choose_detail(
    game_mode: Option<Res<GameMode>>,
    settings: Option<Res<Settings>>,
    mut detail: ResMut<Detail>,
) {
    let lighter = settings.is_none_or(|settings| settings.lighter_split_screen);
    let wanted = game_mode
        .filter(|_| lighter)
        .map_or(Detail::Full, |mode| Detail::of(*mode));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_views_are_drawn_lighter() {
        let (full, split) = (Detail::Full, Detail::Split);
        assert_eq!(Detail::of(GameMode::SinglePlayer), full);
        assert_eq!(Detail::of(GameMode::SplitScreen), split);
        assert!(split.reach() < full.reach());
        assert!(split.shadow_map() < full.shadow_map());
        let (near, far) = (split.cascades(), full.cascades());
        assert!(near.bounds.len() < far.bounds.len());
        assert!(near.bounds.last() < far.bounds.last());
        for (split, full) in split.tree_levels().iter().zip(full.tree_levels()) {
            assert!(*split < full);
        }
    }

    #[test]
    fn what_is_drawn_so_far_comes_nearer_with_two_views_and_back() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameMode>()
            .init_resource::<DirectionalLightShadowMap>()
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
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), Detail::Full.cascades().bounds);

        *app.world_mut().resource_mut::<GameMode>() = GameMode::SplitScreen;
        app.update();
        assert_eq!(
            reach(&app),
            margins(&shortened(&range, Detail::Split.reach()))
        );
        assert_eq!(cascades(&app), Detail::Split.cascades().bounds);
        assert_eq!(
            app.world().resource::<DirectionalLightShadowMap>().size,
            Detail::Split.shadow_map()
        );
        // Something new comes in shortened too.
        let flowers = app.world_mut().spawn(range.clone()).id();
        app.update();
        assert_eq!(
            app.world()
                .get::<VisibilityRange>(flowers)
                .unwrap()
                .end_margin,
            shortened(&range, Detail::Split.reach()).end_margin
        );

        // With the option off, two players see it all.
        app.insert_resource(Settings {
            lighter_split_screen: false,
            ..default()
        });
        app.update();
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), Detail::Full.cascades().bounds);
        app.world_mut()
            .resource_mut::<Settings>()
            .lighter_split_screen = true;
        app.update();
        assert_eq!(cascades(&app), Detail::Split.cascades().bounds);

        *app.world_mut().resource_mut::<GameMode>() = GameMode::SinglePlayer;
        app.update();
        assert_eq!(reach(&app), margins(&range));
        assert_eq!(cascades(&app), Detail::Full.cascades().bounds);
        assert_eq!(
            app.world().resource::<DirectionalLightShadowMap>().size,
            2048
        );
    }
}
