# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 6, the game-ready mesh: port of `gameready_mesh.py` (Route D+ step 3), parameterized.

Run by the driver as: blender --background --factory-startup --python-exit-code 1 --python gameready.py -- params.json

Statement for statement the D+ code, except where a constant is read from params.json (PIPELINE.md §2.3):
the target triangle count, the eye line and sole heights, the centre band, the weld distance, and the bake
settings. Writes gameready.blend and gameready.json (scale, sole_raw, cx, cy, counts, bake device).
"""

import bmesh
import bpy
import os
import sys
from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
SRC = p["download_glb"]
OUT = p["out_dir"]
TARGET = p["target_triangles"]
EYE_Z_MESHY = p["eye_z_raw"]  # D+: the eye line read on the rig/v3_front.png view (rig_meshy.py's pick)
SOLE_Z = p["sole_z_m"]  # the current character's sole height
EYE_FROM_SOLE = p["eye_from_sole_m"]  # the current character's sole-to-eye-line height, m
BAND_Z = p["centre_band_z_raw"]
BAND_HALF = p["centre_band_half_raw"]
WELD = p["weld_m"]
BAKE = p["bake"]
os.makedirs(OUT, exist_ok=True)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=SRC)
src = next(o for o in bpy.data.objects if o.type == "MESH")
src.data.transform(src.matrix_world)
src.parent = None
src.matrix_world.identity()
me = src.data
zs = [v.co.z for v in me.vertices]
sole = min(zs)
s = EYE_FROM_SOLE / (EYE_Z_MESHY - sole)
# centre: x on the midline of the legs region, y on the hip-height cross-section
hip_band = [v.co for v in me.vertices if abs(v.co.z - BAND_Z) < BAND_HALF]
cx = 0.5 * (min(p.x for p in hip_band) + max(p.x for p in hip_band))
cy = sum(p.y for p in hip_band) / len(hip_band)
M = Matrix.Translation((0, 0, SOLE_Z)) @ Matrix.Scale(s, 4) @ Matrix.Translation((-cx, -cy, -sole))
me.transform(M)
me.update()
print(f"scale {s:.4f} (sole {sole:.4f}, eye {EYE_Z_MESHY}); centre offset x {cx:.4f} y {cy:.4f}")
zs = [v.co.z for v in me.vertices]
print(f"after: z {min(zs):.4f} .. {max(zs):.4f} (top includes the bun), {len(me.polygons)} faces")
src.name = "High"

# weld the glTF seam splits so decimation sees one surface
bm = bmesh.new()
bm.from_mesh(me)
n0 = len(bm.verts)
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=WELD)
bm.to_mesh(me)
bm.free()
print(f"high: welded {n0} -> {len(me.vertices)} verts")

low = src.copy()
low.data = me.copy()
low.name = "Body"
bpy.context.scene.collection.objects.link(low)
tris = sum(len(p.vertices) - 2 for p in low.data.polygons)
dec = low.modifiers.new("dec", "DECIMATE")
dec.ratio = TARGET / tris
bpy.context.view_layer.objects.active = low
for o in bpy.data.objects:
    o.select_set(o == low)
bpy.ops.object.modifier_apply(modifier="dec")
tris_low = sum(len(p.vertices) - 2 for p in low.data.polygons)
bm = bmesh.new()
bm.from_mesh(low.data)
loose = 0
seen = set()
bm.verts.ensure_lookup_table()
parts = []
for v in bm.verts:
    if v.index in seen:
        continue
    stack, n = [v], 0
    seen.add(v.index)
    while stack:
        c = stack.pop()
        n += 1
        for e in c.link_edges:
            w = e.other_vert(c)
            if w.index not in seen:
                seen.add(w.index)
                stack.append(w)
    parts.append(n)
nonman = sum(1 for e in bm.edges if not e.is_manifold)
bm.free()
print(
    f"low: {tris_low} tris (ratio {dec.ratio:.4f}), {len(low.data.vertices)} verts, "
    f"{len(parts)} parts {sorted(parts, reverse=True)[:6]}, {nonman} non-manifold edges"
)

# Decimation leaves triangles too large for Meshy's curved UV islands to interpolate linearly
# (the tee's slogan warps), so the base colour is re-baked from the original surface onto the
# low mesh through the low mesh's own (inherited) UVs. The original's material is turned into
# pure emission of its base colour so an EMIT bake copies the albedo with no lighting.
hmat = src.data.materials[0]
hnt = hmat.node_tree
hbsdf = next(n for n in hnt.nodes if n.type == "BSDF_PRINCIPLED")
col_link = next(l for l in hnt.links if l.to_socket == hbsdf.inputs["Base Color"])
emit = hnt.nodes.new("ShaderNodeEmission")
hnt.links.new(col_link.from_socket, emit.inputs["Color"])
hout = next(n for n in hnt.nodes if n.type == "OUTPUT_MATERIAL")
hnt.links.new(emit.outputs["Emission"], hout.inputs["Surface"])

lmat = hmat.copy()
lmat.name = "MW_Meshy"
low.data.materials.clear()
low.data.materials.append(lmat)
nt = lmat.node_tree
for n in [n for n in nt.nodes if n.type == "EMISSION"]:
    nt.nodes.remove(n)
bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
lout = next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")
nt.links.new(bsdf.outputs["BSDF"], lout.inputs["Surface"])
cimg = bpy.data.images.new("base_color", BAKE["size_px"], BAKE["size_px"], alpha=False)
ctex = nt.nodes.new("ShaderNodeTexImage")
ctex.image = cimg
nt.nodes.active = ctex

scene = bpy.context.scene
scene.render.engine = "CYCLES"
bake_device = "CPU"
candidates = ["METAL", "CUDA", "OPTIX", "HIP"] if BAKE["device"] == "auto" else [BAKE["device"]]
prefs = bpy.context.preferences.addons["cycles"].preferences
for candidate in candidates:
    if candidate == "CPU":
        break
    try:
        scene.cycles.device = "GPU"
        prefs.compute_device_type = candidate
        prefs.get_devices()
        for d in prefs.devices:
            d.use = True
        bake_device = candidate
        break
    except Exception as e:  # noqa: BLE001 — D+ printed a failed GPU setup and continued
        print("GPU setup:", candidate, e)
if bake_device == "CPU":
    scene.cycles.device = "CPU"
print(f"bake device: {bake_device}")
scene.cycles.samples = BAKE["samples"]
bake = scene.render.bake
bake.use_selected_to_active = True
bake.use_cage = False
bake.cage_extrusion = BAKE["extrusion_m"]
bake.max_ray_distance = BAKE["ray_m"]
for o in bpy.data.objects:
    o.select_set(o in (src, low))
bpy.context.view_layer.objects.active = low
bpy.ops.object.bake(type="EMIT", margin=BAKE["margin_px"])
cimg.filepath_raw = os.path.abspath(os.path.join(OUT, "base_color.png"))
cimg.file_format = "PNG"
cimg.save()
cimg.pack()
old = next(l for l in nt.links if l.to_socket == bsdf.inputs["Base Color"])
nt.links.remove(old)
nt.links.new(ctex.outputs["Color"], bsdf.inputs["Base Color"])
print(
    "baked base colour ->",
    cimg.filepath_raw,
    f"(extrusion {bake.cage_extrusion}, ray {bake.max_ray_distance}); no normal map",
)

bpy.data.objects.remove(src, do_unlink=True)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(OUT, "gameready.blend"))
print("saved", os.path.join(OUT, "gameready.blend"))

common.write_result(
    "gameready",
    {
        "scale": s,
        "sole_raw": sole,
        "cx": cx,
        "cy": cy,
        "triangles": tris_low,
        "vertices": len(low.data.vertices),
        "parts": sorted(parts, reverse=True)[:6],
        "non_manifold_edges": nonman,
        "bake_device": bake_device,
        "decimate_ratio": dec.ratio,
    },
)
