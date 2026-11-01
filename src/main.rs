//! Motorust: carreras de motos arcade inspiradas en Moto Racer.

// En Windows, la versión optimizada se abre sin ventana de consola.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
mod bench;
mod bezier;
mod bike;
mod camera;
mod circuits;
mod content;
mod curves_hud;
mod debug_start;
mod game;
mod game_info;
mod gamepads;
mod geometry;
mod hud;
#[cfg(target_os = "macos")]
mod latency;
mod legend;
mod lobby;
pub mod locale;
mod mapping;
mod menu;
mod minimap;
mod modes;
mod options;
mod pad_art;
mod pause;
mod perf_log;
mod race;
mod render_scale;
mod settings;
mod shapes;
mod sound;
mod speedometer;
#[cfg(test)]
mod testing;
mod track;
mod track_select;
mod ui;
mod vector_art;
#[cfg(target_arch = "wasm32")]
mod web;
mod welcome;
mod world;

use bevy::prelude::*;

/// Pasos por segundo de la simulación. La física avanza siempre a este ritmo
/// y el render interpola entre pasos, así la moto se conduce igual a 60 que a
/// 240 FPS y la simulación es reproducible.
const SIMULATION_HZ: f64 = 120.0;

/// At most this many steps of the simulation in a frame. After a frame that
/// came late (a hitch, a circuit put in the world), Bevy catches up with as
/// many steps as the time it took, up to a quarter of a second's (30 at
/// 120 Hz): the race jumps on at once, and that frame takes longer still.
/// Past these the game's time slows down for that frame instead: it keeps
/// to real time down to 15 frames a second.
const MAX_STEPS_PER_FRAME: u32 = 8;

/// The simulation's clocks: its fixed step, and the game's time, which goes
/// on at most [`MAX_STEPS_PER_FRAME`] of those steps in a frame.
fn simulation_clocks() -> (Time<Fixed>, Time<Virtual>) {
    let fixed = Time::<Fixed>::from_hz(SIMULATION_HZ);
    let game = Time::<Virtual>::from_max_delta(fixed.timestep() * MAX_STEPS_PER_FRAME);
    (fixed, game)
}

/// The game's version and the commit it is built from (see `build.rs`), for
/// its logs: "0.0.10 (1a2b3c4d5)", "-dirty" after it with changes not
/// committed.
pub const BUILD: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("MOTORUST_COMMIT"),
    ")"
);

fn init_crash_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // What the game is called, if it was read before it fell.
        let (title, folder) = game_info::read_already()
            .map_or(("The game", "game"), |game| (game.title, game.folder));
        let report = format!("{folder}-crash.log");
        let msg = format!(
            "{title} {BUILD} se ha cerrado por un error:\n\n{info}\n\nSe ha guardado un informe en {report}."
        );
        let _ = std::fs::write(&report, &msg);

        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let path = std::path::PathBuf::from(appdata)
                    .join(folder)
                    .join("crash.log");
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, &msg);
            }

            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;

            let title: Vec<u16> = OsStr::new(&format!("{title} - Error"))
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
    // The game's content pack, themes and terrain functions, before any
    // file is read.
    game::register();
    init_crash_handler();
    // On macOS, no App Nap, timers on time and no sleep while it runs.
    #[cfg(target_os = "macos")]
    latency::begin();

    // What the game is called (its folder holds the settings), read now.
    game_info::info();
    // Los ajustes guardados deciden cómo se abre la ventana.
    let settings = settings::Settings::load();
    // The benchmark rides the tracks on its own, with the player's settings
    // changed as it goes and never saved.
    #[cfg(not(target_arch = "wasm32"))]
    let bench = bench::requested();
    #[cfg(not(target_arch = "wasm32"))]
    let settings = if bench {
        bench::settings(settings)
    } else {
        settings
    };
    let mut app = App::new();
    let plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(settings.window()),
            ..default()
        })
        // Los mandos los lee `gamepads`, que además los vuelve a buscar.
        .disable::<bevy::gilrs::GilrsPlugin>();
    // Pipelined rendering draws each frame while the next one is
    // simulated: more frames per second when the CPU holds them back, but
    // each one a frame later after the controls. Options can turn it off
    // (less lag), from the next start.
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = if settings.pipelined_rendering {
        plugins
    } else {
        plugins.disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
    };
    // The CPU and memory in use, for the performance log (read on another
    // thread about once a second), only when it is written.
    #[cfg(not(target_arch = "wasm32"))]
    if settings.performance_log {
        app.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);
    }
    let (fixed, game) = simulation_clocks();
    // The content pack's source, before the asset server is built.
    app.add_plugins(content::ContentPlugin)
        .add_plugins(plugins)
        .insert_resource(fixed)
        .insert_resource(game)
        .insert_resource(settings::StartedWith::from(&settings))
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
            options::OptionsPlugin,
            settings::SettingsPlugin,
            sound::SoundPlugin,
            ui::UiPlugin,
        ))
        .add_plugins((
            race::RacePlugin,
            modes::ModesPlugin,
            perf_log::PerfLogPlugin,
            render_scale::RenderScalePlugin,
            curves_hud::CurvesHudPlugin,
            game::GamePlugin,
        ));
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(web::WebPlugin);
    #[cfg(not(target_arch = "wasm32"))]
    if bench {
        app.add_plugins(bench::BenchPlugin);
    }
    app.run();
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;

    #[derive(Resource, Default)]
    struct Steps(u32);

    #[test]
    fn a_late_frame_catches_up_by_a_few_steps_at_most() {
        let (fixed, game) = simulation_clocks();
        let step = fixed.timestep();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(fixed)
            .insert_resource(game)
            .init_resource::<Steps>()
            .add_systems(FixedUpdate, |mut steps: ResMut<Steps>| steps.0 += 1);
        let frame = |app: &mut App, millis: u64| {
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                millis,
            )));
            app.world_mut().resource_mut::<Steps>().0 = 0;
            app.update();
            app.world().resource::<Steps>().0
        };
        // The clocks start.
        frame(&mut app, 0);
        // A frame of 25 ms: three steps of 1/120 s.
        assert_eq!(frame(&mut app, 25), 3);
        // Half a second late (a hitch): eight steps, not Bevy's thirty (a
        // quarter of a second's), and the game's time on by as much.
        assert_eq!(frame(&mut app, 500), MAX_STEPS_PER_FRAME);
        let moved = app.world().resource::<Time<Virtual>>().delta();
        assert_eq!(moved, step * MAX_STEPS_PER_FRAME);
        // And back to keeping time.
        assert_eq!(frame(&mut app, 25), 3);
    }
}
