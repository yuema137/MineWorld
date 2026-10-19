"""W-6's seeded outfit sweep (design §4.8; step-32 §7.2). Stdlib only.

Per subject: its outfit A and `count` outfits drawn from the pilot library. Each drawn outfit takes exactly
one garment in each required slot (top, legs, feet) and, in each optional slot (socks, outer, head, bag), one
garment or none; the subject's own hair. Every choice is a SHA-256 of the seed, the subject, the draw number
and the slot, so the sweep is the same on every machine and Python version. A draw equal to outfit A or to an
earlier draw is drawn again (counted); every drawn outfit passes `recipe.check_outfit` (slot exclusivity).
"""

from __future__ import annotations

import hashlib

from mwcast.wardrobe.recipe import Outfit, Recipe, check_outfit

SEED = "s32p-w6"  # design §4.8 W-6
REQUIRED = ("top", "legs", "feet")
MAX_REDRAWS = 50


def _pick(seed: str, subject: str, draw: int, slot: str, n: int) -> int:
    digest = hashlib.sha256(f"{seed}|{subject}|{draw}|{slot}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % n


def sweep(recipe: Recipe, subject: str, count: int = 3, seed: str = SEED) -> list[Outfit]:
    base = recipe.outfits[(subject, "A")]
    library: dict[str, list[str]] = {slot: [] for slot in recipe.slots}
    for garment in recipe.garments.values():
        if garment.kind == "garment" and garment.slot is not None:
            library[garment.slot].append(garment.id)
    for ids in library.values():
        ids.sort()
    seen = {tuple(sorted(base.garments))}
    out: list[Outfit] = []
    draw = 0
    while len(out) < count:
        if draw > count + MAX_REDRAWS:
            raise ValueError(f"{subject}: no {count} distinct outfits in {draw} draws")
        chosen: list[str] = []
        for slot in recipe.slots:
            options: list[str | None] = list(library[slot])
            if slot not in REQUIRED:
                options.append(None)
            if not options:
                continue
            choice = options[_pick(seed, subject, draw, slot, len(options))]
            if choice is not None:
                chosen.append(choice)
        draw += 1
        key = tuple(sorted(chosen))
        if key in seen:
            continue
        seen.add(key)
        outfit = Outfit(subject=subject, name=f"W6-{len(out) + 1}", garments=tuple(chosen), hair=base.hair)
        check_outfit(outfit, recipe.garments, f"W-6 {subject} draw {draw}")
        out.append(outfit)
    return out
