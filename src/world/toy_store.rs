//! The inside of a toy shop (see [`ToyStore`]): the room the road runs
//! through, with its ceiling and its lamps, the shelves of toys along the
//! walls, the chair and the tables the road runs over, the plastic slide it
//! comes down, the toy track itself laid over the floor, and the electric
//! train going round on its railway.
//!
//! The road is not asphalt here: it is the toy track, and it is drawn over
//! the same ground the asphalt would be on (see [`super::road`]), in the
//! colour of the set it is over: the shop's own orange circuit, a grey
//! Scalextric set, a Hot Wheels circuit and the yellow chute of the slide.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Assets3d;
use super::lamps::Lamp;
use super::quads::MeshBuilder;
use super::scenery::{Chair, Scenery, Shelf, Table, Toy, ToyKind, ToyStore};
use crate::geometry::{heading_of, random01};
use crate::track::{SURFACE_Y, Track, WALL_DISTANCE};

/// The height of the road where its height profile is at zero: the level
/// the track runs on where it is on the floor of the shop, and the level the
/// furniture of the shop stands at.
const ROAD_BASE: f32 = 2.0;
/// The height of the shop's floor: a little under the road, so that the
/// track, which dips a little at the foot of the slide, is never under it.
const FLOOR_Y: f32 = 1.7;

/// The colours of the shop.
const FLOOR: Color = Color::srgb(0.78, 0.68, 0.55);
const WALL: Color = Color::srgb(0.93, 0.92, 0.89);
const CEILING: Color = Color::srgb(0.86, 0.86, 0.87);
const BEAM: Color = Color::srgb(0.95, 0.95, 0.94);
const SHELF_WOOD: Color = Color::srgb(0.80, 0.70, 0.56);
const SHELF_BACK: Color = Color::srgb(0.88, 0.88, 0.86);
/// The colours of the toy track: the shop's own circuit, a Scalextric set,
/// a Hot Wheels circuit and the slide.
const TRACK_ORANGE: Color = Color::srgb(0.92, 0.38, 0.10);
const TRACK_GREY: Color = Color::srgb(0.42, 0.43, 0.46);
const TRACK_RAIL: Color = Color::srgb(0.86, 0.87, 0.88);
const TRACK_YELLOW: Color = Color::srgb(0.95, 0.78, 0.16);
const SLIDE_BLUE: Color = Color::srgb(0.13, 0.42, 0.80);
/// The colours of the toys on the shelves and about the floor.
const TOY_COLOURS: [Color; 6] = [
    Color::srgb(0.85, 0.16, 0.14),
    Color::srgb(0.95, 0.72, 0.10),
    Color::srgb(0.14, 0.45, 0.82),
    Color::srgb(0.16, 0.68, 0.32),
    Color::srgb(0.85, 0.42, 0.72),
    Color::srgb(0.95, 0.95, 0.95),
];
const CHAIR_COLOUR: Color = Color::srgb(0.72, 0.20, 0.26);
const TABLE_WOOD: Color = Color::srgb(0.84, 0.76, 0.62);
const TRAIN_RED: Color = Color::srgb(0.78, 0.13, 0.11);
const TRAIN_BLUE: Color = Color::srgb(0.13, 0.32, 0.66);
const RAIL_METAL: Color = Color::srgb(0.62, 0.64, 0.68);

/// How many lamps hang from the ceiling, each way (m), and how far below it
/// they hang.
const LAMP_COLUMNS: usize = 6;
const LAMP_ROWS: usize = 8;
const LAMP_DROP: f32 = 3.0;
const LAMP_LENGTH: f32 = 22.0;
/// How bright a lamp's light is (lm).
const LAMP_LUMENS: f32 = 4200.0;

/// The electric train, running round its railway on the floor.
#[derive(Component)]
pub struct TrainCar {
    /// The railway it runs on: the points of its path, closed.
    path: Vec<Vec2>,
    /// How fast it runs (m/s).
    speed: f32,
    /// How far round the path it is (m).
    along: f32,
    /// How far behind the engine it is (m).
    behind: f32,
}

/// Runs the electric train round its railway.
pub fn run_the_train(time: Res<Time>, mut trains: Query<(&mut TrainCar, &mut Transform)>) {
    for (mut car, mut transform) in &mut trains {
        car.along += car.speed * time.delta_secs();
        let (position, heading) = point_at(&car.path, car.along - car.behind);
        // The wheels stand on the rails, which are on the sleepers.
        transform.translation = Vec3::new(position.x, FLOOR_Y + 0.75, position.y);
        // The nose of the engine is its +Z, and the heading of a point of
        // the path is the way the way the path runs points.
        transform.rotation = Quat::from_rotation_y(heading + std::f32::consts::PI);
    }
}

/// Where a point of a closed path is, `along` metres round it, and which way
/// it faces there.
fn point_at(path: &[Vec2], along: f32) -> (Vec2, f32) {
    if path.len() < 2 {
        return (Vec2::ZERO, 0.0);
    }
    let length = (0..path.len())
        .map(|index| path[index].distance(path[(index + 1) % path.len()]))
        .sum::<f32>();
    let mut rest = along.rem_euclid(length);
    for (index, point) in path.iter().enumerate() {
        let next = path[(index + 1) % path.len()];
        let leg = point.distance(next);
        if rest <= leg {
            let t = if leg > 0.0 { rest / leg } else { 0.0 };
            return (point.lerp(next, t), heading_of(next - point));
        }
        rest -= leg;
    }
    (path[0], heading_of(Vec2::X))
}

/// The lamps of the ceiling of the shop, where they hang: their light
/// follows the cameras (see [`super::lamps`]).
pub fn lamps(store: &ToyStore) -> Vec<Lamp> {
    let (min, max) = (store.centre - store.room, store.centre + store.room);
    let top = FLOOR_Y + store.ceiling;
    let mut lamps = Vec::new();
    for column in 0..LAMP_COLUMNS {
        for row in 0..LAMP_ROWS {
            let x = min.x + (max.x - min.x) * (column as f32 + 0.5) / LAMP_COLUMNS as f32;
            let z = min.y + (max.y - min.y) * (row as f32 + 0.5) / LAMP_ROWS as f32;
            lamps.push(Lamp {
                at: Vec3::new(x, top - LAMP_DROP, z),
                color: Color::srgb(1.0, 0.98, 0.94),
                lumens: LAMP_LUMENS,
            });
        }
    }
    lamps
}

/// Puts the shop up: the room and its ceiling, the shelves of toys, the
/// furniture the road runs over, the slide, the toy track itself and the
/// train.
pub fn spawn_toy_store(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    scenery: &Scenery,
    assets: &mut Assets3d,
) {
    let Some(store) = &scenery.toy_store else {
        return;
    };
    let mut shop = Shop {
        world,
        assets,
        track,
        store,
        floor: FLOOR_Y,
    };

    shop.room();
    shop.ceiling();
    shop.toy_track();
    for chair in store.chairs {
        shop.chair(chair);
    }
    for table in store.tables {
        shop.table(table);
    }
    shop.slide();
    for shelf in store.shelves {
        shop.shelf(shelf);
    }
    for toy in store.toys {
        shop.toy(toy);
    }
    shop.train();
}

/// Everything the shop is built of, put in the world as it is made.
struct Shop<'a, 'w, 'x, 't> {
    world: &'a mut ChildSpawnerCommands<'w>,
    assets: &'a mut Assets3d<'x>,
    track: &'t Track,
    store: &'t ToyStore,
    /// The height of the shop's floor.
    floor: f32,
}

impl Shop<'_, '_, '_, '_> {
    /// Puts a mesh in the world with a material of its own.
    fn put(&mut self, mesh: Mesh, material: StandardMaterial) {
        let material = self.assets.materials.add(material);
        let mesh = self.assets.meshes.add(mesh);
        self.world.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
    }

    /// Puts a mesh in the world that casts no shadow: the room itself, whose
    /// shadows would leave the whole shop in the dark.
    fn put_light(&mut self, mesh: Mesh, material: StandardMaterial) {
        let material = self.assets.materials.add(material);
        let mesh = self.assets.meshes.add(mesh);
        self.world
            .spawn((Mesh3d(mesh), MeshMaterial3d(material), NotShadowCaster));
    }

    /// The room: the floor, the walls and the ceiling over it.
    fn room(&mut self) {
        let (min, max) = (
            self.store.centre - self.store.room,
            self.store.centre + self.store.room,
        );
        let top = self.floor + self.store.ceiling;

        // The floor of the shop.
        let mut mesh = MeshBuilder::default();
        mesh.quad(
            [
                (Vec3::new(min.x, self.floor, min.y), Vec2::new(0.0, 0.0)),
                (Vec3::new(max.x, self.floor, min.y), Vec2::new(1.0, 0.0)),
                (Vec3::new(max.x, self.floor, max.y), Vec2::new(1.0, 1.0)),
                (Vec3::new(min.x, self.floor, max.y), Vec2::new(0.0, 1.0)),
            ],
            Vec3::Y,
            Color::WHITE,
        );
        self.put_light(
            mesh.build(),
            StandardMaterial {
                base_color: FLOOR,
                perceptual_roughness: 0.65,
                reflectance: 0.25,
                ..default()
            },
        );

        // The walls, from the floor to the ceiling.
        let mut mesh = MeshBuilder::default();
        let mut wall = |a: Vec2, b: Vec2, out: Vec3| {
            mesh.quad(
                [
                    (Vec3::new(a.x, self.floor, a.y), Vec2::new(0.0, 0.0)),
                    (Vec3::new(b.x, self.floor, b.y), Vec2::new(1.0, 0.0)),
                    (Vec3::new(b.x, top, b.y), Vec2::new(1.0, 1.0)),
                    (Vec3::new(a.x, top, a.y), Vec2::new(0.0, 1.0)),
                ],
                out,
                Color::WHITE,
            );
        };
        wall(
            Vec2::new(min.x, min.y),
            Vec2::new(max.x, min.y),
            Vec3::NEG_Z,
        );
        wall(Vec2::new(max.x, min.y), Vec2::new(max.x, max.y), Vec3::X);
        wall(Vec2::new(max.x, max.y), Vec2::new(min.x, max.y), Vec3::Z);
        wall(
            Vec2::new(min.x, max.y),
            Vec2::new(min.x, min.y),
            Vec3::NEG_X,
        );
        self.put_light(
            mesh.build(),
            StandardMaterial {
                base_color: WALL,
                perceptual_roughness: 0.85,
                ..default()
            },
        );

        // The ceiling over the whole room.
        let mut mesh = MeshBuilder::default();
        mesh.quad(
            [
                (Vec3::new(min.x, top, min.y), Vec2::new(0.0, 0.0)),
                (Vec3::new(max.x, top, min.y), Vec2::new(1.0, 0.0)),
                (Vec3::new(max.x, top, max.y), Vec2::new(1.0, 1.0)),
                (Vec3::new(min.x, top, max.y), Vec2::new(0.0, 1.0)),
            ],
            Vec3::NEG_Y,
            Color::WHITE,
        );
        self.put_light(
            mesh.build(),
            StandardMaterial {
                base_color: CEILING,
                perceptual_roughness: 0.9,
                ..default()
            },
        );
    }

    /// The ceiling: its beams and the lamps hanging from it.
    fn ceiling(&mut self) {
        let (min, max) = (
            self.store.centre - self.store.room,
            self.store.centre + self.store.room,
        );
        let top = self.floor + self.store.ceiling;
        // Beams across the ceiling, from one wall to the other.
        let mut mesh = MeshBuilder::default();
        let beams = 9;
        for k in 0..beams {
            let z = min.y + (max.y - min.y) * (k as f32 + 0.5) / beams as f32;
            box_at(
                &mut mesh,
                Vec3::new(self.store.centre.x, top - 1.2, z),
                Vec3::new(max.x - min.x, 2.4, 2.0),
                0,
                BEAM,
            );
        }
        self.put_light(
            mesh.build(),
            StandardMaterial {
                base_color: BEAM,
                perceptual_roughness: 0.8,
                ..default()
            },
        );

        // The lamps: rows of fluorescent tubes over the shop.
        let mut mesh = MeshBuilder::default();
        let mut lit = MeshBuilder::default();
        for column in 0..LAMP_COLUMNS {
            for row in 0..LAMP_ROWS {
                let x = min.x + (max.x - min.x) * (column as f32 + 0.5) / LAMP_COLUMNS as f32;
                let z = min.y + (max.y - min.y) * (row as f32 + 0.5) / LAMP_ROWS as f32;
                let y = top - LAMP_DROP;
                // Its rod up to the ceiling, and the tube itself.
                box_at(
                    &mut mesh,
                    Vec3::new(x, y + LAMP_DROP / 2.0, z),
                    Vec3::new(0.5, LAMP_DROP, 0.5),
                    1,
                    BEAM,
                );
                box_at(
                    &mut mesh,
                    Vec3::new(x, y + 0.4, z),
                    Vec3::new(LAMP_LENGTH, 0.8, 1.6),
                    0,
                    BEAM,
                );
                box_at(
                    &mut lit,
                    Vec3::new(x, y, z),
                    Vec3::new(LAMP_LENGTH - 1.0, 0.35, 1.2),
                    0,
                    Color::WHITE,
                );
            }
        }
        self.put_light(
            mesh.build(),
            StandardMaterial {
                base_color: BEAM,
                perceptual_roughness: 0.7,
                ..default()
            },
        );
        self.put_light(
            lit.build(),
            StandardMaterial {
                base_color: Color::srgb(1.0, 0.99, 0.96),
                emissive: LinearRgba::rgb(5.0, 4.9, 4.6),
                ..default()
            },
        );
    }

    /// The toy track the road runs on: the bed of it, in the colour of the
    /// set it is over.
    fn toy_track(&mut self) {
        let colours = [TRACK_ORANGE, TRACK_GREY, TRACK_ORANGE, TRACK_YELLOW];
        let mut meshes: Vec<MeshBuilder> = (0..4).map(|_| MeshBuilder::default()).collect();
        let mut rails = MeshBuilder::default();
        let step = self.track.step_length();
        let segments = self.track.segment_count();
        for index in 0..segments {
            let along = (index as f32 + 0.5) * step;
            let style = self.bed_style(along);
            let at =
                |i: usize, lateral: f32| self.track.surface_point(i, lateral) + Vec3::Y * SURFACE_Y;
            let width = WALL_DISTANCE;
            meshes[style].quad(
                [
                    (at(index, -width), Vec2::new(0.0, along / 6.0)),
                    (at(index + 1, -width), Vec2::new(1.0, (along + step) / 6.0)),
                    (at(index + 1, width), Vec2::new(1.0, (along + step) / 6.0)),
                    (at(index, width), Vec2::new(0.0, along / 6.0)),
                ],
                Vec3::Y,
                Color::WHITE,
            );
            // A Scalextric set: two little rails down the middle of it.
            if style == 1 {
                for side in [-1.0, 1.0] {
                    let lateral = side * 2.4;
                    rails.quad(
                        [
                            (at(index, lateral - 0.25) + Vec3::Y * 0.08, Vec2::ZERO),
                            (at(index + 1, lateral - 0.25) + Vec3::Y * 0.08, Vec2::X),
                            (at(index + 1, lateral + 0.25) + Vec3::Y * 0.08, Vec2::ONE),
                            (at(index, lateral + 0.25) + Vec3::Y * 0.08, Vec2::Y),
                        ],
                        Vec3::Y,
                        Color::WHITE,
                    );
                }
            }
        }
        for (style, colour) in colours.iter().enumerate() {
            let mesh = std::mem::take(&mut meshes[style]);
            if mesh.is_empty() {
                continue;
            }
            self.put(
                mesh.build(),
                StandardMaterial {
                    base_color: *colour,
                    perceptual_roughness: 0.42,
                    reflectance: 0.4,
                    ..default()
                },
            );
        }
        self.put(
            rails.build(),
            StandardMaterial {
                base_color: TRACK_RAIL,
                perceptual_roughness: 0.3,
                metallic: 0.6,
                ..default()
            },
        );
    }

    /// Which set of the shop the toy track is over at `along` metres from
    /// the finish line.
    fn bed_style(&self, along: f32) -> usize {
        let lap = self.track.lap_length();
        if self.store.scalextric.contains(along, lap) {
            1
        } else if self.store.hot_wheels.contains(along, lap) {
            2
        } else if self.store.slide.contains(along, lap) {
            3
        } else {
            0
        }
    }

    /// The chair of the shop that the road runs across.
    fn chair(&mut self, chair: &Chair) {
        let seat = ROAD_BASE + chair.height;
        let at = furniture_at(chair.position, chair.heading);
        let mut mesh = MeshBuilder::default();
        // The seat, its back and its two arms.
        box_at(
            &mut mesh,
            at(0.0, 0.0) + Vec3::Y * (seat - 1.6),
            Vec3::new(chair.width, 3.2, chair.depth),
            1,
            CHAIR_COLOUR,
        );
        // Its back, behind the seat: the chair faces the other way.
        box_at(
            &mut mesh,
            at(0.0, -(chair.depth / 2.0 - 2.0)) + Vec3::Y * (seat + 9.0),
            Vec3::new(chair.width, 22.0, 4.0),
            1,
            CHAIR_COLOUR,
        );
        for side in [-1.0, 1.0] {
            box_at(
                &mut mesh,
                at(side * (chair.width / 2.0 - 2.0), 0.0) + Vec3::Y * (seat + 3.0),
                Vec3::new(4.0, 12.0, chair.depth),
                1,
                CHAIR_COLOUR,
            );
            // Its legs.
            for end in [-1.0, 1.0] {
                box_at(
                    &mut mesh,
                    at(
                        side * (chair.width / 2.0 - 3.0),
                        end * (chair.depth / 2.0 - 3.0),
                    ) + Vec3::Y * (self.floor + (seat - self.floor) / 2.0),
                    Vec3::new(3.0, seat - self.floor, 3.0),
                    1,
                    SHELF_WOOD,
                );
            }
        }
        self.put(
            mesh.build(),
            StandardMaterial {
                base_color: CHAIR_COLOUR,
                perceptual_roughness: 0.85,
                ..default()
            },
        );
    }

    /// The table of the shop that the road runs across, with a few toys on
    /// it.
    fn table(&mut self, table: &Table) {
        let top = ROAD_BASE + table.height;
        let at = furniture_at(table.position, table.heading);
        let mut mesh = MeshBuilder::default();
        // The top of it, its pedestal and its foot.
        box_at(
            &mut mesh,
            at(0.0, 0.0) + Vec3::Y * (top - 1.5),
            Vec3::new(table.width, 3.0, table.depth),
            1,
            TABLE_WOOD,
        );
        let pedestal = top - 3.0 - self.floor;
        box_at(
            &mut mesh,
            at(0.0, 0.0) + Vec3::Y * (self.floor + pedestal / 2.0),
            Vec3::new(table.width / 4.0, pedestal, table.depth / 4.0),
            1,
            TABLE_WOOD,
        );
        box_at(
            &mut mesh,
            at(0.0, 0.0) + Vec3::Y * (self.floor + 1.5),
            Vec3::new(table.width / 2.0, 3.0, table.depth / 2.0),
            1,
            TABLE_WOOD,
        );
        self.put(
            mesh.build(),
            StandardMaterial {
                base_color: TABLE_WOOD,
                perceptual_roughness: 0.6,
                ..default()
            },
        );
        // Some toys on the table, away from the track.
        let mut toys = MeshBuilder::default();
        for k in 0..4 {
            let side = if k % 2 == 0 { 1.0 } else { -1.0 };
            let x = side * table.width * (0.28 + 0.06 * random01(k as u32));
            let z = (random01(k as u32 + 7) - 0.5) * table.depth * 0.6;
            let colour = TOY_COLOURS[k % TOY_COLOURS.len()];
            box_at(
                &mut toys,
                at(x, z) + Vec3::Y * (top + 2.5),
                Vec3::new(5.0, 5.0, 5.0),
                1,
                colour,
            );
        }
        self.put(
            toys.build(),
            StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.5,
                ..default()
            },
        );
    }

    /// The slide: the chute the road comes down, with its sides.
    fn slide(&mut self) {
        let lap = self.track.lap_length();
        let indices: Vec<usize> = (0..self.track.sample_count())
            .filter(|&index| {
                self.store
                    .slide
                    .contains(self.track.lap_along_at(index), lap)
            })
            .collect();
        if indices.len() < 2 {
            return;
        }
        let mut mesh = MeshBuilder::default();
        for pair in indices.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let at =
                |i: usize, lateral: f32| self.track.surface_point(i, lateral) + Vec3::Y * SURFACE_Y;
            for side in [-1.0, 1.0] {
                let inner = side * 8.4;
                let outer = side * 9.6;
                let height = 2.6;
                // The side of the chute, inside and out, and over the top.
                mesh.quad(
                    [
                        (at(a, inner), Vec2::new(0.0, 0.0)),
                        (at(b, inner), Vec2::new(1.0, 0.0)),
                        (at(b, inner) + Vec3::Y * height, Vec2::new(1.0, 1.0)),
                        (at(a, inner) + Vec3::Y * height, Vec2::new(0.0, 1.0)),
                    ],
                    -Vec3::X * side,
                    Color::WHITE,
                );
                mesh.quad(
                    [
                        (at(a, outer) + Vec3::Y * height, Vec2::new(0.0, 0.0)),
                        (at(b, outer) + Vec3::Y * height, Vec2::new(1.0, 0.0)),
                        (at(b, outer), Vec2::new(1.0, 1.0)),
                        (at(a, outer), Vec2::new(0.0, 1.0)),
                    ],
                    Vec3::X * side,
                    Color::WHITE,
                );
                mesh.quad(
                    [
                        (at(a, inner) + Vec3::Y * height, Vec2::new(0.0, 0.0)),
                        (at(b, inner) + Vec3::Y * height, Vec2::new(1.0, 0.0)),
                        (at(b, outer) + Vec3::Y * height, Vec2::new(1.0, 1.0)),
                        (at(a, outer) + Vec3::Y * height, Vec2::new(0.0, 1.0)),
                    ],
                    Vec3::Y,
                    Color::WHITE,
                );
            }
        }
        self.put(
            mesh.build(),
            StandardMaterial {
                base_color: SLIDE_BLUE,
                perceptual_roughness: 0.4,
                reflectance: 0.4,
                ..default()
            },
        );
    }

    /// A shelf unit of toys along a wall.
    fn shelf(&mut self, shelf: &Shelf) {
        // The unit faces `heading`, into the room: its shelves run along its
        // left, and it is as deep as it is wide.
        let forward = crate::geometry::forward(shelf.heading);
        let left = Vec2::new(-forward.y, forward.x);
        let at = |depth: f32, length: f32| {
            let point = shelf.position + forward * depth + left * length;
            Vec3::new(point.x, 0.0, point.y)
        };
        let depth = 9.0;
        let height = 24.0;
        let mut wood = MeshBuilder::default();
        let mut back = MeshBuilder::default();
        // Its back and its shelves.
        box_at(
            &mut back,
            at(-depth / 2.0, 0.0) + Vec3::Y * (self.floor + height / 2.0),
            Vec3::new(1.0, height, shelf.length),
            0,
            SHELF_BACK,
        );
        let boards = 5;
        for k in 0..=boards {
            let y = self.floor + height * k as f32 / boards as f32;
            box_at(
                &mut wood,
                at(0.0, 0.0) + Vec3::Y * y,
                Vec3::new(depth, 1.2, shelf.length),
                1,
                SHELF_WOOD,
            );
        }
        // Its ends.
        for side in [-1.0, 1.0] {
            box_at(
                &mut wood,
                at(0.0, side * shelf.length / 2.0) + Vec3::Y * (self.floor + height / 2.0),
                Vec3::new(depth, height, 1.2),
                1,
                SHELF_WOOD,
            );
        }
        self.put(
            wood.build(),
            StandardMaterial {
                base_color: SHELF_WOOD,
                perceptual_roughness: 0.6,
                ..default()
            },
        );
        self.put(
            back.build(),
            StandardMaterial {
                base_color: SHELF_BACK,
                perceptual_roughness: 0.85,
                ..default()
            },
        );
        // The toys on the shelves: boxes of every colour.
        let mut toys = MeshBuilder::default();
        let count = (shelf.length / 14.0).floor() as u32;
        for k in 0..count {
            let board = (k % boards) as f32;
            let y = self.floor + height * board / boards as f32 + 3.0;
            let z = (random01(k * 3 + 1) - 0.5) * (shelf.length - 14.0);
            let colour = TOY_COLOURS[(k as usize) % TOY_COLOURS.len()];
            let width = 5.0 + 4.0 * random01(k * 5);
            box_at(
                &mut toys,
                at(2.0, z) + Vec3::Y * y,
                Vec3::new(width, 6.0 + 3.0 * random01(k * 7), width * 0.7),
                1,
                colour,
            );
        }
        self.put(
            toys.build(),
            StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.55,
                ..default()
            },
        );
    }

    /// A toy lying about on the floor of the shop.
    fn toy(&mut self, toy: &Toy) {
        let mut mesh = MeshBuilder::default();
        let size = toy.size;
        match toy.kind {
            ToyKind::Ball => {
                sphere(
                    &mut mesh,
                    Vec3::new(toy.position.x, self.floor + size, toy.position.y),
                    size,
                );
            }
            ToyKind::Blocks => {
                for (k, colour) in TOY_COLOURS.iter().take(3).enumerate() {
                    box_at(
                        &mut mesh,
                        Vec3::new(
                            toy.position.x,
                            self.floor + size * (k as f32 + 0.5),
                            toy.position.y,
                        ),
                        Vec3::new(size * 2.0, size, size * 2.0),
                        1,
                        *colour,
                    );
                }
            }
            ToyKind::Doll => {
                box_at(
                    &mut mesh,
                    Vec3::new(toy.position.x, self.floor + size, toy.position.y),
                    Vec3::new(size * 1.6, size * 2.0, size * 1.2),
                    1,
                    TOY_COLOURS[4],
                );
                sphere(
                    &mut mesh,
                    Vec3::new(toy.position.x, self.floor + size * 2.6, toy.position.y),
                    size * 0.8,
                );
            }
            ToyKind::Top => {
                sphere(
                    &mut mesh,
                    Vec3::new(toy.position.x, self.floor + size * 0.6, toy.position.y),
                    size * 0.6,
                );
            }
        }
        self.put(
            mesh.build(),
            StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.5,
                ..default()
            },
        );
    }

    /// The electric train and the railway it runs on.
    fn train(&mut self) {
        let Some(train) = &self.store.train else {
            return;
        };
        if train.path.len() < 3 {
            return;
        }
        // The railway: two rails and the sleepers under them.
        let mut mesh = MeshBuilder::default();
        for index in 0..train.path.len() {
            let a = train.path[index];
            let b = train.path[(index + 1) % train.path.len()];
            let heading = heading_of(b - a);
            let forward = crate::geometry::forward(heading);
            let side = Vec3::new(-forward.y, 0.0, forward.x);
            let middle = Vec3::new(a.x, self.floor, a.y);
            let end = Vec3::new(b.x, self.floor, b.y);
            for offset in [-2.0, 2.0] {
                let from = middle + side * offset;
                let to = end + side * offset;
                mesh.beam(
                    from + Vec3::Y * 0.5,
                    to + Vec3::Y * 0.5,
                    0.7,
                    0.5,
                    RAIL_METAL,
                );
            }
            // A sleeper every few metres.
            let length = a.distance(b);
            let sleepers = (length / 6.0).floor().max(1.0) as u32;
            for k in 0..sleepers {
                let t = (k as f32 + 0.5) / sleepers as f32;
                let at = middle.lerp(end, t);
                mesh.beam(
                    at - side * 3.5 + Vec3::Y * 0.2,
                    at + side * 3.5 + Vec3::Y * 0.2,
                    2.2,
                    0.4,
                    SHELF_WOOD,
                );
            }
        }
        self.put(
            mesh.build(),
            StandardMaterial {
                base_color: RAIL_METAL,
                perceptual_roughness: 0.35,
                metallic: 0.7,
                ..default()
            },
        );

        // The engine and its carriages.
        for (index, colour) in [TRAIN_RED, TRAIN_BLUE, TRAIN_BLUE].into_iter().enumerate() {
            let mut mesh = MeshBuilder::default();
            box_at(
                &mut mesh,
                Vec3::new(0.0, 2.2, 0.0),
                Vec3::new(4.4, 4.4, 13.0),
                2,
                colour,
            );
            if index == 0 {
                // The cab of the engine.
                box_at(
                    &mut mesh,
                    Vec3::new(0.0, 6.4, 2.0),
                    Vec3::new(4.0, 4.0, 5.0),
                    2,
                    colour,
                );
            }
            for side in [-1.0, 1.0] {
                for end in [-1.0, 1.0] {
                    box_at(
                        &mut mesh,
                        Vec3::new(side * 2.2, 0.9, end * 4.4),
                        Vec3::new(1.2, 1.8, 1.8),
                        1,
                        RAIL_METAL,
                    );
                }
            }
            let mesh = self.assets.meshes.add(mesh.build());
            let material = self.assets.materials.add(StandardMaterial {
                base_color: colour,
                perceptual_roughness: 0.4,
                ..default()
            });
            self.world.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                TrainCar {
                    path: train.path.to_vec(),
                    speed: train.speed,
                    // The engine and its carriages go round together, each
                    // a carriage's length behind the one before it.
                    along: 0.0,
                    behind: index as f32 * 16.0,
                },
            ));
        }
    }
}

/// Where a point of a piece of furniture is, `across` metres to its left and
/// `along` metres in front of its middle.
fn furniture_at(position: Vec2, heading: f32) -> impl Fn(f32, f32) -> Vec3 {
    let forward = crate::geometry::forward(heading);
    let left = Vec2::new(-forward.y, forward.x);
    move |across: f32, along: f32| {
        let point = position + left * across + forward * along;
        Vec3::new(point.x, 0.0, point.y)
    }
}

/// A box of `size` at `centre`, its longest side along `axis` (0 = x, 1 =
/// y, 2 = z).
fn box_at(mesh: &mut MeshBuilder, centre: Vec3, size: Vec3, axis: usize, colour: Color) {
    let half = size / 2.0;
    let (a, b, width, height) = match axis {
        0 => (
            centre - Vec3::X * half.x,
            centre + Vec3::X * half.x,
            size.z,
            size.y,
        ),
        1 => (
            centre - Vec3::Y * half.y,
            centre + Vec3::Y * half.y,
            size.x,
            size.z,
        ),
        _ => (
            centre - Vec3::Z * half.z,
            centre + Vec3::Z * half.z,
            size.x,
            size.y,
        ),
    };
    mesh.beam(a, b, width, height, colour);
}

/// A ball of `radius` at `centre`.
fn sphere(mesh: &mut MeshBuilder, centre: Vec3, radius: f32) {
    const RINGS: usize = 6;
    const SIDES: usize = 10;
    for ring in 0..RINGS {
        let from = (ring as f32 / RINGS as f32) * std::f32::consts::PI;
        let to = ((ring + 1) as f32 / RINGS as f32) * std::f32::consts::PI;
        for side in 0..SIDES {
            let a = side as f32 / SIDES as f32 * std::f32::consts::TAU;
            let b = (side + 1) as f32 / SIDES as f32 * std::f32::consts::TAU;
            let point = |polar: f32, azimuth: f32| {
                centre
                    + Vec3::new(
                        polar.sin() * azimuth.cos(),
                        polar.cos(),
                        polar.sin() * azimuth.sin(),
                    ) * radius
            };
            mesh.quad(
                [
                    (point(from, a), Vec2::ZERO),
                    (point(from, b), Vec2::X),
                    (point(to, b), Vec2::ONE),
                    (point(to, a), Vec2::Y),
                ],
                point(from, a) - centre,
                Color::WHITE,
            );
        }
    }
}
