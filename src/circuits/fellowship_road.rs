//! Fellowship Road: the journey of *The Fellowship of the Ring*, its
//! stretches in the order of the film, from the Shire to the Argonath and,
//! past the finish, home again (*There and Back Again*). It crosses the
//! Misty Mountains twice, in a figure of eight: first over them, by the pass
//! of Caradhras, and then, as the Fellowship turned back from the snow,
//! through them, by the mines of Moria, under the pass.
//!
//! 1. Hobbiton: the start straight, north, over a crest, a sweep to the
//!    left and the right, and round to the right at the foot of Bag End's
//!    hill.
//! 2. The Shire: esses east through the fields, a straight over a crest
//!    where the bike takes off, and a sweep to the right and the left over
//!    the bridge on the Brandywine.
//! 3. Bree: through its gate, right and left through the town.
//! 4. Weathertop: up its flank at up to 12 %, round to the left and the
//!    right, over its top on a straight with a crest where the bike takes
//!    off, 53 m up, and down at up to 12 %.
//! 5. The Trollshaws: down in esses to the Ford of Bruinen, low over the
//!    water.
//! 6. Rivendell: south along the valley under its cliffs, where the road
//!    splits in two: the lap snakes along it and a way to the left swings
//!    out over a hump.
//! 7. Caradhras: up in bends at up to 12 % to the pass, 49 m up, right over
//!    the mines.
//! 8. Down round the mountain: on south, round to the left, east, north and
//!    back west, to the Doors of Durin.
//! 9. Moria: west through the mountain, under the pass, 38 m beneath it,
//!    and on in a sweep to the left and the right, down into it and up
//!    again.
//! 10. Lothlórien: where the road splits three ways, the lap snaking
//!     between a way to the left over a hump and one to the right through a
//!     dip.
//! 11. The Anduin: down to the river, along its gorge in esses, and west
//!     down its middle on a causeway, narrower than the road, in a sweep to
//!     the right and the left between the two kings of the Argonath.
//! 12. Amon Hen: past the lake, round to the right over the shoulder of the
//!     hill, and down onto the start straight, back in the Shire.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line, in the order of the
/// film.
const HOBBITON: Span = span(5840.0, 424.0);
const SHIRE: Span = span(380.0, BREE.from);
const BRANDYWINE: Span = span(1080.0, 1140.0);
const BREE: Span = span(1280.0, 1494.0);
const TROLLSHAWS: Span = span(2154.0, FORD.from);
/// The Ford of Bruinen: low over the river.
const FORD: Span = span(2495.0, 2555.0);
const RIVENDELL: Span = span(FORD.to, CARADHRAS.from);
const CARADHRAS: Span = span(3014.0, 3414.0);
/// The top of the pass, right over the mines.
const PASS: Span = span(3350.0, 3414.0);
const MORIA: Span = span(3985.0, 4328.0);
const LOTHLORIEN: Span = span(MORIA.to, GORGE.from);
const GORGE: Span = span(4782.0, CAUSEWAY.from);
/// Down the middle of the Anduin, between the kings of the Argonath: a
/// causeway narrower than the road, its walls this far from the middle
/// (m).
const CAUSEWAY: Span = span(5095.0, 5430.0);
const CAUSEWAY_WALL: f32 = 6.5;
const AMON_HEN: Span = span(CAUSEWAY.to, HOBBITON.from);
/// Where the road splits: in two in Rivendell, three ways in Lothlórien.
const RIVENDELL_SPLIT: (f32, f32) = (2614.0, 2964.0);
const LORIEN_SPLIT: (f32, f32) = (4382.0, 4732.0);

/// The Ring's gold, the Shire's green and cream.
const GOLD: Color = Color::srgb(0.86, 0.68, 0.22);
const GREEN: Color = Color::srgb(0.22, 0.45, 0.2);
const CREAM: Color = Color::srgb(0.93, 0.88, 0.72);

pub const FELLOWSHIP_ROAD: Circuit = Circuit {
    name: "Fellowship Road",
    name_en: "Fellowship Road",
    description: "Inspirada en La Comunidad del Anillo: de la Comarca a los\n\
                  Argonath, un ocho sobre y bajo las montañas, cuestas y dos bifurcaciones",
    description_en: "Inspired by The Fellowship of the Ring: from the Shire to the\n\
                     Argonath, a figure of eight over and under the mountains, steep hills and two splits",
    // "Pursuit at the High Pass".
    // Fields and woods: birds, the wind, and crickets in the grass.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.75,
        cicadas: 0.2,
        wind: 0.6,
    },
    // Measured at -18.4 LUFS, but its bass is heavy: over the whole band its
    // loudest second would stand 0.7 dB too near the engine (every song
    // keeps 10 dB under it). Taken as 1 dB louder, it is turned down 1 dB
    // more than the others.
    song: Song {
        mp3: include_bytes!("../../assets/music/pursuit_at_the_high_pass.mp3"),
        length: 166.7,
        loudness: -17.4,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Hardly banked where the road splits, as at Ghost Island.
        banking: &[
            Banking {
                from: RIVENDELL_SPLIT.0,
                to: RIVENDELL_SPLIT.1,
                factor: 0.25,
            },
            Banking {
                from: LORIEN_SPLIT.0,
                to: LORIEN_SPLIT.1,
                factor: 0.25,
            },
        ],
        // The causeway between the kings is narrower than the road: its
        // walls close in on it.
        narrows: &[Narrowing {
            from: CAUSEWAY.from,
            to: CAUSEWAY.to,
            wall: CAUSEWAY_WALL,
        }],
        // In Rivendell a way to the left of the lap over a hump; in
        // Lothlórien one to the left over a hump and one to the right
        // through a dip.
        branches: &[
            Branch {
                from: RIVENDELL_SPLIT.0,
                to: RIVENDELL_SPLIT.1,
                approach: &RIVENDELL_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &RIVENDELL_LEFT_RISE,
            },
            Branch {
                from: LORIEN_SPLIT.0,
                to: LORIEN_SPLIT.1,
                approach: &LORIEN_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &LORIEN_LEFT_RISE,
            },
            Branch {
                from: LORIEN_SPLIT.0,
                to: LORIEN_SPLIT.1,
                approach: &LORIEN_RIGHT,
                vertical_loop: None,
                exit: &[],
                rise: &LORIEN_RIGHT_RISE,
            },
        ],
    },
    // For now, the scenery of the other tracks: the Meshy models of the
    // Shire, Bree, Rivendell and the rest are still being made.
    scenery: Scenery {
        coast: Coast::Inland(water),
        hills: 14.0,
        greens: Greens::Countryside,
        grass_shade: 0.8,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [GREEN, CREAM],
        // Hedgerows through the Shire, Bree's palisade, and stone walls
        // everywhere else.
        walls: &[
            WallRun {
                span: span(HOBBITON.from, SHIRE.to),
                side: Side::Left,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: span(HOBBITON.from, SHIRE.to),
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: BREE,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: BREE,
                side: Side::Right,
                style: WallStyle::Timber,
            },
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Stone,
            },
        ],
        tunnels: &[MORIA],
        // The bridge over the Brandywine, the ford low over the Bruinen and
        // the causeway down the Anduin, their fences low: the water shows.
        bridges: &[
            Bridge {
                span: BRANDYWINE,
                style: BridgeStyle::Wooden,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: FORD,
                style: BridgeStyle::Wooden,
                fence: 0.75,
            },
            Bridge {
                span: CAUSEWAY,
                style: BridgeStyle::Wooden,
                fence: 0.75,
            },
        ],
        gantries: &[
            Gantry {
                along: 0.0,
                color: GOLD,
                text: Some("FELLOWSHIP ROAD"),
            },
            // The checkpoints, 600 m to a kilometre and a half apart (none
            // where the road splits, on the bridges nor in the mines): out of
            // the Shire's esses, down from Weathertop, up Caradhras, on the
            // way round the mountain to the doors, and past Lothlórien.
            Gantry {
                along: 900.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2100.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3150.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3780.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4760.0,
                color: GREEN,
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
        // Oaks, tall trees and poplars.
        trees: &[Species::Broadleaf, Species::Tall, Species::Poplar],
        // The woods of the Trollshaws, of Caradhras' foothills and of Amon
        // Hen; Rivendell's in autumn; and, in gold until the mallorns are
        // made, Lothlórien's.
        forests: &[
            forest(TROLLSHAWS, &[Species::Tall]),
            forest(RIVENDELL, &[Species::Ginkgo, Species::Momiji]),
            forest(span(CARADHRAS.from, PASS.from), &[Species::Tall]),
            forest(LOTHLORIEN, &[Species::Ginkgo]),
            forest(AMON_HEN, &[Species::Tall]),
        ],
        // Hobbiton and Bree, until the hobbit holes and the Prancing Pony
        // are there.
        houses: Houses {
            max: 10,
            paints: &[
                Color::srgb(0.93, 0.86, 0.66),
                Color::srgb(0.86, 0.74, 0.52),
                Color::srgb(0.95, 0.93, 0.86),
            ],
            spans: &[span(5880.0, 300.0), BREE],
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
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        // Down the cliffs of Rivendell, on the right of the valley.
        waterfalls: &[
            Waterfall {
                along: 2624.0,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2950.0,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2988.0,
                side: Side::Right,
                width: 12.0,
            },
        ],
        sky: SkyStyle::Day,
        water: Water::Sea,
        pit: None,
        desert: None,
        mine: None,
        japan: None,
    },
};

const CONTROL_POINTS: [Vec2; 165] = [
    // 1. Hobbiton: the start straight, north (-Z), a sweep to the left and
    // the right, and round to the right at the foot of Bag End's hill.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -62.5),
    Vec2::new(0.0, -125.0),
    Vec2::new(0.0, -187.5),
    Vec2::new(0.0, -250.0),
    Vec2::new(-1.7, -279.0),
    Vec2::new(-6.7, -307.6),
    Vec2::new(-15.1, -335.5),
    Vec2::new(-23.4, -363.4),
    Vec2::new(-28.5, -392.0),
    Vec2::new(-30.1, -421.0),
    Vec2::new(-27.3, -450.3),
    Vec2::new(-18.7, -478.4),
    Vec2::new(-4.9, -504.4),
    Vec2::new(13.8, -527.1),
    Vec2::new(36.5, -545.7),
    Vec2::new(62.4, -559.6),
    Vec2::new(90.6, -568.1),
    Vec2::new(119.8, -571.0),
    // 2. The Shire: esses east through the fields, a straight over a crest,
    // and a sweep to the right and the left over the Brandywine bridge.
    Vec2::new(149.8, -577.0),
    Vec2::new(178.4, -587.9),
    Vec2::new(207.2, -598.0),
    Vec2::new(237.4, -602.2),
    Vec2::new(267.7, -598.0),
    Vec2::new(296.5, -587.9),
    Vec2::new(325.1, -577.0),
    Vec2::new(355.1, -571.0),
    Vec2::new(370.4, -571.0),
    Vec2::new(445.4, -571.0),
    Vec2::new(520.4, -571.0),
    Vec2::new(555.2, -569.0),
    Vec2::new(589.5, -562.9),
    Vec2::new(623.0, -552.9),
    Vec2::new(656.4, -542.9),
    Vec2::new(690.7, -536.9),
    Vec2::new(725.6, -534.8),
    // 3. Bree: through the gate, right and left through the town.
    Vec2::new(756.3, -529.4),
    Vec2::new(783.4, -513.8),
    Vec2::new(806.4, -494.5),
    Vec2::new(833.5, -478.9),
    Vec2::new(864.2, -473.4),
    Vec2::new(924.2, -473.4),
    // 4. Weathertop: up its flank, round to the left and the right, over
    // its top east and round to the right, down.
    Vec2::new(955.8, -478.1),
    Vec2::new(984.7, -491.5),
    Vec2::new(1008.5, -512.7),
    Vec2::new(1025.2, -539.9),
    Vec2::new(1033.5, -570.7),
    Vec2::new(1032.6, -602.5),
    Vec2::new(1031.7, -634.4),
    Vec2::new(1039.9, -665.2),
    Vec2::new(1056.6, -692.3),
    Vec2::new(1080.5, -713.5),
    Vec2::new(1109.4, -727.0),
    Vec2::new(1140.9, -731.6),
    Vec2::new(1200.9, -731.6),
    Vec2::new(1260.9, -731.6),
    Vec2::new(1320.9, -731.6),
    Vec2::new(1352.5, -727.0),
    Vec2::new(1381.3, -713.5),
    Vec2::new(1405.2, -692.3),
    // 5. The Trollshaws: esses down to the Ford of Bruinen, and south.
    Vec2::new(1431.5, -678.4),
    Vec2::new(1461.2, -674.1),
    Vec2::new(1490.8, -669.7),
    Vec2::new(1517.2, -655.8),
    Vec2::new(1533.0, -630.7),
    Vec2::new(1536.2, -601.0),
    Vec2::new(1533.9, -571.0),
    Vec2::new(1537.1, -541.3),
    Vec2::new(1553.0, -516.2),
    Vec2::new(1570.3, -486.1),
    Vec2::new(1576.4, -451.9),
    Vec2::new(1576.4, -381.9),
    // 6. Rivendell: the lap snakes between the ways of its split, as by
    // Ghost Island's temple, turned south.
    Vec2::new(1576.4, -331.9),
    Vec2::new(1585.5, -290.7),
    Vec2::new(1600.3, -249.4),
    Vec2::new(1598.5, -208.1),
    Vec2::new(1576.4, -166.9),
    Vec2::new(1554.3, -125.7),
    Vec2::new(1552.5, -84.4),
    Vec2::new(1567.3, -43.1),
    Vec2::new(1576.4, -1.9),
    Vec2::new(1576.4, 48.1),
    // 7. Caradhras: up in bends to the top, right over Moria.
    Vec2::new(1571.3, 77.2),
    Vec2::new(1556.5, 102.7),
    Vec2::new(1533.9, 121.7),
    Vec2::new(1511.3, 140.7),
    Vec2::new(1496.5, 166.2),
    Vec2::new(1491.4, 195.3),
    Vec2::new(1496.5, 224.4),
    Vec2::new(1511.3, 250.0),
    Vec2::new(1533.9, 268.9),
    Vec2::new(1557.8, 289.0),
    Vec2::new(1573.5, 316.1),
    Vec2::new(1578.9, 346.9),
    Vec2::new(1578.9, 386.9),
    // 8. Over and down round the mountain: on south, round to the left,
    // east, north and back west, to the Doors of Durin.
    Vec2::new(1578.9, 456.9),
    Vec2::new(1583.2, 485.9),
    Vec2::new(1595.8, 512.4),
    Vec2::new(1615.5, 534.2),
    Vec2::new(1640.6, 549.3),
    Vec2::new(1669.1, 556.4),
    Vec2::new(1698.4, 555.0),
    Vec2::new(1726.0, 545.1),
    Vec2::new(1749.6, 527.6),
    Vec2::new(1767.1, 504.0),
    Vec2::new(1777.0, 476.4),
    Vec2::new(1778.4, 447.1),
    Vec2::new(1771.3, 418.6),
    Vec2::new(1756.2, 393.4),
    Vec2::new(1734.5, 373.7),
    Vec2::new(1707.9, 361.2),
    Vec2::new(1678.9, 356.9),
    Vec2::new(1618.9, 356.9),
    // 9. Moria: west under the top, and a sweep to the left and the right.
    Vec2::new(1558.9, 356.9),
    Vec2::new(1498.9, 356.9),
    Vec2::new(1467.0, 359.2),
    Vec2::new(1435.8, 366.1),
    Vec2::new(1405.9, 377.5),
    Vec2::new(1376.0, 388.9),
    Vec2::new(1344.8, 395.8),
    Vec2::new(1313.0, 398.1),
    // 10. Lothlórien: the lap snakes between the three ways of its split, as
    // on Ghost Island's castle ridge.
    Vec2::new(1257.9, 398.1),
    Vec2::new(1216.7, 389.0),
    Vec2::new(1175.4, 374.2),
    Vec2::new(1134.1, 376.0),
    Vec2::new(1092.9, 398.1),
    Vec2::new(1051.7, 420.2),
    Vec2::new(1010.4, 422.0),
    Vec2::new(969.1, 407.2),
    Vec2::new(927.9, 398.1),
    Vec2::new(877.9, 398.1),
    // 11. Down to the Anduin, along its gorge in esses, and west down its
    // middle in a sweep to the right and the left, between the two kings of
    // the Argonath.
    Vec2::new(847.9, 398.1),
    Vec2::new(820.0, 408.6),
    Vec2::new(796.7, 427.4),
    Vec2::new(773.3, 446.2),
    Vec2::new(745.4, 456.8),
    Vec2::new(715.9, 453.1),
    Vec2::new(690.5, 437.4),
    Vec2::new(667.9, 417.5),
    Vec2::new(642.5, 401.8),
    Vec2::new(613.0, 398.1),
    Vec2::new(583.0, 398.1),
    Vec2::new(551.6, 397.0),
    Vec2::new(520.3, 393.7),
    Vec2::new(489.4, 388.3),
    Vec2::new(458.9, 380.7),
    Vec2::new(429.1, 371.0),
    Vec2::new(399.2, 361.3),
    Vec2::new(368.7, 353.7),
    Vec2::new(337.8, 348.2),
    Vec2::new(306.5, 344.9),
    Vec2::new(275.1, 343.8),
    // 12. Past the lake, round to the right over the shoulder of Amon Hen
    // onto the start straight, back in the Shire.
    Vec2::new(235.2, 343.8),
    Vec2::new(204.5, 341.8),
    Vec2::new(174.3, 335.8),
    Vec2::new(145.2, 325.9),
    Vec2::new(117.6, 312.3),
    Vec2::new(92.0, 295.2),
    Vec2::new(68.9, 274.9),
    Vec2::new(48.6, 251.8),
    Vec2::new(31.5, 226.2),
    Vec2::new(17.9, 198.7),
    Vec2::new(8.0, 169.5),
    Vec2::new(2.0, 139.4),
    Vec2::new(0.0, 108.7),
];

/// Up Weathertop, 44 m, and down to the ford; up Caradhras, 32 m, and down
/// round it to the mines, 38 m under the pass; up through Moria and down
/// through Lothlórien to the Anduin, at the water's level between the kings;
/// over Amon Hen and down to the line. Up and down at up to 12 %.
const ELEVATION: [(f32, f32); 55] = [
    // The start straight, over a crest where the bike flies, and round Bag
    // End's hill.
    (0.0, 0.0),
    (110.0, 0.4),
    (185.0, 4.6),
    (225.0, 3.0),
    (300.0, 0.8),
    (370.0, 0.8),
    // Through the Shire's fields, and over the crest of its straight, where
    // the bike flies (as at Ghost Island's castle street, sharper), to the
    // Brandywine.
    (660.0, 5.0),
    (790.0, 8.0),
    (890.0, 6.2),
    (940.0, 6.0),
    (1010.0, 11.0),
    (1045.0, 9.2),
    (1120.0, 6.0),
    // Bree, and up Weathertop at up to 12 %.
    (1340.0, 6.5),
    (1494.0, 7.0),
    (1554.0, 12.0),
    (1686.0, 27.5),
    (1796.0, 40.0),
    (1878.0, 45.5),
    // Over its top: a crest where the bike flies.
    (1898.0, 46.5),
    (1968.0, 51.5),
    (2003.0, 49.6),
    (2058.0, 46.5),
    // Down through the Trollshaws at up to 12 %, to the ford, low over the
    // water.
    (2154.0, 38.0),
    (2244.0, 27.5),
    (2334.0, 17.0),
    (2424.0, 7.0),
    (2494.0, 3.0),
    (2534.0, 0.5),
    // Climbing steadily through Rivendell's split (as Ghost Island's
    // temple's).
    (2614.0, 3.0),
    (2964.0, 15.0),
    // Up Caradhras at up to 12 %, to the pass right over the mines.
    (3094.0, 21.0),
    (3194.0, 30.0),
    (3294.0, 39.0),
    (3354.0, 45.5),
    (3394.0, 47.5),
    // Down round the mountain to the Doors of Durin; through Moria, down
    // into it and up again.
    (3454.0, 45.5),
    (3614.0, 36.0),
    (3774.0, 24.0),
    (3956.0, 15.0),
    (4016.0, 12.0),
    (4076.0, 9.0),
    (4248.0, 20.0),
    (4328.0, 24.0),
    // Down steadily through Lothlórien's split (as Ghost Island's castle's).
    (4382.0, 23.5),
    (4732.0, 11.5),
    // Down the Anduin's gorge to the water, and the causeway, low over it.
    (4812.0, 9.5),
    (5082.0, 1.5),
    (5152.0, -1.4),
    (5406.0, -1.4),
    // Up over the shoulder of Amon Hen, and down to the line.
    (5466.0, 1.0),
    (5566.0, 6.5),
    (5656.0, 10.5),
    (5726.0, 9.5),
    (5884.0, 1.0),
];

/// The ways off the lap, each through points to one side of it (m along
/// the lap and from its centre line in the comments), from where it leaves
/// the lap to where it joins it again: Ghost Island's, turned and placed.
/// In Rivendell, to the left (east): it swings out to the left before the
/// lap does, and back to it late.
const RIVENDELL_LEFT: [Vec2; 11] = [
    // 2624 m, 0 m.
    Vec2::new(1578.2, -322.0),
    // 2654 m, 4 m.
    Vec2::new(1589.0, -293.4),
    // 2692 m, 13 m.
    Vec2::new(1611.6, -260.5),
    // 2722 m, 32 m.
    Vec2::new(1633.5, -226.3),
    // 2742 m, 48 m.
    Vec2::new(1644.7, -193.3),
    // 2762 m, 60 m.
    Vec2::new(1642.1, -160.3),
    // 2796 m, 65 m.
    Vec2::new(1628.1, -127.3),
    // 2874 m, 57 m.
    Vec2::new(1608.7, -94.4),
    // 2908 m, 29 m.
    Vec2::new(1589.4, -66.7),
    // 2928 m, 11 m.
    Vec2::new(1579.4, -39.7),
    // 2950 m, 2 m.
    Vec2::new(1575.9, -15.3),
];

/// In Lothlórien, to the left (south): it runs straight on where the lap
/// swings right, and out round a wide curve back to it.
const LORIEN_LEFT: [Vec2; 11] = [
    // 4394 m, 1 m.
    Vec2::new(1246.2, 397.9),
    // 4422 m, 15 m.
    Vec2::new(1215.0, 404.0),
    // 4444 m, 40 m.
    Vec2::new(1183.0, 418.9),
    // 4506 m, 65 m.
    Vec2::new(1152.3, 438.9),
    // 4566 m, 64 m.
    Vec2::new(1119.3, 457.3),
    // 4594 m, 57 m.
    Vec2::new(1086.3, 467.5),
    // 4610 m, 44 m.
    Vec2::new(1053.3, 464.5),
    // 4634 m, 25 m.
    Vec2::new(1020.3, 448.1),
    // 4666 m, 9 m.
    Vec2::new(986.6, 424.2),
    // 4696 m, 3 m.
    Vec2::new(961.8, 408.1),
    // 4718 m, 0 m.
    Vec2::new(941.5, 400.6),
];

/// In Lothlórien, to the right (north): it swings out to the right before
/// the lap does, and back to it late.
const LORIEN_RIGHT: [Vec2; 11] = [
    // 4394 m, -0 m.
    Vec2::new(1246.6, 396.1),
    // 4424 m, -4 m.
    Vec2::new(1218.5, 385.2),
    // 4462 m, -14 m.
    Vec2::new(1186.2, 362.8),
    // 4490 m, -32 m.
    Vec2::new(1152.3, 341.0),
    // 4512 m, -48 m.
    Vec2::new(1119.3, 329.8),
    // 4530 m, -60 m.
    Vec2::new(1086.3, 332.4),
    // 4564 m, -65 m.
    Vec2::new(1053.3, 346.4),
    // 4642 m, -57 m.
    Vec2::new(1020.4, 365.8),
    // 4676 m, -29 m.
    Vec2::new(992.2, 384.9),
    // 4698 m, -11 m.
    Vec2::new(964.6, 394.8),
    // 4718 m, -2 m.
    Vec2::new(941.8, 398.7),
];

/// How much higher than the lap beside it each way runs (m along it, m):
/// over a hump, or through a dip, where it is far from the lap, as at Ghost
/// Island.
const RIVENDELL_LEFT_RISE: [(f32, f32); 35] = [
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

const LORIEN_LEFT_RISE: [(f32, f32); 35] = [
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

const LORIEN_RIGHT_RISE: [(f32, f32); 33] = [
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

/// The pond by the start straight in Hobbiton: its middle and radius (m).
const POND: Vec2 = Vec2::new(-110.0, -210.0);
const POND_RADIUS: f32 = 50.0;
/// Rivers, as points (x, z) along them with how far their banks are from
/// the middle there (m): the Brandywine, north to south under its bridge
/// into a pool; the Bruinen, from the mountains west across the ford into
/// a pool; and the Anduin, from the south into the middle of the road,
/// widening down the causeway between the kings and narrowing into the lake
/// past it.
const BRANDYWINE_RIVER: [(Vec2, f32); 5] = [
    (Vec2::new(600.0, -1100.0), 18.0),
    (Vec2::new(570.0, -800.0), 18.0),
    (Vec2::new(560.0, -568.0), 18.0),
    (Vec2::new(550.0, -330.0), 18.0),
    (Vec2::new(520.0, -200.0), 40.0),
];
const BRUINEN_RIVER: [(Vec2, f32); 6] = [
    (Vec2::new(1985.0, -565.0), 22.0),
    (Vec2::new(1735.0, -455.0), 22.0),
    (Vec2::new(1576.0, -417.0), 22.0),
    (Vec2::new(1435.0, -375.0), 22.0),
    (Vec2::new(1325.0, -335.0), 22.0),
    (Vec2::new(1235.0, -305.0), 40.0),
];
const ANDUIN: [(Vec2, f32); 12] = [
    (Vec2::new(680.0, 800.0), 18.0),
    (Vec2::new(630.0, 600.0), 18.0),
    (Vec2::new(598.0, 470.0), 14.0),
    (Vec2::new(578.0, 398.0), 12.0),
    (Vec2::new(545.0, 396.5), 45.0),
    (Vec2::new(495.0, 389.5), 65.0),
    (Vec2::new(430.0, 371.3), 65.0),
    (Vec2::new(399.5, 361.4), 65.0),
    (Vec2::new(351.0, 350.0), 60.0),
    (Vec2::new(320.0, 380.0), 45.0),
    (Vec2::new(260.0, 430.0), 32.0),
    (Vec2::new(205.0, 478.0), 30.0),
];
/// Mirrormere, below Moria's gate, and Nen Hithoel, the lake the Anduin
/// opens into past the kings: their middles and radii (m).
const MIRRORMERE: Vec2 = Vec2::new(1350.0, 505.0);
const MIRRORMERE_RADIUS: f32 = 45.0;
const NEN_HITHOEL: Vec2 = Vec2::new(180.0, 500.0);
const NEN_HITHOEL_RADIUS: f32 = 85.0;
/// Where the two kings of the Argonath stand, either side of the causeway.
const KINGS: [Vec2; 2] = [Vec2::new(416.0, 414.0), Vec2::new(444.0, 328.5)];

/// How far `point` is from the shores of the rivers and lakes: negative in
/// their water. Their shores wander a little.
fn water(point: Vec2) -> f32 {
    let wander = 4.0 * (2.0 * fbm(point / 40.0 + 7.0, 2) - 1.0);
    let pond = point.distance(POND) - POND_RADIUS;
    let mirrormere = point.distance(MIRRORMERE) - MIRRORMERE_RADIUS;
    let lake = point.distance(NEN_HITHOEL) - NEN_HITHOEL_RADIUS;
    let rivers = [&BRANDYWINE_RIVER[..], &BRUINEN_RIVER, &ANDUIN]
        .into_iter()
        .map(|river| river_banks(river, point))
        .fold(f32::MAX, f32::min);
    pond.min(mirrormere).min(lake).min(rivers) + wander
}

/// How far `point` is from the banks of a river through `points` (each
/// with how far its banks are from its middle there): negative in it.
fn river_banks(points: &[(Vec2, f32)], point: Vec2) -> f32 {
    points
        .windows(2)
        .map(|pair| {
            let ((a, wide_a), (b, wide_b)) = (pair[0], pair[1]);
            let ab = b - a;
            let t = ((point - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            point.distance(a + ab * t) - wide_a.lerp(wide_b, t)
        })
        .fold(f32::MAX, f32::min)
}

/// Weathertop, Caradhras over the mines, and Amon Hen: their middles,
/// radii and heights (m).
const WEATHERTOP_HILL: Massif = climbed_massif(1225.0, -665.0, 280.0, 46.0);
const CARADHRAS_PEAK: Massif = climbed_massif(1595.0, 385.0, 280.0, 44.0);

const MASSIFS: [Massif; 10] = [
    WEATHERTOP_HILL,
    CARADHRAS_PEAK,
    // Bag End's hill, on the outside of the bend at the end of the start
    // straight, and Amon Hen, on the outside of the last bend.
    climbed_massif(-110.0, -600.0, 150.0, 16.0),
    climbed_massif(40.0, 400.0, 150.0, 20.0),
    // The cliffs on the far side of Rivendell's valley.
    massif(1735.0, -200.0, 80.0, 40.0),
    massif(1745.0, -90.0, 70.0, 36.0),
    // The cliffs of the Anduin's gorge either side, and the rocks the kings
    // stand on in the river.
    massif(461.0, 276.0, 55.0, 42.0),
    massif(399.0, 466.0, 55.0, 42.0),
    massif(KINGS[0].x, KINGS[0].y, 16.0, 6.0),
    massif(KINGS[1].x, KINGS[1].y, 16.0, 6.0),
];

const CUTTINGS: [Cutting; 6] = [
    // The cliffs on the right of Rivendell's valley, down which its
    // waterfalls pour.
    Cutting {
        span: span(FORD.to, 3030.0),
        side: Side::Right,
        height: 32.0,
    },
    // The Walls of Moria, round the doors, and the rock round its east gate.
    Cutting {
        span: span(MORIA.from - 40.0, MORIA.from + 90.0),
        side: Side::Left,
        height: 22.0,
    },
    Cutting {
        span: span(MORIA.from - 40.0, MORIA.from + 90.0),
        side: Side::Right,
        height: 22.0,
    },
    Cutting {
        span: span(MORIA.to - 70.0, MORIA.to + 30.0),
        side: Side::Left,
        height: 16.0,
    },
    Cutting {
        span: span(MORIA.to - 70.0, MORIA.to + 30.0),
        side: Side::Right,
        height: 16.0,
    },
    // The gorge's cliffs on the right, away from the river.
    Cutting {
        span: span(GORGE.from, GORGE.to - 10.0),
        side: Side::Right,
        height: 24.0,
    },
];

/// Far off: the Misty Mountains to the east, a range of peaks under snow,
/// and low hills round the rest of the horizon.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(850.0, -80.0);
    let hills = smoothstep(1500.0, 2500.0, point.distance(CENTER))
        * 150.0
        * (0.35 + 0.65 * fbm(point / 380.0, 4));
    // Running north and south, a little ragged, past the far side of the
    // lap; its peaks over the snow line.
    let range = smoothstep(
        2150.0,
        2750.0,
        point.x + 120.0 * (fbm(point / 900.0, 2) - 0.5),
    );
    let peaks = 380.0 + 620.0 * fbm(point / 520.0 + 3.0, 4).powf(1.4);
    hills.max(range * peaks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    /// Weathertop, from its foot, and its top: a straight over a crest.
    const WEATHERTOP: Span = span(1494.0, TROLLSHAWS.from);
    const SUMMIT: Span = span(1878.0, 2058.0);

    #[test]
    fn the_lap_goes_over_caradhras_and_then_through_moria_beneath() {
        let track = Track::new(&FELLOWSHIP_ROAD.layout);
        let line = track.centerline();
        // Through the mines at the end, over the pass first: crossing well
        // above.
        let under = track.index_at(MORIA.from)..track.index_at(MORIA.to);
        let (gap, over, under) = (track.index_at(PASS.from)..track.index_at(PASS.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the pass and the mines have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 35.0, "{above} m above");
        // Under the mountain's top.
        assert!(line[under].distance(CARADHRAS_PEAK.center) < 0.3 * CARADHRAS_PEAK.radius);
    }

    #[test]
    fn the_hills_are_steep_and_the_road_climbs_and_drops_a_lot() {
        let track = Track::new(&FELLOWSHIP_ROAD.layout);
        let step = track.step_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let grade =
            |index: usize| (track.elevations()[index + 1] - track.elevations()[index]) / step;
        let steepest = |stretch: Span| {
            let indices = track.index_at(stretch.from)..track.index_at(stretch.to);
            let up = indices.clone().map(grade).fold(0.0, f32::max);
            let down = indices.map(grade).fold(0.0, f32::min);
            (up, -down)
        };
        // Up Weathertop, 40 m and more, and down it to the ford, at nearly
        // 12 %.
        assert!(height(SUMMIT.from) - height(WEATHERTOP.from) > 35.0);
        assert!(height(SUMMIT.to) - height(FORD.from) > 40.0);
        assert!(steepest(WEATHERTOP).0 > 0.115);
        assert!(steepest(span(SUMMIT.to, FORD.from)).1 > 0.115);
        // Up from Rivendell over Caradhras, 30 m and more, at more than
        // 11 %, and down round it to the mines.
        let pass = (track.index_at(PASS.from)..track.index_at(PASS.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(
            pass - height(RIVENDELL_SPLIT.1) > 30.0,
            "{pass} m at the top"
        );
        assert!(steepest(CARADHRAS).0 > 0.11);
        assert!(height(PASS.to) - height(MORIA.from) > 30.0);
        // Down to the Anduin's water, from the mines' gate.
        assert!(height(MORIA.to) - height(CAUSEWAY.from + 50.0) > 20.0);
    }

    #[test]
    fn the_road_winds_in_esses_through_the_trollshaws_and_the_gorge() {
        // Bends one after another each the other way, tighter than 70 m.
        let track = Track::new(&FELLOWSHIP_ROAD.layout);
        for stretch in [TROLLSHAWS, GORGE] {
            let bends: Vec<f32> = (track.index_at(stretch.from)..track.index_at(stretch.to))
                .map(|index| track.curvature(index))
                .filter(|curvature| curvature.abs() > 1.0 / 70.0)
                .fold(Vec::new(), |mut bends, curvature| {
                    if bends
                        .last()
                        .is_none_or(|last: &f32| last.signum() != curvature.signum())
                    {
                        bends.push(curvature);
                    }
                    bends
                });
            assert!(
                bends.len() >= 3,
                "{} bends from {}",
                bends.len(),
                stretch.from
            );
        }
    }

    #[test]
    fn the_road_runs_down_the_anduin_between_the_kings_and_is_dry_but_on_its_bridges() {
        let track = Track::new(&FELLOWSHIP_ROAD.layout);
        let lap = track.lap_length();
        let on_bridge = |along: f32| {
            FELLOWSHIP_ROAD
                .scenery
                .bridges
                .iter()
                .any(|bridge| bridge.span.contains(along, lap))
        };
        // Wet only on the bridges; each of them over water.
        let mut wet = [0.0; 3];
        for index in 0..track.sample_count() {
            let along = index as f32 * track.step_length();
            if water(track.centerline()[index]) < 0.0 {
                assert!(on_bridge(along), "wet road at {along} m");
                for (k, bridge) in FELLOWSHIP_ROAD.scenery.bridges.iter().enumerate() {
                    if bridge.span.contains(along, lap) {
                        wet[k] += track.step_length();
                    }
                }
            }
        }
        for (k, bridge) in FELLOWSHIP_ROAD.scenery.bridges.iter().enumerate() {
            assert!(wet[k] > 0.5 * bridge.span.length(lap), "{wet:?}");
        }
        // Down the middle of the river all along the causeway, the water
        // either side as far as the kings and past them.
        assert!(
            wet[2] > 0.85 * CAUSEWAY.length(lap),
            "{} m over water",
            wet[2]
        );
        for king in KINGS {
            let index = track.nearest_index(king);
            let along = index as f32 * track.step_length();
            assert!(CAUSEWAY.contains(along, lap));
            let road = track.centerline()[index];
            let beyond = road + (king - road) * 1.25;
            assert!(water(beyond) < 0.0, "{beyond}");
            assert!((40.0..50.0).contains(&king.distance(road)));
        }
    }

    #[test]
    fn the_misty_mountains_rise_under_snow_to_the_east_and_nothing_by_the_road() {
        let track = Track::new(&FELLOWSHIP_ROAD.layout);
        for &point in track.centerline() {
            assert!(mountains(point) < 1e-3, "{point}");
        }
        // Peaks over the snow line (560 m) along the range to the east.
        let highest = (0..60)
            .map(|k| mountains(Vec2::new(3000.0, -1500.0 + k as f32 * 50.0)))
            .fold(0.0, f32::max);
        assert!(highest > 650.0, "{highest} m");
        let west = (0..60)
            .map(|k| mountains(Vec2::new(-1800.0, -1500.0 + k as f32 * 50.0)))
            .fold(0.0, f32::max);
        assert!(west < 200.0, "{west} m");
    }
}
