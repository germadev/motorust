//! Menú de opciones unificado con pestañas: Juego, Control, Sonido y Gráficos.
//! Accesible tanto desde la pantalla de bienvenida como desde el menú de pausa.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gamepads::{ActiveController, Gamepads};
use crate::hud::UiFont;
use crate::legend::{
    Keys, menu_prompts, section_badges, shortcut_prompts, spawn_legend, spawn_lone_badge,
    spawn_prompts,
};
use crate::locale::{LANGUAGES, Language};
use crate::mapping::{
    Held, MapDevice, Mapping, MappingLook, PAD_HEIGHT, spawn_mapping, use_mapping,
};
use crate::pad_art::{PadArt, PadArts, finish_pad_arts};
use crate::pause::{DebugOptions, ReturnRate, SteerRate, TurnLimit};
use crate::settings::{
    DEFAULT_FRAME_LATENCY, DIFFICULTIES, FPS_LIMITS, FRAME_LATENCIES, RACE_STARTS, RESOLUTIONS,
    START_LIGHTS, Settings, StartedWith, WEB, cycle,
};
use crate::ui::{
    ActiveDevice, InputDevice, MenuInput, MenuSound, Row, UiIcons, current_active_device, text_font,
};

/// Height of the panel (px): the tab with the most options (Control) fits.
const PANEL_HEIGHT: f32 = 600.0;
/// Room kept at the bottom of the screen for the legend (px).
const LEGEND_SPACE: f32 = 56.0;

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OptionsMenu>()
            .init_resource::<PadArts>()
            .add_systems(
                Update,
                (
                    use_options_menu
                        .run_if(|menu: Res<OptionsMenu>| menu.is_open)
                        .in_set(OptionsInput),
                    finish_pad_arts,
                    show_options_menu,
                )
                    .chain(),
            );
    }
}

/// The options read the keys before the screen under them (the start menu
/// or the pause menu), which skips the frame the options close in: the key
/// that closes them must not act there too. The screen opening them runs
/// after, so the key that opens them does not act on them either.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub struct OptionsInput;

/// A dónde vuelve el menú de opciones al cerrarse.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OptionsReturn {
    #[default]
    Welcome,
    Pause,
}

/// Pestañas del menú de opciones.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OptionsTab {
    #[default]
    Game,
    Control,
    Sound,
    Graphics,
}

const TABS: [OptionsTab; 4] = [
    OptionsTab::Game,
    OptionsTab::Control,
    OptionsTab::Sound,
    OptionsTab::Graphics,
];

impl OptionsTab {
    pub fn name(self, lang: Language) -> &'static str {
        match self {
            OptionsTab::Game => lang.tab_game(),
            OptionsTab::Control => lang.tab_controls(),
            OptionsTab::Sound => lang.tab_sound(),
            OptionsTab::Graphics => lang.tab_graphics(),
        }
    }

    pub fn next(self, step: i32) -> Self {
        cycle(&TABS, self, step)
    }

    /// The colour of each section: its tab, the edge of the panel and the
    /// option chosen in it.
    pub fn accent(self) -> Color {
        match self {
            OptionsTab::Game => Color::srgb(1.0, 0.72, 0.18),
            OptionsTab::Control => Color::srgb(0.22, 0.76, 1.0),
            OptionsTab::Sound => Color::srgb(1.0, 0.38, 0.66),
            OptionsTab::Graphics => Color::srgb(0.36, 0.9, 0.44),
        }
    }
}

/// Estado global del menú de opciones.
#[derive(Resource, Default, Clone, Debug)]
pub struct OptionsMenu {
    pub is_open: bool,
    pub tab: OptionsTab,
    /// Whether the row of sections is chosen, rather than an option of the
    /// section (`selected`): the arrows across then change the section.
    pub on_tabs: bool,
    pub selected: usize,
    /// The view of the controls' mapping, while it is open.
    pub mapping: Option<Mapping>,
    pub return_to: OptionsReturn,
    pub rescan_msg_until: f32,
}

impl OptionsMenu {
    pub fn open(&mut self, return_to: OptionsReturn) {
        self.is_open = true;
        self.on_tabs = false;
        self.selected = 0;
        self.mapping = None;
        self.return_to = return_to;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.mapping = None;
    }

    /// Moves round the menu: up and down (`step`) through the row of
    /// sections, the options of the section and its foot (Reset and Back,
    /// side by side: down to it, Back), round and round; across (`side`) on
    /// the row of sections changes the section, and confirming there goes
    /// down to its first option; across on the foot, its other button.
    /// `count` is how many options the section has, the foot's included.
    /// Whether it did anything.
    fn navigate(&mut self, step: i32, side: i32, confirm: bool, count: usize) -> bool {
        if self.on_tabs {
            if side != 0 {
                self.tab = self.tab.next(side);
                self.selected = 0;
                return true;
            }
            if step != 0 || confirm {
                self.on_tabs = false;
                self.selected = if step < 0 { count - 1 } else { 0 };
                return true;
            }
            return false;
        }
        let foot = count - FOOTER;
        let on_foot = self.selected >= foot;
        if step == 0 {
            if on_foot && side != 0 {
                self.selected = if self.selected == foot {
                    foot + 1
                } else {
                    foot
                };
                return true;
            }
            return false;
        }
        let next = if on_foot && step < 0 {
            foot as i32 - 1
        } else if on_foot {
            count as i32
        } else if self.selected as i32 + step == foot as i32 {
            // Onto the foot: Back.
            count as i32 - 1
        } else {
            self.selected as i32 + step
        };
        if next < 0 || next >= count as i32 {
            self.on_tabs = true;
        } else {
            self.selected = next as usize;
        }
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameItem {
    Language,
    Difficulty,
    RaceStart,
    /// How many red lights the start has.
    StartLights,
    /// Whether the results go to the middle once everybody is in.
    CentreResults,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ControlItem {
    /// The view where the keys and buttons are remapped.
    Mapping,
    StickTurnLimit,
    KeyboardTurnLimit,
    KeyboardTurnRate,
    KeyboardReturnRate,
    ShowLegend,
    RescanGamepads,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SoundItem {
    FxVolume,
    MusicVolume,
    MusicEnabled,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GraphicsItem {
    Fullscreen,
    Resolution,
    Fps,
    PipelinedRendering,
    FrameLatency,
}

fn game_items() -> Vec<GameItem> {
    vec![
        GameItem::Language,
        GameItem::Difficulty,
        GameItem::RaceStart,
        GameItem::StartLights,
        GameItem::CentreResults,
    ]
}

/// Room above a row that begins a group of its own (px).
const GROUP_GAP: f32 = 14.0;

/// Whether the row at `index` of `tab` begins a group of its own, with
/// room above: on Control, the steering after the mapping, and the legend
/// and the gamepads after the steering.
fn starts_group(tab: OptionsTab, index: usize) -> bool {
    tab == OptionsTab::Control
        && matches!(
            control_items().get(index),
            Some(ControlItem::StickTurnLimit | ControlItem::ShowLegend)
        )
}

/// The mapping of the controls, and how the stick and the keyboard steer.
fn control_items() -> Vec<ControlItem> {
    vec![
        ControlItem::Mapping,
        ControlItem::StickTurnLimit,
        ControlItem::KeyboardTurnLimit,
        ControlItem::KeyboardTurnRate,
        ControlItem::KeyboardReturnRate,
        ControlItem::ShowLegend,
        ControlItem::RescanGamepads,
    ]
}

fn sound_items() -> Vec<SoundItem> {
    vec![
        SoundItem::FxVolume,
        SoundItem::MusicVolume,
        SoundItem::MusicEnabled,
    ]
}

fn graphics_items() -> Vec<GraphicsItem> {
    if WEB {
        vec![GraphicsItem::Fullscreen, GraphicsItem::Resolution]
    } else {
        vec![
            GraphicsItem::Fullscreen,
            GraphicsItem::Resolution,
            GraphicsItem::Fps,
            GraphicsItem::PipelinedRendering,
            GraphicsItem::FrameLatency,
        ]
    }
}

/// How many options `tab` has, besides Reset and Back.
fn options_count(tab: OptionsTab) -> usize {
    match tab {
        OptionsTab::Game => game_items().len(),
        OptionsTab::Control => control_items().len(),
        OptionsTab::Sound => sound_items().len(),
        OptionsTab::Graphics => graphics_items().len(),
    }
}

/// At the foot of every section, side by side: Reset, which puts its
/// options back as they were at first, and Back. They are its last two
/// options.
const FOOTER: usize = 2;

/// How many options `tab` has, Reset and Back at its foot included.
fn tab_items_count(tab: OptionsTab) -> usize {
    options_count(tab) + FOOTER
}

/// The settings as they are at first.
fn first_settings() -> &'static Settings {
    static FIRST: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();
    FIRST.get_or_init(Settings::default)
}

/// How the stick and the keyboard steer, and whether the legend is shown:
/// the options of Control.
fn steering(debug: &DebugOptions) -> (TurnLimit, TurnLimit, SteerRate, ReturnRate, bool) {
    (
        debug.stick_limit,
        debug.keyboard_limit,
        debug.keyboard_rate,
        debug.keyboard_return,
        debug.show_legend,
    )
}

/// Whether the options of `tab` are as they are at first (the keys and
/// buttons aside: the mapping of Control resets those).
fn section_is_default(tab: OptionsTab, settings: &Settings, debug: &DebugOptions) -> bool {
    let first = first_settings();
    match tab {
        OptionsTab::Game => {
            (
                settings.language,
                settings.difficulty,
                settings.start,
                settings.start_lights,
                settings.centre_results,
            ) == (
                first.language,
                first.difficulty,
                first.start,
                first.start_lights,
                first.centre_results,
            )
        }
        OptionsTab::Control => steering(debug) == steering(&DebugOptions::default()),
        OptionsTab::Sound => {
            (
                settings.fx_volume,
                settings.music_volume,
                settings.music_enabled,
            ) == (first.fx_volume, first.music_volume, first.music_enabled)
        }
        OptionsTab::Graphics => {
            (
                settings.fullscreen,
                settings.resolution,
                settings.fps,
                settings.pipelined_rendering,
                settings.frame_latency,
            ) == (
                first.fullscreen,
                first.resolution,
                first.fps,
                first.pipelined_rendering,
                first.frame_latency,
            )
        }
    }
}

/// Puts the options of `tab` back as they are at first (see
/// [`section_is_default`]).
fn reset_section(tab: OptionsTab, settings: &mut Settings, debug: &mut DebugOptions) {
    let first = first_settings();
    match tab {
        OptionsTab::Game => {
            settings.language = first.language;
            settings.difficulty = first.difficulty;
            settings.start = first.start;
            settings.start_lights = first.start_lights;
            settings.centre_results = first.centre_results;
        }
        OptionsTab::Control => *debug = DebugOptions::default(),
        OptionsTab::Sound => {
            settings.fx_volume = first.fx_volume;
            settings.music_volume = first.music_volume;
            settings.music_enabled = first.music_enabled;
        }
        OptionsTab::Graphics => {
            settings.fullscreen = first.fullscreen;
            settings.resolution = first.resolution;
            settings.fps = first.fps;
            settings.pipelined_rendering = first.pipelined_rendering;
            settings.frame_latency = first.frame_latency;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn use_options_menu(
    mut input: MenuInput,
    time: Res<Time<Real>>,
    mut options_menu: ResMut<OptionsMenu>,
    mut settings: ResMut<Settings>,
    mut debug_options: ResMut<DebugOptions>,
    mut gamepads: ResMut<Gamepads>,
    device: Res<InputDevice>,
    active_controller: Option<Res<ActiveController>>,
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    let now = time.elapsed_secs();

    // The mapping of the controls, while it is open: it follows the device
    // in use.
    if let Some(mapping) = options_menu.mapping.as_mut() {
        if device.is_changed() && !mapping.capturing() {
            mapping.device =
                MapDevice::of(current_active_device(*device, active_controller.as_deref()));
        }
        let held = Held {
            keyboard: &input.keyboard,
            gamepads: input.gamepads.iter().collect(),
        };
        if use_mapping(
            mapping,
            &request,
            &held,
            &mut settings.controls,
            now,
            &mut sounds,
        ) {
            options_menu.mapping = None;
        }
        return;
    }

    // Salir / Volver con Esc o B
    if request.back {
        options_menu.close();
        sounds.write(MenuSound::Move);
        return;
    }

    // Change of section: Tab (Shift + Tab goes back), Q and E, or the
    // shoulder buttons and triggers.
    if request.tab != 0 {
        options_menu.tab = options_menu.tab.next(request.tab);
        options_menu.selected = 0;
        sounds.write(MenuSound::Move);
        return;
    }

    // Up and down through the row of sections and the options; across on
    // the row of sections, another section.
    let count = tab_items_count(options_menu.tab);
    let on_tabs = options_menu.on_tabs;
    if options_menu.navigate(request.step, request.side, request.confirm, count) {
        sounds.write(MenuSound::Move);
    }
    if on_tabs || request.step != 0 {
        return;
    }

    // Reset and Back, at the foot of every section.
    let foot = count - FOOTER;
    if options_menu.selected >= foot {
        if request.confirm && options_menu.selected == foot {
            reset_section(options_menu.tab, &mut settings, &mut debug_options);
            sounds.write(MenuSound::Confirm);
        } else if request.confirm {
            options_menu.close();
            sounds.write(MenuSound::Move);
        }
        return;
    }

    let change = if request.side != 0 {
        request.side
    } else {
        i32::from(request.confirm)
    };

    match options_menu.tab {
        OptionsTab::Game => {
            let items = game_items();
            let item = items[options_menu.selected.min(items.len() - 1)];
            match item {
                GameItem::Language if change != 0 => {
                    settings.language = cycle(&LANGUAGES, settings.language, change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::Difficulty if change != 0 => {
                    settings.difficulty = cycle(&DIFFICULTIES, settings.difficulty, change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::RaceStart if change != 0 => {
                    settings.start = cycle(&RACE_STARTS, settings.start, change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::StartLights if change != 0 => {
                    settings.start_lights = cycle(&START_LIGHTS, settings.start_lights, change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::CentreResults if change != 0 => {
                    settings.centre_results = !settings.centre_results;
                    sounds.write(MenuSound::Move);
                }
                _ => {}
            }
        }
        OptionsTab::Control => {
            let items = control_items();
            let item = items[options_menu.selected.min(items.len() - 1)];
            match item {
                ControlItem::Mapping if request.confirm => {
                    let device = current_active_device(*device, active_controller.as_deref());
                    options_menu.mapping = Some(Mapping::open(MapDevice::of(device)));
                    sounds.write(MenuSound::Confirm);
                }
                ControlItem::StickTurnLimit if change != 0 => {
                    debug_options.stick_limit = debug_options.stick_limit.next(change);
                    sounds.write(MenuSound::Move);
                }
                ControlItem::KeyboardTurnLimit if change != 0 => {
                    debug_options.keyboard_limit = debug_options.keyboard_limit.next(change);
                    sounds.write(MenuSound::Move);
                }
                ControlItem::KeyboardTurnRate if change != 0 => {
                    debug_options.keyboard_rate = debug_options.keyboard_rate.next(change);
                    sounds.write(MenuSound::Move);
                }
                ControlItem::KeyboardReturnRate if change != 0 => {
                    debug_options.keyboard_return = debug_options.keyboard_return.next(change);
                    sounds.write(MenuSound::Move);
                }
                ControlItem::ShowLegend if change != 0 => {
                    debug_options.show_legend = !debug_options.show_legend;
                    sounds.write(MenuSound::Move);
                }
                ControlItem::RescanGamepads if request.confirm => {
                    gamepads.search_again();
                    options_menu.rescan_msg_until = now + 3.0;
                    sounds.write(MenuSound::Confirm);
                }
                _ => {}
            }
        }
        OptionsTab::Sound => {
            let items = sound_items();
            let item = items[options_menu.selected.min(items.len() - 1)];
            match item {
                SoundItem::FxVolume if request.side != 0 => {
                    let next = (settings.fx_volume as i32 + request.side * 10).clamp(0, 100);
                    settings.fx_volume = next as u32;
                    sounds.write(MenuSound::Move);
                }
                SoundItem::MusicVolume if request.side != 0 => {
                    let next = (settings.music_volume as i32 + request.side * 10).clamp(0, 100);
                    settings.music_volume = next as u32;
                    sounds.write(MenuSound::Move);
                }
                SoundItem::MusicEnabled if change != 0 => {
                    settings.music_enabled = !settings.music_enabled;
                    sounds.write(MenuSound::Move);
                }
                _ => {}
            }
        }
        OptionsTab::Graphics => {
            let items = graphics_items();
            let item = items[options_menu.selected.min(items.len() - 1)];
            match item {
                GraphicsItem::Fullscreen if change != 0 => {
                    settings.fullscreen = !settings.fullscreen;
                    sounds.write(MenuSound::Move);
                }
                GraphicsItem::Resolution if change != 0 => {
                    if !WEB {
                        settings.resolution = cycle(RESOLUTIONS, settings.resolution, change);
                        sounds.write(MenuSound::Move);
                    }
                }
                GraphicsItem::Fps if change != 0 => {
                    settings.fps = cycle(&FPS_LIMITS, settings.fps, change);
                    sounds.write(MenuSound::Move);
                }
                GraphicsItem::PipelinedRendering if change != 0 => {
                    settings.pipelined_rendering = !settings.pipelined_rendering;
                    sounds.write(MenuSound::Move);
                }
                GraphicsItem::FrameLatency if change != 0 => {
                    settings.frame_latency =
                        cycle(&FRAME_LATENCIES, settings.frame_latency, change);
                    sounds.write(MenuSound::Move);
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
struct OptionsModal;

/// How far the text of each option is from the sides of its row (px): the
/// keys that change the section stand in line with it.
const ROW_INSET: f32 = 20.0;

/// How thick the edge of the chosen tab is (px). Every tab keeps room for
/// it, so the tabs do not change size when the focus comes and goes.
const TAB_EDGE: f32 = 2.0;

/// A section's tab, in its colour: the one shown (`active`) filled with
/// it, the others faintly. When the row of sections is chosen (`on_tabs`),
/// the one shown is edged in a lighter tone of its colour.
fn spawn_tab(
    tabs: &mut ChildSpawnerCommands,
    font: &UiFont,
    tab: OptionsTab,
    active: bool,
    on_tabs: bool,
    lang: Language,
) {
    let color = tab.accent();
    let fill = if active {
        color
    } else {
        color.with_alpha(0.16)
    };
    // The edge of the others is their fill, so they look as big.
    let edge = if active && on_tabs {
        color.mix(&Color::WHITE, 0.6)
    } else {
        fill
    };
    tabs.spawn((
        Node {
            padding: UiRect::axes(px(18.0 - TAB_EDGE), px(7.0 - TAB_EDGE)),
            border: UiRect::all(px(TAB_EDGE)),
            border_radius: BorderRadius::all(px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(edge),
    ))
    .with_child((
        Text::new(tab.name(lang)),
        text_font(font, 18.0),
        TextColor(if active {
            INK
        } else {
            color.mix(&Color::WHITE, 0.55)
        }),
    ));
}

/// The ink of an option chosen, on the colour of its section.
const INK: Color = Color::srgb(0.06, 0.05, 0.04);

/// A button at the foot of a section (see [`FOOTER`]): filled in the
/// section's colour when chosen; its words faint if it has nothing to do.
fn spawn_foot_button(
    foot: &mut ChildSpawnerCommands,
    font: &UiFont,
    row: &Row,
    accent: Color,
    chosen: bool,
) {
    let (fill, ink) = match (chosen, row.disabled) {
        (true, true) => row.chosen_colors(),
        (true, false) => (accent, INK),
        (false, true) => (
            Color::srgba(1.0, 1.0, 1.0, 0.04),
            Color::srgba(1.0, 1.0, 1.0, 0.4),
        ),
        (false, false) => (Color::srgba(1.0, 1.0, 1.0, 0.08), Color::WHITE),
    };
    foot.spawn((
        Node {
            min_width: px(150),
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(20), px(6)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(accent.with_alpha(0.5)),
    ))
    .with_child((Text::new(&row.label), text_font(font, 19.0), TextColor(ink)));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_options_menu(
    mut commands: Commands,
    options_menu: Res<OptionsMenu>,
    settings: Res<Settings>,
    debug_options: Res<DebugOptions>,
    device: Res<InputDevice>,
    active_controller: Option<Res<ActiveController>>,
    window: Query<&Window, With<PrimaryWindow>>,
    font: Res<UiFont>,
    icons: Res<UiIcons>,
    shown: Query<Entity, With<OptionsModal>>,
    time: Res<Time<Real>>,
    started: Option<Res<StartedWith>>,
    // The keys and buttons held, and the gamepads' names (for the mapping).
    (keyboard, gamepads, names): (
        Res<ButtonInput<KeyCode>>,
        Query<&Gamepad>,
        Query<&Name, With<Gamepad>>,
    ),
    // The mapping's gamepads, drawn.
    mut pad_arts: ResMut<PadArts>,
    mut last: Local<Option<String>>,
) {
    let started = started.as_deref();
    if !options_menu.is_open {
        for entity in &shown {
            commands.entity(entity).despawn();
        }
        *last = None;
        return;
    }

    let lang = settings.language;
    let window_inner = window.iter().next();
    let now = time.elapsed_secs();
    // Beside a setting that takes effect when the game starts again.
    let restart = match lang {
        Language::Spanish => "al reiniciar   ",
        Language::English => "on restart   ",
    };

    // Filas para la pestaña activa
    let rows: Vec<Row> = match options_menu.tab {
        OptionsTab::Game => game_items()
            .into_iter()
            .map(|item| match item {
                GameItem::Language => {
                    Row::with_value(lang.language(), format!("<  {}  >", lang.name()))
                }
                GameItem::Difficulty => Row::with_value(
                    match lang {
                        Language::Spanish => "Dificultad",
                        Language::English => "Difficulty",
                    },
                    format!("<  {}  >", settings.difficulty.name(lang)),
                ),
                GameItem::RaceStart => Row::with_value(
                    match lang {
                        Language::Spanish => "Salida",
                        Language::English => "Race start",
                    },
                    format!("<  {}  >", settings.start.name(lang)),
                ),
                // Faint when the race starts at the throttle, with no lights.
                GameItem::StartLights => Row {
                    disabled: !settings.start.at_lights(),
                    ..Row::with_value(
                        match lang {
                            Language::Spanish => "Luces rojas del semáforo",
                            Language::English => "Red start lights",
                        },
                        format!("<  {}  >", settings.start_lights),
                    )
                },
                GameItem::CentreResults => Row::with_value(
                    match lang {
                        Language::Spanish => "Centrar resultados al llegar todos",
                        Language::English => "Centre results when all are in",
                    },
                    if settings.centre_results {
                        "[X]"
                    } else {
                        "[ ]"
                    },
                ),
            })
            .collect(),
        OptionsTab::Control => control_items()
            .into_iter()
            .map(|item| match item {
                ControlItem::Mapping => Row::with_value(
                    match lang {
                        Language::Spanish => "Asignación",
                        Language::English => "Mapping",
                    },
                    match lang {
                        Language::Spanish => "Teclado y mando   >",
                        Language::English => "Keyboard and gamepad   >",
                    },
                ),
                ControlItem::StickTurnLimit => Row::with_value(
                    match lang {
                        Language::Spanish => "Stick: limitar giro con velocidad",
                        Language::English => "Stick: turn limit with speed",
                    },
                    format!("<  {}  >", debug_options.stick_limit.name(lang)),
                ),
                ControlItem::KeyboardTurnLimit => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: limitar giro con velocidad",
                        Language::English => "Keyboard: turn limit with speed",
                    },
                    format!("<  {}  >", debug_options.keyboard_limit.name(lang)),
                ),
                ControlItem::KeyboardTurnRate => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: rapidez de giro sostenido",
                        Language::English => "Keyboard: sustained turn rate",
                    },
                    format!("<  {}  >", debug_options.keyboard_rate.name(lang)),
                ),
                ControlItem::KeyboardReturnRate => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: rapidez al soltar giro",
                        Language::English => "Keyboard: return rate released",
                    },
                    format!("<  {}  >", debug_options.keyboard_return.name(lang)),
                ),
                ControlItem::ShowLegend => Row::with_value(
                    lang.show_legend(),
                    if debug_options.show_legend {
                        "[X]"
                    } else {
                        "[ ]"
                    },
                ),
                ControlItem::RescanGamepads => {
                    if now < options_menu.rescan_msg_until {
                        Row::with_value(
                            lang.rescan_gamepads(),
                            match lang {
                                Language::Spanish => "Mandos actualizados",
                                Language::English => "Controllers updated",
                            },
                        )
                    } else {
                        Row::new(lang.rescan_gamepads())
                    }
                }
            })
            .collect(),
        OptionsTab::Sound => sound_items()
            .into_iter()
            .map(|item| match item {
                SoundItem::FxVolume => Row::with_value(
                    lang.sound_fx_volume(),
                    format!("<  {}%  >", settings.fx_volume),
                ),
                SoundItem::MusicVolume => Row::with_value(
                    lang.sound_music_volume(),
                    format!("<  {}%  >", settings.music_volume),
                ),
                SoundItem::MusicEnabled => Row::with_value(
                    lang.sound_music(),
                    if settings.music_enabled {
                        format!("<  {}  >", lang.enabled())
                    } else {
                        format!("<  {}  >", lang.disabled())
                    },
                ),
            })
            .collect(),
        OptionsTab::Graphics => graphics_items()
            .into_iter()
            .map(|item| match item {
                GraphicsItem::Fullscreen => Row::with_value(
                    lang.fullscreen(),
                    if settings.fullscreen { "[X]" } else { "[ ]" },
                ),
                GraphicsItem::Resolution => {
                    let name = settings.resolution.name(window_inner, lang);
                    if WEB {
                        Row::disabled(lang.resolution(), name)
                    } else {
                        Row::with_value(lang.resolution(), format!("<  {name}  >"))
                    }
                }
                GraphicsItem::Fps => {
                    Row::with_value(lang.fps(), format!("<  {}  >", settings.fps.name(lang)))
                }
                // These two take effect when the game starts again.
                GraphicsItem::PipelinedRendering => Row::with_value(
                    match lang {
                        Language::Spanish => "Renderizado en paralelo",
                        Language::English => "Pipelined rendering",
                    },
                    format!(
                        "{}{}",
                        if started
                            .is_some_and(|s| s.pipelined_rendering != settings.pipelined_rendering)
                        {
                            restart
                        } else {
                            ""
                        },
                        if settings.pipelined_rendering {
                            "[X]"
                        } else {
                            "[ ]"
                        }
                    ),
                ),
                GraphicsItem::FrameLatency => Row::with_value(
                    match lang {
                        Language::Spanish => "Fotogramas en cola",
                        Language::English => "Frame latency",
                    },
                    format!(
                        "{}<  {}{}  >",
                        if started.is_some_and(|s| s.frame_latency != settings.frame_latency) {
                            restart
                        } else {
                            ""
                        },
                        settings.frame_latency,
                        if settings.frame_latency == DEFAULT_FRAME_LATENCY {
                            " (Bevy)"
                        } else {
                            ""
                        }
                    ),
                ),
            })
            .collect(),
    };

    // The foot of the section: Reset, faint when there is nothing to put
    // back, and Back.
    let footer = [
        Row {
            disabled: section_is_default(options_menu.tab, &settings, &debug_options),
            ..Row::new(lang.reset())
        },
        Row::new(lang.back()),
    ];

    let active_dev = current_active_device(*device, active_controller.as_deref());
    // The mapping view lights the keys and buttons held down.
    let held = Held {
        keyboard: &keyboard,
        gamepads: gamepads.iter().collect(),
    };
    let mut held_down: Vec<String> = keyboard
        .get_pressed()
        .map(|key| format!("{key:?}"))
        .chain(
            gamepads
                .iter()
                .flat_map(|pad| pad.get_pressed())
                .map(|button| format!("{button:?}")),
        )
        .collect();
    held_down.sort();
    // The gamepad of the mapping, drawn as the one in use: by its name.
    let pad_style = current_active_device(InputDevice::Gamepad, active_controller.as_deref());
    let pad_name = active_controller
        .as_deref()
        .and_then(|controller| controller.gamepad)
        .and_then(|pad| names.get(pad).ok())
        .map(|name| name.as_str().to_string());
    let art = PadArt::of(pad_style, pad_name.as_deref());
    let scale_factor = window_inner.map_or(1.0, Window::scale_factor);
    // It is drawn in the background from the Control tab on, to be ready
    // by the time it is shown (on the web the background is this same
    // thread: only once it is shown).
    let pad_shown = options_menu
        .mapping
        .as_ref()
        .is_some_and(|mapping| mapping.device == MapDevice::Gamepad);
    let pad_drawn = (pad_shown || (!WEB && options_menu.tab == OptionsTab::Control)).then(|| {
        pad_arts
            .get(
                art,
                pad_style == ActiveDevice::Switch,
                PAD_HEIGHT,
                scale_factor,
            )
            .is_some()
    });
    // Built again only when something shown changes (the window, for
    // example, changes with every move of the mouse; its scale factor
    // changes the gamepad's pixels).
    let wanted = format!(
        "{:?} {art:?} {scale_factor} {pad_drawn:?} {:?} {:?} {held_down:?} {:?} {} {} {:?} {:?} {:?} {}",
        options_menu.mapping,
        settings.controls,
        current_active_device(InputDevice::Gamepad, active_controller.as_deref()),
        options_menu.tab,
        options_menu.on_tabs,
        options_menu.selected,
        lang,
        active_dev,
        debug_options.show_legend,
        rows.iter()
            .chain(&footer)
            .map(|row| format!("{}={:?}{}", row.label, row.value, row.disabled))
            .collect::<Vec<_>>()
            .join("|"),
    );
    if last.as_ref() == Some(&wanted) && !shown.is_empty() {
        return;
    }
    *last = Some(wanted);
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    let accent = options_menu.tab.accent();
    // The keys that change the section go beside the tabs instead.
    let prompts = menu_prompts(
        active_dev,
        &[
            (Keys::Confirm, lang.hint_accept()),
            (Keys::Back, lang.back()),
        ],
    );
    let [previous, next] = section_badges(active_dev);
    commands
        .spawn((
            OptionsModal,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                // Centred, above the legend. The panel is as tall on every
                // tab, so the tabs stay where they are.
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(px(LEGEND_SPACE)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.78)),
            ZIndex(100),
        ))
        .with_children(|modal| {
            // Panel de Opciones grande
            modal
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        width: px(780),
                        max_width: percent(94),
                        height: px(PANEL_HEIGHT),
                        max_height: percent(100),
                        overflow: Overflow::clip(),
                        padding: UiRect::axes(px(24), px(20)),
                        border_radius: BorderRadius::all(px(18)),
                        border: UiRect::all(px(1)),
                        row_gap: px(12),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.94)),
                    // Edged in the colour of the section.
                    BorderColor::all(accent.with_alpha(0.55)),
                ))
                .with_children(|panel| {
                    // The mapping of the controls, in place of the sections.
                    if let Some(mapping) = &options_menu.mapping {
                        let drawn = pad_shown
                            .then(|| {
                                pad_arts.get(
                                    art,
                                    pad_style == ActiveDevice::Switch,
                                    PAD_HEIGHT,
                                    scale_factor,
                                )
                            })
                            .flatten();
                        let look = MappingLook {
                            font: &font,
                            icons: &icons,
                            lang,
                            using: active_dev,
                            pad: pad_style,
                            art,
                            drawn,
                            letters: pad_style == ActiveDevice::Switch,
                            controls: &settings.controls,
                            held: &held,
                        };
                        spawn_mapping(panel, mapping, &look);
                        return;
                    }
                    // Título
                    panel.spawn((
                        Text::new(lang.options().to_uppercase()),
                        text_font(&font, 36.0),
                        TextShadow {
                            offset: Vec2::splat(2.5),
                            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                        },
                        Node {
                            align_self: AlignSelf::Center,
                            margin: UiRect::bottom(px(4)),
                            ..default()
                        },
                    ));

                    // Barra de pestañas: the keys that change the section
                    // at its ends, in line with the text of the options,
                    // and the sections in the middle.
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(px(ROW_INSET)),
                            margin: UiRect::bottom(px(6)),
                            ..default()
                        },))
                        .with_children(|tab_bar| {
                            // The key for the section before, on the left.
                            spawn_lone_badge(tab_bar, &font, &icons, active_dev, &previous);
                            tab_bar
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(10),
                                    ..default()
                                })
                                .with_children(|tabs| {
                                    for tab in TABS {
                                        spawn_tab(
                                            tabs,
                                            &font,
                                            tab,
                                            tab == options_menu.tab,
                                            options_menu.on_tabs,
                                            lang,
                                        );
                                    }
                                });
                            // The key for the section after, on the right.
                            spawn_lone_badge(tab_bar, &font, &icons, active_dev, &next);
                        });
                    // A line in the colour of the section under the tabs.
                    panel.spawn((
                        Node {
                            height: px(2),
                            min_height: px(2),
                            flex_shrink: 0.0,
                            margin: UiRect::bottom(px(4)),
                            ..default()
                        },
                        BackgroundColor(accent.with_alpha(0.7)),
                    ));

                    // Lista de opciones de la pestaña. It fills the panel, and
                    // Reset and Back stand at its foot on every tab.
                    let (size, padding, gap) = (20.0, 5.0, 3.0);
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            flex_grow: 1.0,
                            row_gap: px(gap),
                            ..default()
                        },))
                        .with_children(|items_container| {
                            for (index, row) in rows.iter().enumerate() {
                                let chosen =
                                    !options_menu.on_tabs && index == options_menu.selected;
                                // Chosen, in the colour of the section.
                                let (chosen_background, chosen_text) = if row.disabled || row.danger
                                {
                                    row.chosen_colors()
                                } else {
                                    (accent, INK)
                                };
                                let color = TextColor(if chosen {
                                    chosen_text
                                } else if row.disabled {
                                    Color::srgba(1.0, 1.0, 1.0, 0.4)
                                } else {
                                    Color::WHITE
                                });

                                items_container
                                    .spawn((
                                        Node {
                                            justify_content: if row.value.is_some() {
                                                JustifyContent::SpaceBetween
                                            } else {
                                                JustifyContent::Center
                                            },
                                            align_items: AlignItems::Center,
                                            column_gap: px(30),
                                            padding: UiRect::axes(px(ROW_INSET), px(padding)),
                                            margin: if starts_group(options_menu.tab, index) {
                                                UiRect::top(px(GROUP_GAP))
                                            } else {
                                                UiRect::ZERO
                                            },
                                            border_radius: BorderRadius::all(px(8)),
                                            ..default()
                                        },
                                        BackgroundColor(if chosen {
                                            chosen_background
                                        } else {
                                            Color::NONE
                                        }),
                                    ))
                                    .with_children(|line| {
                                        line.spawn((
                                            Text::new(&row.label),
                                            text_font(&font, size),
                                            color,
                                        ));
                                        if let Some(value) = &row.value {
                                            line.spawn((
                                                Text::new(value),
                                                text_font(&font, size),
                                                color,
                                            ));
                                        }
                                    });
                            }
                            // The foot, side by side.
                            items_container
                                .spawn(Node {
                                    justify_content: JustifyContent::Center,
                                    column_gap: px(16),
                                    margin: UiRect::top(Val::Auto),
                                    ..default()
                                })
                                .with_children(|foot| {
                                    for (place, row) in footer.iter().enumerate() {
                                        let chosen = !options_menu.on_tabs
                                            && options_menu.selected == rows.len() + place;
                                        spawn_foot_button(foot, &font, row, accent, chosen);
                                    }
                                });
                        });
                });

            // Under the panel, outside it, on the controls' tab: the
            // shortcuts for the sound and fullscreen, on the device in use.
            // The room is kept on every tab, so the panel does not move.
            modal
                .spawn((
                    Node {
                        margin: UiRect::top(px(14)),
                        ..default()
                    },
                    if options_menu.tab == OptionsTab::Control {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ))
                .with_children(|note| {
                    let prompts = shortcut_prompts(active_dev, lang);
                    spawn_prompts(note, &font, &icons, active_dev, &prompts);
                });

            // Leyenda de controles al pie
            spawn_legend(modal, &font, &icons, active_dev, &prompts);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_control_the_steering_and_the_legend_stand_apart() {
        let items = control_items();
        let starts: Vec<ControlItem> = (0..items.len())
            .filter(|&index| starts_group(OptionsTab::Control, index))
            .map(|index| items[index])
            .collect();
        assert_eq!(
            starts,
            [ControlItem::StickTurnLimit, ControlItem::ShowLegend]
        );
        assert!((0..8).all(|index| !starts_group(OptionsTab::Game, index)));
    }

    #[test]
    fn the_arrows_reach_the_row_of_sections_and_change_section_there() {
        let mut menu = OptionsMenu::default();
        menu.open(OptionsReturn::Welcome);
        let count = tab_items_count(menu.tab);
        // Up from the first option: the row of sections.
        assert!(menu.navigate(-1, 0, false, count));
        assert!(menu.on_tabs);
        // Across there: the next section, and the one before.
        assert!(menu.navigate(0, 1, false, count));
        assert_eq!(menu.tab, OptionsTab::Control);
        assert!(menu.navigate(0, -1, false, count));
        assert!(menu.navigate(0, -1, false, count));
        assert_eq!(menu.tab, OptionsTab::Graphics);
        assert!(menu.on_tabs);
        // Down: its first option; up from the row: its last one.
        let count = tab_items_count(menu.tab);
        assert!(menu.navigate(1, 0, false, count));
        assert!(!menu.on_tabs);
        assert_eq!(menu.selected, 0);
        menu.on_tabs = true;
        assert!(menu.navigate(-1, 0, false, count));
        assert_eq!(menu.selected, count - 1);
        // Down from the last option: round to the row of sections.
        assert!(menu.navigate(1, 0, false, count));
        assert!(menu.on_tabs);
        // Confirming on the row goes down into the section.
        assert!(menu.navigate(0, 0, true, count));
        assert!(!menu.on_tabs && menu.selected == 0);
        // Across on an option is left to the option.
        assert!(!menu.navigate(0, 1, false, count));
        assert_eq!(menu.tab, OptionsTab::Graphics);
    }

    use crate::settings::{Controls, Difficulty};

    #[test]
    fn reset_and_back_stand_side_by_side_at_the_foot() {
        let mut menu = OptionsMenu::default();
        menu.open(OptionsReturn::Welcome);
        menu.tab = OptionsTab::Sound;
        let count = tab_items_count(menu.tab);
        assert_eq!(count, sound_items().len() + FOOTER);
        // Down from the last option: Back.
        menu.selected = count - FOOTER - 1;
        assert!(menu.navigate(1, 0, false, count));
        assert_eq!(menu.selected, count - 1);
        // Across: Reset, and Back again.
        assert!(menu.navigate(0, -1, false, count));
        assert_eq!(menu.selected, count - 2);
        assert!(menu.navigate(0, 1, false, count));
        assert_eq!(menu.selected, count - 1);
        // Up from either: the last option; down: the row of sections.
        menu.selected = count - 2;
        assert!(menu.navigate(-1, 0, false, count));
        assert_eq!(menu.selected, count - FOOTER - 1);
        menu.selected = count - 2;
        assert!(menu.navigate(1, 0, false, count));
        assert!(menu.on_tabs);
    }

    #[test]
    fn reset_puts_back_the_options_of_its_section_only() {
        let mut settings = Settings::default();
        let mut debug = DebugOptions::default();
        for tab in TABS {
            assert!(section_is_default(tab, &settings, &debug), "{tab:?}");
        }
        settings.difficulty = Difficulty::Hard;
        settings.music_volume = 10;
        settings.fps = FPS_LIMITS[FPS_LIMITS.len() - 1];
        settings.frame_latency = 3;
        debug.show_legend = true;
        settings.controls.remap(KeyCode::KeyW, &[KeyCode::KeyI]);
        for tab in TABS {
            assert!(!section_is_default(tab, &settings, &debug), "{tab:?}");
        }
        reset_section(OptionsTab::Sound, &mut settings, &mut debug);
        assert_eq!(settings.music_volume, Settings::default().music_volume);
        assert!(section_is_default(OptionsTab::Sound, &settings, &debug));
        assert_eq!(settings.difficulty, Difficulty::Hard);
        reset_section(OptionsTab::Control, &mut settings, &mut debug);
        assert!(!debug.show_legend);
        // The keys and buttons are reset in the mapping, not here.
        assert_ne!(settings.controls, Controls::default());
        reset_section(OptionsTab::Game, &mut settings, &mut debug);
        reset_section(OptionsTab::Graphics, &mut settings, &mut debug);
        for tab in TABS {
            assert!(section_is_default(tab, &settings, &debug), "{tab:?}");
        }
    }

    #[test]
    fn each_section_has_its_own_colour() {
        let colors: Vec<Color> = TABS.iter().map(|tab| tab.accent()).collect();
        for (i, a) in colors.iter().enumerate() {
            for b in &colors[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
