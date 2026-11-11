#!/usr/bin/env python3
"""Cuts Costa Bits' spectators out of a sheet of pixel art and writes them as
the game shows them (`assets/models/costa/crowd.png`, see
`src/game/costa.rs`).

The sheet (made with an image generator) has the spectators in columns, one
person to a column, each row a pose of a loop of cheering, on a flat magenta
ground. Its pixels are not on a grid (each of the art's pixels some three of
the sheet's, blurred at their edges by its compression), so each figure is
keyed out of the magenta, brought down to `HEIGHT` pixels high (its art's own
pixels, roughly) and its edge cut sharp: each pixel solid or see-through.

The rows and columns are found from the gaps between the figures; a row cut
off by the sheet's edge (its figures' feet missing) is left out, as is any
row past `POSES`. Each person's poses are cut by one box round all of them,
so that they move in it as they did on the sheet (a jump rises), their feet
at the bottom of their cell.

Written as one image: a cell `CELL` pixels across and high per person and
pose, the people across, the poses down, see-through round them.

Several sheets may be given: their people one after another.

Usage: python3 tools/make_crowd.py SHEET.png [SHEET.png ...] [--out=OUT.png].
Needs numpy and Pillow.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models" / "costa" / "crowd.png"

# How many poses of each person are kept (a loop played forwards and back), and how many pixels high the
# tallest of them is made (of the sheet's some 215).
POSES = 4
HEIGHT = 64
# How far from the magenta (0..1 a channel) a pixel is still ground.
KEY = 0.33
# The least gap between figures (sheet pixels), across and down, and how many
# pixels of a line across the sheet must be figures for it to be in a row.
GAP = 4
ROW_FULL = 60


def ground(rgb):
    """Where the sheet is its magenta ground: red and blue high, green low,
    against the ground's own colour sampled at the corners."""
    corners = np.concatenate(
        [rgb[:8, :8].reshape(-1, 3), rgb[:8, -8:].reshape(-1, 3)]
    )
    key = np.median(corners, axis=0)
    distance = np.abs(rgb - key).max(axis=2)
    pink = (rgb[..., 0] > 0.6) & (rgb[..., 2] > 0.6) & (rgb[..., 1] < 0.45)
    return (distance < KEY) | pink


def runs(filled, gap):
    """The runs of `filled` (a row of booleans) apart by at least `gap`."""
    spans, start, empty = [], None, 0
    for k, full in enumerate(filled):
        if full:
            if start is None:
                start = k
            empty, end = 0, k + 1
        elif start is not None:
            empty += 1
            if empty >= gap:
                spans.append((start, end))
                start = None
    if start is not None:
        spans.append((start, end))
    return spans


def people(path):
    """The people of a sheet: each a list of their poses, RGBA, a pose's
    feet at its bottom, all of a person's poses as wide."""
    rgb = np.asarray(Image.open(path).convert("RGB")).astype(np.float64) / 255.0
    solid = ~ground(rgb)
    # The rows: where many pixels across are figures (a fist raised may
    # reach into the row above), each out to halfway to the next.
    counts = solid.sum(axis=1)
    height, width = solid.shape
    cores = [(a, b) for a, b in runs(counts > ROW_FULL, GAP) if b - a > 40]
    rows = []
    for k, (a, b) in enumerate(cores):
        top = 0 if k == 0 else (cores[k - 1][1] + a) // 2
        bottom = height if k == len(cores) - 1 else (b + cores[k + 1][0]) // 2
        rows.append((top, bottom))
    # A row cut off at the bottom of the sheet has lost its feet.
    whole = [(a, b) for a, b in rows if b < height or counts[-1] == 0]
    whole = whole[:POSES]
    # Each row down to its lowest feet.
    whole = [
        (a, a + int(np.nonzero(counts[a:b])[0].max()) + 1) for a, b in whole
    ]
    if len(whole) < POSES:
        sys.exit(f"{path}: only {len(whole)} whole rows of figures")
    columns = None
    for a, b in whole:
        found = [
            (c, d) for c, d in runs(solid[a:b].sum(axis=0) > 0, GAP) if d - c > 20
        ]
        if columns is None:
            columns = [[(c, d)] for c, d in found]
        elif len(found) == len(columns):
            for column, span in zip(columns, found):
                column.append(span)
        else:
            sys.exit(f"{path}: row {a}..{b}: {len(found)} figures, not {len(columns)}")
    tallest = max(b - a for a, b in whole)
    scale = HEIGHT / tallest
    found = []
    for spans in columns:
        # Across all their poses, so that they move in it as on the sheet.
        left = min(c for c, _ in spans)
        right = max(d for _, d in spans)
        poses = []
        for a, b in whole:
            # The same height for every pose, nothing above its row's top.
            top = b - tallest
            crop = np.zeros((tallest, right - left, 3))
            alpha = np.zeros((tallest, right - left))
            crop[a - top :] = rgb[a:b, left:right]
            alpha[a - top :] = solid[a:b, left:right]
            poses.append(bring_down(crop, alpha, scale))
        found.append(poses)
    return found


def bring_down(crop, alpha, scale):
    """A figure brought down by `scale` as light mixes (its colour weighed by
    how solid it is), each pixel then solid or see-through."""
    premultiplied = np.dstack([crop * alpha[..., None], alpha])
    w = max(1, round(crop.shape[1] * scale))
    h = max(1, round(crop.shape[0] * scale))
    small = np.stack(
        [
            np.asarray(
                Image.fromarray(premultiplied[..., k].astype(np.float32)).resize(
                    (w, h), Image.BOX
                )
            )
            for k in range(4)
        ],
        axis=-1,
    )
    keep = small[..., 3] > 0.5
    colour = small[..., :3] / np.maximum(small[..., 3:], 1e-6)
    pixels = np.zeros((h, w, 4), np.uint8)
    pixels[..., :3] = np.clip(colour * 255.0 + 0.5, 0, 255).astype(np.uint8)
    pixels[..., 3] = np.where(keep, 255, 0)
    pixels[~keep, :3] = 0
    return pixels


def main():
    sheets = [a for a in sys.argv[1:] if not a.startswith("--out=")]
    outs = [a[6:] for a in sys.argv[1:] if a.startswith("--out=")]
    if not sheets:
        sys.exit(__doc__)
    out = Path(outs[0]) if outs else OUT
    crowd = [person for sheet in sheets for person in people(sheet)]
    cell_w = max(pose.shape[1] for person in crowd for pose in person) + 2
    cell_h = max(pose.shape[0] for person in crowd for pose in person) + 2
    sheet = np.zeros((cell_h * POSES, cell_w * len(crowd), 4), np.uint8)
    for x, person in enumerate(crowd):
        for y, pixels in enumerate(person):
            h, w = pixels.shape[:2]
            # In its cell: feet on the bottom row but one, in the middle.
            x0 = x * cell_w + (cell_w - w) // 2
            y0 = y * cell_h + cell_h - 1 - h
            sheet[y0 : y0 + h, x0 : x0 + w] = pixels
    out.parent.mkdir(parents=True, exist_ok=True)
    Image.fromarray(sheet, "RGBA").save(out, optimize=True)
    print(
        f"{out}: {len(crowd)} people, {POSES} poses, cells {cell_w}x{cell_h} px "
        f"({sheet.shape[1]}x{sheet.shape[0]})"
    )


if __name__ == "__main__":
    main()
