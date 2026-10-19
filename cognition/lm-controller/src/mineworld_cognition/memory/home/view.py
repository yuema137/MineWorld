"""`SeatView`: the projection of a memory home onto one seat, and memory's one storage API (pr-s28a-memory-home.md
§§5.3, 5.6, 5.7; step-28 §4.4; `ARC-109`).

A seat view exposes what P4's per-seat store exposed — identity, cursor, meta, a transaction, the canonical dump,
its sizes — over the home's tables: the seat's private rows (`seat = view.seat_id`) and the fact records it holds
(`access ⋈ fact`). Its dump is the standalone seat view: byte-identical whether the seat shares its home with no one
or with eleven others (I-S28-2), because nothing in it depends on another seat's rows, on rowids, or on the internal
seat number.

**Hold and release** are the only writers of `access` (and of `access_text`, its lexical index), and the only
callers of the `world` scope. `hold` inserts or verifies the record, then the access row, inside the caller's
transaction; `release` deletes the access rows, then collects the records no seat holds any more, in the same
transaction (I-S28-4).

**The meter** keeps `content_size()` — the dump's bytes less `meta`'s lines (FG-b2's definition) — exact after
every write without building the dump: the private tables through `TEMP` triggers that hand each written or removed
row to the home (FG-b2's mechanism, now with the row's seat), and the `l0`/`l0_who` lines through `hold` and
`release`, which compute them with the dump's own encoder. A rollback drops every seat's meter; the next call
measures in full.
"""

from __future__ import annotations

import sqlite3
from collections.abc import Generator, Iterable
from contextlib import contextmanager
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId, EventTypeId

from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.home.encoding import Cell, line, line_bytes, raw
from mineworld_cognition.memory.home.errors import FactRecordMissing
from mineworld_cognition.memory.records import (
    EventKey,
    L0Record,
    Salience,
    StoreIdentity,
    event_key,
)
from mineworld_cognition.memory.scopes import world

if TYPE_CHECKING:
    from mineworld_cognition.memory.home.home import MemoryHome
    from mineworld_cognition.memory.render import Rendered

type Row = dict[str, Cell]

METER_PUT = "mineworld_meter_put"
METER_GONE = "mineworld_meter_gone"
"""The connection functions the meter's `TEMP` triggers call."""
METERED: tuple[schema.ViewTable, ...] = (schema.NAMES, *schema.PRIVATE)
"""The private tables `content_size()` measures by trigger: every dumped seat table but `meta`."""
_L0_SELECT = (
    "SELECT a.event_key, f.event_id, f.at, f.kind, f.place, a.salience, a.mine, a.gist, f.digest, "
    "a.importance FROM access a JOIN fact f ON f.event_key = a.event_key WHERE a.seat = ?"
)
"""A seat's `l0` rows: its access rows joined with the records they hold."""
_WHO_SELECT = (
    "SELECT a.event_key, w.entity FROM access a JOIN fact_who w ON w.event_key = a.event_key "
    "WHERE a.seat = ? AND w.role IN (1, 3) AND w.entity <> ?"
)
"""A seat's `l0_who` rows: the held records' participants other than the seat."""


@dataclass
class SeatMeter:
    """One seat's measured content: each private row's line bytes, each held key's `l0` line and `l0_who`
    lines, and their total."""

    rows: dict[tuple[str, int], int] = field(default_factory=dict[tuple[str, int], int])
    held: dict[str, tuple[int, tuple[int, ...]]] = field(
        default_factory=dict[str, tuple[int, tuple[int, ...]]]
    )
    content: int = 0

    def put(self, table: str, rid: int, size: int) -> None:
        self.content += size - self.rows.get((table, rid), 0)
        self.rows[(table, rid)] = size

    def gone(self, table: str, rid: int) -> None:
        self.content -= self.rows.pop((table, rid), 0)

    def hold(self, key: str, l0: int, who: tuple[int, ...]) -> None:
        self.held[key] = (l0, who)
        self.content += l0 + sum(who)

    def release(self, key: str) -> None:
        sizes = self.held.pop(key, None)
        if sizes is not None:
            self.content -= sizes[0] + sum(sizes[1])


class SeatView:
    """One seat of a home. Obtained from `MemoryHome.seat(identity)`; never constructed by a consumer."""

    def __init__(self, home: MemoryHome, seat_id: int, identity: StoreIdentity) -> None:
        self.home = home
        self.seat_id = seat_id
        """The home's internal number for this seat: bound first in every `seat = ?` predicate. Never dumped."""
        self.identity = identity

    @property
    def connection(self) -> sqlite3.Connection:
        return self.home.connection

    @property
    def me(self) -> EntityId:
        return self.identity.self_entity

    @property
    def fts(self) -> bool:
        return self.home.fts

    @contextmanager
    def transaction(self, *, retention_pass: bool = False) -> Generator[sqlite3.Connection]:
        """The home's transaction: one `BEGIN IMMEDIATE … COMMIT` covering every seat's writes (step-28 §6.3)."""
        with self.home.transaction(retention_pass=retention_pass) as connection:
            yield connection

    # ── Meta and cursor ───────────────────────────────────────────────────────────────────────────

    def meta(self, key: str) -> str:
        identity = self._identity_rows()
        if key in identity:
            return identity[key]
        row = self.connection.execute(
            "SELECT value FROM meta WHERE seat = ? AND key = ?", (self.seat_id, key)
        ).fetchone()
        if row is None:
            raise KeyError(key)
        value: str = row[0]
        return value

    def set_meta(self, key: str, value: str) -> None:
        self.connection.execute(
            "INSERT INTO meta (seat, key, value) VALUES (?, ?, ?) "
            "ON CONFLICT (seat, key) DO UPDATE SET value = excluded.value",
            (self.seat_id, key, value),
        )

    def cursor(self) -> EventId | None:
        """The perceived cursor: every fact at or below it is durably ingested (P4-2)."""
        value = self.meta("cursor")
        return EventId(value) if value else None

    def newest(self) -> EventKey | None:
        """The key of the newest ingested fact."""
        value = self.meta("newest")
        return EventKey(value) if value else None

    def _identity_rows(self) -> dict[str, str]:
        return {
            "world_instance": self.home.world_instance,
            "schema": str(schema.SCHEMA_VERSION),
            "seat": self.identity.seat,
            "self_entity": self.identity.self_entity,
        }

    # ── Holding facts ─────────────────────────────────────────────────────────────────────────────

    def holds(self, key: EventKey) -> bool:
        row = self.connection.execute(
            "SELECT 1 FROM access WHERE seat = ? AND event_key = ?", (self.seat_id, key)
        ).fetchone()
        return row is not None

    def held_digest(self, key: EventKey) -> str | None:
        """The full digest of a record this seat holds, or `None` when it holds none under `key`."""
        if not self.holds(key):
            return None
        row = self.connection.execute(
            "SELECT f.digest FROM access a JOIN fact f ON f.event_key = a.event_key "
            "WHERE a.seat = ? AND a.event_key = ?",
            (self.seat_id, key),
        ).fetchone()
        if row is None:
            raise FactRecordMissing(key)
        digest: str = row[0]
        return digest

    def hold(self, fact: PerceivedEvent, digest: str, rendered: Rendered) -> L0Record:
        """Holds a perceived fact with this seat's rendering, inside the caller's transaction: the record
        inserted or verified (`ConflictingFact` if a held record differs), then the access row, then (notable)
        its lexical index entry, then the meter. The record always comes first."""
        db = self.connection
        key = event_key(fact.id)
        world.insert_or_verify(db, fact, digest)
        mine = int(rendered.mine)
        row = db.execute(
            "INSERT INTO access (seat, event_key, gist, salience, mine, importance) "
            "VALUES (?, ?, ?, ?, ?, ?)",
            (self.seat_id, key, rendered.gist, rendered.salience, mine, rendered.importance),
        )
        if self.fts and rendered.salience == "notable":
            db.execute(
                "INSERT INTO access_text (rowid, gist, seat) VALUES (?, ?, ?)",
                (row.lastrowid, rendered.gist, self.seat_id),
            )
        place = None if fact.place is None else fact.place.entity
        counterparts = tuple(
            entity
            for entity, role in world.who(fact)
            if role != world.ROLE_SUBJECT and entity != self.me
        )
        meter = self.home.meter(self.seat_id)
        if meter is not None:
            cells = (fact.id, fact.at, fact.event_type, place, rendered.salience, mine)
            l0 = line_bytes(
                "l0", (*cells, rendered.gist, digest, rendered.importance), schema.L0.codecs
            )
            # `["l0_who",<id>,"<entity>"]` and its newline: an `EventId` and an `EntityId` are ASCII decimal
            # digits with no leading zero (the SDK validates both), so the id's number is its digits and the
            # entity's JSON is its digits in quotes; the line's length is exact without encoding it once per
            # counterpart (a busy fact names hundreds). `test_the_meter_is_the_dump` holds it to the dump.
            # The precondition, stated: the shortcut is exact only for decimal ids (D-a-14).
            assert _decimal(fact.id) and all(_decimal(entity) for entity in counterparts), fact.id
            fixed = len('["l0_who",') + len(fact.id) + len(',""]') + 1
            who = tuple(fixed + len(entity) for entity in counterparts)
            meter.hold(key, l0, who)
        return L0Record.model_construct(
            event=fact.id,
            at=fact.at,
            kind=fact.event_type,
            place=place,
            salience=rendered.salience,
            mine=rendered.mine,
            gist=rendered.gist,
            counterparts=counterparts,
        )

    def release(self, keys: Iterable[str]) -> None:
        """Releases held keys inside the caller's transaction: their index entries and access rows, then the
        records no seat of the home holds any more (I-S28-4). Keys this seat does not hold are ignored."""
        db = self.connection
        released = list(keys)
        rows = [(self.seat_id, key) for key in released]
        if self.fts:
            db.executemany(
                "DELETE FROM access_text WHERE rowid = "
                "(SELECT rowid FROM access WHERE seat = ? AND event_key = ?)",
                rows,
            )
        db.executemany("DELETE FROM access WHERE seat = ? AND event_key = ?", rows)
        meter = self.home.meter(self.seat_id)
        if meter is not None:
            for key in released:
                meter.release(key)
        world.collect(db, [EventKey(key) for key in released])

    # ── The canonical dump and its sizes ──────────────────────────────────────────────────────────

    def dump(self) -> bytes:
        """The seat view in canonical lines (§5.4): `meta`, `names`, `l0`, `l0_who`, then the private tables,
        each in key order. UTF-8, no float, no wall-clock value, no rowid, no internal seat number."""
        lines = [text for table in schema.VIEW_TABLES for _, text in self._lines(table.name)]
        return ("\n".join(lines) + "\n").encode("utf-8")

    def size(self) -> int:
        """`len(dump())`, without building it once the meter is in use: its total and `meta`'s lines."""
        if self.home.meter(self.seat_id) is not None:
            return self.content_size() + sum(size for _, size in self.row_bytes("meta"))
        return len(self.dump())

    def content_size(self) -> int:
        """`size()` less `meta`'s lines, kept incrementally (pr-fg-b2 §4.2): the first call, and the first after
        a rollback, measure in full."""
        meter = self.home.meter(self.seat_id)
        if meter is None:
            meter = self.home.measure(self)
        return meter.content

    def row_bytes(self, table: str) -> list[tuple[Row, int]]:
        """Every row of a view table, in dump order, with the bytes its line takes, newline included: what
        removing the row subtracts from `dump()` (pr-fg-b §5.4). The cells carry schema 3's column names,
        `event_key` included for `l0` and `l0_who`. Once the meter is in use, its sizes are read, not encoded
        again."""
        meter = self.home.meter(self.seat_id)
        if meter is None or table == "meta":
            return [(cells, len(text.encode("utf-8")) + 1) for cells, text in self._lines(table)]
        if table == "l0":
            return [(cells, meter.held[text(cells, "event_key")][0]) for cells in self._l0_rows()]
        if table == "l0_who":
            out: list[tuple[Row, int]] = []
            index: dict[str, int] = {}
            for cells in self._who_rows():
                key = text(cells, "event_key")
                position = index.get(key, 0)
                out.append((cells, meter.held[key][1][position]))
                index[key] = position + 1
            return out
        view = schema.BY_NAME[table]
        return [(cells, meter.rows[(table, rid)]) for rid, cells in self.private_rows(view)]

    def reclaim(self) -> None:
        self.home.reclaim()

    # ── Rows ──────────────────────────────────────────────────────────────────────────────────────

    def _lines(self, table: str) -> list[tuple[Row, str]]:
        if table == "meta":
            return [
                (
                    {"key": key, "value": value},
                    line("meta", (key, value), (raw, schema.meta_codec(key))),
                )
                for key, value in self._meta_rows()
            ]
        view = schema.BY_NAME[table]
        if table == "l0":
            rows = self._l0_rows()
        elif table == "l0_who":
            rows = [{"event_id": cells["event_key"], **cells} for cells in self._who_rows()]
        else:
            rows = [cells for _, cells in self.private_rows(view)]
        codecs = view.codecs
        return [
            (cells, line(table, [cells[column] for column in view.columns], codecs))
            for cells in rows
        ]

    def _meta_rows(self) -> list[tuple[str, str]]:
        own: list[tuple[str, str]] = self.connection.execute(
            "SELECT key, value FROM meta WHERE seat = ?", (self.seat_id,)
        ).fetchall()
        return sorted([*self._identity_rows().items(), *own])

    def _l0_rows(self) -> list[Row]:
        cursor = self.connection.execute(_L0_SELECT + " ORDER BY a.event_key", (self.seat_id,))
        columns = [column[0] for column in cursor.description]
        return [dict(zip(columns, values, strict=True)) for values in cursor]

    def _who_rows(self) -> list[Row]:
        cursor = self.connection.execute(
            _WHO_SELECT + " ORDER BY a.event_key, w.ordinal", (self.seat_id, self.me)
        )
        return [{"event_key": key, "entity": entity} for key, entity in cursor]

    def private_rows(self, table: schema.ViewTable) -> list[tuple[int, Row]]:
        """A private table's rows of this seat, in key order, with their rowids (for the meter)."""
        cursor = self.connection.execute(
            f"SELECT rowid, {', '.join(table.columns)} FROM {table.name} WHERE seat = ? "
            f"ORDER BY {table.order}",
            (self.seat_id,),
        )
        return [(rid, dict(zip(table.columns, values, strict=True))) for rid, *values in cursor]

    def measured(self) -> SeatMeter:
        """This seat's content, measured in full from its rows."""
        meter = SeatMeter()
        for table in METERED:
            codecs = table.codecs
            for rid, cells in self.private_rows(table):
                values = [cells[column] for column in table.columns]
                meter.put(table.name, rid, line_bytes(table.name, values, codecs))
        who: dict[str, list[int]] = {}
        for cells in self._who_rows():
            key = text(cells, "event_key")
            size = line_bytes("l0_who", (key, cells["entity"]), schema.L0_WHO.codecs)
            who.setdefault(key, []).append(size)
        for cells in self._l0_rows():
            key = text(cells, "event_key")
            values = [cells[column] for column in schema.L0.columns]
            meter.hold(key, line_bytes("l0", values, schema.L0.codecs), tuple(who.get(key, ())))
        return meter

    def __repr__(self) -> str:
        return f"SeatView(seat={self.identity.seat!r})"


def _decimal(value: str) -> bool:
    """ASCII decimal digits with no leading zero: what the SDK's `EventId` and `EntityId` validate, and what the
    meter's `l0_who` shortcut in `SeatView.hold` relies on (its JSON is its digits, quoted or not)."""
    return value.isascii() and value.isdigit() and (value == "0" or value[0] != "0")


def text(row: Row, name: str) -> str:
    value = row[name]
    assert isinstance(value, str), name
    return value


def records_between(view: SeatView, first: EventKey, last: EventKey) -> list[L0Record]:
    """The seat's L0 records from `first` to `last`, both included, in perceived order, with their
    counterparts in envelope order."""
    db = view.connection
    rows: list[tuple[str, str, int, str, str | None, Salience, int, str]] = db.execute(
        "SELECT a.event_key, f.event_id, f.at, f.kind, f.place, a.salience, a.mine, a.gist "
        "FROM access a JOIN fact f ON f.event_key = a.event_key "
        "WHERE a.seat = ? AND a.event_key BETWEEN ? AND ? ORDER BY a.event_key",
        (view.seat_id, first, last),
    ).fetchall()
    who: dict[str, list[EntityId]] = {}
    for key, entity in db.execute(
        _WHO_SELECT + " AND a.event_key BETWEEN ? AND ? ORDER BY a.event_key, w.ordinal",
        (view.seat_id, view.me, first, last),
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
