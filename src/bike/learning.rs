//! The keyboard's self-learning (Options > Control > Keyboard behaviour in
//! Bézier, "Self-learning": centre or race): riding alone in free ride, the
//! line the curves are learnt from is drawn on the road, and how far the
//! bike strays from it is watched. Where it runs wide or cuts inside, and
//! on which curve the lean was then, is noted for the panel of the curves
//! (see [`crate::curves_hud`]), so that what it finds can be checked before
//! it corrects anything.
//!
//! The line is the centre of the road or the racing line (see
//! [`crate::track::racing_line`]). What it notes, in a corner (the road
//! bending tighter than [`CORNER`]), past [`STRAY`] m from the line and
//! still going away from it:
//!
//! | Lean | Going | Curve |
//! |---|---|---|
//! | leaning in, short of its limit | wide | A: too slow to lean over |
//! | held at its limit | wide | D: too little lean at this speed |
//! | coming upright the other way | wide | C: too slow to change sides |
//! | leaning in or let go | inside | E: too much turn for the lean |
//!
//! and on the straight after a corner, standing up, B: inside (back towards
//! the corner) if it stands up too slowly, wide if too quickly.
//!
//! Going off the road in a corner or just after it (onto the grass, into a
//! wall, or down) is an outing, noted the same way and counting for
//! [`OUTING_WEIGHT`] corrections; but going wide with the bike already
//! turning all it can is the bike's limit, not the curves': it is counted
//! apart and corrects nothing (the corner wants less speed). Going wide
//! held at the lean's limit corrects D and E both: more lean at that speed,
//! more turn for it.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use super::curves::{CurveKind, CurveProbe, KeyboardMode, KeyboardShapes, LearnLine, Phase};
use super::{BikeState, CrashState, Player};
use crate::menu::GameState;
use crate::race::{Mode, Rules};
use crate::settings::Settings;
use crate::track::{SURFACE_Y, Track, racing_line::racing_line};
use crate::welcome::GameMode;

pub struct LearningPlugin;

impl Plugin for LearningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LearningLog>()
            .init_resource::<TargetLine>()
            .add_systems(OnEnter(GameState::Race), start_log)
            .add_systems(
                Update,
                (
                    follow_target_line,
                    show_target_line,
                    toggle_correction,
                    watch_the_line,
                )
                    .chain()
                    .run_if(in_state(GameState::Race)),
            );
    }
}

/// What changes between only noting and correcting the curves, in free
/// ride while learning: T on the keyboard, up on the d-pad.
pub const CORRECT_KEY: KeyCode = KeyCode::KeyT;
pub const CORRECT_BUTTON: GamepadButton = GamepadButton::DPadUp;

/// How many corrections an outing counts for.
pub const OUTING_WEIGHT: usize = 3;
/// From how much of all its turn the bike is turning all it can.
const ALL_ITS_TURN: f32 = 0.98;
/// How far past the asphalt's edge counts as off the road (m: past its
/// curbs), and how near a wall counts as against it.
const OFF_THE_ROAD: f32 = 1.0;
const WALL_TOUCH: f32 = 0.6;

/// How far from the line counts as straying from it (m).
pub const STRAY: f32 = 1.5;
/// How tight the road bends to count as a corner (1/m: a radius of 150 m).
pub const CORNER: f32 = 1.0 / 150.0;
/// How many points of the centre line each way the bend is measured over
/// (about 2 m apart).
const BEND_WINDOW: isize = 6;
/// How long after a note on a curve the next one on it can come (s).
const NOTE_GAP: f32 = 1.0;
/// How long the straight after a corner counts as its way out (s).
const WAY_OUT: f32 = 1.5;
/// How wide the line is drawn (m), how high over the asphalt (itself
/// [`SURFACE_Y`] over the track's height), and its colour:
/// the amber of what is learnt.
const LINE_WIDTH: f32 = 0.4;
const LINE_LIFT: f32 = 0.04;
const LINE_COLOR: Color = Color::srgba(1.0, 0.72, 0.2, 0.75);

/// Which way the bike strayed from the line, round the corner: out wide or
/// in towards its inside.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Wide,
    Inside,
}

/// What the self-learning has seen this race: whether it is watching, how
/// far the bike is from the line now (m, positive out wide round the
/// corner, or to the left on a straight), whether it is in a corner, and,
/// for each curve, how many times it went wide and inside, and the last
/// note.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct LearningLog {
    pub active: bool,
    pub stray: f32,
    pub in_corner: bool,
    pub notes: [[u32; 2]; 6],
    pub last: Option<(CurveKind, Side, f32)>,
    /// How many times the bike went off the road in a corner, and how many
    /// of those turning all it could; whether it was off in the step
    /// before, and whether the last note was an outing.
    pub outings: u32,
    pub at_limit: u32,
    pub(crate) was_off: bool,
    pub last_outing: bool,
    /// The bend of the last corner (1/m, positive to the left) and when it
    /// was last in it (s), for its way out; how far to the left of the line
    /// the bike was in the step before; when each curve was last noted.
    pub(crate) corner: Option<(f32, f32)>,
    pub(crate) was: f32,
    pub(crate) noted: [f32; 6],
}

impl LearningLog {
    /// How many times curve `kind` went `side`.
    pub fn count(&self, kind: CurveKind, side: Side) -> u32 {
        self.notes[kind as usize][side as usize]
    }

    /// The curve the lean was on going `side` round a corner, as `probe`
    /// says, if any: wide while leaning in short of what is asked is A,
    /// held where the stick asks short of the limit R, at the limit D,
    /// unwinding C, standing up B; inside is E.
    fn blame(side: Side, probe: &CurveProbe) -> Option<CurveKind> {
        let lean = probe.lean;
        if side == Side::Inside {
            return Some(CurveKind::Turn);
        }
        match lean.phase {
            Phase::LeaningIn if lean.lean.abs() < probe.asked.min(probe.limit) - 0.02 => {
                Some(CurveKind::LeanIn)
            }
            Phase::LeaningIn if probe.stick && probe.asked < probe.limit - 0.02 => {
                Some(CurveKind::Response)
            }
            Phase::LeaningIn => Some(CurveKind::Speed),
            Phase::Unwinding => Some(CurveKind::Unwind),
            Phase::StandingUp => Some(CurveKind::StandUp),
            Phase::Upright => None,
        }
    }

    /// The bike going off the road (`off`) `stray` m left of the line, at
    /// `now`: in a corner bending `bend` (or on the way out of the last
    /// one), an outing on the curve to blame, noted whatever came just
    /// before; turning all it can, out wide, the bike's limit instead.
    /// Returns the curve and the side, to correct.
    pub fn went_off(
        &mut self,
        off: bool,
        stray: f32,
        bend: f32,
        probe: &CurveProbe,
        now: f32,
    ) -> Option<(CurveKind, Side)> {
        let fresh = off && !self.was_off;
        self.was_off = off;
        if !fresh {
            return None;
        }
        let bend = if bend.abs() > CORNER {
            bend
        } else {
            match self.corner {
                Some((corner, left)) if now - left < WAY_OUT => corner,
                _ => return None,
            }
        };
        let side = if -stray * bend.signum() > 0.0 {
            Side::Wide
        } else {
            Side::Inside
        };
        self.outings += 1;
        if side == Side::Wide && probe.turn.abs() >= ALL_ITS_TURN {
            self.at_limit += 1;
            return None;
        }
        let kind = Self::blame(side, probe)?;
        self.notes[kind as usize][side as usize] += 1;
        self.noted[kind as usize] = now;
        self.last = Some((kind, side, now));
        self.last_outing = true;
        Some((kind, side))
    }

    /// Notes curve `kind` going `side` at `now` (s), if not noted just
    /// before.
    fn note(&mut self, kind: CurveKind, side: Side, now: f32) {
        let at = kind as usize;
        if now - self.noted[at] < NOTE_GAP && self.noted[at] > 0.0 {
            return;
        }
        self.noted[at] = now;
        self.notes[at][side as usize] += 1;
        self.last = Some((kind, side, now));
        self.last_outing = false;
    }

    /// Goes on with the bike `stray` m to the left of the line where the
    /// road bends `bend` (1/m, positive to the left), its lean as `probe`
    /// says, at `now` (s).
    pub fn watch(&mut self, stray: f32, bend: f32, probe: &CurveProbe, now: f32) {
        let in_corner = bend.abs() > CORNER;
        let moved = stray - self.was;
        self.was = stray;
        // Round a corner bending `bend`: how far out wide of the line the
        // bike is (negative inside), and whether it is going further from
        // the line, past straying.
        let round = |bend: f32| {
            let (out, going) = (-stray * bend.signum(), -moved * bend.signum());
            (out, out.abs() > STRAY && going * out > 0.0)
        };
        let lean = probe.lean;
        if in_corner {
            let (out, leaving) = round(bend);
            if leaving && out > 0.0 {
                // Standing up or upright in a corner, wide: not the lean's.
                if matches!(lean.phase, Phase::LeaningIn | Phase::Unwinding)
                    && let Some(kind) = Self::blame(Side::Wide, probe)
                {
                    self.note(kind, Side::Wide, now);
                }
            } else if leaving {
                self.note(CurveKind::Turn, Side::Inside, now);
            }
            self.corner = Some((bend, now));
            self.stray = out;
        } else {
            // On the way out of the last corner, standing up.
            if let Some((corner, left)) = self.corner
                && now - left < WAY_OUT
            {
                let (out, leaving) = round(corner);
                if leaving && lean.phase == Phase::StandingUp {
                    let side = if out > 0.0 { Side::Wide } else { Side::Inside };
                    self.note(CurveKind::StandUp, side, now);
                }
            }
            self.stray = stray;
        }
        self.in_corner = in_corner;
    }
}

/// The line drawn and watched: which one, and how far to the left of the
/// centre line it is at each of its points (all nought for the centre).
#[derive(Resource, Default)]
pub struct TargetLine {
    pub line: Option<LearnLine>,
    pub lateral: Vec<f32>,
}

impl TargetLine {
    /// How far to the left of the centre line the line is between points
    /// `index` and the next, `t` of the way.
    pub fn at(&self, index: usize, t: f32) -> f32 {
        let n = self.lateral.len();
        if n == 0 {
            return 0.0;
        }
        let (a, b) = (self.lateral[index % n], self.lateral[(index + 1) % n]);
        a + (b - a) * t
    }
}

/// The line the self-learning watches now, if it is: with the keyboard or
/// the stick in Bézier and learning, one player in free ride. If both are
/// learning, the keyboard's line.
pub fn learning_line(
    settings: &Settings,
    rules: Option<&Rules>,
    mode: GameMode,
) -> Option<LearnLine> {
    let free = rules.is_some_and(|rules| rules.mode == Mode::Free);
    if !free || mode != GameMode::SinglePlayer {
        return None;
    }
    let learning = |mode: KeyboardMode, shapes: &KeyboardShapes| {
        (mode == KeyboardMode::Bezier && shapes.learning()).then_some(shapes.learn)
    };
    learning(settings.keyboard_mode, &settings.keyboard_shapes)
        .or_else(|| learning(settings.stick_mode, &settings.stick_shapes))
}

/// The curves the self-learning works on: the stick's while it leans the
/// bike, the keyboard's otherwise.
pub fn learnt_shapes(settings: &mut Settings, stick: bool) -> &mut KeyboardShapes {
    if stick {
        &mut settings.stick_shapes
    } else {
        &mut settings.keyboard_shapes
    }
}

/// A new race, a new log.
fn start_log(mut log: ResMut<LearningLog>) {
    *log = LearningLog::default();
}

/// The line worked out again when the track or the line chosen change.
fn follow_target_line(
    settings: Res<Settings>,
    rules: Option<Res<Rules>>,
    mode: Res<GameMode>,
    track: Res<Track>,
    mut target: ResMut<TargetLine>,
) {
    let line = learning_line(&settings, rules.as_deref(), *mode);
    if line == target.line && !track.is_changed() {
        return;
    }
    let n = track.centerline().len();
    target.lateral = match line {
        Some(LearnLine::Race) => racing_line(track.centerline(), |i| track.half_width_at(i)),
        _ => vec![0.0; n],
    };
    target.line = line;
}

/// The line on the road.
#[derive(Component)]
struct DrawnLine;

/// The line drawn on the road while it is watched, again when it changes.
fn show_target_line(
    mut commands: Commands,
    target: Res<TargetLine>,
    track: Res<Track>,
    drawn: Query<Entity, With<DrawnLine>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !target.is_changed() && !drawn.is_empty() {
        return;
    }
    for entity in &drawn {
        commands.entity(entity).despawn();
    }
    if target.line.is_none() || target.lateral.is_empty() {
        return;
    }
    commands.spawn((
        DrawnLine,
        DespawnOnExit(GameState::Race),
        Mesh3d(meshes.add(line_mesh(&track, &target.lateral))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LINE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            depth_bias: 50.0,
            // Seen from above whichever way its triangles wind.
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Transform::default(),
    ));
}

/// A strip `LINE_WIDTH` wide all round the lap, `lateral` m to the left of
/// each point of the centre line, just over the road.
fn line_mesh(track: &Track, lateral: &[f32]) -> Mesh {
    let n = lateral.len();
    let mut positions = Vec::with_capacity(2 * n);
    for (index, &left) in lateral.iter().enumerate() {
        for side in [-0.5, 0.5] {
            let point = track.surface_point(index, left + side * LINE_WIDTH);
            positions.push(point + Vec3::Y * (SURFACE_Y + LINE_LIFT));
        }
    }
    let mut indices = Vec::with_capacity(6 * n);
    for index in 0..n as u32 {
        let next = (index + 1) % n as u32;
        let (a, b, c, d) = (2 * index, 2 * index + 1, 2 * next, 2 * next + 1);
        indices.extend([a, c, b, b, c, d]);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// While learning, T (or up on the d-pad) changes between only noting
/// where the bike strays and correcting the curves for it; not paused.
fn toggle_correction(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    pause: Res<crate::pause::Pause>,
    target: Res<TargetLine>,
    probe: Res<CurveProbe>,
    mut settings: ResMut<Settings>,
) {
    let pressed = keyboard.just_pressed(CORRECT_KEY)
        || gamepads.iter().any(|pad| pad.just_pressed(CORRECT_BUTTON));
    if pressed && target.line.is_some() && !pause.is_open() {
        let shapes = learnt_shapes(&mut settings, probe.stick);
        shapes.correct = !shapes.correct;
    }
}

/// Watches how far the player's bike strays from the line and, correcting,
/// corrects the curve it strayed on.
#[allow(clippy::too_many_arguments)]
fn watch_the_line(
    time: Res<Time>,
    target: Res<TargetLine>,
    track: Res<Track>,
    probe: Res<CurveProbe>,
    player: Query<(&BikeState, &CrashState), With<Player>>,
    mut log: ResMut<LearningLog>,
    mut settings: ResMut<Settings>,
) {
    let active = target.line.is_some() && probe.live;
    if log.active != active {
        log.active = active;
    }
    let Ok((state, crash)) = player.single() else {
        return;
    };
    if !active {
        return;
    }
    let nearest = track.nearest_at(state.position, state.elevation);
    if nearest.road != 0 {
        return;
    }
    let now = time.elapsed_secs();
    // Off the road: past the asphalt and its curbs, against a wall, or
    // down.
    let off = crash.is_down()
        || nearest.distance > track.half_width_beside(&nearest) + OFF_THE_ROAD
        || nearest.distance + WALL_TOUCH >= track.edge_beside(&nearest);
    let stray = nearest.lateral - target.at(nearest.index, nearest.t);
    let bend = (-BEND_WINDOW..=BEND_WINDOW)
        .map(|k| track.curvature((nearest.index as isize + k).max(0) as usize))
        .sum::<f32>()
        / (2 * BEND_WINDOW + 1) as f32;
    if let Some((kind, side)) = log.went_off(off, stray, bend, &probe, now) {
        if correcting(&settings, probe.stick) {
            for _ in 0..OUTING_WEIGHT {
                correct(&mut settings, probe.stick, kind, side);
            }
        }
        return;
    }
    if crash.is_down() || !state.grounded || state.speed < 5.0 {
        return;
    }
    let before = log.last;
    log.watch(stray, bend, &probe, now);
    if log.last != before
        && correcting(&settings, probe.stick)
        && let Some((kind, side, _)) = log.last
    {
        correct(&mut settings, probe.stick, kind, side);
    }
}

/// Whether the curves of the stick (or the keyboard) are being corrected,
/// read without touching the settings, which are saved when they change.
fn correcting(settings: &Settings, stick: bool) -> bool {
    if stick {
        settings.stick_shapes.correct
    } else {
        settings.keyboard_shapes.correct
    }
}

/// Corrects curve `kind` of the stick's (or the keyboard's) for going
/// `side`: at the lean's limit, wide, D and E both.
fn correct(settings: &mut Settings, stick: bool, kind: CurveKind, side: Side) {
    let shapes = learnt_shapes(settings, stick);
    let wide = side == Side::Wide;
    shapes.correct_for(kind, wide);
    if kind == CurveKind::Speed && wide {
        shapes.correct_for(CurveKind::Turn, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bike::curves::CurveLean;

    /// The lean on `phase`, `lean` over, its limit `limit`.
    fn probe(phase: Phase, lean: f32, limit: f32) -> CurveProbe {
        let mut following = CurveLean::default();
        (following.lean, following.phase, following.progress) = (lean, phase, 0.5);
        CurveProbe {
            live: true,
            stick: false,
            travel: 0.0,
            asked: 1.0,
            lean: following,
            speed_share: 0.5,
            limit,
            turn: lean,
        }
    }

    /// The same, the stick's, pushed asking for `asked`.
    fn stick_probe(phase: Phase, lean: f32, asked: f32, limit: f32) -> CurveProbe {
        CurveProbe {
            stick: true,
            travel: asked,
            asked,
            ..probe(phase, lean, limit)
        }
    }

    /// The bike going from `from` to `to` m left of the line, in steps of a
    /// tenth of a second, where the road bends `bend`.
    fn ride(log: &mut LearningLog, from: f32, to: f32, bend: f32, probe: &CurveProbe, start: f32) {
        for k in 0..=10 {
            let stray = from + (to - from) * k as f32 / 10.0;
            log.watch(stray, bend, probe, start + k as f32 * 0.1);
        }
    }

    const LEFT: f32 = 1.0 / 60.0;

    #[test]
    fn going_wide_while_leaning_in_is_a_too_slow() {
        let mut log = LearningLog::default();
        // A left-hander: wide is to the right.
        ride(
            &mut log,
            0.0,
            -3.0,
            LEFT,
            &probe(Phase::LeaningIn, 0.4, 0.9),
            1.0,
        );
        assert_eq!(log.count(CurveKind::LeanIn, Side::Wide), 1);
        assert!(log.in_corner && log.stray > STRAY);
        // At its limit: D.
        ride(
            &mut log,
            0.0,
            -3.0,
            LEFT,
            &probe(Phase::LeaningIn, 0.9, 0.9),
            5.0,
        );
        assert_eq!(log.count(CurveKind::Speed, Side::Wide), 1);
        // Unwinding: C.
        ride(
            &mut log,
            0.0,
            3.0,
            -LEFT,
            &probe(Phase::Unwinding, -0.5, 0.9),
            9.0,
        );
        assert_eq!(log.count(CurveKind::Unwind, Side::Wide), 1);
    }

    #[test]
    fn with_the_stick_going_wide_held_where_it_asks_is_its_response() {
        let mut log = LearningLog::default();
        // Held at the lean the stick asks, short of the limit: R.
        let held = stick_probe(Phase::LeaningIn, 0.5, 0.5, 0.9);
        ride(&mut log, 0.0, -3.0, LEFT, &held, 1.0);
        assert_eq!(log.count(CurveKind::Response, Side::Wide), 1);
        // Still leaning in towards it: A.
        let leaning = stick_probe(Phase::LeaningIn, 0.3, 0.5, 0.9);
        ride(&mut log, 0.0, -3.0, LEFT, &leaning, 5.0);
        assert_eq!(log.count(CurveKind::LeanIn, Side::Wide), 1);
        // Pushed all the way, at the limit: D.
        let full = stick_probe(Phase::LeaningIn, 0.9, 1.0, 0.9);
        ride(&mut log, 0.0, -3.0, LEFT, &full, 9.0);
        assert_eq!(log.count(CurveKind::Speed, Side::Wide), 1);
    }

    #[test]
    fn cutting_inside_is_too_much_turn_and_coming_back_is_nothing() {
        let mut log = LearningLog::default();
        ride(
            &mut log,
            0.0,
            3.0,
            LEFT,
            &probe(Phase::LeaningIn, 0.5, 0.9),
            1.0,
        );
        assert_eq!(log.count(CurveKind::Turn, Side::Inside), 1);
        // Coming back towards the line, however far: nothing more.
        ride(
            &mut log,
            3.0,
            2.0,
            LEFT,
            &probe(Phase::LeaningIn, 0.5, 0.9),
            5.0,
        );
        assert_eq!(log.notes.iter().flatten().sum::<u32>(), 1);
        // Near the line, nothing.
        ride(
            &mut log,
            0.0,
            1.0,
            LEFT,
            &probe(Phase::LeaningIn, 0.5, 0.9),
            9.0,
        );
        assert_eq!(log.notes.iter().flatten().sum::<u32>(), 1);
    }

    #[test]
    fn on_the_way_out_of_a_corner_standing_up_is_b() {
        let mut log = LearningLog::default();
        ride(
            &mut log,
            0.0,
            0.5,
            LEFT,
            &probe(Phase::LeaningIn, 0.5, 0.9),
            1.0,
        );
        // Out on the straight, standing up and still turning in: too slow.
        ride(
            &mut log,
            0.5,
            3.0,
            0.0,
            &probe(Phase::StandingUp, 0.3, 1.0),
            2.1,
        );
        assert_eq!(log.count(CurveKind::StandUp, Side::Inside), 1);
        // Long after the corner, nothing.
        ride(
            &mut log,
            0.0,
            3.0,
            0.0,
            &probe(Phase::StandingUp, 0.3, 1.0),
            10.0,
        );
        assert_eq!(log.count(CurveKind::StandUp, Side::Inside), 1);
    }

    #[test]
    fn going_off_in_a_corner_is_an_outing_on_the_curve_to_blame() {
        let mut log = LearningLog::default();
        // Leaning in, short of the limit, off wide in a left-hander: A.
        let leaning = probe(Phase::LeaningIn, 0.5, 0.9);
        assert_eq!(
            log.went_off(true, -6.0, LEFT, &leaning, 1.0),
            Some((CurveKind::LeanIn, Side::Wide))
        );
        assert!(log.last_outing && log.outings == 1);
        // Still off: nothing more; back on and off again inside: E.
        assert_eq!(log.went_off(true, -6.0, LEFT, &leaning, 1.1), None);
        assert_eq!(log.went_off(false, 0.0, LEFT, &leaning, 1.2), None);
        assert_eq!(
            log.went_off(true, 6.0, LEFT, &leaning, 1.3),
            Some((CurveKind::Turn, Side::Inside))
        );
        // At the lean's limit, wide: D (the correction raises E too).
        log.went_off(false, 0.0, LEFT, &leaning, 2.0);
        let held = probe(Phase::LeaningIn, 0.9, 0.9);
        assert_eq!(
            log.went_off(true, -6.0, LEFT, &held, 2.1),
            Some((CurveKind::Speed, Side::Wide))
        );
        // Turning all it can: the bike's limit, no curve to blame.
        log.went_off(false, 0.0, LEFT, &leaning, 3.0);
        let all = CurveProbe { turn: 1.0, ..held };
        assert_eq!(log.went_off(true, -6.0, LEFT, &all, 3.1), None);
        assert_eq!((log.outings, log.at_limit), (4, 1));
        // On a straight, long after any corner: not an outing.
        log.went_off(false, 0.0, 0.0, &leaning, 9.0);
        assert_eq!(log.went_off(true, -6.0, 0.0, &leaning, 9.1), None);
        assert_eq!(log.outings, 4);
    }

    #[test]
    fn going_wide_at_the_lean_s_limit_corrects_d_and_e() {
        let mut settings = Settings::default();
        settings.keyboard_shapes.set_learn(LearnLine::Centre);
        let before = settings.keyboard_shapes;
        correct(&mut settings, false, CurveKind::Speed, Side::Wide);
        let after = settings.keyboard_shapes;
        assert!(after.bezier(CurveKind::Speed).y2 < before.bezier(CurveKind::Speed).y2);
        assert!(after.bezier(CurveKind::Turn).y2 > before.bezier(CurveKind::Turn).y2);
        // The stick's, untouched.
        assert_eq!(settings.stick_shapes, KeyboardShapes::classic(true));
    }

    #[test]
    fn a_curve_is_noted_once_a_second_at_most() {
        let mut log = LearningLog::default();
        let leaning = probe(Phase::LeaningIn, 0.4, 0.9);
        for k in 0..20 {
            log.watch(-2.0 - k as f32 * 0.1, LEFT, &leaning, 1.0 + k as f32 * 0.1);
        }
        assert_eq!(log.count(CurveKind::LeanIn, Side::Wide), 2);
        assert_eq!(
            log.last.map(|(kind, side, _)| (kind, side)),
            Some((CurveKind::LeanIn, Side::Wide))
        );
    }

    #[test]
    fn the_target_line_is_read_between_its_points() {
        let target = TargetLine {
            line: Some(LearnLine::Race),
            lateral: vec![0.0, 2.0, 4.0],
        };
        assert_eq!(target.at(0, 0.5), 1.0);
        // Round the end of the lap, back to the first.
        assert_eq!(target.at(2, 0.5), 2.0);
        assert_eq!(TargetLine::default().at(3, 0.5), 0.0);
    }

    #[test]
    fn it_learns_only_alone_in_free_ride_with_the_keyboard_in_bezier() {
        let mut settings = Settings::default();
        let free = Rules {
            mode: Mode::Free,
            ..default()
        };
        let one = GameMode::SinglePlayer;
        assert_eq!(learning_line(&settings, Some(&free), one), None);
        // The stick's learning.
        settings.stick_mode = KeyboardMode::Bezier;
        settings.stick_shapes.set_learn(LearnLine::Centre);
        assert_eq!(
            learning_line(&settings, Some(&free), one),
            Some(LearnLine::Centre)
        );
        // Both: the keyboard's line.
        settings.keyboard_mode = KeyboardMode::Bezier;
        settings.keyboard_shapes.set_learn(LearnLine::Race);
        assert_eq!(
            learning_line(&settings, Some(&free), one),
            Some(LearnLine::Race)
        );
        let race = Rules {
            mode: Mode::Race,
            ..default()
        };
        assert_eq!(learning_line(&settings, Some(&race), one), None);
        assert_eq!(
            learning_line(&settings, Some(&free), GameMode::SplitScreen),
            None
        );
    }
}
