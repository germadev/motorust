//! The gamepads of the controls' mapping (see [`crate::mapping`]), drawn
//! from illustrations: a DualSense, a DualShock 4, a DualShock 3 and an Xbox
//! controller (`assets/ui/pads/`, from the anygamepad project). Each one is
//! an SVG whose controls carry data attributes:
//!
//! - `data-button="<button>"`: a control; its `.shape` elements are the
//!   button, its `.mark` elements the symbol or arrow printed on it. A
//!   `data-label` on a mark is the letter printed there (A, B, X, Y on the
//!   Xbox controller).
//! - `data-stick`: a stick, whose cap is its click button.
//! - `data-trigger` with `data-x`, `data-y`, `data-width`, `data-height`:
//!   where a trigger goes (the triggers are not in the front view).
//!
//! Bevy does not draw SVG. These are drawn here with zeno, the rasterizer
//! the menus, the speedometer and the clouds already use, and swash for
//! their few words: as much of SVG as the illustrations use (paths, circles,
//! rectangles, `use`, transforms, linear and radial gradients, a dotted
//! pattern, clip paths, drop shadows and a glow, text). Each illustration
//! is drawn once per size: the whole controller, and every button on its
//! own, white with its marks dark, cropped to it, so the view can tint it
//! over the controller (tinted if it does something, lit while held).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};
use swash::shape::ShapeContext;
use swash::zeno::{self, Cap, Fill, Format, Join, Mask, Placement, Stroke};

use crate::ui::ActiveDevice;

/// An illustration of a gamepad.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PadArt {
    DualSense,
    DualShock4,
    DualShock3,
    Xbox,
}

impl PadArt {
    /// The illustration for a gamepad of the kind `device` called `name`
    /// (if known): PlayStation pads by their generation; Xbox, Switch and
    /// any other, the Xbox controller.
    pub fn of(device: ActiveDevice, name: Option<&str>) -> Self {
        if device != ActiveDevice::PlayStation {
            return PadArt::Xbox;
        }
        let name = name.unwrap_or_default().to_ascii_lowercase();
        if name.contains("dualshock 3") || name.contains("playstation(r)3") || name.contains("ps3")
        {
            PadArt::DualShock3
        } else if name.contains("dualshock 4")
            || name.contains("ps4")
            || name == "wireless controller"
        {
            PadArt::DualShock4
        } else {
            PadArt::DualSense
        }
    }

    fn svg(self) -> &'static str {
        match self {
            PadArt::DualSense => include_str!("../assets/ui/pads/dualsense.svg"),
            PadArt::DualShock4 => include_str!("../assets/ui/pads/dualshock4.svg"),
            PadArt::DualShock3 => include_str!("../assets/ui/pads/dualshock3.svg"),
            PadArt::Xbox => include_str!("../assets/ui/pads/xbox.svg"),
        }
    }
}

/// The button of Bevy an illustration's `data-button` (or `data-trigger`)
/// is, if the game uses it.
fn button_of(name: &str) -> Option<GamepadButton> {
    use GamepadButton as B;
    Some(match name {
        "south" => B::South,
        "east" => B::East,
        "west" => B::West,
        "north" => B::North,
        "left_shoulder" => B::LeftTrigger,
        "right_shoulder" => B::RightTrigger,
        "left_trigger" => B::LeftTrigger2,
        "right_trigger" => B::RightTrigger2,
        "select" => B::Select,
        "start" => B::Start,
        "home" => B::Mode,
        "left_stick" => B::LeftThumb,
        "right_stick" => B::RightThumb,
        "dpad_up" => B::DPadUp,
        "dpad_down" => B::DPadDown,
        "dpad_left" => B::DPadLeft,
        "dpad_right" => B::DPadRight,
        _ => return None,
    })
}

/// Controls of the illustrations no pad of the game has: the back paddles
/// and the extra buttons of the Xbox controller.
const LEFT_OUT: [&str; 6] = [
    "left_paddle_1",
    "left_paddle_2",
    "right_paddle_1",
    "right_paddle_2",
    "extra_0",
    "extra_1",
];

// The illustrations as a tree. They are the game's own files, so a small
// reader of the XML they are written in is enough.

/// An element of an illustration.
#[derive(Clone, Debug, Default)]
struct Element {
    tag: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
    /// The text inside it (of `<text>`).
    text: String,
}

impl Element {
    fn get(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn number(&self, name: &str) -> Option<f32> {
        self.get(name).and_then(number)
    }

    fn has_class(&self, class: &str) -> bool {
        self.get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|c| c == class))
    }

    /// It and every element under it.
    fn descendants(&self) -> Vec<&Element> {
        let mut all = vec![self];
        for child in &self.children {
            all.extend(child.descendants());
        }
        all
    }
}

/// A number, or a percentage as a fraction.
fn number(value: &str) -> Option<f32> {
    match value.trim().strip_suffix('%') {
        Some(percent) => percent.parse::<f32>().ok().map(|p| p / 100.0),
        None => value.trim().parse().ok(),
    }
}

/// Reads an illustration: its root element.
fn parse(source: &str) -> Element {
    let mut stack = vec![Element::default()];
    let mut rest = source;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
        } else if let Some(after) = rest.strip_prefix("</") {
            rest = after.find('>').map_or("", |end| &after[end + 1..]);
            if stack.len() > 1 {
                let done = stack.pop().expect("an element is open");
                stack.last_mut().expect("its parent").children.push(done);
            }
        } else if let Some(after) = rest.strip_prefix('<') {
            // The end of the tag, past any quoted value.
            let mut quoted = false;
            let end = after
                .char_indices()
                .find(|&(_, c)| {
                    if c == '"' {
                        quoted = !quoted;
                    }
                    c == '>' && !quoted
                })
                .map_or(after.len(), |(at, _)| at);
            let tag = &after[..end];
            rest = after.get(end + 1..).unwrap_or("");
            if tag.starts_with('?') || tag.starts_with('!') {
                continue;
            }
            let closed = tag.ends_with('/');
            let element = element(tag.trim_end_matches('/'));
            if closed {
                stack.last_mut().expect("a parent").children.push(element);
            } else {
                stack.push(element);
            }
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            stack
                .last_mut()
                .expect("an element")
                .text
                .push_str(&rest[..end]);
            rest = &rest[end..];
        }
    }
    while stack.len() > 1 {
        let done = stack.pop().expect("an element is open");
        stack.last_mut().expect("its parent").children.push(done);
    }
    stack
        .pop()
        .and_then(|root| root.children.into_iter().next())
        .unwrap_or_default()
}

/// An element from the inside of its tag: its name and attributes.
fn element(tag: &str) -> Element {
    let tag = tag.trim();
    let name_end = tag.find(char::is_whitespace).unwrap_or(tag.len());
    let mut attributes = Vec::new();
    let mut rest = &tag[name_end..];
    while let Some(equals) = rest.find('=') {
        let name = rest[..equals].trim().to_string();
        let after = rest[equals + 1..].trim_start();
        let Some(value) = after.strip_prefix('"') else {
            break;
        };
        let Some(close) = value.find('"') else {
            break;
        };
        attributes.push((name, value[..close].replace("&amp;", "&")));
        rest = &value[close + 1..];
    }
    Element {
        tag: tag[..name_end].to_string(),
        attributes,
        ..default()
    }
}

// Geometry.

/// An affine transform as SVG writes it, `matrix(a b c d e f)`: a point
/// (x, y) goes to (a x + c y + e, b x + d y + f), as in zeno.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Affine([f32; 6]);

impl Affine {
    const IDENTITY: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// This, then `then`.
    fn then(self, then: Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [p, q, r, s, t, u] = then.0;
        Affine([
            p * a + r * b,
            q * a + s * b,
            p * c + r * d,
            q * c + s * d,
            p * e + r * f + t,
            q * e + s * f + u,
        ])
    }

    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.0;
        (a * x + c * y + e, b * x + d * y + f)
    }

    /// How much it scales lengths.
    fn scale(self) -> f32 {
        let [a, b, c, d, ..] = self.0;
        (a * d - b * c).abs().sqrt()
    }

    fn zeno(self) -> zeno::Transform {
        let [a, b, c, d, e, f] = self.0;
        zeno::Transform::new(a, b, c, d, e, f)
    }
}

/// A `transform` attribute: translations, rotations, scales and matrices,
/// the rightmost applied first.
fn transform(value: &str) -> Affine {
    let mut total = Affine::IDENTITY;
    let mut rest = value;
    while let Some(open) = rest.find('(') {
        let name = rest[..open].trim().trim_start_matches(',').trim();
        let Some(close) = rest[open..].find(')') else {
            break;
        };
        let args: Vec<f32> = rest[open + 1..open + close]
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter_map(|n| n.parse().ok())
            .collect();
        rest = &rest[open + close + 1..];
        let arg = |i: usize, or: f32| args.get(i).copied().unwrap_or(or);
        let one = match name {
            "translate" => Affine([1.0, 0.0, 0.0, 1.0, arg(0, 0.0), arg(1, 0.0)]),
            "scale" => Affine([arg(0, 1.0), 0.0, 0.0, arg(1, arg(0, 1.0)), 0.0, 0.0]),
            "rotate" => {
                let (sin, cos) = arg(0, 0.0).to_radians().sin_cos();
                let (x, y) = (arg(1, 0.0), arg(2, 0.0));
                Affine([
                    cos,
                    sin,
                    -sin,
                    cos,
                    x - cos * x + sin * y,
                    y - sin * x - cos * y,
                ])
            }
            "matrix" if args.len() == 6 => {
                Affine([args[0], args[1], args[2], args[3], args[4], args[5]])
            }
            _ => Affine::IDENTITY,
        };
        total = one.then(total);
    }
    total
}

/// The id a `url(#id)` (or a `#id`) points to.
fn link(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("url(")
        .trim_end_matches(')')
        .trim_start_matches('#')
}

/// The outline of a shape element, as SVG path data.
fn outline(element: &Element, ids: &HashMap<String, &Element>) -> Option<String> {
    let n = |name: &str| element.number(name).unwrap_or(0.0);
    match element.tag.as_str() {
        "path" => element.get("d").map(str::to_string),
        "circle" => {
            let (cx, cy, r) = (n("cx"), n("cy"), n("r"));
            Some(format!(
                "M{} {cy} A{r} {r} 0 1 0 {} {cy} A{r} {r} 0 1 0 {} {cy} Z",
                cx - r,
                cx + r,
                cx - r
            ))
        }
        "rect" => {
            let (x, y, w, h) = (n("x"), n("y"), n("width"), n("height"));
            let rx = element
                .number("rx")
                .or(element.number("ry"))
                .unwrap_or(0.0)
                .min(w / 2.0);
            let ry = element.number("ry").unwrap_or(rx).min(h / 2.0);
            if rx <= 0.0 || ry <= 0.0 {
                return Some(format!("M{x} {y} H{} V{} H{x} Z", x + w, y + h));
            }
            let (right, bottom) = (x + w, y + h);
            Some(format!(
                "M{} {y} H{} A{rx} {ry} 0 0 1 {right} {} V{} A{rx} {ry} 0 0 1 {} {bottom} H{} \
                 A{rx} {ry} 0 0 1 {x} {} V{} A{rx} {ry} 0 0 1 {} {y} Z",
                x + rx,
                right - rx,
                y + ry,
                bottom - ry,
                right - rx,
                x + rx,
                bottom - ry,
                y + ry,
                x + rx
            ))
        }
        "use" => {
            let id = element.get("href").or(element.get("xlink:href"))?;
            outline(ids.get(link(id))?, ids)
        }
        _ => None,
    }
}

// Paint.

/// A colour as straight RGBA, 0 to 1.
type Rgba = [f32; 4];

fn color(value: &str) -> Option<Rgba> {
    let hex = value.trim().strip_prefix('#')?;
    let digit = |i: usize, len: usize| u8::from_str_radix(hex.get(i..i + len)?, 16).ok();
    let (r, g, b) = match hex.len() {
        6 => (digit(0, 2)?, digit(2, 2)?, digit(4, 2)?),
        3 => (digit(0, 1)? * 17, digit(1, 1)? * 17, digit(2, 1)? * 17),
        _ => return None,
    };
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
}

/// How a gradient spreads over the box of the shape it fills (in fractions
/// of it), and its stops.
#[derive(Clone, Debug)]
enum Gradient {
    Linear([f32; 4], Vec<(f32, Rgba)>),
    Radial([f32; 3], Vec<(f32, Rgba)>),
}

impl Gradient {
    fn read(element: &Element) -> Option<Gradient> {
        let stops: Vec<(f32, Rgba)> = element
            .children
            .iter()
            .filter(|stop| stop.tag == "stop")
            .map(|stop| {
                let mut rgba = stop.get("stop-color").and_then(color).unwrap_or([0.0; 4]);
                rgba[3] *= stop.number("stop-opacity").unwrap_or(1.0);
                (stop.number("offset").unwrap_or(0.0), rgba)
            })
            .collect();
        let n = |name: &str, or: f32| element.number(name).unwrap_or(or);
        match element.tag.as_str() {
            "linearGradient" => Some(Gradient::Linear(
                [n("x1", 0.0), n("y1", 0.0), n("x2", 1.0), n("y2", 0.0)],
                stops,
            )),
            "radialGradient" => Some(Gradient::Radial(
                [n("cx", 0.5), n("cy", 0.5), n("r", 0.5)],
                stops,
            )),
            _ => None,
        }
    }

    /// The colour at (u, v), as fractions of the box.
    fn at(&self, u: f32, v: f32) -> Rgba {
        let (t, stops) = match self {
            Gradient::Linear([x1, y1, x2, y2], stops) => {
                let (dx, dy) = (x2 - x1, y2 - y1);
                let length = (dx * dx + dy * dy).max(1e-6);
                (((u - x1) * dx + (v - y1) * dy) / length, stops)
            }
            Gradient::Radial([cx, cy, r], stops) => {
                let (du, dv) = ((u - cx) / r.max(1e-6), (v - cy) / r.max(1e-6));
                ((du * du + dv * dv).sqrt(), stops)
            }
        };
        let Some(first) = stops.first() else {
            return [0.0; 4];
        };
        if t <= first.0 {
            return first.1;
        }
        for pair in stops.windows(2) {
            let ((a, from), (b, to)) = (pair[0], pair[1]);
            if t <= b {
                let k = if b > a { (t - a) / (b - a) } else { 1.0 };
                return std::array::from_fn(|i| from[i] + (to[i] - from[i]) * k);
            }
        }
        stops.last().map_or(first.1, |last| last.1)
    }
}

/// What a shape is filled or stroked with.
#[derive(Clone, Debug)]
enum Paint {
    None,
    Color(Rgba),
    Gradient(Gradient),
}

/// A filter of the illustrations: a drop shadow (offset and blur, in the
/// illustration's units, and its colour), or a glow (the element blurred
/// under itself).
#[derive(Clone, Copy, Debug)]
enum Filter {
    Shadow {
        dx: f32,
        dy: f32,
        blur: f32,
        color: Rgba,
    },
    Glow {
        blur: f32,
    },
}

impl Filter {
    fn read(element: &Element) -> Option<Filter> {
        let parts = element.descendants();
        if let Some(shadow) = parts.iter().find(|p| p.tag == "feDropShadow") {
            let mut color = shadow
                .get("flood-color")
                .and_then(color)
                .unwrap_or([0.0; 4]);
            color[3] *= shadow.number("flood-opacity").unwrap_or(1.0);
            return Some(Filter::Shadow {
                dx: shadow.number("dx").unwrap_or(0.0),
                dy: shadow.number("dy").unwrap_or(0.0),
                blur: shadow.number("stdDeviation").unwrap_or(0.0),
                color,
            });
        }
        parts
            .iter()
            .find(|p| p.tag == "feGaussianBlur")
            .map(|blur| Filter::Glow {
                blur: blur.number("stdDeviation").unwrap_or(0.0),
            })
    }
}

/// What an element inherits from the groups it is in.
#[derive(Clone, Debug)]
struct Style {
    fill: String,
    stroke: String,
    stroke_width: f32,
    stroke_opacity: f32,
    cap: Cap,
    join: Join,
    font_size: f32,
    letter_spacing: f32,
    bold: bool,
    middle: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: "#000000".into(),
            stroke: "none".into(),
            stroke_width: 1.0,
            stroke_opacity: 1.0,
            cap: Cap::Butt,
            join: Join::Miter,
            font_size: 16.0,
            letter_spacing: 0.0,
            bold: false,
            middle: false,
        }
    }
}

impl Style {
    /// This, with what `element` says.
    fn with(&self, element: &Element) -> Style {
        let mut style = self.clone();
        for (name, value) in &element.attributes {
            match name.as_str() {
                "fill" => style.fill.clone_from(value),
                "stroke" => style.stroke.clone_from(value),
                "stroke-width" => style.stroke_width = number(value).unwrap_or(1.0),
                "stroke-opacity" => style.stroke_opacity = number(value).unwrap_or(1.0),
                "stroke-linecap" => {
                    style.cap = match value.as_str() {
                        "round" => Cap::Round,
                        "square" => Cap::Square,
                        _ => Cap::Butt,
                    }
                }
                "stroke-linejoin" => {
                    style.join = match value.as_str() {
                        "round" => Join::Round,
                        "bevel" => Join::Bevel,
                        _ => Join::Miter,
                    }
                }
                "font-size" => style.font_size = number(value).unwrap_or(16.0),
                "letter-spacing" => style.letter_spacing = number(value).unwrap_or(0.0),
                "font-weight" => {
                    style.bold = value == "bold" || number(value).unwrap_or(400.0) >= 600.0;
                }
                "text-anchor" => style.middle = value == "middle",
                _ => {}
            }
        }
        style
    }
}

// The canvas.

/// Premultiplied RGBA pixels, with the box where anything was drawn (left,
/// top, right, bottom).
struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
    drawn: Option<[i32; 4]>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0.0; 4]; (width * height) as usize],
            drawn: None,
        }
    }

    /// `area` (left, top, right, bottom), inside the canvas.
    fn clamp(&self, [x0, y0, x1, y1]: [i32; 4]) -> [i32; 4] {
        [
            x0.max(0),
            y0.max(0),
            x1.min(self.width as i32),
            y1.min(self.height as i32),
        ]
    }

    fn grow(&mut self, area: [i32; 4]) {
        let area = self.clamp(area);
        if area[2] <= area[0] || area[3] <= area[1] {
            return;
        }
        self.drawn = Some(match self.drawn {
            Some([x0, y0, x1, y1]) => [
                x0.min(area[0]),
                y0.min(area[1]),
                x1.max(area[2]),
                y1.max(area[3]),
            ],
            None => area,
        });
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y as u32 * self.width + x as u32) as usize
    }

    /// Lays the colour `paint` gives at each pixel (straight) over the
    /// canvas, where `coverage` covers it.
    fn cover(
        &mut self,
        coverage: &[u8],
        place: Placement,
        mut paint: impl FnMut(i32, i32) -> Rgba,
    ) {
        let area = [
            place.left,
            place.top,
            place.left + place.width as i32,
            place.top + place.height as i32,
        ];
        let [x0, y0, x1, y1] = self.clamp(area);
        for y in y0..y1 {
            for x in x0..x1 {
                let at = ((y - place.top) as u32 * place.width + (x - place.left) as u32) as usize;
                let cover = coverage[at] as f32 / 255.0;
                if cover <= 0.0 {
                    continue;
                }
                let [r, g, b, a] = paint(x, y);
                let alpha = a * cover;
                let index = self.index(x, y);
                let pixel = &mut self.pixels[index];
                let keep = 1.0 - alpha;
                *pixel = [
                    r * alpha + pixel[0] * keep,
                    g * alpha + pixel[1] * keep,
                    b * alpha + pixel[2] * keep,
                    alpha + pixel[3] * keep,
                ];
            }
        }
        self.grow(area);
    }

    /// Lays `layer` over this canvas, each pixel premultiplied as it is.
    fn over(&mut self, layer: &Canvas) {
        let Some(area) = layer.drawn else {
            return;
        };
        let [x0, y0, x1, y1] = area;
        for y in y0..y1 {
            for x in x0..x1 {
                let index = self.index(x, y);
                let top = layer.pixels[index];
                if top[3] <= 0.0 {
                    continue;
                }
                let pixel = &mut self.pixels[index];
                let keep = 1.0 - top[3];
                *pixel = std::array::from_fn(|i| top[i] + pixel[i] * keep);
            }
        }
        self.grow(area);
    }

    /// Keeps only what `coverage` covers.
    fn clip(&mut self, coverage: &[u8], place: Placement) {
        let Some([x0, y0, x1, y1]) = self.drawn else {
            return;
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let (u, v) = (x - place.left, y - place.top);
                let inside = u >= 0 && v >= 0 && u < place.width as i32 && v < place.height as i32;
                let cover = if inside {
                    coverage[(v as u32 * place.width + u as u32) as usize] as f32 / 255.0
                } else {
                    0.0
                };
                let index = self.index(x, y);
                let pixel = &mut self.pixels[index];
                *pixel = pixel.map(|c| c * cover);
            }
        }
    }

    /// The canvas blurred (as a Gaussian of deviation `sigma` px, by three
    /// box blurs), over its drawn box widened for it.
    fn blurred(&self, sigma: f32) -> Canvas {
        let mut out = Canvas {
            width: self.width,
            height: self.height,
            pixels: self.pixels.clone(),
            drawn: None,
        };
        let Some([x0, y0, x1, y1]) = self.drawn else {
            return out;
        };
        let reach = (sigma * 3.0).ceil() as i32;
        let area = self.clamp([x0 - reach, y0 - reach, x1 + reach, y1 + reach]);
        out.drawn = Some(area);
        let radius = (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) / 2.0).round() as i32;
        if radius < 1 {
            return out;
        }
        let [ax0, ay0, ax1, ay1] = area;
        let width = self.width as i32;
        let span = (2 * radius + 1) as f32;
        let mut line: Vec<Rgba> = Vec::new();
        for _ in 0..3 {
            for horizontal in [true, false] {
                let (outer, inner) = if horizontal {
                    (ay0..ay1, ax0..ax1)
                } else {
                    (ax0..ax1, ay0..ay1)
                };
                for o in outer {
                    let index = |i: i32| {
                        if horizontal {
                            (o * width + i) as usize
                        } else {
                            (i * width + o) as usize
                        }
                    };
                    line.clear();
                    line.extend(inner.clone().map(|i| out.pixels[index(i)]));
                    let n = line.len() as i32;
                    let mut sum = [0.0f32; 4];
                    for i in -radius..=radius {
                        let p = line[i.clamp(0, n - 1) as usize];
                        sum = std::array::from_fn(|c| sum[c] + p[c]);
                    }
                    for (i, at) in inner.clone().enumerate() {
                        let i = i as i32;
                        out.pixels[index(at)] = sum.map(|c| c / span);
                        let add = line[(i + radius + 1).min(n - 1) as usize];
                        let drop = line[(i - radius).max(0) as usize];
                        sum = std::array::from_fn(|c| sum[c] + add[c] - drop[c]);
                    }
                }
            }
        }
        out
    }

    /// The shadow `self` casts: its alpha, moved by (`dx`, `dy`) px, in
    /// `color`.
    fn shadow(&self, dx: i32, dy: i32, color: Rgba) -> Canvas {
        let mut shadow = Canvas::new(self.width, self.height);
        let Some([x0, y0, x1, y1]) = self.drawn else {
            return shadow;
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let (tx, ty) = (x + dx, y + dy);
                if tx < 0 || ty < 0 || tx >= self.width as i32 || ty >= self.height as i32 {
                    continue;
                }
                let a = self.pixels[self.index(x, y)][3] * color[3];
                let index = shadow.index(tx, ty);
                shadow.pixels[index] = [color[0] * a, color[1] * a, color[2] * a, a];
            }
        }
        shadow.grow([x0 + dx, y0 + dy, x1 + dx, y1 + dy]);
        shadow
    }
}

/// What to draw of an illustration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Part<'a> {
    /// The whole controller.
    Whole,
    /// A button's shapes, or its marks, in white, with no shadows.
    Shapes(&'a str),
    Marks(&'a str),
}

/// Draws illustrations.
struct Painter<'a> {
    ids: HashMap<String, &'a Element>,
    part: Part<'a>,
    /// The letters printed on the face buttons are left out.
    unlabelled: bool,
    font: FontRef<'static>,
}

impl Painter<'_> {
    fn paint(&self, value: &str) -> Paint {
        if value == "none" {
            return Paint::None;
        }
        if value.starts_with("url(") {
            let Some(found) = self.ids.get(link(value)) else {
                return Paint::None;
            };
            if found.tag == "pattern" {
                // A dotted pattern: as its dots cover it on average.
                let (w, h) = (
                    found.number("width").unwrap_or(1.0),
                    found.number("height").unwrap_or(1.0),
                );
                let dot = found.children.iter().find(|c| c.tag == "circle");
                let mut rgba = dot
                    .and_then(|d| d.get("fill"))
                    .and_then(color)
                    .unwrap_or([0.0; 4]);
                let r = dot.and_then(|d| d.number("r")).unwrap_or(0.0);
                rgba[3] *= (std::f32::consts::PI * r * r / (w * h)).min(1.0);
                return Paint::Color(rgba);
            }
            return Gradient::read(found).map_or(Paint::None, Paint::Gradient);
        }
        color(value).map_or(Paint::None, Paint::Color)
    }

    /// Draws `element` and what is under it on `canvas`. `inside`: within
    /// the part drawn (the button drawn on its own).
    fn draw(
        &self,
        canvas: &mut Canvas,
        element: &Element,
        style: &Style,
        matrix: Affine,
        inside: bool,
    ) {
        if matches!(
            element.tag.as_str(),
            "defs" | "clipPath" | "pattern" | "title" | "filter"
        ) {
            return;
        }
        // Left out: the controls no pad of the game has, the letters that
        // are not this pad's, and a button's marks when its shapes are
        // drawn.
        if element
            .get("data-button")
            .is_some_and(|b| LEFT_OUT.contains(&b))
            || (self.unlabelled && element.get("data-label").is_some())
            || (inside && matches!(self.part, Part::Shapes(_)) && element.has_class("mark"))
        {
            return;
        }
        let inside = inside
            || match self.part {
                Part::Whole => true,
                Part::Shapes(button) | Part::Marks(button) => {
                    element.get("data-button") == Some(button)
                }
            };
        let style = style.with(element);
        let matrix = element
            .get("transform")
            .map_or(matrix, |t| transform(t).then(matrix));
        let filter = element
            .get("filter")
            .filter(|_| self.part == Part::Whole)
            .and_then(|f| self.ids.get(link(f)))
            .and_then(|f| Filter::read(f));
        let clip = element.get("clip-path").and_then(|c| self.ids.get(link(c)));
        if filter.is_none() && clip.is_none() {
            self.draw_content(canvas, element, &style, matrix, inside);
            return;
        }
        // Drawn apart, clipped, and laid with its filter.
        let mut layer = Canvas::new(canvas.width, canvas.height);
        self.draw_content(&mut layer, element, &style, matrix, inside);
        if let Some(clip) = clip {
            let path: Vec<String> = clip
                .children
                .iter()
                .filter_map(|shape| outline(shape, &self.ids))
                .collect();
            let path = path.join(" ");
            let (coverage, place) = Mask::new(path.as_str())
                .style(Fill::NonZero)
                .transform(Some(matrix.zeno()))
                .render();
            layer.clip(&coverage, place);
        }
        let scale = matrix.scale();
        match filter {
            Some(Filter::Shadow {
                dx,
                dy,
                blur,
                color,
            }) => {
                let (dx, dy) = ((dx * scale).round() as i32, (dy * scale).round() as i32);
                canvas.over(&layer.blurred(blur * scale).shadow(dx, dy, color));
            }
            Some(Filter::Glow { blur }) => canvas.over(&layer.blurred(blur * scale)),
            None => {}
        }
        canvas.over(&layer);
    }

    fn draw_content(
        &self,
        canvas: &mut Canvas,
        element: &Element,
        style: &Style,
        matrix: Affine,
        inside: bool,
    ) {
        for child in &element.children {
            self.draw(canvas, child, style, matrix, inside);
        }
        if !inside || (matches!(self.part, Part::Marks(_)) && !element.has_class("mark")) {
            return;
        }
        if element.tag == "text" {
            self.draw_text(canvas, element, style, matrix);
            return;
        }
        let Some(path) = outline(element, &self.ids) else {
            return;
        };
        // A button on its own is white all over.
        let white = self.part != Part::Whole;
        let paint = |value: &str| {
            if value == "none" {
                Paint::None
            } else if white {
                Paint::Color([1.0; 4])
            } else {
                self.paint(value)
            }
        };
        // The fill, then the stroke.
        let fill = paint(&style.fill);
        if !matches!(fill, Paint::None) {
            let (coverage, place) = Mask::new(path.as_str())
                .style(Fill::NonZero)
                .transform(Some(matrix.zeno()))
                .render();
            lay(canvas, &coverage, place, &fill, 1.0);
        }
        let stroke = paint(&style.stroke);
        if !matches!(stroke, Paint::None) && style.stroke_width > 0.0 {
            let mut pen = Stroke::new(style.stroke_width);
            pen.cap(style.cap).join(style.join).scale(true);
            let (coverage, place) = Mask::new(path.as_str())
                .style(pen)
                .transform(Some(matrix.zeno()))
                .render();
            let opacity = if white { 1.0 } else { style.stroke_opacity };
            lay(canvas, &coverage, place, &stroke, opacity);
        }
    }

    /// Words, in the game's font, filled as the style says.
    fn draw_text(&self, canvas: &mut Canvas, element: &Element, style: &Style, matrix: Affine) {
        let Paint::Color(rgba) = self.paint(&style.fill) else {
            return;
        };
        let rgba = if self.part == Part::Whole {
            rgba
        } else {
            [1.0; 4]
        };
        let scale = matrix.scale();
        let size = style.font_size * scale;
        let spacing = style.letter_spacing * scale;
        let mut shaping = ShapeContext::new();
        let mut shaper = shaping.builder(self.font).size(size).build();
        shaper.add_str(element.text.trim());
        let mut glyphs = Vec::new();
        let mut pen = 0.0;
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                glyphs.push((glyph.id, pen + glyph.x));
                pen += glyph.advance + spacing;
            }
        });
        let width = (pen - spacing).max(0.0);
        let (x, y) = matrix.apply(
            element.number("x").unwrap_or(0.0),
            element.number("y").unwrap_or(0.0),
        );
        let start = if style.middle { x - width / 2.0 } else { x };
        let mut scaling = ScaleContext::new();
        let mut scaler = scaling.builder(self.font).size(size).hint(false).build();
        let mut render = Render::new(&[Source::Outline]);
        render.format(Format::Alpha);
        if style.bold {
            render.embolden(size * 0.03);
        }
        for (id, at) in glyphs {
            let Some(image) = render.render(&mut scaler, id) else {
                continue;
            };
            let place = Placement {
                left: (start + at).round() as i32 + image.placement.left,
                top: y.round() as i32 - image.placement.top,
                ..image.placement
            };
            canvas.cover(&image.data, place, |_, _| rgba);
        }
    }
}

/// Lays `paint` where `coverage` covers, at `opacity`: a gradient spread
/// over the box covered.
fn lay(canvas: &mut Canvas, coverage: &[u8], place: Placement, paint: &Paint, opacity: f32) {
    match paint {
        Paint::None => {}
        Paint::Color([r, g, b, a]) => {
            canvas.cover(coverage, place, |_, _| [*r, *g, *b, a * opacity]);
        }
        Paint::Gradient(gradient) => {
            let (w, h) = (place.width.max(1) as f32, place.height.max(1) as f32);
            canvas.cover(coverage, place, |x, y| {
                let u = (x - place.left) as f32 / w;
                let v = (y - place.top) as f32 / h;
                let mut rgba = gradient.at(u, v);
                rgba[3] *= opacity;
                rgba
            });
        }
    }
}

/// An illustration read: its tree and viewBox (left, top, width, height).
struct Sheet {
    root: Element,
    view: [f32; 4],
}

impl Sheet {
    fn read(art: PadArt) -> Sheet {
        let root = parse(art.svg());
        let view: Vec<f32> = root
            .get("viewBox")
            .unwrap_or("0 0 400 260")
            .split_whitespace()
            .filter_map(|n| n.parse().ok())
            .collect();
        Sheet {
            view: [view[0], view[1], view[2], view[3]],
            root,
        }
    }

    /// Every element with an id, by it.
    fn ids(&self) -> HashMap<String, &Element> {
        self.root
            .descendants()
            .into_iter()
            .filter_map(|e| e.get("id").map(|id| (id.to_string(), e)))
            .collect()
    }

    /// The buttons it draws that the game has, with their `data-button`.
    fn buttons(&self) -> Vec<(GamepadButton, &str)> {
        let mut found: Vec<(GamepadButton, &str)> = Vec::new();
        for element in self.root.descendants() {
            if let Some(name) = element.get("data-button")
                && let Some(button) = button_of(name)
                && !found.iter().any(|(b, _)| *b == button)
            {
                found.push((button, name));
            }
        }
        found
    }

    /// Where each trigger goes, in the illustration's units.
    fn triggers(&self) -> Vec<(GamepadButton, [f32; 4])> {
        self.root
            .descendants()
            .into_iter()
            .filter_map(|e| {
                let button = button_of(e.get("data-trigger")?)?;
                Some((
                    button,
                    [
                        e.number("data-x")?,
                        e.number("data-y")?,
                        e.number("data-width")?,
                        e.number("data-height")?,
                    ],
                ))
            })
            .collect()
    }

    /// `part` of it on a canvas `width` × `height` px.
    fn draw(&self, part: Part, unlabelled: bool, width: u32, height: u32) -> Canvas {
        let [left, top, _, view_height] = self.view;
        let scale = height as f32 / view_height;
        let screen = Affine([scale, 0.0, 0.0, scale, -left * scale, -top * scale]);
        let painter = Painter {
            ids: self.ids(),
            part,
            unlabelled,
            font: FontRef::from_index(crate::vector_art::FONT, 0).expect("the UI font is valid"),
        };
        let mut canvas = Canvas::new(width, height);
        painter.draw(&mut canvas, &self.root, &Style::default(), screen, false);
        canvas
    }
}

/// A canvas as straight RGBA bytes.
fn bytes(canvas: &Canvas) -> Vec<u8> {
    canvas
        .pixels
        .iter()
        .flat_map(|&[r, g, b, a]| {
            let straight = |c: f32| {
                if a > 0.0 {
                    (c / a * 255.0).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                }
            };
            [
                straight(r),
                straight(g),
                straight(b),
                (a * 255.0).round().clamp(0.0, 255.0) as u8,
            ]
        })
        .collect()
}

/// A button lit on its own, from its shapes and its marks drawn apart: the
/// shapes white, the marks dark on them, cropped to where it is drawn
/// (left, top, width and height, px). `None` if nothing is drawn.
fn lit(shapes: &Canvas, marks: &Canvas) -> Option<([u32; 4], Vec<u8>)> {
    let mut area: Option<[i32; 4]> = None;
    for [x0, y0, x1, y1] in [shapes.drawn, marks.drawn].into_iter().flatten() {
        area = Some(match area {
            Some([a, b, c, d]) => [a.min(x0), b.min(y0), c.max(x1), d.max(y1)],
            None => [x0, y0, x1, y1],
        });
    }
    let [x0, y0, x1, y1] = area?;
    const INK: f32 = 40.0;
    let mut data = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = shapes.index(x, y);
            let shape = shapes.pixels[index][3];
            let mark = marks.pixels[index][3];
            let gray = (255.0 + (INK - 255.0) * mark).round() as u8;
            let alpha = (shape.max(mark) * 255.0).round() as u8;
            data.extend_from_slice(&[gray, gray, gray, alpha]);
        }
    }
    Some((
        [x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32],
        data,
    ))
}

/// An image of `width` × `height` pixels of straight RGBA.
fn image(width: u32, height: u32, data: Vec<u8>) -> Image {
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

/// A gamepad drawn for the screen.
#[derive(Clone, Debug)]
pub struct DrawnPad {
    /// Its size on the screen (logical px).
    pub size: Vec2,
    pub whole: Handle<Image>,
    /// Each button, white with its marks dark, and where it goes (logical
    /// px from the top left of the controller).
    pub buttons: Vec<(GamepadButton, Handle<Image>, Rect)>,
    /// Where each trigger goes (logical px).
    pub triggers: Vec<(GamepadButton, Rect)>,
}

/// The gamepads drawn so far, by illustration, whether their face buttons'
/// letters were left out, and height in pixels.
#[derive(Resource, Default)]
pub struct PadArts {
    drawn: HashMap<(PadArt, bool, u32), DrawnPad>,
}

impl PadArts {
    /// `art` drawn `height` logical px tall at `scale_factor` pixels each,
    /// once.
    pub fn get(
        &mut self,
        art: PadArt,
        unlabelled: bool,
        height: f32,
        scale_factor: f32,
        images: &mut Assets<Image>,
    ) -> &DrawnPad {
        let pixels = (height * scale_factor).round().max(1.0) as u32;
        self.drawn
            .entry((art, unlabelled, pixels))
            .or_insert_with(|| draw(art, unlabelled, pixels, scale_factor, images))
    }
}

/// Draws `art` `pixels` tall.
fn draw(
    art: PadArt,
    unlabelled: bool,
    pixels: u32,
    scale_factor: f32,
    images: &mut Assets<Image>,
) -> DrawnPad {
    let sheet = Sheet::read(art);
    let [left, top, view_width, view_height] = sheet.view;
    let pixels_per_unit = pixels as f32 / view_height;
    let width = (view_width * pixels_per_unit).round() as u32;
    let whole = sheet.draw(Part::Whole, unlabelled, width, pixels);
    let whole = images.add(image(width, pixels, bytes(&whole)));
    let logical = |px: f32| px / scale_factor;
    let buttons = sheet
        .buttons()
        .into_iter()
        .filter_map(|(button, name)| {
            let shapes = sheet.draw(Part::Shapes(name), unlabelled, width, pixels);
            let marks = sheet.draw(Part::Marks(name), unlabelled, width, pixels);
            let ([x, y, w, h], data) = lit(&shapes, &marks)?;
            let rect = Rect::new(
                logical(x as f32),
                logical(y as f32),
                logical((x + w) as f32),
                logical((y + h) as f32),
            );
            Some((button, images.add(image(w, h, data)), rect))
        })
        .collect();
    let units = |value: f32| logical(value * pixels_per_unit);
    let triggers = sheet
        .triggers()
        .into_iter()
        .map(|(button, [x, y, w, h])| {
            let (x, y) = (units(x - left), units(y - top));
            (button, Rect::new(x, y, x + units(w), y + units(h)))
        })
        .collect();
    DrawnPad {
        size: Vec2::new(logical(width as f32), logical(pixels as f32)),
        whole,
        buttons,
        triggers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTS: [PadArt; 4] = [
        PadArt::DualSense,
        PadArt::DualShock4,
        PadArt::DualShock3,
        PadArt::Xbox,
    ];

    #[test]
    fn each_pad_gets_its_illustration() {
        let ps = ActiveDevice::PlayStation;
        assert_eq!(
            PadArt::of(ps, Some("DualSense Wireless Controller")),
            PadArt::DualSense
        );
        assert_eq!(
            PadArt::of(ps, Some("Wireless Controller")),
            PadArt::DualShock4
        );
        assert_eq!(PadArt::of(ps, Some("PS4 Controller")), PadArt::DualShock4);
        assert_eq!(
            PadArt::of(ps, Some("Sony PLAYSTATION(R)3 Controller")),
            PadArt::DualShock3
        );
        assert_eq!(PadArt::of(ps, None), PadArt::DualSense);
        assert_eq!(PadArt::of(ActiveDevice::Xbox, None), PadArt::Xbox);
        assert_eq!(
            PadArt::of(ActiveDevice::Switch, Some("Pro Controller")),
            PadArt::Xbox
        );
    }

    #[test]
    fn the_xml_of_the_illustrations_is_read() {
        let root = parse(
            r##"<svg viewBox="0 0 10 10"><!-- a <note> --><g fill="#fff"><circle r="2"/>
               <text x="1">HI</text></g></svg>"##,
        );
        assert_eq!(root.tag, "svg");
        assert_eq!(root.get("viewBox"), Some("0 0 10 10"));
        let group = &root.children[0];
        assert_eq!(group.get("fill"), Some("#fff"));
        assert_eq!(group.children[0].tag, "circle");
        assert_eq!(group.children[1].text, "HI");
    }

    #[test]
    fn transforms_compose_as_in_svg() {
        let (x, y) = transform("rotate(90 10 0)").apply(20.0, 0.0);
        assert!(
            (x - 10.0).abs() < 1e-4 && (y - 10.0).abs() < 1e-4,
            "{x} {y}"
        );
        // The rightmost first: scaled, then moved.
        let (x, y) = transform("translate(5 0) scale(2)").apply(1.0, 1.0);
        assert!((x - 7.0).abs() < 1e-4 && (y - 2.0).abs() < 1e-4, "{x} {y}");
        let (x, y) = transform("matrix(-1 0 0 1 714 0)").apply(14.0, 3.0);
        assert!((x - 700.0).abs() < 1e-4 && (y - 3.0).abs() < 1e-4);
    }

    #[test]
    fn gradients_run_from_stop_to_stop() {
        let gradient = Gradient::Linear(
            [0.0, 0.0, 0.0, 1.0],
            vec![(0.0, [1.0, 1.0, 1.0, 1.0]), (1.0, [0.0, 0.0, 0.0, 1.0])],
        );
        assert_eq!(gradient.at(0.3, 0.0)[0], 1.0);
        assert!((gradient.at(0.3, 0.5)[0] - 0.5).abs() < 1e-4);
        assert_eq!(gradient.at(0.3, 2.0)[0], 0.0);
    }

    #[test]
    fn every_illustration_has_the_buttons_of_the_game() {
        use GamepadButton as B;
        for art in ARTS {
            let sheet = Sheet::read(art);
            let buttons: Vec<GamepadButton> = sheet.buttons().iter().map(|(b, _)| *b).collect();
            for button in [
                B::South,
                B::East,
                B::West,
                B::North,
                B::LeftTrigger,
                B::RightTrigger,
                B::Select,
                B::Start,
                B::Mode,
                B::LeftThumb,
                B::RightThumb,
                B::DPadUp,
                B::DPadDown,
                B::DPadLeft,
                B::DPadRight,
            ] {
                assert!(buttons.contains(&button), "{art:?}: {button:?}");
            }
            let triggers: Vec<GamepadButton> = sheet.triggers().iter().map(|(b, _)| *b).collect();
            assert_eq!(triggers, [B::LeftTrigger2, B::RightTrigger2], "{art:?}");
        }
    }

    #[test]
    fn every_illustration_is_drawn_a_light_controller_on_its_shadow() {
        for art in ARTS {
            let sheet = Sheet::read(art);
            let height = 160;
            let width = (sheet.view[2] / sheet.view[3] * height as f32).round() as u32;
            let canvas = sheet.draw(Part::Whole, false, width, height);
            // The controller covers much of its box, and its body is light.
            let opaque = canvas.pixels.iter().filter(|p| p[3] > 0.9).count() as f32
                / canvas.pixels.len() as f32;
            assert!((0.25..0.9).contains(&opaque), "{art:?}: {opaque}");
            let lightest = canvas
                .pixels
                .iter()
                .filter(|p| p[3] > 0.99)
                .map(|p| p[0] + p[1] + p[2])
                .fold(0.0f32, f32::max);
            assert!(lightest > 2.6, "{art:?}: {lightest}");
            // Its shadow: faint, and below it more than above.
            let column = |y: u32| canvas.pixels[(y * width + width / 2) as usize][3];
            assert!(column(height - 1) < 0.6, "{art:?}");
        }
    }

    #[test]
    fn the_printed_letters_go_when_another_pad_marks_them_otherwise() {
        let sheet = Sheet::read(PadArt::Xbox);
        let (width, height) = (208, 160);
        let labelled = sheet.draw(Part::Marks("south"), false, width, height);
        let unlabelled = sheet.draw(Part::Marks("south"), true, width, height);
        assert!(labelled.drawn.is_some());
        assert!(unlabelled.drawn.is_none());
    }

    #[test]
    fn a_pad_is_drawn_with_each_button_on_it() {
        let mut images = Assets::<Image>::default();
        let mut arts = PadArts::default();
        let drawn = arts
            .get(PadArt::DualSense, false, 120.0, 1.0, &mut images)
            .clone();
        assert!((drawn.size.y - 120.0).abs() < 0.5);
        assert!(drawn.size.x > drawn.size.y);
        assert_eq!(drawn.buttons.len(), 15);
        let pad = Rect::new(0.0, 0.0, drawn.size.x, drawn.size.y);
        for (button, _, rect) in &drawn.buttons {
            assert!(rect.width() > 1.0 && rect.height() > 1.0, "{button:?}");
            assert!(
                pad.contains(rect.min) && pad.contains(rect.max),
                "{button:?}: {rect:?}"
            );
        }
        let at = |button| {
            drawn
                .buttons
                .iter()
                .find(|(b, ..)| *b == button)
                .map(|(_, _, rect)| rect.center())
                .unwrap()
        };
        // The face buttons on the right, the D-pad on the left.
        assert!(at(GamepadButton::South).x > at(GamepadButton::DPadDown).x);
        assert!(at(GamepadButton::North).y < at(GamepadButton::South).y);
        assert!(at(GamepadButton::DPadUp).y < at(GamepadButton::DPadDown).y);
        // A button is white where it is, its mark dark on it.
        let (_, handle, _) = drawn
            .buttons
            .iter()
            .find(|(b, ..)| *b == GamepadButton::South)
            .unwrap();
        let data = images.get(handle).unwrap().data.clone().unwrap();
        assert!(data.chunks(4).any(|p| p[3] == 255 && p[0] == 255));
        assert!(data.chunks(4).any(|p| p[3] > 200 && p[0] < 120));
        // Drawn once for that size.
        let again = arts.get(PadArt::DualSense, false, 120.0, 1.0, &mut images);
        assert_eq!(again.whole, drawn.whole);
    }
}
