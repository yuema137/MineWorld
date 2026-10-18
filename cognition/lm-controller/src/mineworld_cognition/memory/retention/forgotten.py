"""The forgotten set (pr-fg-b §5.8; `CORE_CONCEPTS.md` §5.3): what a seat no longer holds, as sorted,
disjoint ranges over its perceived sequence, plus a watermark.

Two ranges with no surviving unit between them merge: by coverage (FGb-1), every perceived id between them
is forgotten already. Beyond `MAX_RANGES` ranges the oldest are removed and the watermark rises to their
end: "every perceived id at or below the watermark that no surviving unit represents is forgotten". The
set is therefore bounded by construction. The facts themselves stay in the save and resolve through
`mineworld perceived` (FGb-6); this is how a tool says "Alice has forgotten 12 Oct 08:00 to 14:00".
"""

from __future__ import annotations

from dataclasses import dataclass

from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import EventRange, event_key, event_of

MAX_RANGES = 256


@dataclass(frozen=True)
class ForgottenSet:
    _ranges: tuple[EventRange, ...]
    _watermark: EventId | None

    @staticmethod
    def load(store: SeatView) -> ForgottenSet:
        rows: list[tuple[str, str]] = store.connection.execute(
            "SELECT first_key, last_key FROM forgotten WHERE seat = ? ORDER BY first_key",
            (store.seat_id,),
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


def normalize(store: SeatView) -> None:
    """Merges ranges no surviving unit separates, then caps the set at `MAX_RANGES` with the watermark."""
    db = store.connection
    seat = store.seat_id
    rows: list[tuple[str, str]] = db.execute(
        "SELECT first_key, last_key FROM forgotten WHERE seat = ? ORDER BY first_key", (seat,)
    ).fetchall()
    merged: list[list[str]] = []
    for first, last in rows:
        if merged and not _unit_between(store, merged[-1][1], first):
            merged[-1][1] = last
        else:
            merged.append([first, last])
    watermark = store.meta("forgotten_below")
    while len(merged) > MAX_RANGES:
        oldest = merged.pop(0)
        watermark = max(watermark, oldest[1])
    if [tuple(pair) for pair in merged] != rows:
        db.execute("DELETE FROM forgotten WHERE seat = ?", (seat,))
        db.executemany(
            "INSERT INTO forgotten (seat, first_key, last_key) VALUES (?, ?, ?)",
            [(seat, first, last) for first, last in merged],
        )
    store.set_meta("forgotten_below", watermark)


def _unit_between(store: SeatView, after: str, before: str) -> bool:
    """Whether a surviving unit of the seat (an episode, open or closed, or a chapter range) starts strictly
    between two forgotten ranges. Units never overlap the forgotten set, so one between them starts there."""
    seat = store.seat_id
    found = store.connection.execute(
        "SELECT 1 FROM episodes WHERE seat = ? AND first_key > ? AND first_key < ? "
        "UNION ALL SELECT 1 FROM chapter_ranges WHERE seat = ? AND first_key > ? AND first_key < ? "
        "LIMIT 1",
        (seat, after, before, seat, after, before),
    ).fetchone()
    return found is not None
