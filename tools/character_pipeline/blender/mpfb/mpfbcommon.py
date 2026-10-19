# SPDX-License-Identifier: GPL-3.0-or-later
"""Shared helpers of the S32-p MPFB2 stages (pr-s32p-wardrobe-pilot.md §4.3; DEP-72).

These scripts run inside Blender 5.2.2 with the MPFB2 2.0.18 extension loaded from an isolated
BLENDER_USER_RESOURCES (F-S32p-3), so they are GPL-3.0-or-later like MPFB2's API they call. The MIT driver
(`mwcast/wardrobe/stages.py`) never imports them; it runs them as subprocesses with one params.json.

Coordinates: Blender is +Z up with the character facing -Y; the files the stages write use glTF's axes (+Y up,
facing +Z), which are MakeHuman's own axes at 1/10 scale: glTF (x, y, z) = Blender (x, z, -y).
"""

import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

REGIONS = (
    "scalp",
    "face",
    "neck",
    "torso",
    "upper-arms",
    "forearms",
    "hands",
    "hips",
    "thighs",
    "shins",
    "feet",
)
M = "mixamorig:"


def params():
    argv = sys.argv[sys.argv.index("--") + 1 :]
    return json.loads(Path(argv[0]).read_text(encoding="utf-8"))


def write_result(path, data):
    Path(path).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def log(*parts):
    print("[S32P]", *parts, flush=True)


def to_gltf(v):
    return (float(v[0]), float(v[2]), float(-v[1]))


def from_gltf(v):
    return Vector((v[0], -v[2], v[1]))


def clear_scene():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    for mesh in list(bpy.data.meshes):
        bpy.data.meshes.remove(mesh)
    for arm in list(bpy.data.armatures):
        bpy.data.armatures.remove(arm)


def mpfb():
    """The MPFB2 services (imported late: they exist only when the extension is loaded)."""
    from bl_ext.user_default.mpfb.services.humanservice import HumanService
    from bl_ext.user_default.mpfb.services.targetservice import TargetService

    return HumanService, TargetService


# Our map from the casting brief to MPFB2's macro sliders (design §4.3 R-4 step 1). The age map is MakeHuman's
# own (1 y at 0, 25 y at 0.5, 90 y at 1, piecewise linear: modifiers/macro age); the build map is ours, a
# starting point only — the stature is then calibrated by bisection on `height` (R-S32-6).
BUILD_WEIGHT = {"slim": 0.3, "average": 0.5, "heavy": 0.8}


def age_value(years):
    if years <= 25:
        return max(0.0, (years - 1.0) / 48.0)
    return min(1.0, 0.5 + (years - 25.0) / 130.0)


def macro_dict(macros, height):
    _, TargetService = mpfb()
    md = TargetService.get_default_macro_info_dict()
    md["gender"] = 0.0 if macros["sex"] == "female" else 1.0
    md["age"] = age_value(float(macros["apparent_age"]))
    # A fixture may name the slider directly (W-5's heaviest and lightest extremes); a subject never does.
    md["weight"] = (
        float(macros["weight_value"]) if "weight_value" in macros else BUILD_WEIGHT[macros["build"]]
    )
    md["height"] = height
    return md


def body_group_indices(basemesh):
    group = basemesh.vertex_groups["body"].index
    out = []
    for v in basemesh.data.vertices:
        for g in v.groups:
            if g.group == group and g.weight > 0.5:
                out.append(v.index)
                break
    return out


def evaluated_positions(obj):
    """World positions of every vertex of `obj` with its shape keys and modifiers, the helper mask disabled."""
    for mod in obj.modifiers:
        if mod.type == "MASK":
            mod.show_viewport = False
    bpy.context.view_layer.update()
    deps = bpy.context.evaluated_depsgraph_get()
    ev = obj.evaluated_get(deps)
    mesh = ev.to_mesh()
    mw = obj.matrix_world
    out = [mw @ v.co for v in mesh.vertices]
    normals = [(mw.to_3x3() @ v.normal).normalized() for v in mesh.vertices]
    ev.to_mesh_clear()
    return out, normals


def build_human(macros, height):
    """An MPFB2 `hm08` human with the macros and a height slider value; feet on the ground (z = 0)."""
    HumanService, _ = mpfb()
    basemesh = HumanService.create_human(macro_detail_dict=macro_dict(macros, height))
    basemesh.name = "Template"
    return basemesh


def set_height(basemesh, height):
    """Re-apply the macros with a new height (MPFB2's own path for a slider change), feet back on the ground."""
    from bl_ext.user_default.mpfb.entities.objectproperties import HumanObjectProperties

    _, TargetService = mpfb()
    HumanObjectProperties.set_value("height", height, entity_reference=basemesh)
    TargetService.reapply_macro_details(basemesh)


def stature(basemesh, body_ids):
    """Sole-to-crown of the template's body vertices (helpers excluded), and its lowest z."""
    pos, _ = evaluated_positions(basemesh)
    zs = [pos[i].z for i in body_ids]
    return max(zs) - min(zs), min(zs)


def calibrate_height(macros, target_m, tolerance_m=0.005):
    """Build the template and bisect the `height` slider until its stature equals `target_m` within 5 mm
    (R-S32-6; the target is the subject's recorded stature, never the candidate's). Returns (basemesh, log)."""
    basemesh = build_human(macros, 0.5)
    body_ids = body_group_indices(basemesh)
    low, high = 0.0, 1.0
    trail = []
    value = 0.5
    for _ in range(20):
        height, _z = stature(basemesh, body_ids)
        trail.append((round(value, 6), round(height, 5)))
        if abs(height - target_m) <= tolerance_m:
            break
        if height < target_m:
            low = value
        else:
            high = value
        value = 0.5 * (low + high)
        set_height(basemesh, value)
    height, _z = stature(basemesh, body_ids)
    return (
        basemesh,
        body_ids,
        {"height_slider": value, "stature_m": height, "target_m": target_m, "trail": trail},
    )


def region_of_bone(name):
    """R-5's rule: a template vertex's region is the region of its dominant `mixamo` bone (head split later)."""
    n = name[len(M) :] if name.startswith(M) else name
    if n in ("Head", "HeadTop_End"):
        return "head"
    if n == "Neck":
        return "neck"
    if n in ("Spine", "Spine1", "Spine2") or n.endswith("Shoulder"):
        return "torso"
    if n.endswith("ForeArm"):
        return "forearms"
    if n.endswith("Arm"):
        return "upper-arms"
    if "Hand" in n:
        return "hands"
    if n == "Hips":
        return "hips"
    if n.endswith("UpLeg"):
        return "thighs"
    if n.endswith("Leg"):
        return "shins"
    if n.endswith("Foot") or "Toe" in n:
        return "feet"
    raise ValueError("no region for bone " + name)


def dominant_bones(basemesh):
    names = {g.index: g.name for g in basemesh.vertex_groups if g.name.startswith(M)}
    out = {}
    for v in basemesh.data.vertices:
        best, weight = None, 0.0
        for g in v.groups:
            if g.group in names and g.weight > weight:
                best, weight = names[g.group], g.weight
        if best is not None:
            out[v.index] = best
    return out


def joint_cube_centre(basemesh, positions, group_name):
    group = basemesh.vertex_groups[group_name].index
    pts = [positions[v.index] for v in basemesh.data.vertices if any(g.group == group for g in v.groups)]
    return sum(pts, Vector()) / len(pts)


def angle_deg(a, b):
    return math.degrees(a.angle(b))
