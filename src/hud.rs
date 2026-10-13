//! Marcador en pantalla: velocímetro y leyenda de los controles (oculta de
//! entrada; se enseña desde Depuración) en la carrera, y fotogramas por
//! segundo siempre. El minimapa va aparte, en [`crate::minimap`].

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::bike::{BikeState, LocalPlayer, Player};
use crate::gamepads::ActiveController;
use crate::legend::{KeyPrompt, race_prompts, spawn_legend};
use crate::menu::GameState;
use crate::pause::{DebugOptions, Pause};
use crate::settings::Settings;
use crate::ui::{InputDevice, UiIcons, current_active_device};
use crate::welcome::GameMode;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<UiFont>()
            .add_systems(Startup, spawn_fps_counter)
            .add_systems(OnEnter(GameState::Race), spawn_race_hud)
            .add_systems(
                Update,
                (
                    (update_speedometer, show_race_legend).run_if(in_state(GameState::Race)),
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

#[derive(Component)]
struct Speedometer(pub usize);

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
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Speedometer(0),
            Text::new("0 km/h"),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(38.0),
                ..default()
            },
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: percent(50.0),
                right: px(24),
                margin: UiRect::bottom(px(10.0)),
                ..default()
            },
        ));

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
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Speedometer(1),
            Text::new("0 km/h"),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(38.0),
                ..default()
            },
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                right: px(24),
                ..default()
            },
        ));
    } else {
        // Velocímetro modo 1 jugador
        commands.spawn((
            DespawnOnExit(GameState::Race),
            Speedometer(0),
            Text::new("0 km/h"),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(48.0),
                ..default()
            },
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                right: px(24),
                ..default()
            },
        ));
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

fn update_speedometer(
    p1_bike: Query<&BikeState, With<Player>>,
    other_bikes: Query<(&BikeState, &LocalPlayer)>,
    mut speedometers: Query<(&mut Text, &Speedometer)>,
) {
    let p1_speed = p1_bike
        .iter()
        .next()
        .map(|b| b.speed.abs() * 3.6)
        .unwrap_or(0.0);
    let p2_speed = other_bikes
        .iter()
        .find(|(_, lp)| lp.0 == 1)
        .map(|(b, _)| b.speed.abs() * 3.6)
        .unwrap_or(0.0);

    for (mut text, sp) in &mut speedometers {
        let speed = if sp.0 == 0 { p1_speed } else { p2_speed };
        let label = format!("{speed:.0} km/h");
        if text.0 != label {
            text.0 = label;
        }
    }
}

/// Fotogramas por segundo, suavizados para que se puedan leer.
fn update_fps(diagnostics: Res<DiagnosticsStore>, mut text: Single<&mut Text, With<FpsCounter>>) {
    let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    else {
        return;
    };
    let label = format!("{fps:.0} FPS");
    if text.0 != label {
        text.0 = label;
    }
}
