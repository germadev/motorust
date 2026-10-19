//! The works of a toy store around the road ([`Toys`], see
//! [`crate::world::scenery`]): shelves full of toy boxes, toy trains running
//! round circular tracks, piles of giant building blocks, dice and balls.
//!
//! The world round the bikes is built at the scale of the toys, while the
//! bikes keep their usual size and physics: the shop and its playthings stand
//! far above them. Every shelf row, rail ring, pile and die is one mesh
//! coloured vertex by vertex, so each kind draws in a batch; only the train
//! carriages move, each a transform updated every frame.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Assets3d;
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{
    GiantBall, GiantDice, Scenery, Shelf, Side, ToyBlockPile, ToyTable, ToyTrain, WaterTable,
};
use super::terrain::Terrain;
use crate::geometry::{forward, random01};
use crate::track::Track;

/// The colours of the toy boxes, blocks and carriages.
const PLASTIC: [Color; 6] = [
    Color::srgb(0.9, 0.16, 0.12),
    Color::srgb(1.0, 0.8, 0.16),
    Color::srgb(0.14, 0.4, 0.9),
    Color::srgb(0.18, 0.68, 0.3),
    Color::srgb(0.96, 0.46, 0.1),
    Color::srgb(0.94, 0.92, 0.86),
];
/// The dark frame of a shelf, the rails and sleepers of a toy train, and the
/// pips of a die.
const FRAME: Color = Color::srgb(0.3, 0.28, 0.34);
const RAIL: Color = Color::srgb(0.55, 0.56, 0.6);
const SLEEPER: Color = Color::srgb(0.38, 0.27, 0.18);
const PIP: Color = Color::srgb(0.12, 0.12, 0.14);
/// The pale wood of the shop's shelves, tables and chairs.
const SHELF_WOOD: Color = Color::srgb(0.78, 0.58, 0.36);
const TABLE_WOOD: Color = Color::srgb(0.82, 0.62, 0.4);
const TABLE_WOOD_DARK: Color = Color::srgb(0.58, 0.4, 0.24);

/// A toy train carriage going round its track: the middle and radius of the
/// circle, how far round it has come, how fast (m/s) and the height of its
/// rails.
#[derive(Component)]
pub struct ToyTrainCar {
    center: Vec2,
    radius: f32,
    phase: f32,
    speed: f32,
    height: f32,
}

/// How long a carriage is and how far apart they stand (m), and how many
/// pieces each circular track has.
const CAR_LENGTH: f32 = 5.0;
const CAR_SPACING: f32 = 5.8;
const TRACK_STEPS: usize = 96;
/// Half the gauge of the toy rails (m).
const GAUGE: f32 = 2.4;

/// Rolls the toy trains round their tracks.
pub fn roll_trains(time: Res<Time>, mut cars: Query<(&ToyTrainCar, &mut Transform)>) {
    for (car, mut transform) in &mut cars {
        let angle = car.phase + car.speed * time.elapsed_secs() / car.radius;
        let (sin, cos) = angle.sin_cos();
        let position = car.center + Vec2::new(cos, sin) * car.radius;
        let ahead = Vec2::new(-sin, cos);
        transform.translation = Vec3::new(position.x, car.height, position.y);
        transform.rotation = Transform::default()
            .looking_to(Vec3::new(ahead.x, 0.0, ahead.y), Vec3::Y)
            .rotation;
    }
}

/// Builds the toys of `scenery`, if it has any, and returns the ground they
/// take.
pub fn spawn_toys(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let Some(toys) = &scenery.toys else {
        return Vec::new();
    };
    let painted = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.7,
        ..default()
    });
    super::room::spawn_room(world, &toys.room, assets);
    if let Some(table) = &toys.water_table {
        spawn_water_table(world, terrain, table, &painted, assets);
    }
    let mut taken = Vec::new();
    for shelf in toys.shelves {
        taken.push(spawn_shelf(world, track, terrain, shelf, &painted, assets));
    }
    for train in toys.trains {
        taken.push(spawn_train(world, track, terrain, train, &painted, assets));
    }
    for table in toys.tables {
        taken.push(spawn_table(world, track, terrain, table, &painted, assets));
    }
    for pile in toys.block_piles {
        taken.push(spawn_blocks(world, track, terrain, pile, &painted, assets));
    }
    for dice in toys.dice {
        taken.push(spawn_die(world, track, terrain, dice, &painted, assets));
    }
    for ball in toys.balls {
        taken.push(spawn_ball(world, track, terrain, ball, assets));
    }
    taken
}

/// Where something `lateral` metres from the centre line on `side` of the
/// point `along` the lap stands, and its frame: `along` runs the way of the
/// race, `out` away from the road on that side, `up` is up.
fn beside(track: &Track, along: f32, side: Side, lateral: f32) -> (Vec2, Vec3, Vec3, Vec3) {
    let index = track.index_at(along);
    let (_, heading) = track.pose_at(index);
    let ahead = forward(heading);
    let left = Vec2::new(ahead.y, -ahead.x);
    let out = left * side.sign();
    (
        track.offset_point(index, side.sign() * lateral),
        Vec3::new(ahead.x, 0.0, ahead.y),
        Vec3::new(out.x, 0.0, out.y),
        Vec3::Y,
    )
}

/// A box with its middle at `center`, its axes `x`, `y` and `z` and its half
/// sizes along them.
pub(super) fn box_at(
    mesh: &mut MeshBuilder,
    center: Vec3,
    axes: [Vec3; 3],
    half: Vec3,
    color: Color,
) {
    let [x, y, z] = axes;
    let p = |sx: i32, sy: i32, sz: i32| {
        center + x * (half.x * sx as f32) + y * (half.y * sy as f32) + z * (half.z * sz as f32)
    };
    let faces: [([Vec3; 4], Vec3); 6] = [
        ([p(1, -1, -1), p(1, 1, -1), p(1, 1, 1), p(1, -1, 1)], x),
        ([p(-1, -1, -1), p(-1, -1, 1), p(-1, 1, 1), p(-1, 1, -1)], -x),
        ([p(-1, 1, -1), p(-1, 1, 1), p(1, 1, 1), p(1, 1, -1)], y),
        ([p(-1, -1, -1), p(1, -1, -1), p(1, -1, 1), p(-1, -1, 1)], -y),
        ([p(-1, -1, 1), p(1, -1, 1), p(1, 1, 1), p(-1, 1, 1)], z),
        ([p(-1, -1, -1), p(-1, 1, -1), p(1, 1, -1), p(1, -1, -1)], -z),
    ];
    for (corners, facing) in faces {
        mesh.quad(corners.map(|corner| (corner, Vec2::ZERO)), facing, color);
    }
}

/// The lowest ground under a rectangle `half` (along, out) about `spot`.
fn lowest_ground(terrain: &Terrain, spot: Vec2, ahead: Vec3, out: Vec3, half: Vec2) -> f32 {
    let mut low = terrain.height(spot);
    for (sa, so) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let corner = spot
            + Vec2::new(ahead.x, ahead.z) * (sa * half.x)
            + Vec2::new(out.x, out.z) * (so * half.y);
        low = low.min(terrain.height(corner));
    }
    low
}

/// A row of shelves full of toy boxes, one mesh for the whole row.
fn spawn_shelf(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    shelf: &Shelf,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let lap = track.lap_length();
    let length = shelf.span.length(lap);
    let count = (length / shelf.every).floor().max(1.0) as usize;
    let mut mesh = MeshBuilder::default();
    let alongs: Vec<f32> = (0..count)
        .map(|k| shelf.span.from + shelf.every / 2.0 + k as f32 * shelf.every)
        .collect();
    let middle = track.offset_point(
        track.index_at(alongs[count / 2]),
        shelf.side.sign() * shelf.lateral,
    );
    for &along in &alongs {
        let (spot, ahead, out, up) = beside(track, along, shelf.side, shelf.lateral);
        let half_along = shelf.every * 0.38;
        let base = lowest_ground(
            terrain,
            spot,
            ahead,
            out,
            Vec2::new(half_along, shelf.depth / 2.0),
        ) - 0.3;
        let post = 0.35;
        // The frame: four uprights and four boards.
        for (sa, so) in [(-1.0, 0.0), (1.0, 0.0), (-1.0, 1.0), (1.0, 1.0)] {
            let corner = spot
                + Vec2::new(ahead.x, ahead.z) * (sa * half_along)
                + Vec2::new(out.x, out.z) * (so * shelf.depth);
            let foot = Vec3::new(corner.x, base, corner.y);
            let head = foot + up * shelf.height;
            box_at(
                &mut mesh,
                (foot + head) / 2.0,
                [Vec3::X, up, Vec3::Z],
                Vec3::new(post / 2.0, shelf.height / 2.0, post / 2.0),
                SHELF_WOOD,
            );
        }
        for level in 0..4 {
            let height = base + shelf.height * level as f32 / 3.0;
            let center = Vec3::new(spot.x, height, spot.y) + out * (shelf.depth / 2.0);
            box_at(
                &mut mesh,
                center,
                [Vec3::new(ahead.x, 0.0, ahead.z), up, out],
                Vec3::new(half_along + 0.1, 0.2, shelf.depth / 2.0 + 0.1),
                SHELF_WOOD,
            );
        }
        // Toy boxes on the three spaces between the boards.
        for level in 0..3 {
            for slot in 0..3 {
                let u = (slot as f32 - 1.0) * half_along * 0.62;
                let center = Vec3::new(spot.x, 0.0, spot.y)
                    + Vec3::new(ahead.x, 0.0, ahead.z) * u
                    + out * (shelf.depth * 0.55)
                    + up * (base + shelf.height * (level as f32 + 0.42) / 3.0);
                let color = PLASTIC[(slot * 2 + level * 3 + (along as usize / 7)) % PLASTIC.len()];
                box_at(
                    &mut mesh,
                    center,
                    [Vec3::new(ahead.x, 0.0, ahead.z), up, out],
                    Vec3::new(half_along * 0.26, shelf.height * 0.13, shelf.depth * 0.4),
                    if shelf.colors.is_empty() {
                        color
                    } else {
                        shelf.colors[(slot + level) % shelf.colors.len()]
                    },
                );
            }
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    (middle, shelf.every * count as f32 / 2.0 + 3.0)
}

/// A toy train on its circular track, with the rails and the carriages.
fn spawn_train(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    train: &ToyTrain,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (center, _ahead, _out, _up) = beside(track, train.along, train.side, train.lateral);
    let height = terrain.height(center) + 0.12;
    // The track: two rails and the sleepers under them, all round.
    let mut rails = MeshBuilder::default();
    let step = TAU / TRACK_STEPS as f32;
    for k in 0..TRACK_STEPS {
        let a = k as f32 * step;
        let b = (k + 1) as f32 * step;
        let (sa, ca) = a.sin_cos();
        let (sb, cb) = b.sin_cos();
        for side in [-1.0, 1.0] {
            let pa = Vec3::new(
                center.x + ca * (train.radius + side * GAUGE),
                height,
                center.y + sa * (train.radius + side * GAUGE),
            );
            let pb = Vec3::new(
                center.x + cb * (train.radius + side * GAUGE),
                height,
                center.y + sb * (train.radius + side * GAUGE),
            );
            rails.beam(pa, pb, 0.24, 0.2, RAIL);
        }
        if k % 4 == 0 {
            let pa = Vec3::new(
                center.x + ca * (train.radius - GAUGE),
                height - 0.02,
                center.y + sa * (train.radius - GAUGE),
            );
            let pb = Vec3::new(
                center.x + ca * (train.radius + GAUGE),
                height - 0.02,
                center.y + sa * (train.radius + GAUGE),
            );
            rails.beam(pa, pb, 0.5, 0.14, SLEEPER);
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(rails.build())),
        MeshMaterial3d(painted.clone()),
    ));
    // The carriages: a locomotive and the rest, each a transform moved every
    // frame.
    let locomotive = assets.meshes.add(carriage_mesh(true));
    let carriage = assets.meshes.add(carriage_mesh(false));
    for place in 0..train.cars {
        let phase = place as f32 * CAR_SPACING / train.radius;
        let mesh = if place == 0 {
            locomotive.clone()
        } else {
            carriage.clone()
        };
        world.spawn((
            ToyTrainCar {
                center,
                radius: train.radius,
                phase,
                speed: train.speed,
                height,
            },
            Transform::from_xyz(center.x + train.radius, height, center.y),
            Visibility::default(),
            Mesh3d(mesh),
            MeshMaterial3d(painted.clone()),
        ));
    }
    (center, train.radius + CAR_LENGTH)
}

/// A table with a toy railway on it, chairs round it and a rug beneath.
fn spawn_table(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    table: &ToyTable,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (center, ahead, out, up) = beside(track, table.along, table.side, table.lateral);
    let half = Vec2::new(table.length / 2.0, table.width / 2.0);
    let ground = lowest_ground(terrain, center, ahead, out, half) - 0.05;
    let base = Vec3::new(center.x, ground, center.y);
    let axes = [ahead, up, out];
    let mut mesh = MeshBuilder::default();

    // The rug under the table, following the floor.
    rug(
        &mut mesh,
        terrain,
        center,
        ahead,
        out,
        half + Vec2::splat(10.0),
        table.rug,
    );

    // The top, its apron and its legs.
    let top = ground + table.height;
    box_at(
        &mut mesh,
        base + up * (table.height - 0.7),
        axes,
        Vec3::new(half.x, 0.7, half.y),
        TABLE_WOOD,
    );
    box_at(
        &mut mesh,
        base + up * (table.height - 2.4),
        axes,
        Vec3::new(half.x - 2.0, 1.2, half.y - 2.0),
        TABLE_WOOD_DARK,
    );
    let leg_height = table.height - 1.4;
    for (sa, so) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let foot = base
            + ahead * (sa * (half.x - 3.0))
            + out * (so * (half.y - 3.0))
            + up * (leg_height / 2.0);
        box_at(
            &mut mesh,
            foot,
            axes,
            Vec3::new(1.6, leg_height / 2.0, 1.6),
            TABLE_WOOD_DARK,
        );
    }

    // A toy railway on top: an oval of rails, its sleepers and a little train.
    let rx = table.length * 0.27;
    let rz = table.width * 0.27;
    let rail_height = top + 0.3;
    let rail_point = |angle: f32, offset: f32| {
        let (sin, cos) = angle.sin_cos();
        let radial = ahead * cos + out * sin;
        base + ahead * (cos * rx) + out * (sin * rz) + up * rail_height + radial * offset
    };
    const SEGMENTS: usize = 48;
    let gauge = 1.8;
    for k in 0..SEGMENTS {
        let a = k as f32 / SEGMENTS as f32 * TAU;
        let b = (k + 1) as f32 / SEGMENTS as f32 * TAU;
        for side in [-1.0, 1.0] {
            mesh.beam(
                rail_point(a, side * gauge),
                rail_point(b, side * gauge),
                0.32,
                0.28,
                RAIL,
            );
        }
        if k % 4 == 0 {
            mesh.beam(
                rail_point(a, -gauge),
                rail_point(a, gauge),
                0.38,
                0.16,
                SLEEPER,
            );
        }
    }
    for (index, phase) in [0.2f32, 1.5, 2.9, 4.2].into_iter().enumerate() {
        let (sin, cos) = phase.sin_cos();
        let at = rail_point(phase, 0.0) + up * 1.3;
        let tangent = (-ahead * (sin * rx) + out * (cos * rz)).normalize();
        let side = up.cross(tangent).normalize();
        let body = if index == 0 {
            Color::srgb(0.78, 0.16, 0.14)
        } else {
            PLASTIC[(index + 1) % PLASTIC.len()]
        };
        box_at(
            &mut mesh,
            at,
            [tangent, up, side],
            Vec3::new(2.4, 1.1, 0.9),
            body,
        );
        box_at(
            &mut mesh,
            at + up * 1.15,
            [tangent, up, side],
            Vec3::new(2.5, 0.25, 1.0),
            FRAME,
        );
    }

    // A little village and a few trees on the rest of the table.
    for (u, v, color) in [
        (-0.38, 0.34, Color::srgb(0.86, 0.24, 0.18)),
        (0.37, 0.32, Color::srgb(0.95, 0.78, 0.2)),
        (0.34, -0.34, Color::srgb(0.18, 0.4, 0.82)),
    ] {
        let at = base + ahead * (u * table.length) + out * (v * table.width);
        let at = Vec3::new(at.x, top, at.z);
        box_at(
            &mut mesh,
            at + up * 1.6,
            axes,
            Vec3::new(2.0, 1.6, 2.0),
            color,
        );
        box_at(
            &mut mesh,
            at + up * 3.8,
            axes,
            Vec3::new(2.6, 0.9, 2.6),
            TABLE_WOOD_DARK,
        );
    }
    for (u, v) in [(-0.45, -0.3), (0.05, 0.4), (0.5, 0.05)] {
        let at = base + ahead * (u * table.length) + out * (v * table.width);
        let at = Vec3::new(at.x, top, at.z);
        box_at(
            &mut mesh,
            at + up * 1.4,
            axes,
            Vec3::new(0.5, 1.4, 0.5),
            TABLE_WOOD_DARK,
        );
        box_at(
            &mut mesh,
            at + up * 4.0,
            axes,
            Vec3::new(2.6, 2.2, 2.6),
            Color::srgb(0.2, 0.62, 0.3),
        );
    }

    // The chairs, on the rug and facing the table.
    let seat = table.height * 0.55;
    let mut places = Vec::new();
    if table.chairs >= 6 {
        for sa in [-1.0, 1.0] {
            for so in [-1.0, 1.0] {
                places.push((
                    center
                        + Vec2::new(ahead.x, ahead.z) * (sa * table.length * 0.24)
                        + Vec2::new(out.x, out.z) * (so * (half.y + 7.0)),
                    out * so,
                ));
            }
        }
        places.push((center + Vec2::new(ahead.x, ahead.z) * (half.x + 8.0), ahead));
        places.push((
            center - Vec2::new(ahead.x, ahead.z) * (half.x + 8.0),
            -ahead,
        ));
    } else {
        places.push((center + Vec2::new(out.x, out.z) * (half.y + 7.0), out));
        places.push((center - Vec2::new(out.x, out.z) * (half.y + 7.0), -out));
        places.push((center + Vec2::new(ahead.x, ahead.z) * (half.x + 8.0), ahead));
        places.push((
            center - Vec2::new(ahead.x, ahead.z) * (half.x + 8.0),
            -ahead,
        ));
    }
    for (index, (at, outward)) in places.into_iter().enumerate() {
        let foot = Vec3::new(at.x, terrain.height(Vec2::new(at.x, at.y)) + 0.1, at.y);
        chair(
            &mut mesh,
            foot,
            outward,
            seat,
            PLASTIC[(index + 2) % PLASTIC.len()],
        );
    }

    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    (center, half.length() + 18.0)
}

/// A rug on the floor, following its height: `half` metres either way about
/// `center`, along `ahead` and `out`.
fn rug(
    mesh: &mut MeshBuilder,
    terrain: &Terrain,
    center: Vec2,
    ahead: Vec3,
    out: Vec3,
    half: Vec2,
    color: Color,
) {
    const ALONG: usize = 12;
    const ACROSS: usize = 8;
    let point = |u: f32, v: f32| {
        let at = center + Vec2::new(ahead.x, ahead.z) * u + Vec2::new(out.x, out.z) * v;
        Vec3::new(at.x, terrain.height(at) + 0.12, at.y)
    };
    for i in 0..ALONG {
        for j in 0..ACROSS {
            let u0 = -half.x + 2.0 * half.x * i as f32 / ALONG as f32;
            let u1 = -half.x + 2.0 * half.x * (i + 1) as f32 / ALONG as f32;
            let v0 = -half.y + 2.0 * half.y * j as f32 / ACROSS as f32;
            let v1 = -half.y + 2.0 * half.y * (j + 1) as f32 / ACROSS as f32;
            mesh.quad(
                [
                    (point(u0, v0), Vec2::ZERO),
                    (point(u1, v0), Vec2::X),
                    (point(u1, v1), Vec2::ONE),
                    (point(u0, v1), Vec2::Y),
                ],
                Vec3::Y,
                color,
            );
        }
    }
}

/// A chair with its foot at `foot`, its back away from the table along
/// `outward`, a seat `seat` metres high and a cushion of `cushion`.
fn chair(mesh: &mut MeshBuilder, foot: Vec3, outward: Vec3, seat: f32, cushion: Color) {
    let up = Vec3::Y;
    let side = up.cross(outward).normalize();
    let axes = [side, up, outward];
    for (a, b) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        box_at(
            mesh,
            foot + side * (a * 2.6) + outward * (b * 2.6) + up * (seat / 2.0),
            axes,
            Vec3::new(0.55, seat / 2.0, 0.55),
            TABLE_WOOD_DARK,
        );
    }
    box_at(
        mesh,
        foot + up * (seat + 0.5),
        axes,
        Vec3::new(3.4, 0.5, 3.4),
        TABLE_WOOD,
    );
    box_at(
        mesh,
        foot + up * (seat + 1.1),
        axes,
        Vec3::new(2.9, 0.35, 2.9),
        cushion,
    );
    box_at(
        mesh,
        foot + up * (seat + 5.2) + outward * 3.0,
        axes,
        Vec3::new(3.4, 4.4, 0.6),
        TABLE_WOOD,
    );
}

/// A carriage of a toy train, its colours in its vertices: a body with a
/// roof and windows, wheels, and, up front, a cab.
fn carriage_mesh(locomotive: bool) -> Mesh {
    let mut mesh = MeshBuilder::default();
    let x = Vec3::X;
    let y = Vec3::Y;
    let z = Vec3::Z;
    let body = if locomotive {
        Color::srgb(0.7, 0.14, 0.12)
    } else {
        Color::srgb(0.16, 0.38, 0.72)
    };
    // The body, the roof and the wheels.
    box_at(
        &mut mesh,
        Vec3::new(0.0, 1.0, 0.0),
        [x, y, z],
        Vec3::new(1.1, 0.7, CAR_LENGTH / 2.0),
        body,
    );
    box_at(
        &mut mesh,
        Vec3::new(0.0, 1.78, 0.0),
        [x, y, z],
        Vec3::new(1.18, 0.12, CAR_LENGTH / 2.0 + 0.05),
        FRAME,
    );
    for px in [-0.95, 0.95] {
        for pz in [-1.5, 1.5] {
            box_at(
                &mut mesh,
                Vec3::new(px, 0.5, pz),
                [x, y, z],
                Vec3::new(0.18, 0.5, 0.5),
                PIP,
            );
        }
    }
    if locomotive {
        box_at(
            &mut mesh,
            Vec3::new(0.0, 2.3, -0.9),
            [x, y, z],
            Vec3::new(0.8, 0.55, 1.0),
            body,
        );
        box_at(
            &mut mesh,
            Vec3::new(0.0, 3.0, -0.9),
            [x, y, z],
            Vec3::new(0.25, 0.5, 0.25),
            FRAME,
        );
    }
    mesh.build()
}

/// A pile of giant building blocks, scattered in a little heap and stacked.
fn spawn_blocks(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    pile: &ToyBlockPile,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (center, ahead, out, up) = beside(track, pile.along, pile.side, pile.lateral);
    let ground = terrain.height(center);
    let mut mesh = MeshBuilder::default();
    for k in 0..pile.count {
        let r = |n: u32| random01(pile.seed.wrapping_mul(97).wrapping_add(k * 31 + n));
        let along = (r(0) - 0.5) * 22.0;
        let away = (r(1) - 0.5) * 16.0;
        let size = 1.4 + r(2) * 1.8;
        let stacked = (r(3) * 2.0) as u32;
        let spot = Vec3::new(center.x, ground, center.y)
            + Vec3::new(ahead.x, 0.0, ahead.z) * along
            + out * away;
        let color = PLASTIC[(r(4) * PLASTIC.len() as f32) as usize % PLASTIC.len()];
        for layer in 0..=stacked {
            box_at(
                &mut mesh,
                spot + up * (size / 2.0 + layer as f32 * size),
                [Vec3::X, up, Vec3::Z],
                Vec3::splat(size / 2.0),
                color,
            );
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    (center, 18.0)
}

/// A giant die, with the pips on five of its faces.
fn spawn_die(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    dice: &GiantDice,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (center, _ahead, _out, up) = beside(track, dice.along, dice.side, dice.lateral);
    let ground = terrain.height(center) + dice.size / 2.0;
    let mut mesh = MeshBuilder::default();
    let spot = Vec3::new(center.x, ground, center.y);
    let half = dice.size / 2.0;
    box_at(
        &mut mesh,
        spot,
        [Vec3::X, up, Vec3::Z],
        Vec3::splat(half),
        Color::srgb(0.96, 0.94, 0.88),
    );
    let pip = dice.size * 0.07;
    // The five faces that are seen: up, ±X and ±Z, each with its own throw.
    for (face, value) in [
        (Vec3::Y, ((dice.size * 10.0) as u32) % 6 + 1),
        (Vec3::X, ((dice.size * 11.0) as u32) % 6 + 1),
        (-Vec3::X, ((dice.size * 12.0) as u32) % 6 + 1),
        (Vec3::Z, ((dice.size * 13.0) as u32) % 6 + 1),
        (-Vec3::Z, ((dice.size * 14.0) as u32) % 6 + 1),
    ] {
        let (u, v) = if face.y.abs() > 0.5 {
            (Vec3::X, Vec3::Z)
        } else if face.x.abs() > 0.5 {
            (Vec3::Z, Vec3::Y)
        } else {
            (Vec3::X, Vec3::Y)
        };
        for (a, b) in pip_grid(value) {
            let at =
                spot + face * (half + pip * 0.4) + u * (a * half * 0.55) + v * (b * half * 0.55);
            box_at(&mut mesh, at, [Vec3::X, up, Vec3::Z], Vec3::splat(pip), PIP);
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    (center, dice.size)
}

/// The places of the pips of a die showing `value`, in a 3 × 3 grid from
/// -1 to 1.
fn pip_grid(value: u32) -> Vec<(f32, f32)> {
    match value {
        1 => vec![(0.0, 0.0)],
        2 => vec![(-1.0, -1.0), (1.0, 1.0)],
        3 => vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        4 => vec![(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)],
        5 => vec![
            (-1.0, -1.0),
            (-1.0, 1.0),
            (0.0, 0.0),
            (1.0, -1.0),
            (1.0, 1.0),
        ],
        _ => vec![
            (-1.0, -1.0),
            (-1.0, 0.0),
            (-1.0, 1.0),
            (1.0, -1.0),
            (1.0, 0.0),
            (1.0, 1.0),
        ],
    }
}

/// A round water table sunk into the shop floor: a wooden rim round the
/// water and a few bath toys floating on it.
fn spawn_water_table(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    table: &WaterTable,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut mesh = MeshBuilder::default();
    const SEGMENTS: usize = 64;
    // The rim stands where the ground reaches the water's edge (see the
    // shop's coast), a little over it.
    let radius = table.radius + 6.5;
    let at = |k: usize| {
        let angle = k as f32 / SEGMENTS as f32 * TAU;
        let point = table.center + Vec2::new(angle.cos(), angle.sin()) * radius;
        Vec3::new(point.x, terrain.height(point) + 1.4, point.y)
    };
    for k in 0..SEGMENTS {
        mesh.beam(at(k), at(k + 1), 2.6, 2.6, SHELF_WOOD);
    }
    // Toy boats and a duck on the water.
    for (index, (angle, color)) in [(0.3f32, PLASTIC[2]), (2.1, PLASTIC[0]), (4.4, PLASTIC[1])]
        .into_iter()
        .enumerate()
    {
        let point = table.center + Vec2::new(angle.cos(), angle.sin()) * (table.radius * 0.45);
        let center = Vec3::new(point.x, -0.25, point.y);
        let axes = [Vec3::X, Vec3::Y, Vec3::Z];
        box_at(&mut mesh, center, axes, Vec3::new(4.0, 1.2, 2.4), color);
        box_at(
            &mut mesh,
            center + Vec3::Y * 3.2,
            axes,
            Vec3::new(0.35, 2.8, 0.35),
            if index == 0 { FRAME } else { PLASTIC[5] },
        );
        if index == 0 {
            box_at(
                &mut mesh,
                center + Vec3::Y * 5.6,
                axes,
                Vec3::new(2.0, 1.3, 0.25),
                Color::srgb(0.98, 0.94, 0.82),
            );
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
}

/// A giant ball standing by the road.
fn spawn_ball(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    ball: &GiantBall,
    assets: &mut Assets3d,
) -> Footprint {
    let (center, _ahead, _out, _up) = beside(track, ball.along, ball.side, ball.lateral);
    let ground = terrain.height(center) + ball.radius;
    let mesh = assets.meshes.add(
        Sphere::new(ball.radius)
            .mesh()
            .ico(3)
            .expect("valid sphere"),
    );
    let material = assets.materials.add(StandardMaterial {
        base_color: ball.color,
        perceptual_roughness: 0.45,
        ..default()
    });
    world.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(center.x, ground, center.y),
    ));
    (center, ball.radius)
}
