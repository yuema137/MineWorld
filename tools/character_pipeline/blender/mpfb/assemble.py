# SPDX-License-Identifier: GPL-3.0-or-later
"""R-12 (a): the Blender reference fit and the assembled-outfit GLB (pr-s32p-wardrobe-pilot.md §§4.3, 4.4).

For one body and its registered template T_P, and an outfit's parts (garments and hair):

1. each part's neutral-rest `garment.glb` is placed on T_P by the `.mhclo` formula (`binding.bin`), the same rule
   as `mwcast/wardrobe/garment_format.py` `fit` and `godot/wardrobe/fit_at_load.gd`; the raw fitted positions are
   written per part (`<part>_blender.bin`, body mesh frame) for W-9;
2. body push-out (L-b, ARC-138 ruling 2026-10-19): a part vertex inside the body or less than 2 mm outside it
   moves to 2 mm outside the nearest body surface point (within 100 mm);
   then layer push-out in slot order (inner first): a vertex of an outer part lying less than 2 mm outside the
   outermost inner surface along its normal is moved to 2 mm outside it;
3. skin weights by the part's rule: `inherit` (the template's weights, combined with the record's barycentric
   weights, negatives clamped, four largest, normalised) or `rigid <bone>`;
4. the body loses every triangle whose three vertices map (`corr.bin`) to template vertices all hidden by the
   worn parts' `hides.bin`;
5. export: one armature, the body (`Body`, material `MW_Cast`), one skinned mesh per part named `part_<id>` with
   one material `MW_Cast_<id>` (F-S32p-4, F-S32p-5); scene extras carry each part's provenance.

Params: {"body": glb, "template": bin, "weights": bin, "corr": bin, "out": glb, "out_dir": dir,
         "parts": [{"id", "dir", "layer", "weights": "inherit" | "rigid <bone>", "provenance": {...}}]}
"""

import json
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bmesh  # noqa: E402
import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402
from mathutils import Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402

PUSH_M = 0.002  # 2 mm, ≈ cloth thickness (step-32 §6.2 layer push-out; W-4's 2 mm)
REACH_M = 0.05  # how far outside an outer vertex the inward ray starts
BODY_REACH_M = 0.10  # L-b: a part vertex farther than this from the body is never moved by the body push-out


def read(path, magic):
    data = Path(path).read_bytes()
    if data[:4] != magic:
        raise SystemExit(f"{path}: magic {data[:4]!r}, expected {magic!r}")
    return data


def read_template(path):
    data = read(path, b"MWGT")
    (n,) = struct.unpack_from("<I", data, 4)
    values = struct.unpack_from(f"<{3 * n}f", data, 8)
    return [Vector(values[3 * i : 3 * i + 3]) for i in range(n)]


def read_weights(path):
    data = read(path, b"MWGW")
    n, j = struct.unpack_from("<II", data, 4)
    off = 12
    names = []
    for _ in range(j):
        (length,) = struct.unpack_from("<B", data, off)
        names.append(data[off + 1 : off + 1 + length].decode("utf-8"))
        off += 1 + length
    rows = []
    for k in range(n):
        v = struct.unpack_from("<BfBfBfBf", data, off + 20 * k)
        rows.append([(v[2 * i], v[2 * i + 1]) for i in range(4)])
    return names, rows


def read_binding(path):
    data = read(path, b"MWGB")
    _version, n = struct.unpack_from("<II", data, 4)
    scale = [struct.unpack_from("<IIf", data, 12 + 12 * i) for i in range(3)]
    records = [struct.unpack_from("<3I3f3f", data, 48 + 36 * k) for k in range(n)]
    return scale, records


def read_ids(path, magic=b"MWGH"):
    data = read(path, magic)
    (n,) = struct.unpack_from("<I", data, 4)
    return set(struct.unpack_from(f"<{n}I", data, 8))


def read_corr(path):
    data = read(path, b"MWGC")
    (n,) = struct.unpack_from("<I", data, 4)
    return [struct.unpack_from("<3I", data, 8 + 20 * k) for k in range(n)]


def fit(scale, records, template):
    s = [abs(template[a][axis] - template[b][axis]) / rest for axis, (a, b, rest) in enumerate(scale)]
    out = []
    for r in records:
        v0, v1, v2, w0, w1, w2, ox, oy, oz = r
        p = template[v0] * w0 + template[v1] * w1 + template[v2] * w2
        out.append(p + Vector((ox * s[0], oy * s[1], oz * s[2])))
    return out


def part_weights(records, names, rows, rule):
    if rule.startswith("rigid "):
        bone = rule.split(" ", 1)[1]
        bone = bone if bone.startswith(mc.M) else mc.M + bone
        return [{bone: 1.0} for _ in records]
    out = []
    for r in records:
        acc = {}
        for i in range(3):
            bw = max(r[3 + i], 0.0)
            for j, w in rows[r[i]]:
                if w > 0.0:
                    acc[names[j]] = acc.get(names[j], 0.0) + w * bw
        top = sorted(acc.items(), key=lambda kv: (-kv[1], kv[0]))[:4]
        total = sum(w for _n, w in top) or 1.0
        out.append({n: w / total for n, w in top})
    return out


def import_glb(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    return [o for o in bpy.data.objects if o not in before]


def main():
    p = mc.params()
    out_dir = Path(p["out_dir"])
    out_dir.mkdir(parents=True, exist_ok=True)
    mc.clear_scene()
    new = import_glb(p["body"])
    arm = next(o for o in new if o.type == "ARMATURE")
    body = max((o for o in new if o.type == "MESH"), key=lambda o: len(o.data.vertices))
    body.name = "Body"
    # The pipeline export's armature name (A-p-5): the pose sheet's import settings find the skeleton under it.
    arm.name = "mixamo_vitruvian"
    to_world = body.matrix_world.copy()
    template = read_template(p["template"])
    names, rows = read_weights(p["weights"])
    corr = read_corr(p["corr"])
    report = {"parts": {}, "hidden_template_vertices": 0, "body_triangles_removed": 0}

    body_bvh = BVHTree.FromPolygons(
        [to_world @ v.co for v in body.data.vertices], [tuple(f.vertices) for f in body.data.polygons]
    )
    parts = sorted(p["parts"], key=lambda part: part["layer"])
    placed = []  # (part, object) inner first
    hidden = set()
    for part in parts:
        folder = Path(part["dir"])
        scale, records = read_binding(folder / "binding.bin")
        fitted_local = fit(scale, records, template)
        with open(out_dir / f"{part['id']}_blender.bin", "wb") as fh:
            fh.write(b"MWGT" + struct.pack("<I", len(fitted_local)))
            for v in fitted_local:
                fh.write(struct.pack("<3f", *v))
        objs = [o for o in import_glb(folder / "garment.glb") if o.type == "MESH"]
        obj = objs[0]
        if len(obj.data.vertices) != len(records):
            raise SystemExit(
                f"{part['id']}: {len(obj.data.vertices)} vertices for {len(records)} binding records"
            )
        obj.matrix_world = to_world
        for v, g in zip(obj.data.vertices, fitted_local):
            v.co = mc.from_gltf(g)
        obj.data.update()
        # A planted mutation (test only, never a deliverable): M-p-1 / X-S32-5, the shell shrunk on the chest.
        plant = part.get("plant_shrink_chest_mm")
        if plant:
            chest_lo = min(v.co.z for v in obj.data.vertices) + 0.55 * (
                max(v.co.z for v in obj.data.vertices) - min(v.co.z for v in obj.data.vertices)
            )
            front = sorted(v.co.y for v in obj.data.vertices)[len(obj.data.vertices) // 4]
            shrunk = 0
            for v in obj.data.vertices:
                if v.co.z >= chest_lo and v.co.y <= front:  # upper front (the body faces -Y in Blender)
                    v.co -= v.normal.normalized() * (float(plant) / 1000.0)
                    shrunk += 1
            obj.data.update()
            report.setdefault("planted", {})[part["id"]] = {"shrink_mm": plant, "vertices": shrunk}
            mc.log("PLANTED", part["id"], "chest shrunk", plant, "mm on", shrunk, "vertices")
        # L-b (ARC-138 ruling 2026-10-19): the body is the innermost layer. A part vertex inside the body, or
        # less than 2 mm outside it, moves to 2 mm outside the nearest body surface point (rest pose).
        moved = 0
        body_pushed = 0
        mw = obj.matrix_world
        inv = mw.inverted()
        for v in obj.data.vertices:
            world = mw @ v.co
            hit, normal, _i, dist = body_bvh.find_nearest(world, BODY_REACH_M)
            if hit is None:
                continue
            gap = (world - hit).dot(normal)
            if gap < PUSH_M:
                v.co = inv @ (hit + normal * PUSH_M)
                body_pushed += 1
        obj.data.update()
        # Layer push-out against the parts already placed (inner layers).
        if placed:
            deps = bpy.context.evaluated_depsgraph_get()
            verts, polys = [], []
            for _inner, inner_obj in placed:
                mesh = inner_obj.evaluated_get(deps).to_mesh()
                base = len(verts)
                verts += [inner_obj.matrix_world @ v.co for v in mesh.vertices]
                polys += [tuple(base + i for i in poly.vertices) for poly in mesh.polygons]
                inner_obj.evaluated_get(deps).to_mesh_clear()
            bvh = BVHTree.FromPolygons(verts, polys)
            mw = obj.matrix_world
            inv = mw.inverted()
            for v in obj.data.vertices:
                world = mw @ v.co
                normal = (mw.to_3x3() @ v.normal).normalized()
                hit, _n, _i, _d = bvh.ray_cast(world + normal * REACH_M, -normal, REACH_M * 2)
                if hit is None:
                    continue
                gap = (world - hit).dot(normal)
                if gap < PUSH_M and gap > -REACH_M:
                    v.co = inv @ (hit + normal * PUSH_M)
                    moved += 1
            obj.data.update()
        # Weights.
        for vg in list(obj.vertex_groups):
            obj.vertex_groups.remove(vg)
        groups = {}
        for k, w in enumerate(part_weights(records, names, rows, part["weights"])):
            for bone, value in w.items():
                if bone not in groups:
                    groups[bone] = obj.vertex_groups.new(name=bone)
                groups[bone].add([k], value, "REPLACE")
        obj.parent = arm
        obj.matrix_parent_inverse = arm.matrix_world.inverted()
        mod = obj.modifiers.new("Armature", "ARMATURE")
        mod.object = arm
        obj.name = f"part_{part['id']}"
        obj.data.name = obj.name
        if obj.data.materials:
            obj.data.materials[0].name = f"MW_Cast_{part['id']}"
        hides_path = folder / "hides.bin"
        if hides_path.exists():
            hidden |= read_ids(hides_path)
        placed.append((part, obj))
        report["parts"][part["id"]] = {
            "vertices": len(records),
            "pushed_out": moved,
            "pushed_out_of_body": body_pushed,
            "layer": part["layer"],
        }
        mc.log("part", part["id"], "vertices", len(records), "pushed out", moved)

    # W-4's reference body: the whole body, unmasked, exported beside the drawn one as `BodyCheck` (never drawn:
    # the pose sheet hides it and judges with it; the static gates skip it). The mask hides skin; W-4 asks
    # whether the garment encloses the skin in its covered regions, masked or not (X-S32-5's shrunk chest).
    check = body.copy()
    check.data = body.data.copy()
    check.name = "BodyCheck"
    bpy.context.scene.collection.objects.link(check)
    # The body mask.
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    gone = []
    for face in bm.faces:
        if all(all(t in hidden for t in corr[v.index]) for v in face.verts):
            gone.append(face)
    bmesh.ops.delete(bm, geom=gone, context="FACES_ONLY")
    bm.to_mesh(body.data)
    bm.free()
    # Each body vertex's region (of its corresponding template triangle) rides in a second UV map, `u` = region
    # index + 0.5 (glTF TEXCOORD_1, Godot ARRAY_TEX_UV2), so the pose sheet's W-4 knows a vertex's region however
    # the exporter and the importer order the vertices. Parts carry u = -1 (no region).
    region_of = json.loads(Path(p["regions"]).read_text(encoding="utf-8"))["vertex_region"]
    body_region = []
    for tri in corr:
        found = next((region_of[t] for t in tri if region_of[t] >= 0), -1)
        body_region.append(found)

    # `v` says whether the vertex lies under an outer layer in the rest pose (v = 1.5) or not (v = 0.5): its
    # outward normal meets a part of a higher layer (any part, for the body) within 50 mm. W-4 judges only
    # vertices that are under something: skin at an open neckline or below a hem is not covered.
    def under_flags(obj, layer_value):
        outer = [o for prt, o in placed if prt["layer"] > layer_value]
        if not outer:
            return [False] * len(obj.data.vertices)
        deps = bpy.context.evaluated_depsgraph_get()
        verts, polys = [], []
        for o in outer:
            mesh = o.evaluated_get(deps).to_mesh()
            base = len(verts)
            verts += [o.matrix_world @ v.co for v in mesh.vertices]
            polys += [tuple(base + i for i in poly.vertices) for poly in mesh.polygons]
            o.evaluated_get(deps).to_mesh_clear()
        bvh = BVHTree.FromPolygons(verts, polys)
        mw = obj.matrix_world
        flags = []
        for v in obj.data.vertices:
            n = (mw.to_3x3() @ v.normal).normalized()
            flags.append(bvh.ray_cast(mw @ v.co + n * 0.0005, n, 0.05)[0] is not None)
        return flags

    targets = [(body, None, -1.0), (check, None, -1.0)] + [(o, -1, prt["layer"]) for prt, o in placed]
    for obj, values, layer_value in targets:
        flags = under_flags(obj, layer_value)
        layer = obj.data.uv_layers.new(name="region")
        for loop in obj.data.loops:
            u = (body_region[loop.vertex_index] if values is None else values) + 0.5
            layer.data[loop.index].uv = (u, 1.5 if flags[loop.vertex_index] else 0.5)
    report["hidden_template_vertices"] = len(hidden)
    report["body_triangles_removed"] = len(gone)

    bpy.context.scene["mineworld_provenance"] = json.dumps({part["id"]: part["provenance"] for part in parts})
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    body.select_set(True)
    check.select_set(True)
    for _part, obj in placed:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.export_scene.gltf(
        filepath=str(p["out"]),
        export_format="GLB",
        use_selection=True,
        export_skins=True,
        export_animations=False,
        export_morph=False,
        export_extras=True,
    )
    mc.write_result(out_dir / "assemble.json", report)
    mc.log("assembled", p["out"], report)


main()
