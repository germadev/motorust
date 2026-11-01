//! The game: what Motorust adds of its own to the engine, its themed
//! scenery as themes (see [`crate::world::themes`]). Nothing outside this
//! folder (and the game's `main`) names it, so that it can go to a crate of
//! its own, the engine's apart (step 4 of the engine's split,
//! `ROADMAP.md`); a test keeps it so.

pub mod desert;
pub mod mine;

use bevy::prelude::*;

use crate::world::themes;

/// Registers the game's themes, so that the circuits' files can name them.
/// Before the circuits are read (the `WorldPlugin` reads them as it is
/// added); again does nothing.
pub fn register_themes() {
    themes::register::<desert::Desert>("desert");
    themes::register::<mine::Mine>("mine");
}

/// The game's own systems: its themes' (what moves in them).
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        register_themes();
        app.add_plugins((desert::DesertPlugin, mine::MinePlugin));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// The files outside the game's folder that name it, and why: none but
    /// the game's `main`, and the circuits' tests, which read the game's
    /// circuits until the engine's tests have content of their own (4c).
    const ALLOWED: [&str; 2] = ["src/main.rs", "src/circuits/mod.rs"];

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
