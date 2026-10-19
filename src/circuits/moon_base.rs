//! Base Lunar (Moon Base): a lap that starts at an Earth Space Launch &
//! Deep-Space Observatory Complex in the coastal mountains, crosses an
//! interdimensional portal to a lunar colony in Mare Tranquillitatis, and
//! returns through a second portal to Earth's high peaks.
//!
//! 1. **Earth Space Center & Observatory Peak (`0..1120 m`):** The start
//!    straight runs north on Earth past a Saturn-V space launchpad, an
//!    astronomical telescope observatory dome (`Observatory`), and giant
//!    deep-space radio antennas (`RadioDish`), with the Moon shining in the
//!    night sky overhead. After a straightaway jump crest (`55 m`), the road
//!    climbs +31 m up the forested Observatory Mountain (`32 m`), plunges
//!    -31.6 m into the underground Launch Complex Tunnel (`800..1020 m`), and
//!    reaches the first **Interdimensional Portal (`Earth → Moon`, `1120 m`)**.
//! 2. **Lunar Colony & Crater Split (`1120..2895 m`):** Emerging onto the
//!    airless grey regolith of the Moon (where the blue Earth hangs in the
//!    sky), riders cross a steel truss bridge over a lunar rille, pass
//!    pressurized geodesic biodomes, six-wheeled lunar rovers, and solar
//!    arrays, sweep round the Sea of Tranquillity crater, and climb a two-way
//!    Crater Split (`2340..2690 m`) to the second **Interdimensional Portal
//!    (`Moon → Earth`, `2895 m`)**.
//! 3. **High Earth Summit & Western Mountain Roller-Coaster (`2895..4600 m`):**
//!    Back on Earth at `42.5..45.5 m` altitude, the road crosses 45 m over the
//!    top of the tunnel, plunges -40 m into a deep forested valley (`5.5 m`),
//!    climbs +30 m back up the Western Coastal Range (`35.6 m`), launches off a
//!    high-altitude straightaway jump (`4095 m`), and dives -35.6 m back to the
//!    finish line.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Key stretches of the lap, in metres from the finish line.
const START_SECTOR: Span = span(4280.0, 310.0);
const TUNNEL: Span = span(800.0, 1020.0);
const MOON_PORTAL_IN: f32 = 1120.0;
const BRIDGE: Span = span(1470.0, 1590.0);
const EAST_STRAIGHT: Span = span(1360.0, 1900.0);
const LUNAR_CRATER_TURN: Span = span(1920.0, 2290.0);
const CRATER_SPLIT: (f32, f32) = (2340.2, 2690.3);
const EARTH_PORTAL_OUT: f32 = 2895.0;
const CROSSING: Span = span(2970.0, 3140.0);
const WEST_STRAIGHT: Span = span(3860.0, 4340.0);

/// Space center & lunar station colours: orbital cyan, titanium white, and warning orange.
const CYAN: Color = Color::srgb(0.12, 0.68, 0.96);
const WHITE: Color = Color::srgb(0.92, 0.95, 0.98);
const ORANGE: Color = Color::srgb(0.92, 0.36, 0.10);

pub const MOON_BASE: Circuit = Circuit {
    name: "Base Lunar",
    name_en: "Moon Base",
    description: "Centro espacial y observatorio en la Tierra con portal a la Luna:\n\
                  lanzadera, antenas gigantes, cúpulas de telescopio y base lunar",
    description_en: "Earth space center and observatory with portal to the Moon:\n\
                     launchpad, giant radio dishes, telescope domes and lunar base",
    ambience: Ambience {
        sea: 0.15,
        gulls: 0.0,
        birds: 0.28,
        cicadas: 0.0,
        wind: 0.28,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/moon_base.mp3"),
        length: 179.6,
        loudness: -16.5,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[
            // Banked sweep round the Sea of Tranquillity crater rim on the Moon.
            Banking {
                from: LUNAR_CRATER_TURN.from,
                to: LUNAR_CRATER_TURN.to,
                factor: 2.2,
            },
            // Level across the Crater Split so the branch joins smoothly.
            Banking {
                from: CRATER_SPLIT.0,
                to: CRATER_SPLIT.1,
                factor: 0.25,
            },
            // Banked final corner onto the Earth start straight.
            Banking {
                from: 4340.0,
                to: 4540.0,
                factor: 1.5,
            },
        ],
        narrows: &[],
        branches: &[
            // High crater-rim alternative route to the left on the Moon half.
            Branch {
                from: CRATER_SPLIT.0,
                to: CRATER_SPLIT.1,
                approach: &CRATER_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &CRATER_LEFT_RISE,
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 28.0,
        soil: Soil::Lunar,
        greens: Greens::Countryside,
        grass_shade: 0.5,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[
            // Level pads right beside the Earth start straight for the first
            // Telescope Observatory Dome and Saturn-V Space Launchpad.
            Terrace {
                center: Vec2::new(-29.5, -140.0),
                radius: 26.0,
                height: 2.8,
                tilt: Vec2::ZERO,
                skirt: 12.0,
            },
            Terrace {
                center: Vec2::new(28.0, -215.0),
                radius: 26.0,
                height: 3.2,
                tilt: Vec2::ZERO,
                skirt: 12.0,
            },
        ],
        mountains,
        curbs: [CYAN, WHITE],
        walls: &[
            WallRun {
                span: START_SECTOR,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: START_SECTOR,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CROSSING,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CROSSING,
                side: Side::Right,
                style: WallStyle::Stone,
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
        tunnels: &[TUNNEL],
        bridges: &[Bridge {
            span: BRIDGE,
            style: BridgeStyle::Truss,
            fence: 0.0,
        }],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: CYAN,
                text: Some("SPACE CENTER"),
            },
            Gantry {
                along: 1090.0,
                color: ORANGE,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2240.0,
                color: ORANGE,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3450.0,
                color: ORANGE,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 440.0,
                side: Side::Left,
                text: "OBSERVATORY",
                color: Color::srgb(0.10, 0.16, 0.28),
                accent: Some((6, CYAN)),
            },
            Billboard {
                along: 760.0,
                side: Side::Left,
                text: "LAUNCH PAD",
                color: Color::srgb(0.10, 0.12, 0.18),
                accent: Some((6, ORANGE)),
            },
            Billboard {
                along: 1095.0,
                side: Side::Right,
                text: "MOON PORTAL",
                color: Color::srgb(0.08, 0.18, 0.34),
                accent: Some((4, CYAN)),
            },
            Billboard {
                along: 1900.0,
                side: Side::Right,
                text: "MOON BASE",
                color: Color::srgb(0.08, 0.14, 0.26),
                accent: None,
            },
            Billboard {
                along: 2310.0,
                side: Side::Left,
                text: "CRATER SPLIT",
                color: ORANGE,
                accent: None,
            },
            Billboard {
                along: 2870.0,
                side: Side::Right,
                text: "EARTH PORTAL",
                color: Color::srgb(0.10, 0.28, 0.18),
                accent: Some((5, CYAN)),
            },
            Billboard {
                along: 3820.0,
                side: Side::Right,
                text: "EARTH PEAKS",
                color: Color::srgb(0.10, 0.26, 0.18),
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
        lamps: &[
            Row {
                span: START_SECTOR,
                side: Side::Left,
                lateral: WALL_DISTANCE + 0.8,
                every: 26.0,
            },
            Row {
                span: START_SECTOR,
                side: Side::Right,
                lateral: WALL_DISTANCE + 0.8,
                every: 26.0,
            },
            Row {
                span: EAST_STRAIGHT,
                side: Side::Left,
                lateral: WALL_DISTANCE + 0.8,
                every: 32.0,
            },
            Row {
                span: WEST_STRAIGHT,
                side: Side::Right,
                lateral: WALL_DISTANCE + 0.8,
                every: 32.0,
            },
        ],
        // Trees, forests and cottages populate the Earth stretch (`2895..4600..1120 m`),
        // where `is_earth_realm` is true and `Terrain::ground` is `Ground::Grass`.
        // Not all green: autumn maples in crimson, golden ginkgos and
        // almonds and cherries in pink blossom among the greens.
        trees: &[
            Species::Broadleaf,
            Species::Tall,
            Species::MaritimePine,
            Species::Poplar,
            Species::Momiji,
            Species::Ginkgo,
            Species::Almond,
            Species::Sakura,
        ],
        forests: &[
            Forest {
                span: span(320.0, 760.0),
                species: &[
                    Species::Tall,
                    Species::Broadleaf,
                    Species::Momiji,
                    Species::Ginkgo,
                ],
            },
            Forest {
                span: span(3140.0, 3840.0),
                species: &[
                    Species::MaritimePine,
                    Species::Broadleaf,
                    Species::Almond,
                    Species::Momiji,
                ],
            },
            Forest {
                span: span(3860.0, 4480.0),
                species: &[
                    Species::Tall,
                    Species::Poplar,
                    Species::Ginkgo,
                    Species::Sakura,
                ],
            },
        ],
        houses: Houses {
            max: 10,
            paints: &[
                Color::srgb(0.95, 0.92, 0.85),
                Color::srgb(0.92, 0.82, 0.70),
                Color::srgb(0.86, 0.90, 0.94),
            ],
            spans: &[span(320.0, 740.0), span(3200.0, 4280.0)],
            style: HouseStyle::Cottage,
        },
        palms: &[],
        grand_hotel: None,
        hotels: &[],
        grandstands: &[
            Grandstand {
                along: 40.0,
                side: Side::Left,
                lateral: WALL_DISTANCE + 14.0,
                length: 64.0,
            },
            Grandstand {
                along: 2100.0,
                side: Side::Right,
                lateral: WALL_DISTANCE + 14.0,
                length: 54.0,
            },
        ],
        lighthouses: &[],
        buoys: &[],
        boats: &[],
        beach_parasols: &[],
        totems: &[],
        sunflowers: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Space,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        japan: None,
        space: Some(SPACE),
        toys: None,
    },
};

const SPACE: Space = Space {
    // Pressurized geodesic space-station biodomes & command towers right beside
    // the road (`WALL_DISTANCE + r + 2.5 m`).
    biodomes: &[
        Biodome {
            along: 485.0,
            side: Side::Left,
            radius: 17.0,
        },
        Biodome {
            along: 1185.0,
            side: Side::Right,
            radius: 17.0,
        },
        Biodome {
            along: 1315.0,
            side: Side::Left,
            radius: 18.0,
        },
        Biodome {
            along: 1430.0,
            side: Side::Right,
            radius: 16.0,
        },
        Biodome {
            along: 1650.0,
            side: Side::Right,
            radius: 18.0,
        },
        Biodome {
            along: 1795.0,
            side: Side::Left,
            radius: 17.0,
        },
        Biodome {
            along: 1935.0,
            side: Side::Left,
            radius: 18.0,
        },
        Biodome {
            along: 2120.0,
            side: Side::Left,
            radius: 17.0,
        },
        Biodome {
            along: 2265.0,
            side: Side::Right,
            radius: 16.0,
        },
        Biodome {
            along: 2765.0,
            side: Side::Right,
            radius: 18.0,
        },
    ],
    // Six-wheeled lunar rovers on the Moon (`1120..2895 m`).
    rovers: &[
        Rover {
            along: 1180.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 8.5,
            heading: 0.4,
        },
        Rover {
            along: 1410.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 8.5,
            heading: -0.8,
        },
        Rover {
            along: 1760.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 9.0,
            heading: 1.5,
        },
        Rover {
            along: 1990.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 9.0,
            heading: 2.1,
        },
        Rover {
            along: 2280.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 8.5,
            heading: -1.7,
        },
        Rover {
            along: 2790.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 8.5,
            heading: -0.4,
        },
    ],
    // Giant 27 m deep-space radio antennas right beside the Earth road (`26..30 m`
    // from the centerline, oriented toward oncoming riders).
    dishes: &[
        RadioDish {
            position: Vec2::new(26.0, -95.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(-26.0, -255.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(145.0, -368.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(285.0, -414.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(255.0, -582.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(40.0, -568.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(-192.0, -70.0),
            heading: 0.0,
        },
        RadioDish {
            position: Vec2::new(-112.0, 145.0),
            heading: 0.0,
        },
    ],
    // Giant astronomical telescope observatory domes (`cúpulas con telescopio`)
    // placed right beside the Earth road (`28..32 m` from the centerline) and
    // dead ahead on the outside of bends, with their open shutter slits and
    // 21.5 m telescope tubes aimed toward oncoming riders.
    observatories: &[
        Observatory {
            position: Vec2::new(-29.5, -140.0),
            heading: -2.74,
        },
        Observatory {
            position: Vec2::new(6.0, -372.0),
            heading: -2.95,
        },
        Observatory {
            position: Vec2::new(225.0, -350.0),
            heading: 1.25,
        },
        Observatory {
            position: Vec2::new(365.0, -458.0),
            heading: 2.15,
        },
        Observatory {
            position: Vec2::new(480.0, -536.0),
            heading: -1.10,
        },
        Observatory {
            position: Vec2::new(-195.0, -185.0),
            heading: -0.45,
        },
    ],
    // Space launchpads (`lanzaderas espaciales`) with 57 m Saturn-V rockets and
    // red umbilical gantry towers right beside the Earth road.
    rockets: &[
        Rocket {
            position: Vec2::new(28.0, -215.0),
            heading: std::f32::consts::PI,
        },
        Rocket {
            position: Vec2::new(320.0, -368.0),
            heading: std::f32::consts::FRAC_PI_2,
        },
        Rocket {
            position: Vec2::new(-135.0, -295.0),
            heading: 0.0,
        },
    ],
    // Overhead Orbital Space-Station Habitat Modules (`arches`) spanning right
    // across the road on both the Space Center and Lunar Colony stretches.
    arches: &[
        StationArch { along: 85.0 },
        StationArch { along: 195.0 },
        StationArch { along: 305.0 },
        StationArch { along: 540.0 },
        StationArch { along: 1210.0 },
        StationArch { along: 1350.0 },
        StationArch { along: 1620.0 },
        StationArch { along: 1740.0 },
        StationArch { along: 1860.0 },
        StationArch { along: 2080.0 },
        StationArch { along: 2315.0 },
        StationArch { along: 2720.0 },
        StationArch { along: 3056.0 },
        StationArch { along: 4050.0 },
        StationArch { along: 4510.0 },
    ],
    portals: &[
        // First portal: leaves the Earth Space Center onto the Moon (1120 m).
        Portal {
            along: MOON_PORTAL_IN,
        },
        // Second portal: leaves the Moon back to Earth's high summit (2895 m).
        Portal {
            along: EARTH_PORTAL_OUT,
        },
    ],
    // Solar panel arrays on the Moon (`1120..2895 m`).
    solar_arrays: &[
        Row {
            span: span(1160.0, 1440.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 7.5,
            every: 22.0,
        },
        Row {
            span: span(1620.0, 1900.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 7.5,
            every: 22.0,
        },
        Row {
            span: span(2320.0, 2840.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 7.5,
            every: 24.0,
        },
    ],
    boulders: LunarBoulders {
        max: 75,
        spans: &[span(1150.0, 2865.0)],
    },
};

const CONTROL_POINTS: [Vec2; 60] = [
    // 1. Earth Space Center start straight (north, -Z) & Observatory Peak climb.
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
    // 2. Underground Launch Complex Tunnel (under the central massif at (550, -430))
    // followed by the Interdimensional Portal to the Moon at 1120 m.
    Vec2::new(415.0, -430.0),
    Vec2::new(480.0, -430.0),
    Vec2::new(550.0, -430.0),
    Vec2::new(620.0, -430.0),
    Vec2::new(690.0, -430.0),
    // 3. Moon Sector (1120..2895 m): lunar truss bridge, straightaway jump crest,
    // and sweep around the Sea of Tranquillity crater.
    Vec2::new(765.0, -430.0),
    Vec2::new(840.0, -415.0),
    Vec2::new(895.0, -365.0),
    Vec2::new(920.0, -290.0),
    Vec2::new(920.0, -180.0),
    Vec2::new(920.0, -60.0),
    Vec2::new(920.0, 60.0),
    Vec2::new(920.0, 170.0),
    // 4. Banked sweep clockwise round the lunar crater rim.
    Vec2::new(920.0, 270.0),
    Vec2::new(903.0, 338.0),
    Vec2::new(858.0, 382.0),
    Vec2::new(807.5, 395.0),
    Vec2::new(757.0, 382.0),
    Vec2::new(712.0, 338.0),
    Vec2::new(695.0, 270.0),
    // 5. Lunar Crater Split (climbing north along x = 695 m).
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
    // 6. Return through the Earth Portal (2895 m), cross 45 m OVER the tunnel
    // at (550, -430), and plunge into the deep northern Earth valley.
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
    // 7. Climb up the Western Earth Mountain Ridge (x = -165 m), launch off
    // the high straightaway jump, and dive back to the start straight.
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
/// - **Earth Part 1 (`0..1120 m`):** Starts at `0.0 m` on the Earth Space
///   Center straightaway, launches over Jump #1 (`55 m`), climbs **+31 m** up
///   the Observatory Mountain Peak (`32.0 m` at `475 m`), plunges **-31.6 m**
///   down into the underground Launch Complex Tunnel (`800..1020 m` at `0.4 m`),
///   and climbs to **Portal #1 (`Earth → Moon`, `1120 m` at `9.5 m`)**.
/// - **Moon Sector (`1120..2895 m`):** Gently rolls across the Lunar Truss
///   Bridge (`16.0 m`), over Jump #2 (`20.0 m` at `1665 m`), around the Sea of
///   Tranquillity crater (`8.5 m`), and up the Crater Split (`16.5..28.5 m`) to
///   **Portal #2 (`Moon → Earth`, `2895 m` at `42.5 m`)**.
/// - **Earth Part 2 (`2895..4600 m`):** Crests at **`45.5 m`** (45.1 m above
///   the tunnel at `3056 m`), plunges **`-40.0 m`** into the northern Earth
///   valley (`5.5 m` at `3555 m`), climbs **`+30.1 m`** up the Western Earth
///   Mountain Ridge (`35.6 m` at Jump #3, `4095 m`), and plunges **`-35.6 m`**
///   back to the finish line (`0.0 m`).
const ELEVATION: [(f32, f32); 57] = [
    (0.0, 0.0),
    // Jump #1: just past the finish line on the Earth Space Center straight.
    (25.0, 0.3),
    (55.0, 2.0),
    (85.0, 0.5),
    (110.0, 1.0),
    // Earth Mountain #1: steep +31 m climb up the Observatory Peak and -31.6 m
    // plunge into the underground tunnel.
    (160.0, 4.5),
    (250.0, 13.5),
    (350.0, 24.0),
    (420.0, 30.0),
    (475.0, 32.0),
    (530.0, 29.5),
    (610.0, 21.0),
    (710.0, 10.5),
    (765.0, 4.5),
    // Underground Launch Complex Tunnel under the central massif.
    (800.0, 0.8),
    (850.0, 0.4),
    (950.0, 0.4),
    (1020.0, 0.8),
    // Climb out of the tunnel to Portal #1 (Earth -> Moon, 1120 m).
    (1070.0, 4.5),
    (1120.0, 9.5),
    // Moon Sector (1120..2895 m): Lunar Truss Bridge, Jump #2, and Crater Split.
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
    (2340.2, 16.5),
    (2690.3, 28.5),
    (2760.0, 33.0),
    (2830.0, 38.5),
    // Portal #2 (Moon -> Earth, 2895 m) and Earth Mountain #2 (45.5 m overpass
    // summit above the tunnel at 3056 m).
    (2895.0, 42.5),
    (2975.0, 44.8),
    (3056.0, 45.5),
    // Steep -40.0 m plunge down into the northern Earth forested valley (5.5 m).
    (3120.0, 43.0),
    (3220.0, 33.0),
    (3330.0, 21.5),
    (3430.0, 11.5),
    (3500.0, 6.5),
    (3555.0, 5.5),
    // Earth Mountain #3: +30.1 m climb up the Western Earth Mountain Ridge,
    // high-altitude Jump #3 (4095 m at 35.6 m), and -35.6 m plunge to finish.
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

/// Alternative left branch through the Lunar Crater Split (`2340.2..2690.3 m`).
const CRATER_LEFT: [Vec2; 11] = [
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

/// Hump rise along `CRATER_LEFT` (m along the branch, m over the main lap).
const CRATER_LEFT_RISE: [(f32, f32); 35] = [
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

/// High plateau margin round the lap, with an indentation under `BRIDGE`
/// (`1470..1590 m`) so the lunar rille chasm drops away beneath the truss bridge.
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

/// Central Earth massif that the Launch Complex Tunnel runs under and the
/// high return road climbs 45 m over.
const STATION_MASSIF: Massif = climbed_massif(550.0, -430.0, 220.0, 46.0);

const MASSIFS: [Massif; 14] = [
    STATION_MASSIF,
    // Earth mountain massifs supporting the Observatory Peak (32 m), the
    // Overpass Summit (45.5 m), and the Western Mountain Ridge (35.6 m).
    climbed_massif(165.0, -395.0, 195.0, 34.0),
    climbed_massif(640.0, -290.0, 210.0, 44.0),
    climbed_massif(400.0, -520.0, 210.0, 36.0),
    climbed_massif(-165.0, -180.0, 220.0, 36.0),
    // Lunar highland massifs supporting the Crater Split and rille rim.
    climbed_massif(880.0, -340.0, 190.0, 20.0),
    climbed_massif(695.0, 20.0, 220.0, 30.0),
    // Scenic peaks on the Earth and Moon halves.
    massif(200.0, -250.0, 65.0, 30.0),
    massif(430.0, -180.0, 85.0, 38.0),
    massif(807.5, 210.0, 60.0, 24.0),
    massif(1120.0, 80.0, 95.0, 44.0),
    massif(260.0, 140.0, 80.0, 32.0),
    massif(-340.0, -220.0, 90.0, 42.0),
    massif(120.0, -720.0, 85.0, 36.0),
];

const CUTTINGS: [Cutting; 5] = [
    // Rock walls flanking the Launch Complex Tunnel portals on Earth.
    Cutting {
        span: span(TUNNEL.from + 40.0, TUNNEL.to + 20.0),
        side: Side::Left,
        height: 18.0,
    },
    Cutting {
        span: span(TUNNEL.from + 40.0, TUNNEL.to + 20.0),
        side: Side::Right,
        height: 18.0,
    },
    // Crater cliff along the Sea of Tranquillity turn on the Moon.
    Cutting {
        span: span(1960.0, 2240.0),
        side: Side::Left,
        height: 22.0,
    },
    // Mountain rock cuttings along the Earth northern descent and final corner.
    Cutting {
        span: span(3220.0, 3480.0),
        side: Side::Left,
        height: 20.0,
    },
    Cutting {
        span: span(4320.0, 4480.0),
        side: Side::Left,
        height: 18.0,
    },
];

/// Distant ringed mountain ranges rising all round the horizon, with a tall
/// peak to the north-east.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(380.0, -80.0);
    let away = point - CENTER;
    let dist = away.length();
    let ring = smoothstep(1450.0, 2600.0, dist)
        * 340.0
        * (0.35 + 0.65 * fbm(point / 280.0 + Vec2::splat(7.3), 4));
    let crater_peak = Vec2::new(1450.0, -2100.0);
    let reach = point.distance(crater_peak) / 1350.0;
    let cone = 520.0 * (1.0 - reach).max(0.0).powf(1.6);
    let caldera = 90.0 * (1.0 - smoothstep(0.0, 0.14, reach));
    ring.max(cone - caldera)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;
    use crate::world::is_earth_realm;

    #[test]
    fn the_lap_goes_through_the_space_station_tunnel_and_then_over_it() {
        let track = Track::new(&MOON_BASE.layout);
        let line = track.centerline();
        let middle = line[track.index_at((TUNNEL.from + TUNNEL.to) / 2.0)];
        assert!(middle.distance(STATION_MASSIF.center) < 0.2 * STATION_MASSIF.radius);

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
        assert!(above > 40.0, "{above} m above");
    }

    #[test]
    fn the_lap_starts_on_earth_with_launchpads_and_observatories_and_crosses_portals_to_the_moon() {
        let space = MOON_BASE
            .scenery
            .space
            .as_ref()
            .expect("Moon Base has Space");
        assert_eq!(space.portals.len(), 2);
        assert!(!space.observatories.is_empty());
        assert!(!space.rockets.is_empty());
        assert!(!space.dishes.is_empty());

        // All launchpads (rockets), observatories, and giant radio dishes stand on Earth.
        for rocket in space.rockets {
            assert!(is_earth_realm(rocket.position), "rocket on Earth");
        }
        for obs in space.observatories {
            assert!(is_earth_realm(obs.position), "observatory on Earth");
        }
        for dish in space.dishes {
            assert!(is_earth_realm(dish.position), "radio dish on Earth");
        }

        let track = Track::new(&MOON_BASE.layout);
        let line = track.centerline();
        let lap = track.lap_length();

        // Every observatory, rocket launchpad, and giant radio dish stands right
        // beside the road (`22..38 m` from the centerline) so it towers in the
        // rider's view without encroaching on the asphalt.
        for obs in space.observatories {
            let d = track.distance_to_centerline(obs.position);
            assert!(
                (22.0..=38.0).contains(&d),
                "observatory at {:?} is {d:.1} m from road centerline",
                obs.position
            );
        }
        for rocket in space.rockets {
            let d = track.distance_to_centerline(rocket.position);
            assert!(
                (22.0..=38.0).contains(&d),
                "rocket at {:?} is {d:.1} m from road centerline",
                rocket.position
            );
        }
        for dish in space.dishes {
            let d = track.distance_to_centerline(dish.position);
            assert!(
                (20.0..=38.0).contains(&d),
                "radio dish at {:?} is {d:.1} m from road centerline",
                dish.position
            );
        }
        assert!(space.arches.len() >= 12);
        assert!(space.biodomes.len() >= 8);

        // The circuit starts on Earth (along = 0 m), and the Earth stretch is longer than the Moon stretch.
        assert!(is_earth_realm(line[0]), "starts on Earth");
        let moon_length = EARTH_PORTAL_OUT - MOON_PORTAL_IN;
        let earth_length = lap - moon_length;
        assert!(
            earth_length > moon_length + 800.0,
            "Earth stretch ({earth_length:.0} m) is longer than Moon stretch ({moon_length:.0} m)"
        );

        // Before the first portal (on Earth) vs after it (on the Moon).
        assert!(is_earth_realm(line[track.index_at(MOON_PORTAL_IN - 25.0)]));
        assert!(!is_earth_realm(line[track.index_at(MOON_PORTAL_IN + 25.0)]));
        // Before the second portal (on the Moon) vs after it (back on Earth).
        assert!(!is_earth_realm(
            line[track.index_at(EARTH_PORTAL_OUT - 25.0)]
        ));
        assert!(is_earth_realm(
            line[track.index_at(EARTH_PORTAL_OUT + 25.0)]
        ));
    }
}
