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
    /// Agua del mar con oleaje sutil y reflejos vivos mediterráneos.
    pub water: Handle<Image>,
    /// Gradas de público con ropa de colores vivos como en los arcades de los 90.
    pub crowd: Handle<Image>,
}

impl FromWorld for WorldTextures {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let mut tiled = |bytes: &[u8], enhance: TextureEnhance| {
            images.add(embedded(bytes, ImageAddressMode::Repeat, enhance))
        };
        let grass = tiled(
            include_bytes!("../../assets/textures/grass.jpg"),
            TextureEnhance::Grass,
        );
        let rock = tiled(
            include_bytes!("../../assets/textures/rock.jpg"),
            TextureEnhance::Rock,
        );
        let sand = tiled(
            include_bytes!("../../assets/textures/sand.jpg"),
            TextureEnhance::Sand,
        );
        let asphalt = tiled(
            include_bytes!("../../assets/textures/asphalt.jpg"),
            TextureEnhance::Asphalt,
        );
        let wall = tiled(
            include_bytes!("../../assets/textures/wall.jpg"),
            TextureEnhance::Wall,
        );
        let roof = tiled(
            include_bytes!("../../assets/textures/roof.jpg"),
            TextureEnhance::None,
        );
        let plaster = tiled(
            include_bytes!("../../assets/textures/plaster.jpg"),
            TextureEnhance::None,
        );
        // El cielo da la vuelta en horizontal pero no en vertical.
        let sky = images.add(embedded(
            include_bytes!("../../assets/textures/sky.jpg"),
            ImageAddressMode::ClampToEdge,
            TextureEnhance::None,
        ));
        let checker = images.add(checker());
        let panels = images.add(panels());
        let water = images.add(water());
        let crowd = images.add(crowd());
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
            water,
            crowd,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureEnhance {
    Grass,
    Rock,
    Sand,
    Asphalt,
    Wall,
    None,
}

fn embedded(bytes: &[u8], vertical: ImageAddressMode, enhance: TextureEnhance) -> Image {
    let mut image = Image::from_buffer(
        bytes,
        ImageType::Extension("jpg"),
        CompressedImageFormats::NONE,
        true,
        sampler(vertical),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("las texturas incrustadas son JPEG válidos");
    enhance_texture(&mut image, enhance);
    generate_mipmaps(&mut image);
    image
}

/// Realce de contraste y micro-detalle para texturas retro estilo arcade:
/// asfalto con más grano de árido, hierba más frondosa y viva, y roca con estratos más marcados.
fn enhance_texture(image: &mut Image, kind: TextureEnhance) {
    if matches!(kind, TextureEnhance::None) {
        return;
    }
    let width = image.width() as usize;
    let height = image.height() as usize;
    let Some(data) = image.data.as_mut() else {
        return;
    };

    let orig = data.clone();
    let get_pixel = |x: usize, y: usize| -> [f32; 3] {
        let idx = (y * width + x) * 4;
        [
            orig[idx] as f32 / 255.0,
            orig[idx + 1] as f32 / 255.0,
            orig[idx + 2] as f32 / 255.0,
        ]
    };

    for y in 0..height {
        for x in 0..width {
            let center = get_pixel(x, y);
            // Desenfoque 3x3 continuo con repetición en los bordes para mantener la textura sin costuras
            let mut blur = [0.0f32; 3];
            for dy in [-1isize, 0, 1] {
                for dx in [-1isize, 0, 1] {
                    let sx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let sy = (y as isize + dy).rem_euclid(height as isize) as usize;
                    let p = get_pixel(sx, sy);
                    for c in 0..3 {
                        blur[c] += p[c] / 9.0;
                    }
                }
            }

            let mut r = center[0];
            let mut g = center[1];
            let mut b = center[2];

            match kind {
                TextureEnhance::Asphalt => {
                    // Micro-grano de árido sutil y natural, sin ruido de destellos
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (r + 0.40 * high_pass[0]).clamp(0.0, 1.0);
                    g = (g + 0.40 * high_pass[1]).clamp(0.0, 1.0);
                    b = (b + 0.40 * high_pass[2]).clamp(0.0, 1.0);

                    // Curva de luminosidad calibrada para asfalto de circuito:
                    // tono pizarra neutro, legible con sol y sombra, sin quedar negro empastado
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let target_lum = (lum * 1.38 + 0.025).clamp(0.0, 1.0);
                    let ratio = if lum > 1e-4 {
                        target_lum / lum
                    } else {
                        1.0
                    };
                    r = (r * ratio).clamp(0.0, 1.0);
                    g = (g * ratio).clamp(0.0, 1.0);
                    b = (b * ratio).clamp(0.0, 1.0);
                }
                TextureEnhance::Grass => {
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (r + 0.75 * high_pass[0]).clamp(0.0, 1.0);
                    g = (g + 0.75 * high_pass[1]).clamp(0.0, 1.0);
                    b = (b + 0.75 * high_pass[2]).clamp(0.0, 1.0);

                    // Tonalidad verde pradera arcade de los 90, viva y luminosa
                    g = (g * 1.16).min(1.0);
                    r = (r * 0.91 + 0.035).clamp(0.0, 1.0);
                    b = (b * 0.84).clamp(0.0, 1.0);
                    g = ((g - 0.44) * 1.26 + 0.44).clamp(0.0, 1.0);
                }
                TextureEnhance::Rock => {
                    // Fuerte relieve en fisuras y aristas de corte rocoso
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (r + 1.15 * high_pass[0]).clamp(0.0, 1.0);
                    g = (g + 1.15 * high_pass[1]).clamp(0.0, 1.0);
                    b = (b + 1.15 * high_pass[2]).clamp(0.0, 1.0);

                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let contrasted_lum = ((lum - 0.52) * 1.38 + 0.52).clamp(0.0, 1.0);
                    let ratio = if lum > 1e-4 {
                        contrasted_lum / lum
                    } else {
                        1.0
                    };
                    r = (r * ratio * 1.06).clamp(0.0, 1.0);
                    g = (g * ratio * 1.02).clamp(0.0, 1.0);
                    b = (b * ratio * 0.95).clamp(0.0, 1.0);
                }
                TextureEnhance::Sand => {
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (r + 0.65 * high_pass[0]).clamp(0.0, 1.0);
                    g = (g + 0.65 * high_pass[1]).clamp(0.0, 1.0);
                    b = (b + 0.65 * high_pass[2]).clamp(0.0, 1.0);
                    // Arena cálida y luminosa
                    r = (r * 1.07).min(1.0);
                    g = (g * 1.02).min(1.0);
                    b = (b * 0.92).clamp(0.0, 1.0);
                }
                TextureEnhance::Wall => {
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (r + 0.85 * high_pass[0]).clamp(0.0, 1.0);
                    g = (g + 0.85 * high_pass[1]).clamp(0.0, 1.0);
                    b = (b + 0.85 * high_pass[2]).clamp(0.0, 1.0);
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let contrasted_lum = ((lum - 0.5) * 1.25 + 0.5).clamp(0.0, 1.0);
                    let ratio = if lum > 1e-4 {
                        contrasted_lum / lum
                    } else {
                        1.0
                    };
                    r = (r * ratio).clamp(0.0, 1.0);
                    g = (g * ratio).clamp(0.0, 1.0);
                    b = (b * ratio).clamp(0.0, 1.0);
                }
                TextureEnhance::None => {}
            }

            let idx = (y * width + x) * 4;
            data[idx] = (r * 255.0).round() as u8;
            data[idx + 1] = (g * 255.0).round() as u8;
            data[idx + 2] = (b * 255.0).round() as u8;
        }
    }
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

/// Textura de agua para el mar: azules y turquesas vivos con ondulaciones
/// suaves de olas, reflejos y crestas de espuma que se repiten armónicamente.
fn water() -> Image {
    const SIZE: u32 = 256;
    let data = (0..SIZE * SIZE)
        .flat_map(|i| {
            let (x, y) = ((i % SIZE) as f32, (i / SIZE) as f32);
            let point = Vec2::new(x, y);
            let wave1 = (x * 0.08 + y * 0.05).sin();
            let wave2 = ((x * 0.06 - y * 0.09) * 1.3).sin();
            let detail = fbm(point / 12.0, 3);
            let ripple = 0.5 + 0.3 * wave1 + 0.2 * wave2 + 0.25 * (detail - 0.5);
            let deep = Color::srgb(0.03, 0.20, 0.52).to_srgba();
            let turquoise = Color::srgb(0.08, 0.50, 0.68).to_srgba();
            let highlight = Color::srgb(0.40, 0.78, 0.90).to_srgba();
            let c = if ripple < 0.65 {
                let t = (ripple / 0.65).clamp(0.0, 1.0);
                deep.mix(&turquoise, t)
            } else {
                let t = ((ripple - 0.65) / 0.35).clamp(0.0, 1.0);
                turquoise.mix(&highlight, t)
            };
            [
                (c.red * 255.0).round() as u8,
                (c.green * 255.0).round() as u8,
                (c.blue * 255.0).round() as u8,
                255,
            ]
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

/// Público pixelado para las gradas: filas de espectadores con ropa de colores vivos
/// como en los juegos de carreras arcade de los 90.
fn crowd() -> Image {
    const WIDTH: u32 = 128;
    const HEIGHT: u32 = 64;
    const SHIRTS: [Color; 6] = [
        Color::srgb(0.92, 0.15, 0.12),
        Color::srgb(0.98, 0.85, 0.15),
        Color::srgb(0.12, 0.45, 0.88),
        Color::srgb(0.95, 0.95, 0.95),
        Color::srgb(0.95, 0.50, 0.10),
        Color::srgb(0.15, 0.72, 0.28),
    ];
    let data = (0..WIDTH * HEIGHT)
        .flat_map(|i| {
            let (x, y) = (i % WIDTH, i / HEIGHT);
            let row = y / 16;
            let row_y = y % 16;
            let col = x / 8;
            let col_x = x % 8;
            let person_seed = (row * 37 + col * 13) as f32;
            let shirt_idx =
                ((person_seed * 1.7).sin().abs() * SHIRTS.len() as f32) as usize % SHIRTS.len();
            let shirt = SHIRTS[shirt_idx];
            let skin = Color::srgb(0.95, 0.78, 0.65);
            let hair = if (person_seed * 2.3).sin() > 0.0 {
                Color::srgb(0.25, 0.18, 0.12)
            } else {
                Color::srgb(0.55, 0.40, 0.20)
            };
            let bench = Color::srgb(0.55, 0.55, 0.58);
            let c = if row_y < 3 {
                bench
            } else if row_y < 7 && (2..=5).contains(&col_x) {
                if row_y < 5 { hair } else { skin }
            } else if (7..14).contains(&row_y) && (1..=6).contains(&col_x) {
                shirt
            } else {
                bench
            };
            let srgba = c.to_srgba();
            [
                (srgba.red * 255.0).round() as u8,
                (srgba.green * 255.0).round() as u8,
                (srgba.blue * 255.0).round() as u8,
                255,
            ]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
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
            include_bytes!("../../assets/textures/asphalt.jpg"),
            ImageAddressMode::Repeat,
            TextureEnhance::Asphalt,
        );
        assert!(image.texture_descriptor.mip_level_count > 1);
    }
}
