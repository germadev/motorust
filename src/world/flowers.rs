//! Fields of flowers (see [`FlowerField`]), as on the hills of Tsushima and
//! round the lakes at the foot of Mount Fuji: red spider lilies, white
//! flowers, yellow rape blossom, irises, the silvery gold plumes of susuki
//! and carpets of moss phlox.
//!
//! Where the fields are, and of what, is a map ([`FlowerMap`]) the terrain
//! is drawn with: its grass in flower is drawn apart, with a texture of
//! the lawn covered in blossoms (see [`super::foliage::meadow`]), so a
//! field is a carpet of colour from near and from afar at no cost but a
//! texture. Near the roads, clumps of the flowers stand up out of it (but
//! for the phlox, which hugs the ground): two cards standing crossed and one
//! lying over them, cut out of one atlas painted at startup (see
//! [`super::foliage::flowers`]), gathered into a mesh for each square of
//! ground, drawn only near the camera and casting no shadows, as they are
//! many and see-through.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Assets3d;
use super::foliage::{self, ALPHA_CUTOFF};
use super::props::Footprint;
use super::quads::MeshBuilder;
use super::scenery::{FlowerArea, FlowerField, FlowerKind};
use super::terrain::{SEA_LEVEL, Terrain};
use super::tufts::normals_up;
use crate::geometry::{fbm, random01, smoothstep};
use crate::track::Track;

/// How wide the ragged edge of a field is (m): it thins out across it.
const EDGE: f32 = 14.0;
/// Where the fields are everywhere, how big their patches are (m), and the
/// noise below which there is only grass between them; the other fields
/// are in flower all over, but at their ragged edges.
const PATCH: f32 = 40.0;
const GAPS: f32 = 0.4;
/// How far past the roads' bounds the flowers grow (m): over the island,
/// not on the mountains across the sea.
const ISLAND: f32 = 200.0;
/// How big the patches of each kind are, where a field has several (m):
/// broad swathes where it is everywhere.
const KIND_PATCH: f32 = 52.0;
const WIDE_KIND_PATCH: f32 = 130.0;
/// How far the flowers keep from the walls of the roads (m): the shoulders
/// stay grass.
const CLEAR_OF_WALL: f32 = 2.0;
/// The bands of moss phlox: how wide (m), and their colours in turn,
/// magenta most of all.
const PHLOX_BAND: f32 = 16.0;
const PHLOX: [Color; 7] = [
    Color::srgb(0.86, 0.46, 0.72),
    Color::srgb(0.94, 0.66, 0.8),
    Color::srgb(0.86, 0.46, 0.72),
    Color::srgb(0.76, 0.66, 0.9),
    Color::srgb(0.86, 0.46, 0.72),
    Color::srgb(0.94, 0.66, 0.8),
    Color::srgb(0.96, 0.93, 0.94),
];

/// The clumps standing up near the roads: how far apart (m), on a grid,
/// and how far each is shaken off its point of it (times that); out to how
/// far from the centre line of a road (m); how far they keep from whatever
/// else stands on the ground (m, past its footprint), and how steep the
/// ground they grow on may be.
const SPACING: f32 = 3.4;
const SHAKE: f32 = 0.45;
const CLUMP_REACH: f32 = 34.0;
const CLEAR: f32 = 0.5;
const STEEPEST: f32 = 0.6;
/// How deep the stems' roots are in the ground (m).
const ROOTED: f32 = 0.05;
/// How big a square of ground each mesh of clumps gathers (m), up to how
/// far from the camera they are drawn (m, from the middle of each) and over
/// how much further they fade out: further away, the ground in flower is
/// enough.
const CHUNK: f32 = 64.0;
const SEEN: f32 = 150.0;
const SEEN_FADE: f32 = 20.0;
/// The squares of ground the points of the roads are sorted into (m), to
/// find the nearest quickly.
const ROAD_CELL: f32 = 32.0;

/// A point of a road's centre line: where, where its walls are (m from it)
/// and, on the lap, how far round it is (m from the finish line).
#[derive(Clone, Copy)]
struct RoadPoint {
    at: Vec2,
    wall: f32,
    along: Option<f32>,
}

/// The points of the centre lines of all the roads, by squares of ground.
struct Roads {
    cells: HashMap<(i32, i32), Vec<RoadPoint>>,
}

fn road_cell(point: Vec2) -> (i32, i32) {
    (
        (point.x / ROAD_CELL).floor() as i32,
        (point.y / ROAD_CELL).floor() as i32,
    )
}

impl Roads {
    fn new(track: &Track) -> Self {
        let mut cells: HashMap<(i32, i32), Vec<RoadPoint>> = HashMap::new();
        for road in 0..=track.side_roads().len() {
            let line = track.road(road);
            for index in 0..line.sample_count() {
                let at = line.offset_point(index, 0.0);
                cells.entry(road_cell(at)).or_default().push(RoadPoint {
                    at,
                    wall: line.wall_at(index),
                    along: (road == 0).then(|| track.lap_along_at(index)),
                });
            }
        }
        Self { cells }
    }

    /// The nearest point of a centre line to `point` and how far it is, if
    /// any is within `reach` m.
    fn nearest(&self, point: Vec2, reach: f32) -> Option<(RoadPoint, f32)> {
        let (x, y) = road_cell(point);
        let cells = (reach / ROAD_CELL).ceil() as i32;
        let mut best: Option<(RoadPoint, f32)> = None;
        for dy in -cells..=cells {
            for dx in -cells..=cells {
                for road in self.cells.get(&(x + dx, y + dy)).into_iter().flatten() {
                    let distance = road.at.distance(point);
                    if distance <= reach && best.is_none_or(|(_, least)| distance < least) {
                        best = Some((*road, distance));
                    }
                }
            }
        }
        best
    }
}

/// Where the fields of flowers of a circuit are, and of what.
pub struct FlowerMap {
    fields: &'static [FlowerField],
    roads: Roads,
    lap: f32,
    /// The corners of the ground the flowers grow on.
    bounds: (Vec2, Vec2),
}

impl FlowerMap {
    pub fn new(track: &Track, fields: &'static [FlowerField]) -> Self {
        let (min, max) = track.bounds();
        Self {
            fields,
            roads: Roads::new(track),
            lap: track.lap_length(),
            bounds: (min - Vec2::splat(ISLAND), max + Vec2::splat(ISLAND)),
        }
    }

    /// What is in flower at `point`, if anything (whether the ground there
    /// is grass, dry and so on is the terrain's to say).
    pub fn at(&self, point: Vec2) -> Option<FlowerKind> {
        if point.cmplt(self.bounds.0).any() || point.cmpgt(self.bounds.1).any() {
            return None;
        }
        if self
            .roads
            .nearest(point, ROAD_CELL)
            .is_some_and(|(road, distance)| distance < road.wall + CLEAR_OF_WALL)
        {
            return None;
        }
        for (f, field) in self.fields.iter().enumerate() {
            // Past the middle of its ragged edge, the next field.
            let full = fullness(&field.area, point, &self.roads, self.lap);
            if full.is_none_or(|full| full < 0.5) {
                continue;
            }
            let offset = Vec2::splat(f as f32 * 13.7);
            let everywhere = matches!(field.area, FlowerArea::Everywhere);
            if everywhere && fbm(point / PATCH + offset, 2) < GAPS {
                return None;
            }
            let scale = if everywhere {
                WIDE_KIND_PATCH
            } else {
                KIND_PATCH
            };
            let pick = (fbm(point / scale + offset + 41.0, 2) - 0.3) / 0.4;
            let kinds = field.kinds.len() as f32;
            return Some(field.kinds[(pick.clamp(0.0, 0.999) * kinds) as usize]);
        }
        None
    }
}

/// The colour of moss phlox at `point` (linear RGBA): in wavy bands, as it
/// is planted, of magenta, pink, lavender and white.
pub fn phlox_color(point: Vec2) -> [f32; 4] {
    let across = point.dot(Vec2::new(0.6, 0.8)) / PHLOX_BAND + 3.0 * fbm(point / 70.0, 2);
    let band = (across.floor() as i64).rem_euclid(PHLOX.len() as i64) as usize;
    PHLOX[band].to_linear().to_f32_array()
}

/// How full of flowers an area is at `point` (0..1): thinning out across
/// its ragged edge; `None` outside it.
fn fullness(area: &FlowerArea, point: Vec2, roads: &Roads, lap: f32) -> Option<f32> {
    let ragged = (fbm(point / 10.0, 2) - 0.5) * EDGE;
    match *area {
        FlowerArea::Everywhere => Some(1.0),
        FlowerArea::Round {
            center,
            inner,
            outer,
        } => {
            let distance = point.distance(center);
            (distance >= inner && distance <= outer)
                .then(|| smoothstep(outer, outer - EDGE, distance + ragged))
        }
        FlowerArea::Road { span, near, far } => {
            let (road, distance) = roads.nearest(point, far)?;
            let along = road.along?;
            if distance < near || !span.contains(along, lap) {
                return None;
            }
            // Thinning out at both ends of the stretch too.
            let ends = span.weight(along, lap, EDGE * 2.0);
            Some(ends * smoothstep(far, far - EDGE, distance + ragged))
        }
    }
}

/// How tall and wide a clump of each kind is (m), and how high over its
/// root the card lying over it is (times its height): where its flowers
/// open. Moss phlox has none: it hugs the ground.
fn size(kind: FlowerKind) -> Option<(f32, f32, f32)> {
    match kind {
        FlowerKind::SpiderLily => Some((0.65, 1.4, 0.72)),
        FlowerKind::White => Some((0.55, 1.4, 0.7)),
        FlowerKind::Yellow => Some((0.85, 1.6, 0.75)),
        FlowerKind::Iris => Some((0.95, 1.3, 0.68)),
        FlowerKind::Susuki => Some((1.7, 1.8, 0.8)),
        FlowerKind::Phlox => None,
    }
}

/// A clump of flowers: where its root is, of which kind, how big (times
/// its kind's size), turned how (rad) and how light (times the atlas's
/// colours).
#[derive(Clone, Copy, Debug)]
struct Clump {
    root: Vec3,
    kind: FlowerKind,
    scale: f32,
    turn: f32,
    shade: f32,
}

/// The flowers of a circuit, worked out away from the main thread as its
/// trees are (see [`super::props::TreePlan`]): the clumps standing up near
/// the roads, and the atlas they are cut out of. Those in the way of what
/// stands on the ground are left out when they are spawned.
#[derive(Default)]
pub struct FlowerPlan {
    clumps: Vec<Clump>,
    atlas: Option<Image>,
}

/// The flowers of a circuit whose ground is in flower (see [`FlowerPlan`]).
pub fn plan_flowers(track: &Track, terrain: &Terrain) -> FlowerPlan {
    if !terrain.in_flower() {
        return FlowerPlan::default();
    }
    FlowerPlan {
        clumps: clumps(track, terrain),
        atlas: Some(foliage::flowers()),
    }
}

/// The clumps of flowers planned for a circuit, but those in the way of
/// what is `taken`. Returns the ground of its round meadows, which nothing
/// else is put in.
pub fn spawn_flowers(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    fields: &[FlowerField],
    plan: FlowerPlan,
    taken: &[Footprint],
    assets: &mut Assets3d,
) -> Vec<Footprint> {
    let FlowerPlan { clumps, atlas } = plan;
    let Some(atlas) = atlas else {
        return Vec::new();
    };
    let material = assets.materials.add(StandardMaterial {
        base_color_texture: Some(assets.images.add(atlas)),
        alpha_mode: AlphaMode::Mask(ALPHA_CUTOFF),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        // Lit as the ground they grow on, from either side.
        cull_mode: None,
        ..default()
    });
    for mesh in clump_meshes(&clear_of(clumps, taken), terrain) {
        world.spawn((
            Mesh3d(assets.meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: SEEN..SEEN + SEEN_FADE,
                use_aabb: true,
            },
            NotShadowCaster,
        ));
    }
    fields
        .iter()
        .filter_map(|field| match field.area {
            FlowerArea::Round {
                center,
                inner,
                outer,
            } if inner <= 0.0 => Some((center, outer)),
            _ => None,
        })
        .collect()
}

/// The clumps but those in the way of what is `taken`.
fn clear_of(clumps: Vec<Clump>, taken: &[Footprint]) -> Vec<Clump> {
    clumps
        .into_iter()
        .filter(|clump| {
            let root = Vec2::new(clump.root.x, clump.root.z);
            !taken
                .iter()
                .any(|&(at, radius)| at.distance(root) < radius + CLEAR)
        })
        .collect()
}

/// The clumps standing up out of the ground in flower near the roads (see
/// [`Terrain::flowers_drawn`]), on dry ground not too steep.
fn clumps(track: &Track, terrain: &Terrain) -> Vec<Clump> {
    let roads = Roads::new(track);
    let (min, max) = track.bounds();
    let (min, max) = (
        min - Vec2::splat(CLUMP_REACH),
        max + Vec2::splat(CLUMP_REACH),
    );
    let (columns, rows) = (
        ((max.x - min.x) / SPACING).ceil() as u32,
        ((max.y - min.y) / SPACING).ceil() as u32,
    );
    let mut clumps = Vec::new();
    for row in 0..=rows {
        for column in 0..=columns {
            let seed = (row * 7919 + column).wrapping_mul(8);
            let random = |k: u32| random01(seed.wrapping_add(k));
            let shaken = (Vec2::new(random(0), random(1)) - 0.5) * 2.0 * SHAKE;
            let point = min + (Vec2::new(column as f32, row as f32) + shaken) * SPACING;
            // Near a road, but clear of its walls (the ground is in flower
            // by its triangles, which reach a little nearer).
            if roads
                .nearest(point, CLUMP_REACH)
                .is_none_or(|(road, distance)| distance < road.wall + CLEAR_OF_WALL)
            {
                continue;
            }
            let Some(kind) = terrain.flowers_drawn(point) else {
                continue;
            };
            let height = terrain.height(point);
            let grade = Vec2::new(
                terrain.height(point + Vec2::X) - terrain.height(point - Vec2::X),
                terrain.height(point + Vec2::Y) - terrain.height(point - Vec2::Y),
            ) / 2.0;
            if size(kind).is_none() || height < SEA_LEVEL + 0.5 || grade.length() > STEEPEST {
                continue;
            }
            clumps.push(Clump {
                root: Vec3::new(point.x, height - ROOTED, point.y),
                kind,
                scale: 0.8 + 0.4 * random(3),
                turn: random(4) * PI,
                shade: 0.85 + 0.25 * random(5),
            });
        }
    }
    clumps
}

/// The meshes of the clumps, one for each square of ground with any.
fn clump_meshes(clumps: &[Clump], terrain: &Terrain) -> Vec<Mesh> {
    let mut squares: HashMap<(i32, i32), MeshBuilder> = HashMap::new();
    for clump in clumps {
        let square = (
            (clump.root.x / CHUNK).floor() as i32,
            (clump.root.z / CHUNK).floor() as i32,
        );
        add_clump(squares.entry(square).or_default(), clump, terrain);
    }
    let mut squares: Vec<_> = squares.into_iter().collect();
    squares.sort_by_key(|(square, _)| *square);
    squares
        .into_iter()
        .map(|(_, mesh)| {
            let mut mesh = mesh.build();
            normals_up(&mut mesh);
            mesh
        })
        .collect()
}

/// A clump: two cards standing crossed at right angles, turned at random,
/// and one lying over them where the flowers open, as the ground slopes;
/// every normal points up, so it is lit like the ground.
fn add_clump(mesh: &mut MeshBuilder, clump: &Clump, terrain: &Terrain) {
    let Some((height, width, lying)) = size(clump.kind) else {
        return;
    };
    let (height, width) = (height * clump.scale, width * clump.scale);
    let color = Color::srgb(clump.shade, clump.shade, clump.shade);
    let (min, max) = foliage::flower_cell(clump.kind, false);
    let root = clump.root;
    for k in 0..2 {
        let across = Vec2::from_angle(clump.turn + k as f32 * FRAC_PI_2);
        let half = Vec3::new(across.x, 0.0, across.y) * width * 0.5;
        let up = Vec3::Y * height;
        mesh.quad(
            [
                (root - half, Vec2::new(min.x, max.y)),
                (root + half, max),
                (root + half + up, Vec2::new(max.x, min.y)),
                (root - half + up, min),
            ],
            Vec3::new(-across.y, 0.0, across.x),
            color,
        );
    }
    let (min, max) = foliage::flower_cell(clump.kind, true);
    let a = Vec2::from_angle(clump.turn) * width * 0.5;
    let b = a.perp();
    let corner = |offset: Vec2, uv: Vec2| {
        let at = Vec2::new(root.x, root.z) + offset;
        (
            Vec3::new(at.x, terrain.height(at) + lying * height, at.y),
            uv,
        )
    };
    mesh.quad(
        [
            corner(-a - b, min),
            corner(a - b, Vec2::new(max.x, min.y)),
            corner(a + b, max),
            corner(-a + b, Vec2::new(min.x, max.y)),
        ],
        Vec3::Y,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::{CIRCUITS, Circuit};
    use crate::world::terrain::Ground;

    /// The circuits with fields of flowers, with their track and terrain.
    fn in_flower() -> Vec<(&'static Circuit, Track, Terrain)> {
        CIRCUITS
            .iter()
            .filter(|circuit| {
                circuit
                    .scenery
                    .japan
                    .as_ref()
                    .is_some_and(|japan| !japan.flowers.is_empty())
            })
            .map(|circuit| {
                let track = Track::new(&circuit.layout);
                let terrain = Terrain::new(&track, &circuit.scenery);
                (circuit, track, terrain)
            })
            .collect()
    }

    #[test]
    fn most_of_the_grass_is_in_flower_but_the_shoulders() {
        let circuits = in_flower();
        assert!(!circuits.is_empty());
        for (circuit, track, terrain) in &circuits {
            let (min, max) = track.bounds();
            let (mut grass, mut flowers) = (0, 0);
            let mut kinds = Vec::new();
            for z in (min.y as i32..max.y as i32).step_by(9) {
                for x in (min.x as i32..max.x as i32).step_by(9) {
                    let point = Vec2::new(x as f32, z as f32);
                    if terrain.ground_drawn(point) != Ground::Grass
                        || terrain.height(point) < SEA_LEVEL + 0.5
                    {
                        continue;
                    }
                    let nearest = track.nearest(point);
                    let wall = track.road(nearest.road).wall_at(nearest.index);
                    let flower = terrain.flowers_drawn(point);
                    if nearest.distance < wall {
                        assert_eq!(
                            flower, None,
                            "{}: flowers on the road at {point}",
                            circuit.name
                        );
                        continue;
                    }
                    grass += 1;
                    if let Some(kind) = flower {
                        flowers += 1;
                        if !kinds.contains(&kind) {
                            kinds.push(kind);
                        }
                    }
                }
            }
            let share = flowers as f32 / grass as f32;
            assert!(
                share > 0.6,
                "{}: {share} of the grass in flower",
                circuit.name
            );
            assert!(
                share < 0.9,
                "{}: grass between the fields: {share}",
                circuit.name
            );
            let japan = circuit.scenery.japan.as_ref().unwrap();
            for field in japan.flowers {
                for kind in field.kinds {
                    assert!(kinds.contains(kind), "{}: no {kind:?}", circuit.name);
                }
            }
        }
    }

    #[test]
    fn clumps_stand_near_the_roads_in_their_fields_and_not_too_many() {
        for (circuit, track, terrain) in &in_flower() {
            let clumps = clumps(track, terrain);
            assert!(clumps.len() > 2000, "{}: {}", circuit.name, clumps.len());
            assert!(clumps.len() < 14_000, "{}: {}", circuit.name, clumps.len());
            for clump in clumps.iter().step_by(11) {
                let point = Vec2::new(clump.root.x, clump.root.z);
                assert_eq!(terrain.flowers_drawn(point), Some(clump.kind));
                assert_ne!(clump.kind, FlowerKind::Phlox, "phlox hugs the ground");
                let nearest = track.nearest(point);
                let wall = track.road(nearest.road).wall_at(nearest.index);
                assert!(nearest.distance > wall + 1.0, "{point}");
                assert!(nearest.distance < CLUMP_REACH + 2.0, "{point}");
                assert!(
                    clump.root.y > SEA_LEVEL,
                    "{}: in the water at {point}",
                    circuit.name
                );
            }
        }
    }

    #[test]
    fn clumps_keep_clear_of_what_stands_on_the_ground() {
        let (_, track, terrain) = in_flower().remove(0);
        let all = clumps(&track, &terrain);
        let middle = all[all.len() / 2].root;
        let taken = Vec2::new(middle.x, middle.z);
        let clear = clear_of(all.clone(), &[(taken, 10.0)]);
        assert!(clear.len() < all.len());
        assert!(
            clear
                .iter()
                .all(|c| Vec2::new(c.root.x, c.root.z).distance(taken) >= 10.0 + CLEAR)
        );
    }

    #[test]
    fn each_square_of_clumps_is_one_mesh_of_three_cards_a_clump() {
        let (_, track, terrain) = in_flower().remove(0);
        let clumps = clumps(&track, &terrain);
        let meshes = clump_meshes(&clumps, &terrain);
        let vertices: usize = meshes.iter().map(|mesh| mesh.count_vertices()).sum();
        assert_eq!(vertices, clumps.len() * 3 * 4);
        for mesh in &meshes {
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|p| p.as_float3())
                .expect("positions");
            let (min, max) = positions.iter().fold(
                (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
                |(min, max), &[x, _, z]| (min.min(Vec2::new(x, z)), max.max(Vec2::new(x, z))),
            );
            // A square, and the cards reaching a little past it.
            assert!((max - min).max_element() < CHUNK + 2.0 * 1.8 * 1.2);
        }
    }

    #[test]
    fn moss_phlox_grows_in_bands_of_its_colours() {
        let colors: Vec<[f32; 4]> = (0..400)
            .map(|k| phlox_color(Vec2::new(k as f32 * 1.5, k as f32 * 2.0)))
            .collect();
        for color in PHLOX {
            let color = color.to_linear().to_f32_array();
            assert!(colors.contains(&color), "{color:?}");
        }
        // In bands: the colour changes now and then, not at every step.
        let changes = colors.windows(2).filter(|w| w[0] != w[1]).count();
        assert!(changes > 5 && changes < 100, "{changes}");
    }
}
