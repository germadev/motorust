"""Hobbiton, second design: a wider bend round the pool, so the lap through
the village is about as quick as it was."""
import math, numpy as np
from model import Lap, simulate
from turtle import Turtle
from hobbiton_solve2 import solve_ess, CP as OLD_CP, EL as OLD_EL, I0
from parse_rs import array

P = dict(ta=20, tc=50, sb=30.0, Ra=240, Re=85, Rs=250, es=10)
CUR_SRC = open('/home/user/motorust/src/circuits/fellowship_road.rs').read()
CUR = array(CUR_SRC, 'CONTROL_POINTS')
ELc = array(CUR_SRC, 'ELEVATION')
J = CUR.index((370.4, -571.0))

def build(tweak, fine=14.0):
    Rc, sd, sf, te = solve_ess(P["ta"], P["tc"], P["sb"], P["Ra"], P["Re"], P["Rs"], P["es"])
    t = Turtle(0.0, -250.0, 0.0, 26.0)
    marks = {}
    t.R(P["Ra"], P["ta"]); marks["bridge_from"] = t.p.copy()
    t.S(P["sb"]); marks["bridge_to"] = t.p.copy()
    left_centre = t.p + np.array([fwd_left(t.h)]) [0] * Rc
    t.L(Rc, P["tc"]); marks["left_end"] = t.p.copy()
    t.S(sd + tweak); marks["hairpin_from"] = t.p.copy(); marks["hairpin_h"] = t.h
    hairpin_centre = t.p + right_normal(t.h) * P["Re"]
    t.every = fine
    t.R(P["Re"], te)
    t.every = 26.0
    marks["hairpin_to"] = t.p.copy()
    for turn, left in ((P["es"], True), (2 * P["es"], False), (P["es"], True)):
        t.arc(P["Rs"], turn, left)
    t.S(sf - tweak)
    inner = [(round(float(q[0]), 1), round(float(q[1]), 1)) for q in t.points[1:-1]]
    return inner, dict(Rc=Rc, sd=sd, sf=sf, te=te, hairpin_centre=hairpin_centre, left_centre=left_centre, **marks)

def fwd(h): return np.array([-math.sin(h), -math.cos(h)])
def fwd_left(h): return np.array([-math.cos(h), math.sin(h)])
def right_normal(h): return -fwd_left(h)

def best():
    cur = Lap(CUR, ELc)
    out = None
    for tw in np.arange(-2, 2.01, 0.05):
        inner, info = build(tw)
        cp = CUR[:I0 + 1] + inner + CUR[J:]
        lap = Lap(cp, ELc)
        d = lap.length - cur.length
        if out is None or abs(d) < abs(out[0]):
            out = (d, tw, cp, lap, inner, info)
    return cur, out

if __name__ == "__main__":
    cur, (d, tw, cp, lap, inner, info) = best()
    v, t = simulate(lap); v0, t0 = simulate(cur)
    r = 1 / np.maximum(np.abs(lap.k), 1e-9)
    i0, i1 = lap.index_at(250), lap.index_at(920)
    print("len diff", round(d, 3), "tweak", round(tw, 2), "min R", round(r[i0:i1].min(), 1),
          "time", round(t.sum(), 2), "vs", round(t0.sum(), 2), "to 900", round(np.cumsum(t)[lap.index_at(900)], 2), "vs", round(np.cumsum(t0)[cur.index_at(900)], 2), "vmin", round(v[i0:i1].min(), 1))
    for k in ("bridge_from", "bridge_to", "left_end", "hairpin_from", "hairpin_to"):
        p = info[k]; dd = np.linalg.norm(lap.line - p, axis=1)
        print(k, np.round(p, 1), "along", round(dd.argmin() * lap.step, 1))
    print("hairpin centre", np.round(info["hairpin_centre"], 1), "Rc", round(info["Rc"], 1), "sd", round(info["sd"], 1), "sf", round(info["sf"], 1))
    for a in (920, 2000, 4000):
        print(a, cur.at(a).round(2), lap.at(a).round(2))
