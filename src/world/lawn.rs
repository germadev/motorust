//! The lawn's repetition broken up, as Options > Graphics > Grass tiling
//! says ([`GrassTiling`]). The lawn's texture covers 11 m (see
//! [`super::terrain::GRASS_TILE`]), and its clumps and patches, coming back
//! every 11 m over a whole hillside, are what gives the ground away. Two
//! ways of hiding it, both cutting the ground into small triangles, each
//! drawn from a piece of the lawn of its own, cross-fading into its
//! neighbours over their edges:
//!
//! - [`GrassTiling::Wide`] (the default): a lawn four times as wide each way
//!   ([`widen`]), put together like that once at start-up from pieces of the
//!   lawn, with its own relief: a repetition every 44 m instead of 11, all
//!   different within it, and drawn as the lawn is, at the cost of the
//!   memory of a bigger texture.
//! - [`GrassTiling::Stochastic`]: the pieces picked as each pixel is drawn
//!   (`lawn.wgsl`, after Heitz and Neyret's "High-Performance By-Example
//!   Noise using a Histogram-Preserving Blending Operator", 2018), never
//!   repeating, at the cost of three readings of the texture (and of its
//!   normal map) a pixel; without parallax.
//!
//! Each piece is blended round the lawn's average colour so the blend keeps
//! its contrast (the three pieces' differences from it, divided by the
//! square root of the sum of their weights squared), with the weights
//! sharpened ([`SHARP`]) so most of the ground is one piece or another, not
//! a haze of three.
//!
//! Every material drawn with the lawn has it (the ground, the banks, the
//! hedges): the wide lawn in place of the lawn in the material itself
//! ([`tile`]), the stochastic one in a material of its own
//! ([`LawnMaterial`]) in place of the material of each thing drawn with
//! it, given back when the tiling changes ([`Lawn`]).

use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageAddressMode,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat},
    shader::ShaderRef,
};

use super::relief::{self, Look, ReliefMap, ReliefShown};
use super::textures::{WorldTextures, generate_mipmaps, sampler};
use crate::settings::{GrassTiling, Settings};

/// How many times the lawn's width the wide lawn is, each way.
pub const WIDE: u32 = 4;
/// The side of the squares the wide lawn is cut into (px; each split into
/// two triangles): a quarter of the lawn, 2.75 m.
const PATCH: u32 = 128;
/// How many triangles of the stochastic lawn's grid there are across the
/// lawn: its sides about a third of it, 3.7 m (as Heitz and Neyret's).
const STOCHASTIC_CELLS: f32 = 3.464;
/// How sharply the pieces meet: the power their weights are raised to.
const SHARP: f32 = 4.0;

pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "lawn.wgsl");
        // Without the renderer (in the tests), the stochastic lawn is only
        // an asset.
        if app.is_plugin_added::<bevy::pbr::PbrPlugin>() {
            app.add_plugins(MaterialPlugin::<LawnMaterial>::default());
        } else {
            app.init_asset::<LawnMaterial>();
        }
        // The materials, before they are given their relief (of the lawn
        // they have); the things drawn with them, once their meshes have
        // their tangents.
        app.add_systems(
            Last,
            (
                tile_materials
                    .after(relief::show_relief)
                    .before(relief::relieve_materials),
                draw_stochastic.after(relief::give_tangents),
            )
                .after(bevy::asset::AssetEventSystems),
        );
    }
}

/// The lawn stochastic: the lawn's own material (but its texture and its
/// relief), its colour and its normal map read from the lawn three times
/// over.
pub type LawnMaterial = ExtendedMaterial<StandardMaterial, StochasticLawn>;

/// What the stochastic lawn reads (see `lawn.wgsl`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct StochasticLawn {
    #[uniform(100)]
    pub look: LawnLook,
    #[texture(101)]
    #[sampler(102)]
    pub texture: Handle<Image>,
    /// The lawn's normal map, if the relief has one.
    #[texture(103)]
    #[sampler(104)]
    pub normals: Option<Handle<Image>>,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct LawnLook {
    /// The lawn's average colour (linear), round which its pieces blend.
    pub mean: Vec4,
    /// 1 with the lawn's normal map, 0 without.
    pub normals: f32,
    /// How many triangles of the grid across the lawn.
    pub cells: f32,
    /// How sharply the pieces meet.
    pub sharp: f32,
}

impl MaterialExtension for StochasticLawn {
    fn fragment_shader() -> ShaderRef {
        "embedded://motorust/world/lawn.wgsl".into()
    }
}

/// On a thing drawn with the stochastic lawn: the material it had, given
/// back when the tiling changes.
#[derive(Component, Clone, Debug)]
pub struct Lawn(pub Handle<StandardMaterial>);

/// The lawn `lawn` (8-bit sRGB, square, its first level read) [`WIDE`]
/// times as wide each way, put together from pieces of it (see the
/// module's), with its mipmaps; repeating both ways as it does.
pub fn widen(lawn: &Image) -> Image {
    let size = lawn.width();
    assert_eq!(size, lawn.height(), "the lawn is square");
    let data = lawn
        .data
        .as_deref()
        .expect("the lawn has its pixels on the CPU");
    let level = &data[..(size * size * 4) as usize];
    let wide = size * WIDE;
    let mean = mean_of(level);
    let mut pixels = vec![0u8; (wide * wide * 4) as usize];
    let corners = Corners::new(size, wide / PATCH);
    let fill = |rows: &mut [u8], from: u32| {
        for (k, row) in rows.chunks_exact_mut((wide * 4) as usize).enumerate() {
            let y = from + k as u32;
            for x in 0..wide {
                let i = (x * 4) as usize;
                row[i..i + 3].copy_from_slice(&wide_pixel(level, size, &corners, mean, x, y));
                row[i + 3] = 255;
            }
        }
    };
    // In bands of rows, each on a thread of its own (but on the web).
    if cfg!(target_arch = "wasm32") {
        fill(&mut pixels, 0);
    } else {
        let band = wide / 4;
        std::thread::scope(|scope| {
            for (n, rows) in pixels.chunks_mut((band * wide * 4) as usize).enumerate() {
                let fill = &fill;
                scope.spawn(move || fill(rows, n as u32 * band));
            }
        });
    }
    let mut image = Image::new(
        Extent3d {
            width: wide,
            height: wide,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = sampler(ImageAddressMode::Repeat);
    generate_mipmaps(&mut image);
    image
}

/// The average of each colour channel of `pixels` (RGBA, 8 bits).
fn mean_of(pixels: &[u8]) -> [f32; 3] {
    let count = (pixels.len() / 4) as f32;
    let mut sum = [0.0f64; 3];
    for pixel in pixels.chunks_exact(4) {
        for (sum, value) in sum.iter_mut().zip(pixel) {
            *sum += *value as f64;
        }
    }
    sum.map(|sum| (sum / count as f64) as f32)
}

/// Where each corner of the wide lawn's squares reads the lawn from (a
/// shift, px), and which way its square is split: one of each, `cells` of
/// them each way, the last ones meeting the first as the wide lawn repeats.
struct Corners {
    cells: u32,
    shifts: Vec<(u32, u32)>,
    flipped: Vec<bool>,
}

impl Corners {
    fn new(size: u32, cells: u32) -> Self {
        let (mut shifts, mut flipped) = (Vec::new(), Vec::new());
        for j in 0..cells {
            for i in 0..cells {
                let [a, b] = pcg2d(i, j);
                shifts.push((a % size, b % size));
                flipped.push(pcg2d(i + cells, j)[0] & 1 == 1);
            }
        }
        Self {
            cells,
            shifts,
            flipped,
        }
    }

    fn index(&self, i: u32, j: u32) -> usize {
        ((j % self.cells) * self.cells + i % self.cells) as usize
    }
}

/// The colour (sRGB, 8 bits) of the wide lawn at (`x`, `y`): the pieces of
/// the lawn (`level`, `size` px each way) read by the corners of the
/// triangle it is in, blended round the lawn's `mean`.
fn wide_pixel(
    level: &[u8],
    size: u32,
    corners: &Corners,
    mean: [f32; 3],
    x: u32,
    y: u32,
) -> [u8; 3] {
    let (i, j) = (x / PATCH, y / PATCH);
    let fx = ((x % PATCH) as f32 + 0.5) / PATCH as f32;
    let fy = ((y % PATCH) as f32 + 0.5) / PATCH as f32;
    let flipped = corners.flipped[corners.index(i, j)];
    let weights = triangle(fx, fy, flipped).map(|((di, dj), weight)| {
        let (sx, sy) = corners.shifts[corners.index(i + di, j + dj)];
        let at = (((y + sy) % size) * size + (x + sx) % size) as usize * 4;
        (&level[at..at + 3], weight)
    });
    let sharpened = sharpen(weights.map(|(_, weight)| weight));
    let spread = sharpened.iter().map(|w| w * w).sum::<f32>().sqrt();
    let mut color = [0u8; 3];
    for (channel, value) in color.iter_mut().enumerate() {
        let offset: f32 = weights
            .iter()
            .zip(sharpened)
            .map(|((piece, _), w)| w * (piece[channel] as f32 - mean[channel]))
            .sum();
        *value = (mean[channel] + offset / spread).round().clamp(0.0, 255.0) as u8;
    }
    color
}

/// The corners (each a step right and down from the square's first, and
/// its weight) of the triangle at (`fx`, `fy`) of its square, split from
/// its first corner to its last or (`flipped`) from its second to its
/// third.
fn triangle(fx: f32, fy: f32, flipped: bool) -> [((u32, u32), f32); 3] {
    if !flipped {
        if fx > fy {
            [((0, 0), 1.0 - fx), ((1, 0), fx - fy), ((1, 1), fy)]
        } else {
            [((0, 0), 1.0 - fy), ((0, 1), fy - fx), ((1, 1), fx)]
        }
    } else {
        // The same, mirrored left to right.
        let gx = 1.0 - fx;
        if gx > fy {
            [((1, 0), 1.0 - gx), ((0, 0), gx - fy), ((0, 1), fy)]
        } else {
            [((1, 0), 1.0 - fy), ((1, 1), fy - gx), ((0, 1), gx)]
        }
    }
}

/// `weights` raised to [`SHARP`], adding up to 1 again.
fn sharpen(weights: [f32; 3]) -> [f32; 3] {
    let raised = weights.map(|w| w.max(0.0).powf(SHARP));
    let sum = raised.iter().sum::<f32>().max(1e-12);
    raised.map(|w| w / sum)
}

/// Two random numbers from two (Jarzynski and Olano's PCG 2D hash, which
/// `lawn.wgsl` has too).
fn pcg2d(x: u32, y: u32) -> [u32; 2] {
    const M: u32 = 1_664_525;
    const C: u32 = 1_013_904_223;
    let (mut x, mut y) = (
        x.wrapping_mul(M).wrapping_add(C),
        y.wrapping_mul(M).wrapping_add(C),
    );
    x = x.wrapping_add(y.wrapping_mul(M));
    y = y.wrapping_add(x.wrapping_mul(M));
    x ^= x >> 16;
    y ^= y >> 16;
    x = x.wrapping_add(y.wrapping_mul(M));
    y = y.wrapping_add(x.wrapping_mul(M));
    x ^= x >> 16;
    y ^= y >> 16;
    [x, y]
}

/// `material` as `tiling` says, if it is drawn with the lawn (`lawn`) or
/// the wide lawn (`wide`): with the wide one, its texture four times as
/// wide, stretched over four times as much; otherwise with the lawn (the
/// stochastic lawn's own material is another, made from this one).
/// Whether that changed it.
pub fn tile(
    material: &mut StandardMaterial,
    lawn: &Handle<Image>,
    wide: &Handle<Image>,
    tiling: GrassTiling,
) -> bool {
    let Some(texture) = material.base_color_texture.as_ref() else {
        return false;
    };
    let widened = texture == wide;
    if !widened && texture != lawn {
        return false;
    }
    let widen = tiling == GrassTiling::Wide;
    if widen == widened {
        return false;
    }
    let (texture, stretch) = if widen {
        (wide, 1.0 / WIDE as f32)
    } else {
        (lawn, WIDE as f32)
    };
    material.base_color_texture = Some(texture.clone());
    material.uv_transform =
        bevy::math::Affine2::from_scale(Vec2::splat(stretch)) * material.uv_transform;
    true
}

/// The lawn's materials as the tiling says: all of them when it changes,
/// otherwise each one as it is made; with the relief of the lawn they have
/// (which the relief's own system gives only to those made this frame).
fn tile_materials(
    settings: Option<Res<Settings>>,
    shown: Res<ReliefShown>,
    textures: Res<WorldTextures>,
    mut events: MessageReader<AssetEvent<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut given: Local<Option<GrassTiling>>,
) {
    let tiling = settings.map_or_else(GrassTiling::default, |settings| settings.grass_tiling);
    let ids: Vec<AssetId<StandardMaterial>> = if *given != Some(tiling) {
        *given = Some(tiling);
        events.clear();
        materials.ids().collect()
    } else {
        events
            .read()
            .filter_map(|event| match event {
                AssetEvent::Added { id } => Some(*id),
                _ => None,
            })
            .collect()
    };
    for id in ids {
        let Some(mut material) = materials.get(id).cloned() else {
            continue;
        };
        if !tile(&mut material, &textures.grass, &textures.grass_wide, tiling) {
            continue;
        }
        let texture = material
            .base_color_texture
            .clone()
            .expect("it has the lawn");
        if let Some(map) = textures.relief.of(&texture) {
            relief::relieve(&mut material, map, shown.look(false));
        }
        // Only those it changes are touched, so the others are not sent to
        // the GPU again.
        if let Some(mut slot) = materials.get_mut(id) {
            *slot = material;
        }
    }
}

/// The stochastic lawn's material for things drawn with `material` (the
/// lawn's), with the lawn's `map` if it shows its relief (`relieved`).
fn stochastic(
    material: &StandardMaterial,
    textures: &WorldTextures,
    map: Option<&ReliefMap>,
    relieved: bool,
) -> LawnMaterial {
    // The lawn read by its shader, not by the material's own.
    let base = StandardMaterial {
        base_color_texture: None,
        normal_map_texture: None,
        depth_map: None,
        ..material.clone()
    };
    ExtendedMaterial {
        base,
        extension: StochasticLawn {
            look: look(textures, map.is_some() && relieved),
            texture: textures.grass.clone(),
            normals: map.map(|map| map.normal.clone()),
        },
    }
}

/// What the stochastic lawn's shader reads, with the normal map or without.
fn look(textures: &WorldTextures, normals: bool) -> LawnLook {
    let mean = textures.grass_color.to_linear();
    LawnLook {
        mean: Vec4::new(mean.red, mean.green, mean.blue, 1.0),
        normals: if normals { 1.0 } else { 0.0 },
        cells: STOCHASTIC_CELLS,
        sharp: SHARP,
    }
}

/// The things drawn with the lawn drawn with the stochastic one when the
/// tiling says so (those put in the world this frame, or all of them when
/// it changes to it), and given their own material back when it changes
/// from it; the stochastic lawn's materials with the relief shown (its
/// normal map, or none: dropped at speed, or none chosen).
#[allow(clippy::too_many_arguments)]
fn draw_stochastic(
    mut commands: Commands,
    settings: Option<Res<Settings>>,
    shown: Res<ReliefShown>,
    textures: Res<WorldTextures>,
    standard: Res<Assets<StandardMaterial>>,
    mut lawns: ResMut<Assets<LawnMaterial>>,
    new: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Changed<MeshMaterial3d<StandardMaterial>>,
    >,
    all: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    drawn: Query<(Entity, &Lawn)>,
    mut given: Local<Option<(GrassTiling, ReliefShown)>>,
    mut made: Local<HashMap<AssetId<StandardMaterial>, Handle<LawnMaterial>>>,
) {
    let tiling = settings.map_or_else(GrassTiling::default, |settings| settings.grass_tiling);
    let changed = *given != Some((tiling, *shown));
    let was = given.map(|(tiling, _)| tiling);
    *given = Some((tiling, *shown));
    // Parallax is not drawn on the stochastic lawn: its normal map only.
    let relieved = matches!(shown.look(false), Look::Normal | Look::Parallax);
    let map = textures.relief.of(&textures.grass);
    if tiling != GrassTiling::Stochastic {
        made.clear();
        if was == Some(GrassTiling::Stochastic) {
            for (entity, Lawn(material)) in &drawn {
                commands
                    .entity(entity)
                    .remove::<(MeshMaterial3d<LawnMaterial>, Lawn)>()
                    .insert(MeshMaterial3d(material.clone()));
            }
        }
        return;
    }
    if changed {
        // The relief may have changed: the lawn's materials have the
        // normal map or not.
        let normals = map.is_some() && relieved;
        for (_, lawn) in lawns.iter_mut() {
            if (lawn.extension.look.normals > 0.5) != normals {
                lawn.extension.look = look(&textures, normals);
            }
        }
    }
    // Those of materials gone are forgotten.
    made.retain(|id, _| standard.contains(*id));
    let candidates: Vec<(Entity, Handle<StandardMaterial>)> =
        if was != Some(GrassTiling::Stochastic) {
            all.iter()
                .map(|(entity, material)| (entity, material.0.clone()))
                .collect()
        } else {
            new.iter()
                .map(|(entity, material)| (entity, material.0.clone()))
                .collect()
        };
    for (entity, handle) in candidates {
        let Some(material) = standard.get(&handle) else {
            continue;
        };
        if material.base_color_texture.as_ref() != Some(&textures.grass) {
            continue;
        }
        let lawn = made
            .entry(handle.id())
            .or_insert_with(|| lawns.add(stochastic(material, &textures, map, relieved)))
            .clone();
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(lawn), Lawn(handle)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lawn of `size` px each way: grey stripes across, a light spot.
    fn lawn(size: u32) -> Image {
        let mut data = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let stripe = if (y / 4) % 2 == 0 { 60 } else { 140 };
                let spot = if x < size / 8 && y < size / 8 { 100 } else { 0 };
                let value = (stripe + spot) as u8;
                data.extend_from_slice(&[value, value / 2, value / 3, 255]);
            }
        }
        Image::new(
            Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }

    #[test]
    fn the_triangles_weights_add_up_and_meet_at_the_squares_edges() {
        for flipped in [false, true] {
            for (fx, fy) in [(0.1, 0.7), (0.8, 0.3), (0.5, 0.5), (0.0, 0.0), (0.99, 0.01)] {
                let corners = triangle(fx, fy, flipped);
                let sum: f32 = corners.iter().map(|(_, w)| w).sum();
                assert!((sum - 1.0).abs() < 1e-6, "{sum}");
                assert!(corners.iter().all(|(_, w)| *w >= -1e-6));
                // Where it is: its corners, weighted.
                let at = corners.iter().fold(Vec2::ZERO, |at, ((i, j), w)| {
                    at + Vec2::new(*i as f32, *j as f32) * *w
                });
                assert!(at.distance(Vec2::new(fx, fy)) < 1e-5, "{at}");
            }
            // On the left and top edges of its square only those edges'
            // corners count, as they do from the squares beside it.
            for ((i, _), w) in triangle(0.0, 0.4, flipped) {
                assert!(i == 0 || w.abs() < 1e-6);
            }
            for ((_, j), w) in triangle(0.6, 0.0, flipped) {
                assert!(j == 0 || w.abs() < 1e-6);
            }
        }
    }

    #[test]
    fn sharpened_weights_add_up_and_favour_the_heaviest() {
        let sharpened = sharpen([0.5, 0.3, 0.2]);
        assert!((sharpened.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(sharpened[0] > 0.8 && sharpened[2] < 0.05, "{sharpened:?}");
        assert_eq!(sharpen([1.0, 0.0, 0.0]), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn the_wide_lawn_repeats_without_seams() {
        let size = 64;
        let level = lawn(size).data.unwrap();
        let wide = size * WIDE;
        let corners = Corners::new(size, wide / PATCH);
        let mean = mean_of(&level);
        let at = |x, y| wide_pixel(&level, size, &corners, mean, x, y);
        for k in [0, 5, 31, 200] {
            for y in [0, 17, 63, 130] {
                assert_eq!(at(wide + k, y), at(k, y));
                assert_eq!(at(y, wide + k), at(y, k));
            }
        }
    }

    #[test]
    fn the_wide_lawn_is_the_lawn_in_pieces_with_its_colours() {
        let size = 256;
        let image = lawn(size);
        let wide = widen(&image);
        assert_eq!((wide.width(), wide.height()), (size * WIDE, size * WIDE));
        assert!(wide.texture_descriptor.mip_level_count > 1);
        let level = image.data.unwrap();
        let data = wide.data.as_deref().unwrap();
        let pixels = (size * WIDE * size * WIDE) as usize;
        let (from, to) = (mean_of(&level), mean_of(&data[..pixels * 4]));
        for (from, to) in from.iter().zip(to) {
            assert!((from - to).abs() < 3.0, "{from} to {to}");
        }
        // Its stripes are still there, as light and as dark: the blend
        // keeps the contrast.
        let red: Vec<u8> = data[..pixels * 4].chunks_exact(4).map(|p| p[0]).collect();
        let share =
            |of: &dyn Fn(u8) -> bool| red.iter().filter(|&&r| of(r)).count() as f32 / pixels as f32;
        let (dark, light) = (share(&|r| r < 80), share(&|r| r > 120));
        assert!(dark > 0.3 && light > 0.3, "dark {dark}, light {light}");
        // And it is not the lawn over again: most of it is another piece.
        let wide_side = size * WIDE;
        let moved = (0..pixels)
            .filter(|&n| {
                let (x, y) = (n as u32 % wide_side, n as u32 / wide_side);
                let plain = ((y % size) * size + x % size) as usize * 4;
                red[n].abs_diff(level[plain]) > 10
            })
            .count();
        assert!(moved as f32 > 0.3 * pixels as f32, "{moved} of {pixels}");
    }

    #[test]
    fn the_lawn_s_materials_take_the_wide_lawn_and_give_it_back() {
        let mut images = Assets::<Image>::default();
        let (lawn, wide) = (images.add(Image::default()), images.add(Image::default()));
        let mut material = StandardMaterial {
            base_color_texture: Some(lawn.clone()),
            ..default()
        };
        assert!(!tile(&mut material, &lawn, &wide, GrassTiling::Plain));
        assert!(!tile(&mut material, &lawn, &wide, GrassTiling::Stochastic));
        assert!(tile(&mut material, &lawn, &wide, GrassTiling::Wide));
        assert_eq!(material.base_color_texture, Some(wide.clone()));
        let corner = material.uv_transform.transform_point2(Vec2::ONE);
        assert_eq!(corner, Vec2::splat(1.0 / WIDE as f32));
        assert!(!tile(&mut material, &lawn, &wide, GrassTiling::Wide));
        assert!(tile(&mut material, &lawn, &wide, GrassTiling::Stochastic));
        assert_eq!(material.base_color_texture, Some(lawn.clone()));
        assert_eq!(material.uv_transform, bevy::math::Affine2::IDENTITY);
        // Another texture is left as it is.
        let rock = images.add(Image::default());
        let mut other = StandardMaterial {
            base_color_texture: Some(rock),
            ..default()
        };
        assert!(!tile(&mut other, &lawn, &wide, GrassTiling::Wide));
        assert!(!tile(
            &mut StandardMaterial::default(),
            &lawn,
            &wide,
            GrassTiling::Wide
        ));
    }
}
