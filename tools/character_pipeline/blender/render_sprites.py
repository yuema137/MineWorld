# SPDX-License-Identifier: GPL-2.0-or-later
"""Render one cast member's 2D sprite set from the SAME generated 3D character (step-26 §9 as amended by the
operator's ruling of 2026-10-19; PR S26-c1, `pr-s26c1-cast-client.md` D-5; `ARC-9` note `source_type: rendered`).

Run headless (Blender 5.2):
    blender --background --factory-startup --python-exit-code 1 \
        --python tools/character_pipeline/blender/render_sprites.py -- \
        <gltf|glb> <out_dir> <key> <stature_m> --pose <pose.json> [--yaw 45] [--elev 30] [--debug]

The committed default-pack sprites (`presentation/mineworld-default/2D/art/cast/`) were rendered with the
defaults, from `clients/3d-spike/assets/characters/cast/<key>/<key>.gltf` at the brief's stature, and Vera's
from `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` at 1.767 m.

The pose (PR S26-c-pose part F, AC-6): the 3D client's own pose of this export, dumped by
`cast.py ground --dump-pose` (`<pose.json>`: each bone's change from rest in the glTF's frame for the `Walk`
clip's two largest foot separations, after the client's retarget, `Posture` and ground contact). Nothing is posed
or solved here, so the 2D sprite and the 3D body are one pose; the sidecar names the clip and the pose file.

Writes <out_dir>/<key>_front.png, _front_b, _back, _back_b and one ARC-9 style JSON sidecar per file.

Conventions matched (clients/2d/PRESENTATION.md §3, people.gd `_animate`):
- the sprite faces screen RIGHT; the client mirrors with scale.x for leftward motion;
- `_front` = walking toward the viewer and right (screen down-right = plan east in the 2:1 isometric
  projection), `_back` = walking away and right; `_b` = the other stride (the other leg forward);
- the camera is orthographic at 30 degrees elevation (2:1 isometric: sin 30 = 0.5) and the walk direction
  is 45 degrees off the view axis, as plan east is in that projection;
- the image is cropped to its opaque rect and normalized so pixel height = stature_m * 64; the anchor is
  the projected root (between the feet) on x and the bottom row on y.

With `--walk-frames N` (PR vf-2d-walk-cycle; driven by `cast.py walk-sheets`) it writes instead one atlas,
<out_dir>/<key>_walk.png, and its sidecar <key>_walk.json: five views (`SHEET_VIEWS`, 0°…180° between the walk and
the direction toward the camera) as rows, the standing pose and N poses of the dumped `Walk` cycle (`walk_cycle`)
as columns, every cell one size, one scale (the standing body's) and one anchor (the armature origin on the floor).
The sidecar also records the planted feet's slip as the client draws the sheet (`foot_drift`).
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
# the pose the 3D client draws for this export, dumped by `cast.py ground --dump-pose` (see `pose_frame`)
POSE_JSON = argv[argv.index("--pose") + 1]
DEBUG = "--debug" in argv
# --walk-frames N: the 2D walk sheet (2D walk cycle design) -- N frames of one whole `Walk` cycle per view plus the
# standing frame, every frame of a view at one scale, one size and one anchor, instead of the two stride frames
# (not `--cycle`: Blender's own parser reads it as an abbreviation of its `--cycles-*` options)
CYCLE = int(opt("--walk-frames", 0))
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

# ---- pose: the client's own, dumped by the ground probe (`cast.py ground --dump-pose`) ------------------------
# Each bone's change from its rest in the glTF's own frame (Y-up, the body facing +Z), as the 3D client poses this
# export: the stride frames are the `Walk` clip's largest left-forward and right-forward foot separations, after the
# client's retarget, Posture and ground contact. Nothing is posed here: the renderer only draws that pose, so the
# 2D sprite and the 3D body are one pose (PR S26-c-pose part F, AC-6).
PB = {pb.name.split(":")[-1]: pb for pb in arm.pose.bones}
WM = arm.matrix_world
POSE = json.load(open(POSE_JSON))
# glTF (x, y, z) -> Blender (x, -z, y)
C = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))


def update():
    bpy.context.view_layer.update()


def pose_frame(name):
    """Every bone, parents first, to its dumped change from rest: world = delta @ rest world."""
    for pb in arm.pose.bones:
        pb.matrix_basis = Matrix.Identity(4)
    update()
    frame = POSE[name] if isinstance(name, str) else name
    order = sorted(arm.pose.bones, key=lambda p: len(p.parent_recursive))
    for pb in order:
        m = frame.get(pb.name)
        if m is None:
            continue
        delta = Matrix((m[0:4], m[4:8], m[8:12], (0, 0, 0, 1)))
        delta_b = C @ delta @ C.inverted()
        pb.matrix = WM.inverted() @ delta_b @ WM @ pb.bone.matrix_local
        update()


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
    """The camera frame for `view`: "front" (YAW off the walk, toward the camera), "back" (YAW off walking away), or
    a number, the angle in degrees between the walk direction and the direction toward the camera (0 walks straight
    at the camera, 90 is a profile walking screen-right, 180 walks straight away). Every view faces screen right."""
    if isinstance(view, str):
        a = math.radians(YAW if view == "front" else 180.0 - YAW)
    else:
        a = math.radians(view)
    c = Vector((-math.sin(a), -math.cos(a), 0))
    e = math.radians(ELEV)
    back = (c * math.cos(e) + Vector((0, 0, 1)) * math.sin(e)).normalized()  # target -> camera
    d = -back
    right = d.cross(Vector((0, 0, 1))).normalized()
    up = right.cross(d).normalized()
    return back, right, up, d


def aim(view, cu, cv, p, wpx, hpx, centre_depth):
    """The camera and the two suns for `view`, framing the box centred at (cu, cv) at p units per pixel."""
    back, right, up, d = view_basis(view)
    scene.render.resolution_x, scene.render.resolution_y = wpx, hpx
    scene.render.resolution_percentage = 100
    cam.data.ortho_scale = p * max(wpx, hpx)
    centre = Vector(right) * cu + Vector(up) * cv
    # put the camera on the line through the bbox centre, far back along the view direction
    cam.location = centre + d * (centre_depth - 20.0)
    cam.rotation_euler = Matrix((right, up, -d)).transposed().to_euler()
    # light from the upper left of the frame, slightly toward the camera; fill from the right
    key_dir = (-Vector(right) * 0.8 - Vector((0, 0, 1)) * 1.2 + d * 0.6).normalized()
    key_sun.rotation_euler = key_dir.to_track_quat("-Z", "Z").to_euler()
    fill_dir = (Vector(right) * 0.7 - Vector((0, 0, 1)) * 0.5 + d * 0.8).normalized()
    fill_sun.rotation_euler = fill_dir.to_track_quat("-Z", "Z").to_euler()


def render_view(view, frame_b, out_png):
    pose_frame("stride_b" if frame_b else "stride_a")
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
    cu, cv = (umin + umax) / 2, (vmin + vmax) / 2
    aim(view, cu, cv, p, wpx, hpx, Vector(np.mean(pts, axis=0)).dot(d))
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
    out = soften(raw)
    ys, xs = np.nonzero(out[..., 3] > 0)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    out = out[y0:y1, x0:x1]
    anchor = [round(anchor_x_ss / SS - x0, 1), float(y1 - y0)]
    return np.clip(out, 0, 1), anchor


def soften(raw):
    """The supersampled render down to its final size (premultiplied box), with the 1 px contour; not cropped."""
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
    return np.concatenate([out_rgb, out_a], axis=2)


sha = hashlib.sha256(open(SRC, "rb").read()).hexdigest()
bin_path = os.path.splitext(SRC)[0] + ".bin"
sha_bin = hashlib.sha256(open(bin_path, "rb").read()).hexdigest() if os.path.exists(bin_path) else None



# ---- the walk sheet (--walk-frames N; PR vf-2d-walk-cycle) -------------------------------------------------------
# Five rendered views, each facing screen right; the client mirrors them for the three leftward facings, so a person
# has eight facings. The number is the angle between the walk and the direction toward the camera (`view_basis`).
SHEET_VIEWS = (("down", 0.0), ("down_right", 45.0), ("right", 90.0), ("up_right", 135.0), ("up", 180.0))
# a planted foot may float this far above its lowest point and still count as planted (floor.float.planted)
PLANTED_M = 0.010


def foot_lead():
    """How far the left foot is ahead of the right, along the body's facing (glTF +Z is Blender -Y)."""
    return (WM @ PB["RightFoot"].head).y - (WM @ PB["LeftFoot"].head).y


def cycle_frames(n):
    """N poses evenly spaced over the dumped `Walk` cycle, starting at its largest left-foot-forward stride, so
    frame 0 of every body is the same gait phase (the client's frame index is the phase of the cycle)."""
    cyc = POSE["walk_cycle"]
    leads = []
    for f in cyc:
        pose_frame(f)
        leads.append(foot_lead())
    start = int(np.argmax(leads))
    return [cyc[(start + round(i * len(cyc) / n)) % len(cyc)] for i in range(n)]


def foot_masks():
    """Per side, which of `evaluated_points()`'s vertices belong to the foot: the vertex's heaviest group is that
    side's foot or toe bone (PIPELINE.md G-4w's foot set)."""
    masks = {"Left": [], "Right": []}
    for o in meshes:
        if o.hide_render:
            continue
        names = {g.index: g.name.split(":")[-1] for g in o.vertex_groups}
        side = np.full(len(o.data.vertices), "", dtype=object)
        for v in o.data.vertices:
            if v.groups:
                g = max(v.groups, key=lambda e: e.weight)
                name = names.get(g.group, "")
                for s in ("Left", "Right"):
                    if name.startswith(s) and ("Foot" in name or "Toe" in name):
                        side[v.index] = s
        for s in masks:
            masks[s].append(side == s)
    return {s: np.concatenate(m) for s, m in masks.items()}


def foot_drift(frames):
    """How far a planted foot slides against the floor from one drawn frame to the next, as the 2D client draws
    the sheet (frames advance by distance, one frame per 1/N of a cycle). PIPELINE.md G-4w's reading on the drawn
    frames: the contact point is the foot's lowest vertex in frame k, followed to frame k+1, against a floor that
    moves back by 1/N of a cycle; a foot is planted while its lowest point is within PLANTED_M of the floor. A
    stance's slip is the sum over its steps. Returns, per foot, the worst stance and the worst single step in world
    mm, and how many drawn frames it is planted in. (Within one held
    frame the drawn foot moves with the body by up to 1/N of a cycle: `hold_slide_mm`, the price of N.)"""
    n = len(frames)
    scale = float(POSE.get("body_scale") or 1.0)
    step = float(POSE["walk_cycle_m"]) / scale / n  # export units of floor travel per frame
    masks = foot_masks()
    fwd = np.array((0.0, -1.0))
    feet = {s: [] for s in masks}
    for f in frames:
        pose_frame(f)
        pts = evaluated_points()
        for s, m in masks.items():
            feet[s].append(pts[m])
    floor = min(float(fs.min(axis=0)[2]) for v in feet.values() for fs in v)
    worst = {}
    for s, fs in feet.items():
        planted = [float(f[:, 2].min()) <= floor + PLANTED_M / scale for f in fs]
        slips = []
        for k in range(n):
            if planted[k] and planted[(k + 1) % n]:
                i = int(np.argmin(fs[k][:, 2]))
                moved = fs[(k + 1) % n][i, :2] - fs[k][i, :2] + fwd * step
                slips.append(float(np.linalg.norm(moved)))
            else:
                slips.append(None)
        # sum each stance's consecutive slips, cyclically
        run, best = 0.0, 0.0
        for v in slips + slips:
            run = 0.0 if v is None else run + v
            best = max(best, run)
        worst[s] = {
            "stance_mm": round(min(best, sum(v or 0.0 for v in slips)) * scale * 1000.0, 1),
            "step_max_mm": round(max((v for v in slips if v is not None), default=0.0) * scale * 1000.0, 1),
            "planted_frames": int(sum(planted)),
        }
    return worst


def render_sheet_view(theta, poses):
    """One view's frames, framed together at the standing body's scale. Returns the cropped frames and the anchor
    (the armature origin on the floor) in the crop's pixels; the anchor is the same for every frame, so the
    figure bobs and sways as the 3D body does instead of being re-centred per frame."""
    back, right, up, d = view_basis(theta)
    bounds = []
    for f in poses:
        pose_frame(f)
        pts = evaluated_points()
        bounds.append((pts @ np.array(right), pts @ np.array(up), Vector(np.mean(pts, axis=0)).dot(d)))
    p = (bounds[0][1].max() - bounds[0][1].min()) / ((TARGET_H - 2) * SS)  # the standing body sets the scale
    umin, umax = min(b[0].min() for b in bounds), max(b[0].max() for b in bounds)
    vmin, vmax = min(b[1].min() for b in bounds), max(b[1].max() for b in bounds)
    pad = 3 * SS
    wpx = int(math.ceil((umax - umin) / p / SS)) * SS + 2 * pad
    hpx = int(math.ceil((vmax - vmin) / p / SS)) * SS + 2 * pad
    cu, cv = (umin + umax) / 2, (vmin + vmax) / 2
    aim(theta, cu, cv, p, wpx, hpx, bounds[0][2])
    origin = Vector(WM.translation)
    ax = (origin.dot(Vector(right)) - (cu - p * wpx / 2)) / p / SS
    ay = ((cv + p * hpx / 2) - origin.dot(Vector(up))) / p / SS
    imgs = []
    for i, f in enumerate(poses):
        pose_frame(f)
        raw = os.path.join(OUT, f"{KEY}_{int(theta)}_{i}.raw.png")
        scene.render.filepath = raw
        bpy.ops.render.render(write_still=True)
        imgs.append(soften(raw))
        if not DEBUG:
            os.remove(raw)
    alpha = np.max([im[..., 3] for im in imgs], axis=0)
    ys, xs = np.nonzero(alpha > 0)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    return [im[y0:y1, x0:x1] for im in imgs], (ax - x0, ay - y0)


def render_sheet(n):
    """`<key>_walk.png`: one atlas per body, a row per view (SHEET_VIEWS order) and a column per frame (the
    standing frame, then N frames of one cycle). Every cell has one size and one anchor. `<key>_walk.json`
    describes it and carries the foot-drift reading."""
    frames = cycle_frames(n)
    poses = ["stand"] + frames
    views = [render_sheet_view(theta, poses) for _, theta in SHEET_VIEWS]
    left = math.ceil(max(a[0] for _, a in views)) + 1
    top = math.ceil(max(a[1] for _, a in views)) + 1
    right = max(c[0].shape[1] - a[0] for c, a in views)
    bottom = max(c[0].shape[0] - a[1] for c, a in views)
    cw, ch = int(math.ceil(left + right)) + 1, int(math.ceil(top + bottom)) + 1
    atlas = np.zeros((ch * len(views), cw * len(poses), 4))
    for r, (cells, (ax, ay)) in enumerate(views):
        ox, oy = int(round(left - ax)), int(round(top - ay))
        for c, im in enumerate(cells):
            h, w = im.shape[:2]
            atlas[r * ch + oy : r * ch + oy + h, c * cw + ox : c * cw + ox + w] = im
    save_rgba(np.clip(atlas, 0, 1), os.path.join(OUT, f"{KEY}_walk.png"))
    drift = foot_drift(frames)
    cycle_m = float(POSE["walk_cycle_m"])
    sidecar = {
        "key": KEY,
        "stature_m": STATURE,
        "views": [v for v, _ in SHEET_VIEWS],
        "view_angles_deg": [a for _, a in SHEET_VIEWS],
        "columns": ["stand"] + [f"walk:{i}" for i in range(n)],
        "frames_per_cycle": n,
        "cycle_m": round(cycle_m, 4),
        "cell": [cw, ch],
        "anchor": [left, top],
        "phase0": "the Walk cycle's largest left-foot-forward stride",
        "foot_drift_mm": drift,
        "hold_slide_mm": round(cycle_m / n * 1000.0, 1),
        "source": "the 3D client's own Stand and Walk poses of this export, dumped by cast.py ground --dump-pose",
        "source_file": os.path.basename(SRC),
        "source_sha256": sha,
        "source_bin_sha256": sha_bin,
        "pose_sha256": hashlib.sha256(open(POSE_JSON, "rb").read()).hexdigest(),
        "tool": f"Blender {bpy.app.version_string} ({scene.render.engine}), "
        "tools/character_pipeline/blender/render_sprites.py --walk-frames",
        "camera": {"projection": "orthographic", "elevation_deg": ELEV, "faces": "screen right"},
        "postprocess": {
            "supersample": SS,
            "downsample": "premultiplied box",
            "outline_px": 1,
            "rule": "pixel height of the standing frame = stature_m * 64",
        },
    }
    with open(os.path.join(OUT, f"{KEY}_walk.json"), "w") as f:
        json.dump(sidecar, f, indent=2)
    print("SHEET", KEY, "cell", [cw, ch], "anchor", [left, top], "drift_mm", drift)


if CYCLE:
    render_sheet(CYCLE)
    sys.exit(0)


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
                    "frame": "b" if frame_b else "a",
                    "lead_leg": ("right" if frame_b else "left"),
                    "source": "the 3D client's own pose of this export, dumped by cast.py ground --dump-pose: "
                    + POSE["clip"]["stride"]
                    + ", after the client's retarget, Posture and ground contact; the standing loop is "
                    + POSE["clip"]["stand"],
                    "pose_file": os.path.basename(POSE_JSON),
                    "pose_sha256": hashlib.sha256(open(POSE_JSON, "rb").read()).hexdigest(),
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
