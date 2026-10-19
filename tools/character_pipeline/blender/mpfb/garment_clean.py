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
import mpfbcommon as mc  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402

INSIDE_M = 0.0015  # R-8a: a vertex within 1.5 mm outside the clay, or inside it, is clay
COMPONENT_NEAR_M = 0.004  # R-8a: a component 95 % within 4 mm of the clay is clay
COMPONENT_SHARE = 0.95
FIT_RAY_M = 0.05
FRAGMENT_SHARE = 0.01
TIGHT_OFFSET_M = 0.0008  # R-8b: 0.8 mm (Q-S32p-5)
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


def world_bounds(obj):
    pts = [obj.matrix_world @ v.co for v in obj.data.vertices]
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts)))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    return lo, hi


def place_on(download, body):
    """Scale and move the download so its height, horizontal centre and soles match the body's."""
    blo, bhi = world_bounds(body)
    dlo, dhi = world_bounds(download)
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


def fit_template_to(tp_mesh, body_ids, target_bvh):
    """R-4 step 4 against the download: each template body vertex moves to the nearest normal-agreeing surface."""
    out = {}
    for i in body_ids:
        v = tp_mesh.vertices[i]
        p, n = v.co.copy(), v.normal.normalized()
        best = None
        for d in (n, -n):
            loc, normal, _ix, dist = target_bvh.ray_cast(p, d, FIT_RAY_M)
            if loc is not None and normal.dot(n) >= 0.5 and (best is None or dist < best[1]):
                best = (loc, dist)
        out[i] = best[0] if best else p
    for i, p in out.items():
        tp_mesh.vertices[i].co = p
    tp_mesh.update()


def signed_distance(bvh, p):
    loc, normal, _ix, dist = bvh.find_nearest(p)
    if loc is None:
        return math.inf
    return dist if (p - loc).dot(normal) >= 0 else -dist


def subtract(p, out):
    body = joined(import_meshes(p["body"]))
    download = joined(import_meshes(p["download"]))
    scale = place_on(download, body)
    dmesh = download.data
    dverts = [v.co.copy() for v in dmesh.vertices]
    dbvh = BVHTree.FromPolygons(dverts, [tuple(f.vertices) for f in dmesh.polygons])
    tp_mesh, body_ids = template_body_faces(read_template(p["template"]))
    fit_template_to(tp_mesh, body_ids, dbvh)
    clay = BVHTree.FromPolygons(
        [v.co for v in tp_mesh.vertices], [tuple(f.vertices) for f in tp_mesh.polygons]
    )
    bm = bmesh.new()
    bm.from_mesh(dmesh)
    bm.verts.ensure_lookup_table()
    distance = [signed_distance(clay, v.co) for v in bm.verts]
    clay_verts = [v for v in bm.verts if distance[v.index] <= INSIDE_M]
    removed_vertex_rule = len(clay_verts)
    bmesh.ops.delete(bm, geom=clay_verts, context="VERTS")
    # connected components that are almost all near the clay
    bm.verts.ensure_lookup_table()
    seen = set()
    removed_components = 0
    for v in list(bm.verts):
        if v.index in seen or not v.is_valid:
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
        near = sum(1 for x in comp if abs(signed_distance(clay, x.co)) <= COMPONENT_NEAR_M)
        if near >= COMPONENT_SHARE * len(comp):
            bmesh.ops.delete(bm, geom=comp, context="VERTS")
            removed_components += 1
    bm.to_mesh(dmesh)
    bm.free()
    report = {
        "scale": scale,
        "download_vertices": len(dverts),
        "removed_inside": removed_vertex_rule,
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
    for layer in list(obj.data.uv_layers):
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
    # Project the reference image from the front onto the shell (the back view, when given, is a later bake).
    bpy.ops.object.select_all(action="DESELECT")
    body.select_set(True)
    bpy.context.view_layer.objects.active = body
    lo, hi = world_bounds(body)
    uv = body.data.uv_layers.new(name="projection")
    for loop in body.data.loops:
        co = body.data.vertices[loop.vertex_index].co
        uv.data[loop.index].uv = (
            (co.x - lo.x) / max(hi.x - lo.x, 1e-6),
            (co.z - lo.z) / max(hi.z - lo.z, 1e-6),
        )
    mat = bpy.data.materials.new("projection")
    mat.use_nodes = True
    tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(p["front_image"])
    coords = mat.node_tree.nodes.new("ShaderNodeUVMap")
    coords.uv_map = "projection"
    mat.node_tree.links.new(coords.outputs["UV"], tex.inputs["Vector"])
    mat.node_tree.links.new(tex.outputs["Color"], mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"])
    body.data.materials.clear()
    body.data.materials.append(mat)
    report = {
        "vertices": len(body.data.vertices),
        "offset_m": TIGHT_OFFSET_M,
        "covers": p["covers_regions"],
        "note": "front projection only in the pilot (the back view is not baked)",
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
