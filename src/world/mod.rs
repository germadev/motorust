//! El mundo alrededor de la carretera: el terreno, la carretera con lo que
//! la bordea y el decorado de la pista elegida ([`crate::circuits`]), bajo
//! un cielo de verano. Al cambiar de pista se vuelve a construir.

mod cavern;
mod desert;
mod foliage;
mod lamps;
mod landmarks;
mod lettering;
mod mine;
mod props;
mod quads;
mod road;
pub mod scenery;
mod sea;
mod shoulders;
pub mod sky;
mod structures;
mod terrain;
mod textures;
mod tufts;

use bevy::asset::embedded_asset;
use bevy::pbr::PbrPlugin;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder, block_on, poll_once};
use bevy::{
    light::{CascadeShadowConfigBuilder, NotShadowCaster},
    prelude::*,
    transform::TransformSystems,
};

use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::settings::WEB;
use crate::track::{Surface, Track};
pub use textures::generate_mipmaps;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sea.wgsl");
        // Without the renderer (in the tests), the sea's material is only an
        // asset.
        if app.is_plugin_added::<PbrPlugin>() {
            app.add_plugins(MaterialPlugin::<sea::SeaMaterial>::default());
        } else {
            app.init_asset::<sea::SeaMaterial>();
        }
        let current = CurrentCircuit::default();
        app.add_plugins(lamps::LampsPlugin);
        app.insert_resource(current)
            .insert_resource(Track::new(&current.get().layout))
            .init_resource::<Roadsides>()
            .init_resource::<Preparing>()
            .init_resource::<textures::WorldTextures>()
            .init_resource::<Sea>()
            .add_systems(Startup, (build_world, sky::spawn_sky, spawn_sun))
            .add_systems(PreUpdate, rebuild_world)
            .add_systems(
                Update,
                (
                    landmarks::spin,
                    landmarks::flow,
                    sky::drift_clouds,
                    sky::light_the_sky,
                    mine::roll_carts,
                    desert::pump,
                    desert::tumble,
                    spawn_prepared_circuit.run_if(in_state(GameState::Tracks)),
                ),
            )
            .add_systems(OnEnter(GameState::Tracks), cache_all_circuits)
            .add_systems(OnExit(GameState::Tracks), unload_unused_circuits)
            // A championship goes from track to track without choosing
            // them: the one left behind goes when the next race starts.
            .add_systems(OnEnter(GameState::Race), unload_unused_circuits)
            .add_systems(
                PostUpdate,
                sky::follow_camera.before(TransformSystems::Propagate),
            );
    }
}

/// What lies beside the roads of each circuit built so far (see
/// [`Roadside`]), by its index in [`CIRCUITS`]: it comes from the terrain,
/// which is only built once.
#[derive(Resource, Default)]
struct Roadsides(std::collections::HashMap<usize, Roadside>);

/// What lies beside the roads of a circuit: where their solid edge is (see
/// [`Track::set_edges`]) and what their shoulders are made of (see
/// [`Track::set_shoulders`]).
#[derive(Clone)]
struct Roadside {
    edges: Vec<Vec<[f32; 2]>>,
    shoulders: Vec<Vec<[Surface; 2]>>,
}

impl Roadside {
    /// Gives the circuit's track what lies beside its roads.
    fn apply(&self, track: &mut Track) {
        track.set_edges(self.edges.clone());
        track.set_shoulders(self.shoulders.clone());
    }
}

/// Cada circuito cargado tiene su entidad raíz con su índice en [`CIRCUITS`].
#[derive(Component, Debug, PartialEq, Eq)]
pub struct WorldRoot(pub usize);

/// The mesh of the sea, the same for every circuit.
#[derive(Resource)]
struct Sea(Handle<Mesh>);

impl FromWorld for Sea {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource_mut::<Assets<Mesh>>().add(sea::sea_mesh()))
    }
}

/// What can be worked out for a circuit away from the main thread, which
/// is most of the work: its track, its terrain, the terrain's meshes, where
/// its shore is and what lies beside its roads. The entities and assets are
/// made on the main thread.
struct Prepared {
    index: usize,
    track: Track,
    terrain: terrain::Terrain,
    ground: [(terrain::Ground, Mesh); 3],
    shore: sea::ShoreMap,
    shore_image: Image,
    roadside: Roadside,
    /// The meshes of the roads, tunnels and bridges, with their materials.
    pieces: Vec<(Mesh, StandardMaterial)>,
    /// The lamps of its tunnels and mine (see [`lamps`]).
    lamps: Vec<lamps::Lamp>,
    /// Its grass beside the roads, drawn only near the camera.
    grass: Vec<tufts::GrassPiece>,
    trees: props::TreePlan,
}

/// How many threads a circuit's terrain is worked out on: all the cores
/// when it is needed now, and few in the background, so the game and its
/// menus keep their frame rate while the other circuits are prepared.
#[derive(Clone, Copy)]
enum Urgency {
    Now,
    Background,
}

impl Urgency {
    fn threads(self) -> usize {
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        match self {
            Urgency::Now => cores,
            Urgency::Background => (cores / 2).saturating_sub(1).max(1),
        }
    }
}

fn prepare(index: usize, textures: &textures::WorldTextures, urgency: Urgency) -> Prepared {
    let circuit = &CIRCUITS[index];
    let scenery = &circuit.scenery;
    let track = Track::new(&circuit.layout);
    let terrain = terrain::Terrain::on_threads(&track, scenery, urgency.threads());
    let ground = terrain.meshes();
    let shore = sea::ShoreMap::new(&terrain, &track);
    let shore_image = shore.image();
    let roadside = Roadside {
        edges: terrain.track_edges(&track),
        shoulders: shoulders::track_shoulders(&track, &terrain, scenery),
    };
    let mut pieces = road::road_pieces(&track, &terrain, scenery, textures);
    let (structures, lamps) = structures::structure_pieces(&track, &terrain, scenery, textures);
    pieces.extend(structures);
    let grass = tufts::tuft_pieces(
        &track,
        &terrain,
        &roadside.shoulders,
        &roadside.edges,
        textures,
    );
    let trees = props::plan_trees(&track, &terrain, scenery);
    structures::draw_signs(scenery);
    Prepared {
        index,
        track,
        terrain,
        ground,
        shore,
        shore_image,
        roadside,
        pieces,
        lamps,
        grass,
        trees,
    }
}

/// The thread the other circuits are prepared on in the background: their
/// own, not Bevy's async compute pool. The render pipelines are compiled
/// there, and behind a circuit being prepared (seconds of work) they would
/// wait: nothing new would be drawn until it was ready, the screenshots
/// included.
fn circuit_pool() -> &'static TaskPool {
    static POOL: std::sync::OnceLock<TaskPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        TaskPoolBuilder::new()
            .num_threads(1)
            .thread_name("Circuits".into())
            .build()
    })
}

/// The circuits being prepared in the background, by index in [`CIRCUITS`].
#[derive(Resource, Default)]
struct Preparing(std::collections::HashMap<usize, Task<Prepared>>);

/// Al entrar al menú de elección de pistas, prepara en segundo plano las
/// pistas que falten por cargar para que alternar entre ellas sea
/// instantáneo. The menu shows up at once: the work goes on in other
/// threads, and each circuit is put in the world when it is ready
/// ([`spawn_prepared_circuit`]).
fn cache_all_circuits(
    roots: Query<&WorldRoot>,
    textures: Res<textures::WorldTextures>,
    mut preparing: ResMut<Preparing>,
) {
    let built: std::collections::HashSet<usize> = roots.iter().map(|r| r.0).collect();
    let pool = circuit_pool();
    for index in 0..CIRCUITS.len() {
        if !built.contains(&index) && !preparing.0.contains_key(&index) {
            let textures = textures.clone();
            preparing.0.insert(
                index,
                pool.spawn(async move { prepare(index, &textures, Urgency::Background) }),
            );
        }
    }
}

/// Puts in the world, hidden, a circuit prepared in the background (one
/// per frame: making its entities and meshes takes a little while too).
fn spawn_prepared_circuit(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut preparing: ResMut<Preparing>,
    mut builder: Builder,
    mut roadsides: ResMut<Roadsides>,
) {
    let mut ready = None;
    for (index, task) in &mut preparing.0 {
        if let Some(prepared) = block_on(poll_once(task)) {
            ready = Some((*index, prepared));
            break;
        }
    }
    let Some((index, prepared)) = ready else {
        return;
    };
    preparing.0.remove(&index);
    let visibility = if index == current.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let built = build(&mut commands, prepared, &mut builder, visibility);
    roadsides.0.insert(index, built);
}

/// Al salir del menú de pistas (hacia elegir moto o volver a bienvenida),
/// se descargan los circuitos en caché que no se estén usando para liberar
/// memoria, and the ones still being prepared are let go.
fn unload_unused_circuits(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    roots: Query<(Entity, &WorldRoot)>,
    mut preparing: ResMut<Preparing>,
) {
    for (entity, root) in &roots {
        if root.0 != current.0 {
            commands.entity(entity).despawn();
        }
    }
    preparing.0.clear();
}

/// Al cambiar de pista: actualiza el trazado y conmuta la visibilidad entre
/// las pistas cargadas en caché (o la construye si no estuviera).
fn rebuild_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut roots: Query<(Entity, &WorldRoot, &mut Visibility)>,
    mut track: ResMut<Track>,
    mut builder: Builder,
    mut roadsides: ResMut<Roadsides>,
    mut preparing: ResMut<Preparing>,
) {
    if !current.is_changed() || current.is_added() {
        return;
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
        // Needed now: wait for it if it is being prepared. On the web the
        // background work runs on this same thread and cannot be waited
        // for, so it is done here.
        let prepared = match preparing.0.remove(&current.0) {
            Some(task) if !WEB => block_on(task),
            _ => prepare(current.0, &builder.textures, Urgency::Now),
        };
        let built = build(&mut commands, prepared, &mut builder, Visibility::Inherited);
        roadsides.0.insert(current.0, built);
    }
    *track = Track::new(&current.get().layout);
    if let Some(roadside) = roadsides.0.get(&current.0) {
        roadside.apply(&mut track);
    }
}

fn build_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut track: ResMut<Track>,
    mut builder: Builder,
    mut roadsides: ResMut<Roadsides>,
) {
    let prepared = prepare(current.0, &builder.textures, Urgency::Now);
    let built = build(&mut commands, prepared, &mut builder, Visibility::Inherited);
    built.apply(&mut track);
    roadsides.0.insert(current.0, built);
}

/// Lo que hace falta para construir el mundo.
#[derive(bevy::ecs::system::SystemParam)]
struct Builder<'w> {
    textures: Res<'w, textures::WorldTextures>,
    sea: Res<'w, Sea>,
    sea_materials: ResMut<'w, Assets<sea::SeaMaterial>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

/// Builds a prepared circuit's world, and returns what lies beside its
/// roads (see [`Roadside`]).
fn build(
    commands: &mut Commands,
    prepared: Prepared,
    builder: &mut Builder,
    visibility: Visibility,
) -> Roadside {
    let Prepared {
        index,
        track,
        terrain,
        ground,
        shore,
        shore_image,
        roadside,
        pieces,
        lamps,
        grass,
        trees,
    } = prepared;
    let shore_image = builder.images.add(shore_image);
    let scenery = &CIRCUITS[index].scenery;
    let sea_material = builder
        .sea_materials
        .add(shore.material(shore_image, scenery.water));
    let mut assets = Assets3d {
        textures: &builder.textures,
        meshes: &mut builder.meshes,
        materials: &mut builder.materials,
        images: &mut builder.images,
    };
    commands
        .spawn((
            WorldRoot(index),
            lamps::Lamps(lamps),
            Transform::default(),
            visibility,
        ))
        .with_children(|world| {
            terrain::spawn_terrain(
                world,
                &terrain,
                ground,
                assets.textures,
                assets.meshes,
                assets.materials,
            );
            world.spawn((
                Mesh3d(builder.sea.0.clone()),
                MeshMaterial3d(sea_material),
                Transform::from_xyz(0.0, terrain::SEA_LEVEL, 0.0),
            ));
            for (mesh, material) in pieces {
                // What is see-through or glows (light, dust) casts no
                // shadow.
                let shadowless = matches!(material.alpha_mode, AlphaMode::Add | AlphaMode::Blend);
                let mut piece = world.spawn((
                    Mesh3d(assets.meshes.add(mesh)),
                    MeshMaterial3d(assets.materials.add(material)),
                ));
                if shadowless {
                    piece.insert(NotShadowCaster);
                }
            }
            for piece in grass {
                let range = piece.range();
                world.spawn((
                    Mesh3d(assets.meshes.add(piece.mesh)),
                    MeshMaterial3d(assets.materials.add(piece.material)),
                    range,
                    NotShadowCaster,
                ));
            }
            road::spawn_gantries(world, &track, scenery, &mut assets);
            structures::spawn_signs(world, &track, &terrain, scenery, &mut assets);
            props::spawn_props(world, &track, &terrain, scenery, &trees, &mut assets);
        });
    roadside
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
    let sun = sky::Palette::of(scenery::SkyStyle::Day).sun_direction();
    commands.spawn((
        sky::Sunlight,
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_to(-sun, Vec3::Y),
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

        // Entramos al menú de elección de pista: the menu does not wait for
        // the other circuits, which are prepared in the background...
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Tracks);
        app.update();
        assert_eq!(roots_query.iter(app.world()).count(), 1);
        assert_eq!(
            app.world().resource::<Preparing>().0.len(),
            CIRCUITS.len() - 1
        );

        // ...and arrive, hidden, as they are ready.
        let started = std::time::Instant::now();
        while roots_query.iter(app.world()).count() < CIRCUITS.len() {
            assert!(
                started.elapsed().as_secs() < 120,
                "the circuits never arrive"
            );
            app.update();
        }
        assert!(app.world().resource::<Preparing>().0.is_empty());
        let roots: Vec<_> = roots_query.iter(app.world()).collect();
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

    #[test]
    fn a_circuit_chosen_before_it_is_ready_is_built_at_once() {
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
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Tracks);
        app.update();

        // Straight to the last circuit: it is there, with what lies beside
        // its roads, in the next frame.
        let last = CIRCUITS.len() - 1;
        app.world_mut().resource_mut::<CurrentCircuit>().0 = last;
        app.update();
        let mut roots = app.world_mut().query::<(&WorldRoot, &Visibility)>();
        let shown: Vec<usize> = roots
            .iter(app.world())
            .filter(|(_, vis)| **vis == Visibility::Inherited)
            .map(|(root, _)| root.0)
            .collect();
        assert_eq!(shown, [last]);
        assert!(app.world().resource::<Roadsides>().0.contains_key(&last));
        assert!(!app.world().resource::<Preparing>().0.contains_key(&last));
    }
}
