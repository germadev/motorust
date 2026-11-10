#!/usr/bin/env python3
"""Prepares Costa Bits' props, made with Meshy, for the game: the town's
houses and the beach, the mountain road's rocks, signs, plants, ibex and
viewpoint, the hermitage, the castle on the mesa, the billboards, the
bridge over the gorge (see `docs/costa-bits-props.md`).

Reads one GLB per prop (a single textured mesh, normalised to a unit box,
its front to +Z, up along +Y) and writes to `assets/models/costa/` what the
game loads (`src/game/costa.rs`), as `tools/import_props.py` does for feudal
Japan:

- `props.bin`: every prop as a piece named after it, centred on the origin
  with its lowest point on the ground, up along +Y, at the game's size
  (`SIZES`).
- `color.jpg`: their colour textures together, each brought down to `CELL`
  pixels square, in a grid of `COLUMNS`, the texture coordinates of each
  prop moved into its cell: one texture and one material for every prop.

The bridge over the gorge (`gorge_bridge`) came as the old bridges over the
gorges of Andalusia stand: its great arch narrow, about 8 m across at its
size, and closed at its foot by a wall with a small door. The road runs
through it, so it is mended (`open_bridge`): the wall at the foot of the arch
is cut away, the faces of the piers it hid are closed with the stone of the
piers above them, and the middle of the bridge, between the piers, is
stretched across until the arch is as wide as `SIZES` asks, the piers moved
out to match, the rest as made.

The bend signs (`curve_sign`, `curve_sign_left`) are not Meshy's: they are
built by `tools/make_curve_sign.py`.

Usage: python3 tools/import_costa.py NAME=PROP.glb [NAME=PROP.glb ...] with
the names of `SIZES`. Needs numpy and Pillow.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_props import compact, place
from import_trees import read_glb

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "costa"

# How big each prop is in the game: by its height (m), or its length (m,
# the longest way across, for a boat or a car). The bridge by its height
# and how far from its middle the inner faces of its piers stand once its
# arch is widened (m): clear of the road's walls, which close in to 6.5 m
# under it (`narrows` in `assets/circuits/costa_bits.ron`).
SIZES = {
    # The town and the beach.
    "white_house": ("height", 9.0),
    "chiringuito": ("height", 4.5),
    "espeto_boat": ("length", 5.0),
    "retro_sports_car": ("length", 4.4),
    "watchtower": ("height", 14.0),
    "lifeguard_tower": ("height", 5.0),
    "flag_marshal": ("height", 2.2),
    # The mountain road.
    "curve_sign": ("height", 3.0),
    "curve_sign_left": ("height", 3.0),
    "boulder_a": ("height", 4.0),
    "boulder_b": ("height", 5.0),
    "boulder_c": ("height", 2.6),
    "mirador": ("height", 4.0),
    "km_post": ("height", 1.5),
    "ermita": ("height", 10.0),
    "prickly_pear": ("height", 2.5),
    "agave": ("height", 4.0),
    # Standing on its rock: the ibex itself some 1.5 m high.
    "ibex": ("height", 2.1),
    "camper_van": ("length", 4.5),
    "castle_ruin": ("height", 18.0),
    # The homage to the arcades.
    "arcade_billboard": ("height", 7.0),
    "gorge_bridge": ("bridge", 48.0, 7.4),
}
CELL = 512
COLUMNS = 5
MARGIN = 2


def piers(positions):
    """The inner faces of a bridge's piers (x, in its own units): where they
    stand between three tenths and six tenths of its height up, the arch
    open between them."""
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    height = top - bottom
    band = positions[
        (positions[:, 1] > bottom + 0.3 * height)
        & (positions[:, 1] < bottom + 0.6 * height)
    ]
    middle = (positions[:, 0].min() + positions[:, 0].max()) / 2.0
    left, right = band[band[:, 0] < middle], band[band[:, 0] >= middle]
    return left[:, 0].max(), right[:, 0].min()


def open_bridge(positions, normals, uvs, indices, height, half_opening):
    """The bridge with its arch open to the ground and as wide as asked:
    `half_opening` metres from its middle to each pier once it is `height`
    metres high (see the module's docstring). Returns its positions,
    normals, texture coordinates and indices."""
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    left, right = piers(positions)
    middle, half = (left + right) / 2.0, (right - left) / 2.0
    inset = 0.004

    # The wall at the arch's foot: what of the triangles below halfway up
    # lies between the piers' inner faces. Those wholly between them go;
    # those across a face are cut along it, the part outside kept (Meshy's
    # faces run on from the wall into the piers).
    triangles = indices.reshape(-1, 3)
    corners = positions[triangles]
    low = corners[:, :, 1].mean(axis=1) < bottom + 0.5 * (top - bottom)
    lo_x, hi_x = middle - half, middle + half
    across = (
        low
        & (corners[:, :, 0].max(axis=1) > lo_x + inset)
        & (corners[:, :, 0].min(axis=1) < hi_x - inset)
    )
    sill = corners[across][:, :, 1].max() if across.any() else bottom
    kept = [triangles[~across]]
    extra = {"p": [], "n": [], "u": []}

    def vertex(p, n, u):
        extra["p"].append(p)
        extra["n"].append(n)
        extra["u"].append(u)
        return len(positions) + len(extra["p"]) - 1

    for corner in triangles[across]:
        points = [(positions[k], normals[k], uvs[k]) for k in corner]
        for keep in (lambda x: x <= lo_x, lambda x: x >= hi_x):
            plane = lo_x if keep(lo_x - 1.0) else hi_x
            polygon = []
            for k in range(3):
                a, b = points[k], points[(k + 1) % 3]
                if keep(a[0][0]):
                    polygon.append(a)
                if keep(a[0][0]) != keep(b[0][0]):
                    t = (plane - a[0][0]) / (b[0][0] - a[0][0])
                    polygon.append(tuple(a[j] + (b[j] - a[j]) * t for j in range(3)))
            if len(polygon) < 3:
                continue
            ids = [vertex(*point) for point in polygon]
            kept.append(
                np.array([[ids[0], ids[k], ids[k + 1]] for k in range(1, len(ids) - 1)])
            )
    wall = across
    if extra["p"]:
        positions = np.vstack([positions, np.array(extra["p"])])
        normals = np.vstack([normals, np.array(extra["n"])])
        uvs = np.vstack([uvs, np.array(extra["u"])])
    kept = np.vstack(kept)

    # The piers' inner faces where the wall hid them, from the ground to a
    # little over its top, in the stone of the faces above them: each point
    # of a grid over them takes the texture coordinates of the face's
    # nearest vertex as far above it as the wall was high.
    rise = sill - bottom
    new_positions, new_normals, new_uvs, new_triangles = [], [], [], []
    count = len(positions)
    for side in (-1.0, 1.0):
        x = middle + side * half
        face = np.abs(positions[:, 0] - x) < 0.01
        above = face & (positions[:, 1] > sill) & (positions[:, 1] < sill + rise + 0.05)
        reach = face & (positions[:, 1] < bottom + 0.6 * (top - bottom))
        z0, z1 = positions[reach][:, 2].min(), positions[reach][:, 2].max()
        source = positions[above][:, 1:]
        source_uv = uvs[above]
        rows, columns = 5, 5
        start = count + len(new_positions)
        for i in range(rows + 1):
            y = bottom + (sill + 0.01 - bottom) * i / rows
            for j in range(columns + 1):
                z = z0 + (z1 - z0) * j / columns
                if len(source):
                    nearest = np.argmin(((source - [y + rise, z]) ** 2).sum(axis=1))
                    uv = source_uv[nearest]
                else:
                    uv = uvs[np.argmin(np.abs(positions[:, 0] - x))]
                new_positions.append([x, y, z])
                new_normals.append([-side, 0.0, 0.0])
                new_uvs.append(uv)
        for i in range(rows):
            for j in range(columns):
                a = start + i * (columns + 1) + j
                b, c, d = a + 1, a + columns + 2, a + columns + 1
                # Facing into the arch (towards -side along X).
                quad = [[a, d, c], [a, c, b]] if side > 0 else [[a, b, c], [a, c, d]]
                new_triangles.extend(quad)
    positions = np.vstack([positions, np.array(new_positions)])
    normals = np.vstack([normals, np.array(new_normals)])
    uvs = np.vstack([uvs, np.array(new_uvs)])
    indices = np.concatenate(
        [kept.reshape(-1), np.array(new_triangles, dtype=np.uint32).reshape(-1)]
    )

    # Stretched across between the piers until the arch is as wide as
    # asked, the piers and what lies past them moved out to match.
    scale = height / (top - bottom)
    stretch = half_opening / (half * scale)
    x = positions[:, 0] - middle
    within = np.abs(x) <= half + 1e-6
    widened = np.where(within, x * stretch, x + np.sign(x) * half * (stretch - 1.0))
    positions = positions.copy()
    positions[:, 0] = middle + widened
    normals = normals.copy()
    normals[within, 0] /= stretch
    normals /= np.linalg.norm(normals, axis=1, keepdims=True).clip(1e-9)
    print(
        f"gorge_bridge: the wall at its foot cut ({wall.sum()} triangles), its arch stretched {stretch:.2f} times"
    )
    return positions, normals, uvs, indices.astype(np.uint32)


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
        cell = texture.resize((CELL, CELL), Image.NEAREST)
        cell.paste(texture, (MARGIN, MARGIN))
        atlas.paste(cell, (column * CELL, row * CELL))
        corner = np.array(
            [column * CELL + MARGIN, row * CELL + MARGIN], dtype=np.float64
        )

        sized = SIZES[name]
        if sized[0] == "bridge":
            positions, normals, uvs, indices = open_bridge(
                positions, normals, uvs, indices, *sized[1:]
            )
            sized = ("height", sized[1])
        moved = (corner + uvs.clip(0.0, 1.0) * inner) / size
        placed, turned, _, _ = place(positions, normals, sized)
        p, n, u, i = compact(placed, turned, moved, indices)
        pieces.append((name, p, n, u, i))
        low, high = p.min(axis=0), p.max(axis=0)
        print(
            f"{name}: {len(p)} vertices, {len(i) // 3} triangles, "
            f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} m"
        )

    OUT.mkdir(parents=True, exist_ok=True)
    with open(OUT / "props.bin", "wb") as out:
        out.write(b"MRMESH01")
        out.write(len(pieces).to_bytes(4, "little"))
        for name, positions, normals, uvs, indices in pieces:
            encoded = name.encode()
            out.write(len(encoded).to_bytes(4, "little") + encoded)
            out.writelines(
                int(count).to_bytes(4, "little")
                for count in (len(positions), len(indices), 0)
            )
            out.write(positions.astype("<f4").tobytes())
            out.write(normals.astype("<f4").tobytes())
            out.write(uvs.astype("<f4").tobytes())
            out.write(indices.astype("<u4").tobytes())
    atlas.save(OUT / "color.jpg", quality=88, optimize=True)
    print(f"{OUT / 'props.bin'}, {OUT / 'color.jpg'} ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
