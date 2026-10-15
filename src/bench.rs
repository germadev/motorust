//! The benchmark (`make bench`, or the game started with `--bench` or
//! `MOTORUST_BENCH=1`): with no one at the controls, it rides every track
//! (or those in `MOTORUST_BENCH_TRACKS`, as `0,2`) in free ride on
//! autopilot, with one player and with two, under each setting worth
//! comparing (the shadows' levels, the ways of smoothing the edges, the
//! lighter far trees, less detail for two players), with the frame rate
//! unlimited. Each scenario starts the race afresh, so all of them ride the
//! same stretch of road and differ only in what they try; it is let settle
//! a few seconds, then measured for some more (`MOTORUST_BENCH_SECONDS`, 10
//! by default). The first is ridden once before, unmeasured, while the
//! shaders are compiled and the caches fill. The game's share of the CPU is
//! read over each measured stretch.
//!
//! It says what it is doing in the terminal as it goes: the machine, where
//! the results go, each scenario as it starts and what it measured. The
//! results are written as they are measured to `bench-results/` (in the
//! folder it is run from: the repository, with `make bench`), one file for
//! each run named after when and on which machine (`2026-10-16_1830_
//! mac-mini.txt`), headed with the machine, with the performance log of the
//! run beside it (`.perf.log`); so runs on several machines can be kept
//! side by side, committed and compared. Stopping it (Esc, which also
//! prints the table so far, closing the window, or Ctrl+C in the terminal)
//! keeps what was measured. The settings tried are never saved.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use bevy::diagnostic::{DiagnosticsStore, SystemInfo};
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::window::PrimaryWindow;

use crate::bike::{LocalPlayer, Player, Race, autopilot};
use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::perf_log::{CpuClock, FrameStart, PerfLog, Stretch, Summary, folder, process_cpu};
use crate::race::{Mode, Rules};
use crate::settings::{Antialiasing, FpsLimit, Settings, ShadowQuality};
use crate::welcome::GameMode;

/// Seconds each scenario settles for before it is measured (new pipelines,
/// shadow maps, a world built), and is measured for by default.
const SETTLE: f32 = 4.0;
const MEASURE: f32 = 10.0;
/// About how long setting up a race takes (s), building its world.
const SETTING_UP: f32 = 2.5;
/// The folder the results go in, in the folder the game is run from.
const RESULTS: &str = "bench-results";

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

/// `seconds` since 1970 as a date and time (UTC) for a file name:
/// `2026-10-16_1830`.
pub fn stamp(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let (hour, minute) = (seconds % 86_400 / 3600, seconds % 3600 / 60);
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}_{hour:02}{minute:02}")
}

/// A machine's name as it can go in a file name: lowercase letters, digits
/// and dashes, without `.local`.
pub fn file_name(machine: &str) -> String {
    let name = machine.trim().trim_end_matches(".local").to_lowercase();
    let name: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let name = name.trim_matches('-').to_string();
    if name.is_empty() {
        "machine".into()
    } else {
        name
    }
}

/// This machine's name, as the system gives it.
fn machine() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .unwrap_or_default()
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
    /// The game's CPU time when the measuring began.
    cpu: Option<CpuClock>,
    /// Whether the first scenario has been ridden once, unmeasured.
    warmed: bool,
    results: Vec<(Scenario, Option<Summary>, Option<f32>)>,
    /// This machine's name, where the results go and the file they are
    /// written to as they come.
    machine: String,
    path: PathBuf,
    file: Option<File>,
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
        let machine = machine();
        let since = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let name = format!("{}_{}", stamp(since), file_name(&machine));
        // In the folder it is run from, or else beside the performance log.
        let results = std::env::current_dir()
            .map(|here| here.join(RESULTS))
            .ok()
            .filter(|folder| std::fs::create_dir_all(folder).is_ok())
            .or_else(folder)
            .unwrap_or_else(|| PathBuf::from(RESULTS));
        // The performance log of the run, beside its results.
        app.insert_resource(PerfLog::to(results.join(format!("{name}.perf.log"))))
            .insert_resource(Bench {
                scenarios: scenarios(&tracks),
                index: 0,
                phase: Phase::Start,
                measure,
                stretch: Stretch::default(),
                cpu: None,
                warmed: false,
                results: Vec::new(),
                machine: machine.trim().to_string(),
                path: results.join(format!("{name}.txt")),
                file: None,
            })
            .add_systems(Startup, start_bench)
            .add_systems(Update, run_bench);
    }
}

/// The settings, track and players of `scenario`, for the race to be set
/// up again with.
fn apply(
    scenario: &Scenario,
    settings: &mut Settings,
    current: &mut CurrentCircuit,
    game_mode: &mut GameMode,
) {
    settings.shadows = scenario.shadows;
    settings.antialiasing = scenario.antialiasing;
    settings.lighter_far_trees = scenario.far_trees;
    settings.lighter_split_screen = scenario.split_detail;
    let mode = if scenario.players == 2 {
        GameMode::SplitScreen
    } else {
        GameMode::SinglePlayer
    };
    if current.0 != scenario.track {
        current.0 = scenario.track;
    }
    if *game_mode != mode {
        *game_mode = mode;
    }
}

fn start_bench(mut rules: ResMut<Rules>) {
    rules.mode = Mode::Free;
}

/// What the benchmark runs on: the machine, its GPU and system, the
/// window, and the settings it starts from, as the lines of a heading.
fn machine_lines(
    bench: &Bench,
    system: Option<&SystemInfo>,
    adapter: Option<&RenderAdapterInfo>,
    window: Option<&Window>,
    settings: &Settings,
) -> Vec<String> {
    let mut lines = vec![format!(
        "motorust {} benchmark on {}",
        env!("CARGO_PKG_VERSION"),
        if bench.machine.is_empty() {
            "this machine"
        } else {
            &bench.machine
        }
    )];
    if let Some(system) = system {
        lines.push(format!(
            "system: {}; CPU {} ({} cores); memory {}",
            system.os, system.cpu, system.core_count, system.memory
        ));
    }
    if let Some(adapter) = adapter {
        lines.push(format!(
            "GPU: {} ({:?}, {})",
            adapter.name, adapter.backend, adapter.driver
        ));
    }
    if let Some(window) = window {
        lines.push(format!(
            "window: {} x {} pixels at {}x; resolution {:?}, fullscreen {}",
            window.physical_width(),
            window.physical_height(),
            window.scale_factor(),
            settings.resolution,
            if settings.fullscreen {
                format!("{:?}", settings.fullscreen_kind)
            } else {
                "no".into()
            }
        ));
    }
    lines.push(format!(
        "trees {}; pipelined rendering {}; frames queued {}; frame rate unlimited",
        if settings.modelled_trees {
            "new"
        } else {
            "classic"
        },
        settings.pipelined_rendering,
        settings.frame_latency
    ));
    lines.push(format!(
        "{} scenarios, each a race afresh, {SETTLE} s settling and {} s measured, \
         after one to warm up; about {} min",
        bench.scenarios.len(),
        bench.measure,
        minutes(bench.scenarios.len() + 1, bench.measure)
    ));
    lines
}

/// About how many minutes `scenarios` take, measured `measure` s each.
fn minutes(scenarios: usize, measure: f32) -> u32 {
    (scenarios as f32 * (SETTING_UP + SETTLE + measure) / 60.0).ceil() as u32
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn run_bench(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut bench: ResMut<Bench>,
    mut settings: ResMut<Settings>,
    (mut current, mut game_mode, mut next, mut race): (
        ResMut<CurrentCircuit>,
        ResMut<GameMode>,
        ResMut<NextState<GameState>>,
        ResMut<Race>,
    ),
    players: Query<Entity, Or<(With<Player>, With<LocalPlayer>)>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    (frame_start, store, system, adapter, window): (
        Res<FrameStart>,
        Option<Res<DiagnosticsStore>>,
        Option<Res<SystemInfo>>,
        Option<Res<RenderAdapterInfo>>,
        Query<&Window, With<PrimaryWindow>>,
    ),
    mut exit: MessageWriter<AppExit>,
) {
    // Stopped: what has been measured is kept.
    if keyboard.just_pressed(KeyCode::Escape) {
        let (done, total) = (bench.results.len(), bench.scenarios.len());
        if let Some(file) = &mut bench.file {
            let _ = writeln!(file, "# stopped after {done} of {total}");
        }
        println!("\nStopped after {done} of {total}.");
        report(&bench);
        exit.write(AppExit::Success);
        return;
    }
    let now = time.elapsed_secs();
    // The race on as soon as there is one: no lights, the players' bikes
    // ridden by the machine.
    if *state.get() == GameState::Race && !race.started {
        race.started = true;
    }
    let total = bench.scenarios.len();
    let label = |bench: &Bench| {
        let scenario = &bench.scenarios[bench.index];
        format!(
            "[{:>2}/{total}] {} · {}",
            bench.index + 1,
            CIRCUITS[scenario.track].name,
            scenario.name
        )
    };
    match bench.phase.clone() {
        Phase::Start => {
            let Some(first) = bench.scenarios.first().cloned() else {
                exit.write(AppExit::Success);
                return;
            };
            // The heading, in the terminal and in the file.
            let heading = machine_lines(
                &bench,
                system.as_deref(),
                adapter.as_deref(),
                window.iter().next(),
                &settings,
            );
            println!();
            for line in &heading {
                println!("{line}");
            }
            let path = bench.path.clone();
            bench.file = File::create(&path).ok().and_then(|mut file| {
                for line in &heading {
                    writeln!(file, "# {line}").ok()?;
                }
                writeln!(file, "{}", header()).ok()?;
                Some(file)
            });
            match &bench.file {
                Some(_) => println!("results: {}", path.display()),
                None => println!("the results cannot be written to {}", path.display()),
            }
            println!("Esc stops it, keeping what it has measured.\n");
            apply(&first, &mut settings, &mut current, &mut game_mode);
            next.set(GameState::Race);
            println!("{}: setting up the race", label(&bench));
            bench.phase = Phase::Entering(now);
        }
        Phase::Entering(since) => {
            if *state.get() != GameState::Race || players.is_empty() || now - since < 0.5 {
                return;
            }
            for bike in &players {
                autopilot(&mut commands.entity(bike), 0.0);
            }
            if bench.warmed {
                println!(
                    "{}: settling {SETTLE} s, then measuring {} s",
                    label(&bench),
                    bench.measure
                );
            } else {
                println!(
                    "{}: warming up {} s, not measured",
                    label(&bench),
                    SETTLE + bench.measure
                );
            }
            bench.phase = Phase::Settling(now);
        }
        Phase::Settling(since) => {
            if now - since >= SETTLE {
                bench.stretch = Stretch::default();
                bench.cpu = CpuClock::start();
                bench.phase = Phase::Measuring(now);
            }
        }
        Phase::Measuring(since) => {
            bench.stretch.frames.push(time.delta_secs());
            if let Some(update) = frame_start.elapsed() {
                bench.stretch.updates.push(update);
            }
            if now - since < bench.measure {
                return;
            }
            // Ridden once to warm up: now again, measured.
            if !bench.warmed {
                bench.warmed = true;
                next.set(GameState::Race);
                println!("{}: setting up the race again", label(&bench));
                bench.phase = Phase::Entering(now);
                return;
            }
            let scenario = bench.scenarios[bench.index].clone();
            let summary = bench.stretch.summary();
            // The game's share of all the cores over the stretch.
            let cpu = bench
                .cpu
                .and_then(|clock| clock.share())
                .or_else(|| process_cpu(store.as_deref()));
            let left = minutes(total - bench.index - 1, bench.measure);
            println!(
                "{}: {}{}",
                label(&bench),
                said(summary.as_ref(), cpu),
                if left > 0 {
                    format!("  (about {left} min left)")
                } else {
                    String::new()
                }
            );
            if let Some(file) = &mut bench.file {
                let _ = writeln!(file, "{}", row(&scenario, summary.as_ref(), cpu));
                let _ = file.flush();
            }
            bench.results.push((scenario, summary, cpu));
            bench.index += 1;
            let Some(next_scenario) = bench.scenarios.get(bench.index).cloned() else {
                if let Some(file) = &mut bench.file {
                    let _ = writeln!(file, "# finished");
                }
                report(&bench);
                exit.write(AppExit::Success);
                return;
            };
            // The race afresh, so that each scenario rides the same road.
            apply(&next_scenario, &mut settings, &mut current, &mut game_mode);
            next.set(GameState::Race);
            println!("{}: setting up the race", label(&bench));
            bench.phase = Phase::Entering(now);
        }
    }
}

/// What a scenario measured, said in a line of the terminal.
fn said(summary: Option<&Summary>, cpu: Option<f32>) -> String {
    let Some(s) = summary else {
        return "no frames".into();
    };
    let mut line = format!(
        "{:.1} fps · 1% low {:.1} · {:.2} ms (95%: {:.2} ms)",
        s.fps, s.low, s.p50, s.p95
    );
    if let Some(update) = s.update {
        line.push_str(&format!(" · simulation {update:.2} ms"));
    }
    if let Some(cpu) = cpu {
        line.push_str(&format!(" · CPU {cpu:.0} %"));
    }
    line
}

/// The heading of the table of results.
fn header() -> String {
    format!(
        "{:<16} {:<22} {:>8} {:>10} {:>8} {:>8} {:>8} {:>6}",
        "track", "scenario", "fps", "1% low", "p50 ms", "p95 ms", "sim ms", "cpu %"
    )
}

/// A line of the table: a scenario and what it measured.
fn row(scenario: &Scenario, summary: Option<&Summary>, cpu: Option<f32>) -> String {
    let track = CIRCUITS[scenario.track].name;
    let or_dash = |value: Option<f32>, decimals: usize| {
        value.map_or("-".to_string(), |value| format!("{value:.decimals$}"))
    };
    match summary {
        Some(s) => format!(
            "{track:<16} {:<22} {:>8.1} {:>10.1} {:>8.2} {:>8.2} {:>8} {:>6}",
            scenario.name,
            s.fps,
            s.low,
            s.p50,
            s.p95,
            or_dash(s.update, 2),
            or_dash(cpu, 0)
        ),
        None => format!("{track:<16} {:<22} {:>8}", scenario.name, "-"),
    }
}

/// The results as a table: a line for each scenario.
pub fn table(results: &[(Scenario, Option<Summary>, Option<f32>)]) -> String {
    std::iter::once(header())
        .chain(
            results
                .iter()
                .map(|(scenario, summary, cpu)| row(scenario, summary.as_ref(), *cpu)),
        )
        .collect::<Vec<_>>()
        .join("\n")
}

/// Prints the results (they are in their file already).
fn report(bench: &Bench) {
    println!("\n{}\n", table(&bench.results));
    if bench.file.is_some() {
        println!("Written to {}", bench.path.display());
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
            update: Some(2.5),
        };
        let table = table(&[
            (scenario.clone(), Some(summary), Some(34.0)),
            (scenario, None, None),
        ]);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("track"));
        assert!(
            lines[1].contains("72.2") || lines[1].contains("72.3"),
            "{table}"
        );
        assert!(
            lines[1].contains("18.25") && lines[1].contains("2.50"),
            "{table}"
        );
        assert!(lines[1].trim_end().ends_with("34"), "{table}");
        assert!(lines[2].trim_end().ends_with('-'));
        assert!(said(Some(&summary), Some(34.0)).contains("simulation 2.50 ms · CPU 34 %"));
    }

    #[test]
    fn each_run_is_named_after_when_and_where() {
        assert_eq!(stamp(0), "1970-01-01_0000");
        assert_eq!(stamp(1_792_083_939), "2026-10-15_1705");
        // The last day of a leap February.
        assert_eq!(
            stamp(1_709_164_800 + 3_600 * 23 + 59 * 60),
            "2024-02-29_2359"
        );
        assert_eq!(file_name("Joshuas-Mac-mini.local\n"), "joshuas-mac-mini");
        assert_eq!(file_name("DESKTOP 42"), "desktop-42");
        assert_eq!(file_name("  "), "machine");
    }
}
