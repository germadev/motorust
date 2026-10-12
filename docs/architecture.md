# Architecture

How the code is organised and why. To build and run it see [building.md](building.md); the tracks and the bikes have their own pages: [tracks.md](tracks.md) and [models.md](models.md).

## Overview

motorust is a [Bevy](https://bevy.org) 0.19 app built from one plugin per area. `src/main.rs` loads the saved settings (they decide how the window opens), adds Bevy's `DefaultPlugins` without its gilrs plugin (gamepads are read by `src/gamepads.rs`), fixes the simulation rate and adds the game's plugins: world, bike, chase camera, gamepads, HUD, menu, minimap, welcome screen, track selection, pause, settings, sound and UI, plus the web plugin when compiled for `wasm32`.

- **Screens.** `GameState` (`src/menu.rs`) goes Welcome → Tracks (choose a track) → Select (choose a bike and rider) → Race. Changing track rebuilds the world; moving between screens slides the camera instead of cutting.
- **Fixed-step physics, interpolated rendering.** Physics runs in `FixedUpdate` at **120 Hz**: `ai::drive_rivals` → `simulate_bikes` → `separate_bikes`, chained. Before the fixed loop the player's input is read; after it, `interpolate_bikes` places each bike between its previous and current state for the frame being drawn.
- **Pure, tested logic.** The parts that decide how the game behaves are plain functions with no window, renderer or ECS dependency, so `cargo test` exercises them headless: `bike::step` (one physics step), `bike::hit_walls` (walls), `bike::crash` (contacts between bikes, falls and rejoining the track), `bike::chassis` (wheelie and suspension springs), `bike::loop_ride` (riding round a loop), `Track` (the circuit and its queries), `ai::decide` (a rival's controls) and `pilot::pose` (the rider's race pose from its skeleton).
- **Units and axes.** Everything is in metres. The ground plane is Bevy's XZ plane with Y up; ground positions are `Vec2(x, z)` and a heading of 0 faces -Z, with positive values turning left (`src/geometry.rs`). Bike-local space is X right, Y up, -Z forward, with the origin on the ground between the wheels.
- **No external files at runtime.** Models, textures and the font are embedded in the executable, and all sound is synthesised in real time.

## Code map

| Path | What it does |
|---|---|
| `src/main.rs` | Builds the app, registers the plugins and sets the simulation rate (120 Hz). |
| `src/geometry.rs` | Ground-plane conventions (`Vec2(x, z)`, heading) and shared numeric helpers (noise, seeded randomness, smoothing). |
| `src/track.rs` | The circuit layout: centreline (a closed Catmull-Rom curve through control points), elevation, banking and queries (nearest point, height, slope, grid slots). Where the track passes over itself, the queries for something on it (`nearest_at`, `road_height_at`) take its height and answer for the stretch at its level. Also the branches: open roads off the lap (`side_roads`, numbered in `Nearest::road`) that every query also looks at, lying on the lap's surface while they share its road, and the vertical loops between them (`Loop`). Knows nothing about rendering. |
| `src/circuits/` | The tracks: `bay.rs` (Bahía, "the Bay"), `turbo_bay.rs` (Turbo Bay) and `loop_island.rs` (Loop Island), each with its layout and scenery, and `mod.rs`, which lists them and tracks the one in use. See [tracks.md](tracks.md). |
| `src/world/mod.rs` | The world around the road for the chosen track, under a summer sky; rebuilt when the track changes. |
| `src/world/scenery.rs` | The data that describes a track's scenery: coast, hills, crags and rock walls, what lines the road (walls, tunnels, bridges, arches, signs) and the props. |
| `src/world/terrain.rs` | Terrain with sea, beaches, rock (or checkerboard earth), hills, inlets and flat areas, computed once on a grid that is fine near the road and coarser towards the horizon (multi-threaded on the desktop). Also the sea: a grid out to the horizon, tinted in broad patches and with its texture laid askew and warped so it does not show a repeating pattern. |
| `src/world/road.rs` | Asphalt, lines, kerbs in the corners, walls (stone, concrete, railings, hedges or checkerboard earth) and the arches over the road, following the track's height and banking, for the lap and the roads off it; where two roads meet, neither draws walls, lines or kerbs on the other. |
| `src/world/structures.rs` | Tunnels cut through the rock (whose rock carries the road where the track passes over a tunnel), bridges over the water (red arches or wooden) and billboards. |
| `src/world/props.rs` | Street lamps, trees and forests, palms, houses, tall hotels, the grand old hotel and buoys, placed pseudo-randomly but always the same way. |
| `src/world/landmarks.rs` | Loop Island landmarks: the loops the bikes ride (asphalt with kerbs on a ring of checkerboard earth), giant loops to look at on blocks of checkerboard earth, totem poles, sunflowers, spinning golden rings (no track has them now) and waterfalls with flowing water. |
| `src/world/lettering.rs` | Chunky pixel lettering, like 90s games, for the finish arches, banners and signs. |
| `src/world/quads.rs` | Mesh builder made of flat quads and triangles that orient themselves the way they are asked to. |
| `src/world/sky.rs` | Sky dome with a summer-sky photo, always centred on the camera so it looks infinitely far away. |
| `src/world/textures.rs` | Scenery textures (CC0 photos from Poly Haven), embedded, plus procedural ones (checkerboards, water, crowds, checkerboard earth, waterfalls, planks), with mipmaps computed at startup. |
| `src/bike/mod.rs` | The bike: controls, arcade physics (`step`), collisions with walls and other bikes, interpolation, the starting grid and the `ROSTER` of bikes. |
| `src/bike/chassis.rs` | Wheelies and suspension: damped springs that advance with the fixed-step physics. |
| `src/bike/crash.rs` | Contacts between bikes as impulses between two rigid bodies (momentum, restitution, friction, spin), the knock the tyres scrub off afterwards, when a rider goes down, the slide of the fallen bike and the thrown rider, and where the bike rejoins the track. |
| `src/bike/loop_ride.rs` | Riding round a vertical loop on rails: going in at the mouth, the engine and gravity along the running surface, coming out of the way out, rolling back down when too slow, and falling off when the surface no longer holds the bike. |
| `src/bike/ai.rs` | Rival riders: they follow their lane, move out to overtake the bike they catch (or lift off if there is no room), brake before corners and accelerate out of them, using the same `BikeInput` and physics as the player. Some take the branches and ride their loops. |
| `src/bike/model/mod.rs` | 3D bike and rider models (wheels, fork, livery, race number) and their animation: wheels, steering, suspension, brake light and rider pose. |
| `src/bike/model/designs.rs` | The bike designs (`Design`): the four code-built bodies (Afilada, Escudo, Alerones, Rombo) with measurements taken from the design sheets and their vents, grilles and pinstripes, plus the eight imported ones. |
| `src/bike/model/imported.rs` | Loads the Meshy bikes prepared by `tools/import_models.py`, with their riders, textures, exhaust outlets and measurements (`Model`). |
| `src/bike/model/pilot.rs` | The Meshy riders: their skeleton and the race pose, solved with inverse kinematics every frame. |
| `src/bike/model/rider.rs` | The code-built rider: a skinned body with its bones, plus helmet and other rigid parts, and its pose. |
| `src/bike/model/livery.rs` | Liveries for the code-built bikes (colours, fairing graphics, suit and helmet), painted into textures at startup. |
| `src/shapes.rs` | Procedural meshes: smooth and faceted lofts, tubes, prisms and skinned sweeps. |
| `src/camera.rs` | Chase camera with fog and a field of view that widens with speed, right-stick look-around, turning over with the bike round a loop, and the camera slide between menu screens. |
| `src/hud.rs` | Speedometer, controls legend, frames per second and the font used for labels. |
| `src/minimap.rs` | Track map drawn from the centreline and the branches, used on the track selection screen and as the race minimap (one dot per bike in its colour). |
| `src/menu.rs` | The game screens (`GameState`) and the bike and rider selection screen. |
| `src/welcome.rs` | Welcome screen, with the options for resolution, FPS limit and controls. |
| `src/track_select.rs` | Track selection, with the map drawn from the layout. |
| `src/settings.rs` | Settings kept between sessions (resolution, FPS limit, keys and buttons for each action), the fullscreen shortcut and the frame limiter. Saved to `ajustes.ron` on the desktop and to `localStorage` on the web. |
| `src/ui.rs` | Shared menu code: navigation with keyboard or gamepad, the options panel and the gamepad status notice. |
| `src/pause.rs` | Pause menu and the "Depuración" (Debug) options. |
| `src/gamepads.rs` | Gamepad input with gilrs, connected to Bevy's input, retrying the search while no gamepad is found and logging what each one sends. |
| `src/gamepads/apple.rs` | On macOS, the gamepads the system knows (DualSense, DualShock 4, Xbox, Switch Pro, MFi) are read with Apple's GameController framework instead of gilrs. |
| `src/sound/mod.rs` | Decides what sounds, how loud and where (engines with their gearboxes, wind, tyres, grass, impacts, landings, menu), panned from the camera and with Doppler for rivals. |
| `src/sound/synth.rs` | The synthesiser that generates and mixes everything on the audio thread, including the big-bang V4 engine. |
| `src/web.rs` | Web only: tells the page the game is visible so it can remove the loading screen. |
| `web/index.html` | The web page: canvas, loading screen with download progress, audio unlock on first input, error display. |
| `web/build.sh` | Builds the web version into `dist/web` (see [building.md](building.md#web-version)). |
| `tools/import_models.py` | Prepares the Meshy GLB files for the game (see [models.md](models.md#the-importer)). |
| `assets/` | Embedded resources: `models/` (one folder per bike), `textures/`, `fonts/` and `CREDITS.md`. |
| `.github/workflows/` | CI, web publishing and releases (see [building.md](building.md#continuous-integration)). |

## Design decisions

- **Fixed-step simulation with interpolation.** Physics always advances in steps of 1/120 s (`FixedUpdate`) and rendering places the bike between the previous step and the current one. The bike handles the same at any frame rate, and the simulation is reproducible, which will later allow replays, ghosts and multiplayer.
- **Loops on rails.** The bike is a body on the ground plane with a height, so it cannot go upside down by itself. Round a loop it switches to a one-dimensional ride along the running surface (`BikeState::ride`): its position, height and pitch come from how far round it is, and only its speed is simulated, with the engine, the brakes, the resistances and gravity along the surface. That keeps the loop exact and cheap, and the rest of the physics untouched.
- **Arcade physics, not realistic.** The bike moves as a single body with a speed and a heading; it does not simulate balancing on two wheels. Turning is limited by a minimum radius at low speed and by lateral grip at high speed. Lean is purely visual and follows how much of the available grip the corner uses: little in gentle bends, full lean only at the limit (the angle that would balance the corner would lay the bike down with very little steering, given the arcade grip). The parameters live in `BikeTuning`.
- **Wheelies and suspension as springs.** The wheelie, the suspension pitch and its compression are three damped springs that advance with the fixed-step physics (`bike::chassis`). Pulling back, accelerating, braking or landing only changes where each spring pulls or gives it a kick, and the spring does the rest: it rises slowly, drops quickly, bounces and settles. The acceleration that moves the suspension, the rider and the camera is the one the rider would feel, without the slope's: rolling downhill off the throttle does not push you back.
- **Pure logic separated from the ECS.** `bike::step`, the collision functions, `Track`, `ai::decide` and the rider pose do not depend on windows or rendering, so they have tests (`cargo test`) and are shared: the rivals drive with exactly the same physics as the player.
- **Shared model blueprint.** A bike is a list of parts (`BikeBlueprint`), each with a mesh, a paint and the point of the bike it is attached to (chassis, steering, front or rear wheel). Meshes are created once and every bike reuses them, changing only its body and its livery, so adding rivals is almost free.
  - The code-built bodies are made of lofts: cross-sections along each part. The bodywork uses polygonal sections joined by flat faces with flat shading (`shapes::faceted`), the low-poly style of the design sheets; sections can be slanted to bevel the fairings. Each `Design` comes from measuring the side view of its sheet, scaled to the wheelbase.
  - The imported Meshy bikes plug into the same mounts: the importer splits each one into chassis, steering and wheels (see [models.md](models.md)).
- **Skinned riders with inverse kinematics.** The code-built rider's body is a single mesh deformed by a ten-bone skeleton (pelvis, chest, upper arms, forearms, thighs and shins), so shoulders, elbows, waist and knees bend without seams. The Meshy riders use their own 23-bone auto-rigged skeleton. In both, arms and legs use two-bone inverse kinematics: the hands never let go of the grips while the torso moves, and the feet stay on the pegs while the rider hangs off and puts the knee down.
- **Decoupled input.** The bike reads a `BikeInput` and does not care where it comes from: the keyboard or a gamepad fill it for the player, and `ai::decide` fills it for each rival.
- **Self-contained executable.** Models, textures and the font are embedded with `include_bytes!`, liveries are painted into textures at startup and sound is synthesised, so a single file runs the game, on the desktop and on the web alike.

## Tests

There are **213 tests**, all headless. Run them with:

```sh
cargo test   # or: make test
```

CI runs them on every push and PR, together with `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (see [building.md](building.md#buildyml)).

| Area | Tests | What they check (examples) |
|---|---|---|
| Bike physics (`src/bike/mod.rs`) | 49 | Top speed, braking and reversing, steering build-up and release at any frame rate, lean, grass, walls, jumps, wheelies, flat-out laps without touching the walls, the starting lineup, gamepad triggers that do not quite return to zero. |
| Crashes and grass (`src/bike/crash.rs`, `src/bike/grass_kickup.rs`) | 25 | Bumps, rubs and t-bones between bikes, wall hits, falls (also off a loop, down to the road below), slides and loose limbs, rejoining the track, grass kicked up with speed. |
| Chassis (`src/bike/chassis.rs`) | 9 | Wheelies, squat, dive, landing compression, suspension in the air. |
| Rivals (`src/bike/ai.rs`) | 5 | A rival completes a lap without leaving the track, goes round a stopped bike, takes short jumps over the crests and, if it takes the branches, rides round the loop and back; a pack races a lap with hardly any falls. |
| Loops (`src/bike/loop_ride.rs`) | 5 | Going round and out beside the mouth, coming back down too slow, falling off, going in at the mouth. |
| Bike and rider models (`src/bike/model/`) | 27 | Imported parts and wheels on the ground, open exhausts, rigged riders, hands on the grips and feet on the pegs on every bike, liveries, race numbers. |
| Track (`src/track.rs`) | 15 | Drivable curves, rideable slopes and banking, walls of distant stretches that do not meet, the grid, branches that leave and join the lap along it and on its surface, and bikes on a branch finding its road. |
| World and tracks (`src/world/`, `src/circuits/`) | 28 | Terrain hugs the road and keeps it dry, sea and mountains where expected, kerbs only in corners, textures decode, houses fit, loops clear of the road, waterfalls down rock walls, wooden bridges fenced, checkpoints, Turbo Bay passing over its own tunnel. |
| Meshes (`src/shapes.rs`) | 9 | Faces point outwards, sections and bone weights are consistent. |
| Sound (`src/sound/`) | 9 | Gear changes, throttle loudness, panning, one-shot sounds. |
| Other (geometry, settings, camera, minimap, menus, HUD) | 32 | Heading conventions, settings survive a save and load, one action per key, map projection, menu items. |

The track tests run on every circuit: when you change a track, they tell you if it has impossible corners, excessive slopes, overlaps itself, or if the terrain covers the road or leaves it in the air (see [tracks.md](tracks.md)).

## Next steps

- Lap timer and lap counter.
- Rivals that defend their position.
- Dust and smoke when accelerating, braking and landing.
- More tracks on the track selection screen.
- On-screen corner signs (green, yellow and red diamonds), as in Moto Racer.
- Motocross circuits with big jumps.
