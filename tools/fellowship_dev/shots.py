import sys
sys.path.insert(0, "tools")
from props_plan import *

def nearest_along(p):
    d = (X - p[0]) ** 2 + (Z - p[1]) ** 2
    return int(d.argmin()) * STEP

def cam(along, lateral=0.0, up=3.5):
    q = at(along, lateral)
    return (q[0], road_y(along) + up, q[1])

def look(p, name, frac=0.4):
    g = max(ground(p), SEA)
    return (p[0], g + DIMS[name][1] * frac, p[1])

shots = []
def add(label, c, l):
    shots.append(f"{label} {c[0]:.1f} {c[1]:.1f} {c[2]:.1f} {l[0]:.1f} {l[1]:.1f} {l[2]:.1f}")

def by(name, k=0):
    return [p for p in PROPS if p.name == name][k]

# Riders' views of each landmark, from some way back along the road.
for label, name, k, back, lat, up in [
    ("01_start", "green_dragon", 0, 140, 0, 3.0),
    ("02_hobbit_holes", "hobbit_door_red", 0, 55, -3, 2.5),
    ("03_green_dragon", "green_dragon", 0, 45, 3, 2.5),
    ("04_mill", "mill", 0, 60, 4, 3.0),
    ("05_party_tree", "party_tree", 0, 70, -3, 3.0),
    ("06_yellow_hole", "hobbit_door_yellow", 0, 40, -3, 2.5),
    ("07_bag_end", "bag_end", 0, 90, 0, 3.0),
    ("08_root_tree", "root_tree", 0, 50, 0, 3.0),
    ("09_ferry", "ferry", 0, 0, 0, 4.0),
    ("10_bree_gate", "bree_gate", 0, 70, 0, 3.0),
    ("11_pony", "prancing_pony", 0, 45, 0, 3.0),
    ("12_bree_street", "bree_house_a", 1, 40, 0, 3.0),
    ("13_weathertop", "weathertop_ruin", 0, 70, 0, 3.0),
    ("14_trolls", "stone_trolls", 0, 50, 0, 3.0),
    ("15_gazebo", "gazebo", 0, 45, 0, 3.0),
    ("16_tower", "elven_tower", 0, 90, 0, 3.0),
    ("17_last_homely_house", "last_homely_house", 0, 90, 0, 3.0),
    ("18_durin_far", "durin_doors", 0, 70, 0, 3.0),
    ("19_durin_near", "durin_doors", 0, 22, -4, 2.0),
    ("20_mallorn", "mallorn_flet", 0, 60, 0, 3.0),
    ("21_swan_ship", "swan_ship", 0, None, 0, 3.0),
    ("22_argonath", "argonath_a", 0, None, 0, 3.0),
    ("23_elven_ship", "elven_ship", 0, None, 0, 3.0),
    ("24_amon_hen", "amon_hen_seat", 0, 70, 0, 3.0),
    ("25_boats", "elven_boat", 0, None, 0, 3.0),
]:
    p = by(name, k)
    if back is None:
        # From where the road sees it best, straight ahead.
        a0, a1 = seen_from(p.pos)[1]
        along = a0 + (a1 - a0) * 0.35 if a0 is not None else nearest_along(p.pos) - 60
    else:
        along = (p.how[1] if p.how[0] in ("road", "over") else nearest_along(p.pos)) - back
    add(label, cam(along, lat, up), look(p.pos, name))
# From the air.
add("30_air_hobbiton", (60.0, 70.0, 40.0), (-20.0, 0.0, -250.0))
add("31_air_bree", (700.0, 60.0, -440.0), (800.0, 8.0, -510.0))
add("32_air_argonath", (560.0, 60.0, 380.0), (420.0, 20.0, 370.0))
add("33_air_rivendell", (1700.0, 70.0, -340.0), (1600.0, 10.0, -200.0))
open("out/shots.txt", "w").write("\n".join(shots) + "\n")
print("\n".join(shots))
