#!/usr/bin/env python3
"""Own the slice's texture import settings (RL-b SD-RLb-2, SC-3).

The slice's environment textures (`assets/textures/**`) and the Poly Haven props'
textures (`assets/models/**/textures/**`) were imported with Godot's defaults:
Lossless (`compress/mode=0`) and **no mipmaps**: 213 1k maps held uncompressed,
and distant paving sampling a single arbitrary texel per pixel. Measured (RL-b
E-RLb-2, E-RLb-4): video memory 2 516 MB before this tool's settings, 2 050 MB
after, at 1920x1080.

This tool writes, per texture `.import`, the keys that decide that:

  compress/mode=4           Basis Universal (UASTC), one file per texture
  compress/uastc_level=0    the encoder's fastest level; RDO sets the trade-off
  compress/rdo_quality_loss by role (RDO_LOSS below; `DEP-161`)
  mipmaps/generate=true     a mip chain, so a distant surface averages its texels
  compress/normal_map=1     on `*_nor_gl_*` maps only: packed and filtered as
                            normal data (transcoded to BC5 / RGTC); 0 elsewhere
  process/size_limit=512    RL-b2 (MS-Ra-2, SD-B2-5): the small and distant props
                            of `SMALL_512`; 0 everywhere else. Only prop textures are
                            ever listed; environment surfaces and the character keep 1k
  detect_3d/compress_to=0   so an editor session never rewrites the mode

Basis replaced `compress/mode=2` (BPTC) for the release size (`DEP-161`, S23
MS-Ra-2). A VRAM-compressed texture is stored once per GPU format family, and a
macOS universal export needs both S3TC/BPTC and ETC2/ASTC, so every prop map was
in the pack twice. A Basis texture is stored once and transcoded at load to the
GPU's own format: BC7/BC5 on desktop GPUs (macOS on Metal, Windows on D3D12 /
Vulkan, Linux on Vulkan), ASTC/ETC2 where a GPU has no BC formats. The three
operating systems therefore still take the same path.

It never touches `assets/characters/**` (owned by `patch_imports.py`, see
`docs/HUMANOID_PROFILE.md`) or `assets/hdri/**`. It refuses to run on a path
outside the two owned roots, so a mistaken argument cannot widen it.

After it runs, Godot must reimport (`./mineworld-slice` does, because an
`.import` newer than the cache triggers `--import`), which rewrites each file's
`[remap]` section for the new format. The resulting files are committed: the
settings are reviewable as a diff, as the character's are.

  python3 clients/3d-spike/tools/slice_imports.py           write, report changes
  python3 clients/3d-spike/tools/slice_imports.py --check   exit 1 if any file differs

A second run reports no change (idempotent).
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.dirname(HERE)

# The two roots this tool owns, relative to the project. Nothing else is walked.
OWNED = ("assets/textures", "assets/models")
# Never written, even if a path under an owned root somehow led there.
NEVER = ("assets/characters", "assets/hdri")
IMAGE_EXT = (".jpg.import", ".jpeg.import", ".png.import")

# MS-Ra-2 (RL-b2 SD-B2-5): the small and distant props whose textures are imported at 512 px.
# Each class sets its texel-density floor (RL-b2 SD-B2-6, checked in the built slice by
# `--texel-density`): hand-held goods 1 024 px/m (step-22 section 6.4), the other classes
# 512 px/m (DEP-8). A slug is listed here only if it is a prop directory under assets/models;
# the table may shrink, never grow, without a reviewed change to the design (RL-b2 section 13).
# Of section 5.1's 21 slugs, 13 measured under their floor at 512 px and stay 1k (E-B2-5):
# tea_set_01, jug_01, wooden_bowl_01, wine_bottles_01, antique_ceramic_vase_01, ceramic_vase_02,
# pot_enamel_01, brass_pot_01, wicker_basket_01, wooden_crate_01, water_manhole_cover,
# hanging_industrial_lamp, modern_ceiling_lamp_01.
SMALL_512: dict[str, str] = {
    "croissant": "hand-held",
    "carrot_cake": "hand-held",
    "food_apple_01": "hand-held",
    "planter_pot_clay": "small",
    "calathea_orbifolia_01": "small",
    "industrial_pipe_lamp": "distant",
    "wall_clock": "distant",
    "fancy_picture_frame_01": "distant",
}
MODELS = "assets/models"


def slug_of(path: str) -> str:
    """The prop slug of a texture under assets/models/<slug>/textures/, else ''."""
    rel = path.replace(os.sep, "/")
    if f"{MODELS}/" not in rel or "/textures/" not in rel:
        return ""
    parts = rel.split(f"{MODELS}/", 1)[1].split("/")
    return parts[0] if len(parts) > 2 and parts[1] == "textures" else ""


# Rate-distortion loss per texture role (`DEP-161`): how far Basis may trade texel error for a smaller
# file. Chosen from rendered close-ups of three props at the client's closest framing: nearly all of the
# visible error came from the base colour's loss, so colour takes the least and the packed and normal
# maps take the cast's 4.0.
RDO_LOSS = {"colour": "1.0", "packed": "4.0", "normal": "4.0"}


def role(path: str) -> str:
    """`normal` for Poly Haven's `*_nor_gl_*` maps, `packed` for the AO/roughness/metal maps
    (`*_arm_*`, `*_rough_*`), `colour` for everything else (base colour, emission, alpha)."""
    name = os.path.basename(path)
    if "_nor_gl_" in name:
        return "normal"
    if "_arm_" in name or "_rough_" in name:
        return "packed"
    return "colour"


def wanted(path: str) -> dict[str, str]:
    r = role(path)
    return {
        "compress/mode": "4",
        "compress/uastc_level": "0",
        "compress/rdo_quality_loss": RDO_LOSS[r],
        "mipmaps/generate": "true",
        "compress/normal_map": "1" if r == "normal" else "0",
        "process/size_limit": "512" if slug_of(path) in SMALL_512 else "0",
        "detect_3d/compress_to": "0",
    }


def table_errors() -> list[str]:
    """Table entries that are not prop directories under assets/models (refused before any write)."""
    return [
        slug
        for slug in SMALL_512
        if not os.path.isdir(os.path.join(PROJ, MODELS, slug))
    ]


def owned_texture(rel: str) -> bool:
    rel = rel.replace(os.sep, "/")
    if any(rel == n or rel.startswith(n + "/") for n in NEVER):
        return False
    if not any(rel.startswith(o + "/") for o in OWNED):
        return False
    if rel.startswith("assets/models/") and "/textures/" not in rel:
        return False
    return rel.lower().endswith(IMAGE_EXT)


def patch(path: str, write: bool) -> bool:
    want = wanted(path)
    with open(path, encoding="utf-8") as f:
        text = f.read()
    if 'importer="texture"' not in text:
        return False
    out, seen = [], set()
    for line in text.splitlines():
        key = line.split("=", 1)[0].strip()
        if key in want:
            out.append(f"{key}={want[key]}")
            seen.add(key)
        else:
            out.append(line)
    missing = [k for k in want if k not in seen]
    if missing:
        at = max(i for i, ln in enumerate(out) if ln.startswith("compress/")) + 1
        for k in missing:
            out.insert(at, f"{k}={want[k]}")
    new = "\n".join(out) + "\n"
    if new == text:
        return False
    if write:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(new)
    return True


def main(argv: list[str]) -> int:
    check = "--check" in argv
    bad = table_errors()
    for slug in bad:
        print(f"slice_imports: {slug} is not a prop under assets/models; refusing")
    if bad:
        return 2
    changed, seen, at_512 = [], 0, 0
    for root in OWNED:
        for dirpath, _dirs, files in os.walk(os.path.join(PROJ, root)):
            for name in sorted(files):
                full = os.path.join(dirpath, name)
                rel = os.path.relpath(full, PROJ)
                if not owned_texture(rel):
                    continue
                seen += 1
                if slug_of(full) in SMALL_512:
                    at_512 += 1
                if patch(full, write=not check):
                    changed.append(rel)
    verb = "differ" if check else "changed"
    print(f"slice_imports: {seen} texture imports owned, {len(changed)} {verb}, "
          f"{at_512} at 512 px")
    for rel in changed:
        print(f"  {rel}")
    return 1 if check and changed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
