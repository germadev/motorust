//! Speed Bay, como la de Moto Racer: una vuelta en el sentido de las agujas
//! del reloj a una península, con el mar siempre a la izquierda.
//!
//! 1. La recta de meta es un paseo marítimo: barandilla verde, playa y mar a
//!    la izquierda; palmeras y hoteles altos de colores a la derecha.
//! 2. Al final, tras un cambio de rasante, una curva a la derecha que baja
//!    fuerte y peraltada y otra a la izquierda que sube hasta una cresta de
//!    roca, que la carretera cruza subiendo por un túnel.
//! 3. A la salida, bajando, curvas enlazadas junto a un acantilado y una
//!    curva a la derecha hacia un gran hotel de los de antes, sobre el mar.
//! 4. Un paso estrecho entre rocas con carteles y la carretera de la costa
//!    este, entre colinas verdes, con el mar tras una barrera blanca.
//! 5. Un bosque de árboles altos con una curva larga a la derecha.
//! 6. Un puente de arcos rojos sobre una ría, un acantilado a la derecha y
//!    el pueblo, con sus soportales, antes de volver al paseo.

use bevy::prelude::*;

use super::Circuit;
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Layout, WALL_DISTANCE};
use crate::world::scenery::*;

/// Dónde están las cosas, en metros desde la meta.
const SEAFRONT: Span = span(2560.0, 260.0);
const TUNNEL: Span = span(510.0, 750.0);
const FOREST: Span = span(1640.0, 1960.0);
const BRIDGE: Span = span(2340.0, 2420.0);
const TOWN: Span = span(2430.0, 2580.0);

pub const SPEED_BAY: Circuit = Circuit {
    name: "Speed Bay",
    name_en: "Speed Bay",
    description: "La de Moto Racer: paseo marítimo con palmeras,\n\
                  túnel, gran hotel, bosque y puente rojo",
    description_en: "The Moto Racer classic: palm-lined promenade,\n\
                     tunnel, grand hotel, forest and red bridge",
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // La bajada tras el cambio de rasante, peraltada.
        banking: &[
            Banking {
                from: 320.0,
                to: 450.0,
                factor: 2.5,
            },
            Banking {
                from: 560.0,
                to: 680.0,
                factor: 2.0,
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 14.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[
            // El gran hotel, sobre el mar.
            Terrace {
                center: Vec2::new(215.0, -425.0),
                radius: 48.0,
                height: 16.0,
            },
        ],
        mountains,
        curbs: [Color::srgb(0.1, 0.25, 0.75), Color::srgb(0.95, 0.95, 0.95)],
        walls: &[
            WallRun {
                span: SEAFRONT,
                side: Side::Left,
                style: WallStyle::Railing,
            },
            WallRun {
                span: TOWN,
                side: Side::Left,
                style: WallStyle::Railing,
            },
            WallRun {
                span: span(2480.0, 250.0),
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: span(1250.0, 1600.0),
                side: Side::Left,
                style: WallStyle::Concrete,
            },
        ],
        tunnels: &[TUNNEL],
        bridges: &[BRIDGE],
        gantries: &[
            Gantry {
                along: 0.0,
                color: Color::srgb(0.08, 0.22, 0.75),
                text: Some("SPEED BAY"),
            },
            Gantry {
                along: 1100.0,
                color: Color::srgb(0.8, 0.08, 0.06),
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1935.0,
                color: Color::srgb(0.8, 0.08, 0.06),
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[
            Billboard {
                along: 1120.0,
                side: Side::Right,
                text: "MOTORUST",
                color: Color::srgb(0.1, 0.1, 0.12),
                // RUST, del color del óxido, como en el título.
                accent: Some((4, crate::ui::RUST_ORANGE)),
            },
            Billboard {
                along: 1150.0,
                side: Side::Right,
                text: "RACING",
                color: Color::srgb(0.1, 0.2, 0.7),
                accent: None,
            },
            Billboard {
                along: 2450.0,
                side: Side::Left,
                text: "BEVY",
                color: Color::srgb(0.1, 0.1, 0.1),
                accent: None,
            },
        ],
        lamps: &[Row {
            span: SEAFRONT,
            side: Side::Left,
            lateral: WALL_DISTANCE + 0.9,
            every: 40.0,
        }],
        trees: &[Species::Broadleaf, Species::Cypress, Species::UmbrellaPine],
        forests: &[FOREST],
        houses: Houses {
            max: 9,
            paints: &[
                Color::srgb(0.95, 0.86, 0.66),
                Color::srgb(0.97, 0.95, 0.9),
                Color::srgb(0.93, 0.75, 0.52),
            ],
            spans: &[TOWN, span(1060.0, 1160.0)],
        },
        palms: &[
            Row {
                span: span(2480.0, 250.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 4.0,
                every: 15.0,
            },
            Row {
                span: span(2500.0, 220.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 6.0,
                every: 38.0,
            },
        ],
        hotels: &[Row {
            span: span(2500.0, 240.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 30.0,
            every: 38.0,
        }],
        grand_hotel: Some((880.0, 50.0)),
        buoys: &[
            Vec2::new(-62.0, -60.0),
            Vec2::new(-80.0, -190.0),
            Vec2::new(-58.0, 80.0),
            Vec2::new(-95.0, -270.0),
        ],
        lighthouses: &[Lighthouse {
            position: Vec2::new(-180.0, -780.0),
            height: 24.0,
        }],
        boats: &[
            Boat {
                position: Vec2::new(-70.0, -100.0),
                heading: 0.6,
                kind: BoatKind::Sailboat,
            },
            Boat {
                position: Vec2::new(-110.0, -220.0),
                heading: -0.9,
                kind: BoatKind::Yacht,
            },
            Boat {
                position: Vec2::new(-80.0, 40.0),
                heading: 1.4,
                kind: BoatKind::Sailboat,
            },
            Boat {
                position: Vec2::new(265.0, 160.0),
                heading: -2.1,
                kind: BoatKind::Sailboat,
            },
        ],
        chevrons: &[
            ChevronSign {
                along: 320.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 440.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 610.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1090.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 1750.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
        ],
        grandstands: &[Grandstand {
            along: 70.0,
            side: Side::Right,
            lateral: WALL_DISTANCE + 4.5,
            length: 36.0,
        }],
        beach_parasols: &[
            ParasolCluster {
                position: Vec2::new(-32.0, -140.0),
                count: 6,
            },
            ParasolCluster {
                position: Vec2::new(-30.0, -50.0),
                count: 5,
            },
        ],
        brake_markers: &[
            BrakeMarker {
                along: 240.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 270.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 400.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 430.0,
                side: Side::Right,
                distance: 50,
            },
        ],
    },
};

const CONTROL_POINTS: [Vec2; 44] = [
    // 1. Paseo marítimo, hacia el norte (-Z)
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -85.0),
    Vec2::new(0.0, -170.0),
    Vec2::new(-2.0, -245.0),
    // 2. Cresta, bajada peraltada y subida hacia el túnel
    Vec2::new(12.0, -310.0),
    Vec2::new(32.0, -370.0),
    Vec2::new(52.0, -440.0),
    Vec2::new(65.0, -510.0),
    // Túnel y arco norte de la península (oeste)
    Vec2::new(65.0, -560.0),
    Vec2::new(79.6, -595.4),
    Vec2::new(115.0, -610.0),
    Vec2::new(150.4, -595.4),
    Vec2::new(165.0, -560.0),
    // 3. Descenso junto al acantilado y el gran hotel hacia el sur (+Z)
    Vec2::new(165.0, -510.0),
    Vec2::new(164.0, -430.0),
    Vec2::new(163.0, -350.0),
    Vec2::new(162.5, -260.0),
    // 4. Paso estrecho entre rocas con carteles y Checkpoint 1 (horquilla)
    Vec2::new(162.5, -200.0),
    Vec2::new(175.7, -168.2),
    Vec2::new(207.5, -155.0),
    Vec2::new(239.3, -168.2),
    Vec2::new(252.5, -200.0),
    // 5. Costa este, subiendo hacia el bosque (montaña rusa)
    Vec2::new(252.5, -260.0),
    Vec2::new(252.0, -340.0),
    Vec2::new(252.0, -420.0),
    Vec2::new(252.0, -510.0),
    // 6. Bosque en lo alto, curva amplia a derechas y Checkpoint 2
    Vec2::new(252.0, -570.0),
    Vec2::new(266.1, -603.9),
    Vec2::new(300.0, -618.0),
    Vec2::new(333.9, -603.9),
    Vec2::new(348.0, -570.0),
    // 7. Bajada desde el bosque hacia el puente de arcos rojos
    Vec2::new(348.0, -510.0),
    Vec2::new(340.0, -400.0),
    Vec2::new(325.0, -280.0),
    Vec2::new(305.0, -160.0),
    Vec2::new(275.0, -50.0),
    Vec2::new(235.0, 40.0),
    // 8. El puente sobre la ría y el pueblo con soportales de vuelta a meta
    Vec2::new(185.0, 85.0),
    Vec2::new(135.0, 100.0),
    Vec2::new(80.0, 95.0),
    Vec2::new(35.0, 80.0),
    Vec2::new(14.0, 58.0),
    Vec2::new(2.0, 35.0),
    Vec2::new(0.0, 18.0),
];

const ELEVATION: [(f32, f32); 39] = [
    (0.0, 0.0),
    (150.0, 0.0),
    (210.0, 0.5),
    (290.0, 4.8),
    (330.0, 6.8),
    (390.0, 3.8),
    (440.0, 1.2),
    (470.0, 0.6),
    (530.0, 3.5),
    (610.0, 9.5),
    (690.0, 15.5),
    (770.0, 21.0),
    (810.0, 22.5),
    (880.0, 17.5),
    (960.0, 12.5),
    (1040.0, 9.0),
    (1110.0, 8.2),
    (1180.0, 8.0),
    (1240.0, 9.0),
    (1310.0, 14.0),
    (1350.0, 16.5),
    (1420.0, 11.5),
    (1470.0, 7.8),
    (1540.0, 13.0),
    (1580.0, 15.8),
    (1650.0, 10.8),
    (1700.0, 7.0),
    (1770.0, 12.0),
    (1840.0, 17.0),
    (1910.0, 22.0),
    (1960.0, 24.5),
    (2040.0, 19.0),
    (2120.0, 14.0),
    (2200.0, 10.0),
    (2270.0, 8.5),
    (2350.0, 6.5),
    (2430.0, 4.0),
    (2510.0, 1.5),
    (2580.0, 0.4),
];

const COAST: [(f32, f32); 20] = [
    (0.0, 34.0),
    (270.0, 34.0),
    (360.0, 80.0),
    (500.0, 130.0),
    (700.0, 130.0),
    (850.0, 65.0),
    (1000.0, 45.0),
    (1100.0, 45.0),
    (1300.0, 32.0),
    (1700.0, 32.0),
    (1850.0, 80.0),
    (2000.0, 65.0),
    (2200.0, 45.0),
    (2300.0, 38.0),
    (2340.0, -170.0),
    (2410.0, -170.0),
    (2440.0, 38.0),
    (2520.0, 34.0),
    (2600.0, 34.0),
    (2680.0, 34.0),
];

const MASSIFS: [Massif; 8] = [
    // La cresta que cruza el túnel.
    massif(115.0, -620.0, 90.0, 30.0),
    massif(70.0, -580.0, 70.0, 26.0),
    massif(160.0, -580.0, 70.0, 26.0),
    // Rocas sobre el pueblo y el paso entre rocas.
    massif(100.0, 40.0, 50.0, 18.0),
    massif(205.0, -150.0, 55.0, 20.0),
    // Cresta del bosque.
    massif(300.0, -620.0, 80.0, 28.0),
    // Islotes.
    massif(-180.0, -780.0, 32.0, 13.0),
    massif(460.0, -120.0, 36.0, 15.0),
];

const CUTTINGS: [Cutting; 7] = [
    // El túnel, con roca a los dos lados desde antes de entrar.
    Cutting {
        span: span(510.0, 750.0),
        side: Side::Left,
        height: 24.0,
    },
    Cutting {
        span: span(510.0, 750.0),
        side: Side::Right,
        height: 24.0,
    },
    // El acantilado a la salida del túnel.
    Cutting {
        span: span(750.0, 920.0),
        side: Side::Right,
        height: 16.0,
    },
    // El paso entre rocas, después del hotel.
    Cutting {
        span: span(1060.0, 1180.0),
        side: Side::Right,
        height: 14.0,
    },
    Cutting {
        span: span(1060.0, 1180.0),
        side: Side::Left,
        height: 10.0,
    },
    // Roca a la izquierda antes del puente y a la derecha después.
    Cutting {
        span: span(2250.0, 2330.0),
        side: Side::Left,
        height: 14.0,
    },
    Cutting {
        span: span(2430.0, 2520.0),
        side: Side::Right,
        height: 16.0,
    },
];

/// Al otro lado del mar, una costa montañosa al este y al norte.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(220.0, -400.0);
    let away = point - CENTER;
    let distance = away.length();
    // Solo hacia el este y el norte; al oeste y al sur, mar abierto.
    let facing = smoothstep(
        -0.2,
        0.5,
        away.normalize_or_zero().dot(Vec2::new(0.6, -0.8)),
    );
    smoothstep(1700.0, 2600.0, distance) * facing * 240.0 * (0.35 + 0.65 * fbm(point / 280.0, 4))
}
