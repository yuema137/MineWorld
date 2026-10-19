# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 8c, the pack side panels: port of `reweight_pack.py` (D+ section 8.7), parameterized.

Run by the driver as: blender --background --factory-startup <weights_fingers.blend> --python-exit-code 1
--python weights_pack.py -- params.json. Driven only when fit.accessories names a pack.

Statement for statement the D+ code, except the values of fit.weights.pack_panels (x_in, x_out, arm_zero, smooth,
smooth_repeat) and the pack-core bounds of fit.weights.pack_core, which are read from params.json. The zone of the
smoothing pass (its 0.02, 0.13 and 0.02 margins) and the sleeve test (ARMISH, a share of 0.5) are method constants,
each with its D+ line (PIPELINE.md §2.4). Writes weights_pack.blend and weights_pack.json.
"""

import bpy
import numpy as np
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
X_IN = p["x_in"]
X_OUT = p["x_out"]
ARMZERO = p["arm_zero"]
SMOOTH = p["smooth"]
SMOOTH_REPEAT = p["smooth_repeat"]
CORE = p["pack_core"]
SLEEVE_SHARE_MAX = p["sleeve_share_max"]
body = bpy.data.objects["Body"]
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
vg = body.vertex_groups
mw = arm.matrix_world


def bh(n):
    return mw @ arm.data.bones["mixamorig:" + n].head_local


sp1, sp2, hips = bh("Spine1"), bh("Spine2"), bh("Hips")
co = np.array([v.co[:] for v in body.data.vertices])
ax = np.abs(co[:, 0])
region = (
    (co[:, 1] > sp1.y + CORE["behind_spine1_m"])
    & (co[:, 2] > hips.z + CORE["above_hips_m"])
    & (co[:, 2] < sp2.z + CORE["above_spine2_m"])
    & (ax >= X_IN)
    & (ax < X_OUT)
)
ARMISH = ("Arm", "ForeArm", "Shoulder", "Hand")
sp2_g = vg["mixamorig:Spine2"].index
n_done = n_sleeve = 0
for i in np.nonzero(region)[0].tolist():
    v = body.data.vertices[i]
    ws = {vg[g.group].name: g.weight for g in v.groups}
    armw = sum(w for n, w in ws.items() if any(n.split(":")[-1].endswith(s) for s in ARMISH))
    if armw > SLEEVE_SHARE_MAX:
        n_sleeve += 1
        continue
    f = float(np.clip((X_OUT - ax[i]) / (X_OUT - X_IN), 0.0, 1.0))
    new = {n: w * (1.0 - f) for n, w in ws.items()}
    new["mixamorig:Spine2"] = new.get("mixamorig:Spine2", 0.0) + f
    if ARMZERO:
        # no upper-arm/forearm share at all on the pack: in the jog, with the arm
        # raised forward, a residual share folded the side panel into dark slits
        moved = sum(w for n, w in new.items() if n.split(":")[-1].endswith(("Arm", "ForeArm")))
        new = {n: w for n, w in new.items() if not n.split(":")[-1].endswith(("Arm", "ForeArm"))}
        new["mixamorig:Spine2"] = new.get("mixamorig:Spine2", 0.0) + moved
    tot = sum(new.values())
    for g in list(v.groups):
        vg[g.group].remove([i])
    for n, w in new.items():
        if w / tot > 1e-4:
            vg[n].add([i], w / tot, "REPLACE")
    n_done += 1
print(
    f"pack side panels |x| {X_IN}..{X_OUT}: {n_done} verts blended toward Spine2, "
    f"{n_sleeve} sleeve verts left alone"
)
smoothed = 0

if SMOOTH:
    # The pack's side wall reaches from its back face down to the hoodie, and the part nearest
    # the body (inside the depth margin) kept its heat weights: in the jog the seam between the
    # two opened into dark slits. Smooth all groups over the whole pack volume, wall included,
    # so the weights change gradually across it.
    # method constants of the smoothing zone (reweight_pack.py:66-67): margins 0.02, 0.13 and 0.02
    zone = (co[:, 1] > sp1.y + 0.02) & (co[:, 2] > hips.z) & (co[:, 2] < sp2.z + 0.13) & (ax < X_OUT + 0.02)
    me = body.data
    for v in me.vertices:
        v.select = bool(zone[v.index])
    for o in bpy.data.objects:
        o.select_set(o == body)
    bpy.context.view_layer.objects.active = body
    me.use_paint_mask_vertex = True
    bpy.ops.object.mode_set(mode="WEIGHT_PAINT")
    bpy.ops.object.vertex_group_smooth(group_select_mode="ALL", factor=0.5, repeat=SMOOTH_REPEAT)
    bpy.ops.object.vertex_group_limit_total(group_select_mode="ALL", limit=4)
    bpy.ops.object.vertex_group_normalize_all(group_select_mode="ALL", lock_active=False)
    bpy.ops.object.mode_set(mode="OBJECT")
    me.use_paint_mask_vertex = False
    smoothed = int(zone.sum())
    print(f"smoothed weights over the pack volume: {smoothed} verts, {SMOOTH_REPEAT} passes")

common.write_result(
    "weights_pack",
    {
        "blended": n_done,
        "sleeve_left_alone": n_sleeve,
        "smoothed_verts": smoothed,
        "smooth_passes": SMOOTH_REPEAT if SMOOTH else 0,
    },
)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(p["out_dir"], "weights_pack.blend"))
