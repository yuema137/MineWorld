"""The bound at the bound in a shared home (pr-s28b C5; AC-b-7, AC-b-8, AC-b-10; step-28 AS28-5, AS28-7, AS28-9;
I-a-6 as ruled, QS28a-1).

- AC-b-7 / AC-b-8: the multi-seat heavy stream (`home_streams.heavy_world`, three seats, every record held by all
  of them but one in seven) is fed, frame by frame in a seeded interleaving, into **one shared home** (verified at
  every commit) and, in lockstep, into **three one-seat homes**. After every frame: each seat view ≤ the bound;
  its dump in the shared home equals its one-seat home's (a home-mate's forced forgetting moves no byte of it,
  AS28-5); I-a-6 (a) the shared home ≤ Σ the one-seat homes; (b) each one-seat home ≤ its view + `K` x held
  records; every fact the shared home stores is held by some seat (AS28-7: the last holder's release collects it,
  in that seat's transaction). After every pass: ≤ the target, and ≤ the bound before it. Two stacks: `keep-all`
  for every seat (AS28-9: the backstop forces every drop), and Alice under `consolidate` among two `keep-all`
  mates (AS28-5 at the bound; her 200 sections equal hers alone at the end).
- AC-b-10: FG-b2's margin (`bound - trigger`) holds a 3 000-participant fact in positional bytes (F-S28a-8).

Every figure is a literal written here from the operator's 20 MiB, never imported from the code under test, except
`K`, which I-a-6 (b) names as the code's own constant (QS28a-1). The heavy runs are `slow` (the nightly
`stability` layer, pr-s28a D-a-15 precedent; pr-s28b D-b-6); the margin runs per PR.
"""

from __future__ import annotations

import json
from collections.abc import Callable, Generator
from contextlib import contextmanager
from dataclasses import dataclass

import pytest
from home_harness import Cast, sections
from home_streams import CAST, HEAVY, STRANGERS, heavy_world, interleave
from memory_fixtures import INSTANCE, fact, named, seat_view
from mineworld_sdk.wire.ids import EntityId, EntityKey
from retention_driver import retention
from retention_stream import CROWD, PLACES
from support import run

from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.home.schema import K_ACCESS_OVERHEAD_BYTES as K
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention import absorb as absorb_module
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.host import PassCause, PassReport, run_pass
from mineworld_cognition.memory.retention.selection import Retention

MIB = 1_048_576
BOUND = 20 * MIB  # 20 971 520: the operator's default (2026-10-14)
TARGET = BOUND * 3 // 4  # 15 728 640
TRIGGER = BOUND - BOUND // 16  # 19 660 800
MARGIN = BOUND - TRIGGER  # 1 310 720: FG-b2 §4.1's room for the crossing fact and `meta`
FRAME = 256
INTERLEAVING = 1
CAST3 = Cast(CAST, INSTANCE, (*CAST.values(), *STRANGERS))


@dataclass(frozen=True)
class Pass:
    seat: EntityKey
    shared: bool
    cause: PassCause
    before: int
    after: int
    forced: int


@contextmanager
def passes(shared: Callable[[SeatView], bool]) -> Generator[list[Pass]]:
    """Every pass `absorb` runs, in either arm, with the seat view's dump length around it."""
    seen: list[Pass] = []

    async def measured(
        store: SeatView, kept: Retention, now: int, cause: PassCause = "day"
    ) -> PassReport:
        before = len(store.dump())
        report = await run_pass(store, kept, now, cause)
        after = len(store.dump())
        seen.append(Pass(store.identity.seat, shared(store), cause, before, after, report.forced))
        return report

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(absorb_module, "run_pass", measured)
        yield seen


def _keys(home: MemoryHome, table: str) -> set[int]:
    return {
        int(cells[1])
        for cells in (json.loads(line) for line in home.dump().decode("utf-8").splitlines())
        if cells[0] == table
    }


def lockstep(policies: dict[EntityKey, str]) -> tuple[list[Pass], dict[EntityKey, tuple[str, str]]]:
    """Runs the heavy world in a shared home and three one-seat homes, checking after every frame (module
    docstring). Returns every pass and, per seat, its sections at the end in both arms."""
    seed, days, per_day, gathered = HEAVY
    streams = heavy_world(seed, days, len(policies), per_day, gathered)
    assert sorted(streams) == sorted(policies)
    kept = {seat: retention(policy) for seat, policy in policies.items()}
    shared_views: dict[EntityKey, SeatView] = {}
    alone_homes: dict[EntityKey, MemoryHome] = {}
    alone_views: dict[EntityKey, SeatView] = {}
    rendered: dict[EntityKey, tuple[str, str]] = {}
    frames = 0
    with (
        passes(lambda view: any(view is each for each in shared_views.values())) as seen,
        MemoryHome.open(":memory:", INSTANCE, verify="every-commit") as home,
    ):
        for seat in sorted(streams):
            alone_homes[seat] = MemoryHome.open(":memory:", INSTANCE)
            alone_views[seat] = alone_homes[seat].seat(CAST3.identity(seat))
        try:
            for seat, batch in interleave(streams, FRAME, INTERLEAVING):
                view = shared_views.get(seat)
                if view is None:
                    view = shared_views[seat] = home.seat(CAST3.identity(seat))
                run(absorb(view, batch, batch[-1].id, kept[seat]), timeout_s=600)
                alone = alone_views[seat]
                run(absorb(alone, batch, batch[-1].id, kept[seat]), timeout_s=600)
                frames += 1
                dump = view.dump()
                assert len(dump) <= BOUND, (seat, frames, len(dump))
                assert dump == alone.dump(), (seat, frames)  # AS28-5: no home-mate moves a byte
                separate = 0
                for other, one in alone_homes.items():
                    size = len(one.dump())
                    held = len(alone_views[other].row_bytes("l0"))
                    view_size = len(alone_views[other].dump())
                    assert size <= view_size + K * held, (other, frames)  # I-a-6 (b)
                    separate += size
                assert len(home.dump()) <= separate, frames  # I-a-6 (a)
            # AS28-7: every stored fact is held by some seat; nothing a seat holds is missing.
            assert _keys(home, "fact") == {
                int(str(row[0]["event_key"]))
                for v in shared_views.values()
                for row in v.row_bytes("l0")
            }
            for seat in sorted(streams):
                now = streams[seat][-1].at
                index = CAST3.index(seat)
                rendered[seat] = (
                    sections(shared_views[seat], index, 0, now, CAST3.people),
                    sections(alone_views[seat], index, 0, now, CAST3.people),
                )
        finally:
            for one in alone_homes.values():
                one.close()
    print(
        f"\n[AC-b-7 {policies}] {frames} frames; passes (seat, shared, cause, before, after, forced): "
        f"{[(p.seat, p.shared, p.cause, p.before, p.after, p.forced) for p in seen]}"
    )
    for each in seen:
        assert each.after <= TARGET, each
        assert each.before <= BOUND, each
    return seen, rendered


@pytest.mark.slow
def test_keep_all_everywhere_holds_the_bound_and_i_a_6_at_the_bound() -> None:
    # AC-b-7 (AS28-9): every seat at its bound, the backstop forcing drops of records its mates hold too.
    seen, _ = lockstep(
        {
            EntityKey("alice"): "keep-all",
            EntityKey("bob"): "keep-all",
            EntityKey("carol"): "keep-all",
        }
    )
    for seat in ("alice", "bob", "carol"):
        mine = [p for p in seen if p.shared and p.seat == seat]
        assert len([p for p in mine if p.cause == "size"]) >= 2, seat
        assert sum(p.forced for p in mine) > 0, seat


@pytest.mark.slow
def test_a_consolidating_seat_is_untouched_by_mates_forced_at_the_bound() -> None:
    # AC-b-8 (AS28-5, AS28-7): Alice consolidates; Bob and Carol keep everything and are forced. Her view and
    # her 200 sections are hers alone at every frame and at the end; the last holder collects.
    seen, rendered = lockstep(
        {
            EntityKey("alice"): "consolidate",
            EntityKey("bob"): "keep-all",
            EntityKey("carol"): "keep-all",
        }
    )
    assert sum(p.forced for p in seen if p.shared and p.seat != "alice") > 0
    for seat, (shared, alone) in rendered.items():
        assert shared == alone, seat


# AC-b-10 — the margin in positional bytes (F-S28a-8) ------------------------------------------------------


def _growth(participants: int) -> tuple[int, int]:
    """The content growth of one fact naming `participants` new 20-digit people, on a seat view that already
    holds a named seat and one fact; and the view's `meta` bytes after it."""
    me = CAST[EntityKey("alice")]
    people: list[EntityId] = [EntityId(str(10**19 + 73 * 10**8 + n)) for n in range(participants)]
    with seat_view(CAST3.identity(EntityKey("alice"))) as view:
        first = [
            named(1001, me, "Alice Moreau"),
            fact(
                1002,
                3600,
                "crowd-gathered",
                {"size": 1},
                subjects=[CROWD[0]],
                participants=[me, CROWD[0]],
                place=PLACES[0],
            ),
        ]
        ingest(view, first, first[-1].id)
        before = view.content_size()
        crossing = fact(
            99_999_999_999_999_999,
            3700,
            "crowd-gathered",
            {
                "crowd": True
            },  # the same bytes for every count, so the growth is the participants' alone
            subjects=[people[0]],
            participants=[me, *people],
            place=PLACES[0],
        )
        ingest(view, [crossing], crossing.id)
        dump = view.dump()
        meta = sum(
            len(line.encode("utf-8")) + 1
            for line in dump.decode("utf-8").splitlines()
            if json.loads(line)[0] == "meta"
        )
        return view.content_size() - before, meta


def test_the_trigger_margin_holds_a_three_thousand_participant_fact_in_positional_bytes() -> None:
    # AC-b-10: FG-b2's argument (pr-fg-b2 §4.1) re-made on the seat view: the growth is linear in the
    # participants up to the digits of each counterpart's ordinal (D-b-7), and 3 000 of them plus `meta` fit
    # `bound - trigger`.
    grown = {p: _growth(p) for p in (1, 60, 600, 3000)}
    slope = -(
        -(grown[3000][0] - grown[600][0]) // 2400
    )  # the steepest per-participant cost, rounded up
    for p, (growth, _) in grown.items():
        assert grown[1][0] <= growth <= grown[1][0] + slope * (p - 1), (p, growth, slope)
    growth, meta = grown[3000]
    holds = (MARGIN - meta - grown[1][0]) // slope
    print(
        f"\n[AC-b-10] {slope} B per participant; a 3 000-participant fact grows {growth} B, meta {meta} B; "
        f"the margin {MARGIN} B holds {holds} participants"
    )
    assert growth + meta <= MARGIN
