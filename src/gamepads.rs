//! Mandos: gilrs conectado a la entrada de Bevy, como hace `bevy::gilrs`, y
//! una búsqueda de mandos de nuevo mientras no se vea ninguno.
//!
//! En macOS gilrs busca los mandos ya enchufados en otro hilo, después de
//! arrancar, y a veces se le escapa alguno: había que desenchufarlo y volverlo
//! a enchufar. Mientras no haya ningún mando, el juego crea cada poco un gilrs
//! nuevo, que los vuelve a buscar todos, como si se reconectaran. Los que se
//! enchufan con el juego ya en marcha llegan igual que siempre.
//!
//! Para saber qué pasa con un mando que no responde, el juego escribe en la
//! terminal lo que ve de cada uno: cuando se conecta, lo primero que manda y
//! un aviso si en un rato no manda nada. Desde el menú de pausa se puede
//! pedir que los vuelva a buscar.
//!
//! En macOS, además, los mandos que conoce el sistema (PlayStation, Xbox,
//! Switch Pro, MFi) se leen con GameController, el framework de Apple
//! ([`apple`]): con gilrs, el DualSense se ve conectado pero no manda nada.
//! Esos mandos ya no los cuenta gilrs ([`ClaimedPads`]).

#[cfg(target_os = "macos")]
mod apple;

use bevy::input::InputSystems;
use bevy::input::gamepad::{
    GamepadAxis, GamepadButton, GamepadConnection, GamepadConnectionEvent,
    RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::platform::cell::SyncCell;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use gilrs::ev::filter::{Filter, axis_dpad_to_button};
use gilrs::{EventType, GamepadId};

/// Cada cuánto se vuelve a buscar mandos si no hay ninguno (s de reloj, vaya
/// el juego como vaya), y cuántas veces como mucho: en macOS cada búsqueda
/// deja un hilo de gilrs, así que no se busca para siempre.
const SEARCH_EVERY: f32 = 2.5;
const SEARCHES: u32 = 8;
/// Tras cuánto tiempo conectado sin mandar nada se avisa de que un mando no
/// responde (s).
const SILENT_WARNING: f32 = 10.0;

pub struct GamepadsPlugin;

impl Plugin for GamepadsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gamepads::new())
            .init_resource::<ClaimedPads>()
            .add_systems(PreUpdate, read_gamepads.before(InputSystems));
        #[cfg(target_os = "macos")]
        app.init_non_send::<apple::ApplePads>().add_systems(
            PreUpdate,
            apple::read_apple_pads
                .before(read_gamepads)
                .before(InputSystems),
        );
    }
}

/// Los mandos que lee otro sistema (en macOS, GameController), por su
/// nombre: gilrs no los cuenta, para que no salgan dos veces.
#[derive(Resource, Default)]
pub struct ClaimedPads(pub Vec<String>);

impl ClaimedPads {
    /// Si el mando de gilrs que se llama `name` ya lo lee otro.
    pub fn has(&self, name: &str) -> bool {
        let name_norm = name.to_ascii_lowercase();
        self.0.iter().any(|claimed| {
            let claimed_norm = claimed.to_ascii_lowercase();
            if claimed_norm == name_norm
                || claimed_norm.contains(&name_norm)
                || name_norm.contains(&claimed_norm)
            {
                return true;
            }
            // En macOS, GameController usa el vendorName ("DualSense Wireless Controller",
            // "Xbox Wireless Controller", "DUALSHOCK 4 Wireless Controller", etc.)
            // mientras que gilrs/IOKit suele llamarlo "Wireless Controller", "Sony Interactive
            // Entertainment Wireless Controller", "Microsoft Xbox Controller", etc.
            let is_generic_wireless = name_norm == "wireless controller"
                || name_norm == "wireless gamepad"
                || name_norm.starts_with("controller");
            let claimed_is_wireless = claimed_norm.contains("controller")
                || claimed_norm.contains("dualsense")
                || claimed_norm.contains("dualshock")
                || claimed_norm.contains("xbox");
            if is_generic_wireless && claimed_is_wireless {
                return true;
            }
            let is_sony = claimed_norm.contains("dualsense")
                || claimed_norm.contains("dualshock")
                || claimed_norm.contains("playstation")
                || claimed_norm.contains("sony");
            let gilrs_sony = name_norm.contains("sony")
                || name_norm.contains("playstation")
                || name_norm.contains("wireless controller");
            if is_sony && gilrs_sony {
                return true;
            }
            let is_xbox = claimed_norm.contains("xbox") || claimed_norm.contains("gulikit");
            let gilrs_xbox = name_norm.contains("xbox")
                || name_norm.contains("xinput")
                || name_norm.contains("gulikit");
            if is_xbox && gilrs_xbox {
                return true;
            }
            let is_nintendo = claimed_norm.contains("switch") || claimed_norm.contains("nintendo");
            let gilrs_nintendo = name_norm.contains("switch") || name_norm.contains("nintendo");
            if is_nintendo && gilrs_nintendo {
                return true;
            }
            false
        })
    }
}

/// Si el juego sigue buscando mandos porque no ve ninguno.
#[derive(Resource)]
pub struct Gamepads {
    gilrs: Option<SyncCell<gilrs::Gilrs>>,
    /// La entidad de cada mando que ha visto este gilrs.
    entities: HashMap<GamepadId, Entity>,
    /// Desde cuándo no hay ningún mando (s), cuántas veces se ha vuelto a
    /// buscar y si ya se ha dejado de buscar.
    without: f32,
    searches: u32,
    gave_up: bool,
    /// De cada mando conectado que todavía no ha mandado nada: su nombre y
    /// cuánto lleva conectado (s).
    silent: HashMap<Entity, (String, f32)>,
    /// Si se ha pedido volver a buscar los mandos.
    search_requested: bool,
}

impl Gamepads {
    fn new() -> Self {
        Self {
            gilrs: start(),
            entities: HashMap::default(),
            without: 0.0,
            searches: 0,
            gave_up: false,
            silent: HashMap::default(),
            search_requested: false,
        }
    }

    /// Vuelve a buscar los mandos, como si se reconectaran todos.
    pub fn search_again(&mut self) {
        self.search_requested = true;
    }

    /// Si todavía va a volver a buscar mandos.
    pub fn searching(&self) -> bool {
        self.gilrs.is_some() && !self.gave_up
    }
}

fn start() -> Option<SyncCell<gilrs::Gilrs>> {
    match gilrs::GilrsBuilder::new()
        .with_default_filters(false)
        .set_update_state(false)
        .build()
    {
        Ok(gilrs) => Some(SyncCell::new(gilrs)),
        Err(error) => {
            error!("No se pueden leer los mandos: {error}");
            None
        }
    }
}

/// Lleva a Bevy lo que ha pasado en los mandos desde el fotograma anterior y,
/// si no hay ninguno, vuelve a buscarlos de vez en cuando.
#[allow(clippy::too_many_arguments)]
fn read_gamepads(
    mut commands: Commands,
    time: Res<Time<Real>>,
    claimed: Res<ClaimedPads>,
    mut pads: ResMut<Gamepads>,
    mut events: MessageWriter<RawGamepadEvent>,
    mut connections: MessageWriter<GamepadConnectionEvent>,
    mut buttons: MessageWriter<RawGamepadButtonChangedEvent>,
    mut axes: MessageWriter<RawGamepadAxisChangedEvent>,
) {
    let pads = pads.as_mut();
    if std::mem::take(&mut pads.search_requested) {
        info!("Se vuelven a buscar los mandos");
        restart(pads, &mut commands);
        (pads.searches, pads.gave_up, pads.without) = (0, false, 0.0);
    }
    let Some(gilrs) = pads.gilrs.as_mut().map(SyncCell::get) else {
        return;
    };
    // Los que ya lee otro sistema dejan de contar aquí.
    let taken: Vec<GamepadId> = pads
        .entities
        .keys()
        .copied()
        .filter(|&id| claimed.has(gilrs.gamepad(id).name()))
        .collect();
    for id in taken {
        if let Some(entity) = pads.entities.remove(&id) {
            info!(
                "El mando {} se lee con GameController",
                gilrs.gamepad(id).name()
            );
            pads.silent.remove(&entity);
            let event = GamepadConnectionEvent::new(entity, GamepadConnection::Disconnected);
            events.write(event.clone().into());
            connections.write(event);
            commands.entity(entity).despawn();
        }
    }
    // Los que gilrs ya conocía al crearse (en Linux y Windows, los
    // enchufados), y los que deja de leer otro sistema.
    let mut connected: Vec<_> = gilrs
        .gamepads()
        .map(|(id, _)| id)
        .filter(|id| !pads.entities.contains_key(id))
        .collect();
    let mut changes = Vec::new();
    while let Some(event) = gilrs.next_event().filter_ev(&axis_dpad_to_button, gilrs) {
        gilrs.update(&event);
        match event.event {
            EventType::Connected => connected.push(event.id),
            _ => changes.push(event),
        }
    }
    gilrs.inc();

    for id in connected {
        if claimed.has(gilrs.gamepad(id).name()) || pads.entities.contains_key(&id) {
            continue;
        }
        let entity = *pads
            .entities
            .entry(id)
            .or_insert_with(|| commands.spawn_empty().id());
        let pad = gilrs.gamepad(id);
        // En la terminal, para saber qué ve el juego si un mando no responde.
        info!(
            "Mando: {} (fabricante {:04x}, producto {:04x}, botones {})",
            pad.name(),
            pad.vendor_id().unwrap_or(0),
            pad.product_id().unwrap_or(0),
            match pad.mapping_source() {
                gilrs::MappingSource::SdlMappings => "según la base de datos de SDL",
                gilrs::MappingSource::Driver => "según el sistema",
                gilrs::MappingSource::None => "sin asignar: puede que no respondan bien",
            }
        );
        let event = GamepadConnectionEvent::new(
            entity,
            GamepadConnection::Connected {
                name: pad.name().to_string(),
                vendor_id: pad.vendor_id(),
                product_id: pad.product_id(),
            },
        );
        events.write(event.clone().into());
        connections.write(event);
        pads.silent.insert(entity, (pad.name().to_string(), 0.0));
    }
    for event in changes {
        let Some(&entity) = pads.entities.get(&event.id) else {
            continue;
        };
        if event.event != EventType::Disconnected
            && let Some((name, _)) = pads.silent.remove(&entity)
        {
            // Lo primero que manda, tal cual: si llega como desconocido, el
            // juego no lo puede usar.
            info!("El mando {name} responde: {:?}", event.event);
        }
        match event.event {
            EventType::Disconnected => {
                pads.silent.remove(&entity);
                let event = GamepadConnectionEvent::new(entity, GamepadConnection::Disconnected);
                events.write(event.clone().into());
                connections.write(event);
            }
            EventType::ButtonChanged(button, value, _) => {
                if let Some(button) = convert_button(button) {
                    let event = RawGamepadButtonChangedEvent::new(entity, button, value);
                    events.write(event.into());
                    buttons.write(event);
                }
            }
            EventType::AxisChanged(axis, value, _) => {
                if let Some(axis) = convert_axis(axis) {
                    let event = RawGamepadAxisChangedEvent::new(entity, axis, value);
                    events.write(event.into());
                    axes.write(event);
                }
            }
            _ => (),
        }
    }

    // Los que siguen sin mandar nada, al cabo de un rato, se avisan.
    let dt = time.delta_secs();
    for (name, connected) in pads.silent.values_mut() {
        let before = *connected;
        *connected += dt;
        if before < SILENT_WARNING && *connected >= SILENT_WARNING {
            warn!(
                "El mando {name} está conectado pero no ha mandado nada en \
                 {SILENT_WARNING} s. Si lo estabas usando, prueba en el menú de pausa \
                 Depuración > Volver a buscar mandos, o reconéctalo"
            );
        }
    }

    // Sin ningún mando, cada poco se crea un gilrs nuevo, que busca de nuevo.
    if gilrs.gamepads().next().is_some() || !claimed.0.is_empty() {
        pads.without = 0.0;
        return;
    }
    pads.without += dt;
    if pads.without < SEARCH_EVERY || !pads.searching() {
        return;
    }
    pads.without = 0.0;
    if pads.searches == SEARCHES {
        pads.gave_up = true;
        warn!(
            "No aparece ningún mando. Si hay uno enchufado, puede que esté en un modo \
             que el sistema no reconoce como mando (en macOS, XInput): prueba otro \
             modo o reconéctalo"
        );
    } else {
        pads.searches += 1;
        info!(
            "Sin mandos: se vuelven a buscar ({}/{SEARCHES})",
            pads.searches
        );
        restart(pads, &mut commands);
    }
}

/// Crea un gilrs nuevo, que busca los mandos desde cero.
fn restart(pads: &mut Gamepads, commands: &mut Commands) {
    // Las entidades de los mandos que había: el gilrs nuevo los numera desde
    // cero, y los que sigan conectados vuelven a aparecer.
    for (_, entity) in pads.entities.drain() {
        commands.entity(entity).despawn();
    }
    pads.silent.clear();
    pads.gilrs = start();
}

fn convert_button(button: gilrs::Button) -> Option<GamepadButton> {
    Some(match button {
        gilrs::Button::South => GamepadButton::South,
        gilrs::Button::East => GamepadButton::East,
        gilrs::Button::North => GamepadButton::North,
        gilrs::Button::West => GamepadButton::West,
        gilrs::Button::C => GamepadButton::C,
        gilrs::Button::Z => GamepadButton::Z,
        gilrs::Button::LeftTrigger => GamepadButton::LeftTrigger,
        gilrs::Button::LeftTrigger2 => GamepadButton::LeftTrigger2,
        gilrs::Button::RightTrigger => GamepadButton::RightTrigger,
        gilrs::Button::RightTrigger2 => GamepadButton::RightTrigger2,
        gilrs::Button::Select => GamepadButton::Select,
        gilrs::Button::Start => GamepadButton::Start,
        gilrs::Button::Mode => GamepadButton::Mode,
        gilrs::Button::LeftThumb => GamepadButton::LeftThumb,
        gilrs::Button::RightThumb => GamepadButton::RightThumb,
        gilrs::Button::DPadUp => GamepadButton::DPadUp,
        gilrs::Button::DPadDown => GamepadButton::DPadDown,
        gilrs::Button::DPadLeft => GamepadButton::DPadLeft,
        gilrs::Button::DPadRight => GamepadButton::DPadRight,
        gilrs::Button::Unknown => return None,
    })
}

fn convert_axis(axis: gilrs::Axis) -> Option<GamepadAxis> {
    Some(match axis {
        gilrs::Axis::LeftStickX => GamepadAxis::LeftStickX,
        gilrs::Axis::LeftStickY => GamepadAxis::LeftStickY,
        gilrs::Axis::LeftZ => GamepadAxis::LeftZ,
        gilrs::Axis::RightStickX => GamepadAxis::RightStickX,
        gilrs::Axis::RightStickY => GamepadAxis::RightStickY,
        gilrs::Axis::RightZ => GamepadAxis::RightZ,
        // La cruceta llega como botones (filtro `axis_dpad_to_button`).
        gilrs::Axis::Unknown | gilrs::Axis::DPadX | gilrs::Axis::DPadY => return None,
    })
}
