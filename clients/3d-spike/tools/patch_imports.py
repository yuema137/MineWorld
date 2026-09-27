#!/usr/bin/env python3
"""Point the two character .import files at their BoneMaps.

Godot writes `_subresources={}` when it first imports a GLB, and the retarget
settings live inside that dictionary. They are reachable from the editor's
Advanced Import dialog, but this project has no committed editor session, so the
settings are written here instead — which also makes them reviewable as a diff
rather than as a binary blob nobody can see into.

The six options below are the whole retarget contract, and the reason each is
set the way it is:

* **bone_map** — the committed profile mapping (`make_bonemaps.gd`).
* **rename_bones** — rewrites `mixamorig_LeftArm` / `DEF-upper_arm.L` to the
  profile's `LeftUpperArm`. This is what makes one animation set portable across
  characters; without it the tracks address names only one rig has.
* **apply_node_transforms** — Blender bakes a −90° X quaternion on the root of
  its glTF exports. Quaternius's rig carries it, ours does not. Set it on BOTH
  or a clip plays with the character lying on its back and nothing in the log
  says why.
* **overwrite_axis** — rewrites each bone's rest onto the profile's axis
  convention (+Y parent→child). Godot's own docs call this the most important
  option for sharing animations.
* **fix_silhouette** — rotates an A-posed rest toward the profile's T-pose
  reference. The CharMorph character is A-posed and the Quaternius clips are
  authored on a T-pose, so the character needs it and the clips do not. The feet
  are filtered out: silhouette fixing rotates a foot to point at its toe, which
  tips the whole foot forward and leaves the character on tiptoe.
* **normalize_position_tracks** — stores hip height as the skeleton's
  motion_scale so one clip set serves a 1.68 m and a 1.80 m person. Note it
  rescales *position* tracks only; stride length is baked into leg *rotation*,
  so it does not fix foot sliding. That is tuned in `human.gd`.
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.dirname(HERE)

TARGETS = [
    # (.import file, skeleton node path, bone map, fix silhouette)
    ("assets/characters/vitruvian/vitruvian.glb.import", "Vitruvian/Skeleton3D",
     "res://assets/characters/vitruvian/vitruvian_bonemap.tres", True),
    ("assets/characters/quaternius_ual.glb.import", "Rig/Skeleton3D",
     "res://assets/characters/quaternius_ual_bonemap.tres", False),
]


def subresources(node_path: str, bone_map: str, fix_silhouette: bool) -> str:
    opts = [
        f'"retarget/bone_map": Resource("{bone_map}")',
        '"retarget/bone_renamer/rename_bones": true',
        '"retarget/rest_fixer/apply_node_transforms": true',
        '"retarget/rest_fixer/overwrite_axis": true',
        '"retarget/rest_fixer/normalize_position_tracks": true',
        '"retarget/rest_fixer/reset_all_bone_poses_after_import": false',
        f'"retarget/rest_fixer/fix_silhouette/enable": {str(fix_silhouette).lower()}',
        '"retarget/rest_fixer/fix_silhouette/threshold": 15.0',
        '"retarget/rest_fixer/fix_silhouette/filter": Array[String](["LeftFoot", "RightFoot", "LeftToes", "RightToes"])',
    ]
    body = ",\n".join(opts)
    return ('_subresources={\n"nodes": {\n"PATH:%s": {\n%s\n}\n}\n}' % (node_path, body))


def main() -> int:
    changed = 0
    for rel, node_path, bone_map, fix in TARGETS:
        path = os.path.join(PROJ, rel)
        if not os.path.exists(path):
            print(f"  missing {rel} — run `godot --headless --path . --import` first")
            return 1
        text = open(path).read()
        block = subresources(node_path, bone_map, fix)
        out, seen = [], False
        skip = False
        for line in text.splitlines():
            if line.startswith("_subresources="):
                out.append(block)
                seen = True
                skip = not line.rstrip().endswith("}") or line.strip() == "_subresources={"
                if line.strip() == "_subresources={}":
                    skip = False
                continue
            if skip:
                if line.strip() == "}":
                    skip = False
                continue
            out.append(line)
        if not seen:
            print(f"  {rel}: no _subresources line")
            return 1
        new = "\n".join(out) + "\n"
        if new != text:
            open(path, "w").write(new)
            changed += 1
            print(f"  patched {rel}  (fix_silhouette={fix})")
        else:
            print(f"  unchanged {rel}")
    print(f"{changed} file(s) changed; re-run `godot --headless --path . --import`")
    return 0


if __name__ == "__main__":
    sys.exit(main())
