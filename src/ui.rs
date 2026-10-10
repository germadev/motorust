//! Lo común a los menús: moverse por ellos con el teclado o el mando, el
//! panel con las opciones y el aviso de qué mandos ve el juego.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::gamepads::Gamepads;
use crate::hud::UiFont;
use crate::locale::Language;
use crate::settings::{Settings, fullscreen_keys};

/// El naranja del óxido, para «RUST» en «MOTORUST».
pub const RUST_ORANGE: Color = Color::srgb(0.9, 0.4, 0.12);

/// Cuánto hay que inclinar el stick para moverse por un menú.
const STICK_THRESHOLD: f32 = 0.5;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MenuSound>()
            .add_systems(Update, show_gamepads);
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
    /// Enter, Espacio o A.
    pub confirm: bool,
    /// Start.
    pub start: bool,
    /// Esc o B.
    pub back: bool,
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
        };
        let mut stick = Vec2::ZERO;
        for gamepad in &self.gamepads {
            let pressed = |button| i32::from(gamepad.just_pressed(button));
            request.step += pressed(GamepadButton::DPadDown) - pressed(GamepadButton::DPadUp);
            request.side += pressed(GamepadButton::DPadRight) - pressed(GamepadButton::DPadLeft);
            request.confirm |= gamepad.just_pressed(GamepadButton::South);
            request.back |= gamepad.just_pressed(GamepadButton::East);
            request.start |= gamepad.just_pressed(GamepadButton::Start);
            let axis = |axis| gamepad.get(axis).unwrap_or(0.0);
            let (x, y) = (axis(GamepadAxis::LeftStickX), axis(GamepadAxis::LeftStickY));
            if x.abs() > stick.x.abs() {
                stick.x = x;
            }
            if y.abs() > stick.y.abs() {
                stick.y = y;
            }
        }
        // El stick mueve una vez cada vez que se inclina (arriba es positivo).
        let (vertical, horizontal) = &mut *self.held;
        request.step -= tilt(stick.y, vertical);
        request.side += tilt(stick.x, horizontal);
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

/// Una opción de un menú: su texto y, si tiene, su valor, a la derecha.
pub struct Row {
    pub label: String,
    pub value: Option<String>,
    pub disabled: bool,
}

impl Row {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
            disabled: false,
        }
    }

    pub fn with_value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            disabled: false,
        }
    }

    pub fn disabled(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            disabled: true,
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
                let color = TextColor(if chosen {
                    if row.disabled {
                        Color::srgba(0.85, 0.85, 0.85, 0.7)
                    } else {
                        Color::srgb(0.08, 0.06, 0.02)
                    }
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
                            if row.disabled {
                                Color::srgba(0.5, 0.5, 0.5, 0.25)
                            } else {
                                Color::srgba(1.0, 0.8, 0.2, 0.95)
                            }
                        } else {
                            Color::NONE
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

/// Texto, abajo a la derecha, con los mandos que ve el juego.
#[derive(Component)]
pub struct GamepadStatus;

pub fn gamepad_status(font: &UiFont) -> impl Bundle {
    (
        GamepadStatus,
        Text::new(""),
        text_font(font, 15.0),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            right: px(16),
            ..default()
        },
    )
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

/// El mando que ve el juego, o un aviso si no ve ninguno.
fn show_gamepads(
    gamepads: Query<&Name, With<Gamepad>>,
    search: Res<Gamepads>,
    settings: Res<Settings>,
    mut status: Query<&mut Text, With<GamepadStatus>>,
) {
    if status.is_empty() {
        return;
    }
    let lang = settings.language;
    let names: Vec<&str> = gamepads.iter().map(Name::as_str).collect();
    let label = if !names.is_empty() {
        match lang {
            Language::Spanish => format!("Mando: {}", names.join(", ")),
            Language::English => format!("Gamepad: {}", names.join(", ")),
        }
    } else if search.searching() {
        match lang {
            Language::Spanish => "Buscando mando...".to_string(),
            Language::English => "Searching for gamepad...".to_string(),
        }
    } else {
        match lang {
            Language::Spanish => {
                "Sin mando: si tienes uno, reconéctalo o prueba otro modo del mando".to_string()
            }
            Language::English => {
                "No gamepad: if you have one, reconnect it or try another gamepad mode".to_string()
            }
        }
    };
    for mut text in &mut status {
        if text.0 != label {
            text.0.clone_from(&label);
        }
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
}
