"""`absorb`: ingestion with retention, the only caller of a pass (pr-fg-b §5.6).

A pass is due when the newest ingested fact falls on a later simulated day than the last pass. `absorb`
splits a frame at every first fact of a new day: it ingests the facts up to and including that fact (the
day change closes the previous day's last episode, D-P4-7), runs the pass, and goes on. The split points
are facts, never frame boundaries, so frames of 1, of 256 or the whole export give one sequence of passes
and one store (I-5 with retention). A crash between an ingest and its pass leaves the store exactly in the
post-trigger state; the next `absorb` sees the pass is due and runs it before anything else.

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
    day = _retention_day(store, facts)
    newest = _newest_day(store)
    if day is not None and newest is not None and newest > day:
        passes.append(run_pass(store, retention, newest * DAY_S))
        day = newest
    cursor = store.cursor()
    bound = max(store.newest() or "", "" if cursor is None else event_key(cursor))
    start = 0
    for index, fact in enumerate(facts):
        key = event_key(fact.id)
        if key <= bound:
            continue  # held, merged or forgotten: ingest decides; it opens no day
        bound = key
        if day is not None and fact.at // DAY_S > day:
            ingested.append(ingest(store, facts[start : index + 1], fact.id))
            start = index + 1
            day = fact.at // DAY_S
            passes.append(run_pass(store, retention, day * DAY_S))
    cursor = store.cursor()
    if start < len(facts) or cursor is None or event_key(through) > event_key(cursor):
        ingested.append(ingest(store, facts[start:], through))
    return AbsorbReport(tuple(ingested), tuple(passes))


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
    newest = store.newest()
    if newest is None:
        return None
    (at,) = store.connection.execute("SELECT at FROM l0 WHERE event_key = ?", (newest,)).fetchone()
    return at // DAY_S
