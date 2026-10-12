//! Riding round a vertical loop ([`Loop`]), upside down at the top.
//!
//! On the loop the bike runs on rails: it follows the running surface,
//! steering does nothing and the walls and the other bikes are left out. Its
//! speed comes from the same engine, brake and resistances as on the road
//! ([`step`]), plus the part of gravity along the surface, which slows it on
//! the way up and speeds it up on the way down. The surface holds the bike as
//! long as it presses on it: past the side, too slow to be pressed against the
//! surface (the speed squared over the radius is less than the part of gravity
//! pulling it away), it falls off. Too slow to get past the side, it rolls
//! back down and out of the mouth, backwards.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{BIKE_HALF_WIDTH, BikeInput, BikeState, BikeTuning, GRAVITY, Ground, step};
use crate::geometry::wrap_angle;
use crate::track::{LOOP_HALF_WIDTH, Loop, Surface, Track};

/// A bike riding round a loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ride {
    /// Which loop of the track (see [`Track::loops`]).
    pub index: usize,
    /// How far round it is (rad): 0 at the mouth, a full turn at the way out.
    pub angle: f32,
    /// How far to the left of the middle of the running surface (m).
    pub lateral: f32,
}

/// How a step on a loop ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// Still on the loop.
    Riding,
    /// Back on the road: out of the way out, or rolled back out of the mouth.
    Off,
    /// Fell off the running surface, with this velocity (x, up, z; m/s).
    Fell(Vec3),
}

/// How far from the middle of the running surface a bike can ride (m).
const RIDE_HALF_WIDTH: f32 = LOOP_HALF_WIDTH - BIKE_HALF_WIDTH;

/// If a bike that has just taken a step on the road has gone in at the mouth
/// of a loop, puts it on the loop.
pub fn enter(state: &mut BikeState, track: &Track) {
    if state.ride.is_some() || state.speed <= 0.0 {
        return;
    }
    for (index, looped) in track.loops().iter().enumerate() {
        let facing = crate::geometry::forward(state.heading).dot(looped.ahead) > 0.5;
        let Some((into, lateral)) = looped.entered(state.position, state.elevation) else {
            continue;
        };
        if !facing {
            continue;
        }
        let ride = Ride {
            index,
            angle: into / looped.arc(),
            lateral: lateral.clamp(-RIDE_HALF_WIDTH, RIDE_HALF_WIDTH),
        };
        state.steer = 0.0;
        state.steer_rate = 0.0;
        state.steer_direct = 0.0;
        state.grass = 0.0;
        state.heading += wrap_angle(looped.heading - state.heading);
        place(state, looped, ride);
        return;
    }
}

/// One step of `dt` seconds of a bike on a loop.
pub fn ride(
    state: &BikeState,
    input: &BikeInput,
    looped: &Loop,
    tuning: &BikeTuning,
    dt: f32,
) -> (BikeState, Outcome) {
    let Some(mut ride) = state.ride else {
        return (*state, Outcome::Off);
    };
    // The engine, the brake and the resistances as on a flat road; the
    // running surface does the steering, and presses the suspension down.
    let input = BikeInput {
        steer: 0.0,
        steer_direct: 0.0,
        wheelie: 0.0,
        ..*input
    };
    let ground = Ground {
        surface: Surface::Asphalt,
        grade: 0.0,
        crest: -1.0 / looped.radius,
    };
    let mut next = step(state, &input, &ground, tuning, dt);
    next.speed -= GRAVITY * ride.angle.sin() * dt;
    ride.angle += 0.5 * (state.speed + next.speed) * dt / looped.arc();
    next.ride = Some(ride);
    // How hard the surface presses on the bike, per unit of mass (m/s²).
    let pressed = next.speed * next.speed / looped.radius + GRAVITY * ride.angle.cos();
    if pressed < 0.0 {
        let velocity = looped.tangent(ride.angle) * next.speed;
        place(&mut next, looped, ride);
        next.ride = None;
        return (next, Outcome::Fell(velocity));
    }
    if ride.angle >= TAU {
        // Out of the way out, as far along the road as it went past it.
        let past = (ride.angle - TAU) * looped.arc();
        leave(
            &mut next,
            looped,
            looped.way_out() + looped.ahead * past,
            ride,
        );
        return (next, Outcome::Off);
    }
    if ride.angle < 0.0 {
        // Rolled back out of the mouth.
        let past = ride.angle * looped.arc();
        leave(&mut next, looped, looped.mouth + looped.ahead * past, ride);
        return (next, Outcome::Off);
    }
    place(&mut next, looped, ride);
    (next, Outcome::Riding)
}

/// Puts a bike on the running surface where `ride` says, facing round it.
fn place(state: &mut BikeState, looped: &Loop, ride: Ride) {
    let point = looped.surface_point(ride.angle, ride.lateral);
    state.position = Vec2::new(point.x, point.z);
    state.elevation = point.y;
    state.pitch = ride.angle;
    state.climb = state.speed * ride.angle.sin();
    state.grounded = true;
    state.ride = Some(ride);
}

/// Takes a bike off the loop onto the road at `point` (the middle of the
/// road there), as far to the side as it rode.
fn leave(state: &mut BikeState, looped: &Loop, point: Vec2, ride: Ride) {
    state.position = point + looped.left() * ride.lateral;
    state.elevation = looped.height;
    state.pitch = wrap_angle(ride.angle);
    state.climb = 0.0;
    state.grounded = true;
    state.ride = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::WALL_DISTANCE;

    const DT: f32 = 1.0 / 120.0;

    fn looped() -> Loop {
        Loop {
            mouth: Vec2::ZERO,
            ahead: Vec2::NEG_Y,
            heading: 0.0,
            height: 2.0,
            radius: 16.0,
            shift: -16.0,
            approach: 1,
            exit: 2,
            fork: 0,
            join: 0,
        }
    }

    /// A bike going in at the mouth at `speed`, with `throttle`; how it
    /// came off the loop and when (s).
    fn go_round(speed: f32, throttle: f32) -> (BikeState, Outcome, f32) {
        let state = BikeState {
            speed,
            ride: Some(Ride {
                index: 0,
                angle: 0.0,
                lateral: 1.0,
            }),
            ..BikeState::parked_at((Vec2::ZERO, 0.0))
        };
        let input = BikeInput {
            throttle,
            ..default()
        };
        let (state, outcome, seconds) = keep_riding(state, input, 20.0);
        assert_ne!(outcome, Outcome::Riding, "still on the loop");
        (state, outcome, seconds)
    }

    /// A bike on the loop riding with `input` for up to `seconds`, or until
    /// it comes off.
    fn keep_riding(
        mut state: BikeState,
        input: BikeInput,
        seconds: f32,
    ) -> (BikeState, Outcome, f32) {
        let (looped, tuning) = (looped(), BikeTuning::default());
        for k in 0..(seconds / DT) as usize {
            let outcome;
            (state, outcome) = ride(&state, &input, &looped, &tuning, DT);
            if outcome != Outcome::Riding {
                return (state, outcome, k as f32 * DT);
            }
        }
        (state, Outcome::Riding, seconds)
    }

    #[test]
    fn the_running_surface_goes_round_and_comes_out_beside_the_mouth() {
        let looped = looped();
        let start = looped.surface_point(0.0, 0.0);
        let top = looped.surface_point(TAU / 2.0, 0.0);
        let end = looped.surface_point(TAU, 0.0);
        assert!((top.y - start.y - 2.0 * looped.radius).abs() < 1e-3);
        assert!((Vec2::new(end.x, end.z) - looped.way_out()).length() < 1e-3);
        // Side by side at the bottom, clear of each other.
        assert!(looped.shift.abs() > 2.0 * LOOP_HALF_WIDTH);
        assert!(looped.shift.abs() < 2.0 * WALL_DISTANCE);
        // At the top, upside down: up is down.
        assert!((looped.up(TAU / 2.0) - Vec3::NEG_Y).length() < 1e-4);
        assert!((looped.tangent(TAU / 4.0) - Vec3::Y).length() < 1e-4);
    }

    #[test]
    fn fast_enough_the_bike_goes_round_and_out_of_the_way_out() {
        let looped = looped();
        let (state, outcome, seconds) = go_round(40.0, 1.0);
        assert_eq!(outcome, Outcome::Off);
        assert!(state.ride.is_none());
        let out = state.position - looped.way_out();
        assert!(out.dot(looped.ahead) >= 0.0 && out.dot(looped.ahead) < 1.0);
        assert!(
            (out.dot(looped.left()) - 1.0).abs() < 1e-3,
            "keeps its side"
        );
        assert!((state.elevation - looped.height).abs() < 1e-3);
        assert!(state.speed > 30.0, "{}", state.speed);
        assert!(seconds < 3.0, "{seconds} s");
    }

    #[test]
    fn too_slow_to_reach_the_side_the_bike_comes_back_down_and_out() {
        let looped = looped();
        let state = BikeState {
            speed: 12.0,
            ride: Some(Ride {
                index: 0,
                angle: 0.0,
                lateral: 0.0,
            }),
            ..BikeState::parked_at((Vec2::ZERO, 0.0))
        };
        // It stops short of the side and comes back down, where the slope
        // is gentle enough for the engine braking to hold it.
        let (state, outcome, _) = keep_riding(state, BikeInput::default(), 6.0);
        if outcome == Outcome::Riding {
            let ride = state.ride.unwrap();
            assert!(ride.angle < 0.6 && state.speed.abs() < 0.5, "{ride:?}");
        }
        // Reversing, it rolls out of the mouth backwards.
        let reverse = BikeInput {
            brake: 1.0,
            ..default()
        };
        let (state, outcome, _) = keep_riding(state, reverse, 10.0);
        assert_eq!(outcome, Outcome::Off);
        assert!(state.speed < 0.0, "backwards: {}", state.speed);
        let out = state.position - looped.mouth;
        assert!(out.dot(looped.ahead) <= 0.0);
        assert!((state.elevation - looped.height).abs() < 1e-3);
    }

    #[test]
    fn too_slow_to_go_over_the_top_the_bike_falls_off() {
        let (_, outcome, _) = go_round(28.0, 0.0);
        let Outcome::Fell(velocity) = outcome else {
            panic!("{outcome:?}");
        };
        // Past the side, still going up and round.
        assert!(velocity.length() > 1.0, "{velocity}");
    }

    #[test]
    fn a_bike_at_the_mouth_goes_onto_the_loop() {
        let mut track = Track::default();
        let looped = looped();
        track.add_loop_for_tests(looped);
        let mut state = BikeState {
            speed: 30.0,
            elevation: looped.height,
            ..BikeState::parked_at((looped.mouth + looped.ahead * 0.5 + looped.left() * 2.0, 0.0))
        };
        enter(&mut state, &track);
        let ride = state.ride.expect("on the loop");
        assert!((ride.lateral - 2.0).abs() < 1e-3);
        assert!(ride.angle > 0.0 && ride.angle < 0.05);
        // Backwards, or not yet at the mouth, it does not.
        for (speed, along) in [(-3.0, 0.5), (30.0, -0.5)] {
            let mut state = BikeState {
                speed,
                elevation: looped.height,
                ..BikeState::parked_at((looped.mouth + looped.ahead * along, 0.0))
            };
            enter(&mut state, &track);
            assert!(state.ride.is_none());
        }
    }
}
