#!/usr/bin/env python3
"""Prepares the trees modelled with Meshy for the game.

Reads one GLB per tree (a single textured mesh, about four hundred
triangles, normalised to a unit box) and writes to `assets/models/trees/`
what the game loads (`src/world/trees.rs`):

- `trees.bin`: every tree as a piece named after it, standing with the foot
  of its trunk at the origin, up along +Y, as tall in metres as the game's
  own trees of that species (`HEIGHTS`). Same format as the bikes' packs
  (see `tools/import_models.py`): b"MRMESH01", u32 number of pieces and, per
  piece, u32 name length, name, u32 vertices, u32 indices, u32 flags (0),
  positions f32x3, normals f32x3, uv f32x2, indices u32.
- `color.jpg`: their colour textures together, each brought down to
  `CELL` pixels square, in a grid of `COLUMNS`, the texture coordinates of
  each tree moved into its cell. One texture and one material for every
  tree, so they all draw alike; the normal and metal-roughness maps are
  left out (trees are seen from afar, their shape is in their normals).

Usage: python3 tools/import_trees.py NAME=TREE.glb [NAME=TREE.glb ...]
with the names of `HEIGHTS`. Needs numpy and Pillow.
"""

import io
import json
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "trees"

# How tall each tree stands (m): as the game's own trees of the species
# (the average of their three variants), and the fir as the forests' trees,
# a little lower.
HEIGHTS = {
    "umbrella_pine": 7.9,
    "cypress": 10.4,
    "olive": 5.0,
    "tall": 16.1,
    "broadleaf": 8.4,
    "maritime_pine": 8.8,
    "poplar": 11.8,
    "almond": 4.8,
    "fir": 14.0,
}
# Each texture's size in the grid (px), how many to a row, and a margin
# (px) kept round each one so that filtering does not reach the next.
CELL = 512
COLUMNS = 3
MARGIN = 2


def read_glb(path):
    """The mesh of a GLB (positions, normals, uv, indices) and its colour
    texture."""
    data = Path(path).read_bytes()
    offset, chunks = 12, []
    while offset < len(data):
        length, _ = struct.unpack_from("<II", data, offset)
        chunks.append(data[offset + 8 : offset + 8 + length])
        offset += 8 + length
    gltf, binary = json.loads(chunks[0]), chunks[1]

    def accessor(index):
        acc = gltf["accessors"][index]
        view = gltf["bufferViews"][acc["bufferView"]]
        width = {"SCALAR": 1, "VEC2": 2, "VEC3": 3}[acc["type"]]
        kind = {5126: np.float32, 5125: np.uint32, 5123: np.uint16}[acc["componentType"]]
        start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
        values = np.frombuffer(binary, dtype=kind, count=acc["count"] * width, offset=start)
        return values.reshape(-1, width) if width > 1 else values

    (mesh,) = gltf["meshes"]
    (primitive,) = mesh["primitives"]
    attributes = primitive["attributes"]
    positions = accessor(attributes["POSITION"]).astype(np.float64)
    normals = accessor(attributes["NORMAL"]).astype(np.float64)
    uvs = accessor(attributes["TEXCOORD_0"]).astype(np.float64)
    indices = accessor(primitive["indices"]).astype(np.uint32)
    material = gltf["materials"][primitive.get("material", 0)]
    texture = material["pbrMetallicRoughness"]["baseColorTexture"]["index"]
    image = gltf["images"][gltf["textures"][texture]["source"]]
    view = gltf["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    color = Image.open(io.BytesIO(binary[start : start + view["byteLength"]])).convert("RGB")
    return positions, normals, uvs, indices, color


def stand(positions, height):
    """The tree standing on the origin: the foot of its trunk (the middle
    of its lowest vertices) there, as tall as `height`."""
    bottom = positions[:, 1].min()
    foot = positions[positions[:, 1] < bottom + 0.03][:, [0, 2]].mean(axis=0)
    scale = height / (positions[:, 1].max() - bottom)
    placed = positions.copy()
    placed[:, 0] -= foot[0]
    placed[:, 2] -= foot[1]
    placed[:, 1] -= bottom
    return placed * scale


def main():
    trees = []
    for argument in sys.argv[1:]:
        name, path = argument.split("=", 1)
        if name not in HEIGHTS:
            sys.exit(f"{name}: not one of {', '.join(HEIGHTS)}")
        trees.append((name, path))
    if not trees:
        sys.exit(__doc__)
    # Always in the same order, that of `HEIGHTS`, the game's.
    trees.sort(key=lambda tree: list(HEIGHTS).index(tree[0]))
    rows = (len(trees) + COLUMNS - 1) // COLUMNS
    atlas = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    pieces = []
    for slot, (name, path) in enumerate(trees):
        positions, normals, uvs, indices, color = read_glb(path)
        positions = stand(positions, HEIGHTS[name])
        column, row = slot % COLUMNS, slot // COLUMNS
        inner = CELL - 2 * MARGIN
        texture = color.resize((inner, inner), Image.LANCZOS)
        # The margin repeats the edge, so that filtering finds the same
        # colours there.
        cell = texture.resize((CELL, CELL), Image.NEAREST)
        cell.paste(texture, (MARGIN, MARGIN))
        atlas.paste(cell, (column * CELL, row * CELL))
        size = np.array([COLUMNS * CELL, rows * CELL], dtype=np.float64)
        corner = np.array([column * CELL + MARGIN, row * CELL + MARGIN], dtype=np.float64)
        moved = (corner + uvs.clip(0.0, 1.0) * inner) / size
        pieces.append((name, positions, normals, moved, indices))
        print(
            f"{name}: {len(positions)} vertices, {len(indices) // 3} triangles, "
            f"{positions[:, 1].max():.1f} m tall, "
            f"{np.hypot(positions[:, 0], positions[:, 2]).max():.1f} m round"
        )

    OUT.mkdir(parents=True, exist_ok=True)
    with open(OUT / "trees.bin", "wb") as out:
        out.write(b"MRMESH01")
        out.write(struct.pack("<I", len(pieces)))
        for name, positions, normals, uvs, indices in pieces:
            encoded = name.encode()
            out.write(struct.pack("<I", len(encoded)) + encoded)
            out.write(struct.pack("<III", len(positions), len(indices), 0))
            out.write(positions.astype("<f4").tobytes())
            out.write(normals.astype("<f4").tobytes())
            out.write(uvs.astype("<f4").tobytes())
            out.write(indices.astype("<u4").tobytes())
    atlas.save(OUT / "color.jpg", quality=88, optimize=True)
    print(f"{OUT / 'trees.bin'}, {OUT / 'color.jpg'} ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
