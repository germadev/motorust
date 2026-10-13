//! Loop Island: a lap inspired by Green Hill, the first zone of Sonic the
//! Hedgehog. A roller coaster of a road, clockwise round an island of green
//! hills under a blue sky, with the sea all around.
//!
//! 1. The start straight runs north between palm trees and sunflowers, with
//!    a crest where the bike takes off.
//! 2. A long right-hander climbs round the island's own loop, and S-bends
//!    roll on east over the hills. On the climb to the crest, a branch forks
//!    off to the right, sweeps inside the corner and goes round the loop,
//!    upside down at the top, before it joins the lap again on the S-bends.
//! 3. The road climbs into a tunnel through a checkerboard ridge and comes
//!    out by a waterfall, high above the sea, into a long banked right-hander.
//! 4. Down past a second waterfall, a wooden bridge crosses an inlet where
//!    the sea runs in under the road.
//! 5. Rolling S-bends with totem poles lead down to the south shore: a
//!    beach road with palms, back to the start.

use bevy::prelude::*;

use super::{Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use std::f32::consts::FRAC_PI_4;

use crate::track::{Banking, Branch, Layout, VerticalLoop, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const START: Span = span(2060.0, 290.0);
const TUNNEL: Span = span(770.0, 870.0);
const BRIDGE: Span = span(1230.0, 1310.0);
const SHORE: Span = span(1660.0, 1960.0);
/// The stretch of lap the branch with the loop leaves out: it forks off on
/// the climb to the crest of the start straight, so the turn off the lap is
/// a long, gentle one.
const SHORTCUT: Span = span(110.0, 637.0);

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
    song: Song {
        mp3: include_bytes!("../../assets/music/loop_island.mp3"),
        loudness: -13.3,
    },
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
            // A long right-hander (about 160 m radius) round to the north-east,
            // over the shoulder of the crest and gently down to the loop.
            approach: &[
                Vec2::new(0.5, -122.5),
                Vec2::new(2.0, -135.0),
                Vec2::new(4.4, -147.3),
                Vec2::new(7.8, -159.4),
                Vec2::new(12.2, -171.2),
                Vec2::new(17.4, -182.6),
                Vec2::new(23.6, -193.6),
                Vec2::new(30.6, -204.0),
                Vec2::new(38.3, -213.9),
            ],
            vertical_loop: Some(VerticalLoop {
                mouth: Vec2::new(82.2, -258.5),
                // North-east.
                heading: -FRAC_PI_4,
                height: 5.5,
                radius: 16.0,
                shift: -16.0,
            }),
            // On north-east, round to the right (200 m radius), straight on
            // towards the lap, and onto it in a last long right-hander
            // (340 m radius), climbing gently all the way.
            exit: &[
                Vec2::new(137.5, -290.6),
                Vec2::new(146.5, -298.2),
                Vec2::new(156.3, -305.9),
                Vec2::new(166.1, -313.6),
                Vec2::new(175.8, -321.3),
                Vec2::new(185.6, -329.0),
                Vec2::new(195.1, -336.2),
                Vec2::new(204.8, -343.0),
                Vec2::new(214.7, -349.5),
                Vec2::new(224.8, -355.7),
                Vec2::new(235.2, -361.5),
                Vec2::new(245.7, -366.9),
                Vec2::new(256.5, -372.0),
                Vec2::new(267.4, -376.7),
                Vec2::new(278.4, -381.0),
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
            // Just before the branch forks off to the right.
            Billboard {
                along: 95.0,
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
            // Inside the first long corner, round the branch.
            Row {
                span: span(220.0, 520.0),
                side: Side::Right,
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
        sky: SkyStyle::Day,
        mine: None,
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

/// Ups and downs all the way round: a crest where the bike takes off on the
/// start straight, gentle rises over the S-bends and along the shore (none
/// to fly off in a bend), and the high ground of the tunnel.
const ELEVATION: [(f32, f32); 31] = [
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
    // A long, gentle rise over the S-bends: no crest to fly off in them.
    (1360.0, 6.8),
    (1420.0, 8.0),
    (1480.0, 6.6),
    (1540.0, 4.0),
    (1600.0, 3.0),
    // The shore road rolls gently: its bends come too close together to
    // land a jump between them.
    (1680.0, 4.3),
    (1760.0, 6.4),
    (1840.0, 4.4),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    #[test]
    fn the_branch_forks_before_the_crest_and_turns_gently() {
        let track = Track::new(&LOOP_ISLAND.layout);
        // The crest of the start straight, where the bike takes off.
        let crest = (0..track.index_at(START.to))
            .max_by(|&a, &b| track.elevations()[a].total_cmp(&track.elevations()[b]))
            .unwrap();
        let crest = crest as f32 * track.step_length();
        assert!(SHORTCUT.from < crest - 50.0, "the crest is at {crest} m");
        // The road to the loop turns off the lap in a long, easy curve, and
        // the road back is no tighter than the lap where it joins it.
        let looped = track.loops()[0];
        for (road, tightest) in [(looped.approach, 100.0), (looped.exit, 75.0)] {
            let road = track.road(road);
            for i in 1..road.sample_count() - 1 {
                let radius = 1.0 / road.curvature(i).abs();
                assert!(radius > tightest, "{radius} m at {i}");
            }
        }
    }
}
