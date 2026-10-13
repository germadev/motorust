// The sea, drawn like the cartoon sea of Zelda: The Wind Waker: flat,
// saturated blues with no lighting, white wavy lines drifting across it,
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

// Size of the cells of the wave lines (m), how thick the lines are (in cell
// units) and how far away they fade out (m).
const WAVE_CELL: f32 = 16.0;
const WAVE_LINE: f32 = 0.055;
const WAVE_FADE: vec2<f32> = vec2(180.0, 420.0);
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

// How much of a white wave line covers this point: the borders between
// drifting, wobbling cells, broken into separate squiggles.
fn wave_lines(p: vec2<f32>, t: f32) -> f32 {
    let drift = vec2(0.031, 0.019) * t;
    let warp = vec2(noise(p * 0.35 + t * 0.05), noise(p * 0.35 + vec2(5.2, 1.3) - t * 0.05));
    let q = p + drift + (warp - 0.5) * 0.9;
    let cell = floor(q);
    let f = fract(q);
    var first = 8.0;
    var second = 8.0;
    var nearest = vec2(0.0);
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2(f32(x), f32(y));
            let h = hash2(cell + offset);
            // Each point circles slowly around its place.
            let point = offset + 0.5 + 0.38 * sin(t * 0.6 + 6.2831 * h) - f;
            let distance = dot(point, point);
            if distance < first {
                second = first;
                first = distance;
                nearest = cell + offset;
            } else if distance < second {
                second = distance;
            }
        }
    }
    let edge = sqrt(second) - sqrt(first);
    let width = fwidth(edge);
    let line = 1.0 - smoothstep(WAVE_LINE - width, WAVE_LINE + width, edge);
    // Only some cells draw their border, so the lines come in pieces.
    let shown = step(0.45, hash1(nearest * 1.7 + 3.1));
    return line * shown;
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

    // White wave lines, fading out with distance (they would shimmer).
    let distance = length(in.world_position.xyz - view.world_position.xyz);
    let fade = 1.0 - smoothstep(WAVE_FADE.x, WAVE_FADE.y, distance);
    var foam = wave_lines(at / WAVE_CELL, t) * 0.85 * fade;

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
