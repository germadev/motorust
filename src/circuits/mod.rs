//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
mod loops_hill;
mod turbo_bay;

use bevy::prelude::*;

use crate::locale::Language;
use crate::track::Layout;
use crate::world::scenery::Scenery;

/// Una pista para elegir.
pub struct Circuit {
    pub name: &'static str,
    pub name_en: &'static str,
    pub description: &'static str,
    pub description_en: &'static str,
    pub layout: Layout,
    pub scenery: Scenery,
}

impl Circuit {
    pub fn name(&self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => self.name,
            Language::English => self.name_en,
        }
    }

    pub fn description(&self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => self.description,
            Language::English => self.description_en,
        }
    }

    /// Puntos a lo largo de la vuelta (m desde la meta) donde hay un checkpoint.
    pub fn checkpoints(&self) -> Vec<f32> {
        self.scenery
            .gantries
            .iter()
            .filter(|g| g.text == Some("CHECKPOINT"))
            .map(|g| g.along)
            .collect()
    }
}

/// Las pistas, en el orden de la pantalla de elegir pista.
pub static CIRCUITS: [Circuit; 3] = [bay::BAY, turbo_bay::TURBO_BAY, loops_hill::LOOPS_HILL];

/// La pista elegida: su índice en [`CIRCUITS`]. Al cambiarla, el mundo se
/// vuelve a construir ([`crate::world`]).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CurrentCircuit(pub usize);

impl CurrentCircuit {
    pub fn get(self) -> &'static Circuit {
        &CIRCUITS[self.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuits_have_checkpoints() {
        assert_eq!(CIRCUITS[0].checkpoints(), vec![780.0]);
        assert_eq!(CIRCUITS[1].checkpoints(), vec![960.0, 1600.0]);
        assert_eq!(CIRCUITS[2].checkpoints(), vec![700.0, 1420.0]);
    }
}
