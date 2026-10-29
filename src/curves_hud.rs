//! The panel of the keyboard's curves, under the performance HUD at the top
//! right, when Options > Graphics > Performance HUD says so: each of the five
//! curves (see [`crate::bike::curves`]) as it is set (in amber when it is
//! the learnt one), over the classic curve in grey, with its control
//! points, and a dot where the first player's bike is on it now. The curve being edited in the options stands
//! out in the colour of Control, the panel drawn over them.
//!
//! Each curve is drawn once into an image of its own, again only when the
//! curves change; the dots are interface nodes moved every frame.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::bike::curves::{
    CURVE_KINDS, CurveKind, CurveProbe, CurveShape, KeyboardCurves, KeyboardShapes, Phase,
};
use crate::bike::learning::{LearningLog, Side};
use crate::hud::UiFont;
use crate::locale::Language;
use crate::options::{OptionsMenu, OptionsTab, Page};
use crate::settings::Settings;
use crate::ui::text_font;

pub struct CurvesHudPlugin;

impl Plugin for CurvesHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_panel).add_systems(
            Update,
            (
                show_panel,
                draw_curves,
                place_dots,
                label_cards,
                show_learning,
            )
                .chain(),
        );
    }
}

/// How big each curve is drawn on the screen (logical px), and its image
/// (px: twice as fine, for sharp lines on any screen).
const CARD_WIDTH: f32 = 150.0;
const CARD_HEIGHT: f32 = 92.0;
const IMAGE_WIDTH: u32 = 300;
const IMAGE_HEIGHT: u32 = 184;
/// Room round the square the curve is drawn in, inside its image (px).
const INSET: f32 = 14.0;
/// How far below the top of the screen the panel starts (px): under the
/// performance HUD's line.
const PANEL_TOP: f32 = 40.0;
const GAP: f32 = 8.0;
/// Side of the dot that shows where the bike is on a curve (px).
const DOT: f32 = 9.0;
/// Where the panel is drawn in the interface's global order: over the
/// options (50), so that a curve can be watched while it is edited, under
/// the results (90).
const PANEL_LAYER: i32 = 60;

/// The colour of Control, the curves' own; the classic curve under them,
/// the control points and their arms, the grid and the ground.
const ACCENT: [f32; 4] = [0.22, 0.76, 1.0, 1.0];
/// A learnt curve's colour: the amber of what changes by itself.
const LEARNED: [f32; 4] = [1.0, 0.72, 0.2, 1.0];
const CLASSIC: [f32; 4] = [1.0, 1.0, 1.0, 0.28];
const HANDLE: [f32; 4] = [1.0, 0.42, 0.32, 0.95];
const GRID: [f32; 4] = [1.0, 1.0, 1.0, 0.1];
const GROUND: Color = Color::srgba(0.03, 0.04, 0.08, 0.72);
const EDGE: Color = Color::srgba(1.0, 1.0, 1.0, 0.18);
const LABEL: Color = Color::srgba(1.0, 1.0, 1.0, 0.85);

#[derive(Component)]
struct CurvesPanel;

/// A curve's frame, its words and its dot.
#[derive(Component)]
struct CurveCard(CurveKind);
#[derive(Component)]
struct CurveLabel(CurveKind);
#[derive(Component)]
struct CurveDot(CurveKind);
/// Under each curve, learning, how many times the bike strayed on it; and
/// under them all how far it is from the line now.
#[derive(Component)]
struct CurveNotes(CurveKind);
#[derive(Component)]
struct LearningStatus;

/// How long a curve's notes stay lit after it is noted (s).
const NOTE_LIT: f32 = 1.5;

/// The images the curves are drawn in, and the curves drawn in them.
#[derive(Resource)]
struct CurveImages {
    images: [Handle<Image>; 5],
    drawn: Option<KeyboardShapes>,
}

fn spawn_panel(mut commands: Commands, font: Res<UiFont>, mut images: ResMut<Assets<Image>>) {
    let images: [Handle<Image>; 5] = std::array::from_fn(|_| images.add(blank()));
    commands
        .spawn((
            CurvesPanel,
            GlobalZIndex(PANEL_LAYER),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: px(PANEL_TOP),
                right: px(16),
                width: px(2.0 * CARD_WIDTH + GAP),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexEnd,
                column_gap: px(GAP),
                row_gap: px(GAP),
                ..default()
            },
        ))
        .with_children(|panel| {
            for (kind, image) in CURVE_KINDS.into_iter().zip(&images) {
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(2),
                        ..default()
                    })
                    .with_children(|card| {
                        card.spawn((
                            CurveLabel(kind),
                            Text::new(""),
                            text_font(&font, 11.0),
                            TextColor(LABEL),
                        ));
                        card.spawn((
                            CurveCard(kind),
                            Node {
                                width: px(CARD_WIDTH),
                                height: px(CARD_HEIGHT),
                                border: UiRect::all(px(1)),
                                border_radius: BorderRadius::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(GROUND),
                            BorderColor::all(EDGE),
                        ))
                        .with_children(|frame| {
                            frame.spawn((
                                ImageNode::new(image.clone()),
                                Node {
                                    width: percent(100),
                                    height: percent(100),
                                    ..default()
                                },
                            ));
                            frame.spawn((
                                CurveDot(kind),
                                Visibility::Hidden,
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: px(DOT),
                                    height: px(DOT),
                                    margin: UiRect {
                                        left: px(-DOT / 2.0),
                                        top: px(-DOT / 2.0),
                                        ..default()
                                    },
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(Color::WHITE),
                                BorderColor::all(Color::srgb(0.02, 0.05, 0.1)),
                            ));
                        });
                        card.spawn((
                            CurveNotes(kind),
                            Text::new(""),
                            text_font(&font, 10.0),
                            TextColor(LABEL),
                            Node {
                                display: Display::None,
                                ..default()
                            },
                        ));
                    });
            }
            panel.spawn((
                LearningStatus,
                Text::new(""),
                text_font(&font, 12.0),
                TextColor(Color::srgba(LEARNED[0], LEARNED[1], LEARNED[2], 1.0)),
                Node {
                    display: Display::None,
                    width: percent(100),
                    ..default()
                },
            ));
        });
    commands.insert_resource(CurveImages {
        images,
        drawn: None,
    });
}

/// The panel shown or not, as the options say.
fn show_panel(settings: Res<Settings>, mut panel: Single<&mut Node, With<CurvesPanel>>) {
    let display = if settings.show_keyboard_curves {
        Display::Flex
    } else {
        Display::None
    };
    if panel.display != display {
        panel.display = display;
    }
}

/// Each curve drawn again when the curves change.
fn draw_curves(
    settings: Res<Settings>,
    mut drawn: ResMut<CurveImages>,
    mut images: ResMut<Assets<Image>>,
) {
    let shapes = settings.keyboard_shapes;
    if !settings.show_keyboard_curves || drawn.drawn == Some(shapes) {
        return;
    }
    let curves = shapes.curves();
    for (kind, handle) in CURVE_KINDS.into_iter().zip(&drawn.images) {
        let learned = shapes.shape(kind) == CurveShape::Learned;
        images
            .insert(handle, draw(kind, &curves, learned))
            .expect("the image of a curve is still there");
    }
    drawn.drawn = Some(shapes);
}

/// Where the bike is on curve `kind` (each a share of its square across
/// and up), if it is on it now: on A, B or C while the lean follows it,
/// how far through and how far over; on D, how fast and the lean it
/// allows there; on E, how far over and how much it turns.
fn dot_on(kind: CurveKind, probe: &CurveProbe) -> Option<Vec2> {
    if !probe.live {
        return None;
    }
    let lean = probe.lean;
    let following =
        |phase: Phase| (lean.phase == phase).then(|| Vec2::new(lean.progress, lean.lean.abs()));
    match kind {
        CurveKind::LeanIn => following(Phase::LeaningIn),
        CurveKind::StandUp => following(Phase::StandingUp),
        CurveKind::Unwind => following(Phase::Unwinding),
        CurveKind::Speed => Some(Vec2::new(probe.speed_share.min(1.0), probe.limit)),
        CurveKind::Turn => (lean.lean != 0.0).then(|| Vec2::new(lean.lean.abs(), probe.turn.abs())),
    }
}

/// A point of a curve's square (shares across and up) on its image (px
/// from the top left), as a share of the image.
fn on_image(point: Vec2) -> Vec2 {
    let size = Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
    let inner = size - 2.0 * INSET;
    Vec2::new(
        INSET + point.x * inner.x,
        size.y - INSET - point.y * inner.y,
    ) / size
}

fn place_dots(
    settings: Res<Settings>,
    probe: Res<CurveProbe>,
    mut dots: Query<(&CurveDot, &mut Node, &mut Visibility)>,
) {
    if !settings.show_keyboard_curves {
        return;
    }
    for (dot, mut node, mut visibility) in &mut dots {
        match dot_on(dot.0, &probe) {
            Some(point) => {
                let at = on_image(point.clamp(Vec2::ZERO, Vec2::ONE)) * 100.0;
                let (left, top) = (percent(at.x), percent(at.y));
                if node.left != left || node.top != top {
                    node.left = left;
                    node.top = top;
                }
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
}

/// Each curve's words (its letter, name and number), and the one being
/// edited in the options edged in the colour of Control.
fn label_cards(
    settings: Res<Settings>,
    menu: Res<OptionsMenu>,
    mut labels: Query<(&CurveLabel, &mut Text)>,
    mut cards: Query<(&CurveCard, &mut BorderColor)>,
) {
    if !settings.show_keyboard_curves {
        return;
    }
    let lang = settings.language;
    let curves = &settings.keyboard_shapes;
    for (label, mut text) in &mut labels {
        let kind = label.0;
        let number = curves
            .scale(kind)
            .map(|value| format!("  {}", kind.scale_text(value, lang)))
            .unwrap_or_default();
        let words = format!("{}  {}{number}", kind.letter(), kind.short_name(lang));
        if text.0 != words {
            text.0 = words;
        }
    }
    let editing = (menu.is_open
        && menu.tab == OptionsTab::Control
        && menu.page == Some(Page::KeyboardCurves))
    .then_some(menu.curve);
    for (card, mut border) in &mut cards {
        let edge = if editing == Some(card.0) {
            Color::srgba(ACCENT[0], ACCENT[1], ACCENT[2], 1.0)
        } else {
            EDGE
        };
        border.set_if_neq(BorderColor::all(edge));
    }
}

/// Learning, under each curve how many times the bike went wide and
/// inside on it (lit for a moment when noted), and under them all how far
/// it is from the line and whether it corrects.
fn show_learning(
    settings: Res<Settings>,
    log: Res<LearningLog>,
    time: Res<Time>,
    mut notes: Query<(&CurveNotes, &mut Text, &mut TextColor, &mut Node), Without<LearningStatus>>,
    mut status: Single<(&mut Text, &mut Node), With<LearningStatus>>,
) {
    if !settings.show_keyboard_curves {
        return;
    }
    let lang = settings.language;
    let display = if log.active {
        Display::Flex
    } else {
        Display::None
    };
    let now = time.elapsed_secs();
    for (note, mut text, mut color, mut node) in &mut notes {
        if node.display != display {
            node.display = display;
        }
        let words = notes_text(&log, note.0, lang);
        if text.0 != words {
            text.0 = words;
        }
        let lit = log
            .last
            .is_some_and(|(kind, _, at)| kind == note.0 && now - at < NOTE_LIT);
        let tint = if lit {
            Color::srgba(LEARNED[0], LEARNED[1], LEARNED[2], 1.0)
        } else {
            LABEL
        };
        color.set_if_neq(TextColor(tint));
    }
    let (text, node) = &mut *status;
    if node.display != display {
        node.display = display;
    }
    let words = status_text(&log, settings.keyboard_shapes.correct, lang);
    if text.0 != words {
        text.0 = words;
    }
}

/// How many times the bike went wide and inside on curve `kind`.
fn notes_text(log: &LearningLog, kind: CurveKind, lang: Language) -> String {
    let (wide, inside) = (log.count(kind, Side::Wide), log.count(kind, Side::Inside));
    match lang {
        Language::Spanish => format!("fuera {wide} · dentro {inside}"),
        Language::English => format!("wide {wide} · inside {inside}"),
    }
}

/// How far from the line the bike is (round a corner, out wide or inside;
/// on a straight, to the left or right), and whether it corrects.
fn status_text(log: &LearningLog, correcting: bool, lang: Language) -> String {
    let metres = log.stray.abs();
    let side = match (log.in_corner, log.stray >= 0.0, lang) {
        (true, true, Language::Spanish) => "fuera",
        (true, false, Language::Spanish) => "dentro",
        (false, true, Language::Spanish) => "izquierda",
        (false, false, Language::Spanish) => "derecha",
        (true, true, Language::English) => "wide",
        (true, false, Language::English) => "inside",
        (false, true, Language::English) => "left",
        (false, false, Language::English) => "right",
    };
    let doing = match (correcting, lang) {
        (false, Language::Spanish) => "solo detección",
        (true, Language::Spanish) => "corrigiendo",
        (false, Language::English) => "noting only",
        (true, Language::English) => "correcting",
    };
    match lang {
        Language::Spanish => format!("Desvío {metres:.1} m {side} · {doing}"),
        Language::English => format!("Off the line {metres:.1} m {side} · {doing}"),
    }
}

/// An image the size of a curve's, empty.
fn blank() -> Image {
    image(vec![[0.0; 4]; (IMAGE_WIDTH * IMAGE_HEIGHT) as usize])
}

fn image(pixels: Vec<[f32; 4]>) -> Image {
    let data = pixels
        .iter()
        .flat_map(|pixel| pixel.map(|channel| (channel * 255.0).round() as u8))
        .collect();
    Image::new(
        Extent3d {
            width: IMAGE_WIDTH,
            height: IMAGE_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// How high curve `kind` of `curves` is drawn (a share of its square) for
/// the height `y` of its Bézier: A and E as they are; B and C, what is
/// left of the lean, coming down; D, the lean allowed, from all of it down
/// to what is kept at top speed.
fn shown_height(kind: CurveKind, curves: &KeyboardCurves, y: f32) -> f32 {
    match kind {
        CurveKind::LeanIn | CurveKind::Turn => y,
        CurveKind::StandUp | CurveKind::Unwind => 1.0 - y,
        CurveKind::Speed => 1.0 - (1.0 - curves.top_speed_lean) * y,
    }
}

/// The points of curve `kind` of `curves` as drawn (shares of its square),
/// from one side to the other.
fn shown_curve(kind: CurveKind, curves: &KeyboardCurves) -> Vec<Vec2> {
    let bezier = curves.bezier(kind);
    (0..=96)
        .map(|k| {
            let (x, y) = bezier.point(k as f32 / 96.0);
            Vec2::new(x, shown_height(kind, curves, y))
        })
        .collect()
}

/// Curve `kind` of `curves`, drawn (in amber if `learned`): the grid, the
/// classic curve in
/// grey, the arms to its control points and the curve itself.
pub fn draw(kind: CurveKind, curves: &KeyboardCurves, learned: bool) -> Image {
    let mut canvas = Canvas::new();
    for k in 0..=4 {
        let share = k as f32 / 4.0;
        canvas.line(
            Vec2::new(share, 0.0),
            Vec2::new(share, 1.0),
            0.6,
            GRID,
            false,
        );
        canvas.line(
            Vec2::new(0.0, share),
            Vec2::new(1.0, share),
            0.6,
            GRID,
            false,
        );
    }
    let classic = KeyboardCurves::default();
    canvas.polyline(&shown_curve(kind, &classic), 1.6, CLASSIC);
    // The control points, each with its arm from the end it pulls on.
    let bezier = curves.bezier(kind);
    let shown = |x: f32, y: f32| Vec2::new(x, shown_height(kind, curves, y));
    for (end, point) in [
        (shown(0.0, 0.0), shown(bezier.x1, bezier.y1)),
        (shown(1.0, 1.0), shown(bezier.x2, bezier.y2)),
    ] {
        canvas.line(end, point, 1.0, HANDLE, true);
        canvas.dot(point, 3.5, HANDLE);
    }
    let color = if learned { LEARNED } else { ACCENT };
    canvas.polyline(&shown_curve(kind, curves), 2.4, color);
    image(canvas.pixels)
}

/// The pixels of a curve's image, drawn on with soft round strokes.
struct Canvas {
    pixels: Vec<[f32; 4]>,
}

impl Canvas {
    fn new() -> Self {
        Self {
            pixels: vec![[0.0; 4]; (IMAGE_WIDTH * IMAGE_HEIGHT) as usize],
        }
    }

    /// A point of the square (shares) in pixels of the image.
    fn at(point: Vec2) -> Vec2 {
        on_image(point) * Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32)
    }

    /// A round spot `radius` px wide at `center` (px), its edge softened.
    fn stamp(&mut self, center: Vec2, radius: f32, color: [f32; 4]) {
        let (width, height) = (IMAGE_WIDTH as usize, IMAGE_HEIGHT as usize);
        let low = (center - radius - 1.0).max(Vec2::ZERO);
        let high = (center + radius + 1.0).min(Vec2::new(width as f32 - 1.0, height as f32 - 1.0));
        if low.x > high.x || low.y > high.y {
            return;
        }
        for y in low.y as usize..=high.y as usize {
            for x in low.x as usize..=high.x as usize {
                let middle = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let cover = (radius + 0.5 - middle.distance(center)).clamp(0.0, 1.0) * color[3];
                if cover <= 0.0 {
                    continue;
                }
                let pixel = &mut self.pixels[y * width + x];
                for channel in 0..3 {
                    pixel[channel] += (color[channel] - pixel[channel]) * cover;
                }
                pixel[3] += (1.0 - pixel[3]) * cover;
            }
        }
    }

    /// A line between two points of the square, broken into dashes if
    /// `dashed`.
    fn line(&mut self, a: Vec2, b: Vec2, radius: f32, color: [f32; 4], dashed: bool) {
        let (a, b) = (Self::at(a), Self::at(b));
        let steps = (a.distance(b) * 2.0).ceil().max(1.0) as usize;
        for step in 0..=steps {
            // Dashes and gaps of 5 px.
            if dashed && (step / 10) % 2 == 1 {
                continue;
            }
            self.stamp(a.lerp(b, step as f32 / steps as f32), radius, color);
        }
    }

    fn polyline(&mut self, points: &[Vec2], radius: f32, color: [f32; 4]) {
        // Each stretch whole, then the next: where they meet, no darker.
        let mut stamped = Vec::new();
        for pair in points.windows(2) {
            let (a, b) = (Self::at(pair[0]), Self::at(pair[1]));
            let steps = (a.distance(b) * 2.0).ceil().max(1.0) as usize;
            stamped.extend((0..steps).map(|step| a.lerp(b, step as f32 / steps as f32)));
        }
        if let Some(&last) = points.last() {
            stamped.push(Self::at(last));
        }
        for center in stamped {
            self.stamp(center, radius, color);
        }
    }

    fn dot(&mut self, point: Vec2, radius: f32, color: [f32; 4]) {
        self.stamp(Self::at(point), radius, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bike::curves::CurveLean;

    #[test]
    fn learning_says_how_often_and_how_far_the_bike_strays() {
        let log = LearningLog {
            active: true,
            stray: -2.04,
            in_corner: true,
            notes: [[0, 0], [1, 2], [0, 0], [0, 0], [0, 0]],
            ..default()
        };
        assert_eq!(
            notes_text(&log, CurveKind::StandUp, Language::Spanish),
            "fuera 1 · dentro 2"
        );
        assert_eq!(
            status_text(&log, false, Language::Spanish),
            "Desvío 2.0 m dentro · solo detección"
        );
        assert_eq!(
            status_text(&log, true, Language::English),
            "Off the line 2.0 m inside · correcting"
        );
    }

    #[test]
    fn the_square_of_a_curve_sits_inside_its_image() {
        let bottom_left = on_image(Vec2::ZERO);
        let top_right = on_image(Vec2::ONE);
        assert!(bottom_left.x > 0.0 && bottom_left.y < 1.0);
        assert!(top_right.x < 1.0 && top_right.y > 0.0);
        assert!(bottom_left.y > top_right.y, "up is up");
    }

    #[test]
    fn each_curve_is_drawn_from_where_it_starts_to_where_it_ends() {
        let curves = KeyboardCurves::default();
        let ends = |kind| {
            let points = shown_curve(kind, &curves);
            (points[0], *points.last().unwrap())
        };
        // A and E rise from nought to all of it; B and C come down.
        assert_eq!(ends(CurveKind::LeanIn), (Vec2::ZERO, Vec2::ONE));
        assert_eq!(ends(CurveKind::Turn), (Vec2::ZERO, Vec2::ONE));
        assert_eq!(ends(CurveKind::StandUp), (Vec2::Y, Vec2::X));
        // D, from all the lean to what is kept at top speed.
        let (start, end) = ends(CurveKind::Speed);
        assert_eq!(start, Vec2::Y);
        assert!((end.y - curves.top_speed_lean).abs() < 1e-6);
        // Drawn: some pixels in the colour of the curve.
        let image = draw(CurveKind::LeanIn, &curves, false);
        let data = image.data.unwrap();
        assert!(
            data.chunks(4)
                .any(|p| p[3] > 200 && p[2] > 200 && p[0] < 120)
        );
    }

    #[test]
    fn the_dots_show_where_the_bike_is_on_the_curves_it_follows() {
        let curves = KeyboardCurves::default();
        let mut lean = CurveLean::default();
        for _ in 0..10 {
            lean.update(1.0, &curves, 0.5, 1.0 / 120.0);
        }
        let probe = CurveProbe {
            live: true,
            lean,
            speed_share: 0.5,
            limit: curves.lean_limit(0.5),
            turn: curves.turn_for(lean.lean),
        };
        let on_a = dot_on(CurveKind::LeanIn, &probe).unwrap();
        assert!((on_a.y - lean.lean).abs() < 1e-6);
        assert!(dot_on(CurveKind::StandUp, &probe).is_none());
        assert!(dot_on(CurveKind::Unwind, &probe).is_none());
        assert_eq!(dot_on(CurveKind::Speed, &probe).unwrap().x, 0.5);
        assert!(dot_on(CurveKind::Turn, &probe).is_some());
        // With the curves off, none.
        let off = CurveProbe::default();
        assert!(CURVE_KINDS.iter().all(|&kind| dot_on(kind, &off).is_none()));
    }
}
