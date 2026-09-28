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
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.update()
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

def inflate(obj, dist: float, smooth_first: int = 0):
    """Push every vertex out along its own normal.

    `smooth_first` averages the *normals* before offsetting rather than the
    positions afterwards.  Offsetting along raw per-vertex normals is what made
    an earlier hoodie read as a quilted puffer: the surface underneath carries
    wrinkle detail, and a per-vertex push amplifies every fold into a panel.
    """
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.normal_update()
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
        v.co += nrm[v.index] * dist
    bm.to_mesh(obj.data)
    bm.free()


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

def boundary_loop(bm, pick):
    """Boundary vertices of `bm` for which `pick(vertex.co)` is true."""
    return [v for v in bm.verts
            if any(e.is_boundary for e in v.link_edges) and pick(v.co)]


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
        # stitch: for every boundary edge of the old ring, make a quad
        made = set()
        for v_old in ring:
            for e in v_old.link_edges:
                o = e.other_vert(v_old)
                if o not in pair or (o, v_old) in made or not e.is_boundary:
                    continue
                made.add((v_old, o))
                try:
                    bm.faces.new((v_old, o, pair[o], pair[v_old]))
                except ValueError:
                    pass
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
    made = set()
    for a in ring:
        for e in a.link_edges:
            b = e.other_vert(a)
            if b not in pair or (b, a) in made or not e.is_boundary:
                continue
            made.add((a, b))
            try:
                f = bm.faces.new((a, b, pair[b], pair[a]))
                if material_index is not None:
                    f.material_index = material_index
            except ValueError:
                pass
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
