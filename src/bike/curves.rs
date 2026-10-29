//! The keyboard's curves: with Options > Control > Advanced's keyboard
//! behaviour on Bézier, the keyboard and the d-pad lean the bike over and the lean
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
//! Each curve's shape is chosen (see [`CurveShape`]): the classic one (the
//! keyboard as it is without the curves, the default), CSS's easings, its
//! own four numbers, or the one the game learns from how the player rides.
//! The lean shown is the one the curves give (see [`super::step`]), so
//! what is seen is what turns the bike.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::bezier::Bezier;
use crate::locale::Language;

/// How the keyboard and the d-pad steer, as Options > Control > Advanced
/// says: as always (Options > Control > Keyboard behaviour's rates and
/// limit), or by the curves (that page then sets them).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum KeyboardMode {
    #[default]
    Classic,
    Bezier,
}

pub const KEYBOARD_MODES: [KeyboardMode; 2] = [KeyboardMode::Classic, KeyboardMode::Bezier];

impl KeyboardMode {
    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (KeyboardMode::Classic, Language::Spanish) => "Clásico",
            (KeyboardMode::Classic, Language::English) => "Classic",
            (KeyboardMode::Bezier, _) => "Bézier",
        }
    }
}

/// A curve run over a time: how far through it is how long since it began
/// out of `seconds`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TimedCurve {
    pub curve: Bezier,
    pub seconds: f32,
}

/// The keyboard's curves the bike follows: what [`KeyboardShapes`] (what
/// Options > Control > Keyboard behaviour chooses in Bézier) comes to.
#[derive(Clone, Copy, PartialEq, Debug)]
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
        KeyboardShapes::default().curves()
    }
}

/// How much a correction moves a learnt curve's control points up or down,
/// and the least E's may come down to (y1, y2).
pub const NUDGE: f32 = 0.02;
pub const TURN_FLOOR: (f32, f32) = (0.1, 0.3);

/// Bounds of what the curves' numbers can be set to.
pub const SECONDS: (f32, f32) = (0.05, 1.5);
pub const TOP_SPEED_LEAN: (f32, f32) = (0.3, 1.0);

/// The classic keyboard's times and its lean at top speed: as
/// `DigitalSteer` at medium, held 1.875 t + 1.875 t² up to all of it
/// (0.385 s), let go 0.25 t + t² back to upright (0.883 s), upright at once
/// the other way; its limit with speed two thirds of the way from all of
/// it to 0.8.
const CLASSIC_SECONDS: [f32; 3] = [0.385, 0.883, 0.0];
const CLASSIC_TOP_SPEED_LEAN: f32 = 1.0 - (1.0 - 0.8) * 2.0 / 3.0;

impl KeyboardCurves {
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

/// The shape chosen for one of the keyboard's curves.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum CurveShape {
    /// The classic keyboard's (see [`CurveShape::fixed`]).
    #[default]
    Classic,
    /// CSS's own easings.
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// Its four numbers as set.
    Custom,
    /// As the game has learnt from how the player rides.
    Learned,
}

pub const CURVE_SHAPES: [CurveShape; 8] = [
    CurveShape::Classic,
    CurveShape::Linear,
    CurveShape::Ease,
    CurveShape::EaseIn,
    CurveShape::EaseOut,
    CurveShape::EaseInOut,
    CurveShape::Custom,
    CurveShape::Learned,
];

impl CurveShape {
    /// Its Bézier for curve `kind`, if it has one of its own (not custom
    /// nor learnt). The classic keyboard's lean in and stand up are
    /// polynomials of the time, so their control points stand on the thirds
    /// across (y1 = a/3, y2 = (a + 1)/3 for a t + (1 - a) t²); its limit
    /// with speed, the smoothstep from 15 to 70 m/s, comes close as an
    /// easing with both control points at 0.56 across; it unwinds at once
    /// (any curve) and turns as far as it leans.
    pub fn fixed(self, kind: CurveKind) -> Option<Bezier> {
        let third = 1.0 / 3.0;
        Some(match self {
            CurveShape::Classic => match kind {
                CurveKind::LeanIn => Bezier::new(third, 0.2406, 2.0 * third, 0.5739),
                CurveKind::StandUp => Bezier::new(third, 0.0736, 2.0 * third, 0.4069),
                CurveKind::Speed => Bezier::new(0.56, 0.0, 0.56, 1.0),
                CurveKind::Unwind | CurveKind::Turn => Bezier::LINEAR,
            },
            CurveShape::Linear => Bezier::LINEAR,
            CurveShape::Ease => Bezier::new(0.25, 0.1, 0.25, 1.0),
            CurveShape::EaseIn => Bezier::new(0.42, 0.0, 1.0, 1.0),
            CurveShape::EaseOut => Bezier::new(0.0, 0.0, 0.58, 1.0),
            CurveShape::EaseInOut => Bezier::new(0.42, 0.0, 0.58, 1.0),
            CurveShape::Custom | CurveShape::Learned => return None,
        })
    }

    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CurveShape::Classic, Language::Spanish) => "Clásica",
            (CurveShape::Classic, Language::English) => "Classic",
            (CurveShape::Linear, Language::Spanish) => "Lineal",
            (CurveShape::Linear, Language::English) => "Linear",
            (CurveShape::Ease, _) => "Ease",
            (CurveShape::EaseIn, _) => "Ease-in",
            (CurveShape::EaseOut, _) => "Ease-out",
            (CurveShape::EaseInOut, _) => "Ease-in-out",
            (CurveShape::Custom, Language::Spanish) => "Personalizada",
            (CurveShape::Custom, Language::English) => "Custom",
            (CurveShape::Learned, Language::Spanish) => "Aprendida",
            (CurveShape::Learned, Language::English) => "Learned",
        }
    }
}

/// What is chosen for one of the keyboard's curves: its shape, its own four
/// numbers (used when custom) and what has been learnt for it, if anything
/// (used when learnt).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CurveChoice {
    pub shape: CurveShape,
    pub custom: Bezier,
    pub learned: Option<Bezier>,
}

impl CurveChoice {
    /// Classic, its custom numbers the classic ones to start from.
    fn classic(kind: CurveKind) -> Self {
        Self {
            shape: CurveShape::Classic,
            custom: CurveShape::Classic.fixed(kind).unwrap_or(Bezier::LINEAR),
            learned: None,
        }
    }
}

impl Default for CurveChoice {
    fn default() -> Self {
        Self::classic(CurveKind::Turn)
    }
}

/// Whether the game learns the keyboard's curves from how the player rides,
/// and how far from which line it measures them (see
/// [`super::learning`]): the centre of the road, or a racing line (see
/// [`crate::track::racing_line`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum LearnLine {
    #[default]
    Off,
    Centre,
    Race,
}

pub const LEARN_LINES: [LearnLine; 3] = [LearnLine::Off, LearnLine::Centre, LearnLine::Race];

impl LearnLine {
    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (LearnLine::Off, _) => "Off",
            (LearnLine::Centre, Language::Spanish) => "Centro",
            (LearnLine::Centre, Language::English) => "Centre",
            (LearnLine::Race, Language::Spanish) => "Carrera",
            (LearnLine::Race, Language::English) => "Race",
        }
    }
}

/// The keyboard's curves as Options > Control > Keyboard behaviour chooses
/// them in Bézier, saved with the settings: each curve's shape, the times
/// of A, B and C, the lean D keeps at top speed, and whether the game is
/// learning them all from how the player rides (then every curve is the
/// learnt one, whatever its shape).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardShapes {
    pub lean_in: CurveChoice,
    pub stand_up: CurveChoice,
    pub unwind: CurveChoice,
    pub speed: CurveChoice,
    pub turn: CurveChoice,
    pub lean_in_seconds: f32,
    pub stand_up_seconds: f32,
    pub unwind_seconds: f32,
    pub top_speed_lean: f32,
    /// The line the game learns them from, if it is learning them: then
    /// every curve is the learnt one.
    pub learn: LearnLine,
    /// Whether learning corrects the learnt curves, or only notes where
    /// the bike strays (the key of free ride changes it, see
    /// [`super::learning`]).
    pub correct: bool,
}

impl Default for KeyboardShapes {
    fn default() -> Self {
        Self {
            lean_in: CurveChoice::classic(CurveKind::LeanIn),
            stand_up: CurveChoice::classic(CurveKind::StandUp),
            unwind: CurveChoice::classic(CurveKind::Unwind),
            speed: CurveChoice::classic(CurveKind::Speed),
            turn: CurveChoice::classic(CurveKind::Turn),
            lean_in_seconds: CLASSIC_SECONDS[0],
            stand_up_seconds: CLASSIC_SECONDS[1],
            unwind_seconds: CLASSIC_SECONDS[2],
            top_speed_lean: CLASSIC_TOP_SPEED_LEAN,
            learn: LearnLine::Off,
            correct: false,
        }
    }
}

impl KeyboardShapes {
    pub fn choice(&self, kind: CurveKind) -> &CurveChoice {
        match kind {
            CurveKind::LeanIn => &self.lean_in,
            CurveKind::StandUp => &self.stand_up,
            CurveKind::Unwind => &self.unwind,
            CurveKind::Speed => &self.speed,
            CurveKind::Turn => &self.turn,
        }
    }

    pub fn choice_mut(&mut self, kind: CurveKind) -> &mut CurveChoice {
        match kind {
            CurveKind::LeanIn => &mut self.lean_in,
            CurveKind::StandUp => &mut self.stand_up,
            CurveKind::Unwind => &mut self.unwind,
            CurveKind::Speed => &mut self.speed,
            CurveKind::Turn => &mut self.turn,
        }
    }

    /// The shape curve `kind` has now: learnt for all of them while the
    /// game learns them, the one chosen otherwise.
    pub fn shape(&self, kind: CurveKind) -> CurveShape {
        if self.learning() {
            CurveShape::Learned
        } else {
            self.choice(kind).shape
        }
    }

    /// The Bézier of curve `kind` now: its shape's own, its custom numbers,
    /// or what has been learnt (until anything has, its custom numbers).
    pub fn bezier(&self, kind: CurveKind) -> Bezier {
        let choice = self.choice(kind);
        match self.shape(kind) {
            CurveShape::Custom => choice.custom,
            CurveShape::Learned => choice.learned.unwrap_or(choice.custom),
            shape => shape.fixed(kind).unwrap_or(Bezier::LINEAR),
        }
    }

    /// Curve `kind` gets the shape `step` along. Learnt for the first time,
    /// it starts from the curve it had.
    pub fn step_shape(&mut self, kind: CurveKind, step: i32) {
        let was = self.bezier(kind);
        let choice = self.choice_mut(kind);
        choice.shape = crate::settings::cycle(&CURVE_SHAPES, choice.shape, step);
        if choice.shape == CurveShape::Learned && choice.learned.is_none() {
            choice.learned = Some(was);
        }
    }

    /// Whether the game is learning the curves.
    pub fn learning(&self) -> bool {
        self.learn != LearnLine::Off
    }

    /// The game learning every curve from `line`, or not: on, each curve
    /// not learnt yet starts from the one it has; off, each goes back to
    /// its shape.
    pub fn set_learn(&mut self, line: LearnLine) {
        if line != LearnLine::Off && !self.learning() {
            for kind in CURVE_KINDS {
                let was = self.bezier(kind);
                let choice = self.choice_mut(kind);
                choice.learned.get_or_insert(was);
            }
        }
        self.learn = line;
    }

    /// Corrects the learnt curve `kind` a little for the bike having gone
    /// `wide` (or inside) on it: A and C quicker for going wide, D keeping
    /// more lean, E less turn for the lean for cutting inside, B quicker for
    /// standing up too slowly (inside) and slower for too quickly (wide).
    /// The curve only rises or falls, both control points together, within
    /// the square; E never comes down below [`TURN_FLOOR`], so a touch of
    /// the key always turns the bike.
    pub fn correct_for(&mut self, kind: CurveKind, wide: bool) {
        let up = match (kind, wide) {
            (CurveKind::LeanIn | CurveKind::Unwind, true) => 1.0,
            (CurveKind::Speed, true) => -1.0,
            (CurveKind::Turn, false) => -1.0,
            (CurveKind::StandUp, false) => 1.0,
            (CurveKind::StandUp, true) => -1.0,
            _ => return,
        };
        let Some(curve) = self.choice_mut(kind).learned.as_mut() else {
            return;
        };
        let floor = if kind == CurveKind::Turn {
            TURN_FLOOR
        } else {
            (0.0, 0.0)
        };
        let moved = |value: f32, floor: f32| {
            ((value + up * NUDGE).clamp(floor, 1.0) * 1000.0).round() / 1000.0
        };
        curve.y1 = moved(curve.y1, floor.0);
        curve.y2 = moved(curve.y2, floor.1);
    }

    /// The number curve `kind` is scaled by, if it has one: the seconds of
    /// A, B and C, the lean kept at top speed of D.
    pub fn scale(&self, kind: CurveKind) -> Option<f32> {
        match kind {
            CurveKind::LeanIn => Some(self.lean_in_seconds),
            CurveKind::StandUp => Some(self.stand_up_seconds),
            CurveKind::Unwind => Some(self.unwind_seconds),
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
            CurveKind::LeanIn => self.lean_in_seconds = stepped(self.lean_in_seconds, SECONDS),
            CurveKind::StandUp => {
                self.stand_up_seconds = stepped(self.stand_up_seconds, SECONDS);
            }
            CurveKind::Unwind => {
                // From the least time down to nought, and back up.
                let next = stepped(self.unwind_seconds, (0.0, SECONDS.1));
                self.unwind_seconds = if next > 0.0 && next < SECONDS.0 {
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

    /// The curves the bike follows.
    pub fn curves(&self) -> KeyboardCurves {
        let timed = |kind, seconds| TimedCurve {
            curve: self.bezier(kind),
            seconds,
        };
        KeyboardCurves {
            lean_in: timed(CurveKind::LeanIn, self.lean_in_seconds),
            stand_up: timed(CurveKind::StandUp, self.stand_up_seconds),
            unwind: timed(CurveKind::Unwind, self.unwind_seconds),
            speed: self.bezier(CurveKind::Speed),
            top_speed_lean: self.top_speed_lean,
            turn: self.bezier(CurveKind::Turn),
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

    /// What its control points' coordinates measure, as the options name
    /// them: across and up.
    pub fn axes(self, lang: Language) -> (&'static str, &'static str) {
        match (self, lang) {
            (CurveKind::LeanIn, Language::Spanish) => ("tiempo", "tumbada"),
            (CurveKind::LeanIn, Language::English) => ("time", "lean"),
            (CurveKind::StandUp | CurveKind::Unwind, Language::Spanish) => ("tiempo", "levantada"),
            (CurveKind::StandUp | CurveKind::Unwind, Language::English) => ("time", "rise"),
            (CurveKind::Speed, Language::Spanish) => ("velocidad", "reducción"),
            (CurveKind::Speed, Language::English) => ("speed", "reduction"),
            (CurveKind::Turn, Language::Spanish) => ("tumbada", "giro"),
            (CurveKind::Turn, Language::English) => ("lean", "turn"),
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
    fn every_curve_starts_classic_and_takes_each_shape_in_turn() {
        let mut shapes = KeyboardShapes::default();
        for kind in CURVE_KINDS {
            assert_eq!(shapes.shape(kind), CurveShape::Classic);
            // Its custom numbers start from the classic curve.
            assert_eq!(shapes.choice(kind).custom, shapes.bezier(kind));
        }
        shapes.step_shape(CurveKind::Turn, 1);
        assert_eq!(shapes.shape(CurveKind::Turn), CurveShape::Linear);
        shapes.step_shape(CurveKind::Turn, 1);
        assert_eq!(
            shapes.bezier(CurveKind::Turn),
            Bezier::new(0.25, 0.1, 0.25, 1.0)
        );
        // Custom: its own numbers.
        shapes.turn.custom = Bezier::new(0.1, 0.6, 0.5, 1.0);
        shapes.turn.shape = CurveShape::Custom;
        assert_eq!(shapes.bezier(CurveKind::Turn), shapes.turn.custom);
        assert_eq!(shapes.curves().turn, shapes.turn.custom);
        // Back round from the first to the last: learnt.
        shapes.step_shape(CurveKind::LeanIn, -1);
        assert_eq!(shapes.shape(CurveKind::LeanIn), CurveShape::Learned);
    }

    #[test]
    fn a_learnt_curve_starts_from_the_one_it_had_and_learning_makes_them_all_learnt() {
        let mut shapes = KeyboardShapes::default();
        shapes.speed.shape = CurveShape::EaseIn;
        let ease_in = shapes.bezier(CurveKind::Speed);
        // Chosen learnt, it starts from the curve it had.
        shapes.step_shape(CurveKind::Speed, 4);
        assert_eq!(shapes.shape(CurveKind::Speed), CurveShape::Learned);
        assert_eq!(shapes.bezier(CurveKind::Speed), ease_in);
        shapes.speed.shape = CurveShape::Linear;
        // Learning: every curve learnt, each from what it had.
        let before = shapes.curves();
        shapes.set_learn(LearnLine::Centre);
        for kind in CURVE_KINDS {
            assert_eq!(shapes.shape(kind), CurveShape::Learned);
        }
        assert_eq!(shapes.bezier(CurveKind::LeanIn), before.lean_in.curve);
        // D was learnt before: it keeps what it had learnt.
        assert_eq!(shapes.bezier(CurveKind::Speed), ease_in);
        // What is learnt changes the curve followed.
        shapes.lean_in.learned = Some(Bezier::LINEAR);
        assert_eq!(shapes.curves().lean_in.curve, Bezier::LINEAR);
        // Off, each back to its shape.
        shapes.set_learn(LearnLine::Off);
        assert_eq!(shapes.shape(CurveKind::Speed), CurveShape::Linear);
        assert_eq!(shapes.curves(), before);
    }

    #[test]
    fn corrections_move_a_learnt_curve_the_way_it_strayed_within_bounds() {
        let mut shapes = KeyboardShapes::default();
        // Nothing learnt yet: nothing to correct.
        shapes.correct_for(CurveKind::LeanIn, true);
        assert_eq!(shapes, KeyboardShapes::default());
        shapes.set_learn(LearnLine::Centre);
        let was = shapes.bezier(CurveKind::LeanIn);
        shapes.correct_for(CurveKind::LeanIn, true);
        let now = shapes.bezier(CurveKind::LeanIn);
        assert!((now.y1 - was.y1 - NUDGE).abs() < 1e-3 && (now.y2 - was.y2 - NUDGE).abs() < 1e-3);
        assert_eq!((now.x1, now.x2), (was.x1, was.x2));
        // D: more lean (less reduction) for going wide.
        let d = shapes.bezier(CurveKind::Speed);
        shapes.correct_for(CurveKind::Speed, true);
        assert!(shapes.bezier(CurveKind::Speed).y2 < d.y2);
        // E: less turn for cutting inside, never below its floor.
        for _ in 0..200 {
            shapes.correct_for(CurveKind::Turn, false);
        }
        let e = shapes.bezier(CurveKind::Turn);
        assert_eq!((e.y1, e.y2), TURN_FLOOR);
        // What does not point at a curve changes nothing.
        let before = shapes;
        shapes.correct_for(CurveKind::Turn, true);
        shapes.correct_for(CurveKind::LeanIn, false);
        assert_eq!(shapes, before);
    }

    #[test]
    fn holding_a_key_leans_over_along_a_in_its_time() {
        let curves = KeyboardCurves::default();
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
        let curves = KeyboardCurves::default();
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
        let curves = KeyboardCurves::default();
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
        let curves = KeyboardCurves::default();
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
        let curves = KeyboardCurves::default();
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
        let curves = KeyboardCurves::default();
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
        let mut curves = KeyboardCurves::default();
        assert_eq!(curves.turn_for(0.0), 0.0);
        assert!((curves.turn_for(0.5) - 0.5).abs() < 1e-5);
        assert!((curves.turn_for(-0.5) + 0.5).abs() < 1e-5);
        curves.turn = Bezier::new(0.1, 0.6, 0.5, 1.0);
        assert!(curves.turn_for(0.2) > 0.4);
    }

    #[test]
    fn the_same_ride_at_any_frame_rate() {
        // Smooth: easing in and out, unwinding over a moment.
        let mut shapes = KeyboardShapes {
            lean_in_seconds: 0.5,
            stand_up_seconds: 0.6,
            unwind_seconds: 0.15,
            ..default()
        };
        shapes.lean_in.shape = CurveShape::EaseInOut;
        shapes.stand_up.shape = CurveShape::EaseOut;
        shapes.unwind.shape = CurveShape::Ease;
        let curves = shapes.curves();
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
        let mut curves = KeyboardShapes::default();
        for _ in 0..100 {
            curves.step_scale(CurveKind::LeanIn, 1);
        }
        assert_eq!(curves.lean_in_seconds, SECONDS.1);
        for _ in 0..100 {
            curves.step_scale(CurveKind::Speed, -1);
        }
        assert_eq!(curves.top_speed_lean, TOP_SPEED_LEAN.0);
        // C: from nought up to the least time, and back down to nought.
        assert_eq!(curves.unwind_seconds, 0.0);
        curves.step_scale(CurveKind::Unwind, 1);
        assert_eq!(curves.unwind_seconds, SECONDS.0);
        curves.step_scale(CurveKind::Unwind, -1);
        assert_eq!(curves.unwind_seconds, 0.0);
        curves.step_scale(CurveKind::Unwind, -1);
        assert_eq!(curves.unwind_seconds, 0.0);
    }
}
