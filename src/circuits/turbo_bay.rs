//! Turbo Bay, inspired by Speed Bay in Moto Racer: a figure of eight on a
//! hilly island, with the sea all around and two bays cutting in towards
//! its waist. The first loop goes clockwise and the second anticlockwise,
//! and halfway round the road climbs back over the ridge it first went
//! through in a tunnel.
//!
//! 1. The start straight is a seafront promenade heading north: railing,
//!    beach and sea on the left; palms and tall, colourful hotels on the
//!    right.
//! 2. On level into a steep banked right-hander at the water's edge, and
//!    the road climbs east into a tunnel through the ridge at the island's
//!    waist.
//! 3. Out of the tunnel, past an old-style grand hotel on a headland above
//!    the south-east bay, to a pass between rocks.
//! 4. The east coast road rises and falls behind a white barrier, with the
//!    sea on the right.
//! 5. A mountain road climbs on north in S-bends between rock walls, 37 m
//!    up, over a broad top with the lighthouse straight ahead below, and
//!    dives steeply at it to the foot of its tower on the northern cape,
//!    round a long left-hander.
//! 6. Back up through a forest it climbs, eases off over a broad shoulder and
//!    rises again, then
//!    runs down the ridge and crosses over the tunnel, down the shore of the
//!    bay to a bridge with red arches over the river mouth.
//! 7. Up a gorge to an arch dam; away from it up the hillside to a hairpin
//!    over the east coast, and back down straight onto its crest, narrow,
//!    the lake mirroring the hills on the left over a low parapet; down
//!    through the town and back to the seafront.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// The height of the lake behind the dam (m over the sea), a couple of
/// metres under the road along its crest, so it shows over the parapet.
const LAKE_LEVEL: f32 = 41.0;
/// The square with the monument, by the far end of the crest, on the
/// lake's side, as at El Atazar.
const SQUARE: Vec2 = Vec2::new(76.0, 462.0);

/// Where things are, in metres from the finish line.
const SEAFRONT: Span = span(4304.0, 250.0);
const TUNNEL: Span = span(505.0, 625.0);
const ROCK_PASS: Span = span(870.0, 990.0);
const EAST_COAST: Span = span(1000.0, 1280.0);
const MOUNTAIN: Span = span(1280.0, 1660.0);
/// Down at the foot of the lighthouse, at the bottom of the dive.
const CAPE: Span = span(1560.0, 1700.0);
const FOREST: Span = span(1660.0, 2160.0);
const BAY_SHORE: Span = span(2460.0, 2710.0);
const BRIDGE: Span = span(2720.0, 2805.0);
const GORGE: Span = span(2830.0, 3100.0);
/// Down the hillside straight at the dam, the crest and the lake below.
const DESCENT: Span = span(3480.0, 3706.0);
const CREST: Span = span(3706.0, 3858.0);
/// The road over the crest is narrow, as a dam's: its walls this far from
/// the middle (m), closing in on the way down to it.
const CREST_WALL: f32 = 5.5;
const TOWN: Span = span(3984.0, 4294.0);

/// The lighthouse, on the northern cape.
const LIGHTHOUSE: Vec2 = Vec2::new(480.0, -1046.0);

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
    pace: 1.0,
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // The drop to the water after the seafront, banked like a velodrome;
        // and the crest of the dam, level across like a dam's, so its low
        // parapet on the lake's side does not stand up into the view.
        wall: WALL_DISTANCE,
        banking: &[
            Banking {
                from: 330.0,
                to: 470.0,
                factor: 2.5,
                raised: false,
                grip: 1.0,
            },
            Banking {
                from: CREST.from,
                to: CREST.to,
                factor: 0.0,
                raised: false,
                grip: 1.0,
            },
        ],
        narrows: &[Narrowing {
            from: CREST.from,
            to: CREST.to,
            wall: CREST_WALL,
        }],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::Outline(&COAST),
        hills: 22.0,
        // Not the arcade lawn: the many greens of real land in a
        // Mediterranean summer, straw, olive, sage and scrub, less vivid
        // than painted.
        greens: Greens::Mediterranean,
        // Those greens are as dark as they should be already.
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
            // The square at the far end of the dam's crest, level with it.
            Terrace {
                center: SQUARE,
                radius: 16.0,
                height: 43.4,
                tilt: Vec2::ZERO,
                skirt: 14.0,
            },
            // The lighthouse's cape: low, level and bare, a little above
            // the road at the bottom of the dive, so the tower is seen
            // whole from it.
            Terrace {
                center: LIGHTHOUSE,
                radius: 12.0,
                height: 23.0,
                tilt: Vec2::ZERO,
                skirt: 30.0,
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
            // The mountain road, with the sea on the right, and a railing
            // round the cape, so nothing hides the lighthouse.
            WallRun {
                span: span(MOUNTAIN.from, CAPE.from),
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: CAPE,
                side: Side::Right,
                style: WallStyle::Railing,
            },
            WallRun {
                span: BAY_SHORE,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            // Up the gorge, a barrier over the drop to the river.
            WallRun {
                span: GORGE,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
            // A railing on the way down to the dam, so the lake shows below
            // over it.
            WallRun {
                span: DESCENT,
                side: Side::Left,
                style: WallStyle::Railing,
            },
        ],
        tunnels: &[TUNNEL],
        bridges: &[
            Bridge {
                span: BRIDGE,
                style: BridgeStyle::RedArches,
                fence: 0.0,
            },
            Bridge {
                span: CREST,
                style: BridgeStyle::Dam,
                fence: 0.0,
            },
        ],
        dams: &[Dam {
            crest: CREST,
            lake_side: Side::Left,
            lake: &LAKE,
            level: LAKE_LEVEL,
            river: &RIVER,
            road_bank: Side::Right,
            monument: Some(SQUARE),
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
            Gantry {
                along: 2980.0,
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
                along: 4084.0,
                side: Side::Right,
                text: "BEVY",
                color: Color::srgb(0.1, 0.1, 0.1),
                accent: None,
            },
        ],
        lamps: &[
            Row {
                span: SEAFRONT,
                side: Side::Left,
                lateral: WALL_DISTANCE + 0.9,
                every: 40.0,
            },
            // Along the crest of the dam, on its parapets.
            Row {
                span: CREST,
                side: Side::Left,
                lateral: CREST_WALL + 0.3,
                every: 30.0,
            },
            Row {
                span: span(CREST.from + 15.0, CREST.to),
                side: Side::Right,
                lateral: CREST_WALL + 0.3,
                every: 30.0,
            },
        ],
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
            span: span(4324.0, 230.0),
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
            position: LIGHTHOUSE,
            height: 28.0,
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
            // Round to the left below the dam, round the hairpin, and off
            // the crest down to the town.
            ChevronSign {
                along: 3040.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 3080.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 3325.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 3370.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 3415.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 3904.0,
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
            // Before the turn below the dam, and before the hairpin.
            BrakeMarker {
                along: 2925.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 2955.0,
                side: Side::Right,
                distance: 50,
            },
            BrakeMarker {
                along: 3205.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 3235.0,
                side: Side::Left,
                distance: 50,
            },
        ],
        soil: Soil::Rock,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Mediterranean,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        wheelie_halo: None,
        japan: None,
        middle_earth: None,
        space: None,
        toys: None,
    },
};

const CONTROL_POINTS: [Vec2; 69] = [
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
    // Down the shore of the bay to the bridge over the river mouth.
    Vec2::new(185.0, -180.0),
    Vec2::new(178.0, -95.0),
    Vec2::new(176.0, -10.0),
    Vec2::new(170.0, 60.0),
    // 7. Up the gorge, beside the river, towards the dam.
    Vec2::new(176.0, 135.0),
    Vec2::new(186.0, 210.0),
    Vec2::new(188.0, 250.0),
    Vec2::new(188.0, 288.8),
    // Round to the left below the dam's near end, and up the hillside away
    // from it, east-north-east above the bay.
    Vec2::new(194.3, 311.5),
    Vec2::new(211.7, 327.8),
    Vec2::new(235.1, 332.7),
    Vec2::new(257.2, 324.8),
    Vec2::new(284.7, 305.7),
    Vec2::new(325.6, 277.0),
    Vec2::new(366.6, 248.3),
    Vec2::new(407.5, 219.6),
    // A hairpin to the right, high over the east coast, and back down the
    // hillside straight at the dam, the crest and the lake below ahead.
    Vec2::new(443.3, 211.6),
    Vec2::new(474.3, 231.3),
    Vec2::new(482.3, 267.3),
    Vec2::new(462.6, 298.2),
    Vec2::new(413.5, 332.6),
    Vec2::new(364.4, 367.0),
    Vec2::new(315.2, 401.5),
    // 8. Over the crest of the dam: an arch 125 m in radius, bowed towards
    //    the lake on the left.
    Vec2::new(249.7, 447.4),
    Vec2::new(215.6, 464.2),
    Vec2::new(178.0, 470.0),
    Vec2::new(140.4, 464.2),
    Vec2::new(106.3, 447.4),
    // 9. Down the far side, through the town and back to the seafront.
    Vec2::new(73.0, 424.0),
    Vec2::new(38.0, 390.0),
    Vec2::new(16.0, 340.0),
    Vec2::new(6.0, 280.0),
    Vec2::new(8.0, 195.0),
    Vec2::new(6.0, 115.0),
    Vec2::new(2.0, 50.0),
];

/// Up and down all the way: from the seafront to the water's edge, up
/// through the tunnel and along the coast, up the mountain, 40 m up, down
/// in a dive to the foot of the lighthouse, back up through the forest,
/// rolling, and down the ridge, about 18 m over the tunnel, to the bridge
/// and the town.
const ELEVATION: [(f32, f32); 70] = [
    (0.0, 0.0),
    (160.0, 0.5),
    // On level to the end of the seafront and into the bend at the water's
    // edge, with no crest to unsettle the bike as it brakes for it.
    (300.0, 1.0),
    (420.0, 1.5),
    (480.0, 2.5),
    (540.0, 6.5),
    (600.0, 8.5),
    (700.0, 9.5),
    (760.0, 13.5),
    (820.0, 16.0),
    // Through the rock pass, in a bend, rising steadily with no crest.
    (880.0, 12.5),
    (940.0, 13.0),
    (1000.0, 14.5),
    (1060.0, 17.5),
    // A crest on the coast road, with the sea straight ahead.
    (1120.0, 22.0),
    (1180.0, 18.5),
    (1240.0, 22.0),
    // Up the mountain to its top, with the lighthouse straight ahead below:
    // a broad top (a crest over 600 m round, which the bike only leaves the
    // road over at more than 270 km/h), as it goes over it in a bend.
    (1310.0, 28.0),
    (1380.0, 34.0),
    (1430.0, 35.5),
    // The dive to the cape, steep and straight at the lighthouse.
    (1480.0, 33.5),
    (1530.0, 29.0),
    (1580.0, 23.5),
    // Down at the foot of the lighthouse, round the cape.
    (1630.0, 18.5),
    (1680.0, 19.0),
    // Back up the west side into the forest, easing off over a broad
    // shoulder (no crest to throw the bike off its lean in the bends), and
    // up again.
    (1740.0, 23.5),
    (1800.0, 30.0),
    (1870.0, 34.8),
    (1930.0, 36.3),
    (1990.0, 37.5),
    (2060.0, 41.5),
    (2120.0, 43.5),
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
    // Down into the gorge, and up it beside the river.
    (2830.0, 11.0),
    (2880.0, 8.5),
    (2950.0, 7.0),
    (3016.0, 7.5),
    // Round below the dam's near end, and up the hillside away from it.
    (3065.0, 10.5),
    (3112.0, 14.5),
    (3170.0, 20.5),
    (3235.0, 27.0),
    (3296.0, 33.0),
    // Round the hairpin, high over the coast.
    (3370.0, 40.5),
    (3446.0, 46.5),
    // Over the top and down the hillside onto the dam, the lake below.
    (3506.0, 49.5),
    (3566.0, 49.0),
    (3626.0, 46.5),
    (3664.0, 43.3),
    (3690.0, 41.9),
    // The crest, level, the lake a few metres under it.
    (3706.0, 41.5),
    (3744.0, 41.5),
    (3782.0, 41.5),
    (3820.0, 41.5),
    (3858.0, 41.5),
    // Down the far side to the town and the seafront.
    (3900.0, 40.2),
    (3948.0, 36.0),
    (4003.0, 30.0),
    (4064.0, 23.0),
    (4149.0, 14.0),
    (4229.0, 7.0),
    (4295.0, 2.5),
];

/// The island, clockwise from the south-west: the beach off the seafront,
/// the north-west point, the north bay up to a rocky headland, the northern
/// cape round the mountain, the east coast, the hotel headland, the south-east bay with the
/// river mouth that runs in under the bridge, and the south coast.
const COAST: [Vec2; 72] = [
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
    // The south of the island, round the gorge, the dam and its lake, and
    // up the beach beside the road down to the town.
    Vec2::new(222.0, 66.0),
    Vec2::new(262.0, 82.0),
    Vec2::new(330.0, 112.0),
    Vec2::new(410.0, 160.0),
    Vec2::new(500.0, 240.0),
    Vec2::new(505.0, 350.0),
    Vec2::new(495.0, 480.0),
    Vec2::new(430.0, 570.0),
    Vec2::new(390.0, 665.0),
    Vec2::new(320.0, 735.0),
    Vec2::new(220.0, 770.0),
    Vec2::new(110.0, 765.0),
    Vec2::new(20.0, 730.0),
    Vec2::new(-40.0, 650.0),
    Vec2::new(-62.0, 550.0),
    Vec2::new(-66.0, 450.0),
    Vec2::new(-62.0, 350.0),
    Vec2::new(-55.0, 260.0),
];

const MASSIFS: [Massif; 16] = [
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
    // A crag inside the mountain road, a headland on the north bay, and
    // rocks above the river and the town.
    massif(420.0, -880.0, 70.0, 22.0),
    massif(250.0, -760.0, 35.0, 14.0),
    massif(50.0, 55.0, 32.0, 12.0),
    // The mountains round the lake behind the dam, and the hill the road
    // climbs in a hairpin, away from the dam, before coming down onto it.
    climbed_massif(10.0, 610.0, 170.0, 62.0),
    climbed_massif(200.0, 790.0, 180.0, 70.0),
    climbed_massif(390.0, 610.0, 170.0, 64.0),
    climbed_massif(345.0, 318.0, 90.0, 52.0),
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
    // A rock wall on the inside of the bay shore, and the gorge's wall on
    // the left, across the road from the river.
    Cutting {
        span: span(2590.0, 2690.0),
        side: Side::Right,
        height: 12.0,
    },
    Cutting {
        span: span(GORGE.from + 20.0, GORGE.to - 10.0),
        side: Side::Left,
        height: 30.0,
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

/// The lake behind the dam: along the crest, then out under the hillside
/// the road comes down, in sight from it, and round the mountains to the
/// south.
const LAKE: [Vec2; 17] = [
    Vec2::new(106.3, 447.4),
    Vec2::new(140.4, 464.2),
    Vec2::new(178.0, 470.0),
    Vec2::new(215.6, 464.2),
    Vec2::new(249.7, 447.4),
    Vec2::new(276.0, 462.0),
    Vec2::new(306.0, 482.0),
    Vec2::new(318.0, 520.0),
    Vec2::new(305.0, 570.0),
    Vec2::new(290.0, 625.0),
    Vec2::new(255.0, 660.0),
    Vec2::new(195.0, 672.0),
    Vec2::new(135.0, 660.0),
    Vec2::new(95.0, 625.0),
    Vec2::new(80.0, 575.0),
    Vec2::new(82.0, 525.0),
    Vec2::new(98.0, 486.0),
];

/// The river from under the dam down the gorge to its mouth, under the red
/// bridge, and how high the gorge's far wall stands over it.
const RIVER: [(Vec2, f32); 8] = [
    (Vec2::new(178.0, 470.0), 40.0),
    (Vec2::new(170.0, 430.0), 40.0),
    (Vec2::new(161.0, 390.0), 41.0),
    (Vec2::new(152.0, 330.0), 40.0),
    (Vec2::new(146.0, 250.0), 34.0),
    (Vec2::new(140.0, 170.0), 26.0),
    (Vec2::new(134.0, 100.0), 16.0),
    (Vec2::new(130.0, 45.0), 6.0),
];
