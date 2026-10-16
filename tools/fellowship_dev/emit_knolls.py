import sys
sys.path.insert(0, "tools")
from props_plan import *

LIFT = 1.3
RAISE = {
    "hobbit_door_green": 6.0, "hobbit_door_yellow": 6.0, "hobbit_door_red": 6.0, "hobbit_door_blue": 6.0,
    "bag_end": 7.0, "green_dragon": 17.0, "mill": 7.0, "prancing_pony": 16.0, "bree_house_a": 9.0,
    "stone_trolls": 8.0, "gazebo": 5.0, "last_homely_house": 19.0,
}
NAMES = {
    "hobbit_door_green": "a hobbit hole", "hobbit_door_yellow": "a hobbit hole", "hobbit_door_red": "a hobbit hole",
    "hobbit_door_blue": "a hobbit hole", "bag_end": "Bag End", "green_dragon": "the Green Dragon", "mill": "the mill",
    "prancing_pony": "the Prancing Pony", "bree_house_a": "a house of Bree", "stone_trolls": "the stone trolls",
    "gazebo": "Rivendell's pavilion", "last_homely_house": "the Last Homely House",
}
out = []
for p in PROPS:
    if p.name not in RAISE:
        continue
    g = lowest(p.pos, reach(p.name, p.scale))
    out.append(f"    // {NAMES[p.name][0].upper() + NAMES[p.name][1:]}.\n    knoll({p.pos[0]:.1f}, {p.pos[1]:.1f}, {RAISE[p.name]:.1f}, {g + LIFT:.1f}),")
print(len(out))
print("\n".join(out))
