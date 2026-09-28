#!/usr/bin/env python3
"""Cut the Quaternius Universal Animation Library down to the clips we use.

The free Standard pack ships 46 clips and an 8.5k-triangle mannequin in one
6.7 MB GLB. A town needs four of the clips and none of the mannequin, so this
keeps the rig, keeps the named clips, and replaces the mesh with a single
degenerate triangle — Godot only builds a `Skeleton3D` for a skin that some mesh
actually references, and the retarget importer needs a `Skeleton3D`.

Licence: the pack is CC0 1.0; its `License.txt` is copied next to the output.

    python3 animation_trim.py --src ".../AnimationLibrary_Godot_Standard.glb" \
        --out ../assets/characters/quaternius_ual.glb
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from character_bake import Glb, Builder, COMP, NCOMP  # noqa: E402

# Idle and Walk are what the controller actually blends between; Jog covers the
# shift key; the T-pose is kept because it makes a broken retarget obvious at a
# glance instead of at the end of a debugging session.
DEFAULT_CLIPS = ("A_TPose", "Idle_Loop", "Walk_Loop", "Jog_Fwd_Loop")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--clips", nargs="*", default=list(DEFAULT_CLIPS))
    args = ap.parse_args()

    src = Glb(args.src)
    out = Builder()

    skin = src.j["skins"][0]
    joints = skin["joints"]
    old2new = {old: new for new, old in enumerate(joints)}

    for old in joints:
        n = src.j["nodes"][old]
        out.doc["nodes"].append({k: v for k, v in n.items()
                                 if k in ("name", "translation", "rotation", "scale")})
    for new, old in enumerate(joints):
        kids = [old2new[c] for c in src.j["nodes"][old].get("children", []) if c in old2new]
        if kids:
            out.doc["nodes"][new]["children"] = kids

    out.doc["skins"].append({
        "name": "Rig",
        "joints": list(range(len(joints))),
        "inverseBindMatrices": out.accessor(src.accessor(skin["inverseBindMatrices"]),
                                            5126, "MAT4"),
    })

    # A placeholder skinned triangle, so the importer produces a Skeleton3D.
    pos = out.accessor([(0.0, 0.0, 0.0)] * 3, 5126, "VEC3", target=34962, minmax=True)
    jts = out.accessor([(0, 0, 0, 0)] * 3, 5121, "VEC4", target=34962)
    wts = out.accessor([(255, 0, 0, 0)] * 3, 5121, "VEC4", target=34962, normalized=True)
    idx = out.accessor([0, 1, 2], 5123, "SCALAR", target=34963)
    out.doc["meshes"].append({"name": "RigAnchor", "primitives": [
        {"attributes": {"POSITION": pos, "JOINTS_0": jts, "WEIGHTS_0": wts}, "indices": idx}]})
    anchor = len(out.doc["nodes"])
    out.doc["nodes"].append({"name": "RigAnchor", "mesh": 0, "skin": 0})

    # --- animations ---------------------------------------------------------
    out.doc["animations"] = []
    kept = []
    for anim in src.j["animations"]:
        if anim["name"] not in args.clips:
            continue
        samplers, channels = [], []
        smap = {}
        for ch in anim["channels"]:
            node = ch["target"].get("node")
            if node not in old2new:
                continue
            si = ch["sampler"]
            if si not in smap:
                s = anim["samplers"][si]
                tin = src.accessor(s["input"])
                tout = src.accessor(s["output"])
                a_in = src.j["accessors"][s["input"]]
                a_out = src.j["accessors"][s["output"]]
                smap[si] = len(samplers)
                samplers.append({
                    "input": out.accessor(tin, a_in["componentType"], a_in["type"], minmax=True),
                    "output": out.accessor(tout, a_out["componentType"], a_out["type"],
                                           normalized=a_out.get("normalized", False)),
                    "interpolation": s.get("interpolation", "LINEAR"),
                })
            channels.append({"sampler": smap[si],
                             "target": {"node": old2new[node], "path": ch["target"]["path"]}})
        out.doc["animations"].append({"name": anim["name"], "samplers": samplers,
                                      "channels": channels})
        kept.append((anim["name"], len(channels)))

    out.doc["nodes"].append({"name": "Rig", "children": [0, anchor]})
    out.doc["scenes"][0]["nodes"] = [len(out.doc["nodes"]) - 1]

    os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
    size = out.save(args.out)
    for name, n in kept:
        print(f"  {name:<16} {n} channels")
    missing = set(args.clips) - {k[0] for k in kept}
    if missing:
        print(f"  WARNING clips not found: {sorted(missing)}")
    print(f"wrote {args.out}  {size / 1e6:.2f} MB  ({len(joints)} joints)")

    lic = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(args.src))), "License.txt")
    if os.path.exists(lic):
        dst = os.path.join(os.path.dirname(os.path.abspath(args.out)),
                           "LICENSE.quaternius-ual.txt")
        shutil.copyfile(lic, dst)
        print(f"wrote {dst}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
