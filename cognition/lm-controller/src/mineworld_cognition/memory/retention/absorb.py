"""`absorb`: ingestion with retention, the only caller of a pass (pr-fg-b §5.6).

A pass is due when the newest ingested fact falls on a later simulated day than the last pass. `absorb`
splits a frame at every first fact of a new day: it ingests the facts up to and including that fact (the
day change closes the previous day's last episode, D-P4-7), runs the pass, and goes on. The split points
are facts, never frame boundaries, so frames of 1, of 256 or the whole export give one sequence of passes
and one store (I-5 with retention). A crash between an ingest and its pass leaves the store exactly in the
post-trigger state; the next `absorb` sees the pass is due and runs it before anything else.

A pass also runs early when the store nears its bound (pr-fg-b2 §4.3; operator, 2026-10-14: "快满了就提前
整理"): every ingest call stops after the fact that takes the store's content over `bounds.trigger`, and a
size pass runs there, then ingestion goes on. The measure after a fact is a function of the facts up to it
(`MemoryStore.content_size`), so size passes, too, fall on the same facts however the frame was cut. At a
day's first fact only the daily pass runs. A crash between a stop and its pass resumes with the pass.

`ingest` alone never forgets (FGb-7). `absorb` is async because an assisted policy (FG-c) awaits a
gateway; FG-b's policies are called directly.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.ingest import IngestReport, ingest
from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.retention.host import PassReport, run_pass
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.store import MemoryStore


@dataclass(frozen=True)
class AbsorbReport:
    ingest: tuple[IngestReport, ...]
    """One per ingest call made."""
    passes: tuple[PassReport, ...]
    """One per pass run."""


async def absorb(
    store: MemoryStore, facts: Sequence[PerceivedEvent], through: EventId, retention: Retention
) -> AbsorbReport:
    ingested: list[IngestReport] = []
    passes: list[PassReport] = []
    trigger = retention.bounds.trigger

    def full() -> bool:
        return store.content_size() > trigger

    day = _retention_day(store, facts)
    newest = _newest_day(store)
    if day is not None and newest is not None and newest > day:
        passes.append(run_pass(store, retention, newest * DAY_S))
        day = newest
    elif full() and (at := _newest_at(store)) is not None:
        # A stop committed and its size pass did not (pr-fg-b2 §4.3, resume): run it first.
        passes.append(run_pass(store, retention, at, "size"))
    start = 0
    for end, day_start in _segments(store, facts, day):
        last = facts[end - 1].id if day_start else through
        while start < end:
            report = ingest(store, facts[start:end], last, stop_when=full)
            ingested.append(report)
            if report.stopped is None:
                start = end
                break
            stop = next(i for i in range(start, end) if facts[i].id == report.stopped)
            start = stop + 1
            if day_start and start == end:
                break  # the day's first fact: its daily pass follows, and no size pass (FGb2-3)
            passes.append(run_pass(store, retention, facts[stop].at, "size"))
        if day_start:
            passes.append(run_pass(store, retention, facts[end - 1].at // DAY_S * DAY_S))
    cursor = store.cursor()
    if cursor is None or event_key(through) > event_key(cursor):
        ingested.append(ingest(store, [], through))
    return AbsorbReport(tuple(ingested), tuple(passes))


def _segments(
    store: MemoryStore, facts: Sequence[PerceivedEvent], day: int | None
) -> list[tuple[int, bool]]:
    """Where the frame splits for daily passes: `(end, True)` after every first fact of a new day, then
    `(len(facts), False)`. Facts held, merged or forgotten open no day; `ingest` decides on them."""
    cursor = store.cursor()
    bound = max(store.newest() or "", "" if cursor is None else event_key(cursor))
    segments: list[tuple[int, bool]] = []
    for index, fact in enumerate(facts):
        key = event_key(fact.id)
        if key <= bound:
            continue
        bound = key
        if day is not None and fact.at // DAY_S > day:
            segments.append((index + 1, True))
            day = fact.at // DAY_S
    segments.append((len(facts), False))
    return segments


def _retention_day(store: MemoryStore, facts: Sequence[PerceivedEvent]) -> int | None:
    """The day of the last pass. A store's first fact sets it to that fact's day, in its own small
    transaction, so no pass runs for a day with no earlier day; a store that plain `ingest` filled sets it
    to its newest fact's day."""
    found: tuple[int | None] = store.connection.execute(
        "SELECT CASE WHEN value = '' THEN NULL ELSE CAST(value AS INTEGER) END FROM meta "
        "WHERE key = 'retention_day'"
    ).fetchone()
    if found[0] is not None:
        return found[0]
    first = _newest_day(store)
    if first is None:
        if not facts:
            return None
        first = facts[0].at // DAY_S
    with store.transaction():
        store.set_meta("retention_day", str(first))
    return first


def _newest_day(store: MemoryStore) -> int | None:
    at = _newest_at(store)
    return None if at is None else at // DAY_S


def _newest_at(store: MemoryStore) -> int | None:
    """The world time of the newest ingested fact: the open episode's last record, which no pass
    removes."""
    newest = store.newest()
    if newest is None:
        return None
    (at,) = store.connection.execute("SELECT at FROM l0 WHERE event_key = ?", (newest,)).fetchone()
    return at
