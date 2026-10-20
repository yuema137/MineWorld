"""AC-e-13 (QS28e-4, deferred by S28-c to S28-d as D-c-11) and R-S28d-e-1 on the `lakeside-like` arm: `inspect
scopes`, per-scope bytes, the trace's scope from `SectionLine.level`, and the seat export with scope rows.
"""

from __future__ import annotations

from collections.abc import Generator

import anchor_streams
import pytest
from memory_fixtures import INSTANCE
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import days, feed, retention

from mineworld_cognition.inspect import BytesRequest, Inputs, RecallRequest, ScopesRequest, report
from mineworld_cognition.inspect.report import BytesReport, RecallReport, ScopesReport
from mineworld_cognition.memory.home import MemoryHome, export_seat, import_seat
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.registry import SourceSpan
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, resolve_stack

SEAT = anchor_streams.LAKE_SEATS[0]
ME = anchor_streams.LAKE_PEOPLE[0]
WHERE = SourceSpan(file="worlds/lakeside/decisions.yaml", line=12, column=9)
CHOICE = MemoryPolicyChoice.model_validate(
    {
        "policy": "keep-all",
        "source": WHERE,
        "scopes": (
            MemoryScopeChoice(
                id="person", parameters={"sections": {"L1": {"lines": 8, "bytes": 400}}}
            ),
            MemoryScopeChoice(id="place", parameters={"section_bytes": 600}, source=WHERE),
            MemoryScopeChoice(id="item", parameters={"section_bytes": 600}),
        ),
    }
)
INPUTS = Inputs(choice=CHOICE)


@pytest.fixture
def home() -> Generator[MemoryHome]:
    given = anchor_streams.streams(anchor_streams.lakeside(11, 3))
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        view = opened.seat(StoreIdentity(world_instance=INSTANCE, seat=SEAT, self_entity=ME))
        opened.bind_scopes(view, resolve_stack(CHOICE, ScopeRegistry.installed(), seat=SEAT))
        feed(view, days(given[SEAT]), retention("keep-all"))
        yield opened


def test_scopes_lists_the_installed_the_stack_and_the_tables(home: MemoryHome) -> None:
    found = report(home, ScopesRequest(seat=SEAT), INPUTS)
    assert isinstance(found, ScopesReport)
    assert [(s.id, s.distribution) for s in found.installed] == [
        (scope, "mineworld-cognition") for scope in ("item", "person", "place", "world")
    ]
    assert [s.id for s in found.stack] == ["person", "place", "item"]
    assert found.stack[1].source == str(WHERE) and found.stack_source == "given"
    assert {(t.table, t.scope, t.named) for t in found.tables} == {
        ("item_known", "item", True),
        ("place_known", "place", True),
    }
    alone = report(home, ScopesRequest(seat=SEAT), Inputs())
    assert isinstance(alone, ScopesReport)
    assert alone.stack_source == "default" and not any(t.named for t in alone.tables)


def test_bytes_are_attributed_by_scope_and_sum_to_the_meter(home: MemoryHome) -> None:
    found = report(home, BytesRequest(seat=SEAT), INPUTS)
    assert isinstance(found, BytesReport) and found.seat is not None
    by_scope: dict[str, int] = {}
    for table in found.seat.tables:
        if table.table != "meta":
            by_scope[table.scope] = by_scope.get(table.scope, 0) + table.bytes
    assert set(by_scope) == {"person", "place", "item"}
    assert sum(by_scope.values()) == found.seat.content_size


def test_the_trace_names_each_lines_scope(home: MemoryHome) -> None:
    request = RecallRequest(seat=SEAT, at=3 * 86_400, trace=True)
    found = report(home, request, INPUTS)
    assert isinstance(found, RecallReport)
    assert found.stack == ("person", "place", "item")
    assert all(
        line.scope == ("person" if line.level in ("with", "L3", "L2", "L1", "L0") else line.level)
        for line in found.trace
    )


def test_the_export_carries_scope_rows_and_an_import_rebuilds_them(home: MemoryHome) -> None:
    exported = export_seat(home, SEAT)
    assert b'["place_known",' in exported
    with MemoryHome.open(":memory:", INSTANCE) as fresh:
        assert import_seat(fresh, exported) == EntityKey(SEAT)
        view = fresh.view_of(SEAT)
        fresh.bind_scopes(view, resolve_stack(CHOICE, ScopeRegistry.installed(), seat=SEAT))
        # keep-all released nothing, so the rebuild equals the rows exported (QS28d-5).
        assert view.dump() == home.view_of(SEAT).dump()
