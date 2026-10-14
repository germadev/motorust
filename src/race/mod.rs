//! The race: what is being played ([`Mode`]), each bike's progress round
//! its laps ([`Progress`]), the order they are in, the clock, the
//! checkpoints, who is eliminated and when it is over; and the
//! championships, in which the tracks are raced one after another
//! ([`Championship`]).
//!
//! What it shows: the laps, places and times on the screen ([`hud`]) and
//! the results when the race is over ([`results`]).

pub mod hud;
pub mod results;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::bike::{BikeState, LocalPlayer, Player, ROSTER, Race, RaceControls, Retired, Rider};
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::pause;
use crate::settings::{Action, Settings};
use crate::track::Track;

/// How many laps a race of a championship has.
pub const CHAMPIONSHIP_LAPS: u32 = 5;
/// The laps a race can be set to, and how many it has at first.
pub const LAPS: std::ops::RangeInclusive<u32> = 1..=9;
pub const DEFAULT_LAPS: u32 = 3;
/// The points for each place in a race of a championship.
pub const POINTS: [u32; 8] = [10, 8, 6, 5, 4, 3, 2, 1];
/// Against the clock: the seconds at the start and each checkpoint gives,
/// for every metre of track to the next one at this pace (m/s), and a few
/// more to spare.
const CHECKPOINT_PACE: f32 = 36.0;
const CHECKPOINT_SPARE: f32 = 4.0;
/// Seconds between the last player crossing the line (or being out) and
/// the results.
const RESULTS_AFTER: f32 = 2.5;

/// What is being played.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Mode {
    /// The championship: the tracks one after another, five laps each,
    /// with points for each place.
    Story,
    /// Riding round a track with no end.
    #[default]
    Free,
    /// A race of the laps chosen, on the track chosen.
    Race,
    /// Like the championship, but at the end of each lap the last one is
    /// out.
    Elimination,
    /// Against the clock: from a few seconds, each checkpoint passed gives
    /// more; at nought it is over.
    Checkpoint,
}

impl Mode {
    /// Whether the tracks are raced one after another.
    pub fn is_championship(self) -> bool {
        matches!(self, Mode::Story | Mode::Elimination)
    }

    /// How many laps a race has, if it ends after some.
    pub fn laps(self, chosen: u32) -> Option<u32> {
        match self {
            Mode::Free | Mode::Checkpoint => None,
            Mode::Race => Some(chosen),
            Mode::Story | Mode::Elimination => Some(CHAMPIONSHIP_LAPS),
        }
    }
}

/// What is being played, and the laps chosen for a race.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub mode: Mode,
    pub laps: u32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            mode: Mode::Free,
            laps: DEFAULT_LAPS,
        }
    }
}

/// A championship under way: the race it is on (the index of its track in
/// [`CIRCUITS`]) and the points of each bike of the [`ROSTER`] so far.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Championship {
    pub round: usize,
    pub points: [u32; 8],
}

impl Championship {
    /// Whether it is over: every track raced.
    pub fn is_over(&self) -> bool {
        self.round >= CIRCUITS.len()
    }

    /// Scores a race finished in `order` (bikes of the roster, first to
    /// last) and goes on to the next.
    pub fn score(&mut self, order: &[usize]) {
        for (place, &rider) in order.iter().enumerate() {
            self.points[rider] += POINTS.get(place).copied().unwrap_or(0);
        }
        self.round += 1;
    }

    /// The bikes of the roster from most points to fewest (ties: the
    /// order of the roster).
    pub fn standings(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..ROSTER.len()).collect();
        order.sort_by_key(|&rider| std::cmp::Reverse(self.points[rider]));
        order
    }
}

/// What is kept between sessions of the races: the championship under way
/// (to go on with it) and the best of each track.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Records {
    pub story: Option<Championship>,
    /// Best lap on each track (s), by its name.
    pub best_laps: Vec<(String, f32)>,
    /// Most laps against the clock on each track, by its name.
    pub checkpoint_laps: Vec<(String, u32)>,
}

impl Records {
    pub fn best_lap(&self, track: &str) -> Option<f32> {
        self.best_laps
            .iter()
            .find(|(name, _)| name == track)
            .map(|(_, time)| *time)
    }

    /// Keeps `time` as the best lap of `track` if it is. Whether it was.
    pub fn lap(&mut self, track: &str, time: f32) -> bool {
        match self.best_laps.iter_mut().find(|(name, _)| name == track) {
            Some((_, best)) if *best <= time => false,
            Some((_, best)) => {
                *best = time;
                true
            }
            None => {
                self.best_laps.push((track.to_string(), time));
                true
            }
        }
    }

    pub fn checkpoint_best(&self, track: &str) -> Option<u32> {
        self.checkpoint_laps
            .iter()
            .find(|(name, _)| name == track)
            .map(|(_, laps)| *laps)
    }

    /// Keeps `laps` as the most against the clock on `track` if it is.
    /// Whether it was.
    pub fn checkpoint(&mut self, track: &str, laps: u32) -> bool {
        match self
            .checkpoint_laps
            .iter_mut()
            .find(|(name, _)| name == track)
        {
            Some((_, best)) if *best >= laps => false,
            Some((_, best)) => {
                *best = laps;
                true
            }
            None => {
                self.checkpoint_laps.push((track.to_string(), laps));
                true
            }
        }
    }
}

/// The gates round a lap, as fractions of it: a lap only counts once a bike
/// has gone through the first two (so riding back and forth over the line
/// does not), and against the clock each of the three is a checkpoint.
const GATES: [f32; 2] = [1.0 / 3.0, 2.0 / 3.0];

/// How a bike is getting on in the race.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct Progress {
    /// Laps done.
    pub laps: u32,
    /// How far into this lap it is (m from the finish line).
    pub along: f32,
    /// The gates of this lap gone through (0, 1 or 2).
    gates: usize,
    /// When it crossed the line at the end of its race (s on the clock).
    pub finished: Option<f32>,
    /// When the lap under way began (s on the clock), and the best lap.
    lap_start: f32,
    pub best_lap: Option<f32>,
    /// Checkpoints gone through (against the clock).
    pub checkpoints: u32,
}

/// What a bike went through in a step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Passed {
    /// A gate on the way round (a checkpoint against the clock).
    pub gate: bool,
    /// The finish line, at the end of a lap.
    pub lap: bool,
}

impl Progress {
    /// The bike is now `along` m into a lap `lap` m long: what it went
    /// through.
    pub fn pass(&mut self, along: f32, lap: f32) -> Passed {
        let before = self.along;
        self.along = along;
        let mut passed = Passed::default();
        // A gate counts within a stretch after it, in case a step jumps it.
        if let Some(&gate) = GATES.get(self.gates) {
            let at = gate * lap;
            if (at..at + lap / 6.0).contains(&along) {
                self.gates += 1;
                passed.gate = true;
            }
        }
        if self.gates == GATES.len() && before > lap * 5.0 / 6.0 && along < lap / 6.0 {
            self.gates = 0;
            self.laps += 1;
            passed.lap = true;
            passed.gate = true;
        }
        passed
    }

    /// How far it has gone in the race (m): laps done and how far into
    /// this one. On the grid, before the line, it is short of nought.
    pub fn distance(&self, lap: f32) -> f32 {
        let along = if self.gates == 0 && self.along > lap / 2.0 {
            self.along - lap
        } else {
            self.along
        };
        self.laps as f32 * lap + along
    }
}

/// The order of the bikes of a race, first to last, given each one's
/// progress (`racing`: an id and its progress) and those out of it, in the
/// order they went out: those that finished by when, then those racing by
/// how far they have gone, then those out, the last out first.
pub fn order<T: Copy>(racing: &[(T, &Progress)], out: &[T], lap: f32) -> Vec<T> {
    let mut finished: Vec<(T, f32)> = racing
        .iter()
        .filter_map(|(id, progress)| progress.finished.map(|time| (*id, time)))
        .collect();
    finished.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut running: Vec<(T, f32)> = racing
        .iter()
        .filter(|(_, progress)| progress.finished.is_none())
        .map(|(id, progress)| (*id, progress.distance(lap)))
        .collect();
    running.sort_by(|a, b| b.1.total_cmp(&a.1));
    finished
        .into_iter()
        .chain(running)
        .map(|(id, _)| id)
        .chain(out.iter().rev().copied())
        .collect()
}

/// The seconds the clock gives against the clock for the stretch from the
/// gate a bike just went through to the next (m).
pub fn checkpoint_bonus(stretch: f32) -> f32 {
    stretch / CHECKPOINT_PACE + CHECKPOINT_SPARE
}

/// The state of the race under way.
#[derive(Resource, Clone, Debug, Default)]
pub struct Standing {
    /// Seconds on the clock since the start.
    pub clock: f32,
    /// Against the clock: the seconds left, and the last bonus and when it
    /// came (s on the clock).
    pub time_left: f32,
    pub bonus: Option<(f32, f32)>,
    /// The bikes out, in the order they went out (bikes of the roster).
    pub out: Vec<usize>,
    /// The order of the race now (bikes of the roster), first to last.
    pub order: Vec<usize>,
    /// Laps the leader has done that have already put someone out.
    eliminations: u32,
    /// The last one put out and when (s on the clock), to show it.
    pub last_out: Option<(usize, f32)>,
    /// When all the players were done (s on the clock).
    done_at: Option<f32>,
    /// The best lap of this race on this track, if it was a record.
    pub record: Option<f32>,
}

/// The race is over: what is shown on the results.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Outcome {
    pub mode: Mode,
    pub track: usize,
    /// The bikes of the roster in the order they finished, with each one's
    /// time (if it finished) and whether it was out.
    pub places: Vec<Place>,
    /// The bikes of the players (of the roster), player 1 first.
    pub players: Vec<usize>,
    /// Against the clock: the laps done and whether they are a record.
    pub laps: u32,
    pub record: bool,
}

/// A bike's place in a race.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub rider: usize,
    pub time: Option<f32>,
    pub out: bool,
}

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rules>()
            .init_resource::<Championship>()
            .init_resource::<Standing>()
            .add_plugins((hud::RaceHudPlugin, results::ResultsPlugin))
            // Over, the players' controls no longer drive the bikes nor
            // restart the race: they work the results.
            .configure_sets(
                RunFixedMainLoop,
                RaceControls.run_if(not(resource_exists::<Outcome>)),
            )
            .add_systems(OnEnter(GameState::Race), start_race)
            .add_systems(
                Update,
                restart_elimination.run_if(
                    in_state(GameState::Race)
                        .and_then(pause::running)
                        .and_then(not(resource_exists::<Outcome>)),
                ),
            )
            .add_systems(
                Update,
                (attach_progress, follow_the_race, eliminate, finish_the_race)
                    .chain()
                    .run_if(in_state(GameState::Race).and_then(not(resource_exists::<Outcome>))),
            )
            .add_systems(OnExit(GameState::Race), clear_outcome);
    }
}

/// A new race: the clock at nought and nobody out.
fn start_race(mut commands: Commands, rules: Res<Rules>, track: Res<Track>) {
    let mut standing = Standing::default();
    if rules.mode == Mode::Checkpoint {
        standing.time_left = checkpoint_bonus(track.lap_length() * GATES[0]);
    }
    commands.insert_resource(standing);
    commands.remove_resource::<Outcome>();
}

fn clear_outcome(mut commands: Commands) {
    commands.remove_resource::<Outcome>();
}

/// Restarting an elimination once someone is out sets the race up again
/// from the start, those out back on the grid: the rivals out have left
/// it, so parking the bikes there (as [`crate::bike`] does) is not enough.
fn restart_elimination(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    settings: Res<Settings>,
    rules: Res<Rules>,
    standing: Res<Standing>,
    mut next: ResMut<NextState<GameState>>,
) {
    if rules.mode == Mode::Elimination
        && !standing.out.is_empty()
        && settings
            .controls
            .just_pressed(Action::Restart, &keyboard, gamepads.iter())
    {
        next.set(GameState::Race);
    }
}

/// Every bike of the race keeps count of its laps.
#[allow(clippy::type_complexity)]
fn attach_progress(
    mut commands: Commands,
    bikes: Query<Entity, (With<BikeState>, With<Rider>, Without<Progress>)>,
) {
    for bike in &bikes {
        commands.entity(bike).insert(Progress::default());
    }
}

/// The clock, each bike's laps, the order of the race and, against the
/// clock, the time left. Restarting the race starts it all again.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn follow_the_race(
    time: Res<Time>,
    race: Res<Race>,
    rules: Res<Rules>,
    track: Res<Track>,
    current: Res<CurrentCircuit>,
    mut settings: ResMut<Settings>,
    mut standing: ResMut<Standing>,
    mut was_started: Local<bool>,
    mut bikes: Query<(
        &BikeState,
        &Rider,
        &mut Progress,
        Has<Player>,
        Has<LocalPlayer>,
    )>,
) {
    let lap = track.lap_length();
    // Restarted: everyone back to the grid.
    if *was_started && !race.started {
        let left = (rules.mode == Mode::Checkpoint).then(|| checkpoint_bonus(lap * GATES[0]));
        *standing = Standing {
            time_left: left.unwrap_or(0.0),
            ..default()
        };
        for (_, _, mut progress, ..) in &mut bikes {
            *progress = Progress::default();
        }
    }
    *was_started = race.started;
    if !race.started {
        return;
    }
    standing.clock += time.delta_secs();
    let clock = standing.clock;
    let target = rules.mode.laps(rules.laps);
    let name = current.get().name;
    let mut record = None;
    for (state, _, mut progress, player, local) in &mut bikes {
        let nearest = track.nearest_at(state.position, state.elevation);
        let passed = progress.pass(track.along(&nearest), lap);
        let human = player || local;
        if passed.lap {
            let lap_time = clock - progress.lap_start;
            progress.lap_start = clock;
            if progress.best_lap.is_none_or(|best| lap_time < best) {
                progress.best_lap = Some(lap_time);
            }
            if human && settings.records.lap(name, lap_time) {
                record = Some(lap_time);
            }
            if target.is_some_and(|laps| progress.laps >= laps) && progress.finished.is_none() {
                progress.finished = Some(clock);
            }
        }
        if human && passed.gate && rules.mode == Mode::Checkpoint {
            progress.checkpoints += 1;
            let next = progress.checkpoints as usize % 3;
            let from = if next == 0 { 0.0 } else { GATES[next - 1] };
            let to = GATES.get(next).copied().unwrap_or(1.0);
            let bonus = checkpoint_bonus((to - from) * lap);
            standing.time_left += bonus;
            standing.bonus = Some((bonus, clock));
        }
    }
    if record.is_some() {
        standing.record = record;
    }
    if rules.mode == Mode::Checkpoint {
        standing.time_left = (standing.time_left - time.delta_secs()).max(0.0);
    }
    // A player who is out is in the race still, but in the order of those
    // out.
    let racing: Vec<(usize, &Progress)> = bikes
        .iter()
        .filter(|(_, rider, ..)| !standing.out.contains(&rider.0))
        .map(|(_, rider, progress, ..)| (rider.0, progress))
        .collect();
    standing.order = order(&racing, &standing.out, lap);
}

/// In elimination, each time the leader starts a new lap (but the last) the
/// last one is out: a rival leaves the race; a player stops where they
/// are.
#[allow(clippy::type_complexity)]
fn eliminate(
    mut commands: Commands,
    rules: Res<Rules>,
    mut standing: ResMut<Standing>,
    bikes: Query<(Entity, &Rider, &Progress, Has<Player>, Has<LocalPlayer>), Without<Retired>>,
) {
    if rules.mode != Mode::Elimination {
        return;
    }
    let target = rules.mode.laps(rules.laps).unwrap_or(u32::MAX);
    let leader = bikes.iter().map(|(_, _, p, ..)| p.laps).max().unwrap_or(0);
    if leader <= standing.eliminations || leader >= target || bikes.iter().count() <= 1 {
        return;
    }
    standing.eliminations = leader;
    let Some(&last) = standing
        .order
        .iter()
        .rev()
        .find(|rider| !standing.out.contains(rider))
    else {
        return;
    };
    let Some((bike, _, _, player, local)) = bikes.iter().find(|(_, rider, ..)| rider.0 == last)
    else {
        return;
    };
    if player || local {
        commands.entity(bike).insert(Retired);
    } else {
        commands.entity(bike).despawn();
    }
    standing.out.push(last);
    standing.last_out = Some((last, standing.clock));
}

/// The race is over when every player has crossed the line at the end of
/// it, or is out, or against the clock has no time left; a moment later the
/// results are shown, the players' bikes roll to a stop, and a
/// championship scores it.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn finish_the_race(
    mut commands: Commands,
    rules: Res<Rules>,
    current: Res<CurrentCircuit>,
    mut settings: ResMut<Settings>,
    mut standing: ResMut<Standing>,
    mut championship: ResMut<Championship>,
    bikes: Query<(
        Entity,
        &Rider,
        &Progress,
        Has<Player>,
        Has<LocalPlayer>,
        Has<Retired>,
    )>,
) {
    if rules.mode == Mode::Free {
        return;
    }
    // (player 1 or 2, their bike, whether they are done)
    let mut players: Vec<(usize, Entity, usize, bool)> = bikes
        .iter()
        .filter(|(_, _, _, player, local, _)| *player || *local)
        .map(|(bike, rider, progress, player, _, retired)| {
            let index = if player { 0 } else { 1 };
            (index, bike, rider.0, progress.finished.is_some() || retired)
        })
        .collect();
    players.sort_by_key(|(index, ..)| *index);
    let done = match rules.mode {
        Mode::Checkpoint => standing.time_left <= 0.0,
        _ => !players.is_empty() && players.iter().all(|(.., done)| *done),
    };
    if !done {
        return;
    }
    let clock = standing.clock;
    let done_at = *standing.done_at.get_or_insert(clock);
    if clock - done_at < RESULTS_AFTER {
        return;
    }
    let places: Vec<Place> = standing
        .order
        .iter()
        .map(|&rider| {
            let progress = bikes
                .iter()
                .find(|(_, r, ..)| r.0 == rider)
                .map(|(_, _, progress, ..)| progress);
            Place {
                rider,
                time: progress.and_then(|p| p.finished),
                out: standing.out.contains(&rider),
            }
        })
        .collect();
    let name = current.get().name;
    let laps = bikes
        .iter()
        .find(|(_, _, _, player, ..)| *player)
        .map_or(0, |(_, _, progress, ..)| progress.laps);
    let record =
        rules.mode == Mode::Checkpoint && laps > 0 && settings.records.checkpoint(name, laps);
    if rules.mode.is_championship() {
        let order: Vec<usize> = places.iter().map(|place| place.rider).collect();
        championship.score(&order);
        if rules.mode == Mode::Story {
            settings.records.story = (!championship.is_over()).then(|| championship.clone());
        }
    }
    for (_, bike, ..) in &players {
        commands.entity(*bike).insert(Retired);
    }
    commands.insert_resource(Outcome {
        mode: rules.mode,
        track: current.0,
        places,
        players: players.iter().map(|(_, _, rider, _)| *rider).collect(),
        laps,
        record,
    });
}

/// Puts a championship on its first race, or on the one it was left on.
pub fn start_championship(
    mode: Mode,
    settings: &Settings,
    championship: &mut Championship,
    current: &mut CurrentCircuit,
) {
    *championship = match (mode, &settings.records.story) {
        (Mode::Story, Some(saved)) if !saved.is_over() => saved.clone(),
        _ => Championship::default(),
    };
    current.0 = championship.round.min(CIRCUITS.len() - 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAP: f32 = 3000.0;

    /// Rides `progress` from `from` to `to` m along (forward, round the
    /// line if need be) in steps of 10 m.
    fn ride(progress: &mut Progress, from: f32, to: f32) -> Vec<Passed> {
        let length = (to - from).rem_euclid(LAP);
        (0..=(length / 10.0) as usize)
            .map(|k| progress.pass((from + k as f32 * 10.0).rem_euclid(LAP), LAP))
            .filter(|passed| passed.gate || passed.lap)
            .collect()
    }

    #[test]
    fn a_lap_counts_after_going_all_the_way_round() {
        let mut progress = Progress::default();
        // From the grid, behind the line: crossing it does not count.
        let passed = ride(&mut progress, LAP - 40.0, 100.0);
        assert!(passed.is_empty());
        assert_eq!(progress.laps, 0);
        // Round to the line: the two gates and the line.
        let passed = ride(&mut progress, 100.0, 20.0);
        assert_eq!(passed.len(), 3);
        assert!(passed[2].lap);
        assert_eq!(progress.laps, 1);
    }

    #[test]
    fn riding_back_and_forth_over_the_line_does_not_count() {
        let mut progress = Progress::default();
        ride(&mut progress, 100.0, LAP - 10.0);
        // Now back and forth over the line.
        for _ in 0..5 {
            progress.pass(LAP - 20.0, LAP);
            progress.pass(15.0, LAP);
        }
        assert_eq!(progress.laps, 1, "only the first time");
        progress.pass(LAP - 20.0, LAP);
        progress.pass(15.0, LAP);
        assert_eq!(progress.laps, 1);
    }

    #[test]
    fn the_order_is_by_finish_then_distance_then_who_went_out_last() {
        let lap = LAP;
        let at = |laps: u32, along: f32, finished: Option<f32>| Progress {
            laps,
            along,
            gates: 1,
            finished,
            ..default()
        };
        let (a, b, c, d) = (
            at(2, 500.0, None),
            at(3, 10.0, Some(200.0)),
            at(2, 900.0, None),
            at(3, 30.0, Some(190.0)),
        );
        let racing = [(0, &a), (1, &b), (2, &c), (3, &d)];
        assert_eq!(order(&racing, &[5, 4], lap), [3, 1, 2, 0, 4, 5]);
        // On the grid, behind the line, they are not a lap ahead.
        let grid = Progress {
            along: lap - 20.0,
            ..default()
        };
        let moving = Progress {
            along: 30.0,
            ..default()
        };
        assert_eq!(order(&[(0, &grid), (1, &moving)], &[], lap), [1, 0]);
    }

    #[test]
    fn a_championship_adds_up_points_race_after_race() {
        let mut championship = Championship::default();
        championship.score(&[3, 1, 0, 2, 4, 5, 6, 7]);
        championship.score(&[1, 3, 0, 2, 4, 5, 6, 7]);
        assert_eq!(championship.points[3], 10 + 8);
        assert_eq!(championship.points[1], 8 + 10);
        assert_eq!(championship.round, 2);
        assert_eq!(&championship.standings()[..3], &[1, 3, 0]);
        championship.round = CIRCUITS.len();
        assert!(championship.is_over());
    }

    #[test]
    fn records_keep_the_best() {
        let mut records = Records::default();
        assert!(records.lap("Bay", 70.0));
        assert!(!records.lap("Bay", 75.0));
        assert!(records.lap("Bay", 65.0));
        assert_eq!(records.best_lap("Bay"), Some(65.0));
        assert!(records.checkpoint("Bay", 2));
        assert!(!records.checkpoint("Bay", 1));
        assert_eq!(records.checkpoint_best("Bay"), Some(2));
    }

    #[test]
    fn each_mode_has_its_laps() {
        assert_eq!(Mode::Free.laps(3), None);
        assert_eq!(Mode::Checkpoint.laps(3), None);
        assert_eq!(Mode::Race.laps(7), Some(7));
        assert_eq!(Mode::Story.laps(7), Some(CHAMPIONSHIP_LAPS));
        assert_eq!(Mode::Elimination.laps(2), Some(CHAMPIONSHIP_LAPS));
        assert!(Mode::Story.is_championship() && Mode::Elimination.is_championship());
        assert!(!Mode::Race.is_championship());
        // The time against the clock grows with the stretch.
        assert!(checkpoint_bonus(1000.0) > checkpoint_bonus(500.0));
        assert!(checkpoint_bonus(1000.0) > 1000.0 / 80.0);
    }

    /// An elimination with four bikes of the roster (0, 1, 2 and 3), the
    /// player on 2, in the order 0, 1, 2, 3, the leader on `laps`.
    fn elimination(laps: u32) -> (App, Vec<Entity>) {
        let mut app = App::new();
        app.insert_resource(Rules {
            mode: Mode::Elimination,
            laps: DEFAULT_LAPS,
        })
        .insert_resource(Standing {
            order: vec![0, 1, 2, 3],
            ..default()
        })
        .add_systems(Update, eliminate);
        let bikes = (0..4)
            .map(|rider| {
                let mut bike = app.world_mut().spawn((
                    Rider(rider),
                    Progress {
                        laps: if rider == 0 { laps } else { 0 },
                        ..default()
                    },
                ));
                if rider == 2 {
                    bike.insert(Player);
                }
                bike.id()
            })
            .collect();
        (app, bikes)
    }

    #[test]
    fn each_lap_of_the_leader_puts_the_last_one_out() {
        // Nobody has finished a lap: nobody out.
        let (mut app, _) = elimination(0);
        app.update();
        assert!(app.world().resource::<Standing>().out.is_empty());
        // The leader finishes a lap: the last rival leaves the race...
        let (mut app, bikes) = elimination(1);
        app.update();
        let standing = app.world().resource::<Standing>();
        assert_eq!(standing.out, [3]);
        assert_eq!(standing.last_out.map(|(rider, _)| rider), Some(3));
        assert!(app.world().get_entity(bikes[3]).is_err());
        // ...and only once for that lap.
        app.update();
        assert_eq!(app.world().resource::<Standing>().out, [3]);
        // On the next, the last is the player: out, but the bike stays,
        // rolling to a stop.
        app.world_mut().get_mut::<Progress>(bikes[0]).unwrap().laps = 2;
        app.update();
        assert_eq!(app.world().resource::<Standing>().out, [3, 2]);
        assert!(app.world().get::<Retired>(bikes[2]).is_some());
        // No one goes out on the last lap: the finish decides it.
        app.world_mut().get_mut::<Progress>(bikes[0]).unwrap().laps = CHAMPIONSHIP_LAPS;
        app.update();
        assert_eq!(app.world().resource::<Standing>().out, [3, 2]);
    }
}
