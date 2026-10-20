"""Every item the shipped worlds disclose has a name in the agent's view (the promo bug: goods for sale had no
name and `inspect` of one answered `unknown_ref`, so an agent could not buy a coffee by name).

The observations are `tests/fixtures/observations/items/<world>.json`, crawled from the real server in every
shipped world (`tools/cli/tests/item_names.rs`, which also checks the server half). The expected name of each
item is read from the raw JSON on the observer's place, never from `view.py`: its `item-catalogue` name, or,
for a loose object (bodies' `loose-objects`, not a kind), its listing entry's tags joined by ", ".
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId

from mineworld_agent.refs import RefTable
from mineworld_agent.view import Rendered, catalogued, render

FIXTURES = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "observations" / "items"
WORLDS = sorted(path.stem for path in FIXTURES.glob("*.json"))

type Raw = dict[str, Any]


def _observations(world: str) -> list[Raw]:
    value: list[Raw] = json.loads((FIXTURES / f"{world}.json").read_text(encoding="utf-8"))
    return value


def _payload(value: Raw, entity_id: str, kind: str) -> Any:  # noqa: ANN401 (a pack's raw JSON)
    entity = next((e for e in value["entities"] if e["id"] == entity_id), None)
    if entity is None:
        return None
    return next((c["payload"] for c in entity["components"] if c["component_type"] == kind), None)


def _items(node: Any, found: set[str]) -> None:  # noqa: ANN401 (raw JSON)
    if isinstance(node, dict):
        if node.get("entity_type") == "item" and isinstance(node.get("entity"), str):
            found.add(node["entity"])
            return
        for inner in node.values():
            _items(inner, found)
    elif isinstance(node, list):
        for inner in node:
            _items(inner, found)


def _expected(value: Raw) -> tuple[dict[str, str], set[str]]:
    """What my place calls each item, and every item the observation names."""
    place = value["self_location"]["place"]["entity"]
    loose = _payload(value, place, "loose-objects") or []
    names = {
        lying["object"]["entity"]: ", ".join(lying["tags"]) for lying in loose if lying.get("tags")
    }
    catalogue = _payload(value, place, "item-catalogue") or {"kinds": []}
    names.update({kind["item"]["entity"]: kind["name"] for kind in catalogue["kinds"]})
    found: set[str] = set()
    _items(value, found)
    return names, found


def _inspected(rendered: Rendered, item: str) -> str | None:
    """What `inspect` shows for an item: its kind's name, else the loose object's tags."""
    kind = catalogued(rendered, EntityId(item))
    if kind is not None:
        return kind.name
    tags = rendered.loose.get(EntityId(item))
    return ", ".join(tags) if tags else None


def test_there_are_fixtures_with_goods() -> None:
    assert WORLDS, f"no item fixtures under {FIXTURES}"
    named = {item for world in WORLDS for v in _observations(world) for item in _expected(v)[1]}
    assert named, "the fixtures name no item"


@pytest.mark.parametrize("world", WORLDS)
def test_every_disclosed_item_has_its_name_in_the_view(world: str) -> None:
    for n, value in enumerate(_observations(world)):
        names, goods = _expected(value)
        refs = RefTable()
        rendered = render(Observation.model_validate_json(json.dumps(value)), refs, 1, True)
        for item in sorted(goods):
            assert item in names, (
                f"{world} #{n}: item {item} is named nowhere (the server half fails too)"
            )
            assert _inspected(rendered, item) == names[item], f"{world} #{n}: inspect({item})"
        by_ref = {refs.ref(EntityId(item)): names[item] for item in goods}
        for count in [
            *rendered.me.holdings,
            *(rendered.me.job.produces if rendered.me.job else []),
        ]:
            expected = by_ref.get(count.item)
            assert expected is not None, f"{world} #{n}: {count.item} is no disclosed good"
            assert count.name is not None and count.name.text == expected, f"{world} #{n}: {count}"
        for offer in rendered.offers:
            for ref, name in zip(offer.about, offer.about_names, strict=True):
                if ref in by_ref:
                    assert name is not None and name.text == by_ref[ref], (
                        f"{world} #{n}: offer {offer.n}"
                    )


def test_a_buy_offer_names_its_good() -> None:
    # The promo's case: at least one fixture is in a shop, and every `buy` there names what it sells.
    buys = 0
    for world in WORLDS:
        for value in _observations(world):
            rendered = render(
                Observation.model_validate_json(json.dumps(value)), RefTable(), 1, True
            )
            for offer in rendered.offers:
                if offer.action_type == "buy":
                    buys += 1
                    assert offer.about_names and all(offer.about_names), f"{world}: {offer}"
    assert buys > 0, "no fixture stands in a shop"
