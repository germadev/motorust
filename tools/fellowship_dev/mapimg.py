"""A plan of part of the lap on the probed ground: heights coloured, the
centre line with its alongs, and overlays."""
import sys
sys.path.insert(0, "tools")
from place import *
from PIL import Image, ImageDraw

def plan(x0, x1, z0, z1, scale=2.0, vmax=100.0, marks=(), circles=(), lines=(), name="map.png", every=50):
    w, h = int((x1 - x0) * scale), int((z1 - z0) * scale)
    img = Image.new("RGB", (w, h))
    px = img.load()
    for j in range(h):
        z = z1 - j / scale
        for i in range(w):
            x = x0 + i / scale
            g = ground((x, z))
            if g != g:
                px[i, j] = (0, 0, 0)
                continue
            if g < SEA + 0.3:
                px[i, j] = (60, 90, 200)
            else:
                t = max(0.0, min(1.0, g / vmax))
                band = 1.0 if int(g / 5) % 2 == 0 else 0.9
                px[i, j] = (int((70 + 185 * t) * band), int((140 + 80 * t) * band), int((60 + 160 * t) * band))
    d = ImageDraw.Draw(img)
    to = lambda p: ((p[0] - x0) * scale, (z1 - p[1]) * scale)
    pts = [to((X[k], Z[k])) for k in range(N)]
    for a, b in zip(pts, pts[1:]):
        if 0 <= a[0] < w and 0 <= a[1] < h:
            d.line([a, b], fill=(0, 0, 0), width=2)
    for a in range(0, int(LAP), every):
        p = at(a)
        q = to(p)
        if 0 <= q[0] < w and 0 <= q[1] < h:
            d.text(q, str(a), fill=(255, 255, 255))
    for (x, z, r, label) in circles:
        c = to((x, z))
        d.ellipse([c[0] - r * scale, c[1] - r * scale, c[0] + r * scale, c[1] + r * scale], outline=(255, 0, 0), width=2)
        d.text(c, str(label), fill=(255, 0, 0))
    for (x, z, label) in marks:
        c = to((x, z))
        d.ellipse([c[0] - 4, c[1] - 4, c[0] + 4, c[1] + 4], fill=(255, 0, 255))
        d.text((c[0] + 5, c[1]), str(label), fill=(255, 0, 255))
    for (pl, col) in lines:
        d.line([to(p) for p in pl], fill=col, width=2)
    img.save(S / name)
