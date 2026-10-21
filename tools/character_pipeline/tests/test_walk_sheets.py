"""The default 2D pack's walk sheets (PR vf-2d-walk-cycle) agree with their bindings: every cast body
has one, its atlas is exactly `cell` × (`columns`, views), its anchor lies inside a cell, and its
sidecar names the same cycle.

The atlas size is read from the lossless WebP header (RIFF / VP8L), so no image library is needed."""

from __future__ import annotations

import json
import struct
from pathlib import Path

from conftest import REPO_ROOT

from mwcast import walk_sheets

PACK = REPO_ROOT / walk_sheets.PACK_2D


def _webp_size(path: Path) -> tuple[int, int]:
    data = path.read_bytes()[:30]
    assert data[:4] == b"RIFF" and data[8:12] == b"WEBP", f"{path} is not a WebP file"
    assert data[12:16] == b"VP8L", f"{path} is not lossless WebP"
    assert data[20] == 0x2F
    bits = struct.unpack("<I", data[21:25])[0]
    return (bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1


def test_every_cast_body_has_a_walk_sheet_that_matches_its_binding() -> None:
    sprites = json.loads((REPO_ROOT / walk_sheets.BINDINGS).read_text())["sprites"]
    bases = sorted(
        k.removesuffix("_front") for k in sprites if k.startswith("cast_") and k.endswith("_front")
    )
    assert bases, "the default pack binds no cast sprites"
    for base in bases:
        walk = sprites.get(f"{base}_walk")
        assert walk is not None, f"{base} has no walk sheet"
        sheet = walk["sheet"]
        cw, ch = sheet["cell"]
        assert _webp_size(PACK / walk["file"]) == (cw * sheet["columns"], ch * len(sheet["views"])), base
        assert sheet["columns"] == sheet["frames_per_cycle"] + 1, base
        assert sheet["views"] == ["down", "down_right", "right", "up_right", "up"], base
        ax, ay = walk["anchor"]
        assert 0 < ax < cw and 0 < ay < ch, base
        side = json.loads((PACK / "art/provenance" / f"{base}_walk.json").read_text())["render"]
        assert side["cycle_m"] == sheet["cycle_m"] and side["cell"] == sheet["cell"], base
        assert side["stature_m"] == walk["height_m"] == sprites[f"{base}_front"]["height_m"], base
