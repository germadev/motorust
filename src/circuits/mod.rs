//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
mod gold_mine;
mod loop_island;
mod tsushima;
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
    /// What its surroundings sound like, choosing the bike.
    pub ambience: Ambience,
    pub layout: Layout,
    pub scenery: Scenery,
}

/// The sounds of a track's surroundings, heard while choosing the bike
/// (see [`crate::sound::ambience`]): how much of each, from 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ambience {
    /// Waves breaking on the shore.
    pub sea: f32,
    /// Seagulls calling now and then.
    pub gulls: f32,
    /// Small birds chirping in the trees.
    pub birds: f32,
    /// Cicadas buzzing in the heat.
    pub cicadas: f32,
    pub wind: f32,
}

/// A song: an MP3 built into the game ([`crate::sound`] loops it, or, in a
/// list, plays it through and goes on to the next).
pub struct Song {
    pub mp3: &'static [u8],
    /// How long it lasts (s), as `ffprobe` gives it.
    pub length: f32,
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
pub static CIRCUITS: [Circuit; 5] = [
    bay::BAY,
    turbo_bay::TURBO_BAY,
    loop_island::LOOP_ISLAND,
    gold_mine::GOLD_MINE,
    tsushima::TSUSHIMA,
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
        assert_eq!(
            CIRCUITS[3].checkpoints(),
            vec![1200.0, 2640.0, 3540.0, 4440.0, 5250.0]
        );
        assert_eq!(
            CIRCUITS[4].checkpoints(),
            vec![1000.0, 2200.0, 3150.0, 4150.0, 5100.0]
        );
    }
}
