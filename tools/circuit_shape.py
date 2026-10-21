#!/usr/bin/env python3
"""Draws the centre line of a rounded circuit and writes its control points,
ready to paste into a circuit's source.

A circuit is a rounded rectangle: four straights joined by circular corners,
laid out on the ground plane, with the finish line at the origin and the
start straight running towards -Z (north), as the game expects. On top of
that it can lay a smooth S (a pair of esses) into a stretch of a straight,
for a twisty aisle.

It prints:
- the control points, as the Rust `Vec2` array of a circuit's
  `CONTROL_POINTS`, sampled every `SPACING` metres along the line and at
  the ends of each piece of it, where the line turns (so the curve the game
  draws turns where this one does);
- the distance from the finish line of each of them, to place a section of
  the circuit's `ELEVATION` profile or its scenery along it;
- the pieces of the line, with the stretch of the lap each one covers.

Usage:
    python3 tools/circuit_shape.py            # the shape below
    python3 tools/circuit_shape.py --plan plan.png

The shape is set at the top of the file: the straights, the radius of each
corner, and the esses. Change them and run it again.

Needs numpy (and matplotlib for the picture).
"""

import argparse
import math
import sys

# The shape of the circuit: the straights and the corners, in order, from
# the finish line. The first straight runs north (-Z) from the origin.
#
# Each piece is one of:
#   ("line", (x0, z0), (x1, z1))          a straight from one point to another
#   ("arc", (x, z), radius, a0, a1)        a circular corner, the angles in
#                                          degrees, measured from +x towards
#                                          +z, and the piece drawn from a0
#                                          to a1
SHAPE = [
    ("line", (0.0, 0.0), (0.0, -420.0)),
    ("arc", (-100.0, -420.0), 100.0, 0.0, -90.0),
    ("line", (-100.0, -520.0), (-290.0, -520.0)),
    ("arc", (-290.0, -420.0), 100.0, -90.0, -180.0),
    ("line", (-390.0, -420.0), (-390.0, 50.0)),
    ("arc", (-290.0, 50.0), 100.0, 180.0, 90.0),
    ("line", (-290.0, 150.0), (-60.0, 150.0)),
    ("arc", (-60.0, 90.0), 60.0, 90.0, 0.0),
    ("line", (0.0, 90.0), (0.0, 0.0)),
]

# The esses: a smooth S laid into a stretch of the line, from `along`
# metres of the lap, over `length` metres, `amp` metres to the side.
ESSES = [(1550.0, 160.0, 7.0)]

# The distance between the control points (m), and how finely the line is
# worked out before that.
SPACING = 32.0
STEP = 1.0


def build(shape):
    """The line as a fine polyline, with the distance along it."""
    points = []
    along = 0.0
    for piece in shape:
        if piece[0] == "line":
            (x0, z0), (x1, z1) = piece[1], piece[2]
            length = math.hypot(x1 - x0, z1 - z0)
        else:
            (_, (cx, cz), radius, a0, a1) = piece
            length = abs(a1 - a0) * math.pi / 180.0 * radius
        steps = max(1, int(math.ceil(length / STEP)))
        for step in range(steps):
            t = step / steps
            if piece[0] == "line":
                (x0, z0), (x1, z1) = piece[1], piece[2]
                point = (x0 + (x1 - x0) * t, z0 + (z1 - z0) * t)
            else:
                (_, (cx, cz), radius, a0, a1) = piece
                angle = math.radians(a0 + (a1 - a0) * t)
                point = (cx + radius * math.cos(angle), cz + radius * math.sin(angle))
            points.append((point[0], point[1], along + length * t))
        along += length
    return points, along


def smoothstep(t):
    t = max(0.0, min(1.0, t))
    return t * t * (3.0 - 2.0 * t)


def esses(points, start, length, amp):
    """Shift the line sideways over a stretch, in a smooth S that starts and
    ends without twisting it."""
    out = []
    count = len(points)
    for index, (x, z, along) in enumerate(points):
        before = points[max(0, index - 1)]
        after = points[min(count - 1, index + 1)]
        dx, dz = after[0] - before[0], after[1] - before[1]
        length_here = math.hypot(dx, dz) or 1.0
        # To the left of the way the line runs.
        nx, nz = -dz / length_here, dx / length_here
        shift = 0.0
        if start <= along <= start + length:
            u = (along - start) / length
            shift = amp * math.sin(math.pi * u) ** 2 * math.sin(2 * math.pi * u)
        out.append((x + nx * shift, z + nz * shift, along))
    return out


def sample(points, spacing):
    """Points every `spacing` metres along the line."""
    out = []
    total = points[-1][2]
    count = int(round(total / spacing))
    index = 0
    for i in range(count):
        target = i * total / count
        while index + 1 < len(points) and points[index + 1][2] < target:
            index += 1
        a = points[index]
        b = points[min(index + 1, len(points) - 1)]
        t = 0.0 if b[2] == a[2] else (target - a[2]) / (b[2] - a[2])
        out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, target))
    return out


def turns(shape, points):
    """The ends of the pieces of the line, where it turns."""
    marks = []
    along = 0.0
    for piece in shape:
        if piece[0] == "line":
            (_, (x0, z0), (x1, z1)) = piece
            length = math.hypot(x1 - x0, z1 - z0)
        else:
            (_, (_, _), radius, a0, a1) = piece
            length = abs(a1 - a0) * math.pi / 180.0 * radius
        along += length
        for point in points:
            if abs(point[2] - along) < 0.6:
                marks.append(point)
    return marks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", help="a picture of the line from above (png)")
    args = parser.parse_args()

    points, total = build(SHAPE)
    for start, length, amp in ESSES:
        points = esses(points, start, length, amp)
    sampled = sample(points, SPACING)
    # The ends of the pieces of the line take the place of the sample nearest
    # them, so that the curve the game draws turns where this one does.
    for mark in turns(SHAPE, points):
        nearest = min(range(len(sampled)), key=lambda i: abs(sampled[i][2] - mark[2]))
        sampled[nearest] = mark
    sampled.sort(key=lambda p: p[2])
    final = []
    for point in sampled:
        if final and abs(point[2] - final[-1][2]) < 1.0:
            continue
        final.append(point)

    print(f"lap about {total:.0f} m, {len(final)} control points\n")
    print(f"const CONTROL_POINTS: [Vec2; {len(final)}] = [")
    for x, z, _ in final:
        print(f"    Vec2::new({x:.1f}, {z:.1f}),")
    print("];\n")
    print("// along:", ", ".join(f"{along:.0f}" for _, _, along in final))

    if args.plan:
        plot(final, args.plan)


def plot(points, path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    xs = [p[0] for p in points]
    zs = [p[1] for p in points]
    fig, ax = plt.subplots(figsize=(8, 10))
    ax.plot(xs, zs, color="#222222", linewidth=2)
    ax.plot(xs, zs, "o", color="#cc3311", markersize=3)
    ax.set_aspect("equal")
    ax.invert_yaxis()
    ax.grid(True, alpha=0.3)
    fig.savefig(path, dpi=110, bbox_inches="tight")
    print(f"plan: {path}")


if __name__ == "__main__":
    sys.exit(main())
