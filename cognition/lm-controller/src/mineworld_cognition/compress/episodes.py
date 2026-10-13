"""L1: the episode roll-up, decided by the facts alone, never by batch boundaries (D-P4-7; QP4-2).

An episode is a maximal run of consecutive L0 records, in perceived order, such that each record

- is at the episode's place (a record with no place, or an `ambient` one, does not test the place);
- follows the previous record by at most 1 800 s of world time;
- falls on the episode's day (`at // 86 400`, QP4-10).

A record that does not fit closes the open episode and opens the next, so an `ambient` record (another
person's comings and goings) never splits an episode by place; it opens one only when none is open.
The counterparts are the union of the records'. Episodes are numbered in order, so the open episode's
number is the one it will close with. It is stored, open, between ingest calls, so a store resumes it
exactly; `closed` turns it into a summary.
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass, field

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.compress.stable import Stable
from mineworld_cognition.compress.summarize import DAY_S, Summarizer
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Episode, EventKey, L0Record, event_key, event_of
from mineworld_cognition.memory.store import MemoryStore, records_between

GAP_S = 1800


@dataclass
class _Open:
    number: int
    day: int
    place: EntityId | None
    first_key: EventKey
    last_key: EventKey
    opened_at: int
    last_at: int
    records: int
    who: dict[EntityId, None] = field(default_factory=dict[EntityId, None])  # an ordered set


class Episodes:
    """The roll-up over one store, fed one record at a time inside ingestion's transaction."""

    def __init__(
        self, store: MemoryStore, names: NameBook, summarizer: Summarizer, stable: Stable
    ) -> None:
        self._store = store
        self._db = store.connection
        self._names = names
        self._summarizer = summarizer
        self._stable = stable
        self._open = _load_open(self._db)
        self.closed = 0

    def feed(self, record: L0Record) -> None:
        current = self._open
        if current is not None and _fits(current, record):
            self._join(current, record)
            return
        place = record.place if record.place is not None or current is None else current.place
        if current is not None:
            self._close(current)
        number = current.number + 1 if current is not None else _next_number(self._db)
        key = event_key(record.event)
        opened = _Open(number, record.at // DAY_S, place, key, key, record.at, record.at, 0)
        self._db.execute(
            "INSERT INTO episodes (episode, day, place, first_key, last_key, opened_at, closed_at, "
            "closed, records, text, prose, prose_key) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, '', NULL, "
            "NULL)",
            (number, opened.day, place, key, key, record.at, record.at),
        )
        self._open = opened
        self._join(opened, record)

    def _join(self, current: _Open, record: L0Record) -> None:
        current.last_key = event_key(record.event)
        current.last_at = record.at
        current.records += 1
        new = [c for c in record.counterparts if c not in current.who]
        current.who.update(dict.fromkeys(new))
        self._db.execute(
            "UPDATE episodes SET last_key = ?, closed_at = ?, records = ? WHERE episode = ?",
            (current.last_key, current.last_at, current.records, current.number),
        )
        self._db.executemany(
            "INSERT INTO episode_who (episode, entity) VALUES (?, ?)",
            [(current.number, entity) for entity in new],
        )

    def _close(self, current: _Open) -> None:
        records = records_between(self._store, current.first_key, current.last_key)
        text = self._summarizer.episode(records, self._names, place=current.place)
        self._db.execute(
            "UPDATE episodes SET closed = 1, text = ? WHERE episode = ?", (text, current.number)
        )
        self._stable.met(current.who)
        self._open = None
        self.closed += 1


def _fits(current: _Open, record: L0Record) -> bool:
    if record.at - current.last_at > GAP_S or record.at // DAY_S != current.day:
        return False
    return record.salience == "ambient" or record.place is None or record.place == current.place


def _next_number(db: sqlite3.Connection) -> int:
    highest: int | None = db.execute("SELECT max(episode) FROM episodes").fetchone()[0]
    return 1 if highest is None else highest + 1


def _load_open(db: sqlite3.Connection) -> _Open | None:
    row: tuple[int, int, str | None, str, str, int, int, int] | None = db.execute(
        "SELECT episode, day, place, first_key, last_key, opened_at, closed_at, records "
        "FROM episodes WHERE closed = 0"
    ).fetchone()
    if row is None:
        return None
    number, day, place, first, last, opened_at, last_at, records = row
    who: list[tuple[str]] = db.execute(
        "SELECT entity FROM episode_who WHERE episode = ? ORDER BY rowid", (number,)
    ).fetchall()
    return _Open(
        number,
        day,
        None if place is None else EntityId(place),
        EventKey(first),
        EventKey(last),
        opened_at,
        last_at,
        records,
        dict.fromkeys(EntityId(entity) for (entity,) in who),
    )


def load_episodes(
    store: MemoryStore, where: str, parameters: tuple[int | str, ...]
) -> list[Episode]:
    """Episodes matching a SQL condition over `episodes`, in the condition's `ORDER BY` if it has one."""
    db = store.connection
    rows: list[tuple[int, int, str | None, str, str, int, int, int, int, str, str | None]] = (
        db.execute(
            "SELECT episode, day, place, first_key, last_key, opened_at, closed_at, closed, records, "
            f"text, prose FROM episodes WHERE {where}",
            parameters,
        ).fetchall()
    )
    episodes: list[Episode] = []
    for number, day, place, first, last, opened_at, closed_at, closed, records, text, prose in rows:
        who: list[tuple[str]] = db.execute(
            "SELECT entity FROM episode_who WHERE episode = ? ORDER BY rowid", (number,)
        ).fetchall()
        episodes.append(
            Episode(
                number=number,
                day=day,
                place=None if place is None else EntityId(place),
                first=event_of(first),
                last=event_of(last),
                opened_at=opened_at,
                closed_at=closed_at,
                closed=bool(closed),
                records=records,
                counterparts=tuple(EntityId(entity) for (entity,) in who),
                text=text,
                prose=prose,
            )
        )
    return episodes
