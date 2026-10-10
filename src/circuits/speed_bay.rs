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
const SEAFRONT: Span = span(2440.0, 300.0);
const TUNNEL: Span = span(600.0, 780.0);
const FOREST: Span = span(1790.0, 2110.0);
const BRIDGE: Span = span(2172.0, 2288.0);
const TOWN: Span = span(2350.0, 2560.0);

pub const SPEED_BAY: Circuit = Circuit {
    name: "Speed Bay",
    description: "La de Moto Racer: paseo marítimo con palmeras,\n\
                  túnel, gran hotel, bosque y puente rojo",
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // La curva a la derecha que baja hacia el túnel y la de la izquierda
        // que sube hasta él, muy peraltadas.
        banking: &[Banking {
            from: 330.0,
            to: 620.0,
            factor: 2.5,
        }],
    },
    scenery: Scenery {
        coast: Coast::AroundTrack(&COAST),
        hills: 14.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[
            // El gran hotel, sobre el mar.
            Terrace {
                center: Vec2::new(150.0, -1010.0),
                radius: 48.0,
                height: 24.0,
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
                span: span(2380.0, 300.0),
                side: Side::Right,
                style: WallStyle::Hedge,
            },
            WallRun {
                span: span(1230.0, 1760.0),
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
                along: 1140.0,
                side: Side::Right,
                text: "MOTORUST",
                color: Color::srgb(0.1, 0.1, 0.12),
                // RUST, del color del óxido, como en el título.
                accent: Some((4, crate::ui::RUST_ORANGE)),
            },
            Billboard {
                along: 1170.0,
                side: Side::Right,
                text: "RACING",
                color: Color::srgb(0.1, 0.2, 0.7),
                accent: None,
            },
            Billboard {
                along: 2330.0,
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
                span: span(2400.0, 300.0),
                side: Side::Right,
                lateral: WALL_DISTANCE + 4.0,
                every: 15.0,
            },
            Row {
                span: span(2420.0, 260.0),
                side: Side::Left,
                lateral: WALL_DISTANCE + 6.0,
                every: 38.0,
            },
        ],
        hotels: &[Row {
            span: span(2470.0, 290.0),
            side: Side::Right,
            lateral: WALL_DISTANCE + 30.0,
            every: 38.0,
        }],
        grand_hotel: Some((1010.0, 75.0)),
        buoys: &[
            Vec2::new(-62.0, -60.0),
            Vec2::new(-80.0, -190.0),
            Vec2::new(-58.0, 80.0),
            Vec2::new(-95.0, -300.0),
        ],
        lighthouses: &[Lighthouse {
            position: Vec2::new(-190.0, -880.0),
            height: 24.0,
        }],
        boats: &[
            Boat {
                position: Vec2::new(-70.0, -100.0),
                heading: 0.6,
                kind: BoatKind::Sailboat,
            },
            Boat {
                position: Vec2::new(-110.0, -240.0),
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
                along: 370.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 540.0,
                side: Side::Right,
                direction: ChevronDirection::Left,
            },
            ChevronSign {
                along: 840.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 940.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
            ChevronSign {
                along: 2020.0,
                side: Side::Left,
                direction: ChevronDirection::Right,
            },
        ],
        grandstands: &[Grandstand {
            along: 80.0,
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
                along: 280.0,
                side: Side::Left,
                distance: 100,
            },
            BrakeMarker {
                along: 310.0,
                side: Side::Left,
                distance: 50,
            },
            BrakeMarker {
                along: 500.0,
                side: Side::Right,
                distance: 100,
            },
            BrakeMarker {
                along: 530.0,
                side: Side::Right,
                distance: 50,
            },
        ],
    },
};

const CONTROL_POINTS: [Vec2; 29] = [
    // El paseo marítimo, hacia el norte.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -150.0),
    Vec2::new(-4.0, -290.0),
    // Suave a la izquierda y a la derecha, hacia la cresta del túnel.
    Vec2::new(-30.0, -370.0),
    Vec2::new(-25.0, -445.0),
    Vec2::new(10.0, -505.0),
    // El túnel.
    Vec2::new(35.0, -570.0),
    Vec2::new(33.0, -680.0),
    Vec2::new(45.0, -770.0),
    // Curvas enlazadas y curva a la derecha.
    Vec2::new(40.0, -850.0),
    Vec2::new(75.0, -905.0),
    // El gran hotel y el paso entre rocas.
    Vec2::new(160.0, -930.0),
    Vec2::new(235.0, -905.0),
    Vec2::new(280.0, -860.0),
    Vec2::new(340.0, -820.0),
    // La costa este, hacia el sur.
    Vec2::new(385.0, -740.0),
    Vec2::new(395.0, -630.0),
    Vec2::new(420.0, -530.0),
    Vec2::new(415.0, -420.0),
    // Suave a la izquierda y el bosque, con su curva larga a la derecha.
    Vec2::new(445.0, -330.0),
    Vec2::new(460.0, -220.0),
    Vec2::new(440.0, -110.0),
    Vec2::new(385.0, -30.0),
    // El puente sobre la ría.
    Vec2::new(310.0, 40.0),
    Vec2::new(230.0, 95.0),
    // El acantilado, la curva a la derecha y el pueblo.
    Vec2::new(160.0, 125.0),
    Vec2::new(95.0, 145.0),
    Vec2::new(32.0, 132.0),
    Vec2::new(0.0, 70.0),
];

/// La altimetría, sacada de una vuelta grabada de Moto Racer, donde las
/// cuestas son lo que más se nota: el paseo y el pueblo, llanos a ras del
/// mar; al final del paseo, un cambio de rasante y una curva a la derecha que
/// baja fuerte y peraltada casi hasta el agua; una curva a la izquierda que
/// sube hasta el túnel, que se cruza subiendo; a la salida, bajada entre los
/// acantilados y casi llano junto al gran hotel; una bajada por el paso entre
/// rocas y la carretera de la costa, una montaña rusa: cambio de rasante,
/// hondonada, otro cambio de rasante con el mar delante, bajada a ras del
/// agua junto a la barrera blanca y una subida fuerte hasta el bosque, que se
/// baja entre curvas hasta el puente.
const ELEVATION: [(f32, f32); 35] = [
    (0.0, 0.0),
    (150.0, 0.0),
    (230.0, 0.6),
    // El cambio de rasante al final del paseo y la bajada de la curva.
    (300.0, 4.5),
    (335.0, 6.2),
    (385.0, 3.6),
    (430.0, 1.0),
    (465.0, 0.4),
    // La subida hasta el túnel y por dentro de él.
    (530.0, 2.5),
    (600.0, 6.0),
    (690.0, 10.0),
    (780.0, 14.0),
    (820.0, 15.0),
    // La bajada a la salida, junto al acantilado.
    (900.0, 10.5),
    (970.0, 8.0),
    (1040.0, 8.0),
    (1110.0, 8.5),
    (1180.0, 9.5),
    (1260.0, 7.0),
    // La montaña rusa de la costa.
    (1336.0, 11.5),
    (1430.0, 5.0),
    (1495.0, 9.0),
    (1600.0, 2.5),
    (1640.0, 3.0),
    (1700.0, 8.0),
    (1780.0, 16.5),
    (1810.0, 17.5),
    // El bosque, bajando hasta el puente.
    (1900.0, 15.0),
    (2000.0, 13.0),
    (2100.0, 11.0),
    (2172.0, 10.0),
    (2288.0, 9.5),
    (2350.0, 7.0),
    (2420.0, 4.0),
    (2480.0, 1.0),
];

/// Hasta dónde llega la tierra por fuera del circuito (metros desde la meta
/// y distancia a la línea central). En el puente, el mar entra por debajo.
const COAST: [(f32, f32); 22] = [
    (0.0, 34.0),
    (290.0, 34.0),
    (380.0, 90.0),
    (500.0, 170.0),
    (720.0, 170.0),
    (830.0, 70.0),
    (900.0, 50.0),
    (960.0, 130.0),
    (1060.0, 140.0),
    (1150.0, 55.0),
    (1250.0, 27.0),
    (1700.0, 27.0),
    (1790.0, 70.0),
    (1900.0, 110.0),
    (2090.0, 80.0),
    (2160.0, 40.0),
    (2195.0, -170.0),
    (2265.0, -170.0),
    (2300.0, 38.0),
    (2400.0, 32.0),
    (2500.0, 34.0),
    (2600.0, 34.0),
];

const MASSIFS: [Massif; 8] = [
    // La cresta que cruza el túnel.
    massif(-60.0, -630.0, 90.0, 30.0),
    massif(35.0, -690.0, 110.0, 34.0),
    massif(135.0, -660.0, 80.0, 26.0),
    // Rocas sobre el pueblo y junto a la costa este.
    massif(120.0, 40.0, 55.0, 18.0),
    massif(300.0, -620.0, 60.0, 22.0),
    // Islotes.
    massif(-190.0, -880.0, 32.0, 13.0),
    massif(560.0, -120.0, 36.0, 15.0),
    massif(300.0, 250.0, 28.0, 11.0),
];

const CUTTINGS: [Cutting; 7] = [
    // El túnel, con roca a los dos lados desde antes de entrar.
    Cutting {
        span: span(560.0, 820.0),
        side: Side::Left,
        height: 24.0,
    },
    Cutting {
        span: span(560.0, 820.0),
        side: Side::Right,
        height: 24.0,
    },
    // El acantilado a la salida del túnel.
    Cutting {
        span: span(820.0, 990.0),
        side: Side::Right,
        height: 16.0,
    },
    // El paso entre rocas, después del hotel.
    Cutting {
        span: span(1080.0, 1185.0),
        side: Side::Right,
        height: 14.0,
    },
    Cutting {
        span: span(1110.0, 1185.0),
        side: Side::Left,
        height: 10.0,
    },
    // Roca a la izquierda antes del puente y a la derecha después.
    Cutting {
        span: span(2100.0, 2170.0),
        side: Side::Left,
        height: 14.0,
    },
    Cutting {
        span: span(2290.0, 2410.0),
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
