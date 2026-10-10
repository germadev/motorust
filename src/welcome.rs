//! Pantalla de bienvenida: el título del juego mientras la cámara recorre la
//! pista, con Comenzar, Opciones y Salir. En Opciones se cambian la
//! resolución, el límite de fotogramas por segundo y las teclas y botones de
//! cada acción ([`crate::settings`]). En la web no se sale del juego (se
//! cierra la pestaña) y los fotogramas los marca el navegador.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::CameraGlide;
use crate::hud::UiFont;
use crate::locale::{LANGUAGES, Language};
use crate::menu::GameState;
use crate::settings::{
    ACTIONS, Action, Controls, FPS_LIMITS, FULLSCREEN_KEYS, RESOLUTIONS, Settings, WEB, cycle,
};
use crate::track::Track;
use crate::ui::{
    MenuInput, MenuSound, Row, gamepad_status, spawn_hint, spawn_logo, spawn_panel, spawn_title,
    text_font, version_label,
};

/// Tiempo para pulsar la tecla o el botón nuevo antes de que se cancele (s).
const CAPTURE_TIME: f32 = 8.0;
/// La cámara recorre la pista a esta velocidad (m/s), a un lado del centro
/// de la carretera (m, positivo a la izquierda) y a esta altura sobre ella;
/// mira a un punto de la carretera más adelante (m), a esta altura, con este
/// campo de visión (rad).
const FLYBY_SPEED: f32 = 22.0;
const FLYBY_SIDE: f32 = -4.0;
const FLYBY_HEIGHT: f32 = 6.0;
const FLYBY_AHEAD: f32 = 45.0;
const FLYBY_LOOK_HEIGHT: f32 = 1.5;
const FLYBY_FOV: f32 = 0.9;

pub struct WelcomePlugin;

impl Plugin for WelcomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Welcome>()
            .add_systems(OnEnter(GameState::Welcome), open)
            .add_systems(
                Update,
                (use_welcome, show_welcome, fly_over_track)
                    .chain()
                    .run_if(in_state(GameState::Welcome)),
            );
    }
}

/// Qué página se ve, qué opción está elegida y si se espera una tecla nueva.
#[derive(Resource, Default)]
struct Welcome {
    page: Page,
    selected: usize,
    capture: Option<Capture>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Page {
    #[default]
    Main,
    Options,
    Controls,
}

/// Esperando la tecla o el botón nuevo de una acción desde este momento (s
/// de reloj).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Capture {
    action: Action,
    since: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Item {
    Start,
    Options,
    Quit,
    Fullscreen,
    Resolution,
    Fps,
    Language,
    Controls,
    Bind(Action),
    ResetControls,
    Back,
}

/// Las opciones de cada página.
fn items(page: Page) -> Vec<Item> {
    match page {
        Page::Main if WEB => vec![Item::Start, Item::Options],
        Page::Main => vec![Item::Start, Item::Options, Item::Quit],
        Page::Options if WEB => vec![Item::Fullscreen, Item::Language, Item::Controls, Item::Back],
        Page::Options => vec![
            Item::Fullscreen,
            Item::Resolution,
            Item::Fps,
            Item::Language,
            Item::Controls,
            Item::Back,
        ],
        Page::Controls => ACTIONS
            .iter()
            .map(|&action| Item::Bind(action))
            .chain([Item::ResetControls, Item::Back])
            .collect(),
    }
}

/// Cómo se ve una opción, con su valor si tiene.
fn row(item: Item, settings: &Settings, capture: Option<Capture>, window: Option<&Window>) -> Row {
    let lang = settings.language;
    match item {
        Item::Start => Row::new(lang.start()),
        Item::Options => Row::new(lang.options()),
        Item::Quit => Row::new(match lang {
            Language::Spanish => "Salir",
            Language::English => "Exit",
        }),
        Item::Fullscreen => Row::with_value(
            lang.fullscreen(),
            if settings.fullscreen { "[X]" } else { "[ ]" },
        ),
        Item::Resolution => Row::with_value(
            lang.resolution(),
            format!("<  {}  >", settings.resolution.name(window, lang)),
        ),
        Item::Fps => Row::with_value(lang.fps(), format!("<  {}  >", settings.fps.name(lang))),
        Item::Language => Row::with_value(lang.language(), format!("<  {}  >", lang.name())),
        Item::Controls => Row::new(lang.controls()),
        Item::Bind(action) => Row::with_value(
            action.name(lang),
            match capture {
                Some(capture) if capture.action == action => match lang {
                    Language::Spanish => "Pulsa una tecla o un botón".into(),
                    Language::English => "Press a key or a button".into(),
                },
                _ => settings.controls.describe(action),
            },
        ),
        Item::ResetControls => Row::new(match lang {
            Language::Spanish => "Restablecer los controles",
            Language::English => "Reset controls",
        }),
        Item::Back => Row::new(lang.back()),
    }
}

/// Al llegar, la página principal con Comenzar elegido.
fn open(mut welcome: ResMut<Welcome>) {
    *welcome = Welcome::default();
}

fn use_welcome(
    mut input: MenuInput,
    time: Res<Time<Real>>,
    mut welcome: ResMut<Welcome>,
    mut settings: ResMut<Settings>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    let now = time.elapsed_secs();
    if let Some(capture) = welcome.capture {
        // La primera tecla o el primer botón que se pulse; Esc cancela, y M
        // es siempre el sonido.
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
            welcome.capture = None;
            sounds.write(MenuSound::Move);
        } else if let Some(key) = key {
            settings.controls.bind_key(capture.action, key);
            welcome.capture = None;
            sounds.write(MenuSound::Confirm);
        } else if let Some(button) = button {
            settings.controls.bind_button(capture.action, button);
            welcome.capture = None;
            sounds.write(MenuSound::Confirm);
        }
        return;
    }

    let items = items(welcome.page);
    if request.step != 0 {
        welcome.selected =
            (welcome.selected as i32 + request.step).rem_euclid(items.len() as i32) as usize;
        sounds.write(MenuSound::Move);
    }
    let item = items[welcome.selected];
    // Esc o B vuelven a la página anterior.
    if request.back {
        match welcome.page {
            Page::Main => return,
            Page::Options => go(&mut welcome, Page::Main, Item::Options),
            Page::Controls => go(&mut welcome, Page::Options, Item::Controls),
        }
        sounds.write(MenuSound::Move);
        return;
    }
    // Izquierda y derecha cambian los valores; Enter o A, también, hacia
    // delante.
    let change = if request.side != 0 {
        request.side
    } else {
        i32::from(request.confirm)
    };
    match item {
        Item::Fullscreen if change != 0 => {
            settings.fullscreen = !settings.fullscreen;
            sounds.write(MenuSound::Move);
        }
        Item::Resolution if change != 0 => {
            settings.resolution = cycle(RESOLUTIONS, settings.resolution, change);
            sounds.write(MenuSound::Move);
        }
        Item::Fps if change != 0 => {
            settings.fps = cycle(&FPS_LIMITS, settings.fps, change);
            sounds.write(MenuSound::Move);
        }
        Item::Language if change != 0 => {
            settings.language = cycle(&LANGUAGES, settings.language, change);
            sounds.write(MenuSound::Move);
        }
        _ if !(request.confirm || (request.start && welcome.page == Page::Main)) => {}
        Item::Start => {
            sounds.write(MenuSound::Confirm);
            next.set(GameState::Tracks);
        }
        Item::Options => {
            go(&mut welcome, Page::Options, Item::Fullscreen);
            sounds.write(MenuSound::Confirm);
        }
        Item::Quit => {
            exit.write(AppExit::Success);
        }
        Item::Controls => {
            go(&mut welcome, Page::Controls, Item::Bind(ACTIONS[0]));
            sounds.write(MenuSound::Confirm);
        }
        Item::Bind(action) => {
            welcome.capture = Some(Capture { action, since: now });
            sounds.write(MenuSound::Confirm);
        }
        Item::ResetControls => {
            if settings.controls != Controls::default() {
                settings.controls = Controls::default();
            }
            sounds.write(MenuSound::Confirm);
        }
        Item::Back => {
            match welcome.page {
                Page::Controls => go(&mut welcome, Page::Options, Item::Controls),
                _ => go(&mut welcome, Page::Main, Item::Options),
            }
            sounds.write(MenuSound::Move);
        }
        Item::Fullscreen | Item::Resolution | Item::Fps | Item::Language => {}
    }
}

/// Pasa a otra página con una opción elegida.
fn go(welcome: &mut Welcome, page: Page, item: Item) {
    let selected = items(page).iter().position(|&i| i == item).unwrap_or(0);
    (welcome.page, welcome.selected) = (page, selected);
}

/// La pantalla, rehecha cada vez que cambia algo.
#[derive(Component)]
struct WelcomeScreen;

fn show_welcome(
    mut commands: Commands,
    welcome: Res<Welcome>,
    settings: Res<Settings>,
    window: Query<Ref<Window>, With<PrimaryWindow>>,
    font: Res<UiFont>,
    shown: Query<Entity, With<WelcomeScreen>>,
) {
    let window_ref = window.iter().next();
    let window_changed = window_ref.as_ref().is_some_and(|w| w.is_changed());
    if !welcome.is_changed() && !settings.is_changed() && !window_changed {
        return;
    }
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    let window_inner = window_ref.as_deref();
    let rows: Vec<Row> = items(welcome.page)
        .into_iter()
        .map(|item| row(item, &settings, welcome.capture, window_inner))
        .collect();
    let lang = settings.language;
    let hint = match (welcome.page, welcome.capture) {
        (Page::Main, _) => match lang {
            Language::Spanish => "Arriba y abajo: elegir   ·   Enter o A: aceptar".to_string(),
            Language::English => "Up and down: choose   ·   Enter or A: accept".to_string(),
        },
        (Page::Options, _) => match lang {
            Language::Spanish => {
                "Arriba y abajo: elegir   ·   Izquierda y derecha: cambiar   ·   Esc o B: volver"
                    .to_string()
            }
            Language::English => {
                "Up and down: choose   ·   Left and right: change   ·   Esc or B: back".to_string()
            }
        },
        (Page::Controls, None) => match lang {
            Language::Spanish => {
                "Enter o A: cambiar la tecla o el botón   ·   Esc o B: volver".to_string()
            }
            Language::English => {
                "Enter or A: change key or button   ·   Esc or B: back".to_string()
            }
        },
        (Page::Controls, Some(capture)) => match lang {
            Language::Spanish => format!(
                "Pulsa la tecla o el botón para «{}»   ·   Esc: cancelar",
                capture.action.name(lang)
            ),
            Language::English => format!(
                "Press the key or button for \"{}\"   ·   Esc: cancel",
                capture.action.name(lang)
            ),
        },
    };
    commands
        .spawn((
            WelcomeScreen,
            DespawnOnExit(GameState::Welcome),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.06, 0.3)),
        ))
        .with_children(|screen| {
            match welcome.page {
                Page::Main => {
                    spawn_logo(screen, &font, 110.0);
                    screen.spawn((
                        Text::new(match lang {
                            Language::Spanish => "Carreras de motos arcade",
                            Language::English => "Arcade motorcycle racing",
                        }),
                        text_font(&font, 24.0),
                        TextShadow::default(),
                        Node {
                            margin: UiRect::bottom(px(26)),
                            ..default()
                        },
                    ));
                }
                Page::Options => spawn_title(
                    screen,
                    &font,
                    match lang {
                        Language::Spanish => "OPCIONES",
                        Language::English => "OPTIONS",
                    },
                    52.0,
                ),
                Page::Controls => spawn_title(
                    screen,
                    &font,
                    match lang {
                        Language::Spanish => "CONTROLES",
                        Language::English => "CONTROLS",
                    },
                    52.0,
                ),
            }
            spawn_panel(
                screen,
                &font,
                &rows,
                welcome.selected,
                welcome.page == Page::Controls,
            );
            if welcome.page == Page::Controls {
                let note = match lang {
                    Language::Spanish => format!(
                        "Siempre: stick izquierdo para girar (hacia atrás, caballito)   ·   \
                         stick derecho para mirar   ·   M para el sonido   ·   \
                         {FULLSCREEN_KEYS} para la pantalla completa"
                    ),
                    Language::English => format!(
                        "Always: left stick to steer (backward, wheelie)   ·   \
                         right stick to look   ·   M for sound   ·   \
                         {FULLSCREEN_KEYS} for fullscreen"
                    ),
                };
                screen.spawn((
                    Text::new(note),
                    text_font(&font, 16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                    TextShadow::default(),
                ));
            }
            spawn_hint(screen, &font, &hint);
            screen.spawn(gamepad_status(&font));
            screen.spawn(version_label(&font));
        });
}

/// La cámara recorre la pista despacio, un poco por encima de la carretera.
fn fly_over_track(
    time: Res<Time>,
    track: Res<Track>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut glide: ResMut<CameraGlide>,
    mut travelled: Local<f32>,
) {
    let dt = time.delta_secs();
    let length = track.sample_count() as f32 * track.step_length();
    *travelled = (*travelled + FLYBY_SPEED * dt) % length;
    // Un punto de la carretera a `distance` m de la meta, entre dos muestras.
    let at = |distance: f32, lateral: f32| {
        let samples = distance.rem_euclid(length) / track.step_length();
        let index = samples as usize;
        track
            .surface_point(index, lateral)
            .lerp(track.surface_point(index + 1, lateral), samples.fract())
    };
    let position = at(*travelled, FLYBY_SIDE) + Vec3::Y * FLYBY_HEIGHT;
    let target = at(*travelled + FLYBY_AHEAD, 0.0) + Vec3::Y * FLYBY_LOOK_HEIGHT;
    let wanted = Transform::from_translation(position).looking_at(target, Vec3::Y);
    let (mut transform, mut projection) = camera.into_inner();
    glide.apply(&mut transform, &mut projection, wanted, FLYBY_FOV, dt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Resolution;

    #[test]
    fn the_welcome_screen_offers_what_was_asked() {
        assert_eq!(items(Page::Main), [Item::Start, Item::Options, Item::Quit]);
        if WEB {
            assert_eq!(
                items(Page::Options),
                [Item::Fullscreen, Item::Language, Item::Controls, Item::Back]
            );
        } else {
            assert_eq!(
                items(Page::Options),
                [
                    Item::Fullscreen,
                    Item::Resolution,
                    Item::Fps,
                    Item::Language,
                    Item::Controls,
                    Item::Back,
                ]
            );
        }
        // Una fila por acción, y después restablecer y volver.
        let controls = items(Page::Controls);
        assert_eq!(controls.len(), ACTIONS.len() + 2);
        assert_eq!(controls[0], Item::Bind(Action::Throttle));
        // Mientras se espera la tecla nueva, su fila lo dice.
        let mut settings = Settings {
            language: Language::Spanish,
            ..default()
        };
        let capture = Capture {
            action: Action::Brake,
            since: 0.0,
        };
        let value = |action| {
            row(Item::Bind(action), &settings, Some(capture), None)
                .value
                .unwrap()
        };
        assert_eq!(value(Action::Brake), "Pulsa una tecla o un botón");
        assert_eq!(value(Action::Throttle), "W, Arriba   ·   RT, A");

        settings.language = Language::English;
        let value_en = |action| {
            row(Item::Bind(action), &settings, Some(capture), None)
                .value
                .unwrap()
        };
        assert_eq!(value_en(Action::Brake), "Press a key or a button");
    }

    #[test]
    fn the_resolution_row_shows_dynamic_by_default() {
        let mut settings = Settings {
            language: Language::Spanish,
            ..default()
        };
        assert_eq!(settings.resolution, Resolution::Dynamic);
        let res_row = row(Item::Resolution, &settings, None, None);
        assert_eq!(res_row.value, Some("<  Dinámica  >".into()));

        settings.language = Language::English;
        let res_row_en = row(Item::Resolution, &settings, None, None);
        assert_eq!(res_row_en.value, Some("<  Dynamic  >".into()));
    }

    #[test]
    fn the_fullscreen_row_shows_checkbox() {
        let mut settings = Settings {
            fullscreen: false,
            ..default()
        };
        let fs_row = row(Item::Fullscreen, &settings, None, None);
        assert_eq!(fs_row.value, Some("[ ]".into()));

        settings.fullscreen = true;
        let fs_row_on = row(Item::Fullscreen, &settings, None, None);
        assert_eq!(fs_row_on.value, Some("[X]".into()));
    }

    #[test]
    fn the_welcome_screen_carries_the_version() {
        let version = concat!("v", env!("CARGO_PKG_VERSION"));
        assert!(version.starts_with('v'));
        assert_eq!(version, format!("v{}", env!("CARGO_PKG_VERSION")));
    }
}
