//! Elegir pista: la pista se ve desde lo alto, girando despacio, con su
//! nombre, una descripción, su longitud y un plano del trazado. Izquierda y
//! derecha cambian de pista (y el mundo se vuelve a construir), Enter o A
//! pasan a elegir moto, y Esc o B vuelven a la bienvenida.

use bevy::prelude::*;

use crate::bike::ROSTER;
use crate::camera::CameraGlide;
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::hud::UiFont;
use crate::menu::GameState;
use crate::minimap::TrackMap;
use crate::track::Track;
use crate::ui::{MenuInput, MenuSound, gamepad_status, spawn_title, text_font};

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

/// Punto que marca qué pista se está viendo.
#[derive(Component)]
struct Dot(usize);

fn spawn_screen(mut commands: Commands, font: Res<UiFont>, map: Res<TrackMap>) {
    let soft = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8));
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
            spawn_title(screen, &font, "ELIGE PISTA", 44.0);
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
                        // Con una sola pista no hay nada que elegir con las flechas.
                        let choose = if CIRCUITS.len() > 1 {
                            "< >  elegir   ·   "
                        } else {
                            ""
                        };
                        text.spawn((
                            Text::new(format!(
                                "{choose}Enter o A: elegir moto   ·   Esc o B: volver"
                            )),
                            text_font(&font, 18.0),
                            soft,
                        ));
                    });
                });
            screen.spawn(gamepad_status(&font));
        });
}

/// Izquierda y derecha cambian de pista; Enter, Espacio, A o Start pasan a
/// elegir moto, y Esc o B vuelven a la bienvenida.
fn choose_track(
    mut input: MenuInput,
    mut choice: ResMut<CurrentCircuit>,
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
        next.set(GameState::Welcome);
    }
}

/// Nombre, descripción, longitud y puntos de la pista elegida.
fn show_track(
    choice: Res<CurrentCircuit>,
    track: Res<Track>,
    mut name: Single<&mut Text, NameFilter>,
    mut description: Single<&mut Text, DescriptionFilter>,
    mut length: Single<&mut Text, LengthFilter>,
    mut dots: Query<(&Dot, &mut BackgroundColor)>,
) {
    let circuit = choice.get();
    if name.0 != circuit.name {
        name.0 = circuit.name.to_string();
        description.0 = circuit.description.to_string();
    }
    let kilometres = format!("{:.1}", track.lap_length() / 1000.0).replace('.', ",");
    let label = format!("{kilometres} km por vuelta   ·   {} motos", ROSTER.len());
    if length.0 != label {
        length.0 = label;
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
