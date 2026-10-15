#!/usr/bin/env python3
"""Prepares the props modelled with Meshy for the game: buildings,
landmarks, statues, lanterns and gates.

Reads one GLB per prop (a single textured mesh, normalised to a unit box)
and writes to `assets/models/props/` what the game loads
(`src/world/japan.rs`):

- `props.bin`: every prop as a piece named after it, centred on the origin
  with its lowest point on the ground, up along +Y, at the game's size
  (`SIZES`). Same format as the trees' and bikes' packs (see
  `tools/import_trees.py`).
- `color.jpg`: their colour textures together, each brought down to `CELL`
  pixels square, in a grid of `COLUMNS`, the texture coordinates of each
  prop moved into its cell: one texture and one material for every prop.
  The normal and metal-roughness maps are left out.

Some models need mending as they are read:

- The great torii (`o_torii`) came standing on a slab of blue water: the
  triangles of its texture's blue round its feet are cut away (the game
  has its own sea).
- The gates over the road (`torii_wide`, `torii_road`) came stout: their
  pillars are made slimmer and the gates shallower (`SLIM`), so that one
  after another they read as gates, not as a wall.
- The water wheel (`water_wheel`) came in one piece with its axle and the
  post that holds it: it is split in two, `water_wheel`, the wheel and its
  axle, and `water_wheel_frame`, the post, both with their origin at the
  middle of the axle (along Z), so that the wheel can turn about it; the
  post reaches down to the ground, as far below its origin as the axle is
  high.
- The great maple (`great_maple`) came in reds so saturated they glared:
  its texture is toned down and a little lightened (`TONED`).

Every prop faces +Z as Meshy made it, with its width along X: a gate spans
X. Usage: python3 tools/import_props.py NAME=PROP.glb [NAME=PROP.glb ...]
with the names of `SIZES`. Needs numpy and Pillow.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageEnhance

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_trees import read_glb  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "props"

# How big each prop is in the game, by its height (m), its length (m, the
# longest way across, for a boat), or for a gate how far from its middle
# the feet of its pillars stand, inside (m, where they flare inwards: what
# keeps clear of the road), and its height (m, squashed or stretched to it;
# `None` keeps its proportions).
SIZES = {
    # Landmarks.
    "o_torii": ("height", 16.0),
    "castle_keep": ("height", 30.0),
    "temple_hall": ("height", 14.0),
    "stilt_shrine": ("height", 11.0),
    "ancient_maple": ("height", 14.0),
    # Centred on the foot of its trunk: its crown spreads far to one side
    # (along -X).
    "great_maple": ("height", 16.0, "foot"),
    "pagoda": ("height", 30.0),
    # Gates over the road: the wide one spans the road between its walls
    # (11.5 m either side of the middle), the narrow one the corridor of
    # torii (6.5 m), the black feet of their pillars, which flare inwards,
    # just outside the walls.
    "torii_wide": ("opening", 11.63, None),
    "torii_road": ("opening", 6.6, 16.0),

    # Things by the road.
    "log_torii": ("height", 6.5),
    "hokora": ("height", 3.0),
    "jizo": ("height", 1.1),
    "komainu": ("height", 1.8),
    "bronze_lantern": ("height", 3.2),
    "stone_lantern": ("height", 2.6),
    "stele": ("height", 3.5),
    "brazier": ("height", 1.8),
    "supply_pile": ("height", 2.4),
    "fishing_boat": ("length", 9.0),
    "minka": ("height", 8.0),
    "water_wheel": ("height", 5.5),
}
# Gates made slimmer than Meshy made them: their pillars this many times as
# thick (about their inner faces, which stay where they are) and the whole
# gate this many times as deep, along the road; Meshy's are stout, and one
# after another they would close into a wall.
SLIM = {
    "torii_wide": (0.6, 0.6),
    "torii_road": (0.45, 0.5),
}
# How saturated and how bright each texture is left (1 as it came): the
# great maple's reds came so saturated they glared, and toned down they
# need a little more light.
TONED = {"great_maple": (0.8, 1.12)}
CELL = 512
COLUMNS = 5
MARGIN = 2


def texels(uvs, color):
    """The colour (0..1) of the texture at each texture coordinate."""
    pixels = np.asarray(color.resize((512, 512))).astype(np.float64) / 255.0
    at = (np.clip(uvs, 0.0, 0.999) * 512).astype(int)
    return pixels[at[:, 1], at[:, 0]]


def keep_triangles(indices, keep):
    """The triangles of `indices` (a flat list) for which `keep` is true."""
    return indices.reshape(-1, 3)[keep].reshape(-1)


def without_water(positions, uvs, indices, color):
    """The great torii without the slab of water it stands on: the blue
    triangles in the lowest fifth of it, round its feet (its top beams are
    black, or a blue too dark to count)."""
    triangles = indices.reshape(-1, 3)
    rgb = texels(uvs[triangles].mean(axis=1), color)
    r, g, b = rgb[:, 0], rgb[:, 1], rgb[:, 2]
    blue = (b > r + 0.12) & (b > g * 0.95)
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    low = positions[triangles][:, :, 1].mean(axis=1) < bottom + 0.2 * (top - bottom)
    return keep_triangles(indices, ~(blue & low))


def wheel_and_frame(positions, indices):
    """The water wheel split in two (indices of each) and the middle of its
    axle (x, y): the post that holds it stands behind the wheel (towards
    -Z), the wheel and its axle are the rest."""
    triangles = indices.reshape(-1, 3)
    middle = positions[triangles].mean(axis=1)
    frame = middle[:, 2] < -0.15
    # The middle of the wheel's rim: the centre of the box round the
    # vertices of the wheel itself (its face, between z 0 and 0.25).
    wheel = positions[np.unique(triangles[~frame])]
    face = wheel[(wheel[:, 2] > 0.0) & (wheel[:, 2] < 0.26)]
    axle = (face[:, :2].min(axis=0) + face[:, :2].max(axis=0)) / 2.0
    return keep_triangles(indices, ~frame), keep_triangles(indices, frame), axle


def pillars(positions):
    """The inner faces of the two pillars of a gate (x, in its own units):
    where they stand, between a sixth and a half of its height up."""
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    height = top - bottom
    band = positions[(positions[:, 1] > bottom + 0.15 * height) & (positions[:, 1] < bottom + 0.45 * height)]
    middle = (positions[:, 0].min() + positions[:, 0].max()) / 2.0
    left, right = band[band[:, 0] < middle], band[band[:, 0] >= middle]
    return left[:, 0].max(), right[:, 0].min()


def slim(positions, normals, pillar, depth):
    """A gate with slimmer pillars, `pillar` times as thick about their
    inner faces (what lies past them drawn in to match: the ends of the
    beams, the outer side of the feet), and `depth` times as deep (along
    Z); its normals turned to match."""
    left, right = pillars(positions)
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    height = top - bottom
    band = positions[(positions[:, 1] > bottom + 0.15 * height) & (positions[:, 1] < bottom + 0.45 * height)]
    outer_left = band[band[:, 0] < left + 1e-6][:, 0].min()
    outer_right = band[band[:, 0] > right - 1e-6][:, 0].max()
    x = positions[:, 0]
    squeezed = x.copy()
    stretch = np.ones(len(x))
    # Right pillar, from its inner face out; and past it.
    inside = (x > right) & (x <= outer_right)
    squeezed[inside] = right + (x[inside] - right) * pillar
    stretch[inside] = pillar
    past = x > outer_right
    squeezed[past] = x[past] - (1.0 - pillar) * (outer_right - right)
    # The left one, mirrored.
    inside = (x < left) & (x >= outer_left)
    squeezed[inside] = left + (x[inside] - left) * pillar
    stretch[inside] = pillar
    past = x < outer_left
    squeezed[past] = x[past] + (1.0 - pillar) * (left - outer_left)
    placed = positions.copy()
    placed[:, 0] = squeezed
    placed[:, 2] *= depth
    turned = normals.copy()
    turned[:, 0] /= stretch
    turned[:, 2] /= depth
    turned /= np.linalg.norm(turned, axis=1, keepdims=True).clip(1e-9)
    return placed, turned


def place(positions, normals, size):
    """Scaled to its size, centred on the origin, its lowest point on the
    ground; its normals turned to match a squash."""
    low, high = positions.min(axis=0), positions.max(axis=0)
    kind = size[0]
    if kind == "height":
        scale = np.full(3, size[1] / (high[1] - low[1]))
    elif kind == "length":
        scale = np.full(3, size[1] / max(high[0] - low[0], high[2] - low[2]))
    else:
        # A gate centred between its pillars (Meshy's are not quite
        # symmetric), the nearest of their feet below its tie beam as far
        # from the middle as asked.
        left, right = pillars(positions)
        middle = (left + right) / 2.0
        below = positions[positions[:, 1] < low[1] + 0.6 * (high[1] - low[1])]
        feet = np.abs(below[:, 0] - middle).min()
        scale = np.full(3, size[1] / feet)
        if size[2] is not None:
            scale[1] = size[2] / (high[1] - low[1])
    centre = np.array([(low[0] + high[0]) / 2.0, low[1], (low[2] + high[2]) / 2.0])
    if kind == "opening":
        centre[0] = middle
    if kind == "height" and len(size) > 2 and size[2] == "foot":
        # On the middle of its lowest vertices: the foot of its trunk.
        foot = positions[positions[:, 1] < low[1] + 0.03 * (high[1] - low[1])]
        centre[0], centre[2] = foot[:, 0].mean(), foot[:, 2].mean()
    placed = (positions - centre) * scale
    turned = normals / scale
    turned /= np.linalg.norm(turned, axis=1, keepdims=True).clip(1e-9)
    return placed, turned, centre, scale


def compact(positions, normals, uvs, indices):
    """Only the vertices the triangles use, renumbered."""
    used, renumbered = np.unique(indices, return_inverse=True)
    return positions[used], normals[used], uvs[used], renumbered.astype(np.uint32)


def main():
    props = []
    for argument in sys.argv[1:]:
        name, path = argument.split("=", 1)
        if name not in SIZES:
            sys.exit(f"{name}: not one of {', '.join(SIZES)}")
        props.append((name, path))
    if not props:
        sys.exit(__doc__)
    props.sort(key=lambda prop: list(SIZES).index(prop[0]))
    rows = (len(props) + COLUMNS - 1) // COLUMNS
    atlas = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    size = np.array([COLUMNS * CELL, rows * CELL], dtype=np.float64)
    pieces = []
    for slot, (name, path) in enumerate(props):
        positions, normals, uvs, indices, color = read_glb(path)
        column, row = slot % COLUMNS, slot // COLUMNS
        inner = CELL - 2 * MARGIN
        texture = color.resize((inner, inner), Image.LANCZOS)
        if name in TONED:
            saturation, brightness = TONED[name]
            texture = ImageEnhance.Color(texture).enhance(saturation)
            texture = ImageEnhance.Brightness(texture).enhance(brightness)
        cell = texture.resize((CELL, CELL), Image.NEAREST)
        cell.paste(texture, (MARGIN, MARGIN))
        atlas.paste(cell, (column * CELL, row * CELL))
        corner = np.array([column * CELL + MARGIN, row * CELL + MARGIN], dtype=np.float64)
        moved = (corner + uvs.clip(0.0, 1.0) * inner) / size

        if name == "o_torii":
            indices = without_water(positions, uvs, indices, color)
        if name in SLIM:
            positions, normals = slim(positions, normals, *SLIM[name])
        parts = [(name, indices)]
        axle = None
        if name == "water_wheel":
            wheel, frame, axle = wheel_and_frame(positions, indices)
            parts = [("water_wheel", wheel), ("water_wheel_frame", frame)]
        # Scaled and placed as a whole (cut first, so that what was cut
        # does not count), then the parts.
        used = np.unique(np.concatenate([part for _, part in parts]))
        _, _, centre, scale = place(positions[used], normals[used], SIZES[name])
        placed = (positions - centre) * scale
        turned = normals / scale
        turned /= np.linalg.norm(turned, axis=1, keepdims=True).clip(1e-9)
        for piece, part in parts:
            p, n, u, i = compact(placed, turned, moved, part)
            if axle is not None:
                # Both about the middle of the axle.
                pivot = (np.array([axle[0], axle[1], 0.0]) - centre) * scale
                pivot[2] = 0.0
                p = p - pivot
            pieces.append((piece, p, n, u, i))
            low, high = p.min(axis=0), p.max(axis=0)
            print(
                f"{piece}: {len(p)} vertices, {len(i) // 3} triangles, "
                f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} m"
            )

    OUT.mkdir(parents=True, exist_ok=True)
    with open(OUT / "props.bin", "wb") as out:
        out.write(b"MRMESH01")
        out.write(len(pieces).to_bytes(4, "little"))
        for name, positions, normals, uvs, indices in pieces:
            encoded = name.encode()
            out.write(len(encoded).to_bytes(4, "little") + encoded)
            for count in (len(positions), len(indices), 0):
                out.write(int(count).to_bytes(4, "little"))
            out.write(positions.astype("<f4").tobytes())
            out.write(normals.astype("<f4").tobytes())
            out.write(uvs.astype("<f4").tobytes())
            out.write(indices.astype("<u4").tobytes())
    atlas.save(OUT / "color.jpg", quality=88, optimize=True)
    print(f"{OUT / 'props.bin'}, {OUT / 'color.jpg'} ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
