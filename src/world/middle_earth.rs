//! The scenery of Middle-earth ([`MiddleEarth`]), as at Fellowship Road:
//! props made with Meshy standing about (prepared by
//! `tools/import_fellowship.py`): hobbit holes set into grassy banks with
//! their chimneys rising out of them, Bag End under its oak, the Green
//! Dragon and the mill of Bywater, its wheel turning, Bree's gate over the
//! road and its inn, the ruins on Weathertop, the stone trolls, the halls of
//! Rivendell, the Doors of Durin open over the road into Moria, the drawing
//! of ithildin glowing on them, outside and on their leaves within, a
//! mallorn, boats on the river and the kings of the Argonath standing in
//! it; and, as in feudal Japan (see
//! [`super::japan`], whose props are placed alike), fields of flowers and
//! leaves drifting down.
//!
//! Every prop's colours are in one texture, so they all share one material,
//! and every kind of prop is one mesh (but the ithildin on the Doors of
//! Durin and its glow, each in a material of its own).

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::Assets3d;
use super::flowers::{FlowerPlan, spawn_flowers};
use super::japan::{PropPack, Turning, in_view, spawn_petals, spot};
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{MiddleEarthModel, Prop, Scenery, Spot};
use super::terrain::{GRASS_TILE, SEA_LEVEL, Terrain};
use crate::geometry::{fbm, smoothstep};
use crate::track::Track;

/// The mill's wheel and the chimney over a hobbit hole, by their names in
/// `props.bin`; the props are by [`name`].
const WHEEL: &str = "mill_wheel";
const CHIMNEY: &str = "chimney";
/// The drawing of ithildin on the Doors of Durin (a design of our own, see
/// `tools/import_fellowship.py`), and the glow round its lines: pieces of
/// `props.bin` going with the doors.
const ITHILDIN: &str = "durin_ithildin";
const GLOW: &str = "durin_glow";
/// How far in front of the rock at the mouth of the tunnel into Moria the
/// back of the Doors of Durin stands (m): a little, so the stars drawn on
/// the rock over them show.
const DOORS_OFF_ROCK: f32 = 0.05;
/// The ithildin, silver with a touch of blue, as it shines in the film,
/// brighter than anything the sun lights (linear); and its glow, added over
/// the stone round its lines.
const ITHILDIN_LIGHT: LinearRgba = LinearRgba::rgb(2.2, 2.6, 3.6);
const GLOW_COLOR: Color = Color::srgb(0.16, 0.22, 0.42);
/// How far under the water a boat's keel floats (m), and how far the foot
/// of a king of the Argonath stands under the river (m).
const DRAUGHT: f32 = 0.6;
const WADE: f32 = 1.2;
/// How far either side of the middle of the road the towers of Bree's gate
/// stand (m), past the walls: the ground under them sets its foot.
const GATE_TOWERS: f32 = 14.5;

/// The little hill a hobbit hole is dug into, a dome over its rooms behind
/// its front: its top so much higher than the hole's (m), its middle so far
/// behind the hole's back (m), round from its middle out to so far past the
/// hole's sides (m), so far behind its middle (m) and so far before the
/// hole's front (m), its top the share of that round its middle where it
/// is all but level; the dome cut away round the hole, flush with its
/// sides, the cut opening out so much for each metre before its front (m)
/// and softened over so many metres (m); how far its edge sinks into the
/// ground at least (m), how far under the ground its mesh lies where the
/// ground is higher (m), and over how many metres the two round into one
/// another (m); and how finely its mesh is cut (m).
const HILL_RISE: f32 = 3.0;
const HILL_MIDDLE: f32 = 1.0;
const HILL_SIDES: f32 = 6.5;
const HILL_BEHIND: f32 = 13.0;
const HILL_BEFORE: f32 = 2.5;
const HILL_TOP: f32 = 0.3;
const CUT_FLARE: f32 = 1.5;
const CUT_SOFT: f32 = 0.7;
const BANK_SINK: f32 = 2.5;
const BANK_UNDER: f32 = 0.3;
const BANK_BLEND: f32 = 2.0;
const BANK_STEP: f32 = 0.5;
/// Before each hole, its garden: a low fence of weathered pickets round it,
/// this far out either side of its front and this far before it (m), posts
/// this far apart (m), so high (m), and a gap for its gate in the middle
/// (rad either side).
const GARDEN_SIDE: f32 = 1.2;
const GARDEN_DEPTH: f32 = 5.5;
const PICKET_EVERY: f32 = 0.55;
const PICKET_HEIGHT: f32 = 0.9;
const GARDEN_GATE: f32 = 0.22;
const PICKET_WOOD: Color = Color::srgb(0.66, 0.6, 0.5);
/// The footpaths of bare earth between the holes and to the road: how wide
/// (m), cut in slices so long (m), over the ground by so much (m), the
/// sand's grain tinted earth brown, and how far trees and flowers keep off
/// their middle (m).
const PATH_WIDTH: f32 = 2.2;
const PATH_STEP: f32 = 1.0;
const PATH_OVER: f32 = 0.07;
const PATH_EARTH: Color = Color::srgb(0.5, 0.43, 0.36);
const PATH_CLEAR: f32 = 2.5;
/// On the banks, as in Hobbiton: washing hung out on a line between two
/// posts, and bunting strung between two poles, every other hole one or
/// the other; the colours of the clothes and of the flags.
const CLOTHES: [Color; 4] = [
    Color::srgb(0.95, 0.94, 0.9),
    Color::srgb(0.75, 0.2, 0.16),
    Color::srgb(0.3, 0.42, 0.66),
    Color::srgb(0.92, 0.85, 0.6),
];
const FLAGS: [Color; 5] = [
    Color::srgb(0.85, 0.18, 0.15),
    Color::srgb(0.95, 0.78, 0.2),
    Color::srgb(0.25, 0.6, 0.3),
    Color::srgb(0.25, 0.42, 0.8),
    Color::srgb(0.95, 0.94, 0.9),
];
const POST_WOOD: Color = Color::srgb(0.42, 0.32, 0.22);

/// Every prop's name in `props.bin`.
fn name(model: MiddleEarthModel) -> &'static str {
    match model {
        MiddleEarthModel::HobbitDoorGreen => "hobbit_door_green",
        MiddleEarthModel::HobbitDoorYellow => "hobbit_door_yellow",
        MiddleEarthModel::HobbitDoorRed => "hobbit_door_red",
        MiddleEarthModel::HobbitDoorBlue => "hobbit_door_blue",
        MiddleEarthModel::BagEnd => "bag_end",
        MiddleEarthModel::PartyTree => "party_tree",
        MiddleEarthModel::BagEndOak => "bag_end_oak",
        MiddleEarthModel::Mill => "mill",
        MiddleEarthModel::GreenDragon => "green_dragon",
        MiddleEarthModel::GreenDragonSign => "green_dragon_sign",
        MiddleEarthModel::GandalfCart => "gandalf_cart",
        MiddleEarthModel::LogPile => "log_pile",
        MiddleEarthModel::Scarecrow => "scarecrow",
        MiddleEarthModel::RootTree => "root_tree",
        MiddleEarthModel::Ferry => "ferry",
        MiddleEarthModel::BreeGate => "bree_gate",
        MiddleEarthModel::PrancingPony => "prancing_pony",
        MiddleEarthModel::BreeHouseA => "bree_house_a",
        MiddleEarthModel::BreeHouseB => "bree_house_b",
        MiddleEarthModel::WeathertopRuin => "weathertop_ruin",
        MiddleEarthModel::RuinArch => "ruin_arch",
        MiddleEarthModel::StoneTrolls => "stone_trolls",
        MiddleEarthModel::LastHomelyHouse => "last_homely_house",
        MiddleEarthModel::ElvenTower => "elven_tower",
        MiddleEarthModel::Gazebo => "gazebo",
        MiddleEarthModel::ElvenLamp => "elven_lamp",
        MiddleEarthModel::DurinDoors => "durin_doors",
        MiddleEarthModel::MallornFlet => "mallorn_flet",
        MiddleEarthModel::SwanShip => "swan_ship",
        MiddleEarthModel::ElvenShip => "elven_ship",
        MiddleEarthModel::ElvenBoat => "elven_boat",
        MiddleEarthModel::ArgonathA => "argonath_a",
        MiddleEarthModel::ArgonathB => "argonath_b",
        MiddleEarthModel::AmonHenSeat => "amon_hen_seat",
    }
}

/// Every model, for the tests.
#[cfg(test)]
const MODELS: [MiddleEarthModel; 34] = [
    MiddleEarthModel::HobbitDoorGreen,
    MiddleEarthModel::HobbitDoorYellow,
    MiddleEarthModel::HobbitDoorRed,
    MiddleEarthModel::HobbitDoorBlue,
    MiddleEarthModel::BagEnd,
    MiddleEarthModel::PartyTree,
    MiddleEarthModel::BagEndOak,
    MiddleEarthModel::Mill,
    MiddleEarthModel::GreenDragon,
    MiddleEarthModel::GreenDragonSign,
    MiddleEarthModel::GandalfCart,
    MiddleEarthModel::LogPile,
    MiddleEarthModel::Scarecrow,
    MiddleEarthModel::RootTree,
    MiddleEarthModel::Ferry,
    MiddleEarthModel::BreeGate,
    MiddleEarthModel::PrancingPony,
    MiddleEarthModel::BreeHouseA,
    MiddleEarthModel::BreeHouseB,
    MiddleEarthModel::WeathertopRuin,
    MiddleEarthModel::RuinArch,
    MiddleEarthModel::StoneTrolls,
    MiddleEarthModel::LastHomelyHouse,
    MiddleEarthModel::ElvenTower,
    MiddleEarthModel::Gazebo,
    MiddleEarthModel::ElvenLamp,
    MiddleEarthModel::DurinDoors,
    MiddleEarthModel::MallornFlet,
    MiddleEarthModel::SwanShip,
    MiddleEarthModel::ElvenShip,
    MiddleEarthModel::ElvenBoat,
    MiddleEarthModel::ArgonathA,
    MiddleEarthModel::ArgonathB,
    MiddleEarthModel::AmonHenSeat,
];

/// Where a prop stands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Footing {
    /// On the ground: on the lowest of it under the prop, so that it never
    /// floats.
    Ground,
    /// Set into a grassy bank that rises over it (a hobbit hole).
    Bank,
    /// Afloat on the water.
    Afloat,
    /// Drawn up on a shore: on the ground or, where it is under the water,
    /// afloat.
    Shore,
    /// Standing on a rock in the river, its foot a little under the water
    /// where the rock is under it.
    Wading,
    /// Over the road, on the lower of the ground under its two sides.
    OverRoad,
}

fn footing(model: MiddleEarthModel) -> Footing {
    match model {
        MiddleEarthModel::HobbitDoorGreen
        | MiddleEarthModel::HobbitDoorYellow
        | MiddleEarthModel::HobbitDoorRed
        | MiddleEarthModel::HobbitDoorBlue
        | MiddleEarthModel::BagEnd => Footing::Bank,
        MiddleEarthModel::SwanShip | MiddleEarthModel::ElvenShip | MiddleEarthModel::Ferry => {
            Footing::Afloat
        }
        MiddleEarthModel::ElvenBoat => Footing::Shore,
        MiddleEarthModel::ArgonathA | MiddleEarthModel::ArgonathB => Footing::Wading,
        MiddleEarthModel::BreeGate | MiddleEarthModel::DurinDoors => Footing::OverRoad,
        _ => Footing::Ground,
    }
}

/// Whether a prop is one of the landmarks, to be seen from afar: no tree
/// stands between it and the road coming at it. The hobbit holes too, high
/// across the water from the road.
fn landmark(model: MiddleEarthModel) -> bool {
    matches!(
        model,
        MiddleEarthModel::HobbitDoorGreen
            | MiddleEarthModel::HobbitDoorYellow
            | MiddleEarthModel::HobbitDoorRed
            | MiddleEarthModel::HobbitDoorBlue
            | MiddleEarthModel::BagEnd
            | MiddleEarthModel::BagEndOak
            | MiddleEarthModel::PartyTree
            | MiddleEarthModel::Mill
            | MiddleEarthModel::GreenDragon
            | MiddleEarthModel::RootTree
            | MiddleEarthModel::PrancingPony
            | MiddleEarthModel::WeathertopRuin
            | MiddleEarthModel::StoneTrolls
            | MiddleEarthModel::LastHomelyHouse
            | MiddleEarthModel::ElvenTower
            | MiddleEarthModel::Gazebo
            | MiddleEarthModel::DurinDoors
            | MiddleEarthModel::MallornFlet
            | MiddleEarthModel::SwanShip
            | MiddleEarthModel::ElvenShip
            | MiddleEarthModel::ArgonathA
            | MiddleEarthModel::ArgonathB
            | MiddleEarthModel::AmonHenSeat
    )
}

/// The props of Middle-earth.
#[derive(Resource)]
pub struct FellowshipProps(PropPack);

impl FromWorld for FellowshipProps {
    fn from_world(world: &mut World) -> Self {
        Self(PropPack::load(
            world,
            include_bytes!("../../assets/models/fellowship/props.bin"),
            include_bytes!("../../assets/models/fellowship/color.jpg"),
        ))
    }
}

/// The scenery of Middle-earth of a circuit, if it has one. Returns the
/// ground the props take.
pub fn spawn_middle_earth(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    flowers: FlowerPlan,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let Some(middle_earth) = &scenery.middle_earth else {
        return Vec::new();
    };
    let props = &assets.fellowship.0;
    // Shining of themselves: the ithildin, and its glow, added over the
    // stone.
    let gate = GateMaterials {
        ithildin: assets.materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: ITHILDIN_LIGHT,
            ..default()
        }),
        glow: assets.materials.add(StandardMaterial {
            base_color: GLOW_COLOR,
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..default()
        }),
    };
    let mut banks = BankMesh::default();
    let mut taken: Vec<Footprint> = middle_earth
        .props
        .iter()
        .map(|prop| spawn_prop(world, track, terrain, prop, props, &gate, &mut banks))
        .collect();
    if !banks.dressing.is_empty() {
        // Cloth and flags, seen from either side.
        let material = assets.materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            cull_mode: None,
            double_sided: true,
            ..default()
        });
        let dressing = std::mem::take(&mut banks.dressing).build();
        world.spawn((
            Mesh3d(assets.meshes.add(dressing)),
            MeshMaterial3d(material),
        ));
    }
    if !banks.positions.is_empty() {
        // The banks are grass, as the ground they rise from.
        let material = assets.materials.add(StandardMaterial {
            base_color_texture: Some(assets.textures.grass.clone()),
            perceptual_roughness: 0.9,
            reflectance: 0.18,
            ..default()
        });
        world.spawn((
            Mesh3d(assets.meshes.add(banks.mesh())),
            MeshMaterial3d(material),
        ));
    }
    if let Some(paths) = path_mesh(middle_earth.paths, terrain) {
        // Bare earth: the sand's grain, tinted brown by the vertices.
        let material = assets.materials.add(StandardMaterial {
            base_color_texture: Some(assets.textures.sand.clone()),
            perceptual_roughness: 0.95,
            reflectance: 0.12,
            ..default()
        });
        world.spawn((Mesh3d(assets.meshes.add(paths)), MeshMaterial3d(material)));
    }
    taken.extend(path_footprints(middle_earth.paths));
    spawn_petals(world, track, terrain, middle_earth.petals, assets);
    let meadows = spawn_flowers(
        world,
        terrain,
        middle_earth.flowers,
        flowers,
        &taken,
        assets,
    );
    taken.extend(meadows);
    // No tree in the way of the landmarks, from where the road comes at
    // them.
    for prop in middle_earth.props.iter().filter(|p| landmark(p.model)) {
        let (position, _) = spot(track, prop);
        taken.extend(in_view(track, position));
    }
    taken
}

/// The lowest of the ground under a prop round `position`, `reach` metres
/// across its middle.
fn lowest_ground(terrain: &Terrain, position: Vec2, reach: f32) -> f32 {
    [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .into_iter()
        .map(|offset| terrain.height(position + offset * 0.6 * reach))
        .fold(f32::MAX, f32::min)
}

/// How high a prop's foot stands (see [`Footing`]).
fn foot(track: &Track, terrain: &Terrain, prop: &Prop<MiddleEarthModel>, reach: f32) -> f32 {
    let (position, _) = spot(track, prop);
    let ground = || lowest_ground(terrain, position, reach);
    match footing(prop.model) {
        Footing::Ground | Footing::Bank => ground(),
        Footing::Afloat => SEA_LEVEL - DRAUGHT,
        Footing::Shore => ground().max(SEA_LEVEL - DRAUGHT),
        Footing::Wading => ground().max(SEA_LEVEL - WADE),
        Footing::OverRoad => {
            let along = match prop.spot {
                Spot::Over { along } => along,
                _ => track.along(&track.nearest(position)),
            };
            let index = track.index_at(along);
            [-GATE_TOWERS, GATE_TOWERS]
                .into_iter()
                .map(|lateral| {
                    let road = track.surface_point(index, lateral).y;
                    road.min(terrain.height(track.offset_point(index, lateral)))
                })
                .fold(f32::MAX, f32::min)
                - 0.3
        }
    }
}

/// The materials of the ithildin on the Doors of Durin and of its glow.
struct GateMaterials {
    ithildin: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

/// A prop, standing as its [`Footing`] says; the mill with its wheel
/// turning, a hobbit hole in its bank (added to `banks`) with its chimney
/// over it, the Doors of Durin against the rock at the mouth of the tunnel,
/// their ithildin shining. Returns the
/// ground it takes: the middle of its box and how far it reaches from it.
fn spawn_prop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    prop: &Prop<MiddleEarthModel>,
    props: &PropPack,
    gate: &GateMaterials,
    banks: &mut BankMesh,
) -> Footprint {
    let piece = props.piece(name(prop.model));
    let (position, rotation) = spot(track, prop);
    let (low, high) = (piece.low * prop.scale, piece.high * prop.scale);
    let reach = (high.x - low.x).max(high.z - low.z) / 2.0;
    let foot = foot(track, terrain, prop, reach);
    let material = MeshMaterial3d(props.material.clone());
    // The mill and its wheel have their origin at the middle of the axle,
    // as high over the ground as the mill reaches down.
    let lift = if prop.model == MiddleEarthModel::Mill {
        -low.y
    } else {
        0.0
    };
    let mut transform = Transform::from_xyz(position.x, foot + lift, position.y)
        .with_rotation(rotation)
        .with_scale(Vec3::splat(prop.scale));
    // The Doors of Durin stand where they are placed with their back, at
    // the rock of the mouth of the tunnel, their front to the riders coming.
    if prop.model == MiddleEarthModel::DurinDoors {
        transform.translation += rotation * Vec3::Z * (DOORS_OFF_ROCK - low.z);
    }
    world.spawn((Mesh3d(piece.mesh.clone()), material.clone(), transform));
    if prop.model == MiddleEarthModel::DurinDoors {
        for (part, paint) in [(ITHILDIN, &gate.ithildin), (GLOW, &gate.glow)] {
            world.spawn((
                Mesh3d(props.piece(part).mesh.clone()),
                MeshMaterial3d(paint.clone()),
                transform,
            ));
        }
    }
    if prop.model == MiddleEarthModel::Mill {
        let wheel = props.piece(WHEEL);
        world.spawn((
            Mesh3d(wheel.mesh.clone()),
            material.clone(),
            transform,
            Turning,
        ));
    }
    let bank = Bank::of(low, high);
    if footing(prop.model) == Footing::Bank {
        let base = Vec3::new(position.x, foot, position.y);
        let bank = bank.sunk(base, rotation, terrain);
        banks.add(&bank, base, rotation, terrain);
        // A chimney rising out of the bank, behind and to one side.
        let chimney = props.piece(CHIMNEY);
        let at = Vec2::new(bank.half_width * 0.45, bank.back - 3.5);
        let up = bank.height(at) - 0.4;
        world.spawn((
            Mesh3d(chimney.mesh.clone()),
            material,
            Transform::from_translation(base + rotation * Vec3::new(at.x, up, at.y)),
        ));
    }
    // A hobbit hole takes the ground of its bank.
    let (middle, reach) = if footing(prop.model) == Footing::Bank {
        let (x0, z0, z1) = bank.extent();
        let middle = (z0 + z1) / 2.0;
        let reach = x0.max((z1 - z0) / 2.0);
        (Vec3::new(0.0, 0.0, middle), reach)
    } else {
        ((low + high) / 2.0, reach)
    };
    let turned = transform.translation - Vec3::new(position.x, transform.translation.y, position.y)
        + rotation * Vec3::new(middle.x, 0.0, middle.z);
    (position + Vec2::new(turned.x, turned.z), reach)
}

/// The points of a footpath every `PATH_STEP` along it, from point to point.
fn path_samples(path: &[Vec2]) -> Vec<Vec2> {
    let mut samples = Vec::new();
    for pair in path.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let steps = (a.distance(b) / PATH_STEP).ceil().max(1.0) as usize;
        samples.extend((0..steps).map(|k| a.lerp(b, k as f32 / steps as f32)));
    }
    samples.extend(path.last().copied());
    samples
}

/// The footpaths as one mesh: a strip of earth along each, in slices across
/// it of three points (its edges and its middle) each on the ground there,
/// a little over it; its colour varying along it as bare earth does.
fn path_mesh(paths: &[&[Vec2]], terrain: &Terrain) -> Option<Mesh> {
    let mut mesh = MeshBuilder::default();
    for path in paths {
        let samples = path_samples(path);
        let mut along = 0.0;
        let slices: Vec<[Vec3; 3]> = (0..samples.len())
            .map(|k| {
                let (before, after) = (
                    samples[k.saturating_sub(1)],
                    samples[(k + 1).min(samples.len() - 1)],
                );
                let across = (after - before).normalize_or_zero().perp() * PATH_WIDTH / 2.0;
                [-1.0, 0.0, 1.0].map(|side| {
                    let p = samples[k] + across * side;
                    Vec3::new(p.x, terrain.height(p) + PATH_OVER, p.y)
                })
            })
            .collect();
        for (k, pair) in slices.windows(2).enumerate() {
            let step = samples[k].distance(samples[k + 1]);
            let (v0, v1) = (along / 4.0, (along + step) / 4.0);
            along += step;
            let shade = 0.86 + 0.2 * fbm(samples[k] / 6.0, 2);
            let earth = Color::LinearRgba(PATH_EARTH.to_linear() * shade);
            for (i, u) in [(0, 0.0), (1, 0.275)] {
                mesh.quad(
                    [
                        (pair[0][i], Vec2::new(u, v0)),
                        (pair[0][i + 1], Vec2::new(u + 0.275, v0)),
                        (pair[1][i + 1], Vec2::new(u + 0.275, v1)),
                        (pair[1][i], Vec2::new(u, v1)),
                    ],
                    Vec3::Y,
                    earth,
                );
            }
        }
    }
    (!mesh.is_empty()).then(|| mesh.build())
}

/// The ground the footpaths take, for the trees and the flowers to keep
/// off: circles along them.
fn path_footprints(paths: &[&[Vec2]]) -> Vec<Footprint> {
    paths
        .iter()
        .flat_map(|path| path_samples(path).into_iter().step_by(2))
        .map(|at| (at, PATH_CLEAR))
        .collect()
}

/// The grassy hill a hobbit hole is dug into, in its own frame (its front
/// facing +Z, its foot at the origin): a little round hill over its rooms,
/// higher than its front, falling away behind it and to its sides and,
/// either side of it, coming down round its front, which stands flush in a
/// cut opening out before it.
#[derive(Clone, Copy)]
struct Bank {
    /// How far the front reaches either side of its middle (m).
    half_width: f32,
    /// How far back its back stands (m, negative: towards -Z), and how far
    /// forward its front (m).
    back: f32,
    front: f32,
    /// How high the top of the hole is (m).
    top: f32,
    /// How far under its foot its edge is (m).
    sink: f32,
}

impl Bank {
    /// The bank of a hobbit hole from `low` to `high` (the corners of its
    /// box, scaled), its edge `BANK_SINK` under its foot.
    fn of(low: Vec3, high: Vec3) -> Self {
        Self {
            half_width: high.x.max(-low.x),
            back: low.z,
            front: high.z,
            top: high.y,
            sink: BANK_SINK,
        }
    }

    /// How far it reaches, in its frame: either side of its middle (m), and
    /// back and forward (z, m).
    fn extent(&self) -> (f32, f32, f32) {
        (
            self.half_width + HILL_SIDES,
            self.back - HILL_MIDDLE - HILL_BEHIND,
            self.front + HILL_BEFORE,
        )
    }

    /// The same bank with its foot at `base`, turned by `rotation`, its edge
    /// sunk deeper where the ground round it falls away, so that all round
    /// it is `BANK_UNDER` under the ground at least.
    fn sunk(self, base: Vec3, rotation: Quat, terrain: &Terrain) -> Self {
        let (x0, z0, z1) = self.extent();
        let across = (2.0 * x0 / BANK_STEP).ceil() as usize;
        let deep = (z1 - z0) / BANK_STEP;
        let edge = (0..=across)
            .flat_map(|k| {
                let x = -x0 + 2.0 * x0 * k as f32 / across as f32;
                [Vec2::new(x, z0), Vec2::new(x, z1)]
            })
            .chain((0..=deep.ceil() as usize).flat_map(|k| {
                let z = z0 + (z1 - z0) * k as f32 / deep.ceil();
                [Vec2::new(-x0, z), Vec2::new(x0, z)]
            }));
        let sink = edge
            .map(|at| {
                let p = base + rotation * Vec3::new(at.x, 0.0, at.y);
                base.y - terrain.height(Vec2::new(p.x, p.z)) + BANK_UNDER
            })
            .fold(self.sink, f32::max);
        Self { sink, ..self }
    }

    /// The height of the bank at `at` (x, z) in its frame (m): down to
    /// `-sink` at its edge, into the ground.
    fn height(&self, at: Vec2) -> f32 {
        // The dome: round about its middle, behind the hole, its top all
        // but level and its sides falling away, rounded, to its edge.
        let middle = self.back - HILL_MIDDLE;
        let across = at.x / (self.half_width + HILL_SIDES);
        let along = if at.y > middle {
            (at.y - middle) / (self.front + HILL_BEFORE - middle)
        } else {
            (middle - at.y) / HILL_BEHIND
        };
        let up = 1.0 - smoothstep(HILL_TOP, 1.0, (across * across + along * along).sqrt());
        let hill = (self.top + HILL_RISE) * up - self.sink * (1.0 - up) * (1.0 - up);
        // Cut away round the hole from just behind its back, flush with its
        // sides, and opening out before its front.
        if at.y <= self.back + 0.15 {
            return hill;
        }
        let cut = self.half_width + 0.15 + CUT_FLARE * (at.y - self.front).max(0.0);
        let away = 1.0 - smoothstep(cut, cut + CUT_SOFT, at.x.abs());
        hill.lerp(-self.sink, away)
    }
}

/// The greater of `a` and `b`, rounded where they are within `k` of one
/// another, so that two surfaces meet without a crease.
fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.max(b) + h * h * k / 4.0
}

/// The banks of all the hobbit holes, as one mesh: positions, normals,
/// colours (the grass's, as the ground's there) and texture coordinates
/// (the ground's).
#[derive(Default)]
struct BankMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    /// The washing lines and the bunting on the banks, and how many banks
    /// there are so far.
    dressing: MeshBuilder,
    count: usize,
}

impl BankMesh {
    /// The bank of a hobbit hole whose foot is at `base`, turned by
    /// `rotation`.
    fn add(&mut self, bank: &Bank, base: Vec3, rotation: Quat, terrain: &Terrain) {
        self.dress(bank, base, rotation, terrain);
        self.count += 1;
        let (x0, z0, z1) = bank.extent();
        let x0 = -x0;
        let columns = ((-2.0 * x0) / BANK_STEP).ceil() as usize + 1;
        let rows = ((z1 - z0) / BANK_STEP).ceil() as usize + 1;
        let start = self.positions.len() as u32;
        for row in 0..rows {
            for column in 0..columns {
                let at = Vec2::new(x0 + column as f32 * BANK_STEP, z0 + row as f32 * BANK_STEP);
                // Rounded into the ground round it, which it lies just
                // under where that is higher, so that it rises out of the
                // land as a hill does.
                let surface = |at: Vec2| {
                    let p = base + rotation * Vec3::new(at.x, bank.height(at), at.y);
                    let ground = terrain.height(Vec2::new(p.x, p.z)) - BANK_UNDER;
                    Vec3::new(p.x, smooth_max(p.y, ground, BANK_BLEND), p.z)
                };
                let p = surface(at);
                let e = 0.1;
                let across = surface(at + Vec2::X * e) - surface(at - Vec2::X * e);
                let along = surface(at + Vec2::Y * e) - surface(at - Vec2::Y * e);
                let n = along.cross(across).normalize();
                self.positions.push(p.to_array());
                self.normals.push(n.to_array());
                self.colors.push(terrain.grass_color(p, n));
                // The grass laid along its slopes as well as over its top.
                self.uvs.push([p.x / GRASS_TILE, (p.z - p.y) / GRASS_TILE]);
            }
        }
        for row in 0..rows - 1 {
            for column in 0..columns - 1 {
                let a = start + (row * columns + column) as u32;
                let b = a + 1;
                let c = a + columns as u32;
                let d = c + 1;
                self.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }

    /// On the top of a bank, washing on a line between two posts, or, on
    /// every other one, bunting between two poles higher up behind.
    fn dress(&mut self, bank: &Bank, base: Vec3, rotation: Quat, terrain: &Terrain) {
        // On the bank, or on the hill it is dug into where that is higher.
        let on = |x: f32, z: f32, up: f32| {
            let at = base + rotation * Vec3::new(x, bank.height(Vec2::new(x, z)), z);
            let ground = at.y.max(terrain.height(Vec2::new(at.x, at.z)));
            Vec3::new(at.x, ground + up, at.z)
        };
        let facing = rotation * Vec3::Z;
        let washing = self.count.is_multiple_of(2);
        let (span, behind, tall) = if washing {
            (2.6, bank.back - 4.5, 1.9)
        } else {
            (4.2, bank.back - 8.0, 4.0)
        };
        let feet = [on(-span, behind, -0.3), on(span, behind, -0.3)];
        let tops = [on(-span, behind, tall), on(span, behind, tall)];
        for (foot, top) in feet.into_iter().zip(tops) {
            self.dressing.beam(foot, top, 0.1, 0.1, POST_WOOD);
        }
        // The line, sagging a little, in pieces.
        const PIECES: usize = 8;
        let sag = if washing { 0.15 } else { 0.5 };
        let line = |t: f32| tops[0].lerp(tops[1], t) - Vec3::Y * (sag * 4.0 * t * (1.0 - t) + 0.05);
        for k in 0..PIECES {
            let (a, b) = (k as f32 / PIECES as f32, (k + 1) as f32 / PIECES as f32);
            self.dressing
                .beam(line(a), line(b), 0.03, 0.03, Color::srgb(0.85, 0.82, 0.75));
        }
        let along = (tops[1] - tops[0]).normalize_or_zero();
        if washing {
            // Clothes pegged to it: shirts, sheets and towels.
            for (k, t) in [0.2, 0.42, 0.62, 0.82].into_iter().enumerate() {
                let at = line(t);
                let (wide, long) = [(0.55, 0.7), (0.8, 0.9), (0.45, 0.55), (0.6, 0.8)][k];
                let half = along * wide / 2.0;
                self.dressing.quad(
                    [
                        (at - half, Vec2::ZERO),
                        (at + half, Vec2::X),
                        (at + half - Vec3::Y * long, Vec2::ONE),
                        (at - half - Vec3::Y * long, Vec2::Y),
                    ],
                    facing,
                    CLOTHES[k % CLOTHES.len()],
                );
            }
        } else {
            // Triangular flags hanging from it.
            const COUNT: usize = 9;
            for k in 0..COUNT {
                let t = (k as f32 + 0.5) / COUNT as f32;
                let at = line(t);
                let half = along * 0.22;
                self.dressing.triangle(
                    [
                        (at - half, Vec2::ZERO),
                        (at + half, Vec2::X),
                        (at - Vec3::Y * 0.5, Vec2::new(0.5, 1.0)),
                    ],
                    facing,
                    FLAGS[(k + self.count) % FLAGS.len()],
                );
            }
        }
        self.garden(bank, base, rotation, terrain);
    }

    /// Before the hole, its garden's fence: pickets on a half oval from
    /// either side of its front round before it, rails between them, and a
    /// gap for the gate in the middle, on the ground.
    fn garden(&mut self, bank: &Bank, base: Vec3, rotation: Quat, terrain: &Terrain) {
        let (wide, deep) = (bank.half_width + GARDEN_SIDE, GARDEN_DEPTH);
        let around = std::f32::consts::PI * (wide + deep) / 2.0;
        let count = (around / PICKET_EVERY).round() as usize;
        let at = |angle: f32, up: f32| {
            let local = Vec3::new(wide * angle.sin(), 0.0, bank.front + deep * angle.cos());
            let p = base + rotation * local;
            Vec3::new(p.x, terrain.height(Vec2::new(p.x, p.z)) + up, p.z)
        };
        let angles: Vec<f32> = (0..=count)
            .map(|k| -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * k as f32 / count as f32)
            .filter(|angle| angle.abs() > GARDEN_GATE)
            .collect();
        for &angle in &angles {
            self.dressing.beam(
                at(angle, -0.2),
                at(angle, PICKET_HEIGHT),
                0.07,
                0.07,
                PICKET_WOOD,
            );
        }
        for pair in angles.windows(2) {
            // No rail across the gate.
            if pair[0] < 0.0 && pair[1] > 0.0 {
                continue;
            }
            for up in [0.3, 0.7] {
                self.dressing
                    .beam(at(pair[0], up), at(pair[1], up), 0.04, 0.05, PICKET_WOOD);
            }
        }
    }

    fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bike::model::imported::read;
    use crate::circuits::CIRCUITS;
    use crate::track::WALL_DISTANCE;
    use crate::world::japan::{VIEW_CLEAR, bounds};

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(include_bytes!("../../assets/models/fellowship/props.bin"))
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

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|p| p.as_float3())
            .unwrap()
    }

    /// The circuits with the scenery of Middle-earth.
    fn in_middle_earth() -> Vec<(&'static crate::circuits::Circuit, Track, Terrain)> {
        CIRCUITS
            .iter()
            .filter(|circuit| circuit.scenery.middle_earth.is_some())
            .map(|circuit| {
                let track = Track::new(&circuit.layout);
                let terrain = Terrain::new(&track, &circuit.scenery);
                (circuit, track, terrain)
            })
            .collect()
    }

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        let pack = pack();
        for name in MODELS
            .iter()
            .map(|&model| name(model))
            .chain([WHEEL, CHIMNEY])
        {
            let mesh = piece(&pack, name);
            let (low, high) = bounds(mesh);
            // The mill and its wheel are about the wheel's axle; the rest
            // stand on the ground, centred on their origin (the great trees
            // on the foot of their trunks).
            if name != "mill" && name != WHEEL {
                assert!(low.y.abs() < 1e-3, "{name} stands on the ground");
            }
            if !matches!(
                name,
                "mill"
                    | WHEEL
                    | "party_tree"
                    | "bag_end_oak"
                    | "root_tree"
                    | "mallorn_flet"
                    | "durin_doors"
            ) {
                assert!((low.x + high.x).abs() < 0.05 * (high.x - low.x), "{name}");
                assert!((low.z + high.z).abs() < 0.05 * (high.z - low.z), "{name}");
            }
            let triangles = mesh.indices().unwrap().len() / 3;
            assert!(triangles < 8_000, "{name}: {triangles} triangles");
            assert!(high.y - low.y < 61.0, "{name}: {} m", high.y - low.y);
        }
        // The kings of the Argonath, 60 m high.
        let (low, high) = bounds(piece(&pack, "argonath_a"));
        assert!((high.y - low.y - 60.0).abs() < 0.1);
    }

    #[test]
    fn the_mill_wheel_turns_about_its_axle_against_the_mill() {
        let pack = pack();
        let (wheel_low, wheel_high) = bounds(piece(&pack, WHEEL));
        let (mill_low, mill_high) = bounds(piece(&pack, "mill"));
        // Round its axle, at the origin.
        assert!(
            (wheel_low.x + wheel_high.x).abs() < 0.2,
            "{wheel_low} {wheel_high}"
        );
        assert!(
            (wheel_low.y + wheel_high.y).abs() < 0.2,
            "{wheel_low} {wheel_high}"
        );
        // The axle over the ground, the wheel's rim a little into it.
        assert!(mill_low.y < -2.0, "{mill_low}");
        let sunk = mill_low.y - wheel_low.y;
        assert!((0.1..0.6).contains(&sunk), "{sunk} m into the ground");
        // The mill behind the wheel (towards -Z), the wheel turning clear
        // of its wall.
        let wall = positions(piece(&pack, "mill"))
            .iter()
            .filter(|p| p[0].abs() < 2.0 && p[1].abs() < 1.5)
            .map(|p| p[2])
            .fold(f32::MIN, f32::max);
        assert!(
            wall < wheel_low.z,
            "the wall at {wall}, the wheel from {}",
            wheel_low.z
        );
        assert!(mill_high.z < wheel_high.z);
    }

    #[test]
    fn bree_gate_leaves_the_road_clear_under_its_beam() {
        let pack = pack();
        let mesh = piece(&pack, "bree_gate");
        let (_, high) = bounds(mesh);
        // Below its beam nothing stands nearer the middle than its towers,
        // just outside the walls, and the beam is high over a rider's head.
        let nearest = positions(mesh)
            .iter()
            .filter(|p| p[1] < 0.5 * high.y)
            .map(|p| p[0].abs())
            .fold(f32::MAX, f32::min);
        assert!(
            nearest > WALL_DISTANCE && nearest < WALL_DISTANCE + 0.6,
            "{nearest} m"
        );
        let beam = positions(mesh)
            .iter()
            .filter(|p| p[0].abs() < nearest - 0.05)
            .map(|p| p[1])
            .fold(f32::MAX, f32::min);
        assert!(beam > 7.5, "the beam {beam} m up");
    }

    #[test]
    fn the_doors_of_durin_stand_open_with_their_ithildin_on_them() {
        let pack = pack();
        let doors = piece(&pack, "durin_doors");
        let (low, high) = bounds(doors);
        // Grand, their leaves swung open in front of them.
        assert!((high.y - 35.0).abs() < 0.1, "{high}");
        assert!(high.z - low.z > 7.0, "{low} {high}");
        // Under the arch, nothing in the way but at its sides (the feet of
        // the pillars), and the leaves up to the arch.
        let opening = positions(doors)
            .iter()
            .filter(|p| p[1] > 0.2 && p[1] < 25.0)
            .map(|p| p[0].abs())
            .fold(f32::MAX, f32::min);
        assert!(opening > 5.6, "{opening} m");
        // The drawing on the arch and the pillars, over it on the rock
        // behind, and on the leaves, swung forward: from just above the
        // ground to over the doors, wider than the doorway, and out on the
        // leaves.
        let (drawn_low, drawn_high) = bounds(piece(&pack, ITHILDIN));
        assert!(drawn_high.y > high.y, "{drawn_high}");
        assert!(drawn_low.y < 2.0 && drawn_low.z > low.z, "{drawn_low}");
        assert!(drawn_high.x > opening + 2.0 && -drawn_low.x > opening + 2.0);
        assert!(drawn_high.z > high.z - 2.5, "{drawn_high} {high}");
        // Its glow round it, wider.
        let (glow_low, glow_high) = bounds(piece(&pack, GLOW));
        assert!(glow_high.x > drawn_high.x && glow_low.x < drawn_low.x);
        assert!(glow_high.y > drawn_high.y);
    }

    #[test]
    fn a_hobbit_hole_is_dug_into_a_little_hill() {
        let bank = Bank {
            half_width: 4.5,
            back: -1.0,
            front: 0.5,
            top: 5.6,
            sink: BANK_SINK,
        };
        // Behind it a dome, higher than its top, as high as its top either
        // side of its back.
        assert!(bank.height(Vec2::new(0.0, -1.5)) > 5.6 + 2.0);
        assert!(bank.height(Vec2::new(0.0, -1.0 - HILL_MIDDLE)) > 5.6 + 2.0);
        assert!(bank.height(Vec2::new(5.4, -1.0)) > 5.6);
        // Round: as high as far from its middle every way.
        let middle = Vec2::new(0.0, -1.0 - HILL_MIDDLE);
        let reach = Vec2::new(4.5 + HILL_SIDES, HILL_BEHIND);
        let round = |angle: f32| {
            bank.height(middle + reach * 0.6 * Vec2::new(angle.cos(), -angle.sin().abs()))
        };
        assert!((round(0.3) - round(1.2)).abs() < 1e-3);
        // Over it and before it nothing: the hill cut away round it, the
        // cut opening out before it.
        assert!(bank.height(Vec2::new(0.0, -0.5)) < 0.0);
        assert!(bank.height(Vec2::new(4.4, 0.0)) < 0.0);
        assert!(bank.height(Vec2::new(0.0, 0.6)) < 0.0);
        assert!(bank.height(Vec2::new(4.0, 1.0)) < 0.0);
        assert!(bank.height(Vec2::new(5.5, 3.0)) < 0.0);
        // Either side of it the hill comes down round its front, with no
        // bulge standing out before it.
        assert!(bank.height(Vec2::new(6.0, 1.0)) < 4.0);
        assert!(bank.height(Vec2::new(7.0, 2.5)) < 0.0);
        // Behind it the dome falls away, rounded, into the ground.
        let behind = bank.height(Vec2::new(0.0, -12.0));
        assert!(behind > 0.0 && behind < 5.6, "{behind}");
        assert!(bank.height(Vec2::new(0.0, -8.0)) < bank.height(Vec2::new(0.0, -2.0)));
        let (x0, z0, z1) = bank.extent();
        for at in [
            Vec2::new(0.0, z0),
            Vec2::new(x0, -0.5),
            Vec2::new(-x0, -3.0),
            Vec2::new(4.5, z1),
            Vec2::new(8.0, z1),
        ] {
            assert!((bank.height(at) + BANK_SINK).abs() < 1e-3, "{at}");
        }
    }

    #[test]
    fn the_footpaths_lie_on_the_ground_off_the_roads_and_out_of_the_water() {
        for (circuit, track, terrain) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            assert!(!middle_earth.paths.is_empty(), "{}", circuit.name);
            for path in middle_earth.paths {
                for at in path_samples(path) {
                    let road = track.nearest(at).distance;
                    assert!(
                        road > WALL_DISTANCE + 1.5,
                        "{}: a path at {at}, {road} m from a road",
                        circuit.name
                    );
                    assert!(
                        terrain.height(at) > SEA_LEVEL + 0.3,
                        "{}: a path at {at} in the water",
                        circuit.name
                    );
                }
            }
            let mesh = path_mesh(middle_earth.paths, terrain).unwrap();
            for p in positions(&mesh) {
                let ground = terrain.height(Vec2::new(p[0], p[2]));
                assert!(
                    (p[1] - ground - PATH_OVER).abs() < 1e-3,
                    "{}: a path {} m over the ground",
                    circuit.name,
                    p[1] - ground
                );
            }
        }
    }

    #[test]
    fn props_stand_clear_of_the_roads_on_land_or_on_the_water() {
        let circuits = in_middle_earth();
        assert!(!circuits.is_empty());
        let pack = pack();
        for (circuit, track, terrain) in &circuits {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            for prop in middle_earth.props {
                let mesh = piece(&pack, name(prop.model));
                let (low, high) = bounds(mesh);
                let reach = prop.scale * (high.x - low.x).max(high.z - low.z) / 2.0;
                let (position, _) = spot(track, prop);
                let road = track.nearest(position).distance;
                let what = format!("{}: {:?} at {position}", circuit.name, prop.model);
                match prop.model {
                    // Over the road.
                    MiddleEarthModel::BreeGate => assert!(road < 1.0, "{what}"),
                    // Over the road, at the mouth of a tunnel, where the
                    // road narrows to go through them.
                    MiddleEarthModel::DurinDoors => {
                        assert!(road < 1.0, "{what}");
                        let along = track.along(&track.nearest(position));
                        let lap = track.lap_length();
                        assert!(circuit.scenery.tunnels.iter().any(|tunnel| {
                            (along - tunnel.from).rem_euclid(lap) < 1.0
                                || (tunnel.from - along).rem_euclid(lap) < 1.0
                        }));
                        let index = track.index_at(along);
                        let opening = positions(mesh)
                            .iter()
                            .filter(|p| p[1] > 0.2 && p[1] < 6.0)
                            .map(|p| p[0].abs())
                            .fold(f32::MAX, f32::min);
                        assert!(
                            track.wall_at(index) < opening,
                            "{what}: walls {} m out, the doors {opening} m",
                            track.wall_at(index)
                        );
                    }
                    // Over a way of the road, which runs through its archway:
                    // nothing of it as high as a rider stands within that
                    // way's walls.
                    MiddleEarthModel::PrancingPony => {
                        let nearest = track.nearest(position);
                        assert!(nearest.distance < 1.0 && nearest.road != 0, "{what}");
                        let way = track.road(nearest.road);
                        let (_, rotation) = spot(track, prop);
                        for p in positions(mesh).iter().filter(|p| p[1] * prop.scale < 3.0) {
                            let at = rotation * (Vec3::from_array(*p) * prop.scale);
                            let at = position + Vec2::new(at.x, at.z);
                            let on = way.nearest_among(at, 0, way.sample_count() - 1);
                            assert!(
                                on.distance > way.wall_at(on.index),
                                "{what}: {} m from its way, its walls {} m",
                                on.distance,
                                way.wall_at(on.index)
                            );
                        }
                    }
                    _ => assert!(road - reach * 0.7 > WALL_DISTANCE, "{what}: {road} m"),
                }
                let wet = terrain.height(position) < SEA_LEVEL;
                match footing(prop.model) {
                    Footing::Afloat => assert!(wet, "{what}: a boat ashore"),
                    Footing::Wading => {
                        // On its rock in the river, a little over the water
                        // or under it.
                        let ground = terrain.height(position);
                        assert!((ground - SEA_LEVEL).abs() < 6.0, "{what}: {ground} m");
                    }
                    Footing::Shore => {
                        let ground = terrain.height(position);
                        assert!((ground - SEA_LEVEL).abs() < 1.5, "{what}: {ground} m");
                    }
                    Footing::Ground | Footing::Bank => assert!(!wet, "{what}: in the water"),
                    Footing::OverRoad => {}
                }
            }
        }
    }

    #[test]
    fn the_banks_of_the_hobbit_holes_sink_into_the_ground_all_round() {
        for (circuit, track, terrain) in &in_middle_earth() {
            let pack = pack();
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            let mut banks = 0;
            for prop in middle_earth.props {
                if footing(prop.model) != Footing::Bank {
                    continue;
                }
                banks += 1;
                let (low, high) = bounds(piece(&pack, name(prop.model)));
                let (low, high) = (low * prop.scale, high * prop.scale);
                let reach = (high.x - low.x).max(high.z - low.z) / 2.0;
                let foot = foot(track, terrain, prop, reach);
                let (position, rotation) = spot(track, prop);
                let base = Vec3::new(position.x, foot, position.y);
                let bank = Bank::of(low, high).sunk(base, rotation, terrain);
                assert!(
                    bank.sink < 12.0,
                    "{}: {:?}'s bank sinks {} m",
                    circuit.name,
                    prop.model,
                    bank.sink
                );
                // Round its edge, its foot is in the ground: no bank floats.
                let (x0, z0, z1) = bank.extent();
                for k in 0..=40 {
                    let x = -x0 + 2.0 * x0 * k as f32 / 40.0;
                    for z in [z0, z1] {
                        let local = Vec3::new(x, bank.height(Vec2::new(x, z)), z);
                        let world = base + rotation * local;
                        let ground = terrain.height(Vec2::new(world.x, world.z));
                        assert!(
                            world.y < ground + 0.1,
                            "{}: {:?}'s bank {} m over the ground",
                            circuit.name,
                            prop.model,
                            world.y - ground
                        );
                    }
                }
            }
            assert!(banks >= 5, "{}: {banks} hobbit holes", circuit.name);
        }
    }

    #[test]
    fn no_tree_stands_in_the_way_of_a_landmark() {
        for (circuit, track, _) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            for prop in middle_earth.props.iter().filter(|p| landmark(p.model)) {
                let (position, _) = spot(track, prop);
                let clear = in_view(track, position);
                // Ground kept clear from past the road's walls up to it,
                // from some way along the road coming at it.
                let near = clear
                    .iter()
                    .map(|&(at, _)| at.distance(position))
                    .fold(f32::MAX, f32::min);
                assert!(near < 2.0 * VIEW_CLEAR + 10.0, "{:?}: {near} m", prop.model);
                let from = clear
                    .iter()
                    .map(|&(at, _)| at.distance(position))
                    .fold(0.0, f32::max);
                assert!(
                    from > 60.0,
                    "{}: {:?} seen from {from} m",
                    circuit.name,
                    prop.model
                );
            }
        }
    }
}
