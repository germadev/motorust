// The lake behind a dam: still, dark water that mirrors what stands round
// it, the hills, the road, the dam and the sky, more the lower one looks
// across it, stirred by a faint ripple (see LakeMaterial in
// src/world/dam.rs).

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{globals, view},
    pbr_functions::main_pass_post_lighting_processing,
    pbr_types,
}

// xy: corner of the map of the lake's surroundings (m, x and z); zw: its
// size (m).
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> area: vec4<f32>;
// x: the water's level (m).
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> water: vec4<f32>;
// What stands round the lake: rgb, its colour as lit (linear); a, how high
// it stands over the water (m, less than 0 under it).
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var surroundings: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var surroundings_sampler: sampler;
// The sky's colour each way: across, round from the north (-Z) towards the
// east (+X); down, from overhead to the horizon.
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var sky_sampler: sampler;
struct LakeLook {
    // The water's own colour, deep down (linear).
    deep: vec4<f32>,
    // xyz: towards the sun; w: how bright its glint is.
    sun: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<uniform> look: LakeLook;

const PI: f32 = 3.14159265;
// The reflected ray is followed this far (m) in this many steps, closer
// together near the water, and its hit narrowed down in a few more.
const REACH: f32 = 520.0;
const STEPS: i32 = 64;
const REFINE: i32 = 6;
// The ripple: how much it tilts the water, how big its swells are (m), and
// from how far (m) it fades out, where it would shimmer.
const RIPPLE_TILT: f32 = 0.035;
const RIPPLE_SIZE: f32 = 3.5;
const RIPPLE_FADE: vec2<f32> = vec2(40.0, 260.0);

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

// How the ripple tilts the water at `at` (x and z slopes): two faint sets
// of swells drifting slowly across each other.
fn ripple(at: vec2<f32>, t: f32) -> vec2<f32> {
    let p = at / RIPPLE_SIZE;
    let a = vec2(noise(p + vec2(t * 0.21, t * 0.07)), noise(p + vec2(5.2, 1.3) - vec2(t * 0.05, t * 0.18)));
    let b = vec2(noise(p * 2.3 + vec2(9.1, -t * 0.3)), noise(p * 2.3 + vec2(t * 0.26, 4.7)));
    return (a + 0.5 * b - 0.75) * RIPPLE_TILT;
}

// What the map holds at `at`: off it, nothing, down at the water.
fn surroundings_at(at: vec2<f32>) -> vec4<f32> {
    let uv = (at - area.xy) / area.zw;
    let stored = textureSampleLevel(surroundings, surroundings_sampler, clamp(uv, vec2(0.0), vec2(1.0)), 0.0);
    let inside = all(uv >= vec2(0.0)) && all(uv <= vec2(1.0));
    return select(vec4(0.0), stored, inside);
}

// Whether a point is under the ground round the lake.
fn under(q: vec3<f32>) -> bool {
    return q.y - water.x < surroundings_at(q.xz).a;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.world_position.xyz;
    let to_eye = view.world_position.xyz - p;
    let distance = length(to_eye);
    let v = to_eye / distance;

    let fade = 1.0 - smoothstep(RIPPLE_FADE.x, RIPPLE_FADE.y, distance);
    let slope = ripple(p.xz, globals.time) * fade;
    let n = normalize(vec3(-slope.x, 1.0, -slope.y));
    var r = reflect(-v, n);
    r.y = max(r.y, 0.001);

    // Follow the reflected ray up from the water until it runs into the
    // ground round the lake, or leaves the map for the sky.
    var hit = false;
    var near = 0.0;
    var far = 0.0;
    for (var i = 1; i <= STEPS; i++) {
        let f = f32(i) / f32(STEPS);
        let t = REACH * f * f;
        if under(p + r * t) {
            hit = true;
            far = t;
            break;
        }
        near = t;
    }
    for (var k = 0; k < REFINE; k++) {
        let t = 0.5 * (near + far);
        let below = under(p + r * t);
        far = select(far, t, below);
        near = select(t, near, below);
    }
    let ground = surroundings_at((p + r * far).xz).rgb;

    let elevation = asin(clamp(r.y, 0.0, 1.0));
    let azimuth = atan2(r.x, -r.z);
    let sky_uv = vec2(fract(azimuth / (2.0 * PI)), 1.0 - elevation / (0.5 * PI));
    let sky_color = textureSampleLevel(sky, sky_sampler, sky_uv, 0.0).rgb;
    let glint = pow(max(dot(r, look.sun.xyz), 0.0), 600.0) * look.sun.w;
    let reflected = select(sky_color + vec3(glint), ground, hit);

    // Water reflects little looking down into it and more and more the
    // lower one looks across it (Schlick's Fresnel).
    let facing = clamp(dot(v, n), 0.0, 1.0);
    let fresnel = 0.02 + 0.98 * pow(1.0 - facing, 5.0);
    let color = mix(look.deep.rgb, reflected, fresnel);

    // Fog and tone mapping like everything else, with no lighting: what it
    // mirrors is lit already.
    var pbr_input = pbr_types::pbr_input_new();
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT
        | pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT;
    return main_pass_post_lighting_processing(pbr_input, vec4(color, 1.0));
}
