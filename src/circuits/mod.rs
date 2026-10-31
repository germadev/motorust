//! Las pistas del juego: el trazado y el escenario de cada una, y cuál se
//! está usando.

mod bay;
pub mod data;
mod fellowship_road;
mod ghost_island;
mod gold_mine;
#[cfg(test)]
mod loop_island;
#[cfg(test)]
mod moon_base;

#[cfg(test)]
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

/// The circuits, by the names of their files in `assets/circuits/`, built
/// into the game, in the order of the track select screen.
const FILES: [(&str, &str); 8] = [
    ("bahia.ron", include_str!("../../assets/circuits/bahia.ron")),
    (
        "turbo_bay.ron",
        include_str!("../../assets/circuits/turbo_bay.ron"),
    ),
    (
        "loop_island.ron",
        include_str!("../../assets/circuits/loop_island.ron"),
    ),
    (
        "gold_mine.ron",
        include_str!("../../assets/circuits/gold_mine.ron"),
    ),
    (
        "ghost_island.ron",
        include_str!("../../assets/circuits/ghost_island.ron"),
    ),
    (
        "moon_base.ron",
        include_str!("../../assets/circuits/moon_base.ron"),
    ),
    (
        "fellowship_road.ron",
        include_str!("../../assets/circuits/fellowship_road.ron"),
    ),
    (
        "toy_store.ron",
        include_str!("../../assets/circuits/toy_store.ron"),
    ),
];

/// The terrain's functions the circuits' files name (see [`data`]).
fn terrain_function(name: &str) -> Option<fn(Vec2) -> f32> {
    Some(match name {
        // No water anywhere, and no mountains: for a circuit indoors.
        "dry" => |_| 1000.0,
        "flat" => |_| 0.0,
        "bay_coast" => bay::coast,
        "gold_mine_water" => gold_mine::water,
        "ghost_island_lakes" => ghost_island::lakes,
        "ghost_island_volcano_relief" => ghost_island::volcano_gullies,
        "fellowship_road_water" => fellowship_road::water,
        _ => return None,
    })
}

/// The songs the circuits' files name, by their files in `assets/music/`.
fn music(file: &str) -> Option<&'static [u8]> {
    Some(match file {
        "bahia.mp3" => include_bytes!("../../assets/music/bahia.mp3"),
        "turbo_bay.mp3" => include_bytes!("../../assets/music/turbo_bay.mp3"),
        "loop_island.mp3" => include_bytes!("../../assets/music/loop_island.mp3"),
        "moon_base.mp3" => include_bytes!("../../assets/music/moon_base.mp3"),
        "gold_mine.mp3" => include_bytes!("../../assets/music/gold_mine.mp3"),
        "autumn_gale.mp3" => include_bytes!("../../assets/music/autumn_gale.mp3"),
        "beyond_the_final_turn.mp3" => {
            include_bytes!("../../assets/music/beyond_the_final_turn.mp3")
        }
        "plastic_velocity_rush.mp3" => {
            include_bytes!("../../assets/music/plastic_velocity_rush.mp3")
        }
        _ => return None,
    })
}

/// Las pistas, en el orden de la pantalla de elegir pista: read from their
/// files once, when the game starts ([`Circuits::load`]), and kept for as
/// long as it runs.
#[derive(Resource, Clone, Copy)]
pub struct Circuits(&'static [Circuit]);

impl Circuits {
    /// Every circuit, read from its file (see [`data::load`]).
    pub fn load() -> Self {
        let circuits: Vec<Circuit> = FILES
            .iter()
            .map(|&(name, text)| data::load("circuits", name, text))
            .collect();
        Circuits(Vec::leak(circuits))
    }

    /// The one chosen.
    pub fn get(&self, current: CurrentCircuit) -> &'static Circuit {
        self.at(current.0)
    }

    /// The one at `index`, for as long as the game runs.
    pub fn at(&self, index: usize) -> &'static Circuit {
        &self.0[index]
    }
}

impl Default for Circuits {
    fn default() -> Self {
        Self::load()
    }
}

impl std::ops::Deref for Circuits {
    type Target = [Circuit];

    fn deref(&self) -> &[Circuit] {
        self.0
    }
}

/// The circuits, for the tests: read once, as the game reads them.
#[cfg(test)]
pub static CIRCUITS: LazyLock<Circuits> = LazyLock::new(Circuits::load);

/// La pista elegida: su índice en [`Circuits`]. Al cambiarla, el mundo se
/// vuelve a construir ([`crate::world`]).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CurrentCircuit(pub usize);

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
