# SPDX-License-Identifier: GPL-2.0-or-later
"""Stage 9, the fleck repair: port of `fix_flecks.py` (D+ section 8.1), parameterized.

Run by the driver as: blender --background --factory-startup <weights_pack.blend or weights_fingers.blend>
--python-exit-code 1 --python flecks.py -- params.json. Driven only when fit.flecks.enabled is true (on by default
for new characters, off for Vera: F-B1-4). Writes flecks.blend and flecks.json; the base colour is saved in
out_dir as base_color.png (the same name the stage-6 bake used, so the export finds it).

Statement for statement the D+ code, except the three numbers of fit.flecks (z_min_m, dark, lighter) and the
Meshy base-colour texture, which are read from params.json.
"""

import bpy
import numpy as np
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

p = common.params()
TEX = p["meshy_texture"]  # the download's base colour (D+ `meshy_tex.png`)
OUT_DIR = p["out_dir"]
Z_MIN = p["z_min_m"]  # the eye line (rig_meshy.py's pick) plus a margin
DARK = p["dark"]
LIGHTER = p["lighter"]

body = bpy.data.objects["Body"]
me = body.data
mat = me.materials[0]
# the image feeding Base Color: the material, copied from Meshy's, also still holds Meshy's own
# texture node (unlinked), and picking the first image node read that one
bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
img = next(l.from_node.image for l in mat.node_tree.links if l.to_socket == bsdf.inputs["Base Color"])
print("base colour image:", img.name)
W, H = img.size
baked = np.array(img.pixels[:], dtype=np.float32).reshape(H, W, 4)
orig_img = bpy.data.images.load(TEX)
orig = np.array(orig_img.pixels[:], dtype=np.float32).reshape(H, W, 4)

# UV-space mask of the faces wholly above Z_MIN
me.calc_loop_triangles()
uv = me.uv_layers.active.data
co_z = np.array([v.co.z for v in me.vertices])
mask = np.zeros((H, W), dtype=bool)
n_tri = 0
for t in me.loop_triangles:
    if min(co_z[i] for i in t.vertices) < Z_MIN:
        continue
    n_tri += 1
    pts = np.array([uv[li].uv[:] for li in t.loops]) * [W, H]
    x0, y0 = np.floor(pts.min(0)).astype(int) - 1
    x1, y1 = np.ceil(pts.max(0)).astype(int) + 1
    x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, W - 1), min(y1, H - 1)
    if x1 < x0 or y1 < y0:
        continue
    gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
    (ax, ay), (bx, by), (cx, cy) = pts
    d = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
    if abs(d) < 1e-9:
        continue
    l1 = ((by - cy) * (gx - cx) + (cx - bx) * (gy - cy)) / d
    l2 = ((cy - ay) * (gx - cx) + (ax - cx) * (gy - cy)) / d
    l3 = 1 - l1 - l2
    eps = 1.5 / max(abs(d) ** 0.5, 1.0)  # about a texel of slack at the edges
    inside = (l1 > -eps) & (l2 > -eps) & (l3 > -eps)
    mask[y0 : y1 + 1, x0 : x1 + 1] |= inside

# Triangles that straddle a UV seam. The decimation welded seam splits and collapsed edges
# across them, so a few low triangles take their corners' UVs from different islands: their UV
# edges run across the atlas and they render a streak of whatever lies between (skin, the
# zip...). Found by UV edge length against 3D edge length at the median texel density; each one
# is given a single UV (the corner nearest the island the others agree on: corner 0's), so it
# samples one texel of its own surface.
uvs = np.array([[uv[li].uv[:] for li in t.loops] for t in me.loop_triangles]) * [W, H]
pos = np.array([[me.vertices[i].co[:] for i in t.vertices] for t in me.loop_triangles])
uv_e = np.linalg.norm(uvs - np.roll(uvs, 1, axis=1), axis=2).max(1)
xyz_e = np.linalg.norm(pos - np.roll(pos, 1, axis=1), axis=2).max(1)
dens = np.median(uv_e / np.maximum(xyz_e, 1e-6))
# method constants of the straddle test (fix_flecks.py:76): 6 times the median density, and 12 texels
bad = (uv_e > 6.0 * dens * xyz_e) & (uv_e > 12.0)
print(f"median density {dens:.0f} texels/m; straddling triangles {bad.sum()} of {len(bad)}")
for k in np.nonzero(bad)[0]:
    t = me.loop_triangles[int(k)]
    u0 = uv[t.loops[0]].uv.copy()
    for li in t.loops:
        uv[li].uv = u0
me.update()

lum = lambda c: c[..., 0] * 0.2126 + c[..., 1] * 0.7152 + c[..., 2] * 0.0722
lb, lo = lum(baked), lum(orig)
fix = mask & (lo < DARK) & (lb - lo > LIGHTER)
print(
    f"head faces above z {Z_MIN}: {n_tri}; mask {mask.sum()} texels; replaced {fix.sum()} "
    f"({100.0 * fix.sum() / max(mask.sum(), 1):.2f} % of the mask)"
)
out = baked.copy()
out[fix, :3] = orig[fix, :3]
img.pixels[:] = out.ravel()
img.update()
img.filepath_raw = os.path.join(OUT_DIR, "base_color.png")
img.file_format = "PNG"
img.save()
img.pack()

common.write_result(
    "flecks",
    {
        "median_density_texels_per_m": float(dens),
        "straddling_triangles": int(bad.sum()),
        "faces_above_z_min": n_tri,
        "mask_texels": int(mask.sum()),
        "replaced_texels": int(fix.sum()),
    },
)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(OUT_DIR, "flecks.blend"))
