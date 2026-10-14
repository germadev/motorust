//! The start lights, as in a Grand Prix: a moment after the grid is set
//! up, five red lights come on one by one, a second apart, and a moment
//! after the last one (never quite the same) they go out and the race
//! starts. Until then every bike waits on the grid, though the players can
//! rev their engines ([`Race::held`]). Each light beeps as it comes on, and
//! going out beeps higher, while the lights flash green.
//!
//! In the options ([`RaceStart`]) the race can start as it used to
//! instead, when a player first opens the throttle, with no lights.

use std::ops::Range;

use bevy::prelude::*;

use crate::bike::Race;
use crate::menu::GameState;
use crate::pause;
use crate::settings::{RaceStart, Settings};
use crate::welcome::GameMode;

/// How many red lights there are.
pub const LIGHTS: u32 = 5;
/// Seconds from the grid being set up to the first light, and between one
/// light and the next.
const FIRST_LIGHT: f32 = 1.0;
const BETWEEN: f32 = 1.0;
/// Seconds the five lights stay on before they go out: somewhere in this
/// range, so that the start cannot be timed.
const HOLD: Range<f32> = 0.3..1.3;
/// Seconds the lights show green once they are out.
const GREEN_FOR: f32 = 0.8;

const RED: Color = Color::srgb(1.0, 0.1, 0.06);
const GREEN: Color = Color::srgb(0.15, 0.95, 0.3);
const DARK: Color = Color::srgb(0.12, 0.12, 0.13);

pub struct StartLightsPlugin;

impl Plugin for StartLightsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StartLights>()
            .add_message::<StartBeep>()
            .add_systems(OnEnter(GameState::Race), (set_the_lights, spawn_lights))
            .add_systems(
                Update,
                (run_lights.run_if(pause::running), show_lights)
                    .chain()
                    .run_if(in_state(GameState::Race)),
            );
    }
}

/// A beep of the lights: one coming on, or all of them going out.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StartBeep {
    Light,
    Go,
}

/// Where the start is at.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct StartLights {
    /// Seconds since the grid was set up.
    elapsed: f32,
    /// When the lights go out (s since the grid was set up).
    out_at: f32,
    /// Seconds since they went out, once they have.
    since_out: Option<f32>,
}

impl Default for StartLights {
    fn default() -> Self {
        Self::new(0.5)
    }
}

impl StartLights {
    /// The lights of a new start, staying on for a share `chance` (0 to 1)
    /// of [`HOLD`] once all five are lit.
    pub fn new(chance: f32) -> Self {
        let last = FIRST_LIGHT + (LIGHTS - 1) as f32 * BETWEEN;
        Self {
            elapsed: 0.0,
            out_at: last + HOLD.start + (HOLD.end - HOLD.start) * chance.clamp(0.0, 1.0),
            since_out: None,
        }
    }

    /// How many lights are on.
    pub fn lit(&self) -> u32 {
        if self.elapsed >= self.out_at || self.elapsed < FIRST_LIGHT {
            return 0;
        }
        (((self.elapsed - FIRST_LIGHT) / BETWEEN) as u32 + 1).min(LIGHTS)
    }

    /// Whether they have gone out: the race is on.
    pub fn out(&self) -> bool {
        self.elapsed >= self.out_at
    }

    /// `dt` s later: what beeps.
    pub fn advance(&mut self, dt: f32) -> Option<StartBeep> {
        if let Some(since) = &mut self.since_out {
            *since += dt;
            return None;
        }
        let (lit, out) = (self.lit(), self.out());
        self.elapsed += dt;
        if self.out() && !out {
            self.since_out = Some(0.0);
            Some(StartBeep::Go)
        } else {
            (self.lit() > lit).then_some(StartBeep::Light)
        }
    }
}

/// A share (0 to 1) that is hard to guess: from the nanoseconds of the
/// clock.
fn chance(time: &Time<Real>) -> f32 {
    let nanos = time.elapsed().subsec_nanos();
    let mixed = nanos.wrapping_mul(2_654_435_761) >> 8;
    (mixed % 1000) as f32 / 1000.0
}

/// A new race: new lights.
fn set_the_lights(mut lights: ResMut<StartLights>, time: Res<Time<Real>>) {
    *lights = StartLights::new(chance(&time));
}

/// With a start at the lights, the bikes wait for them; once out, the race
/// is on. Restarting the race lights them again.
fn run_lights(
    time: Res<Time>,
    real: Res<Time<Real>>,
    settings: Res<Settings>,
    mut race: ResMut<Race>,
    mut lights: ResMut<StartLights>,
    mut beeps: MessageWriter<StartBeep>,
) {
    let held = settings.start == RaceStart::Lights;
    if race.held != held {
        race.held = held;
    }
    if !held {
        return;
    }
    // Restarted (see `crate::bike`): back on the grid, the lights again.
    if !race.started && lights.out() {
        *lights = StartLights::new(chance(&real));
    }
    if let Some(beep) = lights.advance(time.delta_secs()) {
        beeps.write(beep);
    }
    if lights.out() && !race.started {
        race.started = true;
        info!("Empieza la carrera: semáforo");
    }
}

/// The lights on the screen.
#[derive(Component)]
struct LightsPanel;

/// One of the lights, from the left.
#[derive(Component)]
struct Lamp(u32);

fn spawn_lights(mut commands: Commands, game_mode: Res<GameMode>) {
    let two = *game_mode == GameMode::SplitScreen;
    let size = if two { 34.0 } else { 46.0 };
    commands
        .spawn((
            DespawnOnExit(GameState::Race),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                // In split screen, across the line between the halves.
                top: if two {
                    Val::Percent(50.0)
                } else {
                    Val::Percent(14.0)
                },
                margin: UiRect::top(px(if two { -size } else { 0.0 })),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            LightsPanel,
            GlobalZIndex(5),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    column_gap: px(size * 0.35),
                    padding: UiRect::axes(px(size * 0.45), px(size * 0.3)),
                    border_radius: BorderRadius::all(px(size * 0.4)),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.88)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            ))
            .with_children(|gantry| {
                for index in 0..LIGHTS {
                    gantry.spawn((
                        Lamp(index),
                        Node {
                            width: px(size),
                            height: px(size),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(DARK),
                    ));
                }
            });
        });
}

/// The lights as they are: on in red one by one, then green a moment once
/// out; with no lights to wait for, hidden.
fn show_lights(
    settings: Res<Settings>,
    race: Res<Race>,
    lights: Res<StartLights>,
    mut panel: Query<&mut Visibility, With<LightsPanel>>,
    mut lamps: Query<(&Lamp, &mut BackgroundColor)>,
) {
    let green = lights.since_out.is_some_and(|since| since < GREEN_FOR);
    let shown = settings.start == RaceStart::Lights && (!race.started || green);
    for mut visibility in &mut panel {
        let wanted = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    let lit = lights.lit();
    for (lamp, mut color) in &mut lamps {
        let wanted = if green {
            GREEN
        } else if lamp.0 < lit {
            RED
        } else {
            DARK
        };
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_red_lights_come_on_a_second_apart_and_go_out() {
        let mut lights = StartLights::new(0.0);
        let mut beeps = Vec::new();
        let mut lit = Vec::new();
        let dt = 0.01;
        for step in 0..800 {
            if let Some(beep) = lights.advance(dt) {
                beeps.push((step as f32 * dt, beep));
            }
            lit.push(lights.lit());
        }
        let on: Vec<f32> = beeps
            .iter()
            .filter(|(_, beep)| *beep == StartBeep::Light)
            .map(|(at, _)| *at)
            .collect();
        assert_eq!(on.len(), LIGHTS as usize);
        for pair in on.windows(2) {
            assert!((pair[1] - pair[0] - BETWEEN).abs() < 0.02, "{on:?}");
        }
        // They go out once, a moment after the fifth.
        let go: Vec<f32> = beeps
            .iter()
            .filter(|(_, beep)| *beep == StartBeep::Go)
            .map(|(at, _)| *at)
            .collect();
        assert_eq!(go.len(), 1);
        assert!(go[0] - on[4] >= HOLD.start - 0.02);
        assert!(lights.out());
        assert_eq!(lights.lit(), 0);
        assert_eq!(lit.iter().max(), Some(&LIGHTS));
    }

    #[test]
    fn the_wait_after_the_fifth_light_varies() {
        let soon = StartLights::new(0.0).out_at;
        let late = StartLights::new(1.0).out_at;
        assert!(late - soon > 0.9);
        assert!(soon > FIRST_LIGHT + 4.0 * BETWEEN);
    }
}
