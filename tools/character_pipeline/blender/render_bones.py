# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 12, the bone-overlay review frames: port of `render_rig.py` (D+ section 7).

Front and side orthographic views of a rigged .blend: the body translucent, every bone drawn as a thin red cylinder
from head to tail, joints as small spheres. Optionally pose the rig first.

Run by the driver as: blender --background --factory-startup <rig.blend> --python-exit-code 1
--python render_bones.py -- params.json, with params `out_prefix` (a path prefix for the PNGs) and `pose`
(an object: bone name -> [rx, ry, rz] degrees in the bone's local frame; empty for the overlay).
"""

import bpy
import math
import os
import sys
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
out = p["out_prefix"]
pose = p["pose"]
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
body = bpy.data.objects["Body"]
for name, deg in pose.items():
    pb = arm.pose.bones[name]
    pb.rotation_mode = "XYZ"
    pb.rotation_euler = Euler([math.radians(d) for d in deg], "XYZ")
bpy.context.view_layer.update()

red = bpy.data.materials.new("red")
red.use_nodes = True
red.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (1, 0.05, 0.05, 1)
red.node_tree.nodes["Principled BSDF"].inputs["Emission Color"].default_value = (1, 0, 0, 1)
red.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 2.0
show_bones = not pose
if show_bones:
    mat = body.data.materials[0]
    bs = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bs.inputs["Alpha"].default_value = 0.35
    if hasattr(mat, "surface_render_method"):
        mat.surface_render_method = "BLENDED"
    for pb in arm.pose.bones:
        h = arm.matrix_world @ pb.head
        t = arm.matrix_world @ pb.tail
        d = t - h
        bpy.ops.mesh.primitive_cylinder_add(radius=0.004, depth=d.length, location=(h + t) / 2)
        c = bpy.context.object
        c.rotation_euler = d.to_track_quat("Z", "Y").to_euler()
        c.data.materials.append(red)
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.008, location=h)
        bpy.context.object.data.materials.append(red)
scene = bpy.context.scene
scene.render.engine = (
    "BLENDER_EEVEE_NEXT"
    if "BLENDER_EEVEE_NEXT"
    in [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
    else "BLENDER_EEVEE"
)
scene.render.resolution_x, scene.render.resolution_y = 1024, 1024
scene.view_settings.view_transform = "Standard"
w = bpy.data.worlds.new("w")
scene.world = w
w.use_nodes = True
w.node_tree.nodes["Background"].inputs[0].default_value = (0.5, 0.5, 0.5, 1)
w.node_tree.nodes["Background"].inputs[1].default_value = 0.8
cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.data.type = "ORTHO"
sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
sun.data.energy = 3.0
scene.collection.objects.link(sun)
views = {
    "front": Vector((0, -1, 0)),
    "side": Vector((-1, 0, 0)),
    "back": Vector((0, 1, 0)),
    "tq": Vector((-0.5, -0.866, 0)),
}
for name, d in views.items():
    for region, c, sc in (("full", Vector((0, 0.05, 0.9)), 2.0), ("upper", Vector((0, 0.05, 1.35)), 0.8)):
        cam.data.ortho_scale = sc
        cam.location = c + d * 5
        cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
        sun.rotation_euler = (-(d + Vector((0.3, 0.2, 0.6)))).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = f"{out}_{name}_{region}.png"
        bpy.ops.render.render(write_still=True)
print("done")
