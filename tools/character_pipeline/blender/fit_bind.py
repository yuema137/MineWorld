# SPDX-License-Identifier: GPL-2.0-or-later
"""Stages 7–8a, fit the armature, bind, and the head and pack-core fixes: port of `rig_meshy.py` (D+ step 4).

Run by the driver as: blender --background --factory-startup <gameready.blend> --python-exit-code 1
--python fit_bind.py -- params.json

Statement for statement the D+ code, except where a value is read from params.json (PIPELINE.md §2.3, §4):
the gameready transform (scale, sole raw and sole height), the picks, the pixel divisor, the CharMorph landmark
heights, the CharMorph body that supplies the front surface (`charmorph_body`), the head and pack bounds, the smoothing
settings, and whether the pack core applies. The method constants of `rig_meshy.py` (slab half-widths, the
x filters, the toe offset, the foot selector) stay here, each with its D+ line (PIPELINE.md §2.4).

The export moved to export.py (stage 10); the provenance too. Writes fit_bind.blend and fit_bind.json.
"""

import bpy
import numpy as np
import os
import sys
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
CM = p["rig_source"]
# The CharMorph mesh whose front surface the centre points read: "cm_vitruvian" in body_g5.glb (D+ rig_meshy.py:99).
CM_FRONT_MESH = p["cm_front_mesh"]
# gameready.json's scale and lowest raw z, at the 4 decimals D+ handed over (rig_meshy.py:26; the driver rounds,
# pipeline.GAMEREADY_HANDOFF_DECIMALS, F-B1-B10-3)
S = p["scale"]
SOLE_M = p["sole_raw"]
SOLE_Z = p["sole_z_m"]  # fit.gameready.sole_z_m
PICKS = p["picks"]  # fit.fit.picks, the 2D picks on the frontal ortho render
PX = p["px_per_raw_unit"]  # 1 px = 1/PX unit on the render
CM_LM = p["charmorph_landmarks_z"]
CM_BODY = p["charmorph_body"]  # the CharMorph body export whose "cm_vitruvian" front surface the centres read
HEAD_ABOVE_CHIN = p["head_rigid"]["above_chin_m"]
HEAD_HALF = p["head_rigid"]["half_width_m"]
PACK = p["pack_core"]
SMOOTH = p["smooth"]
HAS_PACK = "backpack" in p["accessories"]


def px(u, v):
    """1024-view pixel of rig/v3_front.png -> new-frame (x, z)."""
    xm, zm = u / PX - 1.0, 1.0 - v / PX
    return xm * S, (zm - SOLE_M) * S + SOLE_Z


body = bpy.data.objects["Body"]
co = np.array([v.co[:] for v in body.data.vertices])
P = PICKS
J = {k: px(*v) for k, v in P.items()}


def slab_y(x, z, rx=0.03, rz=0.02, xmin=None):
    # method constants: the slab half-widths of rig_meshy.py:51 (rx 0.03, rz 0.02) and its callers' overrides
    sel = (np.abs(co[:, 0] - x) < rx) & (np.abs(co[:, 2] - z) < rz)
    if xmin is not None:
        sel &= np.abs(co[:, 0]) > xmin
    if sel.sum() < 5:
        raise SystemExit(f"slab at x {x:.3f} z {z:.3f}: {sel.sum()} verts")
    return float(co[sel, 1].mean())


def front_y(mesh_co, z, half=0.03, band=0.01):
    sel = (np.abs(mesh_co[:, 0]) < half) & (np.abs(mesh_co[:, 2] - z) < band)
    return float(mesh_co[sel, 1].min()) if sel.sum() else None


# --- CharMorph armature (the rig source) for its bone layout
before = set(bpy.data.objects)
bpy.ops.import_scene.gltf(filepath=CM)
new_objs = [o for o in bpy.data.objects if o not in before]
arm = next(o for o in new_objs if o.type == "ARMATURE")
arm.matrix_world.identity()
for o in new_objs:
    if o.type == "MESH":
        bpy.data.objects.remove(o, do_unlink=True)

# --- CharMorph body (PIPELINE.md §2.4, F-B1-B10-2): its own front surface, read at every centre point. It is
# imported, read and removed; only the armature above stays. Blender renames an object that collides with the
# scene's "Body" to "Body.001", so the mesh is matched by its base name.
before_body = set(bpy.data.objects)
bpy.ops.import_scene.gltf(filepath=CM_BODY)
body_objs = [o for o in bpy.data.objects if o not in before_body]
cm_mesh = next(o for o in body_objs if o.type == "MESH" and o.name.split(".")[0] == CM_FRONT_MESH)
cm_mesh.matrix_world.identity()
cm_co = np.array([v.co[:] for v in cm_mesh.data.vertices])
for o in body_objs:
    bpy.data.objects.remove(o, do_unlink=True)

bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="EDIT")
eb = arm.data.edit_bones
B = lambda n: eb["mixamorig:" + n]
cm = {b.name.split(":")[1]: (Vector(b.head), Vector(b.tail)) for b in eb}

# height map CM z -> new z through landmarks (sole, ankle, knee, hip, shoulder, chin, eye)
cm_lm = CM_LM
nw_lm = [
    SOLE_Z,
    0.5 * (J["r_ankle"][1] + J["l_ankle"][1]),
    0.5 * (J["r_knee"][1] + J["l_knee"][1]),
    0.5 * (J["r_hip"][1] + J["l_hip"][1]),
    0.5 * (J["r_shoulder"][1] + J["l_shoulder"][1]),
    J["chin"][1],
    J["eye"][1],
]
zmap = lambda z: (
    float(np.interp(z, cm_lm, nw_lm, left=None, right=None))
    if z <= cm_lm[-1]
    else nw_lm[-1] + (z - cm_lm[-1]) * (nw_lm[-1] - nw_lm[-2]) / (cm_lm[-1] - cm_lm[-2])
)
print("landmark heights CM -> new: " + ", ".join(f"{c:.3f}->{n:.3f}" for c, n in zip(cm_lm, nw_lm)))
x_scale = (J["l_shoulder"][0] - J["r_shoulder"][0]) / (cm["LeftArm"][0].x - cm["RightArm"][0].x)
print(f"shoulder width ratio {x_scale:.3f}")


def centre(p_cm):
    """A centre-line point: z mapped, x scaled, y kept at its offset behind the CharMorph front surface at its
    height and behind the new body's front surface at the mapped height (D+ rig_meshy.py:99-100)."""
    z = zmap(p_cm.z)
    fc = front_y(cm_co, p_cm.z)
    fn = front_y(co, z)
    if fc is None or fn is None:
        raise SystemExit(f"no front surface near z {p_cm.z:.3f}/{z:.3f}")
    return Vector((p_cm.x * x_scale, fn + (p_cm.y - fc), z))


new_head = {}
for n in ("Hips", "Spine", "Spine1", "Spine2", "Neck", "Head", "LeftShoulder", "RightShoulder"):
    new_head[n] = centre(cm[n][0])
head_tail = centre(cm["Head"][1])
# Above the hips the front-surface offset is unreliable: the open hoodie and the tee stand off the
# chest, and the backpack hides the back, so the spine landed on the back surface (rig/bones_side).
# The hips are pack-free and measured; the spinal curve above them is CharMorph's, kept relative
# to the hips.
hy = new_head["Hips"].y
for n in ("Spine", "Spine1", "Spine2", "Neck", "Head", "LeftShoulder", "RightShoulder"):
    new_head[n].y = hy + (cm[n][0].y - cm["Hips"][0].y)
head_tail.y = hy + (cm["Head"][1].y - cm["Hips"][0].y)
spine2_y_off = new_head["Spine2"].y - cm["Spine2"][0].y
# S32-p (Ivan's G-6): on a bare base-layer body nothing stands off the chest, and a stooped, heavy trunk is far
# from CharMorph's curve, so the trunk joints and the shoulders take the middle of the body's own depth instead.
BODY_CENTRE = p.get("spine_depth", "hips-relative") == "body-centre"


def slab_mid(x, z, rx=0.03, rz=0.02):
    """The middle of the body's depth at (x, z): halfway between its front-most and back-most vertex in the slab
    (a mean leans towards the denser front surface)."""
    sel = (np.abs(co[:, 0] - x) < rx) & (np.abs(co[:, 2] - z) < rz)
    if sel.sum() < 5:
        raise SystemExit(f"slab at x {x:.3f} z {z:.3f}: {sel.sum()} verts")
    return 0.5 * float(co[sel, 1].min() + co[sel, 1].max())


if BODY_CENTRE:
    spine2_before = new_head["Spine2"].y
    for n in ("Spine", "Spine1", "Spine2", "LeftShoulder", "RightShoulder"):
        new_head[n].y = slab_mid(new_head[n].x, new_head[n].z)
    # the neck and the head move with Spine2 as one piece, so their rest angles stay #211's: placing each at
    # its own middle tipped Ivan's head back in every walk frame (the clips animate from the rest angles)
    neck_shift = new_head["Spine2"].y - spine2_before
    new_head["Neck"].y += neck_shift
    new_head["Head"].y += neck_shift
    head_tail.y += neck_shift

for side, s in (("Left", "l"), ("Right", "r")):
    xs, zs = J[f"{s}_shoulder"]
    arm_y = slab_mid(xs, zs) if BODY_CENTRE else cm[f"{side}Arm"][0].y + spine2_y_off
    new_head[f"{side}Arm"] = Vector((xs, arm_y, zs))
    xw, zw = J[f"{s}_wrist"]
    wrist = Vector((xw, slab_y(xw, zw, xmin=0.2), zw))
    if f"{s}_elbow" in J:  # the sleeve's bend, picked
        mx, mz = J[f"{s}_elbow"]
        mid = Vector((mx, 0.0, mz))
    else:
        mid = 0.5 * (new_head[f"{side}Arm"] + wrist)
    elbow = Vector((mid.x, slab_y(mid.x, mid.z, xmin=0.2), mid.z))
    new_head[f"{side}ForeArm"], new_head[f"{side}Hand"] = elbow, wrist
    xh, zh = J[f"{s}_handtip"]
    tip = Vector((xh, slab_y(xh, zh, rx=0.02, rz=0.02, xmin=0.3), zh))
    xh_, zh_ = J[f"{s}_hip"]
    new_head[f"{side}UpLeg"] = Vector((xh_, slab_y(xh_, zh_, rx=0.04), zh_))
    xk, zk = J[f"{s}_knee"]
    new_head[f"{side}Leg"] = Vector((xk, slab_y(xk, zk, rx=0.04), zk))
    xa, za = J[f"{s}_ankle"]
    new_head[f"{side}Foot"] = Vector((xa, slab_y(xa, za, rx=0.04), za))
    # hands: CM hand chain rotated and scaled from its wrist->fingertip onto the new one
    cm_w, cm_tip = cm[f"{side}Hand"][0], cm[f"{side}HandMiddle3"][1]
    rot = (cm_tip - cm_w).rotation_difference(tip - wrist)
    k = (tip - wrist).length / (cm_tip - cm_w).length
    for bn in [b for b in cm if b.startswith(f"{side}Hand")]:
        h, t = cm[bn]
        new_head.setdefault(bn, wrist + rot @ ((h - cm_w) * k))
        cm[bn] = (h, t, wrist + rot @ ((t - cm_w) * k))
    # feet: CM foot chain translated to the new ankle, toes reaching the new shoe tip
    # method constants: the foot selector |x - xa| < 0.06, z < 0.06 and the toe offset 0.012 (rig_meshy.py:149, 152)
    foot_sel = (np.abs(co[:, 0] - xa) < 0.06) & (co[:, 2] < 0.06)
    toe_y = float(co[foot_sel, 1].min())
    cm_a, cm_toe = cm[f"{side}Foot"][0], cm[f"{side}ToeBase"][1]
    new_toe = Vector((cm_toe.x - cm_a.x + xa, toe_y + 0.012, cm_toe.z))
    rot_f = (cm_toe - cm_a).rotation_difference(new_toe - new_head[f"{side}Foot"])
    kf = (new_toe - new_head[f"{side}Foot"]).length / (cm_toe - cm_a).length
    for bn in (f"{side}Foot", f"{side}ToeBase"):
        h, t = cm[bn][0], cm[bn][1]
        nh = new_head[f"{side}Foot"] + rot_f @ ((h - cm_a) * kf)
        if bn != f"{side}Foot":
            new_head[bn] = nh
        cm[bn] = (h, t, new_head[f"{side}Foot"] + rot_f @ ((t - cm_a) * kf))
    print(
        f"{side}: shoulder {tuple(round(c, 3) for c in new_head[side + 'Arm'])}, "
        f"elbow {tuple(round(c, 3) for c in elbow)}, wrist {tuple(round(c, 3) for c in wrist)}, "
        f"hand scale {k:.3f}, foot scale {kf:.3f}"
    )

# write heads; tails follow the child heads where CM's tail met a child's head, else mapped
for b in eb:
    n = b.name.split(":")[1]
    if n in new_head:
        b.head = new_head[n]
for b in eb:
    n = b.name.split(":")[1]
    kids = [c for c in b.children if (Vector(cm[c.name.split(":")[1]][0]) - cm[n][1]).length < 1e-3]
    if kids:
        b.tail = kids[0].head.copy()
    elif n == "Head":
        b.tail = head_tail
    elif len(cm[n]) == 3:
        b.tail = cm[n][2]
    else:
        b.tail = b.head + (cm[n][1] - cm[n][0])
    b.roll = b.roll  # rolls kept as imported
joints_report = {}
for n in (
    "Hips",
    "Spine2",
    "Neck",
    "Head",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "LeftUpLeg",
    "LeftLeg",
    "LeftFoot",
):
    joints_report[n] = {"cm": [round(c, 3) for c in cm[n][0]], "new": [round(c, 3) for c in B(n).head]}
    print(
        f"  {n:12s} CM {tuple(round(c, 3) for c in cm[n][0])} -> new {tuple(round(c, 3) for c in B(n).head)}"
    )
bpy.ops.object.mode_set(mode="OBJECT")

# --- bind: automatic (heat) weights
for o in bpy.data.objects:
    o.select_set(o in (body, arm))
bpy.context.view_layer.objects.active = arm
bpy.ops.object.parent_set(type="ARMATURE_AUTO")
vg = body.vertex_groups
w = np.zeros((len(co), len(vg)))
for v in body.data.vertices:
    for g in v.groups:
        w[v.index, g.group] = g.weight
unweighted = int((w.sum(1) < 1e-4).sum())
print(f"auto weights: {len(vg)} groups, {unweighted} vertices with no weight")

# --- fixes, each printed
gi = {g.name.split(":")[1]: g.index for g in vg if ":" in g.name}
fixes = []


def set_rigid(mask, bone, label):
    idx = np.nonzero(mask)[0].tolist()
    for g in vg:
        g.remove(idx)
    vg[gi[bone]].add(idx, 1.0, "REPLACE")
    print(f"fix: {len(idx)} verts {label} -> {bone} only")
    fixes.append({"label": label, "bone": bone, "verts": len(idx)})


chin_z = J["chin"][1]
# hair, bun and face above the chin follow the head alone (no neck or shoulder pull on strands)
set_rigid((co[:, 2] > chin_z + HEAD_ABOVE_CHIN) & (np.abs(co[:, 0]) < HEAD_HALF), "Head", "above the chin")
if HAS_PACK:
    # the backpack body: behind the back by more than its own depth margin, between hips and shoulders
    sp1 = new_head["Spine1"]
    pack = (
        (co[:, 1] > sp1.y + PACK["behind_spine1_m"])
        & (co[:, 2] > new_head["Hips"].z + PACK["above_hips_m"])
        & (co[:, 2] < new_head["Spine2"].z + PACK["above_spine2_m"])
        & (np.abs(co[:, 0]) < PACK["half_width_m"])
    )
    # |x| < 0.12, not 0.19: at 0.19 the rigid region reached the back of the armpit and the
    # surface between it and the arm tore when the arm lifted (runtime_d2, P2/P4 3x zoom)
    set_rigid(pack, "Spine2", "of backpack (behind the back)")
# unweighted leftovers: nearest bone head
if unweighted:
    heads = {n: B(n).head for n in gi}
    for i in np.nonzero(w.sum(1) < 1e-4)[0]:
        p_i = Vector(co[i])
        best = min(heads, key=lambda n: (heads[n] - p_i).length)
        vg[gi[best]].add([int(i)], 1.0, "REPLACE")
    print(f"fix: {unweighted} unweighted verts -> nearest bone head")
    fixes.append({"label": "unweighted leftovers", "bone": "nearest bone head", "verts": unweighted})

# --- soften the seams the rigid fixes leave (chin line, pack edge): smooth all groups, keep the
# four strongest influences per vertex (what the glTF skin carries), normalise
for o in bpy.data.objects:
    o.select_set(o == body)
bpy.context.view_layer.objects.active = body
bpy.ops.object.mode_set(mode="WEIGHT_PAINT")
bpy.ops.object.vertex_group_smooth(group_select_mode="ALL", factor=SMOOTH["factor"], repeat=SMOOTH["repeat"])
bpy.ops.object.vertex_group_limit_total(group_select_mode="ALL", limit=SMOOTH["limit"])
bpy.ops.object.vertex_group_normalize_all(group_select_mode="ALL", lock_active=False)
bpy.ops.object.mode_set(mode="OBJECT")
print("weights: smoothed, limited to 4 influences, normalised")

# the armature and body names are the export's (stage 10); this stage leaves the scene as it is
body.name = body.data.name = "Body"
arm.name = "mixamo_vitruvian"
common.write_result(
    "fit_bind",
    {
        "shoulder_width_ratio": x_scale,
        "auto_groups": len(vg),
        "auto_unweighted": unweighted,
        "joints": joints_report,
        "fixes": fixes,
    },
)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(p["out_dir"], "fit_bind.blend"))
