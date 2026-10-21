//! La Juguetería: una pista dentro de una tienda de juguetes. El trazado es
//! un circuito enorme de plástico Orange Asphalt repartido por el suelo de
//! la tienda, que sube por una rampa a la altura del asiento de un sillón,
//! pasa de un sillón a una mesa de juego, baja por un tobogán de plástico y
//! vuelve al suelo, donde pasa al lado de un tren eléctrico, por encima de
//! un circuito de Scalextric y por otro circuito estilo Hot Wheels. No hay
//! cielo: se ve el techo de la tienda con sus lámparas.

use bevy::prelude::*;

use super::{Ambience, Circuit, Song};
use crate::track::Layout;
use crate::world::scenery::*;

pub const TOY_STORE: Circuit = Circuit {
    name: "La Juguetería",
    name_en: "The Toy Store",
    description: "El interior de una juguetería: un circuito de juguete\n\
                  que sube a un sillón, pasa por una mesa y baja por un\n\
                  tobogán, al lado de un tren eléctrico",
    description_en: "The inside of a toy shop: a toy circuit that climbs\n\
                     on to an armchair, crosses a table and comes down a\n\
                     slide, beside an electric train",
    // Indoors: no sea, no gulls, no birds, no cicadas. Only the hum of the
    // shop.
    ambience: Ambience {
        sea: 0.0,
        gulls: 0.0,
        birds: 0.0,
        cicadas: 0.0,
        wind: 0.0,
    },
    song: Song {
        mp3: include_bytes!("../../assets/music/orange_asphalt.mp3"),
        length: 168.7,
        loudness: -15.5,
    },
    layout: Layout {
        control_points: &CONTROL_POINTS,
        elevation: &ELEVATION,
        banking: &[],
        narrows: &[],
        branches: &[],
    },
    scenery: Scenery {
        coast: Coast::Inland(|_| 1000.0),
        hills: 0.0,
        greens: Greens::Lawn,
        grass_shade: 1.0,
        massifs: &[],
        cuttings: &[],
        terraces: &[],
        mountains: |_| 0.0,
        curbs: [Color::srgb(0.95, 0.35, 0.05), Color::srgb(0.95, 0.95, 0.95)],
        // The edges of the toy track: bright blue plastic rails.
        walls: &[
            WallRun {
                span: LAP,
                side: Side::Left,
                style: WallStyle::ToyRail,
            },
            WallRun {
                span: LAP,
                side: Side::Right,
                style: WallStyle::ToyRail,
            },
        ],
        tunnels: &[],
        bridges: &[],
        dams: &[],
        gantries: &[
            Gantry {
                along: 0.0,
                color: Color::srgb(0.92, 0.38, 0.10),
                text: Some("TOY STORE"),
            },
            Gantry {
                along: 680.0,
                color: Color::srgb(0.13, 0.42, 0.80),
                text: Some("CHECKPOINT"),
            },
            Gantry {
                along: 1250.0,
                color: Color::srgb(0.13, 0.42, 0.80),
                text: Some("CHECKPOINT"),
            },
        ],
        billboards: &[],
        lamps: &[],
        trees: &[],
        forests: &[],
        houses: Houses {
            max: 0,
            paints: &[],
            spans: &[],
            style: HouseStyle::Cottage,
        },
        palms: &[],
        hotels: &[],
        grand_hotel: None,
        buoys: &[],
        lighthouses: &[],
        boats: &[],
        chevrons: &[],
        grandstands: &[],
        beach_parasols: &[],
        brake_markers: &[],
        soil: Soil::Rock,
        sunflowers: &[],
        totems: &[],
        loops: &[],
        rings: &[],
        waterfalls: &[],
        sky: SkyStyle::Indoor,
        water: Water::Sea,
        cartoon_water: false,
        pit: None,
        mine: None,
        desert: None,
        wheelie_halo: None,
        japan: None,
        middle_earth: None,
        space: None,
        toy_store: Some(SHOP),
    },
};

/// El circuito, dando la vuelta por la tienda: desde la mesa de juego, al
/// norte, por el tobogán; luego al oeste por el pasillo de los juguetes de
/// carrera, por el del ferrocarril —donde corre el tren eléctrico— y por el
/// de los juguetes de construcción, y de vuelta a la mesa por la rampa del
/// fondo.
const CONTROL_POINTS: [Vec2; 61] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -32.2),
    Vec2::new(0.0, -64.4),
    Vec2::new(0.0, -96.6),
    Vec2::new(0.0, -128.8),
    Vec2::new(0.0, -161.0),
    Vec2::new(0.0, -193.2),
    Vec2::new(0.0, -225.4),
    Vec2::new(0.0, -257.6),
    Vec2::new(0.0, -289.8),
    Vec2::new(0.0, -322.0),
    Vec2::new(0.0, -354.3),
    Vec2::new(0.0, -386.5),
    Vec2::new(0.0, -420.0),
    Vec2::new(-4.7, -450.4),
    Vec2::new(-19.2, -479.0),
    Vec2::new(-42.1, -501.5),
    Vec2::new(-70.8, -515.7),
    Vec2::new(-100.0, -520.0),
    Vec2::new(-134.8, -520.0),
    Vec2::new(-167.0, -520.0),
    Vec2::new(-199.2, -520.0),
    Vec2::new(-231.4, -520.0),
    Vec2::new(-263.6, -520.0),
    Vec2::new(-290.0, -520.0),
    Vec2::new(-327.1, -512.9),
    Vec2::new(-354.6, -496.3),
    Vec2::new(-375.4, -472.0),
    Vec2::new(-387.5, -442.2),
    Vec2::new(-390.0, -420.0),
    Vec2::new(-390.0, -378.0),
    Vec2::new(-390.0, -345.8),
    Vec2::new(-390.0, -313.6),
    Vec2::new(-390.0, -281.4),
    Vec2::new(-390.0, -249.2),
    Vec2::new(-390.0, -217.0),
    Vec2::new(-390.0, -184.8),
    Vec2::new(-390.0, -152.6),
    Vec2::new(-390.0, -120.4),
    Vec2::new(-390.0, -88.2),
    Vec2::new(-390.0, -56.0),
    Vec2::new(-390.0, -23.8),
    Vec2::new(-390.0, 8.4),
    Vec2::new(-390.0, 50.0),
    Vec2::new(-387.4, 72.6),
    Vec2::new(-375.2, 102.3),
    Vec2::new(-354.3, 126.6),
    Vec2::new(-326.7, 143.0),
    Vec2::new(-290.0, 150.0),
    Vec2::new(-263.2, 151.7),
    Vec2::new(-231.0, 154.2),
    Vec2::new(-198.8, 146.9),
    Vec2::new(-166.6, 147.2),
    Vec2::new(-134.4, 150.0),
    Vec2::new(-102.2, 150.0),
    Vec2::new(-60.0, 150.0),
    Vec2::new(-38.3, 145.9),
    Vec2::new(-12.7, 127.0),
    Vec2::new(0.0, 90.0),
    Vec2::new(0.0, 65.4),
    Vec2::new(0.0, 33.2),
];

/// La altura de la pista por la vuelta: sube por la rampa al sillón, pasa
/// por su asiento, sube otra vez a la mesa de juego, y baja por el tobogán.
const ELEVATION: [(f32, f32); 22] = [
    // La mesa de juego, donde está la meta: el circuito empieza en su
    // borde y se tira por el tobogán.
    (0.0, 6.0),
    (30.0, 6.0),
    // El tobogán, que baja a poco a poco.
    (46.0, 4.8),
    (62.0, 3.2),
    (78.0, 1.6),
    (90.0, 0.4),
    (100.0, 0.0),
    // La rampa al sillón.
    (130.0, 0.0),
    (155.0, 1.5),
    (175.0, 3.0),
    // El asiento del sillón.
    (195.0, 3.0),
    // Y de vuelta al suelo.
    (215.0, 1.5),
    (235.0, 0.4),
    (245.0, 0.0),
    (600.0, 0.0),
    (1200.0, 0.0),
    (1700.0, 0.0),
    // La rampa de vuelta a la mesa, ya casi en la meta.
    (1880.0, 0.0),
    (1900.0, 1.5),
    (1920.0, 3.0),
    (1940.0, 4.5),
    (1960.0, 6.0),
];

/// La tienda: el salón, su techo, el mobiliario y los juguetes.
const SHOP: ToyStore = ToyStore {
    // La sala: 400 m por 640 m, y el techo a 30 m.
    room: Vec2::new(225.0, 365.0),
    centre: Vec2::new(-195.0, -185.0),
    ceiling: 30.0,
    // El sillón por el que pasa el circuito, y la mesa de juego.
    chairs: &[CHAIR],
    tables: &[TABLE],
    // Las estanterías de las paredes.
    shelves: &SHELVES,
    // El tobogán, el circuito de Scalextric y el de Hot Wheels.
    slide: span(30.0, 95.0),
    scalextric: span(580.0, 740.0),
    hot_wheels: span(1520.0, 1735.0),
    // El tren eléctrico, por la vía que hay al lado del circuito.
    train: Some(TRAIN),
    toys: &TOYS,
};

/// El sillón enorme por el que pasa el circuito.
const CHAIR: Chair = Chair {
    position: Vec2::new(0.0, -190.0),
    heading: 0.0,
    width: 44.0,
    depth: 30.0,
    height: 3.0,
};

/// La mesa de juego por la que pasa el circuito.
const TABLE: Table = Table {
    position: Vec2::new(0.0, -2.0),
    heading: 0.0,
    width: 70.0,
    depth: 60.0,
    height: 6.0,
};

/// Las estanterías de juguetes de las paredes.
const SHELVES: [Shelf; 6] = [
    // La pared del fondo, con los juguetes de carrera.
    Shelf {
        position: Vec2::new(-160.0, -543.0),
        heading: std::f32::consts::PI,
        length: 300.0,
    },
    // La pared de la izquierda.
    Shelf {
        position: Vec2::new(-411.0, -180.0),
        heading: -std::f32::consts::FRAC_PI_2,
        length: 500.0,
    },
    // La pared de abajo.
    Shelf {
        position: Vec2::new(-190.0, 171.0),
        heading: 0.0,
        length: 320.0,
    },
    // La pared de la derecha.
    Shelf {
        position: Vec2::new(21.0, -180.0),
        heading: std::f32::consts::FRAC_PI_2,
        length: 500.0,
    },
    // Y dos estanterías sueltas en el medio de la sala.
    Shelf {
        position: Vec2::new(-110.0, -100.0),
        heading: 0.0,
        length: 90.0,
    },
    Shelf {
        position: Vec2::new(-210.0, 60.0),
        heading: std::f32::consts::PI,
        length: 90.0,
    },
];

/// La vía del tren eléctrico, al lado del circuito por el oeste.
const TRAIN: Train = Train {
    path: &[
        Vec2::new(-338.0, -330.0),
        Vec2::new(-312.0, -330.0),
        Vec2::new(-312.0, 10.0),
        Vec2::new(-338.0, 10.0),
    ],
    speed: 11.0,
};

/// Los juguetes tirados por el suelo de la tienda.
const TOYS: [Toy; 14] = [
    Toy {
        position: Vec2::new(-60.0, -40.0),
        kind: ToyKind::Ball,
        size: 5.0,
    },
    Toy {
        position: Vec2::new(-95.0, 20.0),
        kind: ToyKind::Blocks,
        size: 4.0,
    },
    Toy {
        position: Vec2::new(-140.0, 60.0),
        kind: ToyKind::Doll,
        size: 5.0,
    },
    Toy {
        position: Vec2::new(-210.0, 30.0),
        kind: ToyKind::Ball,
        size: 4.0,
    },
    Toy {
        position: Vec2::new(-250.0, -40.0),
        kind: ToyKind::Top,
        size: 3.5,
    },
    Toy {
        position: Vec2::new(-290.0, 60.0),
        kind: ToyKind::Blocks,
        size: 4.5,
    },
    Toy {
        position: Vec2::new(-160.0, -20.0),
        kind: ToyKind::Ball,
        size: 6.0,
    },
    Toy {
        position: Vec2::new(-100.0, -120.0),
        kind: ToyKind::Top,
        size: 3.0,
    },
    Toy {
        position: Vec2::new(-200.0, -150.0),
        kind: ToyKind::Doll,
        size: 5.5,
    },
    Toy {
        position: Vec2::new(-280.0, -180.0),
        kind: ToyKind::Ball,
        size: 5.5,
    },
    Toy {
        position: Vec2::new(-150.0, -280.0),
        kind: ToyKind::Blocks,
        size: 4.0,
    },
    Toy {
        position: Vec2::new(-60.0, -330.0),
        kind: ToyKind::Doll,
        size: 5.0,
    },
    Toy {
        position: Vec2::new(-240.0, -330.0),
        kind: ToyKind::Ball,
        size: 4.5,
    },
    Toy {
        position: Vec2::new(-40.0, 60.0),
        kind: ToyKind::Top,
        size: 3.5,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    /// La altura a la que corre la pista donde su perfil vale cero.
    const ROAD_BASE: f32 = 2.0;

    #[test]
    fn el_sillon_y_la_mesa_estan_bajo_la_pista() {
        let track = Track::new(&TOY_STORE.layout);
        for chair in SHOP.chairs {
            let road = track.road_height_at(chair.position, 20.0);
            assert!(
                (road - (ROAD_BASE + chair.height)).abs() < 0.2,
                "el sillón está a {road} m y la pista a {} m",
                ROAD_BASE + chair.height
            );
        }
        for table in SHOP.tables {
            let road = track.road_height_at(table.position, 20.0);
            assert!(
                (road - (ROAD_BASE + table.height)).abs() < 0.2,
                "la mesa está a {road} m y la pista a {} m",
                ROAD_BASE + table.height
            );
        }
    }

    #[test]
    fn el_tobogan_es_donde_la_pista_baja() {
        let track = Track::new(&TOY_STORE.layout);
        let lap = track.lap_length();
        let mut down = 0;
        for index in 0..track.sample_count() {
            let along = track.lap_along_at(index);
            let height = track.elevations()[index];
            if SHOP.slide.contains(along, lap) {
                down += 1;
                // La pista baja del borde de la mesa al suelo, y no sube.
                assert!(height <= ROAD_BASE + 6.1, "el tobogán sube a {height} m");
            }
        }
        assert!(down > 20, "el tobogán es muy corto: {down} puntos");
    }

    #[test]
    fn la_via_del_tren_queda_al_lado_de_la_pista() {
        let track = Track::new(&TOY_STORE.layout);
        let train = SHOP.train.as_ref().expect("el tren");
        for point in train.path {
            let nearest = track.nearest(*point);
            assert!(
                nearest.distance > 20.0,
                "la vía del tren pasa por la pista, a {} m",
                nearest.distance
            );
        }
    }

    #[test]
    fn la_sala_abrga_toda_la_pista() {
        let track = Track::new(&TOY_STORE.layout);
        let (min, max) = track.bounds();
        let (min, max) = (min - 20.0, max + 20.0);
        assert!(min.x >= SHOP.centre.x - SHOP.room.x);
        assert!(max.x <= SHOP.centre.x + SHOP.room.x);
        assert!(min.y >= SHOP.centre.y - SHOP.room.y);
        assert!(max.y <= SHOP.centre.y + SHOP.room.y);
    }
}
