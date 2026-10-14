//! Marcador en pantalla: velocímetro y leyenda de los controles (oculta de
//! entrada; se enseña desde Depuración) en la carrera, y fotogramas por
//! segundo siempre. El minimapa va aparte, en [`crate::minimap`], and the
//! speedometer's dial is drawn in [`crate::speedometer`].

use bevy::prelude::*;

use crate::gamepads::ActiveController;
use crate::legend::{KeyPrompt, race_prompts, spawn_legend};
use crate::menu::GameState;
use crate::pause::{DebugOptions, Pause};
use crate::settings::Settings;
use crate::speedometer::{SpeedometerPlugin, spawn_speedometer};
use crate::ui::{InputDevice, UiIcons, current_active_device};
use crate::welcome::GameMode;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpeedometerPlugin)
            .init_resource::<UiFont>()
            .add_systems(Startup, spawn_fps_counter)
            .add_systems(OnEnter(GameState::Race), spawn_race_hud)
            .add_systems(
                Update,
                (
                    show_race_legend.run_if(in_state(GameState::Race)),
                    update_fps,
                ),
            );
    }
}

/// Fuente de los rótulos: Russo One, con tildes y eñes (la que trae Bevy
/// solo tiene ASCII).
#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

impl FromWorld for UiFont {
    fn from_world(world: &mut World) -> Self {
        let font =
            Font::from_bytes(include_bytes!("../assets/fonts/RussoOne-Regular.ttf").to_vec());
        Self(world.resource_mut::<Assets<Font>>().add(font))
    }
}

/// La leyenda con los controles.
#[derive(Component)]
struct Legend;

#[derive(Component)]
struct FpsCounter;

fn spawn_fps_counter(mut commands: Commands) {
    commands.spawn((
        FpsCounter,
        Text::new("-- FPS"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(16),
            ..default()
        },
    ));
}

/// How wide the speedometer is (px): alone, and with two players, in each
/// half of the screen.
const DIAL: f32 = 220.0;
const SPLIT_DIAL: f32 = 160.0;

fn spawn_race_hud(mut commands: Commands, font: Res<UiFont>, game_mode: Res<GameMode>) {
    if *game_mode == GameMode::SplitScreen {
        // Línea divisoria central entre ambos jugadores
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Node {
                position_type: PositionType::Absolute,
                top: percent(50.0),
                left: px(0),
                width: percent(100.0),
                height: px(4),
                margin: UiRect::top(px(-2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        ));

        // Indicador Jugador 1 (Arriba)
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Text::new("1P"),
            TextColor(Color::srgb(0.3, 0.8, 1.0)),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                right: px(90),
                ..default()
            },
        ));

        // Velocímetro Jugador 1 (abajo a la derecha de la mitad superior)
        spawn_speedometer(
            &mut commands,
            0,
            SPLIT_DIAL,
            Node {
                position_type: PositionType::Absolute,
                bottom: percent(50.0),
                right: px(16),
                margin: UiRect::bottom(px(10.0)),
                ..default()
            },
            &font.0,
        );

        // Indicador Jugador 2 (Abajo)
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Text::new("2P"),
            TextColor(Color::srgb(1.0, 0.4, 0.4)),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                top: percent(50.0),
                right: px(90),
                margin: UiRect::top(px(12.0)),
                ..default()
            },
        ));

        // Velocímetro Jugador 2 (abajo a la derecha de la mitad inferior)
        spawn_speedometer(
            &mut commands,
            1,
            SPLIT_DIAL,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                right: px(16),
                ..default()
            },
            &font.0,
        );
    } else {
        // Velocímetro modo 1 jugador
        spawn_speedometer(
            &mut commands,
            0,
            DIAL,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                right: px(16),
                ..default()
            },
            &font.0,
        );
    }
}

/// La leyenda se ve u oculta según la opción de Opciones y se adapta al
/// mando activo y a los controles elegidos. In the pause menu it gives way
/// to the menu's own legend.
#[allow(clippy::too_many_arguments)]
fn show_race_legend(
    mut commands: Commands,
    options: Res<DebugOptions>,
    settings: Res<Settings>,
    pause: Res<Pause>,
    device: Option<Res<InputDevice>>,
    active_controller: Option<Res<ActiveController>>,
    font: Res<UiFont>,
    icons: Res<UiIcons>,
    shown: Query<Entity, With<Legend>>,
    mut last: Local<Option<Vec<KeyPrompt>>>,
) {
    let active_dev = current_active_device(
        device.as_deref().copied().unwrap_or_default(),
        active_controller.as_deref(),
    );
    let wanted = (options.show_legend && !pause.is_open())
        .then(|| race_prompts(active_dev, settings.language, &settings.controls));
    if *last == wanted && shown.is_empty() == wanted.is_none() {
        return;
    }
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    *last = wanted.clone();
    let Some(prompts) = wanted else {
        return;
    };

    commands
        .spawn((
            DespawnOnExit(GameState::Race),
            Legend,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            GlobalZIndex(5),
        ))
        .with_children(|screen| {
            spawn_legend(screen, &font, &icons, active_dev, &prompts);
        });
}

/// Fotogramas por segundo, suavizados para que se puedan leer.
/// How often the frame rate shown changes (s): it is the average over that
/// time, so it reads steadily instead of flickering frame by frame.
const FPS_EVERY: f32 = 0.5;

/// Frames counted since the frame rate shown last changed, and over how
/// long.
#[derive(Default)]
struct FpsMeter {
    frames: u32,
    elapsed: f32,
}

impl FpsMeter {
    /// Counts a frame that took `dt` s: every [`FPS_EVERY`], the average
    /// frame rate since the last.
    fn tick(&mut self, dt: f32) -> Option<f32> {
        self.frames += 1;
        self.elapsed += dt;
        if self.elapsed < FPS_EVERY {
            return None;
        }
        let fps = self.frames as f32 / self.elapsed;
        *self = Self::default();
        Some(fps)
    }
}

fn update_fps(
    time: Res<Time<Real>>,
    mut meter: Local<FpsMeter>,
    mut text: Single<&mut Text, With<FpsCounter>>,
) {
    let Some(fps) = meter.tick(time.delta_secs()) else {
        return;
    };
    let label = format!("{fps:.0} FPS");
    if text.0 != label {
        text.0 = label;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_rate_is_shown_as_an_average_twice_a_second() {
        let mut meter = FpsMeter::default();
        // 60 frames a second for two seconds: shown four times.
        let shown: Vec<f32> = (0..120).filter_map(|_| meter.tick(1.0 / 60.0)).collect();
        assert_eq!(shown.len(), 4, "{shown:?}");
        assert!(
            shown.iter().all(|fps| (fps - 60.0).abs() < 2.5),
            "{shown:?}"
        );
        // Uneven frames read as their average.
        let mut meter = FpsMeter::default();
        let shown = (0..40)
            .filter_map(|k| meter.tick(if k % 2 == 0 { 0.01 } else { 0.03 }))
            .next()
            .expect("shown after half a second");
        assert!((shown - 50.0).abs() < 1.0, "{shown}");
    }
}
