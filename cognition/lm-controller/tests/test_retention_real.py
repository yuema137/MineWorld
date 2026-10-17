"""Gate 2 of pr-fg-b (§7, §15.5): retention over the real binary's 100-day social-cafe history.

Alice's real export (`mineworld perceived`, shared with `test_ac10.py` through `conftest.social_cafe_100`)
goes through `absorb` under `consolidate` and `keep-all`: AFGb-3 (coverage, against the export), AFGb-4
(a) to (c) (the bound after every pass; before every pass under `consolidate`; the host's own removals under
`keep-all`), AFGb-5 (a) real half (day by day = frames of 256), AFGb-11 (c) (the file on disk). The save is
read only through the CLI. Bounds are the 20 MiB default and its 15 MiB target (pr-fg-b2 §4.1, §7).
"""

from __future__ import annotations

import time
from collections.abc import Sequence
from pathlib import Path

import pytest
from ac10_harness import History
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey, EventId
from retention_driver import DAY, days, feed, frames, retention
from retention_oracle import check_coverage
from support import run

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.store import MemoryStore

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
    store: MemoryStore,
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
        if (each := _pass(store, frame, kept, perceived)) is not None:
            measured.append(each)
    return measured


def _pass(
    store: MemoryStore, frame: list[PerceivedEvent], kept: Retention, perceived: Sequence[EventId]
) -> tuple[int, int, int, int] | None:
    """One day: its frame ingested by plain `ingest`, the dump measured just before the pass, `absorb`
    runs the due pass, and coverage is checked against the export. Returns (day, before, after, forced),
    or `None` when no pass was due."""
    ingest(store, frame, frame[-1].id)
    before = len(store.dump())
    report = run(absorb(store, [], frame[-1].id, kept), timeout_s=120)
    if not report.passes:
        return None
    dump = store.dump()
    check_coverage(dump, perceived)
    return (frame[-1].at // DAY, before, len(dump), report.passes[-1].forced)


def _keep_all_window(
    store: MemoryStore, history: History
) -> tuple[int, list[tuple[int, int, int, int]]]:
    """AFGb-4 (c)'s window, chosen by the store's measured size rather than by fixed days (operator,
    2026-10-15). Whole days are ingested without retention while the store's content stays under
    `KEEP_ALL_LEAD`, since a keep-all pass removes nothing there. From then on every day runs its pass,
    until `KEEP_ALL_AFTER` days past the first forced pass, or until the history ends, at most
    `KEEP_ALL_CAP_DAYS` days. Returns the day the passes started and (day, before, after, forced) for
    each pass."""
    perceived = history.ids()
    kept = retention("keep-all")
    measured: list[tuple[int, int, int, int]] = []
    leading: list[PerceivedEvent] = []
    passing = False
    started = KEEP_ALL_CAP_DAYS
    for frame in days(f for f in history.alice if f.at // DAY < KEEP_ALL_CAP_DAYS):
        day = frame[-1].at // DAY
        if not passing:
            ingest(store, frame, frame[-1].id)
            leading.extend(frame)
            if store.content_size() < KEEP_ALL_LEAD:
                continue
            # The store is near the target: absorb first sees it here, and every later day passes.
            run(absorb(store, [], leading[-1].id, kept), timeout_s=120)
            passing, started = True, day + 1
            continue
        if (each := _pass(store, frame, kept, perceived)) is not None:
            measured.append(each)
        first = next((d for d, _, _, forced in measured if forced > 0), None)
        if first is not None and day >= first + KEEP_ALL_AFTER:
            break
    return started, measured


def _at(measured: list[tuple[int, int, int, int]], day: int) -> tuple[int, int, int, int]:
    """The first pass at or after `day`; for day 100 of a 100-day history (days 0 to 99), the last."""
    return next((each for each in measured if each[0] >= day), measured[-1])


def test_consolidate_holds_the_bound_and_coverage_on_the_real_history(
    social_cafe_100: History, tmp_path: Path
) -> None:
    history = social_cafe_100
    path = tmp_path / "alice.sqlite"
    started = time.monotonic()
    with MemoryStore.open(path, _identity(history)) as store:
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
    with MemoryStore.open(":memory:", _identity(history)) as store:
        feed(store, frames(history.alice, 256), retention("consolidate"))
        assert store.dump() == day_by_day


KEEP_ALL_LEAD = TARGET // 2
"""7.5 MiB: below it a keep-all pass removes nothing (the store is under the target), so those days are
ingested at once, without passes. Half the target leaves more than one day's growth before the crossing
on any history measured so far (≈ 433 KB a day on main 6306977, ≈ 760 KB a day on 12d's town)."""
KEEP_ALL_AFTER = 3
"""Days measured after the first forced pass: the store is held at the target, not only cut once."""
KEEP_ALL_CAP_DAYS = 100
"""The fixture's whole history (days 0 … 99). If the store never reaches the target by then, the test
fails by name instead of running on."""


def test_keep_all_is_held_by_the_host_on_the_real_history(social_cafe_100: History) -> None:
    """AFGb-4 (c), amended 2026-10-15 (operator; pr-fg-b2 §7 note): the window is chosen by the store's
    measured size, not by fixed days, so that a change to the town's history (12d: walls and bodies)
    moves the window instead of breaking the claim. The claim is unchanged: under keep-all the host's
    own removals start once the store reaches the target, and every pass leaves it at or under the
    target."""
    history = social_cafe_100
    started = time.monotonic()
    with MemoryStore.open(":memory:", _identity(history)) as store:
        window, measured = _keep_all_window(store, history)
        largest = len(store.dump())
    first = next((day for day, _, _, forced in measured if forced > 0), None)
    print(
        f"\n[retention-real keep-all] passes from day {window}: {len(measured)}, "
        f"{time.monotonic() - started:.1f} s; first and last (day, before, after, forced): "
        f"{measured[:1]} {measured[-1:]}; first forced pass: day {first}"
    )
    assert window < KEEP_ALL_CAP_DAYS, (
        f"keep-all: the store never reached the lead-in size {KEEP_ALL_LEAD} B in "
        f"{KEEP_ALL_CAP_DAYS} days ({largest} B at the end)"
    )
    for day, before, after, _ in measured:
        assert after <= TARGET, (day, after)  # AFGb-4 (a)
        assert before <= STORE_BOUND, (day, before)
    assert first is not None, (
        f"keep-all: no forced pass in {KEEP_ALL_CAP_DAYS} days; the store never reached the "
        f"{TARGET} B target ({largest} B at the end)"
    )  # AFGb-4 (c)
    # The window opens under the target: the first pass removes nothing, so removals begin at the crossing.
    assert measured[0][3] == 0, measured[0]
