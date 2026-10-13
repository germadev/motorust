//! Ajustes que se guardan entre partidas: resolución, límite de fotogramas
//! por segundo y controles. Se leen al arrancar y se guardan cada vez que
//! cambian, en `ajustes.ron` dentro de la carpeta de ajustes del sistema (en
//! macOS, `~/Library/Application Support/motorust`); en la web, en el
//! almacenamiento del navegador.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};
use serde::{Deserialize, Serialize};

use crate::locale::Language;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_fullscreen, apply_settings).chain());
        // En la web, el navegador decide cuándo se dibuja cada fotograma.
        if !WEB {
            app.add_systems(Last, limit_frames);
            #[cfg(all(not(target_arch = "wasm32"), not(windows)))]
            app.add_systems(Update, set_window_icon);
        }
    }
}

/// Si el juego corre en el navegador.
pub const WEB: bool = cfg!(target_arch = "wasm32");

/// Todo lo que se puede ajustar.
#[derive(Resource, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fullscreen: bool,
    pub resolution: Resolution,
    pub fps: FpsLimit,
    pub language: Language,
    pub controls: Controls,
    pub fx_volume: u32,
    pub music_volume: u32,
    pub music_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            resolution: Resolution::default(),
            fps: FpsLimit::default(),
            language: Language::default(),
            controls: Controls::default(),
            fx_volume: 80,
            music_volume: 80,
            music_enabled: true,
        }
    }
}

/// Tamaño de la ventana (dinámica o fija en puntos —en una pantalla Retina, el
/// doble de píxeles—).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Resolution {
    #[default]
    Dynamic,
    Window(u32, u32),
    #[doc(hidden)]
    Fullscreen,
}

/// Las resoluciones que se pueden elegir, en orden. En la web, el juego
/// ocupa la página.
pub const RESOLUTIONS: &[Resolution] = if WEB {
    &[Resolution::Dynamic]
} else {
    &[
        Resolution::Dynamic,
        Resolution::Window(1280, 720),
        Resolution::Window(1600, 900),
        Resolution::Window(1920, 1080),
        Resolution::Window(2560, 1440),
    ]
};

impl Resolution {
    pub fn name(self, window: Option<&Window>, lang: Language) -> String {
        if WEB {
            return match window {
                Some(w) if w.physical_width() > 0 && w.physical_height() > 0 => {
                    format!(
                        "{} ({} × {})",
                        lang.dynamic(),
                        w.physical_width(),
                        w.physical_height()
                    )
                }
                _ => lang.dynamic().into(),
            };
        }
        match self {
            Resolution::Dynamic => match window {
                Some(w) if w.physical_width() > 0 && w.physical_height() > 0 => {
                    format!(
                        "{} ({} × {})",
                        lang.dynamic(),
                        w.physical_width(),
                        w.physical_height()
                    )
                }
                _ => lang.dynamic().into(),
            },
            Resolution::Window(width, height) => format!("{width} × {height}"),
            Resolution::Fullscreen => lang.fullscreen().into(),
        }
    }
}

/// Cuántos fotogramas por segundo como mucho: los de la pantalla, un número
/// fijo o tantos como se pueda.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum FpsLimit {
    #[default]
    Screen,
    Cap(u32),
    Unlimited,
}

/// Los límites que se pueden elegir, en orden.
pub const FPS_LIMITS: [FpsLimit; 5] = [
    FpsLimit::Screen,
    FpsLimit::Cap(30),
    FpsLimit::Cap(60),
    FpsLimit::Cap(120),
    FpsLimit::Unlimited,
];

impl FpsLimit {
    pub fn name(self, lang: Language) -> String {
        match self {
            FpsLimit::Screen => lang.fps_screen().into(),
            FpsLimit::Cap(fps) => fps.to_string(),
            FpsLimit::Unlimited => lang.fps_unlimited().into(),
        }
    }

    /// Con un número fijo también se espera a la pantalla para enseñar cada
    /// fotograma, así la imagen no se corta.
    fn present_mode(self) -> PresentMode {
        match self {
            FpsLimit::Screen | FpsLimit::Cap(_) => PresentMode::AutoVsync,
            FpsLimit::Unlimited => PresentMode::AutoNoVsync,
        }
    }
}

/// La opción de `options` que va `step` puestos después de `current`, dando la
/// vuelta. Si `current` no está entre ellas, se empieza por la primera.
pub fn cycle<T: Copy + PartialEq>(options: &[T], current: T, step: i32) -> T {
    let index = options.iter().position(|&o| o == current);
    let next = match index {
        Some(index) => (index as i32 + step).rem_euclid(options.len() as i32) as usize,
        None => 0,
    };
    options[next]
}

/// Lo que hace el jugador en la carrera y se puede cambiar de tecla o botón.
/// El stick izquierdo (girar y, hacia atrás, caballito), el derecho (mirar
/// alrededor), M (sonido) y lo que mueve los menús no se cambian.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Throttle,
    Brake,
    Left,
    Right,
    Wheelie,
    Restart,
    Pause,
}

pub const ACTIONS: [Action; 7] = [
    Action::Throttle,
    Action::Brake,
    Action::Left,
    Action::Right,
    Action::Wheelie,
    Action::Restart,
    Action::Pause,
];

impl Action {
    pub fn name(self, lang: Language) -> &'static str {
        match self {
            Action::Throttle => lang.action_throttle(),
            Action::Brake => lang.action_brake(),
            Action::Left => lang.action_left(),
            Action::Right => lang.action_right(),
            Action::Wheelie => lang.action_wheelie(),
            Action::Restart => lang.restart(),
            Action::Pause => lang.action_pause(),
        }
    }
}

/// Las teclas y los botones del mando de una acción.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Binding {
    pub keys: Vec<KeyCode>,
    pub buttons: Vec<GamepadButton>,
}

impl Binding {
    fn new(keys: &[KeyCode], buttons: &[GamepadButton]) -> Self {
        Self {
            keys: keys.to_vec(),
            buttons: buttons.to_vec(),
        }
    }
}

/// Las teclas y botones de cada acción.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Controls {
    pub throttle: Binding,
    pub brake: Binding,
    pub left: Binding,
    pub right: Binding,
    pub wheelie: Binding,
    pub restart: Binding,
    pub pause: Binding,
}

impl Default for Controls {
    fn default() -> Self {
        use GamepadButton as B;
        use KeyCode as K;
        Self {
            throttle: Binding::new(&[K::KeyW, K::ArrowUp], &[B::RightTrigger2, B::South]),
            brake: Binding::new(&[K::KeyS, K::ArrowDown], &[B::LeftTrigger2, B::West]),
            left: Binding::new(&[K::KeyA, K::ArrowLeft], &[B::DPadLeft]),
            right: Binding::new(&[K::KeyD, K::ArrowRight], &[B::DPadRight]),
            wheelie: Binding::new(&[K::Space, K::ShiftLeft], &[B::East]),
            restart: Binding::new(&[K::KeyR], &[B::Select]),
            pause: Binding::new(&[K::Escape], &[B::Start]),
        }
    }
}

impl Controls {
    pub fn get(&self, action: Action) -> &Binding {
        match action {
            Action::Throttle => &self.throttle,
            Action::Brake => &self.brake,
            Action::Left => &self.left,
            Action::Right => &self.right,
            Action::Wheelie => &self.wheelie,
            Action::Restart => &self.restart,
            Action::Pause => &self.pause,
        }
    }

    fn get_mut(&mut self, action: Action) -> &mut Binding {
        match action {
            Action::Throttle => &mut self.throttle,
            Action::Brake => &mut self.brake,
            Action::Left => &mut self.left,
            Action::Right => &mut self.right,
            Action::Wheelie => &mut self.wheelie,
            Action::Restart => &mut self.restart,
            Action::Pause => &mut self.pause,
        }
    }

    /// The wheelie used to have no gamepad button (it was the left stick
    /// pulled back): settings saved then get B, unless B already does
    /// something else.
    fn give_the_wheelie_a_button(&mut self) {
        let east = GamepadButton::East;
        let taken = ACTIONS
            .iter()
            .any(|&action| self.get(action).buttons.contains(&east));
        if self.wheelie.buttons.is_empty() && !taken {
            self.wheelie.buttons.push(east);
        }
    }

    /// La acción pasa a hacerse con esta tecla (en lugar de las que tuviera),
    /// y la tecla deja de hacer cualquier otra.
    pub fn bind_key(&mut self, action: Action, key: KeyCode) {
        for other in ACTIONS {
            self.get_mut(other).keys.retain(|&k| k != key);
        }
        self.get_mut(action).keys = vec![key];
    }

    /// Lo mismo con un botón del mando.
    pub fn bind_button(&mut self, action: Action, button: GamepadButton) {
        for other in ACTIONS {
            self.get_mut(other).buttons.retain(|&b| b != button);
        }
        self.get_mut(action).buttons = vec![button];
    }

    /// 1 si se pulsa alguna tecla de la acción, y si no, 0.
    pub fn key_value(&self, action: Action, keyboard: &ButtonInput<KeyCode>) -> f32 {
        if keyboard.any_pressed(self.get(action).keys.iter().copied()) {
            1.0
        } else {
            0.0
        }
    }

    /// Cuánto se aprieta en este mando algún botón de la acción (0..1): los
    /// gatillos, según lo que se aprieten; los demás, a tope o nada.
    pub fn button_value(&self, action: Action, gamepad: &Gamepad) -> f32 {
        self.get(action)
            .buttons
            .iter()
            .map(|&button| gamepad.get(button).unwrap_or(0.0))
            .fold(0.0, f32::max)
    }

    /// Si se acaba de pulsar una tecla o un botón de la acción.
    pub fn just_pressed<'a>(
        &self,
        action: Action,
        keyboard: &ButtonInput<KeyCode>,
        gamepads: impl IntoIterator<Item = &'a Gamepad>,
    ) -> bool {
        let binding = self.get(action);
        keyboard.any_just_pressed(binding.keys.iter().copied())
            || gamepads.into_iter().any(|gamepad| {
                binding
                    .buttons
                    .iter()
                    .any(|&button| gamepad.just_pressed(button))
            })
    }

    /// La primera tecla de la acción, para las ayudas en pantalla.
    #[allow(dead_code)]
    pub fn first_key(&self, action: Action) -> String {
        self.get(action)
            .keys
            .first()
            .map_or("—".into(), |&key| key_name(key))
    }

    /// Las teclas y los botones de la acción, para la tabla de controles.
    pub fn describe(&self, action: Action) -> String {
        let binding = self.get(action);
        let list = |names: Vec<String>| {
            if names.is_empty() {
                "—".to_string()
            } else {
                names.join(", ")
            }
        };
        let buttons = if action == Action::Wheelie && binding.buttons.is_empty() {
            "RT + A".to_string()
        } else {
            list(binding.buttons.iter().map(|&b| button_name(b)).collect())
        };
        format!(
            "{}   ·   {}",
            list(binding.keys.iter().map(|&k| key_name(k)).collect()),
            buttons
        )
    }
}

/// Cómo se llama una tecla en pantalla (la fuente no tiene flechas). Los
/// signos son los de la tecla en un teclado estadounidense: el juego conoce
/// el sitio de cada tecla, no lo que pone en ella.
pub fn key_name(key: KeyCode) -> String {
    use KeyCode as K;
    let name = match key {
        K::ArrowUp => "Arriba",
        K::ArrowDown => "Abajo",
        K::ArrowLeft => "Izquierda",
        K::ArrowRight => "Derecha",
        K::Space => "Espacio",
        K::Enter => "Enter",
        K::NumpadEnter => "Enter (num.)",
        K::Escape => "Esc",
        K::Tab => "Tab",
        K::Backspace => "Borrar",
        K::CapsLock => "Bloq Mayús",
        K::ShiftLeft => "Mayús izq.",
        K::ShiftRight => "Mayús der.",
        K::ControlLeft => "Ctrl izq.",
        K::ControlRight => "Ctrl der.",
        K::AltLeft => "Alt izq.",
        K::AltRight => "Alt der.",
        K::SuperLeft if cfg!(target_os = "macos") => "Cmd izq.",
        K::SuperRight if cfg!(target_os = "macos") => "Cmd der.",
        K::Comma => ",",
        K::Period => ".",
        K::Minus => "-",
        K::Equal => "=",
        K::Slash => "/",
        K::Backslash => "\\",
        K::Semicolon => ";",
        K::Quote => "'",
        K::BracketLeft => "[",
        K::BracketRight => "]",
        K::Backquote => "`",
        _ => {
            // KeyW, Digit1, Numpad1, F1...: sin el prefijo.
            let name = format!("{key:?}");
            return match name.strip_prefix("Key").or(name.strip_prefix("Digit")) {
                Some(short) => short.to_string(),
                None => match name.strip_prefix("Numpad") {
                    Some(short) => format!("{short} (num.)"),
                    None => name,
                },
            };
        }
    };
    name.into()
}

/// Cómo se llama un botón del mando en pantalla: como en los de Xbox.
pub fn button_name(button: GamepadButton) -> String {
    use GamepadButton as B;
    let name = match button {
        B::South => "A",
        B::East => "B",
        B::West => "X",
        B::North => "Y",
        B::LeftTrigger => "LB",
        B::RightTrigger => "RB",
        B::LeftTrigger2 => "LT",
        B::RightTrigger2 => "RT",
        B::Select => "Select",
        B::Start => "Start",
        B::Mode => "Home",
        B::LeftThumb => "L3",
        B::RightThumb => "R3",
        B::DPadUp => "Cruceta arriba",
        B::DPadDown => "Cruceta abajo",
        B::DPadLeft => "Cruceta izq.",
        B::DPadRight => "Cruceta der.",
        other => return format!("{other:?}"),
    };
    name.into()
}

impl Settings {
    /// La ventana del juego con estos ajustes.
    pub fn window(&self) -> Window {
        let mut window = Window {
            title: "Motorust".into(),
            // En la web se dibuja en el canvas de la página y lo llena.
            canvas: WEB.then(|| "#motorust".into()),
            fit_canvas_to_parent: true,
            ..default()
        };
        self.configure(&mut window);
        window
    }

    fn configure(&self, window: &mut Window) {
        if self.fullscreen {
            window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        } else {
            window.mode = WindowMode::Windowed;
            match self.resolution {
                Resolution::Dynamic | Resolution::Fullscreen => {}
                Resolution::Window(width, height) => {
                    if !WEB {
                        window.resolution.set(width as f32, height as f32);
                    }
                }
            }
        }
        window.present_mode = self.fps.present_mode();
    }

    /// Los ajustes guardados, o los de siempre si no hay o no se entienden.
    /// Se lee antes de abrir la ventana, cuando aún no hay registro, así que
    /// los problemas se escriben directamente en la terminal.
    pub fn load() -> Self {
        let Some(text) = storage::read() else {
            return Self::default();
        };
        let mut settings = ron::from_str::<Self>(&text).unwrap_or_else(|error| {
            eprintln!(
                "No se entienden los ajustes de {}: {error}. Se usan los de siempre",
                storage::place()
            );
            Self::default()
        });
        if settings.resolution == Resolution::Fullscreen {
            settings.fullscreen = true;
            settings.resolution = Resolution::Dynamic;
        }
        settings.controls.give_the_wheelie_a_button();
        settings
    }

    fn save(&self) {
        let saved = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|error| error.to_string())
            .and_then(|text| storage::write(&text));
        if let Err(error) = saved {
            warn!(
                "No se pueden guardar los ajustes en {}: {error}",
                storage::place()
            );
        }
    }
}

/// Los ajustes, en un archivo de la carpeta de ajustes del sistema.
#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::path::PathBuf;

    /// Lo guardado, si hay algo.
    pub fn read() -> Option<String> {
        let path = path()?;
        match std::fs::read_to_string(&path) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                eprintln!(
                    "No se pueden leer los ajustes de {}: {error}",
                    path.display()
                );
                None
            }
        }
    }

    /// Guarda el texto. Si el sistema no tiene carpeta de ajustes, no se
    /// guarda en ningún sitio.
    pub fn write(text: &str) -> Result<(), String> {
        let Some(path) = path() else {
            return Ok(());
        };
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
        }
        std::fs::write(&path, text).map_err(|error| error.to_string())
    }

    /// Dónde se guardan, para los mensajes.
    pub fn place() -> String {
        path().map_or_else(|| "ningún sitio".into(), |path| path.display().to_string())
    }

    /// Dónde se guardan los ajustes en cada sistema.
    fn path() -> Option<PathBuf> {
        let var = |name| std::env::var_os(name).map(PathBuf::from);
        let folder = if cfg!(target_os = "macos") {
            var("HOME")?.join("Library/Application Support")
        } else if cfg!(windows) {
            var("APPDATA")?
        } else {
            match var("XDG_CONFIG_HOME").filter(|path| path.is_absolute()) {
                Some(folder) => folder,
                None => var("HOME")?.join(".config"),
            }
        };
        Some(folder.join("motorust").join("ajustes.ron"))
    }
}

/// Los ajustes, en el almacenamiento del navegador (`localStorage`), que
/// guarda lo de cada página aunque se cierre.
#[cfg(target_arch = "wasm32")]
mod storage {
    const KEY: &str = "motorust.ajustes";

    fn local() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    /// Lo guardado, si hay algo.
    pub fn read() -> Option<String> {
        local()?.get_item(KEY).ok()?
    }

    /// Guarda el texto.
    pub fn write(text: &str) -> Result<(), String> {
        local()
            .and_then(|storage| storage.set_item(KEY, text).ok())
            .ok_or_else(|| "el navegador no deja guardar".into())
    }

    /// Dónde se guardan, para los mensajes.
    pub fn place() -> String {
        "el navegador".into()
    }
}

/// Las teclas que ponen y quitan la pantalla completa: Cmd+Enter en macOS y
/// Alt+Enter en Windows y Linux. En la web valen las dos, que no se sabe en
/// qué sistema corre el navegador.
pub const FULLSCREEN_KEYS: &str = if cfg!(target_os = "macos") {
    "Cmd+Enter"
} else if WEB {
    "Alt+Enter o Cmd+Enter"
} else {
    "Alt+Enter"
};

/// Si se acaban de pulsar las teclas de la pantalla completa
/// ([`FULLSCREEN_KEYS`]). Ese Enter no cuenta para los menús.
pub fn fullscreen_keys(keyboard: &ButtonInput<KeyCode>) -> bool {
    let command = keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
    let alt = keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let held = if cfg!(target_os = "macos") {
        command
    } else if WEB {
        command || alt
    } else {
        alt
    };
    held && keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
}

/// Con [`FULLSCREEN_KEYS`], conmuta la pantalla completa. Se guarda como si
/// se hubiera elegido en Opciones.
fn toggle_fullscreen(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<Settings>) {
    if !fullscreen_keys(&keyboard) {
        return;
    }
    settings.fullscreen = !settings.fullscreen;
}

/// Al cambiar los ajustes se guardan y, si cambian la pantalla completa, la
/// resolución o el límite de FPS, la ventana se pone como piden.
fn apply_settings(
    settings: Res<Settings>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut applied: Local<Option<(bool, Resolution, FpsLimit)>>,
) {
    let wanted = (settings.fullscreen, settings.resolution, settings.fps);
    // La ventana ya se abrió con los ajustes guardados.
    let applied = applied.get_or_insert(wanted);
    if !settings.is_changed() || settings.is_added() {
        return;
    }
    if *applied != wanted
        && let Ok(mut window) = window.single_mut()
    {
        settings.configure(&mut window);
        *applied = wanted;
    }
    settings.save();
}

/// Con un límite fijo, al acabar cada fotograma se espera lo que falte para
/// no pasar de él. Cuenta desde el plazo anterior, no desde que se acabó la
/// espera, así el ritmo no se va retrasando.
fn limit_frames(settings: Res<Settings>, mut deadline: Local<Option<Instant>>) {
    let FpsLimit::Cap(fps) = settings.fps else {
        *deadline = None;
        return;
    };
    let frame = Duration::from_secs_f64(1.0 / f64::from(fps.max(1)));
    let now = Instant::now();
    *deadline = Some(match *deadline {
        Some(time) if time > now => {
            std::thread::sleep(time - now);
            time + frame
        }
        // El primero, o uno que ya llega tarde: se empieza a contar de nuevo.
        _ => now + frame,
    });
}

/// En Linux y macOS, pone el icono de la ventana en cuanto el manejador de
/// ventanas de winit esté disponible. En Windows se incrusta en el ejecutable
/// con winres y el sistema operativo lo aplica directamente.
#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
fn set_window_icon(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Option<NonSend<bevy::winit::WinitWindows>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(windows) = windows else {
        return;
    };
    let Ok(entity) = primary_window.single() else {
        return;
    };
    let Some(window) = windows.get_window(entity) else {
        return;
    };
    if let Ok(image) = image::load_from_memory(include_bytes!("../assets/icon-256.png")) {
        let rgba = image.into_rgba8();
        let (width, height) = rgba.dimensions();
        if let Ok(icon) = winit::window::Icon::from_rgba(rgba.into_raw(), width, height) {
            window.set_window_icon(Some(icon));
        }
    }
    *done = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_survive_saving_and_loading() {
        let mut settings = Settings {
            fullscreen: true,
            resolution: Resolution::Dynamic,
            fps: FpsLimit::Cap(60),
            language: Language::Spanish,
            ..default()
        };
        settings.controls.bind_key(Action::Wheelie, KeyCode::KeyJ);
        settings
            .controls
            .bind_button(Action::Throttle, GamepadButton::RightTrigger);
        let text = ron::ser::to_string_pretty(&settings, default()).unwrap();
        let loaded: Settings = ron::from_str(&text).unwrap();
        assert_eq!(loaded, settings);
        // Un archivo a medias (de una versión anterior, por ejemplo) completa
        // lo que falta con lo de siempre.
        let partial: Settings = ron::from_str("(fps: Unlimited)").unwrap();
        assert_eq!(partial.fps, FpsLimit::Unlimited);
        assert_eq!(partial.controls, Controls::default());
    }

    #[test]
    fn a_key_does_only_one_thing() {
        let mut controls = Controls::default();
        // W pasa a frenar: deja de acelerar, que se queda con la flecha.
        controls.bind_key(Action::Brake, KeyCode::KeyW);
        assert_eq!(controls.brake.keys, [KeyCode::KeyW]);
        assert_eq!(controls.throttle.keys, [KeyCode::ArrowUp]);
        // Los botones van aparte: frenar sigue con los suyos.
        assert_eq!(
            controls.brake.buttons,
            [GamepadButton::LeftTrigger2, GamepadButton::West]
        );
        controls.bind_button(Action::Wheelie, GamepadButton::South);
        assert_eq!(controls.wheelie.buttons, [GamepadButton::South]);
        assert_eq!(controls.throttle.buttons, [GamepadButton::RightTrigger2]);
    }

    #[test]
    fn options_cycle_around() {
        assert_eq!(
            cycle(RESOLUTIONS, RESOLUTIONS[RESOLUTIONS.len() - 1], 1),
            RESOLUTIONS[0]
        );
        assert_eq!(
            cycle(&FPS_LIMITS, FpsLimit::Screen, -1),
            FpsLimit::Unlimited
        );
        // Una que no está en la lista (escrita a mano en el archivo) vuelve a
        // la primera.
        assert_eq!(
            cycle(RESOLUTIONS, Resolution::Window(800, 600), 1),
            RESOLUTIONS[0]
        );
    }

    #[test]
    fn dynamic_resolution_has_name() {
        assert_eq!(
            Resolution::Dynamic.name(None, Language::Spanish),
            "Dinámica"
        );
        assert_eq!(Resolution::Dynamic.name(None, Language::English), "Dynamic");
        assert_eq!(RESOLUTIONS[0], Resolution::Dynamic);
        assert_eq!(Resolution::default(), Resolution::Dynamic);
    }

    #[test]
    fn a_fixed_limit_holds_the_frames_back() {
        let mut app = App::new();
        app.insert_resource(Settings {
            fps: FpsLimit::Cap(50),
            ..default()
        })
        .add_systems(Update, limit_frames);
        let start = Instant::now();
        for _ in 0..6 {
            app.update();
        }
        // El primero no espera; los otros cinco, lo que falte hasta 20 ms.
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(98), "{elapsed:?}");
    }

    #[test]
    fn the_fullscreen_keys_go_there_and_back() {
        let mut app = App::new();
        app.insert_resource(Settings {
            fullscreen: false,
            ..default()
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, toggle_fullscreen);
        let modifier = if cfg!(target_os = "macos") {
            KeyCode::SuperLeft
        } else {
            KeyCode::AltLeft
        };
        let mut press = |keys: &[KeyCode]| {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.release_all();
            keyboard.clear();
            for &key in keys {
                keyboard.press(key);
            }
            app.update();
            app.world().resource::<Settings>().fullscreen
        };
        // Enter solo no hace nada; con la otra tecla, conmuta pantalla completa.
        assert!(!press(&[KeyCode::Enter]));
        assert!(press(&[modifier, KeyCode::Enter]));
        assert!(!press(&[modifier, KeyCode::Enter]));
    }

    #[test]
    fn keys_and_buttons_have_readable_names() {
        assert_eq!(key_name(KeyCode::KeyW), "W");
        assert_eq!(key_name(KeyCode::Digit3), "3");
        assert_eq!(key_name(KeyCode::Numpad3), "3 (num.)");
        assert_eq!(key_name(KeyCode::ArrowUp), "Arriba");
        assert_eq!(key_name(KeyCode::F5), "F5");
        assert_eq!(button_name(GamepadButton::RightTrigger2), "RT");
        let controls = Controls::default();
        assert_eq!(controls.describe(Action::Throttle), "W, Arriba   ·   RT, A");
        assert_eq!(
            controls.describe(Action::Wheelie),
            "Espacio, Mayús izq.   ·   B"
        );
        assert_eq!(controls.first_key(Action::Pause), "Esc");
    }

    #[test]
    fn saved_controls_without_a_wheelie_button_get_b() {
        let mut saved = Controls::default();
        saved.wheelie.buttons.clear();
        saved.give_the_wheelie_a_button();
        assert_eq!(saved.wheelie.buttons, [GamepadButton::East]);
        // Unless B already does something else.
        let mut taken = Controls::default();
        taken.bind_button(Action::Brake, GamepadButton::East);
        taken.give_the_wheelie_a_button();
        assert!(taken.wheelie.buttons.is_empty());
        assert_eq!(taken.brake.buttons, [GamepadButton::East]);
    }
}
