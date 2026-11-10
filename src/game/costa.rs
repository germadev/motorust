//! The scenery of Costa Bits ([`Costa`]): props made with Meshy (prepared
//! by `tools/import_costa.py`, see `docs/costa-bits-props.md`) standing
//! about the bay: whitewashed houses in the town and on the rims of the
//! gorge, a beach bar, a fishing boat with its fire of skewered fish and a
//! lifeguard's tower on the beach, a sports car parked by the grid, the
//! marshal by the line with his chequered flag, a watchtower on the cape,
//! a viewpoint over the sea, boulders along the mountain road; the old
//! stone bridge over the gorge, the road running under its arch; and a
//! yellow sign before every bend of the mountain road, its arrow bending
//! the way the road does.
//!
//! Every prop's colours are in one texture, so they all share one material,
//! and every kind of prop is one mesh.
//!
//! A theme of the game's (see [`motorust_engine::world::themes`]): a
//! circuit's file writes it in its `themes` as `"costa": (...)`. Its props
//! are a pack of the engine's (`models/costa/`).

use bevy::prelude::*;

use motorust_engine::track::Track;
use motorust_engine::world::Assets3d;
use motorust_engine::world::mirror::MirrorPoint;
use motorust_engine::world::prop_pack::{PropPack, in_view, spot};
use motorust_engine::world::props::Footprint;
use motorust_engine::world::scenery::{Prop, Scenery, Side, Span, Spot};
use motorust_engine::world::terrain::Terrain;
use motorust_engine::world::themes::Theme;

/// The pack of its props, in `assets/models/`.
const PACK: &str = "costa";

/// The scenery of Costa Bits (see [`crate::game::costa`]): props made
/// with Meshy standing about, the bridge over the gorge, and signs before
/// the bends.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Costa {
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub props: &'static [Prop<CostaModel>],
    /// Stretches of the lap with a bend sign before each of their bends.
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub bend_signs: &'static [BendSigns],
}

/// The bends of a stretch of the lap, each with a sign before it (see
/// [`bend_signs`]): on `side` of the road, `lateral` metres from its
/// middle, just outside its wall.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BendSigns {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
}

/// The props of Costa Bits, made with Meshy, as a circuit's file names
/// them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum CostaModel {
    WhiteHouse,
    Chiringuito,
    EspetoBoat,
    RetroSportsCar,
    Watchtower,
    LifeguardTower,
    FlagMarshal,
    CurveSign,
    CurveSignLeft,
    BoulderA,
    BoulderB,
    BoulderC,
    Mirador,
    /// The old stone bridge over the gorge, over the road: placed
    /// `Over(along)`, the road running under its arch, where the walls
    /// close in.
    GorgeBridge,
}

/// Every prop of the theme.
#[cfg(test)]
const MODELS: [CostaModel; 14] = [
    CostaModel::WhiteHouse,
    CostaModel::Chiringuito,
    CostaModel::EspetoBoat,
    CostaModel::RetroSportsCar,
    CostaModel::Watchtower,
    CostaModel::LifeguardTower,
    CostaModel::FlagMarshal,
    CostaModel::CurveSign,
    CostaModel::CurveSignLeft,
    CostaModel::BoulderA,
    CostaModel::BoulderB,
    CostaModel::BoulderC,
    CostaModel::Mirador,
    CostaModel::GorgeBridge,
];

/// Every prop's name in `props.bin`.
fn name(model: CostaModel) -> &'static str {
    match model {
        CostaModel::WhiteHouse => "white_house",
        CostaModel::Chiringuito => "chiringuito",
        CostaModel::EspetoBoat => "espeto_boat",
        CostaModel::RetroSportsCar => "retro_sports_car",
        CostaModel::Watchtower => "watchtower",
        CostaModel::LifeguardTower => "lifeguard_tower",
        CostaModel::FlagMarshal => "flag_marshal",
        CostaModel::CurveSign => "curve_sign",
        CostaModel::CurveSignLeft => "curve_sign_left",
        CostaModel::BoulderA => "boulder_a",
        CostaModel::BoulderB => "boulder_b",
        CostaModel::BoulderC => "boulder_c",
        CostaModel::Mirador => "mirador",
        CostaModel::GorgeBridge => "gorge_bridge",
    }
}

/// Whether a prop is a landmark, to be seen from afar: no tree stands
/// between it and the road coming at it.
fn landmark(model: CostaModel) -> bool {
    matches!(
        model,
        CostaModel::Watchtower | CostaModel::Mirador | CostaModel::GorgeBridge
    )
}

/// How sharp a bend is for a sign before it: a radius under this (m),
/// the road's curvature taken over [`BEND_SPAN`] metres of it (so that
/// the wobbles of a long sweep do not count).
const BEND_RADIUS: f32 = 300.0;
const BEND_SPAN: f32 = 30.0;
/// How far before a bend its sign stands (m), and how far past the end of
/// one the next may start for it to have its own (m): bends closer than
/// that are one, for a sign, if they bend the same way.
const SIGN_BEFORE: f32 = 70.0;
const BENDS_APART: f32 = 60.0;
/// How far the bridge's foot goes below the road at its piers (m), so that
/// it never stands on air where the road climbs.
const BRIDGE_SINK: f32 = 0.5;
/// How far a house's foot goes below the ground under its middle (m), so
/// that it stands in rock that is not quite level.
const HOUSE_SINK: f32 = 1.0;
/// Where the bridge's piers stand, either side of the road's middle (m),
/// for the height of its foot.
const BRIDGE_PIERS: f32 = 9.0;

impl Theme for Costa {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        _scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        let pack = assets.packs.get(PACK);
        let mut taken = Vec::new();
        let signs = self
            .bend_signs
            .iter()
            .flat_map(|signs| bend_signs(track, signs));
        for prop in self.props.iter().copied().chain(signs) {
            taken.push(spawn_prop(
                world,
                track,
                terrain,
                &prop,
                pack,
                &mut assets.mirrored,
            ));
        }
        // No tree in the way of the landmarks, from where the road comes at
        // them.
        for prop in self.props.iter().filter(|prop| landmark(prop.model)) {
            let (position, _) = spot(track, prop);
            taken.extend(in_view(track, position));
        }
        taken
    }

    fn packs() -> &'static [&'static str] {
        &[PACK]
    }
}

/// A sign before each bend of a stretch of the lap (see [`BendSigns`]):
/// where the road starts bending tighter than [`BEND_RADIUS`],
/// [`SIGN_BEFORE`] metres before it, facing the riders coming, its arrow
/// bending the way the road does.
fn bend_signs(track: &Track, signs: &BendSigns) -> Vec<Prop<CostaModel>> {
    let lap = track.lap_length();
    let step = track.step_length();
    let length = signs.span.length(lap);
    let mut placed = Vec::new();
    let mut last_bend = f32::MIN;
    let mut last_left = None;
    let mut into = 0.0;
    while into < length {
        let along = signs.span.from + into;
        let curvature = bend(track, along);
        if curvature.abs() > 1.0 / BEND_RADIUS {
            let left = curvature > 0.0;
            if into - last_bend > BENDS_APART || last_left != Some(left) {
                let model = if left {
                    CostaModel::CurveSignLeft
                } else {
                    CostaModel::CurveSign
                };
                placed.push(Prop {
                    model,
                    spot: Spot::Road {
                        along: (along - SIGN_BEFORE).rem_euclid(lap),
                        lateral: signs.side.sign() * signs.lateral,
                    },
                    scale: 1.0,
                    turn: 0.0,
                    lean: 0.0,
                    facing_riders: true,
                });
            }
            last_bend = into;
            last_left = Some(left);
        }
        into += step;
    }
    placed
}

/// The road's curvature at `along` (1/m, positive bending left), over
/// [`BEND_SPAN`] metres of it about there.
fn bend(track: &Track, along: f32) -> f32 {
    let step = track.step_length();
    let reach = (BEND_SPAN / 2.0 / step).round() as i32;
    let sum: f32 = (-reach..=reach)
        .map(|k| track.curvature(track.index_at(along + k as f32 * step)))
        .sum();
    sum / (2 * reach + 1) as f32
}

/// The lowest of the ground under a prop round `position`, `reach` metres
/// across its middle.
fn lowest_ground(terrain: &Terrain, position: Vec2, reach: f32) -> f32 {
    [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .into_iter()
        .map(|offset| terrain.height(position + offset * 0.6 * reach))
        .fold(f32::MAX, f32::min)
}

/// A prop, standing on the lowest of the ground under its box, so that it
/// never floats; a house on the ground under its middle; the bridge on the road under its arch, a little below the
/// lowest of it at its piers. Its surface is given to the water to mirror.
/// Returns the ground it takes.
fn spawn_prop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    prop: &Prop<CostaModel>,
    props: &PropPack,
    mirrored: &mut Vec<MirrorPoint>,
) -> Footprint {
    let piece = props.piece(name(prop.model));
    let (position, rotation) = spot(track, prop);
    let (low, high) = (piece.low * prop.scale, piece.high * prop.scale);
    let reach = (high.x - low.x).max(high.z - low.z) / 2.0;
    let foot = match prop.spot {
        Spot::Over { along } => {
            let index = track.index_at(along);
            [-BRIDGE_PIERS, 0.0, BRIDGE_PIERS]
                .into_iter()
                .map(|lateral| track.surface_point(index, lateral).y)
                .fold(f32::MAX, f32::min)
                - BRIDGE_SINK
        }
        // A house on a rim stands on the rock under its middle, its front
        // out over the edge, as the houses over a gorge do; anything else on
        // the lowest of the ground under it.
        _ if prop.model == CostaModel::WhiteHouse => terrain.height(position) - HOUSE_SINK,
        _ => lowest_ground(terrain, position, reach),
    };
    let placed = Transform::from_xyz(position.x, foot, position.y)
        .with_rotation(rotation)
        .with_scale(Vec3::splat(prop.scale));
    world.spawn((
        Mesh3d(piece.mesh.clone()),
        MeshMaterial3d(props.material.clone()),
        placed,
    ));
    mirrored.extend(piece.surface.iter().map(|point| point.placed(&placed)));
    // The bridge takes the ground of its piers, not the road's.
    (position, reach)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;
    use motorust_engine::bike::model::imported::read;
    use motorust_engine::world::prop_pack::bounds;

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(motorust_engine::content::file("models/costa/props.bin"))
            .meshes
            .into_iter()
            .map(|piece| (piece.name, piece.mesh))
            .collect()
    }

    fn piece<'a>(pack: &'a [(String, Mesh)], name: &str) -> &'a Mesh {
        &pack
            .iter()
            .find(|(piece, _)| piece == name)
            .unwrap_or_else(|| panic!("no {name}"))
            .1
    }

    /// The circuits with the theme, and the theme of each.
    fn costas() -> Vec<(&'static motorust_engine::circuits::Circuit, &'static Costa)> {
        crate::game::register();
        CIRCUITS
            .iter()
            .filter_map(|circuit| Some((circuit, circuit.scenery.themes.get::<Costa>()?)))
            .collect()
    }

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        crate::game::register();
        let pack = pack();
        for model in MODELS {
            let name = name(model);
            let mesh = piece(&pack, name);
            let (low, high) = bounds(mesh);
            assert!(low.y.abs() < 1e-3, "{name} stands on the ground");
            assert!((low.x + high.x).abs() < 0.05 * (high.x - low.x), "{name}");
            assert!((low.z + high.z).abs() < 0.05 * (high.z - low.z), "{name}");
            let triangles = mesh.indices().unwrap().len() / 3;
            assert!(triangles < 4_000, "{name}: {triangles} triangles");
        }
    }

    #[test]
    fn the_bridge_s_arch_is_open_to_the_ground_and_clear_of_the_walls() {
        let pack = pack();
        let mesh = piece(&pack, name(CostaModel::GorgeBridge));
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|p| p.as_float3())
            .unwrap();
        // Up to the height of a bike and its rider, and well over it, no
        // stone nearer the road's middle than the piers' faces.
        let (_, high) = bounds(mesh);
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        let nearest = indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|corner| corner.map(|i| Vec3::from(positions[i])))
            .filter(|p| p.iter().map(|p| p.y).fold(f32::MAX, f32::min) < 12.0)
            .map(|p| {
                // How near the middle the triangle comes (it may cross it).
                let (lo, hi) = (
                    p.iter().map(|p| p.x).fold(f32::MAX, f32::min),
                    p.iter().map(|p| p.x).fold(f32::MIN, f32::max),
                );
                if lo < 0.0 && hi > 0.0 {
                    0.0
                } else {
                    lo.abs().min(hi.abs())
                }
            })
            .fold(f32::MAX, f32::min);
        assert!(nearest > 7.0, "stone {nearest} m from the middle");
        assert!(high.y > 40.0, "{} m high", high.y);
        // Where it stands, the walls close in short of its piers.
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            for prop in costa
                .props
                .iter()
                .filter(|p| p.model == CostaModel::GorgeBridge)
            {
                let Spot::Over { along } = prop.spot else {
                    panic!("{}: the bridge stands over the road", circuit.name);
                };
                let depth = 10.0;
                for k in -2..=2 {
                    let index = track.index_at(along + k as f32 * depth / 2.0);
                    let wall = track.wall_at(index);
                    assert!(
                        wall < nearest - 0.3,
                        "{}: the wall at {wall} m under the bridge",
                        circuit.name
                    );
                }
            }
        }
    }

    #[test]
    fn props_stand_off_the_road_and_the_bridge_over_it() {
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            let signs = costa
                .bend_signs
                .iter()
                .flat_map(|signs| bend_signs(&track, signs));
            for prop in costa.props.iter().copied().chain(signs) {
                if let Spot::Road { along, lateral } = prop.spot {
                    let wall = track.wall_at(track.index_at(along));
                    assert!(
                        lateral.abs() > wall + 0.5,
                        "{}: {:?} at {along} m, {lateral} m from the middle",
                        circuit.name,
                        prop.model
                    );
                }
            }
        }
    }

    #[test]
    fn a_sign_stands_before_each_bend_bending_its_way() {
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            let mut count = 0;
            for signs in costa.bend_signs {
                let placed = bend_signs(&track, signs);
                assert!(!placed.is_empty(), "{}: no sign", circuit.name);
                count += placed.len();
                for sign in placed {
                    let Spot::Road { along, .. } = sign.spot else {
                        unreachable!()
                    };
                    // The bend it warns of, ahead, bends as its arrow does.
                    let ahead = bend(&track, along + SIGN_BEFORE);
                    let left = sign.model == CostaModel::CurveSignLeft;
                    assert_eq!(ahead > 0.0, left, "{}: a sign at {along} m", circuit.name);
                }
            }
            assert!(count >= 5, "{}: {count} signs", circuit.name);
        }
    }
}
