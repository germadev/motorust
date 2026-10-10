//! El mundo alrededor de la carretera: el terreno, la carretera con lo que
//! la bordea y el decorado de la pista elegida ([`crate::circuits`]), bajo
//! un cielo de verano. Al cambiar de pista se vuelve a construir.

mod lettering;
mod props;
mod quads;
mod road;
pub mod scenery;
mod sky;
mod structures;
mod terrain;
mod textures;

use bevy::{light::CascadeShadowConfigBuilder, prelude::*, transform::TransformSystems};

use crate::circuits::CurrentCircuit;
use crate::track::Track;
pub use textures::generate_mipmaps;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let current = CurrentCircuit::default();
        app.insert_resource(current)
            .insert_resource(Track::new(&current.get().layout))
            .init_resource::<textures::WorldTextures>()
            .add_systems(Startup, (build_world, sky::spawn_sky, spawn_sun))
            .add_systems(PreUpdate, rebuild_world)
            .add_systems(
                PostUpdate,
                sky::follow_camera.before(TransformSystems::Propagate),
            );
    }
}

/// Todo lo que depende de la pista cuelga de esta entidad: al cambiar de
/// pista se borra entera.
#[derive(Component)]
struct WorldRoot;

/// Al cambiar de pista: fuera el mundo anterior, y el trazado y el mundo de
/// la nueva.
fn rebuild_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    roots: Query<Entity, With<WorldRoot>>,
    mut track: ResMut<Track>,
    builder: Builder,
) {
    if !current.is_changed() || current.is_added() {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    *track = Track::new(&current.get().layout);
    build(&mut commands, *current, &track, builder);
}

fn build_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    track: Res<Track>,
    builder: Builder,
) {
    build(&mut commands, *current, &track, builder);
}

/// Lo que hace falta para construir el mundo.
#[derive(bevy::ecs::system::SystemParam)]
struct Builder<'w> {
    textures: Res<'w, textures::WorldTextures>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

fn build(commands: &mut Commands, current: CurrentCircuit, track: &Track, mut builder: Builder) {
    let scenery = &current.get().scenery;
    let terrain = terrain::Terrain::new(track, scenery);
    let mut assets = Assets3d {
        textures: &builder.textures,
        meshes: &mut builder.meshes,
        materials: &mut builder.materials,
        images: &mut builder.images,
    };
    commands
        .spawn((WorldRoot, Transform::default(), Visibility::default()))
        .with_children(|world| {
            terrain::spawn_terrain(
                world,
                &terrain,
                assets.textures,
                assets.meshes,
                assets.materials,
            );
            road::spawn_road(world, track, &terrain, scenery, &mut assets);
            structures::spawn_structures(world, track, &terrain, scenery, &mut assets);
            props::spawn_props(world, track, &terrain, scenery, &mut assets);
        });
}

/// Dónde guardar las mallas, los materiales y las imágenes del mundo.
pub struct Assets3d<'a> {
    pub textures: &'a textures::WorldTextures,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub images: &'a mut Assets<Image>,
}

/// El sol, en el mismo sitio que en la foto del cielo, y la luz ambiente.
fn spawn_sun(mut commands: Commands) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.75, 0.82, 1.0),
        brightness: 700.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_to(-sky::sun_direction(), Vec3::Y),
        // Las sombras solo hacen falta cerca de la cámara; así tienen más resolución.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 25.0,
            maximum_distance: 250.0,
            ..default()
        }
        .build(),
    ));
}
