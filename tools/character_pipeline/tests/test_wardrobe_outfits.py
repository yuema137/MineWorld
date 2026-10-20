"""W-6's seeded sweep: reproducible, slot-compatible, distinct from outfit A and from each other."""

from __future__ import annotations

from pathlib import Path

from mwcast.wardrobe.outfits import REQUIRED, sweep
from mwcast.wardrobe.recipe import load_recipe

RECIPE = load_recipe(Path(__file__).resolve().parents[1] / "recipes" / "s32p-pilot.yaml")


def test_the_sweep_is_reproducible_and_slot_compatible() -> None:
    for subject in RECIPE.subjects:
        first = sweep(RECIPE, subject)
        again = sweep(RECIPE, subject)
        assert first == again
        assert len(first) == 3
        keys = {tuple(sorted(o.garments)) for o in first}
        assert len(keys) == 3
        assert tuple(sorted(RECIPE.outfits[(subject, "A")].garments)) not in keys
        for outfit in first:
            slots = [RECIPE.garment(g).slot for g in outfit.garments]
            assert len(slots) == len(set(slots))
            assert set(REQUIRED) <= set(slots)
            assert outfit.hair == RECIPE.outfits[(subject, "A")].hair


def test_another_seed_draws_another_sweep() -> None:
    assert sweep(RECIPE, "hana-sato") != sweep(RECIPE, "hana-sato", seed="another-seed")
