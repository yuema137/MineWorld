# SPDX-License-Identifier: GPL-3.0-or-later
"""R-10 and R-11: bind a garment to the `hm08` template with MakeClothes, and its hidden set
(pr-s32p-wardrobe-pilot.md §4.3; DEP-72).

**bind** (R-10). The template is set to the positions the garment was made on (`template`: the wearer's T_P, or
the neutral template), helpers included and the helper mask removed so vertex ids stay the template's; the
garment (`garment.glb`, one surface, in the same body frame) gets every vertex in the basemesh's `body` group, as
MakeClothes requires; MPFB2's MakeClothes (`ClothesService.create_mhclo_from_clothes_matching`) matches each
garment vertex to template vertices. The binding is written from MakeClothes' records in memory, at full
precision, as `binding.bin` (the `.mhclo` file it can write rounds weights to 4 decimals, which W-1's 1e-4 sum
bound would see); the `.mhclo` is also written, for audit. The garment's vertex order is the GLB's, so the
binding needs no re-ordering.

**hides** (R-11). On the template the garment was fitted to (`template`), a body vertex of the garment's covered
regions is hidden when the garment shell is hit along its outward normal within 100 mm and also along four
directions tilted 20° from it (front, back, left, right): coverage with a margin, so a vertex near a hem or an
opening stays visible. The pilot computes this in the rest pose only (the W-gate pose set is the W-4 and W-7
sweeps' business; recorded as a deviation).

Params: {"op": "bind" | "hides", "template": bin, "garment": glb, "out_dir": dir, "id": str,
         "regions": json, "covers_regions": [..]}
"""

import json
import math
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402

HIDE_RAY_M = 0.10
HIDE_TILT_DEG = 20.0


def read_template(path):
    data = Path(path).read_bytes()
    if data[:4] != b"MWGT":
        raise SystemExit(f"{path}: not a template file")
    (n,) = struct.unpack_from("<I", data, 4)
    values = struct.unpack_from(f"<{3 * n}f", data, 8)
    return [Vector(values[3 * i : 3 * i + 3]) for i in range(n)]


def template_object(positions):
    """The MPFB2 basemesh with its vertices set to `positions` (glTF frame), shape keys and helper mask removed."""
    HumanService, TargetService = mc.mpfb()
    basemesh = HumanService.create_human(macro_detail_dict=TargetService.get_default_macro_info_dict())
    for mod in list(basemesh.modifiers):
        basemesh.modifiers.remove(mod)
    if basemesh.data.shape_keys:
        bpy.context.view_layer.objects.active = basemesh
        basemesh.shape_key_clear()
    basemesh.matrix_world = Matrix.Identity(4)
    if len(positions) != len(basemesh.data.vertices):
        raise SystemExit(f"template has {len(positions)} positions, hm08 {len(basemesh.data.vertices)}")
    for v, p in zip(basemesh.data.vertices, positions):
        v.co = mc.from_gltf(p)
    basemesh.data.update()
    return basemesh


def import_garment(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    meshes = [o for o in bpy.data.objects if o not in before and o.type == "MESH"]
    garment = meshes[0]
    garment.parent = None
    garment.matrix_world = Matrix.Identity(4)
    return garment


def bind(p, out):
    from bl_ext.user_default.mpfb.services.clothesservice import ClothesService

    basemesh = template_object(read_template(p["template"]))
    garment = import_garment(p["garment"])
    for vg in list(garment.vertex_groups):
        garment.vertex_groups.remove(vg)
    group = garment.vertex_groups.new(name="body")
    group.add(list(range(len(garment.data.vertices))), 1.0, "REPLACE")
    report = ClothesService.mesh_is_valid_as_clothes(garment, basemesh)
    if not report["all_checks_ok"]:
        raise SystemExit("MakeClothes refuses the garment: " + json.dumps(report, default=str))
    mhclo = ClothesService.create_mhclo_from_clothes_matching(
        basemesh, garment, properties_dict={"name": p["id"]}
    )
    reference = ClothesService.get_reference_scale(basemesh)
    mhclo.write_mhclo(str(out / f"{p['id']}.mhclo"), also_export_obj=False, reference_scale=reference)
    # MakeHuman axes: x = Blender x, y = Blender z, z = -Blender y; MakeClothes writes y_scale from Blender's z
    # references and z_scale from Blender's y references ("Y and Z are flipped in MakeHuman").
    # The rest distances are re-measured per axis on the bind template, in MakeHuman units (the scale factor):
    # the fit (MakeHuman's and MPFB2's own `fit_clothes_to_human`) divides a per-axis difference by them, while
    # MakeClothes records Euclidean lengths; per axis, a fit on the bind template reproduces the garment exactly.
    positions = read_template(p["template"])
    unit = 0.1  # MPFB2's basemesh scale factor: one MakeHuman unit is 0.1 m
    refs = []
    for axis, (lo, hi) in enumerate((("xmin", "xmax"), ("zmin", "zmax"), ("ymin", "ymax"))):
        a, b = reference[lo], reference[hi]
        refs.append((a, b, abs(positions[a][axis] - positions[b][axis]) / unit))
    data = bytearray(b"MWGB" + struct.pack("<II", 1, len(garment.data.vertices)))
    for a, b, rest in refs:
        data += struct.pack("<IIf", a, b, rest)
    strategies = {}
    for k in range(len(garment.data.vertices)):
        line = mhclo.verts[k]
        verts, weights, offsets = line["verts"], line["weights"], line["offsets"]
        data += struct.pack("<3I3f3f", *verts, *weights, *offsets)
        key = "exact" if weights[0] == 1 and weights[1] == 0 and weights[2] == 0 else "face"
        strategies[key] = strategies.get(key, 0) + 1
    (out / "binding.bin").write_bytes(bytes(data))
    mc.write_result(
        out / "bind.json",
        {
            "vertices": len(garment.data.vertices),
            "strategies": strategies,
            "reference_scale": {k: v for k, v in reference.items()},
        },
    )
    mc.log("bind", p["id"], len(garment.data.vertices), strategies)


def hides(p, out):
    basemesh = template_object(read_template(p["template"]))
    garment = import_garment(p["garment"])
    deps = bpy.context.evaluated_depsgraph_get()
    gmesh = garment.evaluated_get(deps).to_mesh()
    verts = [garment.matrix_world @ v.co for v in gmesh.vertices]
    polys = [tuple(poly.vertices) for poly in gmesh.polygons]
    bvh = BVHTree.FromPolygons(verts, polys)
    regions = json.loads(Path(p["regions"]).read_text(encoding="utf-8"))
    names = regions["regions"]
    wanted = {names.index(r) for r in p["covers_regions"]}
    region_of = regions["vertex_region"]
    basemesh.data.update()
    hidden = []
    tilt = math.radians(HIDE_TILT_DEG)
    for v in basemesh.data.vertices:
        if region_of[v.index] not in wanted:
            continue
        n = v.normal.normalized()
        side = n.cross(Vector((0.0, 0.0, 1.0)))
        if side.length < 1e-6:
            side = Vector((1.0, 0.0, 0.0))
        side.normalize()
        up = n.cross(side).normalized()
        directions = [n] + [
            (n * math.cos(tilt) + d * math.sin(tilt)).normalized() for d in (side, -side, up, -up)
        ]
        if all(bvh.ray_cast(v.co, d, HIDE_RAY_M)[0] is not None for d in directions):
            hidden.append(v.index)
    data = b"MWGH" + struct.pack(f"<I{len(hidden)}I", len(hidden), *sorted(hidden))
    (out / "hides.bin").write_bytes(data)
    mc.write_result(out / "hides.json", {"hidden": len(hidden), "covers_regions": p["covers_regions"]})
    mc.log("hides", p["id"], len(hidden))


def main():
    p = mc.params()
    out = Path(p["out_dir"])
    out.mkdir(parents=True, exist_ok=True)
    mc.clear_scene()
    if p["op"] == "bind":
        bind(p, out)
    elif p["op"] == "hides":
        hides(p, out)
    else:
        raise SystemExit("unknown op " + str(p["op"]))


main()
