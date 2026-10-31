//! The works of a mine around the road ([`Mine`]): rail lines beside it with
//! trains of ore carts rolling up and down them, timber trestles carrying a
//! rail across the road from one rock wall to the other with a cart going
//! over and back, headframes over the shafts with their winding wheels
//! turning, lanterns on posts, and piles of crates, barrels and ore.
//!
//! Every rail line, trestle and headframe is one mesh coloured vertex by
//! vertex, so each costs a single draw; the carts share theirs.

use std::f32::consts::{FRAC_PI_2, TAU};
use std::sync::Arc;

use bevy::prelude::*;

use super::Assets3d;
use super::landmarks::Spin;
use super::props::{Footprint, row_points};
use super::quads::MeshBuilder;
use super::scenery::{Headframe, Mine, Pile, PileKind, Rails, Scenery, Trestle};
use super::terrain::Terrain;
use crate::geometry::{forward, random01};
use crate::track::{Track, WALL_DISTANCE};

/// Half the gauge of the rails (m), and how far the sleepers reach out and
/// how far apart they are (m).
const GAUGE: f32 = 0.45;
const SLEEPER_REACH: f32 = 0.85;
const SLEEPER_EVERY: f32 = 0.8;
/// How high the trestles carry their rail above the road (m), and how far
/// they reach to either side of its centre line, into the rock (m).
const TRESTLE_HEIGHT: f32 = 7.0;
const TRESTLE_REACH: f32 = WALL_DISTANCE + 13.0;
/// The cart on a trestle goes this far into the rock on either side (m).
const TRESTLE_RUN: f32 = WALL_DISTANCE + 10.0;
/// How fast the carts go at most (m/s), and how far apart the carts of a
/// train are (m).
const CART_SPEED: f32 = 4.0;
const CART_SPACING: f32 = 2.2;
/// Height of a headframe up to its wheels (m), and their radius (m).
const HEADFRAME_HEIGHT: f32 = 17.0;
const SHEAVE_RADIUS: f32 = 1.8;
/// How fast the winding wheels turn (rad/s).
const SHEAVE_SPIN: f32 = 0.9;
/// Height of a lantern post (m).
const LANTERN_HEIGHT: f32 = 3.4;

/// The colours of the works (sRGB).
const WOOD: Color = Color::srgb(0.46, 0.32, 0.2);
const DARK_WOOD: Color = Color::srgb(0.3, 0.2, 0.13);
const STEEL: Color = Color::srgb(0.42, 0.42, 0.44);
const GRAVEL: Color = Color::srgb(0.36, 0.3, 0.27);
const RUSTY_IRON: Color = Color::srgb(0.45, 0.24, 0.15);
const IRON: Color = Color::srgb(0.17, 0.15, 0.14);
const ORE: Color = Color::srgb(0.34, 0.27, 0.22);
const SHEAVE_RED: Color = Color::srgb(0.62, 0.12, 0.08);
const TNT_RED: Color = Color::srgb(0.75, 0.1, 0.07);

/// A line of rails in the world: points along it, and how far along it each
/// one is (m).
#[derive(Debug)]
pub struct RailPath {
    points: Vec<Vec3>,
    distances: Vec<f32>,
}

impl RailPath {
    fn new(points: Vec<Vec3>) -> Self {
        let mut distances = Vec::with_capacity(points.len());
        let mut total = 0.0;
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                total += point.distance(points[i - 1]);
            }
            distances.push(total);
        }
        Self { points, distances }
    }

    fn length(&self) -> f32 {
        self.distances.last().copied().unwrap_or(0.0)
    }

    /// The point `distance` metres along the line, and which way it goes
    /// there.
    fn at(&self, distance: f32) -> (Vec3, Vec3) {
        let distance = distance.clamp(0.0, self.length());
        let segment = self
            .distances
            .partition_point(|&d| d <= distance)
            .clamp(1, self.points.len() - 1);
        let (a, b) = (self.points[segment - 1], self.points[segment]);
        let span = self.distances[segment] - self.distances[segment - 1];
        let t = if span > 0.0 {
            (distance - self.distances[segment - 1]) / span
        } else {
            0.0
        };
        (a.lerp(b, t), (b - a).normalize_or(Vec3::Z))
    }
}

/// An ore cart: the line it rolls on, the place in its train, and when its
/// train sets off (s), so the trains do not all move together.
#[derive(Component)]
pub struct Cart {
    path: Arc<RailPath>,
    place: usize,
    phase: f32,
}

/// Where along its line a train is at `time` (m): it goes up and down the
/// line, slowing down at the ends and setting off again, at most at
/// [`CART_SPEED`].
fn train_distance(length: f32, time: f32) -> f32 {
    if length <= 0.0 {
        return 0.0;
    }
    let rate = 2.0 * CART_SPEED / length;
    length * 0.5 * (1.0 - (rate * time).cos())
}

/// Rolls the carts along their rails.
pub fn roll_carts(time: Res<Time>, mut carts: Query<(&Cart, &mut Transform)>) {
    for (cart, mut transform) in &mut carts {
        let train = cart.path.length() - CART_SPACING * cart.place as f32;
        let along = train_distance(train, time.elapsed_secs() + cart.phase)
            + CART_SPACING * cart.place as f32;
        let (position, direction) = cart.path.at(along);
        transform.translation = position;
        transform.rotation = Transform::default()
            .looking_to(Vec3::new(direction.x, 0.0, direction.z), Vec3::Y)
            .rotation
            * Quat::from_rotation_x(direction.y.clamp(-0.5, 0.5).asin());
    }
}

/// Builds the works of the mine of `scenery`, if it has one, and returns the
/// ground they take.
pub fn spawn_mine(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let Some(mine) = &scenery.mine else {
        return Vec::new();
    };
    let painted = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.85,
        ..default()
    });
    let gold = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.78, 0.22),
        emissive: LinearRgba::rgb(0.5, 0.36, 0.05),
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });
    let cart = Cart3d {
        body: assets.meshes.add(cart_mesh()),
        nuggets: assets.meshes.add(nugget_mesh()),
        painted: painted.clone(),
        gold: gold.clone(),
    };
    let mut seed = 0;
    for rails in mine.rails {
        let path = rail_path(track, terrain, rails);
        let mut builder = MeshBuilder::default();
        lay_rails(&mut builder, &path, true);
        world.spawn((
            Mesh3d(assets.meshes.add(builder.build())),
            MeshMaterial3d(painted.clone()),
        ));
        spawn_train(world, &cart, path, rails.carts, &mut seed);
    }
    for trestle in mine.trestles {
        let (mesh, path) = trestle_mesh(track, terrain, trestle);
        world.spawn((
            Mesh3d(assets.meshes.add(mesh)),
            MeshMaterial3d(painted.clone()),
        ));
        spawn_train(world, &cart, path, 1, &mut seed);
    }
    let mut taken = Vec::new();
    for headframe in mine.headframes {
        taken.push(spawn_headframe(
            world, track, terrain, headframe, &painted, assets,
        ));
    }
    spawn_lanterns(world, track, terrain, mine, &painted, assets);
    for pile in mine.piles {
        taken.push(spawn_pile(
            world, track, terrain, pile, &painted, &gold, assets,
        ));
    }
    taken
}

/// The shared meshes and materials of the ore carts.
struct Cart3d {
    body: Handle<Mesh>,
    nuggets: Handle<Mesh>,
    painted: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
}

/// A train of `count` carts on `path`.
fn spawn_train(
    world: &mut ChildSpawnerCommands,
    cart: &Cart3d,
    path: RailPath,
    count: u32,
    seed: &mut u32,
) {
    let path = Arc::new(path);
    *seed += 1;
    let phase = random01(*seed * 31) * 60.0;
    for place in 0..count as usize {
        world
            .spawn((
                Cart {
                    path: path.clone(),
                    place,
                    phase,
                },
                Transform::from_translation(path.at(0.0).0),
                Visibility::default(),
                Mesh3d(cart.body.clone()),
                MeshMaterial3d(cart.painted.clone()),
            ))
            .with_child((
                Mesh3d(cart.nuggets.clone()),
                MeshMaterial3d(cart.gold.clone()),
            ));
    }
}

/// The line of a run of rails: beside the road, on the ground, smoothed so
/// the carts do not jolt.
fn rail_path(track: &Track, terrain: &Terrain, rails: &Rails) -> RailPath {
    let lap = track.lap_length();
    let count = (rails.span.length(lap) / track.step_length()) as usize;
    let first = track.index_at(rails.span.from);
    let ground: Vec<Vec3> = (0..=count)
        .map(|k| {
            let index = (first + k) % track.sample_count();
            let point = track.offset_point(index, rails.side.sign() * rails.lateral);
            Vec3::new(point.x, terrain.height(point), point.y)
        })
        .collect();
    // Averaged over a few points either way, and never into the ground.
    let points = (0..ground.len())
        .map(|i| {
            let around = &ground[i.saturating_sub(3)..(i + 4).min(ground.len())];
            let height = around.iter().map(|p| p.y).sum::<f32>() / around.len() as f32;
            Vec3::new(ground[i].x, height.max(ground[i].y) + 0.12, ground[i].z)
        })
        .collect();
    RailPath::new(points)
}

/// Lays rails along `path`: sleepers across it, two steel rails on them and,
/// if `ballast`, a bed of gravel under them.
fn lay_rails(builder: &mut MeshBuilder, path: &RailPath, ballast: bool) {
    let across = |direction: Vec3| Vec3::new(-direction.z, 0.0, direction.x).normalize_or(Vec3::X);
    if ballast {
        for pair in path.points.windows(2) {
            let side = across(pair[1] - pair[0]) * (SLEEPER_REACH + 0.35);
            let drop = Vec3::Y * 0.05;
            builder.quad(
                [
                    (pair[0] - side - drop, Vec2::ZERO),
                    (pair[1] - side - drop, Vec2::X),
                    (pair[1] + side - drop, Vec2::ONE),
                    (pair[0] + side - drop, Vec2::Y),
                ],
                Vec3::Y,
                GRAVEL,
            );
        }
    }
    let mut distance = SLEEPER_EVERY / 2.0;
    while distance < path.length() {
        let (at, direction) = path.at(distance);
        let side = across(direction) * SLEEPER_REACH;
        builder.beam(at - side, at + side, 0.22, 0.12, DARK_WOOD);
        distance += SLEEPER_EVERY;
    }
    for pair in path.points.windows(2) {
        let side = across(pair[1] - pair[0]) * GAUGE;
        for sign in [-1.0, 1.0] {
            let lift = Vec3::Y * 0.12;
            builder.beam(
                pair[0] + side * sign + lift,
                pair[1] + side * sign + lift,
                0.07,
                0.1,
                STEEL,
            );
        }
    }
}

/// Adds a cylinder round the axis from `a` to `b`, of `radius`, with `sides`
/// flat sides and its ends closed.
fn cylinder(builder: &mut MeshBuilder, a: Vec3, b: Vec3, radius: f32, sides: usize, color: Color) {
    let axis = (b - a).normalize_or(Vec3::Y);
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    let ring = |k: usize| {
        let angle = k as f32 / sides as f32 * TAU;
        (u * angle.cos() + v * angle.sin()) * radius
    };
    for k in 0..sides {
        let (p, q) = (ring(k), ring(k + 1));
        let out = (p + q).normalize_or(u);
        builder.quad(
            [
                (a + p, Vec2::ZERO),
                (b + p, Vec2::X),
                (b + q, Vec2::ONE),
                (a + q, Vec2::Y),
            ],
            out,
            color,
        );
        for (end, facing) in [(a, -axis), (b, axis)] {
            builder.triangle(
                [(end, Vec2::ZERO), (end + p, Vec2::X), (end + q, Vec2::Y)],
                facing,
                color,
            );
        }
    }
}

/// An ore cart, on its origin on the rails (X across, Y up, Z along): an
/// iron bin wider at the top, with a dark rim and straps, on a frame with
/// four wheels, heaped with ore.
fn cart_mesh() -> Mesh {
    let mut cart = MeshBuilder::default();
    for x in [-GAUGE, GAUGE] {
        for z in [-0.55, 0.55] {
            cylinder(
                &mut cart,
                Vec3::new(x - 0.04, 0.24, z),
                Vec3::new(x + 0.04, 0.24, z),
                0.22,
                10,
                IRON,
            );
        }
    }
    for x in [-0.35, 0.35] {
        cart.beam(
            Vec3::new(x, 0.38, -0.8),
            Vec3::new(x, 0.38, 0.8),
            0.12,
            0.12,
            IRON,
        );
    }
    // The bin: wider at the top than at the bottom.
    let (bottom, top) = (0.45, 1.25);
    let low = Vec2::new(0.5, 0.7);
    let high = Vec2::new(0.66, 0.92);
    let corner = |sx: f32, sz: f32, y: f32, half: Vec2| Vec3::new(sx * half.x, y, sz * half.y);
    for (sx, sz, tx, tz, out) in [
        (-1.0, -1.0, 1.0, -1.0, Vec3::NEG_Z),
        (1.0, -1.0, 1.0, 1.0, Vec3::X),
        (1.0, 1.0, -1.0, 1.0, Vec3::Z),
        (-1.0, 1.0, -1.0, -1.0, Vec3::NEG_X),
    ] {
        cart.quad(
            [
                (corner(sx, sz, bottom, low), Vec2::ZERO),
                (corner(tx, tz, bottom, low), Vec2::X),
                (corner(tx, tz, top, high), Vec2::ONE),
                (corner(sx, sz, top, high), Vec2::Y),
            ],
            out + Vec3::Y * 0.2,
            RUSTY_IRON,
        );
        // The rim round the top, and a strap down each corner.
        cart.beam(
            corner(sx, sz, top, high),
            corner(tx, tz, top, high),
            0.09,
            0.09,
            IRON,
        );
        cart.beam(
            corner(sx, sz, bottom, low),
            corner(sx, sz, top, high),
            0.07,
            0.07,
            IRON,
        );
    }
    cart.quad(
        [
            (corner(-1.0, -1.0, bottom, low), Vec2::ZERO),
            (corner(1.0, -1.0, bottom, low), Vec2::X),
            (corner(1.0, 1.0, bottom, low), Vec2::ONE),
            (corner(-1.0, 1.0, bottom, low), Vec2::Y),
        ],
        Vec3::NEG_Y,
        RUSTY_IRON,
    );
    // The ore, heaped in lumps over the rim.
    for k in 0..14u32 {
        let r = |n: u32| random01(k * 7 + n);
        let x = (r(0) - 0.5) * 1.0;
        let z = (r(1) - 0.5) * 1.4;
        let hump = 1.0 - (x * x / 0.3 + z * z / 0.5).min(1.0);
        let y = top - 0.12 + 0.25 * hump;
        let size = 0.22 + 0.16 * r(2);
        let tilt = Vec3::new(r(3) - 0.5, 0.0, r(4) - 0.5) * size;
        cart.beam(
            Vec3::new(x, y, z) - tilt,
            Vec3::new(x, y, z) + tilt + Vec3::Y * 0.01,
            size,
            size * 0.8,
            ORE,
        );
    }
    cart.build()
}

/// The gold glinting in a cart's ore.
fn nugget_mesh() -> Mesh {
    let mut gold = MeshBuilder::default();
    for k in 0..7u32 {
        let r = |n: u32| random01(1000 + k * 7 + n);
        let x = (r(0) - 0.5) * 0.8;
        let z = (r(1) - 0.5) * 1.1;
        let hump = 1.0 - (x * x / 0.3 + z * z / 0.5).min(1.0);
        let y = 1.25 + 0.22 * hump;
        let size = 0.09 + 0.07 * r(2);
        let at = Vec3::new(x, y, z);
        gold.beam(
            at - Vec3::new(size, 0.0, size * 0.4),
            at + Vec3::new(size, 0.02, size * 0.4),
            size,
            size,
            Color::WHITE,
        );
    }
    gold.build()
}

/// The point of the road `along` a lap: its index, its centre, which way it
/// goes and which way is left.
fn road_frame(track: &Track, along: f32) -> (usize, Vec3, Vec3, Vec3) {
    let index = track.index_at(along);
    let center = track.surface_point(index, 0.0);
    let (_, heading) = track.pose_at(index);
    let ahead = forward(heading);
    let left = -ahead.perp();
    (
        index,
        center,
        Vec3::new(ahead.x, 0.0, ahead.y),
        Vec3::new(left.x, 0.0, left.y),
    )
}

/// A timber trestle across the road: a truss over it on a bent of posts
/// just beyond each wall, carrying rails from one rock wall into the
/// other. Returns its mesh and the line its cart rolls on.
fn trestle_mesh(track: &Track, terrain: &Terrain, trestle: &Trestle) -> (Mesh, RailPath) {
    let (_, center, ahead, left) = road_frame(track, trestle.along);
    let deck = center.y + TRESTLE_HEIGHT;
    let at = |lateral: f32, along: f32, y: f32| {
        Vec3::new(center.x, y, center.z) + left * lateral + ahead * along
    };
    let ground = |p: Vec3| terrain.height(Vec2::new(p.x, p.z));
    let mut mesh = MeshBuilder::default();
    // The stringers, under the sleepers, from rock to rock.
    for side in [-0.7, 0.7] {
        mesh.beam(
            at(-TRESTLE_REACH, side, deck - 0.25),
            at(TRESTLE_REACH, side, deck - 0.25),
            0.3,
            0.4,
            WOOD,
        );
    }
    // A truss on each side over the road: top chord, posts and diagonals.
    let span = WALL_DISTANCE + 2.0;
    const BAYS: usize = 10;
    for side in [-1.1, 1.1] {
        mesh.beam(
            at(-span, side, deck + 1.4),
            at(span, side, deck + 1.4),
            0.2,
            0.2,
            WOOD,
        );
        for bay in 0..=BAYS {
            let lateral = -span + 2.0 * span * bay as f32 / BAYS as f32;
            mesh.beam(
                at(lateral, side, deck - 0.4),
                at(lateral, side, deck + 1.4),
                0.18,
                0.18,
                WOOD,
            );
            if bay < BAYS {
                let next = lateral + 2.0 * span / BAYS as f32;
                let (from, to) = if bay < BAYS / 2 {
                    (at(lateral, side, deck + 1.4), at(next, side, deck - 0.4))
                } else {
                    (at(lateral, side, deck - 0.4), at(next, side, deck + 1.4))
                };
                mesh.beam(from, to, 0.14, 0.14, DARK_WOOD);
            }
        }
    }
    // The bents beyond the walls, down to the ground, braced across.
    for lateral in [-(WALL_DISTANCE + 1.2), WALL_DISTANCE + 1.2] {
        let feet: Vec<Vec3> = [-0.9, 0.9]
            .into_iter()
            .map(|side| {
                let top = at(lateral, side, deck - 0.4);
                Vec3::new(top.x, ground(top) - 0.3, top.z)
            })
            .collect();
        for (side, foot) in [-0.9, 0.9].into_iter().zip(&feet) {
            mesh.beam(*foot, at(lateral, side, deck - 0.3), 0.32, 0.32, WOOD);
        }
        mesh.beam(
            at(lateral, -1.0, deck - 0.5),
            at(lateral, 1.0, deck - 0.5),
            0.25,
            0.3,
            WOOD,
        );
        let low = feet[0].y.max(feet[1].y) + 0.8;
        mesh.beam(
            Vec3::new(feet[0].x, low, feet[0].z),
            at(lateral, 0.9, deck - 0.6),
            0.16,
            0.16,
            DARK_WOOD,
        );
        mesh.beam(
            Vec3::new(feet[1].x, low, feet[1].z),
            at(lateral, -0.9, deck - 0.6),
            0.16,
            0.16,
            DARK_WOOD,
        );
    }
    let path = RailPath::new(vec![
        at(-TRESTLE_RUN, 0.0, deck),
        at(TRESTLE_RUN, 0.0, deck),
    ]);
    let rails = RailPath::new(vec![
        at(-TRESTLE_REACH, 0.0, deck),
        at(TRESTLE_REACH, 0.0, deck),
    ]);
    lay_rails(&mut mesh, &rails, false);
    (mesh.build(), path)
}

/// A headframe: a tapering timber tower over the shaft, with two struts
/// leaning back to the winding house beside it along the road, and the two
/// winding wheels on top, turning, seen side on from the road.
fn spawn_headframe(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    headframe: &Headframe,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (index, ..) = road_frame(track, headframe.along);
    let ground = track.offset_point(index, headframe.side.sign() * headframe.lateral);
    let base = Vec3::new(ground.x, terrain.height(ground), ground.y);
    // Local frame: X towards the road, Z along it.
    let toward_road = {
        let center = track.offset_point(index, 0.0) - ground;
        Vec3::new(center.x, 0.0, center.y).normalize_or(Vec3::X)
    };
    let rotation = Quat::from_mat3(&Mat3::from_cols(
        toward_road,
        Vec3::Y,
        toward_road.cross(Vec3::Y),
    ));
    let mut tower = MeshBuilder::default();
    let height = HEADFRAME_HEIGHT;
    let corner = |sx: f32, sz: f32, y: f32| {
        let half = 3.0 - 1.7 * y / height;
        Vec3::new(sx * half, y, sz * half)
    };
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    for &(sx, sz) in &corners {
        tower.beam(corner(sx, sz, -0.5), corner(sx, sz, height), 0.4, 0.4, WOOD);
    }
    for level in 1..=4 {
        let y = height * level as f32 / 4.0;
        let below = y - height / 4.0;
        for k in 0..4 {
            let (a, b) = (corners[k], corners[(k + 1) % 4]);
            tower.beam(corner(a.0, a.1, y), corner(b.0, b.1, y), 0.25, 0.25, WOOD);
            tower.beam(
                corner(a.0, a.1, below),
                corner(b.0, b.1, y),
                0.16,
                0.16,
                DARK_WOOD,
            );
        }
    }
    // The back struts, leaning down to the winding house, along the road.
    for sx in [-1.0, 1.0] {
        tower.beam(
            corner(sx, 1.0, height - 0.5),
            Vec3::new(sx * 1.8, -0.5, 11.0),
            0.35,
            0.35,
            WOOD,
        );
    }
    // The platform on top, and the winding house.
    tower.beam(
        Vec3::new(0.0, height, -1.6),
        Vec3::new(0.0, height, 1.6),
        3.2,
        0.3,
        DARK_WOOD,
    );
    tower.beam(
        Vec3::new(0.0, -0.5, 12.5),
        Vec3::new(0.0, -0.5, 18.5),
        5.0,
        9.5,
        Color::srgb(0.55, 0.4, 0.28),
    );
    tower.beam(
        Vec3::new(0.0, 4.35, 12.2),
        Vec3::new(0.0, 4.35, 18.8),
        5.6,
        0.5,
        Color::srgb(0.32, 0.26, 0.24),
    );
    // The cables, from the tops of the wheels to the winding house.
    for x in [-0.8, 0.8] {
        tower.beam(
            Vec3::new(x, height + 1.0 + 2.0 * SHEAVE_RADIUS, 0.0),
            Vec3::new(x, 3.5, 12.5),
            0.05,
            0.05,
            IRON,
        );
    }
    let mut wheel = MeshBuilder::default();
    const RIM: usize = 20;
    for k in 0..RIM {
        let angle = |k: usize| k as f32 / RIM as f32 * TAU;
        let point = |k: usize| Vec3::new(angle(k).cos(), 0.0, angle(k).sin()) * SHEAVE_RADIUS;
        wheel.beam(point(k), point(k + 1), 0.16, 0.22, SHEAVE_RED);
        if k % 4 == 0 {
            wheel.beam(Vec3::ZERO, point(k), 0.08, 0.08, SHEAVE_RED);
        }
    }
    cylinder(
        &mut wheel,
        Vec3::new(0.0, -0.2, 0.0),
        Vec3::new(0.0, 0.2, 0.0),
        0.25,
        8,
        IRON,
    );
    let wheel = assets.meshes.add(wheel.build());
    world
        .spawn((
            Transform::from_translation(base).with_rotation(rotation),
            Visibility::default(),
            Mesh3d(assets.meshes.add(tower.build())),
            MeshMaterial3d(painted.clone()),
        ))
        .with_children(|frame| {
            for x in [-0.8, 0.8] {
                // Turned so that the wheel's own up is across the tower: it
                // turns about it, in the plane along the road.
                frame
                    .spawn((
                        Transform::from_xyz(x, height + 1.0 + SHEAVE_RADIUS, 0.0)
                            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
                        Visibility::default(),
                    ))
                    .with_child((
                        Mesh3d(wheel.clone()),
                        MeshMaterial3d(painted.clone()),
                        Transform::default(),
                        Spin(if x < 0.0 { SHEAVE_SPIN } else { -SHEAVE_SPIN }),
                    ));
            }
        });
    (ground, 12.0)
}

/// Lanterns on wooden posts along the road, each hanging from an arm over
/// it, glowing orange.
fn spawn_lanterns(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    mine: &Mine,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if mine.lanterns.is_empty() {
        return;
    }
    // The post with its arm towards +X, and the lantern hanging from it.
    let mut post = MeshBuilder::default();
    post.beam(
        Vec3::new(0.0, -0.4, 0.0),
        Vec3::new(0.0, LANTERN_HEIGHT, 0.0),
        0.2,
        0.2,
        WOOD,
    );
    post.beam(
        Vec3::new(-0.1, LANTERN_HEIGHT - 0.15, 0.0),
        Vec3::new(1.0, LANTERN_HEIGHT - 0.15, 0.0),
        0.12,
        0.12,
        WOOD,
    );
    post.beam(
        Vec3::new(0.0, LANTERN_HEIGHT - 0.9, 0.0),
        Vec3::new(0.6, LANTERN_HEIGHT - 0.2, 0.0),
        0.08,
        0.08,
        DARK_WOOD,
    );
    post.beam(
        Vec3::new(0.9, LANTERN_HEIGHT - 0.22, 0.0),
        Vec3::new(0.9, LANTERN_HEIGHT - 0.55, 0.0),
        0.03,
        0.03,
        IRON,
    );
    post.beam(
        Vec3::new(0.9, LANTERN_HEIGHT - 0.58, 0.0),
        Vec3::new(0.9, LANTERN_HEIGHT - 0.52, 0.0),
        0.34,
        0.34,
        IRON,
    );
    let mut lamp = MeshBuilder::default();
    lamp.beam(
        Vec3::new(0.9, LANTERN_HEIGHT - 1.0, 0.0),
        Vec3::new(0.9, LANTERN_HEIGHT - 0.58, 0.0),
        0.26,
        0.26,
        Color::WHITE,
    );
    let post = assets.meshes.add(post.build());
    let lamp = assets.meshes.add(lamp.build());
    let glow = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.75, 0.4),
        emissive: LinearRgba::rgb(10.0, 5.0, 1.3),
        ..default()
    });
    for row in mine.lanterns {
        for (index, point) in row_points(track, row) {
            let toward = track.offset_point(index, 0.0) - point;
            let rotation = Quat::from_rotation_y((-toward.y).atan2(toward.x));
            world
                .spawn((
                    Transform::from_xyz(point.x, terrain.height(point), point.y)
                        .with_rotation(rotation),
                    Visibility::default(),
                    Mesh3d(post.clone()),
                    MeshMaterial3d(painted.clone()),
                ))
                .with_child((Mesh3d(lamp.clone()), MeshMaterial3d(glow.clone())));
        }
    }
}

/// A pile beside the road: crates, barrels and a red box of dynamite, or a
/// heap of ore with gold in it.
fn spawn_pile(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    pile: &Pile,
    painted: &Handle<StandardMaterial>,
    gold: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let index = track.index_at(pile.along);
    let ground = track.offset_point(index, pile.side.sign() * pile.lateral);
    let base = Vec3::new(ground.x, terrain.height(ground), ground.y);
    let seed = (pile.along * 10.0) as u32;
    let r = |n: u32| random01(seed * 13 + n);
    let mut mesh = MeshBuilder::default();
    let mut nuggets = MeshBuilder::default();
    match pile.kind {
        PileKind::Supplies => {
            let crate_wood = Color::srgb(0.62, 0.46, 0.3);
            for k in 0..5u32 {
                let size = 0.9 + 0.3 * r(k * 5);
                let at = Vec3::new((r(k * 5 + 1) - 0.5) * 4.0, 0.0, (r(k * 5 + 2) - 0.5) * 4.0);
                let turn = Quat::from_rotation_y(r(k * 5 + 3) * TAU);
                let half = turn * Vec3::new(size * 0.5, 0.0, 0.0);
                mesh.beam(at - half, at + half, size, size, crate_wood);
                // A plank across each face, darker.
                mesh.beam(
                    at - half * 1.02 + Vec3::Y * size * 0.5,
                    at + half * 1.02 + Vec3::Y * size * 0.5,
                    size * 1.04,
                    0.12,
                    DARK_WOOD,
                );
                if k == 0 {
                    // Another crate on top of the first.
                    let top = at + Vec3::Y * size * 0.5;
                    mesh.beam(
                        top - half * 0.8,
                        top + half * 0.8,
                        size * 0.8,
                        size * 0.8,
                        crate_wood,
                    );
                }
            }
            for k in 0..4u32 {
                let at = Vec3::new(2.6 + (r(40 + k) - 0.5) * 1.6, 0.0, (k as f32 - 1.5) * 0.85);
                cylinder(
                    &mut mesh,
                    at,
                    at + Vec3::Y * 1.1,
                    0.38,
                    10,
                    Color::srgb(0.5, 0.33, 0.2),
                );
                for y in [0.2, 0.9] {
                    cylinder(
                        &mut mesh,
                        at + Vec3::Y * y,
                        at + Vec3::Y * (y + 0.06),
                        0.395,
                        10,
                        IRON,
                    );
                }
            }
            // The dynamite: a red box, banded in black.
            let at = Vec3::new(-2.6, 0.0, 0.4);
            mesh.beam(at, at + Vec3::X * 1.1, 0.7, 0.6, TNT_RED);
            mesh.beam(
                at + Vec3::new(0.45, 0.0, 0.0),
                at + Vec3::new(0.65, 0.0, 0.0),
                0.72,
                0.62,
                IRON,
            );
        }
        PileKind::Ore => {
            // A heap of lumps, highest in the middle.
            for k in 0..40u32 {
                let angle = r(k * 3) * TAU;
                let out = r(k * 3 + 1).sqrt() * 3.2;
                let x = angle.cos() * out;
                let z = angle.sin() * out;
                let y = (3.2 - out) * 0.55;
                let size = 0.5 + 0.5 * r(k * 3 + 2);
                let tilt = Vec3::new(r(k + 200) - 0.5, 0.0, r(k + 300) - 0.5) * size;
                mesh.beam(
                    Vec3::new(x, y - size * 0.4, z) - tilt,
                    Vec3::new(x, y - size * 0.4, z) + tilt + Vec3::Y * 0.01,
                    size,
                    size * 0.8,
                    ORE,
                );
                if k % 3 == 0 {
                    let at = Vec3::new(x * 1.05, y + size * 0.35, z * 1.05);
                    nuggets.beam(at, at + Vec3::new(0.2, 0.03, 0.1), 0.16, 0.14, Color::WHITE);
                }
            }
        }
    }
    let mut entity = world.spawn((
        Transform::from_translation(base),
        Visibility::default(),
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    if !nuggets.is_empty() {
        entity.with_child((
            Mesh3d(assets.meshes.add(nuggets.build())),
            MeshMaterial3d(gold.clone()),
        ));
    }
    (ground, 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    #[test]
    fn trains_go_up_and_down_their_line_gently() {
        let length = 120.0;
        let mut last = train_distance(length, 0.0);
        let mut fastest = 0.0f32;
        let (mut lowest, mut highest) = (f32::MAX, 0.0f32);
        for step in 1..20_000 {
            let time = step as f32 * 0.01;
            let distance = train_distance(length, time);
            fastest = fastest.max((distance - last).abs() / 0.01);
            lowest = lowest.min(distance);
            highest = highest.max(distance);
            last = distance;
        }
        assert!(fastest <= CART_SPEED * 1.01, "{fastest}");
        assert!(lowest < 0.01 && highest > length - 0.01);
    }

    #[test]
    fn a_rail_path_is_measured_along_its_points() {
        let path = RailPath::new(vec![
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(10.0, 0.0, -10.0),
        ]);
        assert_eq!(path.length(), 20.0);
        let (at, way) = path.at(15.0);
        assert!(at.distance(Vec3::new(5.0, 0.0, -10.0)) < 1e-4);
        assert!(way.distance(Vec3::X) < 1e-4);
        assert_eq!(path.at(-3.0).0, Vec3::ZERO);
        assert_eq!(path.at(99.0).0, Vec3::new(10.0, 0.0, -10.0));
    }

    #[test]
    fn mine_rails_run_beside_the_road_on_the_ground() {
        for circuit in CIRCUITS.iter() {
            let Some(mine) = &circuit.scenery.mine else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            for rails in mine.rails {
                assert!(rails.lateral > WALL_DISTANCE + 2.0, "{}", circuit.name);
                let path = rail_path(&track, &terrain, rails);
                assert!(path.length() > 50.0);
                for pair in path.points.windows(2) {
                    // Never buried, never far above the ground, never
                    // steep.
                    for point in pair {
                        let ground = terrain.height(Vec2::new(point.x, point.z));
                        let above = point.y - ground;
                        assert!((0.0..0.8).contains(&above), "{} {above}", circuit.name);
                        assert!(
                            track.distance_to_centerline(Vec2::new(point.x, point.z))
                                > WALL_DISTANCE + 1.0
                        );
                    }
                    let grade = (pair[1].y - pair[0].y) / pair[0].distance(pair[1]);
                    assert!(grade.abs() < 0.2, "{} {grade}", circuit.name);
                }
            }
        }
    }

    #[test]
    fn trestles_run_from_rock_to_rock_high_over_the_road() {
        for circuit in CIRCUITS.iter() {
            let Some(mine) = &circuit.scenery.mine else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            for trestle in mine.trestles {
                let (_, center, _, left) = road_frame(&track, trestle.along);
                let deck = center.y + TRESTLE_HEIGHT;
                // Into the rock on both sides, where the cart goes.
                for side in [-1.0, 1.0] {
                    let end = Vec3::new(center.x, 0.0, center.z) + left * side * TRESTLE_RUN;
                    let rock = terrain.height(Vec2::new(end.x, end.z));
                    assert!(
                        rock > deck + 1.5,
                        "{} at {}: {rock} {deck}",
                        circuit.name,
                        trestle.along
                    );
                }
            }
        }
    }
}
