//! The performance log: every few seconds, how fast the frames came (their
//! average rate, the slowest ones, the times at a few percentiles), how
//! long the simulation took of each (the CPU's share: the rest is drawing,
//! or waiting for the GPU), the CPU and memory the game and the system use,
//! how many entities there are, with what was being played and the graphics
//! options in use, written as one line of `key=value` pairs to a file of its
//! own for each session, beside the settings (on macOS,
//! `~/Library/Application Support/motorust/logs`). A stretch ends early when
//! anything it is noted with changes (the screen, the mode, an option, the
//! resolution), so no line mixes two of them. On the desktop only, and while
//! Options > Graphics > "Performance log" is on.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore, SystemInformationDiagnosticsPlugin};
use bevy::ecs::entity::Entities;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::window::PrimaryWindow;

use crate::circuits::CurrentCircuit;
use crate::menu::GameState;
use crate::pause::Pause;
use crate::race::Rules;
use crate::settings::Settings;
use crate::welcome::GameMode;
use crate::world::Dressed;
use crate::world::detail::Detail;

/// How long each line of the log covers (s), at most.
const STRETCH: f32 = 5.0;

pub struct PerfLogPlugin;

impl Plugin for PerfLogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerfLog>()
            .init_resource::<FrameStart>()
            .add_systems(First, mark_frame_start)
            .add_systems(Last, log_performance);
    }
}

/// What a stretch of the log is noted with: when any of it changes, a new
/// stretch begins.
#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub state: String,
    pub mode: String,
    pub players: u32,
    pub paused: bool,
    pub track: String,
    pub trees: &'static str,
    pub far_trees: bool,
    pub split_detail: bool,
    pub shadows: String,
    pub antialiasing: String,
    pub fps_limit: String,
    pub resolution: String,
    pub fullscreen: String,
    pub pipelined: bool,
    pub frame_latency: u32,
    pub window: String,
    pub ui_scale: f32,
}

impl Context {
    fn pairs(&self) -> String {
        format!(
            "state={} mode={} players={} paused={} track={:?} trees={} lighter_far_trees={} \
             lighter_split_screen={} shadows={} antialiasing={} fps_limit={} resolution={} fullscreen={} \
             pipelined_rendering={} frame_latency={} window={} ui_scale={:.2}",
            self.state,
            self.mode,
            self.players,
            self.paused,
            self.track,
            self.trees,
            self.far_trees,
            self.split_detail,
            self.shadows,
            self.antialiasing,
            self.fps_limit,
            self.resolution,
            self.fullscreen,
            self.pipelined,
            self.frame_latency,
            self.window,
            self.ui_scale,
        )
    }
}

/// How a stretch went: the frames' times (s), and how long the simulation
/// took of each (s, from the start of its update to the end).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stretch {
    pub frames: Vec<f32>,
    pub updates: Vec<f32>,
}

/// The time at `share` (0 to 1) of `times` (s), sorted, in ms.
fn percentile(sorted: &[f32], share: f32) -> f32 {
    let index = ((sorted.len() as f32 - 1.0) * share).round() as usize;
    sorted[index.min(sorted.len() - 1)] * 1000.0
}

/// What a stretch comes to: its average rate, the rate of its slowest 1 %
/// of frames, and its frames' times (ms) at the middle and at 95 %.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    pub fps: f32,
    pub low: f32,
    pub p50: f32,
    pub p95: f32,
}

impl Stretch {
    pub fn summary(&self) -> Option<Summary> {
        if self.frames.is_empty() {
            return None;
        }
        let mut sorted = self.frames.clone();
        sorted.sort_by(f32::total_cmp);
        let total: f32 = sorted.iter().sum();
        let slowest = (sorted.len() / 100).max(1);
        let low: f32 = sorted.iter().rev().take(slowest).sum::<f32>() / slowest as f32;
        Some(Summary {
            fps: sorted.len() as f32 / total.max(1e-6),
            low: 1.0 / low.max(1e-6),
            p50: percentile(&sorted, 0.5),
            p95: percentile(&sorted, 0.95),
        })
    }

    /// Its figures: frames, how long it lasted (s), the average rate, the
    /// rate of the slowest 1 % of the frames, the times (ms) at the middle,
    /// at 95 % and 99 %, and the longest; and the simulation's (ms) at the
    /// middle and at 95 %.
    pub fn figures(&self) -> Option<String> {
        if self.frames.is_empty() {
            return None;
        }
        let mut sorted = self.frames.clone();
        sorted.sort_by(f32::total_cmp);
        let total: f32 = sorted.iter().sum();
        let at = |share: f32| percentile(&sorted, share);
        let mut updates = self.updates.clone();
        updates.sort_by(f32::total_cmp);
        let update = if updates.is_empty() {
            String::new()
        } else {
            format!(
                " update_ms_p50={:.2} update_ms_p95={:.2}",
                percentile(&updates, 0.5),
                percentile(&updates, 0.95)
            )
        };
        // The slowest 1 % (at least one frame), as a rate.
        let slowest = (sorted.len() / 100).max(1);
        let low: f32 = sorted.iter().rev().take(slowest).sum::<f32>() / slowest as f32;
        Some(format!(
            "frames={} seconds={:.2} fps={:.1} fps_1pct_low={:.1} ms_p50={:.2} ms_p95={:.2} \
             ms_p99={:.2} ms_max={:.2}{update}",
            sorted.len(),
            total,
            sorted.len() as f32 / total.max(1e-6),
            1.0 / low.max(1e-6),
            at(0.5),
            at(0.95),
            at(0.99),
            sorted[sorted.len() - 1] * 1000.0,
        ))
    }
}

/// The log of this session: its file (opened when the first line is
/// written), when the stretch under way began and what it is noted with.
#[derive(Resource, Default)]
pub struct PerfLog {
    file: Option<File>,
    failed: bool,
    started: Option<f32>,
    context: Option<Context>,
    stretch: Stretch,
}

/// When this frame's update began.
#[derive(Resource, Default)]
struct FrameStart(Option<Instant>);

fn mark_frame_start(mut start: ResMut<FrameStart>) {
    start.0 = Some(Instant::now());
}

/// The CPU and memory the game and the whole system use, averaged over the
/// last few readings (taken about once a second), if they are measured
/// (see [`SystemInformationDiagnosticsPlugin`]): the game's CPU as a share
/// of all the cores together, as the system's.
fn usage(store: Option<&DiagnosticsStore>) -> String {
    let Some(store) = store else {
        return String::new();
    };
    let read = |path: &DiagnosticPath| store.get(path).and_then(|diagnostic| diagnostic.average());
    let mut pairs = String::new();
    for (key, path, unit) in [
        (
            "cpu_process_pct",
            SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE,
            "",
        ),
        (
            "cpu_system_pct",
            SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE,
            "",
        ),
        (
            "mem_process_gib",
            SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
            "",
        ),
        (
            "mem_system_pct",
            SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE,
            "",
        ),
    ] {
        if let Some(value) = read(&path) {
            pairs.push_str(&format!(" {key}={value:.2}{unit}"));
        }
    }
    pairs
}

/// The folder the logs go in: beside the settings.
pub(crate) fn folder() -> Option<PathBuf> {
    let var = |name| std::env::var_os(name).map(PathBuf::from);
    let base = if cfg!(target_os = "macos") {
        var("HOME")?.join("Library/Application Support")
    } else if cfg!(windows) {
        var("APPDATA")?
    } else {
        match var("XDG_CONFIG_HOME").filter(|path| path.is_absolute()) {
            Some(folder) => folder,
            None => var("HOME")?.join(".config"),
        }
    };
    Some(base.join("motorust").join("logs"))
}

impl PerfLog {
    /// Writes `line` to the file, opening it first (a new one for each
    /// session, named after when it began), with a heading.
    fn write(&mut self, line: &str, heading: impl FnOnce() -> String) {
        if self.failed || cfg!(target_arch = "wasm32") {
            return;
        }
        if self.file.is_none() {
            let opened = folder().and_then(|folder| {
                std::fs::create_dir_all(&folder).ok()?;
                let since = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |since| since.as_secs());
                let path = folder.join(format!("perf-{since}.log"));
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok()?;
                info!("Performance log: {}", path.display());
                Some((file, since))
            });
            let Some((mut file, since)) = opened else {
                warn!("The performance log cannot be written");
                self.failed = true;
                return;
            };
            let _ = writeln!(
                file,
                "# motorust {} perf log, unix time {since}",
                env!("CARGO_PKG_VERSION")
            );
            let _ = writeln!(file, "# {}", heading());
            self.file = Some(file);
        }
        if let Some(file) = &mut self.file
            && writeln!(file, "{line}").is_err()
        {
            self.failed = true;
        }
    }
}

/// Notes each frame's time, and every [`STRETCH`] s (or as soon as what it
/// is noted with changes) writes a line for the stretch.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn log_performance(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    state: Res<State<GameState>>,
    rules: Res<Rules>,
    game_mode: Res<GameMode>,
    pause: Res<Pause>,
    current: Res<CurrentCircuit>,
    (window, ui_scale, adapter, trees): (
        Query<&Window, With<PrimaryWindow>>,
        Option<Res<UiScale>>,
        Option<Res<RenderAdapterInfo>>,
        Query<(&Dressed, &InheritedVisibility)>,
    ),
    (start, store, entities, detail): (
        Res<FrameStart>,
        Option<Res<DiagnosticsStore>>,
        &Entities,
        Option<Res<Detail>>,
    ),
    mut log: ResMut<PerfLog>,
) {
    if !settings.performance_log {
        // Off: what was under way is dropped, and a new stretch begins
        // when it is on again.
        log.started = None;
        log.context = None;
        log.stretch = Stretch::default();
        return;
    }
    let now = time.elapsed_secs();
    let window = window.iter().next();
    let context = Context {
        state: format!("{:?}", state.get()),
        mode: format!("{:?}", rules.mode),
        players: if *game_mode == GameMode::SplitScreen {
            2
        } else {
            1
        },
        paused: pause.is_open(),
        track: current.get().name.to_string(),
        trees: if settings.modelled_trees {
            "new"
        } else {
            "classic"
        },
        far_trees: settings.lighter_far_trees,
        split_detail: settings.lighter_split_screen,
        // The quality chosen, and the level the shadows are drawn at now.
        shadows: format!(
            "{:?}/{:?}",
            settings.shadows,
            detail.as_deref().map(|detail| detail.level)
        ),
        antialiasing: format!("{:?}", settings.antialiasing),
        fps_limit: format!("{:?}", settings.fps),
        resolution: format!("{:?}", settings.resolution),
        fullscreen: if settings.fullscreen {
            format!("{:?}", settings.fullscreen_kind)
        } else {
            "no".into()
        },
        pipelined: settings.pipelined_rendering,
        frame_latency: settings.frame_latency,
        window: window.map_or_else(
            || "none".into(),
            |window| {
                format!(
                    "{}x{}@{}",
                    window.physical_width(),
                    window.physical_height(),
                    window.scale_factor()
                )
            },
        ),
        ui_scale: ui_scale.map_or(1.0, |scale| scale.0),
    };
    let changed = log.context.as_ref() != Some(&context);
    let due = log.started.is_some_and(|started| now - started >= STRETCH);
    if changed || due {
        if let (Some(figures), Some(noted)) = (log.stretch.figures(), log.context.clone()) {
            // The trees in sight now, by the mesh they are drawn with:
            // full, lighter, lightest.
            let mut levels = [0; 3];
            for (dressed, visible) in &trees {
                if visible.get() {
                    levels[dressed.level().min(2)] += 1;
                }
            }
            let levels = format!(" tree_levels={}/{}/{}", levels[0], levels[1], levels[2]);
            let line = format!(
                "t={now:.1} {figures}{} entities={} {}{levels}",
                usage(store.as_deref()),
                entities.len(),
                noted.pairs()
            );
            let gpu = adapter.as_deref().map_or_else(
                || "GPU unknown".to_string(),
                |adapter| {
                    format!(
                        "GPU {} ({:?}, {})",
                        adapter.name, adapter.backend, adapter.driver
                    )
                },
            );
            log.write(&line, || {
                format!(
                    "{gpu}; {} CPUs; {}",
                    std::thread::available_parallelism().map_or(0, |n| n.get()),
                    std::env::consts::OS
                )
            });
        }
        log.stretch = Stretch::default();
        log.started = Some(now);
        log.context = Some(context);
    }
    // This frame's time (the first after a stretch begins counts in it),
    // and its update's so far, nearly all of it.
    let delta = time.delta_secs();
    if delta > 0.0 {
        log.stretch.frames.push(delta);
    }
    if let Some(started) = start.0 {
        log.stretch.updates.push(started.elapsed().as_secs_f32());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stretch_reads_as_its_rate_and_its_slowest_frames() {
        assert_eq!(Stretch::default().figures(), None);
        // 99 frames at 60 fps and one of 100 ms.
        let mut frames = vec![1.0 / 60.0; 99];
        frames.push(0.1);
        let figures = Stretch {
            frames,
            updates: vec![0.004; 100],
        }
        .figures()
        .unwrap();
        assert!(
            figures.starts_with("frames=100 seconds=1.75 fps=57.1"),
            "{figures}"
        );
        assert!(figures.contains("fps_1pct_low=10.0"), "{figures}");
        assert!(figures.contains("ms_p50=16.67"), "{figures}");
        assert!(figures.contains("ms_max=100.00"), "{figures}");
        assert!(
            figures.ends_with("update_ms_p50=4.00 update_ms_p95=4.00"),
            "{figures}"
        );
    }
}
