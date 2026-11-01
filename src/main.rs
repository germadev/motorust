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

fn init_crash_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // What the game is called, if it was read before it fell.
        let (title, folder) = motorust_engine::game_info::read_already()
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
    motorust_engine::latency::begin();
    motorust_engine::set_build(BUILD);

    let mut app = motorust_engine::app();
    app.add_plugins(game::GamePlugin);
    app.run();
}
