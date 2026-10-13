// The sea, drawn like the cartoon sea of Zelda: The Wind Waker: flat,
// saturated blues with no lighting, white waves coming and going on it,
// lighter water by the coast, and white foam hugging the shore with rings
// of foam rolling out from it (see src/world/sea.rs).

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{globals, view},
    pbr_functions::main_pass_post_lighting_processing,
    pbr_types,
}

// xy: corner of the shore map (m, x and z); zw: its size (m).
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> shore_area: vec4<f32>;
// Signed distance to the shore (m), see `SHORE_NEAR` and `SHORE_FAR`.
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var shore_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var shore_sampler: sampler;

// What the shore map holds: from SHORE_NEAR (inland) to SHORE_FAR (out at
// sea) metres from the shore. Must match src/world/sea.rs.
const SHORE_NEAR: f32 = -10.0;
const SHORE_FAR: f32 = 40.0;

// The colours (sRGB): the open sea, the sea by the coast, and the foam.
const DEEP: vec3<f32> = vec3(0.07, 0.29, 0.78);
const OPEN: vec3<f32> = vec3(0.12, 0.42, 0.88);
const SHALLOW: vec3<f32> = vec3(0.25, 0.72, 0.93);
const FOAM: vec3<f32> = vec3(0.97, 0.99, 1.0);

// Size of the cells the waves are scattered over (m), how thick a wave is in
// the middle (in cell units), how long each one lives (s) and how far away
// they fade out (m).
const WAVE_CELL: f32 = 7.0;
const WAVE_LINE: f32 = 0.045;
const WAVE_LIFE: f32 = 5.0;
const WAVE_FADE: vec2<f32> = vec2(160.0, 380.0);
// The foam rings: how far they roll out from the shore (m), how long that
// takes (s) and how thick they are (m).
const RING_REACH: f32 = 13.0;
const RING_PERIOD: f32 = 4.5;
const RING_WIDTH: f32 = 0.9;

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let q = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(q) * 43758.5453);
}

fn hash1(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2(41.3, 289.1))) * 15731.743);
}

// Smooth value noise, from 0 to 1.
fn noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash1(cell);
    let b = hash1(cell + vec2(1.0, 0.0));
    let c = hash1(cell + vec2(0.0, 1.0));
    let d = hash1(cell + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// How much of a wave covers this point. Each cell of a jittered grid may
// hold one wave: a short arc, like a crest (︵), thick in the middle and
// thin at the ends, leaning a little its own way. Waves drift with the wind
// and each one swells and fades away in turn, so they come and go.
fn waves(p: vec2<f32>, t: f32) -> f32 {
    let q = p + vec2(0.06, 0.035) * t;
    let cell = floor(q);
    var cover = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let id = cell + vec2(f32(x), f32(y));
            let h = hash2(id);
            let k = hash1(id + 17.3);
            // About two cells in three have a wave. (No branching: the
            // derivatives below need every pixel to run the same code.)
            let present = step(0.35, k);
            // Each wave lives WAVE_LIFE seconds, from nothing to full and back.
            let life = fract(t / WAVE_LIFE + h.x);
            let swell = sin(life * 3.14159);
            let center = id + 0.5 + (h - 0.5) * 0.5;
            let radius = (0.28 + 0.14 * h.y) * (0.75 + 0.25 * swell);
            // Lean the wave a little either way.
            let lean = (k - 0.675) * 0.9;
            let d = rotate(q - center, lean);
            // The top of a circle: an arc of `span` either side of straight up.
            let span = 0.75 + 0.35 * swell;
            let angle = atan2(d.x, -d.y);
            let along = clamp(angle / span, -1.0, 1.0);
            let ring = abs(length(d) - radius);
            let past_end = max(abs(angle) - span, 0.0) * radius;
            let distance = length(vec2(ring, past_end));
            let thickness = WAVE_LINE * (1.0 - along * along) * swell * present;
            let width = fwidth(distance) + 1e-4;
            let line = 1.0 - smoothstep(thickness - width, thickness + width, distance);
            cover = max(cover, line * present);
        }
    }
    return cover;
}

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn to_linear(srgb: vec3<f32>) -> vec3<f32> {
    return pow(srgb, vec3(2.2));
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    let t = globals.time;
    let at = in.world_position.xz;

    // Distance to the shore (m): far out at sea outside the map.
    // (Sampled everywhere: textures cannot be sampled under a branch.)
    let uv = (at - shore_area.xy) / shore_area.zw;
    let stored = textureSample(shore_map, shore_sampler, clamp(uv, vec2(0.0), vec2(1.0))).r;
    let inside = all(uv >= vec2(0.0)) && all(uv <= vec2(1.0));
    let shore = select(SHORE_FAR, mix(SHORE_NEAR, SHORE_FAR, stored), inside);
    // The coast wobbles a little, and the foam with it.
    let wobble = (noise(at * 0.18 + t * 0.4) - 0.5) * 1.2;
    let near = shore + wobble;

    // Broad patches of deeper and lighter blue out at sea.
    let patches = noise(at / 260.0 + vec2(3.7, -1.9));
    var color = mix(DEEP, OPEN, smoothstep(0.25, 0.75, patches));
    color = mix(SHALLOW, color, smoothstep(2.0, 30.0, near));

    // White waves, fading out with distance (they would shimmer).
    let distance = length(in.world_position.xyz - view.world_position.xyz);
    let fade = 1.0 - smoothstep(WAVE_FADE.x, WAVE_FADE.y, distance);
    var foam = waves(at / WAVE_CELL, t) * 0.9 * fade;

    // Foam along the shore, and rings of foam rolling out from it, fading
    // as they go.
    // At least a pixel wide, however far: seen from afar they stay lines.
    let pixel = fwidth(near);
    foam = max(foam, 1.0 - smoothstep(0.4, 1.6 + pixel, near));
    let ring_width = max(RING_WIDTH, pixel * 1.2);
    for (var ring = 0; ring < 2; ring++) {
        let phase = fract(t / RING_PERIOD + f32(ring) * 0.5);
        let reach = phase * RING_REACH + 1.0;
        let line = 1.0 - smoothstep(ring_width * 0.5, ring_width, abs(near - reach));
        foam = max(foam, line * (1.0 - phase) * step(0.0, near));
    }

    let srgb = mix(color, FOAM, clamp(foam, 0.0, 1.0));

    // Fog and tone mapping like everything else, with no lighting.
    var pbr_input = pbr_types::pbr_input_new();
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT
        | pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT;
    return main_pass_post_lighting_processing(pbr_input, vec4(to_linear(srgb), 1.0));
}
