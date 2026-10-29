//! The keyboard's curves: with Options > Control > Advanced's «Keyboard by
//! curves» on, the keyboard and the d-pad lean the bike over and the lean
//! turns it, each step shaped by a Bézier curve (see [`crate::bezier`]):
//!
//! - **A, leaning in**: how far over the rider is for how long the key has
//!   been held, and how long that takes.
//! - **B, standing up**: how far back up for how long it has been let go.
//! - **C, unwinding**: turning the other way, how the lean comes back to
//!   upright first (at once, as always, or over a time).
//! - **D, the lean at speed**: how far over the bike goes at most for how
//!   fast it goes, down to a share at top speed.
//! - **E, the turn for the lean**: how much the bike turns for how far
//!   over it is.
//!
//! The lean shown is the one the curves give (see [`super::step`]), so
//! what is seen is what turns the bike. The classic preset is the keyboard
//! as it is without the curves.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::bezier::Bezier;
use crate::locale::Language;

/// A curve run over a time: how far through it is how long since it began
/// out of `seconds`.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct TimedCurve {
    pub curve: Bezier,
    pub seconds: f32,
}

/// The keyboard's curves, as Options > Control > Advanced sets them.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardCurves {
    /// A: from upright to all the way over, holding a key.
    pub lean_in: TimedCurve,
    /// B: from all the way over to upright, with no key held.
    pub stand_up: TimedCurve,
    /// C: from all the way over to upright, holding the other way's key; at
    /// once with no time (then it leans in from upright as A says).
    pub unwind: TimedCurve,
    /// D: how much of the bike's lean is kept, from all of it standing still
    /// down to `top_speed_lean` at top speed, for the share of top speed.
    pub speed: Bezier,
    pub top_speed_lean: f32,
    /// E: the share of the bike's turn for the share of its lean.
    pub turn: Bezier,
}

impl Default for KeyboardCurves {
    fn default() -> Self {
        CurvePreset::Classic.curves()
    }
}

/// Bounds of what the curves' options can be set to.
pub const SECONDS: (f32, f32) = (0.05, 1.5);
pub const TOP_SPEED_LEAN: (f32, f32) = (0.3, 1.0);

impl KeyboardCurves {
    /// The preset these curves are, if any.
    pub fn preset(&self) -> Option<CurvePreset> {
        CURVE_PRESETS
            .into_iter()
            .find(|preset| preset.curves().same_as(self))
    }

    /// Whether `other` is these curves, but for the rounding of what is
    /// saved.
    fn same_as(&self, other: &Self) -> bool {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        let curve = |a: Bezier, b: Bezier| {
            near(a.x1, b.x1) && near(a.y1, b.y1) && near(a.x2, b.x2) && near(a.y2, b.y2)
        };
        let timed =
            |a: TimedCurve, b: TimedCurve| curve(a.curve, b.curve) && near(a.seconds, b.seconds);
        timed(self.lean_in, other.lean_in)
            && timed(self.stand_up, other.stand_up)
            && timed(self.unwind, other.unwind)
            && curve(self.speed, other.speed)
            && near(self.top_speed_lean, other.top_speed_lean)
            && curve(self.turn, other.turn)
    }

    /// The Bézier of curve `kind`.
    pub fn bezier(&self, kind: CurveKind) -> Bezier {
        match kind {
            CurveKind::LeanIn => self.lean_in.curve,
            CurveKind::StandUp => self.stand_up.curve,
            CurveKind::Unwind => self.unwind.curve,
            CurveKind::Speed => self.speed,
            CurveKind::Turn => self.turn,
        }
    }

    pub fn bezier_mut(&mut self, kind: CurveKind) -> &mut Bezier {
        match kind {
            CurveKind::LeanIn => &mut self.lean_in.curve,
            CurveKind::StandUp => &mut self.stand_up.curve,
            CurveKind::Unwind => &mut self.unwind.curve,
            CurveKind::Speed => &mut self.speed,
            CurveKind::Turn => &mut self.turn,
        }
    }

    /// The number curve `kind` is scaled by, if it has one: the seconds of
    /// A, B and C, the lean kept at top speed of D.
    pub fn scale(&self, kind: CurveKind) -> Option<f32> {
        match kind {
            CurveKind::LeanIn => Some(self.lean_in.seconds),
            CurveKind::StandUp => Some(self.stand_up.seconds),
            CurveKind::Unwind => Some(self.unwind.seconds),
            CurveKind::Speed => Some(self.top_speed_lean),
            CurveKind::Turn => None,
        }
    }

    /// Steps the number curve `kind` is scaled by (`step` steps of 0.05),
    /// kept within its bounds. C may come down to nought: at once.
    pub fn step_scale(&mut self, kind: CurveKind, step: i32) {
        let stepped = |value: f32, (low, high): (f32, f32)| {
            ((value + 0.05 * step as f32).clamp(low, high) * 1000.0).round() / 1000.0
        };
        match kind {
            CurveKind::LeanIn => self.lean_in.seconds = stepped(self.lean_in.seconds, SECONDS),
            CurveKind::StandUp => self.stand_up.seconds = stepped(self.stand_up.seconds, SECONDS),
            CurveKind::Unwind => {
                // From the least time down to nought, and back up.
                let next = stepped(self.unwind.seconds, (0.0, SECONDS.1));
                self.unwind.seconds = if next > 0.0 && next < SECONDS.0 {
                    if step < 0 { 0.0 } else { SECONDS.0 }
                } else {
                    next
                };
            }
            CurveKind::Speed => {
                self.top_speed_lean = stepped(self.top_speed_lean, TOP_SPEED_LEAN);
            }
            CurveKind::Turn => {}
        }
    }

    /// The most the bike leans over (a share of all it can) going at
    /// `speed_share` of its top speed: D.
    pub fn lean_limit(&self, speed_share: f32) -> f32 {
        1.0 - (1.0 - self.top_speed_lean) * self.speed.ease(speed_share.abs())
    }

    /// The share of its turn the bike makes leaning `lean` (a share of all
    /// it can, either way): E, on the side it leans to.
    pub fn turn_for(&self, lean: f32) -> f32 {
        self.turn.ease(lean.abs()).copysign(lean) * f32::from(lean != 0.0)
    }
}

/// The ready-made sets of curves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CurvePreset {
    /// The keyboard as it is without the curves (its options as they are at
    /// first): leaning in faster and faster, standing up slowly at first,
    /// upright at once turning the other way, a little less at speed.
    Classic,
    /// Gentler in and out, and unwinding over a moment.
    Smooth,
    /// Quick to lean in from the first instant, quick back up, a little
    /// more turn for a little lean.
    Arcade,
    /// Straight lines, short times: as the key says, nearly at once.
    Direct,
}

pub const CURVE_PRESETS: [CurvePreset; 4] = [
    CurvePreset::Classic,
    CurvePreset::Smooth,
    CurvePreset::Arcade,
    CurvePreset::Direct,
];

/// The top speed of the keyboard's classic limit as a share of the bike's
/// top speed (70 of 80 m/s), and the lean it keeps there: the turn limit
/// as it is at first, two thirds of the way from all of it to 0.8.
const CLASSIC_TOP_SPEED_LEAN: f32 = 1.0 - (1.0 - 0.8) * 2.0 / 3.0;

impl CurvePreset {
    pub fn curves(self) -> KeyboardCurves {
        let timed = |x1, y1, x2, y2, seconds| TimedCurve {
            curve: Bezier::new(x1, y1, x2, y2),
            seconds,
        };
        let third = 1.0 / 3.0;
        match self {
            // As `DigitalSteer` at medium: held, 1.875 t + 1.875 t² up to
            // all of it (0.385 s); let go, 0.25 t + t² back to upright
            // (0.883 s). Both polynomials of the time, so their control
            // points stand on the thirds across. Its limit with speed, the
            // smoothstep from 15 to 70 m/s, comes close as an easing with
            // both control points at 0.56 across.
            CurvePreset::Classic => KeyboardCurves {
                lean_in: timed(third, 0.2406, 2.0 * third, 0.5739, 0.385),
                stand_up: timed(third, 0.0736, 2.0 * third, 0.4069, 0.883),
                unwind: timed(third, third, 2.0 * third, 2.0 * third, 0.0),
                speed: Bezier::new(0.56, 0.0, 0.56, 1.0),
                top_speed_lean: CLASSIC_TOP_SPEED_LEAN,
                turn: Bezier::LINEAR,
            },
            CurvePreset::Smooth => KeyboardCurves {
                lean_in: timed(0.3, 0.1, 0.6, 0.9, 0.5),
                stand_up: timed(0.25, 0.1, 0.5, 1.0, 0.6),
                unwind: timed(0.2, 0.3, 0.6, 1.0, 0.15),
                speed: Bezier::new(0.4, 0.0, 0.6, 1.0),
                top_speed_lean: 0.8,
                turn: Bezier::LINEAR,
            },
            CurvePreset::Arcade => KeyboardCurves {
                lean_in: timed(0.1, 0.5, 0.5, 0.95, 0.25),
                stand_up: timed(0.1, 0.5, 0.4, 1.0, 0.35),
                unwind: timed(0.1, 0.5, 0.4, 1.0, 0.08),
                speed: Bezier::new(0.5, 0.0, 0.6, 1.0),
                top_speed_lean: 0.9,
                turn: Bezier::new(0.2, 0.4, 0.6, 0.85),
            },
            CurvePreset::Direct => KeyboardCurves {
                lean_in: timed(third, third, 2.0 * third, 2.0 * third, 0.15),
                stand_up: timed(third, third, 2.0 * third, 2.0 * third, 0.2),
                unwind: timed(third, third, 2.0 * third, 2.0 * third, 0.0),
                speed: Bezier::new(0.5, 0.0, 0.6, 1.0),
                top_speed_lean: 0.95,
                turn: Bezier::LINEAR,
            },
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CurvePreset::Classic, Language::Spanish) => "Clásico",
            (CurvePreset::Classic, Language::English) => "Classic",
            (CurvePreset::Smooth, Language::Spanish) => "Suave",
            (CurvePreset::Smooth, Language::English) => "Smooth",
            (CurvePreset::Arcade, _) => "Arcade",
            (CurvePreset::Direct, Language::Spanish) => "Directo",
            (CurvePreset::Direct, Language::English) => "Direct",
        }
    }
}

/// Each of the keyboard's curves.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CurveKind {
    #[default]
    LeanIn,
    StandUp,
    Unwind,
    Speed,
    Turn,
}

pub const CURVE_KINDS: [CurveKind; 5] = [
    CurveKind::LeanIn,
    CurveKind::StandUp,
    CurveKind::Unwind,
    CurveKind::Speed,
    CurveKind::Turn,
];

impl CurveKind {
    /// Its letter, as the panel of the curves and the options call it.
    pub fn letter(self) -> char {
        match self {
            CurveKind::LeanIn => 'A',
            CurveKind::StandUp => 'B',
            CurveKind::Unwind => 'C',
            CurveKind::Speed => 'D',
            CurveKind::Turn => 'E',
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CurveKind::LeanIn, Language::Spanish) => "Tumbada al pulsar",
            (CurveKind::LeanIn, Language::English) => "Leaning in",
            (CurveKind::StandUp, Language::Spanish) => "Levantada al soltar",
            (CurveKind::StandUp, Language::English) => "Standing up",
            (CurveKind::Unwind, Language::Spanish) => "Cambio de lado",
            (CurveKind::Unwind, Language::English) => "Other way",
            (CurveKind::Speed, Language::Spanish) => "Tumbada según velocidad",
            (CurveKind::Speed, Language::English) => "Lean for speed",
            (CurveKind::Turn, Language::Spanish) => "Giro según tumbada",
            (CurveKind::Turn, Language::English) => "Turn for lean",
        }
    }

    /// Its name in few words, for the panel of the curves.
    pub fn short_name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CurveKind::LeanIn, Language::Spanish) => "Tumbada",
            (CurveKind::LeanIn, Language::English) => "Lean in",
            (CurveKind::StandUp, Language::Spanish) => "Levantada",
            (CurveKind::StandUp, Language::English) => "Stand up",
            (CurveKind::Unwind, Language::Spanish) => "Cambio",
            (CurveKind::Unwind, Language::English) => "Other way",
            (CurveKind::Speed, Language::Spanish) => "Velocidad",
            (CurveKind::Speed, Language::English) => "Speed",
            (CurveKind::Turn, Language::Spanish) => "Giro",
            (CurveKind::Turn, Language::English) => "Turn",
        }
    }

    /// What its number is, as the options name it.
    pub fn scale_name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CurveKind::Speed, Language::Spanish) => "Tumbada a toda velocidad",
            (CurveKind::Speed, Language::English) => "Lean at top speed",
            (_, Language::Spanish) => "Duración",
            (_, Language::English) => "Duration",
        }
    }

    /// Its number as it is shown: seconds, at once (C at nought), or the
    /// share of the lean.
    pub fn scale_text(self, value: f32, lang: Language) -> String {
        match self {
            CurveKind::Speed => format!("{:.0} %", value * 100.0),
            CurveKind::Unwind if value <= 0.0 => match lang {
                Language::Spanish => "al momento".to_string(),
                Language::English => "at once".to_string(),
            },
            _ => format!("{value:.2} s"),
        }
    }

    /// The other one, `step` along.
    pub fn next(self, step: i32) -> Self {
        crate::settings::cycle(&CURVE_KINDS, self, step)
    }
}

/// Which of the keyboard's curves the lean is following.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Phase {
    /// Upright with no key held.
    #[default]
    Upright,
    /// Leaning in or held over (A, up to D's limit).
    LeaningIn,
    /// Standing back up with no key held (B).
    StandingUp,
    /// Coming back up holding the other way (C).
    Unwinding,
}

/// The lean the keyboard or the d-pad asks for with the curves: how far
/// over (a share of all the bike can, positive to the left), which curve it
/// is following and how far through it (0 to 1), and the way held then.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct CurveLean {
    pub lean: f32,
    pub phase: Phase,
    pub progress: f32,
    held: f32,
}

impl CurveLean {
    /// Goes on `dt` seconds with `pressed` held (-1, 0 or 1) at
    /// `speed_share` of the top speed. Each curve is followed from wherever
    /// the lean is (the first point of it at that height), so going from
    /// one to another never jumps. Returns the lean.
    pub fn update(
        &mut self,
        pressed: f32,
        curves: &KeyboardCurves,
        speed_share: f32,
        dt: f32,
    ) -> f32 {
        let pressed = pressed.clamp(-1.0, 1.0).round();
        let limit = curves.lean_limit(speed_share);
        let mut dt = dt.max(0.0);
        // At most twice round: unwinding may end within the step and lean
        // in with what is left of it.
        for _ in 0..2 {
            let over = self.lean.abs();
            let phase = if pressed == 0.0 {
                if over > 0.0 {
                    Phase::StandingUp
                } else {
                    Phase::Upright
                }
            } else if self.lean * pressed < 0.0 {
                Phase::Unwinding
            } else {
                Phase::LeaningIn
            };
            let curve = match phase {
                Phase::Upright => {
                    *self = Self::default();
                    return 0.0;
                }
                Phase::LeaningIn => curves.lean_in,
                Phase::StandingUp => curves.stand_up,
                Phase::Unwinding => curves.unwind,
            };
            // Where on the curve the lean is: leaning in, how far over;
            // coming back up, how much of the way back.
            let height = |lean: f32| match phase {
                Phase::LeaningIn => lean,
                _ => 1.0 - lean,
            };
            if phase != self.phase || pressed != self.held {
                (self.phase, self.held) = (phase, pressed);
                self.progress = curve.curve.first_reaching(height(over));
            }
            if phase == Phase::Unwinding && curve.seconds <= 0.0 {
                // Upright at once, and on into the turn.
                *self = Self::default();
                continue;
            }
            let duration = curve.seconds.max(1e-3);
            let to_end = (1.0 - self.progress) * duration;
            let spent = dt.min(to_end);
            self.progress = (self.progress + spent / duration).min(1.0);
            let shape = curve.curve.ease(self.progress);
            match phase {
                Phase::LeaningIn => {
                    let lean = shape.min(limit);
                    if shape > limit {
                        // Held where the limit is, to go on from there if
                        // the limit rises.
                        self.progress = curve.curve.first_reaching(limit);
                    }
                    self.lean = lean * pressed;
                    return self.lean;
                }
                Phase::StandingUp => {
                    self.lean = ((1.0 - shape).min(over)).copysign(self.lean);
                    if self.progress >= 1.0 {
                        *self = Self::default();
                    }
                    return self.lean;
                }
                Phase::Unwinding => {
                    self.lean = ((1.0 - shape).min(over)).copysign(self.lean);
                    if self.progress < 1.0 {
                        return self.lean;
                    }
                    // Upright: the rest of the step leans in.
                    *self = Self::default();
                    dt -= spent;
                }
                Phase::Upright => unreachable!(),
            }
        }
        self.lean
    }
}

/// How the first player's lean is going on the keyboard's curves, for the
/// panel of the curves (see [`crate::curves_hud`]): which curve and how far
/// through it, how far over, how fast the bike goes (a share of its top
/// speed), D's limit there and the turn E gives.
#[derive(Resource, Clone, Copy, Default, Debug, PartialEq)]
pub struct CurveProbe {
    /// Whether the curves are steering a player's bike now.
    pub live: bool,
    pub lean: CurveLean,
    pub speed_share: f32,
    pub limit: f32,
    pub turn: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    /// The lean, step by step, holding `pressed(t)` at `speed_share`.
    fn run(
        curves: &KeyboardCurves,
        speed_share: f32,
        seconds: f32,
        pressed: impl Fn(f32) -> f32,
    ) -> Vec<f32> {
        let mut lean = CurveLean::default();
        (0..(seconds / DT).round() as usize)
            .map(|frame| lean.update(pressed(frame as f32 * DT), curves, speed_share, DT))
            .collect()
    }

    fn at(values: &[f32], seconds: f32) -> f32 {
        values[(seconds / DT).round() as usize - 1]
    }

    #[test]
    fn every_preset_is_found_by_its_curves_and_the_classic_one_is_the_default() {
        for preset in CURVE_PRESETS {
            assert_eq!(preset.curves().preset(), Some(preset));
        }
        assert_eq!(
            KeyboardCurves::default().preset(),
            Some(CurvePreset::Classic)
        );
        let mut custom = KeyboardCurves::default();
        custom.turn.y1 = 0.5;
        assert_eq!(custom.preset(), None);
    }

    #[test]
    fn holding_a_key_leans_over_along_a_in_its_time() {
        let curves = CurvePreset::Classic.curves();
        let values = run(&curves, 0.0, 1.0, |_| 1.0);
        // Moving from the first step, all the way over by A's time.
        assert!(values[0] > 0.0);
        assert!(values.windows(2).all(|w| w[1] >= w[0]));
        assert!((at(&values, 0.39) - 1.0).abs() < 1e-3);
        // Faster and faster, as the classic keyboard.
        assert!(at(&values, 0.2) - at(&values, 0.1) > at(&values, 0.1));
        // The other way, the same to the right.
        let right = run(&curves, 0.0, 1.0, |_| -1.0);
        assert_eq!(right[20], -values[20]);
    }

    #[test]
    fn the_classic_preset_leans_as_the_classic_keyboard_steers() {
        // DigitalSteer at medium: 1.875 t + 1.875 t² held; from all the way
        // over, 0.25 t + t² back once let go.
        let curves = CurvePreset::Classic.curves();
        let held = run(&curves, 0.0, 0.3, |_| 1.0);
        for seconds in [0.05, 0.1, 0.2, 0.3] {
            let classic = 1.875 * seconds + 1.875 * seconds * seconds;
            assert!(
                (at(&held, seconds) - classic).abs() < 0.02,
                "{seconds}: {}",
                at(&held, seconds)
            );
        }
        let released = run(&curves, 0.0, 1.5, |t| if t < 0.5 { 1.0 } else { 0.0 });
        for after in [0.2, 0.4, 0.6] {
            let classic = 1.0 - (0.25 * after + after * after);
            assert!(
                (at(&released, 0.5 + after) - classic).abs() < 0.02,
                "{after}: {}",
                at(&released, 0.5 + after)
            );
        }
        assert_eq!(at(&released, 1.5), 0.0);
    }

    #[test]
    fn the_lean_is_limited_at_speed_as_d_says() {
        let curves = CurvePreset::Classic.curves();
        assert!((curves.lean_limit(0.0) - 1.0).abs() < 1e-6);
        let top = curves.lean_limit(1.0);
        assert!((top - curves.top_speed_lean).abs() < 1e-5);
        assert!(curves.lean_limit(0.5) < 1.0 && curves.lean_limit(0.5) > top);
        let flat_out = run(&curves, 1.0, 1.0, |_| 1.0);
        assert!((at(&flat_out, 1.0) - top).abs() < 1e-5);
        // As fast to answer, whatever the speed.
        let parked = run(&curves, 0.0, 1.0, |_| 1.0);
        assert_eq!(at(&flat_out, 0.1), at(&parked, 0.1));
    }

    #[test]
    fn the_limit_rising_lets_the_lean_go_on_without_a_jump() {
        let curves = CurvePreset::Classic.curves();
        let mut lean = CurveLean::default();
        for _ in 0..120 {
            lean.update(1.0, &curves, 1.0, DT);
        }
        let held = lean.lean;
        // Slowing down: the limit opens and the lean goes on from there.
        let next = lean.update(1.0, &curves, 0.0, DT);
        assert!(next > held && next - held < 0.05, "{held} -> {next}");
    }

    #[test]
    fn letting_go_halfway_comes_back_from_where_it_was() {
        let curves = CurvePreset::Classic.curves();
        let values = run(&curves, 0.0, 1.5, |t| if t < 0.15 { 1.0 } else { 0.0 });
        let let_go = at(&values, 0.15);
        let after = at(&values, 0.15 + DT);
        assert!(
            after <= let_go && let_go - after < 0.02,
            "{let_go} -> {after}"
        );
        assert!(values.iter().skip(18).all(|&v| v <= let_go));
        assert_eq!(*values.last().unwrap(), 0.0);
    }

    #[test]
    fn turning_the_other_way_comes_upright_as_c_says() {
        let flip = |t: f32| if t < 0.5 { 1.0 } else { -1.0 };
        // Classic: upright at once and over the other way as A says.
        let curves = CurvePreset::Classic.curves();
        let values = run(&curves, 0.0, 1.0, flip);
        let after = at(&values, 0.5 + DT);
        assert!(after < 0.0 && after > -0.05, "{after}");
        // Over a time: still leaning left a moment later, right after it.
        let mut slow = curves;
        slow.unwind.seconds = 0.2;
        let values = run(&slow, 0.0, 1.5, flip);
        assert!(at(&values, 0.6) > 0.0);
        assert!(at(&values, 0.75) < 0.0);
        assert!(values.windows(2).skip(60).all(|w| w[1] <= w[0]));
        assert!((at(&values, 1.5) + 1.0).abs() < 1e-3);
    }

    #[test]
    fn the_turn_follows_the_lean_through_e() {
        let mut curves = CurvePreset::Classic.curves();
        assert_eq!(curves.turn_for(0.0), 0.0);
        assert!((curves.turn_for(0.5) - 0.5).abs() < 1e-5);
        assert!((curves.turn_for(-0.5) + 0.5).abs() < 1e-5);
        curves.turn = Bezier::new(0.1, 0.6, 0.5, 1.0);
        assert!(curves.turn_for(0.2) > 0.4);
    }

    #[test]
    fn the_same_ride_at_any_frame_rate() {
        let curves = CurvePreset::Smooth.curves();
        let ride = |dt: f32| {
            let mut lean = CurveLean::default();
            for frame in 0..(1.1 / dt).round() as usize {
                // In the middle of the step, clear of where the keys change.
                let t = (frame as f32 + 0.5) * dt;
                let pressed = if t < 0.4 {
                    1.0
                } else if t < 0.7 {
                    -1.0
                } else {
                    0.0
                };
                lean.update(pressed, &curves, 0.3, dt);
            }
            lean.lean
        };
        let (fine, coarse) = (ride(1.0 / 240.0), ride(1.0 / 40.0));
        assert!((fine - coarse).abs() < 0.05, "{fine} vs {coarse}");
    }

    #[test]
    fn the_scales_step_within_their_bounds_and_c_reaches_at_once() {
        let mut curves = KeyboardCurves::default();
        for _ in 0..100 {
            curves.step_scale(CurveKind::LeanIn, 1);
        }
        assert_eq!(curves.lean_in.seconds, SECONDS.1);
        for _ in 0..100 {
            curves.step_scale(CurveKind::Speed, -1);
        }
        assert_eq!(curves.top_speed_lean, TOP_SPEED_LEAN.0);
        // C: from nought up to the least time, and back down to nought.
        assert_eq!(curves.unwind.seconds, 0.0);
        curves.step_scale(CurveKind::Unwind, 1);
        assert_eq!(curves.unwind.seconds, SECONDS.0);
        curves.step_scale(CurveKind::Unwind, -1);
        assert_eq!(curves.unwind.seconds, 0.0);
        curves.step_scale(CurveKind::Unwind, -1);
        assert_eq!(curves.unwind.seconds, 0.0);
    }
}
