import sys, math
sys.path.insert(0, "tools")
from props_plan import *

MODEL = {
 "hobbit_door_green": "HobbitDoorGreen", "hobbit_door_yellow": "HobbitDoorYellow", "hobbit_door_red": "HobbitDoorRed",
 "hobbit_door_blue": "HobbitDoorBlue", "bag_end": "BagEnd", "party_tree": "PartyTree", "bag_end_oak": "BagEndOak",
 "mill": "Mill", "green_dragon": "GreenDragon", "green_dragon_sign": "GreenDragonSign", "gandalf_cart": "GandalfCart",
 "log_pile": "LogPile", "scarecrow": "Scarecrow", "root_tree": "RootTree", "ferry": "Ferry", "bree_gate": "BreeGate",
 "prancing_pony": "PrancingPony", "bree_house_a": "BreeHouseA", "bree_house_b": "BreeHouseB",
 "weathertop_ruin": "WeathertopRuin", "ruin_arch": "RuinArch", "stone_trolls": "StoneTrolls",
 "last_homely_house": "LastHomelyHouse", "elven_tower": "ElvenTower", "gazebo": "Gazebo", "elven_lamp": "ElvenLamp",
 "durin_doors": "DurinDoors", "mallorn_flet": "MallornFlet", "swan_ship": "SwanShip", "elven_ship": "ElvenShip",
 "elven_boat": "ElvenBoat", "argonath_a": "ArgonathA", "argonath_b": "ArgonathB", "amon_hen_seat": "AmonHenSeat",
}
def f(v):
    s = f"{v:.1f}"
    return s
def f2(v):
    return f"{v:.2f}"
for p in PROPS:
    m = "MiddleEarthModel::" + MODEL[p.name]
    if p.how[0] == "road":
        _, along, lateral, t = p.how
        line = f"by_road({m}, {f(along)}, {f(lateral)})"
        if abs(t) > 1e-6:
            line += f".turned({f2(t)})"
    elif p.how[0] == "over":
        line = f"over_road({m}, {f(p.how[1])})"
    else:
        line = f"standing({m}, {f(p.pos[0])}, {f(p.pos[1])}, {f2(p.heading)})"
    print("    " + line + ",")
print(len(PROPS))
