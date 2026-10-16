#!/usr/bin/env python3
"""Prepares the props of Fellowship Road modelled with Meshy for the game:
hobbit holes, inns, a mill, gates and houses, ruins, elven halls, boats,
statues and great trees.

Reads one GLB per prop (a single textured mesh, normalised to a unit box)
and writes to `assets/models/fellowship/` what the game loads
(`src/world/middle_earth.rs`), as `tools/import_props.py` does for Ghost
Island's:

- `props.bin`: every prop as a piece named after it, centred on the origin
  with its lowest point on the ground (a tree on the foot of its trunk), up
  along +Y, at the game's size (`SIZES`).
- `color.jpg`: their colour textures together, each brought down to `CELL`
  pixels square, in a grid of `COLUMNS`.

Some models need mending as they are read:

- The stone trolls (`stone_trolls`): the left one came with a third leg out
  of its back, between its two. It is cut away, and the hole it leaves in
  its hip closed with a fan of triangles in the colour of its stone.
- Bree's gate (`bree_gate`): Meshy put two beams across its opening, a
  quarter and a third of the way up, where the bikes ride through: they are
  cut away. Sized by its opening, as the torii over Ghost Island's road,
  and squashed down to `SIZES`' height, from the 19 m it would stand.
- A house of Bree (`bree_house_a`) came hung with the Green Dragon's sign,
  off its right side: cut away (the Green Dragon is in Bywater).
- The elven boat (`elven_boat`) came with four oars out over its sides,
  lower than its keel: cut away, so that it lies on the shore.
- The mill (`mill`) and its wheel (`mill_wheel`) came apart, the mill with
  a hole for the axle in its side: the mill is turned so that that side
  faces +Z (the road, as the game places it) and the wheel is set against
  it, its axle in the hole, its rim a little below the ground; both have
  their origin at the middle of the axle, about which the wheel turns (on
  Z), as Ghost Island's water wheel.
- The Doors of Durin (`durin_doors`) came closed, and narrower than they
  are drawn in the film: they are made wider (`DURIN_WIDER`), and their
  two leaves cut from the frame and swung open outwards, a quarter turn
  about their hinges, as they open in the book and the film, so that the
  road goes through them into Moria (sized by its height, the gate stands
  over the road with its leaves just outside the walls of a narrow way
  through it). With them, two pieces of their own, drawn in a material of
  their own by the game: `durin_ithildin`, the drawing of ithildin on the
  arch and the rock over it (the arch, two trees up the pillars, the crown
  and the seven stars), and `durin_dark`, the darkness of Moria filling the
  doorway over the tunnel the road goes into.
- Textures toned (`TONED`): the oak over Bag End came a vivid green, the
  Doors of Durin a pale grey (darker, the drawing glowing on them in the
  game stands out).

Every prop faces +Z as Meshy made it, its width along X. Usage:
python3 tools/import_fellowship.py NAME=PROP.glb [NAME=PROP.glb ...] with
the names of `SIZES` (`mill_wheel` too). Needs numpy and Pillow.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageEnhance

sys.path.insert(0, str(Path(__file__).resolve().parent))
from import_props import compact, keep_triangles, place, texels  # noqa: E402
from import_trees import read_glb  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "fellowship"

# How big each prop is in the game: by its height (m), its length (m, the
# longest way across, for a boat or a ring of ruins), or for the gate how
# far from its middle its towers' inner faces stand (m, just outside the
# road's walls) and its height (m). "foot" stands a tree on the foot of its
# trunk, in the order of the lap.
SIZES = {
    # Hobbiton.
    "hobbit_door_green": ("height", 4.0),
    "hobbit_door_yellow": ("height", 4.0),
    "hobbit_door_red": ("height", 4.0),
    "hobbit_door_blue": ("height", 4.0),
    "bag_end": ("height", 5.5),
    "chimney": ("height", 2.4),
    "party_tree": ("height", 24.0, "foot"),
    "bag_end_oak": ("height", 14.0, "foot"),
    "mill": ("height", 11.0),
    "green_dragon": ("height", 8.0),
    "green_dragon_sign": ("height", 3.2),
    "gandalf_cart": ("length", 4.5),
    "log_pile": ("height", 1.5),
    # The Shire.
    "scarecrow": ("height", 2.3),
    "root_tree": ("height", 16.0, "foot"),
    "ferry": ("length", 9.0),
    # Bree.
    "bree_gate": ("opening", 11.63, 15.4),
    "prancing_pony": ("height", 14.0),
    "bree_house_a": ("height", 10.0),
    "bree_house_b": ("height", 11.0),
    # Weathertop and the Trollshaws.
    "weathertop_ruin": ("length", 32.0),
    "ruin_arch": ("height", 6.0),
    "stone_trolls": ("height", 7.0),
    # Rivendell.
    "last_homely_house": ("height", 18.0),
    "gazebo": ("height", 9.0),
    "elven_tower": ("height", 24.0),
    "elven_lamp": ("height", 3.4),
    # Moria.
    "durin_doors": ("height", 35.0),
    # Lothlórien, the Anduin and Amon Hen.
    "mallorn_flet": ("height", 38.0, "foot"),
    "swan_ship": ("length", 16.0),
    "elven_ship": ("length", 20.0),
    "elven_boat": ("length", 6.0),
    "argonath_a": ("height", 60.0),
    "argonath_b": ("height", 60.0),
    "amon_hen_seat": ("height", 9.0),
}
# How saturated and how bright each texture is left (1 as it came).
TONED = {
    "bag_end_oak": (0.8, 0.95),
    "durin_doors": (0.9, 0.72),
}
# How far the mill wheel's rim reaches below the ground (m), and how far
# out from the mill's wall it turns (m, between the wall and its side).
WHEEL_SUNK = 0.3
WHEEL_GAP = 0.1
# How much wider the Doors of Durin are made, closer to the arch drawn in
# the film. Over the doorway, the darkness starts this high over the
# gate's foot (m): just under the roof of the tunnel behind it
# (`TUNNEL_HEIGHT`, 6.5 m over the road, in `src/world/structures.rs`).
DURIN_WIDER = 1.4
DURIN_DARK_FROM = 6.3
# The ithildin's lines: how wide (m), how far in front of the stone (m),
# and how long their pieces are at most (m), each laid on the stone.
ITHILDIN_WIDTH = 0.38
ITHILDIN_OFF = 0.08
ITHILDIN_STEP = 0.4
CELL = 512
COLUMNS = 6
MARGIN = 2


def welded(positions):
    """An index per vertex, the same for vertices at the same place (Meshy
    splits them along the seams of its texture)."""
    _, ids = np.unique(np.round(positions, 5), axis=0, return_inverse=True)
    return ids.ravel()


def parts(positions, indices):
    """The connected part each triangle belongs to (a label per triangle),
    vertices at the same place counting as one."""
    ids = welded(positions)
    parent = np.arange(ids.max() + 1)

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    triangles = ids[indices.reshape(-1, 3)]
    for a, b, c in triangles:
        root = find(a)
        parent[find(b)] = root
        parent[find(c)] = root
    return np.array([find(a) for a in triangles[:, 0]])


def without_third_leg(positions, normals, uvs, indices, color):
    """The stone trolls without the third leg out of the left one's back
    (the largest part): the triangles of it behind its hips, between its two
    legs and low; the hole left in its hip closed with a fan of triangles
    about its middle, all in the colour of the stone where it was cut."""
    triangles = indices.reshape(-1, 3)
    label = parts(positions, indices)
    names, counts = np.unique(label, return_counts=True)
    left = label == names[counts.argmax()]
    middle = positions[triangles].mean(axis=1)
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    leg = (
        left
        & (middle[:, 0] < -0.30)
        & (middle[:, 2] > -0.09)
        & (middle[:, 2] < 0.085)
        & (middle[:, 1] < bottom + 0.38 * (top - bottom))
    )
    assert 250 < leg.sum() < 400, f"{leg.sum()} triangles of the leg"
    kept = triangles[~leg]
    # The edges of the hole: those of a kept triangle that were shared with
    # the leg's, in the kept triangle's order.
    ids = welded(positions)
    cut = set()
    for a, b, c in ids[triangles[leg]]:
        cut |= {(a, b), (b, c), (c, a)}
    edges = []
    for triangle in kept[left[~leg]]:
        for a, b in ((triangle[0], triangle[1]), (triangle[1], triangle[2]), (triangle[2], triangle[0])):
            if (ids[b], ids[a]) in cut:
                edges.append((a, b))
    assert 15 < len(edges) < 40, f"{len(edges)} edges round the hole"
    rim = np.unique(np.array(edges).ravel())
    centre = positions[rim].mean(axis=0)
    # One texture coordinate for the whole cap: that of the edge's vertex
    # nearest its middle, in the stone's colour.
    uv = uvs[rim[np.argmin(np.linalg.norm(positions[rim] - centre, axis=1))]]
    normal = normals[rim].mean(axis=0)
    normal /= np.linalg.norm(normal).clip(1e-9)
    # New vertices for the cap, so that it takes no other colour.
    new_positions, new_normals, new_uvs, new_triangles = [centre], [normal], [uv], []
    start = len(positions)
    for a, b in edges:
        k = start + len(new_positions)
        new_positions += [positions[b], positions[a]]
        new_normals += [normal, normal]
        new_uvs += [uv, uv]
        new_triangles.append((k, k + 1, start))
    return (
        np.vstack([positions, new_positions]),
        np.vstack([normals, new_normals]),
        np.vstack([uvs, new_uvs]),
        np.concatenate([kept.reshape(-1), np.array(new_triangles, dtype=np.uint32).reshape(-1)]),
    )


def without_bars(positions, indices):
    """Bree's gate without the beams across its opening: the triangles
    whose middle lies between its towers' inner faces (a quarter of its
    width either side of the middle) in the lower half of it."""
    middle = positions[indices.reshape(-1, 3)].mean(axis=1)
    bottom, top = positions[:, 1].min(), positions[:, 1].max()
    bars = (np.abs(middle[:, 0]) < 0.24) & (middle[:, 1] < bottom + 0.5 * (top - bottom))
    assert 8 <= bars.sum() <= 40, f"{bars.sum()} triangles of the bars"
    return keep_triangles(indices, ~bars)


def without_sign(positions, indices):
    """The house without the sign hanging off its right side (+X): its
    board, chains and the end of its bracket."""
    middle = positions[indices.reshape(-1, 3)].mean(axis=1)
    sign = middle[:, 0] > 0.372
    assert 20 < sign.sum() < 200, f"{sign.sum()} triangles of the sign"
    return keep_triangles(indices, ~sign)


def without_oars(positions, indices):
    """The elven boat without its oars: the parts reaching out past its
    hull's sides (an eighth of its length from its middle line)."""
    triangles = indices.reshape(-1, 3)
    label = parts(positions, indices)
    oars = np.zeros(len(triangles), dtype=bool)
    for name in np.unique(label):
        mine = label == name
        if np.abs(positions[triangles[mine]][:, :, 2]).max() > 0.12:
            oars |= mine
    assert 4 * 30 < oars.sum() < 400, f"{oars.sum()} triangles of the oars"
    return keep_triangles(indices, ~oars)


def door_leaves(positions, indices):
    """Which triangles are the two leaves of the Doors of Durin, as they
    came: between the pillars (an eighth of its height either side of its
    middle), in the middle of its depth, over the threshold and under the
    arch. And which are its threshold, a step across the road under them."""
    low, high = positions.min(axis=0), positions.max(axis=0)
    height = high[1] - low[1]
    middle = (low + high) / 2.0
    q = (positions - np.array([middle[0], low[1], middle[2]])) / height
    corners = q[indices.reshape(-1, 3)]
    leaves = (
        (np.abs(corners[:, :, 0]) <= 0.1285).all(axis=1)
        & (np.abs(corners[:, :, 2]) <= 0.0215).all(axis=1)
        & (corners[:, :, 1] >= 0.07).all(axis=1)
        & (corners[:, :, 1] <= 0.92).all(axis=1)
    )
    assert 300 < leaves.sum() < 600, f"{leaves.sum()} triangles of the leaves"
    threshold = (np.abs(corners[:, :, 0].mean(axis=1)) <= 0.128) & (corners[:, :, 1] <= 0.085).all(axis=1)
    assert 4 <= threshold.sum() < 300, f"{threshold.sum()} triangles of the threshold"
    # Nothing else low down in the way of the road (but flat on the ground).
    rest = corners[~threshold & ~leaves]
    top = rest[:, :, 1].max(axis=1)
    low = (top < 0.085) & (top > 0.01)
    assert (np.abs(rest[low][:, :, 0]).min(axis=1) > 0.115).all()
    return leaves, threshold


def widened(positions, normals, factor):
    """Stretched along X by `factor`, its normals turned to match."""
    stretch = np.array([factor, 1.0, 1.0])
    bent = normals / stretch
    bent /= np.linalg.norm(bent, axis=1, keepdims=True).clip(1e-9)
    return positions * stretch, bent


def swung_open(positions, normals, uvs, indices, leaves):
    """The doors (in metres) with their leaves swung open outwards, a
    quarter turn each about its hinge: the back of its outer edge, against
    the pillar. The leaves get vertices of their own."""
    triangles = indices.reshape(-1, 3)
    frame = triangles[~leaves].reshape(-1)
    p, n, u, i = [positions], [normals], [uvs], [frame]
    count = len(positions)
    for side in (-1.0, 1.0):
        mine = triangles[leaves & (side * positions[triangles].mean(axis=1)[:, 0] > 0)]
        corners = mine.reshape(-1)
        leaf_p, leaf_n = positions[corners].copy(), normals[corners].copy()
        hinge = np.array([side * np.abs(leaf_p[:, 0]).max(), leaf_p[:, 2].min()])
        dx, dz = leaf_p[:, 0] - hinge[0], leaf_p[:, 2] - hinge[1]
        # The left leaf (-X) turns its free edge from +X to +Z, the right
        # one from -X to +Z.
        if side < 0:
            leaf_p[:, 0], leaf_p[:, 2] = hinge[0] - dz, hinge[1] + dx
            leaf_n[:, 0], leaf_n[:, 2] = -normals[corners][:, 2], normals[corners][:, 0]
        else:
            leaf_p[:, 0], leaf_p[:, 2] = hinge[0] + dz, hinge[1] - dx
            leaf_n[:, 0], leaf_n[:, 2] = normals[corners][:, 2], -normals[corners][:, 0]
        # Down to the ground, on which the threshold they stood on was.
        leaf_p[:, 1] -= leaf_p[:, 1].min()
        p.append(leaf_p)
        n.append(leaf_n)
        u.append(uvs[corners])
        i.append(np.arange(count, count + len(corners), dtype=np.uint32))
        count += len(corners)
    return np.concatenate(p), np.concatenate(n), np.concatenate(u), np.concatenate(i).astype(np.uint32)


def frontmost(positions, indices, points):
    """How far forward (Z) the stone is at each of `points` (x, y), seen
    from the front: NaN where there is none."""
    corners = positions[indices.reshape(-1, 3)]
    a, b, c = corners[:, 0], corners[:, 1], corners[:, 2]
    found = np.full(len(points), np.nan)
    for k, (x, y) in enumerate(points):
        den = (b[:, 1] - c[:, 1]) * (a[:, 0] - c[:, 0]) + (c[:, 0] - b[:, 0]) * (a[:, 1] - c[:, 1])
        ok = np.abs(den) > 1e-12
        safe = np.where(ok, den, 1.0)
        u = ((b[:, 1] - c[:, 1]) * (x - c[:, 0]) + (c[:, 0] - b[:, 0]) * (y - c[:, 1])) / safe
        v = ((c[:, 1] - a[:, 1]) * (x - c[:, 0]) + (a[:, 0] - c[:, 0]) * (y - c[:, 1])) / safe
        w = 1.0 - u - v
        inside = ok & (u >= -1e-6) & (v >= -1e-6) & (w >= -1e-6)
        if inside.any():
            z = u * a[:, 2] + v * b[:, 2] + w * c[:, 2]
            found[k] = z[inside].max()
    return found


def ithildin(positions, indices):
    """The drawing of ithildin on the Doors of Durin (in metres, the doors
    open): an arch on the pillars, a tree climbing each, its branches
    spreading under the arch, the crown over its keystone and the seven
    stars round it on the rock; as ribbons laid on the stone just in front
    of it, or on the rock behind (the back of the gate)."""
    low, high = positions.min(axis=0), positions.max(axis=0)
    height = high[1] - low[1]
    pillar = 0.172 * DURIN_WIDER * height
    spring = 0.664 * height
    ring = high[1] - spring - 0.09 * height
    back = low[2]
    lines = []

    def arc(cx, cy, rx, ry, start, end, count=48):
        t = np.linspace(start, end, count)
        return np.stack([cx + rx * np.cos(t), cy + ry * np.sin(t)], axis=1)

    # The arch, down the pillars to their feet.
    lines.append(np.concatenate([
        [[-pillar, 0.04 * height]],
        arc(0.0, spring, pillar, ring, np.pi, 0.0),
        [[pillar, 0.04 * height]],
    ]))
    # A tree up each pillar, outside the arch's line (inside it, the leaves
    # open hide the pillar): its trunk, and its branches spreading up and in
    # over the arch.
    for side in (-1.0, 1.0):
        trunk = side * (pillar + 0.03 * height)
        lines.append(np.array([[trunk, 0.06 * height], [trunk, spring - 0.06 * height]]))
        for up, angle in [(0.0, 20.0), (0.04, 38.0), (0.08, 56.0)]:
            start = np.array([trunk, spring - 0.1 * height + up * height])
            t = np.linspace(0.0, 1.0, 16)[:, None]
            # Ending on the ring of the arch, inside its line.
            a = np.radians(angle if side > 0 else 180.0 - angle)
            end = np.array([0.88 * pillar * np.cos(a), spring + 0.88 * ring * np.sin(a)])
            bend = np.array([trunk, end[1]])
            lines.append((1 - t) ** 2 * start + 2 * t * (1 - t) * bend + t**2 * end)
    # The crown on the keystone.
    crown_y, crown = high[1] - 0.075 * height, 0.045 * height
    lines.append(np.array([
        [-crown, crown_y], [-crown, crown_y + crown], [-crown / 2, crown_y + crown * 0.55],
        [0.0, crown_y + crown * 1.15], [crown / 2, crown_y + crown * 0.55], [crown, crown_y + crown],
        [crown, crown_y], [-crown, crown_y],
    ]))
    # Ribbons, their pieces laid on the stone.
    vertices, triangles = [], []

    def lay(points):
        on = frontmost(positions, indices, points)
        return np.where(np.isnan(on), back, on) + ITHILDIN_OFF

    for line in lines:
        dense = [line[0]]
        for a, b in zip(line[:-1], line[1:]):
            pieces = max(1, int(np.ceil(np.linalg.norm(b - a) / ITHILDIN_STEP)))
            dense.extend(a + (b - a) * (k / pieces) for k in range(1, pieces + 1))
        dense = np.array(dense)
        ahead = np.gradient(dense, axis=0)
        ahead /= np.linalg.norm(ahead, axis=1, keepdims=True).clip(1e-9)
        side = np.stack([-ahead[:, 1], ahead[:, 0]], axis=1) * ITHILDIN_WIDTH / 2
        left, right = dense + side, dense - side
        zl, zr = lay(left), lay(right)
        z = np.maximum(zl, zr)
        start = len(vertices)
        for k in range(len(dense)):
            vertices.append([left[k, 0], left[k, 1], z[k]])
            vertices.append([right[k, 0], right[k, 1], z[k]])
        for k in range(len(dense) - 1):
            a = start + 2 * k
            triangles += [[a, a + 1, a + 3], [a, a + 3, a + 2]]
    # The seven stars, round the arch on the rock over it.
    for k in range(7):
        angle = np.radians(58.0 + k * 64.0 / 6.0)
        centre = np.array([0.0, spring]) + np.array(
            [np.cos(angle) * (pillar + 0.12 * height), np.sin(angle) * (ring + 0.13 * height)]
        )
        star = []
        for j in range(8):
            radius = 0.022 * height if j % 2 == 0 else 0.007 * height
            a = np.pi / 2 + j * np.pi / 4
            star.append(centre + radius * np.array([np.cos(a), np.sin(a)]))
        z = lay(np.array(star + [centre])).max()
        start = len(vertices)
        vertices.append([centre[0], centre[1], z])
        vertices += [[x, y, z] for x, y in star]
        triangles += [[start, start + 1 + j, start + 1 + (j + 1) % 8] for j in range(8)]
    vertices = np.array(vertices)
    # Facing the front (+Z), counter-clockwise from there.
    triangles = np.array(triangles, dtype=np.uint32)
    corners = vertices[triangles]
    facing = np.cross(corners[:, 1] - corners[:, 0], corners[:, 2] - corners[:, 0])[:, 2]
    triangles[facing < 0] = triangles[facing < 0][:, [0, 2, 1]]
    normals = np.tile([0.0, 0.0, 1.0], (len(vertices), 1))
    return vertices, normals, np.zeros((len(vertices), 2)), triangles.reshape(-1)


def doorway_dark(leaves):
    """The darkness of Moria in the doorway (in metres), where the leaves
    stood closed, from `DURIN_DARK_FROM` up into the arch: a little wider,
    its edges hidden in the stone, in the plane of their backs."""
    hull = convex_hull(leaves[:, :2])
    back = leaves[:, 2].min()
    middle = hull.mean(axis=0)
    hull = middle + (hull - middle) * np.array([1.04, 1.02])
    hull[:, 1] = np.maximum(hull[:, 1], DURIN_DARK_FROM)
    centre = np.array([0.0, (hull[:, 1].min() + hull[:, 1].max()) / 2.0])
    vertices = np.concatenate([[centre], hull])
    vertices = np.concatenate([vertices, np.full((len(vertices), 1), back)], axis=1)
    count = len(hull)
    triangles = np.array([[0, 1 + k, 1 + (k + 1) % count] for k in range(count)], dtype=np.uint32)
    normals = np.tile([0.0, 0.0, 1.0], (len(vertices), 1))
    return vertices, normals, np.zeros((len(vertices), 2)), triangles.reshape(-1)


def convex_hull(points):
    """The convex hull of `points` (x, y), counter-clockwise."""
    points = np.unique(points.round(4), axis=0)
    points = points[np.lexsort((points[:, 1], points[:, 0]))]

    def half(ordered):
        chain = []
        for point in ordered:
            while len(chain) >= 2:
                a, b = chain[-1] - chain[-2], point - chain[-2]
                if a[0] * b[1] - a[1] * b[0] > 0:
                    break
                chain.pop()
            chain.append(point)
        return chain

    lower, upper = half(points), half(points[::-1])
    return np.array(lower[:-1] + upper[:-1])


def turned(positions, normals):
    """Turned a quarter about Y, so that the side that was -X faces +Z."""
    turn = np.array([[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]])
    return positions @ turn.T, normals @ turn.T


def axle_hole(positions, normals, uvs, indices, color):
    """The middle of the dark hole for the axle in the mill's side that was
    -X (its stone wall, low): (z, y), in its own units."""
    triangles = indices.reshape(-1, 3)
    middle = positions[triangles].mean(axis=1)
    facing = normals[triangles].mean(axis=1)
    wall = (middle[:, 0] < -0.18) & (facing[:, 0] < -0.8) & (middle[:, 1] < -0.1)
    # The hole is painted, not modelled: the texture is looked up over the
    # wall, a point every 2 mm (z, y), in each of its triangles.
    dark = []
    for triangle in triangles[wall]:
        corners = positions[triangle][:, [2, 1]]
        low, high = corners.min(axis=0), corners.max(axis=0)
        z, y = np.meshgrid(np.arange(low[0], high[0], 0.002), np.arange(low[1], high[1], 0.002))
        points = np.stack([z.ravel(), y.ravel()], axis=1)
        a, b, c = corners
        den = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])
        if abs(den) < 1e-12 or len(points) == 0:
            continue
        u = ((b[1] - c[1]) * (points[:, 0] - c[0]) + (c[0] - b[0]) * (points[:, 1] - c[1])) / den
        v = ((c[1] - a[1]) * (points[:, 0] - c[0]) + (a[0] - c[0]) * (points[:, 1] - c[1])) / den
        inside = (u >= 0) & (v >= 0) & (u + v <= 1)
        weights = np.stack([u, v, 1 - u - v], axis=1)[inside]
        rgb = texels(weights @ uvs[triangle], color)
        dark.append(points[inside][rgb.mean(axis=1) < 0.16])
    hole = np.concatenate(dark)
    assert len(hole) > 50, f"{len(hole)} points of the hole"
    return hole[:, 0].mean(), hole[:, 1].mean()


def main():
    props = []
    for argument in sys.argv[1:]:
        name, path = argument.split("=", 1)
        if name not in SIZES and name != "mill_wheel":
            sys.exit(f"{name}: not one of {', '.join(SIZES)}, mill_wheel")
        props.append((name, path))
    if not props:
        sys.exit(__doc__)
    order = list(SIZES) + ["mill_wheel"]
    props.sort(key=lambda prop: order.index(prop[0]))
    given = {name: path for name, path in props}
    if ("mill" in given) != ("mill_wheel" in given):
        sys.exit("the mill and its wheel go together")
    rows = (len(props) + COLUMNS - 1) // COLUMNS
    atlas = Image.new("RGB", (COLUMNS * CELL, rows * CELL))
    size = np.array([COLUMNS * CELL, rows * CELL], dtype=np.float64)
    pieces = []
    mill = None
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

        if name == "stone_trolls":
            positions, normals, uvs, indices = without_third_leg(positions, normals, uvs, indices, color)
        elif name == "bree_gate":
            indices = without_bars(positions, indices)
        elif name == "bree_house_a":
            indices = without_sign(positions, indices)
        elif name == "elven_boat":
            indices = without_oars(positions, indices)
        moved = (corner + uvs.clip(0.0, 1.0) * inner) / size

        if name == "durin_doors":
            leaves, threshold = door_leaves(positions, indices)
            indices = keep_triangles(indices, ~threshold)
            leaves = leaves[~threshold]
            positions, normals = widened(positions, normals, DURIN_WIDER)
            _, _, centre, scale = place(positions[np.unique(indices)], normals[np.unique(indices)], SIZES[name])
            placed = (positions - centre) * scale
            bent = normals / scale
            bent /= np.linalg.norm(bent, axis=1, keepdims=True).clip(1e-9)
            closed = placed[indices.reshape(-1, 3)[leaves].reshape(-1)]
            triangles = indices.reshape(-1, 3)
            frame = (placed, triangles[~leaves].reshape(-1))
            opened = swung_open(placed, bent, moved, indices, leaves)
            for piece, (p, n, u, i) in (
                ("durin_doors", opened),
                ("durin_ithildin", ithildin(*frame)),
                ("durin_dark", doorway_dark(closed)),
            ):
                p, n, u, i = compact(p, n, u, i)
                pieces.append((piece, p, n, u, i))
                report(piece, p, i)
            continue

        if name == "mill":
            hole = axle_hole(positions, normals, uvs, indices, color)
            positions, normals = turned(positions, normals)
            # Its wall with the hole now faces +Z, the hole at (x, y) =
            # (its z, y) as it came.
            p, n, centre, scale = place(positions, normals, SIZES["mill"])
            axle = np.array([(hole[0] - centre[0]) * scale[0], (hole[1] - centre[1]) * scale[1], 0.0])
            # Its wall's face about the hole.
            about = (np.abs(p[:, 0] - axle[0]) < 2.0) & (np.abs(p[:, 1] - axle[1]) < 1.5)
            axle[2] = p[about, 2].max()
            mill = (p, n, moved, indices, axle)
            continue
        if name == "mill_wheel":
            assert mill is not None
            p_mill, n_mill, uv_mill, i_mill, axle = mill
            low, high = positions.min(axis=0), positions.max(axis=0)
            hub = (low + high) / 2.0
            diameter = 2.0 * (axle[1] + WHEEL_SUNK)
            scale = diameter / (high[1] - low[1])
            wheel = (positions - hub) * scale
            depth = (high[2] - low[2]) * scale
            # Against the wall, past anything on it about the hole.
            axle[2] += WHEEL_GAP + depth / 2.0
            for piece, p, n, u, i in (("mill", p_mill, n_mill, uv_mill, i_mill), ("mill_wheel", wheel + axle, normals, moved, indices)):
                p, n, u, i = compact(p - axle, n, u, i)
                pieces.append((piece, p, n, u, i))
                report(piece, p, i)
            continue
        _, _, centre, scale = place(positions[np.unique(indices)], normals[np.unique(indices)], SIZES[name])
        placed = (positions - centre) * scale
        bent = normals / scale
        bent /= np.linalg.norm(bent, axis=1, keepdims=True).clip(1e-9)
        p, n, u, i = compact(placed, bent, moved, indices)
        pieces.append((name, p, n, u, i))
        report(name, p, i)

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


def report(name, positions, indices):
    low, high = positions.min(axis=0), positions.max(axis=0)
    print(
        f"{name}: {len(positions)} vertices, {len(indices) // 3} triangles, "
        f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} m"
    )


if __name__ == "__main__":
    main()
