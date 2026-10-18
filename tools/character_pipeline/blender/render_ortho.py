# SPDX-License-Identifier: GPL-2.0-or-later
"""Support: a frontal orthographic clay render with a known pixel <-> world mapping, for landmark picking.

Port of `render_ortho.py` (D+). Run by the driver as: blender --background --factory-startup --python-exit-code 1
--python render_ortho.py -- params.json, with params `src` (a GLB), `out` (a PNG), `cx`, `cz`, `scale`, `res`.

Camera on -Y looking +Y, up +Z, so image right = +X and image down = -Z:
    x = cx + (u / res - 0.5) * scale        z = cz - (v / res - 0.5) * scale
Meshes named Hair* are hidden (the CharMorph build), as are garment objects.
"""

import bpy
import os
import sys
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
src, out = p["src"], p["out"]
cx, cz, scale = float(p["cx"]), float(p["cz"]), float(p["scale"])
res = int(p["res"])
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=src)
keep = None
for o in bpy.data.objects:
    if o.type == "MESH" and o.data.shape_keys:
        for kb in o.data.shape_keys.key_blocks:
            kb.value = 0.0
meshes = [o for o in bpy.data.objects if o.type == "MESH"]
if len(meshes) > 1:  # a CharMorph build: show the body only
    for o in meshes:
        o.hide_render = o.name not in ("Body", "cm_vitruvian")
scene = bpy.context.scene
scene.render.engine = (
    "BLENDER_EEVEE_NEXT"
    if "BLENDER_EEVEE_NEXT"
    in [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
    else "BLENDER_EEVEE"
)
scene.render.resolution_x = scene.render.resolution_y = res
scene.view_settings.view_transform = "Standard"
w = bpy.data.worlds.new("w")
scene.world = w
w.use_nodes = True
w.node_tree.nodes["Background"].inputs[0].default_value = (0.5, 0.5, 0.5, 1)
w.node_tree.nodes["Background"].inputs[1].default_value = 0.7
clay = bpy.data.materials.new("clay")
clay.use_nodes = True
clay.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.75, 0.65, 0.6, 1)
dark = bpy.data.materials.new("dark")
dark.use_nodes = True
dark.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.1, 0.07, 0.06, 1)
for o in meshes:
    for s in o.material_slots:
        n = s.material.name if s.material else ""
        if len(meshes) > 1:
            s.material = dark if ("Iris" in n or "Pupil" in n) else clay
cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.data.type = "ORTHO"
cam.data.ortho_scale = scale
cam.data.clip_end = 100
cam.location = (cx, -10.0, cz)
cam.rotation_euler = Vector((0, 1, 0)).to_track_quat("-Z", "Z").to_euler()
sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
sun.data.energy = 3.0
sun.rotation_euler = Vector((0.3, 1.0, -0.5)).to_track_quat("-Z", "Z").to_euler()
scene.collection.objects.link(sun)
scene.render.filepath = out
bpy.ops.render.render(write_still=True)
print("mapping", cx, cz, scale, res)
