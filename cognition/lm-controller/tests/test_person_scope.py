"""AC-c-6: the person scope's parameters shape its sections, against oracles from the facts (pr-s28c §5.4)."""

from __future__ import annotations

from memory_fixtures import ALICE, BOB, CAFE, HOUR, named, seat_view, spoke
from mineworld_sdk.wire.ids import EntityKey, JsonValue

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.retrieve import MemorySection, RecallQuery, retrieve
from mineworld_cognition.memory.scopes.host import recall
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, resolve_stack
from mineworld_cognition.memory.scopes.view import BaseView

# Day 0: twenty exchanges, one every 10 minutes from 08:00; the query is at 12:00 + 1 s.
FACTS = [named(1, BOB, "Bob")] + [
    spoke(
        10 + n,
        8 * HOUR + 600 * n,
        BOB if n % 2 else ALICE,
        ALICE if n % 2 else BOB,
        f"Line {n}.",
        CAFE,
    )
    for n in range(20)
]
AT = {fact.id: fact.at for fact in FACTS}
QUERY = RecallQuery(at=12 * HOUR + 1)


def _recalled(**parameters: JsonValue) -> MemorySection:
    choice = MemoryPolicyChoice(
        policy="consolidate", scopes=(MemoryScopeChoice(id="person", parameters=parameters),)
    )
    stack = resolve_stack(choice, ScopeRegistry.installed(), seat=EntityKey("alice"))
    with seat_view() as view:
        ingest(view, FACTS, FACTS[-1].id)
        return recall(view, stack, QUERY, on_defect="raise").section


def _of(section: MemorySection, level: str) -> list[str]:
    return [line.render() for line in section.lines if line.level == level]


def test_fewer_l0_lines_shrink_only_l0() -> None:
    default = _recalled()
    shrunk = _recalled(sections={"L0": {"lines": 5, "bytes": 1000}})
    assert len(_of(default, "L0")) > 5
    assert _of(shrunk, "L0") == _of(default, "L0")[:5]
    for level in ("with", "L3", "L2", "L1"):
        assert _of(shrunk, level) == _of(default, level)


def test_the_l0_window_keeps_only_the_hour_before_the_query() -> None:
    lines = _recalled(l0_window_s=3600).lines
    shown = [line for line in lines if line.level == "L0"]
    assert shown
    for line in shown:
        for cited in line.citation.ranges:
            assert QUERY.at - 3600 < AT[cited.first] <= QUERY.at
    # The oracle's count: facts in (11:00:01, 12:00:01], that is 11:10 … 12:00 at 10-minute steps.
    assert len(shown) == sum(1 for fact in FACTS[1:] if QUERY.at - 3600 < fact.at <= QUERY.at)


def test_the_default_parameters_recall_what_retrieve_recalls() -> None:
    with seat_view() as view:
        ingest(view, FACTS, FACTS[-1].id)
        plain = retrieve(view, QUERY).render()
    assert _recalled().render() == plain


def test_the_account_sums_to_the_seats_meter() -> None:
    # F-S28c-16: the person scope's account is the seat's content, table by table.
    with seat_view() as view:
        ingest(view, FACTS, FACTS[-1].id)
        account = PersonScope().account(BaseView(view, "person"))
        assert account.total() == view.content_size()
        assert [table for table, _ in account.tables][:3] == ["names", "l0", "l0_who"]
