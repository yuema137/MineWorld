"""The anchored reads of a `ScopeView` (pr-s28d §5.5), written once over `(connection, seat)`.

Every statement binds the seat first and reads `fact` only through the seat's `access` join, so a scope sees the
held facts of its own seat and nothing else (I-d-2, I-d-5). `own_rows` reads only the calling scope's own tables,
by their key columns.
"""

from __future__ import annotations

import json
import sqlite3

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.anchors import AnchorKind
from mineworld_cognition.memory.records import event_of
from mineworld_cognition.memory.scopes.interface import ScopeRow


def anchored(
    db: sqlite3.Connection,
    seat: int,
    entity: EntityId,
    *,
    at: bool,
    named: frozenset[AnchorKind],
    until: int,
    limit: int,
    notable: bool,
) -> tuple[EventId, ...]:
    """The seat's held facts anchored to `entity` with `at <= until`, newest key first, at most `limit`."""
    salience = "AND a.salience = 'notable' " if notable else ""
    rows: list[tuple[str]] = db.execute(
        "SELECT a.event_key FROM access a JOIN fact f ON f.event_key = a.event_key "
        "WHERE a.seat = ? AND f.at <= ? AND ((? AND f.place = ?) OR EXISTS ("
        "SELECT 1 FROM access_anchor x WHERE x.seat = a.seat AND x.event_key = a.event_key "
        "AND x.entity = ? AND x.kind IN (SELECT value FROM json_each(?)))) "
        f"{salience}ORDER BY a.event_key DESC LIMIT ?",
        (seat, until, 1 if at else 0, entity, entity, json.dumps(sorted(named)), max(0, limit)),
    ).fetchall()
    return tuple(event_of(key) for (key,) in rows)


def key_columns(db: sqlite3.Connection, scope: str, table: str) -> tuple[str, ...] | None:
    """The key-role columns of `table`, when it is one of `scope`'s registered tables; `None` otherwise."""
    owner: tuple[str] | None = db.execute(
        "SELECT scope FROM scope_tables WHERE name = ?", (table,)
    ).fetchone()
    if owner is None or owner[0] != scope:
        return None
    info: list[tuple[int, str, str, int, object, int]] = db.execute(
        f"PRAGMA table_info({table})"
    ).fetchall()
    return tuple(
        name for _, name, _, _, _, pk in sorted(info, key=lambda c: c[5]) if pk and name != "seat"
    )


def own_rows(
    db: sqlite3.Connection,
    seat: int,
    table: str,
    keys: tuple[str, ...],
    where: tuple[tuple[str, str | int], ...],
) -> tuple[ScopeRow, ...]:
    """The seat's rows of `table` (whose key columns are `keys`, already checked) matching `where`."""
    info: list[tuple[int, str, str, int, object, int]] = db.execute(
        f"PRAGMA table_info({table})"
    ).fetchall()
    columns = [name for _, name, _, _, _, _ in info if name != "seat"]
    test = "".join(f" AND {column} = ?" for column, _ in where)
    rows: list[tuple[str | int, ...]] = db.execute(
        f"SELECT {', '.join(columns)} FROM {table} WHERE seat = ?{test} ORDER BY {', '.join(keys)}",
        (seat, *(value for _, value in where)),
    ).fetchall()
    return tuple(ScopeRow(table=table, cells=tuple(row)) for row in rows)
