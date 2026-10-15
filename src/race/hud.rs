//! What the race shows on the screen for each player: their place, their
//! lap, the clock and their best lap; against the clock, the time left and
//! what each checkpoint gives; in elimination, who is out (and how many are
//! still in, which pops as it changes); and their place when they cross the
//! line at the end. Once the players' race is over,
//! which bike the camera watches come in, and its place when it does.

use bevy::prelude::*;

use super::{Mode, Outcome, Progress, Rules, Standing, Watching};
use crate::bike::{LocalPlayer, Player, ROSTER, Race, Retired, Rider};
use crate::geometry::smoothstep;
use crate::hud::UiFont;
use crate::locale::Language;
use crate::menu::GameState;
use crate::settings::Settings;
use crate::welcome::GameMode;

/// Seconds a checkpoint's bonus and who is out are shown for.
const NOTICE: f32 = 2.5;
const GOLD: Color = Color::srgb(1.0, 0.82, 0.2);
const RED: Color = Color::srgb(1.0, 0.3, 0.25);
/// How big the place is (px), and the total after it ("/ 8").
const PLACE_SIZE: f32 = 40.0;
/// Seconds the total pops for when it changes, and how much bigger it gets.
const POP: f32 = 0.8;
const POP_GROWTH: f32 = 0.6;

pub struct RaceHudPlugin;

impl Plugin for RaceHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Race), spawn_race_info)
            .add_systems(
                Update,
                (
                    show_race_info,
                    show_place_total,
                    show_watched,
                    show_arrivals,
                )
                    .run_if(in_state(GameState::Race)),
            );
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

/// A text about the bike the camera watches come in: who it is and how it
/// is getting on, or, as it crosses the line, its place.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum WatchInfo {
    Caption,
    Notice,
}

/// Top right, under the frame rate: who has crossed the line at the end, in
/// order, and when.
#[derive(Component)]
struct Arrivals;

/// After a player's place, how many there are ("/ 8"), and since when (s,
/// real time) the total it shows changed, to pop.
#[derive(Component, Debug, Default)]
struct PlaceTotal {
    player: usize,
    shown: Option<usize>,
    changed: Option<f32>,
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
    // Top right, under the frame rate, with no panel behind: those in.
    commands.spawn((
        DespawnOnExit(GameState::Race),
        Arrivals,
        Text::new(""),
        text(17.0),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Right),
        TextShadow {
            offset: Vec2::splat(1.5),
            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(40),
            right: px(16),
            ..default()
        },
    ));
    // Over the whole screen, left of the results: the bike watched.
    commands
        .spawn((
            DespawnOnExit(GameState::Race),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(60),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|left| {
            left.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(18),
                top: px(14),
                ..default()
            })
            .with_child((
                WatchInfo::Caption,
                Text::new(""),
                text(26.0),
                TextColor(Color::WHITE),
                TextShadow::default(),
            ));
            left.spawn(Node {
                position_type: PositionType::Absolute,
                top: percent(30),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_child((
                WatchInfo::Notice,
                Text::new(""),
                text(44.0),
                TextColor(GOLD),
                TextLayout::justify(Justify::Center),
                TextShadow {
                    offset: Vec2::splat(3.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                },
            ));
        });
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
                        (InfoKind::Place, PLACE_SIZE, Color::WHITE),
                        (InfoKind::Lap, 22.0, Color::WHITE),
                        (InfoKind::Clock, 20.0, Color::srgba(1.0, 1.0, 1.0, 0.85)),
                        (InfoKind::Best, 15.0, GOLD),
                    ] {
                        let info = (
                            Info { player, kind },
                            Text::new(""),
                            text(size),
                            TextColor(color),
                            TextShadow::default(),
                        );
                        if kind != InfoKind::Place {
                            column.spawn(info);
                            continue;
                        }
                        // The total after the place, apart, to pop without
                        // moving the rest.
                        column.spawn(Node::default()).with_children(|row| {
                            row.spawn(info);
                            row.spawn((
                                PlaceTotal {
                                    player,
                                    ..default()
                                },
                                Text::new(""),
                                text(PLACE_SIZE),
                                TextColor(Color::WHITE),
                                TextShadow::default(),
                                UiTransform::IDENTITY,
                            ));
                        });
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

/// A bike's place in the race (`rider`, of the roster) and how many bikes
/// it is among: all of them; but in elimination, while it is still in,
/// only those still in, so that the count comes down as they go out
/// ("2nd / 6" with two of eight out). One out keeps its place among them
/// all.
fn place_among(standing: &Standing, mode: Mode, rider: usize) -> (usize, usize) {
    let racers = standing.order.len();
    let place = standing
        .order
        .iter()
        .position(|&r| r == rider)
        .map_or(racers, |p| p + 1);
    let among = if mode == Mode::Elimination && !standing.out.contains(&rider) {
        racers.saturating_sub(standing.out.len())
    } else {
        racers
    };
    (place, among)
}

#[allow(clippy::type_complexity)]
fn show_race_info(
    rules: Res<Rules>,
    race: Res<Race>,
    standing: Res<Standing>,
    settings: Res<Settings>,
    outcome: Option<Res<Outcome>>,
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
    for (info, mut text, mut color) in &mut texts {
        let Some((rider, progress, _, _, retired)) =
            bikes.iter().find(|(_, _, player, local, _)| {
                let index = local.map_or(usize::from(!*player), |local| local.0);
                index == info.player
            })
        else {
            continue;
        };
        let (place, _) = place_among(&standing, rules.mode, rider.0);
        let (wanted, tint) = match info.kind {
            // Over for the players: the results say it all.
            _ if outcome.is_some() => (String::new(), color.0),
            InfoKind::Place if rules.mode != Mode::Checkpoint => (ordinal(place, lang), color.0),
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
            InfoKind::Notice => {
                let out = match (retired, standing.disqualified.contains(&rider.0)) {
                    (_, true) => Out::Disqualified,
                    (true, false) => Out::Retired,
                    (false, false) => Out::In,
                };
                // Back on the grid after a false start: how many so far.
                let made = standing
                    .false_starts
                    .iter()
                    .filter(|&&r| r == rider.0)
                    .count() as u32;
                let false_starts =
                    (!race.started && made > 0).then_some((made, settings.false_starts));
                notice(&rules, &standing, progress, out, false_starts, place, lang)
            }
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

/// How much the total after a place pops (0 to 1) `elapsed` s after it
/// changed: at once all the way, and back over [`POP`].
pub fn pop(elapsed: f32) -> f32 {
    if !(0.0..POP).contains(&elapsed) {
        return 0.0;
    }
    let back = 1.0 - elapsed / POP;
    smoothstep(0.0, 0.06, elapsed) * back * back
}

/// After each player's place, how many there are (" / 8"; in elimination,
/// how many are still in), popping bigger and red as it changes: scaled from
/// its left end, so nothing round it moves.
#[allow(clippy::type_complexity)]
fn show_place_total(
    time: Res<Time<Real>>,
    rules: Res<Rules>,
    standing: Res<Standing>,
    outcome: Option<Res<Outcome>>,
    bikes: Query<(&Rider, Has<Player>, Option<&LocalPlayer>)>,
    mut totals: Query<(
        &mut PlaceTotal,
        &mut Text,
        &mut UiTransform,
        &mut TextColor,
        &ComputedNode,
    )>,
) {
    let now = time.elapsed_secs();
    for (mut total, mut text, mut transform, mut color, node) in &mut totals {
        let rider = bikes.iter().find_map(|(rider, player, local)| {
            let index = local.map_or(usize::from(!player), |local| local.0);
            (index == total.player).then_some(rider.0)
        });
        let among = rider
            .filter(|_| outcome.is_none() && rules.mode != Mode::Checkpoint)
            .map(|rider| place_among(&standing, rules.mode, rider).1);
        // A change of the total pops; its first showing does not.
        if among != total.shown {
            if total.shown.is_some_and(|shown| shown > 0) && among.is_some() {
                total.changed = Some(now);
            }
            total.shown = among;
        }
        let wanted = among.map_or(String::new(), |among| format!(" / {among}"));
        if text.0 != wanted {
            text.0 = wanted;
        }
        let popped = total.changed.map_or(0.0, |at| pop(now - at));
        let scale = 1.0 + POP_GROWTH * popped;
        let width = node.size().x * node.inverse_scale_factor;
        let popping = UiTransform {
            translation: Val2::px(width * (scale - 1.0) / 2.0, 0.0),
            scale: Vec2::splat(scale),
            ..UiTransform::IDENTITY
        };
        if *transform != popping {
            *transform = popping;
        }
        let tint = Color::WHITE.mix(&RED, popped);
        if color.0 != tint {
            color.0 = tint;
        }
    }
}

/// Who the camera watches come in: their place and lap, and as they cross
/// the line, their place and how far behind the winner.
#[allow(clippy::type_complexity)]
fn show_watched(
    rules: Res<Rules>,
    standing: Res<Standing>,
    settings: Res<Settings>,
    watching: Option<Res<Watching>>,
    bikes: Query<&Progress>,
    mut texts: Query<(&WatchInfo, &mut Text)>,
) {
    let lang = settings.language;
    let watched = watching
        .as_deref()
        .and_then(|watching| Some((watching, bikes.get(watching.bike).ok()?)));
    let place = watched.and_then(|(watching, _)| {
        standing
            .order
            .iter()
            .position(|&rider| rider == watching.rider)
            .map(|index| index + 1)
    });
    // When the first one crossed the line.
    let winner = bikes
        .iter()
        .filter_map(|progress| progress.finished)
        .min_by(f32::total_cmp);
    for (info, mut text) in &mut texts {
        let wanted = match (info, watched, place) {
            (WatchInfo::Caption, Some((watching, progress)), Some(place)) => {
                let name = ROSTER[watching.rider].name;
                let lap = match lang {
                    Language::Spanish => "VUELTA",
                    Language::English => "LAP",
                };
                match rules.mode.laps(rules.laps) {
                    Some(laps) if progress.finished.is_none() => format!(
                        "{name}  ·  {}  ·  {lap} {}/{laps}",
                        ordinal(place, lang),
                        (progress.laps + 1).min(laps)
                    ),
                    _ => format!("{name}  ·  {}", ordinal(place, lang)),
                }
            }
            (WatchInfo::Notice, Some((watching, progress)), Some(place)) => {
                match progress.finished {
                    Some(at) if standing.clock - at < NOTICE => {
                        let gap = winner
                            .filter(|first| at - first > 0.05)
                            .map_or(String::new(), |first| format!("  +{:.1} s", at - first));
                        let finish = match lang {
                            Language::Spanish => "¡META!",
                            Language::English => "FINISH!",
                        };
                        format!(
                            "{finish}  {}  {}{gap}",
                            ROSTER[watching.rider].name,
                            ordinal(place, lang)
                        )
                    }
                    _ => String::new(),
                }
            }
            _ => String::new(),
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
    }
}

/// In a race or the arcade, those who have crossed the line at the end, as
/// they do: their place, name and time. Once the results are up, they say
/// it instead.
fn show_arrivals(
    rules: Res<Rules>,
    standing: Res<Standing>,
    settings: Res<Settings>,
    outcome: Option<Res<Outcome>>,
    bikes: Query<(&Rider, &Progress)>,
    mut text: Query<&mut Text, With<Arrivals>>,
) {
    let wanted = if matches!(rules.mode, Mode::Race | Mode::Arcade) && outcome.is_none() {
        arrival_lines(&standing.order, settings.language, |rider| {
            bikes
                .iter()
                .find(|(r, _)| r.0 == rider)
                .and_then(|(_, progress)| progress.finished)
        })
    } else {
        String::new()
    };
    for mut text in &mut text {
        if text.0 != wanted {
            text.0.clone_from(&wanted);
        }
    }
}

/// A line for each bike of `order` (bikes of the roster) that has crossed
/// the line at the end (`finished`: when), in order: "1st  Name  1:23.4".
fn arrival_lines(
    order: &[usize],
    lang: Language,
    finished: impl Fn(usize) -> Option<f32>,
) -> String {
    let mut arrived: Vec<(usize, f32)> = order
        .iter()
        .filter_map(|&rider| finished(rider).map(|time| (rider, time)))
        .collect();
    arrived.sort_by(|a, b| a.1.total_cmp(&b.1));
    arrived
        .iter()
        .enumerate()
        .map(|(index, (rider, time))| {
            format!(
                "{}   {}   {}",
                ordinal(index + 1, lang),
                ROSTER[*rider].name,
                clock(*time)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether a player is still in the race.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Out {
    In,
    Retired,
    /// For jumping the start.
    Disqualified,
}

/// The big notice in the middle for a player at `place`: their place when
/// they cross the line at the end, that their time is up, they are out or
/// disqualified, back on the grid after a false start (`false_starts`: how
/// many they have made and are let off), who has just gone out, or a lap
/// record.
fn notice(
    rules: &Rules,
    standing: &Standing,
    progress: &Progress,
    out: Out,
    false_starts: Option<(u32, u32)>,
    place: usize,
    lang: Language,
) -> (String, Color) {
    if let Some((made, allowed)) = false_starts.filter(|_| out == Out::In) {
        return (
            match lang {
                Language::Spanish => format!("¡SALIDA NULA!  {made} de {allowed}"),
                Language::English => format!("FALSE START!  {made} of {allowed}"),
            },
            GOLD,
        );
    }
    if out == Out::Disqualified {
        return (
            match lang {
                Language::Spanish => "¡SALIDA ANTICIPADA!  DESCALIFICADO".into(),
                Language::English => "JUMP START!  DISQUALIFIED".into(),
            },
            RED,
        );
    }
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
    if out == Out::Retired {
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
    // Just after the lap that beat it, to the player who rode it.
    if let Some((best, at)) = standing.record
        && progress.best_lap == Some(best)
        && standing.clock - at < NOTICE
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
    fn those_in_are_listed_by_when_they_crossed_the_line() {
        let finished = |rider: usize| match rider {
            3 => Some(83.4),
            1 => Some(81.2),
            _ => None,
        };
        let lines = arrival_lines(&[1, 3, 0, 2], Language::English, finished);
        assert_eq!(
            lines,
            format!(
                "1st   {}   1:21.2\n2nd   {}   1:23.4",
                ROSTER[1].name, ROSTER[3].name
            )
        );
        assert_eq!(arrival_lines(&[0, 2], Language::Spanish, finished), "");
    }

    #[test]
    fn a_lap_record_is_told_once_just_after_the_lap_that_beat_it() {
        let rules = Rules {
            mode: Mode::Race,
            laps: 5,
        };
        let mut standing = Standing {
            clock: 72.0,
            record: Some((70.0, 71.0)),
            ..default()
        };
        let progress = Progress {
            laps: 1,
            best_lap: Some(70.0),
            ..default()
        };
        let told = |standing: &Standing, progress: &Progress| {
            notice(
                &rules,
                standing,
                progress,
                Out::In,
                None,
                8,
                Language::English,
            )
            .0
            .starts_with("LAP RECORD")
        };
        assert!(told(&standing, &progress));
        // Not to the other player, whose best is another.
        let other = Progress {
            best_lap: Some(74.0),
            ..progress.clone()
        };
        assert!(!told(&standing, &other));
        // Laps later, slower ones: no more.
        standing.clock = 150.0;
        let later = Progress {
            laps: 2,
            ..progress.clone()
        };
        assert!(!told(&standing, &later));
    }

    #[test]
    fn back_on_the_grid_after_a_false_start_it_says_how_many() {
        let rules = Rules {
            mode: Mode::Race,
            laps: 5,
        };
        let standing = Standing::default();
        let progress = Progress::default();
        let said = |out, false_starts| {
            notice(
                &rules,
                &standing,
                &progress,
                out,
                false_starts,
                8,
                Language::Spanish,
            )
            .0
        };
        assert_eq!(said(Out::In, Some((1, 2))), "¡SALIDA NULA!  1 de 2");
        assert_eq!(said(Out::In, None), "");
        // Disqualified says so instead.
        assert!(said(Out::Disqualified, Some((2, 2))).contains("DESCALIFICADO"));
    }

    #[test]
    fn in_elimination_a_place_is_among_those_still_in() {
        // Eight bikes, two of them out: 7 first, then 5.
        let standing = Standing {
            order: vec![3, 0, 1, 2, 4, 6, 5, 7],
            out: vec![7, 5],
            ..default()
        };
        assert_eq!(place_among(&standing, Mode::Elimination, 0), (2, 6));
        assert_eq!(place_among(&standing, Mode::Elimination, 6), (6, 6));
        // Those out keep their place among them all.
        assert_eq!(place_among(&standing, Mode::Elimination, 5), (7, 8));
        assert_eq!(place_among(&standing, Mode::Elimination, 7), (8, 8));
        // Nobody out yet: among them all.
        let start = Standing {
            out: Vec::new(),
            ..standing.clone()
        };
        assert_eq!(place_among(&start, Mode::Elimination, 0), (2, 8));
        // In a race, a bike out (disqualified) still counts.
        assert_eq!(place_among(&standing, Mode::Race, 0), (2, 8));
    }

    #[test]
    fn the_total_pops_at_once_and_settles_back() {
        assert_eq!(pop(-0.1), 0.0);
        assert!(pop(0.08) > 0.8, "at once");
        // Smaller and smaller, and gone after a moment.
        let mut last = pop(0.08);
        for k in 2..16 {
            let now = pop(0.05 * k as f32);
            assert!(now < last, "{k}");
            last = now;
        }
        assert_eq!(pop(POP), 0.0);
        assert_eq!(pop(5.0), 0.0);
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
