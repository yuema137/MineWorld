#!/usr/bin/env python3
"""Bake the CC0 Vitruvian character into one MineWorld-ready GLB.

Input is a clone of https://github.com/ibrews/VitruvianGodot, which is **not**
vendored into this repository (317 MB, and it carries Mixamo source clips that
`DEP-8` forbids us to commit). See `docs/CHARACTER_ASSET_AUDIT.md` for the
licence forensics behind what this script keeps and what it drops.

What it does, and why each step exists:

* **Drops the 6 animations** baked into `vitruvian_body.glb`
  (`HappyIdle, Idle, Sway, Turn, Walk, Wave`). They are Mixamo motion retargeted
  onto a CC0 rig; retargeting does not launder provenance, so they may not be
  committed. Locomotion comes from Quaternius instead.
* **Merges head, eyes and hair into the body's skin.** Upstream ships them as
  separate unskinned meshes attached in-engine with `BoneAttachment3D`. Rigid-
  binding them to the `mixamorig:Head` joint instead gives one mesh hierarchy on
  one `Skeleton3D`, which is what lets a single `BoneMap` retarget the whole
  character.
* **Strips what a town NPC does not need**: the 26 FACS morph targets (≈8 MB,
  and `ARC-4` scopes facial fidelity below the reference anyway), seven spare
  vertex-colour layers and three spare UV sets left over from CharMorph's UDIM
  workflow.
* **Decimates the hair.** Upstream's groom is 121k independent quads / 242k
  triangles — a look-dev asset. Quads are dropped on a spatial stride so the
  thinning is even rather than clumped.

Blender is deliberately not required: everything here is glTF surgery, because
the machine this was written on had no Blender and needing one would have made
the pipeline much more expensive to re-run.

    python3 character_bake.py --upstream ~/src/VitruvianGodot --out ../assets/characters/vitruvian
"""

from __future__ import annotations

import argparse
import json
import os
import struct
import subprocess
import sys

# glTF component types -> (struct code, byte size)
COMP = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
NCOMP = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


# --------------------------------------------------------------------------- read

class Glb:
    """A parsed .glb: the JSON chunk plus the binary chunk."""

    def __init__(self, path: str):
        raw = open(path, "rb").read()
        if raw[:4] != b"glTF":
            raise SystemExit(f"{path}: not a GLB")
        jlen = struct.unpack("<I", raw[12:16])[0]
        self.j = json.loads(raw[20:20 + jlen])
        # the BIN chunk follows the JSON chunk, both 4-byte aligned
        off = 20 + jlen
        self.bin = b""
        while off < len(raw):
            clen, ctype = struct.unpack("<II", raw[off:off + 8])
            if ctype == 0x004E4942:
                self.bin = raw[off + 8:off + 8 + clen]
            off += 8 + clen
        self.path = path

    def accessor(self, idx: int) -> list:
        """Read accessor `idx` into a list of tuples (or scalars for SCALAR)."""
        a = self.j["accessors"][idx]
        code, size = COMP[a["componentType"]]
        n = NCOMP[a["type"]]
        count = a["count"]
        bv = self.j["bufferViews"][a["bufferView"]]
        base = bv.get("byteOffset", 0) + a.get("byteOffset", 0)
        stride = bv.get("byteStride") or size * n
        out = []
        for i in range(count):
            vals = struct.unpack_from("<" + code * n, self.bin, base + i * stride)
            out.append(vals[0] if n == 1 else vals)
        return out


# -------------------------------------------------------------------------- write

class Builder:
    """Accumulates accessors into a fresh glTF document with one buffer."""

    def __init__(self):
        self.blobs: list[bytes] = []
        self.length = 0
        self.doc = {
            "asset": {"version": "2.0", "generator": "MineWorld character_bake.py"},
            "accessors": [], "bufferViews": [], "meshes": [], "materials": [],
            "nodes": [], "skins": [], "scenes": [{"nodes": []}], "scene": 0,
        }

    def _view(self, data: bytes, target: int | None) -> int:
        pad = (-len(data)) % 4
        self.blobs.append(data + b"\0" * pad)
        bv = {"buffer": 0, "byteOffset": self.length, "byteLength": len(data)}
        if target:
            bv["target"] = target
        self.length += len(data) + pad
        self.doc["bufferViews"].append(bv)
        return len(self.doc["bufferViews"]) - 1

    def accessor(self, values, ctype: int, typ: str, target=None, normalized=False, minmax=False) -> int:
        code, size = COMP[ctype]
        n = NCOMP[typ]
        flat: list = []
        for v in values:
            if n == 1:
                flat.append(v)
            else:
                flat.extend(v)
        data = struct.pack("<" + code * len(flat), *flat)
        a = {"bufferView": self._view(data, target), "componentType": ctype,
             "count": len(values), "type": typ}
        if normalized:
            a["normalized"] = True
        if minmax and values:
            if n == 1:
                a["min"], a["max"] = [min(values)], [max(values)]
            else:
                a["min"] = [min(v[k] for v in values) for k in range(n)]
                a["max"] = [max(v[k] for v in values) for k in range(n)]
        self.doc["accessors"].append(a)
        return len(self.doc["accessors"]) - 1

    def material(self, name: str) -> int:
        self.doc["materials"].append({
            "name": name,
            "pbrMetallicRoughness": {"baseColorFactor": [0.8, 0.8, 0.8, 1.0],
                                     "metallicFactor": 0.0, "roughnessFactor": 0.6},
        })
        return len(self.doc["materials"]) - 1

    def save(self, path: str) -> int:
        self.doc["buffers"] = [{"byteLength": self.length}]
        js = json.dumps(self.doc, separators=(",", ":")).encode()
        js += b" " * ((-len(js)) % 4)
        binc = b"".join(self.blobs)
        total = 12 + 8 + len(js) + 8 + len(binc)
        with open(path, "wb") as f:
            f.write(b"glTF" + struct.pack("<II", 2, total))
            f.write(struct.pack("<II", len(js), 0x4E4F534A) + js)
            f.write(struct.pack("<II", len(binc), 0x004E4942) + binc)
        return total


# ----------------------------------------------------------------------- geometry

def mat_mul(a, b):
    return [sum(a[i * 4 + k] * b[k * 4 + j] for k in range(4)) for i in range(4) for j in range(4)]


def trs_matrix(node) -> list:
    """Column-major 4x4 for a glTF node's TRS, matching glTF's own convention."""
    if "matrix" in node:
        return list(node["matrix"])
    t = node.get("translation", [0, 0, 0])
    r = node.get("rotation", [0, 0, 0, 1])
    s = node.get("scale", [1, 1, 1])
    x, y, z, w = r
    rot = [
        1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w), 0,
        2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w), 0,
        2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y), 0,
        0, 0, 0, 1,
    ]
    for c in range(3):
        for r_ in range(3):
            rot[c * 4 + r_] *= s[c]
    rot[12], rot[13], rot[14] = t
    return rot


def xform_point(m, p):
    x, y, z = p
    return (m[0] * x + m[4] * y + m[8] * z + m[12],
            m[1] * x + m[5] * y + m[9] * z + m[13],
            m[2] * x + m[6] * y + m[10] * z + m[14])


def xform_dir(m, v):
    x, y, z = v[0], v[1], v[2]
    return (m[0] * x + m[4] * y + m[8] * z,
            m[1] * x + m[5] * y + m[9] * z,
            m[2] * x + m[6] * y + m[10] * z)


def world_matrix(glb: Glb, node_idx: int) -> list:
    """World matrix of a node, walking up the (single-rooted) parent chain."""
    parent = {}
    for i, n in enumerate(glb.j["nodes"]):
        for c in n.get("children", []):
            parent[c] = i
    chain = []
    i = node_idx
    while i is not None:
        chain.append(i)
        i = parent.get(i)
    m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]
    for i in reversed(chain):
        m = mat_mul(m, trs_matrix(glb.j["nodes"][i]))
    return m


# --------------------------------------------------------------------------- bake

# Vertex attributes worth carrying. CharMorph leaves COLOR_1..7 and TEXCOORD_1..3
# on the mesh from its UDIM workflow; nothing downstream of here reads them.
# TANGENT is dropped too -- the Godot importer has `meshes/ensure_tangents=true`
# and regenerates it from normals and UVs, which costs nothing at runtime and
# takes 16 bytes per vertex off a file that has to live in the repository.
KEEP_ATTRS = ("POSITION", "NORMAL", "TEXCOORD_0", "JOINTS_0", "WEIGHTS_0")


def copy_primitive(src: Glb, out: Builder, prim: dict, matmap: dict,
                   world=None, rigid_joint=None, keep=KEEP_ATTRS, quad_stride=1,
                   offset=None, min_y=None):
    """Copy one primitive into `out`, optionally baking a transform, rigid-binding
    it to a single joint, dropping quads on a stride, translating it (`offset`,
    used to lift the iris out of the sclera it sits inside), and discarding
    triangles whose centre falls below `min_y` (used to crop the groom to a
    short cut)."""
    attrs = {}
    src_attrs = {k: v for k, v in prim["attributes"].items() if k in keep}
    pos = src.accessor(src_attrs["POSITION"])
    idx = src.accessor(prim["indices"]) if "indices" in prim else list(range(len(pos)))

    # Hair decimation: the groom is independent quads (2 tris), so dropping whole
    # quads keeps every remaining card intact. Stride rather than random, so the
    # thinning is spatially even.
    if quad_stride > 1:
        tris = [idx[i:i + 3] for i in range(0, len(idx), 3)]
        quads = [tris[i:i + 2] for i in range(0, len(tris), 2)]
        quads = quads[::quad_stride]
        idx = [v for q in quads for t in q for v in t]

    # Crop: the CC0 groom is shoulder length and the reference character's hair
    # is a short tousled crop. Whole quads are dropped by the height of their
    # centre, so every surviving card stays intact and the crown keeps its
    # volume -- cutting through cards would leave them ending in hard edges.
    if min_y is not None:
        kept = []
        for i in range(0, len(idx), 3):
            t = idx[i:i + 3]
            if sum(pos[v][1] for v in t) / 3.0 >= min_y:
                kept.extend(t)
        idx = kept

    used = sorted(set(idx))
    remap = {v: i for i, v in enumerate(used)}
    idx = [remap[v] for v in idx]

    for name, acc in src_attrs.items():
        vals = src.accessor(acc)
        vals = [vals[i] for i in used]
        a = src.j["accessors"][acc]
        if offset is not None and name == "POSITION":
            vals = [tuple(v[i] + offset[i] for i in range(3)) for v in vals]
        if world is not None:
            if name == "POSITION":
                vals = [xform_point(world, v) for v in vals]
            elif name == "NORMAL":
                vals = [xform_dir(world, v) for v in vals]
            elif name == "TANGENT":
                vals = [tuple(xform_dir(world, v)) + (v[3],) for v in vals]
        if name in ("JOINTS_0", "WEIGHTS_0") and rigid_joint is not None:
            continue
        attrs[name] = out.accessor(vals, a["componentType"], a["type"],
                                   target=34962, normalized=a.get("normalized", False),
                                   minmax=(name == "POSITION"))

    if rigid_joint is not None:
        n = len(used)
        attrs["JOINTS_0"] = out.accessor([(rigid_joint, 0, 0, 0)] * n, 5121, "VEC4", target=34962)
        attrs["WEIGHTS_0"] = out.accessor([(255, 0, 0, 0)] * n, 5121, "VEC4",
                                          target=34962, normalized=True)

    ctype = 5125 if len(used) > 65535 else 5123
    out_prim = {"attributes": attrs,
                "indices": out.accessor(idx, ctype, "SCALAR", target=34963)}
    name = src.j["materials"][prim["material"]]["name"] if "material" in prim else "Default"
    if name not in matmap:
        matmap[name] = out.material(name)
    out_prim["material"] = matmap[name]
    return out_prim, len(used), len(idx) // 3


# Arm joints the sleeves come from. Hands and fingers are excluded, which is
# what puts the cuff at the wrist.
SLEEVE_JOINTS = {
    "mixamorig:LeftShoulder", "mixamorig:LeftArm", "mixamorig:LeftForeArm",
    "mixamorig:RightShoulder", "mixamorig:RightArm", "mixamorig:RightForeArm",
}


def make_hoodie(src: Glb, out: Builder, sources: list, matmap: dict, joint_names: list,
                thickness: float, open_half_width: float):
    """Derive an open zip hoodie by offsetting existing garment and arm surfaces
    along their normals.

    There is no hoodie in the CC0 asset and no Blender on this machine to model
    one. What the asset does have is a correctly weighted t-shirt and a pair of
    bare arms, and a shell offset from those is a garment that fits, drapes with
    the character and needs no new skin weights because it inherits theirs.

    It has to come from two surfaces, which is the part that is not obvious:
    the body mesh's torso does not exist. Upstream occlusion-deletes the skin
    hidden under the clothing, so a shell taken from the body alone is a pair of
    sleeves with nothing between them. The torso comes from the t-shirt and the
    sleeves from the arms, offset by the same amount so they meet.

    The front opens by dropping the triangles down the centre line, which is
    what makes it read as an unzipped jacket over the tee rather than a second
    skin.

    The limit, stated plainly: a close-fitting shell, not tailoring. No hood, no
    zip teeth, no pockets, no slack. It reads as a red open jacket at
    conversation distance and would not survive a close-up.
    """
    P, N, U, J, W, tri = [], [], [], [], [], []
    aj = aw = None
    for prim, joint_filter in sources:
        a = prim["attributes"]
        pos = src.accessor(a["POSITION"])
        nrm = src.accessor(a["NORMAL"])
        jts = src.accessor(a["JOINTS_0"])
        wts = src.accessor(a["WEIGHTS_0"])
        uvs = src.accessor(a["TEXCOORD_0"])
        idx = src.accessor(prim["indices"])
        aj = src.j["accessors"][a["JOINTS_0"]]
        aw = src.j["accessors"][a["WEIGHTS_0"]]

        ok = None
        if joint_filter is not None:
            ok = {i for i, n in enumerate(joint_names) if n in joint_filter}
        dom = []
        for v in range(len(pos)):
            w = wts[v]
            j = jts[v]
            dom.append(j[max(range(4), key=lambda k: w[k])])

        kept = []
        for i in range(0, len(idx), 3):
            t = idx[i:i + 3]
            if ok is not None and not all(dom[v] in ok for v in t):
                continue
            cx = sum(pos[v][0] for v in t) / 3.0
            cz = sum(pos[v][2] for v in t) / 3.0
            if abs(cx) < open_half_width and cz > 0.0:
                continue  # the open front
            kept.extend(t)
        if not kept:
            continue
        used = sorted(set(kept))
        base = len(P)
        remap = {v: base + i for i, v in enumerate(used)}
        tri.extend(remap[v] for v in kept)
        P.extend(tuple(pos[v][k] + nrm[v][k] * thickness for k in range(3)) for v in used)
        N.extend(nrm[v] for v in used)
        U.extend(uvs[v] for v in used)
        J.extend(jts[v] for v in used)
        W.extend(wts[v] for v in used)

    if not tri:
        return None, 0, 0
    attrs = {
        "POSITION": out.accessor(P, 5126, "VEC3", target=34962, minmax=True),
        "NORMAL": out.accessor(N, 5126, "VEC3", target=34962),
        "TEXCOORD_0": out.accessor(U, 5126, "VEC2", target=34962),
        "JOINTS_0": out.accessor(J, aj["componentType"], "VEC4", target=34962),
        "WEIGHTS_0": out.accessor(W, aw["componentType"], "VEC4", target=34962,
                                  normalized=aw.get("normalized", False)),
    }
    ctype = 5125 if len(P) > 65535 else 5123
    if "VitHoodie" not in matmap:
        matmap["VitHoodie"] = out.material("VitHoodie")
    return ({"attributes": attrs, "indices": out.accessor(tri, ctype, "SCALAR", target=34963),
             "material": matmap["VitHoodie"]}, len(P), len(tri) // 3)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--upstream", required=True, help="path to a VitruvianGodot clone")
    ap.add_argument("--out", required=True, help="output asset directory")
    ap.add_argument("--hair-stride", type=int, default=3,
                    help="keep 1 hair quad in N (default 3)")
    ap.add_argument("--hair-crop", type=float, default=None,
                    help="drop hair below this world Y, in metres (e.g. 1.62 for a "
                         "short crop; the groom runs 1.477..1.761)")
    ap.add_argument("--hoodie", action="store_true",
                    help="derive an open zip hoodie shell from the body mesh")
    ap.add_argument("--hoodie-thickness", type=float, default=0.030)
    ap.add_argument("--skip-textures", action="store_true")
    args = ap.parse_args()

    gp = os.path.join(args.upstream, "godot_project")
    if not os.path.isdir(gp):
        raise SystemExit(f"no godot_project/ under {args.upstream}")
    os.makedirs(args.out, exist_ok=True)

    body = Glb(os.path.join(gp, "vitruvian_body.glb"))
    head = Glb(os.path.join(gp, "vitruvian_head.glb"))
    hair = Glb(os.path.join(gp, "vitruvian_hair_rigged.glb"))

    out = Builder()
    matmap: dict[str, int] = {}

    # --- skeleton -----------------------------------------------------------
    # Copy the 52 CC0 CharMorph joints verbatim, keeping mixamorig: names. Godot's
    # BoneMap renames them to SkeletonProfileHumanoid at import; renaming them here
    # would hide the mapping in a binary instead of stating it in a committed .tres.
    skin = body.j["skins"][0]
    joints = skin["joints"]
    old2new = {}
    for new_i, old_i in enumerate(joints):
        n = body.j["nodes"][old_i]
        out.doc["nodes"].append({k: v for k, v in n.items()
                                 if k in ("name", "translation", "rotation", "scale")})
        old2new[old_i] = new_i
    for new_i, old_i in enumerate(joints):
        kids = [old2new[c] for c in body.j["nodes"][old_i].get("children", []) if c in old2new]
        if kids:
            out.doc["nodes"][new_i]["children"] = kids

    ibm = body.accessor(skin["inverseBindMatrices"])
    out.doc["skins"].append({
        "name": "Vitruvian",
        "joints": list(range(len(joints))),
        "inverseBindMatrices": out.accessor(ibm, 5126, "MAT4"),
    })
    head_joint = next(i for i, j in enumerate(joints)
                      if body.j["nodes"][j]["name"] == "mixamorig:Head")

    stats = []

    def add_mesh(name, src, node_idx, rigid=None, keep=KEEP_ATTRS, stride=1, eye=False,
                 min_y=None):
        mesh = src.j["meshes"][src.j["nodes"][node_idx]["mesh"]]
        world = world_matrix(src, node_idx) if rigid is not None else None
        prims, nv, nt = [], 0, 0
        front = None
        if eye:
            # The eyeball is a sclera sphere with the iris modelled ~5 mm *inside*
            # it, plus two clear shells (VitEyeBack, VitCornea2) that upstream's
            # look-dev shader handles and a StandardMaterial3D cannot. Drop the
            # shells and push the iris just proud of the sclera along the model's
            # forward axis (+Z), or the eye renders as a blank white ball.
            front = src.j["accessors"][mesh["primitives"][0]["attributes"]["POSITION"]]["max"][2]
        for p in mesh["primitives"]:
            mname = src.j["materials"][p["material"]]["name"] if "material" in p else ""
            if eye and ("EyeBack" in mname or "Cornea" in mname):
                continue
            off = None
            if eye and "Iris" in mname:
                back = src.j["accessors"][p["attributes"]["POSITION"]]["min"][2]
                off = (0.0, 0.0, front - back + 0.0006)
            pr, v, t = copy_primitive(src, out, p, matmap, world=world,
                                      rigid_joint=rigid, keep=keep, quad_stride=stride,
                                      offset=off, min_y=min_y)
            prims.append(pr)
            nv += v
            nt += t
        out.doc["meshes"].append({"name": name, "primitives": prims})
        out.doc["nodes"].append({"name": name, "mesh": len(out.doc["meshes"]) - 1, "skin": 0})
        stats.append((name, nv, nt))
        return len(out.doc["nodes"]) - 1

    mesh_nodes = []
    # --- body, already skinned to all 52 joints ------------------------------
    for node_i, nm in ((i, n["name"]) for i, n in enumerate(body.j["nodes"]) if "mesh" in n):
        mesh_nodes.append(add_mesh({"cm_vitruvian": "Body"}.get(nm, nm), body, node_i))

    if args.hoodie:
        jnames = [body.j["nodes"][j]["name"] for j in joints]

        def _prim(node_name, k=0):
            ni = next(i for i, n in enumerate(body.j["nodes"]) if n.get("name") == node_name)
            return body.j["meshes"][body.j["nodes"][ni]["mesh"]]["primitives"][k]

        hp, hv, ht = make_hoodie(body, out,
                                 [(_prim("Shirt"), None),
                                  (_prim("cm_vitruvian"), SLEEVE_JOINTS)],
                                 matmap, jnames, args.hoodie_thickness, 0.055)
        if hp:
            out.doc["meshes"].append({"name": "Hoodie", "primitives": [hp]})
            out.doc["nodes"].append({"name": "Hoodie",
                                     "mesh": len(out.doc["meshes"]) - 1, "skin": 0})
            mesh_nodes.append(len(out.doc["nodes"]) - 1)
            stats.append(("Hoodie", hv, ht))

    # --- head: strip the FACS morphs, rigid-bind to the Head joint ------------
    want_head = {"cm_vitruvian": "Head", "Eye_L_eyeball": "EyeL", "Eye_R_eyeball": "EyeR"}
    for node_i, n in enumerate(head.j["nodes"]):
        if n.get("name") in want_head and "mesh" in n:
            mesh_nodes.append(add_mesh(want_head[n["name"]], head, node_i, rigid=head_joint,
                                       eye=n["name"].startswith("Eye_")))

    # --- hair: decimate, drop the spring-bone skin, rigid-bind to Head --------
    for node_i, n in enumerate(hair.j["nodes"]):
        if n.get("name") == "VitHair" and "mesh" in n:
            mesh_nodes.append(add_mesh("Hair", hair, node_i, rigid=head_joint,
                                       keep=("POSITION", "NORMAL", "TEXCOORD_0"),
                                       stride=args.hair_stride, min_y=args.hair_crop))

    # One root holding the joint hierarchy and every mesh, so Godot builds a single
    # Skeleton3D with the meshes as its siblings.
    out.doc["nodes"].append({"name": "Vitruvian", "children": [0] + mesh_nodes})
    out.doc["scenes"][0]["nodes"] = [len(out.doc["nodes"]) - 1]

    glb_path = os.path.join(args.out, "vitruvian.glb")
    size = out.save(glb_path)

    # --- report -------------------------------------------------------------
    lo = [9e9] * 3
    hi = [-9e9] * 3
    for a in out.doc["accessors"]:
        if a["type"] == "VEC3" and "min" in a and a["componentType"] == 5126:
            for k in range(3):
                lo[k] = min(lo[k], a["min"][k])
                hi[k] = max(hi[k], a["max"][k])
    print(f"{'mesh':<10}{'verts':>9}{'tris':>9}")
    for n, v, t in stats:
        print(f"{n:<10}{v:>9}{t:>9}")
    print(f"{'TOTAL':<10}{sum(s[1] for s in stats):>9}{sum(s[2] for s in stats):>9}")
    print(f"height  {hi[1] - lo[1]:.4f} m   (y {lo[1]:.4f} .. {hi[1]:.4f})")
    print(f"wrote   {glb_path}  {size / 1e6:.2f} MB")

    if not args.skip_textures:
        bake_textures(gp, os.path.join(args.out, "textures"))
    return 0


# Only the maps the MineWorld materials actually sample. Sizes are deliberately
# below upstream's 4K: `ASSETS.md` holds the spike to roughly 512 px/m and the
# repository to a size a contributor can clone.
TEXTURES = [
    ("vit_face_bc.png", "face_bc.jpg", 2048), ("vit_face_n.png", "face_n.jpg", 2048),
    ("vit_face_rough.png", "face_rough.jpg", 1024),
    ("vit_body_bc.png", "body_bc.jpg", 1024), ("vit_body_n.png", "body_n.jpg", 1024),
    ("vit_body_rough.png", "body_rough.jpg", 1024),
    ("vit_fabric_n.png", "fabric_n.jpg", 512),
    ("vit_hair_diffuse.png", "hair_bc.jpg", 512),
    ("vit_hair_opacity.png", "hair_opacity.png", 512),
    ("vit_iris.png", "iris.jpg", 256), ("vit_sclera.png", "sclera.jpg", 256),
    ("vit_mouth.png", "mouth.jpg", 512),
]


def bake_textures(gp: str, out_dir: str) -> None:
    """Downscale the CC0 4K maps with macOS `sips`, JPEG where there is no alpha."""
    os.makedirs(out_dir, exist_ok=True)
    total = 0
    for src_name, dst_name, px in TEXTURES:
        src = os.path.join(gp, src_name)
        dst = os.path.join(out_dir, dst_name)
        if not os.path.exists(src):
            print(f"  missing {src_name}")
            continue
        fmt = "jpeg" if dst_name.endswith(".jpg") else "png"
        cmd = ["sips", "-s", "format", fmt, "-Z", str(px), src, "--out", dst]
        if fmt == "jpeg":
            cmd[1:1] = ["-s", "formatOptions", "78"]
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        n = os.path.getsize(dst)
        total += n
        print(f"  {dst_name:<18} {px:>5}px  {n / 1e6:.2f} MB")
    print(f"  textures total {total / 1e6:.2f} MB")


if __name__ == "__main__":
    sys.exit(main())
