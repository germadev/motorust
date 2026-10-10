//! Terreno alrededor del circuito, inspirado en Speed Bay: junto a la
//! carretera sigue su altura y su peralte; más allá hay colinas de hierba,
//! peñascos que caen a pico sobre los muros, playas y el mar al oeste y al
//! sur, y montañas tierra adentro, al noreste.
//!
//! Se calcula una vez al arrancar sobre una rejilla: fina cerca del circuito
//! y cada vez más gruesa hacia el horizonte.

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::scenery::{Coast, Scenery, Side};
use super::textures::WorldTextures;
use crate::geometry::{fbm, smoothstep};
use crate::track::{Nearest, Track, WALL_DISTANCE};

pub const SEA_LEVEL: f32 = -1.0;
/// Hasta esta distancia de la línea central, el terreno es la prolongación de
/// la carretera (incluido el arcén hasta los muros).
const FLAT: f32 = WALL_DISTANCE + 1.5;
/// Lo que queda el terreno por debajo de la carretera, para que no asome.
const UNDER_ROAD: f32 = 0.03;
const FINE_SPACING: f32 = 3.5;
/// Margen alrededor del circuito con rejilla fina.
const FINE_MARGIN: f32 = 260.0;
/// Hasta dónde llega el terreno desde el centro del circuito.
const EXTENT: f32 = 3500.0;
/// Cuánto crece cada celda respecto a la anterior fuera de la zona fina.
const COARSE_GROWTH: f32 = 1.15;
/// Tamaño (m) que cubre cada repetición de las texturas.
const GRASS_TILE: f32 = 7.0;
const SAND_TILE: f32 = 6.0;
const ROCK_TILE: f32 = 9.0;
/// Las caras más inclinadas que esto son roca.
const CLIFF_NORMAL_Y: f32 = 0.8;
/// Por debajo de esta altura, cerca del mar, hay arena.
const BEACH_HEIGHT: f32 = SEA_LEVEL + 1.6;

/// Tipo de suelo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    Grass,
    Rock,
    Sand,
}

/// La altura del terreno muestreada en una rejilla rectangular (no uniforme).
#[derive(Resource)]
pub struct Terrain {
    xs: Vec<f32>,
    zs: Vec<f32>,
    /// Alturas por filas: `heights[z * xs.len() + x]`.
    heights: Vec<f32>,
}

impl Terrain {
    pub fn new(track: &Track, scenery: &Scenery) -> Self {
        let (min, max) = track.bounds();
        let center = (min + max) / 2.0;
        let xs = axis(min.x - FINE_MARGIN, max.x + FINE_MARGIN, center.x);
        let zs = axis(min.y - FINE_MARGIN, max.y + FINE_MARGIN, center.y);
        let fine = (min - FINE_MARGIN, max + FINE_MARGIN);
        let shaper = Shaper::new(track, scenery);

        let points: Vec<Vec2> = zs
            .iter()
            .flat_map(|&z| xs.iter().map(move |&x| Vec2::new(x, z)))
            .collect();
        let heights_of = |part: &[Vec2]| {
            part.iter()
                .map(|&point| {
                    let inside = point.cmpge(fine.0).all() && point.cmple(fine.1).all();
                    let nearest = inside.then(|| track.nearest(point));
                    shaper.height_at(point, nearest.as_ref())
                })
                .collect::<Vec<_>>()
        };
        // Lo más caro es buscar la carretera más cercana a cada punto: se
        // reparte entre los núcleos del procesador. En la web no hay hilos y
        // se hace todo en uno.
        let heights = if cfg!(target_arch = "wasm32") {
            heights_of(&points)
        } else {
            let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
            let chunk = points.len().div_ceil(threads);
            std::thread::scope(|scope| {
                let workers: Vec<_> = points
                    .chunks(chunk)
                    .map(|part| scope.spawn(|| heights_of(part)))
                    .collect();
                workers
                    .into_iter()
                    .flat_map(|worker| worker.join().expect("el cálculo del terreno no falla"))
                    .collect()
            })
        };
        Self { xs, zs, heights }
    }

    /// Altura del terreno en un punto (interpolada en la rejilla).
    pub fn height(&self, point: Vec2) -> f32 {
        let (i, tx) = locate(&self.xs, point.x);
        let (j, tz) = locate(&self.zs, point.y);
        let at = |i: usize, j: usize| self.heights[j * self.xs.len() + i];
        let bottom = at(i, j).lerp(at(i + 1, j), tx);
        let top = at(i, j + 1).lerp(at(i + 1, j + 1), tx);
        bottom.lerp(top, tz)
    }

    /// Normal del terreno en un punto.
    pub fn normal(&self, point: Vec2) -> Vec3 {
        const D: f32 = 1.5;
        let dx = self.height(point + Vec2::X * D) - self.height(point - Vec2::X * D);
        let dz = self.height(point + Vec2::Y * D) - self.height(point - Vec2::Y * D);
        Vec3::new(-dx, 2.0 * D, -dz).normalize()
    }

    pub fn ground(&self, point: Vec2) -> Ground {
        classify(self.height(point), self.normal(point))
    }

    /// Mallas del terreno, una por tipo de suelo.
    fn meshes(&self) -> [(Ground, Mesh); 3] {
        let (width, depth) = (self.xs.len(), self.zs.len());
        let position =
            |i: usize, j: usize| Vec3::new(self.xs[i], self.heights[j * width + i], self.zs[j]);
        let normals: Vec<Vec3> = (0..depth)
            .flat_map(|j| (0..width).map(move |i| (i, j)))
            .map(|(i, j)| {
                let (left, right) = (i.saturating_sub(1), (i + 1).min(width - 1));
                let (back, front) = (j.saturating_sub(1), (j + 1).min(depth - 1));
                let along_x = position(right, j) - position(left, j);
                let along_z = position(i, front) - position(i, back);
                along_z.cross(along_x).normalize()
            })
            .collect();

        let mut grass = Builder::shared(width * depth);
        let mut sand = Builder::shared(width * depth);
        let mut rock = Builder::default();
        for j in 0..depth - 1 {
            for i in 0..width - 1 {
                let corners = [(i, j), (i, j + 1), (i + 1, j), (i + 1, j + 1)];
                // Dos triángulos por celda, en sentido antihorario visto desde arriba.
                for triangle in [[0, 1, 2], [2, 1, 3]] {
                    let vertices = triangle.map(|k| corners[k]);
                    let [a, b, c] = vertices.map(|(i, j)| position(i, j));
                    let face = (b - a).cross(c - a).normalize();
                    let height = (a.y + b.y + c.y) / 3.0;
                    let builder = match classify(height, face) {
                        Ground::Grass => &mut grass,
                        Ground::Sand => &mut sand,
                        Ground::Rock => &mut rock,
                    };
                    for (i, j) in vertices {
                        let index = j * width + i;
                        let p = position(i, j);
                        let uv = match builder.shared_vertices {
                            Some(_) => Vec2::new(p.x, p.z),
                            None => rock_projection(p, face),
                        };
                        builder.push(index, p, normals[index], uv);
                    }
                }
            }
        }
        [
            (Ground::Grass, grass.build(GRASS_TILE)),
            (Ground::Sand, sand.build(SAND_TILE)),
            (Ground::Rock, rock.build(ROCK_TILE)),
        ]
    }
}

pub fn spawn_terrain(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    textures: &WorldTextures,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (ground, mesh) in terrain.meshes() {
        let (texture, roughness) = match ground {
            Ground::Grass => (&textures.grass, 0.95),
            Ground::Sand => (&textures.sand, 0.9),
            Ground::Rock => (&textures.rock, 0.85),
        };
        world.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(texture.clone()),
                perceptual_roughness: roughness,
                ..default()
            })),
        ));
    }

    // El mar: un plano que llega hasta el horizonte, liso y brillante.
    world.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0 * EXTENT, 4.0 * EXTENT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.27, 0.5),
            perceptual_roughness: 0.08,
            reflectance: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
    ));
}

fn classify(height: f32, normal: Vec3) -> Ground {
    if normal.y < CLIFF_NORMAL_Y {
        Ground::Rock
    } else if height < BEACH_HEIGHT {
        Ground::Sand
    } else {
        Ground::Grass
    }
}

/// Lo que da forma al terreno de una pista.
struct Shaper<'a> {
    track: &'a Track,
    scenery: &'a Scenery,
    lap: f32,
    /// Alturas de la carretera repartidas por el circuito, para un campo
    /// suave que no tenga escalones entre tramos a distinta altura.
    road_field: Vec<(Vec2, f32)>,
    /// En una isla, la costa: un polígono alrededor del circuito, y la caja
    /// que lo contiene.
    shore: Option<(Vec<Vec2>, Vec2, Vec2)>,
}

impl<'a> Shaper<'a> {
    fn new(track: &'a Track, scenery: &'a Scenery) -> Self {
        let road_field = track
            .centerline()
            .iter()
            .zip(track.elevations())
            .step_by(4)
            .map(|(&point, &height)| (point, height))
            .collect();
        let lap = track.lap_length();
        let shore = match scenery.coast {
            Coast::Function(_) => None,
            Coast::AroundTrack(margins) => {
                let polygon: Vec<Vec2> = (0..track.sample_count())
                    .step_by(2)
                    .map(|index| {
                        let along = index as f32 * track.step_length();
                        track.offset_point(index, interpolate(margins, lap, along))
                    })
                    .collect();
                let (min, max) = polygon.iter().fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(min, max), &p| (min.min(p), max.max(p)),
                );
                Some((polygon, min, max))
            }
        };
        Self {
            track,
            scenery,
            lap,
            road_field,
            shore,
        }
    }

    /// Distancia (con signo) a la costa: positiva tierra adentro.
    fn coast(&self, point: Vec2) -> f32 {
        match (&self.scenery.coast, &self.shore) {
            (Coast::Function(coast), _) => coast(point),
            (_, Some((polygon, min, max))) => {
                // Lejos de la isla, basta con la distancia a la caja.
                let outside = (*min - point).max(point - *max).max(Vec2::ZERO);
                if outside.length() > 150.0 {
                    return -outside.length();
                }
                signed_distance(polygon, point)
            }
            _ => unreachable!("una isla siempre tiene costa"),
        }
    }

    /// El terreno sin la carretera: playa, fondo del mar y colinas.
    fn natural(&self, point: Vec2) -> f32 {
        let land = self.coast(point);
        let shore = SEA_LEVEL - 4.0 + 5.6 * smoothstep(-30.0, 25.0, land);
        let hills = self.scenery.hills
            * (fbm(point / 160.0, 4) - 0.4).max(0.0)
            * smoothstep(10.0, 80.0, land);
        shore + hills
    }

    fn rocks(&self, point: Vec2) -> f32 {
        self.scenery
            .massifs
            .iter()
            .map(|massif| {
                let reach = point.distance(massif.center) / massif.radius;
                let shape = 1.0 - smoothstep(0.45, 1.0, reach);
                let rugged = 0.65 + 0.7 * fbm(point / 23.0 + massif.center / 100.0, 3);
                massif.height * shape * rugged
            })
            .fold(0.0, f32::max)
    }

    /// Altura de las paredes de roca junto a la carretera en un punto que
    /// está a `along` metros de la vuelta.
    fn cutting(&self, nearest: &Nearest, along: f32, point: Vec2) -> f32 {
        self.scenery
            .cuttings
            .iter()
            .filter(|cutting| (nearest.lateral > 0.0) == (cutting.side == Side::Left))
            .map(|cutting| {
                let inside = cutting.span.weight(along, self.lap, 30.0);
                // Una franja de roca de unos 40 m detrás del muro, con la cima llana.
                let band = 1.0 - smoothstep(FLAT + 30.0, FLAT + 75.0, nearest.distance);
                let rugged = 0.75 + 0.5 * fbm(point / 17.0, 3);
                cutting.height * inside * band * rugged
            })
            .fold(0.0, f32::max)
    }

    /// Altura del terreno en un punto. `nearest` es la carretera más cercana,
    /// o `None` si está tan lejos que no importa.
    fn height_at(&self, point: Vec2, nearest: Option<&Nearest>) -> f32 {
        let mountains = (self.scenery.mountains)(point);
        let Some(nearest) = nearest else {
            return self.natural(point) + self.rocks(point) + mountains;
        };
        let track = self.track;
        let along = track.along(nearest);
        let rock = self.rocks(point).max(self.cutting(nearest, along, point));
        let distance = nearest.distance;
        let road = track.height_beside(nearest) - UNDER_ROAD;
        let around_road = road.lerp(
            road_height_field(&self.road_field, point),
            smoothstep(FLAT, FLAT + 25.0, distance),
        );
        let mut blend = smoothstep(FLAT + 10.0, FLAT + 170.0, distance);
        if self.shore.is_some() {
            // En una isla la costa está cerca: el terreno baja hasta el mar
            // en cuanto se pasan los muros.
            let to_sea = smoothstep(35.0, 0.0, self.coast(point));
            blend = blend.max(to_sea * smoothstep(FLAT, FLAT + 6.0, distance));
        }
        let land = around_road.lerp(self.natural(point), blend);
        // La roca sube a pico justo detrás de los muros. En los túneles,
        // algo más lejos: la rejilla del terreno es más gruesa que el hueco
        // entre la pared y la roca, y sus triángulos asomarían dentro.
        let tunnel = self
            .scenery
            .tunnels
            .iter()
            .map(|span| span.weight(along, self.lap, 20.0))
            .fold(0.0, f32::max);
        let rise = smoothstep(FLAT, FLAT + 4.0, distance)
            .lerp(smoothstep(FLAT + 4.0, FLAT + 8.0, distance), tunnel);
        let mut height = land + rock * rise + mountains;
        // Explanadas para los edificios grandes, más allá de los muros.
        for terrace in self.scenery.terraces {
            let reach = 1.0
                - smoothstep(
                    terrace.radius,
                    terrace.radius + 15.0,
                    point.distance(terrace.center),
                );
            height = height.lerp(
                terrace.height,
                reach * smoothstep(FLAT, FLAT + 8.0, distance),
            );
        }
        // Bajo los puentes, el terreno sigue su forma natural (el mar de la
        // ría), siempre por debajo del tablero.
        let bridge = self
            .scenery
            .bridges
            .iter()
            .map(|span| span.weight(along, self.lap, 15.0))
            .fold(0.0, f32::max);
        if bridge > 0.0 {
            let wild = self.natural(point) + self.rocks(point) + mountains;
            height = height.lerp(wild.min(road - 2.5), bridge);
        }
        height
    }
}

/// El valor en `along` de una función a trozos dada por pares (metros desde
/// la meta, valor) a lo largo de una vuelta de `lap` metros.
pub fn interpolate(points: &[(f32, f32)], lap: f32, along: f32) -> f32 {
    let along = along.rem_euclid(lap);
    let n = points.len();
    let after = points.iter().position(|&(at, _)| at > along).unwrap_or(n);
    // Antes del primer punto y después del último se enlaza con la vuelta
    // anterior o la siguiente.
    let (a, b) = match after {
        0 => ((points[n - 1].0 - lap, points[n - 1].1), points[0]),
        _ if after == n => (points[n - 1], (points[0].0 + lap, points[0].1)),
        _ => (points[after - 1], points[after]),
    };
    let t = ((along - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    a.1.lerp(b.1, t)
}

/// Distancia con signo de un punto a un polígono cerrado: positiva dentro.
fn signed_distance(polygon: &[Vec2], point: Vec2) -> f32 {
    let mut distance = f32::INFINITY;
    let mut inside = false;
    let n = polygon.len();
    for i in 0..n {
        let (a, b) = (polygon[i], polygon[(i + 1) % n]);
        let ab = b - a;
        let t = ((point - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
        distance = distance.min(point.distance(a + ab * t));
        if (a.y > point.y) != (b.y > point.y)
            && point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x)
        {
            inside = !inside;
        }
    }
    if inside { distance } else { -distance }
}

/// Media de las alturas de la carretera ponderada por cercanía: suave en
/// todas partes y casi igual a la carretera cerca de ella.
fn road_height_field(road_field: &[(Vec2, f32)], point: Vec2) -> f32 {
    let (mut sum, mut weights) = (0.0, 0.0);
    for &(at, height) in road_field {
        let weight = 1.0 / (at.distance_squared(point) + 100.0).powi(2);
        sum += weight * height;
        weights += weight;
    }
    sum / weights
}

/// Coordenadas de una rejilla en un eje: cada `FINE_SPACING` m entre `from` y
/// `to`, y por fuera celdas cada vez mayores hasta `EXTENT` m de `center`.
fn axis(from: f32, to: f32, center: f32) -> Vec<f32> {
    let mut before = Vec::new();
    let (mut at, mut step) = (from, FINE_SPACING);
    while at > center - EXTENT {
        step *= COARSE_GROWTH;
        at -= step;
        before.push(at);
    }
    before.reverse();
    let fine = ((to - from) / FINE_SPACING).ceil() as usize;
    let mut coordinates = before;
    coordinates.extend((0..=fine).map(|i| from + i as f32 * FINE_SPACING));
    let (mut at, mut step) = (*coordinates.last().expect("hay coordenadas"), FINE_SPACING);
    while at < center + EXTENT {
        step *= COARSE_GROWTH;
        at += step;
        coordinates.push(at);
    }
    coordinates
}

/// Celda de la rejilla que contiene `value` y la posición dentro de ella (0..1).
fn locate(axis: &[f32], value: f32) -> (usize, f32) {
    let value = value.clamp(axis[0], axis[axis.len() - 1]);
    let i = axis
        .partition_point(|&a| a <= value)
        .clamp(1, axis.len() - 1)
        - 1;
    (i, (value - axis[i]) / (axis[i + 1] - axis[i]))
}

/// Coordenadas de textura de la roca: proyectada desde el lado al que más
/// mira cada cara, para que las paredes no salgan estiradas.
fn rock_projection(point: Vec3, face: Vec3) -> Vec2 {
    let abs = face.abs();
    if abs.y > 0.75 {
        Vec2::new(point.x, point.z)
    } else if abs.x > abs.z {
        Vec2::new(point.z, -point.y)
    } else {
        Vec2::new(point.x, -point.y)
    }
}

/// Acumula los triángulos de un tipo de suelo.
#[derive(Default)]
struct Builder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    /// Para compartir vértices entre triángulos: índice nuevo de cada vértice
    /// de la rejilla. La roca no los comparte, porque cada cara proyecta su
    /// textura desde un lado distinto.
    shared_vertices: Option<Vec<u32>>,
}

impl Builder {
    fn shared(grid_vertices: usize) -> Self {
        Self {
            shared_vertices: Some(vec![u32::MAX; grid_vertices]),
            ..default()
        }
    }

    fn push(&mut self, grid_index: usize, position: Vec3, normal: Vec3, uv: Vec2) {
        if let Some(shared) = &mut self.shared_vertices
            && shared[grid_index] != u32::MAX
        {
            self.indices.push(shared[grid_index]);
            return;
        }
        let index = self.positions.len() as u32;
        self.positions.push(position.to_array());
        self.normals.push(normal.to_array());
        self.uvs.push(uv.to_array());
        self.indices.push(index);
        if let Some(shared) = &mut self.shared_vertices {
            shared[grid_index] = index;
        }
    }

    fn build(self, tile: f32) -> Mesh {
        let uvs: Vec<[f32; 2]> = self.uvs.iter().map(|[u, v]| [u / tile, v / tile]).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    fn bay() -> (Track, Terrain) {
        let circuit = &CIRCUITS[0];
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        (track, terrain)
    }

    #[test]
    fn terrain_hugs_the_road_and_the_track_stays_dry() {
        for circuit in &CIRCUITS {
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let lap = track.lap_length();
            for index in (0..track.sample_count()).step_by(7) {
                let along = index as f32 * track.step_length();
                // En los puentes, debajo está la ría.
                if circuit
                    .scenery
                    .bridges
                    .iter()
                    .any(|b| b.contains(along, lap))
                {
                    continue;
                }
                for lateral in [-9.0, 0.0, 9.0] {
                    let road = track.surface_point(index, lateral);
                    let ground = terrain.height(Vec2::new(road.x, road.z));
                    let name = circuit.name;
                    // Por debajo del asfalto (si no, asomaría), pero muy cerca.
                    assert!(
                        ground < road.y + 0.02,
                        "el terreno tapa la carretera en {index} de {name}"
                    );
                    assert!(
                        ground > road.y - 0.4,
                        "hay un hueco bajo la carretera en {index} de {name}"
                    );
                    assert!(
                        road.y > SEA_LEVEL + 0.5,
                        "la carretera se moja en {index} de {name}"
                    );
                }
            }
        }
    }

    #[test]
    fn there_is_sea_to_the_west_and_mountains_to_the_north_east() {
        let (_, terrain) = bay();
        assert!(terrain.height(Vec2::new(-400.0, -200.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(1000.0, -1000.0)) > 50.0);
    }

    #[test]
    fn speed_bay_is_a_peninsula_with_an_inlet_under_the_bridge() {
        let circuit = &CIRCUITS[1];
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        // Mar más allá del paseo marítimo y tierra dentro del circuito.
        assert!(terrain.height(Vec2::new(-80.0, -100.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(-15.0, -100.0)) > SEA_LEVEL + 1.0);
        assert!(terrain.height(Vec2::new(230.0, -450.0)) > SEA_LEVEL + 1.0);
        // Bajo el centro del puente, agua.
        let bridge = circuit.scenery.bridges[0];
        let middle = track.index_at((bridge.from + bridge.to) / 2.0);
        let below = track.offset_point(middle, 0.0);
        assert!(
            terrain.height(below) < SEA_LEVEL,
            "{}",
            terrain.height(below)
        );
    }

    #[test]
    fn rock_walls_rise_beside_the_main_straight() {
        let (track, terrain) = bay();
        // A 200 m de la meta, la roca está a la derecha (-) y no a la izquierda.
        let index = (200.0 / track.step_length()) as usize;
        let road = track.surface_point(index, 0.0).y;
        let right = terrain.height(track.offset_point(index, -(FLAT + 15.0)));
        let left = terrain.height(track.offset_point(index, FLAT + 15.0));
        assert!(right - road > 8.0, "pared de {} m", right - road);
        assert!(left - road < 3.0);
    }

    #[test]
    fn piecewise_values_wrap_around_the_lap() {
        let points = [(100.0, 10.0), (500.0, 20.0)];
        assert_eq!(interpolate(&points, 1000.0, 300.0), 15.0);
        // Después del último, hacia el primero de la vuelta siguiente.
        assert_eq!(interpolate(&points, 1000.0, 800.0), 15.0);
        assert!((interpolate(&points, 1000.0, 50.0) - 10.833).abs() < 0.01);
    }

    #[test]
    fn signed_distance_is_positive_inside() {
        let square = [
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        assert_eq!(signed_distance(&square, Vec2::new(5.0, 3.0)), 3.0);
        assert_eq!(signed_distance(&square, Vec2::new(5.0, -2.0)), -2.0);
    }

    #[test]
    fn axis_is_fine_inside_and_coarse_outside() {
        let axis = axis(-100.0, 100.0, 0.0);
        assert!(axis.windows(2).all(|w| w[1] > w[0]));
        assert!(axis[0] <= -EXTENT && *axis.last().unwrap() >= EXTENT);
        let (i, _) = locate(&axis, 0.0);
        assert!((axis[i + 1] - axis[i] - FINE_SPACING).abs() < 1e-3);
    }
}
