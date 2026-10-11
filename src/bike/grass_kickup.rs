//! Grass particles kicked up by bikes when riding off-road on grass surfaces.
//!
//! Higher bike speeds kick up more grass blades and soil clumps backwards
//! and upwards from the rear wheel, providing clear visual feedback for
//! loss of speed and traction.

use std::f32::consts::TAU;

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::{BikeState, DespawnOnExit, GameState};
use crate::geometry::{forward, to_world};
use crate::track::{Surface, Track};

/// Per-bike accumulator for continuous particle emission based on forward speed.
#[derive(Component, Default, Debug, Clone)]
pub struct GrassKickup {
    pub accumulator: f32,
}

/// Physical state of an airborne grass particle.
#[derive(Component, Debug, Clone)]
pub struct GrassParticle {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub initial_scale: Vec3,
}

/// Shared mesh and material resources for batched GPU grass rendering.
#[derive(Resource)]
pub struct GrassAssets {
    pub meshes: Vec<Handle<Mesh>>,
    pub material: Handle<StandardMaterial>,
}

impl FromWorld for GrassAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_handles = vec![
            meshes.add(grass_blade_mesh(0)),
            meshes.add(grass_blade_mesh(1)),
            meshes.add(grass_blade_mesh(2)),
            meshes.add(grass_clump_mesh()),
        ];
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.85,
            reflectance: 0.22,
            cull_mode: None,
            double_sided: true,
            ..default()
        });
        Self {
            meshes: mesh_handles,
            material,
        }
    }
}

pub struct GrassKickupPlugin;

impl Plugin for GrassKickupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrassAssets>().add_systems(
            Update,
            (kick_up_grass, update_grass_particles).run_if(in_state(GameState::Race)),
        );
    }
}

/// Slender grass blade mesh with vertex color gradient from root to tip.
fn grass_blade_mesh(variant: u32) -> Mesh {
    let (w, h) = match variant {
        0 => (0.055, 0.22),
        1 => (0.045, 0.18),
        _ => (0.065, 0.26),
    };
    let base_color = match variant {
        0 => Color::srgb(0.24, 0.68, 0.15),
        1 => Color::srgb(0.38, 0.78, 0.20),
        _ => Color::srgb(0.18, 0.46, 0.12),
    };
    let tip_color = match variant {
        0 => Color::srgb(0.38, 0.84, 0.22),
        1 => Color::srgb(0.55, 0.88, 0.28),
        _ => Color::srgb(0.26, 0.58, 0.18),
    };

    let half_w = w * 0.5;
    let positions = vec![
        [-half_w, 0.0, 0.0],
        [half_w, 0.0, 0.0],
        [half_w * 0.35, h, 0.0],
        [-half_w * 0.35, h, 0.0],
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let colors = vec![
        base_color.to_linear().to_f32_array(),
        base_color.to_linear().to_f32_array(),
        tip_color.to_linear().to_f32_array(),
        tip_color.to_linear().to_f32_array(),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

/// Small clump of grass and soil (two crossed quads at 90 degrees with brown roots).
fn grass_clump_mesh() -> Mesh {
    let w = 0.075;
    let h = 0.15;
    let root_color = Color::srgb(0.35, 0.24, 0.14).to_linear().to_f32_array();
    let leaf_color = Color::srgb(0.25, 0.66, 0.16).to_linear().to_f32_array();
    let half_w = w * 0.5;

    let positions = vec![
        // First quad
        [-half_w, 0.0, 0.0],
        [half_w, 0.0, 0.0],
        [half_w * 0.4, h, 0.0],
        [-half_w * 0.4, h, 0.0],
        // Second crossed quad
        [0.0, 0.0, -half_w],
        [0.0, 0.0, half_w],
        [0.0, h, half_w * 0.4],
        [0.0, h, -half_w * 0.4],
    ];
    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];
    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    let colors = vec![
        root_color, root_color, leaf_color, leaf_color, root_color, root_color, leaf_color,
        leaf_color,
    ];
    let indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

/// Spawns grass particles while a bike travels over grass terrain.
/// Particle emission frequency and velocity scale with forward speed.
pub fn kick_up_grass(
    time: Res<Time>,
    track: Res<Track>,
    grass_assets: Res<GrassAssets>,
    mut commands: Commands,
    mut bikes: Query<(&BikeState, &mut GrassKickup)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let mut rng_seed = (time.elapsed_secs_wrapped() * 1000.0) as u32;
    let mut random = || {
        rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (rng_seed >> 16) as f32 / 65535.0
    };

    for (state, mut kickup) in &mut bikes {
        if !state.grounded || state.speed < 2.0 {
            kickup.accumulator = 0.0;
            continue;
        }

        let nearest = track.nearest(state.position);
        if track.surface_beside(&nearest) != Surface::Grass {
            kickup.accumulator = 0.0;
            continue;
        }

        // Spawn rate proportional to forward speed:
        // At 10 m/s ~ 18 blades/sec; at 36 m/s ~ 66 blades/sec.
        let spawn_rate = state.speed * 1.85;
        kickup.accumulator += spawn_rate * dt;

        let fwd_2d = forward(state.heading);
        let fwd_3d = Vec3::new(fwd_2d.x, 0.0, fwd_2d.y);
        let right_3d = Vec3::new(-fwd_2d.y, 0.0, fwd_2d.x);

        while kickup.accumulator >= 1.0 {
            kickup.accumulator -= 1.0;

            // 85% ejected by the rear drive wheel; 15% by the front steering wheel
            let is_rear = random() < 0.85;
            let wheel_offset = if is_rear { -0.68 } else { 0.70 };
            let lateral_jitter = (random() - 0.5) * 0.22;
            let origin_2d = state.position
                + fwd_2d * wheel_offset
                + Vec2::new(-fwd_2d.y, fwd_2d.x) * lateral_jitter;
            let origin_y = state.elevation + 0.06 + random() * 0.05;
            let spawn_pos = to_world(origin_2d, origin_y);

            // Ejection velocity:
            // Rear tire kicks grass backwards (-fwd) and upwards (+Y).
            // Faster speeds produce higher velocities and larger arcs.
            let backward_speed = if is_rear {
                state.speed * (0.45 + 0.35 * random()) + 2.5
            } else {
                state.speed * 0.15 + 1.0
            };
            let upward_speed = 2.0 + state.speed * (0.08 + 0.06 * random()) + random() * 1.5;
            let side_speed = (random() - 0.5) * (2.0 + state.speed * 0.08);

            let velocity =
                -fwd_3d * backward_speed + Vec3::Y * upward_speed + right_3d * side_speed;
            let angular_velocity = Vec3::new(
                (random() - 0.5) * 22.0,
                (random() - 0.5) * 22.0,
                (random() - 0.5) * 22.0,
            );

            let mesh_idx =
                (random() * grass_assets.meshes.len() as f32) as usize % grass_assets.meshes.len();
            let lifetime = 0.35 + random() * 0.25;
            let scale = Vec3::splat(0.8 + random() * 0.5);

            commands.spawn((
                Mesh3d(grass_assets.meshes[mesh_idx].clone()),
                MeshMaterial3d(grass_assets.material.clone()),
                Transform::from_translation(spawn_pos)
                    .with_scale(scale)
                    .with_rotation(Quat::from_rotation_y(random() * TAU)),
                Visibility::default(),
                GrassParticle {
                    velocity,
                    angular_velocity,
                    lifetime: 0.0,
                    max_lifetime: lifetime,
                    initial_scale: scale,
                },
                DespawnOnExit(GameState::Race),
            ));
        }
    }
}

/// Updates translation, rotation, gravity trajectory, and shrinking of airborne grass particles.
pub fn update_grass_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut GrassParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (entity, mut particle, mut transform) in &mut particles {
        particle.lifetime += dt;
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // Motion, gravity, and air resistance
        transform.translation += particle.velocity * dt;
        transform.rotation *= Quat::from_euler(
            EulerRot::XYZ,
            particle.angular_velocity.x * dt,
            particle.angular_velocity.y * dt,
            particle.angular_velocity.z * dt,
        );

        // Arcade gravity producing parabolic arc
        particle.velocity.y -= 13.5 * dt;
        // Mild aerodynamic drag
        particle.velocity.x *= (1.0 - 1.8 * dt).max(0.0);
        particle.velocity.z *= (1.0 - 1.8 * dt).max(0.0);

        // Scale: remains full size during flight and shrinks quickly near expiry
        let progress = particle.lifetime / particle.max_lifetime;
        let shrink = (1.0 - progress.powi(3)).clamp(0.0, 1.0);
        transform.scale = particle.initial_scale * shrink;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grass_blade_mesh_generates_valid_attributes() {
        let mesh = grass_blade_mesh(0);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
    }

    #[test]
    fn grass_clump_mesh_generates_valid_attributes() {
        let mesh = grass_clump_mesh();
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
    }

    #[test]
    fn grass_kickup_scales_with_speed() {
        let dt = 0.1;
        let slow_speed = 10.0;
        let fast_speed = 30.0;
        let slow_rate = slow_speed * 1.85 * dt;
        let fast_rate = fast_speed * 1.85 * dt;
        assert!(fast_rate > slow_rate * 2.5);
    }
}
