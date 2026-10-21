# SPDX-License-Identifier: GPL-3.0-or-later
"""R-12 (a): the Blender reference fit and the assembled-outfit GLB (pr-s32p-wardrobe-pilot.md §§4.3, 4.4).

For one body and its registered template T_P, and an outfit's parts (garments and hair):

1. each part's neutral-rest `garment.glb` is placed on T_P by the `.mhclo` formula (`binding.bin`), the same rule
   as `mwcast/wardrobe/garment_format.py` `fit` and `godot/wardrobe/fit_at_load.gd`; the raw fitted positions are
   written per part (`<part>_blender.bin`, body mesh frame) for W-9;
2. the push-out (L-b, ARC-138; S36-P3) runs between two phases. `"phase": "fit"` does step 1, the stretched-face
   drop and the plants for every part, writes each part's and the body's arrays (`<id>_fitted.npy`,
   `<id>_faces.npy`, `body_region.npy`) and stops. Python (`mwcast/wardrobe/pushout.py`, by winding number,
   inner layer to outer) writes `<id>_pushed.npy`. `"phase": "finish"` repeats the deterministic fit, loads the
   pushed positions (a count mismatch is refused by name) and goes on with steps 3-5;
3. skin weights by the part's rule: `inherit` (the template's weights, combined with the record's barycentric
   weights, negatives clamped, four largest, normalised) or `rigid <bone>`;
4. the body loses every triangle whose three vertices map (`corr.bin`) to template vertices all hidden by the
   worn parts' `hides.bin`;
5. export: one armature, the body (`Body`, material `MW_Cast`), one skinned mesh per part named `part_<id>` with
   one material `MW_Cast_<id>` (F-S32p-4, F-S32p-5); scene extras carry each part's provenance. Beside them,
   never drawn: `BodyCheck` (the unmasked body) and `PartCheck_<id>` (each part before the inner-face removal),
   the uncut shells C-2 and W-4w judge (S36-P3b).

The body mask of step 4 also removes a face with any vertex inside a worn part (generalized winding number,
libigl through `mwcast/contact.py`) within 15 mm of it (D-impl-22); that query runs in the
pipeline's Python (`contact_python`, `pipeline_root`), since Blender's cannot load libigl.

Params: {"phase": "fit" | "finish", "body": glb, "template": bin, "weights": bin, "corr": bin, "out": glb,
         "out_dir": dir, "regions": json, "contact_python": path, "pipeline_root": dir,
         "parts": [{"id", "dir", "layer", "weights": "inherit" | "rigid <bone>", "provenance": {...}}]}
"""

import json
import struct
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bmesh  # noqa: E402
import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402
import numpy as np  # noqa: E402 - bundled with Blender
from mathutils import Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402

STRETCH_MAX = 3.0  # L-b: a fitted edge longer than 3x its rest length (G-9's stretch bound) ...
STRETCH_MIN_M = 0.02  # ... and longer than 20 mm drops its triangle
COVER_INSIDE_M = 0.015  # L-b: skin with a worn part up to 15 mm beneath it is hidden as well
LAYER_COVER_M = 0.015  # L-b: an inner part's face is hidden only with an outer part within 15 mm outside it
# (50 mm hid Ivan's polo behind the narrow opening of his open jacket)
COVER_RAY_M = 0.05  # L-b: a body vertex whose outward normal meets a worn part within 50 mm is hidden


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


def mesh_arrays(objs):
    """The evaluated meshes of `objs` in world space, triangulated: (vertices (n, 3), triangles (m, 3))."""
    deps = bpy.context.evaluated_depsgraph_get()
    verts, tris = [], []
    for o in objs:
        mesh = o.evaluated_get(deps).to_mesh()
        mesh.calc_loop_triangles()
        base = len(verts)
        verts += [tuple(o.matrix_world @ v.co) for v in mesh.vertices]
        tris += [tuple(base + i for i in t.vertices) for t in mesh.loop_triangles]
        o.evaluated_get(deps).to_mesh_clear()
    return np.array(verts, dtype=np.float64).reshape(-1, 3), np.array(tris, dtype=np.int64).reshape(-1, 3)


def inside_parts(p, out_dir, tag, points, objs):
    """D-impl-22: which `points` lie inside one of `objs` (generalized winding number > 0.5) within
    COVER_INSIDE_M of it. Blender's Python cannot load libigl, so the query runs in the pipeline's Python
    (`mwcast.wardrobe.cover`, which calls `mwcast/contact.py`); without it the stage stops (I-S36-4: a check that
    cannot look fails)."""
    if not objs or len(points) == 0:
        return np.zeros(len(points), dtype=bool)
    query = out_dir / f"cover_{tag}.npz"
    answer = out_dir / f"cover_{tag}.npy"
    arrays = {"points": np.asarray(points, dtype=np.float64), "reach_m": np.float64(COVER_INSIDE_M)}
    for k, o in enumerate(objs):
        arrays[f"v{k}"], arrays[f"f{k}"] = mesh_arrays([o])
    np.savez(query, **arrays)
    run = subprocess.run(
        [p["contact_python"], "-m", "mwcast.wardrobe.cover", str(query), str(answer)],
        cwd=p["pipeline_root"],
        capture_output=True,
        text=True,
        check=False,
    )
    mc.log("cover", tag, run.stdout.strip(), run.stderr.strip())
    if run.returncode != 0:
        raise SystemExit(f"cover query {tag} failed ({run.returncode}): {run.stderr.strip()}")
    return np.load(answer)


def shrink_chest(obj, mm):
    """M-p-1 / X-S32-5: the upper front of the shell moved inward along its normals by `mm`."""
    zs = [v.co.z for v in obj.data.vertices]
    chest_lo = min(zs) + 0.55 * (max(zs) - min(zs))
    front = sorted(v.co.y for v in obj.data.vertices)[len(obj.data.vertices) // 4]
    shrunk = 0
    for v in obj.data.vertices:
        if v.co.z >= chest_lo and v.co.y <= front:  # upper front (the body faces -Y in Blender)
            v.co -= v.normal.normalized() * (mm / 1000.0)
            shrunk += 1
    obj.data.update()
    return shrunk


def world_positions(obj):
    mw = obj.matrix_world
    return [tuple(mw @ v.co) for v in obj.data.vertices]


def triangles_of(obj):
    """The mesh's faces as triangles (a fan per polygon), vertex indices unchanged."""
    out = []
    for poly in obj.data.polygons:
        ids = list(poly.vertices)
        out += [(ids[0], ids[k], ids[k + 1]) for k in range(1, len(ids) - 1)]
    return out


def write_arrays(out_dir, name, positions, faces, region=None):
    """The fit phase's hand-off to the Python push-out: `<name>_fitted.npy` (float64, Blender world frame),
    `<name>_faces.npy` (int32) and, for the body, `<name>_region.npy` (int32, -1 where none)."""
    np.save(out_dir / f"{name}_fitted.npy", np.asarray(positions, dtype=np.float64).reshape(-1, 3))
    np.save(out_dir / f"{name}_faces.npy", np.asarray(faces, dtype=np.int32).reshape(-1, 3))
    if region is not None:
        np.save(out_dir / f"{name}_region.npy", np.asarray(region, dtype=np.int32))


def read_pushed(path, count, name):
    if not path.exists():
        raise SystemExit(f"{name}: {path} is missing (run the push-out between the fit and finish phases)")
    pushed = np.load(path)
    if pushed.shape != (count, 3):
        raise SystemExit(f"{name}: {path} holds {pushed.shape[0]} positions for {count} vertices")
    return [tuple(float(c) for c in row) for row in pushed]


def body_regions(corr, regions_path):
    """Each body vertex's region index (of its corresponding template triangle), -1 where none."""
    region_of = json.loads(Path(regions_path).read_text(encoding="utf-8"))["vertex_region"]
    return [next((region_of[t] for t in tri if region_of[t] >= 0), -1) for tri in corr]


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

    phase = p.get("phase")
    if phase not in ("fit", "finish"):
        raise SystemExit(f"assemble: 'phase' must be 'fit' or 'finish', got {phase!r}")
    if phase == "fit":
        # the unmasked body (the future BodyCheck) and each vertex's region: the push-out's under surface
        write_arrays(
            out_dir, "body", world_positions(body), triangles_of(body), body_regions(corr, p["regions"])
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
        rest = [v.co.copy() for v in obj.data.vertices]
        for v, g in zip(obj.data.vertices, fitted_local):
            v.co = mc.from_gltf(g)
        obj.data.update()
        # L-b: a triangle the fit stretched past 3x its rest edge (and past 20 mm) is a binding to a far template
        # triangle (a hair strand, a strap end); it draws as a spike, so it is dropped (counted in the report).
        stretched = []
        for poly in obj.data.polygons:
            ids = list(poly.vertices)
            for a, b in zip(ids, ids[1:] + ids[:1]):
                now = (obj.data.vertices[a].co - obj.data.vertices[b].co).length
                was = (rest[a] - rest[b]).length
                if now > STRETCH_MAX * max(was, 1e-6) and now > STRETCH_MIN_M:
                    stretched.append(poly.index)
                    break
        if stretched:
            sb = bmesh.new()
            sb.from_mesh(obj.data)
            sb.faces.ensure_lookup_table()
            bmesh.ops.delete(sb, geom=[sb.faces[i] for i in stretched], context="FACES_ONLY")
            sb.to_mesh(obj.data)
            sb.free()
            obj.data.update()
        # A planted mutation (test only, never a deliverable): M-p-1 / X-S32-5, the shell shrunk on the chest.
        plant = part.get("plant_shrink_chest_mm")
        if plant:
            shrunk = shrink_chest(obj, float(plant))
            report.setdefault("planted", {})[part["id"]] = {"shrink_mm": plant, "vertices": shrunk}
            mc.log("PLANTED", part["id"], "chest shrunk", plant, "mm on", shrunk, "vertices")
        # A planted mutation (test only, S36-P3): the whole shell scaled about its centroid by this percentage.
        scale_pct = part.get("plant_scale_pct")
        if scale_pct:
            centre = sum((v.co for v in obj.data.vertices), Vector()) / len(obj.data.vertices)
            for v in obj.data.vertices:
                v.co = centre + (v.co - centre) * (1.0 + float(scale_pct) / 100.0)
            obj.data.update()
            report.setdefault("planted", {})[part["id"] + "_scale"] = {"scale_pct": scale_pct}
            mc.log("PLANTED", part["id"], "scaled", scale_pct, "%")
        # The push-out (S36-P3; L-b's rule, ARC-138) runs in Python between the two phases, by winding number
        # (`mwcast/wardrobe/pushout.py`): the fit phase writes each part's fitted positions and faces, and the
        # finish phase loads the pushed positions. Nothing is installed into Blender (I-P3-2).
        if phase == "fit":
            write_arrays(out_dir, part["id"], world_positions(obj), triangles_of(obj))
            report["parts"][part["id"]] = {
                "vertices": len(records),
                "stretched_triangles_dropped": len(stretched),
            }
            continue
        pushed = read_pushed(out_dir / f"{part['id']}_pushed.npy", len(obj.data.vertices), part["id"])
        inv = obj.matrix_world.inverted()
        for v, world in zip(obj.data.vertices, pushed):
            v.co = inv @ Vector(world)
        obj.data.update()
        # A planted mutation (test only, never a deliverable): the chest shrunk after the push-out, so the
        # push-out cannot repair it and C-2 / W-4w must see it.
        late = part.get("plant_after_pushout_mm")
        if late:
            shrunk = shrink_chest(obj, float(late))
            entry = {"shrink_mm": late, "vertices": shrunk}
            report.setdefault("planted", {})[part["id"] + "_after_pushout"] = entry
            mc.log("PLANTED", part["id"], "chest shrunk after the push-out", late, "mm on", shrunk)
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
            "stretched_triangles_dropped": len(stretched),
            "layer": part["layer"],
        }
        mc.log("part", part["id"], "vertices", len(records))

    if phase == "fit":
        mc.write_result(out_dir / "assemble_fit.json", report)
        mc.log("fit phase done", out_dir)
        return

    # W-4's reference body: the whole body, unmasked, exported beside the drawn one as `BodyCheck` (never drawn:
    # the pose sheet hides it and judges with it; the static gates skip it). The mask hides skin; W-4 asks
    # whether the garment encloses the skin in its covered regions, masked or not (X-S32-5's shrunk chest).
    check = body.copy()
    check.data = body.data.copy()
    check.name = "BodyCheck"
    bpy.context.scene.collection.objects.link(check)
    # S36-P3b (Q-S36-P3-4): each part as it stands before the inner-face removal below, exported beside the drawn
    # one as `PartCheck_<id>` (never drawn, never part of a cover or mask computation). C-2 and W-4w read these
    # uncut shells, so a poke under an outer layer stays visible where the drawn inner shell has been cut.
    uncut = []
    for part, part_obj in placed:
        copy = part_obj.copy()
        copy.data = part_obj.data.copy()
        copy.name = f"PartCheck_{part['id']}"
        copy.data.name = copy.name
        bpy.context.scene.collection.objects.link(copy)
        uncut.append((part, copy))
    # L-b, the fitted cover: a body vertex is also hidden when the ray along its outward normal meets a worn
    # (non-rigid) part within 50 mm in the assembled rest pose. The template hidden sets are computed on each
    # garment's own clay; this one is computed on the fit actually drawn, so skin under a close garment does not
    # show through it.
    deps = bpy.context.evaluated_depsgraph_get()
    cover_verts, cover_polys = [], []
    # a rigid part (the backpack) and hair (strands hang in front of the face) never hide skin this way
    worn = [
        o
        for part, o in placed
        if not str(part.get("weights", "inherit")).startswith("rigid") and part.get("class") != "hair"
    ]
    for part_obj in worn:
        mesh = part_obj.evaluated_get(deps).to_mesh()
        base = len(cover_verts)
        cover_verts += [part_obj.matrix_world @ v.co for v in mesh.vertices]
        cover_polys += [tuple(base + i for i in poly.vertices) for poly in mesh.polygons]
        part_obj.evaluated_get(deps).to_mesh_clear()
    covered = set()
    if cover_polys:
        cover_bvh = BVHTree.FromPolygons(cover_verts, cover_polys)
        n3 = to_world.to_3x3()
        for v in body.data.vertices:
            normal = (n3 @ v.normal).normalized()
            at = to_world @ v.co
            hit, _n, _i, _d = cover_bvh.ray_cast(at + normal * 0.0005, normal, COVER_RAY_M)
            if hit is None:
                # skin standing just outside a part (a flat garment triangle across curved skin) is under
                # the garment too: hidden when the part lies within 15 mm inside it
                hit, _n, _i, _d = cover_bvh.ray_cast(at, -normal, COVER_INSIDE_M)
            if hit is not None:
                covered.add(v.index)
    report["body_vertices_covered_by_fit"] = len(covered)
    # D-impl-22: skin poking through a worn part. A decimated shell's flat triangles cut across curved skin, so a
    # body face can have one vertex a few mm outside the shell and the others inside: the all-vertices rule below
    # keeps that face, and it draws as a pale fleck on the garment (Vera's jeans, Hana's tights). A face with any
    # vertex inside a worn part (winding number) within 15 mm of it is under the garment and is removed.
    poked = inside_parts(p, out_dir, "body", [tuple(to_world @ v.co) for v in body.data.vertices], worn)
    report["body_vertices_inside_worn_parts"] = int(poked.sum())
    # L-b, layers: an inner part's face under an outer worn part (same rule as the body's cover) is removed, so a
    # tee never shows through a hoodie where the two shells cross (the white shards on Vera's hoodie)
    for k, (part, part_obj) in enumerate(placed):
        if part.get("class") == "hair":
            continue
        outer = [
            o
            for q, o in placed[k + 1 :]
            if q["layer"] > part["layer"]
            and q.get("class") != "hair"
            and not str(q.get("weights", "inherit")).startswith("rigid")
        ]
        if not outer:
            continue
        ov, op = [], []
        for o in outer:
            mesh = o.evaluated_get(deps).to_mesh()
            base = len(ov)
            ov += [o.matrix_world @ v.co for v in mesh.vertices]
            op += [tuple(base + i for i in poly.vertices) for poly in mesh.polygons]
            o.evaluated_get(deps).to_mesh_clear()
        outer_bvh = BVHTree.FromPolygons(ov, op)
        mw3 = part_obj.matrix_world
        n3p = mw3.to_3x3()
        lb = bmesh.new()
        lb.from_mesh(part_obj.data)
        # D-impl-22 is not applied between layers: an inner layer lies inside an open outer one everywhere near its
        # opening (a tee inside an open hoodie), and the winding-number rule cut the tee's visible front away
        under = []
        for face in lb.faces:
            c = mw3 @ face.calc_center_median()
            n = (n3p @ face.normal).normalized()
            hit = outer_bvh.ray_cast(c + n * 0.0005, n, LAYER_COVER_M)[0]
            if hit is None:
                hit = outer_bvh.ray_cast(c, -n, COVER_INSIDE_M)[0]
            if hit is not None:
                under.append(face)
        if under:
            bmesh.ops.delete(lb, geom=under, context="FACES_ONLY")
            lb.to_mesh(part_obj.data)
            part_obj.data.update()
        lb.free()
        report["parts"][part["id"]]["faces_under_outer_layers"] = len(under)
    # The body mask.
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    gone = []
    for face in bm.faces:
        if any(poked[v.index] for v in face.verts) or all(
            v.index in covered or all(t in hidden for t in corr[v.index]) for v in face.verts
        ):
            gone.append(face)
    bmesh.ops.delete(bm, geom=gone, context="FACES_ONLY")
    bm.to_mesh(body.data)
    bm.free()
    # Each body vertex's region (of its corresponding template triangle) rides in a second UV map, `u` = region
    # index + 0.5 (glTF TEXCOORD_1, Godot ARRAY_TEX_UV2), so the pose sheet's W-4 knows a vertex's region however
    # the exporter and the importer order the vertices. Parts carry u = -1 (no region).
    body_region = body_regions(corr, p["regions"])

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

    targets = [(body, None, -1.0), (check, None, -1.0)] + [(o, -1, prt["layer"]) for prt, o in placed + uncut]
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
    for _part, obj in placed + uncut:
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
