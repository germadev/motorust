//! A theme of the game's, `"neon"`: the props of Puerto Neón, a harbour
//! city of the future by night, made with Meshy (`assets/models/neon/`,
//! prepared by `tools/import_neon.py`): the gate over the finish line, the
//! towers, the screens on their masts and the grandstand. A circuit's file
//! says where each stands (`props`).
//!
//! They are lit of their own, a little, in their own colours, so that their
//! windows and strips of light show at night.

use bevy::prelude::*;

use motorust_engine::track::Track;
use motorust_engine::world::Assets3d;
use motorust_engine::world::prop_pack::spot;
use motorust_engine::world::props::Footprint;
use motorust_engine::world::scenery::{Prop, Scenery, Spot};
use motorust_engine::world::terrain::{SEA_LEVEL, Terrain};
use motorust_engine::world::themes::Theme;

/// The pack of its props, in `assets/models/`.
const PACK: &str = "neon";
/// How bright the props are lit of their own, over their colours.
const GLOW: f32 = 0.9;
/// How far under the ground a prop's foot goes (m): none of it floats where
/// the ground slopes.
const SUNK: f32 = 0.3;
/// How deep the ship lies in the water, and how high the airships fly (m).
const DRAUGHT: f32 = 2.5;
const AIRSHIP_HEIGHT: f32 = 110.0;

/// One of the theme's props, by its name in the pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum NeonModel {
    StartGate,
    TowerSpire,
    TowerStepped,
    TowerTwin,
    ScreenMast,
    Grandstand,
    HarbourCrane,
    ControlTower,
    FusionPlant,
    /// Afloat: its foot a little under the water.
    ContainerShip,
    /// In the air, high over where it is put.
    AirshipCobalt,
    AirshipNeon,
}

impl NeonModel {
    /// Every one of them, for the tests.
    #[cfg(test)]
    pub const ALL: [NeonModel; 12] = [
        NeonModel::StartGate,
        NeonModel::TowerSpire,
        NeonModel::TowerStepped,
        NeonModel::TowerTwin,
        NeonModel::ScreenMast,
        NeonModel::Grandstand,
        NeonModel::HarbourCrane,
        NeonModel::ControlTower,
        NeonModel::FusionPlant,
        NeonModel::ContainerShip,
        NeonModel::AirshipCobalt,
        NeonModel::AirshipNeon,
    ];

    /// Its piece's name in the pack.
    pub fn name(self) -> &'static str {
        match self {
            NeonModel::StartGate => "start_gate",
            NeonModel::TowerSpire => "tower_spire",
            NeonModel::TowerStepped => "tower_stepped",
            NeonModel::TowerTwin => "tower_twin",
            NeonModel::ScreenMast => "screen_mast",
            NeonModel::Grandstand => "grandstand",
            NeonModel::HarbourCrane => "harbour_crane",
            NeonModel::ControlTower => "control_tower",
            NeonModel::FusionPlant => "fusion_plant",
            NeonModel::ContainerShip => "container_ship",
            NeonModel::AirshipCobalt => "airship_cobalt",
            NeonModel::AirshipNeon => "airship_neon",
        }
    }
}

/// The props of a harbour city of the future, as a circuit's file places
/// them.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Neon {
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub props: &'static [Prop<NeonModel>],
}

/// How high a prop's foot stands: over the road, on the road there; else
/// on the lowest of the ground under it, a little sunk.
pub fn foot(track: &Track, terrain: &Terrain, prop: &Prop<NeonModel>, reach: f32) -> f32 {
    let (position, _) = spot(track, prop);
    match (prop.model, prop.spot) {
        (NeonModel::ContainerShip, _) => SEA_LEVEL - DRAUGHT,
        (NeonModel::AirshipCobalt | NeonModel::AirshipNeon, _) => AIRSHIP_HEIGHT,
        (_, Spot::Over { along }) => track.surface_point(track.index_at(along), 0.0).y,
        _ => {
            [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
                .into_iter()
                .map(|offset| terrain.height(position + offset * 0.6 * reach))
                .fold(f32::MAX, f32::min)
                - SUNK
        }
    }
}

impl Theme for Neon {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        _scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        let pack = assets.packs.get(PACK);
        // The pack's material, lit of its own in its own colours.
        let lit = assets.materials.get(&pack.material).cloned().map(|base| {
            assets.materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(GLOW, GLOW, GLOW),
                emissive_texture: base.base_color_texture.clone(),
                ..base
            })
        });
        let material = lit.unwrap_or_else(|| pack.material.clone());
        self.props
            .iter()
            .map(|prop| {
                let piece = pack.piece(prop.model.name());
                let size = (piece.high - piece.low) * prop.scale;
                let reach = size.x.max(size.z) / 2.0;
                let (position, rotation) = spot(track, prop);
                let foot = foot(track, terrain, prop, reach);
                world.spawn((
                    Mesh3d(piece.mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(position.x, foot, position.y)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(prop.scale)),
                ));
                // A gate over the road and what flies take no ground from
                // what stands by them.
                match (prop.model, prop.spot) {
                    (_, Spot::Over { .. })
                    | (NeonModel::AirshipCobalt | NeonModel::AirshipNeon, _) => (position, 0.0),
                    _ => (position, reach),
                }
            })
            .collect()
    }

    fn packs() -> &'static [&'static str] {
        &[PACK]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;
    use motorust_engine::bike::model::imported::read;
    use motorust_engine::track::WALL_DISTANCE;

    fn pieces() -> Vec<(String, Vec<Vec3>)> {
        let bin = motorust_engine::content::file("models/neon/props.bin");
        read(bin)
            .meshes
            .into_iter()
            .map(|piece| {
                let positions = piece
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|positions| positions.as_float3())
                    .expect("a prop has its points")
                    .iter()
                    .map(|&p| Vec3::from(p))
                    .collect();
                (piece.name, positions)
            })
            .collect()
    }

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        crate::game::register();
        let pieces = pieces();
        for model in NeonModel::ALL {
            let (_, points) = pieces
                .iter()
                .find(|(name, _)| name == model.name())
                .unwrap_or_else(|| panic!("no {} in the pack", model.name()));
            let low = points.iter().map(|p| p.y).fold(f32::MAX, f32::min);
            assert!(low.abs() < 0.01, "{}: {low}", model.name());
            assert!(points.len() < 8000, "{}: {}", model.name(), points.len());
        }
    }

    #[test]
    fn the_gate_leaves_the_road_clear_between_its_legs_and_under_its_beam() {
        crate::game::register();
        let pieces = pieces();
        let (_, gate) = pieces
            .iter()
            .find(|(name, _)| name == "start_gate")
            .unwrap();
        // Nothing of it within the barriers lower than well over a rider.
        let lowest = gate
            .iter()
            .filter(|p| p.x.abs() < WALL_DISTANCE + 0.5)
            .map(|p| p.y)
            .fold(f32::MAX, f32::min);
        assert!(lowest > 6.0, "{lowest} m over the road");
    }

    #[test]
    fn puerto_neon_s_props_stand_clear_of_its_roads_but_the_gate_over_the_line() {
        crate::game::register();
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.name_en == "Neon Harbor")
            .expect("Puerto Neón");
        let neon = circuit.scenery.themes.get::<Neon>().expect("its props");
        let track = Track::new(&circuit.layout);
        let pieces = pieces();
        assert!(
            neon.props
                .iter()
                .any(|prop| prop.model == NeonModel::StartGate
                    && matches!(prop.spot, Spot::Over { along } if along == 0.0))
        );
        for prop in neon.props {
            if matches!(prop.spot, Spot::Over { .. }) {
                assert_eq!(prop.model, NeonModel::StartGate);
                continue;
            }
            // The airships fly far over the roads.
            if matches!(
                prop.model,
                NeonModel::AirshipCobalt | NeonModel::AirshipNeon
            ) {
                continue;
            }
            let (_, points) = pieces
                .iter()
                .find(|(name, _)| name == prop.model.name())
                .unwrap();
            // One by the road faces it: its depth is what reaches towards
            // it; any other may be turned any way.
            let beside = matches!(prop.spot, Spot::Road { .. }) && !prop.facing_riders;
            let reach = points
                .iter()
                .map(|p| {
                    if beside {
                        p.z.abs()
                    } else {
                        p.x.abs().max(p.z.abs())
                    }
                })
                .fold(0.0, f32::max)
                * prop.scale;
            let (position, _) = spot(&track, prop);
            let clear = track.distance_to_centerline(position) - reach;
            assert!(
                clear > WALL_DISTANCE + 1.0,
                "{:?} at {position}: {clear} m from the road's middle",
                prop.model
            );
        }
    }
}
