//! Circuits written as data, in RON (`assets/circuits/`): how a file is read
//! into the same [`Circuit`] the circuits written in Rust are.
//!
//! What a file gives as a list is read into a `Vec` and leaked, as is each
//! text, so that a circuit read from a file is as `&'static` as one written
//! in Rust and the code using it stays as it is. Each file is read once, at
//! startup ([`super::CIRCUITS`]).
//!
//! How a file writes what is not plain numbers, lists and structs:
//!
//! - a colour as its sRGB, `(0.85, 0.08, 0.06)`;
//! - a stretch of the lap as `Span(40.0, 340.0)`, or `Lap` for all of it;
//! - a point as `(x, z)`;
//! - the terrain's functions (the coast, the mountains) by their names in
//!   [`super::terrain_function`], as `Function("bay_coast")`: they stay in
//!   Rust, as the music does (`song: (file: "bahia.mp3", ...)`, built in by
//!   [`super::music`]).
//!
//! What a file leaves out of the scenery is not there (no tunnels, no
//! bridges, ...). The scenery a file cannot give yet (the themed scenery of
//! Middle-earth and the toy store) is not read: a file giving it fails, as
//! does any other field the game does not know.

use bevy::prelude::*;
use serde::de::{Deserialize, Deserializer, Error};

use super::{Circuit, Song};
use crate::world::scenery::{Coast, Span, span};

/// A circuit from its file, `name` (for the messages) holding `text`
/// (`'static`, as what it has of texts is, see [`Circuit::name`]).
pub fn parse(name: &str, text: &'static str) -> Result<Circuit, String> {
    ron::from_str(text).map_err(|error| format!("{name}: {error}"))
}

/// A circuit built into the game from its file in `assets/circuits/`.
///
/// In a desktop development build the file is read from the disk if it is
/// there, so a circuit can be changed and tried again without compiling,
/// just restarting the game.
pub fn load(name: &str, built_in: &'static str) -> Circuit {
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    let on_disk = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/circuits")
            .join(name),
    )
    .ok()
    .map(|text| &*String::leak(text));
    #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
    let on_disk: Option<&'static str> = None;
    let text = on_disk.unwrap_or(built_in);
    parse(name, text).unwrap_or_else(|error| panic!("{error}"))
}

/// A list, leaked.
pub fn leak<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<&'static [T], D::Error> {
    Ok(Vec::leak(Vec::deserialize(deserializer)?))
}

/// A text, leaked.
pub fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<&'static str, D::Error> {
    Ok(String::leak(String::deserialize(deserializer)?))
}

pub fn optional_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<&'static str>, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?
        .map(String::leak)
        .map(|text| &*text))
}

/// A colour's sRGB.
type Rgb = (f32, f32, f32);

fn srgb((red, green, blue): Rgb) -> Color {
    Color::srgb(red, green, blue)
}

pub fn color<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
    Rgb::deserialize(deserializer).map(srgb)
}

pub fn optional_color<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Color>, D::Error> {
    Ok(Option::<Rgb>::deserialize(deserializer)?.map(srgb))
}

pub fn colors<'de, D: Deserializer<'de>>(deserializer: D) -> Result<&'static [Color], D::Error> {
    let colors: Vec<Color> = Vec::<Rgb>::deserialize(deserializer)?
        .into_iter()
        .map(srgb)
        .collect();
    Ok(Vec::leak(colors))
}

/// Two colours, as a list.
pub fn color_pair<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[Color; 2], D::Error> {
    match Vec::<Rgb>::deserialize(deserializer)?[..] {
        [first, second] => Ok([srgb(first), srgb(second)]),
        ref other => Err(D::Error::invalid_length(other.len(), &"two colours")),
    }
}

/// A billboard's accent: from which letter on, and in what colour.
pub fn accent<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<(usize, Color)>, D::Error> {
    Ok(Option::<(usize, Rgb)>::deserialize(deserializer)?.map(|(from, rgb)| (from, srgb(rgb))))
}

/// One of the terrain's functions, by name.
pub fn function<'de, D: Deserializer<'de>>(deserializer: D) -> Result<fn(Vec2) -> f32, D::Error> {
    named(&String::deserialize(deserializer)?).map_err(D::Error::custom)
}

fn named(name: &str) -> Result<fn(Vec2) -> f32, String> {
    super::terrain_function(name).ok_or_else(|| format!("no terrain function called `{name}`"))
}

/// A song, its MP3 by the name of its file.
pub fn song<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Song, D::Error> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct File {
        file: String,
        length: f32,
        loudness: f32,
    }
    let File {
        file,
        length,
        loudness,
    } = File::deserialize(deserializer)?;
    let mp3 = super::music(&file)
        .ok_or_else(|| D::Error::custom(format!("no song built in called `{file}`")))?;
    Ok(Song {
        mp3,
        length,
        loudness,
    })
}

/// How a file writes a [`Span`].
#[derive(serde::Deserialize)]
pub enum SpanFile {
    /// All the lap.
    Lap,
    /// From and to, in metres from the finish line.
    Span(f32, f32),
}

impl From<SpanFile> for Span {
    fn from(file: SpanFile) -> Self {
        match file {
            SpanFile::Lap => crate::world::scenery::LAP,
            SpanFile::Span(from, to) => span(from, to),
        }
    }
}

/// How a file writes a [`Coast`]: as it is, the functions by name.
#[derive(serde::Deserialize)]
pub enum CoastFile {
    Function(String),
    AroundTrack(Vec<(f32, f32)>),
    Outline(Vec<Vec2>),
    OutlineWithLakes(Vec<Vec2>, String),
    Inland(String),
}

impl TryFrom<CoastFile> for Coast {
    type Error = String;

    fn try_from(file: CoastFile) -> Result<Self, String> {
        Ok(match file {
            CoastFile::Function(name) => Coast::Function(named(&name)?),
            CoastFile::AroundTrack(shore) => Coast::AroundTrack(Vec::leak(shore)),
            CoastFile::Outline(shore) => Coast::Outline(Vec::leak(shore)),
            CoastFile::OutlineWithLakes(shore, lakes) => {
                Coast::OutlineWithLakes(Vec::leak(shore), named(&lakes)?)
            }
            CoastFile::Inland(name) => Coast::Inland(named(&name)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    /// Every circuit read from a file reads, so a mistake in one fails
    /// here and not in the game.
    #[test]
    fn every_circuit_file_reads() {
        for (name, text) in super::super::FILES {
            parse(name, text).unwrap_or_else(|error| panic!("{error}"));
        }
        assert_eq!(CIRCUITS[0].name, "Bahía GP");
    }

    #[test]
    fn mistakes_are_told() {
        let (name, text) = super::super::FILES[0];
        let changed = |from, to| &*String::leak(text.replacen(from, to, 1));
        let unknown = changed("pace:", "speed:");
        assert!(parse(name, unknown).err().unwrap().contains("speed"));
        let function = changed("\"bay_coast\"", "\"nowhere\"");
        assert!(parse(name, function).err().unwrap().contains("nowhere"));
        // What a file cannot give yet is not read.
        let toys = changed("scenery: (", "scenery: (toys: None, ");
        assert!(parse(name, toys).is_err());
    }

    #[test]
    fn spans_and_colours() {
        #[derive(serde::Deserialize)]
        struct Both {
            #[serde(deserialize_with = "color")]
            color: Color,
            spans: Vec<Span>,
        }
        let both: Both =
            ron::from_str("(color: (1.0, 0.5, 0.0), spans: [Lap, Span(2400.0, 300.0)])").unwrap();
        assert_eq!(both.color, Color::srgb(1.0, 0.5, 0.0));
        assert_eq!(
            both.spans,
            [crate::world::scenery::LAP, span(2400.0, 300.0)]
        );
    }
}
