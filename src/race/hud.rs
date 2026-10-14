//! What the race shows on the screen for each player: their place, their
//! lap, the clock and their best lap; against the clock, the time left and
//! what each checkpoint gives; in elimination, who is out; and their place
//! when they cross the line at the end.

use bevy::prelude::*;

use super::{Mode, Progress, Rules, Standing};
use crate::bike::{LocalPlayer, Player, ROSTER, Retired, Rider};
use crate::hud::UiFont;
use crate::locale::Language;
use crate::menu::GameState;
use crate::settings::Settings;
use crate::welcome::GameMode;

/// Seconds a checkpoint's bonus and who is out are shown for.
const NOTICE: f32 = 2.5;
const GOLD: Color = Color::srgb(1.0, 0.82, 0.2);
const RED: Color = Color::srgb(1.0, 0.3, 0.25);

pub struct RaceHudPlugin;

impl Plugin for RaceHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Race), spawn_race_info)
            .add_systems(Update, show_race_info.run_if(in_state(GameState::Race)));
    }
}

/// A text of the race for a player (0 or 1).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
struct Info {
    player: usize,
    kind: InfoKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InfoKind {
    Place,
    Lap,
    Clock,
    Best,
    /// Against the clock, the time left; or a big notice in the middle.
    TimeLeft,
    Notice,
}

fn spawn_race_info(mut commands: Commands, font: Res<UiFont>, game_mode: Res<GameMode>) {
    let players = if *game_mode == GameMode::SplitScreen {
        2
    } else {
        1
    };
    let text = |size: f32| TextFont {
        font: font.0.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    };
    for player in 0..players {
        // Each player's half of the screen (or all of it).
        let (top, height) = if players == 1 {
            (0.0, 100.0)
        } else {
            (50.0 * player as f32, 50.0)
        };
        commands
            .spawn((
                DespawnOnExit(GameState::Race),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: percent(top),
                    width: percent(100),
                    height: percent(height),
                    ..default()
                },
            ))
            .with_children(|half| {
                // Top left: place, lap, clock and best lap.
                half.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px(18),
                    top: px(12),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                })
                .with_children(|column| {
                    for (kind, size, color) in [
                        (InfoKind::Place, 40.0, Color::WHITE),
                        (InfoKind::Lap, 22.0, Color::WHITE),
                        (InfoKind::Clock, 20.0, Color::srgba(1.0, 1.0, 1.0, 0.85)),
                        (InfoKind::Best, 15.0, GOLD),
                    ] {
                        column.spawn((
                            Info { player, kind },
                            Text::new(""),
                            text(size),
                            TextColor(color),
                            TextShadow::default(),
                        ));
                    }
                });
                // Top middle: the time left against the clock.
                half.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: px(10),
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_child((
                    Info {
                        player,
                        kind: InfoKind::TimeLeft,
                    },
                    Text::new(""),
                    text(46.0),
                    TextLayout::justify(Justify::Center),
                    TextShadow::default(),
                ));
                // In the middle: notices.
                half.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: percent(30),
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_child((
                    Info {
                        player,
                        kind: InfoKind::Notice,
                    },
                    Text::new(""),
                    text(if players == 1 { 52.0 } else { 36.0 }),
                    TextLayout::justify(Justify::Center),
                    TextShadow {
                        offset: Vec2::splat(3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                    },
                ));
            });
    }
}

/// A time as minutes, seconds and tenths: 1:23.4.
pub fn clock(seconds: f32) -> String {
    let tenths = (seconds.max(0.0) * 10.0).floor() as u32;
    format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

/// A place: 1º, 2º... in Spanish; 1st, 2nd... in English.
pub fn ordinal(place: usize, lang: Language) -> String {
    match lang {
        Language::Spanish => format!("{place}º"),
        Language::English => {
            let suffix = match (place % 10, place % 100) {
                (1, 11) | (2, 12) | (3, 13) => "th",
                (1, _) => "st",
                (2, _) => "nd",
                (3, _) => "rd",
                _ => "th",
            };
            format!("{place}{suffix}")
        }
    }
}

#[allow(clippy::type_complexity)]
fn show_race_info(
    rules: Res<Rules>,
    standing: Res<Standing>,
    settings: Res<Settings>,
    bikes: Query<(
        &Rider,
        &Progress,
        Has<Player>,
        Option<&LocalPlayer>,
        Has<Retired>,
    )>,
    mut texts: Query<(&Info, &mut Text, &mut TextColor)>,
) {
    let lang = settings.language;
    let target = rules.mode.laps(rules.laps);
    let racers = standing.order.len();
    for (info, mut text, mut color) in &mut texts {
        let Some((rider, progress, _, _, retired)) =
            bikes.iter().find(|(_, _, player, local, _)| {
                let index = local.map_or(usize::from(!*player), |local| local.0);
                index == info.player
            })
        else {
            continue;
        };
        let place = standing
            .order
            .iter()
            .position(|r| *r == rider.0)
            .map_or(racers, |p| p + 1);
        let (wanted, tint) = match info.kind {
            InfoKind::Place if rules.mode != Mode::Checkpoint => {
                (format!("{} / {racers}", ordinal(place, lang)), color.0)
            }
            InfoKind::Lap => {
                let lap = match lang {
                    Language::Spanish => "VUELTA",
                    Language::English => "LAP",
                };
                let now = progress.laps + 1;
                (
                    match target {
                        Some(laps) => format!("{lap} {}/{laps}", now.min(laps)),
                        None => format!("{lap} {now}"),
                    },
                    color.0,
                )
            }
            InfoKind::Clock => (clock(standing.clock), color.0),
            InfoKind::Best => (
                progress.best_lap.map_or(String::new(), |best| {
                    let label = match lang {
                        Language::Spanish => "MEJOR",
                        Language::English => "BEST",
                    };
                    format!("{label} {}", clock(best))
                }),
                color.0,
            ),
            InfoKind::TimeLeft if rules.mode == Mode::Checkpoint => {
                let left = standing.time_left;
                let bonus = standing
                    .bonus
                    .filter(|(_, at)| standing.clock - at < NOTICE)
                    .map_or(String::new(), |(bonus, _)| format!("   +{bonus:.1}"));
                (
                    format!("{:.1}{bonus}", left),
                    if left < 10.0 { RED } else { GOLD },
                )
            }
            InfoKind::Notice => notice(&rules, &standing, progress, retired, place, lang),
            _ => (String::new(), color.0),
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
        if color.0 != tint {
            color.0 = tint;
        }
    }
}

/// The big notice in the middle for a player at `place`: their place when
/// they cross the line at the end, that their time is up or they are out,
/// who has just gone out, or a lap record.
fn notice(
    rules: &Rules,
    standing: &Standing,
    progress: &Progress,
    retired: bool,
    place: usize,
    lang: Language,
) -> (String, Color) {
    if progress.finished.is_some() {
        return (
            match lang {
                Language::Spanish => format!("¡META!  {}", ordinal(place, lang)),
                Language::English => format!("FINISH!  {}", ordinal(place, lang)),
            },
            GOLD,
        );
    }
    if rules.mode == Mode::Checkpoint && standing.time_left <= 0.0 {
        return (
            match lang {
                Language::Spanish => "¡TIEMPO!".into(),
                Language::English => "TIME UP!".into(),
            },
            RED,
        );
    }
    if retired {
        return (
            match lang {
                Language::Spanish => "¡ELIMINADO!".into(),
                Language::English => "ELIMINATED!".into(),
            },
            RED,
        );
    }
    if let Some((rider, at)) = standing.last_out
        && standing.clock - at < NOTICE
    {
        let name = ROSTER[rider].name;
        return (
            match lang {
                Language::Spanish => format!("Eliminado: {name}"),
                Language::English => format!("Out: {name}"),
            },
            RED,
        );
    }
    if let Some(best) = standing.record.filter(|_| progress.laps > 0)
        && progress.best_lap == Some(best)
        && standing.clock - progress_lap_start(progress) < NOTICE
    {
        return (
            match lang {
                Language::Spanish => format!("¡RÉCORD DE VUELTA!  {}", clock(best)),
                Language::English => format!("LAP RECORD!  {}", clock(best)),
            },
            GOLD,
        );
    }
    (String::new(), Color::WHITE)
}

/// When the lap under way began (s on the clock).
fn progress_lap_start(progress: &Progress) -> f32 {
    progress.lap_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_as_minutes_seconds_and_tenths() {
        assert_eq!(clock(0.0), "0:00.0");
        assert_eq!(clock(83.47), "1:23.4");
        assert_eq!(clock(600.0), "10:00.0");
    }

    #[test]
    fn places_read_in_each_language() {
        assert_eq!(ordinal(1, Language::Spanish), "1º");
        assert_eq!(ordinal(1, Language::English), "1st");
        assert_eq!(ordinal(2, Language::English), "2nd");
        assert_eq!(ordinal(3, Language::English), "3rd");
        assert_eq!(ordinal(4, Language::English), "4th");
        assert_eq!(ordinal(11, Language::English), "11th");
    }
}
