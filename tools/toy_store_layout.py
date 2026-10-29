#!/usr/bin/env python3
"""Draws the Toy Store's lap as a turtle would (straights and arcs, with the
road's height marked along the way), and its two branches (a way through a
castle on the diorama table, and a way round a loop of orange track beside
the north wall), and writes them out as the game wants them:
`CONTROL_POINTS`, `ELEVATION`, `CASTLE_WAY`, `LOOP_APPROACH`, `LOOP_EXIT`
and the spans and the loop's mouth for `src/circuits/toy_store.rs`.

The road is narrow (its walls `WALL` from the centre line, as the circuit's
`Layout::wall`) and twists. The lap folds over itself (the start straight
under the diorama table, the Hot Wheels helix and the slide each under its
own upper turn, the floor under the first table), so it also finds where a
stretch passes under another (the circuit's `tunnels`, which the tests ask
of every point with another road within reach overhead) and checks what the
tests check of the centre line and the profile: the tightest bend, the
steepest grade, distant stretches clear of each other unless one passes more
than 12 m over the other, the crests the bikes take off over at top speed
(on straights only), and the branches' bends and how far they part from the
lap. The centre line and the heights are worked out as `src/track.rs` does,
through `tools/circuit_layout.py`: keep the two in step. The banking is left
out, and with it the lift of the bends on the floor and the furniture,
which bank raised (their middles up to about 1.3 m higher, eased in and out:
see `Banking::raised`); the game's tests check the grades and crests with
it. The way through the castle and the lap beside it are mirror images, as
long and as gently bent, so that neither way is quicker.

Usage:
    python3 tools/toy_store_layout.py              # the checks
    python3 tools/toy_store_layout.py --emit       # the Rust arrays
    python3 tools/toy_store_layout.py --plan plan.png --profile profile.png
    python3 tools/toy_store_layout.py --pace 760 1500 2140  # a rival's time

`--plan` draws the lap and its branches from above (north, -Z, up),
coloured by height, with their walls, the furniture of
`src/circuits/toy_store.rs` and a label every 100 m; `--profile` its height
and grade, the stretches under another shaded. `--pace` estimates, with a
rough model of a good rival (flat out on the straights, as fast round a bend
as its radius allows, gathering and losing speed at fixed rates), its time
over each stretch between the checkpoints given, and the pace that is (the
circuit's `pace` is about what that is of Hard's): a first look, before the
tests ride it for real.

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

# How far the walls stand from the centre line (m), as `Layout::wall`.
WALL = 7.5
# The heights (m over the floor) of the chair's seat, the first table, the
# diorama, the top of the tall shelving unit and the train table.
CHAIR, T0, T1, TOP, T2 = 5.5, 14.0, 14.0, 39.0, 9.0
# The loop: its radius, how far to the right its way out is from its way in
# (m), and the straight leads into it and out of it the game lays (m, as
# `LOOP_LEAD` in `src/track.rs`), two each.
LOOP_RADIUS, LOOP_SHIFT, LOOP_LEAD = 14.0, 14.0, 25.0
# How far along the north wall the way round the loop runs beside the lap
# (m, from its fork to its join); how far the two run together from the fork
# before parting; and how far to the left (north, toward the wall) the lap
# swings away while the way to the loop goes straight on into it, so that it
# makes no detour to it: as far as the row of blocks along the lap's left
# leaves room for before the shelves.
LOOP_SPAN, LOOP_PARTING, LOOP_SWING = 242.0, 10.0, 16.0
# The bends either side of the swing, and how far round each turns (°).
SWING_BEND = 80.0
SWING_ANGLE = math.degrees(math.acos(1 - LOOP_SWING / (2 * SWING_BEND)))
# How far to the right (south) the way to the loop drifts off the lap's
# line, in two wide bends, for its walls to part from the lap's (2 m apart):
# all but straight.
LOOP_DRIFT, DRIFT_BEND = 2.5, 200.0
DRIFT_ANGLE = math.degrees(math.acos(1 - LOOP_DRIFT / (2 * DRIFT_BEND)))
# The bends the way out of the loop comes back onto the lap's line by,
# from the way out's side: how far round each turns (°).
LOOP_BACK_BEND = 40.0
LOOP_BACK_ANGLE = math.degrees(
    math.acos(1 - (LOOP_SHIFT + LOOP_DRIFT) / (2 * LOOP_BACK_BEND))
)
# How far along from the fork the loop's mouth is: as far as leaves room
# after the way out for its lead out, its bends back and a last straight.
LOOP_MOUTH_AT = (
    LOOP_SPAN
    - 10.0
    - 2 * LOOP_BACK_BEND * math.sin(math.radians(LOOP_BACK_ANGLE))
    - 2 * LOOP_LEAD
)
# The radius of the Hot Wheels helix (m): wide enough for a pack to climb it
# side by side.
HELIX_RADIUS = 36.0
# The way through the castle: straight on along the lap at first and last
# (m), out to the side and back in bends of this radius (m) and angle (°),
# gently, and straight through the castle (m); as far along the lap as it
# runs beside it. The lap beside it does the same to the other side, its
# mirror image: neither way is the longer or the slower.
CASTLE_LEAD, CASTLE_BEND, CASTLE_ANGLE, CASTLE_THROUGH = 10.0, 100.0, 20.7, 20.0
CASTLE_SPAN = 2 * CASTLE_LEAD + 4 * CASTLE_BEND * math.sin(math.radians(CASTLE_ANGLE)) + CASTLE_THROUGH
CIRCUIT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "circuits", "toy_store.rs")


def forward(heading):
    """The way a heading points on the ground (x, z), as `geometry::forward`."""
    return np.array([-math.sin(heading), -math.cos(heading)])


class Turtle:
    """Walks the centre line: `S` straight on, `L` and `R` round an arc to
    the left or the right, `at` marks the road's height where it stands.
    A point every `every` metres round an arc, twice as far apart on a
    straight: close enough that the curve the game draws through them
    (centripetal Catmull-Rom, its tangent smooth but not its bending) keeps
    to the arcs, with no wobble between points where they lie unevenly (as
    they did 12 m apart, the road snaking in the train table's esses)."""

    def __init__(self, every=6.0):
        self.p = np.zeros(2)
        self.h = 0.0
        self.points = [tuple(self.p)]
        self.every = every
        self.marks = []
        self.dist = 0.0
        self.kept = {}

    def keep(self, name):
        """Notes where it is and which way it heads, by `name`."""
        self.kept[name] = (np.array(self.p), self.h)
        return self

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
    # The drag strip, with its booster's kicker, and on under the diorama.
    t.S(40).at(0)
    t.S(15).at(0.05)
    t.S(20).at(0.9, "kicker")
    t.S(10).at(1.2)
    t.S(14).at(0.1)
    t.S(14).at(0.0)
    t.S(57).at(0, "north")
    t.R(32, 90)
    # Up onto the chair and on onto the first table.
    t.S(8).at(0, "ramp")
    t.S(10).at(0.4)
    t.S(50).at(CHAIR, "chair")
    t.S(20).at(CHAIR, "chair end")
    t.S(8).at(CHAIR + 0.8)
    # Short of the top before the table, as the lift of the esses on it,
    # eased in up the ramp, steepens it; and over the top gently, no crest
    # to take off over into them.
    t.S(60).at(T0 - 1.5, "T0")
    t.S(14).at(T0 - 0.6)
    t.S(12).at(T0 - 0.15)
    t.S(12).at(T0)
    # Esses across it and a hairpin round its far end.
    t.L(35, 40).at(T0)
    t.R(35, 80).at(T0)
    t.L(35, 40).at(T0)
    t.S(8).at(T0)
    t.R(30, 180).at(T0, "hairpin")
    t.S(40).at(T0, "T0 edge")
    # Across to the diorama; round its castle (a way through it, see
    # `castle_way`, while the lap swings out the other way, as far and as
    # gently) and over the start straight.
    t.S(36).at(T1, "T1 edge")
    t.S(40).at(T1)
    t.keep("castle fork")
    t.S(CASTLE_LEAD)
    t.R(CASTLE_BEND, CASTLE_ANGLE)
    t.L(CASTLE_BEND, CASTLE_ANGLE)
    t.S(CASTLE_THROUGH / 2).at(T1, "castle")
    t.S(CASTLE_THROUGH / 2)
    t.L(CASTLE_BEND, CASTLE_ANGLE)
    t.R(CASTLE_BEND, CASTLE_ANGLE)
    t.S(CASTLE_LEAD).at(T1, "castle join")
    t.keep("castle join")
    # Swinging right and left over the start straight.
    t.R(60, 20).at(T1)
    t.L(60, 40).at(T1)
    t.R(60, 20).at(T1)
    t.S(t.p[0] + 70).at(T1)
    t.S(40).at(T1, "X")
    # The Hot Wheels helix, a turn and a quarter, evenly up.
    for k in range(5):
        t.R(HELIX_RADIUS, 90).at(T1 + (TOP - 2.0 - T1) * (k + 1) / 5, "helix top" if k == 4 else "")
    # Along the top of the shelves.
    t.S(30).at(TOP, "gondola")
    t.S(32).at(TOP, "shelves end")
    # The slide, two turns and a quarter down round its tower.
    t.S(30).at(TOP - 2.0, "slide")
    for k in range(9):
        t.R(30, 90).at(TOP - 2.0 - (TOP - 4.5) * (k + 1) / 9)
    t.S(30).at(0.3, "floor")
    t.S(36).at(0)
    # East along the north wall, swinging out toward it while the way to
    # the loop goes straight on into it (see `loop_way`), and back.
    t.keep("loop fork")
    t.S(LOOP_PARTING).at(0, "loop fork")
    t.L(SWING_BEND, SWING_ANGLE).at(0)
    t.R(SWING_BEND, SWING_ANGLE).at(0)
    swing = 4 * SWING_BEND * math.sin(math.radians(SWING_ANGLE))
    t.S(LOOP_SPAN - LOOP_PARTING - swing).at(0)
    t.R(SWING_BEND, SWING_ANGLE).at(0)
    t.L(SWING_BEND, SWING_ANGLE).at(0, "loop join")
    t.keep("loop join")
    t.S(40).at(0, "north wall end")
    t.R(32, 90).at(0)
    # South under the first table, esses down the east side and up onto the
    # train table.
    t.S(100).at(0, "under T0")
    t.L(40, 35).at(0)
    t.R(40, 70).at(0)
    t.L(40, 35).at(0)
    t.S(10).at(0.3)
    t.S(76).at(T2 - 0.6, "T2")
    t.S(40).at(T2)
    # Round on it past its train, and straight off its edge.
    t.R(32, 90).at(T2)
    t.S(6).at(T2)
    t.L(35, 30).at(T2)
    t.R(35, 60).at(T2)
    t.L(35, 30).at(T2)
    t.S(25).at(T2, "drop")
    t.S(80).at(0.3, "drop end")
    t.S(t.p[0] - 40).at(0)
    # Home.
    t.R(40, 90).at(0, "home")
    t.S(t.p[1])
    return t


def branch_from(t, name):
    """A turtle where the lap's turtle `t` was at the point kept as `name`."""
    b = Turtle()
    b.p, b.h = t.kept[name]
    b.points = [tuple(b.p)]
    return b


def castle_way(t):
    """The way through the castle, as it leaves the lap at the castle's fork
    (where the lap swings north): straight on, curving south, straight
    through the castle, and back onto the lap at its join. Its control
    points (the first and last on the lap left to the game) and the ends of
    its straight through the castle."""
    b = branch_from(t, "castle fork")
    # Its leads off the lap and back onto it in short pieces, so that the
    # game's curve keeps them straight: it sets which way the way leaves
    # and joins the lap by the lap's points 20 m off, which past the join
    # are already round its next bend.
    for _ in range(3):
        b.S(CASTLE_LEAD / 3)
    b.L(CASTLE_BEND, CASTLE_ANGLE)
    b.R(CASTLE_BEND, CASTLE_ANGLE)
    gate = np.array(b.p)
    b.S(CASTLE_THROUGH)
    through = (gate, np.array(b.p))
    b.R(CASTLE_BEND, CASTLE_ANGLE)
    b.L(CASTLE_BEND, CASTLE_ANGLE)
    for _ in range(3):
        b.S(CASTLE_LEAD / 3)
    return np.array(b.points[1:-1]), through


def loop_way(t):
    """The way round the loop: from its fork on along the lap's line,
    drifting a little off it (the lap swinging away to the north), into the
    loop's mouth, and
    from its way out (to the right of the mouth) straight on and back onto
    the lap at its join. The control points of the road in (the last of
    them where the game's straight lead into the mouth begins) and out (the
    first where its lead out ends), the mouth and the heading."""
    b = branch_from(t, "loop fork")
    b.S(LOOP_PARTING)
    b.R(DRIFT_BEND, DRIFT_ANGLE)
    b.L(DRIFT_BEND, DRIFT_ANGLE)
    drift = 2 * DRIFT_BEND * math.sin(math.radians(DRIFT_ANGLE))
    b.S(LOOP_MOUTH_AT - 2 * LOOP_LEAD - LOOP_PARTING - drift)
    approach = np.array(b.points[1:-1])
    b.S(2 * LOOP_LEAD)
    mouth, heading = np.array(b.p), b.h
    way_out = mouth + forward(heading - math.pi / 2) * LOOP_SHIFT
    e = Turtle()
    e.p, e.h = way_out + forward(heading) * 2 * LOOP_LEAD, heading
    e.points = [tuple(e.p)]
    e.L(LOOP_BACK_BEND, LOOP_BACK_ANGLE)
    e.R(LOOP_BACK_BEND, LOOP_BACK_ANGLE)
    e.S(10)
    exit_ = np.array(e.points[1:-1])
    return approach, mouth, (heading + math.pi) % (2 * math.pi) - math.pi, exit_


def along_of(line, step, point):
    """How far along the lap (m) the point of its centre line nearest
    `point` is."""
    return round(int(np.argmin(np.linalg.norm(line - point, axis=1))) * step, 1)


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
    print(f"tightest bend: {radius.min():.1f} m (over {2.5 * WALL:.2f})")
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


def open_curve(points, before, after, spacing):
    """Points about `spacing` apart along the open curve through `points`,
    `before` and `after` setting which way it leaves the first and reaches
    the last, as `open_curve` in `src/track.rs`."""
    every = np.vstack([before, points, after])
    u = np.arange(512) / 512.0
    dense = np.concatenate(
        [catmull_rom_at(every, i, u) for i in range(len(every) - 3)] + [every[-2:-1]]
    )
    lengths = np.concatenate([[0.0], np.cumsum(np.linalg.norm(np.diff(dense, axis=0), axis=1))])
    count = max(1, int(round(lengths[-1] / spacing)))
    targets = np.arange(count + 1) * lengths[-1] / count
    return np.stack(
        [np.interp(targets, lengths, dense[:, 0]), np.interp(targets, lengths, dense[:, 1])], axis=1
    )


def catmull_rom_at(points, i, u):
    """The centripetal Catmull-Rom curve between `points[i + 1]` and
    `points[i + 2]` at `u`."""
    return cl.catmull_rom(points[i], points[i + 1], points[i + 2], points[i + 3], u)


def branch_line(points, first, last, before, after):
    """A branch's centre line (every 2 m) through its control points, from
    `first` to `last`, as the game lays it."""
    return open_curve(np.vstack([first, points, last]), before, after, cl.SAMPLE_SPACING)


def check_branches(t, line):
    """What the tests check of the branches: bends, and how far they part
    from the lap (their walls apart, and rock between a fork and the lap)."""
    way, _ = castle_way(t)
    approach, mouth, heading, exit_ = loop_way(t)
    ahead = forward(heading) * LOOP_LEAD
    way_out = mouth + forward(heading - math.pi / 2) * LOOP_SHIFT
    # The lap's points 20 m before a fork and after a join set which way a
    # branch leaves it and joins it again, as the game's.
    lap_ahead = lambda name, metres: t.kept[name][0] + forward(t.kept[name][1]) * metres
    roads = {
        "castle way": branch_line(
            way,
            t.kept["castle fork"][0],
            t.kept["castle join"][0],
            lap_ahead("castle fork", -20),
            lap_ahead("castle join", 20),
        ),
        "into the loop": branch_line(
            np.vstack([approach, mouth - 2 * ahead, mouth - ahead]),
            t.kept["loop fork"][0],
            mouth,
            lap_ahead("loop fork", -20),
            mouth + ahead,
        ),
        "out of the loop": branch_line(
            np.vstack([way_out + ahead, way_out + 2 * ahead, exit_]),
            way_out,
            t.kept["loop join"][0],
            way_out - ahead,
            lap_ahead("loop join", 20),
        ),
    }
    for name, road in roads.items():
        prev, after = road[:-2], road[2:]
        turn = (
            np.arctan2(*(after - road[1:-1]).T[::-1]) - np.arctan2(*(road[1:-1] - prev).T[::-1]) + np.pi
        ) % (2 * np.pi) - np.pi
        radius = cl.SAMPLE_SPACING / np.maximum(np.abs(turn), 1e-9)
        apart = np.array([np.linalg.norm(line - p, axis=1).min() for p in road])
        print(
            f"{name}: {len(road) * cl.SAMPLE_SPACING:.0f} m, tightest bend {radius.min():.1f} m, "
            f"furthest from the lap {apart.max():.1f} m (walls apart past {2 * WALL + 2:.0f} m, "
            f"rock between past {2 * WALL + 10:.0f} m)"
        )


def emit(points, profile):
    """`CONTROL_POINTS` and `ELEVATION` as Rust: the points to the
    centimetre (to the decimetre, 6 m apart, they would bend the curve
    through them out of true)."""
    number = lambda v: "0.0" if f"{v:.1f}" == "-0.0" else f"{v:.1f}"
    print(f"const CONTROL_POINTS: [Vec2; {len(points)}] = [")
    for x, z in points:
        print(f"    Vec2::new({centimetres(x)}, {centimetres(z)}),")
    print("];\n")
    print(f"const ELEVATION: [(f32, f32); {len(profile)}] = [")
    for along, height, name in profile:
        note = f" // {name}" if name else ""
        print(f"    ({number(along)}, {round(float(height), 2)!r}),{note}")
    print("];")


def centimetres(v):
    """`v` (m) as Rust, to the centimetre."""
    text = f"{v:.2f}".rstrip("0")
    text = text + "0" if text.endswith(".") else text
    return "0.0" if text == "-0.0" else text


def emit_branches(t, line, step):
    """The branches' spans, control points and loop as Rust."""
    number = lambda v: "0.0" if f"{v:.1f}" == "-0.0" else f"{v:.1f}"
    points = lambda name, array: (
        f"const {name}: [Vec2; {len(array)}] = [\n"
        + "".join(f"    Vec2::new({centimetres(x)}, {centimetres(z)}),\n" for x, z in array)
        + "];"
    )
    way, through = castle_way(t)
    approach, mouth, heading, exit_ = loop_way(t)
    fork = lambda name: along_of(line, step, t.kept[name][0])
    print(f"const CASTLE: Span = span({number(fork('castle fork'))}, {number(fork('castle join'))});")
    print(f"const LOOP: Span = span({number(fork('loop fork'))}, {number(fork('loop join'))});")
    print(points("CASTLE_WAY", way))
    print(f"// Through the castle: from {through[0].round(1)} to {through[1].round(1)}.")
    print(points("LOOP_APPROACH", approach))
    print(f"const LOOP_MOUTH: Vec2 = Vec2::new({number(mouth[0])}, {number(mouth[1])});")
    print(f"const LOOP_HEADING: f32 = {heading:.4f};")
    print(points("LOOP_EXIT", exit_))


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


def plot(t, points, line, step, heights, profile, tunnels, plan, profile_out):
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
        way, _ = castle_way(t)
        approach, mouth, heading, exit_ = loop_way(t)
        for branch in (way, approach, exit_):
            ax.plot(*branch.T, "r.-", lw=0.8, ms=3)
        ax.plot(*mouth, "bx")
        ax.plot(*(mouth + forward(heading - math.pi / 2) * LOOP_SHIFT), "bx")
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
    walk = lap()
    points, line, step, length, profile, heights = resolve(walk)
    tunnels = under_another(line, step, heights)
    if args.emit:
        emit(points, profile)
        emit_branches(walk, line, step)
    else:
        check(line, step, heights, tunnels)
        check_branches(walk, line)
    if args.pace is not None:
        pace(line, step, sorted(args.pace) + [length])
    if args.plan or args.profile:
        plot(walk, points, line, step, heights, profile, tunnels, args.plan, args.profile)


if __name__ == "__main__":
    sys.exit(main())
