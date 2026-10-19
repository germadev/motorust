"""Hobbiton's new S, from the end of the start straight (0,-250) to the
start of the Shire's straight (370.4,-571): a wide right curve down to the
bridge over the Water, the bridge, a left curve up, a straight, a right
curve at the foot of Bag End's hill round to the east, and a straight to
the Shire's: as long as before, so every along after it stays."""
import math, itertools, sys
import numpy as np
from model import Lap, heading_of
from parse_rs import array

import subprocess
SRC = subprocess.run(["git", "show", "12c2864:src/circuits/fellowship_road.rs"], capture_output=True, text=True, cwd="/home/user/motorust").stdout
CP = array(SRC, "CONTROL_POINTS")
EL = array(SRC, "ELEVATION")
I0, I1 = 4, 27
assert CP[I0] == (0.0, -250.0) and CP[I1] == (370.4, -571.0), (CP[I0], CP[I1])
old = Lap(CP, EL)

def along_of(lap, p):
    return np.linalg.norm(lap.line - np.array(p), axis=1).argmin() * lap.step

A0, A1 = along_of(old, CP[I0]), along_of(old, CP[I1])
H_END = -math.pi / 2  # east, along the Shire's straight

def fwd(h): return np.array([-math.sin(h), -math.cos(h)])

def arc_disp(h, a, sign):
    left = np.array([fwd(h)[1], -fwd(h)[0]])
    centre = left * sign
    h2 = h + sign * a
    ln = np.array([fwd(h2)[1], -fwd(h2)[0]])
    return centre - ln * sign, h2

L_TARGET = A1 - A0
TARGET = np.array(CP[I1]) - np.array(CP[I0])

def solve(ta, tc, sb, Ra, Re):
    """Unknowns: Rc, sd (after the left curve), sf (after the right one)."""
    a, c = math.radians(ta), math.radians(tc)
    h = 0.0
    ua, h = arc_disp(h, a, -1)
    vb = fwd(h)
    uc, h = arc_disp(h, c, +1)
    vd = fwd(h)
    te = h - H_END
    if te <= 0.05:
        return None
    ue, h = arc_disp(h, te, -1)
    vf = fwd(h)
    M = np.array([[uc[0], vd[0], vf[0]], [uc[1], vd[1], vf[1]], [c, 1.0, 1.0]])
    rhs = np.concatenate([TARGET - Ra * ua - sb * vb - Re * ue, [L_TARGET - Ra * a - sb - Re * te]])
    try:
        Rc, sd, sf = np.linalg.solve(M, rhs)
    except np.linalg.LinAlgError:
        return None
    return Rc, sd, sf, math.degrees(te)

if __name__ == "__main__":
    print("target length", round(L_TARGET, 2), "disp", TARGET, "A0", A0, "A1", A1)
    sols = []
    for ta, tc in itertools.product(range(40, 91, 5), range(40, 121, 5)):
        for sb in (36.0, 44.0):
            for Ra in range(110, 241, 10):
                for Re in range(50, 131, 10):
                    r = solve(ta, tc, sb, Ra, Re)
                    if r is None:
                        continue
                    Rc, sd, sf, te = r
                    if Rc > 90 and sd >= 10 and sf >= 20:
                        sols.append((ta, tc, sb, Ra, Re, round(Rc, 1), round(sd, 1), round(sf, 1), round(te, 1)))
    print(len(sols))
    key = lambda s: -(min(s[3], 200) * min(s[0], 70) / 70) - 0.6 * min(s[5], 170) * min(s[1], 80) / 80 + 0.2 * abs(s[6] - 40)
    for s in sorted(sols, key=key)[:30]:
        print(s)

def ess_disp(Rs, angle):
    """Esses east through the fields, from heading east and back to it:
    left `angle`, right twice it, left `angle`, all of radius Rs."""
    h = H_END
    total = np.zeros(2)
    for turn, sign in ((angle, 1), (2 * angle, -1), (angle, 1)):
        u, h = arc_disp(h, math.radians(turn), sign)
        total += Rs * u
    return total, Rs * math.radians(4 * angle)

def solve_ess(ta, tc, sb, Ra, Re, Rs, es):
    a, c = math.radians(ta), math.radians(tc)
    h = 0.0
    ua, h = arc_disp(h, a, -1)
    vb = fwd(h)
    uc, h = arc_disp(h, c, +1)
    vd = fwd(h)
    te = h - H_END
    if te <= 0.05:
        return None
    ue, h = arc_disp(h, te, -1)
    vf = fwd(h)
    d_ess, l_ess = ess_disp(Rs, es)
    M = np.array([[uc[0], vd[0], vf[0]], [uc[1], vd[1], vf[1]], [c, 1.0, 1.0]])
    rhs = np.concatenate([TARGET - Ra * ua - sb * vb - Re * ue - d_ess, [L_TARGET - Ra * a - sb - Re * te - l_ess]])
    try:
        Rc, sd, sf = np.linalg.solve(M, rhs)
    except np.linalg.LinAlgError:
        return None
    return Rc, sd, sf, math.degrees(te)
