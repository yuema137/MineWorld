"""`MemoryHome`: the memory of every seat one cognition process hosts in one world instance, in one SQLite file
(pr-s28a-memory-home.md §§5.2, 5.4, 5.5; step-28 §§4, 6; `ARC-98`, `ARC-109`).

The home never chooses a directory and never sits in a world's save. It opens a file of schema 5 or creates one,
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
from collections.abc import Callable, Generator
from contextlib import contextmanager
from pathlib import Path
from types import TracebackType
from typing import Literal

from mineworld_sdk.wire.ids import EntityId, EntityKey, WorldInstanceId

from mineworld_cognition.memory import fts
from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.home.encoding import (
    Cell,
    Codec,
    as_number,
    encoded,
    line,
    line_bytes,
    raw,
)
from mineworld_cognition.memory.home.errors import (
    HolderInvariantBroken,
    HomeBusy,
    HomeIdentityMismatch,
    NotAMemoryHome,
    SeatNotInHome,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.home.view import METER_GONE, METER_PUT, METERED, SeatMeter, SeatView
from mineworld_cognition.memory.records import StoreIdentity
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
        self._views: dict[int, SeatView] = {}
        self._meters: dict[int, SeatMeter] = {}
        """Each measured seat's meter; empty until a seat is measured, and after a rollback."""
        self._triggers = False

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
        self._meters.clear()

    def check(self) -> None:
        """I-S28-4 (the records are exactly the held keys, and every participant row has its record), then every
        measured seat's meter against its dump."""
        broken = world.check_holders(self.connection)
        if broken:
            raise HolderInvariantBroken(broken)
        for seat, meter in self._meters.items():
            view = self._views[seat]
            meta = sum(size for _, size in view.row_bytes("meta"))
            if meter.content != len(view.dump()) - meta:
                raise HolderInvariantBroken(seat=view.identity.seat)

    # ── Seats ─────────────────────────────────────────────────────────────────────────────────────

    def seat(self, identity: StoreIdentity) -> SeatView:
        """The seat's view, its `seats` row and `meta` rows created on first use. Refuses another world
        instance's identity, and a seat whose stored self entity differs, by name; nothing is changed."""
        if identity.world_instance != self.world_instance:
            raise HomeIdentityMismatch(
                "world_instance", self.world_instance, identity.world_instance
            )
        db = self.connection
        row: tuple[int, str] | None = db.execute(
            "SELECT seat, self_entity FROM seats WHERE key = ?", (identity.seat,)
        ).fetchone()
        if row is not None:
            seat, stored = row
            if stored != identity.self_entity:
                raise HomeIdentityMismatch("self_entity", stored, identity.self_entity)
        elif db.in_transaction:
            seat = _new_seat(db, identity)
        else:
            with self.transaction():
                seat = _new_seat(db, identity)
        view = self._views.get(seat)
        if view is None:
            view = self._views[seat] = SeatView(self, seat, identity)
        return view

    def view_of(self, key: EntityKey) -> SeatView:
        """The view of a seat the home already has (`SeatNotInHome` otherwise); creates nothing."""
        row: tuple[int, str] | None = self.connection.execute(
            "SELECT seat, self_entity FROM seats WHERE key = ?", (key,)
        ).fetchone()
        if row is None:
            raise SeatNotInHome(key)
        seat, self_entity = row
        view = self._views.get(seat)
        if view is None:
            identity = StoreIdentity(
                world_instance=self.world_instance, seat=key, self_entity=EntityId(self_entity)
            )
            view = self._views[seat] = SeatView(self, seat, identity)
        return view

    def remove_seat(self, key: EntityKey) -> None:
        """A seat leaving the home (step-28 §6.4): every key it holds released, with collection, and its
        private rows deleted, in one transaction."""
        view = self.view_of(key)
        db = self.connection
        with self.transaction():
            held: list[tuple[str]] = db.execute(
                "SELECT event_key FROM access WHERE seat = ?", (view.seat_id,)
            ).fetchall()
            view.release(held_key for (held_key,) in held)
            for table in schema.SEAT_TABLES:
                db.execute(f"DELETE FROM {table.name} WHERE seat = ?", (view.seat_id,))
            db.execute("DELETE FROM seats WHERE seat = ?", (view.seat_id,))
            self._meters.pop(view.seat_id, None)
        self._views.pop(view.seat_id, None)

    # ── The meter (pr-s28a §5.7; FG-b2 §4.2) ─────────────────────────────────────────────────────

    def meter(self, seat: int) -> SeatMeter | None:
        return self._meters.get(seat)

    def measure(self, view: SeatView) -> SeatMeter:
        """Measures one seat in full and keeps its meter current from then on: SQLite `TEMP` triggers, in this
        connection only and never in the file, hand every private row a write puts or removes to the home,
        which encodes it as `dump()` would."""
        if not self._triggers:
            _install_triggers(self.connection, self._put, self._gone)
            self._triggers = True
        meter = view.measured()
        self._meters[view.seat_id] = meter
        return meter

    def _put(self, table: str, seat: int, rid: int, *values: Cell) -> None:
        meter = self._meters.get(seat)
        if meter is not None:
            view = schema.BY_NAME[table]
            meter.put(table, rid, line_bytes(table, values, view.codecs))

    def _gone(self, table: str, seat: int, rid: int) -> None:
        meter = self._meters.get(seat)
        if meter is not None:
            meter.gone(table, rid)

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


def _new_seat(db: sqlite3.Connection, identity: StoreIdentity) -> int:
    row = db.execute(
        "INSERT INTO seats (key, self_entity) VALUES (?, ?)", (identity.seat, identity.self_entity)
    )
    seat = row.lastrowid
    assert seat is not None
    db.executemany(
        "INSERT INTO meta (seat, key, value) VALUES (?, ?, ?)",
        [(seat, key, value) for key, value in schema.SEAT_META],
    )
    return seat


def _install_triggers(
    db: sqlite3.Connection,
    put: Callable[..., None],
    gone: Callable[[str, int, int], None],
) -> None:
    db.create_function(METER_PUT, -1, put)
    db.create_function(METER_GONE, 3, gone)
    for table in METERED:
        new = ", ".join(f"NEW.{column}" for column in table.columns)
        put_sql = f"SELECT {METER_PUT}('{table.name}', NEW.seat, NEW.rowid, {new});"
        gone_sql = f"SELECT {METER_GONE}('{table.name}', OLD.seat, OLD.rowid);"
        for event, body in (
            ("INSERT", put_sql),
            # An update keeps the row's rowid (nothing here updates a rowid), and `put` replaces the size the
            # row had: one call, not a removal and a write.
            ("UPDATE", put_sql),
            ("DELETE", gone_sql),
        ):
            db.execute(
                f"CREATE TEMP TRIGGER IF NOT EXISTS meter_{table.name}_{event.lower()} AFTER {event} "
                f"ON main.{table.name} BEGIN {body} END"
            )


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
