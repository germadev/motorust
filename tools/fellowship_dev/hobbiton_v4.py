"""Hobbiton's fourth design on Fellowship Road (see
`RECORD/2026-10-16.fellowship-track.WIP.md`, section 16): its centre line
and where its scenery stands.

The centre line, from the end of the start straight (0, -110) heading
north (-Z) to the Shire (z = -571) heading east, is drawn by integrating a
piecewise linear curvature: a long sweep to the right (R 180, 76 degrees)
down to the Water, the bridge's straight, a sweep to the left (R 150, 73
degrees) round the village's hill and one to the right past Bag End's
hill, with 30 m ramps of curvature between straights and arcs and between
opposite arcs, so that the spline through the points (every 13 m, to the
centimetre) has no kinks. The last arc's radius and length are found
(Newton) so that it ends heading east on the Shire's line, where the lap
goes on as before (18.9 m shorter than with the third design).

Its scenery is placed by offsets from that line (m along it, m to its
left) or where it stands: the Water, 64 m wide, along the left of the
first sweep, square under the bridge and along the right of the second,
into the pool and on into the Brandywine; the hills of the village, each
checked to rise no more than 6 m at the road and 1.2 m at the water, and
to be no steeper than 35 degrees; the hobbit holes on the left low and
spread round the foot of the first hill; those on the right on the south
flank of the second, low, high and low; all facing the riders coming; the
Green Dragon on the other side of the road, on the outside of the last
bend, near it, raised over it and facing the riders, with Bag End and the
party tree past it,
none of them hiding another seen from the road (`clear`); and the other
buildings, each checked to be seen ahead from the road as the game clears
the view to its landmarks (`ahead`: 220 m, 0.6 rad).

Run: `python3 tools/fellowship_dev/hobbiton_v4.py [name=value ...]` from the
repository root prints the last arc, the lap's length and its tightest
radii, whether the centre line matches the file's, the hills' heights at
the road and the water, and where each building stands and from where it
is seen ahead. Needs numpy, and `model.py`, `parse_rs.py` beside it; it
reads the control points and the profile from
`src/circuits/fellowship_road.rs`.
"""
import sys, math
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import numpy as np
from model import Lap
from parse_rs import array

SRC = open(Path(__file__).parents[2] / "src/circuits/fellowship_road.rs").read()
CP = array(SRC, "CONTROL_POINTS")
EL = array(SRC, "ELEVATION")
# The Shire's first point past Hobbiton: the lap goes on from there.
J = CP.index((555.2, -569.0))
Z_SHIRE = -571.0
DEC = 2

def fwd(h):
    return np.array([-math.sin(h), -math.cos(h)])

def integrate(knots, z0, ds=0.25):
    """knots: [(s, k)] curvature (1/m, right positive) at distances s, linear
    between. Returns points every ds, headings (game: right turn subtracts)."""
    total = knots[-1][0]
    s_vals = np.arange(0.0, total + 1e-9, ds)
    ks = np.interp(s_vals, [a for a, _ in knots], [b for _, b in knots])
    p = np.array([0.0, -z0]); h = 0.0
    pts = [p.copy()]; hs = [h]
    for k in ks[:-1]:
        p = p + fwd(h - k * ds / 2) * ds
        h -= k * ds
        pts.append(p.copy()); hs.append(h)
    return np.array(pts), np.array(hs), s_vals, ks

def knots_for(P, RD, LD):
    T = P["ramp"]
    kA, kC, kD = 1 / P["RA"], -1 / P["RC"], 1 / RD
    s = 0.0; out = [(0.0, 0.0)]
    def add(length, k):
        nonlocal s
        s += length; out.append((s, k))
    add(T, kA)                 # into A
    add(P["LA"], kA)           # A, to the right
    add(T, 0.0)                # out of A
    add(P["bridge"], 0.0)      # the bridge's straight
    add(T, kC)                 # into C
    add(P["LC"], kC)           # C, to the left
    add(P["TCD"], kD)          # C into D
    add(LD, kD)                # D, to the right
    add(T, 0.0)                # out of D
    return out

def solve(P):
    """Find RD, LD: end heading -pi/2 (east) and end z on the Shire's line."""
    def err(RD, LD):
        pts, hs, _, _ = integrate(knots_for(P, RD, LD), P["z0"])
        return np.array([hs[-1] + math.pi / 2, pts[-1][1] - Z_SHIRE]), pts
    x = np.array([200.0, 150.0])
    for _ in range(40):
        e, pts = err(*x)
        if np.abs(e).max() < 1e-4:
            break
        J_ = np.zeros((2, 2)); d = [0.5, 0.5]
        for i in range(2):
            x2 = x.copy(); x2[i] += d[i]
            J_[:, i] = (err(*x2)[0] - e) / d[i]
        x = x - np.linalg.solve(J_, e)
    return x, pts

def control_points(P, every=13.0):
    (RD, LD), _ = solve(P)
    pts, hs, s_vals, ks = integrate(knots_for(P, RD, LD), P["z0"])
    n = max(1, int(round(s_vals[-1] / every)))
    idx = [int(round(i * (len(pts) - 1) / n)) for i in range(1, n + 1)]
    inner = [(round(float(pts[i][0]), DEC), round(float(pts[i][1]), DEC)) for i in idx]
    return inner, dict(RD=RD, LD=LD, end=pts[-1], pts=pts, s=s_vals, ks=ks)

DEFAULT = dict(z0=110.0, ramp=30.0, RA=180.0, LA=210.0, bridge=70.0, RC=150.0, LC=140.0, TCD=40.0)

def lap_for(P, every=13.0):
    inner, info = control_points(P, every)
    start = [(0.0, 0.0), (0.0, -50.0), (0.0, -100.0), (0.0, -P["z0"])]
    cp = start + inner + CP[J:]
    return cp, Lap(cp, EL), info

CP_NEW, LAP, INFO = lap_for(DEFAULT)

def pos(along, lateral=0.0):
    return LAP.offset(along, lateral)

def heading(along):
    i = LAP.index_at(along)
    t = LAP.line[(i + 1) % LAP.n] - LAP.line[(i - 1) % LAP.n]
    return math.atan2(-t[0], -t[1])

def road_d(p):
    return float(np.min(np.linalg.norm(LAP.line - np.asarray(p, float), axis=1)))

# The Water: offsets of the line (m along, m left), 32 m from its middle to
# its banks; square under the bridge; away from the road into the pool, and
# out of it into the Brandywine.
WL, WW = 60.0, 32.0
CROSS0, CROSS1 = 392.0, 438.0
def water_lateral(a):
    if a <= CROSS0:
        return WL
    if a >= CROSS1:
        return -WL
    return WL * math.cos(math.pi * (a - CROSS0) / (CROSS1 - CROSS0))
WATER_UP = [((-150.0, 40.0), 8.0), ((-115.0, -60.0), 9.0), ((-75.0, -110.0), 12.0)]
WATER_ALONG = [(130.0, 16.0), (165.0, 22.0), (200.0, 28.0), (240.0, WW), (280.0, WW), (320.0, WW),
               (355.0, WW), (380.0, WW), (395.0, WW), (405.0, WW), (415.0, WW), (425.0, WW),
               (435.0, WW), (450.0, WW), (470.0, WW), (495.0, WW)]
POND, POND_R = np.array([470.0, -330.0]), 30.0
CHANNEL = [((505.0, -332.0), 16.0), ((548.0, -330.0), 18.0)]
BRANDYWINE = [((600, -1100), 18), ((570, -800), 18), ((560, -568), 18), ((550, -330), 18), ((520, -200), 40)]

def water_points():
    pts = [(np.array(p), w) for p, w in WATER_UP]
    pts += [(pos(a, water_lateral(a)), w) for a, w in WATER_ALONG]
    pts += [(np.array([440.0, -345.0]), WW), (POND, POND_R)]
    return pts + [(np.array(p), w) for p, w in CHANNEL]

def river_d(p, pts):
    best = 1e9
    for (a, wa), (b, wb) in zip(pts, pts[1:]):
        a = np.asarray(a, float); b = np.asarray(b, float); ab = b - a
        t = np.clip((p - a) @ ab / (ab @ ab), 0, 1)
        best = min(best, np.linalg.norm(p - (a + ab * t)) - (wa + (wb - wa) * t))
    return best

def water_d(p):
    p = np.asarray(p, float)
    return min(river_d(p, water_points()), np.linalg.norm(p - POND) - POND_R, river_d(p, BRANDYWINE))

def shape(reach):
    """A climbed massif's height, as a share of its own, at `reach` (its
    distance from its middle over its radius), as `terrain.rs` has it."""
    t = min(max((reach - 0.45) / 0.55, 0.0), 1.0)
    return 1.0 - t * t * (3.0 - 2.0 * t)

# The hills: (middle, radius, height).
HILL_A = (pos(470.0, 200.0), 120.0, 28.0)
HILL_B = (np.array([440.0, -470.0]), 80.0, 20.0)
BAG_HILL = (pos(770.0, 190.0), 170.0, 32.0)

def on_hill(hill, angle, reach):
    """A point on a hill's flank: `angle` degrees from north (-Z) through
    east (180 due south), at `reach` of its radius."""
    c, r, _ = hill
    a = math.radians(angle)
    return c + r * reach * np.array([math.sin(a - math.pi), math.cos(a - math.pi)])

# The holes on the left, low round the foot of their hill, from the east
# (228 degrees from north, at 0.86 of its radius) round to the south-west
# (152 degrees), spread across the left of the riders' view as they come
# down to the bridge; those on the right on the south flank of the other
# (as `on_hill(HILL_B, 155, 0.8)`, `190, 0.55`, `220, 0.78`). All by the
# road, facing the riders coming (as `facing_riders`, the road 110 m back).
HOLES_A = [("HobbitDoorBlue", 613.9, 84.7), ("HobbitDoorRed", 503.9, 103.5),
           ("HobbitDoorYellow", 434.0, 101.9), ("HobbitDoorGreen", 398.0, 128.3)]
MILL = ("Mill", 360.0, 108.0)
HOLES_B = [("HobbitDoorGreen", 625.9, -71.0), ("HobbitDoorRed", 645.9, -101.1),
           ("HobbitDoorYellow", 651.9, -133.7)]
# On the left of the last bend, 34 m out, on a terrace 2 m over the road,
# facing the riders coming; the party tree past Bag End, so that none of
# them hides another (`clear`; with the Green Dragon this near the road,
# its roof crosses the foot of Bag End and its oak by under a degree from
# some of the bend).
GREEN_DRAGON = ("GreenDragon", 700.0, 34.0)
SIGN = ("GreenDragonSign", 668.0, 13.5)
PARTY_TREE = ("PartyTree", 800.0, 50.0)
BAG_END = ("BagEnd", 770.0, 70.0)
OAK = ("BagEndOak", 778.0, 84.0)
ROOT_TREE = ("RootTree", 845.0, 22.0)

def clear(front, front_half, front_tall, behind, views=range(540, 771, 10), cam=2.5, lim=45.0):
    """The least angle (degrees) between a building `front` (its middle,
    the height of its foot) and what stands `behind` it (each its middle,
    the height of its foot, its half width and its height), seen from the
    road coming at them, within `lim` of straight ahead."""
    def box(e, ey, f, p, base, half, tall):
        d = p - e; dist = np.linalg.norm(d)
        az = math.degrees(math.atan2(f[0] * d[1] - f[1] * d[0], f @ d)); w = math.degrees(half / dist)
        return (az - w, az + w, math.degrees(math.atan2(base - ey, dist)), math.degrees(math.atan2(base + tall - ey, dist)))
    worst = 1e9
    for va in views:
        e = pos(va); ey = float(LAP.h[LAP.index_at(va)]) + cam; f = fwd(heading(va))
        g = box(e, ey, f, front[0], front[1], front_half, front_tall)
        for p, h, half, tall in behind:
            b = box(e, ey, f, p, h, half, tall)
            if abs((g[0] + g[1]) / 2) < lim and abs((b[0] + b[1]) / 2) < lim:
                worst = min(worst, max(b[0] - g[1], g[0] - b[1], b[2] - g[3], g[2] - b[3]))
    return worst

def ahead(p, a_from=0.0, a_to=1100.0, angle=0.6, near=11.5, far=220.0):
    """The alongs from which `p` is within `angle` of straight ahead."""
    seen = []
    for a in np.arange(a_from, a_to, 4.0):
        d = np.asarray(p) - pos(a)
        dist = np.linalg.norm(d)
        if near < dist < far and math.acos(max(-1, min(1, d.dot(fwd(heading(a))) / dist))) < angle:
            seen.append(a)
    return (min(seen), max(seen), len(seen) * 4) if seen else None


if __name__ == "__main__":
    P = dict(DEFAULT)
    for kv in sys.argv[1:]:
        k, v = kv.split("="); P[k] = float(v)
    cp, lap, info = lap_for(P)
    r = 1 / np.maximum(np.abs(lap.k), 1e-9)
    worst = sorted(((float(r[lap.index_at(a)]), a) for a in range(100, 900, 2)))[:5]
    print("last arc R", round(info["RD"], 1), "length", round(info["LD"], 1), "ends at", np.round(info["end"], 2))
    print("lap", round(lap.length, 2), "m; tightest", [(round(x), a) for x, a in worst])
    print("centre line as the file's:", [tuple(map(float, p)) for p in cp] == [tuple(map(float, p)) for p in CP])
    for name, (c, rad, h) in [("hill A", HILL_A), ("hill B", HILL_B), ("Bag End's hill", BAG_HILL)]:
        wet = max(h * shape(max(0.0, np.linalg.norm(q - c) - w) / rad) for q, w in water_points())
        print(f"{name:15s} at {np.round(c, 1)} R {rad:.0f} {h:.0f} m: {h * shape(road_d(c) / rad):.1f} m at the road,"
              f" {wet:.1f} m at the water, steepest {math.degrees(math.atan(h * 1.5 / (0.55 * rad))):.0f} deg")
    placed = [(n, pos(a, l)) for n, a, l in HOLES_A + [MILL] + HOLES_B] \
        + [(n, pos(a, l)) for n, a, l in [GREEN_DRAGON, SIGN, PARTY_TREE, BAG_END, OAK, ROOT_TREE]]
    gd = pos(*GREEN_DRAGON[1:])
    behind = [(pos(*BAG_END[1:]), 20.6, 6.0, 12.0), (pos(*OAK[1:]), 22.0, 7.0, 16.0), (pos(*PARTY_TREE[1:]), 5.0, 10.0, 24.0)]
    print("the Green Dragon clear of Bag End, its oak and the party tree by", round(clear((gd, 6.8), 16.5, 9.0, behind), 1), "degrees")
    for name, p in placed:
        print(f"{name:18s} at {np.round(p, 1)}: {road_d(p):5.1f} m from the road, {water_d(p):5.1f} m from the water,"
              f" seen ahead from {ahead(p)}")
