# SPDX-License-Identifier: GPL-3.0-or-later
"""A review render: several meshes (GLB or OBJ, glTF axes) in flat colours, orthographic front, side and back views.

Used to look at a registration (the fitted template over the body) or a garment over a body; it judges nothing.
Params: {"items": [{"path": <glb|obj>, "colour": [r, g, b, a]}], "out": <png prefix>, "size": [w, h]}.
Writes <out>_front.png, <out>_side.png, <out>_back.png.
"""

import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402


def load(path):
    before = set(bpy.data.objects)
    if path.endswith(".obj"):
        # glTF axes in the file: Blender = (x, -z, y), which is the OBJ importer's -Z forward, +Y up.
        bpy.ops.wm.obj_import(filepath=path, forward_axis="NEGATIVE_Z", up_axis="Y")
    else:
        bpy.ops.import_scene.gltf(filepath=path)
    return [o for o in bpy.data.objects if o not in before and o.type == "MESH"]


def main():
    p = mc.params()
    mc.clear_scene()
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "OBJECT"
    scene.display.shading.show_xray = False
    w, h = p.get("size", [700, 1000])
    scene.render.resolution_x, scene.render.resolution_y = w, h
    zmax = 0.0
    for item in p["items"]:
        for obj in load(item["path"]):
            obj.color = item["colour"]
            zmax = max(zmax, max((obj.matrix_world @ v.co).z for v in obj.data.vertices))
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = max(2.0, zmax * 1.1)
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    centre_z = zmax / 2
    for view, (x, y, rz) in {"front": (0, -6, 0), "side": (6, 0, 90), "back": (0, 6, 180)}.items():
        cam.location = (x, y, centre_z)
        cam.rotation_euler = (math.radians(90), 0, math.radians(rz))
        scene.render.filepath = f"{p['out']}_{view}.png"
        bpy.ops.render.render(write_still=True)
        mc.log("rendered", scene.render.filepath)


main()
