# Asset credits

All textures come from [Poly Haven](https://polyhaven.com) under the
[CC0](https://creativecommons.org/publicdomain/zero/1.0/) licence (public
domain): they can be used, modified and redistributed without permission or
attribution. Thank you anyway.

They have been downsized and retouched (more saturation, less contrast in the
asphalt cracks and the grass recoloured to a brighter green) to bring them
closer to the look of 90s games. Two of them are only the base of textures
painted at startup: blades of grass are painted over the grass photo, and the
stone wall photo gives its fine grain to a procedural wall of rough stones.

| File | Poly Haven original |
|---|---|
| `textures/grass.jpg` | [Leafy Grass](https://polyhaven.com/a/leafy_grass) |
| `textures/rock.jpg` | [Marble Cliff 03](https://polyhaven.com/a/marble_cliff_03) |
| `textures/wall.jpg` | [Stone Wall](https://polyhaven.com/a/stone_wall) |
| `textures/asphalt.jpg` | [Asphalt 02](https://polyhaven.com/a/asphalt_02) |
| `textures/sand.jpg` | [Coast Sand 01](https://polyhaven.com/a/coast_sand_01) |
| `textures/roof.jpg` | [Clay Roof Tiles](https://polyhaven.com/a/clay_roof_tiles) |
| `textures/plaster.jpg` | [Plastered Wall](https://polyhaven.com/a/plastered_wall) |

## The Moon, the Earth and the Milky Way

The maps of the Moon, the Earth and the Milky Way in the space sky are NASA imagery, in the
public domain, downsized to 1024 × 512 (but the Milky Way, see below):

| File | Original |
|---|---|
| `textures/moon.jpg` | [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720) colour map, Lunar Reconnaissance Orbiter (NASA's Scientific Visualization Studio) |
| `textures/earth.jpg` | [Blue Marble: land surface, shallow water and shaded topography](https://visibleearth.nasa.gov/images/57752) (NASA Earth Observatory) |
| `textures/earth_clouds.jpg` | [Blue Marble: clouds](https://visibleearth.nasa.gov/images/57747) (NASA Earth Observatory) |
| `textures/milky_way.jpg` | [Deep Star Maps 2020](https://svs.gsfc.nasa.gov/4851), the Milky Way alone in galactic coordinates, 4096 × 2048 (converted from its EXR), cut to the band 45° either side of the galactic plane (NASA's Scientific Visualization Studio, from ESA's Gaia and other surveys) |

## Models

The bikes and their riders (`models/shogun`, `models/kraken`, `models/falcon`,
`models/stiletto`, `models/azura`, `models/legend`, `models/cobra` and
`models/ronin`) were made by the project's author with
[Meshy](https://www.meshy.ai) from their own design sheets, and the riders use
the skeleton from Meshy's auto-rigging. The originals are attached to the
repository's [model releases](https://github.com/germadev/motorust/releases);
`tools/import_models.py` splits them into parts, scales them to the game's
measurements and downsizes their textures.

The trees (`models/trees/`: stone pine, cypress, olive, tall forest tree,
leafy broadleaf, maritime pine, poplar, almond, Nordmann fir, and the
Japanese momiji, ginkgo, sakura, black pine, sugi and bamboo) were made by
the project's author with [Meshy](https://www.meshy.ai);
`tools/import_trees.py` stands them on the ground at the game's sizes and puts
their colour textures together.

The props of feudal Japan (`models/props/`: the great torii, the castle keep,
the temple hall and its pagoda, the shrine on stilts, the ancient and the
great maple, the torii of the corridor, a log torii, a wayside shrine, a
Jizō, a komainu, bronze and stone lanterns, a memorial stone, a brazier, a
supply pile, a fishing boat, a minka and a water wheel) were made by the
project's author with [Meshy](https://www.meshy.ai), the originals attached
to the repository's [assets release](https://github.com/germadev/motorust/releases/tag/assets_pj);
`tools/import_props.py` mends those that came out wrong, stands them on the
ground at the game's sizes and puts their colour textures together.

The toys of the toy store (`models/toys/`: a big and a small castle of
bricks, a pirate ship, an electric train's locomotive, coach and freight
wagon, a teddy bear, a tin robot, a rubber duck, a spinning top, a biplane,
a helicopter, a plush giraffe, a doll's house, a fire engine, a mechanical
pegasus, a crocodile monster truck, a garage playset, a railway station, a
tunnel mountain, a rocking horse, a nutcracker, and in `models/toys2/` a
dump truck, a teepee, a tyrannosaurus, a rocket and a toy chest) were made
by the project's author with
[Meshy](https://www.meshy.ai); `tools/import_toys.py` stands them on the
ground at the game's sizes, turns those made facing sideways and puts their
colour textures together.

The flowers, the lawn in flower and the snow are painted in code.

## Font

`fonts/RussoOne-Regular.ttf` is [Russo One](https://fonts.google.com/specimen/Russo+One)
by Jovanny Lemonad, licensed under the SIL Open Font License 1.1
(`fonts/RussoOne-OFL.txt`).

## Gamepads

The gamepads drawn in Options > Control > Mapping (`ui/pads/dualsense.svg`,
`ui/pads/dualshock4.svg`, `ui/pads/dualshock3.svg` and `ui/pads/xbox.svg`) were
designed by the project's author, with Claude, for
[anygamepad](https://github.com/anygamepad/anygamepad.github.io)
(`apps/web/src/art`), licensed under either the MIT or the Apache-2.0 licence.
They are used as they are; the game draws them with resvg (`src/pad_art.rs`).
