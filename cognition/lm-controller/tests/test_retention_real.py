"""Gate 2 of pr-fg-b (§7, §15.5): retention over the real binary's 100-day social-cafe history.

Alice's real export (`mineworld perceived`, shared with `test_ac10.py` through `conftest.social_cafe_100`)
goes through `absorb` under `consolidate` and `keep-all`: AFGb-3 (coverage, against the export), AFGb-4
(a) to (c) (the bound after every pass; before every pass under `consolidate`; the host's own removals under
`keep-all`), AFGb-5 (a) real half (day by day = frames of 256), AFGb-11 (c) (the file on disk). The save is
read only through the CLI. Bounds are §5.7's literals.
"""

from __future__ import annotations

import time
from pathlib import Path

import pytest
from ac10_harness import History
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import DAY, days, feed, frames, retention
from retention_oracle import check_coverage
from support import run

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.store import MemoryStore

pytestmark = pytest.mark.real_binary

TARGET = 786_432
STORE_BOUND = 1_048_576
ON_DISK = 4 * 1_048_576
CHECKPOINTS = (30, 100)


def _identity(history: History) -> StoreIdentity:
    return StoreIdentity(
        world_instance=history.instance, seat=EntityKey("alice"), self_entity=history.me
    )


def _day_by_day(
    store: MemoryStore, history: History, kept: Retention
) -> list[tuple[int, int, int, int]]:
    """Each day's frame ingested up to its first fact by plain `ingest`, the dump measured (just before
    the pass), then `absorb` runs the due pass, the dump measured again (just after it), and coverage is
    checked against the export. Returns (day, before, after, forced) per pass."""
    perceived = history.ids()
    measured: list[tuple[int, int, int, int]] = []
    for frame in days(history.alice):
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
        assert before <= STORE_BOUND, (day, before)  # AFGb-4 (b)
    assert on_disk <= ON_DISK  # AFGb-11 (c)
    # AFGb-5 (a), real half: frames of 256 through absorb's main path give the same store.
    with MemoryStore.open(":memory:", _identity(history)) as store:
        feed(store, frames(history.alice, 256), retention("consolidate"))
        assert store.dump() == day_by_day


def test_keep_all_is_held_by_the_host_on_the_real_history(social_cafe_100: History) -> None:
    history = social_cafe_100
    started = time.monotonic()
    with MemoryStore.open(":memory:", _identity(history)) as store:
        measured = _day_by_day(store, history, retention("keep-all"))
    print(
        f"\n[retention-real keep-all] {len(measured)} passes, {time.monotonic() - started:.1f} s; "
        f"(day, before, after, forced) at 30/100: {[_at(measured, d) for d in CHECKPOINTS]}"
    )
    for day, _, after, _ in measured:
        assert after <= TARGET, (day, after)  # AFGb-4 (a)
    assert any(forced > 0 for day, _, _, forced in measured if day <= 30)  # AFGb-4 (c)
