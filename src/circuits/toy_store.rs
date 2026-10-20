//! Juguetería (Toy Store GP): an indoor circuit set in a giant toy store,
//! where miniature toy bikes race across shop counters, past cash registers,
//! through a toy train level crossing, along a construction block canyon, and
//! down an elevated Hot Wheels speedway.
//!
//! 1. **Sector 1 — Cash Register Counter (`0..460 m`):**
//!    Starts on the wooden counter heading north (-Z) under the AA battery
//!    finish gantry (`along: 0.0 m`), shoots under the barcode scanner's red
//!    laser beam (`90.0 m`), flies off a comic-book ramp (`160.0 m`), and sweeps
//!    right around the giant plush teddy bear at "Curva Peluches" (`240..420 m`).
//! 2. **Sector 2 — Toy Train Level Crossing (`460..940 m`):**
//!    Descent to table level into a winding S-bend sector passing right through
//!    a classic wooden railway loop with an electric steam locomotive, cargo
//!    wagons, and miniature turned wooden trees.
//! 3. **Sector 3 — Construction Block Canyon (`940..1420 m`):**
//!    Technical chicane weaving between interlocking plastic block skyscraper
//!    towers, giant spotted dice, and a delicate multi-story card house.
//! 4. **Sector 4 — Hot Wheels Speedway (`1420..1920 m`):**
//!    Climbing onto an elevated bright-orange track with high banked velodrome
//!    curves and toy sports cars parked on the verges.
//! 5. **Sector 5 — Scalextric Slot Circuit (`1920..2420 m`):**
//!    Textured black slot track with dual metal rails, red/white safety
//!    barriers, and a fast exit leading straight back to the counter and meta.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::{Banking, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Key stretches of the lap in metres from the finish line.
const START_STRAIGHT: Span = span(2350.0, 240.0);
const CURVA_PELUCHES: Span = span(240.0, 430.0);
const TRAIN_CROSSING: Span = span(540.0, 860.0);
const BLOCK_CANYON: Span = span(960.0, 1380.0);
const HOT_WHEELS_CLIMB: Span = span(1440.0, 1880.0);
const SCALEXTRIC_RUN: Span = span(1920.0, 2350.0);

pub const TOY_STORE: Circuit = Circuit {
    name: "Juguetería",
    name_en: "Toy Store GP",
    description: "Pista interior entre juguetes gigantes: mostrador y caja registradora,\n\
                  paso a nivel del tren, cañón de bloques y bajada Hot Wheels",
    description_en: "Indoor track among giant toys: counter & cash register,\n\
                     train crossing, building block canyon and Hot Wheels drop",
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.0,
        cicadas: 0.0,
        wind: 0.12,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/plastic_velocity_rush.mp3"),
        length: 176.3,
        loudness: -16.0,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[
            // Curva Peluches banked right-hander.
            Banking {
                from: CURVA_PELUCHES.from,
                to: CURVA_PELUCHES.to,
                factor: 1.6,
            },
            // Block canyon chicane banking.
            Banking {
                from: 1040.0,
                to: 1260.0,
                factor: 1.4,
            },
            // Hot Wheels high-speed banked sweep.
            Banking {
                from: HOT_WHEELS_CLIMB.from,
                to: HOT_WHEELS_CLIMB.to,
                factor: 2.1,
            },
            // Final Scalextric curve before the counter straight.
            Banking {
                from: 2240.0,
                to: 2380.0,
                factor: 1.5,
            },
        ],
        narrows: &[],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 0.0,
        greens: Greens::Lawn,
        grass_shade: 0.95,
        massifs: &[],
        cuttings: &[],
        terraces: &[],
        mountains: |_| 0.0,
        curbs: [
            Color::srgb(0.95, 0.15, 0.15), // Red
            Color::srgb(0.96, 0.96, 0.96), // White
        ],
        walls: &[
            WallRun {
                span: START_STRAIGHT,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: START_STRAIGHT,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: TRAIN_CROSSING,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: TRAIN_CROSSING,
                side: Side::Right,
                style: WallStyle::Timber,
            },
            WallRun {
                span: BLOCK_CANYON,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: BLOCK_CANYON,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: HOT_WHEELS_CLIMB,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: HOT_WHEELS_CLIMB,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: SCALEXTRIC_RUN,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: SCALEXTRIC_RUN,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            // Fallback for entire lap ensuring no gap in track walls.
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
        ],
        tunnels: &[],
        bridges: &[],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: Color::srgb(0.95, 0.78, 0.1),
                text: Some("TOY STORE GP"),
            },
            Gantry {
                along: 580.0,
                color: Color::srgb(0.92, 0.2, 0.2),
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1220.0,
                color: Color::srgb(0.2, 0.5, 0.95),
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1840.0,
                color: Color::srgb(0.98, 0.45, 0.1),
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 80.0,
                side: Side::Left,
                text: "REGISTER 1",
                color: Color::srgb(0.12, 0.14, 0.20),
                accent: Some((8, Color::srgb(0.2, 0.9, 0.3))),
            },
            Billboard {
                along: 480.0,
                side: Side::Right,
                text: "EXPRESS LINE",
                color: Color::srgb(0.15, 0.45, 0.85),
                accent: None,
            },
            Billboard {
                along: 920.0,
                side: Side::Left,
                text: "BLOCK CITY",
                color: Color::srgb(0.92, 0.2, 0.15),
                accent: None,
            },
            Billboard {
                along: 1410.0,
                side: Side::Right,
                text: "HOT WHEELS",
                color: Color::srgb(0.98, 0.45, 0.08),
                accent: Some((3, Color::srgb(0.15, 0.55, 0.95))),
            },
            Billboard {
                along: 1910.0,
                side: Side::Left,
                text: "SCALEXTRIC",
                color: Color::srgb(0.12, 0.12, 0.15),
                accent: Some((5, Color::srgb(0.95, 0.15, 0.15))),
            },
        ],
        chevrons: &[
            ChevronSign {
                along: 280.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 370.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1110.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1540.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 2310.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
        ],
        brake_markers: &[
            BrakeMarker {
                along: 160.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 210.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 1000.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 1050.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 2210.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 2260.0,
                side: Side::Right,
                distance: 50,
            },
        ],
        lamps: &[
            Row {
                span: START_STRAIGHT,
                side: Side::Left,
                lateral: WALL_DISTANCE + 1.2,
                every: 24.0,
            },
            Row {
                span: START_STRAIGHT,
                side: Side::Right,
                lateral: WALL_DISTANCE + 1.2,
                every: 24.0,
            },
        ],
        trees: &[],
        forests: &[],
        houses: Houses {
            max: 0,
            paints: &[],
            spans: &[],
            style: HouseStyle::Cottage,
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[],
        grandstands: &[],
        beach_parasols: &[],
        soil: Soil::ToyStore,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::ToyStore,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        wheelie_halo: None,
        japan: None,
        middle_earth: None,
        space: None,
        toy_store: Some(TOY_STORE_PROPS),
    },
};

const TOY_STORE_PROPS: ToyStore = ToyStore {
    battery_gantries: &[BatteryGantry {
        along: 0.0,
        height: 14.0,
    }],
    cash_registers: &[CashRegister {
        position: Vec2::new(28.0, -110.0),
        heading: 0.4,
    }],
    barcode_scanners: &[BarcodeScanner {
        along: 85.0,
        side: Side::Right,
    }],
    comic_ramps: &[ComicRamp {
        along: 155.0,
        width: 14.0,
    }],
    teddy_bears: &[TeddyBear {
        position: Vec2::new(175.0, -485.0),
        heading: -2.3,
        scale: 1.8,
    }],
    toy_trains: &[ToyTrain {
        center: Vec2::new(430.0, -280.0),
        radius: 48.0,
    }],
    block_towers: &[
        BlockTower {
            position: Vec2::new(600.0, -320.0),
            height: 24.0,
            color_primary: Color::srgb(0.92, 0.15, 0.15),
            color_secondary: Color::srgb(0.12, 0.45, 0.95),
        },
        BlockTower {
            position: Vec2::new(720.0, -310.0),
            height: 32.0,
            color_primary: Color::srgb(0.96, 0.82, 0.10),
            color_secondary: Color::srgb(0.15, 0.78, 0.25),
        },
        BlockTower {
            position: Vec2::new(670.0, -420.0),
            height: 20.0,
            color_primary: Color::srgb(0.98, 0.42, 0.08),
            color_secondary: Color::srgb(0.92, 0.15, 0.15),
        },
    ],
    dice: &[
        Die {
            position: Vec2::new(640.0, -220.0),
            heading: 0.75,
            size: 9.0,
            roll: 6,
        },
        Die {
            position: Vec2::new(750.0, -240.0),
            heading: -0.35,
            size: 7.5,
            roll: 4,
        },
    ],
    card_houses: &[CardHouse {
        position: Vec2::new(780.0, -340.0),
        heading: 0.2,
        stories: 3,
    }],
    ceiling_lights: &[
        CeilingLight {
            position: Vec2::new(100.0, -200.0),
            height: 65.0,
            warm: true,
        },
        CeilingLight {
            position: Vec2::new(450.0, -250.0),
            height: 65.0,
            warm: true,
        },
        CeilingLight {
            position: Vec2::new(650.0, -200.0),
            height: 65.0,
            warm: true,
        },
        CeilingLight {
            position: Vec2::new(300.0, 50.0),
            height: 65.0,
            warm: true,
        },
    ],
    shelves: &[
        ToyShelf {
            center: Vec2::new(150.0, -560.0),
            heading: 0.0,
            length: 80.0,
            height: 45.0,
        },
        ToyShelf {
            center: Vec2::new(880.0, -300.0),
            heading: FRAC_PI_2,
            length: 120.0,
            height: 50.0,
        },
        ToyShelf {
            center: Vec2::new(400.0, 180.0),
            heading: 0.0,
            length: 100.0,
            height: 45.0,
        },
        ToyShelf {
            center: Vec2::new(-80.0, -150.0),
            heading: FRAC_PI_2,
            length: 90.0,
            height: 45.0,
        },
    ],
    nutcrackers: &[
        // Sector 1: Start straight guard squad (both sides of straight).
        Nutcracker {
            position: Vec2::new(-16.0, -30.0),
            heading: FRAC_PI_2,
            height: 10.0,
        },
        Nutcracker {
            position: Vec2::new(16.0, -30.0),
            heading: -FRAC_PI_2,
            height: 10.0,
        },
        Nutcracker {
            position: Vec2::new(-16.0, -70.0),
            heading: FRAC_PI_2,
            height: 10.0,
        },
        Nutcracker {
            position: Vec2::new(16.0, -70.0),
            heading: -FRAC_PI_2,
            height: 10.0,
        },
        Nutcracker {
            position: Vec2::new(-16.0, -140.0),
            heading: FRAC_PI_2,
            height: 10.0,
        },
        Nutcracker {
            position: Vec2::new(16.0, -140.0),
            heading: -FRAC_PI_2,
            height: 10.0,
        },
        // Sector 2: Train crossing gate guards.
        Nutcracker {
            position: Vec2::new(370.0, -155.0),
            heading: 0.8,
            height: 9.0,
        },
        Nutcracker {
            position: Vec2::new(495.0, -155.0),
            heading: -0.8,
            height: 9.0,
        },
        // Sector 4: Hot Wheels approach.
        Nutcracker {
            position: Vec2::new(645.0, -115.0),
            heading: -1.2,
            height: 11.0,
        },
        Nutcracker {
            position: Vec2::new(425.0, -15.0),
            heading: -1.8,
            height: 11.0,
        },
        // Sector 5: Scalextric straight track marshals.
        Nutcracker {
            position: Vec2::new(260.0, 45.0),
            heading: 1.1,
            height: 9.5,
        },
        Nutcracker {
            position: Vec2::new(170.0, 90.0),
            heading: 0.9,
            height: 9.5,
        },
        Nutcracker {
            position: Vec2::new(90.0, 125.0),
            heading: 0.4,
            height: 9.5,
        },
        Nutcracker {
            position: Vec2::new(35.0, 120.0),
            heading: 0.2,
            height: 9.5,
        },
    ],
    matryoshkas: &[
        // Curva Peluches apex.
        Matryoshka {
            position: Vec2::new(250.0, -425.0),
            heading: -1.4,
            scale: 1.6,
        },
        // Train loop scenic curve.
        Matryoshka {
            position: Vec2::new(400.0, -135.0),
            heading: 0.2,
            scale: 1.4,
        },
        Matryoshka {
            position: Vec2::new(465.0, -135.0),
            heading: -0.2,
            scale: 1.4,
        },
        // Hot Wheels climb backdrop.
        Matryoshka {
            position: Vec2::new(580.0, -75.0),
            heading: 1.8,
            scale: 1.7,
        },
        // Scalextric hairpin.
        Matryoshka {
            position: Vec2::new(160.0, 85.0),
            heading: -1.6,
            scale: 1.5,
        },
    ],
    sailboats: &[
        // Curva Peluches display.
        ToySailboat {
            position: Vec2::new(105.0, -450.0),
            heading: 0.6,
            length: 11.0,
            hull_color: Color::srgb(0.12, 0.45, 0.95),
        },
        // Train crossing pond island.
        ToySailboat {
            position: Vec2::new(335.0, -165.0),
            heading: -0.8,
            length: 9.5,
            hull_color: Color::srgb(0.92, 0.15, 0.15),
        },
        ToySailboat {
            position: Vec2::new(530.0, -165.0),
            heading: 1.1,
            length: 9.5,
            hull_color: Color::srgb(0.15, 0.75, 0.35),
        },
        // Scalextric straight lake.
        ToySailboat {
            position: Vec2::new(100.0, 130.0),
            heading: -2.1,
            length: 12.0,
            hull_color: Color::srgb(0.12, 0.45, 0.95),
        },
    ],
    xylophones: &[
        // Curva Peluches inside curve.
        Xylophone {
            position: Vec2::new(140.0, -390.0),
            heading: 0.9,
            length: 12.0,
        },
        // Train S-bend.
        Xylophone {
            position: Vec2::new(435.0, -130.0),
            heading: 0.0,
            length: 13.0,
        },
        // Block Canyon chicane apex.
        Xylophone {
            position: Vec2::new(690.0, -225.0),
            heading: -1.2,
            length: 12.0,
        },
        // Scalextric return curve.
        Xylophone {
            position: Vec2::new(75.0, 125.0),
            heading: 1.8,
            length: 11.0,
        },
    ],
    stacking_pyramids: &[
        // Sector 1 entry to Curva Peluches.
        StackingPyramid {
            position: Vec2::new(65.0, -415.0),
            height: 10.0,
        },
        // Sector 2 train station corner.
        StackingPyramid {
            position: Vec2::new(315.0, -280.0),
            height: 9.0,
        },
        StackingPyramid {
            position: Vec2::new(555.0, -280.0),
            height: 9.0,
        },
        // Sector 3 block canyon.
        StackingPyramid {
            position: Vec2::new(640.0, -425.0),
            height: 11.0,
        },
        StackingPyramid {
            position: Vec2::new(775.0, -310.0),
            height: 10.0,
        },
        // Sector 5 Scalextric sweep.
        StackingPyramid {
            position: Vec2::new(40.0, 125.0),
            height: 9.0,
        },
        StackingPyramid {
            position: Vec2::new(10.0, 60.0),
            height: 8.5,
        },
    ],
    tin_robots: &[
        // Sector 3: Block Canyon colossal robot guardians.
        TinRobot {
            position: Vec2::new(610.0, -360.0),
            heading: 0.8,
            height: 14.0,
            color: Color::srgb(0.20, 0.50, 0.85),
        },
        TinRobot {
            position: Vec2::new(735.0, -370.0),
            heading: -0.9,
            height: 16.0,
            color: Color::srgb(0.85, 0.20, 0.20),
        },
        TinRobot {
            position: Vec2::new(740.0, -280.0),
            heading: -2.3,
            height: 15.0,
            color: Color::srgb(0.20, 0.70, 0.40),
        },
        // Sector 4: Hot Wheels base guardian.
        TinRobot {
            position: Vec2::new(455.0, -15.0),
            heading: 1.6,
            height: 13.0,
            color: Color::srgb(0.85, 0.45, 0.10),
        },
    ],
    toy_castles: &[
        // Sector 3: Block Canyon citadel fortress.
        ToyCastle {
            position: Vec2::new(750.0, -380.0),
            heading: -0.7,
            scale: 1.3,
        },
        // Sector 5: Scalextric garden palace.
        ToyCastle {
            position: Vec2::new(220.0, 95.0),
            heading: 0.6,
            scale: 1.1,
        },
    ],
    suspended_dinos: &[
        // Suspended T-Rex dinosaur hanging over Sector 4 (Hot Wheels Speedway).
        SuspendedDino {
            position: Vec2::new(510.0, -70.0),
            height: 38.0,
            heading: -0.6,
            scale: 2.4,
        },
    ],
    toy_cars: &[
        // Parked on counter furniture.
        ToyCar {
            position: Vec2::new(-18.0, -180.0),
            heading: 0.0,
            length: 7.0,
            color: Color::srgb(0.92, 0.15, 0.15),
        },
        ToyCar {
            position: Vec2::new(20.0, -200.0),
            heading: 0.1,
            length: 6.5,
            color: Color::srgb(0.12, 0.45, 0.95),
        },
        // Sector 1 Curva Peluches.
        ToyCar {
            position: Vec2::new(30.0, -325.0),
            heading: 0.4,
            length: 6.8,
            color: Color::srgb(0.96, 0.82, 0.10),
        },
        ToyCar {
            position: Vec2::new(95.0, -395.0),
            heading: 1.1,
            length: 6.5,
            color: Color::srgb(0.15, 0.78, 0.25),
        },
        // Sector 4 Hot Wheels verges.
        ToyCar {
            position: Vec2::new(550.0, -95.0),
            heading: -1.2,
            length: 7.2,
            color: Color::srgb(0.98, 0.42, 0.08),
        },
        ToyCar {
            position: Vec2::new(480.0, -45.0),
            heading: -1.7,
            length: 6.8,
            color: Color::srgb(0.92, 0.15, 0.15),
        },
        // Sector 5 Scalextric run.
        ToyCar {
            position: Vec2::new(195.0, 80.0),
            heading: 1.0,
            length: 6.5,
            color: Color::srgb(0.12, 0.45, 0.95),
        },
        ToyCar {
            position: Vec2::new(125.0, 115.0),
            heading: 0.4,
            length: 7.0,
            color: Color::srgb(0.96, 0.82, 0.10),
        },
        ToyCar {
            position: Vec2::new(22.0, 95.0),
            heading: -0.3,
            length: 6.5,
            color: Color::srgb(0.15, 0.78, 0.25),
        },
    ],
    display_tables: &[
        // Curva Peluches apex island (showcase table with boat and pyramid).
        DisplayTable {
            position: Vec2::new(215.0, -425.0),
            radius: 14.0,
            height: 7.0,
            tiers: 3,
        },
        // Sector 2 Train Crossing tables.
        DisplayTable {
            position: Vec2::new(355.0, -230.0),
            radius: 12.0,
            height: 6.0,
            tiers: 3,
        },
        DisplayTable {
            position: Vec2::new(505.0, -230.0),
            radius: 12.0,
            height: 6.0,
            tiers: 3,
        },
        // Sector 4 Hot Wheels spectator table.
        DisplayTable {
            position: Vec2::new(495.0, -45.0),
            radius: 15.0,
            height: 7.5,
            tiers: 3,
        },
        // Sector 5 Scalextric corner table.
        DisplayTable {
            position: Vec2::new(135.0, 95.0),
            radius: 13.0,
            height: 6.5,
            tiers: 3,
        },
    ],
};

const CONTROL_POINTS: [Vec2; 29] = [
    // 1. Counter start straight, running north (-Z).
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -120.0),
    Vec2::new(0.0, -230.0),
    // 2. Curva Peluches sweeping right around the teddy bear.
    Vec2::new(15.0, -320.0),
    Vec2::new(55.0, -385.0),
    Vec2::new(125.0, -425.0),
    Vec2::new(195.0, -425.0),
    Vec2::new(260.0, -390.0),
    Vec2::new(295.0, -330.0),
    // 3. Train crossing S-bends.
    Vec2::new(315.0, -250.0),
    Vec2::new(365.0, -185.0),
    Vec2::new(435.0, -165.0),
    Vec2::new(505.0, -185.0),
    Vec2::new(545.0, -250.0),
    // 4. Construction block canyon.
    Vec2::new(575.0, -330.0),
    Vec2::new(635.0, -395.0),
    Vec2::new(710.0, -400.0),
    Vec2::new(765.0, -345.0),
    Vec2::new(760.0, -265.0),
    Vec2::new(715.0, -195.0),
    // 5. Hot Wheels climb and speedway.
    Vec2::new(630.0, -130.0),
    Vec2::new(530.0, -70.0),
    Vec2::new(410.0, -25.0),
    // 6. Scalextric run and smooth finish straight return.
    Vec2::new(290.0, 25.0),
    Vec2::new(180.0, 70.0),
    Vec2::new(105.0, 105.0),
    Vec2::new(50.0, 105.0),
    Vec2::new(15.0, 80.0),
    Vec2::new(0.0, 45.0),
];

const ELEVATION: [(f32, f32); 21] = [
    // Counter start line flat at 2.5 m.
    (0.0, 2.5),
    (70.0, 2.5),
    // Comic book jump crest.
    (130.0, 2.8),
    (155.0, 3.8),
    (180.0, 2.8),
    (240.0, 2.5),
    // Curva Peluches gentle incline.
    (360.0, 3.0),
    (480.0, 2.5),
    // Gentle dip into the train railway crossing.
    (600.0, 2.0),
    (750.0, 2.0),
    (900.0, 2.2),
    // Block canyon.
    (1050.0, 2.5),
    (1250.0, 3.0),
    (1400.0, 3.5),
    // Hot Wheels speedway elevated climb up to +8.5 m.
    (1550.0, 6.0),
    (1700.0, 8.5),
    (1820.0, 8.0),
    // Descent to Scalextric track.
    (1960.0, 5.5),
    (2100.0, 3.2),
    (2250.0, 2.5),
    (2400.0, 2.5),
];

const COAST: [(f32, f32); 8] = [
    (0.0, 260.0),
    (600.0, 260.0),
    (1100.0, 60.0),
    (1180.0, -160.0),
    (1240.0, -160.0),
    (1300.0, 60.0),
    (1800.0, 260.0),
    (2350.0, 260.0),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toy_store_layout_is_valid() {
        let track = crate::track::Track::new(&TOY_STORE.layout);
        assert!(track.length() > 2000.0 && track.length() < 2800.0);
    }
}
