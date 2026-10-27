//! Juguetería (Toy Shop): an indoor circuit inside a giant toy store and hobby
//! workshop where the motorcycles feel like miniature toy bikes racing across
//! the parquet floor, up wooden chairs onto hobby tables, through a Hot Wheels
//! vertical loop, across a Scalextric slot-car tabletop, beside animated
//! electric model trains, through a Playmobil medieval castle gate, along high
//! `Kallax` wooden cube shelves, and down a bright playground slide (`tobogán`).
//!
//! 1. **Shop Floor, Red Carpet & Hot Wheels Dino Loop (`0..800 m`):** Riders
//!    launch down the polished honey-oak parquet floor between rows of giant
//!    wooden Nutcracker soldiers (`photo 2`), painted hopscotch courts
//!    (`rayuela`, `photo 3`), and a giant `Big`-style floor piano, passing
//!    under a blue-and-yellow **5-Lane Hot Wheels Start Gate**. At `110 m` the
//!    road splits: the main line climbs past a giant wooden chair while the
//!    right-hand Hot Wheels stunt branch shoots through the open jaws of a
//!    giant green **T-Rex / Dragon head (`photo 5`)** into a 16 m
//!    **360° Vertical Loop (`VerticalLoop`)** before diving into the **Giant
//!    Toy Box Tunnel (`800..1020 m`)**.
//! 2. **Scalextric Tabletop, Electric Train & Xylophone Bridge (`1020..2250 m`):**
//!    Emerging from the toy box, the road climbs past a giant wooden chair up
//!    onto the green felt **Scalextric / Ninco Tabletop (`photo 4`)**, with
//!    braided copper contact strips and slot-car guide rails embedded in the
//!    asphalt, arched Scalextric overpass bridges, and 1:32 slot cars, while an
//!    animated **Electric Model Train (`Tren Eléctrico`)** chugs along railway
//!    tracks beside the road. At `1470..1590 m`, bikes cross a narrow
//!    **Xylophone & LEGO Block Bridge** over the toy-boat pond and sweep around
//!    the tabletop hairpin.
//! 3. **Playmobil Castle Gate & Board-Game Split (`2250..2895 m`):** Riders
//!    charge through a grand **Playmobil Medieval Castle Gatehouse** ("Puerta
//!    de castillo de Playmobil") with twin crenellated stone towers, conical
//!    red roofs, a raised portcullis arch and Playmobil knights, then choose
//!    between two paths through the **Castle & Board-Game Split
//!    (`2340.2..2690.3 m`)** flanked by cardboard castle turrets (`photo 3`)
//!    and stacks of giant board-game boxes.
//! 4. **High Kallax Cube Shelves & Playground Slide (`Tobogán`, `2895..4742 m`):**
//!    Climbing to the highest tier of the shop (`42.5..45.5 m`, crossing `45.1 m`
//!    over the Toy Box tunnel), the road runs along birch **`Kallax` wooden
//!    cube shelves (`photo 1`)** stocked with giant Matryoshka dolls, wooden
//!    robots, spotted rocking horses and toy pirate ships, before plunging
//!    **`-40.0 m`** down a glossy yellow-and-blue **Playground Slide
//!    (`Tobogán`, `3120..3555 m`)**, climbing one last Hot Wheels tabletop
//!    launch ramp (`4095 m`), and diving back to the shop floor finish line.

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, VerticalLoop, WALL_DISTANCE};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line.
const START_FLOOR: Span = span(4280.0, 310.0);
const LOOP_BRANCH: Span = span(110.0, 637.0);
const TOY_BOX_TUNNEL: Span = span(800.0, 1020.0);
const SCALEXTRIC_TABLE: Span = span(1040.0, 2260.0);
const XYLOPHONE_BRIDGE: Span = span(1470.0, 1590.0);
const TABLE_TURN: Span = span(1920.0, 2290.0);
const CASTLE_SPLIT: (f32, f32) = (2340.2, 2690.3);
const HIGH_SHELF: Span = span(2720.0, 3140.0);
const SLIDE_PLUNGE: Span = span(3120.0, 3555.0);
const HOT_WHEELS_RUN: Span = span(3600.0, 4460.0);

/// Toy-shop curb & sign colours: Hot Wheels flame orange, Scalextric red,
/// Playmobil royal blue, and LEGO yellow.
const HW_ORANGE: Color = Color::srgb(0.99, 0.42, 0.04);
const SCALEXTRIC_RED: Color = Color::srgb(0.88, 0.14, 0.16);
const PLAYMOBIL_BLUE: Color = Color::srgb(0.12, 0.44, 0.88);
const LEGO_YELLOW: Color = Color::srgb(0.98, 0.80, 0.10);
const WHITE: Color = Color::srgb(0.96, 0.96, 0.98);

pub const TOY_SHOP: Circuit = Circuit {
    name: "Juguetería",
    name_en: "Toy Shop",
    description: "Interior de una juguetería sin cielo bajo luces fluorescentes:\n\
                  Scalextric, looping Hot Wheels, tren eléctrico, puerta Playmobil y tobogán",
    description_en: "Indoor toy store under fluorescent ceiling lights:\n\
                     Scalextric table, Hot Wheels loop, electric train, Playmobil gate and slide",
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.0,
        cicadas: 0.0,
        wind: 0.12,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/orange_asphalt.mp3"),
        length: 168.7,
        loudness: -14.6,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[
            // Banked Scalextric tabletop hairpin curve.
            Banking {
                from: TABLE_TURN.from,
                to: TABLE_TURN.to,
                factor: 2.2,
            },
            // Level across the Playmobil Castle Split so the branch joins smoothly.
            Banking {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                factor: 0.25,
            },
            // Banked final Hot Wheels corner back onto the shop floor.
            Banking {
                from: 4340.0,
                to: 4540.0,
                factor: 1.5,
            },
        ],
        narrows: &[],
        branches: &[
            // 1. Hot Wheels 360° Vertical Loop branch through the T-Rex jaws.
            Branch {
                from: LOOP_BRANCH.from,
                to: LOOP_BRANCH.to,
                approach: &LOOP_APPROACH,
                vertical_loop: Some(VerticalLoop {
                    mouth: Vec2::new(82.2, -258.5),
                    heading: -FRAC_PI_4,
                    height: 5.5,
                    radius: 16.0,
                    shift: -16.0,
                }),
                exit: &LOOP_EXIT,
                rise: &[],
                narrows: &[],
            },
            // 2. Playmobil Castle & Board-Game alternative left branch.
            Branch {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                approach: &CASTLE_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &CASTLE_LEFT_RISE,
                narrows: &[],
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 16.0,
        soil: Soil::ToyShop,
        greens: Greens::Lawn,
        grass_shade: 0.9,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [SCALEXTRIC_RED, LEGO_YELLOW],
        walls: &[
            WallRun {
                span: START_FLOOR,
                side: Side::Left,
                style: WallStyle::Checkered,
            },
            WallRun {
                span: START_FLOOR,
                side: Side::Right,
                style: WallStyle::Checkered,
            },
            WallRun {
                span: HIGH_SHELF,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: HIGH_SHELF,
                side: Side::Right,
                style: WallStyle::Timber,
            },
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Concrete,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Concrete,
            },
        ],
        tunnels: &[TOY_BOX_TUNNEL],
        bridges: &[Bridge {
            span: XYLOPHONE_BRIDGE,
            style: BridgeStyle::Wooden,
            fence: WOODEN_FENCE,
        }],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: SCALEXTRIC_RED,
                text: Some("JUGUETERIA"),
            },
            Gantry {
                along: 1090.0,
                color: PLAYMOBIL_BLUE,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2240.0,
                color: HW_ORANGE,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3450.0,
                color: SCALEXTRIC_RED,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 95.0,
                side: Side::Right,
                text: "DINO LOOP",
                color: HW_ORANGE,
                accent: Some((4, LEGO_YELLOW)),
            },
            Billboard {
                along: 440.0,
                side: Side::Left,
                text: "HOT WHEELS",
                color: PLAYMOBIL_BLUE,
                accent: Some((3, HW_ORANGE)),
            },
            Billboard {
                along: 760.0,
                side: Side::Left,
                text: "TOY BOX",
                color: SCALEXTRIC_RED,
                accent: Some((3, LEGO_YELLOW)),
            },
            Billboard {
                along: 1140.0,
                side: Side::Right,
                text: "SCALEXTRIC",
                color: Color::srgb(0.14, 0.48, 0.24),
                accent: Some((5, WHITE)),
            },
            Billboard {
                along: 1445.0,
                side: Side::Left,
                text: "XYLOPHONE",
                color: PLAYMOBIL_BLUE,
                accent: Some((4, LEGO_YELLOW)),
            },
            Billboard {
                along: 2295.0,
                side: Side::Left,
                text: "PLAYMOBIL",
                color: PLAYMOBIL_BLUE,
                accent: Some((4, SCALEXTRIC_RED)),
            },
            Billboard {
                along: 3105.0,
                side: Side::Right,
                text: "TOBOGAN",
                color: HW_ORANGE,
                accent: None,
            },
        ],
        chevrons: &[
            ChevronSign {
                along: 340.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 410.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1220.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1290.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1980.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 2060.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 2140.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 3200.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 3280.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 3700.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 3780.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 4350.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 4430.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
        ],
        brake_markers: &[
            BrakeMarker {
                along: 220.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 270.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 1820.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 1870.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 4220.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 4270.0,
                side: Side::Right,
                distance: 50,
            },
        ],
        lamps: &[],
        // Indoor circuit: no outdoor trees or cottages.
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
        grandstands: &[
            Grandstand {
                along: 40.0,
                side: Side::Left,
                lateral: WALL_DISTANCE + 15.0,
                length: 60.0,
            },
            Grandstand {
                along: 2100.0,
                side: Side::Right,
                lateral: WALL_DISTANCE + 15.0,
                length: 52.0,
            },
        ],
        lighthouses: &[],
        buoys: &[],
        boats: &[],
        beach_parasols: &[],
        totems: &[],
        sunflowers: &[],
        loops: &[
            // Decorative giant Hot Wheels loop beside the tabletop straight.
            Loop {
                along: 1750.0,
                side: Side::Right,
                lateral: 30.0,
                radius: 16.0,
            },
        ],
        rings: &[
            // Bonus golden rings over the start jump, the Scalextric bridge
            // jump, the Castle split, and the Playground Slide plunge.
            Rings {
                along: 45.0,
                count: 5,
                lateral: 0.0,
                height: 2.2,
            },
            Rings {
                along: 1655.0,
                count: 5,
                lateral: 0.0,
                height: 2.4,
            },
            Rings {
                along: 2490.0,
                count: 6,
                lateral: 0.0,
                height: 1.6,
            },
            Rings {
                along: 3250.0,
                count: 6,
                lateral: 0.0,
                height: 1.6,
            },
        ],
        waterfalls: &[],
        sky: SkyStyle::ToyShop,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        wheelie_halo: None,
        japan: None,
        middle_earth: None,
        space: None,
        toy_shop: Some(TOY_SHOP_SCENERY),
    },
};

const TOY_SHOP_SCENERY: ToyShop = ToyShop {
    scalextric: SCALEXTRIC_TABLE,
    hot_wheels: HOT_WHEELS_RUN,
    dino_loop_along: 345.0,
    hot_wheels_gates: &[
        HotWheelsGate { along: 75.0 },
        HotWheelsGate { along: 660.0 },
        HotWheelsGate { along: 3620.0 },
        HotWheelsGate { along: 4105.0 },
    ],
    scalextric_bridges: &[
        ScalextricBridge { along: 1185.0 },
        ScalextricBridge { along: 1380.0 },
        ScalextricBridge { along: 1840.0 },
        ScalextricBridge { along: 2180.0 },
    ],
    playmobil_gates: &[
        // Main Playmobil Castle Gatehouse before the Castle & Board-Game Split.
        PlaymobilGate { along: 2305.0 },
        // Second Playmobil Castle Gatehouse at the exit of the split.
        PlaymobilGate { along: 2715.0 },
    ],
    train_lines: &[
        // Electric model railway running along the Scalextric tabletop.
        ToyTrainLine {
            span: span(1120.0, 1455.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 4.8,
            trains: 2,
        },
        ToyTrainLine {
            span: span(1610.0, 2240.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.8,
            trains: 3,
        },
        // Upper wooden Brio & electric railway beside the shelf climb.
        ToyTrainLine {
            span: span(2730.0, 3090.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.2,
            trains: 2,
        },
    ],
    chairs: &[
        // Giant wooden chairs beside the climb ramps from the floor to the
        // tables and shelves.
        ToyChair {
            along: 175.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
            heading: 0.25,
        },
        ToyChair {
            along: 720.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 11.5,
            heading: -0.35,
        },
        ToyChair {
            along: 1065.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
            heading: 0.15,
        },
        ToyChair {
            along: 2810.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 12.0,
            heading: -0.2,
        },
        ToyChair {
            along: 3560.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
            heading: 0.4,
        },
        ToyChair {
            along: 4380.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
            heading: -0.3,
        },
    ],
    cube_shelves: &[
        // High Kallax wooden cube shelves (`photo 1`) flanking the high shelf
        // overpass (`2740..3120 m`).
        CubeShelf {
            span: span(2750.0, 3110.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
        },
        CubeShelf {
            span: span(2750.0, 3110.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 11.5,
        },
        // Lower toy-display shelves along the back straight.
        CubeShelf {
            span: span(3860.0, 4180.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.5,
        },
    ],
    slides: &[
        // Steep -40 m Playground Slide (`Tobogán`) plunge from the high Kallax
        // shelf down to the floor (`3120..3555 m`).
        SlideRun { span: SLIDE_PLUNGE },
    ],
    xylophones: &[
        // Rainbow metallic xylophone bars & mallets across the narrow bridge.
        XylophoneSpan {
            span: XYLOPHONE_BRIDGE,
        },
    ],
    floor_pianos: &[
        // Giant `Big`-style floor pianos along the shop floor sector.
        FloorPiano {
            span: span(15.0, 95.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.8,
        },
        FloorPiano {
            span: span(4460.0, 4560.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 4.8,
        },
    ],
    soldier_rows: &[
        // Giant wooden Nutcracker toy soldiers (`photo 2`) guarding the red
        // runner carpet along the start/finish straight.
        SoldierRow {
            span: span(4320.0, 95.0),
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.2,
            every: 18.0,
        },
        SoldierRow {
            span: span(4320.0, 95.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.2,
            every: 18.0,
        },
    ],
    props: &[
        // Hopscotch courts (`rayuela`, photo 3) on the shop floor.
        ToyProp {
            along: 145.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::Hopscotch,
            scale: 1.3,
        },
        ToyProp {
            along: 3585.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::Hopscotch,
            scale: 1.3,
        },
        // Hot Wheels stunt tire towers beside the loop and back straight.
        ToyProp {
            along: 295.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 7.5,
            kind: ToyKind::HotWheelsTower,
            scale: 1.25,
        },
        ToyProp {
            along: 525.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 7.5,
            kind: ToyKind::HotWheelsTower,
            scale: 1.25,
        },
        ToyProp {
            along: 3980.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 7.5,
            kind: ToyKind::HotWheelsTower,
            scale: 1.3,
        },
        // Giant 1:32 Scalextric / Ninco slot cars on the green felt tabletop.
        ToyProp {
            along: 1150.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::SlotCar,
            scale: 1.4,
        },
        ToyProp {
            along: 1320.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::SlotCar,
            scale: 1.4,
        },
        ToyProp {
            along: 1710.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::SlotCar,
            scale: 1.4,
        },
        ToyProp {
            along: 2040.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::SlotCar,
            scale: 1.4,
        },
        // Wooden Brio railway helipad control towers beside the train lines.
        ToyProp {
            along: 1260.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 11.0,
            kind: ToyKind::TrainHelipadTower,
            scale: 1.2,
        },
        ToyProp {
            along: 1890.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 11.0,
            kind: ToyKind::TrainHelipadTower,
            scale: 1.2,
        },
        // Giant LEGO brick towers & toy pirate ships around the Xylophone bridge.
        ToyProp {
            along: 1455.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::LegoTower,
            scale: 1.35,
        },
        ToyProp {
            along: 1455.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::LegoTower,
            scale: 1.35,
        },
        ToyProp {
            along: 1605.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::LegoTower,
            scale: 1.35,
        },
        ToyProp {
            along: 1605.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.5,
            kind: ToyKind::LegoTower,
            scale: 1.35,
        },
        ToyProp {
            along: 1515.0,
            side: Side::Left,
            lateral: 26.0,
            kind: ToyKind::ToyPirateShip,
            scale: 1.6,
        },
        ToyProp {
            along: 1555.0,
            side: Side::Right,
            lateral: 26.0,
            kind: ToyKind::ToyPirateShip,
            scale: 1.6,
        },
        // Playmobil knights, cardboard castle turrets (photo 3) and giant
        // board-game box stacks along the Castle Split (`2280..2730 m`).
        ToyProp {
            along: 2275.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.0,
            kind: ToyKind::PlaymobilKnight,
            scale: 1.4,
        },
        ToyProp {
            along: 2275.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.0,
            kind: ToyKind::PlaymobilKnight,
            scale: 1.4,
        },
        ToyProp {
            along: 2390.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 8.5,
            kind: ToyKind::CardboardTurret,
            scale: 1.25,
        },
        ToyProp {
            along: 2520.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 8.5,
            kind: ToyKind::BoardGameStack,
            scale: 1.35,
        },
        ToyProp {
            along: 2630.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 8.5,
            kind: ToyKind::CardboardTurret,
            scale: 1.25,
        },
        // Giant Matryoshka dolls, wooden robots & rocking horses (photo 1) on
        // the high shelves and tabletops.
        ToyProp {
            along: 2860.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::Matryoshka,
            scale: 1.3,
        },
        ToyProp {
            along: 2960.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.5,
            kind: ToyKind::WoodenRobot,
            scale: 1.25,
        },
        ToyProp {
            along: 3040.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.8,
            kind: ToyKind::RockingHorse,
            scale: 1.3,
        },
        ToyProp {
            along: 3680.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.8,
            kind: ToyKind::WoodenRobot,
            scale: 1.3,
        },
        ToyProp {
            along: 3820.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 6.8,
            kind: ToyKind::Matryoshka,
            scale: 1.35,
        },
        ToyProp {
            along: 4190.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 6.8,
            kind: ToyKind::RockingHorse,
            scale: 1.35,
        },
    ],
};

/// Approach and exit curves for the Hot Wheels 360° `VerticalLoop` branch
/// (`110.0..637.0 m`).
const LOOP_APPROACH: [Vec2; 9] = [
    Vec2::new(0.5, -122.5),
    Vec2::new(2.0, -135.0),
    Vec2::new(4.4, -147.3),
    Vec2::new(7.8, -159.4),
    Vec2::new(12.2, -171.2),
    Vec2::new(17.4, -182.6),
    Vec2::new(23.6, -193.6),
    Vec2::new(30.6, -204.0),
    Vec2::new(38.3, -213.9),
];

const LOOP_EXIT: [Vec2; 15] = [
    Vec2::new(137.5, -290.6),
    Vec2::new(146.5, -298.2),
    Vec2::new(156.3, -305.9),
    Vec2::new(166.1, -313.6),
    Vec2::new(175.8, -321.3),
    Vec2::new(185.6, -329.0),
    Vec2::new(195.1, -336.2),
    Vec2::new(204.8, -343.2),
    Vec2::new(214.7, -349.9),
    Vec2::new(224.8, -356.3),
    Vec2::new(235.2, -362.4),
    Vec2::new(245.7, -368.1),
    Vec2::new(256.5, -373.3),
    Vec2::new(267.4, -377.9),
    Vec2::new(278.4, -381.7),
];

const CONTROL_POINTS: [Vec2; 60] = [
    // 1. Shop Floor start straight (north, -Z) & Hot Wheels Dino Loop sector.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -110.0),
    Vec2::new(0.0, -220.0),
    Vec2::new(6.0, -300.0),
    Vec2::new(35.0, -360.0),
    Vec2::new(90.0, -395.0),
    Vec2::new(160.0, -400.0),
    Vec2::new(225.0, -380.0),
    Vec2::new(290.0, -385.0),
    Vec2::new(345.0, -420.0),
    // 2. Toy Box Tunnel (under the high Kallax shelf crossing at (550, -430)).
    Vec2::new(415.0, -430.0),
    Vec2::new(480.0, -430.0),
    Vec2::new(550.0, -430.0),
    Vec2::new(620.0, -430.0),
    Vec2::new(690.0, -430.0),
    // 3. Scalextric Tabletop, Xylophone Bridge & Electric Train sector.
    Vec2::new(765.0, -430.0),
    Vec2::new(840.0, -415.0),
    Vec2::new(895.0, -365.0),
    Vec2::new(920.0, -290.0),
    Vec2::new(920.0, -180.0),
    Vec2::new(920.0, -60.0),
    Vec2::new(920.0, 60.0),
    Vec2::new(920.0, 170.0),
    // 4. Banked sweep round the Scalextric tabletop turn.
    Vec2::new(920.0, 270.0),
    Vec2::new(903.0, 338.0),
    Vec2::new(858.0, 382.0),
    Vec2::new(807.5, 395.0),
    Vec2::new(757.0, 382.0),
    Vec2::new(712.0, 338.0),
    Vec2::new(695.0, 270.0),
    // 5. Playmobil Castle Gate & Board-Game Split (`x = 695 m`).
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
    // 6. High Kallax Cube Shelf (`45.5 m` overpass over the Toy Box tunnel at
    // (550, -430)) & steep -40 m Playground Slide (`Tobogán`) plunge.
    Vec2::new(688.0, -230.0),
    Vec2::new(658.0, -300.0),
    Vec2::new(608.0, -368.0),
    Vec2::new(550.0, -430.0),
    Vec2::new(485.0, -490.0),
    Vec2::new(405.0, -535.0),
    Vec2::new(310.0, -552.0),
    Vec2::new(200.0, -555.0),
    Vec2::new(85.0, -548.0),
    Vec2::new(-25.0, -525.0),
    Vec2::new(-105.0, -475.0),
    Vec2::new(-150.0, -395.0),
    // 7. Final Hot Wheels tabletop climb (`x = -165 m`), straightaway jump, and
    // ramp down to the shop floor finish line.
    Vec2::new(-165.0, -295.0),
    Vec2::new(-165.0, -185.0),
    Vec2::new(-165.0, -75.0),
    Vec2::new(-160.0, 25.0),
    Vec2::new(-135.0, 98.0),
    Vec2::new(-75.0, 122.0),
    Vec2::new(-22.0, 92.0),
    Vec2::new(-2.0, 50.0),
];

/// Elevation profile (over `BASE_ELEVATION = 2.0 m`):
/// - **`0..660 m` (Shop Floor & Hot Wheels Loop Sector):** Starts on the parquet
///   floor (`0.0 m`), launches over Jump #1 (`55 m`), and rolls between `1.0 m`
///   and `10.0 m` while the right-hand Hot Wheels branch loops through the 16 m
///   `VerticalLoop`.
/// - **`660..1020 m` (Toy Box Tunnel):** Descends from the chair ramp into the
///   Giant Toy Box Tunnel (`0.4 m`).
/// - **`1020..2690 m` (Chair Climb, Scalextric Table, Xylophone Bridge &
///   Playmobil Castle Split):** Climbs via a giant chair ramp onto the
///   Scalextric tabletop (`15.5..20.0 m`), crosses the Xylophone Bridge over the
///   toy-boat pond (`16.0 m`), sweeps round the tabletop hairpin (`8.5 m`), and
///   climbs through the Playmobil Castle Gate & Split (`16.5..28.5 m`).
/// - **`2690..3555 m` (High Kallax Cube Shelf & Playground Slide `Tobogán`):**
///   Ascends to the high Kallax shelf (`45.5 m` at `3056 m`, `45.1 m` above the
///   Toy Box tunnel), then plunges **`-40.0 m`** down the Playground Slide
///   (`3120..3555 m`) back to `5.5 m`.
/// - **`3555..4742 m` (Hot Wheels Boost Climb & Final Ramp):** Climbs up the
///   western Hot Wheels tabletop (`35.6 m` at Jump #3, `4095 m`) and ramps
///   down to `0.0 m` at the finish line.
const ELEVATION: [(f32, f32); 59] = [
    (0.0, 0.0),
    // Jump #1 under the Hot Wheels 5-lane start gate.
    (25.0, 0.3),
    (55.0, 2.0),
    (85.0, 0.5),
    (110.0, 1.0),
    // Smooth elevation beside the Hot Wheels VerticalLoop branch (110..637 m).
    (160.0, 4.0),
    (195.0, 5.0),
    (235.0, 4.5),
    (300.0, 2.5),
    (380.0, 3.0),
    (470.0, 7.0),
    (540.0, 9.0),
    (600.0, 7.5),
    (660.0, 10.0),
    (710.0, 10.5),
    (765.0, 4.5),
    // Giant Toy Box Tunnel (800..1020 m).
    (800.0, 0.8),
    (850.0, 0.4),
    (950.0, 0.4),
    (1020.0, 0.8),
    // Chair-to-table ramp up to the Scalextric tabletop (1120..2250 m).
    (1070.0, 4.5),
    (1120.0, 9.5),
    (1190.0, 14.0),
    (1320.0, 15.5),
    (1470.0, 16.0),
    (1590.0, 16.0),
    (1615.0, 16.2),
    (1665.0, 20.0),
    (1705.0, 17.5),
    (1820.0, 13.5),
    (1960.0, 10.0),
    (2110.0, 8.5),
    (2250.0, 12.0),
    // Playmobil Castle Gate & 2-way Board-Game Split.
    (2340.2, 16.5),
    (2690.3, 28.5),
    (2760.0, 33.0),
    (2830.0, 38.5),
    // High Kallax Cube Shelf (45.5 m overpass above the Toy Box tunnel at 3056 m).
    (2895.0, 42.5),
    (2975.0, 44.8),
    (3056.0, 45.5),
    // Steep -40.0 m plunge down the Playground Slide (`Tobogán`, 3120..3555 m).
    (3120.0, 43.0),
    (3220.0, 33.0),
    (3330.0, 21.5),
    (3430.0, 11.5),
    (3500.0, 6.5),
    (3555.0, 5.5),
    // Final Hot Wheels tabletop climb, high-altitude Jump #3 (4095 m), and
    // ramp down to the shop floor finish line.
    (3610.0, 8.5),
    (3700.0, 17.5),
    (3790.0, 26.5),
    (3860.0, 31.0),
    (3960.0, 31.0),
    (4020.0, 31.4),
    (4095.0, 35.6),
    (4135.0, 34.0),
    (4200.0, 28.5),
    (4300.0, 18.0),
    (4400.0, 8.0),
    (4480.0, 2.0),
    (4540.0, 0.4),
];

/// Alternative left branch through the Playmobil Castle & Board-Game Split
/// (`2340.2..2690.3 m`).
const CASTLE_LEFT: [Vec2; 11] = [
    Vec2::new(693.2, 210.1),
    Vec2::new(682.4, 181.5),
    Vec2::new(659.8, 148.6),
    Vec2::new(637.9, 114.4),
    Vec2::new(626.7, 81.4),
    Vec2::new(629.3, 48.4),
    Vec2::new(643.3, 15.4),
    Vec2::new(662.7, -17.5),
    Vec2::new(682.0, -45.2),
    Vec2::new(692.0, -72.2),
    Vec2::new(695.5, -96.6),
];

const CASTLE_LEFT_RISE: [(f32, f32); 35] = [
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

/// Tabletop & floor margin round the lap, with an indentation under
/// `XYLOPHONE_BRIDGE` (`1470..1590 m`) for the toy-boat water pond.
const COAST: [(f32, f32); 12] = [
    (0.0, 260.0),
    (1200.0, 260.0),
    (1420.0, 60.0),
    (1475.0, -180.0),
    (1585.0, -180.0),
    (1640.0, 60.0),
    (1800.0, 240.0),
    (2400.0, 260.0),
    (3100.0, 260.0),
    (3700.0, 240.0),
    (4200.0, 240.0),
    (4550.0, 260.0),
];

/// Central toy-box & shelf support massif at `(550, -430)`.
const TOY_BOX_MASSIF: Massif = climbed_massif(550.0, -430.0, 220.0, 46.0);

const MASSIFS: [Massif; 10] = [
    TOY_BOX_MASSIF,
    climbed_massif(165.0, -395.0, 195.0, 14.0),
    climbed_massif(640.0, -290.0, 210.0, 44.0),
    climbed_massif(400.0, -520.0, 210.0, 36.0),
    climbed_massif(-165.0, -180.0, 220.0, 36.0),
    climbed_massif(880.0, -340.0, 190.0, 20.0),
    climbed_massif(695.0, 20.0, 220.0, 30.0),
    massif(200.0, -250.0, 55.0, 22.0),
    massif(430.0, -180.0, 65.0, 26.0),
    massif(807.5, 210.0, 55.0, 22.0),
];

const CUTTINGS: [Cutting; 2] = [
    Cutting {
        span: span(TOY_BOX_TUNNEL.from + 40.0, TOY_BOX_TUNNEL.to + 20.0),
        side: Side::Left,
        height: 16.0,
    },
    Cutting {
        span: span(TOY_BOX_TUNNEL.from + 40.0, TOY_BOX_TUNNEL.to + 20.0),
        side: Side::Right,
        height: 16.0,
    },
];

fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(260.0, -240.0);
    let dist = point.distance(CENTER);
    smoothstep(1800.0, 2800.0, dist)
        * 120.0
        * (0.4 + 0.6 * fbm(point / 300.0 + Vec2::splat(3.7), 3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    #[test]
    fn toy_shop_has_scalextric_hot_wheels_loop_trains_playmobil_gate_and_slide() {
        let toy = TOY_SHOP
            .scenery
            .toy_shop
            .as_ref()
            .expect("Toy Shop has ToyShop scenery");
        assert!(!toy.playmobil_gates.is_empty(), "has Playmobil castle gate");
        assert!(
            !toy.hot_wheels_gates.is_empty(),
            "has Hot Wheels lane gates"
        );
        assert!(!toy.scalextric_bridges.is_empty(), "has Scalextric bridges");
        assert!(!toy.train_lines.is_empty(), "has electric toy train lines");
        assert!(!toy.chairs.is_empty(), "has giant wooden chairs");
        assert!(!toy.cube_shelves.is_empty(), "has Kallax cube shelves");
        assert!(!toy.slides.is_empty(), "has playground slide (tobogán)");
        assert!(!toy.xylophones.is_empty(), "has xylophone bridge");
        assert!(!toy.floor_pianos.is_empty(), "has floor piano");
        assert!(!toy.soldier_rows.is_empty(), "has Nutcracker soldier rows");

        let track = Track::new(&TOY_SHOP.layout);
        // Has both the 360-degree Hot Wheels vertical loop and the 2-way
        // Playmobil castle split.
        assert_eq!(TOY_SHOP.layout.branches.len(), 2);
        assert!(TOY_SHOP.layout.branches[0].vertical_loop.is_some());

        // High Kallax shelf crosses >40 m above the Toy Box tunnel.
        let line = track.centerline();
        let under = track.index_at(TOY_BOX_TUNNEL.from)..track.index_at(TOY_BOX_TUNNEL.to);
        let (gap, over, under) = (track.index_at(2970.0)..track.index_at(3140.0))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("crossing and tunnel have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 40.0, "{above} m above");
    }
}
