# Tracks

Motorust has five tracks, picked on the track select screen (see [gameplay.md](gameplay.md#screens)). All five are closed roads with walls on both sides, slopes, crests where the bike takes off when you are flat out, and banked corners: three on the coast under a summer sky, a desert mountain at dusk and an island of feudal Japan. Loop Island also has a branch: an alternative route off the lap that goes round a loop the bikes ride upside down. Gold Mine has four more, without loops: other ways through its excavation, past pillars of rock; Tsushima Isle has four too, out in the open.

| Track | Length | In the game's words |
|---|---|---|
| [Bahía GP](#bahía-gp) ("the Bay") | 1.6 km | "Una carretera junto al mar, con cuestas, peraltes y un salto en la recta de meta" (a seaside road with slopes, banking and a jump on the start straight) |
| [Turbo Bay](#turbo-bay) | 3.1 km | "Inspirada en Moto Racer: paseo marítimo, túnel bajo la propia pista, puerto de montaña y puente rojo" (inspired by *Moto Racer*: seafront, a tunnel under the track itself, a mountain pass and red bridge) |
| [Loop Island](#loop-island) | 2.1 km | "Inspirada en Sonic: un desvío con un rizo para dar la vuelta, tierra a cuadros y cascadas" (inspired by Sonic: a branch with a loop to ride, checkered hills and waterfalls) |
| [Gold Mine](#gold-mine) ("Mina Dorada") | 5.95 km | "Inspirada en las minas de Mario Kart y Crash: vagonetas, una montaña cruzada por dentro y por encima, y una corta inundada" (inspired by the mines of *Mario Kart* and *Crash*: ore carts, a mountain crossed through and over, and a flooded open pit) |
| [Tsushima Isle](#tsushima-isle) | 5.89 km | "Inspirada en Ghost of Tsushima: un castillo cruzado por debajo y por encima, cuestas, un lago entre flores y dos bifurcaciones" (inspired by *Ghost of Tsushima*: a castle crossed under and over, steep hills, a lake among flowers and two splits) |

Distances below are in metres from the finish line, as in the track data.

## Bahía GP

Bahía GP is the first track: *Moto Racer*'s Speed Bay reimagined as a Mediterranean bay. It is a seaside road under a summer sky, with the sea to the west and south and mountains closing the horizon to the north-east. Its data is in `src/circuits/bay.rs`.

- **Scenery.**
  - Stone walls line the whole lap, with street lamps along it.
  - Walls of cut rock rise beside the road. On the start straight the rock is on the right and the sea on the left.
  - Umbrella pines, cypresses and olive trees.
  - Whitewashed houses and others in cream, ochre and pink.
  - Rocky outcrops inland and small islets offshore.
- **Start.** A red arch with a chequered board spans the finish line. The curbs are red and white.
- **Slopes.** About 150 m into the lap, the start straight crosses a crest where the bike jumps when you are flat out; it lands on the straight, well before the first bend. The road then climbs to the high eastern side, about 18 m above the start, with rock on one side and a drop on the other. From there it runs back down to the finish.
- **Banking.** Corners bank according to how tight they are. This track has no extra banked spans.

## Turbo Bay

![Racing out of the blue start arch on Turbo Bay's seafront](images/turbo-bay.jpg)

Turbo Bay is inspired by Speed Bay in *Moto Racer*. It is a figure of eight on a hilly island, with the sea all around and two bays cutting in towards the island's waist. The first loop runs clockwise, with the sea on the left; the second runs anticlockwise, with the sea on the right, and climbs a mountain at the island's northern tip. Two thirds of the way round the lap, the road comes back over the ridge it went through in a tunnel. Its data is in `src/circuits/turbo_bay.rs`. The curbs are blue and white.

The coastline is drawn on its own (`Coast::Outline`), not as a distance from the road, so its bays, headlands and the river mouth under the bridge do not follow the track. Hills, a rocky ridge at the waist and islets offshore complete the island.

![Over the tunnel: the pack crosses on top while the road below goes under the grassy hill](images/turbo-bay-crossing.jpg)

A lap, in order:

1. **The seafront** (start straight, 0–250 m, heading north). The blue "TURBO BAY" start arch spans the road, with a grandstand on the right.
   - On the left: a green railing with lamps, a beach with parasols, and the sea with striped buoys and boats.
   - On the right: palm trees and tall, brightly coloured hotels behind a hedge.
2. **Down to the water and into the tunnel** (about 250–625 m).
   - The seafront ends in a crest where the bike takes off.
   - A steep right-hander, heavily banked, drops almost to the water at the north-west point.
   - The road climbs east into a lit tunnel (505–625 m) through the ridge at the island's waist.
3. **The grand hotel and the rock pass** (about 625–990 m). Out of the tunnel, past palms and an old-style grand hotel on a headland above the south-east bay, to a pass between rocks with billboards and a red "CHECKPOINT" arch (960 m).
4. **The east coast road** (about 1000–1280 m). It runs behind a white concrete barrier, with the sea on the right. It is a roller coaster: a crest where the bike jumps with the sea straight ahead, a dip, and the start of a long climb.
5. **The mountain road** (about 1280–1720 m). The road climbs on north in S-bends, with rock walls on the left and the barrier and the cliffs above the sea on the right, up to 11 % at its steepest. Half way up it dips, then climbs again to the summit at the island's northern tip, 52 m above the start, round a long left-hander with a lighthouse on the cape below. A rocky crag stands inside the bends.
6. **The forest** (about 1660–2160 m). Down the west side through tall trees, along the cliffs of the north bay: a hump, a dip and a rise past a second "CHECKPOINT" arch (1980 m), then a right-hand kink onto the ridge.
7. **Over the tunnel** (about 2160–2450 m). The road runs down the ridge and crosses over the tunnel, about 18 m above the road inside it, on the grassy hill the tunnel goes through.
8. **The bay shore and the bridge** (about 2450–2805 m). The road runs down the shore of the south-east bay behind a concrete barrier to a bridge with red arches over the mouth of a river (2720–2805 m).
9. **The town** (about 2830 m to the finish). The road passes the town and its arcades, with rock on the right, and turns back onto the seafront.

![The mountain road climbing north between rock walls, with the cliffs and the sea on the right](images/turbo-bay-mountain.jpg)

![The summit at the island's northern tip, with the lighthouse on its cape below](images/turbo-bay-summit.jpg)

## Loop Island

![Loop Island's branch forking off the start straight on the climb to the crest, in a long curve to the loop, with the lap going on over the crest on the left](images/loop-island.jpg)

Loop Island is inspired by Green Hill, the first zone of *Sonic the Hedgehog*: a roller coaster of a road under a blue sky, running clockwise round an island of green hills with the sea all round. Its data is in `src/circuits/loop_island.rs`, and its landmarks are drawn by `src/world/landmarks.rs`.

- **Scenery.**
  - Every cliff, rock wall and mesa is checkerboard earth: chocolate and caramel squares under a grass top. The walls along the road are low blocks of the same earth, with hedges on the start straight.
  - Palm trees, round broadleaf trees and rows of tall sunflowers turned towards the road.
  - The loop of the branch (see below), and a giant loop on a block of checkerboard earth inland from the south shore, in the plane of the road, so you see it side on as you ride past, as in the original game.
  - Carved totem poles with eyes, beaks and wings.
  - Distant mountains across the sea to the north and west; to the south and east, open sea.
- **Start.** A blue "LOOP ISLAND" arch with a grandstand on the right. The curbs are yellow and blue.
- **A lap, in order:**
  1. **The start straight** (0–290 m), north between palms and sunflowers. A crest at about 195 m makes the bike take off.
  2. **The first long corner** (about 300–540 m). A long right-hander climbs round the loop of the branch, then S-bends roll on east over the hills to a red "CHECKPOINT" arch (700 m).
  3. **The tunnel** (770–870 m). The road climbs through a ridge of checkerboard earth in a tunnel lined with it, its highest point, 18 m above the start.
  4. **The waterfalls** (about 880–1230 m). Out of the tunnel, a waterfall pours down the rock wall on the left into a long right-hander banked like a velodrome. The road drops past a second waterfall, on the right.
  5. **The wooden bridge** (1230–1310 m). A deck of planks on log piers, with a wooden fence on each side, crosses an inlet where the sea runs in under the road. It is narrower than the road: its fences stand 6 m from the middle, and the walls close in on it over the 25 m before it.
  6. **The S-bends** (about 1340–1650 m). A long, gentle rise, the second "CHECKPOINT" arch (1420 m), totem poles and the giant loop, inland on the right.
  7. **The south shore** (1660–1960 m). A beach road with palms on the sea side and sunflowers on the other, rolling gently, and a banked right-hander back to the start. The S-bends and the shore have no crests to fly off: in the air a bike neither turns nor leans, so a crest in a bend, or one landing in a bend, would straighten it up.
- **The branch with the loop.** On the climb to the crest of the start straight (110 m), a narrower road forks off to the right. It peels away in a long, gentle right-hander (about 160 m radius), over the shoulder of the crest and gently down to the mouth of a loop of checkerboard earth with a 16 m radius, heading north-east with its walls closing in. The running surface is asphalt with yellow and blue curbs, between low walls of the same earth. The bikes ride round it, upside down at the top, and come out beside the mouth, as in Sonic's loops. The road back sweeps east in wide curves (200 m and 340 m radius), climbing gently (no steeper than 5 %), and joins the lap on the S-bends (637 m). It is about as long as the lap it leaves out, and two of the rivals always take it.

![Two bikes riding round the loop, one upside down at the top](images/loop-island-loop.jpg)

## Gold Mine

Gold Mine ("Mina Dorada") is a gold mine in a desert at dusk, after Wario's Gold Mine in *Mario Kart 64* and Dragon Mines in *Crash Team Racing*, with the roads of West Way in *Moto Racer* round it: red and ochre rock, ore carts on rails, timber trestles and headframes, dunes with oil pumpjacks and a gas station, a narrow red canyon, adobe houses and big diamond warning signs. The lap twists and rolls all the way round, and is a figure of eight that crosses a mountain twice: first through it, east to west, by the galleries and halls of the mine, where the road splits three ways twice; and then over it, north to south, on a road that passes 70 m above the galleries. Its data is in `src/circuits/gold_mine.rs`; the works of the mine are drawn by `src/world/mine.rs`, its excavation by `src/world/cavern.rs`, and the roadside of the desert by `src/world/desert.rs`. Its song is "Toll of the Iron Mile". The curbs are rust red and cream; timber fences line the road, and stone parapets the rim of the pit and the switchbacks.

- **The light.** A low orange sun in the west-north-west, in front on the start straight; a violet sky, rose above the horizon and orange at it, glowing round the sun; warm, slanting light, purple shade and peach clouds. The distance melts into an orange haze (`SkyStyle::Dusk`).
- **The land.** Dry red earth on the flat, with saguaro cacti and tumbleweeds blowing across the road; low dunes of red sand north of town; in the middle of the eight, a mountain 78 m high and over a kilometre across, with smooth slopes, a broad top and shoulders the road climbs onto it on, and towers of red rock standing up from it (`climbed_massif` and `massif`, see [Scenery](#scenery)); red buttes round about and mesas with horizontal bands of red, rust and ochre rock all round the horizon; north-west of the mountain, an open pit nearly 600 m across, after the Corta Atalaya at Rio Tinto: five benches 3.4 m high, cut in steps from its rim down to a great round lake of red water, 470 m across, at its bottom, following its shore, and on up into the mountainside where it is cut into it, with spoil heaped round the rim elsewhere and a red river running out of the lake to the north (`pit` and `water`, see [Scenery](#scenery)).
- **A lap, in order** (5.95 km):
  1. **The mining town** (start straight, 0–340 m, north). A gold "GOLD MINE" arch, a grandstand, wooden houses with false fronts standing up over their roofs and porches along them, lanterns on posts, piles of crates, barrels and dynamite, and a train of three ore carts rolling up and down rails on the right. A crest at about 185 m where the bike takes off.
  2. **The dunes** (340–1000 m). A kink to the right and up onto the dunes, a straight over a blind crest (about 595 m) where the bike takes off, past a gas station on the right, with its pumps under a yellow and red canopy and a tall "GAS" sign, and oil pumpjacks nodding on the tops of the dunes either side; round a sharp left at their far end, down into a hollow and up again.
  3. **The red canyon** (1000–1655 m). South in esses between sheer red walls 36 m high, rolling up and down between 11 and 23 m, through a red "CHECKPOINT" arch (1200 m), under a concrete highway viaduct 14 m up from one wall to the other (1300 m), and under a timber trestle (1625 m) where an ore cart crosses over the road, into the east mouth of the excavation.
  4. **Through the mountain** (1655–2555 m, 900 m), west by its middle, dark but for the lanterns: down into the mine from 22 m to 5.5 m at the bottom, and up again to 21 m at the west mouth. Its walls stand straight up from the road's walls to above a rider's head and lean out a little higher; from there the roof rises in a rough arch, with domes, hollows and stalactites, to the crown of each hall. In order:
     - **A timbered gallery** at each mouth: an arched mouth framed in heavy timber, a vault about 10 m high propped every 8 m by a timber set the shape of the vault, lanterns hanging from the rafters of every other one, and planks on the ground beside the road.
     - **The first split** (1706–2056 m), in a hall of red sandstone (about 1712–1880 m, its roof up to 28 m: layered red, cream, rust and ochre rock, red gravel) and one of slate (about 1880–2050 m, the highest roof, up to 34 m: dark slate with veins of gold winding over the roof and seams of it in the walls, black gravel). The three ways fan out up to 130 m from side to side, and none runs straight through: the lap swings 26 m to the right and back across to the left; the **high road**, to the left, runs straight on where the lap swings away and then out round a wide curve, 66 m from the lap at most, over a broad hump 4.6 m above the lap beside it; the **deep road**, to the right, swings out early, 65 m from the lap, through a dip 4.2 m deep. They run side by side, each in its own gallery, between pillars of rock where the roofs come down to meet them.
     - **The timbered neck** (about 2050–2160 m), at the bottom of the mine, where the road over the top of the mountain crosses 70 m overhead (at about 2100 m).
     - **The second split** (2156–2504 m), in a hall of ochre clay (about 2162–2330 m, its walls lined with planks to 3 m, beaten earth, a roof up to 24 m) and one of sandstone again (about 2330–2500 m, up to 31 m), the first one mirrored: the lap swings left first, the **upper gallery**, to the left, swings out early over a hump 4.5 m high, and the **lower gallery**, to the right, runs straight on and out round a wide curve through a dip 4.2 m deep.

     The ways off the lap are narrower (10 m of asphalt, between walls that close in as they part from the lap), laid with planks between beams, and lit like the halls. None is clearly the quickest: flat out, a rival takes each way through a split within 5 % of the quickest. In the halls, lanterns hang on posts by the walls, one side and then the other, every 14 m along each road, and every third one high up on a tall pole, lighting the rock far above. The halls blend into each other: the roof rises and falls over some 30 m, and where one stone ends and the next begins wavers.
  5. **Along the rim of the pit** (2555–2905 m). Out of the west mouth between stone parapets, rolling along the rim of the pit past a second "CHECKPOINT" arch (2640 m), a train of ore carts on rails, a heap of ore, a "DANGER TNT" sign and a headframe whose winding wheels turn, and round to the right onto the bridge.
  6. **The wooden bridge** (2905–3425 m). From the rim straight across the flooded pit to the north-east for nearly half a kilometre, planks on log piers 18 m over the red water, narrower than the road (its fences 6 m from the middle, the walls closing in on it over the 25 m before it), the benches stepping down to it all round, with a crest in the middle (about 3230 m) where the bike takes off.
  7. **Up the switchbacks** (3425–4580 m). Up the mountain's north flank, from 18 m to 72 m, between stone parapets: off the bridge through a third "CHECKPOINT" arch (3540 m), along the shore of the lake, round a hairpin to the right and one to the left, with red diamond signs before them, over a steel truss bridge across the gully between the shoulder and the mountain (4200–4390 m), past a fourth "CHECKPOINT" arch (4440 m), and round a long sweep onto the top.
  8. **Over the top** (about 4580–5000 m), south, 75 m up, across the road through the mountain beneath, past a heap of ore and a headframe over a shaft. Over the excavation the rock that covers it carries the road, and a rocky ridge runs along it either side.
  9. **Down the esses into town** (about 5000 m to the finish). Down the mountain's south-east flank in esses at up to 10 %, with yellow and green diamond signs before the bends, through the fifth "CHECKPOINT" arch (5250 m), easing past an adobe pueblo on the outside of a bend (5450 m), with its flat roofs, beams sticking out under them, ladders and a dry tree, past a "MINE 49" sign and more piles, and a left-hander into the start straight.

## Tsushima Isle

Tsushima Isle is an island of feudal Japan, after *Ghost of Tsushima*, each of its stretches with a colour of its own, as the game paints its regions: the pink of the castle town, the blue of the coast, the vermilion of a shrine on the water, the white of the susuki grass, the flowers round a lake, the gold of a temple, a red ridge climbed through a corridor of torii, as at Fushimi Inari in Kyoto, and green bamboo. The lap is a figure of eight that crosses the castle hill twice: first through it, by a tunnel at the end of the castle street, and at the end of the lap over it, through the castle, 29 m above the tunnel. It climbs and drops hard, at up to 10 %, and winds most of the way. Its data is in `src/circuits/tsushima.rs`. Its song is "Blade of the Autumn Gale". The curbs are vermilion and white; stone walls line the castle street, the coast and the castle ridge, a hedge the outside of the road round the lake and a shrine's vermilion fence, low and open, its lake side, and timber fences the rest.

Its scenery is the island's own, much of it made with Meshy (see [models.md](models.md)) and the rest in code (`src/world/japan.rs`, `src/world/flowers.rs`): Japanese trees (momiji, ginkgo, sakura, black pine, sugi and bamboo), dense cedar and maple up the red ridge and bamboo down from the castle; the great torii standing in the sea off the coast, the castle keep on its hill, the temple and its five-storey pagoda, the shrine on stilts in the inlet, thatched minka, stone and bronze lanterns, Jizō statues, komainu, braziers, a log torii and a water wheel turning by the lake; the corridor of torii up the red ridge; the great maple beside the road round the lake, over a clearing of red leaves ringed with logs, as in the duels of *Ghost of Tsushima*; red maple leaves blowing down the red ridge and round the great maple, and white cherry petals down the castle street; and fields of flowers over most of the island (more than 60 % of its grass), after its fields in *Ghost of Tsushima* and the moss phlox round the lakes at the foot of Mount Fuji: carpets of moss phlox (shibazakura) all round the lake in bands of magenta, pink, lavender and white, white flowers round the great maple for its reds to stand out, red spider lilies beside the castle street and on the lake's open shore, golden susuki over the susuki hills, yellow rape blossom and white flowers up the slopes over the coast road, and everywhere else broad swathes of them with grass between. Its grass is darker than the other tracks' (`grass_shade`), the moss phlox and the great maple's reds muted. Across the strait to the north stands a lone volcano, as Mount Yōtei or Fuji, under dark forest, bare scree above it and snow on its top, straight ahead down the castle street over the castle and behind the great torii in the sea. Its light is a clear autumn day (`SkyStyle::Clear`): a softer blue sky, a light haze far off, a lower, warm sun and cool shade, and the whole picture graded luminous and contrasted but not saturated, as Japan's landscapes look.

- **The land.** An island (`Coast::OutlineWithLakes`), with the sea right beyond the walls along the coast road, an inlet the boardwalk crosses and a lake inside it, 200 m across, with an islet in it; the castle hill over the tunnel, 46 m high, with rock round its mouths; the castle ridge and the red ridge east of it, the susuki hills along the east coast and the bamboo slopes west of the castle, mountains the road climbs (`climbed_massif`); crags on the red ridge and stacks of rock in the sea off the coast road; across the strait to the north, a lone volcano 900 m high, its flanks sweeping up ever steeper to a crater, snow on its top third, its foot just behind the great torii, and lower mountains round the rest of the horizon, clear of it.
- **A lap, in order** (5.89 km):
  1. **The castle street** (start straight, 0–440 m, north). A vermilion "TSUSHIMA" arch, a grandstand and the houses of the castle town, a crest at about 185 m where the bike takes off, and straight at the castle hill, into the tunnel under it (440–620 m).
  2. **The coast** (640–1420 m). Round to the right out of the tunnel and east along the shore in esses, rolling, with the sea right beyond the wall on the left, stacks of rock in it and a fishing village by the road.
  3. **The shrine's boardwalk** (1420–1620 m). Straight across the inlet, narrower than the road (its fences 6 m from the middle), with a crest in the middle (about 1540 m) where the bike takes off.
  4. **The susuki hills** (1620–2700 m). Round to the right and up in esses at up to 9 %, from 4 m to 34 m, south over their rolling top, with two crests shaped like the castle street's where the bike takes off (about 2345 and 2535 m), and down at up to 10 % towards the lake.
  5. **Round the lake** (2700–3400 m). Clockwise, round three sides of it, 175 m from its middle, along a shrine's vermilion fence on the right, over carpets of moss phlox down to the water and its islet, past a water wheel and, on the south shore (about 3115 m), the great maple over its clearing of red leaves, 31 m from the middle of the road.
  6. **The temple's split** (3414–3764 m), north, climbing at 3.4 %: Gold Mine's second split (where its lap climbs too), turned north. The lap swings left and back across to the right through the courtyard, the way to the left swings out early over a hump, and the way to the right runs straight on where the lap swings away and out round a wide curve through a dip.
  7. **The corridor of torii up the red ridge** (3764–4322 m). In bends at up to 10 %, from 19 m to 51 m, most of the way (3830–4120 m) narrower than the road, its walls 6.5 m from the middle, under the corridor's gates, one every 6 m, with wide ones over the road before and after, and round to the left over the top onto the castle ridge, west.
  8. **The castle** (4322–4880 m). Down along the castle ridge at 3.4 % (steeper before and after it, so that the road splits where its grade eases), through the second split, Gold Mine's first (where its lap comes down too): the lap swings right first, the way to the left runs straight on and out round a wide curve over a hump, the way to the right swings out early through a dip; and over the tunnel through the castle hill (about 4810 m), 29 m above it.
  9. **The bamboo** (4880–5600 m). Down from the castle in esses, through dense forest, from 28 m to 3 m, and round to the left into the castle street.

## Editing or adding a track

### Where tracks live

Each track is a `Circuit` in `src/circuits/` (`bay.rs`, `turbo_bay.rs`, `loop_island.rs`, `gold_mine.rs` and `tsushima.rs`). A `Circuit` has six fields:

- `name`: shown on the track select screen.
- `description`: shown on the track select screen; `\n` breaks the line.
- `song`: the music for the race, an MP3 in `assets/music/` embedded with `include_bytes!`, its `length` in seconds (as `ffprobe` gives it) and its `loudness`: the integrated loudness (LUFS) of its first two minutes above 500 Hz, which `ffmpeg -t 120 -i song.mp3 -af highpass=f=500,ebur128 -f null -` prints as `I:` in its summary. The game turns each song up or down by the difference so they all sound equally loud over the engine (whose drone covers the lower notes), and well under it.
- `ambience`: what its surroundings sound like on the bike select screen: how much sea, seagulls, small birds, cicadas and wind (0 to 1 each).
- `layout`: the road itself (a `Layout` from `src/track.rs`).
- `scenery`: everything around the road (a `Scenery` from `src/world/scenery.rs`).

`CIRCUITS` in `src/circuits/mod.rs` lists the tracks in the order the track select screen shows them. The first one is also the default track. To add a track:

1. Create a module next to the others that defines a `pub const` `Circuit`.
2. Declare the module in `src/circuits/mod.rs`.
3. Add the track to `CIRCUITS` and update the array length.

When a different track is picked, the world is rebuilt from its `Circuit` (see [architecture.md](architecture.md)).

### Layout

`Layout` (`src/track.rs`) describes the road:

- **`control_points`**: points in metres on the ground plane (`Vec2(x, z)`), in race order. The centre line is a closed centripetal Catmull-Rom curve through all of them, resampled every 2 m. The start is at the first point, and the start straight runs towards −Z.
- **`elevation`**: the height profile, as pairs of (metres from the finish line, road height in metres). The profile is closed and interpolated smoothly between pairs. Heights are added to a base level 2 m above the sea; every track starts the profile at 0 on the finish line.
- **`banking`**: banking comes automatically from curvature. The tighter the corner, the more the road tilts inwards, up to a fixed maximum. A `Banking { from, to, factor }` span multiplies it between two distances, still capped at the maximum. A factor above 1 banks more and below 1 banks less.
- **`branches`**: alternative routes off the lap, each with a loop the bikes ride (see [Branches and loops](#branches-and-loops)). Most tracks have none: `branches: &[]`.
- **`narrows`**: stretches where the walls close in closer than the edge of the asphalt, as on the wooden bridges of Gold Mine and Loop Island, or Tsushima Isle's boardwalk and corridor of torii, narrower than the road: `Narrowing { from, to, wall }`, the walls `wall` metres from the centre line between `from` and `to`, closing in over the 25 m before and opening out over the 25 m after (`NARROW_EASE`). The road counts as ending half a metre short of them (for the grass, the rivals' lines and where a fallen rider rejoins). Most tracks have none: `narrows: &[]`.

The road is 14 m wide plus curbs, and the walls stand 11.5 m from the centre line (`WALL_DISTANCE`).

The centre line may cross itself only where one stretch goes through a tunnel and the other passes well above it, at least 12 m higher, as in Turbo Bay. The physics follows each bike, its rider and the rivals on the stretch at their own level (`Track::nearest_at`), and the scenery takes care of the rest (see `tunnels` below).

The example below is illustrative and is not a real track. It shows a short loop with a crest on the start straight and an extra-banked hairpin:

```rust
use bevy::prelude::*;

use crate::track::{Banking, Layout};

const CONTROL_POINTS: [Vec2; 10] = [
    // Start straight, towards -Z.
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, -250.0),
    // Hairpin to the right.
    Vec2::new(40.0, -330.0),
    Vec2::new(130.0, -350.0),
    Vec2::new(210.0, -300.0),
    // Back straight and the last corners.
    Vec2::new(230.0, -200.0),
    Vec2::new(220.0, -60.0),
    Vec2::new(180.0, 40.0),
    Vec2::new(90.0, 70.0),
    Vec2::new(20.0, 50.0),
];

/// (metres from the finish line, height in metres).
const ELEVATION: [(f32, f32); 7] = [
    (0.0, 0.0),
    (120.0, 1.0),
    // Crest on the start straight: flat out, the bike jumps.
    (200.0, 6.0),
    (250.0, 3.5),
    (450.0, 2.0),
    (750.0, 10.0),
    (950.0, 3.0),
];

pub const LAYOUT: Layout = Layout {
    control_points: &CONTROL_POINTS,
    elevation: &ELEVATION,
    // The hairpin, with twice the banking its curvature gives it.
    banking: &[Banking {
        from: 260.0,
        to: 480.0,
        factor: 2.0,
    }],
};
```

In a real track, the layout goes straight into the `Circuit`, as in `turbo_bay.rs`: `layout: Layout { control_points: &CONTROL_POINTS, elevation: &ELEVATION, banking: &[...], branches: &[], narrows: &[] }`. (The example needs `branches: &[]` and `narrows: &[]` too.)

### Branches and loops

A `Branch` (`src/track.rs`) is a road that forks off the lap and joins it again further on, with a vertical loop in the middle:

A branch may also have no loop (`vertical_loop: None`, `exit: &[]`): then it is another way along the lap, through the control points of `approach`, as in Gold Mine's excavation.

- **`from`, `to`**: where it leaves the lap and where it joins it again, in metres from the finish line. Pick straight, gentle stretches of the lap.
- **`approach`**: control points of the road from the fork to the loop, and **`exit`**: those of the road from the loop back to the lap. Each road is an open Catmull-Rom curve, like the centre line of the lap. It starts on the lap's centre line, going its way, and ends on it; give points along a smooth curve that peels away (an arc a dozen metres apart works well). The last 50 m into the loop and the first 50 m out of it are added for you, straight along it.
- **`vertical_loop`**: `Some(VerticalLoop { mouth, heading, height, radius, shift })`, or `None` for a branch without a loop. `mouth` is where the bikes go in (the middle of the running surface at the bottom), `heading` the way they go in and come out, `height` the height of the road there (like the heights of `elevation`), `radius` that of the running surface, and `shift` how far to the left (negative: to the right) the way out is from the way in. The running surface drifts sideways as it goes round so that it does not run into itself; the shift must be more than the width of the running surface.

The roads off the lap are narrower than the lap (10 m of asphalt plus curbs). While a branch shares the lap's road, it lies on the lap's surface, so the two are one road, with the lap's walls; once its walls part from the lap's, it goes over to its own height and banking. Its own height runs smoothly along it: from the lap's height and grade where it parts, it climbs or drops to the height of the loop, which it reaches level 25 m before the mouth (and leaves level 25 m after the way out), spread over all the road in between, with no ramps or dips. So that the turn off the lap is gentle, fork where the lap is straight and give the road room to peel away: it shares the lap's road (and its crests) until it is about 23 m from the lap's centre line. Its walls close in to the running surface of the loop over the last 70 m before it, and open out again after it. Where the two roads meet, neither draws walls, lines or curbs on the other's road.

A branch without a loop runs at the height of the lap's surface beside it, higher or lower by its **`rise`**: pairs of (metres along the branch, metres above the lap beside it), through which the difference runs smoothly (Hermite with Catmull-Rom tangents), none at its ends; `&[]` for none. (Far from the lap, where its nearest point moves along the lap's centre line in fits and starts, how far along the lap it is beside is smoothed over 8 points either side first; and once its walls have parted from the lap's, it goes over, by 45 m further away, to the height of the lap as far along it as it counts as, which runs on steadily however much the lap winds.) It has its own banking once apart. Keep a rise to where the branch is well apart from the lap, 30 m or more from the lap's centre line, where it has a surface all its own, and keep its crests gentle: Gold Mine's are broad-topped humps and dips with gentle edges (sharp in the hollows, where nothing takes off), 4–5 m over some 180 m, with no crest sharper than about 0.003 1/m and grades under 11 % where the lap beside it climbs or drops at about 3.5 %. Flat out, the bikes ride through a split near their top speed, so which way is quicker is mostly a matter of how long each way is and how sharply it bends, not of its climbs: in Gold Mine all three ways are about as long, and none runs straight through: at each split the lap swings one way and back, one branch runs straight on where the lap swings away before curving out, and the other swings out early. Its walls are as wide as the lap's while the two share the road, and close in to 8.5 m from its centre line as it parts from the lap, so that past about 35 m from the lap there is rock between the two. Leave and join the lap gently: its first control point a metre or two from the lap's centre line, 15 m along, and its last likewise before the join. A bike is on whichever road's centre line is nearest, so the walls of each road are the edges of the rock between them. Rivals that take the branches with loops take every branch without one too; the others take one on the side of their lane, 2 m or more to that side of the middle (see `Rival::takes` in `src/bike/ai.rs`). Over the last 120 m of a branch, a rival gives way to any bike on the lap about level with it and near the line it will join the lap on, dropping in behind it. Each rival sees the others across the road it rides itself, so one coming off another road where two meet is seen where it is.

Round the loop (`src/bike/loop_ride.rs`), a bike rides on the running surface: steering turns it on the surface as on the road and takes it across it, and the walls along its edges (at the same distance from the middle as the walls at the mouth) keep it on it, though nobody falls against them. Riding straight on, the surface carries it round and sideways by the shift, out beside the mouth. Other bikes are left out. Its speed comes from the engine, the brake and the resistances as on the road, plus the part of gravity along its way. Too slow to reach the side, it comes back down and can reverse out of the mouth; past the side, too slow for the surface to hold it, it falls off and drops to the road below, with its rider. It comes out pointing the way it was. The chase camera turns over with it and follows it as it turns.

### Scenery

`Scenery` (`src/world/scenery.rs`) is described mostly by **spans** of the lap. A span is `span(from, to)` in metres from the finish line. It may wrap past the finish line: `span(2440.0, 300.0)` covers the end of one lap and the start of the next. `LAP` is the whole lap. `Side::Left` and `Side::Right` are relative to the direction of the race.

| Field | What it describes |
|---|---|
| `coast` | Where the sea is. One of: `Coast::Function(f)`, a signed distance to the shore (positive inland); `Coast::AroundTrack(&[(along, distance)])`, an island with land inside the loop and, outside it, out to that many metres from the centre line at each point of the lap (a negative distance brings the sea in under the road, making an inlet for a bridge); `Coast::Outline(&[Vec2])`, an island whose shore is a smooth closed curve through the given points, drawn like the centre line, with its own bays, headlands and inlets (draw an inlet under a bridge to bring the sea in under it); or `Coast::OutlineWithLakes(&[Vec2], f)`, the same island with lakes inside it, `f` the signed distance to their shores (negative in the water), as at Tsushima Isle. On an island, the land drops to the sea (and to its lakes) right beyond the walls where the shore is near. |
| `hills` | Height of the rolling hills (m). |
| `greens`, `grass_shade` | How the grass is coloured, each track its own: `Greens::Lawn`, the arcade lawn, a bright green a little lighter or deeper here and there (the coastal tracks); or `Greens::Countryside`, many greens: broad swathes of fresh, yellow, olive, deep and bluish greens, lusher by the water, drier and yellower on the heights and the slopes turned to the sun, and the dark green of conifer forests up the hills, over the land far from the roads and on the mountains across the sea (Tsushima Isle). And how light it is: 1 as painted, less for darker, less vivid greens (0.36 at Tsushima Isle); the grass in flower darkens half as much. |
| `massifs` | Free-standing rock masses: `massif(x, z, radius, height)`, rising sheer right behind the walls of any road near them. Also used for islets. `climbed_massif(x, z, radius, height)` is a mountain the road climbs instead: smooth slopes with a broad top that are the ground itself, the road cut into them or built up to them within 30 m of its flat verges (and round a tunnel through it, at once, as steeply as rock). |
| `cuttings` | Walls of cut rock beside the road: `Cutting { span, side, height }`. |
| `terraces` | Flat circular platforms at a given height, to stand a large building on: `Terrace { center, radius, height, skirt }`, the ground blending back into the land over `skirt` metres past the edge (wider where the land round it stands far above or below it, so the bank stays grass instead of breaking into a cliff). |
| `mountains` | A function for the distant mountains that close the horizon. Their slopes are under dark conifer forest; above about 400 m (the tree line, higher or lower here and there) they are bare volcanic scree, and above about 560 m (higher or lower along the ravines), on slopes not too steep, under snow. Far from the track the terrain's cells grow up to 70 m, so a peak keeps its shape. |
| `curbs` | The two curb stripe colours. |
| `walls` | Wall style per span and side: `WallRun { span, side, style }`. `WallStyle` is `Stone` (the default everywhere else), `Concrete` (white barrier), `Railing` (green seafront railing), `Hedge` (hedge on a low wall), `Checkered` (blocks of checkerboard earth), `Timber` (a fence of heavy planks) or `ShrineFence` (a low vermilion fence of posts and rails, as round a shrine: like the railing, the ground beyond it shows through, where a wall would hide whatever lies on the ground near the road). The first run that covers a point wins, so a run over `LAP` after the others sets the default. |
| `tunnels` | Spans that run through a tunnel. Another stretch of the track may pass over a tunnel (see [Layout](#layout)): there the ground stays down at the level of the tunnel, under its rock, and the rock carries the road above, just below its asphalt. Where the tunnel's top is gentle, grass grows on it unless the soil is checkerboard. |
| `bridges` | Spans carried by a bridge, each built its own way: `Bridge { span, style }`, with `style` one of `RedArches` (a concrete deck under two red arches, with concrete walls), `Wooden` (planks on log piers, with a wooden fence instead of walls) or `Truss` (a concrete deck between two steel through trusses braced overhead, on concrete piers, with concrete walls, as in West Way). Under a bridge the ground keeps its natural shape, below the deck: water, or a gully where the road runs high over it. |
| `soil` | What cliffs, rock walls and the rock around tunnels are made of: `Soil::Rock` (natural rock with strata), `Soil::Checkered` (Loop Island's checkerboard earth, which also lines the tunnels) or `Soil::Mine` (a desert canyon: dry red earth and banded red rock; its tunnels are a mine's excavation, see `mine` below). |
| `gantries` | Arches over the road: `Gantry { along, color, text }`. Without text they carry a chequered board. Those reading "CHECKPOINT" are the checkpoints of the checkpoint mode (with the finish line) and are marked on the minimap: keep them about a kilometre apart (550–1500 m, a test checks it), off bridges and branches. |
| `billboards` | Roadside signs: `Billboard { along, side, text, color, accent }`, where `accent` recolours the text from a given letter. |
| `lamps`, `palms`, `hotels` | Rows along the road: `Row { span, side, lateral, every }`, at `lateral` metres from the centre line, one every `every` metres. Hotels are tall and colourful. Nothing in a row stands on a branch or right beside its walls. |
| `trees`, `forests` | Species of the scattered trees (umbrella pine, cypress, olive, tall, broadleaf, maritime pine, poplar, almond, and the Japanese momiji, ginkgo, sakura, black pine, sugi and bamboo), and the forests: `forest(span, &[species])`, a span with trees of those species close together on both sides. |
| `houses` | `Houses { max, paints, spans, style }`: how many at most, their paint colours, optionally the spans they are limited to, and how they are built: `HouseStyle::Cottage` (plaster under a tiled gable roof) or `HouseStyle::FalseFront` (the same, with a wooden false front standing up over the roof and a porch along the front, as in the Old West). |
| `grand_hotel` | `Some((along, lateral))`: an old-style grand hotel at that point of the lap and signed distance from the road (positive to the left). |
| `buoys` | Positions of striped buoys in the sea. |
| `lighthouses` | Coastal lighthouses with striped towers, observation gallery and beacon: `Lighthouse { position, height }`. |
| `boats` | Sailboats and yachts anchored in the bay and open sea: `Boat { position, heading, kind }`. |
| `chevrons` | Curve warning arrow signs on sharp bends: `ChevronSign { along, side, direction }`. |
| `grandstands` | Covered spectator grandstands with crowds along the straights: `Grandstand { along, side, lateral, length }`. |
| `beach_parasols` | Striped beach umbrellas and loungers on the sand: `ParasolCluster { position, count }`. |
| `brake_markers` | Roadside braking countdown signs (100 m, 50 m) before corners: `BrakeMarker { along, side, distance }`. |
| `sunflowers` | Rows of tall sunflowers turned towards the road: `Row { span, side, lateral, every }`. They only grow on grass. |
| `totems` | Carved totem poles beside the road, facing it: `Totem { along, side }`. |
| `loops` | Giant loops on a block of checkerboard earth, to look at: `Loop { along, side, lateral, radius }`, with `lateral` the distance from the centre line to the middle of the loop. The loops the bikes ride belong to the branches of the layout. |
| `rings` | Lines of golden rings spinning over the road: `Rings { along, count, lateral, height }`, one every 6 m from `along`, `lateral` metres to the left of the centre line and `height` metres above the road. No track has them now. |
| `waterfalls` | Waterfalls with flowing water down the rock wall of a cutting: `Waterfall { along, side, width }`. Put a `Cutting` on that side first. |
| `sky` | The light: `SkyStyle::Day` (a summer day), `SkyStyle::Clear` (a clear autumn day, as in Japan: a paler, softer blue, a light haze far off at the horizon, so the mountains across the sea stand out, the sun lower and warm and the shade cooler, and the picture graded luminous, with contrast, but not saturated, as at Tsushima Isle) or `SkyStyle::Dusk` (a low sun in a warm, hazy sky, as in the Gold Mine). Each style's colour grade is its own: the summer day and dusk are not graded. |
| `water` | The colour of the sea, the lakes and the rivers: `Water::Sea` (saturated blues and white foam) or `Water::Iron` (the wine-red, rust and ochre water of a flooded mine rich in iron, with a pale yellow scum, as in the Gold Mine). |
| `pit` | An open pit round the water, or `None`: `Pit { benches, step, tread, spoil }`, that many benches, each face `step` metres high and steep, each bench `tread` metres wide, cut from just above the water up to the rim and following the shore (the coast's distance), with the spoil heaped round the rim sloping down to the ground over `spoil` metres; where the ground round it is higher than the rim, the benches go on up into it, a step at a time. Past the flat verges of a road, the road's level gives way to the benches within 12 m; under a bridge they stay under the deck. |
| `desert` | The roadside of a desert of the American West, as in West Way (*Moto Racer*), or `None`: `Desert { gas_stations, pumpjacks, overpasses, pueblos, curve_signs, cacti, tumbleweeds }`. `GasStation { along, side, lateral }` is a gas station facing the road, its pumps under a canopy, a shop behind and a tall "GAS" sign by the road; `Pumpjack { position, heading }` an oil pumpjack whose beam nods as its crank turns; `Overpass { along, height }` a concrete highway viaduct across the road, the bottom of its deck `height` metres up, from one rock wall into the other (put a `Cutting` on both sides first); `Pueblo { along, side, lateral }` a cluster of flat-roofed adobe houses with beams sticking out under their roofs, ladders and a dry tree; `CurveSign { along, direction, color }` a big diamond warning sign high on a post on the outside of a bend, green (`SignColor::Green`), yellow (`Yellow`, sharp) or red (`Red`, a hairpin), with an arrow bending the way of the bend; `Cacti { count, spans }` about that many saguaros scattered along the spans either side, on level dry ground clear of the roads; and `Tumbleweed { along, reach }` a tumbleweed blown across the road there, rolling and bouncing from `reach` metres out on one side to as far on the other (hopping over the walls), over and over. |
| `japan` | The scenery of feudal Japan, or `None`: `Japan { corridors, props, maple_grounds, petals, flowers }`. `ToriiCorridor { span, every, approach, approach_every }` stands a narrow torii over the road every `every` metres where its walls close in along `span` (give it a `Narrowing`), and a wide one every `approach_every` metres over the `approach` metres before and after it. `props` are the Meshy models (`PropModel`): `by_road(model, along, lateral)` facing the road, or `standing(model, x, z, heading)` anywhere, `.scaled(k)` times as big; each stands on the lowest of the ground under it, or in the sea with its foot under the water (a boat floats, a water wheel turns). `MapleGround { center, radius }` is a clearing carpeted with red maple leaves, ringed with logs (level its ground with a `Terrace`). `Petals { area, kind }` drift down over `PetalArea::Road { span, reach }` (out to `reach` metres either side of the lap) or `PetalArea::Round { center, radius }`: red maple leaves (`PetalKind::Maple`) or white cherry petals (`PetalKind::Cherry`). `FlowerField { area, kinds }` is a field of flowers: its grass is drawn in flower, with a texture of the lawn covered in blossoms painted at startup (so a field is a carpet of colour from near and from afar, at no cost but a texture), and near the roads clumps of its flowers stand up out of it. Its kinds (`FlowerKind::SpiderLily`, `White`, `Yellow`, `Iris`, `Susuki`, and `Phlox`, moss phlox, a carpet hugging the ground in bands of magenta, pink, lavender and white, without clumps) grow in patches of each; it lies over `FlowerArea::Road { span, near, far }` (both sides of the lap, from `near` to `far` metres from its centre line), `FlowerArea::Round { center, inner, outer }` (a ring, or from 0 a round meadow that trees and houses keep out of) or `FlowerArea::Everywhere` (the rest of the island, with gaps of grass between its patches). Where fields overlap, the first one listed decides; past the middle of a field's ragged edge, the next one. Flowers only grow on dry grass beyond the walls' verges, over the island (not on the mountains across the sea), and the clumps only within 34 m of a road, on ground not too steep and clear of what stands on it. Remember that walls hide what lies on the ground near the road: put fields on ground rising away from it, beyond 40 m or so, or behind a see-through fence. |
| `mine` | The works of a mine, or `None`: `Mine { rails, trestles, headframes, lanterns, piles, halls }`. `Rails { span, side, lateral, carts }` lays rails on the ground beside the road with a train of that many ore carts rolling up and down them; `Trestle { along }` carries a rail across the road from one rock wall into the other, with a cart going over and back (put a `Cutting` on both sides first); `Headframe { along, side, lateral }` is a timber tower over a shaft with its winding wheels turning; `lanterns` are rows of lanterns on posts; `Pile { along, side, lateral, kind }` is a pile of crates, barrels and dynamite (`PileKind::Supplies`) or a heap of ore with gold in it (`PileKind::Ore`); `halls` are the halls of its excavation (its `tunnels`), `Hall { span, stone, crown }`: the stone they are cut through (`Stone::Sandstone`, `Stone::Slate`, `Stone::Clay`, or `Stone::Timbered`, the gallery propped with timber that the excavation is wherever there is no hall) and how high the roof rises over the middle of the road (m). |

### What `cargo test` checks

Run `cargo test` (or `make test`, see [building.md](building.md)) after any change. Every track in `CIRCUITS` is checked for the following:

- **No impossible corners.** Every corner radius is more than two and a half times the wall distance.
- **No self-crossing.** Distant stretches of the track never come close enough for their walls to meet; there must be room for scenery between them. The one exception is a stretch passing over a tunnel of another, at least 12 m above it.
- **Start.** The grid sits just behind the finish line, on the Z axis, facing down the start straight towards −Z. Every track starts at the origin.
- **Narrow bridges.** Where the walls close in on a narrow stretch, they stand where it says, nearer than the edge of the asphalt, the road ending short of them; they close in and open out without a step, and elsewhere stand where they always do.
- **Rideable slopes and banking.** Slopes stay under 13 %, banking stays within its maximum, and the tight corners do have banking.
- **Terrain.** The terrain neither covers the road nor leaves a gap under it, except on bridges and where the road passes over a tunnel, and the road never touches the sea.
- **Houses.** At least half of `houses.max` find a place to stand.
- **A rival can do a lap.** An AI rival completes a lap within 90 seconds without leaving the road.
- **A flat-out lap.** Full throttle, never braking and only steering towards the road ahead, completes a lap within 100 seconds (Gold Mine takes about 87) without touching the walls and stays almost entirely on the asphalt.
- **Landmarks.** No loop's block comes within 2 m of the walls, rings float over the asphalt within reach of a bike, and every waterfall pours down a rock wall more than 6 m high.
- **Mines.** Rails lie on the ground beside the road, outside its walls and never steep, and every trestle runs into rock on both sides, above its deck, where its cart goes. In Gold Mine's excavation, the roof is above a rider's head everywhere inside the walls, over 15 m high in the halls and 8–13 m in the timbered galleries; the walls of the rock stand where the bikes hit them; there is rock half way between each branch and the lap; every stone is drawn, with gold, planks, the outside of the rock and more than 30 lanterns, brighter by a lantern than high above it; and where the road over the mountain crosses it, the outside of its rock carries that road, level with it, more than 40 m above the roof. An open pit steps down to the water bench by bench, each one level and a step above the one below, and on up into higher ground the same way; Gold Mine's great lake lies at the bottom of one, its benches where they should be all round wherever no road passes; the water of a mine is red, the sea blue.
- **The desert.** A pumpjack's beam rises and falls once a turn of its crank; cacti stand on level dry ground clear of the roads; tumbleweeds roll from one side of the road to the other, over its middle, never under the ground and up off it over the walls; a diamond sign's corners are clear and its arrow points the way of the bend, and each stands before a sharp bend that way; gas stations and pueblos stand clear of every road's walls, and pumpjacks on fairly level ground away from the roads; a viaduct runs more than 7 m over the road, into rock higher than its deck on both sides. Concrete bridges (red arches and steel trusses) have a deck out to their walls level with the road.
- **Japan.** Every prop is in the pack and stands on the ground (or in the water where it should), clear of the roads; the gates leave the road clear under their beams and the corridor's stand round the road beside its walls; a clearing of maple leaves lies on level ground ringed with logs, short of the walls; petals and leaves fall from the top of their column to the ground, swaying, over their areas; the grass in flower covers most of the grass (but the shoulders) with every kind of its fields; the clumps stand in it near the roads (no moss phlox), clear of the walls and of what stands on the ground, not too many, a mesh of three cards a clump for each square of ground; moss phlox grows in bands of its colours.
- **Short jumps.** Each jump lasts under 0.75 s and rises less than 1.3 m, and flat out a rival does take off over the crests (at least twice in 70 seconds).
- **Jumps on the straights.** Rivals on three lines never take off or land in a bend (tighter than 600 m), except on Turbo Bay, whose seafront drops over a crest into its corner on purpose, as in *Moto Racer*.
- **Branches.** Each starts and ends on the lap's centre line, going its way, on the lap's surface; its roads go into and out of the loop straight, level and with the walls closed in; corners and slopes stay rideable; once apart from the lap it climbs and drops smoothly, with no ramp or dip sharper than the lap's crests, and its walls never meet the lap's again; a bike on it finds its road; and a rival that takes the branches goes round the loop and back onto the lap, and comes out of the loop at any speed without leaving the ground. A branch without a loop leaves the lap and joins it again along it, on its surface, on one side of it, with rideable curves, grades under 12 % and no crest sharp enough to take off from flat out, walls as wide as the lap's where they share the road and more than 10 m of rock between them somewhere; a rival rides it if its lane is on that side (and the lap if it is in the middle), and is back on the lap a little past the join (before any split that follows); and where the road splits three ways, the way to the left, the lap and the way to the right take rivals about as long, the slowest within 5 % of the quickest. On Loop Island, the branch forks more than 50 m before the crest of the start straight, and its road to the loop is never tighter than a 100 m radius.

Some tests cover a single track: for example, that Bahía GP has sea to the west and mountains to the north-east; that Turbo Bay is an island with its bays and a river under the bridge, that its banked span banks more than its curvature alone would, and that it passes over its own tunnel, with each level found by the bikes on it and the ground kept down under the tunnel's rock; that Loop Island is an island of checkerboard cliffs with the sea under its wooden bridge; that Gold Mine crosses its mountain through it, by the middle, and then over it, climbing 45 m and more steadily (the switchbacks and the truss bridge included) and coming down 60 m and more into town, its road over the top crossing the excavation more than 50 m above it, each level found by the bikes on it and the ground kept down in the excavation; that up and down the mountain its road is cut into the mountainside rather than sunk in a canyon; and that its bridge crosses the great lake for more than 450 m; and that a pack of rivals races a lap of it (and of the Bay, and of Gold Mine through the forks of its excavation, and of Tsushima Isle through its splits) with at most two falls, the two that take its branch going round the loop. Tsushima Isle goes through its castle hill by the middle and over it at the end of the lap, more than 25 m above the tunnel; it climbs the susuki hills 30 m and more, and the red ridge to its top 30 m and more, at up to nearly 10 %, and comes down from the castle 35 m and more; its road rounds the lake on three sides, about 75 m from its shore, with the lake on its right and an islet in it, and crosses the inlet on its boardwalk; and its volcano rises across the strait, alone, with a crater and snow at its top, straight ahead down the castle street, raising nothing by the road. Its grass is more than 60 % in flower (but the shoulders), every kind of each field growing somewhere. The loop riding itself is tested on its own: going round, coming back down too slow, falling off, and going in at the mouth.
