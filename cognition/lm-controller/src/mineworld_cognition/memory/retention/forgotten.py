"""The forgotten set (pr-fg-b §5.8; `CORE_CONCEPTS.md` §5.3): what a seat no longer holds, as sorted,
disjoint ranges over its perceived sequence, plus a watermark.

Two ranges with no surviving unit between them merge: by coverage (FGb-1), every perceived id between them
is forgotten already. Beyond `MAX_RANGES` ranges the oldest are removed and the watermark rises to their
end: "every perceived id at or below the watermark that no surviving unit represents is forgotten". The
set is therefore bounded by construction. The facts themselves stay in the save and resolve through
`mineworld perceived` (FGb-6); this is how a tool says "Alice has forgotten 12 Oct 08:00 to 14:00".
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass

from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory.records import EventRange, event_key, event_of
from mineworld_cognition.memory.store import MemoryStore

MAX_RANGES = 256


@dataclass(frozen=True)
class ForgottenSet:
    _ranges: tuple[EventRange, ...]
    _watermark: EventId | None

    @staticmethod
    def load(store: MemoryStore) -> ForgottenSet:
        rows: list[tuple[str, str]] = store.connection.execute(
            "SELECT first_key, last_key FROM forgotten ORDER BY first_key"
        ).fetchall()
        watermark = store.meta("forgotten_below")
        return ForgottenSet(
            tuple(EventRange(first=event_of(first), last=event_of(last)) for first, last in rows),
            event_of(watermark) if watermark else None,
        )

    def ranges(self) -> tuple[EventRange, ...]:
        return self._ranges

    def watermark(self) -> EventId | None:
        return self._watermark

    def contains(self, event: EventId) -> bool:
        """Whether `event` lies in a forgotten range. At or below the watermark, an id is forgotten
        unless a surviving unit represents it, which only the store can say; `contains` answers for the
        explicit ranges and reports the watermark separately."""
        key = event_key(event)
        return any(event_key(r.first) <= key <= event_key(r.last) for r in self._ranges)


def normalize(db: sqlite3.Connection) -> None:
    """Merges ranges no surviving unit separates, then caps the set at `MAX_RANGES` with the watermark."""
    rows: list[tuple[str, str]] = db.execute(
        "SELECT first_key, last_key FROM forgotten ORDER BY first_key"
    ).fetchall()
    merged: list[list[str]] = []
    for first, last in rows:
        if merged and not _unit_between(db, merged[-1][1], first):
            merged[-1][1] = last
        else:
            merged.append([first, last])
    watermark: str = db.execute("SELECT value FROM meta WHERE key = 'forgotten_below'").fetchone()[
        0
    ]
    while len(merged) > MAX_RANGES:
        oldest = merged.pop(0)
        watermark = max(watermark, oldest[1])
    if [tuple(pair) for pair in merged] != rows:
        db.execute("DELETE FROM forgotten")
        db.executemany("INSERT INTO forgotten (first_key, last_key) VALUES (?, ?)", merged)
    db.execute("UPDATE meta SET value = ? WHERE key = 'forgotten_below'", (watermark,))


def _unit_between(db: sqlite3.Connection, after: str, before: str) -> bool:
    """Whether a surviving unit (an episode, open or closed, or a chapter range) starts strictly between
    two forgotten ranges. Units never overlap the forgotten set, so one between them starts there."""
    found = db.execute(
        "SELECT 1 FROM episodes WHERE first_key > ? AND first_key < ? "
        "UNION ALL SELECT 1 FROM chapter_ranges WHERE first_key > ? AND first_key < ? LIMIT 1",
        (after, before, after, before),
    ).fetchone()
    return found is not None
