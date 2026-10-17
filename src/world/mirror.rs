//! Water that mirrors what stands round it (`mirror.wgsl`): still and dark,
//! the hills, the roads, the dams and the sky in it, more the lower one
//! looks across it, stirred by a faint ripple near the camera, as a
//! reservoir's. A dam's lake is drawn so (see [`super::dam`]), and so is
//! the sea, unless Options > Graphics > Water says otherwise (or the track
//! is Loop Island, whose sea is always the cartoon one, see
//! [`super::sea`]).
//!
//! What the water mirrors is mapped from above with the rest of the circuit,
//! away from the main thread ([`prepare_mirror`]): at each point, the colour
//! of what stands there, lit, and how high it stands over the water; and the
//! sky's gradient each way. The shader follows the reflected line of sight
//! over that map until it meets something, or else takes the sky's colour
//! that way.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

use super::dam::edge;
use super::scenery::{Scenery, Soil, Water};
use super::sky::{Palette, direction};
use super::structures::indices;
use super::terrain::{Ground, Terrain, on_cores};
use super::textures::WorldTextures;
use crate::track::{CURB_WIDTH, Track};

/// The sky it mirrors, mapped round the compass and from the horizon up.
const SKY_MAP: (u32, u32) = (64, 16);
/// The water's own colour, deep down (sRGB): a reservoir's dark green-blue,
/// the sea's too; and the wine red of a flooded pit rich in iron.
const DEEP: Color = Color::srgb(0.06, 0.17, 0.22);
const DEEP_IRON: Color = Color::srgb(0.24, 0.06, 0.04);
/// How bright the sun's glint on the water is.
const GLINT: f32 = 4.0;
/// The ground as it is lit for the mirror: its colour times this much in
/// the shade, and this much more facing the sun.
const MIRROR_SHADE: f32 = 0.55;
const MIRROR_SUN: f32 = 0.75;
/// The colours, before they are lit, of what the textures make the rock,
/// the sand, the checkerboard earth, the asphalt and the concrete look on
/// average (linear).
const ROCK_AVERAGE: [f32; 3] = [0.3, 0.27, 0.23];
const SAND_AVERAGE: [f32; 3] = [0.55, 0.48, 0.36];
const SOIL_AVERAGE: [f32; 3] = [0.3, 0.18, 0.09];
const ASPHALT_AVERAGE: [f32; 3] = [0.08, 0.08, 0.085];
const CONCRETE_AVERAGE: [f32; 3] = [0.62, 0.6, 0.55];
/// How far round the centre line of a dam's crest it may stand over the
/// water (m): past its deck and its brackets.
const CREST_BOX: f32 = 25.0;

/// Water that mirrors what stands round it (see `mirror.wgsl`). Unlit, as
/// what it mirrors is lit already.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct MirrorMaterial {
    /// xy: corner of the map of what stands round the water (m, x and z);
    /// zw: its size (m).
    #[uniform(0)]
    pub area: Vec4,
    /// x: the water's level (m); y: how high over it the map's heights
    /// reach at most (m); z: how far a reflected line of sight is followed
    /// (m).
    #[uniform(1)]
    pub water: Vec4,
    /// What stands round the water: its colour as lit, and how high it
    /// stands over the water (see [`surroundings_map`]).
    #[texture(2)]
    #[sampler(3)]
    pub surroundings: Handle<Image>,
    /// The sky's colour each way (see [`sky_map`]).
    #[texture(4)]
    #[sampler(5)]
    pub sky: Handle<Image>,
    #[uniform(6)]
    pub look: MirrorLook,
}

/// The water's own colour and the sun's glint on it (see `mirror.wgsl`).
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct MirrorLook {
    /// The water's colour, deep down (linear).
    pub deep: Vec4,
    /// xyz: towards the sun; w: how bright its glint is.
    pub sun: Vec4,
}

impl Material for MirrorMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://motorust/world/mirror.wgsl".into()
    }

    // Nothing to draw before the colour or into the shadows: the water is
    // flat and unlit.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

/// What is mapped round some water for it to mirror: the box (m, x and z),
/// a texel every `texel` m, the water's level (m) and how far a reflected
/// line of sight is followed over it (m).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reach {
    pub min: Vec2,
    pub max: Vec2,
    pub texel: f32,
    pub level: f32,
    pub follow: f32,
}

/// Mirroring water worked out away from the main thread, to be drawn with a
/// [`MirrorMaterial`] once its maps are images.
pub struct PreparedMirror {
    area: Vec4,
    water: Vec4,
    look: MirrorLook,
    surroundings: Image,
    sky: Image,
}

/// A point over the surface of something standing round the water (a
/// building, a torii), for the water to mirror it (see
/// [`PreparedMirror::stand_up`]): where it is, which way its face looks and
/// its colour there, before it is lit (linear).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MirrorPoint {
    pub at: Vec3,
    pub normal: Vec3,
    pub color: [f32; 3],
}

impl MirrorPoint {
    /// This point of something drawn with `transform`.
    pub fn placed(self, transform: &Transform) -> Self {
        Self {
            at: transform.transform_point(self.at),
            normal: (transform.rotation * self.normal).normalize_or_zero(),
            ..self
        }
    }
}

impl PreparedMirror {
    /// What stands at `points` mirrored too: each texel they reach over
    /// what the map has there stands as high as the highest of them, in
    /// their colour on average, lit by the sun as the ground is. The map
    /// is of the ground alone, worked out with the circuit; what stands on
    /// it is put in the world later, and given here.
    pub fn stand_up(&mut self, points: &[MirrorPoint]) {
        let (width, height) = (
            self.surroundings.width() as usize,
            self.surroundings.height() as usize,
        );
        let (corner, size) = (self.area.xy(), self.area.zw());
        let texel = size / Vec2::new(width as f32, height as f32);
        let sun = self.look.sun.truncate();
        let level = self.water.x;
        // Each texel reached: the highest point over it, and the colours.
        let mut reached: std::collections::HashMap<usize, (f32, Vec3, f32)> =
            std::collections::HashMap::new();
        for point in points {
            let at = (point.at.xz() - corner) / texel;
            if at.x < 0.0 || at.y < 0.0 || at.x >= width as f32 || at.y >= height as f32 {
                continue;
            }
            let index = at.y as usize * width + at.x as usize;
            let light = MIRROR_SHADE + MIRROR_SUN * point.normal.dot(sun).max(0.0);
            let entry = reached.entry(index).or_insert((f32::MIN, Vec3::ZERO, 0.0));
            entry.0 = entry.0.max(point.at.y - level);
            entry.1 += Vec3::from(point.color) * light;
            entry.2 += 1.0;
        }
        let Some(data) = self.surroundings.data.as_mut() else {
            return;
        };
        for (index, (over, color, count)) in reached {
            let at = index * 8;
            let mapped = from_half(u16::from_le_bytes([data[at + 6], data[at + 7]]));
            if over <= mapped {
                continue;
            }
            let color = color / count;
            for (k, value) in [color.x, color.y, color.z, over].into_iter().enumerate() {
                data[at + 2 * k..at + 2 * k + 2].copy_from_slice(&half_float(value).to_le_bytes());
            }
            self.water.y = self.water.y.max(over);
        }
    }

    /// Its material, its maps added to `images`.
    pub fn into_material(self, images: &mut Assets<Image>) -> MirrorMaterial {
        MirrorMaterial {
            area: self.area,
            water: self.water,
            surroundings: images.add(self.surroundings),
            sky: images.add(self.sky),
            look: self.look,
        }
    }
}

/// The water's own colour, deep down (sRGB), for `water`.
pub fn deep(water: Water) -> Color {
    match water {
        Water::Sea => DEEP,
        Water::Iron => DEEP_IRON,
    }
}

/// Water at `reach`'s level that mirrors what stands round it on `track`
/// with `scenery` (the map worked out on `threads` threads, leaving out
/// where `unseen` says no line of sight from the water ever gets), deep down
/// `deep` (sRGB).
#[allow(clippy::too_many_arguments)]
pub fn prepare_mirror(
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    textures: &WorldTextures,
    reach: &Reach,
    unseen: &(dyn Fn(Vec2) -> bool + Sync),
    deep: Color,
    threads: usize,
) -> PreparedMirror {
    let palette = Palette::of(scenery.sky);
    let sun = palette.sun_direction();
    let grass_average = textures.grass_color.to_linear();
    let (corner, size, highest, surroundings) = surroundings_map(
        track,
        terrain,
        scenery,
        reach,
        unseen,
        sun,
        grass_average,
        threads,
    );
    PreparedMirror {
        area: corner.extend(size.x).extend(size.y),
        water: Vec4::new(reach.level, highest, reach.follow, 0.0),
        look: MirrorLook {
            deep: deep.to_linear().to_vec4(),
            sun: sun.extend(GLINT),
        },
        surroundings,
        sky: sky_map(&palette),
    }
}

/// What stands round some water within `reach`, mapped from above for it to
/// mirror: the map's corner and size (m), how high over the water its
/// heights reach at most (m), and an image of, at each point, the colour it
/// shows lit by the sun towards `sun` (rgb, linear) and how high it stands
/// over the water (alpha, m; under it, less than 0), in half floats, fine
/// enough for the low, grazing lines of sight across the water. The ground,
/// by what it is (its grass's greens over the grass's `grass_average`
/// colour, rock, sand, snow); the roads' asphalt where they lie on it; and
/// the dams' crests, standing up from the water to the road. What is under
/// the water, and wherever `unseen` says no line of sight from the water
/// ever gets, is down at it, with no colour. Worked out on `threads`
/// threads.
#[allow(clippy::too_many_arguments)]
pub fn surroundings_map(
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    reach: &Reach,
    unseen: &(dyn Fn(Vec2) -> bool + Sync),
    sun: Vec3,
    grass_average: LinearRgba,
    threads: usize,
) -> (Vec2, Vec2, f32, Image) {
    let (width, height) = (
        ((reach.max.x - reach.min.x) / reach.texel).ceil().max(1.0) as u32,
        ((reach.max.y - reach.min.y) / reach.texel).ceil().max(1.0) as u32,
    );
    let lap = track.lap_length();
    let crests: Vec<usize> = scenery
        .dams
        .iter()
        .flat_map(|dam| indices(track, dam.crest, lap))
        .collect();
    // Where the crests stand: the only places under the water where
    // something stands up out of it.
    let crest_boxes: Vec<(Vec2, Vec2)> = scenery
        .dams
        .iter()
        .map(|dam| {
            indices(track, dam.crest, lap)
                .into_iter()
                .map(|i| track.centerline()[i])
                .fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(min, max), p| (min.min(p - CREST_BOX), max.max(p + CREST_BOX)),
                )
        })
        .collect();
    let by_a_crest = |at: Vec2| {
        crest_boxes
            .iter()
            .any(|(min, max)| at.cmpge(*min).all() && at.cmple(*max).all())
    };
    let rows: Vec<u32> = (0..height).collect();
    let texels = on_cores(&rows, threads, |rows| {
        let mut texels = Vec::with_capacity(rows.len() * width as usize);
        for &j in rows {
            for i in 0..width {
                let at = reach.min + (Vec2::new(i as f32, j as f32) + 0.5) * reach.texel;
                // Under the water nothing is ever met, nor its colour seen:
                // only its height is needed (but by a dam's crest).
                texels.push(if unseen(at) {
                    ([0.0; 3], reach.level)
                } else {
                    let ground = terrain.height(at);
                    if ground < reach.level && !by_a_crest(at) {
                        ([0.0; 3], ground)
                    } else {
                        texel(track, terrain, &crests, at, sun, grass_average)
                    }
                });
            }
        }
        texels
    });
    let mut highest = 0.0f32;
    let mut data = Vec::with_capacity(texels.len() * 8);
    for (color, top) in texels {
        let over = top - reach.level;
        highest = highest.max(over);
        for value in [color[0], color[1], color[2], over] {
            data.extend(half_float(value).to_le_bytes());
        }
    }
    let size = Vec2::new(width as f32, height as f32) * reach.texel;
    (
        reach.min,
        size,
        highest,
        map_image(
            (width, height),
            data,
            TextureFormat::Rgba16Float,
            ImageAddressMode::ClampToEdge,
        ),
    )
}

/// What stands at `at`, lit by the sun towards `sun`: its colour (linear)
/// and the height of its top (m). The crests of the dams are the points of
/// the lap `crests`.
fn texel(
    track: &Track,
    terrain: &Terrain,
    crests: &[usize],
    at: Vec2,
    sun: Vec3,
    grass_average: LinearRgba,
) -> ([f32; 3], f32) {
    let ground = terrain.height(at);
    let normal = terrain.normal(at);
    let p = Vec3::new(at.x, ground, at.y);
    let nearest = track.nearest(at);
    let road = track.height_beside(&nearest);
    let on_crest = nearest.road == 0 && crests.contains(&nearest.index);
    let (top, color) = if on_crest && nearest.distance < edge(track, nearest.index) {
        (road, CONCRETE_AVERAGE)
    } else if nearest.distance < track.half_width_beside(&nearest) + CURB_WIDTH
        && (road - ground).abs() < 1.0
    {
        (ground, ASPHALT_AVERAGE)
    } else {
        let kind = terrain.ground_drawn(at);
        let tint = terrain.ground_color(kind, p, normal);
        // The texture each ground is drawn with (see
        // `terrain::spawn_terrain`): a desert canyon's flat ground is dusty
        // earth, not grass.
        let texture = match (kind, terrain.soil()) {
            (Ground::Grass, Soil::Mine) | (Ground::Sand, _) => SAND_AVERAGE,
            (Ground::Grass, _) => [grass_average.red, grass_average.green, grass_average.blue],
            (Ground::Rock, Soil::Checkered) => SOIL_AVERAGE,
            (Ground::Rock, _) => ROCK_AVERAGE,
            (Ground::Snow, _) => [0.9, 0.92, 0.95],
        };
        (ground, [0, 1, 2].map(|k| tint[k] * texture[k]))
    };
    let light = MIRROR_SHADE + MIRROR_SUN * normal.dot(sun).max(0.0);
    (color.map(|c| c * light), top)
}

/// The bits of `value` as a half float (IEEE 754 binary16), rounded to the
/// nearest; those too small for one, 0, and those too big, infinite.
pub(super) fn half_float(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = ((bits >> 13) & 0x3ff) as u16;
    let round = ((bits >> 12) & 1) as u16;
    match exponent {
        ..=0 => sign,
        31.. => sign | 0x7c00,
        // Rounding up past the mantissa's top carries into the exponent.
        _ => (sign | ((exponent as u16) << 10) | mantissa) + round,
    }
}

/// The value of half-float bits (see [`half_float`]).
pub(super) fn from_half(bits: u16) -> f32 {
    let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = ((bits >> 10) & 0x1f) as i32;
    let mantissa = (bits & 0x3ff) as f32 / 1024.0;
    match exponent {
        0 => 0.0,
        31 => sign * f32::INFINITY,
        _ => sign * (1.0 + mantissa) * 2f32.powi(exponent - 15),
    }
}

/// The sky's colour each way, as it is drawn (see [`Palette`]): across,
/// round the compass from the north (-Z) towards the east (+X); down, from
/// overhead to the horizon.
fn sky_map(palette: &Palette) -> Image {
    let (width, height) = SKY_MAP;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for j in 0..height {
        for i in 0..width {
            let azimuth = (i as f32 + 0.5) / width as f32 * std::f32::consts::TAU;
            let elevation = (1.0 - (j as f32 + 0.5) / height as f32) * std::f32::consts::FRAC_PI_2;
            let color = palette.sky_color(direction(azimuth, elevation)).to_srgba();
            data.extend([
                to_byte(color.red),
                to_byte(color.green),
                to_byte(color.blue),
                255,
            ]);
        }
    }
    map_image(
        (width, height),
        data,
        TextureFormat::Rgba8UnormSrgb,
        ImageAddressMode::Repeat,
    )
}

fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// An image of `data` in `format` for a shader to look things up in,
/// smoothly, wrapping across (`across`) and clamped down.
fn map_image(
    (width, height): (u32, u32),
    data: Vec<u8>,
    format: TextureFormat,
    across: ImageAddressMode,
) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: across,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// What a map holds at `p` (m): its colour and how high it stands over
    /// the water.
    pub(in crate::world) fn mapped(image: &Image, corner: Vec2, texel: f32, p: Vec2) -> [f32; 4] {
        let width = image.width() as usize;
        let data = image.data.as_ref().expect("the map's texels");
        let (i, j) = (
            ((p.x - corner.x) / texel) as usize,
            ((p.y - corner.y) / texel) as usize,
        );
        let at = (j * width + i) * 8;
        [0, 1, 2, 3]
            .map(|c| from_half(u16::from_le_bytes([data[at + 2 * c], data[at + 2 * c + 1]])))
    }

    #[test]
    fn half_floats_keep_the_heights_round_the_water_to_a_few_centimetres() {
        assert_eq!(half_float(1.0), 0x3c00);
        assert_eq!(half_float(0.5), 0x3800);
        assert_eq!(half_float(-2.0), 0xc000);
        assert_eq!(half_float(1e-6), 0);
        assert_eq!(half_float(1e6), 0x7c00);
        for value in [0.37f32, 3.2, 27.9, -14.6, 85.3] {
            assert!(
                (from_half(half_float(value)) - value).abs() < 0.04,
                "{value}"
            );
        }
    }

    #[test]
    fn a_desert_canyon_mirrors_its_red_earth_not_grass() {
        use crate::circuits::CIRCUITS;
        use crate::world::sea::{MIRROR_INLAND, ShoreMap, mirror_reach};
        let circuit = CIRCUITS
            .iter()
            .find(|circuit| circuit.scenery.soil == Soil::Mine)
            .expect("a desert canyon");
        let track = Track::new(&circuit.layout);
        let terrain = Terrain::new(&track, &circuit.scenery);
        let shore = ShoreMap::new(&terrain, &track);
        let reach = mirror_reach(&track);
        let unseen = |at| shore.distance_at(at) < -MIRROR_INLAND;
        let grass = Color::srgb(0.25, 0.5, 0.12).to_linear();
        let (corner, _, highest, image) = surroundings_map(
            &track,
            &terrain,
            &circuit.scenery,
            &reach,
            &unseen,
            Vec3::Y,
            grass,
            4,
        );
        assert!(highest > 10.0, "{highest}");
        // The land round the water, on average: red earth and rock.
        let (mut sum, mut count) = (Vec3::ZERO, 0);
        let step = reach.texel * 4.0;
        let mut at = corner + step / 2.0;
        while at.y < reach.max.y {
            at.x = corner.x + step / 2.0;
            while at.x < reach.max.x {
                let [r, g, b, over] = mapped(&image, corner, reach.texel, at);
                if over > 1.0 && shore.distance_at(at) > -200.0 {
                    sum += Vec3::new(r, g, b);
                    count += 1;
                }
                at.x += step;
            }
            at.y += step;
        }
        assert!(count > 100, "{count}");
        let average = sum / count as f32;
        assert!(
            average.x > 1.2 * average.y && average.y > average.z,
            "{average:?}"
        );
    }

    #[test]
    fn what_stands_by_the_water_is_mirrored_as_high_as_it_stands() {
        // A map 10 texels a side, 2 m each, of ground 1 m over the water
        // but for a hill 20 m over it in one texel.
        let mut data = Vec::new();
        for k in 0..100 {
            let over = if k == 55 { 20.0 } else { 1.0 };
            for value in [0.1, 0.2, 0.1, over] {
                data.extend(half_float(value).to_le_bytes());
            }
        }
        let mut mirror = PreparedMirror {
            area: Vec4::new(0.0, 0.0, 20.0, 20.0),
            water: Vec4::new(-1.0, 20.0, 500.0, 0.0),
            look: MirrorLook {
                deep: Vec4::ONE,
                sun: Vec4::new(0.0, 1.0, 0.0, 1.0),
            },
            surroundings: map_image(
                (10, 10),
                data,
                TextureFormat::Rgba16Float,
                ImageAddressMode::ClampToEdge,
            ),
            sky: Image::default(),
        };
        let red = [0.8, 0.1, 0.05];
        let point = |x: f32, y: f32, z: f32| MirrorPoint {
            at: Vec3::new(x, y, z),
            normal: Vec3::Y,
            color: red,
        };
        // A red shrine 9 m tall over two texels, and one low over the hill.
        mirror.stand_up(&[
            point(3.0, 8.0, 3.0),
            point(5.0, 2.0, 3.0),
            point(3.5, 2.0, 3.5),
            point(11.0, 5.0, 11.0),
        ]);
        let corner = Vec2::ZERO;
        let at = |x: f32, z: f32| mapped(&mirror.surroundings, corner, 2.0, Vec2::new(x, z));
        let shrine = at(3.0, 3.0);
        assert!((shrine[3] - 9.0).abs() < 0.05, "{shrine:?}");
        assert!(shrine[0] > 4.0 * shrine[1], "{shrine:?}");
        assert!((at(5.0, 3.0)[3] - 3.0).abs() < 0.05);
        // The hill stays as high as it is; the ground elsewhere, as it was.
        assert!((at(11.0, 11.0)[3] - 20.0).abs() < 0.05);
        assert!((at(15.0, 15.0)[3] - 1.0).abs() < 0.05);
        assert!(mirror.water.y >= 20.0);
    }

    #[test]
    fn the_sea_is_as_dark_as_the_lake_and_a_pit_s_water_red() {
        assert_eq!(deep(Water::Sea), DEEP);
        let iron = deep(Water::Iron).to_srgba();
        assert!(iron.red > 2.0 * iron.green && iron.red > 2.0 * iron.blue);
    }
}
