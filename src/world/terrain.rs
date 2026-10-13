//! Terreno alrededor del circuito, inspirado en Speed Bay (Moto Racer): junto a la
//! carretera sigue su altura y su peralte; más allá hay colinas de hierba,
//! peñascos que caen a pico sobre los muros, playas y el mar al oeste y al
//! sur, y montañas tierra adentro, al noreste.
//!
//! Se calcula una vez al arrancar sobre una rejilla: fina cerca del circuito
//! y cada vez más gruesa hacia el horizonte.

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::scenery::{Coast, Scenery, Side, Soil, span};
use super::structures::{TUNNEL_SHELL, Tunnels};
use super::textures::WorldTextures;
use crate::geometry::{fbm, smoothstep, value_noise};
use crate::track::{Nearest, Track, WALL_DISTANCE, closed_curve};

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
pub const GRASS_TILE: f32 = 11.0;
const SAND_TILE: f32 = 8.5;
const ROCK_TILE: f32 = 20.0;
/// Size (m) of each repetition of the checkerboard earth: two by two squares.
const SOIL_TILE: f32 = 7.0;
/// Las caras más inclinadas que esto son roca.
pub const CLIFF_NORMAL_Y: f32 = 0.8;
/// Por debajo de esta altura, cerca del mar, hay arena.
const BEACH_HEIGHT: f32 = SEA_LEVEL + 1.6;
/// Where a road passes over a tunnel, the ground stays down at the tunnel
/// this far from its centre line (m): short of the edge of its rock, so the
/// slope up to the road above is under it too.
const BURIED: f32 = TUNNEL_SHELL - 2.0 * FINE_SPACING;
/// How far apart the points of a shore drawn as an outline are (m).
const SHORE_SPACING: f32 = 4.0;
/// How far above the road the earth must stand to stop a bike (m), and the
/// steps in which [`Terrain::track_edges`] looks for it (m).
const EARTH_RISE: f32 = 0.3;
const EDGE_STEP: f32 = 0.1;

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
    /// What the steep faces are made of.
    soil: Soil,
}

impl Terrain {
    /// The terrain of a circuit, worked out on every core.
    #[cfg(test)]
    pub fn new(track: &Track, scenery: &Scenery) -> Self {
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        Self::on_threads(track, scenery, cores)
    }

    /// The terrain of a circuit, worked out on at most `threads` threads.
    pub fn on_threads(track: &Track, scenery: &Scenery, threads: usize) -> Self {
        let (min, max) = track.bounds();
        let center = (min + max) / 2.0;
        let xs = axis(min.x - FINE_MARGIN, max.x + FINE_MARGIN, center.x);
        let zs = axis(min.y - FINE_MARGIN, max.y + FINE_MARGIN, center.y);
        let fine = (min - FINE_MARGIN, max + FINE_MARGIN);
        let shaper = Shaper::new(track, scenery);
        let tunnels = Tunnels::new(track, scenery);

        let points: Vec<Vec2> = zs
            .iter()
            .flat_map(|&z| xs.iter().map(move |&x| Vec2::new(x, z)))
            .collect();
        let heights_of = |part: &[Vec2]| {
            part.iter()
                .map(|&point| {
                    let inside = point.cmpge(fine.0).all() && point.cmple(fine.1).all();
                    if !inside {
                        return shaper.height_at(point, None);
                    }
                    // Under the road that passes over a tunnel, the ground
                    // stays down at the tunnel, under its rock.
                    if let Some(tunnel) = tunnels.beneath(track, point, BURIED) {
                        return track.height_beside(&tunnel) - UNDER_ROAD;
                    }
                    shaper.height_at(point, Some(&track.nearest(point)))
                })
                .collect::<Vec<_>>()
        };
        // Lo más caro es buscar la carretera más cercana a cada punto: se
        // reparte entre los núcleos del procesador. En la web no hay hilos y
        // se hace todo en uno.
        let heights = if cfg!(target_arch = "wasm32") || threads <= 1 {
            heights_of(&points)
        } else {
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
        Self {
            xs,
            zs,
            heights,
            soil: scenery.soil,
        }
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

    /// Where the earth rises beside the road, at each point of the centre
    /// line, to the left and to the right (m from it): the first spot, going
    /// out from the edge of the asphalt, where the terrain as drawn (the
    /// triangles of its grid, which can poke inside the walls next to a rock
    /// face) stands [`EARTH_RISE`] above the road, or [`WALL_DISTANCE`] if it
    /// does not before the walls. Looks half way to the next points too, so
    /// a bump between two of them is not missed. The lap first, then each
    /// road off it, short of its own walls. See [`Track::set_edges`].
    pub fn track_edges(&self, track: &Track) -> Vec<Vec<[f32; 2]>> {
        std::iter::once(track)
            .chain(track.side_roads())
            .map(|road| self.road_edges(road))
            .collect()
    }

    /// Like [`Self::track_edges`], for one road.
    fn road_edges(&self, track: &Track) -> Vec<[f32; 2]> {
        let n = track.sample_count();
        let steps = ((WALL_DISTANCE - track.half_width()) / EDGE_STEP).ceil() as usize;
        (0..n)
            .map(|index| {
                let wall = track.wall_at(index);
                [1.0, -1.0].map(|side| {
                    (0..=steps)
                        .map(|k| (track.half_width() + k as f32 * EDGE_STEP).min(wall))
                        .find(|&lateral| {
                            [-1, 0, 1].iter().any(|&offset| {
                                let other = index as isize + offset;
                                let other = if track.is_closed() {
                                    other.rem_euclid(n as isize) as usize
                                } else {
                                    other.clamp(0, n as isize - 1) as usize
                                };
                                let here = track.surface_point(index, side * lateral);
                                let there = track.surface_point(other, side * lateral);
                                // Here and half way to the neighbour.
                                [here, (here + there) / 2.0].iter().any(|road| {
                                    self.height(Vec2::new(road.x, road.z)) - road.y > EARTH_RISE
                                })
                            })
                        })
                        .unwrap_or(wall)
                })
            })
            .collect()
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
    /// The ground's meshes, by kind of ground. Pure data: they can be worked
    /// out away from the main thread.
    pub fn meshes(&self) -> [(Ground, Mesh); 3] {
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
                    let ground_type = classify(height, face);
                    let builder = match ground_type {
                        Ground::Grass => &mut grass,
                        Ground::Sand => &mut sand,
                        Ground::Rock => &mut rock,
                    };
                    for (i, j) in vertices {
                        let index = j * width + i;
                        let p = position(i, j);
                        let uv = match builder.shared_vertices {
                            Some(_) => {
                                let warp = Vec2::new(
                                    fbm(Vec2::new(p.x, p.z) * 0.035, 2) - 0.5,
                                    fbm(Vec2::new(p.z + 41.0, p.x - 29.0) * 0.035, 2) - 0.5,
                                ) * 2.2;
                                Vec2::new(p.x + warp.x, p.z + warp.y)
                            }
                            // Checkerboard earth: straight rows at the same
                            // heights on every cliff, as in Loop Island.
                            None if self.soil == Soil::Checkered => rock_projection(p, face),
                            None => {
                                // Continuous 3D domain warp based on world coordinates before projection.
                                // Warping in 3D breaks world-elevation synchronicity and introduces
                                // natural undulating strata following the mountain curvature.
                                let warp_x =
                                    fbm(Vec2::new(p.z * 0.022 + 13.0, p.y * 0.032 - 7.0), 3) - 0.5;
                                let warp_y =
                                    fbm(Vec2::new(p.x * 0.025 - 29.0, p.z * 0.025 + 41.0), 3) - 0.5;
                                let warp_z =
                                    fbm(Vec2::new(p.x * 0.022 + 37.0, p.y * 0.032 + 19.0), 3) - 0.5;
                                let warped_p =
                                    p + Vec3::new(warp_x * 9.0, warp_y * 7.5, warp_z * 9.0);

                                let proj = rock_projection(warped_p, face);

                                // Apply 14-degree geological dip angle so rock layers follow a realistic tilt
                                let dip_angle = 0.244f32;
                                let cos_dip = dip_angle.cos();
                                let sin_dip = dip_angle.sin();
                                let tilted_u = proj.x * cos_dip - proj.y * sin_dip;
                                let tilted_v = proj.x * sin_dip + proj.y * cos_dip;

                                let micro_warp = Vec2::new(
                                    fbm(Vec2::new(tilted_u, tilted_v) * 0.05, 2) - 0.5,
                                    fbm(Vec2::new(tilted_v + 50.0, tilted_u - 30.0) * 0.05, 2)
                                        - 0.5,
                                ) * 2.5;

                                Vec2::new(tilted_u + micro_warp.x, tilted_v + micro_warp.y)
                            }
                        };
                        let color = match (self.soil, ground_type) {
                            (Soil::Checkered, Ground::Rock) => soil_vertex_color(p, normals[index]),
                            (Soil::Mine, _) => canyon_vertex_color(ground_type, p, normals[index]),
                            _ => ground_vertex_color(ground_type, p, normals[index]),
                        };
                        builder.push(index, p, normals[index], uv, color);
                    }
                }
            }
        }
        [
            (Ground::Grass, grass.build(GRASS_TILE)),
            (Ground::Sand, sand.build(SAND_TILE)),
            (
                Ground::Rock,
                rock.build(match self.soil {
                    Soil::Rock | Soil::Mine => ROCK_TILE,
                    Soil::Checkered => SOIL_TILE,
                }),
            ),
        ]
    }
}

/// The ground, from its meshes ([`Terrain::meshes`]). The sea is apart
/// ([`super::sea`]).
pub fn spawn_terrain(
    world: &mut ChildSpawnerCommands,
    terrain: &Terrain,
    ground_meshes: [(Ground, Mesh); 3],
    textures: &WorldTextures,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (ground, mesh) in ground_meshes {
        let (texture, roughness, reflectance) = match (ground, terrain.soil) {
            // In a desert canyon the flat ground is dry, dusty earth.
            (Ground::Grass, Soil::Mine) => (&textures.sand, 0.95, 0.15),
            (Ground::Grass, _) => (&textures.grass, 0.90, 0.18),
            (Ground::Sand, _) => (&textures.sand, 0.88, 0.28),
            (Ground::Rock, Soil::Rock | Soil::Mine) => (&textures.rock, 0.78, 0.42),
            (Ground::Rock, Soil::Checkered) => (&textures.soil, 0.92, 0.2),
        };
        world.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(texture.clone()),
                perceptual_roughness: roughness,
                reflectance,
                ..default()
            })),
        ));
    }
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
        let road_field = std::iter::once(track)
            .chain(track.side_roads())
            .flat_map(|road| road.centerline().iter().zip(road.elevations()).step_by(4))
            .map(|(&point, &height)| (point, height))
            .collect();
        let lap = track.lap_length();
        let polygon: Option<Vec<Vec2>> = match scenery.coast {
            Coast::Function(_) => None,
            Coast::AroundTrack(margins) => Some(
                (0..track.sample_count())
                    .step_by(2)
                    .map(|index| {
                        let along = index as f32 * track.step_length();
                        track.offset_point(index, interpolate(margins, lap, along))
                    })
                    .collect(),
            ),
            Coast::Outline(points) => Some(closed_curve(points, SHORE_SPACING)),
        };
        let shore = polygon.map(|polygon| {
            let (min, max) = polygon.iter().fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), &p| (min.min(p), max.max(p)),
            );
            (polygon, min, max)
        });
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
        // A mine's excavation has no walls of its own past the bikes' (see
        // `cavern`): the rock keeps clear of them all the way to its mouths.
        let past = if self.scenery.soil == Soil::Mine {
            20.0
        } else {
            0.0
        };
        let tunnel = self
            .scenery
            .tunnels
            .iter()
            .map(|tunnel| span(tunnel.from - past, tunnel.to + past).weight(along, self.lap, 20.0))
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

/// Rock texture coordinates: planar projection from dominant face normal
/// with continuous coordinate orientation.
fn rock_projection(point: Vec3, face: Vec3) -> Vec2 {
    let abs = face.abs();
    if abs.y > 0.75 {
        Vec2::new(point.x, point.z)
    } else if abs.x > abs.z {
        let u = if face.x > 0.0 { point.z } else { -point.z };
        Vec2::new(u, -point.y)
    } else {
        let u = if face.z > 0.0 { -point.x } else { point.x };
        Vec2::new(u, -point.y)
    }
}

pub fn ground_vertex_color(ground: Ground, p: Vec3, normal: Vec3) -> [f32; 4] {
    let color = match ground {
        Ground::Grass => {
            // Natural macro variation across tens of meters (bright meadows, shaded hollows, golden knolls)
            let macro_n = fbm(Vec2::new(p.x * 0.012, p.z * 0.012), 3);
            let detail_n = value_noise(Vec2::new(p.x * 0.045 + 17.0, p.z * 0.045 - 23.0));
            let steepness = (1.0 - normal.y).clamp(0.0, 1.0);

            // Three harmonious arcade tones:
            // 1. Vibrant lush meadow green
            let lush = Color::srgb(0.92, 1.05, 0.88);
            // 2. Warm golden sunny slope green
            let sunny = Color::srgb(1.05, 1.01, 0.85);
            // 3. Deep green natural shade
            let deep = Color::srgb(0.84, 0.93, 0.82);

            let base_tint = if macro_n > 0.5 {
                lush.mix(&sunny, (macro_n - 0.5) * 2.0)
            } else {
                deep.mix(&lush, macro_n * 2.0)
            };

            // Subtle micro-relief accent and drier embankments on steeper inclines
            let earthy = Color::srgb(1.02, 0.95, 0.82);
            base_tint.mix(
                &earthy,
                (steepness * 1.8 + (detail_n - 0.5) * 0.3).clamp(0.0, 0.55),
            )
        }
        Ground::Rock => {
            // 1. Macro massif geological formation (scale ~120m):
            // Regional variations across mountain ranges:
            // - Warm Mediterranean ochre/terracotta sandstone
            // - Sun-baked golden coastal limestone
            // - Dark volcanic slate/basalt
            let massif = fbm(Vec2::new(p.x * 0.007 + 100.0, p.z * 0.007 - 100.0), 3);
            let mineral_patch = fbm(Vec2::new(p.x * 0.018 - 45.0, p.z * 0.018 + 75.0), 2);

            let golden_limestone = Color::srgb(1.14, 1.06, 0.92);
            let ochre_sandstone = Color::srgb(1.18, 0.94, 0.80);
            let slate_basalt = Color::srgb(0.76, 0.80, 0.88);
            let pale_granite = Color::srgb(0.96, 0.96, 0.94);

            let base_rock = if massif > 0.55 {
                let t = (massif - 0.55) / 0.45;
                golden_limestone.mix(&ochre_sandstone, t)
            } else if massif < 0.45 {
                let t = (0.45 - massif) / 0.45;
                pale_granite.mix(&slate_basalt, t)
            } else {
                let t = (massif - 0.45) * 10.0;
                pale_granite.mix(&golden_limestone, t)
            };

            // 2. Tilted sedimentary strata layers (scale ~6m - 14m):
            let dip_plane = p.y * 0.28 + (p.x * 0.08 - p.z * 0.06);
            let strata_noise = fbm(Vec2::new(p.x * 0.02, p.z * 0.02), 2) * 1.8;
            let strata_phase = (dip_plane + strata_noise).sin();
            let strata_factor = (strata_phase * 0.5 + 0.5).powf(1.4);

            let strata_dark = Color::srgb(0.70, 0.68, 0.72);
            let strata_light = Color::srgb(1.18, 1.12, 1.04);
            let layered = if strata_factor > 0.5 {
                base_rock.mix(&strata_light, (strata_factor - 0.5) * 0.75)
            } else {
                base_rock.mix(&strata_dark, (0.5 - strata_factor) * 0.65)
            };

            // 3. Directional sun exposure & ambient occlusion in crevices:
            let sun_dir = Vec3::new(0.55, 0.82, -0.35).normalize();
            let sun_dot = normal.dot(sun_dir).clamp(-1.0, 1.0);
            let sun_highlight = (sun_dot * 0.5 + 0.5).powf(1.3);
            let ao = (0.52 + 0.48 * normal.y.max(0.0)).clamp(0.52, 1.0);

            // 4. Subtle moss and lichen on sheltered rock ledges:
            let moss_noise = fbm(Vec2::new(p.x * 0.035 - 15.0, p.z * 0.035 + 30.0), 2);
            let mossy = Color::srgb(0.76, 0.88, 0.68);
            let sheltered = (normal.y > 0.25 && normal.y < 0.82 && moss_noise > 0.56) as i32 as f32;
            let moss_tint = (moss_noise - 0.56).max(0.0) * 1.5 * sheltered;

            // 5. Mineral patch accents:
            let mineral_dark = Color::srgb(0.85, 0.82, 0.80);
            let with_minerals = layered.mix(&mineral_dark, (mineral_patch * 0.5).clamp(0.0, 0.4));

            let lit = with_minerals.mix(&Color::srgb(1.22, 1.16, 1.04), sun_highlight * 0.35);
            let shaded = lit.mix(&Color::srgb(0.56, 0.56, 0.62), (1.0 - ao) * 0.65);
            shaded.mix(&mossy, moss_tint.clamp(0.0, 0.45))
        }
        Ground::Sand => {
            // Moisture gradient near sea level and windblown dune variations
            let moisture = ((BEACH_HEIGHT - p.y) / (BEACH_HEIGHT - SEA_LEVEL)).clamp(0.0, 1.0);
            let dune = value_noise(Vec2::new(p.x * 0.05, p.z * 0.05));

            let dry_sand = Color::srgb(1.03, 1.01, 0.95);
            let wet_sand = Color::srgb(0.78, 0.74, 0.68);
            let dune_tint = Color::srgb(1.05, 0.98, 0.90);

            let beach = dry_sand.mix(&dune_tint, (dune - 0.5) * 0.3);
            beach.mix(&wet_sand, (moisture * 1.1).clamp(0.0, 1.0))
        }
    };
    color.to_linear().to_f32_array()
}

/// The colours of a desert canyon, over the textures: dry red-brown earth on
/// the flat, and on the faces bands of red, rust and ochre rock, like the
/// mesas of the American south-west, darker in the crevices.
fn canyon_vertex_color(ground: Ground, p: Vec3, normal: Vec3) -> [f32; 4] {
    let color = match ground {
        Ground::Grass => {
            let patch = fbm(Vec2::new(p.x * 0.015, p.z * 0.015), 3);
            let grit = value_noise(Vec2::new(p.x * 0.06 + 11.0, p.z * 0.06 - 5.0));
            let dust = Color::srgb(1.12, 0.78, 0.58);
            let red_earth = Color::srgb(1.0, 0.62, 0.45);
            let pale = Color::srgb(1.15, 0.9, 0.7);
            let earth = red_earth.mix(&dust, patch);
            earth.mix(
                &pale,
                ((grit - 0.5) * 0.6 + (1.0 - normal.y) * 1.5).clamp(0.0, 0.5),
            )
        }
        Ground::Rock => {
            // Horizontal strata, as in a canyon cut by a river, a little
            // wavy.
            let wave = fbm(Vec2::new(p.x * 0.01, p.z * 0.01), 2) * 6.0;
            let band = ((p.y + wave) * 0.45).sin() * 0.5 + 0.5;
            let thin = ((p.y + wave) * 1.7).sin() * 0.5 + 0.5;
            let red = Color::srgb(1.05, 0.58, 0.42);
            let rust = Color::srgb(0.88, 0.5, 0.36);
            let ochre = Color::srgb(1.2, 0.86, 0.58);
            let layered = if band > 0.5 {
                red.mix(&ochre, (band - 0.5) * 1.6)
            } else {
                red.mix(&rust, (0.5 - band) * 1.6)
            };
            let streaked = layered.mix(&Color::srgb(1.1, 0.95, 0.8), thin.powi(6) * 0.35);
            let ao = (0.55 + 0.45 * normal.y.max(0.0)).clamp(0.55, 1.0);
            streaked.mix(&Color::srgb(0.5, 0.36, 0.34), (1.0 - ao) * 0.6)
        }
        // By the river: damp, darker sand.
        Ground::Sand => {
            let dune = value_noise(Vec2::new(p.x * 0.05, p.z * 0.05));
            Color::srgb(0.95, 0.78, 0.62).mix(&Color::srgb(0.8, 0.66, 0.55), dune * 0.5)
        }
    };
    color.to_linear().to_f32_array()
}

/// Checkerboard earth only needs light and shade: its colours are in the
/// texture. Faces towards the sun are lighter and a little warmer, faces
/// away from it darker, with broad, faint patches.
fn soil_vertex_color(p: Vec3, normal: Vec3) -> [f32; 4] {
    let sun = Vec3::new(0.55, 0.82, -0.35).normalize();
    let lit = (normal.dot(sun) * 0.5 + 0.5).clamp(0.0, 1.0);
    let patch = fbm(Vec2::new(p.x, p.z) * 0.02, 2);
    let shade = 0.78 + 0.32 * lit + 0.1 * (patch - 0.5);
    Color::srgb(shade * 1.04, shade, shade * 0.96)
        .to_linear()
        .to_f32_array()
}

/// Acumula los triángulos de un tipo de suelo.
#[derive(Default)]
struct Builder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
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

    fn push(&mut self, grid_index: usize, position: Vec3, normal: Vec3, uv: Vec2, color: [f32; 4]) {
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
        self.colors.push(color);
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
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
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
            let tunnels = Tunnels::new(&track, &circuit.scenery);
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
                    // Over a tunnel, the road rests on its rock.
                    if tunnels
                        .beneath(&track, Vec2::new(road.x, road.z), TUNNEL_SHELL)
                        .is_some_and(|tunnel| track.height_beside(&tunnel) < road.y - 5.0)
                    {
                        continue;
                    }
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
    fn the_track_edge_is_where_the_earth_rises_inside_the_walls() {
        for circuit in &CIRCUITS {
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let edges = terrain.track_edges(&track);
            assert_eq!(edges.len(), 1 + track.side_roads().len());
            let edges = &edges[0];
            assert_eq!(edges.len(), track.sample_count());
            let mut inside = 0;
            for (index, sides) in edges.iter().enumerate() {
                for (edge, side) in sides.iter().zip([1.0, -1.0]) {
                    assert!((track.half_width()..=WALL_DISTANCE).contains(edge));
                    inside += usize::from(*edge < WALL_DISTANCE);
                    // Short of the edge, the earth is not above the road.
                    let road = track.surface_point(index, side * (edge - 0.15));
                    let earth = terrain.height(Vec2::new(road.x, road.z));
                    assert!(earth - road.y <= EARTH_RISE, "{} {index}", circuit.name);
                }
            }
            // Next to the rock faces, the earth comes in before the walls.
            assert!(inside > 0, "{}", circuit.name);
        }
    }

    #[test]
    fn there_is_sea_to_the_west_and_mountains_to_the_north_east() {
        let (_, terrain) = bay();
        assert!(terrain.height(Vec2::new(-400.0, -200.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(1000.0, -1000.0)) > 50.0);
    }

    #[test]
    fn turbo_bay_is_an_island_with_a_river_under_the_bridge() {
        let circuit = &CIRCUITS[1];
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        // Sea beyond the beach of the seafront, and land inside both loops
        // of the figure of eight.
        assert!(terrain.height(Vec2::new(-90.0, -100.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(-22.0, -100.0)) > SEA_LEVEL + 1.0);
        assert!(terrain.height(Vec2::new(90.0, -150.0)) > SEA_LEVEL + 1.0);
        assert!(terrain.height(Vec2::new(420.0, -500.0)) > SEA_LEVEL + 1.0);
        // The bays between the loops, north and south of the waist.
        assert!(terrain.height(Vec2::new(120.0, -520.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(330.0, -60.0)) < SEA_LEVEL);
        // The river runs in under the middle of the bridge.
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
    fn turbo_bay_passes_over_its_own_tunnel() {
        let circuit = &CIRCUITS[1];
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        let tunnels = Tunnels::new(&track, &circuit.scenery);
        // Where the stretch coming down from the forest crosses the tunnel.
        let lap = track.lap_length();
        let tunnel = circuit.scenery.tunnels[0];
        let (below, above) = (0..track.sample_count())
            .filter(|&i| tunnel.contains(i as f32 * track.step_length(), lap))
            .flat_map(|i| (0..track.sample_count()).map(move |j| (i, j)))
            .filter(|&(i, j)| {
                !tunnel.contains(j as f32 * track.step_length(), lap)
                    && (i as f32 - j as f32).abs() > 200.0
            })
            .min_by(|&(a, b), &(c, d)| {
                let gap =
                    |i: usize, j: usize| track.centerline()[i].distance(track.centerline()[j]);
                gap(a, b).total_cmp(&gap(c, d))
            })
            .expect("the track crosses the tunnel");
        let point = track.centerline()[below];
        assert!(point.distance(track.centerline()[above]) < 2.0);
        let (low, high) = (track.elevations()[below], track.elevations()[above]);
        assert!(high - low > 14.0, "{low} m under {high} m");
        // On each level, the track is the road at that level.
        assert_eq!(track.nearest_index_at(point, low), below);
        assert_eq!(track.nearest_index_at(point, high), above);
        assert!((track.road_height_at(point, low) - low).abs() < 0.5);
        assert!((track.road_height_at(point, high) - high).abs() < 0.5);
        // The ground stays down at the tunnel, under the rock that carries
        // the road above.
        assert!(tunnels.beneath(&track, point, BURIED).is_some());
        assert!((terrain.height(point) - low).abs() < 0.1);
    }

    #[test]
    fn loop_island_is_an_island_of_checkerboard_cliffs() {
        let circuit = &CIRCUITS[2];
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        assert_eq!(terrain.soil, Soil::Checkered);
        // Sea all round, beyond the beach of the south shore and offshore
        // to the west, and the sea comes in under the middle of the bridge.
        assert!(terrain.height(Vec2::new(200.0, 260.0)) < SEA_LEVEL);
        assert!(terrain.height(Vec2::new(-300.0, -200.0)) < SEA_LEVEL);
        let bridge = circuit.scenery.bridges[0];
        let middle = track.index_at((bridge.from + bridge.to) / 2.0);
        assert!(terrain.height(track.offset_point(middle, 0.0)) < SEA_LEVEL);
        // The walls of the tunnel stand well above the road.
        let tunnel = circuit.scenery.tunnels[0];
        let index = track.index_at((tunnel.from + tunnel.to) / 2.0);
        let road = track.surface_point(index, 0.0).y;
        for side in [-1.0, 1.0] {
            let wall = terrain.height(track.offset_point(index, side * (FLAT + 15.0)));
            assert!(wall - road > 10.0, "{} m", wall - road);
        }
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
