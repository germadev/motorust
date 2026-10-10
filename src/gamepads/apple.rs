//! Mandos en macOS con GameController, el framework de Apple: el que mejor
//! conoce los mandos de PlayStation (DualSense, DualShock 4), Xbox, Switch
//! Pro y MFi. macOS pone el DualSense en un modo en el que gilrs lo ve
//! conectado pero no le llega ni un botón; con GameController sí.
//!
//! Cada fotograma se mira qué mandos hay y cómo están sus botones y sus
//! sticks, y lo que cambia se lleva a Bevy como si viniera de gilrs. Los
//! mandos que lee GameController no los cuenta gilrs ([`ClaimedPads`]).

use bevy::input::gamepad::{
    GamepadAxis, GamepadButton, GamepadConnection, GamepadConnectionEvent,
    RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::prelude::*;
use objc2::rc::Retained;

use super::ClaimedPads;
use objc2_game_controller::{
    GCController, GCControllerButtonInput, GCControllerDirectionPad, GCDevice, GCExtendedGamepad,
};

/// Los botones, en el orden en que se leen: con los nombres de Bevy, que son
/// los de su sitio en el mando (A, abajo; B, a la derecha...).
const BUTTONS: [GamepadButton; 17] = [
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::West,
    GamepadButton::North,
    GamepadButton::LeftTrigger,
    GamepadButton::RightTrigger,
    GamepadButton::LeftTrigger2,
    GamepadButton::RightTrigger2,
    GamepadButton::Start,
    GamepadButton::Select,
    GamepadButton::Mode,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
];
const AXES: [GamepadAxis; 4] = [
    GamepadAxis::LeftStickX,
    GamepadAxis::LeftStickY,
    GamepadAxis::RightStickX,
    GamepadAxis::RightStickY,
];
/// Cambios más pequeños que esto no se mandan.
const CHANGE: f32 = 1e-3;

/// Los mandos que lee GameController. Solo se puede tocar desde el hilo
/// principal, así que va como recurso no `Send`.
#[derive(Default)]
pub struct ApplePads {
    pads: Vec<Pad>,
}

struct Pad {
    controller: Retained<GCController>,
    entity: Entity,
    name: String,
    buttons: [f32; BUTTONS.len()],
    axes: [f32; AXES.len()],
}

/// Los mensajes de los mandos, como los manda `bevy::gilrs`.
#[derive(bevy::ecs::system::SystemParam)]
pub struct PadMessages<'w> {
    events: MessageWriter<'w, RawGamepadEvent>,
    connections: MessageWriter<'w, GamepadConnectionEvent>,
    buttons: MessageWriter<'w, RawGamepadButtonChangedEvent>,
    axes: MessageWriter<'w, RawGamepadAxisChangedEvent>,
}

impl PadMessages<'_> {
    fn connection(&mut self, event: GamepadConnectionEvent) {
        self.events.write(event.clone().into());
        self.connections.write(event);
    }
}

/// Mira qué mandos hay y lleva a Bevy lo que ha cambiado en cada uno.
pub fn read_apple_pads(
    mut apple: NonSendMut<ApplePads>,
    mut names: ResMut<ClaimedPads>,
    mut commands: Commands,
    mut messages: PadMessages,
) {
    // SAFETY: GameController se usa desde el hilo principal (el recurso no
    // es `Send`, así que Bevy corre este sistema en él).
    let controllers = unsafe { GCController::controllers() };
    let same = |a: &Retained<GCController>, b: &Retained<GCController>| {
        Retained::as_ptr(a) == Retained::as_ptr(b)
    };

    // Los que se han desconectado.
    apple.pads.retain(|pad| {
        let still = controllers.iter().any(|c| same(&c, &pad.controller));
        if !still {
            info!("Mando desconectado: {}", pad.name);
            messages.connection(GamepadConnectionEvent::new(
                pad.entity,
                GamepadConnection::Disconnected,
            ));
            commands.entity(pad.entity).despawn();
        }
        still
    });

    // Los nuevos, si son mandos completos (con dos sticks y gatillos).
    for controller in controllers.iter() {
        if apple
            .pads
            .iter()
            .any(|pad| same(&pad.controller, &controller))
        {
            continue;
        }
        // SAFETY: como arriba.
        if unsafe { controller.extendedGamepad() }.is_none() {
            continue;
        }
        let name = unsafe { controller.vendorName() }
            .map_or_else(|| "Mando".to_string(), |name| name.to_string());
        info!("Mando: {name} (leído con GameController de macOS)");
        let entity = commands.spawn_empty().id();
        messages.connection(GamepadConnectionEvent::new(
            entity,
            GamepadConnection::Connected {
                name: name.clone(),
                vendor_id: None,
                product_id: None,
            },
        ));
        apple.pads.push(Pad {
            controller,
            entity,
            name,
            buttons: [0.0; BUTTONS.len()],
            axes: [0.0; AXES.len()],
        });
    }

    // Lo que ha cambiado en cada uno.
    for pad in &mut apple.pads {
        // SAFETY: como arriba.
        let Some(gamepad) = (unsafe { pad.controller.extendedGamepad() }) else {
            continue;
        };
        for ((&button, last), now) in BUTTONS.iter().zip(&mut pad.buttons).zip(buttons(&gamepad)) {
            if (now - *last).abs() > CHANGE {
                *last = now;
                let event = RawGamepadButtonChangedEvent::new(pad.entity, button, now);
                messages.events.write(event.into());
                messages.buttons.write(event);
            }
        }
        for ((&axis, last), now) in AXES.iter().zip(&mut pad.axes).zip(axes(&gamepad)) {
            if (now - *last).abs() > CHANGE {
                *last = now;
                let event = RawGamepadAxisChangedEvent::new(pad.entity, axis, now);
                messages.events.write(event.into());
                messages.axes.write(event);
            }
        }
    }

    let current: Vec<String> = apple.pads.iter().map(|pad| pad.name.clone()).collect();
    if names.0 != current {
        names.0 = current;
    }
}

/// Cómo están los botones, en el orden de [`BUTTONS`] (de 0 a 1; los
/// gatillos, según lo que se aprieten).
fn buttons(gamepad: &GCExtendedGamepad) -> [f32; BUTTONS.len()] {
    let value = |button: Retained<GCControllerButtonInput>| unsafe { button.value() };
    let optional = |button: Option<Retained<GCControllerButtonInput>>| button.map_or(0.0, value);
    // SAFETY: GameController desde el hilo principal.
    unsafe {
        let dpad: Retained<GCControllerDirectionPad> = gamepad.dpad();
        [
            value(gamepad.buttonA()),
            value(gamepad.buttonB()),
            value(gamepad.buttonX()),
            value(gamepad.buttonY()),
            value(gamepad.leftShoulder()),
            value(gamepad.rightShoulder()),
            value(gamepad.leftTrigger()),
            value(gamepad.rightTrigger()),
            value(gamepad.buttonMenu()),
            optional(gamepad.buttonOptions()),
            optional(gamepad.buttonHome()),
            optional(gamepad.leftThumbstickButton()),
            optional(gamepad.rightThumbstickButton()),
            value(dpad.up()),
            value(dpad.down()),
            value(dpad.left()),
            value(dpad.right()),
        ]
    }
}

/// Cómo están los sticks, en el orden de [`AXES`] (de -1 a 1, arriba
/// positivo, como en Bevy).
fn axes(gamepad: &GCExtendedGamepad) -> [f32; AXES.len()] {
    // SAFETY: GameController desde el hilo principal.
    unsafe {
        let (left, right) = (gamepad.leftThumbstick(), gamepad.rightThumbstick());
        [
            left.xAxis().value(),
            left.yAxis().value(),
            right.xAxis().value(),
            right.yAxis().value(),
        ]
    }
}
