#!/usr/bin/env python3
"""Apply the recorded geometry corrections in `assets/models/prop_fixes.json` to the fetched props.

A Poly Haven prop is fetched unchanged, and then this tool applies the rigid corrections that
`cast.py contact-items` (S36-P1, gates C-3 and C-4) showed it needs: a chair whose feet sit 5.9 mm under
its own floor, a bulb seated 4 mm into its socket, a holder passing through a glass cover. Each correction is data -- which node or primitive,
which move, and why -- so a re-fetch reproduces the corrected file, and a reviewer reads the reason next
to the number. The gate stays the judge; this tool never measures anything.

The operations, all in millimetres:

- `translate_node`: add `by_mm` to a node's translation (`"node": "*"` moves every scene root, i.e. the
  whole prop).
- `translate_primitive`: add `by_mm` to every vertex of one primitive, chosen by its node and its material
  name, in the mesh's own space.
- `cut`: remove the part of one primitive (`node`, `material`) that lies inside another (`by`: its
  `node` and `material`), for a part modelled passing through another where the real object has a
  hole or a groove. `prop_cut.py` does it with Blender (`blender` on PATH, or `MINEWORLD_BLENDER`).

Each fix has an `id`. The ids applied are recorded in the file's top-level `extras.mineworld_prop_fixes`,
and a fix already recorded is skipped, so running the tool twice changes nothing. It uses the standard
library only, like the fetch scripts that call it.

Usage: fix_props.py [--models DIR] [--check]
  --check   exit 1 if a fix is not yet applied, change nothing
"""

from __future__ import annotations

import argparse
import json
import struct
import sys
from pathlib import Path
from typing import Any, cast

import prop_cut  # the sibling module; this file runs as a script, so its directory is on sys.path

FLOAT = 5126  # glTF componentType for float32
EXTRAS_KEY = "mineworld_prop_fixes"
OPS = ("translate_node", "translate_primitive", "cut")

Json = dict[str, Any]


def _vec3(value: object, what: str) -> tuple[float, float, float]:
    if not isinstance(value, list) or len(cast(list[object], value)) != 3:
        raise ValueError(f"{what}: expected [x, y, z]")
    x, y, z = (float(cast(float, v)) for v in cast(list[object], value))
    return x, y, z


def _node(doc: Json, name: str) -> Json:
    for node in cast(list[Json], doc["nodes"]):
        if node.get("name") == name:
            return node
    raise ValueError(f"no node named {name!r}")


def _primitive(doc: Json, node_name: str, material: str) -> Json:
    node = _node(doc, node_name)
    if "mesh" not in node:
        raise ValueError(f"node {node_name!r} has no mesh")
    mesh = cast(list[Json], doc["meshes"])[int(node["mesh"])]
    materials = cast(list[Json], doc.get("materials", []))
    found = [
        p
        for p in cast(list[Json], mesh["primitives"])
        if "material" in p and materials[int(p["material"])].get("name") == material
    ]
    if len(found) != 1:
        raise ValueError(
            f"node {node_name!r}: {len(found)} primitives with material {material!r}, expected 1"
        )
    return found[0]


class _Positions:
    """One float VEC3 POSITION accessor, read from and written back to its buffer."""

    def __init__(self, doc: Json, buffers: dict[int, bytearray], accessor: int) -> None:
        users = [
            p
            for m in cast(list[Json], doc["meshes"])
            for p in cast(list[Json], m["primitives"])
            if cast(Json, p["attributes"]).get("POSITION") == accessor
        ]
        if len(users) != 1:
            raise ValueError(
                f"accessor {accessor} is shared by {len(users)} primitives; refusing to move it"
            )
        self.acc = cast(list[Json], doc["accessors"])[accessor]
        if (
            self.acc.get("componentType") != FLOAT
            or self.acc.get("type") != "VEC3"
            or "sparse" in self.acc
        ):
            raise ValueError(f"accessor {accessor} is not a dense float VEC3")
        view = cast(list[Json], doc["bufferViews"])[int(self.acc["bufferView"])]
        self.data = buffers[int(view["buffer"])]
        self.start = int(view.get("byteOffset", 0)) + int(self.acc.get("byteOffset", 0))
        self.stride = int(view.get("byteStride", 12))
        self.count = int(self.acc["count"])

    def read(self) -> list[list[float]]:
        return [
            list(struct.unpack_from("<3f", self.data, self.start + i * self.stride))
            for i in range(self.count)
        ]

    def write(self, points: list[list[float]]) -> None:
        for i, p in enumerate(points):
            struct.pack_into("<3f", self.data, self.start + i * self.stride, *p)
        # min/max are what the float32 file holds, so they are taken after the round trip.
        stored = self.read()
        self.acc["min"] = [min(p[k] for p in stored) for k in range(3)]
        self.acc["max"] = [max(p[k] for p in stored) for k in range(3)]


def _apply(doc: Json, buffers: dict[int, bytearray], fix: Json) -> None:
    op = fix.get("op")
    if op == "translate_node":
        dx, dy, dz = (v / 1000.0 for v in _vec3(fix.get("by_mm"), f"{fix['id']} by_mm"))
        name = str(fix["node"])
        if name == "*":
            scene = cast(list[Json], doc["scenes"])[int(doc.get("scene", 0))]
            targets = [
                cast(list[Json], doc["nodes"])[int(i)]
                for i in cast(list[int], scene["nodes"])
            ]
        else:
            targets = [_node(doc, name)]
        for node in targets:
            if "matrix" in node:
                raise ValueError(
                    f"{fix['id']}: node {node.get('name')!r} uses a matrix; not supported"
                )
            t = cast(list[float], node.get("translation", [0.0, 0.0, 0.0]))
            node["translation"] = [t[0] + dx, t[1] + dy, t[2] + dz]
        return
    if op == "cut":
        print(f"  {fix['id']}: {prop_cut.cut(doc, buffers, fix)}")
        return
    primitive = _primitive(doc, str(fix["node"]), str(fix["material"]))
    positions = _Positions(
        doc, buffers, int(cast(Json, primitive["attributes"])["POSITION"])
    )
    points = positions.read()
    if op == "translate_primitive":
        d = [v / 1000.0 for v in _vec3(fix.get("by_mm"), f"{fix['id']} by_mm")]
        positions.write([[p[k] + d[k] for k in range(3)] for p in points])
    else:
        raise ValueError(
            f"{fix.get('id')}: unknown op {op!r}; expected one of {list(OPS)}"
        )


def fix_prop(path: Path, fixes: list[Json], check: bool) -> list[str]:
    """Apply the fixes not yet recorded in the file; return the ids applied (or, with `check`, pending)."""
    doc = cast(Json, json.loads(path.read_text(encoding="utf-8")))
    done = set(cast(list[str], cast(Json, doc.get("extras", {})).get(EXTRAS_KEY, [])))
    pending = [f for f in fixes if f["id"] not in done]
    if check or not pending:
        return [str(f["id"]) for f in pending]
    buffers: dict[int, bytearray] = {}
    for i, buf in enumerate(cast(list[Json], doc["buffers"])):
        uri = str(buf.get("uri", ""))
        if not uri or uri.startswith("data:"):
            raise ValueError(
                f"{path}: buffer {i} is not an external .bin; not supported"
            )
        buffers[i] = bytearray((path.parent / uri).read_bytes())
    for fix in pending:
        _apply(doc, buffers, fix)
    extras = cast(Json, doc.setdefault("extras", {}))
    extras[EXTRAS_KEY] = sorted(done | {str(f["id"]) for f in pending})
    for i, buf in enumerate(cast(list[Json], doc["buffers"])):
        (path.parent / str(buf["uri"])).write_bytes(bytes(buffers[i]))
    path.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    return [str(f["id"]) for f in pending]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Apply prop_fixes.json to the fetched props."
    )
    here = Path(__file__).resolve().parent
    parser.add_argument(
        "--models", type=Path, default=here.parent / "assets" / "models"
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    spec = cast(
        Json, json.loads((args.models / "prop_fixes.json").read_text(encoding="utf-8"))
    )
    if spec.get("format") != 1:
        raise SystemExit("prop_fixes.json: expected format 1")
    unapplied = 0
    for slug, raw in sorted(cast(Json, spec["fixes"]).items()):
        fixes = cast(list[Json], raw)
        ids = [str(f.get("id")) for f in fixes]
        if len(set(ids)) != len(ids):
            raise SystemExit(f"{slug}: duplicate fix ids {ids}")
        path = args.models / slug / f"{slug}.gltf"
        if not path.is_file():
            print(f"{slug}: not fetched, skipped")
            continue
        changed = fix_prop(path, fixes, args.check)
        if changed:
            unapplied += len(changed)
            print(
                f"{slug}: {'pending' if args.check else 'applied'} {', '.join(changed)}"
            )
    return 1 if args.check and unapplied else 0


if __name__ == "__main__":
    sys.exit(main())
