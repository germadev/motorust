//! Procedural textures of plants, painted at startup like the other
//! procedural textures: the lawn the grass is drawn with, tufts of blades
//! standing up beside the road, and clusters of leaves and of pine needles
//! for the crowns of the trees.
//!
//! The ones with see-through gaps (tufts and leaves) are drawn with alpha
//! masking: their mipmaps keep as much of them solid as the full-size
//! image has (see [`keep_coverage`]), so the trees and the grass do not
//! thin out and vanish in the distance.

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::textures::generate_mipmaps;
use crate::geometry::{fbm, random01, smoothstep};

/// The alpha below which masked textures are see-through.
pub const ALPHA_CUTOFF: f32 = 0.5;

/// A canvas of linear RGB colours with straight alpha, painted with
/// anti-aliased strokes. If it wraps, what goes past one edge comes back
/// in at the other, so the texture repeats without seams.
struct Canvas {
    width: u32,
    height: u32,
    wrap: bool,
    pixels: Vec<Vec4>,
}

impl Canvas {
    /// A canvas all of `color`, as opaque as `alpha`.
    fn new(width: u32, height: u32, wrap: bool, color: Color, alpha: f32) -> Self {
        let linear = color.to_linear();
        let pixel = Vec4::new(linear.red, linear.green, linear.blue, alpha);
        Self {
            width,
            height,
            wrap,
            pixels: vec![pixel; (width * height) as usize],
        }
    }

    /// Lays `color` over pixel (x, y) with this much `coverage` (0..1).
    fn blend(&mut self, x: i32, y: i32, color: Vec4, coverage: f32) {
        let (w, h) = (self.width as i32, self.height as i32);
        let (x, y) = if self.wrap {
            (x.rem_euclid(w), y.rem_euclid(h))
        } else if x < 0 || y < 0 || x >= w || y >= h {
            return;
        } else {
            (x, y)
        };
        let under = &mut self.pixels[(y * w + x) as usize];
        let a = color.w * coverage.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let out = a + under.w * (1.0 - a);
        let rgb = if under.w <= 0.0 {
            color.truncate()
        } else {
            (color.truncate() * a + under.truncate() * under.w * (1.0 - a)) / out
        };
        *under = rgb.extend(out);
    }

    /// A stroke from `from` to `to` (px), `width` wide at its start and
    /// `end_width` at its end, of colour `start` turning to `end` along it.
    fn stroke(
        &mut self,
        from: Vec2,
        to: Vec2,
        (width, end_width): (f32, f32),
        start: Vec4,
        end: Vec4,
    ) {
        let reach = width.max(end_width) * 0.5 + 1.0;
        let (min, max) = (from.min(to) - reach, from.max(to) + reach);
        let along = to - from;
        let length2 = along.length_squared().max(1e-6);
        for y in min.y.floor() as i32..=max.y.ceil() as i32 {
            for x in min.x.floor() as i32..=max.x.ceil() as i32 {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let t = ((p - from).dot(along) / length2).clamp(0.0, 1.0);
                let distance = p.distance(from + along * t);
                let radius = width.lerp(end_width, t) * 0.5;
                let coverage = radius - distance + 0.5;
                if coverage > 0.0 {
                    self.blend(x, y, start.lerp(end, t), coverage);
                }
            }
        }
    }

    /// A leaf: pointed at both ends, `length` long and `width` wide (px),
    /// centred on `center` along `direction`, `color` with a darker rib
    /// down its middle and darker edges.
    fn leaf(&mut self, center: Vec2, direction: Vec2, (length, width): (f32, f32), color: Vec4) {
        let along = direction.normalize();
        let across = along.perp();
        let reach = length * 0.5 + 1.0;
        for y in (center.y - reach).floor() as i32..=(center.y + reach).ceil() as i32 {
            for x in (center.x - reach).floor() as i32..=(center.x + reach).ceil() as i32 {
                let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - center;
                let u = d.dot(along) / (length * 0.5);
                if u.abs() >= 1.0 {
                    continue;
                }
                let half = width * 0.5 * (1.0 - u * u).powf(0.8);
                let v = d.dot(across);
                let coverage = half - v.abs() + 0.5;
                if coverage <= 0.0 {
                    continue;
                }
                let side = (v.abs() / half.max(0.5)).min(1.0);
                // Lighter across the blade, darker at the rib and edges.
                let rib = 1.0 - 0.25 * (1.0 - (v.abs() * 2.0).min(1.0));
                let shade = rib * (1.0 - 0.25 * side * side) * (0.9 + 0.1 * u);
                let tinted = (color.truncate() * shade).extend(color.w);
                self.blend(x, y, tinted, coverage);
            }
        }
    }

    /// The canvas as an sRGB image with mipmaps; masked ones keep their
    /// coverage down the mipmaps.
    fn image(self, masked: bool) -> Image {
        let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let data = self
            .pixels
            .iter()
            .flat_map(|p| {
                let srgb = Color::linear_rgb(p.x, p.y, p.z).to_srgba();
                [byte(srgb.red), byte(srgb.green), byte(srgb.blue), byte(p.w)]
            })
            .collect();
        let mut image = Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        let address = if self.wrap {
            ImageAddressMode::Repeat
        } else {
            ImageAddressMode::ClampToEdge
        };
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: address,
            address_mode_v: address,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            // Masked cards are many and overlap: cheaper filtering for them.
            anisotropy_clamp: if masked { 4 } else { 16 },
            ..default()
        });
        generate_mipmaps(&mut image);
        if masked {
            keep_coverage(&mut image, ALPHA_CUTOFF);
        }
        image
    }
}

/// Raises the alpha of each mipmap of an alpha-masked image so as much of
/// it is solid (at least `cutoff`) as of the full-size image. Averaging
/// alone thins out the leaves and blades as they get smaller, until they
/// vanish.
pub fn keep_coverage(image: &mut Image, cutoff: f32) {
    let levels = image.texture_descriptor.mip_level_count;
    let (width, height) = (image.width() as usize, image.height() as usize);
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let cut = (cutoff * 255.0).round() as u32;
    let coverage = |pixels: &[u8], scale: f32| {
        let solid = pixels
            .chunks(4)
            .filter(|p| ((p[3] as f32 * scale).round() as u32) >= cut)
            .count();
        solid as f32 / (pixels.len() / 4) as f32
    };
    let wanted = coverage(&data[..width * height * 4], 1.0);
    let (mut offset, mut w, mut h) = (width * height * 4, width, height);
    for _ in 1..levels {
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
        let size = w * h * 4;
        let level = &mut data[offset..offset + size];
        // The scale that makes as much of it solid, found by halving.
        let (mut low, mut high) = (0.0f32, 4.0f32);
        for _ in 0..16 {
            let middle = (low + high) / 2.0;
            if coverage(level, middle) < wanted {
                low = middle;
            } else {
                high = middle;
            }
        }
        for pixel in level.chunks_mut(4) {
            pixel[3] = (pixel[3] as f32 * high).round().min(255.0) as u8;
        }
        offset += size;
    }
}

/// A random number from 0 to 1 for stroke `n`, number `k` of its numbers.
fn rand(seed: u32, n: u32, k: u32) -> f32 {
    random01(seed.wrapping_mul(7919) ^ n.wrapping_mul(16) ^ k)
}

/// Lawn: a mat of blades of grass seen from above, of several greens, with
/// a few dry ones, over the shade between them, which is the `photo` (the
/// grass photograph, darkened). It repeats both ways. Its average colour is
/// the photo's, so the grass keeps its tone and only gains detail.
pub fn lawn(photo: &Image) -> Image {
    let (width, height) = (photo.width(), photo.height());
    let Some(source) = photo.data.as_ref() else {
        return photo.clone();
    };
    let photo_at = |x: u32, y: u32| {
        let i = ((y * width + x) * 4) as usize;
        Color::srgb_u8(source[i], source[i + 1], source[i + 2]).to_linear()
    };
    let mut canvas = Canvas::new(width, height, true, Color::BLACK, 1.0);
    for y in 0..height {
        for x in 0..width {
            let p = photo_at(x, y);
            canvas.pixels[(y * width + x) as usize] =
                Vec4::new(p.red, p.green, p.blue, 1.0) * Vec4::new(0.55, 0.55, 0.55, 1.0);
        }
    }
    let greens = [
        Color::srgb(0.24, 0.5, 0.1),
        Color::srgb(0.33, 0.6, 0.13),
        Color::srgb(0.42, 0.68, 0.17),
        Color::srgb(0.2, 0.42, 0.08),
        Color::srgb(0.5, 0.66, 0.2),
    ];
    let dry = Color::srgb(0.66, 0.62, 0.34);
    let scale = width as f32 / 512.0;
    let blades = (24_000.0 * scale * scale) as u32;
    for n in 0..blades {
        let r = |k: u32| rand(1, n, k);
        let root = Vec2::new(r(0) * width as f32, r(1) * height as f32);
        // Clumps lean the same way, as grass does where it is trodden or
        // the wind combs it.
        let lean = fbm(root / (60.0 * scale), 2) * std::f32::consts::TAU * 2.0;
        let angle = lean + (r(2) - 0.5) * 2.4;
        let length = (5.0 + 9.0 * r(3)) * scale;
        let tip = root + Vec2::from_angle(angle) * length;
        let color = if r(4) < 0.03 {
            dry
        } else {
            greens[(r(5) * greens.len() as f32) as usize % greens.len()]
        }
        .to_linear();
        let base =
            Vec4::new(color.red, color.green, color.blue, 0.9) * Vec4::new(0.7, 0.7, 0.7, 1.0);
        let lit =
            Vec4::new(color.red, color.green, color.blue, 0.9) * Vec4::new(1.2, 1.2, 1.15, 1.0);
        canvas.stroke(root, tip, ((1.6 + r(6)) * scale, 0.5 * scale), base, lit);
    }
    // The same average colour as the photo.
    let average = |pixels: &mut dyn Iterator<Item = Vec3>| {
        let (sum, count) = pixels.fold((Vec3::ZERO, 0.0), |(s, c), p| (s + p, c + 1.0));
        sum / count
    };
    let target = average(&mut (0..height).flat_map(|y| {
        (0..width).map(move |x| {
            let i = ((y * width + x) * 4) as usize;
            let p = Color::srgb_u8(source[i], source[i + 1], source[i + 2]).to_linear();
            Vec3::new(p.red, p.green, p.blue)
        })
    }));
    let painted = average(&mut canvas.pixels.iter().map(|p| p.truncate()));
    let correction = target / painted.max(Vec3::splat(1e-4));
    for pixel in &mut canvas.pixels {
        *pixel = (pixel.truncate() * correction).extend(1.0);
    }
    canvas.image(false)
}

/// A tuft of grass seen from the side: blades rising from the bottom edge,
/// curving out and over, darker at their roots, on a clear ground. Pale,
/// to be tinted by the colour of the grass they grow in.
pub fn tuft() -> Image {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 128;
    let base = Color::srgb(0.8, 0.8, 0.75);
    let mut canvas = Canvas::new(WIDTH, HEIGHT, false, base, 0.0);
    for n in 0..70 {
        let r = |k: u32| rand(2, n, k);
        let x = 20.0 + r(0) * (WIDTH as f32 - 40.0);
        // Taller in the middle of the tuft.
        let middle = 1.0 - ((x / WIDTH as f32) * 2.0 - 1.0).abs();
        let height = HEIGHT as f32 * (0.35 + 0.6 * middle * (0.6 + 0.4 * r(1)));
        let lean = (r(2) - 0.5) * 1.2 + ((x / WIDTH as f32) - 0.5) * 0.9;
        let bend = (r(3) - 0.5) * 0.8;
        let shade = 0.65 + 0.35 * r(4);
        // A few dry, straw-coloured blades among the green.
        let green = if r(5) < 0.06 {
            Vec4::new(1.0, 0.85, 0.5, 1.0)
        } else {
            Vec4::new(1.0, 1.0, 0.95, 1.0)
        };
        let root = green * Vec4::new(0.45, 0.45, 0.45, 1.0);
        let tip = green * Vec4::new(shade, shade, shade, 1.0);
        let width = 3.0 + 3.0 * r(6);
        // A curve of four strokes, tapering to a point.
        let mut at = Vec2::new(x, HEIGHT as f32 + 2.0);
        let mut angle = -std::f32::consts::FRAC_PI_2 + lean * 0.4;
        for k in 0..4 {
            let t0 = k as f32 / 4.0;
            let t1 = (k + 1) as f32 / 4.0;
            angle += (lean + bend * t0) * 0.25;
            let next = at + Vec2::from_angle(angle) * height / 4.0;
            canvas.stroke(
                at,
                next,
                (width * (1.0 - t0 * 0.85), width * (1.0 - t1 * 0.95)),
                root.lerp(tip, t0),
                root.lerp(tip, t1),
            );
            at = next;
        }
    }
    canvas.image(true)
}

/// The ragged edge of the grass along the road, seen from above, to lie
/// over the edge of the asphalt: clear on the road's side (the top rows),
/// grass on the other (the bottom ones), and between them an uneven line
/// of grass with blades and clumps reaching out over the road. Towards the
/// bottom the grass thins out in specks, so it melts into the grass of the
/// ground instead of ending on a line. Pale, to be tinted by the colour of
/// the grass. It repeats along (u) but not across (v).
pub fn grass_edge() -> Image {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 64;
    let pale = Color::srgb(0.85, 0.85, 0.82);
    let mut canvas = Canvas::new(WIDTH, HEIGHT, true, pale, 0.0);
    // Where the grass begins down each column: an uneven line that repeats
    // along.
    let phases: Vec<f32> = (0..8)
        .map(|k| rand(7, k, 0) * std::f32::consts::TAU)
        .collect();
    let line = |u: f32| {
        let mut wave = 0.0;
        for (k, phase) in phases.iter().enumerate() {
            let frequency = (k + 1) as f32;
            wave += (u * std::f32::consts::TAU * frequency + phase).sin() / frequency.sqrt();
        }
        0.42 + 0.05 * wave
    };
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (u, v) = (
                (x as f32 + 0.5) / WIDTH as f32,
                (y as f32 + 0.5) / HEIGHT as f32,
            );
            let into = (v - line(u)) * HEIGHT as f32;
            // Thinning out into specks towards the bottom.
            let specks = rand(8, x * HEIGHT + y, 0);
            let thinning = smoothstep(0.75, 0.98, v);
            if into > 0.0 && specks > thinning {
                let shade = 0.78 + 0.22 * rand(9, x * HEIGHT + y, 0);
                let color = Vec4::new(shade, shade, shade * 0.97, 1.0);
                canvas.blend(x as i32, y as i32, color, into.min(1.0));
            }
        }
    }
    // Blades and little clumps reaching out over the road.
    for n in 0..520 {
        let r = |k: u32| rand(10, n, k);
        let x = r(0) * WIDTH as f32;
        let root = Vec2::new(x, (line(x / WIDTH as f32) + 0.04 * r(1)) * HEIGHT as f32);
        let angle = -std::f32::consts::FRAC_PI_2 + (r(2) - 0.5) * 1.8;
        let length = HEIGHT as f32 * (0.06 + 0.22 * r(3) * r(3));
        let tip = root + Vec2::from_angle(angle) * length;
        let shade = 0.72 + 0.28 * r(4);
        let color = Vec4::new(shade, shade, shade * 0.95, 1.0);
        canvas.stroke(
            root,
            tip,
            (2.2 + 1.5 * r(5), 0.6),
            color * Vec4::new(0.85, 0.85, 0.85, 1.0),
            color,
        );
    }
    let mut image = canvas.image(true);
    // Across it does not repeat: the clear road side must not take in the
    // grass of the other side.
    if let ImageSampler::Descriptor(descriptor) = &mut image.sampler {
        descriptor.address_mode_v = ImageAddressMode::ClampToEdge;
    }
    image
}

/// The leaves of the trees: two squares side by side, a cluster of broad
/// leaves on twigs on the left and a spray of pine needles on the right,
/// each round its middle on a clear ground. Pale, to be tinted by the
/// colour of each tree's foliage.
pub fn leaves() -> Image {
    const TILE: u32 = 256;
    let base = Color::srgb(0.7, 0.75, 0.62);
    let mut canvas = Canvas::new(2 * TILE, TILE, false, base, 0.0);
    let center = Vec2::splat(TILE as f32 / 2.0);
    let twig = Vec4::new(0.35, 0.28, 0.2, 1.0);
    // Broad leaves on a few twigs from the middle out.
    for n in 0..9 {
        let r = |k: u32| rand(3, n, k);
        let direction = Vec2::from_angle(n as f32 / 9.0 * std::f32::consts::TAU + r(0) * 0.6);
        let end = center + direction * TILE as f32 * (0.32 + 0.1 * r(1));
        canvas.stroke(center, end, (3.0, 1.5), twig, twig);
    }
    for n in 0..150 {
        let r = |k: u32| rand(4, n, k);
        // Thicker towards the middle, as round a twig's end.
        let distance = TILE as f32 * 0.4 * r(0).sqrt();
        let at = center + Vec2::from_angle(r(1) * std::f32::consts::TAU) * distance;
        let direction = (at - center)
            .normalize_or(Vec2::X)
            .rotate(Vec2::from_angle((r(2) - 0.5) * 1.6));
        let shade = 0.62 + 0.38 * r(3);
        let color = Vec4::new(shade, shade, shade * 0.95, 1.0);
        let length = 26.0 + 20.0 * r(4);
        canvas.leaf(
            at,
            direction,
            (length, length * (0.36 + 0.12 * r(5))),
            color,
        );
    }
    // Pine needles: bunches of thin strokes fanning out from twigs.
    let needles = center + Vec2::new(TILE as f32, 0.0);
    for n in 0..14 {
        let r = |k: u32| rand(5, n, k);
        let direction = Vec2::from_angle(n as f32 / 14.0 * std::f32::consts::TAU + r(0) * 0.4);
        let length = TILE as f32 * (0.28 + 0.12 * r(1));
        let end = needles + direction * length;
        canvas.stroke(needles, end, (3.0, 1.2), twig, twig);
        for k in 0..22 {
            let s = |j: u32| rand(6, n * 32 + k, j);
            let along = needles + direction * length * (0.15 + 0.85 * k as f32 / 22.0);
            for side in [-1.0, 1.0] {
                let spread = direction.rotate(Vec2::from_angle(side * (0.5 + 0.35 * s(0))));
                let tip = along + spread * TILE as f32 * (0.07 + 0.05 * s(1));
                let shade = 0.6 + 0.4 * s(2);
                let color = Vec4::new(shade, shade, shade * 0.9, 1.0);
                canvas.stroke(
                    along,
                    tip,
                    (2.4, 1.0),
                    color * Vec4::new(0.8, 0.8, 0.8, 1.0),
                    color,
                );
            }
        }
    }
    canvas.image(true)
}

/// Where each of the two kinds of foliage is in [`leaves`] (u from, u to).
pub const BROAD_LEAVES: (f32, f32) = (0.0, 0.5);
pub const NEEDLES: (f32, f32) = (0.5, 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(image: &Image, x: u32, y: u32) -> u8 {
        let data = image.data.as_ref().expect("painted");
        data[((y * image.width() + x) * 4 + 3) as usize]
    }

    /// The share of the pixels of mipmap `level` of `image` that are solid.
    fn coverage_at(image: &Image, level: u32) -> f32 {
        let data = image.data.as_ref().expect("painted");
        let (mut offset, mut w, mut h) = (0, image.width() as usize, image.height() as usize);
        for _ in 0..level {
            offset += w * h * 4;
            (w, h) = ((w / 2).max(1), (h / 2).max(1));
        }
        let pixels = &data[offset..offset + w * h * 4];
        let cut = (ALPHA_CUTOFF * 255.0) as u8;
        pixels.chunks(4).filter(|p| p[3] >= cut).count() as f32 / (w * h) as f32
    }

    #[test]
    fn leaves_are_clusters_on_a_clear_ground() {
        let image = leaves();
        assert_eq!((image.width(), image.height()), (512, 256));
        for (x, y) in [(0, 0), (255, 0), (256, 255), (511, 255), (0, 128)] {
            assert_eq!(alpha(&image, x, y), 0, "clear at ({x}, {y})");
        }
        assert!(alpha(&image, 128, 128) > 200, "solid in the middle");
        let full = coverage_at(&image, 0);
        assert!((0.2..0.8).contains(&full), "{full}");
    }

    #[test]
    fn masked_mipmaps_keep_their_coverage() {
        let image = leaves();
        let full = coverage_at(&image, 0);
        for level in 1..6 {
            let smaller = coverage_at(&image, level);
            assert!(
                (smaller - full).abs() < 0.12,
                "level {level}: {smaller} vs {full}"
            );
        }
    }

    #[test]
    fn the_grass_edge_is_ragged() {
        let image = grass_edge();
        let (width, height) = (image.width(), image.height());
        // Clear along the top (the road), grass lower down.
        assert!((0..width).all(|x| alpha(&image, x, 0) == 0));
        let solid = |y: u32| (0..width).filter(|&x| alpha(&image, x, y) > 128).count();
        assert!(solid(height * 6 / 10) > width as usize * 9 / 10);
        // Where the grass begins goes up and down along it.
        let starts: Vec<u32> = (0..width)
            .filter_map(|x| (0..height).find(|&y| alpha(&image, x, y) > 128))
            .collect();
        let (top, bottom) = (starts.iter().min().unwrap(), starts.iter().max().unwrap());
        assert!(bottom - top > 8, "{top}..{bottom}");
        // It does not repeat across: the road side stays clear when drawn.
        let ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("it has its sampler");
        };
        assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);
    }

    #[test]
    fn tufts_rise_from_the_bottom_edge() {
        let image = tuft();
        let (width, height) = (image.width(), image.height());
        assert_eq!(alpha(&image, 0, 0), 0);
        assert_eq!(alpha(&image, width - 1, 0), 0);
        let bottom = (0..width)
            .filter(|&x| alpha(&image, x, height - 1) > 128)
            .count();
        let top = (0..width).filter(|&x| alpha(&image, x, 4) > 128).count();
        assert!(bottom > top, "denser at the roots: {bottom} vs {top}");
        assert!(bottom > 0);
    }

    #[test]
    fn the_lawn_keeps_the_photo_s_colour_with_more_detail() {
        let photo = Image::new(
            Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            [90u8, 170, 50, 255].repeat(64 * 64),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        let image = lawn(&photo);
        let data = image.data.as_ref().expect("painted");
        let pixels = &data[..64 * 64 * 4];
        let mean = |c: usize| pixels.chunks(4).map(|p| p[c] as f32).sum::<f32>() / 4096.0;
        assert!((mean(1) - 170.0).abs() < 12.0, "green {}", mean(1));
        let spread = pixels.chunks(4).map(|p| p[1]).max().unwrap() as i32
            - pixels.chunks(4).map(|p| p[1]).min().unwrap() as i32;
        assert!(spread > 40, "blades show: {spread}");
        assert!(pixels.chunks(4).all(|p| p[3] == 255), "opaque");
    }
}
