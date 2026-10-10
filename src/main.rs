//! Motorust: carreras de motos arcade inspiradas en Moto Racer.

// En Windows, la versión optimizada se abre sin ventana de consola.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bike;
mod camera;
mod circuits;
mod gamepads;
mod geometry;
mod hud;
mod lobby;
pub mod locale;
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

fn init_crash_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!(
            "Motorust se ha cerrado por un error:\n\n{info}\n\nSe ha guardado un informe en motorust-crash.log."
        );
        let _ = std::fs::write("motorust-crash.log", &msg);

        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let path = std::path::PathBuf::from(appdata)
                    .join("motorust")
                    .join("crash.log");
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, &msg);
            }

            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;

            let title: Vec<u16> = OsStr::new("Motorust - Error")
                .encode_wide()
                .chain(Some(0))
                .collect();
            let text: Vec<u16> = OsStr::new(&msg).encode_wide().chain(Some(0)).collect();

            unsafe {
                type MessageBoxW = unsafe extern "system" fn(
                    hwnd: *mut std::ffi::c_void,
                    lp_text: *const u16,
                    lp_caption: *const u16,
                    u_type: u32,
                ) -> i32;

                unsafe extern "system" {
                    fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
                    fn GetProcAddress(
                        hModule: *mut std::ffi::c_void,
                        lpProcName: *const u8,
                    ) -> *mut std::ffi::c_void;
                }

                let user32 = LoadLibraryA(b"user32.dll\0".as_ptr());
                if !user32.is_null() {
                    let proc = GetProcAddress(user32, b"MessageBoxW\0".as_ptr());
                    if !proc.is_null() {
                        let msg_box: MessageBoxW = std::mem::transmute(proc);
                        msg_box(
                            std::ptr::null_mut(),
                            text.as_ptr(),
                            title.as_ptr(),
                            0x10, /* MB_ICONERROR */
                        );
                    }
                }
            }
        }

        default_hook(info);
    }));
}

fn main() {
    init_crash_handler();

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
        lobby::LobbyPlugin,
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
