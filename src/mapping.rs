//! The mapping of the controls, a view of Options > Control: the keyboard
//! or the gamepad (the one in use when it opens, or chosen with the keys
//! that change section), drawn with the keys or buttons that do something
//! tinted and the ones held down lit, each action with its keys or buttons
//! underneath, and two buttons: "Remap" waits for the key or button to
//! remap and then for the new one, or several pressed together, that is to
//! do its action instead; "Reset" puts the device's back as they were at
//! first. The keyboard and the gamepad are mapped apart.

use bevy::prelude::*;

use crate::hud::UiFont;
use crate::legend::{Glyph, Label, button_badge};
use crate::locale::Language;
use crate::settings::{ACTIONS, Action, Controls, key_name};
use crate::ui::{ActiveDevice, MenuSound, Request, UiIcons, text_font};

/// Seconds to press the key or button before the remapping is called off.
const CAPTURE_TIME: f32 = 8.0;
/// Seconds what was done is shown.
const NOTE_TIME: f32 = 3.0;

/// Which device is being mapped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapDevice {
    Keyboard,
    Gamepad,
}

const DEVICES: [MapDevice; 2] = [MapDevice::Keyboard, MapDevice::Gamepad];

impl MapDevice {
    /// The one `device` is.
    pub fn of(device: ActiveDevice) -> Self {
        match device {
            ActiveDevice::Keyboard => MapDevice::Keyboard,
            _ => MapDevice::Gamepad,
        }
    }

    fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (MapDevice::Keyboard, Language::Spanish) => "Teclado",
            (MapDevice::Keyboard, Language::English) => "Keyboard",
            (MapDevice::Gamepad, Language::Spanish) => "Mando",
            (MapDevice::Gamepad, Language::English) => "Gamepad",
        }
    }
}

/// What is chosen in the view.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapFocus {
    /// The row of devices: across, the other one.
    Devices,
    Remap,
    Reset,
    Back,
}

/// A key or a gamepad button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pressed {
    Key(KeyCode),
    Button(GamepadButton),
}

/// Where the remapping is at.
#[derive(Clone, PartialEq, Debug)]
pub enum Step {
    Idle,
    /// Waiting for the key or button to remap, since then (s).
    Choose {
        since: f32,
    },
    /// Waiting for the new key or combination that is to do `action` in
    /// place of `from`; those pressed so far.
    Assign {
        since: f32,
        action: Action,
        from: Pressed,
        held: Vec<Pressed>,
    },
}

/// What was just done, to say so for a moment.
#[derive(Clone, PartialEq, Debug)]
pub enum Note {
    Remapped {
        action: Action,
        from: Pressed,
        to: Vec<Pressed>,
    },
    /// The key or button chosen does nothing.
    Unbound(Pressed),
    Cancelled,
    Reset(MapDevice),
}

/// The view, while it is open.
#[derive(Clone, PartialEq, Debug)]
pub struct Mapping {
    pub device: MapDevice,
    pub focus: MapFocus,
    pub step: Step,
    /// What was just done, and until when (s).
    pub note: Option<(Note, f32)>,
}

impl Mapping {
    /// The view of `device`, with "Remap" chosen.
    pub fn open(device: MapDevice) -> Self {
        Self {
            device,
            focus: MapFocus::Remap,
            step: Step::Idle,
            note: None,
        }
    }

    /// Whether it waits for a key or a button.
    pub fn capturing(&self) -> bool {
        self.step != Step::Idle
    }

    /// Moves round the view: up and down through the devices, the buttons
    /// and Back; across, the other device or the other button. Whether it
    /// did anything.
    fn navigate(&mut self, request: &Request) -> bool {
        use MapFocus::*;
        let before = (self.focus, self.device);
        if request.tab != 0 {
            self.device = crate::settings::cycle(&DEVICES, self.device, request.tab);
        }
        if request.step != 0 {
            let row = |focus| match focus {
                Devices => 0,
                Remap | Reset => 1,
                Back => 2,
            };
            self.focus = match (row(self.focus) + request.step).rem_euclid(3) {
                0 => Devices,
                1 => Remap,
                _ => Back,
            };
        } else if request.side != 0 {
            match self.focus {
                Devices => {
                    self.device = crate::settings::cycle(&DEVICES, self.device, request.side);
                }
                Remap => self.focus = Reset,
                Reset => self.focus = Remap,
                Back => {}
            }
        }
        (self.focus, self.device) != before
    }
}

/// The keys and buttons of this frame, as the view reads them.
pub struct Held<'a> {
    pub keyboard: &'a ButtonInput<KeyCode>,
    pub gamepads: Vec<&'a Gamepad>,
}

impl Held<'_> {
    /// The keys or buttons of `device` just pressed: not M, which mutes,
    /// nor Esc, which calls off a remapping.
    fn just_pressed(&self, device: MapDevice) -> Vec<Pressed> {
        match device {
            MapDevice::Keyboard => self
                .keyboard
                .get_just_pressed()
                .filter(|&&key| key != KeyCode::KeyM && key != KeyCode::Escape)
                .map(|&key| Pressed::Key(key))
                .collect(),
            MapDevice::Gamepad => self
                .gamepads
                .iter()
                .flat_map(|gamepad| gamepad.get_just_pressed())
                .map(|&button| Pressed::Button(button))
                .collect(),
        }
    }

    /// Whether `input` is held down.
    pub fn is_held(&self, input: Pressed) -> bool {
        match input {
            Pressed::Key(key) => self.keyboard.pressed(key),
            Pressed::Button(button) => self.gamepads.iter().any(|pad| pad.pressed(button)),
        }
    }
}

/// Adds the keys or buttons of `device` just pressed to `held`; once none
/// of them is held down any more, they are the combination (one, or several
/// pressed together).
fn combination(held: &mut Vec<Pressed>, input: &Held, device: MapDevice) -> Option<Vec<Pressed>> {
    for pressed in input.just_pressed(device) {
        if !held.contains(&pressed) {
            held.push(pressed);
        }
    }
    (!held.is_empty() && !held.iter().any(|&pressed| input.is_held(pressed))).then(|| held.clone())
}

/// What `from` did is done with `to` instead (see [`Controls::remap`]).
fn remap(controls: &mut Controls, from: Pressed, to: &[Pressed]) -> Option<Action> {
    match from {
        Pressed::Key(key) => {
            let keys: Vec<KeyCode> = to
                .iter()
                .filter_map(|pressed| match pressed {
                    Pressed::Key(key) => Some(*key),
                    Pressed::Button(_) => None,
                })
                .collect();
            controls.remap(key, &keys)
        }
        Pressed::Button(button) => {
            let buttons: Vec<GamepadButton> = to
                .iter()
                .filter_map(|pressed| match pressed {
                    Pressed::Button(button) => Some(*button),
                    Pressed::Key(_) => None,
                })
                .collect();
            controls.remap(button, &buttons)
        }
    }
}

/// The action of a key or button, if it has one.
fn action_of(controls: &Controls, input: Pressed) -> Option<Action> {
    match input {
        Pressed::Key(key) => controls.action_of(key),
        Pressed::Button(button) => controls.action_of(button),
    }
}

/// Works the view with this frame's `request` and keys and buttons
/// (`input`). Whether it is to close, back to the options of Control.
pub fn use_mapping(
    mapping: &mut Mapping,
    request: &Request,
    input: &Held,
    controls: &mut Controls,
    now: f32,
    sounds: &mut MessageWriter<MenuSound>,
) -> bool {
    if mapping.note.as_ref().is_some_and(|(_, until)| now > *until) {
        mapping.note = None;
    }
    let cancel = input.keyboard.just_pressed(KeyCode::Escape);
    let note = |note: Note| Some((note, now + NOTE_TIME));
    match mapping.step.clone() {
        Step::Choose { since } => {
            if cancel || now - since > CAPTURE_TIME {
                mapping.step = Step::Idle;
                mapping.note = note(Note::Cancelled);
                sounds.write(MenuSound::Move);
            } else if let Some(&from) = input.just_pressed(mapping.device).first() {
                match action_of(controls, from) {
                    Some(action) => {
                        mapping.step = Step::Assign {
                            since: now,
                            action,
                            from,
                            held: Vec::new(),
                        };
                        sounds.write(MenuSound::Confirm);
                    }
                    None => {
                        mapping.step = Step::Idle;
                        mapping.note = note(Note::Unbound(from));
                        sounds.write(MenuSound::Move);
                    }
                }
            }
            false
        }
        Step::Assign {
            since,
            action,
            from,
            mut held,
        } => {
            if cancel || (held.is_empty() && now - since > CAPTURE_TIME) {
                mapping.step = Step::Idle;
                mapping.note = note(Note::Cancelled);
                sounds.write(MenuSound::Move);
            } else if let Some(to) = combination(&mut held, input, mapping.device) {
                remap(controls, from, &to);
                mapping.step = Step::Idle;
                mapping.note = note(Note::Remapped { action, from, to });
                sounds.write(MenuSound::Confirm);
            } else {
                mapping.step = Step::Assign {
                    since,
                    action,
                    from,
                    held,
                };
            }
            false
        }
        Step::Idle => {
            if request.back {
                sounds.write(MenuSound::Move);
                return true;
            }
            if mapping.navigate(request) {
                sounds.write(MenuSound::Move);
                return false;
            }
            if !request.confirm {
                return false;
            }
            match mapping.focus {
                MapFocus::Devices => {
                    mapping.focus = MapFocus::Remap;
                    sounds.write(MenuSound::Move);
                }
                MapFocus::Remap => {
                    mapping.step = Step::Choose { since: now };
                    mapping.note = None;
                    sounds.write(MenuSound::Confirm);
                }
                MapFocus::Reset => {
                    match mapping.device {
                        MapDevice::Keyboard => controls.reset::<KeyCode>(),
                        MapDevice::Gamepad => controls.reset::<GamepadButton>(),
                    }
                    mapping.note = note(Note::Reset(mapping.device));
                    sounds.write(MenuSound::Confirm);
                }
                MapFocus::Back => {
                    sounds.write(MenuSound::Move);
                    return true;
                }
            }
            false
        }
    }
}

// The view.

/// The colour of Control, whose view this is.
const ACCENT: Color = Color::srgb(0.22, 0.76, 1.0);
/// The key or button being remapped, and the new ones pressed.
const FROM: Color = Color::srgb(1.0, 0.55, 0.2);
const NEW: Color = Color::srgb(1.0, 0.82, 0.2);
const INK: Color = Color::srgb(0.06, 0.05, 0.04);

/// What the view is drawn from.
pub struct MappingLook<'a> {
    pub font: &'a UiFont,
    pub icons: &'a UiIcons,
    pub lang: Language,
    /// How the buttons are marked on the gamepad (Xbox, PlayStation...).
    pub pad: ActiveDevice,
    pub controls: &'a Controls,
    pub held: &'a Held<'a>,
}

/// What a key or button says on it.
enum Mark {
    Text(String),
    Glyph(Glyph),
}

/// The keys of the keyboard, row by row, and how wide each is (in keys).
const KEY_ROWS: [&[(KeyCode, f32)]; 5] = {
    use KeyCode as K;
    [
        &[
            (K::Escape, 1.25),
            (K::Digit1, 1.0),
            (K::Digit2, 1.0),
            (K::Digit3, 1.0),
            (K::Digit4, 1.0),
            (K::Digit5, 1.0),
            (K::Digit6, 1.0),
            (K::Digit7, 1.0),
            (K::Digit8, 1.0),
            (K::Digit9, 1.0),
            (K::Digit0, 1.0),
            (K::Minus, 1.0),
            (K::Equal, 1.0),
            (K::Backspace, 1.75),
        ],
        &[
            (K::Tab, 1.5),
            (K::KeyQ, 1.0),
            (K::KeyW, 1.0),
            (K::KeyE, 1.0),
            (K::KeyR, 1.0),
            (K::KeyT, 1.0),
            (K::KeyY, 1.0),
            (K::KeyU, 1.0),
            (K::KeyI, 1.0),
            (K::KeyO, 1.0),
            (K::KeyP, 1.0),
            (K::BracketLeft, 1.0),
            (K::BracketRight, 1.0),
            (K::Backslash, 1.5),
        ],
        &[
            (K::CapsLock, 1.75),
            (K::KeyA, 1.0),
            (K::KeyS, 1.0),
            (K::KeyD, 1.0),
            (K::KeyF, 1.0),
            (K::KeyG, 1.0),
            (K::KeyH, 1.0),
            (K::KeyJ, 1.0),
            (K::KeyK, 1.0),
            (K::KeyL, 1.0),
            (K::Semicolon, 1.0),
            (K::Quote, 1.0),
            (K::Enter, 2.25),
        ],
        &[
            (K::ShiftLeft, 2.25),
            (K::KeyZ, 1.0),
            (K::KeyX, 1.0),
            (K::KeyC, 1.0),
            (K::KeyV, 1.0),
            (K::KeyB, 1.0),
            (K::KeyN, 1.0),
            (K::KeyM, 1.0),
            (K::Comma, 1.0),
            (K::Period, 1.0),
            (K::Slash, 1.0),
            (K::ShiftRight, 2.75),
        ],
        &[
            (K::ControlLeft, 1.5),
            (K::AltLeft, 1.25),
            (K::SuperLeft, 1.25),
            (K::Space, 7.0),
            (K::SuperRight, 1.25),
            (K::AltRight, 1.25),
            (K::ControlRight, 1.5),
        ],
    ]
};

/// The size of a key (px) and the gap between keys.
const KEY: f32 = 30.0;
const KEY_GAP: f32 = 3.0;

/// What a key says on it: short names for the wide ones.
fn key_mark(key: KeyCode, lang: Language) -> Mark {
    use KeyCode as K;
    let english = lang == Language::English;
    let mac = cfg!(target_os = "macos");
    let text = match key {
        K::ArrowUp => return Mark::Glyph(Glyph::Up),
        K::ArrowDown => return Mark::Glyph(Glyph::Down),
        K::ArrowLeft => return Mark::Glyph(Glyph::Left),
        K::ArrowRight => return Mark::Glyph(Glyph::Right),
        K::Backspace if english => "Bksp",
        K::Backspace => "Borrar",
        K::CapsLock if english => "Caps",
        K::CapsLock => "Bloq",
        K::ShiftLeft | K::ShiftRight if english => "Shift",
        K::ShiftLeft | K::ShiftRight => "Mayús",
        K::Space if english => "Space",
        K::Space => "Espacio",
        K::ControlLeft | K::ControlRight => "Ctrl",
        K::AltLeft | K::AltRight if mac => "Opt",
        K::AltLeft | K::AltRight => "Alt",
        K::SuperLeft | K::SuperRight if mac => "Cmd",
        K::SuperLeft | K::SuperRight => "Win",
        other => return Mark::Text(key_name(other, lang)),
    };
    Mark::Text(text.into())
}

/// The gamepad's buttons and where they are drawn: left and top, width and
/// height (px), and whether round.
const PAD: [(GamepadButton, f32, f32, f32, f32, bool); 17] = {
    use GamepadButton as B;
    [
        (B::LeftTrigger2, 96.0, 0.0, 64.0, 22.0, false),
        (B::LeftTrigger, 96.0, 26.0, 64.0, 18.0, false),
        (B::RightTrigger2, 360.0, 0.0, 64.0, 22.0, false),
        (B::RightTrigger, 360.0, 26.0, 64.0, 18.0, false),
        (B::DPadUp, 133.0, 76.0, 26.0, 26.0, false),
        (B::DPadLeft, 105.0, 104.0, 26.0, 26.0, false),
        (B::DPadRight, 161.0, 104.0, 26.0, 26.0, false),
        (B::DPadDown, 133.0, 132.0, 26.0, 26.0, false),
        (B::Select, 214.0, 84.0, 40.0, 20.0, false),
        (B::Start, 266.0, 84.0, 40.0, 20.0, false),
        (B::Mode, 246.0, 112.0, 28.0, 28.0, true),
        (B::LeftThumb, 196.0, 142.0, 42.0, 42.0, true),
        (B::RightThumb, 282.0, 142.0, 42.0, 42.0, true),
        (B::North, 361.0, 72.0, 30.0, 30.0, true),
        (B::West, 331.0, 102.0, 30.0, 30.0, true),
        (B::East, 391.0, 102.0, 30.0, 30.0, true),
        (B::South, 361.0, 132.0, 30.0, 30.0, true),
    ]
};

/// Spawns the view in the options panel: its title, the devices, the one
/// chosen drawn, what each action does on it, what is going on, the
/// buttons and Back.
pub fn spawn_mapping(panel: &mut ChildSpawnerCommands, mapping: &Mapping, look: &MappingLook) {
    let lang = look.lang;
    panel.spawn((
        Text::new(match lang {
            Language::Spanish => "OPCIONES  >  CONTROL",
            Language::English => "OPTIONS  >  CONTROL",
        }),
        text_font(look.font, 32.0),
        TextShadow {
            offset: Vec2::splat(2.5),
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
        },
        Node {
            align_self: AlignSelf::Center,
            ..default()
        },
    ));
    // The devices, as the sections of the options.
    panel
        .spawn(Node {
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        })
        .with_children(|row| {
            for device in DEVICES {
                let active = device == mapping.device;
                let edge = if active && mapping.focus == MapFocus::Devices {
                    ACCENT.mix(&Color::WHITE, 0.6)
                } else if active {
                    ACCENT
                } else {
                    ACCENT.with_alpha(0.16)
                };
                row.spawn((
                    Node {
                        padding: UiRect::axes(px(18), px(5)),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(if active {
                        ACCENT
                    } else {
                        ACCENT.with_alpha(0.16)
                    }),
                    BorderColor::all(edge),
                ))
                .with_child((
                    Text::new(device.name(lang)),
                    text_font(look.font, 18.0),
                    TextColor(if active {
                        INK
                    } else {
                        ACCENT.mix(&Color::WHITE, 0.55)
                    }),
                ));
            }
        });
    panel.spawn((
        Node {
            height: px(2),
            min_height: px(2),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(ACCENT.with_alpha(0.7)),
    ));
    // The device, drawn.
    panel
        .spawn(Node {
            justify_content: JustifyContent::Center,
            height: px(196),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|area| match mapping.device {
            MapDevice::Keyboard => spawn_keyboard(area, mapping, look),
            MapDevice::Gamepad => spawn_gamepad(area, mapping, look),
        });
    // What is going on.
    let (status, color) = status(mapping, look);
    panel.spawn((
        Text::new(status),
        text_font(look.font, 17.0),
        TextColor(color),
        TextLayout::justify(Justify::Center),
        Node {
            align_self: AlignSelf::Center,
            min_height: px(22),
            ..default()
        },
    ));
    // What each action does on it, in two columns.
    panel
        .spawn(Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
            column_gap: px(28),
            row_gap: px(2),
            padding: UiRect::horizontal(px(20)),
            ..default()
        })
        .with_children(|grid| {
            for action in ACTIONS {
                let busy = matches!(&mapping.step, Step::Assign { action: a, .. } if *a == action);
                let done = matches!(
                    &mapping.note,
                    Some((Note::Remapped { action: a, .. }, _)) if *a == action
                );
                let color = if busy {
                    FROM
                } else if done {
                    NEW
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.85)
                };
                let value = match mapping.device {
                    MapDevice::Keyboard => look.controls.describe_keys(action, lang),
                    MapDevice::Gamepad => look.controls.describe_buttons(action, lang),
                };
                grid.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        Text::new(action.name(lang)),
                        text_font(look.font, 15.0),
                        TextColor(color),
                    ));
                    line.spawn((
                        Text::new(value),
                        text_font(look.font, 15.0),
                        TextColor(color),
                    ));
                });
            }
        });
    // Remap and reset, side by side.
    panel
        .spawn(Node {
            justify_content: JustifyContent::Center,
            column_gap: px(16),
            margin: UiRect::top(px(6)),
            ..default()
        })
        .with_children(|row| {
            let remap = match (&mapping.step, lang) {
                (Step::Idle, Language::Spanish) => "Reasignar",
                (Step::Idle, Language::English) => "Remap",
                (_, Language::Spanish) => "Esperando...",
                (_, Language::English) => "Waiting...",
            };
            let reset = match lang {
                Language::Spanish => "Restablecer",
                Language::English => "Reset",
            };
            for (focus, label) in [(MapFocus::Remap, remap), (MapFocus::Reset, reset)] {
                spawn_button(
                    row,
                    look,
                    label,
                    mapping.focus == focus,
                    mapping.capturing(),
                );
            }
        });
    // Back, at the foot.
    let chosen = mapping.focus == MapFocus::Back && !mapping.capturing();
    panel
        .spawn((
            Node {
                justify_content: JustifyContent::Center,
                padding: UiRect::vertical(px(5)),
                margin: UiRect::top(Val::Auto),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(if chosen { ACCENT } else { Color::NONE }),
        ))
        .with_child((
            Text::new(lang.back()),
            text_font(look.font, 20.0),
            TextColor(if chosen { INK } else { Color::WHITE }),
        ));
}

/// A button of the view: filled in the colour of Control when chosen.
fn spawn_button(
    row: &mut ChildSpawnerCommands,
    look: &MappingLook,
    label: &str,
    chosen: bool,
    waiting: bool,
) {
    let fill = match (chosen, waiting) {
        (true, true) => NEW,
        (true, false) => ACCENT,
        _ => Color::srgba(1.0, 1.0, 1.0, 0.08),
    };
    row.spawn((
        Node {
            min_width: px(170),
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(20), px(6)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(ACCENT.with_alpha(0.5)),
    ))
    .with_child((
        Text::new(label),
        text_font(look.font, 19.0),
        TextColor(if chosen { INK } else { Color::WHITE }),
    ));
}

/// How a key or button is drawn: lit if held down (or new, or the one being
/// remapped), tinted if it does something.
fn shade(input: Pressed, mapping: &Mapping, look: &MappingLook) -> (Color, Color, Color) {
    let (from, new) = match &mapping.step {
        Step::Assign { from, held, .. } => (Some(*from), held.contains(&input)),
        _ => (None, false),
    };
    if new {
        (NEW, NEW, INK)
    } else if from == Some(input) {
        (FROM, FROM, INK)
    } else if look.held.is_held(input) {
        (Color::WHITE, Color::WHITE, INK)
    } else if action_of(look.controls, input).is_some() {
        (
            ACCENT.with_alpha(0.32),
            ACCENT.with_alpha(0.75),
            Color::WHITE,
        )
    } else {
        (
            Color::srgba(1.0, 1.0, 1.0, 0.07),
            Color::srgba(1.0, 1.0, 1.0, 0.18),
            Color::srgba(1.0, 1.0, 1.0, 0.6),
        )
    }
}

/// A key or button: its shape, colours and what it says.
#[allow(clippy::too_many_arguments)]
fn spawn_cap(
    parent: &mut ChildSpawnerCommands,
    look: &MappingLook,
    node: Node,
    round: bool,
    (fill, edge, ink): (Color, Color, Color),
    mark: Mark,
    size: f32,
) {
    parent
        .spawn((
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: if round {
                    BorderRadius::MAX
                } else {
                    BorderRadius::all(px(5))
                },
                ..node
            },
            BackgroundColor(fill),
            BorderColor::all(edge),
        ))
        .with_children(|cap| match mark {
            Mark::Text(text) => {
                cap.spawn((Text::new(text), text_font(look.font, size), TextColor(ink)));
            }
            Mark::Glyph(glyph) => {
                cap.spawn((
                    ImageNode::new(look.icons.glyph(glyph)).with_color(ink),
                    Node {
                        width: px(size),
                        height: px(size),
                        ..default()
                    },
                ));
            }
        });
}

fn spawn_keyboard(area: &mut ChildSpawnerCommands, mapping: &Mapping, look: &MappingLook) {
    let key = |parent: &mut ChildSpawnerCommands, code: KeyCode, width: f32| {
        let input = Pressed::Key(code);
        spawn_cap(
            parent,
            look,
            Node {
                width: px(KEY * width + KEY_GAP * (width - 1.0).max(0.0)),
                height: px(KEY),
                ..default()
            },
            false,
            shade(input, mapping, look),
            key_mark(code, look.lang),
            if width > 1.0 { 12.0 } else { 14.0 },
        );
    };
    area.spawn(Node {
        column_gap: px(KEY_GAP * 4.0),
        align_items: AlignItems::FlexEnd,
        margin: UiRect::top(px(8)),
        ..default()
    })
    .with_children(|board| {
        board
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(KEY_GAP),
                ..default()
            })
            .with_children(|rows| {
                for row in KEY_ROWS {
                    rows.spawn(Node {
                        column_gap: px(KEY_GAP),
                        ..default()
                    })
                    .with_children(|keys| {
                        for &(code, width) in row {
                            key(keys, code, width);
                        }
                    });
                }
            });
        // The arrows, under the right of the keyboard.
        board
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(KEY_GAP),
                ..default()
            })
            .with_children(|arrows| {
                key(arrows, KeyCode::ArrowUp, 1.0);
                arrows
                    .spawn(Node {
                        column_gap: px(KEY_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        for code in [KeyCode::ArrowLeft, KeyCode::ArrowDown, KeyCode::ArrowRight] {
                            key(row, code, 1.0);
                        }
                    });
            });
    });
}

fn spawn_gamepad(area: &mut ChildSpawnerCommands, mapping: &Mapping, look: &MappingLook) {
    area.spawn(Node {
        width: px(520),
        height: px(196),
        ..default()
    })
    .with_children(|pad| {
        // The body, and its grips.
        for (left, top, width, height, radius) in [
            (70.0, 52.0, 380.0, 112.0, 40.0),
            (60.0, 100.0, 120.0, 92.0, 46.0),
            (340.0, 100.0, 120.0, 92.0, 46.0),
        ] {
            pad.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(width),
                    height: px(height),
                    border_radius: BorderRadius::all(px(radius)),
                    ..default()
                },
                // Opaque, so the grips and the body make one shape.
                BackgroundColor(Color::srgb(0.11, 0.13, 0.19)),
            ));
        }
        for (button, left, top, width, height, round) in PAD {
            let badge = button_badge(look.pad, button);
            let mark = match badge.label {
                Label::Text(text) => Mark::Text(text),
                Label::Glyphs(glyphs) => Mark::Glyph(glyphs[0]),
            };
            spawn_cap(
                pad,
                look,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(width),
                    height: px(height),
                    ..default()
                },
                round,
                shade(Pressed::Button(button), mapping, look),
                mark,
                if width > 40.0 || height < 24.0 {
                    12.0
                } else {
                    14.0
                },
            );
        }
    });
}

/// The name of a key or a button.
fn input_name(input: Pressed, look: &MappingLook) -> String {
    match input {
        Pressed::Key(key) => key_name(key, look.lang),
        Pressed::Button(button) => crate::settings::button_name(button, look.lang),
    }
}

/// The line that says what is going on: what to press, what was done, or
/// what the keys or buttons held down do.
fn status(mapping: &Mapping, look: &MappingLook) -> (String, Color) {
    let lang = look.lang;
    let keyboard = mapping.device == MapDevice::Keyboard;
    let names = |inputs: &[Pressed]| {
        inputs
            .iter()
            .map(|&input| input_name(input, look))
            .collect::<Vec<_>>()
            .join("+")
    };
    match &mapping.step {
        Step::Choose { .. } => (
            match (lang, keyboard) {
                (Language::Spanish, true) => {
                    "Pulsa la tecla que quieres reasignar   (Esc: cancelar)"
                }
                (Language::English, true) => "Press the key to remap   (Esc: cancel)",
                (Language::Spanish, false) => "Pulsa el botón que quieres reasignar",
                (Language::English, false) => "Press the button to remap",
            }
            .into(),
            NEW,
        ),
        Step::Assign { action, from, .. } => (
            match lang {
                Language::Spanish => format!(
                    "\"{}\" ({}): pulsa {}, o varios a la vez",
                    action.name(lang),
                    input_name(*from, look),
                    if keyboard {
                        "la tecla nueva"
                    } else {
                        "el botón nuevo"
                    }
                ),
                Language::English => format!(
                    "\"{}\" ({}): press the new {}, or several together",
                    action.name(lang),
                    input_name(*from, look),
                    if keyboard { "key" } else { "button" }
                ),
            },
            NEW,
        ),
        Step::Idle => match &mapping.note {
            Some((Note::Remapped { action, from, to }, _)) => (
                format!(
                    "{}:  {}  >  {}",
                    action.name(lang),
                    input_name(*from, look),
                    names(to)
                ),
                NEW,
            ),
            Some((Note::Unbound(input), _)) => (
                match lang {
                    Language::Spanish => format!("{} no hace nada", input_name(*input, look)),
                    Language::English => format!("{} does nothing", input_name(*input, look)),
                },
                Color::srgba(1.0, 1.0, 1.0, 0.7),
            ),
            Some((Note::Cancelled, _)) => (
                match lang {
                    Language::Spanish => "Cancelado".into(),
                    Language::English => "Cancelled".into(),
                },
                Color::srgba(1.0, 1.0, 1.0, 0.7),
            ),
            Some((Note::Reset(device), _)) => (
                match lang {
                    Language::Spanish => format!("{}: como al principio", device.name(lang)),
                    Language::English => format!("{}: back to the defaults", device.name(lang)),
                },
                NEW,
            ),
            // What the keys or buttons held down do.
            None => {
                let held: Vec<String> = match mapping.device {
                    MapDevice::Keyboard => look
                        .held
                        .keyboard
                        .get_pressed()
                        .map(|&key| Pressed::Key(key))
                        .collect::<Vec<_>>(),
                    MapDevice::Gamepad => PAD
                        .iter()
                        .map(|&(button, ..)| Pressed::Button(button))
                        .filter(|&input| look.held.is_held(input))
                        .collect(),
                }
                .into_iter()
                .map(|input| {
                    let does =
                        action_of(look.controls, input).map_or("—", |action| action.name(lang));
                    format!("{}: {does}", input_name(input, look))
                })
                .collect();
                (held.join("    "), Color::srgba(1.0, 1.0, 1.0, 0.8))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request::default()
    }

    /// The view at work: the keys pressed this frame (`press`), held from
    /// before (`hold`) and let go of (`release`).
    struct Bench {
        mapping: Mapping,
        controls: Controls,
        keyboard: ButtonInput<KeyCode>,
        now: f32,
    }

    impl Bench {
        fn new() -> Self {
            Self {
                mapping: Mapping::open(MapDevice::Keyboard),
                controls: Controls::default(),
                keyboard: ButtonInput::default(),
                now: 0.0,
            }
        }

        fn frame(&mut self, request: Request, press: &[KeyCode], release: &[KeyCode]) -> bool {
            self.keyboard.clear();
            for &key in press {
                self.keyboard.press(key);
            }
            for &key in release {
                self.keyboard.release(key);
            }
            self.now += 0.1;
            let held = Held {
                keyboard: &self.keyboard,
                gamepads: Vec::new(),
            };
            let mut world = World::new();
            world.init_resource::<Messages<MenuSound>>();
            let mut state =
                bevy::ecs::system::SystemState::<MessageWriter<MenuSound>>::new(&mut world);
            let mut sounds = state.get_mut(&mut world).unwrap();
            use_mapping(
                &mut self.mapping,
                &request,
                &held,
                &mut self.controls,
                self.now,
                &mut sounds,
            )
        }
    }

    #[test]
    fn remap_waits_for_the_key_to_remap_then_for_the_new_one() {
        let mut bench = Bench::new();
        let confirm = Request {
            confirm: true,
            ..request()
        };
        // Remap: then W (the throttle's).
        bench.frame(confirm, &[KeyCode::Enter], &[]);
        assert!(matches!(bench.mapping.step, Step::Choose { .. }));
        bench.frame(request(), &[KeyCode::KeyW], &[KeyCode::Enter]);
        assert!(matches!(
            bench.mapping.step,
            Step::Assign {
                action: Action::Throttle,
                ..
            }
        ));
        // Let go of W; then I is pressed, and counts once let go of.
        bench.frame(request(), &[], &[KeyCode::KeyW]);
        bench.frame(request(), &[KeyCode::KeyI], &[]);
        assert!(matches!(bench.mapping.step, Step::Assign { .. }));
        bench.frame(request(), &[], &[KeyCode::KeyI]);
        assert_eq!(bench.mapping.step, Step::Idle);
        assert_eq!(
            bench.controls.throttle.keys,
            [KeyCode::KeyI, KeyCode::ArrowUp]
        );
        assert!(matches!(
            bench.mapping.note,
            Some((Note::Remapped { .. }, _))
        ));
    }

    #[test]
    fn several_keys_pressed_together_make_a_combination() {
        let mut bench = Bench::new();
        bench.frame(
            Request {
                confirm: true,
                ..request()
            },
            &[KeyCode::Enter],
            &[],
        );
        bench.frame(request(), &[KeyCode::KeyR], &[KeyCode::Enter]);
        bench.frame(request(), &[], &[KeyCode::KeyR]);
        // Ctrl down, then R with it, then both let go of.
        bench.frame(request(), &[KeyCode::ControlLeft], &[]);
        bench.frame(request(), &[KeyCode::KeyR], &[]);
        bench.frame(request(), &[], &[KeyCode::ControlLeft]);
        assert!(
            matches!(bench.mapping.step, Step::Assign { .. }),
            "R still held"
        );
        bench.frame(request(), &[], &[KeyCode::KeyR]);
        assert_eq!(
            bench.controls.restart.key_combos,
            [vec![KeyCode::ControlLeft, KeyCode::KeyR]]
        );
    }

    #[test]
    fn a_key_that_does_nothing_or_esc_calls_it_off() {
        let mut bench = Bench::new();
        let confirm = Request {
            confirm: true,
            ..request()
        };
        bench.frame(confirm, &[KeyCode::Enter], &[]);
        bench.frame(request(), &[KeyCode::KeyK], &[KeyCode::Enter]);
        assert_eq!(bench.mapping.step, Step::Idle);
        assert!(matches!(bench.mapping.note, Some((Note::Unbound(_), _))));
        bench.frame(confirm, &[KeyCode::Enter], &[KeyCode::KeyK]);
        bench.frame(request(), &[KeyCode::Escape], &[KeyCode::Enter]);
        assert_eq!(bench.mapping.step, Step::Idle);
        assert!(matches!(bench.mapping.note, Some((Note::Cancelled, _))));
        assert_eq!(bench.controls, Controls::default());
    }

    #[test]
    fn the_view_is_walked_and_resets_its_device() {
        let mut bench = Bench::new();
        bench.controls.remap(KeyCode::KeyW, &[KeyCode::KeyI]);
        // Across from Remap: Reset.
        bench.frame(
            Request {
                side: 1,
                ..request()
            },
            &[],
            &[],
        );
        assert_eq!(bench.mapping.focus, MapFocus::Reset);
        bench.frame(
            Request {
                confirm: true,
                ..request()
            },
            &[],
            &[],
        );
        assert_eq!(bench.controls, Controls::default());
        // Up to the devices and across: the gamepad.
        bench.frame(
            Request {
                step: -1,
                ..request()
            },
            &[],
            &[],
        );
        assert_eq!(bench.mapping.focus, MapFocus::Devices);
        bench.frame(
            Request {
                side: 1,
                ..request()
            },
            &[],
            &[],
        );
        assert_eq!(bench.mapping.device, MapDevice::Gamepad);
        // Back closes it.
        assert!(bench.frame(
            Request {
                back: true,
                ..request()
            },
            &[],
            &[],
        ));
    }
}
