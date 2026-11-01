#!/usr/bin/env python3
"""Prepara los modelos 3D de Meshy para el juego.

Lee los GLB originales de una moto y su piloto (en las releases del
repositorio) y escribe en `assets/models/NOMBRE/` lo que carga el juego:

- `bike.bin`: la moto separada en cuatro piezas (chasis, dirección, rueda
  delantera y rueda trasera), ya en los ejes y la escala del juego, más las
  bocas de los escapes que se le abren en las tapas de los silenciadores; y sus
  texturas reducidas (`base_color.jpg`, `metallic_roughness.jpg`, `normal.jpg`).
- `pilot.bin`: el piloto con sus pesos y su esqueleto (postura de reposo de
  cada hueso), girado para mirar hacia -Z, y su textura reducida
  (`pilot_color.jpg`).
- `model.ron`: its measures (axles, wheels, steering head, grips, pegs and
  seat), whether its fork and bars turn with the steering, and its slicks
  (`Medium`; change them by hand, and keep the comment at its top, if the
  file was there: it is written anew). The roster names the bike by its
  folder, `design: Model("NOMBRE")` (`assets/bikes/roster.ron`); list its
  files in `assets/pack.txt` too.

El GLB de la moto es una sola malla hecha de entre noventa y doscientas
piezas sueltas. Se clasifican sin cortar triángulos: los neumáticos y las que
están centradas en un eje y caben en su rueda giran con ella; las que rodean el eje de la horquilla (barras,
tijas, pinzas, guardabarros y semimanillares) giran con la dirección; el resto
va con el chasis.

Uso: python3 tools/import_models.py NOMBRE MOTO.glb PILOTO.glb [CARPETA]
Necesita numpy y Pillow. Al acabar imprime las medidas de la moto (ejes,
ruedas, dirección, puños, asiento y estriberas), as it writes them in
`model.ron`, read by `src/bike/model/imported.rs`.

Formato de los .bin (little endian): b"MRMESH01", u32 número de piezas y, por
pieza, u32 largo del nombre, nombre, u32 vértices, u32 índices, u32 banderas
(1 = lleva huesos), posiciones f32×3, normales f32×3, uv f32×2, [huesos u16×4,
pesos f32×4], índices u32. Si hay esqueleto, después: b"MRSKEL01", u32 número
de huesos y, por hueso, u32 largo del nombre, nombre, i32 padre (-1 en la
raíz), posición f32×3 y giro f32×4 (x, y, z, w) en reposo.
"""

import io
import json
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "models"

# Medidas del juego (src/bike/model/mod.rs): la moto se escala para que su
# batalla sea esta y se coloca con la rueda delantera en este Z.
WHEELBASE = 1.38
FRONT_AXLE_Z = -0.70
# El piloto de Meshy mide 1,64 m: se agranda hasta 1,75 m.
PILOT_SCALE = 1.07
# Lanzamiento de la horquilla (grados desde la vertical) y distancia del eje
# delantero a la pipa (m) en las motos con la horquilla soldada al carenado,
# que no se puede separar: medias de las demás motos.
RAKE = 24.0
AXLE_TO_HEAD = 0.66
# Medidas del piloto sobre la moto, sacadas de la Shogun TZ: la cadera va por
# encima del punto más bajo del asiento; los tobillos, a esta distancia de la
# cadera (a la derecha, abajo y detrás), y las manos, algo hacia dentro del
# centro de los puños.
HIPS_ABOVE_SEAT = 0.125
ANKLE_FROM_HIPS = (0.16, -0.55, 0.07)
HAND_INSET = 0.035

# Silenciadores: piezas del chasis alargadas hacia atrás (largo en m), más o
# menos redondas (radio en m) y a un lado, con una tapa al final de al menos
# este área (m²) que no esté junto al eje trasero (así no se confunden con
# basculantes, subchasis ni bielas). Los que van bajo el colín, como el de la
# Cobra XT, están en el centro, por encima de esta altura (m) y al final de la
# moto, y son más cortos y estrechos que el colín.
SILENCER_LENGTH = (0.3, 1.0)
SILENCER_RADIUS = (0.05, 0.14)
TAIL_SILENCER_LENGTH = (0.2, 0.6)
TAIL_SILENCER_RADIUS = (0.04, 0.07)
TAIL_SILENCER_HEIGHT = 0.7
SILENCER_CAP_AREA = 0.0015
SILENCER_CAP_FROM_AXLE = 0.15
# Boca del escape, como en las fotos de silenciadores de carreras: un hueco
# negro con la forma de la tapa que ocupa algo más de la mitad de ella,
# rodeado por el labio del tubo interior, que asoma un poco. Grosor del labio,
# lo que asoma sobre la tapa y lo que se mete por dentro para que no quede
# hueco entre los dos (m).
OUTLET_SCALE = 0.6
OUTLET_WALL = 0.005
OUTLET_LIP = 0.012
OUTLET_BURY = 0.01
# El fondo va justo por encima de la tapa (m), en anillos que la siguen; y
# puntos del contorno.
OUTLET_FLOOR = 0.002
OUTLET_FLOOR_RINGS = 4
OUTLET_POINTS = 32

COMPONENT_TYPES = {5126: np.float32, 5125: np.uint32, 5123: np.uint16, 5121: np.uint8}
ITEM_SIZES = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


class Glb:
    """Lectura mínima de un GLB."""

    def __init__(self, path):
        data = Path(path).read_bytes()
        json_length, _ = struct.unpack_from("<II", data, 12)
        self.json = json.loads(data[20 : 20 + json_length])
        offset = 20 + json_length
        bin_length, _ = struct.unpack_from("<II", data, offset)
        self.binary = data[offset + 8 : offset + 8 + bin_length]

    def accessor(self, index):
        accessor = self.json["accessors"][index]
        view = self.json["bufferViews"][accessor["bufferView"]]
        dtype = np.dtype(COMPONENT_TYPES[accessor["componentType"]])
        size = ITEM_SIZES[accessor["type"]]
        start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        count = accessor["count"]
        stride = view.get("byteStride", dtype.itemsize * size)
        raw = np.frombuffer(self.binary, dtype=np.uint8, count=stride * (count - 1) + dtype.itemsize * size, offset=start)
        rows = np.lib.stride_tricks.as_strided(raw, shape=(count, dtype.itemsize * size), strides=(stride, 1))
        values = np.frombuffer(rows.copy().tobytes(), dtype=dtype).reshape(count, size)
        if accessor.get("normalized") and dtype.kind == "u":
            values = values / np.iinfo(dtype).max
        return values

    def primitive(self):
        primitive = self.json["meshes"][0]["primitives"][0]
        attributes = {name: self.accessor(index) for name, index in primitive["attributes"].items()}
        indices = self.accessor(primitive["indices"]).reshape(-1, 3).astype(np.int64)
        return attributes, indices, primitive.get("material")

    def image(self, texture_index):
        source = self.json["textures"][texture_index]["source"]
        view = self.json["bufferViews"][self.json["images"][source]["bufferView"]]
        start = view.get("byteOffset", 0)
        return Image.open(io.BytesIO(self.binary[start : start + view["byteLength"]]))


def write_mesh_pack(path, parts, skeleton=None):
    out = bytearray(b"MRMESH01")
    out += struct.pack("<I", len(parts))
    for part in parts:
        name = part["name"].encode()
        has_joints = "joints" in part
        out += struct.pack("<I", len(name)) + name
        out += struct.pack("<III", len(part["pos"]), part["idx"].size, 1 if has_joints else 0)
        out += part["pos"].astype("<f4").tobytes()
        out += part["nor"].astype("<f4").tobytes()
        out += part["uv"].astype("<f4").tobytes()
        if has_joints:
            out += part["joints"].astype("<u2").tobytes()
            out += part["weights"].astype("<f4").tobytes()
        out += part["idx"].astype("<u4").tobytes()
    if skeleton:
        out += b"MRSKEL01" + struct.pack("<I", len(skeleton))
        for joint in skeleton:
            name = joint["name"].encode()
            out += struct.pack("<I", len(name)) + name
            out += struct.pack("<i", joint["parent"])
            out += np.asarray(joint["position"], dtype="<f4").tobytes()
            out += np.asarray(joint["rotation"], dtype="<f4").tobytes()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(bytes(out))


def save_texture(image, path, size, quality=90):
    image = image.convert("RGB")
    if max(image.size) > size:
        image = image.resize((size, size), Image.LANCZOS)
    image.save(path, quality=quality)


def components(positions, triangles):
    """Piezas sueltas: triángulos conectados, soldando vértices que coinciden."""
    _, weld = np.unique(np.round(positions, 5), axis=0, return_inverse=True)
    weld = weld.ravel()
    parent = np.arange(weld.max() + 1)

    def find(a):
        root = a
        while parent[root] != root:
            root = parent[root]
        while parent[a] != root:
            parent[a], a = root, parent[a]
        return root

    for a, b, c in weld[triangles]:
        ra = find(a)
        parent[find(b)] = ra
        parent[find(c)] = ra
    return np.array([find(weld[t[0]]) for t in triangles])


def top_profile(pos, triangles, zs):
    """Altura de la parte de arriba de una malla en el centro (x = 0), en cada z."""
    a, b, c = (pos[triangles[:, k]][:, [0, 2]] for k in range(3))
    ya, yb, yc = (pos[triangles[:, k], 1] for k in range(3))
    d = (b[:, 1] - c[:, 1]) * (a[:, 0] - c[:, 0]) + (c[:, 0] - b[:, 0]) * (a[:, 1] - c[:, 1])
    d = np.where(np.abs(d) > 1e-12, d, np.nan)
    tops = []
    for z in zs:
        l1 = ((b[:, 1] - c[:, 1]) * (0 - c[:, 0]) + (c[:, 0] - b[:, 0]) * (z - c[:, 1])) / d
        l2 = ((c[:, 1] - a[:, 1]) * (0 - c[:, 0]) + (a[:, 0] - c[:, 0]) * (z - c[:, 1])) / d
        l3 = 1 - l1 - l2
        inside = (l1 >= 0) & (l2 >= 0) & (l3 >= 0)
        tops.append((l1 * ya + l2 * yb + l3 * yc)[inside].max() if inside.any() else -np.inf)
    return np.array(tops)


def convex_hull(points):
    """Envolvente convexa de puntos 2D, en sentido antihorario (cadena monótona)."""
    points = sorted(map(tuple, np.round(points, 6)))

    def turn(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    def half(sequence):
        chain = []
        for p in sequence:
            while len(chain) >= 2 and turn(chain[-2], chain[-1], p) <= 0:
                chain.pop()
            chain.append(p)
        return chain[:-1]

    return np.array(half(points) + half(points[::-1]))


def rounded_outline(polygon, count):
    """Contorno cerrado suavizado (dos pasadas de Chaikin) con `count` puntos a la misma distancia."""
    for _ in range(2):
        following = np.roll(polygon, -1, axis=0)
        polygon = np.stack([0.75 * polygon + 0.25 * following, 0.25 * polygon + 0.75 * following], axis=1).reshape(-1, 2)
    closed = np.vstack([polygon, polygon[:1]])
    lengths = np.linalg.norm(np.diff(closed, axis=0), axis=1)
    along = np.concatenate([[0.0], np.cumsum(lengths)])
    wanted = np.arange(count) * along[-1] / count
    return np.stack([np.interp(wanted, along, closed[:, k]) for k in range(2)], axis=-1)


def is_silencer(points, length, radius, rear_axle):
    """¿Tiene una pieza el sitio y la forma de un silenciador, a un lado o bajo el colín?"""
    centre = points.mean(0)
    beside = (
        abs(centre[0]) > 0.05
        and points[:, 1].min() < 0.75
        and SILENCER_LENGTH[0] < length < SILENCER_LENGTH[1]
        and SILENCER_RADIUS[0] < radius < SILENCER_RADIUS[1]
    )
    under_tail = (
        abs(centre[0]) < 0.05
        and points[:, 1].min() > TAIL_SILENCER_HEIGHT
        and points[:, 2].max() > rear_axle[2] + 0.1
        and TAIL_SILENCER_LENGTH[0] < length < TAIL_SILENCER_LENGTH[1]
        and TAIL_SILENCER_RADIUS[0] < radius < TAIL_SILENCER_RADIUS[1]
    )
    return beside or under_tail


def silencer_outlets(chassis, rear_axle):
    """Tapas de los silenciadores del chasis, ya en los ejes del juego.

    De cada una: su centro y la dirección hacia la que mira, dos ejes sobre
    ella, su contorno visto de frente (la envolvente de sus vértices, en esos
    ejes) y los triángulos del extremo del silenciador, para saber a qué
    altura está su superficie en cada punto.
    """
    pos, triangles = chassis["pos"], chassis["idx"]
    piece = components(pos, triangles)
    outlets = []
    for p in np.unique(piece):
        tris = triangles[piece == p]
        v = pos[np.unique(tris.ravel())]
        centre = v.mean(0)
        _, spread, axes = np.linalg.svd(v - centre, full_matrices=False)
        axis = axes[0] if axes[0][2] > 0 else -axes[0]
        along = (v - centre) @ axis
        length = along.max() - along.min()
        radial = np.linalg.norm((v - centre) - np.outer(along, axis), axis=1)
        if not (
            v[:, 2].max() > 0.1
            and abs(axis[2]) > 0.6
            and spread[1] < 2 * spread[2]
            and is_silencer(v, length, np.percentile(radial, 90), rear_axle)
        ):
            continue
        a, b, c = (pos[tris[:, k]] for k in range(3))
        normals = np.cross(b - a, c - a)
        areas = np.linalg.norm(normals, axis=1) / 2
        normals /= np.maximum(2 * areas[:, None], 1e-12)
        middles = (a + b + c) / 3
        end = (middles - centre) @ axis > along.max() - 0.15 * length
        cap = end & (normals @ axis > 0.7)
        area = areas[cap].sum()
        if area < SILENCER_CAP_AREA:
            continue
        point = (middles[cap] * areas[cap, None]).sum(0) / area
        if np.linalg.norm(point[1:] - rear_axle[1:]) < SILENCER_CAP_FROM_AXLE:
            continue
        normal = (normals[cap] * areas[cap, None]).sum(0)
        normal /= np.linalg.norm(normal)
        u = np.cross([0.0, 1.0, 0.0], normal)
        u /= np.linalg.norm(u)
        w = np.cross(normal, u)
        corners = np.unique(tris[cap].ravel())
        hull = convex_hull((pos[corners] - point) @ np.stack([u, w], axis=1))
        surface = pos[tris[end & (normals @ normal > 0.2)]]
        outlets.append({"point": point, "normal": normal, "u": u, "w": w, "hull": hull, "surface": surface})
    return outlets


def surface_height(outlet, flat):
    """Altura de la tapa sobre su centro (a lo largo de su normal) en cada punto 2D del contorno."""
    frame = np.stack([outlet["u"], outlet["w"]], axis=1)
    tris = outlet["surface"] - outlet["point"]
    a, b, c = (tris[:, k] @ frame for k in range(3))
    ha, hb, hc = (tris[:, k] @ outlet["normal"] for k in range(3))
    d = (b[:, 1] - c[:, 1]) * (a[:, 0] - c[:, 0]) + (c[:, 0] - b[:, 0]) * (a[:, 1] - c[:, 1])
    d = np.where(np.abs(d) > 1e-12, d, np.nan)
    heights = []
    for x, y in flat:
        l1 = ((b[:, 1] - c[:, 1]) * (x - c[:, 0]) + (c[:, 0] - b[:, 0]) * (y - c[:, 1])) / d
        l2 = ((c[:, 1] - a[:, 1]) * (x - c[:, 0]) + (a[:, 0] - c[:, 0]) * (y - c[:, 1])) / d
        l3 = 1 - l1 - l2
        inside = (l1 >= -1e-6) & (l2 >= -1e-6) & (l3 >= -1e-6)
        heights.append((l1 * ha + l2 * hb + l3 * hc)[inside].max() if inside.any() else np.nan)
    heights = np.array(heights)
    # Si algún punto se sale de la tapa, la altura del más alto.
    return np.where(np.isnan(heights), np.nanmax(heights) if np.isfinite(heights).any() else 0.0, heights)


def exhaust_part(outlets):
    """Las bocas de los escapes, encajadas en la tapa de cada silenciador.

    Cada una es un hueco con la forma de la tapa, rodeado por el labio del
    tubo interior, que asoma de la tapa lo mismo en todo su contorno: su canto,
    su pared de dentro y el fondo siguen la superficie de la tapa, un poco
    por encima de ella. La coordenada u de textura dice cómo de dentro está
    cada punto: 0 en el labio (metal) y hasta 1 en el centro del fondo (negro).
    """
    positions, normals, uvs, triangles = [], [], [], []

    def ring(points3, normals3, depth):
        start = sum(len(p) for p in positions)
        positions.append(points3)
        normals.append(normals3)
        depth = np.broadcast_to(depth, (len(points3),))
        uvs.append(np.stack([depth, np.zeros(len(points3))], axis=-1))
        return start

    def band(first, second, count, facing):
        for i in range(count):
            j = (i + 1) % count
            for tri in ((first + i, first + j, second + j), (first + i, second + j, second + i)):
                triangles.append((tri, facing))

    for outlet in outlets:
        point, normal, u, w, hull = outlet["point"], outlet["normal"], outlet["u"], outlet["w"], outlet["hull"]
        middle = hull.mean(0)
        hole = rounded_outline(middle + (hull - middle) * OUTLET_SCALE, OUTLET_POINTS)
        # Normal hacia fuera del contorno en cada punto (va en sentido antihorario).
        tangent = np.roll(hole, -1, axis=0) - np.roll(hole, 1, axis=0)
        outward = np.stack([tangent[:, 1], -tangent[:, 0]], axis=-1)
        outward /= np.linalg.norm(outward, axis=1, keepdims=True)
        lip = hole + outward * OUTLET_WALL

        def lift(flat, height):
            return point + flat[:, :1] * u + flat[:, 1:] * w + height[:, None] * normal

        def across(flat):
            return flat[:, :1] * u + flat[:, 1:] * w

        n = len(hole)
        up = np.tile(normal, (n, 1))
        at_lip, at_hole = surface_height(outlet, lip), surface_height(outlet, hole)
        # Pared de fuera del labio, desde dentro de la tapa hasta el canto.
        band(
            ring(lift(lip, at_lip - OUTLET_BURY), across(outward), 0.0),
            ring(lift(lip, at_lip + OUTLET_LIP), across(outward), 0.0),
            n,
            None,
        )
        # Canto del labio.
        band(ring(lift(lip, at_lip + OUTLET_LIP), up, 0.0), ring(lift(hole, at_hole + OUTLET_LIP), up, 0.0), n, normal)
        # Pared de dentro, del canto al fondo, cada vez más oscura.
        band(
            ring(lift(hole, at_hole + OUTLET_LIP), -across(outward), 0.04),
            ring(lift(hole, at_hole + OUTLET_FLOOR), -across(outward), 0.6),
            n,
            None,
        )
        # Fondo: anillos cada vez más pequeños que siguen la tapa hasta el centro.
        previous = None
        for k, scale in enumerate(np.linspace(1.0, 0.0, OUTLET_FLOOR_RINGS + 1)):
            flat = middle + (hole - middle) * scale
            depth = 0.6 + 0.4 * (1.0 - scale)
            if scale == 0.0:
                flat = middle[None]
            current = ring(lift(flat, surface_height(outlet, flat) + OUTLET_FLOOR), up[: len(flat)], depth)
            if previous is not None:
                if scale == 0.0:
                    for i in range(n):
                        triangles.append(((previous + i, previous + (i + 1) % n, current), normal))
                else:
                    band(previous, current, n, normal)
            previous = current

    if not positions:
        return None
    pos = np.concatenate(positions)
    nor = np.concatenate(normals)
    # Cada triángulo, de frente hacia donde mira su superficie.
    idx = []
    for (a, b, c), facing in triangles:
        facing = nor[[a, b, c]].mean(0) if facing is None else facing
        if np.cross(pos[b] - pos[a], pos[c] - pos[a]) @ facing < 0:
            b, c = c, b
        idx.append((a, b, c))
    return {"name": "exhaust", "pos": pos, "nor": nor, "uv": np.concatenate(uvs), "idx": np.array(idx)}


def import_bike(path, out):
    glb = Glb(path)
    attributes, triangles, _ = glb.primitive()
    pos, nor, uv = attributes["POSITION"], attributes["NORMAL"], attributes["TEXCOORD_0"]
    piece = components(pos, triangles)
    pieces = np.unique(piece)
    vertices = {p: np.unique(triangles[piece == p].ravel()) for p in pieces}

    def bbox(p):
        v = pos[vertices[p]]
        return v.min(0), v.max(0)

    # En el modelo, delante es -X, arriba +Y y la izquierda +Z. Los neumáticos
    # son las piezas grandes que tocan el suelo; su huella está aplastada, así
    # que el radio es la mayor de sus dos medidas.
    ground = pos[:, 1].min()
    tires = sorted(
        (p for p in pieces if bbox(p)[0][1] < ground + 0.02 and bbox(p)[1][0] - bbox(p)[0][0] > 0.45),
        key=lambda p: bbox(p)[0][0],
    )

    def axle(p):
        lo, hi = bbox(p)
        return (lo[:2] + hi[:2]) / 2, max(hi[0] - lo[0], hi[1] - lo[1]) / 2

    (front, front_radius), (rear, rear_radius) = axle(tires[0]), axle(tires[-1])

    # Eje de la dirección: recta que mejor pasa por las dos barras de la horquilla.
    def is_fork_tube(p):
        lo, hi = bbox(p)
        size = hi - lo
        return size[1] > 0.5 and size[2] < 0.12 and lo[0] < front[0] + 0.1 and hi[0] < front[0] + 0.45 and abs(lo[2] + hi[2]) > 0.1

    scale = WHEELBASE / (rear[0] - front[0])
    fork_tubes = [p for p in pieces if is_fork_tube(p)]
    # Si la horquilla viene soldada al carenado, no se puede separar sin
    # cortar triángulos: se estima su eje y la dirección no gira en el juego.
    steers = bool(fork_tubes)
    if steers:
        tubes = pos[np.concatenate([vertices[p] for p in fork_tubes])][:, :2]
        centre = tubes.mean(0)
        direction = np.linalg.svd(tubes - centre)[2][0]
        direction = direction if direction[1] > 0 else -direction
        head = tubes[np.argmax((tubes - centre) @ direction)]
    else:
        # Delante es -X: el eje sube hacia atrás.
        rake = np.radians(RAKE)
        centre = front
        direction = np.array([np.sin(rake), np.cos(rake)])
        head = front + direction * AXLE_TO_HEAD / scale
        print("  horquilla soldada al carenado: la dirección no gira")

    def axis_distance(points):
        d = points[:, :2] - centre
        return np.abs(d[:, 0] * direction[1] - d[:, 1] * direction[0])

    def classify(p):
        # Los neumáticos van con su rueda aunque no sean del todo redondos.
        if p == tires[0]:
            return "front_wheel"
        if p == tires[-1]:
            return "rear_wheel"
        v = pos[vertices[p]]
        lo, hi = v.min(0), v.max(0)
        middle = (lo[:2] + hi[:2]) / 2
        for name, wheel_axle, radius in (("front_wheel", front, front_radius), ("rear_wheel", rear, rear_radius)):
            distance = np.linalg.norm(v[:, :2] - wheel_axle, axis=1)
            if distance.max() <= radius * 1.04 and np.linalg.norm(middle - wheel_axle) < 0.06 and np.abs(v[:, 2]).max() < 0.16:
                return name
        from_front = np.linalg.norm(v[:, :2] - front, axis=1)
        along = (v[:, :2] - centre) @ direction
        fork = axis_distance(v).max() < 0.17 and np.abs(v[:, 2]).max() < 0.21 and along.max() < (head - centre) @ direction + 0.06
        fender = from_front.min() > front_radius - 0.03 and from_front.max() < front_radius + 0.08 and np.abs(v[:, 2]).max() < 0.15
        bars = lo[0] > head[0] - 0.16 and hi[0] < head[0] + 0.16 and lo[1] > head[1] - 0.2 and hi[1] < head[1] + 0.09
        return "steering" if fork or fender or bars else "chassis"

    kind = {p: classify(p) for p in pieces}

    # A los ejes del juego: delante -Z, la derecha +X, el suelo en Y = 0.

    def to_game(points):
        points = np.asarray(points, dtype=np.float64)
        return np.stack(
            [-points[..., 2] * scale, (points[..., 1] - ground) * scale, (points[..., 0] - front[0]) * scale + FRONT_AXLE_Z],
            axis=-1,
        )

    def flat(point2):
        return to_game([point2[0], point2[1], 0.0])

    parts = []
    for name in ("chassis", "steering", "front_wheel", "rear_wheel"):
        tris = triangles[np.isin(piece, [p for p in pieces if kind[p] == name])]
        used, remap = np.unique(tris.ravel(), return_inverse=True)
        normals = np.stack([-nor[used, 2], nor[used, 1], nor[used, 0]], axis=-1)
        parts.append({"name": name, "pos": to_game(pos[used]), "nor": normals, "uv": uv[used], "idx": remap.reshape(-1, 3)})
        print(f"  {name}: {len(tris)} triángulos")
    outlets = silencer_outlets(parts[0], flat(rear))
    for outlet in outlets:
        print(f"  boca de escape en {np.round(outlet['point'], 3).tolist()} hacia {np.round(outlet['normal'], 2).tolist()}")
    exhaust = exhaust_part(outlets)
    if exhaust is None:
        print("  sin silenciadores reconocibles: no se abren bocas de escape")
    else:
        parts.append(exhaust)
    write_mesh_pack(out / "bike.bin", parts)

    material = glb.json["materials"][0]
    pbr = material["pbrMetallicRoughness"]
    save_texture(glb.image(pbr["baseColorTexture"]["index"]), out / "base_color.jpg", 2048)
    save_texture(glb.image(pbr["metallicRoughnessTexture"]["index"]), out / "metallic_roughness.jpg", 1024)
    save_texture(glb.image(material["normalTexture"]["index"]), out / "normal.jpg", 1024)

    # Puños: el extremo de fuera de los semimanillares, a la derecha. Si la
    # dirección no gira, pueden estar soldados al carenado: se buscan cerca
    # de la pipa.
    top = flat(head)
    if steers:
        handlebar = parts[1]["pos"]
        bars = handlebar[(handlebar[:, 1] > top[1] - 0.15) & (handlebar[:, 0] > 0.15)]
    else:
        handlebar = np.concatenate([parts[0]["pos"], parts[1]["pos"]])
        near = (np.abs(handlebar[:, 1] - top[1] + 0.03) < 0.12) & (np.abs(handlebar[:, 2] - top[2]) < 0.2)
        bars = handlebar[near & (handlebar[:, 0] > 0.15)]
    end = bars[:, 0].max()
    grip = bars[(bars[:, 0] > end - 0.1) & (bars[:, 0] < end - 0.02)].mean(0) - [HAND_INSET, 0, 0]

    # Asiento: el punto más bajo de la parte de arriba del chasis entre el
    # depósito y el colín (el primero, si es plano).
    chassis = parts[0]
    zs = np.arange(0.0, 0.5, 0.025)
    tops = top_profile(chassis["pos"], chassis["idx"], zs)
    lowest = tops.min()
    seat_z = zs[np.argmax(tops <= lowest + 0.01)]
    seat = np.array([0.0, lowest + HIPS_ABOVE_SEAT, seat_z])
    peg = seat + ANKLE_FROM_HIPS

    measures = {
        "front_axle": flat(front),
        "rear_axle": flat(rear),
        "front_radius": front_radius * scale,
        "rear_radius": rear_radius * scale,
        "steering_head": top,
        "grip": grip,
        "peg": peg,
        "seat": seat,
        "steers": steers,
    }
    for name, value in measures.items():
        print(f"  {name}: {np.round(value, 4).tolist() if not isinstance(value, bool) else value}")
    return measures


def quaternion_to_matrix(q):
    x, y, z, w = q
    return np.array(
        [
            [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
            [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
            [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
        ]
    )


def matrix_to_quaternion(m):
    w = np.sqrt(max(0.0, 1 + m[0, 0] + m[1, 1] + m[2, 2])) / 2
    x = np.sqrt(max(0.0, 1 + m[0, 0] - m[1, 1] - m[2, 2])) / 2
    y = np.sqrt(max(0.0, 1 - m[0, 0] + m[1, 1] - m[2, 2])) / 2
    z = np.sqrt(max(0.0, 1 - m[0, 0] - m[1, 1] + m[2, 2])) / 2
    x = np.copysign(x, m[2, 1] - m[1, 2])
    y = np.copysign(y, m[0, 2] - m[2, 0])
    z = np.copysign(z, m[1, 0] - m[0, 1])
    q = np.array([x, y, z, w])
    return q / np.linalg.norm(q)


def import_pilot(path, out):
    glb = Glb(path)
    attributes, triangles, material = glb.primitive()
    skin = glb.json["skins"][0]
    nodes = glb.json["nodes"]
    inverse_binds = glb.accessor(skin["inverseBindMatrices"]).reshape(-1, 4, 4).transpose(0, 2, 1)

    # Mira hacia +Z: se gira media vuelta para que mire hacia -Z, como la moto.
    turn = np.diag([-1.0, 1.0, -1.0])
    pos = attributes["POSITION"] @ turn.T * PILOT_SCALE
    nor = attributes["NORMAL"] @ turn.T
    weights = attributes["WEIGHTS_0"].astype(np.float64)
    weights /= weights.sum(1, keepdims=True)
    joints = attributes["JOINTS_0"].astype(np.uint16)

    index_of = {node: i for i, node in enumerate(skin["joints"])}
    parent_node = {child: i for i, node in enumerate(nodes) for child in node.get("children", [])}
    skeleton = []
    for i, node in enumerate(skin["joints"]):
        bind = np.linalg.inv(inverse_binds[i])
        basis = bind[:3, :3]
        basis = basis / np.linalg.norm(basis, axis=0)
        parent = parent_node.get(node)
        while parent is not None and parent not in index_of:
            parent = parent_node.get(parent)
        skeleton.append(
            {
                "name": nodes[node]["name"].replace("mixamorig:", ""),
                "parent": index_of[parent] if parent is not None else -1,
                "position": turn @ bind[:3, 3] * PILOT_SCALE,
                "rotation": matrix_to_quaternion(turn @ basis @ turn.T),
            }
        )
    part = {"name": "pilot", "pos": pos, "nor": nor, "uv": attributes["TEXCOORD_0"], "joints": joints, "weights": weights, "idx": triangles}
    write_mesh_pack(out / "pilot.bin", [part], skeleton)
    base = glb.json["materials"][material]["pbrMetallicRoughness"]["baseColorTexture"]["index"]
    save_texture(glb.image(base), out / "pilot_color.jpg", 1024)
    print(f"  piloto: {len(triangles)} triángulos, {len(skeleton)} huesos")
    for joint in skeleton:
        print(f"    {joint['name']:16s} padre {joint['parent']:3d} en {np.round(joint['position'], 3).tolist()}")


if __name__ == "__main__":
    if len(sys.argv) not in (4, 5):
        sys.exit(__doc__)
    out = Path(sys.argv[4]) if len(sys.argv) > 4 else OUT / sys.argv[1]
    out.mkdir(parents=True, exist_ok=True)
    print("Moto:")
    measures = import_bike(sys.argv[2], out)
    print("Piloto:")
    import_pilot(sys.argv[3], out)

    def vector(v):
        return "({:.4}, {:.4}, {:.4})".format(*(0.0 if abs(x) < 5e-5 else x for x in v))

    # What it wore before, if it was imported already.
    previous = out / "model.ron"
    tyres = "Medium"
    if previous.exists():
        for line in previous.read_text().splitlines():
            if line.strip().startswith("tyres:"):
                tyres = line.split(":")[1].strip().rstrip(",")
    lines = [
        f"// The {sys.argv[1]}, made with Meshy and prepared by tools/import_models.py.",
        "(",
        "    geometry: (",
    ]
    for name, value in measures.items():
        if name != "steers":
            lines.append(f"        {name}: " + (vector(value) if np.ndim(value) else f"{value:.4f}") + ",")
    lines += [
        "    ),",
        f"    steers: {str(measures['steers']).lower()},",
        f"    tyres: {tyres},",
        ")",
    ]
    previous.write_text("\n".join(lines) + "\n")
    print(f"\nMedidas, en {previous}:\n")
    print("\n".join(lines))
