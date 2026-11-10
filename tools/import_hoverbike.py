#!/usr/bin/env python3
"""Prepares a Meshy hover bike for the game.

Reads the GLB of a hover bike (one mesh, with its colour and its
metal-roughness textures; the originals are in the repository's model
releases) and writes in `assets/models/NAME/` what the game loads for a
model that hovers (the engine's `src/bike/model/hover.rs`):

- `bike.bin`: the whole bike as one part, `chassis`, in the game's axes
  (forward -Z, right +X, up +Y) and size (as long as a bike with wheels,
  from NOSE to TAIL), floating: its lowest point CLEARANCE over the ground.
- `base_color.jpg`, `metallic_roughness.jpg`, and `normal.jpg`, flat (Meshy
  gives these models none).
- `emissive.jpg`: its lights, lit whatever light falls on it: the bright
  cyan and blue of its colour texture on its lower part (the pads and the
  thrusters), black elsewhere.
- `model.ron`: its measures (the middles of its pads as its axles and their
  radii, its grips, pegs and seat), that it hovers and the colour of the
  light its pads cast, and whose rider it carries: it has none of its own,
  and takes that of the bike with wheels it is the hover version of
  (`RIDER`, that bike's folder).

The roster names it as that bike's `hover` (`assets/bikes/roster.ron`:
`hover: Some(Model("NAME"))`); list its files in `assets/pack.txt` too
(`model.ron`, `bike.bin`, `base_color.jpg`, `metallic_roughness.jpg`,
`normal.jpg`, `emissive.jpg`).

Usage: python3 tools/import_hoverbike.py NAME HOVERBIKE.glb RIDER
Needs numpy and Pillow, and tools/import_models.py beside it (its GLB
reader and its pack writer). Prints the measures as it writes them.
"""

import colorsys
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_models import ANKLE_FROM_HIPS, HAND_INSET, HIPS_ABOVE_SEAT, OUT, Glb, save_texture, top_profile, write_mesh_pack  # noqa: E402

# The game's measures (the engine's src/bike/model/mod.rs): how far a bike
# reaches ahead of where it stands and behind it (m).
NOSE = 1.01
TAIL = 0.99
# How high its lowest point floats over the road (m).
CLEARANCE = 0.22
# The pads: what of it lies within this of its lowest point (m), further
# than PAD_FROM ahead of its middle and behind it (m).
PAD_DEPTH = 0.07
PAD_FROM = 0.3
# Its lights: texels this saturated and bright, of a hue from cyan to blue
# (degrees), on the part of it under this share of its height.
LIGHT_HUES = (165.0, 232.0)
LIGHT_SATURATION = 0.45
LIGHT_VALUE = 0.5
LIGHT_BELOW = 0.42
EMISSIVE_SIZE = 1024


def height_map(uv, height, triangles, size):
    """For each texel, how high the surface it paints is (NaN where none)."""
    out = np.full((size, size), np.nan, dtype=np.float32)
    px = uv[:, 0] % 1.0 * size
    py = uv[:, 1] % 1.0 * size
    for a, b, c in triangles:
        xs, ys = px[[a, b, c]], py[[a, b, c]]
        x0, x1 = int(max(xs.min() - 1, 0)), int(min(xs.max() + 1, size - 1))
        y0, y1 = int(max(ys.min() - 1, 0)), int(min(ys.max() + 1, size - 1))
        # A triangle across the texture's seam: left out.
        if x1 - x0 > size / 2 or y1 - y0 > size / 2:
            continue
        out[y0 : y1 + 1, x0 : x1 + 1] = np.nanmin(
            np.stack([out[y0 : y1 + 1, x0 : x1 + 1], np.full((y1 - y0 + 1, x1 - x0 + 1), height[[a, b, c]].mean(), dtype=np.float32)]),
            axis=0,
        )
    return out


def lights(color, heights, low, high):
    """The model's lights, cut out of its colour texture (see LIGHT_*)."""
    rgb = np.asarray(color.convert("RGB").resize((EMISSIVE_SIZE, EMISSIVE_SIZE), Image.LANCZOS), dtype=np.float32) / 255.0
    top, bottom = rgb.max(2), rgb.min(2)
    saturation = np.where(top > 0, (top - bottom) / np.maximum(top, 1e-6), 0.0)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    spread = np.maximum(top - bottom, 1e-6)
    hue = np.where(top == r, (g - b) / spread % 6, np.where(top == g, (b - r) / spread + 2, (r - g) / spread + 4)) * 60.0
    lit = (hue > LIGHT_HUES[0]) & (hue < LIGHT_HUES[1]) & (saturation > LIGHT_SATURATION) & (top > LIGHT_VALUE)
    lit &= heights < low + LIGHT_BELOW * (high - low)
    mask = Image.fromarray((lit * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(1.2))
    glowing = rgb * (np.asarray(mask, dtype=np.float32) / 255.0)[..., None]
    mean = rgb[lit].mean(0) if lit.any() else np.array([0.3, 0.8, 1.0])
    return Image.fromarray((glowing * 255).astype(np.uint8)), mean, lit.mean()


def import_hoverbike(path, out):
    glb = Glb(path)
    attributes, triangles, _ = glb.primitive()
    model, nor, uv = attributes["POSITION"], attributes["NORMAL"], attributes["TEXCOORD_0"]
    # In the model, ahead is -X, up +Y and the left +Z; in the game, ahead is
    # -Z and the right +X.
    scale = (NOSE + TAIL) / (model[:, 0].max() - model[:, 0].min())
    turn = lambda v: np.stack([-v[:, 2], v[:, 1], v[:, 0]], axis=1)
    pos = turn(model) * scale
    pos[:, 2] += -NOSE - pos[:, 2].min()
    pos[:, 1] += CLEARANCE - pos[:, 1].min()
    pos[:, 0] -= (pos[:, 0].max() + pos[:, 0].min()) / 2
    part = {"name": "chassis", "pos": pos, "nor": turn(nor), "uv": uv, "idx": triangles.astype(np.uint32)}
    write_mesh_pack(out / "bike.bin", [part])
    print(f"  chassis: {len(triangles)} triangles, {np.round(pos.min(0), 3).tolist()} to {np.round(pos.max(0), 3).tolist()}")

    pbr = glb.json["materials"][0]["pbrMetallicRoughness"]
    color = glb.image(pbr["baseColorTexture"]["index"])
    save_texture(color, out / "base_color.jpg", 2048)
    save_texture(glb.image(pbr["metallicRoughnessTexture"]["index"]), out / "metallic_roughness.jpg", 1024)
    Image.new("RGB", (16, 16), (128, 128, 255)).save(out / "normal.jpg", quality=95)
    heights = height_map(uv, pos[:, 1], triangles, EMISSIVE_SIZE)
    # Images have their first row at the top, as the texture coordinates do.
    emissive, glow, share = lights(color, heights, pos[:, 1].min(), pos[:, 1].max())
    emissive.save(out / "emissive.jpg", quality=88)
    hue, _, _ = colorsys.rgb_to_hsv(*glow)
    glow = colorsys.hsv_to_rgb(hue, 0.75, 1.0)
    print(f"  lights: {100 * share:.1f} % of the texture, glow {np.round(glow, 2).tolist()}")

    # The pads: what lies lowest, ahead of and behind the middle.
    low = pos[pos[:, 1] < CLEARANCE + PAD_DEPTH]
    pads = []
    for side in (low[low[:, 2] < -PAD_FROM], low[low[:, 2] > PAD_FROM]):
        lo, hi = side.min(0), side.max(0)
        pads.append((np.array([0.0, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2]), float(np.clip(max(hi[2] - lo[2], hi[0] - lo[0]) / 2, 0.22, 0.3))))
    (front, front_radius), (rear, rear_radius) = pads

    # The grips: the ends of the bars, the furthest out of its upper half.
    upper = pos[(pos[:, 1] > pos[:, 1].mean()) & (pos[:, 2] < 0)]
    end = upper[:, 0].max()
    bars = upper[(upper[:, 0] > end - 0.1) & (upper[:, 0] < end - 0.02)]
    grip = bars.mean(0) - [HAND_INSET, 0, 0]
    head = np.array([0.0, grip[1] + 0.06, grip[2] - 0.05])
    # The seat: the lowest of its top between the tank and the tail.
    zs = np.arange(0.0, 0.5, 0.025)
    tops = top_profile(pos, part["idx"], zs)
    lowest = tops.min()
    seat = np.array([0.0, lowest + HIPS_ABOVE_SEAT, zs[np.argmax(tops <= lowest + 0.01)]])
    peg = seat + ANKLE_FROM_HIPS
    return {
        "front_axle": front,
        "rear_axle": rear,
        "front_radius": front_radius,
        "rear_radius": rear_radius,
        "steering_head": head,
        "grip": grip,
        "peg": peg,
        "seat": seat,
        "glow": np.array(glow),
    }


def write_measures(path, name, rider, measures):
    point = lambda v: "(" + ", ".join(f"{x:.4g}" for x in v) + ")"
    path.write_text(
        f"""// The hover version of `{rider}`, by its folder: `{name}`.
// Made with Meshy and prepared by tools/import_hoverbike.py, which measured
// it: the middles of its pads (its axles) and how big they are, and where
// its grips, pegs and seat are (m, in the bike's space); it hovers, its pads
// casting a light of the colour of `glow` on the road; its lights are in
// emissive.jpg; and its rider is `{rider}`'s.
(
    geometry: (
        front_axle: {point(measures["front_axle"])},
        rear_axle: {point(measures["rear_axle"])},
        front_radius: {measures["front_radius"]:.4g},
        rear_radius: {measures["rear_radius"]:.4g},
        steering_head: {point(measures["steering_head"])},
        grip: {point(measures["grip"])},
        peg: {point(measures["peg"])},
        seat: {point(measures["seat"])},
    ),
    steers: false,
    hover: Some((glow: {point(measures["glow"])})),
    rider: Some("{rider}"),
    emissive: true,
)
"""
    )


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__.split("Usage: ")[1].split("\n")[0])
    name, glb, rider = sys.argv[1:]
    if not (OUT / rider / "pilot.bin").exists():
        sys.exit(f"{rider}: no bike with a rider in assets/models/")
    out = OUT / name
    out.mkdir(parents=True, exist_ok=True)
    print(f"{name}:")
    measures = import_hoverbike(glb, out)
    write_measures(out / "model.ron", name, rider, measures)
    for key, value in measures.items():
        print(f"  {key}: {np.round(value, 4).tolist()}")


if __name__ == "__main__":
    main()
