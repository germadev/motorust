//! Menú de opciones unificado con pestañas: Juego, Control, Sonido y Gráficos.
//! Accesible tanto desde la pantalla de bienvenida como desde el menú de pausa.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gamepads::{ActiveController, Gamepads};
use crate::hud::UiFont;
use crate::legend::{Keys, menu_prompts, spawn_legend};
use crate::locale::{LANGUAGES, Language};
use crate::pause::DebugOptions;
use crate::settings::{
    ACTIONS, Action, Controls, FPS_LIMITS, FULLSCREEN_KEYS, RESOLUTIONS, Settings, WEB, cycle,
};
use crate::ui::{
    InputDevice, MenuInput, MenuSound, Row, UiIcons, current_active_device, text_font,
};

/// Tiempo para pulsar la tecla o el botón nuevo antes de que se cancele (s).
const CAPTURE_TIME: f32 = 8.0;
/// Height of the panel (px): the tab with the most options (Control) fits.
const PANEL_HEIGHT: f32 = 600.0;
/// Room kept at the bottom of the screen for the legend (px).
const LEGEND_SPACE: f32 = 56.0;

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OptionsMenu>().add_systems(
            Update,
            (
                use_options_menu
                    .run_if(|menu: Res<OptionsMenu>| menu.is_open)
                    .in_set(OptionsInput),
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
}

/// Estado global del menú de opciones.
#[derive(Resource, Default, Clone, Debug)]
pub struct OptionsMenu {
    pub is_open: bool,
    pub tab: OptionsTab,
    pub selected: usize,
    pub capture: Option<Capture>,
    pub return_to: OptionsReturn,
    pub rescan_msg_until: f32,
}

impl OptionsMenu {
    pub fn open(&mut self, return_to: OptionsReturn) {
        self.is_open = true;
        self.selected = 0;
        self.capture = None;
        self.return_to = return_to;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.capture = None;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Capture {
    pub action: Action,
    pub since: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameItem {
    Language,
    StickTurnLimit,
    KeyboardTurnLimit,
    KeyboardTurnRate,
    KeyboardReturnRate,
    Back,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ControlItem {
    Bind(Action),
    ResetControls,
    ShowLegend,
    RescanGamepads,
    Back,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SoundItem {
    FxVolume,
    MusicVolume,
    MusicEnabled,
    Back,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GraphicsItem {
    Fullscreen,
    Resolution,
    Fps,
    Back,
}

fn game_items() -> Vec<GameItem> {
    vec![
        GameItem::Language,
        GameItem::StickTurnLimit,
        GameItem::KeyboardTurnLimit,
        GameItem::KeyboardTurnRate,
        GameItem::KeyboardReturnRate,
        GameItem::Back,
    ]
}

fn control_items() -> Vec<ControlItem> {
    ACTIONS
        .iter()
        .map(|&action| ControlItem::Bind(action))
        .chain([
            ControlItem::ResetControls,
            ControlItem::ShowLegend,
            ControlItem::RescanGamepads,
            ControlItem::Back,
        ])
        .collect()
}

fn sound_items() -> Vec<SoundItem> {
    vec![
        SoundItem::FxVolume,
        SoundItem::MusicVolume,
        SoundItem::MusicEnabled,
        SoundItem::Back,
    ]
}

fn graphics_items() -> Vec<GraphicsItem> {
    if WEB {
        vec![
            GraphicsItem::Fullscreen,
            GraphicsItem::Resolution,
            GraphicsItem::Back,
        ]
    } else {
        vec![
            GraphicsItem::Fullscreen,
            GraphicsItem::Resolution,
            GraphicsItem::Fps,
            GraphicsItem::Back,
        ]
    }
}

fn tab_items_count(tab: OptionsTab) -> usize {
    match tab {
        OptionsTab::Game => game_items().len(),
        OptionsTab::Control => control_items().len(),
        OptionsTab::Sound => sound_items().len(),
        OptionsTab::Graphics => graphics_items().len(),
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
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    let now = time.elapsed_secs();

    // Captura de nueva tecla o botón
    if let Some(capture) = options_menu.capture {
        let key = input
            .keyboard
            .get_just_pressed()
            .copied()
            .find(|&key| key != KeyCode::KeyM);
        let button = input
            .gamepads
            .iter()
            .find_map(|gamepad| gamepad.get_just_pressed().next().copied());

        if key == Some(KeyCode::Escape) || now - capture.since > CAPTURE_TIME {
            options_menu.capture = None;
            sounds.write(MenuSound::Move);
        } else if let Some(key) = key {
            settings.controls.bind_key(capture.action, key);
            options_menu.capture = None;
            sounds.write(MenuSound::Confirm);
        } else if let Some(button) = button {
            settings.controls.bind_button(capture.action, button);
            options_menu.capture = None;
            sounds.write(MenuSound::Confirm);
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

    // Navegación vertical
    let count = tab_items_count(options_menu.tab);
    if request.step != 0 {
        options_menu.selected =
            (options_menu.selected as i32 + request.step).rem_euclid(count as i32) as usize;
        sounds.write(MenuSound::Move);
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
                GameItem::StickTurnLimit if change != 0 => {
                    debug_options.stick_limit = debug_options.stick_limit.next(change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::KeyboardTurnLimit if change != 0 => {
                    debug_options.keyboard_limit = debug_options.keyboard_limit.next(change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::KeyboardTurnRate if change != 0 => {
                    debug_options.keyboard_rate = debug_options.keyboard_rate.next(change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::KeyboardReturnRate if change != 0 => {
                    debug_options.keyboard_return = debug_options.keyboard_return.next(change);
                    sounds.write(MenuSound::Move);
                }
                GameItem::Back if request.confirm => {
                    options_menu.close();
                    sounds.write(MenuSound::Move);
                }
                _ => {}
            }
        }
        OptionsTab::Control => {
            let items = control_items();
            let item = items[options_menu.selected.min(items.len() - 1)];
            match item {
                ControlItem::Bind(action) if request.confirm => {
                    options_menu.capture = Some(Capture { action, since: now });
                    sounds.write(MenuSound::Confirm);
                }
                ControlItem::ResetControls if request.confirm => {
                    settings.controls = Controls::default();
                    sounds.write(MenuSound::Confirm);
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
                ControlItem::Back if request.confirm => {
                    options_menu.close();
                    sounds.write(MenuSound::Move);
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
                SoundItem::Back if request.confirm => {
                    options_menu.close();
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
                GraphicsItem::Back if request.confirm => {
                    options_menu.close();
                    sounds.write(MenuSound::Move);
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
struct OptionsModal;

#[allow(clippy::too_many_arguments)]
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
    mut last: Local<Option<String>>,
) {
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

    // Filas para la pestaña activa
    let rows: Vec<Row> = match options_menu.tab {
        OptionsTab::Game => game_items()
            .into_iter()
            .map(|item| match item {
                GameItem::Language => {
                    Row::with_value(lang.language(), format!("<  {}  >", lang.name()))
                }
                GameItem::StickTurnLimit => Row::with_value(
                    match lang {
                        Language::Spanish => "Stick: limitar giro con velocidad",
                        Language::English => "Stick: turn limit with speed",
                    },
                    format!("<  {}  >", debug_options.stick_limit.name(lang)),
                ),
                GameItem::KeyboardTurnLimit => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: limitar giro con velocidad",
                        Language::English => "Keyboard: turn limit with speed",
                    },
                    format!("<  {}  >", debug_options.keyboard_limit.name(lang)),
                ),
                GameItem::KeyboardTurnRate => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: rapidez de giro sostenido",
                        Language::English => "Keyboard: sustained turn rate",
                    },
                    format!("<  {}  >", debug_options.keyboard_rate.name(lang)),
                ),
                GameItem::KeyboardReturnRate => Row::with_value(
                    match lang {
                        Language::Spanish => "Teclado: rapidez al soltar giro",
                        Language::English => "Keyboard: return rate released",
                    },
                    format!("<  {}  >", debug_options.keyboard_return.name(lang)),
                ),
                GameItem::Back => Row::new(lang.back()),
            })
            .collect(),
        OptionsTab::Control => control_items()
            .into_iter()
            .map(|item| match item {
                ControlItem::Bind(action) => Row::with_value(
                    action.name(lang),
                    match options_menu.capture {
                        Some(capture) if capture.action == action => match lang {
                            Language::Spanish => "Pulsa una tecla o botón".into(),
                            Language::English => "Press key or button".into(),
                        },
                        _ => settings.controls.describe(action, lang),
                    },
                ),
                ControlItem::ResetControls => Row::new(lang.reset_controls()),
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
                ControlItem::Back => Row::new(lang.back()),
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
                SoundItem::Back => Row::new(lang.back()),
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
                GraphicsItem::Back => Row::new(lang.back()),
            })
            .collect(),
    };

    let active_dev = current_active_device(*device, active_controller.as_deref());
    // Built again only when something shown changes (the window, for
    // example, changes with every move of the mouse).
    let wanted = format!(
        "{:?} {} {:?} {:?} {:?} {}",
        options_menu.tab,
        options_menu.selected,
        lang,
        active_dev,
        debug_options.show_legend,
        rows.iter()
            .map(|row| format!("{}={:?}", row.label, row.value))
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

    let prompts = menu_prompts(
        active_dev,
        &[
            (Keys::Confirm, lang.hint_accept()),
            (Keys::Back, lang.back()),
            (Keys::Tabs, lang.hint_section()),
        ],
    );
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
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                ))
                .with_children(|panel| {
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

                    // Barra de pestañas
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: px(10),
                            margin: UiRect::bottom(px(6)),
                            ..default()
                        },))
                        .with_children(|tab_bar| {
                            for tab in TABS {
                                let is_active = tab == options_menu.tab;
                                tab_bar
                                    .spawn((
                                        Node {
                                            padding: UiRect::axes(px(18), px(7)),
                                            border_radius: BorderRadius::all(px(10)),
                                            border: UiRect::all(px(1)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(if is_active {
                                            Color::srgba(1.0, 0.8, 0.2, 0.95)
                                        } else {
                                            Color::srgba(1.0, 1.0, 1.0, 0.07)
                                        }),
                                        BorderColor::all(if is_active {
                                            Color::srgba(1.0, 0.85, 0.3, 1.0)
                                        } else {
                                            Color::srgba(1.0, 1.0, 1.0, 0.12)
                                        }),
                                    ))
                                    .with_child((
                                        Text::new(tab.name(lang)),
                                        text_font(&font, 18.0),
                                        TextColor(if is_active {
                                            Color::srgb(0.08, 0.06, 0.02)
                                        } else {
                                            Color::WHITE
                                        }),
                                    ));
                            }
                        });

                    // Lista de opciones de la pestaña
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: px(3),
                            ..default()
                        },))
                        .with_children(|items_container| {
                            for (index, row) in rows.iter().enumerate() {
                                let chosen = index == options_menu.selected;
                                let (chosen_background, chosen_text) = row.chosen_colors();
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
                                            padding: UiRect::axes(px(20), px(5)),
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
                                            text_font(&font, 20.0),
                                            color,
                                        ));
                                        if let Some(value) = &row.value {
                                            line.spawn((
                                                Text::new(value),
                                                text_font(&font, 20.0),
                                                color,
                                            ));
                                        }
                                    });
                            }
                        });

                    // Nota al pie en la pestaña de controles
                    if options_menu.tab == OptionsTab::Control {
                        let note = match lang {
                            Language::Spanish => format!(
                                "Stick izq: girar   ·   Stick der: mirar   ·   M: sonido   ·   {FULLSCREEN_KEYS}: pantalla completa"
                            ),
                            Language::English => format!(
                                "Left stick: steer   ·   Right stick: look   ·   M: sound   ·   {FULLSCREEN_KEYS}: fullscreen"
                            ),
                        };
                        panel.spawn((
                            Text::new(note),
                            text_font(&font, 14.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.65)),
                            TextShadow::default(),
                            Node {
                                align_self: AlignSelf::Center,
                                margin: UiRect::top(px(4)),
                                ..default()
                            },
                        ));
                    }
                });

            // Leyenda de controles al pie
            spawn_legend(modal, &font, &icons, active_dev, &prompts);
        });
}
