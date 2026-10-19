# SPDX-License-Identifier: GPL-3.0-or-later
"""R-8a, R-8b and R-9: from a Meshy garment-on-clay download to a clean garment in the wearer's frame
(pr-s32p-wardrobe-pilot.md §4.3; step-32 §6.6 g3-g4).

**R-8a, subtraction** (`op: "subtract"`). The download (one fused mesh: the grey clay figure wearing the
garment) is placed on the wearer: scaled so its height equals the wearer's body height, centred over it, its soles
on the body's. The wearer's fitting template T_P (its body faces only) is then fitted onto the download as R-4
step 4 fits it onto a body (normal-agreeing nearest surface within 50 mm), giving the clay's surface. Every
download vertex lying within 1.5 mm outside the fitted clay surface, or inside it, is deleted; then every
connected component whose vertices lie 95 % or more within 4 mm of it is deleted. Kept and removed counts are
printed per region of the nearest template vertex.

**R-9, cleanup** (continued in the same op). Loose fragments under 1 % of the largest piece are dropped; the
shell is decimated to the slot's W-2 budget (minus 5 %); UVs by Smart UV Project (1 % margin); the Meshy base
colour is baked onto the new UVs (Cycles, selected-to-active, cage extrusion 15 mm) at the declared size; one
material `MW_Cast_<id>`. Exported as `garment_wearer.glb` (the garment on the wearer, no skin). Normal and ORM
maps and the tint mask are not baked in the pilot (recorded in the ledger).

**R-8b, tight-fit** (`op: "tight"`). A shell offset 0.8 mm outward from the wearer's own body over the garment's
declared regions (body faces whose three vertices map, through `corr.bin`, to template vertices of those
regions), textured by projecting the garment reference image from the front (and from the back, when a back
view is given) at the v2 camera; baked and exported like R-9. No Meshy run, no MPFB2 asset.

Params (subtract): {"op", "id", "download": glb, "body": glb, "template": bin, "regions": json,
                    "budget": int, "texture_px": int, "out_dir"}
Params (tight):    {"op", "id", "body": glb, "corr": bin, "regions": json, "covers_regions": [..],
                    "front_image": png, "back_image": png | null, "budget": int, "texture_px": int, "out_dir"}
"""

import json
import math
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bmesh  # noqa: E402
import bpy  # noqa: E402
import numpy as np  # noqa: E402 - bundled with Blender
import mpfbcommon as mc  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402
from mathutils.kdtree import KDTree  # noqa: E402

INSIDE_M = 0.0015  # R-8a: a vertex within 1.5 mm outside the clay, or inside it, is clay
CLAY_TOL_MIN = 0.04  # D-impl-14: the clay-colour tolerance's bounds (RGB distance, 0..1 per channel)
CLAY_TOL_MAX = 0.10
DIFFUSE_ROUNDS = 400  # L-a: rounds of neighbour averaging that carry the clay fit under a garment
CHROMA_TOL_ABOVE = 0.20  # L-a: a tolerance bound above this marks a chroma-key mannequin (the recipe's)
CLAY_REFERENCE_REGIONS = ("hands", "forearms", "shins", "feet")
CLAY_NEAR_M = 0.008  # D-impl-14: a clay-coloured vertex is clay only within 8 mm of the fitted clay surface
COVERED_SHARE = 0.5  # D-impl-13: a kept component lies at least half over the garment's covered regions
FIT_RAY_M = 0.05
FRAGMENT_SHARE = 0.01
TIGHT_OFFSET_M = 0.0008  # R-8b: 0.8 mm (Q-S32p-5)
TIGHT_DARK_LUMA = 0.25  # D-impl-15: reference pixels darker than this are the tight garment
BAKE_EXTRUSION_M = 0.015


def import_meshes(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    return [o for o in bpy.data.objects if o not in before]


def joined(objs):
    meshes = [o for o in objs if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    obj.parent = None
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    return obj


def world_bounds(obj, only=None):
    pts = [obj.matrix_world @ v.co for v in obj.data.vertices]
    if only is not None and int(only.sum()) > 1000:
        pts = [q for q, keep in zip(pts, only) if keep]
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts)))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    return lo, hi


def place_on(download, body, only=None):
    """Scale and move the download so its height, horizontal centre and soles match the body's (`only`: a
    per-vertex mask of the download's vertices that bound it, when given and non-trivial)."""
    blo, bhi = world_bounds(body)
    alo, _ahi = world_bounds(download)
    dlo, dhi = world_bounds(download, only)
    # the soles from everything (a shoe's sole is under the clay's), the crown and the horizontal centre from
    # the mask (hair or a hat above the head, a backpack behind it)
    dlo = Vector((dlo.x, dlo.y, alo.z))
    s = (bhi.z - blo.z) / (dhi.z - dlo.z)
    bc = (blo + bhi) / 2
    dc = (dlo + dhi) / 2
    m = (
        Matrix.Translation(Vector((bc.x, bc.y, blo.z)))
        @ Matrix.Scale(s, 4)
        @ Matrix.Translation(-Vector((dc.x, dc.y, dlo.z)))
    )
    download.matrix_world = m @ download.matrix_world
    bpy.ops.object.select_all(action="DESELECT")
    download.select_set(True)
    bpy.context.view_layer.objects.active = download
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    return s


def read_template(path):
    data = Path(path).read_bytes()
    (n,) = struct.unpack_from("<I", data, 4)
    values = struct.unpack_from(f"<{3 * n}f", data, 8)
    return [mc.from_gltf(values[3 * i : 3 * i + 3]) for i in range(n)]


def template_body_faces(template_positions):
    """The hm08 basemesh's body faces (topology from MPFB2), with the given positions."""
    HumanService, TargetService = mc.mpfb()
    basemesh = HumanService.create_human(macro_detail_dict=TargetService.get_default_macro_info_dict())
    body = set(mc.body_group_indices(basemesh))
    faces = [tuple(p.vertices) for p in basemesh.data.polygons if all(v in body for v in p.vertices)]
    normals_mesh = bpy.data.meshes.new("tp")
    normals_mesh.from_pydata([tuple(v) for v in template_positions], [], faces)
    normals_mesh.update()
    bpy.data.objects.remove(basemesh, do_unlink=True)
    return normals_mesh, body


def fit_template_to(tp_mesh, body_ids, target_bvh, diffuse=False):
    """R-4 step 4 against the download: each template body vertex moves to the nearest normal-agreeing surface.
    With `diffuse` (L-a), a vertex with no hit (under the garment, where no clay surface is visible) takes the
    smooth interpolation of its neighbours' displacements, so the template follows the clay under the garment."""
    out = {}
    hit = set()
    for i in body_ids:
        v = tp_mesh.vertices[i]
        p, n = v.co.copy(), v.normal.normalized()
        best = None
        for d in (n, -n):
            loc, normal, _ix, dist = target_bvh.ray_cast(p, d, FIT_RAY_M)
            if loc is not None and normal.dot(n) >= 0.5 and (best is None or dist < best[1]):
                best = (loc, dist)
        out[i] = best[0] if best else p
        if best:
            hit.add(i)
    if diffuse and hit:
        out = diffuse_misses(tp_mesh, body_ids, out, hit)
    for i, p in out.items():
        tp_mesh.vertices[i].co = p
    tp_mesh.update()


def diffuse_misses(tp_mesh, body_ids, out, hit, rounds=DIFFUSE_ROUNDS):
    """Displacements of the hit vertices held fixed; every other body vertex takes the mean displacement of its
    body neighbours, iterated (a discrete harmonic interpolation over the template's own topology)."""
    ids = sorted(body_ids)
    index = {v: k for k, v in enumerate(ids)}
    rest = np.array([tuple(tp_mesh.vertices[i].co) for i in ids])
    disp = np.array([tuple(out[i]) for i in ids]) - rest
    fixed = np.array([i in hit for i in ids])
    pairs = [
        (index[e.vertices[0]], index[e.vertices[1]])
        for e in tp_mesh.edges
        if e.vertices[0] in index and e.vertices[1] in index
    ]
    if not pairs:
        return out
    a = np.array([p[0] for p in pairs])
    b = np.array([p[1] for p in pairs])
    degree = np.zeros(len(ids))
    np.add.at(degree, a, 1.0)
    np.add.at(degree, b, 1.0)
    free = ~fixed & (degree > 0)
    disp[~fixed] = 0.0
    for _ in range(rounds):
        acc = np.zeros_like(disp)
        np.add.at(acc, a, disp[b])
        np.add.at(acc, b, disp[a])
        disp[free] = acc[free] / degree[free][:, None]
    moved = rest + disp
    return {i: Vector(tuple(moved[index[i]])) for i in ids}


def signed_distance(bvh, p):
    loc, normal, _ix, dist = bvh.find_nearest(p)
    if loc is None:
        return math.inf
    return dist if (p - loc).dot(normal) >= 0 else -dist


def base_colour_image(obj):
    for mat in obj.data.materials:
        if mat is None or not mat.use_nodes:
            continue
        for node in mat.node_tree.nodes:
            if node.type == "TEX_IMAGE" and node.image is not None:
                return node.image
    raise SystemExit("the download has no base-colour texture")


def vertex_colours(obj):
    """Per-vertex mean base colour (texture sampled at each loop's UV)."""
    mesh = obj.data
    image = base_colour_image(obj)
    w, h = image.size
    px = np.empty(w * h * 4, dtype=np.float32)
    image.pixels.foreach_get(px)
    px = px.reshape(h, w, 4)
    n_loops = len(mesh.loops)
    vidx = np.empty(n_loops, dtype=np.int64)
    mesh.loops.foreach_get("vertex_index", vidx)
    uv = np.empty(2 * n_loops, dtype=np.float32)
    mesh.uv_layers.active.data.foreach_get("uv", uv)
    uv = uv.reshape(-1, 2) % 1.0
    x = np.clip((uv[:, 0] * w).astype(np.int64), 0, w - 1)
    y = np.clip((uv[:, 1] * h).astype(np.int64), 0, h - 1)
    sums = np.zeros((len(mesh.vertices), 3), dtype=np.float64)
    counts = np.zeros(len(mesh.vertices), dtype=np.float64)
    np.add.at(sums, vidx, px[y, x, :3])
    np.add.at(counts, vidx, 1.0)
    return sums / np.maximum(counts, 1.0)[:, None]


def clay_by_colour(download, tp_mesh, body_ids, regions, covers, tol_max=CLAY_TOL_MAX):
    """D-impl-14: a download vertex is clay when its texture colour is the clay's. The clay colour is the median
    colour of the download's vertices over the limb ends the garment does not cover (hands, forearms, shins,
    feet: there the figure is bare clay, and no hair strand or hem hangs over them); the tolerance is 1.5 × the
    90th-percentile spread of those vertices, within [0.04, 0.10] (RGB distance); the labels are then smoothed
    by two rounds of neighbour majority (the heather and denim textures are noisy). The caller also requires a
    clay vertex to lie on the fitted clay surface (CLAY_NEAR_M)."""
    colours = vertex_colours(download)
    kd = KDTree(len(body_ids))
    for i in body_ids:
        kd.insert(tp_mesh.vertices[i].co, i)
    kd.balance()
    names = regions["regions"]
    at = [names[regions["vertex_region"][kd.find(v.co)[1]]] for v in download.data.vertices]
    outside = np.array([r in CLAY_REFERENCE_REGIONS and r not in covers for r in at])
    if not outside.any():
        outside = np.array([r not in covers for r in at])
    reference = np.median(colours[outside], axis=0) if outside.any() else np.array([0.5, 0.5, 0.5])
    distance = np.linalg.norm(colours - reference, axis=1)
    spread = float(np.percentile(distance[outside], 90)) if outside.any() else 0.05
    tolerance = min(tol_max, max(CLAY_TOL_MIN, 1.5 * spread))
    label = distance < tolerance
    n_edges = len(download.data.edges)
    ev = np.empty(2 * n_edges, dtype=np.int64)
    download.data.edges.foreach_get("vertices", ev)
    ev = ev.reshape(-1, 2)
    for _ in range(2):
        votes = np.zeros(len(label))
        degree = np.zeros(len(label))
        np.add.at(votes, ev[:, 0], label[ev[:, 1]])
        np.add.at(votes, ev[:, 1], label[ev[:, 0]])
        np.add.at(degree, ev[:, 0], 1.0)
        np.add.at(degree, ev[:, 1], 1.0)
        share = (votes + label) / (degree + 1.0)
        label = share > 0.5
    mc.log(
        "clay", json.dumps({"rgb": reference.tolist(), "tolerance": tolerance, "share": float(label.mean())})
    )
    return [bool(b) for b in label], [round(float(c), 4) for c in reference]


def subtract(p, out):
    body = joined(import_meshes(p["body"]))
    download = joined(import_meshes(p["download"]))
    # the glTF import splits vertices at UV seams; weld them (UVs are per loop and survive), so a connected
    # component is a piece of surface, not a UV island
    wb = bmesh.new()
    wb.from_mesh(download.data)
    bmesh.ops.remove_doubles(wb, verts=wb.verts, dist=1e-6)
    wb.to_mesh(download.data)
    wb.free()
    # L-a: with a chroma-key mannequin the figure is placed by its key-coloured surface only, so hair or a hat
    # above the head does not stretch the bounds (the first L-a run sat Vera's updo over her face)
    key = p.get("clay_srgb")
    only = None
    if key is not None and float(p.get("clay_tolerance_max", CLAY_TOL_MAX)) > CHROMA_TOL_ABOVE:
        lin = np.linalg.norm(vertex_colours(download) - np.array(key), axis=1)
        only = lin < float(p["clay_tolerance_max"])
    scale = place_on(download, body, only)
    dmesh = download.data
    dverts = [v.co.copy() for v in dmesh.vertices]
    tp_mesh, body_ids = template_body_faces(read_template(p["template"]))
    regions = json.loads(Path(p["regions"]).read_text(encoding="utf-8"))
    covers = set(p.get("covers_regions") or regions["regions"])
    tol_max = float(p.get("clay_tolerance_max", CLAY_TOL_MAX))
    # L-a: a chroma-key mannequin (a wide tolerance) is far from every garment colour, so its colour alone
    # decides; a grey one also needs the 8 mm geometric condition and the inside test (D-impl-14). Under a
    # garment there are no clay faces, so the template is not fitted there and the inside test would cut a
    # close-fitting garment (Vera's jeans in the first L-a run): a chroma key skips it.
    chroma = tol_max > CHROMA_TOL_ABOVE
    near_m = math.inf if chroma else CLAY_NEAR_M
    is_clay, clay_rgb = clay_by_colour(download, tp_mesh, body_ids, regions, covers, tol_max)
    # The template is fitted onto the clay faces only, so it never snaps onto the garment (D-impl-14)
    clay_faces = [tuple(f.vertices) for f in dmesh.polygons if all(is_clay[i] for i in f.vertices)]
    dbvh = BVHTree.FromPolygons(dverts, clay_faces)
    tp_mesh.update(calc_edges=True)
    fit_template_to(tp_mesh, body_ids, dbvh, diffuse=chroma)
    # The template fitted onto the download's clay is the surface the garment was generated on: R-10 binds the
    # garment against it (not against T_P, whose shape differs from the generated clay), so the binding carries
    # the garment from the clay to any T (D-impl-12).
    t_bin = bytearray(b"MWGT" + struct.pack("<I", len(tp_mesh.vertices)))
    for v in tp_mesh.vertices:
        t_bin += struct.pack("<3f", *mc.to_gltf(v.co))
    (out / "clay_template.bin").write_bytes(bytes(t_bin))
    clay = BVHTree.FromPolygons(
        [v.co for v in tp_mesh.vertices], [tuple(f.vertices) for f in tp_mesh.polygons]
    )
    bm = bmesh.new()
    bm.from_mesh(dmesh)
    bm.verts.ensure_lookup_table()
    # clay: the vertices whose texture colour is the clay's (D-impl-14), plus any vertex inside the fitted clay
    # surface by more than the 1.5 mm tolerance (garment geometry never lies under the skin)
    clay_verts = [
        v
        for v in bm.verts
        if (is_clay[v.index] and signed_distance(clay, v.co) <= near_m)
        or (chroma is False and signed_distance(clay, v.co) < -INSIDE_M)
    ]
    removed_vertex_rule = len(clay_verts)
    bmesh.ops.delete(bm, geom=clay_verts, context="VERTS")
    # every component lying mostly over regions the garment does not cover (clay left at the hands or feet)
    # is dropped: kept only when ≥ 50 % of it is nearest a covered region (D-impl-13). Components are found on
    # the surviving vertices (renumbered after the delete) and deleted together.
    kd = KDTree(len(body_ids))
    for i in body_ids:
        kd.insert(tp_mesh.vertices[i].co, i)
    kd.balance()

    def region_at(co):
        _loc, index, _dist = kd.find(co)
        return regions["regions"][regions["vertex_region"][index]]

    bm.verts.index_update()
    bm.verts.ensure_lookup_table()
    seen = set()
    doomed = []
    removed_components = 0
    for v in bm.verts:
        if v.index in seen:
            continue
        stack, comp = [v], []
        seen.add(v.index)
        while stack:
            x = stack.pop()
            comp.append(x)
            for e in x.link_edges:
                y = e.other_vert(x)
                if y.index not in seen:
                    seen.add(y.index)
                    stack.append(y)
        covered = sum(1 for x in comp if region_at(x.co) in covers)
        if covered < COVERED_SHARE * len(comp):
            doomed.extend(comp)
            removed_components += 1
    if doomed:
        bmesh.ops.delete(bm, geom=doomed, context="VERTS")
    bm.to_mesh(dmesh)
    bm.free()
    report = {
        "scale": scale,
        "download_vertices": len(dverts),
        "removed_clay": removed_vertex_rule,
        "clay_rgb": clay_rgb,
        "clay_share": sum(is_clay) / max(1, len(is_clay)),
        "removed_components": removed_components,
        "kept_vertices": len(dmesh.vertices),
    }
    cleanup(download, p, out, report)


def drop_fragments(obj):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    seen, comps = set(), []
    for v in bm.verts:
        if v.index in seen:
            continue
        stack, comp = [v], []
        seen.add(v.index)
        while stack:
            x = stack.pop()
            comp.append(x)
            for e in x.link_edges:
                y = e.other_vert(x)
                if y.index not in seen:
                    seen.add(y.index)
                    stack.append(y)
        comps.append(comp)
    largest = max((len(c) for c in comps), default=0)
    dropped = 0
    for comp in comps:
        if len(comp) < FRAGMENT_SHARE * largest:
            bmesh.ops.delete(bm, geom=comp, context="VERTS")
            dropped += 1
    bm.to_mesh(obj.data)
    bm.free()
    return dropped


def cleanup(obj, p, out, report):
    """R-9: fragments, decimation to budget, UVs, base-colour bake, one material, export (no skin)."""
    source = obj.copy()
    source.data = obj.data.copy()
    bpy.context.scene.collection.objects.link(source)
    report["fragments_dropped"] = drop_fragments(obj)
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    tris = sum(len(f.vertices) - 2 for f in obj.data.polygons)
    target = int(p["budget"] * 0.95)
    if tris > target:
        mod = obj.modifiers.new("decimate", "DECIMATE")
        mod.ratio = target / tris
        bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.quads_convert_to_tris()
    for name in [layer.name for layer in obj.data.uv_layers if not layer.name.startswith(".")]:
        layer = obj.data.uv_layers.get(name)
        if layer is not None:
            obj.data.uv_layers.remove(layer)
    obj.data.uv_layers.new(name="UVMap")
    bpy.ops.uv.smart_project(island_margin=0.01)
    bpy.ops.object.mode_set(mode="OBJECT")
    size = int(p["texture_px"])
    image = bpy.data.images.new(f"{p['id']}_albedo", size, size)
    mat = bpy.data.materials.new(f"MW_Cast_{p['id']}")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = image
    bsdf = nodes.get("Principled BSDF")
    mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    obj.data.materials.clear()
    obj.data.materials.append(mat)
    nodes.active = tex
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 1
    scene.render.bake.use_selected_to_active = True
    scene.render.bake.cage_extrusion = BAKE_EXTRUSION_M
    scene.render.bake.margin = 8
    bpy.ops.object.select_all(action="DESELECT")
    source.select_set(True)
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"COLOR"})
    image.filepath_raw = str(out / "albedo.png")
    image.file_format = "PNG"
    image.save()
    image.pack()
    bpy.data.objects.remove(source, do_unlink=True)
    report["triangles"] = sum(len(f.vertices) - 2 for f in obj.data.polygons)
    report["texture_px"] = size
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    obj.name = f"part_{p['id']}"
    bpy.ops.export_scene.gltf(
        filepath=str(out / "garment_wearer.glb"),
        export_format="GLB",
        use_selection=True,
        export_skins=False,
        export_animations=False,
        export_tangents=True,
    )
    mc.write_result(out / "clean.json", report)
    mc.log("clean", p["id"], json.dumps(report))


def tight(p, out):
    body = joined(import_meshes(p["body"]))
    data = Path(p["corr"]).read_bytes()
    (n,) = struct.unpack_from("<I", data, 4)
    corr = [struct.unpack_from("<3I", data, 8 + 20 * k) for k in range(n)]
    regions = json.loads(Path(p["regions"]).read_text(encoding="utf-8"))
    wanted = {regions["regions"].index(r) for r in p["covers_regions"]}
    region_of = regions["vertex_region"]
    covered = [any(region_of[t] in wanted for t in corr[i]) for i in range(len(body.data.vertices))]
    bm = bmesh.new()
    bm.from_mesh(body.data)
    drop = [f for f in bm.faces if not all(covered[v.index] for v in f.verts)]
    bmesh.ops.delete(bm, geom=drop, context="FACES")
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bm.normal_update()
    for v in bm.verts:
        v.co += v.normal * TIGHT_OFFSET_M
    bm.to_mesh(body.data)
    bm.free()
    # The garment's colour from the reference image (D-impl-15): the median of the pixels darker than the clay
    # figure (the clay renders at ≈ 0.4; opaque tights are the darkest surface in the image). A front
    # projection needs the reference camera's framing, which the pilot does not record; the shell is uniform.
    bpy.ops.object.select_all(action="DESELECT")
    body.select_set(True)
    bpy.context.view_layer.objects.active = body
    image = bpy.data.images.load(p["front_image"])
    w, h = image.size
    px = np.empty(w * h * 4, dtype=np.float32)
    image.pixels.foreach_get(px)
    rgb = px.reshape(-1, 4)[:, :3]
    dark = rgb[rgb.mean(axis=1) < TIGHT_DARK_LUMA]
    colour = np.median(dark, axis=0) if len(dark) else np.array([0.12, 0.12, 0.13])
    colour = ((colour + 0.055) / 1.055) ** 2.4  # sRGB pixel values → the linear socket value
    mat = bpy.data.materials.new("projection")
    mat.use_nodes = True
    mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*colour.tolist(), 1.0)
    body.data.materials.clear()
    body.data.materials.append(mat)
    report = {
        "vertices": len(body.data.vertices),
        "offset_m": TIGHT_OFFSET_M,
        "covers": p["covers_regions"],
        "colour": [round(float(c), 4) for c in colour],
        "note": "uniform colour from the reference image (D-impl-15)",
    }
    cleanup(body, p, out, report)


def main():
    p = mc.params()
    out = Path(p["out_dir"])
    out.mkdir(parents=True, exist_ok=True)
    mc.clear_scene()
    if p["op"] == "subtract":
        subtract(p, out)
    elif p["op"] == "tight":
        tight(p, out)
    else:
        raise SystemExit("unknown op " + str(p["op"]))


main()
