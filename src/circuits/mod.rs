//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
mod gold_mine;
mod loop_island;
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
    /// The music that plays during the race.
    pub song: Song,
    pub layout: Layout,
    pub scenery: Scenery,
}

/// A track's music: an MP3 built into the game ([`crate::sound`] loops it).
pub struct Song {
    pub mp3: &'static [u8],
    /// How loud it sounds over the engines, as mastered: the integrated
    /// loudness (LUFS) of its first two minutes, as long as a race, above
    /// 500 Hz, as `ffmpeg -t 120 -i song.mp3 -af highpass=f=500,ebur128 -f
    /// null -` measures it. The game turns each song up or down by the
    /// difference, so they all sound equally loud.
    pub loudness: f32,
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
pub static CIRCUITS: [Circuit; 4] = [
    bay::BAY,
    turbo_bay::TURBO_BAY,
    loop_island::LOOP_ISLAND,
    gold_mine::GOLD_MINE,
];

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
        assert_eq!(CIRCUITS[1].checkpoints(), vec![960.0, 1980.0]);
        assert_eq!(CIRCUITS[2].checkpoints(), vec![700.0, 1420.0]);
    }
}
