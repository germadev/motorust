//! El trazado del circuito: su línea central y las consultas que necesitan
//! la física, la IA y el escenario. No sabe nada de render, así se puede
//! probar sin abrir una ventana.

use bevy::prelude::*;

use crate::geometry::{heading_of, wrap_angle};

/// Altura del asfalto sobre la hierba (si estuvieran a la misma, parpadearían).
pub const SURFACE_Y: f32 = 0.05;

/// El trazado de una pista.
pub struct Layout {
    /// Puntos de control, en metros sobre el plano XZ y en orden de carrera.
    /// La pista es una curva Catmull-Rom cerrada que pasa por todos ellos.
    /// La salida está en el primero y la recta de meta va hacia -Z.
    pub control_points: &'static [Vec2],
    /// Altimetría: altura de la carretera (m) a lo largo de la vuelta (m
    /// desde la línea de meta). Entre puntos se interpola con suavidad.
    pub elevation: &'static [(f32, f32)],
    /// Tramos con más (o menos) peralte del que les toca por su curva.
    pub banking: &'static [Banking],
}

/// Un tramo (m desde la meta) en el que el peralte se multiplica por
/// `factor`, sin pasar del máximo.
pub struct Banking {
    pub from: f32,
    pub to: f32,
    pub factor: f32,
}

/// Altura de la línea de meta sobre el nivel del mar: el resto del perfil va
/// por encima, así ni el lado bajo de los peraltes toca el agua.
const BASE_ELEVATION: f32 = 2.0;
/// Peralte (rad) por cada unidad de curvatura (1/m): cuanto más cerrada la
/// curva, más inclinada hacia dentro está la carretera.
const BANK_PER_CURVATURE: f32 = 7.0;
const MAX_BANK: f32 = 0.17;
/// Puntos a cada lado con los que se suaviza el peralte.
const BANK_SMOOTHING: usize = 10;

const TRACK_WIDTH: f32 = 14.0;
pub const CURB_WIDTH: f32 = 1.2;
/// Distancia de la línea central a la cara interior de los muros que
/// bordean la carretera: lo que se puede salir uno de la pista.
pub const WALL_DISTANCE: f32 = 11.5;
/// Separación entre puntos de la línea central; también el largo de cada
/// franja de los pianos.
pub const SAMPLE_SPACING: f32 = 2.0;
/// Cuántos puntos por detrás de la línea de meta está la primera fila de la parrilla.
const START_BEHIND_SAMPLES: usize = 5;
/// Distancia entre filas de la parrilla, en puntos del trazado.
const GRID_ROW_SAMPLES: usize = 5;
/// Parrilla como la de los Grandes Premios: tres motos por fila, escalonadas (cada una un
/// punto del trazado más atrás que la de su izquierda), a estos metros a la
/// izquierda de la línea central.
const GRID_LANES: [f32; 3] = [4.5, 0.0, -4.5];

/// Tipo de suelo bajo un punto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// Asfalto o piano.
    Asphalt,
    Grass,
}

/// Datos estáticos del circuito.
#[derive(Resource)]
pub struct Track {
    /// Línea central equiespaciada, en orden de carrera; el último punto
    /// enlaza con el primero.
    centerline: Vec<Vec2>,
    /// Altura de la carretera en cada punto de la línea central.
    elevation: Vec<f32>,
    /// Peralte en cada punto (rad): positivo si el lado derecho está más alto,
    /// como en una curva a la izquierda.
    bank: Vec<f32>,
    half_width: f32,
    /// Distancia entre puntos de la línea central (ver [`Self::step_length`]).
    step: f32,
    /// Where the solid edge is at each point of the centre line, to the left
    /// and to the right (m from it): the walls, or the earth where it rises
    /// before them (see [`Self::set_edges`]). Empty: the walls everywhere.
    edges: Vec<[f32; 2]>,
}

/// El punto de la línea central más cercano a otro.
#[derive(Clone, Copy, Debug)]
pub struct Nearest {
    pub point: Vec2,
    pub distance: f32,
    /// Tramo de la línea central en el que está (de `index` a `index + 1`) y
    /// por dónde de él (0..1).
    pub index: usize,
    pub t: f32,
    /// Distancia con signo a la línea central: positiva a la izquierda.
    pub lateral: f32,
}

impl Default for Track {
    /// La primera pista, la Bahía.
    fn default() -> Self {
        Self::new(&crate::circuits::CIRCUITS[0].layout)
    }
}

impl Track {
    pub fn new(layout: &Layout) -> Self {
        let mut track = Self {
            centerline: resample(&sample_spline(layout.control_points), SAMPLE_SPACING),
            elevation: Vec::new(),
            bank: Vec::new(),
            half_width: TRACK_WIDTH / 2.0,
            step: 0.0,
            edges: Vec::new(),
        };
        track.step = track.segments().map(|(a, b)| a.distance(b)).sum::<f32>()
            / track.centerline.len() as f32;
        let (n, step) = (track.centerline.len(), track.step);
        let lap = n as f32 * step;
        track.elevation = (0..n)
            .map(|i| BASE_ELEVATION + profile_height(layout.elevation, lap, i as f32 * step))
            .collect();
        let factor = |j: usize| {
            let along = (j % n) as f32 * step;
            layout
                .banking
                .iter()
                .find(|banking| (banking.from..banking.to).contains(&along))
                .map_or(1.0, |banking| banking.factor)
        };
        track.bank = (0..n)
            .map(|i| {
                let window = (i + n - BANK_SMOOTHING)..=(i + n + BANK_SMOOTHING);
                let count = window.clone().count() as f32;
                let curvature = window.map(|j| track.curvature(j) * factor(j)).sum::<f32>() / count;
                (curvature * BANK_PER_CURVATURE).clamp(-MAX_BANK, MAX_BANK)
            })
            .collect();
        track
    }

    pub fn half_width(&self) -> f32 {
        self.half_width
    }

    /// Where the solid edge is at each point of the centre line, to the left
    /// and to the right: the walls, or the earth where the terrain rises
    /// before them. Each is capped at [`WALL_DISTANCE`].
    pub fn set_edges(&mut self, edges: Vec<[f32; 2]>) {
        debug_assert!(edges.is_empty() || edges.len() == self.centerline.len());
        self.edges = edges
            .into_iter()
            .map(|sides| sides.map(|edge| edge.min(WALL_DISTANCE)))
            .collect();
    }

    /// How far from the centre line the solid edge is beside a point
    /// (m): the wall, or the earth where it rises before the wall.
    pub fn edge_beside(&self, nearest: &Nearest) -> f32 {
        if self.edges.is_empty() {
            return WALL_DISTANCE;
        }
        let side = usize::from(nearest.lateral < 0.0);
        let n = self.centerline.len();
        let (a, b) = (
            self.edges[nearest.index % n][side],
            self.edges[(nearest.index + 1) % n][side],
        );
        a.lerp(b, nearest.t)
    }

    /// The closest the solid edge comes on either side anywhere near the
    /// point `index` of the centre line (within `reach` samples).
    pub fn closest_edge_near(&self, index: usize, reach: usize) -> f32 {
        if self.edges.is_empty() {
            return WALL_DISTANCE;
        }
        let n = self.centerline.len();
        (index + n - reach..=index + n + reach)
            .flat_map(|i| self.edges[i % n])
            .fold(WALL_DISTANCE, f32::min)
    }

    pub fn sample_count(&self) -> usize {
        self.centerline.len()
    }

    /// Lo que mide una vuelta (m).
    pub fn lap_length(&self) -> f32 {
        self.centerline.len() as f32 * self.step_length()
    }

    /// Por dónde de la vuelta (m desde la meta) está el punto más cercano.
    pub fn along(&self, nearest: &Nearest) -> f32 {
        (nearest.index as f32 + nearest.t) * self.step_length()
    }

    /// El punto de la línea central más cercano a `along` metros de la meta.
    pub fn index_at(&self, along: f32) -> usize {
        let n = self.centerline.len();
        ((along.rem_euclid(self.lap_length()) / self.step_length()).round() as usize) % n
    }

    /// Distancia real entre puntos consecutivos: casi `SAMPLE_SPACING`,
    /// ajustada para que la vuelta cierre exacta.
    pub fn step_length(&self) -> f32 {
        self.step
    }

    /// Posición y orientación en el punto `index` de la línea central.
    pub fn pose_at(&self, index: usize) -> (Vec2, f32) {
        let n = self.centerline.len();
        let point = self.centerline[index % n];
        let next = self.centerline[(index + 1) % n];
        (point, heading_of(next - point))
    }

    /// Puesto en la parrilla de salida: `row` filas por detrás de la primera
    /// y `lane` metros a la izquierda de la línea central. Para las pruebas, que
    /// colocan motos en cualquier carril.
    #[cfg(test)]
    pub fn grid_pose(&self, row: usize, lane: f32) -> (Vec2, f32) {
        let index = self.centerline.len() - START_BEHIND_SAMPLES - row * GRID_ROW_SAMPLES;
        let (_, heading) = self.pose_at(index);
        (self.offset_point(index, lane), heading)
    }

    /// Puesto `slot` de la parrilla de salida (0 es la pole), con tres motos
    /// por fila escalonadas.
    pub fn grid_slot(&self, slot: usize) -> (Vec2, f32) {
        let (row, column) = (slot / GRID_LANES.len(), slot % GRID_LANES.len());
        let index = self.centerline.len() - START_BEHIND_SAMPLES - row * GRID_ROW_SAMPLES - column;
        let (_, heading) = self.pose_at(index);
        (self.offset_point(index, GRID_LANES[column]), heading)
    }

    /// Índice del punto de la línea central más cercano a `point`.
    #[cfg(test)]
    pub fn nearest_index(&self, point: Vec2) -> usize {
        self.nearest_index_on(point, None)
    }

    /// Like [`Self::nearest_index`], for something at `height`: where the
    /// track passes over itself, the point of the stretch at its level.
    pub fn nearest_index_at(&self, point: Vec2, height: f32) -> usize {
        self.nearest_index_on(point, Some(height))
    }

    fn nearest_index_on(&self, point: Vec2, height: Option<f32>) -> usize {
        let closest = |on_level: &dyn Fn(usize) -> bool| {
            let mut best: Option<(f32, usize)> = None;
            for (index, at) in self.centerline.iter().enumerate() {
                let distance = at.distance_squared(point);
                if best.is_none_or(|(least, _)| distance < least) && on_level(index) {
                    best = Some((distance, index));
                }
            }
            best.map(|(_, index)| index)
        };
        height
            .and_then(|height| closest(&|index| on_level_with(self.elevation[index], height)))
            .or_else(|| closest(&|_| true))
            .unwrap_or(0)
    }

    /// El punto de la línea central (no solo de sus vértices) más cercano.
    /// Seen from above: where the track passes over itself, it may be on
    /// either stretch. For something on the track, see [`Self::nearest_at`].
    pub fn nearest(&self, point: Vec2) -> Nearest {
        self.nearest_on(point, None)
    }

    /// Like [`Self::nearest`], for something at `height` (a bike, its
    /// rider): where the track passes over itself, the point on the stretch
    /// at its level, so a bike in the tunnel underneath stays on its road and
    /// one on the road above stays on that one.
    pub fn nearest_at(&self, point: Vec2, height: f32) -> Nearest {
        self.nearest_on(point, Some(height))
    }

    fn nearest_on(&self, point: Vec2, height: Option<f32>) -> Nearest {
        let (mut nearest, mut on_level): (Option<Nearest>, Option<Nearest>) = (None, None);
        for index in 0..self.centerline.len() {
            let here = self.project(index, point);
            let closer =
                |best: &Option<Nearest>| best.is_none_or(|best| here.distance < best.distance);
            if closer(&nearest) {
                nearest = Some(here);
            }
            if let Some(height) = height
                && closer(&on_level)
                && on_level_with(self.height_beside(&here), height)
            {
                on_level = Some(here);
            }
        }
        on_level.or(nearest).expect("el trazado tiene puntos")
    }

    /// Like [`Self::nearest`], looking only at the `count` segments of the
    /// centre line from `from` on (going round past the finish line).
    pub fn nearest_among(&self, point: Vec2, from: usize, count: usize) -> Nearest {
        let n = self.centerline.len();
        (from..from + count.max(1))
            .map(|index| self.project(index % n, point))
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
            .expect("hay algún tramo")
    }

    /// The point of the segment from `index` to the next point of the centre
    /// line nearest to `point`.
    fn project(&self, index: usize, point: Vec2) -> Nearest {
        let n = self.centerline.len();
        let (a, b) = (self.centerline[index], self.centerline[(index + 1) % n]);
        let ab = b - a;
        let t = ((point - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
        let closest = a + ab * t;
        let left = Vec2::new(ab.y, -ab.x).normalize();
        Nearest {
            point: closest,
            distance: point.distance(closest),
            index,
            t,
            lateral: (point - closest).dot(left),
        }
    }

    /// Altura de la carretera (con su peralte) en un punto. Más allá de los
    /// muros sigue la inclinación que tiene en ellos.
    pub fn road_height(&self, point: Vec2) -> f32 {
        self.height_beside(&self.nearest(point))
    }

    /// Like [`Self::road_height`], under something at `height` (see
    /// [`Self::nearest_at`]).
    pub fn road_height_at(&self, point: Vec2, height: f32) -> f32 {
        self.height_beside(&self.nearest_at(point, height))
    }

    /// Altura de la carretera junto a un punto ya localizado.
    pub fn height_beside(&self, nearest: &Nearest) -> f32 {
        let n = self.centerline.len();
        let (i, j) = (nearest.index, (nearest.index + 1) % n);
        let elevation = self.elevation[i].lerp(self.elevation[j], nearest.t);
        let bank = self.bank[i].lerp(self.bank[j], nearest.t);
        let lateral = nearest.lateral.clamp(-WALL_DISTANCE, WALL_DISTANCE);
        elevation - lateral * bank.tan()
    }

    /// Punto de la superficie de la carretera `lateral` metros a la izquierda
    /// del punto `index` de la línea central.
    pub fn surface_point(&self, index: usize, lateral: f32) -> Vec3 {
        let index = index % self.centerline.len();
        let ground = self.offset_point(index, lateral);
        let height = self.elevation[index] - lateral * self.bank[index].tan();
        Vec3::new(ground.x, height, ground.y)
    }

    /// Alturas de la línea central, una por punto: para calcular el terreno.
    pub fn elevations(&self) -> &[f32] {
        &self.elevation
    }

    pub fn centerline(&self) -> &[Vec2] {
        &self.centerline
    }

    pub fn distance_to_centerline(&self, point: Vec2) -> f32 {
        self.nearest(point).distance
    }

    pub fn surface_beside(&self, nearest: &Nearest) -> Surface {
        if nearest.distance <= self.half_width + CURB_WIDTH {
            Surface::Asphalt
        } else {
            Surface::Grass
        }
    }

    /// Pendiente de la carretera en la dirección `direction` (unitaria): metros
    /// que sube por cada metro que se avanza, under something at `height`
    /// (see [`Self::nearest_at`]).
    pub fn grade(&self, point: Vec2, direction: Vec2, height: f32) -> f32 {
        const REACH: f32 = 0.75;
        let ahead = self.road_height_at(point + direction * REACH, height);
        let behind = self.road_height_at(point - direction * REACH, height);
        (ahead - behind) / (2.0 * REACH)
    }

    /// Curvatura vertical de la carretera (1/m) junto a un punto: positiva en
    /// los cambios de rasante (lomas), negativa en los valles.
    pub fn crest(&self, nearest: &Nearest) -> f32 {
        let n = self.centerline.len();
        let at = |offset: isize| {
            self.elevation[(nearest.index as isize + offset).rem_euclid(n as isize) as usize]
        };
        let span = 2.0 * self.step_length();
        -(at(2) - 2.0 * at(0) + at(-2)) / (span * span)
    }

    /// Punto desplazado `lateral` metros de la línea central (positivo = izquierda).
    pub fn offset_point(&self, index: usize, lateral: f32) -> Vec2 {
        let index = index % self.centerline.len();
        self.centerline[index] + self.left_normal(index) * lateral
    }

    /// Curvatura en el punto `index`: 1/radio, positiva en curvas a la izquierda.
    pub fn curvature(&self, index: usize) -> f32 {
        let n = self.centerline.len();
        let index = index % n;
        let prev = self.centerline[(index + n - 1) % n];
        let point = self.centerline[index];
        let next = self.centerline[(index + 1) % n];
        let turn = wrap_angle(heading_of(next - point) - heading_of(point - prev));
        turn / (0.5 * (prev.distance(point) + point.distance(next)))
    }

    /// Curvatura sin signo media alrededor de un punto: la de un solo punto
    /// tiene ruido.
    pub fn smoothed_curvature(&self, index: usize, window: usize) -> f32 {
        let n = self.centerline.len();
        let range = (index + n - window)..=(index + n + window);
        let count = range.clone().count() as f32;
        range.map(|i| self.curvature(i).abs()).sum::<f32>() / count
    }

    /// Esquinas del rectángulo que contiene la línea central.
    pub fn bounds(&self) -> (Vec2, Vec2) {
        self.centerline.iter().fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), &point| (min.min(point), max.max(point)),
        )
    }

    fn segments(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        let next = self.centerline.iter().cycle().skip(1);
        self.centerline.iter().copied().zip(next.copied())
    }

    /// Normal unitaria hacia la izquierda en el punto `index`.
    fn left_normal(&self, index: usize) -> Vec2 {
        let n = self.centerline.len();
        let prev = self.centerline[(index + n - 1) % n];
        let next = self.centerline[(index + 1) % n];
        let tangent = (next - prev).normalize();
        Vec2::new(tangent.y, -tangent.x)
    }
}

/// How far above or below the road something on it can be and still count
/// as on it (m): a bike in the air over a crest, a rider thrown up. Where
/// the track passes over itself, the two stretches are much further apart.
const LEVEL_REACH: f32 = 6.0;

/// Whether something at `height` is on the level of road at `road` (see
/// [`Track::nearest_at`]).
fn on_level_with(road: f32, height: f32) -> bool {
    (road - height).abs() <= LEVEL_REACH
}

/// Altura de un perfil cerrado de `lap` metros en el punto `s`, con una curva
/// suave (Hermite con tangentes de Catmull-Rom) que pasa por los puntos dados.
fn profile_height(points: &[(f32, f32)], lap: f32, s: f32) -> f32 {
    let n = points.len();
    // Puntos del perfil como si se repitiera vuelta tras vuelta.
    let at = |i: isize| {
        let wrapped = i.rem_euclid(n as isize) as usize;
        let laps = i.div_euclid(n as isize) as f32;
        (points[wrapped].0 + laps * lap, points[wrapped].1)
    };
    let s = s.rem_euclid(lap);
    let i = points.iter().rposition(|&(at, _)| at <= s).unwrap_or(0) as isize;
    let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
    let span = p2.0 - p1.0;
    let tangent1 = (p2.1 - p0.1) / (p2.0 - p0.0) * span;
    let tangent2 = (p3.1 - p1.1) / (p3.0 - p1.0) * span;
    let t = (s - p1.0) / span;
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * p1.1
        + (t3 - 2.0 * t2 + t) * tangent1
        + (-2.0 * t3 + 3.0 * t2) * p2.1
        + (t3 - t2) * tangent2
}

/// Points about `spacing` metres apart along the smooth closed curve through
/// `points`, drawn like the centre line of a track: for coastlines.
pub fn closed_curve(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    resample(&sample_spline(points), spacing)
}

/// Muestrea densamente la curva Catmull-Rom centrípeta cerrada que pasa por
/// `points`. La variante centrípeta no hace bucles raros aunque los puntos
/// estén a distancias muy distintas entre sí.
fn sample_spline(points: &[Vec2]) -> Vec<Vec2> {
    const STEPS_PER_SEGMENT: usize = 512;
    let n = points.len();
    let mut samples = Vec::with_capacity(n * STEPS_PER_SEGMENT);
    for i in 0..n {
        let p0 = points[(i + n - 1) % n];
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];
        for step in 0..STEPS_PER_SEGMENT {
            let u = step as f32 / STEPS_PER_SEGMENT as f32;
            samples.push(catmull_rom(p0, p1, p2, p3, u));
        }
    }
    samples
}

/// Punto en `u ∈ [0, 1)` del tramo entre `p1` y `p2` (algoritmo de Barry-Goldman).
fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, u: f32) -> Vec2 {
    let t0 = 0.0;
    let t1 = t0 + p0.distance(p1).sqrt();
    let t2 = t1 + p1.distance(p2).sqrt();
    let t3 = t2 + p2.distance(p3).sqrt();
    let t = t1.lerp(t2, u);
    let a1 = (p0 * (t1 - t) + p1 * (t - t0)) / (t1 - t0);
    let a2 = (p1 * (t2 - t) + p2 * (t - t1)) / (t2 - t1);
    let a3 = (p2 * (t3 - t) + p3 * (t - t2)) / (t3 - t2);
    let b1 = (a1 * (t2 - t) + a2 * (t - t0)) / (t2 - t0);
    let b2 = (a2 * (t3 - t) + a3 * (t - t1)) / (t3 - t1);
    (b1 * (t2 - t) + b2 * (t - t1)) / (t2 - t1)
}

/// Recorre el polígono cerrado `points` y devuelve puntos equiespaciados a unos
/// `spacing` metros (ajustado para que la vuelta cierre exacta).
fn resample(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    let n = points.len();
    let lengths: Vec<f32> = (0..n)
        .map(|i| points[i].distance(points[(i + 1) % n]))
        .collect();
    let total: f32 = lengths.iter().sum();
    let count = (total / spacing).round() as usize;
    let step = total / count as f32;

    let mut resampled = Vec::with_capacity(count);
    let (mut segment, mut segment_start) = (0, 0.0);
    for k in 0..count {
        let target = k as f32 * step;
        while segment < n - 1 && segment_start + lengths[segment] < target {
            segment_start += lengths[segment];
            segment += 1;
        }
        let t = ((target - segment_start) / lengths[segment]).clamp(0.0, 1.0);
        resampled.push(points[segment].lerp(points[(segment + 1) % n], t));
    }
    resampled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        Track::default()
    }

    /// Todas las pistas del juego, con su nombre.
    fn every_track() -> Vec<(&'static str, Track)> {
        crate::circuits::CIRCUITS
            .iter()
            .map(|circuit| (circuit.name, Track::new(&circuit.layout)))
            .collect()
    }

    #[test]
    fn centerline_is_evenly_spaced_and_closed() {
        let track = track();
        for (a, b) in track.segments() {
            let length = a.distance(b);
            assert!(
                (length - SAMPLE_SPACING).abs() < 0.1 * SAMPLE_SPACING,
                "tramo de {length} m"
            );
        }
    }

    #[test]
    fn the_grid_has_three_staggered_bikes_per_row_on_the_road() {
        let track = track();
        let poses: Vec<_> = (0..8).map(|slot| track.grid_slot(slot)).collect();
        for (slot, (point, _)) in poses.iter().enumerate() {
            assert!(
                track.distance_to_centerline(*point) < track.half_width() - 1.0,
                "puesto {slot} en el asfalto"
            );
            for (other, (point2, _)) in poses.iter().enumerate().skip(slot + 1) {
                assert!(
                    point.distance(*point2) > 3.0,
                    "puestos {slot} y {other} separados"
                );
            }
        }
        // Cada moto, por detrás de la anterior: la pole delante de todas.
        let start = track.centerline().len() - 1;
        let behind = |slot: usize| {
            let index = track.nearest_index(poses[slot].0);
            (start + track.centerline().len() - index) % track.centerline().len()
        };
        assert!((1..8).all(|slot| behind(slot) > behind(slot - 1)));
    }

    #[test]
    fn curves_are_drivable() {
        // Por debajo de este radio el muro del interior de la curva se doblaría
        // sobre sí mismo, y además no habría quien la tomase.
        const MIN_RADIUS: f32 = 2.5 * WALL_DISTANCE;
        for (name, track) in every_track() {
            for index in 0..track.centerline.len() {
                let radius = 1.0 / track.curvature(index).abs();
                assert!(
                    radius > MIN_RADIUS,
                    "curva de radio {radius} m en {index} de {name}"
                );
            }
        }
    }

    #[test]
    fn walls_of_distant_stretches_do_not_meet() {
        for circuit in &crate::circuits::CIRCUITS {
            let (name, track) = (circuit.name, Track::new(&circuit.layout));
            let n = track.centerline.len();
            let lap = track.lap_length();
            let in_tunnel = |i: usize| {
                let along = i as f32 * track.step;
                circuit
                    .scenery
                    .tunnels
                    .iter()
                    .any(|span| span.contains(along, lap))
            };
            // Puntos cercanos sobre el trazado pueden estar cerca en el
            // espacio; más allá, entre sus muros tiene que haber sitio para el
            // escenario.
            let neighborhood = (8.0 * WALL_DISTANCE / SAMPLE_SPACING) as usize;
            let clearance = 2.0 * WALL_DISTANCE + 10.0;
            for i in 0..n {
                for j in i + neighborhood..n.min(i + n - neighborhood) {
                    let distance = track.centerline[i].distance(track.centerline[j]);
                    if distance > clearance {
                        continue;
                    }
                    // Unless one passes over the other, which goes through a
                    // tunnel well below.
                    let (low, high) = if track.elevation[i] < track.elevation[j] {
                        (i, j)
                    } else {
                        (j, i)
                    };
                    assert!(
                        in_tunnel(low) && track.elevation[high] - track.elevation[low] > 12.0,
                        "los puntos {i} y {j} de {name} se tocan"
                    );
                }
            }
        }
    }

    #[test]
    fn start_faces_down_the_main_straight() {
        for (name, track) in every_track() {
            let (position, heading) = track.grid_pose(0, 0.0);
            assert!(position.x.abs() < 1.0, "{name}");
            assert!(
                position.y > 0.0,
                "la salida está por detrás de la meta en {name}"
            );
            assert!(heading.abs() < 0.05, "la recta va hacia -Z en {name}");
        }
    }

    #[test]
    fn surface_changes_at_the_curb() {
        let track = track();
        let edge = track.half_width + CURB_WIDTH;
        assert_eq!(
            track.surface_beside(&track.nearest(track.offset_point(100, 0.0))),
            Surface::Asphalt
        );
        assert_eq!(
            track.surface_beside(&track.nearest(track.offset_point(100, edge - 0.1))),
            Surface::Asphalt
        );
        assert_eq!(
            track.surface_beside(&track.nearest(track.offset_point(100, -edge - 0.5))),
            Surface::Grass
        );
    }

    #[test]
    fn slopes_and_banking_stay_rideable() {
        for (name, track) in every_track() {
            let (n, step) = (track.sample_count(), track.step_length());
            for i in 0..n {
                let grade = (track.elevation[(i + 1) % n] - track.elevation[i]) / step;
                assert!(
                    grade.abs() < 0.13,
                    "pendiente del {:.0} % en {i} de {name}",
                    grade * 100.0
                );
                assert!(track.bank[i].abs() <= MAX_BANK);
            }
            // Las curvas cerradas sí tienen peralte.
            assert!(track.bank.iter().any(|b| b.abs() > 0.08), "{name}");
        }
    }

    #[test]
    fn banked_spans_bank_more() {
        let layout = &crate::circuits::CIRCUITS[1].layout;
        let banked = Track::new(layout);
        let plain = Track::new(&Layout {
            banking: &[],
            ..*layout
        });
        let most = |track: &Track, from: f32, to: f32| {
            (track.index_at(from)..track.index_at(to))
                .map(|i| track.bank[i].abs())
                .fold(0.0, f32::max)
        };
        let span = &layout.banking[0];
        assert!(most(&banked, span.from, span.to) > 1.8 * most(&plain, span.from, span.to));
        // Lejos del tramo, igual.
        assert_eq!(most(&banked, 1400.0, 1600.0), most(&plain, 1400.0, 1600.0));
    }

    #[test]
    fn positions_along_the_lap_round_trip() {
        let track = track();
        let lap = track.lap_length();
        assert!((lap - track.sample_count() as f32 * track.step_length()).abs() < 1e-3);
        let index = track.index_at(321.0);
        let nearest = track.nearest(track.offset_point(index, 3.0));
        assert!((track.along(&nearest) - 321.0).abs() < track.step_length());
        // Más allá de una vuelta, da la vuelta.
        assert_eq!(track.index_at(lap + 321.0), index);
    }

    #[test]
    fn profile_passes_through_its_points_and_closes() {
        let lap = 1000.0;
        let points = [(0.0, 0.0), (300.0, 10.0), (600.0, 4.0)];
        for (s, h) in points {
            assert!((profile_height(&points, lap, s) - h).abs() < 1e-4);
        }
        let before = profile_height(&points, lap, lap - 0.01);
        assert!((before - profile_height(&points, lap, 0.0)).abs() < 0.01);
    }

    #[test]
    fn banking_raises_the_outside_of_the_curve() {
        let track = track();
        let index = (0..track.sample_count())
            .max_by(|&a, &b| track.bank[a].abs().total_cmp(&track.bank[b].abs()))
            .unwrap();
        let curvature = track.curvature(index);
        // El exterior de una curva a la izquierda es la derecha (lateral < 0).
        let outside = track.surface_point(index, -curvature.signum() * 7.0);
        let inside = track.surface_point(index, curvature.signum() * 7.0);
        assert!(outside.y > inside.y);
    }

    #[test]
    fn nearest_point_lies_on_the_centerline() {
        let track = track();
        let nearest = track.nearest(track.offset_point(200, 5.0));
        assert!((nearest.distance - 5.0).abs() < 0.05);
        assert!((nearest.lateral - 5.0).abs() < 0.05);
        let height = track.road_height(track.offset_point(200, 5.0));
        assert!((height - track.surface_point(200, 5.0).y).abs() < 0.05);
        assert!(nearest.point.distance(track.offset_point(200, 0.0)) < 0.1);
    }
}
