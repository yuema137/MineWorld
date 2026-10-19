# SPDX-License-Identifier: GPL-3.0-or-later
"""R-6: the wearer's own body rendered as grey clay, the mannequin of every garment reference image
(pr-s32p-wardrobe-pilot.md §4.3, F-S32p-10, D-p-1). No MPFB2 call: it lives here for the scratch environment.

The body in one uniform matte mid-grey (skin cap included), at casting template v2's camera: a level camera at
navel height (taken as 0.60 of the stature; the template asks only for a level lens near mid-body), 85 mm
full-frame equivalent
on the long side (portrait), framed crown to soles with 6 % empty margin above and below, front view; the v2
backdrop colour #C8C4BE. Workbench, studio light, no shadows: the shape is what the image model needs.

Params: {"body": glb, "out": png, "size": [1024, 1536]}.
"""

import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402

BACKDROP_SRGB = (0xC8 / 255, 0xC4 / 255, 0xBE / 255)  # template.yaml full_body.background
CLAY_SRGB = (0.50, 0.50, 0.50)
LENS_MM = 85.0
SENSOR_LONG_MM = 36.0
MARGIN = 0.06
NAVEL_RATIO = 0.60


def linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def main():
    p = mc.params()
    mc.clear_scene()
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=p["body"])
    meshes = [o for o in bpy.data.objects if o not in before and o.type == "MESH"]
    zs = [(o.matrix_world @ v.co).z for o in meshes for v in o.data.vertices]
    low, high = min(zs), max(zs)
    stature = high - low
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "SINGLE"
    shading.single_color = tuple(linear(c) for c in p.get("srgb", CLAY_SRGB))  # L-a: the recipe's mannequin
    shading.show_shadows = False
    shading.show_cavity = True
    if scene.world is None:
        scene.world = bpy.data.worlds.new("World")
    scene.world.color = tuple(linear(c) for c in BACKDROP_SRGB)
    w, h = p.get("size", [1024, 1536])
    scene.render.resolution_x, scene.render.resolution_y = w, h
    scene.render.film_transparent = False
    cam_data = bpy.data.cameras.new("v2")
    cam_data.lens = LENS_MM
    cam_data.sensor_fit = "VERTICAL"
    cam_data.sensor_height = SENSOR_LONG_MM
    cam = bpy.data.objects.new("v2", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    half_fov = math.atan((SENSOR_LONG_MM / 2) / LENS_MM)
    frame_h = stature / (1 - 2 * MARGIN)
    navel = low + NAVEL_RATIO * stature
    # A level camera at navel height: the frame's vertical half-extent must reach both crown and soles.
    reach = max(high + MARGIN * frame_h - navel, navel - (low - MARGIN * frame_h))
    distance = reach / math.tan(half_fov)
    cam.location = (0.0, -distance, navel)
    cam.rotation_euler = (math.radians(90), 0.0, 0.0)
    scene.render.filepath = p["out"]
    bpy.ops.render.render(write_still=True)
    mc.write_result(
        Path(p["out"]).with_suffix(".json"),
        {"stature_m": stature, "distance_m": distance, "camera_height_m": navel, "lens_mm": LENS_MM},
    )
    mc.log("clay", p["out"], "stature", round(stature, 4), "distance", round(distance, 3))


main()
