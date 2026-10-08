# SPDX-License-Identifier: GPL-2.0-or-later
"""Blender/bmesh helpers shared by the garment and hair modellers.

Imported by `garments.py`, `hair.py` and `character_model.py`, both of which run inside
Blender (`blender --background --python ...`).  Nothing here touches CharMorph:
the input is the CC0 body GLB, and Blender is used purely as a modelling
library.

Coordinate frame, stated once because every offset below depends on it.  The
body arrives as a glTF exported Y-up and Blender converts it back to its own
Z-up on import, so inside these tools:

    +Z  up          +X  the character's own left       -Y  the way she faces

All distances are metres on a figure whose crown is at z = 1.747.
"""

from __future__ import annotations

import math

import bmesh  # pylint: disable=import-error
import bpy  # pylint: disable=import-error
from mathutils import Vector  # pylint: disable=import-error


# --------------------------------------------------------------------- loading

def load_body(path: str):
    """Import the baked body GLB and return (mesh object, armature object)."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    mesh = next(o for o in bpy.data.objects if o.type == "MESH")
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    # The importer parents the mesh to the armature; garments are built in the
    # same space, so every object keeps an identity world matrix.
    for o in (mesh, arm):
        o.matrix_world.identity()
    return mesh, arm


def bone_head(arm, name: str) -> Vector:
    return (arm.matrix_world @ arm.data.bones["mixamorig:" + name].head_local).copy()


# ------------------------------------------------------------------- selection

def dup_region(body, keep, name: str):
    """A new object holding the body faces for which `keep(face)` is true.

    Vertex groups come with the duplicate, which is the whole reason garments
    are cut out of the body rather than modelled free-standing: a surface that
    was skinned correctly stays skinned correctly, and no weight painting or
    transfer is needed for the part of the garment that hugs the figure.

    `keep` receives the bmesh face, whose vertex indices still match the body's
    because the delete has not happened yet.  That matters: a predicate written
    against the body's own indices and evaluated *after* a delete selects the
    wrong faces entirely, which is how a first attempt produced jeans full of
    holes.
    """
    obj = body.copy()
    obj.data = body.data.copy()
    obj.name = obj.data.name = name
    obj.modifiers.clear()          # the copy inherits the body's armature modifier
    bpy.context.scene.collection.objects.link(obj)

    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.faces.ensure_lookup_table()
    bm.verts.ensure_lookup_table()
    doomed = [f for f in bm.faces if not keep(f)]
    bmesh.ops.delete(bm, geom=doomed, context="FACES")
    bm.verts.ensure_lookup_table()
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context="VERTS")
    # Weld the UV seams before anything moves a vertex.  The body arrives from
    # glTF, which splits a vertex wherever its UV or normal differs, so every
    # texture seam is two coincident, *disconnected* edges -- 290 seam pairs in
    # the hoodie's cut, 317 in the jeans'.  Coincident, they are invisible; but
    # `relax` pulls each side toward its own one-sided neighbours and `inflate`
    # pushes each side along a normal computed from that side alone, and the
    # seams opened by up to 65 mm: the torn shoulders, the spiked sleeve and
    # shoulder seams, the serrated hem, the split thigh and the neckline.  That
    # was the tearing blamed on the offset's radius of curvature.  UVs are per
    # face corner, so welding loses none of them, and every garment is re-UV'd.
    n0 = len(bm.verts)
    bmesh.ops.remove_doubles(bm, verts=list(bm.verts), dist=1e-6)
    welded = n0 - len(bm.verts)
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.update()
    print(f"    cut {name}: {len(obj.data.vertices)} verts, {welded} seam verts welded")
    return obj


def dominant_group(body):
    """vertex index -> name of the vertex group with the largest weight."""
    names = [g.name for g in body.vertex_groups]
    out = {}
    for v in body.data.vertices:
        best, bw = None, -1.0
        for g in v.groups:
            if g.weight > bw:
                best, bw = g.group, g.weight
        out[v.index] = names[best] if best is not None else ""
    return out


# -------------------------------------------------------------------- reshaping

def _untangle(obj, origin, report: bool, rings: int = 2, passes: int = 40) -> int:
    """Smooth out the folds an offset left in concave creases, and only there.

    Where the surface is concave -- the crotch between the thighs, the armpit,
    the cleft of the seat -- neighbours' normals converge, and offsetting them
    any distance at all makes them cross.  Cloth does not follow skin into a
    crease; it bridges it.  So the faces that came out inverted, plus `rings`
    of neighbours, are relaxed until none is inverted.  Smoothing the whole
    shell instead (`relax_between`) was measured: it made the armpit worse and
    took 10 mm off the lift everywhere.

    Boundary vertices are held, so an opening does not move.  Returns the
    inverted-face count that remains.
    """
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.faces.ensure_lookup_table()
    start = left = None
    for _ in range(passes):
        bm.normal_update()
        bad = [f for f in bm.faces if f.normal.dot(origin[f.index]) < 0.0]
        left = len(bad)
        if start is None:
            start = left
        if not bad:
            break
        region = {v for f in bad for v in f.verts}
        for _ in range(rings):
            region |= {e.other_vert(v) for v in region for e in v.link_edges}
        region = [v for v in region if not any(e.is_boundary for e in v.link_edges)]
        bmesh.ops.smooth_vert(bm, verts=region, factor=0.5,
                              use_axis_x=True, use_axis_y=True, use_axis_z=True)
    bm.normal_update()
    left = sum(1 for f in bm.faces if f.normal.dot(origin[f.index]) < 0.0)
    bm.to_mesh(obj.data)
    bm.free()
    if report:
        print(f"    untangle {obj.name}: {start} inverted faces -> {left}")
    return left


def inflate(obj, dist: float, smooth_first: int = 0, steps: int = 1,
            relax_between: float = 0.0, report: bool = True,
            untangle: bool = True):
    """Push every vertex out along its own normal, and say if that tore it.

    `smooth_first` averages the *normals* before offsetting rather than the
    positions afterwards.  Offsetting along raw per-vertex normals is what made
    an earlier hoodie read as a quilted puffer: the surface underneath carries
    wrinkle detail, and a per-vertex push amplifies every fold into a panel.

    **`steps` exists because a normal offset is only valid while the distance
    stays under the surface's local radius of curvature.** Past that, neighbours
    travelling along converging normals cross over and the surface folds through
    itself; the faces in the fold come out inverted, and after decimation they
    render as angular holes with the garment's backfaces showing through. A
    shoulder, an armpit and the side of a neck all have radii well under the
    46 mm this was once asked for in one jump. Offsetting in several smaller
    steps, re-deriving the normals each time, follows the surface instead of
    shooting past it -- the same reason a real offset is computed iteratively.

    The flipped-face count is reported rather than left to be discovered in a
    screenshot, because that is how it was discovered the first time.
    """
    # Measured against the *original* orientation, not against the previous
    # step: a face that inverted early and stayed inverted does not flip again,
    # so a per-step count reports zero for a surface that is already inside out.
    origin = None
    flipped = 0
    for _ in range(max(1, steps)):
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        bm.normal_update()
        if origin is None:
            origin = {f.index: f.normal.copy() for f in bm.faces}
        before = origin
        nrm = {v.index: v.normal.copy() for v in bm.verts}
        for _ in range(smooth_first):
            nxt = {}
            for v in bm.verts:
                acc = nrm[v.index].copy()
                for e in v.link_edges:
                    acc += nrm[e.other_vert(v).index]
                nxt[v.index] = acc.normalized()
            nrm = nxt
        for v in bm.verts:
            v.co += nrm[v.index] * (dist / max(1, steps))
        # Let converging normals relax instead of crossing. Smoothing *between*
        # increments is what stops a fold forming at all; smoothing afterwards
        # only averages a fold that already exists.
        if relax_between > 0.0:
            bmesh.ops.smooth_vert(bm, verts=bm.verts, factor=relax_between,
                                  use_axis_x=True, use_axis_y=True,
                                  use_axis_z=True)
        bm.normal_update()
        flipped = sum(1 for f in bm.faces if f.normal.dot(before[f.index]) < 0.0)
        bm.to_mesh(obj.data)
        bm.free()
    if untangle and flipped:
        flipped = _untangle(obj, origin, report)
    if report:
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        bm.normal_update()
        bad = [f.calc_center_median() for f in bm.faces
               if origin and f.normal.dot(origin[f.index]) < 0.0]
        where = ""
        if bad:
            where = ("  z %.3f..%.3f  |x| %.3f..%.3f" % (
                min(c.z for c in bad), max(c.z for c in bad),
                min(abs(c.x) for c in bad), max(abs(c.x) for c in bad)))
        bm.free()
        print(f"    inflate {obj.name}: {dist * 1000:.0f} mm in {max(1, steps)} "
              f"step(s), {flipped} inverted faces{where}")
    return flipped


def keep_outside(obj, body, gap: float, pick=None) -> int:
    """Push any vertex closer than `gap` to the skin back out to `gap`.

    Measured against the body's nearest surface point and normal, so it holds
    after smoothing and decimation have moved things -- both of which pull a
    convex garment inward, and both of which sank the tee's collar into the
    neck.  Returns how many vertices moved.
    """
    from mathutils.bvhtree import BVHTree  # pylint: disable=import-error,import-outside-toplevel
    src = bmesh.new()
    src.from_mesh(body.data)
    src.normal_update()
    tree = BVHTree.FromBMesh(src)
    src.free()
    n = 0
    for v in obj.data.vertices:
        if pick is not None and not pick(v.co):
            continue
        hit, nrm, _i, _d = tree.find_nearest(v.co)
        if hit is None:
            continue
        depth = (v.co - hit).dot(nrm)
        if depth < gap:
            v.co = v.co + nrm * (gap - depth)
            n += 1
    obj.data.update()
    return n


def relax(obj, iterations: int = 4, factor: float = 0.6):
    """Laplacian smoothing, the cloth-versus-quilt fix stated in `inflate`."""
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    for _ in range(iterations):
        bmesh.ops.smooth_vert(bm, verts=bm.verts, factor=factor,
                              use_axis_x=True, use_axis_y=True, use_axis_z=True)
    bm.to_mesh(obj.data)
    bm.free()


def apply_modifier(obj, mod):
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=mod.name)


def solidify(obj, thickness: float, offset: float = -1.0):
    m = obj.modifiers.new("solid", "SOLIDIFY")
    m.thickness = thickness
    m.offset = offset
    m.use_rim = True
    apply_modifier(obj, m)


def decimate(obj, ratio: float):
    if ratio >= 1.0:
        return
    m = obj.modifiers.new("dec", "DECIMATE")
    m.ratio = ratio
    apply_modifier(obj, m)


def shade_smooth(obj):
    for p in obj.data.polygons:
        p.use_smooth = True
    obj.data.update()


def flare(obj, z_lo: float, z_hi: float, amount: float, axis_y: float = 0.03):
    """Widen the garment toward `z_lo`, so it hangs off the body instead of on it.

    A shell offset by a constant distance is a wetsuit.  Cotton fleece falls
    away from the waist, and this is the cheapest thing that stops the hoodie
    reading as a fitted track top.
    """
    for v in obj.data.vertices:
        t = 1.0 - min(1.0, max(0.0, (v.co.z - z_lo) / (z_hi - z_lo)))
        s = 1.0 + amount * t * t
        v.co.x *= s
        v.co.y = axis_y + (v.co.y - axis_y) * s


# ------------------------------------------------------------------------ edges

def report_boundaries(bm, label: str, expect: int = 0):
    """Enumerate the open edge loops of a mesh, with where each one is.

    `ARC-23`: a hole *is* a boundary loop, so listing the loops names the cut
    that made it, instead of inferring a cause from a symptom.  A garment cut
    from the body should have one loop per intended opening — hem, neck, two
    cuffs, the front — and any loop besides those is a hole someone has to
    explain.

    Location is printed with the count on purpose. The count alone cannot
    distinguish a hem from a tear.
    """
    # Tracked by the edge itself, not `e.index`: edges an extrusion has just
    # created carry stale indices until `index_update()`, and an index-keyed
    # seen-set merged the +X cuff's ring into another loop and reported it as
    # a two-vertex fragment.
    seen = set()
    loops = []
    for e0 in bm.edges:
        if not e0.is_boundary or e0 in seen:
            continue
        stack = [e0]
        verts, edges = set(), []
        while stack:
            e = stack.pop()
            if e in seen:
                continue
            seen.add(e)
            edges.append(e)
            for v in e.verts:
                verts.add(v)
                for e2 in v.link_edges:
                    if e2.is_boundary and e2 not in seen:
                        stack.append(e2)
        co = [v.co for v in verts]
        perim = sum((e.verts[0].co - e.verts[1].co).length for e in edges)
        loops.append((len(verts), perim, co))
    loops.sort(key=lambda t: -t[1])
    print(f"    {label}: {len(loops)} open loop(s)"
          + (f"  [expected {expect}]" if expect else ""))
    for n, perim, co in loops[:12]:
        print(f"      {n:>4} verts  perimeter {perim * 1000:6.0f} mm  "
              f"x {min(c.x for c in co):+.3f}..{max(c.x for c in co):+.3f}  "
              f"y {min(c.y for c in co):+.3f}..{max(c.y for c in co):+.3f}  "
              f"z {min(c.z for c in co):.3f}..{max(c.z for c in co):.3f}")
    if len(loops) > 12:
        print(f"      ... and {len(loops) - 12} more")
    return loops


def report_winding(bm, label: str):
    """Count, and locate, edges whose two faces disagree about winding.

    Two faces sharing an edge must traverse it in opposite directions.  Where
    they do not, one of them is a backface to the renderer and disappears;
    this is the measurement that names a striped cuff, before a screenshot
    does.
    """
    bad, fan = [], []
    for e in bm.edges:
        mid = (e.verts[0].co + e.verts[1].co) * 0.5
        if len(e.link_loops) > 2:
            fan.append(mid)
        if len(e.link_loops) != 2:
            continue
        l0, l1 = e.link_loops
        if l0.vert is l1.vert:
            bad.append(mid)

    def where(pts):
        if not pts:
            return ""
        return "  x %+.3f..%+.3f  z %.3f..%.3f" % (
            min(c.x for c in pts), max(c.x for c in pts),
            min(c.z for c in pts), max(c.z for c in pts))
    print(f"    {label}: {len(bad)} edges with inconsistent winding{where(bad)}; "
          f"{len(fan)} edges with more than two faces{where(fan)}")
    return len(bad) + len(fan)


def boundary_loop(bm, pick):
    """Boundary vertices of `bm` for which `pick(vertex.co)` is true."""
    return [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and pick(v.co)]


def stitch(bm, ring, pair, material_index=None):
    """Quads between a boundary ring and its extruded copy, wound consistently.

    Each new quad continues the face already on its boundary edge, so it must
    run that edge the *opposite* way.  Winding a quad by whichever endpoint a
    loop happened to visit first -- what the three extruders here each used to
    do -- flips every other quad, and a renderer that culls backfaces drops
    them: the cuffs came out as separate strips with the wrist showing through
    the slits, the hem serrated and the zip tape a sawtooth.

    The boundary edges are collected before any face is added, because adding
    one makes its edge stop being a boundary.
    """
    edges = []
    for a in ring:
        for e in a.link_edges:
            b = e.other_vert(a)
            if b in pair and e.is_boundary and e not in edges:
                edges.append(e)
    made = []
    for e in edges:
        a, b = e.verts
        lp = e.link_loops[0] if e.link_loops else None
        # the existing face runs lp.vert -> other; the new one runs back
        if lp is not None and lp.vert is a:
            quad = (b, a, pair[a], pair[b])
        else:
            quad = (a, b, pair[b], pair[a])
        try:
            f = bm.faces.new(quad)
        except ValueError:
            continue
        if material_index is not None:
            f.material_index = material_index
        made.append(f)
    return made


def extrude_band(bm, verts, steps, centre=None):
    """Extrude a boundary ring in `steps`, each a (offset vector, radial scale).

    Used for the hoodie's cuffs and hem: a knitted band is a short cylinder
    gathered tighter than the sleeve above it, and `rib_displace` then cuts the
    vertical grooves that make it read as rib rather than as more fleece.
    """
    ring = list(verts)
    if not ring:
        return
    if centre is None:
        cx = sum(v.co.x for v in ring) / len(ring)
        cy = sum(v.co.y for v in ring) / len(ring)
        centre = Vector((cx, cy, 0.0))
    for shift, scale in steps:
        res = bmesh.ops.extrude_vert_indiv(bm, verts=ring)
        # extrude_vert_indiv gives loose verts; build the quad strip by hand
        new = res["verts"]
        pair = dict(zip(ring, new))
        for v_old, v_new in pair.items():
            rad = Vector((v_old.co.x - centre.x, v_old.co.y - centre.y, 0.0))
            v_new.co = v_old.co + shift + rad * (scale - 1.0)
        stitch(bm, ring, pair)
        ring = [pair[v] for v in ring]
    bm.normal_update()
    return ring


def smooth_boundary(bm, pick, iterations: int = 6, factor: float = 0.6):
    """Straighten an open edge by averaging each boundary vertex along the loop.

    Decimation does not respect a cut line: a clean strip removed from the dense
    shell comes back out of the decimator with a 5 mm sawtooth, and on the front
    panels of an open jacket that reads as a torn garment rather than a seam.
    Smoothing along the loop only — never across it — fixes the edge without
    moving the surface behind it.
    """
    ring = [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and pick(v.co)]
    idx = set(v.index for v in ring)
    nbr = {}
    for v in ring:
        nbr[v.index] = [e.other_vert(v) for e in v.link_edges
                        if e.is_boundary and e.other_vert(v).index in idx]
    for _ in range(iterations):
        moved = {}
        for v in ring:
            ns = nbr[v.index]
            if len(ns) != 2:
                continue
            mid = (ns[0].co + ns[1].co) * 0.5
            moved[v] = v.co + (mid - v.co) * factor
        for v, co in moved.items():
            v.co = co
    bm.normal_update()


def extrude_strip(bm, ring, disp, material_index=None):
    """Extrude a boundary ring by a per-vertex displacement and return the ring.

    `extrude_band` sweeps a ring radially, which is right for a cuff and wrong
    for a straight seam.  This one takes `disp(co) -> Vector`, so a flat tape can
    be laid along the edge of an opening, and it tags the new faces with a
    material index directly rather than leaving them to be picked out afterwards
    by position — painting a band by face centre on a decimated shell produces a
    row of whole triangles, which reads as a torn edge, not as a zip.
    """
    ring = list(ring)
    if not ring:
        return ring
    res = bmesh.ops.extrude_vert_indiv(bm, verts=ring)
    pair = dict(zip(ring, res["verts"]))
    for old, new in pair.items():
        new.co = old.co + disp(old.co)
    stitch(bm, ring, pair, material_index)
    bm.normal_update()
    return [pair[v] for v in ring]


def snap_opening(bm, pick, target):
    """Pull the boundary vertices of a cut onto the cut line exactly.

    Smoothing along the loop helps but cannot fix a boundary the decimator has
    left with T-junctions, and one of the two front panel edges kept its
    sawtooth while the other came out clean.  The cut is a known surface — a
    vertical line at a known x — so the honest fix is to put the vertices back
    on it rather than to average them and hope.
    """
    n = 0
    for v in bm.verts:
        if not any(e.is_boundary for e in v.link_edges) or not pick(v.co):
            continue
        v.co.x = math.copysign(target(v.co.z), v.co.x)
        n += 1
    bm.normal_update()
    return n


def rib_displace(bm, pick, ribs: int, depth: float, centre_xy=(0.0, 0.0)):
    """cos(n·θ) radial displacement, the knit groove of a cuff or hem."""
    for v in bm.verts:
        if not pick(v.co):
            continue
        dx, dy = v.co.x - centre_xy[0], v.co.y - centre_xy[1]
        r = math.hypot(dx, dy)
        if r < 1e-6:
            continue
        ang = math.atan2(dy, dx)
        k = math.cos(ang * ribs) * depth
        v.co.x += dx / r * k
        v.co.y += dy / r * k


# ------------------------------------------------------------------- new meshes

def loft(rings, cap_first=False, cap_last=True):
    """(verts, faces) for a tube through a list of equal-length rings."""
    verts, faces = [], []
    n = len(rings[0])
    for ring in rings:
        verts.extend(ring)
    for r in range(len(rings) - 1):
        for k in range(n):
            k2 = (k + 1) % n
            a, b = r * n + k, r * n + k2
            c, d = (r + 1) * n + k, (r + 1) * n + k2
            faces.append((a, b, d, c))
    for do_cap, r, flip in ((cap_first, 0, True), (cap_last, len(rings) - 1, False)):
        if not do_cap:
            continue
        cx = sum(verts[r * n + k][0] for k in range(n)) / n
        cy = sum(verts[r * n + k][1] for k in range(n)) / n
        cz = sum(verts[r * n + k][2] for k in range(n)) / n
        c = len(verts)
        verts.append((cx, cy, cz))
        for k in range(n):
            k2 = (k + 1) % n
            a, b = r * n + k, r * n + k2
            faces.append((c, a, b) if flip else (c, b, a))
    return verts, faces


def tube(path, radii, segments=8, flatten=1.0, cap=True):
    """A swept tube along `path` (list of Vectors) with per-point radius.

    `flatten` squashes the cross-section in the horizontal axis, which is what
    makes a hoodie drawstring read as a flat cord rather than a wire.
    """
    rings = []
    for i, p in enumerate(path):
        if i == 0:
            t = (path[1] - path[0]).normalized()
        elif i == len(path) - 1:
            t = (path[-1] - path[-2]).normalized()
        else:
            t = (path[i + 1] - path[i - 1]).normalized()
        up = Vector((0, 0, 1))
        if abs(t.dot(up)) > 0.95:
            up = Vector((0, 1, 0))
        a = t.cross(up).normalized()
        b = t.cross(a).normalized()
        r = radii[i]
        rings.append([tuple(p + a * (math.cos(th) * r) + b * (math.sin(th) * r * flatten))
                      for th in [math.tau * k / segments for k in range(segments)]])
    return loft(rings, cap_first=cap, cap_last=cap)


def new_object(name: str, verts, faces, material: str):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v) for v in verts], [], [tuple(f) for f in faces])
    me.validate()
    me.update()
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    set_material(obj, material)
    return obj


# --------------------------------------------------------------------- finishing

def set_material(obj, name: str):
    obj.data.materials.clear()
    obj.data.materials.append(bpy.data.materials.get(name) or bpy.data.materials.new(name))
    for p in obj.data.polygons:
        p.material_index = 0


def assign_material(obj, name: str, pick):
    """Add `name` as a second slot and move the faces `pick` selects into it."""
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    if mat.name not in [m.name for m in obj.data.materials]:
        obj.data.materials.append(mat)
    slot = [m.name for m in obj.data.materials].index(mat.name)
    n = 0
    for p in obj.data.polygons:
        if pick(p.center, p.normal):
            p.material_index = slot
            n += 1
    return n


def transfer_weights(obj, body):
    """Re-derive skin weights from the nearest body surface.

    Geometry cut out of the body already carries its weights; geometry we
    created — a hood, a drawstring, a ribbed band extruded past the hem — does
    not.  One nearest-surface transfer over the whole garment covers both, and
    it is correct for the created parts because a hood's nearest body surface is
    the back of the head and a drawstring's is the chest.
    """
    for g in body.vertex_groups:
        if g.name not in obj.vertex_groups:
            obj.vertex_groups.new(name=g.name)
    m = obj.modifiers.new("wxfer", "DATA_TRANSFER")
    m.object = body
    m.use_vert_data = True
    m.data_types_verts = {"VGROUP_WEIGHTS"}
    m.vert_mapping = "POLYINTERP_NEAREST"
    m.layers_vgroup_select_src = "ALL"
    m.layers_vgroup_select_dst = "NAME"
    apply_modifier(obj, m)


def bind(obj, arm):
    obj.parent = arm
    m = obj.modifiers.new("skin", "ARMATURE")
    m.object = arm
    m.use_vertex_groups = True


def cylindrical_uv(obj, z_lo: float, z_hi: float, u_turns: float = 1.0,
                   u_centre: float = 0.5):
    """Wrap a UV around +Z with u = 0.5 dead ahead of the character.

    The tee graphic has to land on the chest and nowhere else, so the front of
    the garment is pinned to the middle of the texture and the back wraps to its
    edges.  `-Y` is forward, hence the atan2 argument order.
    """
    me = obj.data
    lay = me.uv_layers.get("UVMap") or me.uv_layers.new(name="UVMap")
    for poly in me.polygons:
        for li, vi in zip(poly.loop_indices, poly.vertices):
            co = me.vertices[vi].co
            ang = math.atan2(co.x, -co.y)          # 0 straight ahead
            u = u_centre + ang / math.tau * u_turns
            v = (co.z - z_lo) / (z_hi - z_lo)
            lay.data[li].uv = (u, v)


def planar_uv(obj, scale: float):
    """A metres-to-UV planar projection for a tiling material (denim, canvas)."""
    me = obj.data
    lay = me.uv_layers.get("UVMap") or me.uv_layers.new(name="UVMap")
    for poly in me.polygons:
        n = poly.normal
        for li, vi in zip(poly.loop_indices, poly.vertices):
            co = me.vertices[vi].co
            if abs(n.z) > max(abs(n.x), abs(n.y)):
                u, v = co.x, co.y
            elif abs(n.x) > abs(n.y):
                u, v = co.y, co.z
            else:
                u, v = co.x, co.z
            lay.data[li].uv = (u / scale, v / scale)


def report(objs):
    print(f"{'object':<16}{'verts':>9}{'tris':>9}  materials")
    tv = tt = 0
    for o in objs:
        nv = len(o.data.vertices)
        nt = sum(len(p.vertices) - 2 for p in o.data.polygons)
        tv += nv
        tt += nt
        print(f"{o.name:<16}{nv:>9}{nt:>9}  {[m.name for m in o.data.materials]}")
    print(f"{'TOTAL':<16}{tv:>9}{tt:>9}")


def rounded_box(centre, size, radius: float, segs: int = 5, slices: int = 6):
    """(verts, faces) for a box with rounded vertical edges and a domed top/bottom.

    A rucksack is a soft bag, not a crate: the corners are what stop it reading
    as a cardboard box strapped to someone's back.
    """
    cx, cy, cz = centre
    hx, hy, hz = size[0] / 2, size[1] / 2, size[2] / 2
    r = min(radius, hx * 0.9, hy * 0.9)
    ring_xy = []
    corners = [(hx - r, hy - r), (-(hx - r), hy - r),
               (-(hx - r), -(hy - r)), (hx - r, -(hy - r))]
    for k, (qx, qy) in enumerate(corners):
        a0 = math.tau * k / 4
        for j in range(segs):
            a = a0 + math.tau / 4 * j / segs
            ring_xy.append((qx + math.cos(a) * r, qy + math.sin(a) * r))
    rings = []
    for s in range(slices + 1):
        t = s / slices
        z = cz - hz + 2 * hz * t
        # dome the ends so the top and bottom are not flat lids
        k = math.sin(math.pi * min(1.0, max(0.0, t))) ** 0.35
        k = max(k, 0.25) if 0.0 < t < 1.0 else 0.62
        rings.append([(cx + x * k, cy + y * k, z) for x, y in ring_xy])
    return loft(rings, cap_first=True, cap_last=True)
