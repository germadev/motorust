//! La carretera y lo que la bordea: asfalto, líneas, pianos en las curvas,
//! muros (de piedra, de hormigón, barandillas o setos, según el tramo) y los
//! arcos sobre la carretera. Todo sigue la altura y el peralte del trazado.

use bevy::prelude::*;

use super::Assets3d;
use super::lettering::sign_image;
use super::quads::MeshBuilder;
use super::scenery::{BridgeStyle, Gantry, Scenery, Side, WallStyle};
use super::terrain::Terrain;
use crate::track::{CURB_WIDTH, SURFACE_Y, Track, WALL_DISTANCE};

/// Tamaño (m) que cubre cada repetición de la textura del asfalto.
const ASPHALT_TILE: f32 = 7.0;
const LINE_WIDTH: f32 = 0.25;
/// Distancia del borde del asfalto a la línea blanca lateral.
const EDGE_LINE_INSET: f32 = 0.45;
/// La línea central: tramos pintados y huecos, en puntos del trazado.
const DASH: (usize, usize) = (2, 3);
/// Las pinturas van un poco por encima del asfalto para no parpadear.
const PAINT_LIFT: f32 = 0.012;
/// Hay pianos donde la curva es más cerrada que este radio (m).
const CURB_RADIUS: f32 = 130.0;

const WALL_HEIGHT: f32 = 1.1;
/// Las barreras de hormigón son algo más bajas, y los setos, más altos.
const CONCRETE_HEIGHT: f32 = 1.0;
const HEDGE_HEIGHT: f32 = 1.3;
/// La barandilla: alto (un poste en cada punto del trazado) y color.
const RAILING_HEIGHT: f32 = 1.05;
const RAILING_GREEN: Color = Color::srgb(0.08, 0.42, 0.28);
const WALL_THICKNESS: f32 = 0.6;
/// Lo que se hunde el muro en el suelo, para que no flote en las cuestas.
const WALL_FOOTING: f32 = 0.5;
const WALL_TILE: f32 = 2.4;
/// Donde el terreno detrás del muro sube más que esto, la roca hace de muro.
const CLIFF_RISE: f32 = 3.0;

const GANTRY_HEIGHT: f32 = 7.5;
const GANTRY_BEAM: f32 = 1.6;
/// Tamaño de cada casilla del tablero del arco y de la línea de meta (m).
const CHECKER_TILE: f32 = 1.0;

pub fn spawn_road(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    assets: &mut Assets3d,
) {
    let half = track.half_width();
    let textures = assets.textures;
    let mut spawn = |mesh: Mesh, material: StandardMaterial| {
        world.spawn((
            Mesh3d(assets.meshes.add(mesh)),
            MeshMaterial3d(assets.materials.add(material)),
        ));
    };

    spawn(
        ribbon(track, -half, half, 0.0, ASPHALT_TILE, |_| {
            Some(Color::WHITE)
        }),
        StandardMaterial {
            base_color_texture: Some(textures.asphalt.clone()),
            perceptual_roughness: 0.88,
            reflectance: 0.5,
            ..default()
        },
    );

    // Pinturas: líneas blancas a los lados, discontinua en el centro y pianos
    // de dos colores solo en las curvas.
    let painted = || StandardMaterial {
        perceptual_roughness: 0.7,
        ..default()
    };
    let white = |_| Some(Color::WHITE);
    let edge = half - EDGE_LINE_INSET;
    spawn(
        ribbon(track, edge - LINE_WIDTH, edge, PAINT_LIFT, 1.0, white),
        painted(),
    );
    spawn(
        ribbon(track, -edge, -edge + LINE_WIDTH, PAINT_LIFT, 1.0, white),
        painted(),
    );
    let dashes = |index: usize| (index % (DASH.0 + DASH.1) < DASH.0).then_some(Color::WHITE);
    let center = LINE_WIDTH / 2.0;
    spawn(
        ribbon(track, -center, center, PAINT_LIFT, 1.0, dashes),
        painted(),
    );
    let curbs = |index: usize| {
        let curved = track.smoothed_curvature(index, 4) > 1.0 / CURB_RADIUS;
        curved.then_some(scenery.curbs[index % 2])
    };
    spawn(
        ribbon(track, half, half + CURB_WIDTH, 0.0, 1.0, curbs),
        painted(),
    );
    spawn(
        ribbon(track, -half - CURB_WIDTH, -half, 0.0, 1.0, curbs),
        painted(),
    );

    // Los muros, cada uno con su aspecto.
    let lap = track.lap_length();
    let style = |index: usize, side: Side| {
        wall_style(scenery, lap, index as f32 * track.step_length(), side)
    };
    for (wanted, height, material) in [
        (
            WallStyle::Stone,
            WALL_HEIGHT,
            StandardMaterial {
                base_color_texture: Some(textures.wall.clone()),
                perceptual_roughness: 0.95,
                ..default()
            },
        ),
        (
            WallStyle::Concrete,
            CONCRETE_HEIGHT,
            StandardMaterial {
                base_color: Color::srgb(0.92, 0.92, 0.9),
                base_color_texture: Some(textures.plaster.clone()),
                perceptual_roughness: 0.9,
                ..default()
            },
        ),
        (
            WallStyle::Hedge,
            HEDGE_HEIGHT,
            StandardMaterial {
                base_color: Color::srgb(0.55, 0.85, 0.4),
                base_color_texture: Some(textures.grass.clone()),
                perceptual_roughness: 1.0,
                ..default()
            },
        ),
        (
            WallStyle::Checkered,
            WALL_HEIGHT,
            StandardMaterial {
                base_color_texture: Some(textures.soil.clone()),
                perceptual_roughness: 0.9,
                ..default()
            },
        ),
    ] {
        let mesh = walls(track, terrain, height, |index, side| {
            style(index, side) == Some(wanted)
        });
        if let Some(mesh) = mesh {
            spawn(mesh, material);
        }
    }
    if let Some(mesh) = railing(track, |index, side| {
        style(index, side) == Some(WallStyle::Railing)
    }) {
        spawn(
            mesh,
            StandardMaterial {
                perceptual_roughness: 0.5,
                ..default()
            },
        );
    }

    // Línea de meta: una franja de tablero de ajedrez de lado a lado.
    let checkered = || StandardMaterial {
        base_color_texture: Some(textures.checker.clone()),
        perceptual_roughness: 0.7,
        ..default()
    };
    spawn(finish_line(track), checkered());

    // Parrilla de salida con cajones pintados para las motos.
    spawn(
        starting_grid(track),
        StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.6,
            ..default()
        },
    );

    // Huellas de derrape y frenada en las curvas para dar dinamismo al asfalto.
    if let Some(skids) = skid_marks(track) {
        spawn(
            skids,
            StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.14),
                perceptual_roughness: 0.95,
                ..default()
            },
        );
    }

    for gantry in scenery.gantries {
        spawn_gantry(world, track, gantry, assets);
    }
}

/// Cómo se cierra la carretera a un lado en un punto de la vuelta, o `None`
/// si no hay muro (en los túneles, que tienen sus paredes).
fn wall_style(scenery: &Scenery, lap: f32, along: f32, side: Side) -> Option<WallStyle> {
    if scenery.tunnels.iter().any(|span| span.contains(along, lap)) {
        return None;
    }
    if scenery.bridges.iter().any(|span| span.contains(along, lap)) {
        // Wooden bridges have their own fence (see `structures`).
        return match scenery.bridge_style {
            BridgeStyle::RedArches => Some(WallStyle::Concrete),
            BridgeStyle::Wooden => None,
        };
    }
    let run = scenery
        .walls
        .iter()
        .find(|run| run.side == side && run.span.contains(along, lap));
    Some(run.map_or(WallStyle::Stone, |run| run.style))
}

/// Cinta que sigue la carretera entre dos desplazamientos laterales (`from <
/// to`, positivo = izquierda), `lift` metros por encima del asfalto. Cada
/// tramo lleva el color que diga `paint`, o no se pinta si es `None`.
fn ribbon(
    track: &Track,
    from: f32,
    to: f32,
    lift: f32,
    tile: f32,
    paint: impl Fn(usize) -> Option<Color>,
) -> Mesh {
    debug_assert!(from < to, "la cinta va de derecha a izquierda");
    let step = track.step_length();
    let mut mesh = MeshBuilder::default();
    for index in 0..track.sample_count() {
        let Some(color) = paint(index) else {
            continue;
        };
        let at =
            |i: usize, lateral: f32| track.surface_point(i, lateral) + Vec3::Y * (SURFACE_Y + lift);
        let (v0, v1) = (index as f32 * step / tile, (index + 1) as f32 * step / tile);
        let (u0, u1) = (0.0, (to - from) / tile);
        mesh.quad(
            [
                (at(index, from), Vec2::new(u0, v0)),
                (at(index + 1, from), Vec2::new(u0, v1)),
                (at(index + 1, to), Vec2::new(u1, v1)),
                (at(index, to), Vec2::new(u1, v0)),
            ],
            Vec3::Y,
            color,
        );
    }
    mesh.build()
}

/// Muros de `height` de alto a los dos lados, donde diga `wanted`. Donde
/// detrás sube la roca, no hace falta muro: la pared del peñasco cierra la
/// carretera. `None` si no hay ninguno.
fn walls(
    track: &Track,
    terrain: &Terrain,
    height: f32,
    wanted: impl Fn(usize, Side) -> bool,
) -> Option<Mesh> {
    let step = track.step_length();
    let n = track.sample_count();
    let mut mesh = MeshBuilder::default();
    for (side, wall_side) in [(1.0, Side::Left), (-1.0, Side::Right)] {
        let (inner, outer) = (
            side * WALL_DISTANCE,
            side * (WALL_DISTANCE + WALL_THICKNESS),
        );
        let cliff = |index: usize| {
            let behind = track.offset_point(index, side * (WALL_DISTANCE + 5.0));
            terrain.height(behind) - track.surface_point(index, inner).y > CLIFF_RISE
        };
        let visible: Vec<bool> = (0..n)
            .map(|i| wanted(i, wall_side) && !cliff(i) && !cliff((i + 1) % n))
            .collect();
        for index in 0..n {
            if !visible[index] {
                continue;
            }
            let next = index + 1;
            let outward = |i: usize| {
                let toward = track.offset_point(i, side) - track.offset_point(i, 0.0);
                Vec3::new(toward.x, 0.0, toward.y)
            };
            let top = |i: usize| track.surface_point(i, inner).y + SURFACE_Y + height;
            let point = |i: usize, lateral: f32, y: f32| {
                let ground = track.surface_point(i, lateral);
                Vec3::new(ground.x, y, ground.z)
            };
            let bottom = |i: usize, lateral: f32| track.surface_point(i, lateral).y - WALL_FOOTING;
            let (u0, u1) = (
                index as f32 * step / WALL_TILE,
                next as f32 * step / WALL_TILE,
            );
            let v = |from: f32, to: f32| (to - from) / WALL_TILE;
            let face_normal = outward(index);
            // Cara hacia la carretera, cara de arriba y cara de fuera.
            for (lateral, facing) in [(inner, -face_normal), (outer, face_normal)] {
                let height = v(bottom(index, lateral), top(index));
                mesh.quad(
                    [
                        (
                            point(index, lateral, bottom(index, lateral)),
                            Vec2::new(u0, height),
                        ),
                        (
                            point(next, lateral, bottom(next, lateral)),
                            Vec2::new(u1, height),
                        ),
                        (point(next, lateral, top(next)), Vec2::new(u1, 0.0)),
                        (point(index, lateral, top(index)), Vec2::new(u0, 0.0)),
                    ],
                    facing,
                    Color::WHITE,
                );
            }
            let thickness = WALL_THICKNESS / WALL_TILE;
            mesh.quad(
                [
                    (point(index, inner, top(index)), Vec2::new(u0, 0.0)),
                    (point(next, inner, top(next)), Vec2::new(u1, 0.0)),
                    (point(next, outer, top(next)), Vec2::new(u1, thickness)),
                    (point(index, outer, top(index)), Vec2::new(u0, thickness)),
                ],
                Vec3::Y,
                Color::WHITE,
            );
            // Remates donde el muro empieza o acaba junto a la roca.
            let along = |i: usize| {
                let (a, b) = (track.offset_point(i, 0.0), track.offset_point(i + 1, 0.0));
                Vec3::new(b.x - a.x, 0.0, b.y - a.y)
            };
            for (at, facing, open) in [
                (index, -along(index), !visible[(index + n - 1) % n]),
                (next, along(index), !visible[next % n]),
            ] {
                if open {
                    let (bi, bo) = (bottom(at, inner), bottom(at, outer));
                    mesh.quad(
                        [
                            (point(at, inner, bi), Vec2::new(0.0, v(bi, top(at)))),
                            (point(at, outer, bo), Vec2::new(thickness, v(bo, top(at)))),
                            (point(at, outer, top(at)), Vec2::new(thickness, 0.0)),
                            (point(at, inner, top(at)), Vec2::ZERO),
                        ],
                        facing,
                        Color::WHITE,
                    );
                }
            }
        }
    }
    (!mesh.is_empty()).then(|| mesh.build())
}

/// Barandilla de paseo marítimo donde diga `wanted`: postes y dos barras
/// verdes, sobre el sitio de los muros.
fn railing(track: &Track, wanted: impl Fn(usize, Side) -> bool) -> Option<Mesh> {
    let n = track.sample_count();
    let mut mesh = MeshBuilder::default();
    for side in [Side::Left, Side::Right] {
        let lateral = side.sign() * (WALL_DISTANCE + 0.2);
        let at = |i: usize, height: f32| {
            track.surface_point(i, lateral) + Vec3::Y * (SURFACE_Y + height)
        };
        for index in 0..n {
            if !wanted(index, side) {
                continue;
            }
            let next = (index + 1) % n;
            mesh.beam(
                at(index, -0.3),
                at(index, RAILING_HEIGHT),
                0.12,
                0.12,
                RAILING_GREEN,
            );
            if wanted(next, side) {
                for height in [RAILING_HEIGHT, RAILING_HEIGHT * 0.55] {
                    mesh.beam(
                        at(index, height),
                        at(next, height),
                        0.08,
                        0.08,
                        RAILING_GREEN,
                    );
                }
            }
        }
    }
    (!mesh.is_empty()).then(|| mesh.build())
}

fn finish_line(track: &Track) -> Mesh {
    let half = track.half_width();
    let (_, heading) = track.pose_at(0);
    let ahead = crate::geometry::forward(heading);
    // Por encima de la línea central discontinua, que pasa por debajo.
    let lift = SURFACE_Y + 2.0 * PAINT_LIFT;
    let at = |lateral: f32, along: f32| {
        track.surface_point(0, lateral) + Vec3::new(ahead.x, 0.0, ahead.y) * along + Vec3::Y * lift
    };
    let squares = 2.0 * half / CHECKER_TILE / 2.0;
    let mut mesh = MeshBuilder::default();
    mesh.quad(
        [
            (at(-half, -CHECKER_TILE), Vec2::new(0.0, 0.0)),
            (at(-half, CHECKER_TILE), Vec2::new(0.0, 1.0)),
            (at(half, CHECKER_TILE), Vec2::new(squares, 1.0)),
            (at(half, -CHECKER_TILE), Vec2::new(squares, 0.0)),
        ],
        Vec3::Y,
        Color::WHITE,
    );
    mesh.build()
}

/// Arco sobre la carretera, con un tablero de ajedrez o un rótulo por
/// delante y por detrás.
fn spawn_gantry(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    gantry: &Gantry,
    assets: &mut Assets3d,
) {
    let index = track.index_at(gantry.along);
    let (point, heading) = track.pose_at(index);
    let ground = track.surface_point(index, 0.0).y;
    let span = WALL_DISTANCE + 1.2;
    let paint = assets.materials.add(StandardMaterial {
        base_color: gantry.color,
        perceptual_roughness: 0.5,
        ..default()
    });
    let pillar = assets
        .meshes
        .add(Cuboid::new(0.8, GANTRY_HEIGHT + 2.0, 0.8));
    let beam = assets
        .meshes
        .add(Cuboid::new(2.0 * span + 0.8, GANTRY_BEAM, 0.7));

    // El tablero o el rótulo, en las dos caras del travesaño: el de delante se
    // lee al llegar, y el de detrás, al mirar atrás.
    let (banner_material, repeat) = match gantry.text {
        None => (
            StandardMaterial {
                base_color_texture: Some(assets.textures.checker.clone()),
                perceptual_roughness: 0.7,
                ..default()
            },
            Vec2::new(
                2.0 * span / CHECKER_TILE / 2.0,
                GANTRY_BEAM / CHECKER_TILE / 2.0,
            ),
        ),
        Some(text) => {
            let image = sign_image(
                text,
                Color::WHITE,
                None,
                gantry.color,
                None,
                2.0 * span / GANTRY_BEAM,
            );
            (
                StandardMaterial {
                    base_color_texture: Some(assets.images.add(image)),
                    perceptual_roughness: 0.6,
                    ..default()
                },
                Vec2::ONE,
            )
        }
    };
    let mut banner = MeshBuilder::default();
    for (z, facing) in [(-0.36, Vec3::NEG_Z), (0.36, Vec3::Z)] {
        let y = GANTRY_HEIGHT - GANTRY_BEAM / 2.0;
        // Visto desde detrás, la derecha es -X: el rótulo va al revés.
        let (left, right) = if z > 0.0 {
            (-span, span)
        } else {
            (span, -span)
        };
        banner.quad(
            [
                (
                    Vec3::new(left, y - GANTRY_BEAM / 2.0, z),
                    Vec2::new(0.0, repeat.y),
                ),
                (
                    Vec3::new(right, y - GANTRY_BEAM / 2.0, z),
                    Vec2::new(repeat.x, repeat.y),
                ),
                (
                    Vec3::new(right, y + GANTRY_BEAM / 2.0, z),
                    Vec2::new(repeat.x, 0.0),
                ),
                (Vec3::new(left, y + GANTRY_BEAM / 2.0, z), Vec2::ZERO),
            ],
            facing,
            Color::WHITE,
        );
    }

    let flag_pole = assets.meshes.add(Cylinder::new(0.04, 2.2));
    let pole_mat = assets.materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.82),
        perceptual_roughness: 0.3,
        ..default()
    });
    let mut flag_mesh = MeshBuilder::default();
    flag_mesh.triangle(
        [
            (Vec3::new(0.0, 1.9, 0.0), Vec2::ZERO),
            (Vec3::new(0.0, 0.9, 0.0), Vec2::Y),
            (Vec3::new(1.3, 1.4, 0.15), Vec2::ONE),
        ],
        Vec3::Z,
        gantry.color,
    );
    let flag_cloth = assets.meshes.add(flag_mesh.build());
    let flag_mat = assets.materials.add(StandardMaterial {
        base_color: gantry.color,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.6,
        ..default()
    });

    world.spawn((
        Transform::from_xyz(point.x, ground, point.y).with_rotation(Quat::from_rotation_y(heading)),
        Visibility::default(),
        children![
            (
                Mesh3d(pillar.clone()),
                MeshMaterial3d(paint.clone()),
                Transform::from_xyz(-span, GANTRY_HEIGHT / 2.0 - 1.0, 0.0),
            ),
            (
                Mesh3d(pillar),
                MeshMaterial3d(paint.clone()),
                Transform::from_xyz(span, GANTRY_HEIGHT / 2.0 - 1.0, 0.0),
            ),
            (
                Mesh3d(beam),
                MeshMaterial3d(paint),
                Transform::from_xyz(0.0, GANTRY_HEIGHT - GANTRY_BEAM / 2.0, 0.0),
            ),
            (
                Mesh3d(assets.meshes.add(banner.build())),
                MeshMaterial3d(assets.materials.add(banner_material)),
            ),
            (
                Mesh3d(flag_pole.clone()),
                MeshMaterial3d(pole_mat.clone()),
                Transform::from_xyz(-span, GANTRY_HEIGHT + 1.1, 0.0),
            ),
            (
                Mesh3d(flag_cloth.clone()),
                MeshMaterial3d(flag_mat.clone()),
                Transform::from_xyz(-span, GANTRY_HEIGHT, 0.0),
            ),
            (
                Mesh3d(flag_pole),
                MeshMaterial3d(pole_mat),
                Transform::from_xyz(span, GANTRY_HEIGHT + 1.1, 0.0),
            ),
            (
                Mesh3d(flag_cloth),
                MeshMaterial3d(flag_mat),
                Transform::from_xyz(span, GANTRY_HEIGHT, 0.0),
            ),
        ],
    ));
}

/// Marcas en la parrilla de salida: cajones para las motos alineadas antes de la meta.
fn starting_grid(track: &Track) -> Mesh {
    let mut mesh = MeshBuilder::default();
    let lift = SURFACE_Y + 1.5 * PAINT_LIFT;
    for slot in 0..8 {
        let (point, heading) = track.grid_slot(slot);
        let fwd = crate::geometry::forward(heading);
        let left = Vec2::new(-fwd.y, fwd.x);
        let at = |x_off: f32, z_off: f32| {
            let p = point + left * x_off + fwd * z_off;
            Vec3::new(p.x, track.road_height(p) + lift, p.y)
        };
        // Línea frontal transversal de salida
        let (w, d) = (0.85, 0.09);
        mesh.quad(
            [
                (at(-w, 0.4 - d), Vec2::ZERO),
                (at(w, 0.4 - d), Vec2::X),
                (at(w, 0.4 + d), Vec2::ONE),
                (at(-w, 0.4 + d), Vec2::Y),
            ],
            Vec3::Y,
            Color::WHITE,
        );
        // Marcas laterales del cajón
        for side in [-w, w] {
            let sw = 0.06;
            mesh.quad(
                [
                    (at(side - sw, -0.9), Vec2::ZERO),
                    (at(side + sw, -0.9), Vec2::X),
                    (at(side + sw, 0.4), Vec2::ONE),
                    (at(side - sw, 0.4), Vec2::Y),
                ],
                Vec3::Y,
                Color::WHITE,
            );
        }
    }
    mesh.build()
}

/// Huellas de neumático en las curvas para dar sensación de carrera.
fn skid_marks(track: &Track) -> Option<Mesh> {
    let mut mesh = MeshBuilder::default();
    let n = track.sample_count();
    let step = track.step_length();
    let lift = SURFACE_Y + 0.5 * PAINT_LIFT;
    for i in 0..n {
        let curv = track.smoothed_curvature(i, 4);
        if curv <= 1.0 / CURB_RADIUS {
            continue;
        }
        let next = (i + 1) % n;
        let at = |idx: usize, lat: f32| track.surface_point(idx, lat) + Vec3::Y * lift;
        for base_lat in [-2.4, 2.4] {
            let (lat0, lat1) = (base_lat - 0.16, base_lat + 0.16);
            let u0 = i as f32 * step / 3.0;
            let u1 = (i + 1) as f32 * step / 3.0;
            mesh.quad(
                [
                    (at(i, lat0), Vec2::new(u0, 0.0)),
                    (at(next, lat0), Vec2::new(u1, 0.0)),
                    (at(next, lat1), Vec2::new(u1, 1.0)),
                    (at(i, lat1), Vec2::new(u0, 1.0)),
                ],
                Vec3::Y,
                Color::srgb(0.12, 0.12, 0.14),
            );
        }
    }
    (!mesh.is_empty()).then(|| mesh.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curbs_only_in_the_corners() {
        let track = Track::default();
        let curved = (0..track.sample_count())
            .filter(|&i| track.smoothed_curvature(i, 4) > 1.0 / CURB_RADIUS)
            .count();
        assert!(curved > 0 && curved < track.sample_count() / 2);
    }

    #[test]
    fn wooden_bridges_have_their_fence_instead_of_walls() {
        for circuit in &crate::circuits::CIRCUITS {
            let scenery = &circuit.scenery;
            let lap = Track::new(&circuit.layout).lap_length();
            for bridge in scenery.bridges {
                let middle = bridge.from + bridge.length(lap) / 2.0;
                let wall = wall_style(scenery, lap, middle, Side::Left);
                match scenery.bridge_style {
                    BridgeStyle::RedArches => assert_eq!(wall, Some(WallStyle::Concrete)),
                    BridgeStyle::Wooden => assert_eq!(wall, None, "{}", circuit.name),
                }
                // Off the bridge, the walls are back.
                let off = wall_style(scenery, lap, bridge.to + 20.0, Side::Left);
                assert!(off.is_some(), "{}", circuit.name);
            }
        }
    }
}
