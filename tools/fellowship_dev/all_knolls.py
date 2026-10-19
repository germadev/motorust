import sys, io, contextlib
sys.path.insert(0, "/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/tools")
with contextlib.redirect_stdout(io.StringIO()):
    from hobbiton_props import place_all, HOLES
    from props_plan import PROPS
from place import ground, lowest, fwd
from plan import reach
LIFT = 1.3
RAISE = {"bag_end": 7.0, "green_dragon": 14.0, "prancing_pony": 16.0, "bree_house_a": 9.0,
         "stone_trolls": 8.0, "gazebo": 5.0, "last_homely_house": 19.0}
NAMES = {"bag_end": "Bag End", "green_dragon": "The Green Dragon", "prancing_pony": "The Prancing Pony",
         "bree_house_a": "A house of Bree", "stone_trolls": "The stone trolls", "gazebo": "Rivendell's pavilion",
         "last_homely_house": "The Last Homely House"}
out = []
for p in place_all():
    if p.name in HOLES:
        c = p.pos + fwd(p.heading) * 3.0
        out.append(f"    // A hobbit hole's garden.\n    shelf({c[0]:.1f}, {c[1]:.1f}, 4.5, {ground(p.pos):.1f}),")
    elif p.name == "mill":
        out.append(f"    // The mill, by the water.\n    shelf({p.pos[0]:.1f}, {p.pos[1]:.1f}, 5.5, {lowest(p.pos, 3.0) + 0.6:.1f}),")
    elif p.name in RAISE:
        g = lowest(p.pos, reach(p.name))
        out.append(f"    // {NAMES[p.name]}.\n    knoll({p.pos[0]:.1f}, {p.pos[1]:.1f}, {RAISE[p.name]:.1f}, {g + LIFT:.1f}),")
for p in PROPS:
    if p.name in RAISE and p.name not in ("bag_end", "green_dragon"):
        g = lowest(p.pos, reach(p.name, p.scale))
        out.append(f"    // {NAMES[p.name]}.\n    knoll({p.pos[0]:.1f}, {p.pos[1]:.1f}, {RAISE[p.name]:.1f}, {g + LIFT:.1f}),")
print(len(out))
print("\n".join(out))
