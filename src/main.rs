//! Motorust: carreras de motos arcade inspiradas en Moto Racer. The game:
//! its content, its themes and its own plugins (`src/game/`), over the
//! engine (`motorust-engine/`, the `motorust_engine` crate).

// En Windows, la versión optimizada se abre sin ventana de consola.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;

/// The game's version and the commit it is built from (see `build.rs`), for
/// its logs: "0.0.10 (1a2b3c4d5)", "-dirty" after it with changes not
/// committed.
pub const BUILD: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("MOTORUST_COMMIT"),
    ")"
);

fn main() {
    // The game's content pack, themes and terrain functions, before any
    // file is read.
    game::register();
    // The crash report, with the game's name, folder and build.
    motorust_engine::crash::install();
    // On macOS, no App Nap, timers on time and no sleep while it runs.
    #[cfg(target_os = "macos")]
    motorust_engine::latency::begin();
    motorust_engine::set_build(BUILD);

    let mut app = motorust_engine::app();
    app.add_plugins(game::GamePlugin);
    app.run();
}
