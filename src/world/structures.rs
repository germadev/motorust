//! The works along the road: tunnels cut through the rock, bridges over the
//! water (with red arches, or of wood) and billboards.

use bevy::prelude::*;

use super::Assets3d;
use super::lettering::sign_image;
use super::quads::MeshBuilder;
use super::scenery::{
    Billboard, BrakeMarker, BridgeStyle, ChevronDirection, ChevronSign, Scenery, Soil, Span,
};
use super::terrain::{CLIFF_NORMAL_Y, GRASS_TILE, Ground, Terrain, ground_vertex_color};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Nearest, SURFACE_Y, Track, WALL_DISTANCE};

/// Alto de las paredes del túnel hasta el techo (m), y lo que se abomba la
/// roca de encima.
const TUNNEL_HEIGHT: f32 = 6.5;
const TUNNEL_ROCK: f32 = 15.0;
/// Hasta dónde llega la roca del túnel a cada lado de la línea central (m).
pub const TUNNEL_SHELL: f32 = WALL_DISTANCE + 12.0;
/// Points of the centre line at least this many apart along the lap are on
/// different stretches of the track: one can pass over a tunnel of the
/// other.
const OTHER_STRETCH: usize = 60;
/// Where another stretch passes over a tunnel, the rock carries it: just
/// below its asphalt as far as this from its centre line (m), and down to
/// the rest of the rock over this much further.
const DECK_FLAT: f32 = WALL_DISTANCE + 1.5;
const DECK_SLOPE: f32 = 12.0;
const DECK_DROP: f32 = 0.3;
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
/// What each repetition of the concrete texture covers on the deck (m).
const DECK_TILE: f32 = 4.0;
/// Wooden bridges: what each repetition of the planks covers (m), how far
/// above the asphalt they lie (over its paint), how tall the fence is (m)
/// and the colours of the wood.
const PLANK_TILE: f32 = 4.0;
const PLANK_LIFT: f32 = 0.02;
const FENCE_HEIGHT: f32 = 1.2;
const POST_WOOD: Color = Color::srgb(0.34, 0.2, 0.09);
const RAIL_WOOD: Color = Color::srgb(0.52, 0.33, 0.15);

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
        let parts = tunnel_meshes(track, terrain, scenery.soil, &indices);
        let rock = match scenery.soil {
            Soil::Rock => StandardMaterial {
                base_color: Color::srgb(0.85, 0.75, 0.6),
                base_color_texture: Some(assets.textures.rock.clone()),
                perceptual_roughness: 0.9,
                ..default()
            },
            Soil::Checkered => StandardMaterial {
                base_color_texture: Some(assets.textures.soil.clone()),
                perceptual_roughness: 0.9,
                ..default()
            },
        };
        // Warm and lit by the lamps, as in Moto Racer; through checkerboard
        // earth, a cave of it.
        let lining = match scenery.soil {
            Soil::Rock => StandardMaterial {
                base_color: Color::srgb(0.95, 0.78, 0.52),
                base_color_texture: Some(assets.textures.panels.clone()),
                emissive: LinearRgba::rgb(0.12, 0.08, 0.035),
                perceptual_roughness: 0.85,
                ..default()
            },
            Soil::Checkered => StandardMaterial {
                base_color_texture: Some(assets.textures.soil.clone()),
                emissive: LinearRgba::rgb(0.1, 0.06, 0.02),
                perceptual_roughness: 0.9,
                ..default()
            },
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
        let grass = StandardMaterial {
            base_color_texture: Some(assets.textures.grass.clone()),
            perceptual_roughness: 0.9,
            reflectance: 0.18,
            ..default()
        };
        for (mesh, material) in [
            (parts.rock, rock),
            (parts.grass, grass),
            (parts.lining, lining),
            (parts.portals, concrete.clone()),
            (parts.sidewalks, concrete),
            (parts.lights, lights),
            (parts.checker, checker),
        ] {
            // Through checkerboard earth, no grass grows on the top.
            if mesh.count_vertices() == 0 {
                continue;
            }
            world.spawn((
                Mesh3d(assets.meshes.add(mesh)),
                MeshMaterial3d(assets.materials.add(material)),
            ));
        }
    }
    for bridge in scenery.bridges {
        let indices = indices(track, *bridge, lap);
        if scenery.bridge_style == BridgeStyle::Wooden {
            let (planks, logs) = wooden_bridge_meshes(track, terrain, &indices);
            world.spawn((
                Mesh3d(assets.meshes.add(planks)),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    base_color_texture: Some(assets.textures.planks.clone()),
                    perceptual_roughness: 0.85,
                    ..default()
                })),
            ));
            world.spawn((
                Mesh3d(assets.meshes.add(logs)),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    perceptual_roughness: 0.9,
                    ..default()
                })),
            ));
            continue;
        }
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

/// The tunnels of a track, as runs of segments of its centre line: the
/// first point and how many segments.
pub struct Tunnels(Vec<(usize, usize)>);

impl Tunnels {
    pub fn new(track: &Track, scenery: &Scenery) -> Self {
        let lap = track.lap_length();
        Self(
            scenery
                .tunnels
                .iter()
                .map(|&span| {
                    let indices = indices(track, span, lap);
                    (indices[0], indices.len() - 1)
                })
                .collect(),
        )
    }

    /// Where another stretch of the track passes over a tunnel and `point`
    /// is within `reach` of the road through it, under its rock (at most
    /// [`TUNNEL_SHELL`]), near that stretch: the point of the road through
    /// the tunnel nearest to it. The ground stays down there, at the level of
    /// the tunnel, and the rock carries the road above (see
    /// [`tunnel_meshes`]).
    pub fn beneath(&self, track: &Track, point: Vec2, reach: f32) -> Option<Nearest> {
        self.0.iter().find_map(|&(from, count)| {
            let road = track.nearest_among(point, from, count);
            if road.distance >= reach {
                return None;
            }
            let over = passing_over(track, from, count, point, track.height_beside(&road))?;
            (over.distance < TUNNEL_SHELL).then_some(road)
        })
    }
}

/// The point nearest to `point` of the stretch of the track that passes
/// over the tunnel through the `count` segments from `from` on, whose road
/// is at `height` there: the nearest away from the tunnel and its
/// approaches, if it is above the tunnel's rock.
fn passing_over(
    track: &Track,
    from: usize,
    count: usize,
    point: Vec2,
    height: f32,
) -> Option<Nearest> {
    let n = track.sample_count();
    let skipped = (count + 2 * OTHER_STRETCH).min(n - 1);
    let over = track.nearest_among(point, (from + count + OTHER_STRETCH) % n, n - skipped);
    (track.height_beside(&over) > height + TUNNEL_HEIGHT + 2.0).then_some(over)
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
    /// Grass on the gentle parts of the top.
    grass: Mesh,
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
fn tunnel_meshes(track: &Track, terrain: &Terrain, soil: Soil, indices: &[usize]) -> TunnelParts {
    let mut rock = MeshBuilder::default();
    let mut grass = MeshBuilder::default();
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
    // Where another stretch of the road passes over the tunnel, the rock
    // carries it: just below its asphalt, and sloping down to the rest of
    // the rock on either side.
    let (first, count) = (indices[0], indices.len() - 1);
    let deck = |i: usize, lateral: f32| {
        let ground = track.offset_point(i, lateral);
        let over = passing_over(track, first, count, ground, road(i))?;
        let reach = 1.0 - smoothstep(DECK_FLAT, DECK_FLAT + DECK_SLOPE, over.distance);
        (reach > 0.0).then(|| (track.height_beside(&over) - DECK_DROP - road(i), reach))
    };
    let rock_top = |i: usize, lateral: f32| {
        let t = (lateral / TUNNEL_SHELL + 1.0) / 2.0;
        let ground = track.surface_point(i, lateral);
        let rugged = fbm(Vec2::new(ground.x, ground.z) / 19.0, 3);
        let hump = TUNNEL_ROCK * 0.2 * (1.0 - (2.0 * t - 1.0).powi(2)) * (0.5 + rugged);
        let top = edge(i, -1.0).lerp(edge(i, 1.0), t) + hump;
        deck(i, lateral).map_or(top, |(deck, reach)| {
            // Over the ground that rises towards the road above, too, but
            // never over that road.
            let earth = terrain.height(Vec2::new(ground.x, ground.z)) - road(i);
            top.lerp(deck, reach).max((earth + 0.05).min(deck))
        })
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
        const STRIPS: usize = 12;
        for k in 0..STRIPS {
            let (a, b) = (
                -TUNNEL_SHELL + 2.0 * TUNNEL_SHELL * k as f32 / STRIPS as f32,
                -TUNNEL_SHELL + 2.0 * TUNNEL_SHELL * (k + 1) as f32 / STRIPS as f32,
            );
            let corners = [
                (at(i, a, rock_top(i, a)), Vec2::new(vi, a / 4.0)),
                (at(j, a, rock_top(j, a)), Vec2::new(vj, a / 4.0)),
                (at(j, b, rock_top(j, b)), Vec2::new(vj, b / 4.0)),
                (at(i, b, rock_top(i, b)), Vec2::new(vi, b / 4.0)),
            ];
            // Through natural rock, grass grows where the top is gentle, as
            // on the hills around.
            let [p, q, r, _] = corners.map(|(p, _)| p);
            let face = (q - p).cross(r - p).normalize_or_zero();
            let face = if face.y < 0.0 { -face } else { face };
            if soil == Soil::Rock && face.y > CLIFF_NORMAL_Y {
                let center = corners.iter().map(|(p, _)| *p).sum::<Vec3>() / 4.0;
                let tint = ground_vertex_color(Ground::Grass, center, face);
                grass.quad(
                    corners.map(|(p, _)| (p, Vec2::new(p.x, p.z) / GRASS_TILE)),
                    Vec3::Y,
                    Color::LinearRgba(LinearRgba::from_f32_array(tint)),
                );
            } else {
                rock.quad(corners, Vec3::Y, Color::WHITE);
            }
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
        grass: grass.build(),
        lining: lining.build(),
        portals: portals.build(),
        sidewalks: sidewalks.build(),
        lights: lights.build(),
        checker: checker.build(),
    }
}

/// El tablero de un puente (de hormigón, por debajo de la carretera) y sus
/// dos arcos rojos, uno a cada lado, con los tirantes que cuelgan de ellos.
/// The deck's top runs out from under the asphalt to past the walls: off
/// the bridge the ground does that, but under a bridge it drops away.
fn bridge_meshes(track: &Track, terrain: &Terrain, indices: &[usize]) -> (Mesh, Mesh) {
    let mut deck = MeshBuilder::default();
    let mut arches = MeshBuilder::default();
    let edge = WALL_DISTANCE + 0.8;
    // Tucked a little under the asphalt, so no crack shows between them.
    let inner = track.half_width() - 0.3;
    let road = |i: usize, lateral: f32| track.surface_point(i, lateral);
    for (k, pair) in indices.windows(2).enumerate() {
        let (i, j) = (pair[0], pair[1]);
        let down = Vec3::Y * DECK_DEPTH;
        // The top, beside the asphalt, on each side.
        let (across, along) = (
            (edge - inner) / DECK_TILE,
            road(i, 0.0).distance(road(j, 0.0)) / DECK_TILE,
        );
        for side in [-1.0, 1.0] {
            deck.quad(
                [
                    (road(i, side * inner), Vec2::ZERO),
                    (road(j, side * inner), Vec2::new(0.0, along)),
                    (road(j, side * edge), Vec2::new(across, along)),
                    (road(i, side * edge), Vec2::new(across, 0.0)),
                ],
                Vec3::Y,
                Color::WHITE,
            );
        }
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

/// A wooden bridge: planks laid across from fence to fence over the road and
/// on the sides and underside of the deck (the first mesh), and the logs
/// (the second): a fence of posts and two rails on each side, and piers of
/// two logs and a crossbeam down to the water.
fn wooden_bridge_meshes(track: &Track, terrain: &Terrain, indices: &[usize]) -> (Mesh, Mesh) {
    let mut planks = MeshBuilder::default();
    let mut logs = MeshBuilder::default();
    let edge = WALL_DISTANCE + 0.8;
    let step = track.step_length();
    let road = |i: usize, lateral: f32| track.surface_point(i, lateral);
    let deck = |i: usize, lateral: f32| road(i, lateral) + Vec3::Y * (SURFACE_Y + PLANK_LIFT);
    let down = Vec3::Y * (DECK_DEPTH + SURFACE_Y + PLANK_LIFT);
    let across = 2.0 * edge / PLANK_TILE;
    for (k, pair) in indices.windows(2).enumerate() {
        let (i, j) = (pair[0], pair[1]);
        let (v0, v1) = (
            k as f32 * step / PLANK_TILE,
            (k + 1) as f32 * step / PLANK_TILE,
        );
        // The boards run across the road: along them is U.
        for (lift, facing) in [(Vec3::ZERO, Vec3::Y), (down, Vec3::NEG_Y)] {
            planks.quad(
                [
                    (deck(i, -edge) - lift, Vec2::new(0.0, v0)),
                    (deck(j, -edge) - lift, Vec2::new(0.0, v1)),
                    (deck(j, edge) - lift, Vec2::new(across, v1)),
                    (deck(i, edge) - lift, Vec2::new(across, v0)),
                ],
                facing,
                Color::WHITE,
            );
        }
        for side in [-1.0, 1.0] {
            let out = road(i, side * (edge + 1.0)) - road(i, side * edge);
            let thick = DECK_DEPTH / PLANK_TILE;
            planks.quad(
                [
                    (deck(i, side * edge), Vec2::new(0.0, v0)),
                    (deck(j, side * edge), Vec2::new(0.0, v1)),
                    (deck(j, side * edge) - down, Vec2::new(thick, v1)),
                    (deck(i, side * edge) - down, Vec2::new(thick, v0)),
                ],
                out,
                Color::WHITE,
            );
            // The fence: a post every other point, rails between them.
            let fence =
                |i: usize, height: f32| deck(i, side * (WALL_DISTANCE + 0.2)) + Vec3::Y * height;
            for (at, wanted) in [(i, k % 2 == 0), (j, k + 2 == indices.len())] {
                if !wanted {
                    continue;
                }
                logs.beam(
                    fence(at, -0.3),
                    fence(at, FENCE_HEIGHT),
                    0.3,
                    0.3,
                    POST_WOOD,
                );
            }
            for height in [FENCE_HEIGHT - 0.12, FENCE_HEIGHT * 0.5] {
                logs.beam(fence(i, height), fence(j, height), 0.17, 0.17, RAIL_WOOD);
            }
        }
    }
    // Piers at the ends, at the quarters and in the middle.
    let last = indices.len() - 1;
    for k in [0, last / 4, last / 2, 3 * last / 4, last] {
        let i = indices[k];
        let mut tops = [Vec3::ZERO; 2];
        for (top, side) in tops.iter_mut().zip([-1.0, 1.0]) {
            *top = road(i, side * (edge - 1.2)) - Vec3::Y * DECK_DEPTH;
            let ground = terrain.height(Vec2::new(top.x, top.z)).min(top.y) - 2.0;
            logs.beam(Vec3::new(top.x, ground, top.z), *top, 1.1, 1.1, POST_WOOD);
        }
        logs.beam(
            tops[0] - Vec3::Y * 0.6,
            tops[1] - Vec3::Y * 0.6,
            0.7,
            0.7,
            RAIL_WOOD,
        );
    }
    (planks.build(), logs.build())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::CIRCUITS;
    use bevy::mesh::VertexAttributeValues;

    /// The height of the highest upward-facing triangle of `mesh` over
    /// `point` (x and z), if any.
    fn top_at(mesh: &Mesh, point: Vec2) -> Option<f32> {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return None;
        };
        let indices: Vec<usize> = mesh.indices()?.iter().collect();
        indices
            .chunks(3)
            .filter_map(|triangle| {
                let [a, b, c] = [0, 1, 2].map(|n| Vec3::from(positions[triangle[n]]));
                let normal = (b - a).cross(c - a).normalize_or_zero();
                if normal.y < 0.5 {
                    return None;
                }
                // Barycentric coordinates of the point in the triangle, seen
                // from above.
                let flat = |v: Vec3| Vec2::new(v.x, v.z);
                let (a2, b2, c2) = (flat(a), flat(b), flat(c));
                let area = (b2 - a2).perp_dot(c2 - a2);
                let u = (b2 - point).perp_dot(c2 - point) / area;
                let v = (c2 - point).perp_dot(a2 - point) / area;
                let w = 1.0 - u - v;
                (u >= 0.0 && v >= 0.0 && w >= 0.0).then_some(a.y * u + b.y * v + c.y * w)
            })
            .reduce(f32::max)
    }

    #[test]
    fn red_bridges_have_a_deck_out_to_the_walls() {
        let mut checked = 0;
        for circuit in &CIRCUITS {
            let scenery = &circuit.scenery;
            if scenery.bridge_style != BridgeStyle::RedArches {
                continue;
            }
            let track = Track::new(&circuit.layout);
            let terrain = Terrain::new(&track, scenery);
            let lap = track.lap_length();
            for bridge in scenery.bridges {
                let indices = indices(&track, *bridge, lap);
                let (deck, _) = bridge_meshes(&track, &terrain, &indices);
                let middle = indices[indices.len() / 2];
                let beside = track.half_width() + 0.5;
                let by_the_wall = WALL_DISTANCE - 0.3;
                for lateral in [-by_the_wall, -beside, beside, by_the_wall] {
                    let road = track.surface_point(middle, lateral);
                    let top = top_at(&deck, Vec2::new(road.x, road.z)).unwrap_or_else(|| {
                        panic!("{}: a hole {lateral} m from the centre", circuit.name)
                    });
                    assert!((top - road.y).abs() < 0.1, "{top} {}", road.y);
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "some circuit has a red bridge");
    }
}
