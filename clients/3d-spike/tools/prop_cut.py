"""The `cut` fix of `fix_props.py`: remove the part of one primitive that lies inside another.

Some props are modelled with one part passing through another where the real object has a hole or a
groove: a lamp's bulb holder through its glass cover, a pane's edge into its post, stems through the
soil. Nothing rigid separates them without opening a gap, so the hidden part is cut away instead.

This module reads both primitives out of the glTF, moves them into the scene's frame, hands them to
`blender/cut_part.py` (Blender's exact intersect plus a winding-number test; Blender is GPL, so that
script lives in the GPL directory and runs as a separate process), and writes the cut primitive back
as new accessors appended to the buffer, in its own node's frame. Everything else in the file is left
byte-for-byte as it was. Standard library only.
"""

from __future__ import annotations

import json
import math
import os
import struct
import subprocess
import tempfile
from pathlib import Path
from typing import Any, cast

Json = dict[str, Any]
Mat = list[list[float]]  # 4x4, row-major, column vectors: p' = M p

_FLOAT, _UBYTE, _USHORT, _UINT = 5126, 5121, 5123, 5125
_COMPONENTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3}
_FORMAT = {_FLOAT: "f", _UBYTE: "B", _USHORT: "H", _UINT: "I"}
_SIZE = {_FLOAT: 4, _UBYTE: 1, _USHORT: 2, _UINT: 4}
_ARRAY_BUFFER, _ELEMENT_ARRAY_BUFFER = 34962, 34963
_SCRIPT = Path(__file__).resolve().parent / "blender" / "cut_part.py"
_ATTRIBUTES = {"POSITION", "NORMAL", "TEXCOORD_0"}
# How far outside B the cut ends: far below what the eye resolves next to the part that covers it,
# and enough that the two parts no longer touch (see blender/cut_part.py).
CLEARANCE_MM = 0.3


def _read(
    doc: Json, buffers: dict[int, bytearray], index: int
) -> list[tuple[float, ...]]:
    acc = cast(list[Json], doc["accessors"])[index]
    if "sparse" in acc or "bufferView" not in acc:
        raise ValueError(
            f"accessor {index} is sparse or has no buffer view; not supported"
        )
    ctype, n = int(acc["componentType"]), _COMPONENTS[str(acc["type"])]
    view = cast(list[Json], doc["bufferViews"])[int(acc["bufferView"])]
    data = buffers[int(view["buffer"])]
    start = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", _SIZE[ctype] * n))
    fmt = "<" + _FORMAT[ctype] * n
    return [
        struct.unpack_from(fmt, data, start + i * stride)
        for i in range(int(acc["count"]))
    ]


def _append(
    doc: Json, data: bytearray, buffer: int, rows: list[tuple[float, ...]], kind: str
) -> int:
    """Append one tightly packed accessor (`kind`: VEC3, VEC2 float, or SCALAR uint32 indices)."""
    while len(data) % 4:
        data.append(0)
    ctype = _UINT if kind == "SCALAR" else _FLOAT
    n = _COMPONENTS[kind]
    offset = len(data)
    fmt = "<" + _FORMAT[ctype] * n
    for row in rows:
        data.extend(struct.pack(fmt, *(int(v) if ctype == _UINT else v for v in row)))
    views = cast(list[Json], doc["bufferViews"])
    view: Json = {
        "buffer": buffer,
        "byteOffset": offset,
        "byteLength": len(data) - offset,
    }
    view["target"] = _ELEMENT_ARRAY_BUFFER if kind == "SCALAR" else _ARRAY_BUFFER
    views.append(view)
    acc: Json = {
        "bufferView": len(views) - 1,
        "componentType": ctype,
        "count": len(rows),
        "type": kind,
    }
    if kind == "VEC3":
        acc["min"] = [min(r[k] for r in rows) for k in range(3)]
        acc["max"] = [max(r[k] for r in rows) for k in range(3)]
    accessors = cast(list[Json], doc["accessors"])
    accessors.append(acc)
    return len(accessors) - 1


def _local(node: Json) -> Mat:
    if "matrix" in node:
        m = cast(list[float], node["matrix"])  # column-major
        return [[m[c * 4 + r] for c in range(4)] for r in range(4)]
    tx, ty, tz = cast(list[float], node.get("translation", [0.0, 0.0, 0.0]))
    x, y, z, w = cast(list[float], node.get("rotation", [0.0, 0.0, 0.0, 1.0]))
    sx, sy, sz = cast(list[float], node.get("scale", [1.0, 1.0, 1.0]))
    r = [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]
    s = (sx, sy, sz)
    t = (tx, ty, tz)
    return [
        [r[i][0] * s[0], r[i][1] * s[1], r[i][2] * s[2], t[i]] for i in range(3)
    ] + [[0.0, 0.0, 0.0, 1.0]]


def _mul(a: Mat, b: Mat) -> Mat:
    return [
        [sum(a[i][k] * b[k][j] for k in range(4)) for j in range(4)] for i in range(4)
    ]


def _world(doc: Json, node_index: int) -> Mat:
    nodes = cast(list[Json], doc["nodes"])
    parent = {
        c: i
        for i, n in enumerate(nodes)
        for c in cast(list[int], n.get("children", []))
    }
    m = _local(nodes[node_index])
    while node_index in parent:
        node_index = parent[node_index]
        m = _mul(_local(nodes[node_index]), m)
    return m


def _inverse3(m: Mat) -> list[list[float]]:
    a = [row[:3] for row in m[:3]]
    det = (
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    )
    if abs(det) < 1e-12:
        raise ValueError("a node transform is singular")
    cof = [
        [
            (a[(i + 1) % 3][(j + 1) % 3] * a[(i + 2) % 3][(j + 2) % 3])
            - (a[(i + 1) % 3][(j + 2) % 3] * a[(i + 2) % 3][(j + 1) % 3])
            for j in range(3)
        ]
        for i in range(3)
    ]
    return [[cof[j][i] / det for j in range(3)] for i in range(3)]  # adjugate / det


def _point(m: Mat, p: tuple[float, ...]) -> tuple[float, float, float]:
    x, y, z = (sum(m[i][k] * p[k] for k in range(3)) + m[i][3] for i in range(3))
    return x, y, z


def _direction(
    m3: list[list[float]], v: tuple[float, ...]
) -> tuple[float, float, float]:
    x, y, z = (sum(m3[i][k] * v[k] for k in range(3)) for i in range(3))
    length = math.sqrt(x * x + y * y + z * z) or 1.0
    return x / length, y / length, z / length


def _find(doc: Json, node_name: str, material: str) -> tuple[int, Json]:
    nodes = cast(list[Json], doc["nodes"])
    materials = cast(list[Json], doc.get("materials", []))
    for i, node in enumerate(nodes):
        if node.get("name") != node_name or "mesh" not in node:
            continue
        prims = cast(
            list[Json], cast(list[Json], doc["meshes"])[int(node["mesh"])]["primitives"]
        )
        found = [
            p
            for p in prims
            if "material" in p and materials[int(p["material"])].get("name") == material
        ]
        if len(found) == 1:
            return i, found[0]
    raise ValueError(
        f"no single primitive with material {material!r} on node {node_name!r}"
    )


def _triangles(doc: Json, buffers: dict[int, bytearray], prim: Json) -> list[int]:
    if int(prim.get("mode", 4)) != 4:
        raise ValueError("only triangle primitives can be cut")
    if "indices" not in prim:
        count = int(
            cast(list[Json], doc["accessors"])[int(prim["attributes"]["POSITION"])][
                "count"
            ]
        )
        return list(range(count))
    return [int(r[0]) for r in _read(doc, buffers, int(prim["indices"]))]


def cut(doc: Json, buffers: dict[int, bytearray], fix: Json) -> str:
    """Apply one `cut` fix: {"node", "material", "by": {"node", "material"}}. Returns Blender's summary."""
    a_node, a_prim = _find(doc, str(fix["node"]), str(fix["material"]))
    by = cast(Json, fix["by"])
    b_node, b_prim = _find(doc, str(by["node"]), str(by["material"]))
    extra = sorted(set(cast(Json, a_prim["attributes"])) - _ATTRIBUTES)
    if extra or set(cast(Json, a_prim["attributes"])) != _ATTRIBUTES:
        raise ValueError(
            f"{fix['id']}: the cut primitive must have exactly {sorted(_ATTRIBUTES)}"
        )
    attrs = cast(Json, a_prim["attributes"])
    a_world, b_world = _world(doc, a_node), _world(doc, b_node)
    a_normal = [list(r) for r in zip(*_inverse3(a_world))]  # inverse transpose
    payload = {
        "a": {
            "positions": [
                c
                for p in _read(doc, buffers, int(attrs["POSITION"]))
                for c in _point(a_world, p)
            ],
            "normals": [
                c
                for n in _read(doc, buffers, int(attrs["NORMAL"]))
                for c in _direction(a_normal, n)
            ],
            "uvs": [
                c for t in _read(doc, buffers, int(attrs["TEXCOORD_0"])) for c in t
            ],
            "indices": _triangles(doc, buffers, a_prim),
        },
        "b": {
            "positions": [
                c
                for p in _read(
                    doc, buffers, int(cast(Json, b_prim["attributes"])["POSITION"])
                )
                for c in _point(b_world, p)
            ],
            "indices": _triangles(doc, buffers, b_prim),
        },
        "clearance_mm": float(fix.get("clearance_mm", CLEARANCE_MM)),
    }
    with tempfile.TemporaryDirectory() as tmp:
        src, dst = Path(tmp) / "in.json", Path(tmp) / "out.json"
        src.write_text(json.dumps(payload), encoding="utf-8")
        blender = os.environ.get("MINEWORLD_BLENDER", "blender")
        run = subprocess.run(
            [
                blender,
                "--background",
                "--factory-startup",
                "--python",
                str(_SCRIPT),
                "--",
                str(src),
                str(dst),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if run.returncode != 0 or not dst.is_file():
            raise RuntimeError(
                f"{fix['id']}: cut_part.py failed:\n{run.stdout[-2000:]}\n{run.stderr[-2000:]}"
            )
        out = cast(Json, json.loads(dst.read_text(encoding="utf-8")))
        said = "; ".join(
            line.removeprefix("cut_part: ")
            for line in run.stdout.splitlines()
            if line.startswith("cut_part: ")
        )
    if not out["indices"]:
        raise RuntimeError(f"{fix['id']}: the cut removed the whole primitive")
    back = [r + [0.0] for r in _inverse3(a_world)]
    t = [a_world[i][3] for i in range(3)]
    inv: Mat = [
        [back[i][0], back[i][1], back[i][2], -sum(back[i][k] * t[k] for k in range(3))]
        for i in range(3)
    ]
    inv.append([0.0, 0.0, 0.0, 1.0])
    pos = cast(list[float], out["positions"])
    nrm = cast(list[float], out["normals"])
    uvs = cast(list[float], out["uvs"])
    a_dir = [row[:3] for row in a_world[:3]]
    a_dir_t = [
        list(r) for r in zip(*a_dir)
    ]  # world normals back to local: transpose of the forward
    buffer = int(
        cast(list[Json], doc["bufferViews"])[
            int(
                cast(list[Json], doc["accessors"])[int(attrs["POSITION"])]["bufferView"]
            )
        ]["buffer"]
    )
    data = buffers[buffer]
    attrs["POSITION"] = _append(
        doc,
        data,
        buffer,
        [_point(inv, tuple(pos[i : i + 3])) for i in range(0, len(pos), 3)],
        "VEC3",
    )
    attrs["NORMAL"] = _append(
        doc,
        data,
        buffer,
        [_direction(a_dir_t, tuple(nrm[i : i + 3])) for i in range(0, len(nrm), 3)],
        "VEC3",
    )
    attrs["TEXCOORD_0"] = _append(
        doc, data, buffer, [(uvs[i], uvs[i + 1]) for i in range(0, len(uvs), 2)], "VEC2"
    )
    a_prim["indices"] = _append(
        doc, data, buffer, [(i,) for i in cast(list[int], out["indices"])], "SCALAR"
    )
    cast(list[Json], doc["buffers"])[buffer]["byteLength"] = len(data)
    return said
