//! Motorust: carreras de motos arcade inspiradas en Moto Racer.

// En Windows, la versión optimizada se abre sin ventana de consola.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bike;
mod camera;
mod circuits;
mod gamepads;
mod geometry;
mod hud;
mod menu;
mod minimap;
mod pause;
mod settings;
mod shapes;
mod sound;
mod track;
mod track_select;
mod ui;
#[cfg(target_arch = "wasm32")]
mod web;
mod welcome;
mod world;

use bevy::prelude::*;

/// Pasos por segundo de la simulación. La física avanza siempre a este ritmo
/// y el render interpola entre pasos, así la moto se conduce igual a 60 que a
/// 240 FPS y la simulación es reproducible.
const SIMULATION_HZ: f64 = 120.0;

fn main() {
    // Los ajustes guardados deciden cómo se abre la ventana.
    let settings = settings::Settings::load();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(settings.window()),
                ..default()
            })
            // Los mandos los lee `gamepads`, que además los vuelve a buscar.
            .disable::<bevy::gilrs::GilrsPlugin>(),
    )
    .insert_resource(Time::<Fixed>::from_hz(SIMULATION_HZ))
    .insert_resource(settings)
    .add_plugins((
        world::WorldPlugin,
        bike::BikePlugin,
        camera::ChaseCameraPlugin,
        gamepads::GamepadsPlugin,
        hud::HudPlugin,
        menu::MenuPlugin,
        minimap::MinimapPlugin,
        welcome::WelcomePlugin,
        track_select::TrackSelectPlugin,
        pause::PausePlugin,
        settings::SettingsPlugin,
        sound::SoundPlugin,
        ui::UiPlugin,
    ));
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(web::WebPlugin);
    app.run();
}
