//! Cómo es el escenario de cada pista: el terreno (costa, colinas, peñascos y
//! paredes de roca), lo que bordea la carretera (muros, túneles, puentes,
//! arcos y carteles) y el decorado (árboles, casas, hoteles y palmeras).
//! Los datos de cada pista están en [`crate::circuits`].

use bevy::prelude::*;
use serde::Deserialize;

use crate::geometry::smoothstep;

/// Un tramo de la vuelta, en metros desde la meta. Puede pasar por la meta:
/// de 2400 a 300, por ejemplo.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(from = "crate::circuits::data::SpanFile")]
pub struct Span {
    pub from: f32,
    pub to: f32,
}

pub const fn span(from: f32, to: f32) -> Span {
    Span { from, to }
}

/// Toda la vuelta.
pub const LAP: Span = span(0.0, f32::MAX);

impl Span {
    /// Cuánto se está dentro del tramo en el punto `along` de una vuelta de
    /// `lap` metros: 1 dentro, 0 fuera, y de uno a otro en los primeros y
    /// últimos `fade` metros.
    pub fn weight(&self, along: f32, lap: f32, fade: f32) -> f32 {
        if self.to - self.from >= lap {
            return 1.0;
        }
        let length = (self.to - self.from).rem_euclid(lap);
        let into = (along - self.from).rem_euclid(lap);
        if into > length {
            return 0.0;
        }
        if fade <= 0.0 {
            return 1.0;
        }
        smoothstep(0.0, fade, into) * smoothstep(0.0, fade, length - into)
    }

    /// Whether it and `other` share any of a lap `lap` metres long.
    pub fn overlaps(&self, other: &Span, lap: f32) -> bool {
        let middle = |span: &Span| span.from + span.length(lap) / 2.0;
        self.contains(other.from, lap)
            || other.contains(self.from, lap)
            || self.contains(middle(other), lap)
    }

    pub fn contains(&self, along: f32, lap: f32) -> bool {
        self.weight(along, lap, 0.0) > 0.0
    }

    /// Lo que mide el tramo en una vuelta de `lap` metros.
    pub fn length(&self, lap: f32) -> f32 {
        if self.to - self.from >= lap {
            lap
        } else {
            (self.to - self.from).rem_euclid(lap)
        }
    }
}

/// A qué lado de la carretera, mirando hacia donde se corre.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// El signo del desplazamiento lateral de ese lado (positivo a la izquierda).
    pub fn sign(self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }
}

/// Un peñasco: una masa de roca con forma irregular.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Massif {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
    /// Whether the road climbs it: its smooth slopes are the ground the road
    /// is cut into, rather than rock standing up behind the walls.
    #[serde(default)]
    pub climbed: bool,
}

pub const fn massif(x: f32, z: f32, radius: f32, height: f32) -> Massif {
    Massif {
        center: Vec2::new(x, z),
        radius,
        height,
        climbed: false,
    }
}

/// A mountain the road climbs (see [`Massif::climbed`]).
pub const fn climbed_massif(x: f32, z: f32, radius: f32, height: f32) -> Massif {
    Massif {
        climbed: true,
        ..massif(x, z, radius, height)
    }
}

/// Una pared de roca a un lado de la carretera, como si la hubieran abierto
/// cortando la montaña.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cutting {
    pub span: Span,
    pub side: Side,
    pub height: f32,
}

/// The greens of a track's grass.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Greens {
    /// The arcade lawn: a bright green, a little lighter or deeper here and
    /// there.
    Lawn,
    /// A countryside's: swathes of many greens, lusher by the water, drier
    /// up high and towards the sun, and dark conifer forest up the hills,
    /// far from the roads and on the mountains across the sea.
    Countryside,
    /// A Mediterranean island's in summer: straw-coloured dry grass, olive
    /// and grey-green sage, fresher green down the hollows and by the
    /// water, and dark scrub up the slopes before the rock, in patches of
    /// every size with ragged edges.
    Mediterranean,
    /// A lush green land's, as the Shire's: the countryside's, but deeper
    /// and greener, not yellowed by the sun, with darker drifts where the
    /// grass grows thick.
    Lush,
}

/// Una explanada: el terreno, llano a una altura (m sobre el mar) en un
/// círculo, para poner encima un edificio grande; or sloping, facing one
/// way.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrace {
    pub center: Vec2,
    pub radius: f32,
    /// How high its middle stands.
    pub height: f32,
    /// How much higher it stands for each metre along x and along z: zero
    /// for level ground.
    #[serde(default)]
    pub tilt: Vec2,
    /// How far past its edge the ground blends back into the land (m):
    /// wider where the land round it stands far above or below it, so the
    /// bank stays grass instead of breaking into a cliff.
    pub skirt: f32,
}

/// Dónde está el mar.
#[derive(Deserialize)]
#[serde(try_from = "crate::circuits::data::CoastFile")]
pub enum Coast {
    /// Distancia con signo a la costa, positiva tierra adentro.
    Function(fn(Vec2) -> f32),
    /// Una isla: tierra dentro del circuito y, por fuera, hasta tantos metros
    /// de la línea central según el punto de la vuelta (pares de metros desde
    /// la meta y distancia; entre ellos se interpola). Con una distancia
    /// negativa el mar entra por debajo de la carretera: una ría.
    AroundTrack(&'static [(f32, f32)]),
    /// An island whose shore is a smooth closed curve through these points
    /// (x, z), drawn like the centre line of the track: the land is inside
    /// it, with its own bays, headlands and inlets. As with `AroundTrack`,
    /// the land drops to the sea right beyond the walls where the shore is
    /// near; an inlet drawn under a bridge brings the sea in under it.
    Outline(&'static [Vec2]),
    /// An island drawn as with `Outline`, with lakes inside it: the function
    /// gives the signed distance to their shores, negative in the water, as
    /// `Function` does to the sea's. The land drops to them right beyond the
    /// walls where their shores are near, as it does to the sea.
    OutlineWithLakes(&'static [Vec2], fn(Vec2) -> f32),
    /// Inland, with no sea: the function gives the signed distance to the
    /// shores of the rivers and lakes, negative in the water, as `Function`
    /// does; but, as round an island's lakes, the land drops to them right
    /// beyond the walls where their shores are near.
    Inland(fn(Vec2) -> f32),
}

/// Cómo se cierra la carretera por los lados.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum WallStyle {
    /// Muro de piedra.
    Stone,
    /// Barrera de hormigón blanca.
    Concrete,
    /// Barandilla verde de paseo marítimo.
    Railing,
    /// Seto sobre un murete.
    Hedge,
    /// Blocks of brown checkerboard earth, as in Loop Island.
    Checkered,
    /// A fence of heavy timber planks, as round a mine.
    Timber,
    /// A low vermilion fence of posts and rails, as round a shrine: like
    /// the railing, the ground beyond it shows through.
    ShrineFence,
    /// A low fence of weathered wooden posts and rails, as along the lanes
    /// of the Shire: the ground beyond it shows through.
    Fence,
    /// A low concrete parapet, as along a dam's crest on its lake's side:
    /// the water shows over it, a little way down.
    Parapet,
    /// The low border of a slot-car track, in red and white stripes.
    SlotBorder,
    /// The raised orange edge of Hot Wheels' track, with a little light
    /// every few metres along its top.
    OrangeEdge,
    /// The tall, rounded side of a slide's chute, of red plastic.
    ChuteSide,
    /// A row of wooden toy blocks, each painted a bright colour of its own
    /// with a letter on its face.
    Blocks,
    /// A low wall of plastic building bricks in bright colours, a course of
    /// bricks and a course of plates on it in a running bond, studs on top
    /// (see `world::bricks`).
    Bricks,
}

/// What the steep faces of the terrain (cliffs, rock walls and the rock
/// around the tunnels) are made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Soil {
    /// Natural rock with tilted strata.
    Rock,
    /// Brown checkerboard earth under a grass top, as in Loop Island.
    Checkered,
    /// A desert canyon of red and ochre rock over dry, dusty earth; its
    /// tunnels are mine galleries propped with timber and lit by lanterns.
    Mine,
    /// Grey granite and schist, bare and hard, as the Argonath are carved
    /// from: the rock of the high country of *The Lord of the Rings*.
    Granite,
    /// Grey lunar regolith and basaltic crater rock; its tunnels are
    /// pressurized space-station corridors lit by cool white-cyan strips.
    Lunar,
    /// The floor of a room, indoors: flat boards, with no rock, sand nor
    /// grass; its tunnels are what stands over the road (a table, another
    /// stretch of the road), which draw nothing of their own (see
    /// [`ToyStore`]).
    Floor,
}

/// The light of a track: a summer day; a clear Mediterranean summer
/// afternoon, graded as a photograph is; the last hours of a clear autumn
/// afternoon, the sun low and golden; a low sun setting in a warm, hazy
/// sky; or the black sky of the Moon with the Earth hanging above the
/// horizon.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum SkyStyle {
    Day,
    Mediterranean,
    GoldenHour,
    Dusk,
    /// A summer sunset in high country, the hills' long shadows over one
    /// another, graded as a film is: the light of New Zealand, where *The
    /// Lord of the Rings* was shot.
    HighCountry,
    Space,
    /// Indoors, under a ceiling: no sky, no sun and no clouds, but the
    /// light of the lamps from overhead, white and warm, softly filling the
    /// shade, as in a shop (see [`ToyStore`]).
    Indoor,
}

/// The colour of the water: the blue sea, or the red water of a flooded pit
/// rich in iron, as at Rio Tinto.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Water {
    Sea,
    Iron,
    /// The rivers and lakes of high country: deep green and grey, clearer
    /// over the shallows.
    River,
}

/// An open-pit mine round the water (see [`Scenery::pit`]): benches cut in
/// steps from its rim down to the water, following the shore, and its spoil
/// heaped round the rim.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pit {
    /// How many benches, how high each face steps up (m) and how wide each
    /// bench is (m).
    pub benches: u32,
    pub step: f32,
    pub tread: f32,
    /// How far past the rim the spoil heaped round it slopes down to the
    /// ground (m).
    pub spoil: f32,
}

/// A stretch of the lap carried by a bridge, and how it is built.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bridge {
    pub span: Span,
    pub style: BridgeStyle,
    /// How high a wooden bridge's fence stands over its deck (m): most as
    /// high as [`WOODEN_FENCE`], a boardwalk low over the water lower. The
    /// other bridges have walls of their own, and leave it unused.
    #[serde(default)]
    pub fence: f32,
}

/// How high the fence of a wooden bridge usually stands over its deck (m).
pub const WOODEN_FENCE: f32 = 1.2;

impl BridgeStyle {
    /// Whether it is of wood, old or new, with its own fence.
    pub fn wooden(self) -> bool {
        matches!(self, BridgeStyle::Wooden | BridgeStyle::OldWooden)
    }
}

/// How a bridge is built.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum BridgeStyle {
    /// A concrete deck under two red steel arches.
    RedArches,
    /// A wooden deck of planks on log piers, with a wooden fence on each side.
    Wooden,
    /// The same, old and weathered: its boards silver-grey, patched darker
    /// and mossy, its posts leaning a little and its rails sagging.
    OldWooden,
    /// A concrete deck between two steel through trusses, braced across
    /// overhead, on concrete piers, as in West Way (Moto Racer).
    Truss,
    /// The crest of a dam: the concrete of the dam itself, down to the
    /// gorge below it, with the lake it holds on the other side (see
    /// [`Dam`], which says how).
    Dam,
    /// A slot-car track: its dark plastic with the slots and their rails,
    /// lying on the furniture under it, or carried over the floor on blue
    /// lattice pillars, between red and white borders (see [`ToyStore`]).
    SlotTrack,
    /// Hot Wheels' orange track, with its chevrons, on blue pillars, between
    /// its raised edges.
    OrangeTrack,
    /// A slide's chute of yellow plastic between tall red sides.
    Slide,
}

/// A dam across a gorge, as El Atazar's, with the road along its crest (a
/// bridge of [`BridgeStyle::Dam`] over the same span): an arch of concrete
/// bowed towards the lake it holds, its water up near the crest; on the
/// other side its face drops nearly sheer to the river, which runs on down
/// the gorge to the sea.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dam {
    /// Where the road runs along the crest.
    pub crest: Span,
    /// Which side of the road the lake is on.
    pub lake_side: Side,
    /// The lake's shore: a smooth closed curve through these points (x,
    /// z), drawn as a coast's is, round a middle from which every point of
    /// it can be seen (the water is a fan from there). Where it meets the
    /// dam it runs along the crest, so the water reaches the dam's face.
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub lake: &'static [Vec2],
    /// The height of the lake's water (m over the sea): a little under the
    /// crest.
    pub level: f32,
    /// The river below the dam: points (x, z) along its course, from
    /// under the dam (its first point past the crest, so the gorge runs on
    /// under it) down to the sea, each with how high the gorge's far
    /// wall stands there over the sea (m). Its water is the sea's, coming up
    /// the gorge to the foot of the dam.
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub river: &'static [(Vec2, f32)],
    /// Which bank of the river (going downstream) the road up the gorge
    /// runs along: on that side the bank only drops from the road to the
    /// water; across the river, the gorge's far wall rises.
    pub road_bank: Side,
    /// Where a paved square stands by the end of the crest with a
    /// monument in the middle, if one does (x, z): on level ground, as a
    /// [`Terrace`] makes it.
    #[serde(default)]
    pub monument: Option<Vec2>,
}

/// Un tramo con otro muro distinto del de piedra a un lado.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WallRun {
    pub span: Span,
    pub side: Side,
    pub style: WallStyle,
}

/// Un arco sobre la carretera con un rótulo.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gantry {
    pub along: f32,
    #[serde(deserialize_with = "crate::circuits::data::color")]
    pub color: Color,
    /// El rótulo, o un tablero de ajedrez si no hay.
    #[serde(deserialize_with = "crate::circuits::data::optional_text", default)]
    pub text: Option<&'static str>,
}

/// Un cartel publicitario junto a la carretera.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Billboard {
    pub along: f32,
    pub side: Side,
    #[serde(deserialize_with = "crate::circuits::data::text")]
    pub text: &'static str,
    #[serde(deserialize_with = "crate::circuits::data::color")]
    pub color: Color,
    /// Desde qué letra, y en qué color, van las demás.
    #[serde(deserialize_with = "crate::circuits::data::accent", default)]
    pub accent: Option<(usize, Color)>,
}

/// Una fila de algo a lo largo de la carretera: en un tramo, a un lado, a
/// esta distancia de la línea central y cada tantos metros.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
    pub every: f32,
}

/// Especies de árbol.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Species {
    UmbrellaPine,
    Cypress,
    Olive,
    /// Árbol alto de bosque, de copa alargada y oscura.
    Tall,
    /// Árbol de hoja ancha, de copa redonda y verde claro.
    Broadleaf,
    /// Maritime pine with tiered branches and coastal habit.
    MaritimePine,
    /// Slender poplar / aspen with upright crown and bright green foliage.
    Poplar,
    /// Almond / Mediterranean flowering tree with warm pink-tinted blossom.
    Almond,
    /// Japanese maple, crimson in autumn.
    Momiji,
    /// Ginkgo, golden yellow in autumn.
    Ginkgo,
    /// Japanese cherry in pale pink blossom.
    Sakura,
    /// Japanese black pine, windswept, its foliage in flat pads.
    BlackPine,
    /// Japanese cedar, tall and narrow.
    Sugi,
    /// A clump of tall bamboo.
    Bamboo,
}

/// A forest: a stretch with trees close together on both sides of the road,
/// of these species.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forest {
    pub span: Span,
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub species: &'static [Species],
    /// A tree every so many metres along the road, in each of its rows
    /// either side ([`FOREST_EVERY`], or more where it is more open).
    #[serde(default = "forest_every")]
    pub every: f32,
}

/// How far apart along the road a forest's trees are, in each row (m).
pub const FOREST_EVERY: f32 = 5.0;

fn forest_every() -> f32 {
    FOREST_EVERY
}

pub const fn forest(span: Span, species: &'static [Species]) -> Forest {
    Forest {
        span,
        species,
        every: FOREST_EVERY,
    }
}

impl Forest {
    /// The forest more open: a tree every `every` metres in each row.
    pub const fn open(self, every: f32) -> Self {
        Forest { every, ..self }
    }
}

/// Las casas: cuántas como mucho, de qué colores y, si hace falta, solo en
/// unos tramos.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Houses {
    pub max: usize,
    #[serde(deserialize_with = "crate::circuits::data::colors")]
    pub paints: &'static [Color],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub spans: &'static [Span],
    pub style: HouseStyle,
}

/// How the houses are built.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum HouseStyle {
    /// Plastered walls under a tiled gable roof.
    Cottage,
    /// The same, with a wooden false front standing up over the roof at the
    /// front and a porch along it, as in the towns of the Old West (and of
    /// West Way, in Moto Racer).
    FalseFront,
}

/// Todo lo que hace falta para construir el escenario de una pista.
#[derive(Deserialize)]
// Its gantries' and billboards' texts are `&'static`, so a file is read
// from a text as lasting (see `crate::circuits::data::parse`).
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'static"))]
pub struct Scenery {
    pub coast: Coast,
    /// Altura de las colinas (m).
    pub hills: f32,
    /// How the grass is coloured: one track's greens never change
    /// another's.
    pub greens: Greens,
    /// How light the grass is: 1 as painted, less for darker, less vivid
    /// greens (the grass in flower darkens half as much, the flowers being
    /// lighter; the hedges as much as the grass).
    pub grass_shade: f32,
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub massifs: &'static [Massif],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub cuttings: &'static [Cutting],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub terraces: &'static [Terrace],
    /// Montañas lejanas que cierran el horizonte.
    #[serde(deserialize_with = "crate::circuits::data::function")]
    pub mountains: fn(Vec2) -> f32,
    /// Colores de las franjas de los pianos.
    #[serde(deserialize_with = "crate::circuits::data::color_pair")]
    pub curbs: [Color; 2],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub walls: &'static [WallRun],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub tunnels: &'static [Span],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub bridges: &'static [Bridge],
    /// Dams, each with the road along its crest (see [`Dam`]).
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub dams: &'static [Dam],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub gantries: &'static [Gantry],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub billboards: &'static [Billboard],
    /// Farolas, cada `every` metros.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub lamps: &'static [Row],
    /// Especies de los árboles sueltos, y bosques: tramos con árboles altos
    /// muy juntos a los dos lados.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub trees: &'static [Species],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub forests: &'static [Forest],
    pub houses: Houses,
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub palms: &'static [Row],
    /// Hoteles altos en fila, de colores.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub hotels: &'static [Row],
    /// Un gran hotel de los de antes: punto de la vuelta y distancia a la
    /// carretera (con signo, positiva a la izquierda).
    #[serde(default)]
    pub grand_hotel: Option<(f32, f32)>,
    /// Boyas de rayas en el mar.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub buoys: &'static [Vec2],
    /// Faros costeros en islotes o cabos.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub lighthouses: &'static [Lighthouse],
    /// Veleros y yates fondeados en la bahía o el mar abierto.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub boats: &'static [Boat],
    /// Flechas indicadoras de curva (<<< y >>>) en curvas cerradas.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub chevrons: &'static [ChevronSign],
    /// Tribunas cubiertas con público frente a la pista.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub grandstands: &'static [Grandstand],
    /// Sombrillas y tumbonas de colores en las playas.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub beach_parasols: &'static [ParasolCluster],
    /// Carteles de aviso de frenada (100 m, 50 m).
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub brake_markers: &'static [BrakeMarker],
    /// What cliffs and rock walls are made of.
    pub soil: Soil,
    /// Rows of tall sunflowers turned towards the road.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub sunflowers: &'static [Row],
    /// Carved totem poles beside the road.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub totems: &'static [Totem],
    /// Giant vertical loops standing beside the road.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub loops: &'static [Loop],
    /// Lines of spinning golden rings floating over the road.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rings: &'static [Rings],
    /// Waterfalls pouring down the rock walls of the cuttings.
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub waterfalls: &'static [Waterfall],
    /// The light: day or dusk.
    pub sky: SkyStyle,
    /// The colour of the sea, the lakes and the rivers.
    pub water: Water,
    /// Whether the sea is always drawn as in *The Wind Waker*, flat cartoon
    /// blues with waves rolling in (Loop Island, an arcade island); on the
    /// other tracks it mirrors what stands round it unless Options >
    /// Graphics > Water says otherwise (see [`crate::world::sea`]).
    #[serde(default)]
    pub cartoon_water: bool,
    /// An open pit round the water, if there is one.
    #[serde(default)]
    pub pit: Option<Pit>,
    /// The works of a mine around the road, if there is one.
    #[serde(default)]
    pub mine: Option<Mine>,
    /// The roadside of a desert of the American West, if the road runs
    /// through one.
    #[serde(default)]
    pub desert: Option<Desert>,
    /// The colour of the halo the rear wheel leaves behind it on the road
    /// while the bike rides a wheelie, as Sonic's spin (Loop Island's), or
    /// `None` for none (see `crate::bike::wheelie_halo`).
    #[serde(deserialize_with = "crate::circuits::data::optional_color", default)]
    pub wheelie_halo: Option<Color>,
    /// The scenery of feudal Japan, from models made with Meshy, or `None`:
    /// see [`Japan`].
    #[serde(skip)]
    pub japan: Option<Japan>,
    /// The scenery of Middle-earth, from models made with Meshy, or `None`:
    /// see [`MiddleEarth`].
    #[serde(skip)]
    pub middle_earth: Option<MiddleEarth>,
    /// The works of a lunar base and orbital space station, if any: see
    /// [`Space`].
    #[serde(default)]
    pub space: Option<Space>,
    /// The inside of a toy store, if the road runs through one: see
    /// [`ToyStore`].
    #[serde(skip)]
    pub toys: Option<ToyStore>,
}

impl Scenery {
    /// The halls of the tunnels where they are excavations, a mine's or
    /// Moria's (see [`super::cavern`]), rather than tunnels lined with
    /// concrete panels.
    pub fn excavation(&self) -> Option<&'static [Hall]> {
        match (&self.mine, &self.middle_earth) {
            (Some(mine), _) if self.soil == Soil::Mine => Some(mine.halls),
            _ if self.soil == Soil::Mine => Some(&[]),
            (_, Some(middle_earth)) if !middle_earth.halls.is_empty() => Some(middle_earth.halls),
            _ => None,
        }
    }

    /// Where the ground stays grass however steep it is (see
    /// [`MiddleEarth::turf`]).
    pub fn turf(&self) -> &'static [Turf] {
        self.middle_earth
            .as_ref()
            .map_or(&[], |middle_earth| middle_earth.turf)
    }

    /// Where the granite is of another colour (see
    /// [`MiddleEarth::rock_tints`]).
    pub fn rock_tints(&self) -> &'static [RockTint] {
        self.middle_earth
            .as_ref()
            .map_or(&[], |middle_earth| middle_earth.rock_tints)
    }

    /// The fields of flowers, of feudal Japan or of Middle-earth.
    pub fn flower_fields(&self) -> &'static [FlowerField] {
        match (&self.japan, &self.middle_earth) {
            (Some(japan), _) => japan.flowers,
            (None, Some(middle_earth)) => middle_earth.flowers,
            (None, None) => &[],
        }
    }
}

/// The scenery of feudal Japan, as at Ghost Island (see
/// [`crate::world::japan`]): corridors of torii over the road, props made
/// with Meshy standing about, and fields of flowers.
pub struct Japan {
    pub corridors: &'static [ToriiCorridor],
    pub props: &'static [Prop],
    /// The small ones over and over along the road (see [`PropRow`]).
    pub rows: &'static [PropRow],
    pub maple_grounds: &'static [MapleGround],
    pub petals: &'static [Petals],
    pub flowers: &'static [FlowerField],
}

/// A field of flowers (see [`crate::world::flowers`]): the ground in
/// flower, of one kind or, in patches, of several, with gaps of grass among
/// them and a ragged edge, and, near the road, clumps standing up. Where
/// fields overlap, the first one decides.
pub struct FlowerField {
    pub area: FlowerArea,
    pub kinds: &'static [FlowerKind],
}

/// Where a field of flowers grows: beside a stretch of the lap, on both
/// sides, from `near` to `far` metres from its centre line; on the
/// shoulders of the roads along a stretch of the lap, between the asphalt
/// and the walls (in patches, with as much grass between them: bare earth
/// worn by the wheels, or fallen leaves), which the other fields keep off;
/// round a point, from `inner` to `outer` metres from it (a ring round a
/// lake, or, from 0, a round meadow, which nothing else is put in); or
/// everywhere else.
pub enum FlowerArea {
    Road {
        span: Span,
        near: f32,
        far: f32,
    },
    Shoulders {
        span: Span,
    },
    Round {
        center: Vec2,
        inner: f32,
        outer: f32,
    },
    Everywhere,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FlowerKind {
    /// Red spider lilies (higanbana): bare stems crowned with curling
    /// petals and long stamens.
    SpiderLily,
    /// White flowers with yellow hearts among their leaves.
    White,
    /// Yellow rape blossom (nanohana), in clusters on branching stems.
    Yellow,
    /// Purple irises among their sword-shaped leaves, as by the water.
    Iris,
    /// Susuki, the pampas grass of the autumn hills: arching blades under
    /// silvery gold plumes, taller than the others.
    Susuki,
    /// Shibazakura, moss phlox: a carpet of tiny flowers hugging the ground,
    /// in bands of magenta, pink, lavender and white, as round the lakes at
    /// the foot of Mount Fuji.
    Phlox,
    /// Not flowers, but the grass worn down to bare earth, brown and dusty,
    /// with tufts of grass and pebbles in it, as round a town or a wood.
    BareEarth,
    /// Not flowers, but leaves fallen over the grass, brown, rust, ochre,
    /// gold and red, as under the trees in autumn.
    FallenLeaves,
}

/// Petals, or leaves, drifting down through the air over an area, falling
/// and swaying, over and over.
pub struct Petals {
    pub area: PetalArea,
    pub kind: PetalKind,
}

/// Where petals drift: over a stretch of the lap, out to `reach` metres
/// either side of its centre line, or round a point.
pub enum PetalArea {
    Road { span: Span, reach: f32 },
    Round { center: Vec2, radius: f32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PetalKind {
    /// Red maple leaves.
    Maple,
    /// White cherry petals.
    Cherry,
    /// The golden leaves of the mallorns.
    Gold,
}

/// A clearing carpeted with red maple leaves, ringed with low logs, as
/// under a great maple by a lake: its middle (x, z) and the radius of its
/// ring of logs (m). Its leaves spill a little past the ring. Lay it on
/// level ground (a [`Terrace`]).
pub struct MapleGround {
    pub center: Vec2,
    pub radius: f32,
}

/// A corridor of torii over the road, as at Fushimi Inari: where the walls
/// close in on the road along `span` (see [`crate::track::Narrowing`]), a
/// narrow gate every `every` metres; and over the `approach` metres before
/// and after it, where the road is as wide as ever, a wide gate every
/// `approach_every` metres.
pub struct ToriiCorridor {
    pub span: Span,
    pub every: f32,
    pub approach: f32,
    pub approach_every: f32,
}

/// One of the props of feudal Japan, made with Meshy (see
/// [`crate::world::japan`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropModel {
    /// The great torii standing in the sea, as at Itsukushima.
    GreatTorii,
    CastleKeep,
    TempleHall,
    /// A shrine hall on stilts, over the water.
    StiltShrine,
    /// An ancient, gnarled maple on its boulders, with a straw rope round
    /// its trunk.
    AncientMaple,
    /// A great maple, its crown spreading far to one side over the ground,
    /// as over a clearing of its leaves.
    GreatMaple,
    /// A five-storey pagoda.
    Pagoda,
    /// A torii of bare logs, with a straw rope and bells.
    LogTorii,
    /// A tiny wayside shrine with its small torii.
    Hokora,
    /// A stone Jizō in a red bib and cap.
    Jizo,
    /// A stone guardian lion-dog.
    Komainu,
    BronzeLantern,
    /// A stone lantern (ishidōrō), mossy.
    StoneLantern,
    /// A memorial stone.
    Stele,
    /// A fire basket on a tripod.
    Brazier,
    /// Rice bales, sake barrels and crates.
    SupplyPile,
    FishingBoat,
    /// A thatched farmhouse.
    Minka,
    /// A wooden water wheel on its post, turning.
    WaterWheel,
}

impl PropModel {
    /// Whether it is one of the island's landmarks, to be seen from afar:
    /// no tree stands in the way of it (see [`crate::world::japan`]).
    pub fn landmark(self) -> bool {
        matches!(
            self,
            PropModel::GreatTorii
                | PropModel::CastleKeep
                | PropModel::TempleHall
                | PropModel::StiltShrine
                | PropModel::AncientMaple
                | PropModel::GreatMaple
                | PropModel::Pagoda
                | PropModel::LogTorii
                | PropModel::Minka
                | PropModel::WaterWheel
        )
    }

    /// What it stands on by the road, so that it shows over the wall:
    /// stone under the stone ones, wood under the wooden (see
    /// [`crate::world::japan`]). The big ones need none: they are seen
    /// over any wall as they are.
    pub fn plinth(self) -> Option<Plinth> {
        match self {
            PropModel::Jizo
            | PropModel::Komainu
            | PropModel::Stele
            | PropModel::StoneLantern
            | PropModel::BronzeLantern => Some(Plinth::Stone),
            PropModel::Brazier | PropModel::Hokora => Some(Plinth::Wood),
            _ => None,
        }
    }
}

/// What a small prop by the road stands on, so that it is seen over the
/// road's wall: a block of stone, or a deck of planks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Plinth {
    Stone,
    Wood,
}

/// A prop and where it stands, facing: by the road, `along` metres from the
/// finish line and `lateral` metres to the left of its centre line (to the
/// right if negative), facing the road; over the road, facing along it; or
/// anywhere, at `position` (x, z) facing `heading` (see
/// [`crate::geometry`]). It stands on the ground, or in the water, with its
/// foot a little under it; `scale` times as big as made (see
/// [`Prop::scaled`]), turned `turn` radians from that (see
/// [`Prop::turned`]), and leaning `lean` radians towards its side that was
/// -X as made (see [`Prop::leaning`]). Its model is one of feudal Japan's
/// ([`PropModel`]) or of Middle-earth's ([`MiddleEarthModel`]).
#[derive(Clone, Copy, Debug)]
pub struct Prop<M = PropModel> {
    pub model: M,
    pub spot: Spot,
    pub scale: f32,
    pub turn: f32,
    pub lean: f32,
    /// Whether it faces the riders coming up the road at it, instead of
    /// the road beside it (see [`Prop::facing_riders`]).
    pub facing_riders: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Spot {
    Road {
        along: f32,
        lateral: f32,
    },
    /// Over the middle of the road, as a gate the road runs through.
    Over {
        along: f32,
    },
    At {
        position: Vec2,
        heading: f32,
    },
}

/// A prop by the road (see [`Prop`]).
pub const fn by_road<M: Copy>(model: M, along: f32, lateral: f32) -> Prop<M> {
    Prop {
        model,
        spot: Spot::Road { along, lateral },
        scale: 1.0,
        turn: 0.0,
        lean: 0.0,
        facing_riders: false,
    }
}

/// A prop over the road, facing along it (see [`Prop`]).
pub const fn over_road<M: Copy>(model: M, along: f32) -> Prop<M> {
    Prop {
        model,
        spot: Spot::Over { along },
        scale: 1.0,
        turn: 0.0,
        lean: 0.0,
        facing_riders: false,
    }
}

impl<M: Copy> Prop<M> {
    /// The same prop, `scale` times as big.
    pub const fn scaled(self, scale: f32) -> Self {
        Prop { scale, ..self }
    }

    /// The same prop turned `turn` radians about its foot (anticlockwise
    /// seen from above): one by the road turned to face the riders coming
    /// along it.
    pub const fn turned(self, turn: f32) -> Self {
        Prop { turn, ..self }
    }

    /// The same prop leaning `lean` radians about its foot, towards its side
    /// that was -X as made: a tree leaning out over what its crown spreads
    /// over.
    pub const fn leaning(self, lean: f32) -> Self {
        Prop { lean, ..self }
    }

    /// The same prop turned to face the riders coming up the road at it,
    /// not the road beside it: a gate on a hilltop reads as a gate from
    /// far down the road, not as two posts edge on.
    pub const fn facing_riders(self) -> Self {
        Prop {
            facing_riders: true,
            ..self
        }
    }
}

/// The same small prop over and over by the road: one every so many
/// metres of a span, to one side, as the lanterns of a shrine's approach
/// or the jizō down a hillside road. Each stands as one of [`by_road`],
/// on its plinth, and the wall opens for it; where another road runs, or
/// where one of the props put by hand already stands, there is none.
pub struct PropRow {
    /// The kinds it is made of, each in turn: one kind over and over, or
    /// several, so that a stretch is not all of one thing.
    pub models: &'static [PropModel],
    pub row: Row,
}

/// A row of a small prop by the road (see [`PropRow`]).
pub const fn prop_row(
    models: &'static [PropModel],
    span: Span,
    side: Side,
    lateral: f32,
    every: f32,
) -> PropRow {
    PropRow {
        models,
        row: Row {
            span,
            side,
            lateral,
            every,
        },
    }
}

/// A prop anywhere (see [`Prop`]).
pub const fn standing<M: Copy>(model: M, x: f32, z: f32, heading: f32) -> Prop<M> {
    Prop {
        model,
        spot: Spot::At {
            position: Vec2::new(x, z),
            heading,
        },
        scale: 1.0,
        turn: 0.0,
        lean: 0.0,
        facing_riders: false,
    }
}

/// The scenery of Middle-earth, as at Fellowship Road (see
/// [`crate::world::middle_earth`]): props made with Meshy standing about,
/// hobbit holes in their grassy banks, fields of flowers and leaves
/// drifting down.
pub struct MiddleEarth {
    pub props: &'static [Prop<MiddleEarthModel>],
    pub flowers: &'static [FlowerField],
    pub petals: &'static [Petals],
    /// The halls of Moria: where there are, the tunnels are excavations, as
    /// a mine's, through them (see [`Scenery::excavation`]).
    pub halls: &'static [Hall],
    /// Footpaths of bare earth, as between the hobbit holes: each a line of
    /// points (x, z) over the ground.
    pub paths: &'static [&'static [Vec2]],
    /// Where the ground stays grass steeper than elsewhere, as the banks and
    /// the cut doorsteps of Hobbiton's hillsides are (but sheer), or the
    /// ledges of Rivendell's rock, its faces mossy.
    pub turf: &'static [Turf],
    /// Where the granite is of another colour, as the browner rock of the
    /// Anduin's gorge.
    pub rock_tints: &'static [RockTint],
    /// Lights that fill a stretch of land the low sun lights only from
    /// behind, as Rivendell's halls are, seen by the riders coming at them.
    pub glows: &'static [Glow],
    /// Arches of layered rock over the road, out of the rock beside it, as
    /// the sea's arches on a beach of cliffs.
    pub arches: &'static [RockArch],
}

/// An arch of rock over the road (see [`MiddleEarth::arches`]): its middle
/// `along` the lap, turned `turn` radians from square across it, its
/// opening `width` wide across the road at its foot and `height` high over
/// its middle, and `depth` thick along the road.
#[derive(Clone, Copy, Debug)]
pub struct RockArch {
    pub along: f32,
    pub turn: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

/// A light that fills what the low sun lights only from behind (see
/// [`MiddleEarth::glows`]): shining from `from` at `to`, `illuminance` lux
/// there, in `color`, in a cone `angle` radians round its axis, casting no
/// shadow, as the evening sky's light does. With a `shaft`, it comes down
/// from the sky onto one building, a sunbeam through the clouds, and its
/// shaft shows in the air.
pub struct Glow {
    pub from: Vec3,
    pub to: Vec3,
    pub illuminance: f32,
    pub color: Color,
    pub angle: f32,
    pub shaft: Option<Shaft>,
}

/// How a sunbeam's shaft shows in the air (see [`Glow::shaft`]): how many
/// times wider than at its foot it spreads at its top, and how bright it is
/// down its middle at its foot (linear, added over what is behind it, as
/// light is); the more it spreads and the fainter, the more diffused.
#[derive(Clone, Copy)]
pub struct Shaft {
    pub spread: f32,
    pub light: f32,
}

/// A round stretch of land whose steep slopes are grass, not rock (see
/// [`MiddleEarth::turf`]): grass up to `steepest`, the upward part of the
/// ground's normal under which it is rock (0.45: all but sheer faces; more,
/// grass on the ledges and gentler slopes of a rock face, rock between); and
/// its granite green with moss over much of it.
pub struct Turf {
    pub center: Vec2,
    pub radius: f32,
    pub steepest: f32,
}

/// A round stretch of land whose granite is of another colour (see
/// [`MiddleEarth::rock_tints`]): `color`, as light as the grey it stands
/// for, fading back into the grey over the outer quarter of `radius`.
pub struct RockTint {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

/// One of the props of Middle-earth, made with Meshy (see
/// [`crate::world::middle_earth`]), in the order of the film.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MiddleEarthModel {
    /// The front of a hobbit hole, with its round door (green, yellow, red
    /// or blue), set into a grassy bank, a chimney rising out of it.
    HobbitDoorGreen,
    HobbitDoorYellow,
    HobbitDoorRed,
    HobbitDoorBlue,
    /// Bag End: the grandest hobbit hole, its picket fence before its
    /// garden, set into its hill.
    BagEnd,
    /// The great pine of the party field.
    PartyTree,
    /// The gnarled oak on the hill over Bag End.
    BagEndOak,
    /// The mill of Bywater, its wheel turning.
    Mill,
    /// The Green Dragon of Bywater: a long, low inn under rounded thatch.
    GreenDragon,
    /// Its sign on its post.
    GreenDragonSign,
    /// Gandalf's cart, loaded with fireworks.
    GandalfCart,
    LogPile,
    Scarecrow,
    /// The old tree the hobbits hid under, its roots arched over a hollow.
    RootTree,
    /// The ferry raft over the Brandywine.
    Ferry,
    /// Bree's gate, over the road.
    BreeGate,
    /// The Prancing Pony: three storeys over an archway into its yard.
    PrancingPony,
    /// Houses of Bree: a wide one under thatch, a tall, narrow one.
    BreeHouseA,
    BreeHouseB,
    /// The ring of ruins of Amon Sûl on Weathertop, and a broken arch.
    WeathertopRuin,
    RuinArch,
    /// The three trolls turned to stone.
    StoneTrolls,
    /// The Last Homely House of Rivendell, an elven tower, the domed
    /// pavilion and the lamps of its paths.
    LastHomelyHouse,
    ElvenTower,
    Gazebo,
    ElvenLamp,
    /// The Doors of Durin, closed, in the Walls of Moria.
    DurinDoors,
    /// A mallorn of Lothlórien with its flet and its stair round its trunk.
    MallornFlet,
    /// Galadriel's swan ship, the sailing ship of the elves and an elven
    /// rowing boat.
    SwanShip,
    ElvenShip,
    ElvenBoat,
    /// The kings of the Argonath: the first crowned, the second helmed.
    ArgonathA,
    ArgonathB,
    /// The seat of seeing on Amon Hen.
    AmonHenSeat,
}

/// The roadside of a desert of the American West, as in West Way (Moto
/// Racer): gas stations, oil pumpjacks nodding on the dunes, highway
/// viaducts across the road, adobe pueblos, diamond warning signs before
/// the bends, cacti, and tumbleweeds rolling across the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Desert {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub gas_stations: &'static [GasStation],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub pumpjacks: &'static [Pumpjack],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub overpasses: &'static [Overpass],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub pueblos: &'static [Pueblo],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub curve_signs: &'static [CurveSign],
    pub cacti: Cacti,
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub tumbleweeds: &'static [Tumbleweed],
}

/// Saguaro cacti beside the road: about `count` of them, scattered along
/// `spans` either side, on fairly level ground clear of the roads.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cacti {
    pub count: usize,
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub spans: &'static [Span],
}

/// A tumbleweed blown across the road `along` the lap, rolling and bouncing
/// from `reach` metres out on one side to as far on the other, and again.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tumbleweed {
    pub along: f32,
    pub reach: f32,
}

/// A gas station beside the road, `lateral` metres from its centre line on
/// `side`, facing it: pumps under a canopy, a shop behind them and a tall
/// sign by the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GasStation {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// An oil pumpjack at `position` (x, z), its beam nodding towards
/// `heading` (see [`crate::geometry`]).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pumpjack {
    pub position: Vec2,
    pub heading: f32,
}

/// A concrete highway viaduct across the road `along` the lap, the bottom
/// of its deck `height` metres over the road, from one rock wall into the
/// other on columns beyond the walls (put a [`Cutting`] on both sides
/// first).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overpass {
    pub along: f32,
    pub height: f32,
}

/// An adobe pueblo beside the road, `lateral` metres from its centre line on
/// `side`: flat-roofed houses of earth, some of two floors, with beams
/// sticking out under their roofs, ladders and a dry tree.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pueblo {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// A diamond warning sign before a bend that way, high on a post on its
/// outside, `along` the lap.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveSign {
    pub along: f32,
    pub direction: ChevronDirection,
    pub color: SignColor,
}

/// The colour of a diamond warning sign: green with a white arrow for a
/// bend, yellow with a black one for a sharp one, red with a white one for
/// a hairpin.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
pub enum SignColor {
    Green,
    Yellow,
    Red,
}

/// The works of a mine: rails with ore carts rolling on them, timber
/// trestles carrying a rail over the road, headframes over the shafts,
/// lanterns on posts, piles of crates, barrels and ore, and the halls of
/// its excavation (its tunnels, see [`Scenery::tunnels`]).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mine {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rails: &'static [Rails],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub trestles: &'static [Trestle],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub headframes: &'static [Headframe],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub lanterns: &'static [Row],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub piles: &'static [Pile],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub halls: &'static [Hall],
}

/// A stretch of a mine's excavation, in metres along the lap (a branch off
/// the lap counts as the stretch it stands in for): the rock it is cut
/// through and how high its roof rises above the road in the middle (m).
/// Where there is none, the excavation is a gallery propped with timber.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hall {
    pub span: Span,
    pub stone: Stone,
    pub crown: f32,
    /// How much wider than the road between its walls the hall is, either
    /// side (m): where it is, the road runs through it between low walls of
    /// its stone (none in a mine).
    #[serde(default)]
    pub width: f32,
}

/// What the rock of a stretch of a mine's excavation is like, and the
/// ground beside its roads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Stone {
    /// A gallery propped with timber sets, with planks beside the road.
    Timbered,
    /// Red and cream sandstone in layers, over red gravel.
    Sandstone,
    /// Dark slate with veins of gold, over black gravel.
    Slate,
    /// Ochre clay, its walls lined with planks, over beaten earth.
    Clay,
    /// The dwarves' masonry, as in Moria: walls and vaults of dressed grey
    /// blocks over flagstones, great pillars in the widest halls.
    Masonry,
    /// A natural cave in grey granite, over grit.
    Cave,
}

/// A rail line beside the road over a span, `lateral` metres from its
/// centre line, with `carts` ore carts rolling up and down it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rails {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
    pub carts: u32,
}

/// A timber trestle carrying a rail across the road at `along`, from one
/// rock wall to the other, with an ore cart going over and back.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trestle {
    pub along: f32,
}

/// A headframe over a mine shaft: a timber tower with the winding wheels
/// turning on top, beside the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Headframe {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// What a pile beside the road is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum PileKind {
    /// Crates, barrels and a box of dynamite.
    Supplies,
    /// A heap of ore with gold glinting in it.
    Ore,
}

/// A pile of something beside the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pile {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub kind: PileKind,
}

/// A carved totem pole beside the road, facing it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Totem {
    pub along: f32,
    pub side: Side,
}

/// A giant vertical loop on a block of earth beside the road, in the plane
/// of the road: from the road it is seen side on, as in a 2D platformer.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loop {
    pub along: f32,
    pub side: Side,
    /// Distance from the centre line to the middle of the loop (m).
    pub lateral: f32,
    /// Radius of the running surface inside the loop (m).
    pub radius: f32,
}

/// A line of `count` golden rings over the road from `along` on, `lateral`
/// metres to the left of the centre line and `height` metres above the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rings {
    pub along: f32,
    pub count: u32,
    pub lateral: f32,
    pub height: f32,
}

/// A waterfall `width` metres wide down the rock wall of a cutting.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Waterfall {
    pub along: f32,
    pub side: Side,
    pub width: f32,
}

/// Un faro costero sobre un peñasco o cabo.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lighthouse {
    pub position: Vec2,
    pub height: f32,
}

/// Embarcaciones fondeadas en el mar.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum BoatKind {
    Sailboat,
    Yacht,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boat {
    pub position: Vec2,
    pub heading: f32,
    pub kind: BoatKind,
}

/// Dirección de la flecha de curva.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
pub enum ChevronDirection {
    Left,
    Right,
}

/// Cartel de curva con flechas reflectantes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChevronSign {
    pub along: f32,
    pub side: Side,
    pub direction: ChevronDirection,
}

/// Tribuna de espectadores cubierta.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grandstand {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub length: f32,
}

/// Grupo de sombrillas y tumbonas en la arena.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParasolCluster {
    pub position: Vec2,
    pub count: usize,
}

/// Cartel de distancia a curva (100 m, 50 m).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrakeMarker {
    pub along: f32,
    pub side: Side,
    pub distance: u32,
}

/// The works of a lunar base and Earth space center (see
/// [`crate::world::space`]): geodesic biodomes, lunar rovers, solar panel
/// arrays, huge deep-space radio dishes turning slowly, astronomical
/// observatory domes with telescopes, orbital rockets on their launchpads,
/// illuminated station ribs over the road, interdimensional stargate portals
/// connecting Earth and the Moon, and lunar boulders with survey beacons.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Space {
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub biodomes: &'static [Biodome],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rovers: &'static [Rover],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub solar_arrays: &'static [Row],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub dishes: &'static [RadioDish],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub observatories: &'static [Observatory],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub rockets: &'static [Rocket],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub arches: &'static [StationArch],
    #[serde(deserialize_with = "crate::circuits::data::leak", default)]
    pub portals: &'static [Portal],
    pub boulders: LunarBoulders,
}

/// An astronomical telescope observatory dome at `position` (x, z), its open
/// shutter slit and giant reflector telescope aimed towards `heading`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observatory {
    pub position: Vec2,
    pub heading: f32,
}

/// An interdimensional stargate portal spanning over the road at `along`
/// metres from the finish line, with twin energy pylons, counter-rotating
/// plasma rings, and a translucent glowing vortex curtain.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Portal {
    pub along: f32,
}

/// A geodesic habitat dome beside the road, with a metallic ring base,
/// glass panels and an airlock facing the track.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biodome {
    pub along: f32,
    pub side: Side,
    pub radius: f32,
}

/// A six-wheeled lunar rover beside the road, its gold-foil chassis facing
/// `heading` (relative to the road direction).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rover {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub heading: f32,
}

/// A parabolic deep-space radio telescope dish at `position` (x, z), its
/// dish pointed towards `heading` and turning slowly.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadioDish {
    pub position: Vec2,
    pub heading: f32,
}

/// An orbital rocket standing on its launchpad at `position` (x, z), with an
/// umbilical launch tower beside it facing `heading`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rocket {
    pub position: Vec2,
    pub heading: f32,
}

/// A sleek space-station structural rib/arch spanning over the road at
/// `along` metres from the finish line, with glowing cyan-white beacon strips.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationArch {
    pub along: f32,
}

/// Lunar boulders scattered along `spans` on both sides of the road.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LunarBoulders {
    pub max: usize,
    #[serde(deserialize_with = "crate::circuits::data::leak")]
    pub spans: &'static [Span],
}

/// The inside of a toy store (see [`crate::world::toys`]): the room, with
/// shelves of boxed toys round its walls and lamps in its ceiling; the
/// furniture the roads run on (tables, chairs and a tall shelving unit), the
/// tower a slide winds round, electric trains, castles of plastic bricks,
/// loops of orange track, mats on the floor and toys about. Heights are
/// the road's, as [`crate::track::Layout::elevation`]'s: the floor is at 0.
pub struct ToyStore {
    /// The corners of the room's floor (x, z): the least and the most.
    pub room: (Vec2, Vec2),
    /// How high its ceiling is over the floor (m).
    pub ceiling: f32,
    pub furniture: &'static [Furniture],
    pub towers: &'static [Tower],
    pub trains: &'static [Train],
    pub castles: &'static [Castle],
    pub loops: &'static [ToyLoop],
    pub mats: &'static [Mat],
    pub toys: &'static [Toy],
    pub bricks: &'static [Bricks],
    /// Its way in, if it is seen: a front of glass in a wall, the car park
    /// outside it.
    pub entrance: Option<Entrance>,
}

/// A shop's way in, in the wall of its room at the greatest z: a front of
/// glass from `from` to `to` along it (x), sliding doors in it round
/// `doors` (x), out onto a car park in the daylight (see
/// `world::shopfront`).
pub struct Entrance {
    pub from: f32,
    pub to: f32,
    pub doors: f32,
}

/// A piece of furniture: a box `size` across (x and z before it is turned
/// `heading`, as a road's heading), its top `top` over the floor, at
/// `center` (x, z).
pub struct Furniture {
    pub kind: FurnitureKind,
    pub center: Vec2,
    pub size: Vec2,
    pub heading: f32,
    pub top: f32,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FurnitureKind {
    /// A table on four legs: the slot-car track lies on its top.
    Table,
    /// A table whose top is a green baseplate of plastic bricks, studded,
    /// as a diorama's, castles on it.
    Diorama,
    /// A chair on four legs, its back on its left side (along x).
    Chair,
    /// A tall shelving unit, as down the middle of a shop, its shelves full
    /// of boxed toys on both sides: the road runs along its top.
    Shelves,
}

/// A tower a spiral road winds round, carrying it on brackets: a slide's,
/// as a fairground's helter-skelter, or Hot Wheels' blue column.
pub struct Tower {
    pub kind: TowerKind,
    pub center: Vec2,
    pub radius: f32,
    /// How high its top is over the floor (m): a helter-skelter's eaves,
    /// under its pointed roof.
    pub height: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TowerKind {
    /// Striped red and white under a pointed roof.
    HelterSkelter,
    /// A blue column with a cap.
    Column,
}

/// An electric train going round and round its track, a closed curve
/// through `track` (x, z), on whatever stands under it, `height` over the
/// floor: a locomotive and `wagons` wagons of `wagon` at `speed` (m/s).
pub struct Train {
    pub track: &'static [Vec2],
    pub height: f32,
    pub wagon: Wagon,
    pub wagons: u32,
    pub speed: f32,
}

/// What a train's locomotive pulls: Meshy's (see `tools/import_toys.py`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wagon {
    /// A green coach, longer than the locomotive.
    Coach,
    /// A red open wagon loaded with crates.
    Freight,
}

/// A castle of plastic bricks `size` across (x and z before it is turned
/// `heading`, its front along +z), standing `base` over the floor: one of
/// Meshy's, as big as fits; or one a road runs through (see
/// [`CastleKind::Through`]).
pub struct Castle {
    pub center: Vec2,
    pub size: Vec2,
    pub heading: f32,
    pub base: f32,
    pub kind: CastleKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CastleKind {
    /// Meshy's castle: walls and towers round a keep, its gate on its
    /// front.
    Big,
    /// Meshy's small castle: a round tower under a red roof, its gate on
    /// its front.
    Tower,
    /// Grey bricks with red roofs on its towers built round a road, which
    /// runs along it (the way `heading` says, as a road's), in under a
    /// gatehouse at one end and out under another at the other, between its
    /// walls.
    Through,
}

/// A loop of orange toy track on a blue stand, to look at: standing up,
/// facing along `heading`, `radius` round, `base` over the floor.
pub struct ToyLoop {
    pub center: Vec2,
    pub heading: f32,
    pub radius: f32,
    pub base: f32,
}

/// Something lying on the floor: a mat of foam tiles in bright colours, or
/// a rug of one colour, `size` across, turned `heading`.
pub struct Mat {
    pub center: Vec2,
    pub size: Vec2,
    pub heading: f32,
    pub kind: MatKind,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MatKind {
    Foam,
    Rug(Color),
}

/// A toy standing about, turned `heading` (its front along +z), `base` over
/// the floor, as big as `scale` says: times the size it was made at, for
/// one Meshy made (see `tools/import_toys.py`), or times its own size
/// (about a metre) for one built in code.
pub struct Toy {
    pub kind: ToyKind,
    pub position: Vec2,
    pub heading: f32,
    pub scale: f32,
    pub base: f32,
}

/// Something built of plastic bricks as big as the studs of the diorama's
/// baseplate (see `world::bricks`), standing `base` over the floor at
/// `position` (x, z), turned `heading` (its front along +z).
pub struct Bricks {
    pub kind: BrickKind,
    pub position: Vec2,
    pub heading: f32,
    pub base: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrickKind {
    /// `count` bricks spilled in a heap: most lying flat, a few on their
    /// sides, some stacked on others.
    Heap { count: u32 },
    /// A tower of `count` bricks stacked crosswise, as a child stacks them.
    Tower { count: u32 },
    /// A little house `width` studs across its front and `depth` deep, its
    /// walls `courses` bricks high, a door in its front and windows round
    /// it, under a roof stepping up to its ridge.
    House {
        width: u32,
        depth: u32,
        courses: u32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToyKind {
    /// A stack of wooden blocks with letters on them.
    Blocks,
    /// A beach ball in segments of bright colours.
    Ball,
    /// A teddy bear, sitting, a red bow under its chin (Meshy's).
    Bear,
    /// A rubber duck (Meshy's).
    Duck,
    /// A tin robot standing guard, blue (Meshy's).
    Robot,
    /// A spinning top in stripes (Meshy's).
    Top,
    /// A biplane hanging from the ceiling on its strings, `base` the height
    /// of its middle (Meshy's).
    Plane,
    /// A toy helicopter hanging from the ceiling, as the biplane (Meshy's).
    Helicopter,
    /// A stack of black tyres, as round a racetrack.
    Tyres,
    /// A pirate ship, its sails striped red and white (Meshy's).
    Ship,
    /// A plush giraffe standing tall (Meshy's).
    Giraffe,
    /// A doll's house of three storeys, its front painted in pastel colours
    /// (Meshy's).
    Dollhouse,
    /// A toy fire engine with its ladder (Meshy's).
    FireTruck,
    /// A winged mechanical pegasus (Meshy's).
    Pegasus,
    /// A monster truck made as a crocodile (Meshy's).
    CrocCar,
    /// A garage playset, its storeys stacked up a tower, yellow ramps
    /// spiralling down its side (Meshy's).
    Garage,
    /// A toy railway station, its platform under a canopy along its front
    /// (Meshy's).
    Station,
    /// A toy railway's mountain, a tunnel through it along its front and
    /// back (Meshy's): a train's track runs through it, along it, its
    /// middle on its tunnel.
    TunnelMountain,
    /// A rocking horse, white with a red saddle (Meshy's).
    RockingHorse,
    /// A nutcracker soldier standing guard (Meshy's).
    Nutcracker,
    /// A yellow toy dump truck (Meshy's).
    DumpTruck,
    /// A play teepee of striped canvas on its poles (Meshy's).
    Teepee,
    /// A tyrannosaurus (Meshy's), its head reaching out over the road beside
    /// it, well over the bikes: the one toy that does.
    Trex,
    /// A toy rocket standing on its fins, red and white (Meshy's).
    Rocket,
    /// A wooden toy chest, its lid open, full of toys (Meshy's).
    ToyChest,
    /// A jack-in-the-box, its clown sprung out of it (Meshy's).
    JackInBox,
    /// A xylophone lying on the floor, its bars in the colours of the
    /// rainbow, its two mallets beside them (Meshy's).
    Xylophone,
    /// A giant robot standing guard at the shop's door, as tall as the
    /// glass beside it is (Meshy's).
    GiantRobot,
    /// The shop's checkout counter: its till and card reader on top, sweets
    /// and drinks on shelves along its front, for those queueing (Meshy's).
    CheckoutCounter,
    /// A shopping trolley, red-handled; in a row of them, nested (Meshy's).
    ShoppingCart,
    /// A potted plant, a leafy tree in a white pot (Meshy's).
    PottedPlant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_can_wrap_around_the_finish_line() {
        let lap = 1000.0;
        let wrapped = span(900.0, 100.0);
        assert!(wrapped.contains(950.0, lap) && wrapped.contains(50.0, lap));
        assert!(!wrapped.contains(500.0, lap));
        assert_eq!(wrapped.length(lap), 200.0);
        // Se entra poco a poco.
        assert!(wrapped.weight(905.0, lap, 20.0) < 0.5);
        assert_eq!(wrapped.weight(0.0, lap, 20.0), 1.0);
        assert_eq!(LAP.weight(500.0, lap, 20.0), 1.0);
        assert_eq!(LAP.length(lap), lap);
    }
}
