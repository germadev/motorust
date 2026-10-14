//! Tsushima Isle: a lap of an island of feudal Japan, after Ghost of
//! Tsushima, its stretches each in a colour of their own. It crosses the
//! castle hill twice, in a figure of eight: first through it, by a tunnel at
//! the end of the castle street, and at the end of the lap over it, 29 m
//! above, through the castle.
//!
//! 1. The castle street, north, over a crest where the bike takes off,
//!    straight at the castle hill and into the tunnel under it.
//! 2. Out onto the coast, round to the right, and east along it in esses,
//!    past the great torii standing in the sea.
//! 3. The shrine on the water: a boardwalk, narrower than the road, straight
//!    across an inlet, with a crest in the middle where the bike takes off.
//! 4. Up the susuki hills in esses at up to 10 %, south over their rolling
//!    top, two crests where the bike takes off, and down at 10 % to the lake.
//! 5. Round the lake, clockwise, three sides of it, between the walls and
//!    its flowers.
//! 6. North from the lake past the temple, where the road splits three ways:
//!    the lap snakes through its courtyard, a way to the left swings out
//!    early over a hump and one to the right runs straight on and out
//!    through a dip.
//! 7. Up the red ridge in bends at up to 10 %, most of the way through a
//!    narrow corridor of torii, as at Fushimi Inari in Kyoto, to its top,
//!    53 m up, and round to the left onto the castle ridge, west.
//! 8. Down along the castle ridge, where the road splits three ways again,
//!    the other way round, and over the tunnel through the castle hill.
//! 9. Down through the bamboo in esses, and round to the left into the
//!    castle street.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const CASTLE_STREET: Span = span(5740.0, 430.0);
const TUNNEL: Span = span(440.0, 620.0);
const COAST: Span = span(640.0, 1422.0);
const BOARDWALK: Span = span(1422.0, 1622.0);
/// How far from the centre line the fences of the boardwalk stand (m): it
/// is narrower than the asphalt of the road.
const BOARDWALK_WALL: f32 = 6.0;
const LAKE_SHORE: Span = span(2700.0, 3400.0);
const RED_RIDGE: Span = span(3764.0, 4322.0);
/// The corridor of torii up the red ridge, after Fushimi Inari's thousand
/// gates in Kyoto: narrower than the road, the gates standing over it where
/// its walls close in (m from the centre line).
const TORII_CORRIDOR: Span = span(3830.0, 4120.0);
const TORII_WALL: f32 = 6.5;
const CROSSING: Span = span(4672.0, 4880.0);
const BAMBOO: Span = span(4880.0, 5600.0);
/// Where the road splits three ways: by the temple, and on the castle ridge.
const TEMPLE_SPLIT: (f32, f32) = (3414.0, 3764.0);
const CASTLE_SPLIT: (f32, f32) = (4322.0, 4672.0);

/// The island's colours: torii vermilion, and white.
const VERMILION: Color = Color::srgb(0.86, 0.2, 0.09);
const WHITE: Color = Color::srgb(0.95, 0.94, 0.9);

pub const TSUSHIMA: Circuit = Circuit {
    name: "Tsushima Isle",
    name_en: "Tsushima Isle",
    description: "Inspirada en Ghost of Tsushima: un castillo cruzado por\n\
                  debajo y por encima, cuestas, un lago entre flores y dos bifurcaciones",
    description_en: "Inspired by Ghost of Tsushima: a castle crossed under and\n\
                     over, steep hills, a lake among flowers and two splits",
    // "Blade of the Autumn Gale".
    // An island in autumn: the wind in the grass, birds, the sea and a few
    // gulls along the coast, and the last cicadas.
    ambience: Ambience {
        sea: 0.35,
        gulls: 0.25,
        birds: 0.6,
        cicadas: 0.3,
        wind: 0.65,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/autumn_gale.mp3"),
        length: 175.8,
        loudness: -15.9,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Hardly banked where the road splits, as in Gold Mine.
        banking: &[
            Banking {
                from: TEMPLE_SPLIT.0,
                to: TEMPLE_SPLIT.1,
                factor: 0.25,
            },
            Banking {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                factor: 0.25,
            },
        ],
        // The shrine's boardwalk and the corridor of torii are narrower than
        // the road: their walls close in on it.
        narrows: &[
            Narrowing {
                from: BOARDWALK.from,
                to: BOARDWALK.to,
                wall: BOARDWALK_WALL,
            },
            Narrowing {
                from: TORII_CORRIDOR.from,
                to: TORII_CORRIDOR.to,
                wall: TORII_WALL,
            },
        ],
        // The road splits three ways twice: a way to the left of the lap
        // over a hump and one to its right through a dip.
        branches: &[
            // By the temple, to the left, over a hump.
            Branch {
                from: TEMPLE_SPLIT.0,
                to: TEMPLE_SPLIT.1,
                approach: &TEMPLE_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &TEMPLE_LEFT_RISE,
            },
            // By the temple, to the right, through a dip.
            Branch {
                from: TEMPLE_SPLIT.0,
                to: TEMPLE_SPLIT.1,
                approach: &TEMPLE_RIGHT,
                vertical_loop: None,
                exit: &[],
                rise: &TEMPLE_RIGHT_RISE,
            },
            // On the castle ridge, to the left, over a hump.
            Branch {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                approach: &CASTLE_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &CASTLE_LEFT_RISE,
            },
            // On the castle ridge, to the right, through a dip.
            Branch {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                approach: &CASTLE_RIGHT,
                vertical_loop: None,
                exit: &[],
                rise: &CASTLE_RIGHT_RISE,
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::OutlineWithLakes(&SHORE, lakes),
        hills: 14.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [VERMILION, WHITE],
        // Stone walls along the castle street, the coast and the castle
        // ridge, a hedge round the lake, and timber everywhere else.
        walls: &[
            WallRun {
                span: CASTLE_STREET,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CASTLE_STREET,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: COAST,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: COAST,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: LAKE_SHORE,
                side: Side::Left,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: LAKE_SHORE,
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: span(CASTLE_SPLIT.0, CROSSING.to),
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: span(CASTLE_SPLIT.0, CROSSING.to),
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
        tunnels: &[TUNNEL],
        // The shrine's boardwalk, planks on piers across the inlet.
        bridges: &[Bridge {
            span: BOARDWALK,
            style: BridgeStyle::Wooden,
        }],
        gantries: &[
            Gantry {
                along: 0.0,
                color: VERMILION,
                text: Some("TSUSHIMA"),
            },
            // The checkpoints, about a kilometre apart (none on the
            // boardwalk nor where the road splits): along the coast, up the
            // susuki hills, by the lake, at the top of the red ridge and
            // down through the bamboo.
            Gantry {
                along: 1000.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2200.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3150.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4150.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 5100.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[Billboard {
            along: 250.0,
            side: Side::Right,
            text: "MOTORUST",
            color: Color::srgb(0.1, 0.1, 0.12),
            accent: Some((4, crate::ui::RUST_ORANGE)),
        }],
        lamps: &[],
        // Cherry (the almond's pink blossom), pines, maples (the broadleaf)
        // and cedars (the tall trees) until the island has trees of its own.
        trees: &[
            Species::Almond,
            Species::MaritimePine,
            Species::Broadleaf,
            Species::Tall,
        ],
        // Dense forest up the red ridge, which the corridor of torii climbs
        // through, as on Mount Inari, and the bamboo, on both sides of the
        // road down from the castle.
        forests: &[RED_RIDGE, BAMBOO],
        // The castle town and a fishing village on the coast: white plaster
        // under tiled roofs.
        houses: Houses {
            max: 12,
            paints: &[
                Color::srgb(0.94, 0.92, 0.86),
                Color::srgb(0.88, 0.84, 0.76),
                Color::srgb(0.55, 0.42, 0.32),
            ],
            spans: &[CASTLE_STREET, span(700.0, 1000.0)],
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
            along: 60.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.0,
            length: 30.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Rock,
        // Yellow flowers all round the lake, between the road and the water
        // (on the right, the inside of the bend).
        sunflowers: &[
            Row {
                span: span(2830.0, 3350.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 6.0,
                every: 7.0,
            },
            Row {
                span: span(2840.0, 3340.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 13.0,
                every: 9.0,
            },
        ],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Day,
        water: Water::Sea,
        pit: None,
        desert: None,
        mine: None,
    },
};

const CONTROL_POINTS: [Vec2; 81] = [
    // 1. The castle street, north (-Z), and the tunnel under the castle hill.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -140.0),
    Vec2::new(0.0, -280.0),
    Vec2::new(0.0, -420.0),
    Vec2::new(0.0, -520.0),
    Vec2::new(0.0, -620.0),
    // 2. Out of the tunnel, round to the right onto the coast, and east
    // along it in esses.
    Vec2::new(8.0, -700.0),
    Vec2::new(45.0, -760.0),
    Vec2::new(110.0, -795.0),
    Vec2::new(185.0, -800.0),
    Vec2::new(255.0, -778.0),
    Vec2::new(325.0, -790.0),
    Vec2::new(390.0, -828.0),
    Vec2::new(460.0, -842.0),
    Vec2::new(530.0, -818.0),
    Vec2::new(590.0, -800.0),
    // 3. The shrine's boardwalk, straight across the inlet.
    Vec2::new(660.0, -800.0),
    Vec2::new(760.0, -800.0),
    Vec2::new(860.0, -800.0),
    // 4. Round to the right, up the susuki hills in esses, and south over
    // their rolling top.
    Vec2::new(935.0, -790.0),
    Vec2::new(990.0, -750.0),
    Vec2::new(1015.0, -685.0),
    Vec2::new(1005.0, -615.0),
    Vec2::new(975.0, -550.0),
    Vec2::new(975.0, -480.0),
    Vec2::new(1010.0, -415.0),
    Vec2::new(1040.0, -345.0),
    Vec2::new(1045.0, -260.0),
    Vec2::new(1045.0, -170.0),
    Vec2::new(1045.0, -80.0),
    Vec2::new(1045.0, 10.0),
    Vec2::new(1045.0, 95.0),
    // 5. Round the lake, clockwise: its east, south and west shores, 175 m
    // from its middle.
    Vec2::new(1045.0, 270.0),
    Vec2::new(1021.6, 357.5),
    Vec2::new(957.5, 421.6),
    Vec2::new(870.0, 445.0),
    Vec2::new(782.5, 421.6),
    Vec2::new(718.4, 357.5),
    Vec2::new(695.0, 270.0),
    // 6. North past the temple: the lap snakes between the ways of its
    // split, as through the second split of Gold Mine (where the lap climbs
    // as it does here).
    Vec2::new(695.0, 220.0),
    Vec2::new(685.9, 178.8),
    Vec2::new(671.1, 137.5),
    Vec2::new(672.9, 96.2),
    Vec2::new(695.0, 55.0),
    Vec2::new(717.1, 13.8),
    Vec2::new(718.9, -27.5),
    Vec2::new(704.1, -68.8),
    Vec2::new(695.0, -110.0),
    Vec2::new(695.0, -160.0),
    // 7. Up the red ridge in bends, and round to the left at its top.
    Vec2::new(707.0, -230.0),
    Vec2::new(705.0, -300.0),
    Vec2::new(675.0, -360.0),
    Vec2::new(655.0, -420.0),
    Vec2::new(630.0, -475.0),
    Vec2::new(585.0, -512.0),
    Vec2::new(525.0, -525.0),
    // 8. West along the castle ridge, through its split (the first split
    // of Gold Mine, where the lap comes down as it does here), and over the
    // tunnel.
    Vec2::new(470.0, -525.0),
    Vec2::new(428.8, -534.1),
    Vec2::new(387.5, -548.9),
    Vec2::new(346.2, -547.1),
    Vec2::new(305.0, -525.0),
    Vec2::new(263.8, -502.9),
    Vec2::new(222.5, -501.1),
    Vec2::new(181.2, -515.9),
    Vec2::new(140.0, -525.0),
    Vec2::new(90.0, -525.0),
    Vec2::new(-40.0, -523.0),
    // 9. Down through the bamboo in esses, and round to the left into the
    // castle street.
    Vec2::new(-100.0, -505.0),
    Vec2::new(-150.0, -470.0),
    Vec2::new(-185.0, -420.0),
    Vec2::new(-185.0, -355.0),
    Vec2::new(-150.0, -300.0),
    Vec2::new(-128.0, -235.0),
    Vec2::new(-140.0, -165.0),
    Vec2::new(-178.0, -100.0),
    Vec2::new(-196.0, -30.0),
    Vec2::new(-188.0, 45.0),
    Vec2::new(-155.0, 112.0),
    Vec2::new(-98.0, 150.0),
    Vec2::new(-42.0, 138.0),
    Vec2::new(-9.0, 82.0),
];

/// Through the tunnel near the sea; rolling along the coast; up the susuki
/// hills, 36 m, and down to the lake; up past the temple and the red ridge,
/// 45 m, and down along the castle ridge, over the tunnel, 29 m above it,
/// and through the bamboo into the castle street.
const ELEVATION: [(f32, f32); 60] = [
    // The castle street, over a crest where the bike flies, and the tunnel.
    (0.0, 0.0),
    (110.0, 0.4),
    (185.0, 4.6),
    (225.0, 3.0),
    (300.0, 0.8),
    (400.0, 0.3),
    (525.0, 0.5),
    (640.0, 1.0),
    // Rolling along the coast.
    (720.0, 2.0),
    (840.0, 5.5),
    (960.0, 8.0),
    (1080.0, 7.0),
    (1200.0, 8.5),
    (1320.0, 5.5),
    (1400.0, 3.2),
    // The boardwalk, with a crest in the middle where the bike flies.
    (1470.0, 3.0),
    (1540.0, 7.2),
    (1585.0, 5.8),
    (1650.0, 3.6),
    // Up the susuki hills, at up to 9 %.
    (1760.0, 6.0),
    (1880.0, 15.0),
    (2000.0, 25.5),
    (2120.0, 32.0),
    (2200.0, 33.6),
    // Over their top, two crests where the bike flies, shaped like the one
    // in the castle street.
    (2270.0, 34.0),
    (2345.0, 38.2),
    (2385.0, 36.6),
    (2460.0, 34.2),
    (2535.0, 38.4),
    (2575.0, 36.8),
    (2620.0, 35.0),
    // Down to the lake, at up to 10 %, and round it.
    (2700.0, 27.5),
    (2780.0, 19.5),
    (2860.0, 12.5),
    (2940.0, 8.0),
    (3020.0, 6.5),
    (3200.0, 6.0),
    (3364.0, 6.5),
    // Climbing steadily through the temple's split.
    (3414.0, 7.0),
    (3764.0, 19.0),
    // Up the red ridge, at up to 10 %, and over its top in the bend onto
    // the castle ridge.
    (3814.0, 21.0),
    (3900.0, 28.5),
    (4000.0, 38.0),
    (4090.0, 46.0),
    (4170.0, 50.2),
    (4230.0, 51.2),
    (4290.0, 49.6),
    // Down steadily through the castle's split (steeper before it and after
    // it, so that the road splits where its grade eases), and over the
    // tunnel.
    (4322.0, 48.2),
    (4672.0, 36.2),
    (4722.0, 34.0),
    (4780.0, 31.3),
    (4852.0, 28.0),
    // Down through the bamboo into the castle street.
    (4950.0, 23.0),
    (5040.0, 18.5),
    (5170.0, 13.5),
    (5310.0, 9.5),
    (5460.0, 5.5),
    (5610.0, 2.8),
    (5738.0, 1.0),
    (5810.0, 0.3),
];

/// The ways off the lap, each through points to one side of it (m along the
/// lap and from its centre line in the comments), from where it leaves the
/// lap to where it joins it again: those of Gold Mine's second split,
/// turned north. By the temple, to the left: it swings out to the left
/// before the lap does, and back to it late.
const TEMPLE_LEFT: [Vec2; 11] = [
    // 3426 m, 0 m.
    Vec2::new(693.2, 210.1),
    // 3456 m, 4 m.
    Vec2::new(682.4, 181.5),
    // 3494 m, 13 m.
    Vec2::new(659.8, 148.6),
    // 3522 m, 32 m.
    Vec2::new(637.9, 114.4),
    // 3544 m, 48 m.
    Vec2::new(626.7, 81.4),
    // 3562 m, 60 m.
    Vec2::new(629.3, 48.4),
    // 3598 m, 65 m.
    Vec2::new(643.3, 15.4),
    // 3676 m, 57 m.
    Vec2::new(662.7, -17.5),
    // 3710 m, 29 m.
    Vec2::new(682.0, -45.2),
    // 3728 m, 11 m.
    Vec2::new(692.0, -72.2),
    // 3752 m, 2 m.
    Vec2::new(695.5, -96.6),
];

/// By the temple, to the right: it runs straight on where the lap swings
/// left, and out round a wide curve back to it.
const TEMPLE_RIGHT: [Vec2; 11] = [
    // 3426 m, -1 m.
    Vec2::new(695.0, 209.7),
    // 3454 m, -15 m.
    Vec2::new(701.2, 178.1),
    // 3476 m, -40 m.
    Vec2::new(715.9, 145.2),
    // 3538 m, -65 m.
    Vec2::new(735.8, 114.4),
    // 3598 m, -64 m.
    Vec2::new(754.2, 81.4),
    // 3626 m, -57 m.
    Vec2::new(764.4, 48.4),
    // 3644 m, -44 m.
    Vec2::new(761.4, 15.4),
    // 3666 m, -25 m.
    Vec2::new(745.0, -17.6),
    // 3698 m, -9 m.
    Vec2::new(721.3, -50.8),
    // 3728 m, -3 m.
    Vec2::new(705.3, -75.0),
    // 3752 m, -0 m.
    Vec2::new(697.4, -96.9),
];

/// On the castle ridge, as in Gold Mine's first split, to the left: it runs
/// straight on where the lap swings right, and out round a wide curve back
/// to it.
const CASTLE_LEFT: [Vec2; 11] = [
    // 4334 m, 1 m.
    Vec2::new(458.3, -525.2),
    // 4362 m, 15 m.
    Vec2::new(427.1, -519.1),
    // 4384 m, 40 m.
    Vec2::new(395.1, -504.2),
    // 4446 m, 65 m.
    Vec2::new(364.4, -484.2),
    // 4506 m, 64 m.
    Vec2::new(331.4, -465.8),
    // 4534 m, 57 m.
    Vec2::new(298.4, -455.6),
    // 4550 m, 44 m.
    Vec2::new(265.4, -458.6),
    // 4574 m, 25 m.
    Vec2::new(232.4, -475.0),
    // 4608 m, 9 m.
    Vec2::new(198.7, -498.9),
    // 4636 m, 3 m.
    Vec2::new(173.9, -515.0),
    // 4658 m, 0 m.
    Vec2::new(153.6, -522.5),
];

/// On the castle ridge, to the right: it swings out to the right before the
/// lap does, and back to it late.
const CASTLE_RIGHT: [Vec2; 11] = [
    // 4334 m, -0 m.
    Vec2::new(458.7, -527.0),
    // 4364 m, -4 m.
    Vec2::new(430.6, -537.9),
    // 4402 m, -14 m.
    Vec2::new(398.3, -560.3),
    // 4430 m, -32 m.
    Vec2::new(364.4, -582.1),
    // 4452 m, -48 m.
    Vec2::new(331.4, -593.3),
    // 4470 m, -60 m.
    Vec2::new(298.4, -590.7),
    // 4504 m, -65 m.
    Vec2::new(265.4, -576.7),
    // 4584 m, -57 m.
    Vec2::new(232.5, -557.3),
    // 4618 m, -29 m.
    Vec2::new(204.3, -538.2),
    // 4638 m, -11 m.
    Vec2::new(176.7, -528.3),
    // 4658 m, -2 m.
    Vec2::new(153.9, -524.4),
];

/// How much higher than the lap beside it the way to the left of the temple
/// runs: over a hump where it is far from the lap, broad at its top (m along
/// it, m).
const TEMPLE_LEFT_RISE: [(f32, f32); 35] = [
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

/// How much higher than the lap beside it the way to the right of the temple
/// runs: through a dip where it is far from the lap, gentle at its edges (m
/// along it, m).
const TEMPLE_RIGHT_RISE: [(f32, f32); 33] = [
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

/// How much higher than the lap beside it the way to the left on the castle
/// ridge runs: over a hump where it is far from the lap (m along it, m).
const CASTLE_LEFT_RISE: [(f32, f32); 35] = [
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

/// How much higher than the lap beside it the way to the right on the castle
/// ridge runs: through a dip where it is far from the lap (m along it, m).
const CASTLE_RIGHT_RISE: [(f32, f32); 33] = [
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

/// The island, clockwise from the north-west: the north coast along the
/// coast road, the inlet the boardwalk crosses, the east coast beyond the
/// susuki hills, the south coast round the lake and the castle town, and
/// the west coast beyond the bamboo.
const SHORE: [Vec2; 49] = [
    Vec2::new(-280.0, -660.0),
    Vec2::new(-190.0, -765.0),
    Vec2::new(-80.0, -850.0),
    Vec2::new(20.0, -870.0),
    Vec2::new(110.0, -862.0),
    Vec2::new(250.0, -848.0),
    Vec2::new(390.0, -880.0),
    Vec2::new(470.0, -898.0),
    Vec2::new(560.0, -874.0),
    Vec2::new(630.0, -856.0),
    // The inlet, in under the boardwalk and a little way past it.
    Vec2::new(664.0, -818.0),
    Vec2::new(680.0, -775.0),
    Vec2::new(702.0, -738.0),
    Vec2::new(760.0, -722.0),
    Vec2::new(818.0, -738.0),
    Vec2::new(840.0, -775.0),
    Vec2::new(856.0, -818.0),
    Vec2::new(900.0, -858.0),
    Vec2::new(980.0, -878.0),
    Vec2::new(1070.0, -858.0),
    Vec2::new(1135.0, -800.0),
    Vec2::new(1172.0, -700.0),
    Vec2::new(1195.0, -585.0),
    Vec2::new(1208.0, -450.0),
    Vec2::new(1205.0, -300.0),
    Vec2::new(1212.0, -150.0),
    Vec2::new(1216.0, 0.0),
    Vec2::new(1212.0, 150.0),
    Vec2::new(1200.0, 300.0),
    Vec2::new(1170.0, 435.0),
    Vec2::new(1110.0, 545.0),
    Vec2::new(1010.0, 612.0),
    Vec2::new(880.0, 640.0),
    Vec2::new(750.0, 622.0),
    Vec2::new(640.0, 562.0),
    Vec2::new(540.0, 470.0),
    Vec2::new(410.0, 370.0),
    Vec2::new(260.0, 325.0),
    Vec2::new(110.0, 312.0),
    Vec2::new(-40.0, 302.0),
    Vec2::new(-170.0, 282.0),
    Vec2::new(-275.0, 215.0),
    Vec2::new(-330.0, 110.0),
    Vec2::new(-350.0, -40.0),
    Vec2::new(-352.0, -220.0),
    Vec2::new(-345.0, -400.0),
    Vec2::new(-325.0, -550.0),
    Vec2::new(-310.0, -610.0),
    Vec2::new(-298.0, -635.0),
];

/// The lake the road goes round: its middle, the radius of its shore (m)
/// and how far the shore wanders (m); and the islet in it, where a tree of
/// its own will stand.
const LAKE: Vec2 = Vec2::new(870.0, 270.0);
const LAKE_RADIUS: f32 = 100.0;
const LAKE_WANDER: f32 = 8.0;
const ISLET: Vec2 = Vec2::new(885.0, 255.0);
const ISLET_RADIUS: f32 = 16.0;

/// How far `point` is from the lake's shore: negative in its water, and
/// positive on the islet.
fn lakes(point: Vec2) -> f32 {
    let wander = LAKE_WANDER * (2.0 * fbm(point / 50.0 + 11.0, 2) - 1.0);
    let lake = point.distance(LAKE) - LAKE_RADIUS + wander;
    let islet = ISLET_RADIUS - point.distance(ISLET);
    lake.max(islet)
}

/// The castle hill the castle street runs into, by a tunnel, and the lap
/// comes back over: its middle, its radius and its height (m).
const CASTLE_HILL: Massif = climbed_massif(0.0, -530.0, 210.0, 46.0);

const MASSIFS: [Massif; 10] = [
    CASTLE_HILL,
    // The castle ridge, east of it, and the red ridge, east of that, which
    // the road climbs from the temple.
    climbed_massif(300.0, -530.0, 230.0, 50.0),
    climbed_massif(600.0, -460.0, 260.0, 58.0),
    // The susuki hills.
    climbed_massif(1040.0, -330.0, 300.0, 40.0),
    climbed_massif(1050.0, 20.0, 240.0, 38.0),
    // The slopes the bamboo grows on, west of the castle.
    climbed_massif(-130.0, -420.0, 200.0, 32.0),
    // Crags on the red ridge, away from the road.
    massif(520.0, -330.0, 40.0, 22.0),
    massif(800.0, -420.0, 45.0, 24.0),
    // Stacks of rock standing in the sea off the coast road.
    massif(300.0, -950.0, 28.0, 16.0),
    massif(560.0, -990.0, 24.0, 13.0),
];

const CUTTINGS: [Cutting; 2] = [
    // The rock round the mouths of the tunnel, under the castle.
    Cutting {
        span: span(TUNNEL.from - 60.0, TUNNEL.to + 60.0),
        side: Side::Left,
        height: 16.0,
    },
    Cutting {
        span: span(TUNNEL.from - 60.0, TUNNEL.to + 60.0),
        side: Side::Right,
        height: 16.0,
    },
];

/// The volcano across the strait to the south-west: its middle, its radius
/// and its height (m).
const VOLCANO: Vec2 = Vec2::new(-600.0, 1900.0);
const VOLCANO_RADIUS: f32 = 1150.0;
const VOLCANO_HEIGHT: f32 = 620.0;

/// Across the sea: a volcano to the south-west, a broad cone with a crater
/// at its top, and mountains round the rest of the horizon.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(450.0, -150.0);
    let ranges = smoothstep(1900.0, 2800.0, point.distance(CENTER))
        * 220.0
        * (0.35 + 0.65 * fbm(point / 300.0, 4));
    let reach = point.distance(VOLCANO) / VOLCANO_RADIUS;
    let cone = VOLCANO_HEIGHT * (1.0 - reach).max(0.0).powf(1.6);
    let crater = 45.0 * (1.0 - smoothstep(0.0, 0.07, reach));
    ranges.max(cone - crater)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    /// From the end of the boardwalk over the susuki hills to the lake.
    const SUSUKI_HILLS: Span = span(1622.0, 2700.0);

    #[test]
    fn the_lap_goes_through_the_castle_hill_and_then_over_it() {
        let track = Track::new(&TSUSHIMA.layout);
        let line = track.centerline();
        // Through it, by its middle.
        let middle = line[track.index_at((TUNNEL.from + TUNNEL.to) / 2.0)];
        assert!(middle.distance(CASTLE_HILL.center) < 0.2 * CASTLE_HILL.radius);
        // Over it at the end of the lap, well above the tunnel.
        let under = track.index_at(TUNNEL.from)..track.index_at(TUNNEL.to);
        let (gap, over, under) = (track.index_at(CROSSING.from)..track.index_at(CROSSING.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the crossing and the tunnel have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 25.0, "{above} m above");
    }

    #[test]
    fn the_hills_are_steep_and_the_road_climbs_and_drops_a_lot() {
        let track = Track::new(&TSUSHIMA.layout);
        let step = track.step_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let grade = |along: f32| {
            let index = track.index_at(along);
            (track.elevations()[index + 1] - track.elevations()[index]) / step
        };
        let steepest = |stretch: Span| {
            (track.index_at(stretch.from)..track.index_at(stretch.to))
                .map(|index| grade(index as f32 * step).abs())
                .fold(0.0, f32::max)
        };
        // Up the susuki hills, and down them to the lake, at nearly 10 %.
        assert!(height(2300.0) - height(SUSUKI_HILLS.from) > 30.0);
        assert!(steepest(span(1700.0, 2300.0)) > 0.085);
        assert!(steepest(span(2600.0, 2900.0)) > 0.085);
        // Up the red ridge to its top, 30 m and more, and down from the
        // castle to town.
        let top = (track.index_at(RED_RIDGE.from)..track.index_at(RED_RIDGE.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(top - height(RED_RIDGE.from) > 30.0, "{top} m at the top");
        assert!(steepest(RED_RIDGE) > 0.085);
        assert!(height(CROSSING.from) - height(0.0) > 35.0);
    }

    #[test]
    fn the_road_rounds_a_lake_with_an_islet_and_crosses_an_inlet_on_the_boardwalk() {
        let track = Track::new(&TSUSHIMA.layout);
        let lap = track.lap_length();
        // Three sides of the lake, about 75 m from its shore all the way,
        // with the lake on the right.
        let mut round = 0.0;
        for index in 0..track.sample_count() {
            let point = track.centerline()[index];
            let shore = lakes(point);
            assert!(shore > 40.0, "{shore} m from the lake at {index}");
            if LAKE_SHORE.contains(index as f32 * track.step_length(), lap) && shore < 85.0 {
                round += track.step_length();
                let lake_side = (LAKE - point).dot(track.offset_point(index, 1.0) - point);
                assert!(lake_side < 0.0, "the lake on the left at {index}");
            }
        }
        assert!(round > 450.0, "{round} m round the lake");
        assert!(lakes(LAKE + Vec2::new(-40.0, 30.0)) < -20.0, "deep water");
        assert!(lakes(ISLET) > 10.0, "an islet");
        // The boardwalk crosses the inlet; the water reaches in under it.
        let middle = track.centerline()[track.index_at((BOARDWALK.from + BOARDWALK.to) / 2.0)];
        let shore = crate::track::closed_curve(&SHORE, 4.0);
        let inside = |point: Vec2| {
            let n = shore.len();
            (0..n).fold(false, |inside, i| {
                let (a, b) = (shore[i], shore[(i + 1) % n]);
                let crosses = (a.y > point.y) != (b.y > point.y)
                    && point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
                inside != crosses
            })
        };
        assert!(!inside(middle), "the inlet is under the boardwalk");
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.from - 30.0)]
        ));
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.to + 30.0)]
        ));
    }

    #[test]
    fn the_volcano_rises_across_the_strait_and_nothing_is_raised_by_the_road() {
        let track = Track::new(&TSUSHIMA.layout);
        assert!(mountains(VOLCANO) > 0.85 * VOLCANO_HEIGHT);
        assert!(mountains(VOLCANO) < VOLCANO_HEIGHT, "a crater at its top");
        for &point in track.centerline() {
            assert!(mountains(point) < 1e-3, "{point}");
        }
    }
}
