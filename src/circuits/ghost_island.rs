//! Ghost Island: a lap of an island of feudal Japan, after Ghost of
//! Tsushima, its stretches each in a colour of their own. It crosses the
//! castle hill twice, in a figure of eight: first through it, by a tunnel at
//! the end of the castle street, and at the end of the lap over it, 33 m
//! above, through the castle.
//!
//! 1. The castle street, north, over a crest where the bike takes off,
//!    straight at the castle hill and into the tunnel under it, at its foot.
//! 2. Out onto the coast, round to the right, and east along it in esses,
//!    past the great torii standing in the sea.
//! 3. The shrine on the water: a boardwalk, narrower than the road, straight
//!    across an inlet, with a crest in the middle where the bike takes off.
//! 4. Up the susuki hills in esses at up to 12 %, south over their rolling
//!    top, two crests where the bike takes off, and down at 12 % to the lake.
//! 5. Round the lake, clockwise, three sides of it, along a shrine's
//!    vermilion fence on the lake's side: fields of flowers down to the
//!    water, a water wheel on its shore and, on the outside of the bend at
//!    its foot, the great maple over its clearing of red leaves.
//! 6. North from the lake to the temple, where the road splits in two: the
//!    lap snakes through its courtyard and a way to the left swings out
//!    early over a hump.
//! 7. Up the red ridge in bends at up to 12 %, most of the way through a
//!    narrow corridor of torii, as at Fushimi Inari in Kyoto, to its top,
//!    53 m up, and round to the left onto the castle ridge, west.
//! 8. Down along the castle ridge, where the road splits three ways again,
//!    the other way round, and over the tunnel through the castle hill.
//! 9. Up the bamboo hill at 12 %, and down it through the bamboo at 11 % in
//!    a run of tight esses, four bends winding like a river, and round to
//!    the left into the castle street.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::geometry::{fbm, smoothstep};
use crate::track::{Banking, Branch, Layout, Narrowing, WALL_DISTANCE};
use crate::world::scenery::*;

/// Where things are, in metres from the finish line.
const CASTLE_STREET: Span = span(5821.0, TUNNEL.from);
/// The tunnel through the castle hill, opening at the foot of its slope at
/// the end of the castle street.
const TUNNEL: Span = span(360.0, 620.0);
const COAST: Span = span(640.0, 1422.0);
const BOARDWALK: Span = span(1422.0, 1622.0);
/// How far from the centre line the fences of the boardwalk stand (m): it
/// is narrower than the asphalt of the road.
const BOARDWALK_WALL: f32 = 6.0;
const LAKE_SHORE: Span = span(2700.0, 3400.0);
/// Along the great maple's clearing, round the foot of the lake.
const MAPLE_FENCE: Span = span(3010.0, 3165.0);
const RED_RIDGE: Span = span(3764.0, 4322.0);
/// The corridor of torii up the red ridge, after Fushimi Inari's thousand
/// gates in Kyoto: narrower than the road, the gates standing over it where
/// its walls close in (m from the centre line).
const TORII_CORRIDOR: Span = span(3830.0, 4120.0);
const TORII_WALL: f32 = 6.5;
const CROSSING: Span = span(4672.0, 4880.0);
/// Over the susuki hills, from the boardwalk to the lake, and down the
/// castle ridge: the stretches the rows of small props run along (see
/// [`PROP_ROWS`]).
const SUSUKI: Span = span(1622.0, 2700.0);
const CASTLE_RIDGE: Span = span(CASTLE_SPLIT.0, CROSSING.to);
const BAMBOO: Span = span(4880.0, 5675.0);
/// Where the road splits: in two by the temple, and three ways on the castle
/// ridge.
const TEMPLE_SPLIT: (f32, f32) = (3414.0, 3764.0);
const CASTLE_SPLIT: (f32, f32) = (4322.0, 4672.0);

/// The island's colours: torii vermilion, and white.
const VERMILION: Color = Color::srgb(0.86, 0.2, 0.09);
const WHITE: Color = Color::srgb(0.95, 0.94, 0.9);

pub const GHOST_ISLAND: Circuit = Circuit {
    name: "Ghost Island",
    name_en: "Ghost Island",
    description: "Inspirada en Ghost of Tsushima: un castillo cruzado por\n\
                  debajo y por encima, cuestas, un lago entre flores y dos bifurcaciones",
    description_en: "Inspired by Ghost of Tsushima: a castle crossed under and\n\
                     over, steep hills, a lake among flowers and two splits",
    // "Blade of the Autumn Gale".
    // An island in autumn: the wind in the grass, birds, the sea and a few
    // gulls along the coast, and the last cicadas.
    ambience: Ambience {
        sea: 0.35,
        gulls: 0.25,
        birds: 0.6,
        cicadas: 0.3,
        wind: 0.65,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/autumn_gale.mp3"),
        length: 175.8,
        loudness: -15.9,
    },
    pace: 1.0,
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        // Banked round the lake, and hardly banked where the road splits,
        // as in Gold Mine.
        wall: WALL_DISTANCE,
        banking: &[
            Banking {
                from: 2860.0,
                to: 3300.0,
                factor: 2.5,
            },
            Banking {
                from: TEMPLE_SPLIT.0,
                to: TEMPLE_SPLIT.1,
                factor: 0.25,
            },
            Banking {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                factor: 0.25,
            },
        ],
        // The shrine's boardwalk and the corridor of torii are narrower than
        // the road: their walls close in on it.
        narrows: &[
            Narrowing {
                from: BOARDWALK.from,
                to: BOARDWALK.to,
                wall: BOARDWALK_WALL,
            },
            Narrowing {
                from: TORII_CORRIDOR.from,
                to: TORII_CORRIDOR.to,
                wall: TORII_WALL,
            },
        ],
        // The road splits in two by the temple, a way to the left of the lap
        // over a hump, and three ways on the castle ridge: one to the left
        // and one to the right.
        branches: &[
            // By the temple, to the left, over a hump.
            Branch {
                from: TEMPLE_SPLIT.0,
                to: TEMPLE_SPLIT.1,
                approach: &TEMPLE_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &TEMPLE_LEFT_RISE,
                narrows: &[],
            },
            // On the castle ridge, to the left, over a hump.
            Branch {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                approach: &CASTLE_LEFT,
                vertical_loop: None,
                exit: &[],
                rise: &CASTLE_LEFT_RISE,
                narrows: &[],
            },
            // On the castle ridge, to the right, through a dip.
            Branch {
                from: CASTLE_SPLIT.0,
                to: CASTLE_SPLIT.1,
                approach: &CASTLE_RIGHT,
                vertical_loop: None,
                exit: &[],
                rise: &CASTLE_RIGHT_RISE,
                narrows: &[],
            },
        ],
    },
    scenery: Scenery {
        coast: Coast::OutlineWithLakes(&SHORE, lakes),
        hills: 22.0,
        greens: Greens::Countryside,
        grass_shade: 0.36,
        massifs: &MASSIFS,
        cuttings: &CUTTINGS,
        // The clearing under the great maple, on a knoll rising from the
        // road beside it, tilted up away from it, so that the tree stands
        // high and its carpet of leaves faces the riders; and the
        // fishing village's farmhouses raised a little over the coast road
        // climbing past them, so that they show over its wall.
        terraces: &[
            // The top of the great torii's islet, level for it to stand on,
            // a little under the road.
            Terrace {
                center: TORII_ISLET,
                radius: 10.0,
                height: 5.5,
                tilt: Vec2::ZERO,
                skirt: 5.0,
            },
            // The pagoda's ground before the corridor of torii, a little
            // over the road, so that it stands out as the riders come up
            // at it.
            Terrace {
                center: PAGODA_GROUND,
                radius: 10.0,
                height: 25.0,
                tilt: Vec2::ZERO,
                skirt: 10.0,
            },
            // The shrine in the bamboo, a little over the road.
            Terrace {
                center: BAMBOO_SHRINE,
                radius: 12.0,
                height: 41.0,
                tilt: Vec2::ZERO,
                skirt: 10.0,
            },
            Terrace {
                center: MAPLE_GROUND,
                radius: 16.0,
                height: 9.0,
                tilt: Vec2::new(0.002, 0.15),
                skirt: 40.0,
            },
            Terrace {
                center: Vec2::new(96.6, -760.7),
                radius: 7.0,
                height: 9.0,
                tilt: Vec2::ZERO,
                skirt: 10.0,
            },
            Terrace {
                center: Vec2::new(154.0, -774.4),
                radius: 7.0,
                height: 10.9,
                tilt: Vec2::ZERO,
                skirt: 10.0,
            },
            Terrace {
                center: Vec2::new(213.2, -764.5),
                radius: 7.0,
                height: 12.0,
                tilt: Vec2::ZERO,
                skirt: 10.0,
            },
        ],
        mountains,
        curbs: [VERMILION, WHITE],
        // Stone walls along the castle street, the coast and the castle
        // ridge, a hedge round the lake, and timber everywhere else.
        walls: &[
            // The wall opens into the shrine's vermilion fence on its
            // own where a small prop stands (see `world::japan`); the
            // stores, which stand on no plinth, have their openings here.
            WallRun {
                span: span(123.0, 137.0),
                side: Side::Left,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: span(313.0, 327.0),
                side: Side::Right,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: span(4180.0, 4262.0),
                side: Side::Right,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: span(4693.0, 4747.0),
                side: Side::Left,
                style: WallStyle::ShrineFence,
            },
            // Along the shore before the great torii, open to the sea: the
            // gate shows whole, from the water up, as the road swings
            // towards it, and the bronze lanterns past it at the boardwalk.
            WallRun {
                span: span(1090.0, 1402.0),
                side: Side::Left,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: CASTLE_STREET,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: CASTLE_STREET,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: COAST,
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: COAST,
                side: Side::Right,
                style: WallStyle::Stone,
            },
            // Before the great maple, on the outside of the bend, the
            // shrine's vermilion fence too: its clearing shows through it.
            WallRun {
                span: MAPLE_FENCE,
                side: Side::Left,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: LAKE_SHORE,
                side: Side::Left,
                style: WallStyle::Hedge,
            },
            // On the lake's side, a shrine's vermilion fence, low and open:
            // the flowers and the water show through it.
            WallRun {
                span: LAKE_SHORE,
                side: Side::Right,
                style: WallStyle::ShrineFence,
            },
            WallRun {
                span: span(CASTLE_SPLIT.0, CROSSING.to),
                side: Side::Left,
                style: WallStyle::Stone,
            },
            WallRun {
                span: span(CASTLE_SPLIT.0, CROSSING.to),
                side: Side::Right,
                style: WallStyle::Stone,
            },
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::Timber,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::Timber,
            },
        ],
        tunnels: &[TUNNEL],
        // The shrine's boardwalk, planks on piers across the inlet, low over
        // the water, its fence low: the shrine on stilts shows over it.
        bridges: &[Bridge {
            span: BOARDWALK,
            style: BridgeStyle::Wooden,
            fence: 0.75,
        }],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: VERMILION,
                text: Some("GHOST ISLAND"),
            },
            // The checkpoints, about a kilometre apart (none on the
            // boardwalk nor where the road splits): along the coast, up the
            // susuki hills, by the lake, at the top of the red ridge and
            // down through the bamboo.
            Gantry {
                along: 1000.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 2200.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 3150.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 4230.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 5100.0,
                color: VERMILION,
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[Billboard {
            along: 250.0,
            side: Side::Right,
            text: "MOTORUST",
            color: Color::srgb(0.1, 0.1, 0.12),
            accent: Some((4, crate::ui::RUST_ORANGE)),
        }],
        lamps: &[],
        // Maples, ginkgos, cherries, black pines and cedars.
        trees: &[
            Species::Momiji,
            Species::Ginkgo,
            Species::Sakura,
            Species::BlackPine,
            Species::Sugi,
        ],
        // Dense forest of cedars and maples up the red ridge, which the
        // corridor of torii climbs through, as on Mount Inari, and the
        // bamboo, on both sides of the road down from the castle.
        forests: &[
            forest(RED_RIDGE, &[Species::Sugi, Species::Momiji]),
            forest(BAMBOO, &[Species::Bamboo]),
        ],
        // The castle town: white plaster under tiled roofs (the fishing
        // village on the coast is of thatched farmhouses, see `PROPS`).
        houses: Houses {
            max: 8,
            paints: &[
                Color::srgb(0.94, 0.92, 0.86),
                Color::srgb(0.88, 0.84, 0.76),
                Color::srgb(0.55, 0.42, 0.32),
            ],
            spans: &[CASTLE_STREET],
            style: HouseStyle::Cottage,
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[],
        chevrons: &[],
        grandstands: &[Grandstand {
            along: 60.0,
            side: Side::Left,
            lateral: WALL_DISTANCE + 5.0,
            length: 30.0,
        }],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Rock,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::GoldenHour,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        desert: None,
        mine: None,
        space: None,
        toys: None,
        wheelie_halo: None,
        japan: Some(Japan {
            // Up the red ridge, as at Fushimi Inari: a gate every 6 m where
            // the walls close in, and wide ones before and after.
            corridors: &[ToriiCorridor {
                span: TORII_CORRIDOR,
                every: 6.0,
                approach: 30.0,
                approach_every: 15.0,
            }],
            props: &PROPS,
            rows: &PROP_ROWS,
            // Under the great maple, beside the road round the foot of the
            // lake.
            maple_grounds: &[MapleGround {
                center: MAPLE_GROUND,
                radius: 14.0,
            }],
            // Red maple leaves blowing down up the red ridge, through the
            // corridor of torii, and round the great maple; white cherry
            // petals down the castle street.
            petals: &[
                Petals {
                    area: PetalArea::Road {
                        span: RED_RIDGE,
                        reach: 24.0,
                    },
                    kind: PetalKind::Maple,
                },
                Petals {
                    area: PetalArea::Round {
                        center: MAPLE_GROUND,
                        radius: 36.0,
                    },
                    kind: PetalKind::Maple,
                },
                // Blowing over the whole bend round the foot of the lake,
                // along the great maple's clearing.
                Petals {
                    area: PetalArea::Round {
                        center: Vec2::new(956.8, 422.0),
                        radius: 32.0,
                    },
                    kind: PetalKind::Maple,
                },
                Petals {
                    area: PetalArea::Round {
                        center: MAPLE_ROAD,
                        radius: 32.0,
                    },
                    kind: PetalKind::Maple,
                },
                Petals {
                    area: PetalArea::Round {
                        center: Vec2::new(790.0, 425.0),
                        radius: 32.0,
                    },
                    kind: PetalKind::Maple,
                },
                Petals {
                    area: PetalArea::Road {
                        span: CASTLE_STREET,
                        reach: 24.0,
                    },
                    kind: PetalKind::Cherry,
                },
            ],
            flowers: &FLOWERS,
        }),
        middle_earth: None,
    },
};

/// The fields of flowers, as on the hills of Tsushima and round the lakes at
/// the foot of Mount Fuji, over most of the island; where they overlap, the
/// first one listed.
const FLOWERS: [FlowerField; 7] = [
    // White round the great maple, for its reds to stand out.
    FlowerField {
        area: FlowerArea::Round {
            center: MAPLE_GROUND,
            inner: 0.0,
            outer: 60.0,
        },
        kinds: &[FlowerKind::White],
    },
    // Red spider lilies on the lake's open shore, seen across the water.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(836.0, 100.0),
            inner: 0.0,
            outer: 60.0,
        },
        kinds: &[FlowerKind::SpiderLily],
    },
    // All round the lake, from its water up to the road's fence: carpets of
    // moss phlox in drifts of magenta, pink, lavender and white.
    FlowerField {
        area: FlowerArea::Round {
            center: LAKE,
            inner: LAKE_RADIUS - LAKE_WANDER - 4.0,
            outer: 175.0,
        },
        kinds: &[FlowerKind::Phlox],
    },
    // A meadow of red spider lilies beside the castle street.
    FlowerField {
        area: FlowerArea::Round {
            center: Vec2::new(110.0, -150.0),
            inner: 0.0,
            outer: 110.0,
        },
        kinds: &[FlowerKind::SpiderLily],
    },
    // The susuki hills, golden with susuki either side of the road.
    FlowerField {
        area: FlowerArea::Road {
            span: span(1800.0, 2650.0),
            near: WALL_DISTANCE,
            far: 150.0,
        },
        kinds: &[FlowerKind::Susuki],
    },
    // Yellow rape blossom and white flowers up the slopes over the coast.
    FlowerField {
        area: FlowerArea::Road {
            span: span(700.0, 1400.0),
            near: WALL_DISTANCE,
            far: 120.0,
        },
        kinds: &[FlowerKind::Yellow, FlowerKind::White],
    },
    // And everywhere else, in broad swathes.
    FlowerField {
        area: FlowerArea::Everywhere,
        kinds: &[
            FlowerKind::Phlox,
            FlowerKind::SpiderLily,
            FlowerKind::Yellow,
            FlowerKind::White,
            FlowerKind::Iris,
        ],
    },
];

/// How far from the middle of the road the small props stand (m): just
/// past its walls.
const NEAR: f32 = WALL_DISTANCE + 1.3;

/// The props of feudal Japan, made with Meshy: landmarks out in the open,
/// and statues, lanterns, shrines and houses by the road.
const PROPS: [Prop; 44] = [
    // The great torii on its islet in the sea off the coast road, facing
    // it, ahead of the road as it swings along the shore.
    standing(PropModel::GreatTorii, TORII_ISLET.x, TORII_ISLET.y, PI).scaled(1.6),
    // The castle keep on the bamboo hill, on the outside of the bend past
    // the castle, facing the riders coming over the tunnel: ahead of them
    // all the way along the castle ridge.
    standing(PropModel::CastleKeep, -162.1, -527.7, -1.76).scaled(1.6),
    // The temple and its pagoda before the corridor of torii, both facing
    // the riders coming up the red ridge at them: the hall stands on the
    // bank to the left, in the open (among the gates it was hidden), and
    // the pagoda by the road to the right, on a terrace a little over it.
    by_road(PropModel::TempleHall, 3640.0, 40.0).facing_riders(),
    by_road(PropModel::Pagoda, 3780.0, -34.0).facing_riders(),
    // The shrine on stilts in the inlet, beside the boardwalk, and another
    // on land, on a terrace on the outside of a bend in the bamboo, facing
    // the riders coming down.
    standing(PropModel::StiltShrine, 760.0, -755.0, 0.0),
    standing(
        PropModel::StiltShrine,
        BAMBOO_SHRINE.x,
        BAMBOO_SHRINE.y,
        0.39,
    )
    .scaled(1.5),
    // The great maple on the outside of the bend round the foot of the
    // lake, straight ahead all the way down to it: at the far side of its
    // clearing of red leaves, its crown spreading over it towards the road
    // and the riders coming, leaning out a little over it; and a stone
    // lantern either side of the clearing. A wayside shrine on the islet.
    standing(
        PropModel::GreatMaple,
        MAPLE_GROUND.x + 2.0,
        MAPLE_GROUND.y + 11.9,
        1.215,
    )
    .leaning(0.12),
    standing(
        PropModel::StoneLantern,
        MAPLE_GROUND.x - 18.1,
        MAPLE_GROUND.y - 3.6,
        -0.5,
    ),
    standing(
        PropModel::StoneLantern,
        MAPLE_GROUND.x + 17.9,
        MAPLE_GROUND.y - 4.4,
        0.55,
    ),
    standing(PropModel::Hokora, ISLET.x, ISLET.y, 2.36),
    // A water wheel on the lake's west shore, ahead past the great maple,
    // facing the riders coming round from it.
    standing(PropModel::WaterWheel, 745.6, 355.6, -2.44).scaled(1.8),
    // Fishing boats off the coast.
    standing(PropModel::FishingBoat, 250.0, -925.0, 1.2),
    standing(PropModel::FishingBoat, 560.0, -935.0, -0.4),
    standing(PropModel::FishingBoat, 700.0, -870.0, 2.0),
    // Stone lanterns along the castle street, before the temple, by the
    // lake, at the top of the red ridge and at the castle.
    // The small ones stand right by the road, past the walls where the
    // shrine's fence opens for them (see the walls).
    by_road(PropModel::StoneLantern, 60.0, NEAR),
    by_road(PropModel::StoneLantern, 200.0, -NEAR),
    by_road(PropModel::StoneLantern, 330.0, NEAR),
    by_road(PropModel::StoneLantern, 3395.0, NEAR),
    by_road(PropModel::StoneLantern, 3395.0, -NEAR),
    by_road(PropModel::StoneLantern, 3300.0, -NEAR),
    by_road(PropModel::StoneLantern, 4280.0, NEAR),
    by_road(PropModel::StoneLantern, 4740.0, NEAR),
    // The castle town's stores.
    by_road(PropModel::SupplyPile, 130.0, 13.5),
    by_road(PropModel::SupplyPile, 320.0, -13.5),
    // The fishing village: thatched farmhouses inland of the coast road,
    // each on its terrace, facing the riders coming along it.
    standing(PropModel::Minka, 96.6, -760.7, 1.83),
    standing(PropModel::Minka, 154.0, -774.4, 1.44),
    standing(PropModel::Minka, 213.2, -764.5, 1.01),
    // Bronze lanterns at either end of the boardwalk.
    by_road(PropModel::BronzeLantern, 1395.0, NEAR),
    by_road(PropModel::BronzeLantern, 1395.0, -NEAR),
    by_road(PropModel::BronzeLantern, 1655.0, NEAR),
    by_road(PropModel::BronzeLantern, 1655.0, -NEAR),
    // A torii of logs on the top of the susuki hills, facing the riders
    // coming up at it: it stands against the sky all the way up.
    by_road(PropModel::LogTorii, 2350.0, 30.0).facing_riders(),
    // Guardians at the foot of the corridor of torii.
    by_road(PropModel::Komainu, 3800.0, NEAR),
    by_road(PropModel::Komainu, 3800.0, -NEAR),
    // An ancient maple on its boulders with a wayside shrine beside it, and
    // a memorial stone, at the top of the red ridge.
    by_road(PropModel::AncientMaple, 4190.0, -24.0),
    by_road(PropModel::Hokora, 4205.0, -13.5),
    by_road(PropModel::Stele, 4250.0, -NEAR),
    // Braziers at the castle.
    by_road(PropModel::Brazier, 4700.0, NEAR),
    by_road(PropModel::Brazier, 4700.0, -NEAR),
    // Jizō by the road down through the bamboo.
    by_road(PropModel::Jizo, 4960.0, NEAR),
    by_road(PropModel::Jizo, 5160.0, -NEAR),
    by_road(PropModel::Jizo, 5360.0, NEAR),
    by_road(PropModel::Jizo, 5520.0, -NEAR),
    // And one more by the castle street.
    by_road(PropModel::Jizo, 5881.0, NEAR),
];

/// The small props over and over along the lap (see [`PropRow`]): each
/// stands on its plinth, with the wall opening for it, so that the road
/// is lined with them. Every row takes its kinds in turn, so that no
/// stretch is all of one thing, and the two sides of a stretch are half a
/// step apart, so they do not stand in pairs facing each other (but for
/// the guardians, which come in twos). Where one of [`PROPS`] already
/// stands, or another road runs, there is none.
const PROP_ROWS: [PropRow; 12] = [
    // The castle street, lit on both sides as a town's is, with a brazier
    // and a jizō among the lanterns.
    prop_row(
        &[PropModel::StoneLantern, PropModel::Brazier],
        CASTLE_STREET,
        Side::Left,
        NEAR,
        70.0,
    ),
    prop_row(
        &[PropModel::StoneLantern, PropModel::Jizo],
        span(CASTLE_STREET.from + 35.0, CASTLE_STREET.to),
        Side::Right,
        NEAR,
        70.0,
    ),
    // The coast road, on its inland side: the sea's is left open.
    prop_row(
        &[PropModel::Jizo, PropModel::StoneLantern, PropModel::Stele],
        COAST,
        Side::Right,
        NEAR,
        115.0,
    ),
    // The lonely road over the susuki hills: wayside statues, a shrine
    // and a memorial stone.
    prop_row(
        &[PropModel::Jizo, PropModel::Hokora, PropModel::Stele],
        SUSUKI,
        Side::Left,
        NEAR,
        190.0,
    ),
    // Round the lake, on the water's side, along the shrine's fence.
    prop_row(
        &[PropModel::StoneLantern, PropModel::BronzeLantern],
        LAKE_SHORE,
        Side::Right,
        NEAR,
        85.0,
    ),
    // Up the red ridge: braziers and memorial stones, and guardians in
    // twos, one either side, as at the foot of the corridor of torii.
    prop_row(
        &[PropModel::Brazier, PropModel::Stele],
        RED_RIDGE,
        Side::Left,
        NEAR,
        110.0,
    ),
    prop_row(&[PropModel::Komainu], RED_RIDGE, Side::Left, NEAR, 150.0),
    prop_row(&[PropModel::Komainu], RED_RIDGE, Side::Right, NEAR, 150.0),
    // Down the castle ridge, lit for the night.
    prop_row(
        &[PropModel::Brazier, PropModel::Komainu, PropModel::Stele],
        CASTLE_RIDGE,
        Side::Left,
        NEAR,
        95.0,
    ),
    prop_row(
        &[PropModel::Brazier, PropModel::Hokora],
        span(CASTLE_RIDGE.from + 47.0, CASTLE_RIDGE.to),
        Side::Right,
        NEAR,
        95.0,
    ),
    // Down through the bamboo, where the jizō stand: a shrine and a stone
    // among them, one side half a step after the other.
    prop_row(
        &[PropModel::Jizo, PropModel::Hokora],
        BAMBOO,
        Side::Left,
        NEAR,
        140.0,
    ),
    prop_row(
        &[PropModel::Jizo, PropModel::Stele],
        span(BAMBOO.from + 70.0, BAMBOO.to),
        Side::Right,
        NEAR,
        140.0,
    ),
];

const CONTROL_POINTS: [Vec2; 110] = [
    // 1. The castle street, north (-Z), and the tunnel under the castle hill.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -140.0),
    Vec2::new(0.0, -280.0),
    Vec2::new(0.0, -420.0),
    Vec2::new(0.0, -520.0),
    Vec2::new(0.0, -620.0),
    // 2. Out of the tunnel, round to the right onto the coast, and east
    // along it in esses.
    Vec2::new(8.0, -700.0),
    Vec2::new(45.0, -760.0),
    Vec2::new(110.0, -795.0),
    Vec2::new(185.0, -800.0),
    Vec2::new(255.0, -778.0),
    Vec2::new(325.0, -790.0),
    Vec2::new(390.0, -828.0),
    Vec2::new(460.0, -842.0),
    Vec2::new(530.0, -818.0),
    Vec2::new(590.0, -800.0),
    // 3. The shrine's boardwalk, straight across the inlet.
    Vec2::new(660.0, -800.0),
    Vec2::new(760.0, -800.0),
    Vec2::new(860.0, -800.0),
    // 4. Round to the right, up the susuki hills in esses, and south over
    // their rolling top.
    Vec2::new(935.0, -790.0),
    Vec2::new(990.0, -750.0),
    Vec2::new(1015.0, -685.0),
    Vec2::new(1005.0, -615.0),
    Vec2::new(975.0, -550.0),
    Vec2::new(975.0, -480.0),
    Vec2::new(1010.0, -415.0),
    Vec2::new(1040.0, -345.0),
    Vec2::new(1045.0, -260.0),
    Vec2::new(1045.0, -170.0),
    Vec2::new(1045.0, -80.0),
    Vec2::new(1045.0, 10.0),
    Vec2::new(1045.0, 95.0),
    // 5. Round the lake, clockwise: its east, south and west shores, 175 m
    // from its middle.
    Vec2::new(1045.0, 270.0),
    Vec2::new(1021.6, 357.5),
    Vec2::new(957.5, 421.6),
    Vec2::new(870.0, 445.0),
    Vec2::new(782.5, 421.6),
    Vec2::new(718.4, 357.5),
    Vec2::new(695.0, 270.0),
    // 6. North past the temple: the lap snakes between the ways of its
    // split, as through the second split of Gold Mine (where the lap climbs
    // as it does here).
    Vec2::new(695.0, 220.0),
    Vec2::new(685.9, 178.8),
    Vec2::new(671.1, 137.5),
    Vec2::new(672.9, 96.2),
    Vec2::new(695.0, 55.0),
    Vec2::new(717.1, 13.8),
    Vec2::new(718.9, -27.5),
    Vec2::new(704.1, -68.8),
    Vec2::new(695.0, -110.0),
    Vec2::new(695.0, -160.0),
    // 7. Up the red ridge in bends, and round to the left at its top.
    Vec2::new(707.0, -230.0),
    Vec2::new(705.0, -300.0),
    Vec2::new(675.0, -360.0),
    Vec2::new(655.0, -420.0),
    Vec2::new(630.0, -475.0),
    Vec2::new(585.0, -512.0),
    Vec2::new(525.0, -525.0),
    // 8. West along the castle ridge, through its split (the first split
    // of Gold Mine, where the lap comes down as it does here), and over the
    // tunnel.
    Vec2::new(470.0, -525.0),
    Vec2::new(428.8, -534.1),
    Vec2::new(387.5, -548.9),
    Vec2::new(346.2, -547.1),
    Vec2::new(305.0, -525.0),
    Vec2::new(263.8, -502.9),
    Vec2::new(222.5, -501.1),
    Vec2::new(181.2, -515.9),
    Vec2::new(140.0, -525.0),
    Vec2::new(90.0, -525.0),
    Vec2::new(-40.0, -523.0),
    // 9. Up the bamboo hill, and down it in a run of tight esses, four
    // bends winding like a river, and round to the left into the castle
    // street.
    Vec2::new(-58.2, -523.0),
    Vec2::new(-76.5, -523.0),
    Vec2::new(-94.5, -523.0),
    Vec2::new(-112.7, -521.5),
    Vec2::new(-129.5, -514.7),
    Vec2::new(-143.8, -503.8),
    Vec2::new(-157.5, -491.7),
    Vec2::new(-168.9, -477.6),
    Vec2::new(-175.6, -460.9),
    Vec2::new(-175.5, -442.8),
    Vec2::new(-168.8, -426.2),
    Vec2::new(-157.3, -412.1),
    Vec2::new(-143.6, -400.0),
    Vec2::new(-130.2, -388.0),
    Vec2::new(-119.2, -373.5),
    Vec2::new(-113.2, -356.4),
    Vec2::new(-114.1, -338.5),
    Vec2::new(-121.5, -322.0),
    Vec2::new(-133.3, -308.4),
    Vec2::new(-147.1, -296.4),
    Vec2::new(-160.4, -284.0),
    Vec2::new(-170.9, -269.5),
    Vec2::new(-176.2, -252.1),
    Vec2::new(-174.5, -234.1),
    Vec2::new(-166.5, -218.0),
    Vec2::new(-154.2, -204.5),
    Vec2::new(-140.4, -192.7),
    Vec2::new(-127.4, -180.2),
    Vec2::new(-117.2, -165.1),
    Vec2::new(-112.8, -147.8),
    Vec2::new(-115.3, -129.8),
    Vec2::new(-124.0, -113.9),
    Vec2::new(-136.4, -100.9),
    Vec2::new(-150.3, -89.0),
    Vec2::new(-163.4, -76.3),
    Vec2::new(-173.1, -61.3),
    Vec2::new(-178.3, -43.9),
    Vec2::new(-180.7, -26.0),
    Vec2::new(-188.0, 45.0),
    Vec2::new(-155.0, 112.0),
    Vec2::new(-98.0, 150.0),
    Vec2::new(-42.0, 138.0),
    Vec2::new(-9.0, 82.0),
];

/// Through the tunnel near the sea; rolling along the coast; up the susuki
/// hills, 30 m, and down to the lake; up past the temple and the red ridge,
/// 45 m, and down along the castle ridge, over the tunnel, 33 m above it;
/// up the bamboo hill, 15 m, and down its esses, 45 m, into the castle
/// street. Up and down at up to 12 %.
const ELEVATION: [(f32, f32); 71] = [
    // The castle street, over a crest where the bike flies, and the tunnel.
    (0.0, 0.0),
    (110.0, 0.4),
    (185.0, 4.6),
    (225.0, 3.0),
    (300.0, 0.8),
    (400.0, 0.3),
    (525.0, 0.5),
    (640.0, 1.0),
    // Rolling along the coast.
    (720.0, 2.0),
    (840.0, 5.5),
    (960.0, 8.0),
    // Down by the shore before the great torii.
    (1080.0, 6.0),
    (1200.0, 4.2),
    (1320.0, 3.4),
    (1400.0, 1.0),
    // The boardwalk, at the water's level by the shrine on stilts: 1.6 m
    // over it.
    (1470.0, -1.4),
    (1540.0, -1.4),
    (1585.0, -1.4),
    (1650.0, -1.2),
    // Up the susuki hills, at up to 12 %.
    (1760.0, 6.0),
    (1820.0, 9.0),
    (1900.0, 18.0),
    (1980.0, 27.0),
    (2060.0, 32.4),
    (2125.0, 33.4),
    (2200.0, 33.6),
    // Over their top, two crests where the bike flies, shaped like the one
    // in the castle street.
    (2270.0, 34.0),
    (2345.0, 38.2),
    (2385.0, 36.6),
    (2460.0, 34.2),
    (2535.0, 38.4),
    (2575.0, 36.8),
    (2620.0, 35.0),
    // Down to the lake, at up to 12 %, and round it low, just over the
    // water.
    (2700.0, 27.4),
    (2780.0, 18.4),
    (2860.0, 9.4),
    (2940.0, 3.4),
    (3020.0, 1.0),
    (3100.0, 0.6),
    (3200.0, 0.8),
    (3290.0, 3.8),
    (3364.0, 6.5),
    // Climbing steadily through the temple's split.
    (3414.0, 7.0),
    (3764.0, 19.0),
    // Up the red ridge, at up to 12 %, and over its top in the bend onto
    // the castle ridge.
    (3814.0, 21.0),
    (3880.0, 26.0),
    (3960.0, 35.4),
    (4040.0, 44.6),
    (4110.0, 49.6),
    (4170.0, 51.0),
    (4230.0, 51.2),
    (4290.0, 49.6),
    // Down steadily through the castle's split (steeper before it and after
    // it, so that the road splits where its grade eases), and over the
    // tunnel.
    (4322.0, 48.2),
    (4672.0, 36.2),
    (4722.0, 34.0),
    (4780.0, 33.4),
    (4840.0, 33.6),
    // Up the bamboo hill at up to 12 %, over its top in the first bend, and
    // down its esses at 11 % into the castle street.
    (4900.0, 37.0),
    (4960.0, 43.5),
    (5010.0, 47.5),
    (5050.0, 48.4),
    (5090.0, 46.5),
    (5150.0, 40.0),
    (5230.0, 31.0),
    (5310.0, 22.0),
    (5390.0, 13.5),
    (5460.0, 7.5),
    (5530.0, 4.2),
    (5650.0, 2.2),
    (5800.0, 1.0),
    (5900.0, 0.3),
];

/// The ways off the lap, each through points to one side of it (m along the
/// lap and from its centre line in the comments), from where it leaves the
/// lap to where it joins it again: those of Gold Mine's second split,
/// turned north. By the temple, to the left: it swings out to the left
/// before the lap does, and back to it late.
const TEMPLE_LEFT: [Vec2; 11] = [
    // 3426 m, 0 m.
    Vec2::new(693.2, 210.1),
    // 3456 m, 4 m.
    Vec2::new(682.4, 181.5),
    // 3494 m, 13 m.
    Vec2::new(659.8, 148.6),
    // 3522 m, 32 m.
    Vec2::new(637.9, 114.4),
    // 3544 m, 48 m.
    Vec2::new(626.7, 81.4),
    // 3562 m, 60 m.
    Vec2::new(629.3, 48.4),
    // 3598 m, 65 m.
    Vec2::new(643.3, 15.4),
    // 3676 m, 57 m.
    Vec2::new(662.7, -17.5),
    // 3710 m, 29 m.
    Vec2::new(682.0, -45.2),
    // 3728 m, 11 m.
    Vec2::new(692.0, -72.2),
    // 3752 m, 2 m.
    Vec2::new(695.5, -96.6),
];

/// On the castle ridge, as in Gold Mine's first split, to the left: it runs
/// straight on where the lap swings right, and out round a wide curve back
/// to it.
const CASTLE_LEFT: [Vec2; 11] = [
    // 4334 m, 1 m.
    Vec2::new(458.3, -525.2),
    // 4362 m, 15 m.
    Vec2::new(427.1, -519.1),
    // 4384 m, 40 m.
    Vec2::new(395.1, -504.2),
    // 4446 m, 65 m.
    Vec2::new(364.4, -484.2),
    // 4506 m, 64 m.
    Vec2::new(331.4, -465.8),
    // 4534 m, 57 m.
    Vec2::new(298.4, -455.6),
    // 4550 m, 44 m.
    Vec2::new(265.4, -458.6),
    // 4574 m, 25 m.
    Vec2::new(232.4, -475.0),
    // 4608 m, 9 m.
    Vec2::new(198.7, -498.9),
    // 4636 m, 3 m.
    Vec2::new(173.9, -515.0),
    // 4658 m, 0 m.
    Vec2::new(153.6, -522.5),
];

/// On the castle ridge, to the right: it swings out to the right before the
/// lap does, and back to it late.
const CASTLE_RIGHT: [Vec2; 11] = [
    // 4334 m, -0 m.
    Vec2::new(458.7, -527.0),
    // 4364 m, -4 m.
    Vec2::new(430.6, -537.9),
    // 4402 m, -14 m.
    Vec2::new(398.3, -560.3),
    // 4430 m, -32 m.
    Vec2::new(364.4, -582.1),
    // 4452 m, -48 m.
    Vec2::new(331.4, -593.3),
    // 4470 m, -60 m.
    Vec2::new(298.4, -590.7),
    // 4504 m, -65 m.
    Vec2::new(265.4, -576.7),
    // 4584 m, -57 m.
    Vec2::new(232.5, -557.3),
    // 4618 m, -29 m.
    Vec2::new(204.3, -538.2),
    // 4638 m, -11 m.
    Vec2::new(176.7, -528.3),
    // 4658 m, -2 m.
    Vec2::new(153.9, -524.4),
];

/// How much higher than the lap beside it the way to the left of the temple
/// runs: over a hump where it is far from the lap, broad at its top (m along
/// it, m).
const TEMPLE_LEFT_RISE: [(f32, f32); 35] = [
    (106.3, 0.00),
    (112.3, 0.01),
    (118.3, 0.06),
    (124.3, 0.22),
    (130.3, 0.51),
    (136.3, 0.87),
    (142.3, 1.26),
    (148.3, 1.65),
    (154.3, 2.04),
    (160.3, 2.43),
    (166.3, 2.82),
    (172.3, 3.20),
    (178.3, 3.58),
    (184.3, 3.91),
    (190.3, 4.17),
    (196.3, 4.36),
    (202.3, 4.47),
    (208.3, 4.50),
    (214.3, 4.45),
    (220.3, 4.32),
    (226.3, 4.11),
    (232.3, 3.83),
    (238.3, 3.48),
    (244.3, 3.11),
    (250.3, 2.72),
    (256.3, 2.33),
    (262.3, 1.94),
    (268.3, 1.55),
    (274.3, 1.16),
    (280.3, 0.77),
    (286.3, 0.42),
    (292.3, 0.16),
    (298.3, 0.04),
    (304.3, 0.01),
    (310.3, 0.00),
];

/// How much higher than the lap beside it the way to the left on the castle
/// ridge runs: over a hump where it is far from the lap (m along it, m).
const CASTLE_LEFT_RISE: [(f32, f32); 35] = [
    (55.9, 0.00),
    (61.9, 0.01),
    (67.9, 0.06),
    (73.9, 0.22),
    (79.9, 0.51),
    (85.9, 0.87),
    (91.9, 1.26),
    (97.9, 1.65),
    (103.9, 2.04),
    (109.9, 2.43),
    (115.9, 2.82),
    (121.9, 3.21),
    (127.9, 3.58),
    (133.9, 3.92),
    (139.9, 4.19),
    (145.9, 4.38),
    (151.9, 4.50),
    (157.9, 4.54),
    (163.9, 4.50),
    (169.9, 4.37),
    (175.9, 4.17),
    (181.9, 3.90),
    (187.9, 3.56),
    (193.9, 3.18),
    (199.9, 2.80),
    (205.9, 2.41),
    (211.9, 2.02),
    (217.9, 1.63),
    (223.9, 1.24),
    (229.9, 0.85),
    (235.9, 0.49),
    (241.9, 0.21),
    (247.9, 0.06),
    (253.9, 0.01),
    (259.9, 0.00),
];

/// How much higher than the lap beside it the way to the right on the castle
/// ridge runs: through a dip where it is far from the lap (m along it, m).
const CASTLE_RIGHT_RISE: [(f32, f32); 33] = [
    (112.3, 0.00),
    (118.3, -0.01),
    (124.3, -0.06),
    (130.3, -0.14),
    (136.3, -0.29),
    (142.3, -0.49),
    (148.3, -0.75),
    (154.3, -1.06),
    (160.3, -1.42),
    (166.3, -1.80),
    (172.3, -2.19),
    (178.3, -2.58),
    (184.3, -2.97),
    (190.3, -3.36),
    (196.3, -3.71),
    (202.3, -3.97),
    (208.3, -4.05),
    (214.3, -3.92),
    (220.3, -3.63),
    (226.3, -3.26),
    (232.3, -2.87),
    (238.3, -2.48),
    (244.3, -2.09),
    (250.3, -1.70),
    (256.3, -1.32),
    (262.3, -0.98),
    (268.3, -0.68),
    (274.3, -0.43),
    (280.3, -0.24),
    (286.3, -0.12),
    (292.3, -0.04),
    (298.3, -0.01),
    (304.3, 0.00),
];

/// The island, clockwise from the north-west: the north coast along the
/// coast road, the inlet the boardwalk crosses, the east coast beyond the
/// susuki hills, the south coast round the lake and the castle town, and
/// the west coast beyond the bamboo.
const SHORE: [Vec2; 49] = [
    Vec2::new(-280.0, -660.0),
    Vec2::new(-190.0, -765.0),
    Vec2::new(-80.0, -850.0),
    Vec2::new(20.0, -870.0),
    Vec2::new(110.0, -862.0),
    Vec2::new(250.0, -848.0),
    Vec2::new(390.0, -872.0),
    Vec2::new(470.0, -870.0),
    Vec2::new(560.0, -856.0),
    Vec2::new(630.0, -856.0),
    // The inlet, in under the boardwalk and a little way past it.
    Vec2::new(664.0, -818.0),
    Vec2::new(680.0, -775.0),
    Vec2::new(702.0, -738.0),
    Vec2::new(760.0, -722.0),
    Vec2::new(818.0, -738.0),
    Vec2::new(840.0, -775.0),
    Vec2::new(856.0, -818.0),
    Vec2::new(900.0, -858.0),
    Vec2::new(980.0, -878.0),
    Vec2::new(1070.0, -858.0),
    Vec2::new(1135.0, -800.0),
    Vec2::new(1172.0, -700.0),
    Vec2::new(1195.0, -585.0),
    Vec2::new(1208.0, -450.0),
    Vec2::new(1205.0, -300.0),
    Vec2::new(1212.0, -150.0),
    Vec2::new(1216.0, 0.0),
    Vec2::new(1212.0, 150.0),
    Vec2::new(1200.0, 300.0),
    Vec2::new(1170.0, 435.0),
    Vec2::new(1110.0, 545.0),
    Vec2::new(1010.0, 612.0),
    Vec2::new(880.0, 640.0),
    Vec2::new(750.0, 622.0),
    Vec2::new(640.0, 562.0),
    Vec2::new(540.0, 470.0),
    Vec2::new(410.0, 370.0),
    Vec2::new(260.0, 325.0),
    Vec2::new(110.0, 312.0),
    Vec2::new(-40.0, 302.0),
    Vec2::new(-170.0, 282.0),
    Vec2::new(-275.0, 215.0),
    Vec2::new(-330.0, 110.0),
    Vec2::new(-350.0, -40.0),
    Vec2::new(-352.0, -220.0),
    Vec2::new(-345.0, -400.0),
    Vec2::new(-325.0, -550.0),
    Vec2::new(-310.0, -610.0),
    Vec2::new(-298.0, -635.0),
];

/// The lake the road goes round: its middle, the radius of its shore (m)
/// and how far the shore wanders (m); and the islet in it, with a wayside
/// shrine.
const LAKE: Vec2 = Vec2::new(870.0, 270.0);
const LAKE_RADIUS: f32 = 150.0;
const LAKE_WANDER: f32 = 8.0;
const ISLET: Vec2 = Vec2::new(885.0, 255.0);
const ISLET_RADIUS: f32 = 16.0;
/// The clearing of red leaves under the great maple: beside the road round
/// the foot of the lake, 35 m from its centre line on the outside of the
/// bend, away from the water.
const MAPLE_GROUND: Vec2 = Vec2::new(875.4, 479.9);
/// The middle of the road beside it (3086 m).
const MAPLE_ROAD: Vec2 = Vec2::new(874.8, 444.9);
/// The shrine on the outside of the bend in the bamboo at 5180 m, 28 m from
/// the road: ahead of the riders coming down to the bend.
const BAMBOO_SHRINE: Vec2 = Vec2::new(-98.9, -305.5);
/// Where the pagoda stands before the corridor of torii (34 m to the right
/// of the road at 3780 m), for its terrace.
const PAGODA_GROUND: Vec2 = Vec2::new(727.9, -126.5);
/// The islet the great torii stands on, in the sea 56 m from the coast
/// road (at about 1240 m).
const TORII_ISLET: Vec2 = Vec2::new(497.0, -892.0);

/// How far `point` is from the lake's shore: negative in its water, and
/// positive on the islet.
fn lakes(point: Vec2) -> f32 {
    let wander = LAKE_WANDER * (2.0 * fbm(point / 50.0 + 11.0, 2) - 1.0);
    let lake = point.distance(LAKE) - LAKE_RADIUS + wander;
    let islet = ISLET_RADIUS - point.distance(ISLET);
    lake.max(islet)
}

/// The castle hill the castle street runs into, by a tunnel, and the lap
/// comes back over: its middle, its radius and its height (m).
const CASTLE_HILL: Massif = climbed_massif(0.0, -530.0, 210.0, 34.0);

/// The hills, each about as high as the road over its top, so that the road
/// runs over them and along their flanks, never sunk between banks cut on
/// both sides.
const MASSIFS: [Massif; 12] = [
    CASTLE_HILL,
    // The castle ridge, east of it, and the red ridge, east of that, which
    // the road climbs from the temple.
    climbed_massif(300.0, -530.0, 230.0, 42.0),
    climbed_massif(600.0, -460.0, 260.0, 48.0),
    // The susuki hills.
    climbed_massif(1040.0, -330.0, 300.0, 36.0),
    climbed_massif(1050.0, 20.0, 240.0, 34.0),
    // The hill of the great maple, rising from the road round the foot of
    // the lake to the island's south shore.
    climbed_massif(880.0, 560.0, 140.0, 10.0),
    // The bamboo hill west of the castle, which the road climbs and winds
    // down, and its lower slopes south of it.
    climbed_massif(-170.0, -430.0, 190.0, 46.0),
    climbed_massif(-200.0, -230.0, 150.0, 22.0),
    // Crags on the red ridge, away from the road.
    massif(520.0, -330.0, 40.0, 22.0),
    massif(800.0, -420.0, 45.0, 24.0),
    // Stacks of rock standing in the sea off the coast road, and the islet
    // of the great torii.
    massif(300.0, -950.0, 28.0, 16.0),
    massif(TORII_ISLET.x, TORII_ISLET.y, 20.0, 10.0),
];

const CUTTINGS: [Cutting; 2] = [
    // The rock over the tunnel under the castle, from up the hill's slope,
    // and the rock the road is cut through out of its mouth by the sea (the
    // other opens at the foot of the hill, under its slope, not in a wall of
    // rock).
    Cutting {
        span: span(TUNNEL.from + 60.0, TUNNEL.to + 110.0),
        side: Side::Left,
        height: 16.0,
    },
    Cutting {
        span: span(TUNNEL.from + 60.0, TUNNEL.to + 110.0),
        side: Side::Right,
        height: 16.0,
    },
];

/// The volcano across the strait to the north, as Mount Yōtei over Hokkaido
/// or Fuji over its lakes: its middle, its radius and its height (m). Its
/// foot rises out of the sea just behind the great torii, and from the
/// castle street it stands over the castle; its top is under snow (see
/// `SNOW_LINE` in `crate::world::terrain`), the sky clear enough to see it
/// sharp from across the island.
const VOLCANO: Vec2 = Vec2::new(430.0, -2750.0);
const VOLCANO_RADIUS: f32 = 1650.0;
const VOLCANO_HEIGHT: f32 = 900.0;

/// Across the sea: the volcano to the north, a lone cone with its flanks
/// sweeping up ever steeper to a crater at its top, and lower mountains
/// round the rest of the horizon, clear of it.
fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(450.0, -150.0);
    let reach = point.distance(VOLCANO) / VOLCANO_RADIUS;
    let ranges = smoothstep(1900.0, 2800.0, point.distance(CENTER))
        * smoothstep(1.1, 1.7, reach)
        * 220.0
        * (0.35 + 0.65 * fbm(point / 300.0, 4));
    let cone = VOLCANO_HEIGHT * (1.0 - reach).max(0.0).powf(1.7);
    let crater = 40.0 * (1.0 - smoothstep(0.0, 0.08, reach));
    ranges.max(cone - crater + volcano_relief(point, reach))
}

/// The volcano's flanks as erosion and its own fires have left them (m over
/// or under its smooth cone, `reach` its share of the way out from its
/// middle): gullies carved down them, rounded at their bottoms, between
/// ridges that are the cone itself (so its skyline stays smooth), deepest
/// half way down and none at the rim nor at the foot, each wandering a
/// little as it runs down; broad lumps and hollows; and a parasitic cone on
/// the flank facing the island.
fn volcano_relief(point: Vec2, reach: f32) -> f32 {
    if reach >= 1.0 {
        return 0.0;
    }
    let angle = (point - VOLCANO).to_angle();
    let mut gullies = 0.0;
    // Each set of gullies only where they are far enough apart for the
    // land's grid (about 70 m there) to draw them.
    for (k, (count, weight, from)) in [(13.0, 1.0, 0.18), (21.0, 0.6, 0.28), (34.0, 0.35, 0.46)]
        .into_iter()
        .enumerate()
    {
        let wander = 2.5 * (fbm(Vec2::new(reach * 5.0, k as f32 * 3.7), 2) - 0.5);
        let wave = (count * angle + wander + k as f32 * 1.3).sin();
        gullies += weight * wave.max(0.0).powi(2) * smoothstep(from, from + 0.1, reach);
    }
    let down = 1.0 - smoothstep(0.75, 1.0, reach);
    let lumps = 60.0 * (fbm(point / 420.0, 3) - 0.5) * smoothstep(0.1, 0.3, reach) * down;
    let parasite = VOLCANO + Vec2::new(0.5, 0.86) * 0.55 * VOLCANO_RADIUS;
    let near = point.distance(parasite) / 230.0;
    let cone =
        95.0 * (1.0 - near * near).max(0.0).powi(2) - 18.0 * (1.0 - smoothstep(0.0, 0.18, near));
    lumps + cone - 45.0 * gullies * down
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    /// From the end of the boardwalk over the susuki hills to the lake.
    const SUSUKI_HILLS: Span = span(1622.0, 2700.0);

    #[test]
    fn the_lap_goes_through_the_castle_hill_and_then_over_it() {
        let track = Track::new(&GHOST_ISLAND.layout);
        let line = track.centerline();
        // Through it, by its middle.
        let middle = line[track.index_at((TUNNEL.from + TUNNEL.to) / 2.0)];
        assert!(middle.distance(CASTLE_HILL.center) < 0.2 * CASTLE_HILL.radius);
        // Over it at the end of the lap, well above the tunnel.
        let under = track.index_at(TUNNEL.from)..track.index_at(TUNNEL.to);
        let (gap, over, under) = (track.index_at(CROSSING.from)..track.index_at(CROSSING.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the crossing and the tunnel have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 25.0, "{above} m above");
    }

    #[test]
    fn the_hills_are_steep_and_the_road_climbs_and_drops_a_lot() {
        let track = Track::new(&GHOST_ISLAND.layout);
        let step = track.step_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let grade = |along: f32| {
            let index = track.index_at(along);
            (track.elevations()[index + 1] - track.elevations()[index]) / step
        };
        let steepest = |stretch: Span| {
            (track.index_at(stretch.from)..track.index_at(stretch.to))
                .map(|index| grade(index as f32 * step).abs())
                .fold(0.0, f32::max)
        };
        // Up the susuki hills, and down them to the lake, at nearly 12 %.
        assert!(height(2300.0) - height(SUSUKI_HILLS.from) > 30.0);
        assert!(steepest(span(1700.0, 2300.0)) > 0.115);
        assert!(steepest(span(2600.0, 2900.0)) > 0.115);
        // Up the red ridge to its top, 30 m and more, and down from the
        // castle to town.
        let top = (track.index_at(RED_RIDGE.from)..track.index_at(RED_RIDGE.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(top - height(RED_RIDGE.from) > 30.0, "{top} m at the top");
        assert!(steepest(RED_RIDGE) > 0.115);
        assert!(height(CROSSING.from) - height(0.0) > 35.0);
        // Up the bamboo hill, and down it, 40 m and more, at 11 % most of
        // the way.
        let hill = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(hill - height(CROSSING.to) > 12.0, "{hill} m at the top");
        assert!(hill - height(BAMBOO.to) > 40.0, "{hill} m at the top");
        assert!(steepest(span(CROSSING.to, CROSSING.to + 150.0)) > 0.115);
        let down = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .filter(|&index| grade(index as f32 * step) < -0.105)
            .count() as f32
            * step;
        assert!(down > 200.0, "{down} m at 11 %");
    }

    #[test]
    fn the_road_winds_down_the_bamboo_hill_in_tight_esses() {
        // Four tight bends and more, one after the other, each the other
        // way.
        let track = Track::new(&GHOST_ISLAND.layout);
        let bends: Vec<f32> = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| track.curvature(index))
            .filter(|curvature| curvature.abs() > 1.0 / 50.0)
            .fold(Vec::new(), |mut bends, curvature| {
                if bends
                    .last()
                    .is_none_or(|last: &f32| last.signum() != curvature.signum())
                {
                    bends.push(curvature);
                }
                bends
            });
        assert!(bends.len() >= 5, "{} bends", bends.len());
        let tightest = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| 1.0 / track.curvature(index).abs())
            .fold(f32::MAX, f32::min);
        assert!(tightest < 42.0, "{tightest} m");
    }

    #[test]
    fn the_road_rounds_a_lake_with_an_islet_and_crosses_an_inlet_on_the_boardwalk() {
        let track = Track::new(&GHOST_ISLAND.layout);
        let lap = track.lap_length();
        // Three sides of the lake, close by its shore all the way, with the
        // lake on the right, and low, just over the water.
        let mut round = 0.0;
        for index in 0..track.sample_count() {
            let point = track.centerline()[index];
            let shore = lakes(point);
            assert!(shore > 15.0, "{shore} m from the lake at {index}");
            let along = index as f32 * track.step_length();
            if span(2950.0, 3250.0).contains(along, lap) {
                let height = track.elevations()[index];
                assert!(height < 6.0, "{height} m up by the lake at {along}");
            }
            if LAKE_SHORE.contains(along, lap) && shore < 40.0 {
                round += track.step_length();
                let lake_side = (LAKE - point).dot(track.offset_point(index, 1.0) - point);
                assert!(lake_side < 0.0, "the lake on the left at {index}");
            }
        }
        assert!(round > 450.0, "{round} m round the lake");
        assert!(lakes(LAKE + Vec2::new(-40.0, 30.0)) < -20.0, "deep water");
        assert!(lakes(ISLET) > 10.0, "an islet");
        // The boardwalk crosses the inlet; the water reaches in under it.
        let middle = track.centerline()[track.index_at((BOARDWALK.from + BOARDWALK.to) / 2.0)];
        let shore = crate::track::closed_curve(&SHORE, 4.0);
        let inside = |point: Vec2| {
            let n = shore.len();
            (0..n).fold(false, |inside, i| {
                let (a, b) = (shore[i], shore[(i + 1) % n]);
                let crosses = (a.y > point.y) != (b.y > point.y)
                    && point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
                inside != crosses
            })
        };
        assert!(!inside(middle), "the inlet is under the boardwalk");
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.from - 30.0)]
        ));
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.to + 30.0)]
        ));
    }

    #[test]
    fn the_volcanos_flanks_are_carved_into_gullies_and_ridges() {
        // Half way down, round it: ridges and gullies one after another, tens
        // of metres apart in height, not a smooth cone.
        let heights: Vec<f32> = (0..720)
            .map(|k| {
                let away = Vec2::from_angle(k as f32 / 720.0 * std::f32::consts::TAU);
                mountains(VOLCANO + away * 0.5 * VOLCANO_RADIUS)
            })
            .collect();
        let (low, high) = heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &y| (l.min(y), h.max(y)));
        assert!(high - low > 50.0, "{} m from gully to ridge", high - low);
        let turns = heights
            .windows(3)
            .filter(|w| (w[1] - w[0]) * (w[2] - w[1]) < 0.0)
            .count();
        assert!(turns > 40, "{turns} ridges and gullies round it");
        // None at the rim, nor past the foot.
        let rim = mountains(VOLCANO + Vec2::X * 0.1 * VOLCANO_RADIUS);
        let smooth = VOLCANO_HEIGHT * 0.9f32.powf(1.7) - 40.0 * (1.0 - smoothstep(0.0, 0.08, 0.1));
        assert!((rim - smooth).abs() < 1.0, "{rim} at the rim");
    }

    #[test]
    fn the_volcano_rises_across_the_strait_and_nothing_is_raised_by_the_road() {
        let track = Track::new(&GHOST_ISLAND.layout);
        assert!(mountains(VOLCANO) > 0.85 * VOLCANO_HEIGHT);
        assert!(mountains(VOLCANO) < VOLCANO_HEIGHT, "a crater at its top");
        // Under snow at its top, with its rim, and alone: no mountain
        // beside it reaches a third of its height.
        let rim = VOLCANO + Vec2::X * 0.1 * VOLCANO_RADIUS;
        assert!(mountains(rim) > 600.0, "{}", mountains(rim));
        for k in 0..36 {
            let away = Vec2::from_angle(k as f32 / 36.0 * std::f32::consts::TAU);
            let beside = VOLCANO + away * 1.3 * VOLCANO_RADIUS;
            assert!(mountains(beside) < VOLCANO_HEIGHT / 3.0, "{beside}");
        }
        // Straight ahead, over the castle, down the castle street.
        let street = track.centerline()[track.index_at(100.0)];
        let ahead = track.centerline()[track.index_at(300.0)] - street;
        let toward = VOLCANO - street;
        assert!(
            ahead.angle_to(toward).abs() < 0.3,
            "{}",
            ahead.angle_to(toward)
        );
        for &point in track.centerline() {
            assert!(mountains(point) < 1e-3, "{point}");
        }
    }
}
