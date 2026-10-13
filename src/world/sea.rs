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
    AsBindGroup, Extent3d, PrimitiveTopology, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

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

/// How the sea looks (see `sea.wgsl`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SeaMaterial {
    /// xy: corner of the shore map (m, x and z); zw: its size (m).
    #[uniform(0)]
    pub shore_area: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub shore: Handle<Image>,
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
        Self::from_land(corner, texel, width, height, &land)
    }

    /// The map of a grid where `land` says which texels are dry.
    fn from_land(corner: Vec2, texel: f32, width: usize, height: usize, land: &[bool]) -> Self {
        let to_land = chamfer(land, width, height, true);
        let to_sea = chamfer(land, width, height, false);
        // Half a texel either side: the shore runs between the centres of a
        // dry texel and a wet one.
        let distance = land
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
        Self {
            corner,
            texel,
            width: width as u32,
            height: height as u32,
            distance,
        }
    }

    /// The map as the shader reads it: corner and size, and an image.
    pub fn material(&self, images: &mut Assets<Image>) -> SeaMaterial {
        let data = self.distance.iter().flat_map(|&d| encode(d)).collect();
        let mut image = Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rg8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::ClampToEdge,
            address_mode_v: ImageAddressMode::ClampToEdge,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            ..default()
        });
        let size = Vec2::new(self.width as f32, self.height as f32) * self.texel;
        SeaMaterial {
            shore_area: self.corner.extend(size.x).extend(size.y),
            shore: images.add(image),
        }
    }
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
        let map = ShoreMap::from_land(Vec2::ZERO, 2.0, width, height, &land);
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
    fn with_no_land_the_whole_map_is_open_sea() {
        let map = ShoreMap::from_land(Vec2::ZERO, 2.0, 4, 3, &[false; 12]);
        assert!(map.distance.iter().all(|&d| d > SHORE_FAR));
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
