//! The sea, drawn like the cartoon sea of Zelda: The Wind Waker by its own
//! shader (`sea.wgsl`): flat saturated blues with no lighting, white crests
//! rolling in towards the coast (few and short out at sea, more, longer and
//! thicker near it, where they break), lighter water by the coast, and foam
//! hugging the shore.
//!
//! The shader finds the shore in a map of each circuit: how far each point
//! is from it ([`ShoreMap`]), worked out from the terrain along with the rest
//! of the circuit, away from the main thread (see [`super::prepare`]). The
//! crests run along the lines at the same distance from the shore, so they
//! follow the coast and come in square to it.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, PrimitiveTopology, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

use super::scenery::Water;
use super::terrain::{SEA_LEVEL, Terrain};
use crate::track::Track;

/// How far the sea reaches from the centre of the circuit (m): past the fog.
const SEA_EXTENT: f32 = 7000.0;
/// The shore map covers the track and this much around it (m)...
const SHORE_MARGIN: f32 = 700.0;
/// ...with at most this many texels along its longest side, and texels no
/// smaller than this (m).
const SHORE_TEXELS: f32 = 1024.0;
const SHORE_TEXEL_MIN: f32 = 1.5;
/// What the shore map holds: in red, from SHORE_NEAR (inland) to SHORE_FAR
/// (out at sea) metres from the shore, for the foam; in green, up to
/// WAVES_REACH metres out, for the waves rolling in. Must match `sea.wgsl`.
const SHORE_NEAR: f32 = -10.0;
const SHORE_FAR: f32 = 40.0;
const WAVES_REACH: f32 = 400.0;
/// The swell comes in to the coast near the start of the race: the shore
/// within this distance of it (m), seen from the sea up to this far off it
/// (m).
const SWELL_AROUND_START: f32 = 500.0;
const SWELL_WATER: f32 = 150.0;

/// How the sea looks (see `sea.wgsl`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SeaMaterial {
    /// xy: corner of the shore map (m, x and z); zw: its size (m).
    #[uniform(0)]
    pub shore_area: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub shore: Handle<Image>,
    /// xy: where the swell heads (m, x and z, length 1).
    #[uniform(3)]
    pub swell: Vec4,
    #[uniform(4)]
    pub colors: SeaColors,
}

/// The colours of the water (sRGB, see `sea.wgsl`): far out, at sea, by the
/// shore, and of the foam.
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct SeaColors {
    pub deep: Vec4,
    pub open: Vec4,
    pub shallow: Vec4,
    pub foam: Vec4,
}

impl SeaColors {
    pub fn of(water: Water) -> Self {
        let rgb = |r: f32, g: f32, b: f32| Vec4::new(r, g, b, 1.0);
        match water {
            // Saturated blues and white foam.
            Water::Sea => Self {
                deep: rgb(0.07, 0.29, 0.78),
                open: rgb(0.12, 0.42, 0.88),
                shallow: rgb(0.25, 0.72, 0.93),
                foam: rgb(0.97, 0.99, 1.0),
            },
            // Wine red where it is deep, rust and orange ochre by the
            // shore, and a pale yellow scum.
            Water::Iron => Self {
                deep: rgb(0.36, 0.07, 0.05),
                open: rgb(0.52, 0.13, 0.06),
                shallow: rgb(0.82, 0.38, 0.12),
                foam: rgb(0.96, 0.86, 0.62),
            },
        }
    }
}

impl Material for SeaMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://motorust/world/sea.wgsl".into()
    }

    // Nothing to draw before the colour or into the shadows: the sea is
    // flat, unlit and casts no shadow.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

/// How far each point around a circuit is from the shore (m), positive out
/// at sea and negative inland, on a grid over the circuit.
#[derive(Clone, Debug, PartialEq)]
pub struct ShoreMap {
    /// Corner of the grid (m, x and z) and size of each texel (m).
    pub corner: Vec2,
    pub texel: f32,
    pub width: u32,
    pub height: u32,
    /// Row by row, from `corner` towards +x and +z.
    pub distance: Vec<f32>,
    /// Towards the nearest shore, at each texel: 1 long where the shore is
    /// clearly one way, shorter where two shores pull apart (half way
    /// between an island and the coast), nothing on land.
    pub toward: Vec<Vec2>,
    /// Where the swell heads: the way the sea comes in to the coast near
    /// the start. Waves only roll in to the shores that face it, so they all
    /// go one way (between an island and the coast, only to one of them).
    pub swell: Vec2,
}

impl ShoreMap {
    /// Where the sea meets the terrain of this circuit.
    pub fn new(terrain: &Terrain, track: &Track) -> Self {
        let (min, max) = track.bounds();
        let corner = min - SHORE_MARGIN;
        let size = max + SHORE_MARGIN - corner;
        let texel = (size.max_element() / SHORE_TEXELS).max(SHORE_TEXEL_MIN);
        let (width, height) = (
            (size.x / texel).ceil() as usize,
            (size.y / texel).ceil() as usize,
        );
        let land: Vec<bool> = (0..height)
            .flat_map(|j| (0..width).map(move |i| (i, j)))
            .map(|(i, j)| {
                let at = corner + (Vec2::new(i as f32, j as f32) + 0.5) * texel;
                terrain.height(at) > SEA_LEVEL
            })
            .collect();
        let start = track.centerline().first().copied().unwrap_or_default();
        Self::from_land(corner, texel, width, height, &land, start)
    }

    /// The map of a grid where `land` says which texels are dry, for a race
    /// that starts at `start`.
    fn from_land(
        corner: Vec2,
        texel: f32,
        width: usize,
        height: usize,
        land: &[bool],
        start: Vec2,
    ) -> Self {
        let to_land = chamfer(land, width, height, true);
        let to_sea = chamfer(land, width, height, false);
        // Half a texel either side: the shore runs between the centres of a
        // dry texel and a wet one.
        let distance: Vec<f32> = land
            .iter()
            .zip(to_land.iter().zip(&to_sea))
            .map(|(&dry, (&wet_to_land, &dry_to_sea))| {
                if dry {
                    -(dry_to_sea - 0.5) * texel
                } else {
                    (wet_to_land - 0.5) * texel
                }
            })
            .collect();
        let toward = toward_shore(&distance, width, height, texel);

        // The swell: the average way to the shore, from the sea near the
        // start; from anywhere near a shore if there is none there.
        let center =
            |k: usize| corner + (Vec2::new((k % width) as f32, (k / width) as f32) + 0.5) * texel;
        let average = |near_start: bool| {
            distance
                .iter()
                .zip(&toward)
                .enumerate()
                .filter(|&(k, (&d, _))| {
                    d > 0.0
                        && d < SWELL_WATER
                        && (!near_start || center(k).distance(start) < SWELL_AROUND_START)
                })
                .map(|(_, (_, &way))| way)
                .sum::<Vec2>()
        };
        let swell = average(true)
            .try_normalize()
            .or_else(|| average(false).try_normalize())
            .unwrap_or(Vec2::Y);
        Self {
            corner,
            texel,
            width: width as u32,
            height: height as u32,
            distance,
            toward,
            swell,
        }
    }

    /// The map as the shader reads it: corner and size, and an image.
    /// The map as an image: distance to the shore and the way to it, texel
    /// by texel (made away from the main thread with the rest of the
    /// circuit).
    pub fn image(&self) -> Image {
        let data = self
            .distance
            .iter()
            .zip(&self.toward)
            .flat_map(|(&d, &way)| {
                let [near, far] = encode(d);
                let [x, z] = way
                    .to_array()
                    .map(|v| ((v * 0.5 + 0.5) * 255.0).round() as u8);
                [near, far, x, z]
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
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::ClampToEdge,
            address_mode_v: ImageAddressMode::ClampToEdge,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            ..default()
        });
        image
    }

    /// The sea's material, with the map as an image (see [`Self::image`]),
    /// for water of that colour.
    pub fn material(&self, shore: Handle<Image>, water: Water) -> SeaMaterial {
        let size = Vec2::new(self.width as f32, self.height as f32) * self.texel;
        SeaMaterial {
            shore_area: self.corner.extend(size.x).extend(size.y),
            shore,
            swell: self.swell.extend(0.0).extend(0.0),
            colors: SeaColors::of(water),
        }
    }
}

/// Towards the nearest shore at each texel of a map of `distance` to it:
/// down the slope of the distance (over two texels each way, to smooth the
/// steps of the distance), at most 1 long.
fn toward_shore(distance: &[f32], width: usize, height: usize, texel: f32) -> Vec<Vec2> {
    let at = |i: usize, j: usize| distance[j * width + i].min(WAVES_REACH);
    let mut toward = Vec::with_capacity(distance.len());
    for j in 0..height {
        for i in 0..width {
            if distance[j * width + i] <= 0.0 {
                toward.push(Vec2::ZERO);
                continue;
            }
            let (left, right) = (i.saturating_sub(2), (i + 2).min(width - 1));
            let (back, front) = (j.saturating_sub(2), (j + 2).min(height - 1));
            let slope = Vec2::new(
                (at(right, j) - at(left, j)) / ((right - left).max(1) as f32 * texel),
                (at(i, front) - at(i, back)) / ((front - back).max(1) as f32 * texel),
            );
            toward.push((-slope).clamp_length_max(1.0));
        }
    }
    smooth_ways(&toward, width, height)
}

/// The ways to the shore with their direction averaged over a few texels
/// around (the distance comes in steps, and so would the waves showing or
/// not), keeping how long each one is.
fn smooth_ways(ways: &[Vec2], width: usize, height: usize) -> Vec<Vec2> {
    const REACH: usize = 3;
    let blur = |ways: &[Vec2], along_rows: bool| -> Vec<Vec2> {
        let (lines, length) = if along_rows {
            (height, width)
        } else {
            (width, height)
        };
        let mut out = vec![Vec2::ZERO; ways.len()];
        for line in 0..lines {
            let index = |n: usize| {
                if along_rows {
                    line * width + n
                } else {
                    n * width + line
                }
            };
            for n in 0..length {
                let (from, to) = (n.saturating_sub(REACH), (n + REACH).min(length - 1));
                out[index(n)] = (from..=to).map(|m| ways[index(m)]).sum();
            }
        }
        out
    };
    let blurred = blur(&blur(ways, true), false);
    ways.iter()
        .zip(blurred)
        .map(|(way, around)| around.normalize_or_zero() * way.length())
        .collect()
}

/// How the shore map stores a distance to the shore (m): close up, and how
/// far out to sea.
fn encode(distance: f32) -> [u8; 2] {
    let near = (distance - SHORE_NEAR) / (SHORE_FAR - SHORE_NEAR);
    let far = distance / WAVES_REACH;
    [near, far].map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// How far each texel is from the nearest texel where `land` is `wanted`
/// (in texels), measured in steps of one texel straight or diagonally.
fn chamfer(land: &[bool], width: usize, height: usize, wanted: bool) -> Vec<f32> {
    let far = f32::INFINITY;
    let mut distance: Vec<f32> = land
        .iter()
        .map(|&dry| if dry == wanted { 0.0 } else { far })
        .collect();
    let diagonal = std::f32::consts::SQRT_2;
    // Down and to the right, then up and to the left.
    for j in 0..height {
        for i in 0..width {
            let k = j * width + i;
            let mut d = distance[k];
            if i > 0 {
                d = d.min(distance[k - 1] + 1.0);
            }
            if j > 0 {
                d = d.min(distance[k - width] + 1.0);
                if i > 0 {
                    d = d.min(distance[k - width - 1] + diagonal);
                }
                if i + 1 < width {
                    d = d.min(distance[k - width + 1] + diagonal);
                }
            }
            distance[k] = d;
        }
    }
    for j in (0..height).rev() {
        for i in (0..width).rev() {
            let k = j * width + i;
            let mut d = distance[k];
            if i + 1 < width {
                d = d.min(distance[k + 1] + 1.0);
            }
            if j + 1 < height {
                d = d.min(distance[k + width] + 1.0);
                if i + 1 < width {
                    d = d.min(distance[k + width + 1] + diagonal);
                }
                if i > 0 {
                    d = d.min(distance[k + width - 1] + diagonal);
                }
            }
            distance[k] = d;
        }
    }
    distance
}

/// The sea: a flat square out past the fog, the same for every circuit.
pub fn sea_mesh() -> Mesh {
    const CELLS: u32 = 16;
    let row = CELLS + 1;
    let positions: Vec<[f32; 3]> = (0..row)
        .flat_map(|j| (0..row).map(move |i| (i, j)))
        .map(|(i, j)| {
            let at = |n: u32| (n as f32 / CELLS as f32 - 0.5) * 2.0 * SEA_EXTENT;
            [at(i), 0.0, at(j)]
        })
        .collect();
    let indices: Vec<u32> = (0..CELLS)
        .flat_map(|j| (0..CELLS).map(move |i| (i, j)))
        .flat_map(|(i, j)| {
            let corner = |di: u32, dj: u32| (j + dj) * row + i + di;
            // Anticlockwise seen from above, facing up.
            [
                corner(0, 0),
                corner(0, 1),
                corner(1, 0),
                corner(1, 0),
                corner(0, 1),
                corner(1, 1),
            ]
        })
        .collect();
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shore_runs_between_dry_and_wet_texels() {
        // An island of 3 × 3 texels in the middle of a 9 × 9 sea, 2 m texels.
        let (width, height) = (9, 9);
        let land: Vec<bool> = (0..height)
            .flat_map(|j| (0..width).map(move |i| (i, j)))
            .map(|(i, j): (usize, usize)| (3..6).contains(&i) && (3..6).contains(&j))
            .collect();
        let map = ShoreMap::from_land(Vec2::ZERO, 2.0, width, height, &land, Vec2::ZERO);
        let at = |i: usize, j: usize| map.distance[j * width + i];
        // Next to the coast, half a texel either side of it.
        assert_eq!(at(2, 4), 1.0);
        assert_eq!(at(3, 4), -1.0);
        // The middle of the island is the furthest inland, and the corners
        // of the sea are the furthest out.
        assert_eq!(at(4, 4), -3.0);
        assert!(at(0, 0) > at(1, 1) && at(1, 1) > at(2, 2));
        // Straight out from the coast, one texel (2 m) at a time.
        assert_eq!(at(1, 4) - at(2, 4), 2.0);
    }

    #[test]
    fn the_map_keeps_the_shore_close_up_and_the_distance_far_out() {
        assert_eq!(encode(SHORE_NEAR), [0, 0]);
        assert_eq!(encode(0.0)[1], 0);
        assert_eq!(encode(SHORE_FAR), [255, 26]);
        assert_eq!(encode(1000.0), [255, 255]);
        // Close up, a tenth of a metre still shows.
        assert_ne!(encode(1.0)[0], encode(1.2)[0]);
    }

    #[test]
    fn waves_head_one_way_between_an_island_and_the_coast() {
        // A coast along the far side (+z) of a 60 × 60 map of 2 m texels,
        // and an island in the middle; the race starts on the coast.
        let (width, height) = (60, 60);
        let land: Vec<bool> = (0..height)
            .flat_map(|j| (0..width).map(move |i| (i, j)))
            .map(|(i, j): (usize, usize)| {
                j >= 50 || ((25..35).contains(&i) && (20..30).contains(&j))
            })
            .collect();
        let map = ShoreMap::from_land(
            Vec2::ZERO,
            2.0,
            width,
            height,
            &land,
            Vec2::new(60.0, 104.0),
        );
        // The swell heads for the coast.
        assert!(map.swell.dot(Vec2::Y) > 0.9, "{}", map.swell);
        let facing = |i: usize, j: usize| map.toward[j * width + i].dot(map.swell);
        // In front of the island and of the coast, the waves come in...
        assert!(facing(30, 16) > 0.8);
        assert!(facing(30, 47) > 0.8);
        // ...but not on the island's side away from the swell, which faces
        // the coast: between the two, only one way.
        assert!(facing(30, 33) < 0.0);
        // Nothing on land.
        assert_eq!(map.toward[55 * width + 30], Vec2::ZERO);
    }

    #[test]
    fn with_no_land_the_whole_map_is_open_sea() {
        let map = ShoreMap::from_land(Vec2::ZERO, 2.0, 4, 3, &[false; 12], Vec2::ZERO);
        assert!(map.distance.iter().all(|&d| d > SHORE_FAR));
    }

    #[test]
    fn the_sea_is_blue_and_the_water_of_a_mine_red() {
        let sea = SeaColors::of(Water::Sea);
        let iron = SeaColors::of(Water::Iron);
        for color in [sea.deep, sea.open, sea.shallow] {
            assert!(color.z > color.x && color.z > color.y);
        }
        for color in [iron.deep, iron.open, iron.shallow] {
            assert!(color.x > 2.0 * color.y && color.x > 2.0 * color.z);
        }
        // The Gold Mine's lake and river are red, every other track's sea
        // blue.
        for circuit in &crate::circuits::CIRCUITS {
            let red = circuit.scenery.mine.is_some();
            let wanted = if red { Water::Iron } else { Water::Sea };
            assert_eq!(circuit.scenery.water, wanted, "{}", circuit.name);
        }
    }

    #[test]
    fn every_circuit_has_shore_near_its_road() {
        for circuit in &crate::circuits::CIRCUITS {
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, &circuit.scenery);
            let map = ShoreMap::new(&terrain, &track);
            assert!(map.width <= SHORE_TEXELS as u32 + 1 && map.height <= SHORE_TEXELS as u32 + 1);
            // Some sea and some land.
            assert!(map.distance.iter().any(|&d| d > 5.0));
            assert!(map.distance.iter().any(|&d| d < -5.0));
        }
    }
}
