//! Tyres for the imported bikes. The Meshy wheels bring their tyres as
//! lumpy, low-poly rings (their outline wanders by a couple of centimetres
//! and some have smudges of the bodywork's colours), so they are cut off the
//! wheel ([`strip`]) and replaced by a true surface of revolution ([`mesh`])
//! with the rounded profile of a racing tyre. Its tread pattern ([`Tread`])
//! is painted at startup into a colour texture and a normal map that repeat
//! round the tyre, so the grooves turn with the wheel.

use std::f32::consts::{PI, TAU};

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::Design;
use crate::geometry::{random01, smoothstep, value_noise};
use crate::world::generate_mipmaps;

/// Segments round the tyre and round its cross-section.
const SEGMENTS: usize = 96;
const PROFILE: usize = 32;
/// How many times the tread pattern repeats round the tyre.
const REPEATS: f32 = 18.0;
/// Where the new tyre meets the rim, and where the old one is cut off, as
/// fractions of the wheel's radius: the Meshy tyres are rings from about
/// 0.62 of the radius outwards, their rims end inside that.
const BEAD: f32 = 0.6;
const STRIP: f32 = 0.63;
/// Nor does any triangle reaching out beyond this: the spikes where the
/// Meshy rims meet their tyres.
const STRIP_REACH: f32 = 0.67;
/// The widest part of the old tyre is measured beyond this fraction of the
/// radius, and the new one is a little narrower than it.
const WIDTH_FROM: f32 = 0.7;
const WIDTH_SCALE: f32 = 0.95;
/// The cross-section is made of two half superellipses: a little squarer
/// than an ellipse on the crown, with full sidewalls, and squarer still
/// towards the rim, where the tyre is as wide as the rim it covers.
const SQUARENESS: f32 = 2.6;
const BEAD_SQUARENESS: f32 = 4.0;
/// Half the angle of the cross-section covered by the tread, from the crown
/// (rad); the sidewalls go on round to the rim.
const TREAD_ANGLE: f32 = 75.0 * PI / 180.0;
/// Where the tread band lies across the texture (v), centred on 0.5.
const TREAD_V: f32 = 0.3;
/// Size of the tread textures (px): one repeat round, the whole width
/// across.
const IMAGE: UVec2 = UVec2::new(128, 256);
/// About how long one repeat is round a tyre, against half the width of
/// its tread (both m): grooves are drawn in true proportions.
const REPEAT_LENGTH: f32 = 0.105;
const HALF_TREAD: f32 = 0.075;

/// Tread patterns, each with the colour of the stripe round its sidewalls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tread {
    /// A racing slick: no grooves, a red stripe (the soft compound).
    Slick,
    /// A sports road tyre: a lightning-bolt groove on each half of the
    /// tread, staggered from one side to the other.
    Sport,
    /// A touring tyre: a groove round the middle and chevrons out to the
    /// shoulders.
    Touring,
    /// Long, thin sipes sweeping out from the middle to the shoulders.
    Sipe,
}

impl Tread {
    pub const ALL: [Tread; 4] = [Tread::Slick, Tread::Sport, Tread::Touring, Tread::Sipe];

    /// The tyres each imported bike wears.
    pub fn of(design: Design) -> Self {
        match design {
            Design::Kraken | Design::Stiletto => Tread::Slick,
            Design::Shogun | Design::Falcon => Tread::Sport,
            Design::Legend | Design::Cobra => Tread::Touring,
            _ => Tread::Sipe,
        }
    }

    /// The colour of the stripe round the sidewalls (sRGB).
    fn stripe(self) -> [f32; 3] {
        match self {
            Tread::Slick => [0.82, 0.1, 0.08],
            Tread::Sport => [0.95, 0.76, 0.1],
            Tread::Touring => [0.15, 0.45, 0.9],
            Tread::Sipe => [0.88, 0.88, 0.86],
        }
    }

    /// How deep the grooves are at `(u, t)`, from 0 (rubber) to 1 (the
    /// bottom of a groove): `u` goes once round a repeat of the pattern
    /// (it wraps), `t` across the tread from -1 to 1. `blur` is the size of
    /// a pixel, in the units of `t`, for smooth edges.
    fn groove(self, u: f32, t: f32, blur: f32) -> f32 {
        // Distance from a point to a polyline in (u, t), in the units of t,
        // with the pattern repeating round the tyre.
        let to_polyline = |points: &[Vec2], offset: Vec2| {
            let mut nearest = f32::MAX;
            for wrap in [-1.0, 0.0, 1.0] {
                let p = Vec2::new(u + wrap, t) - offset;
                for pair in points.windows(2) {
                    nearest = nearest.min(segment_distance(p, pair[0], pair[1]));
                }
            }
            nearest
        };
        let groove = |distance: f32, half_width: f32| {
            1.0 - smoothstep(half_width - blur, half_width + blur, distance)
        };
        let mirror = |points: &[Vec2]| {
            points
                .iter()
                .map(|p| Vec2::new(p.x, -p.y))
                .collect::<Vec<_>>()
        };
        match self {
            Tread::Slick => 0.0,
            Tread::Sport => {
                const BOLT: [Vec2; 4] = [
                    Vec2::new(0.0, 0.16),
                    Vec2::new(0.3, 0.52),
                    Vec2::new(0.37, 0.46),
                    Vec2::new(0.68, 0.88),
                ];
                let right = to_polyline(&BOLT, Vec2::ZERO);
                let left = to_polyline(&mirror(&BOLT), Vec2::new(0.5, 0.0));
                groove(right.min(left), 0.05)
            }
            Tread::Touring => {
                const CHEVRON: [Vec2; 2] = [Vec2::new(0.0, 0.14), Vec2::new(0.3, 0.9)];
                let middle = groove(t.abs(), 0.045);
                let chevrons = [0.0, 0.5]
                    .into_iter()
                    .map(|shift| {
                        let offset = Vec2::new(shift, 0.0);
                        to_polyline(&CHEVRON, offset).min(to_polyline(&mirror(&CHEVRON), offset))
                    })
                    .fold(f32::MAX, f32::min);
                middle.max(groove(chevrons, 0.048))
            }
            Tread::Sipe => {
                let sweep: Vec<Vec2> = (0..=8)
                    .map(|k| {
                        let along = 0.06 + 0.8 * k as f32 / 8.0;
                        Vec2::new(0.42 * (along - 0.06).powf(1.6), along)
                    })
                    .collect();
                let right = to_polyline(&sweep, Vec2::ZERO);
                let left = to_polyline(&mirror(&sweep), Vec2::new(0.5, 0.0));
                groove(right.min(left), 0.026)
            }
        }
    }
}

/// Distance from `p` to the segment from `a` to `b`, with `u` stretched to
/// metres in the same proportion as `t`.
fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let scale = Vec2::new(REPEAT_LENGTH / HALF_TREAD, 1.0);
    let (p, a, b) = (p * scale, a * scale, b * scale);
    let ab = b - a;
    let along = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    p.distance(a + ab * along)
}

/// Cuts the tyre off a Meshy wheel `mesh`, in the bike's space, round
/// `axle` with `radius`: every triangle further from the axle than
/// [`STRIP`] of the radius goes. Also returns half the width of the tyre
/// that should take its place (m).
pub fn strip(mut mesh: Mesh, axle: Vec3, radius: f32) -> (Mesh, f32) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return (mesh, 0.0);
    };
    let from_axle = |p: &[f32; 3]| Vec2::new(p[1] - axle.y, p[2] - axle.z).length();
    let half_width = positions
        .iter()
        .filter(|p| from_axle(p) > WIDTH_FROM * radius)
        .map(|p| (p[0] - axle.x).abs())
        .fold(0.0, f32::max)
        * WIDTH_SCALE;
    let radii: Vec<f32> = positions.iter().map(from_axle).collect();
    if let Some(Indices::U32(indices)) = mesh.indices() {
        let kept: Vec<u32> = indices
            .chunks_exact(3)
            .filter(|triangle| {
                let reach = triangle.iter().map(|&i| radii[i as usize]);
                let mean = reach.clone().sum::<f32>() / 3.0;
                mean <= STRIP * radius && reach.fold(0.0, f32::max) <= STRIP_REACH * radius
            })
            .flatten()
            .copied()
            .collect();
        mesh.insert_indices(Indices::U32(kept));
    }
    (mesh, half_width)
}

/// Where a point of the cross-section is, `alpha` radians round it from the
/// crown: across (-1 to 1, of the half width) and up (-1 to 1, of the half
/// height), and the outward normal for a section `half` metres big.
fn section(alpha: f32, half: Vec2) -> (Vec2, Vec2) {
    let (sin, cos) = alpha.sin_cos();
    let power = |v: f32, p: f32| v.signum() * v.abs().powf(p);
    let n = if cos >= 0.0 {
        SQUARENESS
    } else {
        BEAD_SQUARENESS
    };
    let point = Vec2::new(power(sin, 2.0 / n), power(cos, 2.0 / n));
    let normal = Vec2::new(
        power(point.x, n - 1.0) / half.x,
        power(point.y, n - 1.0) / half.y,
    );
    // At the very sides of the crown the gradient vanishes in one axis;
    // fall back to the direction of the point.
    let normal = normal.try_normalize().unwrap_or(point.normalize_or_zero());
    (point, normal)
}

/// The texture's `v` for a point `alpha` radians round the cross-section:
/// the tread in the middle band, the sidewalls on either side of it.
fn v_of(alpha: f32) -> f32 {
    let side = 0.5 - TREAD_V;
    if alpha.abs() <= TREAD_ANGLE {
        0.5 + TREAD_V * alpha / TREAD_ANGLE
    } else {
        let beyond = (alpha.abs() - TREAD_ANGLE) / (PI - TREAD_ANGLE);
        0.5 + alpha.signum() * (TREAD_V + side * beyond)
    }
}

/// The inverse of [`v_of`].
fn alpha_of(v: f32) -> f32 {
    let off = v - 0.5;
    if off.abs() <= TREAD_V {
        off / TREAD_V * TREAD_ANGLE
    } else {
        let beyond = (off.abs() - TREAD_V) / (0.5 - TREAD_V);
        off.signum() * (TREAD_ANGLE + beyond * (PI - TREAD_ANGLE))
    }
}

/// A tyre round the X axis at the origin: `radius` to the crown, `half_width`
/// at its widest, sitting on the rim at [`BEAD`] of the radius. Its texture
/// coordinates go [`REPEATS`] times round and once across (see [`v_of`]).
pub fn mesh(radius: f32, half_width: f32) -> Mesh {
    let bead = BEAD * radius;
    let half = Vec2::new(half_width, (radius - bead) / 2.0);
    let middle = (radius + bead) / 2.0;
    let mut positions = Vec::with_capacity((SEGMENTS + 1) * (PROFILE + 1));
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    for i in 0..=SEGMENTS {
        let angle = i as f32 / SEGMENTS as f32 * TAU;
        let (sin, cos) = angle.sin_cos();
        for j in 0..=PROFILE {
            let alpha = -PI + TAU * j as f32 / PROFILE as f32;
            let (point, normal) = section(alpha, half);
            let across = point.x * half.x;
            let out = middle + point.y * half.y;
            positions.push([across, out * cos, out * sin]);
            normals.push([normal.x, normal.y * cos, normal.y * sin]);
            uvs.push([i as f32 / SEGMENTS as f32 * REPEATS, v_of(alpha)]);
        }
    }
    let row = (PROFILE + 1) as u32;
    let mut indices = Vec::with_capacity(SEGMENTS * PROFILE * 6);
    for i in 0..SEGMENTS as u32 {
        for j in 0..PROFILE as u32 {
            // Counter-clockwise seen from outside, so that the outside is
            // the face that is drawn.
            let (a, b) = (i * row + j, (i + 1) * row + j);
            indices.extend([a, b, a + 1, b, b + 1, a + 1]);
        }
    }
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices));
    mesh.with_generated_tangents()
        .expect("the tyre has normals and texture coordinates")
}

/// The rubber's shades (sRGB): the tread, a little lighter where it is
/// scrubbed in the middle, the sidewalls and the bottom of the grooves.
const TREAD_RUBBER: f32 = 0.13;
const SCRUBBED: f32 = 0.025;
const SIDEWALL_RUBBER: f32 = 0.09;
const GROOVE_RUBBER: f32 = 0.04;
/// Where the stripe runs round the sidewalls (rad from the crown), and how
/// steep the grooves' walls look in the normal map.
const STRIPE: (f32, f32) = (1.95, 2.07);
const GROOVE_RELIEF: f32 = 2.5;

/// The colour texture and the normal map of a tread pattern.
pub fn images(tread: Tread) -> (Image, Image) {
    let (width, height) = (IMAGE.x as usize, IMAGE.y as usize);
    // The size of a pixel across the tread, in the units of `t`.
    let pixel = 1.0 / (height as f32 * TREAD_V);
    let depth = |x: usize, y: usize| {
        let u = (x as f32 + 0.5) / width as f32;
        let v = (y as f32 + 0.5) / height as f32;
        let t = (v - 0.5) / TREAD_V;
        // Only on the tread, fading out at the shoulders.
        let on_tread = 1.0 - smoothstep(0.9, 1.0, t.abs());
        tread.groove(u, t, pixel) * on_tread
    };
    let grooves: Vec<f32> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| depth(x, y))
        .collect();
    let at = |x: isize, y: isize| {
        let x = x.rem_euclid(width as isize) as usize;
        let y = y.clamp(0, height as isize - 1) as usize;
        grooves[y * width + x]
    };
    let stripe = tread.stripe();
    let mut color = Vec::with_capacity(width * height * 4);
    let mut normal = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;
        let alpha = alpha_of(v).abs();
        let t = (v - 0.5) / TREAD_V;
        for x in 0..width {
            let groove = grooves[y * width + x];
            // The rubber's own grain, so it does not look painted.
            let grain = (value_noise(Vec2::new(x as f32, y as f32) * 0.35) - 0.5) * 0.02
                + (random01((y * width + x) as u32) - 0.5) * 0.012;
            let rubber = if alpha <= TREAD_ANGLE {
                TREAD_RUBBER + SCRUBBED * (1.0 - smoothstep(0.3, 0.8, t.abs()))
            } else {
                SIDEWALL_RUBBER.lerp(
                    TREAD_RUBBER,
                    1.0 - smoothstep(0.0, 0.2, alpha - TREAD_ANGLE),
                )
            };
            let shade = (rubber + grain).lerp(GROOVE_RUBBER, groove);
            let band = smoothstep(STRIPE.0 - 0.02, STRIPE.0 + 0.02, alpha)
                * (1.0 - smoothstep(STRIPE.1 - 0.02, STRIPE.1 + 0.02, alpha));
            let rgb = stripe.map(|c| shade.lerp(c, band));
            color.extend(rgb.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8));
            color.push(255);
            // The grooves are dips: their walls lean the normal away.
            let (x, y) = (x as isize, y as isize);
            let du = (at(x + 1, y) - at(x - 1, y)) * GROOVE_RELIEF;
            let dv = (at(x, y + 1) - at(x, y - 1)) * GROOVE_RELIEF;
            let n = Vec3::new(du, dv, 1.0).normalize();
            normal.extend(
                (n * 0.5 + 0.5)
                    .to_array()
                    .map(|c| (c * 255.0).round() as u8),
            );
            normal.push(255);
        }
    }
    (
        image(color, TextureFormat::Rgba8UnormSrgb),
        image(normal, TextureFormat::Rgba8Unorm),
    )
}

/// An image of the tread's size that repeats round the tyre, with mipmaps.
fn image(data: Vec<u8>, format: TextureFormat) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: IMAGE.x,
            height: IMAGE.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    generate_mipmaps(&mut image);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 8,
        ..default()
    });
    image
}

/// The material of a tread pattern: rubber, a little glossy.
pub fn material(tread: Tread, images: &mut Assets<Image>) -> StandardMaterial {
    let (color, normal) = self::images(tread);
    StandardMaterial {
        base_color_texture: Some(images.add(color)),
        normal_map_texture: Some(images.add(normal)),
        perceptual_roughness: 0.72,
        reflectance: 0.3,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_face_of_the_tyre_is_drawn_from_outside() {
        let tyre = mesh(0.3, 0.07);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            tyre.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            tyre.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("no normals");
        };
        let Some(Indices::U32(indices)) = tyre.indices() else {
            panic!("no indices");
        };
        // The front of each triangle (counter-clockwise, as drawn) faces the
        // way its corners' normals point: out of the tyre.
        for corners in indices.chunks(3) {
            let [a, b, c] = [0, 1, 2].map(|k| Vec3::from(positions[corners[k] as usize]));
            let front = (b - a).cross(c - a);
            if front.length() < 1e-9 {
                continue;
            }
            let normal: Vec3 = corners
                .iter()
                .map(|&k| Vec3::from(normals[k as usize]))
                .sum();
            assert!(front.dot(normal) > 0.0, "a face turned inwards at {a}");
        }
    }

    #[test]
    fn the_tyre_is_round_and_sits_on_its_rim() {
        let (radius, half_width) = (0.3, 0.07);
        let tyre = mesh(radius, half_width);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            tyre.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        // The crown of every segment round it is exactly at the radius.
        let per_segment = PROFILE + 1;
        for segment in positions.chunks(per_segment) {
            let outer = segment
                .iter()
                .map(|p| Vec2::new(p[1], p[2]).length())
                .fold(0.0, f32::max);
            assert!((outer - radius).abs() < 1e-4, "{outer}");
            let inner = segment
                .iter()
                .map(|p| Vec2::new(p[1], p[2]).length())
                .fold(f32::MAX, f32::min);
            assert!((inner - BEAD * radius).abs() < 1e-4, "{inner}");
        }
        let widest = positions.iter().map(|p| p[0].abs()).fold(0.0, f32::max);
        assert!((widest - half_width).abs() < 1e-4);
        assert!(tyre.attribute(Mesh::ATTRIBUTE_TANGENT).is_some());
    }

    #[test]
    fn normals_point_out_of_the_cross_section() {
        let half = Vec2::new(0.07, 0.06);
        for j in 0..PROFILE {
            let alpha = -PI + TAU * (j as f32 + 0.5) / PROFILE as f32;
            let (point, normal) = section(alpha, half);
            assert!(point.dot(normal) > 0.0, "{alpha}");
            assert!((normal.length() - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn the_texture_maps_the_tread_to_the_middle_band() {
        for alpha in [-3.0, -1.5, -0.5, 0.0, 0.7, 1.4, 2.9] {
            assert!((alpha_of(v_of(alpha)) - alpha).abs() < 1e-4);
        }
        assert_eq!(v_of(0.0), 0.5);
        assert!((v_of(TREAD_ANGLE) - (0.5 + TREAD_V)).abs() < 1e-6);
        assert!((v_of(-PI)).abs() < 1e-6 && (v_of(PI) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn every_pattern_but_the_slick_has_grooves() {
        for tread in Tread::ALL {
            let (color, normal) = images(tread);
            let color = color.data.unwrap();
            let normal = normal.data.unwrap();
            let grooved = (0..IMAGE.x * IMAGE.y)
                .filter(|&i| color[i as usize * 4] < 15)
                .count();
            let tilted = normal.chunks(4).filter(|n| n[2] < 250).count();
            if tread == Tread::Slick {
                assert_eq!(grooved, 0, "{tread:?}");
                assert_eq!(tilted, 0, "{tread:?}");
            } else {
                // Grooves cover a part of the tread, not all of it.
                let share = grooved as f32 / (IMAGE.x * IMAGE.y) as f32;
                assert!((0.01..0.3).contains(&share), "{tread:?}: {share}");
                assert!(tilted > 0, "{tread:?}");
            }
        }
    }

    #[test]
    fn the_pattern_repeats_seamlessly_round_the_tyre() {
        for tread in Tread::ALL {
            for t in [-0.7, -0.3, 0.0, 0.4, 0.8] {
                let a = tread.groove(0.0, t, 0.01);
                let b = tread.groove(1.0, t, 0.01);
                assert!((a - b).abs() < 1e-4, "{tread:?} at {t}");
            }
        }
    }

    #[test]
    fn stripping_leaves_the_rim_and_takes_the_tyre() {
        // A flat ring of triangles from the axle out to the radius.
        let (axle, radius) = (Vec3::new(0.0, 0.3, -0.7), 0.3);
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for k in 0..20 {
            let r0 = radius * k as f32 / 20.0;
            let r1 = radius * (k + 1) as f32 / 20.0;
            let base = positions.len() as u32;
            for (r, x) in [(r0, -0.05), (r1, -0.05), (r1, 0.05)] {
                positions.push([axle.x + x, axle.y + r, axle.z]);
            }
            indices.extend([base, base + 1, base + 2]);
        }
        let ring = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices));
        let (rim, half_width) = strip(ring, axle, radius);
        let kept = rim.indices().unwrap().len() / 3;
        assert!(kept > 5 && kept < 20, "{kept}");
        assert!((half_width - 0.05 * WIDTH_SCALE).abs() < 1e-5);
    }

    #[test]
    fn the_imported_bikes_wear_every_pattern() {
        let worn: Vec<Tread> = Design::ALL
            .into_iter()
            .filter(|design| design.model().is_some())
            .map(Tread::of)
            .collect();
        for tread in Tread::ALL {
            assert!(worn.contains(&tread), "{tread:?}");
        }
    }
}
