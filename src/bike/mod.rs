//! La moto: controles, física arcade a paso fijo y su representación visual.
//!
//! La física no simula el equilibrio sobre dos ruedas: la moto se mueve como
//! un cuerpo con velocidad y orientación, y la inclinación es solo visual (la
//! que tendría una moto real tomando esa curva a esa velocidad).

mod ai;
mod chassis;
mod model;

use std::ops::Range;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::geometry::{approach, forward, heading_of, smoothstep, to_world, wrap_angle};
use crate::lobby::{ControllerBinding, LocalPlayersConfig};
use crate::locale::Language;
use crate::menu::{Choice, GameState};
use crate::pause::{self, DebugOptions, TurnLimit};
use crate::settings::{Action, Settings};
use crate::track::{SURFACE_Y, Surface, Track, WALL_DISTANCE};
use crate::welcome::GameMode;
use ai::Rival;
use chassis::{Chassis, Forces};
use model::{
    BikeBlueprint, BikeModelPlugin, COVER, DIAGONAL, Design, FLECHA, Livery, ROSSO, TRICOLOR, VERDE,
};

const GRAVITY: f32 = 9.81;
/// En el aire la moto sube y cae con más gravedad que en la realidad (y aún
/// más al bajar): los saltos son cortos y secos, como en un arcade.
const RISE_GRAVITY: f32 = 2.0 * GRAVITY;
const FALL_GRAVITY: f32 = 4.0 * GRAVITY;
/// En el aire se puede corregir algo la dirección (fracción del giro normal).
const AIR_STEERING: f32 = 0.3;
/// Forma de la inclinación según el agarre usado: por debajo de 1 se tumba
/// algo más al principio de la curva que al final.
const LEAN_CURVE: f32 = 0.75;
/// Con teclado o cruceta (todo o nada), la dirección va hacia donde se pulsa
/// desde el primer instante, ya deprisa, y cada vez más cuanto más tiempo se
/// lleva pulsando: empieza a esta rapidez (1/s) y acelera tanto así (1/s²).
/// Al soltar vuelve al centro más suave: empieza despacio y va cada vez más
/// deprisa cuanto más tiempo se lleva sin pulsar.
const DIGITAL_STEER_START: f32 = 2.5;
const DIGITAL_STEER_ACCEL: f32 = 5.0;
const DIGITAL_RELEASE_START: f32 = 0.5;
const DIGITAL_RELEASE_ACCEL: f32 = 4.0;
/// Cuanto más deprisa va la moto, menos gira el teclado manteniéndolo
/// pulsado: a toda velocidad, hasta esta fracción del tope, casi sin entrar
/// en la zona en la que la moto pierde velocidad (ver `CORNER_LOSS_FROM`).
/// Despacio gira del todo. Entre estas dos velocidades (m/s) se pasa de uno
/// a otro. Responde igual de rápido a cualquier velocidad. Es el límite
/// «alto» de las opciones de depuración, que pueden quitarlo o dejar una
/// parte, para el teclado y para el stick.
const DIGITAL_STEER_FAST: f32 = 0.8;
const DIGITAL_STEER_SPEEDS: (f32, f32) = (15.0, 70.0);
/// Lo que tarda la hierba en frenar del todo (s), y la parte de ese tiempo
/// en la que apenas se nota: pisarla un momento casi no frena. Al volver al
/// asfalto se recupera el doble de rápido.
const GRASS_BITE_TIME: f32 = 1.0;
const GRASS_GRACE: f32 = 0.25;
/// Rapidez con la que el cabeceo sigue a la pendiente o a la trayectoria (1/s).
const PITCH_RESPONSE: f32 = 10.0;
/// Medio ancho de la moto con el piloto: lo que se puede arrimar a un muro.
const BIKE_HALF_WIDTH: f32 = 0.45;
/// Distancia mínima entre dos motos, vistas desde arriba como círculos.
const BIKE_SPACING: f32 = 1.2;
/// Fracción de la velocidad que se pierde en un choque de frente.
const IMPACT_SPEED_LOSS: f32 = 0.75;
/// Qué parte del ángulo contra el muro se corrige en cada paso de contacto:
/// la moto se pone paralela al muro y sigue, en vez de quedarse clavada.
const WALL_ALIGN: f32 = 0.6;
/// En curva el motor empuja algo más (fracción, del todo desde
/// `CORNER_GRIP_FROM`): la moto sale lanzada de las curvas en vez de quedarse
/// en ellas.
const CORNER_DRIVE: f32 = 0.4;
/// Desde qué parte del tope de la dirección se apura la curva: la moto agarra
/// cada vez más, hasta tanto más (fracción) con la dirección a tope, despacio
/// .. deprisa.
const CORNER_GRIP_FROM: f32 = 0.7;
const CORNER_GRIP: Range<f32> = 0.6..1.4;
/// Y solo desde aquí, casi a tope, el empuje de más se acaba, el motor empuja
/// menos (fracción) y la moto pierde velocidad sola (m/s², deprisa) lo justo
/// para cerrar las curvas más cerradas: en arcade, mantener el giro basta para
/// no salirse. Tumbarse sin llegar ahí no cuesta velocidad.
const CORNER_LOSS_FROM: f32 = 0.85;
const CORNER_POWER_CUT: f32 = 0.6;
const CORNER_SCRUB: f32 = 35.0;

pub struct BikePlugin;

impl Plugin for BikePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BikeModelPlugin)
            .init_resource::<BikeTuning>()
            .init_resource::<Race>()
            .add_systems(OnEnter(GameState::Race), spawn_grid)
            .add_systems(
                FixedUpdate,
                (ai::drive_rivals, simulate_bikes, separate_bikes).chain(),
            )
            .add_systems(
                RunFixedMainLoop,
                (
                    (restart_race, read_player_input)
                        .run_if(in_state(GameState::Race).and_then(pause::running))
                        .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                    interpolate_bikes.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
                ),
            );
    }
}

/// Estado de la carrera: los rivales esperan en la parrilla hasta que el
/// jugador acelera por primera vez.
#[derive(Resource, Default)]
pub struct Race {
    pub started: bool,
}

/// Puesto de salida de cada moto ([`Track::grid_slot`]), para volver a él al
/// reiniciar la carrera.
#[derive(Component, Clone, Copy)]
struct GridPosition(usize);

/// Marca la moto que lleva el jugador.
#[derive(Component)]
pub struct Player;

/// Marca los jugadores locales en modo multijugador (0 para Player 1, 1 para Player 2).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct LocalPlayer(pub usize);

/// Lo que pide el piloto en este momento. Al jugador lo rellena el teclado o
/// el mando; a los rivales, su IA.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct BikeInput {
    /// Acelerador, de 0 a 1.
    pub throttle: f32,
    /// Freno (y marcha atrás si está parada), de 0 a 1.
    pub brake: f32,
    /// Dirección, de -1 (derecha) a 1 (izquierda). La moto la sigue como un
    /// muelle, poco a poco.
    pub steer: f32,
    /// Dirección que entra tal cual, sin el muelle: la del teclado o la
    /// cruceta, que ya llega suavizada (ver `DigitalSteer`). Se suma a la
    /// otra.
    pub steer_direct: f32,
    /// Cuánto tira del manillar para hacer el caballito, de 0 a 1.
    pub wheelie: f32,
}

/// Estado físico de la moto. Solo cambia en los pasos fijos de la simulación.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct BikeState {
    /// Posición sobre el suelo (x, z).
    pub position: Vec2,
    /// Orientación (ver `geometry`). No se acota a [-π, π) para poder
    /// interpolar entre pasos sin saltos.
    pub heading: f32,
    /// Velocidad en m/s; negativa en marcha atrás.
    pub speed: f32,
    /// Dirección ya suavizada, de -1 a 1.
    pub steer: f32,
    /// Lo deprisa que cambia la parte de la dirección que va con muelle (1/s).
    pub steer_rate: f32,
    /// La parte de `steer` que entró directa en el último paso.
    pub steer_direct: f32,
    /// Cuánto se ha metido en la hierba, de 0 a 1: sube mientras la pisa y
    /// baja al volver al asfalto.
    pub grass: f32,
    /// Inclinación en radianes; positiva hacia la izquierda.
    pub lean: f32,
    /// Altura de referencia del chasis (m): la de las ruedas en el suelo con
    /// la suspensión en reposo. En el aire, las ruedas cuelgan por debajo lo
    /// que se extienda la suspensión.
    pub elevation: f32,
    /// Velocidad vertical (m/s).
    pub climb: f32,
    /// Si las ruedas tocan el suelo.
    pub grounded: bool,
    /// Cabeceo (rad): positivo con el morro hacia arriba. Sigue la pendiente
    /// en el suelo y la trayectoria en el aire.
    pub pitch: f32,
    /// Aceleración hacia delante que se nota en el último paso (m/s²): la del
    /// motor, el freno y las resistencias, sin la de la cuesta (la gravedad
    /// tira igual de la moto que del piloto). Es la que hunde la suspensión y
    /// echa atrás o adelante al piloto.
    pub accel: f32,
    /// Caballito y suspensión.
    pub chassis: Chassis,
}

impl BikeState {
    pub fn parked_at((position, heading): (Vec2, f32)) -> Self {
        Self {
            position,
            heading,
            speed: 0.0,
            steer: 0.0,
            steer_rate: 0.0,
            steer_direct: 0.0,
            grass: 0.0,
            lean: 0.0,
            elevation: 0.0,
            climb: 0.0,
            grounded: true,
            pitch: 0.0,
            accel: 0.0,
            chassis: Chassis::default(),
        }
    }

    /// Parada en una posición del circuito, sobre el asfalto.
    pub fn parked_on(track: &Track, pose: (Vec2, f32)) -> Self {
        Self {
            elevation: track.road_height(pose.0),
            ..Self::parked_at(pose)
        }
    }
}

/// Lo que hay bajo las ruedas al empezar un paso.
#[derive(Clone, Copy, Debug)]
pub struct Ground {
    pub surface: Surface,
    /// Pendiente en la dirección de la moto: metros que sube por metro.
    pub grade: f32,
    /// Curvatura vertical (1/m): positiva en lo alto de una loma.
    pub crest: f32,
}

impl Ground {
    #[cfg(test)]
    pub fn flat(surface: Surface) -> Self {
        Self {
            surface,
            grade: 0.0,
            crest: 0.0,
        }
    }
}

/// Estado del paso anterior, para interpolar el render entre dos pasos.
#[derive(Component, Clone, Copy, Debug)]
struct PreviousBikeState(BikeState);

/// Pivote de la inclinación: hijo de la moto, en el punto de contacto con el suelo.
#[derive(Component)]
struct LeanPivot;

/// Parámetros de conducción. Es un recurso para poder ajustarlos en caliente.
#[derive(Resource, Clone, Debug)]
pub struct BikeTuning {
    /// Punta en asfalto (m/s).
    pub max_speed: f32,
    /// Aceleración del motor desde parado (m/s²); se va apagando cerca de la punta.
    pub engine_accel: f32,
    pub brake_decel: f32,
    pub reverse_accel: f32,
    pub reverse_max_speed: f32,
    /// Deceleración que siempre actúa: rodadura y aire, simplificados.
    pub rolling_resistance: f32,
    /// Deceleración extra al soltar acelerador y freno.
    pub engine_braking: f32,
    /// Punta sobre hierba (m/s).
    pub grass_max_speed: f32,
    /// Deceleración extra cuando se va más rápido que la punta del suelo que se pisa.
    pub overspeed_drag: f32,
    /// Radio de giro mínimo (m): limita el giro a poca velocidad.
    pub min_turn_radius: f32,
    /// Aceleración lateral máxima (m/s²): limita el giro a mucha velocidad.
    pub max_lateral_accel: f32,
    /// Rapidez con la que la dirección sigue al mando (1/s): va como un muelle
    /// sin rebote, así entra y sale de las curvas poco a poco y sin tirones.
    pub steer_response: f32,
    /// Inclinación máxima (rad).
    pub max_lean: f32,
    /// Rapidez con la que la moto se tumba al entrar en la curva (1/s): casi
    /// al momento.
    pub lean_in_response: f32,
    /// Rapidez con la que se levanta (1/s): más suave.
    pub lean_response: f32,
}

impl Default for BikeTuning {
    fn default() -> Self {
        Self {
            max_speed: 80.0,
            engine_accel: 14.5,
            brake_decel: 28.0,
            reverse_accel: 4.0,
            reverse_max_speed: 5.0,
            rolling_resistance: 0.5,
            engine_braking: 2.5,
            grass_max_speed: 36.0,
            overspeed_drag: 10.0,
            min_turn_radius: 3.5,
            max_lateral_accel: 65.0,
            steer_response: 11.0,
            max_lean: 55f32.to_radians(),
            lean_in_response: 16.0,
            lean_response: 7.0,
        }
    }
}

impl BikeTuning {
    /// Lo más deprisa que puede girar la moto a esta velocidad (rad/s), con
    /// la dirección a fondo.
    pub fn max_yaw_rate(&self, speed: f32) -> f32 {
        self.max_yaw_rate_with_grip(speed, 1.0)
    }

    /// Igual, con el agarre multiplicado por `grip`.
    fn max_yaw_rate_with_grip(&self, speed: f32, grip: f32) -> f32 {
        let v = speed.abs();
        (v / self.min_turn_radius).min(self.max_lateral_accel * grip / v)
    }
}

/// Avanza la moto un paso de `dt` segundos. Es una función pura: no toca el
/// ECS, así se puede probar y reutilizar (repeticiones, IA, red...).
pub fn step(
    state: &BikeState,
    input: &BikeInput,
    ground: &Ground,
    tuning: &BikeTuning,
    dt: f32,
) -> BikeState {
    // En el aire ni el motor ni el freno ni la dirección hacen nada.
    let contact = if state.grounded { 1.0 } else { 0.0 };
    let throttle = input.throttle.clamp(0.0, 1.0) * contact;
    let brake = input.brake.clamp(0.0, 1.0) * contact;
    // La hierba frena poco a poco: la punta baja según lo que se lleva en ella.
    let on_grass = matches!(ground.surface, Surface::Grass) && state.grounded;
    let grass = if on_grass {
        move_toward(state.grass, 1.0, dt / GRASS_BITE_TIME)
    } else {
        move_toward(state.grass, 0.0, 2.0 * dt / GRASS_BITE_TIME)
    };
    let top_speed = tuning
        .max_speed
        .lerp(tuning.grass_max_speed, smoothstep(GRASS_GRACE, 1.0, grass))
        * (1.0 + state.chassis.wheelie_boost());

    // Motor: empuja fuerte desde parado y se queda sin empuje al llegar a la punta.
    let speed_ratio = (state.speed.max(0.0) / top_speed).min(1.0);
    // En curva empuja algo más: tumbarse se premia, no se paga. Solo al
    // apurar, con la dirección a tope, el motor empuja menos y la moto frena
    // (más abajo) para cerrar la curva.
    let steer = state.steer.abs();
    let at_limit = smoothstep(CORNER_LOSS_FROM, 1.0, steer);
    let corner_power = 1.0 + CORNER_DRIVE * (steer / CORNER_GRIP_FROM).min(1.0) * (1.0 - at_limit)
        - CORNER_POWER_CUT * at_limit;
    // Con un caballito empuja algo más.
    let power = throttle
        * (1.0 - speed_ratio * speed_ratio)
        * corner_power
        * (1.0 + state.chassis.wheelie_drive());
    let mut accel = power * tuning.engine_accel;
    // Freno: frena mientras avanza; parada (o casi), da marcha atrás.
    if state.speed > 0.5 {
        accel -= brake * tuning.brake_decel;
    } else {
        accel -= brake * tuning.reverse_accel;
    }
    // Cuesta arriba la gravedad frena; cuesta abajo, empuja.
    let slope_sine = ground.grade / (1.0 + ground.grade * ground.grade).sqrt();
    accel -= GRAVITY * slope_sine * contact;
    let mut speed = state.speed + accel * dt;

    // Resistencias: se oponen al movimiento, pero nunca lo invierten.
    let mut resistance = tuning.rolling_resistance;
    if state.grounded && throttle == 0.0 && brake == 0.0 {
        resistance += tuning.engine_braking;
    }
    if speed.abs() > top_speed {
        resistance += tuning.overspeed_drag;
    }
    resistance += CORNER_SCRUB * at_limit * smoothstep(25.0, 60.0, speed) * contact;
    speed = move_toward(speed, 0.0, resistance * dt);
    let speed = speed.clamp(-tuning.reverse_max_speed, top_speed.max(tuning.max_speed));
    let accel = (speed - state.speed) / dt + GRAVITY * slope_sine * contact;

    // Giro: a poca velocidad lo limita el radio de giro; a mucha, el agarre.
    // El stick (y la IA) mueven la dirección con un muelle; el teclado entra
    // directo, que ya llega suave.
    let response = tuning.steer_response;
    let spring = state.steer - state.steer_direct;
    let steer_accel = response * response * (input.steer.clamp(-1.0, 1.0) - spring)
        - 2.0 * response * state.steer_rate;
    let steer_rate = state.steer_rate + steer_accel * dt;
    let steer_direct = input.steer_direct.clamp(-1.0, 1.0);
    let steer = (spring + steer_rate * dt + steer_direct).clamp(-1.0, 1.0);
    // Al apurar con la dirección a tope, la moto agarra más y cierra la curva.
    // Más cuanto más deprisa: despacio manda el radio de giro.
    let extra_grip = CORNER_GRIP
        .start
        .lerp(CORNER_GRIP.end, smoothstep(25.0, 60.0, speed.abs()));
    let corner_grip = 1.0 + extra_grip * smoothstep(CORNER_GRIP_FROM, 1.0, steer.abs());
    let max_yaw_rate = tuning.max_yaw_rate_with_grip(speed, corner_grip);
    // Con una rueda en el aire apenas se puede girar.
    let grip = state.chassis.steering_grip();
    let traction = if state.grounded { 1.0 } else { AIR_STEERING };
    let yaw_rate = steer * max_yaw_rate * speed.signum() * traction * grip;
    let heading = state.heading + yaw_rate * dt;

    // Inclinación según cuánto agarre pide la curva: poco giro, poca
    // inclinación, y del todo solo al apurar. (La de equilibrio frente a la
    // gravedad, con un agarre tan alto, tumbaría la moto con muy poco giro.)
    let grip_used = (speed * yaw_rate / tuning.max_lateral_accel).clamp(-1.0, 1.0);
    let target_lean = tuning.max_lean * grip_used.abs().powf(LEAN_CURVE).copysign(grip_used);
    // Se tumba deprisa y se levanta suave. Al cambiar de lado en una curva,
    // también se tumba deprisa hacia el lado contrario.
    let leaning_in = (target_lean.abs() > state.lean.abs() && target_lean * state.lean >= 0.0)
        || target_lean * state.lean < 0.0;
    let lean_rate = if leaning_in {
        tuning.lean_in_response
    } else {
        tuning.lean_response
    };

    let target_pitch = if state.grounded {
        ground.grade.atan()
    } else {
        state.climb.atan2(speed.abs().max(1.0))
    };

    let chassis = state.chassis.step(
        &Forces {
            pull: input.wheelie.clamp(0.0, 1.0),
            brake,
            throttle,
            power,
            speed,
            accel,
            // En los valles la carretera empuja hacia arriba y comprime la suspensión.
            lift: -ground.crest * speed * speed * contact,
            grounded: state.grounded,
        },
        dt,
    );

    BikeState {
        position: state.position + forward(heading) * speed * dt,
        heading,
        speed,
        steer,
        steer_rate,
        steer_direct,
        grass,
        lean: approach(state.lean, target_lean, lean_rate, dt),
        pitch: approach(state.pitch, target_pitch, PITCH_RESPONSE, dt),
        accel,
        chassis,
        ..*state
    }
}

/// Pega la moto al suelo o la deja volar. Sobre el suelo sigue su altura; si
/// en un cambio de rasante haría falta más fuerza hacia abajo de la que da la
/// gravedad para seguir la carretera, despega, y aterriza al volver a tocarla.
/// `ground` es lo que había bajo las ruedas al empezar el paso y
/// `ground_height`, la altura del suelo donde ha acabado.
pub fn settle(state: &mut BikeState, ground: &Ground, ground_height: f32, dt: f32) {
    let road_climb = ground.grade * state.speed;
    if state.grounded {
        // Se compara con la gravedad con la que volaría: así no da saltitos
        // que no llegan a nada.
        let gravity = if road_climb > 0.0 {
            RISE_GRAVITY
        } else {
            FALL_GRAVITY
        };
        if ground.crest * state.speed * state.speed > gravity {
            state.grounded = false;
            state.climb = road_climb;
            // Despega el chasis: las ruedas se quedan donde estaban y, a partir
            // de aquí, cuelgan lo que se extienda la suspensión.
            state.elevation = ground_height + state.chassis.heave;
        } else {
            state.climb = road_climb;
            state.elevation = ground_height;
            return;
        }
    }
    let gravity = if state.climb > 0.0 {
        RISE_GRAVITY
    } else {
        FALL_GRAVITY
    };
    state.elevation += state.climb * dt - 0.5 * gravity * dt * dt;
    state.climb -= gravity * dt;
    let wheels = state.elevation - state.chassis.heave;
    if wheels <= ground_height {
        // El golpe del aterrizaje lo absorbe la suspensión.
        state.chassis.land(state.climb - road_climb);
        state.elevation = ground_height;
        state.climb = road_climb;
        state.grounded = true;
    }
}

/// Un paso completo de una moto en el circuito: conducción, muros y suelo.
pub fn advance(
    state: &BikeState,
    input: &BikeInput,
    track: &Track,
    tuning: &BikeTuning,
    dt: f32,
) -> BikeState {
    let nearest = track.nearest(state.position);
    let ground = Ground {
        surface: track.surface_beside(&nearest),
        grade: track.grade(state.position, forward(state.heading)),
        crest: track.crest(&nearest),
    };
    let mut next = step(state, input, &ground, tuning, dt);
    hit_walls(&mut next, track);
    let ground_height = track.road_height(next.position);
    settle(&mut next, &ground, ground_height, dt);
    next
}

/// Mantiene la moto entre los muros que bordean la carretera. Si se mete en
/// uno, la saca y le quita velocidad según lo de frente que haya sido el golpe
/// (rozarlo apenas frena, darse de cara la frena mucho) y la pone paralela al
/// muro, para que siga en vez de quedarse clavada.
pub fn hit_walls(state: &mut BikeState, track: &Track) {
    let limit = WALL_DISTANCE - BIKE_HALF_WIDTH;
    let nearest = track.nearest(state.position);
    if nearest.distance <= limit {
        return;
    }
    // Hacia fuera de la pista, perpendicular al muro.
    let normal = (state.position - nearest.point) / nearest.distance;
    state.position = nearest.point + normal * limit;
    let moving = forward(state.heading) * state.speed.signum();
    // Se pierde la velocidad contra el muro, no la que va a lo largo.
    let impact = moving.dot(normal).max(0.0);
    state.speed *= 1.0 - IMPACT_SPEED_LOSS * impact * impact;
    if state.speed > 0.0 {
        let along = Vec2::new(-normal.y, normal.x);
        let along = if along.dot(moving) >= 0.0 {
            along
        } else {
            -along
        };
        state.heading += wrap_angle(heading_of(along) - state.heading) * WALL_ALIGN;
    }
}

fn move_toward(value: f32, target: f32, max_delta: f32) -> f32 {
    value + (target - value).clamp(-max_delta, max_delta)
}

/// Una moto que se puede elegir: su diseño, su decoración (solo cuenta en las
/// hechas por código) y cómo se presenta en la pantalla de inicio.
pub struct Entry {
    pub name: &'static str,
    pub description: &'static str,
    pub description_en: &'static str,
    /// Color de su nombre en la pantalla de inicio.
    pub color: Srgba,
    design: Design,
    livery: &'static Livery,
}

impl Entry {
    pub fn description(&self, lang: Language) -> &'static str {
        match lang {
            Language::Spanish => self.description,
            Language::English => self.description_en,
        }
    }
}

/// Las motos de la carrera. La que elige el jugador sale desde su puesto y
/// las demás ocupan, por orden, los de los rivales.
pub const ROSTER: [Entry; 8] = [
    Entry {
        name: "Shogun TZ",
        description: "Azul y blanca, como la de la portada de Moto Racer",
        description_en: "Blue and white, like the Moto Racer cover",
        color: Srgba::rgb(0.35, 0.6, 1.0),
        design: Design::Shogun,
        livery: &COVER,
    },
    Entry {
        name: "Legend 88",
        description: "Blanca y roja",
        description_en: "White and red",
        color: Srgba::rgb(1.0, 0.3, 0.35),
        design: Design::Legend,
        livery: &FLECHA,
    },
    Entry {
        name: "Azura RZ",
        description: "Amarilla, blanca y negra",
        description_en: "Yellow, white and black",
        color: Srgba::rgb(1.0, 0.85, 0.15),
        design: Design::Azura,
        livery: &DIAGONAL,
    },
    Entry {
        name: "Kraken RS",
        description: "Verde lima y negra",
        description_en: "Lime green and black",
        color: Srgba::rgb(0.5, 0.95, 0.2),
        design: Design::Kraken,
        livery: &VERDE,
    },
    Entry {
        name: "Falcon KRZ",
        description: "Roja, azul y blanca",
        description_en: "Red, blue and white",
        color: Srgba::rgb(1.0, 0.25, 0.25),
        design: Design::Falcon,
        livery: &TRICOLOR,
    },
    Entry {
        name: "Stiletto V4",
        description: "Roja",
        description_en: "Red",
        color: Srgba::rgb(0.95, 0.15, 0.15),
        design: Design::Stiletto,
        livery: &ROSSO,
    },
    Entry {
        name: "Cobra XT",
        description: "Naranja y negra",
        description_en: "Orange and black",
        color: Srgba::rgb(1.0, 0.5, 0.1),
        design: Design::Cobra,
        livery: &ROSSO,
    },
    Entry {
        name: "Ronin YS",
        description: "Azul y negra",
        description_en: "Blue and black",
        color: Srgba::rgb(0.25, 0.45, 1.0),
        design: Design::Ronin,
        livery: &COVER,
    },
];

/// Quién sale de cada puesto de la parrilla ([`Track::grid_slot`]), de la
/// pole hacia atrás: los rivales, del más hábil al menos, y el jugador el
/// último, para tener a quién adelantar. De cada rival, su habilidad y su
/// carril en carrera.
const GRID: [Option<Rival>; 8] = [
    Some(Rival {
        skill: 0.95,
        lane: 2.0,
    }),
    Some(Rival {
        skill: 0.94,
        lane: -2.0,
    }),
    Some(Rival {
        skill: 0.92,
        lane: 4.0,
    }),
    Some(Rival {
        skill: 0.91,
        lane: -4.0,
    }),
    Some(Rival {
        skill: 0.9,
        lane: 0.0,
    }),
    Some(Rival {
        skill: 0.89,
        lane: 1.0,
    }),
    Some(Rival {
        skill: 0.88,
        lane: -1.0,
    }),
    None,
];

/// El puesto del jugador en [`GRID`].
const PLAYER_SLOT: usize = GRID.len() - 1;

/// Qué moto de [`ROSTER`] va en cada puesto de [`GRID`] si el jugador elige
/// `choice`: la suya en el del jugador y las demás, por orden, en los de los
/// rivales.
fn lineup(choice: usize) -> [usize; GRID.len()] {
    let mut lineup = [choice; GRID.len()];
    let mut others = (0..ROSTER.len()).filter(|&entry| entry != choice);
    for (slot, entry) in lineup.iter_mut().enumerate() {
        if slot != PLAYER_SLOT {
            *entry = others.next().expect("hay una moto para cada rival");
        }
    }
    lineup
}

/// El color de una moto, el de su nombre en [`ROSTER`]: el de su punto en el
/// minimapa.
#[derive(Component, Clone, Copy)]
pub struct BikeColor(pub Color);

/// Lo que hace falta para crear motos: las mallas compartidas y dónde guardar
/// las de cada moto.
#[derive(SystemParam)]
pub struct BikeAssets<'w> {
    blueprint: Res<'w, BikeBlueprint>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

impl BikeAssets<'_> {
    /// Crea una moto de [`ROSTER`] parada en `state`, con su piloto.
    pub fn spawn(&mut self, commands: &mut Commands, entry: &Entry, state: BikeState) -> Entity {
        let bike = commands
            .spawn((
                state,
                PreviousBikeState(state),
                BikeInput::default(),
                BikeColor(entry.color.into()),
                Transform::from_translation(to_world(state.position, SURFACE_Y))
                    .with_rotation(Quat::from_rotation_y(state.heading)),
                Visibility::default(),
            ))
            .id();
        model::spawn(
            commands,
            bike,
            &self.blueprint,
            entry.design,
            entry.livery,
            &mut self.meshes,
            &mut self.materials,
            &mut self.images,
        );
        bike
    }
}

/// Al empezar la carrera: la parrilla con la moto elegida para el jugador.
fn spawn_grid(
    mut commands: Commands,
    track: Res<Track>,
    choice: Res<Choice>,
    mut race: ResMut<Race>,
    mut assets: BikeAssets,
    game_mode: Res<GameMode>,
) {
    race.started = false;
    for (slot, (rival, entry)) in GRID.iter().zip(lineup(choice.0)).enumerate() {
        let state = BikeState::parked_on(&track, track.grid_slot(slot));
        let bike = assets.spawn(&mut commands, &ROSTER[entry], state);
        let mut bike = commands.entity(bike);
        bike.insert((GridPosition(slot), DespawnOnExit(GameState::Race)));
        if *game_mode == GameMode::SplitScreen && slot == PLAYER_SLOT - 1 {
            bike.insert(LocalPlayer(1));
        } else {
            match *rival {
                Some(rival) => {
                    bike.insert((rival, ai::Line(rival.lane)));
                }
                None => {
                    bike.insert(Player);
                    if *game_mode == GameMode::SplitScreen {
                        bike.insert(LocalPlayer(0));
                    }
                }
            }
        }
    }
}

fn read_controller_binding(
    binding: ControllerBinding,
    keyboard: &ButtonInput<KeyCode>,
    gamepads: &Query<&Gamepad>,
    options: &DebugOptions,
    speed: f32,
    dt: f32,
    digital: &mut DigitalSteer,
) -> BikeInput {
    match binding {
        ControllerBinding::Wasd => {
            let throttle = if keyboard.pressed(KeyCode::KeyW) {
                1.0
            } else {
                0.0
            };
            let brake = if keyboard.pressed(KeyCode::KeyS) {
                1.0
            } else {
                0.0
            };
            let left = if keyboard.pressed(KeyCode::KeyA) {
                1.0f32
            } else {
                0.0f32
            };
            let right = if keyboard.pressed(KeyCode::KeyD) {
                1.0f32
            } else {
                0.0f32
            };
            let wheelie = if keyboard.pressed(KeyCode::Space) {
                1.0
            } else {
                0.0
            };
            let steer_direct = digital.update(
                (left - right).clamp(-1.0, 1.0),
                reach_with(options.keyboard_limit, speed),
                options.keyboard_rate.factor(),
                dt,
            );
            BikeInput {
                throttle,
                brake,
                steer: 0.0,
                steer_direct,
                wheelie,
            }
        }
        ControllerBinding::Arrows => {
            let throttle = if keyboard.pressed(KeyCode::ArrowUp) {
                1.0
            } else {
                0.0
            };
            let brake = if keyboard.pressed(KeyCode::ArrowDown) {
                1.0
            } else {
                0.0
            };
            let left = if keyboard.pressed(KeyCode::ArrowLeft) {
                1.0f32
            } else {
                0.0f32
            };
            let right = if keyboard.pressed(KeyCode::ArrowRight) {
                1.0f32
            } else {
                0.0f32
            };
            let wheelie = if keyboard.any_pressed([
                KeyCode::ControlRight,
                KeyCode::ShiftRight,
                KeyCode::Numpad0,
                KeyCode::KeyL,
            ]) {
                1.0
            } else {
                0.0
            };
            let steer_direct = digital.update(
                (left - right).clamp(-1.0, 1.0),
                reach_with(options.keyboard_limit, speed),
                options.keyboard_rate.factor(),
                dt,
            );
            BikeInput {
                throttle,
                brake,
                steer: 0.0,
                steer_direct,
                wheelie,
            }
        }
        ControllerBinding::Gamepad(entity) => {
            if let Ok(gamepad) = gamepads.get(entity) {
                let trigger_r = gamepad.get(GamepadButton::RightTrigger2).unwrap_or(0.0);
                let trigger_l = gamepad.get(GamepadButton::LeftTrigger2).unwrap_or(0.0);
                let south = if gamepad.pressed(GamepadButton::South) {
                    1.0
                } else {
                    0.0
                };
                let west = if gamepad.pressed(GamepadButton::West) {
                    1.0
                } else {
                    0.0
                };
                let throttle = trigger_r.max(south);
                let brake = trigger_l.max(west);

                let mut steer = -stick_steer(gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0));
                steer *= reach_with(options.stick_limit, speed);

                let dpad_left = if gamepad.pressed(GamepadButton::DPadLeft) {
                    1.0f32
                } else {
                    0.0f32
                };
                let dpad_right = if gamepad.pressed(GamepadButton::DPadRight) {
                    1.0f32
                } else {
                    0.0f32
                };
                let steer_direct = digital.update(
                    (dpad_left - dpad_right).clamp(-1.0, 1.0),
                    reach_with(options.keyboard_limit, speed),
                    options.keyboard_rate.factor(),
                    dt,
                );

                let pulled_back =
                    (-gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0) - 0.3) / 0.7;
                let wheelie_btn = if gamepad.pressed(GamepadButton::East) {
                    1.0
                } else {
                    0.0
                };
                let wheelie = pulled_back.clamp(0.0, 1.0).max(wheelie_btn);

                BikeInput {
                    throttle,
                    brake,
                    steer: steer.clamp(-1.0, 1.0),
                    steer_direct,
                    wheelie,
                }
            } else {
                BikeInput::default()
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn read_player_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    settings: Res<Settings>,
    game_mode: Res<GameMode>,
    local_players: Res<LocalPlayersConfig>,
    mut race: ResMut<Race>,
    options: Res<DebugOptions>,
    player: Single<(&mut BikeInput, &BikeState), With<Player>>,
    mut other_players: Query<(&mut BikeInput, &BikeState, &LocalPlayer), Without<Player>>,
    mut digital_p1: Local<DigitalSteer>,
    mut digital_p2: Local<DigitalSteer>,
) {
    let (mut input, state) = player.into_inner();
    let dt = time.delta_secs();

    if *game_mode == GameMode::SplitScreen {
        if let Some(binding) = local_players.p1 {
            *input = read_controller_binding(
                binding,
                &keyboard,
                &gamepads,
                &options,
                state.speed,
                dt,
                &mut digital_p1,
            );
        }
        for (mut p2_input, p2_state, local_player) in &mut other_players {
            if let (1, Some(binding)) = (local_player.0, local_players.p2) {
                *p2_input = read_controller_binding(
                    binding,
                    &keyboard,
                    &gamepads,
                    &options,
                    p2_state.speed,
                    dt,
                    &mut digital_p2,
                );
            }
        }
    } else {
        // Modo 1 jugador normal
        let controls = &settings.controls;
        let key = |action| controls.key_value(action, &keyboard);
        let mut throttle = key(Action::Throttle);
        let mut brake = key(Action::Brake);
        let mut pressed = key(Action::Left) - key(Action::Right);
        let mut steer = 0.0;
        let mut wheelie = key(Action::Wheelie);

        for gamepad in &gamepads {
            let button = |action| controls.button_value(action, gamepad);
            throttle = throttle.max(button(Action::Throttle));
            brake = brake.max(button(Action::Brake));
            steer -= stick_steer(gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0));
            let digital = |action| if button(action) > 0.5 { 1.0 } else { 0.0 };
            pressed += digital(Action::Left) - digital(Action::Right);
            let pulled_back = (-gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0) - 0.3) / 0.7;
            wheelie = wheelie
                .max(pulled_back.clamp(0.0, 1.0))
                .max(button(Action::Wheelie));
        }

        steer *= reach_with(options.stick_limit, state.speed);
        let steer_direct = digital_p1.update(
            pressed.clamp(-1.0, 1.0),
            reach_with(options.keyboard_limit, state.speed),
            options.keyboard_rate.factor(),
            dt,
        );

        *input = BikeInput {
            throttle,
            brake,
            steer: steer.clamp(-1.0, 1.0),
            steer_direct,
            wheelie,
        };
    }

    let p2_throttles = other_players.iter().any(|(inp, _, _)| inp.throttle > 0.0);
    if (input.throttle > 0.0 || p2_throttles) && !race.started {
        race.started = true;
    }
}

/// Lo más que gira el teclado manteniéndolo pulsado yendo a `speed` (m/s).
fn digital_reach(speed: f32) -> f32 {
    let (slow, fast) = DIGITAL_STEER_SPEEDS;
    1.0 - (1.0 - DIGITAL_STEER_FAST) * smoothstep(slow, fast, speed.abs())
}

/// Lo más que se gira yendo a `speed` (m/s) con este límite de depuración:
/// todo, sin límite; lo del teclado de siempre, con el alto.
fn reach_with(limit: TurnLimit, speed: f32) -> f32 {
    1.0 - (1.0 - digital_reach(speed)) * limit.strength()
}

/// Dirección del teclado o la cruceta: su valor (de -1 a 1), lo que se pulsa
/// y cuánto tiempo lleva así.
#[derive(Default)]
struct DigitalSteer {
    value: f32,
    pressed: f32,
    time: f32,
}

impl DigitalSteer {
    /// Avanza `dt` segundos con `pressed` pulsado (-1, 0 o 1), girando como
    /// mucho `reach` (de 0 a 1) y con una rapidez multiplicada por `rate_factor`.
    /// Solo suaviza lo que se pulsa: sin pulsar, vuelve al centro y la moto no gira.
    fn update(&mut self, pressed: f32, reach: f32, rate_factor: f32, dt: f32) -> f32 {
        if pressed != self.pressed {
            // Si se pulsa hacia el otro lado estando inclinado o recuperando,
            // aborta la recuperación para empezar de inmediato hacia el nuevo
            // lado (la misma respuesta que pulsando desde el centro).
            if pressed != 0.0 && self.value * pressed < 0.0 {
                self.value = 0.0;
            }
            (self.pressed, self.time) = (pressed, 0.0);
        }
        // En pasos de un milisegundo: igual a cualquier tasa de fotogramas.
        let steps = (dt * 1000.0).ceil().max(1.0);
        let dt = dt / steps;
        for _ in 0..steps as usize {
            if pressed != 0.0 && self.value * pressed < 0.0 {
                self.value = 0.0;
            }
            let returning = pressed == 0.0;
            let (start, accel, target) = if returning {
                (DIGITAL_RELEASE_START, DIGITAL_RELEASE_ACCEL, 0.0)
            } else {
                (
                    DIGITAL_STEER_START * rate_factor,
                    DIGITAL_STEER_ACCEL * rate_factor,
                    pressed * reach,
                )
            };
            let rate = start + accel * (self.time + 0.5 * dt);
            self.value = move_toward(self.value, target, rate * dt);
            self.time += dt;
        }
        self.value
    }
}

/// Curva de respuesta del stick: cerca del centro gira poco, para corregir
/// la trazada con precisión, y a fondo gira todo.
fn stick_steer(x: f32) -> f32 {
    const DEAD_ZONE: f32 = 0.08;
    const EXPONENT: f32 = 1.6;
    let pushed = ((x.abs() - DEAD_ZONE) / (1.0 - DEAD_ZONE)).clamp(0.0, 1.0);
    pushed.powf(EXPONENT).copysign(x)
}

/// Con la tecla de reiniciar (R o Select en el mando, si no se han cambiado)
/// todas las motos vuelven a la parrilla y la carrera espera a que el jugador
/// acelere.
fn restart_race(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    settings: Res<Settings>,
    track: Res<Track>,
    mut race: ResMut<Race>,
    mut bikes: Query<(&GridPosition, &mut BikeState, &mut PreviousBikeState)>,
) {
    if !settings
        .controls
        .just_pressed(Action::Restart, &keyboard, gamepads.iter())
    {
        return;
    }
    race.started = false;
    for (grid, mut state, mut previous) in &mut bikes {
        *state = BikeState::parked_on(&track, track.grid_slot(grid.0));
        // Sin estado anterior distinto, la interpolación no dibuja un "viaje".
        previous.0 = *state;
    }
}

fn simulate_bikes(
    time: Res<Time>,
    tuning: Res<BikeTuning>,
    track: Res<Track>,
    mut bikes: Query<(&BikeInput, &mut BikeState, &mut PreviousBikeState)>,
) {
    let dt = time.delta_secs();
    for (input, mut state, mut previous) in &mut bikes {
        previous.0 = *state;
        *state = advance(&state, input, &track, &tuning, dt);
    }
}

fn separate_bikes(mut bikes: Query<&mut BikeState>) {
    let mut pairs = bikes.iter_combinations_mut();
    while let Some([mut a, mut b]) = pairs.fetch_next() {
        bump(&mut a, &mut b);
    }
}

/// Separa dos motos que se solapan. La que alcanza a la otra por detrás se
/// queda con la velocidad de la de delante, como si la hubiera tocado.
pub fn bump(a: &mut BikeState, b: &mut BikeState) {
    let offset = a.position - b.position;
    let distance = offset.length();
    if distance >= BIKE_SPACING {
        return;
    }
    let normal = offset.try_normalize().unwrap_or(Vec2::X);
    let push = (BIKE_SPACING - distance) / 2.0;
    a.position += normal * push;
    b.position -= normal * push;
    // `normal` apunta de b hacia a.
    if forward(a.heading).dot(-normal) > 0.0 && a.speed > b.speed {
        a.speed = b.speed;
    }
    if forward(b.heading).dot(normal) > 0.0 && b.speed > a.speed {
        b.speed = a.speed;
    }
}

/// Coloca cada moto entre su estado anterior y el actual según cuánto falta
/// para el siguiente paso fijo, para que el movimiento se vea suave a
/// cualquier tasa de fotogramas.
fn interpolate_bikes(
    fixed_time: Res<Time<Fixed>>,
    mut bikes: Query<(&BikeState, &PreviousBikeState, &mut Transform), Without<LeanPivot>>,
    mut pivots: Query<(&ChildOf, &mut Transform), With<LeanPivot>>,
) {
    let alpha = fixed_time.overstep_fraction();
    for (state, previous, mut transform) in &mut bikes {
        let (from, to) = (&previous.0, state);
        let position = from.position.lerp(to.position, alpha);
        let elevation = from.elevation.lerp(to.elevation, alpha);
        let heading = from.heading.lerp(to.heading, alpha);
        let pitch = from.pitch.lerp(to.pitch, alpha);
        transform.translation = to_world(position, elevation + SURFACE_Y);
        transform.rotation = Quat::from_rotation_y(heading) * Quat::from_rotation_x(pitch);
    }
    for (child_of, mut transform) in &mut pivots {
        if let Ok((state, previous, _)) = bikes.get(child_of.parent()) {
            let lean = previous.0.lean.lerp(state.lean, alpha);
            transform.rotation = Quat::from_rotation_z(lean);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use crate::pause::SteerRate;

    use super::*;

    const DT: f32 = 1.0 / 120.0;

    fn drive(input: BikeInput, surface: Surface, seconds: f32, from: BikeState) -> BikeState {
        let tuning = BikeTuning::default();
        let mut state = from;
        for _ in 0..(seconds / DT) as usize {
            state = step(&state, &input, &Ground::flat(surface), &tuning, DT);
        }
        state
    }

    fn parked() -> BikeState {
        BikeState::parked_at((Vec2::ZERO, 0.0))
    }

    fn full_throttle() -> BikeInput {
        BikeInput {
            throttle: 1.0,
            ..default()
        }
    }

    #[test]
    fn throttle_moves_forward_up_to_top_speed() {
        let tuning = BikeTuning::default();
        let state = drive(full_throttle(), Surface::Asphalt, 3.0, parked());
        assert!(state.speed > 20.0, "acelera con brío: {}", state.speed);
        assert!(state.position.y < 0.0, "avanza hacia -Z");

        let state = drive(full_throttle(), Surface::Asphalt, 120.0, parked());
        assert!(state.speed <= tuning.max_speed);
        assert!(state.speed > 0.9 * tuning.max_speed);
    }

    #[test]
    fn grass_slows_the_bike_down() {
        let tuning = BikeTuning::default();
        let fast = drive(full_throttle(), Surface::Asphalt, 30.0, parked());
        let on_grass = drive(full_throttle(), Surface::Grass, 10.0, fast);
        assert!(on_grass.speed <= tuning.grass_max_speed + 0.5);
    }

    #[test]
    fn the_chosen_bike_takes_the_player_slot_and_the_rest_the_rivals() {
        assert!(GRID[PLAYER_SLOT].is_none(), "el jugador no es un rival");
        assert_eq!(GRID.iter().filter(|rival| rival.is_none()).count(), 1);
        for choice in 0..ROSTER.len() {
            let lineup = lineup(choice);
            assert_eq!(lineup[PLAYER_SLOT], choice);
            let mut sorted = lineup;
            sorted.sort();
            assert_eq!(sorted, [0, 1, 2, 3, 4, 5, 6, 7], "cada moto una vez");
        }
        // Sin cambiar de moto: las demás, por orden, delante del jugador.
        assert_eq!(lineup(0), [1, 2, 3, 4, 5, 6, 7, 0]);
    }

    #[test]
    fn brushing_the_grass_hardly_slows_down() {
        let fast = drive(full_throttle(), Surface::Asphalt, 30.0, parked());
        let brushed = drive(full_throttle(), Surface::Grass, 0.3, fast);
        assert!(
            fast.speed - brushed.speed < 1.0,
            "pierde {}",
            fast.speed - brushed.speed
        );
        // Un segundo dentro sí frena, y al volver al asfalto se recupera.
        let deep = drive(full_throttle(), Surface::Grass, 1.0, fast);
        assert!(
            fast.speed - deep.speed > 5.0,
            "pierde {}",
            fast.speed - deep.speed
        );
        let back = drive(full_throttle(), Surface::Asphalt, 0.6, deep);
        assert_eq!(back.grass, 0.0);
    }

    #[test]
    fn brake_stops_then_reverses() {
        let tuning = BikeTuning::default();
        let moving = drive(full_throttle(), Surface::Asphalt, 5.0, parked());
        let brake = BikeInput {
            brake: 1.0,
            ..default()
        };
        let reversing = drive(brake, Surface::Asphalt, 8.0, moving);
        assert!(reversing.speed < 0.0);
        assert!(reversing.speed >= -tuning.reverse_max_speed);
    }

    #[test]
    fn coasting_comes_to_a_stop() {
        let moving = drive(full_throttle(), Surface::Asphalt, 5.0, parked());
        let stopped = drive(BikeInput::default(), Surface::Asphalt, 60.0, moving);
        assert_eq!(stopped.speed, 0.0);
    }

    #[test]
    fn cannot_turn_while_stopped() {
        let input = BikeInput {
            steer: 1.0,
            ..default()
        };
        let state = drive(input, Surface::Asphalt, 2.0, parked());
        assert_eq!(state.heading, 0.0);
        assert_eq!(state.position, Vec2::ZERO);
    }

    #[test]
    fn steering_builds_up_and_settles_without_jerks() {
        let tuning = BikeTuning::default();
        let moving = drive(full_throttle(), Surface::Asphalt, 3.0, parked());
        let input = BikeInput {
            throttle: 1.0,
            steer: 1.0,
            ..default()
        };
        let mut state = moving;
        let mut previous_rate = 0.0;
        let mut steps = 0;
        while state.steer < 0.99 {
            state = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
            // Empieza despacio: el cambio de la dirección crece de paso a paso.
            let rate_change = (state.steer_rate - previous_rate).abs();
            assert!(rate_change * DT < 0.01, "tirón de {rate_change}");
            previous_rate = state.steer_rate;
            steps += 1;
        }
        let seconds = steps as f32 * DT;
        assert!((0.3..0.9).contains(&seconds), "tarda {seconds} s");
        // Y no se pasa: llega sin rebotar.
        let settled = drive(input, Surface::Asphalt, 1.0, state);
        assert!(settled.steer <= 1.0 && settled.steer > 0.99);
    }

    /// Valores de la dirección del teclado, fotograma a fotograma, pulsando
    /// lo que diga `pressed` en cada instante.
    fn digital_steer(seconds: f32, dt: f32, pressed: impl Fn(f32) -> f32) -> Vec<f32> {
        digital_steer_at(0.0, seconds, dt, pressed)
    }

    /// Igual, yendo a `speed` (m/s).
    fn digital_steer_at(
        speed: f32,
        seconds: f32,
        dt: f32,
        pressed: impl Fn(f32) -> f32,
    ) -> Vec<f32> {
        digital_steer_with_rate(speed, 1.0, seconds, dt, pressed)
    }

    /// Igual, con un multiplicador de rapidez `rate_factor`.
    fn digital_steer_with_rate(
        speed: f32,
        rate_factor: f32,
        seconds: f32,
        dt: f32,
        pressed: impl Fn(f32) -> f32,
    ) -> Vec<f32> {
        let mut digital = DigitalSteer::default();
        (0..(seconds / dt).round() as usize)
            .map(|frame| {
                digital.update(
                    pressed(frame as f32 * dt),
                    digital_reach(speed),
                    rate_factor,
                    dt,
                )
            })
            .collect()
    }

    #[test]
    fn keyboard_leans_the_bike_at_once_and_lets_it_up_softly() {
        let tuning = BikeTuning::default();
        let mut digital = DigitalSteer::default();
        let mut state = BikeState {
            speed: 60.0,
            ..parked()
        };
        let (hold, release) = (0.8, (0.8 / DT).round() as usize);
        let mut leans = Vec::new();
        for frame in 0..(1.6 / DT).round() as usize {
            let pressed = if frame < release { 1.0 } else { 0.0 };
            let input = BikeInput {
                throttle: 1.0,
                steer_direct: digital.update(pressed, digital_reach(state.speed), 1.0, DT),
                ..default()
            };
            state = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
            leans.push(state.lean);
        }
        let held = leans[release - 1];
        let seconds = |frames: usize| frames as f32 * DT;
        let half_in = seconds(leans.iter().position(|&l| l >= 0.5 * held).unwrap());
        let half_out = seconds(
            leans[release..]
                .iter()
                .position(|&l| l <= 0.5 * held)
                .unwrap(),
        );
        println!(
            "tumbada {held:.3} rad en {hold} s: la mitad en {half_in:.3} s, se levanta a la mitad en {half_out:.3} s"
        );
        assert!(half_in < 0.2, "tarda {half_in} s en tumbarse a medias");
        assert!(half_out > 1.5 * half_in, "se levanta en {half_out} s");
    }

    #[test]
    fn the_debug_limits_take_all_part_or_none_of_the_turn_limit() {
        let flat_out = 80.0;
        assert_eq!(reach_with(TurnLimit::Off, flat_out), 1.0);
        assert_eq!(
            reach_with(TurnLimit::High, flat_out),
            digital_reach(flat_out)
        );
        let low = reach_with(TurnLimit::Low, flat_out);
        let medium = reach_with(TurnLimit::Medium, flat_out);
        assert!(1.0 > low && low > medium && medium > digital_reach(flat_out));
        // Despacio no limita ninguno.
        assert_eq!(reach_with(TurnLimit::High, 5.0), 1.0);
    }

    #[test]
    fn keyboard_steering_turns_less_the_faster_the_bike_goes() {
        let held = |speed: f32| digital_steer_at(speed, 1.5, DT, |_| 1.0);
        let at = |values: &[f32], seconds: f32| values[(seconds / DT).round() as usize - 1];
        let (parked, cruising, flat_out) = (held(5.0), held(45.0), held(80.0));
        // Despacio gira del todo; deprisa, mantenerlo se queda a las puertas de
        // apurar.
        assert_eq!(*parked.last().unwrap(), 1.0);
        let most = *flat_out.last().unwrap();
        assert!((0.75..0.85).contains(&most), "a fondo: {most}");
        let middle = *cruising.last().unwrap();
        assert!(most < middle && middle < 1.0, "a media: {middle}");
        // Responde igual de rápido a cualquier velocidad.
        for seconds in [0.05, 0.1, 0.2] {
            assert_eq!(at(&flat_out, seconds), at(&parked, seconds));
        }
        // Y llega a lo más que gira en algo menos de medio segundo.
        assert!(at(&flat_out, 0.5) > most - 0.01);
    }

    #[test]
    fn keyboard_steering_answers_at_once_and_speeds_up_while_held() {
        let values = digital_steer(1.0, DT, |_| 1.0);
        let at = |seconds: f32| values[(seconds / DT).round() as usize - 1];
        // Sin retraso: ya en el primer fotograma se mueve.
        assert!(values[0] > 0.0);
        // Un toque corto gira un poco; mantenerlo un poco, bastante; algo más
        // de un cuarto de segundo, del todo.
        assert!((0.2..0.35).contains(&at(0.1)), "toque: {}", at(0.1));
        assert!((0.5..0.7).contains(&at(0.2)), "{}", at(0.2));
        assert_eq!(at(0.35), 1.0);
        // Cada vez más deprisa: gira más en la segunda décima que en la primera.
        assert!(at(0.2) - at(0.1) > at(0.1));
        assert!(values.windows(2).all(|w| w[1] >= w[0]));
    }

    #[test]
    fn keyboard_steering_returns_faster_the_longer_it_is_released() {
        let values = digital_steer(1.5, DT, |t| if t < 0.7 { 1.0 } else { 0.0 });
        let at = |seconds: f32| values[(seconds / DT).round() as usize - 1];
        assert_eq!(at(0.7), 1.0);
        // Al soltar empieza a volver enseguida, poco a poco y cada vez más
        // deprisa: más suave que al pulsar.
        let first = at(0.7) - at(0.8);
        let second = at(0.8) - at(0.9);
        assert!(first > 0.0 && second > first * 1.3, "{first} y {second}");
        let pressing = digital_steer(0.1, DT, |_| 1.0);
        assert!(first < 0.5 * pressing.last().unwrap());
        // En algo más de medio segundo está en el centro, sin cruzar al otro
        // lado.
        assert!(at(1.2) > 0.0);
        assert_eq!(at(1.35), 0.0);
        assert!(values.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn keyboard_steering_aborts_recovery_to_turn_the_other_way() {
        // Al cambiar de lado aborta la recuperación y empieza de inmediato
        // hacia el otro lado, como si se pulsara desde el centro.
        let values = digital_steer(1.5, DT, |t| if t < 0.7 { 1.0 } else { -1.0 });
        let at = |seconds: f32| values[(seconds / DT).round() as usize - 1];
        let from_center = digital_steer(0.2, DT, |_| -1.0);
        let at_center = |seconds: f32| from_center[(seconds / DT).round() as usize - 1];
        // En 0.1 s tras cambiar de lado está exactamente en la misma posición
        // que pulsando brevemente sin estar inclinado.
        assert_eq!(at(0.8), at_center(0.1));
        assert!(at(0.8) < 0.0);
        assert_eq!(*values.last().unwrap(), -1.0);
    }

    #[test]
    fn keyboard_steering_aborts_active_recovery_when_tapping_opposite() {
        // Si se suelta (empieza a recuperar) y se da un toque al lado opuesto,
        // aborta la recuperación y va a la misma posición que desde el centro.
        let values = digital_steer(1.5, DT, |t| {
            if t < 0.4 {
                1.0
            } else if t < 0.6 {
                0.0
            } else {
                -1.0
            }
        });
        let at = |seconds: f32| values[(seconds / DT).round() as usize - 1];
        let from_center = digital_steer(0.2, DT, |_| -1.0);
        let at_center = |seconds: f32| from_center[(seconds / DT).round() as usize - 1];
        // En 0.6 s estaba recuperando hacia el centro (todavía positivo).
        assert!(at(0.6) > 0.0);
        // En 0.7 s (0.1 s de toque opuesto), está exactamente en la posición del
        // toque desde el centro.
        assert_eq!(at(0.7), at_center(0.1));
        assert!(at(0.7) < 0.0);
    }

    #[test]
    fn keyboard_steering_is_the_same_at_any_frame_rate() {
        let pressed = |t: f32| if t < 0.4 { 1.0 } else { 0.0 };
        let fast = digital_steer(0.6, DT, pressed);
        let slow = digital_steer(0.6, 0.05, pressed);
        for (frame, value) in slow.iter().enumerate() {
            let same_moment = fast[(frame + 1) * 6 - 1];
            assert!(
                (value - same_moment).abs() < 0.02,
                "{value} y {same_moment}"
            );
        }
    }

    #[test]
    fn keyboard_steering_rate_option_changes_how_fast_it_turns_while_held() {
        let at = |values: &[f32], seconds: f32| values[(seconds / DT).round() as usize - 1];
        let held = |rate: SteerRate| digital_steer_with_rate(0.0, rate.factor(), 1.0, DT, |_| 1.0);
        let slow = held(SteerRate::Slow);
        let medium = held(SteerRate::Medium);
        let fast = held(SteerRate::Fast);

        // Al dejar pulsado, lento gira menos que medio, y medio menos que alto.
        assert!(at(&slow, 0.2) < at(&medium, 0.2));
        assert!(at(&medium, 0.2) < at(&fast, 0.2));

        // Alto llega al tope en unos 0.35 s; lento tarda más de medio segundo.
        assert_eq!(at(&fast, 0.35), 1.0);
        assert!(at(&slow, 0.35) < 0.8);
        assert_eq!(at(&slow, 0.8), 1.0);
    }

    #[test]
    fn lean_follows_how_hard_the_bike_turns() {
        let tuning = BikeTuning::default();
        let lean_at = |steer: f32| {
            let mut state = BikeState {
                speed: 50.0,
                steer,
                ..parked()
            };
            let input = BikeInput {
                throttle: 1.0,
                steer,
                ..default()
            };
            for _ in 0..240 {
                state = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
            }
            state.lean
        };
        let (gentle, full) = (lean_at(0.15), lean_at(1.0));
        assert!(
            gentle < 0.45 * tuning.max_lean,
            "poco giro, poca inclinación: {gentle}"
        );
        assert!(full > 0.9 * tuning.max_lean);
        // Y se tumba deprisa, pero no de golpe.
        let mut state = BikeState {
            speed: 50.0,
            steer: 1.0,
            ..parked()
        };
        let input = BikeInput {
            throttle: 1.0,
            steer: 1.0,
            ..default()
        };
        for _ in 0..6 {
            state = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
        }
        assert!(
            state.lean < 0.6 * tuning.max_lean,
            "a los 0,05 s: {}",
            state.lean
        );
    }

    #[test]
    fn stick_turns_little_near_the_center_and_fully_at_the_edge() {
        assert_eq!(stick_steer(0.05), 0.0);
        assert!(stick_steer(0.3) < 0.2);
        assert!(stick_steer(-0.3) > -0.2 && stick_steer(-0.3) < 0.0);
        assert_eq!(stick_steer(1.0), 1.0);
        assert_eq!(stick_steer(-1.0), -1.0);
        let mut last = 0.0;
        for i in 0..=20 {
            let value = stick_steer(i as f32 / 20.0);
            assert!(value >= last, "crece sin saltos");
            last = value;
        }
    }

    #[test]
    fn steering_left_turns_and_leans_left() {
        let moving = drive(full_throttle(), Surface::Asphalt, 3.0, parked());
        let input = BikeInput {
            throttle: 1.0,
            steer: 1.0,
            ..default()
        };
        let turning = drive(input, Surface::Asphalt, 0.5, moving);
        assert!(turning.heading > moving.heading);
        assert!(turning.lean > 0.0);
        assert!(turning.lean <= BikeTuning::default().max_lean);
    }

    #[test]
    fn turning_radius_grows_with_speed() {
        let tuning = BikeTuning::default();
        let radius_at = |speed: f32| {
            let state = BikeState {
                speed,
                steer: 1.0,
                ..parked()
            };
            let input = BikeInput {
                steer: 1.0,
                ..default()
            };
            let next = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
            next.speed / ((next.heading - state.heading) / DT)
        };
        assert!(radius_at(40.0) > radius_at(20.0));
        assert!(radius_at(20.0) > radius_at(5.0));
    }

    /// Una moto en el borde del muro izquierdo de la recta de meta, con la
    /// orientación dada.
    fn against_left_wall(heading: f32) -> (Track, BikeState) {
        let track = Track::default();
        // 100 m después de la meta, en plena recta hacia -Z.
        let position = track.offset_point(50, WALL_DISTANCE + 1.0);
        let state = BikeState {
            position,
            heading,
            speed: 40.0,
            ..parked()
        };
        (track, state)
    }

    #[test]
    fn head_on_wall_crash_pushes_back_and_kills_speed() {
        // Mirando a la izquierda (-X), de cara contra el muro.
        let (track, mut state) = against_left_wall(FRAC_PI_2);
        hit_walls(&mut state, &track);
        let distance = track.distance_to_centerline(state.position);
        assert!(distance <= WALL_DISTANCE - BIKE_HALF_WIDTH + 1e-3);
        assert!(state.speed < 40.0 * (1.0 - IMPACT_SPEED_LOSS) + 0.5);
    }

    #[test]
    fn rear_ending_slows_the_follower_down() {
        let mut leader = BikeState {
            position: Vec2::new(0.0, -1.0),
            speed: 30.0,
            ..parked()
        };
        let mut follower = BikeState {
            speed: 40.0,
            ..parked()
        };
        bump(&mut follower, &mut leader);
        assert!(follower.position.distance(leader.position) >= BIKE_SPACING - 1e-4);
        assert_eq!(follower.speed, 30.0);
        assert_eq!(leader.speed, 30.0);
    }

    #[test]
    fn uphill_slows_and_downhill_speeds_up() {
        let tuning = BikeTuning::default();
        let moving = drive(full_throttle(), Surface::Asphalt, 3.0, parked());
        let coast = BikeInput::default();
        let on = |grade| {
            let ground = Ground {
                grade,
                ..Ground::flat(Surface::Asphalt)
            };
            step(&moving, &coast, &ground, &tuning, DT).speed
        };
        assert!(on(0.1) < on(0.0));
        assert!(on(-0.1) > on(0.0));
    }

    #[test]
    fn fast_over_a_crest_the_bike_flies_and_lands() {
        // Subiendo a una loma.
        let crest = Ground {
            crest: 0.005,
            grade: 0.02,
            ..Ground::flat(Surface::Asphalt)
        };
        // Despacio, sigue pegada al suelo.
        let mut slow = BikeState {
            speed: 30.0,
            ..parked()
        };
        settle(&mut slow, &crest, 0.0, DT);
        assert!(slow.grounded);

        // A 250 km/h hace falta más que la gravedad para seguir la loma.
        let mut fast = BikeState {
            speed: 70.0,
            ..parked()
        };
        settle(&mut fast, &crest, -0.01, DT);
        assert!(!fast.grounded);
        let mut steps = 0;
        while !fast.grounded {
            settle(&mut fast, &Ground::flat(Surface::Asphalt), -1.0, DT);
            steps += 1;
            assert!(steps < 1000, "no aterriza nunca");
        }
        assert_eq!(fast.elevation, -1.0);
    }

    #[test]
    fn in_the_air_the_bike_barely_steers_and_cannot_accelerate() {
        let tuning = BikeTuning::default();
        let input = BikeInput {
            throttle: 1.0,
            steer: 1.0,
            ..default()
        };
        let turn = |grounded: bool| {
            let mut state = BikeState {
                speed: 50.0,
                grounded,
                ..parked()
            };
            for _ in 0..60 {
                state = step(&state, &input, &Ground::flat(Surface::Asphalt), &tuning, DT);
            }
            state
        };
        let (flying, riding) = (turn(false), turn(true));
        assert!(flying.heading > 0.0, "algo se corrige");
        assert!(flying.heading < 0.4 * riding.heading);
        assert!(flying.speed <= 50.0);
    }

    #[test]
    fn a_jump_does_not_hang_in_the_air() {
        // A 230 km/h por una loma que hace despegar, y luego cuesta abajo.
        let mut state = BikeState {
            speed: 64.0,
            ..parked()
        };
        let crest = Ground {
            crest: 0.005,
            grade: 0.03,
            ..Ground::flat(Surface::Asphalt)
        };
        settle(&mut state, &crest, 0.0, DT);
        assert!(!state.grounded);
        let downhill = Ground {
            grade: -0.08,
            ..Ground::flat(Surface::Asphalt)
        };
        let mut steps = 0;
        while !state.grounded {
            let road = -0.08 * 64.0 * steps as f32 * DT;
            settle(&mut state, &downhill, road, DT);
            steps += 1;
        }
        let seconds = steps as f32 * DT;
        assert!(seconds < 0.7, "vuela {seconds} s");
    }

    #[test]
    fn a_wheelie_accelerates_a_little_more() {
        let tuning = BikeTuning::default();
        let wheelie = BikeInput {
            wheelie: 1.0,
            ..full_throttle()
        };
        let lifted = drive(wheelie, Surface::Asphalt, 60.0, parked());
        // A la misma velocidad, con la rueda en el aire gana velocidad algo
        // más deprisa que con las dos en el suelo.
        let accel = |chassis: Chassis| {
            let state = BikeState {
                speed: 40.0,
                chassis,
                ..parked()
            };
            let next = step(
                &state,
                &wheelie,
                &Ground::flat(Surface::Asphalt),
                &tuning,
                DT,
            );
            (next.speed - state.speed) / DT
        };
        let (up, down) = (accel(lifted.chassis), accel(Chassis::default()));
        assert!(
            up > down * 1.08 && up < down * 1.2,
            "{up} frente a {down} m/s²"
        );
    }

    #[test]
    fn a_wheelie_on_the_straight_is_faster_but_hardly_turns() {
        let tuning = BikeTuning::default();
        let flat_out = drive(full_throttle(), Surface::Asphalt, 60.0, parked());
        let wheelie = BikeInput {
            wheelie: 1.0,
            ..full_throttle()
        };
        let lifted = drive(wheelie, Surface::Asphalt, 60.0, parked());
        assert!(lifted.chassis.wheelie > 0.25, "rueda en el aire");
        assert!(
            lifted.speed > flat_out.speed + 1.5,
            "{} frente a {}",
            lifted.speed,
            flat_out.speed
        );

        // A la misma velocidad, con la rueda en el aire se gira mucho menos.
        let turn = BikeInput {
            steer: 1.0,
            ..wheelie
        };
        let yaw_rate = |chassis: Chassis| {
            let state = BikeState {
                speed: 40.0,
                steer: 1.0,
                chassis,
                ..parked()
            };
            let next = step(&state, &turn, &Ground::flat(Surface::Asphalt), &tuning, DT);
            (next.heading - state.heading) / DT
        };
        assert!(yaw_rate(lifted.chassis) < 0.3 * yaw_rate(Chassis::default()));
    }

    #[test]
    fn braking_dives_and_accelerating_squats() {
        let moving = drive(full_throttle(), Surface::Asphalt, 4.0, parked());
        assert!(moving.accel > 2.0 && moving.chassis.squat > 0.0);
        let braking = BikeInput {
            brake: 1.0,
            ..default()
        };
        let braked = drive(braking, Surface::Asphalt, 0.5, moving);
        assert!(braked.accel < -20.0);
        assert!(braked.chassis.squat < -0.04, "morro hundido");
    }

    #[test]
    fn rolling_downhill_is_not_felt_as_acceleration() {
        let tuning = BikeTuning::default();
        let moving = drive(full_throttle(), Surface::Asphalt, 3.0, parked());
        // Bajando un 40 %: la gravedad empuja más de lo que frena el motor.
        let downhill = Ground {
            grade: -0.4,
            ..Ground::flat(Surface::Asphalt)
        };
        let next = step(&moving, &BikeInput::default(), &downhill, &tuning, DT);
        assert!(next.speed > moving.speed, "la cuesta empuja");
        assert!(next.accel < 0.0, "pero el piloto solo nota el freno motor");
    }

    #[test]
    fn landing_a_jump_compresses_the_suspension() {
        let tuning = BikeTuning::default();
        let mut state = BikeState {
            speed: 40.0,
            elevation: 3.0,
            grounded: false,
            ..parked()
        };
        let ground = Ground::flat(Surface::Asphalt);
        let mut deepest = 0.0f32;
        for _ in 0..240 {
            state = step(&state, &BikeInput::default(), &ground, &tuning, DT);
            settle(&mut state, &ground, 0.0, DT);
            deepest = deepest.min(state.chassis.heave);
        }
        assert!(state.grounded);
        assert!(deepest < -0.06, "comprime {deepest}");
        assert!(state.chassis.heave.abs() < 0.01, "y se recupera");
    }

    #[test]
    fn hitting_a_wall_at_an_angle_turns_the_bike_along_it() {
        // 30° contra el muro izquierdo (que queda hacia -X).
        let (track, mut state) = against_left_wall(0.5);
        let before = state.heading;
        for _ in 0..12 {
            hit_walls(&mut state, &track);
            state.position += forward(state.heading) * state.speed * DT;
        }
        assert!(
            state.heading < before * 0.2,
            "paralela al muro: {}",
            state.heading
        );
        assert!(state.speed > 15.0, "y sigue: {}", state.speed);
    }

    #[test]
    fn holding_full_lock_fast_sheds_speed_to_make_the_corner() {
        let tuning = BikeTuning::default();
        let fast = BikeState {
            speed: 70.0,
            ..parked()
        };
        let input = |steer| BikeInput {
            throttle: 1.0,
            steer,
            ..default()
        };
        let straight = drive(input(0.0), Surface::Asphalt, 2.0, fast);
        let turning = drive(input(1.0), Surface::Asphalt, 2.0, fast);
        assert!(turning.speed < straight.speed - 10.0);
        // Despacio no frena: solo gira.
        let slow = BikeState {
            speed: 15.0,
            ..parked()
        };
        let next = step(
            &slow,
            &input(1.0),
            &Ground::flat(Surface::Asphalt),
            &tuning,
            DT,
        );
        assert!(next.speed >= slow.speed);
    }

    /// Lo que pide un juego arcade: a fondo, sin frenar nunca y solo girando
    /// hacia la carretera, se da la vuelta sin tocar los muros.
    #[test]
    fn moderate_corners_flat_out_gain_speed() {
        let fast = BikeState {
            speed: 60.0,
            ..parked()
        };
        let input = |steer| BikeInput {
            throttle: 1.0,
            steer,
            ..default()
        };
        let straight = drive(input(0.0), Surface::Asphalt, 2.0, fast);
        // También al 80 % al que llega el teclado yendo deprisa.
        for steer in [0.3, 0.5, 0.65, 0.8] {
            let turning = drive(input(steer), Surface::Asphalt, 2.0, fast);
            assert!(
                turning.speed > straight.speed,
                "girando {steer}: {} frente a {} en recta",
                turning.speed,
                straight.speed
            );
        }
    }

    #[test]
    fn flat_out_steering_only_laps_without_touching_the_walls() {
        for circuit in &crate::circuits::CIRCUITS {
            flat_out_lap(&Track::new(&circuit.layout), circuit.name);
        }
    }

    fn flat_out_lap(track: &Track, name: &str) {
        let tuning = BikeTuning::default();
        let mut state = BikeState::parked_on(track, track.grid_pose(0, 0.0));
        let n = track.sample_count();
        let (mut progress, mut last) = (0i64, track.nearest_index(state.position) as i64);
        let mut on_grass = 0;
        let steps = (60.0 / DT) as usize;
        for _ in 0..steps {
            // Mira 18 m por delante de la línea central y gira hacia allí.
            let ahead = track.offset_point(track.nearest_index(state.position) + 9, 0.0);
            let angle = wrap_angle(heading_of(ahead - state.position) - state.heading);
            let input = BikeInput {
                throttle: 1.0,
                steer: (angle * 3.0).clamp(-1.0, 1.0),
                ..default()
            };
            state = advance(&state, &input, track, &tuning, DT);
            let distance = track.distance_to_centerline(state.position);
            assert!(
                distance < WALL_DISTANCE - BIKE_HALF_WIDTH - 0.01,
                "toca el muro de {name} en {:?} (muestra {}) a {:.0} km/h",
                state.position,
                track.nearest_index(state.position),
                state.speed * 3.6
            );
            if distance > track.half_width() {
                on_grass += 1;
            }
            let index = track.nearest_index(state.position) as i64;
            progress += (index - last + n as i64 * 3 / 2) % n as i64 - n as i64 / 2;
            last = index;
        }
        assert!(
            progress > n as i64,
            "da la vuelta a {name}: {progress} de {n}"
        );
        assert!(
            on_grass < steps / 20,
            "fuera del asfalto de {name} {on_grass} pasos"
        );
    }

    #[test]
    fn scraping_along_a_wall_keeps_most_of_the_speed() {
        // Casi paralelo al muro, apuntando un poco hacia él.
        let (track, mut state) = against_left_wall(0.1);
        hit_walls(&mut state, &track);
        assert!(state.speed > 35.0);
    }
}
