//! La Bahía: la primera pista, inspirada en Speed Bay a la mediterránea. Una
//! carretera de costa con muros de piedra, farolas, paredes de roca cortada,
//! pinos piñoneros, cipreses, olivos y casas encaladas, con el mar al oeste
//! y al sur y montañas al noreste.
//!
//! The circuit is written in RON, `assets/circuits/bahia.ron`; here are
//! only the functions its file names: its coast and its mountains.

use bevy::prelude::*;

use crate::geometry::{fbm, smoothstep};

/// Distancia (aproximada, con signo) a la costa: positiva tierra adentro. El
/// mar queda al oeste y al sur del circuito.
pub(super) fn coast(point: Vec2) -> f32 {
    let west = point.x + 95.0 - 25.0 * (point.y / 97.0).sin() - 12.0 * (point.y / 41.0 + 1.0).sin();
    let south =
        205.0 + 20.0 * (point.x / 83.0).sin() + 10.0 * (point.x / 37.0 + 2.0).sin() - point.y;
    west.min(south)
}

/// Montañas tierra adentro, al noreste, que cierran el horizonte.
pub(super) fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(170.0, -180.0);
    let inland = (point - CENTER).dot(Vec2::new(1.0, -1.0).normalize());
    smoothstep(380.0, 1300.0, inland) * 260.0 * (0.4 + 0.6 * fbm(point / 300.0, 4))
}
