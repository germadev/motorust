//! Texturas del escenario: fotos CC0 de Poly Haven (ver `assets/CREDITS.md`)
//! retocadas con colores más vivos, como en los juegos de los 90. Van
//! incrustadas en el ejecutable y sus mipmaps se calculan al arrancar, así no
//! parpadean a lo lejos.

use crate::geometry::{fbm, value_noise};
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageFilterMode, ImageSampler,
        ImageSamplerDescriptor, ImageType,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Resource)]
pub struct WorldTextures {
    pub sky: Handle<Image>,
    pub grass: Handle<Image>,
    pub rock: Handle<Image>,
    pub sand: Handle<Image>,
    pub asphalt: Handle<Image>,
    pub wall: Handle<Image>,
    pub roof: Handle<Image>,
    pub plaster: Handle<Image>,
    /// Tablero de ajedrez blanco y negro de 2 × 2 casillas.
    pub checker: Handle<Image>,
    /// Paneles de hormigón con juntas, para los túneles.
    pub panels: Handle<Image>,
}

impl FromWorld for WorldTextures {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let mut tiled = |bytes: &[u8]| images.add(embedded(bytes, ImageAddressMode::Repeat));
        let grass = tiled(include_bytes!("../../assets/textures/grass.jpg"));
        let rock = tiled(include_bytes!("../../assets/textures/rock.jpg"));
        let sand = tiled(include_bytes!("../../assets/textures/sand.jpg"));
        let asphalt = tiled(include_bytes!("../../assets/textures/asphalt.jpg"));
        let wall = tiled(include_bytes!("../../assets/textures/wall.jpg"));
        let roof = tiled(include_bytes!("../../assets/textures/roof.jpg"));
        let plaster = tiled(include_bytes!("../../assets/textures/plaster.jpg"));
        // El cielo da la vuelta en horizontal pero no en vertical.
        let sky = images.add(embedded(
            include_bytes!("../../assets/textures/sky.jpg"),
            ImageAddressMode::ClampToEdge,
        ));
        let checker = images.add(checker());
        let panels = images.add(panels());
        Self {
            sky,
            grass,
            rock,
            sand,
            asphalt,
            wall,
            roof,
            plaster,
            checker,
            panels,
        }
    }
}

fn sampler(vertical: ImageAddressMode) -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: vertical,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        // Nitidez en el suelo visto en ángulo rasante, como el asfalto a lo lejos.
        anisotropy_clamp: 16,
        ..default()
    })
}

fn embedded(bytes: &[u8], vertical: ImageAddressMode) -> Image {
    let mut image = Image::from_buffer(
        bytes,
        ImageType::Extension("jpg"),
        CompressedImageFormats::NONE,
        true,
        sampler(vertical),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("las texturas incrustadas son JPEG válidos");
    generate_mipmaps(&mut image);
    image
}

/// Dos por dos paneles de hormigón, cada uno algo distinto, con juntas
/// oscuras y manchas: se repite sin que se note la costura.
fn panels() -> Image {
    const SIZE: u32 = 256;
    const JOINT: u32 = 5;
    let data = (0..SIZE * SIZE)
        .flat_map(|i| {
            let (x, y) = (i % SIZE, i / SIZE);
            let half = SIZE / 2;
            let (px, py) = (x % half, y % half);
            let joint = px < JOINT || py < JOINT;
            // Cada panel, de un tono; con manchas finas y gruesas.
            let panel = (x / half + 2 * (y / half)) as f32;
            let point = Vec2::new(x as f32, y as f32);
            let stain = fbm(point / 18.0 + Vec2::splat(panel * 7.3), 3);
            let grain = value_noise(point / 2.5);
            let tone =
                0.78 + 0.05 * (panel * 1.7).sin() + 0.12 * (stain - 0.5) + 0.06 * (grain - 0.5);
            let value = if joint { 0.35 } else { tone };
            let v = (value.clamp(0.0, 1.0) * 255.0) as u8;
            [v, v, v, 255]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = sampler(ImageAddressMode::Repeat);
    generate_mipmaps(&mut image);
    image
}

fn checker() -> Image {
    const SIZE: u32 = 64;
    let data = (0..SIZE * SIZE)
        .flat_map(|i| {
            let (x, y) = (i % SIZE, i / SIZE);
            let value = if (x < SIZE / 2) == (y < SIZE / 2) {
                245
            } else {
                15
            };
            [value, value, value, 255]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = sampler(ImageAddressMode::Repeat);
    generate_mipmaps(&mut image);
    image
}

/// Añade a una imagen RGBA de 8 bits todos sus niveles de mipmap, cada uno la
/// mitad de grande que el anterior, promediando en espacio lineal: las de
/// color (sRGB) se pasan a lineal para promediar; las de datos (normales,
/// rugosidad) ya lo están.
pub fn generate_mipmaps(image: &mut Image) {
    let srgb = match image.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb => true,
        TextureFormat::Rgba8Unorm => false,
        other => panic!("solo se generan mipmaps de imágenes RGBA de 8 bits, no {other:?}"),
    };
    let to_linear: Vec<f32> = (0..=255u8)
        .map(|v| {
            if srgb {
                Color::srgb_u8(v, 0, 0).to_linear().red
            } else {
                v as f32 / 255.0
            }
        })
        .collect();
    let to_srgb = |linear: f32| {
        let value = if srgb {
            Color::linear_rgb(linear, 0.0, 0.0).to_srgba().red
        } else {
            linear
        };
        (value * 255.0).round() as u8
    };

    let (mut width, mut height) = (image.width(), image.height());
    let mut data = image.data.take().expect("la imagen tiene datos en la CPU");
    let mut level = data.clone();
    let mut levels = 1;
    while width > 1 || height > 1 {
        let (next_width, next_height) = ((width / 2).max(1), (height / 2).max(1));
        let mut next = Vec::with_capacity((next_width * next_height * 4) as usize);
        for y in 0..next_height {
            for x in 0..next_width {
                for channel in 0..4 {
                    let mut sum = 0.0;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (sx, sy) = ((2 * x + dx).min(width - 1), (2 * y + dy).min(height - 1));
                        let value = level[((sy * width + sx) * 4 + channel) as usize];
                        sum += if channel < 3 {
                            to_linear[value as usize]
                        } else {
                            value as f32 / 255.0
                        };
                    }
                    let average = sum / 4.0;
                    next.push(if channel < 3 {
                        to_srgb(average)
                    } else {
                        (average * 255.0).round() as u8
                    });
                }
            }
        }
        data.extend_from_slice(&next);
        level = next;
        (width, height) = (next_width, next_height);
        levels += 1;
    }
    image.data = Some(data);
    image.texture_descriptor.mip_level_count = levels;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mipmaps_go_down_to_one_pixel() {
        let mut image = checker();
        let levels = image.texture_descriptor.mip_level_count;
        assert_eq!(levels, 7, "64, 32, 16, 8, 4, 2 y 1 píxeles");
        let expected: u32 = (0..levels).map(|l| (64 >> l) * (64 >> l) * 4).sum();
        assert_eq!(image.data.as_ref().unwrap().len() as u32, expected);
        // El último nivel es la media: gris, no negro ni blanco.
        let last = &image.data.take().unwrap()[expected as usize - 4..];
        assert!(last[0] > 100 && last[0] < 220);
    }

    #[test]
    fn embedded_textures_decode() {
        let image = embedded(
            include_bytes!("../../assets/textures/plaster.jpg"),
            ImageAddressMode::Repeat,
        );
        assert!(image.texture_descriptor.mip_level_count > 1);
    }
}
