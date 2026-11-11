//! The arcade game played on the rock over Costa Bits' tunnel (see
//! [`super::Screen`]): a vertical shoot-'em-up of our own, playing itself,
//! as the arcades' attract mode played their games.
//!
//! A formation of saucers marches across the screen a pixel at a time, all
//! together, a step down and back at each edge, quicker as fewer are left,
//! their legs stepping as they march, and drops bombs from the lowest of a
//! column now and then. The ship at the foot plays as a player would: it
//! takes up a place, waits there for the formation to bring a saucer over
//! it and fires (one shot on the screen at a time, as such games had it),
//! and every few seconds moves on to another place, nearer where the
//! saucers are; it slips aside from a bomb coming down on it, and when one hits
//! it, it bursts and comes back a moment later. Each saucer bursts when hit
//! and the score counts up; when the formation is gone, or has come down
//! too far, a new one comes in at the top after a pause.
//!
//! Its state is stepped at a fixed rate ([`Game::step`]) and drawn as pixel
//! art into an image ([`Game::draw`]), its sprites alone, see-through round
//! them, so that the screen shows them on the rock as it is.

use bevy::prelude::*;

use motorust_engine::geometry::random01;

/// How many of the screen's pixels up (its width follows from the screen's
/// shape, [`Game::new`]).
pub const HEIGHT: u32 = 128;
/// How many steps of the game a second.
pub const RATE: f32 = 30.0;

const SHIP: [&str; 7] = [
    "....c....",
    "...ccc...",
    "...cwc...",
    "..ccccc..",
    ".crcccrc.",
    "cc.ccc.cc",
    "c...y...c",
];
/// A saucer, its legs one way and the other as it marches.
const SAUCER: [[&str; 6]; 2] = [
    [
        "...mmm...",
        ".mmmmmmm.",
        "mmwmmmwmm",
        "mmmmmmmmm",
        ".r.r.r.r.",
        "r.......r",
    ],
    [
        "...mmm...",
        ".mmmmmmm.",
        "mmwmmmwmm",
        "mmmmmmmmm",
        "r.r.r.r.r",
        ".r.....r.",
    ],
];
const BURST: [&str; 7] = [
    "y...o...y",
    ".y..o..y.",
    "..oyyyo..",
    "ooyywyyoo",
    "..oyyyo..",
    ".y..o..y.",
    "y...o...y",
];
const DIGITS: [[&str; 5]; 10] = [
    ["111", "101", "101", "101", "111"],
    ["010", "110", "010", "010", "111"],
    ["111", "001", "111", "100", "111"],
    ["111", "001", "111", "001", "111"],
    ["101", "101", "111", "001", "001"],
    ["111", "100", "111", "001", "111"],
    ["111", "100", "111", "101", "111"],
    ["111", "001", "010", "010", "010"],
    ["111", "101", "111", "101", "111"],
    ["111", "101", "111", "001", "111"],
];

fn colour(c: char) -> Option<[u8; 4]> {
    Some(match c {
        'c' => [60, 220, 255, 255],
        'w' => [255, 255, 255, 255],
        'r' => [255, 60, 80, 255],
        'y' => [255, 230, 60, 255],
        'm' => [255, 70, 200, 255],
        'o' => [255, 140, 30, 255],
        _ => return None,
    })
}

/// The formation: how many saucers across and down, how far apart (px),
/// where it comes in, how far it steps down at each edge, and how low it may
/// come before a new one takes its place.
const COLUMNS: usize = 6;
const ROWS: usize = 4;
const APART: Vec2 = Vec2::new(16.0, 12.0);
const START_Y: f32 = 14.0;
const STEP_DOWN: f32 = 4.0;
const LOWEST: f32 = HEIGHT as f32 - 40.0;
/// How many steps of the game between two of the formation's moves, a
/// pixel each: with all of it, and with one saucer left.
const MARCH: (f32, f32) = (6.0, 1.0);
/// The ship: how quickly it slides (px a step), how quickly its shot and
/// the bombs fly (px a step), how long a burst shows, the ship is gone when
/// hit, and the pause before a new formation (steps); and how often the
/// formation drops a bomb (steps, the least and the most).
const SHIP_SPEED: f32 = 1.2;
const SHOT_SPEED: f32 = 4.0;
const BOMB_SPEED: f32 = 1.5;
const BURST_STEPS: u32 = 10;
const RESPAWN: u32 = 60;
const PAUSE: u32 = 45;
const BOMBS_EVERY: (u32, u32) = (35, 90);
/// The ship's row (px from the top), and how far over it a bomb has to be,
/// and how near across, for it to slip aside.
const SHIP_Y: f32 = HEIGHT as f32 - 12.0;
/// How long the ship waits at a place before moving on (steps, the least and
/// the most).
const STAY: (u32, u32) = (60, 150);
const DODGE: (f32, f32) = (34.0, 7.0);

/// A game under way.
#[derive(Component, Clone, Debug)]
pub struct Game {
    width: u32,
    alive: [bool; COLUMNS * ROWS],
    /// The formation's top left (px), which way it marches (±1), steps till
    /// it moves again, and which way its legs are.
    formation: Vec2,
    heading: f32,
    march: f32,
    legs: usize,
    ship: f32,
    /// Where it waits, and steps till it moves on.
    place: f32,
    stay: u32,
    /// Steps till the ship is back after a hit, and till a new formation.
    down: u32,
    pause: u32,
    shot: Option<Vec2>,
    bombs: Vec<Vec2>,
    next_bomb: u32,
    bursts: Vec<(Vec2, u32)>,
    pub score: u32,
    steps: u32,
    seed: u32,
}

impl Game {
    /// A game on a screen `width` pixels across ([`HEIGHT`] up), from `seed`.
    pub fn new(width: u32, seed: u32) -> Self {
        let mut game = Self {
            width,
            alive: [true; COLUMNS * ROWS],
            formation: Vec2::ZERO,
            heading: 1.0,
            march: MARCH.0,
            legs: 0,
            ship: width as f32 / 2.0,
            place: width as f32 / 2.0,
            stay: 0,
            down: 0,
            pause: 0,
            shot: None,
            bombs: Vec::new(),
            next_bomb: BOMBS_EVERY.1,
            bursts: Vec::new(),
            score: 0,
            steps: 0,
            seed,
        };
        game.new_formation();
        game
    }

    /// How many pixels across its screen is.
    #[cfg(test)]
    pub fn width(&self) -> u32 {
        self.width
    }

    fn new_formation(&mut self) {
        self.alive = [true; COLUMNS * ROWS];
        let across = (COLUMNS - 1) as f32 * APART.x + 9.0;
        self.formation = Vec2::new((self.width as f32 - across) / 2.0, START_Y);
        self.heading = 1.0;
    }

    fn chance(&self, n: u32) -> f32 {
        random01(self.seed.wrapping_mul(97) ^ self.steps.wrapping_mul(7919) ^ n)
    }

    /// Where saucer `k` is (its top left, px).
    fn saucer(&self, k: usize) -> Vec2 {
        let (col, row) = (k % COLUMNS, k / COLUMNS);
        self.formation + Vec2::new(col as f32, row as f32) * APART
    }

    /// The lowest saucer left in column `col`, if any.
    fn lowest(&self, col: usize) -> Option<usize> {
        (0..ROWS)
            .rev()
            .map(|row| row * COLUMNS + col)
            .find(|&k| self.alive[k])
    }

    /// How many saucers are left.
    pub fn left(&self) -> usize {
        self.alive.iter().filter(|alive| **alive).count()
    }

    /// The ship's middle across (px), where it is.
    #[cfg(test)]
    pub fn ship(&self) -> f32 {
        self.ship
    }

    /// One step of the game, playing itself.
    pub fn step(&mut self) {
        self.steps = self.steps.wrapping_add(1);
        for burst in &mut self.bursts {
            burst.1 = burst.1.saturating_sub(1);
        }
        self.bursts.retain(|b| b.1 > 0);
        if self.pause > 0 {
            self.pause -= 1;
            if self.pause == 0 {
                self.new_formation();
            }
            return;
        }
        self.march_on();
        self.fly_bombs();
        self.ride();
        self.fly_shot();
        if self.left() == 0 || self.lowest_y() > LOWEST {
            // Cleared, or come down too far: a pause, and a new formation.
            if self.left() > 0 {
                self.alive = [false; COLUMNS * ROWS];
            }
            self.bombs.clear();
            self.shot = None;
            self.pause = PAUSE;
        }
    }

    fn lowest_y(&self) -> f32 {
        (0..self.alive.len())
            .filter(|&k| self.alive[k])
            .map(|k| self.saucer(k).y)
            .fold(0.0, f32::max)
    }

    /// The formation marches a pixel at a time, quicker as fewer are left,
    /// a step down and back at each edge.
    fn march_on(&mut self) {
        self.march -= 1.0;
        if self.march > 0.0 {
            return;
        }
        let share = self.left() as f32 / (COLUMNS * ROWS) as f32;
        self.march = MARCH.1 + (MARCH.0 - MARCH.1) * share;
        self.legs = 1 - self.legs;
        let (left, right) = (0..self.alive.len())
            .filter(|&k| self.alive[k])
            .map(|k| self.saucer(k).x)
            .fold((f32::MAX, f32::MIN), |(l, r), x| (l.min(x), r.max(x + 9.0)));
        let width = self.width as f32;
        if (self.heading > 0.0 && right >= width - 2.0) || (self.heading < 0.0 && left <= 2.0) {
            self.heading = -self.heading;
            self.formation.y += STEP_DOWN;
        } else {
            self.formation.x += self.heading;
        }
    }

    /// Now and then the lowest saucer of a column drops a bomb; the bombs
    /// fall, and one that meets the ship bursts it.
    fn fly_bombs(&mut self) {
        self.next_bomb = self.next_bomb.saturating_sub(1);
        if self.next_bomb == 0 {
            let columns: Vec<usize> = (0..COLUMNS).filter(|&c| self.lowest(c).is_some()).collect();
            if !columns.is_empty() {
                let col = columns[(self.chance(1) * columns.len() as f32) as usize % columns.len()];
                if let Some(k) = self.lowest(col) {
                    self.bombs.push(self.saucer(k) + Vec2::new(4.0, 7.0));
                }
            }
            let (least, most) = BOMBS_EVERY;
            self.next_bomb = least + (self.chance(2) * (most - least) as f32) as u32;
        }
        for bomb in &mut self.bombs {
            bomb.y += BOMB_SPEED;
        }
        if self.down == 0
            && let Some(n) = self
                .bombs
                .iter()
                .position(|b| b.y >= SHIP_Y && b.y <= SHIP_Y + 7.0 && (b.x - self.ship).abs() < 4.5)
        {
            self.bombs.remove(n);
            self.bursts
                .push((Vec2::new(self.ship - 4.5, SHIP_Y), BURST_STEPS));
            self.down = RESPAWN;
        }
        self.bombs.retain(|b| b.y < HEIGHT as f32);
    }

    /// The ship waits at its place, fires at a saucer coming over it, slips
    /// aside from bombs, and now and then moves on to another place.
    fn ride(&mut self) {
        if self.down > 0 {
            self.down -= 1;
            if self.down == 0 {
                self.ship = self.width as f32 / 2.0;
                self.place = self.ship;
            }
            return;
        }
        let width = self.width as f32;
        // A new place now and then: under the formation's columns that
        // are left, one of them at random, give or take.
        self.stay = self.stay.saturating_sub(1);
        if self.stay == 0 {
            let columns: Vec<f32> = (0..COLUMNS)
                .filter_map(|col| self.lowest(col).map(|k| self.saucer(k).x + 4.5))
                .collect();
            if !columns.is_empty() {
                let pick = (self.chance(3) * columns.len() as f32) as usize % columns.len();
                self.place = columns[pick] + (self.chance(4) - 0.5) * APART.x * 2.0;
            }
            let (least, most) = STAY;
            self.stay = least + (self.chance(5) * (most - least) as f32) as u32;
        }
        let threat = self
            .bombs
            .iter()
            .filter(|b| b.y > SHIP_Y - DODGE.0 && b.y < SHIP_Y && (b.x - self.ship).abs() < DODGE.1)
            .map(|b| b.x)
            .next();
        if let Some(x) = threat {
            // Aside, and wait there.
            self.place = if x >= self.ship {
                self.ship - DODGE.1 * 2.0
            } else {
                self.ship + DODGE.1 * 2.0
            };
        }
        let goal = self.place.clamp(6.0, width - 6.0);
        if (goal - self.ship).abs() > 0.5 {
            self.ship += (goal - self.ship).clamp(-SHIP_SPEED, SHIP_SPEED);
        }
        // Fire at a saucer over it, the lowest of its column.
        let over = (0..COLUMNS)
            .filter_map(|col| self.lowest(col))
            .any(|k| (self.saucer(k).x + 4.5 - self.ship).abs() < 3.5);
        if threat.is_none() && self.shot.is_none() && over {
            self.shot = Some(Vec2::new(self.ship.round(), SHIP_Y - 3.0));
        }
    }

    /// The ship's shot flies up and bursts the first saucer it meets.
    fn fly_shot(&mut self) {
        let Some(mut shot) = self.shot else {
            return;
        };
        shot.y -= SHOT_SPEED;
        let hit = (0..self.alive.len()).find(|&k| {
            let at = self.saucer(k);
            self.alive[k]
                && shot.x >= at.x
                && shot.x <= at.x + 9.0
                && shot.y >= at.y
                && shot.y <= at.y + 7.0
        });
        if let Some(k) = hit {
            self.alive[k] = false;
            self.bursts.push((self.saucer(k), BURST_STEPS));
            self.score += 10 * (ROWS - k / COLUMNS) as u32;
            self.shot = None;
        } else if shot.y < 0.0 {
            self.shot = None;
        } else {
            self.shot = Some(shot);
        }
    }

    /// The game drawn into `pixels` (RGBA, [`Self::width`] by [`HEIGHT`]):
    /// its sprites alone, see-through round them.
    pub fn draw(&self, pixels: &mut [u8]) {
        pixels.fill(0);
        let (width, height) = (self.width as i32, HEIGHT as i32);
        let mut put = |x: i32, y: i32, rgba: [u8; 4]| {
            if x >= 0 && y >= 0 && x < width && y < height {
                let i = ((y * width + x) * 4) as usize;
                pixels[i..i + 4].copy_from_slice(&rgba);
            }
        };
        let sprite = |at: Vec2, rows: &[&str], put: &mut dyn FnMut(i32, i32, [u8; 4])| {
            let (x, y) = (at.x.round() as i32, at.y.round() as i32);
            for (dy, row) in rows.iter().enumerate() {
                for (dx, c) in row.chars().enumerate() {
                    if let Some(rgba) = colour(c) {
                        put(x + dx as i32, y + dy as i32, rgba);
                    }
                }
            }
        };
        for k in (0..self.alive.len()).filter(|&k| self.alive[k]) {
            sprite(self.saucer(k), &SAUCER[self.legs], &mut put);
        }
        for (at, _) in &self.bursts {
            sprite(*at, &BURST, &mut put);
        }
        if self.down == 0 {
            sprite(Vec2::new(self.ship - 4.5, SHIP_Y), &SHIP, &mut put);
        }
        if let Some(shot) = self.shot {
            for dy in 0..3 {
                put(shot.x as i32, shot.y as i32 + dy, [255, 230, 60, 255]);
            }
        }
        for bomb in &self.bombs {
            put(bomb.x as i32, bomb.y as i32, [255, 60, 80, 255]);
            put(bomb.x as i32 + 1, bomb.y as i32 + 1, [255, 140, 30, 255]);
            put(bomb.x as i32, bomb.y as i32 + 2, [255, 60, 80, 255]);
        }
        // The score.
        let score = format!("{:06}", self.score % 1_000_000);
        for (i, d) in score.bytes().enumerate() {
            let glyph = DIGITS[(d - b'0') as usize];
            for (dy, row) in glyph.iter().enumerate() {
                for (dx, bit) in row.chars().enumerate() {
                    if bit == '1' {
                        put(
                            2 + i as i32 * 4 + dx as i32,
                            2 + dy as i32,
                            [255, 255, 255, 255],
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_game_plays_itself_scoring_and_bringing_on_new_saucers() {
        let mut game = Game::new(120, 3);
        let mut waves = 0;
        let mut turns = 0;
        let mut last_ship = game.ship();
        let mut last_way = 0.0f32;
        for _ in 0..(RATE as usize * 180) {
            let before = game.left();
            game.step();
            if game.left() > before {
                waves += 1;
            }
            assert!((0.0..=game.width() as f32).contains(&game.ship()));
            // The ship slides steadily, not back and forth from one step to
            // the next.
            let moved = game.ship() - last_ship;
            let way = if moved.abs() > 0.05 {
                moved.signum()
            } else {
                0.0
            };
            if way != 0.0 && last_way != 0.0 && way != last_way {
                turns += 1;
            }
            if way != 0.0 {
                last_way = way;
            }
            last_ship = game.ship();
        }
        // In three minutes it shoots down a good many, clearing formations.
        assert!(game.score >= 600, "{}", game.score);
        assert!(waves >= 2, "{waves}");
        assert!(turns < 300, "{turns} turns");
        // Drawn: something solid, the rest see-through.
        let mut pixels = vec![0u8; (game.width() * HEIGHT * 4) as usize];
        game.draw(&mut pixels);
        let solid = pixels.chunks(4).filter(|p| p[3] == 255).count();
        assert!(solid > 50 && solid < pixels.len() / 4 / 2, "{solid}");
    }
}
