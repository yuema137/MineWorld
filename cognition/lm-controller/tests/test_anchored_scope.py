"""The anchored scope in isolation (pr-s28d §5.6): `index`, `recall` and its budgets, its parameters.

The arm is `lakeside-like`; the expected counts and lines are written from the facts the test feeds.
"""

from __future__ import annotations

from collections.abc import Generator

import pytest
from anchor_streams import BOATHOUSE, LAKE_PEOPLE, LAKE_SEATS, PIER, ROD, THING_NAMES
from memory_fixtures import INSTANCE, fact
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.anchors import Anchor
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity, event_key
from mineworld_cognition.memory.retention.registry import JsonObject
from mineworld_cognition.memory.retrieve import LINE_BYTES
from mineworld_cognition.memory.scopes import FactRecord, MemoryScope, Reading, RecallQuery
from mineworld_cognition.memory.scopes.anchored import (
    ItemParameters,
    ItemScope,
    PlaceParameters,
    PlaceScope,
)
from mineworld_cognition.memory.scopes.errors import ScopeParameterError
from mineworld_cognition.memory.scopes.host import recall
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack

PLACE: MemoryScope[PlaceParameters] = PlaceScope()
ITEM: MemoryScope[ItemParameters] = ItemScope()
ME, BRAM = LAKE_PEOPLE[0], LAKE_PEOPLE[1]
HOUR = 3600


def _fished(event: int, at: int, place: EntityId = PIER, who: EntityId = BRAM) -> PerceivedEvent:
    return fact(
        event,
        at,
        "fished",
        {"catch": "trout", "rod": {"entity": ROD, "entity_type": "item"}},
        subjects=[who],
        participants=list(dict.fromkeys([who, ME])),
        place=place,
    )


# Alone at the boathouse: no stable fact of a person cites fact 30, so the place section may.
FACTS = [_fished(10 + n, 8 * HOUR + n * HOUR) for n in range(5)] + [
    _fished(30, 2 * 86_400 + 9 * HOUR, BOATHOUSE, ME)
]


def _stack(**sections: int) -> MemoryStack:
    registry = ScopeRegistry.installed()
    params: JsonObject = {"section_lines": 4, "section_bytes": sections.get("bytes", 600)}
    scopes = (
        registry.bind("person", {}),
        registry.bind("place", params),
        registry.bind("item", params),
    )
    return MemoryStack(seat=LAKE_SEATS[0], scopes=scopes)


@pytest.fixture
def view() -> Generator[SeatView]:
    with MemoryHome.open(":memory:", INSTANCE) as home:
        seat = home.seat(StoreIdentity(world_instance=INSTANCE, seat=LAKE_SEATS[0], self_entity=ME))
        home.bind_scopes(seat, _stack())
        ingest(seat, FACTS, FACTS[-1].id)
        with seat.transaction():
            names = NameBook.load(seat)
            for entity, name in THING_NAMES.items():
                if entity != BOATHOUSE:
                    names.learn(entity, name)
        yield seat


def test_index_is_pure_and_cites_only_the_fact() -> None:
    record = FactRecord.model_validate(
        {"event": "12", "at": 5, "kind": "fished", "place": PIER, "who": ((BRAM, 3),)}
    )
    reading = Reading(
        gist="g", salience="notable", mine=False, importance=1, anchors=(Anchor(ROD, "item"),)
    )
    key = event_key(EventId("12"))
    place = PLACE.index(record, reading, PlaceParameters())
    assert place == PLACE.index(record, reading, PlaceParameters())
    assert [row.cells for row in place] == [(PIER, key, key, 5, 5, 1)]
    item = ITEM.index(record, reading, ItemParameters())
    assert [row.cells for row in item] == [(ROD, key, key, 5, 5, 1)]


def test_place_and_item_recall_what_i_know_of_them(view: SeatView) -> None:
    query = RecallQuery(
        at=3 * 86_400 + 10 * HOUR,
        anchors=(Anchor(PIER, "place"), Anchor(BOATHOUSE, "place"), Anchor(ROD, "item")),
    )
    section = recall(view, _stack(), query, on_defect="raise").section
    place = [line.text for line in section.lines if line.level == "place"]
    item = [line.text for line in section.lines if line.level == "item"]
    assert place[0] == "the pier: 5 moments I remember, Day 1 to Day 1"
    assert place[1] == "somewhere: 1 moments I remember, Day 3 to Day 3"  # no name: never an id
    # Every fact the rod is anchored to is already cited above (by L3 or the place lines), so the item
    # section repeats none of them.
    assert item == []
    rendered = section.render()
    assert "Places I know:" in rendered
    assert BOATHOUSE not in rendered and ROD not in rendered
    alone = RecallQuery(at=query.at, anchors=(Anchor(ROD, "item"),))
    things = recall(view, _stack(), alone, on_defect="raise").section
    item = [line.text for line in things.lines if line.level == "item"]
    assert item[0] == "a fishing rod: 6 moments I remember, Day 1 to Day 3"
    assert "Things I know:" in things.render()
    for line in section.lines:
        assert len(line.render().encode()) <= LINE_BYTES
    # every place gist line is about a fact not shown by an earlier line
    cited = [r.first for line in section.lines for r in line.citation.ranges]
    assert len(cited) == len(set(cited))


def test_the_section_keeps_to_its_bytes(view: SeatView) -> None:
    query = RecallQuery(at=3 * 86_400 + 10 * HOUR, anchors=(Anchor(PIER, "place"),))
    small = _stack(bytes=120)
    section = recall(view, small, query, on_defect="raise").section
    used = len("Places I know:\n") + sum(
        len(line.render().encode()) + 1 for line in section.lines if line.level == "place"
    )
    assert used <= 120


def test_the_default_section_is_off(view: SeatView) -> None:
    registry = ScopeRegistry.installed()
    stack = MemoryStack(
        seat=LAKE_SEATS[0], scopes=(registry.bind("person", {}), registry.bind("place", {}))
    )
    query = RecallQuery(at=3 * 86_400 + 10 * HOUR, anchors=(Anchor(PIER, "place"),))
    assert not [
        line
        for line in recall(view, stack, query, on_defect="raise").section.lines
        if line.level == "place"
    ]


def test_parameters_widen_and_are_checked() -> None:
    registry = ScopeRegistry.installed()
    bound = registry.bind("place", {"roles": ["at", "named"]})
    assert bound.parameters.model_dump()["roles"] == ["at", "named"]
    with pytest.raises(ScopeParameterError):
        registry.bind("item", {"roles": ["at"]})
