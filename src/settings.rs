//! Ajustes que se guardan entre partidas: resolución, límite de fotogramas
//! por segundo y controles. Se leen al arrancar y se guardan cada vez que
//! cambian, en `ajustes.ron` dentro de la carpeta de ajustes del sistema (en
//! macOS, `~/Library/Application Support/motorust`); en la web, en el
//! almacenamiento del navegador.

use std::num::NonZero;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::{
    Monitor, MonitorSelection, OnMonitor, PresentMode, PrimaryMonitor, PrimaryWindow, VideoMode,
    VideoModeSelection, WindowMode,
};
use serde::{Deserialize, Serialize};

use crate::locale::Language;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleMute>()
            .add_systems(
                PreUpdate,
                gamepad_shortcuts.after(bevy::input::InputSystems),
            )
            .add_systems(Update, (toggle_fullscreen, apply_settings).chain());
        // En la web, el navegador decide cuándo se dibuja cada fotograma.
        if !WEB {
            // The cap is kept where the frame is shown, in the render world.
            app.init_resource::<FrameCap>()
                .add_plugins(ExtractResourcePlugin::<FrameCap>::default())
                .add_systems(Update, cap_frames);
            if let Some(render) = app.get_sub_app_mut(RenderApp) {
                render.add_systems(Render, limit_frames.in_set(RenderSystems::PostCleanup));
            }
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
    /// Volume of the sound effects and of the music (0 to 100 %), see
    /// [`crate::sound::fx_gain`] and [`crate::sound::music_gain`]. They are
    /// saved under new names: the scale changed, and the levels saved with
    /// the old one would now sound different, so they start again from
    /// [`DEFAULT_VOLUME`].
    #[serde(rename = "fx_level")]
    pub fx_volume: u32,
    #[serde(rename = "music_level")]
    pub music_volume: u32,
    pub music_enabled: bool,
    /// How races start: at the lights, or when a player opens the throttle.
    pub start: RaceStart,
    /// How fast the rivals ride, and how much time the checkpoints give.
    pub difficulty: Difficulty,
    /// Whether each frame is drawn while the next one is simulated
    /// (Bevy's pipelined rendering): more frames per second when the CPU
    /// holds them back, a frame more between the controls and the screen.
    /// Applied when the game starts.
    pub pipelined_rendering: bool,
    /// How many frames may wait to go to the GPU (`desired_maximum_frame_latency`):
    /// fewer, less lag; more, steadier frames. Applied when the game starts.
    pub frame_latency: u32,
    /// The best of each track (see [`crate::race::Records`]).
    pub records: crate::race::Records,
}

/// How hard the races are.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    /// The rivals as they have always been.
    #[default]
    Medium,
    Hard,
}

/// The difficulties that can be chosen, in order.
pub const DIFFICULTIES: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

impl Difficulty {
    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Difficulty::Easy, Language::Spanish) => "Fácil",
            (Difficulty::Easy, Language::English) => "Easy",
            (Difficulty::Medium, Language::Spanish) => "Media",
            (Difficulty::Medium, Language::English) => "Medium",
            (Difficulty::Hard, Language::Spanish) => "Difícil",
            (Difficulty::Hard, Language::English) => "Hard",
        }
    }

    /// How much of its bike a rival dares to use, for one that uses
    /// `skill` on medium: less on easy, more on hard (never more than all).
    pub fn rival_skill(self, skill: f32) -> f32 {
        match self {
            Difficulty::Easy => skill - 0.07,
            Difficulty::Medium => skill,
            Difficulty::Hard => (skill + 0.045).min(1.0),
        }
    }
}

/// The frame latencies that can be chosen, and Bevy's own.
pub const FRAME_LATENCIES: [u32; 3] = [1, 2, 3];
pub const DEFAULT_FRAME_LATENCY: u32 = 2;

/// The graphics settings the game started with: those applied only when it
/// starts ([`Settings::pipelined_rendering`], [`Settings::frame_latency`]).
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StartedWith {
    pub pipelined_rendering: bool,
    pub frame_latency: u32,
}

impl From<&Settings> for StartedWith {
    fn from(settings: &Settings) -> Self {
        Self {
            pipelined_rendering: settings.pipelined_rendering,
            frame_latency: settings.frame_latency,
        }
    }
}

/// How a race starts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum RaceStart {
    /// At the lights: five red lights come on one by one and the race
    /// starts when they go out (see [`crate::race::lights`]).
    #[default]
    Lights,
    /// When a player first opens the throttle.
    Throttle,
}

/// The starts that can be chosen, in order.
pub const RACE_STARTS: [RaceStart; 2] = [RaceStart::Lights, RaceStart::Throttle];

impl RaceStart {
    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (RaceStart::Lights, Language::Spanish) => "Semáforo",
            (RaceStart::Lights, Language::English) => "Lights",
            (RaceStart::Throttle, Language::Spanish) => "Acelerando",
            (RaceStart::Throttle, Language::English) => "Throttle",
        }
    }
}

/// Volume the effects and the music start at (%).
pub const DEFAULT_VOLUME: u32 = 80;

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            resolution: Resolution::default(),
            fps: FpsLimit::default(),
            language: Language::default(),
            controls: Controls::default(),
            fx_volume: DEFAULT_VOLUME,
            music_volume: DEFAULT_VOLUME,
            music_enabled: true,
            start: RaceStart::default(),
            difficulty: Difficulty::default(),
            pipelined_rendering: true,
            frame_latency: DEFAULT_FRAME_LATENCY,
            records: crate::race::Records::default(),
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

/// Las teclas y los botones del mando de una acción: each on its own, or
/// several pressed together.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Binding {
    pub keys: Vec<KeyCode>,
    pub buttons: Vec<GamepadButton>,
    pub key_combos: Vec<Vec<KeyCode>>,
    pub button_combos: Vec<Vec<GamepadButton>>,
}

impl Binding {
    fn new(keys: &[KeyCode], buttons: &[GamepadButton]) -> Self {
        Self {
            keys: keys.to_vec(),
            buttons: buttons.to_vec(),
            ..default()
        }
    }
}

/// A kind of input an action is bound to: the keys of the keyboard or the
/// buttons of a gamepad, each kind remapped on its own.
pub trait Bound: Copy + PartialEq + std::fmt::Debug + 'static {
    fn singles(binding: &Binding) -> &Vec<Self>;
    fn singles_mut(binding: &mut Binding) -> &mut Vec<Self>;
    fn combos(binding: &Binding) -> &Vec<Vec<Self>>;
    fn combos_mut(binding: &mut Binding) -> &mut Vec<Vec<Self>>;
}

impl Bound for KeyCode {
    fn singles(binding: &Binding) -> &Vec<Self> {
        &binding.keys
    }
    fn singles_mut(binding: &mut Binding) -> &mut Vec<Self> {
        &mut binding.keys
    }
    fn combos(binding: &Binding) -> &Vec<Vec<Self>> {
        &binding.key_combos
    }
    fn combos_mut(binding: &mut Binding) -> &mut Vec<Vec<Self>> {
        &mut binding.key_combos
    }
}

impl Bound for GamepadButton {
    fn singles(binding: &Binding) -> &Vec<Self> {
        &binding.buttons
    }
    fn singles_mut(binding: &mut Binding) -> &mut Vec<Self> {
        &mut binding.buttons
    }
    fn combos(binding: &Binding) -> &Vec<Vec<Self>> {
        &binding.button_combos
    }
    fn combos_mut(binding: &mut Binding) -> &mut Vec<Vec<Self>> {
        &mut binding.button_combos
    }
}

/// Whether two combinations are the same keys or buttons, in any order.
fn same_combo<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x))
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

    /// The action `input` does, on its own or with others: the first one
    /// it is bound to.
    pub fn action_of<T: Bound>(&self, input: T) -> Option<Action> {
        ACTIONS
            .into_iter()
            .find(|&action| T::singles(self.get(action)).contains(&input))
            .or_else(|| {
                ACTIONS.into_iter().find(|&action| {
                    T::combos(self.get(action))
                        .iter()
                        .any(|combo| combo.contains(&input))
                })
            })
    }

    /// What `from` did (its action, on its own or the combination it is
    /// in) is done with `to` instead: one key or button, or several pressed
    /// together. `to` stops doing anything else, and the action keeps its
    /// other keys or buttons. The action, if `from` had one.
    pub fn remap<T: Bound>(&mut self, from: T, to: &[T]) -> Option<Action> {
        let action = self.action_of(from)?;
        if to.is_empty() {
            return Some(action);
        }
        // Where `from` was, to put `to` there.
        let binding = self.get_mut(action);
        let single = T::singles(binding).iter().position(|&x| x == from);
        match single {
            Some(index) => {
                T::singles_mut(binding).remove(index);
            }
            None => T::combos_mut(binding).retain(|combo| !combo.contains(&from)),
        }
        for other in ACTIONS {
            let binding = self.get_mut(other);
            match to {
                [one] => T::singles_mut(binding).retain(|x| x != one),
                _ => T::combos_mut(binding).retain(|combo| !same_combo(combo, to)),
            }
        }
        let binding = self.get_mut(action);
        match to {
            [one] => {
                let singles = T::singles_mut(binding);
                let index = single.unwrap_or(singles.len()).min(singles.len());
                singles.insert(index, *one);
            }
            _ => T::combos_mut(binding).push(to.to_vec()),
        }
        Some(action)
    }

    /// Every action back to its keys (or buttons) at first; the other kind
    /// is left as it is.
    pub fn reset<T: Bound>(&mut self) {
        let defaults = Controls::default();
        for action in ACTIONS {
            let default = defaults.get(action);
            let binding = self.get_mut(action);
            *T::singles_mut(binding) = T::singles(default).clone();
            *T::combos_mut(binding) = T::combos(default).clone();
        }
    }

    /// 1 si se pulsa alguna tecla de la acción (or all the keys of one of
    /// its combinations), y si no, 0.
    pub fn key_value(&self, action: Action, keyboard: &ButtonInput<KeyCode>) -> f32 {
        let binding = self.get(action);
        let combo = binding
            .key_combos
            .iter()
            .any(|combo| !combo.is_empty() && keyboard.all_pressed(combo.iter().copied()));
        if combo || keyboard.any_pressed(binding.keys.iter().copied()) {
            1.0
        } else {
            0.0
        }
    }

    /// Cuánto se aprieta en este mando algún botón de la acción (0..1): los
    /// gatillos, según lo que se aprieten; los demás, a tope o nada. A
    /// combination counts once all its buttons are pressed, as much as the
    /// least pressed of them.
    pub fn button_value(&self, action: Action, gamepad: &Gamepad) -> f32 {
        let binding = self.get(action);
        let combos = binding.button_combos.iter().map(|combo| {
            if !combo.is_empty() && combo.iter().all(|&button| gamepad.pressed(button)) {
                combo
                    .iter()
                    .map(|&button| gamepad.get(button).unwrap_or(0.0))
                    .fold(1.0, f32::min)
            } else {
                0.0
            }
        });
        binding
            .buttons
            .iter()
            .map(|&button| gamepad.get(button).unwrap_or(0.0))
            .chain(combos)
            .fold(0.0, f32::max)
    }

    /// Si se acaba de pulsar una tecla o un botón de la acción (or completed
    /// one of its combinations).
    pub fn just_pressed<'a>(
        &self,
        action: Action,
        keyboard: &ButtonInput<KeyCode>,
        gamepads: impl IntoIterator<Item = &'a Gamepad>,
    ) -> bool {
        let binding = self.get(action);
        let key_combo = binding.key_combos.iter().any(|combo| {
            !combo.is_empty()
                && keyboard.all_pressed(combo.iter().copied())
                && keyboard.any_just_pressed(combo.iter().copied())
        });
        key_combo
            || keyboard.any_just_pressed(binding.keys.iter().copied())
            || gamepads.into_iter().any(|gamepad| {
                binding
                    .buttons
                    .iter()
                    .any(|&button| gamepad.just_pressed(button))
                    || binding.button_combos.iter().any(|combo| {
                        !combo.is_empty()
                            && combo.iter().all(|&button| gamepad.pressed(button))
                            && combo.iter().any(|&button| gamepad.just_pressed(button))
                    })
            })
    }

    /// The keys of the action, a combination joined with "+".
    pub fn describe_keys(&self, action: Action, lang: Language) -> String {
        let binding = self.get(action);
        names(&binding.keys, &binding.key_combos, |&key| {
            key_name(key, lang)
        })
    }

    /// The gamepad buttons of the action, a combination joined with "+".
    pub fn describe_buttons(&self, action: Action, lang: Language) -> String {
        let binding = self.get(action);
        // The wheelie always has RT and A together too.
        if action == Action::Wheelie
            && binding.buttons.is_empty()
            && binding.button_combos.is_empty()
        {
            return "RT + A".to_string();
        }
        names(&binding.buttons, &binding.button_combos, |&button| {
            button_name(button, lang)
        })
    }
}

/// The names of some keys or buttons, and of combinations of them, in a list.
fn names<T>(singles: &[T], combos: &[Vec<T>], name: impl Fn(&T) -> String) -> String {
    let list: Vec<String> = singles
        .iter()
        .map(&name)
        .chain(
            combos
                .iter()
                .map(|combo| combo.iter().map(&name).collect::<Vec<_>>().join("+")),
        )
        .collect();
    if list.is_empty() {
        "—".to_string()
    } else {
        list.join(", ")
    }
}

/// Cómo se llama una tecla en pantalla (la fuente no tiene flechas). Los
/// signos son los de la tecla en un teclado estadounidense: el juego conoce
/// el sitio de cada tecla, no lo que pone en ella.
pub fn key_name(key: KeyCode, lang: Language) -> String {
    use KeyCode as K;
    let english = lang == Language::English;
    let name = match key {
        K::ArrowUp => {
            if english {
                "Up"
            } else {
                "Arriba"
            }
        }
        K::ArrowDown => {
            if english {
                "Down"
            } else {
                "Abajo"
            }
        }
        K::ArrowLeft => {
            if english {
                "Left"
            } else {
                "Izquierda"
            }
        }
        K::ArrowRight => {
            if english {
                "Right"
            } else {
                "Derecha"
            }
        }
        K::Space => {
            if english {
                "Space"
            } else {
                "Espacio"
            }
        }
        K::Enter => "Enter",
        K::NumpadEnter => "Enter (num.)",
        K::Escape => "Esc",
        K::Tab => "Tab",
        K::Backspace => {
            if english {
                "Backspace"
            } else {
                "Borrar"
            }
        }
        K::CapsLock => {
            if english {
                "Caps Lock"
            } else {
                "Bloq Mayús"
            }
        }
        K::ShiftLeft => {
            if english {
                "Left Shift"
            } else {
                "Mayús izq."
            }
        }
        K::ShiftRight => {
            if english {
                "Right Shift"
            } else {
                "Mayús der."
            }
        }
        K::ControlLeft => {
            if english {
                "Left Ctrl"
            } else {
                "Ctrl izq."
            }
        }
        K::ControlRight => {
            if english {
                "Right Ctrl"
            } else {
                "Ctrl der."
            }
        }
        K::AltLeft => {
            if english {
                "Left Alt"
            } else {
                "Alt izq."
            }
        }
        K::AltRight => {
            if english {
                "Right Alt"
            } else {
                "Alt der."
            }
        }
        K::SuperLeft if cfg!(target_os = "macos") => {
            if english {
                "Left Cmd"
            } else {
                "Cmd izq."
            }
        }
        K::SuperRight if cfg!(target_os = "macos") => {
            if english {
                "Right Cmd"
            } else {
                "Cmd der."
            }
        }
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
pub fn button_name(button: GamepadButton, lang: Language) -> String {
    use GamepadButton as B;
    let english = lang == Language::English;
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
        B::DPadUp => {
            if english {
                "D-pad up"
            } else {
                "Cruceta arriba"
            }
        }
        B::DPadDown => {
            if english {
                "D-pad down"
            } else {
                "Cruceta abajo"
            }
        }
        B::DPadLeft => {
            if english {
                "D-pad left"
            } else {
                "Cruceta izq."
            }
        }
        B::DPadRight => {
            if english {
                "D-pad right"
            } else {
                "Cruceta der."
            }
        }
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
            // As chosen in Options (Bevy's 2 at first; 1 is less lag).
            desired_maximum_frame_latency: NonZero::new(
                if FRAME_LATENCIES.contains(&self.frame_latency) {
                    self.frame_latency
                } else {
                    DEFAULT_FRAME_LATENCY
                },
            ),
            ..default()
        };
        self.configure(&mut window, None);
        window
    }

    /// Whether the screen itself is switched to the resolution chosen: in
    /// fullscreen with a fixed one, out of the browser. Then the monitor's
    /// display modes are needed, known only once the window is open.
    fn exclusive(&self) -> bool {
        !WEB && self.fullscreen && matches!(self.resolution, Resolution::Window(..))
    }

    /// Sets `window` as these settings ask. In fullscreen with a fixed
    /// resolution, the screen is switched to the display mode of `monitor`
    /// that comes closest to it (in pixels); with a dynamic one, or with no
    /// monitor known yet, the window covers the screen as it is.
    fn configure(&self, window: &mut Window, monitor: Option<&Monitor>) {
        if self.fullscreen {
            let mode = match (self.resolution, monitor) {
                (Resolution::Window(width, height), Some(monitor)) if !WEB => {
                    fitting_mode(&monitor.video_modes, width, height)
                }
                _ => None,
            };
            window.mode = match mode {
                Some(mode) => WindowMode::Fullscreen(
                    MonitorSelection::Current,
                    VideoModeSelection::Specific(mode),
                ),
                None => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            };
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

/// Si se acaban de pulsar las teclas de la pantalla completa: Cmd+Enter en
/// macOS y Alt+Enter en Windows y Linux. En la web valen las dos, que no se
/// sabe en qué sistema corre el navegador. Ese Enter no cuenta para los
/// menús.
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

/// The shortcuts of a gamepad: Select held with Start switches fullscreen,
/// with Y (the top face button) the sound off and on.
pub const SHORTCUT_HELD: GamepadButton = GamepadButton::Select;
pub const FULLSCREEN_BUTTON: GamepadButton = GamepadButton::Start;
pub const MUTE_BUTTON: GamepadButton = GamepadButton::North;

/// Asks for the sound to be turned off, or back on (M, or Select and Y).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToggleMute;

/// What a gamepad shortcut asks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shortcut {
    Fullscreen,
    Mute,
}

/// How a gamepad's Select is being held: since it went down, and whether
/// a shortcut was made with it.
#[derive(Clone, Copy, Default, Debug)]
pub struct ShortcutHold {
    held: bool,
    used: bool,
    /// A press of Select alone was handed on: it is let go of next.
    handed_on: bool,
}

/// Reads the shortcuts of a gamepad's buttons, taking their presses out
/// of them so that nothing else acts on them: Select (which restarts the
/// race) goes down unseen and, if no shortcut is made while it is held,
/// counts as pressed when it is let go of; Start (which pauses or accepts)
/// and Y held with Select are the shortcut's.
pub fn read_shortcut(
    buttons: &mut ButtonInput<GamepadButton>,
    hold: &mut ShortcutHold,
) -> Option<Shortcut> {
    if hold.handed_on {
        buttons.release(SHORTCUT_HELD);
        buttons.clear_just_released(SHORTCUT_HELD);
        hold.handed_on = false;
    }
    if buttons.clear_just_pressed(SHORTCUT_HELD) {
        *hold = ShortcutHold {
            held: true,
            ..default()
        };
    }
    let mut shortcut = None;
    if hold.held && buttons.pressed(SHORTCUT_HELD) {
        if buttons.clear_just_pressed(FULLSCREEN_BUTTON) {
            shortcut = Some(Shortcut::Fullscreen);
        } else if buttons.clear_just_pressed(MUTE_BUTTON) {
            shortcut = Some(Shortcut::Mute);
        }
        hold.used |= shortcut.is_some();
    }
    if hold.held && buttons.just_released(SHORTCUT_HELD) {
        hold.held = false;
        if !hold.used {
            // Select alone: pressed now, for a frame.
            buttons.press(SHORTCUT_HELD);
            hold.handed_on = true;
        }
    }
    shortcut
}

/// The gamepads' shortcuts: fullscreen and sound.
fn gamepad_shortcuts(
    mut gamepads: Query<(Entity, &mut Gamepad)>,
    mut holds: Local<bevy::platform::collections::HashMap<Entity, ShortcutHold>>,
    mut settings: ResMut<Settings>,
    mut mute: MessageWriter<ToggleMute>,
) {
    for (entity, mut gamepad) in &mut gamepads {
        let hold = holds.entry(entity).or_default();
        match read_shortcut(gamepad.bypass_change_detection().digital_mut(), hold) {
            Some(Shortcut::Fullscreen) => settings.fullscreen = !settings.fullscreen,
            Some(Shortcut::Mute) => {
                mute.write(ToggleMute);
            }
            None => {}
        }
    }
}

/// With the fullscreen keys ([`fullscreen_keys`]), conmuta la pantalla completa. Se guarda como si
/// se hubiera elegido en Opciones.
fn toggle_fullscreen(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<Settings>) {
    if !fullscreen_keys(&keyboard) {
        return;
    }
    settings.fullscreen = !settings.fullscreen;
}

/// The display mode of a monitor that comes closest to `width` × `height`
/// pixels: the nearest size (the same one if there is), then the fastest
/// refresh rate and the deepest colour.
pub fn fitting_mode(modes: &[VideoMode], width: u32, height: u32) -> Option<VideoMode> {
    modes.iter().copied().min_by_key(|mode| {
        let size = mode.physical_size;
        (
            size.x.abs_diff(width) + size.y.abs_diff(height),
            std::cmp::Reverse(mode.refresh_rate_millihertz),
            std::cmp::Reverse(mode.bit_depth),
        )
    })
}

/// Al cambiar los ajustes se guardan y, si cambian la pantalla completa, la
/// resolución o el límite de FPS, la ventana se pone como piden.
#[allow(clippy::type_complexity)]
fn apply_settings(
    settings: Res<Settings>,
    mut window: Query<(&mut Window, Option<&OnMonitor>), With<PrimaryWindow>>,
    monitors: Query<(&Monitor, Has<PrimaryMonitor>)>,
    mut applied: Local<Option<(bool, Resolution, FpsLimit)>>,
) {
    let wanted = (settings.fullscreen, settings.resolution, settings.fps);
    // La ventana ya se abrió con los ajustes guardados; salvo que la
    // pantalla deba pasar a la resolución elegida, que espera a conocer los
    // modos del monitor.
    let applied = applied.get_or_insert(if settings.exclusive() {
        (false, Resolution::Dynamic, settings.fps)
    } else {
        wanted
    });
    if *applied != wanted
        && let Ok((mut window, on)) = window.single_mut()
    {
        // The monitor the window is on, or else the main one.
        let monitor = on
            .and_then(|on| monitors.get(on.0).ok())
            .or_else(|| monitors.iter().find(|(_, primary)| *primary))
            .map(|(monitor, _)| monitor);
        if monitor.is_some() || !settings.exclusive() {
            settings.configure(&mut window, monitor);
            *applied = wanted;
        }
    }
    if settings.is_changed() && !settings.is_added() {
        settings.save();
    }
}

/// The frame rate cap chosen, if any ([`FpsLimit::Cap`]), copied into the
/// render world, where [`limit_frames`] keeps to it.
#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug, ExtractResource)]
pub struct FrameCap(pub Option<u32>);

fn cap_frames(settings: Res<Settings>, mut cap: ResMut<FrameCap>) {
    let wanted = FrameCap(match settings.fps {
        FpsLimit::Cap(fps) => Some(fps),
        FpsLimit::Screen | FpsLimit::Unlimited => None,
    });
    if *cap != wanted {
        *cap = wanted;
    }
}

/// With a fixed cap, waits whatever is left of each frame so as not to go
/// over it. It counts from the previous deadline, not from when the wait
/// ended, so the pace does not drift.
///
/// It waits right after the frame has been shown, in the render world:
/// then the next frame starts at once, reading the controls as they are
/// after the wait, and is shown as soon as it is drawn. Waiting before the
/// frame is drawn (at the end of the main world's update) would show
/// controls read before the wait, that much later.
fn limit_frames(cap: Res<FrameCap>, mut deadline: Local<Option<Instant>>) {
    let FrameCap(Some(fps)) = *cap else {
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
        settings.controls.remap(KeyCode::Space, &[KeyCode::KeyJ]);
        settings
            .controls
            .remap(GamepadButton::South, &[GamepadButton::RightTrigger]);
        settings
            .controls
            .remap(KeyCode::ShiftLeft, &[KeyCode::ControlLeft, KeyCode::KeyW]);
        // The records and the start are kept too.
        settings.records.lap("Bahía GP", 71.25);
        settings.records.checkpoint("Bahía GP", 3);
        settings.start = RaceStart::Throttle;
        settings.difficulty = Difficulty::Hard;
        settings.pipelined_rendering = false;
        settings.frame_latency = 1;
        let text = ron::ser::to_string_pretty(&settings, default()).unwrap();
        let loaded: Settings = ron::from_str(&text).unwrap();
        assert_eq!(loaded, settings);
        // Un archivo a medias (de una versión anterior, por ejemplo) completa
        // lo que falta con lo de siempre.
        let partial: Settings = ron::from_str("(fps: Unlimited)").unwrap();
        assert_eq!(partial.fps, FpsLimit::Unlimited);
        assert_eq!(partial.controls, Controls::default());
        assert_eq!(partial.start, RaceStart::Lights);
        // Medium, pipelined rendering and Bevy's frame latency at first.
        assert_eq!(partial.difficulty, Difficulty::Medium);
        assert!(partial.pipelined_rendering);
        assert_eq!(partial.frame_latency, 2);
        assert_eq!(
            partial.window().desired_maximum_frame_latency,
            NonZero::new(2)
        );
        // One that still keeps a championship under way (the arcade no
        // longer does) loads without it.
        let older: Settings = ron::from_str(
            "(records: (story: Some((round: 2, points: (10, 8, 6, 5, 4, 3, 2, 1))), \
             best_laps: [(\"Bahía GP\", 71.25)]))",
        )
        .unwrap();
        assert_eq!(older.records.best_lap("Bahía GP"), Some(71.25));
    }

    #[test]
    fn volumes_saved_with_the_old_scale_start_again_from_the_default() {
        // The old scale: effects at 20 % and music at 90 % is the balance
        // that the new scale calls 80 %.
        let old: Settings = ron::from_str("(fx_volume: 20, music_volume: 90)").unwrap();
        assert_eq!(old.fx_volume, DEFAULT_VOLUME);
        assert_eq!(old.music_volume, DEFAULT_VOLUME);
        let new: Settings = ron::from_str("(fx_level: 50, music_level: 100)").unwrap();
        assert_eq!((new.fx_volume, new.music_volume), (50, 100));
    }

    #[test]
    fn a_key_does_only_one_thing() {
        let mut controls = Controls::default();
        // W pasa a frenar en lugar de S: deja de acelerar, que se queda con
        // la flecha, y frenar sigue también con la suya.
        assert_eq!(
            controls.remap(KeyCode::KeyS, &[KeyCode::KeyW]),
            Some(Action::Brake)
        );
        assert_eq!(controls.brake.keys, [KeyCode::KeyW, KeyCode::ArrowDown]);
        assert_eq!(controls.throttle.keys, [KeyCode::ArrowUp]);
        // Los botones van aparte: frenar sigue con los suyos.
        assert_eq!(
            controls.brake.buttons,
            [GamepadButton::LeftTrigger2, GamepadButton::West]
        );
        // B, the wheelie's, becomes A, which stops accelerating.
        controls.remap(GamepadButton::East, &[GamepadButton::South]);
        assert_eq!(controls.wheelie.buttons, [GamepadButton::South]);
        assert_eq!(controls.throttle.buttons, [GamepadButton::RightTrigger2]);
        // A key that does nothing has nothing to remap.
        assert_eq!(controls.remap(KeyCode::KeyK, &[KeyCode::KeyL]), None);
    }

    #[test]
    fn an_action_can_take_keys_pressed_together() {
        let mut controls = Controls::default();
        let together = [KeyCode::ShiftLeft, KeyCode::KeyR];
        assert_eq!(
            controls.remap(KeyCode::KeyR, &together),
            Some(Action::Restart)
        );
        assert!(controls.restart.keys.is_empty());
        assert_eq!(controls.restart.key_combos, [together.to_vec()]);
        // Shift on its own still does the wheelie.
        assert!(controls.wheelie.keys.contains(&KeyCode::ShiftLeft));
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::KeyR);
        assert_eq!(controls.key_value(Action::Restart, &keyboard), 0.0);
        assert!(!controls.just_pressed(Action::Restart, &keyboard, []));
        keyboard.clear();
        keyboard.press(KeyCode::ShiftLeft);
        assert!(controls.just_pressed(Action::Restart, &keyboard, []));
        assert_eq!(controls.key_value(Action::Restart, &keyboard), 1.0);
        assert_eq!(
            controls.describe_keys(Action::Restart, Language::English),
            "Left Shift+R"
        );
        // A key of a combination remaps it all.
        assert_eq!(controls.action_of(KeyCode::KeyR), Some(Action::Restart));
        controls.remap(KeyCode::KeyR, &[KeyCode::F5]);
        assert_eq!(controls.restart.keys, [KeyCode::F5]);
        assert!(controls.restart.key_combos.is_empty());
    }

    #[test]
    fn the_keyboard_and_the_gamepad_are_reset_apart() {
        let mut controls = Controls::default();
        controls.remap(KeyCode::KeyW, &[KeyCode::KeyI]);
        controls.remap(GamepadButton::South, &[GamepadButton::North]);
        controls.reset::<KeyCode>();
        assert_eq!(controls.throttle.keys, Controls::default().throttle.keys);
        assert!(controls.throttle.buttons.contains(&GamepadButton::North));
        controls.reset::<GamepadButton>();
        assert_eq!(controls, Controls::default());
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
    fn fullscreen_takes_the_display_mode_nearest_the_resolution() {
        let mode = |width: u32, height: u32, hz: u32| VideoMode {
            physical_size: UVec2::new(width, height),
            bit_depth: 32,
            refresh_rate_millihertz: hz * 1000,
        };
        let modes = [
            mode(2560, 1600, 60),
            mode(1920, 1200, 60),
            mode(1280, 720, 60),
            mode(1280, 720, 120),
            mode(1024, 768, 60),
        ];
        // The same size, at its fastest rate.
        let found = fitting_mode(&modes, 1280, 720).unwrap();
        assert_eq!(found, mode(1280, 720, 120));
        // A size the screen lacks: the nearest one.
        assert_eq!(
            fitting_mode(&modes, 1920, 1080).unwrap().physical_size,
            UVec2::new(1920, 1200)
        );
        assert_eq!(fitting_mode(&[], 1920, 1080), None);
        // Only a fixed resolution in fullscreen switches the screen.
        let fixed = Settings {
            fullscreen: true,
            resolution: Resolution::Window(1280, 720),
            ..default()
        };
        assert!(fixed.exclusive());
        assert!(
            !Settings {
                resolution: Resolution::Dynamic,
                ..fixed.clone()
            }
            .exclusive()
        );
        let monitor = Monitor {
            name: None,
            physical_height: 1600,
            physical_width: 2560,
            physical_position: IVec2::ZERO,
            refresh_rate_millihertz: Some(60_000),
            scale_factor: 2.0,
            video_modes: modes.to_vec(),
        };
        let mut window = Window::default();
        fixed.configure(&mut window, Some(&monitor));
        assert_eq!(
            window.mode,
            WindowMode::Fullscreen(
                MonitorSelection::Current,
                VideoModeSelection::Specific(mode(1280, 720, 120))
            )
        );
        // Before the monitor is known, the window covers the screen as it is.
        fixed.configure(&mut window, None);
        assert_eq!(
            window.mode,
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        );
    }

    #[test]
    fn select_with_start_or_y_is_a_shortcut_and_alone_it_restarts_on_release() {
        use GamepadButton as B;
        let mut buttons = ButtonInput::<GamepadButton>::default();
        let mut hold = ShortcutHold::default();
        // A frame of the gamepad: what was pressed and let go of.
        let mut frame = |buttons: &mut ButtonInput<GamepadButton>, press: &[B], release: &[B]| {
            buttons.clear();
            for &button in press {
                buttons.press(button);
            }
            for &button in release {
                buttons.release(button);
            }
            read_shortcut(buttons, &mut hold)
        };
        // Select down: nobody sees it yet.
        assert_eq!(frame(&mut buttons, &[B::Select], &[]), None);
        assert!(!buttons.just_pressed(B::Select));
        // With Start: fullscreen, and Start does not pause.
        assert_eq!(
            frame(&mut buttons, &[B::Start], &[]),
            Some(Shortcut::Fullscreen)
        );
        assert!(!buttons.just_pressed(B::Start));
        // With Y: the sound.
        assert_eq!(frame(&mut buttons, &[B::North], &[]), Some(Shortcut::Mute));
        assert!(!buttons.just_pressed(B::North));
        // Let go of after a shortcut: Select does nothing.
        assert_eq!(
            frame(&mut buttons, &[], &[B::Select, B::Start, B::North]),
            None
        );
        assert!(!buttons.just_pressed(B::Select));
        // Select alone: pressed as it is let go of, for that frame only.
        frame(&mut buttons, &[B::Select], &[]);
        frame(&mut buttons, &[], &[B::Select]);
        assert!(buttons.just_pressed(B::Select));
        frame(&mut buttons, &[], &[]);
        assert!(!buttons.pressed(B::Select) && !buttons.just_pressed(B::Select));
        // Start alone still pauses.
        frame(&mut buttons, &[B::Start], &[]);
        assert!(buttons.just_pressed(B::Start));
    }

    #[test]
    fn the_rivals_ride_slower_on_easy_and_faster_on_hard() {
        for skill in [0.88, 0.95] {
            assert_eq!(Difficulty::Medium.rival_skill(skill), skill);
            assert!(Difficulty::Easy.rival_skill(skill) < skill - 0.05);
            assert!(Difficulty::Hard.rival_skill(skill) > skill);
            assert!(Difficulty::Hard.rival_skill(skill) <= 1.0);
        }
    }

    #[test]
    fn a_fixed_limit_holds_the_frames_back() {
        let mut app = App::new();
        app.insert_resource(Settings {
            fps: FpsLimit::Cap(50),
            ..default()
        })
        .init_resource::<FrameCap>()
        .add_systems(Update, (cap_frames, limit_frames).chain());
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
        let es = Language::Spanish;
        assert_eq!(key_name(KeyCode::KeyW, es), "W");
        assert_eq!(key_name(KeyCode::Digit3, es), "3");
        assert_eq!(key_name(KeyCode::Numpad3, es), "3 (num.)");
        assert_eq!(key_name(KeyCode::ArrowUp, es), "Arriba");
        assert_eq!(key_name(KeyCode::ArrowUp, Language::English), "Up");
        assert_eq!(key_name(KeyCode::F5, es), "F5");
        assert_eq!(button_name(GamepadButton::RightTrigger2, es), "RT");
        assert_eq!(
            button_name(GamepadButton::DPadLeft, Language::English),
            "D-pad left"
        );
        let controls = Controls::default();
        assert_eq!(controls.describe_keys(Action::Throttle, es), "W, Arriba");
        assert_eq!(controls.describe_buttons(Action::Throttle, es), "RT, A");
        assert_eq!(
            controls.describe_keys(Action::Throttle, Language::English),
            "W, Up"
        );
        assert_eq!(
            controls.describe_keys(Action::Wheelie, es),
            "Espacio, Mayús izq."
        );
        assert_eq!(controls.describe_buttons(Action::Wheelie, es), "B");
    }

    #[test]
    fn saved_controls_without_a_wheelie_button_get_b() {
        let mut saved = Controls::default();
        saved.wheelie.buttons.clear();
        saved.give_the_wheelie_a_button();
        assert_eq!(saved.wheelie.buttons, [GamepadButton::East]);
        // Unless B already does something else.
        let mut taken = Controls::default();
        taken.wheelie.buttons.clear();
        taken.brake.buttons = vec![GamepadButton::East];
        taken.give_the_wheelie_a_button();
        assert!(taken.wheelie.buttons.is_empty());
        assert_eq!(taken.brake.buttons, [GamepadButton::East]);
    }
}
