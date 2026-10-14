//! When a race is over: its results (each bike's place and time, and the
//! points of a championship), then in a championship the standings, and on
//! to the next race, to the end of the championship or back to the menus.

use bevy::prelude::*;

use super::hud::{clock, ordinal};
use super::{Championship, Mode, Outcome, POINTS};
use crate::bike::ROSTER;
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::gamepads::ActiveController;
use crate::hud::UiFont;
use crate::legend::{Keys, menu_prompts, spawn_legend};
use crate::locale::Language;
use crate::menu::GameState;
use crate::settings::Settings;
use crate::ui::{InputDevice, MenuInput, MenuSound, UiIcons, current_active_device, text_font};

const GOLD: Color = Color::srgb(1.0, 0.82, 0.2);
const PLAYER: Color = Color::srgb(0.45, 0.82, 1.0);

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (use_results, show_results)
                .chain()
                .run_if(in_state(GameState::Race).and_then(resource_exists::<Outcome>)),
        );
    }
}

/// Which page of the results is shown: the race's, then a championship's
/// standings.
#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
enum ResultsPage {
    #[default]
    Race,
    Standings,
}

/// What comes after the results: the standings of a championship, its next
/// race, the menu after the last one, or choosing a track again.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Next {
    Standings,
    Race(usize),
    Menu,
    Tracks,
}

/// What comes after the page `standings` (or not) of the results of a race
/// of `mode`, with the championship as it is now.
pub fn next_after(mode: Mode, standings: bool, championship: &Championship) -> Next {
    if !mode.is_championship() {
        return Next::Tracks;
    }
    if !standings {
        return Next::Standings;
    }
    if championship.is_over() {
        Next::Menu
    } else {
        Next::Race(championship.round)
    }
}

#[allow(clippy::too_many_arguments)]
fn use_results(
    mut commands: Commands,
    mut input: MenuInput,
    outcome: Res<Outcome>,
    page: Option<Res<ResultsPage>>,
    championship: Res<Championship>,
    mut current: ResMut<CurrentCircuit>,
    mut next: ResMut<NextState<GameState>>,
    mut sounds: MessageWriter<MenuSound>,
) {
    let request = input.read();
    let standings = page.is_some_and(|page| *page == ResultsPage::Standings);
    if request.back {
        sounds.write(MenuSound::Move);
        commands.remove_resource::<ResultsPage>();
        next.set(GameState::Modes);
        return;
    }
    if !(request.confirm || request.start) {
        return;
    }
    sounds.write(MenuSound::Confirm);
    match next_after(outcome.mode, standings, &championship) {
        Next::Standings => commands.insert_resource(ResultsPage::Standings),
        Next::Race(round) => {
            commands.remove_resource::<ResultsPage>();
            current.0 = round;
            next.set(GameState::Race);
        }
        Next::Menu => {
            commands.remove_resource::<ResultsPage>();
            next.set(GameState::Welcome);
        }
        Next::Tracks => {
            commands.remove_resource::<ResultsPage>();
            next.set(GameState::Tracks);
        }
    }
}

#[derive(Component)]
struct ResultsScreen;

#[allow(clippy::too_many_arguments)]
fn show_results(
    mut commands: Commands,
    outcome: Res<Outcome>,
    page: Option<Res<ResultsPage>>,
    championship: Res<Championship>,
    settings: Res<Settings>,
    device: Res<InputDevice>,
    active_controller: Option<Res<ActiveController>>,
    font: Res<UiFont>,
    icons: Res<UiIcons>,
    shown: Query<Entity, With<ResultsScreen>>,
    mut last: Local<Option<String>>,
) {
    let lang = settings.language;
    let standings = page.is_some_and(|page| *page == ResultsPage::Standings);
    let active_dev = current_active_device(*device, active_controller.as_deref());
    let wanted = format!("{standings} {lang:?} {active_dev:?} {:?}", outcome.places);
    if last.as_ref() == Some(&wanted) && !shown.is_empty() {
        return;
    }
    *last = Some(wanted);
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    let track = CIRCUITS[outcome.track].name(lang);
    let title = if standings {
        let round = championship.round.min(CIRCUITS.len());
        match lang {
            Language::Spanish if championship.is_over() => "CLASIFICACIÓN FINAL".to_string(),
            Language::English if championship.is_over() => "FINAL STANDINGS".to_string(),
            Language::Spanish => format!("CLASIFICACIÓN  ·  {round}/{}", CIRCUITS.len()),
            Language::English => format!("STANDINGS  ·  {round}/{}", CIRCUITS.len()),
        }
    } else if outcome.mode == Mode::Checkpoint {
        match lang {
            Language::Spanish => "¡TIEMPO AGOTADO!".to_string(),
            Language::English => "TIME UP!".to_string(),
        }
    } else {
        match lang {
            Language::Spanish => format!("RESULTADOS  ·  {track}"),
            Language::English => format!("RESULTS  ·  {track}"),
        }
    };
    let lines = if standings {
        standing_lines(&outcome, &championship, lang)
    } else if outcome.mode == Mode::Checkpoint {
        checkpoint_lines(&outcome, &settings, track, lang)
    } else {
        race_lines(&outcome, lang)
    };
    let next = next_after(outcome.mode, standings, &championship);
    let go_on = match (next, lang) {
        (Next::Standings, Language::Spanish) => "Clasificación",
        (Next::Standings, Language::English) => "Standings",
        (Next::Race(_), Language::Spanish) => "Siguiente carrera",
        (Next::Race(_), Language::English) => "Next race",
        (Next::Menu, Language::Spanish) => "Terminar",
        (Next::Menu, Language::English) => "Finish",
        (Next::Tracks, Language::Spanish) => "Elegir pista",
        (Next::Tracks, Language::English) => "Choose track",
    };
    let prompts = menu_prompts(
        active_dev,
        &[
            (Keys::Confirm, go_on),
            (
                Keys::Back,
                match lang {
                    Language::Spanish => "Modos de juego",
                    Language::English => "Game modes",
                },
            ),
        ],
    );

    commands
        .spawn((
            ResultsScreen,
            DespawnOnExit(GameState::Race),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(px(56)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.6)),
            GlobalZIndex(90),
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: px(640),
                        max_width: percent(94),
                        padding: UiRect::axes(px(26), px(20)),
                        row_gap: px(6),
                        border_radius: BorderRadius::all(px(18)),
                        border: UiRect::all(px(2)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.94)),
                    BorderColor::all(GOLD.with_alpha(0.6)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(title),
                        text_font(&font, 30.0),
                        TextColor(GOLD),
                        TextShadow::default(),
                        Node {
                            align_self: AlignSelf::Center,
                            margin: UiRect::bottom(px(10)),
                            ..default()
                        },
                    ));
                    for line in lines {
                        panel
                            .spawn((
                                Node {
                                    justify_content: JustifyContent::SpaceBetween,
                                    column_gap: px(18),
                                    padding: UiRect::axes(px(12), px(4)),
                                    border_radius: BorderRadius::all(px(8)),
                                    ..default()
                                },
                                BackgroundColor(if line.player {
                                    PLAYER.with_alpha(0.18)
                                } else {
                                    Color::NONE
                                }),
                            ))
                            .with_children(|row| {
                                let color = if line.player { PLAYER } else { Color::WHITE };
                                row.spawn((
                                    Text::new(line.left),
                                    text_font(&font, 20.0),
                                    TextColor(color),
                                ));
                                row.spawn((
                                    Text::new(line.right),
                                    text_font(&font, 20.0),
                                    TextColor(color),
                                ));
                            });
                    }
                });
            spawn_legend(screen, &font, &icons, active_dev, &prompts);
        });
}

/// A line of the results: what goes on the left and on the right, and
/// whether it is a player's.
struct Line {
    left: String,
    right: String,
    player: bool,
}

/// A bike's name, marked if it is a player's.
fn rider_name(rider: usize, outcome: &Outcome, lang: Language) -> (String, bool) {
    let name = ROSTER[rider].name;
    match outcome.players.iter().position(|&p| p == rider) {
        Some(index) if outcome.players.len() > 1 => (format!("{name}  ·  {}P", index + 1), true),
        Some(_) => (
            format!(
                "{name}  ·  {}",
                match lang {
                    Language::Spanish => "TÚ",
                    Language::English => "YOU",
                }
            ),
            true,
        ),
        None => (name.to_string(), false),
    }
}

/// Each bike's place, time (or gap, or out) and, in a championship, the
/// points it got.
fn race_lines(outcome: &Outcome, lang: Language) -> Vec<Line> {
    let winner = outcome.places.first().and_then(|place| place.time);
    outcome
        .places
        .iter()
        .enumerate()
        .map(|(index, place)| {
            let (name, player) = rider_name(place.rider, outcome, lang);
            let result = if place.out {
                match lang {
                    Language::Spanish => "eliminado".to_string(),
                    Language::English => "out".to_string(),
                }
            } else {
                match (place.time, winner) {
                    (Some(time), Some(first)) if index > 0 => {
                        format!("+{:.1} s", time - first)
                    }
                    (Some(time), _) => clock(time),
                    (None, _) => match lang {
                        Language::Spanish => "en carrera".to_string(),
                        Language::English => "racing".to_string(),
                    },
                }
            };
            let points = if outcome.mode.is_championship() {
                format!("   +{}", POINTS.get(index).copied().unwrap_or(0))
            } else {
                String::new()
            };
            Line {
                left: format!("{}   {name}", ordinal(index + 1, lang)),
                right: format!("{result}{points}"),
                player,
            }
        })
        .collect()
}

/// The championship's standings: each bike's place and points.
fn standing_lines(outcome: &Outcome, championship: &Championship, lang: Language) -> Vec<Line> {
    championship
        .standings()
        .into_iter()
        .enumerate()
        .map(|(index, rider)| {
            let (name, player) = rider_name(rider, outcome, lang);
            Line {
                left: format!("{}   {name}", ordinal(index + 1, lang)),
                right: format!("{} pts", championship.points[rider]),
                player,
            }
        })
        .collect()
}

/// Against the clock: the laps done and the best on this track.
fn checkpoint_lines(
    outcome: &Outcome,
    settings: &Settings,
    track: &str,
    lang: Language,
) -> Vec<Line> {
    let name = CIRCUITS[outcome.track].name;
    let best = settings.records.checkpoint_best(name).unwrap_or(0);
    let lap = settings.records.best_lap(name);
    let mut lines = vec![
        Line {
            left: track.to_string(),
            right: String::new(),
            player: false,
        },
        Line {
            left: match lang {
                Language::Spanish => "Vueltas".to_string(),
                Language::English => "Laps".to_string(),
            },
            right: if outcome.record {
                match lang {
                    Language::Spanish => format!("{}   ¡RÉCORD!", outcome.laps),
                    Language::English => format!("{}   RECORD!", outcome.laps),
                }
            } else {
                outcome.laps.to_string()
            },
            player: true,
        },
        Line {
            left: match lang {
                Language::Spanish => "Récord de vueltas".to_string(),
                Language::English => "Most laps".to_string(),
            },
            right: best.to_string(),
            player: false,
        },
    ];
    if let Some(lap) = lap {
        lines.push(Line {
            left: match lang {
                Language::Spanish => "Mejor vuelta".to_string(),
                Language::English => "Best lap".to_string(),
            },
            right: clock(lap),
            player: false,
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn after_the_results_comes_what_the_mode_says() {
        let mut championship = Championship::default();
        assert_eq!(next_after(Mode::Race, false, &championship), Next::Tracks);
        assert_eq!(
            next_after(Mode::Checkpoint, false, &championship),
            Next::Tracks
        );
        // A championship: the standings, then its next race...
        assert_eq!(
            next_after(Mode::Story, false, &championship),
            Next::Standings
        );
        championship.score(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(next_after(Mode::Story, true, &championship), Next::Race(1));
        // ...and after the last one, the menu.
        championship.round = CIRCUITS.len();
        assert_eq!(
            next_after(Mode::Elimination, true, &championship),
            Next::Menu
        );
    }
}
