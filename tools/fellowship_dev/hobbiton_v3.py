"""Hobbiton's third design on Fellowship Road (see
`RECORD/2026-10-16.fellowship-track.WIP.md`, section 10): its centre line
and where its scenery stands.

The centre line, from the end of the start straight (0, -150) heading
north (-Z) to the Shire's straight (z = -571) heading east, is drawn by
integrating a piecewise linear curvature: a wide sweep to the right (R 200)
down to the Water, the bridge, a sweep to the left (R 200) and a long one
to the right round the pool, with 30 m ramps of curvature between straights
and arcs and between opposite arcs, so that the spline through the points
(every 13 m, to the centimetre) has no kinks. The last arc's radius and
length are found (Newton) so that it ends heading east on the Shire's line.

Its scenery is placed by offsets from that line (m along it, m to its
left): the Water, the pool, the hills, the hobbit holes and the other
buildings, each checked to be seen ahead from the road (`ahead`): the
first holes past the bridge on the left, where the riders look all
through the sweep to the right, across the water; each row of holes
up its hill at heights that alternate, low and high (not a staircase),
each to one side of the others, so that none hides another.

Run: `python3 tools/fellowship_dev/hobbiton_v3.py [name=value ...]` from the
repository root prints the last arc, the lap's length and its tightest
radii, and where each building stands and from where it is seen ahead.
Needs numpy, and `model.py`, `parse_rs.py` beside it; it reads the control
points and the profile from `src/circuits/fellowship_road.rs`.
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
J = CP.index((445.4, -571.0))
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
        # game headings: a right turn subtracts (east is -pi/2)
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
    add(P["LA"], kA)           # A
    add(T, 0.0)                # out of A
    add(P["bridge"], 0.0)      # the bridge
    add(T, kC)                 # into C
    add(P["LC"], kC)           # C
    add(P["TCD"], kD)          # C into D
    add(LD, kD)                # D
    add(T, 0.0)                # out of D
    return out

def solve(P):
    """Find RD, LD: end heading -pi/2 (east) and end z on the Shire's line."""
    def err(RD, LD):
        pts, hs, _, _ = integrate(knots_for(P, RD, LD), P["z0"], ds=0.5)
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

DEFAULT = dict(z0=150.0, ramp=30.0, RA=200.0, LA=150.0, bridge=40.0, RC=200.0, LC=60.2, TCD=40.0)

def lap_for(P, every=13.0):
    inner, info = control_points(P, every)
    start = [(0.0, 0.0), (0.0, -50.0), (0.0, -100.0), (0.0, -P["z0"])]
    end = info["end"]
    tail = [p for p in CP[J:] if p[0] > end[0] + 25.0 or p[1] > -560.0]
    cp = start + inner + tail
    return cp, Lap(cp, EL), info


# Where the scenery stands.
P = dict(DEFAULT)
CP_NEW, _, INFO = lap_for(P)

# The new profile through the wood and Hobbiton (along, height over the
# base), up to where it meets the old one, shifted.
EL_REGION = [
    (0.0, 1.5), (60.0, 2.0), (125.0, 6.8), (160.0, 5.6), (200.0, 4.8),
    (260.0, 3.3), (320.0, 1.6), (358.0, 0.6), (402.0, 0.6), (440.0, 1.2),
    (500.0, 2.4), (580.0, 3.6), (680.0, 4.8), (745.0, 5.6),
]
OLD_LAP = Lap(CP, EL)

def shifted_elevation(delta, keep_from=930.0):
    out = list(EL_REGION)
    for a, h in EL:
        if a >= keep_from:
            out.append((round(a + delta, 1), h))
    return out

LAP0 = Lap(CP_NEW, EL)
DELTA = round(LAP0.length - OLD_LAP.length, 1)
# Once applied, the file holds the new lap and its profile already.
EL_NEW = EL if abs(DELTA) < 0.05 else shifted_elevation(DELTA)
LAP = Lap(CP_NEW, EL_NEW)

def at(along, lateral=0.0):
    return LAP.offset(along, lateral)

def road_y(along):
    return float(LAP.h[LAP.index_at(along)])

def heading(along):
    i = LAP.index_at(along)
    t = LAP.line[(i + 1) % LAP.n] - LAP.line[(i - 1) % LAP.n]
    return math.atan2(-t[0], -t[1])

# The Water: offsets of the line (along, lateral, half width).
CROSS, CROSS_W, WL = 380.0, 92.0, 46.0
def water_lateral(a):
    if a <= CROSS - CROSS_W / 2:
        return WL
    if a >= CROSS + CROSS_W / 2:
        return -WL
    return WL * math.cos(math.pi * (a - (CROSS - CROSS_W / 2)) / CROSS_W)

WATER_UP = [(-150.0, 40.0, 8.0), (-115.0, -60.0, 9.0), (-75.0, -120.0, 11.0)]
WATER_ALONG = [(a, water_lateral(a), w) for a, w in [
    (160.0, 13.0), (200.0, 15.0), (240.0, 16.0), (280.0, 16.0), (315.0, 16.0),
    (345.0, 16.0), (365.0, 16.0), (380.0, 16.0), (395.0, 16.0), (415.0, 16.0),
    (445.0, 15.0), (480.0, 14.0), (510.0, 13.0)]]


def ahead(p, a_from=0.0, a_to=900.0, angle=0.45, near=50.0, far=260.0):
    """The alongs from which `p` is within `angle` of straight ahead."""
    seen = []
    for a in np.arange(a_from, a_to, 4.0):
        q = at(a); h = heading(a)
        f = np.array([-math.sin(h), -math.cos(h)])
        d = np.asarray(p) - q
        dist = np.linalg.norm(d)
        if near < dist < far:
            ang = math.acos(max(-1, min(1, d.dot(f) / dist)))
            if ang < angle:
                seen.append(a)
    if not seen:
        return None
    return (min(seen), max(seen), len(seen) * 4)


HOLES_A = [("HobbitDoorBlue", 420.0, 55.0), ("HobbitDoorRed", 400.0, 100.0),
           ("HobbitDoorYellow", 380.0, 55.0), ("HobbitDoorGreen", 370.0, 100.0)]
MILL = ("Mill", 350.0, 70.0)
POND_AT = (560.0, -74.0); POND_R = 24.0
HOLES_B = [("HobbitDoorGreen", 610.0, -125.0), ("HobbitDoorRed", 580.0, -155.0),
           ("HobbitDoorYellow", 550.0, -125.0)]
GREEN_DRAGON = ("GreenDragon", 620.0, -96.0)
SIGN = ("GreenDragonSign", 520.0, -13.5)
PARTY_TREE = ("PartyTree", 640.0, 42.0)
BAG_END = ("BagEnd", 700.0, 62.0)
OAK = ("BagEndOak", 708.0, 76.0)
ROOT_TREE = ("RootTree", 735.0, 20.0)
SCARECROW = ("Scarecrow", 800.0, 14.0)
LOGS = ("LogPile", 880.0, -13.5)
HILL_A = (510.0, 170.0, 160.0, 24.0)
HILL_B = (560.0, -240.0, 150.0, 24.0)
BAG_HILL = (700.0, 185.0, 170.0, 32.0)

def pos(a, l):
    return at(a, l)


if __name__ == "__main__":
    P = dict(DEFAULT)
    for kv in sys.argv[1:]:
        k, v = kv.split("="); P[k] = float(v)
    cp, lap, info = lap_for(P)
    r = 1 / np.maximum(np.abs(lap.k), 1e-9)
    worst = sorted(((float(r[lap.index_at(a)]), a) for a in range(100, 900, 2)))[:5]
    print("last arc R", round(info["RD"], 1), "length", round(info["LD"], 1), "ends at", np.round(info["end"], 2))
    print("lap", round(lap.length, 2), "m; tightest", [(round(x), a) for x, a in worst])
    for name, a, l in HOLES_A + [MILL] + HOLES_B + [GREEN_DRAGON, PARTY_TREE, BAG_END, OAK, ROOT_TREE, SCARECROW]:
        p = pos(a, l)
        print(f"{name:18s} {a:6.1f} {l:7.1f} at {np.round(p, 1)} seen ahead from {ahead(p)}")
