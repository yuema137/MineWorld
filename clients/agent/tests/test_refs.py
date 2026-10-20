"""`RefTable`: references in first-seen order, stable for a handle's life, and nothing else resolves."""

from __future__ import annotations

import pytest
from mineworld_sdk.wire.ids import EntityId

from mineworld_agent.refs import RefTable, UnknownRef


def test_references_are_assigned_in_first_seen_order_and_never_change() -> None:
    table = RefTable()
    seen = [EntityId(i) for i in ("17", "2", "9007199254740995", "2", "17", "8")]
    refs = [table.ref(entity) for entity in seen]
    assert refs == ["E1", "E2", "E3", "E2", "E1", "E4"]
    assert [table.resolve(r) for r in ("E1", "E2", "E3", "E4")] == [
        "17",
        "2",
        "9007199254740995",
        "8",
    ]


def test_a_reference_never_shown_is_unknown() -> None:
    table = RefTable()
    table.ref(EntityId("17"))
    for never in ("E2", "E0", "17", "e1", ""):
        with pytest.raises(UnknownRef):
            table.resolve(never)


def test_two_tables_are_independent() -> None:
    # I-c-2 at the unit level: a second handle's table starts again at E1 and knows nothing of the first.
    first, second = RefTable(), RefTable()
    first.ref(EntityId("7"))
    first.ref(EntityId("8"))
    assert second.ref(EntityId("8")) == "E1"
    with pytest.raises(UnknownRef):
        second.resolve("E2")
