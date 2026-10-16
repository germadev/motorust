import sys, math
import numpy as np
from PIL import Image, ImageDraw
from pack_pieces import read_pack
pieces = {n: (p, u, i) for n, p, u, i in read_pack("/home/user/motorust/assets/models/fellowship/props.bin")}
atlas = np.asarray(Image.open("/home/user/motorust/assets/models/fellowship/color.jpg").convert("RGB").resize((1024, 1024))).astype(float) / 255
def view(yaw, pitch, out, size=700):
    img = Image.new("RGB", (size, size), (120, 110, 105)); d = ImageDraw.Draw(img)
    cy, sy, cp, sp = math.cos(yaw), math.sin(yaw), math.cos(pitch), math.sin(pitch)
    tris = []
    for name, color in [("durin_doors", None), ("durin_ithildin", (225, 235, 255)), ("durin_dark", (8, 8, 10))]:
        p, u, i = pieces[name]
        t = i.reshape(-1, 3)
        x = p[:, 0] * cy + p[:, 2] * sy
        z = -p[:, 0] * sy + p[:, 2] * cy
        y = p[:, 1] * cp - z * sp
        depth = p[:, 1] * sp + z * cp
        for k in t:
            v = p[k]
            nrm = np.cross(v[1] - v[0], v[2] - v[0]); l = np.linalg.norm(nrm)
            if l == 0: continue
            nrm /= l
            if color is None:
                uv = u[k].mean(0)
                c = atlas[int(np.clip(uv[1], 0, .999) * 1024), int(np.clip(uv[0], 0, .999) * 1024)]
                shade = 0.5 + 0.5 * abs(nrm @ np.array([0.5, 0.4, 0.75]))
                col = tuple(int(255 * min(1, a * shade)) for a in c)
            else:
                col = color
            tris.append((depth[k].mean() + (0.05 if color else 0), [(size / 2 + x[j] * 15, size - 40 - y[j] * 15) for j in k], col))
    tris.sort(key=lambda t: t[0])
    for _, poly, col in tris:
        d.polygon(poly, fill=col)
    img.save(out)
view(0.0, 0.0, "../durin/front.png")
view(0.6, 0.15, "../durin/three_quarter.png")
