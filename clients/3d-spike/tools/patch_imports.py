#!/usr/bin/env python3
"""Own the character's import settings: the two BoneMaps, and the texture flags.

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

---

**The texture flags, and why this file owns them too.**

Godot writes a texture's `.import` on first sight and nobody looks at it again.
That left every character map on this project's defaults, and two of those
defaults were wrong in ways that look like art problems rather than settings:

* **`mipmaps/generate=false`.** A 2048 face albedo on a head that covers 60–200
  screen pixels means one pixel spans ten to thirty texels. With no mip chain
  the sampler takes a single arbitrary texel per pixel, so the pore and crease
  detail that should average into the *form* of a nose and a mouth aliases into
  per-pixel noise instead. The face reads as a featureless smear, and no amount
  of brightening fixes it. The same sampling on `hair_opacity.png`, whose strand
  field is far finer than a pixel at this distance, is what produced the
  yellow-tan speckling over the hair: alpha-to-coverage dithering a randomly
  sampled alpha.
* **`compress/normal_map=0` on the normal maps.** It tags a texture as normal
  data so the importer packs and filters it as such. It is set correctly here
  now, but honesty about the evidence: the A/B render that fixed the face
  changed *only* mipmaps, so the mipmap flag is the demonstrated cause and this
  one is correctness rather than a measured improvement.

Godot 4 decides a texture's colour space at the material (an albedo slot samples
sRGB, a normal slot does not), so there is no `flags/srgb` to set here; a first
attempt wrote one and the importer silently dropped it.

Both flags are now written here, per texture, by role.
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.dirname(HERE)

TARGETS = [
    # (.import file, skeleton node path, bone map, fix silhouette)
    # The path is the imported *scene* path, not the glTF node name: the GLB now
    # comes out of Blender, which nests the skeleton under the armature object.
    ("assets/characters/vitruvian/vitruvian.glb.import",
     "mixamo_vitruvian/Skeleton3D",
     "res://assets/characters/vitruvian/vitruvian_bonemap.tres", True),
    ("assets/characters/quaternius_ual.glb.import", "Rig/Skeleton3D",
     "res://assets/characters/quaternius_ual_bonemap.tres", False),
]


# role -> (mipmaps, compress mode, normal_map flag)
# compress/mode 0 is Lossless: skin and a hair alpha mask both show DXT blocking
# badly, and these are small enough that VRAM compression buys little.
ROLES = {
    "albedo": (True, 0, 0),
    "linear": (True, 0, 0),         # roughness, and the hair opacity mask
    "normal": (True, 0, 1),
}

TEXTURE_DIR = "assets/characters/vitruvian/textures"
TEXTURE_ROLES = {
    "face_bc.jpg": "albedo", "body_bc.jpg": "albedo", "hair_bc.jpg": "albedo",
    "iris.jpg": "albedo", "sclera.jpg": "albedo", "mouth.jpg": "albedo",
    "tee_bc.jpg": "albedo", "denim_bc.jpg": "albedo",
    "face_rough.jpg": "linear", "body_rough.jpg": "linear",
    "hair_opacity.png": "linear",
    "face_n.jpg": "normal", "body_n.jpg": "normal", "fabric_n.jpg": "normal",
}


def patch_texture(path: str, role: str) -> bool:
    mip, mode, nmap = ROLES[role]
    want = {
        "mipmaps/generate": "true" if mip else "false",
        "compress/mode": str(mode),
        "compress/normal_map": str(nmap),
    }
    text = open(path).read()
    lines = text.splitlines()
    out, seen = [], set()
    for line in lines:
        key = line.split("=", 1)[0].strip()
        if key in want:
            out.append(f"{key}={want[key]}")
            seen.add(key)
        else:
            out.append(line)
    # keys Godot omitted because they were at their default have to be inserted
    missing = [k for k in want if k not in seen]
    if missing:
        at = max(i for i, ln in enumerate(out) if ln.startswith("compress/")) + 1
        for k in missing:
            out.insert(at, f"{k}={want[k]}")
    new = "\n".join(out) + "\n"
    if new == text:
        return False
    open(path, "w").write(new)
    return True


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
    for name, role in sorted(TEXTURE_ROLES.items()):
        tp = os.path.join(PROJ, TEXTURE_DIR, name + ".import")
        if not os.path.exists(tp):
            print(f"  missing {name}.import — import the project first")
            continue
        if patch_texture(tp, role):
            changed += 1
            print(f"  patched {name:<18} as {role}")
    print(f"{changed} file(s) changed; re-run `godot --headless --path . --import`")
    return 0


if __name__ == "__main__":
    sys.exit(main())
