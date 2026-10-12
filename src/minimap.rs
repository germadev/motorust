//! Plano de la pista: el trazado visto desde arriba (el norte, -Z, arriba),
//! dibujado a partir de la línea central cada vez que cambia la pista. Lo
//! usan la pantalla de elegir pista y el minimapa de la carrera, abajo a la
//! izquierda, con un punto por moto del color de cada una.

use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::bike::{BikeColor, BikeState, LocalPlayer, Player};
use crate::circuits::CurrentCircuit;
use crate::menu::GameState;
use crate::track::Track;
use crate::welcome::GameMode;

/// Lado del plano (píxeles) y margen alrededor del trazado.
pub const MAP_SIZE: u32 = 256;
const MAP_MARGIN: f32 = 18.0;
/// Grosor del trazado y de su borde (px).
const MAP_ROAD: f32 = 3.0;
const MAP_OUTLINE: f32 = 5.5;
/// How much thinner the roads off the lap are drawn (px).
const MAP_SIDE_ROAD: f32 = 1.0;

struct MarkerStyle {
    half_len: f32,
    radius: f32,
    color: [f32; 4],
}

/// Marca de la meta (línea amarilla) y de los checkpoints (líneas rojas más finas).
const START_STYLE: MarkerStyle = MarkerStyle {
    half_len: 5.5,
    radius: 1.6,
    color: [1.0, 0.85, 0.1, 1.0],
};

const CHECKPOINT_STYLE: MarkerStyle = MarkerStyle {
    half_len: 4.8,
    radius: 0.9,
    color: [0.95, 0.15, 0.15, 1.0],
};

/// Lado del minimapa de la carrera en pantalla y de los puntos de las motos
/// (px): el del jugador, más grande y con borde blanco.
const HUD_SIZE: f32 = 190.0;
const RIVAL_DOT: f32 = 9.0;
const PLAYER_DOT: f32 = 14.0;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, create_map)
            .add_systems(Update, redraw_map)
            .add_systems(OnEnter(GameState::Race), spawn_hud_map)
            .add_systems(
                Update,
                place_dots
                    .run_if(in_state(GameState::Race))
                    .after(redraw_map),
            );
    }
}

/// El plano de la pista actual, siempre en la misma imagen: al cambiar de
/// pista se vuelve a dibujar encima.
#[derive(Resource)]
pub struct TrackMap {
    pub image: Handle<Image>,
    pub projection: Projection,
}

/// Cómo se pasa del suelo (m) al plano (px).
#[derive(Clone, Copy)]
pub struct Projection {
    pub min: Vec2,
    pub scale: f32,
    pub offset: Vec2,
}

impl Projection {
    pub fn new(track: &Track) -> Self {
        let (min, max) = track.bounds();
        let scale = (MAP_SIZE as f32 - 2.0 * MAP_MARGIN) / (max - min).max_element();
        let offset = (Vec2::splat(MAP_SIZE as f32) - (max - min) * scale) / 2.0;
        Self { min, scale, offset }
    }

    fn to_map(self, point: Vec2) -> Vec2 {
        (point - self.min) * self.scale + self.offset
    }
}

fn create_map(
    mut commands: Commands,
    circuit: Option<Res<CurrentCircuit>>,
    track: Res<Track>,
    mut images: ResMut<Assets<Image>>,
) {
    let checkpoints = circuit.map(|c| c.get().checkpoints()).unwrap_or_default();
    commands.insert_resource(TrackMap {
        image: images.add(draw(&track, &checkpoints)),
        projection: Projection::new(&track),
    });
}

fn redraw_map(
    circuit: Option<Res<CurrentCircuit>>,
    track: Res<Track>,
    mut map: ResMut<TrackMap>,
    mut images: ResMut<Assets<Image>>,
) {
    let circuit_changed = circuit.as_ref().is_some_and(|c| c.is_changed());
    if (!track.is_changed() && !circuit_changed) || track.is_added() {
        return;
    }
    let checkpoints = circuit.map(|c| c.get().checkpoints()).unwrap_or_default();
    map.projection = Projection::new(&track);
    images
        .insert(&map.image, draw(&track, &checkpoints))
        .expect("la imagen del plano sigue existiendo");
}

/// El trazado en blanco con un borde oscuro, el inicio con una línea amarilla
/// transversal y los checkpoints con líneas rojas más finas, sobre fondo
/// transparente.
pub fn draw(track: &Track, checkpoints: &[f32]) -> Image {
    let size = MAP_SIZE as usize;
    let mut pixels = vec![[0.0f32; 4]; size * size];
    let projection = Projection::new(track);
    let centerline = track.centerline();
    let outline = [0.02, 0.04, 0.1, 0.95];
    let road = [1.0, 1.0, 1.0, 1.0];

    // The lap, and thinner, the roads off it: all the outlines first, so
    // that where they meet the roads are one.
    for (radius, color) in [(MAP_OUTLINE, outline), (MAP_ROAD, road)] {
        for line in std::iter::once(track).chain(track.side_roads()) {
            let points = line.centerline();
            let thinner = if line.is_closed() { 0.0 } else { MAP_SIDE_ROAD };
            for index in 0..line.segment_count() {
                let next = points[(index + 1) % points.len()];
                let (a, b) = (projection.to_map(points[index]), projection.to_map(next));
                // Un círculo cada medio píxel a lo largo del tramo.
                let steps = (a.distance(b) * 2.0).ceil().max(1.0) as usize;
                for step in 0..steps {
                    let at = a.lerp(b, step as f32 / steps as f32);
                    stamp(&mut pixels, size, at, radius - thinner, color);
                }
            }
        }
    }

    // Checkpoints: líneas rojas más finas transversales a la pista.
    for &along in checkpoints {
        let index = track.index_at(along);
        let (cp_center, cp_tangent) = marker_tangent(centerline, index, projection);
        draw_track_marker(
            &mut pixels,
            size,
            cp_center,
            cp_tangent,
            CHECKPOINT_STYLE,
            outline,
        );
    }

    // Inicio: línea amarilla transversal a la pista.
    let (goal, start_tangent) = marker_tangent(centerline, 0, projection);
    draw_track_marker(&mut pixels, size, goal, start_tangent, START_STYLE, outline);

    let data = pixels
        .iter()
        .flat_map(|pixel| pixel.map(|channel| (channel * 255.0).round() as u8))
        .collect();
    Image::new(
        Extent3d {
            width: MAP_SIZE,
            height: MAP_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn marker_tangent(centerline: &[Vec2], index: usize, projection: Projection) -> (Vec2, Vec2) {
    let n = centerline.len();
    let center = projection.to_map(centerline[index % n]);
    let prev = projection.to_map(centerline[(index + n - 1) % n]);
    let next = projection.to_map(centerline[(index + 1) % n]);
    let tangent = (next - prev).normalize_or_zero();
    (center, tangent)
}

fn draw_track_marker(
    pixels: &mut [[f32; 4]],
    size: usize,
    center: Vec2,
    tangent: Vec2,
    style: MarkerStyle,
    outline: [f32; 4],
) {
    let normal = Vec2::new(-tangent.y, tangent.x);
    let a = center - normal * style.half_len;
    let b = center + normal * style.half_len;
    draw_line(pixels, size, a, b, style.radius + 1.0, outline);
    draw_line(pixels, size, a, b, style.radius, style.color);
}

fn draw_line(pixels: &mut [[f32; 4]], size: usize, a: Vec2, b: Vec2, radius: f32, color: [f32; 4]) {
    let steps = (a.distance(b) * 2.0).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let at = a.lerp(b, step as f32 / steps as f32);
        stamp(pixels, size, at, radius, color);
    }
}

/// Pinta un círculo de `color` con el borde suavizado.
fn stamp(pixels: &mut [[f32; 4]], size: usize, center: Vec2, radius: f32, color: [f32; 4]) {
    let low = (center - radius - 1.0).max(Vec2::ZERO);
    let high = (center + radius + 1.0).min(Vec2::splat(size as f32 - 1.0));
    for y in low.y as usize..=high.y as usize {
        for x in low.x as usize..=high.x as usize {
            let pixel_center = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let cover = (radius + 0.5 - pixel_center.distance(center)).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let pixel = &mut pixels[y * size + x];
            for channel in 0..3 {
                pixel[channel] += (color[channel] - pixel[channel]) * cover;
            }
            pixel[3] = pixel[3].max(color[3] * cover);
        }
    }
}

/// El minimapa de la carrera, abajo a la izquierda.
#[derive(Component)]
struct HudMap;

/// El punto de una moto en el minimapa.
#[derive(Component)]
struct Dot(Entity);

fn spawn_hud_map(mut commands: Commands, map: Res<TrackMap>, game_mode: Res<GameMode>) {
    let (node, border) = if *game_mode == GameMode::SplitScreen {
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: percent(50.0),
                margin: UiRect::top(px(-HUD_SIZE / 2.0)),
                width: px(HUD_SIZE),
                height: px(HUD_SIZE),
                border_radius: BorderRadius::all(px(14)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.25, 0.35, 0.5, 0.7)),
        )
    } else {
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                bottom: px(16),
                width: px(HUD_SIZE),
                height: px(HUD_SIZE),
                border_radius: BorderRadius::all(px(14)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.2, 0.25, 0.35, 0.4)),
        )
    };

    commands.spawn((
        HudMap,
        DespawnOnExit(GameState::Race),
        node,
        border,
        GlobalZIndex(10),
        BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.65)),
        children![(
            ImageNode::new(map.image.clone()),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        )],
    ));
}

/// Un punto por moto, donde está, del color de la moto; el del jugador,
/// encima de los demás. Los puntos se crean la primera vez que se ve cada
/// moto.
#[allow(clippy::type_complexity)]
fn place_dots(
    mut commands: Commands,
    map: Res<TrackMap>,
    hud: Single<Entity, With<HudMap>>,
    bikes: Query<(
        Entity,
        &BikeState,
        &BikeColor,
        Has<Player>,
        Option<&LocalPlayer>,
    )>,
    mut dots: Query<(&Dot, &mut Node)>,
) {
    let mut placed: HashMap<Entity, Mut<Node>> =
        dots.iter_mut().map(|(dot, node)| (dot.0, node)).collect();
    for (bike, state, color, player, local) in &bikes {
        let is_human = player || local.is_some();
        let size = if is_human { PLAYER_DOT } else { RIVAL_DOT };
        let at = map.projection.to_map(state.position) / MAP_SIZE as f32 * HUD_SIZE - size / 2.0;
        match placed.get_mut(&bike) {
            Some(node) => {
                // Solo si se ha movido: tocar el nodo obliga a recolocarlo.
                if node.left != px(at.x) || node.top != px(at.y) {
                    node.left = px(at.x);
                    node.top = px(at.y);
                }
            }
            None => {
                let dot = commands
                    .spawn((
                        Dot(bike),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(at.x),
                            top: px(at.y),
                            width: px(size),
                            height: px(size),
                            border: UiRect::all(px(if is_human { 2.5 } else { 1.0 })),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(color.0),
                        BorderColor::all(if is_human {
                            Color::WHITE
                        } else {
                            Color::srgba(0.0, 0.0, 0.0, 0.8)
                        }),
                        ZIndex(i32::from(is_human)),
                    ))
                    .id();
                commands.entity(*hud).add_child(dot);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_shows_the_whole_track_inside_its_margins() {
        let image = draw(&Track::default(), &[780.0]);
        let data = image.data.as_ref().unwrap();
        let size = MAP_SIZE as usize;
        let alpha = |x: usize, y: usize| data[(y * size + x) * 4 + 3];
        // Hay trazado dibujado...
        let drawn = (0..size * size).filter(|i| data[i * 4 + 3] > 200).count();
        assert!(drawn > size * 4, "{drawn} píxeles");
        // ...y no toca los bordes.
        for i in 0..size {
            for (x, y) in [(i, 0), (i, size - 1), (0, i), (size - 1, i)] {
                assert_eq!(alpha(x, y), 0, "borde en ({x}, {y})");
            }
        }
    }

    #[test]
    fn the_map_draws_start_line_and_checkpoints() {
        let track = Track::default();
        let image = draw(&track, &[780.0]);
        let data = image.data.as_ref().unwrap();
        let size = MAP_SIZE as usize;

        let mut yellow_pixels = 0;
        let mut red_pixels = 0;

        for i in 0..(size * size) {
            let r = data[i * 4];
            let g = data[i * 4 + 1];
            let b = data[i * 4 + 2];
            let a = data[i * 4 + 3];

            if a > 150 {
                // Línea de inicio amarilla: rojo y verde altos, azul bajo.
                if r > 180 && g > 150 && b < 100 {
                    yellow_pixels += 1;
                }
                // Checkpoint rojo: rojo alto, verde y azul bajos.
                if r > 180 && g < 100 && b < 100 {
                    red_pixels += 1;
                }
            }
        }

        assert!(
            yellow_pixels > 0,
            "Debe haber píxeles amarillos para la línea de inicio ({yellow_pixels})"
        );
        assert!(
            red_pixels > 0,
            "Debe haber píxeles rojos para los checkpoints ({red_pixels})"
        );
    }

    #[test]
    fn the_projection_keeps_the_track_inside_the_map() {
        let track = Track::default();
        let projection = Projection::new(&track);
        for &point in track.centerline() {
            let at = projection.to_map(point);
            assert!(at.cmpge(Vec2::splat(MAP_MARGIN - 0.5)).all());
            assert!(
                at.cmple(Vec2::splat(MAP_SIZE as f32 - MAP_MARGIN + 0.5))
                    .all()
            );
        }
    }
}
