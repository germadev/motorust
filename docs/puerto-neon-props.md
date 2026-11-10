# Puerto Neón: props to make with Meshy

These are the props to model with Meshy for Puerto Neón, the anti-gravity circuit: a futuristic harbour city at night, the track on pylons over the quay, between the towers, up to a high skyway and out over the sea. It is a homage to the anti-gravity racers as a genre; every prop is a generic, original design.

None of these props carries a brand, a logo, a team's emblem or lettering of any kind, real or from another game. Screens and signs are blank or abstract (bands and blocks of light): the game paints its own texts.

Sizes are in the game's metres, which are real metres: a hover bike is about 2 m long, and the track's barriers stand 11.5 m from its middle. The size is what the importer scales each model to: the height, or the length where the row says so.

## Settings for every model

- **Mode:** Text to 3D. Export as GLB.
- **Topology:** triangles; **target polycount** as in the table (Meshy's remesh).
- **Texture:** base colour (the game ignores the normal and metal-roughness maps of props).
- **Pose:** one object, upright on flat ground, centred, its front facing the viewer.
- **Add to the end of every prompt:**
  > stylized low-poly game asset, clean hard-surface sci-fi design, dark gunmetal and white panels with cyan, magenta and amber light strips, night-time neon look, simple readable silhouette, single object, centered, no ground plane, no base, no text, no letters, no logos
- **Negative prompt** (where Meshy offers one):
  > text, letters, numbers, logo, brand, emblem, trademark, signage text, ground plane, base, pedestal, multiple objects, people, photorealistic, noisy texture

Check each result before sending it: Meshy sometimes invents letters or logos on screens, hulls and signs. Where it does, regenerate it.

## The props

**Priority:** **A** what the riders pass closest or see most, **B** what adds variety, **C** later or optional.

| Name | What | Size in the game | Triangles | Priority | Where |
|---|---|---|---|---|---|
| `start_gate` | Start and finish gantry over the track | 26 m between its legs' inner faces, 10 m clear under the beam | 3,000 | A | Over the finish line |
| `tower_spire` | Slim skyscraper tapering to a needle | 200 m high | 4,000 | A | Downtown, several copies |
| `tower_stepped` | Stepped skyscraper with landing pads | 150 m high | 4,000 | A | Downtown, several copies |
| `tower_twin` | Twin towers joined by a skybridge | 120 m high | 4,000 | A | Downtown, several copies |
| `screen_mast` | Giant blank video screen on a mast | 14 m high, screen 12 m wide | 1,000 | A | Beside the track, many copies |
| `grandstand` | Covered grandstand with a curved roof | 12 m high, 40 m long | 4,000 | A | Start straight |
| `harbour_crane` | Ship-to-shore container gantry crane | 60 m high | 3,500 | A | The container port, in a row |
| `control_tower` | Race control tower: a shaft under a glazed saucer | 45 m high | 3,000 | B | By the start |
| `airship` | Airship with blank screens on its sides | 60 m long | 2,500 | B | Floating over the bay |
| `container_ship` | Container ship, loaded | 180 m long | 5,000 | B | Moored under the stretch over the sea |
| `maglev_train` | Maglev train of three cars | 45 m long | 3,000 | B | On a viaduct beside the track (moved in code) |
| `camera_drone` | Hovering TV camera drone | 2 m wide | 800 | B | Hovering beside the track, many copies |
| `fusion_plant` | Power plant with a glowing core | 35 m high | 4,000 | B | The industrial quay |
| `hover_car_a`, `hover_car_b` | Flying cars, two designs | 5 m long | 1,500 each | C | Lanes of traffic in the sky (moved in code) |
| `fuel_tanks` | Cluster of three spherical storage tanks | 18 m high | 2,000 | C | The industrial quay |
| `radar_dish` | Parabolic antenna on a turret | 25 m high | 2,000 | C | On the hill over the tunnel |
| `tunnel_mouth` | Tunnel portal ringed with light | 26 m between its inner faces, 9 m clear | 2,500 | C | The mouths of the tunnel |

## The prompts

Each prompt below is followed by the shared ending (see [Settings for every model](#settings-for-every-model)).

- **`start_gate`:** A futuristic race start gantry spanning a wide track: two angular pylon legs leaning inwards, joined by a deep horizontal beam, a row of five round start lights under the beam, blank glowing light panels along its front, cyan light strips up the legs, wide open passage underneath.
- **`tower_spire`:** A very tall slim futuristic skyscraper tapering to a needle antenna, faceted glass sides in dark blue with rows of lit windows, vertical cyan light strips at its edges, a wider base of three floors.
- **`tower_stepped`:** A tall futuristic skyscraper built of stacked blocks stepping back as it rises, a round landing pad cantilevered from two of the steps, dark panels with grids of warm lit windows, magenta light bands at each step.
- **`tower_twin`:** Two futuristic towers of different heights joined near the top by an enclosed glass skybridge, dark glass facades with lit windows, amber light strips along the skybridge, antennas on the taller tower.
- **`screen_mast`:** A giant rectangular outdoor video screen mounted on a single sturdy metal mast with a triangular brace, the screen completely blank and softly glowing pale cyan, a thin dark frame, small floodlights on top.
- **`grandstand`:** A long futuristic covered grandstand, tiers of empty seats rising to the back, a sweeping curved cantilever roof on slanted supports, white structure, light strips under the roof edge, glass railing at the front.
- **`harbour_crane`:** A huge ship-to-shore container gantry crane, four tall legs on wheeled bogies, a long horizontal boom reaching out over the water, an operator cabin and trolley under the boom, painted amber with dark machinery, warning lights on top.
- **`control_tower`:** A race control tower, a slender white concrete shaft carrying a wide glazed saucer-shaped observation deck with slanted windows lit from inside, antennas and a small dish on its roof, a ring of cyan light under the saucer.
- **`airship`:** A sleek futuristic airship, a long streamlined rigid hull in white and gunmetal, four tail fins, a small gondola under it, two engine pods with glowing rings, a large blank softly glowing screen panel on each side of the hull.
- **`container_ship`:** A large modern container ship seen whole, dark blue hull with a red waterline, stacks of plain coloured containers along its deck in red, blue, green and orange, a white bridge superstructure at the stern with a funnel, no markings.
- **`maglev_train`:** A futuristic maglev train of three cars, a long pointed aerodynamic nose at each end, smooth white body with a continuous dark window band and a cyan stripe, no wheels, a flat underside skirt.
- **`camera_drone`:** A small hovering broadcast camera drone, a compact dark body with four ducted fans in a ring around it, a large camera lens on a gimbal below, two small glowing status lights.
- **`fusion_plant`:** A futuristic fusion power plant, a large ring-shaped reactor building around a glowing cyan core seen through tall slits, radial cooling fins, thick pipes running to two squat cylinders beside it, warning lights.
- **`hover_car_a`, `hover_car_b`:** A futuristic flying car with no wheels, a low sleek cabin with a dark canopy, four glowing hover pads underneath, generic original design, no badges; `a` a white coupe, `b` a longer amber taxi-like saloon.
- **`fuel_tanks`:** A cluster of three large spherical white storage tanks on short steel legs, joined by pipes and a metal walkway with stairs, a small valve station beside them.
- **`radar_dish`:** A large parabolic antenna dish on a rotating turret base, lattice struts holding the feed horn at its focus, white dish with a dark back, tilted up at forty-five degrees.
- **`tunnel_mouth`:** A futuristic tunnel portal for a wide track: a thick angular arch of dark concrete and metal panels, a continuous ring of bright cyan light around the opening, sloped wing walls either side, wide open passage through it.

## The hover bikes

Three are in (the Shogun TZ's, the Azura RZ's and the Falcon KRZ's); the other five (Legend 88, Kraken RS, Stiletto V4, Cobra XT, Ronin YS) ride one of those three until theirs are made. Ask for each as those three were: the same pose and proportions (about 1.9 long in Meshy's units, nose to -X), about 15,000 triangles, base colour and metal-roughness, no rider, hover pads where the wheels would be, in that bike's colours.
