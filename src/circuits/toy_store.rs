//! Juguetería (Toy Store): a lap round the inside of a toy shop, the bikes
//! as small as toys, under a ceiling of lamps instead of a sky. It climbs
//! and drops all the way, over the furniture and along the toy tracks
//! (see [`crate::world::toys`]):
//!
//! 1. **The drag strip** (start straight, north): Hot Wheels' orange track
//!    on the floor, with a booster jump (about 75 m), and on under the
//!    diorama table the lap crosses on top later (160–240 m).
//! 2. **The chair and the first table** (about 390–940 m): a slot-car
//!    track up a ramp onto a chair's seat (5.5 m up, a jump onto it), on up
//!    another onto a table (14 m, a jump onto it), and round its far end.
//! 3. **From table to table** (about 940–1340 m): across the gap to the
//!    diorama table (15.5 m), its castles of plastic bricks on a green
//!    baseplate, over the start straight below, and round to its edge.
//! 4. **The Hot Wheels helix** (about 1340–2060 m): orange track, banked,
//!    climbing a turn and a half round a blue column to the top of a tall
//!    shelving unit (39 m), near the ceiling's lamps.
//! 5. **Along the top of the shelves** (about 2060–2220 m).
//! 6. **The slide** (about 2220–2800 m): a helter-skelter's chute, a turn
//!    and a quarter round its tower, down to the floor.
//! 7. **The floor** (about 2800–3480 m): along the north wall past the
//!    electric train going round on its rug, and south under the first
//!    table.
//! 8. **The train table** (about 3480–4030 m): up a ramp onto it (9 m),
//!    round past its own electric train, and straight off its edge back
//!    down to the floor.
//! 9. **Home** (about 4030 m to the finish): round onto the drag strip.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::{Banking, Layout, Narrowing};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line.
const DRAG_STRIP: Span = span(4258.0, 130.0);
const TABLES: Span = span(384.0, 1344.0);
const HELIX: Span = span(1344.0, 2078.0);
const SHELVES: Span = span(2078.0, 2218.0);
const SLIDE: Span = span(2218.0, 2798.0);
const TRAIN_TABLE: Span = span(3474.0, 4028.0);
/// Along the north wall, beside the train's rug.
const NORTH_WALL: Span = span(2808.0, 3178.0);

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
                  una espiral de Hot Wheels, un tobogán y trenes eléctricos",
    description_en: "A toy store: slot cars over a chair and the tables,\n\
                     a Hot Wheels spiral, a slide and electric trains",
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
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Level on the floor and on the furniture; banked round the helix
        // and down the slide.
        banking: &[
            Banking {
                from: 0.0,
                to: HELIX.from,
                factor: 0.0,
            },
            Banking {
                from: HELIX.from,
                to: HELIX.to,
                factor: 2.5,
            },
            Banking {
                from: SHELVES.from,
                to: SHELVES.to,
                factor: 0.0,
            },
            Banking {
                from: SLIDE.from,
                to: SLIDE.to,
                factor: 2.5,
            },
            Banking {
                from: SLIDE.to,
                to: f32::MAX,
                factor: 0.0,
            },
        ],
        // Narrower over the chair's seat.
        narrows: &[Narrowing {
            from: 452.0,
            to: 488.0,
            wall: 6.5,
        }],
        branches: &[],
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
        // Toy blocks along the floor by the train's rug, and elsewhere a
        // slot-car track's borders.
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
        // turn, and the floor under the first table, from the road on one
        // of its sides to the road on the other (one span, so that the
        // trackside camera stays down all the way under it).
        tunnels: &[
            span(158.0, 242.0),
            span(1282.0, 1654.0),
            span(2576.0, 2831.0),
            span(3285.0, 3509.0),
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
            // Over the diorama table, on top of the shelves and under the
            // first table.
            Gantry {
                along: 1220.0,
                color: HOT_WHEELS_YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2160.0,
                color: HOT_WHEELS_YELLOW,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3410.0,
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

const STORE: ToyStore = ToyStore {
    room: (Vec2::new(-320.0, -500.0), Vec2::new(450.0, 340.0)),
    ceiling: 55.0,
    furniture: &[
        // The chair the road climbs onto from the floor, its back to the
        // north.
        Furniture {
            kind: FurnitureKind::Chair,
            center: Vec2::new(160.0, -340.0),
            size: Vec2::new(44.0, 20.0),
            heading: 0.0,
            top: 5.1,
            color: CHAIR_RED,
        },
        // The first table, round whose far end the road turns back, the
        // floor road running under it between its legs.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(341.0, -270.0),
            size: Vec2::new(138.0, 172.0),
            heading: 0.0,
            top: 13.6,
            color: FELT_GREEN,
        },
        // The diorama, over the start straight.
        Furniture {
            kind: FurnitureKind::Diorama,
            center: Vec2::new(22.0, -183.5),
            size: Vec2::new(306.0, 97.0),
            heading: 0.0,
            top: 15.1,
            color: CHAIR_RED,
        },
        // The tall shelving unit the road runs along the top of.
        Furniture {
            kind: FurnitureKind::Shelves,
            center: Vec2::new(-255.0, -248.5),
            size: Vec2::new(28.0, 137.0),
            heading: 0.0,
            top: 38.6,
            color: SHELF_WHITE,
        },
        // The train table, the road round it and straight off its edge.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(228.0, 25.0),
            size: Vec2::new(228.0, 200.0),
            heading: 0.0,
            top: 8.6,
            color: TABLE_YELLOW,
        },
        // A low table of blue sea, a pirate ship on it.
        Furniture {
            kind: FurnitureKind::Table,
            center: Vec2::new(62.0, 0.0),
            size: Vec2::new(74.0, 120.0),
            heading: 0.0,
            top: 4.0,
            color: SEA_BLUE,
        },
        // An aisle of shelving units in the south-east.
        Furniture {
            kind: FurnitureKind::Shelves,
            center: Vec2::new(295.0, 185.0),
            size: Vec2::new(230.0, 24.0),
            heading: 0.0,
            top: 26.0,
            color: SHELF_WHITE,
        },
        Furniture {
            kind: FurnitureKind::Shelves,
            center: Vec2::new(295.0, 265.0),
            size: Vec2::new(230.0, 24.0),
            heading: 0.0,
            top: 26.0,
            color: SHELF_WHITE,
        },
    ],
    towers: &[
        Tower {
            kind: TowerKind::Column,
            center: Vec2::new(-185.0, -115.0),
            radius: 8.0,
            height: 41.0,
        },
        Tower {
            kind: TowerKind::HelterSkelter,
            center: Vec2::new(-193.0, -365.0),
            radius: 12.0,
            height: 46.0,
        },
    ],
    trains: &[
        // Along the north wall, on its rug.
        Train {
            track: &[
                Vec2::new(-190.0, -476.0),
                Vec2::new(50.0, -476.0),
                Vec2::new(290.0, -476.0),
                Vec2::new(302.0, -468.0),
                Vec2::new(290.0, -460.0),
                Vec2::new(50.0, -460.0),
                Vec2::new(-190.0, -460.0),
                Vec2::new(-202.0, -468.0),
            ],
            height: 0.0,
            wagons: 6,
            speed: 14.0,
        },
        // Round the train table, beside the road.
        Train {
            track: &[
                Vec2::new(130.0, -55.0),
                Vec2::new(210.0, -55.0),
                Vec2::new(290.0, -55.0),
                Vec2::new(302.0, -33.0),
                Vec2::new(290.0, -11.0),
                Vec2::new(210.0, -11.0),
                Vec2::new(130.0, -11.0),
                Vec2::new(118.0, -33.0),
            ],
            height: 8.6,
            wagons: 4,
            speed: 9.0,
        },
    ],
    castles: &[
        Castle {
            center: Vec2::new(110.0, -156.0),
            size: Vec2::new(46.0, 34.0),
            heading: 0.0,
            base: 15.1,
        },
        Castle {
            center: Vec2::new(30.0, -158.0),
            size: Vec2::new(40.0, 30.0),
            heading: 0.0,
            base: 15.1,
        },
        Castle {
            center: Vec2::new(-62.0, -162.0),
            size: Vec2::new(26.0, 26.0),
            heading: 0.0,
            base: 15.1,
        },
    ],
    loops: &[
        ToyLoop {
            center: Vec2::new(45.0, -100.0),
            heading: 0.0,
            radius: 16.0,
            base: 0.0,
        },
        ToyLoop {
            center: Vec2::new(-100.0, 60.0),
            heading: 0.5,
            radius: 20.0,
            base: 0.0,
        },
    ],
    mats: &[
        Mat {
            center: Vec2::new(-160.0, 160.0),
            size: Vec2::new(220.0, 220.0),
            heading: 0.0,
            kind: MatKind::Foam,
        },
        Mat {
            center: Vec2::new(45.0, -468.0),
            size: Vec2::new(520.0, 36.0),
            heading: 0.0,
            kind: MatKind::Rug(Color::srgb(0.45, 0.62, 0.86)),
        },
    ],
    toys: &[
        Toy {
            kind: ToyKind::Bear,
            position: Vec2::new(-200.0, 230.0),
            heading: 0.6,
            scale: 14.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Blocks,
            position: Vec2::new(-110.0, 120.0),
            heading: 0.3,
            scale: 6.0,
            base: 0.0,
        },
        // In the middle of the U-turn home.
        Toy {
            kind: ToyKind::Blocks,
            position: Vec2::new(65.0, 230.0),
            heading: -0.4,
            scale: 6.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Ball,
            position: Vec2::new(-60.0, 40.0),
            heading: 0.0,
            scale: 8.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Ball,
            position: Vec2::new(240.0, -105.0),
            heading: 1.0,
            scale: 7.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Duck,
            position: Vec2::new(-170.0, 120.0),
            heading: 2.0,
            scale: 7.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Robot,
            position: Vec2::new(-285.0, 40.0),
            heading: -1.57,
            scale: 12.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Top,
            position: Vec2::new(-130.0, 260.0),
            heading: 0.0,
            scale: 6.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Ship,
            position: Vec2::new(62.0, 0.0),
            heading: 0.9,
            scale: 26.0,
            base: 4.0,
        },
        // Hanging from the ceiling, over the floor clear of the high roads.
        Toy {
            kind: ToyKind::Plane,
            position: Vec2::new(120.0, -90.0),
            heading: 2.4,
            scale: 16.0,
            base: 47.0,
        },
        Toy {
            kind: ToyKind::Plane,
            position: Vec2::new(230.0, 100.0),
            heading: -0.6,
            scale: 14.0,
            base: 46.0,
        },
        Toy {
            kind: ToyKind::Helicopter,
            position: Vec2::new(-120.0, 150.0),
            heading: 1.2,
            scale: 14.0,
            base: 44.0,
        },
        // Tyres round the outside of the floor's corners.
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(-22.0, -320.0),
            heading: 0.8,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(65.0, 312.0),
            heading: 0.0,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(345.0, -445.0),
            heading: -0.8,
            scale: 3.0,
            base: 0.0,
        },
        Toy {
            kind: ToyKind::Tyres,
            position: Vec2::new(22.0, 40.0),
            heading: 1.57,
            scale: 3.0,
            base: 0.0,
        },
    ],
};

/// The centre line's points, in race order, as `tools/toy_store_layout.py`
/// walks them.
const CONTROL_POINTS: [Vec2; 281] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -20.0),
    Vec2::new(0.0, -40.0),
    Vec2::new(0.0, -55.0),
    Vec2::new(0.0, -75.0),
    Vec2::new(0.0, -85.0),
    Vec2::new(0.0, -99.0),
    Vec2::new(0.0, -113.0),
    Vec2::new(0.0, -135.4),
    Vec2::new(0.0, -157.9),
    Vec2::new(0.0, -180.3),
    Vec2::new(0.0, -202.7),
    Vec2::new(0.0, -225.1),
    Vec2::new(0.0, -247.6),
    Vec2::new(0.0, -270.0),
    Vec2::new(1.1, -282.2),
    Vec2::new(4.2, -293.9),
    Vec2::new(9.4, -305.0),
    Vec2::new(16.4, -315.0),
    Vec2::new(25.0, -323.6),
    Vec2::new(35.0, -330.6),
    Vec2::new(46.1, -335.8),
    Vec2::new(57.8, -338.9),
    Vec2::new(70.0, -340.0),
    Vec2::new(80.0, -340.0),
    Vec2::new(92.0, -340.0),
    Vec2::new(117.0, -340.0),
    Vec2::new(142.0, -340.0),
    Vec2::new(160.0, -340.0),
    Vec2::new(178.0, -340.0),
    Vec2::new(190.0, -340.0),
    Vec2::new(211.3, -340.0),
    Vec2::new(232.7, -340.0),
    Vec2::new(254.0, -340.0),
    Vec2::new(276.0, -340.0),
    Vec2::new(301.0, -340.0),
    Vec2::new(326.0, -340.0),
    Vec2::new(338.2, -338.9),
    Vec2::new(349.9, -335.8),
    Vec2::new(361.0, -330.6),
    Vec2::new(371.0, -323.6),
    Vec2::new(379.6, -315.0),
    Vec2::new(386.6, -305.0),
    Vec2::new(391.8, -293.9),
    Vec2::new(394.9, -282.2),
    Vec2::new(396.0, -270.0),
    Vec2::new(394.9, -257.8),
    Vec2::new(391.8, -246.1),
    Vec2::new(386.6, -235.0),
    Vec2::new(379.6, -225.0),
    Vec2::new(371.0, -216.4),
    Vec2::new(361.0, -209.4),
    Vec2::new(349.9, -204.2),
    Vec2::new(338.2, -201.1),
    Vec2::new(326.0, -200.0),
    Vec2::new(304.0, -200.0),
    Vec2::new(282.0, -200.0),
    Vec2::new(260.0, -200.0),
    Vec2::new(238.0, -200.0),
    Vec2::new(217.0, -200.0),
    Vec2::new(196.0, -200.0),
    Vec2::new(175.0, -200.0),
    Vec2::new(150.0, -200.0),
    Vec2::new(125.0, -200.0),
    Vec2::new(100.0, -200.0),
    Vec2::new(75.0, -200.0),
    Vec2::new(50.0, -200.0),
    Vec2::new(25.0, -200.0),
    Vec2::new(0.0, -200.0),
    Vec2::new(-25.0, -200.0),
    Vec2::new(-50.0, -200.0),
    Vec2::new(-61.3, -199.0),
    Vec2::new(-72.2, -196.1),
    Vec2::new(-82.5, -191.3),
    Vec2::new(-91.8, -184.8),
    Vec2::new(-99.8, -176.8),
    Vec2::new(-106.3, -167.5),
    Vec2::new(-111.1, -157.2),
    Vec2::new(-114.0, -146.3),
    Vec2::new(-115.0, -135.0),
    Vec2::new(-115.0, -125.0),
    Vec2::new(-115.0, -115.0),
    Vec2::new(-116.1, -102.8),
    Vec2::new(-119.2, -91.1),
    Vec2::new(-124.4, -80.0),
    Vec2::new(-131.4, -70.0),
    Vec2::new(-140.0, -61.4),
    Vec2::new(-150.0, -54.4),
    Vec2::new(-161.1, -49.2),
    Vec2::new(-172.8, -46.1),
    Vec2::new(-185.0, -45.0),
    Vec2::new(-197.2, -46.1),
    Vec2::new(-208.9, -49.2),
    Vec2::new(-220.0, -54.4),
    Vec2::new(-230.0, -61.4),
    Vec2::new(-238.6, -70.0),
    Vec2::new(-245.6, -80.0),
    Vec2::new(-250.8, -91.1),
    Vec2::new(-253.9, -102.8),
    Vec2::new(-255.0, -115.0),
    Vec2::new(-253.9, -127.2),
    Vec2::new(-250.8, -138.9),
    Vec2::new(-245.6, -150.0),
    Vec2::new(-238.6, -160.0),
    Vec2::new(-230.0, -168.6),
    Vec2::new(-220.0, -175.6),
    Vec2::new(-208.9, -180.8),
    Vec2::new(-197.2, -183.9),
    Vec2::new(-185.0, -185.0),
    Vec2::new(-172.8, -183.9),
    Vec2::new(-161.1, -180.8),
    Vec2::new(-150.0, -175.6),
    Vec2::new(-140.0, -168.6),
    Vec2::new(-131.4, -160.0),
    Vec2::new(-124.4, -150.0),
    Vec2::new(-119.2, -138.9),
    Vec2::new(-116.1, -127.2),
    Vec2::new(-115.0, -115.0),
    Vec2::new(-116.1, -102.8),
    Vec2::new(-119.2, -91.1),
    Vec2::new(-124.4, -80.0),
    Vec2::new(-131.4, -70.0),
    Vec2::new(-140.0, -61.4),
    Vec2::new(-150.0, -54.4),
    Vec2::new(-161.1, -49.2),
    Vec2::new(-172.8, -46.1),
    Vec2::new(-185.0, -45.0),
    Vec2::new(-197.2, -46.1),
    Vec2::new(-208.9, -49.2),
    Vec2::new(-220.0, -54.4),
    Vec2::new(-230.0, -61.4),
    Vec2::new(-238.6, -70.0),
    Vec2::new(-245.6, -80.0),
    Vec2::new(-250.8, -91.1),
    Vec2::new(-253.9, -102.8),
    Vec2::new(-255.0, -115.0),
    Vec2::new(-255.0, -135.0),
    Vec2::new(-255.0, -155.0),
    Vec2::new(-255.0, -179.0),
    Vec2::new(-255.0, -201.7),
    Vec2::new(-255.0, -224.3),
    Vec2::new(-255.0, -247.0),
    Vec2::new(-255.0, -269.7),
    Vec2::new(-255.0, -292.3),
    Vec2::new(-255.0, -315.0),
    Vec2::new(-255.0, -340.0),
    Vec2::new(-255.0, -365.0),
    Vec2::new(-253.8, -377.1),
    Vec2::new(-250.3, -388.7),
    Vec2::new(-244.6, -399.4),
    Vec2::new(-236.8, -408.8),
    Vec2::new(-227.4, -416.6),
    Vec2::new(-216.7, -422.3),
    Vec2::new(-205.1, -425.8),
    Vec2::new(-193.0, -427.0),
    Vec2::new(-180.9, -425.8),
    Vec2::new(-169.3, -422.3),
    Vec2::new(-158.6, -416.6),
    Vec2::new(-149.2, -408.8),
    Vec2::new(-141.4, -399.4),
    Vec2::new(-135.7, -388.7),
    Vec2::new(-132.2, -377.1),
    Vec2::new(-131.0, -365.0),
    Vec2::new(-132.2, -352.9),
    Vec2::new(-135.7, -341.3),
    Vec2::new(-141.4, -330.6),
    Vec2::new(-149.2, -321.2),
    Vec2::new(-158.6, -313.4),
    Vec2::new(-169.3, -307.7),
    Vec2::new(-180.9, -304.2),
    Vec2::new(-193.0, -303.0),
    Vec2::new(-205.1, -304.2),
    Vec2::new(-216.7, -307.7),
    Vec2::new(-227.4, -313.4),
    Vec2::new(-236.8, -321.2),
    Vec2::new(-244.6, -330.6),
    Vec2::new(-250.3, -341.3),
    Vec2::new(-253.8, -352.9),
    Vec2::new(-255.0, -365.0),
    Vec2::new(-253.8, -377.1),
    Vec2::new(-250.3, -388.7),
    Vec2::new(-244.6, -399.4),
    Vec2::new(-236.8, -408.8),
    Vec2::new(-227.4, -416.6),
    Vec2::new(-216.7, -422.3),
    Vec2::new(-205.1, -425.8),
    Vec2::new(-193.0, -427.0),
    Vec2::new(-173.0, -427.0),
    Vec2::new(-153.0, -427.0),
    Vec2::new(-133.0, -427.0),
    Vec2::new(-108.7, -427.0),
    Vec2::new(-84.4, -427.0),
    Vec2::new(-60.1, -427.0),
    Vec2::new(-35.8, -427.0),
    Vec2::new(-11.4, -427.0),
    Vec2::new(12.9, -427.0),
    Vec2::new(37.2, -427.0),
    Vec2::new(61.5, -427.0),
    Vec2::new(85.8, -427.0),
    Vec2::new(110.1, -427.0),
    Vec2::new(134.4, -427.0),
    Vec2::new(158.8, -427.0),
    Vec2::new(183.1, -427.0),
    Vec2::new(207.4, -427.0),
    Vec2::new(231.7, -427.0),
    Vec2::new(256.0, -427.0),
    Vec2::new(268.2, -425.9),
    Vec2::new(279.9, -422.8),
    Vec2::new(291.0, -417.6),
    Vec2::new(301.0, -410.6),
    Vec2::new(309.6, -402.0),
    Vec2::new(316.6, -392.0),
    Vec2::new(321.8, -380.9),
    Vec2::new(324.9, -369.2),
    Vec2::new(326.0, -357.0),
    Vec2::new(326.0, -332.7),
    Vec2::new(326.0, -308.4),
    Vec2::new(326.0, -284.1),
    Vec2::new(326.0, -259.9),
    Vec2::new(326.0, -235.6),
    Vec2::new(326.0, -211.3),
    Vec2::new(326.0, -187.0),
    Vec2::new(326.0, -175.0),
    Vec2::new(326.0, -153.0),
    Vec2::new(326.0, -131.0),
    Vec2::new(326.0, -109.0),
    Vec2::new(326.0, -75.0),
    Vec2::new(326.0, -45.0),
    Vec2::new(325.0, -33.7),
    Vec2::new(322.1, -22.8),
    Vec2::new(317.3, -12.5),
    Vec2::new(310.8, -3.2),
    Vec2::new(302.8, 4.8),
    Vec2::new(293.5, 11.3),
    Vec2::new(283.2, 16.1),
    Vec2::new(272.3, 19.0),
    Vec2::new(261.0, 20.0),
    Vec2::new(239.0, 20.0),
    Vec2::new(217.0, 20.0),
    Vec2::new(195.0, 20.0),
    Vec2::new(183.7, 21.0),
    Vec2::new(172.8, 23.9),
    Vec2::new(162.5, 28.7),
    Vec2::new(153.2, 35.2),
    Vec2::new(145.2, 43.2),
    Vec2::new(138.7, 52.5),
    Vec2::new(133.9, 62.8),
    Vec2::new(131.0, 73.7),
    Vec2::new(130.0, 85.0),
    Vec2::new(130.0, 105.0),
    Vec2::new(130.0, 125.0),
    Vec2::new(130.0, 147.5),
    Vec2::new(130.0, 170.0),
    Vec2::new(130.0, 192.5),
    Vec2::new(130.0, 215.0),
    Vec2::new(130.0, 227.0),
    Vec2::new(128.9, 238.9),
    Vec2::new(125.6, 250.5),
    Vec2::new(120.3, 261.2),
    Vec2::new(113.0, 270.8),
    Vec2::new(104.2, 278.9),
    Vec2::new(94.0, 285.2),
    Vec2::new(82.8, 289.5),
    Vec2::new(71.0, 291.7),
    Vec2::new(59.0, 291.7),
    Vec2::new(47.2, 289.5),
    Vec2::new(36.0, 285.2),
    Vec2::new(25.8, 278.9),
    Vec2::new(17.0, 270.8),
    Vec2::new(9.7, 261.2),
    Vec2::new(4.4, 250.5),
    Vec2::new(1.1, 238.9),
    Vec2::new(0.0, 227.0),
    Vec2::new(0.0, 201.8),
    Vec2::new(0.0, 176.6),
    Vec2::new(0.0, 151.3),
    Vec2::new(0.0, 126.1),
    Vec2::new(0.0, 100.9),
    Vec2::new(0.0, 75.7),
    Vec2::new(0.0, 50.4),
    Vec2::new(0.0, 25.2),
];

/// (metres from the finish line, height over the floor), as
/// `tools/toy_store_layout.py` works them out.
const ELEVATION: [(f32, f32); 54] = [
    (0.0, 0.0), // start
    (40.0, 0.0),
    (54.0, 0.05),
    (74.0, 0.9), // kicker
    (84.0, 1.2),
    (98.0, 0.1),
    (112.0, 0.0),
    (270.1, 0.0), // north
    (390.1, 0.0), // ramp
    (402.1, 0.4),
    (452.1, 5.5), // chair
    (488.1, 5.5), // chair end
    (500.1, 6.3),
    (564.1, 13.4), // T0
    (586.1, 14.0),
    (636.1, 14.0),
    (856.2, 14.0),  // T0 back
    (944.2, 14.0),  // T0 edge
    (1006.2, 15.5), // T1 edge
    (1232.2, 15.5), // X
    (1334.3, 15.5),
    (1344.3, 15.5), // T1 south
    (1354.3, 15.5), // HW
    (1464.3, 19.25),
    (1574.3, 23.0),
    (1684.3, 26.75),
    (1794.3, 30.5),
    (1904.4, 34.25),
    (2014.4, 38.0), // helix top
    (2054.4, 39.0), // gondola
    (2078.4, 39.0), // shelves
    (2214.4, 39.0), // shelves end
    (2264.4, 35.5), // slide
    (2360.5, 29.0),
    (2458.5, 22.5),
    (2556.5, 16.0),
    (2652.5, 9.5),
    (2750.5, 3.0),
    (2790.5, 0.3), // floor
    (2810.5, 0.0),
    (3198.6, 0.0), // north leg end
    (3308.6, 0.0),
    (3478.7, 0.0), // under T0
    (3490.7, 0.4),
    (3556.7, 7.7),
    (3590.7, 9.0), // T2
    (3620.7, 9.0),
    (3722.7, 9.0),
    (3788.7, 9.0),
    (3890.7, 9.0),
    (3930.8, 9.0), // drop
    (4020.8, 0.3), // drop end
    (4032.8, 0.0),
    (4238.8, 0.0), // home
];
