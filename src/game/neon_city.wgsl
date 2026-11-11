// Puerto Neón's city at night (see CityLights in src/game/neon.rs): the
// city pack's own material, lit as it is, and over the dark glass its
// models mark (windows.jpg), windows drawn where they are on their walls
// (the second set of coordinates: how far along the wall, how far up, in
// metres), so that they are sharp however near; some lit, in the whites of
// offices and homes and now and then a colour, a few going on and off.
// Far off, where a window is only a few pixels, as much light as they give
// on average, so that they do not shimmer. On a block chosen for it, a
// rain of code instead (after the canvas demo of wasm-gaming's
// engine-specs): columns of characters falling, each at its own speed,
// trailing away, their heads white, their trails in colours that go round
// the rainbow with the time and from one column to the next.

#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
}

// Must match CityLook in src/game/neon.rs.
struct CityLook {
    // The light's tint (rgb) and how bright the windows are (w).
    tint: vec4<f32>,
    // A window's width and a floor's height (m), how many are lit, and 1 for
    // the rain of code instead of windows.
    shape: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> look: CityLook;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var windows_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var windows_sampler: sampler;

// A random number from 0 to 1 for three whole numbers.
fn hash3(p: vec3<i32>) -> f32 {
    var v = bitcast<vec3<u32>>(p) * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3(16u));
    v.x += v.y * v.z;
    return f32(v.x & 0xffffu) / 65535.0;
}

// The colour a lit window is lit in, from a random number: the whites
// mostly, now and then cyan, magenta, amber or violet.
fn window_colour(pick: f32) -> vec3<f32> {
    if pick < 0.34 { return vec3(1.0, 0.82, 0.55); }
    if pick < 0.58 { return vec3(0.8, 0.9, 1.0); }
    if pick < 0.68 { return vec3(1.0, 0.95, 0.75); }
    if pick < 0.77 { return vec3(0.25, 0.95, 1.0); }
    if pick < 0.86 { return vec3(1.0, 0.3, 0.75); }
    if pick < 0.94 { return vec3(1.0, 0.65, 0.2); }
    return vec3(0.65, 0.4, 1.0);
}

// How much of the pixel a box `half` wide either way about the middle of
// a cell covers, at `f` (0 to 1 across the cell) with `fw` of a cell to a
// pixel: sharp, smoothed at its edges by a pixel.
fn in_box(f: vec2<f32>, half: vec2<f32>, fw: vec2<f32>) -> f32 {
    let d = half - abs(f - 0.5);
    let edge = clamp(d / max(fw, vec2(1e-5)) + 0.5, vec2(0.0), vec2(1.0));
    return edge.x * edge.y;
}

// Hue (0 to 1), saturation and lightness to a colour.
fn hsl(h: f32, s: f32, l: f32) -> vec3<f32> {
    let k = (vec3(0.0, 8.0, 4.0) + h * 12.0) % vec3(12.0);
    let a = s * min(l, 1.0 - l);
    return l - a * max(min(min(k - 3.0, 9.0 - k), vec3(1.0)), vec3(-1.0));
}

// The windows of a wall at `wall` (m along it, m up), the wall facing
// `facing` (to tell walls apart): their light.
fn windows(wall: vec2<f32>, facing: vec2<f32>) -> vec3<f32> {
    let size = look.shape.xy;
    let at = wall / size;
    let cell = floor(at);
    let fw = fwidth(at);
    let side = vec2<i32>(round(facing * 6.0));
    let id = vec3<i32>(i32(cell.x) * 7 + side.x * 131, i32(cell.y), side.y * 977);
    // Some go on and off now and then, each at its own pace.
    let pace = 25.0 + 60.0 * hash3(id + vec3(3, 5, 7));
    let flicker = select(0.0, floor(globals.time / pace), hash3(id + vec3(11, 0, 0)) < 0.12);
    let lit = hash3(id + vec3(0, 0, i32(flicker))) < look.shape.z;
    let colour = window_colour(hash3(id + vec3(17, 23, 0)));
    let glass = in_box(at - cell, vec2(0.34, 0.3), fw);
    let sharp = select(0.0, glass, lit) * colour;
    // Far off: on average, as many lit as are, in their colours.
    let average = look.shape.z * 0.35 * vec3(0.95, 0.85, 0.85);
    let crowded = smoothstep(0.12, 0.45, max(fw.x, fw.y));
    return mix(sharp, average, crowded);
}

// The rain of code over a wall at `wall` (m along it, m up).
fn rain(wall: vec2<f32>) -> vec3<f32> {
    let cell_size = 1.4;
    let at = vec2(wall.x, -wall.y) / cell_size;
    let cell = floor(at);
    let column = i32(cell.x);
    let row = cell.y;
    let fw = fwidth(at);
    // Each column its speed (rows a second), its trail's length, where its
    // drop starts, and how bright.
    let speed = (0.5 + 1.5 * hash3(vec3(column, 1, 0))) * 6.0;
    let length = 5.0 + floor(19.0 * hash3(vec3(column, 2, 0)));
    let period = 70.0;
    let head = (globals.time * speed + period * hash3(vec3(column, 3, 0))) % period;
    let behind = ((head - row) % period + period) % period;
    let shown = select(0.0, 1.0, behind < length);
    let fade = (0.3 + 0.7 * hash3(vec3(column, 4, 0))) * (1.0 - behind / length);
    // The character: strokes on a grid of three by four, changing now and
    // then.
    let inside = at - cell;
    let change = floor(globals.time * (1.0 + 2.0 * hash3(vec3(column, i32(row), 5))));
    let id = vec3(column, i32(row), i32(change));
    let grid = (inside - vec2(0.15, 0.1)) / vec2(0.7, 0.8);
    let g = grid * vec2(2.0, 3.0);
    let gc = floor(g);
    let gf = g - gc;
    // A stroke across at each line of the grid, or down at each column.
    let across = hash3(id + vec3(0, i32(gc.y) * 3, 31)) < 0.55;
    let down = hash3(id + vec3(i32(gc.x) * 5, 0, 37)) < 0.5;
    let width = 0.18;
    let line_across = select(0.0, 1.0 - smoothstep(width, width + fw.y * 3.0, abs(gf.y - 0.5)), across);
    let line_down = select(0.0, 1.0 - smoothstep(width, width + fw.x * 2.0, abs(gf.x - 0.5)), down);
    let within = all(grid > vec2(0.0)) && all(grid < vec2(1.0));
    let mark = select(0.0, max(line_across, line_down), within);
    let hue = fract(globals.time * 66.0 / 360.0 + f32(column) * 10.0 / 360.0);
    let colour = select(hsl(hue, 0.8, 0.62), vec3(0.86, 1.0, 1.0), behind < 1.0);
    // Far off, a haze of its colour.
    let crowded = smoothstep(0.15, 0.5, max(fw.x, fw.y));
    return shown * colour * fade * mix(mark, 0.3, crowded);
}

// A sign's panel as a screen of lit dots, as an LCD's (`panel` its number,
// `at` where on it: m across from its middle, m up from its foot): the
// panel's own lit colour behind, dimmed, and over it its dots lit in turns
// of animations, each panel at its own: bars rising and falling as a
// sound's, stripes running across, rings spreading from its middle,
// characters running past. How much of its light shines there.
fn screen(panel: i32, at: vec2<f32>) -> f32 {
    let pitch = 0.8;
    let dots = at / pitch;
    let dot = floor(dots);
    let inside = dots - dot;
    let fw = fwidth(dots);
    // Each dot a square with a dark gap round it.
    let lens = in_box(inside, vec2(0.38), fw);
    let t = globals.time;
    let turn = i32(floor(t / 7.0 + f32(panel) * 0.37));
    let mode = (turn + panel) % 4;
    var on = false;
    if mode == 0 {
        // Bars rising and falling, as a sound's.
        let level = (0.5 + 0.5 * sin(t * 4.0 + dot.x * 0.9 + f32(panel))) * 15.0;
        on = dot.y < level;
    } else if mode == 1 {
        // Stripes running across.
        on = ((i32(dot.x + dot.y) + i32(floor(t * 9.0))) % 8 + 8) % 8 < 3;
    } else if mode == 2 {
        // Rings spreading from the middle.
        let d = length(dot - vec2(0.0, 8.0));
        on = fract(d / 4.0 - t * 0.9) < 0.38;
    } else {
        // Characters running past: three by five dots each, in cells of four
        // by seven.
        let x = i32(dot.x + floor(t * 10.0) + 100.0);
        let y = i32(dot.y);
        let glyph = vec3(x / 4, y / 7, panel * 13);
        let gx = x % 4;
        let gy = y % 7;
        on = gx < 3 && gy > 0 && gy < 6 && hash3(glyph + vec3(gx * 3, gy * 5, 7)) < 0.55;
    }
    // Far off, its average.
    let crowded = smoothstep(0.2, 0.6, max(fw.x, fw.y));
    let near = select(0.3, 1.0, on) * (0.55 + 0.45 * lens);
    return mix(near, 0.6, crowded);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

#ifdef VERTEX_UVS_B
#ifdef VERTEX_UVS_A
    let wall = in.uv_b;
    let marked = textureSample(windows_texture, windows_sampler, in.uv).r;
    // (Both worked out everywhere, as their derivatives must be, then the
    // one wanted taken where it is wanted.)
    let n = normalize(in.world_normal);
    let lit_windows = windows(wall, n.xz);
    let code = rain(wall);
    let light = select(lit_windows, code, look.shape.w > 0.5);
    let on = select(0.0, 1.0, wall.y > -500.0 && marked > 0.5 && wall.x < 5000.0);
    // A sign's panel: its light as a screen's.
    let is_screen = wall.x > 5000.0;
    let panel = i32(floor((wall.x - 10000.0 + 500.0) / 1000.0));
    let across = wall.x - 10000.0 - 1000.0 * f32(panel);
    let shown = screen(panel, vec2(across, wall.y));
    let dimmed = select(1.0, shown, is_screen);
    pbr_input.material.emissive = vec4(
        pbr_input.material.emissive.rgb * dimmed + on * light * look.tint.rgb * look.tint.w,
        pbr_input.material.emissive.a,
    );
#endif
#endif

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
