//! Gold Mine: a lap through a gold mine in a desert at dusk, after Wario's
//! Gold Mine (Mario Kart) and Dragon Mines (Crash Team Racing): ore carts
//! on rails, timber trestles and headframes, red rock and a low orange sun.
//! It crosses the mountain twice, in a figure of eight: first through it,
//! by the galleries of the mine, and then over it.
//!
//! 1. The start straight runs north through the mining town, past its
//!    wooden houses, lanterns and a train of ore carts, over a crest where
//!    the bike takes off.
//! 2. Round the bend at the end of town and up to the east mouth of the
//!    excavation, under a trestle where a cart crosses over the road.
//! 3. Through the mountain (825 m), east to west: a timbered gallery, halls
//!    of sandstone, slate and clay, down into the mine and up again, and
//!    twice the road splits three ways, each way with its own climb or
//!    drop: a high road and a deep one, an upper gallery and a lower one,
//!    and the lap snaking between them. None is clearly the quickest.
//!    Between the splits, a timbered neck runs under the road over the top.
//! 4. Out of the west mouth and round, past ore carts on their rails, to
//!    the shore of a great lake.
//! 5. The wooden bridge, straight across the lake for nearly half a
//!    kilometre, 18 m over the red water at the bottom of an open pit, its
//!    benches stepping down to the water all round, with a crest in the
//!    middle where the bike takes off.
//! 6. Up the mountain's north flank, 48 m, cut into its slope.
//! 7. Over the top, past a headframe, across the road through the mountain
//!    65 m beneath.
//! 8. Down its south-east flank and back into town.

use bevy::prelude::*;

use super::{Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const TOWN: Span = span(4560.0, 300.0);
const APPROACH: Span = span(300.0, 860.0);
const EXCAVATION: Span = span(875.0, 1700.0);
const EXIT: Span = span(1700.0, 2100.0);
const BRIDGE: Span = span(2105.0, 2600.0);
const CLIMB: Span = span(2620.0, 3530.0);
const SUMMIT: Span = span(3530.0, 3880.0);
const DESCENT: Span = span(3880.0, 4560.0);

/// The mine's colours: weathered gold, rust red and cream.
const GOLD: Color = Color::srgb(0.85, 0.62, 0.12);
const RUST: Color = Color::srgb(0.62, 0.2, 0.08);
const CREAM: Color = Color::srgb(0.95, 0.86, 0.62);

pub const GOLD_MINE: Circuit = Circuit {
    name: "Mina Dorada",
    name_en: "Gold Mine",
    description: "Inspirada en las minas de Mario Kart y Crash: vagonetas,\n\
                  una montaña cruzada por dentro y por encima, y una corta inundada",
    description_en: "Inspired by the mines of Mario Kart and Crash: ore carts,\n\
                     a mountain crossed through and over, and a flooded open pit",
    // "Toll of the Iron Mile".
    song: Song {
        mp3: include_bytes!("../../assets/music/gold_mine.mp3"),
        loudness: -16.8,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Hardly banked under the mountain, where the road snakes through
        // its splits.
        banking: &[Banking {
            from: 875.0,
            to: 1700.0,
            factor: 0.25,
        }],
        // Under the mountain the road splits three ways, twice: a way to the
        // left of the lap and one to its right, past pillars of rock, one
        // over a hump and the other through a dip.
        branches: &[
            // The high road, to the left, over a hump.
            Branch {
                from: 932.0,
                to: 1224.0,
                approach: &HIGH_ROAD,
                vertical_loop: None,
                exit: &[],
                rise: &HIGH_ROAD_RISE,
            },
            // The deep road, to the right, through a dip.
            Branch {
                from: 932.0,
                to: 1224.0,
                approach: &DEEP_ROAD,
                vertical_loop: None,
                exit: &[],
                rise: &DEEP_ROAD_RISE,
            },
            // The upper gallery, to the left, over a hump.
            Branch {
                from: 1334.0,
                to: 1648.0,
                approach: &UPPER_GALLERY,
                vertical_loop: None,
                exit: &[],
                rise: &UPPER_GALLERY_RISE,
            },
            // The lower gallery, to the right, through a dip.
            Branch {
                from: 1334.0,
                to: 1648.0,
                approach: &LOWER_GALLERY,
                vertical_loop: None,
                exit: &[],
                rise: &LOWER_GALLERY_RISE,
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::Function(water),
        hills: 8.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [RUST, CREAM],
        walls: &[
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Timber,
            },
        ],
        tunnels: &[EXCAVATION],
        bridges: &[BRIDGE],
        gantries: &[
            Gantry {
                along: 0.0,
                color: GOLD,
                text: Some("GOLD MINE"),
            },
            Gantry {
                along: 1780.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3720.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 110.0,
                side: Side::Left,
                text: "GOLD RUSH",
                color: RUST,
                accent: Some((5, GOLD)),
            },
            Billboard {
                along: 250.0,
                side: Side::Right,
                text: "MOTORUST",
                color: Color::srgb(0.1, 0.1, 0.12),
                accent: Some((4, crate::ui::RUST_ORANGE)),
            },
            Billboard {
                along: 1880.0,
                side: Side::Left,
                text: "DANGER TNT",
                color: Color::srgb(0.75, 0.1, 0.07),
                accent: None,
            },
            Billboard {
                along: 4420.0,
                side: Side::Left,
                text: "MINE 49",
                color: Color::srgb(0.25, 0.18, 0.12),
                accent: Some((5, GOLD)),
            },
        ],
        lamps: &[],
        trees: &[],
        forests: &[],
        // The mining town: weathered wood.
        houses: Houses {
            max: 10,
            paints: &[
                Color::srgb(0.62, 0.45, 0.3),
                Color::srgb(0.72, 0.56, 0.38),
                Color::srgb(0.55, 0.36, 0.26),
                Color::srgb(0.8, 0.66, 0.48),
            ],
            spans: &[TOWN],
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[],
        chevrons: &[
            ChevronSign {
                along: 420.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 815.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 3380.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 4680.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
        ],
        grandstands: &[Grandstand {
            along: 60.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.0,
            length: 30.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Mine,
        bridge_style: BridgeStyle::Wooden,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Dusk,
        // The great lake is the bottom of an open pit, flooded with red water.
        water: Water::Iron,
        pit: Some(Pit {
            benches: 5,
            step: 3.4,
            tread: 9.0,
            spoil: 45.0,
        }),
        mine: Some(Mine {
            rails: &[
                // Through town, on the right of the start straight.
                Rails {
                    span: span(20.0, 270.0),
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 3,
                },
                // Out of the west mouth, round towards the lake.
                Rails {
                    span: span(1760.0, 1960.0),
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 2,
                },
            ],
            trestles: &[Trestle { along: 865.0 }],
            headframes: &[
                // Over a shaft on the top of the mountain.
                Headframe {
                    along: 3770.0,
                    side: Side::Left,
                    lateral: 48.0,
                },
                // By the road down from the west mouth.
                Headframe {
                    along: 1960.0,
                    side: Side::Right,
                    lateral: 46.0,
                },
            ],
            lanterns: &[
                Row {
                    span: TOWN,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 28.0,
                },
                Row {
                    span: TOWN,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 28.0,
                },
                Row {
                    span: APPROACH,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 35.0,
                },
                Row {
                    span: EXIT,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 40.0,
                },
                Row {
                    span: CLIMB,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 40.0,
                },
                // Over the top, once past the road through the mountain
                // beneath.
                Row {
                    span: span(3660.0, SUMMIT.to),
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 40.0,
                },
                Row {
                    span: DESCENT,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 40.0,
                },
            ],
            // From the timbered mouth into a hall of red sandstone, where
            // the high and deep roads part, and a higher one of slate veined
            // with gold; a timbered neck under the road over the top; a hall
            // of clay lined with planks, where the upper and lower galleries
            // part, one of sandstone again, and timber to the way out.
            halls: &[
                Hall {
                    span: span(940.0, 1085.0),
                    stone: Stone::Sandstone,
                    crown: 28.0,
                },
                Hall {
                    span: span(1085.0, 1222.0),
                    stone: Stone::Slate,
                    crown: 34.0,
                },
                Hall {
                    span: span(1340.0, 1490.0),
                    stone: Stone::Clay,
                    crown: 24.0,
                },
                Hall {
                    span: span(1490.0, 1650.0),
                    stone: Stone::Sandstone,
                    crown: 31.0,
                },
            ],
            piles: &[
                Pile {
                    along: 160.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
                Pile {
                    along: 640.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 10.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 1830.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 14.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 2070.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
                Pile {
                    along: 3520.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 12.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 4380.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
            ],
        }),
    },
};

const CONTROL_POINTS: [Vec2; 50] = [
    // 1. The start straight, north (-Z) through town.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -140.0),
    Vec2::new(0.0, -280.0),
    // 2. Round the bend at the north end of town, and south-west up to the
    // east mouth of the excavation.
    Vec2::new(-18.0, -390.0),
    Vec2::new(-75.0, -455.0),
    Vec2::new(-165.0, -450.0),
    Vec2::new(-230.0, -380.0),
    Vec2::new(-260.0, -280.0),
    Vec2::new(-315.0, -210.0),
    // 3. Through the mountain, west, snaking through two splits.
    Vec2::new(-400.0, -199.7),
    Vec2::new(-458.0, -190.8),
    Vec2::new(-525.0, -200.0),
    Vec2::new(-592.0, -209.2),
    Vec2::new(-660.0, -200.0),
    Vec2::new(-720.0, -200.0),
    Vec2::new(-790.0, -200.0),
    Vec2::new(-862.0, -209.1),
    Vec2::new(-935.0, -200.0),
    Vec2::new(-1008.0, -190.9),
    Vec2::new(-1080.0, -200.0),
    Vec2::new(-1140.0, -200.0),
    // 4. Out of the west mouth and round, north, to the lake.
    Vec2::new(-1220.0, -208.0),
    Vec2::new(-1302.0, -240.0),
    Vec2::new(-1365.0, -300.0),
    Vec2::new(-1398.0, -375.0),
    // 5. The bridge, straight across the lake, north-east.
    Vec2::new(-1385.0, -460.0),
    Vec2::new(-1305.0, -543.0),
    Vec2::new(-1225.0, -626.0),
    Vec2::new(-1145.0, -709.0),
    Vec2::new(-1065.0, -792.0),
    // 6. Up the mountain's north flank.
    Vec2::new(-980.0, -840.0),
    Vec2::new(-885.0, -830.0),
    Vec2::new(-805.0, -745.0),
    Vec2::new(-725.0, -675.0),
    Vec2::new(-645.0, -595.0),
    Vec2::new(-575.0, -510.0),
    Vec2::new(-540.0, -420.0),
    Vec2::new(-565.0, -330.0),
    Vec2::new(-640.0, -270.0),
    // 7. Over the top, south, across the road through the mountain beneath.
    Vec2::new(-705.0, -210.0),
    Vec2::new(-745.0, -130.0),
    Vec2::new(-770.0, -45.0),
    Vec2::new(-755.0, 40.0),
    // 8. Down the south-east flank, and back into town.
    Vec2::new(-690.0, 105.0),
    Vec2::new(-580.0, 150.0),
    Vec2::new(-450.0, 185.0),
    Vec2::new(-310.0, 205.0),
    Vec2::new(-175.0, 195.0),
    Vec2::new(-65.0, 150.0),
    Vec2::new(-10.0, 80.0),
];

/// Up from town to the east mouth; through the mountain, down into the
/// mine and up again; down to the lake and over it; up the mountain, 48 m,
/// over its top and down again into town.
const ELEVATION: [(f32, f32); 57] = [
    // Through town, over a crest where the bike flies.
    (0.0, 2.0),
    (110.0, 2.4),
    (185.0, 6.6),
    (225.0, 5.0),
    // Round the bend and up to the east mouth.
    (300.0, 2.6),
    (450.0, 3.5),
    (600.0, 6.5),
    (750.0, 11.0),
    (860.0, 15.0),
    // Through the mountain: down into the mine, under the road over the top,
    // and up again to the west mouth.
    (905.0, 16.6),
    (950.0, 16.2),
    (1010.0, 13.2),
    (1070.0, 10.0),
    (1130.0, 6.8),
    (1190.0, 4.0),
    (1250.0, 2.6),
    (1300.0, 3.0),
    (1350.0, 4.4),
    (1410.0, 7.2),
    (1470.0, 10.3),
    (1530.0, 13.4),
    (1590.0, 16.4),
    (1640.0, 18.6),
    (1690.0, 19.6),
    (1740.0, 19.4),
    // Down to the lake, and over it, with a crest in the middle of the bridge
    // where the bike takes off.
    (1800.0, 19.2),
    (1900.0, 18.6),
    (2000.0, 18.1),
    (2090.0, 17.8),
    (2320.0, 17.6),
    (2390.0, 21.8),
    (2435.0, 20.4),
    (2500.0, 18.2),
    (2560.0, 17.6),
    (2600.0, 17.6),
    // Up the mountain's north flank.
    (2690.0, 19.0),
    (2780.0, 23.0),
    (2880.0, 28.5),
    (2980.0, 34.0),
    (3080.0, 39.5),
    (3180.0, 45.5),
    (3280.0, 52.0),
    (3370.0, 58.0),
    (3450.0, 62.5),
    (3530.0, 66.0),
    // Over the top.
    (3610.0, 68.0),
    (3700.0, 69.0),
    (3790.0, 68.5),
    (3870.0, 66.5),
    // Down its south-east flank into town.
    (3960.0, 62.0),
    (4080.0, 54.5),
    (4200.0, 46.5),
    (4320.0, 38.0),
    (4440.0, 29.0),
    (4560.0, 19.5),
    (4660.0, 10.5),
    (4740.0, 4.5),
];

/// The ways off the lap under the mountain, each through points to one
/// side of it (m along the lap and from its centre line in the comments),
/// from where it leaves the lap to where it joins it again. The high road,
/// to the left, from 932 m to 1224 m.
const HIGH_ROAD: [Vec2; 11] = [
    // 946 m, 1 m.
    Vec2::new(-394.5, -199.0),
    // 967 m, 3 m.
    Vec2::new(-414.8, -194.3),
    // 991 m, 7 m.
    Vec2::new(-438.0, -185.5),
    // 1018 m, 19 m.
    Vec2::new(-467.0, -172.5),
    // 1042 m, 33 m.
    Vec2::new(-496.0, -162.0),
    // 1071 m, 42 m.
    Vec2::new(-525.0, -158.0),
    // 1099 m, 42 m.
    Vec2::new(-554.0, -162.0),
    // 1133 m, 36 m.
    Vec2::new(-583.0, -172.5),
    // 1169 m, 22 m.
    Vec2::new(-612.0, -185.5),
    // 1190 m, 9 m.
    Vec2::new(-635.2, -194.3),
    // 1209 m, 1 m.
    Vec2::new(-655.5, -199.0),
];

/// The deep road, to the right, from 932 m to 1224 m.
const DEEP_ROAD: [Vec2; 11] = [
    // 945 m, -1 m.
    Vec2::new(-394.5, -201.0),
    // 964 m, -8 m.
    Vec2::new(-414.8, -205.7),
    // 986 m, -22 m.
    Vec2::new(-438.0, -214.5),
    // 1022 m, -36 m.
    Vec2::new(-467.0, -227.5),
    // 1056 m, -42 m.
    Vec2::new(-496.0, -238.0),
    // 1084 m, -42 m.
    Vec2::new(-525.0, -242.0),
    // 1112 m, -33 m.
    Vec2::new(-554.0, -238.0),
    // 1137 m, -19 m.
    Vec2::new(-583.0, -227.5),
    // 1164 m, -7 m.
    Vec2::new(-612.0, -214.5),
    // 1188 m, -2 m.
    Vec2::new(-635.2, -205.7),
    // 1209 m, -1 m.
    Vec2::new(-655.5, -201.0),
];

/// The upper gallery, to the left, from 1334 m to 1648 m.
const UPPER_GALLERY: [Vec2; 11] = [
    // 1349 m, 1 m.
    Vec2::new(-795.5, -199.0),
    // 1370 m, 9 m.
    Vec2::new(-817.2, -194.3),
    // 1393 m, 22 m.
    Vec2::new(-842.0, -185.5),
    // 1431 m, 36 m.
    Vec2::new(-873.0, -172.5),
    // 1466 m, 42 m.
    Vec2::new(-904.0, -162.0),
    // 1496 m, 42 m.
    Vec2::new(-935.0, -158.0),
    // 1527 m, 33 m.
    Vec2::new(-966.0, -162.0),
    // 1554 m, 19 m.
    Vec2::new(-997.0, -172.5),
    // 1583 m, 7 m.
    Vec2::new(-1028.0, -185.5),
    // 1609 m, 2 m.
    Vec2::new(-1052.8, -194.3),
    // 1631 m, 1 m.
    Vec2::new(-1074.5, -199.0),
];

/// The lower gallery, to the right, from 1334 m to 1648 m.
const LOWER_GALLERY: [Vec2; 11] = [
    // 1349 m, -1 m.
    Vec2::new(-795.5, -201.0),
    // 1372 m, -2 m.
    Vec2::new(-817.2, -205.7),
    // 1397 m, -7 m.
    Vec2::new(-842.0, -214.5),
    // 1426 m, -19 m.
    Vec2::new(-873.0, -227.5),
    // 1454 m, -33 m.
    Vec2::new(-904.0, -238.0),
    // 1484 m, -42 m.
    Vec2::new(-935.0, -242.0),
    // 1515 m, -42 m.
    Vec2::new(-966.0, -238.0),
    // 1550 m, -36 m.
    Vec2::new(-997.0, -227.5),
    // 1587 m, -22 m.
    Vec2::new(-1028.0, -214.5),
    // 1611 m, -9 m.
    Vec2::new(-1052.8, -205.7),
    // 1631 m, -1 m.
    Vec2::new(-1074.5, -201.0),
];

/// How much higher than the lap beside it the high road runs: over a hump where
/// it is well apart from the lap (m along it, m).
const HIGH_ROAD_RISE: [(f32, f32); 11] = [
    (88.5, 0.00),
    (108.5, 0.00),
    (124.1, 0.28),
    (139.7, 0.95),
    (155.4, 1.62),
    (171.0, 1.90),
    (186.7, 1.62),
    (202.3, 0.95),
    (217.9, 0.28),
    (233.6, 0.00),
    (253.6, 0.00),
];

/// How much higher than the lap beside it the deep road runs: through a dip where
/// it is well apart from the lap (m along it, m).
const DEEP_ROAD_RISE: [(f32, f32); 11] = [
    (48.8, 0.00),
    (68.8, 0.00),
    (84.4, -0.28),
    (100.0, -0.95),
    (115.7, -1.62),
    (131.3, -1.90),
    (146.9, -1.62),
    (162.6, -0.95),
    (178.2, -0.28),
    (193.8, 0.00),
    (213.8, 0.00),
];

/// How much higher than the lap beside it the upper gallery runs: over a hump where
/// it is well apart from the lap (m along it, m).
const UPPER_GALLERY_RISE: [(f32, f32); 11] = [
    (54.1, 0.00),
    (74.1, 0.00),
    (90.8, 0.32),
    (107.4, 1.08),
    (124.0, 1.84),
    (140.7, 2.15),
    (157.3, 1.84),
    (173.9, 1.08),
    (190.5, 0.32),
    (207.2, 0.00),
    (227.2, 0.00),
];

/// How much higher than the lap beside it the lower gallery runs: through a dip where
/// it is well apart from the lap (m along it, m).
const LOWER_GALLERY_RISE: [(f32, f32); 11] = [
    (96.1, 0.00),
    (116.1, 0.00),
    (132.8, -0.32),
    (149.4, -1.08),
    (166.0, -1.84),
    (182.6, -2.15),
    (199.3, -1.84),
    (215.9, -1.08),
    (232.5, -0.32),
    (249.2, 0.00),
    (269.2, 0.00),
];

/// The great lake north-west of the mountain, at the bottom of the open
/// pit: its middle, its radius (m),
/// and how far its shore wanders (m).
const LAKE: Vec2 = Vec2::new(-1225.0, -626.0);
const LAKE_RADIUS: f32 = 235.0;
const LAKE_SHORE: f32 = 18.0;
/// The river that flows out of the lake to the north, and its width either
/// way (m).
const RIVER: [Vec2; 4] = [
    Vec2::new(-1270.0, -830.0),
    Vec2::new(-1310.0, -1300.0),
    Vec2::new(-1380.0, -1800.0),
    Vec2::new(-1480.0, -2400.0),
];
const RIVER_WIDTH: f32 = 14.0;

/// How far `point` is from the water: negative in the lake and the river.
fn water(point: Vec2) -> f32 {
    let to_river = RIVER
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let ab = b - a;
            let t = ((point - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            point.distance(a + ab * t)
        })
        .fold(f32::MAX, f32::min);
    let shore = LAKE_SHORE * (2.0 * fbm(point / 70.0 + 3.0, 2) - 1.0);
    let to_lake = point.distance(LAKE) - LAKE_RADIUS + shore;
    (to_river - RIVER_WIDTH).min(to_lake)
}

/// The mountain the lap crosses through and over: its middle, its radius
/// and its height (m).
const MOUNTAIN: Massif = climbed_massif(-720.0, -200.0, 540.0, 70.0);

const MASSIFS: [Massif; 16] = [
    // The mountain, and the shoulders the road climbs onto it from the
    // lake and comes down from it to town on.
    MOUNTAIN,
    climbed_massif(-840.0, -700.0, 250.0, 34.0),
    climbed_massif(-460.0, 160.0, 300.0, 50.0),
    climbed_massif(-140.0, 200.0, 200.0, 26.0),
    // Towers of red rock standing up from its slopes and its top, well
    // away from the roads.
    massif(-960.0, -520.0, 60.0, 26.0),
    massif(-800.0, -440.0, 50.0, 22.0),
    massif(-1120.0, -420.0, 55.0, 24.0),
    massif(-500.0, 0.0, 45.0, 20.0),
    massif(-920.0, -20.0, 45.0, 18.0),
    massif(-1040.0, 120.0, 60.0, 26.0),
    massif(-380.0, -500.0, 50.0, 22.0),
    // Red buttes round about: west of the lake, north of the mountain,
    // south of it, and east of town.
    massif(-1600.0, -700.0, 120.0, 48.0),
    massif(-460.0, -940.0, 120.0, 40.0),
    massif(-1230.0, 150.0, 140.0, 52.0),
    massif(-640.0, 480.0, 130.0, 55.0),
    massif(250.0, -180.0, 110.0, 36.0),
];

const CUTTINGS: [Cutting; 4] = [
    // The rock round the mouths of the excavation.
    Cutting {
        span: span(835.0, 905.0),
        side: Side::Left,
        height: 30.0,
    },
    Cutting {
        span: span(835.0, 905.0),
        side: Side::Right,
        height: 30.0,
    },
    Cutting {
        span: span(1670.0, 1735.0),
        side: Side::Left,
        height: 30.0,
    },
    Cutting {
        span: span(1670.0, 1735.0),
        side: Side::Right,
        height: 30.0,
    },
];

/// Red mesas and buttes all round the horizon, with flat tops and sheer
/// sides, as in a desert of the American south-west.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(-700.0, -310.0);
    let away = point.distance(CENTER);
    let shape = fbm(point / 420.0, 3);
    let mesas = smoothstep(0.47, 0.53, shape) * (0.7 + 0.3 * fbm(point / 900.0 + 7.0, 2));
    smoothstep(1100.0, 1800.0, away) * mesas * 240.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    #[test]
    fn the_road_crosses_the_great_lake_on_the_long_bridge() {
        let track = Track::new(&GOLD_MINE.layout);
        let lap = track.lap_length();
        // The road crosses the water only on the bridge, and the bridge
        // nearly half a kilometre long, over water most of the way.
        let mut wet = 0.0;
        for index in 0..track.sample_count() {
            let along = index as f32 * track.step_length();
            let over_water = water(track.centerline()[index]) < 0.0;
            assert!(
                !over_water || BRIDGE.contains(along, lap),
                "wet road at {along} m"
            );
            if over_water {
                wet += track.step_length();
            }
        }
        assert!(BRIDGE.length(lap) > 450.0);
        assert!(wet > 0.85 * BRIDGE.length(lap), "{wet} m over water");
        // A great lake, and the town dry.
        assert!(water(LAKE) < -100.0);
        assert!(water(LAKE + Vec2::new(200.0, 0.0)) < 0.0);
        let town = track.centerline()[track.index_at(150.0)];
        assert!(water(town) > 100.0, "town is dry");
    }

    #[test]
    fn the_lap_crosses_the_mountain_through_it_and_then_over_it() {
        let track = Track::new(&GOLD_MINE.layout);
        let step = track.step_length();
        let lap = track.lap_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let point = |along: f32| track.centerline()[track.index_at(along)];
        // Through it, from one side to the other, by its middle.
        for mouth in [EXCAVATION.from, EXCAVATION.to] {
            let out = point(mouth).distance(MOUNTAIN.center) / MOUNTAIN.radius;
            assert!(out > 0.7, "a mouth {out} of the way out");
        }
        let middle = point(EXCAVATION.from + EXCAVATION.length(lap) / 2.0);
        let deep = middle.distance(MOUNTAIN.center);
        assert!(deep < 0.2 * MOUNTAIN.radius, "{deep} m from its middle");
        // Then up it, 45 m and more, steadily, over the top, and down it,
        // 60 m and more, back into town.
        assert!(height(CLIMB.to) - height(CLIMB.from) > 45.0);
        assert!(height(DESCENT.from) - height(0.0) > 60.0);
        let (first, last) = (track.index_at(CLIMB.from + 100.0), track.index_at(CLIMB.to));
        for index in first..last {
            let grade = (track.elevations()[index + 1] - track.elevations()[index]) / step;
            assert!(grade > 0.03, "{grade} at {} m", index as f32 * step);
        }
        // Over the top, the road crosses the road through the mountain,
        // high above it.
        let line = track.centerline();
        let under = track.index_at(EXCAVATION.from)..track.index_at(EXCAVATION.to);
        let (gap, over, under) = (track.index_at(SUMMIT.from)..track.index_at(SUMMIT.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the summit and the excavation have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 50.0, "{above} m above");
    }
}
