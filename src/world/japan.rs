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

use bevy::prelude::*;

use std::f32::consts::TAU;

use bevy::light::{NotShadowCaster, NotShadowReceiver};

use super::Assets3d;
use super::detail::fading_out;
use super::flowers::{FlowerPlan, spawn_flowers};
use super::mirror::MirrorPoint;
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::road::WALL_HEIGHT;
use super::scenery::{
    Japan, MapleGround, PetalArea, PetalKind, Petals, Plinth, Prop, PropModel, PropRow, Scenery,
    Side, Spot, ToriiCorridor, by_road,
};
use super::terrain::{SEA_LEVEL, Terrain};
use crate::bike::model::imported::{read, texture};
use crate::geometry::{forward, random01};
use crate::track::{NARROW_EASE, Track, WALL_DISTANCE};

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
/// How fast a water wheel turns (rad/s).
const WHEEL_RATE: f32 = 0.6;
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
/// Petals in the air: how many to a square metre of ground, over a road
/// and round a point (thicker, seen from afar); how high they fall from
/// (m); how fast they fall (m/s, the slowest and the quickest); how far
/// they sway (m); how far downwind they drift as they fall, per metre
/// fallen; and how far from a camera they are drawn (m).
const PETALS_ON_ROAD: f32 = 1.0 / 32.0;
const PETALS_ROUND: f32 = 1.0 / 5.0;
const PETAL_FALL_FROM: f32 = 13.0;
const PETAL_FALL: (f32, f32) = (0.7, 1.5);
const PETAL_SWAY: (f32, f32) = (0.4, 1.6);
const PETAL_DRIFT: Vec2 = Vec2::new(0.45, 0.2);
const PETALS_SEEN: f32 = 170.0;

/// How far down the road the riders are when a prop that faces them
/// looks at them (m, see [`Prop::facing_riders`]).
const RIDERS_BACK: f32 = 110.0;

/// How big a prop stands, times the size it was made (`props.bin`), over
/// what its spot says ([`Prop::scaled`]): the wayside shrine came the size
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
fn scale_of(prop: &Prop) -> f32 {
    prop.scale * size(prop.model)
}

fn reach_of(prop: &Prop, low: Vec3, high: Vec3) -> f32 {
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

/// A piece of a pack: its mesh and the corners of the box round it, in its
/// own frame, and points over its faces for the water to mirror it (see
/// [`surface`]).
pub(super) struct Piece {
    pub(super) mesh: Handle<Mesh>,
    pub(super) low: Vec3,
    pub(super) high: Vec3,
    pub(super) surface: Vec<MirrorPoint>,
}

/// How far apart the points over a prop's faces are, at most (m of the
/// prop as made): a few to each texel of the water's map, however big.
const SURFACE_SPACING: f32 = 1.2;

/// Points over the faces of `mesh` (in its own frame), at most
/// [`SURFACE_SPACING`] apart, each with the way its face looks and the
/// colour of `atlas` there (linear): what the water mirrors of it.
fn surface(mesh: &Mesh, atlas: &Image) -> Vec<MirrorPoint> {
    let Some(positions) = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|positions| positions.as_float3())
    else {
        return Vec::new();
    };
    let uvs: Vec<[f32; 2]> = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) => uvs.clone(),
        _ => vec![[0.0, 0.0]; positions.len()],
    };
    let indices: Vec<usize> = match mesh.indices() {
        Some(indices) => indices.iter().collect(),
        None => (0..positions.len()).collect(),
    };
    let (width, height) = (atlas.width() as usize, atlas.height() as usize);
    let texels = atlas.data.as_deref().unwrap_or(&[]);
    let color = |uv: Vec2| -> [f32; 3] {
        let x = ((uv.x.fract() + 1.0).fract() * width as f32) as usize;
        let y = ((uv.y.fract() + 1.0).fract() * height as f32) as usize;
        let at = (y.min(height.saturating_sub(1)) * width + x.min(width.saturating_sub(1))) * 4;
        match texels.get(at..at + 3) {
            Some(rgb) => {
                let srgb = Color::srgb_u8(rgb[0], rgb[1], rgb[2]).to_linear();
                [srgb.red, srgb.green, srgb.blue]
            }
            None => [0.5; 3],
        }
    };
    let mut points = Vec::new();
    for &corner in indices.as_chunks::<3>().0 {
        let p = corner.map(|i| Vec3::from(positions[i]));
        let uv = corner.map(|i| Vec2::from(uvs[i]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let longest = (p[1] - p[0])
            .length()
            .max((p[2] - p[1]).length())
            .max((p[0] - p[2]).length());
        let steps = ((longest / SURFACE_SPACING).ceil() as usize).clamp(1, 64);
        // A small one, its middle.
        if steps == 1 {
            points.push(MirrorPoint {
                at: (p[0] + p[1] + p[2]) / 3.0,
                normal,
                color: color((uv[0] + uv[1] + uv[2]) / 3.0),
            });
            continue;
        }
        for i in 0..=steps {
            for j in 0..=steps - i {
                let (a, b) = (i as f32 / steps as f32, j as f32 / steps as f32);
                let c = 1.0 - a - b;
                points.push(MirrorPoint {
                    at: p[0] * c + p[1] * a + p[2] * b,
                    normal,
                    color: color(uv[0] * c + uv[1] * a + uv[2] * b),
                });
            }
        }
    }
    points
}

/// A pack of props made with Meshy: their meshes, by name, and their one
/// material, in the colours of all of them.
pub(super) struct PropPack {
    pieces: Vec<(String, Piece)>,
    pub(super) material: Handle<StandardMaterial>,
}

impl PropPack {
    pub(super) fn piece(&self, name: &str) -> &Piece {
        &self
            .pieces
            .iter()
            .find(|(piece, _)| piece == name)
            .unwrap_or_else(|| panic!("no {name} in the pack"))
            .1
    }

    /// The pack in `bin` (see `tools/import_props.py`) and its colours in
    /// `color`.
    pub(super) fn load(world: &mut World, bin: &[u8], color: &[u8]) -> Self {
        let pack = read(bin);
        let atlas = texture(color, true);
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let pieces = pack
            .meshes
            .into_iter()
            .map(|piece| {
                let (low, high) = bounds(&piece.mesh);
                let surface = surface(&piece.mesh, &atlas);
                let mesh = meshes.add(piece.mesh);
                (
                    piece.name,
                    Piece {
                        mesh,
                        low,
                        high,
                        surface,
                    },
                )
            })
            .collect();
        let atlas = world.resource_mut::<Assets<Image>>().add(atlas);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(atlas),
                perceptual_roughness: 0.85,
                reflectance: 0.3,
                ..default()
            });
        Self { pieces, material }
    }
}

/// The props of feudal Japan.
#[derive(Resource)]
pub struct ModelledProps(pub(super) PropPack);

impl FromWorld for ModelledProps {
    fn from_world(world: &mut World) -> Self {
        Self(PropPack::load(
            world,
            include_bytes!("../../assets/models/props/props.bin"),
            include_bytes!("../../assets/models/props/color.jpg"),
        ))
    }
}

/// The corners of the box round a mesh.
pub(super) fn bounds(mesh: &Mesh) -> (Vec3, Vec3) {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|positions| positions.as_float3())
        .expect("a prop has positions")
        .iter()
        .map(|&p| Vec3::from(p))
        .fold(
            (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
            |(low, high), p| (low.min(p), high.max(p)),
        )
}

/// A water wheel, turning about its axle.
#[derive(Component)]
pub struct Turning;

/// Turns the water wheels.
pub fn turn_wheels(time: Res<Time>, mut wheels: Query<&mut Transform, With<Turning>>) {
    for mut wheel in &mut wheels {
        wheel.rotate_local_z(-WHEEL_RATE * time.delta_secs());
    }
}

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

/// The scenery of feudal Japan of a circuit, if it has one. Returns the
/// ground the props take.
pub fn spawn_japan(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    flowers: FlowerPlan,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let Some(japan) = &scenery.japan else {
        return Vec::new();
    };
    spawn_corridors(world, track, terrain, japan, &assets.props.0);
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
                (&assets.props.0, &mut assets.mirrored),
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
                (&assets.props.0, &mut assets.mirrored),
                &mut plinths,
            ));
        }
    }
    plinths.spawn(world, assets);
    spawn_petals(world, track, terrain, japan.petals, assets);
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
    let meadows = spawn_flowers(world, terrain, japan.flowers, flowers, &taken, assets);
    taken.extend(meadows);
    // No tree in the way of the landmarks, from where the road comes at
    // them (the flowers grow there as anywhere).
    for prop in japan.props.iter().filter(|prop| prop.model.landmark()) {
        let (position, _) = spot(track, prop);
        taken.extend(in_view(track, position));
    }
    taken
}

/// How far a landmark is seen from (m), and how wide of straight ahead
/// (rad), coming along the road at it.
const VIEW_RANGE: f32 = 220.0;
const VIEW_ANGLE: f32 = 0.6;
/// How wide the ground is kept clear of trees either side of a line from
/// the road to a landmark (m), and every how far along the road one is
/// drawn (m).
pub(super) const VIEW_CLEAR: f32 = 4.5;
const VIEW_EVERY: f32 = 16.0;

/// The ground to keep clear of trees so that a landmark at `position` is in
/// view all the way as the road comes at it: along lines from the road,
/// where it is ahead of the riders, to it, past the road's walls (circles
/// along each, as footprints for the trees to keep out of).
pub(super) fn in_view(track: &Track, position: Vec2) -> Vec<Footprint> {
    let mut clear = Vec::new();
    let step = track.step_length();
    let every = (VIEW_EVERY / step).round().max(1.0) as usize;
    let n = track.sample_count();
    for index in (0..n).step_by(every) {
        let here = track.offset_point(index, 0.0);
        let ahead = (track.offset_point(index + 1, 0.0) - here).normalize_or_zero();
        let to = position - here;
        let distance = to.length();
        if !(WALL_DISTANCE..VIEW_RANGE).contains(&distance) || ahead.angle_to(to).abs() > VIEW_ANGLE
        {
            continue;
        }
        let toward = to / distance;
        let mut along = WALL_DISTANCE + VIEW_CLEAR;
        while along < distance {
            clear.push((here + toward * along, VIEW_CLEAR));
            along += VIEW_CLEAR * 1.5;
        }
    }
    clear
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

/// Where a prop stands and which way it faces.
pub(super) fn spot<M>(track: &Track, prop: &Prop<M>) -> (Vec2, Quat) {
    let (position, facing) = match prop.spot {
        Spot::Road { along, lateral } => {
            let index = track.index_at(along);
            let point = track.offset_point(index, lateral);
            // The road beside it, or, where it faces the riders, the road
            // they are coming up at it.
            let looks = if prop.facing_riders {
                track.index_at(along - RIDERS_BACK)
            } else {
                index
            };
            (point, facing(track.offset_point(looks, 0.0) - point))
        }
        Spot::Over { along } => {
            let (point, heading) = track.pose_at(track.index_at(along));
            (point, facing(forward(heading)))
        }
        Spot::At { position, heading } => (position, facing(forward(heading))),
    };
    (
        position,
        facing * Quat::from_rotation_y(prop.turn) * Quat::from_rotation_z(prop.lean),
    )
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
fn row_props(track: &Track, terrain: &Terrain, japan: &Japan) -> Vec<Prop> {
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
pub(super) fn prop_fence(japan: &Japan, lap: f32, along: f32, side: Side) -> bool {
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
fn stands_on(prop: &Prop) -> Option<Plinth> {
    match prop.spot {
        Spot::Road { .. } => prop.model.plinth(),
        Spot::Over { .. } | Spot::At { .. } => None,
    }
}

/// How high the top of a prop's plinth stands (m): over the road beside
/// it, so that the prop is seen whole over the wall, and over the ground
/// it stands on, never more than [`PLINTH_MOST`] where the ground falls
/// away from the road.
fn plinth_top(track: &Track, prop: &Prop, ground: f32) -> f32 {
    let Spot::Road { along, .. } = prop.spot else {
        return ground;
    };
    let road = track.surface_point(track.index_at(along), 0.0).y;
    (road + PLINTH_OVER_ROAD).clamp(ground + PLINTH_LEAST, ground + PLINTH_MOST)
}

/// Where a prop stands and which way it faces: at its spot, and, by the
/// road, never so close that it overhangs the wall, however big it is
/// (the bigger ones of the rows stand a little further out).
fn stand(track: &Track, prop: &Prop, reach: f32) -> (Vec2, Quat) {
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
    prop: &Prop,
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

/// A petal in the air: the ground under where it falls, how high it falls
/// from (m), how fast (m/s), where it starts (m into its fall), how far it
/// sways (m) and how fast (rad/s), and how fast it turns about each axis
/// (rad/s).
#[derive(Component, Clone, Copy, Debug)]
pub struct Petal {
    ground: Vec3,
    column: f32,
    fall: f32,
    start: f32,
    sway: f32,
    sway_rate: f32,
    spin: Vec3,
}

impl Petal {
    /// Where it is `time` seconds in: falling from the top of its column
    /// to the ground, over and over, swaying round and drifting downwind
    /// as it falls.
    fn position(&self, time: f32) -> Vec3 {
        let fallen = (self.start + time * self.fall).rem_euclid(self.column);
        let angle = self.start + time * self.sway_rate;
        let sway = Vec2::new(angle.sin(), (0.7 * angle).cos()) * self.sway;
        let drift = PETAL_DRIFT * fallen + sway;
        self.ground + Vec3::new(drift.x, self.column - fallen, drift.y)
    }

    fn rotation(&self, time: f32) -> Quat {
        let turn = self.spin * time + Vec3::splat(self.start);
        Quat::from_euler(EulerRot::YXZ, turn.y, turn.x, turn.z)
    }
}

/// Moves the petals in the air: those of the circuit in sight near enough a
/// camera to be drawn, the others as they come near (where each one is
/// follows from the time alone).
pub fn drift_petals(
    time: Res<Time>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut petals: Query<(&Petal, &InheritedVisibility, &mut Transform)>,
) {
    let now = time.elapsed_secs();
    let eyes: Vec<Vec3> = cameras
        .iter()
        .filter(|(camera, _)| camera.is_active)
        .map(|(_, transform)| transform.translation())
        .collect();
    // Past where they are drawn, and the column they fall down and drift
    // through.
    let reach = PETALS_SEEN + 20.0 + 2.0 * PETAL_FALL_FROM;
    petals
        .par_iter_mut()
        .for_each(|(petal, visible, mut transform)| {
            let near = eyes
                .iter()
                .any(|eye| eye.distance_squared(petal.ground) < reach * reach);
            if visible.get() && near {
                transform.translation = petal.position(now);
                transform.rotation = petal.rotation(now);
            }
        });
}

/// The petals over each area, each falling over a spot of its own, with
/// its own pace and sway (see [`Petal`]).
fn plan_petals(track: &Track, terrain: &Terrain, petals: &Petals, seed: u32) -> Vec<Petal> {
    let lap = track.lap_length();
    let (count, spot): (usize, Box<dyn Fn(f32, f32) -> Vec2>) = match petals.area {
        PetalArea::Road { span, reach } => {
            let length = span.length(lap);
            (
                (length * 2.0 * reach * PETALS_ON_ROAD) as usize,
                Box::new(move |u: f32, v: f32| {
                    let index = track.index_at(span.from + u * length);
                    track.offset_point(index, (2.0 * v - 1.0) * reach)
                }),
            )
        }
        PetalArea::Round { center, radius } => (
            (std::f32::consts::PI * radius * radius * PETALS_ROUND) as usize,
            Box::new(move |u: f32, v: f32| center + Vec2::from_angle(u * TAU) * radius * v.sqrt()),
        ),
    };
    (0..count as u32)
        .map(|k| {
            let random = |n: u32| random01(seed.wrapping_add(k * 8 + n));
            let at = spot(random(0), random(1));
            let ground = terrain.height(at).max(SEA_LEVEL);
            Petal {
                ground: Vec3::new(at.x, ground, at.y),
                column: PETAL_FALL_FROM * (0.7 + 0.3 * random(2)),
                fall: PETAL_FALL.0.lerp(PETAL_FALL.1, random(3)),
                start: random(4) * PETAL_FALL_FROM,
                sway: PETAL_SWAY.0.lerp(PETAL_SWAY.1, random(5)),
                sway_rate: 0.8 + 1.2 * random(6),
                spin: Vec3::new(
                    1.0 + 3.0 * random(7),
                    0.5 + 2.0 * random(2),
                    2.0 * random(5),
                ),
            }
        })
        .collect()
}

/// The petals in the air, one mesh and one material for each kind.
pub(super) fn spawn_petals(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    petals: &[Petals],
    assets: &mut Assets3d,
) {
    for (number, area) in petals.iter().enumerate() {
        let (mesh, color) = match area.kind {
            PetalKind::Maple => (maple_leaf_mesh(), Color::srgb(0.7, 0.22, 0.12)),
            PetalKind::Cherry => (cherry_petal_mesh(), Color::srgb(0.98, 0.93, 0.95)),
            PetalKind::Gold => (cherry_petal_mesh(), Color::srgb(0.93, 0.72, 0.24)),
        };
        let mesh = assets.meshes.add(mesh);
        // Unlit: a petal is too small, and turns too fast, to show its
        // shading, and its back would show dark.
        let material = assets.materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let seed = 300_000 + 10_000 * number as u32;
        for petal in plan_petals(track, terrain, area, seed) {
            world.spawn((
                petal,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(petal.position(0.0)).with_rotation(petal.rotation(0.0)),
                NotShadowCaster,
                fading_out(PETALS_SEEN, 20.0, false),
            ));
        }
    }
}

/// A maple leaf (m), flat, facing +Z: five pointed lobes round its middle.
fn maple_leaf_mesh() -> Mesh {
    const LOBES: usize = 5;
    const SIZE: f32 = 0.2;
    let mut builder = MeshBuilder::default();
    let white = Color::WHITE;
    for lobe in 0..LOBES {
        // The lobes fan out above the stalk, the middle one longest.
        let middle = (lobe as f32 - 2.0) * 0.55;
        let length = SIZE * (1.0 - 0.2 * (lobe as f32 - 2.0).abs());
        let tip = Vec2::from_angle(std::f32::consts::FRAC_PI_2 + middle) * length;
        let side = |turn: f32| {
            Vec2::from_angle(std::f32::consts::FRAC_PI_2 + middle + turn) * length * 0.45
        };
        for (a, b) in [(side(-0.45), tip), (tip, side(0.45))] {
            builder.triangle(
                [
                    (Vec3::ZERO, Vec2::ZERO),
                    (a.extend(0.0), Vec2::ZERO),
                    (b.extend(0.0), Vec2::ZERO),
                ],
                Vec3::Z,
                white,
            );
        }
    }
    builder.build()
}

/// A cherry petal (m), flat, facing +Z: rounded, notched at its tip.
fn cherry_petal_mesh() -> Mesh {
    const SIZE: f32 = 0.11;
    let outline = [
        Vec2::new(0.0, 0.0),
        Vec2::new(0.45, 0.35),
        Vec2::new(0.55, 0.8),
        Vec2::new(0.25, 1.0),
        Vec2::new(0.0, 0.85),
        Vec2::new(-0.25, 1.0),
        Vec2::new(-0.55, 0.8),
        Vec2::new(-0.45, 0.35),
    ]
    .map(|p| (p * SIZE * 2.0).extend(0.0));
    let mut builder = MeshBuilder::default();
    let middle = Vec3::new(0.0, 0.5 * SIZE * 2.0, 0.0);
    for i in 0..outline.len() {
        builder.triangle(
            [
                (middle, Vec2::ZERO),
                (outline[i], Vec2::ZERO),
                (outline[(i + 1) % outline.len()], Vec2::ZERO),
            ],
            Vec3::Z,
            Color::WHITE,
        );
    }
    builder.build()
}

/// The rotation that turns a prop's front (+Z) towards `direction` (x, z).
pub(super) fn facing(direction: Vec2) -> Quat {
    Quat::from_rotation_y(direction.x.atan2(direction.y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(include_bytes!("../../assets/models/props/props.bin"))
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
    fn every_prop_has_a_surface_for_the_water_to_mirror_in_its_colours() {
        let atlas = texture(include_bytes!("../../assets/models/props/color.jpg"), true);
        let mut total = 0;
        for (name, mesh) in pack() {
            let points = surface(&mesh, &atlas);
            let (low, high) = bounds(&mesh);
            assert!(!points.is_empty(), "{name}");
            // Over its faces, inside its box, a few to each square metre at
            // most.
            for point in &points {
                assert!(
                    point.at.cmpge(low - 0.01).all() && point.at.cmple(high + 0.01).all(),
                    "{name}"
                );
                assert!(
                    point.color.iter().all(|c| (0.0..=1.0).contains(c)),
                    "{name}"
                );
            }
            let size = (high - low).max_element();
            assert!(points.len() < 60_000, "{name}: {} points", points.len());
            assert!(
                points.len() as f32 > size,
                "{name}: {} points",
                points.len()
            );
            total += points.len();
        }
        assert!(total < 400_000, "{total}");
        // Not all the atlas's colour is grey: a vermilion torii is red.
        let torii = surface(
            &pack()
                .into_iter()
                .find(|(name, _)| name == "o_torii")
                .unwrap()
                .1,
            &atlas,
        );
        let red = torii
            .iter()
            .filter(|p| p.color[0] > 2.0 * p.color[1])
            .count();
        assert!(red * 3 > torii.len(), "{red} of {}", torii.len());
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
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
    fn a_petal_falls_from_the_top_of_its_column_to_the_ground_swaying() {
        let petal = Petal {
            ground: Vec3::new(10.0, 4.0, -20.0),
            column: 12.0,
            fall: 1.0,
            start: 3.0,
            sway: 1.0,
            sway_rate: 1.5,
            spin: Vec3::ONE,
        };
        let mut lowest = f32::MAX;
        for step in 0..2_000 {
            let at = petal.position(step as f32 * 0.05);
            assert!(at.y >= 4.0 && at.y <= 16.0, "{at}");
            let away = Vec2::new(at.x - 10.0, at.z + 20.0).length();
            assert!(away <= 1.0 * 2f32.sqrt() + PETAL_DRIFT.length() * 12.0 + 1e-3);
            lowest = lowest.min(at.y);
        }
        // It reaches the ground, and starts again at the top.
        assert!(lowest < 4.1);
        let before = petal.position(8.99);
        let after = petal.position(9.01);
        assert!(after.y > before.y + 11.0, "{before} {after}");
    }

    #[test]
    fn petals_drift_over_their_areas_red_and_white() {
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let lap = track.lap_length();
            for kind in [PetalKind::Maple, PetalKind::Cherry] {
                assert!(
                    japan.petals.iter().any(|petals| petals.kind == kind),
                    "{}: {kind:?}",
                    circuit.name
                );
            }
            for (number, petals) in japan.petals.iter().enumerate() {
                let planned = plan_petals(&track, &terrain, petals, number as u32 * 7);
                assert!(planned.len() > 100, "{}: {}", circuit.name, planned.len());
                for petal in &planned {
                    let at = Vec2::new(petal.ground.x, petal.ground.z);
                    assert!((petal.ground.y - terrain.height(at).max(SEA_LEVEL)).abs() < 1e-3);
                    match petals.area {
                        PetalArea::Road { span, reach } => {
                            let nearest = track.nearest(at);
                            assert!(nearest.distance <= reach + 0.5);
                            // Over the stretch, give or take its reach
                            // (round a bend, the nearest point of the road
                            // can be a little past its ends).
                            let around =
                                crate::world::scenery::span(span.from - reach, span.to + reach);
                            assert!(around.contains(track.along(&nearest), lap));
                        }
                        PetalArea::Round { center, radius } => {
                            assert!(at.distance(center) <= radius + 1e-3);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn no_tree_stands_in_the_way_of_a_landmark() {
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
        for circuit in &CIRCUITS {
            let Some(japan) = &circuit.scenery.japan else {
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
            .find(|circuit| circuit.scenery.japan.is_some())
            .expect("a circuit of feudal Japan");
        let japan = circuit.scenery.japan.as_ref().unwrap();
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
