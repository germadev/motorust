//! The setting of a giant indoor toy store ([`ToyStore`]): full room enclosure
//! with perimeter toy shelves, store ceiling with beams, suspended model airplane,
//! polished counter furniture, and distinct sector track overlays:
//!
//! 1. **Sector 1 — Cash Register Counter (`2350..240 m`):**
//!    Polished wooden counter desk, giant open comic books laid across the track,
//!    AA battery finish line gantry, barcode scanner with red laser fan, mechanical
//!    cash register with glowing green customer display, comic crest jump ramp,
//!    and giant plush teddy bear at "Curva Peluches".
//! 2. **Sector 2 — Toy Train Railway (`520..940 m`):**
//!    Circular wooden and metal railway loop with classic steam locomotive, coal
//!    tender, passenger wagons, turned-wood trees, and crossing barrier.
//! 3. **Sector 3 — Construction Block Canyon (`960..1380 m`):**
//!    Studded building block road, continuous Lego-style interlocking brick walls
//!    in primary hues with top connection studs, block skyscraper towers, giant dice,
//!    card houses, and an overhead Lego archway.
//! 4. **Sector 4 — Hot Wheels Speedway (`1440..1880 m`):**
//!    Iconic bright vibrant orange Hot Wheels plastic track, continuous bright blue
//!    curved lip guardrails, snap connector ribs, and blue tubular steel support
//!    trestles elevating the track high above the floor.
//! 5. **Sector 5 — Scalextric Slot Circuit (`1920..2350 m`):**
//!    Iconic deep matte black slot track with dual metallic silver electrical
//!    conductor strips along each lane, track section seams, and alternating red-and-white
//!    slot-car crash barriers.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Assets3d;
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{
    BarcodeScanner, BatteryGantry, BlockTower, CardHouse, CashRegister, CeilingLight, ComicRamp,
    Die, DisplayTable, Matryoshka, Nutcracker, Scenery, StackingPyramid, SuspendedDino, TeddyBear,
    TinRobot, ToyCar, ToyCastle, ToySailboat, ToyShelf, ToyTrain, Xylophone,
};
use super::terrain::Terrain;
use crate::track::{Track, WALL_DISTANCE};

/// Primary and vibrant toy palette colors.
pub const TOY_RED: Color = Color::srgb(0.92, 0.15, 0.15);
pub const TOY_BLUE: Color = Color::srgb(0.12, 0.45, 0.95);
pub const TOY_YELLOW: Color = Color::srgb(0.96, 0.82, 0.10);
pub const TOY_GREEN: Color = Color::srgb(0.15, 0.78, 0.25);
pub const TOY_ORANGE: Color = Color::srgb(1.00, 0.40, 0.05);
pub const TOY_WHITE: Color = Color::srgb(0.95, 0.95, 0.96);
pub const TOY_BLACK: Color = Color::srgb(0.10, 0.10, 0.12);
pub const BATTERY_GOLD: Color = Color::srgb(0.88, 0.72, 0.25);
pub const TEDDY_BROWN: Color = Color::srgb(0.55, 0.35, 0.20);
pub const TEDDY_MUZZLE: Color = Color::srgb(0.78, 0.62, 0.45);
pub const WOOD_RAIL: Color = Color::srgb(0.72, 0.58, 0.38);
pub const COUNTER_WOOD: Color = Color::srgb(0.68, 0.46, 0.28);
pub const SILVER_METAL: Color = Color::srgb(0.88, 0.90, 0.95);

/// Room bounding box dimensions enclosing the entire circuit.
pub const ROOM_MIN_X: f32 = -150.0;
pub const ROOM_MAX_X: f32 = 950.0;
pub const ROOM_MIN_Z: f32 = -600.0;
pub const ROOM_MAX_Z: f32 = 250.0;
pub const ROOM_HEIGHT: f32 = 75.0;

/// Spawns the toy store room enclosure, sector track overlays, landmarks,
/// toys, and ceiling lighting, returning occupied footprints.
pub fn spawn_toy_store(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let Some(toy_store) = &scenery.toy_store else {
        return Vec::new();
    };

    let mut taken = Vec::new();

    // Standard materials for toy props and track elements.
    let plastic_red = assets.materials.add(StandardMaterial {
        base_color: TOY_RED,
        perceptual_roughness: 0.30,
        ..default()
    });
    let plastic_blue = assets.materials.add(StandardMaterial {
        base_color: TOY_BLUE,
        perceptual_roughness: 0.30,
        ..default()
    });
    let plastic_yellow = assets.materials.add(StandardMaterial {
        base_color: TOY_YELLOW,
        perceptual_roughness: 0.30,
        ..default()
    });
    let plastic_white = assets.materials.add(StandardMaterial {
        base_color: TOY_WHITE,
        perceptual_roughness: 0.25,
        ..default()
    });
    let plastic_black = assets.materials.add(StandardMaterial {
        base_color: TOY_BLACK,
        perceptual_roughness: 0.35,
        ..default()
    });
    let hw_orange_mat = assets.materials.add(StandardMaterial {
        base_color: TOY_ORANGE,
        perceptual_roughness: 0.22,
        reflectance: 0.6,
        ..default()
    });
    let scalextric_mat = assets.materials.add(StandardMaterial {
        base_color: TOY_BLACK,
        perceptual_roughness: 0.85,
        ..default()
    });
    let metal_strip_mat = assets.materials.add(StandardMaterial {
        base_color: SILVER_METAL,
        metallic: 0.9,
        perceptual_roughness: 0.2,
        reflectance: 0.9,
        ..default()
    });
    let plush_brown = assets.materials.add(StandardMaterial {
        base_color: TEDDY_BROWN,
        perceptual_roughness: 0.95,
        ..default()
    });
    let plush_muzzle = assets.materials.add(StandardMaterial {
        base_color: TEDDY_MUZZLE,
        perceptual_roughness: 0.95,
        ..default()
    });
    let laser_glow = assets.materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.1, 0.1, 0.85),
        emissive: LinearRgba::rgb(14.0, 0.3, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let screen_glow = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.95, 0.4),
        emissive: LinearRgba::rgb(0.5, 6.0, 1.5),
        ..default()
    });
    let ceiling_light_mat = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.96, 0.88),
        emissive: LinearRgba::rgb(8.0, 7.5, 6.0),
        ..default()
    });
    let wood_mat = assets.materials.add(StandardMaterial {
        base_color: WOOD_RAIL,
        perceptual_roughness: 0.70,
        ..default()
    });
    let desk_mat = assets.materials.add(StandardMaterial {
        base_color: COUNTER_WOOD,
        perceptual_roughness: 0.45,
        reflectance: 0.4,
        ..default()
    });
    let room_wall_mat = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.96, 0.93, 0.88),
        perceptual_roughness: 0.85,
        ..default()
    });
    let room_ceiling_mat = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.90, 0.86),
        perceptual_roughness: 0.90,
        ..default()
    });
    let room_floor_mat = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.52, 0.35),
        perceptual_roughness: 0.60,
        ..default()
    });
    let toy_vertex_mat = assets.materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.35,
        ..default()
    });

    // 0. The Toy Store Room Enclosure (4 walls, floor, ceiling, shelves, counter furniture).
    spawn_room_enclosure(
        world,
        &room_wall_mat,
        &room_floor_mat,
        &room_ceiling_mat,
        &desk_mat,
        &toy_vertex_mat,
        assets,
    );

    // Track sector surface overlays & toy-themed barriers:
    // Sector 1: Cash Register Counter & Comic Books.
    spawn_counter_sector(world, track, &desk_mat, &plastic_white, assets);

    // Sector 3: Building Block Canyon.
    spawn_block_canyon_sector(world, track, &toy_vertex_mat, assets);

    // Sector 4: Hot Wheels Speedway.
    spawn_hot_wheels_sector(world, track, terrain, &hw_orange_mat, &plastic_blue, assets);

    // Sector 5: Scalextric Slot Circuit.
    spawn_scalextric_sector(
        world,
        track,
        &scalextric_mat,
        &metal_strip_mat,
        &plastic_red,
        &toy_vertex_mat,
        assets,
    );

    // 1. Battery Gantries.
    for gantry in toy_store.battery_gantries {
        spawn_battery_gantry(
            world,
            track,
            terrain,
            gantry,
            &plastic_black,
            &plastic_yellow,
            assets,
        );
        let center = track.offset_point(track.index_at(gantry.along), 0.0);
        taken.push((center, WALL_DISTANCE + 8.0));
    }

    // 2. Cash Registers.
    for register in toy_store.cash_registers {
        spawn_cash_register(
            world,
            terrain,
            register,
            &plastic_white,
            &plastic_blue,
            &screen_glow,
            assets,
        );
        taken.push((register.position, 18.0));
    }

    // 3. Barcode Scanners.
    for scanner in toy_store.barcode_scanners {
        spawn_barcode_scanner(
            world,
            track,
            terrain,
            scanner,
            &plastic_black,
            &laser_glow,
            assets,
        );
    }

    // 4. Comic Book Ramps.
    for comic in toy_store.comic_ramps {
        spawn_comic_ramp(world, track, comic, &plastic_white, assets);
    }

    // 5. Plush Teddy Bears.
    for bear in toy_store.teddy_bears {
        spawn_teddy_bear(world, terrain, bear, &plush_brown, &plush_muzzle, assets);
        taken.push((bear.position, 18.0));
    }

    // 6. Toy Trains and Railway loops.
    for train in toy_store.toy_trains {
        spawn_toy_train(world, terrain, train, &wood_mat, &toy_vertex_mat, assets);
        taken.push((train.center, train.radius + 12.0));
    }

    // 7. Building Block Towers.
    for tower in toy_store.block_towers {
        spawn_block_tower(world, terrain, tower, &plastic_red, assets);
        taken.push((tower.position, 14.0));
    }

    // 8. Dice.
    for die in toy_store.dice {
        spawn_die(world, terrain, die, &plastic_white, &plastic_black, assets);
        taken.push((die.position, die.size * 1.5));
    }

    // 9. Card Houses.
    for card_house in toy_store.card_houses {
        spawn_card_house(world, terrain, card_house, &plastic_white, assets);
        taken.push((card_house.position, 16.0));
    }

    // 10. Ceiling Lights.
    for light in toy_store.ceiling_lights {
        spawn_ceiling_light(world, light, &plastic_black, &ceiling_light_mat, assets);
    }

    // 11. Toy Shelves.
    for shelf in toy_store.shelves {
        spawn_toy_shelf(world, terrain, shelf, &wood_mat, &toy_vertex_mat, assets);
        taken.push((shelf.center, shelf.length / 2.0 + 8.0));
    }

    // 12. Royal Guard Nutcracker Soldiers.
    for soldier in toy_store.nutcrackers {
        spawn_nutcracker(world, terrain, soldier, &toy_vertex_mat, assets);
        taken.push((soldier.position, soldier.height * 0.4));
    }

    // 13. Russian Nesting Dolls (Matryoshka).
    for matryoshka in toy_store.matryoshkas {
        spawn_matryoshka(world, terrain, matryoshka, &toy_vertex_mat, assets);
        taken.push((matryoshka.position, matryoshka.scale * 6.0));
    }

    // 14. Toy Sailboats.
    for boat in toy_store.sailboats {
        spawn_toy_sailboat(world, terrain, boat, &toy_vertex_mat, assets);
        taken.push((boat.position, boat.length * 0.8));
    }

    // 15. Rainbow Toy Xylophones.
    for xylo in toy_store.xylophones {
        spawn_xylophone(world, terrain, xylo, &toy_vertex_mat, assets);
        taken.push((xylo.position, xylo.length * 0.8));
    }

    // 16. Stacking Ring Pyramids.
    for pyramid in toy_store.stacking_pyramids {
        spawn_stacking_pyramid(world, terrain, pyramid, &toy_vertex_mat, assets);
        taken.push((pyramid.position, pyramid.height * 0.6));
    }

    // 17. Vintage Wind-Up Tin Robots.
    for robot in toy_store.tin_robots {
        spawn_tin_robot(world, terrain, robot, &toy_vertex_mat, assets);
        taken.push((robot.position, robot.height * 0.5));
    }

    // 18. Toy Castle Fortresses.
    for castle in toy_store.toy_castles {
        spawn_toy_castle(world, terrain, castle, &toy_vertex_mat, assets);
        taken.push((castle.position, castle.scale * 16.0));
    }

    // 19. Suspended Giant Toy Dinosaurs.
    for dino in toy_store.suspended_dinos {
        spawn_suspended_dino(world, dino, &toy_vertex_mat, assets);
    }

    // 20. Wooden Toy Cars.
    for car in toy_store.toy_cars {
        spawn_toy_car(world, terrain, car, &toy_vertex_mat, assets);
        taken.push((car.position, car.length * 0.7));
    }

    // 21. Multi-Tiered Round Display Tables.
    for table in toy_store.display_tables {
        spawn_display_table(world, terrain, table, &wood_mat, &toy_vertex_mat, assets);
        taken.push((table.position, table.radius + 2.0));
    }

    taken
}

// ---------------------------------------------------------------------------
// Room Enclosure & Furniture
// ---------------------------------------------------------------------------

fn spawn_room_enclosure(
    world: &mut ChildSpawnerCommands,
    wall_mat: &Handle<StandardMaterial>,
    floor_mat: &Handle<StandardMaterial>,
    ceiling_mat: &Handle<StandardMaterial>,
    desk_mat: &Handle<StandardMaterial>,
    plane_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut floor = MeshBuilder::default();
    let mut ceiling = MeshBuilder::default();
    let mut walls = MeshBuilder::default();
    let mut desk = MeshBuilder::default();
    let mut plane = MeshBuilder::default();

    // 1. Room Floor: parquet pattern.
    floor.quad(
        [
            (Vec3::new(ROOM_MIN_X, 0.05, ROOM_MIN_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MAX_X, 0.05, ROOM_MIN_Z), Vec2::X),
            (Vec3::new(ROOM_MAX_X, 0.05, ROOM_MAX_Z), Vec2::ONE),
            (Vec3::new(ROOM_MIN_X, 0.05, ROOM_MAX_Z), Vec2::Y),
        ],
        Vec3::Y,
        Color::srgb(0.72, 0.52, 0.35),
    );

    // Parquet strip accents across the floor.
    let mut pz = ROOM_MIN_Z;
    while pz < ROOM_MAX_Z {
        floor.beam(
            Vec3::new(ROOM_MIN_X, 0.06, pz),
            Vec3::new(ROOM_MAX_X, 0.06, pz),
            0.15,
            0.02,
            Color::srgb(0.55, 0.38, 0.22),
        );
        pz += 16.0;
    }

    // Grand Red Carpet Runner down center aisle (media_1792447057631.png).
    floor.beam(
        Vec3::new(ROOM_MIN_X, 0.08, -50.0),
        Vec3::new(ROOM_MAX_X, 0.08, -50.0),
        22.0,
        0.02,
        Color::srgb(0.85, 0.08, 0.12),
    );
    // Gold trim borders along red carpet:
    floor.beam(
        Vec3::new(ROOM_MIN_X, 0.09, -60.8),
        Vec3::new(ROOM_MAX_X, 0.09, -60.8),
        0.5,
        0.02,
        BATTERY_GOLD,
    );
    floor.beam(
        Vec3::new(ROOM_MIN_X, 0.09, -39.2),
        Vec3::new(ROOM_MAX_X, 0.09, -39.2),
        0.5,
        0.02,
        BATTERY_GOLD,
    );

    // Hopscotch ("rayuela") decal on floor near Sector 1 counter (media_1792447064611.png).
    let hopscotch_colors = [
        Color::srgb(0.20, 0.75, 0.85),
        Color::srgb(0.95, 0.40, 0.60),
        Color::srgb(0.96, 0.82, 0.10),
        Color::srgb(0.98, 0.50, 0.15),
        Color::srgb(0.65, 0.25, 0.85),
        Color::srgb(0.20, 0.80, 0.35),
    ];
    for sq in 0..7 {
        let sq_pos = Vec3::new(45.0, 0.08, -260.0 + sq as f32 * 5.0);
        let sq_col = hopscotch_colors[sq % hopscotch_colors.len()];
        floor.beam(sq_pos, sq_pos + Vec3::new(0.0, 0.0, 4.2), 4.2, 0.02, sq_col);
    }

    // 2. Room Ceiling: spanning overhead at Y = ROOM_HEIGHT.
    ceiling.quad(
        [
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::Y),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::ONE),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::X),
        ],
        Vec3::NEG_Y,
        Color::srgb(0.92, 0.90, 0.86),
    );

    // Ceiling beams / structural rafters.
    let mut rx = ROOM_MIN_X;
    while rx < ROOM_MAX_X {
        ceiling.beam(
            Vec3::new(rx, ROOM_HEIGHT - 0.6, ROOM_MIN_Z),
            Vec3::new(rx, ROOM_HEIGHT - 0.6, ROOM_MAX_Z),
            1.4,
            1.2,
            TOY_BLACK,
        );
        rx += 60.0;
    }

    // 3. Four Room Walls (West, East, North, South).
    // West Wall.
    walls.quad(
        [
            (Vec3::new(ROOM_MIN_X, 0.0, ROOM_MAX_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MIN_X, 0.0, ROOM_MIN_Z), Vec2::X),
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::ONE),
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::Y),
        ],
        Vec3::X,
        Color::srgb(0.95, 0.92, 0.87),
    );
    // East Wall.
    walls.quad(
        [
            (Vec3::new(ROOM_MAX_X, 0.0, ROOM_MIN_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MAX_X, 0.0, ROOM_MAX_Z), Vec2::X),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::ONE),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::Y),
        ],
        Vec3::NEG_X,
        Color::srgb(0.95, 0.92, 0.87),
    );
    // North Wall.
    walls.quad(
        [
            (Vec3::new(ROOM_MIN_X, 0.0, ROOM_MIN_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MAX_X, 0.0, ROOM_MIN_Z), Vec2::X),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::ONE),
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MIN_Z), Vec2::Y),
        ],
        Vec3::Z,
        Color::srgb(0.95, 0.92, 0.87),
    );
    // South Wall.
    walls.quad(
        [
            (Vec3::new(ROOM_MAX_X, 0.0, ROOM_MAX_Z), Vec2::ZERO),
            (Vec3::new(ROOM_MIN_X, 0.0, ROOM_MAX_Z), Vec2::X),
            (Vec3::new(ROOM_MIN_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::ONE),
            (Vec3::new(ROOM_MAX_X, ROOM_HEIGHT, ROOM_MAX_Z), Vec2::Y),
        ],
        Vec3::NEG_Z,
        Color::srgb(0.95, 0.92, 0.87),
    );

    // Wall vertical mouldings every 30 m.
    let mut wz = ROOM_MIN_Z;
    while wz < ROOM_MAX_Z {
        walls.beam(
            Vec3::new(ROOM_MIN_X + 0.3, 0.0, wz),
            Vec3::new(ROOM_MIN_X + 0.3, ROOM_HEIGHT, wz),
            0.6,
            0.3,
            WOOD_RAIL,
        );
        walls.beam(
            Vec3::new(ROOM_MAX_X - 0.3, 0.0, wz),
            Vec3::new(ROOM_MAX_X - 0.3, ROOM_HEIGHT, wz),
            0.6,
            0.3,
            WOOD_RAIL,
        );
        wz += 30.0;
    }

    // Massive Yellow Store Support Columns (media_1792447071124.png).
    let pillar_positions = [
        Vec2::new(120.0, -200.0),
        Vec2::new(120.0, -420.0),
        Vec2::new(380.0, -320.0),
        Vec2::new(380.0, -100.0),
        Vec2::new(680.0, -200.0),
        Vec2::new(680.0, -450.0),
    ];
    let pillar_color = Color::srgb(0.96, 0.88, 0.52);
    for col_pos in pillar_positions {
        cylinder(
            &mut walls,
            Vec3::new(col_pos.x, 0.0, col_pos.y),
            Vec3::new(col_pos.x, 2.0, col_pos.y),
            3.6,
            3.6,
            12,
            WOOD_RAIL,
        );
        cylinder(
            &mut walls,
            Vec3::new(col_pos.x, 2.0, col_pos.y),
            Vec3::new(col_pos.x, ROOM_HEIGHT - 2.0, col_pos.y),
            2.8,
            2.8,
            12,
            pillar_color,
        );
        cylinder(
            &mut walls,
            Vec3::new(col_pos.x, ROOM_HEIGHT - 2.0, col_pos.y),
            Vec3::new(col_pos.x, ROOM_HEIGHT, col_pos.y),
            3.6,
            3.6,
            12,
            WOOD_RAIL,
        );
    }

    // 4. Giant Suspended Toy Airplane (hanging from ceiling above Sector 4).
    let plane_pos = Vec3::new(550.0, 52.0, -100.0);
    // Fuselage cylinder.
    plane.beam(
        plane_pos + Vec3::new(0.0, 0.0, -13.0),
        plane_pos + Vec3::new(0.0, 0.0, 13.0),
        3.2,
        3.2,
        TOY_WHITE,
    );
    // Swept wings.
    plane.beam(
        plane_pos + Vec3::new(-14.0, 0.2, -1.0),
        plane_pos + Vec3::new(14.0, 0.2, -1.0),
        4.8,
        0.5,
        TOY_BLUE,
    );
    // Jet engines under wings.
    plane.beam(
        plane_pos + Vec3::new(-6.0, -1.2, -3.0),
        plane_pos + Vec3::new(-6.0, -1.2, 2.0),
        1.4,
        1.4,
        TOY_BLACK,
    );
    plane.beam(
        plane_pos + Vec3::new(6.0, -1.2, -3.0),
        plane_pos + Vec3::new(6.0, -1.2, 2.0),
        1.4,
        1.4,
        TOY_BLACK,
    );
    // Vertical tail fin.
    plane.beam(
        plane_pos + Vec3::new(0.0, 1.8, 10.0),
        plane_pos + Vec3::new(0.0, 6.0, 13.0),
        0.4,
        4.5,
        TOY_BLUE,
    );
    // Thin hanging suspension wires from ceiling.
    plane.beam(
        plane_pos + Vec3::new(-8.0, 0.2, 0.0),
        Vec3::new(plane_pos.x - 8.0, ROOM_HEIGHT, plane_pos.z),
        0.1,
        0.1,
        Color::srgb(0.7, 0.7, 0.75),
    );
    plane.beam(
        plane_pos + Vec3::new(8.0, 0.2, 0.0),
        Vec3::new(plane_pos.x + 8.0, ROOM_HEIGHT, plane_pos.z),
        0.1,
        0.1,
        Color::srgb(0.7, 0.7, 0.75),
    );

    // Yellow Model Biplane hanging from ceiling (media_1792447348822.jpg).
    let biplane_pos = Vec3::new(420.0, 48.0, -220.0);
    // Fuselage.
    plane.beam(
        biplane_pos + Vec3::new(0.0, 0.0, -7.0),
        biplane_pos + Vec3::new(0.0, 0.0, 7.0),
        2.2,
        2.2,
        TOY_YELLOW,
    );
    // Lower wing.
    plane.beam(
        biplane_pos + Vec3::new(-9.0, -0.6, 0.0),
        biplane_pos + Vec3::new(9.0, -0.6, 0.0),
        3.2,
        0.3,
        TOY_YELLOW,
    );
    // Upper wing.
    plane.beam(
        biplane_pos + Vec3::new(-9.0, 1.8, 0.0),
        biplane_pos + Vec3::new(9.0, 1.8, 0.0),
        3.2,
        0.3,
        TOY_YELLOW,
    );
    // Wing struts.
    for sx in [-7.0, 7.0] {
        plane.beam(
            biplane_pos + Vec3::new(sx, -0.6, 0.0),
            biplane_pos + Vec3::new(sx, 1.8, 0.0),
            0.2,
            0.2,
            TOY_BLACK,
        );
    }
    // Propeller at front nose.
    cylinder(
        &mut plane,
        biplane_pos + Vec3::new(0.0, 0.0, 7.0),
        biplane_pos + Vec3::new(0.0, 0.0, 7.6),
        0.8,
        0.8,
        8,
        Color::srgb(0.85, 0.20, 0.20),
    );
    plane.beam(
        biplane_pos + Vec3::new(-2.2, 0.0, 7.7),
        biplane_pos + Vec3::new(2.2, 0.0, 7.7),
        0.4,
        0.1,
        TOY_BLACK,
    );
    // Suspension wires.
    plane.beam(
        biplane_pos + Vec3::new(-4.0, 1.8, 0.0),
        Vec3::new(biplane_pos.x - 4.0, ROOM_HEIGHT, biplane_pos.z),
        0.08,
        0.08,
        Color::srgb(0.7, 0.7, 0.75),
    );
    plane.beam(
        biplane_pos + Vec3::new(4.0, 1.8, 0.0),
        Vec3::new(biplane_pos.x + 4.0, ROOM_HEIGHT, biplane_pos.z),
        0.08,
        0.08,
        Color::srgb(0.7, 0.7, 0.75),
    );

    // 5. Cashier Desk / Counter Furniture (under Sector 1).
    // Desk surface top just under the road at Y = 2.45 m.
    desk.beam(
        Vec3::new(75.0, 1.2, -450.0),
        Vec3::new(75.0, 1.2, 50.0),
        260.0,
        2.4,
        COUNTER_WOOD,
    );
    // Desk tabletop rim fascia.
    desk.beam(
        Vec3::new(-55.0, 2.35, -450.0),
        Vec3::new(-55.0, 2.35, 50.0),
        1.2,
        0.4,
        WOOD_RAIL,
    );

    // Wooden descent ramp (from counter straight down to Sector 2 floor level).
    let ramp_start = Vec3::new(260.0, 2.5, -390.0);
    let ramp_end = Vec3::new(315.0, 1.9, -250.0);
    desk.beam(ramp_start, ramp_end, 18.0, 0.4, WOOD_RAIL);

    if !floor.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(floor.build())),
            MeshMaterial3d(floor_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !ceiling.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(ceiling.build())),
            MeshMaterial3d(ceiling_mat.clone()),
            Transform::IDENTITY,
            NotShadowCaster,
        ));
    }
    if !walls.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(walls.build())),
            MeshMaterial3d(wall_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !desk.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(desk.build())),
            MeshMaterial3d(desk_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !plane.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(plane.build())),
            MeshMaterial3d(plane_mat.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// Sector 1: Cash Register Counter & Comic Book Overlays
// ---------------------------------------------------------------------------

fn spawn_counter_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    wood_mat: &Handle<StandardMaterial>,
    comic_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut wood_mesh = MeshBuilder::default();
    let mut comic_mesh = MeshBuilder::default();

    let seg_count = track.sample_count();
    let half = track.half_width();

    // Span from 2350 m through 0 to 240 m.
    let i_start = track.index_at(2350.0);
    let i_end = track.index_at(240.0);

    let mut i = i_start;
    while i != i_end {
        let next = (i + 1) % seg_count;
        let p0_l = track.surface_point(i, -half) + Vec3::Y * 0.035;
        let p0_r = track.surface_point(i, half) + Vec3::Y * 0.035;
        let p1_l = track.surface_point(next, -half) + Vec3::Y * 0.035;
        let p1_r = track.surface_point(next, half) + Vec3::Y * 0.035;

        // Wooden counter road surface.
        let wood_col = if (i / 4).is_multiple_of(2) {
            Color::srgb(0.70, 0.48, 0.28)
        } else {
            Color::srgb(0.66, 0.44, 0.25)
        };
        wood_mesh.quad(
            [
                (p0_l, Vec2::ZERO),
                (p0_r, Vec2::X),
                (p1_r, Vec2::ONE),
                (p1_l, Vec2::Y),
            ],
            Vec3::Y,
            wood_col,
        );

        // Counter edge wooden rails along borders.
        wood_mesh.beam(
            p0_l + Vec3::Y * 0.4,
            p1_l + Vec3::Y * 0.4,
            0.6,
            0.8,
            WOOD_RAIL,
        );
        wood_mesh.beam(
            p0_r + Vec3::Y * 0.4,
            p1_r + Vec3::Y * 0.4,
            0.6,
            0.8,
            WOOD_RAIL,
        );

        // Giant Open Comic Book lying on the track across the start/finish line.
        let along = track.lap_along_at(i);
        if along > 2390.0 || along < 30.0 {
            let t = (along - 2390.0).rem_euclid(track.lap_length()) / 60.0;
            let panel_col = match ((t * 8.0) as usize) % 4 {
                0 => TOY_YELLOW,
                1 => TOY_RED,
                2 => TOY_BLUE,
                _ => TOY_WHITE,
            };
            comic_mesh.quad(
                [
                    (p0_l + Vec3::Y * 0.005, Vec2::new(0.0, t)),
                    (p0_r + Vec3::Y * 0.005, Vec2::new(1.0, t)),
                    (p1_r + Vec3::Y * 0.005, Vec2::new(1.0, t + 0.02)),
                    (p1_l + Vec3::Y * 0.005, Vec2::new(0.0, t + 0.02)),
                ],
                Vec3::Y,
                panel_col,
            );
        }

        i = next;
    }

    if !wood_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(wood_mesh.build())),
            MeshMaterial3d(wood_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !comic_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(comic_mesh.build())),
            MeshMaterial3d(comic_mat.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// Sector 3: Building Block Canyon & Lego Walls
// ---------------------------------------------------------------------------

fn spawn_block_canyon_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    block_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut mesh = MeshBuilder::default();
    let half = track.half_width();
    let i_start = track.index_at(960.0);
    let i_end = track.index_at(1380.0);
    let seg_count = track.sample_count();

    let block_colors = [TOY_RED, TOY_BLUE, TOY_YELLOW, TOY_GREEN];

    let mut i = i_start;
    while i != i_end {
        let next = (i + 1) % seg_count;
        let p0_l = track.surface_point(i, -half) + Vec3::Y * 0.035;
        let p0_r = track.surface_point(i, half) + Vec3::Y * 0.035;
        let p1_l = track.surface_point(next, -half) + Vec3::Y * 0.035;
        let p1_r = track.surface_point(next, half) + Vec3::Y * 0.035;

        // Studded block road surface.
        let col = block_colors[(i / 3) % block_colors.len()];
        mesh.quad(
            [
                (p0_l, Vec2::ZERO),
                (p0_r, Vec2::X),
                (p1_r, Vec2::ONE),
                (p1_l, Vec2::Y),
            ],
            Vec3::Y,
            col,
        );

        // Continuous interlocking Lego-style walls with top connection studs.
        let wall_col_l = block_colors[(i / 2) % block_colors.len()];
        let wall_col_r = block_colors[((i / 2) + 1) % block_colors.len()];

        // Left wall block.
        mesh.beam(
            p0_l + Vec3::Y * 0.6,
            p1_l + Vec3::Y * 0.6,
            0.8,
            1.2,
            wall_col_l,
        );
        // Left top studs.
        mesh.beam(
            p0_l + Vec3::Y * 1.25,
            p0_l + Vec3::Y * 1.5,
            0.4,
            0.4,
            wall_col_l,
        );

        // Right wall block.
        mesh.beam(
            p0_r + Vec3::Y * 0.6,
            p1_r + Vec3::Y * 0.6,
            0.8,
            1.2,
            wall_col_r,
        );
        // Right top studs.
        mesh.beam(
            p0_r + Vec3::Y * 1.25,
            p0_r + Vec3::Y * 1.5,
            0.4,
            0.4,
            wall_col_r,
        );

        i = next;
    }

    // Giant Lego archway spanning across the road at along = 1170 m.
    let arch_idx = track.index_at(1170.0);
    let p_arch_l = track.surface_point(arch_idx, -half - 1.2);
    let p_arch_r = track.surface_point(arch_idx, half + 1.2);
    let arch_h = 9.0;
    // Left pillar.
    mesh.beam(p_arch_l, p_arch_l + Vec3::Y * arch_h, 2.2, 2.2, TOY_RED);
    // Right pillar.
    mesh.beam(p_arch_r, p_arch_r + Vec3::Y * arch_h, 2.2, 2.2, TOY_BLUE);
    // Crossbeam.
    mesh.beam(
        p_arch_l + Vec3::Y * arch_h,
        p_arch_r + Vec3::Y * arch_h,
        2.2,
        2.0,
        TOY_YELLOW,
    );

    // Board Game Tokens (Pawns, Checker Chips, Dominoes) along Sector 3 (media_1792447348822.jpg).
    // 1. Giant Halma / Ludo Pawns
    let pawn_specs = [
        (990.0, -half - 5.5, TOY_YELLOW),
        (1040.0, half + 5.5, TOY_RED),
        (1110.0, -half - 6.0, TOY_BLUE),
        (1210.0, half + 6.0, TOY_GREEN),
        (1280.0, -half - 5.5, TOY_RED),
        (1350.0, half + 5.5, TOY_YELLOW),
    ];
    for (along, lateral, pawn_col) in pawn_specs {
        let p_base = track.surface_point(track.index_at(along), lateral);
        // Base rim
        cylinder(
            &mut mesh,
            p_base,
            p_base + Vec3::Y * 0.4,
            1.6,
            1.5,
            12,
            pawn_col,
        );
        // Conical body
        cylinder(
            &mut mesh,
            p_base + Vec3::Y * 0.4,
            p_base + Vec3::Y * 3.6,
            1.4,
            0.5,
            12,
            pawn_col,
        );
        // Neck groove
        cylinder(
            &mut mesh,
            p_base + Vec3::Y * 3.6,
            p_base + Vec3::Y * 4.0,
            0.6,
            0.6,
            12,
            pawn_col,
        );
        // Spherical head
        cylinder(
            &mut mesh,
            p_base + Vec3::Y * 4.0,
            p_base + Vec3::Y * 5.0,
            0.6,
            1.1,
            12,
            pawn_col,
        );
        cylinder(
            &mut mesh,
            p_base + Vec3::Y * 5.0,
            p_base + Vec3::Y * 6.0,
            1.1,
            0.05,
            12,
            pawn_col,
        );
    }

    // 2. Stacks of Giant Checker / Draughts Chips
    let chip_stacks = [
        (1030.0, -half - 8.0, TOY_RED, 4),
        (1190.0, half + 8.5, Color::srgb(0.12, 0.12, 0.14), 5),
        (1310.0, -half - 8.0, TOY_WHITE, 3),
    ];
    for (along, lateral, chip_col, count) in chip_stacks {
        let base_pt = track.surface_point(track.index_at(along), lateral);
        for c in 0..count {
            let y0 = base_pt.y + c as f32 * 0.7;
            let y1 = y0 + 0.6;
            cylinder(
                &mut mesh,
                Vec3::new(base_pt.x, y0, base_pt.z),
                Vec3::new(base_pt.x, y1, base_pt.z),
                2.4,
                2.4,
                16,
                chip_col,
            );
            // Inner raised ridge on chip
            cylinder(
                &mut mesh,
                Vec3::new(base_pt.x, y1, base_pt.z),
                Vec3::new(base_pt.x, y1 + 0.08, base_pt.z),
                1.8,
                1.8,
                16,
                Color::srgb(
                    chip_col.to_srgba().red * 0.85,
                    chip_col.to_srgba().green * 0.85,
                    chip_col.to_srgba().blue * 0.85,
                ),
            );
        }
    }

    // 3. Giant Domino Tiles
    let domino_specs = [
        (1060.0, half + 9.0, 0.3),
        (1140.0, -half - 9.0, -0.5),
        (1250.0, half + 9.5, 0.8),
        (1330.0, -half - 9.0, -0.2),
    ];
    for (along, lateral, angle) in domino_specs {
        let d_pos = track.surface_point(track.index_at(along), lateral);
        let rot = Quat::from_rotation_y(angle);
        let fwd = rot * Vec3::Z;
        let right = rot * Vec3::X;
        let tile_len = 5.2;
        let tile_w = 2.6;
        let tile_h = 0.8;
        // Main tile body (white bone plastic)
        let c0 = d_pos - right * (tile_w * 0.5);
        let c1 = d_pos + right * (tile_w * 0.5);
        mesh.beam(
            c0 + Vec3::Y * (tile_h * 0.5),
            c1 + Vec3::Y * (tile_h * 0.5),
            tile_len,
            tile_h,
            Color::srgb(0.96, 0.95, 0.90),
        );
        // Center dividing groove
        mesh.beam(
            d_pos - right * (tile_w * 0.52) + Vec3::Y * (tile_h + 0.02),
            d_pos + right * (tile_w * 0.52) + Vec3::Y * (tile_h + 0.02),
            0.15,
            0.05,
            TOY_BLACK,
        );
        // Domino pips (black dots on both halves)
        for pip_offset in [-1.5 * fwd, 1.5 * fwd] {
            let pip_pt = d_pos + pip_offset + Vec3::Y * (tile_h + 0.04);
            mesh.beam(
                pip_pt - Vec3::X * 0.25,
                pip_pt + Vec3::X * 0.25,
                0.5,
                0.06,
                TOY_BLACK,
            );
        }
    }

    if !mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(mesh.build())),
            MeshMaterial3d(block_mat.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// Sector 4: Hot Wheels Speedway (Orange Track & Blue Guardrails)
// ---------------------------------------------------------------------------

fn spawn_hot_wheels_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    orange_mat: &Handle<StandardMaterial>,
    blue_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut orange_mesh = MeshBuilder::default();
    let mut blue_mesh = MeshBuilder::default();

    let half = track.half_width();
    let i_start = track.index_at(1440.0);
    let i_end = track.index_at(1880.0);
    let seg_count = track.sample_count();

    let mut i = i_start;
    while i != i_end {
        let next = (i + 1) % seg_count;
        let p0_l = track.surface_point(i, -half) + Vec3::Y * 0.035;
        let p0_r = track.surface_point(i, half) + Vec3::Y * 0.035;
        let p1_l = track.surface_point(next, -half) + Vec3::Y * 0.035;
        let p1_r = track.surface_point(next, half) + Vec3::Y * 0.035;

        // Vibrant orange plastic running surface.
        let col = if i.is_multiple_of(10) {
            Color::srgb(0.85, 0.32, 0.04) // Connector seam rib
        } else {
            TOY_ORANGE
        };
        orange_mesh.quad(
            [
                (p0_l, Vec2::ZERO),
                (p0_r, Vec2::X),
                (p1_r, Vec2::ONE),
                (p1_l, Vec2::Y),
            ],
            Vec3::Y,
            col,
        );

        // Continuous bright blue curved lip / guardrails along both sides.
        blue_mesh.beam(
            p0_l + Vec3::Y * 0.45,
            p1_l + Vec3::Y * 0.45,
            0.6,
            0.9,
            TOY_BLUE,
        );
        blue_mesh.beam(
            p0_r + Vec3::Y * 0.45,
            p1_r + Vec3::Y * 0.45,
            0.6,
            0.9,
            TOY_BLUE,
        );

        // Tubular steel support trestles elevating the orange track every ~30 m.
        if i.is_multiple_of(14) {
            let ground_l = terrain.height(Vec2::new(p0_l.x, p0_l.z)).max(0.05);
            let ground_r = terrain.height(Vec2::new(p0_r.x, p0_r.z)).max(0.05);

            // Left support pillar.
            blue_mesh.beam(
                Vec3::new(p0_l.x, ground_l, p0_l.z),
                p0_l - Vec3::Y * 0.1,
                1.1,
                1.1,
                TOY_BLUE,
            );
            // Right support pillar.
            blue_mesh.beam(
                Vec3::new(p0_r.x, ground_r, p0_r.z),
                p0_r - Vec3::Y * 0.1,
                1.1,
                1.1,
                TOY_BLUE,
            );
            // Under-track horizontal crossbar.
            blue_mesh.beam(
                p0_l - Vec3::Y * 0.2,
                p0_r - Vec3::Y * 0.2,
                1.4,
                0.8,
                TOY_BLUE,
            );
        }

        i = next;
    }

    // 1. Hot Wheels Entrance Arch Gantry at along = 1435.0 m (media_1792447348822.jpg).
    let arch_idx = track.index_at(1435.0);
    let p_l = track.surface_point(arch_idx, -half - 2.0);
    let p_r = track.surface_point(arch_idx, half + 2.0);
    let arch_h = 10.5;

    // Blue tubular side pylons
    blue_mesh.beam(p_l, p_l + Vec3::Y * arch_h, 1.8, 1.8, TOY_BLUE);
    blue_mesh.beam(p_r, p_r + Vec3::Y * arch_h, 1.8, 1.8, TOY_BLUE);
    // Overhead crossbar
    blue_mesh.beam(
        p_l + Vec3::Y * arch_h,
        p_r + Vec3::Y * arch_h,
        2.0,
        1.5,
        TOY_BLUE,
    );

    // Iconic flaming Hot Wheels header billboard
    let dir = (p_r - p_l).normalize_or_zero();
    let center_top = (p_l + p_r) * 0.5 + Vec3::Y * (arch_h + 1.8);
    // Orange flame backing banner
    orange_mesh.beam(
        center_top - dir * 7.5,
        center_top + dir * 7.5,
        1.2,
        2.4,
        Color::srgb(1.0, 0.35, 0.02),
    );
    // Yellow fiery inner flame tongue
    orange_mesh.beam(
        center_top - dir * 5.5 + Vec3::Y * 0.2,
        center_top + dir * 5.5 + Vec3::Y * 0.2,
        0.8,
        1.8,
        Color::srgb(1.0, 0.88, 0.08),
    );

    // 2. Giant Hot Wheels Double Vertical Loop-the-Loop landmark (media_1792447348822.jpg).
    // Located prominently beside Sector 4 at (560.0, 0.0, -95.0).
    let loop_center = Vec3::new(560.0, 0.0, -95.0);
    let loop_radius = 16.5;
    let loop_width = 3.2;
    let loop_segs = 36;
    let loop_offsets = [-4.0, 4.0]; // Two side-by-side vertical spiral loops

    for z_off in loop_offsets {
        let loop_base = loop_center + Vec3::new(0.0, 0.0, z_off);
        for s in 0..loop_segs {
            let th0 = s as f32 * TAU / loop_segs as f32;
            let th1 = (s + 1) as f32 * TAU / loop_segs as f32;

            // Loop in the X-Y plane (vertical 360 circle)
            let pt0 = loop_base
                + Vec3::new(
                    loop_radius * th0.sin(),
                    loop_radius * (1.0 - th0.cos()),
                    0.0,
                );
            let pt1 = loop_base
                + Vec3::new(
                    loop_radius * th1.sin(),
                    loop_radius * (1.0 - th1.cos()),
                    0.0,
                );

            // Orange track ribbon
            orange_mesh.beam(pt0, pt1, loop_width, 0.25, TOY_ORANGE);

            // Blue outer guide lips
            let side_l = Vec3::new(0.0, 0.0, -loop_width * 0.5);
            let side_r = Vec3::new(0.0, 0.0, loop_width * 0.5);

            blue_mesh.beam(pt0 + side_l, pt1 + side_l, 0.35, 0.8, TOY_BLUE);
            blue_mesh.beam(pt0 + side_r, pt1 + side_r, 0.35, 0.8, TOY_BLUE);
        }

        // Tubular A-frame support structure holding each loop upright
        let apex = loop_base + Vec3::new(0.0, loop_radius * 2.0, 0.0);
        let ground_l = loop_base + Vec3::new(-loop_radius * 0.8, 0.0, z_off * 0.5);
        let ground_r = loop_base + Vec3::new(loop_radius * 0.8, 0.0, z_off * 0.5);
        blue_mesh.beam(ground_l, apex, 1.2, 1.2, TOY_BLUE);
        blue_mesh.beam(ground_r, apex, 1.2, 1.2, TOY_BLUE);
        // Horizontal cross tie
        blue_mesh.beam(
            loop_base + Vec3::new(-loop_radius * 0.5, loop_radius, z_off * 0.5),
            loop_base + Vec3::new(loop_radius * 0.5, loop_radius, z_off * 0.5),
            1.0,
            1.0,
            TOY_BLUE,
        );
    }
    // Cross connecting struts between the two loops
    for angle_deg in [45.0f32, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
        let rad = angle_deg * PI / 180.0;
        let p_loop1 = loop_center
            + Vec3::new(
                loop_radius * rad.sin(),
                loop_radius * (1.0 - rad.cos()),
                loop_offsets[0],
            );
        let p_loop2 = loop_center
            + Vec3::new(
                loop_radius * rad.sin(),
                loop_radius * (1.0 - rad.cos()),
                loop_offsets[1],
            );
        blue_mesh.beam(p_loop1, p_loop2, 0.8, 0.8, TOY_BLUE);
    }

    if !orange_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(orange_mesh.build())),
            MeshMaterial3d(orange_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !blue_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(blue_mesh.build())),
            MeshMaterial3d(blue_mat.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// Sector 5: Scalextric Slot Circuit (Black Track & Dual Metallic Rails)
// ---------------------------------------------------------------------------

fn spawn_scalextric_sector(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    black_mat: &Handle<StandardMaterial>,
    metal_mat: &Handle<StandardMaterial>,
    barrier_mat: &Handle<StandardMaterial>,
    skirt_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut track_mesh = MeshBuilder::default();
    let mut metal_mesh = MeshBuilder::default();
    let mut barrier_mesh = MeshBuilder::default();
    let mut skirt_mesh = MeshBuilder::default();

    let half = track.half_width();
    let i_start = track.index_at(1920.0);
    let i_end = track.index_at(2350.0);
    let seg_count = track.sample_count();

    let mut i = i_start;
    while i != i_end {
        let next = (i + 1) % seg_count;
        let p0_l = track.surface_point(i, -half) + Vec3::Y * 0.035;
        let p0_r = track.surface_point(i, half) + Vec3::Y * 0.035;
        let p1_l = track.surface_point(next, -half) + Vec3::Y * 0.035;
        let p1_r = track.surface_point(next, half) + Vec3::Y * 0.035;

        // Matte black slot track surface.
        let col = if i.is_multiple_of(8) {
            Color::srgb(0.04, 0.04, 0.05) // Modular section joint
        } else {
            TOY_BLACK
        };
        track_mesh.quad(
            [
                (p0_l, Vec2::ZERO),
                (p0_r, Vec2::X),
                (p1_r, Vec2::ONE),
                (p1_l, Vec2::Y),
            ],
            Vec3::Y,
            col,
        );

        // DUAL METALLIC ELECTRICAL SLOT RAILS:
        // Left lane slot at lateral = -3.5 m; Right lane slot at lateral = +3.5 m.
        for lane_offset in [-3.5, 3.5] {
            let slot0_l = track.surface_point(i, lane_offset - 0.25) + Vec3::Y * 0.038;
            let slot0_r = track.surface_point(i, lane_offset + 0.25) + Vec3::Y * 0.038;
            let slot1_l = track.surface_point(next, lane_offset - 0.25) + Vec3::Y * 0.038;
            let slot1_r = track.surface_point(next, lane_offset + 0.25) + Vec3::Y * 0.038;

            // Recessed central slot channel.
            track_mesh.beam(
                slot0_l + (slot0_r - slot0_l) * 0.5,
                slot1_l + (slot1_r - slot1_l) * 0.5,
                0.14,
                0.05,
                Color::srgb(0.02, 0.02, 0.02),
            );

            // Left silver metallic conductor rail.
            metal_mesh.beam(slot0_l, slot1_l, 0.12, 0.03, SILVER_METAL);
            // Right silver metallic conductor rail.
            metal_mesh.beam(slot0_r, slot1_r, 0.12, 0.03, SILVER_METAL);
        }

        // Scalextric red-and-white alternating curved slot-car crash barriers.
        let barrier_col = if (i / 3).is_multiple_of(2) {
            TOY_RED
        } else {
            TOY_WHITE
        };
        barrier_mesh.beam(
            p0_l + Vec3::Y * 0.45,
            p1_l + Vec3::Y * 0.45,
            0.4,
            0.9,
            barrier_col,
        );
        barrier_mesh.beam(
            p0_r + Vec3::Y * 0.45,
            p1_r + Vec3::Y * 0.45,
            0.4,
            0.9,
            barrier_col,
        );

        // Red pleated table skirts hanging from track edges down to floor level (media_1792447071124.png).
        let skirt_col = if (i / 2).is_multiple_of(2) {
            Color::srgb(0.76, 0.08, 0.12)
        } else {
            Color::srgb(0.66, 0.06, 0.10) // Pleated shadow fold
        };
        let floor_y = 0.05;
        let fwd_l = (p1_l - p0_l).normalize_or_zero();
        let norm_l = -fwd_l.cross(Vec3::Y).normalize_or_zero();

        let fwd_r = (p1_r - p0_r).normalize_or_zero();
        let norm_r = fwd_r.cross(Vec3::Y).normalize_or_zero();

        // Left table skirt quad:
        skirt_mesh.quad(
            [
                (Vec3::new(p0_l.x, floor_y, p0_l.z), Vec2::ZERO),
                (Vec3::new(p1_l.x, floor_y, p1_l.z), Vec2::X),
                (p1_l, Vec2::ONE),
                (p0_l, Vec2::Y),
            ],
            norm_l,
            skirt_col,
        );
        // Right table skirt quad:
        skirt_mesh.quad(
            [
                (p0_r, Vec2::ZERO),
                (p1_r, Vec2::X),
                (Vec3::new(p1_r.x, floor_y, p1_r.z), Vec2::ONE),
                (Vec3::new(p0_r.x, floor_y, p0_r.z), Vec2::Y),
            ],
            norm_r,
            skirt_col,
        );
        // Bottom white hem trim band:
        skirt_mesh.beam(
            Vec3::new(p0_l.x, floor_y + 0.12, p0_l.z),
            Vec3::new(p1_l.x, floor_y + 0.12, p1_l.z),
            0.2,
            0.24,
            Color::srgb(0.95, 0.95, 0.95),
        );
        skirt_mesh.beam(
            Vec3::new(p0_r.x, floor_y + 0.12, p0_r.z),
            Vec3::new(p1_r.x, floor_y + 0.12, p1_r.z),
            0.2,
            0.24,
            Color::srgb(0.95, 0.95, 0.95),
        );

        i = next;
    }

    if !track_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(track_mesh.build())),
            MeshMaterial3d(black_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !metal_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(metal_mesh.build())),
            MeshMaterial3d(metal_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !barrier_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(barrier_mesh.build())),
            MeshMaterial3d(barrier_mat.clone()),
            Transform::IDENTITY,
        ));
    }
    if !skirt_mesh.is_empty() {
        world.spawn((
            Mesh3d(assets.meshes.add(skirt_mesh.build())),
            MeshMaterial3d(skirt_mat.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// 1. Battery Gantry
// ---------------------------------------------------------------------------

fn spawn_battery_gantry(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    gantry: &BatteryGantry,
    body_mat: &Handle<StandardMaterial>,
    terminal_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let index = track.index_at(gantry.along);
    let left_pos = track.offset_point(index, -(WALL_DISTANCE + 2.4));
    let right_pos = track.offset_point(index, WALL_DISTANCE + 2.4);
    let ground_left = terrain
        .height(left_pos)
        .max(track.surface_point(index, 0.0).y);
    let ground_right = terrain
        .height(right_pos)
        .max(track.surface_point(index, 0.0).y);

    let (battery_mesh, terminal_mesh, arch_mesh) = make_battery_gantry_meshes(gantry.height);

    let battery_h = assets.meshes.add(battery_mesh);
    let terminal_h = assets.meshes.add(terminal_mesh);
    let arch_h = assets.meshes.add(arch_mesh);

    // Left battery.
    world.spawn((
        Mesh3d(battery_h.clone()),
        MeshMaterial3d(body_mat.clone()),
        Transform::from_xyz(left_pos.x, ground_left, left_pos.y),
    ));
    world.spawn((
        Mesh3d(terminal_h.clone()),
        MeshMaterial3d(terminal_mat.clone()),
        Transform::from_xyz(left_pos.x, ground_left + gantry.height, left_pos.y),
    ));

    // Right battery.
    world.spawn((
        Mesh3d(battery_h),
        MeshMaterial3d(body_mat.clone()),
        Transform::from_xyz(right_pos.x, ground_right, right_pos.y),
    ));
    world.spawn((
        Mesh3d(terminal_h),
        MeshMaterial3d(terminal_mat.clone()),
        Transform::from_xyz(right_pos.x, ground_right + gantry.height, right_pos.y),
    ));

    // Overhead truss spanning between the battery poles.
    let mid_pos = track.surface_point(index, 0.0);
    world.spawn((
        Mesh3d(arch_h),
        MeshMaterial3d(terminal_mat.clone()),
        Transform::from_xyz(mid_pos.x, (ground_left + ground_right) * 0.5, mid_pos.z),
    ));
}

fn make_battery_gantry_meshes(height: f32) -> (Mesh, Mesh, Mesh) {
    let r = 2.4;
    let segs = 16;
    let mut body = MeshBuilder::default();
    let mut terminal = MeshBuilder::default();

    // Cylindrical battery barrel.
    for i in 0..segs {
        let a0 = i as f32 * TAU / segs as f32;
        let a1 = (i + 1) as f32 * TAU / segs as f32;
        let p00 = Vec3::new(r * a0.cos(), 0.0, r * a0.sin());
        let p10 = Vec3::new(r * a1.cos(), 0.0, r * a1.sin());
        let p01 = Vec3::new(r * a0.cos(), height, r * a0.sin());
        let p11 = Vec3::new(r * a1.cos(), height, r * a1.sin());

        let col = if i % 4 == 0 { BATTERY_GOLD } else { TOY_BLACK };
        body.quad(
            [
                (p00, Vec2::new(a0 / TAU, 0.0)),
                (p10, Vec2::new(a1 / TAU, 0.0)),
                (p11, Vec2::new(a1 / TAU, 1.0)),
                (p01, Vec2::new(a0 / TAU, 1.0)),
            ],
            p00.normalize_or_zero(),
            col,
        );
    }

    // Positive terminal cap nub on top.
    let term_r = 1.0;
    let term_h = 1.5;
    for i in 0..segs {
        let a0 = i as f32 * TAU / segs as f32;
        let a1 = (i + 1) as f32 * TAU / segs as f32;
        let p00 = Vec3::new(term_r * a0.cos(), 0.0, term_r * a0.sin());
        let p10 = Vec3::new(term_r * a1.cos(), 0.0, term_r * a1.sin());
        let p01 = Vec3::new(term_r * a0.cos(), term_h, term_r * a0.sin());
        let p11 = Vec3::new(term_r * a1.cos(), term_h, term_r * a1.sin());

        terminal.quad(
            [
                (p00, Vec2::ZERO),
                (p10, Vec2::X),
                (p11, Vec2::ONE),
                (p01, Vec2::Y),
            ],
            p00.normalize_or_zero(),
            BATTERY_GOLD,
        );
    }

    // Top horizontal connecting arch.
    let mut arch = MeshBuilder::default();
    arch.beam(
        Vec3::new(-(WALL_DISTANCE + 2.0), height + 0.6, 0.0),
        Vec3::new(WALL_DISTANCE + 2.0, height + 0.6, 0.0),
        1.2,
        0.6,
        BATTERY_GOLD,
    );

    (body.build(), terminal.build(), arch.build())
}

// ---------------------------------------------------------------------------
// 2. Cash Register
// ---------------------------------------------------------------------------

fn spawn_cash_register(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    reg: &CashRegister,
    body_mat: &Handle<StandardMaterial>,
    key_mat: &Handle<StandardMaterial>,
    screen_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(reg.position);
    let rot = Quat::from_rotation_y(reg.heading);
    let trans = Transform::from_xyz(reg.position.x, ground, reg.position.y).with_rotation(rot);

    let (body, keys, screen) = make_cash_register_meshes();

    world.spawn((
        Mesh3d(assets.meshes.add(body)),
        MeshMaterial3d(body_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(keys)),
        MeshMaterial3d(key_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(screen)),
        MeshMaterial3d(screen_mat.clone()),
        trans,
        NotShadowCaster,
    ));
}

fn make_cash_register_meshes() -> (Mesh, Mesh, Mesh) {
    let mut body = MeshBuilder::default();
    // Cash drawer base box.
    body.beam(
        Vec3::new(0.0, 2.0, -6.0),
        Vec3::new(0.0, 2.0, 6.0),
        14.0,
        4.0,
        Color::srgb(0.85, 0.85, 0.88),
    );
    // Upper register housing with angled keyboard tray.
    body.beam(
        Vec3::new(0.0, 6.0, -5.0),
        Vec3::new(0.0, 6.0, 3.0),
        12.0,
        4.0,
        TOY_WHITE,
    );

    let mut keys = MeshBuilder::default();
    // Grid of buttons on the slanted keyboard tray.
    for row in 0..4 {
        for col in 0..6 {
            let kx = (col as f32 - 2.5) * 1.6;
            let kz = 0.5 + row as f32 * 1.1;
            let ky = 7.1;
            let color = match (row + col) % 3 {
                0 => TOY_BLUE,
                1 => TOY_RED,
                _ => TOY_YELLOW,
            };
            keys.beam(
                Vec3::new(kx, ky, kz - 0.35),
                Vec3::new(kx, ky, kz + 0.35),
                1.0,
                0.5,
                color,
            );
        }
    }

    let mut screen = MeshBuilder::default();
    // Backlit green customer-facing display turret on top.
    screen.beam(
        Vec3::new(0.0, 10.5, -3.0),
        Vec3::new(0.0, 10.5, -2.2),
        7.5,
        2.2,
        Color::srgb(0.1, 0.9, 0.3),
    );

    (body.build(), keys.build(), screen.build())
}

// ---------------------------------------------------------------------------
// 3. Barcode Scanner
// ---------------------------------------------------------------------------

fn spawn_barcode_scanner(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scanner: &BarcodeScanner,
    stand_mat: &Handle<StandardMaterial>,
    laser_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let index = track.index_at(scanner.along);
    let lateral = scanner.side.sign() * (WALL_DISTANCE + 2.5);
    let spot = track.offset_point(index, lateral);
    let ground = terrain.height(spot).max(track.surface_point(index, 0.0).y);

    let road_mid = track.surface_point(index, 0.0);
    let across_dir = (road_mid - Vec3::new(spot.x, ground, spot.y)).normalize();

    let mut stand = MeshBuilder::default();
    // Scanner pedestal and flexible gooseneck arm.
    stand.beam(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 7.5, 0.0),
        0.8,
        0.8,
        TOY_BLACK,
    );
    stand.beam(
        Vec3::new(0.0, 7.5, 0.0),
        Vec3::new(across_dir.x * 3.5, 8.5, across_dir.z * 3.5),
        0.9,
        0.9,
        TOY_BLACK,
    );
    // Scanner head nozzle.
    let head_pos = Vec3::new(across_dir.x * 4.2, 8.2, across_dir.z * 4.2);
    stand.beam(head_pos, head_pos + across_dir * 1.5, 1.4, 1.2, TOY_RED);

    // Glowing laser beam fan sweeping across the track surface.
    let mut laser = MeshBuilder::default();
    let road_left = track.surface_point(index, -WALL_DISTANCE);
    let road_right = track.surface_point(index, WALL_DISTANCE);

    let l_start = head_pos + across_dir * 1.5;
    let l_end_left = road_left - Vec3::new(spot.x, ground, spot.y);
    let l_end_right = road_right - Vec3::new(spot.x, ground, spot.y);

    laser.triangle(
        [
            (l_start, Vec2::new(0.5, 0.0)),
            (l_end_left, Vec2::new(0.0, 1.0)),
            (l_end_right, Vec2::new(1.0, 1.0)),
        ],
        Vec3::Y,
        Color::srgba(1.0, 0.15, 0.15, 0.75),
    );

    let trans = Transform::from_xyz(spot.x, ground, spot.y);
    world.spawn((
        Mesh3d(assets.meshes.add(stand.build())),
        MeshMaterial3d(stand_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(laser.build())),
        MeshMaterial3d(laser_mat.clone()),
        trans,
        NotShadowCaster,
    ));
}

// ---------------------------------------------------------------------------
// 4. Comic Book Ramp
// ---------------------------------------------------------------------------

fn spawn_comic_ramp(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    comic: &ComicRamp,
    page_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let index = track.index_at(comic.along);
    let center = track.surface_point(index, 0.0);
    let road_dir =
        (track.surface_point((index + 1) % track.sample_count(), 0.0) - center).normalize();
    let rot = Quat::from_rotation_y(-road_dir.z.atan2(road_dir.x) + FRAC_PI_2);

    let mut pages = MeshBuilder::default();
    let w = comic.width;
    let l = 18.0;

    // Curved open magazine/comic page forming a smooth jump crest.
    let steps = 10;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let z0 = (t0 - 0.5) * l;
        let z1 = (t1 - 0.5) * l;
        let y0 = (1.0 - (t0 * 2.0 - 1.0).powi(2)) * 1.8;
        let y1 = (1.0 - (t1 * 2.0 - 1.0).powi(2)) * 1.8;

        let col = if (i / 2) % 2 == 0 {
            TOY_YELLOW
        } else {
            TOY_BLUE
        };
        pages.quad(
            [
                (Vec3::new(-w / 2.0, y0, z0), Vec2::new(0.0, t0)),
                (Vec3::new(w / 2.0, y0, z0), Vec2::new(1.0, t0)),
                (Vec3::new(w / 2.0, y1, z1), Vec2::new(1.0, t1)),
                (Vec3::new(-w / 2.0, y1, z1), Vec2::new(0.0, t1)),
            ],
            Vec3::Y,
            col,
        );
    }

    world.spawn((
        Mesh3d(assets.meshes.add(pages.build())),
        MeshMaterial3d(page_mat.clone()),
        Transform::from_translation(center).with_rotation(rot),
    ));
}

// ---------------------------------------------------------------------------
// 5. Plush Teddy Bear
// ---------------------------------------------------------------------------

fn spawn_teddy_bear(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    bear: &TeddyBear,
    fur_mat: &Handle<StandardMaterial>,
    muzzle_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(bear.position);
    let s = bear.scale;
    let rot = Quat::from_rotation_y(bear.heading);
    let trans = Transform::from_xyz(bear.position.x, ground, bear.position.y).with_rotation(rot);

    let mut fur = MeshBuilder::default();
    let mut muzzle = MeshBuilder::default();

    // Body (chubby cylinder/box).
    fur.beam(
        Vec3::new(0.0, 0.5 * s, 0.0),
        Vec3::new(0.0, 8.0 * s, 0.0),
        8.0 * s,
        7.0 * s,
        TEDDY_BROWN,
    );
    // Head.
    fur.beam(
        Vec3::new(0.0, 11.5 * s, -0.5 * s),
        Vec3::new(0.0, 12.0 * s, 0.5 * s),
        6.5 * s,
        6.0 * s,
        TEDDY_BROWN,
    );
    // Ears.
    fur.beam(
        Vec3::new(-3.2 * s, 15.0 * s, 0.0),
        Vec3::new(-3.2 * s, 16.2 * s, 0.0),
        2.2 * s,
        1.8 * s,
        TEDDY_BROWN,
    );
    fur.beam(
        Vec3::new(3.2 * s, 15.0 * s, 0.0),
        Vec3::new(3.2 * s, 16.2 * s, 0.0),
        2.2 * s,
        1.8 * s,
        TEDDY_BROWN,
    );
    // Seated legs/paws resting forward.
    fur.beam(
        Vec3::new(-3.0 * s, 1.8 * s, 1.0 * s),
        Vec3::new(-3.0 * s, 1.8 * s, 6.0 * s),
        3.2 * s,
        3.0 * s,
        TEDDY_BROWN,
    );
    fur.beam(
        Vec3::new(3.0 * s, 1.8 * s, 1.0 * s),
        Vec3::new(3.0 * s, 1.8 * s, 6.0 * s),
        3.2 * s,
        3.0 * s,
        TEDDY_BROWN,
    );

    // Muzzle / Snout.
    muzzle.beam(
        Vec3::new(0.0, 11.0 * s, 2.2 * s),
        Vec3::new(0.0, 11.0 * s, 3.4 * s),
        2.8 * s,
        2.2 * s,
        TEDDY_MUZZLE,
    );

    // Two Baby Teddy Cubs seated beside Mama Bear (Curva Peluches in media_1792447348822.jpg).
    let cubs = [
        // Left cub
        (Vec3::new(-7.2 * s, 0.0, 1.2 * s), s * 0.44, TEDDY_BROWN),
        // Right cub
        (
            Vec3::new(6.8 * s, 0.0, 1.8 * s),
            s * 0.38,
            Color::srgb(0.68, 0.45, 0.28),
        ),
    ];
    for (cub_pos, cs, cub_col) in cubs {
        // Cub body
        fur.beam(
            cub_pos + Vec3::new(0.0, 0.4 * cs, 0.0),
            cub_pos + Vec3::new(0.0, 6.5 * cs, 0.0),
            7.0 * cs,
            6.0 * cs,
            cub_col,
        );
        // Cub head
        fur.beam(
            cub_pos + Vec3::new(0.0, 9.5 * cs, -0.4 * cs),
            cub_pos + Vec3::new(0.0, 10.0 * cs, 0.4 * cs),
            5.5 * cs,
            5.0 * cs,
            cub_col,
        );
        // Cub ears
        fur.beam(
            cub_pos + Vec3::new(-2.6 * cs, 12.5 * cs, 0.0),
            cub_pos + Vec3::new(-2.6 * cs, 13.5 * cs, 0.0),
            1.8 * cs,
            1.5 * cs,
            cub_col,
        );
        fur.beam(
            cub_pos + Vec3::new(2.6 * cs, 12.5 * cs, 0.0),
            cub_pos + Vec3::new(2.6 * cs, 13.5 * cs, 0.0),
            1.8 * cs,
            1.5 * cs,
            cub_col,
        );
        // Cub seated paws
        fur.beam(
            cub_pos + Vec3::new(-2.5 * cs, 1.5 * cs, 0.8 * cs),
            cub_pos + Vec3::new(-2.5 * cs, 1.5 * cs, 5.0 * cs),
            2.6 * cs,
            2.4 * cs,
            cub_col,
        );
        fur.beam(
            cub_pos + Vec3::new(2.5 * cs, 1.5 * cs, 0.8 * cs),
            cub_pos + Vec3::new(2.5 * cs, 1.5 * cs, 5.0 * cs),
            2.6 * cs,
            2.4 * cs,
            cub_col,
        );
        // Cub muzzle
        muzzle.beam(
            cub_pos + Vec3::new(0.0, 9.2 * cs, 1.8 * cs),
            cub_pos + Vec3::new(0.0, 9.2 * cs, 2.8 * cs),
            2.2 * cs,
            1.8 * cs,
            TEDDY_MUZZLE,
        );
    }

    world.spawn((
        Mesh3d(assets.meshes.add(fur.build())),
        MeshMaterial3d(fur_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(muzzle.build())),
        MeshMaterial3d(muzzle_mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 6. Toy Train
// ---------------------------------------------------------------------------

fn spawn_toy_train(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    train: &ToyTrain,
    wood_mat: &Handle<StandardMaterial>,
    vertex_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(train.center);
    let mut rails = MeshBuilder::default();
    let r = train.radius;
    let segs = 32;

    // Wooden sleepers and dual metal rails in a circular layout.
    for i in 0..segs {
        let a0 = i as f32 * TAU / segs as f32;
        let a1 = (i + 1) as f32 * TAU / segs as f32;

        let p0_in = Vec3::new((r - 1.4) * a0.cos(), 0.15, (r - 1.4) * a0.sin());
        let p1_in = Vec3::new((r - 1.4) * a1.cos(), 0.15, (r - 1.4) * a1.sin());
        let p0_out = Vec3::new((r + 1.4) * a0.cos(), 0.15, (r + 1.4) * a0.sin());
        let p1_out = Vec3::new((r + 1.4) * a1.cos(), 0.15, (r + 1.4) * a1.sin());

        // Metal rails.
        rails.beam(p0_in, p1_in, 0.3, 0.25, Color::srgb(0.7, 0.72, 0.75));
        rails.beam(p0_out, p1_out, 0.3, 0.25, Color::srgb(0.7, 0.72, 0.75));

        // Wooden cross tie (sleeper).
        let p_sleeper_in = Vec3::new((r - 2.2) * a0.cos(), 0.05, (r - 2.2) * a0.sin());
        let p_sleeper_out = Vec3::new((r + 2.2) * a0.cos(), 0.05, (r + 2.2) * a0.sin());
        rails.beam(p_sleeper_in, p_sleeper_out, 0.8, 0.2, WOOD_RAIL);
    }

    // Classic turned-wood toy trees around the railway set (media_1792447348822.jpg).
    let tree_angles: [f32; 6] = [0.4, 1.2, 2.1, 3.2, 4.2, 5.3];
    for (idx, &a) in tree_angles.iter().enumerate() {
        let tree_r = if idx % 2 == 0 { r - 12.0 } else { r + 12.0 };
        let tree_pos = Vec3::new(tree_r * a.cos(), 0.1, tree_r * a.sin());
        // Turned wooden trunk
        cylinder(
            &mut rails,
            tree_pos,
            tree_pos + Vec3::Y * 2.8,
            0.6,
            0.5,
            8,
            WOOD_RAIL,
        );
        // Stacked green conical tree tiers
        cylinder(
            &mut rails,
            tree_pos + Vec3::Y * 2.2,
            tree_pos + Vec3::Y * 4.6,
            2.8,
            1.8,
            10,
            TOY_GREEN,
        );
        cylinder(
            &mut rails,
            tree_pos + Vec3::Y * 4.2,
            tree_pos + Vec3::Y * 6.6,
            1.9,
            0.8,
            10,
            TOY_GREEN,
        );
        cylinder(
            &mut rails,
            tree_pos + Vec3::Y * 6.2,
            tree_pos + Vec3::Y * 8.2,
            1.0,
            0.08,
            10,
            TOY_GREEN,
        );
    }

    // Complete Toy Steam Train Composition (Locomotive + Tender + Coach + Caboose).
    let mut train_cars = MeshBuilder::default();

    // 1. Steam Locomotive at angle 0.0
    let loc_pos = Vec3::new(r, 0.4, 0.0);
    // Boiler cylinder
    train_cars.beam(
        loc_pos + Vec3::new(0.0, 1.8, -4.0),
        loc_pos + Vec3::new(0.0, 1.8, 2.0),
        2.6,
        2.6,
        TOY_BLACK,
    );
    // Cabin
    train_cars.beam(
        loc_pos + Vec3::new(0.0, 2.5, 2.0),
        loc_pos + Vec3::new(0.0, 2.5, 5.5),
        3.2,
        4.0,
        TOY_RED,
    );
    // Smokestack chimney
    train_cars.beam(
        loc_pos + Vec3::new(0.0, 3.0, -3.0),
        loc_pos + Vec3::new(0.0, 5.0, -3.0),
        0.8,
        0.8,
        TOY_YELLOW,
    );
    // Brass steam dome
    cylinder(
        &mut train_cars,
        loc_pos + Vec3::new(0.0, 3.0, -0.5),
        loc_pos + Vec3::new(0.0, 4.2, -0.5),
        0.7,
        0.5,
        10,
        BATTERY_GOLD,
    );
    // Front cowcatcher
    train_cars.beam(
        loc_pos + Vec3::new(0.0, 0.8, -4.8),
        loc_pos + Vec3::new(0.0, 0.8, -4.0),
        2.8,
        1.2,
        TOY_YELLOW,
    );
    // Locomotive red wheels
    for wz in [-2.5, 0.0, 2.5, 4.5] {
        cylinder(
            &mut train_cars,
            loc_pos + Vec3::new(-1.6, 0.9, wz),
            loc_pos + Vec3::new(-1.4, 0.9, wz),
            0.9,
            0.9,
            10,
            TOY_RED,
        );
        cylinder(
            &mut train_cars,
            loc_pos + Vec3::new(1.4, 0.9, wz),
            loc_pos + Vec3::new(1.6, 0.9, wz),
            0.9,
            0.9,
            10,
            TOY_RED,
        );
    }

    // 2. Coal Tender at angle -0.16 rad
    let a_tender = -0.16;
    let rot_tender = Quat::from_rotation_y(-a_tender);
    let tender_pos = Vec3::new(r * a_tender.cos(), 0.4, r * a_tender.sin());
    // Tender box body
    train_cars.beam(
        tender_pos + rot_tender * Vec3::new(0.0, 1.4, -2.5),
        tender_pos + rot_tender * Vec3::new(0.0, 1.4, 2.5),
        2.8,
        2.2,
        TOY_BLACK,
    );
    // Coal mound inside
    train_cars.beam(
        tender_pos + rot_tender * Vec3::new(0.0, 2.3, -1.8),
        tender_pos + rot_tender * Vec3::new(0.0, 2.3, 1.8),
        2.2,
        1.1,
        Color::srgb(0.08, 0.08, 0.08),
    );
    // Wheels
    for wz in [-1.5, 1.5] {
        cylinder(
            &mut train_cars,
            tender_pos + rot_tender * Vec3::new(-1.5, 0.7, wz),
            tender_pos + rot_tender * Vec3::new(-1.3, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
        cylinder(
            &mut train_cars,
            tender_pos + rot_tender * Vec3::new(1.3, 0.7, wz),
            tender_pos + rot_tender * Vec3::new(1.5, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
    }

    // 3. Red Passenger Coach at angle -0.34 rad
    let a_coach = -0.34;
    let rot_coach = Quat::from_rotation_y(-a_coach);
    let coach_pos = Vec3::new(r * a_coach.cos(), 0.4, r * a_coach.sin());
    // Coach body
    train_cars.beam(
        coach_pos + rot_coach * Vec3::new(0.0, 1.8, -3.8),
        coach_pos + rot_coach * Vec3::new(0.0, 1.8, 3.8),
        3.0,
        3.2,
        TOY_RED,
    );
    // Roof
    train_cars.beam(
        coach_pos + rot_coach * Vec3::new(0.0, 3.5, -4.0),
        coach_pos + rot_coach * Vec3::new(0.0, 3.5, 4.0),
        3.2,
        0.6,
        Color::srgb(0.96, 0.94, 0.88),
    );
    // Yellow window stripe
    train_cars.beam(
        coach_pos + rot_coach * Vec3::new(0.0, 2.2, -3.5),
        coach_pos + rot_coach * Vec3::new(0.0, 2.2, 3.5),
        3.05,
        0.7,
        TOY_YELLOW,
    );
    // Coach wheels
    for wz in [-2.5, 2.5] {
        cylinder(
            &mut train_cars,
            coach_pos + rot_coach * Vec3::new(-1.5, 0.7, wz),
            coach_pos + rot_coach * Vec3::new(-1.3, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
        cylinder(
            &mut train_cars,
            coach_pos + rot_coach * Vec3::new(1.3, 0.7, wz),
            coach_pos + rot_coach * Vec3::new(1.5, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
    }

    // 4. Green Caboose / Freight Wagon at angle -0.52 rad
    let a_caboose = -0.52;
    let rot_cab = Quat::from_rotation_y(-a_caboose);
    let cab_pos = Vec3::new(r * a_caboose.cos(), 0.4, r * a_caboose.sin());
    // Caboose body
    train_cars.beam(
        cab_pos + rot_cab * Vec3::new(0.0, 1.8, -3.2),
        cab_pos + rot_cab * Vec3::new(0.0, 1.8, 3.2),
        3.0,
        3.2,
        TOY_GREEN,
    );
    // Cupola observation roof
    train_cars.beam(
        cab_pos + rot_cab * Vec3::new(0.0, 3.8, -1.2),
        cab_pos + rot_cab * Vec3::new(0.0, 3.8, 1.2),
        2.2,
        1.2,
        TOY_RED,
    );
    // Wheels
    for wz in [-2.0, 2.0] {
        cylinder(
            &mut train_cars,
            cab_pos + rot_cab * Vec3::new(-1.5, 0.7, wz),
            cab_pos + rot_cab * Vec3::new(-1.3, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
        cylinder(
            &mut train_cars,
            cab_pos + rot_cab * Vec3::new(1.3, 0.7, wz),
            cab_pos + rot_cab * Vec3::new(1.5, 0.7, wz),
            0.7,
            0.7,
            8,
            Color::srgb(0.15, 0.15, 0.15),
        );
    }

    // Level Crossing Barrier Gates ("Paso a Nivel", media_1792447348822.jpg).
    // Placed at the railway crossing road approach zones (local to train center).
    let crossing_gates = [
        Vec3::new(-72.0, 0.0, 85.0), // Entry crossing barrier
        Vec3::new(75.0, 0.0, 85.0),  // Exit crossing barrier
    ];
    for gate_pos in crossing_gates {
        // Vertical stanchion post with alternating black & white stripes
        for s in 0..6 {
            let col = if s % 2 == 0 { TOY_BLACK } else { TOY_WHITE };
            train_cars.beam(
                gate_pos + Vec3::Y * (s as f32 * 1.0),
                gate_pos + Vec3::Y * ((s + 1) as f32 * 1.0),
                0.6,
                0.6,
                col,
            );
        }
        // Crossbuck X sign at top
        train_cars.beam(
            gate_pos + Vec3::new(-1.4, 5.5, 0.0),
            gate_pos + Vec3::new(1.4, 6.5, 0.0),
            0.4,
            0.15,
            TOY_WHITE,
        );
        train_cars.beam(
            gate_pos + Vec3::new(-1.4, 6.5, 0.0),
            gate_pos + Vec3::new(1.4, 5.5, 0.0),
            0.4,
            0.15,
            TOY_WHITE,
        );
        // Flashing red warning light box
        train_cars.beam(
            gate_pos + Vec3::new(-1.0, 4.8, 0.0),
            gate_pos + Vec3::new(1.0, 4.8, 0.0),
            0.8,
            0.6,
            Color::srgb(0.1, 0.1, 0.1),
        );
        // Dual red signal lamps
        cylinder(
            &mut train_cars,
            gate_pos + Vec3::new(-0.7, 4.8, 0.35),
            gate_pos + Vec3::new(-0.7, 4.8, 0.5),
            0.35,
            0.35,
            8,
            Color::srgb(1.0, 0.08, 0.08),
        );
        cylinder(
            &mut train_cars,
            gate_pos + Vec3::new(0.7, 4.8, 0.35),
            gate_pos + Vec3::new(0.7, 4.8, 0.5),
            0.35,
            0.35,
            8,
            Color::srgb(1.0, 0.08, 0.08),
        );

        // Striped red-and-white horizontal barrier boom gate
        let boom_len = 10.0;
        let boom_segs = 8;
        for b in 0..boom_segs {
            let col = if b % 2 == 0 { TOY_RED } else { TOY_WHITE };
            let x0 = gate_pos.x + (b as f32 / boom_segs as f32) * boom_len;
            let x1 = gate_pos.x + ((b + 1) as f32 / boom_segs as f32) * boom_len;
            train_cars.beam(
                Vec3::new(x0, 2.5, gate_pos.z),
                Vec3::new(x1, 2.5, gate_pos.z),
                0.3,
                0.4,
                col,
            );
        }
        // Counterweight block
        train_cars.beam(
            gate_pos + Vec3::new(-1.4, 2.5, 0.0),
            gate_pos + Vec3::new(-0.3, 2.5, 0.0),
            0.9,
            0.9,
            Color::srgb(0.2, 0.2, 0.2),
        );
    }

    let trans = Transform::from_xyz(train.center.x, ground, train.center.y);
    world.spawn((
        Mesh3d(assets.meshes.add(rails.build())),
        MeshMaterial3d(wood_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(train_cars.build())),
        MeshMaterial3d(vertex_mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 7. Building Block Tower
// ---------------------------------------------------------------------------

fn spawn_block_tower(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    tower: &BlockTower,
    block_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(tower.position);
    let mut mesh = MeshBuilder::default();

    // Stacked tiers of interlocking plastic blocks.
    let tiers = (tower.height / 3.2).clamp(2.0, 12.0) as usize;
    let mut current_h = 0.0;
    for tier in 0..tiers {
        let size = 8.0 - tier as f32 * 0.4;
        let block_h = 3.0;
        let col = if tier % 2 == 0 {
            tower.color_primary
        } else {
            tower.color_secondary
        };

        // Main block cuboid.
        mesh.beam(
            Vec3::new(0.0, current_h + block_h / 2.0, -size / 2.0),
            Vec3::new(0.0, current_h + block_h / 2.0, size / 2.0),
            size,
            block_h,
            col,
        );

        // Cylindrical studs on top.
        for sx in [-0.3, 0.3] {
            for sz in [-0.3, 0.3] {
                let px = sx * size;
                let pz = sz * size;
                mesh.beam(
                    Vec3::new(px, current_h + block_h, pz),
                    Vec3::new(px, current_h + block_h + 0.6, pz),
                    0.9,
                    0.9,
                    col,
                );
            }
        }
        current_h += block_h + 0.6;
    }

    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(block_mat.clone()),
        Transform::from_xyz(tower.position.x, ground, tower.position.y),
    ));
}

// ---------------------------------------------------------------------------
// 8. Die (Game Cube)
// ---------------------------------------------------------------------------

fn spawn_die(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    die: &Die,
    body_mat: &Handle<StandardMaterial>,
    pip_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(die.position);
    let s = die.size;
    let rot = Quat::from_rotation_y(die.heading);

    let mut body = MeshBuilder::default();
    body.beam(
        Vec3::new(0.0, s / 2.0, -s / 2.0),
        Vec3::new(0.0, s / 2.0, s / 2.0),
        s,
        s,
        TOY_WHITE,
    );

    let mut pips = MeshBuilder::default();
    let top_y = s + 0.05;
    let roll = die.roll.clamp(1, 6);
    let mut positions = Vec::new();
    if roll % 2 == 1 {
        positions.push(Vec2::ZERO);
    }
    if roll >= 2 {
        positions.push(Vec2::new(-0.25, -0.25));
        positions.push(Vec2::new(0.25, 0.25));
    }
    if roll >= 4 {
        positions.push(Vec2::new(-0.25, 0.25));
        positions.push(Vec2::new(0.25, -0.25));
    }
    if roll == 6 {
        positions.push(Vec2::new(-0.25, 0.0));
        positions.push(Vec2::new(0.25, 0.0));
    }

    for p in positions {
        pips.beam(
            Vec3::new(p.x * s, top_y, p.y * s),
            Vec3::new(p.x * s, top_y + 0.1, p.y * s),
            0.18 * s,
            0.18 * s,
            TOY_BLACK,
        );
    }

    let trans = Transform::from_xyz(die.position.x, ground, die.position.y).with_rotation(rot);
    world.spawn((
        Mesh3d(assets.meshes.add(body.build())),
        MeshMaterial3d(body_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(pips.build())),
        MeshMaterial3d(pip_mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 9. Card House
// ---------------------------------------------------------------------------

fn spawn_card_house(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    house: &CardHouse,
    card_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(house.position);
    let rot = Quat::from_rotation_y(house.heading);
    let mut mesh = MeshBuilder::default();

    let stories = house.stories.clamp(1, 4);
    let card_w = 4.5;
    let card_h = 7.0;
    let lean = 1.4;

    for story in 0..stories {
        let base_y = story as f32 * (card_h * 0.85);

        // Leaning card pair tent.
        mesh.quad(
            [
                (Vec3::new(-lean, base_y, -card_w / 2.0), Vec2::ZERO),
                (Vec3::new(-lean, base_y, card_w / 2.0), Vec2::X),
                (
                    Vec3::new(0.0, base_y + card_h * 0.85, card_w / 2.0),
                    Vec2::ONE,
                ),
                (
                    Vec3::new(0.0, base_y + card_h * 0.85, -card_w / 2.0),
                    Vec2::Y,
                ),
            ],
            Vec3::new(-1.0, 1.0, 0.0),
            TOY_WHITE,
        );
        mesh.quad(
            [
                (Vec3::new(lean, base_y, -card_w / 2.0), Vec2::ZERO),
                (Vec3::new(lean, base_y, card_w / 2.0), Vec2::X),
                (
                    Vec3::new(0.0, base_y + card_h * 0.85, card_w / 2.0),
                    Vec2::ONE,
                ),
                (
                    Vec3::new(0.0, base_y + card_h * 0.85, -card_w / 2.0),
                    Vec2::Y,
                ),
            ],
            Vec3::new(1.0, 1.0, 0.0),
            TOY_RED,
        );
        // Floor divider card.
        mesh.beam(
            Vec3::new(-lean * 1.2, base_y + card_h * 0.85, 0.0),
            Vec3::new(lean * 1.2, base_y + card_h * 0.85, 0.0),
            card_w,
            0.15,
            TOY_WHITE,
        );
    }

    world.spawn((
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(card_mat.clone()),
        Transform::from_xyz(house.position.x, ground, house.position.y).with_rotation(rot),
    ));
}

// ---------------------------------------------------------------------------
// 10. Ceiling Lights
// ---------------------------------------------------------------------------

fn spawn_ceiling_light(
    world: &mut ChildSpawnerCommands,
    light: &CeilingLight,
    fixture_mat: &Handle<StandardMaterial>,
    glow_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let mut fixture = MeshBuilder::default();
    let mut bulb = MeshBuilder::default();

    // Overhead mounting beam / rail fixture.
    fixture.beam(
        Vec3::new(-6.0, light.height, 0.0),
        Vec3::new(6.0, light.height, 0.0),
        0.6,
        0.6,
        TOY_BLACK,
    );
    // Downward pendant lamp cones.
    let bulb_color = if light.warm {
        Color::srgb(1.0, 0.94, 0.85)
    } else {
        Color::srgb(0.92, 0.96, 1.0)
    };
    for x in [-3.5, 0.0, 3.5] {
        fixture.beam(
            Vec3::new(x, light.height, 0.0),
            Vec3::new(x, light.height - 1.8, 0.0),
            1.6,
            1.8,
            TOY_BLACK,
        );
        bulb.beam(
            Vec3::new(x, light.height - 1.85, 0.0),
            Vec3::new(x, light.height - 1.9, 0.0),
            1.3,
            0.2,
            bulb_color,
        );
    }

    let trans = Transform::from_xyz(light.position.x, 0.0, light.position.y);
    world.spawn((
        Mesh3d(assets.meshes.add(fixture.build())),
        MeshMaterial3d(fixture_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(bulb.build())),
        MeshMaterial3d(glow_mat.clone()),
        trans,
        NotShadowCaster,
    ));
}

// ---------------------------------------------------------------------------
// 11. Toy Shelf
// ---------------------------------------------------------------------------

fn spawn_toy_shelf(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    shelf: &ToyShelf,
    wood_mat: &Handle<StandardMaterial>,
    box_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(shelf.center);
    let rot = Quat::from_rotation_y(shelf.heading);
    let trans = Transform::from_xyz(shelf.center.x, ground, shelf.center.y).with_rotation(rot);

    let mut bookcase = MeshBuilder::default();
    let l = shelf.length;
    let h = shelf.height;
    let d = 5.0;

    // Upright pillars at ends and middle.
    bookcase.beam(
        Vec3::new(-l / 2.0, h / 2.0, 0.0),
        Vec3::new(-l / 2.0, h / 2.0, 0.0),
        d,
        h,
        WOOD_RAIL,
    );
    bookcase.beam(
        Vec3::new(l / 2.0, h / 2.0, 0.0),
        Vec3::new(l / 2.0, h / 2.0, 0.0),
        d,
        h,
        WOOD_RAIL,
    );

    // Horizontal shelves.
    let shelf_count = 4;
    for s in 0..=shelf_count {
        let sy = s as f32 * (h / shelf_count as f32);
        bookcase.beam(
            Vec3::new(-l / 2.0, sy, 0.0),
            Vec3::new(l / 2.0, sy, 0.0),
            d,
            0.6,
            WOOD_RAIL,
        );
    }

    // Shelf contents: real toys (mini teddy bears, wooden cars, alphabet blocks,
    // stacking rings) alternating with colorful illustrated packaging boxes.
    let mut toys = MeshBuilder::default();
    for s in 0..shelf_count {
        let sy = s as f32 * (h / shelf_count as f32) + 0.3;
        let count_x = (l / 5.5).floor() as usize;
        for bx in 0..count_x {
            let cx = -l / 2.0 + 3.0 + bx as f32 * 5.2;
            let slot = (s + bx) % 5;
            match slot {
                0 => {
                    // Row of colorful toy packages with clear display windows.
                    let col = [TOY_RED, TOY_BLUE, TOY_YELLOW, TOY_GREEN, TOY_ORANGE][bx % 5];
                    toys.beam(
                        Vec3::new(cx, sy + 1.8, -0.8),
                        Vec3::new(cx, sy + 1.8, 0.8),
                        4.2,
                        3.2,
                        col,
                    );
                    // Clear window front.
                    toys.quad(
                        [
                            (Vec3::new(cx - 1.6, sy + 0.6, 0.85), Vec2::ZERO),
                            (Vec3::new(cx + 1.6, sy + 0.6, 0.85), Vec2::X),
                            (Vec3::new(cx + 1.6, sy + 3.0, 0.85), Vec2::ONE),
                            (Vec3::new(cx - 1.6, sy + 3.0, 0.85), Vec2::Y),
                        ],
                        Vec3::Z,
                        TOY_WHITE,
                    );
                }
                1 => {
                    // Miniature teddy bear sitting on shelf.
                    let ts = 0.35;
                    toys.beam(
                        Vec3::new(cx, sy + 1.0 * ts, 0.0),
                        Vec3::new(cx, sy + 5.0 * ts, 0.0),
                        3.0 * ts,
                        4.0 * ts,
                        TEDDY_BROWN,
                    );
                    // Head.
                    toys.beam(
                        Vec3::new(cx, sy + 7.0 * ts, 0.0),
                        Vec3::new(cx, sy + 9.5 * ts, 0.0),
                        2.8 * ts,
                        2.5 * ts,
                        TEDDY_BROWN,
                    );
                    // Muzzle.
                    toys.beam(
                        Vec3::new(cx, sy + 6.8 * ts, 1.2 * ts),
                        Vec3::new(cx, sy + 7.8 * ts, 1.2 * ts),
                        1.4 * ts,
                        1.0 * ts,
                        TEDDY_MUZZLE,
                    );
                }
                2 => {
                    // Wooden toy car parked on shelf.
                    toys.beam(
                        Vec3::new(cx - 1.2, sy + 0.8, 0.0),
                        Vec3::new(cx + 1.2, sy + 0.8, 0.0),
                        1.8,
                        1.2,
                        [TOY_RED, TOY_BLUE, TOY_GREEN][bx % 3],
                    );
                    // Wheels.
                    cylinder(
                        &mut toys,
                        Vec3::new(cx - 1.0, sy + 0.4, 0.9),
                        Vec3::new(cx - 1.0, sy + 0.4, 1.1),
                        0.45,
                        0.45,
                        8,
                        TOY_YELLOW,
                    );
                    cylinder(
                        &mut toys,
                        Vec3::new(cx + 1.0, sy + 0.4, 0.9),
                        Vec3::new(cx + 1.0, sy + 0.4, 1.1),
                        0.45,
                        0.45,
                        8,
                        TOY_YELLOW,
                    );
                }
                3 => {
                    // Stack of primary-colored alphabet blocks.
                    toys.beam(
                        Vec3::new(cx - 0.8, sy + 0.8, 0.0),
                        Vec3::new(cx + 0.8, sy + 0.8, 0.0),
                        1.5,
                        1.5,
                        TOY_YELLOW,
                    );
                    toys.beam(
                        Vec3::new(cx - 0.7, sy + 2.2, 0.0),
                        Vec3::new(cx + 0.7, sy + 2.2, 0.0),
                        1.4,
                        1.4,
                        TOY_RED,
                    );
                }
                _ => {
                    // Miniature stacking ring pyramid.
                    cylinder(
                        &mut toys,
                        Vec3::new(cx, sy + 0.2, 0.0),
                        Vec3::new(cx, sy + 0.8, 0.0),
                        1.2,
                        1.2,
                        10,
                        TOY_RED,
                    );
                    cylinder(
                        &mut toys,
                        Vec3::new(cx, sy + 0.8, 0.0),
                        Vec3::new(cx, sy + 1.4, 0.0),
                        0.9,
                        0.9,
                        10,
                        TOY_YELLOW,
                    );
                    cylinder(
                        &mut toys,
                        Vec3::new(cx, sy + 1.4, 0.0),
                        Vec3::new(cx, sy + 2.0, 0.0),
                        0.6,
                        0.6,
                        10,
                        TOY_BLUE,
                    );
                }
            }
        }
    }

    world.spawn((
        Mesh3d(assets.meshes.add(bookcase.build())),
        MeshMaterial3d(wood_mat.clone()),
        trans,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(toys.build())),
        MeshMaterial3d(box_mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// Mesh Helper: Cylinders & Frustums
// ---------------------------------------------------------------------------

fn cylinder(
    builder: &mut MeshBuilder,
    base: Vec3,
    top: Vec3,
    r_base: f32,
    r_top: f32,
    segments: usize,
    color: Color,
) {
    if r_base <= 0.0 && r_top <= 0.0 {
        return;
    }
    for i in 0..segments {
        let a0 = i as f32 * TAU / segments as f32;
        let a1 = (i + 1) as f32 * TAU / segments as f32;
        let (c0, s0) = (a0.cos(), a0.sin());
        let (c1, s1) = (a1.cos(), a1.sin());

        let p0 = base + Vec3::new(c0 * r_base, 0.0, s0 * r_base);
        let p1 = base + Vec3::new(c1 * r_base, 0.0, s1 * r_base);
        let p2 = top + Vec3::new(c1 * r_top, 0.0, s1 * r_top);
        let p3 = top + Vec3::new(c0 * r_top, 0.0, s0 * r_top);
        let normal = Vec3::new(c0 + c1, 0.0, s0 + s1).normalize_or_zero();

        builder.quad(
            [
                (p0, Vec2::ZERO),
                (p1, Vec2::X),
                (p2, Vec2::ONE),
                (p3, Vec2::Y),
            ],
            normal,
            color,
        );

        if r_top > 0.0 {
            builder.triangle(
                [(top, Vec2::ZERO), (p2, Vec2::X), (p3, Vec2::Y)],
                Vec3::Y,
                color,
            );
        }
        if r_base > 0.0 {
            builder.triangle(
                [(base, Vec2::ZERO), (p0, Vec2::X), (p1, Vec2::Y)],
                -Vec3::Y,
                color,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Royal Guard Nutcracker Soldier
// ---------------------------------------------------------------------------

pub fn make_nutcracker_mesh(height: f32) -> Mesh {
    let mut m = MeshBuilder::default();
    let s = height / 12.0;

    // 1. Turned wood base pedestal.
    cylinder(
        &mut m,
        Vec3::ZERO,
        Vec3::new(0.0, 0.4 * s, 0.0),
        1.8 * s,
        1.6 * s,
        12,
        TOY_BLACK,
    );

    // 2. Black boots with gold top trim.
    for leg_x in [-0.6 * s, 0.6 * s] {
        cylinder(
            &mut m,
            Vec3::new(leg_x, 0.4 * s, 0.0),
            Vec3::new(leg_x, 2.6 * s, 0.0),
            0.55 * s,
            0.55 * s,
            10,
            TOY_BLACK,
        );
        cylinder(
            &mut m,
            Vec3::new(leg_x, 2.6 * s, 0.0),
            Vec3::new(leg_x, 2.8 * s, 0.0),
            0.58 * s,
            0.58 * s,
            10,
            BATTERY_GOLD,
        );
    }

    // 3. White breeches / legs.
    for leg_x in [-0.6 * s, 0.6 * s] {
        cylinder(
            &mut m,
            Vec3::new(leg_x, 2.8 * s, 0.0),
            Vec3::new(leg_x, 4.5 * s, 0.0),
            0.55 * s,
            0.58 * s,
            10,
            TOY_WHITE,
        );
    }

    // 4. Black belt with gold buckle.
    cylinder(
        &mut m,
        Vec3::new(0.0, 4.5 * s, 0.0),
        Vec3::new(0.0, 4.9 * s, 0.0),
        1.25 * s,
        1.25 * s,
        12,
        TOY_BLACK,
    );
    m.beam(
        Vec3::new(-0.35 * s, 4.7 * s, 1.22 * s),
        Vec3::new(0.35 * s, 4.7 * s, 1.22 * s),
        0.5 * s,
        0.4 * s,
        BATTERY_GOLD,
    );

    // 5. Red uniform tunic with gold frogging and epaulets.
    cylinder(
        &mut m,
        Vec3::new(0.0, 4.9 * s, 0.0),
        Vec3::new(0.0, 7.2 * s, 0.0),
        1.20 * s,
        1.35 * s,
        12,
        TOY_RED,
    );
    // Gold buttons and chest frogging.
    for by in [5.4 * s, 6.0 * s, 6.6 * s] {
        m.beam(
            Vec3::new(-0.6 * s, by, 1.28 * s),
            Vec3::new(0.6 * s, by, 1.28 * s),
            0.08 * s,
            0.12 * s,
            BATTERY_GOLD,
        );
    }
    // Gold epaulets.
    for ex in [-1.4 * s, 1.4 * s] {
        m.beam(
            Vec3::new(ex, 7.1 * s, -0.4 * s),
            Vec3::new(ex, 7.1 * s, 0.4 * s),
            0.5 * s,
            0.2 * s,
            BATTERY_GOLD,
        );
    }

    // 6. Arms and hands.
    for (ax, is_right) in [(-1.35 * s, false), (1.35 * s, true)] {
        cylinder(
            &mut m,
            Vec3::new(ax, 5.0 * s, 0.0),
            Vec3::new(ax, 7.0 * s, 0.0),
            0.4 * s,
            0.45 * s,
            8,
            TOY_RED,
        );
        cylinder(
            &mut m,
            Vec3::new(ax, 4.6 * s, 0.0),
            Vec3::new(ax, 5.0 * s, 0.0),
            0.35 * s,
            0.4 * s,
            8,
            TOY_WHITE,
        );
        if is_right {
            // Ceremonial halberd / sword in right hand.
            m.beam(
                Vec3::new(ax + 0.2 * s, 0.0, 0.4 * s),
                Vec3::new(ax + 0.2 * s, 10.5 * s, 0.4 * s),
                0.12 * s,
                0.12 * s,
                BATTERY_GOLD,
            );
            // Spearhead blade.
            m.beam(
                Vec3::new(ax + 0.2 * s, 10.5 * s, 0.4 * s),
                Vec3::new(ax + 0.2 * s, 11.5 * s, 0.4 * s),
                0.4 * s,
                0.08 * s,
                BATTERY_GOLD,
            );
        }
    }

    // 7. Face, collar, mustache, and beard.
    cylinder(
        &mut m,
        Vec3::new(0.0, 7.2 * s, 0.0),
        Vec3::new(0.0, 7.5 * s, 0.0),
        0.9 * s,
        0.95 * s,
        12,
        TOY_WHITE,
    );
    let skin_col = Color::srgb(0.96, 0.80, 0.70);
    cylinder(
        &mut m,
        Vec3::new(0.0, 7.5 * s, 0.0),
        Vec3::new(0.0, 8.8 * s, 0.0),
        1.05 * s,
        1.10 * s,
        12,
        skin_col,
    );
    // Mustache & rosy cheeks.
    m.beam(
        Vec3::new(-0.6 * s, 7.9 * s, 1.05 * s),
        Vec3::new(0.6 * s, 7.9 * s, 1.05 * s),
        0.18 * s,
        0.16 * s,
        TOY_WHITE,
    );
    // White beard chin.
    m.beam(
        Vec3::new(-0.35 * s, 7.4 * s, 0.95 * s),
        Vec3::new(0.35 * s, 7.4 * s, 0.95 * s),
        0.3 * s,
        0.25 * s,
        TOY_WHITE,
    );
    // Eyes.
    for eye_x in [-0.35 * s, 0.35 * s] {
        m.beam(
            Vec3::new(eye_x, 8.3 * s, 1.06 * s),
            Vec3::new(eye_x, 8.4 * s, 1.06 * s),
            0.15 * s,
            0.12 * s,
            TOY_BLACK,
        );
    }

    // 8. Tall black shako hat with golden visor and chin strap.
    cylinder(
        &mut m,
        Vec3::new(0.0, 8.8 * s, 0.0),
        Vec3::new(0.0, 11.5 * s, 0.0),
        1.15 * s,
        1.25 * s,
        12,
        TOY_BLACK,
    );
    // Gold visor.
    m.beam(
        Vec3::new(-0.8 * s, 8.85 * s, 0.8 * s),
        Vec3::new(0.8 * s, 8.85 * s, 0.8 * s),
        0.4 * s,
        0.12 * s,
        BATTERY_GOLD,
    );
    // Gold chin strap.
    cylinder(
        &mut m,
        Vec3::new(0.0, 8.75 * s, 0.0),
        Vec3::new(0.0, 8.95 * s, 0.0),
        1.18 * s,
        1.18 * s,
        12,
        BATTERY_GOLD,
    );
    // Plume crest.
    cylinder(
        &mut m,
        Vec3::new(0.0, 11.5 * s, 0.0),
        Vec3::new(0.0, 12.5 * s, 0.0),
        0.3 * s,
        0.05 * s,
        8,
        TOY_RED,
    );

    m.build()
}

fn spawn_nutcracker(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    soldier: &Nutcracker,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(soldier.position);
    let rot = Quat::from_rotation_y(soldier.heading);
    let trans =
        Transform::from_xyz(soldier.position.x, ground, soldier.position.y).with_rotation(rot);
    let mesh = make_nutcracker_mesh(soldier.height);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 13. Russian Nesting Dolls (Matryoshka)
// ---------------------------------------------------------------------------

pub fn make_matryoshka_mesh(scale: f32) -> Mesh {
    let mut m = MeshBuilder::default();

    let dolls = [
        (Vec3::new(0.0, 0.0, 0.0), 1.0 * scale),
        (Vec3::new(2.8 * scale, 0.0, -0.6 * scale), 0.65 * scale),
        (Vec3::new(-2.2 * scale, 0.0, 0.8 * scale), 0.42 * scale),
    ];

    let apron_col = Color::srgb(0.96, 0.92, 0.82);
    let skin_col = Color::srgb(0.96, 0.82, 0.72);

    for (pos, s) in dolls {
        // Turned wooden circular base stand.
        cylinder(
            &mut m,
            pos,
            pos + Vec3::new(0.0, 0.3 * s, 0.0),
            1.6 * s,
            1.5 * s,
            12,
            WOOD_RAIL,
        );

        // Bulbous lower belly (red babushka dress).
        cylinder(
            &mut m,
            pos + Vec3::new(0.0, 0.3 * s, 0.0),
            pos + Vec3::new(0.0, 1.8 * s, 0.0),
            1.5 * s,
            2.2 * s,
            12,
            TOY_RED,
        );
        cylinder(
            &mut m,
            pos + Vec3::new(0.0, 1.8 * s, 0.0),
            pos + Vec3::new(0.0, 3.2 * s, 0.0),
            2.2 * s,
            1.4 * s,
            12,
            TOY_RED,
        );

        // Front decorative apron.
        m.beam(
            pos + Vec3::new(-0.9 * s, 1.8 * s, 1.8 * s),
            pos + Vec3::new(0.9 * s, 1.8 * s, 1.8 * s),
            0.15 * s,
            1.6 * s,
            apron_col,
        );
        // Rosette flower on apron.
        m.beam(
            pos + Vec3::new(-0.25 * s, 1.8 * s, 1.9 * s),
            pos + Vec3::new(0.25 * s, 1.8 * s, 1.9 * s),
            0.12 * s,
            0.5 * s,
            TOY_YELLOW,
        );

        // Head and babushka hood.
        cylinder(
            &mut m,
            pos + Vec3::new(0.0, 3.2 * s, 0.0),
            pos + Vec3::new(0.0, 4.4 * s, 0.0),
            1.4 * s,
            1.3 * s,
            12,
            TOY_RED,
        );
        // Dome top of head.
        cylinder(
            &mut m,
            pos + Vec3::new(0.0, 4.4 * s, 0.0),
            pos + Vec3::new(0.0, 5.0 * s, 0.0),
            1.3 * s,
            0.0,
            12,
            TOY_RED,
        );

        // Face aperture.
        cylinder(
            &mut m,
            pos + Vec3::new(0.0, 3.5 * s, 0.3 * s),
            pos + Vec3::new(0.0, 4.3 * s, 0.3 * s),
            0.85 * s,
            0.75 * s,
            10,
            skin_col,
        );

        // Painted hair & eyes.
        m.beam(
            pos + Vec3::new(-0.3 * s, 4.0 * s, 1.05 * s),
            pos + Vec3::new(0.3 * s, 4.0 * s, 1.05 * s),
            0.1 * s,
            0.15 * s,
            TOY_BLACK,
        );
        // Rosy red lips.
        m.beam(
            pos + Vec3::new(-0.15 * s, 3.65 * s, 1.05 * s),
            pos + Vec3::new(0.15 * s, 3.65 * s, 1.05 * s),
            0.08 * s,
            0.1 * s,
            TOY_RED,
        );
    }

    m.build()
}

fn spawn_matryoshka(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    matryoshka: &Matryoshka,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(matryoshka.position);
    let rot = Quat::from_rotation_y(matryoshka.heading);
    let trans = Transform::from_xyz(matryoshka.position.x, ground, matryoshka.position.y)
        .with_rotation(rot);
    let mesh = make_matryoshka_mesh(matryoshka.scale);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 14. Toy Sailboat
// ---------------------------------------------------------------------------

pub fn make_sailboat_mesh(length: f32, hull_color: Color) -> Mesh {
    let mut m = MeshBuilder::default();
    let l = length;
    let w = l * 0.40;
    let d = l * 0.22;

    // 1. Wooden stand cradle under the hull.
    m.beam(
        Vec3::new(-w * 0.4, 0.2, -l * 0.2),
        Vec3::new(w * 0.4, 0.2, -l * 0.2),
        0.5,
        0.4,
        WOOD_RAIL,
    );
    m.beam(
        Vec3::new(-w * 0.4, 0.2, l * 0.2),
        Vec3::new(w * 0.4, 0.2, l * 0.2),
        0.5,
        0.4,
        WOOD_RAIL,
    );

    // 2. Hull wedge:
    let stern_z = -l * 0.48;
    let bow_z = l * 0.52;
    let deck_y = d + 0.4;
    let keel_y = 0.4;

    // Keel to deck hull sides.
    m.quad(
        [
            (Vec3::new(-w / 2.0, deck_y, stern_z), Vec2::ZERO),
            (Vec3::new(w / 2.0, deck_y, stern_z), Vec2::X),
            (Vec3::new(w / 4.0, keel_y, stern_z), Vec2::ONE),
            (Vec3::new(-w / 4.0, keel_y, stern_z), Vec2::Y),
        ],
        -Vec3::Z,
        hull_color,
    );

    // Port hull side.
    m.quad(
        [
            (Vec3::new(-w / 2.0, deck_y, stern_z), Vec2::ZERO),
            (Vec3::new(-w / 4.0, keel_y, stern_z), Vec2::Y),
            (Vec3::new(0.0, keel_y, bow_z), Vec2::ONE),
            (Vec3::new(0.0, deck_y, bow_z), Vec2::X),
        ],
        -Vec3::X,
        hull_color,
    );

    // Starboard hull side.
    m.quad(
        [
            (Vec3::new(w / 2.0, deck_y, stern_z), Vec2::ZERO),
            (Vec3::new(0.0, deck_y, bow_z), Vec2::X),
            (Vec3::new(0.0, keel_y, bow_z), Vec2::ONE),
            (Vec3::new(w / 4.0, keel_y, stern_z), Vec2::Y),
        ],
        Vec3::X,
        hull_color,
    );

    // Teak wood deck.
    m.quad(
        [
            (Vec3::new(-w / 2.0, deck_y, stern_z), Vec2::ZERO),
            (Vec3::new(w / 2.0, deck_y, stern_z), Vec2::X),
            (Vec3::new(0.0, deck_y, bow_z), Vec2::ONE),
            (Vec3::new(-w / 2.0, deck_y, stern_z), Vec2::Y),
        ],
        Vec3::Y,
        WOOD_RAIL,
    );

    // Cabin deckhouse.
    m.beam(
        Vec3::new(0.0, deck_y + 0.5, stern_z * 0.4),
        Vec3::new(0.0, deck_y + 0.5, bow_z * 0.1),
        w * 0.45,
        0.8,
        TOY_WHITE,
    );

    // Mast.
    let mast_h = l * 1.35;
    m.beam(
        Vec3::new(0.0, deck_y, 0.0),
        Vec3::new(0.0, deck_y + mast_h, 0.0),
        0.35,
        0.35,
        WOOD_RAIL,
    );

    // Boom.
    m.beam(
        Vec3::new(0.0, deck_y + 1.2, 0.0),
        Vec3::new(0.0, deck_y + 1.2, stern_z * 0.95),
        0.25,
        0.25,
        WOOD_RAIL,
    );

    // Mainsail (triangle).
    m.triangle(
        [
            (Vec3::new(0.0, deck_y + mast_h * 0.95, 0.0), Vec2::ZERO),
            (Vec3::new(0.0, deck_y + 1.4, stern_z * 0.9), Vec2::X),
            (Vec3::new(0.0, deck_y + 1.4, 0.0), Vec2::Y),
        ],
        Vec3::X,
        TOY_WHITE,
    );
    m.triangle(
        [
            (Vec3::new(0.0, deck_y + mast_h * 0.95, 0.0), Vec2::ZERO),
            (Vec3::new(0.0, deck_y + 1.4, 0.0), Vec2::Y),
            (Vec3::new(0.0, deck_y + 1.4, stern_z * 0.9), Vec2::X),
        ],
        -Vec3::X,
        TOY_WHITE,
    );

    // Jib sail (triangle).
    m.triangle(
        [
            (Vec3::new(0.0, deck_y + mast_h * 0.85, 0.0), Vec2::ZERO),
            (Vec3::new(0.0, deck_y + 0.5, bow_z * 0.95), Vec2::X),
            (Vec3::new(0.0, deck_y + 0.5, bow_z * 0.1), Vec2::Y),
        ],
        Vec3::X,
        TOY_WHITE,
    );
    m.triangle(
        [
            (Vec3::new(0.0, deck_y + mast_h * 0.85, 0.0), Vec2::ZERO),
            (Vec3::new(0.0, deck_y + 0.5, bow_z * 0.1), Vec2::Y),
            (Vec3::new(0.0, deck_y + 0.5, bow_z * 0.95), Vec2::X),
        ],
        -Vec3::X,
        TOY_WHITE,
    );

    m.build()
}

fn spawn_toy_sailboat(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    boat: &ToySailboat,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(boat.position);
    let rot = Quat::from_rotation_y(boat.heading);
    let trans = Transform::from_xyz(boat.position.x, ground, boat.position.y).with_rotation(rot);
    let mesh = make_sailboat_mesh(boat.length, boat.hull_color);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 15. Rainbow Toy Xylophone
// ---------------------------------------------------------------------------

pub fn make_xylophone_mesh(length: f32) -> Mesh {
    let mut m = MeshBuilder::default();
    let l = length;
    let w_bass = l * 0.70;
    let w_treble = l * 0.40;

    // 1. Angled wooden side rails.
    let z_bass = -l / 2.0;
    let z_treble = l / 2.0;
    let frame_y = 1.0;

    // Left rail.
    m.beam(
        Vec3::new(-w_bass / 2.0, frame_y, z_bass),
        Vec3::new(-w_treble / 2.0, frame_y, z_treble),
        0.5,
        0.6,
        WOOD_RAIL,
    );
    // Right rail.
    m.beam(
        Vec3::new(w_bass / 2.0, frame_y, z_bass),
        Vec3::new(w_treble / 2.0, frame_y, z_treble),
        0.5,
        0.6,
        WOOD_RAIL,
    );

    // Four wooden stub legs.
    for (lx, lz) in [
        (-w_bass / 2.0, z_bass),
        (w_bass / 2.0, z_bass),
        (-w_treble / 2.0, z_treble),
        (w_treble / 2.0, z_treble),
    ] {
        cylinder(
            &mut m,
            Vec3::new(lx, 0.0, lz),
            Vec3::new(lx, frame_y, lz),
            0.35,
            0.35,
            8,
            WOOD_RAIL,
        );
    }

    // 2. Rainbow tone bars: 7 bars (Red, Orange, Yellow, Green, Cyan, Blue, Purple).
    let bar_colors = [
        TOY_RED,
        TOY_ORANGE,
        TOY_YELLOW,
        TOY_GREEN,
        Color::srgb(0.15, 0.75, 0.90),
        TOY_BLUE,
        Color::srgb(0.65, 0.20, 0.85),
    ];

    let n_bars = bar_colors.len();
    for (i, &col) in bar_colors.iter().enumerate() {
        let t = i as f32 / (n_bars - 1) as f32;
        let bz = z_bass + t * l;
        let bar_len = w_bass + t * (w_treble - w_bass) + 1.2;
        let by = frame_y + 0.4;

        // Tone bar beam.
        m.beam(
            Vec3::new(-bar_len / 2.0, by, bz),
            Vec3::new(bar_len / 2.0, by, bz),
            0.65,
            0.22,
            col,
        );

        // Mounting pins.
        let rail_w = w_bass + t * (w_treble - w_bass);
        for px in [-rail_w / 2.0, rail_w / 2.0] {
            cylinder(
                &mut m,
                Vec3::new(px, by + 0.1, bz),
                Vec3::new(px, by + 0.25, bz),
                0.12,
                0.12,
                6,
                SILVER_METAL,
            );
        }
    }

    // 3. Crossed wooden mallets resting across the treble end.
    m.beam(
        Vec3::new(-w_treble * 0.4, frame_y + 0.8, z_treble * 0.7),
        Vec3::new(w_treble * 0.4, frame_y + 0.8, z_treble * 0.2),
        0.14,
        0.14,
        WOOD_RAIL,
    );
    cylinder(
        &mut m,
        Vec3::new(w_treble * 0.4, frame_y + 0.8, z_treble * 0.2),
        Vec3::new(w_treble * 0.4 + 0.2, frame_y + 0.8, z_treble * 0.2),
        0.35,
        0.35,
        8,
        TOY_RED,
    );

    m.build()
}

fn spawn_xylophone(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    xylo: &Xylophone,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(xylo.position);
    let rot = Quat::from_rotation_y(xylo.heading);
    let trans = Transform::from_xyz(xylo.position.x, ground, xylo.position.y).with_rotation(rot);
    let mesh = make_xylophone_mesh(xylo.length);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 16. Wooden Stacking Ring Pyramid
// ---------------------------------------------------------------------------

pub fn make_stacking_pyramid_mesh(height: f32) -> Mesh {
    let mut m = MeshBuilder::default();
    let h = height;

    // Wooden base platter.
    cylinder(
        &mut m,
        Vec3::ZERO,
        Vec3::new(0.0, h * 0.08, 0.0),
        h * 0.45,
        h * 0.42,
        14,
        WOOD_RAIL,
    );

    // Central wooden peg.
    cylinder(
        &mut m,
        Vec3::new(0.0, h * 0.08, 0.0),
        Vec3::new(0.0, h * 0.88, 0.0),
        h * 0.07,
        h * 0.06,
        10,
        WOOD_RAIL,
    );

    // 5 graduated donut rings.
    let ring_colors = [TOY_RED, TOY_ORANGE, TOY_YELLOW, TOY_GREEN, TOY_BLUE];
    let ring_h = h * 0.15;
    let mut cur_y = h * 0.08;

    for (i, &col) in ring_colors.iter().enumerate() {
        let r = h * (0.40 - i as f32 * 0.06);
        cylinder(
            &mut m,
            Vec3::new(0.0, cur_y, 0.0),
            Vec3::new(0.0, cur_y + ring_h, 0.0),
            r,
            r,
            12,
            col,
        );
        cur_y += ring_h;
    }

    // Wooden sphere cap ball.
    cylinder(
        &mut m,
        Vec3::new(0.0, cur_y, 0.0),
        Vec3::new(0.0, cur_y + h * 0.14, 0.0),
        h * 0.14,
        0.0,
        10,
        Color::srgb(0.70, 0.25, 0.80),
    );

    m.build()
}

fn spawn_stacking_pyramid(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    pyramid: &StackingPyramid,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(pyramid.position);
    let trans = Transform::from_xyz(pyramid.position.x, ground, pyramid.position.y);
    let mesh = make_stacking_pyramid_mesh(pyramid.height);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 17. Vintage Wind-Up Tin Robot
// ---------------------------------------------------------------------------

pub fn make_tin_robot_mesh(height: f32, body_color: Color) -> Mesh {
    let mut m = MeshBuilder::default();
    let h = height;

    // 1. Block feet and legs.
    for leg_x in [-0.22 * h, 0.22 * h] {
        // Foot.
        m.beam(
            Vec3::new(leg_x, 0.15 * h, -0.15 * h),
            Vec3::new(leg_x, 0.15 * h, 0.25 * h),
            0.20 * h,
            0.15 * h,
            SILVER_METAL,
        );
        // Leg.
        m.beam(
            Vec3::new(leg_x, 0.15 * h, 0.0),
            Vec3::new(leg_x, 0.42 * h, 0.0),
            0.16 * h,
            0.26 * h,
            SILVER_METAL,
        );
    }

    // 2. Boxy torso.
    let torso_y = 0.42 * h;
    let torso_h = 0.35 * h;
    m.beam(
        Vec3::new(0.0, torso_y + torso_h / 2.0, -0.15 * h),
        Vec3::new(0.0, torso_y + torso_h / 2.0, 0.15 * h),
        0.48 * h,
        torso_h,
        body_color,
    );

    // Chest gauge meter.
    m.quad(
        [
            (
                Vec3::new(-0.16 * h, torso_y + 0.10 * h, 0.16 * h),
                Vec2::ZERO,
            ),
            (Vec3::new(0.16 * h, torso_y + 0.10 * h, 0.16 * h), Vec2::X),
            (Vec3::new(0.16 * h, torso_y + 0.26 * h, 0.16 * h), Vec2::ONE),
            (Vec3::new(-0.16 * h, torso_y + 0.26 * h, 0.16 * h), Vec2::Y),
        ],
        Vec3::Z,
        TOY_WHITE,
    );
    // Needle in gauge.
    m.beam(
        Vec3::new(-0.02 * h, torso_y + 0.14 * h, 0.165 * h),
        Vec3::new(0.08 * h, torso_y + 0.22 * h, 0.165 * h),
        0.02 * h,
        0.02 * h,
        TOY_RED,
    );

    // 3. Segmented arms and clamp hands.
    for (arm_x, is_right) in [(-0.30 * h, false), (0.30 * h, true)] {
        m.beam(
            Vec3::new(arm_x, torso_y + torso_h - 0.05 * h, 0.0),
            Vec3::new(arm_x, torso_y + 0.08 * h, 0.0),
            0.10 * h,
            0.25 * h,
            SILVER_METAL,
        );
        // Clamp claw hand.
        m.beam(
            Vec3::new(arm_x, torso_y + 0.04 * h, -0.06 * h),
            Vec3::new(arm_x, torso_y + 0.04 * h, 0.06 * h),
            0.08 * h,
            0.08 * h,
            TOY_RED,
        );
        if is_right {
            // Big brass butterfly wind-up key on the side.
            cylinder(
                &mut m,
                Vec3::new(arm_x + 0.08 * h, torso_y + torso_h * 0.6, 0.0),
                Vec3::new(arm_x + 0.22 * h, torso_y + torso_h * 0.6, 0.0),
                0.03 * h,
                0.03 * h,
                8,
                BATTERY_GOLD,
            );
            // Key wing discs.
            cylinder(
                &mut m,
                Vec3::new(arm_x + 0.22 * h, torso_y + torso_h * 0.6 - 0.06 * h, 0.0),
                Vec3::new(arm_x + 0.24 * h, torso_y + torso_h * 0.6 - 0.06 * h, 0.0),
                0.07 * h,
                0.07 * h,
                8,
                BATTERY_GOLD,
            );
            cylinder(
                &mut m,
                Vec3::new(arm_x + 0.22 * h, torso_y + torso_h * 0.6 + 0.06 * h, 0.0),
                Vec3::new(arm_x + 0.24 * h, torso_y + torso_h * 0.6 + 0.06 * h, 0.0),
                0.07 * h,
                0.07 * h,
                8,
                BATTERY_GOLD,
            );
        }
    }

    // 4. Square head.
    let head_y = torso_y + torso_h + 0.03 * h;
    let head_h = 0.22 * h;
    m.beam(
        Vec3::new(0.0, head_y + head_h / 2.0, -0.12 * h),
        Vec3::new(0.0, head_y + head_h / 2.0, 0.12 * h),
        0.32 * h,
        head_h,
        body_color,
    );

    // Visor eyes.
    m.quad(
        [
            (
                Vec3::new(-0.12 * h, head_y + 0.08 * h, 0.13 * h),
                Vec2::ZERO,
            ),
            (Vec3::new(0.12 * h, head_y + 0.08 * h, 0.13 * h), Vec2::X),
            (Vec3::new(0.12 * h, head_y + 0.15 * h, 0.13 * h), Vec2::ONE),
            (Vec3::new(-0.12 * h, head_y + 0.15 * h, 0.13 * h), Vec2::Y),
        ],
        Vec3::Z,
        TOY_BLACK,
    );
    for eye_x in [-0.06 * h, 0.06 * h] {
        cylinder(
            &mut m,
            Vec3::new(eye_x, head_y + 0.115 * h, 0.132 * h),
            Vec3::new(eye_x, head_y + 0.115 * h, 0.138 * h),
            0.025 * h,
            0.025 * h,
            6,
            TOY_YELLOW,
        );
    }

    // Antenna ears on head sides.
    for ear_x in [-0.17 * h, 0.17 * h] {
        cylinder(
            &mut m,
            Vec3::new(ear_x, head_y + head_h * 0.5, 0.0),
            Vec3::new(ear_x * 1.3, head_y + head_h * 0.5, 0.0),
            0.025 * h,
            0.025 * h,
            6,
            SILVER_METAL,
        );
        cylinder(
            &mut m,
            Vec3::new(ear_x * 1.3, head_y + head_h * 0.5, 0.0),
            Vec3::new(ear_x * 1.45, head_y + head_h * 0.5, 0.0),
            0.045 * h,
            0.045 * h,
            8,
            TOY_RED,
        );
    }

    m.build()
}

fn spawn_tin_robot(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    robot: &TinRobot,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(robot.position);
    let rot = Quat::from_rotation_y(robot.heading);
    let trans = Transform::from_xyz(robot.position.x, ground, robot.position.y).with_rotation(rot);
    let mesh = make_tin_robot_mesh(robot.height, robot.color);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 18. Toy Castle Fortress
// ---------------------------------------------------------------------------

pub fn make_toy_castle_mesh(scale: f32) -> Mesh {
    let mut m = MeshBuilder::default();
    let s = scale;
    let stone_col = Color::srgb(0.70, 0.72, 0.75);

    // 1. Castle outer walls with crenellations.
    let w = 18.0 * s;
    let wall_h = 6.0 * s;
    m.beam(
        Vec3::new(-w / 2.0, wall_h / 2.0, 0.0),
        Vec3::new(w / 2.0, wall_h / 2.0, 0.0),
        3.0 * s,
        wall_h,
        stone_col,
    );
    // Wall battlements (crenellations).
    let cren_count = 6;
    for c in 0..cren_count {
        if c % 2 == 0 {
            let cx = -w / 2.0 + (c as f32 + 0.5) * (w / cren_count as f32);
            m.beam(
                Vec3::new(cx, wall_h + 0.6 * s, 0.0),
                Vec3::new(cx, wall_h + 1.2 * s, 0.0),
                2.6 * s,
                1.2 * s,
                stone_col,
            );
        }
    }

    // 2. Gateway wooden drawbridge.
    m.beam(
        Vec3::new(0.0, 2.0 * s, 1.6 * s),
        Vec3::new(0.0, 2.0 * s, 3.6 * s),
        3.5 * s,
        0.3 * s,
        WOOD_RAIL,
    );

    // 3. Two flanking cylindrical corner bastion towers.
    for tx in [-w / 2.0, w / 2.0] {
        cylinder(
            &mut m,
            Vec3::new(tx, 0.0, 0.0),
            Vec3::new(tx, 8.5 * s, 0.0),
            2.5 * s,
            2.5 * s,
            12,
            stone_col,
        );
        // Tower conical roof.
        cylinder(
            &mut m,
            Vec3::new(tx, 8.5 * s, 0.0),
            Vec3::new(tx, 12.0 * s, 0.0),
            2.8 * s,
            0.0,
            12,
            TOY_RED,
        );
    }

    // 4. Central tall Keep tower.
    cylinder(
        &mut m,
        Vec3::new(0.0, wall_h, -3.0 * s),
        Vec3::new(0.0, 13.0 * s, -3.0 * s),
        3.2 * s,
        3.0 * s,
        12,
        stone_col,
    );
    // Steep conical spire roof.
    cylinder(
        &mut m,
        Vec3::new(0.0, 13.0 * s, -3.0 * s),
        Vec3::new(0.0, 18.0 * s, -3.0 * s),
        3.5 * s,
        0.0,
        12,
        TOY_BLUE,
    );
    // Gold flagpole with yellow pennant.
    m.beam(
        Vec3::new(0.0, 18.0 * s, -3.0 * s),
        Vec3::new(0.0, 21.0 * s, -3.0 * s),
        0.15 * s,
        0.15 * s,
        BATTERY_GOLD,
    );
    m.triangle(
        [
            (Vec3::new(0.0, 21.0 * s, -3.0 * s), Vec2::ZERO),
            (Vec3::new(2.5 * s, 20.0 * s, -3.0 * s), Vec2::X),
            (Vec3::new(0.0, 19.5 * s, -3.0 * s), Vec2::Y),
        ],
        Vec3::Z,
        TOY_YELLOW,
    );

    m.build()
}

fn spawn_toy_castle(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    castle: &ToyCastle,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(castle.position);
    let rot = Quat::from_rotation_y(castle.heading);
    let trans =
        Transform::from_xyz(castle.position.x, ground, castle.position.y).with_rotation(rot);
    let mesh = make_toy_castle_mesh(castle.scale);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 19. Suspended Giant Toy Dinosaur
// ---------------------------------------------------------------------------

pub fn make_suspended_dino_mesh(scale: f32) -> Mesh {
    let mut m = MeshBuilder::default();
    let s = scale;
    let dino_green = Color::srgb(0.20, 0.75, 0.25);
    let belly_yellow = Color::srgb(0.85, 0.88, 0.40);

    // 1. Suspension wire cables hanging from ceiling.
    m.beam(
        Vec3::new(-2.0 * s, 0.0, 0.0),
        Vec3::new(-2.0 * s, 25.0 * s, 0.0),
        0.08,
        0.08,
        SILVER_METAL,
    );
    m.beam(
        Vec3::new(2.0 * s, 0.0, 0.0),
        Vec3::new(2.0 * s, 25.0 * s, 0.0),
        0.08,
        0.08,
        SILVER_METAL,
    );

    // 2. Muscular T-Rex body.
    m.beam(
        Vec3::new(0.0, 0.0, -3.5 * s),
        Vec3::new(0.0, 0.0, 3.5 * s),
        3.2 * s,
        3.5 * s,
        dino_green,
    );
    // Yellow underbelly.
    m.beam(
        Vec3::new(0.0, -1.2 * s, -2.5 * s),
        Vec3::new(0.0, -1.2 * s, 2.5 * s),
        2.2 * s,
        1.2 * s,
        belly_yellow,
    );

    // 3. Tail curving backwards.
    m.beam(
        Vec3::new(0.0, 0.2 * s, -3.5 * s),
        Vec3::new(0.5 * s, 0.8 * s, -8.0 * s),
        2.0 * s,
        2.0 * s,
        dino_green,
    );
    m.beam(
        Vec3::new(0.5 * s, 0.8 * s, -8.0 * s),
        Vec3::new(1.2 * s, 1.8 * s, -13.0 * s),
        1.0 * s,
        1.0 * s,
        dino_green,
    );

    // 4. Neck and head with toothy jaw.
    m.beam(
        Vec3::new(0.0, 0.5 * s, 3.0 * s),
        Vec3::new(0.0, 2.8 * s, 6.0 * s),
        2.2 * s,
        2.4 * s,
        dino_green,
    );
    // Head block.
    m.beam(
        Vec3::new(0.0, 3.2 * s, 5.5 * s),
        Vec3::new(0.0, 3.2 * s, 9.5 * s),
        2.2 * s,
        2.2 * s,
        dino_green,
    );
    // Open lower jaw.
    m.beam(
        Vec3::new(0.0, 2.0 * s, 6.0 * s),
        Vec3::new(0.0, 1.5 * s, 9.0 * s),
        1.8 * s,
        0.8 * s,
        dino_green,
    );
    // Sharp white teeth.
    m.beam(
        Vec3::new(-0.8 * s, 2.5 * s, 7.5 * s),
        Vec3::new(0.8 * s, 2.5 * s, 7.5 * s),
        0.2 * s,
        0.3 * s,
        TOY_WHITE,
    );

    // 5. Powerful hind legs.
    for leg_x in [-1.8 * s, 1.8 * s] {
        m.beam(
            Vec3::new(leg_x, 0.0, 0.0),
            Vec3::new(leg_x, -3.5 * s, -s),
            1.4 * s,
            1.8 * s,
            dino_green,
        );
        // Foot with claws.
        m.beam(
            Vec3::new(leg_x, -3.5 * s, -s),
            Vec3::new(leg_x, -3.6 * s, 1.2 * s),
            1.2 * s,
            0.6 * s,
            dino_green,
        );
    }

    // 6. Tiny funny front arms.
    for arm_x in [-1.2 * s, 1.2 * s] {
        m.beam(
            Vec3::new(arm_x, 0.5 * s, 3.2 * s),
            Vec3::new(arm_x * 1.1, -0.4 * s, 4.4 * s),
            0.4 * s,
            0.4 * s,
            dino_green,
        );
    }

    m.build()
}

fn spawn_suspended_dino(
    world: &mut ChildSpawnerCommands,
    dino: &SuspendedDino,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let rot = Quat::from_rotation_y(dino.heading);
    let trans =
        Transform::from_xyz(dino.position.x, dino.height, dino.position.y).with_rotation(rot);
    let mesh = make_suspended_dino_mesh(dino.scale);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 20. Wooden Toy Car
// ---------------------------------------------------------------------------

pub fn make_toy_car_mesh(length: f32, color: Color) -> Mesh {
    let mut m = MeshBuilder::default();
    let l = length;
    let w = l * 0.55;
    let h = l * 0.40;

    // 1. Lower chassis beam.
    m.beam(
        Vec3::new(0.0, h * 0.45, -l / 2.0),
        Vec3::new(0.0, h * 0.45, l / 2.0),
        w,
        h * 0.4,
        color,
    );

    // 2. Cabin top.
    m.beam(
        Vec3::new(0.0, h * 0.85, -l * 0.25),
        Vec3::new(0.0, h * 0.85, l * 0.15),
        w * 0.85,
        h * 0.45,
        color,
    );

    // Windows.
    m.quad(
        [
            (Vec3::new(-w * 0.43, h * 0.70, -l * 0.20), Vec2::ZERO),
            (Vec3::new(-w * 0.43, h * 0.70, l * 0.10), Vec2::X),
            (Vec3::new(-w * 0.43, h * 1.00, l * 0.10), Vec2::ONE),
            (Vec3::new(-w * 0.43, h * 1.00, -l * 0.20), Vec2::Y),
        ],
        -Vec3::X,
        Color::srgb(0.18, 0.22, 0.28),
    );
    m.quad(
        [
            (Vec3::new(w * 0.43, h * 0.70, -l * 0.20), Vec2::ZERO),
            (Vec3::new(w * 0.43, h * 1.00, -l * 0.20), Vec2::Y),
            (Vec3::new(w * 0.43, h * 1.00, l * 0.10), Vec2::ONE),
            (Vec3::new(w * 0.43, h * 0.70, l * 0.10), Vec2::X),
        ],
        Vec3::X,
        Color::srgb(0.18, 0.22, 0.28),
    );

    // 3. 4 Wooden wheel discs.
    let wheel_r = h * 0.35;
    for wx in [-w / 2.0 - 0.2, w / 2.0 + 0.2] {
        for wz in [-l * 0.32, l * 0.32] {
            cylinder(
                &mut m,
                Vec3::new(wx - 0.15 * wx.signum(), wheel_r, wz),
                Vec3::new(wx + 0.15 * wx.signum(), wheel_r, wz),
                wheel_r,
                wheel_r,
                10,
                TOY_YELLOW,
            );
            cylinder(
                &mut m,
                Vec3::new(wx + 0.16 * wx.signum(), wheel_r, wz),
                Vec3::new(wx + 0.20 * wx.signum(), wheel_r, wz),
                wheel_r * 0.35,
                wheel_r * 0.35,
                8,
                TOY_RED,
            );
        }
    }

    // Yellow round headlights.
    for hx in [-w * 0.3, w * 0.3] {
        cylinder(
            &mut m,
            Vec3::new(hx, h * 0.45, l / 2.0 + 0.05),
            Vec3::new(hx, h * 0.45, l / 2.0 + 0.20),
            h * 0.12,
            h * 0.12,
            8,
            TOY_YELLOW,
        );
    }

    m.build()
}

fn spawn_toy_car(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    car: &ToyCar,
    mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(car.position);
    let rot = Quat::from_rotation_y(car.heading);
    let trans = Transform::from_xyz(car.position.x, ground, car.position.y).with_rotation(rot);
    let mesh = make_toy_car_mesh(car.length, car.color);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(mat.clone()),
        trans,
    ));
}

// ---------------------------------------------------------------------------
// 21. Multi-Tiered Round Display Table
// ---------------------------------------------------------------------------

pub fn make_display_table_mesh(radius: f32, height: f32, tiers: usize) -> Mesh {
    let mut m = MeshBuilder::default();
    let birch = Color::srgb(0.85, 0.72, 0.52);
    let trim = TOY_BLUE;

    // Center turned pedestal column.
    cylinder(
        &mut m,
        Vec3::ZERO,
        Vec3::new(0.0, height, 0.0),
        radius * 0.18,
        radius * 0.14,
        14,
        WOOD_RAIL,
    );

    let tier_count = tiers.clamp(1, 4);
    for t in 0..tier_count {
        let frac = (t + 1) as f32 / tier_count as f32;
        let ty = frac * height;
        let r = radius * (1.0 - t as f32 * 0.28);

        // Circular tabletop disc.
        cylinder(
            &mut m,
            Vec3::new(0.0, ty - 0.4, 0.0),
            Vec3::new(0.0, ty, 0.0),
            r,
            r,
            16,
            birch,
        );
        // Decorative perimeter rim.
        cylinder(
            &mut m,
            Vec3::new(0.0, ty - 0.5, 0.0),
            Vec3::new(0.0, ty - 0.35, 0.0),
            r + 0.1,
            r + 0.1,
            16,
            trim,
        );
    }

    m.build()
}

fn spawn_display_table(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    table: &DisplayTable,
    wood_mat: &Handle<StandardMaterial>,
    toy_mat: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    let ground = terrain.height(table.position);
    let trans = Transform::from_xyz(table.position.x, ground, table.position.y);
    let mesh = make_display_table_mesh(table.radius, table.height, table.tiers);
    world.spawn((
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(wood_mat.clone()),
        trans,
    ));

    // Display toys sitting on the table tiers:
    let boat_mesh = make_sailboat_mesh(table.radius * 0.75, TOY_BLUE);
    world.spawn((
        Mesh3d(assets.meshes.add(boat_mesh)),
        MeshMaterial3d(toy_mat.clone()),
        Transform::from_xyz(
            table.position.x,
            ground + table.height * 0.67,
            table.position.y,
        ),
    ));

    let pyr_mesh = make_stacking_pyramid_mesh(table.height * 0.45);
    world.spawn((
        Mesh3d(assets.meshes.add(pyr_mesh)),
        MeshMaterial3d(toy_mat.clone()),
        Transform::from_xyz(table.position.x, ground + table.height, table.position.y),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toy_store_procedural_meshes_have_geometry() {
        let (body, term, arch) = make_battery_gantry_meshes(14.0);
        assert!(body.count_vertices() > 0);
        assert!(term.count_vertices() > 0);
        assert!(arch.count_vertices() > 0);

        let (reg_b, reg_k, reg_s) = make_cash_register_meshes();
        assert!(reg_b.count_vertices() > 0);
        assert!(reg_k.count_vertices() > 0);
        assert!(reg_s.count_vertices() > 0);

        assert!(make_nutcracker_mesh(10.0).count_vertices() > 0);
        assert!(make_matryoshka_mesh(1.0).count_vertices() > 0);
        assert!(make_sailboat_mesh(8.0, TOY_BLUE).count_vertices() > 0);
        assert!(make_xylophone_mesh(6.0).count_vertices() > 0);
        assert!(make_stacking_pyramid_mesh(5.0).count_vertices() > 0);
        assert!(make_tin_robot_mesh(8.0, TOY_BLUE).count_vertices() > 0);
        assert!(make_toy_castle_mesh(1.0).count_vertices() > 0);
        assert!(make_suspended_dino_mesh(1.5).count_vertices() > 0);
        assert!(make_toy_car_mesh(5.0, TOY_RED).count_vertices() > 0);
        assert!(make_display_table_mesh(10.0, 6.0, 3).count_vertices() > 0);
    }
}
