//! Loop Island: a lap inspired by Green Hill, the first zone of Sonic the
//! Hedgehog. A roller coaster of a road, clockwise round an island of green
//! hills under a blue sky, with the sea all around.
//!
//! 1. The start straight runs north between palm trees and sunflowers, with
//!    a crest where the bike takes off.
//! 2. A long right-hander climbs round the island's own loop, and S-bends
//!    roll on east over the hills. Just after the crest, a branch forks off
//!    to the right, cuts inside the corner and goes round the loop, upside
//!    down at the top, before it joins the lap again on the S-bends.
//! 3. The road climbs into a tunnel through a checkerboard ridge and comes
//!    out by a waterfall, high above the sea, into a long banked right-hander.
//! 4. Down past a second waterfall, a wooden bridge crosses an inlet where
//!    the sea runs in under the road.
//! 5. Rolling S-bends with totem poles lead down to the south shore: a
//!    beach road with palms and a jump, back to the start.

use bevy::prelude::*;

use super::Circuit;
use crate::geometry::{fbm, smoothstep};
use std::f32::consts::FRAC_PI_4;

use crate::track::{Banking, Branch, Layout, VerticalLoop, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const START: Span = span(2060.0, 290.0);
const TUNNEL: Span = span(770.0, 870.0);
const BRIDGE: Span = span(1230.0, 1310.0);
const SHORE: Span = span(1660.0, 1960.0);
/// The stretch of lap the branch with the loop leaves out.
const SHORTCUT: Span = span(215.0, 615.0);

/// Sonic blue and gold.
const BLUE: Color = Color::srgb(0.08, 0.28, 0.85);
const GOLD: Color = Color::srgb(1.0, 0.8, 0.1);
const RED: Color = Color::srgb(0.8, 0.08, 0.06);

pub const LOOP_ISLAND: Circuit = Circuit {
    name: "Loop Island",
    name_en: "Loop Island",
    description: "Inspirada en Sonic: un desvío con un rizo para\n\
                  dar la vuelta, tierra a cuadros y cascadas",
    description_en: "Inspired by Sonic: a branch with a loop to ride,\n\
                     checkered hills and waterfalls",
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // The long right-hander after the tunnel and the last corner,
        // banked like a velodrome.
        banking: &[
            Banking {
                from: 880.0,
                to: 1100.0,
                factor: 1.6,
            },
            Banking {
                from: 1930.0,
                to: 2070.0,
                factor: 1.5,
            },
        ],
        // Inside the first long corner, round the loop.
        branches: &[Branch {
            from: SHORTCUT.from,
            to: SHORTCUT.to,
            // Round to the right, then north-east to the loop.
            approach: &[
                Vec2::new(1.0, -227.6),
                Vec2::new(4.4, -238.9),
                Vec2::new(10.0, -249.3),
                Vec2::new(17.4, -258.3),
            ],
            vertical_loop: VerticalLoop {
                mouth: Vec2::new(70.5, -311.4),
                // North-east.
                heading: -FRAC_PI_4,
                height: 4.5,
                radius: 16.0,
                shift: -16.0,
            },
            // On north-east, round to the right, and onto the lap.
            exit: &[
                Vec2::new(124.2, -342.5),
                Vec2::new(132.4, -349.5),
                Vec2::new(141.5, -355.2),
                Vec2::new(151.5, -359.4),
                Vec2::new(161.9, -362.0),
                Vec2::new(188.5, -366.5),
                Vec2::new(215.1, -371.0),
                Vec2::new(241.6, -375.6),
            ],
        }],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 20.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [GOLD, BLUE],
        walls: &[
            // Hedges along the start straight, checkerboard earth elsewhere.
            WallRun {
                span: START,
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: START,
                side: Side::Left,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Checkered,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Checkered,
            },
        ],
        tunnels: &[TUNNEL],
        bridges: &[BRIDGE],
        gantries: &[
            Gantry {
                along: 0.0,
                color: BLUE,
                text: Some("LOOP ISLAND"),
            },
            Gantry {
                along: 700.0,
                color: RED,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1420.0,
                color: RED,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 150.0,
                side: Side::Right,
                text: "MOTORUST",
                color: Color::srgb(0.1, 0.1, 0.12),
                accent: Some((4, crate::ui::RUST_ORANGE)),
            },
            Billboard {
                along: 620.0,
                side: Side::Left,
                text: "GO FAST!",
                color: BLUE,
                accent: None,
            },
            Billboard {
                along: 1870.0,
                side: Side::Right,
                text: "ZONE 1",
                color: Color::srgb(0.1, 0.5, 0.2),
                accent: None,
            },
        ],
        lamps: &[],
        trees: &[Species::Broadleaf],
        forests: &[],
        houses: Houses {
            max: 0,
            paints: &[],
            spans: &[],
        },
        palms: &[
            Row {
                span: START,
                side: Side::Left,
                lateral: WALL_DISTANCE + 6.0,
                every: 22.0,
            },
            Row {
                span: span(110.0, 200.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 6.0,
                every: 26.0,
            },
            Row {
                span: span(300.0, 520.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 8.0,
                every: 30.0,
            },
            Row {
                span: span(1350.0, 1640.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 6.0,
                every: 34.0,
            },
            Row {
                span: SHORE,
                side: Side::Left,
                lateral: WALL_DISTANCE + 7.0,
                every: 20.0,
            },
            // Further back, among the other trees.
            Row {
                span: START,
                side: Side::Left,
                lateral: WALL_DISTANCE + 22.0,
                every: 24.0,
            },
            Row {
                span: span(520.0, 740.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 9.0,
                every: 28.0,
            },
            Row {
                span: span(640.0, 740.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 9.0,
                every: 28.0,
            },
            Row {
                span: span(1350.0, 1640.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 6.0,
                every: 30.0,
            },
            Row {
                span: SHORE,
                side: Side::Right,
                lateral: WALL_DISTANCE + 20.0,
                every: 26.0,
            },
            Row {
                span: span(1960.0, 2060.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 8.0,
                every: 22.0,
            },
        ],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[Boat {
            position: Vec2::new(150.0, 260.0),
            heading: 1.2,
            kind: BoatKind::Sailboat,
        }],
        chevrons: &[
            ChevronSign {
                along: 1630.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 1935.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
        ],
        grandstands: &[Grandstand {
            along: 70.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 5.0,
            length: 32.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Checkered,
        bridge_style: BridgeStyle::Wooden,
        sunflowers: &[
            Row {
                span: span(20.0, 280.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 2.5,
                every: 12.0,
            },
            Row {
                span: span(520.0, 740.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 2.5,
                every: 14.0,
            },
            Row {
                span: span(640.0, 740.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 2.5,
                every: 14.0,
            },
            Row {
                span: SHORE,
                side: Side::Right,
                lateral: WALL_DISTANCE + 2.5,
                every: 12.0,
            },
        ],
        totems: &[
            Totem {
                along: 40.0,
                side: Side::Left,
            },
            Totem {
                along: 690.0,
                side: Side::Right,
            },
            Totem {
                along: 1400.0,
                side: Side::Left,
            },
            Totem {
                along: 1560.0,
                side: Side::Right,
            },
            Totem {
                along: 2000.0,
                side: Side::Left,
            },
        ],
        // The loop to ride is the branch's (see the layout).
        loops: &[
            // Inland from the south shore.
            Loop {
                along: 1590.0,
                side: Side::Right,
                lateral: 45.0,
                radius: 11.0,
            },
        ],
        rings: &[],
        waterfalls: &[
            Waterfall {
                along: 945.0,
                side: Side::Left,
                width: 16.0,
            },
            Waterfall {
                along: 1175.0,
                side: Side::Right,
                width: 14.0,
            },
        ],
    },
};

const CONTROL_POINTS: [Vec2; 30] = [
    // 1. The start straight, north (-Z).
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -110.0),
    Vec2::new(0.0, -220.0),
    Vec2::new(6.0, -300.0),
    // 2. The long right-hander past the loop, and S-bends east.
    Vec2::new(35.0, -360.0),
    Vec2::new(90.0, -395.0),
    Vec2::new(160.0, -400.0),
    Vec2::new(225.0, -380.0),
    Vec2::new(290.0, -385.0),
    Vec2::new(345.0, -420.0),
    // 3. Up through the tunnel and round the long right-hander.
    Vec2::new(390.0, -470.0),
    Vec2::new(450.0, -500.0),
    Vec2::new(520.0, -490.0),
    Vec2::new(565.0, -445.0),
    Vec2::new(575.0, -380.0),
    Vec2::new(555.0, -320.0),
    // 4. Down past the waterfall to the wooden bridge.
    Vec2::new(520.0, -270.0),
    Vec2::new(505.0, -200.0),
    Vec2::new(515.0, -130.0),
    // 5. Rolling S-bends down to the south shore, and back to the start.
    Vec2::new(540.0, -60.0),
    Vec2::new(530.0, 10.0),
    Vec2::new(485.0, 55.0),
    Vec2::new(420.0, 70.0),
    Vec2::new(355.0, 60.0),
    Vec2::new(290.0, 80.0),
    Vec2::new(220.0, 115.0),
    Vec2::new(140.0, 125.0),
    Vec2::new(70.0, 120.0),
    Vec2::new(25.0, 92.0),
    Vec2::new(3.0, 50.0),
];

/// Ups and downs all the way round: crests where the bike takes off on the
/// start straight, after the bridge and on the shore, and the high ground of
/// the tunnel.
const ELEVATION: [(f32, f32); 32] = [
    (0.0, 0.0),
    (100.0, 0.5),
    (160.0, 4.5),
    // Crest: flat out, the bike flies.
    (195.0, 6.5),
    (235.0, 3.5),
    (300.0, 1.5),
    (380.0, 3.0),
    (470.0, 7.0),
    (540.0, 9.0),
    (600.0, 7.5),
    (660.0, 10.0),
    (720.0, 14.0),
    (780.0, 17.0),
    (840.0, 18.0),
    (900.0, 17.0),
    (980.0, 13.0),
    (1060.0, 9.0),
    (1140.0, 6.0),
    (1200.0, 5.0),
    (1300.0, 5.0),
    (1360.0, 7.5),
    (1410.0, 10.0),
    (1450.0, 7.5),
    (1520.0, 4.0),
    (1600.0, 3.0),
    (1680.0, 4.5),
    (1740.0, 7.0),
    (1775.0, 8.5),
    (1815.0, 5.5),
    (1880.0, 3.0),
    (1960.0, 2.0),
    (2040.0, 0.8),
];

/// The island: how far out from the centre line the land goes (m), with an
/// inlet where the sea runs in under the bridge.
const COAST: [(f32, f32); 16] = [
    (0.0, 140.0),
    (280.0, 160.0),
    (400.0, 220.0),
    (700.0, 220.0),
    (880.0, 140.0),
    (1050.0, 70.0),
    (1180.0, 45.0),
    (1200.0, 40.0),
    (1235.0, -170.0),
    (1305.0, -170.0),
    (1340.0, 40.0),
    (1500.0, 40.0),
    (1650.0, 45.0),
    (1950.0, 45.0),
    (2000.0, 70.0),
    (2060.0, 120.0),
];

const MASSIFS: [Massif; 10] = [
    // The ridge the tunnel goes through.
    massif(430.0, -505.0, 80.0, 32.0),
    massif(395.0, -565.0, 60.0, 26.0),
    massif(475.0, -440.0, 55.0, 22.0),
    // Checkerboard mesas inside the lap.
    massif(200.0, -260.0, 70.0, 28.0),
    massif(330.0, -320.0, 55.0, 22.0),
    massif(180.0, -50.0, 55.0, 16.0),
    massif(370.0, -70.0, 50.0, 18.0),
    // Hills behind the loop.
    massif(110.0, -520.0, 75.0, 30.0),
    // Islets.
    massif(-220.0, -160.0, 30.0, 12.0),
    massif(300.0, 260.0, 28.0, 10.0),
];

const CUTTINGS: [Cutting; 5] = [
    // The tunnel, with earth on both sides from before the mouth.
    Cutting {
        span: span(740.0, 900.0),
        side: Side::Left,
        height: 22.0,
    },
    Cutting {
        span: span(740.0, 900.0),
        side: Side::Right,
        height: 22.0,
    },
    // The walls the waterfalls pour down.
    Cutting {
        span: span(900.0, 995.0),
        side: Side::Left,
        height: 18.0,
    },
    Cutting {
        span: span(1120.0, 1225.0),
        side: Side::Right,
        height: 18.0,
    },
    // Earth on the inside of the S-bends towards the shore.
    Cutting {
        span: span(1590.0, 1700.0),
        side: Side::Right,
        height: 14.0,
    },
];

/// Across the sea, checkerboard mountains to the north and west, as in the
/// background of Sonic's Green Hill; to the south and east, open sea.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(280.0, -200.0);
    let away = point - CENTER;
    let facing = smoothstep(
        -0.1,
        0.5,
        away.normalize_or_zero().dot(Vec2::new(-0.6, -0.8)),
    );
    smoothstep(1500.0, 2700.0, away.length()) * facing * 260.0 * (0.4 + 0.6 * fbm(point / 320.0, 4))
}
