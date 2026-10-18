"""A model of src/track.rs: the centre line (centripetal Catmull-Rom through
the control points, sampled 512 steps a segment and resampled every 2 m),
curvature as the game measures it, the closed Hermite height profile, and
checks; plus a map drawn with Pillow."""
import math
import numpy as np
from PIL import Image, ImageDraw

SAMPLE_SPACING = 2.0
WALL = 11.5
BASE = 2.0

def cr(p0, p1, p2, p3, u):
    t0 = 0.0
    t1 = t0 + math.sqrt(np.linalg.norm(p1 - p0))
    t2 = t1 + math.sqrt(np.linalg.norm(p2 - p1))
    t3 = t2 + math.sqrt(np.linalg.norm(p3 - p2))
    t = t1 + (t2 - t1) * u
    a1 = (p0 * (t1 - t) + p1 * (t - t0)) / (t1 - t0)
    a2 = (p1 * (t2 - t) + p2 * (t - t1)) / (t2 - t1)
    a3 = (p2 * (t3 - t) + p3 * (t - t2)) / (t3 - t2)
    b1 = (a1 * (t2 - t) + a2 * (t - t0)) / (t2 - t0)
    b2 = (a2 * (t3 - t) + a3 * (t - t1)) / (t3 - t1)
    return (b1 * (t2 - t) + b2 * (t - t1)) / (t2 - t1)

def cr_vec(p0, p1, p2, p3, steps=512):
    u = np.arange(steps)[:, None] / steps
    d01, d12, d23 = (math.sqrt(np.linalg.norm(b - a)) for a, b in ((p0, p1), (p1, p2), (p2, p3)))
    t0, t1 = 0.0, d01
    t2 = t1 + d12
    t3 = t2 + d23
    t = t1 + (t2 - t1) * u
    a1 = (p0 * (t1 - t) + p1 * (t - t0)) / (t1 - t0)
    a2 = (p1 * (t2 - t) + p2 * (t - t1)) / (t2 - t1)
    a3 = (p2 * (t3 - t) + p3 * (t - t2)) / (t3 - t2)
    b1 = (a1 * (t2 - t) + a2 * (t - t0)) / (t2 - t0)
    b2 = (a2 * (t3 - t) + a3 * (t - t1)) / (t3 - t1)
    return (b1 * (t2 - t) + b2 * (t - t1)) / (t2 - t1)

def sample_spline(points):
    n = len(points)
    return np.concatenate([cr_vec(points[(i - 1) % n], points[i], points[(i + 1) % n], points[(i + 2) % n]) for i in range(n)])

def resample(points, spacing):
    n = len(points)
    lengths = np.linalg.norm(np.roll(points, -1, axis=0) - points, axis=1)
    total = lengths.sum()
    count = int(round(total / spacing))
    step = total / count
    cum = np.concatenate([[0], np.cumsum(lengths)])
    targets = np.arange(count) * step
    seg = np.searchsorted(cum, targets, side="right") - 1
    seg = np.clip(seg, 0, n - 1)
    t = np.clip((targets - cum[seg]) / lengths[seg], 0, 1)[:, None]
    return points[seg] * (1 - t) + np.roll(points, -1, axis=0)[seg] * t

def open_curve(points, before, after):
    allp = [before] + list(points) + [after]
    samples = np.concatenate([cr_vec(*allp[i:i + 4]) for i in range(len(allp) - 3)])
    samples = np.vstack([samples, allp[-2]])
    lengths = np.linalg.norm(np.diff(samples, axis=0), axis=1)
    total = lengths.sum()
    count = max(1, int(round(total / SAMPLE_SPACING)))
    step = total / count
    cum = np.concatenate([[0], np.cumsum(lengths)])
    targets = np.arange(count) * step
    seg = np.clip(np.searchsorted(cum, targets, side="right") - 1, 0, len(lengths) - 1)
    t = np.clip((targets - cum[seg]) / np.maximum(lengths[seg], 1e-6), 0, 1)[:, None]
    res = samples[seg] * (1 - t) + samples[seg + 1] * t
    return np.vstack([res, samples[-1]])

def heading_of(v):
    # The game: heading 0 is -Z, positive turning left (towards -X).
    return math.atan2(-v[0], -v[1])

def wrap(a):
    return (a + math.pi) % (2 * math.pi) - math.pi

def curvatures(line, closed=True):
    n = len(line)
    prev = np.roll(line, 1, axis=0)
    nxt = np.roll(line, -1, axis=0)
    a = prev - line * 0 ; 
    h1 = np.arctan2(-(line - prev)[:, 0], -(line - prev)[:, 1])
    h2 = np.arctan2(-(nxt - line)[:, 0], -(nxt - line)[:, 1])
    turn = (h2 - h1 + np.pi) % (2 * np.pi) - np.pi
    d = 0.5 * (np.linalg.norm(line - prev, axis=1) + np.linalg.norm(nxt - line, axis=1))
    k = turn / d
    if not closed:
        k[0] = k[-1] = 0.0
    return k

def profile(points, lap, s):
    n = len(points)
    def at(i):
        w = i % n
        laps = i // n
        return points[w][0] + laps * lap, points[w][1]
    s = s % lap
    i = max([k for k, p in enumerate(points) if p[0] <= s], default=0)
    p0, p1, p2, p3 = at(i - 1), at(i), at(i + 1), at(i + 2)
    span = p2[0] - p1[0]
    m1 = (p2[1] - p0[1]) / (p2[0] - p0[0]) * span
    m2 = (p3[1] - p1[1]) / (p3[0] - p1[0]) * span
    t = (s - p1[0]) / span
    t2, t3 = t * t, t * t * t
    return (2 * t3 - 3 * t2 + 1) * p1[1] + (t3 - 2 * t2 + t) * m1 + (-2 * t3 + 3 * t2) * p2[1] + (t3 - t2) * m2

class Lap:
    def __init__(self, control, elevation):
        self.control = [np.array(p, dtype=np.float64) for p in control]
        self.line = resample(sample_spline(self.control), SAMPLE_SPACING)
        n = len(self.line)
        seg = np.linalg.norm(np.roll(self.line, -1, axis=0) - self.line, axis=1)
        self.step = seg.sum() / n
        self.length = n * self.step
        self.n = n
        self.k = curvatures(self.line)
        self.elevation_points = elevation
        self.h = np.array([BASE + profile(elevation, self.length, i * self.step) for i in range(n)])

    def index_at(self, along):
        return int(round((along % self.length) / self.step)) % self.n

    def at(self, along):
        return self.line[self.index_at(along)]

    def left(self, i):
        t = self.line[(i + 1) % self.n] - self.line[(i - 1) % self.n]
        t = t / np.linalg.norm(t)
        return np.array([t[1], -t[0]])

    def offset(self, along, lateral):
        i = self.index_at(along)
        return self.line[i] + self.left(i) * lateral

    def grades(self):
        return (np.roll(self.h, -1) - self.h) / self.step

    def crest(self):
        span = 2 * self.step
        return -(np.roll(self.h, -2) - 2 * self.h + np.roll(self.h, 2)) / span ** 2

def smoothed_k(k, i, w=6):
    n = len(k)
    return np.mean(np.abs([k[(i + j) % n] for j in range(-w, w + 1)]))

def check(lap, tunnels=(), report=True):
    problems = []
    k = lap.k
    radius = 1 / np.maximum(np.abs(k), 1e-9)
    tight = np.where(radius < 2.5 * WALL)[0]
    if len(tight):
        problems.append(f"radius under {2.5*WALL}: {[(int(i*lap.step), round(float(radius[i]),1)) for i in tight[::5][:12]]}")
    g = lap.grades()
    steep = np.where(np.abs(g) >= 0.13)[0]
    if len(steep):
        problems.append(f"grade >= 13%: {[(int(i*lap.step), round(float(g[i])*100,1)) for i in steep[::5][:12]]}")
    # Distant stretches.
    def in_tunnel(i):
        a = i * lap.step
        return any(span_contains(t, a, lap.length) for t in tunnels)
    nb = int(8 * WALL / SAMPLE_SPACING)
    clearance = 2 * WALL + 10
    pts = lap.line
    n = lap.n
    hits = []
    for i in range(0, n):
        j = np.arange(i + nb, min(n, i + n - nb))
        if len(j) == 0:
            continue
        d = np.linalg.norm(pts[j] - pts[i], axis=1)
        close = j[d < clearance]
        for jj in close:
            lo, hi = (i, jj) if lap.h[i] < lap.h[jj] else (jj, i)
            if not (in_tunnel(lo) and lap.h[hi] - lap.h[lo] > 12):
                hits.append((int(i * lap.step), int(jj * lap.step), round(float(np.linalg.norm(pts[jj] - pts[i])), 1)))
    if hits:
        problems.append(f"stretches too close ({len(hits)}): {hits[::max(1,len(hits)//10)][:10]}")
    if report:
        print(f"lap {lap.length:.0f} m, {len(lap.control)} control points, heights {lap.h.min():.1f}..{lap.h.max():.1f} m")
        print(f"  tightest radius {radius.min():.1f} m at {int(np.argmin(radius)*lap.step)} m; steepest +{g.max()*100:.1f}% / {g.min()*100:.1f}%")
        for p in problems:
            print("  PROBLEM:", p)
    return problems

def span_contains(span, along, lap):
    a, b = span
    along %= lap
    if a <= b:
        return a <= along < b
    return along >= a or along < b

def crests_report(lap, threshold=0.002):
    c = lap.crest()
    out = []
    i = 0
    n = lap.n
    while i < n:
        if c[i] > threshold:
            j = i
            while j < n and c[j] > threshold:
                j += 1
            peak = i + int(np.argmax(c[i:j]))
            r = 1 / max(smoothed_k(lap.k, peak), 1e-6)
            out.append((int(i * lap.step), int(j * lap.step), round(float(c[peak]), 4), int(min(r, 99999))))
            i = j
        i += 1
    return out

def simulate(lap, a_lat=65.0, vmax=80.0, accel=14.5, brake=22.0, grip=1.0):
    """Rough lap time: the fastest speed each point allows in its bend, with
    acceleration (falling with speed) and braking limits, gravity on grades."""
    n = lap.n
    k = np.array([smoothed_k(lap.k, i, 3) for i in range(n)])
    limit = np.minimum(vmax, np.sqrt(a_lat * grip / np.maximum(k, 1e-6)))
    g = lap.grades()
    v = limit.copy()
    for _ in range(2):
        for i in range(2 * n):
            a, b = i % n, (i + 1) % n
            acc = accel * max(0.0, 1 - (v[a] / vmax) ** 2) - 9.81 * g[a]
            v[b] = min(v[b], math.sqrt(max(1.0, v[a] ** 2 + 2 * acc * lap.step)))
        for i in range(2 * n, 0, -1):
            a, b = i % n, (i - 1) % n
            v[b] = min(v[b], math.sqrt(v[a] ** 2 + 2 * brake * lap.step))
    times = lap.step / v
    return v, times

def draw(lap, path, spans=(), marks=(), extra_lines=(), circles=(), water=None, size=1400, labels_every=250, title=""):
    pts = lap.line
    allp = np.vstack([pts] + [l for l, _ in extra_lines]) if extra_lines else pts
    lo = allp.min(0) - 150
    hi = allp.max(0) + 150
    scale = (size - 40) / max(hi - lo)
    W = int((hi[0] - lo[0]) * scale) + 40
    H = int((hi[1] - lo[1]) * scale) + 60
    img = Image.new("RGB", (W, H), (236, 233, 222))
    d = ImageDraw.Draw(img)
    def P(p):
        return (20 + (p[0] - lo[0]) * scale, 40 + (p[1] - lo[1]) * scale)
    if water is not None:
        step = 8
        for x in np.arange(lo[0], hi[0], step):
            for z in np.arange(lo[1], hi[1], step):
                if water(np.array([x, z])) < 0:
                    a = P((x, z)); d.rectangle([a[0], a[1], a[0] + step * scale + 1, a[1] + step * scale + 1], fill=(150, 190, 225))
    for (c, r, col) in circles:
        a, b = P((c[0] - r, c[1] - r)), P((c[0] + r, c[1] + r))
        d.ellipse([a, b], outline=col, width=2)
    # grid every 200 m
    for x in np.arange(math.floor(lo[0] / 200) * 200, hi[0], 200):
        d.line([P((x, lo[1])), P((x, hi[1]))], fill=(215, 212, 200))
    for z in np.arange(math.floor(lo[1] / 200) * 200, hi[1], 200):
        d.line([P((lo[0], z)), P((hi[0], z))], fill=(215, 212, 200))
    hmin, hmax = lap.h.min(), max(lap.h.max(), lap.h.min() + 1)
    def hcol(h):
        u = (h - hmin) / (hmax - hmin)
        return (int(40 + 200 * u), int(120 - 60 * u), int(200 - 170 * u))
    w = max(3, int(2 * WALL * scale))
    for i in range(lap.n):
        a, b = pts[i], pts[(i + 1) % lap.n]
        col = hcol(lap.h[i])
        for (sa, sb, scol) in spans:
            if span_contains((sa, sb), i * lap.step, lap.length):
                col = scol
        d.line([P(a), P(b)], fill=col, width=w)
    for line, col in extra_lines:
        for i in range(len(line) - 1):
            d.line([P(line[i]), P(line[i + 1])], fill=col, width=max(2, int(2 * 8.5 * scale)))
    for a in np.arange(0, lap.length, labels_every):
        p = lap.at(a)
        q = P(p)
        d.ellipse([q[0] - 3, q[1] - 3, q[0] + 3, q[1] + 3], fill=(0, 0, 0))
        d.text((q[0] + 6, q[1] - 6), f"{int(a)}", fill=(0, 0, 0))
    for (p, text) in marks:
        q = P(p)
        d.text((q[0] + 4, q[1] + 4), text, fill=(120, 0, 0))
    d.text((10, 8), title + f"   lap {lap.length:.0f} m   grid 200 m   heights {lap.h.min():.0f}-{lap.h.max():.0f} m (blue low, red high)", fill=(0, 0, 0))
    img.save(path)

def profile_png(lap, path, spans=(), W=1600, H=420):
    img = Image.new("RGB", (W, H), (245, 243, 236))
    d = ImageDraw.Draw(img)
    hmin, hmax = min(0, lap.h.min()), lap.h.max() + 5
    def P(s, h):
        return (40 + s / lap.length * (W - 60), H - 60 - (h - hmin) / (hmax - hmin) * (H - 100))
    for (sa, sb, col, name) in spans:
        x0, x1 = P(sa, 0)[0], P(sb, 0)[0]
        if sb < sa: x1 = P(lap.length, 0)[0]
        d.rectangle([x0, 30, x1, H - 60], fill=col)
        d.text((x0 + 2, 32), name, fill=(0, 0, 0))
    for h in range(0, int(hmax) + 1, 10):
        d.line([P(0, h), P(lap.length, h)], fill=(210, 210, 210)); d.text((5, P(0, h)[1] - 6), f"{h}", fill=(0, 0, 0))
    for s in range(0, int(lap.length), 500):
        d.text((P(s, hmin)[0] - 10, H - 50), f"{s}", fill=(0, 0, 0))
    pts = [P(i * lap.step, lap.h[i]) for i in range(lap.n)]
    d.line(pts, fill=(160, 40, 20), width=2)
    g = lap.grades()
    for i in range(lap.n):
        if abs(g[i]) > 0.10:
            d.point(P(i * lap.step, hmin + 1), fill=(200, 0, 0) if g[i] > 0 else (0, 0, 200))
    img.save(path)
