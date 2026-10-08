#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""The default character's hair: a loose, messy, warm-brown updo, as cards.

Called by `character_model.py`.

**Why it is made rather than sourced.** CharMorph ships six grooms with the
Vitruvian character -- `Eve`, `Back1`, `Bob`, `Combover_zoro_d`,
`SceneHair_1_O4saken`, `SlickedBack` -- and none of them is an updo.
`CHARACTER_IDENTITY.md` §4 makes the hairstyle *category* a hard-fail item, so
substituting a style that ships was not open to us.  The licence search for a
redistributable CC0 updo is recorded in `docs/CHARACTER_ASSET_AUDIT.md` §13.

**How it is made** (since preview 3): **groomed**, in `hair_groom.py` -- guide
strands grown into hair by Blender's bundled CC0 hair node groups, clumped,
broken up with noise and frizz, and cut into cards.  The previous method,
ribbons swept procedurally across a fitted ellipsoid, read as a cap of flat
stripes in both previews; `docs/references/CHARACTER_ROUTE_ASSESSMENT.md` §3
explains why no tuning of it could have done otherwise.

What stays here: the **opaque cap** under the cards -- alpha-scissored cards
always leak, and without it the scalp shows through where no parting is -- and
the **brows**, which CharMorph ships as a particle system that does not survive
glTF export.

Cards are UV-mapped onto the strand atlas (`hair_atlas.py`) with **v = 0 at the
root and v = 1 at the tip** in Blender; glTF flips v, so in Godot the root is
at `UV.y = 1`, which is what `shaders/hair_card.gdshader` reads.

Frame: +Z up, +X the character's own left, -Y the way she faces.
"""

from __future__ import annotations

import math
import os
import random
import sys

import bpy  # pylint: disable=import-error
from mathutils import Vector  # pylint: disable=import-error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hair_groom  # noqa: E402  pylint: disable=wrong-import-position
from model_lib import (  # noqa: E402  pylint: disable=wrong-import-position
    dup_region, inflate, new_object, relax, set_material, shade_smooth,
)

ATLAS_COLS = hair_groom.ATLAS_COLS
CAP_MARGIN = 0.020       # the cap edge, a little below the roots that cover it
SEED = 20261006          # seeded: `ENGINEERING_STANDARDS` wants a re-runnable bake


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


def ribbon(path, widths, side_hint=Vector((1, 0, 0)), u0=0.0, u1=1.0):
    """(verts, faces, uvs) for one card swept along `path` (used by the brows)."""
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


def build_hair(body, dom, arm):
    """The cap, the groomed cards, and the brows, as one object."""
    rng = random.Random(SEED)
    centre, radii = head_frame(body, dom)
    print(f"  head ellipsoid centre {tuple(round(c, 4) for c in centre)} "
          f"radii {tuple(round(r, 4) for r in radii)}")

    cap = _build_cap(body, dom, centre, radii)
    verts, faces, uvs = hair_groom.grow(centre, radii, cap, SEED)

    # the brows are a second material: the contract calls them dark brown and
    # thick, and at the hair's own tint they disappear into the fringe
    brow_start = len(faces)
    bv, bf, bu = _brows(rng)
    base = len(verts)
    verts.extend(bv)
    uvs.extend(bu)
    faces.extend(tuple(base + i for i in q) for q in bf)

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
    print(f"  hair cards: {brow_start} card quads, {len(faces) - brow_start} brow quads")

    # one object, so the character carries one hair mesh
    for o in bpy.data.objects:
        o.select_set(False)
    cap.select_set(True)
    hair.select_set(True)
    bpy.context.view_layer.objects.active = hair
    bpy.ops.object.join()
    return hair


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
            col = rng.choice(hair_groom.DENSE_COLS)
            base = len(verts)
            v, f, u = ribbon(path, [w, w * 0.95, w * 0.35], Vector((0, 1, 0)),
                             col / ATLAS_COLS, (col + 1) / ATLAS_COLS)
            verts.extend(v)
            uvs.extend(u)
            faces.extend(tuple(base + i for i in q) for q in f)
    return verts, faces, uvs


def _build_cap(body, dom, centre, radii):
    """An opaque shell over the skull, under the cards, and the groom's scalp.

    Alpha-scissored cards leak.  Without a cap the scalp shows between strands,
    which reads as thinning hair rather than as a parting.  It is also the
    surface the sweep's guides are attached to and its children are grown on,
    so the hair starts exactly where the cap does.
    """
    def keep(f):
        if not dom.get(f.verts[0].index, "").endswith("Head"):
            return False
        c = f.calc_center_median()
        # the same hairline the guides start from, on the same ellipsoid
        d = Vector(((c.x - centre.x) / radii.x, (c.y - centre.y) / radii.y,
                    (c.z - centre.z) / radii.z))
        if d.length < 1e-5:
            return False
        d.normalize()
        az = math.degrees(math.atan2(d.x, -d.y))
        return d.z > hair_groom.hairline(az) - CAP_MARGIN

    cap = dup_region(body, keep, "HairCap")
    zs = [v.co.z for v in cap.data.vertices] or [0.0]
    print(f"  hair cap: {len(cap.data.vertices)} verts, z {min(zs):.3f}..{max(zs):.3f}")
    relax(cap, iterations=1, factor=0.3)
    inflate(cap, 0.008, smooth_first=2, steps=6)
    # not decimated: the cap's edge *is* the hairline, and a decimated edge
    # comes out as a visible sawtooth across the forehead
    set_material(cap, "MW_HairCap")
    me = cap.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    # The groom attaches roots by sampling the surface's UV map, which must be
    # one unambiguous layer: the body carries four, and the node group's
    # default reads the one named `UVMap`, which was not the active one.
    # With it, every root snapped to the origin.
    keep = me.uv_layers.active.name
    for name in [layer.name for layer in me.uv_layers]:
        if name != keep:
            me.uv_layers.remove(me.uv_layers[name])
    me.uv_layers[0].name = "UVMap"
    for attr in [a.name for a in me.color_attributes]:
        me.color_attributes.remove(me.color_attributes[attr])
    shade_smooth(cap)
    return cap
