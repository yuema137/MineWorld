# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 10, the export: the one place every D+ export block became (PIPELINE.md §2.1, §8, §9).

Run by the driver as: blender --background --factory-startup <flecks.blend | weights_pack.blend> --python-exit-code 1
--python export.py -- params.json

It renames the character's material to the one the fit names, names the base-colour image `<id>_base_color`,
stores the provenance block the driver built (PIPELINE.md §8), and writes two outputs from the same .blend:
`<id>.glb` (the embedded PNG, the D+ export settings, for the parity comparison) and `<id>.gltf` with `<id>.bin`
and the separate base-colour PNG (`GLTF_SEPARATE`, the stored-once candidate, PIPELINE.md §9). Writes export.json
with each file's SHA-256.
"""

import bpy
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
CHAR_ID = p["character"]
OUT_DIR = p["out_dir"]
MATERIAL = p["material"]
PROVENANCE = p["provenance"]

body = bpy.data.objects["Body"]
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
mat = body.data.materials[0]
mat.name = MATERIAL
bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
base_image = next(l.from_node.image for l in mat.node_tree.links if l.to_socket == bsdf.inputs["Base Color"])
base_image.name = f"{CHAR_ID}_base_color"
common.set_provenance(PROVENANCE)
arm.name = "mixamo_vitruvian"
body.name = body.data.name = "Body"

for o in bpy.data.objects:
    o.select_set(o in (body, arm))
bpy.context.view_layer.objects.active = body

glb_path = os.path.join(OUT_DIR, f"{CHAR_ID}.glb")
bpy.ops.export_scene.gltf(filepath=glb_path, **common.EXPORT_SETTINGS)
print("wrote", glb_path, f"{os.path.getsize(glb_path) / 1e6:.2f} MB")

# Stored once (PIPELINE.md §9, ruling 2026-10-14 on AB1-11): the glTF exporter names a separate image after the
# basename of the image's file path (the bake's `base_color.png`), not after the datablock. The image is packed, so
# the exporter writes the packed bytes unchanged, and its path is renamed to `<id>_base_color.png`.
if base_image.packed_file is None:
    base_image.pack()
base_image.filepath_raw = f"//{CHAR_ID}_base_color.png"
gltf_path = os.path.join(OUT_DIR, f"{CHAR_ID}.gltf")
separate = dict(common.EXPORT_SETTINGS, export_format="GLTF_SEPARATE")
bpy.ops.export_scene.gltf(filepath=gltf_path, **separate)
print("wrote", gltf_path)

files = {}
for name in sorted(os.listdir(OUT_DIR)):
    if name.startswith(CHAR_ID):
        path = os.path.join(OUT_DIR, name)
        with open(path, "rb") as handle:
            files[name] = hashlib.sha256(handle.read()).hexdigest()
common.write_result(
    "export",
    {"material": MATERIAL, "base_image": base_image.name, "files": files},
)
