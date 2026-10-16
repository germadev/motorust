//! Cómo es el escenario de cada pista: el terreno (costa, colinas, peñascos y
//! paredes de roca), lo que bordea la carretera (muros, túneles, puentes,
//! arcos y carteles) y el decorado (árboles, casas, hoteles y palmeras).
//! Los datos de cada pista están en [`crate::circuits`].

use bevy::prelude::*;

use crate::geometry::smoothstep;

/// Un tramo de la vuelta, en metros desde la meta. Puede pasar por la meta:
/// de 2400 a 300, por ejemplo.
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
pub struct Massif {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
    /// Whether the road climbs it: its smooth slopes are the ground the road
    /// is cut into, rather than rock standing up behind the walls.
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
pub struct Cutting {
    pub span: Span,
    pub side: Side,
    pub height: f32,
}

/// The greens of a track's grass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Greens {
    /// The arcade lawn: a bright green, a little lighter or deeper here and
    /// there.
    Lawn,
    /// A countryside's: swathes of many greens, lusher by the water, drier
    /// up high and towards the sun, and dark conifer forest up the hills,
    /// far from the roads and on the mountains across the sea.
    Countryside,
}

/// Una explanada: el terreno, llano a una altura (m sobre el mar) en un
/// círculo, para poner encima un edificio grande; or sloping, facing one
/// way.
pub struct Terrace {
    pub center: Vec2,
    pub radius: f32,
    /// How high its middle stands.
    pub height: f32,
    /// How much higher it stands for each metre along x and along z: zero
    /// for level ground.
    pub tilt: Vec2,
    /// How far past its edge the ground blends back into the land (m):
    /// wider where the land round it stands far above or below it, so the
    /// bank stays grass instead of breaking into a cliff.
    pub skirt: f32,
}

/// Dónde está el mar.
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
}

/// What the steep faces of the terrain (cliffs, rock walls and the rock
/// around the tunnels) are made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Soil {
    /// Natural rock with tilted strata.
    Rock,
    /// Brown checkerboard earth under a grass top, as in Loop Island.
    Checkered,
    /// A desert canyon of red and ochre rock over dry, dusty earth; its
    /// tunnels are mine galleries propped with timber and lit by lanterns.
    Mine,
}

/// The light of a track: a summer day; the last hours of a clear autumn
/// afternoon, the sun low and golden; or a low sun setting in a warm, hazy
/// sky.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SkyStyle {
    Day,
    GoldenHour,
    Dusk,
}

/// The colour of the water: the blue sea, or the red water of a flooded pit
/// rich in iron, as at Rio Tinto.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Water {
    Sea,
    Iron,
}

/// An open-pit mine round the water (see [`Scenery::pit`]): benches cut in
/// steps from its rim down to the water, following the shore, and its spoil
/// heaped round the rim.
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
#[derive(Clone, Copy, Debug)]
pub struct Bridge {
    pub span: Span,
    pub style: BridgeStyle,
    /// How high a wooden bridge's fence stands over its deck (m): most as
    /// high as [`WOODEN_FENCE`], a boardwalk low over the water lower. The
    /// other bridges have walls of their own, and leave it unused.
    pub fence: f32,
}

/// How high the fence of a wooden bridge usually stands over its deck (m).
pub const WOODEN_FENCE: f32 = 1.2;

/// How a bridge is built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BridgeStyle {
    /// A concrete deck under two red steel arches.
    RedArches,
    /// A wooden deck of planks on log piers, with a wooden fence on each side.
    Wooden,
    /// A concrete deck between two steel through trusses, braced across
    /// overhead, on concrete piers, as in West Way (Moto Racer).
    Truss,
}

/// Un tramo con otro muro distinto del de piedra a un lado.
pub struct WallRun {
    pub span: Span,
    pub side: Side,
    pub style: WallStyle,
}

/// Un arco sobre la carretera con un rótulo.
pub struct Gantry {
    pub along: f32,
    pub color: Color,
    /// El rótulo, o un tablero de ajedrez si no hay.
    pub text: Option<&'static str>,
}

/// Un cartel publicitario junto a la carretera.
pub struct Billboard {
    pub along: f32,
    pub side: Side,
    pub text: &'static str,
    pub color: Color,
    /// Desde qué letra, y en qué color, van las demás.
    pub accent: Option<(usize, Color)>,
}

/// Una fila de algo a lo largo de la carretera: en un tramo, a un lado, a
/// esta distancia de la línea central y cada tantos metros.
pub struct Row {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
    pub every: f32,
}

/// Especies de árbol.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
pub struct Forest {
    pub span: Span,
    pub species: &'static [Species],
}

pub const fn forest(span: Span, species: &'static [Species]) -> Forest {
    Forest { span, species }
}

/// Las casas: cuántas como mucho, de qué colores y, si hace falta, solo en
/// unos tramos.
pub struct Houses {
    pub max: usize,
    pub paints: &'static [Color],
    pub spans: &'static [Span],
    pub style: HouseStyle,
}

/// How the houses are built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HouseStyle {
    /// Plastered walls under a tiled gable roof.
    Cottage,
    /// The same, with a wooden false front standing up over the roof at the
    /// front and a porch along it, as in the towns of the Old West (and of
    /// West Way, in Moto Racer).
    FalseFront,
}

/// Todo lo que hace falta para construir el escenario de una pista.
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
    pub massifs: &'static [Massif],
    pub cuttings: &'static [Cutting],
    pub terraces: &'static [Terrace],
    /// Montañas lejanas que cierran el horizonte.
    pub mountains: fn(Vec2) -> f32,
    /// Colores de las franjas de los pianos.
    pub curbs: [Color; 2],
    pub walls: &'static [WallRun],
    pub tunnels: &'static [Span],
    pub bridges: &'static [Bridge],
    pub gantries: &'static [Gantry],
    pub billboards: &'static [Billboard],
    /// Farolas, cada `every` metros.
    pub lamps: &'static [Row],
    /// Especies de los árboles sueltos, y bosques: tramos con árboles altos
    /// muy juntos a los dos lados.
    pub trees: &'static [Species],
    pub forests: &'static [Forest],
    pub houses: Houses,
    pub palms: &'static [Row],
    /// Hoteles altos en fila, de colores.
    pub hotels: &'static [Row],
    /// Un gran hotel de los de antes: punto de la vuelta y distancia a la
    /// carretera (con signo, positiva a la izquierda).
    pub grand_hotel: Option<(f32, f32)>,
    /// Boyas de rayas en el mar.
    pub buoys: &'static [Vec2],
    /// Faros costeros en islotes o cabos.
    pub lighthouses: &'static [Lighthouse],
    /// Veleros y yates fondeados en la bahía o el mar abierto.
    pub boats: &'static [Boat],
    /// Flechas indicadoras de curva (<<< y >>>) en curvas cerradas.
    pub chevrons: &'static [ChevronSign],
    /// Tribunas cubiertas con público frente a la pista.
    pub grandstands: &'static [Grandstand],
    /// Sombrillas y tumbonas de colores en las playas.
    pub beach_parasols: &'static [ParasolCluster],
    /// Carteles de aviso de frenada (100 m, 50 m).
    pub brake_markers: &'static [BrakeMarker],
    /// What cliffs and rock walls are made of.
    pub soil: Soil,
    /// Rows of tall sunflowers turned towards the road.
    pub sunflowers: &'static [Row],
    /// Carved totem poles beside the road.
    pub totems: &'static [Totem],
    /// Giant vertical loops standing beside the road.
    pub loops: &'static [Loop],
    /// Lines of spinning golden rings floating over the road.
    pub rings: &'static [Rings],
    /// Waterfalls pouring down the rock walls of the cuttings.
    pub waterfalls: &'static [Waterfall],
    /// The light: day or dusk.
    pub sky: SkyStyle,
    /// The colour of the sea, the lakes and the rivers.
    pub water: Water,
    /// An open pit round the water, if there is one.
    pub pit: Option<Pit>,
    /// The works of a mine around the road, if there is one.
    pub mine: Option<Mine>,
    /// The roadside of a desert of the American West, if the road runs
    /// through one.
    pub desert: Option<Desert>,
    /// The scenery of feudal Japan, from models made with Meshy, or `None`:
    /// see [`Japan`].
    pub japan: Option<Japan>,
}

/// The scenery of feudal Japan, as at Ghost Island (see
/// [`crate::world::japan`]): corridors of torii over the road, props made
/// with Meshy standing about, and fields of flowers.
pub struct Japan {
    pub corridors: &'static [ToriiCorridor],
    pub props: &'static [Prop],
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
/// sides, from `near` to `far` metres from its centre line; round a point,
/// from `inner` to `outer` metres from it (a ring round a lake, or, from 0,
/// a round meadow, which nothing else is put in); or everywhere else.
pub enum FlowerArea {
    Road {
        span: Span,
        near: f32,
        far: f32,
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
}

/// A prop and where it stands, facing: by the road, `along` metres from the
/// finish line and `lateral` metres to the left of its centre line (to the
/// right if negative), facing the road; or anywhere, at `position` (x, z)
/// facing `heading` (see [`crate::geometry`]). It stands on the ground, or
/// in the sea, with its foot a little under the water; `scale` times as big
/// as made (see [`Prop::scaled`]), and leaning `lean` radians towards its
/// side that was -X as made (see [`Prop::leaning`]).
#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub model: PropModel,
    pub spot: Spot,
    pub scale: f32,
    pub lean: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Spot {
    Road { along: f32, lateral: f32 },
    At { position: Vec2, heading: f32 },
}

/// A prop by the road (see [`Prop`]).
pub const fn by_road(model: PropModel, along: f32, lateral: f32) -> Prop {
    Prop {
        model,
        spot: Spot::Road { along, lateral },
        scale: 1.0,
        lean: 0.0,
    }
}

impl Prop {
    /// The same prop, `scale` times as big.
    pub const fn scaled(self, scale: f32) -> Self {
        Prop { scale, ..self }
    }

    /// The same prop leaning `lean` radians about its foot, towards its side
    /// that was -X as made: a tree leaning out over what its crown spreads
    /// over.
    pub const fn leaning(self, lean: f32) -> Self {
        Prop { lean, ..self }
    }
}

/// A prop anywhere (see [`Prop`]).
pub const fn standing(model: PropModel, x: f32, z: f32, heading: f32) -> Prop {
    Prop {
        model,
        spot: Spot::At {
            position: Vec2::new(x, z),
            heading,
        },
        scale: 1.0,
        lean: 0.0,
    }
}

/// The roadside of a desert of the American West, as in West Way (Moto
/// Racer): gas stations, oil pumpjacks nodding on the dunes, highway
/// viaducts across the road, adobe pueblos, diamond warning signs before
/// the bends, cacti, and tumbleweeds rolling across the road.
pub struct Desert {
    pub gas_stations: &'static [GasStation],
    pub pumpjacks: &'static [Pumpjack],
    pub overpasses: &'static [Overpass],
    pub pueblos: &'static [Pueblo],
    pub curve_signs: &'static [CurveSign],
    pub cacti: Cacti,
    pub tumbleweeds: &'static [Tumbleweed],
}

/// Saguaro cacti beside the road: about `count` of them, scattered along
/// `spans` either side, on fairly level ground clear of the roads.
pub struct Cacti {
    pub count: usize,
    pub spans: &'static [Span],
}

/// A tumbleweed blown across the road `along` the lap, rolling and bouncing
/// from `reach` metres out on one side to as far on the other, and again.
pub struct Tumbleweed {
    pub along: f32,
    pub reach: f32,
}

/// A gas station beside the road, `lateral` metres from its centre line on
/// `side`, facing it: pumps under a canopy, a shop behind them and a tall
/// sign by the road.
pub struct GasStation {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// An oil pumpjack at `position` (x, z), its beam nodding towards
/// `heading` (see [`crate::geometry`]).
pub struct Pumpjack {
    pub position: Vec2,
    pub heading: f32,
}

/// A concrete highway viaduct across the road `along` the lap, the bottom
/// of its deck `height` metres over the road, from one rock wall into the
/// other on columns beyond the walls (put a [`Cutting`] on both sides
/// first).
pub struct Overpass {
    pub along: f32,
    pub height: f32,
}

/// An adobe pueblo beside the road, `lateral` metres from its centre line on
/// `side`: flat-roofed houses of earth, some of two floors, with beams
/// sticking out under their roofs, ladders and a dry tree.
pub struct Pueblo {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// A diamond warning sign before a bend that way, high on a post on its
/// outside, `along` the lap.
pub struct CurveSign {
    pub along: f32,
    pub direction: ChevronDirection,
    pub color: SignColor,
}

/// The colour of a diamond warning sign: green with a white arrow for a
/// bend, yellow with a black one for a sharp one, red with a white one for
/// a hairpin.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SignColor {
    Green,
    Yellow,
    Red,
}

/// The works of a mine: rails with ore carts rolling on them, timber
/// trestles carrying a rail over the road, headframes over the shafts,
/// lanterns on posts, piles of crates, barrels and ore, and the halls of
/// its excavation (its tunnels, see [`Scenery::tunnels`]).
pub struct Mine {
    pub rails: &'static [Rails],
    pub trestles: &'static [Trestle],
    pub headframes: &'static [Headframe],
    pub lanterns: &'static [Row],
    pub piles: &'static [Pile],
    pub halls: &'static [Hall],
}

/// A stretch of a mine's excavation, in metres along the lap (a branch off
/// the lap counts as the stretch it stands in for): the rock it is cut
/// through and how high its roof rises above the road in the middle (m).
/// Where there is none, the excavation is a gallery propped with timber.
pub struct Hall {
    pub span: Span,
    pub stone: Stone,
    pub crown: f32,
}

/// What the rock of a stretch of a mine's excavation is like, and the
/// ground beside its roads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stone {
    /// A gallery propped with timber sets, with planks beside the road.
    Timbered,
    /// Red and cream sandstone in layers, over red gravel.
    Sandstone,
    /// Dark slate with veins of gold, over black gravel.
    Slate,
    /// Ochre clay, its walls lined with planks, over beaten earth.
    Clay,
}

/// A rail line beside the road over a span, `lateral` metres from its
/// centre line, with `carts` ore carts rolling up and down it.
pub struct Rails {
    pub span: Span,
    pub side: Side,
    pub lateral: f32,
    pub carts: u32,
}

/// A timber trestle carrying a rail across the road at `along`, from one
/// rock wall to the other, with an ore cart going over and back.
pub struct Trestle {
    pub along: f32,
}

/// A headframe over a mine shaft: a timber tower with the winding wheels
/// turning on top, beside the road.
pub struct Headframe {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
}

/// What a pile beside the road is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PileKind {
    /// Crates, barrels and a box of dynamite.
    Supplies,
    /// A heap of ore with gold glinting in it.
    Ore,
}

/// A pile of something beside the road.
pub struct Pile {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub kind: PileKind,
}

/// A carved totem pole beside the road, facing it.
pub struct Totem {
    pub along: f32,
    pub side: Side,
}

/// A giant vertical loop on a block of earth beside the road, in the plane
/// of the road: from the road it is seen side on, as in a 2D platformer.
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
pub struct Rings {
    pub along: f32,
    pub count: u32,
    pub lateral: f32,
    pub height: f32,
}

/// A waterfall `width` metres wide down the rock wall of a cutting.
pub struct Waterfall {
    pub along: f32,
    pub side: Side,
    pub width: f32,
}

/// Un faro costero sobre un peñasco o cabo.
pub struct Lighthouse {
    pub position: Vec2,
    pub height: f32,
}

/// Embarcaciones fondeadas en el mar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoatKind {
    Sailboat,
    Yacht,
}

pub struct Boat {
    pub position: Vec2,
    pub heading: f32,
    pub kind: BoatKind,
}

/// Dirección de la flecha de curva.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ChevronDirection {
    Left,
    Right,
}

/// Cartel de curva con flechas reflectantes.
pub struct ChevronSign {
    pub along: f32,
    pub side: Side,
    pub direction: ChevronDirection,
}

/// Tribuna de espectadores cubierta.
pub struct Grandstand {
    pub along: f32,
    pub side: Side,
    pub lateral: f32,
    pub length: f32,
}

/// Grupo de sombrillas y tumbonas en la arena.
pub struct ParasolCluster {
    pub position: Vec2,
    pub count: usize,
}

/// Cartel de distancia a curva (100 m, 50 m).
pub struct BrakeMarker {
    pub along: f32,
    pub side: Side,
    pub distance: u32,
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
