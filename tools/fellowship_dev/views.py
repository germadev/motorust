"""Render a GLB from several yaw angles (degrees) into one sheet:
python views.py model.glb out.png yaw1 yaw2 ... [--pitch 0.25]"""
import sys
from pathlib import Path
import numpy as np
from PIL import Image

src = Path(__file__).with_name("render_glb.py").read_text().split("for path in sys.argv")[0]
exec(src)

args = sys.argv[1:]
path, out = args[0], args[1]
yaws = [float(a) for a in args[2:]]
positions, normals, uvs, indices, color = read_glb(path)
tex = np.asarray(color.resize((1024, 1024))).astype(np.float64) / 255.0
W, H = 640, 420
sheet = Image.new("RGB", (W * 2, H * ((len(yaws) + 1) // 2)), (40, 40, 48))
for k, yaw in enumerate(yaws):
    img = view(positions, uvs, indices, tex, rot_x(0.2) @ rot_y(np.radians(yaw)), W, H)
    sheet.paste(img, ((k % 2) * W, (k // 2) * H))
sheet.save(out)
print(out)
