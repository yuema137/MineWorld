"""The per-person bound and the size-triggered pass (pr-fg-b2): AFGb2-1 (the bound, at every pass and after
every `absorb` call), AFGb2-2 (coverage), AFGb2-3 (frames), AFGb2-4 (resume), AFGb2-5 (one pass per fact),
AFGb2-6 (the incremental measure), AFGb2-7 (configuration, Python half).

A busy world is the heavy synthetic stream (`retention_stream.heavy`): more than the bound in one simulated
day. Every oracle reads `dump()` or the generator's own ids; the figures are written here from the
operator's 20 MiB, never imported from the code under test.
"""

from __future__ import annotations

import json
from collections.abc import Generator, Sequence
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path

import pytest
from memory_fixtures import alice_identity
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EventId
from retention_driver import DAY, built, by_day, feed, frames, retention
from retention_fixtures import DropEverything
from retention_oracle import Dump, check_coverage
from retention_stream import heavy, stream
from support import run

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.retention import absorb as absorb_module
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.bound import StoreBoundOutOfRange, StoreBounds
from mineworld_cognition.memory.retention.host import PassCause, PassReport, run_pass
from mineworld_cognition.memory.retention.registry import PolicyRegistry, SourceSpan
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve
from mineworld_cognition.memory.store import MemoryStore

MIB = 1_048_576
BOUND = 20 * MIB  # 20 971 520: the operator's default (2026-10-14)
TARGET = BOUND * 3 // 4  # 15 728 640
TRIGGER = BOUND - BOUND // 16  # 19 660 800

BUSY = (7, 2, 2800, 100)
"""Two days of 2 800 facts, each with Alice among 100 of the crowd: ≈ 8.5 KB of dump per fact, ≈ 24 MB a
day, so each day crosses the trigger at least once and day 1 opens with a daily pass."""


def busy() -> list[PerceivedEvent]:
    seed, days, per_day, gathered = BUSY
    return list(heavy(seed, days, per_day, gathered))


@dataclass(frozen=True)
class Seen:
    cause: PassCause
    before: int
    after: int
    report: PassReport


@contextmanager
def watched(
    monkeypatch: pytest.MonkeyPatch, perceived: Sequence[EventId] | None = None
) -> Generator[list[Seen]]:
    """Every pass `absorb` runs, with `len(dump())` just before and just after it, and AFGb-3's
    coverage checked after it when `perceived` is given."""
    seen: list[Seen] = []
    real = run_pass

    def measured(
        store: MemoryStore, kept: Retention, now: int, cause: PassCause = "day"
    ) -> PassReport:
        before = len(store.dump())
        report = real(store, kept, now, cause)
        dump = store.dump()
        if perceived is not None:
            check_coverage(dump, perceived)
        seen.append(Seen(cause, before, len(dump), report))
        return report

    monkeypatch.setattr(absorb_module, "run_pass", measured)
    yield seen


# AFGb2-7 — configuration ------------------------------------------------------------------------------


def test_the_bound_is_twenty_mib_by_default_and_derives_its_figures() -> None:
    # The figures are the design's literals (§4.1). Its mutation removes the floor check: 19 is accepted.
    default = resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed()).bounds
    assert (default.bound, default.target, default.trigger) == (20_971_520, 15_728_640, 19_660_800)
    raised = resolve(
        MemoryPolicyChoice(policy="consolidate", store_bound_mib=40), PolicyRegistry.installed()
    ).bounds
    assert (raised.bound, raised.target, raised.trigger) == (41_943_040, 31_457_280, 39_321_600)
    for wrong in (19, 0, 1_025, True):
        with pytest.raises(StoreBoundOutOfRange):
            StoreBounds(wrong)


@pytest.mark.parametrize("given", [19, 1_025])
def test_a_bound_below_the_floor_or_above_the_ceiling_is_refused_by_name(given: int) -> None:
    source = SourceSpan(file="decisions.yaml", line=4, column=7)
    choice = MemoryPolicyChoice(policy="consolidate", store_bound_mib=given, source=source)
    with pytest.raises(StoreBoundOutOfRange) as refused:
        resolve(choice, PolicyRegistry.installed())
    assert (refused.value.given, refused.value.floor, refused.value.ceiling) == (given, 20, 1_024)
    assert refused.value.source == source
    assert str(given) in str(refused.value) and "decisions.yaml:4:7" in str(refused.value)


def test_the_bound_is_a_strict_integer_in_the_choice() -> None:
    with pytest.raises(ValueError):
        MemoryPolicyChoice.model_validate({"policy": "consolidate", "store_bound_mib": "40"})


# AFGb2-6 — the measure --------------------------------------------------------------------------------


def _content(dump: bytes) -> int:
    """`content_size()`'s oracle: the dump's bytes less `meta`'s lines, from the dump itself."""
    lines = dump.decode("utf-8").splitlines()
    return sum(
        len(line.encode("utf-8")) + 1 for line in lines if json.loads(line)["table"] != "meta"
    )


@pytest.mark.parametrize("policy", ["consolidate", "drop-everything"])
def test_the_incremental_measure_is_the_dump_without_meta(policy: str) -> None:
    # After every absorb call (merges under consolidate, drops and the forgotten set under
    # drop-everything) and after a rolled-back transaction. Its mutations omit the UPDATE trigger, or the
    # invalidation on rollback: the measure drifts from the dump.
    facts = list(stream(11, 30, per_day=120))
    kept = (
        retention(policy, PolicyRegistry({"drop-everything": DropEverything}))
        if policy == "drop-everything"
        else retention(policy)
    )
    checked = 0
    with MemoryStore.open(":memory:", alice_identity()) as store:
        for each in by_day(store, facts, kept):
            assert store.content_size() == _content(each.dump), each.day
            checked += 1
        before = store.content_size()
        with pytest.raises(RuntimeError), store.transaction() as db:
            db.execute("UPDATE names SET name = 'Somebody Else' WHERE rowid = 1")
            db.execute("DELETE FROM episodes WHERE rowid = (SELECT max(rowid) FROM episodes)")
            assert store.content_size() != before
            raise RuntimeError("rolled back")
        assert store.content_size() == before == _content(store.dump())
    assert checked >= 25


# AFGb2-1, -2, -5 — a busy world -----------------------------------------------------------------------


def test_a_busy_day_overflows_without_a_size_trigger() -> None:
    # The precondition AFGb2-1 rests on: one simulated day of the heavy stream, ingested with only FG-b's
    # daily pass in mind, is over the bound before its pass, as PR 12d's day 1 was over 1 MiB.
    facts = busy()
    day_zero = [fact for fact in facts if fact.at < DAY]
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, day_zero, day_zero[-1].id)
        assert len(store.dump()) > BOUND


@pytest.mark.parametrize("policy", ["keep-all", "consolidate"])
def test_size_passes_hold_the_bound_at_every_instant(
    policy: str, monkeypatch: pytest.MonkeyPatch
) -> None:
    # AFGb2-1: after every pass ≤ TARGET, before every pass and after every absorb call ≤ BOUND; AFGb2-2
    # after every pass; AFGb2-5 on the pass log. Its mutations: no size trigger (after-call dump over the
    # bound); the backstop off (keep-all's post-pass dump over the target); a size pass at a day's first
    # fact too (two passes for one trigger).
    facts = busy()
    perceived = [fact.id for fact in facts]
    with (
        watched(monkeypatch, perceived) as seen,
        MemoryStore.open(":memory:", alice_identity()) as store,
    ):
        largest = 0
        for batch in frames(facts, 256):
            run(absorb(store, batch, batch[-1].id, retention(policy)), timeout_s=600)
            largest = max(largest, len(store.dump()))
        log = Dump.parse(store.dump()).rows("retention_passes")
    sizes = [each for each in seen if each.cause == "size"]
    days = [each for each in seen if each.cause == "day"]
    print(
        f"\n[{policy}] largest after a call {largest}; passes (cause, before, after, forced): "
        f"{[(each.cause, each.before, each.after, each.report.forced) for each in seen]}"
    )
    assert largest <= BOUND
    assert len(sizes) >= 2 and len(days) == 1
    for each in seen:
        assert each.after <= TARGET, each
        assert each.before <= BOUND, each
    for each in sizes:
        assert each.before > TRIGGER, each  # a size pass only where the store neared its bound
    triggers = [each.report.trigger for each in seen]
    assert len(set(triggers)) == len(triggers)
    assert {str(row["trigger"]) for row in log} == set(triggers)
    assert {str(row["cause"]) for row in log} == {"day", "size"}


def test_a_crossing_at_a_days_first_fact_runs_the_daily_pass_alone(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # FGb2-3: the busy day's crossing fact, and every fact after it, moved to the next day, so the fact
    # that takes the store over its trigger is a day's first. One pass runs there, the daily one. Its
    # mutation lets a size pass run at that fact too: a second pass for one trigger.
    day_zero = [fact for fact in busy() if fact.at < DAY]
    with watched(monkeypatch) as seen, MemoryStore.open(":memory:", alice_identity()) as store:
        feed(store, frames(day_zero, 256), retention("keep-all"))
    crossing = next(each.report.trigger for each in seen if each.cause == "size")
    index = next(i for i, fact in enumerate(day_zero) if event_key(fact.id) == crossing)
    moved = [
        *day_zero[:index],
        *(f.model_copy(update={"at": f.at + DAY}) for f in day_zero[index:]),
    ]
    monkeypatch.undo()
    with watched(monkeypatch) as seen, MemoryStore.open(":memory:", alice_identity()) as store:
        feed(store, frames(moved, 256), retention("keep-all"))
    at_crossing = [each for each in seen if each.report.trigger == crossing]
    assert [each.cause for each in at_crossing] == ["day"], seen
    assert at_crossing[0].before > TRIGGER and at_crossing[0].after <= TARGET


# AFGb2-3 — frames -------------------------------------------------------------------------------------


def test_the_measure_after_each_fact_is_the_same_however_framed() -> None:
    # FGb2-2 at its root: the trigger's measure, read after every fact `ingest` writes, is a function of
    # the facts so far, not of where frames began — over weeks, so chapters are written mid-call. Its
    # mutations: the measure includes `meta` (the cursor is each call's `through`); chapters written
    # only at the end of a call.
    facts = list(stream(13, 23, per_day=40))

    def probed(size: int) -> list[int]:
        seen: list[int] = []
        with MemoryStore.open(":memory:", alice_identity()) as store:

            def probe() -> bool:
                seen.append(store.content_size())
                return False

            for batch in frames(facts, size):
                ingest(store, batch, batch[-1].id, stop_when=probe)
        return seen

    whole = probed(len(facts))
    assert len(whole) == len(facts)
    for size in (1, 7, 256):
        assert probed(size) == whole, size


@pytest.mark.parametrize("policy", ["keep-all", "consolidate"])
def test_frames_never_change_a_busy_store(policy: str) -> None:
    # Size passes fall on the same facts whatever the frames: frames of 1 and of 256 against the whole
    # stream at once.
    facts = busy()
    whole = built(facts, retention(policy), len(facts))
    for size in (1, 256):
        assert built(facts, retention(policy), size) == whole, size


# AFGb2-4 — resume -------------------------------------------------------------------------------------


class Crash(RuntimeError):
    pass


def test_a_crash_between_a_stop_and_its_size_pass_changes_nothing(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # The stop's ingest committed, the process gone before the size pass; reopened, `absorb` runs the
    # pass first. Its mutation drops the entry check: the pass runs a fact later and the dumps differ.
    # Day 0 alone holds the first size pass and facts after it.
    facts = [fact for fact in busy() if fact.at < DAY]
    kept = retention("keep-all")
    uninterrupted = built(facts, kept, 256)
    real = run_pass
    crashed: list[int] = []

    def crash_once(
        store: MemoryStore, kept: Retention, now: int, cause: PassCause = "day"
    ) -> PassReport:
        if cause == "size" and not crashed:
            crashed.append(now)
            raise Crash
        return real(store, kept, now, cause)

    batches = list(frames(facts, 256))
    path = tmp_path / "alice.sqlite"
    with MemoryStore.open(path, alice_identity()) as store:
        monkeypatch.setattr(absorb_module, "run_pass", crash_once)
        resumed_at = _feed_until_crash(store, batches, kept)
    monkeypatch.setattr(absorb_module, "run_pass", real)
    with MemoryStore.open(path, alice_identity()) as store:
        batch = batches[resumed_at]
        first = run(absorb(store, batch, batch[-1].id, kept), timeout_s=600)
        assert first.passes[0].cause == "size"
        feed(store, batches[resumed_at + 1 :], kept)
        assert store.dump() == uninterrupted
    assert crashed


def _feed_until_crash(
    store: MemoryStore, batches: Sequence[Sequence[PerceivedEvent]], kept: Retention
) -> int:
    for index, batch in enumerate(batches):
        try:
            run(absorb(store, batch, batch[-1].id, kept), timeout_s=600)
        except Crash:
            return index
    raise AssertionError("no size pass ran")
