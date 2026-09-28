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
# Measured against this face, not tuned by feel: her brows sit at z = 1.656 to
# 1.664 and the head ellipsoid is centred at z = 1.634 with a vertical radius of
# 0.113, so a hairline elevation of 0.52 puts it at z = 1.693 -- a forehead
# about 30 mm deep. An earlier 0.40/-0.13 put the cap edge at z = 1.664, which
# is exactly on the brow ridge: she had no forehead and the eyebrows were buried
# under the hair.
HAIRLINE_A = 0.62        # how much higher the hairline is at the brow
HAIRLINE_B = -0.10       # and where it sits at the temples
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


def centre_of(centre, radii, point):
    """The ellipsoid centre, as the thing a point is pushed away from."""
    return Vector(centre)


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
        # A swept-up style hugs the skull and gets its height from the twist,
        # not from the sweep.  Lifting the cards 40-80 mm off the head made
        # every clump a spike and the whole groom read as straw.
        lift = volume * (t ** 0.5)
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
        c_vol = 0.014 + 0.026 * rng.random()
        c_drift = rng.uniform(-0.16, 0.16)
        c_over = rng.uniform(0.0, 0.05)      # how far past the gather it splays
        for j in range(5):
            ang = c_ang + rng.uniform(-0.07, 0.07)
            elev = hairline(ang) + rng.uniform(-0.03, 0.03)
            horiz = math.sqrt(max(0.04, 1.0 - elev * elev))
            start = (math.sin(ang) * horiz, -math.cos(ang) * horiz, elev)
            jitter = Vector(GATHER) + Vector((
                c_drift + rng.uniform(-0.05, 0.05),
                rng.uniform(-0.05, 0.05) - c_over,
                rng.uniform(-0.04, 0.04) + c_over))
            path = _sweep_path(centre, radii, start, jitter, 9,
                               c_vol * rng.uniform(0.85, 1.15),
                               0.003 * rng.random(), rng)
            # Layered like roof tiles, not one ribbon from hairline to crown.
            # A card is opaque at its root and fades to its tip, so a single
            # long card per clump puts every opaque root in the same band and
            # they merge into a smooth shell -- which is why the scalp read as
            # a swim cap with strands only near the crown. Three shorter,
            # overlapping cards put a tapered end partway up the head as well.
            for lo, hi, off in ((0, 5, 0.000), (2, 7, 0.004), (4, 9, 0.008)):
                seg = [p + (p - centre_of(centre, radii, p)).normalized() * off
                       for p in path[lo:hi]]
                w0 = 0.020 + 0.010 * rng.random()
                n = len(seg)
                w = [w0 * (1.0 - 0.72 * (i / (n - 1)) ** 1.6) for i in range(n)]
                col = rng.randrange(ATLAS_COLS)
                add(*ribbon(seg, w, Vector((0, 0, 1)),
                            col / ATLAS_COLS, (col + 1) / ATLAS_COLS))

    # The hairline used to get its own layer of short cards pointing down
    # across it, to soften the opaque cap's edge.  It made a ring of separate
    # spikes round the head -- a crown of thorns, not a hairline -- because the
    # cards are tapered locks and a row of locks pointing the same way reads as
    # a fringe of spikes.  The swept clumps' own roots are wide and opaque at
    # v = 0 and cover the cap edge on their own, so the layer is gone.


    # --- the loose strands ------------------------------------------------
    add(*_loose(centre, radii, rng))

    # the brows are a second material: the contract calls them dark brown and
    # thick, and at the hair's own tint they disappear into the fringe
    brow_start = len(faces)
    bv, bf, bu = _brows(rng)
    add(bv, bf, bu)
    hair = new_object("Hair", verts, faces, "MW_Hair")
    brow_mat = bpy.data.materials.get("MW_Brow") or bpy.data.materials.new("MW_Brow")
    hair.data.materials.append(brow_mat)
    for i, poly in enumerate(hair.data.polygons):
        if i >= brow_start:
            poly.material_index = 1
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
                root.x + out.x * length * t * t * 0.16 + wave * math.sin(t * 5.2),
                root.y + out.y * length * t * t * 0.16 + wave * 0.6 * math.cos(t * 4.4),
                root.z + drop * t + wave * 0.4 * math.sin(t * 6.0))))
        emit(path, [width * s for s in (1.0, 1.05, 1.0, 0.9, 0.75, 0.55, 0.3)])
    return verts, faces, uvs


# The brow arc, her left side, inner to outer: (x, y, z).  CharMorph ships the
# eyebrows as a particle system, which does not survive a glTF export, so the
# exported face has none at all -- and `CHARACTER_IDENTITY.md` §3 asks for
# "dark brown, thick, naturally arched".  A face without brows does not read as
# a face; it reads as a mannequin, whatever else is right about it.
BROW_ARC = [(0.013, -0.0712, 1.6560), (0.024, -0.0700, 1.6615),
            (0.035, -0.0662, 1.6642), (0.046, -0.0596, 1.6625),
            (0.056, -0.0516, 1.6558)]
BROW_CARDS = 15
BROW_LEN = 0.0105
BROW_W = 0.0052


def _brow_point(t: float, sx: float) -> Vector:
    """A point along the brow arc, `t` from inner (0) to outer (1)."""
    f = t * (len(BROW_ARC) - 1)
    i = min(int(f), len(BROW_ARC) - 2)
    u = f - i
    a = BROW_ARC[i]
    b = BROW_ARC[i + 1]
    return Vector((sx * (a[0] + (b[0] - a[0]) * u),
                   a[1] + (b[1] - a[1]) * u,
                   a[2] + (b[2] - a[2]) * u))


def _brows(rng):
    """Short cards laid along each brow ridge, angled the way brow hair grows."""
    verts, faces, uvs = [], [], []
    for sx in (-1.0, 1.0):
        for k in range(BROW_CARDS):
            t = (k + rng.uniform(-0.25, 0.25)) / (BROW_CARDS - 1)
            t = min(1.0, max(0.0, t))
            p0 = _brow_point(t, sx)
            # brow hair sweeps outward and slightly up near the inner end,
            # outward and slightly down past the arch
            tangent = (_brow_point(min(1.0, t + 0.08), sx)
                       - _brow_point(max(0.0, t - 0.08), sx))
            if tangent.length < 1e-6:
                continue
            tangent.normalize()
            rise = 0.35 * (1.0 - t) - 0.20 * t
            tip = p0 + tangent * BROW_LEN * rng.uniform(0.8, 1.25) \
                + Vector((0, -0.0016, rise * BROW_LEN))
            path = [p0, p0 + (tip - p0) * 0.5, tip]
            w = BROW_W * rng.uniform(0.8, 1.2)
            col = rng.randrange(ATLAS_COLS)
            base = len(verts)
            v, f, u = ribbon(path, [w, w * 0.95, w * 0.35], Vector((0, 1, 0)),
                             col / ATLAS_COLS, (col + 1) / ATLAS_COLS)
            verts.extend(v)
            uvs.extend(u)
            faces.extend(tuple(base + i for i in q) for q in f)
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
    inflate(cap, 0.011, smooth_first=2, steps=6)
    # not decimated: the cap's edge *is* the hairline, and a decimated edge
    # comes out as a visible sawtooth across the forehead
    set_material(cap, "MW_HairCap")
    me = cap.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    shade_smooth(cap)
    return cap
