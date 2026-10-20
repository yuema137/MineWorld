"""The scope-table host: the per-seat tables an optional memory scope declares, written by the home
(pr-s28d-anchored-scope.md §5.4; QS28c-4, QS28d-3, QS28d-4; `ARC-100`).

A scope never writes. It declares `TableSpec`s and returns `ScopeRow`s from `index`; the home here

- creates each table from its declaration on first use and registers it in `scope_tables` (a later, different
  declaration is `ScopeTableMismatch`);
- checks every returned row (its table is the scope's own, its cells match the columns, every `key` cell is the
  held fact's key; else `ScopeRowRefused`, rolling the ingest back) and writes it as one SQLite upsert built from
  the columns' merge roles;
- applies the table's cap in its `evict` order;
- keeps each seat's bound scopes and their parameters in `scope_seats`, dropping a scope's rows when a stack stops
  naming it and rebuilding them from the held facts when it names it again (`sync`).

Every statement binds `seat = ?`. Every SQL name comes from a load-checked declaration (`registry.SQL_NAME`);
every value is a parameter (R-d-3).
"""

from __future__ import annotations

import json
import sqlite3
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal, cast

from mineworld_sdk.wire.contract import PerceivedEvent

from mineworld_cognition.memory.anchors import Anchor, AnchorKind
from mineworld_cognition.memory.home.errors import ScopeRowRefused, ScopeTableMismatch
from mineworld_cognition.memory.home.schema import ViewTable
from mineworld_cognition.memory.records import EventKey, Salience, event_key
from mineworld_cognition.memory.scopes import world
from mineworld_cognition.memory.scopes.interface import (
    FactRecord,
    Reading,
    ScopeParameters,
    ScopeRow,
    TableSpec,
)

if TYPE_CHECKING:
    from mineworld_cognition.memory.render import Rendered
    from mineworld_cognition.memory.scopes.registry import BoundScope

KEY_TYPE = "KEYTEXT"
"""The declared SQL type of a `key` column: TEXT affinity, and read back by `hosted` to write the column as a
number in a dump, as the home's own key columns are."""
_SQL_TYPES = {"text": "TEXT", "key": KEY_TYPE, "int": "INTEGER"}


@dataclass(frozen=True)
class ScopeSync:
    """What `MemoryHome.bind_scopes` changed for one seat: the scopes added (and rebuilt) and removed, and the
    rows the rebuild wrote."""

    seat: str
    added: tuple[str, ...]
    removed: tuple[str, ...]
    rebuilt_rows: int

    def changed(self) -> bool:
        return bool(self.added or self.removed)


# ── Declarations and the registry ─────────────────────────────────────────────────────────────────


def ddl(spec: TableSpec) -> str:
    """The table's `CREATE TABLE`: `seat`, then the declared columns, every one `NOT NULL`; the primary key is
    `seat` and the `key`-role columns, in declared order."""
    columns = ", ".join(f"{c.name} {_SQL_TYPES[c.type]} NOT NULL" for c in spec.columns)
    keys = ", ".join(c.name for c in spec.columns if c.merge == "key")
    return (
        f"CREATE TABLE {spec.name} (seat INTEGER NOT NULL, {columns}, PRIMARY KEY (seat, {keys}))"
    )


def register(db: sqlite3.Connection, scope: str, spec: TableSpec) -> bool:
    """Creates and registers the table on first use; `True` when it was created now. A registered table whose
    declaration differs, or that another scope registered, is `ScopeTableMismatch`."""
    wanted = ddl(spec)
    row: tuple[str, str] | None = db.execute(
        "SELECT scope, ddl FROM scope_tables WHERE name = ?", (spec.name,)
    ).fetchone()
    if row is not None:
        if row != (scope, wanted):
            raise ScopeTableMismatch(scope, spec.name)
        return False
    db.execute(wanted)
    db.execute(
        "INSERT INTO scope_tables (name, scope, ddl) VALUES (?, ?, ?)", (spec.name, scope, wanted)
    )
    return True


def hosted(db: sqlite3.Connection) -> tuple[ViewTable, ...]:
    """Every registered scope table as a seat view table, in table-name order: its columns after `seat`, its
    key order, and its `key`-typed columns written as numbers."""
    names: list[tuple[str]] = db.execute("SELECT name FROM scope_tables ORDER BY name").fetchall()
    tables: list[ViewTable] = []
    for (name,) in names:
        info: list[tuple[int, str, str, int, object, int]] = db.execute(
            f"PRAGMA table_info({name})"
        ).fetchall()
        columns = [(column, kind, pk) for _, column, kind, _, _, pk in info if column != "seat"]
        order = ", ".join(column for column, _, pk in sorted(columns, key=lambda c: c[2]) if pk)
        tables.append(
            ViewTable(
                name,
                tuple(column for column, _, _ in columns),
                order,
                numbers=frozenset(column for column, kind, _ in columns if kind == KEY_TYPE),
            )
        )
    return tuple(tables)


def tables_of(db: sqlite3.Connection, scope: str) -> tuple[str, ...]:
    rows: list[tuple[str]] = db.execute(
        "SELECT name FROM scope_tables WHERE scope = ? ORDER BY name", (scope,)
    ).fetchall()
    return tuple(name for (name,) in rows)


# ── Writing rows ──────────────────────────────────────────────────────────────────────────────────


def _upsert(spec: TableSpec) -> str:
    names = [c.name for c in spec.columns]
    keys = [c.name for c in spec.columns if c.merge == "key"]
    sets = [
        f"{c.name} = excluded.{c.name}"
        if c.merge == "last"
        else f"{c.name} = {spec.name}.{c.name} + excluded.{c.name}"
        for c in spec.columns
        if c.merge in ("last", "count")
    ]
    head = (
        f"INSERT INTO {spec.name} (seat, {', '.join(names)}) "
        f"VALUES (?, {', '.join('?' for _ in names)}) ON CONFLICT (seat, {', '.join(keys)}) DO "
    )
    return head + (f"UPDATE SET {', '.join(sets)}" if sets else "NOTHING")


def _refusal(spec: TableSpec, cells: Sequence[object], key: EventKey) -> str | None:
    if len(cells) != len(spec.columns):
        return f"has {len(cells)} cells for {len(spec.columns)} columns"
    for column, cell in zip(spec.columns, cells, strict=True):
        if column.type == "int":
            if not isinstance(cell, int) or isinstance(cell, bool):
                return f"holds a non-integer in int column {column.name!r}"
        elif not isinstance(cell, str):
            return f"holds a non-text value in column {column.name!r}"
        elif column.type == "key" and cell != key:
            return f"cites in key column {column.name!r} a fact other than the one being held"
    return None


def write(
    db: sqlite3.Connection,
    seat: int,
    bound: BoundScope,
    key: EventKey,
    rows: object,
) -> int:
    """Checks and writes the rows a scope's `index` returned for the held fact `key`, then applies each
    table's cap. Returns the number of rows written."""
    specs = {spec.name: spec for spec in bound.scope.tables}
    if not isinstance(rows, tuple):
        raise ScopeRowRefused(bound.id, "?", "is not a tuple of ScopeRow")
    written = 0
    touched: dict[str, TableSpec] = {}
    for row in cast(tuple[object, ...], rows):
        if not isinstance(row, ScopeRow):
            raise ScopeRowRefused(bound.id, "?", "is not a ScopeRow")
        spec = specs.get(row.table)
        if spec is None:
            raise ScopeRowRefused(bound.id, row.table, "is not one of the scope's own tables")
        why = _refusal(spec, row.cells, key)
        if why is not None:
            raise ScopeRowRefused(bound.id, row.table, why)
        db.execute(_upsert(spec), (seat, *row.cells))
        touched[spec.name] = spec
        written += 1
    for spec in touched.values():
        _cap(db, seat, spec, bound.parameters)
    return written


def _cap(db: sqlite3.Connection, seat: int, spec: TableSpec, params: ScopeParameters) -> None:
    if spec.cap is None:
        return
    limit = cast(int, getattr(params, spec.cap))
    count: int = db.execute(f"SELECT count(*) FROM {spec.name} WHERE seat = ?", (seat,)).fetchone()[
        0
    ]
    if count > limit:
        db.execute(
            f"DELETE FROM {spec.name} WHERE seat = ? AND rowid IN (SELECT rowid FROM {spec.name} "
            f"WHERE seat = ? ORDER BY {', '.join(spec.evict)} LIMIT ?)",
            (seat, seat, count - limit),
        )


def index_held(
    db: sqlite3.Connection,
    seat: int,
    scopes: Iterable[BoundScope],
    fact: PerceivedEvent,
    rendered: Rendered,
) -> None:
    """`hold`'s call: each bound optional scope's `index` on the fact just held, in stack order."""
    record = FactRecord.model_validate(
        {
            "event": fact.id,
            "at": fact.at,
            "kind": fact.event_type,
            "place": None if fact.place is None else fact.place.entity,
            "who": tuple(world.who(fact)),
        }
    )
    reading = Reading(
        gist=rendered.gist,
        salience=rendered.salience,
        mine=rendered.mine,
        importance=rendered.importance,
        anchors=rendered.anchors,
    )
    key = event_key(fact.id)
    for bound in scopes:
        write(db, seat, bound, key, _index(bound, record, reading))


def _index(bound: BoundScope, record: FactRecord, reading: Reading) -> object:
    """A scope's rows, not trusted to be what its signature says (`write` checks them)."""
    return bound.scope.index(record, reading, bound.parameters)


# ── Binding a seat's stack ────────────────────────────────────────────────────────────────────────


def _canonical(params: ScopeParameters) -> str:
    return json.dumps(params.model_dump(mode="json"), sort_keys=True, separators=(",", ":"))


def sync(db: sqlite3.Connection, seat: int, key: str, optional: Sequence[BoundScope]) -> ScopeSync:
    """Steps 1 to 4 and 6 of `bind_scopes` (§5.4), inside the caller's transaction: drop the rows of every stored
    scope not selected (or selected with other parameters), then create and rebuild every selected scope not
    stored."""
    selected = {bound.id: (bound, _canonical(bound.parameters)) for bound in optional}
    stored: dict[str, str] = dict(
        db.execute("SELECT scope, parameters FROM scope_seats WHERE seat = ?", (seat,)).fetchall()
    )
    removed = [
        scope
        for scope, params in sorted(stored.items())
        if scope not in selected or selected[scope][1] != params
    ]
    for scope in removed:
        drop(db, seat, scope)
        del stored[scope]
    added: list[str] = []
    rebuilt = 0
    for scope, (bound, params) in selected.items():
        if scope in stored:
            continue
        for spec in bound.scope.tables:
            register(db, scope, spec)
        rebuilt += rebuild(db, seat, bound)
        db.execute(
            "INSERT INTO scope_seats (seat, scope, parameters) VALUES (?, ?, ?)",
            (seat, scope, params),
        )
        added.append(scope)
    return ScopeSync(seat=key, added=tuple(added), removed=tuple(removed), rebuilt_rows=rebuilt)


def drop(db: sqlite3.Connection, seat: int, scope: str) -> None:
    """A scope's rows of one seat, and its `scope_seats` row, deleted."""
    for table in tables_of(db, scope):
        db.execute(f"DELETE FROM {table} WHERE seat = ?", (seat,))
    db.execute("DELETE FROM scope_seats WHERE seat = ? AND scope = ?", (seat, scope))


def drop_seat(db: sqlite3.Connection, seat: int) -> None:
    """Every scope row of a seat leaving the home."""
    for (table,) in db.execute("SELECT name FROM scope_tables ORDER BY name").fetchall():
        db.execute(f"DELETE FROM {table} WHERE seat = ?", (seat,))
    db.execute("DELETE FROM scope_seats WHERE seat = ?", (seat,))


def rebuild(db: sqlite3.Connection, seat: int, bound: BoundScope) -> int:
    """The scope's rows rebuilt from every fact the seat holds, in key order, exactly as `hold` writes them."""
    who: dict[str, list[tuple[str, int]]] = {}
    for key, entity, role in db.execute(
        "SELECT w.event_key, w.entity, w.role FROM access a JOIN fact_who w ON w.event_key = a.event_key "
        "WHERE a.seat = ? ORDER BY w.event_key, w.ordinal",
        (seat,),
    ):
        who.setdefault(key, []).append((entity, role))
    anchors: dict[str, list[Anchor]] = {}
    for key, entity, kind in db.execute(
        "SELECT event_key, entity, kind FROM access_anchor WHERE seat = ? ORDER BY event_key, rowid",
        (seat,),
    ):
        anchors.setdefault(key, []).append(Anchor(entity, cast(AnchorKind, kind)))
    rows: list[tuple[str, str, int, str, str | None, str, Salience, int, int]] = db.execute(
        "SELECT a.event_key, f.event_id, f.at, f.kind, f.place, a.gist, a.salience, a.mine, "
        "a.importance FROM access a JOIN fact f ON f.event_key = a.event_key WHERE a.seat = ? "
        "ORDER BY a.event_key",
        (seat,),
    ).fetchall()
    written = 0
    for key, event, at, kind, place, gist, salience, mine, importance in rows:
        record = FactRecord.model_validate(
            {
                "event": event,
                "at": at,
                "kind": kind,
                "place": place,
                "who": tuple(
                    (entity, cast(Literal[1, 2, 3], role)) for entity, role in who.get(key, ())
                ),
            }
        )
        reading = Reading(
            gist=gist,
            salience=salience,
            mine=mine == 1,
            importance=importance,
            anchors=tuple(anchors.get(key, ())),
        )
        written += write(db, seat, bound, EventKey(key), _index(bound, record, reading))
    return written
