# SPDX-License-Identifier: GPL-2.0-or-later
"""Render one cast member's 2D sprite set from the SAME generated 3D character (step-26 §9 as amended by the
operator's ruling of 2026-10-19; PR S26-c1, `pr-s26c1-cast-client.md` D-5; `ARC-9` note `source_type: rendered`).

Run headless (Blender 5.2):
    blender --background --factory-startup --python-exit-code 1 \
        --python tools/character_pipeline/blender/render_sprites.py -- \
        <gltf|glb> <out_dir> <key> <stature_m> [--yaw 45] [--elev 30] [--stride 15]
        [--clearance-mm 30] [--elbow-clearance-mm 12] [--elbow 13] [--abduct-max 35] [--forearm-in-max 20]
        [--debug]

The committed default-pack sprites (`presentation/mineworld-default/2D/art/cast/`) were rendered with the
defaults, from `clients/3d-spike/assets/characters/cast/<key>/<key>.gltf` at the brief's stature, and Vera's
from `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` at 1.767 m.

Arms (operator bug of 2026-10-19: arms pressed to the body, palms forward). The stand is solved per body from
the evaluated mesh, with no per-character constants:
- the forearm is brought back in line with the upper arm (the rig's carrying angle, measured from the joints);
- each upper arm is abducted, by bisection, just enough that its forearm+hand-dominant vertices clear the
  spine/chest/hips/upper-leg-dominant vertices by --clearance-mm (--elbow-clearance-mm in the proximal 30% of
  the forearm, where the elbow of a relaxed arm sits next to the waist);
- the elbow bends --elbow degrees, the forearm twists (pronation/supination) so the palm aims medially, at the
  thigh, and the fingers curl lightly toward the palm. The palm is measured on the hand-dominant vertices
  (`measure_palms`), not on the knuckle joints, which on several bodies are a quarter turn off the mesh;
- each stride frame swings the arms from that stand; a frame whose swung arm comes inside the targets gets
  the least extra abduction that clears it (`solve_frame`).
Log lines, one JSON object each: `STAND_CLEARANCE` per side (`before_mm` = the previous renderer's fixed 8 degree
hang, `after_mm` = the solved stand, both the minimum over all forearm+hand vertices, negative = inside the
body; `*_distal_mm` past the elbow region), and `STRIDE_CLEARANCE` per rendered frame.

Writes <out_dir>/<key>_front.png, _front_b, _back, _back_b and one ARC-9 style JSON sidecar per file.

Conventions matched (clients/2d/PRESENTATION.md §3, people.gd `_animate`):
- the sprite faces screen RIGHT; the client mirrors with scale.x for leftward motion;
- `_front` = walking toward the viewer and right (screen down-right = plan east in the 2:1 isometric
  projection), `_back` = walking away and right; `_b` = the other stride (the other leg forward);
- the camera is orthographic at 30 degrees elevation (2:1 isometric: sin 30 = 0.5) and the walk direction
  is 45 degrees off the view axis, as plan east is in that projection;
- the image is cropped to its opaque rect and normalized so pixel height = stature_m * 64; the anchor is
  the projected root (between the feet) on x and the bottom row on y.
"""

import bpy
import hashlib
import json
import math
import os
import sys
import numpy as np
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1 :]
SRC, OUT, KEY, STATURE = argv[0], argv[1], argv[2], float(argv[3])


def opt(name, default):
    return float(argv[argv.index(name) + 1]) if name in argv else default


YAW = opt("--yaw", 45.0)
ELEV = opt("--elev", 30.0)
STRIDE = opt("--stride", 15.0)
# relaxed stand (measured per body, see `solve_stand`): forearm+hand clear torso, hips and thighs by
# CLEARANCE_MM; the elbow bends ELBOW_DEG; the forearm turns the palm toward the thigh; the fingers curl lightly
ELBOW_CLEARANCE_MM = opt("--elbow-clearance-mm", 12.0)  # within the elbow region the gap may be smaller
ELBOW_REGION = 0.3  # the proximal fraction of the forearm counted as the elbow region
# an arm vertex nearer than this and behind most of its nearest body normals reads as inside (negative). Kept
# below every target, so the sign only reports contact and never decides the solve: the inner faces of open
# garments face inward and would otherwise read as deep penetration
PENETRATION_M = 0.01
CLEARANCE_MM = opt("--clearance-mm", 30.0)
ELBOW_DEG = opt("--elbow", 13.0)
ABDUCT_MAX = opt("--abduct-max", 35.0)  # upper bound of the search, degrees of the upper arm from vertical
FOREARM_IN_MAX = opt("--forearm-in-max", 20.0)  # upper bound: forearm turned back in toward the thigh
STRIDE_ABDUCT_MAX = 15.0  # upper bound of the extra abduction a stride frame may need to keep clear
CURL_DEG = (8.0, 14.0, 10.0)  # proximal, middle, distal finger segments (thumb: half)
LEGACY_HANG = 8.0  # the previous renderer's fixed hang (arms pressed to the body); measured for the log only
DEBUG = "--debug" in argv
SS = 4  # supersampling factor
OUTLINE = (0.16, 0.12, 0.10)  # soft dark-brown contour, like the generated sprites' line
os.makedirs(OUT, exist_ok=True)
TARGET_H = round(STATURE * 64)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=SRC)
scene = bpy.context.scene
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
# only the skinned meshes in the scene (the glTF importer also creates an unlinked "Icosphere" bone shape)
meshes = [o for o in scene.objects if o.type == "MESH" and o.find_armature() is not None]
for o in scene.objects:
    if o.type == "MESH" and o not in meshes:
        o.hide_render = True

# ---- look: matte, even, soft (the 2D sprites are flat painted, lit from the upper left) -------------------
engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
scene.render.engine = "BLENDER_EEVEE_NEXT" if "BLENDER_EEVEE_NEXT" in engines else "BLENDER_EEVEE"
scene.render.film_transparent = True
scene.render.image_settings.file_format = "PNG"
scene.render.image_settings.color_mode = "RGBA"
scene.view_settings.view_transform = "Standard"
scene.view_settings.look = "None"
w = bpy.data.worlds.new("w")
scene.world = w
w.use_nodes = True
bg = w.node_tree.nodes["Background"]
bg.inputs[0].default_value = (1.0, 0.97, 0.92, 1)
bg.inputs[1].default_value = 0.95
for m in bpy.data.materials:
    if not m.use_nodes:
        continue
    for n in m.node_tree.nodes:
        if n.type == "BSDF_PRINCIPLED":
            n.inputs["Roughness"].default_value = 1.0
            if "Specular IOR Level" in n.inputs:
                n.inputs["Specular IOR Level"].default_value = 0.1
            if "Metallic" in n.inputs and not n.inputs["Metallic"].is_linked:
                n.inputs["Metallic"].default_value = 0.0


def add_sun(name, energy, direction, angle_deg):
    s = bpy.data.objects.new(name, bpy.data.lights.new(name, "SUN"))
    s.data.energy = energy
    s.data.angle = math.radians(angle_deg)
    s.rotation_euler = Vector(direction).to_track_quat("-Z", "Z").to_euler()
    scene.collection.objects.link(s)
    return s


key_sun = add_sun(
    "key", 2.6, (0, 0, -1), 25
)  # re-aimed per view so it comes from the upper left of the frame
fill_sun = add_sun("fill", 0.6, (0, 0, -1), 40)

cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.data.type = "ORTHO"
cam.data.clip_start = 0.01
cam.data.clip_end = 100

# ---- pose: a walk stride, applied as world-space rotations about each joint ------------------------------
PB = {pb.name.split(":")[-1]: pb for pb in arm.pose.bones}
WM = arm.matrix_world


def update():
    bpy.context.view_layer.update()


def rotate_world(bone, axis, deg):
    """Rotate pose bone `bone` by `deg` about world `axis` through its head."""
    pb = PB[bone]
    head_w = WM @ pb.head
    rot = (
        Matrix.Translation(head_w)
        @ Matrix.Rotation(math.radians(deg), 4, Vector(axis))
        @ Matrix.Translation(-head_w)
    )
    pb.matrix = WM.inverted() @ rot @ WM @ pb.matrix
    update()


def bone_dir(bone):
    pb = PB[bone]
    return ((WM @ pb.tail) - (WM @ pb.head)).normalized()


def reset_pose():
    for pb in arm.pose.bones:
        pb.matrix_basis = Matrix.Identity(4)
    update()


# Facing: glTF +Z forward imports as Blender -Y. Lateral axis = X.
FWD = Vector((0, -1, 0))


SIDES = (("Left", 1), ("Right", -1))  # sgn: the side's lateral direction on world X
FINGERS = ("Index", "Middle", "Ring", "Pinky")


def abduct(side, sgn, deg):
    """Swing the upper arm in the coronal plane (about the forward axis) to `deg` from vertical, outward."""
    d = bone_dir(side + "Arm")
    ang = math.degrees(math.atan2(d.x, -d.z))  # 0 = straight down; + = toward world +X
    rotate_world(side + "Arm", (0, 1, 0), ang - sgn * deg)


def dominant_sets():
    """Per skinned mesh, vertex indices by dominant bone: each side's arm (forearm, hand, fingers) and hand
    (hand, fingers), and the body (spine, chest, hips, upper legs) the hanging arm must clear."""
    body_bones = {"Hips", "Spine", "Spine1", "Spine2", "LeftUpLeg", "RightUpLeg"}
    sets = []
    for o in meshes:
        names = {g.index: g.name.split(":")[-1] for g in o.vertex_groups}
        arm_idx = {"Left": [], "Right": []}
        hand_idx = {"Left": [], "Right": []}
        body_idx = []
        for v in o.data.vertices:
            if not v.groups:
                continue
            g = max(v.groups, key=lambda e: e.weight)
            bone = names.get(g.group, "")
            if bone in body_bones:
                body_idx.append(v.index)
            for side, _ in SIDES:
                if bone.startswith(side + "Hand"):
                    hand_idx[side].append(v.index)
                if bone == side + "ForeArm" or bone.startswith(side + "Hand"):
                    arm_idx[side].append(v.index)
        sets.append((o, arm_idx, hand_idx, body_idx))
    return sets


def hand_rotation(side):
    """World rotation of the hand bone from its rest to its current pose."""
    pb = PB[side + "Hand"]
    return (WM @ pb.matrix @ pb.bone.matrix_local.inverted() @ WM.inverted()).to_3x3()


def measure_palms(sets):
    """Per side, the palm normal and finger direction of the rest mesh (call in the rest pose).

    The rig's knuckle joints do not follow the generated hand (on several bodies they are a quarter turn off
    the mesh), so the palm is measured on the hand-dominant vertices: the finger direction is the principal
    axis pointing away from the wrist, the palm plane's normal is the least-variance axis, and its sign is the
    side the hand is cupped toward (fingers flexed and palm hollowed toward the palm: the quadratic terms of
    the vertices' height over the plane, across and along the hand)."""
    dg = bpy.context.evaluated_depsgraph_get()
    palms = {}
    for side, _ in SIDES:
        pts = []
        for o, _, hand_idx, _ in sets:
            me = o.evaluated_get(dg).to_mesh()
            pts += [tuple(o.matrix_world @ me.vertices[i].co) for i in hand_idx[side]]
            o.evaluated_get(dg).to_mesh_clear()
        p = np.array(pts)
        c = p.mean(axis=0)
        _, _, vt = np.linalg.svd(p - c, full_matrices=False)
        f = vt[0] * np.sign(vt[0] @ (c - np.array(WM @ PB[side + "Hand"].head)))
        n = vt[2]
        a, s, z = (p - c) @ np.cross(n, f), (p - c) @ f, (p - c) @ n
        k = np.linalg.lstsq(np.stack([a * a, s * s, a * s, a, s, np.ones_like(a)], 1), z, rcond=None)[0]
        palms[side] = {
            "palm": Vector(n * np.sign(k[0] + k[1])),
            "fingers": Vector(f),
            "cup": (round(float(k[0]), 2), round(float(k[1]), 2)),
        }
    return palms


def signed_angle(a, b, axis):
    """Angle turning `a` onto `b` about `axis`, both projected on the plane normal to `axis` (degrees)."""
    pa = (a - axis * a.dot(axis)).normalized()
    pb = (b - axis * b.dot(axis)).normalized()
    return math.degrees(math.atan2(axis.dot(pa.cross(pb)), pa.dot(pb)))


def relaxed_stand(stand):
    """The standing pose: per side `stand[side] = (abduct_deg, forearm_in_deg)` — the upper arm abducted, the
    forearm brought back in toward the thigh (taking out the rig's carrying angle), a slight elbow bend, palms
    toward the thighs, fingers lightly curled. Every axis is measured from the posed joints and the measured
    palm (PALMS)."""
    info = {}
    for side, sgn in SIDES:
        abduct(side, sgn, stand[side][0])
        fa = side + "ForeArm"
        rotate_world(fa, (0, 1, 0), sgn * stand[side][1])  # + about forward turns the left forearm medially
        # elbow: flex the forearm toward the front, about the axis normal to the upper arm and forward
        flex_axis = bone_dir(side + "Arm").cross(FWD).normalized()
        rotate_world(fa, flex_axis, ELBOW_DEG)
        # forearm twist (pronation/supination about its own axis): aim the palm medially, at the thigh
        medial = Vector((-sgn, 0, 0))
        twist = 0.0
        for _ in range(3):  # the hand is not exactly on the forearm axis: converge in a few passes
            axis = bone_dir(fa)
            t = signed_angle(hand_rotation(side) @ PALMS[side]["palm"], medial, axis)
            rotate_world(fa, axis, t)
            twist += t
        r = hand_rotation(side)
        n = r @ PALMS[side]["palm"]
        # light finger curl toward the palm, proximal to distal, about the measured across-palm axis
        curl_axis = (r @ PALMS[side]["fingers"]).cross(n).normalized()
        for finger in FINGERS + ("Thumb",):
            scale = 0.5 if finger == "Thumb" else 1.0
            for seg, deg in enumerate(CURL_DEG, start=1):
                b = f"{side}Hand{finger}{seg}"
                if b in PB:
                    rotate_world(b, curl_axis, scale * deg)
        info[side] = {
            "twist_deg": round(twist, 1),
            "palm_off_medial_deg": round(math.degrees(n.angle(medial)), 1),
            "palm_cup": PALMS[side]["cup"],
        }
    return info


def clearance_mm(sets):
    """Per side, the clearance of the forearm+hand vertices from the body vertices of the evaluated mesh (mm):
    `all` = the minimum over every arm vertex; `distal` = over the vertices past the elbow region (the proximal
    ELBOW_REGION of the forearm); `margin` = the minimum of (distance - that vertex's target), the targets being
    ELBOW_CLEARANCE_MM in the elbow region and CLEARANCE_MM elsewhere. A distance is negative (penetration) when
    the arm vertex is within PENETRATION_M of the body and behind most of its nearest body vertices' normals."""
    from mathutils.kdtree import KDTree

    dg = bpy.context.evaluated_depsgraph_get()
    body, arms = [], {"Left": [], "Right": []}
    for o, arm_idx, _, body_idx in sets:
        me = o.evaluated_get(dg).to_mesh()
        mw = o.matrix_world
        nm = mw.to_3x3()
        body += [(mw @ me.vertices[i].co, (nm @ me.vertices[i].normal).normalized()) for i in body_idx]
        for side in arms:
            arms[side] += [mw @ me.vertices[i].co for i in arm_idx[side]]
        o.evaluated_get(dg).to_mesh_clear()
    kd = KDTree(len(body))
    for i, (co, _) in enumerate(body):
        kd.insert(co, i)
    kd.balance()
    out = {}
    for side, pts in arms.items():
        elbow, axis = joint(side + "ForeArm"), bone_dir(side + "ForeArm")
        length = (joint(side + "Hand") - elbow).length
        res = {"all": math.inf, "distal": math.inf, "margin": math.inf}
        where = None
        for p in pts:
            near = kd.find_n(p, 6)
            dist = near[0][2]
            if (
                dist < PENETRATION_M
                and sum((p - co).dot(body[i][1]) < 0 for co, i, _ in near) > len(near) / 2
            ):
                dist = -dist
            mm = dist * 1000.0
            in_elbow = (p - elbow).dot(axis) < ELBOW_REGION * length
            if mm < res["all"]:
                res["all"], where = mm, (p, near[0][0])
            if not in_elbow:
                res["distal"] = min(res["distal"], mm)
            res["margin"] = min(res["margin"], mm - (ELBOW_CLEARANCE_MM if in_elbow else CLEARANCE_MM))
        out[side] = {k: round(v, 1) for k, v in res.items()}
        if DEBUG and where:
            print(
                "CLOSEST",
                side,
                out[side],
                tuple(round(x, 3) for x in where[0]),
                tuple(round(x, 3) for x in where[1]),
            )
    return out


def joint(bone):
    return WM @ PB[bone].head


def carrying_angle(side, sgn):
    """The rig's carrying angle: how far the forearm leans outward from the upper arm's line in the coronal
    plane (degrees, measured from the joints in the current pose; positive = hand outward of the elbow)."""

    def outward(d):
        return math.degrees(math.atan2(sgn * d.x, -d.z))

    return outward(bone_dir(side + "ForeArm")) - outward(bone_dir(side + "Arm"))


def solve_stand():
    """Per side: the forearm is brought back in line with the upper arm (the rig's carrying angle, capped at
    FOREARM_IN_MAX), then the smallest upper-arm abduction whose relaxed stand clears torso, hips and thighs
    (margin >= 0: CLEARANCE_MM for the forearm and hand, ELBOW_CLEARANCE_MM at the elbow) is found by bisection
    on the measured clearance. Also measures the previous renderer's clearance for the log."""
    global PALMS, SETS
    SETS = dominant_sets()
    reset_pose()
    PALMS = measure_palms(SETS)
    fin = {s: min(max(carrying_angle(s, sgn), 0.0), FOREARM_IN_MAX) for s, sgn in SIDES}
    for side, sgn in SIDES:
        abduct(side, sgn, LEGACY_HANG)
    before = clearance_mm(SETS)
    lo, hi = {s: 0.0 for s, _ in SIDES}, {s: ABDUCT_MAX for s, _ in SIDES}
    for _ in range(12):
        mid = {s: (lo[s] + hi[s]) / 2 for s, _ in SIDES}
        reset_pose()
        relaxed_stand({s: (mid[s], fin[s]) for s, _ in SIDES})
        c = clearance_mm(SETS)
        for s, _ in SIDES:
            if c[s]["margin"] >= 0:
                hi[s] = mid[s]
            else:
                lo[s] = mid[s]
    stand = {s: (hi[s], fin[s]) for s, _ in SIDES}
    reset_pose()
    info = relaxed_stand(stand)
    after = clearance_mm(SETS)
    for side, _ in SIDES:
        print(
            "STAND_CLEARANCE",
            json.dumps(
                {
                    "key": KEY,
                    "side": side,
                    "before_mm": before[side]["all"],
                    "after_mm": after[side]["all"],
                    "before_distal_mm": before[side]["distal"],
                    "after_distal_mm": after[side]["distal"],
                    "target_mm": CLEARANCE_MM,
                    "elbow_target_mm": ELBOW_CLEARANCE_MM,
                    "abduct_deg": round(stand[side][0], 2),
                    "forearm_in_deg": round(stand[side][1], 2),
                    "elbow_deg": ELBOW_DEG,
                    **info[side],
                }
            ),
        )
    return stand, after


def stride(sign):
    """sign=+1: left leg forward, right arm forward. sign=-1: the other stride."""
    a = STRIDE
    # swing about the lateral (X) axis; -deg about +X turns a downward bone toward -Y (forward)
    rotate_world("LeftUpLeg", (1, 0, 0), -sign * a)
    rotate_world("RightUpLeg", (1, 0, 0), sign * a)
    back_leg = "RightLeg" if sign > 0 else "LeftLeg"
    rotate_world(back_leg, (1, 0, 0), 0.7 * a)  # trailing knee bends (heel lifts behind)
    front_leg = "LeftLeg" if sign > 0 else "RightLeg"
    rotate_world(front_leg, (1, 0, 0), 0.15 * a)
    rotate_world("LeftArm", (1, 0, 0), sign * 0.8 * a)
    rotate_world("RightArm", (1, 0, 0), -sign * 0.8 * a)
    for f in ("LeftForeArm", "RightForeArm"):
        rotate_world(f, (1, 0, 0), -0.5 * a)  # elbows slightly bent forward
    # keep the soles on the ground: drop the hips by the leg shortening of the stride
    feet = [WM @ PB[s + "Foot"].head for s in ("Left", "Right")]
    return min(f.z for f in feet)


def evaluated_points():
    dg = bpy.context.evaluated_depsgraph_get()
    pts = []
    for o in meshes:
        if o.hide_render:
            continue
        ev = o.evaluated_get(dg)
        me = ev.to_mesh()
        mw = np.array(o.matrix_world)
        co = np.empty(len(me.vertices) * 3)
        me.vertices.foreach_get("co", co)
        co = co.reshape(-1, 3)
        co = co @ mw[:3, :3].T + mw[:3, 3]
        pts.append(co)
        ev.to_mesh_clear()
    return np.concatenate(pts)


def view_basis(view):
    a = math.radians(YAW)
    if view == "front":
        c = Vector((-math.sin(a), -math.cos(a), 0))
    else:
        c = Vector((-math.sin(a), math.cos(a), 0))
    e = math.radians(ELEV)
    back = (c * math.cos(e) + Vector((0, 0, 1)) * math.sin(e)).normalized()  # target -> camera
    d = -back
    right = d.cross(Vector((0, 0, 1))).normalized()
    up = right.cross(d).normalized()
    return back, right, up, d


def pose_frame(stand, sign):
    """The relaxed stand, then the stride, then the hips moved so the lowest foot joint keeps its height."""
    reset_pose()
    relaxed_stand(stand)
    foot_z0 = min((WM @ PB[s + "Foot"].head).z for s in ("Left", "Right"))
    fz = stride(sign)
    PB["Hips"].matrix = WM.inverted() @ Matrix.Translation((0, 0, foot_z0 - fz)) @ WM @ PB["Hips"].matrix
    update()


def solve_frame(sign):
    """The stride swings each arm from the stand; where a swung arm comes closer than the targets (a hand
    passing the hip or the seat), that side's abduction grows by the least that clears it again (bisection,
    up to STRIDE_ABDUCT_MAX). Returns the frame's stand."""
    pose_frame(STAND, sign)
    c = clearance_mm(SETS)
    fail = [s for s, _ in SIDES if c[s]["margin"] < 0]
    if not fail:
        return STAND
    lo, hi = {s: 0.0 for s in fail}, {s: STRIDE_ABDUCT_MAX for s in fail}
    for _ in range(10):
        mid = {s: (lo[s] + hi[s]) / 2 for s in fail}
        pose_frame({s: (STAND[s][0] + mid.get(s, 0.0), STAND[s][1]) for s, _ in SIDES}, sign)
        c = clearance_mm(SETS)
        for s in fail:
            if c[s]["margin"] >= 0:
                hi[s] = mid[s]
            else:
                lo[s] = mid[s]
    return {s: (STAND[s][0] + hi.get(s, 0.0), STAND[s][1]) for s, _ in SIDES}


def render_view(view, frame_b, out_png):
    sign = -1 if frame_b else 1
    if sign not in FRAME_STAND:
        FRAME_STAND[sign] = solve_frame(sign)
    pose_frame(FRAME_STAND[sign], sign)
    c = clearance_mm(SETS)
    print(
        "STRIDE_CLEARANCE",
        json.dumps(
            {
                "key": KEY,
                "frame": "b" if frame_b else "a",
                "view": view,
                **{s + "_mm": v["all"] for s, v in c.items()},
                **{s + "_abduct_deg": round(FRAME_STAND[sign][s][0], 2) for s, _ in SIDES},
            }
        ),
    )
    back, right, up, d = view_basis(view)
    pts = evaluated_points()
    u = pts @ np.array(right)
    v = pts @ np.array(up)
    umin, umax, vmin, vmax = u.min(), u.max(), v.min(), v.max()
    inner_h = TARGET_H - 2  # 1 px outline added at final scale, top and bottom
    p = (vmax - vmin) / (inner_h * SS)  # world units per supersampled pixel
    pad = 3 * SS
    hpx = inner_h * SS + 2 * pad
    wpx = int(math.ceil((umax - umin) / p / SS)) * SS + 2 * pad
    scene.render.resolution_x, scene.render.resolution_y = wpx, hpx
    scene.render.resolution_percentage = 100
    cam.data.ortho_scale = p * max(wpx, hpx)
    cu, cv = (umin + umax) / 2, (vmin + vmax) / 2
    centre = Vector(right) * cu + Vector(up) * cv
    # put the camera on the line through the bbox centre, far back along the view direction
    centre_depth = Vector(np.mean(pts, axis=0)).dot(d)
    cam.location = centre + d * (centre_depth - 20.0)
    cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
    cam.rotation_euler = Matrix((right, up, -d)).transposed().to_euler()
    # light from the upper left of the frame, slightly toward the camera; fill from the right
    key_dir = (-Vector(right) * 0.8 - Vector((0, 0, 1)) * 1.2 + d * 0.6).normalized()
    key_sun.rotation_euler = key_dir.to_track_quat("-Z", "Z").to_euler()
    fill_dir = (Vector(right) * 0.7 - Vector((0, 0, 1)) * 0.5 + d * 0.8).normalized()
    fill_sun.rotation_euler = fill_dir.to_track_quat("-Z", "Z").to_euler()
    # anchor: the projected root on the ground (mean of the two foot joints dropped to the floor)
    feet = [WM @ PB[s + "Foot"].head for s in ("Left", "Right")]
    root = (feet[0] + feet[1]) / 2
    root.z = vmin  # unused for x
    root_u = Vector(root).dot(Vector(right))
    anchor_x_ss = (root_u - (cu - p * wpx / 2)) / p
    raw = out_png + ".raw.png"
    scene.render.filepath = raw
    bpy.ops.render.render(write_still=True)
    return raw, anchor_x_ss


def load_rgba(path):
    img = bpy.data.images.load(path)
    w, h = img.size
    a = np.array(img.pixels[:], dtype=np.float32).reshape(h, w, 4)[::-1]  # top-down rows
    bpy.data.images.remove(img)
    return a


def save_rgba(arr, path):
    h, w = arr.shape[:2]
    img = bpy.data.images.new(os.path.basename(path), w, h, alpha=True)
    img.colorspace_settings.name = "sRGB"
    img.pixels.foreach_set(np.ascontiguousarray(arr[::-1]).reshape(-1).astype(np.float32))
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)


def normalize(raw, anchor_x_ss):
    a = load_rgba(raw)
    h, w = a.shape[:2]
    a = a[: h - h % SS, : w - w % SS]
    rgb, al = a[..., :3] * a[..., 3:4], a[..., 3:4]  # premultiply

    def box(x):
        return x.reshape(x.shape[0] // SS, SS, x.shape[1] // SS, SS, -1).mean(axis=(1, 3))

    prgb, pal = box(rgb), box(al)
    rgb = np.where(pal > 1e-6, prgb / np.maximum(pal, 1e-6), 0)
    al = pal
    al[al < 6 / 255] = 0
    # 1 px soft contour under the figure
    hh, ww = al.shape[:2]
    padded = np.pad(al[..., 0], 1)
    dil = np.max(
        [padded[1 + dy : 1 + dy + hh, 1 + dx : 1 + dx + ww] for dy in (-1, 0, 1) for dx in (-1, 0, 1)], axis=0
    )
    ring = np.clip(dil, 0, 1)[..., None]
    out_a = al + ring * (1 - al)
    out_rgb = np.where(
        out_a > 1e-6, (rgb * al + np.array(OUTLINE) * ring * (1 - al)) / np.maximum(out_a, 1e-6), 0
    )
    out = np.concatenate([out_rgb, out_a], axis=2)
    ys, xs = np.nonzero(out[..., 3] > 0)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    out = out[y0:y1, x0:x1]
    anchor = [round(anchor_x_ss / SS - x0, 1), float(y1 - y0)]
    return np.clip(out, 0, 1), anchor


sha = hashlib.sha256(open(SRC, "rb").read()).hexdigest()
bin_path = os.path.splitext(SRC)[0] + ".bin"
sha_bin = hashlib.sha256(open(bin_path, "rb").read()).hexdigest() if os.path.exists(bin_path) else None
STAND, CLEAR = solve_stand()
FRAME_STAND = {}  # stride sign -> the stand that frame swings from (see solve_frame)
results = {}
for view in ("front", "back"):
    for frame_b in (False, True):
        name = f"{KEY}_{view}{'_b' if frame_b else ''}"
        png = os.path.join(OUT, name + ".png")
        raw, ax = render_view(view, frame_b, png)
        img, anchor = normalize(raw, ax)
        save_rgba(img, png)
        if not DEBUG:
            os.remove(raw)
        side = {
            "source_type": "rendered",
            "status": "candidate",
            "render": {
                "source": "render of the generated 3D character (step-26 route D+ export), same Person as the 3D client",
                "character": KEY,
                "source_file": os.path.basename(SRC),
                "source_sha256": sha,
                "source_bin_sha256": sha_bin,
                "tool": "Blender "
                + bpy.app.version_string
                + " ("
                + scene.render.engine
                + "), tools/character_pipeline/blender/render_sprites.py",
                "date": "2026-10-19",
                "camera": {
                    "projection": "orthographic",
                    "elevation_deg": ELEV,
                    "yaw_from_walk_deg": YAW,
                    "view": view,
                    "faces": "screen right",
                },
                "pose": {
                    "stride_deg": STRIDE,
                    "frame": "b" if frame_b else "a",
                    "lead_leg": ("right" if frame_b else "left"),
                    "arms": "relaxed stand measured per body: upper arm abducted until forearm and hand clear "
                    "torso, hips and thighs by the target (the elbow region by elbow_target), elbow bent, palms "
                    "turned to the thighs, fingers lightly curled; the stride swings the arms from that hang, "
                    "abducting further only where a swung arm would come inside the targets",
                    "stand_abduction_deg": {s.lower(): round(STAND[s][0], 2) for s, _ in SIDES},
                    "frame_abduction_deg": {
                        s.lower(): round(FRAME_STAND[-1 if frame_b else 1][s][0], 2) for s, _ in SIDES
                    },
                    "forearm_in_deg": {s.lower(): round(STAND[s][1], 2) for s, _ in SIDES},
                    "elbow_deg": ELBOW_DEG,
                    "stand_clearance_mm": {
                        "target": CLEARANCE_MM,
                        **{s.lower(): CLEAR[s]["all"] for s, _ in SIDES},
                        "elbow_target": ELBOW_CLEARANCE_MM,
                        **{s.lower() + "_past_elbow": CLEAR[s]["distal"] for s, _ in SIDES},
                    },
                },
                "lighting": "matte, warm ambient 0.95 + key sun upper-left 2.6 + fill 0.6, film transparent",
            },
            "postprocess": {
                "supersample": SS,
                "downsample": "premultiplied box",
                "outline_px": 1,
                "outline_rgb": list(OUTLINE),
                "normalized": True,
                "height_m": STATURE,
                "pixel_height": int(img.shape[0]),
                "rule": "pixel height = stature_m * 64",
                "anchor": anchor,
                "size": [int(img.shape[1]), int(img.shape[0])],
            },
            "human_curated": False,
            "license": {
                "redistribution_allowed": True,
                "note": "derived from the project's own generated character; inherits its source provenance",
            },
        }
        with open(os.path.join(OUT, name + ".provenance.json"), "w") as f:
            json.dump(side, f, indent=2)
        results[name] = {"size": [int(img.shape[1]), int(img.shape[0])], "anchor": anchor}
        print("SPRITE", name, results[name])
with open(os.path.join(OUT, "summary.json"), "w") as f:
    json.dump({"key": KEY, "stature_m": STATURE, "target_px": TARGET_H, "sprites": results}, f, indent=2)
