#!/usr/bin/env python3
"""Every Presentation Pack's place art stays within its size budget (S14 16f-d2; operator ruling on
finding F-d1-5, 2026-10-21).

A pack's place art is committed as plain files, not Git LFS: the 3D client draws it in CI, and a
checkout without LFS would have no art to draw (`.gitattributes` keeps LFS for the cast only, whose
pointer files fall back to the town figure). Plain files grow the repository's history with every
change, so the art has a budget: 10 MB per pack.

The place art is what `tools/presentation/export_place_art.gd` writes into a pack: `assets/` (the
`.glb` pieces and their `asset.yaml`) and `materials/` (the rules file and the textures made at export
time). Every pack under `presentation/` is measured, by the same rule; no pack and no world is named.

Usage:

    python3 scripts/check_pack_size.py [PACK_DIR ...]   default: every <family>/<dimension> under presentation/
    python3 scripts/check_pack_size.py --self-test

A pack over budget prints `<pack>: place art <n> bytes (<m> MB) is over the <b> MB budget` and the
exit is 1; otherwise each pack prints one line and the exit is 0.
"""

from __future__ import annotations

import os
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PRESENTATION = ROOT / "presentation"
## The operator's budget for a pack's place art, in bytes (10 MB, decimal: the size `ls` and Git show).
BUDGET = 10 * 1000 * 1000
## The folders the place-art export writes into a pack.
ART = ("assets", "materials")


def art_bytes(pack: Path) -> int:
    """Bytes of a pack's place art, without following links."""
    total = 0
    for folder in ART:
        for directory, _, files in os.walk(pack / folder):
            for name in files:
                total += os.lstat(os.path.join(directory, name)).st_size
    return total


def packs(root: Path) -> list[Path]:
    """Every pack directory (`<family>/<dimension>` holding a `pack.yaml`) under `root`, sorted."""
    return sorted(p.parent for p in root.glob("*/*/pack.yaml"))


def check(pack: Path, budget: int = BUDGET) -> str | None:
    """None when the pack is within budget, else the failure line."""
    size = art_bytes(pack)
    if size > budget:
        return (
            f"{pack}: place art {size} bytes ({size / 1e6:.1f} MB) is over the "
            f"{budget / 1e6:g} MB budget (scripts/check_pack_size.py)"
        )
    return None


def main(argv: list[str]) -> int:
    targets = [Path(a) for a in argv] if argv else packs(PRESENTATION)
    failed = 0
    for pack in targets:
        line = check(pack)
        if line is None:
            print(f"{pack}: place art {art_bytes(pack) / 1e6:.1f} MB, within {BUDGET / 1e6:g} MB")
        else:
            print(line)
            failed += 1
    return 1 if failed else 0


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        pack = Path(tmp) / "family" / "3D"
        (pack / "assets" / "places" / "a").mkdir(parents=True)
        (pack / "materials").mkdir()
        (pack / "references").mkdir()
        (pack / "pack.yaml").write_text("id: x\n", encoding="utf-8")
        (pack / "assets" / "places" / "a" / "a.glb").write_bytes(b"\0" * 600)
        (pack / "materials" / "rules.yaml").write_bytes(b"\0" * 300)
        # review material beside the art is not place art
        (pack / "references" / "big.png").write_bytes(b"\0" * 5000)
        if packs(Path(tmp)) != [pack]:
            print(f"self-test: the pack should be found, got {packs(Path(tmp))}")
            return 1
        if art_bytes(pack) != 900:
            print(f"self-test: the art should be 900 bytes, got {art_bytes(pack)}")
            return 1
        if check(pack, 900) is not None:
            print("self-test: art exactly at the budget should pass")
            return 1
        line = check(pack, 899)
        if line is None or str(pack) not in line or "900 bytes" not in line:
            print(f"self-test: art over the budget should fail naming the pack and its size: {line}")
            return 1
    print("check_pack_size self-test: ok")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        sys.exit(self_test())
    sys.exit(main(sys.argv[1:]))
