//! Gold Mine: a lap through a gold mine in a desert canyon at dusk, after
//! Wario's Gold Mine (Mario Kart) and Dragon Mines (Crash Team Racing):
//! ore carts on rails, timber trestles and headframes, a long gallery
//! propped with timber and lit by lanterns, red rock and a low orange sun.
//!
//! 1. The start straight runs north through the mining town, past its
//!    wooden houses, lanterns and a train of ore carts, over a crest where
//!    the bike takes off.
//! 2. A long right-hander climbs east past a headframe to the mouth of the
//!    gallery: nearly 300 m through the mesa, curving south, between timber
//!    sets with lanterns hanging from them.
//! 3. Out into a red canyon, south in S-bends between rock walls, under a
//!    trestle where a cart crosses over the road from one wall to the other.
//! 4. Round the east side and down to the river: a wooden bridge crosses the
//!    gorge, with rails and ore carts along the bank.
//! 5. West past the second headframe and heaps of ore, and round the last
//!    corner back into town.

use bevy::prelude::*;

use super::{Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const TOWN: Span = span(2140.0, 300.0);
const GALLERY: Span = span(560.0, 845.0);
const CANYON: Span = span(880.0, 1200.0);
const BRIDGE: Span = span(1700.0, 1765.0);

/// The mine's colours: weathered gold, rust red and cream.
const GOLD: Color = Color::srgb(0.85, 0.62, 0.12);
const RUST: Color = Color::srgb(0.62, 0.2, 0.08);
const CREAM: Color = Color::srgb(0.95, 0.86, 0.62);

pub const GOLD_MINE: Circuit = Circuit {
    name: "Mina Dorada",
    name_en: "Gold Mine",
    description: "Inspirada en las minas de Mario Kart y Crash: vagonetas,\n\
                  galerías apuntaladas y un cañón rojo al atardecer",
    description_en: "Inspired by the mines of Mario Kart and Crash: ore carts,\n\
                     timbered galleries and a red canyon at dusk",
    // "Toll of the Iron Mile".
    song: Song {
        mp3: include_bytes!("../../assets/music/gold_mine.mp3"),
        loudness: -11.9,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // The last corner into town, banked.
        banking: &[Banking {
            from: 1960.0,
            to: 2110.0,
            factor: 1.4,
        }],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::Function(river),
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
        tunnels: &[GALLERY],
        bridges: &[BRIDGE],
        gantries: &[
            Gantry {
                along: 0.0,
                color: GOLD,
                text: Some("GOLD MINE"),
            },
            Gantry {
                along: 1240.0,
                color: RUST,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1900.0,
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
                along: 1480.0,
                side: Side::Right,
                text: "DANGER TNT",
                color: Color::srgb(0.75, 0.1, 0.07),
                accent: None,
            },
            Billboard {
                along: 2010.0,
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
                along: 300.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 2000.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
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
        mine: Some(Mine {
            rails: &[
                // Through town, on the right of the start straight.
                Rails {
                    span: span(20.0, 270.0),
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 3,
                },
                // Along the river bank, up to the bridge.
                Rails {
                    span: span(1450.0, 1660.0),
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 3,
                },
                // From the bridge to the second headframe.
                Rails {
                    span: span(1800.0, 1990.0),
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 6.0,
                    carts: 2,
                },
            ],
            trestles: &[Trestle { along: 1040.0 }],
            headframes: &[
                Headframe {
                    along: 400.0,
                    side: Side::Left,
                    lateral: 48.0,
                },
                Headframe {
                    along: 1880.0,
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
                    span: span(300.0, 545.0),
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 35.0,
                },
                Row {
                    span: span(1440.0, 2140.0),
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 1.2,
                    every: 40.0,
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
                    along: 470.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 12.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 1330.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 12.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 1560.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 8.0,
                    kind: PileKind::Supplies,
                },
                Pile {
                    along: 1950.0,
                    side: Side::Right,
                    lateral: WALL_DISTANCE + 24.0,
                    kind: PileKind::Ore,
                },
                Pile {
                    along: 2060.0,
                    side: Side::Left,
                    lateral: WALL_DISTANCE + 9.0,
                    kind: PileKind::Supplies,
                },
            ],
        }),
    },
};

const CONTROL_POINTS: [Vec2; 26] = [
    // 1. The start straight, north (-Z) through town.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -140.0),
    Vec2::new(0.0, -280.0),
    // 2. The long right-hander, climbing east to the gallery, and through
    //    the mesa, curving south.
    Vec2::new(18.0, -370.0),
    Vec2::new(70.0, -430.0),
    Vec2::new(145.0, -460.0),
    Vec2::new(230.0, -470.0),
    Vec2::new(315.0, -465.0),
    Vec2::new(395.0, -440.0),
    Vec2::new(460.0, -395.0),
    // 3. The canyon, south in S-bends.
    Vec2::new(505.0, -330.0),
    Vec2::new(530.0, -250.0),
    Vec2::new(545.0, -165.0),
    Vec2::new(575.0, -90.0),
    // 4. Round the east side and down to the river, and west over it.
    Vec2::new(610.0, -20.0),
    Vec2::new(620.0, 60.0),
    Vec2::new(590.0, 135.0),
    Vec2::new(525.0, 180.0),
    Vec2::new(445.0, 200.0),
    Vec2::new(360.0, 205.0),
    Vec2::new(275.0, 205.0),
    // 5. West past the headframe, and the last corner into town.
    Vec2::new(195.0, 215.0),
    Vec2::new(120.0, 230.0),
    Vec2::new(60.0, 215.0),
    Vec2::new(22.0, 170.0),
    Vec2::new(5.0, 100.0),
];

/// A crest in town where the bike takes off, the climb to the gallery and
/// its high ground, the canyon, and down to the river.
const ELEVATION: [(f32, f32); 27] = [
    (0.0, 0.0),
    (120.0, 0.8),
    // Crest: flat out, the bike flies.
    (200.0, 5.5),
    (250.0, 3.2),
    (300.0, 2.5),
    (380.0, 4.5),
    (460.0, 8.0),
    (540.0, 12.0),
    (620.0, 15.0),
    (700.0, 16.5),
    (780.0, 16.0),
    (860.0, 14.0),
    (950.0, 12.0),
    (1040.0, 11.0),
    (1120.0, 12.0),
    (1200.0, 10.5),
    (1280.0, 8.5),
    (1360.0, 7.5),
    (1440.0, 8.0),
    (1560.0, 9.0),
    (1680.0, 9.5),
    (1780.0, 9.5),
    (1860.0, 8.0),
    (1940.0, 5.5),
    (2010.0, 3.8),
    (2080.0, 2.2),
    (2160.0, 0.8),
];

/// The river, from the south up under the bridge and into a lake inside the
/// lap, and its width either way (m).
const RIVER: [Vec2; 5] = [
    Vec2::new(330.0, 1200.0),
    Vec2::new(322.0, 600.0),
    Vec2::new(320.0, 300.0),
    Vec2::new(318.0, 205.0),
    Vec2::new(305.0, 90.0),
];
const RIVER_WIDTH: f32 = 14.0;
const LAKE: Vec2 = Vec2::new(300.0, 20.0);
const LAKE_RADIUS: f32 = 70.0;

/// How far `point` is from the water: negative in the river and the lake.
fn river(point: Vec2) -> f32 {
    let to_river = RIVER
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let ab = b - a;
            let t = ((point - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            point.distance(a + ab * t)
        })
        .fold(f32::MAX, f32::min);
    (to_river - RIVER_WIDTH).min(point.distance(LAKE) - LAKE_RADIUS)
}

const MASSIFS: [Massif; 8] = [
    // The mesa the gallery goes through.
    massif(300.0, -510.0, 170.0, 46.0),
    // Inside the lap: buttes between the canyon and the lake.
    massif(420.0, -250.0, 90.0, 34.0),
    massif(170.0, -230.0, 85.0, 30.0),
    massif(140.0, 60.0, 60.0, 20.0),
    // The gorge south of the bridge, between two mesas.
    massif(200.0, 400.0, 110.0, 40.0),
    massif(450.0, 410.0, 120.0, 48.0),
    // Outside the canyon, to the east.
    massif(740.0, -200.0, 130.0, 55.0),
    // Behind town, to the west.
    massif(-160.0, -180.0, 100.0, 32.0),
];

const CUTTINGS: [Cutting; 5] = [
    // The rock round the mouths of the gallery.
    Cutting {
        span: span(530.0, 875.0),
        side: Side::Left,
        height: 26.0,
    },
    Cutting {
        span: span(530.0, 875.0),
        side: Side::Right,
        height: 26.0,
    },
    // The canyon, walls on both sides.
    Cutting {
        span: CANYON,
        side: Side::Left,
        height: 24.0,
    },
    Cutting {
        span: CANYON,
        side: Side::Right,
        height: 22.0,
    },
    // Rock on the outside of the turn round the east side.
    Cutting {
        span: span(1200.0, 1420.0),
        side: Side::Left,
        height: 18.0,
    },
];

/// Red mesas and buttes all round the horizon, with flat tops and sheer
/// sides, as in a desert of the American south-west.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(300.0, -130.0);
    let away = point.distance(CENTER);
    let shape = fbm(point / 420.0, 3);
    let mesas = smoothstep(0.47, 0.53, shape) * (0.7 + 0.3 * fbm(point / 900.0 + 7.0, 2));
    smoothstep(1000.0, 1700.0, away) * mesas * 240.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    #[test]
    fn the_river_runs_under_the_bridge_into_the_lake() {
        let track = Track::new(&GOLD_MINE.layout);
        let lap = track.lap_length();
        // The road crosses the water only on the bridge.
        for index in 0..track.sample_count() {
            let along = index as f32 * track.step_length();
            let wet = river(track.centerline()[index]) < 0.0;
            assert!(!wet || BRIDGE.contains(along, lap), "wet road at {along} m");
        }
        assert!(river(LAKE) < 0.0);
        assert!(river(Vec2::new(0.0, -100.0)) > 100.0, "town is dry");
    }
}
