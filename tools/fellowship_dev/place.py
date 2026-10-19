"""Placing props on Fellowship Road: the centre line and the ground as the
game has them (dumped by the probe), and helpers."""
import math
from pathlib import Path
import numpy as np

S = Path("/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/out")
L = np.loadtxt(S / "line.tsv")
X, Z, HEAD, Y, K, WALL = L[:, 1], L[:, 2], L[:, 3], L[:, 4], L[:, 5], L[:, 6]
N = len(L)
LAP = 5944.3647
STEP = LAP / N
X0, Z0, G = -500.0, -1000.0, 4.0
NX, NZ = int((2300 - X0) / G) + 1, int((1100 - Z0) / G) + 1
raw = np.fromfile(S / "ground.bin", dtype="<f4").reshape(NZ, NX, 2)
HGT, DIST = raw[:, :, 0], raw[:, :, 1]
SEA = -1.0

def idx(along):
    return int(round((along % LAP) / STEP)) % N

def fwd(h):
    return np.array([-math.sin(h), -math.cos(h)])

def left(h):
    return np.array([-math.cos(h), math.sin(h)])

def at(along, lateral=0.0):
    i = idx(along)
    return np.array([X[i], Z[i]]) + left(HEAD[i]) * lateral

def heading_to(frm, to):
    d = np.asarray(to) - np.asarray(frm)
    return math.atan2(-d[0], -d[1])

def bil(grid, x, z):
    fx, fz = (x - X0) / G, (z - Z0) / G
    i, j = int(math.floor(fx)), int(math.floor(fz))
    if not (0 <= i < NX - 1 and 0 <= j < NZ - 1):
        return float("nan")
    tx, tz = fx - i, fz - j
    return (grid[j, i] * (1 - tx) * (1 - tz) + grid[j, i + 1] * tx * (1 - tz)
            + grid[j + 1, i] * (1 - tx) * tz + grid[j + 1, i + 1] * tx * tz)

def ground(p):
    return bil(HGT, p[0], p[1])

def dist(p):
    return bil(DIST, p[0], p[1])

def lowest(p, reach):
    return min(ground(np.asarray(p) + o * 0.6 * reach) for o in [np.zeros(2), np.array([1, 0]), np.array([-1, 0]), np.array([0, 1]), np.array([0, -1])])

def road_y(along):
    return Y[idx(along)]

def seen_from(p, reach=220.0, angle=0.6, wall=11.5):
    """How many metres of road (coming along it) see the point straight
    ahead within `angle`, and the alongs where it is."""
    seen = []
    for i in range(0, N, 4):
        here = np.array([X[i], Z[i]])
        ahead = fwd(HEAD[i])
        to = np.asarray(p) - here
        d = np.linalg.norm(to)
        if wall < d < reach:
            a = math.acos(max(-1, min(1, ahead @ to / d)))
            if a < angle:
                seen.append(i * STEP)
    return len(seen) * 4 * STEP, (min(seen) if seen else None, max(seen) if seen else None)

def summary(a0, a1, every=20.0, sides=(15, 25, 40, 60)):
    a = a0
    while a <= a1 + 1e-6:
        i = idx(a)
        k = K[i]
        r = (1 / k) if abs(k) > 1e-5 else float("inf")
        row = f"{a:7.0f} ({X[i]:7.1f},{Z[i]:7.1f}) h{math.degrees(HEAD[i]):6.0f} y{Y[i]:5.1f} R{r:7.0f}"
        for s in sides:
            for sign in (1, -1):
                p = at(a, sign * s)
                g = ground(p)
                row += f" {'L' if sign > 0 else 'R'}{s}:{g - Y[i]:+5.1f}{'~' if g < SEA else ''}"
        print(row)
        a += every
