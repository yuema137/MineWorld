"""`anchors_of` (pr-s28d §5.7): the places, items and organizations a person is with now, from what the world
disclosed in one observation, for two worlds' shapes. The scope-change log line is checked at the milestone
with a real seat runner (ledger D-d-2)."""

from __future__ import annotations

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_cognition.controller.log import ScopeChangeRecord
from mineworld_cognition.controller.situation import QUERY_ANCHORS, anchors_of
from mineworld_cognition.memory.anchors import Anchor


def _entity(entity: str, kind: str) -> JsonValue:
    return {"id": entity, "entity_type": kind, "location": None, "tags": [], "components": []}


def _observation(place: str | None, entities: list[JsonValue]) -> Observation:
    return Observation.model_validate(
        {
            "observer": "7000000001",
            "at": 100,
            "self_location": None
            if place is None
            else {"place": {"entity": place, "entity_type": "place"}},
            "entities": entities,
            "relations": [],
            "events": [],
            "affordances": [],
        }
    )


def test_the_place_comes_first_then_the_things_in_id_order_without_persons() -> None:
    seen = _observation(
        "5200000001",
        [
            _entity("5100000002", "person"),
            _entity("5200000001", "place"),
            _entity("5300000001", "item"),
            _entity("5400000001", "organization"),
        ],
    )
    assert anchors_of(seen) == (
        Anchor(EntityId("5200000001"), "place"),
        Anchor(EntityId("5300000001"), "item"),
        Anchor(EntityId("5400000001"), "organization"),
    )


def test_nowhere_and_many_things() -> None:
    many = [_entity(str(9000000000 + n), "item") for n in range(QUERY_ANCHORS + 5)]
    found = anchors_of(_observation(None, many))
    assert len(found) == QUERY_ANCHORS and found[0] == Anchor(EntityId("9000000000"), "item")


def test_the_scope_change_line_names_what_changed() -> None:
    line = ScopeChangeRecord(seat="ada", added=("place",), removed=(), rebuilt_rows=3)
    assert line.model_dump_json() == (
        '{"kind":"scope-change","seat":"ada","added":["place"],"removed":[],"rebuilt_rows":3}'
    )
