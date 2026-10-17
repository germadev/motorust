//! El mundo alrededor de la carretera: el terreno, la carretera con lo que
//! la bordea y el decorado de la pista elegida ([`crate::circuits`]), bajo
//! un cielo de verano. Al cambiar de pista se vuelve a construir.

mod cavern;
mod dam;
mod desert;
pub mod detail;
mod flowers;
mod foliage;
mod japan;
mod lamps;
mod landmarks;
mod lettering;
mod lod;
mod mine;
mod mirror;
mod props;
mod quads;
mod road;
pub mod scenery;
mod sea;
mod shadow_fade;
mod shoulders;
pub mod sky;
pub(crate) mod structures;
mod terrain;
mod textures;
mod trees;
mod tufts;

use bevy::asset::embedded_asset;
use bevy::pbr::PbrPlugin;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder, block_on, poll_once};
use bevy::{
    camera::visibility::VisibilitySystems, light::NotShadowCaster, prelude::*,
    transform::TransformSystems,
};

use crate::circuits::{CIRCUITS, CurrentCircuit};
use crate::menu::GameState;
use crate::settings::WEB;
use crate::track::{Surface, Track};
pub use textures::generate_mipmaps;
pub use trees::Dressed;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sea.wgsl");
        embedded_asset!(app, "mirror.wgsl");
        // Without the renderer (in the tests), the sea's and the mirroring
        // water's materials are only assets.
        if app.is_plugin_added::<PbrPlugin>() {
            app.add_plugins((
                MaterialPlugin::<sea::SeaMaterial>::default(),
                MaterialPlugin::<mirror::MirrorMaterial>::default(),
            ));
        } else {
            app.init_asset::<sea::SeaMaterial>()
                .init_asset::<mirror::MirrorMaterial>();
        }
        let current = CurrentCircuit::default();
        app.add_plugins((
            lamps::LampsPlugin,
            detail::DetailPlugin,
            shadow_fade::ShadowFadePlugin,
        ));
        app.insert_resource(current)
            .insert_resource(Track::new(&current.get().layout))
            .init_resource::<Roadsides>()
            .init_resource::<Preparing>()
            .init_resource::<CircuitLoading>()
            .init_resource::<textures::WorldTextures>()
            .init_resource::<trees::ModelledTrees>()
            .init_resource::<japan::ModelledProps>()
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
                    japan::turn_wheels,
                    japan::drift_petals,
                    spawn_prepared_circuit,
                ),
            )
            .add_systems(OnEnter(GameState::Tracks), cache_all_circuits)
            .add_systems(OnExit(GameState::Tracks), unload_unused_circuits)
            // A championship goes from track to track without choosing
            // them: the one left behind goes when the next race starts.
            .add_systems(OnEnter(GameState::Race), unload_unused_circuits)
            .add_systems(
                PostUpdate,
                (
                    sea::dress_the_sea,
                    sky::follow_camera.before(TransformSystems::Propagate),
                    // With the trees where they are and whether they are
                    // seen, before what each camera sees is worked out.
                    trees::dress_trees
                        .after(TransformSystems::Propagate)
                        .after(VisibilitySystems::VisibilityPropagate)
                        .before(VisibilitySystems::CheckVisibility),
                ),
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
    ground: Vec<terrain::GroundMesh>,
    shore: sea::ShoreMap,
    shore_image: Image,
    /// The sea mirroring what stands round it, unless the track's sea is
    /// always the cartoon one.
    sea_mirror: Option<mirror::PreparedMirror>,
    roadside: Roadside,
    /// The meshes of the roads, tunnels and bridges, with their materials.
    pieces: Vec<(Mesh, StandardMaterial)>,
    /// The lakes behind its dams, which mirror what stands round them.
    lakes: Vec<dam::PreparedLake>,
    /// The lamps of its tunnels and mine (see [`lamps`]).
    lamps: Vec<lamps::Lamp>,
    /// Its grass beside the roads, drawn only near the camera.
    grass: Vec<tufts::GrassPiece>,
    trees: props::TreePlan,
    /// The lawn in flower, for each kind of flower the ground is drawn in,
    /// and the flowers standing up near the roads.
    meadows: Vec<(scenery::FlowerKind, Image)>,
    flowers: flowers::FlowerPlan,
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
    let mut ground = terrain.meshes();
    let shore = sea::ShoreMap::new(&terrain, &track);
    let shore_image = shore.image();
    let roadside = Roadside {
        edges: terrain.track_edges(&track),
        shoulders: shoulders::track_shoulders(&track, &terrain, scenery),
    };
    let mut pieces = road::road_pieces(&track, &terrain, scenery, textures);
    let (structures, lamps, tops) =
        structures::structure_pieces(&track, &terrain, scenery, textures);
    pieces.extend(structures);
    ground.extend(tops);
    let lakes = dam::prepare_lakes(&track, &terrain, scenery, textures, urgency.threads());
    let sea_mirror = (!scenery.cartoon_water).then(|| {
        mirror::prepare_mirror(
            &track,
            &terrain,
            scenery,
            textures,
            &sea::mirror_reach(&track),
            &|at| shore.distance_at(at) < -sea::MIRROR_INLAND,
            mirror::deep(scenery.water),
            urgency.threads(),
        )
    });
    let grass = tufts::tuft_pieces(
        &track,
        &terrain,
        &roadside.shoulders,
        &roadside.edges,
        textures,
    );
    let trees = props::plan_trees(&track, &terrain, scenery);
    // One meadow for each kind of flower drawn (the tunnels' tops drawn in
    // the same as the ground round them).
    let mut kinds = Vec::new();
    for kind in ground.iter().filter_map(|ground| ground.flowers) {
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    let meadows = kinds
        .into_iter()
        .map(|kind| (kind, foliage::meadow(&textures.lawn, kind)))
        .collect();
    let flowers = flowers::plan_flowers(&track, &terrain);
    structures::draw_signs(scenery);
    Prepared {
        index,
        track,
        terrain,
        ground,
        shore,
        shore_image,
        sea_mirror,
        roadside,
        pieces,
        lakes,
        lamps,
        grass,
        trees,
        meadows,
        flowers,
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

/// The thread a circuit needed now is prepared on, in the tracks menu: its
/// own, so that it does not wait behind the others being prepared in the
/// background.
fn urgent_pool() -> &'static TaskPool {
    static POOL: std::sync::OnceLock<TaskPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        TaskPoolBuilder::new()
            .num_threads(1)
            .thread_name("Circuit chosen".into())
            .build()
    })
}

/// A circuit being prepared, and whether the work on it has begun (or it
/// still waits its turn on its thread).
struct Preparation {
    task: Task<Prepared>,
    begun: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Preparation {
    /// Circuit `index` prepared on another thread: the background one, or,
    /// if it is needed now, the one of its own.
    fn start(index: usize, textures: &textures::WorldTextures, urgency: Urgency) -> Self {
        let textures = textures.clone();
        let begun = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = begun.clone();
        let pool = match urgency {
            Urgency::Now => urgent_pool(),
            Urgency::Background => circuit_pool(),
        };
        let task = pool.spawn(async move {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
            prepare(index, &textures, urgency)
        });
        Self { task, begun }
    }

    fn begun(&self) -> bool {
        self.begun.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// The circuits being prepared, by index in [`CIRCUITS`].
#[derive(Resource, Default)]
struct Preparing(std::collections::HashMap<usize, Preparation>);

/// Whether the circuit chosen is still being prepared: in the tracks menu
/// it is not waited for (the menu would stop for seconds), it comes when
/// ready.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CircuitLoading(pub bool);

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
    for index in 0..CIRCUITS.len() {
        if !built.contains(&index) && !preparing.0.contains_key(&index) {
            let preparation = Preparation::start(index, &textures, Urgency::Background);
            preparing.0.insert(index, preparation);
        }
    }
}

/// Puts in the world a circuit prepared on another thread (one per frame:
/// making its entities and meshes takes a little while too): the one
/// chosen first, shown, with what lies beside its roads given to the
/// track; the others hidden.
fn spawn_prepared_circuit(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    mut preparing: ResMut<Preparing>,
    mut builder: Builder,
    mut roadsides: ResMut<Roadsides>,
    mut track: ResMut<Track>,
    mut loading: ResMut<CircuitLoading>,
) {
    let mut order: Vec<usize> = preparing.0.keys().copied().collect();
    order.sort_by_key(|&index| index != current.0);
    let ready = order.into_iter().find_map(|index| {
        let preparation = preparing.0.get_mut(&index)?;
        block_on(poll_once(&mut preparation.task)).map(|prepared| (index, prepared))
    });
    let Some((index, prepared)) = ready else {
        return;
    };
    preparing.0.remove(&index);
    let chosen = index == current.0;
    let visibility = if chosen {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let built = build(&mut commands, prepared, &mut builder, visibility);
    if chosen {
        built.apply(&mut track);
        loading.set_if_neq(CircuitLoading(false));
    }
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
    // The one chosen, if it is still on its way, comes all the same.
    preparing.0.retain(|&index, _| index == current.0);
}

/// Al cambiar de pista: actualiza el trazado y conmuta la visibilidad entre
/// las pistas cargadas en caché, o, si no estuviera, la construye. In the
/// tracks menu it is not waited for: it is prepared on a thread of its own
/// if it has not begun yet, and comes when ready ([`CircuitLoading`],
/// [`spawn_prepared_circuit`]); elsewhere (a championship going on to the
/// next track) it is waited for.
#[allow(clippy::too_many_arguments)]
fn rebuild_world(
    mut commands: Commands,
    current: Res<CurrentCircuit>,
    state: Option<Res<State<GameState>>>,
    mut roots: Query<(Entity, &WorldRoot, &mut Visibility)>,
    mut track: ResMut<Track>,
    mut builder: Builder,
    mut roadsides: ResMut<Roadsides>,
    mut preparing: ResMut<Preparing>,
    mut loading: ResMut<CircuitLoading>,
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
    *track = Track::new(&current.get().layout);
    let in_menu = state.is_some_and(|state| *state.get() == GameState::Tracks);
    if !found && in_menu && !WEB {
        if preparing.0.get(&current.0).is_none_or(|p| !p.begun()) {
            let preparation = Preparation::start(current.0, &builder.textures, Urgency::Now);
            preparing.0.insert(current.0, preparation);
        }
        loading.set_if_neq(CircuitLoading(true));
        return;
    }
    if !found {
        // Needed now: wait for it if it is being prepared. On the web the
        // background work runs on this same thread and cannot be waited
        // for, so it is done here.
        let prepared = match preparing.0.remove(&current.0) {
            Some(preparation) if !WEB => block_on(preparation.task),
            _ => prepare(current.0, &builder.textures, Urgency::Now),
        };
        let built = build(&mut commands, prepared, &mut builder, Visibility::Inherited);
        roadsides.0.insert(current.0, built);
    }
    loading.set_if_neq(CircuitLoading(false));
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
    props: Res<'w, japan::ModelledProps>,
    sea: Res<'w, Sea>,
    sea_materials: ResMut<'w, Assets<sea::SeaMaterial>>,
    mirror_materials: ResMut<'w, Assets<mirror::MirrorMaterial>>,
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
        sea_mirror,
        roadside,
        pieces,
        lakes,
        lamps,
        grass,
        trees,
        meadows,
        flowers,
    } = prepared;
    let meadows: Vec<_> = meadows
        .into_iter()
        .map(|(kind, meadow)| (kind, builder.images.add(meadow)))
        .collect();
    let shore_image = builder.images.add(shore_image);
    let scenery = &CIRCUITS[index].scenery;
    let cartoon_sea = builder
        .sea_materials
        .add(shore.material(shore_image, scenery.water));
    let lakes: Vec<_> = lakes
        .into_iter()
        .map(|lake| {
            let material = lake.mirror.into_material(&mut builder.images);
            (
                builder.meshes.add(lake.mesh),
                builder.mirror_materials.add(material),
            )
        })
        .collect();
    let mut assets = Assets3d {
        textures: &builder.textures,
        props: &builder.props,
        meshes: &mut builder.meshes,
        materials: &mut builder.materials,
        images: &mut builder.images,
        mirrored: Vec::new(),
        mirrored_leaves: Vec::new(),
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
                &meadows,
                assets.textures,
                assets.meshes,
                assets.materials,
            );
            for (mesh, material) in lakes {
                world.spawn((Mesh3d(mesh), MeshMaterial3d(material), NotShadowCaster));
            }
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
            props::spawn_props(
                world,
                &track,
                &terrain,
                scenery,
                &trees,
                flowers,
                &mut assets,
            );
            // The sea, mirroring what stands round it too, now that it
            // stands; the cartoon one at first, mirroring, if it should,
            // before it is first drawn (see `sea::dress_the_sea`).
            let sea_water = sea::SeaWater {
                cartoon: cartoon_sea,
                mirror: sea_mirror.map(|mut mirror| {
                    mirror.stand_up(&assets.mirrored, &assets.mirrored_leaves);
                    let material = mirror.into_material(assets.images);
                    builder.mirror_materials.add(material)
                }),
            };
            world.spawn((
                Mesh3d(builder.sea.0.clone()),
                MeshMaterial3d(sea_water.cartoon.clone()),
                sea_water,
                Transform::from_xyz(0.0, terrain::SEA_LEVEL, 0.0),
            ));
        });
    roadside
}

/// Dónde guardar las mallas, los materiales y las imágenes del mundo.
pub struct Assets3d<'a> {
    pub textures: &'a textures::WorldTextures,
    /// The props made with Meshy (see [`japan`]).
    pub props: &'a japan::ModelledProps,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub images: &'a mut Assets<Image>,
    /// Points over the surfaces of what is built round the water, and over
    /// the crowns of the trees in colour, as they are put in the world, for
    /// the sea to mirror (see [`mirror::PreparedMirror::stand_up`]).
    pub mirrored: Vec<mirror::MirrorPoint>,
    pub mirrored_leaves: Vec<mirror::MirrorPoint>,
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
        // Shadows are only needed near the camera, which gives them more
        // resolution (with the screen split, nearer still: see [`detail`]).
        detail::Detail::default().cascades(),
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
    fn a_circuit_chosen_before_it_is_ready_comes_without_stopping_the_menu() {
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

        // Straight to the last circuit, behind all the others in the
        // background: the menu goes on at once, the circuit on its way.
        let last = CIRCUITS.len() - 1;
        app.world_mut().resource_mut::<CurrentCircuit>().0 = last;
        let started = std::time::Instant::now();
        app.update();
        assert!(started.elapsed().as_secs_f32() < 1.0, "the menu stopped");
        assert_eq!(
            *app.world().resource::<CircuitLoading>(),
            CircuitLoading(true)
        );
        // It comes, shown alone, with what lies beside its roads.
        let mut roots = app.world_mut().query::<(&WorldRoot, &Visibility)>();
        while app.world().resource::<CircuitLoading>().0 {
            assert!(started.elapsed().as_secs() < 120, "it never comes");
            app.update();
        }
        app.update();
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
