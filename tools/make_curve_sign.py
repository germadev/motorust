#!/usr/bin/env python3
"""Builds Costa Bits' bend warning sign (`curve_sign`) as a GLB, in place of
Meshy's, whose arrow came out a smudge.

A yellow diamond with a thick black border and a bold black arrow bending
right (or left with `--left`), on a grey square post. Written as Meshy's
models are, so that the props' importer reads it as one of them: a single
mesh with positions, normals, texture coordinates and indices, one material
with its colour texture embedded, the front facing +Z, up along +Y, its foot
on the ground (the importer scales it to its height in `SIZES`).

The panel is a diamond 0.9 m on a side (1.27 m from corner to corner), its
middle 2.35 m up, so the whole sign stands 3 m high; the post is 8 cm
square. 22 triangles.

Usage: python3 tools/make_curve_sign.py OUT.glb [--left]. Needs numpy and
Pillow.
"""

import io
import json
import math
import struct
import sys

import numpy as np
from PIL import Image, ImageDraw

YELLOW = (250, 196, 20)
BLACK = (18, 18, 18)
METAL = (150, 152, 156)
BACK = (110, 112, 116)

TEXTURE = 512
# The face is drawn in the top left square of the texture, this much of it;
# the greys of the post and the back in the strip to its right.
FACE = 0.75

SIDE = 0.9
CENTRE = 2.35
HALF = SIDE / math.sqrt(2.0)  # from the middle to a corner
DEPTH = 0.04  # the panel's thickness
POST = 0.04  # half the post's width
POST_TOP = CENTRE + 0.2


def face_image(left):
    """The sign's face, drawn at four times its size and brought down."""
    size = 4 * int(TEXTURE * FACE)
    image = Image.new("RGB", (size, size), BLACK)
    draw = ImageDraw.Draw(image)
    s = size / 2
    corners = lambda r: [(s, s - r), (s + r, s), (s, s + r), (s - r, s)]
    draw.polygon(corners(s * 0.84), fill=YELLOW)
    # The arrow: a stem up from below the middle, bending round to the
    # right, and a head pointing right.
    width = int(size * 0.09)
    radius = size * 0.13
    x0 = s - size * 0.07
    bottom, bend = s + size * 0.24, s - size * 0.02
    arc = [
        (x0 + radius - radius * math.cos(a), bend - radius * math.sin(a))
        for a in np.linspace(0.0, math.pi / 2, 24)
    ]
    path = [(x0, bottom), (x0, bend)] + arc
    end_x, end_y = path[-1]
    shaft_end = end_x + size * 0.02
    path.append((shaft_end, end_y))
    draw.line(path, fill=BLACK, width=width, joint="curve")
    draw.ellipse(
        [x0 - width / 2, bottom - width / 2, x0 + width / 2, bottom + width / 2],
        fill=BLACK,
    )
    head = size * 0.14
    draw.polygon(
        [
            (shaft_end - 2, end_y - head),
            (shaft_end + head * 1.1, end_y),
            (shaft_end - 2, end_y + head),
        ],
        fill=BLACK,
    )
    if left:
        image = image.transpose(Image.FLIP_LEFT_RIGHT)
    side = int(TEXTURE * FACE)
    return image.resize((side, side), Image.LANCZOS)


def texture(left):
    image = Image.new("RGB", (TEXTURE, TEXTURE), BACK)
    image.paste(face_image(left), (0, 0))
    draw = ImageDraw.Draw(image)
    edge = int(TEXTURE * FACE)
    draw.rectangle([edge, 0, TEXTURE, TEXTURE // 2], fill=METAL)
    draw.rectangle([edge, TEXTURE // 2, TEXTURE, TEXTURE], fill=BACK)
    return image


METAL_UV = ((FACE + 1.0) / 2, 0.25)
BACK_UV = ((FACE + 1.0) / 2, 0.75)


def build(left):
    positions, normals, uvs, indices = [], [], [], []

    def quad(corners, normal, uv):
        start = len(positions)
        for corner, point in zip(corners, uv):
            positions.append(corner)
            normals.append(normal)
            uvs.append(point)
        indices.extend([start, start + 1, start + 2, start, start + 2, start + 3])

    # The panel: its front (textured), its back, and its four edges.
    diamond = [(0.0, HALF), (HALF, 0.0), (0.0, -HALF), (-HALF, 0.0)]
    front = [(x, CENTRE + y, DEPTH / 2) for x, y in diamond]
    back = [(x, CENTRE + y, -DEPTH / 2) for x, y in diamond]
    face_uv = [
        (FACE * (0.5 + x / (2 * HALF)), FACE * (0.5 - y / (2 * HALF)))
        for x, y in diamond
    ]
    # Counter-clockwise seen from +Z: top, left, bottom, right.
    order = [0, 3, 2, 1]
    quad([front[i] for i in order], (0.0, 0.0, 1.0), [face_uv[i] for i in order])
    quad(back, (0.0, 0.0, -1.0), [BACK_UV] * 4)
    for i in range(4):
        a, b = diamond[i], diamond[(i + 1) % 4]
        out = np.array([a[0] + b[0], a[1] + b[1], 0.0])
        out /= np.linalg.norm(out)
        quad(
            [back[i], front[i], front[(i + 1) % 4], back[(i + 1) % 4]][::-1],
            tuple(out),
            [METAL_UV] * 4,
        )

    # The post, behind the panel, from the ground to just above its middle.
    z0, z1 = -DEPTH / 2 - 2 * POST, -DEPTH / 2
    x0, x1 = -POST, POST
    for normal, corners in [
        ((0, 0, 1), [(x0, 0, z1), (x1, 0, z1), (x1, POST_TOP, z1), (x0, POST_TOP, z1)]),
        (
            (0, 0, -1),
            [(x1, 0, z0), (x0, 0, z0), (x0, POST_TOP, z0), (x1, POST_TOP, z0)],
        ),
        ((1, 0, 0), [(x1, 0, z1), (x1, 0, z0), (x1, POST_TOP, z0), (x1, POST_TOP, z1)]),
        (
            (-1, 0, 0),
            [(x0, 0, z0), (x0, 0, z1), (x0, POST_TOP, z1), (x0, POST_TOP, z0)],
        ),
        (
            (0, 1, 0),
            [
                (x0, POST_TOP, z1),
                (x1, POST_TOP, z1),
                (x1, POST_TOP, z0),
                (x0, POST_TOP, z0),
            ],
        ),
    ]:
        quad(corners, normal, [METAL_UV] * 4)

    return (
        np.array(positions, np.float32),
        np.array(normals, np.float32),
        np.array(uvs, np.float32),
        np.array(indices, np.uint32),
        texture(left),
    )


def write_glb(path, positions, normals, uvs, indices, image):
    png = io.BytesIO()
    image.save(png, "PNG", optimize=True)
    blobs = [
        positions.tobytes(),
        normals.tobytes(),
        uvs.tobytes(),
        indices.tobytes(),
        png.getvalue(),
    ]
    views, binary = [], b""
    for blob in blobs:
        views.append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(blob)})
        binary += blob + b"\0" * (-len(blob) % 4)
    count = len(positions)
    gltf = {
        "asset": {"version": "2.0", "generator": "motorust tools/make_curve_sign.py"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "curve_sign", "mesh": 0}],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                        "indices": 3,
                        "material": 0,
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": count,
                "type": "VEC3",
                "min": positions.min(axis=0).tolist(),
                "max": positions.max(axis=0).tolist(),
            },
            {"bufferView": 1, "componentType": 5126, "count": count, "type": "VEC3"},
            {"bufferView": 2, "componentType": 5126, "count": count, "type": "VEC2"},
            {
                "bufferView": 3,
                "componentType": 5125,
                "count": len(indices),
                "type": "SCALAR",
            },
        ],
        "bufferViews": views,
        "buffers": [{"byteLength": len(binary)}],
        "materials": [
            {
                "pbrMetallicRoughness": {
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": 0.0,
                    "roughnessFactor": 1.0,
                }
            }
        ],
        "textures": [{"source": 0}],
        "images": [{"bufferView": 4, "mimeType": "image/png"}],
    }
    header = json.dumps(gltf, separators=(",", ":")).encode()
    header += b" " * (-len(header) % 4)
    total = 12 + 8 + len(header) + 8 + len(binary)
    with open(path, "wb") as out:
        out.write(struct.pack("<III", 0x46546C67, 2, total))
        out.write(struct.pack("<II", len(header), 0x4E4F534A) + header)
        out.write(struct.pack("<II", len(binary), 0x004E4942) + binary)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 1:
        sys.exit(__doc__)
    mesh = build("--left" in sys.argv)
    write_glb(args[0], *mesh)
    print(f"{args[0]}: {len(mesh[3]) // 3} triangles")


if __name__ == "__main__":
    main()
