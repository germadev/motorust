//! Fellowship Road: the journey of *The Fellowship of the Ring*, its
//! stretches in the order of the film, from the Shire to the Argonath and,
//! past the finish, home again (*There and Back Again*). It crosses the
//! Misty Mountains twice, in a figure of eight: first over them, by the pass
//! of Caradhras, and then, as the Fellowship turned back from the snow,
//! through them, by the mines of Moria, under the pass.
//!
//! 1. Hobbiton: the start straight, north through the wood, over a crest,
//!    a wide sweep to the right down to the Water and over it, a sweep to
//!    the left, and a long one to the right round the pool, past Bag End's
//!    hill.
//! 2. The Shire: a straight east over a crest where the bike takes off,
//!    and a sweep to the right and the left over the bridge on the
//!    Brandywine.
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
const WOOD: Span = span(5664.0, 170.0);
const HOBBITON: Span = span(WOOD.to, 760.0);
const HOBBITON_BRIDGE: Span = span(354.0, 406.0);
const SHIRE: Span = span(HOBBITON.to, BREE.from);
const BRANDYWINE: Span = span(904.0, 964.0);
const BREE: Span = span(1104.0, 1318.0);
const TROLLSHAWS: Span = span(1978.0, FORD.from);
/// The Ford of Bruinen: low over the river.
const FORD: Span = span(2319.0, 2379.0);
const RIVENDELL: Span = span(FORD.to, CARADHRAS.from);
const CARADHRAS: Span = span(2838.0, 3238.0);
/// The top of the pass, right over the mines.
const PASS: Span = span(3174.0, 3238.0);
const MORIA: Span = span(3809.0, 4152.0);
const LOTHLORIEN: Span = span(MORIA.to, GORGE.from);
const GORGE: Span = span(4606.0, CAUSEWAY.from);
/// Down the middle of the Anduin, between the kings of the Argonath: a
/// causeway narrower than the road, its walls this far from the middle
/// (m).
const CAUSEWAY: Span = span(4919.0, 5254.0);
const CAUSEWAY_WALL: f32 = 6.5;
/// How far from the middle the walls stand through the Doors of Durin (m),
/// just inside their leaves.
const DURIN_WALL: f32 = 5.5;
const AMON_HEN: Span = span(CAUSEWAY.to, WOOD.from);
/// Where the road splits: in two in Rivendell, three ways in Lothlórien.
const RIVENDELL_SPLIT: (f32, f32) = (2438.0, 2788.0);
const LORIEN_SPLIT: (f32, f32) = (4206.0, 4556.0);

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
        // The way through the Doors of Durin, between their leaves, and the
        // causeway between the kings are narrower than the road: its walls
        // close in on them.
        narrows: &[
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
        greens: Greens::Countryside,
        grass_shade: 0.8,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &KNOLLS,
        mountains,
        curbs: [GREEN, CREAM],
        // Wooden fences through the wood the race starts in, along the lanes
        // of Hobbiton and the street of Bree, and where the small props stand
        // by the road, so that they show whole (a wall would hide them);
        // hedgerows through the Shire, and stone walls everywhere else.
        walls: &[
            fence(span(WOOD.from, HOBBITON.to), Side::Left),
            fence(span(WOOD.from, HOBBITON.to), Side::Right),
            fence(span(793.0, 807.0), Side::Left),
            fence(span(873.0, 887.0), Side::Right),
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
            fence(span(1776.0, 1792.0), Side::Left),
            fence(span(1816.0, 1832.0), Side::Right),
            fence(span(2387.0, 2401.0), Side::Left),
            fence(span(2387.0, 2401.0), Side::Right),
            fence(span(2807.0, 2821.0), Side::Left),
            fence(span(2807.0, 2821.0), Side::Right),
            fence(span(4162.0, 4176.0), Side::Left),
            fence(span(4162.0, 4176.0), Side::Right),
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
                along: 790.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1924.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2974.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3604.0,
                color: GREEN,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4584.0,
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
            forest(AMON_HEN, &[Species::Broadleaf, Species::Poplar]),
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
            spans: &[span(5704.0, 300.0), BREE],
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
                along: 2438.0,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2792.0,
                side: Side::Right,
                width: 8.0,
            },
            Waterfall {
                along: 2830.0,
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
        japan: None,
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
            // stay grass, as the Shire's.
            turf: &[Turf {
                center: Vec2::new(180.0, -480.0),
                radius: 230.0,
            }],
        }),
    },
};

/// The halls of Moria, through the Doors of Durin: the way in, of the
/// dwarves' masonry, as high as the doors; a natural cave under the pass; the
/// Dwarrowdelf, a great hall of pillars far wider than the road, its roof
/// lost high above; and a natural cave out to the east gate.
const MORIA_HALLS: [Hall; 4] = [
    Hall {
        span: span(MORIA.from - 20.0, 3854.0),
        stone: Stone::Masonry,
        crown: 30.0,
        width: 0.0,
    },
    Hall {
        span: span(3854.0, 3919.0),
        stone: Stone::Cave,
        crown: 22.0,
        width: 5.0,
    },
    Hall {
        span: span(3919.0, 4074.0),
        stone: Stone::Masonry,
        crown: 34.0,
        width: 16.0,
    },
    Hall {
        span: span(4074.0, MORIA.to + 20.0),
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
    // Hobbiton, down the sweep to the Water and round the pool: across the
    // water on the left, hobbit holes in their banks high at the foot of
    // their hill, ahead down the straight and the sweep, and the mill by
    // the water before the bridge; over it, ahead and then on the right,
    // across the water, more holes at the foot of the other hill and the
    // Green Dragon over the pool, its sign by the road; and on the outside
    // of the long sweep round the pool, the party tree in its field and,
    // up on its hill, Bag End under its oak, ahead all the way round. All
    // of them face the riders coming.
    by_road(MiddleEarthModel::HobbitDoorBlue, 410.0, 40.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorRed, 400.0, 60.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorYellow, 390.0, 80.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorGreen, 380.0, 100.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::Mill, 350.0, 70.0).facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorGreen, 600.0, -116.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorRed, 580.0, -136.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::HobbitDoorYellow, 560.0, -156.0)
        .scaled(1.4)
        .facing_riders(),
    by_road(MiddleEarthModel::GreenDragon, 620.0, -96.0).facing_riders(),
    by_road(MiddleEarthModel::GreenDragonSign, 520.0, -13.5).turned(0.79),
    by_road(MiddleEarthModel::PartyTree, 640.0, 42.0),
    by_road(MiddleEarthModel::BagEnd, 700.0, 62.0).facing_riders(),
    by_road(MiddleEarthModel::BagEndOak, 708.0, 76.0),
    // The Shire: the old tree the hobbits hid under on the outside of the
    // last of Hobbiton's sweeps, a scarecrow in the fields, and a pile of
    // logs by the road.
    by_road(MiddleEarthModel::RootTree, 735.0, 20.0),
    by_road(MiddleEarthModel::Scarecrow, 800.0, 14.0).turned(-0.79),
    by_road(MiddleEarthModel::LogPile, 880.0, -13.5),
    standing(MiddleEarthModel::Ferry, 562.0, -626.0, -FRAC_PI_2),
    // Bree: its gate over the road, the Prancing Pony on the outside of the
    // first bend, ahead through the gate, and its houses either side of the
    // street, tall and narrow ones and wide ones.
    over_road(MiddleEarthModel::BreeGate, 1110.0),
    by_road(MiddleEarthModel::PrancingPony, 1159.0, 33.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1124.0, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1136.0, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1179.0, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1191.0, -16.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1202.0, 24.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1222.0, -22.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1262.0, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1274.0, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseA, 1294.0, -22.0).turned(0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1286.0, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1310.0, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::BreeHouseB, 1324.0, -16.0).turned(0.79),
    // Weathertop: the ring of ruins of Amon Sûl beside the top straight, on
    // the side of the summit, and broken arches either side of the road.
    by_road(MiddleEarthModel::WeathertopRuin, 1832.0, -38.0).turned(0.79),
    by_road(MiddleEarthModel::RuinArch, 1784.0, 16.0).turned(-0.79),
    by_road(MiddleEarthModel::RuinArch, 1824.0, -16.0).turned(0.79),
    // The Trollshaws: the trolls turned to stone on the outside of the
    // esses.
    by_road(MiddleEarthModel::StoneTrolls, 2114.0, 21.0).turned(-0.79),
    // Rivendell: lamps along its paths, and at the head of the valley,
    // ahead down it on the outside of the bend, its halls together one over
    // another up the rock: the domed pavilion by the road, the Last Homely
    // House on a ledge over it, and a tower on a crag over both.
    by_road(MiddleEarthModel::ElvenLamp, 2394.0, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2394.0, -13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2814.0, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 2814.0, -13.5),
    by_road(MiddleEarthModel::Gazebo, 2850.0, 26.0).facing_riders(),
    by_road(MiddleEarthModel::LastHomelyHouse, 2876.0, 58.0).facing_riders(),
    by_road(MiddleEarthModel::ElvenTower, 2904.0, 70.0)
        .scaled(1.25)
        .facing_riders(),
    // The Doors of Durin, open, in the sheer Walls of Moria: the road goes
    // through them into the mountain, ahead of the riders coming round the
    // last bend before it.
    over_road(MiddleEarthModel::DurinDoors, MORIA.from).turned(PI),
    // Lothlórien, out of Moria's east gate: elven lamps and a mallorn with
    // its flet.
    by_road(MiddleEarthModel::ElvenLamp, 4169.0, 13.5),
    by_road(MiddleEarthModel::ElvenLamp, 4169.0, -13.5),
    by_road(MiddleEarthModel::MallornFlet, 4176.0, 40.0),
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
    by_road(MiddleEarthModel::ElvenShip, 5169.0, 24.0).turned(PI),
    // Amon Hen: the elven boats drawn up on the shore of Parth Galen, and
    // the seat of seeing on the hill on the outside of the last bend.
    standing(MiddleEarthModel::ElvenBoat, 238.0, 393.0, 2.20),
    standing(MiddleEarthModel::ElvenBoat, 226.0, 404.0, 2.00),
    by_road(MiddleEarthModel::AmonHenSeat, 5424.0, 32.0).turned(-0.79),
];

/// The level ground the buildings stand on: knolls under the buildings by
/// the road, raised 1.3 m over the ground there, so that the walls do not
/// hide their feet, and the gardens before the hobbit holes, level with
/// their doors: their middles, radii and heights (m).
const KNOLLS: [Terrace; 18] = [
    // A hobbit hole's doorstep, cut into the hill.
    garden(100.9, -392.5, 3.0, 3.5),
    // A hobbit hole's doorstep, cut into the hill.
    garden(81.1, -402.1, 3.0, 11.8),
    // A hobbit hole's doorstep, cut into the hill.
    garden(61.1, -411.6, 3.0, 19.7),
    // A hobbit hole's doorstep, cut into the hill.
    garden(40.8, -421.0, 3.0, 26.7),
    // The mill, by the water.
    shelf(35.6, -379.4, 5.5, 1.7),
    // A hobbit hole's doorstep, cut into the hill.
    garden(318.7, -426.5, 3.0, 5.2),
    // A hobbit hole's doorstep, cut into the hill.
    garden(326.8, -406.4, 3.0, 16.0),
    // A hobbit hole's doorstep, cut into the hill.
    garden(337.7, -388.2, 3.0, 22.9),
    // The Green Dragon, by the pool.
    knoll(314.9, -448.6, 14.0, 4.9),
    // Bag End's doorstep, cut into its hill.
    garden(308.5, -619.2, 3.5, 22.6),
    // The Prancing Pony.
    knoll(795.3, -545.7, 16.0, 9.6),
    // A house of Bree.
    knoll(824.7, -511.0, 9.0, 9.2),
    // A house of Bree.
    knoll(819.5, -460.9, 9.0, 10.1),
    // A house of Bree.
    knoll(899.8, -450.4, 9.0, 9.9),
    // The stone trolls.
    knoll(1545.1, -654.1, 8.0, 25.9),
    // Rivendell's halls, one over another: the pavilion 4 m over the road
    // beside it, the Last Homely House on a ledge 10 m over it, and the
    // tower on a crag cut into the rock behind, 24 m over it.
    knoll(1601.0, 64.0, 6.0, 23.0),
    Terrace {
        center: Vec2::new(1620.0, 110.0),
        radius: 20.0,
        height: 31.0,
        tilt: Vec2::ZERO,
        skirt: 10.0,
    },
    knoll(1598.0, 160.0, 6.0, 46.0),
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
/// the film was made.
const FLOWERS: [FlowerField; 12] = [
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
            center: Vec2::new(99.5, -389.9),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(80.2, -399.7),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(60.3, -408.1),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(40.6, -418.6),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(316.3, -426.4),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(324.3, -406.4),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(334.2, -388.2),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
    // A hobbit hole's garden.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(305.9, -615.3),
            inner: 0.0,
            outer: 4.0,
        },
        kinds: &[FlowerKind::Phlox, FlowerKind::White],
    },
];

const CONTROL_POINTS: [Vec2; 189] = [
    // 1. The wood by Hobbiton: the start straight, north (-Z), over a crest.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -50.0),
    Vec2::new(0.0, -100.0),
    Vec2::new(0.0, -150.0),
    // Hobbiton: a wide sweep to the right down to the Water, over it on
    // an old wooden bridge, a sweep to the left, and a long sweep to the
    // right round the pool and past Bag End's hill, each eased in and out
    // (drawn by `tools/fellowship_dev/hobbiton_v3.py`, as dense as that,
    // so that the spline through them keeps their curvature).
    Vec2::new(0.06, -163.25),
    Vec2::new(0.50, -176.24),
    Vec2::new(1.67, -189.44),
    Vec2::new(3.67, -202.28),
    Vec2::new(6.56, -215.21),
    Vec2::new(10.22, -227.68),
    Vec2::new(14.78, -240.12),
    Vec2::new(20.04, -252.00),
    Vec2::new(26.19, -263.74),
    Vec2::new(32.96, -274.83),
    Vec2::new(40.59, -285.66),
    Vec2::new(48.76, -295.77),
    Vec2::new(57.74, -305.51),
    Vec2::new(67.16, -314.47),
    Vec2::new(77.25, -323.05),
    Vec2::new(87.40, -331.18),
    Vec2::new(97.78, -339.42),
    Vec2::new(107.96, -347.50),
    Vec2::new(118.34, -355.73),
    Vec2::new(128.49, -363.85),
    Vec2::new(138.61, -372.41),
    Vec2::new(148.06, -381.33),
    Vec2::new(157.08, -391.04),
    Vec2::new(165.28, -401.12),
    Vec2::new(172.95, -411.92),
    Vec2::new(179.89, -423.21),
    Vec2::new(186.06, -434.65),
    Vec2::new(192.09, -446.44),
    Vec2::new(198.24, -457.90),
    Vec2::new(205.11, -469.22),
    Vec2::new(212.50, -479.92),
    Vec2::new(220.66, -490.35),
    Vec2::new(229.25, -500.11),
    Vec2::new(238.58, -509.51),
    Vec2::new(248.27, -518.18),
    Vec2::new(258.64, -526.43),
    Vec2::new(269.27, -533.90),
    Vec2::new(280.54, -540.86),
    Vec2::new(291.98, -547.03),
    Vec2::new(303.99, -552.62),
    Vec2::new(316.08, -557.41),
    Vec2::new(328.67, -561.54),
    Vec2::new(341.23, -564.87),
    Vec2::new(354.22, -567.50),
    Vec2::new(367.09, -569.32),
    Vec2::new(380.29, -570.41),
    Vec2::new(393.28, -570.81),
    Vec2::new(406.53, -570.88),
    // 2. The Shire: a straight over a crest, and a sweep to the right and
    // the left over the Brandywine bridge.
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
const ELEVATION: [(f32, f32); 60] = [
    // The start straight through the wood, over a crest where the bike
    // flies; down the sweep into Hobbiton, 6 m, to the bridge low over the
    // Water, and up round the pool past Bag End's hill.
    (0.0, 1.5),
    (60.0, 2.0),
    (125.0, 6.8),
    (160.0, 5.6),
    (200.0, 4.8),
    (260.0, 3.3),
    (320.0, 1.6),
    (358.0, 0.6),
    (402.0, 0.6),
    (440.0, 1.2),
    (500.0, 2.4),
    (580.0, 3.6),
    (680.0, 4.8),
    (745.0, 5.6),
    // Over the crest of the Shire's straight, where the bike flies (as at
    // Ghost Island's castle street, sharper), to the Brandywine.
    (764.0, 6.0),
    (834.0, 11.0),
    (869.0, 9.2),
    (944.0, 6.0),
    // Bree, and up Weathertop at up to 12 %.
    (1164.0, 6.5),
    (1318.0, 7.0),
    (1378.0, 12.0),
    (1510.0, 27.5),
    (1620.0, 40.0),
    (1702.0, 45.5),
    // Over its top: a crest where the bike flies.
    (1722.0, 46.5),
    (1792.0, 51.5),
    (1827.0, 49.6),
    (1882.0, 46.5),
    // Down through the Trollshaws at up to 12 %, to the ford, low over the
    // water.
    (1978.0, 38.0),
    (2068.0, 27.5),
    (2158.0, 17.0),
    (2248.0, 7.0),
    (2318.0, 3.0),
    (2358.0, 0.5),
    // Climbing steadily through Rivendell's split (as Ghost Island's
    // temple's).
    (2438.0, 3.0),
    (2788.0, 15.0),
    // Up Caradhras at up to 12 %, to the pass right over the mines.
    (2918.0, 21.0),
    (3018.0, 30.0),
    (3118.0, 39.0),
    (3178.0, 45.5),
    (3218.0, 47.5),
    // Down round the mountain to the Doors of Durin; through Moria, down
    // into it and up again.
    (3278.0, 45.5),
    (3438.0, 36.0),
    (3598.0, 24.0),
    (3780.0, 15.0),
    (3840.0, 12.0),
    (3900.0, 9.0),
    (4072.0, 20.0),
    (4152.0, 24.0),
    // Down steadily through Lothlórien's split (as Ghost Island's castle's).
    (4206.0, 23.5),
    (4556.0, 11.5),
    // Down the Anduin's gorge to the water, and the causeway, low over it.
    (4636.0, 9.5),
    (4906.0, 1.5),
    (4976.0, -1.4),
    (5230.0, -1.4),
    // Up over the shoulder of Amon Hen, and down to the line.
    (5290.0, 1.0),
    (5390.0, 6.5),
    (5480.0, 10.5),
    (5550.0, 9.5),
    (5708.0, 1.0),
];

/// The ways off the lap, each through points to one side of it (m along
/// the lap and from its centre line in the comments), from where it leaves
/// the lap to where it joins it again: Ghost Island's, turned and placed.
/// In Rivendell, to the left (east): it swings out to the left before the
/// lap does, and back to it late.
const RIVENDELL_LEFT: [Vec2; 11] = [
    // 2448 m, 0 m.
    Vec2::new(1578.2, -322.0),
    // 2478 m, 4 m.
    Vec2::new(1589.0, -293.4),
    // 2516 m, 13 m.
    Vec2::new(1611.6, -260.5),
    // 2546 m, 32 m.
    Vec2::new(1633.5, -226.3),
    // 2566 m, 48 m.
    Vec2::new(1644.7, -193.3),
    // 2586 m, 60 m.
    Vec2::new(1642.1, -160.3),
    // 2620 m, 65 m.
    Vec2::new(1628.1, -127.3),
    // 2698 m, 57 m.
    Vec2::new(1608.7, -94.4),
    // 2732 m, 29 m.
    Vec2::new(1589.4, -66.7),
    // 2752 m, 11 m.
    Vec2::new(1579.4, -39.7),
    // 2774 m, 2 m.
    Vec2::new(1575.9, -15.3),
];

/// In Lothlórien, to the left (south): it runs straight on where the lap
/// swings right, and out round a wide curve back to it.
const LORIEN_LEFT: [Vec2; 11] = [
    // 4218 m, 1 m.
    Vec2::new(1246.2, 397.9),
    // 4246 m, 15 m.
    Vec2::new(1215.0, 404.0),
    // 4268 m, 40 m.
    Vec2::new(1183.0, 418.9),
    // 4330 m, 65 m.
    Vec2::new(1152.3, 438.9),
    // 4390 m, 64 m.
    Vec2::new(1119.3, 457.3),
    // 4418 m, 57 m.
    Vec2::new(1086.3, 467.5),
    // 4434 m, 44 m.
    Vec2::new(1053.3, 464.5),
    // 4458 m, 25 m.
    Vec2::new(1020.3, 448.1),
    // 4490 m, 9 m.
    Vec2::new(986.6, 424.2),
    // 4520 m, 3 m.
    Vec2::new(961.8, 408.1),
    // 4542 m, 0 m.
    Vec2::new(941.5, 400.6),
];

/// In Lothlórien, to the right (north): it swings out to the right before
/// the lap does, and back to it late.
const LORIEN_RIGHT: [Vec2; 11] = [
    // 4218 m, -0 m.
    Vec2::new(1246.6, 396.1),
    // 4248 m, -4 m.
    Vec2::new(1218.5, 385.2),
    // 4286 m, -14 m.
    Vec2::new(1186.2, 362.8),
    // 4314 m, -32 m.
    Vec2::new(1152.3, 341.0),
    // 4336 m, -48 m.
    Vec2::new(1119.3, 329.8),
    // 4354 m, -60 m.
    Vec2::new(1086.3, 332.4),
    // 4388 m, -65 m.
    Vec2::new(1053.3, 346.4),
    // 4466 m, -57 m.
    Vec2::new(1020.4, 365.8),
    // 4500 m, -29 m.
    Vec2::new(992.2, 384.9),
    // 4522 m, -11 m.
    Vec2::new(964.6, 394.8),
    // 4542 m, -2 m.
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
/// up the hill, zigzagging past the gates of the first holes, and from
/// half way up along the water to the mill; from the road up to the Green Dragon and past the gates of the
/// other holes; and from Bag End's gate down past the party tree to the
/// road.
const HOBBITON_PATH_A: [Vec2; 23] = [
    Vec2::new(118.9, -377.9),
    Vec2::new(115.4, -379.5),
    Vec2::new(112.0, -381.1),
    Vec2::new(108.5, -382.7),
    Vec2::new(105.0, -384.3),
    Vec2::new(101.6, -385.9),
    Vec2::new(98.0, -387.7),
    Vec2::new(94.3, -389.5),
    Vec2::new(90.6, -391.4),
    Vec2::new(86.8, -393.3),
    Vec2::new(83.1, -395.2),
    Vec2::new(79.2, -397.0),
    Vec2::new(75.4, -398.8),
    Vec2::new(71.5, -400.5),
    Vec2::new(67.6, -402.1),
    Vec2::new(63.8, -403.8),
    Vec2::new(60.1, -405.5),
    Vec2::new(56.6, -407.2),
    Vec2::new(53.3, -408.9),
    Vec2::new(50.1, -410.7),
    Vec2::new(46.8, -412.4),
    Vec2::new(43.6, -414.1),
    Vec2::new(40.4, -415.9),
];
const HOBBITON_PATH_MILL: [Vec2; 8] = [
    Vec2::new(59.8, -405.5),
    Vec2::new(56.8, -403.3),
    Vec2::new(53.9, -401.2),
    Vec2::new(51.0, -398.9),
    Vec2::new(48.3, -396.3),
    Vec2::new(45.8, -393.2),
    Vec2::new(43.4, -389.9),
    Vec2::new(41.1, -386.3),
];
const HOBBITON_PATH_B: [Vec2; 36] = [
    Vec2::new(261.0, -506.7),
    Vec2::new(263.8, -503.6),
    Vec2::new(266.6, -500.6),
    Vec2::new(269.4, -497.5),
    Vec2::new(272.2, -494.5),
    Vec2::new(274.9, -491.4),
    Vec2::new(277.7, -488.4),
    Vec2::new(280.4, -485.3),
    Vec2::new(283.0, -482.3),
    Vec2::new(285.4, -479.2),
    Vec2::new(287.7, -476.2),
    Vec2::new(290.0, -473.2),
    Vec2::new(292.2, -470.2),
    Vec2::new(294.4, -467.2),
    Vec2::new(296.7, -464.1),
    Vec2::new(298.8, -461.1),
    Vec2::new(300.7, -457.9),
    Vec2::new(302.4, -454.5),
    Vec2::new(303.9, -451.0),
    Vec2::new(305.3, -447.5),
    Vec2::new(306.7, -444.0),
    Vec2::new(308.0, -440.4),
    Vec2::new(309.4, -436.9),
    Vec2::new(310.8, -433.4),
    Vec2::new(312.2, -429.8),
    Vec2::new(313.7, -426.1),
    Vec2::new(315.2, -422.3),
    Vec2::new(316.8, -418.3),
    Vec2::new(318.4, -414.3),
    Vec2::new(320.0, -410.4),
    Vec2::new(321.7, -406.5),
    Vec2::new(323.6, -402.8),
    Vec2::new(325.5, -399.1),
    Vec2::new(327.5, -395.5),
    Vec2::new(329.4, -391.9),
    Vec2::new(331.4, -388.2),
];
const BAG_END_PATH: [Vec2; 24] = [
    Vec2::new(304.4, -613.0),
    Vec2::new(301.0, -611.2),
    Vec2::new(297.5, -609.3),
    Vec2::new(294.1, -607.4),
    Vec2::new(290.7, -605.5),
    Vec2::new(287.2, -603.7),
    Vec2::new(283.8, -601.8),
    Vec2::new(280.3, -599.9),
    Vec2::new(276.9, -598.0),
    Vec2::new(273.4, -596.1),
    Vec2::new(270.2, -594.1),
    Vec2::new(267.6, -591.6),
    Vec2::new(265.7, -588.6),
    Vec2::new(264.3, -585.2),
    Vec2::new(263.1, -581.7),
    Vec2::new(262.0, -578.1),
    Vec2::new(260.9, -574.6),
    Vec2::new(260.3, -571.1),
    Vec2::new(260.5, -568.0),
    Vec2::new(262.0, -565.3),
    Vec2::new(264.4, -562.9),
    Vec2::new(267.3, -560.6),
    Vec2::new(270.1, -558.4),
    Vec2::new(273.0, -556.1),
];
const PATHS: [&[Vec2]; 4] = [
    &HOBBITON_PATH_A,
    &HOBBITON_PATH_MILL,
    &HOBBITON_PATH_B,
    &BAG_END_PATH,
];

/// The pool of Bywater, inside the long sweep round past Bag End's hill,
/// at the end of the Water: its middle and radius (m).
const POND: Vec2 = Vec2::new(273.3, -437.7);
const POND_RADIUS: f32 = 24.0;
/// The middles of the hills of Hobbiton, either side of the Water, their
/// feet on its far banks: the hobbit holes are dug into them high over
/// the water, the hills rising on behind them.
const HOBBITON_HILLS: [Vec2; 2] = [Vec2::new(36.9, -516.3), Vec2::new(405.6, -337.5)];
/// The Water, the stream through Hobbiton: out of the wood, wide along
/// the left of the sweep down into the village, between the road and the
/// hobbit holes high across it; square under the bridge, and along the
/// right of the sweep on, between the road and the rest of the village,
/// into the pool.
const THE_WATER: [(Vec2, f32); 16] = [
    (Vec2::new(-150.0, 40.0), 8.0),
    (Vec2::new(-115.0, -60.0), 9.0),
    (Vec2::new(-75.0, -120.0), 11.0),
    (Vec2::new(-46.0, -160.4), 13.0),
    (Vec2::new(-42.1, -207.8), 15.0),
    (Vec2::new(-28.8, -255.1), 16.0),
    (Vec2::new(-6.3, -298.8), 16.0),
    (Vec2::new(19.3, -331.8), 16.0),
    (Vec2::new(47.6, -354.1), 16.0),
    (Vec2::new(76.5, -351.3), 16.0),
    (Vec2::new(103.1, -343.6), 16.0),
    (Vec2::new(128.1, -334.7), 16.0),
    (Vec2::new(156.9, -331.8), 16.0),
    (Vec2::new(185.2, -353.6), 15.0),
    (Vec2::new(212.5, -388.4), 14.0),
    (Vec2::new(229.6, -418.8), 13.0),
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
    // Bag End's hill, on the outside of the long sweep round the pool, and
    // Amon Hen, on the outside of the last bend.
    climbed_massif(273.6, -738.2, 170.0, 32.0),
    climbed_massif(40.0, 400.0, 150.0, 20.0),
    // The hills of Hobbiton, either side of the Water, rising behind the
    // hobbit holes dug into their feet across it from the road.
    climbed_massif(HOBBITON_HILLS[0].x, HOBBITON_HILLS[0].y, 160.0, 24.0),
    climbed_massif(HOBBITON_HILLS[1].x, HOBBITON_HILLS[1].y, 150.0, 24.0),
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
        span: span(FORD.to, 2854.0),
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
    const WEATHERTOP: Span = span(1318.0, TROLLSHAWS.from);
    const SUMMIT: Span = span(1702.0, 1882.0);

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
