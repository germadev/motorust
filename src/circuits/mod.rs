//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
pub mod data;
mod fellowship_road;
mod ghost_island;
mod gold_mine;
mod loop_island;
mod moon_base;
mod toy_store;
mod turbo_bay;

use std::sync::LazyLock;

use bevy::prelude::*;
use serde::Deserialize;

use crate::locale::Language;
use crate::track::Layout;
use crate::world::scenery::Scenery;

/// Una pista para elegir.
///
/// Written in Rust, or read from a RON file (see [`data`]).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circuit {
    #[serde(deserialize_with = "data::text")]
    pub name: &'static str,
    #[serde(deserialize_with = "data::text")]
    pub name_en: &'static str,
    #[serde(deserialize_with = "data::text")]
    pub description: &'static str,
    #[serde(deserialize_with = "data::text")]
    pub description_en: &'static str,
    /// The music that plays during the race.
    #[serde(deserialize_with = "data::song")]
    pub song: Song,
    /// What its surroundings sound like, choosing the bike.
    pub ambience: Ambience,
    /// How fast it is ridden against most tracks (1): against the clock,
    /// the checkpoints of a slower one, narrow and twisting, give their
    /// seconds at a pace this much slower (see
    /// [`crate::race::CheckpointTimes::at`]).
    pub pace: f32,
    pub layout: Layout,
    pub scenery: Scenery,
}

/// The sounds of a track's surroundings, heard while choosing the bike
/// (see [`crate::sound::ambience`]): how much of each, from 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
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

/// The circuits written in RON, by the names of their files in
/// `assets/circuits/`, built into the game.
const FILES: [(&str, &str); 2] = [
    ("bahia.ron", include_str!("../../assets/circuits/bahia.ron")),
    (
        "turbo_bay.ron",
        include_str!("../../assets/circuits/turbo_bay.ron"),
    ),
];

/// The terrain's functions the circuits' files name (see [`data`]).
fn terrain_function(name: &str) -> Option<fn(Vec2) -> f32> {
    Some(match name {
        "bay_coast" => bay::coast,
        "bay_mountains" => bay::mountains,
        "turbo_bay_mountains" => turbo_bay::mountains,
        _ => return None,
    })
}

/// The songs the circuits' files name, by their files in `assets/music/`.
fn music(file: &str) -> Option<&'static [u8]> {
    Some(match file {
        "bahia.mp3" => include_bytes!("../../assets/music/bahia.mp3"),
        "turbo_bay.mp3" => include_bytes!("../../assets/music/turbo_bay.mp3"),
        _ => return None,
    })
}

/// Las pistas, en el orden de la pantalla de elegir pista: those written in
/// RON read from their files the first time they are wanted.
pub static CIRCUITS: LazyLock<[Circuit; 8]> = LazyLock::new(|| {
    let [(bay, bay_file), (turbo_bay, turbo_bay_file)] = FILES;
    [
        data::load(bay, bay_file),
        data::load(turbo_bay, turbo_bay_file),
        loop_island::LOOP_ISLAND,
        gold_mine::GOLD_MINE,
        ghost_island::GHOST_ISLAND,
        moon_base::MOON_BASE,
        fellowship_road::FELLOWSHIP_ROAD,
        toy_store::TOY_STORE,
    ]
});

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
        assert_eq!(CIRCUITS[1].checkpoints(), vec![960.0, 1980.0, 2980.0]);
        assert_eq!(CIRCUITS[2].checkpoints(), vec![700.0, 1560.0]);
        assert_eq!(
            CIRCUITS[3].checkpoints(),
            vec![1200.0, 2640.0, 3540.0, 4440.0, 5250.0]
        );
        assert_eq!(
            CIRCUITS[4].checkpoints(),
            vec![1000.0, 2200.0, 3150.0, 4230.0, 5100.0]
        );
        assert_eq!(CIRCUITS[5].checkpoints(), vec![1090.0, 2240.0, 3450.0]);
        assert_eq!(
            CIRCUITS[6].checkpoints(),
            vec![868.0, 1905.1, 2955.1, 3585.1, 4565.1]
        );
        assert_eq!(CIRCUITS[7].checkpoints(), vec![635.0, 1381.0, 2303.0]);
    }
}
