//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
mod speed_bay;

use bevy::prelude::*;

use crate::track::Layout;
use crate::world::scenery::Scenery;

/// Una pista para elegir.
pub struct Circuit {
    pub name: &'static str,
    pub description: &'static str,
    pub layout: Layout,
    pub scenery: Scenery,
}

/// Las pistas, en el orden de la pantalla de elegir pista.
pub static CIRCUITS: [Circuit; 2] = [bay::BAY, speed_bay::SPEED_BAY];

/// La pista elegida: su índice en [`CIRCUITS`]. Al cambiarla, el mundo se
/// vuelve a construir ([`crate::world`]).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CurrentCircuit(pub usize);

impl CurrentCircuit {
    pub fn get(self) -> &'static Circuit {
        &CIRCUITS[self.0]
    }
}
