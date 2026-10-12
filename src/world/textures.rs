//! Texturas del escenario: fotos CC0 de Poly Haven (ver `assets/CREDITS.md`)
//! retocadas con colores más vivos, como en los juegos de los 90. Van
//! incrustadas en el ejecutable y sus mipmaps se calculan al arrancar, así no
//! parpadean a lo lejos.

use crate::geometry::{fbm, random01, value_noise};
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
    /// Two by two squares of brown checkerboard earth, as in Green Hill.
    pub soil: Handle<Image>,
    /// Streaks of falling water, repeating downwards.
    pub falls: Handle<Image>,
    /// Wooden planks laid across, for the wooden bridges.
    pub planks: Handle<Image>,
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
        let soil = images.add(soil());
        let falls = images.add(falls());
        let planks = images.add(planks());
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
            soil,
            falls,
            planks,
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

    let mut sum_lum = 0.0f32;
    for idx in (0..width * height * 4).step_by(4) {
        let r0 = orig[idx] as f32 / 255.0;
        let g0 = orig[idx + 1] as f32 / 255.0;
        let b0 = orig[idx + 2] as f32 / 255.0;
        sum_lum += 0.299 * r0 + 0.587 * g0 + 0.114 * b0;
    }
    let avg_lum = sum_lum / (width * height) as f32;

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
                    let ratio = if lum > 1e-4 { target_lum / lum } else { 1.0 };
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
                    // Gentle macro-gradient detrending to eliminate repetitive tiling blotches
                    let blur_lum = 0.299 * blur[0] + 0.587 * blur[1] + 0.114 * blur[2];
                    let detrend = (avg_lum - blur_lum) * 0.40;
                    r = (r + 0.58 * high_pass[0] + detrend).clamp(0.0, 1.0);
                    g = (g + 0.58 * high_pass[1] + detrend).clamp(0.0, 1.0);
                    b = (b + 0.58 * high_pass[2] + detrend).clamp(0.0, 1.0);

                    // 90s arcade meadow green hue, vibrant and balanced
                    g = (g * 1.15).min(1.0);
                    r = (r * 0.92 + 0.03).clamp(0.0, 1.0);
                    b = (b * 0.85).clamp(0.0, 1.0);
                }
                TextureEnhance::Rock => {
                    // Dual-phase seamless blend to break the prominent continuous horizontal crack in rock.jpg
                    let half_w = width / 2;
                    let half_h = height / 2;
                    let ox = (x + half_w) % width;
                    let oy = (y + half_h) % height;
                    let phase2 = get_pixel(ox, oy);

                    // Organic periodic blending weight that tiles seamlessly across 512x512
                    let fx = x as f32 / width as f32 * std::f32::consts::TAU;
                    let fy = y as f32 / height as f32 * std::f32::consts::TAU;
                    let harmonic = (fx * 2.0).sin() * (fy * 3.0).cos() * 0.26
                        + (fx * 4.0 + fy * 2.0).sin() * 0.14;
                    let blend_w = (0.5 + harmonic).clamp(0.18, 0.82);

                    let base_r = center[0] * (1.0 - blend_w) + phase2[0] * blend_w;
                    let base_g = center[1] * (1.0 - blend_w) + phase2[1] * blend_w;
                    let base_b = center[2] * (1.0 - blend_w) + phase2[2] * blend_w;

                    // Wide-kernel macro detrending (17x17) to eliminate large repeating corner blotches
                    let mut macro_sum = [0.0f32; 3];
                    let step = (width / 16).max(8) as isize;
                    for mdy in [-step, 0, step] {
                        for mdx in [-step, 0, step] {
                            let msx = (x as isize + mdx).rem_euclid(width as isize) as usize;
                            let msy = (y as isize + mdy).rem_euclid(height as isize) as usize;
                            let mp = get_pixel(msx, msy);
                            for c in 0..3 {
                                macro_sum[c] += mp[c] / 9.0;
                            }
                        }
                    }
                    let macro_lum =
                        0.299 * macro_sum[0] + 0.587 * macro_sum[1] + 0.114 * macro_sum[2];
                    let detrend = (avg_lum - macro_lum) * 0.45;

                    // Crisp high-pass micro-relief on stone fissures without overpowering black bars
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    r = (base_r + 0.72 * high_pass[0] + detrend).clamp(0.0, 1.0);
                    g = (base_g + 0.72 * high_pass[1] + detrend).clamp(0.0, 1.0);
                    b = (base_b + 0.72 * high_pass[2] + detrend).clamp(0.0, 1.0);

                    // Luminous, warm Mediterranean limestone/granite tone curve
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let target_lum = ((lum - 0.48) * 1.16 + 0.48).clamp(0.0, 1.0);
                    let ratio = if lum > 1e-4 { target_lum / lum } else { 1.0 };
                    r = (r * ratio * 1.06 + 0.015).clamp(0.0, 1.0);
                    g = (g * ratio * 1.02 + 0.01).clamp(0.0, 1.0);
                    b = (b * ratio * 0.96).clamp(0.0, 1.0);
                }
                TextureEnhance::Sand => {
                    let high_pass = [
                        center[0] - blur[0],
                        center[1] - blur[1],
                        center[2] - blur[2],
                    ];
                    let blur_lum = 0.299 * blur[0] + 0.587 * blur[1] + 0.114 * blur[2];
                    let detrend = (avg_lum - blur_lum) * 0.35;
                    r = (r + 0.55 * high_pass[0] + detrend).clamp(0.0, 1.0);
                    g = (g + 0.55 * high_pass[1] + detrend).clamp(0.0, 1.0);
                    b = (b + 0.55 * high_pass[2] + detrend).clamp(0.0, 1.0);
                    // Warm, luminous sand
                    r = (r * 1.06).min(1.0);
                    g = (g * 1.02).min(1.0);
                    b = (b * 0.93).clamp(0.0, 1.0);
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

/// Textura de agua para el mar: azules y turquesas vivos mediterráneos con ondulaciones
/// suaves de olas armónicas y periódicas, sin saltos ni altas frecuencias para eliminar el muaré.
fn water() -> Image {
    const SIZE: u32 = 512;
    let inv_size = std::f32::consts::TAU / SIZE as f32;
    // Componentes armónicos estrictamente periódicos (frecuencias enteras):
    // garantizan continuidad matemática en los bordes de repetición (sin costuras)
    // y un oleaje amplio que previene el aliasing y el efecto muaré a la distancia.
    let waves = [
        (2.0, 1.0, 0.0f32, 0.35f32),
        (-1.0, 2.0, 1.3, 0.25),
        (3.0, -1.0, 2.6, 0.18),
        (-2.0, 3.0, 0.9, 0.12),
        (4.0, 2.0, 3.2, 0.07),
        (1.0, -4.0, 1.8, 0.03),
    ];
    let deep_abyss = Color::srgb(0.015, 0.09, 0.28).to_srgba();
    let ocean_blue = Color::srgb(0.03, 0.22, 0.44).to_srgba();
    let turquoise = Color::srgb(0.06, 0.42, 0.56).to_srgba();
    let wave_crest = Color::srgb(0.12, 0.58, 0.68).to_srgba();
    let sun_foam = Color::srgb(0.24, 0.66, 0.74).to_srgba();

    let data = (0..SIZE * SIZE)
        .flat_map(|i| {
            let (x, y) = ((i % SIZE) as f32, (i / SIZE) as f32);
            let u = x * inv_size;
            let v = y * inv_size;
            let mut sum = 0.0;
            for &(nx, ny, phase, weight) in &waves {
                sum += weight * (nx * u + ny * v + phase).sin();
            }
            let ripple = (0.5 + 0.5 * sum).clamp(0.0, 1.0);
            let shaped = ripple * ripple * (3.0 - 2.0 * ripple);

            let c = if shaped < 0.35 {
                let t = shaped / 0.35;
                deep_abyss.mix(&ocean_blue, t)
            } else if shaped < 0.70 {
                let t = (shaped - 0.35) / 0.35;
                ocean_blue.mix(&turquoise, t)
            } else if shaped < 0.90 {
                let t = (shaped - 0.70) / 0.20;
                turquoise.mix(&wave_crest, t)
            } else {
                let t = (shaped - 0.90) / 0.10;
                wave_crest.mix(&sun_foam, t)
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

/// An sRGB texture of `width` × `height` pixels that repeats both ways, with
/// its mipmaps, from the colour of each pixel.
fn tiling(width: u32, height: u32, pixel: impl Fn(u32, u32) -> Color) -> Image {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    let data = (0..width * height)
        .flat_map(|i| {
            let color = pixel(i % width, i / width).to_srgba();
            [byte(color.red), byte(color.green), byte(color.blue), 255]
        })
        .collect();
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
    image.sampler = sampler(ImageAddressMode::Repeat);
    generate_mipmaps(&mut image);
    image
}

/// Green Hill earth: two by two squares, chocolate and caramel, each with a
/// light bevel on its upper and left edges, a dark one on the others, and
/// some grain.
fn soil() -> Image {
    const SIZE: u32 = 256;
    const HALF: u32 = SIZE / 2;
    const BEVEL: u32 = 7;
    let dark = Color::srgb(0.42, 0.21, 0.07);
    let light = Color::srgb(0.80, 0.46, 0.16);
    tiling(SIZE, SIZE, |x, y| {
        let (px, py) = (x % HALF, y % HALF);
        let base = if (x < HALF) == (y < HALF) {
            light
        } else {
            dark
        };
        let point = Vec2::new(x as f32, y as f32);
        let grain = 0.08 * (fbm(point / 9.0, 3) - 0.5) + 0.05 * (value_noise(point / 2.0) - 0.5);
        let shade = if px < BEVEL || py < BEVEL {
            1.18
        } else if px >= HALF - BEVEL || py >= HALF - BEVEL {
            0.72
        } else {
            1.0
        };
        Color::LinearRgba(base.to_linear() * (shade + grain))
    })
}

/// Falling water: blue with white streaks of foam running down, which
/// repeat seamlessly in both directions.
fn falls() -> Image {
    const WIDTH: u32 = 128;
    const HEIGHT: u32 = 256;
    let deep = Color::srgb(0.12, 0.42, 0.78).to_linear();
    let foam = Color::srgb(0.92, 0.97, 1.0).to_linear();
    tiling(WIDTH, HEIGHT, |x, y| {
        let u = x as f32 / WIDTH as f32 * std::f32::consts::TAU;
        let v = y as f32 / HEIGHT as f32 * std::f32::consts::TAU;
        // Streaks: thin across, long down, wavering a little.
        let waver = 0.4 * (v * 2.0 + u * 3.0).sin();
        let streaks = 0.5
            + 0.25 * (u * 9.0 + waver).sin()
            + 0.15 * (u * 23.0 + 1.3 + (v * 3.0).sin()).sin()
            + 0.1 * (u * 5.0 - v * 2.0).sin();
        let t = ((streaks - 0.45) * 2.4).clamp(0.0, 1.0);
        Color::LinearRgba(deep.mix(&foam, t * t))
    })
}

/// Wooden planks laid across: eight boards, each of its own tone, with
/// dark gaps and grain along them.
fn planks() -> Image {
    const SIZE: u32 = 128;
    const BOARDS: u32 = 8;
    let wood = Color::srgb(0.58, 0.36, 0.18).to_linear();
    tiling(SIZE, SIZE, |x, y| {
        let board = y * BOARDS / SIZE;
        let within = (y * BOARDS) % SIZE;
        if within < BOARDS * 2 {
            return Color::srgb(0.16, 0.09, 0.04);
        }
        let tone = 0.82 + 0.3 * random01(board + 17);
        let point = Vec2::new(x as f32 / 14.0, y as f32 * 1.6 + board as f32 * 31.0);
        let grain = 0.85 + 0.3 * value_noise(point);
        Color::LinearRgba(wood * (tone * grain))
    })
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

    /// The colour of a pixel of the first mip level.
    fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
        let width = image.texture_descriptor.size.width;
        let at = ((y * width + x) * 4) as usize;
        let data = image.data.as_ref().expect("the texture has its pixels");
        [data[at], data[at + 1], data[at + 2], data[at + 3]]
    }

    #[test]
    fn checkerboard_earth_alternates_two_browns() {
        let soil = soil();
        let (light, dark) = (pixel(&soil, 64, 64), pixel(&soil, 192, 64));
        // Brown: more red than green, more green than blue.
        for brown in [light, dark] {
            assert!(brown[0] > brown[1] && brown[1] > brown[2], "{brown:?}");
        }
        assert!(light[0] > dark[0] + 40, "{light:?} {dark:?}");
        // Diagonally, the same square again.
        assert_eq!(pixel(&soil, 192, 192)[0] / 16, light[0] / 16);
    }

    #[test]
    fn falling_water_repeats_without_seams() {
        let falls = falls();
        let (width, height) = (
            falls.texture_descriptor.size.width,
            falls.texture_descriptor.size.height,
        );
        let step = |a: [u8; 4], b: [u8; 4]| (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap();
        // Across each seam, no bigger a step than between neighbours inside.
        for x in (0..width).step_by(7) {
            let inside = (1..height)
                .map(|y| step(pixel(&falls, x, y - 1), pixel(&falls, x, y)))
                .max()
                .unwrap();
            assert!(step(pixel(&falls, x, height - 1), pixel(&falls, x, 0)) <= inside);
        }
        for y in (0..height).step_by(7) {
            let inside = (1..width)
                .map(|x| step(pixel(&falls, x - 1, y), pixel(&falls, x, y)))
                .max()
                .unwrap();
            assert!(step(pixel(&falls, width - 1, y), pixel(&falls, 0, y)) <= inside);
        }
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
