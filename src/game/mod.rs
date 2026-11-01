//! The game: what Motorust adds of its own to the engine, its themed
//! scenery as themes (see [`crate::world::themes`]). Nothing outside this
//! folder (and the game's `main`) names it, so that it can go to a crate of
//! its own, the engine's apart (step 4 of the engine's split,
//! `ROADMAP.md`); a test keeps it so.

pub mod bricks;
#[cfg(test)]
mod checks;
pub mod circuits;
pub mod desert;
pub mod japan;
pub mod middle_earth;
pub mod mine;
pub mod shopfront;
pub mod space;
pub mod toys;

use bevy::prelude::*;

use crate::world::themes;

/// The game's circuits, for the tests: read once, as the game reads them,
/// its themes registered first.
#[cfg(test)]
pub static CIRCUITS: std::sync::LazyLock<crate::circuits::Circuits> =
    std::sync::LazyLock::new(|| {
        register();
        crate::circuits::Circuits::load()
    });

/// The game's bikes, for the tests: read once, as the game reads them.
#[cfg(test)]
pub static ROSTER: std::sync::LazyLock<crate::bike::Roster> =
    std::sync::LazyLock::new(crate::bike::Roster::load);

/// The game's content pack (`assets/`), over the engine's (see
/// [`crate::content`]).
pub const PACK: crate::content::Pack = crate::content::Pack {
    folder: concat!(env!("CARGO_MANIFEST_DIR"), "/assets"),
    built_in: built_in::FILES,
};

/// The game's pack's files, built in, as `build.rs` lists them from its
/// `pack.txt`.
mod built_in {
    #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
    include!(concat!(env!("OUT_DIR"), "/pack.rs"));
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    pub const FILES: &[(&str, &[u8])] = &[];
}

/// Registers what the game has of its own: its content pack, before any
/// file is read, and what its circuits' files name, its themes and its
/// terrain's functions, before the circuits are read (the `WorldPlugin`
/// reads them as it is added). Again does nothing.
pub fn register() {
    crate::content::register_pack(PACK);
    crate::circuits::register_terrain_function(
        "ghost_island_volcano_relief",
        circuits::ghost_island::volcano_gullies,
    );
    themes::register::<desert::Desert>("desert");
    themes::register::<japan::Japan>("japan");
    themes::register::<middle_earth::MiddleEarth>("middle_earth");
    themes::register::<mine::Mine>("mine");
    themes::register::<space::Space>("space");
    themes::register::<toys::ToyStore>("toys");
}

/// The game's own systems: its themes' (what moves in them).
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        register();
        app.add_plugins((
            desert::DesertPlugin,
            mine::MinePlugin,
            space::SpacePlugin,
            toys::ToysPlugin,
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// The files outside the game's folder that name it: none but the
    /// game's `main` (the engine's tests run on the engine's own content,
    /// [`crate::testing`]; the game runs the engine's checks on its own,
    /// [`super::checks`]).
    const ALLOWED: [&str; 1] = ["src/main.rs"];

    fn sources(folder: &Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }

    #[test]
    fn nothing_outside_the_game_names_it() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut found = Vec::new();
        sources(&root.join("src"), &mut found);
        let naming: Vec<String> = found
            .iter()
            .filter(|path| !path.starts_with(root.join("src/game")))
            .filter(|path| {
                let text = std::fs::read_to_string(path).unwrap();
                text.contains("crate::game::") || text.contains("crate::game;")
            })
            .map(|path| path.strip_prefix(root).unwrap().display().to_string())
            .filter(|path| !ALLOWED.contains(&path.as_str()))
            .collect();
        assert!(naming.is_empty(), "they name the game: {naming:?}");
    }
}
