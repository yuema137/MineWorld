#!/usr/bin/env python3
"""The updo as a groom: guide strands, grown into hair by Blender's own hair
node groups, then cut into cards.

Called by `hair.py`.  Frame: +Z up, +X the character's own left, -Y the way
she faces.

**Why a groom.**  The previous hair swept ribbons procedurally across a fitted
ellipsoid, and it read as a cap of stripes in every preview: nothing in that
method clumps, layers or separates strands (`CHARACTER_ROUTE_ASSESSMENT.md`
§3).  Hair artists work the other way round: a few dozen *guide* strands set
the shape, the hair system grows thousands of children between them, clumps
them into locks and breaks them up with noise and frizz, and the result is
then cut into cards for a real-time engine.  This file does exactly that with
the hair node groups Blender bundles as Essentials assets
(`datafiles/assets/nodes/procedural_hair_node_assets.blend`, CC0 under the
Blender asset-bundle guidelines): *Attach Hair Curves to Surface*,
*Interpolate Hair Curves*, *Clump Hair Curves*, *Duplicate Hair Curves*,
*Hair Curves Noise*, *Frizz Hair Curves* and *Curl Hair Curves*.  What is
authored here is only the guides -- the shape of the style -- and they are
data, so the style is reviewable and re-runnable.

**The style, from `CHARACTER_IDENTITY.md` §4**: warm mid-brown, gathered up
and back into a soft twist at the crown, the silhouette wider and taller than
the skull; a soft fringe sweeping across the forehead to her right; wavy
strands in front of and below each ear; wisps at the nape.

Three families, each groomed by its own node chain:

```text
sweep   hairline and scalp -> up and back into the base of the bun
        guides attached to the scalp; children interpolated over it, clumped
bun     a loose twisted knot high at the crown, with ends escaping it
        guides duplicated around themselves, frizzed
loose   the face-framing strands, the fringe and the nape wisps
        guides duplicated into thin locks, curled, frizzed
```

Each grown strand becomes one card: a ribbon lying across the surface it
grows from, `v` = 0 at the root and 1 at the tip, `u` one column of the strand
atlas (`hair_atlas.py`).
"""

from __future__ import annotations

import math
import random

import bpy  # pylint: disable=import-error
from mathutils import Vector  # pylint: disable=import-error

ESSENTIALS = ("procedural_hair_node_assets.blend",)
GROUPS = ["Attach Hair Curves to Surface", "Interpolate Hair Curves",
          "Clump Hair Curves", "Duplicate Hair Curves", "Hair Curves Noise",
          "Frizz Hair Curves", "Curl Hair Curves", "Smooth Hair Curves"]

ATLAS_COLS = 8
DENSE_COLS = (0, 1, 2, 3, 4, 5)
WISP_COLS = (5, 6, 7)
# the face-framing locks are locks, not single hairs: mostly the mid-density
# columns, with a wisp now and then
LOOSE_COLS = (0, 1, 2, 3, 4, 6)


# ----------------------------------------------------------------- node groups

def _essentials_path() -> str:
    import os  # pylint: disable=import-outside-toplevel
    base = bpy.utils.system_resource("DATAFILES")
    return os.path.join(base, "assets", "nodes", ESSENTIALS[0])


def load_groups() -> None:
    """Append the Essentials hair node groups once."""
    missing = [g for g in GROUPS if g not in bpy.data.node_groups]
    if not missing:
        return
    with bpy.data.libraries.load(_essentials_path()) as (_src, dst):
        dst.node_groups = list(missing)
    for g in GROUPS:
        if g not in bpy.data.node_groups:
            raise RuntimeError(f"Essentials hair node group not found: {g}")


def groom(obj, chain) -> None:
    """Put a geometry-nodes modifier on `obj` running `chain` in order.

    `chain` is a list of `(group name, {input name: value})`.  Inputs are set
    by name and an unknown name is an error, so a renamed socket in a later
    Blender cannot silently fall back to a default.
    """
    tree = bpy.data.node_groups.new(f"Groom_{obj.name}", "GeometryNodeTree")
    tree.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    tree.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    gin = tree.nodes.new("NodeGroupInput")
    gout = tree.nodes.new("NodeGroupOutput")
    prev = gin.outputs[0]
    import os  # pylint: disable=import-outside-toplevel
    cut = os.environ.get("MW_GROOM_CUT_" + obj.name)
    if cut is not None:                  # diagnostic: run only the first N steps
        chain = chain[:int(cut)]
    for i, (name, values) in enumerate(chain):
        node = tree.nodes.new("GeometryNodeGroup")
        node.node_tree = bpy.data.node_groups[name]
        node.location = (200 * (i + 1), 0)
        for key, val in values.items():
            sock = next((s for s in node.inputs if s.name == key and s.enabled), None)
            if sock is None:
                raise KeyError(f"{name}: no input {key!r}")
            sock.default_value = val
        tree.links.new(prev, node.inputs[0])
        prev = node.outputs[0]
    tree.links.new(prev, gout.inputs[0])
    print(f"    groom {obj.name}: {' -> '.join(n for n, _ in chain)}")
    mod = obj.modifiers.new("Groom", "NODES")
    mod.node_group = tree


# ---------------------------------------------------------------- curves I/O

def new_curves(name: str, strands, surface=None):
    """A hair-curves object holding `strands` (lists of Vectors)."""
    cd = bpy.data.hair_curves.new(name)
    cd.add_curves([len(s) for s in strands])
    flat = [c for s in strands for p in s for c in (p.x, p.y, p.z)]
    cd.attributes["position"].data.foreach_set("vector", flat)
    obj = bpy.data.objects.new(name, cd)
    bpy.context.scene.collection.objects.link(obj)
    if surface is not None:
        cd.surface = surface
        cd.surface_uv_map = surface.data.uv_layers.active.name
    return obj


def evaluated_strands(obj):
    """The grown strands, as lists of Vectors, after the modifier stack."""
    dg = bpy.context.evaluated_depsgraph_get()
    ev = obj.evaluated_get(dg)
    cd = ev.data
    n_pts = len(cd.points)
    print(f"    evaluated {obj.name}: {len(cd.curves)} curves, {n_pts} points, "
          f"attributes {[a.name for a in cd.attributes]}")
    flat = [0.0] * (3 * n_pts)
    cd.attributes["position"].data.foreach_get("vector", flat)
    offs = [0] * (len(cd.curves) + 1)
    cd.curve_offset_data.foreach_get("value", offs)
    out = []
    for i in range(len(cd.curves)):
        a, b = offs[i], offs[i + 1]
        out.append([Vector(flat[3 * k:3 * k + 3]) for k in range(a, b)])
    return out


def resample(path, n):
    """`n` points evenly spaced along the polyline `path`."""
    seg = [(path[i + 1] - path[i]).length for i in range(len(path) - 1)]
    total = sum(seg)
    if total < 1e-6:
        return [path[0].copy() for _ in range(n)]
    out = []
    k, acc = 0, 0.0
    for j in range(n):
        target = total * j / (n - 1)
        while k < len(seg) - 1 and acc + seg[k] < target:
            acc += seg[k]
            k += 1
        t = 0.0 if seg[k] < 1e-9 else (target - acc) / seg[k]
        out.append(path[k].lerp(path[k + 1], min(1.0, max(0.0, t))))
    return out


# --------------------------------------------------------------------- frame

class Head:
    """The fitted head ellipsoid, and directions on it by azimuth/elevation.

    Azimuth 0 is straight ahead (-Y), +90 her left (+X); elevation is the
    sine of the angle above the ellipsoid's equator.
    """

    def __init__(self, centre, radii):
        self.c = Vector(centre)
        self.r = Vector(radii)

    @staticmethod
    def dir(az_deg, elev):
        a = math.radians(az_deg)
        elev = max(-0.99, min(0.99, elev))
        h = math.sqrt(1.0 - elev * elev)
        return Vector((math.sin(a) * h, -math.cos(a) * h, elev))

    def at(self, az_deg, elev, lift=0.0):
        d = self.dir(az_deg, elev)
        return Vector((self.c.x + d.x * (self.r.x + lift),
                       self.c.y + d.y * (self.r.y + lift),
                       self.c.z + d.z * (self.r.z + lift)))


# ------------------------------------------------------------------ the bun

def bun_centre(head: Head) -> Vector:
    # High on the crown and a little behind it.  The reference's mass rises
    # about 60 mm above the top of the skull and is wider than half the
    # head; this centre and BUN_RADII put its top and sides there.
    return head.c + Vector((0.002, 0.036, head.r.z + 0.030))


# The knot, as an ellipsoid.  Narrower than the swept mass it sits on: a knot
# wider than the head below it left a shelf round the crown, and the head
# read as a beehive.
BUN_RADII = Vector((0.045, 0.042, 0.038))
# Lock ends escaping the knot.  Off: at portrait distance each one read as a
# spike standing off the top of the head; the messiness comes from the bowed
# loops and the frizz instead.
ESCAPES = False


def bun_guides(head: Head, rng):
    """A soft twisted knot: locks wrapped round it on tilted great circles.

    Each lock runs part of the way round the knot's ellipsoid on a circle of
    random tilt, so the locks cross one another the way a twisted bun's do.
    A quarter of them bow away from the surface in the middle -- the loose
    loops of a bun that was not pinned tight -- and every third one ends
    escaping the knot, which is what makes an updo read as messy rather than
    as a sculpted ball.
    """
    b = bun_centre(head)
    out = []
    for i in range(34):
        axis = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.6, 0.6)))
        if axis.length < 1e-3:
            axis = Vector((0, 0, 1))
        axis.normalize()
        u = axis.orthogonal().normalized()
        w = axis.cross(u).normalized()
        th0 = rng.uniform(0, math.tau)
        span = rng.uniform(1.0, 1.7) * math.pi
        layer = rng.uniform(0.0, 0.10)
        bow = rng.uniform(0.15, 0.35) if i % 4 == 0 else 0.0
        n = 18
        path = []
        for k in range(n):
            t = k / (n - 1)
            th = th0 + span * t
            d = u * math.cos(th) + w * math.sin(th)
            # under the knot is the head; a lock does not pass through it
            if d.z < -0.35:
                d.z = -0.35 + (d.z + 0.35) * 0.2
            s = 1.0 + layer + bow * math.sin(math.pi * t)
            path.append(b + Vector((d.x * BUN_RADII.x * s, d.y * BUN_RADII.y * s,
                                    d.z * BUN_RADII.z * s)))
        if ESCAPES and i % 6 == 0:
            tip, prev = path[-1], path[-2]
            go = (tip - prev).normalized()
            # mostly along the lock and falling a little: an end pointing
            # straight out of the knot is a spike, not a strand
            out_dir = go
            for k in range(1, 6):
                path.append(tip + out_dir * 0.007 * k + Vector((0, 0, -0.005 * k * k / 5)))
        out.append(path)
    return out


# --------------------------------------------------------------- the sweep

def hairline(az_deg: float) -> float:
    """Shared with the cap in `hair.py`: where the hair starts."""
    a = math.radians(az_deg)
    return (0.62 * math.cos(a) - 0.10
            + 0.022 * math.sin(a * 7.0 + 0.6)
            + 0.013 * math.sin(a * 13.0 - 1.1))


def sweep_guides(head: Head, rng):
    """Guides from the hairline and the scalp up and back into the bun."""
    b = bun_centre(head)
    out = []
    for az in range(-180, 180, 12):
        az = az + rng.uniform(-3, 3)
        e0 = hairline(az)
        for lvl, e in enumerate((e0 + 0.02, e0 + 0.30, e0 + 0.58)):
            if e > 0.93:
                continue
            root = head.at(az, e, 0.004)
            # where this strand enters the knot: on the knot's lower rim, on
            # the strand's own side of it
            a = math.radians(az)
            # into the knot, not onto its rim
            end = b + Vector((math.sin(a) * BUN_RADII.x * 0.40,
                              -math.cos(a) * BUN_RADII.y * 0.35, -BUN_RADII.z * 0.35))
            n = 12
            path = []
            # The sweep follows the skull with volume that grows toward the
            # crown: a gathered style is close at the hairline and fuller on
            # top.  The front gets more lift, as the reference's soft
            # pompadour has, and so do the sides, where the reference's
            # silhouette is wider than the skull.
            # The back is pulled close: the mass belongs in the knot, and a
            # full dome behind the crown buried it.
            front = max(0.0, math.cos(a))
            back = max(0.0, -math.cos(a))
            sides = abs(math.sin(a))
            vol = (0.008 + 0.010 * front + 0.008 * sides - 0.004 * back
                   + rng.uniform(0.0, 0.004))
            for k in range(n):
                t = k / (n - 1)
                d0 = head.dir(az, e)
                d1 = (end - head.c)
                d1 = Vector((d1.x / head.r.x, d1.y / head.r.y, d1.z / head.r.z)).normalized()
                d = d0.lerp(d1, t).normalized()
                lift = 0.004 + vol * math.sin(math.pi * min(1.0, t * 1.15)) ** 0.8
                p = Vector((head.c.x + d.x * (head.r.x + lift),
                            head.c.y + d.y * (head.r.y + lift),
                            head.c.z + d.z * (head.r.z + lift)))
                # the last stretch leaves the skull for the knot
                if t > 0.75:
                    w = (t - 0.75) / 0.25
                    p = p.lerp(end, w * w)
                path.append(p)
            out.append(path)
    return out


# ------------------------------------------------------------- loose strands

# Written out, not randomised: which strands exist is an identity fact
# (`CHARACTER_IDENTITY.md` §4).  The fringe sweeps across the forehead from
# her left to her right and falls past the temple; the front pieces hang in
# front of the ears to the jaw; more behind the ears; wisps at the nape.
#
# Fringe: (root az, root elev, end az, end elev, fall below the end, waves)
# The fringe is rooted well behind the hairline, on top of the head, and
# swoops forward and down across the forehead, so it covers the upper third
# of the forehead on her right as the reference's does.  Rooted at the
# hairline it read as the brim of a hat.
FRINGE = [
    (30, 0.80, -62, 0.10, 0.060, 1.2),
    (18, 0.82, -56, 0.16, 0.040, 1.0),
    (40, 0.76, -46, 0.26, 0.020, 0.9),
    (8, 0.84, -70, 0.00, 0.085, 1.4),
    (24, 0.80, -66, 0.04, 0.075, 1.3),
    (36, 0.78, -58, 0.12, 0.050, 1.1),
    (14, 0.83, -40, 0.30, 0.012, 0.8),
    (2, 0.86, -30, 0.36, 0.000, 0.7),
]
# Falling: (az, elev, length, waves, outward flick)
FALL = [
    (-76, 0.20, 0.165, 1.6, 0.012),     # her right, in front of the ear
    (-84, 0.06, 0.150, 1.8, 0.010),
    (-70, 0.32, 0.120, 1.3, 0.008),
    (74, 0.22, 0.175, 1.7, 0.014),      # her left, in front of the ear
    (84, 0.08, 0.160, 1.9, 0.012),
    (68, 0.34, 0.125, 1.4, 0.010),
    (-80, 0.12, 0.140, 2.0, 0.014),
    (78, 0.14, 0.150, 2.1, 0.016),
    (92, -0.02, 0.140, 1.6, 0.010),
    (-104, -0.10, 0.090, 1.5, 0.008),   # behind the ears
    (106, -0.10, 0.095, 1.5, 0.008),
    (150, -0.40, 0.050, 1.0, 0.006),    # the nape: short, "wisps"
    (170, -0.46, 0.045, 0.9, 0.005),
    (-152, -0.40, 0.055, 1.0, 0.006),
    (-172, -0.46, 0.040, 0.8, 0.005),
]


def loose_guides(head: Head, rng):
    out = []
    for az0, el0, az1, el1, fall, waves in FRINGE:
        n = 16
        path = []
        for k in range(n):
            t = k / (n - 1)
            # across the brow first, then falling past the temple
            s = min(1.0, t / 0.7)
            p = head.at(az0 + (az1 - az0) * s, el0 + (el1 - el0) * s,
                        0.004 + 0.006 * math.sin(math.pi * s))
            if t > 0.7:
                p.z -= fall * ((t - 0.7) / 0.3)
            p.z += 0.003 * math.sin(t * math.tau * waves)
            path.append(p)
        out.append(path)
    for az, el, length, waves, flick in FALL:
        root = head.at(az, el, 0.010)
        d = head.dir(az, el)
        side = Vector((d.x, d.y, 0.0))
        if side.length < 1e-5:
            side = Vector((0, 1, 0))
        side.normalize()
        # a falling strand hangs clear of the cheek, so it starts by
        # leaving the head along its own outward direction
        tang = side.cross(Vector((0, 0, 1))).normalized()
        n = 16
        path = []
        for k in range(n):
            t = k / (n - 1)
            amp = 0.007 * t
            p = (root + side * (0.006 * math.sin(math.pi * min(1.0, t * 2)) + flick * t * t)
                 + Vector((0, 0, -length * t))
                 + tang * amp * math.sin(t * math.tau * waves + az))
            path.append(p)
        out.append(path)
    return out


# --------------------------------------------------------------------- cards

def cards_from(strands, width_root, width_tip, centre_of, cols, rng, points=9):
    """(verts, faces, uvs): one card per strand, lying across its surface."""
    verts, faces, uvs = [], [], []
    for s in strands:
        if len(s) < 2:
            continue
        path = resample(s, points)
        col = rng.choice(cols)
        u0, u1 = col / ATLAS_COLS, (col + 1) / ATLAS_COLS
        base = len(verts)
        w0 = width_root * rng.uniform(0.8, 1.2)
        w1 = width_tip * rng.uniform(0.8, 1.2)
        prev_side = None
        for i, p in enumerate(path):
            if i == 0:
                t = path[1] - path[0]
            elif i == len(path) - 1:
                t = path[-1] - path[-2]
            else:
                t = path[i + 1] - path[i - 1]
            t.normalize()
            out = (p - centre_of(p)).normalized()
            side = t.cross(out)
            # where the strand points along `out` the cross product is noise;
            # keep the previous orientation rather than twisting the card
            if side.length < 0.25:
                side = prev_side.copy() if prev_side is not None else t.orthogonal()
            side.normalize()
            if prev_side is not None and side.dot(prev_side) < 0:
                side = -side
            prev_side = side
            v = i / (len(path) - 1)
            w = (w0 + (w1 - w0) * v) * 0.5
            verts.append(p - side * w)
            verts.append(p + side * w)
            uvs.append((u0, v))
            uvs.append((u1, v))
        for i in range(len(path) - 1):
            a = base + 2 * i
            faces.append((a, a + 1, a + 3, a + 2))
    return verts, faces, uvs


# ---------------------------------------------------------------------- run

def grow(body_centre, radii, scalp, seed):
    """Groom the three families and return their cards as one mesh's data."""
    load_groups()
    rng = random.Random(seed)
    head = Head(body_centre, radii)
    bun_c = bun_centre(head)

    sweep = new_curves("GroomSweep", sweep_guides(head, rng), surface=scalp)
    groom(sweep, [
        ("Interpolate Hair Curves", {"Density": 5200.0, "Interpolation Guides": 3,
                                     "Resting Surface": False,
                                     "Part by Mesh Islands": False, "Seed": seed % 1000,
                                     "Viewport Amount": 1.0}),
        # Locks, not a carpet: children pulled hard into clumps leave gaps
        # between the clumps, and the gaps -- the darker cap showing between
        # locks -- are most of what makes a groom read as hair.
        ("Clump Hair Curves", {"Factor": 0.85, "Shape": 0.35, "Guide Distance": 0.022,
                               "Tip Spread": 0.002, "Clump Offset": 0.002, "Seed": 7}),
        ("Hair Curves Noise", {"Distance": 0.0045, "Scale": 30.0, "Shape": 0.7,
                               "Offset per Curve": 1.0, "Seed": 11}),
    ])
    bun = new_curves("GroomBun", bun_guides(head, rng))
    groom(bun, [
        ("Duplicate Hair Curves", {"Amount": 4, "Radius": 0.006, "Seed": 3}),
        ("Hair Curves Noise", {"Distance": 0.0030, "Scale": 25.0, "Seed": 5}),
        ("Frizz Hair Curves", {"Distance": 0.0012, "Shape": 0.9, "Seed": 9}),
    ])
    loose = new_curves("GroomLoose", loose_guides(head, rng))
    groom(loose, [
        ("Duplicate Hair Curves", {"Amount": 4, "Radius": 0.0045, "Seed": 13}),
        ("Curl Hair Curves", {"Radius": 0.0045, "Curl Start": 0.35, "Frequency": 1.6,
                              "Factor Start": 0.4, "Factor End": 1.0,
                              "Random Offset": 0.5, "Subdivision": 1, "Seed": 23}),
        ("Hair Curves Noise", {"Distance": 0.0030, "Scale": 18.0, "Shape": 0.8,
                               "Seed": 17}),
        ("Frizz Hair Curves", {"Distance": 0.0012, "Shape": 0.9, "Seed": 19}),
    ])

    s_sweep = evaluated_strands(sweep)
    s_bun = evaluated_strands(bun)
    s_loose = evaluated_strands(loose)
    for tag, ss in (("sweep", s_sweep), ("bun", s_bun), ("loose", s_loose)):
        pts = [len(s) for s in ss]
        lens = [sum((s[i + 1] - s[i]).length for i in range(len(s) - 1)) for s in ss]
        roots = [s[0] for s in ss if s]
        lo = Vector((min(p.x for p in roots), min(p.y for p in roots), min(p.z for p in roots)))
        hi = Vector((max(p.x for p in roots), max(p.y for p in roots), max(p.z for p in roots)))
        print(f"  groom {tag}: {len(ss)} strands, points {min(pts)}..{max(pts)}, "
              f"length {min(lens) * 1000:.0f}..{max(lens) * 1000:.0f} mm, "
              f"roots x {lo.x:.3f}..{hi.x:.3f} y {lo.y:.3f}..{hi.y:.3f} z {lo.z:.3f}..{hi.z:.3f}")

    def from_head(_p):
        return head.c

    def from_bun(_p):
        return bun_c

    verts, faces, uvs = [], [], []
    for strands, w0, w1, centre_of, cols in (
            (s_sweep, 0.014, 0.008, from_head, DENSE_COLS),
            (s_bun, 0.016, 0.009, from_bun, DENSE_COLS),
            (s_loose, 0.015, 0.008, from_head, LOOSE_COLS)):
        v, f, u = cards_from(strands, w0, w1, centre_of, cols, rng)
        base = len(verts)
        verts.extend(v)
        uvs.extend(u)
        faces.extend(tuple(base + i for i in q) for q in f)
    for o in (sweep, bun, loose):
        bpy.data.objects.remove(o, do_unlink=True)
    return verts, faces, uvs
