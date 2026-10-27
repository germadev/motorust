//! The indoor scenery of a toy shop and hobby workshop (`Juguetería` /
//! `Toy Shop`, see [`super::scenery::ToyShop`]):
//! - An enclosing indoor room with warm yellow shop walls, red and blue dado
//!   stripes, wooden baseboards, floor-to-ceiling structural pillars, giant
//!   brand banners (`SCALEXTRIC`, `HOT WHEELS`, `PLAYMOBIL`, `BRIO`, `LEGO`),
//!   and an overhead industrial ceiling grid with glowing twin-tube fluorescent
//!   light fixtures and spotlights (no sky).
//! - A floor sector with a royal red runner carpet, rows of giant Nutcracker
//!   toy soldiers, painted hopscotch courts (`rayuela`), a giant `Big`-style
//!   floor piano, and giant wooden chairs and trestle table legs beside the
//!   climb ramps.
//! - A Scalextric / Ninco tabletop sector with twin metallic slot-car guide
//!   rails and copper braids embedded along the road, red slot-car barriers,
//!   arched Scalextric overpass bridges, and giant 1:32 slot cars.
//! - A Hot Wheels sector with bright orange side rails, blue track boosters,
//!   overhead 5-lane start/finish flag gates, a fiery tire tower, and a giant
//!   green T-Rex / Dragon head whose open jaws frame the vertical loop.
//! - An electric model railway beside the road with colourful steam locomotives
//!   and wagons animated every frame (`roll_toy_trains`), plus a wooden Brio
//!   helipad control tower.
//! - A narrow bridge lined with rainbow metallic xylophone bars, mallets, and
//!   giant studded LEGO bricks over the toy-boat pond.
//! - A grand Playmobil medieval castle gatehouse spanning over the road with
//!   twin crenellated grey towers, conical red roofs, portcullis arch, heraldic
//!   shields and Playmobil knights, alongside cardboard castle turrets and
//!   stacks of giant board-game boxes.
//! - Tall birch `Kallax`-style wooden cube shelves stocked with Matryoshka
//!   nesting dolls, wooden robots, spotted rocking horses and toy ships,
//!   leading into a steep bright yellow-and-blue playground slide (`tobogán`).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{light::NotShadowCaster, prelude::*};

use super::{
    Assets3d,
    quads::MeshBuilder,
    scenery::{
        CubeShelf, FloorPiano, HotWheelsGate, PlaymobilGate, ScalextricBridge, Side, SlideRun,
        SoldierRow, Span, ToyChair, ToyKind, ToyProp, ToyShop, ToyTrainLine, XylophoneSpan,
    },
    terrain::Terrain,
};
use crate::track::{SURFACE_Y, Track};

/// Height of the indoor toy-shop ceiling grid above sea level (m): high enough
/// to clear the `45.5 m` shelf road and the intro flyby camera.
const CEILING_Y: f32 = 115.0;
/// Floor level of the outer shop walls (m).
const FLOOR_WALL_Y: f32 = -18.0;
/// Enclosing room bounds `(min_x, max_x, min_z, max_z)` around the entire
/// circuit.
const ROOM_MIN_X: f32 = -520.0;
const ROOM_MAX_X: f32 = 920.0;
const ROOM_MIN_Z: f32 = -860.0;
const ROOM_MAX_Z: f32 = 340.0;

/// Speed of the electric toy trains along their railway lines (m/s).
const TRAIN_SPEED: f32 = 22.0;
/// Distance between railway sleepers (m) and gauge between the two rails (m).
const SLEEPER_STEP: f32 = 1.25;
const RAIL_GAUGE: f32 = 1.35;
/// Spacing between cars of one toy train (locomotive + 3 wagons, m).
const CAR_SPACING: f32 = 6.2;
const CARS_PER_TRAIN: u32 = 4;

/// Marks a locomotive or wagon of an animated electric toy train.
#[derive(Component)]
pub struct ToyTrainCar {
    span: Span,
    side: Side,
    lateral: f32,
    /// Offset along the train line at `t = 0` (m).
    phase: f32,
}

/// Spawns the indoor toy-shop room, ceiling lights, track sectors and giant
/// toy props when the circuit defines [`Scenery::toy_shop`].
pub fn spawn_toy_shop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    toy_shop: &ToyShop,
    assets: &mut Assets3d,
) {
    spawn_toy_room(world, assets);
    spawn_red_carpet(world, track, assets);
    spawn_scalextric_sector(world, track, toy_shop.scalextric, assets);
    spawn_hot_wheels_sector(world, track, toy_shop.hot_wheels, assets);
    spawn_dino_loop_jaws(world, track, toy_shop.dino_loop_along, assets);

    for gate in toy_shop.hot_wheels_gates {
        spawn_hot_wheels_gate(world, track, gate, assets);
    }
    for bridge in toy_shop.scalextric_bridges {
        spawn_scalextric_bridge(world, track, bridge, assets);
    }
    for gate in toy_shop.playmobil_gates {
        spawn_playmobil_gate(world, track, gate, assets);
    }
    for line in toy_shop.train_lines {
        spawn_train_line(world, track, terrain, line, assets);
    }
    for chair in toy_shop.chairs {
        spawn_toy_chair(world, track, terrain, chair, assets);
    }
    for shelf in toy_shop.cube_shelves {
        spawn_cube_shelf(world, track, terrain, shelf, assets);
    }
    for slide in toy_shop.slides {
        spawn_slide_run(world, track, slide, assets);
    }
    for xylophone in toy_shop.xylophones {
        spawn_xylophone_bridge(world, track, xylophone, assets);
    }
    for piano in toy_shop.floor_pianos {
        spawn_floor_piano(world, track, terrain, piano, assets);
    }
    for row in toy_shop.soldier_rows {
        spawn_soldier_row(world, track, terrain, row, assets);
    }
    spawn_toy_props(world, track, terrain, toy_shop.props, assets);
}

/// Moves the electric toy train locomotives and wagons along their railway
/// lines every frame.
pub fn roll_toy_trains(
    time: Res<Time>,
    track: Option<Res<Track>>,
    mut cars: Query<(&ToyTrainCar, &mut Transform)>,
) {
    let Some(track) = track else {
        return;
    };
    let lap = track.lap_length();
    let t = time.elapsed_secs();
    for (car, mut transform) in &mut cars {
        let len = car.span.length(lap).max(1.0);
        let progress = (car.phase + t * TRAIN_SPEED).rem_euclid(len);
        let along = (car.span.from + progress).rem_euclid(lap);
        let (pos, dir, _) = road_pose(track.as_ref(), along);
        let right = Vec2::new(-dir.y, dir.x);
        let spot = Vec2::new(pos.x, pos.z) + right * (car.side.sign() * car.lateral);

        // Smoothly sample ahead and behind along the railway line so the car
        // pitches with the slope of the table or floor.
        let ahead_along = (along + 2.5).rem_euclid(lap);
        let (ahead_pos, ahead_dir, _) = road_pose(track.as_ref(), ahead_along);
        let ahead_right = Vec2::new(-ahead_dir.y, ahead_dir.x);
        let ahead_spot =
            Vec2::new(ahead_pos.x, ahead_pos.z) + ahead_right * (car.side.sign() * car.lateral);

        let p0 = Vec3::new(spot.x, pos.y + 0.35, spot.y);
        let p1 = Vec3::new(ahead_spot.x, ahead_pos.y + 0.35, ahead_spot.y);
        let forward = (p1 - p0).normalize_or_zero();
        if forward.length_squared() > 0.5 {
            *transform = Transform::from_translation(p0).looking_to(forward, Vec3::Y);
        }
    }
}

/// Road surface position, horizontal forward unit vector, and heading at
/// `along` metres from the finish line, smoothly interpolated between
/// consecutive track samples.
fn road_pose(track: &Track, along: f32) -> (Vec3, Vec2, f32) {
    let lap = track.lap_length();
    let step = track.step_length();
    let exact = along.rem_euclid(lap) / step;
    let i0 = exact.floor() as usize;
    let t = exact.fract();
    let pos = track
        .surface_point(i0, 0.0)
        .lerp(track.surface_point(i0 + 1, 0.0), t);
    let (_, heading) = track.pose_at(i0);
    let dir = Vec2::new(heading.sin(), -heading.cos());
    (pos, dir, heading)
}

/// Spawns the enclosing indoor toy-shop walls, structural pillars, giant wall
/// banners, ceiling panels, steel roof trusses and glowing twin-tube
/// fluorescent light fixtures (`NotShadowCaster` so overhead directional light
/// still illuminates the room cleanly).
fn spawn_toy_room(world: &mut ChildSpawnerCommands, assets: &mut Assets3d) {
    let mut walls = MeshBuilder::default();
    let mut glow = MeshBuilder::default();

    let yellow_wall = Color::srgb(0.95, 0.80, 0.24);
    let wall_upper = Color::srgb(0.92, 0.76, 0.20);
    let baseboard = Color::srgb(0.46, 0.28, 0.14);
    let red_dado = Color::srgb(0.88, 0.16, 0.16);
    let blue_dado = Color::srgb(0.14, 0.38, 0.82);
    let pillar_col = Color::srgb(0.90, 0.72, 0.18);
    let ceiling_slate = Color::srgb(0.16, 0.18, 0.22);
    let truss_grey = Color::srgb(0.28, 0.31, 0.36);
    let fixture_housing = Color::srgb(0.82, 0.84, 0.88);
    let tube_white = Color::srgb(1.0, 0.98, 0.92);

    // 1. Four enclosing perimeter walls with baseboard, red/blue dado stripes,
    //    and giant colourful toy-brand murals.
    let corners = [
        (
            Vec2::new(ROOM_MIN_X, ROOM_MIN_Z),
            Vec2::new(ROOM_MAX_X, ROOM_MIN_Z),
            Vec3::Z,
        ),
        (
            Vec2::new(ROOM_MAX_X, ROOM_MIN_Z),
            Vec2::new(ROOM_MAX_X, ROOM_MAX_Z),
            Vec3::NEG_X,
        ),
        (
            Vec2::new(ROOM_MAX_X, ROOM_MAX_Z),
            Vec2::new(ROOM_MIN_X, ROOM_MAX_Z),
            Vec3::NEG_Z,
        ),
        (
            Vec2::new(ROOM_MIN_X, ROOM_MAX_Z),
            Vec2::new(ROOM_MIN_X, ROOM_MIN_Z),
            Vec3::X,
        ),
    ];

    let banner_colors = [
        Color::srgb(0.88, 0.15, 0.15), // Scalextric / Ninco red
        Color::srgb(0.98, 0.45, 0.08), // Hot Wheels orange
        Color::srgb(0.12, 0.48, 0.88), // Playmobil blue
        Color::srgb(0.18, 0.68, 0.30), // Brio green
    ];

    for (wall_idx, (a, b, inward)) in corners.into_iter().enumerate() {
        let bands = [
            (FLOOR_WALL_Y, 4.0, baseboard),
            (4.0, 9.5, red_dado),
            (9.5, 14.0, blue_dado),
            (14.0, 62.0, yellow_wall),
            (62.0, CEILING_Y, wall_upper),
        ];
        for (y0, y1, color) in bands {
            walls.quad_positions(
                [
                    Vec3::new(a.x, y0, a.y),
                    Vec3::new(b.x, y0, b.y),
                    Vec3::new(b.x, y1, b.y),
                    Vec3::new(a.x, y1, a.y),
                ],
                inward,
                color,
            );
        }

        // Giant toy-brand wall posters & display shelf ledges along each wall.
        let wall_len = a.distance(b);
        let dir = (b - a) / wall_len;
        let count = 6;
        for k in 0..count {
            let t0 = (k as f32 + 0.18) / count as f32;
            let t1 = (k as f32 + 0.82) / count as f32;
            let p0 = a + dir * (wall_len * t0) + Vec2::new(inward.x, inward.z) * 1.2;
            let p1 = a + dir * (wall_len * t1) + Vec2::new(inward.x, inward.z) * 1.2;
            let banner = banner_colors[(wall_idx + k) % banner_colors.len()];
            walls.quad_positions(
                [
                    Vec3::new(p0.x, 32.0, p0.y),
                    Vec3::new(p1.x, 32.0, p1.y),
                    Vec3::new(p1.x, 56.0, p1.y),
                    Vec3::new(p0.x, 56.0, p0.y),
                ],
                inward,
                banner,
            );
            // White inner stripe on the poster so it reads like a giant toy box
            // logo from across the room.
            let p0_in = a + dir * (wall_len * (t0 + 0.06)) + Vec2::new(inward.x, inward.z) * 1.6;
            let p1_in = a + dir * (wall_len * (t1 - 0.06)) + Vec2::new(inward.x, inward.z) * 1.6;
            walls.quad_positions(
                [
                    Vec3::new(p0_in.x, 39.0, p0_in.y),
                    Vec3::new(p1_in.x, 39.0, p1_in.y),
                    Vec3::new(p1_in.x, 49.0, p1_in.y),
                    Vec3::new(p0_in.x, 49.0, p0_in.y),
                ],
                inward,
                Color::srgb(0.98, 0.96, 0.88),
            );
        }
    }

    // 2. Structural yellow shop pillars along the perimeter walls (`photo 4`).
    for x in (-480..=880).step_by(170) {
        for z in [ROOM_MIN_Z + 8.0, ROOM_MAX_Z - 8.0] {
            walls.cuboid(
                Vec3::new(x as f32, (FLOOR_WALL_Y + CEILING_Y) * 0.5, z),
                Vec3::new(7.0, (CEILING_Y - FLOOR_WALL_Y) * 0.5, 7.0),
                pillar_col,
            );
        }
    }
    for z in (-800..=280).step_by(180) {
        for x in [ROOM_MIN_X + 8.0, ROOM_MAX_X - 8.0] {
            walls.cuboid(
                Vec3::new(x, (FLOOR_WALL_Y + CEILING_Y) * 0.5, z as f32),
                Vec3::new(7.0, (CEILING_Y - FLOOR_WALL_Y) * 0.5, 7.0),
                pillar_col,
            );
        }
    }

    // 3. Overhead dark ceiling deck at `CEILING_Y`.
    walls.quad_positions(
        [
            Vec3::new(ROOM_MIN_X, CEILING_Y, ROOM_MIN_Z),
            Vec3::new(ROOM_MAX_X, CEILING_Y, ROOM_MIN_Z),
            Vec3::new(ROOM_MAX_X, CEILING_Y, ROOM_MAX_Z),
            Vec3::new(ROOM_MIN_X, CEILING_Y, ROOM_MAX_Z),
        ],
        Vec3::NEG_Y,
        ceiling_slate,
    );

    // 4. Crisscross industrial steel ceiling trusses and twin-tube fluorescent
    //    light fixtures (`photo 4`).
    let truss_y = CEILING_Y - 2.2;
    for z in (-800..=280).step_by(90) {
        let zf = z as f32;
        walls.cuboid(
            Vec3::new((ROOM_MIN_X + ROOM_MAX_X) * 0.5, truss_y, zf),
            Vec3::new((ROOM_MAX_X - ROOM_MIN_X) * 0.5, 1.4, 1.4),
            truss_grey,
        );
    }
    for x in (-440..=840).step_by(110) {
        let xf = x as f32;
        walls.cuboid(
            Vec3::new(xf, truss_y, (ROOM_MIN_Z + ROOM_MAX_Z) * 0.5),
            Vec3::new(1.2, 1.2, (ROOM_MAX_Z - ROOM_MIN_Z) * 0.5),
            truss_grey,
        );
        // Twin-tube fluorescent light banks suspended along each bay.
        for z in (-760..=240).step_by(85) {
            let zf = z as f32;
            let light_y = CEILING_Y - 4.0;
            // Reflective white sheet-metal fixture hood.
            walls.cuboid(
                Vec3::new(xf, light_y + 0.6, zf),
                Vec3::new(4.2, 0.45, 18.0),
                fixture_housing,
            );
            // Twin glowing T8 fluorescent tubes facing downward.
            for dx in [-1.6, 1.6] {
                glow.quad_positions(
                    [
                        Vec3::new(xf + dx - 0.55, light_y, zf - 16.5),
                        Vec3::new(xf + dx + 0.55, light_y, zf - 16.5),
                        Vec3::new(xf + dx + 0.55, light_y, zf + 16.5),
                        Vec3::new(xf + dx - 0.55, light_y, zf + 16.5),
                    ],
                    Vec3::NEG_Y,
                    tube_white,
                );
            }
        }
    }

    let wall_mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.78,
        reflectance: 0.22,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let glow_mat = assets.materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(6.5, 6.2, 5.4),
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    world.spawn((
        Mesh3d(assets.meshes.add(walls.build())),
        MeshMaterial3d(wall_mat),
        NotShadowCaster,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(glow.build())),
        MeshMaterial3d(glow_mat),
        NotShadowCaster,
    ));
}

/// Spawns a royal red runner carpet with gold trim along the start/finish
/// straight (`photo 2`).
fn spawn_red_carpet(world: &mut ChildSpawnerCommands, track: &Track, assets: &mut Assets3d) {
    let mut builder = MeshBuilder::default();
    let red = Color::srgb(0.78, 0.10, 0.14);
    let gold = Color::srgb(0.95, 0.78, 0.22);
    let lap = track.lap_length();

    for (from, to) in [(0.0, 105.0), ((lap - 110.0).max(105.0), lap)] {
        let steps = ((to - from) / 4.0).ceil() as usize;
        for k in 0..steps {
            let a0 = from + (to - from) * (k as f32 / steps as f32);
            let a1 = from + (to - from) * ((k + 1) as f32 / steps as f32);
            let i0 = track.index_at(a0.rem_euclid(lap));
            let i1 = track.index_at(a1.rem_euclid(lap));
            if i0 == i1 {
                continue;
            }
            for side in [-1.0, 1.0] {
                let inner = track.half_width_at(i0) + 0.8;
                let outer = inner + 3.2;
                let gold_in = inner + 0.35;
                let gold_out = outer - 0.35;
                let lift = Vec3::Y * (SURFACE_Y + 0.035);
                builder.quad_positions(
                    [
                        track.surface_point(i0, side * inner) + lift,
                        track.surface_point(i1, side * inner) + lift,
                        track.surface_point(i1, side * outer) + lift,
                        track.surface_point(i0, side * outer) + lift,
                    ],
                    Vec3::Y,
                    red,
                );
                let gold_lift = Vec3::Y * (SURFACE_Y + 0.042);
                builder.quad_positions(
                    [
                        track.surface_point(i0, side * gold_in) + gold_lift,
                        track.surface_point(i1, side * gold_in) + gold_lift,
                        track.surface_point(i1, side * (gold_in + 0.25)) + gold_lift,
                        track.surface_point(i0, side * (gold_in + 0.25)) + gold_lift,
                    ],
                    Vec3::Y,
                    gold,
                );
                builder.quad_positions(
                    [
                        track.surface_point(i0, side * (gold_out - 0.25)) + gold_lift,
                        track.surface_point(i1, side * (gold_out - 0.25)) + gold_lift,
                        track.surface_point(i1, side * gold_out) + gold_lift,
                        track.surface_point(i0, side * gold_out) + gold_lift,
                    ],
                    Vec3::Y,
                    gold,
                );
            }
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.88,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(builder.build())),
        MeshMaterial3d(mat),
    ));
}

/// Spawns twin metallic Scalextric / Ninco slot-car guide rails and braided
/// copper contact strips embedded along the road surface (`photo 4`), plus
/// red-and-white slot-car side barriers.
fn spawn_scalextric_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    span: Span,
    assets: &mut Assets3d,
) {
    let mut metal = MeshBuilder::default();
    let mut barriers = MeshBuilder::default();
    let lap = track.lap_length();
    let len = span.length(lap);
    let step = 3.0;
    let steps = (len / step).ceil() as usize;

    let copper = Color::srgb(0.86, 0.56, 0.32);
    let slot_dark = Color::srgb(0.10, 0.11, 0.13);
    let red_barrier = Color::srgb(0.88, 0.14, 0.16);
    let white_barrier = Color::srgb(0.95, 0.95, 0.96);

    for k in 0..steps {
        let a0 = (span.from + k as f32 * step).rem_euclid(lap);
        let a1 = (span.from + ((k + 1) as f32 * step).min(len)).rem_euclid(lap);
        let i0 = track.index_at(a0);
        let i1 = track.index_at(a1);
        if i0 == i1 {
            continue;
        }
        let lift = Vec3::Y * (SURFACE_Y + 0.018);

        // Two parallel slot-car lanes at lateral `-2.6 m` and `+2.6 m`.
        for lane in [-2.6f32, 2.6f32] {
            // Central black guide slot.
            metal.quad_positions(
                [
                    track.surface_point(i0, lane - 0.08) + lift,
                    track.surface_point(i1, lane - 0.08) + lift,
                    track.surface_point(i1, lane + 0.08) + lift,
                    track.surface_point(i0, lane + 0.08) + lift,
                ],
                Vec3::Y,
                slot_dark,
            );
            // Braided copper contact strips on either side of the guide slot.
            for braid in [-0.22f32, 0.22f32] {
                metal.quad_positions(
                    [
                        track.surface_point(i0, lane + braid - 0.09) + lift,
                        track.surface_point(i1, lane + braid - 0.09) + lift,
                        track.surface_point(i1, lane + braid + 0.09) + lift,
                        track.surface_point(i0, lane + braid + 0.09) + lift,
                    ],
                    Vec3::Y,
                    copper,
                );
            }
        }

        // Red & white clip-on Scalextric guardrails along the road shoulders.
        let color = if k % 2 == 0 {
            red_barrier
        } else {
            white_barrier
        };
        for side in [-1.0f32, 1.0f32] {
            let lat = side * (track.half_width_at(i0) + 1.35);
            let p0 = track.surface_point(i0, lat);
            let p1 = track.surface_point(i1, lat);
            barriers.beam(p0 + Vec3::Y * 0.55, p1 + Vec3::Y * 0.55, 0.22, 0.85, color);
        }
    }

    let metal_mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.32,
        metallic: 0.78,
        ..default()
    });
    let barrier_mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.42,
        reflectance: 0.5,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(metal.build())),
        MeshMaterial3d(metal_mat),
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(barriers.build())),
        MeshMaterial3d(barrier_mat),
    ));
}

/// Spawns bright orange Hot Wheels raised track side-rails, blue track
/// connector tongues, and twin-wheel motorized track boosters along `span`.
fn spawn_hot_wheels_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    span: Span,
    assets: &mut Assets3d,
) {
    let mut builder = MeshBuilder::default();
    let lap = track.lap_length();
    let len = span.length(lap);
    let step = 3.0;
    let steps = (len / step).ceil() as usize;

    let hw_orange = Color::srgb(0.99, 0.42, 0.04);
    let hw_yellow = Color::srgb(0.99, 0.78, 0.08);
    let hw_blue = Color::srgb(0.08, 0.42, 0.90);
    let wheel_green = Color::srgb(0.45, 0.88, 0.16);

    for k in 0..steps {
        let a0 = (span.from + k as f32 * step).rem_euclid(lap);
        let a1 = (span.from + ((k + 1) as f32 * step).min(len)).rem_euclid(lap);
        let i0 = track.index_at(a0);
        let i1 = track.index_at(a1);
        if i0 == i1 {
            continue;
        }

        // Orange centre stripe along the road surface.
        let lift = Vec3::Y * (SURFACE_Y + 0.016);
        builder.quad_positions(
            [
                track.surface_point(i0, -1.2) + lift,
                track.surface_point(i1, -1.2) + lift,
                track.surface_point(i1, 1.2) + lift,
                track.surface_point(i0, 1.2) + lift,
            ],
            Vec3::Y,
            hw_orange,
        );
        builder.quad_positions(
            [
                track.surface_point(i0, -0.35) + lift + Vec3::Y * 0.004,
                track.surface_point(i1, -0.35) + lift + Vec3::Y * 0.004,
                track.surface_point(i1, 0.35) + lift + Vec3::Y * 0.004,
                track.surface_point(i0, 0.35) + lift + Vec3::Y * 0.004,
            ],
            Vec3::Y,
            hw_yellow,
        );

        // Raised orange Hot Wheels side lips and periodic blue track connectors.
        for side in [-1.0f32, 1.0f32] {
            let lat = side * (track.half_width_at(i0) + 1.05);
            let p0 = track.surface_point(i0, lat);
            let p1 = track.surface_point(i1, lat);
            builder.beam(
                p0 + Vec3::Y * 0.65,
                p1 + Vec3::Y * 0.65,
                0.45,
                1.1,
                hw_orange,
            );
            if k % 6 == 0 {
                builder.cuboid(
                    (p0 + p1) * 0.5 + Vec3::Y * 0.35,
                    Vec3::new(0.85, 0.45, 0.85),
                    hw_blue,
                );
            }
            // Motorized blue & lime-green Hot Wheels booster box every 18 steps.
            if k % 18 == 4 {
                let booster_at = track.surface_point(i0, side * (track.half_width_at(i0) + 2.6));
                builder.cuboid(
                    booster_at + Vec3::Y * 1.1,
                    Vec3::new(1.8, 1.1, 2.6),
                    hw_blue,
                );
                builder.cylinder(booster_at + Vec3::Y * 1.4, 1.3, 1.2, 12, wheel_green);
            }
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.36,
        reflectance: 0.58,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(builder.build())),
        MeshMaterial3d(mat),
    ));
}

/// Spawns a giant green T-Rex / Dragon head with wide-open jaws, sharp white
/// teeth, glowing yellow eyes and dorsal spikes framing the entry to the
/// vertical loop (`photo 5`).
fn spawn_dino_loop_jaws(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    along: f32,
    assets: &mut Assets3d,
) {
    let (pos, _, heading) = road_pose(track, along);
    let mut b = MeshBuilder::default();

    let scale_green = Color::srgb(0.18, 0.66, 0.24);
    let dark_green = Color::srgb(0.10, 0.44, 0.16);
    let jaw_red = Color::srgb(0.78, 0.18, 0.22);
    let tooth_white = Color::srgb(0.97, 0.96, 0.92);
    let eye_yellow = Color::srgb(0.98, 0.86, 0.12);
    let horn_orange = Color::srgb(0.96, 0.48, 0.10);

    // Side muscular jaw pillars flanking the road (`x = ±10.5 m`).
    for side in [-1.0f32, 1.0f32] {
        let sx = side * 10.5;
        b.cuboid(
            Vec3::new(sx, 5.5, 0.0),
            Vec3::new(2.6, 5.8, 5.2),
            scale_green,
        );
        b.cuboid(
            Vec3::new(sx * 0.92, 5.2, 0.4),
            Vec3::new(1.2, 4.8, 4.2),
            jaw_red,
        );
        // Cheek spikes.
        b.cone(
            Vec3::new(sx + side * 2.2, 4.5, 1.0),
            1.2,
            2.8,
            8,
            horn_orange,
        );
        b.cone(
            Vec3::new(sx + side * 2.2, 7.8, -1.0),
            1.2,
            2.8,
            8,
            horn_orange,
        );
    }

    // Upper snout & skull arch high over the road (`y = 11.2..16.5 m`).
    b.cuboid(
        Vec3::new(0.0, 12.6, 0.5),
        Vec3::new(11.8, 2.4, 5.8),
        scale_green,
    );
    b.cuboid(
        Vec3::new(0.0, 15.2, -0.8),
        Vec3::new(8.8, 1.8, 4.6),
        dark_green,
    );
    // Upper palate inside the mouth.
    b.cuboid(
        Vec3::new(0.0, 10.4, 0.8),
        Vec3::new(9.8, 0.45, 4.8),
        jaw_red,
    );

    // Brow ridges and fierce yellow reptilian eyes with slit pupils.
    for side in [-1.0f32, 1.0f32] {
        let ex = side * 7.2;
        b.cuboid(
            Vec3::new(ex, 15.8, 2.2),
            Vec3::new(2.2, 1.1, 2.4),
            dark_green,
        );
        b.icosphere(Vec3::new(ex, 14.4, 3.6), 1.35, 1, eye_yellow);
        b.cuboid(
            Vec3::new(ex, 14.4, 4.85),
            Vec3::new(0.28, 1.1, 0.25),
            Color::srgb(0.05, 0.05, 0.05),
        );
        // Nostril bumps at the front of the snout.
        b.cuboid(
            Vec3::new(side * 4.2, 13.8, 5.2),
            Vec3::new(1.1, 0.8, 1.1),
            dark_green,
        );
        // Brow horns.
        b.cone(Vec3::new(ex, 16.8, 1.2), 1.3, 3.8, 8, horn_orange);
    }

    // Dorsal crest spikes along the top of the head.
    for z in [-3.2f32, -0.6, 2.0] {
        b.cone(Vec3::new(0.0, 16.8, z), 1.4, 3.6, 8, horn_orange);
    }

    // Row of giant sharp white upper teeth hanging down over the road and
    // lower teeth rising from the jaw edges (`photo 5`).
    for k in -4..=4 {
        let tx = k as f32 * 2.1;
        // Upper fangs pointing down (built by flipping a cone via `beam` / pyramid).
        let tip = Vec3::new(tx, 8.2, 4.2);
        let base = Vec3::new(tx, 10.4, 4.2);
        b.beam(base, tip, 0.85, 0.85, tooth_white);
    }
    for side in [-1.0f32, 1.0f32] {
        for k in 0..3 {
            let tx = side * (7.8 + k as f32 * 0.9);
            let tz = 3.8 - k as f32 * 2.2;
            b.cone(Vec3::new(tx, 0.2, tz), 0.75, 2.8, 6, tooth_white);
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.45,
        reflectance: 0.48,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(b.build())),
        MeshMaterial3d(mat),
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(-heading)),
    ));
}

/// Spawns an overhead Hot Wheels 5-lane start/finish gate with a blue plastic
/// frame, flame header, and 5 numbered yellow hanging lane paddles (inspired
/// by the uploaded Hot Wheels 5-lane raceway photo).
fn spawn_hot_wheels_gate(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    gate: &HotWheelsGate,
    assets: &mut Assets3d,
) {
    let (pos, _, heading) = road_pose(track, gate.along);
    let mut b = MeshBuilder::default();

    let hw_blue = Color::srgb(0.08, 0.40, 0.88);
    let hw_orange = Color::srgb(0.99, 0.42, 0.04);
    let hw_red = Color::srgb(0.90, 0.14, 0.14);
    let flag_yellow = Color::srgb(0.99, 0.82, 0.10);
    let dark = Color::srgb(0.12, 0.14, 0.18);

    // Side blue plastic support towers with orange base clamps.
    for side in [-1.0f32, 1.0f32] {
        let x = side * 10.2;
        b.cuboid(Vec3::new(x, 0.8, 0.0), Vec3::new(1.5, 0.8, 2.2), hw_orange);
        b.cuboid(Vec3::new(x, 5.4, 0.0), Vec3::new(0.95, 5.0, 1.3), hw_blue);
        // Diagonal side struts.
        b.beam(
            Vec3::new(x, 8.2, 0.0),
            Vec3::new(x - side * 2.2, 9.8, 0.0),
            0.55,
            0.55,
            hw_blue,
        );
    }

    // Overhead blue crossbeam & flame-red/orange Hot Wheels header board.
    b.cuboid(
        Vec3::new(0.0, 9.8, 0.0),
        Vec3::new(11.2, 0.85, 1.4),
        hw_blue,
    );
    b.cuboid(Vec3::new(0.0, 11.2, 0.0), Vec3::new(8.6, 1.1, 0.65), hw_red);
    b.cuboid(
        Vec3::new(0.0, 11.2, 0.72),
        Vec3::new(7.4, 0.65, 0.12),
        hw_orange,
    );
    b.cuboid(
        Vec3::new(0.0, 11.2, -0.72),
        Vec3::new(7.4, 0.65, 0.12),
        hw_orange,
    );

    // Five hanging yellow lane paddles (`1..5`) below the crossbeam, high
    // enough (`y = 6.9..9.0 m`) for bikes to race cleanly underneath.
    for lane in -2..=2 {
        let lx = lane as f32 * 3.4;
        b.cuboid(
            Vec3::new(lx, 8.0, 0.0),
            Vec3::new(0.95, 1.05, 0.22),
            flag_yellow,
        );
        // Dark number badge in the centre of each paddle.
        b.cuboid(Vec3::new(lx, 7.9, 0.25), Vec3::new(0.38, 0.55, 0.06), dark);
        b.cuboid(Vec3::new(lx, 7.9, -0.25), Vec3::new(0.38, 0.55, 0.06), dark);
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.38,
        reflectance: 0.52,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(b.build())),
        MeshMaterial3d(mat),
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(-heading)),
    ));
}

/// Spawns an arched grey plastic Scalextric / Ninco overpass bridge spanning
/// across the road (`photo 4`).
fn spawn_scalextric_bridge(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    bridge: &ScalextricBridge,
    assets: &mut Assets3d,
) {
    let (pos, _, heading) = road_pose(track, bridge.along);
    let mut b = MeshBuilder::default();

    let plastic_grey = Color::srgb(0.68, 0.70, 0.74);
    let dark_grey = Color::srgb(0.36, 0.38, 0.42);
    let ninco_red = Color::srgb(0.86, 0.14, 0.16);
    let banner_white = Color::srgb(0.96, 0.96, 0.96);

    for side in [-1.0f32, 1.0f32] {
        let x = side * 10.6;
        // Stepped plastic pier feet.
        b.cuboid(
            Vec3::new(x, 4.6, 0.0),
            Vec3::new(1.2, 4.6, 2.8),
            plastic_grey,
        );
        b.cuboid(Vec3::new(x, 9.4, 0.0), Vec3::new(1.3, 1.2, 3.2), dark_grey);
    }

    // Arched overpass deck and red/white Scalextric side parapets.
    b.cuboid(
        Vec3::new(0.0, 9.2, 0.0),
        Vec3::new(11.4, 0.45, 2.8),
        dark_grey,
    );
    for z in [-2.6f32, 2.6f32] {
        b.cuboid(
            Vec3::new(0.0, 10.3, z),
            Vec3::new(11.4, 1.1, 0.25),
            ninco_red,
        );
        b.cuboid(
            Vec3::new(0.0, 10.3, z * 1.08),
            Vec3::new(8.5, 0.65, 0.08),
            banner_white,
        );
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.48,
        reflectance: 0.42,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(b.build())),
        MeshMaterial3d(mat),
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(-heading)),
    ));
}

/// Spawns a Playmobil medieval castle gatehouse ("Puerta de castillo como de
/// Playmobil") spanning over the road, with twin crenellated grey stone
/// towers, red conical plastic roofs, yellow flags, a raised timber portcullis
/// arch, royal lion shields, and giant Playmobil knights standing guard.
fn spawn_playmobil_gate(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    gate: &PlaymobilGate,
    assets: &mut Assets3d,
) {
    let (pos, _, heading) = road_pose(track, gate.along);
    let mut b = MeshBuilder::default();

    let stone_grey = Color::srgb(0.66, 0.68, 0.72);
    let stone_light = Color::srgb(0.76, 0.78, 0.82);
    let stone_dark = Color::srgb(0.46, 0.48, 0.52);
    let roof_red = Color::srgb(0.84, 0.16, 0.16);
    let timber_brown = Color::srgb(0.42, 0.25, 0.12);
    let portcullis_iron = Color::srgb(0.22, 0.23, 0.26);
    let gold = Color::srgb(0.96, 0.78, 0.18);
    let royal_blue = Color::srgb(0.14, 0.38, 0.84);

    // 1. Twin crenellated Playmobil stone towers flanking the road (`x = ±11.6 m`).
    for side in [-1.0f32, 1.0f32] {
        let tx = side * 11.6;
        // Main square stone tower shaft with Playmobil dovetail corner blocks.
        b.cuboid(
            Vec3::new(tx, 7.2, 0.0),
            Vec3::new(3.2, 7.2, 3.2),
            stone_grey,
        );
        b.cuboid(
            Vec3::new(tx, 1.2, 0.0),
            Vec3::new(3.5, 1.2, 3.5),
            stone_dark,
        );
        // Narrow dark arrow slits on the front and back faces.
        for z in [-3.25f32, 3.25f32] {
            b.cuboid(
                Vec3::new(tx, 5.5, z),
                Vec3::new(0.28, 1.4, 0.08),
                stone_dark,
            );
            b.cuboid(
                Vec3::new(tx, 9.8, z),
                Vec3::new(0.32, 1.4, 0.08),
                stone_dark,
            );
        }
        // Corbelled stone parapet platform at `y = 14.4 m`.
        b.cuboid(
            Vec3::new(tx, 14.6, 0.0),
            Vec3::new(3.7, 0.6, 3.7),
            stone_light,
        );
        // Battlement merlons around the tower top.
        for mx in [-2.8f32, 0.0, 2.8] {
            for mz in [-3.2f32, 3.2] {
                b.cuboid(
                    Vec3::new(tx + mx, 15.65, mz),
                    Vec3::new(0.65, 0.65, 0.45),
                    stone_light,
                );
            }
        }
        // Classic Playmobil red conical plastic turret roof + yellow flag finial.
        b.cone(Vec3::new(tx, 15.2, 0.0), 4.1, 6.8, 12, roof_red);
        b.cylinder(Vec3::new(tx, 23.0, 0.0), 0.14, 2.2, 8, timber_brown);
        b.cuboid(
            Vec3::new(tx + 0.9, 23.4, 0.0),
            Vec3::new(0.9, 0.48, 0.08),
            gold,
        );

        // Giant Playmobil Knight standing guard at the foot of each tower!
        append_playmobil_knight(
            &mut b,
            Vec3::new(tx - side * 1.2, 0.0, 4.2),
            if side < 0.0 { royal_blue } else { roof_red },
        );
    }

    // 2. Central gatehouse wall spanning over the road (`y = 8.4..13.6 m`).
    b.cuboid(
        Vec3::new(0.0, 10.8, 0.0),
        Vec3::new(8.8, 2.4, 2.4),
        stone_grey,
    );
    // Timber horde / gallery band along the upper gatehouse.
    b.cuboid(
        Vec3::new(0.0, 12.6, 0.0),
        Vec3::new(9.0, 0.65, 2.65),
        timber_brown,
    );
    // Crenellated stone merlons along the top of the gatehouse bridge.
    for k in -4..=4 {
        let mx = k as f32 * 1.95;
        for mz in [-2.2f32, 2.2] {
            b.cuboid(
                Vec3::new(mx, 13.9, mz),
                Vec3::new(0.65, 0.75, 0.42),
                stone_light,
            );
        }
    }

    // 3. Raised Playmobil portcullis grate (`y = 6.9..9.0 m`) hanging under the
    //    stone arch, with vertical iron bars and spikes clear of the bikes.
    for k in -5..=5 {
        let gx = k as f32 * 1.35;
        b.cuboid(
            Vec3::new(gx, 8.0, 0.0),
            Vec3::new(0.14, 1.05, 0.14),
            portcullis_iron,
        );
    }
    for gy in [7.2f32, 8.1] {
        b.cuboid(
            Vec3::new(0.0, gy, 0.0),
            Vec3::new(7.2, 0.14, 0.16),
            portcullis_iron,
        );
    }

    // 4. Royal heraldic shields (blue & gold lion crests) on both faces of the
    //    gatehouse arch.
    for z in [-2.52f32, 2.52f32] {
        for sx in [-4.2f32, 0.0, 4.2] {
            b.cuboid(
                Vec3::new(sx, 10.8, z),
                Vec3::new(0.95, 1.25, 0.12),
                royal_blue,
            );
            b.cuboid(
                Vec3::new(sx, 10.8, z * 1.03),
                Vec3::new(0.55, 0.78, 0.08),
                gold,
            );
        }
    }

    // 5. Playmobil Knight captain standing on top of the gatehouse walkway!
    append_playmobil_knight(&mut b, Vec3::new(0.0, 13.25, 0.0), royal_blue);

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.44,
        reflectance: 0.48,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(b.build())),
        MeshMaterial3d(mat),
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(-heading)),
    ));
}

/// Appends a chunky, recognizable Playmobil medieval knight figure (cylindrical
/// smiling head, conical helmet with visor, torso tabard, articulated blocky
/// legs, shield and halberd) at local offset `base`.
fn append_playmobil_knight(b: &mut MeshBuilder, base: Vec3, tunic: Color) {
    let skin_yellow = Color::srgb(0.98, 0.82, 0.42);
    let silver = Color::srgb(0.76, 0.80, 0.86);
    let dark_iron = Color::srgb(0.28, 0.30, 0.35);
    let gold = Color::srgb(0.96, 0.78, 0.18);
    let wood = Color::srgb(0.45, 0.28, 0.14);

    // Legs & boots.
    for lx in [-0.45f32, 0.45f32] {
        b.cuboid(
            base + Vec3::new(lx, 0.9, 0.0),
            Vec3::new(0.36, 0.9, 0.38),
            silver,
        );
    }
    // Tunic torso & gold belt crest.
    b.cuboid(
        base + Vec3::new(0.0, 2.5, 0.0),
        Vec3::new(0.95, 0.95, 0.58),
        tunic,
    );
    b.cuboid(
        base + Vec3::new(0.0, 1.85, 0.0),
        Vec3::new(0.98, 0.18, 0.62),
        gold,
    );
    b.cuboid(
        base + Vec3::new(0.0, 2.6, 0.60),
        Vec3::new(0.42, 0.48, 0.06),
        gold,
    );

    // Arms.
    for side in [-1.0f32, 1.0f32] {
        b.cuboid(
            base + Vec3::new(side * 1.25, 2.45, 0.15),
            Vec3::new(0.28, 0.75, 0.32),
            tunic,
        );
    }
    // Iconic cylindrical Playmobil head + knight helmet + feather plume.
    b.cylinder(base + Vec3::new(0.0, 3.9, 0.0), 0.68, 0.95, 12, skin_yellow);
    b.cylinder(base + Vec3::new(0.0, 4.35, 0.0), 0.78, 0.75, 12, silver);
    b.cone(base + Vec3::new(0.0, 4.72, 0.0), 0.82, 0.85, 12, silver);
    b.cuboid(
        base + Vec3::new(0.0, 5.75, -0.15),
        Vec3::new(0.18, 0.55, 0.45),
        gold,
    );

    // Shield on left arm & tall halberd in right hand.
    b.cuboid(
        base + Vec3::new(-1.55, 2.4, 0.55),
        Vec3::new(0.12, 0.95, 0.72),
        tunic,
    );
    b.cuboid(
        base + Vec3::new(-1.68, 2.4, 0.55),
        Vec3::new(0.06, 0.55, 0.42),
        gold,
    );
    b.cylinder(base + Vec3::new(1.55, 3.2, 0.45), 0.10, 6.2, 8, wood);
    b.cuboid(
        base + Vec3::new(1.55, 5.8, 0.65),
        Vec3::new(0.08, 0.65, 0.45),
        dark_iron,
    );
}

/// Spawns a model railway line (`ToyTrainLine`) beside the road with wooden
/// sleepers, twin steel rails, and animated colourful steam locomotives +
/// wagons (`ToyTrainCar`).
fn spawn_train_line(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    _terrain: &Terrain,
    line: &ToyTrainLine,
    assets: &mut Assets3d,
) {
    let mut rails = MeshBuilder::default();
    let lap = track.lap_length();
    let len = line.span.length(lap);
    let steps = (len / SLEEPER_STEP).ceil() as usize;

    let sleeper_wood = Color::srgb(0.44, 0.28, 0.15);
    let steel_rail = Color::srgb(0.74, 0.78, 0.84);

    for k in 0..steps {
        let a0 = (line.span.from + k as f32 * SLEEPER_STEP).rem_euclid(lap);
        let a1 = (line.span.from + ((k + 1) as f32 * SLEEPER_STEP).min(len)).rem_euclid(lap);
        let (p0, d0, _) = road_pose(track, a0);
        let (p1, d1, _) = road_pose(track, a1);
        let r0 = Vec2::new(-d0.y, d0.x);
        let r1 = Vec2::new(-d1.y, d1.x);

        let c0 = Vec3::new(p0.x, p0.y + 0.08, p0.z)
            + Vec3::new(r0.x, 0.0, r0.y) * (line.side.sign() * line.lateral);
        let c1 = Vec3::new(p1.x, p1.y + 0.08, p1.z)
            + Vec3::new(r1.x, 0.0, r1.y) * (line.side.sign() * line.lateral);

        // Wooden cross-tie sleeper.
        let right3 = Vec3::new(r0.x, 0.0, r0.y);
        rails.beam(
            c0 - right3 * (RAIL_GAUGE * 0.85),
            c0 + right3 * (RAIL_GAUGE * 0.85),
            0.28,
            0.14,
            sleeper_wood,
        );

        // Twin steel rails.
        for rail_offset in [-RAIL_GAUGE * 0.5, RAIL_GAUGE * 0.5] {
            rails.beam(
                c0 + right3 * rail_offset + Vec3::Y * 0.14,
                c1 + Vec3::new(r1.x, 0.0, r1.y) * rail_offset + Vec3::Y * 0.14,
                0.12,
                0.16,
                steel_rail,
            );
        }
    }

    let rail_mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.45,
        metallic: 0.35,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(rails.build())),
        MeshMaterial3d(rail_mat),
    ));

    // Shared meshes for the steam locomotive and colourful wagons.
    let loco_mesh = assets.meshes.add(build_toy_locomotive_mesh());
    let wagon_meshes = [
        assets
            .meshes
            .add(build_toy_wagon_mesh(Color::srgb(0.16, 0.48, 0.88))),
        assets
            .meshes
            .add(build_toy_wagon_mesh(Color::srgb(0.96, 0.78, 0.14))),
        assets
            .meshes
            .add(build_toy_wagon_mesh(Color::srgb(0.20, 0.72, 0.32))),
    ];
    let train_mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.36,
        reflectance: 0.55,
        ..default()
    });

    for t_idx in 0..line.trains.max(1) {
        let train_base = (t_idx as f32 / line.trains.max(1) as f32) * len;
        for car_idx in 0..CARS_PER_TRAIN {
            let phase = (train_base - car_idx as f32 * CAR_SPACING).rem_euclid(len);
            let mesh = if car_idx == 0 {
                loco_mesh.clone()
            } else {
                wagon_meshes[(car_idx as usize - 1) % wagon_meshes.len()].clone()
            };
            world.spawn((
                ToyTrainCar {
                    span: line.span,
                    side: line.side,
                    lateral: line.lateral,
                    phase,
                },
                Mesh3d(mesh),
                MeshMaterial3d(train_mat.clone()),
                Transform::default(),
            ));
        }
    }
}

/// Builds a colourful toy steam locomotive mesh with a red cylindrical boiler,
/// black smokestack with white cotton steam puffs, yellow cab, gold cowcatcher
/// and bright red wheels.
fn build_toy_locomotive_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let red = Color::srgb(0.88, 0.15, 0.16);
    let yellow = Color::srgb(0.98, 0.80, 0.14);
    let blue = Color::srgb(0.14, 0.44, 0.86);
    let black = Color::srgb(0.14, 0.15, 0.18);
    let steam = Color::srgb(0.96, 0.97, 0.99);

    // Chassis & running board (`-Z` is forward for `Transform::looking_to`).
    b.cuboid(
        Vec3::new(0.0, 0.65, 0.0),
        Vec3::new(1.05, 0.28, 2.45),
        black,
    );
    // Red cylindrical boiler along `-Z`.
    b.cuboid(
        Vec3::new(0.0, 1.45, -0.55),
        Vec3::new(0.85, 0.78, 1.45),
        red,
    );
    // Yellow engineer cab at the back (`+Z`).
    b.cuboid(
        Vec3::new(0.0, 1.95, 1.35),
        Vec3::new(1.02, 1.15, 0.92),
        yellow,
    );
    b.cuboid(
        Vec3::new(0.0, 3.15, 1.35),
        Vec3::new(1.18, 0.18, 1.08),
        blue,
    );
    // Smokestack & white steam puffs at the front (`-Z`).
    b.cylinder(Vec3::new(0.0, 2.65, -1.35), 0.32, 1.1, 10, black);
    b.icosphere(Vec3::new(0.0, 3.55, -1.15), 0.48, 1, steam);
    b.icosphere(Vec3::new(0.0, 4.15, -0.65), 0.62, 1, steam);
    // Front gold headlamp & cowcatcher.
    b.cuboid(
        Vec3::new(0.0, 0.55, -2.55),
        Vec3::new(0.95, 0.35, 0.35),
        yellow,
    );

    // Six red toy train wheels.
    for z in [-1.45f32, 0.0, 1.45] {
        for x in [-1.08f32, 1.08f32] {
            b.cuboid(Vec3::new(x, 0.48, z), Vec3::new(0.14, 0.48, 0.48), red);
        }
    }
    b.build()
}

/// Builds a colourful toy railway wagon mesh filled with wooden blocks.
fn build_toy_wagon_mesh(body_color: Color) -> Mesh {
    let mut b = MeshBuilder::default();
    let black = Color::srgb(0.15, 0.16, 0.20);
    let wheel_red = Color::srgb(0.86, 0.16, 0.16);
    let cargo_gold = Color::srgb(0.95, 0.74, 0.20);

    b.cuboid(Vec3::new(0.0, 0.6, 0.0), Vec3::new(0.98, 0.22, 2.1), black);
    b.cuboid(
        Vec3::new(0.0, 1.45, 0.0),
        Vec3::new(0.95, 0.72, 1.95),
        body_color,
    );
    b.cuboid(
        Vec3::new(0.0, 2.25, 0.0),
        Vec3::new(0.72, 0.38, 1.45),
        cargo_gold,
    );
    for z in [-1.2f32, 1.2] {
        for x in [-1.02f32, 1.02f32] {
            b.cuboid(
                Vec3::new(x, 0.45, z),
                Vec3::new(0.12, 0.45, 0.45),
                wheel_red,
            );
        }
    }
    b.build()
}

/// Spawns a giant wooden chair (`ToyChair`) beside the road where the bikes
/// climb from the floor to the chair seat and up onto the tabletop.
fn spawn_toy_chair(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    chair: &ToyChair,
    assets: &mut Assets3d,
) {
    let (pos, dir, heading) = road_pose(track, chair.along);
    let right = Vec2::new(-dir.y, dir.x);
    let spot = Vec2::new(pos.x, pos.z) + right * (chair.side.sign() * chair.lateral);
    let ground_y = terrain.height(spot).min(pos.y);

    let mut b = MeshBuilder::default();
    let wood_warm = Color::srgb(0.76, 0.50, 0.24);
    let wood_dark = Color::srgb(0.56, 0.34, 0.16);
    let cushion_red = Color::srgb(0.84, 0.18, 0.20);

    let seat_h = 9.5;
    let back_h = 21.0;
    let half_w = 5.4;

    // Four sturdy wooden chair legs + cross-stretchers.
    for lx in [-half_w, half_w] {
        for lz in [-half_w, half_w] {
            b.cuboid(
                Vec3::new(lx, seat_h * 0.5, lz),
                Vec3::new(0.65, seat_h * 0.5, 0.65),
                wood_dark,
            );
        }
        b.cuboid(
            Vec3::new(lx, 3.8, 0.0),
            Vec3::new(0.42, 0.42, half_w),
            wood_dark,
        );
    }
    // Wooden seat slab & plush red seat cushion.
    b.cuboid(
        Vec3::new(0.0, seat_h + 0.45, 0.0),
        Vec3::new(half_w + 0.6, 0.55, half_w + 0.6),
        wood_warm,
    );
    b.cuboid(
        Vec3::new(0.0, seat_h + 1.15, 0.0),
        Vec3::new(half_w + 0.1, 0.35, half_w + 0.1),
        cushion_red,
    );

    // Tall ladder-back posts and horizontal backrest slats.
    for lx in [-half_w, half_w] {
        b.cuboid(
            Vec3::new(lx, (seat_h + back_h) * 0.5, -half_w),
            Vec3::new(0.65, (back_h - seat_h) * 0.5, 0.65),
            wood_dark,
        );
    }
    for sy in [13.2f32, 16.4, 19.6] {
        b.cuboid(
            Vec3::new(0.0, sy, -half_w),
            Vec3::new(half_w, 0.85, 0.35),
            wood_warm,
        );
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.68,
        reflectance: 0.28,
        ..default()
    });
    world.spawn((
        Mesh3d(assets.meshes.add(b.build())),
        MeshMaterial3d(mat),
        Transform::from_translation(Vec3::new(spot.x, ground_y, spot.y))
            .with_rotation(Quat::from_rotation_y(-heading + chair.heading)),
    ));
}

/// Spawns a tall birch `Kallax`-style wooden cube shelf unit (`photo 1`)
/// stocked with colourful cubby boxes, Matryoshka dolls and toy blocks.
fn spawn_cube_shelf(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    _terrain: &Terrain,
    shelf: &CubeShelf,
    assets: &mut Assets3d,
) {
    let mut b = MeshBuilder::default();
    let lap = track.lap_length();
    let len = shelf.span.length(lap);
    let bay_len = 14.0;
    let bays = (len / bay_len).ceil().max(1.0) as usize;

    let birch = Color::srgb(0.88, 0.74, 0.52);
    let birch_back = Color::srgb(0.76, 0.60, 0.40);
    let cubby_colors = [
        Color::srgb(0.88, 0.20, 0.22),
        Color::srgb(0.18, 0.52, 0.88),
        Color::srgb(0.96, 0.78, 0.16),
        Color::srgb(0.24, 0.72, 0.36),
    ];

    for k in 0..bays {
        let along = (shelf.span.from + (k as f32 + 0.5) * bay_len).rem_euclid(lap);
        let (pos, dir, heading) = road_pose(track, along);
        let right = Vec2::new(-dir.y, dir.x);
        let spot = Vec2::new(pos.x, pos.z) + right * (shelf.side.sign() * shelf.lateral);
        let rot = Quat::from_rotation_y(-heading + if shelf.side == Side::Left { 0.0 } else { PI });
        let base = Vec3::new(spot.x, pos.y - 0.5, spot.y);
        let xform = |local: Vec3| base + rot * local;

        // 3 tiers of cube shelves (`height = 14.4 m`, `width = 12.6 m`, `depth = 3.8 m`).
        let w = 6.3;
        let h = 14.4;
        let d = 1.9;
        // Backboard.
        b.quad_positions(
            [
                xform(Vec3::new(-w, 0.0, -d)),
                xform(Vec3::new(w, 0.0, -d)),
                xform(Vec3::new(w, h, -d)),
                xform(Vec3::new(-w, h, -d)),
            ],
            rot * Vec3::Z,
            birch_back,
        );
        // Horizontal shelves.
        for level in 0..=3 {
            let y = level as f32 * 4.8;
            b.beam(
                xform(Vec3::new(-w, y, 0.0)),
                xform(Vec3::new(w, y, 0.0)),
                0.45,
                d * 2.0,
                birch,
            );
        }
        // Vertical dividers + toys inside the cubbies.
        for col in 0..=3 {
            let x = -w + col as f32 * 4.2;
            b.beam(
                xform(Vec3::new(x, 0.0, 0.0)),
                xform(Vec3::new(x, h, 0.0)),
                0.45,
                d * 2.0,
                birch,
            );
            if col < 3 {
                for row in 0..3 {
                    let cx = x + 2.1;
                    let cy = row as f32 * 4.8 + 0.35;
                    let color = cubby_colors[(k + col + row) % cubby_colors.len()];
                    if (k + col + row) % 2 == 0 {
                        // Matryoshka doll inside the cubby (`photo 1`).
                        b.cylinder(xform(Vec3::new(cx, cy + 1.2, 0.0)), 1.05, 2.2, 10, color);
                        b.icosphere(
                            xform(Vec3::new(cx, cy + 2.8, 0.0)),
                            0.85,
                            1,
                            Color::srgb(0.98, 0.86, 0.72),
                        );
                    } else {
                        // Colourful toy storage bin / block inside the cubby.
                        b.beam(
                            xform(Vec3::new(cx - 1.3, cy + 1.5, 0.0)),
                            xform(Vec3::new(cx + 1.3, cy + 1.5, 0.0)),
                            2.8,
                            2.6,
                            color,
                        );
                    }
                }
            }
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.62,
        reflectance: 0.32,
        ..default()
    });
    world.spawn((Mesh3d(assets.meshes.add(b.build())), MeshMaterial3d(mat)));
}

/// Spawns a bright glossy yellow-and-blue playground slide (`Tobogán`,
/// `SlideRun`) with curved side-chutes and tubular steel support struts.
fn spawn_slide_run(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    slide: &SlideRun,
    assets: &mut Assets3d,
) {
    let mut b = MeshBuilder::default();
    let lap = track.lap_length();
    let len = slide.span.length(lap);
    let step = 3.5;
    let steps = (len / step).ceil() as usize;

    let slide_yellow = Color::srgb(0.99, 0.80, 0.08);
    let slide_blue = Color::srgb(0.12, 0.48, 0.92);
    let slide_red = Color::srgb(0.92, 0.18, 0.18);

    // Top entry arch at the start of the slide (`slide.span.from`).
    let (start_pos, _, start_heading) = road_pose(track, slide.span.from);
    let start_rot = Quat::from_rotation_y(-start_heading);
    for side in [-1.0f32, 1.0f32] {
        let post = start_pos + start_rot * Vec3::new(side * 10.2, 4.8, 0.0);
        b.cuboid(post, Vec3::new(0.85, 4.8, 0.85), slide_red);
    }
    b.beam(
        start_pos + start_rot * Vec3::new(-10.2, 9.4, 0.0),
        start_pos + start_rot * Vec3::new(10.2, 9.4, 0.0),
        1.1,
        1.1,
        slide_blue,
    );

    for k in 0..steps {
        let a0 = (slide.span.from + k as f32 * step).rem_euclid(lap);
        let a1 = (slide.span.from + ((k + 1) as f32 * step).min(len)).rem_euclid(lap);
        let i0 = track.index_at(a0);
        let i1 = track.index_at(a1);
        if i0 == i1 {
            continue;
        }

        // Glossy yellow slide surface strips along the road shoulders and
        // curved blue/yellow side-chute walls.
        let chute_color = if (k / 4) % 2 == 0 {
            slide_yellow
        } else {
            slide_blue
        };
        for side in [-1.0f32, 1.0f32] {
            let inner = side * (track.half_width_at(i0) - 0.4);
            let outer = side * (track.half_width_at(i0) + 1.8);
            let lift = Vec3::Y * (SURFACE_Y + 0.025);
            b.quad_positions(
                [
                    track.surface_point(i0, inner) + lift,
                    track.surface_point(i1, inner) + lift,
                    track.surface_point(i1, outer) + lift,
                    track.surface_point(i0, outer) + lift,
                ],
                Vec3::Y,
                slide_yellow,
            );
            let p0 = track.surface_point(i0, outer);
            let p1 = track.surface_point(i1, outer);
            b.beam(
                p0 + Vec3::Y * 1.15,
                p1 + Vec3::Y * 1.15,
                0.75,
                2.1,
                chute_color,
            );
            // Tubular red support struts every 5 segments.
            if k % 5 == 0 {
                b.cylinder(p0 - Vec3::Y * 2.5, 0.42, 5.5, 8, slide_red);
            }
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.22,
        reflectance: 0.72,
        ..default()
    });
    world.spawn((Mesh3d(assets.meshes.add(b.build())), MeshMaterial3d(mat)));
}

/// Spawns rainbow metallic xylophone bars, silver resonator tubes, and wooden
/// mallets lining both sides of a narrow bridge (`XylophoneSpan`).
fn spawn_xylophone_bridge(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    xylo: &XylophoneSpan,
    assets: &mut Assets3d,
) {
    let mut b = MeshBuilder::default();
    let lap = track.lap_length();
    let len = xylo.span.length(lap);
    let step = 4.0;
    let steps = (len / step).ceil() as usize;

    let rainbow = [
        Color::srgb(0.92, 0.16, 0.18), // C - Red
        Color::srgb(0.98, 0.48, 0.10), // D - Orange
        Color::srgb(0.98, 0.84, 0.12), // E - Yellow
        Color::srgb(0.20, 0.76, 0.28), // F - Green
        Color::srgb(0.14, 0.72, 0.86), // G - Cyan
        Color::srgb(0.16, 0.40, 0.90), // A - Blue
        Color::srgb(0.64, 0.22, 0.86), // B - Violet
    ];
    let pin_silver = Color::srgb(0.84, 0.86, 0.90);
    let wood = Color::srgb(0.78, 0.56, 0.30);

    for k in 0..steps {
        let along = (xylo.span.from + (k as f32 + 0.5) * step).rem_euclid(lap);
        let idx = track.index_at(along);
        let (_, heading) = track.pose_at(idx);
        let rot = Quat::from_rotation_y(-heading);
        let bar_color = rainbow[k % rainbow.len()];

        for side in [-1.0f32, 1.0f32] {
            let lat = side * (track.wall_at(idx) + 1.8);
            let p = track.surface_point(idx, lat) + Vec3::Y * 0.45;
            // Metallic rainbow xylophone tone bar + silver mounting pin.
            let bar_half = 2.4 - (k % 7) as f32 * 0.14;
            b.beam(
                p + rot * Vec3::new(-bar_half, 0.0, 0.0),
                p + rot * Vec3::new(bar_half, 0.0, 0.0),
                0.35,
                1.45,
                bar_color,
            );
            b.cylinder(p + Vec3::Y * 0.32, 0.26, 0.25, 8, pin_silver);
        }

        // Giant wooden xylophone mallets posed at both ends of the bridge.
        if k == 1 || k + 2 == steps {
            for side in [-1.0f32, 1.0f32] {
                let p = track.surface_point(idx, side * (track.wall_at(idx) + 3.2));
                b.cylinder(p + Vec3::Y * 3.5, 0.18, 7.0, 8, wood);
                b.icosphere(p + Vec3::Y * 7.2, 0.95, 1, Color::srgb(0.92, 0.22, 0.20));
            }
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.28,
        metallic: 0.45,
        ..default()
    });
    world.spawn((Mesh3d(assets.meshes.add(b.build())), MeshMaterial3d(mat)));
}

/// Spawns a giant `Big`-style floor piano (`FloorPiano`) with white ivory keys
/// and raised black sharps/flats beside the road.
fn spawn_floor_piano(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    _terrain: &Terrain,
    piano: &FloorPiano,
    assets: &mut Assets3d,
) {
    let mut b = MeshBuilder::default();
    let lap = track.lap_length();
    let len = piano.span.length(lap);
    let key_step = 2.4;
    let keys = (len / key_step).ceil() as usize;

    let ivory = Color::srgb(0.97, 0.96, 0.93);
    let ebony = Color::srgb(0.10, 0.11, 0.13);
    let frame_red = Color::srgb(0.84, 0.14, 0.16);

    for k in 0..keys {
        let along = (piano.span.from + (k as f32 + 0.5) * key_step).rem_euclid(lap);
        let (pos, dir, _) = road_pose(track, along);
        let right = Vec3::new(-dir.y, 0.0, dir.x);
        let forward = Vec3::new(dir.x, 0.0, dir.y);
        let center = pos + right * (piano.side.sign() * piano.lateral) + Vec3::Y * 0.12;

        // White key.
        b.beam(
            center - right * 2.8,
            center + right * 2.8,
            0.22,
            key_step * 0.88,
            ivory,
        );
        // Red casing border along the back edge.
        let back = center + right * (piano.side.sign() * 3.1);
        b.beam(
            back - forward * (key_step * 0.5),
            back + forward * (key_step * 0.5),
            0.42,
            0.65,
            frame_red,
        );
        // Black sharp/flat key on pattern `(0, 1, 3, 4, 5)` of each 7-key octave.
        if matches!(k % 7, 0 | 1 | 3 | 4 | 5) {
            let black_c = center
                + right * (piano.side.sign() * 0.9)
                + forward * (key_step * 0.48)
                + Vec3::Y * 0.18;
            b.beam(
                black_c - right * 1.65,
                black_c + right * 1.65,
                0.28,
                key_step * 0.48,
                ebony,
            );
        }
    }

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.35,
        reflectance: 0.55,
        ..default()
    });
    world.spawn((Mesh3d(assets.meshes.add(b.build())), MeshMaterial3d(mat)));
}

/// Spawns rows of giant wooden Nutcracker toy soldiers (`SoldierRow`, `photo 2`)
/// standing at attention beside the road.
fn spawn_soldier_row(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    row: &SoldierRow,
    assets: &mut Assets3d,
) {
    let soldier_mesh = assets.meshes.add(build_nutcracker_soldier_mesh());
    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.48,
        reflectance: 0.42,
        ..default()
    });

    let lap = track.lap_length();
    let len = row.span.length(lap);
    let count = (len / row.every.max(4.0)).floor() as usize;
    for k in 0..=count {
        let along = (row.span.from + k as f32 * row.every).rem_euclid(lap);
        let (pos, dir, heading) = road_pose(track, along);
        let right = Vec2::new(-dir.y, dir.x);
        let spot = Vec2::new(pos.x, pos.z) + right * (row.side.sign() * row.lateral);
        let y = terrain.height(spot).max(pos.y - 0.3);
        let face_road = -heading
            + if row.side == Side::Left {
                -FRAC_PI_2
            } else {
                FRAC_PI_2
            };
        world.spawn((
            Mesh3d(soldier_mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(Vec3::new(spot.x, y, spot.y))
                .with_rotation(Quat::from_rotation_y(face_road)),
        ));
    }
}

/// Builds a giant painted wooden Nutcracker soldier mesh (`photo 2`) with a
/// red pedestal, black boots, blue trousers, red tunic with gold buttons,
/// white beard/moustache, and a tall black bearskin hat with gold badge.
fn build_nutcracker_soldier_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let red = Color::srgb(0.86, 0.14, 0.16);
    let blue = Color::srgb(0.14, 0.38, 0.82);
    let gold = Color::srgb(0.96, 0.78, 0.18);
    let black = Color::srgb(0.12, 0.13, 0.16);
    let skin = Color::srgb(0.96, 0.78, 0.62);
    let white = Color::srgb(0.96, 0.96, 0.96);

    // Red & gold octagonal pedestal base.
    b.cylinder(Vec3::new(0.0, 0.35, 0.0), 1.35, 0.7, 12, red);
    b.cylinder(Vec3::new(0.0, 0.75, 0.0), 1.42, 0.16, 12, gold);
    // Boots & blue trousers.
    for lx in [-0.48f32, 0.48f32] {
        b.cylinder(Vec3::new(lx, 1.65, 0.0), 0.36, 1.7, 10, black);
        b.cylinder(Vec3::new(lx, 3.15, 0.0), 0.38, 1.5, 10, blue);
    }
    // Red tunic torso, black belt & gold cross-straps.
    b.cylinder(Vec3::new(0.0, 5.1, 0.0), 1.05, 2.5, 12, red);
    b.cylinder(Vec3::new(0.0, 4.1, 0.0), 1.08, 0.32, 12, black);
    b.cuboid(Vec3::new(0.0, 4.1, 1.06), Vec3::new(0.28, 0.22, 0.08), gold);
    // Epaulettes & arms at attention.
    for sx in [-1.25f32, 1.25f32] {
        b.icosphere(Vec3::new(sx, 6.1, 0.0), 0.42, 1, gold);
        b.cylinder(Vec3::new(sx, 4.85, 0.0), 0.28, 2.1, 10, red);
        b.icosphere(Vec3::new(sx, 3.65, 0.0), 0.26, 1, white);
    }
    // Head, white moustache/beard & tall black bearskin hat (`photo 2`).
    b.cylinder(Vec3::new(0.0, 7.0, 0.0), 0.82, 1.35, 12, skin);
    b.cuboid(
        Vec3::new(0.0, 6.75, 0.82),
        Vec3::new(0.55, 0.16, 0.14),
        white,
    );
    b.cuboid(
        Vec3::new(0.0, 6.25, 0.78),
        Vec3::new(0.28, 0.45, 0.12),
        white,
    );
    b.cylinder(Vec3::new(0.0, 8.85, 0.0), 0.96, 2.4, 12, black);
    b.cylinder(Vec3::new(0.0, 7.75, 0.0), 1.02, 0.24, 12, gold);
    b.icosphere(Vec3::new(0.0, 9.1, 0.96), 0.28, 1, gold);
    b.build()
}

/// Spawns all individual [`ToyProp`] items along the circuit, sharing one mesh
/// handle per [`ToyKind`] for maximum GPU instancing.
fn spawn_toy_props(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    props: &[ToyProp],
    assets: &mut Assets3d,
) {
    let slot_car_mesh = assets.meshes.add(build_slot_car_mesh());
    let hw_tower_mesh = assets.meshes.add(build_hot_wheels_tower_mesh());
    let turret_mesh = assets.meshes.add(build_cardboard_turret_mesh());
    let board_game_mesh = assets.meshes.add(build_board_game_stack_mesh());
    let robot_mesh = assets.meshes.add(build_wooden_robot_mesh());
    let matryoshka_mesh = assets.meshes.add(build_matryoshka_mesh());
    let horse_mesh = assets.meshes.add(build_rocking_horse_mesh());
    let helipad_mesh = assets.meshes.add(build_helipad_tower_mesh());
    let ship_mesh = assets.meshes.add(build_pirate_ship_mesh());
    let lego_mesh = assets.meshes.add(build_lego_tower_mesh());
    let hopscotch_mesh = assets.meshes.add(build_hopscotch_mesh());
    let knight_mesh = assets.meshes.add(build_standalone_knight_mesh());

    let mat = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.42,
        reflectance: 0.48,
        ..default()
    });

    for prop in props {
        let (pos, dir, heading) = road_pose(track, prop.along);
        let right = Vec2::new(-dir.y, dir.x);
        let spot = Vec2::new(pos.x, pos.z) + right * (prop.side.sign() * prop.lateral);
        let ground_y = match prop.kind {
            ToyKind::ToyPirateShip if terrain.height(spot) < 1.0 => 0.0,
            ToyKind::Hopscotch => pos.y + 0.04,
            _ => terrain.height(spot).max(pos.y - 0.5),
        };
        let face_road = -heading
            + if prop.side == Side::Left {
                -FRAC_PI_2
            } else {
                FRAC_PI_2
            };
        let mesh = match prop.kind {
            ToyKind::SlotCar => slot_car_mesh.clone(),
            ToyKind::HotWheelsTower => hw_tower_mesh.clone(),
            ToyKind::CardboardTurret => turret_mesh.clone(),
            ToyKind::BoardGameStack => board_game_mesh.clone(),
            ToyKind::WoodenRobot => robot_mesh.clone(),
            ToyKind::Matryoshka => matryoshka_mesh.clone(),
            ToyKind::RockingHorse => horse_mesh.clone(),
            ToyKind::TrainHelipadTower => helipad_mesh.clone(),
            ToyKind::ToyPirateShip => ship_mesh.clone(),
            ToyKind::LegoTower => lego_mesh.clone(),
            ToyKind::Hopscotch => hopscotch_mesh.clone(),
            ToyKind::PlaymobilKnight => knight_mesh.clone(),
        };
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(Vec3::new(spot.x, ground_y, spot.y))
                .with_rotation(Quat::from_rotation_y(face_road))
                .with_scale(Vec3::splat(prop.scale)),
        ));
    }
}

/// Giant 1:32 Scalextric / Ninco slot car with sleek GT bodywork, racing
/// stripes, rear wing and black rubber tires.
fn build_slot_car_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let red = Color::srgb(0.90, 0.14, 0.16);
    let white = Color::srgb(0.96, 0.96, 0.98);
    let glass = Color::srgb(0.18, 0.24, 0.34);
    let tire = Color::srgb(0.12, 0.12, 0.14);
    let gold = Color::srgb(0.95, 0.78, 0.22);

    b.cuboid(Vec3::new(0.0, 0.85, 0.0), Vec3::new(2.2, 0.58, 4.4), red);
    b.cuboid(
        Vec3::new(0.0, 1.45, 0.1),
        Vec3::new(2.22, 0.06, 4.42),
        white,
    );
    b.cuboid(
        Vec3::new(0.0, 1.65, -0.2),
        Vec3::new(1.65, 0.52, 2.1),
        glass,
    );
    // Rear GT wing.
    b.cuboid(Vec3::new(0.0, 2.1, 3.8), Vec3::new(2.1, 0.12, 0.55), red);
    for wx in [-1.5f32, 1.5f32] {
        b.cuboid(Vec3::new(wx, 1.65, 3.8), Vec3::new(0.12, 0.45, 0.45), tire);
    }
    // Four wheels with gold rims.
    for z in [-2.6f32, 2.6f32] {
        for x in [-2.15f32, 2.15f32] {
            b.cuboid(Vec3::new(x, 0.68, z), Vec3::new(0.35, 0.68, 0.68), tire);
            b.cuboid(
                Vec3::new(x * 1.08, 0.68, z),
                Vec3::new(0.08, 0.42, 0.42),
                gold,
            );
        }
    }
    b.build()
}

/// Fiery Hot Wheels stunt tower with blue base, orange vertical track columns
/// and giant circular tire ring at the top.
fn build_hot_wheels_tower_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let blue = Color::srgb(0.08, 0.40, 0.88);
    let orange = Color::srgb(0.99, 0.42, 0.04);
    let yellow = Color::srgb(0.99, 0.82, 0.10);
    let tire = Color::srgb(0.14, 0.15, 0.18);

    b.cuboid(Vec3::new(0.0, 1.0, 0.0), Vec3::new(3.2, 1.0, 3.2), blue);
    for x in [-1.6f32, 1.6f32] {
        b.cuboid(Vec3::new(x, 7.5, 0.0), Vec3::new(0.55, 6.5, 0.55), orange);
    }
    b.cylinder(Vec3::new(0.0, 14.8, 0.0), 3.4, 1.4, 16, tire);
    b.cylinder(Vec3::new(0.0, 14.8, 0.0), 2.1, 1.55, 16, orange);
    b.cylinder(Vec3::new(0.0, 14.8, 0.0), 1.0, 1.65, 12, yellow);
    b.build()
}

/// Cardboard castle turret with pink curtains, tan cardboard battlements and
/// a blue heraldic shield (`photo 3`).
fn build_cardboard_turret_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let cardboard = Color::srgb(0.82, 0.67, 0.48);
    let cardboard_dark = Color::srgb(0.68, 0.52, 0.35);
    let curtain_pink = Color::srgb(0.90, 0.36, 0.52);
    let shield_blue = Color::srgb(0.22, 0.46, 0.84);
    let gold = Color::srgb(0.96, 0.80, 0.22);

    b.cylinder(Vec3::new(0.0, 6.5, 0.0), 3.8, 13.0, 16, cardboard);
    b.cylinder(Vec3::new(0.0, 13.2, 0.0), 4.1, 1.2, 16, cardboard_dark);
    // Crenellated cardboard merlons around the top rim.
    for k in 0..8 {
        let angle = k as f32 * TAU / 8.0;
        b.cuboid(
            Vec3::new(angle.cos() * 3.8, 14.5, angle.sin() * 3.8),
            Vec3::new(0.75, 0.85, 0.75),
            cardboard,
        );
    }
    // Pink draped curtains in the turret doorway & upper window (`photo 3`).
    b.cuboid(
        Vec3::new(0.0, 3.2, 3.75),
        Vec3::new(1.4, 2.8, 0.25),
        curtain_pink,
    );
    b.cuboid(
        Vec3::new(0.0, 9.2, 3.75),
        Vec3::new(1.1, 1.6, 0.25),
        curtain_pink,
    );
    // Blue & gold shield emblem above the window.
    b.cuboid(
        Vec3::new(0.0, 11.8, 3.85),
        Vec3::new(0.95, 1.1, 0.18),
        shield_blue,
    );
    b.cuboid(
        Vec3::new(0.0, 11.8, 4.02),
        Vec3::new(0.55, 0.65, 0.08),
        gold,
    );
    b.build()
}

/// Stack of giant colourful board-game boxes and a giant white six-sided die.
fn build_board_game_stack_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let box_red = Color::srgb(0.88, 0.18, 0.20);
    let box_blue = Color::srgb(0.16, 0.46, 0.88);
    let box_yellow = Color::srgb(0.96, 0.78, 0.16);
    let box_green = Color::srgb(0.20, 0.72, 0.34);
    let white = Color::srgb(0.97, 0.97, 0.97);
    let pip = Color::srgb(0.12, 0.12, 0.14);

    b.cuboid(Vec3::new(0.0, 0.9, 0.0), Vec3::new(4.2, 0.9, 3.2), box_red);
    b.cuboid(
        Vec3::new(0.3, 2.6, -0.2),
        Vec3::new(3.9, 0.8, 2.9),
        box_blue,
    );
    b.cuboid(
        Vec3::new(-0.2, 4.2, 0.1),
        Vec3::new(3.6, 0.8, 2.7),
        box_yellow,
    );
    b.cuboid(
        Vec3::new(0.1, 5.7, 0.0),
        Vec3::new(3.2, 0.7, 2.4),
        box_green,
    );
    // Giant white die on top of the stack.
    b.cuboid(Vec3::new(0.0, 7.6, 0.0), Vec3::new(1.2, 1.2, 1.2), white);
    b.icosphere(Vec3::new(0.0, 7.6, 1.22), 0.24, 1, pip);
    b.build()
}

/// Giant painted wooden toy robot with blocky limbs, antenna and glowing eyes.
fn build_wooden_robot_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let teal = Color::srgb(0.18, 0.68, 0.76);
    let red = Color::srgb(0.90, 0.20, 0.20);
    let yellow = Color::srgb(0.98, 0.82, 0.16);
    let wood = Color::srgb(0.84, 0.66, 0.44);

    for lx in [-1.1f32, 1.1f32] {
        b.cuboid(Vec3::new(lx, 1.8, 0.0), Vec3::new(0.65, 1.8, 0.75), wood);
        b.cuboid(Vec3::new(lx, 0.4, 0.2), Vec3::new(0.75, 0.4, 1.05), red);
    }
    b.cuboid(Vec3::new(0.0, 5.2, 0.0), Vec3::new(2.1, 1.8, 1.3), teal);
    b.cuboid(Vec3::new(0.0, 5.3, 1.35), Vec3::new(1.3, 0.95, 0.1), yellow);
    for ax in [-2.65f32, 2.65f32] {
        b.cuboid(Vec3::new(ax, 5.1, 0.0), Vec3::new(0.45, 1.5, 0.55), wood);
    }
    b.cuboid(Vec3::new(0.0, 8.1, 0.0), Vec3::new(1.45, 1.15, 1.25), teal);
    for ex in [-0.55f32, 0.55f32] {
        b.icosphere(Vec3::new(ex, 8.2, 1.25), 0.32, 1, yellow);
    }
    b.cylinder(Vec3::new(0.0, 9.7, 0.0), 0.12, 1.0, 8, wood);
    b.icosphere(Vec3::new(0.0, 10.3, 0.0), 0.35, 1, red);
    b.build()
}

/// Giant painted Russian nesting doll (`Matryoshka`, `photo 1`).
fn build_matryoshka_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let scarf_red = Color::srgb(0.88, 0.14, 0.18);
    let apron_yellow = Color::srgb(0.98, 0.80, 0.18);
    let dress_blue = Color::srgb(0.16, 0.44, 0.86);
    let face_peach = Color::srgb(0.98, 0.86, 0.72);

    b.cylinder(Vec3::new(0.0, 2.6, 0.0), 2.2, 5.2, 14, dress_blue);
    b.icosphere(Vec3::new(0.0, 5.2, 0.0), 2.25, 2, scarf_red);
    b.icosphere(Vec3::new(0.0, 7.6, 0.0), 1.65, 2, scarf_red);
    b.icosphere(Vec3::new(0.0, 7.5, 0.65), 1.25, 2, face_peach);
    b.cylinder(Vec3::new(0.0, 3.2, 0.45), 1.85, 3.6, 12, apron_yellow);
    b.build()
}

/// Giant spotted plush rocking horse on curved wooden rockers (`photo 1`).
fn build_rocking_horse_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let cream = Color::srgb(0.96, 0.93, 0.86);
    let spot_brown = Color::srgb(0.48, 0.30, 0.16);
    let saddle_red = Color::srgb(0.86, 0.18, 0.20);
    let wood = Color::srgb(0.72, 0.48, 0.24);

    // Curved wooden rocker rails.
    for x in [-1.4f32, 1.4f32] {
        b.cuboid(Vec3::new(x, 0.35, 0.0), Vec3::new(0.24, 0.25, 4.2), wood);
    }
    // Four legs.
    for x in [-1.1f32, 1.1f32] {
        for z in [-1.8f32, 1.8f32] {
            b.beam(
                Vec3::new(x, 0.45, z * 1.2),
                Vec3::new(x * 0.85, 3.8, z * 0.85),
                0.48,
                0.48,
                cream,
            );
        }
    }
    // Barrel body, red saddle, neck & head.
    b.cuboid(Vec3::new(0.0, 4.3, 0.0), Vec3::new(1.35, 1.1, 2.4), cream);
    b.cuboid(
        Vec3::new(0.0, 4.9, 0.0),
        Vec3::new(1.42, 0.65, 1.05),
        saddle_red,
    );
    b.beam(
        Vec3::new(0.0, 4.6, 1.6),
        Vec3::new(0.0, 7.2, 2.5),
        1.1,
        1.2,
        cream,
    );
    b.cuboid(Vec3::new(0.0, 7.3, 3.1), Vec3::new(0.75, 0.72, 1.35), cream);
    // Brown plush spots & mane.
    b.icosphere(Vec3::new(1.1, 4.4, -0.8), 0.55, 1, spot_brown);
    b.icosphere(Vec3::new(-1.1, 4.2, 0.9), 0.52, 1, spot_brown);
    b.cuboid(
        Vec3::new(0.0, 6.2, 1.4),
        Vec3::new(0.25, 1.4, 0.35),
        spot_brown,
    );
    b.build()
}

/// Wooden Brio-style railway control tower with a green/yellow rooftop helipad
/// (inspired by the uploaded wooden railway rug photo).
fn build_helipad_tower_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let green = Color::srgb(0.20, 0.72, 0.34);
    let yellow = Color::srgb(0.98, 0.82, 0.14);
    let wood = Color::srgb(0.84, 0.66, 0.44);
    let glass = Color::srgb(0.56, 0.82, 0.96);

    for x in [-2.6f32, 2.6f32] {
        for z in [-2.6f32, 2.6f32] {
            b.cuboid(Vec3::new(x, 4.2, z), Vec3::new(0.55, 4.2, 0.55), green);
        }
    }
    b.cuboid(Vec3::new(0.0, 6.8, 0.0), Vec3::new(3.2, 1.6, 3.2), glass);
    b.cylinder(Vec3::new(0.0, 8.9, 0.0), 4.4, 0.55, 16, green);
    b.cylinder(Vec3::new(0.0, 9.22, 0.0), 3.4, 0.12, 16, yellow);
    b.cuboid(Vec3::new(0.0, 8.9, 0.0), Vec3::new(3.8, 0.25, 0.8), wood);
    b.build()
}

/// Toy wooden pirate ship / sailboat with a red hull, white square sails and
/// mast pennants (`photo 1`).
fn build_pirate_ship_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let hull_red = Color::srgb(0.76, 0.16, 0.16);
    let deck_wood = Color::srgb(0.80, 0.58, 0.32);
    let sail_white = Color::srgb(0.97, 0.96, 0.92);
    let gold = Color::srgb(0.96, 0.78, 0.18);

    b.cuboid(Vec3::new(0.0, 1.2, 0.0), Vec3::new(2.1, 1.2, 4.8), hull_red);
    b.cuboid(
        Vec3::new(0.0, 2.5, 0.0),
        Vec3::new(2.2, 0.25, 5.2),
        deck_wood,
    );
    b.cuboid(
        Vec3::new(0.0, 3.4, -3.6),
        Vec3::new(2.0, 0.85, 1.4),
        deck_wood,
    );
    for z in [-1.5f32, 1.8f32] {
        b.cylinder(Vec3::new(0.0, 6.8, z), 0.22, 8.8, 8, deck_wood);
        b.cuboid(
            Vec3::new(0.0, 7.2, z + 0.25),
            Vec3::new(2.8, 2.4, 0.14),
            sail_white,
        );
        b.cuboid(Vec3::new(0.5, 11.4, z), Vec3::new(0.75, 0.35, 0.08), gold);
    }
    b.build()
}

/// Cluster of giant primary-coloured studded LEGO construction bricks.
fn build_lego_tower_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let bricks = [
        (Vec3::new(0.0, 1.2, 0.0), Color::srgb(0.90, 0.14, 0.16)),
        (Vec3::new(0.8, 3.6, 0.0), Color::srgb(0.12, 0.44, 0.88)),
        (Vec3::new(-0.4, 6.0, 0.0), Color::srgb(0.98, 0.80, 0.10)),
        (Vec3::new(0.2, 8.4, 0.0), Color::srgb(0.18, 0.72, 0.28)),
    ];

    for (center, color) in bricks {
        b.cuboid(center, Vec3::new(2.4, 1.2, 1.2), color);
        for sx in [-1.8f32, -0.6, 0.6, 1.8] {
            for sz in [-0.6f32, 0.6] {
                b.cylinder(center + Vec3::new(sx, 1.42, sz), 0.36, 0.45, 10, color);
            }
        }
    }
    b.build()
}

/// Painted hopscotch court (`rayuela`, `photo 3`) laid flat on the shop floor.
fn build_hopscotch_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    let colors = [
        Color::srgb(0.94, 0.44, 0.62), // Pink
        Color::srgb(0.32, 0.72, 0.94), // Sky blue
        Color::srgb(0.98, 0.84, 0.22), // Sunny yellow
        Color::srgb(0.42, 0.84, 0.46), // Mint green
        Color::srgb(0.72, 0.48, 0.92), // Lavender
    ];
    let border = Color::srgb(0.98, 0.98, 0.98);

    let squares = [
        (0.0f32, -6.0f32, 0usize),
        (0.0, -3.6, 1),
        (-1.25, -1.2, 2),
        (1.25, -1.2, 3),
        (0.0, 1.2, 4),
        (-1.25, 3.6, 0),
        (1.25, 3.6, 1),
        (0.0, 6.0, 2),
    ];
    for (x, z, idx) in squares {
        b.cuboid(Vec3::new(x, 0.04, z), Vec3::new(1.18, 0.04, 1.18), border);
        b.cuboid(
            Vec3::new(x, 0.07, z),
            Vec3::new(1.02, 0.04, 1.02),
            colors[idx % colors.len()],
        );
    }
    b.build()
}

/// Standalone giant Playmobil medieval knight figure (`ToyKind::PlaymobilKnight`).
fn build_standalone_knight_mesh() -> Mesh {
    let mut b = MeshBuilder::default();
    append_playmobil_knight(&mut b, Vec3::ZERO, Color::srgb(0.14, 0.38, 0.84));
    b.build()
}

impl MeshBuilder {
    /// Adds a flat quad from four 3D corner positions (counter-clockwise).
    fn quad_positions(&mut self, corners: [Vec3; 4], normal: Vec3, color: Color) {
        self.quad(corners.map(|p| (p, Vec2::ZERO)), normal, color);
    }

    /// Adds an axis-aligned box centred at `center` with `half_extents`.
    fn cuboid(&mut self, center: Vec3, half_extents: Vec3, color: Color) {
        self.beam(
            Vec3::new(center.x, center.y - half_extents.y, center.z),
            Vec3::new(center.x, center.y + half_extents.y, center.z),
            half_extents.x * 2.0,
            half_extents.z * 2.0,
            color,
        );
    }

    /// Adds a vertical cylinder centred at `center` with `radius`, `height`
    /// and `segments` sides.
    fn cylinder(&mut self, center: Vec3, radius: f32, height: f32, segments: u32, color: Color) {
        let n = segments.max(6);
        let y0 = center.y - height * 0.5;
        let y1 = center.y + height * 0.5;
        let top_c = Vec3::new(center.x, y1, center.z);
        let bot_c = Vec3::new(center.x, y0, center.z);
        for k in 0..n {
            let a0 = (k as f32 / n as f32) * TAU;
            let a1 = ((k + 1) as f32 / n as f32) * TAU;
            let p0 = Vec2::new(a0.cos(), a0.sin()) * radius;
            let p1 = Vec2::new(a1.cos(), a1.sin()) * radius;
            let mid = Vec3::new((p0.x + p1.x) * 0.5, 0.0, (p0.y + p1.y) * 0.5).normalize_or_zero();
            self.quad_positions(
                [
                    Vec3::new(center.x + p0.x, y0, center.z + p0.y),
                    Vec3::new(center.x + p1.x, y0, center.z + p1.y),
                    Vec3::new(center.x + p1.x, y1, center.z + p1.y),
                    Vec3::new(center.x + p0.x, y1, center.z + p0.y),
                ],
                mid,
                color,
            );
            self.triangle(
                [
                    (top_c, Vec2::ZERO),
                    (Vec3::new(center.x + p1.x, y1, center.z + p1.y), Vec2::ZERO),
                    (Vec3::new(center.x + p0.x, y1, center.z + p0.y), Vec2::ZERO),
                ],
                Vec3::Y,
                color,
            );
            self.triangle(
                [
                    (bot_c, Vec2::ZERO),
                    (Vec3::new(center.x + p0.x, y0, center.z + p0.y), Vec2::ZERO),
                    (Vec3::new(center.x + p1.x, y0, center.z + p1.y), Vec2::ZERO),
                ],
                Vec3::NEG_Y,
                color,
            );
        }
    }

    /// Adds a vertical cone whose base is centred at `base` with `radius`,
    /// rising `height` metres to its apex.
    fn cone(&mut self, base: Vec3, radius: f32, height: f32, segments: u32, color: Color) {
        let n = segments.max(6);
        let apex = base + Vec3::Y * height;
        for k in 0..n {
            let a0 = (k as f32 / n as f32) * TAU;
            let a1 = ((k + 1) as f32 / n as f32) * TAU;
            let p0 = base + Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius);
            let p1 = base + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius);
            let outward = Vec3::new(
                (a0.cos() + a1.cos()) * 0.5,
                0.35,
                (a0.sin() + a1.sin()) * 0.5,
            )
            .normalize_or_zero();
            self.triangle(
                [(p0, Vec2::ZERO), (p1, Vec2::ZERO), (apex, Vec2::ZERO)],
                outward,
                color,
            );
            self.triangle(
                [(base, Vec2::ZERO), (p0, Vec2::ZERO), (p1, Vec2::ZERO)],
                Vec3::NEG_Y,
                color,
            );
        }
    }

    /// Adds afaceted sphere centred at `center` with `radius`.
    fn icosphere(&mut self, center: Vec3, radius: f32, _subdivisions: u32, color: Color) {
        let rings = 6u32;
        let sectors = 10u32;
        for r in 0..rings {
            let v0 = r as f32 / rings as f32;
            let v1 = (r + 1) as f32 / rings as f32;
            let phi0 = (v0 - 0.5) * PI;
            let phi1 = (v1 - 0.5) * PI;
            for s in 0..sectors {
                let u0 = (s as f32 / sectors as f32) * TAU;
                let u1 = ((s + 1) as f32 / sectors as f32) * TAU;
                let pt = |phi: f32, u: f32| {
                    Vec3::new(phi.cos() * u.cos(), phi.sin(), phi.cos() * u.sin())
                };
                let n00 = pt(phi0, u0);
                let n10 = pt(phi0, u1);
                let n11 = pt(phi1, u1);
                let n01 = pt(phi1, u0);
                let facing = (n00 + n10 + n11 + n01).normalize_or_zero();
                self.quad_positions(
                    [
                        center + n00 * radius,
                        center + n10 * radius,
                        center + n11 * radius,
                        center + n01 * radius,
                    ],
                    facing,
                    color,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toy_shop_meshes_have_valid_geometry() {
        for mut mesh in [
            build_toy_locomotive_mesh(),
            build_toy_wagon_mesh(Color::WHITE),
            build_nutcracker_soldier_mesh(),
            build_slot_car_mesh(),
            build_hot_wheels_tower_mesh(),
            build_cardboard_turret_mesh(),
            build_board_game_stack_mesh(),
            build_wooden_robot_mesh(),
            build_matryoshka_mesh(),
            build_rocking_horse_mesh(),
            build_helipad_tower_mesh(),
            build_pirate_ship_mesh(),
            build_lego_tower_mesh(),
            build_hopscotch_mesh(),
            build_standalone_knight_mesh(),
        ] {
            assert!(mesh.count_vertices() > 20);
            mesh.take_gpu_data().expect("valid finite mesh bounds");
            assert!(mesh.final_aabb.is_some());
        }
    }
}
