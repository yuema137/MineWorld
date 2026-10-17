"""The recorded market-town observations (`fixtures/`, `fixtures/SOURCE.md`), raw and decoded, and a
synthetic crowd built from them for paging.

Expected values in the view tests are read from the raw JSON here, never from `view.py`'s output (test
rules §25).
"""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

from mineworld_sdk.wire.contract import Observation

FIXTURES = Path(__file__).resolve().parent / "fixtures"
NAMES = ("market-town-start", "market-town-walking", "market-town-street")

type Raw = dict[str, Any]


def raw(name: str) -> Raw:
    value: Raw = json.loads((FIXTURES / f"{name}.json").read_text(encoding="utf-8"))
    return value


def decoded(value: Raw) -> Observation:
    return Observation.model_validate_json(json.dumps(value))


def observation(name: str) -> Observation:
    return decoded(raw(name))


def component(entity: Raw, kind: str) -> Any:  # noqa: ANN401 (a pack's raw JSON, read by the test)
    return next((c["payload"] for c in entity["components"] if c["component_type"] == kind), None)


def entity(value: Raw, entity_id: str) -> Raw:
    return next(e for e in value["entities"] if e["id"] == entity_id)


def crowd(people: int, things: int, offers: int, *, name: str = "Guest") -> Raw:
    """The start observation with `people` more persons, `things` items and `offers` more `talk` offers
    (to the new persons, round robin), each with fresh ids above anything the town allocates."""
    value = raw("market-town-start")
    template = entity(value, "7")
    talk = next(a for a in value["affordances"] if a["action_type"] == "talk")
    made: list[str] = []
    for n in range(people):
        person = copy.deepcopy(template)
        person["id"] = str(10_000 + n)
        person["components"] = [
            {
                "entity": person["id"],
                "component_type": "display-name",
                "schema_version": 1,
                "payload": {"name": f"{name} {n}"},
            }
        ]
        value["entities"].append(person)
        made.append(person["id"])
    for n in range(things):
        value["entities"].append(
            {
                "id": str(20_000 + n),
                "entity_type": "item",
                "location": None,
                "tags": [],
                "components": [],
            }
        )
    for n in range(offers):
        offer = copy.deepcopy(talk)
        offer["target"] = made[n % len(made)] if made else talk["target"]
        value["affordances"].append(offer)
    return value
