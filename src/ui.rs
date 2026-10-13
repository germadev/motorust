use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::gamepads::{ActiveController, ClaimedPads, ControllerModel, Gamepads};
use crate::hud::UiFont;
use crate::locale::Language;
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
    if keyboard.get_just_pressed().next().is_some() {
        if *device != InputDevice::Keyboard {
            *device = InputDevice::Keyboard;
        }
        if let Some(ref mut ac) = active_controller {
            ac.gamepad = None;
        }
    } else {
        for (entity, gamepad) in &gamepads {
            let has_button = gamepad.get_just_pressed().next().is_some();
            let stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0).abs();
            let stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0).abs();
            if has_button || stick_x > 0.4 || stick_y > 0.4 {
                if *device != InputDevice::Gamepad {
                    *device = InputDevice::Gamepad;
                }
                if let Some(ref mut ac) = active_controller {
                    ac.gamepad = Some(entity);
                    let mut name = claimed.as_ref().and_then(|c| {
                        c.0.iter()
                            .find(|(e, _)| *e == entity)
                            .map(|(_, n)| n.as_str())
                    });
                    if name.is_none()
                        && let Some(ref pads) = gamepads_res
                    {
                        name = pads.pad_names.get(&entity).map(|s| s.as_str());
                    }
                    if let Some(n) = name {
                        ac.model = crate::gamepads::detect_controller_model(n);
                    }
                }
                break;
            }
        }
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
    /// Tab o Select (opciones).
    pub options: bool,
    /// Gatillos superiores / bumpers (Q/LB es -1, E/RB es 1).
    pub shoulder: i32,
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
            options: keyboard.just_pressed(KeyCode::Tab),
            shoulder: i32::from(keyboard.just_pressed(KeyCode::KeyE))
                - i32::from(keyboard.just_pressed(KeyCode::KeyQ)),
        };
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
            request.options |= gamepad.just_pressed(GamepadButton::Select);
            request.shoulder += i32::from(gamepad.just_pressed(GamepadButton::RightTrigger))
                - i32::from(gamepad.just_pressed(GamepadButton::LeftTrigger));
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

/// Iconos gráficos del juego generados por procedimiento.
#[derive(Resource, Clone)]
pub struct UiIcons {
    pub helmet: Handle<Image>,
    pub card_blue: Handle<Image>,
    pub card_dark: Handle<Image>,
    pub banner_yellow: Handle<Image>,
    pub banner_red: Handle<Image>,
}

impl FromWorld for UiIcons {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let helmet_img = load_png_bytes(include_bytes!("../assets/ui/helmet.png"));
        let helmet = images.add(helmet_img);
        let card_blue = images.add(draw_wavy_checker_card(
            512,
            256,
            [0, 107, 251, 255],
            [0, 85, 215, 255],
        ));
        let card_dark = images.add(draw_wavy_checker_card(
            512,
            256,
            [28, 30, 36, 255],
            [18, 19, 23, 255],
        ));
        let banner_yellow = images.add(draw_wavy_checker_card(
            512,
            128,
            [252, 182, 0, 255],
            [232, 158, 0, 255],
        ));
        let banner_red = images.add(draw_wavy_checker_card(
            512,
            128,
            [232, 28, 36, 255],
            [205, 18, 24, 255],
        ));
        Self {
            helmet,
            card_blue,
            card_dark,
            banner_yellow,
            banner_red,
        }
    }
}

fn load_png_bytes(bytes: &[u8]) -> Image {
    let dyn_img = image::load_from_memory(bytes).expect("valid embedded png");
    let rgba = dyn_img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn draw_wavy_checker_card(width: u32, height: u32, color_a: [u8; 4], color_b: [u8; 4]) -> Image {
    paint_rgba_image(width, height, |u, v| {
        let wave_x = (v * 10.0).sin() * 0.04;
        let wave_y = (u * 8.0).sin() * 0.03;
        let su = u + wave_x;
        let sv = v + wave_y;

        let check_x = ((su * 10.0).floor() as i32).rem_euclid(2);
        let check_y = ((sv * 5.0).floor() as i32).rem_euclid(2);
        let is_alt = (check_x ^ check_y) != 0;

        if is_alt { color_b } else { color_a }
    })
}

fn paint_rgba_image(width: u32, height: u32, f: impl Fn(f32, f32) -> [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;
            data.extend_from_slice(&f(u, v));
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Elemento de leyenda con tecla/botón adaptado al mando y su acción.
#[derive(Clone, Debug)]
pub struct KeyPrompt {
    pub badge: &'static str,
    pub badge_bg: Color,
    pub badge_text: Color,
    pub badge_border: Color,
    pub is_round: bool,
    pub action: String,
}

pub fn race_prompts(device: ActiveDevice, lang: Language) -> Vec<KeyPrompt> {
    match device {
        ActiveDevice::PlayStation => vec![
            KeyPrompt {
                badge: "R2",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: lang.action_throttle().into(),
            },
            KeyPrompt {
                badge: "L2",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Frenar".into(),
                    Language::English => "Brake".into(),
                },
            },
            KeyPrompt {
                badge: "Stick L",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Girar".into(),
                    Language::English => "Steer".into(),
                },
            },
            KeyPrompt {
                badge: "○",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.92, 0.18, 0.22),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: true,
                action: lang.action_wheelie().into(),
            },
            KeyPrompt {
                badge: "Share",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: lang.restart().into(),
            },
            KeyPrompt {
                badge: "Options",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: lang.action_pause().into(),
            },
        ],
        ActiveDevice::Xbox => vec![
            KeyPrompt {
                badge: "RT",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.action_throttle().into(),
            },
            KeyPrompt {
                badge: "LT",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Frenar".into(),
                    Language::English => "Brake".into(),
                },
            },
            KeyPrompt {
                badge: "LS",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Girar".into(),
                    Language::English => "Steer".into(),
                },
            },
            KeyPrompt {
                badge: "B",
                badge_bg: Color::srgb(0.32, 0.08, 0.1),
                badge_text: Color::srgb(0.95, 0.35, 0.35),
                badge_border: Color::srgb(0.7, 0.15, 0.2),
                is_round: true,
                action: lang.action_wheelie().into(),
            },
            KeyPrompt {
                badge: "View",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.restart().into(),
            },
            KeyPrompt {
                badge: "Menu",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.action_pause().into(),
            },
        ],
        ActiveDevice::Switch => vec![
            KeyPrompt {
                badge: "ZR",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.action_throttle().into(),
            },
            KeyPrompt {
                badge: "ZL",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Frenar".into(),
                    Language::English => "Brake".into(),
                },
            },
            KeyPrompt {
                badge: "Stick L",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Girar".into(),
                    Language::English => "Steer".into(),
                },
            },
            KeyPrompt {
                badge: "A",
                badge_bg: Color::srgb(0.28, 0.12, 0.15),
                badge_text: Color::srgb(0.95, 0.35, 0.35),
                badge_border: Color::srgb(0.65, 0.18, 0.22),
                is_round: true,
                action: lang.action_wheelie().into(),
            },
            KeyPrompt {
                badge: "-",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.restart().into(),
            },
            KeyPrompt {
                badge: "+",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: lang.action_pause().into(),
            },
        ],
        ActiveDevice::Keyboard => vec![
            KeyPrompt {
                badge: "W / ↑",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: lang.action_throttle().into(),
            },
            KeyPrompt {
                badge: "S / ↓",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Frenar".into(),
                    Language::English => "Brake".into(),
                },
            },
            KeyPrompt {
                badge: "A / D",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Girar".into(),
                    Language::English => "Steer".into(),
                },
            },
            KeyPrompt {
                badge: match lang {
                    Language::Spanish => "Espacio",
                    Language::English => "Space",
                },
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: lang.action_wheelie().into(),
            },
            KeyPrompt {
                badge: "R",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: lang.restart().into(),
            },
            KeyPrompt {
                badge: "Esc",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: lang.action_pause().into(),
            },
            KeyPrompt {
                badge: "M",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: match lang {
                    Language::Spanish => "Sonido".into(),
                    Language::English => "Sound".into(),
                },
            },
        ],
    }
}

pub fn menu_prompts(
    device: ActiveDevice,
    lang: Language,
    has_side: bool,
    confirm_action: &str,
    back_action: Option<&str>,
) -> Vec<KeyPrompt> {
    let choose_text = match lang {
        Language::Spanish => "Elegir",
        Language::English => "Choose",
    };
    let change_text = match lang {
        Language::Spanish => "Cambiar",
        Language::English => "Change",
    };

    match device {
        ActiveDevice::PlayStation => {
            let mut list = vec![KeyPrompt {
                badge: "Cruz / Stick",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.12, 0.14, 0.2),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: false,
                action: choose_text.into(),
            }];
            if has_side {
                list.push(KeyPrompt {
                    badge: "← / →",
                    badge_bg: Color::WHITE,
                    badge_text: Color::srgb(0.12, 0.14, 0.2),
                    badge_border: Color::srgb(0.82, 0.84, 0.88),
                    is_round: false,
                    action: change_text.into(),
                });
            }
            list.push(KeyPrompt {
                badge: "✕",
                badge_bg: Color::WHITE,
                badge_text: Color::srgb(0.2, 0.42, 0.95),
                badge_border: Color::srgb(0.82, 0.84, 0.88),
                is_round: true,
                action: confirm_action.into(),
            });
            if let Some(back) = back_action {
                list.push(KeyPrompt {
                    badge: "○",
                    badge_bg: Color::WHITE,
                    badge_text: Color::srgb(0.92, 0.18, 0.22),
                    badge_border: Color::srgb(0.82, 0.84, 0.88),
                    is_round: true,
                    action: back.into(),
                });
            }
            list
        }
        ActiveDevice::Xbox => {
            let mut list = vec![KeyPrompt {
                badge: "D-Pad / Stick",
                badge_bg: Color::srgb(0.16, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: choose_text.into(),
            }];
            if has_side {
                list.push(KeyPrompt {
                    badge: "← / →",
                    badge_bg: Color::srgb(0.16, 0.18, 0.22),
                    badge_text: Color::WHITE,
                    badge_border: Color::srgb(0.35, 0.4, 0.48),
                    is_round: false,
                    action: change_text.into(),
                });
            }
            list.push(KeyPrompt {
                badge: "A",
                badge_bg: Color::srgb(0.08, 0.28, 0.12),
                badge_text: Color::srgb(0.35, 0.95, 0.45),
                badge_border: Color::srgb(0.18, 0.65, 0.25),
                is_round: true,
                action: confirm_action.into(),
            });
            if let Some(back) = back_action {
                list.push(KeyPrompt {
                    badge: "B",
                    badge_bg: Color::srgb(0.32, 0.08, 0.1),
                    badge_text: Color::srgb(0.95, 0.35, 0.35),
                    badge_border: Color::srgb(0.7, 0.15, 0.2),
                    is_round: true,
                    action: back.into(),
                });
            }
            list
        }
        ActiveDevice::Switch => {
            let mut list = vec![KeyPrompt {
                badge: "Cruz / Stick",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: false,
                action: choose_text.into(),
            }];
            if has_side {
                list.push(KeyPrompt {
                    badge: "← / →",
                    badge_bg: Color::srgb(0.15, 0.18, 0.22),
                    badge_text: Color::WHITE,
                    badge_border: Color::srgb(0.35, 0.4, 0.48),
                    is_round: false,
                    action: change_text.into(),
                });
            }
            list.push(KeyPrompt {
                badge: "B",
                badge_bg: Color::srgb(0.15, 0.18, 0.22),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.48),
                is_round: true,
                action: confirm_action.into(),
            });
            if let Some(back) = back_action {
                list.push(KeyPrompt {
                    badge: "A",
                    badge_bg: Color::srgb(0.28, 0.12, 0.15),
                    badge_text: Color::srgb(0.95, 0.35, 0.35),
                    badge_border: Color::srgb(0.65, 0.18, 0.22),
                    is_round: true,
                    action: back.into(),
                });
            }
            list
        }
        ActiveDevice::Keyboard => {
            let mut list = vec![KeyPrompt {
                badge: "↑ / ↓",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: choose_text.into(),
            }];
            if has_side {
                list.push(KeyPrompt {
                    badge: "← / →",
                    badge_bg: Color::srgb(0.18, 0.2, 0.26),
                    badge_text: Color::WHITE,
                    badge_border: Color::srgb(0.35, 0.4, 0.52),
                    is_round: false,
                    action: change_text.into(),
                });
            }
            list.push(KeyPrompt {
                badge: "ENTER",
                badge_bg: Color::srgb(0.18, 0.2, 0.26),
                badge_text: Color::WHITE,
                badge_border: Color::srgb(0.35, 0.4, 0.52),
                is_round: false,
                action: confirm_action.into(),
            });
            if let Some(back) = back_action {
                list.push(KeyPrompt {
                    badge: "ESC",
                    badge_bg: Color::srgb(0.18, 0.2, 0.26),
                    badge_text: Color::WHITE,
                    badge_border: Color::srgb(0.35, 0.4, 0.52),
                    is_round: false,
                    action: back.into(),
                });
            }
            list
        }
    }
}

/// Spawns the bottom legend bar with key/button badges and action labels.
pub fn spawn_legend_bar(parent: &mut ChildSpawnerCommands, font: &UiFont, prompts: &[KeyPrompt]) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(14),
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(18),
                padding: UiRect::axes(px(22), px(8)),
                border_radius: BorderRadius::all(px(22)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.09, 0.88)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.14)),
            ZIndex(150),
        ))
        .with_children(|bar| {
            for prompt in prompts {
                bar.spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(7),
                    ..default()
                },))
                    .with_children(|item| {
                        item.spawn((
                            Node {
                                padding: UiRect::axes(px(8), px(3)),
                                border_radius: BorderRadius::all(if prompt.is_round {
                                    px(12)
                                } else {
                                    px(6)
                                }),
                                border: UiRect::all(px(1)),
                                min_width: if prompt.is_round { px(24) } else { px(0) },
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(prompt.badge_bg),
                            BorderColor::all(prompt.badge_border),
                        ))
                        .with_child((
                            Text::new(prompt.badge),
                            text_font(font, 14.0),
                            TextColor(prompt.badge_text),
                        ));

                        item.spawn((
                            Text::new(&prompt.action),
                            text_font(font, 15.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                            TextShadow::default(),
                        ));
                    });
            }
        });
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
