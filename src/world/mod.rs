//! El mundo alrededor de la carretera: el terreno, la carretera con lo que
//! la bordea y el decorado de la pista elegida ([`crate::circuits`]), bajo
//! un cielo de verano. Al cambiar de pista se vuelve a construir.

mod landmarks;
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

use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::track::Track;
pub use textures::generate_mipmaps;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let current = CurrentCircuit::default();
        app.insert_resource(current)
            .insert_resource(Track::new(&current.get().layout))
            .init_resource::<TrackEdges>()
            .init_resource::<textures::WorldTextures>()
            .add_systems(Startup, (build_world, sky::spawn_sky, spawn_sun))
            .add_systems(PreUpdate, rebuild_world)
            .add_systems(Update, (landmarks::spin, landmarks::flow))
            .add_systems(OnEnter(GameState::Tracks), cache_all_circuits)
            .add_systems(OnExit(GameState::Tracks), unload_unused_circuits)
            .add_systems(
                PostUpdate,
                sky::follow_camera.before(TransformSystems::Propagate),
            );
    }
}

/// Where the solid edge of each circuit built so far is (see
/// [`Track::set_edges`]), by its index in [`CIRCUITS`]: it comes from the
/// terrain, which is only built once.
#[derive(Resource, Default)]
struct TrackEdges(std::collections::HashMap<usize, Vec<[f32; 2]>>);

/// Cada circuito cargado tiene su entidad raíz con su índice en [`CIRCUITS`].
#[derive(Component, Debug, PartialEq, Eq)]
pub struct WorldRoot(pub usize);

/// Al entrar al menú de elección de pistas, precalcula en segundo plano las
/// pistas que falten por cargar para que alternar entre ellas sea instantáneo.
fn cache_all_circuits(
    mut commands: Commands,
    roots: Query<&WorldRoot>,
    current: Res<CurrentCircuit>,
    mut builder: Builder,
    mut edges: ResMut<TrackEdges>,
) {
    let built: std::collections::HashSet<usize> = roots.iter().map(|r| r.0).collect();
    for (index, circuit) in CIRCUITS.iter().enumerate() {
        if !built.contains(&index) {
            let track = Track::new(&circuit.layout);
            let visibility = if index == current.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            let built = build(&mut commands, index, &track, &mut builder, visibility);
            edges.0.insert(index, built);
        }
    }
}

/// Al salir del menú de pistas (hacia elegir moto o volver a bienvenida),
/// se descargan los circuitos en caché que no se estén usando para liberar memoria.
fn unload_unused_circuits(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    roots: Query<(Entity, &WorldRoot)>,
) {
    for (entity, root) in &roots {
        if root.0 != current.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Al cambiar de pista: actualiza el trazado y conmuta la visibilidad entre
/// las pistas cargadas en caché (o la construye si no estuviera).
fn rebuild_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut roots: Query<(Entity, &WorldRoot, &mut Visibility)>,
    mut track: ResMut<Track>,
    mut builder: Builder,
    mut edges: ResMut<TrackEdges>,
) {
    if !current.is_changed() || current.is_added() {
        return;
    }
    *track = Track::new(&current.get().layout);
    if let Some(built) = edges.0.get(&current.0) {
        track.set_edges(built.clone());
    }
    let mut found = false;
    for (_entity, root, mut visibility) in &mut roots {
        if root.0 == current.0 {
            *visibility = Visibility::Inherited;
            found = true;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    if !found {
        let built = build(
            &mut commands,
            current.0,
            &track,
            &mut builder,
            Visibility::Inherited,
        );
        track.set_edges(built.clone());
        edges.0.insert(current.0, built);
    }
}

fn build_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut track: ResMut<Track>,
    mut builder: Builder,
    mut edges: ResMut<TrackEdges>,
) {
    let built = build(
        &mut commands,
        current.0,
        &track,
        &mut builder,
        Visibility::Inherited,
    );
    track.set_edges(built.clone());
    edges.0.insert(current.0, built);
}

/// Lo que hace falta para construir el mundo.
#[derive(bevy::ecs::system::SystemParam)]
struct Builder<'w> {
    textures: Res<'w, textures::WorldTextures>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

/// Builds a circuit's world, and returns where its solid edge is (see
/// [`Track::set_edges`]).
fn build(
    commands: &mut Commands,
    circuit_index: usize,
    track: &Track,
    builder: &mut Builder,
    visibility: Visibility,
) -> Vec<[f32; 2]> {
    let circuit = &CIRCUITS[circuit_index];
    let scenery = &circuit.scenery;
    let terrain = terrain::Terrain::new(track, scenery);
    let mut assets = Assets3d {
        textures: &builder.textures,
        meshes: &mut builder.meshes,
        materials: &mut builder.materials,
        images: &mut builder.images,
    };
    commands
        .spawn((WorldRoot(circuit_index), Transform::default(), visibility))
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
    terrain.track_edges(track)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuits_cache_on_tracks_menu_and_unload_on_exit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .init_state::<GameState>()
            .add_plugins(WorldPlugin);

        app.update();

        // Al inicio, solo la pista inicial (0) está cargada y visible.
        let mut roots_query = app.world_mut().query::<(&WorldRoot, &Visibility)>();
        let roots: Vec<_> = roots_query.iter(app.world()).collect();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0.0, 0);
        assert_eq!(*roots[0].1, Visibility::Inherited);

        // Entramos al menú de elección de pista: se deben cachear todas las pistas.
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Tracks);
        app.update();

        let roots: Vec<_> = roots_query.iter(app.world()).collect();
        assert_eq!(roots.len(), CIRCUITS.len());
        for (root, vis) in &roots {
            if root.0 == 0 {
                assert_eq!(*vis, Visibility::Inherited);
            } else {
                assert_eq!(*vis, Visibility::Hidden);
            }
        }

        // Cambiamos de pista a la 1: debe conmutar visibilidad instantáneamente.
        app.world_mut().resource_mut::<CurrentCircuit>().0 = 1;
        app.update();

        let roots: Vec<_> = roots_query.iter(app.world()).collect();
        assert_eq!(roots.len(), CIRCUITS.len());
        for (root, vis) in &roots {
            if root.0 == 1 {
                assert_eq!(*vis, Visibility::Inherited);
            } else {
                assert_eq!(*vis, Visibility::Hidden);
            }
        }

        // Salimos del menú de pistas hacia Select (elegir moto):
        // la pista no usada (0) debe descargarse y quedar solo la pista elegida (1).
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Select);
        app.update();

        let roots: Vec<_> = roots_query.iter(app.world()).collect();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0.0, 1);
        assert_eq!(*roots[0].1, Visibility::Inherited);
    }
}
