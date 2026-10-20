# SPDX-License-Identifier: GPL-2.0-or-later
"""Cut mesh A by mesh B: remove the part of A that lies inside B (step-36 asset follow-up).

Run by `tools/fix_props.py` (through `prop_cut.py`) for a `cut` fix, never by hand:

    blender --background --factory-startup --python cut_part.py -- in.json out.json

`in.json` holds two triangle meshes in one shared frame (the prop's glTF scene space, metres; the
caller moves them there and back, so no axis convention matters here):

    {"a": {"positions": [x, y, z, ...], "normals": [...], "uvs": [u, v, ...], "indices": [...]},
     "b": {"positions": [...], "indices": [...]}, "clearance_mm": 0.3}

How, step by step.

1. B is inflated by `clearance_mm` along its vertex normals (vertices that share a position share
   the normal, so UV seams do not tear). The cut then ends that far outside B instead of exactly on
   it: a 0.3 mm gap, far below what the eye resolves under the part that covers it, and the parts no
   longer touch, so a contact check sees no contact rather than a grazing one.
2. The inflated B is copied into A's mesh, and Blender's exact intersect (`mesh.intersect`,
   Selected/Unselected, separate mode CUT) splits every face of A along the curve where it crosses
   B, interpolating A's UVs and normals onto the new corners. Each face of A is then wholly inside or
   wholly outside B.
3. A face is inside when its centre's generalized winding number against B is above 0.5 (Jacobson
   et al. 2013). That needs B to enclose a volume, so the cutter is always the closed part: a bulb
   holder, a post, a stem, a pot wall. The faces inside are deleted, and so are B's.

`out.json` is A in the same shape as its input, triangulated, one vertex per distinct (position,
normal, uv). The script prints one `cut_part:` line saying how much it cut.
"""

import json
import sys

import bmesh
import bpy
import numpy as np

INSIDE = (
    0.5  # the winding-number threshold between outside (about 0) and inside (about 1)
)
CHUNK = 256  # query points per block of the vectorised winding-number sum


def winding(tri: np.ndarray, query: np.ndarray) -> np.ndarray:
    """Generalized winding number of the triangle soup `tri` (m, 3, 3) at `query` (n, 3), by the
    Van Oosterom-Strackee solid angle of each triangle, summed over 4 pi."""
    out = np.zeros(len(query))
    for start in range(0, len(query), CHUNK):
        q = query[start : start + CHUNK, None, None, :]
        r = tri[None, :, :, :] - q  # (n, m, 3, 3)
        a, b, c = r[:, :, 0], r[:, :, 1], r[:, :, 2]
        la, lb, lc = (np.linalg.norm(v, axis=-1) for v in (a, b, c))
        det = np.einsum("nmk,nmk->nm", a, np.cross(b, c))
        dot = (
            la * lb * lc
            + np.einsum("nmk,nmk->nm", a, b) * lc
            + np.einsum("nmk,nmk->nm", b, c) * la
            + np.einsum("nmk,nmk->nm", c, a) * lb
        )
        out[start : start + CHUNK] = (
            np.arctan2(det, dot).sum(axis=1) * 2.0 / (4.0 * np.pi)
        )
    return out


def inflate(pos: np.ndarray, faces: np.ndarray, by: float) -> np.ndarray:
    """Move each vertex `by` metres along the area-weighted normal of every face around its position."""
    tri = pos[faces]
    fn = np.cross(
        tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0]
    )  # length = twice the area
    _, group = np.unique(np.round(pos, 7), axis=0, return_inverse=True)
    group = group.reshape(-1)
    acc = np.zeros((group.max() + 1, 3))
    for k in range(3):
        np.add.at(acc, group[faces[:, k]], fn)
    length = np.linalg.norm(acc, axis=1, keepdims=True)
    normal = np.divide(acc, length, out=np.zeros_like(acc), where=length > 0)
    return pos + normal[group] * by


def build(data: dict, pb: np.ndarray) -> bpy.types.Object:
    a, b = data["a"], data["b"]
    pa = np.asarray(a["positions"], float).reshape(-1, 3)
    fa = np.asarray(a["indices"], int).reshape(-1, 3)
    fb = np.asarray(b["indices"], int).reshape(-1, 3) + len(pa)
    mesh = bpy.data.meshes.new("cut")
    mesh.from_pydata(np.vstack([pa, pb]).tolist(), [], np.vstack([fa, fb]).tolist())
    mesh.update()
    cutter = mesh.attributes.new("mw_cutter", "INT", "FACE")
    cutter.data.foreach_set("value", [0] * len(fa) + [1] * len(fb))
    uv = mesh.uv_layers.new(name="uv")
    nrm = mesh.attributes.new("mw_normal", "FLOAT_VECTOR", "CORNER")
    na = np.asarray(a["normals"], float).reshape(-1, 3)
    ua = np.asarray(a["uvs"], float).reshape(-1, 2)
    loop_vert = np.zeros(len(mesh.loops), int)
    mesh.loops.foreach_get("vertex_index", loop_vert)
    in_a = loop_vert < len(pa)
    vert_a = np.where(in_a, loop_vert, 0)
    nrm.data.foreach_set("vector", np.where(in_a[:, None], na[vert_a], 0.0).ravel())
    uv.data.foreach_set("uv", np.where(in_a[:, None], ua[vert_a], 0.0).ravel())
    obj = bpy.data.objects.new("cut", mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def intersect(obj: bpy.types.Object) -> int:
    """Split A along B (Blender's exact intersect); returns how many faces the split added."""
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    before = len(obj.data.polygons)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_mode(type="FACE")
    bm = bmesh.from_edit_mesh(obj.data)
    layer = bm.faces.layers.int["mw_cutter"]
    for seq in (bm.verts, bm.edges, bm.faces):
        for item in seq:
            item.select = False
    for f in bm.faces:
        if f[layer] == 1:
            f.select = True  # selects its edges and vertices too
    bm.select_flush_mode()
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.mesh.intersect(mode="SELECT_UNSELECT", separate_mode="CUT", solver="EXACT")
    bpy.ops.object.mode_set(mode="OBJECT")
    return len(obj.data.polygons) - before


def remove_inside(
    obj: bpy.types.Object, pb: np.ndarray, fb: np.ndarray
) -> tuple[int, int]:
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    layer = bm.faces.layers.int["mw_cutter"]
    # Triangles first: the split leaves n-gons, and an n-gon's median centre can lie outside it.
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    a_faces = [f for f in bm.faces if f[layer] == 0]
    centres = np.array([list(f.calc_center_median()) for f in a_faces]).reshape(-1, 3)
    lo, hi = pb.min(0), pb.max(0)
    near = np.flatnonzero(np.all((centres >= lo) & (centres <= hi), axis=1))
    inside = np.zeros(len(a_faces), bool)
    inside[near] = winding(pb[fb], centres[near]) > INSIDE
    doomed = [f for f, i in zip(a_faces, inside) if i] + [
        f for f in bm.faces if f[layer] == 1
    ]
    bmesh.ops.delete(bm, geom=doomed, context="FACES")
    bm.to_mesh(obj.data)
    bm.free()
    return int(inside.sum()), len(obj.data.polygons)


def export(obj: bpy.types.Object) -> dict:
    mesh = obj.data
    nrm = mesh.attributes["mw_normal"].data
    uv = mesh.uv_layers["uv"].data
    index: dict[tuple, int] = {}
    positions: list[float] = []
    normals: list[float] = []
    uvs: list[float] = []
    indices: list[int] = []
    for poly in mesh.polygons:
        for li in poly.loop_indices:
            p = tuple(mesh.vertices[mesh.loops[li].vertex_index].co)
            n = np.asarray(nrm[li].vector, float)
            length = float(np.linalg.norm(n))
            n = tuple(n / length) if length > 1e-9 else tuple(poly.normal)
            t = tuple(uv[li].uv)
            key = tuple(round(x, 7) for x in (*p, *n, *t))
            if key not in index:
                index[key] = len(index)
                positions.extend(p)
                normals.extend(n)
                uvs.extend(t)
            indices.append(index[key])
    return {"positions": positions, "normals": normals, "uvs": uvs, "indices": indices}


def main() -> None:
    src, dst = sys.argv[sys.argv.index("--") + 1 :][:2]
    with open(src, encoding="utf-8") as f:
        data = json.load(f)
    fb = np.asarray(data["b"]["indices"], int).reshape(-1, 3)
    pb = inflate(
        np.asarray(data["b"]["positions"], float).reshape(-1, 3),
        fb,
        data["clearance_mm"] / 1000.0,
    )
    obj = build(data, pb)
    added = intersect(obj)
    removed, kept = remove_inside(obj, pb, fb)
    with open(dst, "w", encoding="utf-8") as f:
        json.dump(export(obj), f)
    print(
        f"cut_part: the split added {added} faces; removed {removed} faces of A inside B, kept {kept}"
    )


main()
