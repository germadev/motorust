# Playing Motorust

Motorust is an arcade motorcycle racer inspired by *Moto Racer*. You pick a track and a bike and race seven rivals on a coastal road lined with walls. This page covers the screens, the bikes, how the bike handles, the controls, the options and the sound. The tracks are described in [tracks.md](tracks.md); building and running the game is in [building.md](building.md).

The game's menus are in Spanish or English: it starts in the system's language and can be changed in [Options](#options). This page quotes on-screen text in English.

## Screens

The game moves through four screens. Between them the camera glides from one place to the next instead of cutting.

1. **Welcome.** The game title, while the camera tours the track, over the start menu, which is exactly as wide as the title: two cards side by side, "1P" (one helmet) and "2P" (two helmets facing each other), and two full-width banners below, "OPTIONS" and "EXIT GAME". They are translucent black; the chosen one lights up in its colour (blue, green, yellow or red) with a faint chequered flag sliding across it: to the right on 1P, to the left on 2P, up on Options and down on Exit. Arrows move around the menu; up or down from the banners comes back to the card last chosen. Esc or B asks whether to quit, with "Exit game" in red. There is no Exit in the browser: close the tab instead.
2. **Track select** ("SELECT TRACK"). The track is seen from above, slowly turning, with its name, a short description, its length and a map of the layout with the finish line marked. Left and right switch tracks. Enter, Space, A or Start go on to bike select. The screen shows up at once: the other tracks are built in the background meanwhile, so switching to them is instant once they are ready (a track chosen before it is ready is finished on the spot).
3. **Bike select** ("SELECT BIKE"). The bikes wait on the starting grid, three per row and staggered like a Grand Prix grid. The camera circles the chosen bike, and its name and colours are shown on screen. Left and right change bike. The right stick turns the view around the bike. Enter, Space, A or Start begin the race. The chosen bike idles here and revs each time you switch to it.
4. **Race.** Your bike starts last, so there is always someone to overtake. The other seven bikes are your rivals. They wait on the grid until you first open the throttle.

**Esc** or **B** goes back one screen: from bike select to track select, and from track select to the welcome screen. During the race, Esc pauses (see [Pause menu](#pause-menu)), and the pause menu can take you back to the welcome screen.

Menus always work the same way, whatever controls you have set:

- Up and down (arrows, W and S, D-pad or left stick) move between items.
- Left and right change a value.
- Enter, Space or A accept.
- Esc or B go back.
- In Options, Tab (Shift + Tab backwards), Q and E, or the shoulder buttons and triggers change section.

### Controls legend

Every screen has the same legend at the bottom, with no panel behind it: each key or button, as it looks on the device you are using, and what it does there. It follows the last thing you touched: keyboard keycaps, or the buttons of an Xbox, PlayStation (white, with the symbols in their classic colours) or Switch controller, told apart by the controller's name. Moving around a menu goes without saying, so the legend only shows accepting, going back and, in Options, changing section. The symbols the UI font lacks (arrows and the PlayStation symbols) are drawn as icons.

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
  - On release, the steering returns to centre more gently: slowly at first, then faster. The bike is straight again in about 0.7 s (0.6 s with the fast return rate).
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
  - When a rider goes down, bike and rider separate. The bike drops onto its side and slides with the velocity it had after the hit. The rider keeps the momentum they had before it, so they are thrown on past the bike, land, tumble and slide on their back until friction stops them. Their arms and legs move as in real racing crashes: thrown, the arms let go loosely out to the sides and the legs come apart; tumbling, they go limp and bent (face down, knees to the ground and feet up); sliding on the back, feet first, the legs stay together and loosely bent and the hands are held up over the chest, off the asphalt, while the legs drag behind; once still, the arms rest by the sides. Off the bike the hands hang relaxed from the forearms instead of gripping. Hands and feet hang loosely from that pose, like on damped springs (the legs looser than the arms, each knee swaying on its own): they lag behind when the body is stopped, flop when it lands and fling out when it spins, without shaking, and never go under the asphalt (riders slide on asphalt with a drag factor of about 0.6, a bike on its side about 0.45, more on grass). Walls stop both.
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

### Rivals

Rivals keep to their own lane. They pull out to pass a bike they catch, or lift off if there is no room. They brake before corners and accelerate out of them. The most skilled rival starts on pole. On Loop Island two of them take the branch and go round the loop, flat out, steering to keep to their line.

## Race HUD

| Element | Where | What it shows |
|---|---|---|
| Speedometer | Bottom right | Speed in km/h. |
| FPS counter | Top right | Frames per second, smoothed so it can be read. |
| Minimap | Bottom left | The layout seen from above (north up), with the finish line in yellow and one dot per bike in that bike's colour. Your dot is larger, with a white border; with two players, each one's dot carries their number (1 or 2). |
| Controls legend | Bottom centre | The keys or buttons currently bound to each action, on the device in use (see [Controls legend](#controls-legend)). It is hidden by default; turn it on in **Options > Control**. It gives way to the pause menu's own legend. |

The chase camera sits low behind the bike, as in *Moto Racer*. Its field of view widens with speed. On a gamepad, the right stick turns the camera relative to the bike:

- up looks ahead (the default);
- left or right looks to that side;
- down looks back, with the camera in front of the bike.

Release the stick and the camera looks ahead again.

## Controls

These are the defaults. Every action except the sticks, mute and fullscreen can be changed in **Options > Controls** (see [Rebinding](#rebinding)). Gamepad buttons are named as on an Xbox controller: A is the bottom button, B the right one and X the left one.

| Action | Keyboard | Gamepad |
|---|---|---|
| Throttle | W / ↑ | Right trigger (RT) / A |
| Brake, and reverse when stopped | S / ↓ | Left trigger (LT) / X |
| Steer | A, D / ←, → | Left stick (progressive) / D-pad |
| Wheelie (with throttle) | Space / Left Shift | B, or RT and A together |
| Look around | | Right stick |
| Restart the race | R | Select |
| Pause | Esc | Start |
| Mute or unmute sound | M | |
| Toggle fullscreen | Cmd+Enter (macOS) / Alt+Enter (Windows, Linux) | |

The triggers are analogue: throttle and brake follow how far you squeeze them. The first 15 % of their travel does nothing, so a trigger that does not quite return to zero at rest (as a DualSense can, with its adaptive resistance on) neither opens the throttle nor stops the engine braking. Restart sends every bike back to its grid slot, and the rivals wait again until you accelerate. In the browser, both fullscreen shortcuts work, because the game cannot tell which system the browser runs on.

## Options

"Options", on the welcome screen and in the pause menu, has four sections. Changes apply immediately. The panel is centred above the legend and is as tall in every section, so the section tabs do not move.

- **Game.** The language (Spanish or English) and the steering variants:

  | Option | Values | Default |
  |---|---|---|
  | "Stick: turn limit with speed" | off, low, medium, high | medium |
  | "Keyboard: turn limit with speed" | off, low, medium, high | medium |
  | "Keyboard: sustained turn rate" | slow, medium, fast | medium |
  | "Keyboard: return rate released" | slow, medium, fast | medium |

  The turn-limit levels apply a share of the speed-dependent limit described under [Steering](#steering): off is no limit, low a third, medium two thirds and high the whole limit (at top speed, holding the input gives 80 % of full lock). The two rates scale how fast keyboard and D-pad steering builds up while a key is held, and how fast it returns to centre once released: slow is half as fast as fast, medium three quarters.
- **Control.** One row per action (see [Rebinding](#rebinding)), "Reset controls", the race's "Controls legend" (shown or hidden, hidden by default) and "Rescan gamepads".
- **Sound.** "FX Volume" and "Music Volume", from 0 to 100 % in steps of 10 (80 % by default), and the music on or off. 80 % is the balance found by ear: the effects at a fifth and the music at nine tenths of the scale the game used before. Volumes saved with that older scale are not carried over: they start again at 80 %.
- **Graphics.**
  - **Fullscreen.** At any time, Cmd+Enter on macOS or Alt+Enter on Windows and Linux switches to fullscreen and back. The change is saved as if you had picked it here.
  - **Resolution.** "Dynamic" (the window adapts to the window and screen pixels; the default), a fixed 1280 × 720 window, 1600 × 900, 1920 × 1080 or 2560 × 1440. Sizes are in points, so on a Retina display the window has twice as many pixels. In the browser the game fills the page.
  - **FPS.** The screen's rate (synchronised with it; the default), 30, 60, 120, 144, 240 or unlimited. With a fixed number, the game waits out the rest of each frame and stays synchronised with the screen so the image does not tear. If the screen is slower than the limit, the screen wins. This option is not available in the browser, which paces the frames itself.

### Rebinding

The Control section has one row per action, each listing its keys and buttons: Throttle, Brake / Reverse, Steer Left, Steer Right, Wheelie, Restart and Pause.

To rebind an action:

1. Select its row and press Enter or A.
2. Press a key or a gamepad button. That key or button now performs the action:
   - A key replaces the action's keys; a button replaces its buttons. The other kind is left as it was.
   - The new key or button stops doing any other action.

Esc cancels, and so does waiting eight seconds without pressing anything. M cannot be bound, because it always toggles sound. "Reset controls" restores the defaults.

Some controls cannot be changed: the left stick (steering), the right stick (look around), M (sound) and the menu navigation. Pulling the left stick back does nothing: it used to pop a wheelie, which was easy to do by mistake while steering. Controls saved without a wheelie button get B, unless B already does something else. The race's controls legend always shows the keys you have chosen.

### Where settings are saved

Settings (language, fullscreen, resolution, FPS limit, controls and volumes) are saved each time they change, in `ajustes.ron` ("settings") inside the system's settings folder:

| System | File |
|---|---|
| macOS | `~/Library/Application Support/motorust/ajustes.ron` |
| Linux | `~/.config/motorust/ajustes.ron` (or under `$XDG_CONFIG_HOME` if set) |
| Windows | `%APPDATA%\motorust\ajustes.ron` |
| Browser | `localStorage`, under the key `motorust.ajustes` |

They are read at startup. If the file is missing or cannot be parsed, the game uses the defaults and prints the reason in the terminal. The steering variants and the race legend are not saved: they go back to their defaults each time the game starts.

## Pause menu

Esc or Start (or whatever you bound to Pause) stops the race and muffles its sound. The "PAUSE" menu offers:

- "Resume".
- "Options": the same [Options](#options) as on the welcome screen; closing them comes back here.
- "Return to main menu": returns to the welcome screen.
- "Exit game", which asks first, with "Exit game" in red. This item is not shown in the browser.

Up and down select, Enter or A accept, and Esc or B go back. From the main page, going back resumes the race.

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

Every sound effect is synthesised in real time, with no recordings. The synthesiser runs on the audio thread (`src/sound/synth.rs`). The game only decides what plays, how loud, and from where (`src/sound/mod.rs`). The only recordings are the tracks' music.

- **Engines.** Each bike has a 1000 cc Grand Prix V4 that revs to 17,500 rpm. It has a "big bang" firing order: two pairs of explosions close together give it a deep, rough roar.
- **Gearbox.** A six-speed straight-cut gearbox that whines. You hear:
  - the revs climbing and near-instant upshifts;
  - automatic throttle blips on downshifts;
  - the rev limiter;
  - the clutch slipping as you pull away;
  - pops and crackles when you lift off.
- **Rivals.** Rival engines are louder the closer they are, panned left or right. They sound higher-pitched approaching than moving away (the Doppler effect), so you hear them come and go.
- **Surroundings.** Wind that grows with speed; tyres when you push in corners or brake hard; grass; impacts against walls; landings.
- **Two players.** Both players' bikes sound as yours: full volume, no Doppler. Every bike, rivals included, is heard from the nearer of the two cameras, and the wind, tyres and grass follow whichever player makes them louder.
- **Menus.** Moving through menus clicks. On the bike select screen, the chosen bike idles and revs when you switch to it.
- **Music.** Each track has its own song (`assets/music/`), which loops during the race (`src/sound/music.rs`). It plays well below the bike: every song is brought to the same loudness, about 13 dB under your engine flat out (and still under it off the throttle), so the revs, gear changes, tyres and impacts are always heard over it. It fades in when the race starts and out when you leave it.

Pausing muffles the race and turns the music down. **M** mutes and unmutes everything, music included. In the browser, sound starts after the first key press or click, because browsers do not allow audio before that.
