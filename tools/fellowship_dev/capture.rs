//! A temporary capture harness (not committed): loads a circuit, puts the
//! camera at the poses listed in `MOTORUST_SHOTS` (lines of `name x y z tx
//! ty tz`: where the camera is and what it looks at) and saves a screenshot
//! of each to `MOTORUST_SHOTS_OUT`, then quits.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::transform::TransformSystems;

use crate::circuits::CurrentCircuit;
use crate::menu::GameState;
use crate::race::{Mode, Rules};

pub struct CapturePlugin;

#[derive(Resource)]
struct Shots {
    poses: Vec<(String, Vec3, Vec3)>,
    out: String,
    index: usize,
    frames: u32,
    entered: Option<f32>,
    circuit: usize,
}

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let list = std::env::var("MOTORUST_SHOTS").expect("MOTORUST_SHOTS");
        let poses = std::fs::read_to_string(list)
            .expect("the shots")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let n: Vec<f32> = parts[1..7].iter().map(|v| v.parse().unwrap()).collect();
                (
                    parts[0].to_string(),
                    Vec3::new(n[0], n[1], n[2]),
                    Vec3::new(n[3], n[4], n[5]),
                )
            })
            .collect();
        let circuit = std::env::var("MOTORUST_SHOTS_CIRCUIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        app.insert_resource(Shots {
            poses,
            out: std::env::var("MOTORUST_SHOTS_OUT").unwrap_or_else(|_| "shots".into()),
            index: 0,
            frames: 0,
            entered: None,
            circuit,
        })
        .add_systems(Update, step)
        .add_systems(PostUpdate, place_camera.before(TransformSystems::Propagate));
    }
}

fn step(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut shots: ResMut<Shots>,
    mut rules: ResMut<Rules>,
    mut current: ResMut<CurrentCircuit>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = time.elapsed_secs();
    if *state.get() != GameState::Race {
        if shots.entered.is_none() {
            rules.mode = Mode::Free;
            current.0 = shots.circuit;
            next.set(GameState::Race);
        }
        return;
    }
    let entered = *shots.entered.get_or_insert(now);
    // Let the world be built and its shaders compiled.
    if now - entered < 40.0 {
        return;
    }
    shots.frames += 1;
    if shots.frames == 14 {
        let (name, ..) = shots.poses[shots.index].clone();
        let path = format!("{}/{name}.png", shots.out);
        println!("shot {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if shots.frames >= 22 {
        shots.frames = 0;
        shots.index += 1;
        if shots.index >= shots.poses.len() {
            exit.write(AppExit::Success);
        }
    }
}

fn place_camera(shots: Res<Shots>, mut cameras: Query<&mut Transform, With<Camera3d>>) {
    let Some((_, at, look)) = shots.poses.get(shots.index) else {
        return;
    };
    if shots.entered.is_none() {
        return;
    }
    for mut transform in &mut cameras {
        *transform = Transform::from_translation(*at).looking_at(*look, Vec3::Y);
    }
}
