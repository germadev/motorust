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
}

pub const fn massif(x: f32, z: f32, radius: f32, height: f32) -> Massif {
    Massif {
        center: Vec2::new(x, z),
        radius,
        height,
    }
}

/// Una pared de roca a un lado de la carretera, como si la hubieran abierto
/// cortando la montaña.
pub struct Cutting {
    pub span: Span,
    pub side: Side,
    pub height: f32,
}

/// Una explanada: el terreno, llano a una altura (m sobre el mar) en un
/// círculo, para poner encima un edificio grande.
pub struct Terrace {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
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
}

/// Las casas: cuántas como mucho, de qué colores y, si hace falta, solo en
/// unos tramos.
pub struct Houses {
    pub max: usize,
    pub paints: &'static [Color],
    pub spans: &'static [Span],
}

/// Todo lo que hace falta para construir el escenario de una pista.
pub struct Scenery {
    pub coast: Coast,
    /// Altura de las colinas (m).
    pub hills: f32,
    pub massifs: &'static [Massif],
    pub cuttings: &'static [Cutting],
    pub terraces: &'static [Terrace],
    /// Montañas lejanas que cierran el horizonte.
    pub mountains: fn(Vec2) -> f32,
    /// Colores de las franjas de los pianos.
    pub curbs: [Color; 2],
    pub walls: &'static [WallRun],
    pub tunnels: &'static [Span],
    pub bridges: &'static [Span],
    pub gantries: &'static [Gantry],
    pub billboards: &'static [Billboard],
    /// Farolas, cada `every` metros.
    pub lamps: &'static [Row],
    /// Especies de los árboles sueltos, y bosques: tramos con árboles altos
    /// muy juntos a los dos lados.
    pub trees: &'static [Species],
    pub forests: &'static [Span],
    pub houses: Houses,
    pub palms: &'static [Row],
    /// Hoteles altos en fila, de colores.
    pub hotels: &'static [Row],
    /// Un gran hotel de los de antes: punto de la vuelta y distancia a la
    /// carretera (con signo, positiva a la izquierda).
    pub grand_hotel: Option<(f32, f32)>,
    /// Boyas de rayas en el mar.
    pub buoys: &'static [Vec2],
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
