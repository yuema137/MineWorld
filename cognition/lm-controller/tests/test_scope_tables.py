"""AC-d-4, AC-d-5, AC-d-9's host half: the scope-table host (pr-s28d §5.4).

The fixture scope `Tally` keeps one row per entity a fact's envelope names; the arm is `generated` (random ids,
event types and payloads). Every expected row is a fold over the facts written here, never read from the code
under test.
"""

from __future__ import annotations

import json
from collections.abc import Generator, Sequence
from typing import ClassVar

import pytest
from anchor_streams import generated
from memory_fixtures import ALICE, BOB, CAROL, INSTANCE
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from scope_fixtures import TALLY_COLUMNS, Hostile, Tally

from mineworld_cognition.memory.home import (
    MemoryHome,
    ScopeRowRefused,
    ScopeTableMismatch,
    SeatView,
)
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity, event_key
from mineworld_cognition.memory.scopes import Column, TableSpec
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack
from mineworld_cognition.memory.scopes.world import WorldScope

SEAT = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)
REGISTRY = ScopeRegistry(
    {"person": PersonScope, "world": WorldScope, "tally": Tally, "hostile": Hostile},
    base=("person", "world"),
)
FACTS, _ = generated(5, ALICE, 300, others=(BOB, CAROL))


def stack(*optional: tuple[str, dict[str, int]]) -> MemoryStack:
    bound = [REGISTRY.bind("person", {})]
    bound += [REGISTRY.bind(scope, dict(params)) for scope, params in optional]
    return MemoryStack(seat=SEAT.seat, scopes=tuple(bound))


def fold(facts: Sequence[PerceivedEvent]) -> list[list[object]]:
    """The expected `tally_seen` rows, in key order, as dumped (keys as numbers)."""
    rows: dict[str, list[object]] = {}
    for one in facts:
        for entity in dict.fromkeys([*one.participants, *one.subjects]):
            row = rows.get(entity)
            if row is None:
                rows[entity] = [entity, int(one.id), int(one.id), one.at, one.at, 1]
            else:
                row[2], row[4] = int(one.id), one.at
                row[5] = int(str(row[5])) + 1
    return [rows[entity] for entity in sorted(rows)]


def lines(view: SeatView, table: str) -> list[list[object]]:
    return [
        json.loads(line)[1:]
        for line in view.dump().decode().splitlines()
        if line.startswith(f'["{table}",')
    ]


@pytest.fixture
def home() -> Generator[MemoryHome]:
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        yield opened


def test_rows_merge_by_first_last_and_count(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    synced = home.bind_scopes(view, stack(("tally", {})))
    assert (synced.added, synced.removed, synced.rebuilt_rows) == (("tally",), (), 0)
    ingest(view, FACTS, FACTS[-1].id)
    assert lines(view, "tally_seen") == fold(FACTS)
    meta = sum(size for _, size in view.row_bytes("meta"))
    assert view.content_size() == len(view.dump()) - meta


def test_tables_are_created_once_and_a_changed_declaration_is_refused(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {})))
    again = home.bind_scopes(view, stack(("tally", {})))
    assert (again.added, again.removed) == ((), ())
    registered = home.connection.execute("SELECT name, scope FROM scope_tables").fetchall()
    assert registered == [("tally_seen", "tally")]

    class Changed(Tally):
        tables: ClassVar[tuple[TableSpec, ...]] = (
            TableSpec("tally_seen", (*TALLY_COLUMNS, Column("more", "int", "count")), "seat"),
        )

    changed = ScopeRegistry({"person": PersonScope, "tally": Changed}, base=("person",))
    other = home.seat(
        StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=BOB)
    )
    bound = (changed.bind("person", {}), changed.bind("tally", {}))
    with pytest.raises(ScopeTableMismatch) as raised:
        home.bind_scopes(other, MemoryStack(seat=EntityKey("bob"), scopes=bound))
    assert (raised.value.scope, raised.value.table) == ("tally", "tally_seen")


def test_the_cap_evicts_the_least_recently_seen(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {"most": 2})))
    ingest(view, FACTS, FACTS[-1].id)
    kept = sorted(fold(FACTS), key=lambda row: (int(str(row[4])), str(row[0])))[-2:]
    got = lines(view, "tally_seen")
    assert len(got) == 2
    # A row evicted and seen again restarts its counts, so only the identity of the survivors is fixed.
    assert {str(row[0]) for row in got} == {str(row[0]) for row in kept}


def test_remove_seat_leaves_no_scope_row(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {})))
    ingest(view, FACTS, FACTS[-1].id)
    home.remove_seat(SEAT.seat)
    assert home.connection.execute("SELECT count(*) FROM tally_seen").fetchone()[0] == 0
    assert home.connection.execute("SELECT count(*) FROM scope_seats").fetchone()[0] == 0


def test_the_home_dump_writes_the_scope_rows_as_the_seat_view_does(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {})))
    ingest(view, FACTS, FACTS[-1].id)
    mine = [line for line in view.dump().decode().splitlines() if line.startswith('["tally_seen"')]
    whole = [line for line in home.dump().decode().splitlines() if line.startswith('["tally_seen"')]
    assert mine == whole and mine


def _bool_row(fact: object) -> object:
    from mineworld_cognition.memory.scopes import ScopeRow

    key = event_key(fact.event)  # type: ignore[attr-defined]
    # The carrier is strict; a scope can still build one unchecked, which the home must refuse.
    return ScopeRow.model_construct(table="hostile_seen", cells=("1", key, key, True, 0, 1))


def _other_key_row(fact: object) -> object:
    from mineworld_cognition.memory.scopes import ScopeRow

    return ScopeRow(table="hostile_seen", cells=("1", "00000000000000000001", "x", 0, 0, 1))


@pytest.mark.parametrize(
    ("planted", "why"),
    [
        ("foreign", "is not one of the scope's own tables"),
        ("short", "has 2 cells for 6 columns"),
        ("bool", "holds a non-integer"),
        ("key", "cites in key column"),
    ],
)
def test_a_hostile_row_is_refused_and_the_ingest_rolls_back(
    home: MemoryHome, planted: str, why: str
) -> None:
    from mineworld_cognition.memory.scopes import ScopeRow

    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("hostile", {})))
    ingest(view, FACTS[:10], FACTS[9].id)
    before = home.dump()
    Hostile.rows[:] = [
        {
            "foreign": ScopeRow(table="tally_seen", cells=()),
            "short": ScopeRow(table="hostile_seen", cells=("1", "2")),
            "bool": _bool_row,
            "key": _other_key_row,
        }[planted]
    ]
    try:
        with pytest.raises(ScopeRowRefused) as raised:
            ingest(view, FACTS[10:20], FACTS[19].id)
    finally:
        Hostile.rows.clear()
    assert raised.value.scope == "hostile"
    assert raised.value.why.startswith(why), raised.value.why
    assert home.dump() == before


def test_removing_drops_rows_and_re_adding_rebuilds_them(home: MemoryHome) -> None:
    # AC-d-9's host half (keep-all: nothing released, so the rebuild equals an uninterrupted run).
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {})))
    ingest(view, FACTS[:150], FACTS[149].id)
    removed = home.bind_scopes(view, stack())
    assert removed.removed == ("tally",)
    assert lines(view, "tally_seen") == []
    ingest(view, FACTS[150:], FACTS[-1].id)
    rebuilt = home.bind_scopes(view, stack(("tally", {})))
    assert rebuilt.added == ("tally",) and rebuilt.rebuilt_rows > 0
    assert lines(view, "tally_seen") == fold(FACTS)
    meta = sum(size for _, size in view.row_bytes("meta"))
    assert view.content_size() == len(view.dump()) - meta


def test_changed_parameters_drop_and_rebuild(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    home.bind_scopes(view, stack(("tally", {})))
    ingest(view, FACTS, FACTS[-1].id)
    synced = home.bind_scopes(view, stack(("tally", {"most": 5000})))
    assert (synced.added, synced.removed) == (("tally",), ("tally",))
    assert lines(view, "tally_seen") == fold(FACTS)


def test_a_home_no_seat_binds_is_unchanged(home: MemoryHome) -> None:
    view = home.seat(SEAT)
    ingest(view, FACTS[:50], FACTS[49].id)
    before = home.dump()
    synced = home.bind_scopes(view, stack())
    assert not synced.changed()
    assert home.dump() == before
