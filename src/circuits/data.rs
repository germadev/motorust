//! Circuits written as data, in RON (`assets/circuits/`): how a file is read
//! into the same [`Circuit`] the circuits written in Rust were. The bikes'
//! roster (`assets/bikes/roster.ron`, see [`crate::bike::Roster`]) is read
//! the same way.
//!
//! What a file gives as a list is read into a `Vec` and leaked, as is each
//! text, so that a circuit read from a file is as `&'static` as one written
//! in Rust and the code using it stays as it is. Each file is read once, at
//! startup ([`super::Circuits::load`]).
//!
//! How a file writes what is not plain numbers, lists and structs:
//!
//! - a colour as its sRGB, `(0.85, 0.08, 0.06)`;
//! - a stretch of the lap as `Span(40.0, 340.0)`, or `Lap` for all of it;
//! - a point as `(x, z)`;
//! - the mountains and the water as shapes with their parameters (see
//!   [`crate::world::mountains`] and [`crate::world::waters`]);
//! - what the game adds of its own to them by the name of a function in
//!   Rust, in [`super::terrain_function`] (a volcano's gullies);
//! - a song by the name of its MP3 in `assets/music/` (`song: (file:
//!   "bahia.mp3", ...)`), loaded from the content pack (see
//!   [`crate::content`]) when it plays.
//!
//! What a file leaves out of the scenery is not there (no tunnels, no
//! bridges, ...). A file giving a field the game does not know fails.

use bevy::prelude::*;
use serde::de::{Deserialize, Deserializer, Error};

#[cfg(any(doc, test))]
use super::Circuit;
use super::Song;
use crate::world::scenery::{Coast, Span, span};
use crate::world::waters::{Islet, Lake, River, Shore, Waters};

/// What a file, `name` (for the messages), holding `text` says: a circuit,
/// say (`'static`, as what it has of texts is, see [`Circuit::name`]).
pub fn parse<T: Deserialize<'static>>(name: &str, text: &'static str) -> Result<T, String> {
    ron::from_str(text).map_err(|error| format!("{name}: {error}"))
}

/// A colour's sRGB, as Bevy's `Srgba` (the bikes' colours).
pub fn srgba<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Srgba, D::Error> {
    let (red, green, blue) = Rgb::deserialize(deserializer)?;
    Ok(Srgba::rgb(red, green, blue))
}

/// A list, leaked.
pub fn leak<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<&'static [T], D::Error> {
    Ok(Vec::leak(Vec::deserialize(deserializer)?))
}

/// Lists of points, each leaked.
pub fn paths<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<&'static [&'static [Vec2]], D::Error> {
    let paths: Vec<&'static [Vec2]> = Vec::<Vec<Vec2>>::deserialize(deserializer)?
        .into_iter()
        .map(Vec::leak)
        .map(|path| &*path)
        .collect();
    Ok(Vec::leak(paths))
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

/// One of the terrain's functions: how high, or how far from the water, a
/// point is.
pub type TerrainFunction = fn(Vec2) -> f32;

/// One of the terrain's functions, by name, if there is one.
pub fn optional_function<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<TerrainFunction>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|name| named(&name).map_err(D::Error::custom))
        .transpose()
}

fn named(name: &str) -> Result<fn(Vec2) -> f32, String> {
    super::terrain_function(name).ok_or_else(|| format!("no terrain function called `{name}`"))
}

/// A song, its MP3 by the name of its file in `assets/music/`.
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
    if file.contains(['/', '\\']) || !file.ends_with(".mp3") {
        return Err(D::Error::custom(format!(
            "`{file}` is not the name of an MP3 in assets/music/"
        )));
    }
    Ok(Song {
        file: String::leak(format!("music/{file}")),
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

/// How a file writes a [`Coast`]: its waters' shapes as fields of its own
/// (`Sea(shores: [...])`, `Inland(lakes: [...], rivers: [...])`), each list
/// empty if left out.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CoastFile {
    Sea {
        #[serde(default)]
        shores: Vec<Shore>,
        #[serde(default)]
        lakes: Vec<Lake>,
        #[serde(default)]
        rivers: Vec<River>,
        #[serde(default)]
        islets: Vec<Islet>,
    },
    AroundTrack(Vec<(f32, f32)>),
    Outline(Vec<Vec2>),
    OutlineWithLakes {
        outline: Vec<Vec2>,
        #[serde(default)]
        lakes: Vec<Lake>,
        #[serde(default)]
        rivers: Vec<River>,
        #[serde(default)]
        islets: Vec<Islet>,
    },
    Inland {
        #[serde(default)]
        lakes: Vec<Lake>,
        #[serde(default)]
        rivers: Vec<River>,
        #[serde(default)]
        islets: Vec<Islet>,
    },
}

impl From<CoastFile> for Coast {
    fn from(file: CoastFile) -> Self {
        let waters =
            |shores: Vec<Shore>, lakes: Vec<Lake>, rivers: Vec<River>, islets: Vec<Islet>| Waters {
                shores: Vec::leak(shores),
                lakes: Vec::leak(lakes),
                rivers: Vec::leak(rivers),
                islets: Vec::leak(islets),
            };
        match file {
            CoastFile::Sea {
                shores,
                lakes,
                rivers,
                islets,
            } => Coast::Sea(waters(shores, lakes, rivers, islets)),
            CoastFile::AroundTrack(shore) => Coast::AroundTrack(Vec::leak(shore)),
            CoastFile::Outline(shore) => Coast::Outline(Vec::leak(shore)),
            CoastFile::OutlineWithLakes {
                outline,
                lakes,
                rivers,
                islets,
            } => Coast::OutlineWithLakes(
                Vec::leak(outline),
                waters(Vec::new(), lakes, rivers, islets),
            ),
            CoastFile::Inland {
                lakes,
                rivers,
                islets,
            } => Coast::Inland(waters(Vec::new(), lakes, rivers, islets)),
        }
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
        for (name, text) in super::super::files().unwrap() {
            parse::<Circuit>(&name, text).unwrap_or_else(|error| panic!("{error}"));
        }
        assert_eq!(CIRCUITS[0].name, "Bahía GP");
    }

    #[test]
    fn mistakes_are_told() {
        let files = super::super::files().unwrap();
        let (name, text) = (&*files[0].0, files[0].1);
        let changed = |from, to| &*String::leak(text.replacen(from, to, 1));
        let unknown = changed("pace:", "speed:");
        assert!(
            parse::<Circuit>(name, unknown)
                .err()
                .unwrap()
                .contains("speed")
        );
        // Nor a function the game does not have (Ghost Island's file names
        // its volcano's gullies).
        let (volcano, text) = (&*files[4].0, files[4].1);
        let function =
            &*String::leak(text.replacen("\"ghost_island_volcano_relief\"", "\"nowhere\"", 1));
        assert!(
            parse::<Circuit>(volcano, function)
                .err()
                .unwrap()
                .contains("nowhere")
        );
        // Nor a song that is not an MP3 in assets/music/.
        let song = changed("\"bahia.mp3\"", "\"../bahia.mp3\"");
        assert!(
            parse::<Circuit>(name, song)
                .err()
                .unwrap()
                .contains("../bahia.mp3")
        );
        // Nor a field the scenery does not have.
        let scenery = changed("scenery: (", "scenery: (lakes: [], ");
        assert!(
            parse::<Circuit>(name, scenery)
                .err()
                .unwrap()
                .contains("lakes")
        );
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
