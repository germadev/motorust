# Asset credits

The game's own content (this folder). The textures, the maps of the Moon,
the Earth and the Milky Way, the font and the gamepads' art are the
engine's, credited in `motorust-engine/assets/CREDITS.md`.

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
dump truck, a teepee, a tyrannosaurus, a rocket, a toy chest, a
jack-in-the-box, a xylophone, a giant robot, a checkout counter, a
shopping trolley and a potted plant) were made
by the project's author with
[Meshy](https://www.meshy.ai); `tools/import_toys.py` stands them on the
ground at the game's sizes, turns those made facing sideways and puts their
colour textures together.

The flowers, the lawn in flower and the snow are painted in code.

