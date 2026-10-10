//! Las pantallas del juego y la de elegir moto y piloto.
//!
//! Se empieza en la de bienvenida ([`crate::welcome`]), de ahí se pasa a
//! elegir pista ([`crate::track_select`]) y luego moto. Las motos de la
//! carrera esperan en la parrilla de salida y la cámara gira despacio
//! alrededor de la elegida; al cambiar de moto se desliza hasta la siguiente.
//! Al confirmar empieza la carrera con esa moto; con Esc o B se vuelve a
//! elegir pista.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::bike::{BikeAssets, BikeState, ROSTER};
use crate::camera::{CameraGlide, right_stick};
use crate::hud::UiFont;
use crate::locale::Language;
use crate::settings::Settings;
use crate::track::Track;
use crate::ui::{InputDevice, MenuInput, MenuSound};

/// En qué pantalla está el juego.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    /// Bienvenida, con las opciones.
    #[default]
    Welcome,
    /// Elegir jugador y control (2 jugadores locales).
    Lobby,
    /// Elegir pista.
    Tracks,
    /// Elegir moto y piloto.
    Select,
    Race,
}

/// La moto que ha elegido el jugador: su índice en [`ROSTER`].
#[derive(Resource, Default)]
pub struct Choice(pub usize);

/// Distancia y altura de la cámara alrededor de la moto, altura a la que
/// mira (m), lo deprisa que da la vuelta (rad/s) y su campo de visión (rad).
const ORBIT_DISTANCE: f32 = 5.0;
const ORBIT_HEIGHT: f32 = 2.2;
const LOOK_HEIGHT: f32 = 0.5;
const ORBIT_SPEED: f32 = 0.25;
const ORBIT_FOV: f32 = 0.65;
/// Lo deprisa que se gira la vista con el stick derecho a tope (rad/s).
const ORBIT_STICK_SPEED: f32 = 2.5;
/// Rapidez con la que la cámara llega a la moto elegida (1/s).
const GLIDE_RESPONSE: f32 = 4.0;
/// Desde dónde se empieza a ver cada moto: de tres cuartos por delante y a
/// la derecha (rad, desde detrás).
const FIRST_VIEW: f32 = 2.3;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Choice>()
            .add_systems(OnEnter(GameState::Select), (spawn_showcase, spawn_menu))
            .add_systems(
                Update,
                (choose, orbit_camera, show_choice).run_if(in_state(GameState::Select)),
            );
    }
}

/// Moto en exposición: su índice en [`ROSTER`].
#[derive(Component)]
struct Showcase(usize);

#[derive(Component)]
struct BikeName;

#[derive(Component)]
struct BikeDescription;

#[derive(Component)]
struct BikeHint;

/// Filtros para tocar los textos a la vez.
type NameFilter = (With<BikeName>, Without<BikeDescription>, Without<BikeHint>);
type DescriptionFilter = (With<BikeDescription>, Without<BikeName>, Without<BikeHint>);
type HintFilter = (With<BikeHint>, Without<BikeName>, Without<BikeDescription>);

/// Punto de la fila que marca qué moto se está viendo.
#[derive(Component)]
struct Dot(usize);

/// Las motos en la parrilla de salida, por orden.
fn spawn_showcase(mut commands: Commands, track: Res<Track>, mut assets: BikeAssets) {
    for (index, entry) in ROSTER.iter().enumerate() {
        let state = BikeState::parked_on(&track, track.grid_slot(index));
        let bike = assets.spawn(&mut commands, entry, state);
        commands
            .entity(bike)
            .insert((Showcase(index), DespawnOnExit(GameState::Select)));
    }
}

fn spawn_menu(mut commands: Commands, font: Res<UiFont>, settings: Res<Settings>) {
    let lang = settings.language;
    let text = |size: f32| TextFont {
        font: font.0.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    };
    let shadow = TextShadow {
        offset: Vec2::splat(3.0),
        color: Color::srgba(0.0, 0.0, 0.0, 0.6),
    };
    let panel = Color::srgba(0.02, 0.04, 0.1, 0.65);

    commands
        .spawn((
            DespawnOnExit(GameState::Select),
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
            screen.spawn((Text::new(lang.select_bike()), text(44.0), shadow));
            // Flechas a los lados.
            screen
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|arrows| {
                    for arrow in ["<", ">"] {
                        arrows.spawn((
                            Text::new(arrow),
                            text(80.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                            shadow,
                        ));
                    }
                });
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(10),
                        padding: UiRect::axes(px(40), px(18)),
                        border_radius: BorderRadius::all(px(14)),
                        ..default()
                    },
                    BackgroundColor(panel),
                ))
                .with_children(|card| {
                    card.spawn((BikeName, Text::new(""), text(46.0), shadow));
                    card.spawn((BikeDescription, Text::new(""), text(20.0)));
                    card.spawn(Node {
                        column_gap: px(10),
                        margin: UiRect::vertical(px(6)),
                        ..default()
                    })
                    .with_children(|dots| {
                        for index in 0..ROSTER.len() {
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
                    card.spawn((
                        BikeHint,
                        Text::new(""),
                        text(18.0),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));
                });
        });
}

/// Izquierda y derecha (teclado, cruceta o stick) cambian de moto; Enter,
/// Espacio, A o Start empiezan la carrera, y Esc o B vuelven a elegir pista.
fn choose(
    mut input: MenuInput,
    mut choice: ResMut<Choice>,
    mut next: ResMut<NextState<GameState>>,
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    if request.side != 0 {
        choice.0 = (choice.0 as i32 + request.side).rem_euclid(ROSTER.len() as i32) as usize;
    }
    if request.confirm || request.start {
        sounds.write(MenuSound::Confirm);
        next.set(GameState::Race);
    } else if request.back {
        sounds.write(MenuSound::Move);
        next.set(GameState::Tracks);
    }
}

/// Dónde mira la cámara y cuánto ha girado alrededor de la moto.
#[derive(Default)]
struct Orbit {
    focus: Option<Vec3>,
    angle: f32,
}

fn orbit_camera(
    time: Res<Time>,
    choice: Res<Choice>,
    gamepads: Query<&Gamepad>,
    bikes: Query<(&Showcase, &Transform), Without<Camera3d>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut glide: ResMut<CameraGlide>,
    mut orbit: Local<Orbit>,
) {
    let Some((_, bike)) = bikes.iter().find(|(showcase, _)| showcase.0 == choice.0) else {
        return;
    };
    let dt = time.delta_secs();
    let target = bike.translation;
    let focus = match orbit.focus {
        Some(focus) => focus.lerp(target, 1.0 - (-GLIDE_RESPONSE * dt).exp()),
        None => target,
    };
    orbit.focus = Some(focus);
    // Gira sola despacio y, con el stick derecho, hacia donde se empuje.
    let stick = right_stick(&gamepads).x;
    orbit.angle = (orbit.angle + (ORBIT_SPEED - ORBIT_STICK_SPEED * stick) * dt) % TAU;

    let (mut transform, mut projection) = camera.into_inner();
    let (heading, _, _) = bike.rotation.to_euler(EulerRot::YXZ);
    let around = Quat::from_rotation_y(heading + FIRST_VIEW + orbit.angle) * Vec3::Z;
    let wanted =
        Transform::from_translation(focus + around * ORBIT_DISTANCE + Vec3::Y * ORBIT_HEIGHT)
            .looking_at(focus + Vec3::Y * LOOK_HEIGHT, Vec3::Y);
    glide.apply(&mut transform, &mut projection, wanted, ORBIT_FOV, dt);
}

/// Nombre, descripción y puntos de la moto elegida.
fn show_choice(
    choice: Res<Choice>,
    settings: Res<Settings>,
    device: Res<InputDevice>,
    mut name: Single<(&mut Text, &mut TextColor), NameFilter>,
    mut description: Single<&mut Text, DescriptionFilter>,
    mut hint: Single<&mut Text, HintFilter>,
    mut dots: Query<(&Dot, &mut BackgroundColor)>,
) {
    let entry = &ROSTER[choice.0];
    let wanted_desc = entry.description(settings.language);
    if name.0.0 != entry.name {
        name.0.0 = entry.name.to_string();
        name.1.0 = entry.color.into();
        description.0 = wanted_desc.to_string();
    } else if description.0 != wanted_desc {
        description.0 = wanted_desc.to_string();
    }
    let (confirm, back) = match *device {
        InputDevice::Keyboard => ("ENTER", "ESC"),
        InputDevice::Gamepad => ("A o START", "B"),
    };
    let wanted_hint = match settings.language {
        Language::Spanish => format!("< >  elegir   ·   {confirm}: correr   ·   {back}: volver"),
        Language::English => {
            let (confirm_en, back_en) = match *device {
                InputDevice::Keyboard => ("ENTER", "ESC"),
                InputDevice::Gamepad => ("A or START", "B"),
            };
            format!("< >  choose   ·   {confirm_en}: race   ·   {back_en}: back")
        }
    };
    if hint.0 != wanted_hint {
        hint.0 = wanted_hint;
    }
    for (dot, mut color) in &mut dots {
        let alpha = if dot.0 == choice.0 { 1.0 } else { 0.3 };
        let wanted = Color::srgba(1.0, 1.0, 1.0, alpha);
        // Solo si cambia: tocar el color obliga a redibujar.
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
}
