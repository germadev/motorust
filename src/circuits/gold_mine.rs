//! Gold Mine: a lap through a gold mine in a desert at dusk, after Wario's
//! Gold Mine (Mario Kart) and Dragon Mines (Crash Team Racing), with the
//! roads of West Way (Moto Racer) round it: ore carts on rails, timber
//! trestles and headframes, red rock, dunes and a low orange sun. It
//! crosses the mountain twice, in a figure of eight: first through it, by
//! the galleries of the mine, and then over it.
//!
//! 1. The start straight runs north through the mining town, past its
//!    wooden houses, lanterns and a train of ore carts, over a crest where
//!    the bike takes off.
//! 2. Into the dunes: up and down over them, past a gas station on a blind
//!    crest where the bike takes off, and round their far end.
//! 3. South down a narrow red canyon in esses, between sheer walls, to the
//!    east mouth of the excavation, under a trestle where a cart crosses.
//! 4. Through the mountain (900 m), east to west: a timbered gallery, halls
//!    of sandstone, slate and clay, down into the mine and up again, and
//!    twice the road splits three ways, far apart, each way with its own
//!    climb or drop: a high road and a deep one, an upper gallery and a
//!    lower one, and the lap snaking between them. None is clearly the
//!    quickest. Between the splits, a timbered neck runs under the road over
//!    the top.
//! 5. Out of the west mouth and along the rim of a great open pit.
//! 6. The wooden bridge, straight across the lake at the bottom of the pit
//!    for nearly half a kilometre, 18 m over the red water, with a crest in
//!    the middle where the bike takes off.
//! 7. Up the switchbacks of the mountain's north flank, round two hairpins,
//!    and a long sweep onto the top, 55 m up.
//! 8. Over the top, past a headframe, across the road through the mountain
//!    70 m beneath.
//! 9. Down its south-east flank in esses and back into town.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const TOWN: Span = span(5780.0, 300.0);
const DUNES: Span = span(300.0, 990.0);
const CANYON: Span = span(990.0, 1655.0);
const EXCAVATION: Span = span(1655.0, 2555.0);
const CLIFF: Span = span(2555.0, 2905.0);
const BRIDGE: Span = span(2905.0, 3425.0);
/// How far from the centre line the fences of the wooden bridge stand (m):
/// the bridge is narrower than the asphalt of the road.
const BRIDGE_WALL: f32 = 6.0;
const CLIMB: Span = span(3425.0, 4580.0);
const HAIRPINS: Span = span(3600.0, 4200.0);
const GULLY: Span = span(4200.0, 4390.0);
const SUMMIT: Span = span(4580.0, 5000.0);
const DESCENT: Span = span(5000.0, 5780.0);
/// Where the road splits three ways under the mountain.
const FIRST_SPLIT: (f32, f32) = (1706.0, 2056.0);
const SECOND_SPLIT: (f32, f32) = (2156.0, 2504.0);

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
    // A mine in the desert: the wind and the cicadas in the heat.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.1,
        cicadas: 0.75,
        wind: 0.7,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/gold_mine.mp3"),
        length: 177.9,
        loudness: -16.8,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Hardly banked under the mountain, where the road snakes through
        // its splits, and less than their curves would have it round the
        // hairpins, cut into the mountainside.
        banking: &[
            Banking {
                from: EXCAVATION.from,
                to: EXCAVATION.to,
                factor: 0.25,
            },
            Banking {
                from: HAIRPINS.from,
                to: HAIRPINS.to,
                factor: 0.6,
            },
        ],
        // The wooden bridge is narrower than the road: its fences close in
        // on it.
        narrows: &[Narrowing {
            from: BRIDGE.from,
            to: BRIDGE.to,
            wall: BRIDGE_WALL,
        }],
        // Under the mountain the road splits three ways, twice: a way to the
        // left of the lap and one to its right, past pillars of rock, one
        // over a hump and the other through a dip.
        branches: &[
            // The high road, to the left, over a hump.
            Branch {
                from: FIRST_SPLIT.0,
                to: FIRST_SPLIT.1,
                approach: &HIGH_ROAD,
                vertical_loop: None,
                exit: &[],
                rise: &HIGH_ROAD_RISE,
            },
            // The deep road, to the right, through a dip.
            Branch {
                from: FIRST_SPLIT.0,
                to: FIRST_SPLIT.1,
                approach: &DEEP_ROAD,
                vertical_loop: None,
                exit: &[],
                rise: &DEEP_ROAD_RISE,
            },
            // The upper gallery, to the left, over a hump.
            Branch {
                from: SECOND_SPLIT.0,
                to: SECOND_SPLIT.1,
                approach: &UPPER_GALLERY,
                vertical_loop: None,
                exit: &[],
                rise: &UPPER_GALLERY_RISE,
            },
            // The lower gallery, to the right, through a dip.
            Branch {
                from: SECOND_SPLIT.0,
                to: SECOND_SPLIT.1,
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
        // Stone parapets along the rim of the pit and up the switchbacks,
        // timber fences everywhere else.
        walls: &[
            WallRun {
                span: CLIFF,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CLIFF,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CLIMB,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CLIMB,
                side: Side::Right,
                style: WallStyle::Stone,
            },
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
        // Planks on log piers across the lake, and a steel truss over the
        // gully between the shoulder of the mountain and its flank.
        bridges: &[
            Bridge {
                span: BRIDGE,
                style: BridgeStyle::Wooden,
            },
            Bridge {
                span: GULLY,
                style: BridgeStyle::Truss,
            },
        ],
        gantries: &[
            Gantry {
                along: 0.0,
                color: GOLD,
                text: Some("GOLD MINE"),
            },
            // The checkpoints, about a kilometre apart (none in the
            // excavation, where the road splits, nor on the bridges): down
            // the canyon, out on the cliff, off the lake bridge, past the
            // gully and on the way down to town.
            Gantry {
                along: 1200.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2640.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3540.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4440.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 5250.0,
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
                along: 2720.0,
                side: Side::Left,
                text: "DANGER TNT",
                color: Color::srgb(0.75, 0.1, 0.07),
                accent: None,
            },
            Billboard {
                along: 5680.0,
                side: Side::Left,
                text: "MINE 49",
                color: Color::srgb(0.25, 0.18, 0.12),
                accent: Some((5, GOLD)),
            },
        ],
        lamps: &[],
        trees: &[],
        forests: &[],
        // The mining town: weathered wood, false fronts and porches.
        houses: Houses {
            max: 10,
            paints: &[
                Color::srgb(0.62, 0.45, 0.3),
                Color::srgb(0.72, 0.56, 0.38),
                Color::srgb(0.55, 0.36, 0.26),
                Color::srgb(0.8, 0.66, 0.48),
            ],
            spans: &[TOWN],
            style: HouseStyle::FalseFront,
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[],
        chevrons: &[],
        grandstands: &[Grandstand {
            along: 60.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.0,
            length: 30.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Mine,
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
        // The roadside of West Way (Moto Racer).
        desert: Some(Desert {
            // On the right of the blind crest in the dunes.
            gas_stations: &[GasStation {
                along: 600.0,
                side: Side::Right,
                lateral: 32.0,
            }],
            pumpjacks: &PUMPJACKS,
            // High over the red canyon, from one wall to the other.
            overpasses: &[Overpass {
                along: 1300.0,
                height: 14.0,
            }],
            // On the outside of the bend at the foot of the pueblo's easing.
            pueblos: &[Pueblo {
                along: 5450.0,
                side: Side::Left,
                lateral: 34.0,
            }],
            curve_signs: &CURVE_SIGNS,
            cacti: Cacti {
                count: 90,
                spans: &[TOWN, DUNES, CLIFF, CLIMB, SUMMIT, DESCENT],
            },
            // Blown across the road in town, in the dunes, along the rim of
            // the pit, over the top and on the way down.
            tumbleweeds: &[
                Tumbleweed {
                    along: 230.0,
                    reach: 35.0,
                },
                Tumbleweed {
                    along: 530.0,
                    reach: 40.0,
                },
                Tumbleweed {
                    along: 930.0,
                    reach: 35.0,
                },
                Tumbleweed {
                    along: 2700.0,
                    reach: 30.0,
                },
                Tumbleweed {
                    along: 4930.0,
                    reach: 35.0,
                },
                Tumbleweed {
                    along: 5250.0,
                    reach: 35.0,
                },
                Tumbleweed {
                    along: 5620.0,
                    reach: 35.0,
                },
                Tumbleweed {
                    along: 5860.0,
                    reach: 30.0,
                },
            ],
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
                // Out of the west mouth, along the rim of the pit.
                Rails {
                    span: span(2600.0, 2800.0),
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 2,
                },
            ],
            // In the canyon, just before the east mouth.
            trestles: &[Trestle { along: 1625.0 }],
            headframes: &[
                // Over a shaft on the top of the mountain.
                Headframe {
                    along: 4890.0,
                    side: Side::Left,
                    lateral: 48.0,
                },
                // By the road along the rim of the pit.
                Headframe {
                    along: 2770.0,
                    side: Side::Left,
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
                    span: CLIFF,
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
                    span: span(4830.0, SUMMIT.to),
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
                    span: span(1712.0, 1880.0),
                    stone: Stone::Sandstone,
                    crown: 28.0,
                },
                Hall {
                    span: span(1880.0, 2050.0),
                    stone: Stone::Slate,
                    crown: 34.0,
                },
                Hall {
                    span: span(2162.0, 2330.0),
                    stone: Stone::Clay,
                    crown: 24.0,
                },
                Hall {
                    span: span(2330.0, 2500.0),
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
                    along: 840.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 10.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 2700.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 14.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 2900.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
                Pile {
                    along: 4660.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 12.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 5620.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
            ],
        }),
    },
};

/// Diamond warning signs before the bends, on the outside of each: green
/// for a bend, yellow for a sharp one, red for a hairpin.
const CURVE_SIGNS: [CurveSign; 11] = [
    // Round the far end of the dunes.
    curve_sign(720.0, ChevronDirection::Left, SignColor::Red),
    // The esses of the canyon.
    curve_sign(1100.0, ChevronDirection::Left, SignColor::Green),
    curve_sign(1215.0, ChevronDirection::Right, SignColor::Green),
    curve_sign(1495.0, ChevronDirection::Right, SignColor::Yellow),
    // Along the rim of the pit, before the bend onto the bridge.
    curve_sign(2810.0, ChevronDirection::Right, SignColor::Green),
    // The hairpins.
    curve_sign(3670.0, ChevronDirection::Right, SignColor::Red),
    curve_sign(3940.0, ChevronDirection::Left, SignColor::Red),
    // Down the esses into town.
    curve_sign(5065.0, ChevronDirection::Left, SignColor::Yellow),
    curve_sign(5150.0, ChevronDirection::Right, SignColor::Yellow),
    curve_sign(5315.0, ChevronDirection::Left, SignColor::Green),
    curve_sign(5405.0, ChevronDirection::Right, SignColor::Yellow),
];

const fn curve_sign(along: f32, direction: ChevronDirection, color: SignColor) -> CurveSign {
    CurveSign {
        along,
        direction,
        color,
    }
}

/// Oil pumpjacks nodding on the tops of the dunes.
const PUMPJACKS: [Pumpjack; 6] = [
    Pumpjack {
        position: Vec2::new(-30.0, -560.0),
        heading: 1.0,
    },
    Pumpjack {
        position: Vec2::new(-40.0, -612.0),
        heading: 1.3,
    },
    Pumpjack {
        position: Vec2::new(-45.0, -505.0),
        heading: 0.8,
    },
    Pumpjack {
        position: Vec2::new(215.0, -610.0),
        heading: -0.6,
    },
    Pumpjack {
        position: Vec2::new(195.0, -660.0),
        heading: -0.3,
    },
    Pumpjack {
        position: Vec2::new(30.0, -965.0),
        heading: 2.0,
    },
];

const CONTROL_POINTS: [Vec2; 95] = [
    // 1. The start straight, north (-Z) through town.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -140.0),
    Vec2::new(0.0, -280.0),
    // 2. Into the dunes: a kink to the right, the straight past the gas
    // station, and round to the left at their far end.
    Vec2::new(3.0, -380.0),
    Vec2::new(28.0, -455.0),
    Vec2::new(52.0, -526.0),
    Vec2::new(76.0, -597.0),
    Vec2::new(100.0, -668.0),
    Vec2::new(116.0, -728.0),
    Vec2::new(104.0, -782.0),
    Vec2::new(60.0, -812.0),
    Vec2::new(0.0, -810.0),
    Vec2::new(-55.0, -782.0),
    // 3. South down the red canyon, in esses, to the east mouth.
    Vec2::new(-100.0, -745.0),
    Vec2::new(-140.0, -700.0),
    Vec2::new(-162.0, -645.0),
    Vec2::new(-158.0, -590.0),
    Vec2::new(-150.0, -535.0),
    Vec2::new(-170.0, -470.0),
    Vec2::new(-205.0, -410.0),
    Vec2::new(-218.0, -345.0),
    Vec2::new(-212.0, -288.0),
    Vec2::new(-238.0, -236.0),
    // 4. Through the mountain, west: the lap snakes between the ways of two
    // splits, with the timbered neck between them.
    Vec2::new(-300.0, -215.0),
    Vec2::new(-350.0, -215.0),
    Vec2::new(-391.2, -224.1),
    Vec2::new(-432.5, -238.9),
    Vec2::new(-473.8, -237.1),
    Vec2::new(-515.0, -215.0),
    Vec2::new(-556.2, -192.9),
    Vec2::new(-597.5, -191.1),
    Vec2::new(-638.8, -205.9),
    Vec2::new(-680.0, -215.0),
    Vec2::new(-730.0, -215.0),
    Vec2::new(-780.0, -215.0),
    Vec2::new(-821.2, -205.9),
    Vec2::new(-862.5, -191.1),
    Vec2::new(-903.8, -192.9),
    Vec2::new(-945.0, -215.0),
    Vec2::new(-986.2, -237.1),
    Vec2::new(-1027.5, -238.9),
    Vec2::new(-1068.8, -224.1),
    Vec2::new(-1110.0, -215.0),
    Vec2::new(-1160.0, -215.0),
    // 5. Out of the west mouth and along the rim of the pit.
    Vec2::new(-1215.0, -224.0),
    Vec2::new(-1272.0, -248.0),
    Vec2::new(-1330.0, -282.0),
    Vec2::new(-1380.0, -330.0),
    Vec2::new(-1404.0, -395.0),
    // 6. The bridge, straight across the lake, north-east.
    Vec2::new(-1385.0, -460.0),
    Vec2::new(-1305.0, -543.0),
    Vec2::new(-1225.0, -626.0),
    Vec2::new(-1145.0, -709.0),
    Vec2::new(-1065.0, -792.0),
    // 7. Up the switchbacks of the north flank: a hairpin to the right, one
    // to the left, and a long sweep round onto the top.
    Vec2::new(-1010.0, -845.0),
    Vec2::new(-950.0, -872.0),
    Vec2::new(-890.0, -876.0),
    Vec2::new(-835.0, -876.0),
    Vec2::new(-780.0, -875.0),
    Vec2::new(-752.5, -867.6),
    Vec2::new(-732.4, -847.5),
    Vec2::new(-725.0, -820.0),
    Vec2::new(-732.4, -792.5),
    Vec2::new(-752.5, -772.4),
    Vec2::new(-780.0, -765.0),
    Vec2::new(-827.0, -765.0),
    Vec2::new(-875.0, -765.0),
    Vec2::new(-902.5, -757.6),
    Vec2::new(-922.6, -737.5),
    Vec2::new(-930.0, -710.0),
    Vec2::new(-922.6, -682.5),
    Vec2::new(-902.5, -662.4),
    Vec2::new(-875.0, -655.0),
    Vec2::new(-825.0, -655.0),
    Vec2::new(-772.0, -648.0),
    Vec2::new(-722.0, -622.0),
    Vec2::new(-678.0, -572.0),
    Vec2::new(-646.0, -502.0),
    Vec2::new(-630.0, -422.0),
    Vec2::new(-640.0, -338.0),
    Vec2::new(-685.0, -265.0),
    // 8. Over the top, south, across the road through the mountain beneath.
    Vec2::new(-730.0, -200.0),
    Vec2::new(-742.0, -130.0),
    Vec2::new(-748.0, -50.0),
    Vec2::new(-722.0, 30.0),
    // 9. Down the south-east flank, in esses, and back into town.
    Vec2::new(-660.0, 90.0),
    Vec2::new(-580.0, 110.0),
    Vec2::new(-520.0, 160.0),
    Vec2::new(-440.0, 190.0),
    Vec2::new(-350.0, 170.0),
    Vec2::new(-270.0, 200.0),
    Vec2::new(-180.0, 215.0),
    Vec2::new(-90.0, 180.0),
    Vec2::new(-30.0, 132.0),
    Vec2::new(-4.0, 72.0),
];

/// Up from town over the dunes and the canyon to the east mouth; through the
/// mountain, down into the mine and up again; along the rim of the pit and
/// over the lake; up the switchbacks, 55 m, over the top and down again
/// into town.
const ELEVATION: [(f32, f32); 62] = [
    // Through town, over a crest where the bike flies.
    (0.0, 2.0),
    (110.0, 2.4),
    (185.0, 6.6),
    (225.0, 5.0),
    (300.0, 2.6),
    // Up onto the dunes, over a blind crest on the straight by the gas
    // station, down into a hollow round the far end and up again.
    (380.0, 3.4),
    (450.0, 8.0),
    (520.0, 13.0),
    (595.0, 17.2),
    (635.0, 15.6),
    (700.0, 13.0),
    (780.0, 7.0),
    (860.0, 6.5),
    (940.0, 11.0),
    // Rolling up the red canyon to the east mouth.
    (1010.0, 14.0),
    (1090.0, 11.5),
    (1180.0, 15.5),
    (1270.0, 14.0),
    (1360.0, 18.5),
    (1450.0, 17.0),
    (1540.0, 20.5),
    (1610.0, 23.0),
    // Through the mountain: down past the first split to the neck, under
    // the road over the top, and up past the second.
    (1660.0, 22.5),
    (1706.0, 20.0),
    (2062.0, 8.0),
    (2112.0, 5.5),
    (2162.0, 6.5),
    (2516.0, 18.0),
    (2566.0, 21.0),
    // Rolling along the rim of the pit.
    (2640.0, 24.0),
    (2740.0, 19.5),
    (2830.0, 21.5),
    (2920.0, 18.6),
    // Over the lake, with a crest in the middle of the bridge where the
    // bike takes off.
    (2960.0, 17.8),
    (3160.0, 17.6),
    (3230.0, 21.8),
    (3275.0, 20.4),
    (3340.0, 18.2),
    (3400.0, 17.6),
    // Up the switchbacks and round onto the top.
    (3480.0, 19.0),
    (3600.0, 23.6),
    (3760.0, 29.2),
    (3880.0, 33.4),
    (4000.0, 37.6),
    (4150.0, 42.8),
    (4300.0, 50.0),
    (4450.0, 59.0),
    (4600.0, 68.4),
    (4716.0, 74.3),
    // Over the top.
    (4800.0, 75.5),
    (4900.0, 75.0),
    (5000.0, 72.5),
    // Down the south-east flank, easing past the pueblo, into town.
    (5100.0, 66.5),
    (5200.0, 58.0),
    (5300.0, 50.5),
    (5380.0, 46.5),
    (5450.0, 44.5),
    (5540.0, 37.0),
    (5640.0, 27.0),
    (5740.0, 17.0),
    (5830.0, 8.5),
    (5900.0, 3.5),
];

/// The ways off the lap under the mountain, each through points to one
/// side of it (m along the lap and from its centre line in the comments),
/// from where it leaves the lap to where it joins it again. The high road,
/// to the left, from the first split: it runs straight on where the lap
/// swings right, and out round a wide curve back to it.
const HIGH_ROAD: [Vec2; 11] = [
    // 1718 m, 2 m.
    Vec2::new(-361.7, -215.2),
    // 1746 m, 15 m.
    Vec2::new(-392.9, -209.1),
    // 1768 m, 40 m.
    Vec2::new(-424.9, -194.2),
    // 1829 m, 65 m.
    Vec2::new(-455.6, -174.2),
    // 1889 m, 64 m.
    Vec2::new(-488.6, -155.8),
    // 1918 m, 57 m.
    Vec2::new(-521.6, -145.6),
    // 1935 m, 44 m.
    Vec2::new(-554.6, -148.6),
    // 1958 m, 25 m.
    Vec2::new(-587.6, -165.0),
    // 1991 m, 9 m.
    Vec2::new(-621.3, -188.9),
    // 2021 m, 3 m.
    Vec2::new(-646.1, -205.0),
    // 2042 m, 0 m.
    Vec2::new(-666.4, -212.5),
];

/// The deep road, to the right, from the first split: it swings out to the
/// right before the lap does, and back to it late.
const DEEP_ROAD: [Vec2; 11] = [
    // 1718 m, -0 m.
    Vec2::new(-361.3, -217.0),
    // 1748 m, -4 m.
    Vec2::new(-389.4, -227.9),
    // 1786 m, -14 m.
    Vec2::new(-421.7, -250.3),
    // 1814 m, -32 m.
    Vec2::new(-455.6, -272.1),
    // 1836 m, -49 m.
    Vec2::new(-488.6, -283.3),
    // 1854 m, -60 m.
    Vec2::new(-521.6, -280.7),
    // 1889 m, -65 m.
    Vec2::new(-554.6, -266.7),
    // 1967 m, -57 m.
    Vec2::new(-587.5, -247.3),
    // 2001 m, -29 m.
    Vec2::new(-615.7, -228.2),
    // 2021 m, -11 m.
    Vec2::new(-643.3, -218.3),
    // 2042 m, -1 m.
    Vec2::new(-666.1, -214.4),
];

/// The upper gallery, to the left, from the second split: it swings out to
/// the left before the lap does, and back to it late.
const UPPER_GALLERY: [Vec2; 11] = [
    // 2166 m, 0 m.
    Vec2::new(-789.9, -213.2),
    // 2197 m, 4 m.
    Vec2::new(-818.5, -202.4),
    // 2235 m, 14 m.
    Vec2::new(-851.4, -179.8),
    // 2264 m, 32 m.
    Vec2::new(-885.6, -157.9),
    // 2285 m, 49 m.
    Vec2::new(-918.6, -146.7),
    // 2304 m, 60 m.
    Vec2::new(-951.6, -149.3),
    // 2338 m, 65 m.
    Vec2::new(-984.6, -163.3),
    // 2417 m, 57 m.
    Vec2::new(-1017.5, -182.7),
    // 2450 m, 29 m.
    Vec2::new(-1045.2, -202.0),
    // 2470 m, 11 m.
    Vec2::new(-1072.2, -212.0),
    // 2492 m, 1 m.
    Vec2::new(-1096.6, -215.5),
];

/// The lower gallery, to the right, from the second split: it runs straight
/// on where the lap swings left, and out round a wide curve back to it.
const LOWER_GALLERY: [Vec2; 11] = [
    // 2166 m, -2 m.
    Vec2::new(-790.3, -215.0),
    // 2195 m, -15 m.
    Vec2::new(-821.9, -221.2),
    // 2217 m, -40 m.
    Vec2::new(-854.8, -235.9),
    // 2279 m, -65 m.
    Vec2::new(-885.6, -255.8),
    // 2339 m, -64 m.
    Vec2::new(-918.6, -274.2),
    // 2367 m, -57 m.
    Vec2::new(-951.6, -284.4),
    // 2384 m, -44 m.
    Vec2::new(-984.6, -281.4),
    // 2408 m, -25 m.
    Vec2::new(-1017.6, -265.0),
    // 2440 m, -9 m.
    Vec2::new(-1050.8, -241.3),
    // 2469 m, -3 m.
    Vec2::new(-1075.0, -225.3),
    // 2492 m, -0 m.
    Vec2::new(-1096.9, -217.4),
];

/// How much higher than the lap beside it the high road runs: over a hump
/// where it is far from the lap, broad at its top (m along it, m).
const HIGH_ROAD_RISE: [(f32, f32); 35] = [
    (55.9, 0.00),
    (61.9, 0.01),
    (67.9, 0.06),
    (73.9, 0.22),
    (79.9, 0.51),
    (85.9, 0.87),
    (91.9, 1.26),
    (97.9, 1.65),
    (103.9, 2.04),
    (109.9, 2.43),
    (115.9, 2.82),
    (121.9, 3.21),
    (127.9, 3.58),
    (133.9, 3.92),
    (139.9, 4.19),
    (145.9, 4.38),
    (151.9, 4.50),
    (157.9, 4.54),
    (163.9, 4.50),
    (169.9, 4.37),
    (175.9, 4.17),
    (181.9, 3.90),
    (187.9, 3.56),
    (193.9, 3.18),
    (199.9, 2.80),
    (205.9, 2.41),
    (211.9, 2.02),
    (217.9, 1.63),
    (223.9, 1.24),
    (229.9, 0.85),
    (235.9, 0.49),
    (241.9, 0.21),
    (247.9, 0.06),
    (253.9, 0.01),
    (259.9, 0.00),
];

/// How much higher than the lap beside it the deep road runs: through a dip
/// where it is far from the lap, gentle at its edges (m along it, m).
const DEEP_ROAD_RISE: [(f32, f32); 33] = [
    (112.3, 0.00),
    (118.3, -0.01),
    (124.3, -0.06),
    (130.3, -0.14),
    (136.3, -0.29),
    (142.3, -0.49),
    (148.3, -0.75),
    (154.3, -1.06),
    (160.3, -1.42),
    (166.3, -1.80),
    (172.3, -2.19),
    (178.3, -2.58),
    (184.3, -2.97),
    (190.3, -3.36),
    (196.3, -3.71),
    (202.3, -3.97),
    (208.3, -4.05),
    (214.3, -3.92),
    (220.3, -3.63),
    (226.3, -3.26),
    (232.3, -2.87),
    (238.3, -2.48),
    (244.3, -2.09),
    (250.3, -1.70),
    (256.3, -1.32),
    (262.3, -0.98),
    (268.3, -0.68),
    (274.3, -0.43),
    (280.3, -0.24),
    (286.3, -0.12),
    (292.3, -0.04),
    (298.3, -0.01),
    (304.3, 0.00),
];

/// How much higher than the lap beside it the upper gallery runs: over a
/// hump where it is far from the lap, broad at its top (m along it, m).
const UPPER_GALLERY_RISE: [(f32, f32); 35] = [
    (106.3, 0.00),
    (112.3, 0.01),
    (118.3, 0.06),
    (124.3, 0.22),
    (130.3, 0.51),
    (136.3, 0.87),
    (142.3, 1.26),
    (148.3, 1.65),
    (154.3, 2.04),
    (160.3, 2.43),
    (166.3, 2.82),
    (172.3, 3.20),
    (178.3, 3.58),
    (184.3, 3.91),
    (190.3, 4.17),
    (196.3, 4.36),
    (202.3, 4.47),
    (208.3, 4.50),
    (214.3, 4.45),
    (220.3, 4.32),
    (226.3, 4.11),
    (232.3, 3.83),
    (238.3, 3.48),
    (244.3, 3.11),
    (250.3, 2.72),
    (256.3, 2.33),
    (262.3, 1.94),
    (268.3, 1.55),
    (274.3, 1.16),
    (280.3, 0.77),
    (286.3, 0.42),
    (292.3, 0.16),
    (298.3, 0.04),
    (304.3, 0.01),
    (310.3, 0.00),
];

/// How much higher than the lap beside it the lower gallery runs: through a
/// dip where it is far from the lap, gentle at its edges (m along it, m).
const LOWER_GALLERY_RISE: [(f32, f32); 33] = [
    (61.9, 0.00),
    (67.9, -0.01),
    (73.9, -0.06),
    (79.9, -0.14),
    (85.9, -0.29),
    (91.9, -0.49),
    (97.9, -0.75),
    (103.9, -1.06),
    (109.9, -1.42),
    (115.9, -1.80),
    (121.9, -2.19),
    (127.9, -2.58),
    (133.9, -2.97),
    (139.9, -3.36),
    (145.9, -3.72),
    (151.9, -4.00),
    (157.9, -4.10),
    (163.9, -3.98),
    (169.9, -3.70),
    (175.9, -3.34),
    (181.9, -2.95),
    (187.9, -2.56),
    (193.9, -2.17),
    (199.9, -1.78),
    (205.9, -1.40),
    (211.9, -1.04),
    (217.9, -0.73),
    (223.9, -0.48),
    (229.9, -0.28),
    (235.9, -0.14),
    (241.9, -0.05),
    (247.9, -0.01),
    (253.9, 0.00),
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
const MOUNTAIN: Massif = climbed_massif(-720.0, -200.0, 540.0, 78.0);

const MASSIFS: [Massif; 19] = [
    // The mountain, and the shoulders the road climbs onto it from the
    // lake and comes down from it to town on.
    MOUNTAIN,
    climbed_massif(-840.0, -700.0, 250.0, 34.0),
    climbed_massif(-460.0, 160.0, 300.0, 50.0),
    climbed_massif(-140.0, 200.0, 200.0, 26.0),
    // Dunes of red sand round the road out of town.
    climbed_massif(-60.0, -560.0, 150.0, 15.0),
    climbed_massif(200.0, -600.0, 140.0, 17.0),
    climbed_massif(40.0, -950.0, 170.0, 18.0),
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
    // The sheer walls of the red canyon, down to the east mouth.
    Cutting {
        span: span(CANYON.from + 10.0, EXCAVATION.from + 35.0),
        side: Side::Left,
        height: 36.0,
    },
    Cutting {
        span: span(CANYON.from + 10.0, EXCAVATION.from + 35.0),
        side: Side::Right,
        height: 36.0,
    },
    // The rock round the west mouth.
    Cutting {
        span: span(2520.0, 2595.0),
        side: Side::Left,
        height: 30.0,
    },
    Cutting {
        span: span(2520.0, 2595.0),
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
