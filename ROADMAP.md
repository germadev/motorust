# Roadmap

Where Motorust is going next, in order. Each step is worked on in a branch of its own and recorded in `RECORD/` as it goes; this file says what is planned and why, and is updated as steps are done or change.

## 1. Measure first

The benchmark has not been run since 2026-10-16, before the relief of the textures, Fellowship Road, Base Lunar, Juguetería and the new web, and before the M4 mini's world was drawn at its panel's own pixels.

- `make bench` on the M4 mini, and on the M1 Pro, the M4 Pro and the BC-250, at the commit that adds the scenarios below (then `make bench.push`).
- **The ground in the shadows:** `1p` against `1p-no-ground-shadows`, and `2p` against `2p-no-ground-shadows`. The difference is the most cutting the ground into pieces could save in the shadow maps (step 2).
- **The edges:** `1p-msaa-2x` and `1p-smaa` against `1p` (MSAA 4×) and `1p-fxaa`: what each costs, and whether the game should start with another one (it starts with FXAA).
- **Juguetería**, the first track indoors: its ceiling's lamps glow but cast no shadow, and the room, the furniture, the toy tracks and the toys are batched into a few meshes. Measured as it is, and again once Meshy's toys fill it (see [Alongside](#alongside-jugueterías-toys)).
- The render world's CPU time apart from the simulation's, with Tracy (`--features bevy/trace_tracy`), where a machine is not bound by its GPU (the M4 Pro).

What the measurements said so far, machine by machine, is in `RECORD/2026-10-16.performance-plan-and-measuring-guide.completed.md`: the Macs are bound by the pixels they draw (the render scale and the antialiasing count most, the shadows less), and the simulation takes under a millisecond a frame since the nearest road is found through a grid.

## 2. The ground in pieces (if step 1 says it is worth it)

Each mesh of the ground (one for each kind of ground, or field of flowers) covers the whole circuit, so neither the cameras' culling nor the shadow cascades' leave any of it out: all of it is drawn in the main pass and in each cascade (four at high shadows), and again for each view with two players. Cut into squares of about 100–200 m, each sharing its kind's material, the cascades would draw only the squares they reach. The editor (section 3, step 5) needs it too: a change rebuilds only the squares near it.

Judged of little worth for now: compressed textures (KTX2: the textures that fill the screen are painted at startup; the bikes' would save about 300 MB of memory, not frames), GPU occlusion culling (Apple's GPUs already drop hidden opaque fragments; it needs a depth prepass, which costs pixels), and another engine (the simulation is cheap; the limit is the pixels, not Bevy).

## 3. The engine apart from the game

Two aims:

- **A map editor**, to lay out and dress circuits without writing code or compiling.
- **An open-source engine**, with the game's characters, models, music and circuits kept private.

Both ask for the same thing: an engine that knows nothing of the content it runs. Circuits written in RON are the first step, not the whole of it.

### What stands in the way today

- **The engine knows each circuit and each bike.** `CIRCUITS` (a `const` array) is used in 37 files and `ROSTER` in 6.
- **Themed code inside the engine.** `Scenery` has fields for each theme (`japan`, `middle_earth`, `space`, `desert`, `mine`, `pit`, `loops`, `rings`, `totems`, `toys`, …), served by about 13,100 lines of world code made for one circuit or another (`world/japan.rs`, `middle_earth.rs`, `space.rs`, `desert.rs`, `mine.rs`, `cavern.rs`, `landmarks.rs`, `toys.rs`).
- **Functions, not data.** Each circuit's `mountains`, and the coasts, lakes and rivers given as functions (`Coast::Function`, `OutlineWithLakes`, `Inland`): a volcano with its gullies, coasts as sums of sines over `fbm`. An editor cannot edit a function.
- **Values worked out from others.** `span(TUNNEL.from + 60.0, …)`, `const fn` helpers (`knoll`, `fence`, `curve_sign`); and the circuits' many comments. A file the editor writes keeps neither.
- **Every asset built into the engine's own code** with `include_bytes!`: the bikes in `bike/model/imported.rs`, the music in each circuit, the scenery's model packs in `world/japan.rs`, `middle_earth.rs` and `toys.rs`, the textures in `world/textures.rs`. Nothing goes through Bevy's asset server.

### What is open and what is private

- **Open (the engine's own demo content, so it runs on its own):** the four bikes built in code with their painted liveries (`Design::Afilada`, `Escudo`, `Alerones`, `Rombo`; `bike/model/livery.rs`), the Poly Haven textures (CC0) and the NASA maps (public domain), and a plain circuit (Bahía GP).
- **Private (the game):** the Meshy bikes and riders, the music, the modelled trees, props and toys (feudal Japan, Middle-earth, the toy store), and the circuits after other works (Fellowship Road, Ghost Island).

### Steps

1. **Circuits as data.** The layout and scenery types made serialisable and owned (`Vec`s rather than `&'static` slices, or parsed once and leaked to `&'static` so the code using them stays as it is); a `Circuits` resource filled at startup instead of `CIRCUITS`; Bahía GP first in RON, then the rest one by one, the Rust ones working meanwhile; a test that loads every circuit, so a mistake fails `cargo test` and not the game. The bikes' roster the same way. RON rather than JSON: it is Rust's own (the settings are in it already) and keeps comments in what is written by hand.
   - **Begun** (`RECORD/2026-10-30.circuits-in-ron.WIP.md`): Bahía GP, Turbo Bay, Loop Island and Base Lunar are `assets/circuits/bahia.ron`, `turbo_bay.ron`, `loop_island.ron` and `moon_base.ron`, read by `src/circuits/data.rs` (lists and texts leaked to `&'static`; the terrain's functions and the songs named and kept in Rust), `CIRCUITS` a `LazyLock` reading it the first time it is wanted, from the disk in a desktop development build; `every_circuit_file_reads` in `cargo test`; `tools/circuit_layout.py` reads it. Next: the scenery a file cannot give yet (the pit, the mine, the desert and the themed scenery), with the other circuits one by one, each with the types of its own scenery: Gold Mine (`Pit`, `Mine`, `Desert`), Ghost Island (`Japan`), Fellowship Road (`MiddleEarth`) and Juguetería (`ToyStore`); the `Circuits` resource and the bikes' roster.
2. **The terrain's functions as shapes with parameters**, circuit by circuit: ranges of `fbm` hills (a middle, an inner and an outer radius, a height, a scale), a cone with its crater and gullies, a coast as an outline with a ragged edge. Half of it is there already (`massif`, `climbed_massif`, the outlines as polygons). Named Rust functions only as a way out for what the game adds of its own, not edited. Notes as fields where a comment matters; values written out, or as anchors (a stretch from another's end), where they now come from sums.
3. **A content pack**: circuits, models, music and textures through a Bevy asset source; built into the release of the private game, so it stays a single executable (the web too); read from disk, and reloaded when it changes, in development and by the editor.
4. **A workspace** in this repository: an `engine` crate and a `game` crate. The themed code goes to `game`, as generic features where it can (a corridor of torii is a corridor of gates with a model) and otherwise through points the engine offers to hook into (as plugins adding their own scenery). A CI job runs the engine with the open content alone.
5. **The editor**, in the engine, inside the game's own world, so that what is edited is seen as it will be raced: the plan and the height profile; control points, heights, banking and stretches; what stands along the road. The limits `tools/circuit_layout.py` and the tests check (`curves_are_drivable`, `slopes_and_banking_stay_rideable`, the crests the bike takes off over) shown as it edits, ported to Rust and shared with the tests. Only the road and the ground near a change rebuilt (step 2's pieces).
6. **The engine published** from a new repository, with its licence (MIT or Apache-2.0, as Bevy) and its credits. Never by making this repository public: its history holds every model, song and circuit (a 99 MiB pack).

Steps 1 and 2 are worth it before anything is split: circuits tuned without compiling.

## Alongside: Juguetería's toys

Content, not engine: it runs beside the steps above, in a branch of its own.

Juguetería, the toy store (`RECORD/2026-10-28.toy-store-track.completed.md`), came in [germadev/motorust#28](https://github.com/germadev/motorust/pull/28): a compact lap of 2.92 km on a narrow road (`Layout::wall`, 7.5 m), a way through a castle and a way round a loop, its checkpoints at three quarters of the usual pace (`Circuit::pace`). Meshy's models are to replace the toys built in code and fill the floor, as the author makes them; the list (36 pieces, with their sizes, where each stands, how close it comes to the road and its triangle budget) was given to the author. Thirty-three are in (`RECORD/2026-10-28.toy-store-meshy-toys-and-banking.completed.md`), standing by the road: in the first pack both castles, the pirate ship, the trains' locomotive, coach and freight wagon, the teddy bear, the tin robot, the rubber duck, the spinning top, the biplane, the helicopter, the plush giraffe, the doll's house, the fire engine, a mechanical pegasus, a crocodile monster truck, the garage playset, the railway station, the tunnel mountain the floor train runs through, the rocking horse and the nutcracker; in the second the dump truck, the teepee, the T-rex whose head reaches over the home straight, the rocket, the toy chest, the jack-in-the-box, the xylophone, the giant robot at the shop's doors, and by them the checkout counter, shopping trolleys and potted plants. The shelves round the walls are in bays of departments, and the south wall has the shop's front of glass, a car park outside it in the daylight (`src/world/shopfront.rs`). The way round the loop goes straight on into it while the lap swings away, a Hot Wheels booster on each (`src/world/boosters.rs`), as quick either way. Still to come from Meshy, if wanted: a stack of baskets and a ball pit (in `SIZES` already), in a third atlas row. Built in code there are still the beach balls, the lettered blocks and the tyres, and the plastic building bricks (`src/world/bricks.rs`).

- **One at a time**, with `tools/import_toys.py --keep`: each into the pack at its size (`SIZES`), placed by `spawn_models` (the castles no road runs through, and the toys whose kind has a model, `model_of`), the code-built toy it replaces dropped. The test of the pack checks every piece in it: on the ground, centred, under 8,000 triangles; and that every toy the store places is in it.
- **The atlases:** the first pack's (`assets/models/toys/`) is 4096 pixels square and full; the second's (`toys2/`) is 4096 × 2048 and full (seven big cells and four small): the next toy grows it to 4096 × 3072 (and to sixteen at 4096 square). A full one takes about 85 MB of the graphics card's memory with its mipmaps, loaded at startup for every circuit: if that tells on the web or on small cards, load the toys' packs only for the toy store, or compress their textures for the card (KTX2, BC7 or ASTC).
- **`CELLS`** (the 1024-pixel cells) follow the pieces the riders pass closest: the castles, the ship, the giraffe, the doll's house, the bear, the robot, the fire engine, the pegasus, the garage, the station, the nutcracker, the planes, and to come the checkout counter; the small ones (the duck, the top, the trains, the croc car, the tunnel mountain, the rocking horse) do with 512.
- **Measured** (step 1) once most of them are in: about 200,000 triangles more in all, drawn whole (no levels of detail), one material.
