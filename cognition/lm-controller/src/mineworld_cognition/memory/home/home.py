"""`MemoryHome`: the memory of every seat one cognition process hosts in one world instance, in one SQLite file
(pr-s28a-memory-home.md §§5.2, 5.4, 5.5; step-28 §§4, 6; `ARC-98`, `ARC-109`).

The home never chooses a directory and never sits in a world's save. It opens a file of schema 6 or creates one,
and refuses, by name, a P4/FG-b store (`UnsupportedStoreSchema`, never modified: memory is rebuilt from the
perceived stream, QS28-8), a database that is neither (`NotAMemoryHome`), and another world instance's home
(`HomeIdentityMismatch`). A file is held with SQLite's own exclusive lock until `close()`, so a second process
gets `HomeBusy` and a killed process leaves no stale lock (`DEP-61`). `":memory:"` is the in-memory home tests
use; there is no second implementation.

One transaction covers every write of one call, across seats (step-28 §6.3). Determinism is defined on
`dump()`, the canonical positional lines (§5.4), never on the file's bytes.
"""

from __future__ import annotations

import secrets
import sqlite3
from collections.abc import Callable, Generator
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from types import TracebackType
from typing import TYPE_CHECKING, Literal

from mineworld_sdk.wire.ids import EntityId, EntityKey, WorldInstanceId

from mineworld_cognition.memory import fts
from mineworld_cognition.memory.home import schema, scope_tables
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
    HomeNeedsRecovery,
    NotAMemoryHome,
    SeatNotInHome,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.home.scope_tables import ScopeSync
from mineworld_cognition.memory.home.view import METER_GONE, METER_PUT, METERED, SeatMeter, SeatView
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.scopes import world

if TYPE_CHECKING:
    from mineworld_cognition.memory.scopes.stack import MemoryStack

type Verify = Literal["off", "pass", "every-commit"]
"""When the home checks I-S28-4 and every seat's meter (step-28 §5.4): never, before each retention pass's
commit, or before every commit. A failed check raises `HolderInvariantBroken` and rolls the transaction back."""

_FACT_CODECS: tuple[Codec, ...] = tuple(schema.codec(column) for column in schema.FACT_COLUMNS)
_ACCESS_CODECS: tuple[Codec, ...] = (as_number, raw, raw, raw, raw)
READ_ONLY = "memory.home: the home is open read-only"


def _commit(connection: sqlite3.Connection) -> None:
    connection.execute("COMMIT")


@dataclass(frozen=True)
class HomeTotals:
    """Where a home's bytes are (pr-s28e §5.5): each segment of `dump()`, newlines included, and the file's
    pages. `home_meta + fact + fact_who + Σ seats == dump`."""

    home_meta: int
    fact: int
    fact_who: int
    seats: tuple[tuple[EntityKey, int], ...]
    """Each seat's section, its `["seat", …]` line included, in key order."""
    dump: int
    file: int
    """`page_count * page_size`."""
    free: int
    """`freelist_count * page_size`: pages `reclaim()` would return."""


@dataclass(frozen=True)
class _DumpParts:
    home_meta: list[str]
    fact: list[str]
    fact_who: list[str]
    seats: list[tuple[EntityKey, list[str]]]

    def lines(self) -> list[str]:
        out = [*self.home_meta, *self.fact, *self.fact_who]
        for _, section in self.seats:
            out += section
        return out


def _bytes(lines: list[str]) -> int:
    return sum(len(text.encode("utf-8")) + 1 for text in lines)


class MemoryHome:
    """One home. Close it (or use it as a context manager): the lock is held until then, and Windows cannot delete
    a file a connection still holds."""

    def __init__(
        self,
        connection: sqlite3.Connection,
        path: str,
        world_instance: WorldInstanceId,
        verify: Verify,
        created: str,
        *,
        read_only: bool = False,
    ) -> None:
        self._connection: sqlite3.Connection | None = connection
        self.read_only = read_only
        """Opened by `open_read_only`: every write path raises before touching SQLite (pr-s28e §5.2)."""
        self.path = path
        self.world_instance = world_instance
        self.created = created
        """The home's creation marker (`schema.CREATED_KEY`, `ARC-152`): the same for as long as the file lives,
        different for every new home. Never dumped."""
        self.verify: Verify = verify
        self.fts = fts.available() and _has_table(connection, "access_text")
        self._views: dict[int, SeatView] = {}
        self._meters: dict[int, SeatMeter] = {}
        """Each measured seat's meter; empty until a seat is measured, and after a rollback."""
        self._triggers = False
        self._hosted: tuple[schema.ViewTable, ...] | None = None
        """The registered scope tables (pr-s28d §5.4), read once and again after a registration or rollback."""

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
                created = _marked(connection)
            elif _has_table(connection, "meta"):
                found = connection.execute("SELECT value FROM meta WHERE key = 'schema'").fetchone()
                if found is None:
                    raise NotAMemoryHome(where)
                raise UnsupportedStoreSchema(where, str(found[0]), schema.SCHEMA_VERSION)
            elif connection.execute("SELECT count(*) FROM sqlite_master").fetchone()[0]:
                raise NotAMemoryHome(where)
            else:
                created = _create(connection, world_instance)
        except BaseException:
            connection.close()
            raise
        return MemoryHome(connection, where, world_instance, verify, created)

    @staticmethod
    def open_read_only(path: Path) -> MemoryHome:
        """A stopped home, for inspection (pr-s28e §5.2; F-S28e-2): SQLite's `mode=ro`, the world instance read
        from the file, nothing created, marked or locked for writing. Refused by name: a missing path
        (`NotAMemoryHome`, nothing created), a home another process holds (`HomeBusy`), a hot journal a killed
        process left (`HomeNeedsRecovery`: rolling it back is a write), and every refusal of `open`."""
        where = str(path)
        if not path.is_file():
            raise NotAMemoryHome(where, missing=True)
        connection = sqlite3.connect(
            path.absolute().as_uri() + "?mode=ro", uri=True, isolation_level=None, timeout=0
        )
        try:
            try:
                has_home = _has_table(connection, "home_meta")
            except sqlite3.Error as error:
                refusal = _read_only_refusal(error, where)
                if refusal is error:
                    raise
                raise refusal from error
            if not has_home:
                if _has_table(connection, "meta"):
                    found = connection.execute(
                        "SELECT value FROM meta WHERE key = 'schema'"
                    ).fetchone()
                    if found is not None:
                        raise UnsupportedStoreSchema(where, str(found[0]), schema.SCHEMA_VERSION)
                raise NotAMemoryHome(where)
            stored: dict[str, str] = dict(
                connection.execute("SELECT key, value FROM home_meta").fetchall()
            )
            if stored.get("schema") != str(schema.SCHEMA_VERSION):
                raise UnsupportedStoreSchema(
                    where, str(stored.get("schema")), schema.SCHEMA_VERSION
                )
            instance = WorldInstanceId(stored.get("world_instance", ""))
            created = stored.get(schema.CREATED_KEY, "")
        except BaseException:
            connection.close()
            raise
        return MemoryHome(connection, where, instance, "off", created, read_only=True)

    def writable(self) -> None:
        """Raises on a read-only home; every write path calls it before touching SQLite."""
        if self.read_only:
            raise ValueError(READ_ONLY)

    @property
    def connection(self) -> sqlite3.Connection:
        if self._connection is None:
            raise ValueError("memory.home: the home is closed")
        return self._connection

    @contextmanager
    def transaction(self, *, retention_pass: bool = False) -> Generator[sqlite3.Connection]:
        """One `BEGIN IMMEDIATE … COMMIT` over the whole home; any exception rolls every seat's writes back.
        Under `verify`, the check runs before the commit, so a broken state is never committed."""
        self.writable()
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
        self._hosted = None

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
        self.writable()
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
        self.writable()
        view = self.view_of(key)
        db = self.connection
        with self.transaction():
            held: list[tuple[str]] = db.execute(
                "SELECT event_key FROM access WHERE seat = ?", (view.seat_id,)
            ).fetchall()
            view.release(held_key for (held_key,) in held)
            for table in schema.SEAT_TABLES:
                db.execute(f"DELETE FROM {table.name} WHERE seat = ?", (view.seat_id,))
            scope_tables.drop_seat(db, view.seat_id)
            db.execute("DELETE FROM seats WHERE seat = ?", (view.seat_id,))
            self._meters.pop(view.seat_id, None)
        self._views.pop(view.seat_id, None)

    # ── The meter (pr-s28a §5.7; FG-b2 §4.2) ─────────────────────────────────────────────────────

    def meter(self, seat: int) -> SeatMeter | None:
        return self._meters.get(seat)

    def measure(self, view: SeatView) -> SeatMeter:
        """Measures one seat in full and keeps its meter current from then on: SQLite `TEMP` triggers, in this
        connection only and never in the file, hand every private row a write puts or removes to the home,
        which encodes it as `dump()` would. A read-only home installs no trigger and keeps no meter: each call
        measures in full."""
        if self.read_only:
            return view.measured()
        if not self._triggers:
            _install_triggers(self.connection, self._put, self._gone)
            self._triggers = True
        for table in self.hosted():
            _table_triggers(self.connection, table)
        meter = view.measured()
        self._meters[view.seat_id] = meter
        return meter

    def _put(self, table: str, seat: int, rid: int, *values: Cell) -> None:
        meter = self._meters.get(seat)
        if meter is not None:
            view = self.table(table)
            meter.put(table, rid, line_bytes(table, values, view.codecs))

    def _gone(self, table: str, seat: int, rid: int) -> None:
        meter = self._meters.get(seat)
        if meter is not None:
            meter.gone(table, rid)

    # ── Scope tables (pr-s28d §5.4) ──────────────────────────────────────────────────────────────

    def hosted(self) -> tuple[schema.ViewTable, ...]:
        """The registered scope tables as view tables, in table-name order."""
        if self._hosted is None:
            self._hosted = scope_tables.hosted(self.connection)
        return self._hosted

    def table(self, name: str) -> schema.ViewTable:
        """A view table by name: the home's own, or a registered scope table."""
        own = schema.BY_NAME.get(name)
        if own is not None:
            return own
        return next(table for table in self.hosted() if table.name == name)

    def bind_scopes(self, view: SeatView, stack: MemoryStack) -> ScopeSync:
        """Binds the seat's stack (§5.4), in one transaction: the rows of a scope the stack no longer names (or
        names with other parameters) are dropped, a scope it newly names is created and rebuilt from the held
        facts, and `hold` calls each bound optional scope's `index` from then on. With no optional scope
        selected or stored, nothing is written."""
        optional = tuple(bound for bound in stack.scopes if not bound.base)
        db = self.connection
        if db.in_transaction:
            synced = scope_tables.sync(db, view.seat_id, view.identity.seat, optional)
        else:
            with self.transaction():
                synced = scope_tables.sync(db, view.seat_id, view.identity.seat, optional)
        view.scopes = optional
        if synced.changed():
            self._hosted = None
            if self._triggers:
                for table in self.hosted():
                    _table_triggers(db, table)
            self._meters.pop(view.seat_id, None)
        return synced

    def seats(self) -> tuple[EntityKey, ...]:
        rows: list[tuple[str]] = self.connection.execute(
            "SELECT key FROM seats ORDER BY key"
        ).fetchall()
        return tuple(EntityKey(key) for (key,) in rows)

    def dump(self) -> bytes:
        """The whole home in canonical lines (§5.4): `home_meta`'s `schema` and `world_instance` (never the
        creation marker, `ARC-152`); `fact`; `fact_who`, one line per record with its rows' `role, entity` pairs
        in ordinal order; then, per seat in key order, `["seat", key, self_entity]` and the seat's rows in the
        seat view's table order, `access` where the view has `l0`. `seats.seat` never appears. A function of
        each seat's perceived prefix and configuration (I-S28-5)."""
        return ("\n".join(self._dump_parts().lines()) + "\n").encode("utf-8")

    def totals(self) -> HomeTotals:
        """The bytes of each segment of `dump()` and of the file, all read through SQL (pr-s28e §5.5)."""
        parts = self._dump_parts()
        db = self.connection
        page_size: int = db.execute("PRAGMA page_size").fetchone()[0]
        pages: int = db.execute("PRAGMA page_count").fetchone()[0]
        free: int = db.execute("PRAGMA freelist_count").fetchone()[0]
        return HomeTotals(
            home_meta=_bytes(parts.home_meta),
            fact=_bytes(parts.fact),
            fact_who=_bytes(parts.fact_who),
            seats=tuple((key, _bytes(section)) for key, section in parts.seats),
            dump=_bytes(parts.lines()),
            file=pages * page_size,
            free=free * page_size,
        )

    def seat_section(self, key: EntityKey) -> list[str]:
        """One seat's section of `dump()` after its `["seat", …]` line (the seat export's tail, pr-s28e §5.7)."""
        view = self.view_of(key)
        return _seat_lines(self.connection, view.seat_id, self.hosted())

    def _dump_parts(self) -> _DumpParts:
        db = self.connection
        home_meta = [
            line("home_meta", row, (raw, raw))
            for row in db.execute(
                "SELECT key, value FROM home_meta WHERE key IN (?, ?) ORDER BY key",
                schema.DUMPED_HOME_META,
            )
        ]
        fact = [
            line("fact", row, _FACT_CODECS)
            for row in db.execute(
                "SELECT event_id, at, kind, place, digest FROM fact ORDER BY event_key"
            )
        ]
        sections: list[tuple[EntityKey, list[str]]] = []
        seats: list[tuple[int, str, str]] = db.execute(
            "SELECT seat, key, self_entity FROM seats ORDER BY key"
        ).fetchall()
        for seat, key, self_entity in seats:
            header = line("seat", (key, self_entity), (raw, raw))
            sections.append((EntityKey(key), [header, *_seat_lines(db, seat, self.hosted())]))
        return _DumpParts(home_meta, fact, _fact_who_lines(db), sections)

    def reclaim(self) -> None:
        """Returns the pages freed by deleted rows to the file system, outside any transaction."""
        self.writable()
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
        _table_triggers(db, table)


def _table_triggers(db: sqlite3.Connection, table: schema.ViewTable) -> None:
    """One table's three meter triggers; also every registered scope table's (pr-s28d §5.4)."""
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


def _seat_lines(
    db: sqlite3.Connection, seat: int, hosted: tuple[schema.ViewTable, ...]
) -> list[str]:
    """One seat's section of the home dump, after its header: its own tables, then its scope tables' rows."""
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
    for table in (*schema.PRIVATE, *hosted):
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


def _read_only_refusal(error: sqlite3.Error, where: str) -> Exception:
    """The named refusal for SQLite's error on a read-only home's first read, or the error itself: a held file
    is `SQLITE_BUSY` / `SQLITE_LOCKED`, a hot journal `SQLITE_READONLY_ROLLBACK` (or another read-only error
    whose cause is the journal, as SQLite reports it on some platforms). Any other error propagates unchanged."""
    code = getattr(error, "sqlite_errorcode", None)
    if code is not None:
        if code & 0xFF in (sqlite3.SQLITE_BUSY, sqlite3.SQLITE_LOCKED):
            return HomeBusy(where)
        if code & 0xFF == sqlite3.SQLITE_READONLY:
            return HomeNeedsRecovery(where)
    return error


def _check(connection: sqlite3.Connection, where: str, world_instance: WorldInstanceId) -> None:
    stored = dict(connection.execute("SELECT key, value FROM home_meta").fetchall())
    if stored.get("schema") != str(schema.SCHEMA_VERSION):
        raise UnsupportedStoreSchema(where, str(stored.get("schema")), schema.SCHEMA_VERSION)
    if stored.get("world_instance") != world_instance:
        raise HomeIdentityMismatch(
            "world_instance", str(stored.get("world_instance")), world_instance
        )


def _create(connection: sqlite3.Connection, world_instance: WorldInstanceId) -> str:
    """Schema 4 and its identity rows, the creation marker among them, in one transaction. Returns the marker."""
    created = secrets.token_hex(16)
    connection.execute("BEGIN IMMEDIATE")
    try:
        for statement in schema.HOME_DDL:
            connection.execute(statement)
        if fts.available():
            connection.execute(schema.FTS_DDL)
        connection.executemany(
            "INSERT INTO home_meta (key, value) VALUES (?, ?)",
            [
                ("schema", str(schema.SCHEMA_VERSION)),
                ("world_instance", world_instance),
                (schema.CREATED_KEY, created),
            ],
        )
    except BaseException:
        connection.execute("ROLLBACK")
        raise
    _commit(connection)
    return created


def _marked(connection: sqlite3.Connection) -> str:
    """The creation marker of a home that passed `_check`. A schema-4 home made before the marker existed
    (`ARC-152`) receives it here, once; no other row is touched, and the row is never written again."""
    row: tuple[str] | None = connection.execute(
        "SELECT value FROM home_meta WHERE key = ?", (schema.CREATED_KEY,)
    ).fetchone()
    if row is not None:
        return row[0]
    created = secrets.token_hex(16)
    connection.execute("BEGIN IMMEDIATE")
    try:
        connection.execute(
            "INSERT INTO home_meta (key, value) VALUES (?, ?)", (schema.CREATED_KEY, created)
        )
    except BaseException:
        connection.execute("ROLLBACK")
        raise
    _commit(connection)
    return created
