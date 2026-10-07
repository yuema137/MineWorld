#!/usr/bin/env python3
"""The default character's clothing, modelled in Blender against the identity contract.

Called by `character_model.py`; see that file for the pipeline and for how to
run it.  Each `build_*` returns one object skinned to the body's armature.

**Why this exists.** `presentation/mineworld-default/3D/CHARACTER_IDENTITY.md`
§5 names, as checkable facts, a hood, cream drawstrings, ribbed cuffs, a ribbed
hem, two front panels with visible zip tape and a kangaroo pocket.  The previous
generator offset a shell off the t-shirt and could produce none of those: it had
no way to make a cord, a knit band, or a panel edge.  Those are not finish, they
are the features a viewer matches the garment against, and `VISUAL_FIDELITY.md`
§4 forbids deferring them on a milestone named for the reference.

**Why the garments are cut out of the body.** Every part of a garment that hugs
the figure is a copy of the body surface pushed out along its normal, so it fits
exactly and inherits the body's skin weights — no weight painting, and no
possibility of a sleeve that does not follow the arm.  Only the parts with no
body surface underneath (hood, drawstrings, cuff and hem bands, waistband) are
new geometry, and those get their weights from one nearest-surface transfer.

Frame: +Z up, +X the character's own left, **-Y the way she faces**.
"""

from __future__ import annotations

import math
import os
import sys

import bmesh  # pylint: disable=import-error
import bpy  # pylint: disable=import-error
from mathutils import Vector  # pylint: disable=import-error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model_lib import (  # noqa: E402  pylint: disable=wrong-import-position
    assign_material, bind, boundary_loop, cylindrical_uv, decimate,
    dominant_group, dup_region, extrude_band, extrude_strip, flare, inflate,
    load_body, loft, new_object, planar_uv, relax, report, report_boundaries,
    report_winding,
    rib_displace,
    keep_outside,
    rounded_box, set_material, shade_smooth, smooth_boundary, snap_opening, stitch,
    solidify, transfer_weights, tube,
)


# --------------------------------------------------------------- measured frame
# Read off the body with `blender --background --python probe_body.py`; they are
# joint positions, not authored numbers, so they move if the morph changes.
HIP_Z = 1.0048
SPINE2_Z = 1.3185
NECK_Z = 1.4519
HEAD_Z = 1.5338
KNEE_Z = 0.5099
ANKLE_Z = 0.0825

TORSO_BONES = {"Hips", "Spine", "Spine1", "Spine2", "LeftShoulder", "RightShoulder"}
ARM_BONES = {"LeftArm", "LeftForeArm", "RightArm", "RightForeArm"}
HAND_BONES = {"LeftHand", "RightHand"}
LEG_BONES = {"LeftUpLeg", "LeftLeg", "RightUpLeg", "RightLeg"}
HEAD_BONES = {"Neck", "Head"}


def grp(dom, v) -> str:
    return dom.get(v.index, "").replace("mixamorig:", "")


def any_in(dom, f, names) -> bool:
    return any(grp(dom, v) in names for v in f.verts)


def all_in(dom, f, names) -> bool:
    return all(grp(dom, v) in names for v in f.verts)


# How many increments a normal offset is taken in.  One jump of 40 mm inverted
# 183 faces of the hoodie where the shoulder, the armpit and the side of the
# neck curve tighter than that; twenty increments leave 1.  See `inflate`.
OFFSET_STEPS = 20


def short(name: str) -> str:
    return name.replace("mixamorig:", "")


# ------------------------------------------------------------------------- tee

TEE_HEM_Z = 1.030         # over the jeans waistband, as the reference shows
TEE_LIFT = 0.008          # how far the jersey stands off the skin

# The crew neckline, as one geometric curve shared by the tee and the body trim:
# a plane tilted front to back through the sternal notch (z 1.425 at skin
# y -0.010) and the base of the nape (z 1.475 at y +0.130), both measured off
# this body.  The tee's neckline used to be a 61 mm cylinder at the back and
# wherever the Neck weights stopped at the front -- a weight boundary is not a
# seam -- and the skin's trim edge was a third, unrelated line; between them the
# collar showed serrated skin, white teeth of jersey and the hoodie's inside.
NECK_AXIS_Y = 0.065
CREW_BAND = 0.040         # how far below the neckline the visible jersey ring runs
CREW_REACH = 0.090        # radius from the neck axis the ring is kept within


def crew_z(y: float) -> float:
    """Height of the crew neckline at depth `y` (front is -Y)."""
    return 1.425 + 0.357 * (y + 0.010)


def in_neck_zone(c) -> bool:
    return math.hypot(c.x, c.y - NECK_AXIS_Y) < CREW_REACH


# The short sleeve ends a little under halfway down the upper arm: the shoulder
# joint is at |x| 0.159 and the elbow at 0.315 on this body.
SLEEVE_X = 0.235
UPPER_ARM_BONES = {"LeftArm", "RightArm"}


def under_tee(dom, f, margin: float = 0.0) -> bool:
    """Whether the full tee covers this body face; with `margin`, by that much.

    **One predicate for the garment and for the skin it hides**, because the
    two disagreeing is what took most of a body off every person in town. The
    skin was trimmed for the *hoodie* -- back, shoulders, both arms -- while
    the tee had been cut down to the band the reference character's open front
    shows. Only she wears the hoodie. Everyone else was a front panel of
    jersey, a floating head and two floating hands. The body is trimmed for what
    every outfit covers, and that is the tee.
    """
    c = f.calc_center_median()
    if c.z < TEE_HEM_Z + margin or all_in(dom, f, HEAD_BONES):
        return False
    if in_neck_zone(c) and c.z > crew_z(c.y) - margin:
        return False
    if any_in(dom, f, {"LeftForeArm", "RightForeArm"} | HAND_BONES):
        return False
    if all_in(dom, f, UPPER_ARM_BONES):
        return abs(c.x) < SLEEVE_X - margin
    return any_in(dom, f, TORSO_BONES | UPPER_ARM_BONES)


def build_tee(body, dom):
    """The cream slogan tee: torso, shoulders, short sleeves and a crew neck.

    It is cut whole, not as the band the reference character's open hoodie
    shows, because everyone else in town wears it without a hoodie and the
    skin beneath it is trimmed away (`under_tee`).
    """
    def keep(f):
        return under_tee(dom, f)

    obj = dup_region(body, keep, "Tee")
    # Smooth *before* offsetting.  Laplacian smoothing pulls a convex surface
    # inward, so relaxing an already-offset shell sank the tee back inside the
    # ribcage and the bare chest came through between the hoodie panels.
    relax(obj, iterations=3, factor=0.5)
    inflate(obj, TEE_LIFT, smooth_first=2, steps=OFFSET_STEPS)
    flare(obj, TEE_HEM_Z, HIP_Z + 0.14, 0.02)
    decimate(obj, 0.30)
    # The crew neckline is the one edge of the tee anyone sees, standing above
    # the hoodie's collar.  Cut by face centre and then decimated, it came out
    # as a ring of white teeth.  Put the loop back on the curve it was cut on,
    # straighten it along the loop, and knit a rib collar off it.
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    def _neck_edge(co):
        return in_neck_zone(co) and co.z > crew_z(co.y) - 0.02

    ring = [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and _neck_edge(v.co)]
    for v in ring:
        v.co.z = crew_z(v.co.y)
    smooth_boundary(bm, _neck_edge, iterations=6, factor=0.5)
    # The collar itself is a separate band sewn over this edge (`_crew_rib`).
    # Extruded off the decimated edge, it stood up as a ring of white teeth.
    print(f"  tee neckline: {len(ring)} verts on the crew curve")
    # hem, neck, two short sleeves; anything else is a hole in everyone's shirt
    report_boundaries(bm, "tee", expect=4)
    bm.to_mesh(obj.data)
    bm.free()
    # smoothing and decimation both pull a convex shell inward; the collar
    # is what sank into the neck, so hold the whole tee off the skin
    moved = keep_outside(obj, body, TEE_LIFT * 0.6)
    print(f"  tee: {moved} verts pushed back out of the skin")
    # Anything of the tee left above the neckline is under the crew band at
    # best, and at the front it poked through it as white teeth: the push
    # above lifts the edge forward along the chest's normal.  Trim it.
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    high = [f for f in bm.faces
            if in_neck_zone(f.calc_center_median())
            and f.calc_center_median().z > crew_z(f.calc_center_median().y) - 0.003]
    bmesh.ops.delete(bm, geom=high, context="FACES")
    loose_v = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose_v, context="VERTS")
    bm.to_mesh(obj.data)
    bm.free()
    print(f"  tee: {len(high)} faces above the neckline trimmed under the crew band")
    solidify(obj, 0.0022)
    set_material(obj, "MW_Tee")
    # The reference's crew neck is a ribbed band in a darker maroon-brown
    # than the cream body, and with the hoodie open to the collar it is the
    # tee's most visible edge.
    assign_material(obj, "MW_TeeRib",
                    lambda c, _n: in_neck_zone(c) and c.z > crew_z(c.y) - 0.0015)
    # the graphic must land on the chest, so the front of the shirt is pinned to
    # the middle of the texture (see cylindrical_uv)
    cylindrical_uv(obj, TEE_HEM_Z - 0.02, NECK_Z + 0.03)
    shade_smooth(obj)
    rib = _crew_rib(obj)
    for x in bpy.data.objects:
        x.select_set(False)
    obj.select_set(True)
    rib.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.join()
    return obj


CREW_RIB_H = 0.011        # how far the band rises above the neckline
CREW_RIB_T = 0.0035       # and its thickness


def _crew_rib(tee):
    """The crew neck as a clean knitted band laid over the tee's neckline.

    The tee's own neck edge is a decimated cut, and with the hoodie open to
    the collar it showed as a jagged white line with skin above it.  A real
    crew neck is a separate ribbed band sewn over that edge, so it is built as
    one: a closed band following the neckline round the neck, at the radius
    the tee's own edge has at each angle, covering the cut.
    """
    cx, cy = 0.0, NECK_AXIS_Y
    edge = [v.co.copy() for v in tee.data.vertices
            if in_neck_zone(v.co) and abs(v.co.z - crew_z(v.co.y)) < 0.012]
    steps = 56
    radius = []
    for k in range(steps):
        a = math.tau * k / steps
        near = [p for p in edge
                if abs(math.remainder(math.atan2(p.x - cx, p.y - cy) - a, math.tau)) < 0.20]
        radius.append(max(math.hypot(p.x - cx, p.y - cy) for p in near) if near else None)
    known = [r for r in radius if r is not None] or [0.07]
    radius = [r if r is not None else sum(known) / len(known) for r in radius]
    # a little smoothing round the ring, so the band is a curve and not the
    # decimated edge again
    for _ in range(4):
        radius = [(radius[k - 1] + 2 * radius[k] + radius[(k + 1) % steps]) / 4
                  for k in range(steps)]
    rings = []
    for k in range(steps + 1):
        a = math.tau * (k % steps) / steps
        r = radius[k % steps] + 0.0015
        ox, oy = math.sin(a), math.cos(a)
        z = crew_z(cy + oy * r)
        ring = []
        # from 6 mm below the neckline to 10 mm above it: the cut edge, and
        # the vertices `keep_outside` lifted off it, are under the band
        for dr, dz in ((0.0, -0.006), (CREW_RIB_T, -0.006),
                       (CREW_RIB_T * 0.6, CREW_RIB_H - 0.001), (-0.002, CREW_RIB_H - 0.001)):
            ring.append((cx + ox * (r + dr), cy + oy * (r + dr), z + dz))
        rings.append(ring)
    verts, faces = loft(rings, cap_first=False, cap_last=False)
    rib = new_object("CrewRib", verts, faces, "MW_TeeRib")
    bm = bmesh.new()
    bm.from_mesh(rib.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    rib_displace(bm, lambda co: True, ribs=90, depth=0.0004, centre_xy=(cx, cy))
    bm.to_mesh(rib.data)
    bm.free()
    shade_smooth(rib)
    rib.data.uv_layers.new(name="UVMap")
    print(f"  crew rib: {len(rib.data.vertices)} verts, radius "
          f"{min(radius) * 1000:.0f}..{max(radius) * 1000:.0f} mm from the neck axis")
    return rib


# ------------------------------------------------------------------------ jeans

JEANS_WAIST_Z = 1.015     # mid-rise: on the hip crest, below the navel
JEANS_CUFF_Z = 0.075
JEANS_LIFT = 0.015


def build_jeans(body, dom):
    """Relaxed-straight mid-blue denim, cut from the legs and seat."""
    wanted = LEG_BONES | {"Hips", "Spine", "Spine1"}

    def keep(f):
        c = f.calc_center_median()
        if not JEANS_CUFF_Z <= c.z <= JEANS_WAIST_Z:
            return False
        # in the A-pose the hands hang beside the thigh; they are not trousers
        return not any_in(dom, f, ARM_BONES | HAND_BONES | HEAD_BONES) \
            and any_in(dom, f, wanted)

    obj = dup_region(body, keep, "Jeans")
    relax(obj, iterations=1, factor=0.35)
    inflate(obj, JEANS_LIFT, smooth_first=2, steps=OFFSET_STEPS)
    # "relaxed straight": the leg stops following the calf below the knee and
    # falls straight to the cuff instead.  A skinny jean is a different garment.
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    for v in bm.verts:
        if v.co.z >= KNEE_Z:
            continue
        t = min(1.0, (KNEE_Z - v.co.z) / (KNEE_Z - JEANS_CUFF_Z))
        cx = math.copysign(0.12, v.co.x)
        widen = 1.0 + 0.30 * t
        v.co.x = cx + (v.co.x - cx) * widen
        v.co.y = 0.03 + (v.co.y - 0.03) * widen
    bm.to_mesh(obj.data)
    bm.free()
    relax(obj, iterations=2, factor=0.4)
    decimate(obj, 0.34)

    # waistband, belt loops and a button: small, but they are what says "jeans"
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    top = boundary_loop(bm, lambda co: co.z > JEANS_WAIST_Z - 0.02)
    extrude_band(bm, top, [(Vector((0, 0, 0.018)), 1.006),
                           (Vector((0, 0, 0.015)), 1.002)])
    report_winding(bm, "jeans after waistband")
    bm.to_mesh(obj.data)
    bm.free()

    solidify(obj, 0.004)
    set_material(obj, "MW_Jeans")
    assign_material(obj, "MW_Denim_Trim", lambda c, _n: c.z > JEANS_WAIST_Z + 0.004)
    planar_uv(obj, 0.42)
    shade_smooth(obj)
    return obj


# ----------------------------------------------------------------------- hoodie

HOODIE_HEM_Z = 0.945      # hanging below the waistband, the lowest of the three
HOODIE_TOP_Z = NECK_Z - 0.005     # the collar seam; the hood sits on top of it
# The slack the garment starts with before it is draped.  Not a stand-off: the
# cloth simulation drops the fabric onto the shoulders and arms and it ends up
# lying on them; this is how much more cloth there is than body, which is what
# becomes the folds.  (Until preview 3 it was a 40 mm stand-off, frozen: the
# padded jacket.)
HOODIE_SLACK = 0.030
ZIP_HALF = 0.080          # half-width of the open front gap, at the centre line
ZIP_TAPE = 0.018          # width of the lighter tape band folded in off it

HOODIE_DRAPE_FRAMES = 45
# How far from the skin the draped fabric rests: over the tee (8 mm lift and
# 2.2 mm thick) with a little air.
HOODIE_CLEAR = 0.013

MATS = ("MW_Hoodie", "MW_Zip", "MW_Cord")
MAT_SHELL, MAT_ZIP, MAT_CORD = 0, 1, 2


def build_hoodie(body, dom, arm, colliders=()):
    """The open burgundy zip hoodie: two front panels, hood, cords, ribbing.

    Built in passes, because each needs geometry the previous one created: a
    loose shell with the garment's openings, **draped by cloth simulation**
    onto the body (`drape.py`) so it hangs and folds; the front taped, the
    cuffs and hem extruded off the draped shell's own boundary loops; and
    finally the hood -- a pouch of cloth sewn to the collar and draped down the
    back -- and the drawstrings.

    `colliders` are the garments already built under it (tee, jeans): the
    hoodie rests on them, not through them.
    """
    import drape  # pylint: disable=import-outside-toplevel
    # Selected by joint, not by height: a height cut across the top takes the
    # shoulder cap off with the neck and leaves the garment open at the seam.
    # The torso takes every face that touches a torso joint, so the mixed faces
    # at the armhole belong to it and the sleeve starts exactly where it ends.
    def keep_torso(f):
        c = f.calc_center_median()
        if c.z < HOODIE_HEM_Z:
            return False
        # `any_in(HEAD_BONES)` here dropped every face that so much as touched
        # the Neck vertex group -- which reaches down over the trapezius -- and
        # `any_in(TORSO_BONES)` dropped the seat, whose faces are dominated by
        # the upper leg. Between them they left six holes in the garment: two at
        # the top of the shoulders, two over the glutes, and a large one across
        # the upper back. The neckline is cut geometrically just below, so the
        # joint test only has to exclude faces that are *entirely* head or neck.
        if all_in(dom, f, HEAD_BONES):
            return False
        if not any_in(dom, f, TORSO_BONES | LEG_BONES):
            return False
        # A round neckline, cut geometrically.  Following the Neck joint's
        # weight boundary instead gives a collar with a ragged edge, because a
        # weight boundary is not a seam.
        if c.z > 1.405 and math.hypot(c.x, c.y - 0.070) < 0.074:
            return False
        # The open front, cut here on the dense body rather than after
        # decimation: a strip removed from a decimated shell leaves triangles
        # reaching across the opening, which read as a torn garment.  It runs
        # right up through the neckline: an open zip hoodie is open to the
        # collar, and the tee's crew neck shows between the hood's two ends.
        # Stopped at z 1.405, a strip of hoodie crossed her upper chest above
        # the tee, and its cut edge was the jagged line at the neckline.
        if abs(c.x) < ZIP_HALF and c.y < 0.01:
            return False
        return True

    def keep_sleeve(f):
        if not all_in(dom, f, ARM_BONES):
            return False
        # stop short of the wrist; the ribbed cuff below is extruded back to it
        c = f.calc_center_median()
        return (c - Vector((math.copysign(0.490, c.x), 0.032, 1.059))).length > 0.055

    # One object, not a torso plus two sleeves.  Offsetting three patches
    # separately gives each its own boundary normals, and the shoulder seam came
    # out as a row of spikes where the two offsets disagreed.
    obj = dup_region(body, lambda f: keep_torso(f) or keep_sleeve(f), "Hoodie")
    # slots are fixed up front so the bmesh passes below can tag their own faces;
    # `bpy.ops.object.join` then merges the hood and cords by material identity
    obj.data.materials.clear()
    for nm in MATS:
        obj.data.materials.append(bpy.data.materials.get(nm) or bpy.data.materials.new(nm))
    for pgon in obj.data.polygons:
        pgon.material_index = MAT_SHELL
    # Smoothed *before* the offset, never after: Laplacian smoothing pulls a
    # convex shell inward, and a starting shape that dips inside the tee or
    # the jeans is pushed the wrong way by the solver -- the first drape came
    # out under the tee, with the shirt showing through in patches.
    relax(obj, iterations=6, factor=0.5)
    inflate(obj, HOODIE_SLACK, smooth_first=4, steps=OFFSET_STEPS)
    decimate(obj, 0.55)
    flare(obj, HOODIE_HEM_Z, SPINE2_Z, 0.055)
    moved = keep_outside(obj, body, HOODIE_SLACK * 0.8)
    print(f"  hoodie start: {moved} verts pushed back out to {HOODIE_SLACK * 0.8 * 1000:.0f} mm")
    print(f"  hoodie start: {_bridge_seat(obj)} verts bridged across the seat")

    bm = bmesh.new()
    bm.from_mesh(obj.data)
    # Intended openings: the front, the hem and the neck as one loop -- the
    # open front runs down through the hem and up through the neckline --
    # and one cuff per sleeve.  "Five" was written while the welded seams
    # still showed as ten loops; "four" while the front stopped short of the
    # neck.
    report_boundaries(bm, "hoodie after cut+decimate", expect=3)

    def _front_edge(co):
        # below the neckline: above it the edge turns round the neck, and a
        # tape folded in off that stretch crossed her throat as white teeth
        return abs(co.x) < 0.16 and co.y < 0.03 and co.z < 1.40

    # The front edges are straightened before the drape (a decimated cut is
    # jagged) and never snapped back to a line after it: the open panels hang
    # where the cloth puts them.
    smooth_boundary(bm, _front_edge, iterations=8, factor=0.5)
    # The hem, cut by face centre across the seat and decimated, is a ragged
    # edge; level it before the drape, and remember which vertices it is so
    # the ribbed band is extruded off exactly them afterwards -- the drape
    # moves the hem, so a height test no longer finds it.
    hem_ids = []
    for v in bm.verts:
        # 45 mm: the cut, by face centre across the seat, zigzags that far,
        # and a hem vertex left out of the ring is a thread hanging off it
        if any(e.is_boundary for e in v.link_edges) and v.co.z < HOODIE_HEM_Z + 0.045:
            v.co.z = HOODIE_HEM_Z
            hem_ids.append(v.index)
    smooth_boundary(bm, lambda co: co.z < HOODIE_HEM_Z + 0.001, iterations=4, factor=0.5)
    # What is held: the collar, where the hood is sewn on and the garment
    # hangs from the neck, and the two wrist openings, where the cuffs grip.
    bm.verts.ensure_lookup_table()
    bm.verts.index_update()
    held = set()
    for v in bm.verts:
        if not any(e.is_boundary for e in v.link_edges):
            continue
        c = v.co
        if c.z > 1.36 and math.hypot(c.x, c.y - 0.07) < 0.13:
            held.add(v.index)
        elif abs(c.x) > 0.39 and c.z < 1.16:
            held.add(v.index)
    ring = {e.other_vert(bm.verts[i]).index
            for i in held for e in bm.verts[i].link_edges} - held
    bm.verts.ensure_lookup_table()
    bm.to_mesh(obj.data)
    bm.free()
    # The body is the only collider, with a skin thick enough to hold the
    # fabric clear of the tee lying on it.  The tee and the jeans as colliders
    # in their own right made the solver unstable -- thin solidified shells
    # the starting shape grazes; the fabric was thrown up to 196 mm and ended
    # under the tee -- so the layer under the hoodie is kept by distance
    # instead, and the jeans' waistband by `keep_outside` after the drape.
    del colliders
    drape.add_collider(body, thickness=HOODIE_CLEAR)
    drape.simulate(obj, lambda v: 1.0 if v.index in held else (0.5 if v.index in ring else 0.0),
                   frames=HOODIE_DRAPE_FRAMES)
    # Not over the cleft of the seat: there the nearest skin's normal points
    # sideways, and pushing along it pinched the bridged fabric back into a
    # crease down the middle.
    def _not_cleft(co):
        return not (co.y > 0.02 and abs(co.x) < 0.07 and co.z < 1.12)

    n_out = keep_outside(obj, body, HOODIE_CLEAR + 0.002, pick=_not_cleft)
    n_waist = keep_outside(obj, body, JEANS_LIFT + 0.012,
                           pick=lambda co: co.z < JEANS_WAIST_Z + 0.04 and _not_cleft(co))
    print(f"  hoodie after drape: {n_out} verts lifted clear of the tee, {n_waist} of the waistband")
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    n_snap = len(held)
    report_boundaries(bm, "hoodie after drape", expect=3)
    # the zip tape, as real geometry folded in off the panel edge
    tape = [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and _front_edge(v.co)]
    extrude_strip(bm, tape,
                  lambda co: Vector((-math.copysign(ZIP_TAPE, co.x), 0.0035, 0.0)),
                  material_index=MAT_ZIP)
    print(f"  opening: {n_snap} seam verts held in the drape, {len(tape)} tape verts")

    # --- ribbed hem, off the shell's own bottom boundary --------------------
    # The vertices levelled as the hem before the drape, wherever the drape
    # has put them.
    bm.verts.ensure_lookup_table()
    hem = [bm.verts[i] for i in hem_ids]
    hem_top = max((v.co.z for v in hem), default=HOODIE_HEM_Z)
    extrude_band(bm, hem, [(Vector((0, 0, -0.017)), 0.997),
                           (Vector((0, 0, -0.018)), 0.988),
                           (Vector((0, 0, -0.015)), 0.994)])
    hem_low = min((v.co.z for v in hem), default=HOODIE_HEM_Z) - 0.004
    rib_displace(bm, lambda co: co.z < hem_low and abs(co.x) < 0.30
                 and co.z < 1.10, ribs=34, depth=0.0016)
    print(f"  hem: {len(hem)} verts, top of the draped hem at z {hem_top:.3f}")

    # --- ribbed cuffs, off each sleeve's wrist boundary ---------------------
    for sx in (-1.0, 1.0):
        wrist = [v for v in bm.verts
                 if any(e.is_boundary for e in v.link_edges)
                 and v.co.x * sx > 0.39 and v.co.z < 1.16]
        if not wrist:
            continue
        cx = sum(v.co.x for v in wrist) / len(wrist)
        cy = sum(v.co.y for v in wrist) / len(wrist)
        cz = sum(v.co.z for v in wrist) / len(wrist)
        axis = (Vector((sx * 0.74, -0.18, -0.65))).normalized()
        centre = Vector((cx, cy, cz))
        ring = wrist
        for step, sc in ((0.012, 0.94), (0.014, 0.91), (0.012, 0.95)):
            res = bmesh.ops.extrude_vert_indiv(bm, verts=ring)
            pair = dict(zip(ring, res["verts"]))
            for a, b in pair.items():
                rad = a.co - centre
                rad -= axis * rad.dot(axis)
                b.co = a.co + axis * step + rad * (sc - 1.0)
            stitch(bm, ring, pair)
            ring = [pair[v] for v in ring]
            centre = centre + axis * step
        for v in bm.verts:
            if v.co.x * sx > 0.430 and v.co.z < 1.16:
                rad = v.co - Vector((cx, cy, cz))
                rad -= axis * rad.dot(axis)
                r = rad.length
                if r > 1e-6:
                    ang = math.atan2(rad.z, rad.y)
                    v.co += rad / r * (math.cos(ang * 16) * 0.0014)
    bm.normal_update()
    report_winding(bm, "hoodie after hem, tape and cuffs")
    report_boundaries(bm, "hoodie after hem, tape and cuffs", expect=3)
    # the neckline, where the hood is sewn on
    collar = [v.co.copy() for v in bm.verts
              if any(e.is_boundary for e in v.link_edges)
              and v.co.z > 1.36 and math.hypot(v.co.x, v.co.y - 0.07) < 0.13]
    bm.to_mesh(obj.data)
    bm.free()

    solidify(obj, 0.0032)

    # --- hood and drawstrings ----------------------------------------------
    hood = build_hood(collar, body, obj)
    cords = build_cords(body)
    for o in (hood, cords):
        bpy.context.view_layer.objects.active = obj
        for x in bpy.data.objects:
            x.select_set(False)
        obj.select_set(True)
        o.select_set(True)
        bpy.ops.object.join()

    planar_uv(obj, 0.55)
    shade_smooth(obj)
    return obj


# The hood, down.  The neck axis it wraps, and how far round it goes: 0° is
# straight behind the neck, ±HOOD_SWEEP are the two free ends beside the throat.
def hem_to_pelvis(obj) -> int:
    """Hand the hoodie's thigh weights to the pelvis.

    The nearest body surface to a hem lying over the seat is the top of each
    thigh, so the weight transfer skins the back of the hem to the two legs,
    and in the standing pose -- weight on one leg -- they pulled it apart
    into a crease down the middle.  A hoodie hangs from the shoulders and
    rides on the hips; it does not follow the thighs.
    """
    groups = {g.name: g.index for g in obj.vertex_groups}
    legs = [groups[n] for n in ("mixamorig:LeftUpLeg", "mixamorig:RightUpLeg") if n in groups]
    hips = groups.get("mixamorig:Hips")
    if hips is None or not legs:
        return 0
    hip_grp = obj.vertex_groups[hips]
    n = 0
    for v in obj.data.vertices:
        moved = 0.0
        for g in v.groups:
            if g.group in legs and g.weight > 0.0:
                moved += g.weight
                g.weight = 0.0
        if moved > 0.0:
            cur = next((g.weight for g in v.groups if g.group == hips), 0.0)
            hip_grp.add([v.index], cur + moved, "REPLACE")
            n += 1
    return n


def _bridge_seat(obj) -> int:
    """Carry the back of the starting shape straight across the seat.

    Cloth spans a hollow; it does not follow one.  The starting shape is an
    offset of the body, so below the small of the back it dips into the cleft
    between the buttocks, and the drape -- which only ever pushes fabric *out*
    of the body -- kept it there: the first draped hoodie hugged the seat like
    shorts.  Across the middle of the back, below the waist, every vertex is
    brought out to the furthest-back point of its own height.
    """
    verts = obj.data.vertices
    peak = {}
    for v in verts:
        if v.co.y > 0.02 and v.co.z < 1.12:
            k = round(v.co.z / 0.01)
            peak[k] = max(peak.get(k, -1.0), v.co.y)
    n = 0
    for v in verts:
        c = v.co
        if c.y <= 0.02 or c.z >= 1.12 or abs(c.x) > 0.11:
            continue
        target = peak.get(round(c.z / 0.01))
        if target is None:
            continue
        # full bridge in the middle, easing out toward the sides of the hips
        w = 1.0 - (abs(c.x) / 0.11) ** 2
        y = c.y + (target - c.y) * w
        if y > c.y + 1e-4:
            c.y = y
            n += 1
    obj.data.update()
    return n


HOOD_CENTRE = (0.0, 0.070)        # x, y of the neck axis at collar height
HOOD_SWEEP = 132.0                # degrees each way from straight behind

# The hood, down, as a *modelled form* laid onto what supports it.  Two parts
# of one surface, per column round the collar (0° straight behind, +a toward
# her left, ±SWEEP the two front ends at the zip):
#
#   the rim    -- the hood's face opening, which with the hood down folds over
#                 into a thick soft roll round the neck.  It stands up off the
#                 shoulders at both sides, which is what the reference shows
#                 from the front ("standing proud of the shoulders").
#   the panel  -- the hood's crown and back, lying back from the roll over the
#                 rucksack's lid and down its face: a rounded flap, deepest
#                 behind the neck, with a centre seam and a few large folds.
#                 This is what reads as a hood from behind.
#
# Preview 3 dropped a pouch of cloth onto a pack proxy.  With the lid at the
# collar's height there was nowhere for it to fall, and it came out as a
# crumpled mass that read as a scarf -- a missing defining garment structure.
# Cloth simulation decides drape well; it does not decide *shape*, and a hood
# down is a shape.  So the shape is authored and only its contact is solved:
# every point is pushed clear of the body, the hoodie and the pack.
HOOD_COLS = 61
HOOD_RIM_ROWS = 14
HOOD_PANEL_ROWS = 16
HOOD_ROLL_R = 0.029       # radius of the rolled rim, behind the neck
HOOD_ROLL_STAND = 0.026   # extra stand of the roll at the sides of the neck
# how far the flap reaches from the roll, behind the neck: over the lid's top
# and partway down its face, so the lid itself still shows below the hood
HOOD_PANEL_LEN = 0.135
HOOD_PANEL_HALF = 88.0    # degrees each way the flap spans
HOOD_FOLDS = 3.5          # large soft folds across the flap
HOOD_FOLD_AMP = 0.014
# How far the hood rests off what it lies on.  More than the drape's 6 mm:
# the bag and the hood are skinned from different body surfaces, and in the
# standing pose the lid came 6 mm through the flap.
HOOD_CLEAR = 0.012


def pack_shapes():
    """The rucksack's soft volumes, as `rounded_box` arguments.

    One list, used by the pack itself and by the hood that rests on it, so the
    two cannot disagree about where the lid is.
    """
    return [
        (BAG_CENTRE, BAG_SIZE, 0.055, 5, 6),
        (PACK_LID_CENTRE, PACK_LID_SIZE, 0.040, 4, 3),
        (PACK_POCKET_CENTRE, PACK_POCKET_SIZE, 0.034, 4, 3),
    ]


def _collar_at(collar, a):
    """The collar point at azimuth `a` round the neck axis (0 straight behind).

    Interpolated between the two ring points that bracket `a`: the nearest
    single point made the seam a staircase at 61 columns.
    """
    cx, cy = HOOD_CENTRE
    pts = sorted(((math.atan2(p.x - cx, p.y - cy), p) for p in collar),
                 key=lambda e: e[0])
    lo = max((e for e in pts if e[0] <= a), default=pts[0], key=lambda e: e[0])
    hi = min((e for e in pts if e[0] >= a), default=pts[-1], key=lambda e: e[0])
    if hi[0] - lo[0] < 1e-6:
        return lo[1].copy()
    t = (a - lo[0]) / (hi[0] - lo[0])
    return lo[1].lerp(hi[1], t)


class _Support:
    """What the hood lies on: the body, the hoodie shell, the rucksack."""

    def __init__(self, objs, boxes):
        from mathutils.bvhtree import BVHTree  # pylint: disable=import-error,import-outside-toplevel
        dg = bpy.context.evaluated_depsgraph_get()
        self.trees = [BVHTree.FromObject(o, dg) for o in objs]
        for args in boxes:
            v, f = rounded_box(*args)
            self.trees.append(BVHTree.FromPolygons([Vector(p) for p in v], f))

    def push_out(self, p, clear):
        """`p` moved to at least `clear` outside every support it is near."""
        for _ in range(3):
            moved = False
            for tree in self.trees:
                q, n, _i, d = tree.find_nearest(p, clear + 0.05)
                if q is None:
                    continue
                n = n.normalized()
                side = n.dot(p - q)
                if side < clear:
                    # inside, or too close: out along the surface normal
                    p = q + n * clear
                    moved = True
            if not moved:
                break
        return p

    def outermost(self, origin, out, far=0.32):
        """Distance along `out` from `origin` to the outermost support surface
        at that height, or None where nothing is there."""
        best = None
        for tree in self.trees:
            hit = tree.ray_cast(origin + out * far, -out, far)
            if hit[0] is not None:
                r = far - hit[3]
                best = r if best is None else max(best, r)
        return best

    def drape_profile(self, start, out, length, clear, step=0.002):
        """The flap's path from `start`, hanging over whatever is below it.

        Worked in the vertical plane through `start` along `out`: a cloth
        lying over an obstacle follows the obstacle's outer envelope and hangs
        straight down past it.  So, going down from `start` in millimetre
        steps, the flap's distance out is the furthest-out support surface met
        so far, plus the clearance -- a horizontal run where it crosses the top
        of the bag's lid, then down the bag's face.  Resampled to `length` of
        arc.  Marching the flap point by point and pushing each out of the
        nearest surface was tried first; the nearest surface of a lid is as
        often its front as its top, and the flap tangled behind the bag.
        """
        r_env = 0.0
        pts = [start.copy()]
        z = start.z
        arc = 0.0
        while arc < length:
            z -= step
            origin = Vector((start.x, start.y, z))
            # looked up `clear` below: the run across the top of the lid then
            # rides `clear` above it, and not one step above it -- the lid's
            # dome came through the flap there in the standing pose
            r = self.outermost(origin - Vector((0.0, 0.0, clear)), out)
            if r is not None:
                r_env = max(r_env, r + clear)
            q = origin + out * r_env
            arc += (q - pts[-1]).length
            pts.append(q)
            if z < start.z - 0.6:
                break
        return pts


def _hood_column(seam, a, sweep, support):
    """One column of the hood: the rim roll, then the flap laid over support."""
    t = a / sweep                          # -1 .. 1, 0 straight behind
    out = Vector((math.sin(a), math.cos(a), 0.0))
    up = Vector((0.0, 0.0, 1.0))
    # The roll thins to nothing at the zip, where the hood's two ends meet the
    # front edges, and stands highest at the sides of the neck.
    taper = 1.0 - abs(t) ** 4
    rr = HOOD_ROLL_R * (0.30 + 0.70 * taper)
    side = math.sin(min(1.0, abs(a) / math.radians(100.0)) * math.pi / 2) ** 2
    stand = HOOD_ROLL_STAND * side * taper
    centre = seam + out * (rr * 0.85) + up * (rr * 0.85 + stand)
    rim = []
    for j in range(HOOD_RIM_ROWS):
        # from inside-low (at the seam) over the top to outside-low
        th = math.radians(215.0 - 265.0 * j / (HOOD_RIM_ROWS - 1))
        # a little wider than tall: folded fleece, not a tube
        p = centre + out * (math.cos(th) * rr * 1.15) + up * (math.sin(th) * rr)
        rim.append(p)
    rim[0] = seam + out * 0.002 + up * 0.003
    # The flap: rounded outline, deepest behind the neck with a soft point
    # where the hood's crown seam ends.
    half = math.radians(HOOD_PANEL_HALF)
    u = min(1.0, abs(a) / half)
    length = (HOOD_PANEL_LEN * max(0.0, 1.0 - u ** 2.2) ** 0.55
              + 0.022 * math.exp(-(a / math.radians(14.0)) ** 2) + 0.010)
    fold_phase = a * HOOD_FOLDS / math.radians(90.0) * math.pi
    path = support.drape_profile(rim[-1], out, length, HOOD_CLEAR)
    # resample by arc length, then soften: fleece rounds a corner, it does
    # not fold over it at a right angle
    cum = [0.0]
    for k in range(1, len(path)):
        cum.append(cum[-1] + (path[k] - path[k - 1]).length)
    total = max(cum[-1], 1e-6)
    panel, k = [], 0
    for j in range(HOOD_PANEL_ROWS):
        want = total * (j + 1) / HOOD_PANEL_ROWS
        while k < len(cum) - 2 and cum[k + 1] < want:
            k += 1
        seg = max(cum[k + 1] - cum[k], 1e-9)
        panel.append(path[k].lerp(path[k + 1], min(1.0, (want - cum[k]) / seg)))
    for _ in range(3):
        panel = [panel[0]] + [(panel[i - 1] + panel[i] * 2 + panel[i + 1]) / 4
                              for i in range(1, len(panel) - 1)] + [panel[-1]]
    # folds radiate from the roll and deepen toward the flap's edge; the
    # centre seam is a shallow ridge down the middle
    for j, p in enumerate(panel):
        s = (j + 1) / HOOD_PANEL_ROWS
        fold = HOOD_FOLD_AMP * s * (0.5 + 0.5 * math.cos(fold_phase)) * (1.0 - u * 0.5)
        ridge = 0.0035 * math.exp(-(a / math.radians(3.0)) ** 2)
        panel[j] = p + out * (fold + ridge)
    return rim + panel


def build_hood(collar, body, shell):
    """The hood, down: a rolled rim round the neck and a flap over the pack lid.

    A zip hoodie without a hood is a zip jacket, and the contract calls the hood
    the feature that names the garment: "bunched in soft folds behind and around
    the neck, standing proud of the shoulders".  See the constants above for
    why it is modelled rather than simulated.
    """
    if not collar:
        print("  hood: no collar ring found -- no hood")
        return new_object("Hood", [], [], "MW_Hoodie")
    support = _Support([body, shell], pack_shapes())
    sweep = math.radians(HOOD_SWEEP)
    rows = HOOD_RIM_ROWS + HOOD_PANEL_ROWS
    verts, faces = [], []
    for i in range(HOOD_COLS):
        a = (i / (HOOD_COLS - 1) * 2 - 1) * sweep
        verts.extend(_hood_column(_collar_at(collar, a), a, sweep, support))
    for i in range(HOOD_COLS - 1):
        for j in range(rows - 1):
            k = i * rows + j
            faces.append((k, k + 1, k + rows + 1, k + rows))
    hood = new_object("Hood", verts, faces, "MW_Hoodie")
    solidify(hood, 0.0035)
    zs = [v.co.z for v in hood.data.vertices]
    print(f"  hood: {len(hood.data.vertices)} verts, z {min(zs):.3f}..{max(zs):.3f}")
    return hood


CORD_CLEAR = HOODIE_CLEAR + 0.008     # a cord lies on the fabric, not in the air


def build_cords(body=None):
    """Two flat cream drawstrings hanging from the collar down the chest.

    The reference shows the character's right cord (the viewer's left) long and
    clearly readable against the tee, and the left one shorter.  The character's
    right is -X.

    The path is written in the air and then laid onto the figure: each point
    is kept `CORD_CLEAR` off the skin along the skin's normal.  Written for the
    40 mm stand-off hoodie, it hung 60 mm in front of the draped one.
    """
    tree = None
    if body is not None:
        from mathutils.bvhtree import BVHTree  # pylint: disable=import-error,import-outside-toplevel
        src = bmesh.new()
        src.from_mesh(body.data)
        src.normal_update()
        tree = BVHTree.FromBMesh(src)
        src.free()
    verts, faces = [], []
    for sx, bottom, sway in ((-1.0, 1.138, -0.014), (1.0, 1.268, 0.010)):
        path = []
        # just inside the zip edge, over the tee, as the reference shows the
        # right cord; at 66 mm out they hung behind the backpack straps
        top = Vector((sx * 0.052, -0.100, 1.428))
        n = 8
        for i in range(n):
            t = i / (n - 1)
            p = Vector((
                top.x + sx * (0.012 * math.sin(t * 2.4)) + sway * t,
                top.y - 0.010 * t,
                top.z - (top.z - bottom) * t))
            if tree is not None:
                q, nrm, _i, _d = tree.find_nearest(p)
                if q is not None:
                    p = q + nrm * CORD_CLEAR
            path.append(p)
        radii = [0.0040] * n
        radii[-1] = 0.0046          # the aglet
        v, f = tube(path, radii, segments=7, flatten=0.55)
        base = len(verts)
        verts.extend(v)
        faces.extend(tuple(base + i for i in tri) for tri in f)
    return new_object("Cords", verts, faces, "MW_Cord")


# ------------------------------------------------------------------------ shoes

SHOE_TOP_Z = 0.118        # the collar of a low sneaker, just above the ankle
SHOE_LIFT = 0.010
SOLE_Z = -0.018           # how far the sole drops below the bare sole


def build_shoes(body, dom):
    """Ordinary casual sneakers, cut from the feet.

    The CC0 asset is barefoot.  The reference plate crops at mid-thigh and the
    contract records footwear as unconstrained, so this is plausible
    reconstruction: a low sneaker with a sole, not a claim about the reference.

    It replaces a generated tube swept along the foot's bounding box.  Cutting
    the shoe out of the foot instead means it fits the foot it is on and moves
    with the toe joint, which the swept tube did only approximately.
    """
    def keep(f):
        return (all_in(dom, f, {"LeftFoot", "LeftToeBase",
                                "RightFoot", "RightToeBase"})
                and f.calc_center_median().z < SHOE_TOP_Z)

    obj = dup_region(body, keep, "Shoes")
    relax(obj, iterations=1, factor=0.35)
    inflate(obj, SHOE_LIFT, smooth_first=2, steps=OFFSET_STEPS)
    decimate(obj, 0.30)

    # a sole: drop everything near the ground to a flat plane, which both gives
    # the shoe a straight contact surface and stops the toes reading through it
    for v in obj.data.vertices:
        if v.co.z < 0.022:
            v.co.z = SOLE_Z + (v.co.z - SOLE_Z) * 0.25
    solidify(obj, 0.004)
    set_material(obj, "MW_Shoe")
    assign_material(obj, "MW_Sole", lambda c, _n: c.z < 0.014)
    planar_uv(obj, 0.30)
    shade_smooth(obj)
    return obj


# ---------------------------------------------------------------- the backpack

# The strap paths, over the hoodie rather than inside it: the shell stands
# 40 mm off the body at the chest, so the webbing runs at about 55 mm.  Points
# are (x for her left strap, y, z); the right strap is the mirror.
STRAP_PATH = [
    (0.086, -0.148, 1.120),
    (0.092, -0.152, 1.250),
    (0.090, -0.140, 1.360),
    (0.086, -0.104, 1.442),
    (0.098, -0.016, 1.516),
    (0.094, 0.082, 1.466),
    (0.084, 0.140, 1.352),
    (0.074, 0.156, 1.250),
]
STRAP_W = 0.044           # padded webbing, flat against the chest
STRAP_T = 0.013
# The bag hangs with its lid below the collar seam, as a worn daypack does.
# Until preview 4 the lid stood level with the collar, and the hood -- which
# with the hood down has to lie *somewhere* behind the neck -- had nowhere to
# go but into a crumpled mass on top of it.
PACK_DROP = 0.045
BAG_CENTRE = (0.0, 0.232, 1.262 - PACK_DROP)
BAG_SIZE = (0.268, 0.152, 0.350)
PACK_LID_CENTRE = (0.0, 0.238, 1.408 - PACK_DROP)
PACK_LID_SIZE = (0.248, 0.140, 0.088)
PACK_POCKET_CENTRE = (0.0, 0.272, 1.148 - PACK_DROP)
PACK_POCKET_SIZE = (0.196, 0.086, 0.118)


STRAP_GAP = 0.009         # strap centreline above the hoodie surface it lies on


def _strap_onto(path, over, hold=None):
    """Lay a strap's centreline on the draped hoodie rather than in the air.

    The strap path was written for a hoodie standing 40 mm off the body.  The
    draped one lies on the body, so each point is moved to the hoodie's
    surface plus `STRAP_GAP` along the surface normal -- wherever the hoodie is
    within reach.  Where it is not (over the open front, beside the zip) the
    point is left alone.
    """
    from mathutils.bvhtree import BVHTree  # pylint: disable=import-error,import-outside-toplevel
    tree = BVHTree.FromObject(over, bpy.context.evaluated_depsgraph_get())
    out, moved = [], 0
    for p in path:
        if hold is not None and hold(p):
            out.append(p)
            continue
        hit = tree.find_nearest(p, 0.06)
        if hit[0] is None:
            out.append(p)
            continue
        q, n = hit[0], hit[1].normalized()
        if n.dot(p - q) < 0:
            n = -n
        out.append(q + n * STRAP_GAP)
        moved += 1
    return out, moved


PIPING_R = 0.0042         # a bound seam stands about this far proud of the canvas
CSTRAP_X = 0.066          # the two compression straps down the bag's face
CSTRAP_W = 0.022


def _pack_details(add, webbing, buckle) -> int:
    """Seams, two compression straps with buckles: what makes a bag a rucksack.

    Preview 3's pack was three smooth soft boxes, and from behind it read as a
    khaki lump.  A canvas daypack is recognised by its construction: bound
    seams round the lid and the pocket and down the sides, and webbing straps
    over the lid with side-release buckles.  Each is laid onto the bag's own
    surface by ray cast, so none floats and none sinks in.
    """
    from mathutils.bvhtree import BVHTree  # pylint: disable=import-error,import-outside-toplevel
    shapes = pack_shapes()

    def tree_of(args):
        v, f = rounded_box(*args)
        return BVHTree.FromPolygons([Vector(p) for p in v], f)

    bag, lid, pocket = (tree_of(a) for a in shapes)
    whole = [bag, lid, pocket]

    def hit(trees, origin, direction, lift):
        best = None
        for t in trees:
            h = t.ray_cast(origin, direction, 1.0)
            if h[0] is not None and (best is None or h[3] < best[3]):
                best = h
        if best is None:
            return None
        n = best[1].normalized()
        if n.dot(direction) > 0:
            n = -n
        return best[0] + n * lift

    def ring(tree, centre, z, lift, count=40):
        pts = []
        for k in range(count + 1):
            th = math.tau * k / count
            d = Vector((math.cos(th), math.sin(th), 0.0))
            o = Vector((centre[0], centre[1], z)) + d * 0.5
            p = hit([tree], o, -d, lift)
            if p is not None:
                pts.append(p)
        return pts

    count = [0]

    def emit(v, f, bucket):
        add(v, f, bucket)
        count[0] += len(f)

    # bound seam round the lid at its widest, and round the pocket
    for tree, args in ((lid, shapes[1]), (pocket, shapes[2])):
        (cx, cy, cz), _size, *_rest = args
        pts = ring(tree, (cx, cy), cz, PIPING_R * 0.5)
        if len(pts) > 4:
            v, f = tube(pts, [PIPING_R] * len(pts), segments=6, cap=False)
            emit(v, f, webbing)
    # side seams, down both sides of the bag body
    (bx, by, bz), (sx_, _sy, sz), *_rest = shapes[0]
    for side in (-1.0, 1.0):
        pts = []
        for k in range(14):
            z = bz - sz * 0.42 + sz * 0.80 * k / 13
            d = Vector((side, 0.0, 0.0))
            p = hit([bag], Vector((bx, by, z)) + d * 0.5, -d, PIPING_R * 0.5)
            if p is not None:
                pts.append(p)
        if len(pts) > 3:
            v, f = tube(pts, [PIPING_R] * len(pts), segments=6)
            emit(v, f, webbing)
    # compression straps: from the top of the lid, over its back edge and down
    # the face of the bag to the pocket, each with a buckle where lid meets bag
    (lx, ly, lz), (_lsx, lsy, lsz), *_rest = shapes[1]
    for side in (-1.0, 1.0):
        x = side * CSTRAP_X
        path = []
        for k in range(5):
            y = ly - lsy * 0.15 + lsy * 0.45 * k / 4
            p = hit(whole, Vector((x, y, lz + 0.4)), Vector((0, 0, -1)), 0.003)
            if p is not None:
                path.append(p)
        z = lz + lsz * 0.20
        while z > bz - sz * 0.10:
            p = hit(whole, Vector((x, 0.8, z)), Vector((0, -1, 0)), 0.003)
            if p is not None:
                path.append(p)
            z -= 0.018
        if len(path) > 3:
            v, f = tube(path, [CSTRAP_W / 2] * len(path), segments=6, flatten=0.16)
            emit(v, f, webbing)
            bz_ = lz - lsz * 0.55
            p = hit(whole, Vector((x, 0.8, bz_)), Vector((0, -1, 0)), 0.006)
            if p is not None:
                v, f = rounded_box((p.x, p.y, p.z), (0.034, 0.010, 0.030), 0.004,
                                   segs=2, slices=2)
                emit(v, f, buckle)
    return count[0]


def build_pack(body, dom, arm, over=None):
    """The grey-green canvas rucksack, and the two padded straps.

    Built here, with the clothes, and skinned to the same armature — not
    parented to the character and not hung off a `BoneAttachment3D`.  Both of
    those were tried and both put the bag floating off her back, because a bone
    attachment's frame after retargeting is not character space.  A bag whose
    vertices are weighted to `Spine1`/`Spine2` cannot float: it is on the back
    the same way the hoodie is.

    The contract's own emphasis is on the *straps*, not the bag — "sits behind
    the shoulders; only the edges are visible from the front" — so the webbing
    is where the detail goes.
    """
    verts, faces = [], []
    lower, buckle = [], []

    def add(v, f, bucket=None):
        base = len(verts)
        verts.extend(v)
        for tri in f:
            faces.append(tuple(base + i for i in tri))
            if bucket is not None:
                bucket.append(len(faces) - 1)

    for sx in (-1.0, 1.0):
        path = [Vector((sx * px, py, pz)) for px, py, pz in STRAP_PATH]
        if over is not None:
            # Her left strap's front run is in her hand: the grip pulls it off
            # the chest, and the arm is solved to that run (`solve_grip.gd`,
            # `stand_pose.gd`). Laid on the hoodie it moved 30 mm under her
            # knuckles and she held nothing. Only the run over the shoulder
            # and down the back is laid on the fabric on that side.
            path, n = _strap_onto(path, over,
                                  hold=(lambda p: p.z < 1.40 and p.y < 0.0) if sx > 0 else None)
            print(f"  strap {'L' if sx > 0 else 'R'}: {n} of {len(path)} points laid on the hoodie")
        radii = [STRAP_W / 2] * len(path)
        radii[0] = STRAP_W / 2 * 0.86          # tapers into the adjuster
        v, f = tube(path, radii, segments=8, flatten=STRAP_T / STRAP_W)
        add(v, f)
        # the dark lower section the reference shows below the adjuster
        low = [p.copy() for p in path[:2]]
        v, f = tube([low[0] + Vector((0, 0, -0.085)), low[0]],
                    [STRAP_W / 2 * 0.80, STRAP_W / 2 * 0.84],
                    segments=8, flatten=STRAP_T / STRAP_W)
        add(v, f, lower)
        # the adjuster itself, a flat slider across the webbing
        v, f = rounded_box((path[0].x, path[0].y - 0.002, path[0].z - 0.002),
                           (0.058, 0.020, 0.024),
                           0.006, segs=3, slices=2)
        add(v, f, buckle)

    n_strap = len(faces)
    # the bag, a lid flap and a lower pocket, so the silhouette is not one box
    for args in pack_shapes():
        v, f = rounded_box(*args)
        add(v, f)
    webbing = []
    n_detail = _pack_details(add, webbing, buckle)
    print(f"  pack: {n_detail} detail faces (piping, compression straps, buckles, handle)")

    obj = new_object("Pack", verts, faces, "MW_Pack")
    assign_material(obj, "MW_Webbing", lambda c, _n: False)
    assign_material(obj, "MW_StrapLow", lambda c, _n: False)
    assign_material(obj, "MW_Buckle", lambda c, _n: False)
    slots = {m.name: i for i, m in enumerate(obj.data.materials)}
    webbing_set = set(webbing)
    buckle = set(buckle)
    for i, poly in enumerate(obj.data.polygons):
        if i in buckle:
            poly.material_index = slots["MW_Buckle"]
        elif i in lower:
            poly.material_index = slots["MW_StrapLow"]
        elif i < n_strap or i in webbing_set:
            poly.material_index = slots["MW_Webbing"]
    planar_uv(obj, 0.30)
    shade_smooth(obj)
    print(f"  pack: {len(obj.data.vertices)} verts, straps {n_strap} faces")
    return obj


# ------------------------------------------------------------------- body trim

def trim_body(body, dom):
    """Delete the body surface the clothes cover.

    Upstream shipped the body already occlusion-deleted, which is exactly why it
    could not be re-dressed: there was no torso under the t-shirt to cut a new
    garment from.  So the whole body is carried through the modelling and the
    covered faces are removed at the end, here, once every garment that will
    cover them exists.

    Each region is pulled back from its garment's own boundary by a margin, so a
    garment edge that moves by a few millimetres cannot open a hole in the skin.
    """
    m = 0.020
    torso = TORSO_BONES
    arms = ARM_BONES
    legs = LEG_BONES | {"Hips", "Spine", "Spine1"}
    feet = {"LeftFoot", "LeftToeBase", "RightFoot", "RightToeBase"}
    hands = HAND_BONES

    def covered(f):
        c = f.calc_center_median()
        if all_in(dom, f, legs) and JEANS_CUFF_Z + m <= c.z <= JEANS_WAIST_Z - m:
            return True
        # Round the neck the skin is cut on the tee's own neckline, a little
        # above it, so the cut edge sits under the knitted collar instead of
        # showing as a serrated line between the collar and the throat.
        if in_neck_zone(c) and not all_in(dom, f, HEAD_BONES) \
                and c.z >= TEE_HEM_Z + m:
            # a little *below* it since the crew band (preview 3): the trim's
            # serrated edge has to fall under the band, and at +4 mm its
            # teeth stood above the band's inner face as holes in the skin
            return c.z < crew_z(c.y) - 0.002
        # Everything else above the jeans: only what the *tee* covers, never
        # what the hoodie covers -- see `under_tee`.  The skin under the
        # hoodie's long sleeves stays, as everyone else's forearms.
        if under_tee(dom, f, margin=m):
            return True
        if all_in(dom, f, feet) and c.z < SHOE_TOP_Z - m:
            return True
        return False

    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.faces.ensure_lookup_table()
    doomed = [f for f in bm.faces if covered(f)]
    before = len(bm.faces)
    bmesh.ops.delete(bm, geom=doomed, context="FACES")
    bm.verts.ensure_lookup_table()
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bm.to_mesh(body.data)
    bm.free()
    body.data.update()
    print(f"  body trim: {len(doomed)} of {before} faces covered, "
          f"{len(body.data.vertices)} verts left")
