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
    collar = extrude_band(bm, ring, [(Vector((0, 0, 0.006)), 1.015),
                                     (Vector((0, 0, 0.007)), 0.990)],
                          centre=Vector((0.0, NECK_AXIS_Y, 0.0)))
    rib_displace(bm, lambda co: in_neck_zone(co) and co.z > crew_z(co.y) + 0.001,
                 ribs=40, depth=0.0008, centre_xy=(0.0, NECK_AXIS_Y))
    print(f"  tee collar: {len(ring)} neckline verts, {len(collar or [])} in the rib's top")
    # hem, neck, two short sleeves; anything else is a hole in everyone's shirt
    report_boundaries(bm, "tee", expect=4)
    bm.to_mesh(obj.data)
    bm.free()
    # smoothing and decimation both pull a convex shell inward; the collar
    # is what sank into the neck, so hold the whole tee off the skin
    moved = keep_outside(obj, body, TEE_LIFT * 0.6)
    print(f"  tee: {moved} verts pushed back out of the skin")
    solidify(obj, 0.0022)
    set_material(obj, "MW_Tee")
    # the graphic must land on the chest, so the front of the shirt is pinned to
    # the middle of the texture (see cylindrical_uv)
    cylindrical_uv(obj, TEE_HEM_Z - 0.02, NECK_Z + 0.03)
    shade_smooth(obj)
    return obj


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
HOODIE_LIFT = 0.040
                          # decimation can cut off a curved shoulder
ZIP_HALF = 0.080          # half-width of the open front gap, at the centre line
ZIP_TAPE = 0.018          # width of the lighter tape band folded in off it

MATS = ("MW_Hoodie", "MW_Zip", "MW_Cord")
MAT_SHELL, MAT_ZIP, MAT_CORD = 0, 1, 2


def build_hoodie(body, dom, arm):
    """The open burgundy zip hoodie: two front panels, hood, cords, ribbing.

    Built in five passes, because each needs geometry the previous one created:
    the shell off the body, the front opened and taped, the cuffs and hem
    extruded off the shell's own boundary loops, the pocket lifted off the front
    panels, and finally the hood and drawstrings, which have no body surface
    under them at all.
    """
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
        # reaching across the opening, which read as a torn garment.
        if abs(c.x) < ZIP_HALF and c.y < 0.01 and c.z < 1.405:
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
    relax(obj, iterations=3, factor=0.5)
    inflate(obj, HOODIE_LIFT, smooth_first=3, steps=OFFSET_STEPS)
    decimate(obj, 0.55)
    flare(obj, HOODIE_HEM_Z, SPINE2_Z, 0.055)

    bm = bmesh.new()
    bm.from_mesh(obj.data)
    # Intended openings: the front-and-hem (one loop, because the open front
    # runs down through the hem), the neck, and one cuff per sleeve.  "Five"
    # was written while the welded seams still showed as ten loops, and it
    # counted the hem and the front twice.
    report_boundaries(bm, "hoodie after cut+decimate", expect=4)
    # the two front panel edges, straightened after decimation
    def _flare_scale(z):
        t = 1.0 - min(1.0, max(0.0, (z - HOODIE_HEM_Z) / (SPINE2_Z - HOODIE_HEM_Z)))
        return ZIP_HALF * (1.0 + 0.055 * t * t)

    def _front_edge(co):
        return abs(co.x) < 0.16 and co.y < 0.03 and co.z < 1.40

    n_snap = snap_opening(bm, _front_edge, _flare_scale)
    smooth_boundary(bm, _front_edge, iterations=8, factor=0.5)
    # the zip tape, as real geometry folded in off the panel edge
    tape = [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and _front_edge(v.co)]
    extrude_strip(bm, tape,
                  lambda co: Vector((-math.copysign(ZIP_TAPE, co.x), 0.0035, 0.0)),
                  material_index=MAT_ZIP)
    print(f"  opening: {n_snap} boundary verts snapped, {len(tape)} tape verts")

    # --- ribbed hem, off the shell's own bottom boundary --------------------
    hem = boundary_loop(bm, lambda co: co.z < HOODIE_HEM_Z + 0.012)
    extrude_band(bm, hem, [(Vector((0, 0, -0.017)), 0.997),
                           (Vector((0, 0, -0.018)), 0.988),
                           (Vector((0, 0, -0.015)), 0.994)])
    rib_displace(bm, lambda co: co.z < HOODIE_HEM_Z - 0.004, ribs=34, depth=0.0016)

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
    report_boundaries(bm, "hoodie after hem, tape and cuffs", expect=4)
    bm.to_mesh(obj.data)
    bm.free()

    solidify(obj, 0.0032)

    # --- hood and drawstrings ----------------------------------------------
    hood = build_hood()
    cords = build_cords()
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
HOOD_CENTRE = (0.0, 0.070)        # x, y of the neck axis at collar height
HOOD_RADIUS = 0.094               # from that axis out to the middle of the roll
HOOD_SWEEP = 128.0                # degrees each way from straight behind
HOOD_STEPS = 15
HOOD_SEG = 12
HOOD_Z_BACK = 1.478               # height of the roll's axis at the back
HOOD_Z_END = 1.418                # and at the two front ends
HOOD_R_BACK = 0.070               # cross-section radius at the back
HOOD_R_END = 0.040                # and at the ends, where it tapers into the seam


def build_hood():
    """A hood lying down: a soft roll around the back and sides of the neck,
    with the fabric falling behind it onto the upper back.

    A zip hoodie without a hood is a zip jacket, and the contract calls the hood
    the feature that names the garment.  What the reference actually shows is
    not a hood over the head and not a flat yoke — it is a thick bunched roll
    standing proud of the shoulders, reaching about ear height beside the neck,
    and that is the shape modelled here.  The folds are a low-frequency radial
    wobble that drifts along the sweep: a hood pushed off the head bunches, and
    a perfectly smooth tube reads as a travel pillow.
    """
    cx, cy = HOOD_CENTRE
    rings = []
    for i in range(HOOD_STEPS):
        t = i / (HOOD_STEPS - 1)                 # 0..1 across the sweep
        a = math.radians((t * 2 - 1) * HOOD_SWEEP)
        s = abs(t * 2 - 1)                       # 0 at the back, 1 at the ends
        # centre of the cross-section, on a circle around the neck
        px = cx + math.sin(a) * HOOD_RADIUS
        py = cy + math.cos(a) * HOOD_RADIUS
        pz = HOOD_Z_BACK + (HOOD_Z_END - HOOD_Z_BACK) * s ** 1.6
        r = HOOD_R_BACK + (HOOD_R_END - HOOD_R_BACK) * s ** 1.5
        # outward (away from the neck) and up
        ox, oy = math.sin(a), math.cos(a)
        ring = []
        for k in range(HOOD_SEG):
            th = math.tau * k / HOOD_SEG
            fold = 1.0 + 0.10 * math.cos(th * 3.0 + t * 7.0) + 0.05 * math.cos(th * 5.0 - t * 4.0)
            rr = r * fold
            # squash the inner side so the roll sits against the neck, not in it
            inner = 0.72 if math.cos(th) < 0 else 1.0
            ring.append((px + ox * math.cos(th) * rr * inner,
                         py + oy * math.cos(th) * rr * inner,
                         pz + math.sin(th) * rr * 1.15))
        rings.append(ring)
    verts, faces = loft(rings, cap_first=True, cap_last=True)

    # the fabric behind the roll, falling onto the upper back
    drape = []
    for j, (dz, spread, back) in enumerate((
            (1.452, 1.00, 0.008), (1.400, 1.02, 0.026), (1.348, 0.96, 0.034),
            (1.300, 0.84, 0.030), (1.262, 0.62, 0.020))):
        ring = []
        for k in range(HOOD_SEG):
            t = k / (HOOD_SEG - 1)
            a = math.radians((t * 2 - 1) * (HOOD_SWEEP - 26.0))
            wob = 0.006 * math.cos(t * 14.0 + j)
            ring.append((cx + math.sin(a) * HOOD_RADIUS * spread,
                         cy + math.cos(a) * (HOOD_RADIUS * spread + back) + wob,
                         dz))
        drape.append(ring)
    base = len(verts)
    for ring in drape:
        verts.extend(ring)
    for r in range(len(drape) - 1):
        for k in range(HOOD_SEG - 1):
            a = base + r * HOOD_SEG + k
            faces.append((a, a + 1, a + HOOD_SEG + 1, a + HOOD_SEG))
    return new_object("Hood", verts, faces, "MW_Hoodie")


def build_cords():
    """Two flat cream drawstrings hanging from the collar down the chest.

    The reference shows the character's right cord (the viewer's left) long and
    clearly readable against the tee, and the left one shorter.  The character's
    right is -X.
    """
    verts, faces = [], []
    for sx, bottom, sway in ((-1.0, 1.138, -0.014), (1.0, 1.268, 0.010)):
        path = []
        # in front of the panel, not inside it: the shell's front face sits at
        # about y = -0.11 at the collar and -0.13 at the chest
        top = Vector((sx * 0.066, -0.116, 1.424))
        n = 8
        for i in range(n):
            t = i / (n - 1)
            path.append(Vector((
                top.x + sx * (0.012 * math.sin(t * 2.4)) + sway * t,
                top.y - 0.026 * math.sin(t * 1.9) - 0.008 * t,
                top.z - (top.z - bottom) * t)))
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
BAG_CENTRE = (0.0, 0.232, 1.262)
BAG_SIZE = (0.268, 0.152, 0.350)


def build_pack(body, dom, arm):
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
        radii = [STRAP_W / 2] * len(path)
        radii[0] = STRAP_W / 2 * 0.86          # tapers into the adjuster
        v, f = tube(path, radii, segments=8, flatten=STRAP_T / STRAP_W)
        add(v, f)
        # the dark lower section the reference shows below the adjuster
        low = [Vector((sx * px, py, pz)) for px, py, pz in STRAP_PATH[:2]]
        v, f = tube([low[0] + Vector((0, 0, -0.085)), low[0]],
                    [STRAP_W / 2 * 0.80, STRAP_W / 2 * 0.84],
                    segments=8, flatten=STRAP_T / STRAP_W)
        add(v, f, lower)
        # the adjuster itself, a flat slider across the webbing
        v, f = rounded_box((sx * 0.086, -0.150, 1.118), (0.058, 0.020, 0.024),
                           0.006, segs=3, slices=2)
        add(v, f, buckle)

    n_strap = len(faces)
    v, f = rounded_box(BAG_CENTRE, BAG_SIZE, 0.055)
    add(v, f)
    # a lid flap and a lower pocket, so the silhouette is not one plain box
    v, f = rounded_box((0.0, 0.238, 1.408), (0.248, 0.140, 0.088), 0.040,
                       segs=4, slices=3)
    add(v, f)
    v, f = rounded_box((0.0, 0.272, 1.148), (0.196, 0.086, 0.118), 0.034,
                       segs=4, slices=3)
    add(v, f)

    obj = new_object("Pack", verts, faces, "MW_Pack")
    assign_material(obj, "MW_Webbing", lambda c, _n: False)
    assign_material(obj, "MW_StrapLow", lambda c, _n: False)
    assign_material(obj, "MW_Buckle", lambda c, _n: False)
    slots = {m.name: i for i, m in enumerate(obj.data.materials)}
    for i, poly in enumerate(obj.data.polygons):
        if i in buckle:
            poly.material_index = slots["MW_Buckle"]
        elif i in lower:
            poly.material_index = slots["MW_StrapLow"]
        elif i < n_strap:
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
            return c.z < crew_z(c.y) + 0.004
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
