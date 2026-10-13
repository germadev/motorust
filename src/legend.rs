//! The controls legend at the bottom of every screen: each key or button as
//! it looks on the device in use (keyboard, or an Xbox, PlayStation or
//! Switch controller) and what it does there. It looks the same everywhere:
//! no panel behind it, the badges, and the action in white with a shadow.
//!
//! The UI font has no arrows nor PlayStation symbols, so those are drawn as
//! icons ([`Glyph`]). The keys of the race come from the player's own
//! controls ([`Controls`]), so the legend follows any change made in Options.

use bevy::prelude::*;

use crate::hud::UiFont;
use crate::locale::Language;
use crate::settings::{Action, Controls, key_name};
use crate::ui::{ActiveDevice, UiIcons, text_font};
use crate::vector_art::{Coverage, white_image};

/// A symbol the UI font cannot draw.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Glyph {
    Up,
    Down,
    Left,
    Right,
    /// The PlayStation face buttons.
    Cross,
    Circle,
    Square,
    Triangle,
    /// A thumbstick.
    Stick,
}

pub const GLYPHS: [Glyph; 9] = [
    Glyph::Up,
    Glyph::Down,
    Glyph::Left,
    Glyph::Right,
    Glyph::Cross,
    Glyph::Circle,
    Glyph::Square,
    Glyph::Triangle,
    Glyph::Stick,
];

/// What a badge shows.
#[derive(Clone, PartialEq, Debug)]
pub enum Label {
    Text(String),
    Glyphs(Vec<Glyph>),
}

/// A key or a button.
#[derive(Clone, PartialEq, Debug)]
pub struct Badge {
    pub label: Label,
    /// Round like a face button, or a rounded rectangle like a key.
    pub round: bool,
    /// Colour of the letters or the symbol.
    pub ink: Color,
}

/// One entry of the legend: the keys or buttons, and what they do.
#[derive(Clone, PartialEq, Debug)]
pub struct KeyPrompt {
    pub badges: Vec<Badge>,
    pub action: String,
}

/// What a menu listens to (see [`crate::ui::MenuInput`]). Moving around a
/// menu (arrows, WASD, D-pad or stick) goes without saying, and is left out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Keys {
    /// Enter, Space, A or Start.
    Confirm,
    /// Esc or B.
    Back,
    /// Change of section: Tab, or the shoulder buttons.
    Tabs,
}

/// The face, edge and letters of the badges of each device.
struct Style {
    face: Color,
    edge: Color,
    ink: Color,
}

fn style(device: ActiveDevice) -> Style {
    match device {
        // Light keycaps, with a deeper bottom edge.
        ActiveDevice::Keyboard => Style {
            face: Color::srgb(0.93, 0.94, 0.96),
            edge: Color::srgb(0.55, 0.58, 0.64),
            ink: Color::srgb(0.1, 0.11, 0.14),
        },
        // White, like the PlayStation's own button prompts.
        ActiveDevice::PlayStation => Style {
            face: Color::WHITE,
            edge: Color::srgb(0.78, 0.8, 0.85),
            ink: Color::srgb(0.12, 0.14, 0.2),
        },
        ActiveDevice::Xbox => Style {
            face: Color::srgb(0.1, 0.11, 0.13),
            edge: Color::srgb(0.38, 0.4, 0.45),
            ink: Color::WHITE,
        },
        ActiveDevice::Switch => Style {
            face: Color::srgb(0.14, 0.15, 0.17),
            edge: Color::srgb(0.4, 0.42, 0.47),
            ink: Color::WHITE,
        },
    }
}

/// The classic colours of the PlayStation symbols.
const PS_BLUE: Color = Color::srgb(0.2, 0.42, 0.95);
const PS_RED: Color = Color::srgb(0.92, 0.18, 0.22);
const PS_PINK: Color = Color::srgb(0.9, 0.42, 0.7);
const PS_GREEN: Color = Color::srgb(0.1, 0.68, 0.5);
/// The colours of the Xbox letters.
const XBOX_GREEN: Color = Color::srgb(0.38, 0.85, 0.3);
const XBOX_RED: Color = Color::srgb(0.95, 0.3, 0.28);
const XBOX_BLUE: Color = Color::srgb(0.3, 0.6, 1.0);
const XBOX_YELLOW: Color = Color::srgb(1.0, 0.8, 0.2);

fn text(device: ActiveDevice, label: &str) -> Badge {
    Badge {
        label: Label::Text(label.into()),
        round: false,
        ink: style(device).ink,
    }
}

fn glyphs(device: ActiveDevice, glyphs: &[Glyph]) -> Badge {
    Badge {
        label: Label::Glyphs(glyphs.to_vec()),
        round: false,
        ink: style(device).ink,
    }
}

fn face(label: Label, ink: Color) -> Badge {
    Badge {
        label,
        round: true,
        ink,
    }
}

/// A key of the keyboard: the arrows as arrows, the rest by name.
pub fn key_badge(key: KeyCode, lang: Language) -> Badge {
    let device = ActiveDevice::Keyboard;
    match key {
        KeyCode::ArrowUp => glyphs(device, &[Glyph::Up]),
        KeyCode::ArrowDown => glyphs(device, &[Glyph::Down]),
        KeyCode::ArrowLeft => glyphs(device, &[Glyph::Left]),
        KeyCode::ArrowRight => glyphs(device, &[Glyph::Right]),
        KeyCode::Space if lang == Language::English => text(device, "Space"),
        KeyCode::ShiftLeft | KeyCode::ShiftRight if lang == Language::English => {
            text(device, "Shift")
        }
        KeyCode::ShiftLeft | KeyCode::ShiftRight => text(device, "Mayús"),
        other => text(device, &key_name(other, lang)),
    }
}

/// A button of a controller, as it is marked on that controller.
pub fn button_badge(device: ActiveDevice, button: GamepadButton) -> Badge {
    use GamepadButton as B;
    let label = |name: &str| text(device, name);
    match (device, button) {
        (_, B::DPadUp) => glyphs(device, &[Glyph::Up]),
        (_, B::DPadDown) => glyphs(device, &[Glyph::Down]),
        (_, B::DPadLeft) => glyphs(device, &[Glyph::Left]),
        (_, B::DPadRight) => glyphs(device, &[Glyph::Right]),
        (ActiveDevice::PlayStation, B::South) => face(Label::Glyphs(vec![Glyph::Cross]), PS_BLUE),
        (ActiveDevice::PlayStation, B::East) => face(Label::Glyphs(vec![Glyph::Circle]), PS_RED),
        (ActiveDevice::PlayStation, B::West) => face(Label::Glyphs(vec![Glyph::Square]), PS_PINK),
        (ActiveDevice::PlayStation, B::North) => {
            face(Label::Glyphs(vec![Glyph::Triangle]), PS_GREEN)
        }
        (ActiveDevice::PlayStation, B::LeftTrigger) => label("L1"),
        (ActiveDevice::PlayStation, B::RightTrigger) => label("R1"),
        (ActiveDevice::PlayStation, B::LeftTrigger2) => label("L2"),
        (ActiveDevice::PlayStation, B::RightTrigger2) => label("R2"),
        (ActiveDevice::PlayStation, B::Select) => label("Create"),
        (ActiveDevice::PlayStation, B::Start) => label("Options"),
        (ActiveDevice::PlayStation, B::LeftThumb) => label("L3"),
        (ActiveDevice::PlayStation, B::RightThumb) => label("R3"),
        (ActiveDevice::PlayStation, B::Mode) => label("PS"),
        // Nintendo puts A on the right and B at the bottom.
        (ActiveDevice::Switch, B::South) => face(Label::Text("B".into()), Color::WHITE),
        (ActiveDevice::Switch, B::East) => face(Label::Text("A".into()), Color::WHITE),
        (ActiveDevice::Switch, B::West) => face(Label::Text("Y".into()), Color::WHITE),
        (ActiveDevice::Switch, B::North) => face(Label::Text("X".into()), Color::WHITE),
        (ActiveDevice::Switch, B::LeftTrigger) => label("L"),
        (ActiveDevice::Switch, B::RightTrigger) => label("R"),
        (ActiveDevice::Switch, B::LeftTrigger2) => label("ZL"),
        (ActiveDevice::Switch, B::RightTrigger2) => label("ZR"),
        (ActiveDevice::Switch, B::Select) => label("-"),
        (ActiveDevice::Switch, B::Start) => label("+"),
        (ActiveDevice::Switch, B::Mode) => label("Home"),
        // Xbox, and the names any other controller gets.
        (_, B::South) => face(Label::Text("A".into()), XBOX_GREEN),
        (_, B::East) => face(Label::Text("B".into()), XBOX_RED),
        (_, B::West) => face(Label::Text("X".into()), XBOX_BLUE),
        (_, B::North) => face(Label::Text("Y".into()), XBOX_YELLOW),
        (_, B::LeftTrigger) => label("LB"),
        (_, B::RightTrigger) => label("RB"),
        (_, B::LeftTrigger2) => label("LT"),
        (_, B::RightTrigger2) => label("RT"),
        (_, B::Select) => label("View"),
        (_, B::Start) => label("Menu"),
        (_, B::LeftThumb) => label("LS"),
        (_, B::RightThumb) => label("RS"),
        (_, B::Mode) => label("Xbox"),
        (_, other) => label(&format!("{other:?}")),
    }
}

/// The keys or buttons that do `keys` on this device.
fn badges(device: ActiveDevice, keys: Keys) -> Vec<Badge> {
    use GamepadButton as B;
    let pad = |buttons: &[GamepadButton]| {
        buttons
            .iter()
            .map(|&button| button_badge(device, button))
            .collect()
    };
    match (device, keys) {
        (ActiveDevice::Keyboard, Keys::Confirm) => vec![text(device, "Enter")],
        (ActiveDevice::Keyboard, Keys::Back) => vec![text(device, "Esc")],
        (ActiveDevice::Keyboard, Keys::Tabs) => vec![text(device, "Tab")],
        (_, Keys::Confirm) => pad(&[B::South]),
        (_, Keys::Back) => pad(&[B::East]),
        (_, Keys::Tabs) => pad(&[B::LeftTrigger, B::RightTrigger]),
    }
}

/// The legend of a menu: each kind of key with what it does there.
pub fn menu_prompts(device: ActiveDevice, hints: &[(Keys, &str)]) -> Vec<KeyPrompt> {
    hints
        .iter()
        .map(|&(keys, action)| KeyPrompt {
            badges: badges(device, keys),
            action: action.into(),
        })
        .collect()
}

/// The legend of the race, with the keys or buttons the player has chosen.
pub fn race_prompts(device: ActiveDevice, lang: Language, controls: &Controls) -> Vec<KeyPrompt> {
    let first = |action: Action| -> Vec<Badge> {
        let binding = controls.get(action);
        match device {
            ActiveDevice::Keyboard => binding.keys.first().map(|&key| key_badge(key, lang)),
            _ => binding
                .buttons
                .first()
                .map(|&button| button_badge(device, button)),
        }
        .into_iter()
        .collect()
    };
    let steer = match device {
        ActiveDevice::Keyboard => [first(Action::Left), first(Action::Right)].concat(),
        // The left stick always steers, whatever the buttons.
        _ => vec![glyphs(device, &[Glyph::Stick])],
    };
    let mut prompts = vec![
        (first(Action::Throttle), lang.action_throttle()),
        (first(Action::Brake), lang.hint_brake()),
        (steer, lang.hint_steer()),
        (first(Action::Wheelie), lang.action_wheelie()),
        (first(Action::Restart), lang.restart()),
        (first(Action::Pause), lang.action_pause()),
    ];
    // M turns the sound off and on; it has no button.
    if device == ActiveDevice::Keyboard {
        prompts.push((vec![text(device, "M")], lang.hint_sound()));
    }
    prompts
        .into_iter()
        // An action with nothing to do it is left out.
        .filter(|(badges, _)| !badges.is_empty())
        .map(|(badges, action)| KeyPrompt {
            badges,
            action: action.into(),
        })
        .collect()
}

/// The legend, centred at the bottom of `parent` (which fills the screen).
pub fn spawn_legend(
    parent: &mut ChildSpawnerCommands,
    font: &UiFont,
    icons: &UiIcons,
    device: ActiveDevice,
    prompts: &[KeyPrompt],
) {
    let look = style(device);
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(18),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(24),
                row_gap: px(8),
                padding: UiRect::horizontal(px(16)),
                ..default()
            },
            ZIndex(150),
        ))
        .with_children(|bar| {
            for prompt in prompts {
                bar.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(4),
                    ..default()
                })
                .with_children(|item| {
                    for badge in &prompt.badges {
                        spawn_badge(item, font, icons, &look, device, badge);
                    }
                    item.spawn((
                        Text::new(&prompt.action),
                        text_font(font, 16.0),
                        TextColor(Color::WHITE),
                        TextShadow {
                            offset: Vec2::splat(1.5),
                            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
                        },
                        Node {
                            margin: UiRect::left(px(4)),
                            ..default()
                        },
                    ));
                });
            }
        });
}

/// The legend of a screen (track, bike, players), built again when the
/// device in use or the language changes.
#[derive(Component)]
pub struct ScreenLegend;

/// Shows `wanted` as the legend of the screen of `state`, unless it is shown
/// already.
#[allow(clippy::too_many_arguments)]
pub fn show_screen_legend<S: States>(
    commands: &mut Commands,
    shown: &Query<Entity, With<ScreenLegend>>,
    last: &mut Option<Vec<KeyPrompt>>,
    wanted: Vec<KeyPrompt>,
    device: ActiveDevice,
    font: &UiFont,
    icons: &UiIcons,
    state: S,
) {
    if last.as_ref() == Some(&wanted) && !shown.is_empty() {
        return;
    }
    for entity in shown {
        commands.entity(entity).despawn();
    }
    commands
        .spawn((
            ScreenLegend,
            DespawnOnExit(state),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|screen| spawn_legend(screen, font, icons, device, &wanted));
    *last = Some(wanted);
}

/// Height of the badges (px).
const BADGE: f32 = 26.0;

fn spawn_badge(
    parent: &mut ChildSpawnerCommands,
    font: &UiFont,
    icons: &UiIcons,
    look: &Style,
    device: ActiveDevice,
    badge: &Badge,
) {
    // Keycaps have a deeper bottom edge.
    let bottom = if device == ActiveDevice::Keyboard {
        3.0
    } else {
        1.0
    };
    parent
        .spawn((
            Node {
                min_width: px(BADGE),
                height: px(BADGE),
                padding: UiRect::horizontal(px(if badge.round { 0.0 } else { 7.0 })),
                border: UiRect {
                    left: px(1),
                    right: px(1),
                    top: px(1),
                    bottom: px(bottom),
                },
                border_radius: BorderRadius::all(px(if badge.round { BADGE / 2.0 } else { 6.0 })),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(2),
                ..default()
            },
            BackgroundColor(look.face),
            BorderColor::all(look.edge),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.45),
                px(0),
                px(2),
                px(0),
                px(4),
            ),
        ))
        .with_children(|inside| match &badge.label {
            Label::Text(label) => {
                inside.spawn((
                    Text::new(label),
                    text_font(font, 14.0),
                    TextColor(badge.ink),
                ));
            }
            Label::Glyphs(glyphs) => {
                for &glyph in glyphs {
                    inside.spawn((
                        ImageNode::new(icons.glyph(glyph)).with_color(badge.ink),
                        Node {
                            width: px(15),
                            height: px(15),
                            ..default()
                        },
                    ));
                }
            }
        });
}

/// Size of the symbol images (px).
const GLYPH_SIZE: u32 = 64;

/// The image of a symbol: white, to be tinted.
pub fn draw_glyph(glyph: Glyph) -> Image {
    let size = GLYPH_SIZE;
    let mut alpha = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            // From -1 to 1, y up.
            let p = Vec2::new(
                (x as f32 + 0.5) / size as f32 * 2.0 - 1.0,
                1.0 - (y as f32 + 0.5) / size as f32 * 2.0,
            );
            let distance = glyph_distance(glyph, p);
            // Antialiased edge, one pixel wide.
            let pixel = 2.0 / size as f32;
            let inside = (0.5 - distance / pixel).clamp(0.0, 1.0);
            alpha.push((inside * 255.0).round() as u8);
        }
    }
    let drawing = Coverage {
        width: size,
        height: size,
        alpha,
    };
    // Shown much smaller than drawn: mipmaps keep it smooth.
    white_image(&drawing, true)
}

/// Signed distance from `p` to the shape of the symbol (negative inside).
fn glyph_distance(glyph: Glyph, p: Vec2) -> f32 {
    let arrow = |p: Vec2| {
        triangle(
            p,
            Vec2::new(0.0, 0.66),
            Vec2::new(-0.66, -0.42),
            Vec2::new(0.66, -0.42),
        ) - 0.06
    };
    let stroke = 0.15;
    match glyph {
        Glyph::Up => arrow(p),
        Glyph::Down => arrow(Vec2::new(p.x, -p.y)),
        Glyph::Left => arrow(Vec2::new(-p.y, -p.x)),
        Glyph::Right => arrow(Vec2::new(p.y, p.x)),
        Glyph::Cross => {
            let a = segment(p, Vec2::splat(-0.55), Vec2::splat(0.55));
            let b = segment(p, Vec2::new(-0.55, 0.55), Vec2::new(0.55, -0.55));
            a.min(b) - stroke
        }
        Glyph::Circle => (p.length() - 0.6).abs() - stroke,
        Glyph::Square => {
            let q = p.abs() - Vec2::splat(0.56);
            let outside = q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0);
            outside.abs() - stroke
        }
        Glyph::Triangle => {
            triangle(
                p,
                Vec2::new(0.0, 0.62),
                Vec2::new(-0.66, -0.5),
                Vec2::new(0.66, -0.5),
            )
            .abs()
                - stroke
        }
        Glyph::Stick => {
            let ring = (p.length() - 0.78).abs() - 0.1;
            let cap = p.length() - 0.38;
            ring.min(cap)
        }
    }
}

/// Distance from `p` to the segment from `a` to `b`.
fn segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let (pa, ba) = (p - a, b - a);
    let h = (pa.dot(ba) / ba.length_squared()).clamp(0.0, 1.0);
    (pa - ba * h).length()
}

/// Signed distance from `p` to the triangle `a`, `b`, `c`.
fn triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let edges = [(a, b), (b, c), (c, a)];
    let distance = edges
        .iter()
        .map(|&(from, to)| segment(p, from, to))
        .fold(f32::MAX, f32::min);
    // Inside when on the same side of all three edges.
    let side = |from: Vec2, to: Vec2| (to - from).perp_dot(p - from);
    let sides = edges.map(|(from, to)| side(from, to));
    let inside = sides.iter().all(|&s| s >= 0.0) || sides.iter().all(|&s| s <= 0.0);
    if inside { -distance } else { distance }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_art::has_char;

    const DEVICES: [ActiveDevice; 4] = [
        ActiveDevice::Keyboard,
        ActiveDevice::Xbox,
        ActiveDevice::PlayStation,
        ActiveDevice::Switch,
    ];

    /// Everything written in the legend can be drawn with the UI font.
    fn drawable(prompts: &[KeyPrompt]) -> bool {
        prompts.iter().all(|prompt| {
            prompt.action.chars().all(has_char)
                && prompt.badges.iter().all(|badge| match &badge.label {
                    Label::Text(text) => !text.is_empty() && text.chars().all(has_char),
                    Label::Glyphs(glyphs) => !glyphs.is_empty(),
                })
        })
    }

    #[test]
    fn menus_say_the_same_on_every_device() {
        for lang in [Language::Spanish, Language::English] {
            let hints = [
                (Keys::Confirm, lang.hint_accept()),
                (Keys::Back, lang.back()),
                (Keys::Tabs, lang.hint_section()),
            ];
            let keyboard = menu_prompts(ActiveDevice::Keyboard, &hints);
            for device in DEVICES {
                let prompts = menu_prompts(device, &hints);
                assert!(drawable(&prompts), "{device:?}: {prompts:?}");
                let actions =
                    |p: &[KeyPrompt]| p.iter().map(|p| p.action.clone()).collect::<Vec<_>>();
                assert_eq!(actions(&prompts), actions(&keyboard));
            }
        }
    }

    #[test]
    fn each_controller_shows_its_own_buttons() {
        let confirm = |device| badges(device, Keys::Confirm).remove(0);
        assert_eq!(
            confirm(ActiveDevice::Keyboard).label,
            Label::Text("Enter".into())
        );
        assert_eq!(
            confirm(ActiveDevice::PlayStation).label,
            Label::Glyphs(vec![Glyph::Cross])
        );
        assert_eq!(confirm(ActiveDevice::Xbox).label, Label::Text("A".into()));
        assert_eq!(confirm(ActiveDevice::Switch).label, Label::Text("B".into()));
        assert!(confirm(ActiveDevice::PlayStation).round);
        let tabs = badges(ActiveDevice::PlayStation, Keys::Tabs);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].label, Label::Text("L1".into()));
    }

    #[test]
    fn the_race_legend_shows_the_chosen_keys() {
        let mut controls = Controls::default();
        for lang in [Language::Spanish, Language::English] {
            for device in DEVICES {
                let prompts = race_prompts(device, lang, &controls);
                assert!(drawable(&prompts), "{device:?}: {prompts:?}");
            }
        }
        let keyboard = race_prompts(ActiveDevice::Keyboard, Language::Spanish, &controls);
        let wheelie = keyboard
            .iter()
            .find(|p| p.action == "Caballito")
            .expect("the wheelie is in the legend");
        assert_eq!(wheelie.badges[0].label, Label::Text("Espacio".into()));

        controls.bind_key(Action::Wheelie, KeyCode::KeyJ);
        controls.bind_button(Action::Throttle, GamepadButton::RightTrigger);
        let keyboard = race_prompts(ActiveDevice::Keyboard, Language::English, &controls);
        let wheelie = keyboard.iter().find(|p| p.action == "Wheelie").unwrap();
        assert_eq!(wheelie.badges[0].label, Label::Text("J".into()));
        let pad = race_prompts(ActiveDevice::PlayStation, Language::English, &controls);
        assert_eq!(pad[0].action, "Throttle");
        assert_eq!(pad[0].badges[0].label, Label::Text("R1".into()));
        // Sound only has a key.
        assert!(!pad.iter().any(|p| p.action == "Sound"));
    }

    #[test]
    fn symbols_are_filled_where_expected() {
        let inside = |glyph, x: f32, y: f32| glyph_distance(glyph, Vec2::new(x, y)) < 0.0;
        assert!(inside(Glyph::Up, 0.0, 0.0));
        assert!(!inside(Glyph::Up, 0.0, -0.8));
        assert!(inside(Glyph::Down, 0.0, -0.5));
        assert!(inside(Glyph::Left, -0.5, 0.0));
        assert!(!inside(Glyph::Left, 0.6, 0.0));
        assert!(inside(Glyph::Right, 0.5, 0.0));
        assert!(inside(Glyph::Cross, 0.0, 0.0));
        assert!(!inside(Glyph::Circle, 0.0, 0.0));
        assert!(inside(Glyph::Circle, 0.6, 0.0));
        assert!(!inside(Glyph::Square, 0.0, 0.0));
        assert!(inside(Glyph::Square, 0.56, 0.0));
        assert!(!inside(Glyph::Triangle, 0.0, 0.0));
        assert!(inside(Glyph::Stick, 0.0, 0.0));
    }
}
