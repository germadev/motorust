//! The scenery of feudal Japan ([`Japan`]), as at Ghost Island: corridors
//! of torii over the road, as at Fushimi Inari in Kyoto, and props made
//! with Meshy standing about (prepared by `tools/import_props.py`): a great
//! torii in the sea, a castle keep, a temple, a shrine on stilts, an
//! ancient maple, gates, statues, lanterns, boats and a water wheel that
//! turns.
//!
//! Every prop's colours are in one texture, so they all share one material,
//! and every kind of prop is one mesh: the hundred gates of a corridor draw
//! together.
//!
//! A theme of the game's (see [`crate::world::themes`]): a circuit's file
//! writes it in its `themes` as `"japan": (...)`. Its props are a pack of
//! the engine's ([`crate::world::prop_pack`], `models/props/`); its fields
//! of flowers and its petals are the scenery's (the engine's).

use bevy::prelude::*;

use std::f32::consts::TAU;

use bevy::light::NotShadowReceiver;

use crate::geometry::{forward, random01};
use crate::track::{NARROW_EASE, Track, WALL_DISTANCE};
use crate::world::Assets3d;
use crate::world::flowers::spawn_flowers;
use crate::world::mirror::MirrorPoint;
use crate::world::prop_pack::{PropPack, Turning, facing, in_view, spot};
use crate::world::props::Footprint;
use crate::world::quads::MeshBuilder;
use crate::world::road::WALL_HEIGHT;
use crate::world::scenery::{Plinth, Prop, Row, Scenery, Side, Span, Spot, by_road};
use crate::world::terrain::{SEA_LEVEL, Terrain};
use crate::world::themes::Theme;

/// The pack of its props, in `assets/models/`.
const PACK: &str = "props";

/// The scenery of feudal Japan, as at Ghost Island (see
/// [`crate::game::japan`]): corridors of torii over the road, props made
/// with Meshy standing about, and fields of flowers.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Japan {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub corridors: &'static [ToriiCorridor],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub props: &'static [Prop<PropModel>],
    /// The small ones over and over along the road (see [`PropRow`]).
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rows: &'static [PropRow],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub maple_grounds: &'static [MapleGround],
}

/// A clearing carpeted with red maple leaves, ringed with low logs, as
/// under a great maple by a lake: its middle (x, z) and the radius of its
/// ring of logs (m). Its leaves spill a little past the ring. Lay it on
/// level ground (a [`Terrace`](crate::world::scenery::Terrace)).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapleGround {
    pub center: Vec2,
    pub radius: f32,
}

/// A corridor of torii over the road, as at Fushimi Inari: where the walls
/// close in on the road along `span` (see [`crate::track::Narrowing`]), a
/// narrow gate every `every` metres; and over the `approach` metres before
/// and after it, where the road is as wide as ever, a wide gate every
/// `approach_every` metres.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToriiCorridor {
    pub span: Span,
    pub every: f32,
    pub approach: f32,
    pub approach_every: f32,
}

/// One of the props of feudal Japan, made with Meshy (see
/// [`crate::game::japan`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum PropModel {
    /// The great torii standing in the sea, as at Itsukushima.
    GreatTorii,
    CastleKeep,
    TempleHall,
    /// A shrine hall on stilts, over the water.
    StiltShrine,
    /// An ancient, gnarled maple on its boulders, with a straw rope round
    /// its trunk.
    AncientMaple,
    /// A great maple, its crown spreading far to one side over the ground,
    /// as over a clearing of its leaves.
    GreatMaple,
    /// A five-storey pagoda.
    Pagoda,
    /// A torii of bare logs, with a straw rope and bells.
    LogTorii,
    /// A tiny wayside shrine with its small torii.
    Hokora,
    /// A stone Jizō in a red bib and cap.
    Jizo,
    /// A stone guardian lion-dog.
    Komainu,
    BronzeLantern,
    /// A stone lantern (ishidōrō), mossy.
    StoneLantern,
    /// A memorial stone.
    Stele,
    /// A fire basket on a tripod.
    Brazier,
    /// Rice bales, sake barrels and crates.
    SupplyPile,
    FishingBoat,
    /// A thatched farmhouse.
    Minka,
    /// A wooden water wheel on its post, turning.
    WaterWheel,
}

impl PropModel {
    /// Whether it is one of the island's landmarks, to be seen from afar:
    /// no tree stands in the way of it (see [`crate::game::japan`]).
    pub fn landmark(self) -> bool {
        matches!(
            self,
            PropModel::GreatTorii
                | PropModel::CastleKeep
                | PropModel::TempleHall
                | PropModel::StiltShrine
                | PropModel::AncientMaple
                | PropModel::GreatMaple
                | PropModel::Pagoda
                | PropModel::LogTorii
                | PropModel::Minka
                | PropModel::WaterWheel
        )
    }

    /// What it stands on by the road, so that it shows over the wall:
    /// stone under the stone ones, wood under the wooden (see
    /// [`crate::game::japan`]). The big ones need none: they are seen
    /// over any wall as they are.
    pub fn plinth(self) -> Option<Plinth> {
        match self {
            PropModel::Jizo
            | PropModel::Komainu
            | PropModel::Stele
            | PropModel::StoneLantern
            | PropModel::BronzeLantern => Some(Plinth::Stone),
            PropModel::Brazier | PropModel::Hokora => Some(Plinth::Wood),
            _ => None,
        }
    }
}

/// The same small prop over and over by the road: one every so many
/// metres of a span, to one side, as the lanterns of a shrine's approach
/// or the jizō down a hillside road. Each stands as one of [`by_road`],
/// on its plinth, and the wall opens for it; where another road runs, or
/// where one of the props put by hand already stands, there is none.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropRow {
    /// The kinds it is made of, each in turn: one kind over and over, or
    /// several, so that a stretch is not all of one thing.
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub models: &'static [PropModel],
    pub row: Row,
}

/// The two gates over the road, and the post of the water wheel, by their
/// names in `props.bin`; the props are by [`name`].
const TORII_WIDE: &str = "torii_wide";
const TORII_ROAD: &str = "torii_road";
const WHEEL_FRAME: &str = "water_wheel_frame";
/// How far from its middle the black feet of each gate's pillars stand,
/// inside (m), where they flare inwards, as `tools/import_props.py` made
/// them: the wide one's round the road between its walls, the narrow one's
/// round a corridor's, just outside them.
const WIDE_OPENING: f32 = 11.63;
const ROAD_OPENING: f32 = 6.6;
/// The plinth a small prop by the road stands on (see [`Plinths`]): how
/// far its top stands over the road beside it (m), so that the prop is
/// seen whole over the wall ([`WALL_HEIGHT`]); how high it is over its own
/// ground at the least, where the ground stands higher than the road; and
/// at the most, so that it is a plinth and not a tower where the ground
/// falls away from the road.
const PLINTH_OVER_ROAD: f32 = WALL_HEIGHT - 0.5;
const PLINTH_LEAST: f32 = 0.3;
const PLINTH_MOST: f32 = 2.0;
/// How far its top spreads past the foot of the prop on it, how much wider
/// its own foot is than its top (it is battered), how far it is sunk into
/// the ground so that it never floats on a slope, and how many metres of
/// it a tile of its texture covers (m).
const PLINTH_MARGIN: f32 = 0.24;
const PLINTH_BATTER: f32 = 0.1;
const PLINTH_FOOTING: f32 = 0.4;
/// How far under the road the ground beside it may be for a prop of a row
/// to stand there (m): further down, not even its plinth would show it
/// over the wall, so there is none (the props put by hand stand where
/// they are put).
const PLINTH_DROP: f32 = 1.6;
const PLINTH_TILE: f32 = 1.2;
/// How far either way from a small prop by the road the wall is the
/// shrine's low fence instead (m), so that it never hides it; and how far
/// from anything already standing one of a row keeps (m).
const FENCE_OPENING: f32 = 7.0;
const PROP_APART: f32 = 2.0;
/// How far under the water the foot of a prop standing in it is (m): the
/// great torii and the shrine on stilts stand in shallow water; a boat
/// floats.
const WADE: f32 = 1.2;
const DRAUGHT: f32 = 0.6;
/// A clearing of maple leaves: how far its leaves spill past its ring of
/// logs (times its radius), how high they lie over the ground (m), and the
/// ring's logs: how many, how thick and how high their middle is (m).
const LEAVES_SPILL: f32 = 1.18;
const LEAVES_LIFT: f32 = 0.06;
const RING_LOGS: usize = 30;
const LOG_THICKNESS: f32 = 0.38;
const LOG_HEIGHT: f32 = 0.3;
/// How big a prop stands, times the size it was made (`props.bin`), over
/// what its spot says ([`Prop::scale`]): the wayside shrine came the size
/// of a doll's house, and the torii of logs stands on a hilltop, to be
/// seen from far down the road.
fn size(model: PropModel) -> f32 {
    match model {
        PropModel::Hokora | PropModel::LogTorii => 1.8,
        _ => 1.0,
    }
}

/// How big a prop stands in all, and how far it reaches from its middle
/// (m), from the box round its mesh.
fn scale_of(prop: &Prop<PropModel>) -> f32 {
    prop.scale * size(prop.model)
}

fn reach_of(prop: &Prop<PropModel>, low: Vec3, high: Vec3) -> f32 {
    scale_of(prop) * (high.x - low.x).max(high.z - low.z) / 2.0
}

/// Every prop's name in `props.bin`.
fn name(model: PropModel) -> &'static str {
    match model {
        PropModel::GreatTorii => "o_torii",
        PropModel::CastleKeep => "castle_keep",
        PropModel::TempleHall => "temple_hall",
        PropModel::StiltShrine => "stilt_shrine",
        PropModel::AncientMaple => "ancient_maple",
        PropModel::GreatMaple => "great_maple",
        PropModel::Pagoda => "pagoda",
        PropModel::LogTorii => "log_torii",
        PropModel::Hokora => "hokora",
        PropModel::Jizo => "jizo",
        PropModel::Komainu => "komainu",
        PropModel::BronzeLantern => "bronze_lantern",
        PropModel::StoneLantern => "stone_lantern",
        PropModel::Stele => "stele",
        PropModel::Brazier => "brazier",
        PropModel::SupplyPile => "supply_pile",
        PropModel::FishingBoat => "fishing_boat",
        PropModel::Minka => "minka",
        PropModel::WaterWheel => "water_wheel",
    }
}

/// Every model, for the tests.
#[cfg(test)]
const MODELS: [PropModel; 19] = [
    PropModel::GreatTorii,
    PropModel::CastleKeep,
    PropModel::TempleHall,
    PropModel::StiltShrine,
    PropModel::AncientMaple,
    PropModel::GreatMaple,
    PropModel::Pagoda,
    PropModel::LogTorii,
    PropModel::Hokora,
    PropModel::Jizo,
    PropModel::Komainu,
    PropModel::BronzeLantern,
    PropModel::StoneLantern,
    PropModel::Stele,
    PropModel::Brazier,
    PropModel::SupplyPile,
    PropModel::FishingBoat,
    PropModel::Minka,
    PropModel::WaterWheel,
];

/// A gate of a corridor of torii: the point of the lap it stands over and
/// whether it is a wide one (over the road as wide as ever) or a narrow one
/// (where the walls close in).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Gate {
    index: usize,
    wide: bool,
}

/// The gates of a corridor (see [`ToriiCorridor`]): the narrow ones along
/// it, and the wide ones along its approaches, clear of where the walls
/// close in and open out.
fn corridor_gates(track: &Track, corridor: &ToriiCorridor) -> Vec<Gate> {
    let lap = track.lap_length();
    let span = corridor.span.length(lap);
    let mut gates = Vec::new();
    let mut at = |along: f32, wide: bool| {
        gates.push(Gate {
            index: track.index_at(along.rem_euclid(lap)),
            wide,
        });
    };
    let narrow = (span / corridor.every).floor() as usize;
    for k in 0..=narrow {
        at(corridor.span.from + k as f32 * corridor.every, false);
    }
    // A little way clear of where the walls start closing in.
    let clear = NARROW_EASE + 4.0;
    let wide = (corridor.approach / corridor.approach_every).floor() as usize;
    for k in 0..=wide {
        let into = k as f32 * corridor.approach_every;
        at(corridor.span.from - clear - into, true);
        at(corridor.span.from + span + clear + into, true);
    }
    gates
}

impl Theme for Japan {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        spawn_japan(world, track, terrain, self, scenery, assets)
    }

    fn opens_wall(&self, lap: f32, along: f32, side: Side) -> bool {
        prop_fence(self, lap, along, side)
    }

    fn packs() -> &'static [&'static str] {
        &[PACK]
    }
}

/// The scenery of feudal Japan of a circuit, and the circuit's flowers
/// among it, clear of its props. Returns the ground the props take.
fn spawn_japan(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    japan: &Japan,
    scenery: &Scenery,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let pack = assets.packs.get(PACK);
    spawn_corridors(world, track, terrain, japan, pack);
    let mut plinths = Plinths::default();
    let mut taken: Vec<Footprint> = japan
        .props
        .iter()
        .map(|prop| {
            spawn_prop(
                world,
                track,
                terrain,
                prop,
                (pack, &mut assets.mirrored),
                &mut plinths,
            )
        })
        .collect();
    // The rows' props, each where nothing stands already.
    for prop in row_props(track, terrain, japan) {
        let (position, _) = spot(track, &prop);
        let clear = taken
            .iter()
            .all(|&(at, reach)| at.distance(position) > reach + PROP_APART);
        if clear {
            taken.push(spawn_prop(
                world,
                track,
                terrain,
                &prop,
                (pack, &mut assets.mirrored),
                &mut plinths,
            ));
        }
    }
    plinths.spawn(world, assets);
    if !japan.maple_grounds.is_empty() {
        let material = assets.materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            reflectance: 0.2,
            ..default()
        });
        for ground in japan.maple_grounds {
            // In the sun, not in the shade of the great maple's crown: the
            // carpet of its leaves glows under it.
            world.spawn((
                Mesh3d(assets.meshes.add(maple_ground_mesh(terrain, ground))),
                MeshMaterial3d(material.clone()),
                NotShadowReceiver,
            ));
            taken.push((ground.center, ground.radius * LEAVES_SPILL + 2.0));
        }
    }
    if let Some(flowers) = assets.flowers.take() {
        let meadows = spawn_flowers(world, terrain, scenery.flowers, flowers, &taken, assets);
        taken.extend(meadows);
    }
    // No tree in the way of the landmarks, from where the road comes at
    // them (the flowers grow there as anywhere).
    for prop in japan.props.iter().filter(|prop| prop.model.landmark()) {
        let (position, _) = spot(track, prop);
        taken.extend(in_view(track, position));
    }
    taken
}

/// A clearing of maple leaves (see [`MapleGround`]): a carpet of reds,
/// leaf by leaf darker or brighter, lying on the ground with a ragged edge
/// a little past its ring of logs, and the ring, log after log.
fn maple_ground_mesh(terrain: &Terrain, ground: &MapleGround) -> Mesh {
    const RINGS: usize = 14;
    const ROUND: usize = 64;
    let mut builder = MeshBuilder::default();
    let seed = (ground.center.x.abs() * 7.0 + ground.center.y.abs() * 13.0) as u32;
    let leaf = |k: u32| {
        let random = |n: u32| random01(seed.wrapping_add(k * 4 + n));
        // Autumn reds, from crimson to rust, lighter and duller than the
        // leaves still on the tree, as fallen leaves are in the sun.
        let red = Color::srgb(
            0.66 + 0.16 * random(0),
            0.27 + 0.09 * random(1),
            0.19 + 0.07 * random(2),
        );
        // Some leaves lie in the shade of others.
        if random(3) < 0.25 {
            red.darker(0.25)
        } else {
            red
        }
    };
    // The leaves' edge: ragged, each way round its own distance.
    let edge: Vec<f32> = (0..ROUND)
        .map(|i| ground.radius * (LEAVES_SPILL - 0.1 + 0.2 * random01(seed + 9_000 + i as u32)))
        .collect();
    let point = |ring: usize, i: usize| {
        let angle = (i % ROUND) as f32 / ROUND as f32 * TAU;
        let reach = edge[i % ROUND] * ring as f32 / RINGS as f32;
        let at = ground.center + Vec2::from_angle(angle) * reach;
        Vec3::new(at.x, terrain.height(at) + LEAVES_LIFT, at.y)
    };
    let vertex = |ring: usize, i: usize| {
        let k = (ring * ROUND + i % ROUND) as u32;
        (point(ring, i), Vec2::ZERO, leaf(k))
    };
    for ring in 0..RINGS {
        for i in 0..ROUND {
            let (a, b) = (vertex(ring, i), vertex(ring, i + 1));
            let (c, d) = (vertex(ring + 1, i), vertex(ring + 1, i + 1));
            if ring > 0 {
                builder.shaded_triangle([a, c, b], Vec3::Y);
            }
            builder.shaded_triangle([b, c, d], Vec3::Y);
        }
    }
    // The ring of logs, lying end to end round the clearing.
    let wood = Color::srgb(0.24, 0.17, 0.11);
    let log = |i: usize| {
        let angle = i as f32 / RING_LOGS as f32 * TAU;
        let at = ground.center + Vec2::from_angle(angle) * ground.radius;
        Vec3::new(at.x, terrain.height(at) + LOG_HEIGHT, at.y)
    };
    for i in 0..RING_LOGS {
        let shade = if i % 2 == 0 { wood } else { wood.darker(0.06) };
        builder.beam(log(i), log(i + 1), LOG_THICKNESS, LOG_THICKNESS, shade);
    }
    builder.build()
}

/// The gates of every corridor of torii, standing over the road, upright
/// and square to it, their pillars beside its walls.
fn spawn_corridors(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    japan: &Japan,
    props: &PropPack,
) {
    for corridor in japan.corridors {
        for gate in corridor_gates(track, corridor) {
            let (piece, opening) = if gate.wide {
                (props.piece(TORII_WIDE), WIDE_OPENING)
            } else {
                (props.piece(TORII_ROAD), ROAD_OPENING)
            };
            let (point, heading) = track.pose_at(gate.index);
            // On the lower of the ground under its two pillars, so that
            // neither floats.
            let pillar = opening + 1.5;
            let foot = [-pillar, pillar]
                .into_iter()
                .map(|lateral| {
                    let road = track.surface_point(gate.index, lateral).y;
                    road.min(terrain.height(track.offset_point(gate.index, lateral)))
                })
                .fold(f32::MAX, f32::min);
            world.spawn((
                Mesh3d(piece.mesh.clone()),
                MeshMaterial3d(props.material.clone()),
                Transform::from_xyz(point.x, foot - 0.3, point.y)
                    .with_rotation(facing(forward(heading))),
            ));
        }
    }
}

/// The plinths the small props by the road stand on: one mesh for the
/// stone ones and one for the wooden, so that all of each draw together.
#[derive(Default)]
struct Plinths {
    stone: MeshBuilder,
    wood: MeshBuilder,
}

impl Plinths {
    /// A plinth under the prop at `at`, square to it (`rotation`), its top
    /// `half` metres from its middle and at `top`, from the ground at
    /// `ground`: battered, and sunk into the ground so that it never
    /// floats on a slope.
    fn block(&mut self, kind: Plinth, at: Vec2, rotation: Quat, half: f32, ground: f32, top: f32) {
        let mesh = match kind {
            Plinth::Stone => &mut self.stone,
            Plinth::Wood => &mut self.wood,
        };
        let point = |on: Vec2, height: f32| Vec3::new(on.x, height, on.y);
        let yaw = Quat::from_rotation_y(rotation.to_euler(EulerRot::YXZ).0);
        let corners = |spread: f32| {
            [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(x, z): (f32, f32)| {
                let turned = yaw * Vec3::new(x * spread, 0.0, z * spread);
                at + Vec2::new(turned.x, turned.z)
            })
        };
        let tops = corners(half);
        let feet = corners(half + PLINTH_BATTER);
        let foot = ground - PLINTH_FOOTING;
        let high = (top - foot) / PLINTH_TILE;
        for side in 0..4 {
            let next = (side + 1) % 4;
            let run = tops[side].distance(tops[next]) / PLINTH_TILE;
            let middle = (tops[side] + tops[next]) / 2.0 - at;
            mesh.quad(
                [
                    (point(feet[side], foot), Vec2::new(0.0, high)),
                    (point(feet[next], foot), Vec2::new(run, high)),
                    (point(tops[next], top), Vec2::new(run, 0.0)),
                    (point(tops[side], top), Vec2::ZERO),
                ],
                Vec3::new(middle.x, 0.0, middle.y),
                Color::WHITE,
            );
        }
        mesh.quad(
            tops.map(|corner| (point(corner, top), (corner - at) / PLINTH_TILE)),
            Vec3::Y,
            Color::WHITE,
        );
    }

    /// Spawns them, of the walls' stone and of the roadside's planks.
    fn spawn(self, world: &mut ChildSpawnerCommands, assets: &mut Assets3d) {
        for (built, material) in [
            (
                self.stone,
                StandardMaterial {
                    base_color_texture: Some(assets.textures.wall.clone()),
                    perceptual_roughness: 0.95,
                    ..default()
                },
            ),
            (
                self.wood,
                StandardMaterial {
                    base_color: Color::srgb(0.8, 0.68, 0.56),
                    base_color_texture: Some(assets.textures.planks.clone()),
                    perceptual_roughness: 0.9,
                    ..default()
                },
            ),
        ] {
            if built.is_empty() {
                continue;
            }
            let mesh = assets.meshes.add(built.build());
            let material = assets.materials.add(material);
            world.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
        }
    }
}

/// Where the props of a row stand, metres along the lap, and which kind
/// stands at each: one every [`Row::every`] metres of its span, from half
/// a step into it (as a row of lamps is laid out), round the finish line
/// and all, the row's kinds in turn.
fn row_steps(row: &PropRow, lap: f32) -> impl Iterator<Item = (f32, PropModel)> + use<> {
    let every = row.row.every.max(1.0);
    let count = (row.row.span.length(lap) / every).floor() as usize;
    let from = row.row.span.from + every / 2.0;
    let models = row.models;
    (0..count)
        .filter(move |_| !models.is_empty())
        .map(move |k| {
            (
                (from + k as f32 * every).rem_euclid(lap),
                models[k % models.len()],
            )
        })
}

/// The props of the rows ([`PropRow`]): the same small one every so many
/// metres along a span, and none where another road runs.
fn row_props(track: &Track, terrain: &Terrain, japan: &Japan) -> Vec<Prop<PropModel>> {
    let lap = track.lap_length();
    japan
        .rows
        .iter()
        .flat_map(|row: &PropRow| {
            row_steps(row, lap)
                .map(|(along, model)| by_road(model, along, row.row.side.sign() * row.row.lateral))
        })
        .filter(|prop| {
            let Spot::Road { along, .. } = prop.spot else {
                return false;
            };
            let (position, _) = spot(track, prop);
            let ground = terrain.height(position);
            let road = track.surface_point(track.index_at(along), 0.0).y;
            ground > SEA_LEVEL
                && ground > road - PLINTH_DROP
                && !track.on_other_road(0, position, |road, nearest| {
                    road.wall_at(nearest.index) + 2.0
                })
        })
        .collect()
}

/// The side of the road a prop stands on, by how far it is from the
/// centre line (positive to the left, as [`by_road`] has it).
fn side_at(lateral: f32) -> Side {
    if lateral >= 0.0 {
        Side::Left
    } else {
        Side::Right
    }
}

/// Whether the wall at `along` on `side` is the shrine's low fence
/// because a small prop stands there (one put by hand or one of a row):
/// a wall as high as the prop would hide it, plinth and all.
fn prop_fence(japan: &Japan, lap: f32, along: f32, side: Side) -> bool {
    let apart = |at: f32| {
        let gap = (along - at).rem_euclid(lap);
        gap.min(lap - gap)
    };
    let by_hand = japan.props.iter().any(|prop| match prop.spot {
        Spot::Road { along: at, lateral } => {
            prop.model.plinth().is_some() && side_at(lateral) == side && apart(at) < FENCE_OPENING
        }
        Spot::Over { .. } | Spot::At { .. } => false,
    });
    by_hand
        || japan.rows.iter().any(|row| {
            row.row.side == side
                && row_steps(row, lap)
                    .any(|(at, model)| model.plinth().is_some() && apart(at) < FENCE_OPENING)
        })
}

/// What a prop stands on: a plinth under a small one by the road, so that
/// it is seen over the wall; nothing under those standing away from it,
/// where no wall hides them.
fn stands_on(prop: &Prop<PropModel>) -> Option<Plinth> {
    match prop.spot {
        Spot::Road { .. } => prop.model.plinth(),
        Spot::Over { .. } | Spot::At { .. } => None,
    }
}

/// How high the top of a prop's plinth stands (m): over the road beside
/// it, so that the prop is seen whole over the wall, and over the ground
/// it stands on, never more than [`PLINTH_MOST`] where the ground falls
/// away from the road.
fn plinth_top(track: &Track, prop: &Prop<PropModel>, ground: f32) -> f32 {
    let Spot::Road { along, .. } = prop.spot else {
        return ground;
    };
    let road = track.surface_point(track.index_at(along), 0.0).y;
    (road + PLINTH_OVER_ROAD).clamp(ground + PLINTH_LEAST, ground + PLINTH_MOST)
}

/// Where a prop stands and which way it faces: at its spot, and, by the
/// road, never so close that it overhangs the wall, however big it is
/// (the bigger ones of the rows stand a little further out).
fn stand(track: &Track, prop: &Prop<PropModel>, reach: f32) -> (Vec2, Quat) {
    let (position, rotation) = spot(track, prop);
    let Spot::Road { along, lateral } = prop.spot else {
        return (position, rotation);
    };
    let least = WALL_DISTANCE + reach * 0.7 + 0.2;
    if lateral.abs() >= least {
        return (position, rotation);
    }
    let out = track.offset_point(track.index_at(along), lateral.signum() * least);
    (out, rotation)
}

/// A prop, standing on the ground (on the lowest of it under its box, so
/// that it never floats), or on a plinth by the road ([`Plinths`]), or in
/// the sea a little under the water, or floating on it. A water wheel
/// turns. Its surface is given to the water to mirror (`mirrored`).
/// Returns the ground it takes.
fn spawn_prop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    prop: &Prop<PropModel>,
    (props, mirrored): (&PropPack, &mut Vec<MirrorPoint>),
    plinths: &mut Plinths,
) -> Footprint {
    let piece = props.piece(name(prop.model));
    let scale = scale_of(prop);
    let reach = reach_of(prop, piece.low, piece.high);
    let (position, rotation) = stand(track, prop, reach);
    let ground = [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .into_iter()
        .map(|offset| terrain.height(position + offset * 0.6 * reach))
        .fold(f32::MAX, f32::min);
    let foot = match prop.model {
        PropModel::FishingBoat => SEA_LEVEL - DRAUGHT,
        _ => ground.max(SEA_LEVEL - WADE),
    };
    // A small one by the road stands on its plinth, which takes a little
    // more ground than the prop itself.
    let (foot, taken) = match stands_on(prop) {
        Some(kind) => {
            let top = plinth_top(track, prop, foot);
            plinths.block(kind, position, rotation, reach + PLINTH_MARGIN, foot, top);
            (top, reach + PLINTH_MARGIN + PLINTH_BATTER)
        }
        None => (foot, reach),
    };
    let material = MeshMaterial3d(props.material.clone());
    if prop.model == PropModel::WaterWheel {
        // The wheel and its post have their origin at the middle of the
        // axle, as high over the ground as the post reaches down.
        let frame = props.piece(WHEEL_FRAME);
        let axle = Transform::from_xyz(position.x, foot - scale * frame.low.y, position.y)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(scale));
        world.spawn((Mesh3d(frame.mesh.clone()), material.clone(), axle));
        world.spawn((Mesh3d(piece.mesh.clone()), material, axle, Turning));
        for part in [frame, piece] {
            mirrored.extend(part.surface.iter().map(|point| point.placed(&axle)));
        }
    } else {
        let placed = Transform::from_xyz(position.x, foot, position.y)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(scale));
        world.spawn((Mesh3d(piece.mesh.clone()), material, placed));
        mirrored.extend(piece.surface.iter().map(|point| point.placed(&placed)));
    }
    (position, taken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bike::model::imported::read;
    use crate::circuits::CIRCUITS;
    use crate::world::prop_pack::{RIDERS_BACK, VIEW_CLEAR, bounds};

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(crate::content::file("models/props/props.bin"))
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

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        let pack = pack();
        let names = MODELS
            .iter()
            .map(|&model| name(model))
            .chain([TORII_WIDE, TORII_ROAD]);
        for name in names {
            let mesh = piece(&pack, name);
            let (low, high) = bounds(mesh);
            if name != "water_wheel" {
                assert!(low.y.abs() < 1e-3, "{name} stands on the ground");
            }
            // Centred on its origin (the great maple on its trunk, under
            // one side of its crown).
            if name != "water_wheel" && name != "great_maple" {
                assert!((low.x + high.x).abs() < 0.05 * (high.x - low.x), "{name}");
                assert!((low.z + high.z).abs() < 0.05 * (high.z - low.z), "{name}");
            }
            let triangles = mesh.indices().unwrap().len() / 3;
            assert!(triangles < 6_000, "{name}: {triangles} triangles");
            assert!(high.y - low.y < 32.0, "{name}: {} m", high.y - low.y);
        }
        // The water wheel and its post about the axle, the post reaching
        // down to the ground and the wheel as low.
        let (wheel_low, wheel_high) = bounds(piece(&pack, "water_wheel"));
        let (post_low, _) = bounds(piece(&pack, WHEEL_FRAME));
        assert!(post_low.y < -2.0, "{post_low}");
        assert!(
            (wheel_low.y - post_low.y).abs() < 0.4,
            "{wheel_low} {post_low}"
        );
        assert!((wheel_low.y + wheel_high.y).abs() < 0.3, "round its axle");
        assert!((wheel_low.x + wheel_high.x).abs() < 0.3, "round its axle");
    }

    #[test]
    fn the_gates_leave_the_road_clear_under_their_beams() {
        let pack = pack();
        for (name, opening) in [(TORII_WIDE, WIDE_OPENING), (TORII_ROAD, ROAD_OPENING)] {
            let mesh = piece(&pack, name);
            let (_, high) = bounds(mesh);
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|p| p.as_float3())
                .unwrap();
            // Below the tie beam nothing stands nearer the middle than the
            // feet of the pillars, and they come that near; and the beams
            // are high over a rider's head.
            let nearest = positions
                .iter()
                .filter(|p| p[1] < 0.6 * high.y)
                .map(|p| p[0].abs())
                .fold(f32::MAX, f32::min);
            assert!((nearest - opening).abs() < 0.05, "{name}: {nearest} m");
            let beams = positions
                .iter()
                .filter(|p| p[0].abs() < opening - 0.05)
                .map(|p| p[1])
                .fold(f32::MAX, f32::min);
            assert!(beams > 8.0, "{name}: beams {beams} m up");
        }
    }

    #[test]
    fn the_corridor_of_torii_stands_round_the_road_beside_its_walls() {
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            for corridor in japan.corridors {
                let gates = corridor_gates(&track, corridor);
                let narrow = gates.iter().filter(|gate| !gate.wide).count();
                assert!(narrow > 40, "{}: {narrow} narrow gates", circuit.name);
                for gate in gates {
                    // Each gate's feet just outside the walls where it
                    // stands: the narrow ones where the walls have closed
                    // in, the wide ones where they are as wide as ever.
                    let wall = track.wall_at(gate.index);
                    let opening = if gate.wide {
                        WIDE_OPENING
                    } else {
                        ROAD_OPENING
                    };
                    let outside = opening - wall;
                    assert!(
                        (0.05..0.5).contains(&outside),
                        "{}: a gate at {} m, {outside} m outside the wall",
                        circuit.name,
                        gate.index as f32 * track.step_length()
                    );
                    if gate.wide {
                        assert_eq!(wall, WALL_DISTANCE);
                    }
                }
            }
        }
    }

    #[test]
    fn a_clearing_of_maple_leaves_faces_the_road_ringed_with_logs() {
        let mut checked = 0;
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            for ground in japan.maple_grounds {
                let mesh = maple_ground_mesh(&terrain, ground);
                let positions = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|p| p.as_float3())
                    .unwrap();
                // Every leaf lies just over the ground, and every log on it.
                for p in positions {
                    let over = p[1] - terrain.height(Vec2::new(p[0], p[2]));
                    let top = LOG_HEIGHT + LOG_THICKNESS;
                    assert!((-0.05..top).contains(&over), "{over} m over the ground");
                }
                // The leaves spill past the ring, but not far; on ground
                // rising smoothly away from the road, so that it faces the
                // riders, dry, and short of the roads' walls.
                let reach = positions
                    .iter()
                    .map(|p| Vec2::new(p[0], p[2]).distance(ground.center))
                    .fold(0.0, f32::max);
                assert!(reach > ground.radius && reach < 1.3 * ground.radius);
                let nearest = track.nearest(ground.center);
                let away = (ground.center - nearest.point).normalize();
                let across = away.perp();
                let at = |along: f32, side: f32| {
                    terrain.height(ground.center + (away * along + across * side) * ground.radius)
                };
                let rise = at(0.8, 0.0) - at(-0.8, 0.0);
                assert!((2.0..6.0).contains(&rise), "{rise} m up away from the road");
                assert!((at(0.0, 0.8) - at(0.0, -0.8)).abs() < 0.6, "level across");
                // A plane: its middle half way between its sides.
                let middle = at(0.0, 0.0) - (at(0.4, 0.0) + at(-0.4, 0.0)) / 2.0;
                assert!(middle.abs() < 0.3, "{middle} m off a plane");
                let low = positions
                    .iter()
                    .map(|p| terrain.height(Vec2::new(p[0], p[2])))
                    .fold(f32::MAX, f32::min);
                assert!(low > SEA_LEVEL + 2.0, "dry");
                assert!(nearest.distance > WALL_DISTANCE + LEAVES_SPILL * ground.radius + 2.0);
                checked += 1;
            }
        }
        assert!(checked > 0, "some circuit has a clearing of maple leaves");
    }

    #[test]
    fn no_tree_stands_in_the_way_of_a_landmark() {
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            for prop in japan.props.iter().filter(|prop| prop.model.landmark()) {
                let (position, _) = spot(&track, prop);
                let clear = in_view(&track, position);
                // Ground kept clear from past the road's walls up to it,
                // from some way along the road coming at it.
                let near = clear
                    .iter()
                    .map(|&(at, _)| at.distance(position))
                    .fold(f32::MAX, f32::min);
                assert!(near < 2.0 * VIEW_CLEAR, "{:?}: {near} m", prop.model);
                let from = clear
                    .iter()
                    .map(|&(at, _)| at.distance(position))
                    .fold(0.0, f32::max);
                assert!(from > 60.0, "{:?} seen from {from} m", prop.model);
            }
        }
    }

    #[test]
    fn props_stand_clear_of_the_roads_and_in_the_water_where_they_should() {
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let pack = pack();
            for prop in japan.props {
                let (low, high) = bounds(piece(&pack, name(prop.model)));
                let reach = reach_of(prop, low, high);
                let (position, _) = stand(&track, prop, reach);
                let road = track.nearest(position).distance;
                assert!(
                    road - reach * 0.7 > WALL_DISTANCE,
                    "{}: {:?} {road} m from a road",
                    circuit.name,
                    prop.model
                );
                // Boats at sea, shrines on stilts in the water or on land,
                // and everything else on land (the great torii on its
                // islet).
                let wet = terrain.height(position) < SEA_LEVEL;
                match prop.model {
                    PropModel::FishingBoat => assert!(wet, "{}: a boat ashore", circuit.name),
                    PropModel::StiltShrine => {}
                    _ => assert!(!wet, "{}: {:?} in the water", circuit.name, prop.model),
                }
            }
        }
    }

    #[test]
    fn a_small_prop_by_the_road_stands_on_a_plinth_that_shows_it_over_the_wall() {
        let pack = pack();
        let mut stood = 0;
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let lap = track.lap_length();
            let rows = row_props(&track, &terrain, japan);
            for prop in japan.props.iter().chain(rows.iter()) {
                let Spot::Road { along, lateral } = prop.spot else {
                    continue;
                };
                if stands_on(prop).is_none() {
                    continue;
                }
                let (position, _) = spot(&track, prop);
                let ground = terrain.height(position);
                let top = plinth_top(&track, prop, ground);
                let road = track.surface_point(track.index_at(along), 0.0).y;
                let said = format!("{:?} at {along} m of {}", prop.model, circuit.name);
                // It stands on its plinth, which is a plinth and not a
                // tower where the ground falls away from the road.
                assert!(top > ground + PLINTH_LEAST - 1e-3, "{said}: sunk");
                assert!(top < ground + PLINTH_MOST + 1e-3, "{said}: a tower");
                // Its head shows over the wall, where the riders see it.
                let (low, high) = bounds(piece(&pack, name(prop.model)));
                let head = top + scale_of(prop) * (high.y - low.y);
                assert!(head > road + WALL_HEIGHT + 0.2, "{said}: hidden");
                // And the wall there is the shrine's low fence.
                assert!(
                    prop_fence(japan, lap, along, side_at(lateral)),
                    "{said}: walled"
                );
                stood += 1;
            }
        }
        assert!(stood > 60, "only {stood} small props by the road");
    }

    #[test]
    fn a_prop_that_faces_the_riders_looks_down_the_road_they_come_up() {
        for circuit in CIRCUITS.iter() {
            let Some(japan) = circuit.scenery.themes.get::<Japan>() else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let mut faced = 0;
            for prop in japan.props.iter().filter(|prop| prop.facing_riders) {
                let Spot::Road { along, .. } = prop.spot else {
                    panic!("{:?} faces the riders of no road", prop.model);
                };
                let (position, rotation) = spot(&track, prop);
                let looks = rotation * Vec3::Z;
                let looks = Vec2::new(looks.x, looks.z).normalize();
                // It looks at the riders, not at the road beside it: the
                // way to them is nearer its own than the way to the road.
                let toward =
                    |at: f32| (track.offset_point(track.index_at(at), 0.0) - position).normalize();
                let riders = toward(along - RIDERS_BACK);
                let beside = toward(along);
                assert!(
                    looks.dot(riders) > looks.dot(beside) + 0.01,
                    "{:?} of {} looks beside it, not up the road",
                    prop.model,
                    circuit.name
                );
                assert!(looks.dot(riders) > 0.99, "{:?} looks askew", prop.model);
                faced += 1;
            }
            assert!(faced > 0 || japan.props.is_empty());
        }
    }

    #[test]
    fn the_rows_fill_the_road_without_crowding_what_stands_already() {
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.scenery.themes.get::<Japan>().is_some())
            .expect("a circuit of feudal Japan");
        let japan = circuit.scenery.themes.get::<Japan>().unwrap();
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        let pack = pack();
        let rows = row_props(&track, &terrain, japan);
        assert!(rows.len() > 50, "only {} props in the rows", rows.len());
        // Of every kind, and none of them the whole roadside: the small
        // props are seen as often as one another.
        let mut kinds: Vec<(PropModel, usize)> = Vec::new();
        for prop in &rows {
            match kinds.iter_mut().find(|(model, _)| *model == prop.model) {
                Some((_, count)) => *count += 1,
                None => kinds.push((prop.model, 1)),
            }
        }
        assert!(kinds.len() >= 6, "only {} kinds in the rows", kinds.len());
        for (model, count) in &kinds {
            assert!(*count >= 3, "only {count} of {model:?}");
            assert!(
                5 * count <= 2 * rows.len(),
                "{count} of {model:?} of {}",
                rows.len()
            );
        }
        for prop in &rows {
            let (low, high) = bounds(piece(&pack, name(prop.model)));
            let (position, _) = stand(&track, prop, reach_of(prop, low, high));
            // None of them in the water nor on another road.
            assert!(
                terrain.height(position) > SEA_LEVEL,
                "{:?} in the water",
                prop.model
            );
            // Each stands clear of the roads, as those put by hand do.
            let reach = reach_of(prop, low, high);
            let road = track.nearest(position).distance;
            assert!(
                road - reach * 0.7 > WALL_DISTANCE,
                "{:?} {road} m from the road",
                prop.model
            );
        }
    }
}
