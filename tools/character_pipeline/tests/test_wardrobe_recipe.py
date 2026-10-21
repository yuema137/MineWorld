"""The pilot recipe: the committed file parses; bad cross-references and missing picks are refused by name."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import cast

import pytest

from mwcast.records import RecordError
from mwcast.wardrobe.recipe import parse_recipe

RECIPE = Path(__file__).resolve().parents[1] / "recipes" / "s32p-pilot.yaml"


def _raw() -> dict[str, object]:
    return cast(dict[str, object], json.loads(RECIPE.read_text(encoding="utf-8")))


def _parse(raw: dict[str, object]) -> None:
    parse_recipe(raw, RECIPE)


def _section(raw: dict[str, object], *keys: str) -> dict[str, object]:
    node: dict[str, object] = raw
    for key in keys:
        node = cast(dict[str, object], node[key])
    return node


def test_the_committed_recipe_has_the_frozen_library_and_outfits() -> None:
    recipe = parse_recipe(_raw(), RECIPE)

    garments = [g for g in recipe.garments.values() if g.kind == "garment"]
    hair = [g for g in recipe.garments.values() if g.kind == "hair"]
    assert len(garments) == 15 and len(hair) == 3  # §4.5: 14 + 1 tight-fit garment, 3 hair styles
    assert [g.id for g in garments if g.fit == "tight"] == ["hana-tights"]
    assert recipe.budget.cap == 1200
    assert recipe.image_model.model == "gpt-image-2.5-sunburst"
    assert recipe.image_model.call_cap == 70
    assert recipe.template_version == 2
    assert len(recipe.outfits) == 6
    assert recipe.slot_layer("outer") == 4  # design §4.4's hoodie: layer 4


def test_an_unknown_key_is_refused_with_its_path() -> None:
    raw = _raw()
    _section(raw, "garments", "vera-tee")["colour"] = "oatmeal"

    with pytest.raises(RecordError, match=r"garments\.vera-tee\.colour: unknown key"):
        _parse(raw)


def test_an_open_region_must_be_a_covered_region() -> None:
    """S36-P3b: `open_regions` names covered regions the part does not enclose; any other is refused."""
    raw = _raw()
    _section(raw, "garments", "vera-hoodie")["open_regions"] = ["torso", "thighs"]

    with pytest.raises(RecordError, match=r"vera-hoodie\.open_regions: \['thighs'\] not in covers_regions"):
        _parse(raw)


def test_two_garments_in_one_slot_are_refused() -> None:
    raw = _raw()
    outfit = _section(raw, "outfits", "vera-lindgren", "A")
    outfit["garments"] = ["vera-tee", "hana-shirt", "vera-jeans"]

    with pytest.raises(RecordError, match="both take slot top"):
        _parse(raw)


def test_an_outfit_must_name_a_hair_style() -> None:
    raw = _raw()
    _section(raw, "outfits", "hana-sato", "B")["hair"] = "vera-tee"

    with pytest.raises(RecordError, match="not a hair style"):
        _parse(raw)


def test_a_tight_fit_garment_cannot_be_a_meshy_garment() -> None:
    raw = _raw()
    _section(raw, "garments", "hana-tights")["source"] = "meshy"

    with pytest.raises(RecordError, match="tight-fit garment has source tight"):
        _parse(raw)


def test_a_wearer_must_be_a_subject() -> None:
    raw = _raw()
    _section(raw, "garments", "ivan-polo")["wearer"] = "otto-brandt"

    with pytest.raises(RecordError, match="is not a subject"):
        _parse(raw)


def test_a_missing_base_layer_pick_refuses_the_body_spend() -> None:
    raw = copy.deepcopy(_raw())
    subjects = cast(dict[str, dict[str, object]], raw["subjects"])
    subjects["vera-lindgren"]["base_layer_pick"] = None  # the committed recipe records the operator's pick
    recipe = parse_recipe(raw, RECIPE)

    with pytest.raises(RecordError, match="base_layer_pick is null"):
        recipe.require_base_layer_pick("vera-lindgren")


def test_a_recorded_pick_is_returned_and_a_tight_garment_has_no_meshy_run() -> None:
    raw = copy.deepcopy(_raw())
    pick = {
        "file": "base_layer_02.png",
        "sha256": "a" * 64,
        "by": "operator",
        "date": "2026-10-18",
        "source": "x",
    }
    _section(raw, "subjects", "vera-lindgren")["base_layer_pick"] = pick
    recipe = parse_recipe(raw, RECIPE)

    assert recipe.require_base_layer_pick("vera-lindgren").file == "base_layer_02.png"
    with pytest.raises(RecordError, match="tight-fit garment; it has no Meshy run"):
        recipe.require_reference_pick("hana-tights")


S26C = Path(__file__).resolve().parents[1] / "recipes" / "s26c-cast.yaml"


def test_the_s26c_recipe_is_fused_with_every_pick_and_no_outfits() -> None:
    recipe = parse_recipe(json.loads(S26C.read_text(encoding="utf-8")), S26C)
    assert recipe.route == "fused"
    assert len(recipe.subjects) == 21 and not recipe.outfits and not recipe.garments
    assert all(s.base_layer_pick is not None for s in recipe.subjects.values())
    assert recipe.budget.cap + 1200 <= recipe.budget.s26_approval  # the S32-p pilot keeps its 1,200


def test_a_fused_recipe_with_outfits_is_refused() -> None:
    raw = cast(dict[str, object], json.loads(S26C.read_text(encoding="utf-8")))
    outfit: dict[str, object] = {"garments": [], "hair": "x"}
    raw["outfits"] = {"alice-moreau": {"A": outfit, "B": outfit}}
    with pytest.raises(RecordError, match="fused"):
        parse_recipe(raw, S26C)


def test_a_modular_recipe_still_needs_outfits_for_every_subject() -> None:
    raw = _raw()
    _section(raw, "outfits").pop("hana-sato")
    with pytest.raises(RecordError, match="no outfits"):
        _parse(raw)
