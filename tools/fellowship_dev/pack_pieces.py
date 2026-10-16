"""Read a game pack (MRMESH01): list of (name, positions, uvs, indices)."""
import struct
from pathlib import Path
import numpy as np

def read_pack(path):
    data = Path(path).read_bytes()
    (count,) = struct.unpack_from("<I", data, 8)
    offset = 12
    pieces = []
    for _ in range(count):
        (length,) = struct.unpack_from("<I", data, offset); offset += 4
        name = data[offset:offset + length].decode(); offset += length
        vertices, indices, _f = struct.unpack_from("<III", data, offset); offset += 12
        p = np.frombuffer(data, "<f4", vertices * 3, offset).reshape(-1, 3).astype(np.float64); offset += vertices * 12
        offset += vertices * 12
        u = np.frombuffer(data, "<f4", vertices * 2, offset).reshape(-1, 2).astype(np.float64); offset += vertices * 8
        i = np.frombuffer(data, "<u4", indices, offset).astype(np.uint32); offset += indices * 4
        pieces.append((name, p, u, i))
    return pieces
