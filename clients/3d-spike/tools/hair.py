#!/usr/bin/env python3
"""The default character's hair: a loose, messy, warm-brown updo, as cards.

Called by `character_model.py`.

**Why it is modelled rather than sourced.** CharMorph ships six grooms with the
Vitruvian character — `Eve`, `Back1`, `Bob`, `Combover_zoro_d`,
`SceneHair_1_O4saken`, `SlickedBack` — and none of them is an updo; they are
also particle grooms, which need a commercial converter to become game cards.
`CHARACTER_IDENTITY.md` §4 makes the hairstyle *category* a hard-fail item and
`VISUAL_FIDELITY.md` §5 lists it among the automatic rejections, so substituting
a style that ships was not open to us.  The licence search for a redistributable
CC0 updo is recorded in `docs/CHARACTER_ASSET_AUDIT.md`.

**How it is built.** An ellipsoid is fitted to the character's own head, and
every strand is a ribbon swept across that ellipsoid from a point on the
hairline to one gather point high and behind the crown, lifted off the skull as
it goes so the silhouette is taller and wider than the skull — which is what the
reference shows and what a flat cap of cards never gives.  On top of that sit a
wrapped twist at the gather point and a set of deliberately loose strands: the
fringe, the pieces in front of and below each ear, and the wisps at the nape.
The contract calls those strands "a large part of why the head reads as a person
rather than a mannequin", so they are geometry, not texture.

An opaque cap under the cards is not decoration either: alpha-scissored cards
always leak, and without it the scalp shows through in exactly the places a
parting would not be.

Cards are UV-mapped onto the CC0 strand atlas with **v = 0 at the root and
v = 1 at the tip**, which is the convention `shaders/hair_card.gdshader` reads
(its opacity map fades out toward v = 1 and it darkens the roots at v = 0).

Frame: +Z up, +X the character's own left, -Y the way she faces.
"""

from __future__ import annotations

import math
import os
import random
import sys

import bpy  # pylint: disable=import-error
from mathutils import Quaternion, Vector  # pylint: disable=import-error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model_lib import (  # noqa: E402  pylint: disable=wrong-import-position
    decimate, dup_region, inflate, new_object, relax, set_material, shade_smooth,
)

# The strand atlas holds several strand clumps side by side; a card takes one of
# them so neighbouring cards do not repeat the same silhouette.
ATLAS_COLS = 4

# The hairline, as an elevation on the fitted head ellipsoid against azimuth
# (0 = straight ahead).  One function, used by both the opaque cap and the roots
# of the cards, because they have to be the same line: a cap cut by a plane in
# world space left the sides of the head bare above the ears while the cards
# swept up from lower down, and the character read as having shaved sides.
HAIRLINE_A = 0.40        # how much higher the hairline is at the brow
HAIRLINE_B = -0.13       # and where it sits at the temples
CAP_MARGIN = 0.020       # the cap edge, a little below the roots that cover it


def hairline(azimuth: float) -> float:
    """Where the hair starts, plus a little irregularity.

    A hairline computed from a single cosine is a drawn line, and the opaque
    cap under the cards then ends in one — which is what made an earlier pass
    read as a headband.  Real hairlines are uneven.
    """
    return (HAIRLINE_A * math.cos(azimuth) + HAIRLINE_B
            + 0.022 * math.sin(azimuth * 7.0 + 0.6)
            + 0.013 * math.sin(azimuth * 13.0 - 1.1))
SEED = 20260927          # seeded: `ENGINEERING_STANDARDS` wants a re-runnable bake

# Where the hair is gathered: high, behind the crown.  Expressed as a fraction
# of the fitted head ellipsoid so it follows the morph rather than a constant.
# Nearly straight up and a little back.  With the gather point behind the crown
# the whole mass sat on the back of the head and the front silhouette was a
# plain skull, which read as a bowl cut; the reference's silhouette is taller
# and wider than the skull, and that only happens if the hair is piled on top.
GATHER = (0.0, 0.34, 0.94)


def head_frame(body, dom):
    """Centre and radii of an ellipsoid fitted to this character's head."""
    pts = [v.co for v in body.data.vertices
           if dom.get(v.index, "").endswith(("Head", "Neck")) and v.co.z > 1.52]
    xs = [p.x for p in pts]
    ys = [p.y for p in pts]
    zs = [p.z for p in pts]
    centre = Vector(((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2,
                     (min(zs) + max(zs)) / 2))
    radii = Vector(((max(xs) - min(xs)) / 2, (max(ys) - min(ys)) / 2,
                    (max(zs) - min(zs)) / 2))
    return centre, radii


def surface(centre, radii, direction, lift=0.0):
    """A point on (or `lift` metres off) the fitted head ellipsoid."""
    d = Vector(direction).normalized()
    return Vector((centre.x + d.x * (radii.x + lift),
                   centre.y + d.y * (radii.y + lift),
                   centre.z + d.z * (radii.z + lift)))


def ribbon(path, widths, side_hint=Vector((1, 0, 0)), u0=0.0, u1=1.0):
    """(verts, faces, uvs) for one hair card swept along `path`."""
    verts, uvs, faces = [], [], []
    n = len(path)
    for i, p in enumerate(path):
        if i == 0:
            t = (path[1] - path[0]).normalized()
        elif i == n - 1:
            t = (path[-1] - path[-2]).normalized()
        else:
            t = (path[i + 1] - path[i - 1]).normalized()
        s = t.cross(side_hint)
        if s.length < 1e-5:
            s = t.cross(Vector((0, 0, 1)))
        s.normalize()
        w = widths[i] * 0.5
        v = i / (n - 1)
        verts.append(p - s * w)
        verts.append(p + s * w)
        uvs.append((u0, v))
        uvs.append((u1, v))
    for i in range(n - 1):
        a = i * 2
        faces.append((a, a + 1, a + 3, a + 2))
    return verts, faces, uvs


def _sweep_path(centre, radii, start_dir, gather_dir, steps, volume, wobble, rng):
    """A strand running over the skull from the hairline to the gather point."""
    a = Vector(start_dir).normalized()
    b = Vector(gather_dir).normalized()
    path = []
    for i in range(steps):
        t = i / (steps - 1)
        d = (a * (1 - t) + b * t)
        if d.length < 1e-5:
            d = b
        d.normalize()
        # The standoff grows toward the gather point rather than peaking in the
        # middle: hair swept up piles behind the crown, and a mid-path bulge
        # gives a helmet instead of an updo.
        # zero at the root, but standing off early: the mass has to be wide at
        # the temples, not just thick at the crown
        lift = volume * (t ** 0.45) * (0.45 + 0.55 * t)
        lift += wobble * math.sin(t * 9.0 + rng.random() * 0.3)
        path.append(surface(centre, radii, d, lift))
    return path


def build_hair(body, dom, arm):
    """The cap, the swept mass, the twist at the crown and the loose strands."""
    rng = random.Random(SEED)
    centre, radii = head_frame(body, dom)
    print(f"  head ellipsoid centre {tuple(round(c, 4) for c in centre)} "
          f"radii {tuple(round(r, 4) for r in radii)}")

    cap = _build_cap(body, dom)
    verts, faces, uvs = [], [], []

    def add(v, f, u):
        base = len(verts)
        verts.extend(v)
        uvs.extend(u)
        faces.extend(tuple(base + i for i in q) for q in f)

    gather = Vector(GATHER)

    # --- the swept mass ---------------------------------------------------
    # Clumps, not a fan of identical cards.  A uniform fan of ribbons following
    # one smooth surface renders as a solid dome — a cap, not hair.  Hair
    # separates into locks: each clump here shares a path, a standoff and a
    # drift, and the clumps differ from each other far more than the cards
    # inside one differ.  That separation is most of what reads as hair.
    for c in range(30):
        ct = c / 29.0
        c_ang = math.radians(-176 + 352 * ct)
        c_vol = 0.030 + 0.048 * rng.random()
        c_drift = rng.uniform(-0.16, 0.16)
        c_over = rng.uniform(0.0, 0.16)      # how far past the gather it splays
        for j in range(5):
            ang = c_ang + rng.uniform(-0.07, 0.07)
            elev = hairline(ang) + rng.uniform(-0.03, 0.03)
            horiz = math.sqrt(max(0.04, 1.0 - elev * elev))
            start = (math.sin(ang) * horiz, -math.cos(ang) * horiz, elev)
            jitter = Vector(GATHER) + Vector((
                c_drift + rng.uniform(-0.05, 0.05),
                rng.uniform(-0.05, 0.05) - c_over,
                rng.uniform(-0.04, 0.04) + c_over))
            path = _sweep_path(centre, radii, start, jitter, 7,
                               c_vol * rng.uniform(0.85, 1.15),
                               0.003 * rng.random(), rng)
            w0 = 0.016 + 0.008 * rng.random()
            w = [w0 * q for q in (0.9, 1.0, 1.0, 0.95, 0.85, 0.65, 0.4)]
            col = rng.randrange(ATLAS_COLS)
            add(*ribbon(path, w, Vector((0, 0, 1)),
                        col / ATLAS_COLS, (col + 1) / ATLAS_COLS))

    # --- the twist at the crown ------------------------------------------
    # Short arcs wrapped in every direction around a small core, not one ring.
    # A ring of cards through a single circle came out as a doughnut standing
    # off the back of the head; hair gathered into a soft twist is a rounded
    # mass with strands crossing it at every angle.
    g = surface(centre, radii, gather, 0.030)
    bun = Vector((0.060, 0.056, 0.046))
    for _ in range(52):
        axis = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1),
                       rng.uniform(-1, 1)))
        if axis.length < 1e-3:
            axis = Vector((0, 0, 1))
        axis.normalize()
        seed_dir = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1),
                           rng.uniform(-1, 1)))
        start = (seed_dir - axis * seed_dir.dot(axis))
        if start.length < 1e-3:
            continue
        start.normalize()
        span = math.radians(rng.uniform(130, 210))
        n = 8
        loops = []
        for i in range(n):
            s = i / (n - 1)
            d = start.copy()
            d.rotate(Quaternion(axis, span * s))
            r = 1.0 + 0.10 * math.sin(s * math.pi * 1.6)
            loops.append(Vector((g.x + d.x * bun.x * r,
                                 g.y + d.y * bun.y * r,
                                 g.z + d.z * bun.z * r)))
        w0 = rng.uniform(0.016, 0.026)
        w = [w0 * k for k in (0.5, 0.9, 1.0, 1.0, 0.95, 0.8, 0.6, 0.3)]
        col = rng.randrange(ATLAS_COLS)
        add(*ribbon(loops, w, Vector((0, 0, 1)),
                    col / ATLAS_COLS, (col + 1) / ATLAS_COLS))

    # --- the hairline ------------------------------------------------------
    # Short cards rooted above the hairline and pointing *down* across it.  The
    # swept cards all start at the hairline and run upward, so their opaque
    # roots form a hard edge, and the opaque cap under them ends in a second
    # one: without this layer the character reads as wearing a brown swim cap.
    # These have their faded tips at the bottom, which is what a hairline is.
    for k in range(46):
        t = k / 45.0
        ang = math.radians(-178 + 356 * t) + rng.uniform(-0.04, 0.04)
        base_el = hairline(ang)
        # length and reach vary per card; a row of equal-length cards produced a
        # continuous brim round the head, which is a hat, not a hairline
        reach = rng.uniform(0.030, 0.085)
        rise = rng.uniform(0.075, 0.150)
        n = 5
        path = []
        for i in range(n):
            s_ = i / (n - 1)
            el = base_el + rise * (1.0 - s_) - reach * s_
            d = _dir(math.degrees(ang) + 7.0 * math.sin(t * 21.0) * s_, el)
            path.append(surface(centre, radii, d,
                                0.012 + 0.012 * math.sin(math.pi * s_)))
        w0 = 0.009 + 0.006 * rng.random()
        col = rng.randrange(ATLAS_COLS)
        add(*ribbon(path, [w0 * q for q in (0.8, 1.0, 1.0, 0.85, 0.55)],
                    Vector((0, 0, 1)), col / ATLAS_COLS, (col + 1) / ATLAS_COLS))

    # --- the loose strands ------------------------------------------------
    add(*_loose(centre, radii, rng))

    hair = new_object("Hair", verts, faces, "MW_Hair")
    me = hair.data
    lay = me.uv_layers.new(name="UVMap")
    for poly in me.polygons:
        for li, vi in zip(poly.loop_indices, poly.vertices):
            lay.data[li].uv = uvs[vi]
    shade_smooth(hair)

    # one object, so the character carries one hair mesh
    for o in bpy.data.objects:
        o.select_set(False)
    cap.select_set(True)
    hair.select_set(True)
    bpy.context.view_layer.objects.active = hair
    bpy.ops.object.join()
    return hair


# The loose strands the contract names, one entry each, written out rather than
# randomised: "a soft fringe sweeping across the forehead to the character's
# right, strands in front of and below the ear on both sides, and wisps at the
# nape".  Which strands exist is an identity fact, so it is data, not noise.

# Fringe: (root azimuth°, root elevation, end azimuth°, end elevation,
#          lift off the skull, tip drop, width)
FRINGE = [
    (34, 0.58, -54, 0.22, 0.013, -0.016, 0.014),
    (20, 0.61, -48, 0.26, 0.014, -0.013, 0.015),
    (6, 0.62, -42, 0.30, 0.012, -0.011, 0.013),
    (46, 0.50, -28, 0.36, 0.011, -0.009, 0.011),
]

# Falling strands: (azimuth°, elevation, length, tip drop, wave, width)
FALL = [
    # in front of the ears, both sides
    (-74, 0.06, 0.150, -0.155, 0.014, 0.016),
    (-84, -0.06, 0.130, -0.140, 0.012, 0.013),
    (76, 0.06, 0.155, -0.160, 0.015, 0.016),
    (86, -0.06, 0.135, -0.145, 0.013, 0.013),
    # below and behind the ears
    (-102, -0.18, 0.120, -0.125, 0.011, 0.012),
    (104, -0.18, 0.125, -0.130, 0.012, 0.012),
    # wisps at the nape
    (156, -0.42, 0.100, -0.105, 0.009, 0.011),
    (172, -0.48, 0.090, -0.095, 0.008, 0.010),
    (-158, -0.42, 0.105, -0.110, 0.010, 0.011),
    (-174, -0.48, 0.085, -0.090, 0.008, 0.010),
]


def _dir(az_deg, elev):
    """A unit direction from the head centre: azimuth 0 is straight ahead."""
    a = math.radians(az_deg)
    horiz = math.sqrt(max(0.02, 1.0 - elev * elev))
    return Vector((math.sin(a) * horiz, -math.cos(a) * horiz, elev))


def _loose(centre, radii, rng):
    verts, faces, uvs = [], [], []

    def emit(path, widths):
        col = rng.randrange(ATLAS_COLS)
        base = len(verts)
        v, f, u = ribbon(path, widths, Vector((0, 0, 1)),
                         col / ATLAS_COLS, (col + 1) / ATLAS_COLS)
        verts.extend(v)
        uvs.extend(u)
        faces.extend(tuple(base + i for i in q) for q in f)

    # the fringe sweeps *across* the forehead; it does not hang down it
    for az0, el0, az1, el1, lift, drop, width in FRINGE:
        n = 7
        path = []
        for i in range(n):
            t = i / (n - 1)
            d = _dir(az0 + (az1 - az0) * t, el0 + (el1 - el0) * t)
            p = surface(centre, radii, d, lift * math.sin(math.pi * t) + 0.008)
            p.z += drop * t * t
            path.append(p)
        emit(path, [width * s for s in (0.9, 1.0, 1.0, 0.95, 0.8, 0.6, 0.35)])

    for az, el, length, drop, wave, width in FALL:
        d = _dir(az, el)
        root = surface(centre, radii, d, 0.010)
        out = Vector((d.x, d.y, 0.0))
        if out.length < 1e-5:
            out = Vector((0, 1, 0))
        out.normalize()
        n = 7
        path = []
        for i in range(n):
            t = i / (n - 1)
            # a loose strand mostly falls; it curls out at the tip but does not
            # stand away from the head, which an earlier version had it doing
            path.append(Vector((
                root.x + out.x * length * t * t * 0.22 + wave * math.sin(t * 5.2),
                root.y + out.y * length * t * t * 0.22 + wave * 0.6 * math.cos(t * 4.4),
                root.z + drop * t + wave * 0.4 * math.sin(t * 6.0))))
        emit(path, [width * s for s in (1.0, 1.05, 1.0, 0.9, 0.75, 0.55, 0.3)])
    return verts, faces, uvs


def _build_cap(body, dom):
    """An opaque shell over the skull, under the cards.

    Alpha-scissored cards leak.  Without a cap the scalp shows between strands,
    which reads as thinning hair rather than as a parting.
    """
    centre, radii = head_frame(body, dom)

    def keep(f):
        if not dom.get(f.verts[0].index, "").endswith("Head"):
            return False
        c = f.calc_center_median()
        # the same hairline the cards start from, expressed on the same
        # ellipsoid, so the cap covers exactly what the hair covers
        d = Vector(((c.x - centre.x) / radii.x, (c.y - centre.y) / radii.y,
                    (c.z - centre.z) / radii.z))
        if d.length < 1e-5:
            return False
        d.normalize()
        az = math.atan2(d.x, -d.y)
        return d.z > hairline(az) - CAP_MARGIN

    cap = dup_region(body, keep, "HairCap")
    zs = [v.co.z for v in cap.data.vertices] or [0.0]
    print(f"  hair cap: {len(cap.data.vertices)} verts, z {min(zs):.3f}..{max(zs):.3f}")
    relax(cap, iterations=1, factor=0.3)
    inflate(cap, 0.011, smooth_first=2)
    # not decimated: the cap's edge *is* the hairline, and a decimated edge
    # comes out as a visible sawtooth across the forehead
    set_material(cap, "MW_HairCap")
    me = cap.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    shade_smooth(cap)
    return cap
