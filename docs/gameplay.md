# Playing Motorust

Motorust is an arcade motorcycle racer inspired by *Moto Racer*. You pick a track and a bike and race seven rivals on a coastal road lined with walls. This page covers the screens, the bikes, how the bike handles, the controls, the options and the sound. The tracks are described in [tracks.md](tracks.md); building and running the game is in [building.md](building.md).

The game's menus are in Spanish or English: it starts in the system's language and can be changed in [Options](#options). This page quotes on-screen text in English.

## Screens

The game moves through five screens. Between them the camera glides from one place to the next instead of cutting; between the welcome screen, the players' lobby and the game modes, which share the same flight over the track, it just flies on.

1. **Welcome.** The game title, while the camera tours the track, over the start menu, which is exactly as wide as the title: two cards side by side, "1P" (one helmet) and "2P" (two helmets facing each other), and two full-width banners below, "OPTIONS" and "EXIT GAME". They are translucent black; the chosen one lights up in its colour (blue, purple, yellow or red) with a faint chequered flag sliding across it: to the right on 1P, to the left on 2P, up on Options and down on Exit. Arrows move around the menu; up or down from the banners comes back to the card last chosen. Esc or B asks whether to quit, with "Exit game" in red. There is no Exit in the browser: close the tab instead.
2. **Game mode** ("GAME MODE"). What to play (see [Game modes](#game-modes)), in cards like the start menu's: for one player Arcade big on top (yellow), and below race (blue), elimination (red), checkpoint (purple) and free ride (green); for two players race, elimination and free ride. The arcade goes straight to bike select; the others choose a track first.
3. **Track select** ("SELECT TRACK"). The track is seen from above, slowly turning, with its name, a short description, its length and a map of the layout with the finish line marked. Left and right switch tracks. Enter, Space, A or Start go on to bike select. The screen shows up at once: the other tracks are built in the background meanwhile, so switching to them is instant once they are ready. A track chosen before it is ready is prepared on a thread of its own, ahead of the others, and the menu does not wait for it: "Cargando el circuito…" ("Loading the track…") stands in for its length until it comes, and going on to bike select meanwhile goes as soon as it is there.
4. **Bike select** ("SELECT BIKE"). The bikes wait on the starting grid, three per row and staggered like a Grand Prix grid. The camera circles the chosen bike, and its name and colours are shown on screen. Left and right change bike. The right stick turns the view around the bike. Enter, Space, A or Start begin the race. The chosen bike idles here and revs each time you switch to it, over the sounds of the chosen track's surroundings instead of music: the sea and its gulls by the coast, birds in the trees, cicadas and the wind in the desert.
5. **Race.** Your bike starts last, so there is always someone to overtake. The other seven bikes are your rivals. Everyone waits on the grid for the start lights: a second after the grid is set up, five red lights (from one to five, in Options > Game) come on one by one, a second apart, each with a beep, and a moment after the fifth (never quite the same moment, from 0.3 to 1.3 s; or, if Options > Game says so, a second, on their beat) they go out, flash green with a higher beep, and the race is on. Until then you can rev the engine, but the bike does not move. In Options the lights can also leave the bikes free ("Real lights", once the first light is on): you can move off before they go out, but if you do you jump the start. The first false starts in a race (two at first, from none to three in Options > Game) are let off: "FALSE START! 1 of 2", everybody goes back to the grid and the lights start again, a moment later than usual to let go of the throttle. The next one disqualifies you, out of the race ("JUMP START! DISQUALIFIED", "disqualified" in the results, with no points in the arcade; Restart sets the race up again). Or the race can start as it used to: when you first open the throttle.

**Esc** or **B** goes back one screen: from bike select to track select, and from track select to the welcome screen. During the race, Esc pauses (see [Pause menu](#pause-menu)), and the pause menu can take you back to the bike selection or to the welcome screen.

Menus always work the same way, whatever controls you have set:

- Up and down (arrows, W and S, D-pad or left stick) move between items.
- Left and right change a value.
- Enter, Space or A accept.
- Esc or B go back.
- In Options, Tab (Shift + Tab backwards), Q and E, or the shoulder buttons and triggers change section.

### Controls legend

Every screen has the same legend at the bottom, with no panel behind it: each key or button, as it looks on the device you are using, and what it does there. It follows the last thing you touched: keyboard keycaps, or the buttons of an Xbox, PlayStation (white, with the symbols in their classic colours) or Switch controller, told apart by the controller's name. Moving around a menu goes without saying, so the legend only shows accepting, going back and, in Options, changing section. The symbols the UI font lacks (arrows and the PlayStation symbols) are drawn as icons.

## Game modes

| Mode | Players | What it is |
|---|---|---|
| Arcade | 1 | The five tracks in order, five laps each, with points for each place (10, 8, 6, 5, 4, 3, 2, 1) and the standings after each race. It always starts from the first track. |
| Race | 1 or 2 | A race on the track chosen, of 1 to 9 laps (5 at first), chosen with up and down on the track screen. |
| Elimination | 1 or 2 | Seven laps on the track chosen. Each lap, as soon as all but the last two still in have crossed the line, the one further back of those two is out: a rival leaves the race, a player's bike rolls to a stop. The last lap is raced by the two left, for the win. |
| Checkpoint | 1 | Against the clock: the clock starts with the seconds to reach the first checkpoint, and each checkpoint (the track's red "CHECKPOINT" arches, also on the minimap, and the finish line) adds the seconds of the stretch to the next one. At nought it is over; the laps done are the score. |
| Free ride | 1 or 2 | Ride a track with no end. Y (on a gamepad, the top face button) changes the music, as the line at the bottom of the screen says: the track's song, every track's song one after another from this one's, or three chill songs, a little quieter. Each free ride starts with the track's song. |

The **difficulty** (Options > Game) sets how fast the rivals ride and how much time the checkpoints give. Medium is the rivals as they always were; on easy they use a little less of their bikes, on hard a little more; on very hard more than that: most of them past what they dare on hard, braking later and riding the corners harder (the bike grips more with the steering near its stop), never faster than the bike goes, about 2 % quicker a lap than on hard. Against the clock, the time for each stretch is what it takes at 54 m/s on easy, 62 m/s on medium, 66 m/s on hard and 69 m/s on very hard, plus 3, 2, 1.5 or 1 s, and the start gives 5, 3, 2 or 2.5 s more for the standing start (on very hard a good ride is quicker away, but not that much): a good ride earns a few seconds at each checkpoint, a fall costs more than that.

**The finish.** When you cross the line at the end of the race, the machine takes over your bike and rides it on, as it does a rival's, so you do not crash. A moment later the results come up at the top right of the screen, over the race, which goes on behind them (with "Centre results when all are in" on in Options > Game, they move to the middle, above the bike the camera shows, once everybody is in): the others' times fill in as they cross the line, and the camera follows them in, one after another, from the one nearest to the line (top left, its name, place and lap; in the middle, as it crosses the line, its place and how far behind the winner). Once everybody is in, it shows them riding on, a few seconds on each, cutting to the one in the best spot: round a loop, on a road off the lap, coming up to bends or crests, or riding in a group, not going back to the last ones shown. In split screen this happens once both players are done; until then each half follows its own bike. In the arcade, "Continue" (Enter or A) goes on to the standings (and from there to the next race), and Esc or B to the game modes; in the other modes, "Race again" (Enter or A) sets the same race up again, and "Back" (Esc or B) goes to the track screen. Those still racing then are placed where they are, with the time they would have taken at their pace (marked "~"; in an elimination they are left racing).

## Bikes

There are eight bikes, each with its own rider. They are listed here in the order shown on the bike select screen:

| Bike | Colours |
|---|---|
| Shogun TZ | Blue and white, like the bike on the *Moto Racer* cover |
| Legend 88 | White and red |
| Azura RZ | Yellow, white and black |
| Kraken RS | Lime green and black |
| Falcon KRZ | Red, blue and white |
| Stiletto V4 | Red |
| Cobra XT | Orange and black |
| Ronin YS | Blue and black |

All bikes handle the same; they differ only in looks. On the minimap, each bike's dot uses the colour of its name. The bikes and riders are 3D models made with Meshy. They are fully animated: the wheels spin, the bars turn with the steering, and the rider tucks in on straights, hangs off in corners with the inside knee down, and sits up under braking. See [models.md](models.md) for how the models are prepared.

## Handling

The handling is arcade, like *Moto Racer*. The goal is a bike that is quick to place and forgiving. The parameters live in `BikeTuning` in `src/bike/mod.rs` (see [architecture.md](architecture.md)).

### Steering

- **Progressive.** Steering eases in and out with no jerks. On a gamepad, the left stick is progressive: near the centre it turns only a little, for fine corrections, and fully deflected it turns all the way.
- **Keyboard and D-pad.** These are on/off inputs, so the game shapes them:
  - The turn starts the moment you press, already quickly, and builds faster the longer you hold. A tap corrects your line. In about 0.4 s the bike turns as much as it can at that speed (0.3 s with the fast turn rate, see [Options](#options)).
  - The bike leans almost at once, because keyboard steering goes in directly, without the spring that smooths the stick.
  - On release, the steering returns to centre more gently: slowly at first, then faster. The bike is straight again in about 0.9 s (0.6 s with the fast return rate, 1.1 s with the slow one).
  - With nothing pressed, the bike does not turn. Switching from one side to the other passes through the centre.
- **Speed-dependent turn limit.** The keyboard and D-pad respond equally fast at any speed, but the faster you go, the less they turn when held. At low speed they turn fully. At top speed they stop at 80 % of full lock, just short of the point where the bike starts to scrub speed. By default both the stick and the keyboard use two thirds of that limit. Both can be changed under [Options](#options).
- **Physical limits.** Turning is limited by a minimum turning radius at low speed and by lateral grip at high speed. A stopped bike cannot turn.

### Leaning and cornering

- The bike leans in quickly and comes back up more gently. The lean matches what the corner asks for: a little in gentle bends, and the full lean only when you push to the limit.
- Corners are taken with the throttle open and do not cost speed. Leaned over, the bike drives slightly harder than upright, so it shoots out of corners.
- Near full lock the bike grips more and more. Only with the steering almost at full lock at speed, in the tightest corners, does the engine push less, so the bike sheds just enough speed to make the corner. Holding the turn is enough to stay on the road.
- Top speed is about 290 km/h. Climbs slow you down and descents speed you up.

### Grass, walls and other bikes

- **Grass.** Brushing it after running wide barely slows you. Staying on it slows you gradually to about 130 km/h, but it never stops you. Back on the asphalt, you recover twice as fast.
- **Walls.** Walls close off the road on both sides. Scraping one turns the bike parallel to it so you keep going. Hitting one head-on costs most of your speed. No part of the bike goes into a wall: not the front wheel when you hit it at an angle, nor the rider leaning towards it, nor a fallen bike's handlebar, nor a fallen rider's head, hands or feet. Where the earth of a rock face rises before the wall, it stops you just like the wall. Hitting a wall hard brings you down: going into it at more than about 36 km/h (straight into it at that speed, or a 12° hit at 180 km/h). That is a generous arcade margin; scrapes and bumps are ridden out.
- **Other bikes.** Contacts between bikes follow real momentum: both bikes weigh the same with their riders, the hit is mostly plastic (fairings and bodies do not bounce) and the fairings rub with some friction.
  - Rubbing fairings side by side pushes both bikes apart; the tyres scrub the sideways shove off in a fraction of a second and you keep riding. A nudge from behind slows the one behind and pushes the one in front on.
  - A rider only goes down when the tyres cannot take the hit: a big sideways jolt (a T-bone, a hard swipe at speed), a hit that spins the bike, or hitting another bike with the front wheel much faster than it goes. Contacts at walking pace never bring anyone down.
  - When a rider goes down, bike and rider separate. The bike drops onto its side and slides with the velocity it had after the hit. The harder the hit and the faster it was going, the harder it rolls over: a fall at low speed just lays it on its side, while at 200 km/h or more it barrel-rolls once or several times over its handlebars, footpegs and screen, bouncing up when it lands on its side hard, before it settles on one side. Stopped head on (into a wall, or into a bike much slower), it is thrown up and flips once end over end over its front wheel. Its wheels keep spinning, slowing down little by little. The rider keeps the momentum they had before it, so they are thrown on past the bike, land, tumble and slide on their back until friction stops them. Their arms and legs move as in real racing crashes: thrown, the arms let go loosely out to the sides and the legs come apart; tumbling, they go limp and bent (face down, knees to the ground and feet up); sliding on the back, feet first, the legs stay together and loosely bent and the hands are held up over the chest, off the asphalt, while the legs drag behind; once still, the arms rest by the sides. Off the bike the hands hang relaxed from the forearms instead of gripping. Hands and feet hang loosely from that pose, like on damped springs (the legs looser than the arms, each knee swaying on its own): they lag behind when the body is stopped, flop when it lands and fling out when it spins, without shaking, and never go under the asphalt (riders slide on asphalt with a drag factor of about 0.6, a bike on its side about 0.45, more on grass). Walls stop both.
  - Once bike and rider are still (4 s at most), the bike is put back on the track at once, upright and stopped, next to where it ended up, facing along the track and clear of other bikes.

### Jumps, wheelies and suspension

- **Jumps.** Taken flat out, crests make the bike take off. Jumps are short and sharp: in the air the bike falls with more gravity than in real life. You can correct your direction a little, but you cannot accelerate. Pulling back in the air lifts the nose.
- **Wheelies.** Hold the wheelie control (Space, or B on a gamepad) with the throttle open to lift the front wheel. With the wheel in the air you can barely steer, but you gain a little acceleration and top speed, so it pays off on straights.
- **Suspension and inertia.** The bike has weight you can feel:
  - The rear squats under acceleration and the fork dives under braking. Braking hard at speed lifts the rear.
  - In the air the suspension unloads and the wheels hang. On landing it compresses and rebounds while the wheels stay on the ground.
  - Inertia also moves the rider, back under acceleration and onto the tank under braking. It moves the camera too, which drops back as you accelerate and closes in as you brake.
- **Brakes.** The brake slows you down and, once stopped, reverses.
- **Engine braking.** Letting go of the throttle slows the bike down, and harder the faster you go, as the engine revs higher and the air pushes back: about 4 m/s² at 50 km/h and 7–8 m/s² at 250 km/h, still far from the brake.

### Branches and loops

On Loop Island a branch forks off the lap to the right on the climb to the start straight's crest (see [tracks.md](tracks.md#loop-island)). Keep right to take it, or stay on the lap. Its walls close in before a loop of checkerboard earth:

- **Round the loop** you steer as on the road: turn and you go across the running surface as you go round, and keep pointing that way once you let go. Ride straight on and the loop carries you round by itself, out beside the mouth. The low walls along its edges keep you on it: you scrape along them, losing more speed the more head on you hit them, but you never fall against them. Nobody can hit you. The throttle and the brake still work, and gravity slows you on the way up and speeds you up on the way down. The less the loop presses you against it, the less your tyres grip: slow near the top, you can hardly turn. You come out of the loop pointing the way you were, and the camera turns over with you and follows you as you turn.
- **Fast enough,** you go round, upside down at the top, and come out beside the mouth, on the road back to the lap. Off the throttle that takes about 130 km/h at the mouth; with the throttle open, the engine carries you round from much slower.
- **Too slow to reach the side,** you come back down and stop on the slope; reverse to roll out of the mouth, or open the throttle to try again.
- **Past the side but too slow,** the loop no longer holds you: bike and rider fall to the road below, and the bike is put back on the road once everything is still, as after any fall.

Two of the rivals always take the branch.

In Gold Mine's excavation the road splits three ways twice, with no loop (see [tracks.md](tracks.md#gold-mine)): in the sandstone and slate halls a high road forks off to the left and a deep road to the right, and in the clay and sandstone halls an upper gallery to the left and a lower one to the right, each between pillars of rock, narrower than the lap and laid with planks, over a hump or through a dip, and joining the lap again further on. The three ways fan out up to 130 m apart and none runs straight through: at each split one branch runs straight on where the lap swings away, and the other swings out before it. Keep to that side to take one, or stay in the middle on the lap: no way is clearly the quickest. Ghost Island's road splits in the open (see [tracks.md](tracks.md#ghost-island)): in two by the temple, and three ways along the castle ridge, each way between its own fences.

Gold Mine's wooden bridge over the lake, and Ghost Island's boardwalk across its inlet, are narrower than the road: their fences close in on them over the 25 m before, 6 m either side of the middle, and are as solid as walls.

### Rivals

Rivals keep to their own lane. They set off from the grid on their slot's line and go over to their lane once there is room: none of them moves across into a bike beside it, there or later in the race. They pull out to pass a bike they catch, or lift off if there is no room. They brake before corners and accelerate out of them. The most skilled rival starts on pole. On Loop Island two of them take the branch and go round the loop, flat out, steering to keep to their line. In Gold Mine and Ghost Island those two leave the lap at every split, and the others take the way on the side of their lane, or stay on the lap; coming back onto the lap, they give way to the bikes on it.

## Race HUD

| Element | Where | What it shows |
|---|---|---|
| Race | Top left | The figures white, what is not a figure ("º", "/", the words) a little translucent: your place, big, and how many race, smaller ("1º / 8"; in an elimination, how many are still in: 2º / 6 once two are out, the total popping bigger and red each time it changes; once you are out, your place among them all); the lap, as big, between its word and how many laps, smaller ("VUELTA 2 / 5", "LAP 2 / 5"); on the first lap the lap's time so far, with no word, and from the second on the last lap's instead ("ÚLTIMA" / "LAST"; in gold and a little bigger if that lap beat the track's record), until you cross the line at the end; and, if **Options > Game** says so, the race's whole time ("TOTAL"). In split screen, each player's in their half. |
| Start lights | Top middle | The five lights of the start (in split screen, across the line between the halves). |
| Time left | Top middle | Against the clock: the seconds left (red under 10) and what the last checkpoint gave. |
| Finished | Top right | In a race or the arcade, those who have crossed the line at the end, as they do: their place, name and time, with no panel behind. |
| Notices | Middle | Your place as you cross the line at the end, time up, out or disqualified, who has just gone out, a lap record (just after a lap that beats the track's best one kept; the first lap ever ridden on a track sets it without saying so). |
| Speedometer | Bottom right | A see-through dial, clear in the middle and darker towards its rim, marked from 0 to 360 km/h (red from 300, over a red arc), with a blue pointer and the speed in figures in the middle; the figure of the scale the pointer is at lights up blue, or bright red from 300, with no halo. With two players, each has their own at the bottom right of their half. |
| FPS counter | Top right | Frames per second, smoothed so it can be read. |
| Minimap | Bottom left | The layout seen from above (north up), with the finish line in yellow and one dot per bike in that bike's colour. Your dot is larger, with a white border. With two players, each has their own at the bottom left of their half: both players' dots carry their number (1 or 2), and on each one's map their own is the larger, over the other's. |
| Controls legend | Bottom centre | The keys or buttons currently bound to each action, on the device in use (see [Controls legend](#controls-legend)). It is hidden by default; turn it on in **Options > Control**. It gives way to the pause menu's own legend. |

The chase camera sits low behind the bike, as in *Moto Racer*. Its field of view widens with speed. On a gamepad, the right stick turns the camera relative to the bike:

- up looks ahead (the default);
- left or right looks to that side;
- down looks back, with the camera in front of the bike.

Release the stick and the camera looks ahead again.

## Controls

These are the defaults. Every action except the sticks, mute and fullscreen can be changed in **Options > Control > Mapping** (see [Mapping](#mapping)). Gamepad buttons are named as on an Xbox controller: A is the bottom button, B the right one and X the left one.

| Action | Keyboard | Gamepad |
|---|---|---|
| Throttle | W / ↑ | Right trigger (RT) / A |
| Brake, and reverse when stopped | S / ↓ | Left trigger (LT) / X |
| Steer | A, D / ←, → | Left stick (progressive) / D-pad |
| Wheelie (with throttle) | Space / Left Shift | B, or RT and A together |
| Look around | | Right stick |
| Restart the race | R | Select (when let go of) |
| Pause | Esc | Start |
| Mute or unmute sound | M | Select + Y |
| Toggle fullscreen | Cmd+Enter (macOS) / Alt+Enter (Windows, Linux) | Select + R3 (the right stick pressed in) |

On a gamepad the shortcuts are made holding Select (View on Xbox, Create on PlayStation, − on Switch) and pressing R3 (the right stick pressed in, RS on Xbox) or Y (the top face button). That is why Select restarts the race when it is let go of, and only if no shortcut was made while it was held; and Y held with Select does not accept. Start is no shortcut's: it always pauses.

The triggers are analogue: throttle and brake follow how far you squeeze them. The first 15 % of their travel does nothing, so a trigger that does not quite return to zero at rest (as a DualSense can, with its adaptive resistance on) neither opens the throttle nor stops the engine braking. Restart sends every bike back to its grid slot, and the rivals wait again until you accelerate. In the browser, both fullscreen shortcuts work, because the game cannot tell which system the browser runs on.

## Options

"Options", on the welcome screen and in the pause menu, has four sections. Changes apply immediately. The panel is centred above the legend and is as tall in every section, so the section tabs do not move. A section (or page) with more options than the panel holds, as Graphics at 1280 × 720, scrolls: the list moves as little as needed to keep the chosen option in view, a thin bar in the section's colour beside it shows how much of it is shown and how far down, and Reset and Back stay where they are, below it.

At the foot of the panel, whatever the section, stand "Reset" and "Back" side by side: down from the last option comes Back, and left or right goes to the other one. Reset puts the section's options back as they were at first (in Control, the steering variants and the legend; the keys and buttons are reset in Mapping); it is faint when they already are. In Control, under "Controls legend" and in a lighter hand than the legend (the keys only outlined, the words faint), are the shortcuts for the sound and fullscreen, as the keys or buttons look on the device in use; "Rescan gamepads" stands apart below.

Options that are on or off (fullscreen, the controls legend, pipelined rendering, the results in the middle) end in a box, rounded, ticked when they are on: filled with the colour of the section, or dark on the chosen row, which is already in that colour; empty and outlined when they are off.

- **Game.** The language (Spanish or English), the difficulty (easy, medium, hard or very hard, medium by default; see [Game modes](#game-modes)) and the race start ("Lights", the default; "Real lights", the bikes free on the grid and a jump start disqualified; or "Throttle", the race starting when you first open the throttle), how many red lights the start has (one to five, five by default; faint with a start at the throttle), the wait after the last one before they go out ("Random", the default, never quite the same, so the start cannot be timed; or "Fixed, on the beat", a second, as between one light and the next; faint with a start at the throttle), how many false starts are let off with the bikes free (none to three, two by default; faint with any other start), whether the results go to the middle once everybody is in (off by default) and whether the race's whole time is shown under the lap's ("Show the total time", off by default).
- **Control.** "Mapping" (see [Mapping](#mapping)), "Stick behaviour" ("Comportamiento Stick") and "Keyboard behaviour" ("Comportamiento Teclado"), the race's "Controls legend" (shown or hidden, hidden by default) and "Rescan gamepads". Each behaviour opens a page of its own in the panel ("OPTIONS > STICK", "OPTIONS > KEYBOARD") with its steering variants; Back, Esc or B go back to Control, on its row, and Reset there puts back only that page's variants (Reset on Control, all of them and the legend). The steering variants:

  | Page | Option | Values | Default |
  |---|---|---|---|
  | Stick | "Turn limit with speed" | off, low, medium, high | medium |
  | Keyboard | "Turn limit with speed" | off, low, medium, high | medium |
  | Keyboard | "Sustained turn rate" | slow, medium, fast | medium |
  | Keyboard | "Return rate released" | slow, medium, fast | medium |
  | Keyboard | "Turning the other way" ("rapidez de cambio de sentido") | slow, medium, fast | fast |

  The turn-limit levels apply a share of the speed-dependent limit described under [Steering](#steering): off is no limit, low a third, medium two thirds and high the whole limit (at top speed, holding the input gives 80 % of full lock). The rates scale how fast keyboard and D-pad steering builds up while a key is held (slow is half as fast as fast, medium three quarters), how fast it returns to centre once released (medium is half as fast as fast, slow a third), and how fast the bike and the rider swing over to the other side when the other key is pressed while turning: fast as always; medium at 60 % of that speed and slow at 35 %, both the bike's lean and the rider's body hanging off. The turn itself changes sides at once whatever the rate, so the bike's course is the same: only the swing is slower. With the stick nothing changes.
- **Sound.** "FX Volume" and "Music Volume", from 0 to 100 % in steps of 10 (80 % by default), and the music on or off. 80 % is the balance found by ear: the effects at a fifth and the music at nine tenths of the scale the game used before. Volumes saved with that older scale are not carried over: they start again at 80 %.
- **Graphics.**
  - **Fullscreen.** At any time, Cmd+Enter on macOS or Alt+Enter on Windows and Linux switches to fullscreen and back. The change is saved as if you had picked it here.
  - **Fullscreen type** (macOS only). "Espacio dedicado (escritorio)" (*Own Space (desktop)*, the default): fullscreen in a Space of its own over the desktop as it is, at its resolution; or "Pantalla completa dedicada" (*Dedicated fullscreen*): the screen switched to a display mode of its own, the one nearest the resolution chosen, in pixels, or with "Dynamic" the screen's own size. Settings saved before this option, in fullscreen with a fixed resolution, keep switching the screen. On Windows and Linux a fixed resolution in fullscreen switches the screen and "Dynamic" does not.
  - **Resolution.** "Dynamic" (the window adapts to the window and screen pixels; the default), a fixed 1280 × 720 window, 1600 × 900, 1920 × 1080 or 2560 × 1440. Sizes are in points, so on a Retina display the window has twice as many pixels. In fullscreen, "Dynamic" covers the screen as it is, and a fixed resolution switches the screen to its display mode nearest to it, in pixels (the fastest refresh rate of that size); on macOS, as the fullscreen type says. On a scaled Retina display (for example "looks like 2048 × 1152" on a 2560 × 1440 panel), "Dynamic" reports and draws the scaled size in pixels (4096 × 2304), more than the panel has: a dedicated fullscreen at the panel's own size draws one pixel to each of its pixels. In the browser the game fills the page.
  - **FPS.** The screen's rate (synchronised with it; the default), 30, 60, 120, 144, 240 or unlimited. With a fixed number, the game waits out the rest of each frame and stays synchronised with the screen so the image does not tear. If the screen is slower than the limit, the screen wins. This option is not available in the browser, which paces the frames itself.
  - **Trees.** "New" (the default): the trees modelled with Meshy, one for each species and firs among the tall trees of the forests; or "Classic": the trees built in code. They change at once, where they are.
  - **Performance mode (60 fps).** Off (the default): the shadows and the resolution scale as chosen. On: during a race the game lowers and raises them together to keep 60 frames a second (or the FPS cap, if lower): every second it looks at the frame rate, and under 95 % of the target goes down a ladder (one player: shadows high at 100 %, medium at 100 %, medium at 90 %, low at 90 %, then low at 80, 70, 60 and 50 %; two players, whose shadows are low already: 100, 90, 80, 70, 60 and 50 %), as many rungs at once as it falls short by (up to three); once the rate has kept up for ten seconds, a rung back up (waiting twice as long each time a step up is soon undone). Neither makes a pipeline again, so nothing stutters as they change. Meanwhile the Shadows and Resolution scale rows are faint and say what it has chosen ("Performance: low", "Performance: 80 % (…)").
  - **Shadows.** "Automatic" (the default): high, lowered a level while the frame rate (in a race, not paused) falls short of 60 frames a second (or the FPS cap chosen) and raised again once it has kept up for a while (longer each time a step up is soon undone); or "High" (four cascades out to 250 m, 2048 texels, as always), "Medium" (three out to 150 m) or "Low" (two out to 90 m, 1024 texels). Only the shadows change with the level, nothing else, so nothing pops in or out. At any level the shadows fade out over the last fifth of their reach, so a shadow comes in softly rather than all at once. With two players they are always at Low (unless "Less detail for two players" is off). The row says the level the automatic one has come to.
  - **Resolution scale.** How much of the window's resolution the world is drawn at: 100 % (the default, as always), 85 %, 70 % or 50 % of each side (the row says how many pixels: "70 % (2117 × 1329)"). Under 100 % the world is drawn smaller, stretched over the window and sharpened (contrast adaptive sharpening, the sharpening half of AMD's FSR 1); the interface is always drawn at the window's own resolution. The biggest saving on a Retina display, where "Dynamic" draws very many pixels. It can be changed during a race: it makes no pipeline again. The world is never drawn at more pixels than the screen itself has: on a Mac in a scaled display mode (a 1440p monitor in HiDPI "looking like" 2048 × 1152, whose window has 4096 × 2304 pixels; a laptop's "more space"), macOS shrinks the picture to the panel's pixels, so the share is of the window as the panel shows it: 100 % there is 2560 × 1440 and 85 % is 2176 × 1224.
  - **Antialiasing.** "MSAA 4×" (the default, as always), "FXAA" (cheaper, a little softer) or "No". Fixed, not automatic: changing it makes every pipeline again, and what they draw vanishes for a moment while they are made.
  - **Lighter far trees.** Off (the default): every tree is drawn whole, as before. On: trees far from the camera are drawn with lighter meshes of the same shape (from 100 m and 280 m; with the screen split, from 50 m and 150 m), fewer triangles at the cost of a change you may see as they come near.
  - **Less detail for two players.** On (the default): with the screen split the world is drawn twice, so it is drawn lighter: the sun's shadows nearer and coarser, the grass, flowers and petals less far, and, if the lighter far trees are on, the trees lighter sooner. Off: as with one player.
  - **Advanced** ("Avanzado"): a page of its own ("OPTIONS > ADVANCED", with its own Reset; Back, Esc or B go back to Graphics on its row), not in the browser, with the three options below.
    - **Performance log.** On (the default, not in the browser): every five seconds a line with the frame rate (average, slowest 1 %, frame times), how much of each frame the simulation took, the CPU and memory used by the game and the whole system, and what was being played with which options and at which resolution, in a file of its own for each session, named after when it began, the system, the CPU and the GPU (`2026-10-16_1830_macos-26-3-1_apple-m1-pro.perf.log`) and headed with the game's version and commit, beside the settings (`~/Library/Application Support/motorust/logs` on macOS, `%APPDATA%\motorust\logs` on Windows, `~/.config/motorust/logs` on Linux; the game names it in its output when it starts writing). A line ends early when the screen, the mode or an option changes, so each one is for one of them.
    - **Pipelined rendering.** On (the default), each frame is drawn while the next one is simulated: more frames per second when the processor holds them back, a frame more between the controls and the screen. Off, each frame is drawn right after it is simulated, with less lag.
    - **Frame latency.** How many frames may wait to go to the graphics card: 1 (least lag), 2 (Bevy's own, the default) or 3 (steadiest).

  The last two take effect the next time the game starts; until then the row says "on restart". Neither is shown in the browser.

### Mapping

Control > "Mapping" opens a view of its own, titled "OPTIONS > CONTROL". Instead of the sections it offers the keyboard and the gamepad, each mapped apart; it opens on the one you are using, and follows it if you switch. Left and right on that row, Q and E, or the shoulder buttons change it (the keys or buttons are shown at its ends, as on the sections' row). At the foot, side by side: "Test" (chosen as the view opens), "Remap", "Reset" and "Back". The devices' row is in Control's blue, like the section; below it, what stands out (the button chosen, what is held, what a test lights) is a neutral white.

- **The device, drawn.** A keyboard (with its arrows), or the gamepad in use as it looks: a DualSense, a DualShock 4 or a DualShock 3 for PlayStation pads (told apart by their name), and an Xbox controller for the rest (on a Switch pad, with its own letters on the face buttons); the triggers stand above it. On the gamepad's page the actions are listed beside it. On the keyboard the keys that do something are a lighter grey. The keys and buttons held down light up only while testing or remapping (the keys white, the gamepad's buttons darker), not while you move round the view. Under it, each action with its keys or its buttons.
- **"Test".** Try the keys (or buttons) out: what is held lights up, the actions it does light up in the list, and the line under the device says it ("W > Throttle"); nothing else works meanwhile: the arrows move nothing, A (or Enter) accepts nothing, B goes nowhere, and the shortcuts (sound, fullscreen) do nothing. Only Esc, or B held for 3 seconds on a gamepad, ends it, as the line under the device says.
- **"Remap".** Press the key (or button) to remap: it turns orange, and its action is named. Then press the new key, or several together: they light up in yellow, and count once all are let go of. The new key or combination does that action instead of the old one (the action keeps its other keys), and stops doing anything else. A key that does nothing says so; Esc, or waiting eight seconds, calls it off. M cannot be mapped, because it always toggles sound, nor Esc, which calls off.
- **"Reset".** Puts the keys of every action back as they were at first, or the buttons, on the gamepad's page; the other device is left as it is.

Keys or buttons pressed together do the action only when all of them are held. Some controls cannot be changed: the left stick (steering), the right stick (look around), M (sound), the shortcuts held with Select, and the menu navigation. Pulling the left stick back does nothing: it used to pop a wheelie, which was easy to do by mistake while steering. Controls saved without a wheelie button get B, unless B already does something else. The race's controls legend always shows the keys you have chosen.

### Where settings are saved

Settings (language, difficulty, race start, fullscreen, resolution, FPS limit, pipelined rendering, frame latency, controls, volumes, and the best lap and most laps against the clock of each track) are saved each time they change, in `ajustes.ron` ("settings") inside the system's settings folder:

| System | File |
|---|---|
| macOS | `~/Library/Application Support/motorust/ajustes.ron` |
| Linux | `~/.config/motorust/ajustes.ron` (or under `$XDG_CONFIG_HOME` if set) |
| Windows | `%APPDATA%\motorust\ajustes.ron` |
| Browser | `localStorage`, under the key `motorust.ajustes` |

They are read at startup. If the file is missing or cannot be parsed, the game uses the defaults and prints the reason in the terminal. The steering variants and the race legend are not saved: they go back to their defaults each time the game starts.

## Pause menu

Esc or Start (or whatever you bound to Pause) stops the race and muffles its sound. The "PAUSE" menu offers:

- "Resume" ("Continuar"), the main action: blue when chosen, with nothing behind it otherwise, like the rest.
- "Back" ("Volver"): leaves the race for the bike selection, the screen before it.
- "Options": the same [Options](#options) as on the welcome screen; closing them comes back here.
- "Return to main menu": returns to the welcome screen.

Up and down select, Enter or A accept, and Esc or B resume the race. To quit the game, go back to the main menu and choose "Exit game" there.

## Gamepads

Gamepads are read with **gilrs** and fed into Bevy's input, like `bevy::gilrs` does.

- **Status.** On the menu screens, the bottom-right corner shows one of:
  - "Mando: …" (Gamepad: …) with the name of each gamepad found;
  - "Buscando mando..." (Searching for a gamepad...);
  - "Sin mando: si tienes uno, reconéctalo o prueba otro modo del mando" (No gamepad: if you have one, reconnect it or try another controller mode).
- **Searching again.** While no gamepad is seen, the game searches again every 2.5 seconds for about 20 seconds, as if you had reconnected it. On macOS, gilrs sometimes misses a gamepad that was already plugged in at startup. Pause > Debug > "Volver a buscar mandos" starts a new search at any time. Gamepads plugged in while the game is running are picked up as usual.
- **macOS.** Gamepads the system knows (DualSense, DualShock 4, Xbox, Switch Pro and MFi) are read through Apple's **GameController** framework. With gilrs, the DualSense showed up as connected but sent no input. All other gamepads are still read with gilrs, and no gamepad is counted twice. Gamepads in XInput mode are not seen as gamepads on macOS: switch the controller to another mode.
- **Diagnostics.** The terminal you start the game from logs each gamepad found (name, vendor and product ID), the first input it sends, each search, and a warning if a connected gamepad sends nothing for 10 seconds.
- **Browser.** Browsers only expose a gamepad after a button has been pressed with the page open.

## Sound

Every sound effect is synthesised in real time, with no recordings. The synthesiser runs on the audio thread (`src/sound/synth.rs`). The game only decides what plays, how loud, and from where (`src/sound/mod.rs`). The only recordings are the music.

- **Engines.** Each bike has a 1000 cc Grand Prix V4 that revs to 17,500 rpm. It has a "big bang" firing order: two pairs of explosions close together give it a deep, rough roar.
- **Gearbox.** A six-speed straight-cut gearbox that whines. You hear:
  - the revs climbing and near-instant upshifts;
  - automatic throttle blips on downshifts;
  - the rev limiter;
  - the clutch slipping as you pull away;
  - pops and crackles when you lift off.
- **Rivals.** Rival engines are louder the closer they are, panned left or right. They sound higher-pitched approaching than moving away (the Doppler effect), so you hear them come and go.
- **Surroundings.** Wind that grows with speed; tyres when you push in corners or brake hard; grass; impacts against walls; landings.
- **The moments of a race.** Short tunes, synthesised too: a bright fanfare up a major chord when a player wins; two notes a fifth apart, held, when a player crosses the line in any other place; a quick sparkle of bells for a lap record (and for the most laps against the clock); a short falling sweep when a rival goes out of an elimination; and three notes falling a semitone each, like a trombone, when a player is eliminated or disqualified, or their time is up. They play at the FX volume.
- **Two players.** Both players' bikes sound as yours: full volume, no Doppler. Every bike, rivals included, is heard from the nearer of the two cameras, and the wind, tyres and grass follow whichever player makes them louder.
- **Menus.** Moving through menus clicks. On the bike select screen, the chosen bike idles and revs when you switch to it.
- **The track's surroundings.** On the bike select screen, instead of music, the sounds of the chosen track's surroundings, as much of each as the track has (`src/sound/ambience.rs`): waves that swell, break and wash back, seagulls calling now and then, small birds chirping, cicadas buzzing and the wind. They fade in and out with the screen, about as loud as the bike idling.
- **Music.** The menus (the welcome screen, the lobby, the modes and the tracks) have their own song, "Asphalt Shift", low in the background; Y (or the top face button) turns the music on or off there, as the legend says, the same as Options > Sound. Each track has its own song (`assets/music/`), which loops during the race (`src/sound/music.rs`); in free ride Y changes it (see [Game modes](#game-modes)), and the chill songs ("Velvet Dashboard", "Sunlight on Still Water" and "Leaves on the Still Pond") play 4 dB quieter. Every song is brought to the same loudness above 500 Hz, where the engines' drone does not cover it, so they all sound as loud over the bike (one with heavy bass is not drowned out). It still plays well below the bike, its loudest second at least 10 dB under your engine flat out, so the revs, gear changes, tyres and impacts are always heard over it. It fades in when the race starts and out when you leave it.

Pausing muffles the race and turns the music down. **M** mutes and unmutes everything, music included. In the browser, sound starts after the first key press or click, because browsers do not allow audio before that.
