//! The inside of a toy store ([`ToyStore`]): the room, with tall shelves of
//! boxed toys round its walls and the lamps in its ceiling; the furniture
//! the roads run on (tables, a chair, a tall shelving unit); the toy tracks'
//! decks and what carries them (blue lattice towers under the slot-car
//! track, blue pillars under Hot Wheels', red posts and brackets off its
//! tower under the slide); castles of plastic bricks, loops of orange track,
//! mats on the floor and toys about; and electric trains going round their
//! tracks.
//!
//! Everything that stands still is built in a few big meshes, each of one
//! material, its colours in its vertices, worked out with the rest of the
//! circuit away from the main thread (see [`store_pieces`] and
//! [`bridge_pieces`]); the room's walls and ceiling cast no shadow, so that
//! the light from overhead comes in (see [`spawn_room`]).

use std::f32::consts::{PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Assets3d;
use super::japan::{Piece, PropPack};
use super::quads::MeshBuilder;
use super::scenery::{
    BridgeStyle, Castle, CastleKind, Furniture, FurnitureKind, Mat, MatKind, Scenery, Span, Tower,
    Toy, ToyKind, ToyLoop, ToyStore, Train, Wagon,
};
use super::structures::indices;
use super::textures::{TOY_BOXES, WorldTextures};
use crate::geometry::random01;
use crate::track::{BASE_ELEVATION, LOOP_WALL, Loop, Track, closed_curve};

/// The height over the sea of something `height` over the floor: the
/// heights of a toy store are the road's, the floor at 0.
pub fn y(height: f32) -> f32 {
    BASE_ELEVATION + height
}

/// The axes of something turned `heading` (as a road's heading): where its
/// x and its z point on the ground.
fn axes(heading: f32) -> (Vec2, Vec2) {
    let (sin, cos) = heading.sin_cos();
    (Vec2::new(cos, -sin), Vec2::new(sin, cos))
}

impl Furniture {
    /// `point` (x, z) in its own frame: across it (x) and along it (z),
    /// from its middle.
    fn local(&self, point: Vec2) -> Vec2 {
        let (x, z) = axes(self.heading);
        let d = point - self.center;
        Vec2::new(d.dot(x), d.dot(z))
    }

    /// The point of the ground `local` (x, z) from its middle in its own
    /// frame.
    fn world(&self, local: Vec2) -> Vec2 {
        let (x, z) = axes(self.heading);
        self.center + x * local.x + z * local.y
    }

    /// Whether `point` (x, z) is over it, `margin` in from its edges (or
    /// out, if negative).
    pub fn covers(&self, point: Vec2, margin: f32) -> bool {
        let local = self.local(point).abs();
        local.x <= self.size.x / 2.0 - margin && local.y <= self.size.y / 2.0 - margin
    }
}

/// How high (over the floor) whatever stands under `point` (x, z) is, below
/// `below`: the top of the highest piece of furniture there under it, or
/// the floor.
pub fn ground_under(store: &ToyStore, point: Vec2, below: f32) -> f32 {
    store
        .furniture
        .iter()
        .filter(|furniture| furniture.top <= below + 0.05 && furniture.covers(point, 0.0))
        .map(|furniture| furniture.top)
        .fold(0.0, f32::max)
}

/// The blue of the toy tracks' pillars and towers, a slide's red and
/// yellow, and Hot Wheels' orange.
const PILLAR_BLUE: Color = Color::srgb(0.1, 0.32, 0.82);
const SLIDE_RED: Color = Color::srgb(0.88, 0.12, 0.08);
const SLIDE_YELLOW: Color = Color::srgb(1.0, 0.8, 0.08);
const HOT_WHEELS_ORANGE: Color = Color::srgb(1.0, 0.42, 0.04);
/// How far out the decks of the toy tracks reach past their walls (m):
/// under their borders.
const DECK_PAST_WALLS: f32 = 0.8;

/// How far out the decks of `track`'s toy tracks reach from their centre
/// lines (m).
fn deck_edge(track: &Track) -> f32 {
    track.wall() + DECK_PAST_WALLS
}
/// Every how many points of the centre line a toy track has a support, and
/// how big a gap under it must be for one (m).
const SUPPORT_EVERY: usize = 9;
const LEAST_GAP: f32 = 0.8;

/// A toy track's deck: how thick, and the colour of its underside and
/// edges.
fn deck_of(style: BridgeStyle) -> (f32, Color) {
    match style {
        BridgeStyle::OrangeTrack => (0.35, HOT_WHEELS_ORANGE.mix(&Color::BLACK, 0.2)),
        BridgeStyle::Slide => (0.5, SLIDE_RED),
        _ => (0.4, Color::srgb(0.2, 0.2, 0.22)),
    }
}

/// The decks of the toy tracks over the floor (see [`BridgeStyle`]) and
/// what carries them (those on a stretch `within` of the lap, if given):
/// under a slot-car track blue lattice towers in pairs, under Hot Wheels'
/// a blue pillar in the middle with a cradle across, and under a slide red
/// posts in pairs, or, round a tower, brackets off it; each from the deck
/// down to whatever stands under it (a table, a chair or the floor), but
/// none where it would stand on a road below. And where a road banks
/// raised on what it lies on (see [`crate::track::Banking::raised`]), the
/// grey plastic risers under its outside, and off the bridges its deck too.
pub fn bridge_pieces(
    track: &Track,
    scenery: &Scenery,
    store: &ToyStore,
    within: Option<Span>,
) -> Vec<(Mesh, StandardMaterial)> {
    let lap = track.lap_length();
    let mut decks = MeshBuilder::default();
    let mut supports = MeshBuilder::default();
    for bridge in scenery.bridges.iter().filter(|bridge| {
        matches!(
            bridge.style,
            BridgeStyle::SlotTrack | BridgeStyle::OrangeTrack | BridgeStyle::Slide
        ) && within.is_none_or(|stretch| bridge.span.overlaps(&stretch, lap))
    }) {
        let indices = indices(track, bridge.span, lap);
        let (depth, color) = deck_of(bridge.style);
        deck(&mut decks, track, &indices, depth, color);
        for (k, &index) in indices.iter().enumerate() {
            if k % SUPPORT_EVERY != SUPPORT_EVERY / 2 {
                continue;
            }
            support(&mut supports, track, store, bridge.style, index, depth);
        }
    }
    // Off the bridges, on the floor, where the lap banks raised, its deck,
    // so that its raised outside shows how thick it is.
    let step = track.step_length();
    let on_a_bridge = |index: usize| {
        scenery
            .bridges
            .iter()
            .any(|bridge| bridge.span.contains(index as f32 * step, lap))
    };
    let wanted = |along: f32| within.is_none_or(|stretch| stretch.contains(along, lap));
    let raised: Vec<usize> = (0..track.sample_count())
        .filter(|&index| {
            track.lift(index) > LEAST_LIFT && !on_a_bridge(index) && wanted(index as f32 * step)
        })
        .collect();
    let (depth, color) = deck_of(BridgeStyle::SlotTrack);
    for run in runs(&raised, track.sample_count()) {
        deck(&mut decks, track, &run, depth, color);
    }
    // The risers, on the lap and on the roads off it.
    for number in 0..=track.side_roads().len() {
        let road = track.road(number);
        for index in (0..road.sample_count()).step_by(RISER_EVERY) {
            if road.lift(index) > LEAST_LIFT && wanted(road.lap_along_at(index)) {
                riser(&mut supports, road, store, index, depth);
            }
        }
    }
    let plastic = |roughness: f32| StandardMaterial {
        perceptual_roughness: roughness,
        ..default()
    };
    [(decks, plastic(0.45)), (supports, plastic(0.35))]
        .into_iter()
        .filter(|(mesh, _)| !mesh.is_empty())
        .map(|(mesh, material)| (mesh.build(), material))
        .collect()
}

/// A slab under the road along `indices`, out to [`deck_edge`], `depth`
/// thick: its underside and its edges (its top is the road's).
fn deck(mesh: &mut MeshBuilder, track: &Track, indices: &[usize], depth: f32, color: Color) {
    let road = |i: usize, lateral: f32| track.surface_point(i, lateral);
    let edge = deck_edge(track);
    let down = Vec3::Y * depth;
    for pair in indices.windows(2) {
        let (i, j) = (pair[0], pair[1]);
        mesh.quad(
            [
                (road(i, -edge) - down, Vec2::ZERO),
                (road(j, -edge) - down, Vec2::X),
                (road(j, edge) - down, Vec2::ONE),
                (road(i, edge) - down, Vec2::Y),
            ],
            Vec3::NEG_Y,
            color.mix(&Color::BLACK, 0.25),
        );
        for side in [-1.0, 1.0] {
            let out = (road(i, side * edge) - road(i, 0.0)).normalize_or_zero();
            mesh.quad(
                [
                    (road(i, side * edge), Vec2::ZERO),
                    (road(j, side * edge), Vec2::X),
                    (road(j, side * edge) - down, Vec2::ONE),
                    (road(i, side * edge) - down, Vec2::Y),
                ],
                out,
                color,
            );
        }
    }
}

/// What carries a toy track of `style` at point `index` of the lap, from
/// its deck (`depth` thick) down to whatever stands under it.
fn support(
    mesh: &mut MeshBuilder,
    track: &Track,
    store: &ToyStore,
    style: BridgeStyle,
    index: usize,
    depth: f32,
) {
    // Where it banks raised on what it lies on, its risers carry it.
    if style == BridgeStyle::SlotTrack && resting(track, store, index, depth) {
        return;
    }
    // Under the middle, or in pairs under the lanes, in from the walls.
    let wall = track.wall();
    let laterals: Vec<f32> = match style {
        BridgeStyle::OrangeTrack => vec![0.0],
        BridgeStyle::Slide => vec![-(wall - 2.5), wall - 2.5],
        _ => vec![-(wall - 4.5), wall - 4.5],
    };
    // Round a tower, a spiral hangs off it on brackets.
    let at = track.centerline()[index];
    if matches!(style, BridgeStyle::Slide | BridgeStyle::OrangeTrack)
        && let Some(tower) = store
            .towers
            .iter()
            .find(|tower| at.distance(tower.center) < TOWER_REACH)
    {
        let color = match style {
            BridgeStyle::Slide => SLIDE_RED,
            _ => PILLAR_BLUE,
        };
        bracket(mesh, track, tower, index, depth, color);
        return;
    }
    for &lateral in &laterals {
        let top = track.surface_point(index, lateral) - Vec3::Y * depth;
        let ground = Vec2::new(top.x, top.z);
        let foot = y(ground_under(store, ground, top.y - BASE_ELEVATION));
        if top.y - foot < LEAST_GAP || on_a_road_below(track, ground, foot, top.y) {
            continue;
        }
        let base = Vec3::new(top.x, foot, top.z);
        match style {
            BridgeStyle::OrangeTrack => {
                // A square pillar, and a cradle across under the deck.
                mesh.beam(base, top, 2.4, 2.4, PILLAR_BLUE);
                let across = track.surface_point(index, wall - 3.5)
                    - track.surface_point(index, -(wall - 3.5));
                let half = across / 2.0;
                let under = top - Vec3::Y * 0.6;
                mesh.beam(under - half, under + half, 1.2, 1.2, PILLAR_BLUE);
            }
            BridgeStyle::Slide => mesh.beam(base, top, 0.9, 0.9, SLIDE_RED),
            _ => lattice_tower(mesh, base, top.y - foot, 1.2, PILLAR_BLUE),
        }
    }
}

/// Whether a support from `foot` up to `top` at `point` (x, z) would stand
/// on a road or go through one on its way up, or stand right by one:
/// nothing stands on the roads.
fn on_a_road_below(track: &Track, point: Vec2, foot: f32, top: f32) -> bool {
    let mut height = foot;
    while height < top - 1.0 {
        let nearest = track.nearest_at(point, height);
        if nearest.distance < track.road(nearest.road).wall_at(nearest.index) + 2.0
            && (track.height_beside(&nearest) - height).abs() < 2.5
        {
            return true;
        }
        height += 2.0;
    }
    false
}

/// How much a road must be raised for its banking (m) to stand on risers
/// (see [`riser`]).
const LEAST_LIFT: f32 = 0.05;
/// Every how many points of its centre line a road banked raised has a
/// riser under it, how long each is along the road (m), and the least gap
/// under its deck one fills (m).
const RISER_EVERY: usize = 5;
const RISER_LENGTH: f32 = 0.8;
const RISER_GAP: f32 = 0.1;
/// The grey plastic of a slot-car track's banking supports.
const RISER_GREY: Color = Color::srgb(0.58, 0.6, 0.64);

/// The points of `indices` (in order, round a lap of `count` points) in
/// runs one after another, a run across the finish line joined into one.
pub(super) fn runs(indices: &[usize], count: usize) -> Vec<Vec<usize>> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for &index in indices {
        match runs.last_mut() {
            Some(run) if run.last() == Some(&(index - 1)) => run.push(index),
            _ => runs.push(vec![index]),
        }
    }
    if runs.len() > 1 && runs[0][0] == 0 && runs.last().unwrap().last() == Some(&(count - 1)) {
        let first = runs.remove(0);
        runs.last_mut().unwrap().extend(first);
    }
    runs
}

/// How far to the left of `road`'s centre line (m) its inside is at point
/// `index`, where it banks: its lower wall.
fn inside(road: &Track, index: usize) -> f32 {
    let wall = road.wall_at(index);
    if road.surface_point(index, wall).y <= road.surface_point(index, -wall).y {
        wall
    } else {
        -wall
    }
}

/// Whether `road`'s deck (`depth` thick) at point `index` rests on what
/// stands under it on its inside, where it banks raised: on a table, a
/// chair or the floor, whose risers carry its outside.
fn resting(road: &Track, store: &ToyStore, index: usize, depth: f32) -> bool {
    if road.lift(index) <= LEAST_LIFT {
        return false;
    }
    let under = road.surface_point(index, inside(road, index)) - Vec3::Y * depth;
    let ground = y(ground_under(
        store,
        Vec2::new(under.x, under.z),
        under.y - BASE_ELEVATION,
    ));
    under.y - ground < LEAST_GAP
}

/// A slot-car track's banking support under `road` (its deck `depth`
/// thick) at point `index`, where it banks raised resting on what stands
/// under it: a wedge of grey plastic `RISER_LENGTH` along the road, from
/// where the gap under its deck opens out to its outside edge, its top
/// under the deck and its foot on the table, the chair or the floor.
fn riser(mesh: &mut MeshBuilder, road: &Track, store: &ToyStore, index: usize, depth: f32) {
    if !resting(road, store, index, depth) {
        return;
    }
    let inner = inside(road, index);
    let edge = -inner.signum() * deck_edge(road);
    let under = |lateral: f32| road.surface_point(index, lateral) - Vec3::Y * depth;
    let at = under(inner);
    let ground = y(ground_under(
        store,
        Vec2::new(at.x, at.z),
        at.y - BASE_ELEVATION,
    ));
    // The gap under the deck grows steadily from its inside to its
    // outside: where it is `RISER_GAP`, the riser starts.
    let (low, high) = (under(inner).y - ground, under(edge).y - ground);
    if high <= RISER_GAP {
        return;
    }
    let start = ((RISER_GAP - low) / (high - low)).clamp(0.0, 1.0);
    let from = inner.lerp(edge, start);
    let (_, heading) = road.pose_at(index);
    let along = crate::geometry::forward(heading) * RISER_LENGTH / 2.0;
    let along = Vec3::new(along.x, 0.0, along.y);
    let (top_in, top_out) = (under(from), under(edge));
    let foot = |top: Vec3| Vec3::new(top.x, ground, top.z);
    let out = (top_out - top_in).with_y(0.0).normalize_or_zero();
    // Its sloping top, its outer face and its two sides.
    let (a, b) = (top_in - along, top_in + along);
    let (c, d) = (top_out + along, top_out - along);
    let quads = [
        ([a, b, c, d], Vec3::Y),
        ([foot(d), foot(c), c, d], out),
        ([foot(a), foot(d), d, a], -along),
        ([foot(b), foot(c), c, b], along),
    ];
    for (corners, facing) in quads {
        mesh.quad(
            [
                (corners[0], Vec2::ZERO),
                (corners[1], Vec2::X),
                (corners[2], Vec2::ONE),
                (corners[3], Vec2::Y),
            ],
            facing,
            RISER_GREY,
        );
    }
}

/// A tower of four thin posts `half` either side of `base`, `height` tall,
/// braced crosswise on each face, as the lattice of a toy track's pillars.
fn lattice_tower(mesh: &mut MeshBuilder, base: Vec3, height: f32, half: f32, color: Color) {
    const POST: f32 = 0.28;
    const BRACE: f32 = 0.16;
    const STOREY: f32 = 2.6;
    let corners = [
        Vec3::new(-half, 0.0, -half),
        Vec3::new(half, 0.0, -half),
        Vec3::new(half, 0.0, half),
        Vec3::new(-half, 0.0, half),
    ];
    for corner in corners {
        mesh.beam(
            base + corner,
            base + corner + Vec3::Y * height,
            POST,
            POST,
            color,
        );
    }
    let storeys = (height / STOREY).ceil().max(1.0) as usize;
    let rise = height / storeys as f32;
    for storey in 0..storeys {
        let low = base + Vec3::Y * (storey as f32 * rise);
        let high = low + Vec3::Y * rise;
        for k in 0..4 {
            let (a, b) = (corners[k], corners[(k + 1) % 4]);
            mesh.beam(low + a, high + b, BRACE, BRACE, color);
            mesh.beam(high + a, high + b, BRACE, BRACE, color);
        }
    }
}

/// How far from a tower's middle the road it carries on brackets runs at
/// most (m): round it, a spiral's turns over one another.
const TOWER_REACH: f32 = 80.0;

/// A bracket off a spiral's tower carrying its deck at point `index`, in
/// `color`: a beam from the tower out under the deck to its far edge, and a
/// strut up from the tower to the beam's end.
fn bracket(
    mesh: &mut MeshBuilder,
    track: &Track,
    tower: &Tower,
    index: usize,
    depth: f32,
    color: Color,
) {
    // The deck's edges, under it, the near one (to the tower) and the far
    // one: on a banked spiral the near one is the lower.
    let under = |lateral: f32| track.surface_point(index, lateral) - Vec3::Y * (depth + 0.5);
    let flat = |p: Vec3| Vec2::new(p.x, p.z);
    let (left, right) = (under(deck_edge(track)), under(-deck_edge(track)));
    let (near, far) = if flat(left).distance(tower.center) < flat(right).distance(tower.center) {
        (left, right)
    } else {
        (right, left)
    };
    let toward = (flat(near) - tower.center).normalize_or_zero();
    let root = tower.center + toward * tower.radius;
    let root = Vec3::new(root.x, near.y, root.y);
    mesh.beam(root, near, 0.9, 0.9, color);
    mesh.beam(near, far, 0.9, 0.9, color);
    mesh.beam(root - Vec3::Y * 6.0, near, 0.6, 0.6, color);
}

/// The room's walls and its ceiling, with its lamps: they cast no shadow,
/// so that the light comes in from overhead as from the lamps.
pub fn spawn_room(world: &mut ChildSpawnerCommands, store: &ToyStore, assets: &mut Assets3d) {
    let (min, max) = store.room;
    let (floor, top) = (y(-0.1), y(store.ceiling));
    let mut walls = MeshBuilder::default();
    let corner = |x: f32, z: f32, height: f32| Vec3::new(x, height, z);
    // Pale blue walls over a blue skirting, and a band of bright stripes
    // round the top, under the ceiling, over the shelves.
    let paint = Color::srgb(0.78, 0.86, 0.95);
    let skirting = Color::srgb(0.3, 0.45, 0.85);
    let bands = [
        Color::srgb(0.95, 0.25, 0.2),
        Color::srgb(1.0, 0.78, 0.15),
        Color::srgb(0.25, 0.7, 0.35),
    ];
    for (a, b) in [
        (Vec2::new(min.x, min.y), Vec2::new(max.x, min.y)),
        (Vec2::new(max.x, min.y), Vec2::new(max.x, max.y)),
        (Vec2::new(max.x, max.y), Vec2::new(min.x, max.y)),
        (Vec2::new(min.x, max.y), Vec2::new(min.x, min.y)),
    ] {
        let inward = (min + max) / 2.0 - (a + b) / 2.0;
        let facing = Vec3::new(inward.x, 0.0, inward.y).normalize_or_zero();
        let band = |k: usize| y(WALL_SHELF_HEIGHT + 2.0 + 2.0 * k as f32);
        for (from, to, color) in [
            (floor, y(3.0), skirting),
            (y(3.0), band(0), paint),
            (band(0), band(1), bands[0]),
            (band(1), band(2), bands[1]),
            (band(2), band(3), bands[2]),
            (band(3), top, paint),
        ] {
            walls.quad(
                [
                    (corner(a.x, a.y, from), Vec2::ZERO),
                    (corner(b.x, b.y, from), Vec2::X),
                    (corner(b.x, b.y, to), Vec2::ONE),
                    (corner(a.x, a.y, to), Vec2::Y),
                ],
                facing,
                color,
            );
        }
    }
    let mut ceiling = MeshBuilder::default();
    const CEILING_TILE: f32 = 14.4;
    ceiling.quad(
        [
            (corner(min.x, min.y, top), min / CEILING_TILE),
            (
                corner(max.x, min.y, top),
                Vec2::new(max.x, min.y) / CEILING_TILE,
            ),
            (corner(max.x, max.y, top), max / CEILING_TILE),
            (
                corner(min.x, max.y, top),
                Vec2::new(min.x, max.y) / CEILING_TILE,
            ),
        ],
        Vec3::NEG_Y,
        Color::WHITE,
    );
    // Panels of light in rows, and round downlights between them.
    let mut lights = MeshBuilder::default();
    const PANEL: Vec2 = Vec2::new(14.4, 7.2);
    const PANEL_EVERY: Vec2 = Vec2::new(43.2, 36.0);
    let low = top - 0.08;
    let count = ((max - min) / PANEL_EVERY).floor();
    let start = (min + max) / 2.0 - (count - Vec2::ONE) * PANEL_EVERY / 2.0;
    for i in 0..count.x as usize {
        for j in 0..count.y as usize {
            let center = start + Vec2::new(i as f32, j as f32) * PANEL_EVERY;
            let (a, b) = (center - PANEL / 2.0, center + PANEL / 2.0);
            lights.quad(
                [
                    (corner(a.x, a.y, low), Vec2::ZERO),
                    (corner(b.x, a.y, low), Vec2::X),
                    (corner(b.x, b.y, low), Vec2::ONE),
                    (corner(a.x, b.y, low), Vec2::Y),
                ],
                Vec3::NEG_Y,
                Color::WHITE,
            );
            let spot = center + PANEL_EVERY / 2.0;
            if spot.x < max.x - 10.0 && spot.y < max.y - 10.0 {
                disc(&mut lights, corner(spot.x, spot.y, low), 1.6, Color::WHITE);
            }
        }
    }
    let mut spawn = |mesh: MeshBuilder, material: StandardMaterial| {
        world.spawn((
            Mesh3d(assets.meshes.add(mesh.build())),
            MeshMaterial3d(assets.materials.add(material)),
            NotShadowCaster,
        ));
    };
    spawn(
        walls,
        StandardMaterial {
            perceptual_roughness: 0.9,
            ..default()
        },
    );
    spawn(
        ceiling,
        // Lit by the lamps in it, from below, which the light from
        // overhead does not do.
        StandardMaterial {
            base_color_texture: Some(assets.textures.ceiling.clone()),
            emissive: LinearRgba::rgb(0.35, 0.35, 0.34),
            perceptual_roughness: 0.9,
            ..default()
        },
    );
    spawn(
        lights,
        StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(6.0, 5.8, 5.4),
            unlit: true,
            ..default()
        },
    );
}

/// A flat round disc facing down at `center`, `radius` across.
fn disc(mesh: &mut MeshBuilder, center: Vec3, radius: f32, color: Color) {
    const SIDES: usize = 12;
    for k in 0..SIDES {
        let angle = |k: usize| k as f32 / SIDES as f32 * TAU;
        let at = |k: usize| center + Vec3::new(angle(k).cos(), 0.0, angle(k).sin()) * radius;
        mesh.triangle(
            [
                (center, Vec2::splat(0.5)),
                (at(k), Vec2::ZERO),
                (at(k + 1), Vec2::X),
            ],
            Vec3::NEG_Y,
            color,
        );
    }
}

/// A box `half` its size either way round `center`, turned `yaw` about
/// the vertical (as a road's heading), each face of `color`.
fn cuboid(mesh: &mut MeshBuilder, center: Vec3, half: Vec3, yaw: f32, color: Color) {
    textured_box(
        mesh,
        center,
        half,
        yaw,
        |_| (Vec2::ZERO, Vec2::ONE),
        |_| color,
    );
}

/// A box as [`cuboid`], each face (by its outward direction in the box's
/// own frame) drawn over the rectangle of the texture `uv` gives it (its
/// least corner and its size), in the colour `color` gives it.
fn textured_box(
    mesh: &mut MeshBuilder,
    center: Vec3,
    half: Vec3,
    yaw: f32,
    uv: impl Fn(Vec3) -> (Vec2, Vec2),
    color: impl Fn(Vec3) -> Color,
) {
    let rotation = Quat::from_rotation_y(yaw);
    let corner = |x: f32, y: f32, z: f32| center + rotation * (half * Vec3::new(x, y, z));
    for normal in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
        // Two axes across the face, so that its texture stands upright on
        // the sides.
        let (u, v) = if normal.y.abs() > 0.5 {
            (Vec3::X, Vec3::Z)
        } else {
            (normal.cross(Vec3::Y), Vec3::NEG_Y)
        };
        let (from, size) = uv(normal);
        let at = |a: f32, b: f32| {
            let p = normal + u * a + v * b;
            (
                corner(p.x, p.y, p.z),
                from + size * Vec2::new((a + 1.0) / 2.0, (b + 1.0) / 2.0),
            )
        };
        mesh.quad(
            [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)],
            rotation * normal,
            color(normal),
        );
    }
}

/// A cylinder standing on `base`, `radius` round and `height` tall, its
/// top closed, coloured by `color` (from the height up its side, 0 to 1).
fn cylinder(
    mesh: &mut MeshBuilder,
    base: Vec3,
    radius: f32,
    height: f32,
    sides: usize,
    color: impl Fn(f32) -> Color,
) {
    let around = |k: usize| {
        let angle = k as f32 / sides as f32 * TAU;
        Vec3::new(angle.cos(), 0.0, angle.sin())
    };
    let bands = 8;
    for band in 0..bands {
        let (t0, t1) = (band as f32 / bands as f32, (band + 1) as f32 / bands as f32);
        let paint = color((t0 + t1) / 2.0);
        for k in 0..sides {
            let (a, b) = (around(k), around(k + 1));
            mesh.quad(
                [
                    (base + a * radius + Vec3::Y * height * t0, Vec2::ZERO),
                    (base + b * radius + Vec3::Y * height * t0, Vec2::X),
                    (base + b * radius + Vec3::Y * height * t1, Vec2::ONE),
                    (base + a * radius + Vec3::Y * height * t1, Vec2::Y),
                ],
                (a + b).normalize(),
                paint,
            );
        }
    }
    let top = base + Vec3::Y * height;
    for k in 0..sides {
        mesh.triangle(
            [
                (top, Vec2::splat(0.5)),
                (top + around(k) * radius, Vec2::ZERO),
                (top + around(k + 1) * radius, Vec2::X),
            ],
            Vec3::Y,
            color(1.0),
        );
    }
}

/// A cone on a round base at `base`, `radius` round, its point `height`
/// over it, open underneath.
fn cone(mesh: &mut MeshBuilder, base: Vec3, radius: f32, height: f32, sides: usize, color: Color) {
    let tip = base + Vec3::Y * height;
    for k in 0..sides {
        let angle = |k: usize| k as f32 / sides as f32 * TAU;
        let at = |k: usize| base + Vec3::new(angle(k).cos(), 0.0, angle(k).sin()) * radius;
        let middle = (angle(k) + angle(k + 1)) / 2.0;
        let out = Vec3::new(middle.cos() * height, radius, middle.sin() * height).normalize();
        mesh.triangle(
            [
                (at(k), Vec2::ZERO),
                (at(k + 1), Vec2::X),
                (tip, Vec2::splat(0.5)),
            ],
            out,
            color,
        );
    }
}

/// A ball at `center`, `radius` round (squashed by `scale` along each
/// axis), turned `yaw`, each of its faces painted by `color` (from the way
/// it faces, in its own frame).
fn ball(
    mesh: &mut MeshBuilder,
    center: Vec3,
    radius: f32,
    scale: Vec3,
    yaw: f32,
    color: impl Fn(Vec3) -> Color,
) {
    const RINGS: usize = 8;
    const SEGMENTS: usize = 12;
    let rotation = Quat::from_rotation_y(yaw);
    let unit = |ring: usize, segment: usize| {
        let polar = ring as f32 / RINGS as f32 * PI;
        let angle = segment as f32 / SEGMENTS as f32 * TAU;
        Vec3::new(
            polar.sin() * angle.cos(),
            polar.cos(),
            polar.sin() * angle.sin(),
        )
    };
    let at = |n: Vec3| center + rotation * (n * radius * scale);
    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let corners = [
                unit(ring, segment),
                unit(ring + 1, segment),
                unit(ring + 1, segment + 1),
                unit(ring, segment + 1),
            ];
            let middle = corners.iter().sum::<Vec3>().normalize_or_zero();
            let paint = color(middle);
            let facing = rotation * (middle / scale).normalize_or_zero();
            if ring == 0 || ring == RINGS - 1 {
                // A triangle at each pole.
                let [a, b, c, d] = corners.map(at);
                let (p, q, r) = if ring == 0 { (a, b, c) } else { (a, b, d) };
                mesh.triangle([(p, Vec2::ZERO), (q, Vec2::X), (r, Vec2::Y)], facing, paint);
            } else {
                mesh.quad(corners.map(|n| (at(n), Vec2::ZERO)), facing, paint);
            }
        }
    }
}

/// Plastic toys and furniture, their colours in their vertices.
fn plastic(roughness: f32) -> StandardMaterial {
    StandardMaterial {
        perceptual_roughness: roughness,
        ..default()
    }
}

/// The meshes of a toy store's room and of what stands in it (all but its
/// walls and ceiling, see [`spawn_room`], and its trains, see
/// [`spawn_trains`]): shelves of boxed toys round the walls and in the
/// shelving units, the furniture, the towers, castles, loops of track,
/// mats, toys and the trains' tracks.
pub fn store_pieces(
    track: &Track,
    scenery: &Scenery,
    store: &ToyStore,
    textures: &WorldTextures,
) -> Vec<(Mesh, StandardMaterial)> {
    let mut solid = MeshBuilder::default();
    let mut boxes = MeshBuilder::default();
    let mut baseplates = MeshBuilder::default();
    let mut foam = MeshBuilder::default();
    let mut blocks = MeshBuilder::default();
    let mut bricks = MeshBuilder::default();
    wall_shelves(&mut solid, &mut boxes, store);
    for (index, furniture) in store.furniture.iter().enumerate() {
        match furniture.kind {
            FurnitureKind::Table => table(&mut solid, furniture, track),
            FurnitureKind::Diorama => {
                table(&mut solid, furniture, track);
                baseplate(&mut baseplates, furniture);
            }
            FurnitureKind::Chair => chair(&mut solid, furniture),
            FurnitureKind::Shelves => {
                shelving_unit(&mut solid, &mut boxes, furniture, index as u32)
            }
        }
    }
    for tower in store.towers {
        self::tower(&mut solid, tower);
    }
    // The castles a road runs through; the others are Meshy's (see
    // `spawn_models`).
    for castle in store
        .castles
        .iter()
        .filter(|castle| castle.kind == CastleKind::Through)
    {
        self::castle(&mut solid, castle, track.wall());
    }
    for toy_loop in store.loops {
        self::toy_loop(&mut solid, toy_loop);
    }
    for looped in track.loops() {
        loop_stand(&mut solid, looped);
    }
    for mat in store.mats {
        self::mat(&mut solid, &mut foam, mat);
    }
    for (index, toy) in store.toys.iter().enumerate() {
        self::toy(&mut solid, &mut blocks, toy, index as u32, store.ceiling);
    }
    for train in store.trains {
        rails(&mut solid, train);
    }
    block_walls(&mut blocks, track, scenery);
    for (index, built) in store.bricks.iter().enumerate() {
        super::bricks::build(&mut bricks, built, index as u32);
    }
    super::bricks::walls(&mut bricks, track, scenery);
    let textured = |texture: &Handle<Image>, roughness: f32| StandardMaterial {
        base_color_texture: Some(texture.clone()),
        perceptual_roughness: roughness,
        ..default()
    };
    [
        (solid, plastic(0.45)),
        (boxes, textured(&textures.toy_boxes, 0.55)),
        (baseplates, textured(&textures.baseplate, 0.4)),
        (foam, textured(&textures.foam, 0.85)),
        (blocks, textured(&textures.blocks, 0.6)),
        (bricks, super::bricks::material()),
    ]
    .into_iter()
    .filter(|(mesh, _)| !mesh.is_empty())
    .map(|(mesh, material)| (mesh.build(), material))
    .collect()
}

/// How deep the shelves round the walls are (m), how tall, and how high
/// each shelf over the one under it.
const WALL_SHELF_DEPTH: f32 = 9.0;
const WALL_SHELF_HEIGHT: f32 = 43.0;
const SHELF_STEP: f32 = 6.0;
/// The white of the shelving's frames and boards.
const SHELF_WHITE: Color = Color::srgb(0.93, 0.93, 0.91);

/// Shelves all round the room's walls, full of boxed toys facing in.
fn wall_shelves(solid: &mut MeshBuilder, boxes: &mut MeshBuilder, store: &ToyStore) {
    let (min, max) = store.room;
    let d = WALL_SHELF_DEPTH;
    // Each wall's run: where it starts and ends (along the wall, the
    // corners left to the walls either side), and the way into the room.
    let runs = [
        (
            Vec2::new(min.x, min.y + d),
            Vec2::new(max.x, min.y + d),
            Vec2::Y,
        ),
        (
            Vec2::new(max.x - d, min.y + d),
            Vec2::new(max.x - d, max.y - d),
            -Vec2::X,
        ),
        (
            Vec2::new(max.x, max.y - d),
            Vec2::new(min.x, max.y - d),
            -Vec2::Y,
        ),
        (
            Vec2::new(min.x + d, max.y - d),
            Vec2::new(min.x + d, min.y + d),
            Vec2::X,
        ),
    ];
    for (k, (from, to, inward)) in runs.into_iter().enumerate() {
        shelf_run(
            solid,
            boxes,
            from,
            to,
            inward,
            d,
            WALL_SHELF_HEIGHT,
            k as u32 * 1000,
        );
    }
}

/// A run of shelves from `from` to `to` along their front, `depth` deep
/// back from it (against `inward`, the way they face), `height` tall: a
/// plinth, uprights, boards every [`SHELF_STEP`], a back, and boxed toys on
/// each board, each a box of its own size and colour.
#[allow(clippy::too_many_arguments)]
fn shelf_run(
    solid: &mut MeshBuilder,
    boxes: &mut MeshBuilder,
    from: Vec2,
    to: Vec2,
    inward: Vec2,
    depth: f32,
    height: f32,
    seed: u32,
) {
    let along = (to - from).normalize_or_zero();
    let length = from.distance(to);
    let yaw = crate::geometry::heading_of(inward);
    let at = |s: f32, back: f32, up: f32| {
        let ground = from + along * s - inward * back;
        Vec3::new(ground.x, y(up), ground.y)
    };
    // The back panel, the plinth and the top.
    let middle = |up: f32| at(length / 2.0, depth / 2.0, up);
    let half = |x: f32, up: f32, back: f32| Vec3::new(x, up, back);
    // In its own frame (yaw), x is across its face and z out of it: the
    // half sizes along the run are its length's.
    cuboid(
        solid,
        at(length / 2.0, depth - 0.4, height / 2.0),
        half(length / 2.0, height / 2.0, 0.4),
        yaw,
        Color::srgb(0.98, 0.9, 0.8),
    );
    cuboid(
        solid,
        middle(0.6),
        half(length / 2.0, 0.6, depth / 2.0),
        yaw,
        Color::srgb(0.3, 0.45, 0.85),
    );
    let boards = (height / SHELF_STEP).floor() as u32;
    for board in 1..=boards {
        let up = board as f32 * SHELF_STEP;
        cuboid(
            solid,
            middle(up - 0.25),
            half(length / 2.0, 0.25, depth / 2.0),
            yaw,
            SHELF_WHITE,
        );
    }
    let uprights = (length / 24.0).ceil().max(1.0) as u32;
    for k in 0..=uprights {
        let s = length * k as f32 / uprights as f32;
        cuboid(
            solid,
            at(s, depth / 2.0, height / 2.0),
            half(0.4, height / 2.0, depth / 2.0),
            yaw,
            SHELF_WHITE,
        );
    }
    // The boxes on the floor's plinth and on every board but the top one.
    for shelf in 0..boards {
        let floor = if shelf == 0 {
            1.2
        } else {
            shelf as f32 * SHELF_STEP
        };
        let room = SHELF_STEP - 0.6 - if shelf == 0 { 1.2 } else { 0.0 };
        let mut s = 0.8;
        let mut n = seed + shelf * 97;
        while s < length - 0.8 {
            n += 1;
            let width = 3.0 + 3.5 * random01(n * 3);
            let tall = room * (0.6 + 0.38 * random01(n * 3 + 1));
            let deep = depth * (0.55 + 0.35 * random01(n * 3 + 2));
            if s + width > length - 0.8 {
                break;
            }
            let design = (random01(n * 5 + 7) * TOY_BOXES as f32) as u32 % TOY_BOXES;
            let column = design as f32 / TOY_BOXES as f32;
            let width_uv = 1.0 / TOY_BOXES as f32;
            textured_box(
                boxes,
                at(s + width / 2.0, deep / 2.0 + 0.3, floor + tall / 2.0),
                Vec3::new(width / 2.0, tall / 2.0, deep / 2.0),
                yaw,
                |normal| {
                    if normal.z < -0.5 {
                        // Its front, facing out of the shelves (its frame's
                        // -z, as a heading's).
                        (Vec2::new(column, 0.0), Vec2::new(width_uv, 1.0))
                    } else {
                        // Its sides and top in its plain colour.
                        (Vec2::new(column + width_uv * 0.5, 0.93), Vec2::ZERO)
                    }
                },
                |_| Color::WHITE,
            );
            s += width + 0.3 + 0.6 * random01(n * 7);
        }
    }
}

/// How thick a table's top is (m), how wide its legs and how far in from
/// its corners.
const TABLE_TOP: f32 = 1.2;
const TABLE_LEG: f32 = 2.4;
const LEG_INSET: f32 = 4.0;
/// How far apart a long table's legs are at most (m).
const LEGS_EVERY: f32 = 140.0;

/// How far a leg moves along its table's side at a time, off a road under
/// the table (m).
const LEG_SHIFT: f32 = 4.0;

/// Where a table's legs stand (x, z): at its corners, and along its long
/// sides as far apart as [`LEGS_EVERY`] at most; each standing on a road of
/// `track` under the table moved along its side towards the middle until it
/// is clear of it, or left out if it never is.
pub fn table_legs(furniture: &Furniture, track: &Track) -> Vec<Vec2> {
    let half = furniture.size / 2.0 - Vec2::splat(LEG_INSET);
    let along = |extent: f32| {
        let count = (2.0 * extent / LEGS_EVERY).ceil().max(1.0) as usize;
        (0..=count)
            .map(|k| -extent + 2.0 * extent * k as f32 / count as f32)
            .collect::<Vec<_>>()
    };
    let mut legs = Vec::new();
    for x in along(half.x) {
        for z in along(half.y) {
            if x.abs() < half.x - 0.01 && z.abs() < half.y - 0.01 {
                continue;
            }
            // On a side along x (at either end of z) it moves along x, and
            // on one along z along z.
            let (start, toward) = if z.abs() >= half.y - 0.01 {
                (Vec2::new(x, z), Vec2::new(-x.signum(), 0.0))
            } else {
                (Vec2::new(x, z), Vec2::new(0.0, -z.signum()))
            };
            let reach = (start * toward.abs()).length();
            let clear = (0..)
                .map(|k| start + toward * (k as f32 * LEG_SHIFT))
                .take_while(|local| (*local - start).length() <= reach)
                .map(|local| furniture.world(local))
                .find(|&at| !on_a_road_under(track, at, furniture.top));
            legs.extend(clear);
        }
    }
    legs
}

/// Whether a leg standing at `point` (x, z) under a top `top` over the
/// floor would stand on a road running under it, or right by one.
fn on_a_road_under(track: &Track, point: Vec2, top: f32) -> bool {
    (0..=track.side_roads().len()).any(|number| {
        let road = track.road(number);
        road.centerline()
            .iter()
            .zip(road.elevations())
            .enumerate()
            .any(|(index, (&at, &elevation))| {
                elevation - BASE_ELEVATION < top - 1.0
                    && at.distance(point) < road.wall_at(index) + TABLE_LEG + 1.0
            })
    })
}

/// A table: its top, an apron under its edges and its legs, off the roads
/// of `track` running under it (see [`table_legs`]).
fn table(mesh: &mut MeshBuilder, furniture: &Furniture, track: &Track) {
    let center = |up: f32| Vec3::new(furniture.center.x, y(up), furniture.center.y);
    let half = furniture.size / 2.0;
    let top = furniture.top;
    let edge = if furniture.kind == FurnitureKind::Diorama {
        furniture.color
    } else {
        furniture.color.mix(&Color::BLACK, 0.08)
    };
    // Its edges painted, its underside plain wood.
    let under = Color::srgb(0.7, 0.55, 0.38);
    textured_box(
        mesh,
        center(top - TABLE_TOP / 2.0),
        Vec3::new(half.x, TABLE_TOP / 2.0, half.y),
        furniture.heading,
        |_| (Vec2::ZERO, Vec2::ONE),
        |normal| if normal.y < -0.5 { under } else { edge },
    );
    // The apron, a little in from the edge.
    let apron = Vec3::new(half.x - 1.5, 1.0, half.y - 1.5);
    let leg_color = furniture.color.mix(&Color::BLACK, 0.25);
    for (x, z) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        let offset = furniture.world(Vec2::new(x * apron.x, z * apron.z)) - furniture.center;
        let half = if x != 0.0 {
            Vec3::new(0.3, 1.0, apron.z)
        } else {
            Vec3::new(apron.x, 1.0, 0.3)
        };
        cuboid(
            mesh,
            center(top - TABLE_TOP - 1.0) + Vec3::new(offset.x, 0.0, offset.y),
            half,
            furniture.heading,
            leg_color,
        );
    }
    let leg = (top - TABLE_TOP) / 2.0;
    for at in table_legs(furniture, track) {
        cuboid(
            mesh,
            Vec3::new(at.x, y(leg), at.y),
            Vec3::new(TABLE_LEG / 2.0, leg, TABLE_LEG / 2.0),
            furniture.heading,
            leg_color,
        );
    }
}

/// The studded green top of a diorama table, just over its top.
fn baseplate(mesh: &mut MeshBuilder, furniture: &Furniture) {
    // Four studs to the texture, as far apart as the bricks' (see
    // `textures::baseplate`).
    const STUDS: f32 = 4.0 * super::bricks::STUD;
    let half = furniture.size / 2.0;
    let corner = |x: f32, z: f32| {
        let ground = furniture.world(Vec2::new(x, z));
        (
            Vec3::new(ground.x, y(furniture.top + 0.02), ground.y),
            Vec2::new(x, z) / STUDS,
        )
    };
    mesh.quad(
        [
            corner(-half.x, -half.y),
            corner(half.x, -half.y),
            corner(half.x, half.y),
            corner(-half.x, half.y),
        ],
        Vec3::Y,
        Color::WHITE,
    );
}

/// A chair: its seat, four legs, and its back along its north edge (its
/// local -z), two posts and three rails.
fn chair(mesh: &mut MeshBuilder, furniture: &Furniture) {
    let half = furniture.size / 2.0;
    let seat = furniture.top;
    let place = |local: Vec2, up: f32| {
        let ground = furniture.world(local);
        Vec3::new(ground.x, y(up), ground.y)
    };
    cuboid(
        mesh,
        place(Vec2::ZERO, seat - 0.6),
        Vec3::new(half.x, 0.6, half.y),
        furniture.heading,
        furniture.color,
    );
    let legs = furniture.color.mix(&Color::BLACK, 0.2);
    for (x, z) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        let at = Vec2::new(x * (half.x - 1.5), z * (half.y - 1.5));
        cuboid(
            mesh,
            place(at, (seat - 1.2) / 2.0),
            Vec3::new(1.2, (seat - 1.2) / 2.0, 1.2),
            furniture.heading,
            legs,
        );
    }
    // The back, out of the way of the road along the seat.
    const BACK: f32 = 16.0;
    for x in [-1.0, 1.0] {
        let at = Vec2::new(x * (half.x - 1.5), -half.y + 1.2);
        cuboid(
            mesh,
            place(at, seat + BACK / 2.0),
            Vec3::new(1.2, BACK / 2.0, 1.2),
            furniture.heading,
            legs,
        );
    }
    for rail in 0..3 {
        let up = seat + BACK * (0.45 + 0.25 * rail as f32);
        cuboid(
            mesh,
            place(Vec2::new(0.0, -half.y + 1.2), up),
            Vec3::new(half.x - 1.5, 1.2, 0.8),
            furniture.heading,
            furniture.color,
        );
    }
}

/// A tall shelving unit: shelves of boxed toys on both its long sides, back
/// to back, under its top.
fn shelving_unit(
    solid: &mut MeshBuilder,
    boxes: &mut MeshBuilder,
    furniture: &Furniture,
    seed: u32,
) {
    let half = furniture.size / 2.0;
    // Along its longer side.
    let (length, across) = if half.x > half.y {
        (Vec2::new(half.x, 0.0), Vec2::new(0.0, half.y))
    } else {
        (Vec2::new(0.0, half.y), Vec2::new(half.x, 0.0))
    };
    for side in [1.0, -1.0] {
        let front = |s: f32| furniture.world(across * side + length * s);
        let inward = (furniture.world(across * side) - furniture.center).normalize_or_zero();
        shelf_run(
            solid,
            boxes,
            front(-1.0),
            front(1.0),
            inward,
            across.length(),
            furniture.top,
            seed * 7919 + if side > 0.0 { 0 } else { 500 },
        );
    }
    // Its top, under the road's deck.
    cuboid(
        solid,
        Vec3::new(
            furniture.center.x,
            y(furniture.top - 0.3),
            furniture.center.y,
        ),
        Vec3::new(half.x, 0.3, half.y),
        furniture.heading,
        furniture.color,
    );
}

/// A tower a spiral road winds round: a helter-skelter's, in red and white
/// bands under a yellow pointed roof, or a blue column with a cap.
fn tower(mesh: &mut MeshBuilder, tower: &Tower) {
    let base = Vec3::new(tower.center.x, y(0.0), tower.center.y);
    match tower.kind {
        super::scenery::TowerKind::HelterSkelter => {
            let bands = (tower.height / 4.0).round().max(1.0);
            cylinder(mesh, base, tower.radius, tower.height, 16, |t| {
                if (t * bands).floor() as i32 % 2 == 0 {
                    SLIDE_RED
                } else {
                    Color::srgb(0.96, 0.95, 0.92)
                }
            });
            let eaves = base + Vec3::Y * tower.height;
            cone(mesh, eaves, tower.radius + 3.0, 9.0, 16, SLIDE_YELLOW);
            cylinder(mesh, eaves + Vec3::Y * 9.0, 0.3, 4.0, 6, |_| SLIDE_RED);
        }
        super::scenery::TowerKind::Column => {
            cylinder(mesh, base, tower.radius, tower.height, 16, |_| PILLAR_BLUE);
            cylinder(
                mesh,
                base + Vec3::Y * tower.height,
                tower.radius + 1.5,
                1.2,
                16,
                |_| HOT_WHEELS_ORANGE,
            );
        }
    }
}

/// The grey of plastic bricks and the red of their roofs.
const BRICK_GREY: Color = Color::srgb(0.62, 0.64, 0.66);
const ROOF_RED: Color = Color::srgb(0.8, 0.12, 0.1);

/// A castle of plastic bricks a road runs through: a curtain wall with
/// battlements, a round tower at each corner under a red pointed roof, and
/// a gate in each end wall (as wide as the road's walls, `road_wall` from
/// its middle, stand apart, and a little more) under a gatehouse.
fn castle(mesh: &mut MeshBuilder, castle: &Castle, road_wall: f32) {
    let (x, z) = axes(castle.heading);
    let place = |local: Vec2, up: f32| {
        let ground = castle.center + x * local.x + z * local.y;
        Vec3::new(ground.x, y(castle.base + up), ground.y)
    };
    let half = castle.size / 2.0;
    const WALL: f32 = 6.0;
    const THICK: f32 = 1.0;
    let gate = road_wall + 1.5;
    let mut walls = vec![
        (Vec2::new(half.x, -half.y), Vec2::new(half.x, half.y)),
        (Vec2::new(-half.x, half.y), Vec2::new(-half.x, -half.y)),
    ];
    for end in [-half.y, half.y] {
        walls.push((Vec2::new(-half.x, end), Vec2::new(-gate, end)));
        walls.push((Vec2::new(gate, end), Vec2::new(half.x, end)));
    }
    for (a, b) in walls {
        let middle = (a + b) / 2.0;
        let along = (b - a) / 2.0;
        let extent = Vec3::new(
            along.x.abs().max(THICK),
            WALL / 2.0,
            along.y.abs().max(THICK),
        );
        cuboid(
            mesh,
            place(middle, WALL / 2.0),
            extent,
            castle.heading,
            BRICK_GREY,
        );
        // Battlements: a merlon every 2.4 m along its top.
        let count = (a.distance(b) / 2.4).floor() as usize;
        for k in 0..count {
            let at = a + (b - a) * ((k as f32 + 0.5) / count as f32);
            cuboid(
                mesh,
                place(at, WALL + 0.6),
                Vec3::new(0.6, 0.6, 0.6),
                castle.heading,
                BRICK_GREY,
            );
        }
    }
    for (sx, sz) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        let at = place(Vec2::new(sx * half.x, sz * half.y), 0.0);
        cylinder(mesh, at, 3.0, WALL + 4.0, 10, |_| BRICK_GREY);
        cone(mesh, at + Vec3::Y * (WALL + 4.0), 3.8, 5.0, 10, ROOF_RED);
    }
    gatehouses(mesh, castle, gate, &place);
}

/// Over each gate of a castle a road runs through (`gate` either side of the
/// road's middle), a gatehouse: a tall round tower either side, under a red
/// roof, and a battlemented bridge between them, high over the riders.
fn gatehouses(
    mesh: &mut MeshBuilder,
    castle: &Castle,
    gate: f32,
    place: &impl Fn(Vec2, f32) -> Vec3,
) {
    const TOWER: f32 = 3.2;
    const TALL: f32 = 15.0;
    const ARCH: f32 = 10.0;
    const BRIDGE: f32 = 2.5;
    let half = castle.size / 2.0;
    for end in [-half.y, half.y] {
        for side in [-1.0, 1.0] {
            let at = place(Vec2::new(side * (gate + TOWER), end), 0.0);
            cylinder(mesh, at, TOWER, TALL, 12, |_| BRICK_GREY);
            cone(mesh, at + Vec3::Y * TALL, TOWER + 0.8, 6.0, 12, ROOF_RED);
        }
        cuboid(
            mesh,
            place(Vec2::new(0.0, end), ARCH + BRIDGE / 2.0),
            Vec3::new(gate + TOWER, BRIDGE / 2.0, 1.6),
            castle.heading,
            BRICK_GREY.mix(&Color::WHITE, 0.1),
        );
        let count = ((2.0 * gate) / 2.4).floor() as usize;
        for k in 0..count {
            let across = -gate + 2.0 * gate * (k as f32 + 0.5) / count as f32;
            for face in [-1.2, 1.2] {
                cuboid(
                    mesh,
                    place(Vec2::new(across, end + face), ARCH + BRIDGE + 0.6),
                    Vec3::new(0.6, 0.6, 0.4),
                    castle.heading,
                    BRICK_GREY,
                );
            }
        }
    }
}

/// The blue stand of a loop the bikes ride (see
/// [`super::landmarks::spawn_ridden_loops`]), as a toy loop's: a post
/// either side of it, beyond its edges (its running surface drifts from the
/// way in to the way out), up to its middle, on a foot, and a bar across
/// between them through its middle.
fn loop_stand(mesh: &mut MeshBuilder, looped: &Loop) {
    const POST: f32 = 1.6;
    let left = looped.left();
    let reach = LOOP_WALL + 2.0;
    let middle = looped.height + looped.radius;
    let feet = [
        looped.mouth + left * (looped.shift.min(0.0) - reach),
        looped.mouth + left * (looped.shift.max(0.0) + reach),
    ];
    let top = |foot: Vec2| Vec3::new(foot.x, middle, foot.y);
    for foot in feet {
        let base = Vec3::new(foot.x, y(0.0), foot.y);
        mesh.beam(base, top(foot), POST, POST, PILLAR_BLUE);
        cuboid(
            mesh,
            base + Vec3::Y * 0.4,
            Vec3::new(3.0, 0.4, 5.0),
            looped.heading,
            PILLAR_BLUE,
        );
    }
    mesh.beam(top(feet[0]), top(feet[1]), 1.0, 1.0, PILLAR_BLUE);
}

/// A loop of orange track standing on a blue stand, facing along its
/// heading: a band as wide as a toy car's track, with raised edges, round
/// a circle `radius` across, its foot just over the stand.
fn toy_loop(mesh: &mut MeshBuilder, toy_loop: &ToyLoop) {
    const WIDTH: f32 = 4.0;
    const EDGE: f32 = 0.8;
    const STEPS: usize = 48;
    let ahead = crate::geometry::forward(toy_loop.heading);
    let ahead = Vec3::new(ahead.x, 0.0, ahead.y);
    let side = ahead.cross(Vec3::Y).normalize();
    let center = Vec3::new(
        toy_loop.center.x,
        y(toy_loop.base) + 3.0 + toy_loop.radius,
        toy_loop.center.y,
    );
    // The track drifts sideways as it goes round, as a loop's does.
    let at = |k: usize, across: f32, inward: f32| {
        let angle = k as f32 / STEPS as f32 * TAU;
        let out = -Vec3::Y * angle.cos() + ahead * angle.sin();
        center
            + out * (toy_loop.radius - inward)
            + side * (across + (k as f32 / STEPS as f32 - 0.5) * WIDTH * 1.4)
    };
    for k in 0..STEPS {
        let inward = (at(k, 0.0, 1.0) - at(k, 0.0, 0.0)).normalize_or_zero();
        mesh.quad(
            [
                (at(k, -WIDTH / 2.0, 0.0), Vec2::ZERO),
                (at(k + 1, -WIDTH / 2.0, 0.0), Vec2::X),
                (at(k + 1, WIDTH / 2.0, 0.0), Vec2::ONE),
                (at(k, WIDTH / 2.0, 0.0), Vec2::Y),
            ],
            inward,
            HOT_WHEELS_ORANGE,
        );
        // Its outside, darker, and its raised edges.
        mesh.quad(
            [
                (at(k, -WIDTH / 2.0, -0.3), Vec2::ZERO),
                (at(k + 1, -WIDTH / 2.0, -0.3), Vec2::X),
                (at(k + 1, WIDTH / 2.0, -0.3), Vec2::ONE),
                (at(k, WIDTH / 2.0, -0.3), Vec2::Y),
            ],
            -inward,
            HOT_WHEELS_ORANGE.mix(&Color::BLACK, 0.25),
        );
        for edge in [-WIDTH / 2.0, WIDTH / 2.0] {
            let out = side * edge.signum();
            mesh.quad(
                [
                    (at(k, edge, -0.3), Vec2::ZERO),
                    (at(k + 1, edge, -0.3), Vec2::X),
                    (at(k + 1, edge, EDGE), Vec2::ONE),
                    (at(k, edge, EDGE), Vec2::Y),
                ],
                out,
                HOT_WHEELS_ORANGE,
            );
            mesh.quad(
                [
                    (at(k, edge, -0.3), Vec2::ZERO),
                    (at(k + 1, edge, -0.3), Vec2::X),
                    (at(k + 1, edge, EDGE), Vec2::ONE),
                    (at(k, edge, EDGE), Vec2::Y),
                ],
                -out,
                HOT_WHEELS_ORANGE,
            );
        }
    }
    // The stand: a blue base and two posts up to the loop's foot.
    let foot = Vec3::new(toy_loop.center.x, y(toy_loop.base), toy_loop.center.y);
    cuboid(
        mesh,
        foot + Vec3::Y * 0.8,
        Vec3::new(5.0, 0.8, 7.0),
        toy_loop.heading,
        PILLAR_BLUE,
    );
    for s in [-1.0, 1.0] {
        mesh.beam(
            foot + side * s * 2.5 + Vec3::Y * 1.6,
            foot + side * s * 2.5 + Vec3::Y * 3.2,
            0.8,
            0.8,
            PILLAR_BLUE,
        );
    }
}

/// A mat on the floor: foam tiles, or a rug of one colour, a little thick.
fn mat(solid: &mut MeshBuilder, foam: &mut MeshBuilder, mat: &Mat) {
    const THICK: f32 = 0.12;
    const FOAM_TILE: f32 = 14.4;
    let half = mat.size / 2.0;
    let (x, z) = axes(mat.heading);
    let corner = |sx: f32, sz: f32, up: f32| {
        let ground = mat.center + x * sx * half.x + z * sz * half.y;
        Vec3::new(ground.x, y(up), ground.y)
    };
    let (mesh, color): (&mut MeshBuilder, Color) = match mat.kind {
        MatKind::Foam => (foam, Color::WHITE),
        MatKind::Rug(color) => (solid, color),
    };
    let uv = |sx: f32, sz: f32| Vec2::new(sx * half.x, sz * half.y) / FOAM_TILE;
    mesh.quad(
        [
            (corner(-1.0, -1.0, THICK), uv(-1.0, -1.0)),
            (corner(1.0, -1.0, THICK), uv(1.0, -1.0)),
            (corner(1.0, 1.0, THICK), uv(1.0, 1.0)),
            (corner(-1.0, 1.0, THICK), uv(-1.0, 1.0)),
        ],
        Vec3::Y,
        color,
    );
    for (a, b) in [
        ((-1.0, -1.0), (1.0, -1.0)),
        ((1.0, -1.0), (1.0, 1.0)),
        ((1.0, 1.0), (-1.0, 1.0)),
        ((-1.0, 1.0), (-1.0, -1.0)),
    ] {
        let out = (corner(a.0 + b.0, a.1 + b.1, 0.0) - corner(0.0, 0.0, 0.0)).normalize_or_zero();
        mesh.quad(
            [
                (corner(a.0, a.1, -0.1), uv(a.0, a.1)),
                (corner(b.0, b.1, -0.1), uv(b.0, b.1)),
                (corner(b.0, b.1, THICK), uv(b.0, b.1)),
                (corner(a.0, a.1, THICK), uv(a.0, a.1)),
            ],
            out,
            color.mix(&Color::BLACK, 0.1),
        );
    }
}

/// A toy standing about (see [`ToyKind`]), as big as its `scale` says,
/// those built in code: Meshy's are spawned whole (see [`spawn_models`]),
/// but for the strings those hanging from the ceiling hang from, up to it
/// (`ceiling` over the floor).
fn toy(solid: &mut MeshBuilder, blocks: &mut MeshBuilder, toy: &Toy, seed: u32, ceiling: f32) {
    let s = toy.scale;
    let base = Vec3::new(toy.position.x, y(toy.base), toy.position.y);
    let turn = Quat::from_rotation_y(toy.heading);
    let at = |local: Vec3| base + turn * (local * s);
    match toy.kind {
        ToyKind::Blocks => {
            // A pyramid of blocks: three, two and one, each a letter.
            let mut n = seed * 31;
            for (row, count) in [3, 2, 1].into_iter().enumerate() {
                for k in 0..count {
                    n += 1;
                    let x = (k as f32 - (count - 1) as f32 / 2.0) * 1.05;
                    let yaw = toy.heading + 0.25 * (random01(n) - 0.5);
                    let letter = (random01(n * 3) * 16.0) as u32 % 16;
                    let column = letter as f32 / 16.0;
                    textured_box(
                        blocks,
                        at(Vec3::new(x, 0.5 + row as f32, 0.0)),
                        Vec3::splat(s / 2.0),
                        yaw,
                        |normal| {
                            let shift = if normal.y.abs() > 0.5 {
                                5.0
                            } else {
                                normal.x * 3.0 + normal.z * 7.0
                            };
                            let column = (column + shift.abs() / 16.0).fract();
                            (Vec2::new(column, 0.0), Vec2::new(1.0 / 16.0, 1.0))
                        },
                        |_| Color::WHITE,
                    );
                }
            }
        }
        ToyKind::Ball => {
            let paints = [
                Color::srgb(0.9, 0.15, 0.12),
                Color::srgb(1.0, 0.8, 0.1),
                Color::srgb(0.15, 0.45, 0.9),
                Color::srgb(0.2, 0.7, 0.3),
            ];
            ball(
                solid,
                at(Vec3::Y * 0.5),
                s / 2.0,
                Vec3::ONE,
                toy.heading,
                |n| {
                    if n.y.abs() > 0.85 {
                        return Color::WHITE;
                    }
                    let segment = ((n.z.atan2(n.x) + PI) / TAU * 8.0) as usize;
                    paints[segment % paints.len()]
                },
            );
        }
        ToyKind::Tyres => {
            // Three stacks side by side, five tyres each, the middle one a
            // tyre taller.
            let rubber = Color::srgb(0.08, 0.08, 0.09);
            for (k, x) in [-1.0, 0.0, 1.0].into_iter().enumerate() {
                let count = if k == 1 { 6 } else { 5 };
                for level in 0..count {
                    let base = at(Vec3::new(x * 1.02, level as f32 * 0.3, 0.0));
                    cylinder(solid, base, 0.5 * s, 0.28 * s, 14, |_| rubber);
                }
                cylinder(
                    solid,
                    at(Vec3::new(x * 1.02, count as f32 * 0.3 - 0.01, 0.0)),
                    0.22 * s,
                    0.02 * s,
                    10,
                    |_| Color::srgb(0.2, 0.2, 0.2),
                );
            }
        }
        ToyKind::Plane | ToyKind::Helicopter => {
            // Its strings, from where they are tied on up to the ceiling.
            for &tie in strings_of(toy.kind) {
                let knot = at(tie);
                let top = Vec3::new(knot.x, y(ceiling), knot.z);
                solid.beam(knot, top, 0.05, 0.05, Color::srgb(0.85, 0.85, 0.82));
            }
        }
        ToyKind::Bear
        | ToyKind::Duck
        | ToyKind::Robot
        | ToyKind::Top
        | ToyKind::Ship
        | ToyKind::Giraffe
        | ToyKind::Dollhouse
        | ToyKind::FireTruck
        | ToyKind::Pegasus
        | ToyKind::CrocCar
        | ToyKind::Garage
        | ToyKind::Station
        | ToyKind::TunnelMountain
        | ToyKind::RockingHorse
        | ToyKind::Nutcracker => {}
    }
}

/// Which of Meshy's toys (see `tools/import_toys.py`) a toy of `kind` is,
/// if it is one.
fn model_of(kind: ToyKind) -> Option<&'static str> {
    match kind {
        ToyKind::Bear => Some("teddy_bear"),
        ToyKind::Duck => Some("rubber_duck"),
        ToyKind::Robot => Some("tin_robot"),
        ToyKind::Top => Some("spinning_top"),
        ToyKind::Plane => Some("biplane"),
        ToyKind::Helicopter => Some("helicopter"),
        ToyKind::Ship => Some("pirate_ship"),
        ToyKind::Giraffe => Some("giraffe_plush"),
        ToyKind::Dollhouse => Some("dollhouse"),
        ToyKind::FireTruck => Some("fire_truck"),
        ToyKind::Pegasus => Some("mech_pegasus"),
        ToyKind::CrocCar => Some("croc_car"),
        ToyKind::Garage => Some("garage_playset"),
        ToyKind::Station => Some("train_station"),
        ToyKind::TunnelMountain => Some("tunnel_mountain"),
        ToyKind::RockingHorse => Some("rocking_horse"),
        ToyKind::Nutcracker => Some("nutcracker"),
        ToyKind::Blocks | ToyKind::Ball | ToyKind::Tyres => None,
    }
}

/// Where the strings a toy hanging from the ceiling hangs from are tied on,
/// in its own frame (m, as Meshy's is made, its underside on 0): on a
/// biplane's upper wing either side of it, on the mast of a helicopter's
/// rotor.
fn strings_of(kind: ToyKind) -> &'static [Vec3] {
    const BIPLANE: [Vec3; 2] = [Vec3::new(-4.3, 7.0, 4.0), Vec3::new(4.3, 7.0, 4.0)];
    const HELICOPTER: [Vec3; 1] = [Vec3::new(0.0, 6.1, 1.4)];
    match kind {
        ToyKind::Plane => &BIPLANE,
        ToyKind::Helicopter => &HELICOPTER,
        _ => &[],
    }
}

/// How big each of the blocks of a wall of toy blocks is (m).
const WALL_BLOCK: f32 = 1.9;

/// The walls of toy blocks ([`WallStyle::Blocks`]): a block at each point
/// of the lap where the wall is of them, on the road's level, turned a
/// little this way or that, each with its own letter on its faces.
///
/// [`WallStyle::Blocks`]: super::scenery::WallStyle::Blocks
fn block_walls(mesh: &mut MeshBuilder, track: &Track, scenery: &Scenery) {
    let lap = track.lap_length();
    for index in 0..track.sample_count() {
        let along = index as f32 * track.step_length();
        for side in [super::scenery::Side::Left, super::scenery::Side::Right] {
            if super::road::wall_style(scenery, lap, along, side)
                != Some(super::scenery::WallStyle::Blocks)
            {
                continue;
            }
            let lateral = side.sign() * (track.wall_at(index) + WALL_BLOCK / 2.0);
            let at = track.surface_point(index, lateral) + Vec3::Y * WALL_BLOCK / 2.0;
            let (_, heading) = track.pose_at(index);
            let n = index as u32 * 2 + u32::from(side == super::scenery::Side::Left);
            let letter = (random01(n * 13) * 16.0) as u32 % 16;
            textured_box(
                mesh,
                at,
                Vec3::splat(WALL_BLOCK / 2.0),
                heading + 0.12 * (random01(n * 17) - 0.5),
                |normal| {
                    let face = if normal.y.abs() > 0.5 { 9 } else { 0 };
                    let column = ((letter + face) % 16) as f32 / 16.0;
                    (Vec2::new(column, 0.0), Vec2::new(1.0 / 16.0, 1.0))
                },
                |_| Color::WHITE,
            );
        }
    }
}

/// How far apart a train's rails are (m), about as the wheels of Meshy's
/// trains (1.25 to 1.55 m apart), and its sleepers.
const GAUGE: f32 = 1.4;
const SLEEPER_EVERY: f32 = 1.8;

/// A train's track: its sleepers and its two rails round its curve.
fn rails(mesh: &mut MeshBuilder, train: &Train) {
    let path = closed_curve(train.track, 1.0);
    let n = path.len();
    let up = y(train.height);
    let mut since = 0.0;
    for i in 0..n {
        let (a, b) = (path[i], path[(i + 1) % n]);
        let along = (b - a).normalize_or_zero();
        let side = Vec2::new(-along.y, along.x);
        for s in [-1.0, 1.0] {
            let (p, q) = (a + side * s * GAUGE / 2.0, b + side * s * GAUGE / 2.0);
            mesh.beam(
                Vec3::new(p.x, up + RAIL_TOP - 0.15, p.y),
                Vec3::new(q.x, up + RAIL_TOP - 0.15, q.y),
                0.25,
                0.3,
                Color::srgb(0.55, 0.56, 0.6),
            );
        }
        since += a.distance(b);
        if since >= SLEEPER_EVERY {
            since = 0.0;
            let (p, q) = (a + side * GAUGE * 0.8, a - side * GAUGE * 0.8);
            mesh.beam(
                Vec3::new(p.x, up + 0.15, p.y),
                Vec3::new(q.x, up + 0.15, q.y),
                0.6,
                0.3,
                Color::srgb(0.45, 0.3, 0.18),
            );
        }
    }
}

/// A car of an electric train going round its track: the locomotive or a
/// wagon behind it, `length` long, so far `behind` its front (m), along its
/// path (the track's curve, every metre), at its speed.
#[derive(Component)]
pub struct TrainCar {
    path: std::sync::Arc<Vec<Vec2>>,
    height: f32,
    speed: f32,
    behind: f32,
    length: f32,
}

/// The gap between the cars of a train, buffer to buffer (m).
const CAR_GAP: f32 = 0.5;
/// How high the tops of a train's rails are over what it stands on (m).
const RAIL_TOP: f32 = 0.6;

/// The toys made with Meshy (see `tools/import_toys.py`).
#[derive(Resource)]
pub struct ToyModels(PropPack);

impl FromWorld for ToyModels {
    fn from_world(world: &mut World) -> Self {
        Self(PropPack::load(
            world,
            include_bytes!("../../assets/models/toys/props.bin"),
            include_bytes!("../../assets/models/toys/color.jpg"),
        ))
    }
}

/// Which of Meshy's castles a castle of `kind` is, if it is one: each its
/// gate on its front (+Z).
fn castle_model(kind: CastleKind) -> Option<&'static str> {
    match kind {
        CastleKind::Big => Some("castle_big"),
        CastleKind::Tower => Some("castle_small"),
        CastleKind::Through => None,
    }
}

/// Meshy's locomotive, and the wagons it pulls, each facing +Z.
const LOCOMOTIVE: &str = "locomotive";

fn wagon_model(wagon: Wagon) -> &'static str {
    match wagon {
        Wagon::Coach => "coach",
        Wagon::Freight => "freight_wagon",
    }
}

/// How `piece` stands to fill a box `size` across (x and z before it is
/// turned), turned `heading` (its front, +Z, where the box's +z is), on the
/// ground at `at` (x, z) `base` over the floor: as big as fits.
fn fitted(piece: &Piece, size: Vec2, heading: f32, at: Vec2, base: f32) -> Transform {
    let made = piece.high - piece.low;
    let scale = (size.x / made.x).min(size.y / made.z);
    Transform::from_xyz(at.x, y(base), at.y)
        .with_rotation(Quat::from_rotation_y(heading))
        .with_scale(Vec3::splat(scale))
}

/// The toys of a toy store Meshy made: the castles no road runs through,
/// as big as fits in them, and the toys not built in code, as big as they
/// say, each turned as it says on what it stands on, or, hanging from the
/// ceiling, its underside as high as it says (its strings are with the rest
/// of the store, see [`toy`]).
pub fn spawn_models(world: &mut ChildSpawnerCommands, store: &ToyStore, assets: &mut Assets3d) {
    let pack = &assets.toys.0;
    let mut spawn = |piece: &Piece, transform: Transform| {
        world.spawn((
            Mesh3d(piece.mesh.clone()),
            MeshMaterial3d(pack.material.clone()),
            transform,
        ));
    };
    for castle in store.castles {
        let Some(name) = castle_model(castle.kind) else {
            continue;
        };
        let piece = pack.piece(name);
        spawn(
            piece,
            fitted(
                piece,
                castle.size,
                castle.heading,
                castle.center,
                castle.base,
            ),
        );
    }
    for toy in store.toys {
        let Some(name) = model_of(toy.kind) else {
            continue;
        };
        spawn(
            pack.piece(name),
            Transform::from_xyz(toy.position.x, y(toy.base), toy.position.y)
                .with_rotation(Quat::from_rotation_y(toy.heading))
                .with_scale(Vec3::splat(toy.scale)),
        );
    }
}

/// The electric trains, each Meshy's locomotive and its wagons, put on
/// their tracks to go round them.
pub fn spawn_trains(world: &mut ChildSpawnerCommands, store: &ToyStore, assets: &mut Assets3d) {
    let pack = &assets.toys.0;
    for train in store.trains {
        let path = std::sync::Arc::new(closed_curve(train.track, 1.0));
        let mut behind = 0.0;
        for car in 0..=train.wagons {
            let piece = pack.piece(if car == 0 {
                LOCOMOTIVE
            } else {
                wagon_model(train.wagon)
            });
            let length = piece.high.z - piece.low.z;
            world.spawn((
                Mesh3d(piece.mesh.clone()),
                MeshMaterial3d(pack.material.clone()),
                Transform::default(),
                TrainCar {
                    path: path.clone(),
                    height: train.height,
                    speed: train.speed,
                    behind,
                    length,
                },
            ));
            behind += length + CAR_GAP;
        }
    }
}

/// The trains go round their tracks: each car at its place on its curve at
/// the time, facing along it.
pub fn run_trains(time: Res<Time>, mut cars: Query<(&mut Transform, &TrainCar)>) {
    let elapsed = time.elapsed_secs();
    for (mut transform, car) in &mut cars {
        *transform = car_pose(car, elapsed);
    }
}

/// Where a car of a train is at `elapsed` seconds, and which way it faces:
/// between the points of its curve its wheels are on, half its length
/// either side of its middle, on its rails, its front (+Z) ahead.
fn car_pose(car: &TrainCar, elapsed: f32) -> Transform {
    let n = car.path.len();
    let at = |s: f32| {
        let s = s.rem_euclid(n as f32);
        let i = s.floor() as usize;
        car.path[i % n].lerp(car.path[(i + 1) % n], s.fract())
    };
    let half = car.length / 2.0;
    let middle = elapsed * car.speed - car.behind - half;
    let (front, back) = (at(middle + half), at(middle - half));
    let center = (front + back) / 2.0;
    // A heading turns -Z along the way; the cars face +Z.
    let heading = crate::geometry::heading_of(front - back) + PI;
    Transform::from_xyz(center.x, y(car.height) + RAIL_TOP, center.y)
        .with_rotation(Quat::from_rotation_y(heading))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;

    /// The circuits in a toy store: each one's name, its store and its
    /// track.
    fn stores() -> Vec<(&'static str, &'static ToyStore, Track)> {
        CIRCUITS
            .iter()
            .filter_map(|circuit| {
                let store = circuit.scenery.toys.as_ref()?;
                Some((circuit.name, store, Track::new(&circuit.layout)))
            })
            .collect()
    }

    /// The points of the centre lines of `track`'s roads (the lap and those
    /// off it), each with its height over the floor.
    fn road(track: &Track) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        (0..=track.side_roads().len()).flat_map(move |number| {
            let line = track.road(number);
            line.centerline()
                .iter()
                .zip(line.elevations())
                .map(|(&point, &elevation)| (point, elevation - BASE_ELEVATION))
        })
    }

    /// How far `point` (x, z) is from the nearest point of the centre line
    /// between `low` and `high` over the floor.
    fn road_near(track: &Track, point: Vec2, low: f32, high: f32) -> f32 {
        road(track)
            .filter(|&(_, height)| (low..=high).contains(&height))
            .map(|(at, _)| at.distance(point))
            .fold(f32::INFINITY, f32::min)
    }

    #[test]
    fn there_is_a_toy_store() {
        assert!(!stores().is_empty());
    }

    #[test]
    fn the_road_lies_on_the_furniture_or_well_clear_of_it() {
        // On a piece of furniture the road lies on its top, its inside
        // (where it banks, its lower wall) a little over it, more by no
        // more than it is raised for its banking, on its risers; over it
        // or under it, it leaves room for a bike and its rider.
        for (name, store, track) in stores() {
            for number in 0..=track.side_roads().len() {
                let line = track.road(number);
                for index in 0..line.sample_count() {
                    let (point, _) = line.pose_at(index);
                    let wall = line.wall_at(index);
                    let low = line
                        .surface_point(index, wall)
                        .y
                        .min(line.surface_point(index, -wall).y)
                        - BASE_ELEVATION;
                    for furniture in store.furniture.iter().filter(|f| f.covers(point, 1.0)) {
                        let over = low - furniture.top;
                        assert!(
                            (0.2..=1.5 + line.lift(index)).contains(&over) || over.abs() > 8.0,
                            "{name}: the road at {point} is {over} m over the top of the furniture at {}",
                            furniture.center
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_way_through_the_castle_is_as_long_and_as_fast_as_the_lap_beside_it() {
        // The two ways are mirror images: as long, and their tightest bends
        // as wide.
        for (name, _, track) in stores() {
            let n = track.sample_count();
            for fork in track.forks() {
                let road = track.road(fork.road);
                let lap = ((fork.join + n - fork.fork) % n) as f32 * track.step_length();
                assert!(
                    (road.length() - lap).abs() < 2.0,
                    "{name}: {} m beside {lap} m",
                    road.length()
                );
                let tightest = |line: &Track, from: usize, count: usize| {
                    (from..from + count)
                        .map(|i| line.smoothed_curvature(i % line.sample_count(), 4))
                        .fold(0.0, f32::max)
                };
                let (own, beside) = (
                    tightest(road, 0, road.sample_count()),
                    tightest(&track, fork.fork, (fork.join + n - fork.fork) % n),
                );
                assert!(
                    (own - beside).abs() < 0.1 * beside,
                    "{name}: bends of {} m beside {} m",
                    1.0 / own,
                    1.0 / beside
                );
            }
        }
    }

    #[test]
    fn the_bends_on_the_furniture_bank_raised_on_their_risers() {
        // Banked well, their insides on the furniture, their outsides up
        // on risers.
        for (name, store, track) in stores() {
            let (depth, _) = deck_of(BridgeStyle::SlotTrack);
            let banked = (0..track.sample_count())
                .filter(|&index| {
                    let (point, _) = track.pose_at(index);
                    let wall = track.wall_at(index);
                    let rise = (track.surface_point(index, wall).y
                        - track.surface_point(index, -wall).y)
                        .abs();
                    store.furniture.iter().any(|f| f.covers(point, 0.0))
                        && rise > 1.5
                        && resting(&track, store, index, depth)
                })
                .count();
            assert!(banked > 100, "{name}: {banked} points");
        }
    }

    #[test]
    fn no_table_stands_on_a_road() {
        for (name, store, track) in stores() {
            for furniture in store.furniture.iter().filter(|furniture| {
                matches!(
                    furniture.kind,
                    FurnitureKind::Table | FurnitureKind::Diorama
                )
            }) {
                let legs = table_legs(furniture, &track);
                assert!(legs.len() >= 4, "{name}: a table on {} legs", legs.len());
                for leg in legs {
                    let distance = road_near(&track, leg, -1.0, furniture.top);
                    assert!(
                        distance > track.wall() + TABLE_LEG,
                        "{name}: a table's leg at {leg} is {distance} m from a road under it"
                    );
                }
            }
        }
    }

    #[test]
    fn what_stands_in_the_store_stands_on_something() {
        // On the floor, or on the furniture under it: none in the air but
        // the toys hanging from the ceiling.
        for (name, store, _) in stores() {
            for train in store.trains {
                for &point in train.track {
                    assert_eq!(
                        ground_under(store, point, train.height),
                        train.height,
                        "{name}: a train's track at {point}"
                    );
                }
            }
            for castle in store.castles {
                assert_eq!(
                    ground_under(store, castle.center, castle.base),
                    castle.base,
                    "{name}: the castle at {}",
                    castle.center
                );
            }
            for toy_loop in store.loops {
                assert_eq!(
                    ground_under(store, toy_loop.center, toy_loop.base),
                    toy_loop.base,
                    "{name}: the loop at {}",
                    toy_loop.center
                );
            }
            for toy in store
                .toys
                .iter()
                .filter(|toy| !matches!(toy.kind, ToyKind::Plane | ToyKind::Helicopter))
            {
                assert_eq!(
                    ground_under(store, toy.position, toy.base),
                    toy.base,
                    "{name}: the toy at {}",
                    toy.position
                );
            }
            // What is built of bricks, all of it over what it stands on.
            for built in store.bricks {
                assert_eq!(
                    ground_under(store, built.position, built.base),
                    built.base,
                    "{name}: the bricks at {}",
                    built.position
                );
                let (low, high) = bricks_bounds(built);
                for corner in [
                    low,
                    high,
                    Vec3::new(low.x, 0.0, high.z),
                    Vec3::new(high.x, 0.0, low.z),
                ] {
                    let corner = Vec2::new(corner.x, corner.z);
                    assert_eq!(
                        ground_under(store, corner, built.base),
                        built.base,
                        "{name}: the bricks at {} reach off what they stand on at {corner}",
                        built.position
                    );
                }
            }
        }
    }

    #[test]
    fn nothing_in_the_store_stands_in_the_way_of_the_roads() {
        for (name, store, track) in stores() {
            // Whether nothing of the road's from `low` to `high` over the
            // floor comes within `reach` of `point` (x, z), its walls'
            // width aside.
            let clear = |point: Vec2, reach: f32, low: f32, high: f32| {
                road_near(&track, point, low, high) > track.wall() + reach
            };
            for train in store.trains {
                for point in closed_curve(train.track, 2.0) {
                    assert!(
                        clear(point, 3.0, train.height - 3.0, train.height + 6.0),
                        "{name}: a train's track at {point}"
                    );
                }
            }
            for castle in store
                .castles
                .iter()
                .filter(|castle| castle.kind != CastleKind::Through)
            {
                assert!(
                    clear(
                        castle.center,
                        castle.size.length() / 2.0,
                        castle.base - 3.0,
                        castle.base + 20.0
                    ),
                    "{name}: the castle at {}",
                    castle.center
                );
            }
            for tower in store.towers {
                assert!(
                    clear(tower.center, tower.radius, -1.0, tower.height),
                    "{name}: the tower at {}",
                    tower.center
                );
            }
            for toy_loop in store.loops {
                let top = toy_loop.base + 2.0 * toy_loop.radius;
                assert!(
                    clear(
                        toy_loop.center,
                        toy_loop.radius,
                        toy_loop.base - 3.0,
                        top + 3.0
                    ),
                    "{name}: the loop at {}",
                    toy_loop.center
                );
            }
            for built in store.bricks {
                let (low, high) = bricks_bounds(built);
                let reach = [
                    low,
                    high,
                    Vec3::new(low.x, 0.0, high.z),
                    Vec3::new(high.x, 0.0, low.z),
                ]
                .map(|corner| Vec2::new(corner.x, corner.z).distance(built.position))
                .into_iter()
                .fold(0.0, f32::max);
                let tall = high.y - y(built.base);
                let (low, high) = (built.base - 3.0, built.base + tall + 3.0);
                assert!(
                    clear(built.position, reach, low, high),
                    "{name}: the bricks at {} reach {reach} m, the road {} m off",
                    built.position,
                    road_near(&track, built.position, low, high)
                );
            }
            for toy in store.toys {
                // How far it reaches round its middle and how tall it is:
                // Meshy's as it was made (centred), the others about a
                // metre.
                let (reach, tall) = match model_of(toy.kind) {
                    Some(model) => {
                        let (_, high) = model_bounds(model);
                        (Vec2::new(high.x, high.z).length(), high.y)
                    }
                    None => (1.6, 1.5),
                };
                // Those hanging from the ceiling, from their strings down.
                let (low, high) = match toy.kind {
                    ToyKind::Plane | ToyKind::Helicopter => (toy.base - 3.0, store.ceiling),
                    _ => (toy.base - 3.0, toy.base + tall * toy.scale + 3.0),
                };
                assert!(
                    clear(toy.position, reach * toy.scale, low, high),
                    "{name}: the toy at {}",
                    toy.position
                );
            }
        }
    }

    #[test]
    fn a_road_runs_through_the_castle_built_round_it() {
        let mut castles = 0;
        for (name, store, track) in stores() {
            for castle in store
                .castles
                .iter()
                .filter(|castle| castle.kind == CastleKind::Through)
            {
                // A road along it, through its middle, on what it stands on,
                // its walls well inside the castle's.
                let nearest = track.nearest_at(castle.center, y(castle.base) + 1.0);
                assert!(nearest.distance < 1.0, "{name}: {}", nearest.distance);
                let (_, heading) = track.pose_beside(&nearest, 0.0);
                assert!(
                    (heading - castle.heading).sin().abs() < 0.1,
                    "{name}: {heading}"
                );
                let wall = track.road(nearest.road).wall_at(nearest.index);
                assert!(castle.size.x / 2.0 > wall + 4.0, "{name}");
                let over = track.height_beside(&nearest) - y(castle.base);
                assert!((0.2..1.5).contains(&over), "{name}: {over} m over its base");
                castles += 1;
            }
        }
        assert!(castles > 0);
    }

    /// The corners of the box round what is built of bricks at `built`.
    fn bricks_bounds(built: &crate::world::scenery::Bricks) -> (Vec3, Vec3) {
        let mut mesh = MeshBuilder::default();
        super::super::bricks::build(&mut mesh, built, 0);
        super::super::japan::bounds(&mesh.build())
    }

    /// The pieces of Meshy's toys in their pack.
    fn pack() -> Vec<crate::bike::model::imported::PackedMesh> {
        crate::bike::model::imported::read(include_bytes!("../../assets/models/toys/props.bin"))
            .meshes
    }

    /// The corners of the box round Meshy's toy `name` as it was made.
    fn model_bounds(name: &str) -> (Vec3, Vec3) {
        let pieces = pack();
        let piece = pieces
            .iter()
            .find(|piece| piece.name == name)
            .unwrap_or_else(|| panic!("no {name} in the pack"));
        super::super::japan::bounds(&piece.mesh)
    }

    #[test]
    fn every_toy_meshy_made_is_in_the_pack_on_the_ground_and_light() {
        let pieces = pack();
        // Every one the stores place.
        for (name, store, _) in stores() {
            let castles = store.castles.iter().filter_map(|c| castle_model(c.kind));
            let toys = store.toys.iter().filter_map(|toy| model_of(toy.kind));
            let trains = store
                .trains
                .iter()
                .flat_map(|train| [LOCOMOTIVE, wagon_model(train.wagon)]);
            for model in castles.chain(toys).chain(trains) {
                assert!(
                    pieces.iter().any(|piece| piece.name == model),
                    "{name}: no {model} in the pack"
                );
            }
        }
        for piece in &pieces {
            let name = &piece.name;
            let (low, high) = super::super::japan::bounds(&piece.mesh);
            assert!(low.y.abs() < 1e-3, "{name} stands on the ground");
            assert!((low.x + high.x).abs() < 0.05 * (high.x - low.x), "{name}");
            assert!((low.z + high.z).abs() < 0.05 * (high.z - low.z), "{name}");
            let triangles = piece.mesh.indices().unwrap().len() / 3;
            assert!(triangles < 8_000, "{name}: {triangles} triangles");
        }
    }

    #[test]
    fn the_hanging_toys_strings_are_tied_on_to_them() {
        // Each knot on the toy's top, within its box.
        for kind in [ToyKind::Plane, ToyKind::Helicopter] {
            let model = model_of(kind).unwrap();
            let (low, high) = model_bounds(model);
            assert!(!strings_of(kind).is_empty(), "{model}");
            for knot in strings_of(kind) {
                assert!(
                    (low.x..=high.x).contains(&knot.x) && (low.z..=high.z).contains(&knot.z),
                    "{model}: {knot}"
                );
                assert!(high.y - knot.y < 0.5, "{model}: {knot}");
            }
        }
    }

    #[test]
    fn the_trains_run_clear_of_the_toys_but_through_a_tunnel() {
        // No point of a train's track within a train's half width of the
        // box round a toy standing on what the train runs on, or round
        // something built of bricks; but through a tunnel mountain, along
        // the middle of its tunnel.
        const HALF_TRAIN: f32 = 1.3;
        let mut tunnels = 0;
        for (name, store, _) in stores() {
            for train in store.trains {
                let path = closed_curve(train.track, 1.0);
                let n = path.len();
                for toy in store.toys.iter().filter(|toy| toy.base == train.height) {
                    let Some(model) = model_of(toy.kind) else {
                        continue;
                    };
                    let (low, high) = model_bounds(model);
                    let turn = Quat::from_rotation_y(toy.heading).inverse();
                    let mut through = 0;
                    for (k, &point) in path.iter().enumerate() {
                        let offset = point - toy.position;
                        let local = turn * Vec3::new(offset.x, 0.0, offset.y) / toy.scale;
                        let margin = HALF_TRAIN / toy.scale;
                        let inside = (low.x - margin..=high.x + margin).contains(&local.x)
                            && (low.z - margin..=high.z + margin).contains(&local.z);
                        if !inside {
                            continue;
                        }
                        assert_eq!(
                            toy.kind,
                            ToyKind::TunnelMountain,
                            "{name}: a train's track at {point} through the toy at {}",
                            toy.position
                        );
                        // On the middle of its tunnel, and along it from
                        // one mouth to the other.
                        assert!(
                            local.x.abs() * toy.scale < 0.3,
                            "{name}: {point} off the tunnel"
                        );
                        if (low.z..=high.z).contains(&local.z) {
                            let along = (path[(k + 1) % n] - path[(k + n - 1) % n]).normalize();
                            let along = turn * Vec3::new(along.x, 0.0, along.y);
                            assert!(along.x.abs() < 0.03, "{name}: {point} across the tunnel");
                            through += 1;
                        }
                    }
                    if toy.kind == ToyKind::TunnelMountain {
                        assert!(through as f32 >= high.z - low.z - 1.0, "{name}: {through}");
                        tunnels += 1;
                    }
                }
                for built in store
                    .bricks
                    .iter()
                    .filter(|built| built.base == train.height)
                {
                    let (low, high) = bricks_bounds(built);
                    for &point in &path {
                        let inside = (low.x - HALF_TRAIN..=high.x + HALF_TRAIN).contains(&point.x)
                            && (low.z - HALF_TRAIN..=high.z + HALF_TRAIN).contains(&point.y);
                        assert!(!inside, "{name}: a train's track at {point} through bricks");
                    }
                }
            }
        }
        assert!(tunnels > 0);
    }

    #[test]
    fn the_trains_wheels_run_on_their_rails() {
        // The wheels of each car (most of its vertices near the ground)
        // either side of its middle, about as far apart as the rails.
        for model in [LOCOMOTIVE, "coach", "freight_wagon"] {
            let pieces = pack();
            let piece = pieces.iter().find(|piece| piece.name == model).unwrap();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
                piece.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("{model}: no positions");
            };
            let mut wheels: Vec<f32> = positions
                .iter()
                .filter(|p| p[1] < 0.25)
                .map(|p| p[0].abs())
                .collect();
            wheels.sort_by(f32::total_cmp);
            let spread = wheels[wheels.len() / 2];
            assert!(
                (spread * 2.0 - GAUGE).abs() < 0.3,
                "{model}: its wheels {} m apart",
                spread * 2.0
            );
        }
    }

    #[test]
    fn a_train_runs_round_and_round_its_track() {
        let (_, store, _) = &stores()[0];
        let train = &store.trains[0];
        let path = std::sync::Arc::new(closed_curve(train.track, 1.0));
        let car = TrainCar {
            path: path.clone(),
            speed: train.speed,
            behind: 0.0,
            height: train.height,
            length: 7.0,
        };
        let length = path.len() as f32;
        let lap = length / train.speed;
        let start = car_pose(&car, 0.0).translation;
        assert!(car_pose(&car, lap).translation.distance(start) < 1.5);
        assert!(car_pose(&car, lap / 2.0).translation.distance(start) > 10.0);
        assert!((start.y - y(train.height)).abs() < 1.0);
    }
}
