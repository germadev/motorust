//! Elegir pista: la pista se ve desde lo alto, girando despacio, con su
//! nombre, una descripción, su longitud y un plano del trazado. Izquierda y
//! derecha cambian de pista (y el mundo se vuelve a construir), Enter o A
//! pasan a elegir moto, y Esc o B vuelven a la bienvenida.

use bevy::prelude::*;

use crate::bike::ROSTER;
use crate::camera::CameraGlide;
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::hud::UiFont;
use crate::locale::Language;
use crate::menu::GameState;
use crate::minimap::TrackMap;
use crate::settings::Settings;
use crate::track::Track;
use crate::ui::{InputDevice, MenuInput, MenuSound, spawn_title, text_font};
use crate::welcome::GameMode;

/// La cámara gira alrededor de la pista a esta velocidad (rad/s), a esta
/// distancia del centro y a esta altura (fracciones del tamaño de la pista),
/// con este campo de visión (rad).
const AERIAL_SPEED: f32 = 0.06;
const AERIAL_DISTANCE: f32 = 0.7;
const AERIAL_HEIGHT: f32 = 0.5;
const AERIAL_FOV: f32 = 0.8;

pub struct TrackSelectPlugin;

impl Plugin for TrackSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Tracks), spawn_screen)
            .add_systems(
                Update,
                (choose_track, show_track, aerial_camera).run_if(in_state(GameState::Tracks)),
            );
    }
}

#[derive(Component)]
struct TrackName;

#[derive(Component)]
struct TrackDescription;

#[derive(Component)]
struct TrackLength;

#[derive(Component)]
struct TrackHint;

/// Filtros para tocar los textos a la vez.
type NameFilter = (
    With<TrackName>,
    Without<TrackDescription>,
    Without<TrackLength>,
);
type DescriptionFilter = (
    With<TrackDescription>,
    Without<TrackName>,
    Without<TrackLength>,
);
type LengthFilter = (
    With<TrackLength>,
    Without<TrackName>,
    Without<TrackDescription>,
);
type HintFilter = (
    With<TrackHint>,
    Without<TrackName>,
    Without<TrackDescription>,
    Without<TrackLength>,
);

/// Punto que marca qué pista se está viendo.
#[derive(Component)]
struct Dot(usize);

fn spawn_screen(
    mut commands: Commands,
    font: Res<UiFont>,
    map: Res<TrackMap>,
    settings: Res<Settings>,
) {
    let soft = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8));
    let lang = settings.language;
    commands
        .spawn((
            DespawnOnExit(GameState::Tracks),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                // Abajo, sitio para el aviso del mando.
                padding: UiRect {
                    left: px(24),
                    right: px(24),
                    top: px(20),
                    bottom: px(40),
                },
                ..default()
            },
        ))
        .with_children(|screen| {
            spawn_title(screen, &font, lang.select_track(), 44.0);
            // Flechas a los lados, si hay entre qué elegir.
            screen
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|arrows| {
                    if CIRCUITS.len() > 1 {
                        for arrow in ["<", ">"] {
                            arrows.spawn((
                                Text::new(arrow),
                                text_font(&font, 80.0),
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                                TextShadow::default(),
                            ));
                        }
                    }
                });
            screen
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(28),
                        padding: UiRect::axes(px(28), px(18)),
                        border_radius: BorderRadius::all(px(14)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.65)),
                ))
                .with_children(|card| {
                    card.spawn((
                        ImageNode::new(map.image.clone()),
                        Node {
                            width: px(190),
                            height: px(190),
                            ..default()
                        },
                    ));
                    card.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        row_gap: px(10),
                        ..default()
                    })
                    .with_children(|text| {
                        text.spawn((
                            TrackName,
                            Text::new(""),
                            text_font(&font, 46.0),
                            TextShadow::default(),
                        ));
                        text.spawn((TrackDescription, Text::new(""), text_font(&font, 20.0)));
                        text.spawn((
                            TrackLength,
                            Text::new(""),
                            text_font(&font, 20.0),
                            TextColor(Color::srgb(1.0, 0.8, 0.2)),
                        ));
                        text.spawn(Node {
                            column_gap: px(10),
                            margin: UiRect::vertical(px(4)),
                            ..default()
                        })
                        .with_children(|dots| {
                            for index in 0..CIRCUITS.len() {
                                dots.spawn((
                                    Dot(index),
                                    Node {
                                        width: px(12),
                                        height: px(12),
                                        border_radius: BorderRadius::MAX,
                                        ..default()
                                    },
                                    BackgroundColor(Color::WHITE),
                                ));
                            }
                        });
                        text.spawn((TrackHint, Text::new(""), text_font(&font, 18.0), soft));
                    });
                });
        });
}

/// Izquierda y derecha cambian de pista; Enter, Espacio, A o Start pasan a
/// elegir moto, y Esc o B vuelven a la bienvenida.
fn choose_track(
    mut input: MenuInput,
    mut choice: ResMut<CurrentCircuit>,
    game_mode: Res<GameMode>,
    mut next: ResMut<NextState<GameState>>,
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    if request.side != 0 && CIRCUITS.len() > 1 {
        choice.0 = (choice.0 as i32 + request.side).rem_euclid(CIRCUITS.len() as i32) as usize;
        sounds.write(MenuSound::Move);
    }
    if request.confirm || request.start {
        sounds.write(MenuSound::Confirm);
        next.set(GameState::Select);
    } else if request.back {
        sounds.write(MenuSound::Move);
        if *game_mode == GameMode::SplitScreen {
            next.set(GameState::Lobby);
        } else {
            next.set(GameState::Welcome);
        }
    }
}

/// Nombre, descripción, longitud, pista elegida y leyenda de controles.
#[allow(clippy::too_many_arguments)]
fn show_track(
    choice: Res<CurrentCircuit>,
    track: Res<Track>,
    settings: Res<Settings>,
    device: Res<InputDevice>,
    mut name: Single<&mut Text, NameFilter>,
    mut description: Single<&mut Text, DescriptionFilter>,
    mut length: Single<&mut Text, LengthFilter>,
    mut hint: Single<&mut Text, HintFilter>,
    mut dots: Query<(&Dot, &mut BackgroundColor)>,
) {
    let lang = settings.language;
    let circuit = choice.get();
    let wanted_name = circuit.name(lang);
    let wanted_description = circuit.description(lang);
    if name.0 != wanted_name {
        name.0 = wanted_name.to_string();
        description.0 = wanted_description.to_string();
    }
    let kilometres = match lang {
        Language::Spanish => format!("{:.1}", track.lap_length() / 1000.0).replace('.', ","),
        Language::English => format!("{:.1}", track.lap_length() / 1000.0),
    };
    let label = match lang {
        Language::Spanish => format!("{kilometres} km por vuelta   ·   {} motos", ROSTER.len()),
        Language::English => format!("{kilometres} km per lap   ·   {} bikes", ROSTER.len()),
    };
    if length.0 != label {
        length.0 = label;
    }
    let (confirm, back) = match *device {
        InputDevice::Keyboard => ("ENTER", "ESC"),
        InputDevice::Gamepad => ("A o START", "B"),
    };
    let choose = if CIRCUITS.len() > 1 {
        "< >  elegir   ·   "
    } else {
        ""
    };
    let wanted_hint = match lang {
        Language::Spanish => format!("{choose}{confirm}: elegir moto   ·   {back}: volver"),
        Language::English => {
            let (confirm_en, back_en) = match *device {
                InputDevice::Keyboard => ("ENTER", "ESC"),
                InputDevice::Gamepad => ("A or START", "B"),
            };
            let choose_en = if CIRCUITS.len() > 1 {
                "< >  choose   ·   "
            } else {
                ""
            };
            format!("{choose_en}{confirm_en}: choose bike   ·   {back_en}: back")
        }
    };
    if hint.0 != wanted_hint {
        hint.0 = wanted_hint;
    }
    for (dot, mut color) in &mut dots {
        let alpha = if dot.0 == choice.0 { 1.0 } else { 0.3 };
        let wanted = Color::srgba(1.0, 1.0, 1.0, alpha);
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
}

/// La cámara gira despacio alrededor de la pista, desde lo alto.
fn aerial_camera(
    time: Res<Time>,
    track: Res<Track>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut glide: ResMut<CameraGlide>,
    mut angle: Local<f32>,
) {
    let dt = time.delta_secs();
    *angle = (*angle + AERIAL_SPEED * dt) % std::f32::consts::TAU;
    let (min, max) = track.bounds();
    let center = (min + max) / 2.0;
    let size = (max - min).max_element();
    let around = Vec2::from_angle(*angle) * size * AERIAL_DISTANCE;
    let position = Vec3::new(
        center.x + around.x,
        size * AERIAL_HEIGHT,
        center.y + around.y,
    );
    let wanted = Transform::from_translation(position)
        .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y);
    let (mut transform, mut projection) = camera.into_inner();
    glide.apply(&mut transform, &mut projection, wanted, AERIAL_FOV, dt);
}
