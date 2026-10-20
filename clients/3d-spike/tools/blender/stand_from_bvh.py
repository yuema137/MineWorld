# SPDX-License-Identifier: GPL-2.0-or-later
"""The standing idle clip from a recorded BVH: 100STYLE `Neutral_ID.bvh` (Mason, Starke & Komura 2022, CC BY 4.0)
-> `assets/characters/stand_lib.glb` (PR S26-c-pose, part C; `docs/decisions/DEP-146.md`).

    blender --background --factory-startup --python-exit-code 1 --python stand_from_bvh.py -- \
        <Neutral_ID.bvh> <out stand_lib.glb>

Nothing here is a pose number. The script:

1. imports the BVH with Blender's own importer;
2. makes a copy of its skeleton without the fourth chest joint (100STYLE has four chest joints; the humanoid profile
   has three), every remaining bone following the recording in world space (Copy Transforms, baked with visual
   keying), so no rotation of the recording is lost;
3. renames the bones to the `mixamorig_*` names the 3D client's bone map (`vitruvian_bonemap.tres`) maps onto
   `SkeletonProfileHumanoid`, so the clip retargets through the same import path as every body and the UAL clips;
4. closes the loop on the recording itself: the clip ends at the frame, in the second half of the take, whose pose
   is closest to the first frame's (sum of the bones' local rotation angles), so at least half the take is kept;
5. strips the root's horizontal drift: the hips' mean ground-plane position and its linear trend over the loop are
   removed (the take's weight shift and sway stay; the body does not wander off its spot);
6. exports the skeleton with a one-triangle mesh skinned to the hips (Godot builds a `Skeleton3D` only for a skin
   some mesh uses; `tools/animation_trim.py` does the same) and the one action, named `Stand`.
"""

import math
import sys

import bpy
from mathutils import Vector

args = sys.argv[sys.argv.index("--") + 1 :]
BVH, OUT = args[0], args[1]

# 100STYLE joint -> the mixamorig name the client's bone map uses (vitruvian_bonemap.tres)
NAMES = {
    "Hips": "Hips",
    "Chest": "Spine",
    "Chest2": "Spine1",
    "Chest3": "Spine2",
    "Neck": "Neck",
    "Head": "Head",
}
for side in ("Left", "Right"):
    NAMES |= {
        f"{side}Collar": f"{side}Shoulder",
        f"{side}Shoulder": f"{side}Arm",
        f"{side}Elbow": f"{side}ForeArm",
        f"{side}Wrist": f"{side}Hand",
        f"{side}Hip": f"{side}UpLeg",
        f"{side}Knee": f"{side}Leg",
        f"{side}Ankle": f"{side}Foot",
        f"{side}Toe": f"{side}ToeBase",
    }
DROPPED = "Chest4"

bpy.ops.wm.read_factory_settings(use_empty=True)
# 100STYLE records in centimetres
bpy.ops.import_anim.bvh(filepath=BVH, global_scale=0.01, update_scene_fps=True, update_scene_duration=True)
src = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
scene = bpy.context.scene
f0, f1 = scene.frame_start, scene.frame_end
print(f"stand_from_bvh: {BVH}: {f1 - f0 + 1} frames at {scene.render.fps} fps; bones {[b.name for b in src.data.bones]}")

# the BVH's units: its hips height at rest, for the log (the client normalises position tracks by hips height)
hips_rest = src.data.bones["Hips"].head_local
print(f"stand_from_bvh: rest hips at {tuple(round(c, 3) for c in hips_rest)} (BVH units)")

# 4. the standing part of the take, and its loop. The take walks in, stands and walks out. The standing part is the
# longest run of frames in which both ankles stay within FOOT_PLANT_M of their median position over the take (a
# planted foot; the take stands for most of its length, so the median is the standing position). The loop is the pair
# of frames in it whose poses are closest (sum of the bones' local rotation angles), at least half the standing part
# apart.
FOOT_PLANT_M = 0.02  # method constant: a planted foot's drift, well above optical mocap noise (~1 mm)
ankles = [src.pose.bones["LeftAnkle"], src.pose.bones["RightAnkle"]]
pos = []
for f in range(f0, f1 + 1):
    scene.frame_set(f)
    pos.append([(src.matrix_world @ a.matrix).translation.copy() for a in ankles])


def median(vals):
    s = sorted(vals)
    return s[len(s) // 2]


med = [Vector((median([p[k].x for p in pos]), median([p[k].y for p in pos]), 0.0)) for k in range(2)]
planted = [
    all((Vector((p[k].x, p[k].y, 0.0)) - med[k]).length < FOOT_PLANT_M for k in range(2)) for p in pos
]
run_a, run_len, cur_a = f0, 0, None
for i, ok in enumerate(planted + [False]):
    if ok and cur_a is None:
        cur_a = i
    elif not ok and cur_a is not None:
        if i - cur_a > run_len:
            run_a, run_len = cur_a, i - cur_a
        cur_a = None
a0, a1 = f0 + run_a, f0 + run_a + run_len - 1
print(f"stand_from_bvh: standing part {a0}..{a1} ({run_len / scene.render.fps:.2f} s of {(f1 - f0 + 1) / scene.render.fps:.2f} s)")


def pose_at(frame: int) -> list:
    scene.frame_set(frame)
    return [pb.matrix_basis.to_quaternion().copy() for pb in src.pose.bones]


poses = {f: pose_at(f) for f in range(a0, a1 + 1)}


def distance(fa: int, fb: int) -> float:
    return sum(qa.rotation_difference(qb).angle for qa, qb in zip(poses[fa], poses[fb]))


# the pair (start, end) with the closest poses, the loop at least half the standing part (searched every 4 frames,
# then refined frame by frame round the best pair)
half = (a1 - a0) // 2
best_pair, best_d = (a0, a1), math.inf
for fa in range(a0, a0 + half + 1, 4):
    for fb in range(fa + half, a1 + 1, 4):
        d = distance(fa, fb)
        if d < best_d:
            best_pair, best_d = (fa, fb), d
fa0, fb0 = best_pair
for fa in range(max(a0, fa0 - 3), min(a0 + half, fa0 + 3) + 1):
    for fb in range(max(fa + half, fb0 - 3), min(a1, fb0 + 3) + 1):
        d = distance(fa, fb)
        if d < best_d:
            best_pair, best_d = (fa, fb), d
f0, best = best_pair
print(f"stand_from_bvh: loop {f0}..{best} ({(best - f0) / scene.render.fps:.2f} s), closing distance {math.degrees(best_d):.2f} deg summed over {len(poses[f0])} joints")

# 2. the target skeleton: a copy without the fourth chest joint
tgt = src.copy()
tgt.data = src.data.copy()
tgt.animation_data_clear()
scene.collection.objects.link(tgt)
bpy.context.view_layer.objects.active = tgt
for o in scene.objects:
    o.select_set(o == tgt)
bpy.ops.object.mode_set(mode="EDIT")
eb = tgt.data.edit_bones
parent = eb[DROPPED].parent
for child in list(eb[DROPPED].children):
    child.use_connect = False
    child.parent = parent
eb.remove(eb[DROPPED])
# the rest stands on the floor: the BVH's rest has the hips at the origin, and the client normalises the hips'
# position track by the rest hips height (`Normalize Position Tracks`), so the rest is lifted by its lowest joint
lift = -min(min(b.head.z, b.tail.z) for b in eb)
for b in eb:
    b.head.z += lift
    b.tail.z += lift
print(f"stand_from_bvh: rest lifted {lift:.3f} m (hips rest height {eb['Hips'].head.z:.3f} m)")
bpy.ops.object.mode_set(mode="POSE")
for pb in tgt.pose.bones:
    c = pb.constraints.new("COPY_TRANSFORMS")
    c.target = src
    c.subtarget = pb.name
    c.target_space = "WORLD"
    c.owner_space = "WORLD"
# 5. the hips' horizontal drift: the recording's mean ground-plane position and its linear trend over the loop are
# removed by moving the recording's own armature object against them, frame by frame, before the copy is baked
# (Blender's ground plane is x, y); the take's sway stays
src_hips = src.pose.bones["Hips"]
frames = list(range(f0, best + 1))
world = []
for f in frames:
    scene.frame_set(f)
    world.append((src.matrix_world @ src_hips.matrix).translation.copy())
n = len(frames)
mean = sum(world, Vector()) / n
ts = [(f - f0) / max(best - f0, 1) for f in frames]
tm = sum(ts) / n
slope = sum(((t - tm) * (w - mean) for t, w in zip(ts, world)), Vector()) / max(sum((t - tm) ** 2 for t in ts), 1e-9)
for i, f in enumerate(frames):
    drift = mean + slope * (ts[i] - tm)
    src.location = Vector((-drift.x, -drift.y, 0.0))
    src.keyframe_insert("location", frame=f)
print(f"stand_from_bvh: hips horizontal drift removed: mean ({mean.x:.3f}, {mean.y:.3f}) m, trend ({slope.x:.3f}, {slope.y:.3f}) m per loop")

bpy.ops.nla.bake(
    frame_start=f0,
    frame_end=best,
    only_selected=False,
    visual_keying=True,
    clear_constraints=True,
    use_current_action=True,
    bake_types={"POSE"},
)
bpy.ops.object.mode_set(mode="OBJECT")
action = tgt.animation_data.action
action.name = "Stand"

# 3. the client's names
for b in tgt.data.bones:
    b.name = "mixamorig_" + NAMES[b.name]
tgt.name = tgt.data.name = "stand_lib"

# 6. a one-triangle mesh skinned to the hips, and the export
mesh = bpy.data.meshes.new("stand_lib_skin")
h = tgt.data.bones["mixamorig_Hips"].head_local
mesh.from_pydata([h, h + Vector((0.01, 0, 0)), h + Vector((0, 0.01, 0))], [], [(0, 1, 2)])
ob = bpy.data.objects.new("stand_lib_skin", mesh)
scene.collection.objects.link(ob)
ob.parent = tgt
ob.vertex_groups.new(name="mixamorig_Hips").add([0, 1, 2], 1.0, "REPLACE")
mod = ob.modifiers.new("Armature", "ARMATURE")
mod.object = tgt
bpy.data.objects.remove(src, do_unlink=True)
for a in list(bpy.data.actions):
    if a != action:
        bpy.data.actions.remove(a)
for o in scene.objects:
    o.select_set(o in (tgt, ob))
scene.frame_start, scene.frame_end = f0, best
bpy.ops.export_scene.gltf(
    filepath=OUT,
    export_format="GLB",
    use_selection=True,
    export_animations=True,
    export_animation_mode="ACTIONS",
    export_force_sampling=True,
    export_frame_range=True,
    export_anim_slide_to_zero=True,
    export_def_bones=False,
)
print(f"stand_from_bvh: wrote {OUT}")
