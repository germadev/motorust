//! Menú de pausa: Esc o Start paran la carrera y abren un menú para seguir,
//! volver a la pantalla de inicio, salir del juego o cambiar opciones de
//! depuración.

use bevy::prelude::*;

use crate::gamepads::ActiveController;
use crate::hud::UiFont;
use crate::legend::{Keys, menu_prompts, spawn_legend};
use crate::locale::Language;
use crate::menu::GameState;
use crate::options::{OptionsInput, OptionsMenu, OptionsReturn};
use crate::settings::{Action as Control, Settings, WEB, cycle};
use crate::ui::{
    InputDevice, MenuInput, MenuSound, Row, UiIcons, current_active_device, spawn_panel,
    spawn_title,
};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pause>()
            .init_resource::<DebugOptions>()
            .init_resource::<OptionsMenu>()
            .add_systems(OnExit(GameState::Race), close)
            .add_systems(
                Update,
                (use_pause_menu.after(OptionsInput), show_pause_menu)
                    .chain()
                    .run_if(in_state(GameState::Race)),
            );
    }
}

/// Opciones de depuración, para probar variantes del manejo.
#[derive(Resource)]
pub struct DebugOptions {
    /// Con el stick, cuánto limitar el giro según la velocidad (de entrada,
    /// medio).
    pub stick_limit: TurnLimit,
    /// Con el teclado o la cruceta, lo mismo (de entrada, medio).
    pub keyboard_limit: TurnLimit,
    /// Con el teclado o la cruceta, cuánto gira al dejar pulsada la tecla
    /// (de entrada, medio).
    pub keyboard_rate: SteerRate,
    /// Con el teclado o la cruceta, la rapidez con la que recupera al soltar
    /// la tecla de girar (de entrada, media).
    pub keyboard_return: ReturnRate,
    /// Si se ve en la carrera la leyenda con los controles (de entrada, no).
    pub show_legend: bool,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            stick_limit: TurnLimit::Medium,
            keyboard_limit: TurnLimit::Medium,
            keyboard_rate: SteerRate::Medium,
            keyboard_return: ReturnRate::Medium,
            show_legend: false,
        }
    }
}

/// Cuánto gira el teclado o la cruceta al dejar pulsada la tecla: lento,
/// medio o alto (lo de siempre).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SteerRate {
    Slow,
    Medium,
    Fast,
}

const STEER_RATES: [SteerRate; 3] = [SteerRate::Slow, SteerRate::Medium, SteerRate::Fast];

impl SteerRate {
    /// Factor que multiplica la rapidez con la que gira el teclado.
    pub fn factor(self) -> f32 {
        match self {
            SteerRate::Slow => 0.5,
            SteerRate::Medium => 0.75,
            SteerRate::Fast => 1.0,
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => match self {
                SteerRate::Slow => "lento",
                SteerRate::Medium => "medio",
                SteerRate::Fast => "alto",
            },
            Language::English => match self {
                SteerRate::Slow => "slow",
                SteerRate::Medium => "medium",
                SteerRate::Fast => "fast",
            },
        }
    }

    /// El nivel siguiente (`step` 1) o el anterior (-1), dando la vuelta.
    pub fn next(self, step: i32) -> Self {
        cycle(&STEER_RATES, self, step)
    }
}

/// Rapidez con la que recupera el teclado o la cruceta al soltar la tecla de
/// girar: lenta, media o rápida (lo de antes).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReturnRate {
    Slow,
    Medium,
    Fast,
}

const RETURN_RATES: [ReturnRate; 3] = [ReturnRate::Slow, ReturnRate::Medium, ReturnRate::Fast];

impl ReturnRate {
    /// Factor que multiplica la rapidez de recuperación al soltar. Medium,
    /// the default, is what slow used to be (straight again about 0.9 s
    /// after letting go); slow is two thirds of it, as before (1.1 s), and
    /// fast is the original (0.6 s).
    pub fn factor(self) -> f32 {
        match self {
            ReturnRate::Slow => 1.0 / 3.0,
            ReturnRate::Medium => 0.5,
            ReturnRate::Fast => 1.0,
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => match self {
                ReturnRate::Slow => "lenta",
                ReturnRate::Medium => "media",
                ReturnRate::Fast => "rápida",
            },
            Language::English => match self {
                ReturnRate::Slow => "slow",
                ReturnRate::Medium => "medium",
                ReturnRate::Fast => "fast",
            },
        }
    }

    /// El nivel siguiente (`step` 1) o el anterior (-1), dando la vuelta.
    pub fn next(self, step: i32) -> Self {
        cycle(&RETURN_RATES, self, step)
    }
}

/// Cuánto se limita el giro con la velocidad: nada, o una parte del límite
/// (alto, todo: ver `DIGITAL_STEER_FAST` en `bike`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnLimit {
    Off,
    Low,
    Medium,
    High,
}

const TURN_LIMITS: [TurnLimit; 4] = [
    TurnLimit::Off,
    TurnLimit::Low,
    TurnLimit::Medium,
    TurnLimit::High,
];

impl TurnLimit {
    /// Qué parte del límite se aplica.
    pub fn strength(self) -> f32 {
        match self {
            TurnLimit::Off => 0.0,
            TurnLimit::Low => 1.0 / 3.0,
            TurnLimit::Medium => 2.0 / 3.0,
            TurnLimit::High => 1.0,
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => match self {
                TurnLimit::Off => "no",
                TurnLimit::Low => "bajo",
                TurnLimit::Medium => "medio",
                TurnLimit::High => "alto",
            },
            Language::English => match self {
                TurnLimit::Off => "off",
                TurnLimit::Low => "low",
                TurnLimit::Medium => "medium",
                TurnLimit::High => "high",
            },
        }
    }

    /// El nivel siguiente (`step` 1) o el anterior (-1), dando la vuelta:
    /// después del más alto, sin límite.
    pub fn next(self, step: i32) -> Self {
        cycle(&TURN_LIMITS, self, step)
    }
}

/// Si la carrera está en pausa, qué página del menú se ve y qué opción está
/// elegida.
#[derive(Resource, Default)]
pub struct Pause {
    page: Option<Page>,
    selected: usize,
}

impl Pause {
    pub fn is_open(&self) -> bool {
        self.page.is_some()
    }
}

/// Para los `run_if` de lo que no debe pasar con la carrera en pausa.
pub fn running(pause: Res<Pause>) -> bool {
    !pause.is_open()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Main,
    QuitConfirm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Resume,
    OpenOptions,
    MainMenu,
    OpenQuitConfirm,
    CancelQuit,
    ConfirmQuit,
}

/// Las opciones de cada página: su texto y lo que hacen.
pub fn items(page: Page, lang: Language) -> Vec<(String, Action)> {
    match page {
        Page::Main => {
            let mut items = vec![
                (
                    match lang {
                        Language::Spanish => "Continuar".into(),
                        Language::English => "Resume".into(),
                    },
                    Action::Resume,
                ),
                (
                    match lang {
                        Language::Spanish => "Opciones".into(),
                        Language::English => "Options".into(),
                    },
                    Action::OpenOptions,
                ),
                (
                    match lang {
                        Language::Spanish => "Volver al menú principal".into(),
                        Language::English => "Return to main menu".into(),
                    },
                    Action::MainMenu,
                ),
            ];
            // En la web se sale cerrando la pestaña.
            if !WEB {
                items.push((lang.exit_game().into(), Action::OpenQuitConfirm));
            }
            items
        }
        Page::QuitConfirm => vec![
            (
                match lang {
                    Language::Spanish => "Cancelar".into(),
                    Language::English => "Cancel".into(),
                },
                Action::CancelQuit,
            ),
            (lang.exit_game().into(), Action::ConfirmQuit),
        ],
    }
}

/// Abre y cierra la pausa, se mueve por las opciones y las aplica.
#[allow(clippy::too_many_arguments)]
fn use_pause_menu(
    mut input: MenuInput,
    settings: Res<Settings>,
    mut pause: ResMut<Pause>,
    mut options_menu: Option<ResMut<OptionsMenu>>,
    mut time: ResMut<Time<Virtual>>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut sounds: MessageWriter<MenuSound>,
) {
    // While the options are open, and in the frame they close (the key that
    // closed them must not also act here).
    if options_menu
        .as_ref()
        .is_some_and(|m| m.is_open || m.is_changed())
    {
        return;
    }

    let request = input.read();
    // La tecla o el botón de pausa (Esc o Start, si no se han cambiado).
    let toggle =
        settings
            .controls
            .just_pressed(Control::Pause, &input.keyboard, input.gamepads.iter());
    let Some(page) = pause.page else {
        if toggle {
            *pause = Pause {
                page: Some(Page::Main),
                selected: 0,
            };
            sounds.write(MenuSound::Confirm);
            time.pause();
        }
        return;
    };
    let items = items(page, settings.language);
    if request.step != 0 {
        pause.selected =
            (pause.selected as i32 + request.step).rem_euclid(items.len() as i32) as usize;
        sounds.write(MenuSound::Move);
    }

    let action = if toggle || request.back {
        Some(match page {
            Page::QuitConfirm => Action::CancelQuit,
            Page::Main => Action::Resume,
        })
    } else if request.confirm {
        Some(items[pause.selected].1)
    } else {
        None
    };

    match action {
        Some(Action::Resume) => {
            sounds.write(MenuSound::Move);
            *pause = Pause::default();
            time.unpause();
        }
        Some(Action::OpenOptions) => {
            sounds.write(MenuSound::Confirm);
            if let Some(ref mut menu) = options_menu {
                menu.open(OptionsReturn::Pause);
            }
        }
        Some(Action::MainMenu) => {
            sounds.write(MenuSound::Confirm);
            *pause = Pause::default();
            time.unpause();
            next.set(GameState::Welcome);
        }
        Some(Action::OpenQuitConfirm) => {
            sounds.write(MenuSound::Move);
            *pause = Pause {
                page: Some(Page::QuitConfirm),
                selected: 0,
            };
        }
        Some(Action::CancelQuit) => {
            sounds.write(MenuSound::Move);
            // Back on "Exit game".
            *pause = Pause {
                page: Some(Page::Main),
                selected: self::items(Page::Main, settings.language)
                    .iter()
                    .position(|item| item.1 == Action::OpenQuitConfirm)
                    .unwrap_or(0),
            };
        }
        Some(Action::ConfirmQuit) => {
            exit.write(AppExit::Success);
        }
        None => {}
    }
}

/// Al salir de la carrera la pausa se cierra y el tiempo vuelve a correr.
fn close(mut pause: ResMut<Pause>, mut time: ResMut<Time<Virtual>>) {
    *pause = Pause::default();
    time.unpause();
}

/// El menú en pantalla, rehecho cada vez que cambia algo.
#[derive(Component)]
struct PauseMenu;

#[allow(clippy::too_many_arguments)]
fn show_pause_menu(
    mut commands: Commands,
    pause: Res<Pause>,
    options_menu: Option<Res<OptionsMenu>>,
    settings: Res<Settings>,
    device: Option<Res<InputDevice>>,
    active_controller: Option<Res<ActiveController>>,
    font: Res<UiFont>,
    icons: Res<UiIcons>,
    shown: Query<Entity, With<PauseMenu>>,
) {
    if !pause.is_changed()
        && options_menu.as_ref().is_some_and(|m| !m.is_changed())
        && !settings.is_changed()
        && device.as_ref().is_some_and(|d| !d.is_changed())
        && active_controller.as_ref().is_some_and(|c| !c.is_changed())
    {
        return;
    }
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    if options_menu.as_ref().is_some_and(|m| m.is_open) {
        return;
    }
    let Some(page) = pause.page else {
        return;
    };
    let lang = settings.language;
    let active_dev = current_active_device(
        device.as_deref().copied().unwrap_or_default(),
        active_controller.as_deref(),
    );
    let title = match page {
        Page::Main => match lang {
            Language::Spanish => "PAUSA",
            Language::English => "PAUSE",
        },
        Page::QuitConfirm => match lang {
            Language::Spanish => "¿SALIR DEL JUEGO?",
            Language::English => "EXIT GAME?",
        },
    };
    let rows: Vec<Row> = items(page, lang)
        .into_iter()
        .map(|(label, action)| {
            // Leaving the game, and confirming it, show in red.
            if matches!(action, Action::OpenQuitConfirm | Action::ConfirmQuit) {
                Row::danger(label)
            } else {
                Row::new(label)
            }
        })
        .collect();

    let back = match page {
        Page::QuitConfirm => lang.hint_cancel(),
        Page::Main => lang.hint_resume(),
    };
    let prompts = menu_prompts(
        active_dev,
        &[(Keys::Confirm, lang.hint_accept()), (Keys::Back, back)],
    );

    commands
        .spawn((
            PauseMenu,
            DespawnOnExit(GameState::Race),
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.06, 0.7)),
            // Por encima del marcador.
            GlobalZIndex(10),
        ))
        .with_children(|menu| {
            spawn_title(menu, &font, title, 52.0);
            spawn_panel(menu, &font, &rows, pause.selected, false);
            spawn_legend(menu, &font, &icons, active_dev, &prompts);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A race with the pause menu, and the keyboard to drive it.
    fn race() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::input::InputPlugin)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<Font>()
            .init_asset::<Image>()
            .init_state::<GameState>()
            .init_resource::<Settings>()
            .init_resource::<InputDevice>()
            .init_resource::<UiFont>()
            .init_resource::<UiIcons>()
            .add_message::<MenuSound>()
            .add_message::<AppExit>()
            .add_plugins((
                PausePlugin,
                crate::options::OptionsPlugin,
                crate::gamepads::GamepadsPlugin,
            ));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Race);
        app.update();
        app.update();
        app
    }

    /// Presses and lets go of `key`, a frame each.
    fn press(app: &mut App, key: KeyCode) {
        use bevy::input::ButtonState;
        use bevy::input::keyboard::{Key, KeyboardInput};
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(KeyboardInput {
                key_code: key,
                logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                state,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
            app.update();
        }
    }

    fn page(app: &App) -> (Option<Page>, usize) {
        let pause = app.world().resource::<Pause>();
        (pause.page, pause.selected)
    }

    #[test]
    fn leaving_the_game_from_the_pause_asks_first_and_cancelling_goes_back_to_it() {
        let mut app = race();
        press(&mut app, KeyCode::Escape);
        assert_eq!(page(&app), (Some(Page::Main), 0));
        // Up from the first option: the last one, "Exit game".
        press(&mut app, KeyCode::ArrowUp);
        let exit = items(Page::Main, Language::Spanish).len() - 1;
        assert_eq!(page(&app), (Some(Page::Main), exit));
        if WEB {
            return;
        }
        press(&mut app, KeyCode::Enter);
        assert_eq!(page(&app), (Some(Page::QuitConfirm), 0));
        press(&mut app, KeyCode::Enter);
        assert_eq!(page(&app), (Some(Page::Main), exit), "back on Exit game");
    }

    #[test]
    fn the_key_that_closes_the_options_does_not_resume_the_race() {
        let mut app = race();
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::ArrowDown);
        press(&mut app, KeyCode::Enter);
        assert!(app.world().resource::<OptionsMenu>().is_open);
        press(&mut app, KeyCode::Escape);
        assert!(!app.world().resource::<OptionsMenu>().is_open);
        assert_eq!(
            page(&app),
            (Some(Page::Main), 1),
            "still paused, on Options"
        );
    }

    #[test]
    fn the_pause_menu_offers_what_was_asked() {
        let main: Vec<Action> = items(Page::Main, Language::Spanish)
            .into_iter()
            .map(|i| i.1)
            .collect();
        assert_eq!(
            main,
            [
                Action::Resume,
                Action::OpenOptions,
                Action::MainMenu,
                Action::OpenQuitConfirm,
            ]
        );
        let quit_confirm: Vec<Action> = items(Page::QuitConfirm, Language::Spanish)
            .into_iter()
            .map(|i| i.1)
            .collect();
        assert_eq!(quit_confirm, [Action::CancelQuit, Action::ConfirmQuit]);

        // English items verification
        let main_en: Vec<String> = items(Page::Main, Language::English)
            .into_iter()
            .map(|i| i.0)
            .collect();
        assert_eq!(main_en[0], "Resume");
        assert_eq!(main_en[1], "Options");
        assert_eq!(main_en[2], "Return to main menu");
        assert_eq!(main_en[3], "Exit game");

        // Rates and limits behavior
        assert_eq!(TurnLimit::Off.next(-1), TurnLimit::High);
        assert_eq!(TurnLimit::High.strength(), 1.0);
        assert_eq!(SteerRate::Slow.next(-1), SteerRate::Fast);
        assert_eq!(SteerRate::Fast.factor(), 1.0);
        assert!(SteerRate::Slow.factor() < SteerRate::Medium.factor());
        assert!(SteerRate::Medium.factor() < SteerRate::Fast.factor());
        assert_eq!(ReturnRate::Slow.next(-1), ReturnRate::Fast);
        assert_eq!(ReturnRate::Fast.factor(), 1.0);
        assert!(ReturnRate::Slow.factor() < ReturnRate::Medium.factor());
        assert!(ReturnRate::Medium.factor() < ReturnRate::Fast.factor());
    }
}
