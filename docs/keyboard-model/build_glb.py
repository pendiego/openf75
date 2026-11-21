# Builds the app's keyboard model from the SolidCraft exports, so the app does no geometry work at startup.
#   f75.glb:  export of f75.sc with the "Black" row active (top legends).
#   side.glb: export with "Black (Side-Printed)" active, bodies LegendSideR1..R6 only
#             (MCP: export {format: "glb", bodies: [...]}).
# Every Caps*/Legend*/LegendSide* row group is split into one node per key, with extras {group, key}
# (key = matrix index, found from CAPS in app/src/lib/f75art.ts). Other bodies are copied as they are.
# Run from this folder, then compress (lossless enough for display, ~4x smaller):
#   python build_glb.py f75.glb side.glb f75-keys.glb
#   npx @gltf-transform/cli meshopt f75-keys.glb ../../app/src/lib/f75.glb
import json
import re
import struct
import sys

import numpy as np

MM = 19.05 / 103
caps_src = open("../../app/src/lib/f75art.ts", encoding="utf-8").read()
CAPS = [(int(m), *map(int, b)) for m, *b in re.findall(r"(\d+): \[(\d+), (\d+), (\d+), (\d+)\]", caps_src)]
assert len(CAPS) == 80, len(CAPS)
# Same rule the app used at runtime: x inside the cap (3 px slack), y within 52 px of the row centre.
BOXES = np.array([[m, (x1 - 3 - 880) * MM, (x2 + 3 - 880) * MM, (383 - (y1 + y2) / 2) * MM] for m, x1, y1, x2, y2 in CAPS])

TYPES = {5126: np.float32, 5125: np.uint32, 5123: np.uint16, 5121: np.uint8}
WIDTH = {"SCALAR": 1, "VEC3": 3}


def read(path):
    b = open(path, "rb").read()
    jl = struct.unpack("<I", b[12:16])[0]
    o = 20 + jl
    return json.loads(b[20:o]), b[o + 8 : o + 8 + struct.unpack("<I", b[o : o + 4])[0]]


def accessor(g, binary, i):
    a = g["accessors"][i]
    v = g["bufferViews"][a["bufferView"]]
    assert "byteStride" not in v or v["byteStride"] == WIDTH[a["type"]] * np.dtype(TYPES[a["componentType"]]).itemsize
    start = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    n = a["count"] * WIDTH[a["type"]]
    return np.frombuffer(binary, TYPES[a["componentType"]], n, start).reshape(a["count"], -1)


def key_of(centres):
    keys = np.full(len(centres), -1)
    for m, x1, x2, cy in BOXES:
        hit = (keys < 0) & (centres[:, 0] >= x1) & (centres[:, 0] <= x2) & (np.abs(centres[:, 1] - cy) <= 52 * MM)
        keys[hit] = m
    return keys


out = {"asset": {"version": "2.0", "generator": "docs/keyboard-model/build_glb.py"}, "scenes": [{"nodes": [0]}], "scene": 0,
       "nodes": [], "meshes": [], "materials": [], "accessors": [], "bufferViews": []}
blob = bytearray()


def add_accessor(arr, kind, target):
    arr = np.ascontiguousarray(arr)
    blob.extend(b"\0" * (-len(blob) % 4))
    out["bufferViews"].append({"buffer": 0, "byteOffset": len(blob), "byteLength": arr.nbytes, "target": target})
    blob.extend(arr.tobytes())
    acc = {"bufferView": len(out["bufferViews"]) - 1, "componentType": 5126 if arr.dtype == np.float32 else 5125,
           "count": len(arr), "type": kind}
    if kind == "VEC3" and target == 34962:
        acc["min"], acc["max"] = arr.min(0).tolist(), arr.max(0).tolist()
    out["accessors"].append(acc)
    return len(out["accessors"]) - 1


def add_node(name, pos, nor, idx, material, extras):
    prim = {"attributes": {"POSITION": add_accessor(pos, "VEC3", 34962), "NORMAL": add_accessor(nor, "VEC3", 34962)},
            "indices": add_accessor(idx.astype(np.uint32).ravel(), "SCALAR", 34963), "material": material}
    out["meshes"].append({"name": name, "primitives": [prim]})
    out["nodes"].append({"name": name, "mesh": len(out["meshes"]) - 1, "extras": extras})
    return len(out["nodes"]) - 1


root = None
children = []
for path in sys.argv[1:3]:
    g, binary = read(path)
    if root is None:
        root = {k: v for k, v in g["nodes"][0].items() if k != "children"}
    assert g["nodes"][0]["rotation"] == root["rotation"] and g["nodes"][0]["scale"] == root["scale"]
    mat_base = len(out["materials"])
    out["materials"] += g["materials"]
    for ni in g["nodes"][0]["children"]:
        node = g["nodes"][ni]
        name = node["name"]
        (prim,) = g["meshes"][node["mesh"]]["primitives"]
        pos = accessor(g, binary, prim["attributes"]["POSITION"])
        nor = accessor(g, binary, prim["attributes"]["NORMAL"])
        idx = accessor(g, binary, prim["indices"]).reshape(-1, 3) if "indices" in prim else np.arange(len(pos)).reshape(-1, 3)
        material = prim["material"] + mat_base
        if not re.match(r"Caps|Legend", name):
            children.append(add_node(name, pos, nor, idx, material, {"group": name}))
            continue
        keys = key_of(pos[idx].mean(1))
        assert (keys >= 0).all(), f"{name}: {(keys < 0).sum()} triangles outside every cap"
        for m in np.unique(keys):
            tris = idx[keys == m]
            used, remap = np.unique(tris, return_inverse=True)
            children.append(add_node(name, pos[used], nor[used], remap.reshape(-1, 3), material, {"group": name, "key": int(m)}))

out["nodes"].insert(0, {**root, "children": [c + 1 for c in children]})
blob.extend(b"\0" * (-len(blob) % 4))
out["buffers"] = [{"byteLength": len(blob)}]
js = json.dumps(out, separators=(",", ":")).encode()
js += b" " * (-len(js) % 4)
with open(sys.argv[3], "wb") as f:
    f.write(struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(blob)))
    f.write(struct.pack("<II", len(js), 0x4E4F534A) + js)
    f.write(struct.pack("<II", len(blob), 0x004E4942) + bytes(blob))
print(f"{sys.argv[3]}: {len(out['nodes']) - 1} nodes, {len(blob) / 1e6:.1f} MB")
