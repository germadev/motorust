//! The trees modelled with Meshy, one for each species and a fir for the
//! forests (prepared by `tools/import_trees.py`: about four hundred
//! triangles each, as tall as the classic ones, their colours in one
//! texture). Options > Graphics chooses between them and the classic trees
//! built in code ([`super::props`]); every tree is planted with both looks
//! ([`Tree`]) and dressed in the one chosen ([`dress_trees`]), at once when
//! the option changes.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

use super::scenery::Species;
use crate::bike::model::imported::{read, texture};
use crate::settings::Settings;

/// A modelled tree: one for each species, and the fir.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeModel {
    Of(Species),
    /// A fir, in the forests with the tall trees.
    Fir,
}

/// Every model, in the order of their indices.
const MODELS: [TreeModel; 9] = [
    TreeModel::Of(Species::UmbrellaPine),
    TreeModel::Of(Species::Cypress),
    TreeModel::Of(Species::Olive),
    TreeModel::Of(Species::Tall),
    TreeModel::Of(Species::Broadleaf),
    TreeModel::Of(Species::MaritimePine),
    TreeModel::Of(Species::Poplar),
    TreeModel::Of(Species::Almond),
    TreeModel::Fir,
];

impl TreeModel {
    /// The model of a tree of `kind` planted with `chance` (0 to 1): half
    /// the tall trees of the forests are firs.
    pub fn of(kind: Species, chance: f32) -> Self {
        if kind == Species::Tall && chance < 0.5 {
            TreeModel::Fir
        } else {
            TreeModel::Of(kind)
        }
    }

    /// Its name in `trees.bin`.
    fn name(self) -> &'static str {
        match self {
            TreeModel::Of(Species::UmbrellaPine) => "umbrella_pine",
            TreeModel::Of(Species::Cypress) => "cypress",
            TreeModel::Of(Species::Olive) => "olive",
            TreeModel::Of(Species::Tall) => "tall",
            TreeModel::Of(Species::Broadleaf) => "broadleaf",
            TreeModel::Of(Species::MaritimePine) => "maritime_pine",
            TreeModel::Of(Species::Poplar) => "poplar",
            TreeModel::Of(Species::Almond) => "almond",
            TreeModel::Fir => "fir",
        }
    }

    fn index(self) -> usize {
        MODELS
            .iter()
            .position(|&model| model == self)
            .expect("every model is listed")
    }
}

/// The modelled trees' meshes (by [`TreeModel::index`]) and their one
/// material.
#[derive(Resource)]
pub struct ModelledTrees {
    meshes: Vec<Handle<Mesh>>,
    material: Handle<StandardMaterial>,
}

impl FromWorld for ModelledTrees {
    fn from_world(world: &mut World) -> Self {
        let mut pack = read(include_bytes!("../../assets/models/trees/trees.bin"));
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let meshes = MODELS
            .iter()
            .map(|model| {
                let index = pack
                    .meshes
                    .iter()
                    .position(|piece| piece.name == model.name())
                    .unwrap_or_else(|| panic!("no {} in trees.bin", model.name()));
                meshes.add(pack.meshes.swap_remove(index).mesh)
            })
            .collect();
        let atlas = world.resource_mut::<Assets<Image>>().add(texture(
            include_bytes!("../../assets/models/trees/color.jpg"),
            true,
        ));
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(atlas),
                perceptual_roughness: 0.95,
                reflectance: 0.3,
                ..default()
            });
        Self { meshes, material }
    }
}

/// A tree planted, with both its looks: the classic one (its mesh and
/// material) and its model.
#[derive(Component, Clone)]
pub struct Tree {
    pub classic: Handle<Mesh>,
    pub classic_material: Handle<StandardMaterial>,
    pub model: TreeModel,
}

/// Dresses the trees as Options > Graphics says: each one as it is
/// planted, and all of them when the option changes (their bounds worked
/// out again, for the culling).
#[allow(clippy::type_complexity)]
pub fn dress_trees(
    mut commands: Commands,
    settings: Option<Res<Settings>>,
    modelled: Res<ModelledTrees>,
    mut trees: ParamSet<(
        Query<(
            Entity,
            &Tree,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        )>,
        Query<
            (
                Entity,
                &Tree,
                &mut Mesh3d,
                &mut MeshMaterial3d<StandardMaterial>,
            ),
            Added<Tree>,
        >,
    )>,
    mut dressed: Local<Option<bool>>,
) {
    // Without settings (in tests of the world alone), the new ones, as by
    // default.
    let wanted = settings.is_none_or(|settings| settings.modelled_trees);
    let changed = *dressed != Some(wanted);
    *dressed = Some(wanted);
    let dress = |entity: Entity,
                 tree: &Tree,
                 mut mesh: Mut<Mesh3d>,
                 mut material: Mut<MeshMaterial3d<StandardMaterial>>,
                 commands: &mut Commands| {
        let (look, paint) = if wanted {
            (
                modelled.meshes[tree.model.index()].clone(),
                modelled.material.clone(),
            )
        } else {
            (tree.classic.clone(), tree.classic_material.clone())
        };
        if mesh.0 != look {
            mesh.0 = look;
            commands.entity(entity).remove::<Aabb>();
        }
        if material.0 != paint {
            material.0 = paint;
        }
    };
    if changed {
        for (entity, tree, mesh, material) in &mut trees.p0() {
            dress(entity, tree, mesh, material, &mut commands);
        }
    } else {
        for (entity, tree, mesh, material) in &mut trees.p1() {
            dress(entity, tree, mesh, material, &mut commands);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_is_in_the_pack_standing_on_the_ground() {
        let pack = read(include_bytes!("../../assets/models/trees/trees.bin"));
        for model in MODELS {
            let piece = pack
                .meshes
                .iter()
                .find(|piece| piece.name == model.name())
                .unwrap_or_else(|| panic!("{model:?}"));
            let positions = piece
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|p| p.as_float3())
                .unwrap();
            let bottom = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
            let top = positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
            assert!(bottom.abs() < 1e-3, "{model:?} stands on the ground");
            assert!((4.0..17.0).contains(&top), "{model:?}: {top} m");
            // Light: a few hundred triangles.
            let triangles = piece.mesh.indices().unwrap().len() / 3;
            assert!(triangles < 600, "{model:?}: {triangles}");
            // Its texture coordinates in its own cell of the atlas.
            let uvs = match piece.mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) => uvs,
                _ => panic!("{model:?} has texture coordinates"),
            };
            let cell = |uv: &[f32; 2]| ((uv[0] * 3.0) as usize, (uv[1] * 3.0) as usize);
            let index = model.index();
            assert!(
                uvs.iter().all(|uv| cell(uv) == (index % 3, index / 3)),
                "{model:?} in its cell"
            );
        }
        // Firs only among the tall trees of the forests.
        assert_eq!(TreeModel::of(Species::Tall, 0.2), TreeModel::Fir);
        assert_eq!(
            TreeModel::of(Species::Tall, 0.8),
            TreeModel::Of(Species::Tall)
        );
        assert_eq!(
            TreeModel::of(Species::Olive, 0.2),
            TreeModel::Of(Species::Olive)
        );
    }
}
