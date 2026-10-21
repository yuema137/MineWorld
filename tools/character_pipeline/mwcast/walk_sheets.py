"""The 2D walk sheets of the cast (PR vf-2d-walk-cycle): one atlas per body, rendered from the 3D
client's own `Stand` and `Walk` poses, written into the default 2D pack with its binding and provenance.

`cast.py walk-sheets --poses <dir> --out <dir> [--only ID ...] [--frames 12] [--write]`

`<dir>` holds the pose dumps of `cast.py ground --dump-pose` (`<id>.json`, with `walk_cycle`,
`walk_cycle_m` and `body_scale`). Each body's source export and stature are the ones its current 2D
sprites were rendered from (the `cast_<id>_front` binding's `height_m` and provenance `source_file`),
so the sheet and the two-frame sprites are the same body at the same size. The atlas is stored as
lossless WebP (`cwebp -lossless`): the same pixels as the PNG Blender writes, about a third smaller.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import cast

from mwcast import blender

PACK_2D = Path("presentation/mineworld-default/2D")
BINDINGS = PACK_2D / "assets/asset_bindings.yaml"
CAST_ART = PACK_2D / "art/cast"
PROVENANCE = PACK_2D / "art/provenance"
CHARACTERS = Path("clients/3d-spike/assets/characters")
RENDERER = Path("tools/character_pipeline/blender/render_sprites.py")
# the draw scale of every cast sprite: pixel height = stature × 64, drawn at 32 px/m (PRESENTATION.md §3)
DRAW_SCALE = 0.5


class WalkSheetError(Exception):
    """A body's source, stature or pose cannot be found, or a tool failed."""


def body_source(repo_root: Path, sprites: Mapping[str, Mapping[str, object]], key: str) -> tuple[Path, float]:
    """The export and stature `key`'s current 2D sprites were rendered from."""
    front = sprites.get(f"cast_{key}_front")
    if front is None:
        raise WalkSheetError(f"{key}: no cast_{key}_front binding to take the stature from")
    side = json.loads((repo_root / PROVENANCE / f"cast_{key}_front.json").read_text())
    name = str(side["render"]["source_file"])
    found = sorted((repo_root / CHARACTERS).rglob(name))
    if len(found) != 1:
        raise WalkSheetError(f"{key}: {len(found)} files named {name} under {CHARACTERS}")
    return found[0], float(cast(float, front["height_m"]))


def render(
    repo_root: Path,
    key: str,
    source: Path,
    stature: float,
    pose: Path,
    out: Path,
    frames: int,
    environ: Mapping[str, str],
) -> dict[str, object]:
    """Render one body's atlas into `out` and convert it to lossless WebP. Returns its sidecar."""
    exe = blender.find_blender(environ, sys.platform)
    log = out / f"{key}.log"
    with log.open("w") as handle:
        done = subprocess.run(
            [
                str(exe),
                "--background",
                "--factory-startup",
                "--python-exit-code",
                "1",
                "--python",
                str(repo_root / RENDERER),
                "--",
                str(source),
                str(out),
                key,
                str(stature),
                "--pose",
                str(pose),
                "--walk-frames",
                str(frames),
            ],
            stdout=handle,
            stderr=subprocess.STDOUT,
            check=False,
        )
    if done.returncode != 0:
        raise WalkSheetError(f"{key}: Blender failed; see {log}")
    cwebp = shutil.which("cwebp")
    if cwebp is None:
        raise WalkSheetError("cwebp (libwebp) is not on PATH")
    png, webp = out / f"{key}_walk.png", out / f"{key}_walk.webp"
    subprocess.run([cwebp, "-quiet", "-lossless", "-z", "9", str(png), "-o", str(webp)], check=True)
    return cast(dict[str, object], json.loads((out / f"{key}_walk.json").read_text()))


def binding(key: str, side: Mapping[str, object]) -> dict[str, object]:
    """The `cast_<key>_walk` sprite: the atlas, its cell anchor, the sheet's layout (PRESENTATION.md §3)."""
    return {
        "anchor": side["anchor"],
        "file": f"art/cast/{key}_walk.webp",
        "height_m": side["stature_m"],
        "ink": "none",
        "scale": DRAW_SCALE,
        "sheet": {
            "cell": side["cell"],
            "columns": len(cast(list[object], side["columns"])),
            "cycle_m": side["cycle_m"],
            "frames_per_cycle": side["frames_per_cycle"],
            "views": side["views"],
        },
    }


def write(repo_root: Path, out: Path, sides: Mapping[str, Mapping[str, object]]) -> None:
    """Copy each atlas and sidecar into the pack and bind it."""
    path = repo_root / BINDINGS
    doc = json.loads(path.read_text())
    for key, side in sorted(sides.items()):
        shutil.copy2(out / f"{key}_walk.webp", repo_root / CAST_ART / f"{key}_walk.webp")
        record = {
            "source_type": "rendered",
            "status": "candidate",
            "human_curated": False,
            "render": dict(side),
            "license": {
                "redistribution_allowed": True,
                "note": "derived from the project's own generated character; inherits its source provenance",
            },
        }
        (repo_root / PROVENANCE / f"cast_{key}_walk.json").write_text(json.dumps(record, indent=2) + "\n")
    # each sheet's binding goes right after its body's other sprites; every other entry keeps its place
    walks = {f"cast_{key}_walk": binding(key, side) for key, side in sides.items()}
    placed: dict[str, object] = {}
    for sid, sprite in doc["sprites"].items():
        if sid in walks:
            continue
        placed[sid] = sprite
        walk = sid.removesuffix("_front_b") + "_walk"
        if sid.endswith("_front_b") and walk in walks:
            placed[walk] = walks[walk]
    missing = sorted(set(walks) - set(placed))
    if missing:
        raise WalkSheetError(f"no cast_<id>_front_b binding to place {missing} after")
    doc["sprites"] = placed
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n")


def run(
    repo_root: Path,
    poses: Path,
    out: Path,
    only: Sequence[str] | None,
    frames: int,
    do_write: bool,
    environ: Mapping[str, str],
) -> dict[str, dict[str, object]]:
    sprites = json.loads((repo_root / BINDINGS).read_text())["sprites"]
    keys = sorted(p.stem for p in poses.glob("*.json"))
    if only:
        keys = [k for k in keys if k in set(only)]
    out.mkdir(parents=True, exist_ok=True)
    sides: dict[str, dict[str, object]] = {}
    for key in keys:
        source, stature = body_source(repo_root, sprites, key)
        sides[key] = render(repo_root, key, source, stature, poses / f"{key}.json", out, frames, environ)
        print(f"walk-sheets: {key} cell {sides[key]['cell']} drift {sides[key]['foot_drift_mm']}")
    if do_write:
        write(repo_root, out, sides)
    return sides
