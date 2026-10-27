"""Place labels in the prompt come from what the world discloses (D-P6-43; pr-s10-p6 §5.2.2).

No shipped pack gives a place a display name: a place is disclosed by its tags, its own to whoever stands in
it and each doorway's `to_tags` to whoever stands beside it (ARC-82). These tests read a real observation the
server sent (`clients/agent/tests/fixtures/market-town-street.json`, recorded by `record.py`, see its
`SOURCE.md`) and observations built in the same wire form for two other worlds' tags (social-cafe, lakeside).
Expected labels are literals or read from the recorded file, never from the code under test.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from memory_fixtures import ALICE, CAFE, PARK, open_view
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.controller import Situation
from mineworld_cognition.controller.situation import Place
from mineworld_cognition.memory.names import NameBook

REPOSITORY = Path(__file__).resolve().parents[3]
STREET = REPOSITORY / "clients" / "agent" / "tests" / "fixtures" / "market-town-street.json"
SHORE = EntityId("5000000003")


def _place(
    entity: EntityId, tags: list[str], doors: list[tuple[EntityId, list[str]]]
) -> dict[str, Any]:
    components: list[dict[str, Any]] = []
    if doors:
        components.append(
            {
                "entity": entity,
                "component_type": "passages",
                "schema_version": 1,
                "payload": {
                    "leads_to": [
                        {
                            "to": {"entity": to, "entity_type": "place"},
                            "here": None,
                            "there": None,
                            "to_tags": to_tags,
                        }
                        for to, to_tags in doors
                    ]
                },
            }
        )
    return {
        "id": entity,
        "entity_type": "place",
        "location": None,
        "tags": tags,
        "components": components,
    }


def _named(entity: EntityId, name: str) -> dict[str, Any]:
    return {
        "entity": entity,
        "component_type": "display-name",
        "schema_version": 1,
        "payload": {"name": name},
    }


def _ways(places: list[EntityId]) -> dict[str, Any]:
    return {
        "entity": ALICE,
        "component_type": "ways",
        "schema_version": 1,
        "payload": [
            {"place": {"entity": place, "entity_type": "place"}, "doors": 1, "length": 4000}
            for place in places
        ],
    }


def _observation(here: EntityId, places: list[dict[str, Any]], ways: list[EntityId]) -> Observation:
    me = {
        "id": ALICE,
        "entity_type": "person",
        "location": {
            "place": {"entity": here, "entity_type": "place"},
            "local": None,
            "facing": None,
        },
        "tags": [],
        "components": [_named(ALICE, "Alice"), _ways(ways)],
    }
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": 30_000,
            "self_location": me["location"],
            "entities": [*places, me],
            "relations": [],
            "events": [],
            "affordances": [],
        }
    )


def test_the_recorded_street_names_its_own_place_and_every_doorway_by_their_tags() -> None:
    raw = json.loads(STREET.read_text(encoding="utf-8"))
    observation = Observation.model_validate(raw)
    here = raw["self_location"]["place"]["entity"]
    street = next(entity for entity in raw["entities"] if entity["id"] == here)
    passages = next(c for c in street["components"] if c["component_type"] == "passages")
    expected = {here: "/".join(street["tags"])} | {
        door["to"]["entity"]: "/".join(door["to_tags"]) for door in passages["payload"]["leads_to"]
    }
    assert len(expected) == 6  # the street and its five doorways (SOURCE.md)
    names = NameBook.load(open_view())
    situation = Situation.of(observation, names)
    assert situation.place == "public/street"
    assert {entity: names.place(EntityId(entity)) for entity in expected} == expected
    assert "somewhere" not in expected.values()


def test_a_cafe_world_labels_its_place_and_the_places_a_go_may_name() -> None:
    street = EntityId("5000000009")
    cafe = _place(CAFE, ["cafe", "public"], [(street, ["public", "street"])])
    observation = _observation(CAFE, [cafe], [street])
    situation = Situation.of(observation, NameBook.load(open_view()))
    assert situation.place == "cafe/public"
    assert situation.places == (Place(label="public/street", entity=street),)


def test_a_lakeside_world_gets_its_own_tags_and_equal_tags_stay_unique() -> None:
    """A second world's vocabulary, with two doorways to places tagged alike: labels are made unique as
    person labels are, so a `go` still names exactly one place."""
    lane = _place(
        PARK,
        ["lane", "outdoor", "public"],
        [(CAFE, ["bakery", "indoor"]), (SHORE, ["bakery", "indoor"])],
    )
    observation = _observation(PARK, [lane], [CAFE, SHORE])
    situation = Situation.of(observation, NameBook.load(open_view()))
    assert situation.place == "lane/outdoor/public"
    assert [place.label for place in situation.places] == ["bakery/indoor", "bakery/indoor (2)"]


def test_a_display_name_wins_over_tags_and_a_tag_label_never_replaces_a_name() -> None:
    view = open_view()
    named = _place(CAFE, ["cafe", "public"], [])
    named["components"].append(_named(CAFE, "The Blue Door"))
    Situation.of(_observation(CAFE, [named], []), NameBook.load(view))
    later = Situation.of(
        _observation(CAFE, [_place(CAFE, ["cafe", "public"], [])], []), NameBook.load(view)
    )
    assert later.place == "The Blue Door"


def test_a_place_never_perceived_stays_somewhere_and_a_learned_label_is_remembered() -> None:
    """A way discloses only an id, so a place not yet seen is `somewhere`; once its tags were perceived, the
    seat's name book keeps the label after it walked away."""
    view = open_view()
    street = EntityId("5000000009")
    at_cafe = _observation(
        CAFE, [_place(CAFE, ["cafe", "public"], [(street, ["public", "street"])])], [street, SHORE]
    )
    assert [p.label for p in Situation.of(at_cafe, NameBook.load(view)).places] == [
        "public/street",
        "somewhere",
    ]
    at_shore = _observation(SHORE, [_place(SHORE, ["outdoor", "shore"], [])], [CAFE])
    assert [p.label for p in Situation.of(at_shore, NameBook.load(view)).places] == ["cafe/public"]
