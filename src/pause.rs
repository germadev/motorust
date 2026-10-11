//! Menú de pausa: Esc o Start paran la carrera y abren un menú para seguir,
//! volver a la pantalla de inicio, salir del juego o cambiar opciones de
//! depuración.

use bevy::prelude::*;

use crate::gamepads::Gamepads;
use crate::hud::UiFont;
use crate::locale::Language;
use crate::menu::GameState;
use crate::settings::{Action as Control, Settings, WEB, cycle};
use crate::ui::{MenuInput, Row, spawn_hint, spawn_panel, spawn_title};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pause>()
            .init_resource::<DebugOptions>()
            .add_systems(OnExit(GameState::Race), close)
            .add_systems(
                Update,
                (use_pause_menu, show_pause_menu)
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
    /// la tecla de girar (de entrada, rápido - actual).
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
            keyboard_return: ReturnRate::Fast,
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

    fn name(self, lang: Language) -> &'static str {
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
    fn next(self, step: i32) -> Self {
        cycle(&STEER_RATES, self, step)
    }
}

/// Rapidez con la que recupera el teclado o la cruceta al soltar la tecla de
/// girar: lenta, media o rápido (actual).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReturnRate {
    Slow,
    Medium,
    Fast,
}

const RETURN_RATES: [ReturnRate; 3] = [ReturnRate::Slow, ReturnRate::Medium, ReturnRate::Fast];

impl ReturnRate {
    /// Factor que multiplica la rapidez de recuperación al soltar.
    pub fn factor(self) -> f32 {
        match self {
            ReturnRate::Slow => 0.5,
            ReturnRate::Medium => 0.75,
            ReturnRate::Fast => 1.0,
        }
    }

    fn name(self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => match self {
                ReturnRate::Slow => "lenta",
                ReturnRate::Medium => "media",
                ReturnRate::Fast => "rápido (actual)",
            },
            Language::English => match self {
                ReturnRate::Slow => "slow",
                ReturnRate::Medium => "medium",
                ReturnRate::Fast => "fast (current)",
            },
        }
    }

    /// El nivel siguiente (`step` 1) o el anterior (-1), dando la vuelta.
    fn next(self, step: i32) -> Self {
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

    fn name(self, lang: Language) -> &'static str {
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
    fn next(self, step: i32) -> Self {
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
enum Page {
    Main,
    Debug,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Resume,
    MainMenu,
    OpenDebug,
    Quit,
    NextStickLimit,
    NextKeyboardLimit,
    NextKeyboardRate,
    NextKeyboardReturn,
    ToggleLegend,
    SearchGamepads,
    Back,
}

/// Las opciones de cada página: su texto y lo que hacen.
fn items(page: Page, options: &DebugOptions, lang: Language) -> Vec<(String, Action)> {
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
                        Language::Spanish => "Volver al menú principal".into(),
                        Language::English => "Return to main menu".into(),
                    },
                    Action::MainMenu,
                ),
                (
                    match lang {
                        Language::Spanish => "Depuración".into(),
                        Language::English => "Debug".into(),
                    },
                    Action::OpenDebug,
                ),
            ];
            // En la web se sale cerrando la pestaña.
            if !WEB {
                items.push((
                    match lang {
                        Language::Spanish => "Salir del juego".into(),
                        Language::English => "Exit game".into(),
                    },
                    Action::Quit,
                ));
            }
            items
        }
        Page::Debug => vec![
            (
                match lang {
                    Language::Spanish => format!(
                        "Stick: limitar el giro con la velocidad: {}",
                        options.stick_limit.name(lang)
                    ),
                    Language::English => format!(
                        "Stick: limit turn with speed: {}",
                        options.stick_limit.name(lang)
                    ),
                },
                Action::NextStickLimit,
            ),
            (
                match lang {
                    Language::Spanish => format!(
                        "Teclado y cruceta: limitar el giro con la velocidad: {}",
                        options.keyboard_limit.name(lang)
                    ),
                    Language::English => format!(
                        "Keyboard and D-pad: limit turn with speed: {}",
                        options.keyboard_limit.name(lang)
                    ),
                },
                Action::NextKeyboardLimit,
            ),
            (
                match lang {
                    Language::Spanish => format!(
                        "Teclado y cruceta: cuánto gira al dejar pulsado: {}",
                        options.keyboard_rate.name(lang)
                    ),
                    Language::English => format!(
                        "Keyboard and D-pad: turn rate when held: {}",
                        options.keyboard_rate.name(lang)
                    ),
                },
                Action::NextKeyboardRate,
            ),
            (
                match lang {
                    Language::Spanish => format!(
                        "Teclado y cruceta: al soltar la tecla de girar: {}",
                        options.keyboard_return.name(lang)
                    ),
                    Language::English => format!(
                        "Keyboard and D-pad: return rate when released: {}",
                        options.keyboard_return.name(lang)
                    ),
                },
                Action::NextKeyboardReturn,
            ),
            (
                match lang {
                    Language::Spanish => format!(
                        "Leyenda de los controles: {}",
                        if options.show_legend {
                            "se ve"
                        } else {
                            "oculta"
                        }
                    ),
                    Language::English => format!(
                        "Controls legend: {}",
                        if options.show_legend {
                            "visible"
                        } else {
                            "hidden"
                        }
                    ),
                },
                Action::ToggleLegend,
            ),
            (
                match lang {
                    Language::Spanish => "Volver a buscar mandos".into(),
                    Language::English => "Scan for controllers again".into(),
                },
                Action::SearchGamepads,
            ),
            (
                match lang {
                    Language::Spanish => "Volver".into(),
                    Language::English => "Back".into(),
                },
                Action::Back,
            ),
        ],
    }
}

/// Abre y cierra la pausa, se mueve por las opciones y las aplica.
#[allow(clippy::too_many_arguments)]
fn use_pause_menu(
    mut input: MenuInput,
    settings: Res<Settings>,
    mut pause: ResMut<Pause>,
    mut options: ResMut<DebugOptions>,
    mut time: ResMut<Time<Virtual>>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut gamepads: ResMut<Gamepads>,
) {
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
            time.pause();
        }
        return;
    };
    let items = items(page, &options, settings.language);
    if request.step != 0 {
        pause.selected =
            (pause.selected as i32 + request.step).rem_euclid(items.len() as i32) as usize;
    }
    // Pausa, Esc o B: atrás desde depuración y, desde el menú, seguir.
    // Izquierda y derecha cambian la opción de depuración.
    let action = if toggle || request.back {
        Some(if page == Page::Debug {
            Action::Back
        } else {
            Action::Resume
        })
    } else if request.confirm
        || (request.side != 0
            && matches!(
                items[pause.selected].1,
                Action::NextStickLimit
                    | Action::NextKeyboardLimit
                    | Action::NextKeyboardRate
                    | Action::NextKeyboardReturn
                    | Action::ToggleLegend
            ))
    {
        Some(items[pause.selected].1)
    } else {
        None
    };
    match action {
        Some(Action::Resume) => {
            *pause = Pause::default();
            time.unpause();
        }
        Some(Action::MainMenu) => {
            next.set(GameState::Welcome);
        }
        Some(Action::OpenDebug) => {
            *pause = Pause {
                page: Some(Page::Debug),
                selected: 0,
            };
        }
        Some(Action::Quit) => {
            exit.write(AppExit::Success);
        }
        Some(Action::NextStickLimit) => {
            options.stick_limit = options
                .stick_limit
                .next(if request.side < 0 { -1 } else { 1 });
        }
        Some(Action::NextKeyboardLimit) => {
            options.keyboard_limit =
                options
                    .keyboard_limit
                    .next(if request.side < 0 { -1 } else { 1 });
        }
        Some(Action::NextKeyboardRate) => {
            options.keyboard_rate =
                options
                    .keyboard_rate
                    .next(if request.side < 0 { -1 } else { 1 });
        }
        Some(Action::NextKeyboardReturn) => {
            options.keyboard_return =
                options
                    .keyboard_return
                    .next(if request.side < 0 { -1 } else { 1 });
        }
        Some(Action::ToggleLegend) => {
            options.show_legend = !options.show_legend;
        }
        Some(Action::SearchGamepads) => {
            gamepads.search_again();
        }
        Some(Action::Back) => {
            *pause = Pause {
                page: Some(Page::Main),
                selected: 2,
            };
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

fn show_pause_menu(
    mut commands: Commands,
    pause: Res<Pause>,
    options: Res<DebugOptions>,
    settings: Res<Settings>,
    font: Res<UiFont>,
    shown: Query<Entity, With<PauseMenu>>,
) {
    if !pause.is_changed() && !options.is_changed() && !settings.is_changed() {
        return;
    }
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    let Some(page) = pause.page else {
        return;
    };
    let lang = settings.language;
    let title = match page {
        Page::Main => match lang {
            Language::Spanish => "PAUSA",
            Language::English => "PAUSE",
        },
        Page::Debug => match lang {
            Language::Spanish => "DEPURACIÓN",
            Language::English => "DEBUG",
        },
    };
    let rows: Vec<Row> = items(page, &options, lang)
        .into_iter()
        .map(|(label, _)| Row::new(label))
        .collect();
    let hint = match lang {
        Language::Spanish => {
            "Arriba y abajo: elegir   ·   Enter o A: aceptar   ·   Esc o B: volver"
        }
        Language::English => "Up and down: choose   ·   Enter or A: accept   ·   Esc or B: back",
    };
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
            BackgroundColor(Color::srgba(0.0, 0.02, 0.06, 0.6)),
            // Por encima del marcador.
            GlobalZIndex(10),
        ))
        .with_children(|menu| {
            spawn_title(menu, &font, title, 52.0);
            spawn_panel(menu, &font, &rows, pause.selected, false);
            spawn_hint(menu, &font, hint);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pause_menu_offers_what_was_asked() {
        let options = DebugOptions::default();
        let main: Vec<Action> = items(Page::Main, &options, Language::Spanish)
            .into_iter()
            .map(|i| i.1)
            .collect();
        assert_eq!(
            main,
            [
                Action::Resume,
                Action::MainMenu,
                Action::OpenDebug,
                Action::Quit
            ]
        );
        // La opción de depuración dice cómo está y pasa por los tres niveles
        // antes de volver a no limitar.
        let mut limit = options.stick_limit;
        let mut shown = Vec::new();
        for _ in 0..5 {
            let options = DebugOptions {
                stick_limit: limit,
                ..default()
            };
            let label = items(Page::Debug, &options, Language::Spanish)[0].0.clone();
            shown.push(label.rsplit(' ').next().unwrap().to_string());
            limit = limit.next(1);
        }
        assert_eq!(shown, ["medio", "alto", "no", "bajo", "medio"]);
        assert_eq!(TurnLimit::Off.next(-1), TurnLimit::High);
        // El teclado y la cruceta, limitados a medio de entrada.
        assert_eq!(
            items(Page::Debug, &options, Language::Spanish)[1].1,
            Action::NextKeyboardLimit
        );
        assert!(
            items(Page::Debug, &options, Language::Spanish)[1]
                .0
                .ends_with("medio")
        );
        // Cuánto gira al dejar pulsado: medio de entrada, y pasa por alto y lento.
        assert_eq!(
            items(Page::Debug, &options, Language::Spanish)[2].1,
            Action::NextKeyboardRate
        );
        assert!(
            items(Page::Debug, &options, Language::Spanish)[2]
                .0
                .ends_with("medio")
        );
        let mut rate = options.keyboard_rate;
        let mut shown_rates = Vec::new();
        for _ in 0..4 {
            let options = DebugOptions {
                keyboard_rate: rate,
                ..default()
            };
            let label = items(Page::Debug, &options, Language::Spanish)[2].0.clone();
            shown_rates.push(label.rsplit(' ').next().unwrap().to_string());
            rate = rate.next(1);
        }
        assert_eq!(shown_rates, ["medio", "alto", "lento", "medio"]);
        assert_eq!(SteerRate::Slow.next(-1), SteerRate::Fast);
        assert_eq!(SteerRate::Fast.factor(), 1.0);
        assert!(SteerRate::Slow.factor() < SteerRate::Medium.factor());
        assert!(SteerRate::Medium.factor() < SteerRate::Fast.factor());
        // La velocidad a la que recupera al soltar: rápido (actual) de entrada, y pasa por lenta y media.
        assert_eq!(
            items(Page::Debug, &options, Language::Spanish)[3].1,
            Action::NextKeyboardReturn
        );
        assert!(
            items(Page::Debug, &options, Language::Spanish)[3]
                .0
                .ends_with("rápido (actual)")
        );
        let mut ret = options.keyboard_return;
        let mut shown_returns = Vec::new();
        for _ in 0..4 {
            let options = DebugOptions {
                keyboard_return: ret,
                ..default()
            };
            let label = items(Page::Debug, &options, Language::Spanish)[3].0.clone();
            shown_returns.push(label.split(": ").last().unwrap().to_string());
            ret = ret.next(1);
        }
        assert_eq!(shown_returns, ["rápido (actual)", "lenta", "media", "rápido (actual)"]);
        assert_eq!(ReturnRate::Slow.next(-1), ReturnRate::Fast);
        assert_eq!(ReturnRate::Fast.factor(), 1.0);
        assert!(ReturnRate::Slow.factor() < ReturnRate::Medium.factor());
        assert!(ReturnRate::Medium.factor() < ReturnRate::Fast.factor());
        // La leyenda de los controles, oculta de entrada.
        assert_eq!(
            items(Page::Debug, &options, Language::Spanish)[4].1,
            Action::ToggleLegend
        );
        assert!(
            items(Page::Debug, &options, Language::Spanish)[4]
                .0
                .ends_with("oculta")
        );
        assert_eq!(TurnLimit::High.strength(), 1.0);
        // Volver desde depuración deja elegida la entrada de depuración.
        assert_eq!(
            items(Page::Main, &options, Language::Spanish)[2].1,
            Action::OpenDebug
        );

        // English items verification
        let main_en: Vec<String> = items(Page::Main, &options, Language::English)
            .into_iter()
            .map(|i| i.0)
            .collect();
        assert_eq!(main_en[0], "Resume");
        assert_eq!(main_en[1], "Return to main menu");
        assert_eq!(main_en[2], "Debug");
        let debug_en = items(Page::Debug, &options, Language::English);
        assert!(debug_en[0].0.ends_with("medium"));
        assert!(debug_en[3].0.ends_with("fast (current)"));
    }
}
