//! Cámara de persecución: detrás y por encima de la moto, siguiendo sus giros
//! con algo de retraso, quedándose atrás al acelerar y abriendo el ángulo de
//! visión con la velocidad. El stick derecho del mando dice hacia dónde mira
//! respecto a la moto: arriba, hacia delante como siempre; a un lado, a ese
//! lado, y abajo, hacia atrás. Al soltarlo vuelve a mirar hacia delante.

use std::ops::Range;

use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::bike::{BikeState, BikeTuning, CrashState, Knock, LocalPlayer, Player};
use crate::geometry::{approach, smoothstep, wrap_angle};
use crate::lobby::{ControllerBinding, LocalPlayersConfig};
use crate::menu::GameState;
use crate::welcome::GameMode;

/// Color del horizonte en la foto del cielo: el de la neblina de la distancia.
const HORIZON: Color = Color::srgb(0.6, 0.66, 0.78);
/// Hasta dónde se ve (m): más allá del cielo, que está a 4,5 km.
const FAR: f32 = 6000.0;
/// Distancia detrás de la moto: parada .. a toda velocidad.
const DISTANCE: Range<f32> = 3.4..4.2;
const HEIGHT: f32 = 1.55;
/// La cámara mira a un punto por delante de la moto, no a la moto.
const LOOK_AHEAD: f32 = 5.0;
const LOOK_HEIGHT: f32 = 1.0;
/// Campo de visión vertical: parada .. a toda velocidad. Abrirlo con la
/// velocidad exagera la sensación de rapidez.
const FOV: Range<f32> = 1.05..1.35;
/// Rapidez con la que la cámara se pone detrás de la moto al girar (1/s).
const YAW_RESPONSE: f32 = 5.0;
/// Como si la cámara también tuviera inercia, se queda atrás al acelerar y se
/// echa encima al frenar (m por cada m/s²).
const SURGE_PER_ACCEL: f32 = 0.035;
/// Lo más que se acerca .. se aleja por la inercia (m).
const SURGE: Range<f32> = -0.6..0.45;
const SURGE_RESPONSE: f32 = 3.0;
/// Cuánto hay que inclinar el stick derecho para que la cámara mire hacia
/// donde apunta; por debajo, mira hacia delante.
const LOOK_DEADZONE: f32 = 0.4;
/// Rapidez con la que la cámara gira hacia donde apunta el stick derecho y
/// vuelve al soltarlo (1/s).
const LOOK_RESPONSE: f32 = 6.0;
/// Lo que tarda la cámara en ir de una pantalla de menú a la siguiente (s).
const GLIDE_TIME: f32 = 1.2;
/// How fast the camera comes back upright after leaving a loop the wrong
/// way, falling off it (1/s).
const UPRIGHT_RESPONSE: f32 = 6.0;

pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        // Entre los menús la cámara se desliza de un sitio a otro.
        for (from, to) in [
            (GameState::Welcome, GameState::Tracks),
            (GameState::Tracks, GameState::Welcome),
            (GameState::Welcome, GameState::Lobby),
            (GameState::Lobby, GameState::Welcome),
            (GameState::Lobby, GameState::Tracks),
            (GameState::Tracks, GameState::Lobby),
            (GameState::Tracks, GameState::Select),
            (GameState::Select, GameState::Tracks),
        ] {
            app.add_systems(
                OnTransition {
                    exited: from,
                    entered: to,
                },
                start_glide,
            );
        }
        app.insert_resource(ClearColor(HORIZON))
            .add_systems(Startup, spawn_camera)
            .init_resource::<CameraGlide>()
            .add_systems(OnEnter(GameState::Select), setup_split_cameras)
            .add_systems(OnEnter(GameState::Race), (setup_split_cameras, reset_chase))
            .add_systems(OnEnter(GameState::Tracks), cleanup_camera_viewports)
            .add_systems(OnEnter(GameState::Welcome), cleanup_camera_viewports)
            .add_systems(OnEnter(GameState::Lobby), cleanup_camera_viewports)
            .add_systems(
                Update,
                (
                    update_camera_viewports
                        .run_if(in_state(GameState::Select).or_else(in_state(GameState::Race))),
                    follow_player.run_if(in_state(GameState::Race)),
                ),
            );
    }
}

/// A qué jugador sigue esta cámara (0 para Player 1, 1 para Player 2).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CameraPlayer(pub usize);

#[derive(Component, Default)]
struct ChaseCamera {
    /// Orientación actual de la cámara; `None` hasta el primer fotograma.
    yaw: Option<f32>,
    /// Distancia extra por la inercia (m).
    surge: f32,
    /// Hacia dónde mira respecto a la moto (rad, positivo a la izquierda),
    /// ya suavizado.
    look: f32,
    /// How far the camera is turned head over heels with the bike, about
    /// its own sideways axis: round a loop it goes round with the bike.
    /// `None` is upright.
    tilt: Option<Quat>,
}

/// El stick derecho del mando que más se esté moviendo (x a la derecha, y
/// hacia arriba), o cero.
pub fn right_stick(gamepads: &Query<&Gamepad>) -> Vec2 {
    gamepads
        .iter()
        .map(|gamepad| {
            let axis = |axis| gamepad.get(axis).unwrap_or(0.0);
            Vec2::new(
                axis(GamepadAxis::RightStickX),
                axis(GamepadAxis::RightStickY),
            )
        })
        .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
        .unwrap_or(Vec2::ZERO)
}

fn gamepad_right_stick(gamepads: &Query<&Gamepad>, entity: Entity) -> Vec2 {
    if let Ok(gamepad) = gamepads.get(entity) {
        let axis = |a| gamepad.get(a).unwrap_or(0.0);
        Vec2::new(
            axis(GamepadAxis::RightStickX),
            axis(GamepadAxis::RightStickY),
        )
    } else {
        Vec2::ZERO
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV.start,
            far: FAR,
            ..default()
        }),
        // Neblina del color del horizonte: lo lejano se funde con el cielo.
        DistanceFog {
            color: HORIZON,
            falloff: FogFalloff::Linear {
                start: 450.0,
                end: 3800.0,
            },
            ..default()
        },
        ChaseCamera::default(),
        CameraPlayer(0),
    ));

    // Cámara dedicada a la interfaz (UI) para que la maquetación siempre cubra
    // la ventana completa, independientemente de los viewports de pantalla dividida 3D.
    commands.spawn((
        Camera2d,
        Camera {
            order: 100,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        bevy::ui::IsDefaultUiCamera,
    ));
}

/// En modo pantalla dividida, crea la segunda cámara para el Jugador 2 (abajo).
fn setup_split_cameras(
    mut commands: Commands,
    game_mode: Res<GameMode>,
    existing: Query<Entity, (With<CameraPlayer>, With<Camera3d>)>,
) {
    if *game_mode == GameMode::SplitScreen && existing.iter().count() < 2 {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: FOV.start,
                far: FAR,
                ..default()
            }),
            DistanceFog {
                color: HORIZON,
                falloff: FogFalloff::Linear {
                    start: 450.0,
                    end: 3800.0,
                },
                ..default()
            },
            ChaseCamera::default(),
            CameraPlayer(1),
        ));
    }
}

/// Divide la pantalla en dos viewports horizontales:
/// Jugador 1 arriba (0 .. alto/2), Jugador 2 abajo (alto/2 .. alto).
fn update_camera_viewports(
    windows: Query<&Window, With<PrimaryWindow>>,
    game_mode: Res<GameMode>,
    mut cameras: Query<(&CameraPlayer, &mut Camera)>,
) {
    if *game_mode == GameMode::SplitScreen {
        let Some(window) = windows.iter().next() else {
            return;
        };
        let width = window.physical_width();
        let height = window.physical_height();
        if width == 0 || height == 0 {
            return;
        }

        let half_height = height / 2;
        for (cam_player, mut camera) in &mut cameras {
            camera.is_active = true;
            if cam_player.0 == 0 {
                camera.order = 0;
                camera.viewport = Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(width, half_height),
                    depth: 0.0..1.0,
                });
            } else if cam_player.0 == 1 {
                camera.order = 1;
                camera.viewport = Some(Viewport {
                    physical_position: UVec2::new(0, half_height),
                    physical_size: UVec2::new(width, height.saturating_sub(half_height)),
                    depth: 0.0..1.0,
                });
            }
        }
    } else {
        for (cam_player, mut camera) in &mut cameras {
            if cam_player.0 == 0 {
                camera.order = 0;
                camera.viewport = None;
                camera.is_active = true;
            } else {
                camera.is_active = false;
            }
        }
    }
}

/// Al salir a menús compartidos de pantalla completa, restaura la cámara principal y elimina la secundaria.
fn cleanup_camera_viewports(
    mut commands: Commands,
    mut cameras: Query<(Entity, &CameraPlayer, &mut Camera), With<Camera3d>>,
) {
    for (entity, cam_player, mut camera) in &mut cameras {
        if cam_player.0 == 0 {
            camera.order = 0;
            camera.viewport = None;
            camera.is_active = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
}
/// Para que la cámara vaya poco a poco de donde estaba al sitio que le toca
/// en la pantalla nueva, en lugar de saltar: dónde estaba, su campo de
/// visión y cuánto hace que empezó a moverse (s).
#[derive(Resource, Default)]
pub struct CameraGlide {
    from: Option<(Transform, f32)>,
    elapsed: f32,
}

impl CameraGlide {
    /// Pone la cámara en `wanted` con campo de visión `fov`, o de camino hacia
    /// allí si acaba de cambiar de pantalla.
    pub fn apply(
        &mut self,
        transform: &mut Transform,
        projection: &mut Projection,
        wanted: Transform,
        fov: f32,
        dt: f32,
    ) {
        let (place, angle) = match self.from {
            Some((from, from_fov)) => {
                self.elapsed += dt;
                let t = smoothstep(0.0, GLIDE_TIME, self.elapsed);
                if t >= 1.0 {
                    self.from = None;
                }
                (
                    Transform {
                        translation: from.translation.lerp(wanted.translation, t),
                        rotation: from.rotation.slerp(wanted.rotation, t),
                        ..wanted
                    },
                    from_fov.lerp(fov, t),
                )
            }
            None => (wanted, fov),
        };
        *transform = place;
        if let Projection::Perspective(perspective) = projection {
            perspective.fov = angle;
        }
    }
}

#[allow(clippy::type_complexity)]
fn start_glide(
    cameras: Query<(&Transform, &Projection), (With<Camera3d>, With<ChaseCamera>)>,
    mut glide: ResMut<CameraGlide>,
) {
    let Some((transform, projection)) = cameras.iter().next() else {
        return;
    };
    let fov = match projection {
        Projection::Perspective(perspective) => perspective.fov,
        _ => FOV.start,
    };
    *glide = CameraGlide {
        from: Some((*transform, fov)),
        elapsed: 0.0,
    };
}

/// Al empezar la carrera, la cámara se coloca de golpe detrás del jugador.
fn reset_chase(mut cameras: Query<&mut ChaseCamera>) {
    for mut camera in &mut cameras {
        *camera = ChaseCamera::default();
    }
}

/// Hacia dónde apunta el stick derecho respecto a la moto (rad, positivo a
/// la izquierda: arriba 0, a la derecha -π/2 y abajo ±π), o `None` si apenas
/// se inclina.
fn stick_direction(stick: Vec2) -> Option<f32> {
    (stick.length() > LOOK_DEADZONE).then(|| (-stick.x).atan2(stick.y))
}

/// Dónde va la cámara y a qué punto mira, a `distance` de la moto en `target`
/// orientada hacia `heading`: detrás de la moto según esa dirección, mirando
/// por delante de ella.
#[cfg(test)]
fn chase_view(target: Vec3, heading: f32, distance: f32) -> (Vec3, Vec3) {
    let (position, look_at, _) = tilted_chase_view(target, heading, distance, Quat::IDENTITY);
    (position, look_at)
}

/// Like [`chase_view`], with the view turned by `tilt` (in the frame of
/// the bike, after its heading): behind and above the bike in that frame.
/// Also returns which way is up for the camera.
fn tilted_chase_view(target: Vec3, heading: f32, distance: f32, tilt: Quat) -> (Vec3, Vec3, Vec3) {
    let frame = Quat::from_rotation_y(heading) * tilt;
    let (behind, up) = (frame * Vec3::Z, frame * Vec3::Y);
    let position = target + behind * distance + up * HEIGHT;
    (
        position,
        target - behind * LOOK_AHEAD + up * LOOK_HEIGHT,
        up,
    )
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn follow_player(
    time: Res<Time>,
    tuning: Res<BikeTuning>,
    gamepads: Query<&Gamepad>,
    game_mode: Res<GameMode>,
    local_players: Res<LocalPlayersConfig>,
    player_bikes: Query<
        (&Transform, &BikeState, Option<(&Knock, &CrashState)>),
        (With<Player>, Without<ChaseCamera>),
    >,
    other_bikes: Query<
        (
            &Transform,
            &BikeState,
            Option<(&Knock, &CrashState)>,
            &LocalPlayer,
        ),
        Without<ChaseCamera>,
    >,
    mut cameras: Query<(
        &mut Transform,
        &mut Projection,
        &mut ChaseCamera,
        &CameraPlayer,
    )>,
) {
    for (mut transform, mut projection, mut chase, cam_player) in &mut cameras {
        let bike_data = if cam_player.0 == 0 {
            player_bikes.iter().next()
        } else {
            other_bikes
                .iter()
                .find(|(.., lp)| lp.0 == cam_player.0)
                .map(|(t, s, c, _)| (t, s, c))
        };

        let Some((bike_transform, state, crash)) = bike_data else {
            continue;
        };
        // While the rider is down the bike tumbles and spins: the camera
        // keeps looking the way it was, and pulls in as the bike slows down.
        let down = crash.is_some_and(|(_, crash)| crash.is_down());
        let speed = match crash {
            Some((knock, crash)) if crash.is_down() => knock.velocity.length(),
            _ => state.speed.abs(),
        };

        // El Transform de la moto ya viene interpolado y solo lleva guiñada (la
        // inclinación va en un hijo) y cabeceo. Round a loop the pitch goes
        // past the vertical, so the heading comes from the bike itself.
        let (bike_yaw, _, _) = bike_transform.rotation.to_euler(EulerRot::YXZ);
        let bike_yaw = if state.ride.is_some() {
            state.heading
        } else {
            bike_yaw
        };
        let yaw = match chase.yaw {
            Some(yaw) if down => yaw,
            Some(yaw) => {
                let blend = 1.0 - (-YAW_RESPONSE * time.delta_secs()).exp();
                yaw + wrap_angle(bike_yaw - yaw) * blend
            }
            None => bike_yaw,
        };
        chase.yaw = Some(yaw);

        // Stick derecho: la cámara mira hacia donde apunta, por el camino más
        // corto, y al soltarlo vuelve a mirar hacia delante.
        let dt = time.delta_secs();
        let stick = if *game_mode == GameMode::SplitScreen {
            let binding = if cam_player.0 == 0 {
                local_players.p1
            } else {
                local_players.p2
            };
            match binding {
                Some(ControllerBinding::Gamepad(entity)) => gamepad_right_stick(&gamepads, entity),
                _ => Vec2::ZERO,
            }
        } else {
            right_stick(&gamepads)
        };

        let wanted = stick_direction(stick).unwrap_or(0.0);
        let blend = 1.0 - (-LOOK_RESPONSE * dt).exp();
        chase.look = wrap_angle(chase.look + wrap_angle(wanted - chase.look) * blend);

        let speed_factor = (speed / tuning.max_speed).clamp(0.0, 1.0);
        let surge = (state.accel * SURGE_PER_ACCEL).clamp(SURGE.start, SURGE.end);
        chase.surge = approach(chase.surge, surge, SURGE_RESPONSE, dt);
        let distance = DISTANCE.start.lerp(DISTANCE.end, speed_factor) + chase.surge;
        // Round a loop the camera turns over with the bike, behind and above
        // it in its own frame; off it, it comes back upright.
        chase.tilt = if state.ride.is_some() {
            Some(Quat::from_rotation_y(-state.heading) * bike_transform.rotation)
        } else {
            chase.tilt.and_then(|tilt| {
                let blend = 1.0 - (-UPRIGHT_RESPONSE * dt).exp();
                let tilt = tilt.slerp(Quat::IDENTITY, blend);
                (tilt.angle_between(Quat::IDENTITY) > 1e-3).then_some(tilt)
            })
        };
        let (position, look_at, up) = tilted_chase_view(
            bike_transform.translation,
            yaw + chase.look,
            distance,
            chase.tilt.unwrap_or(Quat::IDENTITY),
        );
        transform.translation = position;
        transform.look_at(look_at, up);

        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = FOV.start.lerp(FOV.end, speed_factor);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

    #[test]
    fn the_right_stick_points_where_the_camera_looks() {
        let direction = |x, y| stick_direction(Vec2::new(x, y));
        // Arriba, hacia delante; a la derecha, a la derecha (ángulo negativo);
        // abajo, hacia atrás.
        assert_eq!(direction(0.0, 1.0), Some(0.0));
        assert!((direction(1.0, 0.0).unwrap() + FRAC_PI_2).abs() < 1e-6);
        assert!((direction(-1.0, 0.0).unwrap() - FRAC_PI_2).abs() < 1e-6);
        assert!((direction(0.0, -1.0).unwrap().abs() - PI).abs() < 1e-6);
        // Apenas inclinado no cuenta.
        assert_eq!(direction(0.2, -0.2), None);
    }

    #[test]
    fn the_camera_looks_past_the_bike_where_it_points() {
        // Moto parada en el origen mirando a -Z: su derecha es +X.
        let view = |look| chase_view(Vec3::ZERO, look, 4.0);
        // Hacia delante: detrás de la moto, mirando por delante de ella.
        let (camera, looking) = view(0.0);
        assert!(camera.z > 3.9 && camera.x.abs() < 1e-4);
        assert!(looking.z < 0.0);
        // A la derecha: la cámara a la izquierda de la moto, mirando a la derecha.
        let (camera, looking) = view(-FRAC_PI_2);
        assert!(camera.x < -3.9 && looking.x > 0.0);
        // Hacia atrás: la cámara delante de la moto, mirando hacia atrás.
        let (camera, looking) = view(PI);
        assert!(camera.z < -3.9 && looking.z > 0.0);
    }
}
