# SPDX-License-Identifier: GPL-3.0-or-later
"""R-4: register the hidden `hm08` fitting template onto a person's body (pr-s32p-wardrobe-pilot.md §4.3).

Steps (design R-4):
1. build the MPFB2 template from the subject's macros and bisect its `height` slider until its sole-to-crown equals
   the recorded stature within 5 mm (R-S32-6; the stature comes from the recipe, never from the candidate);
2. add MPFB2's built-in `mixamo` rig (its 52 names equal ours, P-2);
3. pose that rig onto the body's 52 fitted joints: parents first, each bone's head moved to the body joint's head
   and its direction turned onto the body bone's direction (head to its main child's head); the template follows by
   MPFB2's own linear-blend skinning;
4. non-rigid surface fit of the template's body vertices: along the vertex normal, both ways, the nearest body hit
   within 50 mm whose normal agrees within 60°, else the nearest surface point within 30 mm under the same normal
   test; unfitted vertices take their neighbours' displacement; the displacement field is smoothed (3 Laplacian
   passes, λ 0.5); face, hands and feet move by half their displacement, so they follow the joint chain; helper
   vertices (MakeClothes' skirt and tights helpers among them) move with their nearest body vertex;
5. skin weights for the template from the body: per template vertex, the body's weights interpolated over the
   nearest body polygon, the four largest kept and normalised;
6. per body vertex, the closest template triangle (body polygons, triangulated) and its barycentric coordinates.

Params: {"body": <glb>, "macros": {...}, "stature_m": <float>, "out_dir": <dir>, "name": <subject>,
         "regions": <hm08_regions.json>}
Writes <name>_template.bin, <name>_template_weights.bin, <name>_corr.bin, register.json, template_fit.obj.
"""

import math
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bpy  # noqa: E402
import mpfbcommon as mc  # noqa: E402
from mathutils import Matrix, Quaternion, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402
from mathutils.interpolate import poly_3d_calc  # noqa: E402
from mathutils.kdtree import KDTree  # noqa: E402

RAY_M = 0.05
NEAREST_M = 0.03
NORMAL_DOT = 0.5  # cos 60°
SMOOTH_PASSES = 3
SMOOTH_LAMBDA = 0.5
FOLLOW_CHAIN = {"face": 0.5, "hands": 0.5, "feet": 0.5}
FILL_PASSES = 60
# The child whose head gives a bone's direction where a bone has several children.
MAIN_CHILD = {
    "Hips": "Spine",
    "Spine2": "Neck",
    "LeftHand": "LeftHandMiddle1",
    "RightHand": "RightHandMiddle1",
}


def import_body(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    new = [o for o in bpy.data.objects if o not in before]
    arm = next(o for o in new if o.type == "ARMATURE")
    meshes = [o for o in new if o.type == "MESH"]
    body = max(meshes, key=lambda o: len(o.data.vertices))
    return arm, body


def body_geometry(body):
    deps = bpy.context.evaluated_depsgraph_get()
    ev = body.evaluated_get(deps)
    mesh = ev.to_mesh()
    mw = body.matrix_world
    verts = [mw @ v.co for v in mesh.vertices]
    polys = [tuple(p.vertices) for p in mesh.polygons]
    ev.to_mesh_clear()
    names = {g.index: g.name for g in body.vertex_groups}
    weights = []
    for v in body.data.vertices:
        weights.append(
            {names[g.group]: g.weight for g in v.groups if g.weight > 0.0 and names[g.group].startswith(mc.M)}
        )
    return verts, polys, weights


def ordered_bones(rig):
    out = []

    def walk(bone):
        out.append(bone.name)
        for child in bone.children:
            walk(child)

    for bone in rig.data.bones:
        if bone.parent is None:
            walk(bone)
    return out


def pose_onto(rig, heads):
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    for eb in rig.data.edit_bones:
        eb.use_connect = False
    bpy.ops.object.mode_set(mode="OBJECT")
    mw = rig.matrix_world
    report = {}
    for name in ordered_bones(rig):
        if name not in heads:
            continue
        pb = rig.pose.bones[name]
        bone = rig.data.bones[name]
        short = name[len(mc.M) :]
        child = MAIN_CHILD.get(short)
        child_name = mc.M + child if child else (bone.children[0].name if len(bone.children) == 1 else None)
        current = mw @ pb.head
        rotation = Quaternion()
        if child_name and child_name in heads:
            have = (mw @ rig.pose.bones[child_name].head) - current
            want = heads[child_name] - heads[name]
            if have.length > 1e-6 and want.length > 1e-6:
                rotation = have.rotation_difference(want)
                report[name] = round(mc.angle_deg(have, want), 2)
        world = mw @ pb.matrix
        new = (
            Matrix.Translation(heads[name])
            @ rotation.to_matrix().to_4x4()
            @ Matrix.Translation(-current)
            @ world
        )
        pb.matrix = mw.inverted() @ new
        bpy.context.view_layer.update()
    return report


def forward(heads):
    """A body's facing in the horizontal plane, from its sides: facing = left × up (left = LeftUpLeg − RightUpLeg)."""
    left = heads[mc.M + "LeftUpLeg"] - heads[mc.M + "RightUpLeg"]
    v = left.cross(Vector((0.0, 0.0, 1.0)))
    v.z = 0.0
    return v.normalized()


def face_like(rig, heads):
    """Check that the template faces the way the body faces (both face -Y in Blender, +Z in glTF: MakeHuman's
    axes, in which a `.mhclo` offset is written). A body facing the other way is refused, not turned: a turned
    template would turn every garment offset with it. Returns the angle between the two facings, in degrees."""
    mw = rig.matrix_world
    mine = forward({b.name: mw @ b.head_local for b in rig.data.bones})
    theirs = forward(heads)
    if mine.dot(theirs) < 0.9:
        raise SystemExit(f"the body faces {tuple(theirs)}, the template {tuple(mine)}: refused")
    return round(math.degrees(mine.angle(theirs)), 2)


def neighbours(mesh, ids):
    keep = set(ids)
    adj = {i: set() for i in ids}
    for e in mesh.edges:
        a, b = e.vertices
        if a in keep and b in keep:
            adj[a].add(b)
            adj[b].add(a)
    return adj


def surface_fit(pos, nrm, body_ids, regions, region_names, bvh, adj):
    disp = {}
    for i in body_ids:
        p, n = pos[i], nrm[i]
        best = None
        for d in (n, -n):
            loc, normal, _index, dist = bvh.ray_cast(p, d, RAY_M)
            if loc is not None and normal.dot(n) >= NORMAL_DOT and (best is None or dist < best[1]):
                best = (loc, dist)
        if best is None:
            loc, normal, _index, dist = bvh.find_nearest(p, NEAREST_M)
            if loc is not None and normal.dot(n) >= NORMAL_DOT:
                best = (loc, dist)
        if best is not None:
            disp[i] = best[0] - p
    fitted = set(disp)
    for _ in range(FILL_PASSES):
        missing = [i for i in body_ids if i not in disp]
        if not missing:
            break
        for i in missing:
            known = [disp[j] for j in adj[i] if j in disp]
            if known:
                disp[i] = sum(known, Vector()) / len(known)
    for i in body_ids:
        disp.setdefault(i, Vector())
    for _ in range(SMOOTH_PASSES):
        new = {}
        for i in body_ids:
            ring = [disp[j] for j in adj[i]]
            avg = sum(ring, Vector()) / len(ring) if ring else disp[i]
            new[i] = disp[i] * (1 - SMOOTH_LAMBDA) + avg * SMOOTH_LAMBDA
        disp = new
    for i in body_ids:
        region = region_names[regions[i]] if regions[i] >= 0 else None
        disp[i] = disp[i] * FOLLOW_CHAIN.get(region, 1.0)
    return disp, fitted


def percentile(values, q):
    if not values:
        return None
    s = sorted(values)
    return s[min(len(s) - 1, int(q * (len(s) - 1) + 0.5))]


def main():
    p = mc.params()
    out = Path(p["out_dir"])
    out.mkdir(parents=True, exist_ok=True)
    name = p["name"]
    region_data = mc.json.loads(Path(p["regions"]).read_text(encoding="utf-8"))
    regions = region_data["vertex_region"]
    region_names = region_data["regions"]
    mc.clear_scene()
    arm, body = import_body(p["body"])
    heads = {b.name: arm.matrix_world @ b.head_local for b in arm.data.bones if b.name.startswith(mc.M)}
    bverts, bpolys, bweights = body_geometry(body)
    bvh = BVHTree.FromPolygons(bverts, bpolys)
    zs = [v.z for v in bverts]
    mc.log(
        "body",
        body.name,
        "vertices",
        len(bverts),
        "extent",
        round(max(zs) - min(zs), 4),
        "joints",
        len(heads),
    )

    basemesh, body_ids, calibration = mc.calibrate_height(p["macros"], float(p["stature_m"]))
    mc.log(
        "calibration",
        calibration["height_slider"],
        calibration["stature_m"],
        "target",
        calibration["target_m"],
    )
    HumanService, _ = mc.mpfb()
    rig = HumanService.add_builtin_rig(basemesh, "mixamo")
    rig_names = sorted(b.name for b in rig.data.bones)
    if sorted(heads) != rig_names:
        raise SystemExit(
            "the body's joints differ from the template rig's: " + str(set(heads) ^ set(rig_names))
        )
    facing = face_like(rig, heads)
    turns = pose_onto(rig, heads)
    pos, nrm = mc.evaluated_positions(basemesh)
    adj = neighbours(basemesh.data, body_ids)
    disp, fitted = surface_fit(pos, nrm, body_ids, regions, region_names, bvh, adj)
    final = list(pos)
    for i in body_ids:
        final[i] = pos[i] + disp[i]
    kd = KDTree(len(body_ids))
    for k, i in enumerate(body_ids):
        kd.insert(pos[i], k)
    kd.balance()
    body_set = set(body_ids)
    for i in range(len(final)):
        if i not in body_set:
            _co, k, _d = kd.find(pos[i])
            final[i] = pos[i] + disp[body_ids[k]]

    # Distances, template -> body and body -> template, per region.
    to_body = {r: [] for r in region_names}
    for i in body_ids:
        loc, _n, _ix, dist = bvh.find_nearest(final[i])
        if loc is not None and regions[i] >= 0:
            to_body[region_names[regions[i]]].append(dist)
    tris, tri_region = [], []
    for poly in basemesh.data.polygons:
        vs = list(poly.vertices)
        if not all(v in body_set for v in vs):
            continue
        for k in range(1, len(vs) - 1):
            tris.append((vs[0], vs[k], vs[k + 1]))
            tri_region.append(regions[vs[0]])
    tbvh = BVHTree.FromPolygons(final, tris)
    corr = []
    from_body = []
    for v in bverts:
        loc, _n, ti, dist = tbvh.find_nearest(v)
        a, b, c = tris[ti]
        w = poly_3d_calc([final[a], final[b], final[c]], loc)
        corr.append((a, b, c, w[0], w[1]))
        from_body.append(dist)

    # Weights from the body.
    joints = sorted({j for w in bweights for j in w})
    jindex = {j: k for k, j in enumerate(joints)}
    influences = []
    for i in range(len(final)):
        loc, _n, pi, _d = bvh.find_nearest(final[i])
        poly = bpolys[pi]
        w = poly_3d_calc([bverts[v] for v in poly], loc)
        acc = {}
        for v, wv in zip(poly, w):
            for j, wj in bweights[v].items():
                acc[j] = acc.get(j, 0.0) + wv * wj
        top = sorted(acc.items(), key=lambda kv: -kv[1])[:4]
        total = sum(x for _j, x in top) or 1.0
        row = [(jindex[j], x / total) for j, x in top]
        while len(row) < 4:
            row.append((0, 0.0))
        influences.append(row)

    # The template is written in the body mesh's own frame (glTF mesh space), the space its Skin binds use.
    to_local = body.matrix_world.inverted()
    t_bin = bytearray(b"MWGT" + struct.pack("<I", len(final)))
    for v in final:
        t_bin += struct.pack("<3f", *mc.to_gltf(to_local @ v))
    (out / f"{name}_template.bin").write_bytes(bytes(t_bin))
    w_bin = bytearray(b"MWGW" + struct.pack("<II", len(influences), len(joints)))
    for j in joints:
        e = j.encode("utf-8")
        w_bin += struct.pack("<B", len(e)) + e
    for row in influences:
        for j, x in row:
            w_bin += struct.pack("<Bf", j, x)
    (out / f"{name}_template_weights.bin").write_bytes(bytes(w_bin))
    c_bin = bytearray(b"MWGC" + struct.pack("<I", len(corr)))
    for a, b, c, b0, b1 in corr:
        c_bin += struct.pack("<3I2f", a, b, c, b0, b1)
    (out / f"{name}_corr.bin").write_bytes(bytes(c_bin))
    with open(out / "template_fit.obj", "w", encoding="utf-8") as fh:
        for v in final:
            g = mc.to_gltf(to_local @ v)
            fh.write(f"v {g[0]:.5f} {g[1]:.5f} {g[2]:.5f}\n")
        for a, b, c in tris:
            fh.write(f"f {a + 1} {b + 1} {c + 1}\n")

    table = {}
    for r, values in to_body.items():
        mm = [1000 * x for x in values]
        table[r] = {
            "n": len(mm),
            "mean_mm": round(sum(mm) / len(mm), 2) if mm else None,
            "p95_mm": round(percentile(mm, 0.95), 2) if mm else None,
        }
        mc.log(
            f"region {r:11s} n {len(mm):5d} template->body mean {table[r]['mean_mm']} mm p95 {table[r]['p95_mm']} mm"
        )
    fb = [1000 * x for x in from_body]
    result = {
        "name": name,
        "body": str(p["body"]),
        "calibration": calibration,
        "bone_turns_deg": turns,
        "facing_angle_deg": facing,
        "fitted_by_surface": len(fitted),
        "body_vertices_of_template": len(body_ids),
        "regions_template_to_body": table,
        "body_to_template_mean_mm": round(sum(fb) / len(fb), 2),
        "body_to_template_p95_mm": round(percentile(fb, 0.95), 2),
        "joints": joints,
    }
    mc.write_result(out / "register.json", result)
    mc.log(
        "fitted",
        len(fitted),
        "of",
        len(body_ids),
        "body->template mean",
        result["body_to_template_mean_mm"],
        "p95",
        result["body_to_template_p95_mm"],
    )


main()
