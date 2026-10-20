# SPDX-License-Identifier: GPL-3.0-or-later
"""Hidden test fixtures for the wardrobe gates (pr-s32p-wardrobe-pilot.md §4.8 W-5; C3 validation; DEP-72).

Never drawn on a sheet and never a deliverable (`role: fixture`). Built with MPFB2 2.0.18:

1. **The neutral template**: `hm08` with MPFB2's default macros, in the body frame (it faces +Z in glTF like
   every pipeline export); written as `neutral_template.bin`.
2. **Fixture bodies** (`bodies`: name -> macros + stature): each an MPFB2 human with the built-in `mixamo` rig,
   stature-calibrated like R-4, exported as `<name>.glb` (the armature and the body
   mesh, helpers removed), plus `<name>_exact_template.bin`: its own `hm08` positions (all 19,158), the exact
   template a perfect registration would find.
3. **Stock CC0 garments** (`garments`: id -> .mhclo path): each fitted by MPFB2 itself
   (`add_mhclo_asset`, no subdivision) to the neutral template and to every fixture body; the fitted vertex
   positions are written as `<garment>_on_<body|neutral>_mpfb.bin` (MWGT layout, mesh order), the oracle for our
   formula; and the neutral fit is exported as `<garment>/garment.glb` (one static surface with UVs, normals and
   tangents, no skin), the fixture garment's rest shape.

Params: {"out_dir", "bodies": {name: {"macros": {...}, "stature_m": float}}, "garments": {id: mhclo path}}.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402


def write_positions(path, positions):
    data = bytearray(b"MWGT" + struct.pack("<I", len(positions)))
    for v in positions:
        data += struct.pack("<3f", *mc.to_gltf(v))
    Path(path).write_bytes(bytes(data))


def mesh_positions(obj):
    deps = bpy.context.evaluated_depsgraph_get()
    ev = obj.evaluated_get(deps)
    mesh = ev.to_mesh()
    out = [obj.matrix_world @ v.co for v in mesh.vertices]
    ev.to_mesh_clear()
    return out


def add_garment(path, basemesh):
    HumanService, _ = mc.mpfb()
    return HumanService.add_mhclo_asset(
        path,
        basemesh,
        asset_type="Clothes",
        subdiv_levels=0,
        material_type="GAMEENGINE",
        set_up_rigging=False,
    )


def export(objects, path, skins):
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_skins=skins,
        export_animations=False,
        export_morph=False,
        export_tangents=True,
        export_normals=True,
        export_texcoords=True,
    )


def main():
    p = mc.params()
    out = Path(p["out_dir"])
    out.mkdir(parents=True, exist_ok=True)
    report = {"bodies": {}, "garments": {}}

    # 1. The neutral template, and the stock garments on it.
    mc.clear_scene()
    HumanService, TargetService = mc.mpfb()
    neutral = HumanService.create_human(macro_detail_dict=TargetService.get_default_macro_info_dict())
    write_positions(out / "neutral_template.bin", mc.evaluated_positions(neutral)[0])
    for gid, path in p["garments"].items():
        clothes = add_garment(path, neutral)
        clothes.parent = None
        clothes.matrix_world = neutral.matrix_world.copy()
        bpy.context.view_layer.update()
        positions = mesh_positions(clothes)
        write_positions(out / f"{gid}_on_neutral_mpfb.bin", positions)
        folder = out / gid
        folder.mkdir(exist_ok=True)
        for mod in list(clothes.modifiers):
            clothes.modifiers.remove(mod)
        export([clothes], folder / "garment.glb", skins=False)
        tris = sum(len(poly.vertices) - 2 for poly in clothes.data.polygons)
        report["garments"][gid] = {"vertices": len(positions), "triangles": tris, "mhclo": path}
        mc.log("garment", gid, "vertices", len(positions), "triangles", tris)
        bpy.data.objects.remove(clothes, do_unlink=True)

    # 2. The fixture bodies, each with every garment fitted by MPFB2 itself.
    for name, spec in p["bodies"].items():
        mc.clear_scene()
        basemesh, _ids, calibration = mc.calibrate_height(spec["macros"], float(spec["stature_m"]))
        rig = HumanService.add_builtin_rig(basemesh, "mixamo")
        exact, _ = mc.evaluated_positions(basemesh)
        write_positions(out / f"{name}_exact_template.bin", exact)
        for gid, path in p["garments"].items():
            clothes = add_garment(path, basemesh)
            write_positions(out / f"{gid}_on_{name}_mpfb.bin", mesh_positions(clothes))
            bpy.data.objects.remove(clothes, do_unlink=True)
        for mod in basemesh.modifiers:
            if mod.type == "MASK":
                mod.show_viewport = True
        export([rig, basemesh], out / f"{name}.glb", skins=True)
        report["bodies"][name] = {"calibration": calibration, "role": "fixture"}
        mc.log("body", name, calibration["stature_m"])
    mc.write_result(out / "fixtures.json", report)


main()
