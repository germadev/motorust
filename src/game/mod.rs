//! The game: what Motorust adds of its own to the engine, its themed
//! scenery as themes (see [`motorust_engine::world::themes`]). Nothing outside this
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

use motorust_engine::world::themes;

/// The game's circuits, for the tests: read once, as the game reads them,
/// its themes registered first.
#[cfg(test)]
pub static CIRCUITS: std::sync::LazyLock<motorust_engine::circuits::Circuits> =
    std::sync::LazyLock::new(|| {
        register();
        motorust_engine::circuits::Circuits::load()
    });

/// The game's bikes, for the tests: read once, as the game reads them.
#[cfg(test)]
pub static ROSTER: std::sync::LazyLock<motorust_engine::bike::Roster> =
    std::sync::LazyLock::new(motorust_engine::bike::Roster::load);

/// The game's content pack (`assets/`), over the engine's (see
/// [`motorust_engine::content`]).
pub const PACK: motorust_engine::content::Pack = motorust_engine::content::Pack {
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
    motorust_engine::content::register_pack(PACK);
    motorust_engine::circuits::register_terrain_function(
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
