//! The roadside of a desert of the American West ([`Desert`]), as in West
//! Way (Moto Racer): gas stations with their pumps under a canopy, oil
//! pumpjacks nodding on the dunes, concrete highway viaducts across the
//! road, adobe pueblos, and big diamond warning signs before the bends.
//!
//! A theme of the game's (see [`crate::world::themes`]): a circuit's file
//! writes it in its `themes` as `"desert": (...)`.
//!
//! Each building is one mesh coloured vertex by vertex, so it costs a
//! single draw; the pumpjacks share their meshes, and the signs theirs.

use std::f32::consts::{PI, TAU};

#[cfg(test)]
use std::f32::consts::FRAC_PI_2;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::geometry::{forward, random01, smoothstep};
use crate::track::{Track, WALL_DISTANCE};
use crate::world::Assets3d;
use crate::world::lettering::sign_image;
use crate::world::props::Footprint;
use crate::world::quads::MeshBuilder;
use crate::world::scenery::{ChevronDirection, Scenery, Side, Span};
use crate::world::terrain::{SEA_LEVEL, Terrain};
use crate::world::textures::generate_mipmaps;
use crate::world::themes::Theme;

/// How far the deck of a viaduct reaches to either side of the centre line
/// of the road (m), into the rock walls, how wide it is along the road and
/// how deep (m).
const VIADUCT_REACH: f32 = WALL_DISTANCE + 18.0;
const VIADUCT_WIDTH: f32 = 13.0;
const VIADUCT_DEPTH: f32 = 2.0;
/// How far a pumpjack's beam nods either way (rad), and how fast its crank
/// turns (rad/s).
const NOD: f32 = 0.28;
const CRANK_RATE: f32 = 1.4;
/// Where a pumpjack's beam pivots, and its crank turns, in its own frame
/// (m): Z towards the well, Y up.
const PIVOT: Vec3 = Vec3::new(0.0, 5.2, 0.0);
const CRANK: Vec3 = Vec3::new(0.0, 1.9, -3.4);
/// The diamond signs: their side, corner to corner (m), how high their
/// middle stands over the ground (m), and how far out from the centre line
/// of the road (m).
const SIGN_SIZE: f32 = 3.2;
const SIGN_HEIGHT: f32 = 6.2;
const SIGN_LATERAL: f32 = WALL_DISTANCE + 2.4;
/// Pixels across a diamond sign's picture.
const SIGN_PIXELS: u32 = 128;

/// Cacti stand at least this far from the centre line of any road, and at
/// most this far from that of the road they are put by (m).
const CACTUS_NEAR: f32 = WALL_DISTANCE + 4.0;
const CACTUS_FAR: f32 = 75.0;
/// How many cacti of different shapes there are.
const CACTUS_SHAPES: u32 = 3;
/// Tumbleweeds: how fast they roll (m/s), their radius (m), how high they
/// bounce (m) and every how far (m), and how high they hop over the walls
/// (m).
const TUMBLE_SPEED: f32 = 5.5;
const TUMBLE_RADIUS: f32 = 0.7;
const BOUNCE: f32 = 0.5;
const BOUNCE_EVERY: f32 = 3.5;
const WALL_HOP: f32 = 1.4;
/// How far apart the points of a tumbleweed's way are (m).
const TUMBLE_STEP: f32 = 1.0;

/// The colours of the desert's buildings (sRGB).
const CONCRETE: Color = Color::srgb(0.72, 0.7, 0.66);
const DARK_CONCRETE: Color = Color::srgb(0.55, 0.53, 0.5);
const CANOPY_YELLOW: Color = Color::srgb(0.95, 0.76, 0.12);
const CANOPY_RED: Color = Color::srgb(0.78, 0.13, 0.08);
const WHITE: Color = Color::srgb(0.92, 0.92, 0.9);
const GLASS: Color = Color::srgb(0.16, 0.22, 0.26);
const DOOR: Color = Color::srgb(0.28, 0.17, 0.1);
const IRON: Color = Color::srgb(0.16, 0.15, 0.15);
const PUMPJACK_YELLOW: Color = Color::srgb(0.85, 0.62, 0.1);
const ADOBE: [Color; 3] = [
    Color::srgb(0.8, 0.6, 0.38),
    Color::srgb(0.86, 0.68, 0.46),
    Color::srgb(0.74, 0.52, 0.33),
];
const BEAM: Color = Color::srgb(0.36, 0.24, 0.15);
const FRAME_BLUE: Color = Color::srgb(0.2, 0.42, 0.5);
const DRY_WOOD: Color = Color::srgb(0.42, 0.34, 0.27);
const SAGUARO: Color = Color::srgb(0.3, 0.46, 0.22);
const SAGUARO_RIB: Color = Color::srgb(0.22, 0.36, 0.17);
const STRAW: Color = Color::srgb(0.66, 0.53, 0.33);

/// The roadside of a desert of the American West, as in West Way (Moto
/// Racer): gas stations, oil pumpjacks nodding on the dunes, highway
/// viaducts across the road, adobe pueblos, diamond warning signs before
/// the bends, cacti, and tumbleweeds rolling across the road.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Desert {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub gas_stations: &'static [GasStation],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub pumpjacks: &'static [Pumpjack],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub overpasses: &'static [Overpass],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub pueblos: &'static [Pueblo],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub curve_signs: &'static [CurveSign],
    pub cacti: Cacti,
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub tumbleweeds: &'static [Tumbleweed],
}

/// Saguaro cacti beside the road: about `count` of them, scattered along
/// `spans` either side, on fairly level ground clear of the roads.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cacti {
    pub count: usize,
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub spans: &'static [Span],
}

/// A tumbleweed blown across the road `along` the lap, rolling and bouncing
/// from `reach` metres out on one side to as far on the other, and again.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tumbleweed {
    pub along: f32,
    pub reach: f32,
}

/// A gas station beside the road, `lateral` metres from its centre line on
/// `side`, facing it: pumps under a canopy, a shop behind them and a tall
/// sign by the road.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GasStation {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// An oil pumpjack at `position` (x, z), its beam nodding towards
/// `heading` (see [`crate::geometry`]).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pumpjack {
    pub position: Vec2,
    pub heading: f32,
}

/// A concrete highway viaduct across the road `along` the lap, the bottom
/// of its deck `height` metres over the road, from one rock wall into the
/// other on columns beyond the walls (put a [`Cutting`] on both sides
/// first).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overpass {
    pub along: f32,
    pub height: f32,
}

/// An adobe pueblo beside the road, `lateral` metres from its centre line on
/// `side`: flat-roofed houses of earth, some of two floors, with beams
/// sticking out under their roofs, ladders and a dry tree.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pueblo {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// A diamond warning sign before a bend that way, high on a post on its
/// outside, `along` the lap.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveSign {
    pub along: f32,
    pub direction: ChevronDirection,
    pub color: SignColor,
}

/// The colour of a diamond warning sign: green with a white arrow for a
/// bend, yellow with a black one for a sharp one, red with a white one for
/// a hairpin.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
pub enum SignColor {
    Green,
    Yellow,
    Red,
}

/// The desert's own systems: the pumpjacks nodding and the tumbleweeds
/// rolling.
pub struct DesertPlugin;

impl Plugin for DesertPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (pump, tumble));
    }
}

/// A pumpjack's walking beam, nodding about its pivot as its crank turns.
#[derive(Component)]
pub struct Nod {
    phase: f32,
}

/// A pumpjack's crank, turning about the axle across it.
#[derive(Component)]
pub struct Crank {
    phase: f32,
}

/// A tumbleweed rolling across the road along its way, over and over.
#[derive(Component)]
pub struct Tumbling {
    /// The ground along its way, a point every [`TUMBLE_STEP`] metres, and
    /// how much higher it hops there (over the walls).
    way: Vec<(Vec3, f32)>,
    /// The axis it rolls about, across its way.
    axis: Vec3,
    /// How far along its way it is at the start (m).
    phase: f32,
}

impl Tumbling {
    /// Where its middle is, and how far it has rolled (m), after `time`
    /// seconds: along its way at [`TUMBLE_SPEED`], bouncing, and back to the
    /// start once at the end.
    fn at(&self, time: f32) -> (Vec3, f32) {
        let length = (self.way.len() - 1) as f32 * TUMBLE_STEP;
        let rolled = (time * TUMBLE_SPEED + self.phase).rem_euclid(length);
        let k = ((rolled / TUMBLE_STEP) as usize).min(self.way.len() - 2);
        let t = rolled / TUMBLE_STEP - k as f32;
        let (a, hop_a) = self.way[k];
        let (b, hop_b) = self.way[k + 1];
        let bounce = BOUNCE * (std::f32::consts::PI * rolled / BOUNCE_EVERY).sin().abs();
        let lift = TUMBLE_RADIUS + bounce + hop_a.lerp(hop_b, t);
        (a.lerp(b, t) + Vec3::Y * lift, rolled)
    }
}

/// Rolls the tumbleweeds along their ways, shrinking them away at the ends
/// so that they do not pop out of nowhere.
pub fn tumble(time: Res<Time>, mut weeds: Query<(&mut Transform, &Tumbling)>) {
    let elapsed = time.elapsed_secs();
    for (mut transform, weed) in &mut weeds {
        let (at, rolled) = weed.at(elapsed);
        let length = (weed.way.len() - 1) as f32 * TUMBLE_STEP;
        transform.translation = at;
        transform.rotation = Quat::from_axis_angle(weed.axis, rolled / TUMBLE_RADIUS);
        transform.scale =
            Vec3::splat(smoothstep(0.0, 4.0, rolled) * smoothstep(0.0, 4.0, length - rolled));
    }
}

/// How far a pumpjack's beam is tilted (rad, its head up) when its crank
/// has turned `angle` radians: it rises and falls once a turn.
fn nod_angle(angle: f32) -> f32 {
    NOD * angle.sin()
}

/// Nods the pumpjacks' beams and turns their cranks, together.
pub fn pump(
    time: Res<Time>,
    mut beams: Query<(&mut Transform, &Nod), Without<Crank>>,
    mut cranks: Query<(&mut Transform, &Crank), Without<Nod>>,
) {
    let turned = time.elapsed_secs() * CRANK_RATE;
    for (mut transform, nod) in &mut beams {
        transform.rotation = Quat::from_rotation_x(-nod_angle(turned + nod.phase));
    }
    for (mut transform, crank) in &mut cranks {
        transform.rotation = Quat::from_rotation_x(-(turned + crank.phase));
    }
}

impl Theme for Desert {
    /// Puts the desert's works by the road, and returns the ground each one
    /// takes, so that nothing else is put there.
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        _scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        spawn_desert(world, track, terrain, self, assets)
    }
}

/// Puts the desert's works by the road (see [`Theme::spawn`]).
fn spawn_desert(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    desert: &Desert,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let painted = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.85,
        ..default()
    });
    let mut taken = Vec::new();
    for station in desert.gas_stations {
        taken.push(spawn_gas_station(
            world, track, terrain, station, &painted, assets,
        ));
    }
    spawn_pumpjacks(world, terrain, desert, &painted, assets);
    taken.extend(desert.pumpjacks.iter().map(|jack| (jack.position, 8.0)));
    for overpass in desert.overpasses {
        world.spawn((
            Mesh3d(assets.meshes.add(viaduct_mesh(track, overpass))),
            MeshMaterial3d(painted.clone()),
        ));
    }
    for pueblo in desert.pueblos {
        taken.push(spawn_pueblo(
            world, track, terrain, pueblo, &painted, assets,
        ));
    }
    spawn_curve_signs(world, track, terrain, desert.curve_signs, assets);
    taken.extend(spawn_cacti(
        world,
        track,
        terrain,
        &desert.cacti,
        &taken,
        &painted,
        assets,
    ));
    spawn_tumbleweeds(world, track, terrain, desert.tumbleweeds, &painted, assets);
    taken
}

/// Where something `lateral` metres from the centre line on `side` of the
/// point `along` the lap stands, and its frame: X towards the road, Z along
/// it, Y up.
fn beside_road(track: &Track, along: f32, lateral: f32) -> (Vec2, Quat) {
    let index = track.index_at(along);
    let spot = track.offset_point(index, lateral);
    let toward_road = (track.offset_point(index, 0.0) - spot).normalize_or(Vec2::X);
    let x = Vec3::new(toward_road.x, 0.0, toward_road.y);
    (
        spot,
        Quat::from_mat3(&Mat3::from_cols(x, Vec3::Y, x.cross(Vec3::Y))),
    )
}

/// The lowest ground under the corners of a rectangle `half` (x, z) about
/// `spot`, in the frame `rotation`.
fn lowest_ground(terrain: &Terrain, spot: Vec2, rotation: Quat, half: Vec2) -> f32 {
    [
        (-1.0, -1.0),
        (1.0, -1.0),
        (1.0, 1.0),
        (-1.0, 1.0),
        (0.0, 0.0),
    ]
    .into_iter()
    .map(|(sx, sz)| {
        let corner = rotation * Vec3::new(sx * half.x, 0.0, sz * half.y);
        terrain.height(spot + Vec2::new(corner.x, corner.z))
    })
    .fold(f32::INFINITY, f32::min)
}

/// A box from `low` to `high` (opposite corners, in its own frame), in one
/// colour: a vertical [`MeshBuilder::beam`].
fn block(mesh: &mut MeshBuilder, low: Vec3, high: Vec3, color: Color) {
    let middle = (low + high) / 2.0;
    mesh.beam(
        Vec3::new(middle.x, low.y, middle.z),
        Vec3::new(middle.x, high.y, middle.z),
        high.x - low.x,
        high.z - low.z,
        color,
    );
}

/// A flat panel on a wall facing `facing` (along X or Z), from `low` to
/// `high` (opposite corners, in the wall's plane): a window, a door.
fn panel(mesh: &mut MeshBuilder, low: Vec3, high: Vec3, facing: Vec3, color: Color) {
    let corners = if facing.x.abs() > 0.5 {
        [
            low,
            Vec3::new(low.x, low.y, high.z),
            Vec3::new(low.x, high.y, high.z),
            Vec3::new(low.x, high.y, low.z),
        ]
    } else {
        [
            low,
            Vec3::new(high.x, low.y, low.z),
            Vec3::new(high.x, high.y, low.z),
            Vec3::new(low.x, high.y, low.z),
        ]
    };
    mesh.quad(corners.map(|corner| (corner, Vec2::ZERO)), facing, color);
}

/// A gas station: two islands of pumps under a canopy with a yellow and red
/// fascia, a shop behind them, and a tall sign by the road with "GAS" on it,
/// as in West Way.
fn spawn_gas_station(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    station: &GasStation,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (spot, rotation) = beside_road(track, station.along, station.side.sign() * station.lateral);
    let base = lowest_ground(terrain, spot, rotation, Vec2::new(12.0, 11.0));
    let mut mesh = MeshBuilder::default();
    // The forecourt, a slab of concrete down into the ground.
    block(
        &mut mesh,
        Vec3::new(-11.0, -2.0, -10.0),
        Vec3::new(9.0, 0.15, 10.0),
        DARK_CONCRETE,
    );
    // The canopy on four columns, with its fascia round it.
    const CANOPY: f32 = 5.4;
    for (x, z) in [(-2.5, -6.0), (2.5, -6.0), (-2.5, 6.0), (2.5, 6.0)] {
        block(
            &mut mesh,
            Vec3::new(x - 0.25, 0.0, z - 0.25),
            Vec3::new(x + 0.25, CANOPY, z + 0.25),
            WHITE,
        );
    }
    block(
        &mut mesh,
        Vec3::new(-5.0, CANOPY, -8.5),
        Vec3::new(5.0, CANOPY + 0.35, 8.5),
        WHITE,
    );
    block(
        &mut mesh,
        Vec3::new(-5.1, CANOPY + 0.35, -8.6),
        Vec3::new(5.1, CANOPY + 1.05, 8.6),
        CANOPY_YELLOW,
    );
    block(
        &mut mesh,
        Vec3::new(-5.15, CANOPY + 0.45, -8.65),
        Vec3::new(5.15, CANOPY + 0.65, 8.65),
        CANOPY_RED,
    );
    // Two islands, each with two pumps.
    for x in [-2.5, 2.5] {
        block(
            &mut mesh,
            Vec3::new(x - 0.7, 0.0, -4.5),
            Vec3::new(x + 0.7, 0.3, 4.5),
            CONCRETE,
        );
        for z in [-2.4, 2.4] {
            block(
                &mut mesh,
                Vec3::new(x - 0.35, 0.3, z - 0.5),
                Vec3::new(x + 0.35, 1.9, z + 0.5),
                WHITE,
            );
            block(
                &mut mesh,
                Vec3::new(x - 0.38, 1.5, z - 0.53),
                Vec3::new(x + 0.38, 1.95, z + 0.53),
                CANOPY_RED,
            );
            for side in [-1.0, 1.0] {
                panel(
                    &mut mesh,
                    Vec3::new(x + side * 0.36, 0.9, z - 0.3),
                    Vec3::new(x + side * 0.36, 1.35, z + 0.3),
                    Vec3::X * side,
                    GLASS,
                );
            }
        }
    }
    // The shop behind, away from the road, with a door and a wide window
    // towards the pumps.
    block(
        &mut mesh,
        Vec3::new(-11.0, -1.0, -6.0),
        Vec3::new(-6.5, 3.8, 6.0),
        WHITE,
    );
    block(
        &mut mesh,
        Vec3::new(-11.1, 3.8, -6.1),
        Vec3::new(-6.4, 4.4, 6.1),
        CANOPY_YELLOW,
    );
    panel(
        &mut mesh,
        Vec3::new(-6.48, 0.9, -5.0),
        Vec3::new(-6.48, 2.6, 1.0),
        Vec3::X,
        GLASS,
    );
    panel(
        &mut mesh,
        Vec3::new(-6.48, 0.0, 2.2),
        Vec3::new(-6.48, 2.4, 3.6),
        Vec3::X,
        DOOR,
    );
    // The sign by the road: a tall post with a panel on top.
    const POST: f32 = 9.0;
    block(
        &mut mesh,
        Vec3::new(7.2, -1.0, 7.8),
        Vec3::new(7.8, POST, 8.4),
        WHITE,
    );
    let sign = sign_image(
        "GAS",
        CANOPY_RED,
        None,
        CANOPY_YELLOW,
        Some(CANOPY_RED),
        1.6,
    );
    let mut face = MeshBuilder::default();
    for facing in [Vec3::X, Vec3::NEG_X] {
        let x = 7.5 + facing.x * 0.31;
        // Read from the side it faces: left to right is -Z seen from +X.
        let (near, far) = if facing.x > 0.0 {
            (10.2, 6.0)
        } else {
            (6.0, 10.2)
        };
        face.quad(
            [
                (Vec3::new(x, POST, near), Vec2::new(0.0, 1.0)),
                (Vec3::new(x, POST, far), Vec2::new(1.0, 1.0)),
                (Vec3::new(x, POST + 2.6, far), Vec2::new(1.0, 0.0)),
                (Vec3::new(x, POST + 2.6, near), Vec2::ZERO),
            ],
            facing,
            Color::WHITE,
        );
    }
    block(
        &mut mesh,
        Vec3::new(7.2, POST - 0.1, 5.9),
        Vec3::new(7.8, POST + 2.7, 10.3),
        CANOPY_RED,
    );
    world
        .spawn((
            Transform::from_translation(Vec3::new(spot.x, base, spot.y)).with_rotation(rotation),
            Visibility::default(),
            Mesh3d(assets.meshes.add(mesh.build())),
            MeshMaterial3d(painted.clone()),
        ))
        .with_child((
            Mesh3d(assets.meshes.add(face.build())),
            MeshMaterial3d(assets.materials.add(StandardMaterial {
                base_color_texture: Some(assets.images.add(sign)),
                perceptual_roughness: 0.5,
                ..default()
            })),
        ));
    (spot, 15.0)
}

/// The pumpjacks: each a frame on a skid with its samson post, its walking
/// beam nodding about the top of the post, with the horse's head over the
/// well, and its crank turning with the counterweights. All share their
/// meshes.
fn spawn_pumpjacks(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    desert: &Desert,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if desert.pumpjacks.is_empty() {
        return;
    }
    let mut frame = MeshBuilder::default();
    // The skid, the samson post and the gear box.
    for x in [-0.9, 0.9] {
        frame.beam(
            Vec3::new(x, 0.15, -5.0),
            Vec3::new(x, 0.15, 3.0),
            0.35,
            0.35,
            IRON,
        );
        for z in [-1.4, 1.4] {
            frame.beam(
                Vec3::new(x * 1.2, 0.0, z),
                PIVOT + Vec3::new(x * 0.25, -0.2, 0.0),
                0.28,
                0.28,
                IRON,
            );
        }
    }
    block(
        &mut frame,
        Vec3::new(-0.6, 0.3, CRANK.z - 0.8),
        Vec3::new(0.6, CRANK.y + 0.6, CRANK.z + 0.8),
        PUMPJACK_YELLOW,
    );
    // The well head, with its polished rod up to the horse's head.
    block(
        &mut frame,
        Vec3::new(-0.35, -0.5, 3.4),
        Vec3::new(0.35, 1.1, 4.1),
        IRON,
    );
    frame.beam(
        Vec3::new(0.0, 1.1, 3.75),
        Vec3::new(0.0, 3.4, 3.75),
        0.08,
        0.08,
        WHITE,
    );
    let mut beam = MeshBuilder::default();
    beam.beam(
        Vec3::new(0.0, 0.0, -3.6),
        Vec3::new(0.0, 0.0, 3.0),
        0.5,
        0.7,
        IRON,
    );
    // The horse's head, curving down over the well.
    let head = |angle: f32| Vec3::new(0.0, 0.3 - 2.2 * angle.sin(), 3.0 + 0.8 * angle.cos());
    for k in 0..5 {
        let (a, b) = (k as f32 * 0.35 - 0.2, (k + 1) as f32 * 0.35 - 0.2);
        beam.beam(head(a), head(b), 0.6, 0.5, PUMPJACK_YELLOW);
    }
    // The pitman arms down from the back of the beam towards the crank.
    for x in [-0.55, 0.55] {
        beam.beam(
            Vec3::new(x, 0.0, -3.4),
            Vec3::new(x, -2.4, -3.4),
            0.14,
            0.14,
            IRON,
        );
    }
    let mut crank = MeshBuilder::default();
    for x in [-0.75, 0.75] {
        crank.beam(
            Vec3::new(x, 0.0, 0.0),
            Vec3::new(x, -1.4, 0.0),
            0.25,
            0.3,
            IRON,
        );
        block(
            &mut crank,
            Vec3::new(x - 0.2, -1.9, -0.6),
            Vec3::new(x + 0.2, -1.1, 0.6),
            DARK_CONCRETE,
        );
    }
    crank.beam(
        Vec3::new(-0.9, 0.0, 0.0),
        Vec3::new(0.9, 0.0, 0.0),
        0.2,
        0.2,
        IRON,
    );
    let (frame, beam, crank) = (
        assets.meshes.add(frame.build()),
        assets.meshes.add(beam.build()),
        assets.meshes.add(crank.build()),
    );
    for (k, jack) in desert.pumpjacks.iter().enumerate() {
        let base = terrain.height(jack.position);
        // Its +Z, where the well is, the way it faces.
        let rotation = Quat::from_rotation_y(jack.heading + PI);
        // Each out of step with the others.
        let phase = random01(k as u32 * 7 + 3) * TAU;
        world
            .spawn((
                Transform::from_xyz(jack.position.x, base, jack.position.y).with_rotation(rotation),
                Visibility::default(),
                Mesh3d(frame.clone()),
                MeshMaterial3d(painted.clone()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Transform::from_translation(PIVOT),
                    Visibility::default(),
                    Mesh3d(beam.clone()),
                    MeshMaterial3d(painted.clone()),
                    Nod { phase },
                ));
                parent.spawn((
                    Transform::from_translation(CRANK),
                    Visibility::default(),
                    Mesh3d(crank.clone()),
                    MeshMaterial3d(painted.clone()),
                    Crank { phase },
                ));
            });
    }
}

/// A concrete highway viaduct across the road: a deep deck from one rock
/// wall into the other, with parapets along its edges, on a pair of
/// columns just beyond each wall.
fn viaduct_mesh(track: &Track, overpass: &Overpass) -> Mesh {
    let index = track.index_at(overpass.along);
    let center = track.surface_point(index, 0.0);
    let (_, heading) = track.pose_at(index);
    let ahead = forward(heading);
    let (ahead, left) = (
        Vec3::new(ahead.x, 0.0, ahead.y),
        Vec3::new(-ahead.y, 0.0, ahead.x),
    );
    let bottom = center.y + overpass.height;
    let at = |lateral: f32, along: f32, y: f32| {
        Vec3::new(center.x, y, center.z) + left * lateral + ahead * along
    };
    let mut mesh = MeshBuilder::default();
    let half = VIADUCT_WIDTH / 2.0;
    mesh.beam(
        at(-VIADUCT_REACH, 0.0, bottom + VIADUCT_DEPTH / 2.0),
        at(VIADUCT_REACH, 0.0, bottom + VIADUCT_DEPTH / 2.0),
        VIADUCT_WIDTH,
        VIADUCT_DEPTH,
        CONCRETE,
    );
    let top = bottom + VIADUCT_DEPTH;
    for side in [-1.0, 1.0] {
        mesh.beam(
            at(-VIADUCT_REACH, side * (half - 0.25), top + 0.5),
            at(VIADUCT_REACH, side * (half - 0.25), top + 0.5),
            0.5,
            1.0,
            WHITE,
        );
    }
    // The columns, from below the road just beyond its walls.
    for lateral in [-(WALL_DISTANCE + 2.5), WALL_DISTANCE + 2.5] {
        let foot = track
            .surface_point(index, lateral.clamp(-WALL_DISTANCE, WALL_DISTANCE))
            .y
            - 2.0;
        for along in [-half + 2.0, half - 2.0] {
            mesh.beam(
                at(lateral, along, foot),
                at(lateral, along, bottom),
                1.6,
                1.6,
                DARK_CONCRETE,
            );
        }
        mesh.beam(
            at(lateral, -half + 1.0, bottom - 0.6),
            at(lateral, half - 1.0, bottom - 0.6),
            1.8,
            1.2,
            DARK_CONCRETE,
        );
    }
    mesh.build()
}

/// An adobe pueblo: houses of earth in two rows along the road, square
/// and flat-roofed with a low parapet, some with a smaller second floor set
/// back on top, a row of roof beams sticking out at the front, a door and
/// small windows framed in blue, ladders up to the roofs, and a dry tree.
fn spawn_pueblo(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    pueblo: &Pueblo,
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Footprint {
    let (spot, rotation) = beside_road(track, pueblo.along, pueblo.side.sign() * pueblo.lateral);
    let base = terrain.height(spot);
    // The ground under a point of the pueblo's own frame, from its base.
    let ground = |x: f32, z: f32| {
        let turned = rotation * Vec3::new(x, 0.0, z);
        terrain.height(spot + Vec2::new(turned.x, turned.z)) - base
    };
    let seed = (pueblo.along * 10.0) as u32;
    let r = |k: u32| random01(seed.wrapping_mul(97).wrapping_add(k));
    let mut mesh = MeshBuilder::default();
    let mut k = 0;
    // Two rows of houses along the road, the one behind taller.
    for (row, x) in [(0.0, 5.0), (1.0, -6.0)] {
        let mut z = -16.0;
        while z < 14.0 {
            k += 10;
            let width = 6.0 + 4.0 * r(k);
            let depth = 6.0 + 3.0 * r(k + 1);
            let middle = Vec2::new(x + (r(k + 4) - 0.5) * 2.0, z + width / 2.0);
            let half = Vec2::new(depth / 2.0, width / 2.0);
            let floor = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                .into_iter()
                .map(|(sx, sz)| ground(middle.x + sx * half.x, middle.y + sz * half.y))
                .fold(f32::INFINITY, f32::min);
            house(
                &mut mesh,
                Adobe {
                    middle,
                    half,
                    floor,
                    height: 3.0 + 1.2 * r(k + 2) + 0.8 * row,
                    color: ADOBE[(r(k + 3) * 3.0) as usize % 3],
                    upper: r(k + 5) < 0.5,
                    shift: r(k + 6),
                },
            );
            z += width + 1.5 + 2.0 * r(k + 7);
        }
    }
    // A dry tree at the end of the pueblo nearer the road.
    let foot = Vec3::new(10.0, ground(10.0, 15.0), 15.0);
    let fork = foot + Vec3::Y * 3.5;
    mesh.beam(foot - Vec3::Y * 0.5, fork, 0.45, 0.45, DRY_WOOD);
    for b in 0..6 {
        let angle = b as f32 / 6.0 * TAU + r(90 + b);
        let reach = 2.2 + 1.5 * r(100 + b);
        let tip = fork
            + Vec3::new(
                angle.cos() * reach,
                1.4 + 1.8 * r(110 + b),
                angle.sin() * reach,
            );
        mesh.beam(fork, tip, 0.16, 0.16, DRY_WOOD);
    }
    world.spawn((
        Transform::from_translation(Vec3::new(spot.x, base, spot.y)).with_rotation(rotation),
        Visibility::default(),
        Mesh3d(assets.meshes.add(mesh.build())),
        MeshMaterial3d(painted.clone()),
    ));
    (spot, 24.0)
}

/// One adobe house of a pueblo, in the pueblo's frame (X towards the road).
struct Adobe {
    /// Its middle (x, z), and half its depth (x) and width (z) (m).
    middle: Vec2,
    half: Vec2,
    /// The lowest ground under it, and how high its walls stand above it
    /// (m).
    floor: f32,
    height: f32,
    color: Color,
    /// Whether a smaller floor stands on top, set back, with a ladder up.
    upper: bool,
    /// Where along the front its door is (0 to 1).
    shift: f32,
}

/// Adds an adobe house to the pueblo's mesh: its walls, from under the
/// ground (which is not quite level) to the parapet, the beams sticking out
/// at the front, a door and windows framed in blue, and maybe a floor on top
/// with a ladder up to it.
fn house(mesh: &mut MeshBuilder, adobe: Adobe) {
    let Adobe {
        middle,
        half,
        floor,
        height,
        color,
        upper,
        shift,
    } = adobe;
    let at = |x: f32, y: f32, z: f32| Vec3::new(middle.x + x, floor + y, middle.y + z);
    block(
        mesh,
        at(-half.x, -1.5, -half.y),
        at(half.x, height + 0.5, half.y),
        color,
    );
    // The parapet's top, a little darker.
    block(
        mesh,
        at(-half.x - 0.05, height + 0.4, -half.y - 0.05),
        at(half.x + 0.05, height + 0.55, half.y + 0.05),
        ADOBE[2],
    );
    // The roof beams sticking out of the front wall, towards the road.
    let beams = (half.y * 2.0 / 1.1) as i32;
    for b in 0..beams {
        let z = -half.y + 0.55 + b as f32 * 1.1;
        mesh.beam(
            at(half.x - 0.3, height - 0.35, z),
            at(half.x + 0.6, height - 0.35, z),
            0.22,
            0.22,
            BEAM,
        );
    }
    // A door and two windows on the front, framed in blue.
    let front = half.x + 0.02;
    let door = (shift - 0.5) * half.y;
    let opening = |mesh: &mut MeshBuilder,
                   z: f32,
                   low: f32,
                   width: f32,
                   high: f32,
                   color: Color,
                   lift: f32| {
        panel(
            mesh,
            at(front + lift, low, z - width / 2.0),
            at(front + lift, high, z + width / 2.0),
            Vec3::X,
            color,
        );
    };
    opening(mesh, door, 0.0, 1.2, 2.2, FRAME_BLUE, 0.0);
    opening(mesh, door, 0.0, 0.9, 2.05, DOOR, 0.01);
    for side in [-1.0, 1.0] {
        let z = side * half.y * 0.6;
        if (z - door).abs() < 1.6 {
            continue;
        }
        opening(mesh, z, 1.1, 1.0, 2.1, FRAME_BLUE, 0.0);
        opening(mesh, z, 1.2, 0.75, 2.0, GLASS, 0.01);
    }
    if !upper {
        return;
    }
    // A smaller floor on top, set back from the front, and a ladder up to
    // it leaning on the front wall.
    let top = Vec2::new(half.x * 0.55, half.y * 0.6);
    let (back, along) = (-half.x * 0.35, (shift - 0.5) * half.y * 0.5);
    block(
        mesh,
        at(back - top.x, height, along - top.y),
        at(back + top.x, height * 1.8 + 0.4, along + top.y),
        color,
    );
    let ladder = -half.y * 0.6;
    for dz in [-0.3, 0.3] {
        mesh.beam(
            at(half.x + 1.2, 0.0, ladder + dz),
            at(half.x + 0.1, height + 1.2, ladder + dz),
            0.1,
            0.1,
            DRY_WOOD,
        );
    }
    let rungs = (height / 0.45) as i32;
    for k in 1..rungs {
        let t = k as f32 / rungs as f32;
        let x = half.x + 1.2 - 1.1 * t;
        let y = t * (height + 1.2);
        mesh.beam(
            at(x, y, ladder - 0.3),
            at(x, y, ladder + 0.3),
            0.06,
            0.06,
            DRY_WOOD,
        );
    }
}

/// A prism of `sides` faces about the line from `a` to `b`, `radius` out,
/// its faces alternately in `color` and `rib` (the ribs of a cactus), capped
/// at `b` with a low point.
fn ribbed(
    mesh: &mut MeshBuilder,
    a: Vec3,
    b: Vec3,
    radius: f32,
    sides: usize,
    color: Color,
    rib: Color,
) {
    let along = (b - a).normalize_or(Vec3::Y);
    let side = if along.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let u = along.cross(side).normalize();
    let v = along.cross(u);
    let ring = |k: usize| {
        let angle = k as f32 / sides as f32 * TAU;
        (u * angle.cos() + v * angle.sin()) * radius
    };
    let tip = b + along * radius * 0.6;
    for k in 0..sides {
        let (p, q) = (ring(k), ring(k + 1));
        let out = (p + q).normalize_or_zero();
        let tone = if k % 2 == 0 { color } else { rib };
        mesh.quad(
            [
                (a + p, Vec2::ZERO),
                (b + p, Vec2::ZERO),
                (b + q, Vec2::ZERO),
                (a + q, Vec2::ZERO),
            ],
            out,
            tone,
        );
        mesh.triangle(
            [(b + p, Vec2::ZERO), (b + q, Vec2::ZERO), (tip, Vec2::ZERO)],
            out + along,
            tone,
        );
    }
}

/// A saguaro of shape `shape`: a tall ribbed trunk with arms that bend up.
fn cactus_mesh(shape: u32) -> Mesh {
    let mut mesh = MeshBuilder::default();
    let r = |k: u32| random01(shape * 31 + k);
    let height = 4.0 + 2.5 * r(0);
    ribbed(
        &mut mesh,
        Vec3::Y * -0.5,
        Vec3::Y * height,
        0.38,
        10,
        SAGUARO,
        SAGUARO_RIB,
    );
    let arms = shape + 1;
    for arm in 0..arms {
        let angle = arm as f32 / arms as f32 * TAU + r(10 + arm) * 1.2;
        let out = Vec3::new(angle.cos(), 0.0, angle.sin());
        let from = 1.4 + (height - 2.8) * r(20 + arm);
        let reach = 0.8 + 0.5 * r(30 + arm);
        let up = 1.0 + 1.6 * r(40 + arm);
        let elbow = Vec3::Y * from + out * reach;
        ribbed(
            &mut mesh,
            Vec3::Y * from,
            elbow,
            0.24,
            8,
            SAGUARO,
            SAGUARO_RIB,
        );
        ribbed(
            &mut mesh,
            elbow - Vec3::Y * 0.2,
            elbow + Vec3::Y * up,
            0.24,
            8,
            SAGUARO,
            SAGUARO_RIB,
        );
    }
    mesh.build()
}

/// Where the cacti stand: the point, which way each is turned, how big it is
/// and its shape. Along the spans, either side, scattered out from the
/// road, on fairly level dry ground clear of every road and of the ground
/// `taken`.
fn cactus_spots(
    track: &Track,
    terrain: &Terrain,
    cacti: &Cacti,
    taken: &[Footprint],
) -> Vec<(Vec2, f32, f32, u32)> {
    if cacti.spans.is_empty() {
        return Vec::new();
    }
    let lap = track.lap_length();
    let mut spots: Vec<(Vec2, f32, f32, u32)> = Vec::new();
    for k in 0..(cacti.count as u32 * 3) {
        if spots.len() >= cacti.count {
            break;
        }
        let r = |n: u32| random01(k * 13 + n + 7_919);
        let span = &cacti.spans[k as usize % cacti.spans.len()];
        let along = span.from + span.length(lap) * r(0);
        let side = if r(1) < 0.5 { 1.0 } else { -1.0 };
        let lateral = CACTUS_NEAR.lerp(CACTUS_FAR, r(2) * r(2));
        let point = track.offset_point(track.index_at(along), side * lateral);
        let ground = terrain.height(point);
        let level = [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y]
            .into_iter()
            .all(|out| (terrain.height(point + out * 1.5) - ground).abs() < 0.8);
        let clear = track.nearest(point).distance > CACTUS_NEAR
            && taken
                .iter()
                .all(|&(at, reach)| at.distance(point) > reach + 3.0)
            && spots.iter().all(|&(at, ..)| at.distance(point) > 6.0);
        if level && clear && ground > SEA_LEVEL + 2.0 {
            spots.push((point, r(3) * TAU, 0.8 + 0.45 * r(4), k % CACTUS_SHAPES));
        }
    }
    spots
}

/// The cacti, sharing the meshes of their shapes. Returns the ground they
/// take.
fn spawn_cacti(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    cacti: &Cacti,
    taken: &[Footprint],
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let spots = cactus_spots(track, terrain, cacti, taken);
    let shapes: Vec<Handle<Mesh>> = (0..CACTUS_SHAPES)
        .map(|shape| assets.meshes.add(cactus_mesh(shape)))
        .collect();
    for &(point, turn, size, shape) in &spots {
        world.spawn((
            Transform::from_xyz(point.x, terrain.height(point), point.y)
                .with_rotation(Quat::from_rotation_y(turn))
                .with_scale(Vec3::splat(size)),
            Visibility::default(),
            Mesh3d(shapes[shape as usize].clone()),
            MeshMaterial3d(painted.clone()),
        ));
    }
    spots.iter().map(|&(point, ..)| (point, 2.0)).collect()
}

/// A tumbleweed: a ball of dry twigs tangled together.
fn tumbleweed_mesh() -> Mesh {
    let mut mesh = MeshBuilder::default();
    let on_ball = |k: u32| {
        let (a, b) = (random01(k) * TAU, (2.0 * random01(k + 1) - 1.0).acos());
        Vec3::new(b.sin() * a.cos(), b.cos(), b.sin() * a.sin()) * TUMBLE_RADIUS
    };
    for k in 0..70 {
        mesh.beam(on_ball(k * 4), on_ball(k * 4 + 2), 0.035, 0.035, STRAW);
    }
    mesh.build()
}

/// The way of a tumbleweed across the road: the ground every
/// [`TUMBLE_STEP`] metres from `reach` out on one side to as far on the
/// other, slanting a little with the wind, and how much higher it hops
/// there, over the walls.
fn tumble_way(
    track: &Track,
    terrain: &Terrain,
    weed: &Tumbleweed,
    from_left: bool,
) -> Vec<(Vec3, f32)> {
    let index = track.index_at(weed.along);
    let (_, heading) = track.pose_at(index);
    let ahead = forward(heading);
    let left = -ahead.perp();
    let across = if from_left { -left } else { left };
    let way = (across + ahead * 0.25).normalize();
    let start = track.offset_point(index, 0.0) - way * weed.reach;
    let steps = (2.0 * weed.reach / TUMBLE_STEP) as usize;
    (0..=steps)
        .map(|k| {
            let point = start + way * (k as f32 * TUMBLE_STEP);
            let nearest = track.nearest(point);
            // On the road, on its asphalt.
            let ground = if nearest.distance < track.half_width() {
                terrain.height(point).max(track.road_height(point))
            } else {
                terrain.height(point)
            };
            let wall = track.edge_beside(&nearest);
            let hop = WALL_HOP * (1.0 - ((nearest.distance - wall).abs() / 2.0).min(1.0));
            (Vec3::new(point.x, ground, point.y), hop)
        })
        .collect()
}

/// The tumbleweeds, sharing one mesh, each blown across the road its way.
fn spawn_tumbleweeds(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    weeds: &[Tumbleweed],
    painted: &Handle<StandardMaterial>,
    assets: &mut Assets3d,
) {
    if weeds.is_empty() {
        return;
    }
    let mesh = assets.meshes.add(tumbleweed_mesh());
    for (k, weed) in weeds.iter().enumerate() {
        let way = tumble_way(track, terrain, weed, k % 2 == 0);
        let first = way[0].0;
        let heading = (way[way.len() - 1].0 - first).normalize_or(Vec3::X);
        world.spawn((
            Transform::from_translation(first),
            Visibility::default(),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(painted.clone()),
            Tumbling {
                axis: Vec3::Y.cross(heading).normalize_or(Vec3::X),
                phase: random01(k as u32 * 17 + 5) * 2.0 * weed.reach,
                way,
            },
        ));
    }
}

/// The diamond warning signs, high on their posts on the outside of each
/// bend, turned a little towards the road.
fn spawn_curve_signs(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    signs: &[CurveSign],
    assets: &mut Assets3d,
) {
    if signs.is_empty() {
        return;
    }
    let mut face = MeshBuilder::default();
    let half = SIGN_SIZE / 2.0;
    face.quad(
        [
            (
                Vec3::new(-half, SIGN_HEIGHT - half, 0.0),
                Vec2::new(0.0, 1.0),
            ),
            (
                Vec3::new(half, SIGN_HEIGHT - half, 0.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(half, SIGN_HEIGHT + half, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (Vec3::new(-half, SIGN_HEIGHT + half, 0.0), Vec2::ZERO),
        ],
        Vec3::Z,
        Color::WHITE,
    );
    let face = assets.meshes.add(face.build());
    let mut post = MeshBuilder::default();
    post.beam(
        Vec3::new(0.0, -0.8, -0.12),
        Vec3::new(0.0, SIGN_HEIGHT, -0.12),
        0.2,
        0.2,
        Color::srgb(0.5, 0.52, 0.55),
    );
    let post = assets.meshes.add(post.build());
    let metal = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.6,
        ..default()
    });
    let mut faces: Vec<((SignColor, ChevronDirection), Handle<StandardMaterial>)> = Vec::new();
    for sign in signs {
        let key = (sign.color, sign.direction);
        let material = match faces.iter().find(|(k, _)| *k == key) {
            Some((_, material)) => material.clone(),
            None => {
                let image = assets.images.add(diamond_image(sign.color, sign.direction));
                let material = assets.materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    alpha_mode: AlphaMode::Mask(0.5),
                    perceptual_roughness: 0.5,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                });
                faces.push((key, material.clone()));
                material
            }
        };
        // On the outside of the bend.
        let side = match sign.direction {
            ChevronDirection::Left => -1.0,
            ChevronDirection::Right => 1.0,
        };
        let index = track.index_at(sign.along);
        let point = track.offset_point(index, side * SIGN_LATERAL);
        let base = terrain.height(point);
        let (_, heading) = track.pose_at(index);
        world.spawn((
            Transform::from_xyz(point.x, base, point.y)
                .with_rotation(Quat::from_rotation_y(heading + side * 0.35)),
            Visibility::default(),
            children![
                (Mesh3d(face.clone()), MeshMaterial3d(material)),
                (Mesh3d(post.clone()), MeshMaterial3d(metal.clone())),
            ],
        ));
    }
}

/// The picture on a diamond warning sign: a diamond in its colour with a
/// pale border, and an arrow going up and bending the way of the bend; the
/// corners outside the diamond are clear.
fn diamond_image(color: SignColor, direction: ChevronDirection) -> Image {
    let (paper, ink) = match color {
        SignColor::Green => ([22, 120, 52], [245, 245, 240]),
        SignColor::Yellow => ([248, 200, 20], [20, 20, 22]),
        SignColor::Red => ([214, 60, 24], [245, 245, 240]),
    };
    let mirror = match direction {
        ChevronDirection::Left => -1.0,
        ChevronDirection::Right => 1.0,
    };
    let size = SIGN_PIXELS;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            // Up is +v, the way of the bend +u.
            let u = mirror * ((x as f32 + 0.5) / size as f32 * 2.0 - 1.0);
            let v = 1.0 - (y as f32 + 0.5) / size as f32 * 2.0;
            let pixel = match diamond_pixel(u, v) {
                Mark::Clear => [0, 0, 0, 0],
                Mark::Border | Mark::Arrow => [ink[0], ink[1], ink[2], 255],
                Mark::Paper => [paper[0], paper[1], paper[2], 255],
            };
            data.extend(pixel);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    generate_mipmaps(&mut image);
    image
}

/// What is at a point of a diamond sign's picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mark {
    Clear,
    Border,
    Paper,
    Arrow,
}

/// What is at (`u`, `v`) of a diamond sign's picture, both from -1 to 1, up
/// +v, with its arrow bending towards +u.
fn diamond_pixel(u: f32, v: f32) -> Mark {
    let diamond = u.abs() + v.abs();
    if diamond > 0.97 {
        return Mark::Clear;
    }
    if diamond > 0.86 {
        return Mark::Border;
    }
    const THICK: f32 = 0.075;
    // The shaft, up from below the middle.
    let shaft = (u + 0.1).abs() < THICK && (-0.5..0.06).contains(&v);
    // The bend: a quarter of a ring from the top of the shaft to the right.
    let to_bend = Vec2::new(u - 0.1, v - 0.06);
    let bend = (to_bend.length() - 0.2).abs() < THICK && u < 0.1 && v > 0.06;
    // Then right to the head.
    let across = (v - 0.26).abs() < THICK && (0.1..0.3).contains(&u);
    // The head, a triangle pointing right.
    let head = (0.26..0.5).contains(&u) && (v - 0.26).abs() < 0.22 * (0.5 - u) / 0.24;
    if shaft || bend || across || head {
        Mark::Arrow
    } else {
        Mark::Paper
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    /// The circuit with a desert, its track and its terrain.
    fn desert_circuit() -> (&'static Scenery, &'static Desert, Track, Terrain) {
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.scenery.themes.get::<Desert>().is_some())
            .expect("a circuit runs through a desert");
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        let desert = circuit.scenery.themes.get::<Desert>().unwrap();
        (&circuit.scenery, desert, track, terrain)
    }

    #[test]
    fn a_pumpjack_s_beam_rises_and_falls_once_a_turn_of_its_crank() {
        assert_eq!(nod_angle(0.0), 0.0);
        assert!((nod_angle(FRAC_PI_2) - NOD).abs() < 1e-6);
        assert!((nod_angle(3.0 * FRAC_PI_2) + NOD).abs() < 1e-6);
        assert!((nod_angle(1.0) - nod_angle(1.0 + TAU)).abs() < 1e-5);
    }

    #[test]
    fn diamond_signs_point_the_way_of_the_bend() {
        // The corners are clear, the middle of the diamond is the sign's
        // colour, and the arrow's head is on the side of the bend.
        assert_eq!(diamond_pixel(0.95, 0.95), Mark::Clear);
        assert_eq!(diamond_pixel(0.0, 0.91), Mark::Border);
        assert_eq!(diamond_pixel(0.0, -0.7), Mark::Paper);
        assert_eq!(diamond_pixel(-0.1, -0.3), Mark::Arrow);
        assert_eq!(diamond_pixel(0.38, 0.26), Mark::Arrow);
        assert_eq!(diamond_pixel(-0.38, 0.26), Mark::Paper);
        for direction in [ChevronDirection::Left, ChevronDirection::Right] {
            let image = diamond_image(SignColor::Green, direction);
            let data = image.data.as_ref().unwrap();
            let size = SIGN_PIXELS as usize;
            let pixel = |x: usize, y: usize| &data[(y * size + x) * 4..(y * size + x) * 4 + 4];
            assert_eq!(pixel(2, 2)[3], 0, "a clear corner");
            // The head: right of the middle for a bend to the right.
            let (right, left) = (size * 69 / 100, size * 31 / 100);
            let row = size * 37 / 100;
            let white = |p: &[u8]| p[0] > 200 && p[1] > 200;
            match direction {
                ChevronDirection::Right => {
                    assert!(white(pixel(right, row)) && !white(pixel(left, row)))
                }
                ChevronDirection::Left => {
                    assert!(white(pixel(left, row)) && !white(pixel(right, row)))
                }
            }
        }
    }

    #[test]
    fn the_desert_s_works_stand_clear_of_the_roads_on_the_ground() {
        let (_, desert, track, terrain) = desert_circuit();
        assert!(!desert.gas_stations.is_empty() && !desert.pumpjacks.is_empty());
        assert!(!desert.pueblos.is_empty() && !desert.curve_signs.is_empty());
        // Gas stations and pueblos: none of what they take reaches a road's
        // walls.
        for (along, side, lateral, reach) in desert
            .gas_stations
            .iter()
            .map(|s| (s.along, s.side, s.lateral, 12.5))
            .chain(
                desert
                    .pueblos
                    .iter()
                    .map(|p| (p.along, p.side, p.lateral, 18.0)),
            )
        {
            let (spot, _) = beside_road(&track, along, side.sign() * lateral);
            let nearest = track.nearest(spot);
            assert!(
                nearest.distance - reach > WALL_DISTANCE + 1.0,
                "at {along} m: {} m from a road",
                nearest.distance
            );
        }
        // The pumpjacks stand on fairly level ground, away from the roads.
        for jack in desert.pumpjacks {
            assert!(track.nearest(jack.position).distance > WALL_DISTANCE + 15.0);
            let heights: Vec<f32> = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
                .into_iter()
                .map(|out| terrain.height(jack.position + out * 4.0))
                .collect();
            let spread = heights.iter().copied().fold(f32::MIN, f32::max)
                - heights.iter().copied().fold(f32::MAX, f32::min);
            assert!(spread < 2.5, "{:?}: {spread} m across", jack.position);
        }
    }

    #[test]
    fn viaducts_run_high_over_the_road_into_rock_on_both_sides() {
        let (_, desert, track, terrain) = desert_circuit();
        assert!(!desert.overpasses.is_empty());
        for overpass in desert.overpasses {
            let index = track.index_at(overpass.along);
            let road = track.surface_point(index, 0.0).y;
            assert!(overpass.height > 7.0, "room under it for a bike flying");
            // At the ends of its deck, inside rock higher than its top.
            let top = road + overpass.height + VIADUCT_DEPTH;
            for side in [-1.0, 1.0] {
                let end = track.offset_point(index, side * (VIADUCT_REACH - 2.0));
                assert!(
                    terrain.height(end) > top,
                    "at {} m: rock {} under a deck at {top}",
                    overpass.along,
                    terrain.height(end)
                );
            }
        }
    }

    #[test]
    fn cacti_stand_on_level_dry_ground_clear_of_the_roads() {
        let (_, desert, track, terrain) = desert_circuit();
        let spots = cactus_spots(&track, &terrain, &desert.cacti, &[]);
        assert!(
            spots.len() * 2 >= desert.cacti.count,
            "{} cacti",
            spots.len()
        );
        for &(point, _, size, shape) in &spots {
            assert!(track.nearest(point).distance > CACTUS_NEAR, "{point:?}");
            assert!(terrain.height(point) > SEA_LEVEL + 2.0);
            assert!(shape < CACTUS_SHAPES && (0.8..1.3).contains(&size));
        }
    }

    #[test]
    fn tumbleweeds_roll_across_the_road_and_hop_over_its_walls() {
        let (_, desert, track, terrain) = desert_circuit();
        assert!(!desert.tumbleweeds.is_empty());
        for (k, weed) in desert.tumbleweeds.iter().enumerate() {
            let way = tumble_way(&track, &terrain, weed, k % 2 == 0);
            // From one side of the road to the other, over its middle.
            let side = |point: Vec3| track.nearest(Vec2::new(point.x, point.z)).lateral.signum();
            assert_eq!(
                side(way[0].0),
                -side(way[way.len() - 1].0),
                "at {} m",
                weed.along
            );
            let flat = |point: Vec3| Vec2::new(point.x, point.z);
            assert!(
                way.iter()
                    .any(|&(p, _)| track.distance_to_centerline(flat(p)) < 1.5)
            );
            // Over the walls it is up off the ground.
            let rolling = Tumbling {
                way: way.clone(),
                axis: Vec3::X,
                phase: 0.0,
            };
            let length = (way.len() - 1) as f32 * TUMBLE_STEP;
            for step in 0..(length / TUMBLE_SPEED / 0.02) as usize {
                let (at, _) = rolling.at(step as f32 * 0.02);
                let nearest = track.nearest(flat(at));
                let ground = terrain.height(flat(at));
                assert!(at.y - TUMBLE_RADIUS > ground - 0.3, "at {} m", weed.along);
                if (nearest.distance - track.edge_beside(&nearest)).abs() < 0.3 {
                    assert!(at.y - TUMBLE_RADIUS > ground + 1.0, "at {} m", weed.along);
                }
            }
        }
    }

    #[test]
    fn curve_signs_stand_before_bends_their_way() {
        let (_, desert, track, _) = desert_circuit();
        for sign in desert.curve_signs {
            // Within the next 150 m the road bends the way the arrow says:
            // the first sharp bend ahead.
            let index = track.index_at(sign.along);
            let ahead = (0..75)
                .map(|k| {
                    let curvature = track.curvature(index + k);
                    track.smoothed_curvature(index + k, 3).copysign(curvature)
                })
                .find(|curvature| curvature.abs() > 1.0 / 120.0)
                .unwrap_or(0.0);
            let way = match sign.direction {
                ChevronDirection::Left => 1.0,
                ChevronDirection::Right => -1.0,
            };
            assert!(ahead * way > 0.0, "at {} m: {ahead}", sign.along);
        }
    }
}
