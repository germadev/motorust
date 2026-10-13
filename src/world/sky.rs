//! The sky: a dome always centred on the camera, so it looks infinitely far
//! away, in the cartoon style of the sea: a plain gradient,
//! deep blue overhead and pale towards the horizon (the colour of the fog),
//! the sun, and cartoon clouds (puffs over a flat bottom, white on top and
//! lavender underneath) around the horizon, drifting slowly.
//!
//! The clouds and the sun are vector shapes drawn once, at startup, into a
//! few images (the clouds' outlines with zeno, the rasterizer Bevy uses for
//! the fonts). Every frame they cost a few quads that share their mesh and
//! materials, and turning the one entity that holds them.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    light::NotShadowCaster,
    mesh::Indices,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use swash::zeno::Mask;

use super::generate_mipmaps;
use crate::geometry::smoothstep;

/// Radio de la cúpula: dentro del plano lejano de la cámara.
const RADIUS: f32 = 4500.0;
const SEGMENTS: u32 = 64;
const RINGS: u32 = 24;
/// La cúpula baja un poco por debajo del horizonte para que no se vea el borde.
const LOWEST_ELEVATION: f32 = -0.25;
/// Altura del sol sobre el horizonte (rad), y rumbo: al sureste, detrás a la
/// derecha en la recta de meta (0 es el norte, -Z; π/2 el este, +X).
const SUN_ELEVATION: f32 = 0.834;
const SUN_AZIMUTH: f32 = 0.75 * std::f32::consts::PI;

/// The colours of the sky (sRGB): overhead, a little above the horizon, and
/// at the horizon, where it is the colour of the fog, so the land far away
/// melts into it.
const ZENITH: Color = Color::srgb(0.16, 0.42, 0.88);
const ABOVE_HORIZON: Color = Color::srgb(0.36, 0.62, 0.95);
pub const HORIZON: Color = Color::srgb(0.68, 0.8, 0.94);

/// How many clouds there are, how far from the camera (m: inside the dome),
/// how high above the horizon (rad), how wide (m), and how fast they all
/// turn round the sky (rad/s).
const CLOUDS: usize = 18;
const CLOUD_DISTANCE: f32 = 4200.0;
const CLOUD_ELEVATION: (f32, f32) = (0.035, 0.24);
const CLOUD_WIDTH: (f32, f32) = (650.0, 1250.0);
const CLOUD_DRIFT: f32 = 0.004;
/// The different cloud shapes, and the size of their images (px).
const CLOUD_SHAPES: u32 = 4;
const CLOUD_IMAGE: UVec2 = UVec2::new(512, 256);
/// Where the flat bottom of a cloud is, down its image (fraction).
const CLOUD_BOTTOM: f32 = 0.8;
/// The colours of a cloud (sRGB): its top, and its shaded underside.
const CLOUD_TOP: [f32; 3] = [1.0, 1.0, 1.0];
const CLOUD_UNDERSIDE: [f32; 3] = [0.79, 0.83, 0.95];
/// The sun: how big it looks (m, at the clouds' distance), and its image.
const SUN_SIZE: f32 = 520.0;
const SUN_IMAGE: u32 = 128;

#[derive(Component)]
pub struct Sky;

/// The clouds, turning slowly round the sky.
#[derive(Component)]
pub struct Clouds;

/// Dirección hacia el sol.
pub fn sun_direction() -> Vec3 {
    direction(SUN_AZIMUTH, SUN_ELEVATION)
}

/// Dirección para un rumbo (0 = norte, -Z) y una altura sobre el horizonte.
fn direction(azimuth: f32, elevation: f32) -> Vec3 {
    Vec3::new(
        azimuth.sin() * elevation.cos(),
        elevation.sin(),
        -azimuth.cos() * elevation.cos(),
    )
}

pub fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Far away and drawn as they are: no light, no fog, seen from inside.
    let mut flat = |image: Option<Image>, alpha_mode: AlphaMode| {
        materials.add(StandardMaterial {
            base_color_texture: image.map(|image| images.add(image)),
            alpha_mode,
            unlit: true,
            fog_enabled: false,
            cull_mode: None,
            double_sided: true,
            ..default()
        })
    };
    let dome_material = flat(None, AlphaMode::Opaque);
    let sun_material = flat(Some(sun_image()), AlphaMode::Blend);
    let cloud_materials: Vec<_> = (0..CLOUD_SHAPES)
        .map(|shape| flat(Some(cloud_image(shape)), AlphaMode::Blend))
        .collect();
    let quad = meshes.add(Rectangle::new(1.0, 1.0));

    commands
        .spawn((
            Sky,
            Mesh3d(meshes.add(dome())),
            MeshMaterial3d(dome_material),
            Transform::default(),
            Visibility::default(),
            NotShadowCaster,
        ))
        .with_children(|sky| {
            let at = sun_direction() * CLOUD_DISTANCE;
            sky.spawn((
                Mesh3d(quad.clone()),
                MeshMaterial3d(sun_material),
                facing_the_camera(at, Vec2::splat(SUN_SIZE)),
                NotShadowCaster,
            ));
            sky.spawn((Clouds, Transform::default(), Visibility::default()))
                .with_children(|clouds| {
                    for (index, (at, size)) in cloud_places().into_iter().enumerate() {
                        clouds.spawn((
                            Mesh3d(quad.clone()),
                            MeshMaterial3d(cloud_materials[index % cloud_materials.len()].clone()),
                            facing_the_camera(at, size),
                            NotShadowCaster,
                        ));
                    }
                });
        });
}

/// A quad `size` m big at `at`, facing the camera at the centre of the sky.
fn facing_the_camera(at: Vec3, size: Vec2) -> Transform {
    Transform::from_translation(at)
        .looking_to(at.normalize(), Vec3::Y)
        .with_scale(size.extend(1.0))
}

/// Where each cloud is (m from the camera) and its size (m): spread round
/// the sky, each at its own height and size.
fn cloud_places() -> Vec<(Vec3, Vec2)> {
    let aspect = CLOUD_IMAGE.y as f32 / CLOUD_IMAGE.x as f32;
    (0..CLOUDS)
        .map(|index| {
            let seed = index as u32;
            let azimuth = (index as f32 + 0.8 * random(seed, 0)) / CLOUDS as f32 * TAU;
            let elevation = CLOUD_ELEVATION
                .0
                .lerp(CLOUD_ELEVATION.1, random(seed, 1).powi(2));
            let width = CLOUD_WIDTH.0.lerp(CLOUD_WIDTH.1, random(seed, 2));
            (
                direction(azimuth, elevation) * CLOUD_DISTANCE,
                Vec2::new(width, width * aspect),
            )
        })
        .collect()
}

/// The clouds turn slowly round the sky.
pub fn drift_clouds(time: Res<Time>, mut clouds: Query<&mut Transform, With<Clouds>>) {
    for mut transform in &mut clouds {
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * CLOUD_DRIFT);
    }
}

/// A number from 0 to 1 that only depends on `seed` and `n`.
fn random(seed: u32, n: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9e37_79b9) ^ n.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

/// The outline of cloud `shape` in its image (px), as SVG path data: puffs
/// of different sizes side by side, sitting on a block that fills the gaps
/// between them down to the flat bottom.
fn cloud_outline(shape: u32) -> String {
    let (width, height) = (CLOUD_IMAGE.x as f32, CLOUD_IMAGE.y as f32);
    let bottom = height * CLOUD_BOTTOM;
    let puffs = 4 + shape % 3;
    let mut path = String::new();
    let (mut left, mut right) = (width, 0.0f32);
    for puff in 0..puffs {
        let t = (puff as f32 + 0.5) / puffs as f32;
        // Bigger towards the middle, each a little different.
        let middle = 1.0 - (2.0 * t - 1.0).abs();
        let radius = height * (0.17 + 0.17 * middle + 0.07 * random(shape + 7, puff));
        let x = width * (0.13 + 0.74 * t) + width * 0.04 * (random(shape + 3, puff) - 0.5);
        let y = bottom - radius * 0.55;
        left = left.min(x);
        right = right.max(x);
        path += &format!(
            "M {} {y} A {radius} {radius} 0 1 0 {} {y} A {radius} {radius} 0 1 0 {} {y} Z ",
            x - radius,
            x + radius,
            x - radius,
        );
    }
    // Wound the same way as the puffs, so where they overlap it stays filled.
    let top = bottom - height * 0.18;
    path += &format!("M {left} {top} L {left} {bottom} L {right} {bottom} L {right} {top} Z");
    path
}

/// The image of cloud `shape`: its outline filled, cut flat at the bottom,
/// white on top and shaded underneath, with mipmaps (it is seen at many
/// sizes).
fn cloud_image(shape: u32) -> Image {
    let (width, height) = (CLOUD_IMAGE.x, CLOUD_IMAGE.y);
    let (coverage, _) = Mask::new(cloud_outline(shape).as_str())
        .size(width, height)
        .render();
    let bottom = height as f32 * CLOUD_BOTTOM;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let row = y as f32 + 0.5;
        // The flat bottom, smooth over a pixel.
        let above = (bottom - row + 0.5).clamp(0.0, 1.0);
        let shade = smoothstep(bottom - height as f32 * 0.3, bottom, row);
        let color: [f32; 3] =
            std::array::from_fn(|c| CLOUD_TOP[c] + (CLOUD_UNDERSIDE[c] - CLOUD_TOP[c]) * shade);
        for x in 0..width {
            let alpha = coverage[(y * width + x) as usize] as f32 / 255.0 * above;
            // The colour even where it is clear, so no dark rim shows when
            // the image is shown smaller.
            data.extend(color.map(|c| (c * 255.0).round() as u8));
            data.push((alpha * 255.0).round() as u8);
        }
    }
    smooth_image(width, height, data)
}

/// The sun: a pale disc with a soft glow round it.
fn sun_image() -> Image {
    let size = SUN_IMAGE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) / size as f32 * 2.0 - 1.0;
            let r = p.length();
            let disc = (0.3 - r) * size as f32 * 0.5 + 0.5;
            let glow = (1.0 - r).max(0.0).powi(3) * 0.45;
            let alpha = disc.clamp(0.0, 1.0).max(glow);
            data.extend([255, 250, 226, (alpha * 255.0).round() as u8]);
        }
    }
    smooth_image(size, size, data)
}

/// An sRGB image with mipmaps and smooth sampling.
fn smooth_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    generate_mipmaps(&mut image);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::default()
    });
    image
}

/// La cúpula sigue a la cámara (solo en posición, no en giro).
pub fn follow_camera(
    cameras: Query<&Transform, (With<Camera3d>, Without<Sky>)>,
    mut sky: Single<&mut Transform, With<Sky>>,
) {
    if let Some(camera) = cameras.iter().next() {
        sky.translation = camera.translation;
    }
}

/// The colour of the sky (sRGB) at an elevation (rad).
fn sky_color(elevation: f32) -> Color {
    let low = smoothstep(0.0, 0.18, elevation);
    let high = smoothstep(0.12, 1.2, elevation);
    let mix = |a: Color, b: Color, t: f32| a.mix(&b, t);
    mix(mix(HORIZON, ABOVE_HORIZON, low), ZENITH, high)
}

/// The dome, coloured from the horizon to the zenith.
fn dome() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    for ring in 0..=RINGS {
        let elevation = LOWEST_ELEVATION.lerp(FRAC_PI_2, ring as f32 / RINGS as f32);
        for segment in 0..=SEGMENTS {
            let fraction = segment as f32 / SEGMENTS as f32;
            let toward = direction(fraction * TAU, elevation);
            positions.push((toward * RADIUS).to_array());
            normals.push((-toward).to_array());
            colors.push(sky_color(elevation).to_linear().to_f32_array());
        }
    }
    let row = SEGMENTS + 1;
    let mut indices = Vec::new();
    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let a = ring * row + segment;
            let b = a + row;
            indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_shines_from_the_south_east_and_high() {
        let sun = sun_direction();
        assert!(sun.x > 0.0 && sun.z > 0.0, "sureste: +X y +Z");
        assert!(sun.y > 0.6);
    }

    #[test]
    fn the_sky_meets_the_fog_at_the_horizon() {
        assert_eq!(sky_color(0.0), HORIZON);
        assert_eq!(sky_color(-0.2), HORIZON);
        // Bluer and bluer going up.
        let blue = |elevation: f32| {
            let c = sky_color(elevation).to_srgba();
            c.blue - c.red
        };
        assert!(blue(0.3) > blue(0.05));
        assert!(blue(FRAC_PI_2) > blue(0.3));
    }

    #[test]
    fn clouds_are_puffs_on_a_flat_bottom() {
        for shape in 0..CLOUD_SHAPES {
            let image = cloud_image(shape);
            let data = image.data.as_ref().unwrap();
            let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize);
            let alpha = |x: usize, y: usize| data[(y * width + x) * 4 + 3];
            let bottom = (CLOUD_IMAGE.y as f32 * CLOUD_BOTTOM) as usize;
            // Solid in the middle, clear in the corners and under the bottom.
            assert_eq!(alpha(width / 2, bottom - 4), 255, "{shape}");
            assert_eq!(alpha(1, 1), 0);
            assert_eq!(alpha(width - 2, 1), 0);
            assert!((0..width).all(|x| alpha(x, bottom + 2) == 0));
            // The underside is shaded, the top is white.
            let red = |x: usize, y: usize| data[(y * width + x) * 4];
            assert!(red(width / 2, bottom - 2) < red(width / 2, bottom / 3));
            // Clear all round the edges, so no cloud is cut off.
            assert!((0..width).all(|x| alpha(x, 0) == 0));
            assert!((0..height).all(|y| alpha(0, y) == 0 && alpha(width - 1, y) == 0));
        }
    }

    #[test]
    fn clouds_ring_the_horizon_inside_the_dome() {
        let places = cloud_places();
        assert_eq!(places.len(), CLOUDS);
        for (at, size) in places {
            // Even its corners.
            let corner = (at.length_squared() + (size / 2.0).length_squared()).sqrt();
            assert!(corner < RADIUS);
            let elevation = (at.y / at.length()).asin();
            assert!((CLOUD_ELEVATION.0..=CLOUD_ELEVATION.1).contains(&elevation));
        }
    }
}
