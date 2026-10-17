//! Marcador en pantalla: velocímetro y leyenda de los controles (oculta de
//! entrada; se enseña desde Depuración) en la carrera, and the performance
//! HUD at the top right everywhere, as Options > Graphics > Performance HUD
//! says: the frames per second and the pixels the world is drawn at. El
//! minimapa va aparte, en [`crate::minimap`] (where the player is on it is
//! the performance HUD's too), and the speedometer's dial is drawn in
//! [`crate::speedometer`].

use bevy::prelude::*;

use crate::circuits::CurrentCircuit;
use crate::gamepads::ActiveController;
use crate::legend::{KeyPrompt, music_prompt, race_prompts, spawn_legend};
use crate::menu::GameState;
use crate::pause::{DebugOptions, Pause};
use crate::race::{Mode, Rules};
use crate::render_scale::{NativeShare, drawn_size, render_percent};
use crate::settings::Settings;
use crate::sound::music::FreeMusic;
use crate::speedometer::{SpeedometerPlugin, spawn_speedometer};
use crate::ui::{InputDevice, UiIcons, current_active_device};
use crate::welcome::GameMode;
use crate::world::detail::Detail;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpeedometerPlugin)
            .init_resource::<UiFont>()
            .add_systems(Startup, spawn_performance_hud)
            .add_systems(OnEnter(GameState::Race), spawn_race_hud)
            .add_systems(
                Update,
                (
                    show_race_legend.run_if(in_state(GameState::Race)),
                    (update_fps, show_resolution, keep_the_tag_clear),
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

/// The performance HUD, on a line at the top right: the pixels the world is
/// drawn at, and the frame rate after them.
#[derive(Component)]
struct PerformanceHud;
#[derive(Component)]
struct FpsCounter;
#[derive(Component)]
struct ResolutionShown;

/// How big the frame rate is (px), and how wide its room, always the same
/// ("999 FPS" in Bevy's font, whose figures are all as wide), so what
/// stands left of it does not move as it changes; how big the resolution
/// is, and how far from the frame rate.
const FPS_SIZE: f32 = 18.0;
const FPS_WIDTH: f32 = 7.0 * 0.6 * FPS_SIZE;
const RESOLUTION_SIZE: f32 = 15.0;
const RESOLUTION_GAP: f32 = 12.0;

fn spawn_performance_hud(mut commands: Commands) {
    commands
        .spawn((
            PerformanceHud,
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                right: px(16),
                align_items: AlignItems::FlexEnd,
                column_gap: px(RESOLUTION_GAP),
                ..default()
            },
        ))
        .with_children(|hud| {
            hud.spawn((
                ResolutionShown,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(RESOLUTION_SIZE),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
            hud.spawn((
                FpsCounter,
                Text::new("-- FPS"),
                TextFont {
                    font_size: FontSize::Px(FPS_SIZE),
                    ..default()
                },
                TextLayout::justify(Justify::Right),
                Node {
                    min_width: px(FPS_WIDTH),
                    ..default()
                },
            ));
        });
}

/// Shown or not, taking its room or not.
fn display(shown: bool) -> Display {
    if shown { Display::Flex } else { Display::None }
}

/// The pixels the world is drawn at in a window `window` pixels big, at
/// `percent` of its resolution (never more than the screen's own, see
/// [`NativeShare`]): "1920x1080", and the share when it is not all of it
/// ("1344x756 70%"); in Bevy's font, as the frame rate.
pub fn resolution_line(window: UVec2, percent: u32, native: NativeShare) -> String {
    let size = drawn_size(window, percent, native);
    if percent >= 100 {
        format!("{}x{}", size.x, size.y)
    } else {
        format!("{}x{} {percent}%", size.x, size.y)
    }
}

/// The performance HUD's resolution, if the options say so: the pixels the
/// world is drawn at now (as chosen, or as far as the performance forced
/// has come).
fn show_resolution(
    settings: Res<Settings>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    detail: Option<Res<Detail>>,
    native: Option<Res<NativeShare>>,
    mut line: Single<(&mut Text, &mut Node), With<ResolutionShown>>,
) {
    let (text, node) = &mut *line;
    let shown = display(settings.show_resolution);
    if node.display != shown {
        node.display = shown;
    }
    let Some(window) = window.iter().next().filter(|_| settings.show_resolution) else {
        return;
    };
    let wanted = resolution_line(
        window.physical_size(),
        render_percent(detail.as_deref()),
        native.as_deref().copied().unwrap_or_default(),
    );
    if text.0 != wanted {
        text.0 = wanted;
    }
}

/// With the screen split, the first player's tag at the top right.
#[derive(Component)]
struct PlayerOneTag;

/// How far from the right the first player's tag stands (px): left of the
/// performance HUD `hud` px wide (logical), or at the edge with nothing
/// shown.
fn tag_right(hud: f32) -> f32 {
    if hud > 0.5 {
        16.0 + hud + RESOLUTION_GAP
    } else {
        16.0
    }
}

/// The first player's tag kept left of the performance HUD, as wide as it
/// is.
fn keep_the_tag_clear(
    hud: Query<&ComputedNode, With<PerformanceHud>>,
    mut tags: Query<&mut Node, With<PlayerOneTag>>,
) {
    let Some(hud) = hud.iter().next() else {
        return;
    };
    let right = px(tag_right(hud.size().x * hud.inverse_scale_factor));
    for mut tag in &mut tags {
        if tag.right != right {
            tag.right = right;
        }
    }
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

        // Indicador Jugador 1 (Arriba), left of the performance HUD.
        commands.spawn((
            DespawnOnExit(GameState::Race),
            PlayerOneTag,
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
/// mando activo y a los controles elegidos. In free ride, what changes the
/// music, and what plays, is always in it. In the pause menu it gives way
/// to the menu's own legend.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_race_legend(
    mut commands: Commands,
    options: Res<DebugOptions>,
    settings: Res<Settings>,
    (rules, free_music, circuit): (
        Option<Res<Rules>>,
        Option<Res<FreeMusic>>,
        Option<Res<CurrentCircuit>>,
    ),
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
    let lang = settings.language;
    let mut wanted = options
        .show_legend
        .then(|| race_prompts(active_dev, lang, &settings.controls));
    if rules.is_some_and(|rules| rules.mode == Mode::Free)
        && let (Some(free_music), Some(circuit)) = (free_music, circuit)
    {
        let what = free_music.describe(circuit.0, lang);
        wanted
            .get_or_insert_default()
            .push(music_prompt(active_dev, lang, what));
    }
    let wanted = wanted.filter(|_| !pause.is_open());
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
    settings: Res<Settings>,
    mut meter: Local<FpsMeter>,
    mut counter: Single<(&mut Text, &mut Node), With<FpsCounter>>,
) {
    let (text, node) = &mut *counter;
    let shown = display(settings.show_fps);
    if node.display != shown {
        node.display = shown;
    }
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
    fn the_resolution_line_says_the_pixels_the_world_is_drawn_at() {
        let window = UVec2::new(1920, 1080);
        let whole = NativeShare(1.0);
        assert_eq!(resolution_line(window, 100, whole), "1920x1080");
        assert_eq!(resolution_line(window, 70, whole), "1344x756 70%");
        // On a HiDPI screen of fewer pixels of its own, never more than
        // those.
        let retina = UVec2::new(4096, 2304);
        assert_eq!(
            resolution_line(retina, 100, NativeShare(0.625)),
            "2560x1440"
        );
        // The first player's tag left of the HUD, as wide as it is, or at
        // the edge with nothing shown.
        assert_eq!(tag_right(0.0), 16.0);
        assert_eq!(tag_right(FPS_WIDTH), 16.0 + FPS_WIDTH + RESOLUTION_GAP);
        // Room for "999 FPS" in Bevy's font, whose figures are 0.6 wide.
        assert!((FPS_WIDTH - 75.6).abs() < 0.01);
    }

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
