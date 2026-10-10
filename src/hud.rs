//! Marcador en pantalla: velocímetro y leyenda de los controles (oculta de
//! entrada; se enseña desde Depuración) en la carrera, y fotogramas por
//! segundo siempre. El minimapa va aparte, en [`crate::minimap`].

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::bike::{BikeState, Player};
use crate::locale::Language;
use crate::menu::GameState;
use crate::pause::DebugOptions;
use crate::settings::{Action, Controls, Settings};

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
                    (update_speedometer, show_legend).run_if(in_state(GameState::Race)),
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
struct Speedometer;

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

/// La ayuda de los controles, con las teclas que haya elegido el jugador.
fn help(controls: &Controls, lang: Language) -> String {
    let key = |action| controls.first_key(action);
    match lang {
        Language::Spanish => format!(
            "{}/{} acelerar y frenar  |  {}/{} girar  |  {} caballito  |  {} reiniciar  |  {} pausa  |  M sonido",
            key(Action::Throttle),
            key(Action::Brake),
            key(Action::Left),
            key(Action::Right),
            key(Action::Wheelie),
            key(Action::Restart),
            key(Action::Pause),
        ),
        Language::English => format!(
            "{}/{} accelerate and brake  |  {}/{} steer  |  {} wheelie  |  {} restart  |  {} pause  |  M sound",
            key(Action::Throttle),
            key(Action::Brake),
            key(Action::Left),
            key(Action::Right),
            key(Action::Wheelie),
            key(Action::Restart),
            key(Action::Pause),
        ),
    }
}

fn spawn_race_hud(
    mut commands: Commands,
    font: Res<UiFont>,
    settings: Res<Settings>,
    options: Res<DebugOptions>,
) {
    commands.spawn((
        DespawnOnExit(GameState::Race),
        Legend,
        legend_visibility(&options),
        Text::new(help(&settings.controls, settings.language)),
        TextFont {
            font: font.0.clone().into(),
            font_size: FontSize::Px(17.0),
            ..default()
        },
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
    commands.spawn((
        DespawnOnExit(GameState::Race),
        Speedometer,
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

fn legend_visibility(options: &DebugOptions) -> Visibility {
    if options.show_legend {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// La leyenda se ve u oculta según la opción de depuración.
fn show_legend(options: Res<DebugOptions>, mut legend: Query<&mut Visibility, With<Legend>>) {
    if !options.is_changed() {
        return;
    }
    for mut visibility in &mut legend {
        *visibility = legend_visibility(&options);
    }
}

fn update_speedometer(
    bike: Single<&BikeState, With<Player>>,
    mut text: Single<&mut Text, With<Speedometer>>,
) {
    let label = format!("{:.0} km/h", bike.speed.abs() * 3.6);
    // Solo escribir si cambia: tocar el texto obliga a recolocarlo.
    if text.0 != label {
        text.0 = label;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_help_shows_the_chosen_keys() {
        let mut controls = Controls::default();
        assert_eq!(
            help(&controls, Language::Spanish),
            "W/S acelerar y frenar  |  A/D girar  |  Espacio caballito  |  R reiniciar  |  Esc pausa  |  M sonido"
        );
        controls.bind_key(Action::Wheelie, KeyCode::KeyJ);
        assert!(help(&controls, Language::Spanish).contains("J caballito"));
        assert!(help(&controls, Language::English).contains("J wheelie"));
    }
}
