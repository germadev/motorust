//! Fellowship Road: the journey of *The Fellowship of the Ring*, its
//! stretches in the order of the film, from the Shire to the Argonath and,
//! past the finish, home again (*There and Back Again*). It crosses the
//! Misty Mountains twice, in a figure of eight: first over them, by the pass
//! of Caradhras, and then, as the Fellowship turned back from the snow,
//! through them, by the mines of Moria, under the pass.
//!
//! 1. Hobbiton: the start straight, north through the wood, over a crest,
//!    a long sweep to the right down to the Water and low over it, a
//!    sweep to the left round the village's hill, and one to the right
//!    past Bag End's hill.
//! 2. The Shire: a sweep to the right and the left over the bridge on the
//!    Brandywine, and through its fields.
//! 3. Bree: through its gate, right and left through the town.
//! 4. Weathertop: up its flank at up to 12 %, round to the left and the
//!    right, over its top on a straight with a crest where the bike takes
//!    off, 51 m up, and down at up to 12 %.
//! 5. The Trollshaws: down in esses to the Ford of Bruinen, low over the
//!    water.
//! 6. Rivendell: south along the valley under its cliffs, where the road
//!    splits in two: the lap snakes along it and a way to the left swings
//!    out over a hump.
//! 7. Caradhras: up in bends at up to 12 % to the pass, 49 m up, right over
//!    the mines.
//! 8. Down round the mountain: on south, round to the left, east, north and
//!    back west, to the Doors of Durin.
//! 9. Moria: through the Doors of Durin, open, where the road narrows; west
//!    through the mountain, under the pass, 38 m beneath it,
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

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line, in the order of the
/// film.
/// The wood the race starts in, by Hobbiton; Hobbiton, down to the Water
/// and over it on an old wooden bridge, and round the pool past Bag End's
/// hill.
const WOOD: Span = span(5645.1, 140.0);
const HOBBITON: Span = span(WOOD.to, 867.0);
const HOBBITON_BRIDGE: Span = span(375.0, 455.0);
const SHIRE: Span = span(HOBBITON.to, BREE.from);
const BRANDYWINE: Span = span(885.1, 945.1);
const BREE: Span = span(1085.1, 1299.1);
const TROLLSHAWS: Span = span(1959.1, FORD.from);
/// The Ford of Bruinen: low over the river.
const FORD: Span = span(2300.1, 2360.1);
const RIVENDELL: Span = span(FORD.to, CARADHRAS.from);
const CARADHRAS: Span = span(2819.1, 3219.1);
/// The top of the pass, right over the mines.
const PASS: Span = span(3155.1, 3219.1);
const MORIA: Span = span(3790.1, 4133.1);
const LOTHLORIEN: Span = span(MORIA.to, GORGE.from);
const GORGE: Span = span(4587.1, CAUSEWAY.from);
/// Down the middle of the Anduin, between the kings of the Argonath: a
/// causeway narrower than the road, its walls this far from the middle
/// (m).
const CAUSEWAY: Span = span(4900.1, 5235.1);
const CAUSEWAY_WALL: f32 = 6.5;
/// How far from the middle the walls stand through the Doors of Durin (m),
/// just inside their leaves.
const DURIN_WALL: f32 = 5.5;
const AMON_HEN: Span = span(CAUSEWAY.to, WOOD.from);
/// Where the seat of seeing stands by the road up Amon Hen (m).
const AMON_HEN_SEAT: f32 = 5405.1;
/// How far from the middle the walls stand over the bridge on the Water in
/// Hobbiton, an old, narrow one (m).
const HOBBITON_BRIDGE_WALL: f32 = 6.5;
/// Where the road splits: a way through the Prancing Pony in Bree, in two in
/// Rivendell, three ways in Lothlórien.
const BREE_SPLIT: (f32, f32) = (1097.1, 1525.9);
const RIVENDELL_SPLIT: (f32, f32) = (2364.1, 2714.1);
/// How far along Bree's way the middle of the Prancing Pony's archway is
/// (m), how long its passage is (m), and how far from the middle of the
/// way its walls stand through it (m), just inside the archway.
const PONY_ALONG: f32 = 111.5;
const PONY_PASSAGE: f32 = 33.0;
const PONY_WALL: f32 = 4.0;
const LORIEN_SPLIT: (f32, f32) = (4187.1, 4537.1);

/// The Ring's gold, the Shire's green and cream.
const GOLD: Color = Color::srgb(0.86, 0.68, 0.22);
const GREEN: Color = Color::srgb(0.22, 0.45, 0.2);
const CREAM: Color = Color::srgb(0.93, 0.88, 0.72);
/// The warm light that fills Rivendell's valley (see `MiddleEarth::glows`),
/// as of the evening sky over it.
const RIVENDELL_LIGHT: Color = Color::srgb(1.0, 0.86, 0.68);
/// How steep Rivendell's rock is grass (the upward part of the ground's
/// normal under which it is rock, see `Turf::steepest`): up to 53°.
const RIVENDELL_TURF: f32 = 0.6;
/// The browner stone of the rock on the right of the road down the
/// Anduin's gorge (see `MiddleEarth::rock_tints`).
const GORGE_STONE: Color = Color::srgb(0.62, 0.53, 0.44);
/// The sunbeams down from the sky onto Rivendell's halls: how bright on
/// them (lx), spread wide over them and round them, and their light,
/// golden.
const SUNBEAM: f32 = 4500.0;
/// How bright the diffused sunbeams are on what they light (lx): less, so
/// that the seat of seeing and the ruins on Weathertop keep their detail;
/// and Amon Hen's, the most diffused, still less.
const DIFFUSED_SUNBEAM: f32 = 3500.0;
const AMON_HEN_SUNBEAM: f32 = 2500.0;
const SUNBEAM_LIGHT: Color = Color::srgb(1.0, 0.84, 0.58);
/// How their shafts show in the air (see `Shaft`): Rivendell's diffused,
/// spreading to four and a half times as wide up into the sky; Amon Sûl's
/// more, to six times as wide, and fainter; Amon Hen's most, to nine.
const SUNBEAM_SHAFT: Shaft = Shaft {
    spread: 4.5,
    light: 0.032,
};
const DIFFUSED_SHAFT: Shaft = Shaft {
    spread: 6.0,
    light: 0.03,
};
const AMON_HEN_SHAFT: Shaft = Shaft {
    spread: 9.0,
    light: 0.024,
};

pub const FELLOWSHIP_ROAD: Circuit = Circuit {
    name: "Fellowship Road",
    name_en: "Fellowship Road",
    description: "Inspirada en La Comunidad del Anillo: de la Comarca a los\n\
                  Argonath, un ocho sobre y bajo las montañas, cuestas y dos bifurcaciones",
    description_en: "Inspired by The Fellowship of the Ring: from the Shire to the\n\
                     Argonath, a figure of eight over and under the mountains, steep hills and two splits",
    // "Beyond the Final Turn".
    // Fields and woods: birds, the wind, and crickets in the grass.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.75,
        cicadas: 0.2,
        wind: 0.6,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/beyond_the_final_turn.mp3"),
        length: 179.2,
        loudness: -15.5,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Hardly banked where the road splits, as at Ghost Island.
        banking: &[
            Banking {
                from: BREE_SPLIT.0,
                to: BREE_SPLIT.1,
                factor: 0.25,
            },
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
        // The old bridge over the Water in Hobbiton, the way through the
        // Doors of Durin, between their leaves, and the causeway between the
        // kings are narrower than the road: its walls close in on them.
        narrows: &[
            Narrowing {
                from: HOBBITON_BRIDGE.from,
                to: HOBBITON_BRIDGE.to,
                wall: HOBBITON_BRIDGE_WALL,
            },
            Narrowing {
                from: MORIA.from - 12.0,
                to: MORIA.from + 20.0,
                wall: DURIN_WALL,
            },
            Narrowing {
                from: CAUSEWAY.from,
                to: CAUSEWAY.to,
                wall: CAUSEWAY_WALL,
            },
        ],
        // In Bree a way to the left of the lap, through the Prancing Pony's
        // archway, its walls closing in through it, and on, a short cut,
        // across the inside of the lap's turn up Weathertop; in Rivendell a
        // way to the left of the lap over a hump; in Lothlórien one to the left
        // over a hump and one to the right through a dip.
        branches: &[
            Branch {
                from: BREE_SPLIT.0,
                to: BREE_SPLIT.1,
                approach: &BREE_WAY,
                vertical_loop: None,
                exit: &[],
                rise: &[],
                narrows: &[Narrowing {
                    from: PONY_ALONG - PONY_PASSAGE / 2.0,
                    to: PONY_ALONG + PONY_PASSAGE / 2.0,
                    wall: PONY_WALL,
                }],
            },
            Branch {
                from: RIVENDELL_SPLIT.0,
                to: RIVENDELL_SPLIT.1,
                approach: &RIVENDELL_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &RIVENDELL_LEFT_RISE,
                narrows: &[],
            },
            Branch {
                from: LORIEN_SPLIT.0,
                to: LORIEN_SPLIT.1,
                approach: &LORIEN_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &LORIEN_LEFT_RISE,
                narrows: &[],
            },
            Branch {
                from: LORIEN_SPLIT.0,
                to: LORIEN_SPLIT.1,
                approach: &LORIEN_RIGHT,
                vertical_loop: None,
                exit: &[],
                rise: &LORIEN_RIGHT_RISE,
                narrows: &[],
            },
        ],
    },
    // Middle-earth's own props, made with Meshy (see `PROPS`), among the
    // walls, bridges, cliffs, forests and waterfalls of the other tracks.
    scenery: Scenery {
        coast: Coast::Inland(water),
        hills: 26.0,
        // A lush green land's greens, deep: lighter, the low sun made them
        // pale and vivid.
        greens: Greens::Lush,
        grass_shade: 0.55,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &KNOLLS,
        mountains,
        curbs: [GREEN, CREAM],
        // Wooden fences through the wood the race starts in, along the lanes
        // of Hobbiton and the street of Bree, and where the small props stand
        // by the road, so that they show whole (a wall would hide them);
        // hedgerows through the Shire, pale walls where the road splits in
        // Lothlórien, and stone walls everywhere else.
        walls: &[
            fence(span(WOOD.from, HOBBITON.to), Side::Left),
            fence(span(WOOD.from, HOBBITON.to), Side::Right),
            fence(span(968.0, 982.0), Side::Left),
            fence(span(1013.0, 1027.0), Side::Right),
            WallRun {
                span: SHIRE,
                side: Side::Left,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: SHIRE,
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            fence(BREE, Side::Left),
            fence(BREE, Side::Right),
            fence(span(1757.1, 1773.1), Side::Left),
            fence(span(1797.1, 1813.1), Side::Right),
            fence(span(2368.1, 2382.1), Side::Left),
            fence(span(2368.1, 2382.1), Side::Right),
            fence(span(2788.1, 2802.1), Side::Left),
            fence(span(2788.1, 2802.1), Side::Right),
            fence(span(4143.1, 4157.1), Side::Left),
            fence(span(4143.1, 4157.1), Side::Right),
            // Where the road splits three ways in Lothlórien, pale walls, as
            // of the elves' white stone, standing out against the gold of
            // its woods, so that each way is seen.
            WallRun {
                span: span(LORIEN_SPLIT.0, LORIEN_SPLIT.1),
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: span(LORIEN_SPLIT.0, LORIEN_SPLIT.1),
                side: Side::Right,
                style: WallStyle::Concrete,
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
        // The bridge over the Brandywine, the ford low over the Bruinen, the
        // causeway down the Anduin, and the bridge over the Water in
        // Hobbiton, their fences low (the water shows), all of old,
        // weathered wood.
        bridges: &[
            Bridge {
                span: BRANDYWINE,
                style: BridgeStyle::OldWooden,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: FORD,
                style: BridgeStyle::OldWooden,
                fence: 0.75,
            },
            Bridge {
                span: CAUSEWAY,
                style: BridgeStyle::OldWooden,
                fence: 0.75,
            },
            Bridge {
                span: HOBBITON_BRIDGE,
                style: BridgeStyle::OldWooden,
                fence: WOODEN_FENCE,
            },
        ],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: GOLD,
                text: Some("FELLOWSHIP ROAD"),
            },
            // The checkpoints, 600 m to a kilometre and a half apart (none
            // where the road splits, on the bridges nor in the mines): out of
            // Hobbiton, down from Weathertop, up Caradhras, on the
            // way round the mountain to the doors, and past Lothlórien.
            Gantry {
                along: 868.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1905.1,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2955.1,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3585.1,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4565.1,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[Billboard {
            along: 150.0,
            side: Side::Right,
            text: "MOTORUST",
            color: Color::srgb(0.1, 0.1, 0.12),
            accent: Some((4, crate::ui::RUST_ORANGE)),
        }],
        lamps: &[],
        // Oaks and birches, as in the Shire.
        trees: &[Species::Broadleaf, Species::Poplar],
        // The wood the race starts in, of oaks and dark trees, on from Amon
        // Hen's of broadleaves, as the beech forests where the film was made;
        // dark conifers through the Trollshaws and up Caradhras' foothills;
        // Rivendell's woods in autumn, red and gold, as the film has them;
        // and Lothlórien's in gold.
        forests: &[
            forest(WOOD, &[Species::Broadleaf, Species::Tall]),
            forest(TROLLSHAWS, &[Species::Sugi]),
            forest(RIVENDELL, &[Species::Ginkgo, Species::Momiji]),
            forest(span(CARADHRAS.from, PASS.from), &[Species::Sugi]),
            forest(LOTHLORIEN, &[Species::Ginkgo]),
            forest(
                span(AMON_HEN_SEAT + 20.0, AMON_HEN.to),
                &[Species::Broadleaf, Species::Poplar],
            ),
        ],
        // No houses of the other tracks: Hobbiton has its hobbit holes and
        // Bree its own houses (see `PROPS`).
        houses: Houses {
            max: 0,
            paints: &[
                Color::srgb(0.93, 0.86, 0.66),
                Color::srgb(0.86, 0.74, 0.52),
                Color::srgb(0.95, 0.93, 0.86),
            ],
            spans: &[span(5685.1, 300.0), BREE],
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
        soil: Soil::Granite,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        // Down the cliffs of Rivendell, on the right of the valley: where
        // the foot of the cliff is low, by the road.
        waterfalls: &[
            Waterfall {
                along: 2419.1,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2773.1,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2811.1,
                side: Side::Right,
                width: 12.0,
            },
        ],
        // A summer sunset in high country, its long shadows over the hills,
        // graded as a film; the rivers and lakes green-grey.
        sky: SkyStyle::HighCountry,
        water: Water::River,
        cartoon_water: false,
        pit: None,
        desert: None,
        mine: None,
        wheelie_halo: None,
        japan: None,
        space: None,
        middle_earth: Some(MiddleEarth {
            props: &PROPS,
            flowers: &FLOWERS,
            // Leaves blowing down through Rivendell, red and gold, and
            // through Lothlórien, the mallorns' gold.
            petals: &[
                Petals {
                    area: PetalArea::Road {
                        span: RIVENDELL,
                        reach: 24.0,
                    },
                    kind: PetalKind::Maple,
                },
                Petals {
                    area: PetalArea::Road {
                        span: span(RIVENDELL.from + 150.0, RIVENDELL.to),
                        reach: 24.0,
                    },
                    kind: PetalKind::Gold,
                },
                Petals {
                    area: PetalArea::Road {
                        span: LOTHLORIEN,
                        reach: 24.0,
                    },
                    kind: PetalKind::Gold,
                },
            ],
            halls: &MORIA_HALLS,
            paths: &PATHS,
            // Hobbiton's hillsides, whose banks and doorsteps cut into them
            // stay grass, as the Shire's, but for sheer faces. And Rivendell's
            // rock, as the film's green crags: at the head of its valley, on
            // its far side and down its cliffs on the right, grass on its
            // ledges and gentler slopes, and moss on its faces.
            turf: &[
                Turf {
                    center: Vec2::new(285.0, -495.0),
                    radius: 250.0,
                    steepest: 0.45,
                },
                Turf {
                    center: Vec2::new(1650.0, 180.0),
                    radius: 115.0,
                    steepest: RIVENDELL_TURF,
                },
                Turf {
                    center: Vec2::new(1740.0, -145.0),
                    radius: 130.0,
                    steepest: RIVENDELL_TURF,
                },
                Turf {
                    center: Vec2::new(1555.0, -320.0),
                    radius: 100.0,
                    steepest: RIVENDELL_TURF,
                },
                Turf {
                    center: Vec2::new(1545.0, -170.0),
                    radius: 100.0,
                    steepest: RIVENDELL_TURF,
                },
                Turf {
                    center: Vec2::new(1550.0, -20.0),
                    radius: 100.0,
                    steepest: RIVENDELL_TURF,
                },
            ],
            // The rock on the right of the road down the Anduin's gorge,
            // from 4590 to 4882 m: a browner stone.
            rock_tints: &[
                RockTint {
                    center: Vec2::new(867.9, 355.7),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(800.9, 371.2),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(759.6, 403.9),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(741.4, 414.9),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(727.6, 412.3),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(694.9, 385.3),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
                RockTint {
                    center: Vec2::new(635.1, 357.5),
                    radius: 55.0,
                    color: GORGE_STONE,
                },
            ],
            // Rivendell's halls and the rock at the head of its valley, which
            // the low sun in the south-east lights only from behind as the
            // riders come down the valley at them: a warm light, as of the
            // evening sky, from behind and over the riders, fills them. And
            // up the valley, where the rock on its east side shades both its
            // walls, a light across it onto each wall. And over each of its
            // halls, the pavilion, the Last Homely House and the tower, a
            // sunbeam down from the sky through the clouds, leaning towards
            // the riders, its shaft showing in the air, spreading up into the
            // sky; and diffused ones on the seat of seeing on Amon Hen and on
            // the ruins of Amon Sûl.
            glows: &[
                Glow {
                    from: Vec3::new(1549.2, 133.2, -93.5),
                    to: Vec3::new(1613.0, 36.0, 105.0),
                    illuminance: 3500.0,
                    color: RIVENDELL_LIGHT,
                    angle: 0.6,
                    shaft: None,
                },
                Glow {
                    from: Vec3::new(1720.0, 100.0, -300.0),
                    to: Vec3::new(1565.8, 25.0, -221.8),
                    illuminance: 2500.0,
                    color: RIVENDELL_LIGHT,
                    angle: 0.7,
                    shaft: None,
                },
                Glow {
                    from: Vec3::new(1450.0, 100.0, -300.0),
                    to: Vec3::new(1634.0, 20.0, -208.0),
                    illuminance: 2500.0,
                    color: RIVENDELL_LIGHT,
                    angle: 0.7,
                    shaft: None,
                },
                Glow {
                    from: Vec3::new(1604.8, 163.2, -11.3),
                    to: Vec3::new(1624.5, 27.3, 49.0),
                    illuminance: SUNBEAM,
                    color: SUNBEAM_LIGHT,
                    angle: 0.12,
                    shaft: Some(SUNBEAM_SHAFT),
                },
                Glow {
                    from: Vec3::new(1599.4, 175.9, 52.2),
                    to: Vec3::new(1619.1, 40.0, 112.5),
                    illuminance: SUNBEAM,
                    color: SUNBEAM_LIGHT,
                    angle: 0.25,
                    shaft: Some(SUNBEAM_SHAFT),
                },
                Glow {
                    from: Vec3::new(1557.5, 189.1, 91.7),
                    to: Vec3::new(1577.1, 53.2, 151.9),
                    illuminance: SUNBEAM,
                    color: SUNBEAM_LIGHT,
                    angle: 0.12,
                    shaft: Some(SUNBEAM_SHAFT),
                },
                // And, wider and far more diffused, one on the seat of seeing
                // on Amon Hen, and one on the ruins of Amon Sûl on
                // Weathertop.
                Glow {
                    from: Vec3::new(159.8, 152.2, 331.9),
                    to: Vec3::new(96.8, 16.3, 324.2),
                    illuminance: AMON_HEN_SUNBEAM,
                    color: SUNBEAM_LIGHT,
                    angle: 0.2,
                    shaft: Some(AMON_HEN_SHAFT),
                },
                Glow {
                    from: Vec3::new(1213.3, 195.2, -718.3),
                    to: Vec3::new(1271.7, 59.2, -693.7),
                    illuminance: DIFFUSED_SUNBEAM,
                    color: SUNBEAM_LIGHT,
                    angle: 0.12,
                    shaft: Some(DIFFUSED_SHAFT),
                },
            ],
        }),
    },
};

/// The halls of Moria, through the Doors of Durin: the way in, of the
/// dwarves' masonry, as high as the doors; a natural cave under the pass; the
/// Dwarrowdelf, a great hall of pillars far wider than the road, its roof
/// lost high above; and a natural cave out to the east gate.
const MORIA_HALLS: [Hall; 4] = [
    Hall {
        span: span(MORIA.from - 20.0, 3835.1),
        stone: Stone::Masonry,
        crown: 30.0,
        width: 0.0,
    },
    Hall {
        span: span(3835.1, 3900.1),
        stone: Stone::Cave,
        crown: 22.0,
        width: 5.0,
    },
    Hall {
        span: span(3900.1, 4055.1),
        stone: Stone::Masonry,
        crown: 34.0,
        width: 16.0,
    },
    Hall {
        span: span(4055.1, MORIA.to + 20.0),
        stone: Stone::Cave,
        crown: 20.0,
        width: 4.0,
    },
];

/// The props of Middle-earth, made with Meshy, in the order of the lap:
/// where the road comes at them, and those with a front turned to face the
/// riders coming, towards the road 60 m before them (by at most 45°), so
/// that they see it from afar (see `docs/tracks.md`).
const PROPS: [Prop<MiddleEarthModel>; 55] = [
    // The wood the race starts in: Gandalf's cart, loaded with fireworks,
    // by the road among the trees, and a pile of logs.
    by_road(MiddleEarthModel::GandalfCart, 32.0, -14.5).turned(0.79),
    by_road(MiddleEarthModel::LogPile, 84.0, -13.5),
    // Hobbiton, down the long sweep to the Water and round the village's
    // hills: the mill across the water on the left, ahead down the sweep;
    // ahead on the left, past the bridge, hobbit holes spread low round the
    // foot of the hill inside the sweep to the left, facing the riders
    // coming; ahead on the right, across the field over the pool, more
    // spread over the whole flank of the other hill that faces the road,
    // from the south to the north, facing the road, a little towards the
    // riders coming; and on the outside of the last sweep, on the other
    // side of the road from them, the Green Dragon raised over the road and
    // its fence, facing the riders coming, its sign by the road, and, up on
    // its hill, Bag End under its oak, and the party tree in its field past
    // it, ahead all the way round.
    by_road(MiddleEarthModel::HobbitDoorBlue, 613.9, 84.7)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorRed, 503.9, 103.5)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorYellow, 434.0, 101.9)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorGreen, 398.0, 128.3)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::Mill, 360.0, 108.0).facing_riders(),
    standing(MiddleEarthModel::HobbitDoorGreen, 391.1, -482.2, 2.0816).scaled(1.4),
    standing(MiddleEarthModel::HobbitDoorRed, 394.3, -455.2, 2.0208).scaled(1.4),
    standing(MiddleEarthModel::HobbitDoorYellow, 410.3, -422.5, 1.8387).scaled(1.4),
    by_road(MiddleEarthModel::GreenDragonSign, 668.0, 13.5).turned(-0.79),
    by_road(MiddleEarthModel::GreenDragon, 700.0, 34.0).facing_riders(),
    by_road(MiddleEarthModel::BagEnd, 770.0, 70.0).facing_riders(),
    by_road(MiddleEarthModel::BagEndOak, 778.0, 84.0),
    by_road(MiddleEarthModel::PartyTree, 800.0, 50.0),
    // The old tree the hobbits hid under, on the outside of the last of
    // Hobbiton's sweeps; past the Brandywine, a scarecrow in the Shire's
    // fields, and a pile of logs by the road.
    by_road(MiddleEarthModel::RootTree, 845.0, 22.0),
    by_road(MiddleEarthModel::Scarecrow, 975.0, 14.0).turned(-0.79),
    by_road(MiddleEarthModel::LogPile, 1020.0, -13.5),
    standing(MiddleEarthModel::Ferry, 562.0, -626.0, -FRAC_PI_2),
    // Bree: its gate over the road; past it, ahead on the outside of the
    // first bend, the Prancing Pony, 1.4 times as big, a way of the road
    // forking off to the left through its archway (its middle on the way,
    // the inn facing the riders coming down it); and its houses either side
    // of the street, tall and narrow ones and wide ones.
    over_road(MiddleEarthModel::BreeGate, 1091.1),
    standing(MiddleEarthModel::PrancingPony, 846.1, -528.82, 1.2614).scaled(1.4),
    by_road(MiddleEarthModel::BreeHouseB, 1105.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1117.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1160.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1172.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1203.1, -22.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1243.1, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1255.1, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1275.1, -22.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1221.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1233.1, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1247.1, -24.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1305.1, -16.0).turned(0.79),
    // Weathertop: the ring of ruins of Amon Sûl beside the top straight, on
    // the side of the summit, and broken arches either side of the road.
    by_road(MiddleEarthModel::WeathertopRuin, 1813.1, -38.0).turned(0.79),
    by_road(MiddleEarthModel::RuinArch, 1765.1, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::RuinArch, 1805.1, -16.0).turned(0.79),
    // The Trollshaws: the trolls turned to stone on the outside of the
    // esses.
    by_road(MiddleEarthModel::StoneTrolls, 2095.1, 21.0).turned(-0.79),
    // Rivendell: lamps along its paths, and at the head of the valley,
    // ahead down it on the outside of the bend, its halls together one over
    // another up the rock: the domed pavilion on the left, to one side of
    // the others as the riders see them, the Last Homely House on a ledge
    // over the road, and a tower on a crag over it.
    by_road(MiddleEarthModel::ElvenLamp, 2375.1, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2375.1, -13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2795.1, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2795.1, -13.5),
    by_road(MiddleEarthModel::Gazebo, 2815.0, 48.0).facing_riders(),
    by_road(MiddleEarthModel::LastHomelyHouse, 2857.1, 58.0).facing_riders(),
    by_road(MiddleEarthModel::ElvenTower, 2891.7, 50.3)
        .scaled(1.25)
        .facing_riders(),
    // The Doors of Durin, open, in the sheer Walls of Moria: the road goes
    // through them into the mountain, ahead of the riders coming round the
    // last bend before it.
    over_road(MiddleEarthModel::DurinDoors, MORIA.from).turned(PI),
    // Lothlórien, out of Moria's east gate: elven lamps and a mallorn with
    // its flet.
    by_road(MiddleEarthModel::ElvenLamp, 4150.1, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 4150.1, -13.5),
    by_road(MiddleEarthModel::MallornFlet, 4157.1, 40.0),
    // The Anduin: Galadriel's swan ship going down the river beside the
    // causeway, ahead out of the gorge; the two kings of the Argonath in the
    // river either side of the causeway, facing the riders coming down it,
    // and past them, in the gorge, the sailing ship of the elves going down
    // the river beside it.
    standing(MiddleEarthModel::SwanShip, 515.0, 425.0, FRAC_PI_2),
    standing(
        MiddleEarthModel::ArgonathA,
        KINGS[0].x,
        KINGS[0].y,
        KINGS_FACING,
    ),
    standing(
        MiddleEarthModel::ArgonathB,
        KINGS[1].x,
        KINGS[1].y,
        KINGS_FACING,
    ),
    by_road(MiddleEarthModel::ElvenShip, 5150.1, 24.0).turned(PI),
    // Amon Hen: the elven boats drawn up on the shore of Parth Galen, and
    // the seat of seeing up the hill on the outside of the last bend, by the
    // road, its lions looking out over it (its throne along the road, the
    // way the riders go), a sunbeam on it (see `glows`).
    standing(MiddleEarthModel::ElvenBoat, 238.0, 393.0, 2.20),
    standing(MiddleEarthModel::ElvenBoat, 226.0, 404.0, 2.00),
    by_road(MiddleEarthModel::AmonHenSeat, AMON_HEN_SEAT, 21.0)
        .scaled(1.3)
        .turned(FRAC_PI_2),
];

/// The level ground the buildings stand on: knolls under the buildings by
/// the road, raised 1.3 m over the ground there, so that the walls do not
/// hide their feet, and the gardens before the hobbit holes, level with
/// their doors: their middles, radii and heights (m).
const KNOLLS: [Terrace; 17] = [
    // A hobbit hole's doorstep, cut into the hill.
    garden(261.1, -446.4, 3.0, 8.9),
    // A hobbit hole's doorstep, cut into the hill.
    garden(220.8, -427.7, 3.0, 11.0),
    // A hobbit hole's doorstep, cut into the hill.
    garden(180.3, -414.7, 3.0, 7.1),
    // A hobbit hole's doorstep, cut into the hill.
    garden(139.1, -431.8, 3.0, 9.5),
    // A hobbit hole's doorstep, cut into the hill.
    garden(390.2, -481.7, 3.0, 13.5),
    // A hobbit hole's doorstep, cut into the hill.
    garden(393.4, -454.7, 3.0, 16.1),
    // A hobbit hole's doorstep, cut into the hill.
    garden(409.4, -422.2, 3.0, 12.8),
    // The mill, by the water.
    shelf(103.4, -402.7, 5.5, 1.7),
    // The Green Dragon, by the road, raised well over its fence.
    knoll(341.0, -518.6, 15.0, 8.3),
    // Bag End's doorstep, cut into its hill.
    garden(382.5, -607.3, 3.5, 20.6),
    // A house of Bree.
    knoll(870.3, -448.9, 9.0, 9.1),
    // A house of Bree.
    knoll(819.5, -460.9, 9.0, 10.1),
    // A house of Bree.
    knoll(899.8, -450.4, 9.0, 9.9),
    // The stone trolls.
    knoll(1545.1, -654.1, 8.0, 25.9),
    // Rivendell's halls, one over another: the pavilion to the left of the
    // others, 4 m over the road, the Last Homely House on a ledge 10 m over
    // it, and the tower on a ledge at the edge of the rock behind, 16 m
    // over it, where nothing of the rock hides it from the road.
    knoll(1624.5, 49.0, 6.0, 22.8),
    Terrace {
        center: Vec2::new(1620.0, 110.0),
        radius: 20.0,
        height: 31.0,
        tilt: Vec2::ZERO,
        skirt: 10.0,
    },
    knoll(1577.1, 151.9, 6.0, 38.2),
];

/// A knoll a building stands on (see `KNOLLS`): level, its edge blending
/// back into the land over 8 m.
const fn knoll(x: f32, z: f32, radius: f32, height: f32) -> Terrace {
    Terrace {
        center: Vec2::new(x, z),
        radius,
        height,
        tilt: Vec2::ZERO,
        skirt: 8.0,
    }
}

/// A hobbit hole's doorstep (see `KNOLLS`): level with its door, cut into
/// the hill, its edge blending into it over 8 m, so that it stays grass.
const fn garden(x: f32, z: f32, radius: f32, height: f32) -> Terrace {
    Terrace {
        center: Vec2::new(x, z),
        radius,
        height,
        tilt: Vec2::ZERO,
        skirt: 8.0,
    }
}

/// A garden before a hobbit hole, or the mill's yard (see `KNOLLS`): level
/// with its door, cut into the slope, its edge blending into it over 4 m.
const fn shelf(x: f32, z: f32, radius: f32, height: f32) -> Terrace {
    Terrace {
        center: Vec2::new(x, z),
        radius,
        height,
        tilt: Vec2::ZERO,
        skirt: 4.0,
    }
}

/// A see-through wooden fence on one side of a stretch.
const fn fence(span: Span, side: Side) -> WallRun {
    WallRun {
        span,
        side,
        style: WallStyle::Fence,
    }
}

/// The fields of flowers: in the hobbit holes' gardens; white and yellow
/// through the Shire's fields (not Hobbiton's lawns, kept green),
/// irises round the pond; gold tussock grass up Weathertop and down to the
/// ford, and silver in the Anduin's gorge, as on the hills of Otago where
/// the film was made. And the grass not green everywhere: leaves fallen
/// under the trees of the wood at the start, Rivendell's and Lothlórien's,
/// and the grass worn down to bare earth round Bree, and both round Amon
/// Hen, each in patches with grass between them; and on the shoulders of
/// the roads, fallen leaves or bare earth in patches.
const FLOWERS: [FlowerField; 21] = [
    FlowerField {
        area: FlowerArea::Round {
            center: POND,
            inner: POND_RADIUS - 8.0,
            outer: POND_RADIUS + 12.0,
        },
        kinds: &[FlowerKind::Iris],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: span(HOBBITON.to, BREE.from),
            near: 14.0,
            far: 60.0,
        },
        kinds: &[FlowerKind::White, FlowerKind::Yellow],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: span(BREE.to + 30.0, FORD.to),
            near: 14.0,
            far: 120.0,
        },
        kinds: &[FlowerKind::Susuki],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: GORGE,
            near: 14.0,
            far: 60.0,
        },
        kinds: &[FlowerKind::Susuki],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(261.4, -443.5),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(219.8, -425.4),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(178.4, -411.7),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(138.2, -429.6),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(388.2, -480.6),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(391.2, -453.6),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(406.0, -421.3),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(381.6, -602.7),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: WOOD,
            near: 14.0,
            far: 90.0,
        },
        kinds: &[FlowerKind::FallenLeaves],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: BREE,
            near: 12.0,
            far: 70.0,
        },
        kinds: &[FlowerKind::BareEarth],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: RIVENDELL,
            near: 14.0,
            far: 120.0,
        },
        kinds: &[FlowerKind::FallenLeaves],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: LOTHLORIEN,
            near: 14.0,
            far: 120.0,
        },
        kinds: &[FlowerKind::FallenLeaves],
    },
    FlowerField {
        area: FlowerArea::Road {
            span: AMON_HEN,
            near: 14.0,
            far: 100.0,
        },
        kinds: &[FlowerKind::FallenLeaves, FlowerKind::BareEarth],
    },
    // The shoulders of the roads, between the asphalt and the walls: not
    // all grass, but in patches leaves fallen under the trees of the wood
    // at the start, Rivendell's and Lothlórien's, and earth worn bare
    // everywhere else, with as much grass between them.
    FlowerField {
        area: FlowerArea::Shoulders { span: WOOD },
        kinds: &[FlowerKind::FallenLeaves, FlowerKind::BareEarth],
    },
    FlowerField {
        area: FlowerArea::Shoulders { span: RIVENDELL },
        kinds: &[FlowerKind::FallenLeaves, FlowerKind::BareEarth],
    },
    FlowerField {
        area: FlowerArea::Shoulders { span: LOTHLORIEN },
        kinds: &[FlowerKind::FallenLeaves],
    },
    FlowerField {
        area: FlowerArea::Shoulders { span: LAP },
        kinds: &[FlowerKind::BareEarth],
    },
];

const CONTROL_POINTS: [Vec2; 197] = [
    // 1. The wood by Hobbiton: the start straight, north (-Z), over a crest.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -50.0),
    Vec2::new(0.0, -100.0),
    Vec2::new(0.0, -110.0),
    // Hobbiton: a long sweep to the right down to the Water, over it on
    // an old wooden bridge, a sweep to the left round the village's hill,
    // and one to the right past Bag End's hill, each eased in and out
    // (drawn by `tools/fellowship_dev/hobbiton_v4.py`, as dense as that,
    // so that the spline through them keeps their curvature).
    Vec2::new(0.07, -123.00),
    Vec2::new(0.53, -135.99),
    Vec2::new(1.82, -149.17),
    Vec2::new(4.02, -161.98),
    Vec2::new(7.14, -174.60),
    Vec2::new(11.16, -186.96),
    Vec2::new(16.07, -199.00),
    Vec2::new(21.94, -210.87),
    Vec2::new(28.54, -222.07),
    Vec2::new(35.94, -232.76),
    Vec2::new(44.08, -242.89),
    Vec2::new(52.93, -252.40),
    Vec2::new(62.64, -261.42),
    Vec2::new(72.78, -269.55),
    Vec2::new(83.48, -276.92),
    Vec2::new(94.69, -283.51),
    Vec2::new(106.57, -289.37),
    Vec2::new(118.61, -294.25),
    Vec2::new(130.97, -298.27),
    Vec2::new(143.53, -301.63),
    Vec2::new(156.16, -304.71),
    Vec2::new(169.04, -307.83),
    Vec2::new(181.67, -310.89),
    Vec2::new(194.31, -313.95),
    Vec2::new(206.94, -317.00),
    Vec2::new(219.58, -320.06),
    Vec2::new(232.44, -323.25),
    Vec2::new(244.94, -326.82),
    Vec2::new(257.14, -331.28),
    Vec2::new(268.92, -336.78),
    Vec2::new(280.18, -343.27),
    Vec2::new(291.03, -350.87),
    Vec2::new(300.98, -359.22),
    Vec2::new(310.17, -368.41),
    Vec2::new(318.54, -378.36),
    Vec2::new(326.01, -388.99),
    Vec2::new(332.65, -400.45),
    Vec2::new(338.16, -412.22),
    Vec2::new(342.62, -424.43),
    Vec2::new(346.10, -436.95),
    Vec2::new(349.14, -449.59),
    Vec2::new(352.51, -462.40),
    Vec2::new(356.72, -474.70),
    Vec2::new(361.97, -486.59),
    Vec2::new(368.22, -497.98),
    Vec2::new(375.58, -509.00),
    Vec2::new(383.70, -519.14),
    Vec2::new(392.67, -528.54),
    Vec2::new(402.41, -537.14),
    Vec2::new(412.86, -544.87),
    Vec2::new(424.15, -551.80),
    Vec2::new(435.78, -557.61),
    Vec2::new(447.86, -562.39),
    Vec2::new(460.31, -566.12),
    Vec2::new(473.03, -568.77),
    Vec2::new(486.19, -570.34),
    Vec2::new(499.17, -570.91),
    Vec2::new(512.17, -571.00),
    // 2. The Shire: a sweep to the right and the left over the Brandywine
    // bridge.
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
    // 6. Rivendell: the lap snakes between the ways of its split, as by
    // Ghost Island's temple, turned south, from just past the ford.
    Vec2::new(1576.4, -386.9),
    Vec2::new(1585.5, -345.7),
    Vec2::new(1600.3, -304.4),
    Vec2::new(1598.5, -263.1),
    Vec2::new(1576.4, -221.9),
    Vec2::new(1554.3, -180.7),
    Vec2::new(1552.5, -139.4),
    Vec2::new(1567.3, -98.1),
    Vec2::new(1576.4, -56.9),
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

/// Up Weathertop, 42.7 m, and down to the ford; up Caradhras, 32 m, and down
/// round it to the mines, 38 m under the pass; up through Moria and down
/// through Lothlórien to the Anduin, at the water's level between the kings;
/// over Amon Hen and down to the line. Up and down at up to 12 %.
const ELEVATION: [(f32, f32); 85] = [
    // The start straight through the wood, high over Hobbiton, over a
    // lower crest where the bike flies; down the long sweep into Hobbiton
    // at up to 9.4 % (drawn as a grade, its changes long and gentle) to the
    // bridge low over the wide Water, low round the sweep the other way,
    // and up round the last past Bag End's hill to the Brandywine.
    (0.0, 13.5),
    (50.0, 14.2),
    (95.0, 17.2),
    (140.0, 16.2),
    (170.0, 15.25),
    (200.0, 13.71),
    (230.0, 11.58),
    (260.0, 8.90),
    (290.0, 6.09),
    (320.0, 3.28),
    (350.0, 1.07),
    (380.0, 0.06),
    (410.0, 0.0),
    (460.0, 0.0),
    (520.0, 0.3),
    (580.0, 0.8),
    (640.0, 1.6),
    (720.0, 3.0),
    (800.0, 4.4),
    (860.0, 5.4),
    (925.1, 6.0),
    // Bree, and up Weathertop at up to 12 %: at 7.5 % round the turn that
    // Bree's way cuts across, beside it, and at 10.5 % where it joins it;
    // steeper before and after (drawn as a grade, its changes long and
    // gentle, so that no crest in a bend throws the bikes into the air).
    (1145.1, 6.50),
    (1299.1, 7.00),
    (1314.1, 7.52),
    (1329.1, 8.41),
    (1344.1, 9.46),
    (1359.1, 10.58),
    (1374.1, 11.71),
    (1389.1, 12.83),
    (1404.1, 13.96),
    (1419.1, 15.08),
    (1434.1, 16.21),
    (1449.1, 17.37),
    (1464.1, 18.75),
    (1479.1, 20.30),
    (1494.1, 21.88),
    (1509.1, 23.45),
    (1524.1, 25.03),
    (1539.1, 26.63),
    (1554.1, 28.39),
    (1569.1, 30.19),
    (1584.1, 31.99),
    (1599.1, 33.79),
    (1614.1, 35.59),
    (1629.1, 37.39),
    (1644.1, 39.18),
    (1659.1, 40.83),
    (1674.1, 42.30),
    // Over its top: a crest where the bike flies.
    (1689.1, 43.57),
    (1704.1, 44.74),
    (1773.1, 49.70),
    (1808.1, 47.80),
    (1863.1, 44.70),
    // Down through the Trollshaws at up to 12 %, to the ford, low over the
    // water.
    (1959.1, 38.0),
    (2049.1, 27.5),
    (2139.1, 17.0),
    (2229.1, 7.0),
    (2299.1, 3.0),
    (2339.1, 0.5),
    // Climbing steadily through Rivendell's split (as Ghost Island's
    // temple's).
    (2364.1, 1.1),
    (2714.1, 13.1),
    // Up Caradhras at up to 12 %, to the pass right over the mines.
    (2899.1, 21.0),
    (2999.1, 30.0),
    (3099.1, 39.0),
    (3159.1, 45.5),
    (3199.1, 47.5),
    // Down round the mountain to the Doors of Durin; through Moria, down
    // into it and up again.
    (3259.1, 45.5),
    (3419.1, 36.0),
    (3579.1, 24.0),
    (3761.1, 15.0),
    (3821.1, 12.0),
    (3881.1, 9.0),
    (4053.1, 20.0),
    (4133.1, 24.0),
    // Down steadily through Lothlórien's split (as Ghost Island's castle's).
    (4187.1, 23.5),
    (4537.1, 11.5),
    // Down the Anduin's gorge to the water, and the causeway, low over it.
    (4617.1, 9.5),
    (4887.1, 1.5),
    (4957.1, -1.4),
    (5211.1, -1.4),
    // Up over the shoulder of Amon Hen, and on up into the wood, to the
    // line.
    (5271.1, 1.0),
    (5371.1, 6.5),
    (5461.1, 10.5),
    (5531.1, 11.3),
    (5689.1, 13.0),
];

/// The ways off the lap, each through points to one side of it (m along
/// the lap and from its centre line in the comments), from where it leaves
/// the lap to where it joins it again: Ghost Island's, turned and placed.
/// In Bree, to the left: an S out to the left, 52 m from the lap, as the lap
/// bends right, a straight through the Prancing Pony's archway (45 m, the
/// inn in the middle of it), and straight on past the inn, across the
/// inside of the lap's turn up Weathertop's flank, in one long bend to the
/// left (R 90) onto the lap, running beside it for its last metres: a
/// short cut, 35 m shorter than the lap (drawn by integrating its
/// curvature, as Hobbiton's line). It climbs with the lap up Weathertop's
/// flank, which is gentler round the turn it cuts across (see
/// `ELEVATION`), so that beside the lap, as steep as the lap is there
/// seen from the inside of its turn, it stays under 11 %.
const BREE_WAY: [Vec2; 31] = [
    // 1110 m, 1 m.
    Vec2::new(750.01, -531.87),
    // 1120 m, 4 m.
    Vec2::new(762.24, -531.54),
    // 1128 m, 12 m.
    Vec2::new(773.95, -534.02),
    // 1136 m, 21 m.
    Vec2::new(785.39, -538.39),
    // 1142 m, 30 m.
    Vec2::new(797.04, -541.12),
    // 1150 m, 37 m.
    Vec2::new(809.25, -540.67),
    // 1160 m, 42 m.
    Vec2::new(820.88, -537.75),
    // 1174 m, 47 m.
    Vec2::new(832.55, -534.04),
    // 1190 m, 51 m.
    Vec2::new(843.98, -530.38),
    // 1222 m, 52 m.
    Vec2::new(855.65, -526.65),
    // 1238 m, 50 m.
    Vec2::new(867.08, -522.99),
    // 1254 m, 47 m.
    Vec2::new(878.98, -519.18),
    // 1266 m, 43 m.
    Vec2::new(890.64, -515.45),
    // 1278 m, 39 m.
    Vec2::new(902.29, -511.71),
    // 1292 m, 35 m.
    Vec2::new(913.98, -508.06),
    // 1304 m, 32 m.
    Vec2::new(925.88, -505.18),
    // 1318 m, 29 m.
    Vec2::new(938.04, -503.83),
    // 1334 m, 27 m.
    Vec2::new(950.27, -504.15),
    // 1350 m, 24 m.
    Vec2::new(962.34, -506.13),
    // 1366 m, 21 m.
    Vec2::new(974.04, -509.72),
    // 1380 m, 18 m.
    Vec2::new(985.14, -514.87),
    // 1394 m, 16 m.
    Vec2::new(995.43, -521.47),
    // 1408 m, 13 m.
    Vec2::new(1004.74, -529.41),
    // 1422 m, 11 m.
    Vec2::new(1012.89, -538.54),
    // 1434 m, 8 m.
    Vec2::new(1019.72, -548.69),
    // 1448 m, 6 m.
    Vec2::new(1025.11, -559.67),
    // 1460 m, 5 m.
    Vec2::new(1028.96, -571.28),
    // 1472 m, 3 m.
    Vec2::new(1031.21, -583.31),
    // 1484 m, 1 m.
    Vec2::new(1031.84, -595.53),
    // 1498 m, 1 m.
    Vec2::new(1031.45, -607.77),
    // 1510 m, 1 m.
    Vec2::new(1030.88, -620.01),
];

/// In Rivendell, to the left (east): it swings out to the left before the
/// lap does, and back to it late.
const RIVENDELL_LEFT: [Vec2; 11] = [
    // 2374 m, 0 m.
    Vec2::new(1578.2, -377.0),
    // 2404 m, 4 m.
    Vec2::new(1589.0, -348.4),
    // 2442 m, 13 m.
    Vec2::new(1611.6, -315.5),
    // 2472 m, 32 m.
    Vec2::new(1633.5, -281.3),
    // 2492 m, 48 m.
    Vec2::new(1644.7, -248.3),
    // 2512 m, 60 m.
    Vec2::new(1642.1, -215.3),
    // 2546 m, 65 m.
    Vec2::new(1628.1, -182.3),
    // 2624 m, 57 m.
    Vec2::new(1608.7, -149.4),
    // 2658 m, 29 m.
    Vec2::new(1589.4, -121.7),
    // 2678 m, 11 m.
    Vec2::new(1579.4, -94.7),
    // 2700 m, 2 m.
    Vec2::new(1575.9, -70.3),
];

/// In Lothlórien, to the left (south): it runs straight on where the lap
/// swings right, and out round a wide curve back to it.
const LORIEN_LEFT: [Vec2; 11] = [
    // 4199 m, 1 m.
    Vec2::new(1246.2, 397.9),
    // 4227 m, 15 m.
    Vec2::new(1215.0, 404.0),
    // 4249 m, 40 m.
    Vec2::new(1183.0, 418.9),
    // 4311 m, 65 m.
    Vec2::new(1152.3, 438.9),
    // 4371 m, 64 m.
    Vec2::new(1119.3, 457.3),
    // 4399 m, 57 m.
    Vec2::new(1086.3, 467.5),
    // 4415 m, 44 m.
    Vec2::new(1053.3, 464.5),
    // 4439 m, 25 m.
    Vec2::new(1020.3, 448.1),
    // 4471 m, 9 m.
    Vec2::new(986.6, 424.2),
    // 4501 m, 3 m.
    Vec2::new(961.8, 408.1),
    // 4523 m, 0 m.
    Vec2::new(941.5, 400.6),
];

/// In Lothlórien, to the right (north): it swings out to the right before
/// the lap does, and back to it late.
const LORIEN_RIGHT: [Vec2; 11] = [
    // 4199 m, -0 m.
    Vec2::new(1246.6, 396.1),
    // 4229 m, -4 m.
    Vec2::new(1218.5, 385.2),
    // 4267 m, -14 m.
    Vec2::new(1186.2, 362.8),
    // 4295 m, -32 m.
    Vec2::new(1152.3, 341.0),
    // 4317 m, -48 m.
    Vec2::new(1119.3, 329.8),
    // 4335 m, -60 m.
    Vec2::new(1086.3, 332.4),
    // 4369 m, -65 m.
    Vec2::new(1053.3, 346.4),
    // 4447 m, -57 m.
    Vec2::new(1020.4, 365.8),
    // 4481 m, -29 m.
    Vec2::new(992.2, 384.9),
    // 4503 m, -11 m.
    Vec2::new(964.6, 394.8),
    // 4523 m, -2 m.
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

/// The footpaths of Hobbiton, of bare earth: from the road past the bridge
/// up the hill on the left, past the gates of its holes, low and high,
/// and down to the road again, and from the low one along the water to
/// the mill; from the road up past the gates of the holes on the right;
/// and from the party tree past Bag End's gate and down round the Green
/// Dragon, past its front, to the road by its sign.
const HOBBITON_PATH_A: [Vec2; 52] = [
    Vec2::new(301.2, -384.0),
    Vec2::new(298.9, -387.2),
    Vec2::new(296.6, -390.5),
    Vec2::new(294.2, -393.8),
    Vec2::new(291.9, -397.1),
    Vec2::new(289.6, -400.4),
    Vec2::new(287.3, -403.6),
    Vec2::new(284.9, -406.9),
    Vec2::new(282.6, -410.2),
    Vec2::new(280.3, -413.5),
    Vec2::new(278.0, -416.8),
    Vec2::new(275.6, -420.0),
    Vec2::new(273.3, -423.3),
    Vec2::new(271.0, -426.6),
    Vec2::new(268.7, -429.9),
    Vec2::new(266.3, -433.1),
    Vec2::new(263.9, -435.8),
    Vec2::new(261.1, -437.5),
    Vec2::new(257.9, -437.7),
    Vec2::new(254.4, -436.7),
    Vec2::new(250.8, -435.3),
    Vec2::new(247.2, -433.8),
    Vec2::new(243.6, -432.3),
    Vec2::new(240.0, -430.9),
    Vec2::new(236.4, -429.4),
    Vec2::new(232.7, -427.9),
    Vec2::new(229.1, -426.5),
    Vec2::new(225.5, -425.0),
    Vec2::new(221.8, -423.5),
    Vec2::new(218.2, -422.1),
    Vec2::new(214.4, -420.8),
    Vec2::new(210.6, -419.6),
    Vec2::new(206.8, -418.3),
    Vec2::new(203.0, -417.1),
    Vec2::new(199.2, -415.9),
    Vec2::new(195.4, -414.7),
    Vec2::new(191.6, -413.4),
    Vec2::new(187.8, -412.2),
    Vec2::new(184.0, -411.0),
    Vec2::new(180.2, -410.1),
    Vec2::new(176.5, -409.8),
    Vec2::new(172.8, -410.5),
    Vec2::new(169.2, -411.8),
    Vec2::new(165.6, -413.3),
    Vec2::new(162.0, -414.9),
    Vec2::new(158.4, -416.5),
    Vec2::new(154.8, -418.1),
    Vec2::new(151.2, -419.7),
    Vec2::new(147.6, -421.3),
    Vec2::new(144.0, -422.9),
    Vec2::new(140.4, -424.6),
    Vec2::new(136.8, -426.2),
];
const HOBBITON_PATH_MILL: [Vec2; 13] = [
    Vec2::new(136.8, -426.2),
    Vec2::new(133.8, -423.5),
    Vec2::new(130.7, -420.9),
    Vec2::new(127.7, -418.2),
    Vec2::new(124.7, -415.6),
    Vec2::new(121.7, -413.0),
    Vec2::new(118.7, -410.3),
    Vec2::new(115.7, -407.7),
    Vec2::new(112.7, -405.0),
    Vec2::new(109.7, -402.4),
    Vec2::new(106.7, -399.7),
    Vec2::new(103.7, -397.1),
    Vec2::new(100.7, -394.5),
];
const HOBBITON_PATH_B: [Vec2; 31] = [
    Vec2::new(351.2, -399.6),
    Vec2::new(354.9, -401.1),
    Vec2::new(358.5, -402.6),
    Vec2::new(362.2, -404.1),
    Vec2::new(365.8, -405.6),
    Vec2::new(369.5, -407.0),
    Vec2::new(373.1, -408.5),
    Vec2::new(376.8, -410.0),
    Vec2::new(380.4, -411.5),
    Vec2::new(384.1, -412.9),
    Vec2::new(387.7, -414.4),
    Vec2::new(391.4, -415.9),
    Vec2::new(395.0, -417.4),
    Vec2::new(398.0, -419.1),
    Vec2::new(399.9, -421.3),
    Vec2::new(400.1, -424.1),
    Vec2::new(399.0, -427.4),
    Vec2::new(397.5, -430.9),
    Vec2::new(395.9, -434.4),
    Vec2::new(394.3, -437.9),
    Vec2::new(392.7, -441.4),
    Vec2::new(391.1, -445.0),
    Vec2::new(389.6, -448.5),
    Vec2::new(388.4, -452.1),
    Vec2::new(387.5, -455.9),
    Vec2::new(387.0, -459.6),
    Vec2::new(386.6, -463.5),
    Vec2::new(386.1, -467.3),
    Vec2::new(385.7, -471.1),
    Vec2::new(385.3, -474.9),
    Vec2::new(384.9, -478.7),
];
const BAG_END_PATH: [Vec2; 62] = [
    Vec2::new(442.4, -594.3),
    Vec2::new(438.5, -595.5),
    Vec2::new(434.7, -596.6),
    Vec2::new(430.8, -597.8),
    Vec2::new(426.9, -598.8),
    Vec2::new(423.1, -599.9),
    Vec2::new(419.2, -600.7),
    Vec2::new(415.4, -601.4),
    Vec2::new(411.5, -601.9),
    Vec2::new(407.7, -602.1),
    Vec2::new(403.9, -602.1),
    Vec2::new(400.1, -601.9),
    Vec2::new(396.4, -601.4),
    Vec2::new(392.8, -600.8),
    Vec2::new(389.2, -599.7),
    Vec2::new(385.7, -598.0),
    Vec2::new(382.3, -595.5),
    Vec2::new(379.3, -592.4),
    Vec2::new(377.0, -589.0),
    Vec2::new(375.1, -585.6),
    Vec2::new(373.4, -582.2),
    Vec2::new(371.8, -578.7),
    Vec2::new(370.3, -575.1),
    Vec2::new(368.7, -571.4),
    Vec2::new(367.0, -567.7),
    Vec2::new(365.4, -564.0),
    Vec2::new(363.9, -560.2),
    Vec2::new(362.4, -556.4),
    Vec2::new(360.8, -552.8),
    Vec2::new(358.8, -549.6),
    Vec2::new(356.2, -546.9),
    Vec2::new(353.1, -544.7),
    Vec2::new(349.8, -542.7),
    Vec2::new(346.4, -540.7),
    Vec2::new(343.1, -538.7),
    Vec2::new(339.7, -536.7),
    Vec2::new(336.4, -534.7),
    Vec2::new(333.0, -532.7),
    Vec2::new(329.7, -530.7),
    Vec2::new(326.3, -528.7),
    Vec2::new(323.0, -526.6),
    Vec2::new(320.1, -524.4),
    Vec2::new(318.2, -521.8),
    Vec2::new(317.5, -518.7),
    Vec2::new(318.1, -515.5),
    Vec2::new(319.9, -512.7),
    Vec2::new(322.7, -510.8),
    Vec2::new(326.2, -509.6),
    Vec2::new(329.9, -508.9),
    Vec2::new(333.4, -508.5),
    Vec2::new(336.9, -508.3),
    Vec2::new(340.4, -508.0),
    Vec2::new(343.6, -507.4),
    Vec2::new(345.7, -506.0),
    Vec2::new(346.2, -503.4),
    Vec2::new(345.5, -500.2),
    Vec2::new(344.3, -496.5),
    Vec2::new(343.0, -492.9),
    Vec2::new(341.7, -489.2),
    Vec2::new(340.4, -485.6),
    Vec2::new(339.1, -481.9),
    Vec2::new(337.8, -478.3),
];
const PATHS: [&[Vec2]; 4] = [
    &HOBBITON_PATH_A,
    &HOBBITON_PATH_MILL,
    &HOBBITON_PATH_B,
    &BAG_END_PATH,
];

/// The pool of Bywater, below the right of the village, where the Water
/// widens before it runs on into the Brandywine: its middle and radius
/// (m).
const POND: Vec2 = Vec2::new(470.0, -330.0);
const POND_RADIUS: f32 = 30.0;
/// The middles of the hills of Hobbiton, either side of the road past the
/// bridge: inside the sweep to the left, and inside the one to the right,
/// over the pool; the hobbit holes are dug into their flanks at heights
/// that alternate, the hills rising on behind them.
const HOBBITON_HILLS: [Vec2; 2] = [Vec2::new(184.7, -517.5), Vec2::new(440.0, -470.0)];
/// The Water, the stream through Hobbiton, 64 m wide: out of the wood,
/// along the left of the long sweep down into the village, between the
/// road and the mill; square under the bridge, along the right of the
/// sweep on, and away from the road into the pool, and on out of it into
/// the Brandywine.
const THE_WATER: [(Vec2, f32); 23] = [
    (Vec2::new(-150.0, 40.0), 8.0),
    (Vec2::new(-115.0, -60.0), 9.0),
    (Vec2::new(-75.0, -110.0), 12.0),
    (Vec2::new(-59.7, -132.0), 16.0),
    (Vec2::new(-53.6, -179.1), 22.0),
    (Vec2::new(-39.3, -222.1), 28.0),
    (Vec2::new(-12.6, -268.1), 32.0),
    (Vec2::new(23.7, -307.0), 32.0),
    (Vec2::new(67.6, -337.1), 32.0),
    (Vec2::new(111.6, -355.1), 32.0),
    (Vec2::new(138.2, -362.1), 32.0),
    (Vec2::new(154.0, -364.6), 32.0),
    (Vec2::new(168.6, -346.7), 32.0),
    (Vec2::new(187.2, -312.2), 32.0),
    (Vec2::new(205.8, -277.8), 32.0),
    (Vec2::new(220.5, -259.8), 32.0),
    (Vec2::new(234.5, -261.9), 32.0),
    (Vec2::new(256.1, -267.5), 32.0),
    (Vec2::new(289.7, -280.1), 32.0),
    (Vec2::new(440.0, -345.0), 32.0),
    (Vec2::new(470.0, -330.0), 30.0),
    (Vec2::new(505.0, -332.0), 16.0),
    (Vec2::new(548.0, -330.0), 18.0),
];
/// Rivers, as points (x, z) along them with how far their banks are from
/// the middle there (m): the Brandywine, north to south under its bridge
/// into a pool; the Bruinen, from the mountains west across the ford into
/// a pool; and the Anduin, from the south into the middle of the road,
/// widening down the causeway to the kings, narrowing past them through the
/// gorge in the rock they guard, and turning into the lake.
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
const ANDUIN: [(Vec2, f32); 13] = [
    (Vec2::new(680.0, 800.0), 18.0),
    (Vec2::new(630.0, 600.0), 18.0),
    (Vec2::new(598.0, 470.0), 14.0),
    (Vec2::new(578.0, 398.0), 12.0),
    (Vec2::new(545.0, 396.5), 45.0),
    (Vec2::new(495.0, 389.5), 55.0),
    // Between the kings, where the cliffs close in on it, and past them
    // through the gorge in the rock.
    (Vec2::new(434.0, 372.5), 35.0),
    (Vec2::new(405.0, 362.0), 31.0),
    (Vec2::new(351.0, 350.0), 29.0),
    (Vec2::new(312.0, 360.0), 28.0),
    (Vec2::new(282.0, 405.0), 30.0),
    (Vec2::new(228.0, 452.0), 30.0),
    (Vec2::new(205.0, 478.0), 30.0),
];
/// Mirrormere, below Moria's gate, and Nen Hithoel, the lake the Anduin
/// opens into past the kings: their middles and radii (m).
const MIRRORMERE: Vec2 = Vec2::new(1350.0, 505.0);
const MIRRORMERE_RADIUS: f32 = 45.0;
const NEN_HITHOEL: Vec2 = Vec2::new(180.0, 500.0);
const NEN_HITHOEL_RADIUS: f32 = 85.0;
/// Where the two kings of the Argonath stand, either side of the causeway,
/// close together as in the film: as far apart as two thirds of their
/// height, at the foot of the cliffs. And which way they face: up the
/// river, back along the causeway at the riders coming down it.
const KINGS: [Vec2; 2] = [Vec2::new(426.5, 394.4), Vec2::new(441.0, 350.7)];
const KINGS_FACING: f32 = -1.89;

/// How far `point` is from the shores of the rivers and lakes: negative in
/// their water. Their shores wander a little.
fn water(point: Vec2) -> f32 {
    let wander = 4.0 * (2.0 * fbm(point / 40.0 + 7.0, 2) - 1.0);
    let pond = point.distance(POND) - POND_RADIUS;
    let mirrormere = point.distance(MIRRORMERE) - MIRRORMERE_RADIUS;
    let lake = point.distance(NEN_HITHOEL) - NEN_HITHOEL_RADIUS;
    let rivers = [&THE_WATER[..], &BRANDYWINE_RIVER, &BRUINEN_RIVER, &ANDUIN]
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

const MASSIFS: [Massif; 32] = [
    WEATHERTOP_HILL,
    CARADHRAS_PEAK,
    // Bag End's hill, on the outside of the last of Hobbiton's sweeps, and
    // Amon Hen, on the outside of the last bend.
    climbed_massif(320.6, -711.3, 170.0, 32.0),
    climbed_massif(40.0, 400.0, 150.0, 20.0),
    // The hills of Hobbiton, either side of the road past the bridge, rising
    // behind the hobbit holes dug into their flanks.
    climbed_massif(HOBBITON_HILLS[0].x, HOBBITON_HILLS[0].y, 120.0, 28.0),
    climbed_massif(HOBBITON_HILLS[1].x, HOBBITON_HILLS[1].y, 80.0, 20.0),
    // Rolling hills: beside the Shire's lanes; west of Hobbiton; in the
    // middle of the lap, beyond the Brandywine and towards the Misty
    // Mountains; and south of the Anduin.
    climbed_massif(230.0, -735.0, 150.0, 30.0),
    climbed_massif(-110.0, -600.0, 150.0, 22.0),
    climbed_massif(800.0, -160.0, 200.0, 40.0),
    climbed_massif(1150.0, 140.0, 160.0, 34.0),
    climbed_massif(-320.0, -330.0, 150.0, 30.0),
    climbed_massif(-300.0, 120.0, 140.0, 26.0),
    climbed_massif(440.0, 660.0, 150.0, 30.0),
    climbed_massif(330.0, 120.0, 130.0, 24.0),
    // The cliffs on the far side of Rivendell's valley, and the rock at its
    // head its halls stand on, one over another.
    massif(1735.0, -200.0, 80.0, 40.0),
    massif(1745.0, -90.0, 70.0, 36.0),
    massif(1650.0, 180.0, 100.0, 60.0),
    // The mountain of rock the kings of the Argonath guard: its cliffs close
    // in on the river at the kings, rising sheer from the water just past
    // them, and run on either side of the gorge past them; either side of
    // the kings, as the riders coming down to them see it, higher still,
    // up the Anduin's west bank and over the cutting on the right of the
    // road, so that the river goes through a range and not between two
    // hills; and the rocks the kings stand on in the river.
    massif(455.0, 505.0, 58.0, 100.0),
    massif(505.0, 548.0, 56.0, 110.0),
    massif(548.0, 600.0, 50.0, 104.0),
    massif(515.0, 262.0, 56.0, 104.0),
    massif(568.0, 236.0, 56.0, 114.0),
    massif(622.0, 250.0, 50.0, 100.0),
    massif(411.0, 447.0, 50.0, 85.0),
    massif(376.0, 438.0, 52.0, 92.0),
    massif(346.0, 431.0, 50.0, 78.0),
    massif(460.0, 299.0, 50.0, 85.0),
    massif(419.0, 284.0, 52.0, 92.0),
    massif(374.0, 273.0, 52.0, 82.0),
    massif(325.0, 266.0, 50.0, 70.0),
    massif(KINGS[0].x, KINGS[0].y, 14.0, 5.0),
    massif(KINGS[1].x, KINGS[1].y, 14.0, 5.0),
];

const CUTTINGS: [Cutting; 6] = [
    // The cliffs on the right of Rivendell's valley, down which its
    // waterfalls pour.
    Cutting {
        span: span(FORD.to, 2835.1),
        side: Side::Right,
        height: 32.0,
    },
    // The Walls of Moria, sheer and higher than the doors in them, closing
    // in on the way to them; and the rock round its east gate.
    Cutting {
        span: span(MORIA.from - 30.0, MORIA.from + 90.0),
        side: Side::Left,
        height: 54.0,
    },
    Cutting {
        span: span(MORIA.from - 30.0, MORIA.from + 90.0),
        side: Side::Right,
        height: 54.0,
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
    const WEATHERTOP: Span = span(1299.1, TROLLSHAWS.from);
    const SUMMIT: Span = span(1683.1, 1863.1);

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
        let mut wet = vec![0.0; FELLOWSHIP_ROAD.scenery.bridges.len()];
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
            assert!((20.0..26.0).contains(&king.distance(road)));
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
