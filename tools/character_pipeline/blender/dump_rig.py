# SPDX-License-Identifier: GPL-2.0-or-later
"""Armature dump for the G-5 roll check, and the optional CharMorph hip-front measurement (port of `dump_bones.py`).

Run by the driver as: blender --background --factory-startup --python-exit-code 1 --python dump_rig.py -- params.json.
Params: `glb` (the GLB whose first armature is dumped), `out_dir`, and optionally `front_y` (an object:
`glb`, `mesh`, `z`, `half_x`, `band`), which measures the minimum Blender y of that mesh's vertices with |x| < half_x
and |z - z| < band (the `front_y()` of rig_meshy.py:60-62), writes front_y.json, and is B10 step 1's measurement.

Writes dump_rig.json: {bone: {head, tail, roll, parent}}, in the rest pose, world space (the D+ dump).
"""

import bpy
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
OUT_DIR = p["out_dir"]
os.makedirs(OUT_DIR, exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=p["glb"])
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
arm.matrix_world.identity()
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="EDIT")
out = {}
for b in arm.data.edit_bones:
    out[b.name] = {
        "head": list(b.head),
        "tail": list(b.tail),
        "roll": b.roll,
        "parent": b.parent.name if b.parent else None,
    }
    print(
        f"{b.name:32s} head {tuple(round(c, 3) for c in b.head)} tail {tuple(round(c, 3) for c in b.tail)}"
        f" parent {b.parent.name if b.parent else '-'}"
    )
bpy.ops.object.mode_set(mode="OBJECT")
with open(os.path.join(OUT_DIR, "dump_rig.json"), "w", encoding="utf-8") as handle:
    json.dump(out, handle, indent=1)

spec = p.get("front_y")
if spec is not None:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=spec["glb"])
    mesh = bpy.data.objects[spec["mesh"]]
    half_x, band, z_at = float(spec["half_x"]), float(spec["band"]), float(spec["z"])
    ys = [v.co.y for v in mesh.data.vertices if abs(v.co.x) < half_x and abs(v.co.z - z_at) < band]
    result = {"min_y": min(ys) if ys else None, "vertices": len(ys), "z": z_at}
    print(f"front_y at z {z_at}: min y {result['min_y']} over {len(ys)} vertices")
    with open(os.path.join(OUT_DIR, "front_y.json"), "w", encoding="utf-8") as handle:
        json.dump(result, handle, indent=1)
