use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::gamepads::{ActiveController, ClaimedPads, ControllerModel, Gamepads};
use crate::hud::UiFont;
use crate::legend::{GLYPHS, Glyph, draw_glyph};
use crate::settings::fullscreen_keys;

/// El naranja del óxido, para «RUST» en «MOTORUST».
pub const RUST_ORANGE: Color = Color::srgb(0.9, 0.4, 0.12);

/// Cuánto hay que inclinar el stick para moverse por un menú.
const STICK_THRESHOLD: f32 = 0.5;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MenuSound>()
            .init_resource::<InputDevice>()
            .init_resource::<UiIcons>()
            .add_systems(PreUpdate, update_input_device);
    }
}

/// Si el jugador está usando el teclado o un mando.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputDevice {
    #[default]
    Keyboard,
    Gamepad,
}

/// Dispositivo activo detallado según el tipo de mando o teclado.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ActiveDevice {
    #[default]
    Keyboard,
    Xbox,
    PlayStation,
    Switch,
}

impl ActiveDevice {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            ActiveDevice::Keyboard => "Teclado",
            ActiveDevice::Xbox => "Xbox",
            ActiveDevice::PlayStation => "PlayStation",
            ActiveDevice::Switch => "Switch Pro",
        }
    }
}

pub fn current_active_device(
    device: InputDevice,
    active_controller: Option<&ActiveController>,
) -> ActiveDevice {
    match device {
        InputDevice::Keyboard => ActiveDevice::Keyboard,
        InputDevice::Gamepad => match active_controller.map(|ac| ac.model).unwrap_or_default() {
            ControllerModel::PlayStation => ActiveDevice::PlayStation,
            ControllerModel::Switch => ActiveDevice::Switch,
            ControllerModel::Xbox => ActiveDevice::Xbox,
        },
    }
}

fn update_input_device(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    gamepads_res: Option<Res<Gamepads>>,
    claimed: Option<Res<ClaimedPads>>,
    mut device: ResMut<InputDevice>,
    mut active_controller: Option<ResMut<ActiveController>>,
) {
    // Only written when something changes: the screens that show the
    // controls are built again when these change.
    if keyboard.get_just_pressed().next().is_some() {
        device.set_if_neq(InputDevice::Keyboard);
        if let Some(active) = active_controller.as_mut() {
            let keyboard = ActiveController {
                gamepad: None,
                ..**active
            };
            active.set_if_neq(keyboard);
        }
        return;
    }
    for (entity, gamepad) in &gamepads {
        let has_button = gamepad.get_just_pressed().next().is_some();
        let stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0).abs();
        let stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0).abs();
        if !(has_button || stick_x > 0.4 || stick_y > 0.4) {
            continue;
        }
        device.set_if_neq(InputDevice::Gamepad);
        if let Some(active) = active_controller.as_mut() {
            let name = claimed
                .as_ref()
                .and_then(|c| {
                    c.0.iter()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, n)| n.as_str())
                })
                .or_else(|| {
                    gamepads_res
                        .as_ref()
                        .and_then(|pads| pads.pad_names.get(&entity).map(|s| s.as_str()))
                });
            let wanted = ActiveController {
                gamepad: Some(entity),
                model: name.map_or(active.model, crate::gamepads::detect_controller_model),
            };
            active.set_if_neq(wanted);
        }
        break;
    }
}

/// Lo que suena al moverse por un menú.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuSound {
    /// Cambiar de opción, cambiar un valor o volver atrás.
    Move,
    /// Aceptar.
    Confirm,
}

/// Lo que se pide en este fotograma con el teclado o el mando.
#[derive(Default, Clone, Copy, Debug)]
pub struct Request {
    /// Arriba (-1) o abajo (1).
    pub step: i32,
    /// Izquierda (-1) o derecha (1).
    pub side: i32,
    /// Enter, Espacio, A o Start.
    pub confirm: bool,
    /// Start.
    pub start: bool,
    /// Esc o B.
    pub back: bool,
    /// Change of section: Tab (Shift + Tab goes back), Q and E, or the
    /// shoulder buttons and triggers (left -1, right 1).
    pub tab: i32,
}

/// El teclado y los mandos, para moverse por los menús: flechas, WASD,
/// cruceta o stick izquierdo.
#[derive(SystemParam)]
pub struct MenuInput<'w, 's> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub gamepads: Query<'w, 's, &'static Gamepad>,
    /// Si el stick estaba inclinado en vertical y en horizontal.
    held: Local<'s, (bool, bool)>,
}

impl MenuInput<'_, '_> {
    pub fn read(&mut self) -> Request {
        let keyboard = &self.keyboard;
        let pressed = |keys: [KeyCode; 2]| i32::from(keyboard.any_just_pressed(keys));
        let mut request = Request {
            step: pressed([KeyCode::ArrowDown, KeyCode::KeyS])
                - pressed([KeyCode::ArrowUp, KeyCode::KeyW]),
            side: pressed([KeyCode::ArrowRight, KeyCode::KeyD])
                - pressed([KeyCode::ArrowLeft, KeyCode::KeyA]),
            confirm: keyboard.any_just_pressed([KeyCode::Enter, KeyCode::Space])
                && !fullscreen_keys(keyboard),
            back: keyboard.just_pressed(KeyCode::Escape),
            start: false,
            tab: i32::from(keyboard.just_pressed(KeyCode::KeyE))
                - i32::from(keyboard.just_pressed(KeyCode::KeyQ)),
        };
        if keyboard.just_pressed(KeyCode::Tab) {
            let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            request.tab += if shift { -1 } else { 1 };
        }
        let mut dpad_down = false;
        let mut dpad_up = false;
        let mut dpad_right = false;
        let mut dpad_left = false;
        let mut stick = Vec2::ZERO;
        for gamepad in &self.gamepads {
            dpad_down |= gamepad.just_pressed(GamepadButton::DPadDown);
            dpad_up |= gamepad.just_pressed(GamepadButton::DPadUp);
            dpad_right |= gamepad.just_pressed(GamepadButton::DPadRight);
            dpad_left |= gamepad.just_pressed(GamepadButton::DPadLeft);
            request.confirm |= gamepad.just_pressed(GamepadButton::South)
                || gamepad.just_pressed(GamepadButton::Start);
            request.back |= gamepad.just_pressed(GamepadButton::East);
            request.start |= gamepad.just_pressed(GamepadButton::Start);
            request.tab += i32::from(
                gamepad
                    .any_just_pressed([GamepadButton::RightTrigger, GamepadButton::RightTrigger2]),
            ) - i32::from(
                gamepad.any_just_pressed([GamepadButton::LeftTrigger, GamepadButton::LeftTrigger2]),
            );
            let axis = |axis| gamepad.get(axis).unwrap_or(0.0);
            let (x, y) = (axis(GamepadAxis::LeftStickX), axis(GamepadAxis::LeftStickY));
            if x.abs() > stick.x.abs() {
                stick.x = x;
            }
            if y.abs() > stick.y.abs() {
                stick.y = y;
            }
        }
        let pad_step = i32::from(dpad_down) - i32::from(dpad_up);
        let pad_side = i32::from(dpad_right) - i32::from(dpad_left);
        request.step += pad_step;
        request.side += pad_side;

        // El stick mueve una vez cada vez que se inclina (arriba es positivo).
        let (vertical, horizontal) = &mut *self.held;
        request.step -= tilt(stick.y, vertical);
        request.side += tilt(stick.x, horizontal);
        request.step = request.step.clamp(-1, 1);
        request.side = request.side.clamp(-1, 1);
        request.tab = request.tab.clamp(-1, 1);
        request
    }
}

/// -1 o 1 si el stick se acaba de inclinar hacia ese lado, y si no, 0.
fn tilt(value: f32, held: &mut bool) -> i32 {
    let tilted = value.abs() > STICK_THRESHOLD;
    let step = if tilted && !*held {
        value.signum() as i32
    } else {
        0
    };
    *held = tilted;
    step
}

/// The red of "Exit game": its button in the start menu, and the option
/// that confirms it.
pub const EXIT_RED: Color = Color::srgb(1.0, 32.0 / 255.0, 32.0 / 255.0);
/// The main action of a menu, like resuming the race: the blue of the
/// start menu's 1P card.
pub const PRIMARY_BLUE: Color = Color::srgb(0.04, 0.4, 1.0);

/// Una opción de un menú: su texto y, si tiene, su valor, a la derecha.
pub struct Row {
    pub label: String,
    pub value: Option<String>,
    pub disabled: bool,
    /// Whether it does something that cannot be undone, like leaving the
    /// game: chosen, it shows in red.
    pub danger: bool,
    /// Whether it is the main action of its menu, like resuming the race:
    /// it shows in blue, faintly when it is not chosen.
    pub primary: bool,
    /// Whether it is something on or off, a box ticked or not after its
    /// value (if any). Drawn by the options menu.
    pub check: Option<bool>,
}

impl Row {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
            disabled: false,
            danger: false,
            primary: false,
            check: None,
        }
    }

    /// Something on or off, with a box ticked or not.
    pub fn check(label: impl Into<String>, on: bool) -> Self {
        Self {
            check: Some(on),
            ..Self::new(label)
        }
    }

    /// The main action of its menu, like resuming the race.
    pub fn primary(label: impl Into<String>) -> Self {
        Self {
            primary: true,
            ..Self::new(label)
        }
    }

    /// An option that cannot be undone, like leaving the game.
    pub fn danger(label: impl Into<String>) -> Self {
        Self {
            danger: true,
            ..Self::new(label)
        }
    }

    pub fn with_value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            value: Some(value.into()),
            ..Self::new(label)
        }
    }

    pub fn disabled(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            disabled: true,
            ..Self::with_value(label, value)
        }
    }

    /// Background and text colours of the option when it is chosen.
    pub fn chosen_colors(&self) -> (Color, Color) {
        if self.disabled {
            (
                Color::srgba(0.5, 0.5, 0.5, 0.25),
                Color::srgba(0.85, 0.85, 0.85, 0.7),
            )
        } else if self.danger {
            (EXIT_RED, Color::WHITE)
        } else if self.primary {
            (PRIMARY_BLUE, Color::WHITE)
        } else {
            (
                Color::srgba(1.0, 0.8, 0.2, 0.95),
                Color::srgb(0.08, 0.06, 0.02),
            )
        }
    }

    /// Background of the option when it is not chosen: none, but the main
    /// action stands out in a faint blue.
    pub fn idle_background(&self) -> Color {
        if self.primary {
            PRIMARY_BLUE.with_alpha(0.3)
        } else {
            Color::NONE
        }
    }
}

pub fn text_font(font: &UiFont, size: f32) -> TextFont {
    TextFont {
        font: font.0.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// Las opciones, juntas en un mismo panel; la elegida, resaltada. Con
/// `compact`, más pequeñas, para que quepan muchas.
pub fn spawn_panel(
    parent: &mut ChildSpawnerCommands,
    font: &UiFont,
    rows: &[Row],
    selected: usize,
    compact: bool,
) {
    let (size, padding, width) = if compact {
        (21.0, 7, 640)
    } else {
        (26.0, 10, 440)
    };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(6),
                min_width: px(width),
                padding: UiRect::all(px(14)),
                border_radius: BorderRadius::all(px(16)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.1, 0.8)),
        ))
        .with_children(|panel| {
            for (index, row) in rows.iter().enumerate() {
                let chosen = index == selected;
                let (chosen_background, chosen_text) = row.chosen_colors();
                let color = TextColor(if chosen {
                    chosen_text
                } else if row.disabled {
                    Color::srgba(1.0, 1.0, 1.0, 0.4)
                } else {
                    Color::WHITE
                });
                panel
                    .spawn((
                        Node {
                            justify_content: if row.value.is_some() {
                                JustifyContent::SpaceBetween
                            } else {
                                JustifyContent::Center
                            },
                            column_gap: px(40),
                            padding: UiRect::axes(px(28), px(padding)),
                            border_radius: BorderRadius::all(px(10)),
                            ..default()
                        },
                        BackgroundColor(if chosen {
                            chosen_background
                        } else {
                            row.idle_background()
                        }),
                    ))
                    .with_children(|line| {
                        line.spawn((Text::new(&row.label), text_font(font, size), color));
                        if let Some(value) = &row.value {
                            line.spawn((Text::new(value), text_font(font, size), color));
                        }
                    });
            }
        });
}

/// El título grande de una pantalla.
pub fn spawn_title(parent: &mut ChildSpawnerCommands, font: &UiFont, title: &str, size: f32) {
    parent.spawn((
        Text::new(title),
        text_font(font, size),
        TextShadow {
            offset: Vec2::splat(3.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
        },
        Node {
            margin: UiRect::bottom(px(12)),
            ..default()
        },
    ));
}

/// El título del juego: «MOTO» en blanco y «RUST» en naranja óxido, con
/// una sombra más marcada.
pub fn spawn_logo(parent: &mut ChildSpawnerCommands, font: &UiFont, size: f32) {
    parent
        .spawn((
            Text::new("MOTO"),
            text_font(font, size),
            TextShadow {
                offset: Vec2::splat(5.0),
                color: Color::srgba(0.05, 0.02, 0.0, 0.75),
            },
            Node {
                margin: UiRect::bottom(px(12)),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("RUST"),
            text_font(font, size),
            TextColor(RUST_ORANGE),
        ));
}

/// La línea de ayuda de debajo de un menú.
#[allow(dead_code)]
pub fn spawn_hint(parent: &mut ChildSpawnerCommands, font: &UiFont, hint: &str) {
    parent.spawn((
        Text::new(hint),
        text_font(font, 17.0),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
        TextShadow::default(),
        Node {
            margin: UiRect::top(px(14)),
            ..default()
        },
    ));
}

/// Texto, abajo a la izquierda, con la versión del juego.
#[derive(Component)]
pub struct VersionLabel;

pub fn version_label(font: &UiFont) -> impl Bundle {
    (
        VersionLabel,
        Text::new(concat!("v", env!("CARGO_PKG_VERSION"))),
        text_font(font, 15.0),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(16),
            ..default()
        },
    )
}

/// The symbols of the controls legend ([`crate::legend`]), which the UI
/// font cannot draw.
#[derive(Resource, Clone)]
pub struct UiIcons {
    glyphs: Vec<Handle<Image>>,
}

impl UiIcons {
    pub fn glyph(&self, glyph: Glyph) -> Handle<Image> {
        let index = GLYPHS
            .iter()
            .position(|&g| g == glyph)
            .expect("every glyph is drawn");
        self.glyphs[index].clone()
    }
}

impl FromWorld for UiIcons {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let glyphs = GLYPHS
            .iter()
            .map(|&glyph| images.add(draw_glyph(glyph)))
            .collect();
        Self { glyphs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stick_moves_once_each_time_it_is_tilted() {
        let mut held = false;
        let steps: Vec<i32> = [0.0, 0.8, 0.9, 0.2, -0.7, -0.9, 0.0]
            .into_iter()
            .map(|value| tilt(value, &mut held))
            .collect();
        assert_eq!(steps, [0, 1, 0, 0, -1, 0, 0]);
    }

    #[test]
    fn the_main_action_is_blue_and_leaving_red() {
        let resume = Row::primary("Resume");
        assert_eq!(resume.chosen_colors(), (PRIMARY_BLUE, Color::WHITE));
        assert_ne!(resume.idle_background(), Color::NONE);
        assert_eq!(Row::danger("Exit").chosen_colors().0, EXIT_RED);
        assert_eq!(Row::new("Back").idle_background(), Color::NONE);
    }
}
