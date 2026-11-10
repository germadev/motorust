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
- `billboard.jpg`: the frames of the billboard's picture.
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
out to match, the rest as made; and its ends, which came as stubs hanging
off its side arches, are drawn down to the ground as abutments.

The billboard (`arcade_billboard`) came with its picture blurred and its
palms smeared: a panel is laid over it inside the frame (`mend_billboard`,
the piece `arcade_billboard_panel`), for a picture of our own that the game
shows frame by frame (`sunset`, its frames together in `billboard.jpg`); and
its bulbs are taken apart in three groups (`arcade_billboard_bulbs_N`), which
the game lights one after another.

The bend signs (`curve_sign`, `curve_sign_left`) are not Meshy's: they are
built by `tools/make_curve_sign.py`.

Usage: python3 tools/import_costa.py NAME=PROP.glb [NAME=PROP.glb ...] with
the names of `SIZES`. Needs numpy and Pillow.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_props import compact, place
from import_trees import read_glb

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "costa"

# How big each prop is, as such a thing is (made bigger in the game by
# `ARCADE`): by its height (m), or its length (m, the longest way across,
# for a boat or a car). The bridge by its height
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
    # Flying over the bay, towing its banner: half as big again as such a
    # plane is, so that it reads from the road.
    "banner_plane": ("length", 33.0),
    "gorge_bridge": ("bridge", 60.0, 7.4),
}
# How much bigger than `SIZES` each prop is made in the game: half as big
# again, as the arcade racers drew what stood by their roads, so that it
# reads at speed; the houses a quarter, among the engine's cottages; the
# bridge and the plane as `SIZES` says (sized already for where they are).
ARCADE = 1.5
ARCADE_EXCEPT = {"white_house": 1.25, "gorge_bridge": 1.0, "banner_plane": 1.0}
# How many groups the billboard's bulbs are in, lit one after another.
BULB_GROUPS = 3
# The billboard's picture: how many frames across and down its image, and
# each frame's size (px).
FRAME_GRID = (2, 2)
FRAME_SIZE = (1024, 512)
# How far in from each end of the bridge its abutments reach (m).
ABUTMENT = 6.6
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
    # Its ends, which came as stubs hanging off the side arches (Meshy left
    # out the gorge's walls they stand on), taken down to the ground as
    # abutments: the underside of each stub drawn down, its sides with it.
    reach = np.abs(positions[:, 0] - middle)
    outer = reach >= reach.max() - ABUTMENT / scale
    under = outer & (positions[:, 1] <= positions[outer][:, 1].min() + 1.2 / scale)
    positions[under, 1] = bottom
    print(
        f"gorge_bridge: the wall at its foot cut ({wall.sum()} triangles), its arch stretched {stretch:.2f} times"
    )
    return positions, normals, uvs, indices.astype(np.uint32)


def sunset(width, height, phase):
    """A frame of the billboard's picture, `width` by `height` pixels,
    `phase` (0..1) through its loop: a striped sun going down into the sea
    under a sky from orange to magenta, its stripes sliding down it, its
    light glinting on the water, and palms in black against it at either
    side, their fronds swaying. No text. Drawn twice the size and brought
    down, so its edges are smooth."""
    w, h = 2 * width, 2 * height
    horizon = int(h * 0.62)
    # The sky and the sea, each a gradient down the picture.
    rows = np.arange(h, dtype=np.float64)[:, None]
    stops_sky = np.array(
        [[255, 170, 45], [255, 120, 60], [240, 80, 110], [200, 70, 160]], float
    )
    t = (rows / horizon).clip(0.0, 1.0) * (len(stops_sky) - 1)
    k = np.minimum(t.astype(int), len(stops_sky) - 2)
    sky = stops_sky[k[:, 0]] + (stops_sky[k[:, 0] + 1] - stops_sky[k[:, 0]]) * (t - k)
    u = ((rows - horizon) / (h - horizon)).clip(0.0, 1.0)
    sea = (
        np.array([60, 110, 225], float)
        + (np.array([20, 40, 130], float) - [60, 110, 225]) * u
    )
    column = np.where(rows < horizon, sky, sea)
    image = Image.fromarray(np.repeat(column[:, None, :], w, axis=1).astype(np.uint8))
    draw = ImageDraw.Draw(image)
    # The sun, half under the horizon, cut by stripes of the sky behind it
    # that slide down it and widen towards the sea.
    radius = int(h * 0.3)
    middle = w // 2
    sun = Image.new("L", (w, h), 0)
    ImageDraw.Draw(sun).ellipse(
        [middle - radius, horizon - radius, middle + radius, horizon + radius], fill=255
    )
    cut = ImageDraw.Draw(sun)
    stripes = 6
    for n in range(stripes + 1):
        down = (n + phase) / stripes
        y = horizon - radius + radius * down
        thick = 2 + radius * 0.07 * down
        cut.rectangle([middle - radius, y, middle + radius, y + thick], fill=0)
    cut.rectangle([0, horizon, w, h], fill=0)
    image.paste(Image.new("RGB", (w, h), (255, 222, 70)), (0, 0), sun)
    # The sun's light on the water: dashes narrowing towards the shore,
    # shifting from side to side.
    for n in range(7):
        y = horizon + 6 + n * (h - horizon - 12) // 7
        half = radius * (8 - n) / 9.0
        shift = radius * 0.12 * np.sin(2 * np.pi * (phase + n * 0.37))
        draw.rounded_rectangle(
            [middle - half + shift, y, middle + half + shift, y + max(3, h // 90)],
            radius=h // 180,
            fill=(255, 205, 120),
        )
    black = (14, 10, 22)

    def palm(foot, top, lean, sway):
        """A palm from `foot` (x) up to `top` (y), its trunk bending by
        `lean` pixels, its fronds arching out and drooping, swaying."""
        points = []
        for n in range(25):
            t = n / 24.0
            points.append((foot + lean * t * t, h - (h - top) * t))
        draw.line(points, fill=black, width=max(4, w // 90), joint="curve")
        head_x, head_y = points[-1]
        length = h * 0.3
        for angle in (-2.75, -2.2, -1.75, -1.3, -0.85, -0.4, 0.2, 3.35):
            angle += 0.07 * np.sin(2 * np.pi * (phase + sway)) * np.cos(angle)
            upper, lower = [], []
            for n in range(13):
                t = n / 12.0
                # Out along its angle, falling as the square of how far out.
                x = head_x + length * t * np.cos(angle)
                y = head_y + length * t * np.sin(angle) * 0.7 + length * 0.42 * t * t
                broad = length * 0.11 * np.sin(np.pi * t) ** 0.7
                upper.append((x, y - broad * 0.35))
                lower.append((x, y + broad))
            draw.polygon(upper + lower[::-1], fill=black)

    palm(w * 0.12, h * 0.2, w * 0.05, 0.0)
    palm(w * 0.24, h * 0.44, -w * 0.03, 0.3)
    palm(w * 0.87, h * 0.24, -w * 0.06, 0.6)
    return image.resize((width, height), Image.LANCZOS)


def billboard_frames():
    """The frames of the billboard's picture (`sunset`) together in one
    image, `FRAME_GRID` across and down, in the order of the loop from the
    top left."""
    columns, rows = FRAME_GRID
    width, height = FRAME_SIZE
    sheet = Image.new("RGB", (columns * width, rows * height))
    for frame in range(columns * rows):
        picture = sunset(width, height, frame / (columns * rows))
        sheet.paste(picture, ((frame % columns) * width, (frame // columns) * height))
    return sheet


def mend_billboard(positions, normals, uvs, indices, color):
    """The billboard with its picture drawn again: Meshy's came blurred,
    its palms smeared, and small in its texture. A panel is laid just
    before the old one, inside the frame, a piece of its own with texture
    coordinates over the whole of a picture of its own (`billboard_frames`,
    which the game shows frame by frame). Its bulbs are taken apart from it,
    in `BULB_GROUPS` groups by turns round the frame, for the game to light
    one group after another. Returns the mesh, its triangles without the
    bulbs, each group's triangles and the panel's."""
    corners_of = indices.reshape(-1, 3)
    triangles = positions[corners_of]
    normal = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    area = np.linalg.norm(normal, axis=1)
    front = normal[:, 2] / area.clip(1e-12) > 0.9
    middle = triangles.mean(axis=1)
    pixels = np.asarray(color.resize((512, 512))).astype(np.float64) / 255.0
    at = (np.clip(uvs[corners_of].mean(axis=1), 0.0, 0.999) * 512).astype(int)
    rgb = pixels[at[:, 1], at[:, 0]]
    vivid = rgb.max(axis=1) - rgb.min(axis=1) > 0.25
    # The bulbs: the yellow lumps on the frame's front, forward of it.
    yellow = (rgb[:, 0] > 0.6) & (rgb[:, 1] > 0.45) & (rgb[:, 2] < 0.45 * rgb[:, 0])
    # The picture: the big faces looking forward in vivid colours; the frame
    # round it, and the legs, are grey.
    picture = front & vivid & (area > np.percentile(area[front], 60)) & ~yellow
    panel = triangles[picture].reshape(-1, 3)
    x0, x1 = np.percentile(panel[:, 0], [1, 99])
    y0, y1 = np.percentile(panel[:, 1], [1, 99])
    panel_z = np.median(panel[:, 2])
    inside = (
        (middle[:, 0] > x0 - 0.06) & (middle[:, 0] < x1 + 0.06)
        & (middle[:, 1] > y0 - 0.06) & (middle[:, 1] < y1 + 0.06)
    )  # fmt: skip
    bulb = yellow & inside & (middle[:, 2] > panel_z + 0.004) & ~picture
    z = panel_z + 0.003
    corners = np.array([[x0, y0, z], [x1, y0, z], [x1, y1, z], [x0, y1, z]])
    start = len(positions)
    positions = np.vstack([positions, corners])
    normals = np.vstack([normals, np.tile([0.0, 0.0, 1.0], (4, 1))])
    uvs = np.vstack([uvs, [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]])
    quad = np.array(
        [start, start + 1, start + 2, start, start + 2, start + 3], dtype=np.uint32
    )

    # Each bulb is the triangles close together; round the frame from its
    # top left, one to each group in turn.
    centre = np.array([(x0 + x1) / 2.0, (y0 + y1) / 2.0])
    bulbs = []
    for k in np.where(bulb)[0]:
        for lump in bulbs:
            if np.linalg.norm(middle[k, :2] - middle[lump[0], :2]) < 0.03:
                lump.append(k)
                break
        else:
            bulbs.append([k])
    bulbs.sort(key=lambda lump: np.arctan2(*(middle[lump[0], :2] - centre)))
    groups = [[] for _ in range(BULB_GROUPS)]
    for k, lump in enumerate(bulbs):
        groups[k % BULB_GROUPS].extend(lump)
    rest = np.ones(len(corners_of), dtype=bool)
    rest[np.where(bulb)[0]] = False
    indices = corners_of[rest].reshape(-1).astype(np.uint32)
    lit = [corners_of[group].reshape(-1).astype(np.uint32) for group in groups]
    print(
        f"arcade_billboard: a panel for its picture; {len(bulbs)} bulbs in {BULB_GROUPS} groups"
    )
    return positions, normals, uvs, indices, lit, quad


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
        lit, panel = [], None
        if name == "arcade_billboard":
            positions, normals, uvs, indices, lit, panel = mend_billboard(
                positions, normals, uvs, indices, color
            )
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
        larger = ARCADE_EXCEPT.get(name, ARCADE)
        sized = (sized[0], sized[1] * larger) + tuple(sized[2:])
        if sized[0] == "bridge":
            positions, normals, uvs, indices = open_bridge(
                positions, normals, uvs, indices, *sized[1:]
            )
            sized = ("height", sized[1])
        moved = (corner + uvs.clip(0.0, 1.0) * inner) / size
        placed, turned, _, _ = place(positions, normals, sized)
        p, n, u, i = compact(placed, turned, moved, indices)
        pieces.append((name, p, n, u, i))
        # The billboard's bulbs, each group a piece of its own, in its frame,
        # and its panel, with texture coordinates over its own picture.
        for group, part in enumerate(lit):
            pieces.append(
                (f"{name}_bulbs_{group}", *compact(placed, turned, moved, part))
            )
        if panel is not None:
            pieces.append((f"{name}_panel", *compact(placed, turned, uvs, panel)))
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
    if any(name == "arcade_billboard" for name, _ in props):
        billboard_frames().save(OUT / "billboard.jpg", quality=90, optimize=True)
    print(f"{OUT / 'props.bin'}, {OUT / 'color.jpg'} ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
