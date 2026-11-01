# Bikes, riders, trees, props and textures

Where the bikes, riders, trees, props of feudal Japan and textures come from, how the Meshy models are prepared for the game and how to add a new bike. For how the model code fits into the rest of the game see [architecture.md](architecture.md); for racing them see [gameplay.md](gameplay.md).

## The race bikes

The race uses eight superbikes made with [Meshy](https://www.meshy.ai) from the author's design sheets, each with its own rider, also made with Meshy and rigged with Meshy's auto-rigging. You pick one and the other seven are your rivals. They are listed in `ROSTER` (`src/bike/mod.rs`) in this order, which is also the order on the selection screen:

| Bike | Colours | Folder | Its `design` in the roster | Notes |
|---|---|---|---|---|
| Shogun TZ | Blue and white, like the Moto Racer cover bike | `assets/models/shogun` | `Model("shogun")` | The first one. Its rider measurements were taken by hand and serve as the reference for the importer. |
| Legend 88 | White and red | `assets/models/legend` | `Model("legend")` | The fork is welded to the fairing, so the steering does not turn (`steers: false` in its `model.ron`). |
| Azura RZ | Yellow, white and black | `assets/models/azura` | `Model("azura")` | |
| Kraken RS | Lime green and black | `assets/models/kraken` | `Model("kraken")` | |
| Falcon KRZ | Red, blue and white | `assets/models/falcon` | `Model("falcon")` | |
| Stiletto V4 | Red | `assets/models/stiletto` | `Model("stiletto")` | |
| Cobra XT | Orange and black | `assets/models/cobra` | `Model("cobra")` | The silencer sits under the tail. |
| Ronin YS | Blue and black | `assets/models/ronin` | `Model("ronin")` | |

The roster, `assets/bikes/roster.ron` (read as the game starts into the `Roster` resource, `src/bike/mod.rs`), lists them in the order of the selection screen, each naming its model by its folder (`design: Model("shogun")`, `Design::Model`). The code names none of them: each folder's `model.ron` holds its measurements (axles, wheel radii, steering head, grips, pegs and seat), whether its steering turns (`steers`) and its slicks (`tyres`: `Soft`, `Medium` or `Hard`), read the first time the roster names it (`imported::model`). Each entry also has a description shown on that screen (in Spanish, for example "Azul y blanca, como la de la portada de Moto Racer") and a colour, used for the bike's name and its dot on the minimap.

Each imported bike has colour, metal/roughness and normal textures and is split into chassis, steering and wheels, so the wheels spin, the fork and handlebars turn, and the suspension works. The tyres that come with the Meshy wheels are lumpy, low-poly rings (their outline wanders by a couple of centimetres, and some carry smudges of the bodywork's colours), so the game cuts them off the wheel and fits round ones made in code (`src/bike/model/tyre.rs`): a surface of revolution 96 segments round, as wide as the old one, sitting on the rim, with the round profile of a MotoGP tyre: an ellipse from one shoulder to the other, the tread running nearly round to the rim (the bike leans over 60 degrees on it) and short sidewalls. They are slicks, as MotoGP races on in the dry (Michelin's Power Slicks): no tread pattern, the rubber a little lighter where it is scrubbed in the middle, and the compound marked as in MotoGP by a band round the sidewall: white on the soft (Kraken RS, Stiletto V4, Azura RZ), none on the medium (Shogun TZ, Falcon KRZ, Ronin YS) and yellow on the hard (Legend 88, Cobra XT), as each model's `model.ron` says (`tyres`). The exhausts have open outlets: the lip of the inner pipe and a hole that darkens to black inside. The rider is posed every frame (`src/bike/model/pilot.rs`): hips on the seat, back over the tank on the straights with the helmet behind the screen, hanging off in corners with the inner knee towards the ground, head turned towards where the bike is going and kept more upright than the bike, and sitting up under braking. Arms and legs use two-bone inverse kinematics, so the hands stay on the grips and the feet on the pegs.

The original GLB files are attached to [the repository's model releases](https://github.com/germadev/motorust/releases). The game does not read them directly: it embeds what `tools/import_models.py` leaves in `assets/models/<folder>/`.

## Code-built designs

The code also keeps four low-poly bikes built entirely in code, each with its code-built rider and liveries. They are not in the roster, so they no longer appear in races, but they are still built at startup (`Design::all`) and can be put back in it (`design: Afilada`, ...).

| `Design` | Shape |
|---|---|
| `Design::Afilada` | Sharp nose with slit headlights on the sides, mirrors on the screen and a short exhaust beside the wheel. |
| `Design::Escudo` | Vertical front, long belly pan and a long exhaust rising towards the tail. |
| `Design::Alerones` | Low nose with winglets, blade-like tail, exhaust under the engine and a single-sided swingarm. |
| `Design::Rombo` | Central diamond-shaped headlight, dark belly pan, mirrors and a large hexagonal exhaust. |

- **Bodywork** (`src/bike/model/designs.rs`): each design is a list of faceted parts with measurements taken from the side view of a low-poly design sheet, scaled to the bike's wheelbase. They carry no logos, only the shape. Vents, grilles and pinstripes are painted onto the fairing as details. Parts shared by all designs (frame, engine, swingarm, pegs) live in `common`; wheels, fork and rider are the same for every code-built bike.
- **Rider** (`src/bike/model/rider.rs`): a single skinned mesh deformed by a ten-bone skeleton, plus rigid parts attached to the bones (helmet, hump, boots, gloves and knee sliders).
- **Liveries** (`src/bike/model/livery.rs`): colours, fairing graphics, suit and helmet, described as data and painted into textures at startup, with no image files. The fairing is painted on a side view of the bike and projected sideways onto every body part, as many games of the era did.

| Livery | Look |
|---|---|
| `COVER` | Blue and white like the Moto Racer cover bike, with the number 8 on a yellow plate. |
| `FLECHA` | Bright white with a fluorescent red top forming an arrow on the sides; yellow helmet with a red stripe over a black one. |
| `DIAGONAL` | Yellow front turning blue diagonally, with a light blue pinstripe; white helmet with blue and yellow stripes. |
| `VERDE` | Lime green with the fairing opening in black. |
| `TRICOLOR` | Red with a blue side and white pinstripes. |
| `ROSSO` | Red with a silver tank top. |

Every entry of the roster names a livery (`cover`, `flecha`, `diagonal`, `verde`, `tricolor`, `rosso`, named in `livery` in `src/bike/mod.rs`), but it is only painted on code-built designs: the imported bikes bring their own textures.

## The importer

`tools/import_models.py` turns the Meshy GLB files of a bike and its rider into the files the game embeds. It needs Python 3 with numpy and Pillow.

```sh
python3 tools/import_models.py NAME BIKE.glb RIDER.glb [FOLDER]
```

It writes to `assets/models/NAME/` (or `FOLDER`), the bike's measurements and its `steers` flag in `model.ron` (its `tyres` kept if the file was there, `Medium` if not; the comment at its top is written anew), and prints them when it finishes.

### Outputs

| File | Contents |
|---|---|
| `bike.bin` | The bike in the game's axes and scale, as five parts: `chassis`, `steering`, `front_wheel`, `rear_wheel` and `exhaust` (the silencer outlets). |
| `base_color.jpg` | Colour texture, reduced to 2048 × 2048. |
| `metallic_roughness.jpg` | Metal and roughness texture, 1024 × 1024. |
| `normal.jpg` | Normal map, 1024 × 1024. |
| `pilot.bin` | The rider mesh with its bone weights, and its rest-pose skeleton. |
| `pilot_color.jpg` | The rider's colour texture, 1024 × 1024. |

Reducing the textures takes each bike and its rider from about 50 MB to about 3 MB. The `.bin` files use a small little-endian format (`MRMESH01` meshes, optionally followed by an `MRSKEL01` skeleton) documented at the top of the script and read by `imported::read`.

### The bike

1. **Pieces.** The Meshy bike is a single mesh made of 90 to 200 loose pieces. The script finds them (connected triangles, welding vertices that coincide) and classifies whole pieces, never cutting triangles.
2. **Wheels.** The tyres are the large pieces touching the ground; their bounding boxes give the axles and radii.
3. **Steering axis.** A line fitted through the two fork tubes. If there are no separate fork tubes because the fork is welded to the fairing (as on the Legend 88), the script warns, estimates the axis from the average rake (24°) and axle-to-head distance (0.66 m) of the other bikes, and sets `steers: false`. The wheels still spin, and at speed the handlebars barely turn anyway.
4. **Classification.** Tyres, and pieces centred on an axle that fit inside the wheel, turn with that wheel. Pieces around the steering axis (fork tubes, triple clamps, calipers, front mudguard and clip-ons) turn with the steering. Everything else is chassis.
5. **Axes and scale.** The bike is converted to the game's axes (forward -Z, right +X, ground at Y = 0) and scaled so the wheelbase is 1.38 m, with the front axle at Z = -0.70.
6. **Measurements.** Front and rear axles and wheel radii, steering head, grips (the outer end of the clip-ons, moved slightly inwards for the hands), seat (the lowest point of the top of the chassis between the tank and the tail) and, from the seat, the footpegs. The rider offsets from seat to hips and ankles were taken from the Shogun TZ.
7. **Silencers.** Meshy's silencers come closed, with a solid end cap. The script recognises them: chassis pieces that are elongated towards the back, roughly round, to one side and with an end cap away from the rear axle (so swingarms, subframes and linkages are not mistaken for them), or, like the Cobra XT's, centred under the tail. It then opens a mouth like a racing silencer: a hole shaped like the cap (round, oval, trapezoidal...) surrounded by the metal lip of the inner pipe, which stands slightly proud of the cap and darkens inside, with soot, to the black of the bottom. The outlets go into the separate `exhaust` part, which uses its own material (`imported::exhaust_material`). If it finds no recognisable silencer it says so and opens none.

### The rider

The rider GLB must be the auto-rigged, skinned one. The script keeps the mesh with its bone weights and the skeleton in its rest pose (23 bones with Mixamo names: hips, spine, neck, head, arms and legs), turned half a turn to face -Z like the bike and scaled from Meshy's 1.64 m to 1.75 m.

## Adding a new bike

1. **Make the models.** From Meshy, download the textured bike GLB and the auto-rigged rider GLB (the one with a skin).
2. **Import them.**

   ```sh
   python3 tools/import_models.py NAME bike.glb rider.glb
   ```

   Check the output: the triangle count of each part, the exhaust outlets found, and any "fork welded to the fairing" warning.
3. **`model.ron`:** check its measurements, and set its slicks (`tyres`).
4. **`assets/pack.txt`:** list its folder's seven files (`model.ron`, `bike.bin`, `base_color.jpg`, `metallic_roughness.jpg`, `normal.jpg`, `pilot.bin`, `pilot_color.jpg`), so that a release build has them (a development build refuses a file the list leaves out).
5. **`assets/bikes/roster.ron`:** add an entry with the name, the description (Spanish and English), the colour (sRGB), its model by its folder (`design: Model("NAME")`) and a livery (only used by code-built designs). The roster has one bike for each place on the starting grid (`GRID`, eight, `RIDERS`), checked as it is read and by a test; a ninth bike also needs a ninth `GRID` slot (a `Rival` with its skill and lane), and the championship's points (`POINTS`, `Championship::points`) one more.
6. **Test.** `cargo test` checks, for every model the roster names, that the bike has its parts, that the wheels touch the ground with the measured radii, that the exhausts open behind the seat, that the rider has a rigged 23-bone body, and that the hands reach the grips and the feet the pegs whether the rider is tucked in, sitting up or hanging off to either side.

Upload the original GLB files to a model release of the repository so they can be re-imported later, and credit them in `assets/CREDITS.md`.

## Trees

The trees were made with [Meshy](https://www.meshy.ai) too, one for each species the tracks plant (stone pine, cypress, olive, tall forest tree, leafy broadleaf, maritime pine, poplar, almond, and Ghost Island's momiji, ginkgo, sakura, Japanese black pine, sugi and bamboo) and a Nordmann fir that grows among the tall trees of the forests. Each is one mesh of about four hundred triangles with a colour texture (Meshy's normal and metal-roughness maps are left out: trees are seen from afar, and their shape is in their normals). Options > Graphics chooses them ("New", the default) or the classic trees built in code (`src/world/props.rs`).

`tools/import_trees.py` turns the GLB files into what the game embeds (`assets/models/trees/`):

```sh
python3 tools/import_trees.py umbrella_pine=PINE.glb cypress=CYPRESS.glb olive=OLIVE.glb \
    tall=TALL.glb broadleaf=LEAFY.glb maritime_pine=MARITIME.glb poplar=POPLAR.glb \
    almond=ALMOND.glb fir=FIR.glb momiji=MOMIJI.glb ginkgo=GINKGO.glb sakura=SAKURA.glb \
    black_pine=BLACK_PINE.glb sugi=SUGI.glb bamboo=BAMBOO.glb
```

With `--keep`, the trees already in the pack stay as they are (their meshes, and their cells of the texture moved to the new grid) and only those given are added or replaced, so the GLBs of the others need not be at hand: `python3 tools/import_trees.py --keep sakura=SAKURA.glb`. Some models are mended as they are read (`MENDS`): Meshy's black pine came nearly flat, a quarter as deep as it is wide, so its foliage (its green texels, above its roots) is spread out three times as deep round its trunk, which keeps its shape.

- `trees.bin`: each tree standing with the foot of its trunk at the origin, as tall as the classic trees of its species (`HEIGHTS`: from 4.8 m for the almond to 16.1 m for the tall forest tree; 14 m for the fir, 16 m for the sugi), in the bikes' pack format.
- `color.jpg`: their colour textures together, 512 × 512 each with a 2-pixel margin, in a 4 × 4 grid, so every tree shares one texture and one material.

All fifteen take about 1.1 MB. `cargo test` checks that every tree is in the pack, standing on the ground, with a few hundred triangles and its texture coordinates in its own cell.

Far from the cameras the trees are drawn with lighter meshes made from these when the game starts (`src/world/lod.rs`): the same shape with a third of the triangles, and farther still with a tenth, painted in their vertices with the colours of their texture as it is seen from afar, so they need no texture.

## The props of feudal Japan

Ghost Island's buildings, landmarks, statues, lanterns and gates were made with Meshy by the author too: the great torii in the sea, the castle keep, the temple hall and its pagoda, the shrine on stilts, an ancient maple and the great maple over its clearing, the two torii of the corridor (a wide one over the road and a narrow one over the corridor), a log torii, a wayside shrine (hokora), a Jizō, a komainu, bronze and stone lanterns, a memorial stone, a brazier, a pile of rice bales and barrels, a fishing boat, a thatched minka and a water wheel. Where each stands is in the track's data (see [tracks.md](tracks.md#scenery), `japan`), and `src/game/japan.rs` places them.

`tools/import_props.py` turns the GLB files into what the game embeds (`assets/models/props/`), each by its name in `SIZES`:

```sh
python3 tools/import_props.py o_torii=O_TORII.glb castle_keep=KEEP.glb temple_hall=TEMPLE.glb \
    pagoda=PAGODA.glb torii_wide=WIDE_FUSHIMI.glb torii_road=FUSHIMI_ROAD.glb ...
```

- `props.bin`: every prop as a piece named after it, centred on the origin with its lowest point on the ground, up along +Y, facing +Z, at the game's size (`SIZES`): by its height, its length (the boat) or, for the gates, the opening between the feet of their pillars, so that they stand round the road's walls (11.63 m for the wide torii) or the corridor's (6.6 m for the narrow one); the great maple stands with its trunk, not its spreading crown, at the origin.
- `color.jpg`: their colour textures together, 512 × 512 each, in a 5 × 5 grid: one texture and one material for every prop.

Some came out of Meshy wrong, and are mended as they are read: the great torii stood on a slab of blue water, which is cut away (the game has its own sea); the torii over the road came so stout that one after another they read as a wall, so their pillars are made slimmer and the gates shallower (`SLIM`); the water wheel came in one piece with its post, so it is split into the wheel and its axle, which turns, and the post (`water_wheel_frame`), both with their origin at the middle of the axle; and the great maple's reds came so saturated they glared, so its texture is toned down and a little lightened (`TONED`). All of them take about 3.7 MB. `cargo test` checks that every prop is in the pack, standing on the ground, and not too heavy to draw.

The flowers of Ghost Island are not models: they are cards cut out of an atlas painted at startup (`src/world/foliage.rs`), see [tracks.md](tracks.md#scenery).

## Props of Middle-earth

Fellowship Road's props were made with Meshy by the author too: four hobbit holes (green, yellow, red and blue doors), Bag End, a chimney, the party tree, the oak over Bag End, the mill and its wheel, the Green Dragon and its sign, Gandalf's cart, a log pile, a scarecrow, the old tree the hobbits hid under, the Brandywine's ferry, Bree's gate, the Prancing Pony, two houses of Bree, the ruins of Amon Sûl, a broken arch, the stone trolls, the Last Homely House, an elven tower, Rivendell's pavilion and its lamps, the Doors of Durin, a mallorn with its flet, Galadriel's swan ship, an elven ship and boat, the two kings of the Argonath and the seat of seeing on Amon Hen. Where each stands is in the track's data (see [tracks.md](tracks.md#scenery), `middle_earth`), and `src/game/middle_earth.rs` places them.

`tools/import_fellowship.py` turns the GLB files into `assets/models/fellowship/` as `tools/import_props.py` does Ghost Island's (`props.bin`, and `color.jpg`, an atlas 4096 pixels square: each prop's texture 512 × 512, or, for the big buildings the riders pass close by, at the full size Meshy made it (the Prancing Pony, 2048 × 2048, whose archway the riders see from close) or 1024 × 1024 (the Last Homely House, the Green Dragon, Bag End, a house of Bree), packed as quarters of quarters), each by its name in `SIZES` (`mill_wheel` too), sized by its height, its length (boats and the ring of ruins), its opening (Bree's gate, round the road's walls) or, for the great trees, standing on the foot of its trunk. It mends what came wrong:

- the stone trolls: the left one had a third leg out of its back; it is cut away and the hole closed with a fan of triangles in its stone's colour;
- the Prancing Pony: its archway, through which a way of Bree's road runs, is made wider (`PONY_ARCH`): the inn stretched along its width about it, the rest of it moved out either side, 3.3 m either side of its middle as the inn is sized (4.6 m in the game, where it stands 1.4 times as big);
- Bree's gate: two beams across its opening, where the bikes ride, are cut away; a house of Bree came hung with the Green Dragon's sign, cut away too, as are the elven boat's oars, so that it lies on the shore;
- the mill and its wheel came apart: the mill is turned so the hole for the axle in its side faces the road, and the wheel set on it, both about the axle, which turns;
- the Doors of Durin came closed, and narrower than drawn in the film: made 1.4 times wider and 35 m high, their two leaves (two of the model's five parts) swung open outwards a quarter turn about their hinges, so the road goes through them, and two pieces made with them: `durin_ithildin`, a drawing of shining silver of our own laid on the stone (the arch, a vine winding up each pillar with lozenges for leaves, a mountain of two chevrons on the keystone, a crescent moon between six lozenges on the rock above, and on each leaf's side turned to the road a tall lozenge round a rayed sun over a stem of chevrons; it is not, and is not meant to be, the drawing of the books or the films), which the game draws emissive, and `durin_glow`, the same drawn wider, added round it as a glow;
- the oak over Bag End came a vivid green and the Doors of Durin a pale grey: toned (`TONED`).

All of them take about 10.4 MB. `cargo test` checks that every prop is in the pack, standing on the ground and not too heavy to draw, the mill's wheel turning clear of its wall, Bree's gate leaving the road clear under its beam, the Doors of Durin open with their drawing on them, and that every prop of the track stands clear of the roads, on land or on the water as it should (but for the gates over the road and the Prancing Pony over its way, nothing of it as high as a rider within that way's walls).

## Toys of the toy store

Juguetería's toys are being made with Meshy by the author, one by one. In so far: the big castle of bricks on the diorama table (`castle_big`, 7,192 triangles) and the small one, a round tower under a red roof (`castle_small`), the pirate ship, the electric trains' locomotive, coach and freight wagon, the teddy bear, the tin robot, the rubber duck, the spinning top, the biplane and the helicopter, the plush giraffe, the doll's house, the fire engine, a mechanical pegasus (`mech_pegasus`), a crocodile monster truck (`croc_car`), the garage playset, the railway station, the tunnel mountain, the rocking horse and the nutcracker; and, in a second pack, the dump truck, the teepee, a tyrannosaurus (`trex`, 28 m long so that its head, reaching out over the road, is well over the bikes), the rocket, the toy chest, a jack-in-the-box, a xylophone, a giant robot (`giant_robot`, 26 m tall, 5,239 triangles), and for the shop's way in the checkout counter (30 m long), a shopping trolley (12 m tall, made facing -X) and a potted plant (20 m), 790 to 5,200 triangles each. The rest of the store's toys (beach balls, lettered blocks, stacks of tyres, and the plastic building bricks, `src/game/bricks.rs`, which are better built in code: their studs regular and as far apart as the diorama's) are built in code (`src/game/toys.rs`), to be replaced or joined by more as their models come. Where each stands is in the track's data (see [tracks.md](tracks.md#scenery), `toys`): a castle no road runs through is Meshy's, `castle_big` or `castle_small` by its `kind`, as big as fits in its `size`, its gate where its `heading` says; a toy of a kind Meshy made is its model (`toys::model_of`), `scale` times the size it was made at, its front where its `heading` says; the trains are Meshy's locomotive and coaches or freight wagons, each car as long as it was made (the coach longer), on rails as far apart as their wheels (1.4 m); the biplane and the helicopter hang on strings tied on at their tops (on the biplane's upper wing either side, on the helicopter's rotor mast). The toys are drawn whole, with Meshy's own triangles and colour texture, sharing one mesh per model and one material for all, so that every copy of a toy is drawn in one go.

`tools/import_toys.py` turns the GLB files into two packs as `tools/import_props.py` does Ghost Island's (`props.bin`, and `color.jpg`, an atlas 4096 pixels wide that grows downwards): `assets/models/toys/`, 4096 × 4096 and full with the twenty-two of `FIRST_PACK`, and `assets/models/toys2/` for every other toy (4096 × 2048 and full with the eleven in it: seven big cells and four small; the next toy makes it 4096 × 3072). Each pack has its own atlas and material (`toys::ToyModels` finds a toy in either); a second material costs a few more draw calls, but a full atlas, with its mipmaps, takes about 85 MB of the graphics card's memory, loaded at startup for every circuit. Each toy goes by its name in `SIZES` (by its height, or its length for a ship, a train or a plane: the coach 10.5 m, the locomotive and the wagon 7 m), its texture in a cell 1024 pixels square for the big toys the riders pass close by (`CELLS`: the castles, the ship, the giraffe, the doll's house, the bear, the robot, the fire engine, the pegasus, the garage, the station, the nutcracker, the biplane and the helicopter) and 512 for the small ones (the duck, the top, the trains, the croc car, the tunnel mountain, the rocking horse); Meshy's textures are 2048 pixels, so close up the toys are a little softer than in Meshy's viewer, and the normal and metal-roughness maps are left out. The ship, the trains, the biplane, the helicopter, the fire engine and the croc car came facing -X (their bows, chimneys, noses and cabs), and are turned a quarter turn as they are read to face +Z as the others (`TURNED`); the tunnel mountain, 40 m long so that its tunnel is 4.4 m wide, is centred on its tunnel rather than on its box (`TUNNELS`, `tunnel_middle`: the middle of the widest stretch across it, a metre up, through which a line along it meets none of its triangles), so that a train's track along its middle runs through it:

```sh
python3 tools/import_toys.py castle_big=CASTLE.glb
python3 tools/import_toys.py --keep teddy_bear=BEAR.glb   # add one, keeping the others
```

With `--keep`, the toys already in the pack stay as they are (their meshes, and their cells of the texture moved to the new atlas), so the GLBs of the others need not be at hand; a toy whose cell changes size has to be given again. Each should have no more than 8,000 triangles (Meshy's remesh), fewer the further from the road it stands and the more copies of it there are. All twenty-seven take about 10 MB. `cargo test` checks that every toy the game places is in the pack, that every piece of it stands on the ground, centred and light enough to draw, that the hanging toys' strings are tied on at their tops, and that the trains' wheels are about as far apart as their rails.

## Textures

The scenery textures are CC0 (public domain) photos from [Poly Haven](https://polyhaven.com), reduced in size and retouched to look closer to 90s games: more saturation, less contrast in the asphalt cracks and the grass recoloured to a brighter green. Mipmaps are computed at startup so distant surfaces do not flicker (`src/world/textures.rs`).

| File | Used for |
|---|---|
| `assets/textures/asphalt.jpg` | Road |
| `assets/textures/grass.jpg` | Grassy terrain and hedges |
| `assets/textures/rock.jpg` | Rocky terrain, crags and rock faces |
| `assets/textures/sand.jpg` | Beaches |
| `assets/textures/wall.jpg` | Stone walls |
| `assets/textures/roof.jpg` | Tiled roofs |
| `assets/textures/plaster.jpg` | Plastered and concrete surfaces: houses, hotels, walls and structures |
| *(procedural)* `crowd` | Pixelated grandstand spectators with vibrant retro clothing |
| *(procedural)* `checker` | Start/finish and tunnel checkerboard strip |
| *(procedural)* `panels` | Concrete tunnel lining panels |
| *(procedural)* sky | Sky dome: a gradient from deep blue overhead to the fog colour at the horizon, the sun, and cartoon clouds drawn with zeno (`src/world/sky.rs`) |

The labels use the [Russo One](https://fonts.google.com/specimen/Russo+One) font by Jovanny Lemonad, under the SIL Open Font License 1.1 (`assets/fonts/RussoOne-OFL.txt`).

All of these, like the bike models, are in the content pack (`assets/pack.txt`), built into the release executable, so you do not need to copy the `assets` folder to play. See [`assets/CREDITS.md`](../assets/CREDITS.md) for the original Poly Haven sources and licences.
