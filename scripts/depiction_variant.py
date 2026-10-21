#!/usr/bin/env python3
"""Write a generated variant of a depiction file (S33-d sec.5.0.6; DECISIONS.md ARC-177).

The 3D client's builders must draw whatever a depiction file says and keep no world's numbers. A
variant of a shipped file is the cheapest second world that proves it: the same world, server and
places, with everything the presentation owns redrawn from a seed -- so a builder that kept one of the
original's values in code draws the variant wrong, and its frames and checks show it.

    python3 scripts/depiction_variant.py <file> --seed <n> <out-dir>
    ./mineworld-slice --world=<key> --depiction-file=<out-dir>/<stem>.yaml

Kept, because the server's world fixes them: scope, frame, street, places, spawn, viewpoints, and every
building that depicts a place keeps its footprint and its openings at the doorway. Changed,
deterministically from the seed:

    terrain     every profile height and every rising ramp end scaled by one factor in 0.6 ... 1.6
    water       the outer vertices (on the extent's square) moved out by up to 10 m; the water line stays
    buildings   roof forms walked round gable -> hip -> mono -> flat by one to four steps (the shell's
                index), so all four forms appear; pitches 25 ... 50 deg, wall and roof
                materials swapped among the slugs the file already uses, storeys +-1 within 1 ... 3
                (openings above the new top storey dropped); the undepicting shells moved, resized by up
                to 1 m and their count changed by one, each redrawn until clear of every footprint
    trees       every stand's seed, spacing (+-30 %, within the validator's cap), density rise and species
                weights redrawn; every species' height range scaled by 0.7 ... 1.3
    scatter     every seed and count redrawn (reeds at most 40)
    decks       (ARC-178) footprint, surface, approach, rails and curb kept (the place's floor and the
                pier's ADA heights); piles' bent count +-2, the shelter's roof walked one to three forms
                round the cycle with a pitch in 25 ... 45 deg, every moored craft's heading +-10 deg and,
                with two or more, the last one dropped half the time

The output keeps the input's file name (a depiction file's scope must equal its stem). Standard library
only; the same input and seed give the same bytes.
"""

from __future__ import annotations

import argparse
import copy
import json
import math
import random
from pathlib import Path

TOLERANCE_MM = 50
MAX_TREES = 4000
MAX_REEDS = 40
ROOF_CYCLE_ALL = {"gable": "hip", "hip": "mono", "mono": "flat", "flat": "gable"}


def _round(x: float, step: float = 0.01) -> float:
    return round(round(x / step) * step, 6)


def _rect_poly(r: list[int]) -> list[list[int]]:
    x, y, w, h = r
    return [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]


def _bounds(poly: list[list[int]]) -> tuple[int, int, int, int]:
    xs = [p[0] for p in poly]
    ys = [p[1] for p in poly]
    return min(xs), min(ys), max(xs), max(ys)


def _overlaps(a: tuple[int, int, int, int], b: tuple[int, int, int, int], margin: int) -> bool:
    return a[0] < b[2] - margin and a[2] > b[0] + margin and a[1] < b[3] - margin and a[3] > b[1] + margin


def _terrain(t: dict, rng: random.Random) -> None:
    factor = rng.uniform(0.6, 1.6)
    for profile in t.get("profiles", []):
        for s in profile["samples"]:
            s["height_m"] = _round(s["height_m"] * factor)
    for region in t["regions"]:
        ramp = region["height"].get("ramp")
        if ramp:
            for end in ("from_m", "to_m"):
                if ramp[end] > 0:
                    ramp[end] = _round(ramp[end] * factor)


def _water(w: dict, extent: list[int] | None, rng: random.Random) -> None:
    if not extent:
        return
    x0 = min(p[0] for p in w["polygon_mm"])
    x1 = max(p[0] for p in w["polygon_mm"])
    y0 = min(p[1] for p in w["polygon_mm"])
    out = {"x0": rng.randint(0, 10000), "x1": rng.randint(0, 10000), "y0": rng.randint(0, 10000)}
    lo_x, lo_y = extent[0], extent[1]
    hi_x = extent[0] + extent[2]
    for p in w["polygon_mm"]:
        if p[0] == x0:
            p[0] = max(lo_x, x0 - out["x0"])
        elif p[0] == x1:
            p[0] = min(hi_x, x1 + out["x1"])
        if p[1] == y0:
            p[1] = max(lo_y, y0 - out["y0"])


def _shells(data: dict, rng: random.Random) -> None:
    walls = sorted({b["walls"]["material"] for b in data["buildings"]})
    roofs = sorted({b["roof"]["material"] for b in data["buildings"]})
    floors = [_bounds(_rect_poly(p["footprint_mm"])) for p in data["places"]]
    floors.append(_bounds(_rect_poly(data["street"]["footprint_mm"])))
    kept = [b for b in data["buildings"] if b.get("depicts") is not None]
    free = [b for b in data["buildings"] if b.get("depicts") is None]
    if free:
        if rng.random() < 0.5 and len(free) > 1:
            free.pop(rng.randrange(len(free)))
        else:
            extra = copy.deepcopy(rng.choice(free))
            extra["name"] = f"{extra['name']}-v"
            free.append(extra)
    for i, b in enumerate(kept + free):
        roof = b["roof"]
        # walked round the cycle by the shell's index, so a file of one form still gets all four
        for _ in range(1 + i % 4):
            roof["form"] = ROOF_CYCLE_ALL[roof["form"]]
        roof.pop("falls", None)
        if roof["form"] == "mono":
            roof.pop("ridge", None)
            roof["falls"] = rng.choice(["n", "s", "e", "w"])
        roof["pitch_deg"] = rng.randint(25, 50)
        b["walls"]["material"] = rng.choice(walls)
        roof["material"] = rng.choice(roofs)
        b["storeys"] = max(1, min(3, b["storeys"] + rng.choice([-1, 1])))
        b["openings"] = [o for o in b.get("openings", []) if o["storey"] < b["storeys"]]
    taken = floors + [_bounds(b["footprint_mm"]) for b in kept]
    placed = []
    for b in free:
        x0, y0, x1, y1 = _bounds(b["footprint_mm"])
        for attempt in range(200):
            # the search widens from 3 m round the shell's place, so a crowded edge still finds room
            reach = 3000 + attempt * 50
            dx, dy = rng.randint(-reach, reach), rng.randint(-reach, reach)
            w = max(3000, (x1 - x0) + rng.randint(-1000, 1000))
            h = max(3000, (y1 - y0) + rng.randint(-1000, 1000))
            box = (x0 + dx, y0 + dy, x0 + dx + w, y0 + dy + h)
            if not any(_overlaps(box, t, -TOLERANCE_MM) for t in taken + placed):
                b["footprint_mm"] = _rect_poly([box[0], box[1], w, h])
                placed.append(box)
                break
        else:
            raise SystemExit(f"{b['name']}: no clear place found in 200 tries")
        for o in b.get("openings", []):
            if o.get("offset_m") is not None:
                o["offset_m"] = _round(min(o["offset_m"], max(0.0, (min(w, h) / 1000.0) - o["width_m"] - 0.3)))
    data["buildings"] = kept + free


def _ring_area(area: dict) -> float:
    ring = area.get("ring")
    if ring:
        return math.pi * (ring["outer_m"] ** 2 - ring["inner_m"] ** 2)
    xs, ys = zip(*area["polygon_mm"])
    n = len(xs)
    return abs(sum(xs[i] * ys[(i + 1) % n] - xs[(i + 1) % n] * ys[i] for i in range(n))) / 2e6


def _trees(trees: dict, rng: random.Random) -> None:
    for s in trees["species"]:
        f = rng.uniform(0.7, 1.3)
        lo = max(0.3, _round(s["height_m"]["min"] * f))
        hi = max(lo, _round(s["height_m"]["max"] * f))
        s["height_m"] = {"min": lo, "max": min(100.0, hi)}
    for stand in trees.get("stands", []):
        stand["seed"] = rng.randint(1, 10_000)
        least = math.sqrt(_ring_area(stand["area"]) / MAX_TREES) * 1.02
        stand["spacing_m"] = _round(max(least, 1.0, stand["spacing_m"] * rng.uniform(0.7, 1.3)))
        a, b = stand["density"]["rise_m"]
        shift = rng.uniform(-5.0, 5.0)
        stand["density"]["rise_m"] = [_round(max(0.0, a + shift)), _round(max(a + shift, b + shift, 0.0))]
        for m in stand["mix"]:
            m["weight"] = _round(rng.uniform(0.05, 1.0))
    for single in trees.get("singles", []):
        single["height_m"] = _round(single["height_m"] * rng.uniform(0.7, 1.3))


def _scatter(scatter: list[dict], rng: random.Random) -> None:
    for s in scatter:
        s["seed"] = rng.randint(1, 10_000)
        cap = MAX_REEDS if s["kind"] == "reeds" else max(1, s["count"] * 2)
        s["count"] = rng.randint(max(1, s["count"] // 2), cap)


def _decks(decks: list[dict], rng: random.Random) -> None:
    for d in decks:
        piles = d["piles"]
        piles["bents"] = rng.randint(max(2, piles["bents"] - 2), piles["bents"] + 2)
        shelter = d.get("shelter")
        if shelter:
            roof = shelter["roof"]
            for _ in range(rng.randint(1, 3)):
                roof["form"] = ROOF_CYCLE_ALL[roof["form"]]
            roof.pop("falls", None)
            if roof["form"] == "mono":
                roof["falls"] = rng.choice(["n", "s", "e", "w"])
            roof["pitch_deg"] = rng.randint(25, 45)
        moored = d.get("moored", [])
        for m in moored:
            m["yaw_deg"] = _round(m["yaw_deg"] + rng.uniform(-10.0, 10.0), 1.0)
        if len(moored) >= 2 and rng.random() < 0.5:
            moored.pop()


def variant(data: dict, seed: int) -> dict:
    rng = random.Random(seed)
    out = copy.deepcopy(data)
    terrain = out.get("terrain")
    if terrain:
        _terrain(terrain, rng)
    if out.get("water"):
        _water(out["water"], terrain["extent_mm"] if terrain else None, rng)
    if out.get("buildings"):
        _shells(out, rng)
    if out.get("trees"):
        _trees(out["trees"], rng)
    if out.get("scatter"):
        _scatter(out["scatter"], rng)
    if out.get("decks"):
        _decks(out["decks"], rng)
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("file", type=Path)
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("out_dir", type=Path)
    args = parser.parse_args()
    data = json.loads(args.file.read_text(encoding="utf-8"))
    args.out_dir.mkdir(parents=True, exist_ok=True)
    out = args.out_dir / args.file.name
    out.write_text(json.dumps(variant(data, args.seed), indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(out)


if __name__ == "__main__":
    main()
