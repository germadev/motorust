//! Cielo: una cúpula con la foto de un cielo de verano con nubes, centrada
//! siempre en la cámara para que parezca infinitamente lejos.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    asset::RenderAssetUsages, light::NotShadowCaster, mesh::Indices, prelude::*,
    render::render_resource::PrimitiveTopology,
};

use super::textures::WorldTextures;

/// Radio de la cúpula: dentro del plano lejano de la cámara.
const RADIUS: f32 = 4500.0;
const SEGMENTS: u32 = 64;
const RINGS: u32 = 24;
/// La cúpula baja un poco por debajo del horizonte para que no se vea el borde.
const LOWEST_ELEVATION: f32 = -0.25;
/// Dónde está el sol en la foto del cielo (fracción del ancho y altura sobre
/// el horizonte en radianes).
const SUN_IN_PHOTO: (f32, f32) = (0.596, 0.834);
/// Rumbo al que queremos el sol: al sureste, detrás a la derecha en la recta
/// de meta (0 es el norte, -Z; π/2 el este, +X).
const SUN_AZIMUTH: f32 = 0.75 * std::f32::consts::PI;

#[derive(Component)]
pub struct Sky;

/// Dirección hacia el sol, coherente con la foto del cielo.
pub fn sun_direction() -> Vec3 {
    direction(SUN_AZIMUTH, SUN_IN_PHOTO.1)
}

/// Dirección para un rumbo (0 = norte, -Z) y una altura sobre el horizonte.
fn direction(azimuth: f32, elevation: f32) -> Vec3 {
    Vec3::new(
        azimuth.sin() * elevation.cos(),
        elevation.sin(),
        -azimuth.cos() * elevation.cos(),
    )
}

pub fn spawn_sky(
    mut commands: Commands,
    textures: Res<WorldTextures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Sky,
        Mesh3d(meshes.add(dome())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(textures.sky.clone()),
            unlit: true,
            fog_enabled: false,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        NotShadowCaster,
    ));
}

/// La cúpula sigue a la cámara (solo en posición, no en giro).
pub fn follow_camera(
    camera: Single<&Transform, (With<Camera3d>, Without<Sky>)>,
    mut sky: Single<&mut Transform, With<Sky>>,
) {
    sky.translation = camera.translation;
}

/// Cúpula con la foto del cielo proyectada como un panorama: el ancho de la
/// foto da la vuelta al horizonte y su alto va del horizonte al cénit.
fn dome() -> Mesh {
    // La foto empieza en el rumbo que deja su sol en `SUN_AZIMUTH`.
    let photo_offset = SUN_IN_PHOTO.0 - SUN_AZIMUTH / TAU;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for ring in 0..=RINGS {
        let elevation = LOWEST_ELEVATION.lerp(FRAC_PI_2, ring as f32 / RINGS as f32);
        let v = (1.0 - elevation.max(0.0) / FRAC_PI_2).min(0.995);
        for segment in 0..=SEGMENTS {
            let fraction = segment as f32 / SEGMENTS as f32;
            let toward = direction(fraction * TAU, elevation);
            positions.push((toward * RADIUS).to_array());
            normals.push((-toward).to_array());
            uvs.push([fraction + photo_offset, v]);
        }
    }
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_shines_from_the_south_east_and_high() {
        let sun = sun_direction();
        assert!(sun.x > 0.0 && sun.z > 0.0, "sureste: +X y +Z");
        assert!(sun.y > 0.6);
    }

    #[test]
    fn photo_sun_lands_where_the_light_comes_from() {
        // El vértice de la cúpula más cercano al sol tiene la coordenada
        // horizontal del sol en la foto.
        let mesh = dome();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("la cúpula tiene coordenadas de textura");
        };
        let sun = sun_direction();
        let closest = (0..positions.len())
            .max_by(|&a, &b| {
                let along = |i: usize| Vec3::from(positions[i]).normalize().dot(sun);
                along(a).total_cmp(&along(b))
            })
            .unwrap();
        assert!((uvs[closest][0].rem_euclid(1.0) - SUN_IN_PHOTO.0).abs() < 0.02);
    }
}
