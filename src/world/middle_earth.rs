//! The scenery of Middle-earth ([`MiddleEarth`]), as at Fellowship Road:
//! props made with Meshy standing about (prepared by
//! `tools/import_fellowship.py`): hobbit holes set into grassy banks with
//! their chimneys rising out of them, Bag End under its oak, the Green
//! Dragon and the mill of Bywater, its wheel turning, Bree's gate over the
//! road and its inn, the ruins on Weathertop, the stone trolls, the halls of
//! Rivendell, the Doors of Durin open over the road into Moria, the drawing
//! of ithildin glowing on them, outside and on their leaves within, a
//! mallorn, boats on the river and the kings of the Argonath standing in
//! it; low bushes about the hobbit holes' gardens and along the footpaths;
//! and, as in feudal Japan (see [`super::japan`], whose props are placed
//! alike), fields of flowers and leaves drifting down.
//!
//! Every prop's colours are in one texture, so they all share one material,
//! and every kind of prop is one mesh (but the ithildin on the Doors of
//! Durin and its glow, each in a material of its own).

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use super::Assets3d;
use super::detail::fading_out;
use super::flowers::spawn_flowers;
use super::prop_pack::{PropPack, Turning, in_view, spot};
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{Glow, MiddleEarthModel, Prop, RockArch, Scenery, Shaft, Spot};
use super::terrain::{GRASS_TILE, Ground, SEA_LEVEL, Terrain, ground_look, shaded};
use crate::geometry::{fbm, random01, smoothstep};
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
/// How far past what it lights the light of a [`Glow`] reaches, as many
/// times as far as it is from it: so far that it is as bright there as it
/// says (its light fades out only towards the end of its reach), and dark
/// well past it.
const GLOW_REACH: f32 = 2.5;
/// How much of a [`Glow`]'s cone it lights fully, the rest fading out to
/// its edge.
const GLOW_CORE: f32 = 0.6;
/// The shaft of a [`Glow`] coming down from the sky, seen in the air (see
/// [`Shaft`]): how many planes crossed round its axis make it, how far up
/// from what it lights it shows (m), and over how much of its foot it fades
/// in (m), dark where it reaches what it lights, so as not to veil it.
const SHAFT_PLANES: usize = 6;
const SHAFT_LENGTH: f32 = 110.0;
const SHAFT_CLEAR: f32 = 30.0;
/// How far from the cameras the shafts show (m, to their middle, some 55 m
/// up over what they light), fading out over so many more: seen coming up
/// to what they light, not from afar.
const SHAFT_REACH: f32 = 150.0;
const SHAFT_FADE: f32 = 60.0;

/// An arch of rock (see [`RockArch`]): how far its foot on the right sinks
/// under the road (m), and how far past its opening it reaches into the rock
/// on the left (m across); how thick it is across its curve at its foot,
/// over the road and where it goes into the rock (m); in how many pieces it
/// goes along its curve and round it; how rough it is, as rock broken
/// rather than cut, at two sizes (m either way); and how big each repetition
/// of the rock's texture is over it (m).
const ARCH_SINK: f32 = 6.0;
const ARCH_INTO: f32 = 26.0;
const ARCH_THICK: [f32; 3] = [5.5, 4.0, 8.0];
const ARCH_ALONG: usize = 48;
const ARCH_ROUND: usize = 18;
const ARCH_ROUGH: (f32, f32) = (1.7, 0.45);
const ARCH_TILE: f32 = 14.0;

/// The hill a hobbit hole is dug into, a dome over its rooms behind its
/// front: so many times as high as the hole and as wide as its front,
/// round and curved all over (a paraboloid), its middle behind the hole so
/// that its surface comes down to the top of the hole's back and no higher,
/// over the hole; the dome cut away round the hole, flush with its sides,
/// the cut opening out so much for each metre before its front (m) and
/// softened over so many metres (m); how far its mesh reaches from its
/// middle (times its reach), how far its edge sinks into the ground at
/// least (m), how far under the ground its mesh lies where the ground is
/// higher (m), and over how many metres the two round into one another
/// (m); and how finely its mesh is cut (m).
const HILL_HEIGHT: f32 = 2.0;
const HILL_WIDTH: f32 = 3.5;
const CUT_FLARE: f32 = 1.5;
const CUT_SOFT: f32 = 0.7;
const HILL_MESH: f32 = 1.35;
const BANK_SINK: f32 = 2.5;
const BANK_UNDER: f32 = 0.3;
const BANK_BLEND: f32 = 3.0;
const BANK_STEP: f32 = 0.6;
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
/// Low bushes about the village (see [`bush`]): their leaves, tints over the
/// grass's photo, as a hedge's (a deeper or a fresher green, shaded as the
/// grass round them, see `Scenery::grass_shade`), and the flowers some of
/// them are in; how wide they are,
/// from the least to the most (m), how high their lumps for how wide, and
/// how far into the ground they stand (m). About each hole's garden, where
/// round its fence (rad either side of its gate) and how far out of it (m);
/// along the footpaths, one every so many metres, to either side by turns,
/// so far from the middle of the path (m), but none near its ends.
const BUSH_LEAVES: [Color; 3] = [
    Color::srgb(0.55, 0.85, 0.4),
    Color::srgb(0.45, 0.72, 0.34),
    Color::srgb(0.62, 0.92, 0.5),
];
const BUSH_FLOWERS: [Color; 3] = [
    Color::srgb(0.95, 0.55, 0.7),
    Color::srgb(0.96, 0.95, 0.9),
    Color::srgb(0.98, 0.82, 0.3),
];
const BUSH_WIDTH: (f32, f32) = (1.1, 2.0);
const BUSH_TALL: f32 = 1.1;
const BUSH_SINK: f32 = 0.15;
const BUSH_ROUND_GARDEN: [f32; 4] = [1.15, 1.5, -1.15, -1.5];
const BUSH_OUT: f32 = 1.1;
const BUSH_EVERY: f32 = 8.0;
const BUSH_BESIDE: f32 = PATH_WIDTH / 2.0 + 1.3;
const BUSH_PATH_ENDS: f32 = 8.0;

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
            crate::content::file("models/fellowship/props.bin"),
            crate::content::file("models/fellowship/color.jpg"),
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
    if !middle_earth.arches.is_empty() {
        // The rock of the cliffs they come out of.
        let (texture, roughness, reflectance) =
            ground_look(Ground::Rock, terrain.soil(), assets.textures);
        let material = assets.materials.add(StandardMaterial {
            base_color_texture: texture.cloned(),
            perceptual_roughness: roughness,
            reflectance,
            ..default()
        });
        for arch in middle_earth.arches {
            world.spawn((
                Mesh3d(assets.meshes.add(rock_arch(track, arch))),
                MeshMaterial3d(material.clone()),
            ));
            taken.extend(arch_feet(track, arch));
        }
    }
    // Low bushes along the footpaths, out of the props' way, as round the
    // holes' gardens (see `BankMesh::garden`).
    for (at, seed) in path_bush_spots(middle_earth.paths, &taken) {
        let ground = terrain.height(at);
        if ground > terrain.water_level(at) + 0.3 {
            let width = BUSH_WIDTH.0.lerp(BUSH_WIDTH.1, random01(seed * 7 + 1));
            let foot = Vec3::new(at.x, ground, at.y);
            bush(&mut banks.bushes, &mut banks.dressing, foot, width, seed);
        }
    }
    if !banks.bushes.is_empty() {
        // Leaves, as a hedge's: the grass's photo, as dark as the grass.
        let [red, green, blue, _] = shaded(Color::WHITE, scenery.grass_shade);
        let material = assets.materials.add(StandardMaterial {
            base_color: Color::linear_rgb(red, green, blue),
            base_color_texture: Some(assets.textures.grass.clone()),
            perceptual_roughness: 0.95,
            reflectance: 0.15,
            ..default()
        });
        let bushes = std::mem::take(&mut banks.bushes).build();
        world.spawn((Mesh3d(assets.meshes.add(bushes)), MeshMaterial3d(material)));
    }
    if !banks.dressing.is_empty() {
        // Cloth, flags and flowers, seen from either side.
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
    for glow in middle_earth.glows {
        world.spawn((
            glow_light(glow),
            Transform::from_translation(glow.from).looking_at(glow.to, Vec3::Y),
        ));
    }
    let shafts = middle_earth
        .glows
        .iter()
        .filter_map(|glow| glow.shaft.map(|shaft| (glow, shaft)));
    if shafts.clone().next().is_some() {
        // Light in the air: added over what is behind it, seen from either
        // side, not fogged.
        let material = assets.materials.add(StandardMaterial {
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            double_sided: true,
            fog_enabled: false,
            ..default()
        });
        for (glow, shaft) in shafts {
            world.spawn((
                Mesh3d(assets.meshes.add(shaft_mesh(glow, shaft))),
                MeshMaterial3d(material.clone()),
                fading_out(SHAFT_REACH, SHAFT_FADE, true),
            ));
        }
    }
    // The circuit's flowers, clear of its props.
    if let Some(flowers) = assets.flowers.take() {
        let meadows = spawn_flowers(world, terrain, scenery.flowers, flowers, &taken, assets);
        taken.extend(meadows);
    }
    // No tree in the way of the landmarks, from where the road comes at
    // them.
    for prop in middle_earth.props.iter().filter(|p| landmark(p.model)) {
        let (position, _) = spot(track, prop);
        taken.extend(in_view(track, position));
    }
    taken
}

/// Where an arch of rock stands: the middle of the road under it, across
/// the road to the left (turned as the arch is) and along it.
fn arch_frame(track: &Track, arch: &RockArch) -> (Vec3, Vec3, Vec3) {
    let index = track.index_at(arch.along);
    let foot = track.surface_point(index, 0.0);
    let left = track.offset_point(index, 1.0) - track.offset_point(index, 0.0);
    let left = Vec2::from_angle(arch.turn).rotate(left.normalize());
    (
        foot,
        Vec3::new(left.x, 0.0, left.y),
        Vec3::new(-left.y, 0.0, left.x),
    )
}

/// The curve of an arch's rock (see [`rock_arch`]), at `t` from its foot on
/// the right (0) to where it goes into the rock on the left (1): where it
/// is, across (m, to the left) and up (m) from the middle of the road, and
/// how thick it is across it there (m). Up steeply from its sunk foot, over
/// the road, and on into the rock, sinking a little.
fn arch_curve(arch: &RockArch, t: f32) -> (Vec2, f32) {
    let half = arch.width / 2.0;
    let crown = arch.height + ARCH_THICK[1];
    // Its foot's inside at the opening's edge.
    let foot = half + ARCH_THICK[0];
    let points = [
        Vec2::new(-foot, -ARCH_SINK),
        Vec2::new(-(foot + 1.0), crown * 1.05),
        Vec2::new(half * 0.2, crown * 1.18),
        Vec2::new(half + ARCH_INTO, crown * 0.7),
    ];
    let u = 1.0 - t;
    let at = points[0] * u * u * u
        + points[1] * 3.0 * u * u * t
        + points[2] * 3.0 * u * t * t
        + points[3] * t * t * t;
    let thick = if t < 0.5 {
        ARCH_THICK[0].lerp(ARCH_THICK[1], smoothstep(0.0, 0.5, t))
    } else {
        ARCH_THICK[1].lerp(ARCH_THICK[2], smoothstep(0.5, 1.0, t))
    };
    (at, thick)
}

/// The ground an arch takes (see [`arch_frame`]): its foot, and where it
/// goes into the rock, kept clear of trees.
fn arch_feet(track: &Track, arch: &RockArch) -> [Footprint; 2] {
    let (foot, across, _) = arch_frame(track, arch);
    [0.0, 1.0].map(|t| {
        let (at, thick) = arch_curve(arch, t);
        let at = foot + across * at.x;
        (Vec2::new(at.x, at.z), thick + arch.depth / 2.0)
    })
}

/// A rough fractal noise in space, about ½, over features `size` metres
/// across.
fn rock_noise(p: Vec3, size: f32) -> f32 {
    let a = fbm(Vec2::new(p.x + 0.7 * p.y, p.z) / size, 3);
    let b = fbm(
        Vec2::new(p.y - 0.6 * p.z, p.x) / size + Vec2::splat(31.0),
        3,
    );
    (a + b) / 2.0
}

/// An arch of rock over the road (see [`RockArch`]), as the sea's arches on
/// a beach of cliffs: rock out of the rock on the left, over the road,
/// down onto its foot on the right, sunk under the road: round its curve
/// ([`arch_curve`]) a body as thick across it as the curve says and as
/// `depth` along the road, its section swelling and narrowing, its surface
/// broken at two sizes, smooth between (no cut faces), so it is rock as it
/// broke and the sea wore it, not cut.
fn rock_arch(track: &Track, arch: &RockArch) -> Mesh {
    let (foot, across, along) = arch_frame(track, arch);
    let mut positions = Vec::with_capacity((ARCH_ALONG + 1) * ARCH_ROUND);
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut colors = Vec::with_capacity(positions.capacity());
    for k in 0..=ARCH_ALONG {
        let t = k as f32 / ARCH_ALONG as f32;
        let (at, thick) = arch_curve(arch, t);
        let ahead =
            arch_curve(arch, (t + 0.01).min(1.0)).0 - arch_curve(arch, (t - 0.01).max(0.0)).0;
        // Out from the curve, in the plane across the road.
        let out = Vec2::new(-ahead.y, ahead.x).normalize_or_zero();
        let middle = foot + across * at.x + Vec3::Y * at.y;
        let out = across * out.x + Vec3::Y * out.y;
        for n in 0..ARCH_ROUND {
            let angle = std::f32::consts::TAU * n as f32 / ARCH_ROUND as f32;
            let round = out * (thick * angle.cos()) + along * (arch.depth / 2.0 * angle.sin());
            let p = middle + round;
            // Broken: bulging and hollowed out at two sizes, never more
            // than the rock is thick.
            let rough = ARCH_ROUGH.0 * 2.0 * (rock_noise(p, 7.0) - 0.5)
                + ARCH_ROUGH.1 * 2.0 * (rock_noise(p, 1.8) - 0.5);
            let p = p + round.normalize_or_zero() * rough.max(-0.5 * thick);
            positions.push(p.to_array());
            let flat = p - foot;
            uvs.push([
                (flat.dot(along) + flat.dot(across)) / ARCH_TILE,
                -flat.y / ARCH_TILE,
            ]);
            let shade = 0.78 + 0.22 * smoothstep(-ARCH_SINK, arch.height, flat.y);
            colors.push([shade, shade, shade, 1.0]);
        }
    }
    let mut indices = Vec::with_capacity(ARCH_ALONG * ARCH_ROUND * 6);
    for k in 0..ARCH_ALONG as u32 {
        for n in 0..ARCH_ROUND as u32 {
            let next = (n + 1) % ARCH_ROUND as u32;
            let (a, b) = (k * ARCH_ROUND as u32 + n, k * ARCH_ROUND as u32 + next);
            let (c, d) = (b + ARCH_ROUND as u32, a + ARCH_ROUND as u32);
            indices.extend([a, b, c, a, c, d]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices));
    mesh.compute_smooth_normals();
    // Wound the way its outside faces: on top of its crown, up.
    let top = (ARCH_ALONG / 2) * ARCH_ROUND;
    let upward = matches!(
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
        Some(VertexAttributeValues::Float32x3(normals)) if normals[top][1] > 0.0
    );
    if !upward {
        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            for triangle in indices.as_chunks_mut::<3>().0.iter_mut() {
                triangle.swap(1, 2);
            }
        }
        mesh.compute_smooth_normals();
    }
    mesh
}

/// The light of a [`Glow`]: a spot light shining at its target, as bright
/// there as the glow says (a spot light's lumens spread as a point light's,
/// over the whole sphere), reaching well past it, casting no shadow.
fn glow_light(glow: &Glow) -> SpotLight {
    let distance = glow.from.distance(glow.to);
    SpotLight {
        color: glow.color,
        intensity: glow.illuminance * 4.0 * std::f32::consts::PI * distance * distance,
        range: distance * GLOW_REACH,
        radius: 0.0,
        shadow_maps_enabled: false,
        outer_angle: glow.angle,
        inner_angle: glow.angle * GLOW_CORE,
        ..default()
    }
}

/// The shaft of light a [`Glow`] comes down from the sky in, as it shows in
/// the air: planes crossed round its axis, from what it lights up towards
/// where it comes from, as wide at their foot as its light, its soft edge
/// and all, and spreading wider up into the sky, as light diffused through
/// the haze and the clouds, as much as its `shaft` says; brightest down
/// their middle a little way over what it lights (as bright as `shaft`
/// says there), fading in from dark over [`SHAFT_CLEAR`] above it, and
/// fading out softly to their edges and up into the sky.
fn shaft_mesh(glow: &Glow, shaft: Shaft) -> Mesh {
    const ROWS: usize = 16;
    const ACROSS: [f32; 9] = [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];
    let distance = glow.from.distance(glow.to);
    let up = (glow.from - glow.to) / distance;
    let length = SHAFT_LENGTH.min(distance);
    let foot = distance * glow.angle.tan();
    let light = glow.color.to_linear();
    // How bright it is up its middle, from 0 at its foot to 1 at its top:
    // fading in over its foot, out up into the sky, at most 1.
    let along = |t: f32| smoothstep(0.0, SHAFT_CLEAR / length, t) * (1.0 - t).powi(2);
    let brightest = (0..=ROWS)
        .map(|row| along(row as f32 / ROWS as f32))
        .fold(0.0, f32::max);
    let mut builder = MeshBuilder::default();
    for plane in 0..SHAFT_PLANES {
        let turn = plane as f32 * std::f32::consts::PI / SHAFT_PLANES as f32;
        let across = Quat::from_axis_angle(up, turn) * up.any_orthonormal_vector();
        let corner = |row: usize, column: usize| {
            let t = row as f32 / ROWS as f32;
            let radius = foot * (1.0 + (shaft.spread - 1.0) * t);
            let fade = along(t) / brightest * (1.0 - ACROSS[column].powi(2)).powi(2);
            (
                glow.to + up * length * t + across * radius * ACROSS[column],
                Vec2::new(column as f32, row as f32),
                // Added as light is, by its colour alone: opaque.
                Color::from((light * (shaft.light * fade)).with_alpha(1.0)),
            )
        };
        let facing = across.cross(up);
        for row in 0..ROWS {
            for column in 0..ACROSS.len() - 1 {
                let (a, b) = (corner(row, column), corner(row, column + 1));
                let (c, d) = (corner(row + 1, column + 1), corner(row + 1, column));
                builder.shaded_triangle([a, b, c], facing);
                builder.shaded_triangle([a, c, d], facing);
            }
        }
    }
    builder.build()
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

/// Where low bushes stand along the footpaths, with a number each to pick
/// its shape: one every [`BUSH_EVERY`] metres or so, to either side by
/// turns, [`BUSH_BESIDE`] from the middle of the path, none within
/// [`BUSH_PATH_ENDS`] of its ends (a hole's gate, the road), nor on another
/// path, nor in `taken` ground (the props').
fn path_bush_spots(paths: &[&[Vec2]], taken: &[Footprint]) -> Vec<(Vec2, u32)> {
    let lines: Vec<Vec<Vec2>> = paths.iter().map(|path| path_samples(path)).collect();
    let mut spots = Vec::new();
    for (number, samples) in lines.iter().enumerate() {
        let length: f32 = samples.windows(2).map(|w| w[0].distance(w[1])).sum();
        let (mut along, mut next, mut side) = (0.0, BUSH_PATH_ENDS, 1.0);
        for k in 1..samples.len() - 1 {
            along += samples[k - 1].distance(samples[k]);
            if along < next || along > length - BUSH_PATH_ENDS {
                continue;
            }
            let seed = (number * 1000 + k) as u32;
            next = along + BUSH_EVERY * (0.75 + 0.5 * random01(seed * 5 + 2));
            side = -side;
            let across = (samples[k + 1] - samples[k - 1]).normalize_or_zero().perp();
            let at = samples[k] + across * BUSH_BESIDE * side;
            let on_a_path = lines
                .iter()
                .flatten()
                .any(|p| p.distance(at) < BUSH_BESIDE - 0.1);
            let on_taken = taken
                .iter()
                .any(|(center, radius)| center.distance(at) < radius + 1.0);
            if !on_a_path && !on_taken {
                spots.push((at, seed));
            }
        }
    }
    spots
}

/// A low bush standing on the ground at `foot`, about `width` across: three
/// round lumps of leaves together (into `leaves`, the grass's photo laid
/// over them), a little uneven, flat-shaded, a deeper or a fresher green,
/// darker low down; `seed` picks its shape and its green, and whether it is
/// in flower, specks of colour over its top (into `flowers`).
fn bush(leaves: &mut MeshBuilder, flowers: &mut MeshBuilder, foot: Vec3, width: f32, seed: u32) {
    const AROUND: usize = 7;
    const UP: [f32; 4] = [-0.2, 0.45, 0.95, 1.35];
    let random = |k: u32| random01(seed * 31 + k);
    let green = BUSH_LEAVES[seed as usize % BUSH_LEAVES.len()];
    let flower = (random(0) < 0.35).then(|| BUSH_FLOWERS[(random(1) * 3.0) as usize % 3]);
    // The photo laid over the leaves as over the banks' slopes.
    let uv = |p: Vec3| Vec2::new(p.x + p.z, p.y * 2.0) / (GRASS_TILE * 0.25);
    for lump in 0..3u32 {
        let (offset, radius) = if lump == 0 {
            (Vec2::ZERO, width * 0.5)
        } else {
            let turn = std::f32::consts::TAU * random(lump * 2);
            (
                Vec2::from_angle(turn) * width * 0.25,
                width * (0.3 + 0.08 * random(lump * 2 + 1)),
            )
        };
        let middle = foot + Vec3::new(offset.x, -BUSH_SINK, offset.y);
        let point = |ring: usize, around: usize| {
            let (lat, lon) = (
                UP[ring],
                std::f32::consts::TAU * around as f32 / AROUND as f32,
            );
            let uneven = 0.85 + 0.3 * random(10 + lump * 40 + (ring * AROUND + around) as u32);
            middle
                + Vec3::new(
                    radius * uneven * lat.cos() * lon.cos(),
                    radius * BUSH_TALL * lat.sin().max(0.0),
                    radius * uneven * lat.cos() * lon.sin(),
                )
        };
        let top = middle + Vec3::Y * radius * BUSH_TALL;
        let shade = |ring: usize, k: u32| {
            let dark = 0.72 + 0.12 * ring as f32 + 0.12 * random(200 + lump * 40 + k);
            Color::LinearRgba(green.to_linear() * dark)
        };
        for around in 0..AROUND {
            let next = (around + 1) % AROUND;
            for ring in 0..UP.len() - 1 {
                let [a, b, c, d] = [
                    point(ring, around),
                    point(ring, next),
                    point(ring + 1, next),
                    point(ring + 1, around),
                ];
                let out = (a + c) / 2.0 - middle;
                let color = shade(ring, (ring * AROUND + around) as u32);
                leaves.triangle([(a, uv(a)), (b, uv(b)), (c, uv(c))], out, color);
                leaves.triangle([(a, uv(a)), (c, uv(c)), (d, uv(d))], out, color);
            }
            let (a, b) = (point(UP.len() - 1, around), point(UP.len() - 1, next));
            let out = (a + b + top) / 3.0 - middle;
            leaves.triangle(
                [(a, uv(a)), (b, uv(b)), (top, uv(top))],
                out,
                shade(UP.len() - 1, 99 + around as u32),
            );
            // A flower or two over its top.
            if let Some(flower) = flower {
                let at =
                    point(2, around).lerp(top, 0.3 + 0.4 * random(300 + lump * 9 + around as u32));
                let out = (at - middle).normalize_or_zero();
                let across = out.cross(Vec3::Y).normalize_or_zero() * 0.09;
                let up = across.cross(out).normalize_or_zero() * 0.09;
                let at = at + out * 0.04;
                flowers.triangle(
                    [
                        (at - across - up, Vec2::ZERO),
                        (at + across - up, Vec2::ZERO),
                        (at + up * 1.4, Vec2::ZERO),
                    ],
                    out,
                    flower,
                );
            }
        }
    }
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
        let (middle, reach) = (self.middle(), self.reach());
        (
            HILL_MESH * reach,
            middle - HILL_MESH * reach,
            middle + HILL_MESH * reach,
        )
    }

    /// How far the dome reaches from its middle (m), every way.
    fn reach(&self) -> f32 {
        HILL_WIDTH * self.half_width
    }

    /// Where the middle of the dome is (z, m): behind the hole, so far that
    /// over the hole's back the dome is as high as the hole.
    fn middle(&self) -> f32 {
        self.back - self.reach() * (1.0 - 1.0 / HILL_HEIGHT).sqrt()
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
        // The dome: round about its middle, behind the hole, curved all
        // over, on down into the ground past its foot, and down to its
        // sunk edge by the edge of its mesh.
        let off = (at - Vec2::new(0.0, self.middle())) / self.reach();
        let r = off.length();
        let hill =
            HILL_HEIGHT * self.top * (1.0 - r * r) - self.sink * smoothstep(1.0, HILL_MESH, r);
        let hill = hill.max(-self.sink);
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
    /// The leaves of the low bushes about the gardens and along the paths
    /// (their flowers are with the dressing).
    bushes: MeshBuilder,
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
        // Low bushes outside it, on either side, away from its gate and the
        // path to it: on the ground, or on the bank where it comes down
        // round the front.
        for (k, angle) in BUSH_ROUND_GARDEN.into_iter().enumerate() {
            let (x, z) = (
                (wide + BUSH_OUT) * angle.sin(),
                bank.front + (deep + BUSH_OUT) * angle.cos(),
            );
            let p = base + rotation * Vec3::new(x, bank.height(Vec2::new(x, z)), z);
            let ground = terrain.height(Vec2::new(p.x, p.z));
            let foot = Vec3::new(p.x, p.y.max(ground), p.z);
            let seed = (self.count * BUSH_ROUND_GARDEN.len() + k) as u32;
            let width = BUSH_WIDTH.0.lerp(BUSH_WIDTH.1, random01(seed * 7 + 1));
            bush(&mut self.bushes, &mut self.dressing, foot, width, seed);
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
    use crate::world::prop_pack::{VIEW_CLEAR, bounds};

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(crate::content::file("models/fellowship/props.bin"))
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
    fn the_buildings_stand_on_their_ground_not_buried_in_it() {
        // The ground round a building, as far out as most of its walls,
        // rises over its foot by no more than a fifth of its height: the
        // ruins on Weathertop's summit, not in the bank under it. (The seat
        // of seeing is set into the slope of Amon Hen behind it.)
        let pack = pack();
        let buildings = [
            MiddleEarthModel::WeathertopRuin,
            MiddleEarthModel::StoneTrolls,
            MiddleEarthModel::PrancingPony,
            MiddleEarthModel::GreenDragon,
            MiddleEarthModel::LastHomelyHouse,
            MiddleEarthModel::ElvenTower,
            MiddleEarthModel::Gazebo,
        ];
        for (circuit, track, terrain) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            for prop in middle_earth
                .props
                .iter()
                .filter(|p| buildings.contains(&p.model))
            {
                let (low, high) = bounds(piece(&pack, name(prop.model)));
                let reach = (high.x - low.x).max(high.z - low.z) / 2.0;
                let (position, _) = spot(track, prop);
                let foot = foot(track, terrain, prop, reach);
                let over = (0..16)
                    .map(|k| {
                        let turn = k as f32 * std::f32::consts::TAU / 16.0;
                        let at = position + Vec2::from_angle(turn) * 0.8 * reach;
                        terrain.height(at) - foot
                    })
                    .fold(f32::MIN, f32::max);
                assert!(
                    over < 0.2 * (high.y - low.y),
                    "{}: {:?} buried {over:.1} m",
                    circuit.name,
                    prop.model
                );
            }
        }
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
        // Behind it a dome twice as high as it, as wide as its front three
        // times and more, coming down to its top over its back, and no
        // higher: no brow of grass over it.
        let (middle, reach) = (bank.middle(), bank.reach());
        assert!(2.0 * reach > 3.0 * 2.0 * 4.5);
        assert!((bank.height(Vec2::new(0.0, middle)) - 2.0 * 5.6).abs() < 1e-3);
        assert!((bank.height(Vec2::new(0.0, -1.0)) - 5.6).abs() < 1e-3);
        assert!(bank.height(Vec2::new(4.5, -1.0)) < 5.6);
        // Round, and curved all over: as high as far from its middle every
        // way, and higher the nearer its middle.
        let round = |angle: f32, share: f32| {
            let at = Vec2::new(0.0, middle) + reach * share * Vec2::new(angle.cos(), angle.sin());
            bank.height(at)
        };
        for share in [0.2, 0.5, 0.8] {
            assert!((round(0.3, share) - round(-2.0, share)).abs() < 1e-3);
            assert!(round(-1.0, share) < round(-1.0, share - 0.1));
        }
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
                    bank.sink < 16.0,
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
    fn glows_light_rivendell_s_halls_on_their_fronts_and_sunbeams_fall_on_the_landmarks() {
        for (circuit, track, terrain) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            assert!(!middle_earth.glows.is_empty(), "{}", circuit.name);
            for glow in middle_earth.glows {
                // As bright on what it lights as it says, and reaching
                // well past it.
                let light = glow_light(glow);
                let distance = glow.from.distance(glow.to);
                let lux = light.intensity / (4.0 * std::f32::consts::PI * distance * distance);
                assert!((lux / glow.illuminance - 1.0).abs() < 1e-3, "{lux} lx");
                assert!(light.range > 2.0 * distance);
                assert!(!light.shadow_maps_enabled);
            }
            // The halls at the head of Rivendell's valley, which the low sun
            // lights from behind as the riders come down it: each in the
            // full light of a glow, which falls on its front, and in a
            // sunbeam's, down from the sky onto it; and in a sunbeam's, the
            // seat of seeing on Amon Hen, the ruins on Weathertop and the
            // stone trolls.
            let halls = [
                MiddleEarthModel::Gazebo,
                MiddleEarthModel::LastHomelyHouse,
                MiddleEarthModel::ElvenTower,
                MiddleEarthModel::AmonHenSeat,
                MiddleEarthModel::WeathertopRuin,
                MiddleEarthModel::StoneTrolls,
            ];
            let fronts = &halls[..3];
            for prop in middle_earth
                .props
                .iter()
                .filter(|p| halls.contains(&p.model))
            {
                let (position, rotation) = spot(track, prop);
                let at = Vec3::new(position.x, terrain.height(position) + 8.0, position.y);
                let front = rotation * Vec3::Z;
                let on_front = fronts.contains(&prop.model);
                let lit = |glow: &&Glow| {
                    let axis = glow.to - glow.from;
                    axis.angle_between(at - glow.from) < glow.angle * GLOW_CORE
                        && (!on_front || (at - glow.from).dot(front) < 0.0)
                };
                assert!(
                    middle_earth.glows.iter().any(|glow| lit(&glow)),
                    "{}: {:?} in the shade",
                    circuit.name,
                    prop.model
                );
                assert!(
                    middle_earth
                        .glows
                        .iter()
                        .filter(|glow| glow.shaft.is_some())
                        .any(|glow| {
                            let down = glow.from - glow.to;
                            lit(&glow) && down.y > 0.8 * down.length()
                        }),
                    "{}: no sunbeam on {:?}",
                    circuit.name,
                    prop.model
                );
            }
        }
    }

    #[test]
    fn a_bush_is_low_and_round_on_the_ground() {
        let foot = Vec3::new(10.0, 3.0, -4.0);
        for seed in 0..40 {
            let width = BUSH_WIDTH.0.lerp(BUSH_WIDTH.1, random01(seed * 7 + 1));
            let (mut mesh, mut flowers) = (MeshBuilder::default(), MeshBuilder::default());
            bush(&mut mesh, &mut flowers, foot, width, seed);
            let mesh = mesh.build();
            let points = positions(&mesh);
            let top = points.iter().map(|p| p[1]).fold(f32::MIN, f32::max) - foot.y;
            // Low: about half as high as it is wide, under a man's waist.
            assert!(
                top > 0.35 * width && top < 0.75 * width && top < 1.3,
                "{top} {width}"
            );
            for p in points {
                // Round its foot, and into the ground no deeper than it sinks.
                let out = Vec2::new(p[0] - foot.x, p[2] - foot.z).length();
                assert!(out < 0.75 * width, "{out} {width}");
                assert!(p[1] > foot.y - BUSH_SINK - 0.25, "{p:?}");
            }
        }
    }

    #[test]
    fn bushes_line_the_footpaths_beside_them_out_of_the_props_way() {
        for (circuit, _, _) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            let taken = [(middle_earth.paths[0][10], 6.0)];
            let spots = path_bush_spots(middle_earth.paths, &taken);
            let length: f32 = middle_earth
                .paths
                .iter()
                .flat_map(|path| path.windows(2).map(|w| w[0].distance(w[1])))
                .sum();
            // Many: one every few paces along them.
            assert!(
                spots.len() as f32 > 0.6 * length / BUSH_EVERY,
                "{} bushes along {length} m",
                spots.len()
            );
            for (at, _) in &spots {
                for path in middle_earth.paths {
                    for p in path_samples(path) {
                        assert!(
                            p.distance(*at) > BUSH_BESIDE - 0.2,
                            "a bush on a path at {at}"
                        );
                    }
                    // None at its ends: by a gate or the road.
                    for end in [path[0], path[path.len() - 1]] {
                        assert!(
                            end.distance(*at) > PATH_WIDTH,
                            "a bush at a path's end, {at}"
                        );
                    }
                }
                assert!(taken.iter().all(|(c, r)| c.distance(*at) > *r));
            }
        }
    }

    #[test]
    fn a_sunbeam_s_shaft_fades_out_to_its_edges_and_up_into_the_sky() {
        let glow = Glow {
            from: Vec3::new(0.0, 150.0, 0.0),
            to: Vec3::ZERO,
            illuminance: 6000.0,
            color: Color::WHITE,
            angle: 0.1,
            shaft: None,
        };
        let shaft = Shaft {
            spread: 2.5,
            light: 0.04,
        };
        let mesh = shaft_mesh(&glow, shaft);
        let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(colors)) => colors.clone(),
            _ => panic!("no colours"),
        };
        let foot = 150.0 * 0.1f32.tan();
        let mut brightest = 0.0f32;
        let mut widest_at_top = 0.0f32;
        for (p, c) in positions(&mesh).iter().zip(&colors) {
            let (out, up) = (Vec2::new(p[0], p[2]).length(), p[1]);
            // As wide at its foot as the glow's light, spreading wider up to
            // as high as it shows.
            let radius = foot * (1.0 + (shaft.spread - 1.0) * up / SHAFT_LENGTH);
            assert!(out < radius + 1e-3 && (-1e-3..=SHAFT_LENGTH + 1e-3).contains(&up));
            if up > SHAFT_LENGTH - 1e-3 {
                widest_at_top = widest_at_top.max(out);
            }
            // Dark (adding nothing) at its edges, at its top, and at its
            // foot, so as not to veil what it lights.
            if out > radius - 1e-3 || !(1e-3..=SHAFT_LENGTH - 1e-3).contains(&up) {
                assert!(c[0].max(c[1]).max(c[2]) < 1e-6, "{c:?} at {p:?}");
            }
            assert_eq!(c[3], 1.0);
            brightest = brightest.max(c[0]);
        }
        assert!(
            (widest_at_top - shaft.spread * foot).abs() < 1e-2,
            "{widest_at_top}"
        );
        // Brightest down its middle, a little over what it lights, faint
        // even there.
        assert!((brightest - shaft.light).abs() < 1e-4, "{brightest}");
    }

    #[test]
    fn the_arches_of_rock_come_out_of_the_rock_on_the_left_high_over_the_road() {
        let mut arches = 0;
        for (circuit, track, terrain) in &in_middle_earth() {
            let middle_earth = circuit.scenery.middle_earth.as_ref().unwrap();
            for arch in middle_earth.arches {
                let mesh = rock_arch(track, arch);
                let (foot, across, along) = arch_frame(track, arch);
                // Along the road ahead, and across it to the left.
                let next = track.surface_point(track.index_at(arch.along) + 3, 0.0);
                assert!((next - foot).dot(along) > 0.0);
                let mut crown = f32::MAX;
                for p in positions(&mesh) {
                    assert!(p.iter().all(|v| v.is_finite()), "{p:?}");
                    let point = Vec3::from_array(*p) - foot;
                    let (lateral, up) = (point.dot(across), point.y);
                    // Between the walls, high over the road.
                    if lateral.abs() < crate::track::WALL_DISTANCE + 0.5 {
                        assert!(up > 7.0, "{}: {up} m up, {lateral} m across", circuit.name);
                    }
                    if lateral.abs() < 2.0 {
                        crown = crown.min(up);
                    }
                }
                // Over the middle of the road, about as high as it says.
                assert!(
                    (crown - arch.height).abs() < 3.0,
                    "{}: its crown {crown} m up",
                    circuit.name
                );
                // Out of the rock on the left, where it ends inside it, and
                // its foot on the right sunk in the ground.
                let at = |t: f32| {
                    let (at, thick) = arch_curve(arch, t);
                    (foot + across * at.x + Vec3::Y * at.y, thick)
                };
                let ground = |p: Vec3| terrain.height(Vec2::new(p.x, p.z));
                let (end, thick) = at(1.0);
                assert!(
                    ground(end) > end.y + thick,
                    "{}: its end {} m up, the rock {} m",
                    circuit.name,
                    end.y,
                    ground(end)
                );
                let (start, _) = at(0.0);
                assert!(ground(start) > start.y);
                // Its outside out: on its crown, up.
                let Some(VertexAttributeValues::Float32x3(normals)) =
                    mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
                else {
                    panic!("normals");
                };
                assert!(normals[(ARCH_ALONG / 2) * ARCH_ROUND][1] > 0.5);
                // No tree in it.
                assert!(
                    arch_feet(track, arch)
                        .iter()
                        .all(|(_, reach)| *reach > arch.depth / 2.0)
                );
                arches += 1;
            }
        }
        assert_eq!(arches, 2, "Fellowship Road's two at the pass");
    }

    #[test]
    fn a_sunbeam_s_shaft_shows_coming_up_to_what_it_lights_not_from_afar() {
        let range = fading_out(SHAFT_REACH, SHAFT_FADE, true);
        // Its middle, half way up it, from the road 60 m short of what it
        // lights: whole; from 250 m off, gone.
        let to_middle = |off: f32| Vec2::new(off, SHAFT_LENGTH / 2.0).length();
        assert!(to_middle(60.0) < range.end_margin.start);
        assert!(to_middle(250.0) > range.end_margin.end);
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
