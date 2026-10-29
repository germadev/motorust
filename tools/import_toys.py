#!/usr/bin/env python3
"""Prepares the toys modelled with Meshy for the toy store (Juguetería):
castles of bricks, a pirate ship, trains, plush toys, vehicles...

Reads one GLB per toy (a single textured mesh, normalised to a unit box)
and writes what the game loads (`src/world/toys.rs`) into the pack the toy
belongs to: `assets/models/toys/` for those in `FIRST_PACK`, whose atlas is
full, `assets/models/toys2/` for the rest. In each:

- `props.bin`: every toy as a piece named after it, centred on the origin
  with its lowest point on the ground, up along +Y, facing +Z (those Meshy
  made facing -X, `TURNED`, turned a quarter turn), at the game's size
  (`SIZES`: by its height, or by its length, the longest way across, for a
  ship, a train or a plane). Same format as the other packs (see
  `tools/import_props.py`).
- `color.jpg`: their colour textures together, each brought down to its
  cell (`CELLS`: 1024 pixels square for the big toys the riders pass close
  by, 512 for the small ones), with a 2-pixel margin, the texture
  coordinates of each moved into it, in an atlas `WIDTH` wide: one texture
  and one material for every toy of the pack. The normal and
  metal-roughness maps are left out.

A toy that changes cell size has to be given again (its GLB), as its cell
of the old atlas is cut out at its size in `CELLS`.

The models come one by one: with `--keep`, the toys already in the packs
written stay as they are (their meshes, and their cells of the texture moved
to the new atlas), and only those given are added or replaced, so the GLBs
of the others need not be at hand; only the packs of the toys given are
written.

Usage: python3 tools/import_toys.py [--keep] NAME=TOY.glb [NAME=TOY.glb ...]
with the names of `SIZES`. Needs numpy and Pillow.
"""

import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_props import compact, place  # noqa: E402
from import_trees import read_glb  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "assets" / "models"

# The toys in the first pack (`toys/`), whose atlas is full, 4096 square;
# all others go into the second (`toys2/`).
FIRST_PACK = {
    "castle_big",
    "castle_small",
    "pirate_ship",
    "giraffe_plush",
    "dollhouse",
    "locomotive",
    "coach",
    "freight_wagon",
    "teddy_bear",
    "tin_robot",
    "rubber_duck",
    "spinning_top",
    "fire_truck",
    "mech_pegasus",
    "croc_car",
    "garage_playset",
    "train_station",
    "tunnel_mountain",
    "rocking_horse",
    "nutcracker",
    "biplane",
    "helicopter",
}


def pack_of(name):
    """The folder of the pack toy `name` belongs to."""
    return MODELS / ("toys" if name in FIRST_PACK else "toys2")

# How big each toy is in the game (m): by its height, or by its length (the
# longest way across). The bikes are about 2 m long.
SIZES = {
    # Close by the road.
    "castle_big": ("height", 22.0),
    "castle_small": ("height", 18.0),
    "pirate_ship": ("length", 30.0),
    "giraffe_plush": ("height", 30.0),
    "dollhouse": ("height", 30.0),
    # The trains.
    "locomotive": ("length", 7.0),
    # Longer than the others, as a coach is: as tall and as wide then.
    "coach": ("length", 10.5),
    "freight_wagon": ("length", 7.0),
    # Standing about.
    "teddy_bear": ("height", 15.0),
    "tin_robot": ("height", 14.0),
    "rubber_duck": ("height", 7.0),
    "spinning_top": ("height", 6.0),
    "rocking_horse": ("length", 18.0),
    "nutcracker": ("height", 15.0),
    "fire_truck": ("length", 25.0),
    "mech_pegasus": ("height", 16.0),
    "croc_car": ("length", 16.0),
    "dump_truck": ("length", 20.0),
    "garage_playset": ("height", 20.0),
    "train_station": ("length", 25.0),
    # Its tunnel as wide as a train's and more: 4.4 m.
    "tunnel_mountain": ("length", 40.0),
    "teepee": ("height", 25.0),
    # As big as it takes for its head, over the road, to be well over the
    # bikes.
    "trex": ("length", 28.0),
    "rocket": ("height", 25.0),
    "toy_chest": ("length", 16.0),
    "jack_in_box": ("height", 10.0),
    "xylophone": ("length", 12.0),
    # Hanging from the ceiling.
    "biplane": ("length", 16.0),
    "helicopter": ("length", 14.0),
    # The shop.
    "ball_bin": ("length", 12.0),
    "checkout_counter": ("length", 30.0),
    "shopping_cart": ("height", 12.0),
    "basket_stack": ("height", 6.0),
    "potted_plant": ("height", 20.0),
    "kids_table": ("length", 16.0),
}
# Those Meshy made facing -X (a ship's bow, a locomotive's chimney, a
# plane's nose, a truck's cab), turned a quarter turn to face +Z as the
# others.
TURNED = {
    "pirate_ship",
    "locomotive",
    "coach",
    "freight_wagon",
    "biplane",
    "helicopter",
    "fire_truck",
    "croc_car",
    "dump_truck",
}
# Those a train runs through, centred on their tunnels (along z) rather
# than on their boxes.
TUNNELS = {"tunnel_mountain"}
# The cells of the big toys the riders pass close by (px); the others' are
# `CELL`.
CELLS = {
    "castle_big": 1024,
    "castle_small": 1024,
    "pirate_ship": 1024,
    "giraffe_plush": 1024,
    "dollhouse": 1024,
    "teddy_bear": 1024,
    "tin_robot": 1024,
    "biplane": 1024,
    "helicopter": 1024,
    "fire_truck": 1024,
    "mech_pegasus": 1024,
    "garage_playset": 1024,
    "train_station": 1024,
    "nutcracker": 1024,
    "dump_truck": 1024,
    "teepee": 1024,
    "trex": 1024,
    "rocket": 1024,
    "toy_chest": 1024,
}
CELL = 512
# How wide the atlas is (px): four of the big cells, or eight of the others.
WIDTH = 4096
MARGIN = 2


def read_pack(path):
    """The pieces of a pack (name, positions, normals, uvs, indices)."""
    data = path.read_bytes()
    assert data[:8] == b"MRMESH01", f"{path}: not a pack of meshes"
    (count,) = struct.unpack_from("<I", data, 8)
    offset, pieces = 12, []
    for _ in range(count):
        (length,) = struct.unpack_from("<I", data, offset)
        name = data[offset + 4 : offset + 4 + length].decode()
        offset += 4 + length
        vertices, indices, _ = struct.unpack_from("<III", data, offset)
        offset += 12

        def take(count, kind, width):
            nonlocal offset
            values = np.frombuffer(data, dtype=kind, count=count * width, offset=offset)
            offset += values.nbytes
            return values.reshape(-1, width).astype(np.float64) if width > 1 else values.copy()

        positions = take(vertices, "<f4", 3)
        normals = take(vertices, "<f4", 3)
        uvs = take(vertices, "<f4", 2)
        triangles = take(indices, "<u4", 1).astype(np.uint32)
        pieces.append((name, positions, normals, uvs, triangles))
    return pieces


def tunnel_middle(positions, indices, height=1.0, step=0.05):
    """Where the middle of a tunnel along z through a model is across it
    (x, m): of the widest stretch across at `height` through which a line
    along z meets none of its triangles."""
    triangles = positions[indices.reshape(-1, 3)][:, :, :2]
    a, b, c = triangles[:, 0], triangles[:, 1], triangles[:, 2]

    def blocked(x):
        point = np.array([x, height])

        def side(p, q):
            return (p[:, 0] - point[0]) * (q[:, 1] - point[1]) - (q[:, 0] - point[0]) * (p[:, 1] - point[1])

        d = np.stack([side(a, b), side(b, c), side(c, a)])
        return bool((~((d < 0).any(axis=0) & (d > 0).any(axis=0))).any())

    xs = np.arange(positions[:, 0].min(), positions[:, 0].max(), step)
    clear = [not blocked(x) for x in xs]
    best, start = (0, 0), None
    for k, open_ in enumerate(clear + [False]):
        if open_ and start is None:
            start = k
        elif not open_ and start is not None:
            if k - start > best[1] - best[0]:
                best = (start, k)
            start = None
    assert best[1] > best[0], "no tunnel through it"
    return (xs[best[0]] + xs[best[1] - 1]) / 2.0


def quarter_turn(vectors):
    """`vectors` turned a quarter turn about +Y, -X to +Z."""
    x, y, z = vectors[:, 0], vectors[:, 1], vectors[:, 2]
    return np.stack([z, y, -x], axis=1)


def cells(sizes):
    """Where each cell of `sizes` (px, square) goes in an atlas `WIDTH`
    wide: the big ones first, row by row, then the small ones four to each
    big cell left; and how tall the atlas is."""
    big = max(CELLS.values())
    order = sorted(range(len(sizes)), key=lambda k: -sizes[k])
    corners = [None] * len(sizes)
    slot, quarter = 0, 0
    per_row = WIDTH // big
    for k in order:
        if sizes[k] == big:
            corners[k] = ((slot % per_row) * big, (slot // per_row) * big)
            slot += 1
        else:
            if quarter == 0:
                base = ((slot % per_row) * big, (slot // per_row) * big)
                slot += 1
            corners[k] = (base[0] + (quarter % 2) * CELL, base[1] + (quarter // 2) * CELL)
            quarter = (quarter + 1) % 4
    rows = (slot + per_row - 1) // per_row
    return corners, max(rows, 1) * big


def main():
    keep = "--keep" in sys.argv[1:]
    given = []
    for argument in sys.argv[1:]:
        if argument == "--keep":
            continue
        name, path = argument.split("=", 1)
        if name not in SIZES:
            sys.exit(f"{name}: not one of {', '.join(SIZES)}")
        given.append((name, path))
    if not given:
        sys.exit(__doc__)
    for out in sorted({pack_of(name) for name, _ in given}):
        write_pack(out, [(name, path) for name, path in given if pack_of(name) == out], keep)


def write_pack(out, given, keep):
    """The pack in folder `out` with the toys `given` (name, GLB), and the
    others already in it if `keep`."""
    # Each toy: its mesh at the game's size, and its texture (whole).
    toys = {}
    if keep and (out / "props.bin").exists():
        atlas = Image.open(out / "color.jpg").convert("RGB")
        for name, positions, normals, uvs, indices in read_pack(out / "props.bin"):
            # Its cell of the old atlas, from where its texture coordinates
            # reach, out to the margin round it.
            size = np.array(atlas.size, dtype=np.float64)
            low = np.floor(uvs.min(axis=0) * size) - MARGIN
            side = CELLS.get(name, CELL)
            corner = (low / side).round() * side
            texture = atlas.crop((*corner.astype(int), *(corner + side).astype(int)))
            inner = side - 2 * MARGIN
            local = (uvs * size - corner - MARGIN) / inner
            toys[name] = (positions, normals, local.clip(0.0, 1.0), indices, texture.crop(
                (MARGIN, MARGIN, side - MARGIN, side - MARGIN)
            ))
    for name, path in given:
        positions, normals, uvs, indices, color = read_glb(path)
        if name in TURNED:
            positions, normals = quarter_turn(positions), quarter_turn(normals)
        placed, turned, _, _ = place(positions, normals, SIZES[name])
        if name in TUNNELS:
            # Its tunnel on its middle, rather than its box.
            placed[:, 0] -= tunnel_middle(placed, indices)
        toys[name] = (placed, turned, uvs, indices, color)

    names = sorted(toys, key=lambda name: list(SIZES).index(name))
    sizes = [CELLS.get(name, CELL) for name in names]
    corners, height = cells(sizes)
    atlas = Image.new("RGB", (WIDTH, height))
    scale = np.array([WIDTH, height], dtype=np.float64)
    pieces = []
    for name, side, corner in zip(names, sizes, corners):
        positions, normals, uvs, indices, color = toys[name]
        inner = side - 2 * MARGIN
        texture = color.resize((inner, inner), Image.LANCZOS)
        cell = texture.resize((side, side), Image.NEAREST)
        cell.paste(texture, (MARGIN, MARGIN))
        atlas.paste(cell, corner)
        moved = (np.array(corner, dtype=np.float64) + MARGIN + uvs.clip(0.0, 1.0) * inner) / scale
        p, n, u, i = compact(positions, normals, moved, indices)
        pieces.append((name, p, n, u, i))
        low, high = p.min(axis=0), p.max(axis=0)
        print(
            f"{name}: {len(p)} vertices, {len(i) // 3} triangles, "
            f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} m, "
            f"{side} px"
        )

    out.mkdir(parents=True, exist_ok=True)
    with open(out / "props.bin", "wb") as written:
        written.write(b"MRMESH01")
        written.write(len(pieces).to_bytes(4, "little"))
        for name, positions, normals, uvs, indices in pieces:
            encoded = name.encode()
            written.write(len(encoded).to_bytes(4, "little") + encoded)
            for count in (len(positions), len(indices), 0):
                written.write(int(count).to_bytes(4, "little"))
            written.write(positions.astype("<f4").tobytes())
            written.write(normals.astype("<f4").tobytes())
            written.write(uvs.astype("<f4").tobytes())
            written.write(indices.astype("<u4").tobytes())
    atlas.save(out / "color.jpg", quality=88, optimize=True)
    print(f"{out / 'props.bin'}, {out / 'color.jpg'} ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
