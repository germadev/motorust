# Bikes, riders and textures

Where the bikes, riders and textures come from, how the Meshy models are prepared for the game and how to add a new bike. For how the model code fits into the rest of the game see [architecture.md](architecture.md); for racing them see [gameplay.md](gameplay.md).

## The race bikes

The race uses eight superbikes made with [Meshy](https://www.meshy.ai) from the author's design sheets, each with its own rider, also made with Meshy and rigged with Meshy's auto-rigging. You pick one and the other seven are your rivals. They are listed in `ROSTER` (`src/bike/mod.rs`) in this order, which is also the order on the selection screen:

| Bike | Colours | Folder | `Design` | Notes |
|---|---|---|---|---|
| Shogun TZ | Blue and white, like the Moto Racer cover bike | `assets/models/shogun` | `Design::Shogun` | The first one. Its rider measurements were taken by hand and serve as the reference for the importer. |
| Legend 88 | White and red | `assets/models/legend` | `Design::Legend` | The fork is welded to the fairing, so the steering does not turn (`steers: false`). |
| Azura RZ | Yellow, white and black | `assets/models/azura` | `Design::Azura` | |
| Kraken RS | Lime green and black | `assets/models/kraken` | `Design::Kraken` | |
| Falcon KRZ | Red, blue and white | `assets/models/falcon` | `Design::Falcon` | |
| Stiletto V4 | Red | `assets/models/stiletto` | `Design::Stiletto` | |
| Cobra XT | Orange and black | `assets/models/cobra` | `Design::Cobra` | The silencer sits under the tail. |
| Ronin YS | Blue and black | `assets/models/ronin` | `Design::Ronin` | |

Each `ROSTER` entry also has a description shown on the selection screen (in Spanish, for example "Azul y blanca, como la de la portada de Moto Racer") and a colour, used for the bike's name and its dot on the minimap.

Each imported bike has colour, metal/roughness and normal textures and is split into chassis, steering and wheels, so the wheels spin, the fork and handlebars turn, and the suspension works. The exhausts have open outlets: the lip of the inner pipe and a hole that darkens to black inside. The rider is posed every frame (`src/bike/model/pilot.rs`): hips on the seat, back over the tank on the straights with the helmet behind the screen, hanging off in corners with the inner knee towards the ground, head turned towards where the bike is going and kept more upright than the bike, and sitting up under braking. Arms and legs use two-bone inverse kinematics, so the hands stay on the grips and the feet on the pegs.

The original GLB files are attached to [the repository's model releases](https://github.com/germadev/motorust/releases). The game does not read them directly: it embeds what `tools/import_models.py` leaves in `assets/models/<folder>/`.

## Code-built designs

The code also keeps four low-poly bikes built entirely in code, each with its code-built rider and liveries. They are not in `ROSTER`, so they no longer appear in races, but they are still built at startup (`Design::ALL`) and can be put back in the roster.

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

Every `ROSTER` entry names a livery, but it is only painted on code-built designs: the imported bikes bring their own textures.

## The importer

`tools/import_models.py` turns the Meshy GLB files of a bike and its rider into the files the game embeds. It needs Python 3 with numpy and Pillow.

```sh
python3 tools/import_models.py NAME BIKE.glb RIDER.glb [FOLDER]
```

It writes to `assets/models/NAME/` (or `FOLDER`) and, when it finishes, prints the bike's measurements as a `Geometry { ... }` block plus the `steers` flag, ready to paste into `src/bike/model/imported.rs`.

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
3. **`src/bike/model/imported.rs`:** add a `Model` with the `model!` macro, the folder name and the printed `Geometry`; pass `false` as a third argument if the script printed `steers: false`. Add it to the `MODELS` arrays used by the tests here and in `pilot.rs`.
4. **`src/bike/model/designs.rs`:** add a variant to `Design`, append it to `Design::ALL` (the order must match the variant index; a test checks it), map it to its model in `Design::model()`, and add it to the imported-bike arms of `details()`, `build()` and `plates()`. The compiler points out every `match` that is missing it.
5. **`src/bike/mod.rs`:** add an `Entry` to `ROSTER` with the name, the description, the colour, the `Design` and a livery (only used by code-built designs). `ROSTER` is a fixed-size array (`[Entry; 8]`) and the starting grid (`GRID`) has eight slots, one per bike, checked by a test; a ninth bike also needs a ninth `GRID` slot (a `Rival` with its skill and lane) and that test updated.
6. **Test.** `cargo test` checks that every imported bike has its parts, that the wheels touch the ground with the measured radii, that the exhausts open behind the seat, that the rider has a rigged 23-bone body, and that the hands reach the grips and the feet the pegs whether the rider is tucked in, sitting up or hanging off to either side.

Upload the original GLB files to a model release of the repository so they can be re-imported later, and credit them in `assets/CREDITS.md`.

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

All of these, like the bike models, are embedded in the executable, so you do not need to copy the `assets` folder to play. See [`assets/CREDITS.md`](../assets/CREDITS.md) for the original Poly Haven sources and licences.
