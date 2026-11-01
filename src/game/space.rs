//! The works of a lunar base and orbital station ([`Space`]): geodesic
//! biodomes with glowing windows and airlocks, solar-panel wings beside the
//! road, parabolic deep-space radio dishes that sweep the sky, six-wheeled
//! lunar rovers, Saturn-style rockets on their launchpads with umbilical
//! towers, illuminated station arches over the road, and angular basalt
//! boulders scattered across the regolith.
//!
//! Shared mesh handles and materials are reused across all instances (rovers,
//! dishes, solar wings, arches, boulders) to maximize GPU instancing.
//!
//! A theme of the game's (see [`crate::world::themes`]): a circuit's file
//! writes it in its `themes` as `"space": (...)`. The portals between the
//! realms are the engine's ([`crate::world::portals`]).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use crate::geometry::random01;
use crate::track::{Track, WALL_DISTANCE};
use crate::world::Assets3d;
use crate::world::portals;
use crate::world::props::{Footprint, row_points};
use crate::world::quads::MeshBuilder;
use crate::world::scenery::{Portal, Row, Scenery, Side, Span};
use crate::world::terrain::{SEA_LEVEL, Terrain, is_earth_realm};
use crate::world::themes::Theme;

/// The works of a lunar base and Earth space center (see
/// [`crate::game::space`]): geodesic biodomes, lunar rovers, solar panel
/// arrays, huge deep-space radio dishes turning slowly, astronomical
/// observatory domes with telescopes, orbital rockets on their launchpads,
/// illuminated station ribs over the road, interdimensional stargate portals
/// connecting Earth and the Moon, and lunar boulders with survey beacons.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Space {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub biodomes: &'static [Biodome],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rovers: &'static [Rover],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub solar_arrays: &'static [Row],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub dishes: &'static [RadioDish],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub observatories: &'static [Observatory],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rockets: &'static [Rocket],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub arches: &'static [StationArch],
    pub boulders: LunarBoulders,
}

/// An astronomical telescope observatory dome at `position` (x, z), its open
/// shutter slit and giant reflector telescope aimed towards `heading`.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observatory {
    pub position: Vec2,
    pub heading: f32,
}

/// A geodesic habitat dome beside the road, with a metallic ring base,
/// glass panels and an airlock facing the track.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biodome {
    pub along: f32,
    pub side: Side,
    pub radius: f32,
}

/// A six-wheeled lunar rover beside the road, its gold-foil chassis facing
/// `heading` (relative to the road direction).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rover {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub heading: f32,
}

/// A parabolic deep-space radio telescope dish at `position` (x, z), its
/// dish pointed towards `heading` and turning slowly.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadioDish {
    pub position: Vec2,
    pub heading: f32,
}

/// An orbital rocket standing on its launchpad at `position` (x, z), with an
/// umbilical launch tower beside it facing `heading`.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rocket {
    pub position: Vec2,
    pub heading: f32,
}

/// A sleek space-station structural rib/arch spanning over the road at
/// `along` metres from the finish line, with glowing cyan-white beacon strips.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationArch {
    pub along: f32,
}

/// Lunar boulders scattered along `spans` on both sides of the road.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LunarBoulders {
    pub max: usize,
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub spans: &'static [Span],
}

/// How far out from the centre line of the road a station arch's legs stand
/// (m), and how high its crown clears the asphalt (m).
const ARCH_SPAN: f32 = WALL_DISTANCE + 2.8;
const ARCH_HEIGHT: f32 = 9.2;

/// How far a radio dish sweeps either side of its heading (rad), and how fast
/// (rad/s).
const DISH_SWEEP: f32 = 0.55;
const DISH_RATE: f32 = 0.45;

/// Lunar boulders stand at least this far from the centre line of any road,
/// and at most this far from that of the road they are put by (m).
const BOULDER_NEAR: f32 = WALL_DISTANCE + 4.5;
const BOULDER_FAR: f32 = 85.0;
const BOULDER_SHAPES: u32 = 3;

/// Colours of the lunar base (sRGB).
const HULL_WHITE: Color = Color::srgb(0.88, 0.90, 0.94);
const HULL_SILVER: Color = Color::srgb(0.64, 0.68, 0.75);
const HULL_DARK: Color = Color::srgb(0.26, 0.29, 0.35);
const GOLD_FOIL: Color = Color::srgb(0.88, 0.66, 0.16);
const SOLAR_BLUE: Color = Color::srgb(0.10, 0.22, 0.52);
const SOLAR_GRID: Color = Color::srgb(0.32, 0.48, 0.78);
const DOME_GLASS: Color = Color::srgb(0.20, 0.42, 0.58);
const ROCKET_ORANGE: Color = Color::srgb(0.86, 0.34, 0.12);
const GANTRY_RED: Color = Color::srgb(0.76, 0.18, 0.14);
const BASALT_LIGHT: Color = Color::srgb(0.56, 0.60, 0.68);
const BASALT_DARK: Color = Color::srgb(0.34, 0.37, 0.44);

/// A radio dish's parabolic reflector head, sweeping slowly across the sky.
#[derive(Component)]
pub struct DishSweep {
    base_yaw: f32,
    phase: f32,
}

/// Turns the radio dishes' reflector heads slowly back and forth across the
/// sky.
pub fn turn_dishes(time: Res<Time>, mut dishes: Query<(&mut Transform, &DishSweep)>) {
    let t = time.elapsed_secs() * DISH_RATE;
    for (mut transform, dish) in &mut dishes {
        let yaw = dish.base_yaw + DISH_SWEEP * (t + dish.phase).sin();
        // Tilted back so the reflector (facing -Z) looks up at the sky.
        let pitch = 0.85 + 0.12 * ((t + dish.phase) * 0.7).cos();
        transform.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
    }
}

/// The base's own systems: the radio dishes sweeping the sky.
pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, turn_dishes);
    }
}

impl Theme for Space {
    /// Puts the lunar base's structures, vehicles, antennas, observatories,
    /// solar arrays and boulders by the road, the boulders off the base and
    /// the circuit's portals, and returns the footprints they occupy.
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        spawn_space(world, track, terrain, self, scenery.portals, assets)
    }
}

/// Puts the lunar base by the road (see [`Theme::spawn`]).
fn spawn_space(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    space: &Space,
    portals: &[Portal],
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let hull = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.55,
        metallic: 0.25,
        ..default()
    });
    let glow = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.92, 1.0),
        emissive: LinearRgba::rgb(2.8, 4.8, 6.8),
        ..default()
    });
    let beacon = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.35, 0.15),
        emissive: LinearRgba::rgb(6.0, 1.2, 0.4),
        ..default()
    });
    let mut taken = Vec::new();
    let mut tree_clearings = Vec::new();
    for dome in space.biodomes {
        let spot = spawn_biodome(world, track, terrain, dome, &hull, &glow, assets);
        taken.push((spot, dome.radius + 6.0));
        tree_clearings.extend(sightline_clearings(track, spot, dome.radius + 22.0));
    }
    spawn_rovers(world, track, terrain, space.rovers, &hull, &glow, assets);
    for rover in space.rovers {
        let (spot, _) = beside_road(track, rover.along, rover.side.sign() * rover.lateral);
        taken.push((spot, 7.0));
    }
    spawn_dishes(world, track, terrain, space.dishes, &hull, &beacon, assets);
    for dish in space.dishes {
        taken.push((dish.position, 16.0));
        tree_clearings.extend(sightline_clearings(track, dish.position, 38.0));
    }
    spawn_observatories(
        world,
        track,
        terrain,
        space.observatories,
        &hull,
        &glow,
        assets,
    );
    for obs in space.observatories {
        taken.push((obs.position, 22.0));
        tree_clearings.extend(sightline_clearings(track, obs.position, 44.0));
    }
    spawn_rockets(world, track, terrain, space.rockets, &hull, &beacon, assets);
    for rocket in space.rockets {
        taken.push((rocket.position, 22.0));
        tree_clearings.extend(sightline_clearings(track, rocket.position, 42.0));
    }
    spawn_station_arches(world, track, terrain, space.arches, &hull, &glow, assets);
    taken.extend(spawn_solar_arrays(
        world,
        track,
        terrain,
        space.solar_arrays,
        &hull,
        assets,
    ));
    // Off the ground the base takes and the portals'.
    let around: Vec<Footprint> = portals::footprints(track, portals)
        .into_iter()
        .chain(taken.iter().copied())
        .collect();
    taken.extend(spawn_lunar_boulders(
        world,
        track,
        terrain,
        &space.boulders,
        &around,
        &hull,
        assets,
    ));
    taken.extend(tree_clearings);
    taken
}

/// Where something `lateral` metres from the centre line on `side` of the
/// point `along` the lap stands, and its frame: X towards the road, Z along
/// it, Y up.
fn beside_road(track: &Track, along: f32, lateral: f32) -> (Vec2, Quat) {
    let index = track.index_at(along);
    let spot = track.offset_point(index, lateral);
    let toward_road = (track.offset_point(index, 0.0) - spot).normalize_or(Vec2::X);
    let x = Vec3::new(toward_road.x, 0.0, toward_road.y);
    (
        spot,
        Quat::from_mat3(&Mat3::from_cols(x, Vec3::Y, x.cross(Vec3::Y))),
    )
}

/// Elevation at which a major landmark at `spot` stands so it is never sunk
/// below the nearest road surface when built on a sloped mountainside.
fn road_level_base(track: &Track, terrain: &Terrain, spot: Vec2) -> f32 {
    let nearest = track.nearest(spot);
    let road_y = track.surface_point(nearest.index, 0.0).y;
    terrain.height(spot).max(road_y)
}

/// Footprints that keep both the area around a landmark and the approaching
/// roadside sightline clear of trees and houses.
fn sightline_clearings(track: &Track, spot: Vec2, radius: f32) -> [Footprint; 3] {
    let nearest = track.nearest(spot);
    let along = track.along(&nearest);
    let center = track.offset_point(nearest.index, 0.0);
    let side_dir = (spot - center).normalize_or(Vec2::X);
    let lateral = side_dir.dot(track.offset_point(nearest.index, 1.0) - center);
    let sign = if lateral >= 0.0 { 1.0 } else { -1.0 };
    let upstream_1 =
        track.offset_point(track.index_at(along - 28.0), sign * (WALL_DISTANCE + 14.0));
    let upstream_2 =
        track.offset_point(track.index_at(along - 56.0), sign * (WALL_DISTANCE + 14.0));
    [(spot, radius), (upstream_1, 24.0), (upstream_2, 22.0)]
}

/// The lowest ground under the corners of a rectangle `half` (x, z) about
/// `spot`, in the frame `rotation`.
fn lowest_ground(terrain: &Terrain, spot: Vec2, rotation: Quat, half: Vec2) -> f32 {
    [
        (-1.0, -1.0),
        (1.0, -1.0),
        (1.0, 1.0),
        (-1.0, 1.0),
        (0.0, 0.0),
    ]
    .into_iter()
    .map(|(sx, sz)| {
        let corner = rotation * Vec3::new(sx * half.x, 0.0, sz * half.y);
        terrain.height(spot + Vec2::new(corner.x, corner.z))
    })
    .fold(f32::INFINITY, f32::min)
}

/// Adds a closed axis-aligned box from `low` to `high` to `mesh`.
fn add_box(mesh: &mut MeshBuilder, low: Vec3, high: Vec3, color: Color) {
    let center = (low + high) / 2.0;
    let width = high.x - low.x;
    let height = high.y - low.y;
    mesh.beam(
        Vec3::new(center.x, center.y, low.z),
        Vec3::new(center.x, center.y, high.z),
        width,
        height,
        color,
    );
}

/// Spawns a geodesic lunar space-station habitat dome with a towering command
/// cupola, pressurized airlock tunnel reaching the roadside, and glowing
/// viewport windows.
fn spawn_biodome(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    dome: &Biodome,
    hull: &Handle<StandardMaterial>,
    glow: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Vec2 {
    let r = dome.radius;
    let lateral = dome.side.sign() * (WALL_DISTANCE + r + 2.5);
    let (spot, rotation) = beside_road(track, dome.along, lateral);
    let base = road_level_base(track, terrain, spot);
    let (body, windows) = biodome_meshes(r);
    let body_handle = assets.meshes.add(body);
    let win_handle = assets.meshes.add(windows);
    let transform = Transform::from_xyz(spot.x, base, spot.y).with_rotation(rotation);
    world.spawn((Mesh3d(body_handle), MeshMaterial3d(hull.clone()), transform));
    world.spawn((Mesh3d(win_handle), MeshMaterial3d(glow.clone()), transform));
    spot
}

/// Builds the opaque hull/struts mesh and the emissive viewport mesh of a
/// geodesic space-station biodome of radius `r`, crowned by an orbital command
/// tower so the complex is unmistakable from the road.
fn biodome_meshes(r: f32) -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut glow = MeshBuilder::default();
    let sides = 12usize;
    let rings = 5usize;
    let footing = -22.0;
    let collar = 4.2;
    for i in 0..sides {
        let a0 = i as f32 / sides as f32 * TAU;
        let a1 = (i + 1) as f32 / sides as f32 * TAU;
        let p0 = Vec2::from_angle(a0) * r;
        let p1 = Vec2::from_angle(a1) * r;
        let outward = Vec3::new(p0.x + p1.x, 0.0, p0.y + p1.y).normalize_or_zero();
        hull.quad(
            [
                (Vec3::new(p0.x, footing, p0.y), Vec2::ZERO),
                (Vec3::new(p1.x, footing, p1.y), Vec2::X),
                (Vec3::new(p1.x, collar, p1.y), Vec2::ONE),
                (Vec3::new(p0.x, collar, p0.y), Vec2::Y),
            ],
            outward,
            HULL_SILVER,
        );
        // Glowing viewport band round the collar (skipping the airlock on +X).
        if i != 0 && i != sides - 1 {
            let w0 = p0.lerp(p1, 0.16);
            let w1 = p0.lerp(p1, 0.84);
            let n = outward * 0.08;
            glow.quad(
                [
                    (Vec3::new(w0.x, 1.4, w0.y) + n, Vec2::ZERO),
                    (Vec3::new(w1.x, 1.4, w1.y) + n, Vec2::X),
                    (Vec3::new(w1.x, 3.1, w1.y) + n, Vec2::ONE),
                    (Vec3::new(w0.x, 3.1, w0.y) + n, Vec2::Y),
                ],
                outward,
                Color::WHITE,
            );
        }
        for ring in 0..rings {
            let e0 = ring as f32 / rings as f32 * FRAC_PI_2;
            let e1 = (ring + 1) as f32 / rings as f32 * FRAC_PI_2;
            let (r0, y0) = (r * e0.cos(), collar + r * 0.86 * e0.sin());
            let (r1, y1) = (r * e1.cos(), collar + r * 0.86 * e1.sin());
            let c00 = Vec3::new(a0.cos() * r0, y0, a0.sin() * r0);
            let c10 = Vec3::new(a1.cos() * r0, y0, a1.sin() * r0);
            let c11 = Vec3::new(a1.cos() * r1, y1, a1.sin() * r1);
            let c01 = Vec3::new(a0.cos() * r1, y1, a0.sin() * r1);
            let face = (c00 + c10 + c11 + c01) / 4.0 - Vec3::Y * collar;
            let panel_color = if (ring + i) % 2 == 0 {
                DOME_GLASS
            } else {
                HULL_WHITE
            };
            if ring + 1 == rings {
                hull.triangle(
                    [(c00, Vec2::ZERO), (c10, Vec2::X), (c01, Vec2::Y)],
                    face,
                    HULL_WHITE,
                );
            } else {
                hull.quad(
                    [
                        (c00, Vec2::ZERO),
                        (c10, Vec2::X),
                        (c11, Vec2::ONE),
                        (c01, Vec2::Y),
                    ],
                    face,
                    panel_color,
                );
            }
            // Geodesic rib along the ring and meridian, with glowing meridian strips.
            hull.beam(c00, c10, 0.28, 0.28, HULL_SILVER);
            hull.beam(c00, c01, 0.26, 0.26, HULL_SILVER);
            if i % 3 == 0 && ring < rings - 1 {
                glow.beam(c00 * 1.01, c01 * 1.01, 0.18, 0.18, Color::WHITE);
            }
        }
    }

    // Towering Space-Station Command & Docking Tower rising above the biodome.
    let apex_y = collar + r * 0.86;
    let tower_top = apex_y + 14.5;
    add_box(
        &mut hull,
        Vec3::new(-2.6, apex_y - 3.0, -2.6),
        Vec3::new(2.6, tower_top - 4.0, 2.6),
        HULL_WHITE,
    );
    add_box(
        &mut hull,
        Vec3::new(-3.2, tower_top - 4.5, -3.2),
        Vec3::new(3.2, tower_top - 3.6, 3.2),
        GOLD_FOIL,
    );
    // Panoramic command deck cupola at the top of the station tower.
    add_box(
        &mut hull,
        Vec3::new(-4.4, tower_top - 3.6, -4.4),
        Vec3::new(4.4, tower_top, 4.4),
        HULL_DARK,
    );
    add_box(
        &mut glow,
        Vec3::new(-4.55, tower_top - 2.8, -4.55),
        Vec3::new(4.55, tower_top - 0.8, 4.55),
        Color::WHITE,
    );
    // Solar radiator wings and deep-space comms spire on the command tower.
    for z_sign in [-1.0, 1.0] {
        add_box(
            &mut hull,
            Vec3::new(-1.8, tower_top - 6.8, z_sign * 2.6),
            Vec3::new(1.8, tower_top - 5.4, z_sign * 11.5),
            SOLAR_BLUE,
        );
        hull.beam(
            Vec3::new(0.0, tower_top - 6.1, z_sign * 2.6),
            Vec3::new(0.0, tower_top - 6.1, z_sign * 11.8),
            0.35,
            0.35,
            GOLD_FOIL,
        );
    }
    hull.beam(
        Vec3::new(0.0, tower_top, 0.0),
        Vec3::new(0.0, tower_top + 6.5, 0.0),
        0.35,
        0.35,
        HULL_SILVER,
    );
    add_box(
        &mut glow,
        Vec3::new(-0.55, tower_top + 6.2, -0.55),
        Vec3::new(0.55, tower_top + 7.2, 0.55),
        Color::WHITE,
    );

    // Pressurized airlock module facing the road (+X).
    add_box(
        &mut hull,
        Vec3::new(r - 1.2, footing, -2.6),
        Vec3::new(r + 2.2, 4.4, 2.6),
        HULL_WHITE,
    );
    add_box(
        &mut hull,
        Vec3::new(r + 1.9, 0.2, -1.6),
        Vec3::new(r + 2.4, 3.6, 1.6),
        GOLD_FOIL,
    );
    glow.quad(
        [
            (Vec3::new(r + 2.35, 3.6, -1.5), Vec2::ZERO),
            (Vec3::new(r + 2.35, 3.6, 1.5), Vec2::X),
            (Vec3::new(r + 2.35, 4.1, 1.5), Vec2::ONE),
            (Vec3::new(r + 2.35, 4.1, -1.5), Vec2::Y),
        ],
        Vec3::X,
        Color::WHITE,
    );
    (hull.build(), glow.build())
}

/// Spawns six-wheeled pressurized lunar rovers sharing a single hull mesh and
/// headlamp mesh.
fn spawn_rovers(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    rovers: &[Rover],
    hull: &Handle<StandardMaterial>,
    glow: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if rovers.is_empty() {
        return;
    }
    let (body, lights) = rover_meshes();
    let body_mesh = assets.meshes.add(body);
    let lights_mesh = assets.meshes.add(lights);
    for rover in rovers {
        let (spot, _) = beside_road(track, rover.along, rover.side.sign() * rover.lateral);
        let rotation = Quat::from_rotation_y(rover.heading);
        let base = lowest_ground(terrain, spot, rotation, Vec2::new(1.8, 3.0));
        let transform = Transform::from_xyz(spot.x, base, spot.y).with_rotation(rotation);
        world.spawn((
            Mesh3d(body_mesh.clone()),
            MeshMaterial3d(hull.clone()),
            transform,
        ));
        world.spawn((
            Mesh3d(lights_mesh.clone()),
            MeshMaterial3d(glow.clone()),
            transform,
        ));
    }
}

/// Builds the shared body mesh and headlamp mesh of a six-wheeled lunar rover.
fn rover_meshes() -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut glow = MeshBuilder::default();
    // Chassis and gold thermal foil battery bay.
    add_box(
        &mut hull,
        Vec3::new(-1.3, 0.55, -2.6),
        Vec3::new(1.3, 1.15, 2.6),
        GOLD_FOIL,
    );
    // Pressurized crew cabin with angled front windshield.
    add_box(
        &mut hull,
        Vec3::new(-1.45, 1.15, -1.6),
        Vec3::new(1.45, 2.55, 2.1),
        HULL_WHITE,
    );
    add_box(
        &mut hull,
        Vec3::new(-1.25, 1.55, -1.68),
        Vec3::new(1.25, 2.30, -1.55),
        DOME_GLASS,
    );
    // Roof solar panel and high-gain antenna mast.
    add_box(
        &mut hull,
        Vec3::new(-1.3, 2.55, -0.4),
        Vec3::new(1.3, 2.68, 1.9),
        SOLAR_BLUE,
    );
    hull.beam(
        Vec3::new(0.0, 2.55, -1.1),
        Vec3::new(0.0, 3.65, -1.1),
        0.12,
        0.12,
        HULL_SILVER,
    );
    add_box(
        &mut hull,
        Vec3::new(-0.55, 3.55, -1.25),
        Vec3::new(0.55, 3.95, -0.95),
        HULL_WHITE,
    );
    // Six mesh-tire wheels on suspension struts.
    for z in [-1.85, 0.0, 1.85] {
        for side in [-1.0, 1.0] {
            let x = side * 1.65;
            hull.beam(
                Vec3::new(side * 1.2, 0.85, z),
                Vec3::new(x, 0.55, z),
                0.22,
                0.22,
                HULL_SILVER,
            );
            add_box(
                &mut hull,
                Vec3::new(x - 0.28, 0.0, z - 0.55),
                Vec3::new(x + 0.28, 1.10, z + 0.55),
                HULL_DARK,
            );
            add_box(
                &mut hull,
                Vec3::new(x - 0.32, 0.30, z - 0.25),
                Vec3::new(x + 0.32, 0.80, z + 0.25),
                GOLD_FOIL,
            );
        }
    }
    // Dual front headlamps facing -Z.
    for x in [-0.85, 0.85] {
        glow.quad(
            [
                (Vec3::new(x - 0.25, 1.22, -2.64), Vec2::ZERO),
                (Vec3::new(x + 0.25, 1.22, -2.64), Vec2::X),
                (Vec3::new(x + 0.25, 1.48, -2.64), Vec2::ONE),
                (Vec3::new(x - 0.25, 1.48, -2.64), Vec2::Y),
            ],
            Vec3::NEG_Z,
            Color::WHITE,
        );
    }
    (hull.build(), glow.build())
}

/// Spawns giant deep-space parabolic radio dishes that share their pedestal and
/// reflector meshes, anchored at or above the adjacent road surface.
fn spawn_dishes(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    dishes: &[RadioDish],
    hull: &Handle<StandardMaterial>,
    beacon: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if dishes.is_empty() {
        return;
    }
    let pedestal = assets.meshes.add(dish_pedestal_mesh());
    let (reflector, tip_light) = dish_reflector_meshes();
    let reflector_mesh = assets.meshes.add(reflector);
    let tip_mesh = assets.meshes.add(tip_light);
    for (index, dish) in dishes.iter().enumerate() {
        let base = road_level_base(track, terrain, dish.position);
        let phase = index as f32 * 1.3;
        world
            .spawn((
                Mesh3d(pedestal.clone()),
                MeshMaterial3d(hull.clone()),
                Transform::from_xyz(dish.position.x, base, dish.position.y),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Mesh3d(reflector_mesh.clone()),
                        MeshMaterial3d(hull.clone()),
                        Transform::from_xyz(0.0, 15.8, 0.0),
                        DishSweep {
                            base_yaw: dish.heading,
                            phase,
                        },
                    ))
                    .with_children(|head| {
                        head.spawn((
                            Mesh3d(tip_mesh.clone()),
                            MeshMaterial3d(beacon.clone()),
                            Transform::default(),
                        ));
                    });
            });
    }
}

fn dish_pedestal_mesh() -> Mesh {
    let mut mesh = MeshBuilder::default();
    add_box(
        &mut mesh,
        Vec3::new(-6.2, -24.0, -6.2),
        Vec3::new(6.2, 3.2, 6.2),
        HULL_DARK,
    );
    add_box(
        &mut mesh,
        Vec3::new(-2.8, 3.2, -2.8),
        Vec3::new(2.8, 15.8, 2.8),
        HULL_WHITE,
    );
    add_box(
        &mut mesh,
        Vec3::new(-3.4, 8.5, -3.4),
        Vec3::new(3.4, 9.6, 3.4),
        GOLD_FOIL,
    );
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        mesh.beam(
            Vec3::new(sx * 5.4, 2.8, sz * 5.4),
            Vec3::new(sx * 2.2, 15.0, sz * 2.2),
            0.70,
            0.70,
            HULL_SILVER,
        );
    }
    mesh.build()
}

fn dish_reflector_meshes() -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut beacon = MeshBuilder::default();
    let sides = 16usize;
    let r_inner = 2.6;
    let r_outer = 13.5;
    for i in 0..sides {
        let a0 = i as f32 / sides as f32 * TAU;
        let a1 = (i + 1) as f32 / sides as f32 * TAU;
        let p0_in = Vec3::new(a0.cos() * r_inner, a0.sin() * r_inner, -0.7);
        let p1_in = Vec3::new(a1.cos() * r_inner, a1.sin() * r_inner, -0.7);
        let p1_out = Vec3::new(a1.cos() * r_outer, a1.sin() * r_outer, -5.4);
        let p0_out = Vec3::new(a0.cos() * r_outer, a0.sin() * r_outer, -5.4);
        // Front concave surface and back structural shell.
        hull.quad(
            [
                (p0_in, Vec2::ZERO),
                (p1_in, Vec2::X),
                (p1_out, Vec2::ONE),
                (p0_out, Vec2::Y),
            ],
            Vec3::NEG_Z,
            HULL_WHITE,
        );
        hull.quad(
            [
                (p0_in + Vec3::Z * 0.42, Vec2::ZERO),
                (p0_out + Vec3::Z * 0.42, Vec2::Y),
                (p1_out + Vec3::Z * 0.42, Vec2::ONE),
                (p1_in + Vec3::Z * 0.42, Vec2::X),
            ],
            Vec3::Z,
            HULL_SILVER,
        );
        hull.triangle(
            [(Vec3::ZERO, Vec2::ZERO), (p0_in, Vec2::X), (p1_in, Vec2::Y)],
            Vec3::NEG_Z,
            GOLD_FOIL,
        );
    }
    // Quadripod feed horn legs meeting at the focal point (-Z).
    let focus = Vec3::new(0.0, 0.0, -10.2);
    for k in 0..4 {
        let angle = k as f32 / 4.0 * TAU + 0.4;
        let rim = Vec3::new(angle.cos() * 9.2, angle.sin() * 9.2, -3.5);
        hull.beam(rim, focus, 0.36, 0.36, HULL_SILVER);
    }
    add_box(
        &mut hull,
        focus - Vec3::splat(0.95),
        focus + Vec3::splat(0.95),
        GOLD_FOIL,
    );
    add_box(
        &mut beacon,
        focus + Vec3::new(-0.55, -0.55, -1.45),
        focus + Vec3::new(0.55, 0.55, -0.85),
        Color::WHITE,
    );
    (hull.build(), beacon.build())
}

/// Spawns astronomical telescope observatory domes (`Observatory`), sharing a
/// single building/dome/telescope mesh and glowing optical mirror/window mesh,
/// anchored at or above the adjacent road surface so they tower over the road.
fn spawn_observatories(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    observatories: &[Observatory],
    hull: &Handle<StandardMaterial>,
    glow: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if observatories.is_empty() {
        return;
    }
    let (body, optics) = observatory_meshes();
    let body_mesh = assets.meshes.add(body);
    let optics_mesh = assets.meshes.add(optics);
    for obs in observatories {
        let rotation = Quat::from_rotation_y(obs.heading);
        let base = road_level_base(track, terrain, obs.position);
        let transform =
            Transform::from_xyz(obs.position.x, base, obs.position.y).with_rotation(rotation);
        world.spawn((
            Mesh3d(body_mesh.clone()),
            MeshMaterial3d(hull.clone()),
            transform,
        ));
        world.spawn((
            Mesh3d(optics_mesh.clone()),
            MeshMaterial3d(glow.clone()),
            transform,
        ));
    }
}

/// Builds the cylindrical base, hemispherical shuttered dome, and giant optical
/// Cassegrain reflector telescope of an astronomical observatory.
fn observatory_meshes() -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut glow = MeshBuilder::default();
    let sides = 14usize;
    let rings = 6usize;
    let r_base = 18.2f32;
    let r_dome = 17.5f32;
    let footing = -24.0f32;
    let deck_y = 12.6f32;

    // 1. Deep mountainside plinth and tall white cylindrical observatory drum.
    for i in 0..sides {
        let a0 = i as f32 / sides as f32 * TAU;
        let a1 = (i + 1) as f32 / sides as f32 * TAU;
        let p0 = Vec2::from_angle(a0) * r_base;
        let p1 = Vec2::from_angle(a1) * r_base;
        let outward = Vec3::new(p0.x + p1.x, 0.0, p0.y + p1.y).normalize_or_zero();
        hull.quad(
            [
                (Vec3::new(p0.x * 1.05, footing, p0.y * 1.05), Vec2::ZERO),
                (Vec3::new(p1.x * 1.05, footing, p1.y * 1.05), Vec2::X),
                (Vec3::new(p1.x * 1.05, 0.8, p1.y * 1.05), Vec2::ONE),
                (Vec3::new(p0.x * 1.05, 0.8, p0.y * 1.05), Vec2::Y),
            ],
            outward,
            HULL_DARK,
        );
        hull.quad(
            [
                (Vec3::new(p0.x, 0.8, p0.y), Vec2::ZERO),
                (Vec3::new(p1.x, 0.8, p1.y), Vec2::X),
                (Vec3::new(p1.x, deck_y, p1.y), Vec2::ONE),
                (Vec3::new(p0.x, deck_y, p0.y), Vec2::Y),
            ],
            outward,
            HULL_WHITE,
        );
        // External vertical structural buttresses and observation gallery collar.
        hull.beam(
            Vec3::new(p0.x * 1.05, footing, p0.y * 1.05),
            Vec3::new(p0.x * 1.03, deck_y, p0.y * 1.03),
            0.95,
            0.95,
            HULL_SILVER,
        );
        hull.beam(
            Vec3::new(p0.x * 1.07, deck_y, p0.y * 1.07),
            Vec3::new(p1.x * 1.07, deck_y, p1.y * 1.07),
            1.5,
            1.2,
            GOLD_FOIL,
        );
        // Two tiers of glowing cyan control-room & observation gallery windows.
        let w0 = p0.lerp(p1, 0.16);
        let w1 = p0.lerp(p1, 0.84);
        let n = outward * 0.12;
        for (wy0, wy1) in [(4.2, 6.1), (9.2, 10.9)] {
            glow.quad(
                [
                    (Vec3::new(w0.x, wy0, w0.y) + n, Vec2::ZERO),
                    (Vec3::new(w1.x, wy0, w1.y) + n, Vec2::X),
                    (Vec3::new(w1.x, wy1, w1.y) + n, Vec2::ONE),
                    (Vec3::new(w0.x, wy1, w0.y) + n, Vec2::Y),
                ],
                outward,
                Color::WHITE,
            );
        }

        // 2. Hemispherical dome with a wide open observing slit on the -Z side
        // (sectors 9, 10, and 11), framed by massive golden shutter arches.
        let is_slit = (9..=11).contains(&i);
        for ring in 0..rings {
            let e0 = ring as f32 / rings as f32 * FRAC_PI_2;
            let e1 = (ring + 1) as f32 / rings as f32 * FRAC_PI_2;
            let (r0, y0) = (r_dome * e0.cos(), deck_y + r_dome * e0.sin());
            let (r1, y1) = (r_dome * e1.cos(), deck_y + r_dome * e1.sin());
            let c00 = Vec3::new(a0.cos() * r0, y0, a0.sin() * r0);
            let c10 = Vec3::new(a1.cos() * r0, y0, a1.sin() * r0);
            let c11 = Vec3::new(a1.cos() * r1, y1, a1.sin() * r1);
            let c01 = Vec3::new(a0.cos() * r1, y1, a0.sin() * r1);
            let face = (c00 + c10 + c11 + c01) / 4.0 - Vec3::Y * deck_y;
            if is_slit && ring < rings - 1 {
                // Recessed observatory bay with glowing cyan instrument strips.
                let i00 = Vec3::new(a0.cos() * (r0 * 0.74), y0, a0.sin() * (r0 * 0.74));
                let i10 = Vec3::new(a1.cos() * (r0 * 0.74), y0, a1.sin() * (r0 * 0.74));
                let i11 = Vec3::new(a1.cos() * (r1 * 0.74), y1, a1.sin() * (r1 * 0.74));
                let i01 = Vec3::new(a0.cos() * (r1 * 0.74), y1, a0.sin() * (r1 * 0.74));
                hull.quad(
                    [
                        (i00, Vec2::ZERO),
                        (i10, Vec2::X),
                        (i11, Vec2::ONE),
                        (i01, Vec2::Y),
                    ],
                    face,
                    HULL_DARK,
                );
                if i == 9 {
                    hull.beam(c00, c01, 1.25, 1.25, GOLD_FOIL);
                    glow.beam(c00 * 1.02, c01 * 1.02, 0.36, 0.36, Color::WHITE);
                }
                if i == 11 {
                    hull.beam(c10, c11, 1.25, 1.25, GOLD_FOIL);
                    glow.beam(c10 * 1.02, c11 * 1.02, 0.36, 0.36, Color::WHITE);
                }
            } else if ring + 1 == rings {
                hull.triangle(
                    [(c00, Vec2::ZERO), (c10, Vec2::X), (c01, Vec2::Y)],
                    face,
                    HULL_SILVER,
                );
            } else {
                let panel_color = if ring % 2 == 0 {
                    HULL_WHITE
                } else {
                    HULL_SILVER
                };
                hull.quad(
                    [
                        (c00, Vec2::ZERO),
                        (c10, Vec2::X),
                        (c11, Vec2::ONE),
                        (c01, Vec2::Y),
                    ],
                    face,
                    panel_color,
                );
                hull.beam(c00, c10, 0.34, 0.34, HULL_SILVER);
                hull.beam(c00, c01, 0.28, 0.28, HULL_SILVER);
            }
        }
    }

    // 3. Giant Optical Cassegrain Telescope protruding 21 m out of the dome
    // slit (-Z, +Y) directly toward the road, with a laser guide-star beam.
    let aim = Vec3::new(0.0, 0.50, -0.866).normalize();
    let right = Vec3::X;
    let up = aim.cross(right).normalize();
    let pivot = Vec3::new(0.0, deck_y + 4.4, -2.2);
    let barrel_end = pivot + aim * 21.5;
    // Equatorial fork mount arms.
    for sx in [-1.0, 1.0] {
        hull.beam(
            Vec3::new(sx * 4.8, deck_y, 0.0),
            pivot + right * (sx * 4.2),
            1.45,
            1.45,
            HULL_DARK,
        );
    }
    // Primary mirror cell, central optical barrel, and gold baffle ring.
    hull.beam(pivot - aim * 3.0, pivot + aim * 9.5, 5.8, 5.8, HULL_WHITE);
    hull.beam(pivot + aim * 9.5, barrel_end, 5.0, 5.0, HULL_DARK);
    hull.beam(
        barrel_end - aim * 1.4,
        barrel_end + aim * 0.2,
        5.5,
        5.5,
        GOLD_FOIL,
    );
    // Open Serrurier truss struts along the upper telescope tube.
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let offset = (right * sx + up * sy) * 2.85;
        hull.beam(
            pivot + aim * 2.5 + offset,
            barrel_end + offset * 0.96,
            0.44,
            0.44,
            GOLD_FOIL,
        );
    }
    // Glowing cyan primary/corrector aperture lens and adaptive-optics guide beam.
    let lens_center = barrel_end + aim * 0.25;
    let lens_r = 2.45;
    glow.quad(
        [
            (lens_center - right * lens_r - up * lens_r, Vec2::ZERO),
            (lens_center + right * lens_r - up * lens_r, Vec2::X),
            (lens_center + right * lens_r + up * lens_r, Vec2::ONE),
            (lens_center - right * lens_r + up * lens_r, Vec2::Y),
        ],
        aim,
        Color::WHITE,
    );
    glow.beam(
        lens_center,
        lens_center + aim * 16.0,
        0.26,
        0.26,
        Color::WHITE,
    );
    (hull.build(), glow.build())
}

/// Spawns tall multi-stage rockets and red launch gantry towers on concrete
/// launchpads, sharing mesh handles across all launchpads and anchored at or
/// above road level.
fn spawn_rockets(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    rockets: &[Rocket],
    hull: &Handle<StandardMaterial>,
    beacon: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if rockets.is_empty() {
        return;
    }
    let (structure, lights) = rocket_meshes();
    let structure_mesh = assets.meshes.add(structure);
    let lights_mesh = assets.meshes.add(lights);
    for rocket in rockets {
        let rotation = Quat::from_rotation_y(rocket.heading);
        let base = road_level_base(track, terrain, rocket.position);
        let transform =
            Transform::from_xyz(rocket.position.x, base, rocket.position.y).with_rotation(rotation);
        world.spawn((
            Mesh3d(structure_mesh.clone()),
            MeshMaterial3d(hull.clone()),
            transform,
        ));
        world.spawn((
            Mesh3d(lights_mesh.clone()),
            MeshMaterial3d(beacon.clone()),
            transform,
        ));
    }
}

fn rocket_meshes() -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut beacon = MeshBuilder::default();
    // Deep blast pad foundation and flame trench base.
    add_box(
        &mut hull,
        Vec3::new(-14.0, -24.0, -14.0),
        Vec3::new(14.0, 2.4, 14.0),
        HULL_DARK,
    );
    add_box(
        &mut hull,
        Vec3::new(-10.2, 2.4, -10.2),
        Vec3::new(10.2, 3.8, 10.2),
        HULL_SILVER,
    );
    // Four swept aerodynamic fins at the base of the first stage.
    for (sx, sz) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        let dir = Vec3::new(sx, 0.0, sz);
        hull.beam(
            dir * 3.4 + Vec3::Y * 5.8,
            dir * 6.6 + Vec3::Y * 3.8,
            0.85,
            5.2,
            ROCKET_ORANGE,
        );
    }
    // Cylindrical rocket stages (10-sided prism): white booster, orange core,
    // white upper stage, and nose cone reaching 56 m above the pad.
    let stages = [
        (3.8, 20.0, 3.6, 3.6, HULL_WHITE),
        (20.0, 35.0, 3.5, 3.5, ROCKET_ORANGE),
        (35.0, 47.0, 3.4, 2.6, HULL_WHITE),
        (47.0, 56.0, 2.6, 0.25, HULL_WHITE),
    ];
    let sides = 10usize;
    for (y0, y1, r0, r1, color) in stages {
        for i in 0..sides {
            let a0 = i as f32 / sides as f32 * TAU;
            let a1 = (i + 1) as f32 / sides as f32 * TAU;
            let c00 = Vec3::new(a0.cos() * r0, y0, a0.sin() * r0);
            let c10 = Vec3::new(a1.cos() * r0, y0, a1.sin() * r0);
            let c11 = Vec3::new(a1.cos() * r1, y1, a1.sin() * r1);
            let c01 = Vec3::new(a0.cos() * r1, y1, a0.sin() * r1);
            let outward = Vec3::new((a0 + a1).cos(), 0.2, (a0 + a1).sin()).normalize_or_zero();
            hull.quad(
                [
                    (c00, Vec2::ZERO),
                    (c10, Vec2::X),
                    (c11, Vec2::ONE),
                    (c01, Vec2::Y),
                ],
                outward,
                color,
            );
        }
    }
    // Launch umbilical gantry tower beside the rocket (+X).
    for (lx, lz) in [(6.4, -2.7), (10.4, -2.7), (6.4, 2.7), (10.4, 2.7)] {
        hull.beam(
            Vec3::new(lx, 3.8, lz),
            Vec3::new(lx, 51.0, lz),
            0.52,
            0.52,
            GANTRY_RED,
        );
    }
    for level in (9..=49).step_by(6) {
        let y = level as f32;
        add_box(
            &mut hull,
            Vec3::new(6.2, y - 0.35, -2.9),
            Vec3::new(10.6, y + 0.35, 2.9),
            GANTRY_RED,
        );
        // Swing arm reaching across to the rocket stage.
        hull.beam(
            Vec3::new(6.4, y, 0.0),
            Vec3::new(3.1, y, 0.0),
            0.9,
            0.8,
            GANTRY_RED,
        );
    }
    // Red warning beacons at the top of the rocket and gantry tower.
    add_box(
        &mut beacon,
        Vec3::new(-0.55, 56.0, -0.55),
        Vec3::new(0.55, 57.4, 0.55),
        Color::WHITE,
    );
    add_box(
        &mut beacon,
        Vec3::new(7.9, 51.0, -0.55),
        Vec3::new(9.0, 52.4, 0.55),
        Color::WHITE,
    );
    (hull.build(), beacon.build())
}

/// Spawns overhead orbital space-station habitat modules spanning across the
/// road, sharing one hull mesh and one light-strip/window mesh.
fn spawn_station_arches(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    arches: &[StationArch],
    hull: &Handle<StandardMaterial>,
    glow: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if arches.is_empty() {
        return;
    }
    let (frame, lights) = station_arch_meshes();
    let frame_mesh = assets.meshes.add(frame);
    let lights_mesh = assets.meshes.add(lights);
    for arch in arches {
        let index = track.index_at(arch.along);
        let (center, heading) = track.pose_at(index);
        let road_y = track.surface_point(index, 0.0).y;
        let left = track.offset_point(index, ARCH_SPAN);
        let right = track.offset_point(index, -ARCH_SPAN);
        let ground_y = terrain.height(left).min(terrain.height(right)).min(road_y);
        let transform = Transform::from_xyz(center.x, road_y.max(ground_y), center.y)
            .with_rotation(Quat::from_rotation_y(heading));
        world.spawn((
            Mesh3d(frame_mesh.clone()),
            MeshMaterial3d(hull.clone()),
            transform,
        ));
        world.spawn((
            Mesh3d(lights_mesh.clone()),
            MeshMaterial3d(glow.clone()),
            transform,
        ));
    }
}

/// Builds an overhead orbital space-station bridge module spanning the road:
/// twin docking towers flanking the road walls, a pressurized cylindrical
/// laboratory/habitat module overhead (`y = 9.2..15.2 m`) with a central
/// observation cupola and glowing viewports, radiating photovoltaic solar array
/// wings on both sides, and tall communication antenna spires.
fn station_arch_meshes() -> (Mesh, Mesh) {
    let mut hull = MeshBuilder::default();
    let mut glow = MeshBuilder::default();
    let span = ARCH_SPAN;
    let module_y = ARCH_HEIGHT + 2.6;

    // 1. Twin lateral space-station support & airlock towers flanking the road.
    for side in [-1.0, 1.0] {
        let x_base = side * (span + 2.2);
        let x_top = side * (span - 0.4);
        // Heavy foundation and slanted primary truss leg.
        add_box(
            &mut hull,
            Vec3::new(x_base - 2.4, -18.0, -3.2),
            Vec3::new(x_base + 2.4, 2.6, 3.2),
            HULL_DARK,
        );
        add_box(
            &mut hull,
            Vec3::new(x_base - 2.6, 2.6, -3.4),
            Vec3::new(x_base + 2.6, 3.8, 3.4),
            GOLD_FOIL,
        );
        hull.beam(
            Vec3::new(x_base, 3.4, -1.8),
            Vec3::new(x_top, module_y, -1.8),
            1.6,
            1.6,
            HULL_WHITE,
        );
        hull.beam(
            Vec3::new(x_base, 3.4, 1.8),
            Vec3::new(x_top, module_y, 1.8),
            1.6,
            1.6,
            HULL_WHITE,
        );
        // Vertical glowing cyan status strips facing the road and oncoming riders.
        glow.beam(
            Vec3::new(x_base - side * 0.9, 4.0, -2.0),
            Vec3::new(x_top - side * 0.9, module_y - 1.2, -2.0),
            0.35,
            0.35,
            Color::WHITE,
        );
        glow.beam(
            Vec3::new(x_base - side * 0.9, 4.0, 2.0),
            Vec3::new(x_top - side * 0.9, module_y - 1.2, 2.0),
            0.35,
            0.35,
            Color::WHITE,
        );

        // 2. Huge radiating photovoltaic solar array wings extending outward
        // from each side of the overhead space station module.
        let wing_in = side * (span + 1.0);
        let wing_out = side * (span + 15.5);
        hull.beam(
            Vec3::new(wing_in, module_y + 0.6, 0.0),
            Vec3::new(wing_out, module_y + 2.4, 0.0),
            0.65,
            0.65,
            GOLD_FOIL,
        );
        for z_off in [-2.8, 2.8] {
            hull.beam(
                Vec3::new(wing_in, module_y + 0.6, z_off),
                Vec3::new(wing_out, module_y + 2.4, z_off),
                0.28,
                2.2,
                SOLAR_BLUE,
            );
            hull.beam(
                Vec3::new(wing_in, module_y + 0.6, z_off),
                Vec3::new(wing_out, module_y + 2.4, z_off),
                0.34,
                0.28,
                SOLAR_GRID,
            );
        }
    }

    // 3. Overhead pressurized cylindrical habitat module spanning across the road.
    let cyl_half_x = span + 0.6;
    let cyl_r = 2.65f32;
    let sides = 10usize;
    for i in 0..sides {
        let a0 = i as f32 / sides as f32 * TAU;
        let a1 = (i + 1) as f32 / sides as f32 * TAU;
        let (y0, z0) = (module_y + a0.sin() * cyl_r, a0.cos() * cyl_r);
        let (y1, z1) = (module_y + a1.sin() * cyl_r, a1.cos() * cyl_r);
        let normal = Vec3::new(0.0, (a0 + a1).sin(), (a0 + a1).cos()).normalize_or_zero();
        let color = if i % 2 == 0 { HULL_WHITE } else { HULL_SILVER };
        hull.quad(
            [
                (Vec3::new(-cyl_half_x, y0, z0), Vec2::ZERO),
                (Vec3::new(cyl_half_x, y0, z0), Vec2::X),
                (Vec3::new(cyl_half_x, y1, z1), Vec2::ONE),
                (Vec3::new(-cyl_half_x, y1, z1), Vec2::Y),
            ],
            normal,
            color,
        );
    }
    // Gold docking rings along the overhead habitat cylinder.
    for x_ring in [-8.5, -4.2, 4.2, 8.5] {
        add_box(
            &mut hull,
            Vec3::new(x_ring - 0.45, module_y - 2.85, -2.85),
            Vec3::new(x_ring + 0.45, module_y + 2.85, 2.85),
            GOLD_FOIL,
        );
    }
    // Central Space-Station Command Cupola Node over the middle of the road
    // with panoramic glowing cyan windows facing both ways along the track.
    add_box(
        &mut hull,
        Vec3::new(-3.6, module_y - 3.1, -3.5),
        Vec3::new(3.6, module_y + 3.8, 3.5),
        HULL_DARK,
    );
    add_box(
        &mut glow,
        Vec3::new(-3.1, module_y - 1.8, -3.65),
        Vec3::new(3.1, module_y + 1.8, 3.65),
        Color::WHITE,
    );
    // Glowing viewport windows along the front and back of the habitat cylinder,
    // plus a full-width cyan light guide underneath over the asphalt.
    for x_win in [-6.4, 6.4] {
        add_box(
            &mut glow,
            Vec3::new(x_win - 1.5, module_y - 0.85, -2.78),
            Vec3::new(x_win + 1.5, module_y + 0.85, 2.78),
            Color::WHITE,
        );
    }
    glow.beam(
        Vec3::new(-cyl_half_x * 0.90, module_y - cyl_r - 0.15, 0.0),
        Vec3::new(cyl_half_x * 0.90, module_y - cyl_r - 0.15, 0.0),
        0.75,
        0.25,
        Color::WHITE,
    );
    // Twin orbital communication antenna masts atop the overhead station module.
    for x_ant in [-2.4, 2.4] {
        hull.beam(
            Vec3::new(x_ant, module_y + 3.8, 0.0),
            Vec3::new(x_ant, module_y + 9.8, 0.0),
            0.28,
            0.28,
            HULL_SILVER,
        );
        add_box(
            &mut glow,
            Vec3::new(x_ant - 0.45, module_y + 9.6, -0.45),
            Vec3::new(x_ant + 0.45, module_y + 10.4, 0.45),
            Color::WHITE,
        );
    }
    (hull.build(), glow.build())
}

/// Spawns rows of photovoltaic solar-panel wings beside the road, all sharing
/// a single mesh handle.
fn spawn_solar_arrays(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    rows: &[Row],
    hull: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    if rows.is_empty() {
        return Vec::new();
    }
    let wing_mesh = assets.meshes.add(solar_wing_mesh());
    let mut occupied = Vec::new();
    for row in rows {
        for (index, point) in row_points(track, row) {
            let base = terrain.height(point);
            let road_y = track.surface_point(index, row.side.sign() * row.lateral).y;
            if (base - road_y).abs() > 4.0 || base < SEA_LEVEL + 0.5 {
                continue;
            }
            let (_, heading) = track.pose_at(index);
            let facing = if row.side.sign() > 0.0 {
                heading
            } else {
                heading + PI
            };
            world.spawn((
                Mesh3d(wing_mesh.clone()),
                MeshMaterial3d(hull.clone()),
                Transform::from_xyz(point.x, base, point.y)
                    .with_rotation(Quat::from_rotation_y(facing)),
            ));
            occupied.push((point, 4.5));
        }
    }
    occupied
}

fn solar_wing_mesh() -> Mesh {
    let mut mesh = MeshBuilder::default();
    // Central tracking mast and gold junction box.
    mesh.beam(
        Vec3::new(0.0, -2.0, 0.0),
        Vec3::new(0.0, 4.2, 0.0),
        0.42,
        0.42,
        HULL_SILVER,
    );
    add_box(
        &mut mesh,
        Vec3::new(-0.55, 1.2, -0.55),
        Vec3::new(0.55, 2.1, 0.55),
        GOLD_FOIL,
    );
    // Two tilted photovoltaic wings extending along Z either side of the mast.
    let tilt = Vec3::new(0.45, 0.89, 0.0).normalize();
    let across = Vec3::new(0.89, -0.45, 0.0).normalize();
    for z_sign in [-1.0, 1.0] {
        let z0 = z_sign * 0.5;
        let z1 = z_sign * 5.2;
        let center = Vec3::new(0.0, 4.2, (z0 + z1) / 2.0);
        let half_w = across * 1.65;
        let c00 = Vec3::new(center.x, center.y, z0) - half_w;
        let c10 = Vec3::new(center.x, center.y, z0) + half_w;
        let c11 = Vec3::new(center.x, center.y, z1) + half_w;
        let c01 = Vec3::new(center.x, center.y, z1) - half_w;
        mesh.quad(
            [
                (c00, Vec2::ZERO),
                (c10, Vec2::X),
                (c11, Vec2::ONE),
                (c01, Vec2::Y),
            ],
            tilt,
            SOLAR_BLUE,
        );
        mesh.quad(
            [
                (c00 - tilt * 0.12, Vec2::ZERO),
                (c01 - tilt * 0.12, Vec2::Y),
                (c11 - tilt * 0.12, Vec2::ONE),
                (c10 - tilt * 0.12, Vec2::X),
            ],
            -tilt,
            HULL_SILVER,
        );
        mesh.beam(
            Vec3::new(0.0, 4.1, z0),
            Vec3::new(0.0, 4.1, z1),
            0.18,
            0.18,
            SOLAR_GRID,
        );
    }
    mesh.build()
}

/// Spawns angular lunar basalt boulders near the circuit, sharing 3 faceted
/// mesh handles across all instances.
fn spawn_lunar_boulders(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    boulders: &LunarBoulders,
    occupied: &[Footprint],
    hull: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    if boulders.max == 0 {
        return Vec::new();
    }
    let shapes: Vec<Handle<Mesh>> = (0..BOULDER_SHAPES)
        .map(|shape| assets.meshes.add(boulder_mesh(shape)))
        .collect();
    let placed = place_boulders(track, terrain, boulders, occupied);
    let mut footprints = Vec::with_capacity(placed.len());
    for (index, &(spot, y, yaw, scale, shape)) in placed.iter().enumerate() {
        let pitch = (random01(index as u32 * 5 + 90_001) - 0.5) * 0.35;
        let roll = (random01(index as u32 * 5 + 90_002) - 0.5) * 0.35;
        world.spawn((
            Mesh3d(shapes[shape as usize % shapes.len()].clone()),
            MeshMaterial3d(hull.clone()),
            Transform::from_xyz(spot.x, y, spot.y)
                .with_rotation(
                    Quat::from_rotation_y(yaw)
                        * Quat::from_rotation_x(pitch)
                        * Quat::from_rotation_z(roll),
                )
                .with_scale(Vec3::splat(scale)),
        ));
        footprints.push((spot, scale * 2.5));
    }
    footprints
}

fn place_boulders(
    track: &Track,
    terrain: &Terrain,
    boulders: &LunarBoulders,
    occupied: &[Footprint],
) -> Vec<(Vec2, f32, f32, f32, u32)> {
    let mut placed: Vec<(Vec2, f32, f32, f32, u32)> = Vec::new();
    let lap = track.lap_length();
    for seed in 0..2500u32 {
        if placed.len() >= boulders.max {
            break;
        }
        let r = |k: u32| random01(seed * 7 + k + 77_000);
        let position = if boulders.spans.is_empty() {
            let along = r(0) * lap;
            let side = if r(1) < 0.5 { 1.0 } else { -1.0 };
            let lateral = side * BOULDER_NEAR.lerp(BOULDER_FAR, r(2));
            track.offset_point(track.index_at(along), lateral)
        } else {
            let span = boulders.spans[seed as usize % boulders.spans.len()];
            let along = span.from + span.length(lap) * r(0);
            let side = if r(1) < 0.5 { 1.0 } else { -1.0 };
            let lateral = side * BOULDER_NEAR.lerp(BOULDER_FAR, r(2));
            track.offset_point(track.index_at(along), lateral)
        };
        if is_earth_realm(position) {
            continue;
        }
        let nearest = track.nearest(position);
        if nearest.distance < BOULDER_NEAR
            || track.on_other_road(nearest.road, position, |_, _| BOULDER_NEAR)
        {
            continue;
        }
        let height = terrain.height(position);
        if height < SEA_LEVEL + 1.0 || terrain.normal(position).y < 0.72 {
            continue;
        }
        let scale = 0.85 + 1.65 * r(3);
        if occupied
            .iter()
            .any(|&(at, radius)| at.distance(position) < radius + scale * 2.5)
            || placed.iter().any(|&(at, _, _, other_scale, _)| {
                at.distance(position) < (scale + other_scale) * 3.5
            })
        {
            continue;
        }
        let yaw = r(4) * TAU;
        let shape = (seed + (r(5) * 10.0) as u32) % BOULDER_SHAPES;
        placed.push((position, height - 0.35 * scale, yaw, scale, shape));
    }
    placed
}

/// An angular faceted lunar basalt boulder mesh of variant `shape`.
fn boulder_mesh(shape: u32) -> Mesh {
    let mut mesh = MeshBuilder::default();
    let sides = 7usize;
    let rings = [
        (-0.8, 1.85, BASALT_DARK),
        (0.4, 2.15, BASALT_DARK),
        (1.65, 1.55, BASALT_LIGHT),
        (2.45, 0.65, BASALT_LIGHT),
    ];
    let ring_point = |ring: usize, i: usize| -> Vec3 {
        let (y, r, _) = rings[ring];
        let k = (ring * 13 + i * 7 + shape as usize * 19) as u32;
        let angle = i as f32 / sides as f32 * TAU + 0.18 * (random01(k) - 0.5);
        let rad = r * (0.78 + 0.42 * random01(k + 3));
        let dy = 0.22 * (random01(k + 5) - 0.5);
        Vec3::new(angle.cos() * rad, y + dy, angle.sin() * rad)
    };
    let apex = Vec3::new(
        0.2 * (random01(shape * 11 + 1) - 0.5),
        2.75 + 0.35 * random01(shape * 11 + 2),
        0.2 * (random01(shape * 11 + 3) - 0.5),
    );
    for i in 0..sides {
        let next = (i + 1) % sides;
        for r in 0..rings.len() - 1 {
            let c00 = ring_point(r, i);
            let c10 = ring_point(r, next);
            let c11 = ring_point(r + 1, next);
            let c01 = ring_point(r + 1, i);
            let outward = ((c00 + c10 + c11 + c01) / 4.0 - Vec3::Y * 0.8).normalize_or_zero();
            mesh.quad(
                [
                    (c00, Vec2::ZERO),
                    (c10, Vec2::X),
                    (c11, Vec2::ONE),
                    (c01, Vec2::Y),
                ],
                outward,
                rings[r + 1].2,
            );
        }
        let top0 = ring_point(rings.len() - 1, i);
        let top1 = ring_point(rings.len() - 1, next);
        let outward = ((top0 + top1 + apex) / 3.0 - Vec3::Y).normalize_or_zero();
        mesh.triangle(
            [(top0, Vec2::ZERO), (top1, Vec2::X), (apex, Vec2::Y)],
            outward,
            BASALT_LIGHT,
        );
    }
    mesh.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biodome_rover_dish_rocket_and_boulder_meshes_have_geometry() {
        let (dome_hull, dome_glow) = biodome_meshes(16.0);
        assert!(dome_hull.count_vertices() > 40);
        assert!(dome_glow.count_vertices() > 0);

        let (rover_hull, rover_glow) = rover_meshes();
        assert!(rover_hull.count_vertices() > 40);
        assert!(rover_glow.count_vertices() == 8);

        let (dish_hull, dish_tip) = dish_reflector_meshes();
        assert!(dish_hull.count_vertices() > 40);
        assert!(dish_tip.count_vertices() > 0);

        let (obs_hull, obs_glow) = observatory_meshes();
        assert!(obs_hull.count_vertices() > 80);
        assert!(obs_glow.count_vertices() > 10);

        let (rocket_hull, rocket_beacon) = rocket_meshes();
        assert!(rocket_hull.count_vertices() > 80);
        assert!(rocket_beacon.count_vertices() > 0);

        for shape in 0..BOULDER_SHAPES {
            assert!(boulder_mesh(shape).count_vertices() > 20);
        }
    }
}
