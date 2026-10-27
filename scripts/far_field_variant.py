#!/usr/bin/env python3
"""Write a generated variant of a far-field file: every bearing rotated by a fixed angle.

The rendered far-field check (`clients/3d-spike/scripts/slice/slice_far_field_check.gd`) must take
every expected value from whichever far-field file the slice loads, never from one world's numbers
(RL-e0b I-4). A rotated copy of a shipped file is the cheapest second world that proves it: the same
terrain, water and massing at different bearings, so a check that hard-codes a bearing fails on it.

    python3 scripts/far_field_variant.py <file> <degrees> <out-dir>
    ./mineworld-slice --farfield --far-field-file=<out-dir>/<stem>.yaml

The output keeps the input's file name (a far-field file's scope must equal its stem).

    python3 scripts/far_field_variant.py <file> 0 <pack>/far_field --world=<key>

writes the same far field for another world drawn on the same street, as `<key>.yaml` with its scope
that world and its `datum_source` naming the file it was copied from (Market Town on Social Café's
street, S14 16f-d2): data, so the client keeps no list of which world borrows whose far field.
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


def rescope(data: dict, world: str, source_name: str) -> dict:
    """The same far field presented for another world: its scope is that world, and its datum says
    whose far field it was copied from (presentation data, S14 16f-d2, finding F-d1-8)."""
    data["scope"] = {"world": world}
    note = f"copied from {source_name}'s far field, the street {world} is drawn on (S14 16f-d2)"
    data["datum_source"] = f"{note}; {data.get('datum_source', '')}".rstrip("; ")
    return data


def main(argv: list[str]) -> int:
    args = [a for a in argv[1:] if not a.startswith("--world=")]
    worlds = [a.removeprefix("--world=") for a in argv[1:] if a.startswith("--world=")]
    if len(args) != 3 or len(worlds) > 1:
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        print("usage: far_field_variant.py <file> <degrees> <out-dir> [--world=<key>]", file=sys.stderr)
        return 2
    source = Path(args[0])
    degrees = float(args[1])
    out_dir = Path(args[2])
    data = rotate(json.loads(source.read_text(encoding="utf-8")), degrees)
    name = source.name
    if worlds:
        data = rescope(data, worlds[0], source.stem)
        name = f"{worlds[0]}.yaml"
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / name
    out.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{out}: every bearing of {source} turned by {degrees:g} degrees")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
