#!/usr/bin/env python3
"""Own the slice's texture import settings (RL-b SD-RLb-2, SC-3).

The slice's environment textures (`assets/textures/**`) and the Poly Haven props'
textures (`assets/models/**/textures/**`) were imported with Godot's defaults:
Lossless (`compress/mode=0`) and **no mipmaps**: 213 1k maps held uncompressed,
and distant paving sampling a single arbitrary texel per pixel. Measured (RL-b
E-RLb-2, E-RLb-4): video memory 2 516 MB before this tool's settings, 2 050 MB
after, at 1920x1080.

This tool writes, per texture `.import`, the four keys that decide that:

  compress/mode=2           VRAM compressed
  compress/high_quality=true    BPTC (BC7) on desktop rather than S3TC (DXT)
  mipmaps/generate=true     a mip chain, so a distant surface averages its texels
  compress/normal_map=1     on `*_nor_gl_*` maps only: packed and filtered as
                            normal data (RGTC / BC5); 0 everywhere else

The desktop BC formats are what Godot's Forward+ renderer uploads on macOS
(Metal, Apple silicon), Windows (D3D12 / Vulkan) and Linux (Vulkan), so the three
operating systems take the same path.

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


def wanted(path: str) -> dict[str, str]:
    normal = "_nor_gl_" in os.path.basename(path)
    return {
        "compress/mode": "2",
        "compress/high_quality": "true",
        "mipmaps/generate": "true",
        "compress/normal_map": "1" if normal else "0",
    }


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
    changed, seen = [], 0
    for root in OWNED:
        for dirpath, _dirs, files in os.walk(os.path.join(PROJ, root)):
            for name in sorted(files):
                full = os.path.join(dirpath, name)
                rel = os.path.relpath(full, PROJ)
                if not owned_texture(rel):
                    continue
                seen += 1
                if patch(full, write=not check):
                    changed.append(rel)
    verb = "differ" if check else "changed"
    print(f"slice_imports: {seen} texture imports owned, {len(changed)} {verb}")
    for rel in changed:
        print(f"  {rel}")
    return 1 if check and changed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
