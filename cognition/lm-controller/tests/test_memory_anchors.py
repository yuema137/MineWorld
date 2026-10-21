"""AC-d-2: payload anchors come from the contract's typed references only (pr-s28d §5.2).

The oracle is `anchor_oracle.scan`, written from §5.2's rules without importing the code under test.
"""

from __future__ import annotations

import random
from dataclasses import replace

import anchor_oracle
import pytest
from anchor_streams import ROD, payload
from memory_fixtures import ALICE, BOB, CAFE, INSTANCE, PARK, fact, walk_started
from mineworld_sdk.wire.ids import EntityId, EntityKey, JsonValue

from mineworld_cognition.memory.anchors import ANCHORS_PER_FACT, Anchor, payload_anchors
from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.render import RENDERERS, render


def _with(body: JsonValue, event_type: str = "something-new") -> list[Anchor]:
    one = fact(9, 100, event_type, body, subjects=[ALICE], participants=[ALICE], place=CAFE)
    return list(payload_anchors(one))


def _ref(entity: str, kind: str) -> JsonValue:
    return {"entity": entity, "entity_type": kind}


def test_the_walks_nested_destination_is_one_place_and_the_walker_is_not() -> None:
    walk = walk_started(5, 100, ALICE, PARK)
    assert payload_anchors(walk) == (Anchor(PARK, "place"),)


def test_lakesides_rod_is_an_item() -> None:
    assert _with({"catch": "trout", "rod": _ref(ROD, "item")}, "fished") == [Anchor(ROD, "item")]


def test_items_transferred_gives_its_item_not_its_persons() -> None:
    body = {"from": _ref(ALICE, "person"), "to": _ref(BOB, "person"), "item": _ref("77", "item")}
    assert _with(body) == [Anchor(EntityId("77"), "item")]


def test_keys_are_visited_in_sorted_order_and_lists_in_order() -> None:
    body: JsonValue = {
        "zeta": _ref("3", "place"),
        "alpha": [_ref("2", "item"), _ref("1", "organization")],
    }
    assert [a.entity for a in _with(body)] == ["2", "1", "3"]


def test_the_first_sixteen_distinct_anchors_are_kept() -> None:
    body: JsonValue = {
        "all": [_ref(str(100 + n), "place") for n in range(ANCHORS_PER_FACT + 1)] * 2
    }
    kept = _with(body)
    assert [a.entity for a in kept] == [str(100 + n) for n in range(ANCHORS_PER_FACT)]


@pytest.mark.parametrize(
    "body", [None, 3, "text", True, [], {}, {"entity": "x", "entity_type": "place"}]
)
def test_no_reference_gives_nothing(body: JsonValue) -> None:
    assert _with(body) == []


def test_an_object_with_more_keys_is_walked_not_taken() -> None:
    body = {"entity": "5", "entity_type": "place", "inner": _ref("6", "item")}
    assert _with(body) == [Anchor(EntityId("6"), "item")]


def test_render_carries_the_anchors_and_keeps_the_gist() -> None:
    walk = walk_started(5, 100, ALICE, PARK)
    with MemoryHome.open(":memory:", INSTANCE) as home:
        view = home.seat(
            StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=BOB)
        )
        names = NameBook.load(view)
        rendered = render(walk, BOB, names)
        plain = RENDERERS[walk.event_type](walk, BOB, names)
    assert rendered.anchors == (Anchor(PARK, "place"),)
    assert replace(rendered, anchors=()) == plain


def test_five_hundred_generated_payloads_agree_with_the_oracle() -> None:
    rng = random.Random(28)
    for _ in range(500):
        body = payload(rng)
        got = [(a.entity, a.kind) for a in _with(body)]
        assert got == anchor_oracle.scan(body)
        assert all(kind != "person" for _, kind in got)
