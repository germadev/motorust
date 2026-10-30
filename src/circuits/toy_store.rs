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
//! 4. **The Hot Wheels helix** (about 1050–1330 m): orange track, banked
//!    steeply and gripping hard, a turn and a quarter round a blue column up
//!    to the top of a tall shelving unit (39 m), near the ceiling's lamps,
//!    climbed flat out.
//! 5. **Along the top of the shelves** (about 1330–1420 m).
//! 6. **The slide** (about 1420–1880 m): a helter-skelter's chute, two turns
//!    and a quarter round its tower, down to the floor.
//! 7. **The north wall** (about 1880–2200 m): east along the floor beside a
//!    row of wooden blocks, the lap swinging away toward the wall while a
//!    way to the right goes straight on round a loop of orange track (about
//!    1920–2160 m), a booster on each.
//! 8. **Under the first table** (about 2200–2450 m): south under it and in
//!    esses down the east side.
//! 9. **The train table** (about 2450–2830 m): up a ramp onto it (9 m),
//!    straight on at the shop's glass and its doors, round short of it and
//!    along it past its electric train, right by the road, and straight off
//!    its edge back down to the floor.
//! 10. **Home** (about 2830 m to the finish): round onto the drag strip.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::{Banking, Branch, Layout, VerticalLoop};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line.
const DRAG_STRIP: Span = span(2928.0, 130.0);
const TABLES: Span = span(222.0, 1053.0);
const HELIX: Span = span(1053.0, 1348.0);
const SHELVES: Span = span(1348.0, 1427.0);
const SLIDE: Span = span(1427.0, 1888.0);
const TRAIN_TABLE: Span = span(2453.0, 2838.0);
/// Along the north wall, past the loop; round the north-east corner, under
/// the first table and down the east side; and round the last bend.
const NORTH_WALL: Span = span(1908.0, 2208.0);
const BRICK_WALLS: Span = span(2208.0, 2453.0);
const HOME: Span = span(2838.0, 2928.0);
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
                grip: 1.0,
            },
            Banking {
                from: CASTLE.from,
                to: CASTLE.to,
                factor: 0.0,
                raised: false,
                grip: 1.0,
            },
            Banking {
                from: CASTLE.to,
                to: HELIX.from,
                factor: RAISED_BANKING,
                raised: true,
                grip: 1.0,
            },
            Banking {
                from: HELIX.from,
                to: HELIX.to,
                factor: 2.5,
                raised: false,
                // Hot Wheels' banked track: steep, and gripping hard.
                grip: HOT_WHEELS_GRIP,
            },
            Banking {
                from: SHELVES.from,
                to: SHELVES.to,
                factor: 0.0,
                raised: false,
                grip: 1.0,
            },
            Banking {
                from: SLIDE.from,
                to: SLIDE.to,
                factor: 2.5,
                raised: false,
                grip: 1.0,
            },
            Banking {
                from: SLIDE.to,
                to: LOOP.from,
                factor: RAISED_BANKING,
                raised: true,
                grip: 1.0,
            },
            Banking {
                from: LOOP.from,
                to: LOOP.to,
                factor: 0.0,
                raised: false,
                grip: 1.0,
            },
            Banking {
                from: LOOP.to,
                to: f32::MAX,
                factor: RAISED_BANKING,
                raised: true,
                grip: 1.0,
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
                    // Hot Wheels' boosters, on into the loop and away from
                    // it: the way round it is the longer.
                    boosters: true,
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
            // Walls of bricks round the north-east corner, under the first
            // table and down the east side, and round the last bend.
            WallRun {
                span: BRICK_WALLS,
                side: Side::Left,
                style: WallStyle::Bricks,
            },
            WallRun {
                span: BRICK_WALLS,
                side: Side::Right,
                style: WallStyle::Bricks,
            },
            WallRun {
                span: HOME,
                side: Side::Left,
                style: WallStyle::Bricks,
            },
            WallRun {
                span: HOME,
                side: Side::Right,
                style: WallStyle::Bricks,
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
            span(2270.0, 2418.0),
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
const PACE: f32 = 0.80;
/// How much more the bends on the floor and on the furniture bank than a
/// road's would (see [`Banking`]): most as much as a road may.
const RAISED_BANKING: f32 = 1.5;
/// How much harder the helix grips, banked as much more steeply (see
/// [`Banking::grip`], about 23°): enough to go up it flat out, no braking
/// for its tight turns (36 m round). (Not the slide, down its tower: the
/// bikes come off it as fast as the way round the loop by the north wall
/// is reckoned for, its booster making up for the loop's length.)
const HOT_WHEELS_GRIP: f32 = 2.4;

const STORE: ToyStore = ToyStore {
    room: (Vec2::new(-175.0, -335.0), Vec2::new(380.0, 130.0)),
    ceiling: 55.0,
    // In the south wall, the glass ahead of the bikes coming down the east
    // side onto the train table, along it to the shop's doors, and along
    // the left of the road round it, close by.
    entrance: Some(Entrance {
        from: 168.0,
        to: 344.0,
        doors: 320.0,
    }),
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
        // The train table, the road along it at the shop's doors and the
        // glass, round beside its train and straight off its edge.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(207.0, 66.0),
            size: Vec2::new(146.0, 108.0),
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
        // the east side: coaches, past a station and through a tunnel, on
        // a straight there.
        Train {
            track: &[
                Vec2::new(158.0, -80.0),
                Vec2::new(178.0, -80.0),
                Vec2::new(198.0, -80.0),
                Vec2::new(218.0, -80.0),
                Vec2::new(238.0, -80.0),
                Vec2::new(250.0, -65.0),
                Vec2::new(238.0, -50.0),
                Vec2::new(218.0, -50.0),
                Vec2::new(198.0, -50.0),
                Vec2::new(178.0, -50.0),
                Vec2::new(158.0, -50.0),
                Vec2::new(146.0, -65.0),
            ],
            height: 0.0,
            wagon: Wagon::Coach,
            wagons: 4,
            speed: 12.0,
        },
        // On the train table, right beside the road along it: wagons of
        // crates, on an oval reaching out towards the road's walls.
        Train {
            track: &[
                Vec2::new(160.0, 60.0),
                Vec2::new(190.0, 60.0),
                Vec2::new(220.0, 60.0),
                Vec2::new(238.0, 60.0),
                Vec2::new(249.0, 73.0),
                Vec2::new(238.0, 86.0),
                Vec2::new(220.0, 86.0),
                Vec2::new(190.0, 86.0),
                Vec2::new(160.0, 86.0),
                Vec2::new(149.0, 73.0),
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
        // A plush giraffe standing tall south of the north wall's road,
        // turned to the bikes coming along it; a fire engine at the
        // north-east corner, facing the road; a crocodile monster truck by
        // the start straight, facing it as if to race; a doll's house
        // inside the last bend, its front to the home straight; and a
        // mechanical pegasus on the first table, by its hairpin.
        Toy {
            kind: ToyKind::Giraffe,
            position: Vec2::new(158.0, -262.0),
            heading: -2.4,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::FireTruck,
            position: Vec2::new(288.0, -260.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::CrocCar,
            position: Vec2::new(-18.0, -85.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Dollhouse,
            position: Vec2::new(95.0, 55.0),
            heading: 0.0,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Pegasus,
            position: Vec2::new(320.0, -178.0),
            heading: -2.0,
            scale: 1.0,
            base: 13.2,
        },
        // A tyrannosaurus beside the home straight, its head reaching out
        // over the road, well over the bikes; a rocket by the road to the
        // chair; a teepee by the start straight; a dump truck by the north
        // wall's road, facing the bikes coming; a toy chest by the start
        // straight, its open front to the road.
        Toy {
            kind: ToyKind::Trex,
            position: Vec2::new(45.0, 119.7),
            heading: PI,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Rocket,
            position: Vec2::new(95.0, -183.0),
            heading: PI,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Teepee,
            position: Vec2::new(-19.5, 62.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::DumpTruck,
            position: Vec2::new(205.0, -278.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::ToyChest,
            position: Vec2::new(19.0, 22.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        // A giant robot standing guard inside the shop's door, turned to
        // the bikes coming down the east side; a jack-in-the-box and a
        // xylophone on the floor by the start straight.
        Toy {
            kind: ToyKind::GiantRobot,
            position: Vec2::new(354.0, 98.0),
            heading: -2.6,
            scale: 1.0,
            base: 0.0,
        },
        // By the doors: the checkout counter east of them, its front with
        // the sweets to the room; a row of trolleys before the glass west of
        // them, nested; a potted plant at either end of the glass.
        Toy {
            kind: ToyKind::CheckoutCounter,
            position: Vec2::new(364.0, 116.0),
            heading: PI,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::ShoppingCart,
            position: Vec2::new(286.0, 123.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::ShoppingCart,
            position: Vec2::new(290.5, 123.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::ShoppingCart,
            position: Vec2::new(295.0, 123.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::PottedPlant,
            position: Vec2::new(122.0, 121.0),
            heading: 0.4,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::PottedPlant,
            position: Vec2::new(373.0, 122.0),
            heading: -0.7,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::JackInBox,
            position: Vec2::new(-17.0, -55.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Xylophone,
            position: Vec2::new(22.0, 2.0),
            heading: -FRAC_PI_2 + 0.25,
            scale: 1.0,
            base: 0.0,
        },
        // The floor train's station, its platform along the track, and
        // its tunnel through a mountain on the other straight.
        Toy {
            kind: ToyKind::Station,
            position: Vec2::new(198.0, -88.8),
            heading: 0.0,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::TunnelMountain,
            position: Vec2::new(198.0, -50.0),
            heading: FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        // Two nutcrackers on guard either side of the road where it comes
        // off the slide; a garage playset by the road down the east side;
        // a rocking horse by the start straight, side on to it.
        Toy {
            kind: ToyKind::Nutcracker,
            position: Vec2::new(-65.0, -313.0),
            heading: 0.0,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Nutcracker,
            position: Vec2::new(-65.0, -287.0),
            heading: PI,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Garage,
            position: Vec2::new(298.0, -105.0),
            heading: -FRAC_PI_2,
            scale: 1.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::RockingHorse,
            position: Vec2::new(-19.0, 4.0),
            heading: 0.0,
            scale: 1.0,
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
            position: Vec2::new(358.0, 106.0),
            heading: 0.4,
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
    bricks: &[
        // A village of little houses of bricks on the diorama, round its
        // castles, turned to the road.
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(-95.0, -108.0),
            heading: PI,
            base: 13.4,
        },
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(-25.0, -110.0),
            heading: PI,
            base: 13.4,
        },
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(50.0, -106.0),
            heading: PI,
            base: 13.4,
        },
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(82.0, -106.0),
            heading: PI,
            base: 13.4,
        },
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(150.0, -106.0),
            heading: PI,
            base: 13.4,
        },
        Bricks {
            kind: BrickKind::House {
                width: 10,
                depth: 6,
                courses: 4,
            },
            position: Vec2::new(183.0, -107.0),
            heading: PI,
            base: 13.4,
        },
        // Towers of bricks stacked crosswise by the start straight, on the
        // first table and on the train table.
        Bricks {
            kind: BrickKind::Tower { count: 9 },
            position: Vec2::new(15.0, -12.0),
            heading: 0.3,
            base: 0.0,
        },
        Bricks {
            kind: BrickKind::Tower { count: 7 },
            position: Vec2::new(300.0, -226.0),
            heading: 1.1,
            base: 13.2,
        },
        Bricks {
            kind: BrickKind::Tower { count: 6 },
            position: Vec2::new(200.0, 73.0),
            heading: -0.4,
            base: 8.2,
        },
        // Heaps of them spilled on the floor by the road: inside the
        // north-east corner, by the road to the chair, beside the start
        // straight and by the road down the east side.
        Bricks {
            kind: BrickKind::Heap { count: 14 },
            position: Vec2::new(242.0, -272.0),
            heading: 0.0,
            base: 0.0,
        },
        Bricks {
            kind: BrickKind::Heap { count: 14 },
            position: Vec2::new(60.0, -180.0),
            heading: 1.0,
            base: 0.0,
        },
        Bricks {
            kind: BrickKind::Heap { count: 14 },
            position: Vec2::new(-26.0, -62.0),
            heading: 2.0,
            base: 0.0,
        },
        Bricks {
            kind: BrickKind::Heap { count: 12 },
            position: Vec2::new(243.0, -10.0),
            heading: 0.5,
            base: 0.0,
        },
    ],
};

/// Where the branches leave the lap and join it again (m from the finish
/// line), as `tools/toy_store_layout.py` works them out: the way through
/// the castle, and the way round the loop.
const CASTLE: Span = span(730.1, 914.2);
const LOOP: Span = span(1918.4, 2164.4);

/// The way through the castle's control points, as the tool walks them.
const CASTLE_WAY: [Vec2; 31] = [
    Vec2::new(204.66, -142.0),
    Vec2::new(201.32, -142.0),
    Vec2::new(197.99, -142.0),
    Vec2::new(191.97, -141.82),
    Vec2::new(185.98, -141.28),
    Vec2::new(180.02, -140.37),
    Vec2::new(174.14, -139.11),
    Vec2::new(168.34, -137.5),
    Vec2::new(162.64, -135.54),
    Vec2::new(156.95, -133.59),
    Vec2::new(151.15, -131.98),
    Vec2::new(145.26, -130.72),
    Vec2::new(139.31, -129.81),
    Vec2::new(133.31, -129.27),
    Vec2::new(127.3, -129.09),
    Vec2::new(117.3, -129.09),
    Vec2::new(107.3, -129.09),
    Vec2::new(101.28, -129.27),
    Vec2::new(95.28, -129.81),
    Vec2::new(89.33, -130.72),
    Vec2::new(83.44, -131.98),
    Vec2::new(77.64, -133.59),
    Vec2::new(71.95, -135.54),
    Vec2::new(66.25, -137.5),
    Vec2::new(60.45, -139.11),
    Vec2::new(54.57, -140.37),
    Vec2::new(48.61, -141.28),
    Vec2::new(42.62, -141.82),
    Vec2::new(36.6, -142.0),
    Vec2::new(33.27, -142.0),
    Vec2::new(29.93, -142.0),
];

/// The way round the loop: the control points of the road into it, its
/// mouth (the bikes go in heading east) and the road out of it.
const LOOP_APPROACH: [Vec2; 10] = [
    Vec2::new(-40.0, -300.0),
    Vec2::new(-34.41, -299.92),
    Vec2::new(-28.82, -299.69),
    Vec2::new(-23.24, -299.3),
    Vec2::new(-17.67, -298.75),
    Vec2::new(-12.11, -298.2),
    Vec2::new(-6.53, -297.81),
    Vec2::new(-0.94, -297.58),
    Vec2::new(4.65, -297.5),
    Vec2::new(19.0, -297.5),
];
const LOOP_MOUTH: Vec2 = Vec2::new(83.3, -297.5);
const LOOP_HEADING: f32 = -FRAC_PI_2;
const LOOP_EXIT: [Vec2; 8] = [
    Vec2::new(139.85, -284.03),
    Vec2::new(146.19, -285.62),
    Vec2::new(152.18, -288.21),
    Vec2::new(157.67, -291.75),
    Vec2::new(163.16, -295.29),
    Vec2::new(169.15, -297.88),
    Vec2::new(175.49, -299.47),
    Vec2::new(182.0, -300.0),
];

/// The centre line's points, in race order, as `tools/toy_store_layout.py`
/// walks them.
const CONTROL_POINTS: [Vec2; 381] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -13.33),
    Vec2::new(0.0, -26.67),
    Vec2::new(0.0, -40.0),
    Vec2::new(0.0, -55.0),
    Vec2::new(0.0, -65.0),
    Vec2::new(0.0, -75.0),
    Vec2::new(0.0, -85.0),
    Vec2::new(0.0, -99.0),
    Vec2::new(0.0, -113.0),
    Vec2::new(0.0, -124.4),
    Vec2::new(0.0, -135.8),
    Vec2::new(0.0, -147.2),
    Vec2::new(0.0, -158.6),
    Vec2::new(0.0, -170.0),
    Vec2::new(0.61, -176.24),
    Vec2::new(2.44, -182.25),
    Vec2::new(5.39, -187.78),
    Vec2::new(9.37, -192.63),
    Vec2::new(14.22, -196.61),
    Vec2::new(19.75, -199.56),
    Vec2::new(25.76, -201.39),
    Vec2::new(32.0, -202.0),
    Vec2::new(40.0, -202.0),
    Vec2::new(50.0, -202.0),
    Vec2::new(62.5, -202.0),
    Vec2::new(75.0, -202.0),
    Vec2::new(87.5, -202.0),
    Vec2::new(100.0, -202.0),
    Vec2::new(110.0, -202.0),
    Vec2::new(120.0, -202.0),
    Vec2::new(128.0, -202.0),
    Vec2::new(140.0, -202.0),
    Vec2::new(152.0, -202.0),
    Vec2::new(164.0, -202.0),
    Vec2::new(176.0, -202.0),
    Vec2::new(188.0, -202.0),
    Vec2::new(202.0, -202.0),
    Vec2::new(214.0, -202.0),
    Vec2::new(226.0, -202.0),
    Vec2::new(232.08, -202.53),
    Vec2::new(237.97, -204.11),
    Vec2::new(243.5, -206.69),
    Vec2::new(248.5, -210.19),
    Vec2::new(253.5, -213.69),
    Vec2::new(259.02, -216.27),
    Vec2::new(264.92, -217.85),
    Vec2::new(271.0, -218.38),
    Vec2::new(277.07, -217.85),
    Vec2::new(282.97, -216.27),
    Vec2::new(288.5, -213.69),
    Vec2::new(293.49, -210.19),
    Vec2::new(298.49, -206.69),
    Vec2::new(304.02, -204.11),
    Vec2::new(309.91, -202.53),
    Vec2::new(315.99, -202.0),
    Vec2::new(323.99, -202.0),
    Vec2::new(329.84, -201.42),
    Vec2::new(335.47, -199.72),
    Vec2::new(340.66, -196.94),
    Vec2::new(345.2, -193.21),
    Vec2::new(348.93, -188.67),
    Vec2::new(351.71, -183.48),
    Vec2::new(353.41, -177.85),
    Vec2::new(353.99, -172.0),
    Vec2::new(353.41, -166.15),
    Vec2::new(351.71, -160.52),
    Vec2::new(348.93, -155.33),
    Vec2::new(345.2, -150.79),
    Vec2::new(340.66, -147.06),
    Vec2::new(335.47, -144.28),
    Vec2::new(329.84, -142.58),
    Vec2::new(323.99, -142.0),
    Vec2::new(310.66, -142.0),
    Vec2::new(297.32, -142.0),
    Vec2::new(283.99, -142.0),
    Vec2::new(271.99, -142.0),
    Vec2::new(259.99, -142.0),
    Vec2::new(247.99, -142.0),
    Vec2::new(234.66, -142.0),
    Vec2::new(221.32, -142.0),
    Vec2::new(207.99, -142.0),
    Vec2::new(197.99, -142.0),
    Vec2::new(191.97, -142.18),
    Vec2::new(185.98, -142.72),
    Vec2::new(180.02, -143.63),
    Vec2::new(174.14, -144.89),
    Vec2::new(168.34, -146.5),
    Vec2::new(162.64, -148.46),
    Vec2::new(156.95, -150.41),
    Vec2::new(151.15, -152.02),
    Vec2::new(145.26, -153.28),
    Vec2::new(139.31, -154.19),
    Vec2::new(133.31, -154.73),
    Vec2::new(127.3, -154.91),
    Vec2::new(117.3, -154.91),
    Vec2::new(107.3, -154.91),
    Vec2::new(101.28, -154.73),
    Vec2::new(95.28, -154.19),
    Vec2::new(89.33, -153.28),
    Vec2::new(83.44, -152.02),
    Vec2::new(77.64, -150.41),
    Vec2::new(71.95, -148.46),
    Vec2::new(66.25, -146.5),
    Vec2::new(60.45, -144.89),
    Vec2::new(54.57, -143.63),
    Vec2::new(48.61, -142.72),
    Vec2::new(42.62, -142.18),
    Vec2::new(36.6, -142.0),
    Vec2::new(26.6, -142.0),
    Vec2::new(19.63, -142.41),
    Vec2::new(12.76, -143.62),
    Vec2::new(6.08, -145.62),
    Vec2::new(0.36, -147.38),
    Vec2::new(-5.5, -148.57),
    Vec2::new(-11.45, -149.16),
    Vec2::new(-17.43, -149.16),
    Vec2::new(-23.38, -148.57),
    Vec2::new(-29.25, -147.38),
    Vec2::new(-34.96, -145.62),
    Vec2::new(-41.65, -143.62),
    Vec2::new(-48.52, -142.41),
    Vec2::new(-55.48, -142.0),
    Vec2::new(-70.0, -142.0),
    Vec2::new(-83.33, -142.0),
    Vec2::new(-96.67, -142.0),
    Vec2::new(-110.0, -142.0),
    Vec2::new(-116.25, -142.55),
    Vec2::new(-122.31, -144.17),
    Vec2::new(-128.0, -146.82),
    Vec2::new(-133.14, -150.42),
    Vec2::new(-137.58, -154.86),
    Vec2::new(-141.18, -160.0),
    Vec2::new(-143.83, -165.69),
    Vec2::new(-145.45, -171.75),
    Vec2::new(-146.0, -178.0),
    Vec2::new(-145.45, -184.25),
    Vec2::new(-143.83, -190.31),
    Vec2::new(-141.18, -196.0),
    Vec2::new(-137.58, -201.14),
    Vec2::new(-133.14, -205.58),
    Vec2::new(-128.0, -209.18),
    Vec2::new(-122.31, -211.83),
    Vec2::new(-116.25, -213.45),
    Vec2::new(-110.0, -214.0),
    Vec2::new(-103.75, -213.45),
    Vec2::new(-97.69, -211.83),
    Vec2::new(-92.0, -209.18),
    Vec2::new(-86.86, -205.58),
    Vec2::new(-82.42, -201.14),
    Vec2::new(-78.82, -196.0),
    Vec2::new(-76.17, -190.31),
    Vec2::new(-74.55, -184.25),
    Vec2::new(-74.0, -178.0),
    Vec2::new(-74.55, -171.75),
    Vec2::new(-76.17, -165.69),
    Vec2::new(-78.82, -160.0),
    Vec2::new(-82.42, -154.86),
    Vec2::new(-86.86, -150.42),
    Vec2::new(-92.0, -146.82),
    Vec2::new(-97.69, -144.17),
    Vec2::new(-103.75, -142.55),
    Vec2::new(-110.0, -142.0),
    Vec2::new(-116.25, -142.55),
    Vec2::new(-122.31, -144.17),
    Vec2::new(-128.0, -146.82),
    Vec2::new(-133.14, -150.42),
    Vec2::new(-137.58, -154.86),
    Vec2::new(-141.18, -160.0),
    Vec2::new(-143.83, -165.69),
    Vec2::new(-145.45, -171.75),
    Vec2::new(-146.0, -178.0),
    Vec2::new(-146.0, -193.0),
    Vec2::new(-146.0, -208.0),
    Vec2::new(-146.0, -218.67),
    Vec2::new(-146.0, -229.33),
    Vec2::new(-146.0, -240.0),
    Vec2::new(-146.0, -255.0),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-145.42, -275.85),
    Vec2::new(-143.72, -281.48),
    Vec2::new(-140.94, -286.67),
    Vec2::new(-137.21, -291.21),
    Vec2::new(-132.67, -294.94),
    Vec2::new(-127.48, -297.72),
    Vec2::new(-121.85, -299.42),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-110.15, -299.42),
    Vec2::new(-104.52, -297.72),
    Vec2::new(-99.33, -294.94),
    Vec2::new(-94.79, -291.21),
    Vec2::new(-91.06, -286.67),
    Vec2::new(-88.28, -281.48),
    Vec2::new(-86.58, -275.85),
    Vec2::new(-86.0, -270.0),
    Vec2::new(-86.58, -264.15),
    Vec2::new(-88.28, -258.52),
    Vec2::new(-91.06, -253.33),
    Vec2::new(-94.79, -248.79),
    Vec2::new(-99.33, -245.06),
    Vec2::new(-104.52, -242.28),
    Vec2::new(-110.15, -240.58),
    Vec2::new(-116.0, -240.0),
    Vec2::new(-121.85, -240.58),
    Vec2::new(-127.48, -242.28),
    Vec2::new(-132.67, -245.06),
    Vec2::new(-137.21, -248.79),
    Vec2::new(-140.94, -253.33),
    Vec2::new(-143.72, -258.52),
    Vec2::new(-145.42, -264.15),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-145.42, -275.85),
    Vec2::new(-143.72, -281.48),
    Vec2::new(-140.94, -286.67),
    Vec2::new(-137.21, -291.21),
    Vec2::new(-132.67, -294.94),
    Vec2::new(-127.48, -297.72),
    Vec2::new(-121.85, -299.42),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-110.15, -299.42),
    Vec2::new(-104.52, -297.72),
    Vec2::new(-99.33, -294.94),
    Vec2::new(-94.79, -291.21),
    Vec2::new(-91.06, -286.67),
    Vec2::new(-88.28, -281.48),
    Vec2::new(-86.58, -275.85),
    Vec2::new(-86.0, -270.0),
    Vec2::new(-86.58, -264.15),
    Vec2::new(-88.28, -258.52),
    Vec2::new(-91.06, -253.33),
    Vec2::new(-94.79, -248.79),
    Vec2::new(-99.33, -245.06),
    Vec2::new(-104.52, -242.28),
    Vec2::new(-110.15, -240.58),
    Vec2::new(-116.0, -240.0),
    Vec2::new(-121.85, -240.58),
    Vec2::new(-127.48, -242.28),
    Vec2::new(-132.67, -245.06),
    Vec2::new(-137.21, -248.79),
    Vec2::new(-140.94, -253.33),
    Vec2::new(-143.72, -258.52),
    Vec2::new(-145.42, -264.15),
    Vec2::new(-146.0, -270.0),
    Vec2::new(-145.42, -275.85),
    Vec2::new(-143.72, -281.48),
    Vec2::new(-140.94, -286.67),
    Vec2::new(-137.21, -291.21),
    Vec2::new(-132.67, -294.94),
    Vec2::new(-127.48, -297.72),
    Vec2::new(-121.85, -299.42),
    Vec2::new(-116.0, -300.0),
    Vec2::new(-101.0, -300.0),
    Vec2::new(-86.0, -300.0),
    Vec2::new(-74.0, -300.0),
    Vec2::new(-62.0, -300.0),
    Vec2::new(-50.0, -300.0),
    Vec2::new(-40.0, -300.0),
    Vec2::new(-33.99, -300.23),
    Vec2::new(-28.02, -300.9),
    Vec2::new(-22.11, -302.03),
    Vec2::new(-16.31, -303.59),
    Vec2::new(-10.63, -305.58),
    Vec2::new(-5.13, -308.0),
    Vec2::new(0.38, -310.42),
    Vec2::new(6.05, -312.41),
    Vec2::new(11.85, -313.97),
    Vec2::new(17.76, -315.1),
    Vec2::new(23.73, -315.77),
    Vec2::new(29.74, -316.0),
    Vec2::new(41.31, -316.0),
    Vec2::new(52.87, -316.0),
    Vec2::new(64.44, -316.0),
    Vec2::new(76.0, -316.0),
    Vec2::new(87.56, -316.0),
    Vec2::new(99.13, -316.0),
    Vec2::new(110.69, -316.0),
    Vec2::new(122.26, -316.0),
    Vec2::new(128.27, -315.77),
    Vec2::new(134.24, -315.1),
    Vec2::new(140.15, -313.97),
    Vec2::new(145.95, -312.41),
    Vec2::new(151.62, -310.42),
    Vec2::new(157.13, -308.0),
    Vec2::new(162.63, -305.58),
    Vec2::new(168.31, -303.59),
    Vec2::new(174.11, -302.03),
    Vec2::new(180.02, -300.9),
    Vec2::new(185.99, -300.23),
    Vec2::new(192.0, -300.0),
    Vec2::new(205.33, -300.0),
    Vec2::new(218.67, -300.0),
    Vec2::new(232.0, -300.0),
    Vec2::new(238.24, -299.39),
    Vec2::new(244.25, -297.56),
    Vec2::new(249.78, -294.61),
    Vec2::new(254.63, -290.63),
    Vec2::new(258.61, -285.78),
    Vec2::new(261.56, -280.25),
    Vec2::new(263.39, -274.24),
    Vec2::new(264.0, -268.0),
    Vec2::new(264.0, -255.5),
    Vec2::new(264.0, -243.0),
    Vec2::new(264.0, -230.5),
    Vec2::new(264.0, -218.0),
    Vec2::new(264.0, -205.5),
    Vec2::new(264.0, -193.0),
    Vec2::new(264.0, -180.5),
    Vec2::new(264.0, -168.0),
    Vec2::new(264.47, -161.92),
    Vec2::new(265.85, -155.97),
    Vec2::new(268.13, -150.31),
    Vec2::new(271.23, -145.06),
    Vec2::new(274.34, -139.81),
    Vec2::new(276.62, -134.14),
    Vec2::new(278.0, -128.2),
    Vec2::new(278.47, -122.11),
    Vec2::new(278.0, -116.03),
    Vec2::new(276.62, -110.09),
    Vec2::new(274.34, -104.42),
    Vec2::new(271.23, -99.17),
    Vec2::new(268.13, -93.92),
    Vec2::new(265.85, -88.26),
    Vec2::new(264.47, -82.31),
    Vec2::new(264.0, -76.23),
    Vec2::new(264.0, -66.23),
    Vec2::new(264.0, -53.56),
    Vec2::new(264.0, -40.89),
    Vec2::new(264.0, -28.23),
    Vec2::new(264.0, -15.56),
    Vec2::new(264.0, -2.89),
    Vec2::new(264.0, 9.77),
    Vec2::new(264.0, 21.77),
    Vec2::new(264.0, 33.77),
    Vec2::new(264.0, 45.77),
    Vec2::new(264.0, 57.77),
    Vec2::new(264.0, 69.77),
    Vec2::new(263.39, 76.02),
    Vec2::new(261.56, 82.02),
    Vec2::new(258.61, 87.55),
    Vec2::new(254.63, 92.4),
    Vec2::new(249.78, 96.38),
    Vec2::new(244.25, 99.34),
    Vec2::new(238.24, 101.16),
    Vec2::new(232.0, 101.77),
    Vec2::new(226.0, 101.77),
    Vec2::new(219.92, 102.3),
    Vec2::new(214.03, 103.88),
    Vec2::new(208.5, 106.46),
    Vec2::new(202.97, 109.04),
    Vec2::new(197.08, 110.62),
    Vec2::new(191.0, 111.15),
    Vec2::new(184.92, 110.62),
    Vec2::new(179.03, 109.04),
    Vec2::new(173.5, 106.46),
    Vec2::new(167.97, 103.88),
    Vec2::new(162.08, 102.3),
    Vec2::new(156.0, 101.77),
    Vec2::new(143.5, 101.77),
    Vec2::new(131.0, 101.77),
    Vec2::new(119.57, 101.77),
    Vec2::new(108.14, 101.77),
    Vec2::new(96.71, 101.77),
    Vec2::new(85.29, 101.77),
    Vec2::new(73.86, 101.77),
    Vec2::new(62.43, 101.77),
    Vec2::new(51.0, 101.77),
    Vec2::new(40.0, 101.77),
    Vec2::new(33.74, 101.28),
    Vec2::new(27.64, 99.81),
    Vec2::new(21.84, 97.41),
    Vec2::new(16.49, 94.13),
    Vec2::new(11.72, 90.06),
    Vec2::new(7.64, 85.28),
    Vec2::new(4.36, 79.93),
    Vec2::new(1.96, 74.13),
    Vec2::new(0.49, 68.03),
    Vec2::new(0.0, 61.77),
    Vec2::new(0.0, 49.42),
    Vec2::new(0.0, 37.06),
    Vec2::new(0.0, 24.71),
    Vec2::new(0.0, 12.35),
];

/// The road's height along the lap
/// (metres from the finish line, height over the floor), as `tools/toy_store_layout.py` works them out.
const ELEVATION: [(f32, f32); 75] = [
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
    (288.1, 5.5), // chair
    (308.1, 5.5), // chair end
    (316.1, 6.3),
    (376.1, 12.5), // T0
    (390.1, 13.4),
    (402.1, 13.85),
    (414.1, 14.0),
    (438.1, 14.0),
    (488.1, 14.0),
    (512.1, 14.0),
    (520.1, 14.0),
    (614.1, 14.0), // hairpin
    (654.1, 14.0), // T0 edge
    (690.1, 14.0), // T1 edge
    (730.1, 14.0),
    (822.2, 14.0), // castle
    (914.2, 14.0), // castle join
    (936.2, 14.0),
    (978.2, 14.0),
    (998.2, 14.0),
    (1012.2, 14.0),
    (1052.2, 14.0), // X
    (1110.2, 18.6),
    (1166.2, 23.2),
    (1222.2, 27.8),
    (1278.3, 32.4),
    (1336.3, 37.0), // helix top
    (1366.3, 39.0), // gondola
    (1398.3, 39.0), // shelves end
    (1428.3, 37.0), // slide
    (1474.3, 33.17),
    (1522.3, 29.33),
    (1568.3, 25.5),
    (1616.3, 21.67),
    (1662.3, 17.83),
    (1710.3, 14.0),
    (1758.4, 10.17),
    (1804.4, 6.33),
    (1852.4, 2.5),
    (1882.4, 0.3), // floor
    (1918.4, 0.0),
    (1928.4, 0.0), // loop fork
    (1964.4, 0.0),
    (2000.4, 0.0),
    (2092.4, 0.0),
    (2128.4, 0.0),
    (2164.4, 0.0), // loop join
    (2204.4, 0.0), // north wall end
    (2254.5, 0.0),
    (2354.5, 0.0), // under T0
    (2378.5, 0.0),
    (2428.5, 0.0),
    (2452.5, 0.0),
    (2462.5, 0.3),
    (2538.5, 8.4), // T2
    (2598.5, 9.0), // doors
    (2648.5, 9.0),
    (2654.5, 9.0),
    (2672.5, 9.0),
    (2710.5, 9.0),
    (2728.5, 9.0),
    (2752.6, 9.0), // drop
    (2832.6, 0.3), // drop end
    (2844.6, 0.0),
    (2906.6, 0.0), // home
];
