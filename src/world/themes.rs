//! Themes: scenery a game adds of its own to the engine's, as plugins. (Step
//! 4 of the engine's split, `ROADMAP.md`: the engine knows no theme; the
//! game registers its own.)
//!
//! A theme is a type with what a circuit's file says of it, registered by a
//! name ([`register`]); a circuit's file writes its themes in a map by
//! those names, each read by the theme registered under it:
//!
//! ```ron
//! scenery: (
//!     ...
//!     themes: {
//!         "desert": (gas_stations: [...], ...),
//!     },
//! ),
//! ```
//!
//! A name no theme was registered under is refused, as a field the game
//! does not know is. As the world is built, each theme of the circuit
//! puts its own in it ([`Theme::spawn`]); what moves in it afterwards (a
//! pumpjack nodding, a tumbleweed rolling) the theme's own systems move,
//! added by the game's plugin.

use std::any::{Any, TypeId};
use std::fmt;
use std::sync::RwLock;

use bevy::prelude::*;
use serde::de::{DeserializeOwned, DeserializeSeed, Deserializer, Error, MapAccess, Visitor};

use super::Assets3d;
use super::props::Footprint;
use super::terrain::Terrain;
use crate::track::Track;

/// Scenery a game adds of its own: what a circuit's file says of it, and
/// what it puts in the world from that.
pub trait Theme: Any + Send + Sync {
    /// Puts it in the circuit's world as it is built, beside the road, and
    /// returns the ground each thing takes, so that nothing else (houses,
    /// trees) is put there.
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        let _ = (world, track, terrain, assets);
        Vec::new()
    }
}

/// How a registered theme is read from a file.
type Reader = fn(&mut dyn erased_serde::Deserializer<'_>) -> Result<Box<dyn Theme>, String>;

/// The themes registered, by their names, in the order they were: each its
/// type and how it is read.
static REGISTERED: RwLock<Vec<(&'static str, TypeId, Reader)>> = RwLock::new(Vec::new());

/// Registers the theme `T` under `name`: a circuit's file then writes it in
/// its `themes` by that name. Registering it again does nothing; another
/// theme under a name taken stops the game, saying so.
pub fn register<T: Theme + DeserializeOwned>(name: &'static str) {
    fn read<T: Theme + DeserializeOwned>(
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Box<dyn Theme>, String> {
        erased_serde::deserialize::<T>(deserializer)
            .map(|theme| Box::new(theme) as Box<dyn Theme>)
            .map_err(|error| error.to_string())
    }
    let mut registered = REGISTERED
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match registered.iter().find(|(taken, ..)| *taken == name) {
        Some((_, other, _)) if *other == TypeId::of::<T>() => {}
        Some(_) => panic!("two themes registered as `{name}`"),
        None => registered.push((name, TypeId::of::<T>(), read::<T>)),
    }
}

/// How the theme registered under `name` is read, if one is.
fn reader(name: &str) -> Option<Reader> {
    REGISTERED
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .find(|(registered, ..)| *registered == name)
        .map(|&(.., reader)| reader)
}

/// The names of the themes registered, for the messages.
fn names() -> Vec<&'static str> {
    REGISTERED
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .map(|&(name, ..)| name)
        .collect()
}

/// A circuit's themes, in the order its file writes them, each by its
/// name.
#[derive(Default)]
pub struct Themes(&'static [(&'static str, Box<dyn Theme>)]);

impl Themes {
    /// Each theme, with its name.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static dyn Theme)> {
        self.0.iter().map(|(name, theme)| (*name, &**theme))
    }

    /// The theme of type `T`, if the circuit has it.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "for the themes to come, only tested yet")
    )]
    pub fn get<T: Theme>(&self) -> Option<&'static T> {
        self.0
            .iter()
            .find_map(|(_, theme)| (&**theme as &dyn Any).downcast_ref::<T>())
    }
}

impl<'de> serde::Deserialize<'de> for Themes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ThemesVisitor)
    }
}

struct ThemesVisitor;

impl<'de> Visitor<'de> for ThemesVisitor {
    type Value = Themes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map of themes by their names")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Themes, A::Error> {
        let mut themes: Vec<(&'static str, Box<dyn Theme>)> = Vec::new();
        while let Some(name) = map.next_key::<String>()? {
            let Some(reader) = reader(&name) else {
                return Err(A::Error::custom(format!(
                    "no theme called `{name}` (the themes: {:?})",
                    names()
                )));
            };
            if themes.iter().any(|(taken, _)| *taken == name) {
                return Err(A::Error::custom(format!("the theme `{name}` twice")));
            }
            let theme = map.next_value_seed(Seed(reader))?;
            themes.push((String::leak(name), theme));
        }
        Ok(Themes(Vec::leak(themes)))
    }
}

/// Reads a theme with the reader registered for its name.
struct Seed(Reader);

impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Box<dyn Theme>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.0)(&mut erased).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize, PartialEq, Debug)]
    #[serde(deny_unknown_fields)]
    struct Statues {
        count: usize,
        side: super::super::scenery::Side,
    }

    impl Theme for Statues {}

    #[derive(serde::Deserialize)]
    struct Fountains {}

    impl Theme for Fountains {}

    #[test]
    fn a_file_writes_its_themes_by_the_names_they_were_registered_under() {
        register::<Statues>("statues");
        register::<Statues>("statues");
        register::<Fountains>("fountains");
        let themes: Themes =
            ron::from_str(r#"{ "statues": (count: 3, side: Left), "fountains": () }"#).unwrap();
        assert_eq!(
            themes.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            ["statues", "fountains"]
        );
        assert_eq!(
            themes.get::<Statues>(),
            Some(&Statues {
                count: 3,
                side: super::super::scenery::Side::Left
            })
        );
        assert!(Themes::default().get::<Statues>().is_none());
        // What it gets wrong is told: an unknown theme, one twice, and a
        // field its theme does not have.
        let wrong = |text| ron::from_str::<Themes>(text).err().unwrap().to_string();
        assert!(wrong(r#"{ "nowhere": () }"#).contains("nowhere"));
        assert!(wrong(r#"{ "fountains": (), "fountains": () }"#).contains("twice"));
        assert!(wrong(r#"{ "statues": (count: 3, side: Left, height: 2.0) }"#).contains("height"));
    }

    #[test]
    #[should_panic(expected = "two themes")]
    fn two_themes_cannot_take_one_name() {
        register::<Statues>("taken");
        register::<Fountains>("taken");
    }
}
