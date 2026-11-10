# Costa Bits: props to make with Meshy

These are the props to model with Meshy for [Costa Bits](tracks.md#costa-bits). It is a homage to the 8- and 16-bit arcade racers: a sunset coast road, palms, a whitewashed town, a mountain road over the sea.

None of these props carries a brand, a logo or a real product. Each is a generic, original design, so that nothing registered appears in the game (see [Trademarks](#trademarks)).

Sizes are in the game's metres, which are real metres: a bike is about 2 m long, and the road's walls stand 11.5 m from its middle. The size is what the importer scales each model to (its `SIZES`): the height, or the length for boats, cars and planes, as each row says.

## How the track is drawn, and what it asks of a model

Costa Bits is drawn as voxels (`look: Voxel(size: 1.0, shades: Some(12))`): the ground in blocks of one colour, 1 m on a side by the road and up to 8 m far off, cut to 12 shades of each of red, green and blue; the road in flat bands of colour; the trees and palms as flat cards of pixel art that turn to face the camera (the engine's `src/world/sprites.rs`). Every prop is made in Meshy as a 3D model either way, and is drawn in one of two ways, as the **Drawn as** column of each table says:

- **Model:** drawn in 3D, as the toy store's toys are. For what stands close to the road or has a clear front: signs, the billboard, the buildings by the road, the car. A card turns about its upright to face the camera, so a sign, a car or a long building would be seen to swing round as the bike passes it.
- **Card:** photographed by the engine into a flat sprite of pixel art, as the trees are, and planted facing the camera. For what is far off, round or plant-like: the watchtowers, the castle, the hermitage, the cactus, the agaves, the ibex. A card costs two triangles however many the model has, and casts no shadow.

What follows from this:

- **Flat blocks of colour.** The ground beside every prop is in flat blocks cut to 12 shades; a prop with a photographic or noisy texture looks pasted in. Ask for a few clear colours (white walls, terracotta, ochre, deep blue, red, yellow, green) in big, simple areas.
- **Chunky shapes.** A card is photographed at 4 pixels a metre: a 9 m tower is 36 pixels high, and anything under 25 cm is lost. Models can carry a little more, but silhouettes and blocks of colour carry the look. Slightly blocky, faceted shapes sit well beside the voxels; rounded, smooth ones do not.
- **The colours in the vertices, for cards.** The engine photographs a mesh by its vertices' colours, not by its texture. The importer bakes each card's texture into its vertices, so a texture of flat areas comes through, and fine painted detail does not.
- **Few triangles.** The counts below are the targets for Meshy's remesh. Keep to them for models: the many copies of a sign or a boulder are drawn every frame. For cards they matter less, but keep them low so the photograph is quick at load.
- **Ignore normal and metal-roughness maps.** As for the toy store's models, only the colour texture is used.

## Settings for every model

- **Mode:** Text to 3D. Export as GLB.
- **Topology:** triangles; **target polycount** as in the table.
- **Texture:** base colour only, with flat, hand-painted colours.
- **Pose:** one object, its front facing the viewer, standing upright on flat ground, centred.
- **Add to the end of every prompt:**
  > stylized low-poly game asset, chunky faceted shapes, large flat areas of bold saturated color, simple readable silhouette, toy-like proportions, single object, centered, standing on flat ground, no ground plane, no base, no text, no logos
- **Negative prompt** (where Meshy offers one):
  > text, letters, numbers, logo, brand, trademark, signage text, ground plane, base, pedestal, multiple objects, photorealistic, pixelated texture, noisy texture, gradients, fine detail, smooth glossy surfaces

Don't ask for "pixel art" or "16-bit" in the prompt: Meshy then paints pixels onto the texture, which turn to mush on a model and to speckle on a card. The pixel look comes from the engine.

Check each result before sending it. Meshy sometimes invents letters or logos on signs, cars and parasols. Where it does, regenerate the model or paint over that part.

## The props

**Priority** is the order to make them in: **A** for the ones the riders pass closest or see most, **B** for those that add variety, **C** for later or optional.

### The coast and the town (start straight, south coast)

| Name (`SIZES`) | What | Size | Triangles | Drawn as | Priority | Where |
|---|---|---|---|---|---|---|
| `white_house` | Andalusian whitewashed house, two floors | 9 m high | 2,000 | Model | A | Along the start straight, in place of some of the engine's cottages, a few copies turned differently |
| `chiringuito` | Beach bar with a reed roof | 4.5 m high | 2,500 | Model | A | On the bay's beach below the start straight |
| `espeto_boat` | Small wooden fishing boat on the sand, filled with sand, a fire with fish skewers | 5 m long | 1,500 | Model | A | On the beach, by the beach bar |
| `retro_sports_car` | 1980s wedge-shaped convertible, an original design | 4.4 m long | 3,000 | Model | A | Parked by the grid. Meshy made it red; it was repainted yellow, with two black stripes along its bonnet and boot, to keep it clear of any real car |
| `watchtower` | Round stone coastal watchtower, 16th century | 14 m high | 1,500 | Card | A | On the cape, by the lighthouse; another on the headland |
| `lifeguard_tower` | Wooden lifeguard tower, red and white | 5 m high | 1,200 | Model | B | On the bay's beach |
| `flag_marshal` | Race marshal in white overalls, waving a chequered flag | 2.2 m high | 2,500 | Model | B | By the finish line, beside the road. Meshy's chequers came out smeared; they were painted again, square. Static, behind the wall (13–14 m from the road's middle), on a low platform if the wall hides his legs; his arm may be cut off in the importer and swung about his shoulder later, if wanted |
| `banner_plane` | Vintage propeller plane towing a blank banner | 22 m long with the banner | 2,500 | Model | C | Over the bay, flying round in a circle 40 m up, banked into its turn (`planes` in the circuit's file) |

### The mountain road (Carretera de la sierra)

| Name (`SIZES`) | What | Size | Triangles | Drawn as | Priority | Where |
|---|---|---|---|---|---|---|
| `curve_sign`, `curve_sign_left` | Yellow diamond warning sign of a bend, on a post, bending right and left | 3 m high | 22 | Model | A | Before the bends of the climb and the way down, bending as the road does; many copies. **Not from Meshy:** its arrow came out a smudge, so both are built by `tools/make_curve_sign.py` |
| `boulder_a`, `boulder_b`, `boulder_c` | Big sandstone boulders, three shapes | 3 to 5 m high | 400 each | Model | A | Along the mountain road and the headland; many copies |
| `mirador` | Stone viewpoint with a wooden pergola and a bench | 4 m high, 6 m wide | 2,000 | Model | A | On a bend of the climb, about 2,300 m, over the sea |
| `km_post` | White kilometre post with a red top | 1.2 m high (shown at 1.5 m) | 150 | Model | B | Every 500 m along the lap; many copies |
| `ermita` | Small whitewashed hermitage with a bell gable | 10 m high | 2,000 | Card | B | On the headland, by the hamlet, away from the road |
| `prickly_pear` | Clump of prickly pear cactus | 2.5 m high | 1,000 | Card | B | On the dry slopes of the mountain and the cape |
| `agave` | Agave plant with a tall flower stalk | 4 m high with the stalk | 800 | Card | B | Same, mixed with the cactus |
| `ibex` | Iberian ibex standing on a rock | 1.2 m high (shown at 1.5 m) | 1,200 | Card | C | On the rocks above the cuttings |
| `camper_van` | 1970s camper van, an original design | 4.5 m long | 2,500 | Model | C | Parked at the viewpoint |
| `castle_ruin` | Ruined Moorish hilltop fortress, walls and towers | 18 m high, 30 m wide | 4,000 | Card | C | On the mesa, or on the mountain over the tunnel, seen from afar |

### The gorge

| Name (`SIZES`) | What | Size | Triangles | Drawn as | Priority | Where |
|---|---|---|---|---|---|---|
| `gorge_bridge` | An old stone bridge over a gorge, one great arch | 48 m high, 55 m wide | 3,700 | Model | A | Over the road in the gorge of the mountain road, at 2215 m. Meshy made its arch narrow and closed at its foot; the importer opens it to the ground and widens it to 7.4 m either side of the middle, where the walls close in to 6.5 m |

### Homage to the arcades (no brands)

| Name (`SIZES`) | What | Size | Triangles | Drawn as | Priority | Where |
|---|---|---|---|---|---|---|
| `arcade_billboard` | Retro roadside billboard on two legs, framed with light bulbs, painted with a sunset and palms | 7 m high, 10 m wide | 1,000 | Model | B | On the start straight and the coast highway |

Dropped since the list was first written: `palm_retro`. The palms are already flat cards photographed from the engine's own, and read well.

## The prompts

Each prompt below is followed by the shared ending (see [Settings for every model](#settings-for-every-model)).

- **`white_house`:** A two-storey Andalusian whitewashed village house, a simple box shape, flat white walls, a terracotta tiled gable roof, one blue wooden door, two square windows with blue shutters, a black iron balcony with red flower pots, a short chimney.
- **`chiringuito`:** A small Spanish beach bar kiosk of wood, a roof thatched with dry reeds, an open counter at the front, three bar stools, two blank white folded parasols beside it, blue and white planks, no signs.
- **`espeto_boat`:** A small old wooden fishing rowboat lying on beach sand, painted white and blue with a red stripe, its hull filled with sand, a small wood fire in the middle with a row of fish on skewers leaning over it.
- **`retro_sports_car`:** An original 1980s-style red two-seater convertible sports car, low wedge-shaped angular body, pop-up headlights closed, a wide black rear light strip, black interior, simple five-spoke wheels, generic design not based on any real car, no badges.
- **`watchtower`:** A 16th-century round coastal watchtower of sandstone, cylindrical, slightly wider at the base, a small dark arched doorway high up the wall, a flat top with a low parapet, warm ochre stone, a few large stone blocks.
- **`lifeguard_tower`:** A wooden beach lifeguard tower on four legs with a ladder, a small box cabin with a sloped roof, painted in wide red and white stripes, a blank flag on a pole.
- **`flag_marshal`:** A race marshal standing, white overalls, a white cap, holding up a black-and-white chequered flag in the right hand mid-wave, chunky stylized proportions.
- **`banner_plane`:** A small vintage single-engine propeller plane, high wings, painted yellow and red, towing a long blank white fabric banner on a rope behind it, side view in level flight.
- **`boulder_a`, `boulder_b`, `boulder_c`:** A large angular sandstone boulder with big flat facets, warm ochre and tan with a few darker bands; `b` taller and narrower, `c` flatter and split in two.
- **`mirador`:** A small stone scenic viewpoint over a cliff, a low semicircular parapet of rough stone, a wooden pergola of square beams on four stone pillars above it, a wooden bench under it.
- **`km_post`:** A small roadside kilometre marker post of white concrete, rectangular with a rounded top painted red, a blank face.
- **`ermita`:** A small whitewashed Andalusian hermitage chapel, flat white walls, a terracotta tiled roof, a bell gable with one bronze bell above the arched brown wooden door, a small round window.
- **`prickly_pear`:** A clump of prickly pear cactus, big flat oval green pads growing on each other, a few red fruits on the edges.
- **`agave`:** An agave plant, a rosette of thick spiky blue-green leaves, one tall brown flower stalk rising from it with yellow flower clusters at the top.
- **`ibex`:** An Iberian ibex, a wild mountain goat with long curved backswept horns, standing proudly on a rock, side view, brown and tan coat.
- **`camper_van`:** An original 1970s-style camper van, a rounded boxy body, two-tone cream and turquoise paint, a pop-up roof, round headlights, generic design not based on any real van, no badges.
- **`castle_ruin`:** A ruined Moorish hilltop fortress, crenellated walls of ochre stone on rocky ground, three square towers, one half collapsed, an arched horseshoe gateway, seen from the front.
- **`gorge_bridge`:** A tall old stone bridge of ochre sandstone blocks, one very tall wide central arch reaching down to the ground, its opening half as wide as the bridge, a smaller arch above it, massive solid piers on both sides, a stone parapet along the top, a small arched window above the central arch, front view. Negative prompt, besides the shared one: cliffs, gorge walls, rocks, river, water, ground, town, buildings.
- **`arcade_billboard`:** A retro roadside billboard on two steel legs, a wide rectangular panel framed with a row of round light bulbs, the panel painted with a simple orange-and-pink striped sunset over a blue sea with black palm tree silhouettes, no text.

For the cards, the side Meshy shows at the front is the one the engine photographs: the ibex side on, the castle and the hermitage from the front.

## Trademarks

Neither the props nor anything in the game names or copies a registered mark. In particular:

- **No game names in the game:** neither the names of the arcade racers this track pays homage to nor their makers. The prompts above don't use them either: the style is described instead (sunset, palms, coast road, bold colours).
- **No real vehicles:** the car and the van are original designs. Look at what Meshy makes: if it comes out recognisable as a real model (its grille, its lights, its proportions), make it again. The van Meshy made is close to a well-known bus of the 1970s (its rounded front, two tones and pop-up roof); told so, the author chose to keep it.
- **Not the black bull silhouette of the Spanish roadsides:** it is a registered mark, so no bulls on hills.
- **No real brands:** no drinks, beers or ice creams on the beach bar, the parasols or the billboards; no fuel brands on signs.
- **No real arcade cabinets or console logos.**

## Sending them

Send the GLBs named as in the tables (`white_house.glb`, `watchtower.glb`, …), all at once or a few at a time. `tools/import_costa.py` makes a pack of them in `assets/models/costa/`, and the theme `costa` (`src/game/costa.rs`) places them where the circuit's file says (its `themes`, `"costa"`). Keep the GLBs: the pack is made again from all of them whenever one is added or changed.

Drawn as models so far: every prop, those marked **Card** too. Drawing those marked **Card** as flat sprites needs the engine to offer its sprites (`src/world/sprites.rs`) to the game's themes, and the importer to bake their textures into their vertices; left for later. The banner plane flies round over the bay, 40 m up (`planes`, moved by the theme).
