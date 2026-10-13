"""L3: per counterpart, what I know of them and the facts that establish it (§5.5).

`first_key` is the first perceived fact naming them as a participant, `last_key` the newest; `times_met`
counts the closed episodes they were part of; `exchanges` the notable facts with them in which I took
part; `level` my relationship level towards them, from the newest fact that stated it.
"""

from __future__ import annotations

from collections.abc import Iterable

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.records import L0Record, StableFact, event_key, event_of
from mineworld_cognition.memory.render import LevelChange
from mineworld_cognition.memory.store import MemoryStore


class Stable:
    def __init__(self, store: MemoryStore) -> None:
        self._db = store.connection

    def feed(self, record: L0Record, *, involved: bool, level: LevelChange | None) -> None:
        key = event_key(record.event)
        exchange = int(involved and record.salience == "notable")
        self._db.executemany(
            "INSERT INTO stable (entity, first_key, last_key, times_met, exchanges, level, level_key) "
            "VALUES (?, ?, ?, 0, ?, NULL, NULL) ON CONFLICT (entity) DO UPDATE SET "
            "last_key = excluded.last_key, exchanges = exchanges + excluded.exchanges",
            [(entity, key, key, exchange) for entity in record.counterparts],
        )
        if level is not None:
            self._db.execute(
                "UPDATE stable SET level = ?, level_key = ? WHERE entity = ?",
                (level.level, key, level.counterpart),
            )

    def met(self, entities: Iterable[EntityId]) -> None:
        self._db.executemany(
            "UPDATE stable SET times_met = times_met + 1 WHERE entity = ?",
            [(entity,) for entity in entities],
        )


def stable_facts(store: MemoryStore) -> list[StableFact]:
    """Every counterpart, most met first; ties by the earliest first meeting, then by entity id."""
    rows: list[tuple[str, str, str, int, int, str | None, str | None]] = store.connection.execute(
        "SELECT entity, first_key, last_key, times_met, exchanges, level, level_key FROM stable "
        "ORDER BY times_met DESC, first_key, entity"
    ).fetchall()
    return [
        StableFact(
            entity=EntityId(entity),
            first=event_of(first),
            last=event_of(last),
            times_met=met,
            exchanges=exchanges,
            level=level,
            level_event=None if level_key is None else event_of(level_key),
        )
        for entity, first, last, met, exchanges, level, level_key in rows
    ]
