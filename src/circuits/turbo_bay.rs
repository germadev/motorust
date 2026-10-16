//! Turbo Bay, inspired by Speed Bay in Moto Racer: a figure of eight on a
//! hilly island, with the sea all around and two bays cutting in towards
//! its waist. The first loop goes clockwise and the second anticlockwise,
//! and halfway round the road climbs back over the ridge it first went
//! through in a tunnel.
//!
//! 1. The start straight is a seafront promenade heading north: railing,
//!    beach and sea on the left; palms and tall, colourful hotels on the
//!    right.
//! 2. A crest, then a steep banked right-hander drops to the water's edge
//!    and the road climbs east into a tunnel through the ridge at the
//!    island's waist.
//! 3. Out of the tunnel, past an old-style grand hotel on a headland above
//!    the south-east bay, to a pass between rocks.
//! 4. The east coast road rises and falls behind a white barrier, with the
//!    sea on the right.
//! 5. A mountain road climbs on north in S-bends between rock walls, dips,
//!    and climbs again to the summit at the island's northern tip, 52 m up,
//!    round a long left-hander with the sea and a lighthouse below.
//! 6. Down through a forest it rolls over a hump, dips and rises again, then
//!    runs down the ridge and crosses over the tunnel, down the shore of the
//!    bay to a bridge with red arches over the river mouth, through the town
//!    and back to the seafront.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const SEAFRONT: Span = span(3020.0, 250.0);
const TUNNEL: Span = span(505.0, 625.0);
const ROCK_PASS: Span = span(870.0, 990.0);
const EAST_COAST: Span = span(1000.0, 1280.0);
const MOUNTAIN: Span = span(1280.0, 1660.0);
const FOREST: Span = span(1660.0, 2160.0);
const BAY_SHORE: Span = span(2460.0, 2710.0);
const BRIDGE: Span = span(2720.0, 2805.0);
const TOWN: Span = span(2830.0, 3010.0);

const RED: Color = Color::srgb(0.8, 0.08, 0.06);

pub const TURBO_BAY: Circuit = Circuit {
    name: "Turbo Bay",
    name_en: "Turbo Bay",
    description: "Inspirada en Moto Racer: paseo marítimo, túnel bajo\n\
                  la propia pista, puerto de montaña y puente rojo",
    description_en: "Inspired by Moto Racer: seafront, a tunnel under\n\
                     the track itself, a mountain pass and red bridge",
    // A bay: the sea and many gulls, and the wind off it.
    ambience: Ambience {
        sea: 0.9,
        gulls: 0.9,
        birds: 0.15,
        cicadas: 0.0,
        wind: 0.35,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/turbo_bay.mp3"),
        length: 163.1,
        loudness: -16.0,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // The drop to the water after the seafront, banked like a velodrome.
        banking: &[Banking {
            from: 330.0,
            to: 470.0,
            factor: 2.5,
        }],
        narrows: &[],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::Outline(&COAST),
        hills: 22.0,
        greens: Greens::Lawn,
        grass_shade: 1.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[
            // The grand hotel, on its headland above the bay.
            Terrace {
                center: Vec2::new(410.0, -237.0),
                radius: 32.0,
                height: 15.0,
                tilt: Vec2::ZERO,
                skirt: 15.0,
            },
        ],
        mountains,
        curbs: [Color::srgb(0.1, 0.25, 0.75), Color::srgb(0.95, 0.95, 0.95)],
        walls: &[
            WallRun {
                span: SEAFRONT,
                side: Side::Left,
                style: WallStyle::Railing,
            },
            WallRun {
                span: TOWN,
                side: Side::Left,
                style: WallStyle::Railing,
            },
            WallRun {
                span: SEAFRONT,
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: EAST_COAST,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            // The mountain road, with the sea on the right.
            WallRun {
                span: MOUNTAIN,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: BAY_SHORE,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
        ],
        tunnels: &[TUNNEL],
        bridges: &[Bridge {
            span: BRIDGE,
            style: BridgeStyle::RedArches,
            fence: 0.0,
        }],
        gantries: &[
            Gantry {
                along: 0.0,
                color: Color::srgb(0.08, 0.22, 0.75),
                text: Some("TURBO BAY"),
            },
            Gantry {
                along: 960.0,
                color: RED,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1980.0,
                color: RED,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 900.0,
                side: Side::Right,
                text: "MOTORUST",
                color: Color::srgb(0.1, 0.1, 0.12),
                // RUST in the colour of rust, as in the title.
                accent: Some((4, crate::ui::RUST_ORANGE)),
            },
            Billboard {
                along: 930.0,
                side: Side::Right,
                text: "RACING",
                color: Color::srgb(0.1, 0.2, 0.7),
                accent: None,
            },
            Billboard {
                along: 2880.0,
                side: Side::Right,
                text: "BEVY",
                color: Color::srgb(0.1, 0.1, 0.1),
                accent: None,
            },
        ],
        lamps: &[Row {
            span: SEAFRONT,
            side: Side::Left,
            lateral: WALL_DISTANCE + 0.9,
            every: 40.0,
        }],
        trees: &[
            Species::Broadleaf,
            Species::Cypress,
            Species::UmbrellaPine,
            Species::MaritimePine,
            Species::Poplar,
            Species::Almond,
        ],
        forests: &[forest(FOREST, &[Species::Tall])],
        houses: Houses {
            max: 9,
            paints: &[
                Color::srgb(0.95, 0.86, 0.66),
                Color::srgb(0.97, 0.95, 0.9),
                Color::srgb(0.93, 0.75, 0.52),
            ],
            spans: &[TOWN, ROCK_PASS],
            style: HouseStyle::Cottage,
        },
        palms: &[
            Row {
                span: SEAFRONT,
                side: Side::Right,
                lateral: WALL_DISTANCE + 4.0,
                every: 15.0,
            },
            Row {
                span: SEAFRONT,
                side: Side::Left,
                lateral: WALL_DISTANCE + 6.0,
                every: 38.0,
            },
            Row {
                span: span(660.0, 840.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 5.0,
                every: 30.0,
            },
        ],
        hotels: &[Row {
            span: span(3040.0, 230.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 30.0,
            every: 38.0,
        }],
        grand_hotel: Some((760.0, -42.0)),
        buoys: &[
            Vec2::new(-72.0, -60.0),
            Vec2::new(-88.0, -190.0),
            Vec2::new(-70.0, 60.0),
            Vec2::new(-100.0, -290.0),
        ],
        // On the northern cape, below the summit.
        lighthouses: &[Lighthouse {
            position: Vec2::new(480.0, -1046.0),
            height: 24.0,
        }],
        boats: &[
            Boat {
                position: Vec2::new(-85.0, -110.0),
                heading: 0.6,
                kind: BoatKind::Sailboat,
            },
            Boat {
                position: Vec2::new(-115.0, -230.0),
                heading: -0.9,
                kind: BoatKind::Yacht,
            },
            Boat {
                position: Vec2::new(330.0, -90.0),
                heading: 1.4,
                kind: BoatKind::Sailboat,
            },
            Boat {
                position: Vec2::new(430.0, -130.0),
                heading: -2.1,
                kind: BoatKind::Yacht,
            },
            Boat {
                position: Vec2::new(110.0, -560.0),
                heading: 2.4,
                kind: BoatKind::Sailboat,
            },
        ],
        chevrons: &[
            ChevronSign {
                along: 300.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 420.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 860.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            // Up the mountain, and round the summit.
            ChevronSign {
                along: 1270.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 1470.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 1990.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 2810.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
        ],
        grandstands: &[Grandstand {
            along: 70.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.5,
            length: 36.0,
        }],
        beach_parasols: &[
            ParasolCluster {
                position: Vec2::new(-29.0, -140.0),
                count: 6,
            },
            ParasolCluster {
                position: Vec2::new(-28.0, -40.0),
                count: 5,
            },
        ],
        brake_markers: &[
            BrakeMarker {
                along: 200.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 230.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 2690.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 2720.0,
                side: Side::Right,
                distance: 50,
            },
        ],
        soil: Soil::Rock,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Day,
        water: Water::Sea,
        pit: None,
        mine: None,
        desert: None,
        japan: None,
        middle_earth: None,
    },
};

const CONTROL_POINTS: [Vec2; 42] = [
    // 1. The seafront, north (-Z).
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -95.0),
    Vec2::new(0.0, -190.0),
    Vec2::new(0.0, -255.0),
    // 2. The crest, the banked right-hander down to the water and the climb
    //    east to the tunnel.
    Vec2::new(12.0, -318.0),
    Vec2::new(48.0, -362.0),
    Vec2::new(102.0, -380.0),
    Vec2::new(158.0, -365.0),
    // The tunnel, under the road that comes back over the ridge.
    Vec2::new(215.0, -338.0),
    Vec2::new(275.0, -310.0),
    // 3. The grand hotel and the rock pass, turning north.
    Vec2::new(345.0, -285.0),
    Vec2::new(420.0, -280.0),
    Vec2::new(485.0, -300.0),
    Vec2::new(530.0, -345.0),
    Vec2::new(548.0, -410.0),
    // 4. The east coast road.
    Vec2::new(556.0, -490.0),
    // 5. On north up the mountain, round the summit and down to the west.
    Vec2::new(562.0, -575.0),
    Vec2::new(566.0, -655.0),
    Vec2::new(552.0, -728.0),
    Vec2::new(515.0, -782.0),
    Vec2::new(502.0, -850.0),
    Vec2::new(520.0, -915.0),
    Vec2::new(500.0, -975.0),
    Vec2::new(450.0, -1000.0),
    Vec2::new(390.0, -990.0),
    Vec2::new(350.0, -950.0),
    Vec2::new(312.0, -900.0),
    Vec2::new(292.0, -830.0),
    Vec2::new(300.0, -760.0),
    // 6. Down through the forest.
    Vec2::new(318.0, -680.0),
    // Down the ridge and over the tunnel.
    Vec2::new(282.0, -610.0),
    Vec2::new(258.0, -530.0),
    Vec2::new(240.0, -450.0),
    Vec2::new(200.0, -260.0),
    // Down the shore of the bay to the bridge, the town and the seafront.
    Vec2::new(185.0, -180.0),
    Vec2::new(178.0, -95.0),
    Vec2::new(176.0, -10.0),
    Vec2::new(170.0, 60.0),
    Vec2::new(135.0, 108.0),
    Vec2::new(78.0, 122.0),
    Vec2::new(30.0, 100.0),
    Vec2::new(4.0, 50.0),
];

/// Up and down all the way: from the seafront to the water's edge, up
/// through the tunnel and along the coast, up the mountain to the summit,
/// 52 m up, down through the forest, rolling, and down the ridge, about 18 m
/// over the tunnel, to the bridge and the town.
const ELEVATION: [(f32, f32); 47] = [
    (0.0, 0.0),
    (160.0, 0.5),
    (230.0, 3.0),
    // The crest at the end of the seafront: flat out, the bike flies.
    (290.0, 7.0),
    (340.0, 5.0),
    (420.0, 1.5),
    (480.0, 2.5),
    (540.0, 6.5),
    (600.0, 8.5),
    (700.0, 9.5),
    (760.0, 13.5),
    (820.0, 16.0),
    (880.0, 12.5),
    (940.0, 15.0),
    (1000.0, 13.5),
    (1060.0, 17.5),
    // A crest on the coast road, with the sea straight ahead.
    (1120.0, 22.0),
    (1180.0, 18.5),
    (1240.0, 22.0),
    // Up the mountain, with a dip half way.
    (1310.0, 28.0),
    (1380.0, 35.0),
    (1430.0, 36.5),
    (1470.0, 36.0),
    (1530.0, 41.5),
    (1590.0, 47.5),
    // The summit.
    (1650.0, 52.0),
    (1710.0, 50.5),
    // Down through the forest, over a hump, a dip and up again.
    (1770.0, 45.0),
    (1830.0, 46.5),
    (1890.0, 42.0),
    (1950.0, 39.5),
    (2030.0, 42.0),
    (2110.0, 44.0),
    (2190.0, 41.0),
    (2270.0, 33.0),
    (2340.0, 28.0),
    // Over the tunnel.
    (2385.0, 25.5),
    (2450.0, 21.5),
    (2510.0, 20.5),
    (2560.0, 21.0),
    (2620.0, 17.0),
    (2690.0, 13.5),
    (2760.0, 12.0),
    (2830.0, 11.5),
    (2890.0, 9.0),
    (2950.0, 6.0),
    (3010.0, 3.0),
];

/// The island, clockwise from the south-west: the beach off the seafront,
/// the north-west point, the north bay up to a rocky headland, the northern
/// cape round the mountain, the east coast, the hotel headland, the south-east bay with the
/// river mouth that runs in under the bridge, and the south coast.
const COAST: [Vec2; 60] = [
    Vec2::new(-40.0, 190.0),
    Vec2::new(-50.0, 110.0),
    Vec2::new(-47.0, 30.0),
    Vec2::new(-50.0, -60.0),
    Vec2::new(-46.0, -150.0),
    Vec2::new(-50.0, -235.0),
    Vec2::new(-48.0, -305.0),
    Vec2::new(-22.0, -372.0),
    Vec2::new(20.0, -415.0),
    Vec2::new(80.0, -432.0),
    Vec2::new(140.0, -430.0),
    Vec2::new(178.0, -452.0),
    Vec2::new(196.0, -505.0),
    Vec2::new(204.0, -570.0),
    Vec2::new(218.0, -632.0),
    Vec2::new(226.0, -700.0),
    Vec2::new(214.0, -752.0),
    // The northern cape, round the mountain.
    Vec2::new(232.0, -800.0),
    Vec2::new(236.0, -860.0),
    Vec2::new(252.0, -930.0),
    Vec2::new(285.0, -1000.0),
    Vec2::new(340.0, -1050.0),
    Vec2::new(420.0, -1072.0),
    Vec2::new(500.0, -1060.0),
    Vec2::new(562.0, -1015.0),
    Vec2::new(596.0, -940.0),
    Vec2::new(606.0, -860.0),
    Vec2::new(614.0, -780.0),
    Vec2::new(612.0, -660.0),
    Vec2::new(616.0, -575.0),
    Vec2::new(612.0, -490.0),
    Vec2::new(608.0, -410.0),
    Vec2::new(598.0, -345.0),
    Vec2::new(575.0, -292.0),
    Vec2::new(530.0, -258.0),
    Vec2::new(480.0, -232.0),
    Vec2::new(430.0, -208.0),
    Vec2::new(385.0, -212.0),
    Vec2::new(345.0, -238.0),
    Vec2::new(300.0, -255.0),
    Vec2::new(258.0, -246.0),
    Vec2::new(235.0, -210.0),
    Vec2::new(226.0, -150.0),
    Vec2::new(224.0, -90.0),
    Vec2::new(226.0, -40.0),
    // The river mouth, in under the bridge and up the valley.
    Vec2::new(222.0, 5.0),
    Vec2::new(195.0, 14.0),
    Vec2::new(160.0, 22.0),
    Vec2::new(128.0, 30.0),
    Vec2::new(104.0, 42.0),
    Vec2::new(98.0, 52.0),
    Vec2::new(112.0, 56.0),
    Vec2::new(150.0, 52.0),
    Vec2::new(190.0, 54.0),
    Vec2::new(222.0, 66.0),
    Vec2::new(246.0, 108.0),
    Vec2::new(232.0, 160.0),
    Vec2::new(190.0, 178.0),
    Vec2::new(130.0, 170.0),
    Vec2::new(70.0, 178.0),
];

const MASSIFS: [Massif; 13] = [
    // The ridge at the waist, which the tunnel goes through and the road
    // comes back over.
    massif(262.0, -372.0, 55.0, 13.0),
    massif(170.0, -300.0, 50.0, 11.0),
    // A hill inside the first loop, behind the hotels of the seafront.
    massif(95.0, -210.0, 70.0, 24.0),
    // The rocks of the pass and the hills inside the second loop.
    massif(560.0, -330.0, 40.0, 18.0),
    massif(420.0, -520.0, 100.0, 34.0),
    massif(350.0, -610.0, 60.0, 24.0),
    // A crag inside the mountain road, the lighthouse cape, a headland on
    // the north bay, and rocks above the river and the town.
    massif(420.0, -880.0, 70.0, 22.0),
    massif(482.0, -1040.0, 30.0, 10.0),
    massif(250.0, -760.0, 35.0, 14.0),
    massif(50.0, 55.0, 32.0, 12.0),
    // Islets.
    massif(-140.0, -120.0, 26.0, 11.0),
    massif(680.0, -430.0, 30.0, 14.0),
    massif(340.0, -20.0, 28.0, 12.0),
];

const CUTTINGS: [Cutting; 10] = [
    // Rock on both sides before the mouths of the tunnel, below the road
    // that passes over it.
    Cutting {
        span: span(480.0, 650.0),
        side: Side::Left,
        height: 11.0,
    },
    Cutting {
        span: span(480.0, 650.0),
        side: Side::Right,
        height: 11.0,
    },
    // The pass between rocks.
    Cutting {
        span: ROCK_PASS,
        side: Side::Left,
        height: 12.0,
    },
    Cutting {
        span: ROCK_PASS,
        side: Side::Right,
        height: 9.0,
    },
    // Rock walls on the inside of the mountain road, on the way up and on
    // the way down.
    Cutting {
        span: span(1290.0, 1400.0),
        side: Side::Left,
        height: 14.0,
    },
    Cutting {
        span: span(1520.0, 1640.0),
        side: Side::Left,
        height: 16.0,
    },
    Cutting {
        span: span(1800.0, 1880.0),
        side: Side::Left,
        height: 12.0,
    },
    // The ridge on the right, coming down from the forest.
    Cutting {
        span: span(2210.0, 2310.0),
        side: Side::Left,
        height: 12.0,
    },
    // A rock wall on the inside of the bay shore, and on the right after
    // the town.
    Cutting {
        span: span(2590.0, 2690.0),
        side: Side::Right,
        height: 12.0,
    },
    Cutting {
        span: span(2970.0, 3050.0),
        side: Side::Right,
        height: 14.0,
    },
];

/// Across the sea, a mountainous coast to the east and the north.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(280.0, -300.0);
    let away = point - CENTER;
    let distance = away.length();
    // Only to the east and the north; to the west and the south, open sea.
    let facing = smoothstep(
        -0.2,
        0.5,
        away.normalize_or_zero().dot(Vec2::new(0.6, -0.8)),
    );
    smoothstep(1700.0, 2600.0, distance) * facing * 240.0 * (0.35 + 0.65 * fbm(point / 280.0, 4))
}
