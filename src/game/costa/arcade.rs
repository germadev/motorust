//! The arcade game played on the rock over Costa Bits' tunnel (see
//! [`super::Screen`]): a vertical shoot-'em-up of our own, playing itself.
//! A ship at the foot of the screen chases the saucers, lines up under them
//! and fires, and slips aside from their bombs; a formation of saucers
//! sways from side to side, a step down at each edge, dropping bombs, each
//! bursting when it is hit; the score counts up, and once a formation is
//! gone, or has come too far down, a new one comes in at the top.
//!
//! Its state is stepped at a fixed rate ([`Game::step`]) and drawn as pixel
//! art into an image ([`Game::draw`]), its sprites alone, see-through round
//! them, so that the screen shows them on the rock as it is.

use bevy::prelude::*;

use motorust_engine::geometry::random01;

/// The screen's pixels, across and up (3 to 4, as an arcade's upright).
pub const SIZE: UVec2 = UVec2::new(96, 128);
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
const SAUCER: [&str; 6] = [
    "...mmm...",
    ".mmmmmmm.",
    "mmwmmmwmm",
    "mmmmmmmmm",
    ".r.r.r.r.",
    "m.......m",
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
/// how quickly it sways (px a step), how far it steps down at each edge, and
/// how low it may come before a new one takes its place.
const COLUMNS: usize = 5;
const ROWS: usize = 4;
const APART: Vec2 = Vec2::new(15.0, 11.0);
const SWAY: f32 = 0.35;
const STEP_DOWN: f32 = 3.0;
const LOWEST: f32 = 82.0;
/// The ship: how quickly it slides (px a step), how often it may fire
/// (steps), how quickly its shots and the saucers' bombs fly (px a step),
/// and how long a burst shows and the ship is gone when hit (steps).
const SHIP_SPEED: f32 = 1.4;
const RELOAD: u32 = 9;
const SHOT_SPEED: f32 = 3.0;
const BOMB_SPEED: f32 = 1.3;
const BURST_STEPS: u32 = 8;
const RESPAWN: u32 = 45;
/// How likely a saucer over the ship drops a bomb, a step.
const BOMB_CHANCE: f32 = 0.012;

/// A game under way.
#[derive(Component, Clone, Debug)]
pub struct Game {
    alive: [bool; COLUMNS * ROWS],
    /// The formation's top left (px), and which way it sways (±1).
    formation: Vec2,
    heading: f32,
    ship: f32,
    /// Steps till the ship may fire, and till it is back after a hit.
    reload: u32,
    down: u32,
    shots: Vec<Vec2>,
    bombs: Vec<Vec2>,
    bursts: Vec<(Vec2, u32)>,
    pub score: u32,
    /// Each step's count, for its chances.
    steps: u32,
    seed: u32,
}

impl Game {
    pub fn new(seed: u32) -> Self {
        Self {
            alive: [true; COLUMNS * ROWS],
            formation: Vec2::new(8.0, 14.0),
            heading: 1.0,
            ship: SIZE.x as f32 / 2.0,
            reload: 0,
            down: 0,
            shots: Vec::new(),
            bombs: Vec::new(),
            bursts: Vec::new(),
            score: 0,
            steps: 0,
            seed,
        }
    }

    fn chance(&self, n: u32) -> f32 {
        random01(self.seed.wrapping_mul(97) ^ self.steps.wrapping_mul(7919) ^ n)
    }

    /// Where saucer `k` is (its top left, px).
    fn saucer(&self, k: usize) -> Vec2 {
        let (col, row) = (k % COLUMNS, k / COLUMNS);
        self.formation + Vec2::new(col as f32, row as f32) * APART
    }

    /// How many saucers are left.
    #[cfg(test)]
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
        let width = SIZE.x as f32;
        let foot = SIZE.y as f32 - 12.0;
        // The formation sways, stepping down at each edge.
        let alive: Vec<usize> = (0..self.alive.len()).filter(|&k| self.alive[k]).collect();
        let (left, right) = alive.iter().fold((f32::MAX, f32::MIN), |(l, r), &k| {
            let x = self.saucer(k).x;
            (l.min(x), r.max(x + 9.0))
        });
        self.formation.x += SWAY * self.heading;
        if (self.heading > 0.0 && right >= width - 2.0) || (self.heading < 0.0 && left <= 2.0) {
            self.heading = -self.heading;
            self.formation.y += STEP_DOWN;
        }
        // A new formation once this one is gone or has come too low.
        let lowest = alive.iter().map(|&k| self.saucer(k).y).fold(0.0, f32::max);
        if alive.is_empty() || lowest > LOWEST {
            self.alive = [true; COLUMNS * ROWS];
            self.formation = Vec2::new(8.0, 14.0);
            self.heading = 1.0;
        }
        // The ship: under the nearest saucer of the lowest row it can reach,
        // aside from any bomb coming down on it.
        if self.down > 0 {
            self.down -= 1;
        } else {
            let target = alive
                .iter()
                .map(|&k| self.saucer(k) + Vec2::new(4.5, 0.0))
                .max_by(|a, b| {
                    (a.y - (a.x - self.ship).abs() * 0.3)
                        .total_cmp(&(b.y - (b.x - self.ship).abs() * 0.3))
                })
                .map_or(width / 2.0, |p| p.x);
            let threat = self
                .bombs
                .iter()
                .filter(|b| b.y > foot - 40.0 && (b.x - self.ship).abs() < 8.0)
                .map(|b| b.x)
                .next();
            let goal = match threat {
                Some(x) if x >= self.ship => self.ship - 12.0,
                Some(_) => self.ship + 12.0,
                None => target,
            };
            let goal = goal.clamp(6.0, width - 6.0);
            self.ship += (goal - self.ship).clamp(-SHIP_SPEED, SHIP_SPEED);
            if self.reload > 0 {
                self.reload -= 1;
            } else if (target - self.ship).abs() < 3.0 && threat.is_none() {
                self.shots.push(Vec2::new(self.ship, foot - 3.0));
                self.reload = RELOAD;
            }
        }
        // The shots fly up and burst the saucers they meet.
        for shot in &mut self.shots {
            shot.y -= SHOT_SPEED;
        }
        let mut spent = vec![false; self.shots.len()];
        for k in 0..self.alive.len() {
            if !self.alive[k] {
                continue;
            }
            let at = self.saucer(k);
            if let Some(n) = self
                .shots
                .iter()
                .position(|s| s.x >= at.x && s.x <= at.x + 9.0 && s.y >= at.y && s.y <= at.y + 6.0)
                && !spent[n]
            {
                spent[n] = true;
                self.alive[k] = false;
                self.bursts.push((at, BURST_STEPS));
                self.score += 10 * (ROWS - k / COLUMNS) as u32;
            }
        }
        let mut n = 0;
        self.shots.retain(|s| {
            let keep = !spent[n] && s.y > 0.0;
            n += 1;
            keep
        });
        // The saucers over the ship drop bombs now and then.
        for (i, &k) in alive.iter().enumerate() {
            let at = self.saucer(k) + Vec2::new(4.5, 6.0);
            if (at.x - self.ship).abs() < 20.0 && self.chance(i as u32) < BOMB_CHANCE {
                self.bombs.push(at);
            }
        }
        for bomb in &mut self.bombs {
            bomb.y += BOMB_SPEED;
        }
        if self.down == 0
            && let Some(n) = self
                .bombs
                .iter()
                .position(|b| b.y >= foot && b.y <= foot + 7.0 && (b.x - self.ship).abs() < 4.5)
        {
            self.bombs.remove(n);
            self.bursts
                .push((Vec2::new(self.ship - 4.5, foot), BURST_STEPS));
            self.down = RESPAWN;
        }
        self.bombs.retain(|b| b.y < SIZE.y as f32);
        for burst in &mut self.bursts {
            burst.1 -= 1;
        }
        self.bursts.retain(|b| b.1 > 0);
    }

    /// The game drawn into `pixels` (RGBA, [`SIZE`]): its sprites alone,
    /// see-through round them.
    pub fn draw(&self, pixels: &mut [u8]) {
        pixels.fill(0);
        let mut put = |x: i32, y: i32, rgba: [u8; 4]| {
            if x >= 0 && y >= 0 && x < SIZE.x as i32 && y < SIZE.y as i32 {
                let i = ((y as u32 * SIZE.x + x as u32) * 4) as usize;
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
            sprite(self.saucer(k), &SAUCER, &mut put);
        }
        for (at, _) in &self.bursts {
            sprite(*at, &BURST, &mut put);
        }
        if self.down == 0 {
            let foot = SIZE.y as f32 - 12.0;
            sprite(Vec2::new(self.ship - 4.5, foot), &SHIP, &mut put);
        }
        for shot in &self.shots {
            for dy in 0..3 {
                put(shot.x as i32, shot.y as i32 + dy, [255, 230, 60, 255]);
            }
        }
        for bomb in &self.bombs {
            put(bomb.x as i32, bomb.y as i32, [255, 60, 80, 255]);
            put(bomb.x as i32, bomb.y as i32 + 1, [255, 140, 30, 255]);
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
        let mut game = Game::new(3);
        let mut lowest_left = game.left();
        let mut waves = 0;
        for _ in 0..(RATE as usize * 180) {
            let before = game.left();
            game.step();
            if game.left() > before {
                waves += 1;
            }
            lowest_left = lowest_left.min(game.left());
            assert!((0.0..=SIZE.x as f32).contains(&game.ship()));
        }
        // In three minutes it shoots down a good many and a new formation
        // comes in.
        assert!(game.score >= 300, "{}", game.score);
        assert!(lowest_left < COLUMNS * ROWS / 2);
        assert!(waves >= 1);
        // Drawn: something solid, the rest see-through.
        let mut pixels = vec![0u8; (SIZE.x * SIZE.y * 4) as usize];
        game.draw(&mut pixels);
        let solid = pixels.chunks(4).filter(|p| p[3] == 255).count();
        assert!(solid > 100 && solid < pixels.len() / 4 / 2, "{solid}");
    }
}
