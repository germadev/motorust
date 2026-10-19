//! La Juguetería (Toy Store): a lap through a toy store at the scale of its
//! toys, with the bikes the size of toy motorbikes.
//!
//! 1. The start straight runs north from the checkout among shopping
//!    baskets, price tags and a grandstand of toy boxes.
//! 2. A long Scalextric sweeper rolls east over the shop floor, with slot
//!    rails, bright curbs and a narrow split.
//! 3. A ramp climbs onto the shelf tabletop past a toy train running round
//!    its rails, through a tunnel in a papier-mâché mountain.
//! 4. A Hot Wheels section of orange track on plastic supports, with a
//!    giant loop standing beside the road where the plaster hills rise.
//! 5. A construction corner of giant building blocks, dice and a LEGO
//!    bridge.
//! 6. The slide: a narrow, banked descent from the shelf tabletop back to
//!    the shop floor, with the walls opened to the drop.
//! 7. The plush aisle and the checkout lead back to the start.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::{Banking, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line. Worked out with
/// `tools/circuit_layout.py`.
const SCALEXTRIC: Span = span(600.0, 1740.0);
const TRAIN: Span = span(1750.0, 2400.0);
const TRAIN_TUNNEL: Span = span(1660.0, 1820.0);
const HOT_WHEELS: Span = span(2400.0, 2700.0);
const BLOCKS: Span = span(2700.0, 2980.0);
const SLIDE: Span = span(2980.0, 3420.0);

/// How far from the centre line the walls stand over the narrow Scalextric
/// lane and the slide (m).
const SLOT_WALL: f32 = 6.5;
const SLIDE_WALL: f32 = 6.0;

const ORANGE: Color = Color::srgb(0.95, 0.42, 0.08);
const BLUE: Color = Color::srgb(0.08, 0.32, 0.85);
const RED: Color = Color::srgb(0.85, 0.1, 0.12);
const YELLOW: Color = Color::srgb(1.0, 0.82, 0.16);
const GREEN: Color = Color::srgb(0.15, 0.65, 0.28);
const CREAM: Color = Color::srgb(0.95, 0.92, 0.85);

pub const TOY_STORE: Circuit = Circuit {
    name: "La Juguetería",
    name_en: "The Toy Store",
    description: "Scalextric, Hot Wheels, un tren eléctrico,\n\
                  un tobogán y bloques a tamaño de juguete",
    description_en: "Scalextric, Hot Wheels, an electric train,\n\
                     a slide and toys at toy scale",
    // An empty shop after closing: a hum, the odd train whistle.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.15,
        cicadas: 0.0,
        wind: 0.12,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/plastic_velocity_rush.mp3"),
        length: 176.3,
        loudness: -14.3,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[
            // The Scalextric sweeper: banked like a plastic hairpin.
            Banking {
                from: SCALEXTRIC.from + 120.0,
                to: SCALEXTRIC.to,
                factor: 2.2,
            },
            // The Hot Wheels section: extra banked, as its tracks are.
            Banking {
                from: HOT_WHEELS.from,
                to: HOT_WHEELS.to,
                factor: 2.0,
            },
            // The slide comes down banked all the way.
            Banking {
                from: SLIDE.from,
                to: SLIDE.to,
                factor: 2.6,
            },
        ],
        // The narrow Scalextric lane and the slide, narrower than the road.
        narrows: &[
            Narrowing {
                from: 800.0,
                to: 1200.0,
                wall: SLOT_WALL,
            },
            Narrowing {
                from: 3150.0,
                to: 3350.0,
                wall: SLIDE_WALL,
            },
        ],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::Function(shop_floor_level),
        hills: 0.0,
        greens: Greens::Lawn,
        grass_shade: 0.6,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[Terrace {
            // A level tabletop for the toy train and the Hot Wheels track.
            center: Vec2::new(220.0, -360.0),
            radius: 430.0,
            height: 22.0,
            tilt: Vec2::ZERO,
            skirt: 90.0,
        }],
        mountains: no_mountains,
        curbs: [ORANGE, BLUE],
        walls: &[
            WallRun {
                span: SCALEXTRIC,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: SCALEXTRIC,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: TRAIN,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: TRAIN,
                side: Side::Right,
                style: WallStyle::Timber,
            },
            WallRun {
                span: BLOCKS,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: BLOCKS,
                side: Side::Right,
                style: WallStyle::Timber,
            },
            WallRun {
                span: SLIDE,
                side: Side::Left,
                style: WallStyle::Railing,
            },
            WallRun {
                span: SLIDE,
                side: Side::Right,
                style: WallStyle::Railing,
            },
        ],
        tunnels: &[TRAIN_TUNNEL],
        bridges: &[],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: RED,
                text: Some("TOY STORE"),
            },
            Gantry {
                along: 880.0,
                color: YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1760.0,
                color: YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2640.0,
                color: YELLOW,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 700.0,
                side: Side::Right,
                text: "SCALEXTRIC",
                color: Color::srgb(0.1, 0.12, 0.2),
                accent: Some((4, ORANGE)),
            },
            Billboard {
                along: 1100.0,
                side: Side::Left,
                text: "SLOT RACING",
                color: Color::srgb(0.08, 0.1, 0.16),
                accent: Some((4, BLUE)),
            },
            Billboard {
                along: 1850.0,
                side: Side::Right,
                text: "ELECTRIC TRAIN",
                color: Color::srgb(0.12, 0.14, 0.1),
                accent: Some((8, GREEN)),
            },
            Billboard {
                along: 2500.0,
                side: Side::Left,
                text: "HOT WHEELS",
                color: Color::srgb(0.16, 0.08, 0.06),
                accent: Some((4, ORANGE)),
            },
            Billboard {
                along: 2850.0,
                side: Side::Right,
                text: "BUILDING BLOCKS",
                color: Color::srgb(0.1, 0.1, 0.14),
                accent: Some((8, RED)),
            },
            Billboard {
                along: 3250.0,
                side: Side::Left,
                text: "THE SLIDE",
                color: Color::srgb(0.08, 0.16, 0.18),
                accent: Some((4, BLUE)),
            },
            Billboard {
                along: 200.0,
                side: Side::Right,
                text: "PLUSH",
                color: Color::srgb(0.18, 0.1, 0.14),
                accent: Some((5, YELLOW)),
            },
        ],
        lamps: &[Row {
            span: LAP,
            side: Side::Left,
            lateral: WALL_DISTANCE + 1.2,
            every: 48.0,
        }],
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
        chevrons: &[],
        grandstands: &[Grandstand {
            along: 120.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.0,
            length: 34.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Shop,
        sunflowers: &[],
        totems: &[],
        loops: &[Loop {
            along: 2500.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 15.0,
            radius: 16.0,
        }],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Interior,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        japan: None,
        space: None,
        toys: Some(TOYS),
    },
};

/// The shop's water table, sunk into the floor beside the train shelves.
const WATER_TABLE: WaterTable = WaterTable {
    center: Vec2::new(1020.0, -120.0),
    radius: 32.0,
};

/// The floor outside the water table, and the water in it: a signed distance
/// to the table's rim.
fn shop_floor_level(point: Vec2) -> f32 {
    point.distance(WATER_TABLE.center) - WATER_TABLE.radius
}

fn no_mountains(_: Vec2) -> f32 {
    0.0
}

/// A mountain of papier-mâché the train tunnel runs through.
const MASSIFS: [Massif; 1] = [massif(870.0, -600.0, 74.0, 34.0)];

/// Rock faces where the shop floor is cut away beside the road.
const CUTTINGS: [Cutting; 2] = [
    Cutting {
        span: span(2300.0, 2680.0),
        side: Side::Left,
        height: 12.0,
    },
    Cutting {
        span: span(560.0, 980.0),
        side: Side::Right,
        height: 9.0,
    },
];

const CONTROL_POINTS: [Vec2; 20] = [
    // Start straight, towards -Z.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -220.0),
    Vec2::new(0.0, -440.0),
    // Scalextric sweeper, east.
    Vec2::new(30.0, -640.0),
    Vec2::new(110.0, -780.0),
    Vec2::new(250.0, -870.0),
    Vec2::new(430.0, -900.0),
    Vec2::new(610.0, -860.0),
    Vec2::new(760.0, -760.0),
    // Train table, south down the east side.
    Vec2::new(860.0, -600.0),
    Vec2::new(900.0, -420.0),
    Vec2::new(900.0, -240.0),
    Vec2::new(860.0, -60.0),
    Vec2::new(780.0, 80.0),
    Vec2::new(650.0, 180.0),
    // Hot Wheels and the blocks, west along the table.
    Vec2::new(490.0, 240.0),
    Vec2::new(350.0, 360.0),
    // Plush aisle and back to the checkout.
    Vec2::new(180.0, 210.0),
    Vec2::new(0.0, 150.0),
    Vec2::new(0.0, 60.0),
];

/// A low shelf tabletop at 22 m for the train and the Hot Wheels track, with
/// the slide coming back down to the shop floor.
const ELEVATION: [(f32, f32); 24] = [
    // Two toy ramps on the start straight.
    (0.0, 0.0),
    (120.0, 0.0),
    (160.0, 2.7),
    (200.0, 0.0),
    (260.0, 0.0),
    (300.0, 2.2),
    (340.0, 0.0),
    (440.0, 0.4),
    (900.0, 0.5),
    (1150.0, 1.0),
    (1400.0, 11.0),
    (1650.0, 18.5),
    (1950.0, 22.0),
    (2120.0, 23.5),
    (2220.0, 28.5),
    (2320.0, 23.5),
    (2470.0, 25.5),
    (2570.0, 30.0),
    (2670.0, 25.5),
    (2800.0, 23.5),
    (2980.0, 21.0),
    (3100.0, 17.0),
    (3250.0, 8.0),
    (3340.0, 1.0),
];

/// The shop around the track: walls and ceiling well clear of the road and
/// its tallest toys.
const ROOM: Room = Room {
    center: Vec2::new(450.0, -270.0),
    half: Vec2::new(640.0, 700.0),
    height: 112.0,
};

/// The toys of the store.
const TOYS: Toys = Toys {
    room: ROOM,
    water_table: Some(WATER_TABLE),
    shelves: &[
        // Along the Scalextric sweeper, on both sides.
        Shelf {
            span: span(650.0, 1100.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 4.0,
            every: 26.0,
            height: 26.0,
            depth: 6.0,
            colors: SHELF_COLORS,
        },
        Shelf {
            span: span(650.0, 1100.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.0,
            every: 26.0,
            height: 26.0,
            depth: 6.0,
            colors: SHELF_COLORS,
        },
        // The plush aisle.
        Shelf {
            span: span(100.0, 400.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 4.0,
            every: 30.0,
            height: 22.0,
            depth: 5.0,
            colors: SHELF_COLORS,
        },
        Shelf {
            span: span(100.0, 400.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.0,
            every: 30.0,
            height: 22.0,
            depth: 5.0,
            colors: SHELF_COLORS,
        },
    ],
    trains: &[ToyTrain {
        along: 760.0,
        side: Side::Left,
        lateral: 42.0,
        radius: 26.0,
        cars: 4,
        speed: 7.0,
    }],
    tables: TABLES,
    block_piles: &[
        ToyBlockPile {
            along: 1000.0,
            side: Side::Left,
            lateral: 26.0,
            count: 22,
            seed: 11,
        },
        ToyBlockPile {
            along: 1080.0,
            side: Side::Right,
            lateral: 30.0,
            count: 16,
            seed: 29,
        },
    ],
    dice: &[
        GiantDice {
            along: 900.0,
            side: Side::Right,
            lateral: 22.0,
            size: 9.0,
        },
        GiantDice {
            along: 960.0,
            side: Side::Left,
            lateral: 20.0,
            size: 7.0,
        },
    ],
    balls: &[
        GiantBall {
            along: 820.0,
            side: Side::Left,
            lateral: 24.0,
            radius: 7.0,
            color: ORANGE,
        },
        GiantBall {
            along: 1060.0,
            side: Side::Right,
            lateral: 26.0,
            radius: 5.5,
            color: BLUE,
        },
    ],
};

/// Tables with toy railways on them, and chairs round them, on a rug.
const TABLES: &[ToyTable] = &[
    ToyTable {
        along: 520.0,
        side: Side::Right,
        lateral: 38.0,
        length: 46.0,
        width: 28.0,
        height: 14.0,
        chairs: 6,
        rug: BLUE,
    },
    ToyTable {
        along: 1300.0,
        side: Side::Left,
        lateral: 44.0,
        length: 42.0,
        width: 26.0,
        height: 13.0,
        chairs: 4,
        rug: RED,
    },
];

const SHELF_COLORS: &[Color] = &[RED, YELLOW, BLUE, GREEN, ORANGE, CREAM];
