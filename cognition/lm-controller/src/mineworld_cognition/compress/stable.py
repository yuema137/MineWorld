"""L3: per counterpart, what I know of them and the facts that establish it (§5.5).

`first_key` is the first perceived fact naming them as a participant, `last_key` the newest; `times_met`
counts the closed episodes they were part of; `exchanges` the notable facts with them in which I took
part; `level` my relationship level towards them, from the newest fact that stated it.
"""

from __future__ import annotations

from collections.abc import Iterable

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.compress.summarize import counted, day_number
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import L0Record, StableFact, event_key, event_of
from mineworld_cognition.memory.render import LevelChange


class Stable:
    def __init__(self, store: SeatView) -> None:
        self._db = store.connection
        self._seat = store.seat_id

    def feed(self, record: L0Record, *, involved: bool, level: LevelChange | None) -> None:
        key = event_key(record.event)
        exchange = int(involved and record.salience == "notable")
        self._db.executemany(
            "INSERT INTO stable (seat, entity, first_key, last_key, times_met, exchanges, level, "
            "level_key, first_at, last_at) VALUES (?, ?, ?, ?, 0, ?, NULL, NULL, ?, ?) "
            "ON CONFLICT (seat, entity) DO UPDATE SET last_key = excluded.last_key, "
            "last_at = excluded.last_at, exchanges = exchanges + excluded.exchanges",
            [
                (self._seat, entity, key, key, exchange, record.at, record.at)
                for entity in record.counterparts
            ],
        )
        if level is not None:
            self._db.execute(
                "UPDATE stable SET level = ?, level_key = ? WHERE seat = ? AND entity = ?",
                (level.level, key, self._seat, level.counterpart),
            )

    def met(self, entities: Iterable[EntityId]) -> None:
        self._db.executemany(
            "UPDATE stable SET times_met = times_met + 1 WHERE seat = ? AND entity = ?",
            [(self._seat, entity) for entity in entities],
        )


def stable_facts(store: SeatView, limit: int) -> list[StableFact]:
    """The `limit` most met counterparts; ties by the earliest first meeting, then by entity id. Bounded,
    so retrieval never reads the whole table (F-S24-2)."""
    rows: list[tuple[str, str, str, int, int, int, int, str | None, str | None]] = (
        store.connection.execute(
            "SELECT entity, first_key, last_key, first_at, last_at, times_met, exchanges, level, "
            "level_key FROM stable WHERE seat = ? ORDER BY times_met DESC, first_key, entity LIMIT ?",
            (store.seat_id, limit),
        ).fetchall()
    )
    return [
        StableFact(
            entity=EntityId(entity),
            first=event_of(first),
            last=event_of(last),
            first_at=first_at,
            last_at=last_at,
            times_met=met,
            exchanges=exchanges,
            level=level,
            level_event=None if level_key is None else event_of(level_key),
        )
        for entity, first, last, first_at, last_at, met, exchanges, level, level_key in rows
    ]


def stable_text(fact: StableFact, names: NameBook) -> str:
    """`Bob Achterberg: met on day 1, last seen day 12, met 5 times, 9 exchanges, friendly.` From the stored
    times alone: the L0 rows of the facts it cites may have been merged or forgotten (F-FGb-2)."""
    text = f"{names.person(fact.entity)}: met on day {day_number(fact.first_at)}, "
    text += f"last seen day {day_number(fact.last_at)}, met {counted(fact.times_met, 'time')}, "
    text += counted(fact.exchanges, "exchange")
    return text + (f", {fact.level}." if fact.level else ".")
