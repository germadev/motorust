# Costa del Sol: props to make with Meshy

These are the props to model with Meshy for [Costa del Sol](tracks.md#costa-del-sol). It is a homage to the 8- and 16-bit arcade racers: a sunset coast road, palms, a whitewashed town, a mountain road over the sea.

> **On hold.** This list was written for a world drawn as pixel art, which has been dropped. Its look may become flat sprites that turn to face the player, combined with voxels; the props, their sizes and their prompts will be revised for it before any is made.

None of these props carries a brand, a logo or a real product. Each is a generic, original design, so that nothing registered appears in the game (see [Trademarks](#trademarks)).

Sizes are in the game's metres, which are real metres: a bike is about 2 m long, and the road's walls stand 11.5 m from its middle. The size is what the importer scales each model to (its `SIZES`): the height, or the length for boats, cars and planes, as each row says.

## How the pixel art changed what a model needs (no longer applies)

- **Big shapes, bold colours.** The world is drawn at 224 lines, so a 2 m prop 40 m from the road is only a few pixels high. Fine detail (bricks, leaves, small text) is lost; silhouettes and blocks of colour carry. Props are sized a little larger than life (about 1.2×) where noted.
- **Few, saturated colours.** Every channel is cut to 16 shades with dithering: subtle gradients turn into dither patterns, and noisy textures into speckle. Flat, clearly separated colours read best: white walls, terracotta, deep blue, red, yellow, green.
- **Few triangles.** Detail below a pixel is wasted. The counts below are the targets for Meshy's remesh. Keep to them: the many copies of a sign or a boulder are drawn every frame.
- **Ignore normal and metal-roughness maps.** As for the toy store's models, only the colour texture is used.

## Settings for every model

- **Mode:** Text to 3D. Export as GLB.
- **Topology:** triangles; **target polycount** as in the table.
- **Texture:** base colour only, with flat, hand-painted colours.
- **Pose:** one object, its front facing the viewer, standing upright on flat ground, centred.
- **Add to the end of every prompt:**
  > stylized low-poly game asset, bold flat saturated colors, simple readable silhouette, retro 16-bit arcade racing game style, single object, centered, standing on flat ground, no ground plane, no base, no text, no logos
- **Negative prompt** (where Meshy offers one):
  > text, letters, numbers, logo, brand, trademark, signage text, ground plane, base, pedestal, multiple objects, photorealistic, noisy texture, fine detail

Check each result before sending it. Meshy sometimes invents letters or logos on signs, cars and parasols. Where it does, regenerate the model or paint over that part.

## The props

**Priority** is the order to make them in: **A** for the ones the riders pass closest or see most, **B** for those that add variety, **C** for later or optional.

### The coast and the town (start straight, south coast)

| Name (`SIZES`) | What | Size | Triangles | Priority | Where |
|---|---|---|---|---|---|
| `white_house` | Andalusian whitewashed house, two floors | 9 m high | 2,500 | A | Along the start straight, beside the engine's own houses, a few copies turned differently |
| `watchtower` | Round stone coastal watchtower, 16th century | 14 m high | 2,500 | A | On the cape, by the lighthouse; another on the headland |
| `chiringuito` | Beach bar with a reed roof | 4.5 m high | 3,000 | A | On the bay's beach below the start straight |
| `espeto_boat` | Small wooden fishing boat on the sand, filled with sand, a fire with fish skewers | 5 m long | 2,000 | A | On the beach, by the beach bar |
| `retro_sports_car` | 1980s wedge-shaped red convertible, an original design | 4.4 m long | 4,000 | A | Parked by the grid, and at the viewpoint |
| `lifeguard_tower` | Wooden lifeguard tower, red and white | 5 m high | 1,500 | B | On the bay's beach |
| `flag_marshal` | Race marshal in white overalls, waving a chequered flag | 2.2 m high | 3,000 | B | By the finish line, beside the road |
| `banner_plane` | Vintage propeller plane towing a blank banner | 22 m long with the banner | 3,000 | C | Over the bay (moving it needs code) |

### The mountain road (Carretera de la sierra)

| Name (`SIZES`) | What | Size | Triangles | Priority | Where |
|---|---|---|---|---|---|
| `curve_sign` | Yellow diamond warning sign of a bend, on a post | 3 m high | 300 | A | Before the bends of the climb and the way down; many copies |
| `boulder_a`, `boulder_b`, `boulder_c` | Big sandstone boulders, three shapes | 3 to 5 m high | 800 each | A | Along the mountain road and the headland; many copies |
| `mirador` | Stone viewpoint with a wooden pergola and a bench | 4 m high, 6 m wide | 3,000 | A | On a bend of the climb, about 2,300 m, over the sea |
| `ermita` | Small whitewashed hermitage with a bell gable | 10 m high | 3,500 | B | On the headland, by the hamlet |
| `km_post` | White kilometre post with a red top | 1.2 m high (shown at 1.5 m) | 200 | B | Every 500 m along the lap; many copies |
| `prickly_pear` | Clump of prickly pear cactus | 2.5 m high | 1,200 | B | On the dry slopes of the mountain and the cape |
| `agave` | Agave plant with a tall flower stalk | 4 m high with the stalk | 800 | B | Same, mixed with the cactus |
| `ibex` | Iberian ibex standing on a rock | 1.2 m high (shown at 1.5 m) | 1,500 | C | On the rocks above the cuttings |
| `camper_van` | 1970s camper van, an original design | 4.5 m long | 3,500 | C | Parked at the viewpoint |
| `castle_ruin` | Ruined Moorish hilltop fortress, walls and towers | 18 m high, 30 m wide | 8,000 | C | On the mesa, or on the mountain over the tunnel, seen from afar |

### Homage to the arcades (no brands)

| Name (`SIZES`) | What | Size | Triangles | Priority | Where |
|---|---|---|---|---|---|
| `arcade_billboard` | Retro roadside billboard on two legs, framed with light bulbs, painted with a sunset and palms | 7 m high, 10 m wide | 1,500 | B | On the start straight and the south coast |
| `palm_retro` | Chunky palm tree with big, blocky fronds | 12 m high | 1,500 | C | Only if the engine's palms read too fine at 224 lines |

## The prompts

Each prompt below is followed by the shared ending (see [Settings for every model](#settings-for-every-model)).

- **`white_house`:** A two-storey Andalusian whitewashed village house, white walls, terracotta tiled gable roof, one blue wooden door, two small windows with blue shutters, a black wrought-iron balcony with red geraniums in pots, a short chimney.
- **`watchtower`:** A 16th-century round coastal watchtower of weathered sandstone, cylindrical, slightly wider at the base, a small arched doorway high up the wall, a flat top with a low parapet, warm ochre stone blocks.
- **`chiringuito`:** A small Spanish beach bar kiosk of wood, a roof thatched with dry reeds, an open counter at the front, three bar stools, a few blank white parasols folded beside it, painted blue and white planks, no signs.
- **`espeto_boat`:** A small old wooden fishing rowboat lying on beach sand, painted white and blue with a red stripe, its hull filled with sand, a small wood fire in the middle with a row of fish on skewers leaning over it.
- **`retro_sports_car`:** An original 1980s-style red two-seater convertible sports car, low wedge-shaped body, pop-up headlights closed, wide rear lights strip, black interior, five-spoke wheels, generic design not based on any real car, no badges.
- **`lifeguard_tower`:** A wooden beach lifeguard tower on four legs with a ladder, a small cabin with a sloped roof, painted red and white stripes, a blank flag on a pole.
- **`flag_marshal`:** A race marshal standing, white overalls, a white cap, holding up a black-and-white chequered flag in the right hand mid-wave, friendly stylized proportions.
- **`banner_plane`:** A small vintage single-engine propeller plane, high wings, painted yellow and red, towing a long blank white fabric banner on a rope behind it, side view in level flight.
- **`curve_sign`:** A yellow diamond-shaped road warning sign with a black border and a bold black curved arrow bending right, on a grey metal post.
- **`boulder_a`, `boulder_b`, `boulder_c`:** A large weathered sandstone boulder, rounded and cracked, warm ochre and tan with a few darker streaks; `b` taller and narrower, `c` flatter and split in two.
- **`mirador`:** A small stone scenic viewpoint over a cliff, a low semicircular parapet of rough stone, a wooden pergola of beams on four stone pillars above it, a wooden bench under it.
- **`ermita`:** A small whitewashed Andalusian hermitage chapel, white walls, a terracotta tiled roof, a bell gable with one bronze bell above the arched wooden door, a small round window.
- **`km_post`:** A small roadside kilometre marker post of white concrete, rectangular with a rounded top painted red, a blank face.
- **`prickly_pear`:** A clump of prickly pear cactus, flat oval green pads growing on each other, a few red fruits on the edges.
- **`agave`:** An agave plant, a rosette of thick spiky blue-green leaves, one tall woody flower stalk rising from it with yellow flower clusters at the top.
- **`ibex`:** An Iberian ibex, a wild mountain goat with long curved backswept horns, standing proudly on a rock, brown and tan coat.
- **`camper_van`:** An original 1970s-style camper van, rounded boxy body, two-tone cream and turquoise paint, a pop-up roof, round headlights, generic design not based on any real van, no badges.
- **`castle_ruin`:** A ruined Moorish hilltop fortress, crenellated walls of ochre stone following the rocky ground, three square towers, one half collapsed, an arched horseshoe gateway.
- **`arcade_billboard`:** A retro roadside billboard on two steel legs, a wide rectangular panel framed with rows of round light bulbs, the panel painted with a stylized orange-and-pink sunset over the sea with black palm tree silhouettes, no text.
- **`palm_retro`:** A tall stylized palm tree, a slightly curved trunk with chunky ring segments, a crown of big blocky bright-green fronds, a few coconuts.

## Trademarks

Neither the props nor anything in the game names or copies a registered mark. In particular:

- **No game names in the game:** neither the names of the arcade racers this track pays homage to nor their makers. The prompts above don't use them either: the style is described instead (sunset, palms, coast road, bold colours).
- **No real vehicles:** the car and the van are original designs. Look at what Meshy makes: if it comes out recognisable as a real model (its grille, its lights, its proportions), make it again.
- **Not the black bull silhouette of the Spanish roadsides:** it is a registered mark, so no bulls on hills.
- **No real brands:** no drinks, beers or ice creams on the beach bar, the parasols or the billboards; no fuel brands on signs.
- **No real arcade cabinets or console logos.**

## Sending them

Send the GLBs named as in the tables (`white_house.glb`, `watchtower.glb`, …), all at once or a few at a time. A new importer (`tools/import_costa.py`, as `tools/import_props.py` is for Ghost Island) will make a pack of them in `assets/models/costa/`. A theme in `src/game/` will place them where the circuit's file says.
