//! Las obras de la carretera: túneles abiertos en la roca, puentes de arcos
//! sobre el agua y carteles publicitarios.

use bevy::prelude::*;

use super::Assets3d;
use super::lettering::sign_image;
use super::quads::MeshBuilder;
use super::scenery::{Billboard, BrakeMarker, ChevronDirection, ChevronSign, Scenery, Span};
use super::terrain::Terrain;
use crate::geometry::fbm;
use crate::track::{SURFACE_Y, Track, WALL_DISTANCE};

/// Alto de las paredes del túnel hasta el techo (m), y lo que se abomba la
/// roca de encima.
const TUNNEL_HEIGHT: f32 = 6.5;
const TUNNEL_ROCK: f32 = 15.0;
/// Hasta dónde llega la roca del túnel a cada lado de la línea central (m).
const TUNNEL_SHELL: f32 = WALL_DISTANCE + 12.0;
/// Franja de tablero de ajedrez al pie de las paredes del túnel (m).
const TUNNEL_CHECKER: f32 = 0.7;
/// Una luz del techo y una costilla cada tantos puntos del trazado.
const TUNNEL_LIGHT_EVERY: usize = 3;
const TUNNEL_RIB_EVERY: usize = 4;
/// Alto de los arcos del puente sobre el tablero (m), su grosor y la
/// separación entre tirantes (en puntos del trazado).
const ARCH_RISE: f32 = 15.0;
const ARCH_THICKNESS: f32 = 0.9;
const HANGER_EVERY: usize = 2;
const BRIDGE_RED: Color = Color::srgb(0.72, 0.1, 0.07);
/// Grosor del tablero del puente (m).
const DECK_DEPTH: f32 = 1.6;

pub fn spawn_structures(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    scenery: &Scenery,
    assets: &mut Assets3d,
) {
    let lap = track.lap_length();
    for tunnel in scenery.tunnels {
        let indices = indices(track, *tunnel, lap);
        let parts = tunnel_meshes(track, terrain, &indices);
        let rock = StandardMaterial {
            base_color: Color::srgb(0.85, 0.75, 0.6),
            base_color_texture: Some(assets.textures.rock.clone()),
            perceptual_roughness: 0.9,
            ..default()
        };
        // Cálido y algo iluminado por las lámparas, como en Moto Racer.
        let lining = StandardMaterial {
            base_color: Color::srgb(0.95, 0.78, 0.52),
            base_color_texture: Some(assets.textures.panels.clone()),
            emissive: LinearRgba::rgb(0.12, 0.08, 0.035),
            perceptual_roughness: 0.85,
            ..default()
        };
        let concrete = StandardMaterial {
            base_color: Color::srgb(0.75, 0.73, 0.7),
            base_color_texture: Some(assets.textures.plaster.clone()),
            perceptual_roughness: 0.9,
            ..default()
        };
        let lights = StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.6),
            emissive: LinearRgba::rgb(6.0, 4.6, 2.2),
            ..default()
        };
        let checker = StandardMaterial {
            base_color_texture: Some(assets.textures.checker.clone()),
            perceptual_roughness: 0.7,
            ..default()
        };
        for (mesh, material) in [
            (parts.rock, rock),
            (parts.lining, lining),
            (parts.portals, concrete.clone()),
            (parts.sidewalks, concrete),
            (parts.lights, lights),
            (parts.checker, checker),
        ] {
            world.spawn((
                Mesh3d(assets.meshes.add(mesh)),
                MeshMaterial3d(assets.materials.add(material)),
            ));
        }
    }
    for bridge in scenery.bridges {
        let indices = indices(track, *bridge, lap);
        let (deck, arches) = bridge_meshes(track, terrain, &indices);
        world.spawn((
            Mesh3d(assets.meshes.add(deck)),
            MeshMaterial3d(assets.materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.7, 0.68),
                base_color_texture: Some(assets.textures.plaster.clone()),
                perceptual_roughness: 0.9,
                ..default()
            })),
        ));
        world.spawn((
            Mesh3d(assets.meshes.add(arches)),
            MeshMaterial3d(assets.materials.add(StandardMaterial {
                perceptual_roughness: 0.45,
                metallic: 0.2,
                ..default()
            })),
        ));
    }
    for billboard in scenery.billboards {
        spawn_billboard(world, track, terrain, billboard, assets);
    }
    for chevron in scenery.chevrons {
        spawn_chevron(world, track, terrain, chevron, assets);
    }
    for marker in scenery.brake_markers {
        spawn_brake_marker(world, track, terrain, marker, assets);
    }
}

/// Los puntos del trazado de un tramo, en orden de carrera.
fn indices(track: &Track, span: Span, lap: f32) -> Vec<usize> {
    let first = track.index_at(span.from);
    let count = (span.length(lap) / track.step_length()).round() as usize;
    (0..=count)
        .map(|i| (first + i) % track.sample_count())
        .collect()
}

/// Las partes de un túnel, cada una con su material.
struct TunnelParts {
    /// La roca por encima y a los lados.
    rock: Mesh,
    /// Paredes y techo por dentro.
    lining: Mesh,
    /// El marco de hormigón de las bocas.
    portals: Mesh,
    /// Aceras entre los pianos y las paredes.
    sidewalks: Mesh,
    lights: Mesh,
    /// Tablero de ajedrez al pie de las paredes.
    checker: Mesh,
}

/// Un túnel recto por dentro, con el techo plano, luces cálidas y las
/// paredes con tablero de ajedrez abajo, como en Moto Racer. Por fuera es un
/// bloque de roca que se mete en las paredes de roca que lo rodean.
fn tunnel_meshes(track: &Track, terrain: &Terrain, indices: &[usize]) -> TunnelParts {
    let mut rock = MeshBuilder::default();
    let mut lining = MeshBuilder::default();
    let mut portals = MeshBuilder::default();
    let mut sidewalks = MeshBuilder::default();
    let mut lights = MeshBuilder::default();
    let mut checker = MeshBuilder::default();
    let step = track.step_length();
    let road = |i: usize| track.surface_point(i, 0.0).y + SURFACE_Y;
    // Un punto a `lateral` de la línea central y `height` sobre la carretera.
    let at = |i: usize, lateral: f32, height: f32| {
        let ground = track.surface_point(i, lateral);
        Vec3::new(ground.x, road(i) + height, ground.z)
    };
    // La roca de encima (altura sobre la carretera): en los bordes, a la
    // altura del terreno de al lado, para que no se note dónde empieza, y
    // por el medio, algo abombada e irregular.
    let edge = |i: usize, side: f32| {
        let beyond = track.offset_point(i, side * (TUNNEL_SHELL + 1.0));
        (terrain.height(beyond) - road(i)).max(TUNNEL_HEIGHT + 3.0)
    };
    let rock_top = |i: usize, lateral: f32| {
        let t = (lateral / TUNNEL_SHELL + 1.0) / 2.0;
        let ground = track.surface_point(i, lateral);
        let rugged = fbm(Vec2::new(ground.x, ground.z) / 19.0, 3);
        let hump = TUNNEL_ROCK * 0.2 * (1.0 - (2.0 * t - 1.0).powi(2)) * (0.5 + rugged);
        edge(i, -1.0).lerp(edge(i, 1.0), t) + hump
    };
    let inner = WALL_DISTANCE;
    let sidewalk_from = track.half_width();
    for pair in indices.windows(2) {
        let (i, j) = (pair[0], pair[1]);
        let (vi, vj) = ((i as f32 * step) / 4.0, (i as f32 * step + step) / 4.0);
        let inward = |lateral: f32| {
            let toward = track.offset_point(i, -lateral.signum()) - track.offset_point(i, 0.0);
            Vec3::new(toward.x, 0.0, toward.y)
        };
        for side in [1.0, -1.0] {
            let wall = side * inner;
            // Pared: el tablero abajo y el revestimiento hasta el techo.
            checker.quad(
                [
                    (at(i, wall, -1.5), Vec2::new(vi * 4.0, 2.0)),
                    (at(j, wall, -1.5), Vec2::new(vj * 4.0, 2.0)),
                    (at(j, wall, TUNNEL_CHECKER), Vec2::new(vj * 4.0, 0.0)),
                    (at(i, wall, TUNNEL_CHECKER), Vec2::new(vi * 4.0, 0.0)),
                ],
                inward(wall),
                Color::WHITE,
            );
            lining.quad(
                [
                    (at(i, wall, TUNNEL_CHECKER), Vec2::new(vi, 1.5)),
                    (at(j, wall, TUNNEL_CHECKER), Vec2::new(vj, 1.5)),
                    (at(j, wall, TUNNEL_HEIGHT), Vec2::new(vj, 0.0)),
                    (at(i, wall, TUNNEL_HEIGHT), Vec2::new(vi, 0.0)),
                ],
                inward(wall),
                Color::WHITE,
            );
            // Aceras.
            let walk = side * sidewalk_from;
            // Siguen el peralte, como el arcén que tapan.
            let ground = |k: usize, lateral: f32| {
                track.surface_point(k, lateral) + Vec3::Y * (SURFACE_Y + 0.03)
            };
            sidewalks.quad(
                [
                    (ground(i, walk), Vec2::new(vi, 0.0)),
                    (ground(j, walk), Vec2::new(vj, 0.0)),
                    (ground(j, wall), Vec2::new(vj, 1.0)),
                    (ground(i, wall), Vec2::new(vi, 1.0)),
                ],
                Vec3::Y,
                Color::WHITE,
            );
            // Lámparas en las paredes, a media altura.
            if i % TUNNEL_LIGHT_EVERY == 0 {
                let lamp = wall - side * 0.05;
                lights.quad(
                    [
                        (at(i, lamp, 3.6), Vec2::ZERO),
                        (at(j, lamp, 3.6), Vec2::X),
                        (at(j, lamp, 4.4), Vec2::ONE),
                        (at(i, lamp, 4.4), Vec2::Y),
                    ],
                    inward(wall),
                    Color::WHITE,
                );
            }
            // Los lados de la roca, por fuera.
            let shell = side * TUNNEL_SHELL;
            rock.quad(
                [
                    (at(i, shell, -6.0), Vec2::new(vi, 2.0)),
                    (at(j, shell, -6.0), Vec2::new(vj, 2.0)),
                    (at(j, shell, rock_top(j, shell)), Vec2::new(vj, 0.0)),
                    (at(i, shell, rock_top(i, shell)), Vec2::new(vi, 0.0)),
                ],
                -inward(wall),
                Color::WHITE,
            );
        }
        // El techo, por dentro, con dos filas de luces.
        lining.quad(
            [
                (at(i, -inner, TUNNEL_HEIGHT), Vec2::new(vi, 0.0)),
                (at(j, -inner, TUNNEL_HEIGHT), Vec2::new(vj, 0.0)),
                (at(j, inner, TUNNEL_HEIGHT), Vec2::new(vj, inner / 2.0)),
                (at(i, inner, TUNNEL_HEIGHT), Vec2::new(vi, inner / 2.0)),
            ],
            Vec3::NEG_Y,
            Color::WHITE,
        );
        // Costillas de lado a lado bajo el techo, que le dan relieve.
        if i % TUNNEL_RIB_EVERY == 0 {
            let rib = TUNNEL_HEIGHT - 0.3;
            lining.beam(
                at(i, -inner, rib),
                at(i, inner, rib),
                0.6,
                0.6,
                Color::srgb(0.8, 0.8, 0.8),
            );
        }
        if i % TUNNEL_LIGHT_EVERY == 0 {
            for lateral in [-3.5, 3.5] {
                lights.quad(
                    [
                        (at(i, lateral - 0.3, TUNNEL_HEIGHT - 0.05), Vec2::ZERO),
                        (at(j, lateral - 0.3, TUNNEL_HEIGHT - 0.05), Vec2::X),
                        (at(j, lateral + 0.3, TUNNEL_HEIGHT - 0.05), Vec2::ONE),
                        (at(i, lateral + 0.3, TUNNEL_HEIGHT - 0.05), Vec2::Y),
                    ],
                    Vec3::NEG_Y,
                    Color::WHITE,
                );
            }
        }
        // La roca de encima, en franjas de lado a lado.
        const STRIPS: usize = 8;
        for k in 0..STRIPS {
            let (a, b) = (
                -TUNNEL_SHELL + 2.0 * TUNNEL_SHELL * k as f32 / STRIPS as f32,
                -TUNNEL_SHELL + 2.0 * TUNNEL_SHELL * (k + 1) as f32 / STRIPS as f32,
            );
            rock.quad(
                [
                    (at(i, a, rock_top(i, a)), Vec2::new(vi, a / 4.0)),
                    (at(j, a, rock_top(j, a)), Vec2::new(vj, a / 4.0)),
                    (at(j, b, rock_top(j, b)), Vec2::new(vj, b / 4.0)),
                    (at(i, b, rock_top(i, b)), Vec2::new(vi, b / 4.0)),
                ],
                Vec3::Y,
                Color::WHITE,
            );
        }
    }
    // Las bocas: la roca, de lado a lado, con la abertura del túnel en un
    // marco de hormigón que sobresale un poco.
    const FRAME: f32 = 1.2;
    for (end, outward) in [(indices[0], -1.0), (indices[indices.len() - 1], 1.0)] {
        let (_, heading) = track.pose_at(end);
        let ahead = crate::geometry::forward(heading);
        let facing = Vec3::new(ahead.x, 0.0, ahead.y) * outward;
        let top = |lateral: f32| rock_top(end, lateral);
        let framed = inner + FRAME;
        let lintel = TUNNEL_HEIGHT + FRAME;
        // La roca: a los lados del marco y por encima.
        for (from, to, bottom) in [
            (-TUNNEL_SHELL, -framed, -6.0),
            (framed, TUNNEL_SHELL, -6.0),
            (-framed, framed, lintel),
        ] {
            rock.quad(
                [
                    (at(end, from, bottom), Vec2::new(from / 4.0, 2.0)),
                    (at(end, to, bottom), Vec2::new(to / 4.0, 2.0)),
                    (at(end, to, top(to)), Vec2::new(to / 4.0, 0.0)),
                    (at(end, from, top(from)), Vec2::new(from / 4.0, 0.0)),
                ],
                facing,
                Color::WHITE,
            );
        }
        // El marco: dos pilares y el dintel, por delante de la roca.
        let push = facing * 0.4;
        for (from, to, bottom, height) in [
            (-framed, -inner, -1.5, lintel),
            (inner, framed, -1.5, lintel),
            (-framed, framed, TUNNEL_HEIGHT, lintel),
        ] {
            portals.quad(
                [
                    (at(end, from, bottom) + push, Vec2::new(from / 4.0, 2.0)),
                    (at(end, to, bottom) + push, Vec2::new(to / 4.0, 2.0)),
                    (at(end, to, height) + push, Vec2::new(to / 4.0, 0.0)),
                    (at(end, from, height) + push, Vec2::new(from / 4.0, 0.0)),
                ],
                facing,
                Color::WHITE,
            );
        }
    }
    TunnelParts {
        rock: rock.build(),
        lining: lining.build(),
        portals: portals.build(),
        sidewalks: sidewalks.build(),
        lights: lights.build(),
        checker: checker.build(),
    }
}

/// El tablero de un puente (de hormigón, por debajo de la carretera) y sus
/// dos arcos rojos, uno a cada lado, con los tirantes que cuelgan de ellos.
fn bridge_meshes(track: &Track, terrain: &Terrain, indices: &[usize]) -> (Mesh, Mesh) {
    let mut deck = MeshBuilder::default();
    let mut arches = MeshBuilder::default();
    let edge = WALL_DISTANCE + 0.8;
    let road = |i: usize, lateral: f32| track.surface_point(i, lateral);
    for (k, pair) in indices.windows(2).enumerate() {
        let (i, j) = (pair[0], pair[1]);
        let down = Vec3::Y * DECK_DEPTH;
        // Por debajo y a los lados del tablero.
        deck.quad(
            [
                (road(i, -edge) - down, Vec2::ZERO),
                (road(j, -edge) - down, Vec2::X),
                (road(j, edge) - down, Vec2::ONE),
                (road(i, edge) - down, Vec2::Y),
            ],
            Vec3::NEG_Y,
            Color::WHITE,
        );
        for side in [-1.0, 1.0] {
            let out = road(i, side * (edge + 1.0)) - road(i, side * edge);
            deck.quad(
                [
                    (road(i, side * edge) - down, Vec2::ZERO),
                    (road(j, side * edge) - down, Vec2::X),
                    (road(j, side * edge), Vec2::ONE),
                    (road(i, side * edge), Vec2::Y),
                ],
                out,
                Color::WHITE,
            );
            // El arco, a lo largo de todo el puente.
            let arch = |k: usize| {
                let t = k as f32 / (indices.len() - 1) as f32;
                road(indices[k], side * (edge + 0.4))
                    + Vec3::Y * ARCH_RISE * (std::f32::consts::PI * t).sin()
            };
            arches.beam(
                arch(k),
                arch(k + 1),
                ARCH_THICKNESS,
                ARCH_THICKNESS * 1.4,
                BRIDGE_RED,
            );
            if k % HANGER_EVERY == 0 && k > 0 {
                arches.beam(
                    road(i, side * (edge + 0.4)),
                    arch(k),
                    0.18,
                    0.18,
                    BRIDGE_RED,
                );
            }
        }
    }
    // Pilares hasta el agua en los dos extremos y en el centro.
    for &k in &[0, indices.len() / 2, indices.len() - 1] {
        let i = indices[k];
        for side in [-1.0, 1.0] {
            let top = road(i, side * (edge - 1.5)) - Vec3::Y * DECK_DEPTH;
            let ground = terrain.height(Vec2::new(top.x, top.z)).min(top.y) - 2.0;
            deck.beam(Vec3::new(top.x, ground, top.z), top, 2.2, 2.2, Color::WHITE);
        }
    }
    (deck.build(), arches.build())
}

/// Un cartel con un rótulo, sobre dos postes detrás del muro, mirando a los
/// que llegan.
fn spawn_billboard(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    billboard: &Billboard,
    assets: &mut Assets3d,
) {
    const WIDTH: f32 = 12.0;
    const HEIGHT: f32 = 3.0;
    const LIFT: f32 = 1.4;
    let index = track.index_at(billboard.along);
    let lateral = billboard.side.sign() * (WALL_DISTANCE + 3.0);
    let point = track.offset_point(index, lateral);
    let base = terrain.height(point);
    let (_, heading) = track.pose_at(index);
    // Girado hacia la carretera para que se lea al llegar.
    let turn = billboard.side.sign() * 0.5;
    let image = sign_image(
        billboard.text,
        billboard.color,
        billboard.accent,
        Color::WHITE,
        Some(billboard.color),
        WIDTH / HEIGHT,
    );
    let mut panel = MeshBuilder::default();
    panel.quad(
        [
            (Vec3::new(-WIDTH / 2.0, LIFT, 0.0), Vec2::new(0.0, 1.0)),
            (Vec3::new(WIDTH / 2.0, LIFT, 0.0), Vec2::new(1.0, 1.0)),
            (
                Vec3::new(WIDTH / 2.0, LIFT + HEIGHT, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (Vec3::new(-WIDTH / 2.0, LIFT + HEIGHT, 0.0), Vec2::ZERO),
        ],
        Vec3::Z,
        Color::WHITE,
    );
    let mut back = MeshBuilder::default();
    back.beam(
        Vec3::new(-WIDTH / 2.0, LIFT + HEIGHT / 2.0, -0.08),
        Vec3::new(WIDTH / 2.0, LIFT + HEIGHT / 2.0, -0.08),
        0.12,
        HEIGHT,
        Color::srgb(0.85, 0.85, 0.85),
    );
    for x in [-WIDTH / 3.0, WIDTH / 3.0] {
        back.beam(
            Vec3::new(x, -1.0, -0.2),
            Vec3::new(x, LIFT + HEIGHT * 0.9, -0.2),
            0.2,
            0.2,
            Color::srgb(0.4, 0.4, 0.42),
        );
    }
    world.spawn((
        Transform::from_xyz(point.x, base, point.y)
            .with_rotation(Quat::from_rotation_y(heading + turn)),
        Visibility::default(),
        children![
            (
                Mesh3d(assets.meshes.add(panel.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    base_color_texture: Some(assets.images.add(image)),
                    perceptual_roughness: 0.6,
                    ..default()
                })),
            ),
            (
                Mesh3d(assets.meshes.add(back.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    perceptual_roughness: 0.7,
                    ..default()
                })),
            ),
        ],
    ));
}

/// Cartel de curva con flechas reflectantes (<<< o >>>) de advertencia en curvas cerradas.
fn spawn_chevron(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    chevron: &ChevronSign,
    assets: &mut Assets3d,
) {
    const WIDTH: f32 = 4.8;
    const HEIGHT: f32 = 1.6;
    const LIFT: f32 = 1.0;
    let index = track.index_at(chevron.along);
    let lateral = chevron.side.sign() * (WALL_DISTANCE + 2.4);
    let point = track.offset_point(index, lateral);
    let base = terrain.height(point);
    let (_, heading) = track.pose_at(index);
    let turn = chevron.side.sign() * 0.4;
    let text = match chevron.direction {
        ChevronDirection::Left => "<<<",
        ChevronDirection::Right => ">>>",
    };
    let yellow = Color::srgb(0.98, 0.84, 0.05);
    let black = Color::srgb(0.08, 0.08, 0.1);
    let image = sign_image(text, black, None, yellow, Some(black), WIDTH / HEIGHT);
    let mut panel = MeshBuilder::default();
    panel.quad(
        [
            (Vec3::new(-WIDTH / 2.0, LIFT, 0.0), Vec2::new(0.0, 1.0)),
            (Vec3::new(WIDTH / 2.0, LIFT, 0.0), Vec2::new(1.0, 1.0)),
            (
                Vec3::new(WIDTH / 2.0, LIFT + HEIGHT, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (Vec3::new(-WIDTH / 2.0, LIFT + HEIGHT, 0.0), Vec2::ZERO),
        ],
        Vec3::Z,
        Color::WHITE,
    );
    let mut back = MeshBuilder::default();
    for x in [-WIDTH / 3.0, WIDTH / 3.0] {
        back.beam(
            Vec3::new(x, -0.6, -0.15),
            Vec3::new(x, LIFT + HEIGHT * 0.8, -0.15),
            0.15,
            0.15,
            Color::srgb(0.5, 0.52, 0.55),
        );
    }
    world.spawn((
        Transform::from_xyz(point.x, base, point.y)
            .with_rotation(Quat::from_rotation_y(heading + turn)),
        Visibility::default(),
        children![
            (
                Mesh3d(assets.meshes.add(panel.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    base_color_texture: Some(assets.images.add(image)),
                    perceptual_roughness: 0.5,
                    ..default()
                })),
            ),
            (
                Mesh3d(assets.meshes.add(back.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    perceptual_roughness: 0.6,
                    ..default()
                })),
            ),
        ],
    ));
}

/// Cartel indicador de distancia a curva (100 m, 50 m) antes de frenadas fuertes.
fn spawn_brake_marker(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    marker: &BrakeMarker,
    assets: &mut Assets3d,
) {
    const WIDTH: f32 = 2.2;
    const HEIGHT: f32 = 1.8;
    const LIFT: f32 = 0.9;
    let index = track.index_at(marker.along);
    let lateral = marker.side.sign() * (WALL_DISTANCE + 2.0);
    let point = track.offset_point(index, lateral);
    let base = terrain.height(point);
    let (_, heading) = track.pose_at(index);
    let turn = marker.side.sign() * 0.35;
    let text = if marker.distance >= 100 { "100" } else { "50" };
    let white = Color::srgb(0.96, 0.96, 0.96);
    let black = Color::srgb(0.08, 0.08, 0.1);
    let image = sign_image(text, black, None, white, Some(black), WIDTH / HEIGHT);
    let mut panel = MeshBuilder::default();
    panel.quad(
        [
            (Vec3::new(-WIDTH / 2.0, LIFT, 0.0), Vec2::new(0.0, 1.0)),
            (Vec3::new(WIDTH / 2.0, LIFT, 0.0), Vec2::new(1.0, 1.0)),
            (
                Vec3::new(WIDTH / 2.0, LIFT + HEIGHT, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (Vec3::new(-WIDTH / 2.0, LIFT + HEIGHT, 0.0), Vec2::ZERO),
        ],
        Vec3::Z,
        Color::WHITE,
    );
    let mut back = MeshBuilder::default();
    for x in [-WIDTH / 4.0, WIDTH / 4.0] {
        back.beam(
            Vec3::new(x, -0.6, -0.12),
            Vec3::new(x, LIFT + HEIGHT * 0.8, -0.12),
            0.12,
            0.12,
            Color::srgb(0.45, 0.48, 0.5),
        );
    }
    world.spawn((
        Transform::from_xyz(point.x, base, point.y)
            .with_rotation(Quat::from_rotation_y(heading + turn)),
        Visibility::default(),
        children![
            (
                Mesh3d(assets.meshes.add(panel.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    base_color_texture: Some(assets.images.add(image)),
                    perceptual_roughness: 0.5,
                    ..default()
                })),
            ),
            (
                Mesh3d(assets.meshes.add(back.build())),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    perceptual_roughness: 0.6,
                    ..default()
                })),
            ),
        ],
    ));
}
