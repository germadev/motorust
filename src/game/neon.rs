//! A theme of the game's, `"neon"`: the props of Puerto Neón, a harbour
//! city of the future by night, made with Meshy (`assets/models/neon/`,
//! prepared by `tools/import_neon.py`): the gate over the finish line, the
//! towers, the screens on their masts and the grandstand. A circuit's file
//! says where each stands (`props`).
//!
//! They are lit of their own, a little, in their own colours, so that their
//! windows and strips of light show at night.

use bevy::prelude::*;

use motorust_engine::track::Track;
use motorust_engine::world::Assets3d;
use motorust_engine::world::prop_pack::spot;
use motorust_engine::world::props::Footprint;
use motorust_engine::world::scenery::{Prop, Scenery, Spot};
use motorust_engine::world::terrain::{SEA_LEVEL, Terrain};
use motorust_engine::world::themes::Theme;

/// The pack of its props, in `assets/models/`.
const PACK: &str = "neon";
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
}

impl NeonModel {
    /// Every one of them, for the tests.
    #[cfg(test)]
    pub const ALL: [NeonModel; 12] = [
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

/// A city the lap runs through: blocks built in code, their windows lit,
/// and the pack's towers among them, in `rows` rows either side of the lap,
/// one every `every` metres along it, from `near` to `far` metres from its
/// middle, standing on the ground or in the water; none where another
/// stretch of the lap passes.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct City {
    pub every: f32,
    pub near: f32,
    pub far: f32,
    pub rows: usize,
}

/// A building of a city: where it stands, how it is turned, how wide and
/// deep and how tall it is, and which it is: one of the towers of the
/// pack, or a block built in code, of one of [`BLOCK_KINDS`] kinds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Building {
    pub position: Vec2,
    pub heading: f32,
    pub size: Vec3,
    pub tower: Option<NeonModel>,
    pub kind: usize,
}

/// How many kinds of block there are (their windows' colours), how far a
/// building keeps from the middle of any road (m, over its own half
/// width), and one building in so many is a tower of the pack.
pub const BLOCK_KINDS: usize = 7;
const CITY_CLEAR: f32 = 16.0;
const TOWER_EVERY: u32 = 5;

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
/// on both sides, each its own size, the further rows taller; none within
/// reach of a road (the lap comes back past itself).
pub fn buildings(track: &Track, city: &City) -> Vec<Building> {
    let lap = track.lap_length();
    let count = (lap / city.every) as u32;
    let mut built = Vec::new();
    for k in 0..count {
        let index = track.index_at(k as f32 * city.every);
        for (s, side) in [1.0_f32, -1.0].into_iter().enumerate() {
            for row in 0..city.rows {
                let n = (k * 2 + s as u32) * 8 + row as u32;
                let width = 16.0 + 22.0 * chance(n);
                let depth = 16.0 + 22.0 * chance(n + 101);
                let out = city.near
                    + (city.far - city.near) * (row as f32 + 0.8 * chance(n + 7))
                        / city.rows as f32;
                let position = track.offset_point(index, side * (out + depth / 2.0));
                let reach = width.max(depth) * 0.75;
                if track.distance_to_centerline(position) < reach + CITY_CLEAR {
                    continue;
                }
                let tower = (n % TOWER_EVERY == 0).then(|| {
                    [
                        NeonModel::TowerStepped,
                        NeonModel::TowerSpire,
                        NeonModel::TowerTwin,
                    ][(n / TOWER_EVERY) as usize % 3]
                });
                let height = (40.0 + 110.0 * chance(n + 31).powi(2)) * (1.0 + 0.35 * row as f32);
                let (_, heading) = track.pose_at(index);
                built.push(Building {
                    position,
                    heading: heading + 0.25 * (chance(n + 53) - 0.5),
                    size: Vec3::new(width, height, depth),
                    tower,
                    // Most of them the whites; the colours here and there.
                    kind: (chance(n + 211).powi(2) * BLOCK_KINDS as f32) as usize % BLOCK_KINDS,
                });
            }
        }
    }
    built
}

/// The picture of a block's face: dark, with rows of windows, some lit in
/// `lit` (`share` of them), the rest dark. The same on every face of every
/// block of its kind.
fn windows_image(lit: Color, share: f32, seed: u32) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    const ACROSS: u32 = 12;
    const UP: u32 = 40;
    const CELL: u32 = 4;
    let lit = lit.to_srgba();
    let mut data = Vec::with_capacity((ACROSS * UP * CELL * CELL * 4) as usize);
    for y in 0..UP * CELL {
        for x in 0..ACROSS * CELL {
            let window = (x % CELL) > 0 && (y % CELL) > 1;
            let on = chance(seed + (y / CELL) * ACROSS + x / CELL) < share;
            let (r, g, b) = if window && on {
                (lit.red, lit.green, lit.blue)
            } else if window {
                (0.05, 0.06, 0.08)
            } else {
                (0.0, 0.0, 0.0)
            };
            data.extend([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: ACROSS * CELL,
            height: UP * CELL,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

/// Each kind of block: the colour of its lit windows, of its walls, and
/// how many of its windows are lit. A city's own mix, most of it the
/// whites of offices and homes, a few in colour: warm white, cold white,
/// pale yellow, a green-blue, dim sodium orange, a few red, and one of the
/// circuit's pink.
const KINDS: [(Color, Color, f32); BLOCK_KINDS] = [
    (
        Color::srgb(1.0, 0.9, 0.72),
        Color::srgb(0.2, 0.19, 0.18),
        0.45,
    ),
    (
        Color::srgb(0.85, 0.93, 1.0),
        Color::srgb(0.14, 0.16, 0.2),
        0.55,
    ),
    (
        Color::srgb(1.0, 0.95, 0.55),
        Color::srgb(0.24, 0.22, 0.19),
        0.3,
    ),
    (
        Color::srgb(0.5, 0.95, 0.85),
        Color::srgb(0.12, 0.17, 0.17),
        0.35,
    ),
    (
        Color::srgb(1.0, 0.62, 0.3),
        Color::srgb(0.22, 0.17, 0.14),
        0.25,
    ),
    (
        Color::srgb(1.0, 0.25, 0.2),
        Color::srgb(0.1, 0.1, 0.11),
        0.12,
    ),
    (
        Color::srgb(1.0, 0.45, 0.8),
        Color::srgb(0.15, 0.13, 0.18),
        0.3,
    ),
];

/// How high a prop's foot stands: over the road, on the road there; else
/// on the lowest of the ground under it, a little sunk.
pub fn foot(track: &Track, terrain: &Terrain, prop: &Prop<NeonModel>, reach: f32) -> f32 {
    let (position, _) = spot(track, prop);
    match (prop.model, prop.spot) {
        (NeonModel::ContainerShip, _) => SEA_LEVEL - DRAUGHT,
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

impl Theme for Neon {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        _scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        let pack = assets.packs.get(PACK);
        // The pack's material, lit of its own in its own colours.
        let lit = assets.materials.get(&pack.material).cloned().map(|base| {
            assets.materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(GLOW, GLOW, GLOW),
                emissive_texture: base.base_color_texture.clone(),
                ..base
            })
        });
        let material = lit.unwrap_or_else(|| pack.material.clone());
        // The city: its blocks one mesh and a material for each kind; its
        // towers the pack's, as tall as each is.
        if let Some(city) = &self.city {
            let block = assets.meshes.add(Cuboid::from_length(1.0));
            let kinds: Vec<Handle<StandardMaterial>> = KINDS
                .iter()
                .enumerate()
                .map(|(kind, &(lit, wall, share))| {
                    let windows = assets
                        .images
                        .add(windows_image(lit, share, 977 * kind as u32));
                    assets.materials.add(StandardMaterial {
                        base_color: wall,
                        emissive: LinearRgba::rgb(2.2, 2.2, 2.2),
                        emissive_texture: Some(windows),
                        perceptual_roughness: 0.5,
                        ..default()
                    })
                })
                .collect();
            for building in buildings(track, city) {
                let ground = terrain.height(building.position).max(SEA_LEVEL) - SUNK;
                let at = Vec3::new(building.position.x, ground, building.position.y);
                let turn = Quat::from_rotation_y(building.heading);
                match building.tower {
                    Some(model) => {
                        let piece = pack.piece(model.name());
                        let scale = building.size.y / (piece.high.y - piece.low.y);
                        world.spawn((
                            Mesh3d(piece.mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_translation(at)
                                .with_rotation(turn)
                                .with_scale(Vec3::splat(scale.max(0.5))),
                        ));
                    }
                    None => {
                        world.spawn((
                            Mesh3d(block.clone()),
                            MeshMaterial3d(kinds[building.kind].clone()),
                            Transform::from_translation(at + Vec3::Y * building.size.y / 2.0)
                                .with_rotation(turn)
                                .with_scale(building.size),
                        ));
                    }
                }
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
            .map(|prop| {
                let piece = pack.piece(prop.model.name());
                let size = (piece.high - piece.low) * prop.scale;
                let reach = size.x.max(size.z) / 2.0;
                let (position, rotation) = spot(track, prop);
                let foot = foot(track, terrain, prop, reach);
                world.spawn((
                    Mesh3d(piece.mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(position.x, foot, position.y)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(prop.scale)),
                ));
                // High over the ground or the water: on a pylon.
                let ground = terrain.height(position);
                let flies = matches!(
                    prop.model,
                    NeonModel::AirshipCobalt | NeonModel::AirshipNeon
                );
                if !flies && !matches!(prop.spot, Spot::Over { .. }) && foot - ground > 1.5 {
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
        &[PACK]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;
    use motorust_engine::bike::model::imported::read;
    use motorust_engine::track::WALL_DISTANCE;

    fn pieces() -> Vec<(String, Vec<Vec3>)> {
        let bin = motorust_engine::content::file("models/neon/props.bin");
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
        let pieces = pieces();
        for model in NeonModel::ALL {
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
        let pieces = pieces();
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
        let built = buildings(&track, &city);
        // Many, the same every time, of every kind, towers among them.
        assert!(built.len() > 300, "{}", built.len());
        assert_eq!(built, buildings(&track, &city));
        for kind in 0..BLOCK_KINDS {
            assert!(built.iter().any(|b| b.tower.is_none() && b.kind == kind));
        }
        assert!(built.iter().filter(|b| b.tower.is_some()).count() > 30);
        // None in the way of a road, each tall enough to stand over it.
        for building in &built {
            let clear = track.distance_to_centerline(building.position)
                - building.size.x.max(building.size.z) / 2.0;
            assert!(clear > WALL_DISTANCE + 2.0, "{building:?}: {clear}");
            assert!(building.size.y >= 40.0);
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
}
