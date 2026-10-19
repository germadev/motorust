//! The inside of a toy shop ([`Room`]): cream plaster walls with a pale
//! wooden wainscot, a beamed ceiling hung with glowing globe lamps, tall
//! shelves full of colourful toy boxes, and big painted signs. It is built
//! as a handful of batched meshes: the shell, the wooden trim and shelves,
//! and the glowing lamps, so the whole room costs a few draw calls.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Assets3d;
use super::lettering::sign_image;
use super::quads::MeshBuilder;
use super::scenery::Room;
use super::toys::box_at;
use crate::geometry::random01;

/// How big a repetition of the wall plaster and the wooden boards is (m).
const WALL_TILE: f32 = 14.0;
const WOOD_TILE: f32 = 9.0;
/// How high the wooden wainscot reaches (m), and the bottom and top of the
/// shelves (m).
const PLINTH: f32 = 16.0;
const SHELF_BOTTOM: f32 = 16.0;
const SHELF_TOP: f32 = 84.0;
/// How deep and wide a shelf unit is (m), and how many units there are along
/// each wall.
const SHELF_DEPTH: f32 = 14.0;
const SHELF_WIDTH: f32 = 230.0;
const SHELF_UNITS: usize = 3;

/// The colours of the room and the toy boxes on its shelves.
const CREAM: Color = Color::srgb(0.96, 0.93, 0.86);
const CEILING: Color = Color::srgb(0.97, 0.95, 0.9);
const WAINSCOT: Color = Color::srgb(0.86, 0.71, 0.52);
const WOOD: Color = Color::srgb(0.78, 0.59, 0.39);
const WOOD_DARK: Color = Color::srgb(0.55, 0.39, 0.24);
const IRON: Color = Color::srgb(0.24, 0.22, 0.22);
const LAMP: Color = Color::srgb(1.0, 0.92, 0.78);
const PLASTIC: [Color; 8] = [
    Color::srgb(0.9, 0.16, 0.12),
    Color::srgb(1.0, 0.8, 0.16),
    Color::srgb(0.14, 0.4, 0.9),
    Color::srgb(0.18, 0.68, 0.3),
    Color::srgb(0.96, 0.46, 0.1),
    Color::srgb(0.94, 0.92, 0.86),
    Color::srgb(0.72, 0.2, 0.62),
    Color::srgb(0.16, 0.72, 0.74),
];

/// Builds the room: its shell, trim, shelves and lamps.
pub fn spawn_room(world: &mut ChildSpawnerCommands, room: &Room, assets: &mut Assets3d) {
    let x0 = room.center.x - room.half.x;
    let x1 = room.center.x + room.half.x;
    let z0 = room.center.y - room.half.y;
    let z1 = room.center.y + room.half.y;
    let bottom = -24.0;
    let top = room.height;

    let plaster = assets.materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(assets.textures.plaster.clone()),
        perceptual_roughness: 0.95,
        ..default()
    });
    let boards = assets.materials.add(StandardMaterial {
        base_color_texture: Some(assets.textures.shop_planks.clone()),
        perceptual_roughness: 0.82,
        ..default()
    });
    let painted = assets.materials.add(StandardMaterial {
        perceptual_roughness: 0.78,
        ..default()
    });

    // The shell: the upper walls and the ceiling, of plaster.
    let mut shell = MeshBuilder::default();
    wall(
        &mut shell,
        x0,
        z1,
        x1,
        z1,
        PLINTH,
        top,
        Vec3::NEG_Z,
        WALL_TILE,
        CREAM,
    );
    wall(
        &mut shell,
        x1,
        z0,
        x0,
        z0,
        PLINTH,
        top,
        Vec3::Z,
        WALL_TILE,
        CREAM,
    );
    wall(
        &mut shell,
        x0,
        z0,
        x0,
        z1,
        PLINTH,
        top,
        Vec3::X,
        WALL_TILE,
        CREAM,
    );
    wall(
        &mut shell,
        x1,
        z1,
        x1,
        z0,
        PLINTH,
        top,
        Vec3::NEG_X,
        WALL_TILE,
        CREAM,
    );
    shell.quad(
        [
            (
                Vec3::new(x0, top, z0),
                Vec2::new(x0 / WALL_TILE, z0 / WALL_TILE),
            ),
            (
                Vec3::new(x1, top, z0),
                Vec2::new(x1 / WALL_TILE, z0 / WALL_TILE),
            ),
            (
                Vec3::new(x1, top, z1),
                Vec2::new(x1 / WALL_TILE, z1 / WALL_TILE),
            ),
            (
                Vec3::new(x0, top, z1),
                Vec2::new(x0 / WALL_TILE, z1 / WALL_TILE),
            ),
        ],
        Vec3::NEG_Y,
        CEILING,
    );
    world.spawn((
        Mesh3d(assets.meshes.add(shell.build())),
        MeshMaterial3d(plaster),
        NotShadowCaster,
    ));

    // The wainscot: pale boards all round, from the floor to the shelf line.
    let mut wainscot = MeshBuilder::default();
    wall(
        &mut wainscot,
        x0,
        z1,
        x1,
        z1,
        bottom,
        PLINTH,
        Vec3::NEG_Z,
        WOOD_TILE,
        WAINSCOT,
    );
    wall(
        &mut wainscot,
        x1,
        z0,
        x0,
        z0,
        bottom,
        PLINTH,
        Vec3::Z,
        WOOD_TILE,
        WAINSCOT,
    );
    wall(
        &mut wainscot,
        x0,
        z0,
        x0,
        z1,
        bottom,
        PLINTH,
        Vec3::X,
        WOOD_TILE,
        WAINSCOT,
    );
    wall(
        &mut wainscot,
        x1,
        z1,
        x1,
        z0,
        bottom,
        PLINTH,
        Vec3::NEG_X,
        WOOD_TILE,
        WAINSCOT,
    );
    world.spawn((
        Mesh3d(assets.meshes.add(wainscot.build())),
        MeshMaterial3d(boards),
        NotShadowCaster,
    ));

    // The wooden trim: baseboard, chair rail, crown, corner posts and beams.
    let mut trim = MeshBuilder::default();
    let up = Vec3::Y;
    for (a, b, inward) in [
        (Vec2::new(x0, z1), Vec2::new(x1, z1), Vec3::NEG_Z),
        (Vec2::new(x1, z0), Vec2::new(x0, z0), Vec3::Z),
        (Vec2::new(x0, z0), Vec2::new(x0, z1), Vec3::X),
        (Vec2::new(x1, z1), Vec2::new(x1, z0), Vec3::NEG_X),
    ] {
        let along = Vec3::new(b.x - a.x, 0.0, b.y - a.y);
        let length = along.length();
        let along = along.normalize_or_zero();
        let middle = Vec3::new((a.x + b.x) / 2.0, 0.0, (a.y + b.y) / 2.0);
        for (height, half_height, depth, color) in [
            (1.0, 0.55, 0.45, WOOD_DARK),
            (PLINTH, 0.45, 0.55, WOOD),
            (top - 0.8, 0.6, 0.7, WOOD),
        ] {
            box_at(
                &mut trim,
                middle + up * height + inward * (depth / 2.0),
                [along, up, inward],
                Vec3::new(length / 2.0, half_height, depth / 2.0),
                color,
            );
        }
    }
    for (x, z) in [(x0, z0), (x1, z0), (x1, z1), (x0, z1)] {
        box_at(
            &mut trim,
            Vec3::new(x, (bottom + top) / 2.0, z),
            [Vec3::X, up, Vec3::Z],
            Vec3::new(1.6, (top - bottom) / 2.0, 1.6),
            WOOD,
        );
    }
    for k in 1..6 {
        let z = lerp(z0, z1, k as f32 / 6.0);
        box_at(
            &mut trim,
            Vec3::new(room.center.x, top - 2.6, z),
            [Vec3::X, up, Vec3::Z],
            Vec3::new(room.half.x, 2.2, 2.4),
            WOOD_DARK,
        );
    }

    // Tall shelves against every wall, full of toy boxes and the odd toy car.
    let mut boxes = MeshBuilder::default();
    for (origin, along, inward, length) in [
        (Vec2::new(x0, z0), Vec3::X, Vec3::Z, x1 - x0),
        (Vec2::new(x1, z1), Vec3::NEG_X, Vec3::NEG_Z, x1 - x0),
        (Vec2::new(x0, z0), Vec3::Z, Vec3::X, z1 - z0),
        (Vec2::new(x1, z0), Vec3::Z, Vec3::NEG_X, z1 - z0),
    ] {
        for unit in 0..SHELF_UNITS {
            let u = (unit as f32 + 0.5) * length / SHELF_UNITS as f32;
            let base = Vec3::new(origin.x + along.x * u, SHELF_BOTTOM, origin.y + along.z * u);
            shelf(
                &mut trim,
                &mut boxes,
                base,
                along,
                inward,
                (unit as u32) * 31 + length as u32,
            );
        }
    }

    // The ceiling lamps: cords and caps, and glowing globes below them.
    let globe = assets
        .meshes
        .add(Sphere::new(4.2).mesh().ico(2).expect("valid sphere"));
    let glow = assets.materials.add(StandardMaterial {
        base_color: LAMP,
        emissive: LinearRgba::rgb(14.0, 11.0, 7.0),
        perceptual_roughness: 0.25,
        ..default()
    });
    const LAMPS_X: usize = 4;
    const LAMPS_Z: usize = 4;
    for i in 0..LAMPS_X {
        for j in 0..LAMPS_Z {
            let x = lerp(x0, x1, (i as f32 + 0.5) / LAMPS_X as f32);
            let z = lerp(z0, z1, (j as f32 + 0.5) / LAMPS_Z as f32);
            let y = top - 34.0;
            box_at(
                &mut trim,
                Vec3::new(x, (y + top) / 2.0, z),
                [Vec3::X, up, Vec3::Z],
                Vec3::new(0.18, (top - y) / 2.0, 0.18),
                IRON,
            );
            box_at(
                &mut trim,
                Vec3::new(x, y + 4.6, z),
                [Vec3::X, up, Vec3::Z],
                Vec3::new(1.6, 0.8, 1.6),
                IRON,
            );
            world.spawn((
                Mesh3d(globe.clone()),
                MeshMaterial3d(glow.clone()),
                Transform::from_xyz(x, y, z),
                NotShadowCaster,
            ));
            if (i + j) % 2 == 0 {
                world.spawn((
                    PointLight {
                        color: LAMP,
                        intensity: 55_000_000.0,
                        range: 340.0,
                        radius: 12.0,
                        shadow_maps_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(x, y + 2.0, z),
                ));
            }
        }
    }
    world.spawn((
        Mesh3d(assets.meshes.add(trim.build())),
        MeshMaterial3d(painted.clone()),
        NotShadowCaster,
    ));
    world.spawn((
        Mesh3d(assets.meshes.add(boxes.build())),
        MeshMaterial3d(painted),
        NotShadowCaster,
    ));

    // The shop signs, high on each wall.
    sign(
        world,
        assets,
        "TOY STORE",
        Vec3::new(room.center.x, 98.0, z0 + 0.5),
        Vec3::Z,
        Vec2::new(360.0, 20.0),
    );
    sign(
        world,
        assets,
        "PLAY TIME",
        Vec3::new(room.center.x, 98.0, z1 - 0.5),
        Vec3::NEG_Z,
        Vec2::new(300.0, 20.0),
    );
    sign(
        world,
        assets,
        "PLUSH",
        Vec3::new(x0 + 0.5, 98.0, room.center.y),
        Vec3::X,
        Vec2::new(220.0, 20.0),
    );
    sign(
        world,
        assets,
        "TRAINS",
        Vec3::new(x1 - 0.5, 98.0, room.center.y),
        Vec3::NEG_X,
        Vec2::new(220.0, 20.0),
    );
}

/// The value `t` of the way from `a` to `b`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// A wall quad from `a` to `b` (x, z) between heights `y0` and `y1`, its
/// visible face towards `facing`, with its texture repeating every `tile` m.
#[allow(clippy::too_many_arguments)]
fn wall(
    mesh: &mut MeshBuilder,
    ax: f32,
    az: f32,
    bx: f32,
    bz: f32,
    y0: f32,
    y1: f32,
    facing: Vec3,
    tile: f32,
    color: Color,
) {
    let length = Vec2::new(bx - ax, bz - az).length();
    mesh.quad(
        [
            (Vec3::new(ax, y0, az), Vec2::new(0.0, y0 / tile)),
            (Vec3::new(bx, y0, bz), Vec2::new(length / tile, y0 / tile)),
            (Vec3::new(bx, y1, bz), Vec2::new(length / tile, y1 / tile)),
            (Vec3::new(ax, y1, az), Vec2::new(0.0, y1 / tile)),
        ],
        facing,
        color,
    );
}

/// A shelf unit against a wall: its frame, boards, toy boxes and a toy car,
/// standing on `base` and running `along`, with its depth `inward`.
fn shelf(
    frame: &mut MeshBuilder,
    boxes: &mut MeshBuilder,
    base: Vec3,
    along: Vec3,
    inward: Vec3,
    seed: u32,
) {
    let up = Vec3::Y;
    let height = SHELF_TOP - SHELF_BOTTOM;
    let half_width = SHELF_WIDTH / 2.0;
    let half_depth = SHELF_DEPTH / 2.0;
    for k in 0..4 {
        let u = (k as f32 / 3.0 - 0.5) * SHELF_WIDTH;
        box_at(
            frame,
            base + along * u + inward * half_depth + up * (height / 2.0),
            [along, up, inward],
            Vec3::new(1.0, height / 2.0, half_depth),
            WOOD,
        );
    }
    for level in 0..4 {
        let y = height * level as f32 / 3.0;
        box_at(
            frame,
            base + inward * half_depth + up * y,
            [along, up, inward],
            Vec3::new(half_width, 0.65, half_depth),
            WOOD,
        );
    }
    for level in 0..3 {
        let shelf_y = height * level as f32 / 3.0 + 0.7;
        for slot in 0..5 {
            let u = ((slot as f32 + 0.5) / 5.0 - 0.5) * SHELF_WIDTH;
            let n = random01(seed + slot as u32 * 7 + level as u32 * 13);
            let color = PLASTIC[(n * PLASTIC.len() as f32) as usize % PLASTIC.len()];
            let box_height = 10.0 + n * 6.0;
            let box_width = SHELF_WIDTH / 5.0 * (0.55 + n * 0.22);
            box_at(
                boxes,
                base + along * u + inward * (half_depth * 1.1) + up * (shelf_y + box_height / 2.0),
                [along, up, inward],
                Vec3::new(box_width / 2.0, box_height / 2.0, half_depth * 0.38),
                color,
            );
            if n > 0.68 {
                box_at(
                    boxes,
                    base + along * (u - box_width * 0.1)
                        + inward * (half_depth * 1.15)
                        + up * (shelf_y + box_height + 3.0),
                    [along, up, inward],
                    Vec3::new(box_width * 0.3, 3.0, half_depth * 0.3),
                    PLASTIC[(seed as usize + slot + 3) % PLASTIC.len()],
                );
            }
        }
    }
    // A little toy car on the middle board.
    let car = base
        + along * (SHELF_WIDTH * 0.32)
        + inward * (half_depth * 1.1)
        + up * (height / 3.0 + 4.0);
    box_at(
        boxes,
        car,
        [along, up, inward],
        Vec3::new(6.0, 1.8, 3.2),
        PLASTIC[(seed as usize + 1) % PLASTIC.len()],
    );
    for (a, b) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        box_at(
            boxes,
            car + along * (a * 3.6) + inward * (b * 2.6) + up * -1.6,
            [along, up, inward],
            Vec3::new(1.2, 1.2, 0.7),
            IRON,
        );
    }
}

/// A big painted sign, with a wooden frame, on the wall at `center` with its
/// face towards `facing` and `size` metres across.
fn sign(
    world: &mut ChildSpawnerCommands,
    assets: &mut Assets3d,
    text: &str,
    center: Vec3,
    facing: Vec3,
    size: Vec2,
) {
    let ink = Color::srgb(0.1, 0.18, 0.35);
    let paper = Color::srgb(1.0, 0.96, 0.86);
    let red = Color::srgb(0.82, 0.12, 0.14);
    let image = sign_image(text, ink, Some((4, red)), paper, Some(red), size.x / size.y);
    let right = Vec3::Y.cross(facing).normalize();
    let up = Vec3::Y;
    let mut panel = MeshBuilder::default();
    panel.quad(
        [
            (
                center - right * (size.x / 2.0) - up * (size.y / 2.0),
                Vec2::new(0.0, 1.0),
            ),
            (
                center + right * (size.x / 2.0) - up * (size.y / 2.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                center + right * (size.x / 2.0) + up * (size.y / 2.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                center - right * (size.x / 2.0) + up * (size.y / 2.0),
                Vec2::new(0.0, 0.0),
            ),
        ],
        facing,
        Color::WHITE,
    );
    world.spawn((
        Mesh3d(assets.meshes.add(panel.build())),
        MeshMaterial3d(assets.materials.add(StandardMaterial {
            base_color_texture: Some(assets.images.add(image)),
            unlit: true,
            fog_enabled: false,
            ..default()
        })),
        NotShadowCaster,
    ));
    let mut frame = MeshBuilder::default();
    let along = right;
    let inward = facing;
    let edge = 1.2;
    for (offset, half) in [
        (
            up * (size.y / 2.0 + edge / 2.0),
            Vec3::new(size.x / 2.0 + edge, edge / 2.0, 0.4),
        ),
        (
            -up * (size.y / 2.0 + edge / 2.0),
            Vec3::new(size.x / 2.0 + edge, edge / 2.0, 0.4),
        ),
        (
            right * (size.x / 2.0 + edge / 2.0),
            Vec3::new(edge / 2.0, size.y / 2.0, 0.4),
        ),
        (
            -right * (size.x / 2.0 + edge / 2.0),
            Vec3::new(edge / 2.0, size.y / 2.0, 0.4),
        ),
    ] {
        box_at(
            &mut frame,
            center + offset - inward * 0.3,
            [along, up, inward],
            half,
            WOOD_DARK,
        );
    }
    world.spawn((
        Mesh3d(assets.meshes.add(frame.build())),
        MeshMaterial3d(assets.materials.add(StandardMaterial {
            perceptual_roughness: 0.8,
            ..default()
        })),
        NotShadowCaster,
    ));
}
