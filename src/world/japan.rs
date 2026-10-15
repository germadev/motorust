//! The scenery of feudal Japan ([`Japan`]), as at Tsushima Isle: corridors
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

use bevy::camera::visibility::VisibilityRange;
use bevy::light::{NotShadowCaster, NotShadowReceiver};

use super::Assets3d;
use super::flowers::{FlowerPlan, spawn_flowers};
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{
    Japan, MapleGround, PetalArea, PetalKind, Petals, Prop, PropModel, Scenery, Spot, ToriiCorridor,
};
use super::terrain::{SEA_LEVEL, Terrain};
use crate::bike::model::imported::{read, texture};
use crate::geometry::{forward, random01};
use crate::track::{NARROW_EASE, Track};

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

/// A piece of the pack: its mesh and the corners of the box round it, in
/// its own frame.
struct Piece {
    mesh: Handle<Mesh>,
    low: Vec3,
    high: Vec3,
}

/// The props' meshes, by name, and their one material.
#[derive(Resource)]
pub struct ModelledProps {
    pieces: Vec<(String, Piece)>,
    material: Handle<StandardMaterial>,
}

impl ModelledProps {
    fn piece(&self, name: &str) -> &Piece {
        &self
            .pieces
            .iter()
            .find(|(piece, _)| piece == name)
            .unwrap_or_else(|| panic!("no {name} in props.bin"))
            .1
    }
}

impl FromWorld for ModelledProps {
    fn from_world(world: &mut World) -> Self {
        let pack = read(include_bytes!("../../assets/models/props/props.bin"));
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let pieces = pack
            .meshes
            .into_iter()
            .map(|piece| {
                let (low, high) = bounds(&piece.mesh);
                let mesh = meshes.add(piece.mesh);
                (piece.name, Piece { mesh, low, high })
            })
            .collect();
        let atlas = world.resource_mut::<Assets<Image>>().add(texture(
            include_bytes!("../../assets/models/props/color.jpg"),
            true,
        ));
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

/// The corners of the box round a mesh.
fn bounds(mesh: &Mesh) -> (Vec3, Vec3) {
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
    spawn_corridors(world, track, terrain, japan, assets.props);
    let mut taken: Vec<Footprint> = japan
        .props
        .iter()
        .map(|prop| spawn_prop(world, track, terrain, prop, assets.props))
        .collect();
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
    props: &ModelledProps,
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
fn spot(track: &Track, prop: &Prop) -> (Vec2, Quat) {
    let (position, facing) = match prop.spot {
        Spot::Road { along, lateral } => {
            let index = track.index_at(along);
            let point = track.offset_point(index, lateral);
            (point, facing(track.offset_point(index, 0.0) - point))
        }
        Spot::At { position, heading } => (position, facing(forward(heading))),
    };
    (position, facing * Quat::from_rotation_z(prop.lean))
}

/// A prop, standing on the ground (on the lowest of it under its box, so
/// that it never floats), or in the sea a little under the water, or
/// floating on it. A water wheel turns. Returns the ground it takes.
fn spawn_prop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    prop: &Prop,
    props: &ModelledProps,
) -> Footprint {
    let piece = props.piece(name(prop.model));
    let (position, rotation) = spot(track, prop);
    let reach = prop.scale * (piece.high.x - piece.low.x).max(piece.high.z - piece.low.z) / 2.0;
    let ground = [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .into_iter()
        .map(|offset| terrain.height(position + offset * 0.6 * reach))
        .fold(f32::MAX, f32::min);
    let foot = match prop.model {
        PropModel::FishingBoat => SEA_LEVEL - DRAUGHT,
        _ => ground.max(SEA_LEVEL - WADE),
    };
    let material = MeshMaterial3d(props.material.clone());
    if prop.model == PropModel::WaterWheel {
        // The wheel and its post have their origin at the middle of the
        // axle, as high over the ground as the post reaches down.
        let frame = props.piece(WHEEL_FRAME);
        let axle = Transform::from_xyz(position.x, foot - prop.scale * frame.low.y, position.y)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(prop.scale));
        world.spawn((Mesh3d(frame.mesh.clone()), material.clone(), axle));
        world.spawn((Mesh3d(piece.mesh.clone()), material, axle, Turning));
    } else {
        world.spawn((
            Mesh3d(piece.mesh.clone()),
            material,
            Transform::from_xyz(position.x, foot, position.y)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(prop.scale)),
        ));
    }
    (position, reach)
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

/// Moves the petals in the air.
pub fn drift_petals(time: Res<Time>, mut petals: Query<(&Petal, &mut Transform)>) {
    let now = time.elapsed_secs();
    for (petal, mut transform) in &mut petals {
        transform.translation = petal.position(now);
        transform.rotation = petal.rotation(now);
    }
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
fn spawn_petals(
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
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: PETALS_SEEN..PETALS_SEEN + 20.0,
                    use_aabb: false,
                },
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
fn facing(direction: Vec2) -> Quat {
    Quat::from_rotation_y(direction.x.atan2(direction.y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;
    use crate::track::WALL_DISTANCE;

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
    fn a_clearing_of_maple_leaves_lies_on_level_ground_ringed_with_logs() {
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
                // The leaves spill past the ring, but not far; on level
                // ground, dry, and short of the roads' walls.
                let reach = positions
                    .iter()
                    .map(|p| Vec2::new(p[0], p[2]).distance(ground.center))
                    .fold(0.0, f32::max);
                assert!(reach > ground.radius && reach < 1.3 * ground.radius);
                let (low, high) = positions
                    .iter()
                    .map(|p| terrain.height(Vec2::new(p[0], p[2])))
                    .fold((f32::MAX, f32::MIN), |(l, h), y| (l.min(y), h.max(y)));
                assert!(high - low < 0.6, "{} m from low to high", high - low);
                assert!(low > SEA_LEVEL + 2.0, "dry");
                let road = track.nearest(ground.center).distance;
                assert!(road > WALL_DISTANCE + LEAVES_SPILL * ground.radius + 2.0);
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
                let reach = prop.scale * (high.x - low.x).max(high.z - low.z) / 2.0;
                let (position, _) = spot(&track, prop);
                let road = track.nearest(position).distance;
                assert!(
                    road - reach * 0.7 > WALL_DISTANCE,
                    "{}: {:?} {road} m from a road",
                    circuit.name,
                    prop.model
                );
                let wet = terrain.height(position) < SEA_LEVEL;
                let at_sea = matches!(
                    prop.model,
                    PropModel::GreatTorii | PropModel::StiltShrine | PropModel::FishingBoat
                );
                assert_eq!(wet, at_sea, "{}: {:?}", circuit.name, prop.model);
            }
        }
    }
}
