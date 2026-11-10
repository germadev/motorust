#!/usr/bin/env python3
"""Prepares Puerto Neón's props modelled with Meshy for the game: the start
gate, the towers, the screens, the grandstand, the port's cranes and
ship, the control tower, the power plant and the airships (see
docs/puerto-neon-props.md).

Reads one GLB per prop and writes to `assets/models/neon/` what the game
loads (`src/game/neon.rs`), as `tools/import_props.py` does Ghost Island's,
whose code it runs with its own sizes: `props.bin`, every prop as a piece
named after it, centred on the origin with its lowest point on the ground,
facing +Z, at the game's size (`SIZES`: by its height, or its length
across, for the gate and the grandstand); and
`color.jpg`, their colour textures together, 1024 pixels square each, in a
grid of four.

Every prop has to be given each time (there is no `--keep`).

Usage: python3 tools/import_neon.py NAME=PROP.glb [NAME=PROP.glb ...]
with the names of `SIZES`. Needs numpy and Pillow, and
tools/import_props.py and tools/import_trees.py beside it.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import import_props  # noqa: E402

SIZES = {
    # The gate over the finish line, 38 m across: its legs lean in, their
    # feet 16 m from its middle inside, and still over 15 m at a rider's
    # height, clear of the barriers 11.5 m out.
    "start_gate": ("length", 38.0),
    "tower_spire": ("height", 200.0),
    "tower_stepped": ("height", 150.0),
    "tower_twin": ("height", 120.0),
    "screen_mast": ("height", 14.0),
    "grandstand": ("length", 40.0),
    "harbour_crane": ("height", 60.0),
    "control_tower": ("height", 45.0),
    "fusion_plant": ("height", 35.0),
    "container_ship": ("length", 180.0),
    "airship_cobalt": ("length", 60.0),
    "airship_neon": ("length", 60.0),
}

if __name__ == "__main__":
    import_props.SIZES = SIZES
    import_props.SLIM = {}
    import_props.TONED = {}
    import_props.OUT = import_props.ROOT / "assets" / "models" / "neon"
    import_props.CELL = 1024
    import_props.COLUMNS = 4
    import_props.__doc__ = __doc__
    import_props.main()
