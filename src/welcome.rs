//! Pantalla de bienvenida: el título del juego mientras la cámara recorre la
//! pista, con opciones de 1 jugador, 2 jugadores, Opciones y Salir.
//! En la web no se sale del juego (se cierra la pestaña).

use bevy::prelude::*;

use crate::camera::CameraGlide;
use crate::gamepads::ActiveController;
use crate::hud::UiFont;
use crate::locale::Language;
use crate::menu::GameState;
use crate::options::{OptionsMenu, OptionsReturn};
use crate::settings::{Settings, WEB};
use crate::track::Track;
use crate::ui::{
    InputDevice, MenuInput, MenuSound, UiIcons, current_active_device, menu_prompts,
    spawn_legend_bar, spawn_logo, spawn_title, text_font, version_label,
};

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
            .init_resource::<GameMode>()
            .add_systems(OnEnter(GameState::Welcome), open)
            .add_systems(
                Update,
                (
                    (use_welcome, show_welcome, animate_welcome_cards)
                        .run_if(in_state(GameState::Welcome)),
                    fly_over_track
                        .run_if(in_state(GameState::Welcome).or_else(in_state(GameState::Lobby))),
                ),
            );
    }
}

/// Modo de juego elegido en la pantalla principal.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    #[default]
    SinglePlayer,
    SplitScreen,
}

/// Qué página se ve y qué opción está elegida.
#[derive(Resource, Default)]
pub struct Welcome {
    pub page: Page,
    pub selected: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Page {
    #[default]
    Main,
    QuitConfirm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    SinglePlayer,
    SplitScreen,
    Options,
    Quit,
    CancelQuit,
    ConfirmQuit,
}

pub fn items(page: Page) -> Vec<Item> {
    match page {
        Page::Main => {
            if WEB {
                vec![Item::SinglePlayer, Item::SplitScreen, Item::Options]
            } else {
                vec![
                    Item::SinglePlayer,
                    Item::SplitScreen,
                    Item::Options,
                    Item::Quit,
                ]
            }
        }
        Page::QuitConfirm => vec![Item::CancelQuit, Item::ConfirmQuit],
    }
}

fn open(mut welcome: ResMut<Welcome>, mut options_menu: ResMut<OptionsMenu>) {
    *welcome = Welcome::default();
    options_menu.close();
}

#[allow(clippy::too_many_arguments)]
fn use_welcome(
    mut input: MenuInput,
    mut welcome: ResMut<Welcome>,
    mut options_menu: ResMut<OptionsMenu>,
    mut game_mode: ResMut<GameMode>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut sounds: MessageWriter<MenuSound>,
) {
    if options_menu.is_open {
        return;
    }

    let request = input.read();

    // Esc o B:
    if request.back {
        match welcome.page {
            Page::Main => {
                if !WEB {
                    welcome.page = Page::QuitConfirm;
                    welcome.selected = 0;
                    sounds.write(MenuSound::Move);
                }
            }
            Page::QuitConfirm => {
                welcome.page = Page::Main;
                welcome.selected = 0;
                sounds.write(MenuSound::Move);
            }
        }
        return;
    }

    // Navegación en la pantalla principal
    if welcome.page == Page::Main {
        if request.step != 0 {
            let max_idx = if WEB { 2 } else { 3 };
            welcome.selected = match welcome.selected {
                0 | 1 => {
                    if request.step > 0 {
                        2
                    } else {
                        max_idx
                    }
                }
                2 => {
                    if request.step > 0 {
                        if WEB { 0 } else { 3 }
                    } else {
                        0
                    }
                }
                3 => {
                    if request.step > 0 {
                        0
                    } else {
                        2
                    }
                }
                _ => 0,
            };
            sounds.write(MenuSound::Move);
        }

        if request.side != 0 {
            if welcome.selected == 0 && request.side > 0 {
                welcome.selected = 1;
                sounds.write(MenuSound::Move);
            } else if welcome.selected == 1 && request.side < 0 {
                welcome.selected = 0;
                sounds.write(MenuSound::Move);
            }
        }

        if request.confirm || request.start {
            match welcome.selected {
                0 => {
                    sounds.write(MenuSound::Confirm);
                    *game_mode = GameMode::SinglePlayer;
                    next.set(GameState::Tracks);
                }
                1 => {
                    sounds.write(MenuSound::Confirm);
                    *game_mode = GameMode::SplitScreen;
                    next.set(GameState::Lobby);
                }
                2 => {
                    sounds.write(MenuSound::Confirm);
                    options_menu.open(OptionsReturn::Welcome);
                }
                3 => {
                    sounds.write(MenuSound::Move);
                    welcome.page = Page::QuitConfirm;
                    welcome.selected = 0;
                }
                _ => {}
            }
        }
        return;
    }

    // Navegación en confirmación de salida
    if welcome.page == Page::QuitConfirm {
        let items = items(Page::QuitConfirm);
        if request.step != 0 || request.side != 0 {
            let delta = if request.step != 0 {
                request.step
            } else {
                request.side
            };
            welcome.selected =
                (welcome.selected as i32 + delta).rem_euclid(items.len() as i32) as usize;
            sounds.write(MenuSound::Move);
        }

        if request.confirm {
            match items[welcome.selected] {
                Item::CancelQuit => {
                    sounds.write(MenuSound::Move);
                    welcome.page = Page::Main;
                    welcome.selected = 0;
                }
                Item::ConfirmQuit => {
                    exit.write(AppExit::Success);
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
struct WelcomeScreen;

#[derive(Component)]
struct WelcomeCard {
    index: usize,
}

#[allow(clippy::too_many_arguments)]
fn show_welcome(
    mut commands: Commands,
    welcome: Res<Welcome>,
    options_menu: Res<OptionsMenu>,
    settings: Res<Settings>,
    device: Res<InputDevice>,
    active_controller: Option<Res<ActiveController>>,
    icons: Res<UiIcons>,
    font: Res<UiFont>,
    shown: Query<Entity, With<WelcomeScreen>>,
) {
    if !welcome.is_changed()
        && !options_menu.is_changed()
        && !settings.is_changed()
        && !device.is_changed()
    {
        return;
    }

    for entity in &shown {
        commands.entity(entity).despawn();
    }

    // Si el menú de opciones está abierto, ocultamos la pantalla de bienvenida
    if options_menu.is_open {
        return;
    }

    let lang = settings.language;
    let active_dev = current_active_device(*device, active_controller.as_deref());

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
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.06, 0.3)),
        ))
        .with_children(|screen| {
            match welcome.page {
                Page::Main => {
                    spawn_logo(screen, &font, 110.0);

                    // Contenedor principal de tarjetas según el diseño de referencia
                    screen
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(16),
                            margin: UiRect::top(px(14)),
                            ..default()
                        },))
                        .with_children(|menu_box| {
                            // Fila superior: [ 1P ][ 2P ]
                            menu_box
                                .spawn((Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(20),
                                    ..default()
                                },))
                                .with_children(|row| {
                                    // 1P Card: Casco a la izquierda mirando a la derecha, 1P a la derecha
                                    let is_1p = welcome.selected == 0;
                                    row.spawn((
                                        WelcomeCard { index: 0 },
                                        Node {
                                            width: px(360),
                                            height: px(180),
                                            padding: UiRect::axes(px(24), px(16)),
                                            border_radius: BorderRadius::all(px(16)),
                                            border: UiRect::all(if is_1p {
                                                px(3.0)
                                            } else {
                                                px(1.5)
                                            }),
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceEvenly,
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        ImageNode::new(if is_1p {
                                            icons.card_blue.clone()
                                        } else {
                                            icons.card_dark.clone()
                                        }),
                                        BorderColor::all(if is_1p {
                                            Color::WHITE
                                        } else {
                                            Color::srgba(1.0, 1.0, 1.0, 0.22)
                                        }),
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            ImageNode::new(icons.helmet.clone()),
                                            Node {
                                                width: px(122),
                                                height: px(122),
                                                ..default()
                                            },
                                        ));
                                        card.spawn((
                                            Text::new("1P"),
                                            text_font(&font, 70.0),
                                            TextColor(Color::WHITE),
                                            TextShadow {
                                                offset: Vec2::splat(2.5),
                                                color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                                            },
                                        ));
                                    });

                                    // 2P Card: Casco izquierda, 2P centro, Casco invertido derecha
                                    let is_2p = welcome.selected == 1;
                                    row.spawn((
                                        WelcomeCard { index: 1 },
                                        Node {
                                            width: px(360),
                                            height: px(180),
                                            padding: UiRect::axes(px(16), px(16)),
                                            border_radius: BorderRadius::all(px(16)),
                                            border: UiRect::all(if is_2p {
                                                px(3.0)
                                            } else {
                                                px(1.5)
                                            }),
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        ImageNode::new(if is_2p {
                                            icons.card_blue.clone()
                                        } else {
                                            icons.card_dark.clone()
                                        }),
                                        BorderColor::all(if is_2p {
                                            Color::WHITE
                                        } else {
                                            Color::srgba(1.0, 1.0, 1.0, 0.22)
                                        }),
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            ImageNode::new(icons.helmet.clone()),
                                            Node {
                                                width: px(96),
                                                height: px(96),
                                                ..default()
                                            },
                                        ));
                                        card.spawn((
                                            Text::new("2P"),
                                            text_font(&font, 62.0),
                                            TextColor(Color::WHITE),
                                            TextShadow {
                                                offset: Vec2::splat(2.5),
                                                color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                                            },
                                        ));
                                        card.spawn((
                                            ImageNode {
                                                image: icons.helmet.clone(),
                                                flip_x: true,
                                                ..default()
                                            },
                                            Node {
                                                width: px(96),
                                                height: px(96),
                                                ..default()
                                            },
                                        ));
                                    });
                                });

                            // Fila Opciones: Banner horizontal con fondo amarillo/dorado
                            let is_opt = welcome.selected == 2;
                            menu_box
                                .spawn((
                                    WelcomeCard { index: 2 },
                                    Node {
                                        width: px(740),
                                        height: px(68),
                                        padding: UiRect::axes(px(24), px(12)),
                                        border_radius: BorderRadius::all(px(14)),
                                        border: UiRect::all(if is_opt { px(3.0) } else { px(1.5) }),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: icons.banner_yellow.clone(),
                                        color: if is_opt {
                                            Color::WHITE
                                        } else {
                                            Color::srgba(0.85, 0.85, 0.85, 0.82)
                                        },
                                        ..default()
                                    },
                                    BorderColor::all(if is_opt {
                                        Color::WHITE
                                    } else {
                                        Color::srgba(1.0, 1.0, 1.0, 0.22)
                                    }),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(lang.options().to_uppercase()),
                                        text_font(&font, 32.0),
                                        TextColor(Color::WHITE),
                                        TextShadow {
                                            offset: Vec2::splat(2.0),
                                            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                                        },
                                    ));
                                });

                            // Fila Salir del juego: Banner horizontal con fondo rojo
                            if !WEB {
                                let is_quit = welcome.selected == 3;
                                menu_box
                                    .spawn((
                                        WelcomeCard { index: 3 },
                                        Node {
                                            width: px(740),
                                            height: px(68),
                                            padding: UiRect::axes(px(24), px(12)),
                                            border_radius: BorderRadius::all(px(14)),
                                            border: UiRect::all(if is_quit {
                                                px(3.0)
                                            } else {
                                                px(1.5)
                                            }),
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        ImageNode {
                                            image: icons.banner_red.clone(),
                                            color: if is_quit {
                                                Color::WHITE
                                            } else {
                                                Color::srgba(0.85, 0.85, 0.85, 0.82)
                                            },
                                            ..default()
                                        },
                                        BorderColor::all(if is_quit {
                                            Color::WHITE
                                        } else {
                                            Color::srgba(1.0, 1.0, 1.0, 0.22)
                                        }),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(lang.exit_game().to_uppercase()),
                                            text_font(&font, 32.0),
                                            TextColor(Color::WHITE),
                                            TextShadow {
                                                offset: Vec2::splat(2.0),
                                                color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                                            },
                                        ));
                                    });
                            }
                        });

                    // Leyenda de control en la parte inferior de la pantalla
                    let prompts = menu_prompts(
                        active_dev,
                        lang,
                        welcome.selected < 2,
                        match lang {
                            Language::Spanish => "Jugar / Aceptar",
                            Language::English => "Play / Select",
                        },
                        if !WEB { Some(lang.exit_game()) } else { None },
                    );
                    spawn_legend_bar(screen, &font, &prompts);
                }
                Page::QuitConfirm => {
                    spawn_title(
                        screen,
                        &font,
                        match lang {
                            Language::Spanish => "¿SALIR DEL JUEGO?",
                            Language::English => "EXIT GAME?",
                        },
                        48.0,
                    );

                    screen
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: px(12),
                                min_width: px(380),
                                padding: UiRect::all(px(16)),
                                border_radius: BorderRadius::all(px(16)),
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.88)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
                        ))
                        .with_children(|panel| {
                            let items = [
                                match lang {
                                    Language::Spanish => "Cancelar",
                                    Language::English => "Cancel",
                                },
                                match lang {
                                    Language::Spanish => "Salir del juego",
                                    Language::English => "Exit game",
                                },
                            ];
                            for (index, label) in items.iter().enumerate() {
                                let chosen = index == welcome.selected;
                                panel
                                    .spawn((
                                        Node {
                                            width: percent(100),
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(24), px(10)),
                                            border_radius: BorderRadius::all(px(10)),
                                            ..default()
                                        },
                                        BackgroundColor(if chosen {
                                            Color::srgba(1.0, 0.8, 0.2, 0.95)
                                        } else {
                                            Color::NONE
                                        }),
                                    ))
                                    .with_child((
                                        Text::new(*label),
                                        text_font(&font, 22.0),
                                        TextColor(if chosen {
                                            Color::srgb(0.08, 0.06, 0.02)
                                        } else {
                                            Color::WHITE
                                        }),
                                    ));
                            }
                        });

                    let prompts = menu_prompts(
                        active_dev,
                        lang,
                        false,
                        match lang {
                            Language::Spanish => "Aceptar",
                            Language::English => "Select",
                        },
                        Some(match lang {
                            Language::Spanish => "Cancelar",
                            Language::English => "Cancel",
                        }),
                    );
                    spawn_legend_bar(screen, &font, &prompts);
                }
            }

            screen.spawn(version_label(&font));
        });
}

/// Animación de respiración y realce de escala para la opción seleccionada.
fn animate_welcome_cards(
    time: Res<Time>,
    welcome: Res<Welcome>,
    mut query: Query<(&WelcomeCard, &mut Transform)>,
) {
    if welcome.page != Page::Main {
        return;
    }
    for (card, mut transform) in &mut query {
        if card.index == welcome.selected {
            let pulse = 1.0 + 0.016 * (time.elapsed_secs() * 5.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
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

    #[test]
    fn the_welcome_screen_offers_what_was_asked() {
        if WEB {
            assert_eq!(
                items(Page::Main),
                [Item::SinglePlayer, Item::SplitScreen, Item::Options]
            );
        } else {
            assert_eq!(
                items(Page::Main),
                [
                    Item::SinglePlayer,
                    Item::SplitScreen,
                    Item::Options,
                    Item::Quit
                ]
            );
        }
        assert_eq!(
            items(Page::QuitConfirm),
            [Item::CancelQuit, Item::ConfirmQuit]
        );
    }

    #[test]
    fn the_welcome_screen_carries_the_version() {
        let version = concat!("v", env!("CARGO_PKG_VERSION"));
        assert!(version.starts_with('v'));
        assert_eq!(version, format!("v{}", env!("CARGO_PKG_VERSION")));
    }
}
