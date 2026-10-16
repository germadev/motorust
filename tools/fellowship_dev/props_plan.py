import sys
sys.path.insert(0, "/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/tools")
from plan import *

PI = math.pi
PROPS = [
    # Hobbiton.
    facing("gandalf_cart", 32, -14.5),
    facing("hobbit_door_blue", 70, -24),
    road("log_pile", 84, -13.5, 0),
    facing("hobbit_door_red", 110, -24),
    facing("green_dragon_sign", 112, 13.5),
    facing("green_dragon", 135, 30),
    road("party_tree", 170, -48, 0),
    facing("mill", 210, 46),
    facing("hobbit_door_yellow", 300, -24),
    facing("hobbit_door_green", 360, 26),
    facing("hobbit_door_blue", 405, -24),
    facing("hobbit_door_red", 470, 26),
    facing("bag_end", 505, 34),
    road("bag_end_oak", 505, 50, 0),
    # The Shire.
    facing("scarecrow", 640, 14),
    road("root_tree", 700, -19, 0),
    road("log_pile", 900, -13.5, 0),
    free("ferry", 562, -626, -PI / 2),
    # Bree.
    over("bree_gate", 1286),
    facing("prancing_pony", 1335, 33),
    facing("bree_house_b", 1300, -16),
    facing("bree_house_b", 1312, -16),
    facing("bree_house_b", 1355, -16),
    facing("bree_house_b", 1367, -16),
    facing("bree_house_a", 1378, 24),
    facing("bree_house_a", 1398, -22),
    facing("bree_house_b", 1438, 16),
    facing("bree_house_b", 1450, 16),
    facing("bree_house_a", 1470, -22),
    facing("bree_house_b", 1462, 16),
    facing("bree_house_b", 1486, 16),
    facing("bree_house_b", 1500, -16),
    # Weathertop and the Trollshaws.
    facing("weathertop_ruin", 2008, -38),
    facing("ruin_arch", 1960, 16),
    facing("ruin_arch", 2000, -16),
    facing("stone_trolls", 2290, 21),
    # Rivendell.
    road("elven_lamp", 2570, 13.5, 0),
    road("elven_lamp", 2570, -13.5, 0),
    facing("gazebo", 2600, 24),
    road("elven_tower", 2700, -42, 0),
    road("elven_lamp", 2990, 13.5, 0),
    road("elven_lamp", 2990, -13.5, 0),
    facing("last_homely_house", 3045, 46),
    # Moria.
    road("durin_doors", 3962, 13.2, 0),
    # Lothlórien.
    road("elven_lamp", 4345, 13.5, 0),
    road("elven_lamp", 4345, -13.5, 0),
    road("mallorn_flet", 4352, 40, 0),
    # The Anduin.
    free("swan_ship", 515, 425, PI / 2),
    free("argonath_a", 416, 414, -PI / 2),
    free("argonath_b", 444, 328.5, -PI / 2),
    free("elven_ship", 330, 410, -PI / 2),
    # Amon Hen.
    free("elven_boat", 238, 393, 2.2),
    free("elven_boat", 226, 404, 2.0),
    facing("amon_hen_seat", 5600, 52),
]
if __name__ == "__main__":
    check(PROPS)
    out = "/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/out/"
    if "draw" in sys.argv:
        draw_region(PROPS, -180, -620, 120, 150, out + "plan_hobbiton.png", 2.0)
        draw_region(PROPS, 480, -700, 1020, -420, out + "plan_shire_bree.png", 2.0)
        draw_region(PROPS, 1150, -780, 1650, -300, out + "plan_weathertop_rivendell.png", 1.5)
        draw_region(PROPS, 1500, -420, 1800, 120, out + "plan_rivendell.png", 1.5)
        draw_region(PROPS, 1150, 300, 1800, 480, out + "plan_moria_lorien.png", 1.5)
        draw_region(PROPS, 120, 260, 900, 620, out + "plan_anduin.png", 1.5)
