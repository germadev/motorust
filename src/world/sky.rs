//! The sky: a dome always centred on the camera, so it looks infinitely far
//! away: a plain gradient, deep blue overhead and pale towards the horizon
//! (the colour of the fog), the sun, and clouds round the horizon, drifting
//! slowly: heaped cumulus with flat grey bases and sunlit tops, and thin
//! veils of cirrus higher up.
//!
//! The clouds are painted once, at startup, into a few images: each is a
//! field of thickness (puffs heaped on a flat base, frayed by fractal
//! noise), lit by how much cloud the sunlight goes through to reach each
//! point. Every frame they cost a few quads that share their mesh, and
//! turning the one entity that holds them.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    light::NotShadowCaster,
    mesh::Indices,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    render::view::ColorGrading,
};

use super::generate_mipmaps;
use super::scenery::SkyStyle;
use crate::circuits::CurrentCircuit;
use crate::geometry::{fbm, smoothstep, value_noise};

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
/// Where the sun stands on Turbo Bay's Mediterranean afternoon (heading and
/// height, as above): high in the north-north-west, so the face of its dam,
/// which looks north down the gorge, is in the sun.
pub const MEDITERRANEAN_SUN: (f32, f32) = (1.9 * std::f32::consts::PI, 0.72);

/// The colours of the sky (sRGB): overhead, a little above the horizon, and
/// at the horizon, where it is the colour of the fog, so the land far away
/// melts into it.
const ZENITH: Color = Color::srgb(0.16, 0.42, 0.88);
const ABOVE_HORIZON: Color = Color::srgb(0.36, 0.62, 0.95);
pub const HORIZON: Color = Color::srgb(0.68, 0.8, 0.94);
/// Where the fog begins and where it hides everything (m from the camera):
/// on most days, and in a clear autumn afternoon's golden hour
/// (`SkyStyle::GoldenHour`), whose mountains stand out from across the sea.
pub const FOG: (f32, f32) = (450.0, 3800.0);
const GOLDEN_FOG: (f32, f32) = (2000.0, 6800.0);
/// The haze of high country: from nearer, and thicker far off, so the
/// hills behind fade one after another.
const HAZE: (f32, f32) = (320.0, 4200.0);
/// The haze of a Mediterranean summer: it begins nearer than on most days
/// and thickens slowly, so the island's far side is already veiled.
const MEDITERRANEAN_FOG: (f32, f32) = (300.0, 4200.0);

/// The light of a track ([`SkyStyle`]): the sky's colours, where the sun
/// is and the colour of its light, the light of the sky on the shade, and
/// the colour of the clouds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    zenith: Color,
    above_horizon: Color,
    pub horizon: Color,
    /// How much the horizon glows round the sun (0 to 1), and in what
    /// colour.
    glow: f32,
    glow_color: Color,
    sun_azimuth: f32,
    sun_elevation: f32,
    sun_disc: Color,
    sunlight: Color,
    /// The sunlight's illuminance, as a fraction of a clear day's.
    sunlight_strength: f32,
    ambient: Color,
    ambient_brightness: f32,
    clouds: Color,
    /// Where the fog begins and where it hides everything (m from the
    /// camera).
    fog: (f32, f32),
    /// How the picture is graded, as a film is: not at all but on a clear
    /// day (see [`Grade`]).
    grade: Grade,
}

/// A colour grade of the whole picture: how much saturation it keeps (after
/// tonemapping), how much contrast its midtones get, how many stops lighter
/// it is, how much warmer its white balance (a shift of the white point's
/// CIE *x*, so a few thousandths are already seen), and how much
/// saturation its highlights keep, as film loses it in the bright parts.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Grade {
    saturation: f32,
    contrast: f32,
    exposure: f32,
    warmth: f32,
    highlight_saturation: f32,
}

impl Grade {
    const NONE: Grade = Grade {
        saturation: 1.0,
        contrast: 1.0,
        exposure: 0.0,
        warmth: 0.0,
        highlight_saturation: 1.0,
    };

    /// The camera's colour grading for it.
    fn grading(self) -> ColorGrading {
        let mut grading = ColorGrading::default();
        grading.global.post_saturation = self.saturation;
        grading.global.exposure = self.exposure;
        grading.global.temperature = self.warmth;
        grading.midtones.contrast = self.contrast;
        grading.highlights.saturation = self.highlight_saturation;
        grading
    }
}

impl Palette {
    pub fn of(style: SkyStyle) -> Self {
        match style {
            // The last hours of a clear autumn afternoon, as in Japan: the
            // sun low in the west-south-west, golden, lighting the volcano
            // from the side and casting long shadows, the shade cool and
            // blue; the sky still blue overhead, turning lilac and then
            // reddish towards the horizon, glowing orange round the sun,
            // the clouds tinged pink, a light warm haze far off; and the
            // picture luminous, with contrast, but not saturated.
            SkyStyle::GoldenHour => Self {
                zenith: Color::srgb(0.24, 0.4, 0.72),
                above_horizon: Color::srgb(0.68, 0.62, 0.74),
                horizon: Color::srgb(0.95, 0.78, 0.68),
                glow: 0.6,
                glow_color: Color::srgb(1.0, 0.68, 0.48),
                sun_azimuth: 1.35 * std::f32::consts::PI,
                sun_elevation: 0.3,
                sun_disc: Color::srgb(1.0, 0.82, 0.6),
                sunlight: Color::srgb(1.0, 0.8, 0.6),
                sunlight_strength: 1.0,
                ambient: Color::srgb(0.62, 0.66, 0.84),
                ambient_brightness: 480.0,
                clouds: Color::srgb(1.0, 0.82, 0.74),
                fog: GOLDEN_FOG,
                grade: Grade {
                    saturation: 0.82,
                    contrast: 1.15,
                    exposure: 0.25,
                    ..Grade::NONE
                },
            },
            SkyStyle::Day => Self {
                zenith: ZENITH,
                above_horizon: ABOVE_HORIZON,
                horizon: HORIZON,
                glow: 0.0,
                glow_color: Color::WHITE,
                sun_azimuth: SUN_AZIMUTH,
                sun_elevation: SUN_ELEVATION,
                sun_disc: Color::WHITE,
                sunlight: Color::WHITE,
                sunlight_strength: 1.0,
                ambient: Color::srgb(0.75, 0.82, 1.0),
                ambient_brightness: 700.0,
                clouds: Color::WHITE,
                fog: FOG,
                grade: Grade::NONE,
            },
            // A bright late summer afternoon in high country: the sun well up
            // in the south-east, warm, lighting the fronts of the hills and the
            // faces of the rock, the shade filled by a clear sky; a quiet blue
            // overhead paling to a warm, light haze at the horizon, so that the
            // hills fade into it one behind another; white clouds, a touch
            // warm. Graded as a film is: the greens and blues a little held
            // back, a little more contrast, light and luminous.
            SkyStyle::HighCountry => Self {
                zenith: Color::srgb(0.24, 0.45, 0.8),
                above_horizon: Color::srgb(0.55, 0.69, 0.88),
                horizon: Color::srgb(0.86, 0.86, 0.84),
                glow: 0.25,
                glow_color: Color::srgb(1.0, 0.92, 0.76),
                sun_azimuth: SUN_AZIMUTH,
                sun_elevation: 0.62,
                sun_disc: Color::srgb(1.0, 0.97, 0.88),
                sunlight: Color::srgb(1.0, 0.94, 0.84),
                sunlight_strength: 1.15,
                ambient: Color::srgb(0.72, 0.78, 0.92),
                ambient_brightness: 720.0,
                clouds: Color::srgb(1.0, 0.98, 0.94),
                fog: HAZE,
                grade: Grade {
                    saturation: 0.88,
                    contrast: 1.06,
                    exposure: 0.25,
                    warmth: 0.004,
                    highlight_saturation: 0.9,
                },
            },
            // A clear Mediterranean summer afternoon, as a photograph shows
            // it rather than a cartoon: the sky a deep but not vivid blue
            // overhead, pale and milky towards the horizon, where the haze
            // off the sea whitens it, with a bright halo round the sun; a
            // little lower and warmer sun than on the plain day, in the
            // north-north-west (see [`MEDITERRANEAN_SUN`]), so the shadows
            // are longer and model the hills, with less light from
            // the sky to fill them, so they stay dark and cool; the haze
            // nearer, so the far side of the island and the mountains across
            // the sea fade into it; and the picture graded as film is, with
            // less saturation, a little more contrast in the midtones, the
            // highlights paler and the white balance a touch warm.
            SkyStyle::Mediterranean => Self {
                zenith: Color::srgb(0.2, 0.39, 0.72),
                above_horizon: Color::srgb(0.46, 0.62, 0.84),
                horizon: Color::srgb(0.77, 0.83, 0.89),
                glow: 0.22,
                glow_color: Color::srgb(1.0, 0.97, 0.9),
                sun_azimuth: MEDITERRANEAN_SUN.0,
                sun_elevation: MEDITERRANEAN_SUN.1,
                sun_disc: Color::srgb(1.0, 0.98, 0.94),
                sunlight: Color::srgb(1.0, 0.95, 0.88),
                sunlight_strength: 1.1,
                ambient: Color::srgb(0.66, 0.76, 1.0),
                ambient_brightness: 520.0,
                clouds: Color::srgb(1.0, 0.98, 0.95),
                fog: MEDITERRANEAN_FOG,
                grade: Grade {
                    saturation: 0.78,
                    contrast: 1.15,
                    exposure: 0.15,
                    warmth: 0.006,
                    highlight_saturation: 0.75,
                },
            },
            // The sun low in the west-north-west, in front on the start
            // straight: violet overhead, rose above the horizon and orange
            // at it, glowing towards the sun; warm, slanting light and
            // purple shade; peach clouds.
            SkyStyle::Dusk => Self {
                zenith: Color::srgb(0.17, 0.2, 0.46),
                above_horizon: Color::srgb(0.74, 0.5, 0.58),
                horizon: Color::srgb(0.98, 0.69, 0.47),
                glow: 0.65,
                glow_color: Color::srgb(1.0, 0.9, 0.62),
                sun_azimuth: 1.62 * std::f32::consts::PI,
                sun_elevation: 0.2,
                sun_disc: Color::srgb(1.0, 0.72, 0.42),
                sunlight: Color::srgb(1.0, 0.7, 0.48),
                sunlight_strength: 0.8,
                ambient: Color::srgb(0.72, 0.58, 0.82),
                ambient_brightness: 520.0,
                clouds: Color::srgb(1.0, 0.76, 0.66),
                fog: FOG,
                grade: Grade::NONE,
            },
        }
    }

    /// Where the sun is.
    pub fn sun_direction(&self) -> Vec3 {
        direction(self.sun_azimuth, self.sun_elevation)
    }

    /// The colour of the sky (sRGB) towards `toward`: by its height above
    /// the horizon, and brighter towards the sun.
    pub(crate) fn sky_color(&self, toward: Vec3) -> Color {
        let elevation = toward.y.clamp(-1.0, 1.0).asin();
        let low = smoothstep(0.0, 0.18, elevation);
        let high = smoothstep(0.12, 1.2, elevation);
        let mix = |a: Color, b: Color, t: f32| a.mix(&b, t);
        let gradient = mix(
            mix(self.horizon, self.above_horizon, low),
            self.zenith,
            high,
        );
        let near_sun = toward
            .normalize()
            .dot(self.sun_direction())
            .max(0.0)
            .powi(6);
        mix(gradient, self.glow_color, self.glow * near_sun)
    }
}

/// The sky's parts that change with the light of the track.
#[derive(Resource)]
pub struct SkyParts {
    dome: Handle<Mesh>,
    sun: Entity,
    sun_material: Handle<StandardMaterial>,
    /// Each cloud's material and how opaque it is.
    clouds: Vec<(Handle<StandardMaterial>, f32)>,
}

/// A cloud's place in the sky (m from the camera) and size (m).
#[derive(Component)]
pub struct CloudQuad {
    at: Vec3,
    size: Vec2,
}

/// Marks the sunlight.
#[derive(Component)]
pub struct Sunlight;

/// How many clouds there are, how far from the camera (m: inside the dome),
/// how high above the horizon (rad), how wide (m), and how fast they all
/// turn round the sky (rad/s).
const CLOUDS: usize = 22;
const CLOUD_DISTANCE: f32 = 4200.0;
const CLOUD_ELEVATION: (f32, f32) = (0.035, 0.24);
const CLOUD_WIDTH: (f32, f32) = (650.0, 1250.0);
const CLOUD_DRIFT: f32 = 0.004;
/// The different cloud shapes, and the size of their images (px).
const CLOUD_SHAPES: u32 = 10;
const CLOUD_IMAGE: UVec2 = UVec2::new(512, 256);
/// The lowest any cloud reaches down its image (fraction): below it the
/// image is clear.
const CLOUD_BOTTOM: f32 = 0.92;
/// How opaque a cloud gets per unit of thickness, and how much the
/// sunlight dims going through it (per unit of thickness, in units of the
/// image's height).
const CLOUD_DENSITY: f32 = 16.0;
const LIGHT_ABSORPTION: f32 = 22.0;
/// Where the sunlight comes from across the image: from above, a little to
/// the left (the quads of the clouds with the sun on their right are
/// mirrored).
const LIGHT_FROM: Vec2 = Vec2::new(-0.42, -0.91);
/// The colours of a cloud (sRGB): in full sunlight, in its own shadow and
/// at its base, shaded by the sky's blue.
const CLOUD_SUNLIT: [f32; 3] = [1.0, 0.99, 0.97];
const CLOUD_SHADOW: [f32; 3] = [0.56, 0.61, 0.71];
const CLOUD_BASE: [f32; 3] = [0.44, 0.49, 0.59];
/// Clouds near the horizon fade into the haze: how opaque they are at the
/// lowest and at the highest (fraction).
const CLOUD_HAZE: (f32, f32) = (0.72, 1.0);
/// Veils of cirrus higher up: how many, how high (rad), how wide (m) and
/// how opaque at most.
const CIRRUS: usize = 6;
const CIRRUS_ELEVATION: (f32, f32) = (0.3, 0.55);
const CIRRUS_WIDTH: (f32, f32) = (1800.0, 2600.0);
const CIRRUS_OPACITY: f32 = 0.55;
/// The sun: how big it looks (m, at the clouds' distance), and its image.
const SUN_SIZE: f32 = 520.0;
const SUN_IMAGE: u32 = 128;

#[derive(Component)]
pub struct Sky;

/// The clouds, turning slowly round the sky.
#[derive(Component)]
pub struct Clouds;

/// Dirección para un rumbo (0 = norte, -Z) y una altura sobre el horizonte.
pub(crate) fn direction(azimuth: f32, elevation: f32) -> Vec3 {
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
    let mut flat = |image: Option<Image>, alpha_mode: AlphaMode, color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            base_color_texture: image.map(|image| images.add(image)),
            alpha_mode,
            unlit: true,
            fog_enabled: false,
            cull_mode: None,
            double_sided: true,
            ..default()
        })
    };
    let dome_material = flat(None, AlphaMode::Opaque, Color::WHITE);
    let sun_material = flat(Some(sun_image()), AlphaMode::Blend, Color::WHITE);
    let cloud_images: Vec<Handle<Image>> = (0..CLOUD_SHAPES)
        .map(|shape| images.add(cloud_image(shape)))
        .collect();
    let cirrus_images: Vec<Handle<Image>> =
        (0..2).map(|veil| images.add(cirrus_image(veil))).collect();
    // Each cloud has its material: the lower, the more it fades into the
    // haze.
    let mut cloud = |image: &Handle<Image>, opacity: f32| {
        materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, opacity),
            base_color_texture: Some(image.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            fog_enabled: false,
            cull_mode: None,
            double_sided: true,
            ..default()
        })
    };
    let cumulus = cloud_places()
        .into_iter()
        .enumerate()
        .map(|(index, (at, size))| {
            let elevation = (at.y / at.length()).asin();
            let haze = smoothstep(CLOUD_ELEVATION.0, CLOUD_ELEVATION.1, elevation);
            let opacity = CLOUD_HAZE.0.lerp(CLOUD_HAZE.1, haze);
            (&cloud_images[index % cloud_images.len()], opacity, at, size)
        });
    let cirrus = cirrus_places()
        .into_iter()
        .enumerate()
        .map(|(index, (at, size))| {
            let opacity = CIRRUS_OPACITY * (0.6 + 0.4 * random(index as u32 + 40, 3));
            (
                &cirrus_images[index % cirrus_images.len()],
                opacity,
                at,
                size,
            )
        });
    let clouds: Vec<_> = cumulus
        .chain(cirrus)
        .map(|(image, opacity, at, size)| (cloud(image, opacity), opacity, at, size))
        .collect();
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let palette = Palette::of(SkyStyle::Day);
    let dome = meshes.add(dome(&palette));

    let mut sun = Entity::PLACEHOLDER;
    commands
        .spawn((
            Sky,
            Mesh3d(dome.clone()),
            MeshMaterial3d(dome_material),
            Transform::default(),
            Visibility::default(),
            NotShadowCaster,
        ))
        .with_children(|sky| {
            let at = palette.sun_direction() * CLOUD_DISTANCE;
            sun = sky
                .spawn((
                    Mesh3d(quad.clone()),
                    MeshMaterial3d(sun_material.clone()),
                    facing_the_camera(at, Vec2::splat(SUN_SIZE)),
                    NotShadowCaster,
                ))
                .id();
            sky.spawn((Clouds, Transform::default(), Visibility::default()))
                .with_children(|sky| {
                    for (material, _, at, size) in &clouds {
                        sky.spawn((
                            Mesh3d(quad.clone()),
                            MeshMaterial3d(material.clone()),
                            lit_from_the_sun(*at, *size, palette.sun_direction()),
                            CloudQuad {
                                at: *at,
                                size: *size,
                            },
                            NotShadowCaster,
                        ));
                    }
                });
        });
    commands.insert_resource(SkyParts {
        dome,
        sun,
        sun_material,
        clouds: clouds
            .into_iter()
            .map(|(material, opacity, ..)| (material, opacity))
            .collect(),
    });
}

/// A cloud's quad, facing the camera, mirrored if the sun (towards `sun`)
/// is on its right: the images are lit from the left.
fn lit_from_the_sun(at: Vec3, size: Vec2, sun: Vec3) -> Transform {
    let right = at.normalize().cross(Vec3::Y);
    let mirror = if right.dot(sun) > 0.0 { -1.0 } else { 1.0 };
    facing_the_camera(at, size * Vec2::new(mirror, 1.0))
}

/// Puts on the light of the track in use ([`SkyStyle`]): the sky's colours,
/// the sun and its light, the light of the sky, the clouds' colour, the fog
/// the colour of the horizon, and the picture's colour grade. Also for a
/// camera that has just been made.
#[allow(clippy::too_many_arguments)]
pub fn light_the_sky(
    current: Res<CurrentCircuit>,
    parts: Option<Res<SkyParts>>,
    mut fogs: Query<&mut DistanceFog>,
    mut gradings: Query<&mut ColorGrading>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut transforms: Query<&mut Transform>,
    clouds: Query<(Entity, &CloudQuad)>,
    mut sunlight: Query<(Entity, &mut DirectionalLight), With<Sunlight>>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
    clear: Option<ResMut<ClearColor>>,
) {
    let Some(parts) = parts else {
        return;
    };
    let palette = Palette::of(current.get().scenery.sky);
    let changed = current.is_changed() || parts.is_added();
    for mut fog in &mut fogs {
        if changed || fog.is_added() {
            fog.color = palette.horizon;
            fog.falloff = FogFalloff::Linear {
                start: palette.fog.0,
                end: palette.fog.1,
            };
        }
    }
    for mut grading in &mut gradings {
        if changed || grading.is_added() {
            *grading = palette.grade.grading();
        }
    }
    if !changed {
        return;
    }
    if let Some(mut clear) = clear {
        clear.0 = palette.horizon;
    }
    // A new dome: the old one's data is only on the GPU.
    if let Some(mut mesh) = meshes.get_mut(&parts.dome) {
        *mesh = dome(&palette);
    }
    let sun = palette.sun_direction();
    if let Ok(mut transform) = transforms.get_mut(parts.sun) {
        *transform = facing_the_camera(sun * CLOUD_DISTANCE, Vec2::splat(SUN_SIZE));
    }
    if let Some(mut material) = materials.get_mut(&parts.sun_material) {
        material.base_color = palette.sun_disc;
    }
    for (material, opacity) in &parts.clouds {
        if let Some(mut material) = materials.get_mut(material) {
            material.base_color = palette.clouds.with_alpha(*opacity);
        }
    }
    for (entity, cloud) in &clouds {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = lit_from_the_sun(cloud.at, cloud.size, sun);
        }
    }
    for (entity, mut light) in &mut sunlight {
        light.color = palette.sunlight;
        light.illuminance = DirectionalLight::default().illuminance * palette.sunlight_strength;
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = Transform::default().looking_to(-sun, Vec3::Y);
        }
    }
    if let Some(mut ambient) = ambient {
        ambient.color = palette.ambient;
        ambient.brightness = palette.ambient_brightness;
    }
}

/// A quad `size` m big at `at`, facing the camera at the centre of the sky.
fn facing_the_camera(at: Vec3, size: Vec2) -> Transform {
    Transform::from_translation(at)
        .looking_to(at.normalize(), Vec3::Y)
        .with_scale(size.extend(1.0))
}

/// Where each cloud is (m from the camera) and its size (m): spread round
/// the sky, each at its own height and size, as its kind is (see
/// [`CloudKind`]): torn shreds low down and small, towers and wide layers
/// big. Cloud `index` has shape `index % CLOUD_SHAPES`.
fn cloud_places() -> Vec<(Vec3, Vec2)> {
    let aspect = CLOUD_IMAGE.y as f32 / CLOUD_IMAGE.x as f32;
    (0..CLOUDS)
        .map(|index| {
            let seed = index as u32;
            let azimuth = (index as f32 + 0.8 * random(seed, 0)) / CLOUDS as f32 * TAU;
            let (low, size) = match CloudKind::of(seed % CLOUD_SHAPES) {
                CloudKind::Fractus => (true, (0.45, 0.7)),
                CloudKind::Cluster => (false, (0.75, 1.05)),
                CloudKind::Cumulus => (false, (0.6, 1.0)),
                CloudKind::Towering => (false, (0.9, 1.25)),
                CloudKind::Stratocumulus => (true, (0.95, 1.3)),
            };
            let height = random(seed, 1).powi(2) * if low { 0.45 } else { 1.0 };
            let elevation = CLOUD_ELEVATION.0.lerp(CLOUD_ELEVATION.1, height);
            let width = CLOUD_WIDTH.1 * size.0.lerp(size.1, random(seed, 2));
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

/// Where the veils of cirrus are (m from the camera) and their size (m):
/// high up, long and thin.
fn cirrus_places() -> Vec<(Vec3, Vec2)> {
    (0..CIRRUS)
        .map(|index| {
            let seed = index as u32 + 100;
            let azimuth = (index as f32 + 0.6 * random(seed, 0)) / CIRRUS as f32 * TAU;
            let elevation = CIRRUS_ELEVATION.0.lerp(CIRRUS_ELEVATION.1, random(seed, 1));
            let width = CIRRUS_WIDTH.0.lerp(CIRRUS_WIDTH.1, random(seed, 2));
            // Closer than the cumulus, so they stay inside the dome.
            (
                direction(azimuth, elevation) * (CLOUD_DISTANCE * 0.9),
                Vec2::new(width, width * 0.3),
            )
        })
        .collect()
}

/// What a cloud is like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloudKind {
    /// A fair-weather cumulus: a heap of puffs, higher in the middle, on a
    /// base that is flattish but uneven, curling up at its ends.
    Cumulus,
    /// A towering cumulus: a heap that rises in rounded towers.
    Towering,
    /// A stratocumulus: wide, low rolls side by side at their own heights,
    /// with a lumpy, ragged underside.
    Stratocumulus,
    /// A few small cumulus together, each on its own base.
    Cluster,
    /// Torn shreds of cloud, thin and ragged all round (fractus).
    Fractus,
}

impl CloudKind {
    /// The kind of cloud `shape`.
    fn of(shape: u32) -> Self {
        const KINDS: [CloudKind; CLOUD_SHAPES as usize] = [
            CloudKind::Cumulus,
            CloudKind::Towering,
            CloudKind::Stratocumulus,
            CloudKind::Cluster,
            CloudKind::Cumulus,
            CloudKind::Fractus,
            CloudKind::Towering,
            CloudKind::Cluster,
            CloudKind::Stratocumulus,
            CloudKind::Cumulus,
        ];
        KINDS[(shape % CLOUD_SHAPES) as usize]
    }
}

/// A puff of a cumulus: its centre and radius, in units of the image's
/// height (x from 0 to 2, y down from 0 to 1).
#[derive(Clone, Copy)]
struct Puff {
    center: Vec2,
    radius: f32,
}

/// A heap of puffs and where its underside is: about `base` down the image
/// in the middle, rising by `curl` at its ends, `half` either side of
/// `middle` (x).
struct Heap {
    puffs: Vec<Puff>,
    middle: f32,
    half: f32,
    base: f32,
    curl: f32,
}

/// A heap of `count` puffs `half` either side of `middle`, on a base
/// `base` down the image, as tall as `tall` says (1 for a usual cumulus),
/// with `towers` rising from its top. `seed` makes each one different.
fn heap(seed: u32, middle: f32, half: f32, base: f32, tall: f32, count: u32, towers: u32) -> Heap {
    let mut puffs = Vec::new();
    for puff in 0..count {
        let t = (puff as f32 + 0.5) / count as f32;
        let across = 2.0 * t - 1.0;
        let crown = 1.0 - across.abs();
        let x = middle + across * half + 0.08 * half * (random(seed + 3, puff) - 0.5);
        let radius =
            (0.07 + 0.25 * half * (0.4 + crown * tall)) * (0.75 + 0.5 * random(seed + 7, puff));
        // On the base, higher in the middle; those at the ends ride up with
        // its curl.
        let lift = 0.05 * across * across;
        let y = base - lift - radius * 0.55 - 0.2 * crown * tall * random(seed + 11, puff);
        puffs.push(Puff {
            center: Vec2::new(x, y),
            radius,
        });
    }
    // Smaller bubbles round the top of each puff: the lumpy outline of a
    // cumulus.
    let heaped = puffs.len();
    for puff in 0..heaped {
        for bubble in 0..3 {
            let k = (puff * 3 + bubble) as u32;
            let angle = (25.0 + 65.0 * bubble as f32 + 25.0 * random(seed + 29, k)).to_radians();
            let Puff { center, radius } = puffs[puff];
            let size = radius * (0.3 + 0.22 * random(seed + 31, k));
            let at = center + Vec2::new(angle.cos(), -angle.sin()) * (radius - size * 0.35);
            puffs.push(Puff {
                center: at,
                radius: size,
            });
        }
    }
    // Towers, one on another, leaning a little each way.
    let mut top = Vec2::new(middle, base - 0.3 * tall);
    for tower in 0..towers {
        let radius = (0.09 + 0.07 * random(seed + 19, tower)) * (1.0 - 0.12 * tower as f32);
        top += Vec2::new(
            0.18 * (random(seed + 17, tower) - 0.5),
            -radius * (0.9 + 0.4 * random(seed + 23, tower)),
        );
        for (dx, scale) in [(0.0, 1.0), (-0.7, 0.6), (0.7, 0.55)] {
            puffs.push(Puff {
                center: top + Vec2::new(dx * radius, 0.25 * radius * dx.abs()),
                radius: radius * scale,
            });
        }
    }
    Heap {
        puffs,
        middle,
        half,
        base,
        curl: 0.06 + 0.05 * random(seed + 37, 0),
    }
}

/// The heaps of cloud `shape`, as its kind is (see [`CloudKind`]).
fn heaps(shape: u32) -> Vec<Heap> {
    let r = |n: u32| random(shape + 101, n);
    match CloudKind::of(shape) {
        CloudKind::Cumulus => vec![heap(
            shape,
            1.0 + 0.1 * (r(0) - 0.5),
            0.55 + 0.15 * r(1),
            0.74 + 0.08 * r(2),
            0.8 + 0.3 * r(3),
            7 + shape % 3,
            1 + shape % 2,
        )],
        CloudKind::Towering => vec![heap(
            shape,
            1.0,
            0.45 + 0.1 * r(1),
            0.84 + 0.04 * r(2),
            1.4,
            7,
            3 + shape % 2,
        )],
        CloudKind::Stratocumulus => (0..3)
            .map(|k| {
                heap(
                    shape * 7 + k,
                    0.5 + 0.5 * k as f32 + 0.12 * (r(k) - 0.5),
                    0.36 + 0.1 * r(k + 3),
                    0.66 + 0.16 * r(k + 6),
                    0.35 + 0.15 * r(k + 9),
                    6,
                    0,
                )
            })
            .collect(),
        CloudKind::Cluster => (0..4)
            .map(|k| {
                heap(
                    shape * 7 + k,
                    0.32 + 0.45 * k as f32 + 0.15 * (r(k) - 0.5),
                    0.13 + 0.09 * r(k + 4),
                    0.5 + 0.35 * r(k + 8),
                    0.5 + 0.5 * r(k + 12),
                    4,
                    u32::from(r(k + 16) > 0.6),
                )
            })
            .collect(),
        CloudKind::Fractus => (0..3)
            .map(|k| {
                heap(
                    shape * 7 + k,
                    0.45 + 0.55 * k as f32 + 0.2 * (r(k) - 0.5),
                    0.22 + 0.12 * r(k + 4),
                    0.55 + 0.25 * r(k + 8),
                    0.25,
                    4,
                    0,
                )
            })
            .collect(),
    }
}

/// How thick cloud `shape` is through each point of its image (in units of
/// the image's height), row by row: the puffs of each heap as balls seen
/// through, heaped with a smooth union, frayed and made billowy by fractal
/// noise, and cut off softly along an uneven base, curling up at its ends.
/// Torn shreds are eaten away all over.
fn cloud_thickness(shape: u32) -> Vec<f32> {
    let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize);
    let heaps = heaps(shape);
    let kind = CloudKind::of(shape);
    let seed = Vec2::new(shape as f32 * 37.1, shape as f32 * 11.7);
    let mut thickness = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / height as f32;
            // Warped a little, so the outlines are cauliflower-like, not
            // round.
            let warp = Vec2::new(
                fbm(p * 7.0 + seed, 3) - 0.5,
                fbm(p * 7.0 + seed + 19.3, 3) - 0.5,
            ) * 0.07;
            let q = p + warp;
            // Billows: rounded lumps all over, the cauliflower look of a
            // cumulus, finer and finer.
            let mut billows = 0.0;
            let mut amplitude = 0.55;
            let mut frequency = 6.0;
            for octave in 0..3 {
                let n = value_noise(q * frequency + seed + Vec2::splat(octave as f32 * 7.7));
                billows += amplitude * (1.0 - 2.0 * (n - 0.5).abs());
                amplitude *= 0.5;
                frequency *= 2.1;
            }
            let mut t: f32 = 0.0;
            for heap in &heaps {
                // A smooth union of the balls: their thicknesses through q.
                let mut sum = 0.0;
                for puff in &heap.puffs {
                    let d2 = (q - puff.center).length_squared();
                    let r2 = puff.radius * puff.radius;
                    if d2 < r2 {
                        sum += (r2 - d2).sqrt().powi(4);
                    }
                }
                if sum <= 0.0 {
                    continue;
                }
                let lumps = sum.powf(0.25) * (0.85 + 0.2 * billows) - 0.03 * (1.0 - billows);
                // The underside: flattish in the middle, curling up at the
                // ends, wavy along and lumpy, its edge soft.
                let across = ((q.x - heap.middle) / heap.half).clamp(-1.5, 1.5);
                let wave = 0.035 * (fbm(Vec2::new(q.x * 4.0, 0.5) + seed, 3) - 0.5)
                    + 0.012 * (value_noise(Vec2::new(q.x * 30.0, q.y * 4.0) + seed) - 0.5);
                let base = heap.base - heap.curl * across * across + wave;
                let under = 1.0 - smoothstep(base - 0.03, base + 0.02, q.y);
                t = t.max(lumps * under);
            }
            t -= 0.012;
            if kind == CloudKind::Fractus {
                // Torn: holes and ragged shreds all over, and thin.
                let tatters = fbm(Vec2::new(q.x * 9.0, q.y * 5.0) + seed * 0.7, 3);
                t = (t - 0.04 * smoothstep(0.35, 0.75, tatters)) * 0.7;
            }
            // Nothing reaches the very bottom of the image.
            t *= 1.0 - smoothstep(CLOUD_BOTTOM - 0.04, CLOUD_BOTTOM, p.y);
            thickness.push(t.max(0.0));
        }
    }
    thickness
}

/// How far each point of a cloud is above its underside, column by column
/// (in units of the image's height): the run of cloud below it, down to
/// where the sky shows through.
fn depth_above_underside(thickness: &[f32], width: usize, height: usize) -> Vec<f32> {
    const CLEAR: f32 = 0.004;
    let mut above = vec![0.0; thickness.len()];
    for x in 0..width {
        let mut run = 0usize;
        for y in (0..height).rev() {
            let i = y * width + x;
            if thickness[i] < CLEAR {
                run = 0;
            } else {
                run += 1;
            }
            above[i] = run as f32 / height as f32;
        }
    }
    above
}

/// The image of cloud `shape`: as opaque as it is thick, white where the
/// sunlight reaches it through little cloud, grey-blue in its own shadow
/// and darker along its underside, with mipmaps (it is seen at many
/// sizes).
fn cloud_image(shape: u32) -> Image {
    let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize);
    let thickness = cloud_thickness(shape);
    let above = depth_above_underside(&thickness, width, height);
    let at = |x: f32, y: f32| {
        let (x, y) = (x.round() as isize, y.round() as isize);
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            0.0
        } else {
            thickness[y as usize * width + x as usize]
        }
    };
    // Steps towards the sun, in pixels.
    const MARCH: usize = 14;
    let step = LIGHT_FROM.normalize() * (height as f32 * 0.02);
    // The light as it falls on the lumps of the cloud: from the sun's
    // side, and a little from in front.
    let light = LIGHT_FROM.normalize().extend(0.55).normalize();
    let mut data = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let t = thickness[y * width + x];
            // How much cloud the sunlight goes through to get here.
            let mut through = 0.0;
            for k in 1..=MARCH {
                let p = Vec2::new(x as f32, y as f32) + step * k as f32;
                through += at(p.x, p.y);
            }
            through *= step.length() / height as f32;
            let sunlit = (-LIGHT_ABSORPTION * through).exp();
            // The lumps, as if the thickness were their height: the sides
            // facing the sun catch it, the folds are in shade.
            let (fx, fy) = (x as f32, y as f32);
            const REACH: f32 = 4.0;
            let slope = Vec2::new(
                at(fx + REACH, fy) - at(fx - REACH, fy),
                at(fx, fy + REACH) - at(fx, fy - REACH),
            ) * (height as f32 / (2.0 * REACH));
            let normal = (-slope * 0.6).extend(1.0).normalize();
            let facing = normal.dot(light).max(0.0);
            let lit = (sunlit.powf(0.8) * (0.35 + 0.8 * facing)).clamp(0.0, 1.0);
            // The underside, in the shade of the cloud over it, the darker
            // the more cloud there is above.
            let underside = 1.0 - smoothstep(0.0, 0.1, above[y * width + x]);
            let base = underside * (1.0 - sunlit * 0.5);
            let color: [f32; 3] = std::array::from_fn(|c| {
                let shade = CLOUD_SHADOW[c].lerp(CLOUD_BASE[c], base);
                shade.lerp(CLOUD_SUNLIT[c], lit * (1.0 - 0.6 * base))
            });
            let alpha = 1.0 - (-CLOUD_DENSITY * t).exp();
            data.extend(color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8));
            data.push((alpha * 255.0).round() as u8);
        }
    }
    clear_borders(&mut data, width, height);
    smooth_image(width as u32, height as u32, data)
}

/// A veil of cirrus: long, thin streaks combed by the wind, white and
/// faint, fading out towards the edges of the image.
fn cirrus_image(veil: u32) -> Image {
    let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize / 2);
    let seed = Vec2::splat(veil as f32 * 53.7 + 5.0);
    let mut data = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / height as f32;
            // Combed out along the wind: long fibres, bent only a little,
            // in a few bands.
            let bend = (fbm(Vec2::new(p.x * 0.8, 0.0) + seed, 2) - 0.5) * 0.25;
            let row = p.y + bend + 0.08 * p.x;
            let bands = smoothstep(0.35, 0.75, fbm(Vec2::new(p.x * 0.9, row * 3.0) + seed, 3));
            let fibres = fbm(Vec2::new(p.x * 0.6, row * 40.0) + seed * 1.3, 3);
            let wisps = bands * smoothstep(0.35, 0.85, fibres);
            let edge = Vec2::new(p.x / (width as f32 / height as f32), p.y);
            let fade = smoothstep(0.0, 0.25, edge.x)
                * smoothstep(0.0, 0.25, 1.0 - edge.x)
                * smoothstep(0.0, 0.35, edge.y)
                * smoothstep(0.0, 0.35, 1.0 - edge.y);
            let alpha = wisps * fade;
            data.extend([250, 251, 255, (alpha * 255.0).round() as u8]);
        }
    }
    clear_borders(&mut data, width, height);
    smooth_image(width as u32, height as u32, data)
}

/// Makes the outermost pixels of an image clear, so no cloud is cut off
/// at the edge of its quad.
fn clear_borders(data: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                data[(y * width + x) * 4 + 3] = 0;
            }
        }
    }
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

/// The dome's colours, vertex by vertex (see [`dome`]).
fn dome_colors(palette: &Palette) -> Vec<[f32; 4]> {
    dome_directions()
        .map(|toward| palette.sky_color(toward).to_linear().to_f32_array())
        .collect()
}

/// Which way each vertex of the dome is, ring by ring from below the
/// horizon to the zenith.
fn dome_directions() -> impl Iterator<Item = Vec3> {
    (0..=RINGS).flat_map(|ring| {
        let elevation = LOWEST_ELEVATION.lerp(FRAC_PI_2, ring as f32 / RINGS as f32);
        (0..=SEGMENTS).map(move |segment| {
            let fraction = segment as f32 / SEGMENTS as f32;
            direction(fraction * TAU, elevation)
        })
    })
}

/// The dome, coloured from the horizon to the zenith.
fn dome(palette: &Palette) -> Mesh {
    let positions: Vec<[f32; 3]> = dome_directions()
        .map(|toward| (toward * RADIUS).to_array())
        .collect();
    let normals: Vec<[f32; 3]> = dome_directions()
        .map(|toward| (-toward).to_array())
        .collect();
    let colors = dome_colors(palette);
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
        let sun = Palette::of(SkyStyle::Day).sun_direction();
        assert!(sun.x > 0.0 && sun.z > 0.0, "sureste: +X y +Z");
        assert!(sun.y > 0.6);
    }

    #[test]
    fn the_sky_meets_the_fog_at_the_horizon() {
        for style in [
            SkyStyle::Day,
            SkyStyle::Day,
            SkyStyle::Mediterranean,
            SkyStyle::GoldenHour,
            SkyStyle::Dusk,
            SkyStyle::HighCountry,
        ] {
            let palette = Palette::of(style);
            // Away from the sun, the horizon is the colour of the fog.
            let away = -palette.sun_direction() * Vec3::new(1.0, 0.0, 1.0);
            let at = |elevation: f32| {
                let toward = away.normalize() * elevation.cos() + Vec3::Y * elevation.sin();
                palette.sky_color(toward)
            };
            assert_eq!(at(0.0), palette.horizon, "{style:?}");
            assert_eq!(at(-0.2), palette.horizon, "{style:?}");
            // Bluer and bluer going up.
            let blue = |elevation: f32| {
                let c = at(elevation).to_srgba();
                c.blue - c.red
            };
            assert!(blue(0.3) > blue(0.05), "{style:?}");
            assert!(blue(1.5) > blue(0.3), "{style:?}");
        }
    }

    #[test]
    fn at_dusk_the_sun_is_low_in_the_west_and_the_sky_glows_round_it() {
        let dusk = Palette::of(SkyStyle::Dusk);
        let sun = dusk.sun_direction();
        assert!(sun.x < -0.5 && sun.y > 0.1 && sun.y < 0.3, "{sun}");
        let horizon = |toward: Vec3| {
            let c = dusk
                .sky_color((toward * Vec3::new(1.0, 0.0, 1.0)).normalize())
                .to_srgba();
            c.red + c.green + c.blue
        };
        assert!(horizon(sun) > horizon(-sun) + 0.1);
        // Warm: the horizon is orange, not blue.
        let warm = dusk.horizon.to_srgba();
        assert!(warm.red > warm.blue + 0.3);
        // The day keeps its look.
        let day = Palette::of(SkyStyle::Day);
        assert_eq!(day.horizon, HORIZON);
        assert_eq!(day.sky_color(Vec3::Y), ZENITH);
        // The golden hour: its haze further off, but nearer than the sea's
        // edge; its sky less vivid than a summer day's; its sun low, but not
        // setting, and golden; its picture less saturated, with more
        // contrast and a little lighter. The other days are not graded.
        let clear = Palette::of(SkyStyle::GoldenHour);
        assert!(clear.fog.0 > day.fog.0 && clear.fog.1 > day.fog.1);
        assert!(clear.fog.1 < super::super::sea::SEA_EXTENT);
        let vivid = |c: Color| {
            let c = c.to_srgba();
            c.blue - c.red.min(c.green)
        };
        assert!(vivid(clear.sky_color(Vec3::Y)) < vivid(day.sky_color(Vec3::Y)));
        assert!(clear.sun_elevation < day.sun_elevation / 2.0);
        assert!(clear.sun_elevation > dusk.sun_elevation);
        let gold = clear.sunlight.to_srgba();
        assert!(gold.red > gold.blue + 0.25, "{gold:?}");
        assert!(clear.grade.saturation < 1.0 && clear.grade.contrast > 1.0);
        assert!(clear.grade.exposure > 0.0);
        assert_eq!(day.grade, Grade::NONE);
        assert_eq!(dusk.grade, Grade::NONE);
        // The Mediterranean summer: graded as a photograph, less saturated,
        // with more contrast, its highlights paler and a touch warm; its
        // sky less vivid than the plain day's and its horizon paler, its
        // haze nearer, but never over the sea's edge; its sun warm and
        // lower than the plain day's, with less light from the sky.
        let summer = Palette::of(SkyStyle::Mediterranean);
        assert!(summer.grade.saturation < 1.0 && summer.grade.contrast > 1.0);
        assert!(summer.grade.highlight_saturation < 1.0);
        assert!(summer.grade.warmth > 0.0 && summer.grade.warmth < 0.02);
        assert!(vivid(summer.sky_color(Vec3::Y)) < vivid(day.sky_color(Vec3::Y)));
        assert!(vivid(summer.horizon) < vivid(day.horizon));
        assert!(summer.fog.0 < day.fog.0);
        assert!(summer.fog.1 < super::super::sea::SEA_EXTENT);
        let warm = summer.sunlight.to_srgba();
        assert!(warm.red > warm.blue);
        assert!(summer.sun_elevation < day.sun_elevation);
        assert!(summer.ambient_brightness < day.ambient_brightness);
    }

    #[test]
    fn high_country_is_bright_hazy_and_graded_as_a_film() {
        // A late afternoon's sun, a little lower than a summer day's and
        // warm, but as strong; the shade filled by the sky; a quieter sky,
        // paler at the horizon; a haze from nearer; and a picture a little
        // less saturated than a summer day's, with a little more contrast,
        // and lighter.
        let day = Palette::of(SkyStyle::Day);
        let high = Palette::of(SkyStyle::HighCountry);
        assert!(high.sun_elevation > 0.5 && high.sun_elevation < day.sun_elevation);
        let sun = high.sunlight.to_srgba();
        assert!(sun.red > sun.blue + 0.1);
        assert!(high.sunlight_strength >= 1.0);
        assert!(high.ambient_brightness >= day.ambient_brightness);
        let vivid = |c: Color| {
            let c = c.to_srgba();
            c.blue - c.red.min(c.green)
        };
        assert!(vivid(high.sky_color(Vec3::Y)) < vivid(day.sky_color(Vec3::Y)));
        assert!(vivid(high.horizon) < vivid(day.horizon));
        assert!(high.fog.0 < day.fog.0);
        assert!(high.grade.saturation < 1.0 && high.grade.contrast > 1.0);
        assert!(high.grade.exposure > 0.0);
    }

    /// The lowest row of cloud `data` thicker than half in column `x`, if
    /// there is one.
    fn underside(data: &[u8], x: usize) -> Option<usize> {
        let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize);
        (0..height)
            .rev()
            .find(|&y| data[(y * width + x) * 4 + 3] > 128)
    }

    #[test]
    fn clouds_are_heaps_on_soft_uneven_bases() {
        let (width, height) = (CLOUD_IMAGE.x as usize, CLOUD_IMAGE.y as usize);
        for shape in 0..CLOUD_SHAPES {
            let image = cloud_image(shape);
            let data = image.data.as_ref().unwrap();
            let alpha = |x: usize, y: usize| data[(y * width + x) * 4 + 3];
            // Some of it is solid, if not a torn shred.
            let thickest = (0..width * height).map(|i| data[i * 4 + 3]).max().unwrap();
            let solid = if CloudKind::of(shape) == CloudKind::Fractus {
                100
            } else {
                200
            };
            assert!(thickest > solid, "{shape}: {thickest}");
            // Clear all round the edges, so no cloud is cut off, and below
            // the lowest base.
            assert!((0..width).all(|x| alpha(x, 0) == 0));
            assert!((0..height).all(|y| alpha(0, y) == 0 && alpha(width - 1, y) == 0));
            let bottom = (CLOUD_BOTTOM * height as f32) as usize;
            assert!((0..width).all(|x| alpha(x, bottom + 1) == 0), "{shape}");
            // Not cut off along a straight line: its underside goes up and
            // down.
            let undersides: Vec<usize> = (0..width).filter_map(|x| underside(data, x)).collect();
            let (low, high) = (
                undersides.iter().max().unwrap(),
                undersides.iter().min().unwrap(),
            );
            assert!(low - high > 12, "{shape}: from {high} to {low}");
        }
    }

    #[test]
    fn clouds_are_sunlit_on_top_and_blue_grey_underneath() {
        let width = CLOUD_IMAGE.x as usize;
        for shape in (0..CLOUD_SHAPES).filter(|&s| CloudKind::of(s) != CloudKind::Fractus) {
            let image = cloud_image(shape);
            let data = image.data.as_ref().unwrap();
            let alpha = |x: usize, y: usize| data[(y * width + x) * 4 + 3];
            let brightness = |x: usize, y: usize| {
                let i = (y * width + x) * 4;
                data[i] as u32 + data[i + 1] as u32 + data[i + 2] as u32
            };
            // The column through the thickest part of the cloud.
            let x = (0..width)
                .max_by_key(|&x| {
                    (0..CLOUD_IMAGE.y as usize)
                        .filter(|&y| alpha(x, y) > 200)
                        .count()
                })
                .unwrap();
            let bottom = underside(data, x).expect("solid somewhere");
            let top = (0..bottom).find(|&y| alpha(x, y) > 128).unwrap();
            assert!(
                brightness(x, top + 2) > brightness(x, bottom - 2) + 60,
                "{shape}"
            );
            let i = ((bottom - 2) * width + x) * 4;
            assert!(data[i + 2] > data[i], "{shape}: the underside is blue-grey");
        }
    }

    #[test]
    fn the_sky_has_clouds_of_every_kind_and_size() {
        let kinds: Vec<CloudKind> = (0..CLOUD_SHAPES).map(CloudKind::of).collect();
        for kind in [
            CloudKind::Cumulus,
            CloudKind::Towering,
            CloudKind::Stratocumulus,
            CloudKind::Cluster,
            CloudKind::Fractus,
        ] {
            assert!(kinds.contains(&kind), "{kind:?}");
        }
        let widths: Vec<f32> = cloud_places().iter().map(|(_, size)| size.x).collect();
        let (narrow, wide) = widths
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &w| (a.min(w), b.max(w)));
        assert!(wide > 2.0 * narrow, "{narrow} to {wide}");
    }

    #[test]
    fn cloud_edges_are_soft_and_ragged() {
        // Not a hard cut-out: plenty of half-clear pixels round the cloud.
        let image = cloud_image(0);
        let data = image.data.as_ref().unwrap();
        let partial = data.chunks(4).filter(|p| (20..230).contains(&p[3])).count();
        let solid = data.chunks(4).filter(|p| p[3] >= 230).count();
        assert!(partial * 10 > solid, "{partial} soft, {solid} solid");
    }

    #[test]
    fn cirrus_are_faint_streaks_high_up() {
        let image = cirrus_image(0);
        let data = image.data.as_ref().unwrap();
        let alphas: Vec<u8> = data.chunks(4).map(|p| p[3]).collect();
        assert!(alphas.iter().any(|&a| a > 100));
        let clear = alphas.iter().filter(|&&a| a < 10).count();
        assert!(clear * 3 > alphas.len(), "mostly sky between the streaks");
        for (at, size) in cirrus_places() {
            let corner = (at.length_squared() + (size / 2.0).length_squared()).sqrt();
            assert!(corner < RADIUS);
            let elevation = (at.y / at.length()).asin();
            assert!(elevation > CLOUD_ELEVATION.1);
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
