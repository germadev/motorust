//! The render scale (Options > Graphics): the world drawn at less than the
//! window's pixels and scaled up to it, the interface over it at the
//! window's own. Under 100 %, the 3D cameras draw into an image that size,
//! and the interface's camera, which draws over the whole window, first
//! shows it as big as the window ([`ScaledWorld`], smoothed as it is
//! stretched) and sharpens it (Bevy's contrast adaptive sharpening, the
//! sharpening half of AMD's FSR 1), before the interface goes on top,
//! unsharpened. At 100 % the 3D cameras draw on the window, as they always
//! have, and the interface's camera over what they drew.
//!
//! A change of scale makes no new pipeline for the world, only textures of
//! another size (the sharpening's pipeline, and the interface's without
//! multisampling, are made the first time), so it can be changed in a race.

use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::camera::{CameraUpdateSystems, ImageRenderTarget, RenderTarget};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::ui::IsDefaultUiCamera;
use bevy::window::{PrimaryWindow, WindowRef};

use crate::settings::{DEFAULT_RENDER_SCALE, Settings};

pub struct RenderScalePlugin;

impl Plugin for RenderScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scaled_world)
            .add_systems(PostUpdate, scale_the_world.before(CameraUpdateSystems));
    }
}

/// How much the world drawn smaller is sharpened as it is scaled up (Bevy's
/// `sharpening_strength`, 0 to 1; 0.6 is its own).
const SHARPENING: f32 = 0.5;
/// The smallest share of the window's resolution the world is drawn at (%),
/// whatever the settings say.
const LEAST: u32 = 25;

/// The pixels the world is drawn at in a window of `window` physical pixels
/// at `percent` % of its resolution: that share of each side, one pixel at
/// least; all of them at 100 %.
pub fn render_size(window: UVec2, percent: u32) -> UVec2 {
    if percent >= 100 {
        return window;
    }
    let share = percent.max(LEAST) as f32 / 100.0;
    (window.as_vec2() * share)
        .round()
        .as_uvec2()
        .max(UVec2::ONE)
}

/// The share of the window's resolution the world is drawn at (%), as the
/// settings say if there are any.
pub fn render_percent(settings: Option<&Settings>) -> u32 {
    settings.map_or(DEFAULT_RENDER_SCALE, |settings| settings.render_scale)
}

/// The world drawn smaller, shown over the whole window by the interface's
/// camera (the only 2D one): its image is what the 3D cameras draw into.
#[derive(Component)]
struct ScaledWorld;

fn spawn_scaled_world(mut commands: Commands) {
    commands.spawn((ScaledWorld, Sprite::default(), Visibility::Hidden));
}

/// An image `size` pixels big for the 3D cameras to draw into, smoothed
/// when it is stretched over the window.
fn target_image(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None);
    // Drawn into, never filled from here.
    image.data = None;
    image.sampler = ImageSampler::linear();
    image
}

/// The world drawn at the render scale the settings say: into an image that
/// size, shown and sharpened by the interface's camera, or, at 100 %, on
/// the window.
#[allow(clippy::type_complexity)]
fn scale_the_world(
    mut commands: Commands,
    settings: Option<Res<Settings>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Query<(&mut Sprite, &mut Visibility), With<ScaledWorld>>,
    mut worlds: Query<(&mut RenderTarget, &mut Projection), With<Camera3d>>,
    mut interface: Query<
        (Entity, &mut Camera, Has<ContrastAdaptiveSharpening>),
        (With<IsDefaultUiCamera>, Without<Camera3d>),
    >,
) {
    let (Ok(window), Ok((mut sprite, mut visibility))) = (window.single(), shown.single_mut())
    else {
        return;
    };
    let full = window.physical_size();
    let size = render_size(full, render_percent(settings.as_deref()));
    let scaled = size != full;
    if scaled {
        // An image of the size wanted: a new one when the size changes, so
        // that the cameras drawing into it take its size in the same frame.
        if images
            .get(&sprite.image)
            .is_none_or(|image| image.size() != size)
        {
            sprite.image = images.add(target_image(size));
        }
        let covers = Some(window.size());
        if sprite.custom_size != covers {
            sprite.custom_size = covers;
        }
    }
    visibility.set_if_neq(if scaled {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    for (mut target, mut projection) in &mut worlds {
        let right = match &*target {
            RenderTarget::Image(image) => scaled && image.handle == sprite.image,
            RenderTarget::Window(_) => !scaled,
            _ => false,
        };
        if right {
            continue;
        }
        *target = if scaled {
            RenderTarget::Image(ImageRenderTarget {
                handle: sprite.image.clone(),
                scale_factor: 1.0,
            })
        } else {
            RenderTarget::Window(WindowRef::Primary)
        };
        // Bevy sizes a camera again when its window, its image or its
        // projection changes, not its target.
        projection.set_changed();
    }
    for (camera, mut settings, sharpened) in &mut interface {
        // Scaled, it draws the world first, over the whole window: it
        // clears it. Otherwise it draws over what the world's cameras drew.
        let clears = !matches!(settings.clear_color, ClearColorConfig::None);
        if clears != scaled {
            settings.clear_color = if scaled {
                ClearColorConfig::Default
            } else {
                ClearColorConfig::None
            };
        }
        if scaled && !sharpened {
            commands.entity(camera).insert(ContrastAdaptiveSharpening {
                enabled: true,
                sharpening_strength: SHARPENING,
                denoise: false,
            });
        } else if !scaled && sharpened {
            commands
                .entity(camera)
                .remove::<ContrastAdaptiveSharpening>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_world_is_drawn_at_a_share_of_each_side_of_the_window() {
        let retina = UVec2::new(3024, 1898);
        assert_eq!(render_size(retina, 100), retina);
        assert_eq!(render_size(retina, 70), UVec2::new(2117, 1329));
        assert_eq!(render_size(retina, 50), UVec2::new(1512, 949));
        // Never more than the window, never less than a quarter of each
        // side, never nothing.
        assert_eq!(render_size(retina, 150), retina);
        assert_eq!(render_size(retina, 5), render_size(retina, LEAST));
        assert_eq!(render_size(UVec2::ZERO, 50), UVec2::ONE);
        assert_eq!(render_percent(None), 100);
    }

    #[test]
    fn under_100_percent_the_world_is_drawn_into_an_image_shown_over_the_window() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<Image>()
            .add_plugins(RenderScalePlugin);
        let mut window = Window::default();
        window.resolution.set_physical_resolution(2000, 1000);
        app.world_mut().spawn((window, PrimaryWindow));
        let world = app.world_mut().spawn(Camera3d::default()).id();
        let interface = app
            .world_mut()
            .spawn((
                Camera2d,
                Camera {
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                IsDefaultUiCamera,
            ))
            .id();
        let mut settings = Settings {
            render_scale: 70,
            ..default()
        };
        app.insert_resource(settings.clone());
        app.update();
        let mut sprites = app.world_mut().query::<(&Sprite, &Visibility)>();
        let target = app.world().get::<RenderTarget>(world).unwrap();
        let RenderTarget::Image(drawn) = target else {
            panic!("{target:?}");
        };
        let images = app.world().resource::<Assets<Image>>();
        assert_eq!(
            images.get(&drawn.handle).unwrap().size(),
            UVec2::new(1400, 700)
        );
        let (sprite, visibility) = sprites.single(app.world()).unwrap();
        assert_eq!(sprite.image, drawn.handle);
        assert_eq!(*visibility, Visibility::Inherited);
        assert!(
            app.world()
                .get::<ContrastAdaptiveSharpening>(interface)
                .is_some()
        );
        let camera = app.world().get::<Camera>(interface).unwrap();
        assert!(matches!(camera.clear_color, ClearColorConfig::Default));
        // Back at 100 %: on the window, as always.
        settings.render_scale = 100;
        app.insert_resource(settings);
        app.update();
        let target = app.world().get::<RenderTarget>(world).unwrap();
        assert!(matches!(target, RenderTarget::Window(_)), "{target:?}");
        assert!(
            app.world()
                .get::<ContrastAdaptiveSharpening>(interface)
                .is_none()
        );
        let camera = app.world().get::<Camera>(interface).unwrap();
        assert!(matches!(camera.clear_color, ClearColorConfig::None));
        let (_, visibility) = sprites.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
    }
}
