//! A toy store's way in ([`Entrance`]): a front of glass in the south wall
//! of its room, aluminium framed, from the floor up to under the stripes
//! round its top; sliding doors in it under the box of their works and an
//! exit sign, a mat in front of them and the gates of the shop's alarm; and
//! outside, in the daylight, what is seen through it: a pavement under a
//! canopy, a car park with cars in its bays and lamp posts over them, trees
//! along its far side, a street and the buildings across it, under the sky.
//!
//! All twelve times life size, as the toys make everything else (a car is
//! 54 m long). Outside is lit by a sun of its own, worked into its colours
//! ([`sunlit`]), unlit and unfogged, so that it is brighter than the shop,
//! as daylight is through a shop's windows; a box of sky round it hides the
//! rest of the world, the room's floor running on under it all. Built with
//! the rest of the store away from the main thread (see [`pieces`]).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use super::toys::{Entrance, ToyStore, ball_of, y};
use crate::geometry::random01;
use crate::world::quads::MeshBuilder;

/// How high the glass reaches (m over the floor): the opening in the wall,
/// the frame's head in it.
pub const GLASS_TOP: f32 = 40.0;
/// How high the doors are, the bar across the glass over them; and how wide
/// the two of them are together (m).
const DOOR_HEIGHT: f32 = 30.0;
const DOORS: f32 = 24.0;
/// How far the mat and the alarm's gates inside the doors reach (m): either
/// side of their middle, and in from the wall.
pub const FITTINGS: (f32, f32) = (20.0, 24.0);
/// How wide a pane of glass is at most (m), between two mullions.
const PANE: f32 = 18.0;
/// How wide and deep the frame's bars are (m), and their colour: anodised
/// aluminium.
const BAR: f32 = 1.2;
const BAR_DEPTH: f32 = 1.6;
const ALUMINIUM: Color = Color::srgb(0.32, 0.33, 0.35);

/// Outside: how high its ground is over the room's floor (m), the paint on
/// it and the shade of what stands on it; and how high the pavement.
const GROUND: f32 = 0.05;
const ASPHALT: f32 = 0.12;
const PAINT: f32 = 0.18;
const SHADE: f32 = 0.24;
const KERB: f32 = 1.2;

/// How far out from the wall (m) each strip of the outside reaches: the
/// pavement, the lane along it, the car park's four rows of bays (two back
/// to back, an aisle, two more), a verge of grass, the street, the far
/// pavement and the buildings over it; and the box of sky round it all.
const PAVEMENT: f32 = 48.0;
const LANE: f32 = 120.0;
const ROWS: [f32; 4] = [120.0, 180.0, 312.0, 372.0];
const ROW: f32 = 60.0;
const VERGE: (f32, f32) = (432.0, 492.0);
const STREET: (f32, f32) = (492.0, 600.0);
const FAR_PAVEMENT: f32 = 640.0;
const SKY: f32 = 1150.0;
/// How far either side of the doors (m) the car park reaches, and the box
/// of sky; how wide a bay is; and how high the sky.
const PARK_HALF: f32 = 600.0;
const SKY_HALF: f32 = 1300.0;
const BAY: f32 = 30.0;
const SKY_HEIGHT: f32 = 1600.0;

/// Toward the sun outside, and its light and the sky's (linear), worked
/// into the colours of what is out there (see [`sunlit`]): high behind the
/// shop, to its right as seen from it, so that what faces the shop is lit.
const SUN: Vec3 = Vec3::new(0.35, 0.78, -0.52);
const SUNLIGHT: Vec3 = Vec3::new(0.7, 0.66, 0.58);
const SKYLIGHT: Vec3 = Vec3::new(0.68, 0.72, 0.8);

/// `color` lit by the sun outside on a face toward `normal`.
fn sunlit(color: Color, normal: Vec3) -> Color {
    let direct = normal.normalize_or_zero().dot(SUN.normalize()).max(0.0);
    let light = SKYLIGHT + SUNLIGHT * direct;
    let linear = color.to_linear();
    Color::LinearRgba(LinearRgba::rgb(
        linear.red * light.x,
        linear.green * light.y,
        linear.blue * light.z,
    ))
}

/// Where the shade of something `height` up falls on the ground, from
/// under it.
fn shade_offset(height: f32) -> Vec2 {
    -Vec2::new(SUN.x, SUN.z) / SUN.y * height
}

/// The meshes of a shop's way in, if it has one: its frame, its glass, its
/// fittings (the mat, the alarm's gates, the door works), its exit sign,
/// and the outside, lit by its own sun (see [`sunlit`]).
pub fn pieces(store: &ToyStore) -> Vec<(Mesh, StandardMaterial)> {
    let Some(entrance) = &store.entrance else {
        return Vec::new();
    };
    let wall = store.room.1.y;
    let mut frame = MeshBuilder::default();
    let mut glass = MeshBuilder::default();
    let mut fittings = MeshBuilder::default();
    let mut sign = MeshBuilder::default();
    let mut outside = MeshBuilder::default();
    shopfront(&mut frame, &mut glass, entrance, wall);
    inside(&mut fittings, &mut sign, entrance, wall);
    car_park(&mut outside, entrance, wall);
    sky(&mut outside, entrance.doors, wall);
    [
        (
            frame,
            StandardMaterial {
                metallic: 0.6,
                perceptual_roughness: 0.35,
                ..default()
            },
        ),
        (
            glass,
            StandardMaterial {
                base_color: Color::srgba(0.85, 0.94, 0.96, 0.1),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.06,
                reflectance: 0.6,
                ..default()
            },
        ),
        (
            fittings,
            StandardMaterial {
                perceptual_roughness: 0.6,
                ..default()
            },
        ),
        (
            sign,
            StandardMaterial {
                emissive: LinearRgba::rgb(0.15, 2.4, 0.7),
                ..default()
            },
        ),
        (
            outside,
            // Lit by its own sun, in its colours: unlit, so the shop's
            // lamps do not light it (nor its shadows fall in the shop),
            // and clear of the shop's haze.
            StandardMaterial {
                unlit: true,
                fog_enabled: false,
                ..default()
            },
        ),
    ]
    .into_iter()
    .filter(|(mesh, _)| !mesh.is_empty())
    .map(|(mesh, material)| (mesh.build(), material))
    .collect()
}

/// A box from corner `min` to corner `max`, each face of `color` as it is
/// toward it (`paint`: the face's outward direction to its colour).
fn slab(mesh: &mut MeshBuilder, min: Vec3, max: Vec3, paint: impl Fn(Vec3) -> Color) {
    let center = (min + max) / 2.0;
    let half = (max - min) / 2.0;
    for normal in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
        let (u, v) = if normal.y.abs() > 0.5 {
            (Vec3::X, Vec3::Z)
        } else {
            (normal.cross(Vec3::Y), Vec3::Y)
        };
        let at = |a: f32, b: f32| {
            (
                center + half * (normal + u * a + v * b),
                Vec2::new(a, b) * 0.5 + Vec2::splat(0.5),
            )
        };
        mesh.quad(
            [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)],
            normal,
            paint(normal),
        );
    }
}

/// A box as [`slab`], all of `color`, lit as indoors (by the material).
fn lit_slab(mesh: &mut MeshBuilder, min: Vec3, max: Vec3, color: Color) {
    slab(mesh, min, max, |_| color);
}

/// A box as [`slab`], all of `color`, in the sun outside.
fn block(mesh: &mut MeshBuilder, min: Vec3, max: Vec3, color: Color) {
    slab(mesh, min, max, |normal| sunlit(color, normal));
}

/// A flat stretch of ground from `a` to `b` (x, z), `height` over the
/// floor, in the sun.
fn flat(mesh: &mut MeshBuilder, a: Vec2, b: Vec2, height: f32, color: Color) {
    let at = |x: f32, z: f32| (Vec3::new(x, y(height), z), Vec2::ZERO);
    mesh.quad(
        [at(a.x, a.y), at(b.x, a.y), at(b.x, b.y), at(a.x, b.y)],
        Vec3::Y,
        sunlit(color, Vec3::Y),
    );
}

/// A prism of `sides` round from `a` to `b`, `radius` round, its ends
/// closed, in the sun.
fn prism(mesh: &mut MeshBuilder, a: Vec3, b: Vec3, radius: f32, sides: usize, color: Color) {
    let along = (b - a).normalize_or_zero();
    let side = if along.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let u = along.cross(side).normalize();
    let v = along.cross(u);
    let around = |k: usize| {
        let angle = k as f32 / sides as f32 * TAU;
        u * angle.cos() + v * angle.sin()
    };
    for k in 0..sides {
        let (p, q) = (around(k), around(k + 1));
        let out = (p + q).normalize();
        mesh.quad(
            [
                (a + p * radius, Vec2::ZERO),
                (a + q * radius, Vec2::X),
                (b + q * radius, Vec2::ONE),
                (b + p * radius, Vec2::Y),
            ],
            out,
            sunlit(color, out),
        );
        for (end, facing) in [(a, -along), (b, along)] {
            mesh.triangle(
                [
                    (end, Vec2::ZERO),
                    (end + p * radius, Vec2::X),
                    (end + q * radius, Vec2::Y),
                ],
                facing,
                sunlit(color, facing),
            );
        }
    }
}

/// A flat round patch of shade on the ground round `center` (x, z),
/// `radius` round, squashed to `scale` along x and z.
fn shade(mesh: &mut MeshBuilder, center: Vec2, radius: f32, scale: Vec2, color: Color) {
    const SIDES: usize = 10;
    let middle = Vec3::new(center.x, y(SHADE), center.y);
    let at = |k: usize| {
        let angle = k as f32 / SIDES as f32 * TAU;
        middle + Vec3::new(angle.cos() * scale.x, 0.0, angle.sin() * scale.y) * radius
    };
    let dark = sunlit(color, Vec3::Y).mix(&Color::BLACK, 0.5);
    for k in 0..SIDES {
        mesh.triangle(
            [(middle, Vec2::ZERO), (at(k), Vec2::X), (at(k + 1), Vec2::Y)],
            Vec3::Y,
            dark,
        );
    }
}

/// The front of glass: the wall's opening framed (a sill, a head, a jamb
/// either side, mullions between panes, a bar across at the doors' head),
/// the doors in its middle (two leaves, closed, framed thicker), and the
/// glass of every pane and leaf.
fn shopfront(frame: &mut MeshBuilder, glass: &mut MeshBuilder, entrance: &Entrance, wall: f32) {
    let (from, to) = (entrance.from, entrance.to);
    let (left, right) = (entrance.doors - DOORS / 2.0, entrance.doors + DOORS / 2.0);
    let depth = |x0: f32, x1: f32, y0: f32, y1: f32| {
        (
            Vec3::new(x0, y(y0), wall - BAR_DEPTH / 2.0),
            Vec3::new(x1, y(y1), wall + BAR_DEPTH / 2.0),
        )
    };
    let mut bar = |x0: f32, x1: f32, y0: f32, y1: f32| {
        let (min, max) = depth(x0, x1, y0, y1);
        lit_slab(frame, min, max, ALUMINIUM);
    };
    // The sill, the head (a little into the wall over it), the jambs, the
    // bar across.
    bar(from - BAR, to + BAR, 0.0, 1.4);
    bar(from - BAR, to + BAR, GLASS_TOP - BAR, GLASS_TOP + 0.6);
    bar(from - BAR, from, 0.0, GLASS_TOP + 0.6);
    bar(to, to + BAR, 0.0, GLASS_TOP + 0.6);
    bar(from, to, DOOR_HEIGHT, DOOR_HEIGHT + BAR);
    // The mullions: either side of the doors, and between the panes of the
    // glass either side of them, as wide each; over the doors, the fanlight
    // in two.
    let mut mullions = vec![left - BAR / 2.0, right + BAR / 2.0];
    for (a, b) in [(from, left - BAR), (right + BAR, to)] {
        let panes = ((b - a) / PANE).ceil().max(1.0) as u32;
        for k in 1..panes {
            mullions.push(a + (b - a) * k as f32 / panes as f32);
        }
    }
    for &x in &mullions {
        bar(x - BAR / 2.0, x + BAR / 2.0, 1.4, GLASS_TOP - BAR);
    }
    bar(
        entrance.doors - BAR / 2.0,
        entrance.doors + BAR / 2.0,
        DOOR_HEIGHT + BAR,
        GLASS_TOP - BAR,
    );
    // The glass, a pane across the front behind the doors' leaves: their
    // own in front of it, inside, as on their track.
    let pane = |glass: &mut MeshBuilder, x0: f32, x1: f32, y0: f32, y1: f32, z: f32| {
        let at = |x: f32, h: f32| (Vec3::new(x, y(h), z), Vec2::ZERO);
        glass.quad(
            [at(x0, y0), at(x1, y0), at(x1, y1), at(x0, y1)],
            -Vec3::Z,
            Color::WHITE,
        );
    };
    pane(glass, from, left - BAR, 1.4, GLASS_TOP - BAR, wall);
    pane(glass, right + BAR, to, 1.4, GLASS_TOP - BAR, wall);
    pane(
        glass,
        left - BAR,
        right + BAR,
        DOOR_HEIGHT + BAR,
        GLASS_TOP - BAR,
        wall,
    );
    let track = wall - BAR_DEPTH / 2.0 - 0.4;
    for (x0, x1) in [(left, entrance.doors), (entrance.doors, right)] {
        // The leaf's stiles and rails, and its glass.
        let mut leaf = |a: f32, b: f32, h0: f32, h1: f32| {
            lit_slab(
                frame,
                Vec3::new(a, y(h0), track - 0.35),
                Vec3::new(b, y(h1), track + 0.35),
                ALUMINIUM,
            );
        };
        leaf(x0, x0 + 1.0, 0.2, DOOR_HEIGHT);
        leaf(x1 - 1.0, x1, 0.2, DOOR_HEIGHT);
        leaf(x0, x1, 0.2, 2.2);
        leaf(x0, x1, DOOR_HEIGHT - 1.2, DOOR_HEIGHT);
        pane(glass, x0 + 1.0, x1 - 1.0, 2.2, DOOR_HEIGHT - 1.2, track);
    }
}

/// Inside the doors: the box of their works over them, its sensor and the
/// exit sign over it; a mat in front of them; and the gates of the shop's
/// alarm, either side and between, as tall as a child.
fn inside(fittings: &mut MeshBuilder, sign: &mut MeshBuilder, entrance: &Entrance, wall: f32) {
    let doors = entrance.doors;
    let track = wall - BAR_DEPTH / 2.0 - 0.4;
    lit_slab(
        fittings,
        Vec3::new(doors - DOORS / 2.0 - 3.0, y(DOOR_HEIGHT + 0.4), track - 2.2),
        Vec3::new(doors + DOORS / 2.0 + 3.0, y(DOOR_HEIGHT + 3.4), track - 0.4),
        Color::srgb(0.7, 0.71, 0.73),
    );
    lit_slab(
        fittings,
        Vec3::new(doors - 1.6, y(DOOR_HEIGHT - 0.4), track - 2.0),
        Vec3::new(doors + 1.6, y(DOOR_HEIGHT + 0.4), track - 0.8),
        Color::srgb(0.08, 0.08, 0.1),
    );
    // The exit sign: green, alight, a white door and arrow on it, on an
    // arm off the frame's head.
    let face = track - 2.6;
    lit_slab(
        sign,
        Vec3::new(doors - 4.5, y(34.5), face),
        Vec3::new(doors + 4.5, y(38.5), face + 0.5),
        Color::srgb(0.05, 0.55, 0.22),
    );
    lit_slab(
        fittings,
        Vec3::new(doors - 0.3, y(38.5), face + 0.1),
        Vec3::new(doors + 0.3, y(GLASS_TOP - 0.4), wall - BAR_DEPTH / 2.0),
        ALUMINIUM,
    );
    for (x0, x1, y0, y1) in [
        (doors - 3.6, doors - 1.8, 35.0, 38.0),
        (doors - 0.6, doors + 2.4, 36.3, 36.7),
        (doors + 2.4, doors + 3.0, 35.9, 37.1),
    ] {
        lit_slab(
            fittings,
            Vec3::new(x0, y(y0), face - 0.06),
            Vec3::new(x1, y(y1), face),
            Color::WHITE,
        );
    }
    // The mat, dark, with a lighter border.
    let mat = (
        doors - FITTINGS.0,
        doors + FITTINGS.0,
        wall - FITTINGS.1,
        track - 0.6,
    );
    lit_slab(
        fittings,
        Vec3::new(mat.0, y(-0.03), mat.2),
        Vec3::new(mat.1, y(0.1), mat.3),
        Color::srgb(0.32, 0.32, 0.34),
    );
    lit_slab(
        fittings,
        Vec3::new(mat.0 + 1.0, y(-0.03), mat.2 + 1.0),
        Vec3::new(mat.1 - 1.0, y(0.14), mat.3 - 1.0),
        Color::srgb(0.14, 0.14, 0.16),
    );
    // The alarm's gates: a pale panel on a grey foot, edged in grey.
    for x in [doors - DOORS / 2.0 - 1.5, doors, doors + DOORS / 2.0 + 1.5] {
        let (z0, z1) = (wall - 19.0, wall - 9.0);
        lit_slab(
            fittings,
            Vec3::new(x - 0.6, y(0.0), z0),
            Vec3::new(x + 0.6, y(1.6), z1),
            Color::srgb(0.4, 0.41, 0.44),
        );
        lit_slab(
            fittings,
            Vec3::new(x - 0.3, y(1.6), z0 + 0.6),
            Vec3::new(x + 0.3, y(17.0), z1 - 0.6),
            Color::srgb(0.9, 0.91, 0.93),
        );
        for z in [z0 + 0.3, z1 - 0.3] {
            lit_slab(
                fittings,
                Vec3::new(x - 0.4, y(1.6), z - 0.3),
                Vec3::new(x + 0.4, y(17.6), z + 0.3),
                Color::srgb(0.55, 0.56, 0.6),
            );
        }
    }
}

/// What the car park's paint is: white, the kerb's yellow, the blue of the
/// bays kept for wheelchairs.
const LINE_WHITE: Color = Color::srgb(0.92, 0.92, 0.9);
const LINE_YELLOW: Color = Color::srgb(0.95, 0.78, 0.15);
const BAY_BLUE: Color = Color::srgb(0.18, 0.38, 0.75);
const TARMAC: Color = Color::srgb(0.3, 0.31, 0.33);
const STREET_TARMAC: Color = Color::srgb(0.24, 0.25, 0.27);
const CONCRETE: Color = Color::srgb(0.74, 0.73, 0.7);
const GRASS: Color = Color::srgb(0.36, 0.56, 0.24);

/// The outside: the ground (grass, the pavement under the canopy, the car
/// park and its paint, the verge, the street, the far pavement), the cars
/// parked and in the street, the lamp posts, the trees and the buildings.
fn car_park(mesh: &mut MeshBuilder, entrance: &Entrance, wall: f32) {
    let doors = entrance.doors;
    let out = |o: f32| wall + o;
    let (west, east) = (doors - SKY_HALF, doors + SKY_HALF);
    let (park_west, park_east) = (doors - PARK_HALF, doors + PARK_HALF);
    // Grass all over, under the rest.
    flat(
        mesh,
        Vec2::new(west, out(0.0)),
        Vec2::new(east, out(SKY)),
        GROUND,
        GRASS,
    );
    // The pavement along the shop, the length of the shop's front and
    // more, its slabs' joints; in the shade of the shop and its canopy.
    let shaded = CONCRETE.mix(&Color::BLACK, 0.3);
    slab(
        mesh,
        Vec3::new(west, y(0.0), out(0.3)),
        Vec3::new(east, y(KERB), out(PAVEMENT)),
        |normal| {
            if normal.y > 0.5 {
                sunlit(shaded, normal)
            } else {
                sunlit(Color::srgb(0.62, 0.61, 0.58), normal)
            }
        },
    );
    let joint = shaded.mix(&Color::BLACK, 0.25);
    let mut x = west;
    while x < east {
        flat(
            mesh,
            Vec2::new(x, out(0.0)),
            Vec2::new(x + 0.3, out(PAVEMENT)),
            KERB + 0.02,
            joint,
        );
        x += 12.0;
    }
    for k in 1..4 {
        let o = PAVEMENT * k as f32 / 4.0;
        flat(
            mesh,
            Vec2::new(west, out(o)),
            Vec2::new(east, out(o + 0.3)),
            KERB + 0.02,
            joint,
        );
    }
    canopy(mesh, entrance, wall);
    // The car park's tarmac, and the lane along the pavement.
    flat(
        mesh,
        Vec2::new(park_west, out(LANE)),
        Vec2::new(park_east, out(VERGE.0)),
        ASPHALT,
        TARMAC,
    );
    flat(
        mesh,
        Vec2::new(west, out(PAVEMENT)),
        Vec2::new(east, out(LANE)),
        ASPHALT,
        TARMAC,
    );
    // The kerb's yellow line; the zebra crossing from the doors.
    flat(
        mesh,
        Vec2::new(west, out(PAVEMENT + 1.0)),
        Vec2::new(east, out(PAVEMENT + 2.2)),
        PAINT,
        LINE_YELLOW,
    );
    for k in 0..5 {
        let x = doors - 27.0 + 12.0 * k as f32;
        flat(
            mesh,
            Vec2::new(x, out(PAVEMENT + 5.0)),
            Vec2::new(x + 6.0, out(LANE - 4.0)),
            PAINT,
            LINE_WHITE,
        );
    }
    // The bays: lines between them; a line down the middle of each pair of
    // rows back to back; the two in front of the doors blue; a car in most.
    let bays = (2.0 * PARK_HALF / BAY) as u32;
    for (row, &start) in ROWS.iter().enumerate() {
        for k in 0..=bays {
            let x = park_west + k as f32 * BAY;
            flat(
                mesh,
                Vec2::new(x - 0.6, out(start)),
                Vec2::new(x + 0.6, out(start + ROW)),
                PAINT,
                LINE_WHITE,
            );
        }
        if row % 2 == 1 {
            flat(
                mesh,
                Vec2::new(park_west, out(start - 0.6)),
                Vec2::new(park_east, out(start + 0.6)),
                PAINT,
                LINE_WHITE,
            );
        }
        for k in 0..bays {
            let middle = Vec2::new(park_west + (k as f32 + 0.5) * BAY, out(start + ROW / 2.0));
            if row == 0 && (middle.x - doors).abs() < BAY * 1.5 {
                flat(
                    mesh,
                    Vec2::new(middle.x - BAY / 2.0 + 0.8, out(start + 1.0)),
                    Vec2::new(middle.x + BAY / 2.0 - 0.8, out(start + ROW - 1.0)),
                    PAINT,
                    BAY_BLUE,
                );
                continue;
            }
            let n = row as u32 * 1000 + k;
            let r = |j: u32| random01(n * 11 + j);
            if r(0) > 0.68 {
                continue;
            }
            // Nose in or out, square in its bay or nearly.
            let heading = if r(1) < 0.5 { 0.0 } else { PI } + 0.06 * (r(2) - 0.5);
            let at = middle + Vec2::new(2.5 * (r(3) - 0.5), 3.0 * (r(4) - 0.5));
            car(mesh, at, heading, n);
        }
    }
    // The lamp posts down the middle of each pair of rows.
    for &start in &[ROWS[1], ROWS[3]] {
        let mut x = park_west + 60.0;
        while x < park_east {
            lamp_post(mesh, Vec2::new(x, out(start)));
            x += 120.0;
        }
    }
    // The verge, raised on its kerb, its trees; trees either side of the
    // car park.
    slab(
        mesh,
        Vec3::new(west, y(0.0), out(VERGE.0)),
        Vec3::new(east, y(0.6), out(VERGE.1)),
        |normal| {
            if normal.y > 0.5 {
                sunlit(GRASS, normal)
            } else {
                sunlit(CONCRETE, normal)
            }
        },
    );
    // Their crowns, as wide as 80 m, inside the box of sky.
    let mut x = west + 60.0;
    let mut n = 0;
    while x < east - 60.0 {
        tree(mesh, Vec2::new(x, out((VERGE.0 + VERGE.1) / 2.0)), 0.6, n);
        x += 64.0 + 20.0 * random01(n * 3 + 1);
        n += 1;
    }
    for side in [-1.0, 1.0] {
        for k in 0..14 {
            let m = 500 + k * 2 + if side > 0.0 { 1 } else { 0 };
            let across = PARK_HALF + 60.0 + (SKY_HALF - PARK_HALF - 240.0) * random01(m * 5);
            let o = LANE + 30.0 + (VERGE.0 - LANE - 60.0) * random01(m * 5 + 1);
            tree(mesh, Vec2::new(doors + side * across, out(o)), GROUND, m);
        }
    }
    // The street: its edges, the dashes down its middle, a few cars.
    flat(
        mesh,
        Vec2::new(west, out(STREET.0)),
        Vec2::new(east, out(STREET.1)),
        ASPHALT,
        STREET_TARMAC,
    );
    for o in [STREET.0 + 4.0, STREET.1 - 5.2] {
        flat(
            mesh,
            Vec2::new(west, out(o)),
            Vec2::new(east, out(o + 1.2)),
            PAINT,
            LINE_WHITE,
        );
    }
    let middle = (STREET.0 + STREET.1) / 2.0;
    let mut x = west;
    while x < east {
        flat(
            mesh,
            Vec2::new(x, out(middle - 0.6)),
            Vec2::new((x + 36.0).min(east), out(middle + 0.6)),
            PAINT,
            LINE_WHITE,
        );
        x += 72.0;
    }
    for (k, (along, lane)) in [(-380.0, 1.0), (140.0, -1.0), (520.0, 1.0), (-760.0, -1.0)]
        .into_iter()
        .enumerate()
    {
        let heading = if lane > 0.0 { FRAC_PI_2 } else { -FRAC_PI_2 };
        car(
            mesh,
            Vec2::new(doors + along, out(middle + lane * 26.0)),
            heading,
            9000 + k as u32,
        );
    }
    // The far pavement, and the buildings over it, and down either side.
    block(
        mesh,
        Vec3::new(west, y(0.0), out(STREET.1)),
        Vec3::new(east, y(KERB), out(FAR_PAVEMENT)),
        CONCRETE,
    );
    // Their parapets stand out a metre round their roofs: inside the box
    // of sky.
    let (west, east) = (west + 2.0, east - 2.0);
    let mut x = west;
    let mut n = 0;
    while x < east {
        let width = 140.0 + 140.0 * random01(n * 7);
        building(
            mesh,
            Vec2::new(x, out(FAR_PAVEMENT)),
            Vec2::new(
                (x + width).min(east),
                out(FAR_PAVEMENT + 120.0 + 60.0 * random01(n * 7 + 1)),
            ),
            -Vec3::Z,
            n,
        );
        x += width + 6.0;
        n += 1;
    }
    for side in [-1.0, 1.0] {
        let mut o = LANE + 20.0;
        while o < FAR_PAVEMENT - 20.0 {
            let width = 120.0 + 100.0 * random01(n * 7);
            let near = doors + side * (SKY_HALF - 140.0);
            let far = doors + side * (SKY_HALF - 2.0);
            building(
                mesh,
                Vec2::new(near.min(far), out(o)),
                Vec2::new(near.max(far), out((o + width).min(FAR_PAVEMENT - 20.0))),
                -Vec3::X * side,
                n,
            );
            o += width + 6.0;
            n += 1;
        }
    }
}

/// The canopy over the pavement along the shop's front: a slab over it,
/// pale underneath, on slim columns.
fn canopy(mesh: &mut MeshBuilder, entrance: &Entrance, wall: f32) {
    let (west, east) = (entrance.from - 30.0, entrance.to + 30.0);
    block(
        mesh,
        Vec3::new(west, y(GLASS_TOP + 4.0), wall + 0.5),
        Vec3::new(east, y(GLASS_TOP + 7.0), wall + 40.0),
        Color::srgb(0.85, 0.86, 0.88),
    );
    let mut x = west + 8.0;
    while x < east {
        prism(
            mesh,
            Vec3::new(x, y(KERB), wall + 36.0),
            Vec3::new(x, y(GLASS_TOP + 4.0), wall + 36.0),
            0.9,
            8,
            Color::srgb(0.5, 0.51, 0.54),
        );
        x += 46.0;
    }
}

/// The colours of cars, most of them white, silver, black or grey.
const CAR_PAINTS: [Color; 12] = [
    Color::srgb(0.92, 0.92, 0.91),
    Color::srgb(0.92, 0.92, 0.91),
    Color::srgb(0.7, 0.71, 0.73),
    Color::srgb(0.7, 0.71, 0.73),
    Color::srgb(0.06, 0.06, 0.07),
    Color::srgb(0.06, 0.06, 0.07),
    Color::srgb(0.38, 0.39, 0.42),
    Color::srgb(0.65, 0.07, 0.06),
    Color::srgb(0.12, 0.25, 0.6),
    Color::srgb(0.08, 0.12, 0.3),
    Color::srgb(0.16, 0.34, 0.22),
    Color::srgb(0.76, 0.68, 0.54),
];

/// The shape of a car (m, twelve times life): how long, wide and tall, how
/// high its body is to its waist, and where its cabin's foot and roof begin
/// and end along it (back to front, from its middle).
struct CarShape {
    length: f32,
    width: f32,
    height: f32,
    waist: f32,
    foot: (f32, f32),
    roof: (f32, f32),
}

/// A saloon, a hatchback, an SUV and a people carrier.
const CARS: [CarShape; 4] = [
    CarShape {
        length: 54.0,
        width: 21.6,
        height: 17.0,
        waist: 9.6,
        foot: (-16.0, 11.0),
        roof: (-12.0, 3.0),
    },
    CarShape {
        length: 46.0,
        width: 20.4,
        height: 17.5,
        waist: 9.6,
        foot: (-21.0, 9.0),
        roof: (-18.0, 1.0),
    },
    CarShape {
        length: 56.0,
        width: 22.8,
        height: 21.0,
        waist: 11.5,
        foot: (-26.0, 13.0),
        roof: (-25.0, 7.0),
    },
    CarShape {
        length: 60.0,
        width: 23.4,
        height: 25.0,
        waist: 11.5,
        foot: (-29.0, 22.0),
        roof: (-28.5, 13.0),
    },
];

/// A car, parked or in the street, at `at` (x, z), its front toward
/// `heading` (turned about the vertical from +z), its shape and paint from
/// `n`: a body to its waist on four wheels, a cabin of glass over it under
/// its roof, its lights, its shade under it.
fn car(mesh: &mut MeshBuilder, at: Vec2, heading: f32, n: u32) {
    let shape = &CARS[(random01(n * 13 + 5) * CARS.len() as f32) as usize % CARS.len()];
    let paint =
        CAR_PAINTS[(random01(n * 13 + 6) * CAR_PAINTS.len() as f32) as usize % CAR_PAINTS.len()];
    let rotation = Quat::from_rotation_y(heading);
    let base = Vec3::new(at.x, y(ASPHALT), at.y);
    let place = |p: Vec3| base + rotation * p;
    let (half_length, half_width) = (shape.length / 2.0, shape.width / 2.0);
    let extent = (rotation * Vec3::new(half_width + 3.0, 0.0, half_length + 3.0)).abs();
    shade(
        mesh,
        at + shade_offset(4.0),
        1.0,
        Vec2::new(extent.x, extent.z),
        TARMAC,
    );
    // The wheels, as little of them as is seen under it from across the
    // car park.
    const WHEEL: f32 = 3.9;
    for (side, end) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let z = end * (half_length - 10.0);
        prism(
            mesh,
            place(Vec3::new(side * (half_width - 2.8), WHEEL, z)),
            place(Vec3::new(side * half_width, WHEEL, z)),
            WHEEL,
            6,
            Color::srgb(0.06, 0.06, 0.07),
        );
    }
    // The body, its ends a little drawn in at its waist; the cabin of glass,
    // narrower to its roof, the roof painted.
    let clearance = 3.2;
    frustum(
        mesh,
        [
            place(Vec3::new(-half_width, clearance, -half_length)),
            place(Vec3::new(half_width, clearance, -half_length)),
            place(Vec3::new(half_width, clearance, half_length)),
            place(Vec3::new(-half_width, clearance, half_length)),
            place(Vec3::new(-half_width, shape.waist, -half_length + 1.5)),
            place(Vec3::new(half_width, shape.waist, -half_length + 1.5)),
            place(Vec3::new(half_width, shape.waist, half_length - 2.5)),
            place(Vec3::new(-half_width, shape.waist, half_length - 2.5)),
        ],
        |_| paint,
    );
    let glass = Color::srgb(0.16, 0.2, 0.26);
    let inset = 1.0;
    let narrow = 2.6;
    frustum(
        mesh,
        [
            place(Vec3::new(-half_width + inset, shape.waist, shape.foot.0)),
            place(Vec3::new(half_width - inset, shape.waist, shape.foot.0)),
            place(Vec3::new(half_width - inset, shape.waist, shape.foot.1)),
            place(Vec3::new(-half_width + inset, shape.waist, shape.foot.1)),
            place(Vec3::new(-half_width + narrow, shape.height, shape.roof.0)),
            place(Vec3::new(half_width - narrow, shape.height, shape.roof.0)),
            place(Vec3::new(half_width - narrow, shape.height, shape.roof.1)),
            place(Vec3::new(-half_width + narrow, shape.height, shape.roof.1)),
        ],
        |normal| {
            if normal.y > 0.9 {
                paint
            } else {
                // The sky in the windscreen and the back window.
                glass.mix(&Color::srgb(0.5, 0.62, 0.75), normal.y.max(0.0))
            }
        },
    );
    // Its lights, white in front and red behind, and its bumpers.
    for (end, light) in [
        (1.0, Color::srgb(0.98, 0.97, 0.88)),
        (-1.0, Color::srgb(0.75, 0.08, 0.06)),
    ] {
        // Where its end is at the lights' height, drawn in from its foot to
        // its waist.
        let drawn_in =
            if end > 0.0 { 2.5 } else { 1.5 } * (7.6 - clearance) / (shape.waist - clearance);
        let z = end * (half_length - drawn_in);
        for side in [-1.0, 1.0] {
            let x = side * (half_width - 3.6);
            slab_at(
                mesh,
                &place,
                Vec3::new(x, 7.6, z),
                Vec3::new(2.4, 0.8, 0.3),
                light,
            );
        }
        slab_at(
            mesh,
            &place,
            Vec3::new(0.0, clearance + 1.0, end * (half_length + 0.2)),
            Vec3::new(half_width, 1.0, 0.4),
            Color::srgb(0.12, 0.12, 0.13),
        );
    }
}

/// A box `half` its size either way round `center`, in a frame `place`
/// takes to the world (turned only about the vertical), in the sun.
fn slab_at(
    mesh: &mut MeshBuilder,
    place: &impl Fn(Vec3) -> Vec3,
    center: Vec3,
    half: Vec3,
    color: Color,
) {
    let corner = |x: f32, y: f32, z: f32| place(center + half * Vec3::new(x, y, z));
    frustum(
        mesh,
        [
            corner(-1.0, -1.0, -1.0),
            corner(1.0, -1.0, -1.0),
            corner(1.0, -1.0, 1.0),
            corner(-1.0, -1.0, 1.0),
            corner(-1.0, 1.0, -1.0),
            corner(1.0, 1.0, -1.0),
            corner(1.0, 1.0, 1.0),
            corner(-1.0, 1.0, 1.0),
        ],
        |_| color,
    );
}

/// A solid of six faces from its corners, the four round its foot and the
/// four round its top, in the same order, its underside left open; each
/// face of the colour `paint` gives the way it faces, in the sun.
fn frustum(mesh: &mut MeshBuilder, corners: [Vec3; 8], paint: impl Fn(Vec3) -> Color) {
    let center = corners.iter().sum::<Vec3>() / 8.0;
    let [a, b, c, d, e, f, g, h] = corners;
    for face in [
        [e, f, g, h],
        [a, b, f, e],
        [b, c, g, f],
        [c, d, h, g],
        [d, a, e, h],
    ] {
        let middle = face.iter().sum::<Vec3>() / 4.0;
        let mut normal = (face[1] - face[0])
            .cross(face[2] - face[0])
            .normalize_or_zero();
        if normal.dot(middle - center) < 0.0 {
            normal = -normal;
        }
        mesh.quad(
            [
                (face[0], Vec2::ZERO),
                (face[1], Vec2::X),
                (face[2], Vec2::ONE),
                (face[3], Vec2::Y),
            ],
            normal,
            sunlit(paint(normal), normal),
        );
    }
}

/// A car park's lamp post at `at` (x, z), on a concrete foot, an arm either
/// way along the row at its top, a lamp at each end.
fn lamp_post(mesh: &mut MeshBuilder, at: Vec2) {
    const TOP: f32 = 96.0;
    let grey = Color::srgb(0.55, 0.57, 0.6);
    block(
        mesh,
        Vec3::new(at.x - 2.5, y(0.0), at.y - 2.5),
        Vec3::new(at.x + 2.5, y(4.0), at.y + 2.5),
        CONCRETE,
    );
    prism(
        mesh,
        Vec3::new(at.x, y(4.0), at.y),
        Vec3::new(at.x, y(TOP), at.y),
        1.1,
        8,
        grey,
    );
    block(
        mesh,
        Vec3::new(at.x - 12.0, y(TOP - 1.2), at.y - 0.6),
        Vec3::new(at.x + 12.0, y(TOP), at.y + 0.6),
        grey,
    );
    for side in [-1.0, 1.0] {
        let x = at.x + side * 12.0;
        block(
            mesh,
            Vec3::new(x - 4.0, y(TOP - 2.0), at.y - 2.0),
            Vec3::new(x + 4.0, y(TOP - 0.4), at.y + 2.0),
            grey,
        );
        // Its lamp, underneath.
        let at3 = |dx: f32, dz: f32| (Vec3::new(x + dx, y(TOP - 2.05), at.y + dz), Vec2::ZERO);
        mesh.quad(
            [
                at3(-3.6, -1.6),
                at3(3.6, -1.6),
                at3(3.6, 1.6),
                at3(-3.6, 1.6),
            ],
            -Vec3::Y,
            Color::srgb(1.0, 0.98, 0.9),
        );
    }
    shade(
        mesh,
        at + shade_offset(TOP / 2.0),
        1.0,
        Vec2::new(3.0, 3.0),
        TARMAC,
    );
}

/// A tree at `at` (x, z), on ground `ground` high: a trunk and a round
/// crown of three lumps, its green and size from `n`, its shade under it.
fn tree(mesh: &mut MeshBuilder, at: Vec2, ground: f32, n: u32) {
    let r = |k: u32| random01(n * 17 + k + 3);
    let size = 0.8 + 0.4 * r(0);
    let green = Color::srgb(0.2, 0.42, 0.16).mix(&Color::srgb(0.38, 0.55, 0.2), r(1));
    let foot = Vec3::new(at.x, y(ground), at.y);
    prism(
        mesh,
        foot,
        foot + Vec3::Y * 50.0 * size,
        2.4 * size,
        6,
        Color::srgb(0.38, 0.27, 0.18),
    );
    for (k, offset) in [
        Vec3::new(0.0, 72.0, 0.0),
        Vec3::new(-14.0, 58.0, 6.0),
        Vec3::new(12.0, 60.0, -8.0),
    ]
    .into_iter()
    .enumerate()
    {
        let radius = (24.0 + 8.0 * r(k as u32 + 2)) * size;
        ball_of(
            mesh,
            foot + offset * size,
            radius,
            Vec3::new(1.0, 0.85, 1.0),
            TAU * r(6),
            (5, 7),
            |normal| sunlit(green, normal),
        );
    }
    shade(
        mesh,
        at + shade_offset(65.0 * size),
        30.0 * size,
        Vec2::ONE,
        GRASS,
    );
}

/// The colours of buildings' walls.
const WALLS: [Color; 6] = [
    Color::srgb(0.88, 0.82, 0.7),
    Color::srgb(0.68, 0.38, 0.3),
    Color::srgb(0.66, 0.67, 0.7),
    Color::srgb(0.92, 0.91, 0.88),
    Color::srgb(0.85, 0.66, 0.42),
    Color::srgb(0.72, 0.79, 0.85),
];

/// A building from `a` to `b` (x, z, its foot), its front toward `front`
/// (along x or z), as tall as `n` makes it: storeys of windows along its
/// front, shop fronts at its foot under an awning, a parapet round its
/// roof.
fn building(mesh: &mut MeshBuilder, a: Vec2, b: Vec2, front: Vec3, n: u32) {
    const STOREY: f32 = 36.0;
    let r = |k: u32| random01(n * 23 + k + 11);
    let storeys = 3 + (r(0) * 5.0) as u32;
    let height = storeys as f32 * STOREY + 12.0;
    let color = WALLS[(r(1) * WALLS.len() as f32) as usize % WALLS.len()];
    let (min, max) = (a.min(b), a.max(b));
    block(
        mesh,
        Vec3::new(min.x, y(0.0), min.y),
        Vec3::new(max.x, y(height), max.y),
        color,
    );
    block(
        mesh,
        Vec3::new(min.x - 1.0, y(height), min.y - 1.0),
        Vec3::new(max.x + 1.0, y(height + 4.0), max.y + 1.0),
        color.mix(&Color::BLACK, 0.2),
    );
    // Its front: along x if it faces z, along z if it faces x.
    let along_x = front.z.abs() > 0.5;
    let (start, end) = if along_x {
        (min.x, max.x)
    } else {
        (min.y, max.y)
    };
    let face = if along_x {
        if front.z < 0.0 { min.y } else { max.y }
    } else if front.x < 0.0 {
        min.x
    } else {
        max.x
    } + if front.x + front.z < 0.0 { -0.3 } else { 0.3 };
    let window = sunlit(Color::srgb(0.34, 0.46, 0.58), front);
    let at = |s: f32, h: f32| {
        let point = if along_x {
            Vec3::new(s, y(h), face)
        } else {
            Vec3::new(face, y(h), s)
        };
        (point, Vec2::ZERO)
    };
    let mut pane = |s0: f32, s1: f32, h0: f32, h1: f32, color: Color| {
        mesh.quad(
            [at(s0, h0), at(s1, h0), at(s1, h1), at(s0, h1)],
            front,
            color,
        );
    };
    let columns = ((end - start - 12.0) / 30.0).floor().max(1.0) as u32;
    let step = (end - start) / columns as f32;
    for storey in 1..storeys {
        let h = storey as f32 * STOREY + 9.0;
        for k in 0..columns {
            let s = start + (k as f32 + 0.5) * step;
            pane(s - 8.0, s + 8.0, h, h + 18.0, window);
        }
    }
    // At its foot, shop windows under an awning.
    pane(
        start + 6.0,
        end - 6.0,
        4.0,
        26.0,
        window.mix(&Color::BLACK, 0.3),
    );
    let awning = WALLS[(r(2) * WALLS.len() as f32) as usize % WALLS.len()]
        .mix(&Color::srgb(0.7, 0.1, 0.1), 0.6);
    let out = front * 8.0;
    let (s0, s1) = (start + 4.0, end - 4.0);
    let (p0, p1) = (at(s0, 30.0).0, at(s1, 30.0).0);
    mesh.quad(
        [
            (p0, Vec2::ZERO),
            (p1, Vec2::X),
            (p1 + out - Vec3::Y * 4.0, Vec2::ONE),
            (p0 + out - Vec3::Y * 4.0, Vec2::Y),
        ],
        front + Vec3::Y,
        sunlit(awning, (front + Vec3::Y).normalize()),
    );
}

/// The box of sky round the outside: its far side and either side, paler
/// down to the horizon, its top the blue overhead; and clouds on it.
fn sky(mesh: &mut MeshBuilder, doors: f32, wall: f32) {
    // Colours at heights up it.
    const BANDS: [(f32, Color); 4] = [
        (0.0, Color::srgb(0.84, 0.9, 0.96)),
        (250.0, Color::srgb(0.64, 0.78, 0.94)),
        (800.0, Color::srgb(0.46, 0.65, 0.92)),
        (SKY_HEIGHT, Color::srgb(0.36, 0.56, 0.9)),
    ];
    let (west, east) = (doors - SKY_HALF, doors + SKY_HALF);
    let (near, far) = (wall - 50.0, wall + SKY);
    for (a, b) in [
        (Vec2::new(west, far), Vec2::new(east, far)),
        (Vec2::new(west, near), Vec2::new(west, far)),
        (Vec2::new(east, near), Vec2::new(east, far)),
    ] {
        let middle = (a + b) / 2.0;
        let inward = Vec2::new(doors, wall + SKY / 2.0) - middle;
        let facing = Vec3::new(inward.x, 0.0, inward.y).normalize_or_zero();
        for pair in BANDS.windows(2) {
            let [(h0, c0), (h1, c1)] = [pair[0], pair[1]];
            let p = |point: Vec2, h: f32| Vec3::new(point.x, y(h), point.y);
            mesh.shaded_triangle(
                [
                    (p(a, h0), Vec2::ZERO, c0),
                    (p(b, h0), Vec2::X, c0),
                    (p(b, h1), Vec2::ONE, c1),
                ],
                facing,
            );
            mesh.shaded_triangle(
                [
                    (p(a, h0), Vec2::ZERO, c0),
                    (p(b, h1), Vec2::ONE, c1),
                    (p(a, h1), Vec2::Y, c1),
                ],
                facing,
            );
        }
    }
    let top = |x: f32, z: f32| (Vec3::new(x, y(SKY_HEIGHT), z), Vec2::ZERO);
    mesh.quad(
        [
            top(west, near),
            top(east, near),
            top(east, far),
            top(west, far),
        ],
        -Vec3::Y,
        BANDS[3].1,
    );
    // Clouds: lumps of white, grey at their feet, a little in front of the
    // far side and the sides.
    for k in 0..12u32 {
        let r = |j: u32| random01(k * 29 + j + 101);
        let side = k % 3;
        let height = 260.0 + 520.0 * r(0);
        let size = 50.0 + 60.0 * r(1);
        let (center, facing) = match side {
            0 => (
                Vec3::new(west + (east - west) * r(2), y(height), far - 4.0),
                -Vec3::Z,
            ),
            1 => (
                Vec3::new(west + 4.0, y(height), near + (far - near) * r(2)),
                Vec3::X,
            ),
            _ => (
                Vec3::new(east - 4.0, y(height), near + (far - near) * r(2)),
                -Vec3::X,
            ),
        };
        let across = facing.cross(Vec3::Y).normalize();
        for lump in 0..5u32 {
            let r = |j: u32| random01(k * 131 + lump * 7 + j + 7);
            let offset = across * size * 1.6 * (r(0) - 0.5) * 2.0 + Vec3::Y * size * 0.4 * r(1);
            let radius = size * (0.45 + 0.35 * r(2));
            let white = if offset.y < size * 0.15 {
                Color::srgb(0.9, 0.92, 0.96)
            } else {
                Color::WHITE
            };
            disc(
                mesh,
                center + offset + facing * lump as f32 * 0.5,
                radius,
                facing,
                across,
                white,
            );
        }
    }
}

/// A flat round disc round `center`, `radius` round, squashed to half as
/// tall, facing `facing`, `across` along it.
fn disc(
    mesh: &mut MeshBuilder,
    center: Vec3,
    radius: f32,
    facing: Vec3,
    across: Vec3,
    color: Color,
) {
    const SIDES: usize = 12;
    let at = |k: usize| {
        let angle = k as f32 / SIDES as f32 * TAU;
        center + (across * angle.cos() + Vec3::Y * angle.sin() * 0.5) * radius
    };
    for k in 0..SIDES {
        mesh.triangle(
            [(center, Vec2::ZERO), (at(k), Vec2::X), (at(k + 1), Vec2::Y)],
            facing,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;

    /// The toy stores with a way in, and theirs.
    fn entrances() -> Vec<(&'static str, &'static ToyStore, &'static Entrance)> {
        CIRCUITS
            .iter()
            .filter_map(|circuit| {
                let store = circuit.scenery.themes.get::<ToyStore>()?;
                Some((circuit.name, store, store.entrance.as_ref()?))
            })
            .collect()
    }

    #[test]
    fn there_is_a_way_into_a_shop() {
        assert!(!entrances().is_empty());
    }

    #[test]
    fn the_doors_and_a_pane_either_side_fit_in_the_glass_under_the_stripes() {
        // The bar over the doors under the glass's head, and the head under
        // the stripes round the room's top.
        const { assert!(DOOR_HEIGHT + BAR < GLASS_TOP - BAR) };
        const { assert!(GLASS_TOP + 0.6 < super::super::toys::WALL_SHELF_HEIGHT + 2.0) };
        for (name, store, entrance) in entrances() {
            let (min, max) = store.room;
            assert!(
                min.x < entrance.from - BAR && entrance.to + BAR < max.x,
                "{name}"
            );
            assert!(
                entrance.from + PANE / 2.0 < entrance.doors - DOORS / 2.0 - BAR,
                "{name}"
            );
            assert!(
                entrance.doors + DOORS / 2.0 + BAR + PANE / 2.0 < entrance.to,
                "{name}"
            );
        }
    }

    #[test]
    fn nothing_of_the_outside_comes_into_the_shop() {
        for (name, store, entrance) in entrances() {
            let (min, max) = store.room;
            let mut outside = MeshBuilder::default();
            car_park(&mut outside, entrance, max.y);
            sky(&mut outside, entrance.doors, max.y);
            let mesh = outside.build();
            let Some(positions) = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|positions| positions.as_float3())
            else {
                panic!("{name}: no positions");
            };
            for &[x, height, z] in positions {
                let inside = (min.x + 0.01..max.x - 0.01).contains(&x)
                    && (min.y + 0.01..max.y - 0.01).contains(&z)
                    && height < y(store.ceiling);
                assert!(!inside, "{name}: ({x}, {height}, {z}) is in the shop");
                // And all of it in the box of sky.
                let sky = entrance.doors - SKY_HALF - 0.01..=entrance.doors + SKY_HALF + 0.01;
                assert!(
                    sky.contains(&x) && z <= max.y + SKY + 0.01 && height <= y(SKY_HEIGHT) + 0.01,
                    "{name}: ({x}, {height}, {z}) is out of the sky"
                );
            }
        }
    }

    #[test]
    fn the_sun_outside_lights_what_faces_it_more_than_what_faces_away() {
        let white = Color::WHITE;
        let lit = sunlit(white, SUN).to_linear();
        let shade = sunlit(white, -SUN).to_linear();
        assert!(lit.red > shade.red * 1.5);
        // Its shade falls away from it, out from the shop (to +z).
        assert!(shade_offset(10.0).y > 0.0);
    }
}
