//! El trazado del circuito: su línea central y las consultas que necesitan
//! la física, la IA y el escenario. No sabe nada de render, así se puede
//! probar sin abrir una ventana.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::geometry::{forward, heading_of, smoothstep, wrap_angle};

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
    /// Alternative routes that leave the lap and join it again.
    pub branches: &'static [Branch],
}

/// An alternative route: a road that forks off the lap `from` metres after
/// the finish line, runs through the control points of `approach` to the
/// mouth of a vertical loop, goes round the loop, and from its way out runs
/// through the control points of `exit` back onto the lap, `to` metres
/// after the finish line. Both roads are narrower than the lap.
pub struct Branch {
    pub from: f32,
    pub to: f32,
    pub approach: &'static [Vec2],
    pub vertical_loop: VerticalLoop,
    pub exit: &'static [Vec2],
}

/// A vertical loop the bikes ride round, upside down at the top.
pub struct VerticalLoop {
    /// Where the bikes go in: the middle of the running surface at the
    /// bottom (x, z).
    pub mouth: Vec2,
    /// The heading they go in and come out with (see [`crate::geometry`]).
    pub heading: f32,
    /// Height of the road at the bottom, like the heights of
    /// [`Layout::elevation`].
    pub height: f32,
    /// Radius of the running surface (m).
    pub radius: f32,
    /// How far to the left (negative: to the right) the way out is from the
    /// way in (m): the running surface drifts sideways as it goes round, so
    /// that it does not run into itself.
    pub shift: f32,
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

/// Half the width of the asphalt of a road off the lap (m).
const SIDE_HALF_WIDTH: f32 = 5.0;
/// While a road off the lap shares the road between the lap's walls, it
/// lies on the lap's surface, with its height and its banking. From this far
/// from the lap's centre line (m), where their walls part, it goes over to
/// its own height and banking, all of it this much further away (m).
const JUNCTION_APART: f32 = 2.0 * WALL_DISTANCE;
const JUNCTION_BLEND: f32 = 30.0;
/// Half the width of the running surface of a loop (m): the asphalt of the
/// road that leads to it and its curbs.
pub const LOOP_HALF_WIDTH: f32 = SIDE_HALF_WIDTH + CURB_WIDTH;
/// Distance from the middle of a loop to the walls at its mouth (m), and
/// over how much road before and after it the walls close in to there and
/// open out again (m).
pub const LOOP_WALL: f32 = LOOP_HALF_WIDTH + 0.3;
const FUNNEL: f32 = 70.0;
/// The last metres of road into a loop and the first out of it, straight
/// along it.
const LOOP_LEAD: f32 = 25.0;

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
    /// Whether the centre line closes on itself: the lap does; a road off it
    /// (see [`Branch`]) does not.
    closed: bool,
    /// How far the walls are from the centre line at each point (m). Empty:
    /// at [`WALL_DISTANCE`] everywhere, as on the lap.
    walls: Vec<f32>,
    /// On a road off the lap, how far along the lap each of its points
    /// counts as (m from the finish line). Empty on the lap.
    lap_along: Vec<f32>,
    /// On a road off the lap, how much its surface is its own at each point,
    /// from 0 where it lies on the lap's to 1 once it is apart. Empty on
    /// the lap.
    apart: Vec<f32>,
    /// The roads off the lap: for each branch, the road to its loop and the
    /// road back. Numbered from 1 in [`Nearest::road`]; the lap is 0.
    side_roads: Vec<Track>,
    loops: Vec<Loop>,
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
    /// Which road it is on: 0 for the lap, or a road off it (see
    /// [`Track::road`]). `index` counts along that road.
    pub road: usize,
}

/// A vertical loop as the bikes ride it (see [`VerticalLoop`]): where it
/// is, and the roads in and out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loop {
    pub mouth: Vec2,
    /// Unit vector the way the bikes go in, on the ground.
    pub ahead: Vec2,
    pub heading: f32,
    /// Height of the running surface at the bottom (m).
    pub height: f32,
    pub radius: f32,
    pub shift: f32,
    /// The roads (see [`Nearest::road`]) that lead in and out.
    pub approach: usize,
    pub exit: usize,
}

impl Loop {
    /// Unit vector to the left of the way in, on the ground: across the
    /// running surface.
    pub fn left(&self) -> Vec2 {
        Vec2::new(self.ahead.y, -self.ahead.x)
    }

    /// Where the bikes come out: the middle of the running surface at the
    /// bottom, beside the mouth.
    pub fn way_out(&self) -> Vec2 {
        self.mouth + self.left() * self.shift
    }

    /// Metres of running surface per radian round (m).
    pub fn arc(&self) -> f32 {
        self.radius.hypot(self.shift / TAU)
    }

    /// How long the running surface is, all the way round (m).
    pub fn length(&self) -> f32 {
        TAU * self.arc()
    }

    /// Point of the running surface `angle` radians round from the mouth,
    /// `lateral` metres to the left of its middle (x, height, z).
    pub fn surface_point(&self, angle: f32, lateral: f32) -> Vec3 {
        let ground = self.mouth
            + self.ahead * self.radius * angle.sin()
            + self.left() * (self.shift * angle / TAU + lateral);
        Vec3::new(
            ground.x,
            self.height + self.radius * (1.0 - angle.cos()),
            ground.y,
        )
    }

    /// Unit vector along the running surface the way the bikes go, `angle`
    /// radians round (not counting the drift sideways).
    pub fn tangent(&self, angle: f32) -> Vec3 {
        let ahead = self.ahead * angle.cos();
        Vec3::new(ahead.x, angle.sin(), ahead.y)
    }

    /// Unit vector out of the running surface, towards the middle of the
    /// loop: which way is up for a bike riding it.
    pub fn up(&self, angle: f32) -> Vec3 {
        let back = -self.ahead * angle.sin();
        Vec3::new(back.x, angle.cos(), back.y)
    }

    /// If something at `position`, at `height`, has just gone in at the
    /// mouth, how far in (m) and how far to the left of the middle (m).
    pub fn entered(&self, position: Vec2, height: f32) -> Option<(f32, f32)> {
        let from = position - self.mouth;
        let (into, lateral) = (from.dot(self.ahead), from.dot(self.left()));
        ((0.0..4.0).contains(&into)
            && lateral.abs() < LOOP_WALL
            && (height - self.height).abs() < 1.5)
            .then_some((into, lateral))
    }
}

impl Default for Track {
    /// La primera pista, la Bahía.
    fn default() -> Self {
        Self::new(&crate::circuits::CIRCUITS[0].layout)
    }
}

impl Track {
    pub fn new(layout: &Layout) -> Self {
        let mut track = Self::lap(layout);
        for branch in layout.branches {
            track.add_branch(branch);
        }
        track
    }

    /// The lap of a layout, without its branches.
    fn lap(layout: &Layout) -> Self {
        let mut track = Self {
            centerline: resample(&sample_spline(layout.control_points), SAMPLE_SPACING),
            elevation: Vec::new(),
            bank: Vec::new(),
            half_width: TRACK_WIDTH / 2.0,
            step: 0.0,
            edges: Vec::new(),
            closed: true,
            walls: Vec::new(),
            lap_along: Vec::new(),
            apart: Vec::new(),
            side_roads: Vec::new(),
            loops: Vec::new(),
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

    /// Adds the two roads of a branch, and the loop between them.
    fn add_branch(&mut self, branch: &Branch) {
        let spec = &branch.vertical_loop;
        let ahead = forward(spec.heading);
        let looped = Loop {
            mouth: spec.mouth,
            ahead,
            heading: spec.heading,
            height: BASE_ELEVATION + spec.height,
            radius: spec.radius,
            shift: spec.shift,
            approach: self.side_roads.len() + 1,
            exit: self.side_roads.len() + 2,
        };
        let lap_point = |along: f32| self.centerline[self.index_at(along)];
        // The last stretch into the loop and the first out of it are
        // straight, and each road leaves the lap, or joins it, along it.
        let lead = ahead * LOOP_LEAD;
        let (mouth, way_out) = (looped.mouth, looped.way_out());
        let mut points = vec![lap_point(branch.from)];
        points.extend_from_slice(branch.approach);
        points.extend([mouth - 2.0 * lead, mouth - lead, mouth]);
        let approach = open_curve(&points, lap_point(branch.from - 20.0), mouth + lead);
        let mut points = vec![way_out, way_out + lead, way_out + 2.0 * lead];
        points.extend_from_slice(branch.exit);
        points.push(lap_point(branch.to));
        let exit = open_curve(&points, way_out - lead, lap_point(branch.to + 20.0));

        // How far along the lap each point of the branch counts as: the
        // stretch of lap it stands in for, shared out by distance.
        let length = |line: &[Vec2]| line.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>();
        let (into, out) = (length(&approach), length(&exit));
        let total = into + looped.length() + out;
        let lap = self.lap_length();
        let span = (branch.to - branch.from).rem_euclid(lap);
        let counts_as = |from: f32| move |s: f32| (branch.from + span * (from + s) / total) % lap;
        let approach = self.side_road(approach, &looped, true, counts_as(0.0));
        let exit = self.side_road(exit, &looped, false, counts_as(into + looped.length()));
        self.side_roads.extend([approach, exit]);
        self.loops.push(looped);
    }

    /// A road off the lap along `centerline`, which comes from the lap into
    /// the loop if `into_loop`, or else out of the loop onto the lap.
    /// `counts_as` says how far along the lap each metre of it counts as.
    fn side_road(
        &self,
        centerline: Vec<Vec2>,
        looped: &Loop,
        into_loop: bool,
        counts_as: impl Fn(f32) -> f32,
    ) -> Track {
        let n = centerline.len();
        let step = centerline
            .windows(2)
            .map(|w| w[0].distance(w[1]))
            .sum::<f32>()
            / (n - 1) as f32;
        let mut road = Track {
            centerline,
            elevation: Vec::new(),
            bank: Vec::new(),
            half_width: SIDE_HALF_WIDTH,
            step,
            edges: Vec::new(),
            closed: false,
            walls: Vec::new(),
            lap_along: Vec::new(),
            apart: Vec::new(),
            side_roads: Vec::new(),
            loops: Vec::new(),
        };
        let length = road.length();
        // Metres from the end at the loop.
        let from_loop = |i: usize| {
            let s = i as f32 * step;
            if into_loop { length - s } else { s }
        };
        let mut elevation = Vec::with_capacity(n);
        let mut bank = Vec::with_capacity(n);
        let mut apart = Vec::with_capacity(n);
        for i in 0..n {
            let from_loop = from_loop(i);
            // Near the lap the road lies on its surface; then it goes to the
            // height of the loop, and banks for its own curves, but not in
            // the straight into or out of the loop.
            let on_lap = self.project_on_lap(road.centerline[i]);
            let own = smoothstep(
                JUNCTION_APART,
                JUNCTION_APART + JUNCTION_BLEND,
                on_lap.distance,
            );
            let lap_bank = self.bank[on_lap.index].lerp(
                self.bank[(on_lap.index + 1) % self.sample_count()],
                on_lap.t,
            );
            let window =
                (i as isize - BANK_SMOOTHING as isize)..=(i as isize + BANK_SMOOTHING as isize);
            let count = window.clone().count() as f32;
            let curvature = window.map(|j| road.curvature(road.wrap(j))).sum::<f32>() / count;
            let own_bank = (curvature * BANK_PER_CURVATURE).clamp(-MAX_BANK, MAX_BANK)
                * smoothstep(LOOP_LEAD, LOOP_LEAD + 30.0, from_loop);
            elevation.push(self.height_on_road(&on_lap).lerp(looped.height, own));
            bank.push(lap_bank.lerp(own_bank, own));
            apart.push(own);
        }
        road.elevation = elevation;
        road.bank = bank;
        road.apart = apart;
        road.walls = (0..n)
            .map(|i| LOOP_WALL.lerp(WALL_DISTANCE, smoothstep(10.0, FUNNEL, from_loop(i))))
            .collect();
        road.lap_along = (0..n).map(|i| counts_as(i as f32 * step)).collect();
        road
    }

    /// The nearest point of the lap's own centre line, leaving out the roads
    /// off it.
    fn project_on_lap(&self, point: Vec2) -> Nearest {
        (0..self.sample_count())
            .map(|index| self.project(index, point))
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
            .expect("el trazado tiene puntos")
    }

    pub fn half_width(&self) -> f32 {
        self.half_width
    }

    /// The roads off the lap, as numbered in [`Nearest::road`] from 1.
    pub fn side_roads(&self) -> &[Track] {
        &self.side_roads
    }

    /// The vertical loops of the branches.
    pub fn loops(&self) -> &[Loop] {
        &self.loops
    }

    /// Adds a loop on its own, without roads in or out: for the tests.
    #[cfg(test)]
    pub fn add_loop_for_tests(&mut self, looped: Loop) {
        self.loops.push(looped);
    }

    /// Road number `road` (see [`Nearest::road`]): the lap itself for 0.
    pub fn road(&self, road: usize) -> &Track {
        match road {
            0 => self,
            _ => &self.side_roads[road - 1],
        }
    }

    /// Whether the centre line closes on itself, as the lap's does.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// How many stretches the centre line has between its points.
    pub fn segment_count(&self) -> usize {
        if self.closed {
            self.centerline.len()
        } else {
            self.centerline.len() - 1
        }
    }

    /// How long the centre line is, from end to end (m): a whole lap for the
    /// lap.
    pub fn length(&self) -> f32 {
        self.segment_count() as f32 * self.step
    }

    /// How far the walls are from the centre line at point `index` (m).
    pub fn wall_at(&self, index: usize) -> f32 {
        self.walls
            .get(self.wrap(index as isize))
            .copied()
            .unwrap_or(WALL_DISTANCE)
    }

    /// Point `index` of the centre line, counting round the lap, or held at
    /// the ends of a road off it.
    fn wrap(&self, index: isize) -> usize {
        let n = self.centerline.len();
        if self.closed {
            index.rem_euclid(n as isize) as usize
        } else {
            index.clamp(0, n as isize - 1) as usize
        }
    }

    /// Where the solid edge is beside each road, at each point of its centre
    /// line, to the left and to the right: the walls, or the earth where the
    /// terrain rises before them. The lap first, then the roads off it in
    /// order (see [`Nearest::road`]). Each is capped at its walls.
    pub fn set_edges(&mut self, mut edges: Vec<Vec<[f32; 2]>>) {
        edges.resize(1 + self.side_roads.len(), Vec::new());
        for (road, edges) in edges.into_iter().enumerate() {
            let road = match road {
                0 => &mut *self,
                _ => &mut self.side_roads[road - 1],
            };
            debug_assert!(edges.is_empty() || edges.len() == road.centerline.len());
            let capped = edges
                .into_iter()
                .enumerate()
                .map(|(index, sides)| sides.map(|edge| edge.min(road.wall_at(index))))
                .collect();
            road.edges = capped;
        }
    }

    /// How far from the centre line the solid edge is beside a point
    /// (m): the wall, or the earth where it rises before the wall.
    pub fn edge_beside(&self, nearest: &Nearest) -> f32 {
        let road = self.road(nearest.road);
        let (i, j) = (
            road.wrap(nearest.index as isize),
            road.wrap(nearest.index as isize + 1),
        );
        let side = usize::from(nearest.lateral < 0.0);
        let edge = |index: usize| {
            road.edges
                .get(index)
                .map_or(road.wall_at(index), |edges| edges[side])
        };
        edge(i).lerp(edge(j), nearest.t)
    }

    /// The closest the solid edge comes on either side of the road of
    /// `nearest` anywhere near it (within `reach` points of its centre
    /// line).
    pub fn closest_edge_near(&self, nearest: &Nearest, reach: usize) -> f32 {
        let road = self.road(nearest.road);
        let (index, reach) = (nearest.index as isize, reach as isize);
        (index - reach..=index + reach)
            .map(|i| road.wrap(i))
            .flat_map(|i| road.edges.get(i).copied().unwrap_or([road.wall_at(i); 2]))
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
    /// On a road off the lap, the stretch of the lap it stands in for.
    pub fn along(&self, nearest: &Nearest) -> f32 {
        let road = self.road(nearest.road);
        if road.closed {
            return (nearest.index as f32 + nearest.t) * road.step_length();
        }
        let (a, b) = (
            road.lap_along[road.wrap(nearest.index as isize)],
            road.lap_along[road.wrap(nearest.index as isize + 1)],
        );
        // Across the finish line, the next point counts as one more lap.
        let b = if b < a { b + self.lap_length() } else { b };
        a.lerp(b, nearest.t) % self.lap_length()
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
        let (i, j) = (self.wrap(index as isize), self.wrap(index as isize + 1));
        let (point, next) = (self.centerline[i], self.centerline[j]);
        if i == j {
            // The last point of a road off the lap: the way it was going.
            let prev = self.centerline[self.wrap(index as isize - 1)];
            return (point, heading_of(point - prev));
        }
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
    /// Only the lap counts, not the roads off it.
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

    /// El punto de la línea central (no solo de sus vértices) más cercano,
    /// on the lap or on a road off it.
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
        let (mut nearest, mut on_level) = self.search(point, height, 0);
        for (road, side_road) in self.side_roads.iter().enumerate() {
            let (here, level) = side_road.search(point, height, road + 1);
            nearest = closer(nearest, here);
            on_level = closer(on_level, level);
        }
        on_level.or(nearest).expect("el trazado tiene puntos")
    }

    /// The nearest point of this road's own centre line to `point`, and the
    /// nearest on the level of `height`, if any (see [`Self::nearest_at`]),
    /// counted as road number `road`.
    fn search(
        &self,
        point: Vec2,
        height: Option<f32>,
        road: usize,
    ) -> (Option<Nearest>, Option<Nearest>) {
        let (mut nearest, mut on_level): (Option<Nearest>, Option<Nearest>) = (None, None);
        for index in 0..self.segment_count() {
            let here = Nearest {
                road,
                ..self.project(index, point)
            };
            let closer =
                |best: &Option<Nearest>| best.is_none_or(|best| here.distance < best.distance);
            if closer(&nearest) {
                nearest = Some(here);
            }
            if let Some(height) = height
                && closer(&on_level)
                && on_level_with(self.height_on_road(&here), height)
            {
                on_level = Some(here);
            }
        }
        (nearest, on_level)
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
            road: 0,
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

    /// Altura de la carretera junto a un punto ya localizado. Where a road
    /// off the lap shares the lap's road, the lap's surface, so that the two
    /// are one.
    pub fn height_beside(&self, nearest: &Nearest) -> f32 {
        let road = self.road(nearest.road);
        let own = road.height_on_road(nearest);
        if road.closed {
            return own;
        }
        let (i, j) = (
            road.wrap(nearest.index as isize),
            road.wrap(nearest.index as isize + 1),
        );
        let apart = road.apart[i].lerp(road.apart[j], nearest.t);
        if apart >= 1.0 {
            return own;
        }
        let across = road.centerline[j] - road.centerline[i];
        let point = nearest.point + Vec2::new(across.y, -across.x).normalize() * nearest.lateral;
        self.height_on_road(&self.project_on_lap(point))
            .lerp(own, apart)
    }

    /// Like [`Self::surface_point`], on road number `road` (see
    /// [`Nearest::road`]): where a road off the lap shares the lap's road, on
    /// the lap's surface.
    pub fn surface_point_on(&self, road: usize, index: usize, lateral: f32) -> Vec3 {
        let side_road = self.road(road);
        let point = side_road.surface_point(index, lateral);
        let apart = side_road
            .apart
            .get(side_road.wrap(index as isize))
            .copied()
            .unwrap_or(1.0);
        if apart >= 1.0 {
            return point;
        }
        let lap = self.height_on_road(&self.project_on_lap(Vec2::new(point.x, point.z)));
        Vec3::new(point.x, lap.lerp(point.y, apart), point.z)
    }

    /// Like [`Self::height_beside`], for a point of this road itself.
    fn height_on_road(&self, nearest: &Nearest) -> f32 {
        let (i, j) = (
            self.wrap(nearest.index as isize),
            self.wrap(nearest.index as isize + 1),
        );
        let elevation = self.elevation[i].lerp(self.elevation[j], nearest.t);
        let bank = self.bank[i].lerp(self.bank[j], nearest.t);
        let lateral = nearest.lateral.clamp(-WALL_DISTANCE, WALL_DISTANCE);
        elevation - lateral * bank.tan()
    }

    /// Which way the road beside a point already found goes, and the point
    /// `lateral` metres to the left of its centre line there.
    pub fn pose_beside(&self, nearest: &Nearest, lateral: f32) -> (Vec2, f32) {
        let road = self.road(nearest.road);
        let (_, heading) = road.pose_at(nearest.index);
        let point = road
            .offset_point(nearest.index, lateral)
            .lerp(road.offset_point(nearest.index + 1, lateral), nearest.t);
        (point, heading)
    }

    /// Half the width of the asphalt of the road beside a point already
    /// found.
    pub fn half_width_beside(&self, nearest: &Nearest) -> f32 {
        self.road(nearest.road).half_width
    }

    /// Punto de la superficie de la carretera `lateral` metros a la izquierda
    /// del punto `index` de la línea central.
    pub fn surface_point(&self, index: usize, lateral: f32) -> Vec3 {
        let index = self.wrap(index as isize);
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
        if nearest.distance <= self.half_width_beside(nearest) + CURB_WIDTH {
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
        let road = self.road(nearest.road);
        let at = |offset: isize| road.elevation[road.wrap(nearest.index as isize + offset)];
        let span = 2.0 * road.step_length();
        -(at(2) - 2.0 * at(0) + at(-2)) / (span * span)
    }

    /// Punto desplazado `lateral` metros de la línea central (positivo = izquierda).
    pub fn offset_point(&self, index: usize, lateral: f32) -> Vec2 {
        let index = self.wrap(index as isize);
        self.centerline[index] + self.left_normal(index) * lateral
    }

    /// Curvatura en el punto `index`: 1/radio, positiva en curvas a la izquierda.
    pub fn curvature(&self, index: usize) -> f32 {
        let index = self.wrap(index as isize);
        let (before, after) = (self.wrap(index as isize - 1), self.wrap(index as isize + 1));
        if before == index || after == index {
            // The ends of a road off the lap.
            return 0.0;
        }
        let prev = self.centerline[before];
        let point = self.centerline[index];
        let next = self.centerline[after];
        let turn = wrap_angle(heading_of(next - point) - heading_of(point - prev));
        turn / (0.5 * (prev.distance(point) + point.distance(next)))
    }

    /// Curvatura sin signo media alrededor de un punto: la de un solo punto
    /// tiene ruido.
    pub fn smoothed_curvature(&self, index: usize, window: usize) -> f32 {
        let (index, window) = (index as isize, window as isize);
        let range = (index - window)..=(index + window);
        let count = range.clone().count() as f32;
        range
            .map(|i| self.curvature(self.wrap(i)).abs())
            .sum::<f32>()
            / count
    }

    /// Esquinas del rectángulo que contiene la línea central, and those of
    /// the roads off it.
    pub fn bounds(&self) -> (Vec2, Vec2) {
        self.side_roads
            .iter()
            .flat_map(|road| &road.centerline)
            .chain(&self.centerline)
            .fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), &point| (min.min(point), max.max(point)),
            )
    }

    fn segments(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        let next = self.centerline.iter().cycle().skip(1);
        self.centerline
            .iter()
            .copied()
            .zip(next.copied())
            .take(self.segment_count())
    }

    /// Normal unitaria hacia la izquierda en el punto `index`.
    fn left_normal(&self, index: usize) -> Vec2 {
        let prev = self.centerline[self.wrap(index as isize - 1)];
        let next = self.centerline[self.wrap(index as isize + 1)];
        let tangent = (next - prev).normalize();
        Vec2::new(tangent.y, -tangent.x)
    }
}

/// The nearer of two points found on the centre lines, if any.
fn closer(a: Option<Nearest>, b: Option<Nearest>) -> Option<Nearest> {
    match (a, b) {
        (Some(a), Some(b)) => Some(if b.distance < a.distance { b } else { a }),
        (a, b) => a.or(b),
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

/// Points about [`SAMPLE_SPACING`] apart (adjusted so that the ends fall on
/// points) along the smooth curve from the first of `points` to the last,
/// through the others, drawn like the centre line of the lap. It leaves the
/// first as if it came from `before` and reaches the last as if it went on
/// to `after`.
fn open_curve(points: &[Vec2], before: Vec2, after: Vec2) -> Vec<Vec2> {
    const STEPS_PER_SEGMENT: usize = 512;
    let all: Vec<Vec2> = std::iter::once(before)
        .chain(points.iter().copied())
        .chain(std::iter::once(after))
        .collect();
    let mut samples = Vec::with_capacity(all.len() * STEPS_PER_SEGMENT);
    for w in all.windows(4) {
        for step in 0..STEPS_PER_SEGMENT {
            let u = step as f32 / STEPS_PER_SEGMENT as f32;
            samples.push(catmull_rom(w[0], w[1], w[2], w[3], u));
        }
    }
    samples.push(all[all.len() - 2]);
    let lengths: Vec<f32> = samples.windows(2).map(|w| w[0].distance(w[1])).collect();
    let total: f32 = lengths.iter().sum();
    let count = ((total / SAMPLE_SPACING).round() as usize).max(1);
    let step = total / count as f32;
    let mut resampled = Vec::with_capacity(count + 1);
    let (mut segment, mut segment_start) = (0, 0.0);
    for k in 0..count {
        let target = k as f32 * step;
        while segment < lengths.len() - 1 && segment_start + lengths[segment] < target {
            segment_start += lengths[segment];
            segment += 1;
        }
        let t = ((target - segment_start) / lengths[segment].max(1e-6)).clamp(0.0, 1.0);
        resampled.push(samples[segment].lerp(samples[segment + 1], t));
    }
    resampled.push(samples[samples.len() - 1]);
    resampled
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

    /// The tracks with branches, with their names.
    fn tracks_with_branches() -> Vec<(&'static str, Track)> {
        let tracks: Vec<_> = every_track()
            .into_iter()
            .filter(|(_, track)| !track.loops.is_empty())
            .collect();
        assert!(!tracks.is_empty(), "some track has a branch");
        tracks
    }

    #[test]
    fn a_branch_leaves_the_lap_and_joins_it_again_along_it() {
        for (name, track) in tracks_with_branches() {
            for looped in &track.loops {
                let (into, out) = (track.road(looped.approach), track.road(looped.exit));
                // It starts on the lap, going its way, and ends on it too.
                for (road, index) in [(into, 0), (out, out.sample_count() - 1)] {
                    let point = road.centerline[index];
                    let on_lap = track.project_on_lap(point);
                    assert!(on_lap.distance < 0.5, "{name}: {}", on_lap.distance);
                    let (_, lap_heading) = track.pose_at(on_lap.index);
                    let (_, heading) = road.pose_at(index);
                    let turn = wrap_angle(heading - lap_heading).abs();
                    assert!(turn < 0.08, "{name}: {turn} rad off the lap");
                    // On the lap's surface, height and banking.
                    let lap_height = track.height_on_road(&on_lap);
                    for lateral in [-4.0, 0.0, 4.0] {
                        let here = road.surface_point(index, lateral);
                        let lap = track.road_height(Vec2::new(here.x, here.z));
                        assert!((here.y - lap).abs() < 0.05, "{name}: {} m", here.y - lap);
                    }
                    assert!((road.elevation[index] - lap_height).abs() < 0.01);
                }
                // Into the loop and out of it, straight along it, at its height.
                let mouth = into.sample_count() - 1;
                assert!(into.centerline[mouth].distance(looped.mouth) < 0.01);
                assert!(out.centerline[0].distance(looped.way_out()) < 0.01);
                for (road, index) in [(into, mouth - 5), (out, 5)] {
                    let (_, heading) = road.pose_at(index);
                    assert!(wrap_angle(heading - looped.heading).abs() < 0.01, "{name}");
                    assert!((road.elevation[index] - looped.height).abs() < 0.01);
                    assert!(road.bank[index].abs() < 1e-3);
                    assert!((road.wall_at(index) - LOOP_WALL).abs() < 1e-3);
                }
            }
        }
    }

    #[test]
    fn branches_are_rideable_and_clear_of_the_lap() {
        const MIN_RADIUS: f32 = 2.5 * WALL_DISTANCE;
        for (name, track) in tracks_with_branches() {
            for (number, road) in track.side_roads.iter().enumerate() {
                let n = road.sample_count();
                for i in 0..n - 1 {
                    let radius = 1.0 / road.curvature(i).abs();
                    assert!(
                        radius > MIN_RADIUS,
                        "{name}, road {number}: {radius} m at {i}"
                    );
                    let grade = (road.elevation[i + 1] - road.elevation[i]) / road.step;
                    assert!(grade.abs() < 0.13, "{name}, road {number}: {grade} at {i}");
                    assert!(road.bank[i].abs() <= MAX_BANK);
                }
                // From where it leaves the lap (or joins it) the two share
                // the road between their walls, which stay as wide as the
                // lap's, until they part; from there on their walls stay
                // apart.
                let from_lap: Vec<usize> = if number % 2 == 0 {
                    (0..n).collect()
                } else {
                    (0..n).rev().collect()
                };
                let mut parted = false;
                for i in from_lap {
                    let lap = track.project_on_lap(road.centerline[i]).distance;
                    let apart = lap > WALL_DISTANCE + road.wall_at(i) + 2.0;
                    assert!(
                        apart || !parted,
                        "{name}, road {number}: {lap} m from the lap at {i}"
                    );
                    parted |= apart;
                    // Where they share it, their walls are as far out, so
                    // the road between them is the same seen from either.
                    if lap < WALL_DISTANCE + road.wall_at(i) {
                        assert!(road.wall_at(i) > WALL_DISTANCE - 0.25, "{name}: {i}");
                    }
                }
                assert!(parted, "{name}, road {number}");
            }
        }
    }

    #[test]
    fn bikes_on_a_branch_find_its_road() {
        for (name, track) in tracks_with_branches() {
            for looped in &track.loops {
                let into = track.road(looped.approach);
                let index = into.sample_count() - 20;
                let point = into.offset_point(index, 2.0);
                let height = track.surface_point_on(looped.approach, index, 2.0).y;
                let nearest = track.nearest_at(point, height);
                assert_eq!(nearest.road, looped.approach, "{name}");
                assert!((nearest.lateral - 2.0).abs() < 0.05);
                assert!((track.height_beside(&nearest) - height).abs() < 0.05);
                // Its walls close in towards the loop, and it counts as a
                // stretch of the lap further on than its start.
                assert!(track.edge_beside(&nearest) < WALL_DISTANCE);
                let start = track.nearest_at(into.centerline[0], into.elevation[0]);
                assert!(track.along(&nearest) > track.along(&start));
                let (_, heading) = track.pose_beside(&nearest, 0.0);
                assert!(wrap_angle(heading - looped.heading).abs() < 0.05);
            }
        }
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
