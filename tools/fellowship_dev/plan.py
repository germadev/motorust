"""The plan of Fellowship Road's props: where each stands, checked against
the road, the water and the riders' view, and drawn on maps."""
import math, sys
import numpy as np
from PIL import Image, ImageDraw
sys.path.insert(0, "/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/tools")
from place import *

# Game sizes (x width, y height, z depth) from the import.
DIMS = {
 "hobbit_door_green": (6.8, 4.0, 0.9), "hobbit_door_yellow": (7.5, 4.0, 2.5), "hobbit_door_red": (7.1, 4.0, 1.1),
 "hobbit_door_blue": (6.2, 4.0, 1.8), "bag_end": (10.1, 5.5, 6.7), "chimney": (1.4, 2.4, 0.9),
 "party_tree": (20.1, 24.0, 14.9), "bag_end_oak": (14.0, 14.0, 6.5), "green_dragon": (31.7, 8.0, 11.4),
 "green_dragon_sign": (2.3, 3.2, 0.4), "gandalf_cart": (4.0, 3.3, 4.5), "log_pile": (1.7, 1.5, 0.9),
 "scarecrow": (1.7, 2.3, 0.6), "root_tree": (15.1, 16.0, 14.6), "ferry": (9.0, 4.3, 4.8),
 "bree_gate": (45.6, 15.4, 13.3), "prancing_pony": (28.4, 14.0, 23.7), "bree_house_a": (15.2, 10.0, 10.8),
 "bree_house_b": (4.8, 11.0, 3.9), "weathertop_ruin": (32.0, 11.0, 30.0), "ruin_arch": (6.0, 6.0, 2.6),
 "stone_trolls": (13.8, 7.0, 7.2), "last_homely_house": (23.3, 18.0, 35.8), "gazebo": (7.3, 9.0, 7.3),
 "elven_tower": (5.5, 24.0, 6.2), "elven_lamp": (0.9, 3.4, 0.4), "durin_doors": (6.6, 14.0, 1.9),
 "mallorn_flet": (24.8, 38.0, 24.8), "swan_ship": (5.2, 9.6, 16.0), "elven_ship": (20.0, 10.9, 3.2),
 "elven_boat": (6.0, 1.6, 1.4), "argonath_a": (21.8, 60.0, 22.3), "argonath_b": (19.9, 60.0, 21.7),
 "amon_hen_seat": (13.2, 9.0, 12.3), "mill": (10.4, 11.0, 6.6),
}
WET = {"swan_ship", "elven_ship", "ferry"}
SHORE = {"elven_boat"}

def reach(name, scale=1.0):
    w, h, d = DIMS[name]
    return scale * max(w, d) / 2

class P:
    def __init__(s, name, pos, heading, how, scale=1.0, note=""):
        s.name, s.pos, s.heading, s.how, s.scale, s.note = name, np.asarray(pos, float), heading, how, scale, note

def road(name, along, lateral, turn_deg=0.0, scale=1.0, note=""):
    """By the road, facing it, turned (positive towards riders coming for
    either side, as in the plan; converted for the game)."""
    p = at(along, lateral)
    i = idx(along)
    to_road = np.array([X[i], Z[i]]) - p
    h = heading_to(p, p + to_road)
    t = math.radians(turn_deg) * (1 if lateral < 0 else -1)
    # facing direction rotated by t: (dx cos t + dz sin t, -dx sin t + dz cos t)
    d = to_road / np.linalg.norm(to_road)
    d2 = np.array([d[0] * math.cos(t) + d[1] * math.sin(t), -d[0] * math.sin(t) + d[1] * math.cos(t)])
    return P(name, p, heading_to(p, p + d2), ("road", along, lateral, t), scale, note)

FACING_BACK = 60.0
MAX_TURN = math.radians(45)

def facing(name, along, lateral, scale=1.0, note=""):
    """By the road, its front turned to a point of the road FACING_BACK
    metres before it, so that it faces the riders coming (at most
    MAX_TURN from facing the road)."""
    p = at(along, lateral)
    i = idx(along)
    d = np.array([X[i], Z[i]]) - p
    d /= np.linalg.norm(d)
    d2 = at(along - FACING_BACK, 0) - p
    d2 /= np.linalg.norm(d2)
    t = math.atan2(d[1] * d2[0] - d[0] * d2[1], d[0] * d2[0] + d[1] * d2[1])
    t = max(-MAX_TURN, min(MAX_TURN, t))
    d3 = np.array([d[0] * math.cos(t) + d[1] * math.sin(t), -d[0] * math.sin(t) + d[1] * math.cos(t)])
    return P(name, p, heading_to(p, p + d3), ("road", along, lateral, t), scale, note)

def free(name, x, z, heading, scale=1.0, note=""):
    return P(name, (x, z), heading, ("at",), scale, note)

def over(name, along, note=""):
    i = idx(along)
    return P(name, (X[i], Z[i]), HEAD[i], ("over", along), 1.0, note)

def check(props):
    bad = 0
    for p in props:
        r = reach(p.name, p.scale)
        dd = dist(p.pos)
        g = ground(p.pos)
        wet = g < SEA
        ok = True
        msg = []
        if p.how[0] != "over" and not (dd - 0.7 * r > 11.5):
            ok = False; msg.append(f"near road {dd:.1f} (needs {11.5 + 0.7 * r:.1f})")
        if p.name in WET and not wet:
            ok = False; msg.append("a boat ashore")
        if p.name not in WET and p.name not in SHORE and wet:
            ok = False; msg.append("in the water")
        seen, (a0, a1) = seen_from(p.pos)
        print(f"{'  ' if ok else '!!'} {p.name:18s} ({p.pos[0]:7.1f},{p.pos[1]:7.1f}) h{math.degrees(p.heading):6.0f} road {dd:5.1f} reach {r:4.1f} ground {g:5.1f} seen {seen:5.0f} m {('' if a0 is None else f'{a0:.0f}-{a1:.0f}')} {' '.join(msg)} {p.note}")
        bad += not ok
    print("problems:", bad)

def draw_region(props, x0, z0, x1, z1, path, scale=4.0):
    W, H = int((x1 - x0) * scale), int((z1 - z0) * scale)
    img = Image.new("RGB", (W, H))
    px = img.load()
    for j in range(H):
        for i in range(W):
            x, z = x0 + i / scale, z0 + j / scale
            g = bil(HGT, x, z); d = bil(DIST, x, z)
            if d < 7: c = (60, 60, 64)
            elif d < 11.5: c = (110, 110, 100)
            elif g < SEA: c = (70, 110, 170)
            else:
                v = int(max(0, min(255, 90 + g * 3)))
                c = (v // 2, v, v // 3)
            px[i, j] = c
    dr = ImageDraw.Draw(img)
    def S(p): return ((p[0] - x0) * scale, (p[1] - z0) * scale)
    # alongs every 50 m
    for a in range(0, int(LAP), 50):
        p = at(a, 0)
        if x0 <= p[0] <= x1 and z0 <= p[1] <= z1:
            dr.text(S(p), str(a), fill=(255, 255, 255))
    for p in props:
        if not (x0 - 50 <= p.pos[0] <= x1 + 50 and z0 - 50 <= p.pos[1] <= z1 + 50):
            continue
        w, h, d = DIMS[p.name]
        w, d = w * p.scale, d * p.scale
        f = fwd(p.heading); l = left(p.heading)
        # box: model +Z along f (depth), width along l
        corners = [p.pos + f * sz * d / 2 + l * sx * w / 2 for sx, sz in ((1, 1), (-1, 1), (-1, -1), (1, -1))]
        dr.polygon([S(c) for c in corners], outline=(255, 220, 60))
        dr.line([S(p.pos), S(p.pos + f * (d / 2 + 4))], fill=(255, 80, 80), width=2)
        dr.text(S(p.pos + l * 0), p.name.replace("hobbit_door_", "hd_")[:14], fill=(255, 255, 255))
    img.save(path)
