"""`MemoryHome`: the memory of every seat one cognition process hosts in one world instance, in one SQLite file
(pr-s28a-memory-home.md §§5.2, 5.4, 5.5; step-28 §§4, 6; `ARC-98`, `ARC-109`).

The home never chooses a directory and never sits in a world's save. It opens a file of schema 4 or creates one,
and refuses, by name, a P4/FG-b store (`UnsupportedStoreSchema`, never modified: memory is rebuilt from the
perceived stream, QS28-8), a database that is neither (`NotAMemoryHome`), and another world instance's home
(`HomeIdentityMismatch`). A file is held with SQLite's own exclusive lock until `close()`, so a second process
gets `HomeBusy` and a killed process leaves no stale lock (`DEP-61`). `":memory:"` is the in-memory home tests
use; there is no second implementation.

One transaction covers every write of one call, across seats (step-28 §6.3). Determinism is defined on
`dump()`, the canonical positional lines (§5.4), never on the file's bytes.
"""

from __future__ import annotations

import sqlite3
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path
from types import TracebackType
from typing import Literal

from mineworld_sdk.wire.ids import EntityKey, WorldInstanceId

from mineworld_cognition.memory import fts
from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.home.encoding import Codec, as_number, encoded, line, raw
from mineworld_cognition.memory.home.errors import (
    HolderInvariantBroken,
    HomeBusy,
    HomeIdentityMismatch,
    NotAMemoryHome,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.scopes import world

type Verify = Literal["off", "pass", "every-commit"]
"""When the home checks I-S28-4 and every seat's meter (step-28 §5.4): never, before each retention pass's
commit, or before every commit. A failed check raises `HolderInvariantBroken` and rolls the transaction back."""

_FACT_CODECS: tuple[Codec, ...] = tuple(schema.codec(column) for column in schema.FACT_COLUMNS)
_ACCESS_CODECS: tuple[Codec, ...] = (as_number, raw, raw, raw, raw)


def _commit(connection: sqlite3.Connection) -> None:
    connection.execute("COMMIT")


class MemoryHome:
    """One home. Close it (or use it as a context manager): the lock is held until then, and Windows cannot delete
    a file a connection still holds."""

    def __init__(
        self,
        connection: sqlite3.Connection,
        path: str,
        world_instance: WorldInstanceId,
        verify: Verify,
    ) -> None:
        self._connection: sqlite3.Connection | None = connection
        self.path = path
        self.world_instance = world_instance
        self.verify: Verify = verify
        self.fts = fts.available() and _has_table(connection, "access_text")

    @staticmethod
    def open(
        path: Path | Literal[":memory:"],
        world_instance: WorldInstanceId,
        *,
        verify: Verify = "off",
    ) -> MemoryHome:
        where = str(path)
        connection = sqlite3.connect(where, isolation_level=None, timeout=0)
        try:
            _lock(connection, where)
            if _has_table(connection, "home_meta"):
                _check(connection, where, world_instance)
            elif _has_table(connection, "meta"):
                found = connection.execute("SELECT value FROM meta WHERE key = 'schema'").fetchone()
                if found is None:
                    raise NotAMemoryHome(where)
                raise UnsupportedStoreSchema(where, str(found[0]), schema.SCHEMA_VERSION)
            elif connection.execute("SELECT count(*) FROM sqlite_master").fetchone()[0]:
                raise NotAMemoryHome(where)
            else:
                _create(connection, world_instance)
        except BaseException:
            connection.close()
            raise
        return MemoryHome(connection, where, world_instance, verify)

    @property
    def connection(self) -> sqlite3.Connection:
        if self._connection is None:
            raise ValueError("memory.home: the home is closed")
        return self._connection

    @contextmanager
    def transaction(self, *, retention_pass: bool = False) -> Generator[sqlite3.Connection]:
        """One `BEGIN IMMEDIATE … COMMIT` over the whole home; any exception rolls every seat's writes back.
        Under `verify`, the check runs before the commit, so a broken state is never committed."""
        connection = self.connection
        connection.execute("BEGIN IMMEDIATE")
        try:
            yield connection
            if self.verify == "every-commit" or (retention_pass and self.verify == "pass"):
                self.check()
        except BaseException:
            connection.execute("ROLLBACK")
            self._rolled_back()
            raise
        _commit(connection)

    def _rolled_back(self) -> None:
        """Every cached size is of rows that may no longer exist: dropped, so the next measure is in full."""

    def check(self) -> None:
        """I-S28-4: the records are exactly the held keys, and every participant row has its record."""
        broken = world.check_holders(self.connection)
        if broken:
            raise HolderInvariantBroken(broken)

    def seats(self) -> tuple[EntityKey, ...]:
        rows: list[tuple[str]] = self.connection.execute(
            "SELECT key FROM seats ORDER BY key"
        ).fetchall()
        return tuple(EntityKey(key) for (key,) in rows)

    def dump(self) -> bytes:
        """The whole home in canonical lines (§5.4): `home_meta`; `fact`; `fact_who`, one line per record with
        its rows' `role, entity` pairs in ordinal order; then, per seat in key order, `["seat", key,
        self_entity]` and the seat's rows in the seat view's table order, `access` where the view has `l0`.
        `seats.seat` never appears. A function of each seat's perceived prefix and configuration (I-S28-5)."""
        db = self.connection
        out = [
            line("home_meta", row, (raw, raw))
            for row in db.execute("SELECT key, value FROM home_meta ORDER BY key")
        ]
        out += [
            line("fact", row, _FACT_CODECS)
            for row in db.execute(
                "SELECT event_id, at, kind, place, digest FROM fact ORDER BY event_key"
            )
        ]
        out += _fact_who_lines(db)
        seats: list[tuple[int, str, str]] = db.execute(
            "SELECT seat, key, self_entity FROM seats ORDER BY key"
        ).fetchall()
        for seat, key, self_entity in seats:
            out.append(line("seat", (key, self_entity), (raw, raw)))
            out += _seat_lines(db, seat)
        return ("\n".join(out) + "\n").encode("utf-8")

    def reclaim(self) -> None:
        """Returns the pages freed by deleted rows to the file system, outside any transaction."""
        self.connection.execute("PRAGMA incremental_vacuum").fetchall()

    def close(self) -> None:
        if self._connection is not None:
            self._connection.close()
            self._connection = None

    def __enter__(self) -> MemoryHome:
        return self

    def __exit__(
        self,
        kind: type[BaseException] | None,
        error: BaseException | None,
        trace: TracebackType | None,
    ) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"MemoryHome({self.path!r})"


def _fact_who_lines(db: sqlite3.Connection) -> list[str]:
    lines: list[str] = []
    current: str | None = None
    cells: list[int | str | None] = []
    for key, entity, role in db.execute(
        "SELECT event_key, entity, role FROM fact_who ORDER BY event_key, ordinal"
    ):
        if key != current:
            if current is not None:
                lines.append(encoded("fact_who", cells))
            current, cells = key, [as_number(key)]
        cells += [role, entity]
    if current is not None:
        lines.append(encoded("fact_who", cells))
    return lines


def _seat_lines(db: sqlite3.Connection, seat: int) -> list[str]:
    """One seat's section of the home dump, after its header."""
    out = [
        line("meta", (key, value), (raw, schema.meta_codec(key)))
        for key, value in db.execute(
            "SELECT key, value FROM meta WHERE seat = ? ORDER BY key", (seat,)
        )
    ]
    out += _table_lines(db, seat, schema.NAMES)
    out += [
        line("access", row, _ACCESS_CODECS)
        for row in db.execute(
            "SELECT event_key, gist, salience, mine, importance FROM access WHERE seat = ? "
            "ORDER BY event_key",
            (seat,),
        )
    ]
    for table in schema.PRIVATE:
        out += _table_lines(db, seat, table)
    return out


def _table_lines(db: sqlite3.Connection, seat: int, table: schema.ViewTable) -> list[str]:
    codecs = table.codecs
    return [
        line(table.name, row, codecs)
        for row in db.execute(
            f"SELECT {', '.join(table.columns)} FROM {table.name} WHERE seat = ? "
            f"ORDER BY {table.order}",
            (seat,),
        )
    ]


def _has_table(connection: sqlite3.Connection, name: str) -> bool:
    row = connection.execute("SELECT 1 FROM sqlite_master WHERE name = ?", (name,)).fetchone()
    return row is not None


def _lock(connection: sqlite3.Connection, where: str) -> None:
    """SQLite's own exclusive lock, held until the connection closes (`DEP-61`): `locking_mode = EXCLUSIVE`, then
    one exclusive transaction to take it. With `timeout = 0` a held file fails at once.

    `auto_vacuum` is set on an empty database only, before the lock's transaction (which writes an empty file's
    header), because SQLite ignores it once the first table exists. Freed pages are returned by `reclaim()` after
    each retention pass (pr-fg-b §5.5). It is never set on a database that has pages: there the pragma rewrites
    the header (its change counter) even when the value is the same, and a refused file is never modified
    (I-a-10)."""
    if where == ":memory:":
        connection.execute("PRAGMA auto_vacuum = INCREMENTAL")
        return
    connection.execute("PRAGMA locking_mode = EXCLUSIVE")
    try:
        if connection.execute("PRAGMA page_count").fetchone()[0] == 0:
            connection.execute("PRAGMA auto_vacuum = INCREMENTAL")
        connection.execute("BEGIN EXCLUSIVE")
        connection.execute("COMMIT")
    except sqlite3.OperationalError as error:
        if "locked" in str(error) or "busy" in str(error):
            raise HomeBusy(where) from error
        raise


def _check(connection: sqlite3.Connection, where: str, world_instance: WorldInstanceId) -> None:
    stored = dict(connection.execute("SELECT key, value FROM home_meta").fetchall())
    if stored.get("schema") != str(schema.SCHEMA_VERSION):
        raise UnsupportedStoreSchema(where, str(stored.get("schema")), schema.SCHEMA_VERSION)
    if stored.get("world_instance") != world_instance:
        raise HomeIdentityMismatch(
            "world_instance", str(stored.get("world_instance")), world_instance
        )


def _create(connection: sqlite3.Connection, world_instance: WorldInstanceId) -> None:
    connection.execute("BEGIN IMMEDIATE")
    try:
        for statement in schema.HOME_DDL:
            connection.execute(statement)
        if fts.available():
            connection.execute(schema.FTS_DDL)
        connection.executemany(
            "INSERT INTO home_meta (key, value) VALUES (?, ?)",
            [("schema", str(schema.SCHEMA_VERSION)), ("world_instance", world_instance)],
        )
    except BaseException:
        connection.execute("ROLLBACK")
        raise
    _commit(connection)
