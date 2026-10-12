//! Pilotos rivales: siguen su carril a lo largo del circuito, frenan antes de
//! las curvas y aceleran a la salida. Si alcanzan a otra moto que va por su
//! trazada, se abren hacia el lado con sitio para adelantarla, y si no caben,
//! levantan el gas en vez de embestirla; luego vuelven a su carril. Conducen
//! con los mismos mandos ([`BikeInput`]) y la misma física que el jugador.

use bevy::prelude::*;

use super::{BikeInput, BikeState, BikeTuning, Race, move_toward};
use crate::geometry::{heading_of, wrap_angle};
use crate::track::{SAMPLE_SPACING, Track};

/// Distancia a la que mira por delante para apuntar: fija más la que recorre
/// en este tiempo. Mirar más lejos a más velocidad evita los eses.
const LOOK_AHEAD_DISTANCE: f32 = 8.0;
const LOOK_AHEAD_TIME: f32 = 0.4;
/// Fracción del freno con la que planea las frenadas, para llegar con margen.
const PLANNED_BRAKING: f32 = 0.6;
/// Metros extra que revisa por delante además de su distancia de frenado.
const EXTRA_HORIZON: f32 = 25.0;
/// Puntos a cada lado con los que suaviza la curvatura del trazado.
const CURVATURE_WINDOW: usize = 3;
/// Ve a las motos de delante a esta distancia más lo que recorre en este
/// tiempo (m, s).
const TRAFFIC_DISTANCE: f32 = 6.0;
const TRAFFIC_TIME: f32 = 1.6;
/// Separación de lado a lado para pasar junto a otra moto (m), y lo que deja
/// hasta el borde del asfalto.
const CLEARANCE: f32 = 2.1;
const EDGE_MARGIN: f32 = 1.0;
/// Lo deprisa que cambia de trazada (m/s), más cuanto más deprisa va.
const LINE_RATE: f32 = 2.0;
const LINE_RATE_PER_SPEED: f32 = 0.03;
/// Si no puede pasar, se queda a esta distancia de la de delante (m).
const FOLLOW_GAP: f32 = 4.0;

/// Un rival controlado por la máquina.
#[derive(Component, Clone, Copy, Debug)]
pub struct Rival {
    /// Fracción de la punta y del agarre de la moto que se atreve a usar (0..1].
    pub skill: f32,
    /// Distancia a la izquierda de la línea central por la que va (m).
    pub lane: f32,
}

/// Por dónde va un rival ahora mismo (m a la izquierda de la línea central):
/// su carril o, para adelantar, a un lado.
#[derive(Component, Clone, Copy, Debug)]
pub struct Line(pub f32);

/// Otra moto de la carrera, como la ve un rival.
#[derive(Clone, Copy, Debug)]
pub struct Traffic {
    pub position: Vec2,
    pub speed: f32,
}

#[allow(clippy::type_complexity)]
pub fn drive_rivals(
    time: Res<Time>,
    race: Res<Race>,
    tuning: Res<BikeTuning>,
    track: Res<Track>,
    bikes: Query<(Entity, &BikeState)>,
    mut rivals: Query<(
        Entity,
        &Rival,
        &BikeState,
        &mut Line,
        &mut BikeInput,
        &super::CrashState,
    )>,
) {
    let traffic: Vec<(Entity, Traffic)> = bikes
        .iter()
        .map(|(entity, state)| {
            let traffic = Traffic {
                position: state.position,
                speed: state.speed,
            };
            (entity, traffic)
        })
        .collect();
    for (entity, rival, state, mut line, mut input, crash) in &mut rivals {
        // Down on the track, the rider lets go of everything.
        if !race.started || crash.is_down() {
            *input = BikeInput::default();
            continue;
        }
        let others: Vec<Traffic> = traffic
            .iter()
            .filter(|(other, _)| *other != entity)
            .map(|(_, traffic)| *traffic)
            .collect();
        let (target, limit) = plan_line(rival, line.0, state, &track, &others);
        let rate = LINE_RATE + LINE_RATE_PER_SPEED * state.speed.abs();
        line.0 = move_toward(line.0, target, rate * time.delta_secs());
        *input = decide(rival, line.0, limit, state, &track, &tuning);
    }
}

/// Hacia qué trazada ir con el tráfico que tiene delante, y a qué velocidad
/// como mucho. Si su carril está libre, vuelve a él; si no, sigue por donde
/// va mientras esté libre; si no, se abre al hueco libre más cercano junto a
/// las motos que le estorban. Si no hay hueco, sigue a la de delante.
pub fn plan_line(
    rival: &Rival,
    line: f32,
    state: &BikeState,
    track: &Track,
    traffic: &[Traffic],
) -> (f32, f32) {
    let n = track.sample_count() as i64;
    let here = track.nearest_index(state.position) as i64;
    let speed = state.speed.max(0.0);
    let horizon = TRAFFIC_DISTANCE + speed * TRAFFIC_TIME;
    // Las motos que tiene delante (o al lado) y a las que se acerca: cuánto
    // por delante, por dónde van y a qué velocidad.
    let ahead: Vec<(f32, f32, f32)> = traffic
        .iter()
        .filter_map(|other| {
            let index = track.nearest_index(other.position) as i64;
            let gap = ((index - here + n + n / 2) % n - n / 2) as f32 * SAMPLE_SPACING;
            let closing = speed - other.speed;
            (gap > -SAMPLE_SPACING && gap < horizon && (closing > 0.5 || gap < TRAFFIC_DISTANCE))
                .then(|| (gap, track.nearest(other.position).lateral, other.speed))
        })
        .collect();
    let limit = track.half_width() - EDGE_MARGIN;
    let free = |lateral: f32| {
        lateral.abs() <= limit
            && ahead
                .iter()
                .all(|&(_, other, _)| (other - lateral).abs() >= CLEARANCE)
    };
    if free(rival.lane) {
        return (rival.lane, f32::INFINITY);
    }
    if free(line) {
        return (line, f32::INFINITY);
    }
    let gap = ahead
        .iter()
        .flat_map(|&(_, other, _)| [other + CLEARANCE, other - CLEARANCE])
        .filter(|&lateral| free(lateral))
        .min_by(|a, b| (a - line).abs().total_cmp(&(b - line).abs()));
    if let Some(gap) = gap {
        return (gap, f32::INFINITY);
    }
    // Sin hueco: se pone a la velocidad de la que tiene delante.
    let leader = ahead
        .iter()
        .filter(|&&(gap, other, _)| gap > 0.0 && (other - line).abs() < CLEARANCE)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match leader {
        Some(&(gap, _, leader_speed)) => {
            (line, leader_speed.max(0.0) + (gap - FOLLOW_GAP).max(0.0))
        }
        None => (line, f32::INFINITY),
    }
}

/// Lo que hace un rival en este paso, yendo por `line` (m a la izquierda de
/// la línea central) sin pasar de `limit` (m/s). Es una función pura: se
/// puede probar sin ECS y da siempre lo mismo para la misma situación.
pub fn decide(
    rival: &Rival,
    line: f32,
    limit: f32,
    state: &BikeState,
    track: &Track,
    tuning: &BikeTuning,
) -> BikeInput {
    let here = track.nearest_index(state.position);
    let speed = state.speed.max(0.0);

    // Dirección: persecución pura de un punto de su carril por delante. La
    // curvatura que lleva hasta él, dividida entre la máxima que permite la
    // física a esta velocidad, es la dirección a aplicar.
    let ahead = ((LOOK_AHEAD_DISTANCE + speed * LOOK_AHEAD_TIME) / SAMPLE_SPACING).round() as usize;
    let to_target = track.offset_point(here + ahead, line) - state.position;
    let angle = wrap_angle(heading_of(to_target) - state.heading);
    let wanted_curvature = 2.0 * angle.sin() / to_target.length().max(1.0);
    let max_curvature =
        (1.0 / tuning.min_turn_radius).min(tuning.max_lateral_accel / speed.max(0.1).powi(2));
    let steer = (wanted_curvature / max_curvature).clamp(-1.0, 1.0);

    // Velocidad: la más baja que exijan las curvas de delante, contando con
    // que le dé tiempo a frenar hasta ellas.
    let grip = tuning.max_lateral_accel * rival.skill;
    let braking = tuning.brake_decel * PLANNED_BRAKING;
    let horizon = speed * speed / (2.0 * braking) + EXTRA_HORIZON;
    let mut target_speed = (tuning.max_speed * rival.skill).min(limit);
    for step in 0..(horizon / SAMPLE_SPACING) as usize {
        let curvature = track.smoothed_curvature(here + step, CURVATURE_WINDOW);
        if curvature < 1e-4 {
            continue;
        }
        let corner_speed = (grip / curvature).sqrt();
        let distance = step as f32 * SAMPLE_SPACING;
        target_speed = target_speed.min((corner_speed.powi(2) + 2.0 * braking * distance).sqrt());
    }

    BikeInput {
        throttle: ((target_speed - speed) / 2.0).clamp(0.0, 1.0),
        brake: ((speed - target_speed - 1.0) / 3.0).clamp(0.0, 1.0),
        steer,
        steer_direct: 0.0,
        // Los rivales no hacen caballitos: van a por el tiempo.
        wheelie: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bike::crash::{CrashState, fall, rejoin_pose, slide};
    use crate::bike::{BikeState, Body, Knock, advance};

    #[test]
    fn rival_laps_the_track_without_leaving_it() {
        for circuit in &crate::circuits::CIRCUITS {
            let track = Track::new(&circuit.layout);
            let tuning = BikeTuning::default();
            let rival = Rival {
                skill: 0.95,
                lane: 2.0,
            };
            let mut state = BikeState::parked_on(&track, track.grid_pose(0, rival.lane));
            let dt = 1.0 / 120.0;
            let n = track.sample_count() as i64;
            let mut progress = 0;
            let mut last_index = track.nearest_index(state.position) as i64;
            for _ in 0..(90.0 / dt) as usize {
                let input = decide(&rival, rival.lane, f32::INFINITY, &state, &track, &tuning);
                state = advance(&state, &input, &track, &tuning, dt);
                assert!(
                    track.distance_to_centerline(state.position) < 8.5,
                    "se ha salido de {} en {:?}",
                    circuit.name,
                    state.position
                );
                // Avance con signo, teniendo en cuenta que el índice da la vuelta.
                let index = track.nearest_index(state.position) as i64;
                progress += (index - last_index + n + n / 2) % n - n / 2;
                last_index = index;
            }
            assert!(
                progress > n,
                "debería dar al menos una vuelta a {} en 90 s",
                circuit.name
            );
        }
    }

    #[test]
    fn rival_goes_round_a_stopped_bike_in_its_lane() {
        let track = Track::default();
        let tuning = BikeTuning::default();
        let rival = Rival {
            skill: 0.95,
            lane: 0.0,
        };
        let dt = 1.0 / 120.0;
        let mut state = BikeState::parked_on(&track, track.grid_pose(5, rival.lane));
        // Una moto parada en su carril, 50 m por delante.
        let mut stopped = BikeState::parked_on(&track, track.grid_pose(0, rival.lane));
        let mut line = Line(rival.lane);
        let (mut knock, mut stopped_knock) = (Knock::default(), Knock::default());
        let n = track.sample_count() as i64;
        let gap = |a: &BikeState, b: &BikeState| {
            let (a, b) = (
                track.nearest_index(a.position) as i64,
                track.nearest_index(b.position) as i64,
            );
            (b - a + n + n / 2) % n - n / 2
        };
        let mut closest = f32::MAX;
        for _ in 0..(8.0 / dt) as usize {
            let traffic = [Traffic {
                position: stopped.position,
                speed: stopped.speed,
            }];
            let (target, limit) = plan_line(&rival, line.0, &state, &track, &traffic);
            let rate = LINE_RATE + LINE_RATE_PER_SPEED * state.speed;
            line.0 = move_toward(line.0, target, rate * dt);
            let input = decide(&rival, line.0, limit, &state, &track, &tuning);
            state = advance(&state, &input, &track, &tuning, dt);
            crate::bike::carry(&mut state, &mut knock, &track, dt);
            let contact = crate::bike::collide(
                Body {
                    state: &mut state,
                    knock: &mut knock,
                    down: false,
                },
                Body {
                    state: &mut stopped,
                    knock: &mut stopped_knock,
                    down: false,
                },
            );
            assert!(contact.is_none(), "no la toca: {contact:?}");
            closest = closest.min(state.position.distance(stopped.position));
        }
        assert!(gap(&state, &stopped) < -10, "la ha adelantado");
        assert!(closest > 1.5, "sin tocarla: {closest} m");
        assert!(state.speed > 30.0, "y sigue lanzado: {}", state.speed);
    }

    /// One rival of the pack in [`a_pack_of_rivals_races_without_wrecking_each_other`].
    struct Racer {
        rival: Rival,
        line: Line,
        state: BikeState,
        knock: Knock,
        crash: CrashState,
    }

    #[test]
    fn a_pack_of_rivals_races_without_wrecking_each_other() {
        let track = Track::default();
        let tuning = BikeTuning::default();
        let dt = 1.0 / 120.0;
        let mut pack: Vec<Racer> = crate::bike::GRID
            .iter()
            .enumerate()
            .filter_map(|(slot, rival)| {
                rival.map(|rival| Racer {
                    rival,
                    line: Line(rival.lane),
                    state: BikeState::parked_on(&track, track.grid_slot(slot)),
                    knock: Knock::default(),
                    crash: CrashState::default(),
                })
            })
            .collect();
        let n = track.sample_count() as i64;
        let mut last: Vec<i64> = pack
            .iter()
            .map(|racer| track.nearest_index(racer.state.position) as i64)
            .collect();
        let mut progress = vec![0; pack.len()];
        let mut falls = 0;
        for _ in 0..(90.0 / dt) as usize {
            let traffic: Vec<Traffic> = pack
                .iter()
                .map(|racer| Traffic {
                    position: racer.state.position,
                    speed: racer.state.speed,
                })
                .collect();
            for (i, racer) in pack.iter_mut().enumerate() {
                if racer.crash.is_down() {
                    slide(
                        &mut racer.state,
                        &mut racer.knock,
                        &mut racer.crash,
                        &track,
                        dt,
                    );
                    if racer.crash.is_over() {
                        let (position, heading, lateral) =
                            rejoin_pose(&track, racer.state.position, &[]);
                        racer.state = BikeState::parked_on(&track, (position, heading));
                        racer.knock = Knock::default();
                        racer.crash = CrashState::default();
                        racer.line.0 = lateral;
                    }
                    continue;
                }
                let others: Vec<Traffic> = traffic
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, other)| *other)
                    .collect();
                let (target, limit) =
                    plan_line(&racer.rival, racer.line.0, &racer.state, &track, &others);
                let rate = LINE_RATE + LINE_RATE_PER_SPEED * racer.state.speed.abs();
                racer.line.0 = move_toward(racer.line.0, target, rate * dt);
                let input = decide(
                    &racer.rival,
                    racer.line.0,
                    limit,
                    &racer.state,
                    &track,
                    &tuning,
                );
                racer.state = advance(&racer.state, &input, &track, &tuning, dt);
                crate::bike::carry(&mut racer.state, &mut racer.knock, &track, dt);
            }
            for j in 1..pack.len() {
                let (before, after) = pack.split_at_mut(j);
                let b = &mut after[0];
                for a in before.iter_mut() {
                    let contact = crate::bike::collide(
                        Body {
                            state: &mut a.state,
                            knock: &mut a.knock,
                            down: a.crash.is_down(),
                        },
                        Body {
                            state: &mut b.state,
                            knock: &mut b.knock,
                            down: b.crash.is_down(),
                        },
                    );
                    let Some(contact) = contact else {
                        continue;
                    };
                    if contact.a.falls {
                        a.crash = fall(&mut a.state, &mut a.knock, &contact.a);
                        falls += 1;
                    }
                    if contact.b.falls {
                        b.crash = fall(&mut b.state, &mut b.knock, &contact.b);
                        falls += 1;
                    }
                }
            }
            for (i, racer) in pack.iter().enumerate() {
                assert!(
                    track.distance_to_centerline(racer.state.position) < 11.5,
                    "el rival {i} se ha salido en {:?}",
                    racer.state.position
                );
                let index = track.nearest_index(racer.state.position) as i64;
                progress[i] += (index - last[i] + n + n / 2) % n - n / 2;
                last[i] = index;
            }
        }
        assert!(falls <= 2, "se caen {falls} veces");
        for (i, progress) in progress.iter().enumerate() {
            assert!(
                *progress > n,
                "el rival {i} no da la vuelta: {progress} de {n}"
            );
        }
    }

    #[test]
    fn jumps_on_the_track_are_short() {
        for circuit in &crate::circuits::CIRCUITS {
            let track = Track::new(&circuit.layout);
            let tuning = BikeTuning::default();
            let rival = Rival {
                skill: 1.0,
                lane: 0.0,
            };
            let mut state = BikeState::parked_on(&track, track.grid_pose(0, rival.lane));
            let dt = 1.0 / 120.0;
            let (mut flights, mut air) = (0, 0usize);
            let name = circuit.name;
            for _ in 0..(70.0 / dt) as usize {
                let input = decide(&rival, rival.lane, f32::INFINITY, &state, &track, &tuning);
                state = advance(&state, &input, &track, &tuning, dt);
                if !state.grounded {
                    air += 1;
                    let height = state.elevation - track.road_height(state.position);
                    assert!(height < 1.3, "vuela muy alto en {name}: {height} m");
                } else if air > 0 {
                    flights += 1;
                    assert!(
                        air as f32 * dt < 0.75,
                        "vuela {} s en {name}",
                        air as f32 * dt
                    );
                    air = 0;
                }
            }
            assert!(
                flights >= 2,
                "a fondo, en los cambios de rasante de {name} se vuela"
            );
        }
    }
}
