//! Juguetería (Toy Store): a lap round the inside of a toy shop, the bikes
//! as small as toys, under a ceiling of lamps instead of a sky. Short and
//! twisting, on a narrow road (a slot-car track's, its walls 7.5 m from the
//! middle, see [`Layout::wall`]), it climbs and drops all the way, over the
//! furniture and along the toy tracks (see [`crate::world::toys`]), and
//! splits twice: through a castle and round a loop.
//!
//! 1. **The drag strip** (start straight, north): Hot Wheels' orange track
//!    on the floor, with a booster's kicker (about 75 m), and on under the
//!    diorama table the lap crosses on top later (110–180 m).
//! 2. **The chair and the first table** (about 170–650 m): round to the
//!    right onto a slot-car track, up a ramp onto a chair's seat (5.5 m up,
//!    a jump onto it), on up another onto a table (14 m), esses across it
//!    and a hairpin round its far end.
//! 3. **The diorama** (about 650–1050 m): across the gap onto it, as high,
//!    swinging gently out to the right round a castle of plastic bricks
//!    while a way to the left runs through it, in at its gate and out at
//!    the other (about 730–915 m), the two ways mirror images, as long and
//!    as fast; then swinging right and left over the start straight below.
//! 4. **The Hot Wheels helix** (about 1050–1330 m): orange track, banked,
//!    a turn and a quarter round a blue column up to the top of a tall
//!    shelving unit (39 m), near the ceiling's lamps.
//! 5. **Along the top of the shelves** (about 1330–1420 m).
//! 6. **The slide** (about 1420–1880 m): a helter-skelter's chute, two turns
//!    and a quarter round its tower, down to the floor.
//! 7. **The north wall** (about 1880–2200 m): east along the floor beside a
//!    row of wooden blocks, the lap swinging out while a way to the right
//!    goes round a loop of orange track (about 1910–2160 m).
//! 8. **Under the first table** (about 2200–2450 m): south under it and in
//!    esses down the east side.
//! 9. **The train table** (about 2450–2810 m): up a ramp onto it (9 m),
//!    round past its electric train, and straight off its edge back down to
//!    the floor.
//! 10. **Home** (about 2810 m to the finish): round onto the drag strip.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::{Banking, Branch, Layout, VerticalLoop};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line.
const DRAG_STRIP: Span = span(2888.0, 130.0);
const TABLES: Span = span(222.0, 1053.0);
const HELIX: Span = span(1053.0, 1348.0);
const SHELVES: Span = span(1348.0, 1427.0);
const SLIDE: Span = span(1427.0, 1888.0);
const TRAIN_TABLE: Span = span(2453.0, 2818.0);
/// Along the north wall, past the loop.
const NORTH_WALL: Span = span(1908.0, 2208.0);
/// How far the walls stand from the middle of the road (m): a toy track's,
/// narrow.
const WALL: f32 = 7.5;
/// The loop's radius, and how far its way out is to the right of its way
/// in (m), as `tools/toy_store_layout.py` has them.
const LOOP_RADIUS: f32 = 14.0;
const LOOP_SHIFT: f32 = 14.0;

/// The colours of the toy store's furniture.
const FELT_GREEN: Color = Color::srgb(0.24, 0.52, 0.3);
const CHAIR_RED: Color = Color::srgb(0.86, 0.16, 0.12);
const TABLE_YELLOW: Color = Color::srgb(1.0, 0.8, 0.22);
const SHELF_WHITE: Color = Color::srgb(0.94, 0.94, 0.92);
const SEA_BLUE: Color = Color::srgb(0.2, 0.5, 0.85);
const STORE_BLUE: Color = Color::srgb(0.12, 0.32, 0.86);
const HOT_WHEELS_YELLOW: Color = Color::srgb(1.0, 0.76, 0.05);

pub const TOY_STORE: Circuit = Circuit {
    name: "Juguetería",
    name_en: "Toy Store",
    description: "Una juguetería: Scalextric sobre una silla y las mesas,\n\
                  un castillo, un rizo, una espiral y un tobogán",
    description_en: "A toy store: slot cars over a chair and the tables,\n\
                     a castle, a loop, a spiral and a slide",
    // Indoors: no sea, no birds, no wind.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.0,
        cicadas: 0.0,
        wind: 0.0,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/plastic_velocity_rush.mp3"),
        length: 176.3,
        loudness: -14.4,
    },
    // Narrow and twisting, ridden slower than most.
    pace: PACE,
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Banked in the bends on the floor and on the furniture as a
        // slot-car track's banked curves are, raised on supports under
        // their outsides, but level beside the ways off it, as they are
        // (the way through the castle and the lap beside it mirror images);
        // banked round the helix and down the slide; level on top of the
        // shelves.
        wall: WALL,
        banking: &[
            Banking {
                from: 0.0,
                to: CASTLE.from,
                factor: RAISED_BANKING,
                raised: true,
            },
            Banking {
                from: CASTLE.from,
                to: CASTLE.to,
                factor: 0.0,
                raised: false,
            },
            Banking {
                from: CASTLE.to,
                to: HELIX.from,
                factor: RAISED_BANKING,
                raised: true,
            },
            Banking {
                from: HELIX.from,
                to: HELIX.to,
                factor: 2.5,
                raised: false,
            },
            Banking {
                from: SHELVES.from,
                to: SHELVES.to,
                factor: 0.0,
                raised: false,
            },
            Banking {
                from: SLIDE.from,
                to: SLIDE.to,
                factor: 2.5,
                raised: false,
            },
            Banking {
                from: SLIDE.to,
                to: LOOP.from,
                factor: RAISED_BANKING,
                raised: true,
            },
            Banking {
                from: LOOP.from,
                to: LOOP.to,
                factor: 0.0,
                raised: false,
            },
            Banking {
                from: LOOP.to,
                to: f32::MAX,
                factor: RAISED_BANKING,
                raised: true,
            },
        ],
        narrows: &[],
        branches: &[
            // Through the castle on the diorama, as the lap swings round it.
            Branch {
                from: CASTLE.from,
                to: CASTLE.to,
                approach: &CASTLE_WAY,
                vertical_loop: None,
                exit: &[],
                rise: &[],
                narrows: &[],
            },
            // Round a loop of orange track beside the north wall.
            Branch {
                from: LOOP.from,
                to: LOOP.to,
                approach: &LOOP_APPROACH,
                vertical_loop: Some(VerticalLoop {
                    mouth: LOOP_MOUTH,
                    heading: LOOP_HEADING,
                    height: 0.0,
                    radius: LOOP_RADIUS,
                    shift: -LOOP_SHIFT,
                }),
                exit: &LOOP_EXIT,
                rise: &[],
                narrows: &[],
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::Inland(|_| 1000.0),
        hills: 0.0,
        soil: Soil::Floor,
        greens: Greens::Lawn,
        grass_shade: 1.0,
        massifs: &[],
        cuttings: &[],
        terraces: &[],
        mountains: |_| 0.0,
        // A slot-car track's red and white kerbs.
        curbs: [Color::srgb(0.88, 0.08, 0.06), Color::srgb(0.96, 0.96, 0.94)],
        // Toy blocks along the north wall, and elsewhere a slot-car track's
        // borders.
        walls: &[
            WallRun {
                span: NORTH_WALL,
                side: Side::Left,
                style: WallStyle::Blocks,
            },
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::SlotBorder,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::SlotBorder,
            },
        ],
        // Where another stretch passes over: the start straight under the
        // diorama table, the helix and the slide each under its own upper
        // turn, and the floor under the first table.
        tunnels: &[
            span(112.0, 182.0),
            span(994.0, 1164.0),
            span(1564.0, 1906.0),
            span(2266.0, 2414.0),
        ],
        bridges: &[
            Bridge {
                span: DRAG_STRIP,
                style: BridgeStyle::OrangeTrack,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: TABLES,
                style: BridgeStyle::SlotTrack,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: HELIX,
                style: BridgeStyle::OrangeTrack,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: SHELVES,
                style: BridgeStyle::SlotTrack,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: SLIDE,
                style: BridgeStyle::Slide,
                fence: WOODEN_FENCE,
            },
            Bridge {
                span: TRAIN_TABLE,
                style: BridgeStyle::SlotTrack,
                fence: WOODEN_FENCE,
            },
        ],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: STORE_BLUE,
                text: Some("TOY STORE"),
            },
            // On the first table, on top of the shelves and under the
            // first table.
            Gantry {
                along: 635.0,
                color: HOT_WHEELS_YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1381.0,
                color: HOT_WHEELS_YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2303.0,
                color: HOT_WHEELS_YELLOW,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[],
        lamps: &[],
        trees: &[],
        forests: &[],
        houses: Houses {
            max: 0,
            paints: &[],
            spans: &[],
            style: HouseStyle::Cottage,
        },
        palms: &[],
        grand_hotel: None,
        hotels: &[],
        grandstands: &[],
        lighthouses: &[],
        buoys: &[],
        boats: &[],
        chevrons: &[],
        beach_parasols: &[],
        brake_markers: &[],
        totems: &[],
        sunflowers: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Indoor,
        water: Water::Sea,
        // No water to mirror anything: the sea lies under the floor.
        cartoon_water: true,
        pit: None,
        mine: None,
        desert: None,
        wheelie_halo: None,
        japan: None,
        middle_earth: None,
        space: None,
        toys: Some(STORE),
    },
};

/// How fast it is ridden against most tracks (see [`Circuit::pace`]).
const PACE: f32 = 0.75;
/// How much more the bends on the floor and on the furniture bank than a
/// road's would (see [`Banking`]): most as much as a road may.
const RAISED_BANKING: f32 = 1.5;

const STORE: ToyStore = ToyStore {
    room: (Vec2::new(-175.0, -335.0), Vec2::new(380.0, 130.0)),
    ceiling: 55.0,
    furniture: &[
        // The chair the road climbs onto from the floor, its back to the
        // north.
        Furniture {
            kind: FurnitureKind::Chair,
            center: Vec2::new(109.0, -202.0),
            size: Vec2::new(24.0, 18.0),
            heading: 0.0,
            top: 5.1,
            color: CHAIR_RED,
        },
        // The first table, two pushed together: esses across the one and a
        // hairpin onto the other, the floor road running under both
        // between their legs.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(280.5, -195.0),
            size: Vec2::new(169.0, 70.0),
            heading: 0.0,
            top: 13.2,
            color: FELT_GREEN,
        },
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(317.5, -145.5),
            size: Vec2::new(95.0, 29.0),
            heading: 0.0,
            top: 13.2,
            color: FELT_GREEN,
        },
        // The diorama, over the start straight, its castles on it.
        Furniture {
            kind: FurnitureKind::Diorama,
            center: Vec2::new(70.0, -131.5),
            size: Vec2::new(364.0, 67.0),
            heading: 0.0,
            top: 13.4,
            color: CHAIR_RED,
        },
        // The tall shelving unit the road runs along the top of.
        Furniture {
            kind: FurnitureKind::Shelves,
            center: Vec2::new(-146.0, -226.0),
            size: Vec2::new(20.0, 24.0),
            heading: 0.0,
            top: 38.6,
            color: SHELF_WHITE,
        },
        // The train table, the road round it and straight off its edge.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(204.5, 56.5),
            size: Vec2::new(141.0, 89.0),
            heading: 0.0,
            top: 8.2,
            color: TABLE_YELLOW,
        },
        // A low table of blue sea beside the start straight, a pirate ship
        // on it.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(55.0, -55.0),
            size: Vec2::new(80.0, 60.0),
            heading: 0.0,
            top: 4.0,
            color: SEA_BLUE,
        },
    ],
    towers: &[
        Tower {
            kind: TowerKind::Column,
            center: Vec2::new(-110.0, -178.0),
            radius: 8.0,
            height: 41.0,
        },
        Tower {
            kind: TowerKind::HelterSkelter,
            center: Vec2::new(-116.0, -270.0),
            radius: 12.0,
            height: 46.0,
        },
    ],
    trains: &[
        // On its rug in the middle of the floor, by the road coming down
        // the east side: coaches.
        Train {
            track: &[
                Vec2::new(158.0, -80.0),
                Vec2::new(198.0, -80.0),
                Vec2::new(238.0, -80.0),
                Vec2::new(250.0, -65.0),
                Vec2::new(238.0, -50.0),
                Vec2::new(198.0, -50.0),
                Vec2::new(158.0, -50.0),
                Vec2::new(146.0, -65.0),
            ],
            height: 0.0,
            wagon: Wagon::Coach,
            wagons: 4,
            speed: 12.0,
        },
        // On the train table, beside the road round it: wagons of crates.
        Train {
            track: &[
                Vec2::new(160.0, 40.0),
                Vec2::new(190.0, 40.0),
                Vec2::new(220.0, 40.0),
                Vec2::new(231.0, 51.0),
                Vec2::new(220.0, 62.0),
                Vec2::new(190.0, 62.0),
                Vec2::new(160.0, 62.0),
                Vec2::new(149.0, 51.0),
            ],
            height: 8.2,
            wagon: Wagon::Freight,
            wagons: 4,
            speed: 8.0,
        },
    ],
    castles: &[
        // The castle the way to the left runs through, in at one gate and
        // out at the other.
        Castle {
            center: Vec2::new(117.3, -129.1),
            size: Vec2::new(24.0, 36.0),
            heading: FRAC_PI_2,
            base: 13.4,
            kind: CastleKind::Through,
        },
        // Its gate to the lap.
        Castle {
            center: Vec2::new(-60.0, -114.0),
            size: Vec2::new(28.0, 23.0),
            heading: PI,
            base: 13.4,
            kind: CastleKind::Big,
        },
        // Two towers either side of the lap where it comes onto the
        // diorama, their gates to it.
        Castle {
            center: Vec2::new(240.0, -156.0),
            size: Vec2::new(7.6, 7.6),
            heading: 0.0,
            base: 13.4,
            kind: CastleKind::Tower,
        },
        Castle {
            center: Vec2::new(240.0, -128.0),
            size: Vec2::new(7.6, 7.6),
            heading: PI,
            base: 13.4,
            kind: CastleKind::Tower,
        },
    ],
    loops: &[],
    mats: &[
        Mat {
            center: Vec2::new(-95.0, 0.0),
            size: Vec2::new(120.0, 180.0),
            heading: 0.0,
            kind: MatKind::Foam,
        },
        Mat {
            center: Vec2::new(198.0, -65.0),
            size: Vec2::new(124.0, 52.0),
            heading: 0.0,
            kind: MatKind::Rug(Color::srgb(0.45, 0.62, 0.86)),
        },
    ],
    toys: &[
        // Meshy's, by the road: a tin robot standing by the start straight
        // and a teddy bear sitting by it, turned to the bikes coming.
        Toy {
            kind: ToyKind::Robot,
            position: Vec2::new(-13.0, -25.0),
            heading: 1.0,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Bear,
            position: Vec2::new(-19.0, 35.0),
            heading: 1.2,
            scale: 1.0,
            base: 0.0,
        },
        // The pirate ship on its sea beside the start straight, its bow to
        // the start, and a duck afloat by it.
        Toy {
            kind: ToyKind::Ship,
            position: Vec2::new(26.0, -55.0),
            heading: 0.15,
            scale: 1.0,
            base: 4.0,
        },
        Toy {
            kind: ToyKind::Duck,
            position: Vec2::new(50.0, -35.0),
            heading: -2.2,
            scale: 1.0,
            base: 4.0,
        },
        // A duck and her ducklings by the last straight, going the bikes'
        // way.
        Toy {
            kind: ToyKind::Duck,
            position: Vec2::new(40.0, 67.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Duck,
            position: Vec2::new(51.0, 68.0),
            heading: -FRAC_PI_2 - 0.2,
            scale: 0.6,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Duck,
            position: Vec2::new(60.0, 68.0),
            heading: -FRAC_PI_2 + 0.25,
            scale: 0.6,
            base: 0.0,
        },
        // Spinning tops on the first table, among its esses.
        Toy {
            kind: ToyKind::Top,
            position: Vec2::new(229.0, -214.0),
            heading: 0.0,
            scale: 1.0,
            base: 13.2,
        },
        Toy {
            kind: ToyKind::Top,
            position: Vec2::new(289.0, -199.0),
            heading: 1.0,
            scale: 0.85,
            base: 13.2,
        },
        Toy {
            kind: ToyKind::Top,
            position: Vec2::new(325.0, -214.0),
            heading: 2.0,
            scale: 1.0,
            base: 13.2,
        },
        // Hanging from the ceiling over the floor's roads, their
        // undersides well over the bikes, clear of the high roads: a
        // biplane over the start straight and another over the road
        // coming down the east side, each flying the bikes' way, and the
        // helicopter over the last straight.
        Toy {
            kind: ToyKind::Plane,
            position: Vec2::new(0.0, -45.0),
            heading: PI,
            scale: 1.0,
            base: 16.0,
        },
        Toy {
            kind: ToyKind::Plane,
            position: Vec2::new(264.0, -55.0),
            heading: 0.0,
            scale: 1.0,
            base: 18.0,
        },
        Toy {
            kind: ToyKind::Helicopter,
            position: Vec2::new(90.0, 82.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 20.0,
        },
        // Built in code: blocks, balls and tyres.
        Toy {
            kind: ToyKind::Blocks,
            position: Vec2::new(-60.0, -40.0),
            heading: 0.3,
            scale: 6.0,
            base: 0.0,
        },
        // In the middle of the last bend.
        Toy {
            kind: ToyKind::Blocks,
            position: Vec2::new(40.0, 40.0),
            heading: -0.4,
            scale: 6.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Ball,
            position: Vec2::new(-120.0, -60.0),
            heading: 0.0,
            scale: 8.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Ball,
            position: Vec2::new(110.0, -5.0),
            heading: 1.0,
            scale: 7.0,
            base: 0.0,
        },
        // Tyres round the outside of the floor's corners.
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(-20.0, -215.0),
            heading: 0.8,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(285.0, -315.0),
            heading: -0.8,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(310.0, 110.0),
            heading: 0.0,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(-20.0, 100.0),
            heading: 1.57,
            scale: 3.0,
            base: 0.0,
        },
    ],
};

/// Where the branches leave the lap and join it again (m from the finish
/// line), as `tools/toy_store_layout.py` works them out: the way through
/// the castle, and the way round the loop.
const CASTLE: Span = span(730.1, 914.2);
const LOOP: Span = span(1916.3, 2160.4);

/// The way through the castle's control points, as the tool walks them.
const CASTLE_WAY: [Vec2; 14] = [
    Vec2::new(198.0, -142.0),
    Vec2::new(186.0, -141.3),
    Vec2::new(174.1, -139.1),
    Vec2::new(162.6, -135.5),
    Vec2::new(151.2, -132.0),
    Vec2::new(139.3, -129.8),
    Vec2::new(127.3, -129.1),
    Vec2::new(107.3, -129.1),
    Vec2::new(95.3, -129.8),
    Vec2::new(83.4, -132.0),
    Vec2::new(72.0, -135.5),
    Vec2::new(60.5, -139.1),
    Vec2::new(48.6, -141.3),
    Vec2::new(36.6, -142.0),
];

/// The way round the loop: the control points of the road into it, its
/// mouth (the bikes go in heading east) and the road out of it.
const LOOP_APPROACH: [Vec2; 4] = [
    Vec2::new(-35.0, -300.0),
    Vec2::new(-21.3, -297.6),
    Vec2::new(-9.3, -290.6),
    Vec2::new(2.7, -283.7),
];
const LOOP_MOUTH: Vec2 = Vec2::new(66.4, -281.3);
const LOOP_HEADING: f32 = -FRAC_PI_2;
const LOOP_EXIT: [Vec2; 6] = [
    Vec2::new(128.7, -269.2),
    Vec2::new(139.8, -274.9),
    Vec2::new(148.7, -283.6),
    Vec2::new(157.5, -292.4),
    Vec2::new(168.6, -298.1),
    Vec2::new(180.9, -300.0),
];

/// The centre line's points, in race order, as `tools/toy_store_layout.py`
/// walks them.
const CONTROL_POINTS: [Vec2; 202] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -20.0),
    Vec2::new(0.0, -40.0),
    Vec2::new(0.0, -55.0),
    Vec2::new(0.0, -75.0),
    Vec2::new(0.0, -85.0),
    Vec2::new(0.0, -99.0),
    Vec2::new(0.0, -113.0),
    Vec2::new(0.0, -141.5),
    Vec2::new(0.0, -170.0),
    Vec2::new(2.4, -182.2),
    Vec2::new(9.4, -192.6),
    Vec2::new(19.8, -199.6),
    Vec2::new(32.0, -202.0),
    Vec2::new(40.0, -202.0),
    Vec2::new(50.0, -202.0),
    Vec2::new(75.0, -202.0),
    Vec2::new(100.0, -202.0),
    Vec2::new(120.0, -202.0),
    Vec2::new(128.0, -202.0),
    Vec2::new(158.0, -202.0),
    Vec2::new(188.0, -202.0),
    Vec2::new(202.0, -202.0),
    Vec2::new(214.0, -202.0),
    Vec2::new(226.0, -202.0),
    Vec2::new(238.0, -204.1),
    Vec2::new(248.5, -210.2),
    Vec2::new(259.0, -216.3),
    Vec2::new(271.0, -218.4),
    Vec2::new(283.0, -216.3),
    Vec2::new(293.5, -210.2),
    Vec2::new(304.0, -204.1),
    Vec2::new(316.0, -202.0),
    Vec2::new(324.0, -202.0),
    Vec2::new(335.5, -199.7),
    Vec2::new(345.2, -193.2),
    Vec2::new(351.7, -183.5),
    Vec2::new(354.0, -172.0),
    Vec2::new(351.7, -160.5),
    Vec2::new(345.2, -150.8),
    Vec2::new(335.5, -144.3),
    Vec2::new(324.0, -142.0),
    Vec2::new(304.0, -142.0),
    Vec2::new(284.0, -142.0),
    Vec2::new(266.0, -142.0),
    Vec2::new(248.0, -142.0),
    Vec2::new(228.0, -142.0),
    Vec2::new(208.0, -142.0),
    Vec2::new(198.0, -142.0),
    Vec2::new(186.0, -142.7),
    Vec2::new(174.1, -144.9),
    Vec2::new(162.6, -148.5),
    Vec2::new(151.2, -152.0),
    Vec2::new(139.3, -154.2),
    Vec2::new(127.3, -154.9),
    Vec2::new(117.3, -154.9),
    Vec2::new(107.3, -154.9),
    Vec2::new(95.3, -154.2),
    Vec2::new(83.4, -152.0),
    Vec2::new(72.0, -148.5),
    Vec2::new(60.5, -144.9),
    Vec2::new(48.6, -142.7),
    Vec2::new(36.6, -142.0),
    Vec2::new(26.6, -142.0),
    Vec2::new(16.2, -142.9),
    Vec2::new(6.1, -145.6),
    Vec2::new(-7.5, -148.8),
    Vec2::new(-21.4, -148.8),
    Vec2::new(-35.0, -145.6),
    Vec2::new(-45.1, -142.9),
    Vec2::new(-55.5, -142.0),
    Vec2::new(-70.0, -142.0),
    Vec2::new(-90.0, -142.0),
    Vec2::new(-110.0, -142.0),
    Vec2::new(-121.1, -143.8),
    Vec2::new(-131.2, -148.9),
    Vec2::new(-139.1, -156.8),
    Vec2::new(-144.2, -166.9),
    Vec2::new(-146.0, -178.0),
    Vec2::new(-144.2, -189.1),
    Vec2::new(-139.1, -199.2),
    Vec2::new(-131.2, -207.1),
    Vec2::new(-121.1, -212.2),
    Vec2::new(-110.0, -214.0),
    Vec2::new(-98.9, -212.2),
    Vec2::new(-88.8, -207.1),
    Vec2::new(-80.9, -199.2),
    Vec2::new(-75.8, -189.1),
    Vec2::new(-74.0, -178.0),
    Vec2::new(-75.8, -166.9),
    Vec2::new(-80.9, -156.8),
    Vec2::new(-88.8, -148.9),
    Vec2::new(-98.9, -143.8),
    Vec2::new(-110.0, -142.0),
    Vec2::new(-121.1, -143.8),
    Vec2::new(-131.2, -148.9),
    Vec2::new(-139.1, -156.8),
    Vec2::new(-144.2, -166.9),
    Vec2::new(-146.0, -178.0),
    Vec2::new(-146.0, -208.0),
    Vec2::new(-146.0, -240.0),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-143.7, -281.5),
    Vec2::new(-137.2, -291.2),
    Vec2::new(-127.5, -297.7),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-104.5, -297.7),
    Vec2::new(-94.8, -291.2),
    Vec2::new(-88.3, -281.5),
    Vec2::new(-86.0, -270.0),
    Vec2::new(-88.3, -258.5),
    Vec2::new(-94.8, -248.8),
    Vec2::new(-104.5, -242.3),
    Vec2::new(-116.0, -240.0),
    Vec2::new(-127.5, -242.3),
    Vec2::new(-137.2, -248.8),
    Vec2::new(-143.7, -258.5),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-143.7, -281.5),
    Vec2::new(-137.2, -291.2),
    Vec2::new(-127.5, -297.7),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-104.5, -297.7),
    Vec2::new(-94.8, -291.2),
    Vec2::new(-88.3, -281.5),
    Vec2::new(-86.0, -270.0),
    Vec2::new(-88.3, -258.5),
    Vec2::new(-94.8, -248.8),
    Vec2::new(-104.5, -242.3),
    Vec2::new(-116.0, -240.0),
    Vec2::new(-127.5, -242.3),
    Vec2::new(-137.2, -248.8),
    Vec2::new(-143.7, -258.5),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-143.7, -281.5),
    Vec2::new(-137.2, -291.2),
    Vec2::new(-127.5, -297.7),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-86.0, -300.0),
    Vec2::new(-68.0, -300.0),
    Vec2::new(-50.0, -300.0),
    Vec2::new(-30.0, -300.0),
    Vec2::new(-10.0, -300.0),
    Vec2::new(2.5, -301.0),
    Vec2::new(14.7, -303.9),
    Vec2::new(26.9, -306.9),
    Vec2::new(39.4, -307.8),
    Vec2::new(52.0, -306.9),
    Vec2::new(64.2, -303.9),
    Vec2::new(76.4, -301.0),
    Vec2::new(88.9, -300.0),
    Vec2::new(114.7, -300.0),
    Vec2::new(140.4, -300.0),
    Vec2::new(166.2, -300.0),
    Vec2::new(192.0, -300.0),
    Vec2::new(212.0, -300.0),
    Vec2::new(232.0, -300.0),
    Vec2::new(244.2, -297.6),
    Vec2::new(254.6, -290.6),
    Vec2::new(261.6, -280.2),
    Vec2::new(264.0, -268.0),
    Vec2::new(264.0, -243.0),
    Vec2::new(264.0, -218.0),
    Vec2::new(264.0, -193.0),
    Vec2::new(264.0, -168.0),
    Vec2::new(265.9, -156.0),
    Vec2::new(271.2, -145.1),
    Vec2::new(276.6, -134.1),
    Vec2::new(278.5, -122.1),
    Vec2::new(276.6, -110.1),
    Vec2::new(271.2, -99.2),
    Vec2::new(265.9, -88.3),
    Vec2::new(264.0, -76.2),
    Vec2::new(264.0, -66.2),
    Vec2::new(264.0, -40.9),
    Vec2::new(264.0, -15.6),
    Vec2::new(264.0, 9.8),
    Vec2::new(264.0, 29.8),
    Vec2::new(264.0, 49.8),
    Vec2::new(261.6, 62.0),
    Vec2::new(254.6, 72.4),
    Vec2::new(244.2, 79.3),
    Vec2::new(232.0, 81.8),
    Vec2::new(226.0, 81.8),
    Vec2::new(216.9, 83.0),
    Vec2::new(208.5, 86.5),
    Vec2::new(197.1, 90.6),
    Vec2::new(184.9, 90.6),
    Vec2::new(173.5, 86.5),
    Vec2::new(165.1, 83.0),
    Vec2::new(156.0, 81.8),
    Vec2::new(131.0, 81.8),
    Vec2::new(104.3, 81.8),
    Vec2::new(77.7, 81.8),
    Vec2::new(51.0, 81.8),
    Vec2::new(40.0, 81.8),
    Vec2::new(27.6, 79.8),
    Vec2::new(16.5, 74.1),
    Vec2::new(7.6, 65.3),
    Vec2::new(2.0, 54.1),
    Vec2::new(0.0, 41.8),
    Vec2::new(0.0, 20.9),
];

/// The road's height along the lap
/// (metres from the finish line, height over the floor), as `tools/toy_store_layout.py` works them out.
const ELEVATION: [(f32, f32); 74] = [
    (0.0, 0.0), // start
    (40.0, 0.0),
    (54.0, 0.05),
    (74.0, 0.9), // kicker
    (84.0, 1.2),
    (98.0, 0.1),
    (112.0, 0.0),
    (170.0, 0.0), // north
    (228.0, 0.0), // ramp
    (238.0, 0.4),
    (288.0, 5.5), // chair
    (308.1, 5.5), // chair end
    (316.1, 6.3),
    (376.1, 12.8), // T0
    (390.1, 13.7),
    (402.1, 14.0),
    (414.1, 14.0),
    (438.1, 14.0),
    (488.1, 14.0),
    (512.1, 14.0),
    (520.1, 14.0),
    (614.1, 14.0), // hairpin
    (654.1, 14.0), // T0 edge
    (690.1, 14.0), // T1 edge
    (730.1, 14.0),
    (822.1, 14.0), // castle
    (914.2, 14.0), // castle join
    (936.2, 14.0),
    (976.2, 14.0),
    (998.2, 14.0),
    (1012.2, 14.0),
    (1052.2, 14.0), // X
    (1108.2, 18.6),
    (1166.2, 23.2),
    (1222.2, 27.8),
    (1278.2, 32.4),
    (1336.2, 37.0), // helix top
    (1366.2, 39.0), // gondola
    (1398.2, 39.0), // shelves end
    (1428.2, 37.0), // slide
    (1474.2, 33.17),
    (1522.3, 29.33),
    (1568.3, 25.5),
    (1616.3, 21.67),
    (1662.3, 17.83),
    (1710.3, 14.0),
    (1756.3, 10.17),
    (1804.3, 6.33),
    (1850.3, 2.5),
    (1880.3, 0.3), // floor
    (1916.3, 0.0),
    (1956.3, 0.0), // loop fork
    (1982.3, 0.0),
    (2032.3, 0.0),
    (2058.3, 0.0),
    (2160.4, 0.0), // loop join
    (2200.4, 0.0), // north wall end
    (2250.4, 0.0),
    (2350.4, 0.0), // under T0
    (2374.4, 0.0),
    (2424.4, 0.0),
    (2448.4, 0.0),
    (2458.4, 0.3),
    (2534.4, 8.4), // T2
    (2574.4, 9.0),
    (2624.4, 9.0),
    (2630.4, 9.0),
    (2648.4, 9.0),
    (2686.4, 9.0),
    (2704.5, 9.0),
    (2728.5, 9.0), // drop
    (2808.5, 0.3), // drop end
    (2820.5, 0.0),
    (2882.5, 0.0), // home
];
