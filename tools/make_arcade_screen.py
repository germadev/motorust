#!/usr/bin/env python3
"""Draws the frames of the arcade game projected on the rock over Costa Bits'
tunnel (`assets/models/costa/arcade_screen.jpg`, see `src/game/costa.rs`).

A vertical shoot-'em-up of our own, as the arcades' upright screens showed
them: a ship at the foot of the screen sliding from side to side and firing
up, a formation of saucers swaying across as it comes down, one of them
bursting now and then, and a score at the top. Only the sprites: no ground
behind them, see-through, so that they are drawn on the rock as it is. Pixel art, `ART` pixels a frame (3 across to 4 up),
shown `PIXEL` times larger with its pixels square; `FRAMES` frames, the
loop closing on itself, together in a grid of `GRID`.

Usage: python3 tools/make_arcade_screen.py [OUT.png]. Needs numpy and
Pillow.
"""

import math
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "costa" / "arcade_screen.png"

ART = (72, 96)
PIXEL = 4
FRAMES = 8
GRID = (4, 2)

SHIP = ["....c....", "...ccc...", "...cwc...", "..ccccc..", ".crcccrc.", "cc.ccc.cc", "c...y...c"]
INVADER = ["...mmm...", ".mmmmmmm.", "mmwmmmwmm", "mmmmmmmmm", ".r.r.r.r.", "m.......m", "........."]
BURST = ["y...o...y", ".y..o..y.", "..oyyyo..", "ooyywyyoo", "..oyyyo..", ".y..o..y.", "y...o...y"]
COLOURS = {
    "c": (60, 220, 255),
    "w": (255, 255, 255),
    "r": (255, 60, 80),
    "y": (255, 230, 60),
    "m": (255, 70, 200),
    "o": (255, 140, 30),
}
DIGITS = {
    "0": ["111", "101", "101", "101", "111"],
    "1": ["010", "110", "010", "010", "111"],
    "2": ["111", "001", "111", "100", "111"],
    "3": ["111", "001", "111", "001", "111"],
    "4": ["101", "101", "111", "001", "001"],
    "5": ["111", "100", "111", "001", "111"],
    "6": ["111", "100", "111", "101", "111"],
    "7": ["111", "001", "010", "010", "010"],
    "8": ["111", "101", "111", "101", "111"],
    "9": ["111", "101", "111", "001", "111"],
}


def sprite(art, x, y, rows):
    """`rows` drawn on `art` with their top left at (`x`, `y`)."""
    h, w = art.shape[:2]
    for dy, row in enumerate(rows):
        for dx, c in enumerate(row):
            px, py = x + dx, y + dy
            if c in COLOURS and 0 <= px < w and 0 <= py < h:
                art[py, px] = (*COLOURS[c], 255)


def frame(k):
    """Frame `k` of `FRAMES`, the loop closing on itself."""
    w, h = ART
    t = k / FRAMES
    art = np.zeros((h, w, 4), np.uint8)
    # The saucers: three rows of four, swaying across, a step down and back.
    sway = round(5 * math.sin(2 * math.pi * t))
    down = round(2 * math.sin(4 * math.pi * t))
    hit = (k * 5) % 12
    for row in range(3):
        for col in range(4):
            n = row * 4 + col
            x = 8 + col * 15 + sway
            y = 14 + row * 11 + down
            sprite(art, x, y, BURST if n == hit and k % 2 == 0 else INVADER)
    # The ship, sliding from side to side, firing up.
    ship_x = round(w / 2 - 4 + 22 * math.sin(2 * math.pi * t + 1.0))
    sprite(art, ship_x, h - 12, SHIP)
    for shot in range(3):
        y = h - 16 - ((k * 9 + shot * 24) % 60)
        for dy in range(3):
            if 0 <= y + dy < h:
                art[y + dy, ship_x + 4] = (*COLOURS["y"], 255)
    # The score, counting up.
    score = f"{1230 + 50 * k:06d}"
    for i, d in enumerate(score):
        for dy, row in enumerate(DIGITS[d]):
            for dx, bit in enumerate(row):
                if bit == "1":
                    art[2 + dy, 2 + i * 4 + dx] = (255, 255, 255, 255)
    return Image.fromarray(art, "RGBA").resize((w * PIXEL, h * PIXEL), Image.NEAREST)


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else OUT
    columns, rows = GRID
    w, h = ART[0] * PIXEL, ART[1] * PIXEL
    sheet = Image.new("RGBA", (columns * w, rows * h))
    for k in range(FRAMES):
        sheet.paste(frame(k), ((k % columns) * w, (k // columns) * h))
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out, optimize=True)
    print(f"{out}: {FRAMES} frames of {w}x{h} ({sheet.size[0]}x{sheet.size[1]})")


if __name__ == "__main__":
    main()
