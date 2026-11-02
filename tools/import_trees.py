#!/usr/bin/env python3
"""Prepares the trees modelled with Meshy for the game.

Reads one GLB per tree (a single textured mesh, about four hundred
triangles, normalised to a unit box) and writes to `assets/models/trees/`
what the game loads (`motorust-engine/src/world/trees.rs`):

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

Usage: python3 tools/import_trees.py [--keep] NAME=TREE.glb [NAME=TREE.glb ...]
with the names of `HEIGHTS`. With `--keep`, the trees already in the pack
stay as they are (their meshes, and their cells of the texture, moved to
the new grid) and those given are added or replaced: the GLBs of the trees
already in it need not be at hand. Needs numpy and Pillow.

Some models need mending as they are read (`MENDS`): Meshy's black pine is
nearly flat, a quarter as deep as it is wide, so its foliage (its green
texels, above its roots) is spread out in depth, round its trunk, which
keeps its shape.
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
    # The trees of Ghost Island (feudal Japan).
    "momiji": 7.0,
    "ginkgo": 13.0,
    "sakura": 7.5,
    "black_pine": 8.0,
    "sugi": 16.0,
    "bamboo": 11.0,
}
# How much deeper (along Z) each tree's foliage is made than it came: the
# black pine came nearly flat.
MENDS = {"black_pine": 3.0}
# Each texture's size in the grid (px), how many to a row, and a margin
# (px) kept round each one so that filtering does not reach the next.
CELL = 512
COLUMNS = 4
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


def foliage(positions, uvs, color):
    """Which vertices are leaves: those whose texel is green, above the
    lowest quarter of the tree (where moss on the roots is green too)."""
    pixels = np.asarray(color.resize((512, 512))).astype(np.float64) / 255.0
    at = (np.clip(uvs, 0.0, 0.999) * 512).astype(int)
    r, g, b = (pixels[at[:, 1], at[:, 0], k] for k in range(3))
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    high = positions[:, 1] > bottom + 0.25 * (top - bottom)
    return (g > r * 1.05) & (g > b * 1.05) & high


def deepen(positions, normals, leaves, factor):
    """The leaves spread `factor` times deeper (along Z) round the trunk,
    which stays as it was; their normals turned to match."""
    positions, normals = positions.copy(), normals.copy()
    positions[leaves, 2] *= factor
    normals[leaves, 2] /= factor
    normals /= np.linalg.norm(normals, axis=1, keepdims=True).clip(1e-9)
    return positions, normals


def read_pack(directory):
    """The trees already in a pack: name, positions, normals, uv (in the
    atlas), indices, and its atlas."""
    data = (directory / "trees.bin").read_bytes()
    assert data[:8] == b"MRMESH01", "not a mesh pack"
    (count,) = struct.unpack_from("<I", data, 8)
    offset, pieces = 12, []
    for _ in range(count):
        (length,) = struct.unpack_from("<I", data, offset)
        name = data[offset + 4 : offset + 4 + length].decode()
        offset += 4 + length
        vertices, indices, _flags = struct.unpack_from("<III", data, offset)
        offset += 12

        def take(dtype, count, width):
            nonlocal offset
            values = np.frombuffer(data, dtype=dtype, count=count * width, offset=offset)
            offset += values.nbytes
            return values.reshape(-1, width).astype(np.float64 if dtype == "<f4" else np.uint32)

        positions = take("<f4", vertices, 3)
        normals = take("<f4", vertices, 3)
        uvs = take("<f4", vertices, 2)
        triangles = take("<u4", indices, 1).reshape(-1)
        pieces.append((name, positions, normals, uvs, triangles))
    return pieces, Image.open(directory / "color.jpg").convert("RGB")


def main():
    arguments = sys.argv[1:]
    keep = "--keep" in arguments
    arguments = [argument for argument in arguments if argument != "--keep"]
    trees = []
    for argument in arguments:
        name, path = argument.split("=", 1)
        if name not in HEIGHTS:
            sys.exit(f"{name}: not one of {', '.join(HEIGHTS)}")
        trees.append((name, path))
    if not trees:
        sys.exit(__doc__)
    # The trees kept from the pack, each with its own cell of the old atlas.
    kept = {}
    if keep:
        old, old_atlas = read_pack(OUT)
        given = {name for name, _ in trees}
        old_columns = old_atlas.size[0] // CELL
        for name, positions, normals, uvs, indices in old:
            if name in given:
                continue
            column, row = (np.floor(uvs.mean(axis=0) * np.array(old_atlas.size) / CELL)).astype(int)
            cell = old_atlas.crop((column * CELL, row * CELL, (column + 1) * CELL, (row + 1) * CELL))
            # Back to the texture coordinates of its own texture (0..1).
            local = (uvs * np.array(old_atlas.size) - [column * CELL + MARGIN, row * CELL + MARGIN]) / (CELL - 2 * MARGIN)
            kept[name] = (positions, normals, local, indices, cell)
            assert old_columns >= 1
        trees += [(name, None) for name in kept]
    # Always in the same order, that of `HEIGHTS`, the game's.
    trees.sort(key=lambda tree: list(HEIGHTS).index(tree[0]))
    rows = (len(trees) + COLUMNS - 1) // COLUMNS
    atlas = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    pieces = []
    for slot, (name, path) in enumerate(trees):
        column, row = slot % COLUMNS, slot // COLUMNS
        inner = CELL - 2 * MARGIN
        if path is None:
            positions, normals, uvs, indices, cell = kept[name]
        else:
            positions, normals, uvs, indices, color = read_glb(path)
            if name in MENDS:
                positions, normals = deepen(positions, normals, foliage(positions, uvs, color), MENDS[name])
            positions = stand(positions, HEIGHTS[name])
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
            + (" (kept)" if path is None else "")
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
