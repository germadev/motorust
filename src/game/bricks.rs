//! Plastic building bricks as big as the studs of the diorama's baseplate,
//! as a child's would be beside the toy bikes (see [`Bricks`]): heaps of
//! them spilled on the floor, towers of them stacked crosswise, little
//! houses under roofs stepping up to their ridges, and walls of them along
//! the road ([`WallStyle::Bricks`]). Each brick is a box with its studs on
//! top, a little narrower than its studs' room so that a thin joint shows
//! between bricks laid side by side; all of them go into one mesh of glossy
//! plastic, their colours in its vertices (see [`super::toys::store_pieces`]).

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use super::toys::{BrickKind, Bricks, runs, y};
use motorust_engine::geometry::random01;
use motorust_engine::track::Track;
use motorust_engine::world::quads::MeshBuilder;
use motorust_engine::world::scenery::{Scenery, Side, WallStyle};

/// How far apart a brick's studs are (m): as the diorama baseplate's.
pub const STUD: f32 = 1.2;
/// How tall a brick is without its studs, and a plate, a third of it; how
/// tall and how far round its studs are: a real brick's proportions.
pub const BRICK: f32 = 1.2 * STUD;
const PLATE: f32 = BRICK / 3.0;
const STUD_HEIGHT: f32 = 0.2 * STUD;
const STUD_RADIUS: f32 = 0.3 * STUD;
const STUD_SIDES: usize = 10;
/// How much narrower than its studs' room a brick is on each side (m).
const GAP: f32 = 0.03;

/// The bricks' colours, as a box of them has: bright red, blue, yellow and
/// green, white, orange and light grey.
const COLORS: [Color; 7] = [
    Color::srgb(0.79, 0.1, 0.04),
    Color::srgb(0.0, 0.33, 0.75),
    Color::srgb(0.95, 0.8, 0.22),
    Color::srgb(0.14, 0.55, 0.26),
    Color::srgb(0.95, 0.95, 0.93),
    Color::srgb(1.0, 0.54, 0.09),
    Color::srgb(0.63, 0.65, 0.66),
];
const RED: Color = COLORS[0];
const BLUE: Color = COLORS[1];
const YELLOW: Color = COLORS[2];
const GREEN: Color = COLORS[3];
const WHITE: Color = COLORS[4];
const GREY: Color = COLORS[6];

/// One of `COLORS`, as `pick` (0 to 1) says.
fn color(pick: f32) -> Color {
    COLORS[(pick * COLORS.len() as f32) as usize % COLORS.len()]
}

/// The plastic the bricks are made of: glossy.
pub fn material() -> StandardMaterial {
    StandardMaterial {
        perceptual_roughness: 0.28,
        reflectance: 0.5,
        ..default()
    }
}

/// What is built of bricks at `bricks`, into `mesh`.
pub fn build(mesh: &mut MeshBuilder, bricks: &Bricks, seed: u32) {
    let at = Vec3::new(bricks.position.x, y(bricks.base), bricks.position.y);
    match bricks.kind {
        BrickKind::Heap { count } => heap(mesh, at, bricks.heading, count, seed),
        BrickKind::Tower { count } => tower(mesh, at, bricks.heading, count, seed),
        BrickKind::House {
            width,
            depth,
            courses,
        } => house(mesh, at, bricks.heading, width, depth, courses, seed),
    }
}

/// A brick `size` studs across (its x) and along (its z), `tall` high, in
/// its own frame with the middle of its underside at the origin, placed by
/// `place`, painted `color`, with the studs on its top that `stud` says (by
/// their column and row).
fn brick(
    mesh: &mut MeshBuilder,
    size: UVec2,
    tall: f32,
    place: &Transform,
    color: Color,
    stud: impl Fn(u32, u32) -> bool,
) {
    let (across, along) = (size.x as f32 * STUD, size.y as f32 * STUD);
    let half = Vec3::new(across / 2.0 - GAP, tall / 2.0, along / 2.0 - GAP);
    let middle = Vec3::Y * tall / 2.0;
    for normal in [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ] {
        let (u, v) = if normal.y.abs() > 0.5 {
            (Vec3::X, Vec3::Z)
        } else {
            (normal.cross(Vec3::Y), Vec3::Y)
        };
        let corner = |a: f32, b: f32| {
            let local = middle + half * (normal + u * a + v * b);
            (place.transform_point(local), Vec2::ZERO)
        };
        mesh.quad(
            [
                corner(-1.0, -1.0),
                corner(1.0, -1.0),
                corner(1.0, 1.0),
                corner(-1.0, 1.0),
            ],
            place.rotation * normal,
            color,
        );
    }
    for column in 0..size.x {
        for row in 0..size.y {
            if stud(column, row) {
                let at = Vec3::new(
                    (column as f32 + 0.5) * STUD - across / 2.0,
                    tall,
                    (row as f32 + 0.5) * STUD - along / 2.0,
                );
                stud_on(mesh, place, at, color);
            }
        }
    }
}

/// A brick's stud standing on its top at `at`, in the brick's frame placed
/// by `place`.
fn stud_on(mesh: &mut MeshBuilder, place: &Transform, at: Vec3, color: Color) {
    let around = |k: usize| {
        let angle = k as f32 / STUD_SIDES as f32 * TAU;
        Vec3::new(angle.cos(), 0.0, angle.sin()) * STUD_RADIUS
    };
    let top = at + Vec3::Y * STUD_HEIGHT;
    let point = |local: Vec3| (place.transform_point(local), Vec2::ZERO);
    for k in 0..STUD_SIDES {
        let (a, b) = (around(k), around(k + 1));
        mesh.quad(
            [point(at + a), point(at + b), point(top + b), point(top + a)],
            place.rotation * (a + b),
            color,
        );
        mesh.triangle(
            [point(top), point(top + a), point(top + b)],
            place.rotation * Vec3::Y,
            color,
        );
    }
}

/// The shapes of the bricks in a heap (studs across and along).
const HEAP_SIZES: [UVec2; 7] = [
    UVec2::new(2, 4),
    UVec2::new(2, 2),
    UVec2::new(1, 4),
    UVec2::new(1, 2),
    UVec2::new(2, 3),
    UVec2::new(2, 6),
    UVec2::new(1, 6),
];

/// How far out from its middle a heap of `count` bricks spreads them (m).
pub fn heap_reach(count: u32) -> f32 {
    STUD * (1.4 * (count as f32).sqrt() + 1.0)
}

/// A heap of `count` bricks spilled about `at`, on the ground there, turned
/// `heading`, `seed` choosing them: most lying flat, a few on their sides,
/// some stacked on those whose studs face up, their studs in theirs.
fn heap(mesh: &mut MeshBuilder, at: Vec3, heading: f32, count: u32, seed: u32) {
    let reach = heap_reach(count);
    let mut n = seed.wrapping_mul(7919);
    let mut next = || {
        n = n.wrapping_add(1);
        random01(n)
    };
    let turn = Quat::from_rotation_y(heading);
    // Those with their studs up, to stack on, and the circles of those on
    // the ground, to keep clear of.
    let mut up: Vec<(Transform, UVec2)> = Vec::new();
    let mut ground: Vec<(Vec2, f32)> = Vec::new();
    for _ in 0..count {
        let size = HEAP_SIZES[(next() * HEAP_SIZES.len() as f32) as usize % HEAP_SIZES.len()];
        let paint = color(next());
        if !up.is_empty() && next() < 0.35 {
            let (base, under) = up[(next() * up.len() as f32) as usize % up.len()];
            let crossed = next() < 0.5;
            let place = stacked(&base, under, size, crossed, next(), next());
            brick(mesh, size, BRICK, &place, paint, |_, _| true);
            up.push((place, size));
            continue;
        }
        for _ in 0..40 {
            let (r, angle) = (reach * next().sqrt(), next() * TAU);
            let local = Vec2::new(angle.cos(), angle.sin()) * r;
            let on_its_side = next() < 0.25;
            let footprint = if on_its_side {
                Vec2::new(BRICK + STUD_HEIGHT, size.y as f32 * STUD)
            } else {
                size.as_vec2() * STUD
            };
            let radius = footprint.length() / 2.0;
            if ground
                .iter()
                .any(|&(other, round)| other.distance(local) < round + radius - 0.8)
            {
                continue;
            }
            ground.push((local, radius));
            let mut rotation = turn * Quat::from_rotation_y(next() * TAU);
            if on_its_side {
                rotation *= Quat::from_rotation_z(FRAC_PI_2);
            }
            // Lying on the ground: as high as its lowest corner is low.
            let half = Vec3::new(
                size.x as f32 * STUD / 2.0,
                BRICK,
                size.y as f32 * STUD / 2.0,
            );
            let lowest = [-1.0, 1.0]
                .into_iter()
                .flat_map(|x| [0.0, 1.0].map(|y| (x, y)))
                .flat_map(|(x, y)| [-1.0, 1.0].map(|z| Vec3::new(x, y, z) * half))
                .map(|corner| (rotation * corner).y)
                .fold(f32::INFINITY, f32::min);
            let place = Transform::from_translation(
                at + turn * Vec3::new(local.x, 0.0, local.y) - Vec3::Y * lowest,
            )
            .with_rotation(rotation);
            brick(mesh, size, BRICK, &place, paint, |_, _| true);
            if !on_its_side {
                up.push((place, size));
            }
            break;
        }
    }
}

/// Where a brick `size` studs goes stacked on `base`, a brick `under` studs
/// with its studs up: its studs in the base's, crossed over it or along it,
/// a stud or more of it over the base, `pick_x` and `pick_z` (0 to 1) saying
/// which of the places that leaves.
fn stacked(
    base: &Transform,
    under: UVec2,
    size: UVec2,
    crossed: bool,
    pick_x: f32,
    pick_z: f32,
) -> Transform {
    // How far it is across the base and along it, in the base's frame.
    let over = if crossed {
        UVec2::new(size.y, size.x)
    } else {
        size
    };
    // Its middle as far off the base's as its studs fall in theirs: by
    // whole studs, and half of one more where the two differ by an odd
    // number of studs.
    let offset = |below: u32, above: u32, pick: f32| {
        let odd = (below as f32 - above as f32) / 2.0;
        let most = (below + above) as f32 / 2.0 - 1.0;
        let places: Vec<f32> = (-8..=8)
            .map(|k| k as f32 + odd)
            .filter(|offset| offset.abs() <= most)
            .collect();
        places[(pick * places.len() as f32) as usize % places.len()] * STUD
    };
    let (x, z) = (
        offset(under.x, over.x, pick_x),
        offset(under.y, over.y, pick_z),
    );
    let turn = if crossed {
        Quat::from_rotation_y(FRAC_PI_2)
    } else {
        Quat::IDENTITY
    };
    Transform::from_translation(base.transform_point(Vec3::new(x, BRICK, z)))
        .with_rotation(base.rotation * turn)
}

/// A tower of `count` 2×4 bricks stacked crosswise on `at`, turned
/// `heading`, in colours one after another, now and then one a stud off
/// the one under it, `seed` choosing which.
fn tower(mesh: &mut MeshBuilder, at: Vec3, heading: f32, count: u32, seed: u32) {
    let size = UVec2::new(2, 4);
    let mut place = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(heading));
    for level in 0..count {
        let n = seed.wrapping_mul(104_729).wrapping_add(level * 3);
        let paint = COLORS[(seed as usize + level as usize * 3) % COLORS.len()];
        brick(mesh, size, BRICK, &place, paint, |_, _| true);
        // The next across it, mostly over its middle.
        let shift = |k: u32| {
            if random01(n + k) < 0.7 {
                0.5
            } else {
                random01(n + k + 7)
            }
        };
        place = stacked(&place, size, size, true, shift(1), shift(2));
    }
}

/// The bricks of a row from stud `from` to stud `to`, but for its `holes`
/// (from, to), their joints every four studs from `from` and `shift` more,
/// so that those of one course fall between those of the next: each one's
/// first stud and how many it has.
fn row(from: i32, to: i32, holes: &[(i32, i32)], shift: i32) -> Vec<(i32, i32)> {
    let mut bricks = Vec::new();
    let mut start = from;
    for x in from + 1..=to {
        let hole = |x: i32| holes.iter().any(|&(a, b)| (a..b).contains(&x));
        let joint = (x - from - shift).rem_euclid(4) == 0;
        let edge = x == to || hole(x) != hole(x - 1);
        if (joint || edge) && !hole(x - 1) && start < x {
            bricks.push((start, x - start));
        }
        if joint || edge || hole(x - 1) {
            start = x;
        }
    }
    bricks
}

/// A little house of bricks on `at`, turned `heading` (its front along
/// +z), `width` studs across its front and `depth` deep, its walls a stud
/// thick and `courses` bricks high in a running bond, a door in the middle
/// of its front and windows either side of it and in the middle of each of
/// its other walls; over them a roof stepping up a stud from its front and
/// its back to its ridge; `seed` choosing its colours.
fn house(
    mesh: &mut MeshBuilder,
    at: Vec3,
    heading: f32,
    width: u32,
    depth: u32,
    courses: u32,
    seed: u32,
) {
    let (w, d) = (width as i32, depth as i32);
    let walls = [WHITE, YELLOW, WHITE, GREY][seed as usize % 4];
    let roof = [RED, BLUE, RED, GREEN][(seed as usize / 2) % 4];
    let turn = Quat::from_rotation_y(heading);
    // A brick `size` studs, its least stud at stud `x`, `z` of the house
    // (counted from its back left corner), `level` up.
    let lay = |mesh: &mut MeshBuilder,
               x: i32,
               z: i32,
               size: UVec2,
               level: f32,
               paint: Color,
               stud: &dyn Fn(u32, u32) -> bool| {
        let middle = Vec3::new(
            (x as f32 + size.x as f32 / 2.0 - w as f32 / 2.0) * STUD,
            level,
            (z as f32 + size.y as f32 / 2.0 - d as f32 / 2.0) * STUD,
        );
        let place = Transform::from_translation(at + turn * middle).with_rotation(turn);
        brick(mesh, size, BRICK, &place, paint, stud);
    };
    let mut n = seed.wrapping_mul(31_337);
    let mut accent = |paint: Color| {
        n = n.wrapping_add(1);
        if random01(n) < 0.12 {
            color(random01(n * 3))
        } else {
            paint
        }
    };
    let middle = (w / 2 - 1, w / 2 + 1);
    let none = |_: u32, _: u32| false;
    for course in 0..courses as i32 {
        let level = course as f32 * BRICK;
        let shift = if course % 2 == 0 { 0 } else { 2 };
        // The door up to two courses; the windows on the next two.
        let window = (1..3).contains(&course);
        let mut front = Vec::new();
        if course < 2 {
            front.push(middle);
        }
        if window && w >= 10 {
            front.extend([(1, 3), (w - 3, w - 1)]);
        }
        let back: Vec<(i32, i32)> = if window { vec![middle] } else { vec![] };
        let sides: Vec<(i32, i32)> = if window && d >= 6 {
            vec![(d / 2 - 1, d / 2 + 1)]
        } else {
            vec![]
        };
        // The corners on the front and back on one course, on the sides on
        // the next.
        let (across, along) = if course % 2 == 0 {
            ((0, w), (1, d - 1))
        } else {
            ((1, w - 1), (0, d))
        };
        for (z, holes) in [(d - 1, &front), (0, &back)] {
            for (x, length) in row(across.0, across.1, holes, shift) {
                let paint = accent(walls);
                lay(
                    mesh,
                    x,
                    z,
                    UVec2::new(length as u32, 1),
                    level,
                    paint,
                    &none,
                );
            }
        }
        for x in [0, w - 1] {
            for (z, length) in row(along.0, along.1, &sides, shift) {
                let paint = accent(walls);
                lay(
                    mesh,
                    x,
                    z,
                    UVec2::new(1, length as u32),
                    level,
                    paint,
                    &none,
                );
            }
        }
    }
    // The roof: each step a stud in from the one under it at the front and
    // the back, its studs where the next does not cover them.
    let steps = (d + 1) / 2;
    for step in 0..steps {
        let level = (courses as i32 + step) as f32 * BRICK;
        let deep = d - 2 * step;
        let top = step == steps - 1;
        let shift = if step % 2 == 0 { 2 } else { 0 };
        for (x, length) in row(0, w, &[], shift) {
            let rim = move |_: u32, row: u32| top || row == 0 || row + 1 == deep as u32;
            lay(
                mesh,
                x,
                step,
                UVec2::new(length as u32, deep as u32),
                level,
                roof,
                &rim,
            );
        }
    }
}

/// How far outside the road's walls a wall of bricks stands (m): a stud
/// thick, from the walls out; and how far from an arch over the road it
/// leaves off (m), for the arch's feet.
const BRICK_WALL: f32 = STUD;
const ARCH_GAP: f32 = 6.0;

/// The walls of bricks along the lap ([`WallStyle::Bricks`]): on each side
/// where the wall is of them, just outside it, a course of 1×4 bricks and
/// a course of 1×4 plates on it in a running bond, following the road's
/// surface there, studs on the plates, each brick a colour of its own; a
/// gap left where an arch stands over the road.
pub fn walls(mesh: &mut MeshBuilder, track: &Track, scenery: &Scenery) {
    let lap = track.lap_length();
    let step = track.step_length();
    let n = track.sample_count();
    for side in [Side::Left, Side::Right] {
        let of_bricks: Vec<usize> = (0..n)
            .filter(|&i| {
                motorust_engine::world::road::wall_style(scenery, lap, i as f32 * step, side)
                    == Some(WallStyle::Bricks)
            })
            .collect();
        for run in runs(&of_bricks, n) {
            // The line of its middle, and how far along it each point is.
            let line: Vec<Vec3> = run
                .iter()
                .map(|&i| {
                    track.surface_point(i, side.sign() * (track.wall_at(i) + BRICK_WALL / 2.0))
                })
                .collect();
            let mut along = vec![0.0];
            for pair in line.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                along.push(along.last().unwrap() + Vec2::new(b.x - a.x, b.z - a.z).length());
            }
            let total = *along.last().unwrap();
            let at = |s: f32| {
                let k = along.partition_point(|&a| a < s).clamp(1, line.len() - 1);
                let t = (s - along[k - 1]) / (along[k] - along[k - 1]).max(1e-6);
                (line[k - 1].lerp(line[k], t.clamp(0.0, 1.0)), run[k - 1])
            };
            let length = 4.0 * STUD;
            for (course, tall, offset) in [(0, BRICK, 0.0), (1, PLATE, 2.0 * STUD)] {
                let mut from = if offset > 0.0 { offset - length } else { 0.0 };
                let mut k = 0u32;
                while from < total - STUD {
                    let (start, end) = (from.max(0.0), (from + length).min(total));
                    from += length;
                    k += 1;
                    let studs = ((end - start) / STUD).floor() as u32;
                    if studs == 0 {
                        continue;
                    }
                    let end = start + studs as f32 * STUD;
                    let ((p, index), (q, _)) = (at(start), at(end));
                    let (_, middle) = at((start + end) / 2.0);
                    let near_an_arch = scenery.gantries.iter().any(|gantry| {
                        let apart = (middle as f32 * step - gantry.along).rem_euclid(lap);
                        apart.min(lap - apart) < ARCH_GAP + length / 2.0
                    });
                    if near_an_arch {
                        continue;
                    }
                    let ahead = (q - p).normalize_or_zero();
                    let up = (Vec3::Y - ahead * ahead.y).normalize_or_zero();
                    let rotation = Quat::from_mat3(&Mat3::from_cols(up.cross(ahead), up, ahead));
                    let level = if course == 0 { 0.0 } else { BRICK };
                    let place = Transform::from_translation((p + q) / 2.0 + up * (level - 0.05))
                        .with_rotation(rotation);
                    let seed =
                        (index as u32) * 2 + u32::from(side == Side::Left) + course * 7919 + k;
                    brick(
                        mesh,
                        UVec2::new(1, studs),
                        tall,
                        &place,
                        color(random01(seed * 13)),
                        |_, _| course == 1,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_of_bricks_leaves_its_holes_and_breaks_its_joints() {
        crate::game::register();
        // Four studs at most each, from end to end but for the hole.
        let bricks = row(0, 12, &[(5, 7)], 0);
        assert_eq!(bricks, vec![(0, 4), (4, 1), (7, 1), (8, 4)]);
        // The next course's joints fall between these.
        let next = row(0, 12, &[], 2);
        assert_eq!(next, vec![(0, 2), (2, 4), (6, 4), (10, 2)]);
    }

    #[test]
    fn stacked_bricks_have_their_studs_in_those_under_them() {
        crate::game::register();
        // Its studs on the base's grid, the bricks overlapping.
        let base = Transform::default();
        for (under, size, crossed) in [
            (UVec2::new(2, 4), UVec2::new(2, 4), true),
            (UVec2::new(2, 4), UVec2::new(1, 2), false),
            (UVec2::new(2, 3), UVec2::new(2, 2), true),
            (UVec2::new(1, 6), UVec2::new(2, 3), false),
        ] {
            for pick in [0.0, 0.3, 0.6, 0.99] {
                let place = stacked(&base, under, size, crossed, pick, 1.0 - pick);
                assert!((place.translation.y - BRICK).abs() < 1e-5);
                let over = if crossed {
                    UVec2::new(size.y, size.x)
                } else {
                    size
                };
                for (axis, below, above) in [(0, under.x, over.x), (2, under.y, over.y)] {
                    let offset = place.translation[axis] / STUD;
                    let grid = offset - (below as f32 - above as f32) / 2.0;
                    assert!(
                        (grid - grid.round()).abs() < 1e-4,
                        "{under} {size} {offset}"
                    );
                    assert!(offset.abs() <= (below + above) as f32 / 2.0 - 1.0 + 1e-4);
                }
            }
        }
    }

    #[test]
    fn a_heap_lies_on_the_ground_within_its_reach() {
        crate::game::register();
        let mut mesh = MeshBuilder::default();
        heap(&mut mesh, Vec3::ZERO, 0.4, 16, 3);
        let (low, high) = motorust_engine::world::prop_pack::bounds(&mesh.build());
        assert!(low.y.abs() < 0.01, "{low}");
        let reach = heap_reach(16) + 4.0 * STUD;
        assert!(Vec2::new(low.x, low.z).length() < reach * 1.42, "{low}");
        assert!(Vec2::new(high.x, high.z).length() < reach * 1.42, "{high}");
        assert!(high.y > BRICK && high.y < 6.0 * BRICK, "{high}");
    }

    #[test]
    fn the_walls_of_bricks_stand_outside_the_road_s_walls() {
        crate::game::register();
        // Each of their corners outside the walls, a stud out at most, and
        // none of them by an arch over the road.
        let circuit = crate::game::CIRCUITS
            .iter()
            .find(|circuit| {
                circuit
                    .scenery
                    .themes
                    .get::<super::super::toys::ToyStore>()
                    .is_some()
            })
            .expect("a toy store");
        let track = Track::new(&circuit.layout);
        let mut mesh = MeshBuilder::default();
        walls(&mut mesh, &track, &circuit.scenery);
        assert!(!mesh.is_empty());
        let mesh = mesh.build();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        let lap = track.lap_length();
        for point in points {
            let ground = Vec2::new(point[0], point[2]);
            let nearest = track.nearest_at(ground, point[1]);
            let wall = track.road(nearest.road).wall_at(nearest.index);
            assert!(
                (wall - 0.1..wall + BRICK_WALL + 0.6).contains(&nearest.distance),
                "{ground}: {} m from the road, its walls {wall} m",
                nearest.distance
            );
            let along = track.along(&nearest);
            for gantry in circuit.scenery.gantries {
                let apart = (along - gantry.along).rem_euclid(lap);
                assert!(
                    apart.min(lap - apart) > 4.0,
                    "{ground}: by the arch at {}",
                    gantry.along
                );
            }
        }
    }

    /// How far a house of bricks `width` studs across and `depth` deep
    /// reaches from its middle (m), and how tall it is over its base, its
    /// walls `courses` bricks high and the roof's steps over them.
    fn house_size(width: u32, depth: u32, courses: u32) -> (f32, f32) {
        let reach = Vec2::new(width as f32, depth as f32).length() * STUD / 2.0;
        let steps = depth.div_ceil(2);
        (reach, (courses + steps) as f32 * BRICK + STUD_HEIGHT)
    }

    #[test]
    fn a_house_is_as_big_as_it_says() {
        crate::game::register();
        let mut mesh = MeshBuilder::default();
        house(&mut mesh, Vec3::ZERO, 0.0, 8, 6, 4, 1);
        let (low, high) = motorust_engine::world::prop_pack::bounds(&mesh.build());
        let (reach, tall) = house_size(8, 6, 4);
        assert!((high.y - tall).abs() < 0.01, "{high} {tall}");
        assert!(low.y.abs() < 0.01);
        assert!((high.x - 4.0 * STUD).abs() < 0.05 && (high.z - 3.0 * STUD).abs() < 0.05);
        assert!(Vec2::new(high.x, high.z).length() <= reach + 0.01);
    }
}
