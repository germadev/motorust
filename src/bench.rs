//! The benchmark (`make bench`, or the game started with `--bench` or
//! `MOTORUST_BENCH=1`): with no one at the controls, it rides every track
//! (or those in `MOTORUST_BENCH_TRACKS`, as `0,2`) in free ride on
//! autopilot, with one player and with two, under each setting worth
//! comparing (the shadows' levels, the ways of smoothing the edges, the
//! lighter far trees, less detail for two players), with the frame rate
//! unlimited. Each scenario is let settle a few seconds, then measured for
//! some more (`MOTORUST_BENCH_SECONDS`, 10 by default). The results go to
//! the terminal, as a table, and to a file beside the performance log
//! (`bench-<time>.txt`), and the game closes. The settings tried are never
//! saved.

use std::io::Write;

use bevy::prelude::*;

use crate::bike::{LocalPlayer, Player, Race, autopilot};
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::perf_log::{Stretch, Summary, folder};
use crate::race::{Mode, Rules};
use crate::settings::{Antialiasing, FpsLimit, Settings, ShadowQuality};
use crate::welcome::GameMode;

/// Seconds each scenario settles for before it is measured (new pipelines,
/// shadow maps, a world built), and is measured for by default.
const SETTLE: f32 = 4.0;
const MEASURE: f32 = 10.0;

/// Whether the game was started for the benchmark.
pub fn requested() -> bool {
    std::env::args().any(|arg| arg == "--bench")
        || std::env::var("MOTORUST_BENCH").is_ok_and(|value| value != "0")
}

/// The settings for the benchmark: the player's, never saved, with the
/// frame rate unlimited (to see how fast it can go) and the log on.
pub fn settings(mut settings: Settings) -> Settings {
    settings.transient = true;
    settings.fps = FpsLimit::Unlimited;
    settings.performance_log = true;
    settings
}

/// A scenario: a track, one player or two, and the settings that matter.
#[derive(Clone, Debug, PartialEq)]
pub struct Scenario {
    pub name: &'static str,
    pub track: usize,
    pub players: u32,
    pub shadows: ShadowQuality,
    pub antialiasing: Antialiasing,
    pub far_trees: bool,
    pub split_detail: bool,
}

/// Every scenario on each of `tracks`: one player as the game starts
/// (shadows high, multisampled, whole trees), then one thing changed at a
/// time; then two players, with less detail and with all of it.
pub fn scenarios(tracks: &[usize]) -> Vec<Scenario> {
    let base = |name, track| Scenario {
        name,
        track,
        players: 1,
        shadows: ShadowQuality::High,
        antialiasing: Antialiasing::Msaa,
        far_trees: false,
        split_detail: true,
    };
    tracks
        .iter()
        .flat_map(|&track| {
            [
                base("1p", track),
                Scenario {
                    shadows: ShadowQuality::Medium,
                    ..base("1p-shadows-medium", track)
                },
                Scenario {
                    shadows: ShadowQuality::Low,
                    ..base("1p-shadows-low", track)
                },
                Scenario {
                    antialiasing: Antialiasing::Fxaa,
                    ..base("1p-fxaa", track)
                },
                Scenario {
                    antialiasing: Antialiasing::Off,
                    ..base("1p-no-antialiasing", track)
                },
                Scenario {
                    far_trees: true,
                    ..base("1p-lighter-far-trees", track)
                },
                Scenario {
                    players: 2,
                    ..base("2p", track)
                },
                Scenario {
                    players: 2,
                    split_detail: false,
                    ..base("2p-full-detail", track)
                },
            ]
        })
        .collect()
}

/// The tracks asked for in `value` (as `0,2`), or all of them.
pub fn tracks(value: Option<&str>) -> Vec<usize> {
    let asked: Vec<usize> = value
        .unwrap_or_default()
        .split(',')
        .filter_map(|track| track.trim().parse().ok())
        .filter(|&track| track < CIRCUITS.len())
        .collect();
    if asked.is_empty() {
        (0..CIRCUITS.len()).collect()
    } else {
        asked
    }
}

/// Where the benchmark is at in its scenario.
#[derive(Clone, Debug, PartialEq)]
enum Phase {
    /// About to begin, the game started: its first scenario is set up as
    /// the menus would (not while the first world is being built).
    Start,
    /// The race being set up (a new track, or players), since when.
    Entering(f32),
    /// Settling, since when.
    Settling(f32),
    /// Being measured, since when.
    Measuring(f32),
}

#[derive(Resource)]
pub struct Bench {
    scenarios: Vec<Scenario>,
    index: usize,
    phase: Phase,
    measure: f32,
    stretch: Stretch,
    results: Vec<(Scenario, Option<Summary>)>,
}

pub struct BenchPlugin;

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        let measure = std::env::var("MOTORUST_BENCH_SECONDS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|&seconds: &f32| seconds > 0.0)
            .unwrap_or(MEASURE);
        let tracks = tracks(std::env::var("MOTORUST_BENCH_TRACKS").ok().as_deref());
        app.insert_resource(Bench {
            scenarios: scenarios(&tracks),
            index: 0,
            phase: Phase::Start,
            measure,
            stretch: Stretch::default(),
            results: Vec::new(),
        })
        .add_systems(Startup, start_bench)
        .add_systems(Update, run_bench);
    }
}

/// The settings, track and players of `scenario`; whether the race must be
/// set up again for them.
fn apply(
    scenario: &Scenario,
    settings: &mut Settings,
    current: &mut CurrentCircuit,
    game_mode: &mut GameMode,
) -> bool {
    settings.shadows = scenario.shadows;
    settings.antialiasing = scenario.antialiasing;
    settings.lighter_far_trees = scenario.far_trees;
    settings.lighter_split_screen = scenario.split_detail;
    let mode = if scenario.players == 2 {
        GameMode::SplitScreen
    } else {
        GameMode::SinglePlayer
    };
    let again = current.0 != scenario.track || *game_mode != mode;
    if current.0 != scenario.track {
        current.0 = scenario.track;
    }
    if *game_mode != mode {
        *game_mode = mode;
    }
    again
}

fn start_bench(bench: Res<Bench>, mut rules: ResMut<Rules>) {
    rules.mode = Mode::Free;
    info!("Benchmark: {} scenarios", bench.scenarios.len());
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn run_bench(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut bench: ResMut<Bench>,
    mut settings: ResMut<Settings>,
    mut current: ResMut<CurrentCircuit>,
    mut game_mode: ResMut<GameMode>,
    mut next: ResMut<NextState<GameState>>,
    mut race: ResMut<Race>,
    players: Query<Entity, Or<(With<Player>, With<LocalPlayer>)>>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = time.elapsed_secs();
    // The race on as soon as there is one: no lights, the players' bikes
    // ridden by the machine.
    if *state.get() == GameState::Race && !race.started {
        race.started = true;
    }
    match bench.phase.clone() {
        Phase::Start => {
            let Some(first) = bench.scenarios.first().cloned() else {
                exit.write(AppExit::Success);
                return;
            };
            apply(&first, &mut settings, &mut current, &mut game_mode);
            next.set(GameState::Race);
            bench.phase = Phase::Entering(now);
        }
        Phase::Entering(since) => {
            if *state.get() != GameState::Race || players.is_empty() || now - since < 0.5 {
                return;
            }
            for bike in &players {
                autopilot(&mut commands.entity(bike), 0.0);
            }
            bench.phase = Phase::Settling(now);
        }
        Phase::Settling(since) => {
            if now - since >= SETTLE {
                bench.stretch = Stretch::default();
                bench.phase = Phase::Measuring(now);
            }
        }
        Phase::Measuring(since) => {
            bench.stretch.frames.push(time.delta_secs());
            if now - since < bench.measure {
                return;
            }
            let scenario = bench.scenarios[bench.index].clone();
            let summary = bench.stretch.summary();
            info!(
                "Benchmark {}/{}: {} {} {}",
                bench.index + 1,
                bench.scenarios.len(),
                CIRCUITS[scenario.track].name,
                scenario.name,
                summary.map_or("no frames".into(), |s| format!("{:.1} fps", s.fps))
            );
            bench.results.push((scenario, summary));
            bench.index += 1;
            let Some(next_scenario) = bench.scenarios.get(bench.index).cloned() else {
                report(&bench.results);
                exit.write(AppExit::Success);
                return;
            };
            if apply(&next_scenario, &mut settings, &mut current, &mut game_mode) {
                next.set(GameState::Race);
                bench.phase = Phase::Entering(now);
            } else {
                bench.phase = Phase::Settling(now);
            }
        }
    }
}

/// The results as a table: a line for each scenario.
pub fn table(results: &[(Scenario, Option<Summary>)]) -> String {
    let mut lines = vec![format!(
        "{:<16} {:<22} {:>8} {:>10} {:>8} {:>8}",
        "track", "scenario", "fps", "1% low", "p50 ms", "p95 ms"
    )];
    for (scenario, summary) in results {
        let track = CIRCUITS[scenario.track].name;
        lines.push(match summary {
            Some(s) => format!(
                "{track:<16} {:<22} {:>8.1} {:>10.1} {:>8.2} {:>8.2}",
                scenario.name, s.fps, s.low, s.p50, s.p95
            ),
            None => format!("{track:<16} {:<22} {:>8}", scenario.name, "-"),
        });
    }
    lines.join("\n")
}

/// Prints the results and writes them beside the performance log.
fn report(results: &[(Scenario, Option<Summary>)]) {
    let table = table(results);
    println!("\nmotorust benchmark\n{table}\n");
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let written = folder().and_then(|folder| {
        std::fs::create_dir_all(&folder).ok()?;
        let path = folder.join(format!("bench-{since}.txt"));
        let mut file = std::fs::File::create(&path).ok()?;
        writeln!(
            file,
            "# motorust {} benchmark, unix time {since}",
            env!("CARGO_PKG_VERSION")
        )
        .ok()?;
        writeln!(file, "{table}").ok()?;
        Some(path)
    });
    match written {
        Some(path) => info!("Benchmark results: {}", path.display()),
        None => warn!("The benchmark results cannot be written"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_is_ridden_under_each_setting_with_one_change_at_a_time() {
        let all = scenarios(&tracks(None));
        assert_eq!(all.len(), 8 * CIRCUITS.len());
        let first = &all[..8];
        assert!(first.iter().all(|scenario| scenario.track == 0));
        // The first as the game starts; the others one change from it.
        let base = &first[0];
        assert_eq!(
            (
                base.players,
                base.shadows,
                base.antialiasing,
                base.far_trees
            ),
            (1, ShadowQuality::High, Antialiasing::Msaa, false)
        );
        for scenario in &first[1..] {
            let changes = [
                scenario.players != base.players,
                scenario.shadows != base.shadows,
                scenario.antialiasing != base.antialiasing,
                scenario.far_trees != base.far_trees,
                scenario.split_detail != base.split_detail && scenario.players == 2,
            ]
            .iter()
            .filter(|&&changed| changed)
            .count();
            assert!((1..=2).contains(&changes), "{}", scenario.name);
        }
        // The players change once a track, after all of one player's.
        let two = first
            .iter()
            .position(|scenario| scenario.players == 2)
            .unwrap();
        assert!(first[two..].iter().all(|scenario| scenario.players == 2));
    }

    #[test]
    fn the_tracks_asked_for_or_all() {
        assert_eq!(tracks(None), (0..CIRCUITS.len()).collect::<Vec<_>>());
        assert_eq!(tracks(Some("0, 2")), [0, 2]);
        assert_eq!(
            tracks(Some("99,x")),
            (0..CIRCUITS.len()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_results_read_as_a_table() {
        let scenario = scenarios(&[0])[0].clone();
        let summary = Summary {
            fps: 72.25,
            low: 41.0,
            p50: 13.5,
            p95: 18.25,
        };
        let table = table(&[(scenario.clone(), Some(summary)), (scenario, None)]);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("track"));
        assert!(
            lines[1].contains("72.2") || lines[1].contains("72.3"),
            "{table}"
        );
        assert!(lines[1].contains("18.25"), "{table}");
        assert!(lines[2].trim_end().ends_with('-'));
    }
}
