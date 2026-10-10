//! La Bahía: la primera pista, inspirada en Speed Bay a la mediterránea. Una
//! carretera de costa con muros de piedra, farolas, paredes de roca cortada,
//! pinos piñoneros, cipreses, olivos y casas encaladas, con el mar al oeste
//! y al sur y montañas al noreste.

use bevy::prelude::*;

use super::Circuit;
use crate::geometry::{fbm, smoothstep};
use crate::track::{Layout, WALL_DISTANCE};
use crate::world::scenery::*;

pub const BAY: Circuit = Circuit {
    name: "Bahía",
    description: "Una carretera junto al mar, con cuestas, peraltes\n\
                  y un salto en la recta de meta",
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[],
    },
    scenery: Scenery {
        coast: Coast::Function(coast),
        hills: 16.0,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        terraces: &[],
        mountains,
        curbs: [Color::srgb(0.85, 0.08, 0.06), Color::srgb(0.95, 0.95, 0.95)],
        walls: &[],
        tunnels: &[],
        bridges: &[],
        gantries: &[Gantry {
            along: 0.0,
            color: Color::srgb(0.8, 0.08, 0.06),
            text: None,
        }],
        billboards: &[],
        lamps: &[Row {
            span: LAP,
            side: Side::Left,
            lateral: WALL_DISTANCE + 0.9,
            every: 44.0,
        }],
        trees: &[Species::UmbrellaPine, Species::Cypress, Species::Olive],
        forests: &[],
        houses: Houses {
            max: 12,
            paints: &[
                Color::srgb(0.97, 0.96, 0.93),
                Color::srgb(0.96, 0.9, 0.74),
                Color::srgb(0.93, 0.78, 0.55),
                Color::srgb(0.95, 0.83, 0.8),
            ],
            spans: &[],
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
    },
};

const CONTROL_POINTS: [Vec2; 16] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -220.0),
    Vec2::new(10.0, -380.0),
    Vec2::new(70.0, -450.0),
    Vec2::new(170.0, -460.0),
    Vec2::new(240.0, -410.0),
    Vec2::new(255.0, -320.0),
    Vec2::new(215.0, -240.0),
    Vec2::new(250.0, -160.0),
    Vec2::new(320.0, -110.0),
    Vec2::new(345.0, -10.0),
    Vec2::new(305.0, 80.0),
    Vec2::new(210.0, 110.0),
    Vec2::new(125.0, 104.0),
    Vec2::new(45.0, 104.0),
    Vec2::new(5.0, 58.0),
];

/// Hay un cambio de rasante en la recta de meta, subida hasta lo alto de la
/// zona este y bajada hacia la meta.
const ELEVATION: [(f32, f32); 19] = [
    (0.0, 0.0),
    (90.0, 1.0),
    (170.0, 6.5),
    // Cambio de rasante: a toda velocidad, la moto vuela.
    (205.0, 8.5),
    (240.0, 5.0),
    (300.0, 2.0),
    (382.0, 2.5),
    (476.0, 4.0),
    (578.0, 6.0),
    (666.0, 8.0),
    (758.0, 13.0),
    (848.0, 16.0),
    (938.0, 18.0),
    (1024.0, 15.0),
    (1130.0, 9.0),
    (1230.0, 4.0),
    (1332.0, 2.5),
    (1416.0, 3.5),
    (1498.0, 1.0),
];

const MASSIFS: [Massif; 9] = [
    // A la derecha de la recta de meta: la pared de roca de Speed Bay.
    massif(62.0, -215.0, 52.0, 24.0),
    massif(150.0, -330.0, 40.0, 16.0),
    massif(130.0, -60.0, 55.0, 20.0),
    massif(420.0, -230.0, 75.0, 38.0),
    massif(430.0, 60.0, 60.0, 30.0),
    massif(140.0, -560.0, 85.0, 42.0),
    massif(-60.0, -520.0, 45.0, 22.0),
    // Islotes.
    massif(-170.0, -260.0, 35.0, 14.0),
    massif(40.0, 250.0, 30.0, 10.0),
];

const CUTTINGS: [Cutting; 4] = [
    // La recta de meta, con la roca a la derecha y el mar a la izquierda.
    Cutting {
        span: span(40.0, 340.0),
        side: Side::Right,
        height: 16.0,
    },
    Cutting {
        span: span(470.0, 640.0),
        side: Side::Left,
        height: 20.0,
    },
    // La subida del este, con el precipicio por fuera.
    Cutting {
        span: span(780.0, 1010.0),
        side: Side::Left,
        height: 22.0,
    },
    Cutting {
        span: span(1250.0, 1400.0),
        side: Side::Right,
        height: 11.0,
    },
];

/// Distancia (aproximada, con signo) a la costa: positiva tierra adentro. El
/// mar queda al oeste y al sur del circuito.
fn coast(point: Vec2) -> f32 {
    let west = point.x + 95.0 - 25.0 * (point.y / 97.0).sin() - 12.0 * (point.y / 41.0 + 1.0).sin();
    let south =
        205.0 + 20.0 * (point.x / 83.0).sin() + 10.0 * (point.x / 37.0 + 2.0).sin() - point.y;
    west.min(south)
}

/// Montañas tierra adentro, al noreste, que cierran el horizonte.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(170.0, -180.0);
    let inland = (point - CENTER).dot(Vec2::new(1.0, -1.0).normalize());
    smoothstep(380.0, 1300.0, inland) * 260.0 * (0.4 + 0.6 * fbm(point / 300.0, 4))
}
