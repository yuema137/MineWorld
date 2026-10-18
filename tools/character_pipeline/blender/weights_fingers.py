# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 8b, the finger re-weight: port of `reweight_fingers.py` (D+ section 8.4), parameterized.

Run by the driver as: blender --background --factory-startup <fit_bind.blend> --python-exit-code 1
--python weights_fingers.py -- params.json

Statement for statement the D+ code, except the three numbers of fit.weights.fingers (the hand share, the power,
and the number of bones kept), which are read from params.json. Writes weights_fingers.blend and
weights_fingers.json (the dominant-bone counts per hand).
"""

import bpy
import numpy as np
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
HAND_SHARE = p["hand_share_min"]
POWER = p["power"]
KEEP = p["keep"]
body = bpy.data.objects["Body"]
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
vg = body.vertex_groups
co = np.array([v.co[:] for v in body.data.vertices])
mw = arm.matrix_world
hands = {}


def seg_dist(p_pts, a, b):
    ab = b - a
    t = np.clip(((p_pts - a) @ ab) / max(ab @ ab, 1e-12), 0.0, 1.0)
    return np.linalg.norm(p_pts - (a + t[:, None] * ab), axis=1)


for side in ("Left", "Right"):
    names = [b.name for b in arm.data.bones if b.name.startswith(f"mixamorig:{side}Hand")]
    gidx = {n: vg[n].index for n in names if n in vg}
    hand_w = np.zeros(len(co))
    for v in body.data.vertices:
        for g in v.groups:
            if vg[g.group].name in gidx:
                hand_w[v.index] += g.weight
    sel = np.nonzero(hand_w > HAND_SHARE)[0]
    P = co[sel]
    D = []
    for n in gidx:
        b = arm.data.bones[n]
        D.append(seg_dist(P, np.array((mw @ b.head_local)[:]), np.array((mw @ b.tail_local)[:])))
    D = np.stack(D, 1)  # (verts, bones)
    W = 1.0 / np.maximum(D, 1e-4) ** POWER
    topk = np.argsort(-W, axis=1)[:, :KEEP]
    keep = np.zeros_like(W)
    rows = np.arange(len(sel))[:, None]
    keep[rows, topk] = W[rows, topk]
    keep /= keep.sum(1, keepdims=True)
    order = list(gidx)
    idx_list = sel.tolist()
    for g in vg:
        g.remove(idx_list)
    for j, n in enumerate(order):
        m = keep[:, j] > 1e-4
        for i, w in zip(sel[m].tolist(), keep[m, j].tolist()):
            vg[n].add([i], w, "REPLACE")
    dom = np.array(order)[np.argmax(keep, 1)]
    counts = {n.split("Hand")[1] or "palm": int((dom == n).sum()) for n in order}
    hands[side] = {"verts": len(sel), "dominant": counts}
    print(f"{side} hand: {len(sel)} verts re-weighted; dominant bone counts {counts}")

common.write_result("weights_fingers", {"hands": hands})
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(p["out_dir"], "weights_fingers.blend"))
