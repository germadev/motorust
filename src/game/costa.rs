//! The scenery of Costa Bits ([`Costa`]): props made with Meshy (prepared
//! by `tools/import_costa.py`, see `docs/costa-bits-props.md`) standing
//! about the bay: whitewashed houses in the town and on the rims of the
//! gorge, a beach bar, a fishing boat with its fire of skewered fish and a
//! lifeguard's tower on the beach, a sports car parked by the grid, the
//! marshal by the line with his chequered flag, a watchtower on the cape,
//! a viewpoint over the sea, boulders along the mountain road, prickly
//! pears and agaves on its dry slopes, ibex on the rocks over it, a
//! hermitage on the headland, a camper van at the viewpoint, a ruined
//! castle on the shelf of rock below the mesa, billboards of the arcades'
//! sunset, their pictures moving and their bulbs lit in a chase round their
//! frames, and a kilometre
//! post every 500 m, a plane towing a banner round over the bay; the old
//! stone bridge over the gorge, the road running under its arch; and a
//! stretch of the mountain road under an overhang of rock; and a
//! yellow sign before every bend of the mountain road, its arrow bending
//! the way the road does.
//!
//! Every prop's colours are in one texture, so they all share one material,
//! and every kind of prop is one mesh.
//!
//! A theme of the game's (see [`motorust_engine::world::themes`]): a
//! circuit's file writes it in its `themes` as `"costa": (...)`. Its props
//! are a pack of the engine's (`models/costa/`).

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use motorust_engine::bike::model::imported::texture;
use motorust_engine::geometry::{random01, smoothstep};
use motorust_engine::track::Track;
use motorust_engine::world::Assets3d;
use motorust_engine::world::mirror::MirrorPoint;
use motorust_engine::world::prop_pack::{PropPack, facing, in_view, spot};
use motorust_engine::world::props::Footprint;
use motorust_engine::world::scenery::{Prop, Scenery, Side, Span, Spot};
use motorust_engine::world::terrain::Terrain;
use motorust_engine::world::themes::Theme;

/// The pack of its props, in `assets/models/`.
const PACK: &str = "costa";
/// The billboard's bulbs in the pack: this and the number of their group.
const BILLBOARD_BULBS: &str = "arcade_billboard_bulbs_";
/// The billboard's panel in the pack, and its picture's frames in the
/// content.
const BILLBOARD_PANEL: &str = "arcade_billboard_panel";
const BILLBOARD_PICTURE: &str = "models/costa/billboard.jpg";

/// The scenery of Costa Bits (see [`crate::game::costa`]): props made
/// with Meshy standing about, the bridge over the gorge, and signs before
/// the bends.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Costa {
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub props: &'static [Prop<CostaModel>],
    /// Stretches of the lap with a bend sign before each of their bends.
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub bend_signs: &'static [BendSigns],
    /// Kilometre posts along the lap, if any (see [`KmPosts`]).
    #[serde(default)]
    pub km_posts: Option<KmPosts>,
    /// Stretches of the lap where the rock beside the road reaches out over
    /// it.
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub overhangs: &'static [Overhang],
    /// Planes towing their banners round in circles over the water.
    #[serde(deserialize_with = "motorust_engine::circuits::data::leak", default)]
    pub planes: &'static [Flight],
}

/// Rock reaching out over the road along `span` from the cut rock on
/// `side` of it (see [`overhang_mesh`]), as a mountain road cut into a
/// cliff runs under it: `reach` metres out from the foot of the rock, its
/// underside `clearance` metres over the road there.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overhang {
    pub span: Span,
    pub side: Side,
    pub reach: f32,
    pub clearance: f32,
}

/// Where the cut rock rises beside the road (m from its middle), as the
/// engine's terrain has it, and how far into it an overhang's root goes.
const ROCK_FOOT: f32 = 13.0;
const OVERHANG_ROOT: f32 = 4.0;
/// An overhang's blocks: how wide across the road and how long along it
/// (m), as the blocks of the ground by the road.
const OVERHANG_BLOCK: (f32, f32) = (1.0, 2.0);
/// How thick it is at its root and at its lip (m), how much its underside
/// rises a metre out, and over how far along the road it grows out at each
/// end of its stretch (m).
const OVERHANG_THICK: (f32, f32) = (7.0, 1.5);
const OVERHANG_RISE: f32 = 0.12;
const OVERHANG_EASE: f32 = 40.0;
/// The rock's colour (sRGB), each block a little lighter or darker.
const OVERHANG_ROCK: [f32; 3] = [0.5, 0.41, 0.38];

impl Overhang {
    /// How far out from the rock's foot it reaches `into` metres into its
    /// stretch, `length` long: growing out and drawing back at its ends.
    fn reach_at(&self, into: f32, length: f32) -> f32 {
        self.reach
            * smoothstep(0.0, OVERHANG_EASE, into)
            * smoothstep(0.0, OVERHANG_EASE, length - into)
    }

    /// Its underside and its top (m over the road) `out` metres from the
    /// rock's foot, where it reaches `reach` metres out.
    fn section(&self, out: f32, reach: f32) -> (f32, f32) {
        let t = (out / reach.max(1.0)).clamp(0.0, 1.0);
        let under = self.clearance + OVERHANG_RISE * out.max(0.0);
        let thick = OVERHANG_THICK.0 + (OVERHANG_THICK.1 - OVERHANG_THICK.0) * t;
        (under, under + thick)
    }
}

/// An overhang's rock (see [`Overhang`]), in blocks as the ground beside
/// the road is: columns across the road from inside the rock out to its
/// lip, each a block from its underside to its top, a metre higher or
/// lower here and there, each of one colour.
fn overhang_mesh(track: &Track, overhang: &Overhang) -> Mesh {
    let lap = track.lap_length();
    let length = overhang.span.length(lap);
    let sign = overhang.side.sign();
    let (wide, long) = OVERHANG_BLOCK;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut quad = |corners: [Vec3; 4], facing: Vec3, shade: f32| {
        let normal = (corners[1] - corners[0]).cross(corners[2] - corners[0]);
        let normal = if normal.dot(facing) < 0.0 {
            -normal
        } else {
            normal
        }
        .normalize_or_zero();
        let order = if (corners[1] - corners[0])
            .cross(corners[2] - corners[0])
            .dot(facing)
            < 0.0
        {
            [0, 2, 1, 0, 3, 2]
        } else {
            [0, 1, 2, 0, 2, 3]
        };
        let color = Color::srgb(
            OVERHANG_ROCK[0] * shade,
            OVERHANG_ROCK[1] * shade,
            OVERHANG_ROCK[2] * shade,
        )
        .to_linear();
        for k in order {
            positions.push(corners[k].to_array());
            normals.push(normal.to_array());
            colors.push([color.red, color.green, color.blue, 1.0]);
        }
    };
    let slices = (length / long).floor() as usize;
    for slice in 0..slices {
        let into = slice as f32 * long;
        let reach = overhang.reach_at(into + long / 2.0, length);
        if reach < wide {
            continue;
        }
        let (i0, i1) = (
            track.index_at(overhang.span.from + into),
            track.index_at(overhang.span.from + into + long),
        );
        let (road0, road1) = (
            track.surface_point(i0, 0.0).y,
            track.surface_point(i1, 0.0).y,
        );
        let ahead = {
            let (a, b) = (track.offset_point(i0, 0.0), track.offset_point(i1, 0.0));
            Vec3::new(b.x - a.x, 0.0, b.y - a.y).normalize_or_zero()
        };
        let columns = ((reach + OVERHANG_ROOT) / wide).ceil() as usize;
        for column in 0..columns {
            let out = column as f32 * wide - OVERHANG_ROOT;
            let seed = (slice as u32 / 2) * 131 + column as u32 * 7 + 17;
            let (under, top) = overhang.section(out + wide / 2.0, reach);
            // A metre up or down here and there, the lip left as it is.
            let step = (random01(seed) * 3.0).floor() - 1.0;
            let top = top + step.max(0.0);
            let under = (under + 0.5 * step.min(0.0).abs()).round_ties_even();
            let shade = 0.82 + 0.3 * random01(seed + 5);
            let (l0, l1) = (sign * (ROCK_FOOT - out), sign * (ROCK_FOOT - out - wide));
            let at = |index: usize, lateral: f32, road: f32, height: f32| {
                let p = track.offset_point(index, lateral);
                Vec3::new(p.x, road + height, p.y)
            };
            let (a, b, c, d) = (
                at(i0, l0, road0, under),
                at(i0, l1, road0, under),
                at(i1, l1, road1, under),
                at(i1, l0, road1, under),
            );
            let lift = Vec3::Y * (top - under);
            let towards_road = (b - a).normalize_or_zero();
            quad([a, b, c, d], Vec3::NEG_Y, shade * 0.62);
            quad(
                [a + lift, b + lift, c + lift, d + lift],
                Vec3::Y,
                shade * 1.1,
            );
            quad([b, c, c + lift, b + lift], towards_road, shade * 0.9);
            quad([a, b, b + lift, a + lift], -ahead, shade * 0.78);
            quad([d, c, c + lift, d + lift], ahead, shade * 0.72);
        }
    }
    let count = positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// A plane's flight (see [`Costa::planes`]): round `center` (x, z), `radius`
/// metres from it, `height` metres over the sea, at `speed` metres a
/// second, anticlockwise seen from above, starting `phase` radians round
/// from the east.
#[derive(Clone, Copy, Debug, Component, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flight {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
    pub speed: f32,
    #[serde(default)]
    pub phase: f32,
}

/// How far a plane banks into its turn (rad).
const BANK: f32 = 0.22;

impl Flight {
    /// Where the plane is `time` seconds into its flight (x, height, z),
    /// and which way it heads (x, z).
    fn at(&self, time: f32) -> (Vec3, Vec2) {
        let angle = self.phase + time * self.speed / self.radius;
        let (sin, cos) = angle.sin_cos();
        let position = self.center + Vec2::new(cos, -sin) * self.radius;
        let heading = Vec2::new(-sin, -cos);
        (Vec3::new(position.x, self.height, position.y), heading)
    }

    /// The plane's transform `time` seconds into its flight: its nose (-X
    /// as made) the way it heads, banked into the turn.
    fn transform(&self, time: f32) -> Transform {
        let (position, heading) = self.at(time);
        // Made with its nose to -X: a quarter turn brings it to +Z, which
        // `facing` turns the way it heads; then it rolls about its length
        // into the turn, its left wing (+Z as made) down.
        let rotation = facing(heading)
            * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_x(BANK);
        Transform::from_translation(position).with_rotation(rotation)
    }
}

/// The theme's systems: the planes flying, the billboards' bulbs chasing
/// round their frames and their pictures moving.
pub struct CostaPlugin;

impl Plugin for CostaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fly, chase_bulbs, run_pictures));
    }
}

/// A billboard's picture (see [`run_pictures`]): its panel, in a material
/// of its own with the picture's frames.
#[derive(Component)]
struct Picture;

/// The billboard's picture's frames in its image (`billboard.jpg`, as
/// `tools/import_costa.py` draws it): how many across and down, in the
/// order of the loop from the top left; how many a second are shown; and
/// how far in from a frame's edges its panel shows (of the frame), so that
/// no other frame shows along them from afar.
const PICTURE_GRID: UVec2 = UVec2::new(2, 2);
const PICTURE_RATE: f32 = 6.0;
const PICTURE_INSET: f32 = 0.004;
/// How bright the picture shines of itself, lit from behind as a
/// billboard's is.
const PICTURE_GLOW: f32 = 0.55;

/// The frame of the picture shown `time` seconds in.
fn picture_frame(time: f32) -> u32 {
    (time * PICTURE_RATE) as u32 % (PICTURE_GRID.x * PICTURE_GRID.y)
}

/// Where frame `frame` is in the picture's image: what takes a panel's
/// texture coordinates (0..1 over it) to that frame's.
fn frame_uv(frame: u32) -> bevy::math::Affine2 {
    let cell = Vec2::ONE / PICTURE_GRID.as_vec2();
    let corner = UVec2::new(frame % PICTURE_GRID.x, frame / PICTURE_GRID.x).as_vec2() * cell;
    bevy::math::Affine2::from_scale_angle_translation(
        cell * (1.0 - 2.0 * PICTURE_INSET),
        0.0,
        corner + cell * PICTURE_INSET,
    )
}

/// Shows the billboards' pictures frame by frame (their material changed
/// only as the frame does: a few times a second, one material for them
/// all).
fn run_pictures(
    time: Res<Time>,
    pictures: Query<&MeshMaterial3d<StandardMaterial>, With<Picture>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<Option<u32>>,
) {
    let frame = picture_frame(time.elapsed_secs());
    if *last == Some(frame) || pictures.is_empty() {
        return;
    }
    *last = Some(frame);
    for material in &pictures {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.uv_transform = frame_uv(frame);
        }
    }
}

/// A group of a billboard's bulbs (see [`chase_bulbs`]): one of
/// [`BULB_GROUPS`], round the frame by turns.
#[derive(Component)]
struct Bulbs {
    group: usize,
}

/// How many groups a billboard's bulbs are in (as `tools/import_costa.py`
/// makes them, `arcade_billboard_bulbs_N`), how many times a second the
/// light steps on from one group to the next, and the light of a bulb lit
/// and unlit.
const BULB_GROUPS: usize = 3;
const BULB_RATE: f32 = 5.0;
const BULB_LIT: LinearRgba = LinearRgba::rgb(9.0, 6.5, 1.6);
const BULB_UNLIT: LinearRgba = LinearRgba::rgb(0.12, 0.08, 0.02);

/// Which group of bulbs is lit `time` seconds in: one after another, round
/// and round, so that the light runs round the frame.
fn lit_group(time: f32) -> usize {
    (time * BULB_RATE) as usize % BULB_GROUPS
}

/// Lights the billboards' bulbs one group after another (their materials
/// changed only as the light steps on).
fn chase_bulbs(
    time: Res<Time>,
    bulbs: Query<(&Bulbs, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<Option<usize>>,
) {
    let lit = lit_group(time.elapsed_secs());
    if *last == Some(lit) || bulbs.is_empty() {
        return;
    }
    *last = Some(lit);
    for (bulbs, material) in &bulbs {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = if bulbs.group == lit {
                BULB_LIT
            } else {
                BULB_UNLIT
            };
        }
    }
}

/// Moves each plane round its flight.
fn fly(time: Res<Time>, mut planes: Query<(&Flight, &mut Transform)>) {
    let now = time.elapsed_secs();
    for (flight, mut transform) in &mut planes {
        *transform = flight.transform(now);
    }
}

/// A kilometre post every `every` metres round the lap from the line, on
/// `side` of the road, `lateral` metres from its middle, but where there is
/// no ground beside the road for it: in a tunnel, on a bridge, or where the
/// walls close in.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KmPosts {
    pub every: f32,
    pub side: Side,
    pub lateral: f32,
}

/// The bends of a stretch of the lap, each with a sign before it (see
/// [`bend_signs`]): on `side` of the road, `lateral` metres from its
/// middle, just outside its wall.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BendSigns {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
}

/// The props of Costa Bits, made with Meshy, as a circuit's file names
/// them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum CostaModel {
    WhiteHouse,
    Chiringuito,
    EspetoBoat,
    RetroSportsCar,
    Watchtower,
    LifeguardTower,
    FlagMarshal,
    CurveSign,
    CurveSignLeft,
    BoulderA,
    BoulderB,
    BoulderC,
    Mirador,
    KmPost,
    Ermita,
    PricklyPear,
    Agave,
    Ibex,
    CamperVan,
    CastleRuin,
    ArcadeBillboard,
    /// A small plane towing a blank banner, flying round over the bay (see
    /// [`Costa::planes`]); not placed among the props.
    BannerPlane,
    /// The old stone bridge over the gorge, over the road: placed
    /// `Over(along)`, the road running under its arch, where the walls
    /// close in.
    GorgeBridge,
}

/// Every prop of the theme.
#[cfg(test)]
const MODELS: [CostaModel; 23] = [
    CostaModel::WhiteHouse,
    CostaModel::Chiringuito,
    CostaModel::EspetoBoat,
    CostaModel::RetroSportsCar,
    CostaModel::Watchtower,
    CostaModel::LifeguardTower,
    CostaModel::FlagMarshal,
    CostaModel::CurveSign,
    CostaModel::CurveSignLeft,
    CostaModel::BoulderA,
    CostaModel::BoulderB,
    CostaModel::BoulderC,
    CostaModel::Mirador,
    CostaModel::KmPost,
    CostaModel::Ermita,
    CostaModel::PricklyPear,
    CostaModel::Agave,
    CostaModel::Ibex,
    CostaModel::CamperVan,
    CostaModel::CastleRuin,
    CostaModel::ArcadeBillboard,
    CostaModel::BannerPlane,
    CostaModel::GorgeBridge,
];

/// Every prop's name in `props.bin`.
fn name(model: CostaModel) -> &'static str {
    match model {
        CostaModel::WhiteHouse => "white_house",
        CostaModel::Chiringuito => "chiringuito",
        CostaModel::EspetoBoat => "espeto_boat",
        CostaModel::RetroSportsCar => "retro_sports_car",
        CostaModel::Watchtower => "watchtower",
        CostaModel::LifeguardTower => "lifeguard_tower",
        CostaModel::FlagMarshal => "flag_marshal",
        CostaModel::CurveSign => "curve_sign",
        CostaModel::CurveSignLeft => "curve_sign_left",
        CostaModel::BoulderA => "boulder_a",
        CostaModel::BoulderB => "boulder_b",
        CostaModel::BoulderC => "boulder_c",
        CostaModel::Mirador => "mirador",
        CostaModel::KmPost => "km_post",
        CostaModel::Ermita => "ermita",
        CostaModel::PricklyPear => "prickly_pear",
        CostaModel::Agave => "agave",
        CostaModel::Ibex => "ibex",
        CostaModel::CamperVan => "camper_van",
        CostaModel::CastleRuin => "castle_ruin",
        CostaModel::ArcadeBillboard => "arcade_billboard",
        CostaModel::BannerPlane => "banner_plane",
        CostaModel::GorgeBridge => "gorge_bridge",
    }
}

/// Whether a prop is to be seen from afar: no tree stands between it and
/// the road coming at it. Every one made with Meshy is, but the small ones
/// that stand right by the wall (the signs, the posts, the plants, the
/// boulders and the marshal).
fn landmark(model: CostaModel) -> bool {
    !matches!(
        model,
        CostaModel::CurveSign
            | CostaModel::CurveSignLeft
            | CostaModel::KmPost
            | CostaModel::PricklyPear
            | CostaModel::Agave
            | CostaModel::BoulderA
            | CostaModel::BoulderB
            | CostaModel::BoulderC
            | CostaModel::FlagMarshal
            | CostaModel::BannerPlane
    )
}

/// How sharp a bend is for a sign before it: a radius under this (m),
/// the road's curvature taken over [`BEND_SPAN`] metres of it (so that
/// the wobbles of a long sweep do not count).
const BEND_RADIUS: f32 = 300.0;
const BEND_SPAN: f32 = 30.0;
/// How far before a bend its sign stands (m), and how far past the end of
/// one the next may start for it to have its own (m): bends closer than
/// that are one, for a sign, if they bend the same way.
const SIGN_BEFORE: f32 = 70.0;
const BENDS_APART: f32 = 60.0;
/// How far the bridge's foot goes below the road at its piers (m), so that
/// it never stands on air where the road climbs.
const BRIDGE_SINK: f32 = 0.5;
/// How far a house's foot goes below the ground under its middle (m): its
/// ground floor in the rock of the rim it stands on, as the houses over a
/// gorge are built down into it, so that those on the gorge's rims stand
/// low before the bridge's side arches.
const HOUSE_SINK: f32 = 4.5;
/// Where the bridge's piers stand, either side of the road's middle (m),
/// for the height of its foot.
const BRIDGE_PIERS: f32 = 9.0;

impl Theme for Costa {
    fn spawn(
        &self,
        world: &mut ChildSpawnerCommands,
        track: &Track,
        terrain: &Terrain,
        scenery: &Scenery,
        assets: &mut Assets3d,
    ) -> Vec<Footprint> {
        let pack = assets.packs.get(PACK);
        let mut taken = Vec::new();
        // The billboards' bulbs: each group in a material of its own, the
        // props' with a light of its own (see `chase_bulbs`).
        let bulbs: Vec<Handle<StandardMaterial>> = (0..BULB_GROUPS)
            .map(|_| {
                let material = assets
                    .materials
                    .get(&pack.material)
                    .cloned()
                    .unwrap_or_default();
                assets.materials.add(material)
            })
            .collect();
        // Their picture: its frames in an image of its own, shining a
        // little of itself.
        let has_billboard = self
            .props
            .iter()
            .any(|prop| prop.model == CostaModel::ArcadeBillboard);
        let picture = has_billboard.then(|| {
            let frames = assets.images.add(texture(
                motorust_engine::content::file(BILLBOARD_PICTURE),
                true,
            ));
            assets.materials.add(StandardMaterial {
                base_color_texture: Some(frames.clone()),
                emissive: LinearRgba::gray(PICTURE_GLOW),
                emissive_texture: Some(frames),
                perceptual_roughness: 0.9,
                reflectance: 0.1,
                uv_transform: frame_uv(0),
                ..default()
            })
        });
        let signs = self
            .bend_signs
            .iter()
            .flat_map(|signs| bend_signs(track, signs));
        let posts = self
            .km_posts
            .map(|posts| km_posts(track, scenery, &posts))
            .unwrap_or_default();
        for prop in self.props.iter().copied().chain(signs).chain(posts) {
            let (footprint, placed) =
                spawn_prop(world, track, terrain, &prop, pack, &mut assets.mirrored);
            taken.push(footprint);
            if prop.model == CostaModel::ArcadeBillboard {
                for (group, material) in bulbs.iter().enumerate() {
                    let piece = pack.piece(&format!("{BILLBOARD_BULBS}{group}"));
                    world.spawn((
                        Mesh3d(piece.mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        placed,
                        Bulbs { group },
                    ));
                }
                if let Some(picture) = &picture {
                    world.spawn((
                        Mesh3d(pack.piece(BILLBOARD_PANEL).mesh.clone()),
                        MeshMaterial3d(picture.clone()),
                        placed,
                        Picture,
                    ));
                }
            }
        }
        // The rock over the road.
        if !self.overhangs.is_empty() {
            let rock = assets.materials.add(StandardMaterial {
                perceptual_roughness: 0.95,
                reflectance: 0.15,
                ..default()
            });
            for overhang in self.overhangs {
                world.spawn((
                    Mesh3d(assets.meshes.add(overhang_mesh(track, overhang))),
                    MeshMaterial3d(rock.clone()),
                ));
            }
        }
        // The planes, flying round over the water.
        for flight in self.planes {
            let piece = pack.piece(name(CostaModel::BannerPlane));
            world.spawn((
                Mesh3d(piece.mesh.clone()),
                MeshMaterial3d(pack.material.clone()),
                flight.transform(0.0),
                *flight,
            ));
        }
        // No tree in the way of the landmarks, from where the road comes at
        // them.
        for prop in self.props.iter().filter(|prop| landmark(prop.model)) {
            let (position, _) = spot(track, prop);
            taken.extend(in_view(track, position));
        }
        taken
    }

    fn packs() -> &'static [&'static str] {
        &[PACK]
    }
}

/// A sign before each bend of a stretch of the lap (see [`BendSigns`]):
/// where the road starts bending tighter than [`BEND_RADIUS`],
/// [`SIGN_BEFORE`] metres before it, facing the riders coming, its arrow
/// bending the way the road does.
fn bend_signs(track: &Track, signs: &BendSigns) -> Vec<Prop<CostaModel>> {
    let lap = track.lap_length();
    let step = track.step_length();
    let length = signs.span.length(lap);
    let mut placed = Vec::new();
    let mut last_bend = f32::MIN;
    let mut last_left = None;
    let mut into = 0.0;
    while into < length {
        let along = signs.span.from + into;
        let curvature = bend(track, along);
        if curvature.abs() > 1.0 / BEND_RADIUS {
            let left = curvature > 0.0;
            if into - last_bend > BENDS_APART || last_left != Some(left) {
                let model = if left {
                    CostaModel::CurveSignLeft
                } else {
                    CostaModel::CurveSign
                };
                placed.push(Prop {
                    model,
                    spot: Spot::Road {
                        along: (along - SIGN_BEFORE).rem_euclid(lap),
                        lateral: signs.side.sign() * signs.lateral,
                    },
                    scale: 1.0,
                    turn: 0.0,
                    lean: 0.0,
                    facing_riders: true,
                });
            }
            last_bend = into;
            last_left = Some(left);
        }
        into += step;
    }
    placed
}

/// The kilometre posts round the lap (see [`KmPosts`]), facing the road.
fn km_posts(track: &Track, scenery: &Scenery, posts: &KmPosts) -> Vec<Prop<CostaModel>> {
    let lap = track.lap_length();
    let clear = |along: f32| {
        let off_road = |span: &Span| span.contains(along, lap);
        !scenery.tunnels.iter().any(off_road)
            && !scenery.bridges.iter().any(|bridge| off_road(&bridge.span))
            && track.wall_at(track.index_at(along)) >= track.wall()
    };
    (1..)
        .map(|k| k as f32 * posts.every)
        .take_while(|&along| along < lap)
        .filter(|&along| clear(along))
        .map(|along| Prop {
            model: CostaModel::KmPost,
            spot: Spot::Road {
                along,
                lateral: posts.side.sign() * posts.lateral,
            },
            scale: 1.0,
            turn: 0.0,
            lean: 0.0,
            facing_riders: false,
        })
        .collect()
}

/// The road's curvature at `along` (1/m, positive bending left), over
/// [`BEND_SPAN`] metres of it about there.
fn bend(track: &Track, along: f32) -> f32 {
    let step = track.step_length();
    let reach = (BEND_SPAN / 2.0 / step).round() as i32;
    let sum: f32 = (-reach..=reach)
        .map(|k| track.curvature(track.index_at(along + k as f32 * step)))
        .sum();
    sum / (2 * reach + 1) as f32
}

/// The lowest of the ground under a prop round `position`, `reach` metres
/// across its middle.
fn lowest_ground(terrain: &Terrain, position: Vec2, reach: f32) -> f32 {
    [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .into_iter()
        .map(|offset| terrain.height(position + offset * 0.6 * reach))
        .fold(f32::MAX, f32::min)
}

/// A prop, standing on the lowest of the ground under its box, so that it
/// never floats; one that leans, lifted until its low end is on it (the
/// car tipped towards the road, on show); a
/// house on the ground under its middle; the bridge on the road under its arch, a little below the
/// lowest of it at its piers. Its surface is given to the water to mirror.
/// Returns the ground it takes, and where it was put.
fn spawn_prop(
    world: &mut ChildSpawnerCommands,
    track: &Track,
    terrain: &Terrain,
    prop: &Prop<CostaModel>,
    props: &PropPack,
    mirrored: &mut Vec<MirrorPoint>,
) -> (Footprint, Transform) {
    let piece = props.piece(name(prop.model));
    // The car is on show: its `lean` tips it about its length towards its
    // front (the side it shows the road), so that its bonnet, its seats
    // and its boot are seen from there, not its nose up or down.
    let on_show = prop.model == CostaModel::RetroSportsCar;
    let (position, rotation) = if on_show {
        let (position, upright) = spot(track, &Prop { lean: 0.0, ..*prop });
        (position, upright * Quat::from_rotation_x(prop.lean))
    } else {
        spot(track, prop)
    };
    let (low, high) = (piece.low * prop.scale, piece.high * prop.scale);
    let reach = (high.x - low.x).max(high.z - low.z) / 2.0;
    let foot = match prop.spot {
        Spot::Over { along } => {
            let index = track.index_at(along);
            [-BRIDGE_PIERS, 0.0, BRIDGE_PIERS]
                .into_iter()
                .map(|lateral| track.surface_point(index, lateral).y)
                .fold(f32::MAX, f32::min)
                - BRIDGE_SINK
        }
        // A house on a rim stands on the rock under its middle, its front
        // out over the edge, as the houses over a gorge do; anything else on
        // the lowest of the ground under it.
        _ if prop.model == CostaModel::WhiteHouse => terrain.height(position) - HOUSE_SINK,
        _ => lowest_ground(terrain, position, reach),
    };
    // One that leans is lifted until its low end (the car's low side) is on
    // the ground.
    let across = if on_show {
        high.z - low.z
    } else {
        high.x - low.x
    };
    let foot = foot + across / 2.0 * prop.lean.sin().abs();
    let placed = Transform::from_xyz(position.x, foot, position.y)
        .with_rotation(rotation)
        .with_scale(Vec3::splat(prop.scale));
    world.spawn((
        Mesh3d(piece.mesh.clone()),
        MeshMaterial3d(props.material.clone()),
        placed,
    ));
    mirrored.extend(piece.surface.iter().map(|point| point.placed(&placed)));
    ((position, reach), placed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CIRCUITS;
    use motorust_engine::bike::model::imported::read;
    use motorust_engine::world::prop_pack::bounds;

    /// The props' pack, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(motorust_engine::content::file("models/costa/props.bin"))
            .meshes
            .into_iter()
            .map(|piece| (piece.name, piece.mesh))
            .collect()
    }

    fn piece<'a>(pack: &'a [(String, Mesh)], name: &str) -> &'a Mesh {
        &pack
            .iter()
            .find(|(piece, _)| piece == name)
            .unwrap_or_else(|| panic!("no {name}"))
            .1
    }

    /// The circuits with the theme, and the theme of each.
    fn costas() -> Vec<(&'static motorust_engine::circuits::Circuit, &'static Costa)> {
        crate::game::register();
        CIRCUITS
            .iter()
            .filter_map(|circuit| Some((circuit, circuit.scenery.themes.get::<Costa>()?)))
            .collect()
    }

    #[test]
    fn every_prop_is_in_the_pack_on_the_ground_and_light() {
        crate::game::register();
        let pack = pack();
        for model in MODELS {
            let name = name(model);
            let mesh = piece(&pack, name);
            let (low, high) = bounds(mesh);
            assert!(low.y.abs() < 1e-3, "{name} stands on the ground");
            assert!((low.x + high.x).abs() < 0.05 * (high.x - low.x), "{name}");
            // (The billboard is centred with its bulbs, pieces of their own.)
            if model != CostaModel::ArcadeBillboard {
                assert!((low.z + high.z).abs() < 0.05 * (high.z - low.z), "{name}");
            }
            let triangles = mesh.indices().unwrap().len() / 3;
            assert!(triangles < 4_500, "{name}: {triangles} triangles");
        }
    }

    #[test]
    fn the_bridge_s_arch_is_open_to_the_ground_and_clear_of_the_walls() {
        let pack = pack();
        let mesh = piece(&pack, name(CostaModel::GorgeBridge));
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|p| p.as_float3())
            .unwrap();
        // Up to the height of a bike and its rider, and well over it, no
        // stone nearer the road's middle than the piers' faces.
        let (_, high) = bounds(mesh);
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        let nearest = indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|corner| corner.map(|i| Vec3::from(positions[i])))
            .filter(|p| p.iter().map(|p| p.y).fold(f32::MAX, f32::min) < 12.0)
            .map(|p| {
                // How near the middle the triangle comes (it may cross it).
                let (lo, hi) = (
                    p.iter().map(|p| p.x).fold(f32::MAX, f32::min),
                    p.iter().map(|p| p.x).fold(f32::MIN, f32::max),
                );
                if lo < 0.0 && hi > 0.0 {
                    0.0
                } else {
                    lo.abs().min(hi.abs())
                }
            })
            .fold(f32::MAX, f32::min);
        assert!(nearest > 7.0, "stone {nearest} m from the middle");
        assert!(high.y > 40.0, "{} m high", high.y);
        // Where it stands, the walls close in short of its piers.
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            for prop in costa
                .props
                .iter()
                .filter(|p| p.model == CostaModel::GorgeBridge)
            {
                let Spot::Over { along } = prop.spot else {
                    panic!("{}: the bridge stands over the road", circuit.name);
                };
                let depth = 10.0;
                for k in -2..=2 {
                    let index = track.index_at(along + k as f32 * depth / 2.0);
                    let wall = track.wall_at(index);
                    assert!(
                        wall < nearest - 0.3,
                        "{}: the wall at {wall} m under the bridge",
                        circuit.name
                    );
                }
            }
        }
    }

    #[test]
    fn props_stand_off_the_road_and_the_bridge_over_it() {
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            let signs = costa
                .bend_signs
                .iter()
                .flat_map(|signs| bend_signs(&track, signs));
            let posts = costa
                .km_posts
                .map(|posts| km_posts(&track, &circuit.scenery, &posts))
                .unwrap_or_default();
            for prop in costa.props.iter().copied().chain(signs).chain(posts) {
                if let Spot::Road { along, lateral } = prop.spot {
                    let wall = track.wall_at(track.index_at(along));
                    assert!(
                        lateral.abs() > wall + 0.5,
                        "{}: {:?} at {along} m, {lateral} m from the middle",
                        circuit.name,
                        prop.model
                    );
                }
            }
        }
    }

    #[test]
    fn a_sign_stands_before_each_bend_bending_its_way() {
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            let mut count = 0;
            for signs in costa.bend_signs {
                let placed = bend_signs(&track, signs);
                assert!(!placed.is_empty(), "{}: no sign", circuit.name);
                count += placed.len();
                for sign in placed {
                    let Spot::Road { along, .. } = sign.spot else {
                        unreachable!()
                    };
                    // The bend it warns of, ahead, bends as its arrow does.
                    let ahead = bend(&track, along + SIGN_BEFORE);
                    let left = sign.model == CostaModel::CurveSignLeft;
                    assert_eq!(ahead > 0.0, left, "{}: a sign at {along} m", circuit.name);
                }
            }
            assert!(count >= 5, "{}: {count} signs", circuit.name);
        }
    }

    #[test]
    fn a_kilometre_post_stands_every_500_m_but_where_there_is_no_ground() {
        for (circuit, costa) in costas() {
            let Some(posts) = costa.km_posts else {
                continue;
            };
            let track = Track::new(&circuit.layout);
            let placed = km_posts(&track, &circuit.scenery, &posts);
            let lap = track.lap_length();
            // All round the lap but in the tunnel and on the long bridge.
            assert!(
                placed.len() + 3 >= (lap / posts.every) as usize,
                "{}",
                circuit.name
            );
            for post in placed {
                let Spot::Road { along, .. } = post.spot else {
                    unreachable!()
                };
                assert!(
                    !circuit
                        .scenery
                        .tunnels
                        .iter()
                        .any(|t| t.contains(along, lap))
                );
            }
        }
    }

    #[test]
    fn the_planes_fly_round_over_the_water_clear_of_the_road() {
        let flights: Vec<_> = costas()
            .into_iter()
            .flat_map(|(circuit, costa)| costa.planes.iter().map(move |f| (circuit, f)))
            .collect();
        assert!(!flights.is_empty());
        for (circuit, flight) in flights {
            let track = Track::new(&circuit.layout);
            let lap = 2.0 * std::f32::consts::PI * flight.radius / flight.speed;
            for k in 0..64 {
                let time = lap * k as f32 / 64.0;
                let (position, heading) = flight.at(time);
                let point = Vec2::new(position.x, position.z);
                assert!(
                    track.nearest(point).distance > 60.0,
                    "{}: the plane at {point} over the road",
                    circuit.name
                );
                // Heading round the circle, its nose forward.
                let ahead = flight.at(time + 0.5).0;
                let moved = Vec2::new(ahead.x - position.x, ahead.z - position.z);
                assert!(moved.normalize().dot(heading) > 0.99);
                let nose = flight.transform(time).rotation * Vec3::NEG_X;
                assert!(Vec2::new(nose.x, nose.z).normalize().dot(heading) > 0.95);
            }
        }
    }

    #[test]
    fn the_rock_over_the_road_is_clear_of_the_riders_and_meets_the_rock() {
        let mut overhangs = 0;
        for (circuit, costa) in costas() {
            let track = Track::new(&circuit.layout);
            let lap = track.lap_length();
            for overhang in costa.overhangs {
                overhangs += 1;
                let length = overhang.span.length(lap);
                // High over a rider all the way out, thick at its root.
                for out in [0.0, 5.0, overhang.reach] {
                    let (under, top) = overhang.section(out, overhang.reach);
                    assert!(under >= 6.0, "{}: {under} m over the road", circuit.name);
                    assert!(top > under + 1.0);
                }
                // Out from cut rock on its side, as high as its root at
                // least, all along it; grown out in its middle, drawn back
                // at its ends.
                assert!(overhang.reach_at(length / 2.0, length) > 0.99 * overhang.reach);
                assert!(overhang.reach_at(0.0, length) < 0.01);
                let root = overhang.section(0.0, overhang.reach).1;
                let cut = circuit.scenery.cuttings.iter().any(|cutting| {
                    cutting.side == overhang.side
                        && cutting.span.contains(overhang.span.from, lap)
                        && cutting.span.contains(overhang.span.to, lap)
                        && cutting.height * 0.75 >= root
                });
                assert!(cut, "{}: no rock for it to come out of", circuit.name);
                let mesh = overhang_mesh(&track, overhang);
                assert!(mesh.count_vertices() > 1000);
            }
        }
        assert!(overhangs > 0);
    }

    #[test]
    fn the_billboard_s_bulbs_are_lit_one_group_after_another() {
        crate::game::register();
        let pack = pack();
        // Each group in the pack, round the billboard's frame: over its
        // legs, within its width.
        let (low, high) = bounds(piece(&pack, name(CostaModel::ArcadeBillboard)));
        for group in 0..BULB_GROUPS {
            let (bulbs_low, bulbs_high) =
                bounds(piece(&pack, &format!("{BILLBOARD_BULBS}{group}")));
            assert!(bulbs_low.y > 0.3 * high.y, "group {group}: {bulbs_low}");
            assert!(bulbs_low.x >= low.x - 0.1 && bulbs_high.x <= high.x + 0.1);
        }
        // The light steps round every group in turn, each as long.
        let step = 1.0 / BULB_RATE;
        let lit: Vec<usize> = (0..2 * BULB_GROUPS)
            .map(|k| lit_group((k as f32 + 0.5) * step))
            .collect();
        assert_eq!(lit, [0, 1, 2, 0, 1, 2]);
        // Its picture: a panel inside its frame, and its frames, each a
        // quarter of the image, shown in turn.
        let (panel_low, panel_high) = bounds(piece(&pack, BILLBOARD_PANEL));
        assert!(panel_low.y > 0.3 * high.y && panel_high.y < high.y);
        assert!(panel_low.x > low.x && panel_high.x < high.x);
        assert!(!motorust_engine::content::file(BILLBOARD_PICTURE).is_empty());
        let frames: Vec<Vec2> = (0..4)
            .map(|frame| frame_uv(frame).transform_point2(Vec2::splat(0.5)))
            .collect();
        assert_eq!(
            frames,
            [
                Vec2::new(0.25, 0.25),
                Vec2::new(0.75, 0.25),
                Vec2::new(0.25, 0.75),
                Vec2::new(0.75, 0.75)
            ]
        );
        assert_eq!(picture_frame(0.0), 0);
        assert_eq!(picture_frame(4.5 / PICTURE_RATE), 0);
    }
}
