//! A theme of the game's, `"neon"`: the props of Puerto Neón, a harbour
//! city of the future by night, made with Meshy (`assets/models/neon/`,
//! prepared by `tools/import_neon.py`): the gate over the finish line, the
//! towers, the screens on their masts and the grandstand. A circuit's file
//! says where each stands (`props`).
//!
//! They are lit of their own, a little, in their own colours, so that their
//! windows and strips of light show at night.
//!
//! The city round the lap is a pack of its own (`assets/models/neon_city/`,
//! prepared by `tools/import_city.py`): its blocks and towers, its industry,
//! what stands in the sea and its landmarks (a stadium, an observation
//! wheel, hologram rings, sign towers), with what shines of them at night
//! (their lit windows, lamps and signs: the pack's `glow.jpg`). The wheel
//! and the wind turbines turn ([`NeonPlugin`]).

use bevy::prelude::*;

use motorust_engine::track::Track;
use motorust_engine::world::Assets3d;
use motorust_engine::world::prop_pack::spot;
use motorust_engine::world::props::Footprint;
use motorust_engine::world::scenery::{Prop, Scenery, Spot};
use motorust_engine::world::terrain::{SEA_LEVEL, Terrain};
use motorust_engine::world::themes::Theme;

/// The packs of its props and of its city, in `assets/models/`.
const PACK: &str = "neon";
const CITY_PACK: &str = "neon_city";
/// How bright what shines of the city's pack shines, over its glow.
const CITY_GLOW: f32 = 3.0;
/// How bright the props are lit of their own, over their colours.
const GLOW: f32 = 0.9;
/// How far under the ground a prop's foot goes (m): none of it floats where
/// the ground slopes.
const SUNK: f32 = 0.3;
/// How deep the ship lies in the water, and how high the airships fly (m).
const DRAUGHT: f32 = 2.5;
/// How high over the water what stands in it has its foot (m).
const AFLOAT: f32 = 0.8;
/// How far under the road beside it a platform's top is (m).
const BELOW_ROAD: f32 = 4.0;
/// The pylon under what stands by a skyway: its colour, and how much
/// narrower than what it carries it is.
const PYLON: Color = Color::srgb(0.12, 0.13, 0.16);
const PYLON_SHARE: f32 = 0.45;
const AIRSHIP_HEIGHT: f32 = 110.0;

/// One of the theme's props, by its name in the pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum NeonModel {
    StartGate,
    TowerSpire,
    TowerStepped,
    TowerTwin,
    ScreenMast,
    Grandstand,
    HarbourCrane,
    ControlTower,
    FusionPlant,
    /// Afloat: its foot a little under the water.
    ContainerShip,
    /// In the air, high over where it is put.
    AirshipCobalt,
    AirshipNeon,
    /// The landmarks of the city's pack: a stadium, an observation wheel
    /// (it turns), a hologram ring on its mast and a sign tower.
    Stadium,
    FerrisWheel,
    HologramRing,
    SignTower,
    /// In the sea: an offshore wind turbine (its rotor turns) and an oil
    /// platform.
    WindTurbine,
    OilPlatform,
}

impl NeonModel {
    /// Every one of them, for the tests.
    #[cfg(test)]
    pub const ALL: [NeonModel; 18] = [
        NeonModel::StartGate,
        NeonModel::TowerSpire,
        NeonModel::TowerStepped,
        NeonModel::TowerTwin,
        NeonModel::ScreenMast,
        NeonModel::Grandstand,
        NeonModel::HarbourCrane,
        NeonModel::ControlTower,
        NeonModel::FusionPlant,
        NeonModel::ContainerShip,
        NeonModel::AirshipCobalt,
        NeonModel::AirshipNeon,
        NeonModel::Stadium,
        NeonModel::FerrisWheel,
        NeonModel::HologramRing,
        NeonModel::SignTower,
        NeonModel::WindTurbine,
        NeonModel::OilPlatform,
    ];

    /// Its piece's name in the pack.
    pub fn name(self) -> &'static str {
        match self {
            NeonModel::StartGate => "start_gate",
            NeonModel::TowerSpire => "tower_spire",
            NeonModel::TowerStepped => "tower_stepped",
            NeonModel::TowerTwin => "tower_twin",
            NeonModel::ScreenMast => "screen_mast",
            NeonModel::Grandstand => "grandstand",
            NeonModel::HarbourCrane => "harbour_crane",
            NeonModel::ControlTower => "control_tower",
            NeonModel::FusionPlant => "fusion_plant",
            NeonModel::ContainerShip => "container_ship",
            NeonModel::AirshipCobalt => "airship_cobalt",
            NeonModel::AirshipNeon => "airship_neon",
            NeonModel::Stadium => "stadium",
            NeonModel::FerrisWheel => "ferris_wheel",
            NeonModel::HologramRing => "hologram_ring",
            NeonModel::SignTower => "sign_tower_a",
            NeonModel::WindTurbine => "wind_turbine",
            NeonModel::OilPlatform => "oil_platform",
        }
    }

    /// How far round where it stands the city keeps clear of it (m).
    pub fn clearance(self) -> f32 {
        match self {
            NeonModel::Stadium => 110.0,
            NeonModel::ContainerShip => 100.0,
            NeonModel::FerrisWheel => 55.0,
            NeonModel::OilPlatform => 45.0,
            NeonModel::FusionPlant | NeonModel::HarbourCrane => 40.0,
            NeonModel::TowerSpire
            | NeonModel::TowerStepped
            | NeonModel::TowerTwin
            | NeonModel::WindTurbine => 35.0,
            NeonModel::ControlTower => 25.0,
            NeonModel::AirshipCobalt | NeonModel::AirshipNeon => 0.0,
            _ => 15.0,
        }
    }

    /// The pack it is in.
    pub fn pack(self) -> &'static str {
        match self {
            NeonModel::Stadium
            | NeonModel::FerrisWheel
            | NeonModel::HologramRing
            | NeonModel::SignTower
            | NeonModel::WindTurbine
            | NeonModel::OilPlatform => CITY_PACK,
            _ => PACK,
        }
    }
}

/// The props of a harbour city of the future, as a circuit's file places
/// them.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Neon {
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub props: &'static [Prop<NeonModel>],
    /// The city round the lap, if it runs through one (see [`City`]).
    #[serde(default)]
    pub city: Option<City>,
}

impl Neon {
    /// Where its props stand and how far round each the city keeps clear.
    pub fn landmarks(&self, track: &Track) -> Vec<(Vec2, f32)> {
        self.props
            .iter()
            .map(|prop| (spot(track, prop).0, prop.model.clearance() * prop.scale))
            .collect()
    }
}

/// A city the lap runs through: the city pack's buildings, in `rows` rows
/// either side of the lap, one every `every` metres along it, from `near`
/// to `far` metres from its middle, standing on the ground or in the
/// water; none where another stretch of the lap passes.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct City {
    pub every: f32,
    pub near: f32,
    pub far: f32,
    pub rows: usize,
}

/// A building of a city: where it stands, how it is turned, which of the
/// city pack's pieces it is and how much bigger or smaller than as made,
/// and how wide, tall and deep it then is (m).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Building {
    pub position: Vec2,
    pub heading: f32,
    pub piece: &'static str,
    pub scale: f32,
    pub size: Vec3,
}

/// The city pack's pieces a city is built of, with their sizes as made
/// (wide, tall, deep, m; as `tools/import_city.py` prints them): its
/// blocks and towers; its industry, here and there; and, out in the
/// water, the turbines and platforms of the sea.
const BLOCKS: [(&str, Vec3); 10] = [
    ("city_office", Vec3::new(24.2, 90.0, 25.1)),
    ("city_residential", Vec3::new(29.3, 110.0, 33.9)),
    ("city_megablock", Vec3::new(62.4, 140.0, 89.5)),
    ("city_ziggurat", Vec3::new(81.6, 80.0, 81.7)),
    ("city_cylinder", Vec3::new(23.9, 120.0, 23.2)),
    ("city_stacked", Vec3::new(33.9, 100.0, 29.2)),
    ("city_spire", Vec3::new(27.8, 200.0, 26.4)),
    ("city_stepped", Vec3::new(71.8, 150.0, 37.6)),
    ("city_twin", Vec3::new(63.4, 120.0, 21.6)),
    ("city_faceted", Vec3::new(33.6, 180.0, 33.3)),
];
const INDUSTRY: [(&str, Vec3); 5] = [
    ("chimney_stack", Vec3::new(43.8, 90.0, 46.7)),
    ("cooling_towers", Vec3::new(82.1, 70.0, 38.4)),
    ("refinery", Vec3::new(59.2, 71.6, 60.0)),
    ("silo_row", Vec3::new(50.0, 55.2, 11.2)),
    ("antenna_mast", Vec3::new(37.2, 130.0, 37.8)),
];
const AT_SEA: [(&str, Vec3); 2] = [
    ("wind_turbine", Vec3::new(67.1, 110.0, 18.6)),
    ("oil_platform", Vec3::new(62.9, 70.0, 43.5)),
];
/// The sign towers, two sets of colours, by the road now and then.
const SIGN_TOWERS: [(&str, Vec3); 2] = [
    ("sign_tower_a", Vec3::new(10.8, 60.0, 6.0)),
    ("sign_tower_b", Vec3::new(10.8, 60.0, 6.0)),
];

/// How wide a building of a city may be at most (m, its widest across, so
/// that it keeps to its place in its row), how far it keeps from the
/// middle of any road (m, over its own half width), how much bigger than
/// as made a block may be, and how often a place is the industry's, one
/// in the sea's (where it is water), and one by the road a sign tower's.
const CITY_WIDEST: f32 = 40.0;
const CITY_CLEAR: f32 = 16.0;
const CITY_LARGEST: f32 = 1.3;
const INDUSTRY_SHARE: f32 = 0.12;
const SEA_SHARE: f32 = 0.55;
const SIGN_SHARE: f32 = 0.1;

/// A number from 0 to 1 for `n`, the same every time: what makes each
/// building its own.
fn chance(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// The buildings of `city` round `track`: row after row out from the lap
/// on both sides, each its own, the further rows taller; in the water,
/// out in the last row, more often the sea's own; none within reach of a
/// road (the lap comes back past itself), nor of what the circuit puts
/// there of its own (`clear`: where, and how far round).
pub fn buildings(
    track: &Track,
    terrain: &Terrain,
    city: &City,
    clear: &[(Vec2, f32)],
) -> Vec<Building> {
    let lap = track.lap_length();
    let count = (lap / city.every) as u32;
    let mut built = Vec::new();
    for k in 0..count {
        let index = track.index_at(k as f32 * city.every);
        for (s, side) in [1.0_f32, -1.0].into_iter().enumerate() {
            for row in 0..city.rows {
                let n = (k * 2 + s as u32) * 8 + row as u32;
                let out = city.near
                    + (city.far - city.near) * (row as f32 + 0.8 * chance(n + 7))
                        / city.rows as f32;
                let guess = track.offset_point(index, side * (out + CITY_WIDEST / 2.0));
                let at_sea = row + 1 == city.rows
                    && terrain.height(guess) < SEA_LEVEL
                    && chance(n + 401) < SEA_SHARE;
                let pick = |list: &[(&'static str, Vec3)], salt: u32| {
                    list[(chance(n + salt) * list.len() as f32) as usize % list.len()]
                };
                let ((piece, made), block) = if at_sea {
                    (pick(&AT_SEA, 409), false)
                } else if row == 0 && chance(n + 503) < SIGN_SHARE {
                    (pick(&SIGN_TOWERS, 509), false)
                } else if chance(n + 307) < INDUSTRY_SHARE {
                    (pick(&INDUSTRY, 311), false)
                } else {
                    (pick(&BLOCKS, 211), true)
                };
                let widest = made.x.max(made.z);
                let fit = CITY_WIDEST / widest;
                let scale = if block {
                    let wanted =
                        (40.0 + 110.0 * chance(n + 31).powi(2)) * (1.0 + 0.35 * row as f32);
                    (wanted / made.y).min(fit).min(CITY_LARGEST)
                } else {
                    fit.min(1.0)
                };
                let size = made * scale;
                let position = track.offset_point(index, side * (out + size.z / 2.0));
                let reach = size.x.max(size.z) * 0.75;
                if track.distance_to_centerline(position) < reach + CITY_CLEAR
                    || clear
                        .iter()
                        .any(|&(at, round)| position.distance(at) < round + reach)
                {
                    continue;
                }
                let (_, heading) = track.pose_at(index);
                built.push(Building {
                    position,
                    heading: heading + 0.25 * (chance(n + 53) - 0.5),
                    piece,
                    scale,
                    size,
                });
            }
        }
    }
    built
}

/// How high a prop's foot stands: over the road, on the road there; else
/// on the lowest of the ground under it, a little sunk.
pub fn foot(track: &Track, terrain: &Terrain, prop: &Prop<NeonModel>, reach: f32) -> f32 {
    let (position, _) = spot(track, prop);
    match (prop.model, prop.spot) {
        (NeonModel::ContainerShip, _) => SEA_LEVEL - DRAUGHT,
        // Their legs, or their foundation, in the sea.
        (NeonModel::WindTurbine | NeonModel::OilPlatform, _) => SEA_LEVEL - SUNK,
        (NeonModel::AirshipCobalt | NeonModel::AirshipNeon, _) => AIRSHIP_HEIGHT,
        // What is low and wide stands on a platform up near the road's
        // height, in view from it, not down on the water.
        (NeonModel::FusionPlant | NeonModel::HarbourCrane, _) => {
            track.road_height(position) - BELOW_ROAD
        }
        (_, Spot::Over { along } | Spot::Road { along, .. }) => {
            // Over the road, on it; by the road, level with it (on its own
            // pylon where the road is a skyway, see `spawn`).
            track.surface_point(track.index_at(along), 0.0).y
        }
        _ => {
            [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
                .into_iter()
                .map(|offset| terrain.height(position + offset * 0.6 * reach))
                .fold(f32::MAX, f32::min)
                // In the water, on it: a city on the sea.
                .max(SEA_LEVEL + AFLOAT)
                - SUNK
        }
    }
}

/// What turns: the observation wheel's wheel (about its axle, slowly) and
/// a wind turbine's rotor, each at its own `rate` (rad/s, about its own
/// +Z).
#[derive(Component)]
pub struct Spinning {
    pub rate: f32,
}

/// A cabin of the wheel, hanging from it: it turns back as much as the
/// wheel turns it, so as to hang upright.
#[derive(Component)]
pub struct Hanging {
    pub rate: f32,
}

/// A wind turbine's red light, over its nacelle, blinking: when in its
/// cycle (s) it starts.
#[derive(Component)]
pub struct Blinking {
    pub phase: f32,
}

/// The observation wheel's axle over its foot and how long a turn of it
/// takes (s); a wind turbine's axle over its foot, how fast its rotor
/// turns (rad/s, between the two) and how high its red light is.
const WHEEL_AXLE: Vec3 = Vec3::new(0.0, 49.39, 0.0);
const WHEEL_TURN: f32 = 150.0;
const TURBINE_AXLE: Vec3 = Vec3::new(0.08, 78.1, 0.0);
const TURBINE_RATES: (f32, f32) = (0.9, 1.6);
const TURBINE_LIGHT: f32 = 83.5;
/// The red light: how big (m), how bright, how long its cycle (s) and how
/// much of it it is lit.
const BEACON_SIZE: f32 = 0.9;
const BEACON_GLOW: f32 = 30.0;
const BEACON_CYCLE: f32 = 1.6;
const BEACON_ON: f32 = 0.35;

/// The red lights' mesh and material, shared.
struct Beacon {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl Beacon {
    fn parts(assets: &mut Assets3d) -> Self {
        Self {
            mesh: assets.meshes.add(Sphere::new(BEACON_SIZE / 2.0)),
            material: assets.materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.1, 0.05),
                emissive: LinearRgba::rgb(BEACON_GLOW, 1.0, 0.5),
                unlit: true,
                ..default()
            }),
        }
    }
}

/// Spawns the piece `name` of the pack `folder` where `transform` puts it;
/// the wheel and a turbine with what of them turns, `number` making each
/// its own.
#[allow(clippy::too_many_arguments)]
fn spawn_piece(
    world: &mut ChildSpawnerCommands,
    assets: &Assets3d,
    folder: &str,
    name: &str,
    material: &Handle<StandardMaterial>,
    beacon: &Beacon,
    transform: Transform,
    number: u32,
) {
    let pack = assets.packs.get(folder);
    let mut root = world.spawn((
        Mesh3d(pack.piece(name).mesh.clone()),
        MeshMaterial3d(material.clone()),
        transform,
    ));
    match name {
        "ferris_wheel" => {
            let rate = std::f32::consts::TAU / WHEEL_TURN;
            root.with_children(|wheel_stand| {
                wheel_stand
                    .spawn((
                        Mesh3d(pack.piece("ferris_wheel_wheel").mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_translation(WHEEL_AXLE),
                        Spinning { rate },
                    ))
                    .with_children(|wheel| {
                        for cabin in
                            (0..).map_while(|k| pack.find(&format!("ferris_wheel_cabin_{k}")))
                        {
                            // Hanging from the middle of its top.
                            let hook = Vec3::new(
                                (cabin.low.x + cabin.high.x) / 2.0,
                                cabin.high.y,
                                (cabin.low.z + cabin.high.z) / 2.0,
                            );
                            wheel
                                .spawn((
                                    Transform::from_translation(hook),
                                    Visibility::default(),
                                    Hanging { rate },
                                ))
                                .with_child((
                                    Mesh3d(cabin.mesh.clone()),
                                    MeshMaterial3d(material.clone()),
                                    Transform::from_translation(-hook),
                                ));
                        }
                    });
            });
        }
        "wind_turbine" => {
            let rate = TURBINE_RATES.0 + (TURBINE_RATES.1 - TURBINE_RATES.0) * chance(number + 7);
            root.with_children(|tower| {
                tower.spawn((
                    Mesh3d(pack.piece("wind_turbine_rotor").mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(TURBINE_AXLE)
                        .with_rotation(Quat::from_rotation_z(chance(number + 11) * 2.1)),
                    Spinning { rate },
                ));
                tower.spawn((
                    Mesh3d(beacon.mesh.clone()),
                    MeshMaterial3d(beacon.material.clone()),
                    Transform::from_xyz(0.0, TURBINE_LIGHT, 0.0),
                    Blinking {
                        phase: chance(number + 13) * BEACON_CYCLE,
                    },
                ));
            });
        }
        _ => {}
    }
}

/// Turns what turns, and blinks the red lights.
pub struct NeonPlugin;

impl Plugin for NeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spin, blink));
    }
}

fn spin(
    time: Res<Time>,
    mut spinning: Query<(&Spinning, &mut Transform), Without<Hanging>>,
    mut hanging: Query<(&Hanging, &mut Transform), Without<Spinning>>,
) {
    let dt = time.delta_secs();
    for (spinning, mut transform) in &mut spinning {
        transform.rotate_local_z(spinning.rate * dt);
    }
    for (hanging, mut transform) in &mut hanging {
        transform.rotate_local_z(-hanging.rate * dt);
    }
}

fn blink(time: Res<Time>, mut lights: Query<(&Blinking, &mut Visibility)>) {
    let now = time.elapsed_secs();
    for (light, mut visibility) in &mut lights {
        let on = ((now + light.phase) % BEACON_CYCLE) < BEACON_ON * BEACON_CYCLE;
        let wanted = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

impl Theme for Neon {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        _scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        // Each pack's material, lit of its own: the props' in their own
        // colours, a little; the city's where its glow says.
        let mut lit = |folder: &str| {
            let pack = assets.packs.get(folder);
            let glow = pack.glow.clone();
            match assets.materials.get(&pack.material).cloned() {
                Some(base) => assets.materials.add(match glow {
                    Some(glow) => StandardMaterial {
                        emissive: LinearRgba::rgb(CITY_GLOW, CITY_GLOW, CITY_GLOW),
                        emissive_texture: Some(glow),
                        ..base
                    },
                    None => StandardMaterial {
                        emissive: LinearRgba::rgb(GLOW, GLOW, GLOW),
                        emissive_texture: base.base_color_texture.clone(),
                        ..base
                    },
                }),
                None => pack.material.clone(),
            }
        };
        let materials = [(PACK, lit(PACK)), (CITY_PACK, lit(CITY_PACK))];
        let material_of = |folder: &str| {
            materials
                .iter()
                .find(|(name, _)| *name == folder)
                .map(|(_, material)| material.clone())
                .expect("a pack of the theme's")
        };
        let beacon = Beacon::parts(assets);
        // The city: the city pack's buildings, each as big as it is put.
        if let Some(city) = &self.city {
            let material = material_of(CITY_PACK);
            let clear = self.landmarks(track);
            for (number, building) in buildings(track, terrain, city, &clear)
                .into_iter()
                .enumerate()
            {
                let ground = terrain.height(building.position).max(SEA_LEVEL) - SUNK;
                let at = Vec3::new(building.position.x, ground, building.position.y);
                let transform = Transform::from_translation(at)
                    .with_rotation(Quat::from_rotation_y(building.heading))
                    .with_scale(Vec3::splat(building.scale));
                spawn_piece(
                    world,
                    assets,
                    CITY_PACK,
                    building.piece,
                    &material,
                    &beacon,
                    transform,
                    number as u32,
                );
            }
        }
        let pylon = assets.meshes.add(Cuboid::from_length(1.0));
        let dark = assets.materials.add(StandardMaterial {
            base_color: PYLON,
            perceptual_roughness: 0.6,
            ..default()
        });
        self.props
            .iter()
            .enumerate()
            .map(|(number, prop)| {
                let folder = prop.model.pack();
                let piece = assets.packs.get(folder).piece(prop.model.name());
                let size = (piece.high - piece.low) * prop.scale;
                let reach = size.x.max(size.z) / 2.0;
                let (position, rotation) = spot(track, prop);
                let foot = foot(track, terrain, prop, reach);
                spawn_piece(
                    world,
                    assets,
                    folder,
                    prop.model.name(),
                    &material_of(folder),
                    &beacon,
                    Transform::from_xyz(position.x, foot, position.y)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(prop.scale)),
                    10_000 + number as u32,
                );
                // High over the ground or the water: on a pylon.
                let ground = terrain.height(position);
                let floats = matches!(
                    prop.model,
                    NeonModel::AirshipCobalt
                        | NeonModel::AirshipNeon
                        | NeonModel::WindTurbine
                        | NeonModel::OilPlatform
                );
                if !floats && !matches!(prop.spot, Spot::Over { .. }) && foot - ground > 1.5 {
                    let tall = foot - ground + 2.0;
                    world.spawn((
                        Mesh3d(pylon.clone()),
                        MeshMaterial3d(dark.clone()),
                        Transform::from_xyz(position.x, foot - tall / 2.0, position.y)
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(
                                PYLON_SHARE * size.x,
                                tall,
                                PYLON_SHARE * size.z,
                            )),
                    ));
                }
                // A gate over the road and what flies take no ground from
                // what stands by them.
                match (prop.model, prop.spot) {
                    (_, Spot::Over { .. })
                    | (NeonModel::AirshipCobalt | NeonModel::AirshipNeon, _) => (position, 0.0),
                    _ => (position, reach),
                }
            })
            .collect()
    }

    fn packs() -> &'static [&'static str] {
        &[PACK, CITY_PACK]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;
    use motorust_engine::bike::model::imported::read;
    use motorust_engine::track::WALL_DISTANCE;

    fn pieces() -> Vec<(String, Vec<Vec3>)> {
        pieces_of(PACK)
    }

    fn pieces_of(folder: &str) -> Vec<(String, Vec<Vec3>)> {
        let bin = motorust_engine::content::file(&format!("models/{folder}/props.bin"));
        read(bin)
            .meshes
            .into_iter()
            .map(|piece| {
                let positions = piece
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|positions| positions.as_float3())
                    .expect("a prop has its points")
                    .iter()
                    .map(|&p| Vec3::from(p))
                    .collect();
                (piece.name, positions)
            })
            .collect()
    }

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        crate::game::register();
        let packs = [(PACK, pieces_of(PACK)), (CITY_PACK, pieces_of(CITY_PACK))];
        for model in NeonModel::ALL {
            let (_, pieces) = packs
                .iter()
                .find(|(folder, _)| *folder == model.pack())
                .expect("its pack");
            let (_, points) = pieces
                .iter()
                .find(|(name, _)| name == model.name())
                .unwrap_or_else(|| panic!("no {} in the pack", model.name()));
            let low = points.iter().map(|p| p.y).fold(f32::MAX, f32::min);
            assert!(low.abs() < 0.01, "{}: {low}", model.name());
            assert!(points.len() < 8000, "{}: {}", model.name(), points.len());
        }
    }

    #[test]
    fn the_gate_leaves_the_road_clear_between_its_legs_and_under_its_beam() {
        crate::game::register();
        let pieces = pieces();
        let (_, gate) = pieces
            .iter()
            .find(|(name, _)| name == "start_gate")
            .unwrap();
        // Nothing of it within the barriers lower than well over a rider.
        let lowest = gate
            .iter()
            .filter(|p| p.x.abs() < WALL_DISTANCE + 0.5)
            .map(|p| p.y)
            .fold(f32::MAX, f32::min);
        assert!(lowest > 6.0, "{lowest} m over the road");
    }

    #[test]
    fn puerto_neon_s_props_stand_clear_of_its_roads_but_the_gate_over_the_line() {
        crate::game::register();
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.name_en == "Neon Harbor")
            .expect("Puerto Neón");
        let neon = circuit.scenery.themes.get::<Neon>().expect("its props");
        let track = Track::new(&circuit.layout);
        let pieces: Vec<_> = pieces().into_iter().chain(pieces_of(CITY_PACK)).collect();
        assert!(
            neon.props
                .iter()
                .any(|prop| prop.model == NeonModel::StartGate
                    && matches!(prop.spot, Spot::Over { along } if along == 0.0))
        );
        for prop in neon.props {
            if matches!(prop.spot, Spot::Over { .. }) {
                assert_eq!(prop.model, NeonModel::StartGate);
                continue;
            }
            // The airships fly far over the roads.
            if matches!(
                prop.model,
                NeonModel::AirshipCobalt | NeonModel::AirshipNeon
            ) {
                continue;
            }
            let (_, points) = pieces
                .iter()
                .find(|(name, _)| name == prop.model.name())
                .unwrap();
            // One by the road faces it: its depth is what reaches towards
            // it; any other may be turned any way.
            let beside = matches!(prop.spot, Spot::Road { .. }) && !prop.facing_riders;
            let reach = points
                .iter()
                .map(|p| {
                    if beside {
                        p.z.abs()
                    } else {
                        p.x.abs().max(p.z.abs())
                    }
                })
                .fold(0.0, f32::max)
                * prop.scale;
            let (position, _) = spot(&track, prop);
            let clear = track.distance_to_centerline(position) - reach;
            assert!(
                clear > WALL_DISTANCE + 1.0,
                "{:?} at {position}: {clear} m from the road's middle",
                prop.model
            );
        }
    }

    #[test]
    fn the_city_stands_close_on_both_sides_all_round_the_lap_clear_of_its_roads() {
        crate::game::register();
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.name_en == "Neon Harbor")
            .expect("Puerto Neón");
        let neon = circuit.scenery.themes.get::<Neon>().expect("its props");
        let city = neon.city.expect("its city");
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        let clear = neon.landmarks(&track);
        let built = buildings(&track, &terrain, &city, &clear);
        // Many, the same every time; every block among them, some of the
        // industry, the sea's and the sign towers.
        assert!(built.len() > 300, "{}", built.len());
        assert_eq!(built, buildings(&track, &terrain, &city, &clear));
        // Clear of the landmarks.
        for building in &built {
            for &(at, round) in &clear {
                assert!(
                    building.position.distance(at) >= round,
                    "{building:?} at {at}"
                );
            }
        }
        for (piece, _) in BLOCKS {
            assert!(built.iter().any(|b| b.piece == piece), "no {piece}");
        }
        let count = |list: &[(&str, Vec3)]| {
            built
                .iter()
                .filter(|b| list.iter().any(|(piece, _)| *piece == b.piece))
                .count()
        };
        assert!(count(&INDUSTRY) > 10, "{}", count(&INDUSTRY));
        assert!(count(&AT_SEA) > 5, "{}", count(&AT_SEA));
        assert!(count(&SIGN_TOWERS) > 5, "{}", count(&SIGN_TOWERS));
        // Every piece in the pack, as big as it says.
        let pieces = pieces_of(CITY_PACK);
        for (piece, made) in BLOCKS
            .iter()
            .chain(&INDUSTRY)
            .chain(&AT_SEA)
            .chain(&SIGN_TOWERS)
        {
            let (_, points) = pieces
                .iter()
                .find(|(name, _)| name == piece)
                .unwrap_or_else(|| panic!("no {piece} in the pack"));
            let tall = points.iter().map(|p| p.y).fold(f32::MIN, f32::max);
            // The turbine's rotor reaches over its tower.
            let rotor = if *piece == "wind_turbine" { 30.0 } else { 0.5 };
            assert!((tall - made.y).abs() < rotor, "{piece}: {tall} m tall");
        }
        // None in the way of a road, none wider than its place, each tall
        // enough to stand over the road.
        for building in &built {
            let clear = track.distance_to_centerline(building.position)
                - building.size.x.max(building.size.z) / 2.0;
            assert!(clear > WALL_DISTANCE + 2.0, "{building:?}: {clear}");
            assert!(building.size.x.max(building.size.z) <= CITY_WIDEST + 0.01);
            assert!(building.size.y >= 25.0, "{building:?}");
        }
        // All round the lap there is one near: within 120 m of every
        // point of it, both sides taken together.
        let lap = track.lap_length();
        for along in (0..lap as usize).step_by(100) {
            let here = track.offset_point(track.index_at(along as f32), 0.0);
            let nearest = built
                .iter()
                .map(|b| b.position.distance(here))
                .fold(f32::MAX, f32::min);
            assert!(nearest < 120.0, "{nearest} m at {along} m");
        }
    }

    #[test]
    fn the_wheel_turns_with_its_cabins_hanging_and_the_turbines_with_their_rotors() {
        crate::game::register();
        let pieces = pieces_of(CITY_PACK);
        let find = |name: &str| {
            pieces
                .iter()
                .find(|(piece, _)| piece == name)
                .map(|(_, p)| p)
        };
        // The wheel about its axle: its rim as far up as down.
        let wheel = find("ferris_wheel_wheel").expect("the wheel");
        let (low, high) = wheel
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p.y), h.max(p.y)));
        assert!((high + low).abs() < 3.0, "{low} {high}");
        // Its cabins out at its rim, a dozen and more.
        let cabins = (0..)
            .map_while(|k| find(&format!("ferris_wheel_cabin_{k}")))
            .count();
        assert!(cabins >= 12, "{cabins} cabins");
        // It stands on its legs, its axle a little over half its height.
        let legs = find("ferris_wheel").expect("its legs");
        let top = legs.iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(WHEEL_AXLE.y < top + 2.0 && WHEEL_AXLE.y > 0.4 * 90.0);
        // The rotor about its hub, its blades round it.
        let rotor = find("wind_turbine_rotor").expect("the rotor");
        let reach = rotor
            .iter()
            .map(|p| p.truncate().length())
            .fold(0.0, f32::max);
        assert!(reach > 25.0 && reach < 40.0, "{reach}");
    }
}
