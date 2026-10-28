#!/usr/bin/env python3
"""Draws the Toy Store's lap as a turtle would (straights and arcs, with the
road's height marked along the way) and writes it out as the game wants it:
`CONTROL_POINTS` and `ELEVATION` for `src/circuits/toy_store.rs`.

The lap folds over itself (the start straight under the diorama table, the
Hot Wheels helix and the slide each under its own upper turn, the floor
under the first table), so it also finds where a stretch passes under
another (the circuit's `tunnels`, which the tests ask of every point with
another road within reach overhead) and checks what the tests check of the
centre line and the profile: the tightest bend, the steepest grade, distant
stretches clear of each other unless one passes more than 12 m over the
other, and the crests the bikes take off over at top speed (on straights
only). The centre line and the heights are worked out as `src/track.rs`
does, through `tools/circuit_layout.py`: keep the two in step.

Usage:
    python3 tools/toy_store_layout.py              # the checks
    python3 tools/toy_store_layout.py --emit       # the Rust arrays
    python3 tools/toy_store_layout.py --plan plan.png --profile profile.png
    python3 tools/toy_store_layout.py --pace 1220 2160 3410  # a rival's time

`--plan` draws the lap from above (north, -Z, up), coloured by height, with
its walls, the furniture of `src/circuits/toy_store.rs` and a label every
100 m; `--profile` its height and grade, the stretches under another shaded.
`--pace` estimates, with a rough model of a good rival (flat out on the
straights, as fast round a bend as its radius allows, gathering and losing
speed at fixed rates), its time over each stretch between the checkpoints
given, against what Hard gives for it: a first look, before the tests ride
it for real.

Needs numpy (and matplotlib for the pictures).
"""

import argparse
import math
import os
import re
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import circuit_layout as cl  # noqa: E402

WALL = cl.WALL_DISTANCE
# The heights (m over the floor) of the first table, the diorama, the top of
# the tall shelving unit and the train table.
T0, T1, TOP, T2 = 14.0, 15.5, 39.0, 9.0
CIRCUIT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "circuits", "toy_store.rs")


def forward(heading):
    """The way a heading points on the ground (x, z), as `geometry::forward`."""
    return np.array([-math.sin(heading), -math.cos(heading)])


class Turtle:
    """Walks the centre line: `S` straight on, `L` and `R` round an arc to
    the left or the right, `at` marks the road's height where it stands."""

    def __init__(self, every=12.0):
        self.p = np.zeros(2)
        self.h = 0.0
        self.points = [tuple(self.p)]
        self.every = every
        self.marks = []
        self.dist = 0.0

    def at(self, height, name=""):
        self.marks.append((name, self.dist, height, np.array(self.p)))
        return self

    def S(self, length):
        n = max(1, int(round(length / (2 * self.every))))
        for _ in range(n):
            self.p = self.p + forward(self.h) * (length / n)
            self.points.append(tuple(np.round(self.p, 2)))
        self.dist += length
        return self

    def arc(self, radius, degrees, sign):
        a = math.radians(degrees)
        n = max(2, int(round(radius * a / self.every)))
        centre = self.p + forward(self.h + math.pi / 2) * radius * sign
        for k in range(1, n + 1):
            h = self.h + sign * a * k / n
            self.p = centre - forward(h + math.pi / 2) * radius * sign
            self.points.append(tuple(np.round(self.p, 2)))
        self.h += sign * a
        self.dist += radius * a
        return self

    def L(self, radius, degrees):
        return self.arc(radius, degrees, 1)

    def R(self, radius, degrees):
        return self.arc(radius, degrees, -1)


def lap():
    """The toy store's lap."""
    t = Turtle()
    t.at(0, "start")
    # The drag strip, with its booster jump, and on under the diorama.
    t.S(40).at(0)
    t.S(15).at(0.05)
    t.S(20).at(0.9, "kicker")
    t.S(10).at(1.2)
    t.S(14).at(0.1)
    t.S(14).at(0.0)
    t.S(157).at(0, "north")
    t.R(70, 90)
    # Up onto the chair and on onto the first table.
    t.S(10).at(0, "ramp")
    t.S(12).at(0.4)
    t.S(50).at(5.5, "chair")
    t.S(36).at(5.5, "chair end")
    t.S(12).at(6.3)
    t.S(64).at(T0 - 0.6, "T0")
    t.S(22).at(T0)
    t.S(50).at(T0)
    t.R(70, 180).at(T0, "T0 back")
    # Across to the diorama, over the start straight.
    t.S(88).at(T0, "T0 edge")
    t.S(63).at(T1, "T1 edge")
    t.S(t.p[0] + 50).at(T1, "X")
    t.L(65, 90).at(T1)
    t.S(10).at(T1, "T1 south")
    t.S(10).at(T1, "HW")
    # The Hot Wheels helix, a turn and a half, evenly up.
    for k in range(6):
        t.R(70, 90).at(T1 + (TOP - 1.0 - T1) * (k + 1) / 6, "helix top" if k == 5 else "")
    # Along the top of the shelves.
    t.S(40).at(TOP, "gondola")
    t.S(24).at(TOP, "shelves")
    t.S(136).at(TOP, "shelves end")
    # The slide, a turn and a quarter down.
    t.S(50).at(TOP - 3.5, "slide")
    for k in range(5):
        t.R(62, 90).at(TOP - 3.5 - (TOP - 3.5 - 3.0) * (k + 1) / 5)
    t.S(40).at(0.3, "floor")
    t.S(20).at(0)
    # Along the north wall and south under the first table.
    t.S(256 - t.p[0]).at(0, "north leg end")
    t.R(70, 90).at(0)
    t.S(170).at(0, "under T0")
    # Up onto the train table, round on it and straight off its edge.
    t.S(12).at(0.4)
    t.S(66).at(7.7)
    t.S(34).at(T2, "T2")
    t.S(30).at(T2)
    t.R(65, 90).at(T2)
    t.S(t.p[0] - 195).at(T2)
    t.L(65, 90).at(T2)
    t.S(40).at(T2, "drop")
    t.S(90).at(0.3, "drop end")
    t.S(12).at(0)
    # Home.
    t.R(65, 180).at(0, "home")
    t.S(t.p[1])
    return t


def resolve(t):
    """The centre line (every 2 m) through the turtle's points, its spacing
    and length, and the profile: each mark where it falls along it."""
    points = np.array(t.points[:-1])
    line = cl.closed_curve(points, cl.SAMPLE_SPACING)
    n = len(line)
    loop = np.vstack([line, line[:1]])
    step = np.linalg.norm(np.diff(loop, axis=0), axis=1).sum() / n
    length = n * step
    scale = length / t.dist
    profile = {}
    for name, dist, height, p in t.marks:
        # Near where the walk says, the point of the line nearest the mark.
        guess = int(dist * scale / step)
        best = min(
            ((guess + k) % n for k in range(-60, 61)),
            key=lambda i: np.linalg.norm(line[i] - p),
        )
        along = round(best * step, 1)
        if along < length - 1.0:
            profile[along] = (along, height, name)
    profile = sorted(profile.values())
    heights = np.array(
        [
            cl.BASE_ELEVATION + cl.profile_height([(a, h) for a, h, _ in profile], length, i * step)
            for i in range(n)
        ]
    )
    return points, line, step, length, profile, heights


# Tunnels closer than this (m) are one: the trackside camera of the menus
# (`src/welcome.rs`) comes down over 25 m before a tunnel and goes back up
# over 25 m after it, and a test wants it back up 50 m off either end.
TUNNEL_GAP = 60.0


def under_another(line, step, heights):
    """The stretches with another more than 8 m over them within reach of
    their walls (from, to), those close together joined: the circuit's
    tunnels."""
    n = len(line)
    neighborhood = int(8 * WALL / cl.SAMPLE_SPACING)
    reach = 2 * WALL + 10
    under = np.zeros(n, bool)
    for i in range(n):
        others = np.array([j % n for j in range(i + neighborhood, i + n - neighborhood)])
        near = others[np.linalg.norm(line[others] - line[i], axis=1) < reach]
        under[i] = len(near) > 0 and (heights[near] - heights[i]).max() > 8.0
    tunnels = []
    for a, b in cl.runs(under):
        span = (round(a * step - 10), round(b * step + 10))
        if tunnels and span[0] - tunnels[-1][1] < TUNNEL_GAP:
            tunnels[-1] = (tunnels[-1][0], span[1])
        else:
            tunnels.append(span)
    return tunnels


def check(line, step, heights, tunnels):
    """What the tests check of the centre line and the profile."""
    n = len(line)
    prev, after = np.roll(line, 1, axis=0), np.roll(line, -1, axis=0)
    turn = (
        np.arctan2(*(after - line).T[::-1]) - np.arctan2(*(line - prev).T[::-1]) + np.pi
    ) % (2 * np.pi) - np.pi
    radius = step / np.maximum(np.abs(turn), 1e-9)
    grade = (np.roll(heights, -1) - heights) / step
    print(f"lap: {n * step:.1f} m, {n} points")
    print(f"tightest bend: {radius.min():.1f} m (over {cl.MIN_RADIUS:.2f})")
    steepest = int(np.argmax(np.abs(grade)))
    print(f"steepest grade: {grade[steepest] * 100:.1f} % at {steepest * step:.0f} m (under 13)")
    print(f"under another stretch (the tunnels): {tunnels}")
    along = np.arange(n) * step
    tunnel = np.zeros(n, bool)
    for a, b in tunnels:
        tunnel |= (along >= a) & (along <= b)
    neighborhood = int(8 * WALL / cl.SAMPLE_SPACING)
    touching = 0
    for i in range(n):
        others = np.arange(i + neighborhood, min(n, i + n - neighborhood))
        near = others[np.linalg.norm(line[others] - line[i], axis=1) < 2 * WALL + 10]
        for j in near:
            low, high = (i, j) if heights[i] < heights[j] else (j, i)
            touching += not (tunnel[low] and heights[high] - heights[low] > 12.0)
    print(f"distant stretches too close: {touching} pairs (none wanted)")
    k = int(round(cl.VERTICAL_WINDOW / step))
    vertical = (np.roll(heights, -k) - 2 * heights + np.roll(heights, k)) / (k * step) ** 2
    takeoff = cl.TOP_SPEED**2 / cl.RISE_GRAVITY
    print("crests the bike takes off over at top speed:")
    for first, last in cl.runs(vertical < -1.0 / takeoff):
        top = first + int(np.argmin(vertical[first : last + 1]))
        bend = "on a straight" if radius[top] > 1e4 else f"in a bend of {radius[top]:.0f} m"
        print(
            f"  {first * step:.0f}-{last * step:.0f} m: {-1.0 / vertical[top]:.0f} m round, "
            f"from {math.sqrt(cl.RISE_GRAVITY / -vertical[top]) * 3.6:.0f} km/h, {bend}"
        )


# A rough model of a good rival's pace (skill 0.9, as the checkpoint tests
# ride), fitted to the game's own: flat out at its top speed on the
# straights, in a bend no faster than its grip lets it round it (a speed
# that grows as the square root of the radius), and so fast it gathers
# speed and brakes (m/s, m/s²).
STRAIGHT_PACE = 71.0
BEND_PACE = 7.75
GATHER = 9.0
BRAKE = 14.0
# Hard's pace against the clock (`CheckpointTimes`, src/race/mod.rs): m/s,
# and the seconds spare at each checkpoint.
HARD_PACE = 66.0
HARD_SPARE = 1.5


def pace(line, step, checkpoints):
    """The seconds a good rival takes over each stretch between the
    checkpoints (metres along the lap, the line last), by the rough model
    above, and what Hard gives for each."""
    n = len(line)
    prev, after = np.roll(line, 3, axis=0), np.roll(line, -3, axis=0)
    turn = (
        np.arctan2(*(after - line).T[::-1]) - np.arctan2(*(line - prev).T[::-1]) + np.pi
    ) % (2 * np.pi) - np.pi
    radius = 3 * step / np.maximum(np.abs(turn), 1e-9)
    top = np.minimum(STRAIGHT_PACE, BEND_PACE * np.sqrt(radius))
    # Twice round, so the lap flows on into itself.
    v = np.tile(top, 2)
    for i in range(1, len(v)):
        v[i] = min(v[i], math.sqrt(v[i - 1] ** 2 + 2 * GATHER * step))
    for i in range(len(v) - 2, -1, -1):
        v[i] = min(v[i], math.sqrt(v[i + 1] ** 2 + 2 * BRAKE * step))
    v = v[n:]
    elapsed = np.concatenate([[0.0], np.cumsum(step / v)])
    lap = elapsed[-1]
    print(f"a good rival's flying lap: about {lap:.1f} s")
    marks = [0.0] + list(checkpoints)
    for a, b in zip(marks, marks[1:]):
        taken = elapsed[int(b / step) % (n + 1)] - elapsed[int(a / step)]
        given = (b - a) / HARD_PACE + HARD_SPARE
        print(f"  {a:6.0f}-{b:6.0f} m: {taken:5.1f} s ({(b - a) / taken:4.1f} m/s), Hard gives {given:5.1f} s")


def emit(points, profile):
    """`CONTROL_POINTS` and `ELEVATION` as Rust."""
    number = lambda v: "0.0" if f"{v:.1f}" == "-0.0" else f"{v:.1f}"
    print(f"const CONTROL_POINTS: [Vec2; {len(points)}] = [")
    for x, z in points:
        print(f"    Vec2::new({number(x)}, {number(z)}),")
    print("];\n")
    print(f"const ELEVATION: [(f32, f32); {len(profile)}] = [")
    for along, height, name in profile:
        note = f" // {name}" if name else ""
        print(f"    ({number(along)}, {float(height)!r}),{note}")
    print("];")


def furniture():
    """The furniture of the circuit's source: kind, middle, size."""
    text = open(CIRCUIT).read()
    pattern = (
        r"kind: FurnitureKind::(\w+),\s*center: Vec2::new\((-?[\d.]+), (-?[\d.]+)\),"
        r"\s*size: Vec2::new\((-?[\d.]+), (-?[\d.]+)\)"
    )
    return [
        (kind, float(x), float(z), float(w), float(d))
        for kind, x, z, w, d in re.findall(pattern, text)
    ]


def plot(points, line, step, heights, profile, tunnels, plan, profile_out):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    n = len(line)
    if plan:
        fig, ax = plt.subplots(figsize=(13, 13))
        for kind, x, z, w, d in furniture():
            ax.add_patch(plt.Rectangle((x - w / 2, z - d / 2), w, d, alpha=0.2, color="tab:brown"))
            ax.text(x, z, kind, fontsize=8, ha="center")
        tangent = np.roll(line, -1, axis=0) - np.roll(line, 1, axis=0)
        tangent /= np.linalg.norm(tangent, axis=1)[:, None]
        normal = np.stack([tangent[:, 1], -tangent[:, 0]], axis=1)
        dots = ax.scatter(*line.T, c=heights - cl.BASE_ELEVATION, s=4, cmap="viridis")
        fig.colorbar(dots, shrink=0.6, label="height (m)")
        for side in (1, -1):
            ax.plot(*(line + side * WALL * normal).T, "k:", lw=0.5)
        for along in np.arange(0, n * step, 100):
            i = int(along / step)
            ax.text(*(line[i] + 16 * normal[i]), f"{along:.0f}", fontsize=7, color="m", ha="center")
        ax.invert_yaxis()
        ax.set_aspect("equal")
        ax.grid(True, lw=0.3)
        fig.savefig(plan, dpi=80, bbox_inches="tight")
    if profile_out:
        fig, (top, bottom) = plt.subplots(2, 1, figsize=(14, 6), sharex=True)
        s = np.arange(n) * step
        top.plot(s, heights - cl.BASE_ELEVATION)
        top.plot([a for a, _, _ in profile], [h for _, h, _ in profile], "r.")
        for a, b in tunnels:
            top.axvspan(a, b, color="grey", alpha=0.2)
        top.set_ylabel("height (m)")
        bottom.plot(s, (np.roll(heights, -1) - heights) / step * 100)
        bottom.set_ylabel("grade (%)")
        for axis in (top, bottom):
            axis.grid(True, lw=0.3)
        fig.savefig(profile_out, dpi=70, bbox_inches="tight")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--emit", action="store_true", help="print the Rust arrays")
    parser.add_argument("--plan", help="draw the lap from above to this PNG")
    parser.add_argument("--profile", help="draw the height profile to this PNG")
    parser.add_argument(
        "--pace",
        nargs="*",
        type=float,
        metavar="CHECKPOINT",
        help="estimate a good rival's time over each stretch between these checkpoints",
    )
    args = parser.parse_args()
    points, line, step, length, profile, heights = resolve(lap())
    tunnels = under_another(line, step, heights)
    if args.emit:
        emit(points, profile)
    else:
        check(line, step, heights, tunnels)
    if args.pace is not None:
        pace(line, step, sorted(args.pace) + [length])
    if args.plan or args.profile:
        plot(points, line, step, heights, profile, tunnels, args.plan, args.profile)


if __name__ == "__main__":
    sys.exit(main())
