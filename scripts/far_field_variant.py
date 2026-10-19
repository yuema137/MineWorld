#!/usr/bin/env python3
"""Write a generated variant of a far-field file: every bearing rotated by a fixed angle.

The rendered far-field check (`clients/3d-spike/scripts/slice/slice_far_field_check.gd`) must take
every expected value from whichever far-field file the slice loads, never from one world's numbers
(RL-e0b I-4). A rotated copy of a shipped file is the cheapest second world that proves it: the same
terrain, water and massing at different bearings, so a check that hard-codes a bearing fails on it.

    python3 scripts/far_field_variant.py <file> <degrees> <out-dir>
    ./mineworld-slice --farfield --far-field-file=<out-dir>/<stem>.yaml

The output keeps the input's file name (a far-field file's scope must equal its stem).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def _turn(value: float, degrees: float) -> float:
    turned = (value + degrees) % 360.0
    return int(turned) if float(turned).is_integer() else turned


def _sector(sector: dict, degrees: float) -> None:
    sector["from_deg"] = _turn(sector["from_deg"], degrees)
    sector["to_deg"] = _turn(sector["to_deg"], degrees)


def rotate(data: dict, degrees: float) -> dict:
    for body in data.get("water", []):
        _sector(body["sector"], degrees)
    for band in data.get("terrain_silhouettes", []) or []:
        _sector(band["sector"], degrees)
        for point in band["profile"]:
            point["bearing_deg"] = _turn(point["bearing_deg"], degrees)
    skyline = data.get("skyline") or {}
    for sector in skyline.get("sectors", []):
        _sector(sector["sector"], degrees)
    for slot in (skyline.get("slots") or {}).values():
        if "bearing_deg" in slot:
            slot["bearing_deg"] = _turn(slot["bearing_deg"], degrees)
    return data


def main(argv: list[str]) -> int:
    if len(argv) != 4:
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        print("usage: far_field_variant.py <file> <degrees> <out-dir>", file=sys.stderr)
        return 2
    source = Path(argv[1])
    degrees = float(argv[2])
    out_dir = Path(argv[3])
    data = rotate(json.loads(source.read_text(encoding="utf-8")), degrees)
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / source.name
    out.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{out}: every bearing of {source} turned by {degrees:g} degrees")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
