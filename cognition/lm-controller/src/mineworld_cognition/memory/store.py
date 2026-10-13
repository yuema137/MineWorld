"""`MemoryStore`: one seat's memory, one SQLite file at a path the caller gives (D-P4-3; DEP-37).

The store never chooses a directory and never sits in a world's save (QS10-17). It opens a file with
schema v1, or refuses, by name and with both values, a file whose world instance, seat, self entity or
schema differ. `":memory:"` is the in-memory store tests use; there is no second implementation
(D-P4-13).

The schema belongs to this module; `ingest`, `compress` and `retrieve` read and write its tables
through `connection`, inside the store's `transaction()` when they write. Determinism is defined on
`dump()`, every table in key order as JSON Lines, never on the file's bytes (D-P4-8).
"""

from __future__ import annotations

import json
import sqlite3
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path
from types import TracebackType
from typing import Literal

from mineworld_sdk.wire.ids import EntityId, EventId, EventTypeId

from mineworld_cognition.memory import fts
from mineworld_cognition.memory.records import (
    SCHEMA_VERSION,
    EventKey,
    L0Record,
    Salience,
    StoreIdentity,
)

IdentityField = Literal["world_instance", "seat", "self_entity", "schema"]
type Cell = int | str | None

_SCHEMA = (
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    "CREATE TABLE l0 (event_key TEXT PRIMARY KEY, event_id TEXT NOT NULL, at INTEGER NOT NULL, "
    "kind TEXT NOT NULL, place TEXT, "
    "salience TEXT NOT NULL CHECK (salience IN ('notable', 'ambient')), "
    "mine INTEGER NOT NULL CHECK (mine IN (0, 1)), gist TEXT NOT NULL, digest TEXT NOT NULL)",
    "CREATE TABLE l0_who (event_key TEXT NOT NULL, entity TEXT NOT NULL, "
    "PRIMARY KEY (event_key, entity))",
    "CREATE TABLE episodes (episode INTEGER PRIMARY KEY, day INTEGER NOT NULL, place TEXT, "
    "first_key TEXT NOT NULL, last_key TEXT NOT NULL, opened_at INTEGER NOT NULL, "
    "closed_at INTEGER NOT NULL, closed INTEGER NOT NULL CHECK (closed IN (0, 1)), "
    "records INTEGER NOT NULL, text TEXT NOT NULL, prose TEXT, prose_key TEXT)",
    "CREATE INDEX episodes_by_last_key ON episodes (last_key)",
    "CREATE TABLE episode_who (episode INTEGER NOT NULL, entity TEXT NOT NULL, "
    "PRIMARY KEY (episode, entity))",
    "CREATE TABLE chapters (week INTEGER PRIMARY KEY, first_episode INTEGER NOT NULL, "
    "last_episode INTEGER NOT NULL, text TEXT NOT NULL, prose TEXT, prose_key TEXT)",
    "CREATE TABLE stable (entity TEXT PRIMARY KEY, first_key TEXT NOT NULL, last_key TEXT NOT NULL, "
    "times_met INTEGER NOT NULL, exchanges INTEGER NOT NULL, level TEXT, level_key TEXT)",
    "CREATE TABLE names (entity TEXT PRIMARY KEY, name TEXT NOT NULL, source_key TEXT)",
)
_FTS = f"CREATE VIRTUAL TABLE l0_text USING fts5(gist, tokenize='{fts.TOKENIZER}')"
"""Notable gists, `rowid` equal to their `l0` row's: an index, rebuilt from `l0`, never dumped."""

# Every table the dump writes, with the order of its rows. `l0_text` is an index, not content.
_DUMPED: tuple[tuple[str, str], ...] = (
    ("meta", "key"),
    ("names", "entity"),
    ("l0", "event_key"),
    ("l0_who", "event_key, entity"),
    ("episodes", "episode"),
    ("episode_who", "episode, entity"),
    ("chapters", "week"),
    ("stable", "entity"),
)


class StoreIdentityMismatch(ValueError):
    """The file is another seat's, another world's, or another schema's memory. Nothing was changed."""

    def __init__(self, path: str, field: IdentityField, stored: str, given: str) -> None:
        super().__init__(
            f"{path} is not this seat's memory store: its {field} is {stored!r}, and the seat's is "
            f"{given!r}"
        )
        self.field: IdentityField = field
        self.stored = stored
        self.given = given


class NotAMemoryStore(ValueError):
    """The file is a SQLite database with tables, but not a memory store (a save, for instance)."""


class MemoryStore:
    """One seat's memory. Close it (or use it as a context manager): Windows cannot delete a file a
    connection still holds."""

    def __init__(self, connection: sqlite3.Connection, identity: StoreIdentity) -> None:
        self._connection: sqlite3.Connection | None = connection
        self.identity = identity
        self.fts = fts.available() and _has_table(connection, "l0_text")

    @staticmethod
    def open(path: Path | Literal[":memory:"], identity: StoreIdentity) -> MemoryStore:
        where = str(path)
        connection = sqlite3.connect(where, isolation_level=None)
        try:
            if _has_table(connection, "meta"):
                _check_identity(connection, where, identity)
            elif connection.execute("SELECT count(*) FROM sqlite_master").fetchone()[0]:
                raise NotAMemoryStore(f"{where} holds tables but no memory store")
            else:
                _create(connection, identity)
        except BaseException:
            connection.close()
            raise
        return MemoryStore(connection, identity)

    @property
    def connection(self) -> sqlite3.Connection:
        if self._connection is None:
            raise ValueError("the memory store is closed")
        return self._connection

    @property
    def me(self) -> EntityId:
        return self.identity.self_entity

    @contextmanager
    def transaction(self) -> Generator[sqlite3.Connection]:
        """One `BEGIN IMMEDIATE … COMMIT`; any exception rolls every write back (P4-2)."""
        connection = self.connection
        connection.execute("BEGIN IMMEDIATE")
        try:
            yield connection
        except BaseException:
            connection.execute("ROLLBACK")
            raise
        connection.execute("COMMIT")

    def meta(self, key: str) -> str:
        row = self.connection.execute("SELECT value FROM meta WHERE key = ?", (key,)).fetchone()
        if row is None:
            raise KeyError(key)
        value: str = row[0]
        return value

    def set_meta(self, key: str, value: str) -> None:
        self.connection.execute(
            "INSERT INTO meta (key, value) VALUES (?, ?) "
            "ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            (key, value),
        )

    def cursor(self) -> EventId | None:
        """The perceived cursor: every fact at or below it is durably ingested (P4-2)."""
        value = self.meta("cursor")
        return EventId(value) if value else None

    def newest(self) -> EventKey | None:
        """The key of the newest ingested fact."""
        value = self.meta("newest")
        return EventKey(value) if value else None

    def dump(self) -> bytes:
        """Every table in key order, one JSON object per row with sorted keys: UTF-8 JSON Lines with no
        float and no wall-clock value. Two stores with the same content dump the same bytes."""
        lines: list[str] = []
        connection = self.connection
        for table, order in _DUMPED:
            cursor = connection.execute(f"SELECT * FROM {table} ORDER BY {order}")
            columns = [column[0] for column in cursor.description]
            for row in cursor:
                cells: dict[str, Cell] = dict(zip(columns, row, strict=True))
                cells["table"] = table
                lines.append(
                    json.dumps(cells, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
                )
        return ("\n".join(lines) + "\n").encode("utf-8")

    def close(self) -> None:
        if self._connection is not None:
            self._connection.close()
            self._connection = None

    def __enter__(self) -> MemoryStore:
        return self

    def __exit__(
        self,
        kind: type[BaseException] | None,
        error: BaseException | None,
        trace: TracebackType | None,
    ) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"MemoryStore(seat={self.identity.seat!r})"


def records_between(store: MemoryStore, first: EventKey, last: EventKey) -> list[L0Record]:
    """The L0 records from `first` to `last`, both included, in perceived order."""
    db = store.connection
    rows: list[tuple[str, str, int, str, str | None, Salience, int, str]] = db.execute(
        "SELECT event_key, event_id, at, kind, place, salience, mine, gist FROM l0 "
        "WHERE event_key BETWEEN ? AND ? ORDER BY event_key",
        (first, last),
    ).fetchall()
    who: dict[str, list[EntityId]] = {}
    for key, entity in db.execute(
        "SELECT event_key, entity FROM l0_who WHERE event_key BETWEEN ? AND ? ORDER BY rowid",
        (first, last),
    ):
        who.setdefault(key, []).append(EntityId(entity))
    return [
        L0Record(
            event=EventId(event),
            at=at,
            kind=EventTypeId(kind),
            place=None if place is None else EntityId(place),
            salience=salience,
            mine=bool(mine),
            gist=gist,
            counterparts=tuple(who.get(key, ())),
        )
        for key, event, at, kind, place, salience, mine, gist in rows
    ]


def _has_table(connection: sqlite3.Connection, name: str) -> bool:
    row = connection.execute("SELECT 1 FROM sqlite_master WHERE name = ?", (name,)).fetchone()
    return row is not None


def _identity_rows(identity: StoreIdentity) -> tuple[tuple[IdentityField, str], ...]:
    return (
        ("world_instance", identity.world_instance),
        ("seat", identity.seat),
        ("self_entity", identity.self_entity),
        ("schema", str(identity.schema_version)),
    )


def _check_identity(connection: sqlite3.Connection, where: str, identity: StoreIdentity) -> None:
    stored = dict(connection.execute("SELECT key, value FROM meta").fetchall())
    for field, given in _identity_rows(identity):
        if stored.get(field) != given:
            raise StoreIdentityMismatch(where, field, str(stored.get(field)), given)


def _create(connection: sqlite3.Connection, identity: StoreIdentity) -> None:
    if identity.schema_version != SCHEMA_VERSION:
        raise StoreIdentityMismatch(
            "(new store)", "schema", str(SCHEMA_VERSION), str(identity.schema_version)
        )
    connection.execute("BEGIN IMMEDIATE")
    try:
        for statement in _SCHEMA:
            connection.execute(statement)
        if fts.available():
            connection.execute(_FTS)
        rows = [*_identity_rows(identity), ("cursor", ""), ("newest", "")]
        connection.executemany("INSERT INTO meta (key, value) VALUES (?, ?)", rows)
    except BaseException:
        connection.execute("ROLLBACK")
        raise
    connection.execute("COMMIT")
