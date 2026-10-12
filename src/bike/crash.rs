//! Contacts between bikes and falls.
//!
//! Two bikes that touch are resolved as two rigid bodies of the same mass
//! (bike plus rider) exchanging an impulse at the contact point: momentum is
//! conserved, the contact is mostly plastic (fairings and bodies crumple, they
//! do not bounce) and the fairings rub against each other with some friction.
//! The part of the change of velocity along the bike goes into its speed; the
//! part across it, and the spin, go into a [`Knock`] that the tyres scrub off
//! in a fraction of a second. Without contacts the knock is zero and the bike
//! handles exactly as [`super::step`] says.
//!
//! A rider only goes down when the tyres cannot absorb the hit: a sideways
//! jolt, a spin or a hit on the front wheel bigger than a rider can ride out
//! (see [`SIDE_FALL`], [`YAW_FALL`], [`FRONT_FALL`]). Rubbing fairings or a
//! nudge from behind is part of racing; a T-bone at speed is not.
//!
//! When a rider goes down, bike and rider separate: the bike drops onto its
//! side and slides with the velocity it had after the hit, and the rider keeps
//! going with the momentum they had before it (nothing holds them to the
//! bike), is thrown, lands, tumbles and slides on their leathers. Both stop by
//! friction. Accident reconstruction gives the drag factors: about 0.3–0.55
//! for a motorcycle sliding on its side on asphalt (less with a fairing) and
//! more on grass, where it digs in; about 0.5–0.7 for a rider sliding on
//! asphalt. Once everything is still the bike is put back on the track at
//! once, on the spot where it ended up ([`rejoin_pose`]).

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::{BikeState, GRAVITY, WallHit, footprint, move_toward, wall_contact};
use crate::geometry::{approach, forward};
use crate::track::{Surface, Track};

/// Collision shape seen from above: a capsule along the bike, from wheel to
/// wheel, as wide as bike and rider.
const DISC_RADIUS: f32 = 0.45;
const DISC_SPACING: f32 = 0.6;
/// Beyond this distance between centres two bikes cannot touch (m).
const REACH: f32 = 2.0 * (DISC_SPACING + DISC_RADIUS);
/// Coefficient of restitution of a bike-to-bike contact: mostly plastic.
const RESTITUTION: f32 = 0.25;
/// Friction between fairings, leathers and tyres rubbing at the contact.
const CONTACT_FRICTION: f32 = 0.35;
/// Squared radius of gyration about the vertical axis (m²), I = m·k². A
/// bike with its rider spins like a rod of about 2 m; on its wheels the tyres
/// also hold it, which acts as a much larger inertia.
const RIDING_GYRATION: f32 = 1.0;
const SLIDING_GYRATION: f32 = 0.35;
/// Sideways deceleration with which the tyres scrub off a knock (m/s²): a
/// good share of the grip of a racing tyre.
const KNOCK_GRIP: f32 = 8.0;
/// How fast the tyres and the rider stop a spin from a knock (1/s).
const KNOCK_YAW_DAMPING: f32 = 10.0;
/// What a rider can ride out (see the module docs): sideways change of
/// velocity (m/s), spin (rad/s) and deceleration from a hit on the front
/// wheel (m/s).
pub const SIDE_FALL: f32 = 3.0;
pub const YAW_FALL: f32 = 2.5;
pub const FRONT_FALL: f32 = 4.5;
/// Drag factors (fraction of g) of a bike sliding on its side and of a
/// rider sliding, on asphalt and on grass.
const BIKE_SLIDE: (f32, f32) = (0.45, 0.7);
const RIDER_SLIDE: (f32, f32) = (0.6, 0.75);
/// How fast a sliding bike stops spinning (1/s): friction on its side.
const SLIDE_SPIN_DAMPING: f32 = 1.5;
/// A fallen bike lies almost flat: handlebar and footpeg prop it (rad).
pub const DOWN_LEAN: f32 = 1.3;
/// How fast the bike drops onto its side (1/s).
const FALL_RATE: f32 = 8.0;
/// The rider's hips when seated: height above the ground and how far behind
/// the centre of the bike (m).
const SEAT_HEIGHT: f32 = 0.92;
const SEAT_BACK: f32 = 0.2;
/// How much of the bike's change of velocity reaches the rider: they are
/// only loosely held by hands, knees and pegs.
const RIDER_COUPLING: f32 = 0.4;
/// Upward speed with which the hit throws the rider (m/s) per m/s of jolt,
/// and its limits.
const RIDER_LAUNCH: f32 = 0.15;
const RIDER_LAUNCH_RANGE: (f32, f32) = (0.8, 3.0);
/// Height of the hips of a rider lying on the ground (m).
pub const LYING_HEIGHT: f32 = 0.18;
/// The rider ends up on their back, feet first (rad), and how fast they get
/// there (1/s).
pub const LIE_ANGLE: f32 = FRAC_PI_2;
const LIE_RATE: f32 = 5.0;
/// Tumble: initial roll rate (rad/s) plus this much per m/s of jolt, the
/// most, how fast the ground stops it (rad/s²) and below which rate the rider
/// settles on their back or front.
const TUMBLE: f32 = 2.0;
const TUMBLE_PER_JOLT: f32 = 0.6;
const MAX_TUMBLE: f32 = 10.0;
const TUMBLE_FRICTION: f32 = 6.0;
const SETTLE_RATE: f32 = 1.5;
/// Size of the rider against the walls (m): the hips; the helmet, from the
/// hips along the back, and its radius; and the radius of hands and feet.
const RIDER_RADIUS: f32 = 0.3;
const HELMET: Vec3 = Vec3::new(0.0, 0.62, -0.08);
const HELMET_RADIUS: f32 = 0.17;
const HAND_RADIUS: f32 = 0.1;
const FOOT_RADIUS: f32 = 0.12;
/// What bounces back off a wall (fraction of the speed into it).
const WALL_RESTITUTION: f32 = 0.2;
/// Speed into a wall that a rider rides out (m/s): about 36 km/h straight
/// into it, or a 12° hit at 180 km/h. A real rider would go down with much
/// less; this is an arcade game. The most a wall hit spins a bike (rad/s).
pub const WALL_FALL: f32 = 10.0;
const MAX_WALL_SPIN: f32 = 5.0;
/// Below this speed bike and rider count as still (m/s); they stay down at
/// least this long, rejoin after being still this long, and in any case
/// after this long (s).
const STILL_SPEED: f32 = 0.5;
const MIN_DOWN_TIME: f32 = 1.5;
const REST_TIME: f32 = 0.5;
const MAX_DOWN_TIME: f32 = 4.0;
/// Distance from the edge of the asphalt at which a bike rejoins (m), the
/// room it needs and the steps in which it looks for it to the sides (m).
const REJOIN_MARGIN: f32 = 1.0;
const REJOIN_ROOM: f32 = 2.5;
const REJOIN_STEP: f32 = 2.5;

/// Motion from contacts that the wheels do not carry: the sideways velocity
/// and the spin from a hit, while the tyres scrub them off, or all the
/// motion of a fallen bike sliding on its side.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Knock {
    /// Velocity on the ground (m/s).
    pub velocity: Vec2,
    /// Spin (rad/s), positive to the left like the heading.
    pub yaw_rate: f32,
}

impl Knock {
    fn is_idle(&self) -> bool {
        self.velocity == Vec2::ZERO && self.yaw_rate == 0.0
    }
}

/// A rider thrown off their bike, in the ground plane of the world.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThrownRider {
    /// Hips on the ground (x, z).
    pub position: Vec2,
    /// Height of the hips above the ground (m).
    pub height: f32,
    /// Height of the ground under them (m), to draw them without the track.
    pub ground: f32,
    pub velocity: Vec2,
    /// Vertical speed (m/s).
    pub climb: f32,
    /// Where the body faces, as a heading: the bike's at the moment of the fall.
    pub heading: f32,
    /// Roll about the body's long axis (rad): the lean of the bike at first,
    /// then the tumble. Positive to the left.
    pub roll: f32,
    pub roll_rate: f32,
    /// How far back the body has been laid (rad), from seated (0) to lying
    /// on the back ([`LIE_ANGLE`]).
    pub lie: f32,
    /// Where the hands and feet are, and how fast they move, in the frame of
    /// the body: they swing freely after the pose they reach for (see
    /// [`Self::pose`]).
    pub limbs: Limbs,
    pub limb_velocity: Limbs,
}

/// Where a rider down on the track has their hands and feet: offsets from
/// the hips in the frame of the body with its back straight (x to the right,
/// y along the back, -z out of the chest). The right one first.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Limbs {
    pub hands: [Vec3; 2],
    pub feet: [Vec3; 2],
}

impl Limbs {
    /// The same pose for both sides, mirrored: `hand` and `foot` are the
    /// right ones.
    fn mirrored(hand: Vec3, foot: Vec3) -> Self {
        let left = Vec3::new(-1.0, 1.0, 1.0);
        Self {
            hands: [hand, hand * left],
            feet: [foot, foot * left],
        }
    }

    fn weighted(poses: &[(f32, Limbs)]) -> Self {
        let total = poses
            .iter()
            .map(|(weight, _)| weight)
            .sum::<f32>()
            .max(1e-6);
        let sum = |part: fn(&Limbs) -> Vec3| {
            poses
                .iter()
                .map(|(weight, limbs)| part(limbs) * *weight)
                .sum::<Vec3>()
                / total
        };
        Self {
            hands: [sum(|l| l.hands[0]), sum(|l| l.hands[1])],
            feet: [sum(|l| l.feet[0]), sum(|l| l.feet[1])],
        }
    }

    fn shifted(mut self, by: Vec3) -> Self {
        for limb in self.hands.iter_mut().chain(&mut self.feet) {
            *limb += by;
        }
        self
    }
}

/// A rider's right shoulder from the hips, with the back straight.
const SHOULDER: Vec3 = Vec3::new(0.18, 0.45, 0.0);
/// Thrown, the arms go up and out and the legs apart, drifting slowly (Hz,
/// m) while the body flies.
const DRIFT_RATE: f32 = 0.8;
const DRIFT: f32 = 0.12;
/// How far the limbs trail behind while sliding (m).
const TRAIL: f32 = 0.15;
/// How often (Hz) and how far (m) a rider lying still stirs an arm or a leg.
const STIR_RATE: f32 = 0.35;
const STIR: f32 = 0.15;
/// Hands and feet hang on springs from the pose they reach for: how stiff
/// (natural frequency, Hz) and how damped (fraction of critical damping:
/// below one they swing a little past it and back, loosely, without
/// shaking).
const LIMB_FREQUENCY: f32 = 2.5;
const LIMB_DAMPING: f32 = 0.45;
/// The most of a jolt the limbs feel (m/s²) and the fastest they move (m/s):
/// a landing stops the body in an instant, but arms and legs are not thrown
/// about by it, they flop.
const MAX_FELT: f32 = 120.0;
const MAX_LIMB_SPEED: f32 = 3.5;

impl ThrownRider {
    /// Where the hands and feet reach for `elapsed` seconds into the fall,
    /// from what racers do when they crash: thrown, arms go up and out and
    /// legs apart; tumbling, they are tucked in against the body (hands out
    /// break wrists and collarbones); sliding, they are spread out like a
    /// starfish to slow down, spread the load and stop rolling, trailing
    /// behind on the asphalt; once still, they rest. Each moment blends these
    /// by how much the rider is flying, rolling and sliding, so the pose
    /// changes smoothly; the limbs swing after it (see [`Self::step`]).
    pub fn pose(&self, elapsed: f32) -> Limbs {
        let flying = crate::geometry::smoothstep(LYING_HEIGHT, LYING_HEIGHT + 0.25, self.height);
        let tumbling =
            crate::geometry::smoothstep(SETTLE_RATE, 2.0 * SETTLE_RATE + 1.0, self.roll_rate.abs());
        let sliding = crate::geometry::smoothstep(STILL_SPEED, 3.0, self.velocity.length());

        // Thrown: arms up and out, legs apart, drifting slowly.
        let drift = |side_phase: f32| {
            let phase = elapsed * DRIFT_RATE * std::f32::consts::TAU + side_phase;
            (
                SHOULDER + Vec3::new(0.3, 0.25 + DRIFT * phase.sin(), -0.15 + DRIFT * phase.cos()),
                Vec3::new(
                    0.25,
                    -0.62 + 0.5 * DRIFT * (phase + 1.0).sin(),
                    -0.15 + DRIFT * (phase + 1.0).cos(),
                ),
            )
        };
        let ((right_hand, right_foot), (left_hand, left_foot)) = (drift(0.0), drift(PI));
        let mut flail = Limbs::mirrored(right_hand, right_foot);
        let left = Vec3::new(-1.0, 1.0, 1.0);
        flail.hands[1] = left_hand * left;
        flail.feet[1] = left_foot * left;

        // Rolling over: tucked in, hands by the face and knees to the chest.
        let tuck = Limbs::mirrored(Vec3::new(0.12, 0.32, -0.25), Vec3::new(0.15, -0.32, -0.35));
        // Sliding: spread out. Still: resting.
        let spread = Limbs::mirrored(Vec3::new(0.65, 0.4, 0.05), Vec3::new(0.38, -0.78, 0.05));
        let rest = Limbs::mirrored(Vec3::new(0.32, 0.02, -0.1), Vec3::new(0.18, -0.8, 0.0));

        let on_ground = 1.0 - flying;
        let resting = on_ground * (1.0 - tumbling) * (1.0 - sliding);
        let mut pose = Limbs::weighted(&[
            (flying, flail),
            (on_ground * tumbling, tuck),
            (on_ground * (1.0 - tumbling) * sliding, spread),
            (resting, rest),
        ]);
        // Lying still, the rider stirs, slowly bending an arm or a knee.
        let limbs = pose.hands.iter_mut().chain(&mut pose.feet);
        for (i, limb) in limbs.enumerate() {
            let phase = elapsed * STIR_RATE * std::f32::consts::TAU + 1.9 * i as f32;
            *limb += Vec3::new(0.0, 0.6, -1.0) * phase.sin().max(0.0) * STIR * resting;
        }
        // The asphalt drags the limbs back against the way the body slides.
        let body = Self::body_rotation(self.heading, self.roll, self.lie);
        let slide = body.inverse() * Vec3::new(self.velocity.x, 0.0, self.velocity.y);
        let trail = -slide.normalize_or_zero() * TRAIL * on_ground * sliding;
        pose.shifted(trail)
    }

    /// Swings the hands and feet after [`Self::pose`]: each hangs on a
    /// damped spring from where it reaches for, and feels the body's jolts
    /// (`accel`, m/s² in the world, without gravity) the other way, and the
    /// tumble flinging it out. So they lag behind when the body is stopped,
    /// flop when it lands and fling out when it spins, loosely.
    fn swing(&mut self, accel: Vec3, elapsed: f32, dt: f32) {
        let target = self.pose(elapsed);
        let felt = (Self::body_rotation(self.heading, self.roll, self.lie).inverse() * accel)
            .clamp_length_max(MAX_FELT);
        let omega = std::f32::consts::TAU * LIMB_FREQUENCY;
        let spin = self.roll_rate * self.roll_rate;
        let limbs = self.limbs.hands.iter_mut().chain(&mut self.limbs.feet);
        let velocities = self
            .limb_velocity
            .hands
            .iter_mut()
            .chain(&mut self.limb_velocity.feet);
        let goals = target.hands.iter().chain(&target.feet);
        for ((limb, velocity), &goal) in limbs.zip(velocities).zip(goals) {
            // Spinning about the body's long axis (its y) flings it out.
            let fling = Vec3::new(limb.x, 0.0, limb.z) * spin;
            let pull = (goal - *limb) * omega * omega - *velocity * 2.0 * LIMB_DAMPING * omega;
            *velocity = (*velocity + (pull - felt + fling) * dt).clamp_length_max(MAX_LIMB_SPEED);
            *limb += *velocity * dt;
        }
    }

    /// Takes the body out of any wall a part of it went into. Bouncing off
    /// changes how the limbs trail, which can put one back into the wall:
    /// looks again until none is.
    fn keep_off_walls(&mut self, track: &Track) {
        for _ in 0..4 {
            let contact = wall_contact(track, self.position, &self.footprint());
            if contact.is_none() {
                break;
            }
            contain(&mut self.position, &mut self.velocity, contact);
        }
    }

    /// How the body is turned: facing `heading`, rolled about its long axis
    /// by `roll` and laid back by `lie`. Seated (`lie` 0) the roll is a lean
    /// like the bike's; laid on the back, the back points behind, the feet
    /// ahead and the chest up (down, with a half turn of roll).
    pub fn body_rotation(heading: f32, roll: f32, lie: f32) -> Quat {
        Quat::from_rotation_y(heading) * Quat::from_rotation_z(roll) * Quat::from_rotation_x(lie)
    }

    /// The body seen from above, as circles: the hips, the helmet, and the
    /// hands and feet where they are, so none of them goes through a wall.
    fn footprint(&self) -> [(Vec2, f32); 6] {
        let body = Self::body_rotation(self.heading, self.roll, self.lie);
        let at = |offset: Vec3| {
            let world = body * offset;
            self.position + Vec2::new(world.x, world.z)
        };
        let limbs = self.limbs;
        [
            (self.position, RIDER_RADIUS),
            (at(HELMET), HELMET_RADIUS),
            (at(limbs.hands[0]), HAND_RADIUS),
            (at(limbs.hands[1]), HAND_RADIUS),
            (at(limbs.feet[0]), FOOT_RADIUS),
            (at(limbs.feet[1]), FOOT_RADIUS),
        ]
    }

    pub fn is_airborne(&self) -> bool {
        self.height > LYING_HEIGHT || self.climb > 0.0
    }

    /// One step of a rider in the air, tumbling or sliding, `elapsed`
    /// seconds into the fall.
    pub fn step(&mut self, track: &Track, elapsed: f32, dt: f32) {
        let before = Vec3::new(self.velocity.x, self.climb, self.velocity.y);
        self.lie = approach(self.lie, LIE_ANGLE, LIE_RATE, dt);
        let airborne = self.is_airborne();
        if airborne {
            self.climb -= GRAVITY * dt;
            self.height += self.climb * dt;
            if self.height <= LYING_HEIGHT {
                self.height = LYING_HEIGHT;
                // A hard landing bounces a little; a soft one does not.
                self.climb = if self.climb < -3.0 {
                    -0.2 * self.climb
                } else {
                    0.0
                };
            }
        } else {
            let drag = drag_factor(track, self.position, RIDER_SLIDE);
            self.velocity = shrink(self.velocity, drag * GRAVITY * dt);
            self.roll_rate = move_toward(self.roll_rate, 0.0, TUMBLE_FRICTION * dt);
        }
        self.position += self.velocity * dt;
        self.roll += self.roll_rate * dt;
        if !airborne && self.roll_rate.abs() < SETTLE_RATE {
            // Comes to rest on the back or on the front, whichever is nearer.
            let rest = (self.roll / PI).round() * PI;
            self.roll = approach(self.roll, rest, 6.0, dt);
        }
        self.keep_off_walls(track);
        // What the body went through in this step, without the gravity that
        // pulls on the limbs as much as on it.
        let after = Vec3::new(self.velocity.x, self.climb, self.velocity.y);
        let mut accel = (after - before) / dt;
        if airborne {
            accel.y += GRAVITY;
        }
        self.swing(accel, elapsed, dt);
        self.keep_off_walls(track);
        self.ground = track.road_height(self.position);
    }
}

/// A bike whose rider has gone down. `Default` is a bike in the race.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct CrashState {
    /// Whether the rider is down.
    pub down: bool,
    /// Seconds since the fall.
    pub elapsed: f32,
    /// Seconds that bike and rider have been still.
    pub still: f32,
    /// Side the bike lies on: 1 on its left, -1 on its right (like the lean).
    pub fall_side: f32,
    pub rider: ThrownRider,
    /// The rider in the previous step, to interpolate the drawing.
    pub previous: ThrownRider,
}

impl CrashState {
    pub fn is_down(&self) -> bool {
        self.down
    }

    /// From 0 to 1, how far the rider has let go of the bike: hands off the
    /// grips and feet off the pegs.
    pub fn let_go(&self) -> f32 {
        if self.down {
            crate::geometry::smoothstep(0.0, 0.3, self.elapsed)
        } else {
            0.0
        }
    }

    /// Whether the fall is over and the bike can rejoin.
    pub fn is_over(&self) -> bool {
        self.down
            && (self.elapsed >= MAX_DOWN_TIME
                || (self.elapsed >= MIN_DOWN_TIME && self.still >= REST_TIME))
    }
}

/// One of the two bikes in a contact.
pub struct Body<'a> {
    pub state: &'a mut BikeState,
    pub knock: &'a mut Knock,
    /// Whether it is already down, sliding on its side.
    pub down: bool,
}

impl Body<'_> {
    fn velocity(&self) -> Vec2 {
        forward(self.state.heading) * self.state.speed + self.knock.velocity
    }

    fn gyration(&self) -> f32 {
        if self.down {
            SLIDING_GYRATION
        } else {
            RIDING_GYRATION
        }
    }

    /// Takes a change of velocity and spin: along the bike into its speed,
    /// the rest into the knock. A fallen bike takes it all as a knock.
    fn take(&mut self, dv: Vec2, dw: f32) {
        if self.down {
            self.knock.velocity += dv;
        } else {
            let ahead = forward(self.state.heading);
            let along = dv.dot(ahead);
            self.state.speed += along;
            self.knock.velocity += dv - ahead * along;
        }
        self.knock.yaw_rate += dw;
    }
}

/// What a contact did to one of the bikes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// Velocity before the contact (m/s).
    pub before: Vec2,
    /// Change of velocity (m/s) and of spin (rad/s).
    pub dv: Vec2,
    pub dw: f32,
    /// Whether the rider cannot ride it out and goes down.
    pub falls: bool,
}

/// A contact between two bikes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// Speed at which they were coming together (m/s).
    pub closing: f32,
    pub a: Hit,
    pub b: Hit,
}

impl Contact {
    /// From 0 to 1, how hard it was.
    pub fn severity(&self) -> f32 {
        (self.a.dv.length().max(self.b.dv.length()) / 6.0).clamp(0.0, 1.0)
    }
}

/// Separates two bikes that overlap and, if they were coming together,
/// exchanges the impulse of the hit. Returns the contact if there was one.
pub fn collide(mut a: Body, mut b: Body) -> Option<Contact> {
    let centers = a.state.position - b.state.position;
    if centers.length_squared() >= REACH * REACH {
        return None;
    }
    let (pa, pb) = closest_points(segment(a.state), segment(b.state));
    let distance = pa.distance(pb);
    if distance >= 2.0 * DISC_RADIUS {
        return None;
    }
    // From b to a, and always pulling the centres apart.
    let mut normal = (pa - pb)
        .try_normalize()
        .or_else(|| centers.try_normalize())
        .unwrap_or(Vec2::X);
    if normal.dot(centers) < 0.0 {
        normal = -normal;
    }
    let push = normal * (2.0 * DISC_RADIUS - distance) / 2.0;
    a.state.position += push;
    b.state.position -= push;

    let point = (pa + pb) / 2.0;
    let (ra, rb) = (point - a.state.position, point - b.state.position);
    let (va, vb) = (a.velocity(), b.velocity());
    let relative =
        va + spin_velocity(a.knock.yaw_rate, ra) - vb - spin_velocity(b.knock.yaw_rate, rb);
    let closing = -relative.dot(normal);
    if closing <= 0.0 {
        // Already moving apart.
        return None;
    }
    let (ga, gb) = (a.gyration(), b.gyration());
    // Change of velocity per unit of impulse, with both masses equal.
    let mass = |direction: Vec2| {
        2.0 + cross(ra, direction).powi(2) / ga + cross(rb, direction).powi(2) / gb
    };
    let normal_impulse = (1.0 + RESTITUTION) * closing / mass(normal);
    let tangent = normal.perp();
    let max_friction = CONTACT_FRICTION * normal_impulse;
    let friction = (-relative.dot(tangent) / mass(tangent)).clamp(-max_friction, max_friction);
    let impulse = normal * normal_impulse + tangent * friction;

    let hit = |body: &Body, r: Vec2, before: Vec2, dv: Vec2, dw: f32| Hit {
        before,
        dv,
        dw,
        falls: !body.down && rider_falls(body.state, r, dv, dw),
    };
    let (dwa, dwb) = (cross(ra, impulse) / ga, -cross(rb, impulse) / gb);
    let contact = Contact {
        closing,
        a: hit(&a, ra, va, impulse, dwa),
        b: hit(&b, rb, vb, -impulse, dwb),
    };
    a.take(impulse, dwa);
    b.take(-impulse, dwb);
    Some(contact)
}

/// Whether a rider on a bike in `state` goes down after a hit at `r` (from
/// the centre of the bike) that changes its velocity by `dv` and its spin by
/// `dw`.
pub fn rider_falls(state: &BikeState, r: Vec2, dv: Vec2, dw: f32) -> bool {
    let ahead = forward(state.heading);
    let sideways = dv.reject_from_normalized(ahead).length();
    let front_wheel = r.dot(ahead) > 0.3;
    sideways > SIDE_FALL || dw.abs() > YAW_FALL || (front_wheel && dv.dot(ahead) < -FRONT_FALL)
}

/// The spine of a bike's collision shape, from the rear wheel to the front.
fn segment(state: &BikeState) -> (Vec2, Vec2) {
    let ahead = forward(state.heading) * DISC_SPACING;
    (state.position - ahead, state.position + ahead)
}

/// The closest points between two segments. When they run side by side, the
/// middle of the stretch where they overlap.
fn closest_points((a0, a1): (Vec2, Vec2), (b0, b1): (Vec2, Vec2)) -> (Vec2, Vec2) {
    let (da, db, r) = (a1 - a0, b1 - b0, a0 - b0);
    let (aa, bb, ab) = (da.length_squared(), db.length_squared(), da.dot(db));
    let (c, f) = (da.dot(r), db.dot(r));
    let denominator = aa * bb - ab * ab;
    if denominator < 1e-4 * aa * bb {
        let along = |p: Vec2| ((p - a0).dot(da) / aa).clamp(0.0, 1.0);
        let pa = a0 + da * (along(b0) + along(b1)) / 2.0;
        let t = ((pa - b0).dot(db) / bb).clamp(0.0, 1.0);
        return (pa, b0 + db * t);
    }
    let s = ((ab * f - c * bb) / denominator).clamp(0.0, 1.0);
    let t = (ab * s + f) / bb;
    let (s, t) = if t < 0.0 {
        ((-c / aa).clamp(0.0, 1.0), 0.0)
    } else if t > 1.0 {
        (((ab - c) / aa).clamp(0.0, 1.0), 1.0)
    } else {
        (s, t)
    };
    (a0 + da * s, b0 + db * t)
}

/// Velocity of a point at `r` from the centre of a body spinning at
/// `yaw_rate` (positive to the left, about the vertical).
fn spin_velocity(yaw_rate: f32, r: Vec2) -> Vec2 {
    Vec2::new(r.y, -r.x) * yaw_rate
}

/// Spin that a push `force` at `r` gives, with the sign of [`spin_velocity`].
fn cross(r: Vec2, force: Vec2) -> f32 {
    r.y * force.x - r.x * force.y
}

/// One step of the knock of a bike in the race: it moves the bike and the
/// tyres scrub it off. Without a knock it does nothing.
pub fn carry(state: &mut BikeState, knock: &mut Knock, track: &Track, dt: f32) {
    if knock.is_idle() {
        return;
    }
    state.position += knock.velocity * dt;
    state.heading += knock.yaw_rate * dt;
    if state.grounded {
        knock.velocity = shrink(knock.velocity, KNOCK_GRIP * dt);
        knock.yaw_rate = approach(knock.yaw_rate, 0.0, KNOCK_YAW_DAMPING, dt);
        if knock.yaw_rate.abs() < 1e-3 {
            knock.yaw_rate = 0.0;
        }
    }
    let contact = wall_contact(track, state.position, &footprint(state));
    contain(&mut state.position, &mut knock.velocity, contact);
}

/// The rider goes down: the bike, with the velocity it has after the hit,
/// starts to slide on its side, and the rider is thrown with the velocity
/// they had before it.
pub fn fall(state: &mut BikeState, knock: &mut Knock, hit: &Hit) -> CrashState {
    let ahead = forward(state.heading);
    let left = -ahead.perp();
    knock.velocity += ahead * state.speed;
    state.speed = 0.0;
    state.steer = 0.0;
    state.steer_rate = 0.0;
    state.steer_direct = 0.0;
    state.accel = 0.0;
    state.chassis = Default::default();
    // A leaning bike goes down on the side it leans (a lowside); an upright
    // one, on the side it is pushed to.
    let pushed = hit.dv.dot(left);
    let fall_side = if state.lean.abs() > 0.15 {
        state.lean.signum()
    } else if pushed.abs() > 0.1 {
        pushed.signum()
    } else {
        1.0
    };
    let jolt = hit.dv.length();
    let mut rider = ThrownRider {
        position: state.position + left * state.lean.sin() * SEAT_HEIGHT - ahead * SEAT_BACK,
        height: state.lean.cos() * SEAT_HEIGHT,
        ground: state.elevation,
        velocity: hit.before + hit.dv * RIDER_COUPLING,
        climb: (jolt * RIDER_LAUNCH).clamp(RIDER_LAUNCH_RANGE.0, RIDER_LAUNCH_RANGE.1),
        heading: state.heading,
        roll: state.lean,
        roll_rate: fall_side * (TUMBLE + TUMBLE_PER_JOLT * jolt).min(MAX_TUMBLE),
        lie: 0.0,
        ..default()
    };
    rider.limbs = rider.pose(0.0);
    CrashState {
        down: true,
        elapsed: 0.0,
        still: 0.0,
        fall_side,
        rider,
        previous: rider,
    }
}

/// Whether hitting a wall like this brings the rider down: only going into
/// it faster than [`WALL_FALL`]. Scraping along it, or bumping into it
/// slowly, is ridden out.
pub fn wall_knocks_down(hit: &WallHit) -> bool {
    hit.into > WALL_FALL
}

/// The rider goes down against a wall: the wall stops the bike's velocity
/// into it (it bounces back a little) at the point that touched it, which
/// spins the bike if the hit was at an angle, and the rider is thrown on.
pub fn fall_against_wall(state: &mut BikeState, knock: &mut Knock, hit: &WallHit) -> CrashState {
    let dv = -hit.normal * hit.into * (1.0 + WALL_RESTITUTION);
    let dw = (cross(hit.at, dv) / SLIDING_GYRATION).clamp(-MAX_WALL_SPIN, MAX_WALL_SPIN);
    state.speed = 0.0;
    knock.velocity = hit.before + dv;
    knock.yaw_rate = dw;
    let hit = Hit {
        before: hit.before,
        dv,
        dw,
        falls: true,
    };
    fall(state, knock, &hit)
}

/// One step of a fallen bike sliding on its side and of its rider.
pub fn slide(
    state: &mut BikeState,
    knock: &mut Knock,
    crash: &mut CrashState,
    track: &Track,
    dt: f32,
) {
    crash.elapsed += dt;
    let drag = drag_factor(track, state.position, BIKE_SLIDE);
    knock.velocity = shrink(knock.velocity, drag * GRAVITY * dt);
    knock.yaw_rate = approach(knock.yaw_rate, 0.0, SLIDE_SPIN_DAMPING, dt);
    state.position += knock.velocity * dt;
    state.heading += knock.yaw_rate * dt;
    let contact = wall_contact(track, state.position, &footprint(state));
    contain(&mut state.position, &mut knock.velocity, contact);
    state.lean = approach(state.lean, crash.fall_side * DOWN_LEAN, FALL_RATE, dt);
    state.pitch = approach(state.pitch, 0.0, FALL_RATE, dt);
    state.elevation = track.road_height(state.position);
    state.climb = 0.0;
    state.grounded = true;

    crash.previous = crash.rider;
    crash.rider.step(track, crash.elapsed, dt);
    let rider = &crash.rider;
    let still = knock.velocity.length() < STILL_SPEED
        && rider.velocity.length() < STILL_SPEED
        && !rider.is_airborne();
    crash.still = if still { crash.still + dt } else { 0.0 };
}

/// Where a bike that ended up at `at` rejoins: on the asphalt next to it,
/// facing along the track, away from the bikes at `others`. Returns the
/// position, the heading and the distance to the left of the centre line.
pub fn rejoin_pose(track: &Track, at: Vec2, others: &[Vec2]) -> (Vec2, f32, f32) {
    let nearest = track.nearest(at);
    let (_, heading) = track.pose_at(nearest.index);
    let limit = track.half_width() - REJOIN_MARGIN;
    let wanted = nearest.lateral.clamp(-limit, limit);
    let place = |lateral: f32| {
        track
            .offset_point(nearest.index, lateral)
            .lerp(track.offset_point(nearest.index + 1, lateral), nearest.t)
    };
    let free = |lateral: f32| {
        others
            .iter()
            .all(|other| other.distance(place(lateral)) >= REJOIN_ROOM)
    };
    let lateral = if free(wanted) {
        wanted
    } else {
        (0..8)
            .map(|i| {
                let step = (i / 2 + 1) as f32 * REJOIN_STEP;
                if i % 2 == 0 {
                    wanted - step
                } else {
                    wanted + step
                }
            })
            .filter(|lateral| lateral.abs() <= limit)
            .find(|&lateral| free(lateral))
            .unwrap_or(wanted)
    };
    (place(lateral), heading, lateral)
}

/// Drag factor at `point` for the given (asphalt, grass) pair.
fn drag_factor(track: &Track, point: Vec2, (asphalt, grass): (f32, f32)) -> f32 {
    match track.surface_beside(&track.nearest(point)) {
        Surface::Asphalt => asphalt,
        Surface::Grass => grass,
    }
}

/// Takes something out of the wall it went `contact` into (see
/// [`wall_contact`]); what goes into the wall bounces back a little.
fn contain(position: &mut Vec2, velocity: &mut Vec2, contact: Option<(Vec2, f32)>) {
    let Some((out, depth)) = contact else {
        return;
    };
    *position -= out * depth;
    let into = velocity.dot(out);
    if into > 0.0 {
        *velocity -= out * into * (1.0 + WALL_RESTITUTION);
    }
}

/// Shortens a vector by `amount` without turning it around.
fn shrink(v: Vec2, amount: f32) -> Vec2 {
    let length = v.length();
    if length <= amount {
        Vec2::ZERO
    } else {
        v * (1.0 - amount / length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::wrap_angle;
    use crate::track::WALL_DISTANCE;

    const DT: f32 = 1.0 / 120.0;

    fn riding(position: Vec2, heading: f32, speed: f32) -> BikeState {
        BikeState {
            position,
            heading,
            speed,
            lean: 0.0,
            ..BikeState::parked_at((position, heading))
        }
    }

    fn hit(
        a: &mut BikeState,
        ka: &mut Knock,
        b: &mut BikeState,
        kb: &mut Knock,
    ) -> Option<Contact> {
        collide(
            Body {
                state: a,
                knock: ka,
                down: false,
            },
            Body {
                state: b,
                knock: kb,
                down: false,
            },
        )
    }

    fn momentum(a: &BikeState, ka: &Knock, b: &BikeState, kb: &Knock) -> Vec2 {
        forward(a.heading) * a.speed + ka.velocity + forward(b.heading) * b.speed + kb.velocity
    }

    #[test]
    fn rubbing_side_by_side_pushes_both_bikes_apart_without_a_fall() {
        // Side by side at 30 m/s, a on the left drifting 1.5 m/s into b.
        let mut a = riding(Vec2::new(-0.4, 0.0), 0.0, 30.0);
        let mut ka = Knock {
            velocity: Vec2::new(1.5, 0.0),
            ..default()
        };
        let mut b = riding(Vec2::new(0.4, 0.0), 0.0, 30.0);
        let mut kb = Knock::default();
        let before = momentum(&a, &ka, &b, &kb);
        let contact = hit(&mut a, &mut ka, &mut b, &mut kb).expect("they touch");
        assert!(!contact.a.falls && !contact.b.falls, "{contact:?}");
        assert!(kb.velocity.x > 0.0, "b is pushed right: {kb:?}");
        assert!(ka.velocity.x < 1.5, "a is held back: {ka:?}");
        // Mostly plastic: they move apart, but slowly.
        let parting = kb.velocity.x - ka.velocity.x;
        assert!((parting - RESTITUTION * 1.5).abs() < 1e-3, "{parting}");
        assert!((momentum(&a, &ka, &b, &kb) - before).length() < 1e-3);
        assert!((a.speed - 30.0).abs() < 0.5 && (b.speed - 30.0).abs() < 0.5);
    }

    #[test]
    fn a_nudge_from_behind_trades_speed_and_keeps_the_momentum() {
        let mut follower = riding(Vec2::new(0.0, 0.0), 0.0, 33.0);
        let mut leader = riding(Vec2::new(0.0, -2.0), 0.0, 30.0);
        let (mut kf, mut kl) = (Knock::default(), Knock::default());
        let contact = hit(&mut follower, &mut kf, &mut leader, &mut kl).expect("they touch");
        assert!(!contact.a.falls && !contact.b.falls, "{contact:?}");
        assert!(follower.speed < 33.0 && leader.speed > 30.0);
        assert!(follower.speed < leader.speed, "they move apart again");
        assert!((follower.speed + leader.speed - 63.0).abs() < 1e-3);
    }

    #[test]
    fn ramming_from_behind_much_faster_throws_the_follower_off() {
        let mut follower = riding(Vec2::new(0.0, 0.0), 0.0, 45.0);
        let mut leader = riding(Vec2::new(0.0, -2.0), 0.0, 30.0);
        let (mut kf, mut kl) = (Knock::default(), Knock::default());
        let contact = hit(&mut follower, &mut kf, &mut leader, &mut kl).expect("they touch");
        assert!(contact.a.falls, "the one that hits with its front wheel");
        assert!(!contact.b.falls, "the one hit from behind is pushed on");
    }

    #[test]
    fn a_t_bone_at_speed_brings_both_riders_down() {
        // b crosses in front of a, a hits it in the middle.
        let mut a = riding(Vec2::new(0.0, 0.0), 0.0, 20.0);
        let mut b = riding(Vec2::new(0.0, -1.4), FRAC_PI_2, 15.0);
        let (mut ka, mut kb) = (Knock::default(), Knock::default());
        let contact = hit(&mut a, &mut ka, &mut b, &mut kb).expect("they touch");
        assert!(contact.a.falls && contact.b.falls, "{contact:?}");
    }

    #[test]
    fn crossing_at_walking_pace_is_only_a_bump() {
        let mut a = riding(Vec2::new(0.0, 0.0), 0.0, 3.0);
        let mut b = riding(Vec2::new(0.6, -1.0), 1.0, 3.0);
        let (mut ka, mut kb) = (Knock::default(), Knock::default());
        if let Some(contact) = hit(&mut a, &mut ka, &mut b, &mut kb) {
            assert!(!contact.a.falls && !contact.b.falls, "{contact:?}");
        }
    }

    #[test]
    fn bikes_moving_apart_are_separated_without_a_hit() {
        let mut a = riding(Vec2::new(-0.3, 0.0), 0.0, 30.0);
        let mut b = riding(Vec2::new(0.3, 0.0), 0.0, 30.0);
        let (mut ka, mut kb) = (
            Knock {
                velocity: Vec2::new(-1.0, 0.0),
                ..default()
            },
            Knock::default(),
        );
        assert!(hit(&mut a, &mut ka, &mut b, &mut kb).is_none());
        assert!(a.position.distance(b.position) >= 2.0 * DISC_RADIUS - 1e-4);
        assert_eq!((a.speed, b.speed), (30.0, 30.0));
    }

    #[test]
    fn a_knock_is_scrubbed_off_in_a_moment_and_leaves_the_bike_alone() {
        let track = Track::default();
        let (position, heading) = track.grid_slot(0);
        let mut state = riding(position, heading, 30.0);
        let mut knock = Knock {
            velocity: forward(heading).perp() * 2.0,
            yaw_rate: 1.0,
        };
        for _ in 0..(1.0 / DT) as usize {
            carry(&mut state, &mut knock, &track, DT);
        }
        assert_eq!(knock, Knock::default());
        let moved = state.position.distance(position);
        assert!(moved > 0.15 && moved < 0.5, "pushed aside {moved} m");
        let turned = wrap_angle(state.heading - heading).abs();
        assert!(turned < 0.15, "turned {turned} rad");
        // Without a knock nothing changes.
        let still = state;
        carry(&mut state, &mut knock, &track, DT);
        assert_eq!(state, still);
    }

    fn crash_on_the_straight(speed: f32) -> (Track, BikeState, Knock, CrashState) {
        let track = Track::default();
        let pose = track.grid_slot(0);
        let mut state = BikeState {
            speed,
            lean: 0.0,
            ..BikeState::parked_on(&track, pose)
        };
        let mut knock = Knock::default();
        let ahead = forward(state.heading);
        // A hit on the front wheel that stops the bike by a third.
        let hit = Hit {
            before: ahead * speed,
            dv: -ahead * speed / 3.0,
            dw: 0.0,
            falls: true,
        };
        state.speed += hit.dv.dot(ahead);
        let crash = fall(&mut state, &mut knock, &hit);
        (track, state, knock, crash)
    }

    #[test]
    fn the_rider_flies_on_past_the_bike_that_was_stopped() {
        let (track, mut state, mut knock, mut crash) = crash_on_the_straight(18.0);
        let start = state.position;
        let ahead = forward(state.heading);
        for _ in 0..(1.0 / DT) as usize {
            slide(&mut state, &mut knock, &mut crash, &track, DT);
        }
        let bike = (state.position - start).dot(ahead);
        let rider = (crash.rider.position - start).dot(ahead);
        assert!(rider > bike + 2.0, "rider {rider} m, bike {bike} m");
        assert!(!crash.rider.is_airborne(), "lands within a second");
        assert!((crash.rider.lie - LIE_ANGLE).abs() < 0.05, "on their back");
        assert!(state.lean.abs() > DOWN_LEAN - 0.05, "bike on its side");
    }

    #[test]
    fn the_bike_slides_to_a_stop_with_asphalt_friction() {
        let (track, mut state, mut knock, mut crash) = crash_on_the_straight(15.0);
        let v0 = knock.velocity.length();
        for _ in 0..(1.0 / DT) as usize {
            slide(&mut state, &mut knock, &mut crash, &track, DT);
        }
        let expected = v0 - BIKE_SLIDE.0 * GRAVITY;
        assert!((knock.velocity.length() - expected).abs() < 0.1);
    }

    #[test]
    fn once_everything_is_still_the_bike_can_rejoin() {
        let (track, mut state, mut knock, mut crash) = crash_on_the_straight(15.0);
        let mut seconds = 0.0;
        while !crash.is_over() {
            slide(&mut state, &mut knock, &mut crash, &track, DT);
            seconds += DT;
            assert!(seconds <= MAX_DOWN_TIME + DT, "never ends");
        }
        assert!(seconds >= MIN_DOWN_TIME);
        let (point, heading, lateral) = rejoin_pose(&track, state.position, &[]);
        assert!(lateral.abs() <= track.half_width() - REJOIN_MARGIN + 1e-3);
        assert!(point.distance(state.position) < 5.0);
        let (_, along) = track.pose_at(track.nearest(point).index);
        assert!(wrap_angle(heading - along).abs() < 0.2);
    }

    #[test]
    fn a_bike_rejoins_beside_a_bike_in_the_way() {
        let track = Track::default();
        let (point, _) = track.grid_slot(0);
        let (spot, _, _) = rejoin_pose(&track, point, &[point]);
        assert!(spot.distance(point) >= REJOIN_ROOM);
        let nearest = track.nearest(spot);
        assert!(nearest.lateral.abs() <= track.half_width());
    }

    #[test]
    fn a_rider_down_lies_flat_on_their_back_feet_first() {
        let on_back = ThrownRider::body_rotation(0.0, 0.0, LIE_ANGLE);
        // Heading 0 goes towards -Z.
        assert!((on_back * Vec3::Y).distance(Vec3::Z) < 1e-4, "back behind");
        assert!(
            (on_back * Vec3::NEG_Y).distance(Vec3::NEG_Z) < 1e-4,
            "feet ahead"
        );
        assert!((on_back * Vec3::NEG_Z).distance(Vec3::Y) < 1e-4, "chest up");
        let on_front = ThrownRider::body_rotation(0.0, PI, LIE_ANGLE);
        assert!(
            (on_front * Vec3::NEG_Z).distance(Vec3::NEG_Y) < 1e-4,
            "chest down"
        );
        // Seated, a roll leans the rider like the bike: positive to the left.
        let leaning = ThrownRider::body_rotation(0.0, 0.5, 0.0);
        assert!((leaning * Vec3::Y).x < -0.4);
    }

    fn wall_hit(speed: f32, angle: f32) -> (Track, BikeState, Option<WallHit>) {
        // Along the straight after the finish, heading `angle` into the left wall.
        let track = Track::default();
        let (_, along) = track.pose_at(50);
        let mut state = BikeState {
            position: track.offset_point(50, WALL_DISTANCE - 0.9),
            heading: along + angle,
            speed,
            lean: 0.0,
            ..BikeState::parked_at((Vec2::ZERO, 0.0))
        };
        let mut hit = None;
        for _ in 0..60 {
            state.position += forward(state.heading) * state.speed * DT;
            hit = hit.or(super::super::hit_walls(&mut state, &track));
        }
        (track, state, hit)
    }

    #[test]
    fn scraping_or_bumping_a_wall_is_ridden_out() {
        // A shallow scrape at top speed, and straight into it at 30 km/h.
        for (speed, angle) in [(80.0, 0.1), (45.0, 0.2), (8.3, FRAC_PI_2)] {
            let (_, _, hit) = wall_hit(speed, angle);
            let hit = hit.expect("touches the wall");
            assert!(!wall_knocks_down(&hit), "{speed} m/s at {angle}: {hit:?}");
        }
    }

    #[test]
    fn hitting_a_wall_hard_brings_the_rider_down() {
        // A 20° hit at 180 km/h, and straight into it at 50 km/h.
        for (speed, angle) in [(50.0, 0.35), (14.0, FRAC_PI_2)] {
            let (_, _, hit) = wall_hit(speed, angle);
            let hit = hit.expect("touches the wall");
            assert!(wall_knocks_down(&hit), "{speed} m/s at {angle}: {hit:?}");
        }
    }

    #[test]
    fn against_a_wall_the_bike_bounces_off_and_the_rider_is_thrown_on() {
        let (track, mut state, hit) = wall_hit(40.0, 0.4);
        let hit = hit.unwrap();
        let mut knock = Knock::default();
        let mut crash = fall_against_wall(&mut state, &mut knock, &hit);
        assert!(
            knock.velocity.dot(hit.normal) < 0.0,
            "bounces off: {knock:?}"
        );
        assert!(knock.yaw_rate != 0.0, "an angled hit spins the bike");
        let along = hit.before - hit.normal * hit.before.dot(hit.normal);
        assert!(crash.rider.velocity.dot(along) > 0.9 * along.length());
        for _ in 0..(3.0 / DT) as usize {
            slide(&mut state, &mut knock, &mut crash, &track, DT);
            for &(point, radius) in &footprint(&state) {
                let into = track.distance_to_centerline(point) + radius - WALL_DISTANCE;
                assert!(into < 1e-3, "the bike goes into the wall: {into}");
            }
            for &(point, radius) in &crash.rider.footprint() {
                let into = track.distance_to_centerline(point) + radius - WALL_DISTANCE;
                assert!(into < 1e-3, "the rider goes into the wall: {into}");
            }
        }
    }

    fn lying(velocity: Vec2, roll_rate: f32) -> ThrownRider {
        ThrownRider {
            height: LYING_HEIGHT,
            velocity,
            roll_rate,
            lie: LIE_ANGLE,
            ..default()
        }
    }

    fn spread(limbs: &Limbs) -> f32 {
        limbs.hands[0].x - limbs.hands[1].x + limbs.feet[0].x - limbs.feet[1].x
    }

    /// Every hand and foot, step by step, through a whole fall at `speed`.
    fn limb_paths(speed: f32) -> Vec<[Vec3; 4]> {
        let (track, mut state, mut knock, mut crash) = crash_on_the_straight(speed);
        let mut paths = Vec::new();
        while !crash.is_over() {
            slide(&mut state, &mut knock, &mut crash, &track, DT);
            let limbs = crash.rider.limbs;
            paths.push([limbs.hands[0], limbs.hands[1], limbs.feet[0], limbs.feet[1]]);
        }
        paths
    }

    #[test]
    fn a_falling_rider_moves_arms_and_legs_loosely_without_shaking() {
        let paths = limb_paths(18.0);
        for limb in 0..4 {
            let path: Vec<Vec3> = paths.iter().map(|limbs| limbs[limb]).collect();
            let moved: f32 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
            assert!(moved > 0.5, "limb {limb} hardly moves: {moved} m");
            // Shaking is going back and forth many times a second: count how
            // often each limb turns round, along each axis.
            let steps: Vec<Vec3> = path.windows(2).map(|w| w[1] - w[0]).collect();
            let seconds = path.len() as f32 * DT;
            for axis in 0..3 {
                let turns = steps
                    .windows(2)
                    .filter(|w| w[0][axis] * w[1][axis] < -1e-10)
                    .count();
                assert!(
                    turns as f32 / seconds < 6.0,
                    "limb {limb} shakes: {turns} turns in {seconds} s"
                );
            }
            let fastest = steps.iter().map(|step| step.length()).fold(0.0, f32::max);
            assert!(fastest < 0.06, "limb {limb} jumps {fastest} m in a step");
        }
    }

    #[test]
    fn once_still_the_limbs_settle() {
        let paths = limb_paths(12.0);
        let [.., before, last] = paths.as_slice() else {
            panic!("the fall takes some steps");
        };
        for (a, b) in before.iter().zip(last) {
            assert!(a.distance(*b) < 0.005);
        }
    }

    #[test]
    fn tumbling_tucks_in_and_sliding_spreads_out() {
        let tumbling = lying(Vec2::new(0.0, -8.0), 8.0).pose(1.0);
        let sliding = lying(Vec2::new(0.0, -8.0), 0.0).pose(1.0);
        assert!(spread(&sliding) > spread(&tumbling) + 0.6);
        // Tucked: knees to the chest, hands by the face.
        for i in 0..2 {
            assert!(tumbling.feet[i].y > sliding.feet[i].y + 0.1, "knees up");
            assert!(tumbling.hands[i].length() < sliding.hands[i].length());
        }
    }

    #[test]
    fn sliding_limbs_trail_behind() {
        // On the back, feet first (heading 0 slides towards -Z): in the body,
        // the feet are ahead (-y), so trailing pulls everything towards +y.
        let sliding = lying(Vec2::new(0.0, -8.0), 0.0).pose(1.0);
        let still = lying(Vec2::new(0.0, -0.01), 0.0).pose(1.0);
        assert!(sliding.feet[0].y > still.feet[0].y + 0.1);
        assert!(sliding.hands[0].y > still.hands[0].y + 0.1);
    }

    #[test]
    fn a_rider_sliding_off_the_track_stops_at_the_wall() {
        let track = Track::default();
        let (position, heading) = track.grid_slot(0);
        let mut rider = ThrownRider {
            position,
            height: LYING_HEIGHT,
            velocity: forward(heading).perp() * 25.0,
            heading,
            ..default()
        };
        for i in 0..(3.0 / DT) as usize {
            let elapsed = i as f32 * DT;
            rider.step(&track, elapsed, DT);
            // Not the hips, the head, the hands or the feet.
            for &(point, radius) in &rider.footprint() {
                let into = track.distance_to_centerline(point) + radius - WALL_DISTANCE;
                assert!(into < 1e-3, "{into}");
            }
        }
    }
}
