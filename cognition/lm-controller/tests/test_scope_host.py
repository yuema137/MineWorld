"""AC-c-4, AC-c-5: the recall host checks every line a scope returns (pr-s28c §5.7, AS28-2, I-S28-1).

A two-seat home: Alice and Bob hold different facts. The hostile fixture scope `Cites` returns planted lines;
the oracle is which events each seat was given, written here, never what the code under test computes.
"""

from __future__ import annotations

from collections.abc import Generator

import pytest
from memory_fixtures import ALICE, BOB, CAFE, DAY, INSTANCE, named, spoke
from mineworld_sdk.wire.ids import EntityKey, EventId
from scope_fixtures import Cites, CitesParameters

from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import Citation, EventRange, StoreIdentity, event_key
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.retrieve import RecallQuery
from mineworld_cognition.memory.scopes import ScopeView, SectionLine
from mineworld_cognition.memory.scopes.errors import (
    AccessDenied,
    ScopeRecallFailed,
    ScopeSectionOverBudget,
)
from mineworld_cognition.memory.scopes.host import recall
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, MemoryStack, resolve_stack
from mineworld_cognition.memory.scopes.view import GuardedView
from mineworld_cognition.memory.scopes.world import WorldScope

HOUR = 3600
REGISTRY = ScopeRegistry(
    {"person": PersonScope, "world": WorldScope, "cites": Cites}, base=("person", "world")
)
QUERY = RecallQuery(at=3 * DAY, counterparts=(BOB,), words="coffee")


ALICE_SEAT = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)
BOB_SEAT = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=BOB)

# Alice: 10, 11 (day 0, one episode), 20 (day 1, released later: forgotten), 30 (day 2).
# Bob alone: 40. Event 99 was perceived by no one.
ALICE_FACTS = [
    named(1, BOB, "Bob"),
    spoke(10, 9 * HOUR, BOB, ALICE, "Coffee?", CAFE),
    spoke(11, 9 * HOUR + 60, ALICE, BOB, "Yes, coffee.", CAFE),
    spoke(20, DAY + 9 * HOUR, BOB, ALICE, "Again coffee.", CAFE),
    spoke(30, 2 * DAY + 9 * HOUR, BOB, ALICE, "Tea today.", CAFE),
]
BOB_FACTS = [named(2, ALICE, "Alice"), spoke(40, 9 * HOUR, ALICE, BOB, "Bob only.", CAFE)]


@pytest.fixture
def home() -> Generator[MemoryHome]:
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        alice = opened.seat(ALICE_SEAT)
        ingest(alice, ALICE_FACTS, ALICE_FACTS[-1].id)
        bob = opened.seat(BOB_SEAT)
        ingest(bob, BOB_FACTS, BOB_FACTS[-1].id)
        with alice.transaction():
            alice.release([event_key(EventId("20"))])
        yield opened
    Cites.reset()


def _stack(*optional: str) -> MemoryStack:
    """`person` with 400 of L0's bytes given up (the room `cites` takes), then `optional`."""
    person = MemoryScopeChoice(
        id="person", parameters={"sections": {"L0": {"lines": 48, "bytes": 600}}}
    )
    scopes = (person, *(MemoryScopeChoice(id=s) for s in optional))
    choice = MemoryPolicyChoice(policy="consolidate", scopes=scopes)
    return resolve_stack(choice, REGISTRY, seat=EntityKey("alice"))


def _line(
    first: str, last: str | None = None, *, key: str = "cites", text: str = "x"
) -> SectionLine:
    cited = Citation(ranges=(EventRange(first=EventId(first), last=EventId(last or first)),))
    return SectionLine(level=key, text=text, citation=cited)


def _alice(home: MemoryHome) -> SeatView:
    return home.seat(ALICE_SEAT)


def test_a_held_key_is_cited_and_rendered_after_the_persons_sections(home: MemoryHome) -> None:
    Cites.lines.append(_line("10"))
    got = recall(_alice(home), _stack("cites"), QUERY, on_defect="raise")
    assert got.defects == ()
    assert got.section.render().endswith("Cited:\nx [#10]\n")


@pytest.mark.parametrize(
    ("first", "last", "denied"),
    [
        ("98", None, "98"),  # (i) a key the seat never held (and no one did)
        ("40", None, "40"),  # (ii) a key only the home-mate holds
        ("20", None, "20"),  # (iii) a key the seat forgot
        ("99", None, "99"),  # (iv) an id never perceived by anyone
    ],
)
def test_a_line_citing_what_the_seat_may_not_cite_is_denied(
    home: MemoryHome, first: str, last: str | None, denied: str
) -> None:
    Cites.lines.append(_line(first, last))
    with pytest.raises(AccessDenied) as raised:
        recall(_alice(home), _stack("cites"), QUERY, on_defect="raise")
    assert (raised.value.scope, raised.value.seat, raised.value.event) == (
        "cites",
        EntityKey("alice"),
        denied,
    )


def test_an_episode_end_is_citable_by_a_base_scope_only(home: MemoryHome) -> None:
    # (v): the day-1 episode's single record (20) was released; the episode survives and still cites it.
    alice = _alice(home)
    ends: list[tuple[str, str]] = alice.connection.execute(
        "SELECT first_key, last_key FROM episodes WHERE seat = ? ORDER BY episode", (alice.seat_id,)
    ).fetchall()
    assert (event_key(EventId("20")), event_key(EventId("20"))) in ends
    unheld = event_key(EventId("20"))
    assert alice.citable([unheld], units=True) == {unheld}
    assert alice.citable([unheld], units=False) == frozenset()
    Cites.lines.append(_line("10", "20"))
    with pytest.raises(AccessDenied) as raised:
        recall(alice, _stack("cites"), QUERY, on_defect="raise")
    assert raised.value.event == "20"


def test_withholding_drops_only_the_defective_scope(home: MemoryHome) -> None:
    alice = _alice(home)
    alone = recall(alice, _stack(), QUERY, on_defect="raise").section
    Cites.lines += [_line("10"), _line("40")]
    got = recall(alice, _stack("cites"), QUERY, on_defect="withhold")
    assert [(d.scope, d.seat) for d in got.defects] == [("cites", EntityKey("alice"))]
    assert got.defects[0].error.startswith("AccessDenied")
    assert all(line.level != "cites" for line in got.section.lines)
    assert got.section.render() == alone.render()


def test_an_optional_scope_reads_a_guarded_view_with_no_connection(home: MemoryHome) -> None:
    def reads_an_unheld_record(view: ScopeView) -> object:
        assert isinstance(view, GuardedView)
        assert not hasattr(view, "connection") and not hasattr(view, "seat_view")
        assert view.holds(EventId("10")) and not view.holds(EventId("40"))
        fact, reading = view.record(EventId("10"))
        assert (fact.event, reading.gist != "") == ("10", True)
        assert view.held_between(HOUR, 3 * DAY, 10) == (EventId("30"), EventId("11"), EventId("10"))
        return view.record(EventId("40"))

    Cites.reads.append(reads_an_unheld_record)
    with pytest.raises(AccessDenied) as raised:
        recall(_alice(home), _stack("cites"), QUERY, on_defect="raise")
    assert raised.value.event == "40"


@pytest.mark.parametrize(
    ("lines", "what"),
    [
        ([_line("10", text="y" * 160)], "line"),
        ([_line("10")] * 5, "lines"),  # its default is 4 lines
        ([_line("10", key="L1")], "section"),  # a key it did not declare
        (
            [_line("10", key="l1")],
            "section",
        ),  # lower case is a scope id, not the person's L1 (R-c-1)
    ],
)
def test_a_line_over_its_budget_or_outside_its_sections_is_refused(
    home: MemoryHome, lines: list[SectionLine], what: str
) -> None:
    Cites.lines += lines
    with pytest.raises(ScopeSectionOverBudget) as raised:
        recall(_alice(home), _stack("cites"), QUERY, on_defect="raise")
    assert raised.value.what == what


def test_a_section_one_byte_over_its_bytes_is_refused() -> None:
    # Header "Cited:\n" (7) + one line "x [#10]\n" (8) = 15 bytes: a 14-byte budget is one over.
    params = CitesParameters(section_lines=4, section_bytes=14)
    assert Cites().sections(params)[0].budget.bytes == 14
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        alice = opened.seat(ALICE_SEAT)
        ingest(alice, ALICE_FACTS, ALICE_FACTS[-1].id)
        Cites.lines.append(_line("10"))
        choice = MemoryPolicyChoice(
            policy="consolidate",
            scopes=(
                MemoryScopeChoice(
                    id="person", parameters={"sections": {"L0": {"lines": 48, "bytes": 986}}}
                ),
                MemoryScopeChoice(id="cites", parameters={"section_bytes": 14}),
            ),
        )
        stack = resolve_stack(choice, REGISTRY, seat=EntityKey("alice"))
        with pytest.raises(ScopeSectionOverBudget) as raised:
            recall(alice, stack, QUERY, on_defect="raise")
        assert (raised.value.what, raised.value.bytes, raised.value.limit) == ("bytes", 15, 14)
    Cites.reset()


def test_a_scope_that_raises_is_a_named_defect(home: MemoryHome) -> None:
    Cites.fails.append(RuntimeError("boom"))
    with pytest.raises(ScopeRecallFailed) as raised:
        recall(_alice(home), _stack("cites"), QUERY, on_defect="raise")
    assert raised.value.error == "RuntimeError"
    got = recall(_alice(home), _stack("cites"), QUERY, on_defect="withhold")
    assert [d.scope for d in got.defects] == ["cites"]
