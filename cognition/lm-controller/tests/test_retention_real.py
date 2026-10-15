"""Gate 2 of pr-fg-b (§7, §15.5): retention over the real binary's 100-day social-cafe history.

Alice's real export (`mineworld perceived`, shared with `test_ac10.py` through `conftest.social_cafe_100`)
goes through `absorb` under `consolidate` and `keep-all`: AFGb-3 (coverage, against the export), AFGb-4
(a) to (c) (the bound after every pass; before every pass under `consolidate`; the host's own removals under
`keep-all`), AFGb-5 (a) real half (day by day = frames of 256), AFGb-11 (c) (the file on disk). The save is
read only through the CLI. Bounds are the 20 MiB default and its 15 MiB target (pr-fg-b2 §4.1, §7).
"""

from __future__ import annotations

import time
from pathlib import Path

import pytest
from ac10_harness import History
from memory_fixtures import seat_view
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import DAY, days, feed, frames, retention
from retention_oracle import check_coverage
from support import run

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.selection import Retention

pytestmark = pytest.mark.real_binary

MIB = 1_048_576
STORE_BOUND = 20 * MIB  # the default per-person bound (pr-fg-b2 §4.1; operator, 2026-10-14)
TARGET = STORE_BOUND * 3 // 4  # 15 MiB: the dump after every pass
ON_DISK = 4 * MIB  # the consolidate store's file: far under the bound, which it never nears
CHECKPOINTS = (30, 100)


def _identity(history: History) -> StoreIdentity:
    return StoreIdentity(
        world_instance=history.instance, seat=EntityKey("alice"), self_entity=history.me
    )


def _day_by_day(
    store: SeatView,
    history: History,
    kept: Retention,
    *,
    from_day: int = 0,
    through_day: int | None = None,
) -> list[tuple[int, int, int, int]]:
    """Each day's frame ingested up to its first fact by plain `ingest`, the dump measured (just before
    the pass), then `absorb` runs the due pass, the dump measured again (just after it), and coverage is
    checked against the export. Returns (day, before, after, forced) per pass, for the whole history or
    for its days up to `through_day`. Days before `from_day` are ingested at once without retention, and
    `absorb` first sees the store at their end (no pass for them)."""
    perceived = history.ids()
    measured: list[tuple[int, int, int, int]] = []
    head = [f for f in history.alice if f.at // DAY < from_day]
    if head:
        ingest(store, head, head[-1].id)
        run(absorb(store, [], head[-1].id, kept), timeout_s=120)
    facts = [
        f for f in history.alice[len(head) :] if through_day is None or f.at // DAY <= through_day
    ]
    for frame in days(facts):
        ingest(store, frame, frame[-1].id)
        before = len(store.dump())
        report = run(absorb(store, [], frame[-1].id, kept), timeout_s=120)
        if not report.passes:
            continue
        dump = store.dump()
        check_coverage(dump, perceived)
        measured.append((frame[-1].at // DAY, before, len(dump), report.passes[-1].forced))
    return measured


def _at(measured: list[tuple[int, int, int, int]], day: int) -> tuple[int, int, int, int]:
    """The first pass at or after `day`; for day 100 of a 100-day history (days 0 to 99), the last."""
    return next((each for each in measured if each[0] >= day), measured[-1])


def test_consolidate_holds_the_bound_and_coverage_on_the_real_history(
    social_cafe_100: History, tmp_path: Path
) -> None:
    history = social_cafe_100
    path = tmp_path / "alice.sqlite"
    started = time.monotonic()
    with seat_view(_identity(history), path) as store:
        measured = _day_by_day(store, history, retention("consolidate"))
        day_by_day = store.dump()
    wall = time.monotonic() - started
    on_disk = path.stat().st_size
    print(
        f"\n[retention-real consolidate] {len(measured)} passes, {wall:.1f} s; "
        f"(day, before, after, forced) at 30/100: {[_at(measured, d) for d in CHECKPOINTS]}; "
        f"largest before {max(m[1] for m in measured)}, after {max(m[2] for m in measured)}; "
        f"file {on_disk} B"
    )
    assert len(measured) >= 95
    for day, before, after, _ in measured:
        assert after <= TARGET, (day, after)  # AFGb-4 (a)
        assert before <= STORE_BOUND, (day, before)  # AFGb-4 (b), now held by the size trigger
    assert on_disk <= ON_DISK  # AFGb-11 (c)
    # AFGb-5 (a), real half: frames of 256 through absorb's main path give the same store.
    with seat_view(_identity(history)) as store:
        feed(store, frames(history.alice, 256), retention("consolidate"))
        assert store.dump() == day_by_day


KEEP_ALL_FROM = 75
KEEP_ALL_DAYS = 85
"""AFGb-4 (c) re-derived for the 20 MiB bound (pr-fg-b2 §7), then for schema 4's positional seat view
(pr-s28a C5 step 2): at ≈ 195 KB a day (measured: 19 530 040 B of content after 100 days of plain ingest,
was ≈ 433 KB a day in schema 3's objects) keep-all reaches the 15 MiB target after ≈ 81 days, so the
host's own removals begin between days 75 and 85. Before day 75 a keep-all pass removes nothing (the store
is under the target), so those days are ingested at once, without passes; after day 85 every pass works on
a store of ≈ 15 MiB. Both cuts keep the test within FG-b's 180 s per CI leg."""


def test_keep_all_is_held_by_the_host_on_the_real_history(social_cafe_100: History) -> None:
    history = social_cafe_100
    started = time.monotonic()
    with seat_view(_identity(history)) as store:
        measured = _day_by_day(
            store, history, retention("keep-all"), from_day=KEEP_ALL_FROM, through_day=KEEP_ALL_DAYS
        )
    first = next((day for day, _, _, forced in measured if forced > 0), None)
    print(
        f"\n[retention-real keep-all] {len(measured)} passes, {time.monotonic() - started:.1f} s; "
        f"(day, before, after, forced) at {KEEP_ALL_FROM}/{KEEP_ALL_DAYS}: "
        f"{[_at(measured, d) for d in (KEEP_ALL_FROM, KEEP_ALL_DAYS)]}; "
        f"first forced pass: day {first}"
    )
    for day, before, after, _ in measured:
        assert after <= TARGET, (day, after)  # AFGb-4 (a)
        assert before <= STORE_BOUND, (day, before)
    assert first is not None and KEEP_ALL_FROM < first <= KEEP_ALL_DAYS, first  # AFGb-4 (c)
