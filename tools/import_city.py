#!/usr/bin/env python3
"""Prepares the city round Puerto Neón's lap, modelled with Meshy, for the
game (see docs/puerto-neon-props.md, "The city round the lap"): its blocks
and towers, its industry, its landmarks and what stands in the sea.

Reads one GLB per model and writes to `assets/models/neon_city/` what the
game loads (`src/game/neon.rs`): `props.bin`, every model as a piece named
after it, centred on the origin with its lowest point on the ground, facing
+Z, at the game's size (`MODELS`); `color.jpg`, their colours together, in a
grid of cells; and `glow.jpg`, laid out as `color.jpg`, what of them shines
of its own at night.

What shines is drawn from the models' own colours, each as its `MODELS`
entry says:

- `windows`: the models were made with their windows as flat yellow
  patches; on the walls those become a grid of windows, a floor every
  3.8 m and one every 3.2 m along, most lit (warm or cold white, cyan,
  magenta, amber, violet), the rest dark glass. Yellow on a roof is dark.

Every model is made brighter and more colourful than as made, and its own
colours glow a little all over (`FACADE`): the city is bright and in
colour at night.
- `painted`: their lit windows are painted on, warm: those glow.
- `lamps`: their own warm and bright touches (warning lights, the yellow
  edges of the silos) glow, as lamps.
- `cabins`, `ring`: their yellow glows whole, warm (a wheel's cabins) or
  as a hologram, cyan.
- `stadium`: its yellow facing up is the pitch, green under floodlights;
  on its sides, its stands' lights.
- `rig`: its yellow facing up is a helideck, lit; the rest, windows.
- `paint`: its yellow is paint (an offshore turbine's): nothing glows.
- `facets`: dark glass with pale streaks: the streaks glow, cold.
- `signs`: no texture at all (only its shape): it is given one, dark steel,
  and its three panels (its three largest parts, faced front and back)
  glow, each a colour of `SIGNS`, in as many sets as there are; one piece
  for each set.

Two models turn: the wind turbine's rotor (its parts in front of the tower,
over its middle) and the observation wheel (the wheel and, each hanging
from it, its cabins, which turn back so as to hang upright). Each is cut
into pieces, all about the turning part's axle (+Z), named `NAME_rotor`,
`NAME_wheel` and `NAME_cabin_K`; the rest stays `NAME`, its axle where the
pieces' origin is.

Usage: python3 tools/import_city.py NAME=MODEL.glb [NAME=MODEL.glb ...]
with every name of `MODELS`. Needs numpy and Pillow, and
tools/import_props.py and tools/import_trees.py beside it.
"""

import json
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageEnhance

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_props import compact, place  # noqa: E402
from import_trees import read_glb  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "neon_city"

# Each model: its size in the game (by its height, or its length across)
# and what of it shines.
MODELS = {
    "city_office": (("height", 90.0), "windows"),
    "city_residential": (("height", 110.0), "painted"),
    "city_megablock": (("height", 140.0), "windows"),
    "city_ziggurat": (("height", 80.0), "windows"),
    "city_cylinder": (("height", 120.0), "windows"),
    "city_stacked": (("height", 100.0), "windows"),
    "city_spire": (("height", 200.0), "windows"),
    "city_stepped": (("height", 150.0), "windows"),
    "city_twin": (("height", 120.0), "windows"),
    "city_faceted": (("height", 180.0), "facets"),
    "chimney_stack": (("height", 90.0), "lamps"),
    "cooling_towers": (("height", 70.0), "lamps"),
    "refinery": (("length", 60.0), "lamps"),
    "silo_row": (("length", 50.0), "lamps"),
    "antenna_mast": (("height", 130.0), "lamps"),
    "rail_viaduct": (("length", 60.0), "lamps"),
    "sign_tower": (("height", 60.0), "signs"),
    "stadium": (("length", 160.0), "stadium"),
    "ferris_wheel": (("height", 90.0), "cabins"),
    "hologram_ring": (("height", 50.0), "ring"),
    "wind_turbine": (("height", 110.0), "paint"),
    "oil_platform": (("height", 70.0), "rig"),
}

CELL = 1024
COLUMNS = 5
MARGIN = 2

# The windows: how far apart (m) along a wall and up it, how much of each
# is glass, and how many are lit; the colours they are lit in, and how
# often each (the whites most).
WINDOW = (3.2, 3.8)
GLASS = (0.62, 0.52)
LIT = 0.68
LIGHTS = [
    ((1.0, 0.86, 0.62), 0.3),
    ((0.82, 0.92, 1.0), 0.2),
    ((0.35, 0.95, 1.0), 0.14),
    ((1.0, 0.35, 0.8), 0.14),
    ((1.0, 0.72, 0.25), 0.12),
    ((0.65, 0.45, 1.0), 0.1),
]
DARK_GLASS = (0.05, 0.06, 0.08)
MULLION = (0.03, 0.03, 0.035)
# How much brighter and more colourful the models' own colours are made,
# and how much of them shines of its own at night, all over: a city
# bright and in colour, not dark blocks with windows.
BRIGHTER = 1.2
COLOURFUL = 1.35
FACADE = 0.28
# The colours of the hologram, the pitch, a helideck and the cabins.
HOLOGRAM = (0.35, 0.95, 1.0)
PITCH = (0.25, 0.75, 0.3)
HELIDECK = (0.95, 0.8, 0.35)
CABIN = (1.0, 0.82, 0.5)
# A sign tower's steel, and its panels' colours, set by set.
STEEL = (0.13, 0.14, 0.17)
SIGNS = [
    [(1.0, 0.25, 0.75), (0.3, 0.9, 1.0), (1.0, 0.7, 0.2)],
    [(0.3, 0.9, 1.0), (1.0, 0.7, 0.2), (0.6, 0.35, 1.0)],
]


def hsv(rgb):
    """Hue (degrees), saturation and value of colours 0..1 (N x 3)."""
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    high, low = rgb.max(axis=-1), rgb.min(axis=-1)
    span = (high - low).clip(1e-9)
    hue = np.where(
        high == r,
        ((g - b) / span) % 6.0,
        np.where(high == g, (b - r) / span + 2.0, (r - g) / span + 4.0),
    ) * 60.0
    return hue, np.where(high > 0, (high - low) / high.clip(1e-9), 0.0), high


def yellow(rgb):
    """The window yellow of the models made with it."""
    hue, saturation, value = hsv(rgb)
    return (hue > 36.0) & (hue < 64.0) & (saturation > 0.5) & (value > 0.5)


def chance(*numbers):
    """A number from 0 to 1 for `numbers`, the same every time."""
    x = 1469598103934665603
    for n in numbers:
        x = ((x ^ (int(n) & 0xFFFFFFFF)) * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    x ^= x >> 29
    return (x % 100000) / 100000.0


def light(cell):
    """The colour a lit window is lit in, if it is (else None)."""
    if chance(*cell, 7) > LIT:
        return None
    pick = chance(*cell, 13)
    for colour, share in LIGHTS:
        if pick < share:
            return colour
        pick -= share
    return LIGHTS[0][0]


def raster(positions, uvs, indices, inner, keep):
    """For each texel of a cell `inner` texels square that a triangle
    covers and `keep` (a mask of the cell) keeps: its texel (x, y), the
    point of the model there and the triangle's normal."""
    triangles = indices.reshape(-1, 3)
    at = uvs * (inner - 1)
    out_xy, out_point, out_normal = [], [], []
    for a, b, c in triangles:
        (ua, va), (ub, vb), (uc, vc) = at[a], at[b], at[c]
        x0, x1 = int(np.floor(min(ua, ub, uc))), int(np.ceil(max(ua, ub, uc)))
        y0, y1 = int(np.floor(min(va, vb, vc))), int(np.ceil(max(va, vb, vc)))
        x0, y0 = max(x0, 0), max(y0, 0)
        x1, y1 = min(x1, inner - 1), min(y1, inner - 1)
        if x1 < x0 or y1 < y0 or not keep[y0 : y1 + 1, x0 : x1 + 1].any():
            continue
        det = (vb - vc) * (ua - uc) + (uc - ub) * (va - vc)
        if abs(det) < 1e-12:
            continue
        ys, xs = np.mgrid[y0 : y1 + 1, x0 : x1 + 1]
        xs, ys = xs.ravel(), ys.ravel()
        px, py = xs + 0.5, ys + 0.5
        l1 = ((vb - vc) * (px - uc) + (uc - ub) * (py - vc)) / det
        l2 = ((vc - va) * (px - uc) + (ua - uc) * (py - vc)) / det
        l3 = 1.0 - l1 - l2
        inside = (l1 >= -0.02) & (l2 >= -0.02) & (l3 >= -0.02) & keep[ys, xs]
        if not inside.any():
            continue
        pa, pb, pc = positions[a], positions[b], positions[c]
        normal = np.cross(pb - pa, pc - pa)
        normal /= max(np.linalg.norm(normal), 1e-12)
        weights = np.stack([l1, l2, l3], axis=1)[inside]
        out_xy.append(np.stack([xs[inside], ys[inside]], axis=1))
        out_point.append(weights @ np.stack([pa, pb, pc]))
        out_normal.append(np.repeat(normal[None, :], inside.sum(), axis=0))
    if not out_xy:
        return np.zeros((0, 2), int), np.zeros((0, 3)), np.zeros((0, 3))
    return np.concatenate(out_xy), np.concatenate(out_point), np.concatenate(out_normal)


def windows(base, glow, positions, uvs, indices, mask, seed):
    """Over the texels of `mask`, a grid of windows on the walls (in the
    model's metres), lit or dark; dark on its roofs and where no triangle
    covers it."""
    inner = base.shape[0]
    base[mask] = DARK_GLASS
    xy, point, normal = raster(positions, uvs, indices, inner, mask)
    if len(xy) == 0:
        return
    wall = np.abs(normal[:, 1]) < 0.6
    across = np.cross(np.array([0.0, 1.0, 0.0]), normal)
    across /= np.linalg.norm(across, axis=1, keepdims=True).clip(1e-9)
    along = (point * across).sum(axis=1)
    up = point[:, 1]
    column, floor = np.floor(along / WINDOW[0]), np.floor(up / WINDOW[1])
    fx, fy = along / WINDOW[0] - column, up / WINDOW[1] - floor
    glass = (np.abs(fx - 0.5) < GLASS[0] / 2) & (np.abs(fy - 0.55) < GLASS[1] / 2) & wall
    facing = np.round(np.arctan2(normal[:, 0], normal[:, 2]) / (np.pi / 4)).astype(int)
    for k in np.flatnonzero(wall & ~glass):
        x, y = xy[k]
        base[y, x] = MULLION
    for k in np.flatnonzero(glass):
        x, y = xy[k]
        colour = light((seed, facing[k], column[k], floor[k]))
        if colour is None:
            base[y, x] = DARK_GLASS
        else:
            base[y, x] = np.array(colour) * 0.8
            glow[y, x] = colour


def components(positions, indices):
    """Which connected part each triangle is in (by points shared, welded
    where they are at the same place)."""
    _, weld = np.unique(np.round(positions, 5), axis=0, return_inverse=True)
    weld = weld.reshape(-1)
    triangles = weld[indices.reshape(-1, 3)]
    parent = np.arange(weld.max() + 1)

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    for a, b, c in triangles:
        for x, y in ((a, b), (b, c)):
            rx, ry = find(x), find(y)
            if rx != ry:
                parent[rx] = ry
    return np.array([find(v) for v in triangles[:, 0]])


def parts_of(name, positions, indices):
    """The pieces a model is cut into (name and indices of each), and the
    middle of its turning part's axle (x, y, in its own units), if it has
    one."""
    if name not in ("wind_turbine", "ferris_wheel"):
        return [(name, indices)], None
    triangles = indices.reshape(-1, 3)
    roots = components(positions, indices)
    groups = {}
    for k, root in enumerate(roots):
        groups.setdefault(root, []).append(k)
    boxes = {
        root: (positions[triangles[ks]].reshape(-1, 3).min(axis=0), positions[triangles[ks]].reshape(-1, 3).max(axis=0))
        for root, ks in groups.items()
    }
    pick = lambda roots_: keep_of(triangles, groups, roots_)  # noqa: E731
    if name == "wind_turbine":
        # Its rotor: what stands in front of the tower (the blades and the
        # spinner), over its foot.
        rotor = [r for r, (low, high) in boxes.items() if low[2] > 0.03 and high[1] > 0.0]
        spinner = max(
            (r for r in rotor if (boxes[r][1] - boxes[r][0])[0] < 0.1),
            key=lambda r: len(groups[r]),
        )
        low, high = boxes[spinner]
        axle = ((low[0] + high[0]) / 2.0, (low[1] + high[1]) / 2.0)
        rest = [r for r in boxes if r not in rotor]
        return [(name, pick(rest)), (f"{name}_rotor", pick(rotor))], axle
    # The wheel: its rim, the largest part reaching far round; its legs,
    # down to the ground; its cabins, small, out at its rim; the rest
    # (spokes, the hub) turns with the rim.
    bottom = positions[:, 1].min()
    rim = max(boxes, key=lambda r: (boxes[r][1] - boxes[r][0])[0])
    low, high = boxes[rim]
    axle = ((low[0] + high[0]) / 2.0, (low[1] + high[1]) / 2.0)
    radius = (high[0] - low[0]) / 2.0
    legs, cabins, wheel = [], [], []
    for r, (lo, hi) in boxes.items():
        middle = (lo + hi) / 2.0
        out = np.hypot(middle[0] - axle[0], middle[1] - axle[1])
        if lo[1] < bottom + 0.02:
            legs.append(r)
        elif (
            r != rim
            and out > 0.75 * radius
            and len(groups[r]) >= 24
            and 0.04 < (hi - lo)[2] < 0.07
            and min((hi - lo)[0], (hi - lo)[1]) > 0.04
            and (hi - lo).max() < 0.1
        ):
            # A cabin: as wide and tall as it is deep, a few metres.
            cabins.append(r)
        else:
            wheel.append(r)
    pieces = [(name, pick(legs)), (f"{name}_wheel", pick(wheel))]
    for k, r in enumerate(sorted(cabins, key=lambda r: np.arctan2(*(boxes[r][0] + boxes[r][1])[:2][::-1]))):
        pieces.append((f"{name}_cabin_{k}", pick([r])))
    return pieces, axle


def keep_of(triangles, groups, roots):
    keep = np.zeros(len(triangles), bool)
    for root in roots:
        keep[groups[root]] = True
    return triangles[keep].reshape(-1)


def read_bare(path):
    """A GLB with only its shape (no texture, no normals, no texture
    coordinates): its points and triangles."""
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
        width = {"SCALAR": 1, "VEC3": 3}[acc["type"]]
        kind = {5126: np.float32, 5125: np.uint32, 5123: np.uint16}[acc["componentType"]]
        start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
        values = np.frombuffer(binary, dtype=kind, count=acc["count"] * width, offset=start)
        return values.reshape(-1, width) if width > 1 else values

    (primitive,) = gltf["meshes"][0]["primitives"]
    positions = accessor(primitive["attributes"]["POSITION"]).astype(np.float64)
    return positions, accessor(primitive["indices"]).astype(np.uint32)


def signs(path, inner):
    """The sign tower, given a texture: its triangles each on their own
    points (flat), steel but its panels' faces, which take a swatch of
    their set's colour; a piece for each set. Its colours and glow (a cell
    of swatches) and its sets (positions, normals, uvs, indices)."""
    positions, indices = read_bare(path)
    triangles = indices.reshape(-1, 3)
    roots = components(positions, indices)
    sizes = {r: (roots == r).sum() for r in np.unique(roots)}
    wide = [r for r in sizes if np.ptp(positions[triangles[roots == r]].reshape(-1, 3)[:, 0]) > 0.15]
    panels = sorted(wide, key=lambda r: -sizes[r])[:3]
    panels.sort(key=lambda r: positions[triangles[roots == r]].reshape(-1, 3)[:, 1].mean())
    corners = positions[triangles]
    normals = np.cross(corners[:, 1] - corners[:, 0], corners[:, 2] - corners[:, 0])
    normals /= np.linalg.norm(normals, axis=1, keepdims=True).clip(1e-12)
    # Swatches: steel in the first column, each set's three colours across.
    base = np.zeros((inner, inner, 3))
    glow = np.zeros((inner, inner, 3))
    swatches = len(SIGNS) * 3 + 1
    side = inner // swatches

    def swatch(k):
        return (k * side + side / 2.0) / inner

    base[:, :side] = STEEL
    sets = []
    for s, colours in enumerate(SIGNS):
        for p, colour in enumerate(colours):
            k = 1 + 3 * s + p
            base[:, k * side : (k + 1) * side] = np.array(colour) * 0.7
            glow[:, k * side : (k + 1) * side] = colour
        uv = np.full((len(triangles), 2), [swatch(0), 0.5])
        for p, root in enumerate(panels):
            face = (roots == root) & (np.abs(normals[:, 2]) > 0.8)
            uv[face] = [swatch(1 + 3 * s + p), 0.5]
        sets.append(
            (
                corners.reshape(-1, 3),
                np.repeat(normals, 3, axis=0),
                np.repeat(uv, 3, axis=0),
                np.arange(len(triangles) * 3, dtype=np.uint32),
            )
        )
    return base, glow, sets


def shine(mode, name, rgb, positions, uvs, indices, seed):
    """The colours and the glow of a model's cell (`rgb`, 0..1), as its
    mode says."""
    base, glow = rgb.copy(), np.zeros_like(rgb)
    hue, saturation, value = hsv(rgb)
    mask = yellow(rgb)
    if mode == "windows":
        windows(base, glow, positions, uvs, indices, mask, seed)
    elif mode == "rig":
        inner = rgb.shape[0]
        xy, _, normal = raster(positions, uvs, indices, inner, mask)
        up = np.zeros(mask.shape, bool)
        up[xy[:, 1], xy[:, 0]] = normal[:, 1] > 0.7
        glow[up] = HELIDECK
        windows(base, glow, positions, uvs, indices, mask & ~up, seed)
        base[up] = np.array(HELIDECK) * 0.8
    elif mode == "stadium":
        inner = rgb.shape[0]
        xy, _, normal = raster(positions, uvs, indices, inner, mask)
        up = np.zeros(mask.shape, bool)
        up[xy[:, 1], xy[:, 0]] = normal[:, 1] > 0.7
        base[up] = PITCH
        glow[up] = np.array(PITCH) * 0.7
        stands = mask & ~up
        base[stands] = (0.95, 0.95, 0.85)
        glow[stands] = (1.0, 0.95, 0.8)
    elif mode == "painted":
        warm = (hue > 15.0) & (hue < 50.0) & (saturation > 0.35) & (value > 0.45)
        glow[warm] = rgb[warm] * 1.3
    elif mode == "lamps":
        warm = ((hue < 60.0) | (hue > 330.0)) & (saturation > 0.55) & (value > 0.6)
        glow[warm] = rgb[warm]
    elif mode == "cabins":
        base[mask] = np.array(CABIN) * 0.8
        glow[mask] = CABIN
    elif mode == "ring":
        base[mask] = np.array(HOLOGRAM) * 0.7
        glow[mask] = HOLOGRAM
    elif mode == "facets":
        streak = (value > 0.55) & (saturation < 0.45)
        glow[streak] = np.array([0.6, 0.85, 1.0]) * value[streak, None]
    return base, glow


def to_image(values):
    return Image.fromarray((values.clip(0.0, 1.0) * 255.0).round().astype(np.uint8))


def main():
    given = {}
    for argument in sys.argv[1:]:
        name, path = argument.split("=", 1)
        if name not in MODELS:
            sys.exit(f"{name}: not one of {', '.join(MODELS)}")
        given[name] = path
    missing = [name for name in MODELS if name not in given]
    if missing:
        sys.exit(f"every model has to be given; missing: {', '.join(missing)}\n\n{__doc__}")
    rows = (len(MODELS) + COLUMNS - 1) // COLUMNS
    size = np.array([COLUMNS * CELL, rows * CELL], dtype=np.float64)
    atlas = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    glows = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    inner = CELL - 2 * MARGIN
    pieces = []
    for slot, name in enumerate(MODELS):
        (sizing, mode), path = MODELS[name], given[name]
        column, row = slot % COLUMNS, slot // COLUMNS
        corner = np.array([column * CELL + MARGIN, row * CELL + MARGIN], dtype=np.float64)
        if mode == "signs":
            base, glow, sets = signs(path, inner)
            glow = np.maximum(glow, base * FACADE)
            models = [(f"{name}_{'abcdefgh'[k]}", *model) for k, model in enumerate(sets)]
        else:
            positions, normals, uvs, indices, color = read_glb(path)
            models = [(name, positions, normals, uvs, indices)]
            color = ImageEnhance.Color(color).enhance(COLOURFUL)
            color = ImageEnhance.Brightness(color).enhance(BRIGHTER)
            rgb = np.asarray(color.resize((inner, inner), Image.LANCZOS)).astype(np.float64) / 255.0
            # The model at the game's size, for its windows in metres.
            _, _, centre, scale = place(positions, normals, sizing)
            base, glow = shine(mode, name, rgb, (positions - centre) * scale, uvs.clip(0.0, 1.0), indices, slot * 1009)
            # Its walls glow a little in their own colours too.
            glow = np.maximum(glow, base * FACADE)
        for image, values in ((atlas, base), (glows, glow)):
            cell = to_image(values).resize((CELL, CELL), Image.NEAREST)
            cell.paste(to_image(values), (MARGIN, MARGIN))
            image.paste(cell, (column * CELL, row * CELL))

        for model, positions, normals, uvs, indices in models:
            moved = (corner + uvs.clip(0.0, 1.0) * inner) / size
            parts, axle = parts_of(name, positions, indices)
            used = np.unique(np.concatenate([part for _, part in parts]))
            _, _, centre, scale = place(positions[used], normals[used], sizing)
            placed = (positions - centre) * scale
            turned = normals / scale
            turned /= np.linalg.norm(turned, axis=1, keepdims=True).clip(1e-9)
            pivot = None
            if axle is not None:
                pivot = (np.array([axle[0], axle[1], 0.0]) - centre) * scale
                pivot[2] = 0.0
            for piece, part in parts:
                p, n, u, i = compact(placed, turned, moved, part)
                if pivot is not None:
                    # The turning pieces about their axle; what holds them,
                    # its foot on the ground, the axle where it is.
                    if piece != name:
                        p = p - pivot
                piece_name = piece if mode != "signs" else model
                pieces.append((piece_name, p, n, u, i))
                low, high = p.min(axis=0), p.max(axis=0)
                print(
                    f"{piece_name}: {len(p)} vertices, {len(i) // 3} triangles, "
                    f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} m"
                )
            if pivot is not None:
                print(f"{name}: its axle {pivot[0]:.2f}, {pivot[1]:.2f} m over its foot")

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
    # Full colour resolution (no chroma subsampling) and little loss: the
    # windows are fine patterns of colour, blurred otherwise.
    atlas.save(OUT / "color.jpg", quality=90, subsampling=0, optimize=True)
    glows.save(OUT / "glow.jpg", quality=94, subsampling=0, optimize=True)
    print(f"{OUT}: props.bin, color.jpg, glow.jpg ({atlas.size[0]}x{atlas.size[1]})")


if __name__ == "__main__":
    main()
