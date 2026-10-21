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


def render_cycle(n):
    """The walk sheet of one view: N cycle frames and the standing frame, framed together at the standing scale.
    The anchor is the body's root on the floor (the armature origin), fixed for every frame, so the sheet bobs and
    sways as the 3D body does instead of being re-centred per frame."""
    frames = cycle_frames(n)
    sheet = {}
    for view in ("front", "back"):
        back, right, up, d = view_basis(view)
        bounds = []
        for f in ["stand"] + frames:
            pose_frame(f)
            pts = evaluated_points()
            bounds.append((pts @ np.array(right), pts @ np.array(up), Vector(np.mean(pts, axis=0)).dot(d)))
        inner_h = TARGET_H - 2
        p = (bounds[0][1].max() - bounds[0][1].min()) / (inner_h * SS)  # the standing body sets the scale
        umin = min(b[0].min() for b in bounds)
        umax = max(b[0].max() for b in bounds)
        vmin = min(b[1].min() for b in bounds)
        vmax = max(b[1].max() for b in bounds)
        pad = 3 * SS
        wpx = int(math.ceil((umax - umin) / p / SS)) * SS + 2 * pad
        hpx = int(math.ceil((vmax - vmin) / p / SS)) * SS + 2 * pad
        cu, cv = (umin + umax) / 2, (vmin + vmax) / 2
        aim(view, cu, cv, p, wpx, hpx, bounds[0][2])
        origin = WM.translation
        ax = (Vector(origin).dot(Vector(right)) - (cu - p * wpx / 2)) / p / SS
        ay = ((cv + p * hpx / 2) - Vector(origin).dot(Vector(up))) / p / SS
        imgs = []
        for i, f in enumerate(["stand"] + frames):
            pose_frame(f)
            raw = os.path.join(OUT, f"{KEY}_{view}_{i}.raw.png")
            scene.render.filepath = raw
            bpy.ops.render.render(write_still=True)
            imgs.append(soften(raw))
            if not DEBUG:
                os.remove(raw)
        alpha = np.max([im[..., 3] for im in imgs], axis=0)
        ys, xs = np.nonzero(alpha > 0)
        y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
        names = [f"{KEY}_{view}_stand"] + [f"{KEY}_{view}_w{i}" for i in range(n)]
        for name, im in zip(names, imgs):
            save_rgba(np.clip(im[y0:y1, x0:x1], 0, 1), os.path.join(OUT, name + ".png"))
        sheet[view] = {
            "frames": names,
            "size": [int(x1 - x0), int(y1 - y0)],
            "anchor": [round(ax - x0, 1), round(ay - y0, 1)],
        }
        print("SHEET", view, sheet[view]["size"], sheet[view]["anchor"])
    with open(os.path.join(OUT, f"{KEY}_walk.json"), "w") as f:
        json.dump(
            {
                "key": KEY,
                "stature_m": STATURE,
                "frames_per_cycle": n,
                "cycle_m": POSE.get("walk_cycle_m"),
                "phase0": "largest left-foot-forward stride",
                "source": "the 3D client's own Walk cycle of this export, dumped by cast.py ground --dump-pose",
                "source_sha256": sha,
                "pose_sha256": hashlib.sha256(open(POSE_JSON, "rb").read()).hexdigest(),
                "views": sheet,
            },
            f,
            indent=2,
        )


if CYCLE:
    render_cycle(CYCLE)
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
