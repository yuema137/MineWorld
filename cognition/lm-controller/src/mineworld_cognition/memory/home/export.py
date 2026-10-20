"""The seat export: one seat of a home as a self-contained, deterministic file (pr-s28e §5.7; QS28e-1 (a); step-28
§6.6, AS28-11).

**Format 1.** UTF-8, one JSON array per line in the home dump's encoding (`encoding.py`), `\\n` after every line:

    ["mineworld-seat-export", 1, "<schema>", "<world_instance>", "<seat key>", "<self entity>"]
    ["fact", <id>, <at>, "<kind>", "<place>"|null, "<digest, all 64 hex>"]     every held record, key order
    ["fact_who", <id>, <role>, "<entity>", <role>, "<entity>", …]               every held record with rows
    <the seat's section of `MemoryHome.dump()`, without its ["seat", …] line>

The tail **is** the home dump's seat section, produced by the same function (`home._seat_lines`), so the export
cannot drift from the canonical form (R-e-2). A seat view dump cannot be imported: it keeps a 128-bit digest and
only the `fact_who` rows the seat sees (F-S28e-1).

`import_seat` parses every line before it writes, refuses by name (`NotASeatExport` naming the line number, never
its content; `UnsupportedExportFormat`; `UnsupportedStoreSchema`; `HomeIdentityMismatch`; `SeatAlreadyInHome`),
then writes in one transaction of the home: each record inserted or verified with its full digest
(`ConflictingFact` when a record the home holds differs), the seat, its access rows with their lexical index
entries, its private rows and its `meta` rows. Any error rolls everything back.
"""

from __future__ import annotations

import json
import re
from collections.abc import Sequence
from dataclasses import dataclass, field
from typing import cast

from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId

from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.home.encoding import Cell, as_number, encoded, line, raw
from mineworld_cognition.memory.home.errors import (
    HomeIdentityMismatch,
    NotASeatExport,
    SeatAlreadyInHome,
    UnsupportedExportFormat,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.home.home import MemoryHome
from mineworld_cognition.memory.records import KEY_WIDTH, StoreIdentity, event_key
from mineworld_cognition.memory.scopes import world

MAGIC = "mineworld-seat-export"
FORMAT = 1
_KEY_META = frozenset({"newest", "forgotten_below"})
"""`meta` values stored as `EventKey`s (padded); `cursor` is stored as an `EventId`."""
assert _KEY_META | {"cursor"} == schema.NUMBER_META, "a numeric meta key's stored form is unknown"


def export_seat(home: MemoryHome, key: EntityKey) -> bytes:
    """The seat export of `key` (`SeatNotInHome` otherwise), read in one read transaction."""
    view = home.view_of(key)
    db = home.connection
    opened = not db.in_transaction
    if opened:
        db.execute("BEGIN")
    try:
        out = [
            encoded(
                MAGIC,
                [FORMAT, str(schema.SCHEMA_VERSION), home.world_instance, key, view.me],
            )
        ]
        records: list[tuple[str, str, int, str, str | None, str]] = db.execute(
            "SELECT f.event_key, f.event_id, f.at, f.kind, f.place, f.digest FROM access a "
            "JOIN fact f ON f.event_key = a.event_key WHERE a.seat = ? ORDER BY f.event_key",
            (view.seat_id,),
        ).fetchall()
        out += [
            line("fact", (event_id, at, kind, place, digest), (as_number, raw, raw, raw, raw))
            for _, event_id, at, kind, place, digest in records
        ]
        who: dict[str, list[Cell]] = {}
        for record_key, entity, role in db.execute(
            "SELECT w.event_key, w.entity, w.role FROM access a JOIN fact_who w "
            "ON w.event_key = a.event_key WHERE a.seat = ? ORDER BY w.event_key, w.ordinal",
            (view.seat_id,),
        ):
            who.setdefault(record_key, []).extend((role, entity))
        out += [
            encoded("fact_who", [as_number(record_key), *cells])
            for record_key, cells in who.items()
        ]
        out += home.seat_section(key)
    finally:
        if opened:
            db.execute("COMMIT")
    return ("\n".join(out) + "\n").encode("utf-8")


# ── Import ─────────────────────────────────────────────────────────────────────────────────────────


@dataclass(frozen=True)
class _Record:
    event: EventId
    at: int
    kind: str
    place: str | None
    digest: str


@dataclass
class _Parsed:
    schema: str
    world_instance: str
    seat: EntityKey
    self_entity: EntityId
    records: list[_Record] = field(default_factory=list[_Record])
    who: dict[str, list[tuple[str, int]]] = field(default_factory=dict[str, list[tuple[str, int]]])
    meta: list[tuple[str, str]] = field(default_factory=list[tuple[str, str]])
    access: list[tuple[str, str, str, int, int]] = field(
        default_factory=list[tuple[str, str, str, int, int]]
    )
    rows: dict[str, list[tuple[Cell, ...]]] = field(
        default_factory=dict[str, list[tuple[Cell, ...]]]
    )
    scope_lines: int = 0
    """Lines of optional scopes' tables (pr-s28d §5.4), after every other section: checked, not written."""


_SECTIONS = ("fact", "fact_who", "meta", "names", "access", *(t.name for t in schema.PRIVATE))
"""The order of the tags after the header: each tag's lines are contiguous and in this order. Lines of an
optional scope's tables (pr-s28d) may follow them all: an import does not write them, and the seat's next
`MemoryHome.bind_scopes` rebuilds them from the imported records (QS28d-5; ledger D-d-7)."""
_SCOPE_TABLE = re.compile(r"^[a-z][a-z0-9_]{0,47}$")


def _key(value: object, number: int) -> str:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise NotASeatExport(f"line {number}: an id or key is not a non-negative number")
    text = str(value)
    if len(text) > KEY_WIDTH:
        raise NotASeatExport(f"line {number}: an id is longer than {KEY_WIDTH} digits")
    return text


def _text(value: object, number: int) -> str:
    if not isinstance(value, str):
        raise NotASeatExport(f"line {number}: a text cell is not a string")
    return value


def _int(value: object, number: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise NotASeatExport(f"line {number}: a number cell is not an integer")
    return value


def _cell(column: str, value: object, number: int) -> Cell:
    """A private table's cell back to its stored form: keys padded, ids unpadded, the rest as written."""
    if column in schema.KEY_COLUMNS:
        return None if value is None else event_key(EventId(_key(value, number)))
    if column in schema.ID_COLUMNS:
        return _key(value, number)
    if (
        value is None
        or isinstance(value, str)
        or (isinstance(value, int) and not isinstance(value, bool))
    ):
        return value
    raise NotASeatExport(f"line {number}: a cell is neither text, a number nor null")


def _meta(key: str, value: object, number: int) -> str:
    if key not in schema.NUMBER_META:
        return _text(value, number)
    if value is None:
        return ""
    text = _key(value, number)
    return event_key(EventId(text)) if key in _KEY_META else text


def _lines(data: bytes) -> list[list[object]]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        raise NotASeatExport("the file is not UTF-8") from None
    if not text.endswith("\n"):
        raise NotASeatExport("the last line has no newline")
    out: list[list[object]] = []
    for number, each in enumerate(text[:-1].split("\n"), start=1):
        try:
            cells: object = json.loads(each)
        except json.JSONDecodeError:
            raise NotASeatExport(f"line {number} is not a JSON array") from None
        if not isinstance(cells, list) or not cells or not isinstance(cells[0], str):
            raise NotASeatExport(f"line {number} is not a tagged JSON array")
        out.append(cast(list[object], cells))
    return out


def _header(cells: Sequence[object]) -> _Parsed:
    if cells[0] != MAGIC:
        raise NotASeatExport("line 1 is not a seat export header")
    if len(cells) < 2:
        raise NotASeatExport("line 1 has no format")
    if cells[1] != FORMAT:
        raise UnsupportedExportFormat(str(cells[1]), FORMAT)
    if len(cells) != 6 or not all(isinstance(each, str) for each in cells[2:]):
        raise NotASeatExport("line 1 is not a format-1 header")
    return _Parsed(
        schema=str(cells[2]),
        world_instance=str(cells[3]),
        seat=EntityKey(str(cells[4])),
        self_entity=EntityId(str(cells[5])),
    )


def parse(data: bytes) -> _Parsed:
    """Every line checked before anything is written."""
    lines = _lines(data)
    if not lines:
        raise NotASeatExport("the file is empty")
    parsed = _header(lines[0])
    position = 0
    for number, cells in enumerate(lines[1:], start=2):
        tag = str(cells[0])
        if tag not in _SECTIONS and _SCOPE_TABLE.fullmatch(tag):
            position = len(_SECTIONS)
            parsed.scope_lines += 1
            continue
        if tag not in _SECTIONS or _SECTIONS.index(tag) < position:
            raise NotASeatExport(f"line {number}: a {tag!r} line is out of place")
        position = _SECTIONS.index(tag)
        _parse_line(parsed, tag, cells[1:], number)
    return parsed


def _parse_line(parsed: _Parsed, tag: str, cells: Sequence[object], number: int) -> None:
    if tag == "fact":
        if len(cells) != 5:
            raise NotASeatExport(f"line {number}: a fact line has {len(cells)} cells, not 5")
        place = None if cells[3] is None else _text(cells[3], number)
        digest = _text(cells[4], number)
        if len(digest) != 64:
            raise NotASeatExport(f"line {number}: a fact's digest is not 64 hex characters")
        event = EventId(_key(cells[0], number))
        parsed.records.append(
            _Record(event, _int(cells[1], number), _text(cells[2], number), place, digest)
        )
    elif tag == "fact_who":
        if len(cells) < 3 or len(cells) % 2 != 1:
            raise NotASeatExport(f"line {number}: a fact_who line is not an id and role pairs")
        key = event_key(EventId(_key(cells[0], number)))
        pairs = [
            (_text(cells[i + 1], number), _int(cells[i], number)) for i in range(1, len(cells), 2)
        ]
        if any(role not in (1, 2, 3) for _, role in pairs):
            raise NotASeatExport(f"line {number}: a role is not 1, 2 or 3")
        parsed.who[key] = pairs
    elif tag == "meta":
        if len(cells) != 2:
            raise NotASeatExport(f"line {number}: a meta line has {len(cells)} cells, not 2")
        key = _text(cells[0], number)
        parsed.meta.append((key, _meta(key, cells[1], number)))
    elif tag == "access":
        if len(cells) != 5:
            raise NotASeatExport(f"line {number}: an access line has {len(cells)} cells, not 5")
        salience = _text(cells[2], number)
        if salience not in ("notable", "ambient"):
            raise NotASeatExport(f"line {number}: a salience is neither notable nor ambient")
        parsed.access.append(
            (
                event_key(EventId(_key(cells[0], number))),
                _text(cells[1], number),
                salience,
                _int(cells[3], number),
                _int(cells[4], number),
            )
        )
    else:
        table = schema.BY_NAME[tag]
        if len(cells) != len(table.columns):
            raise NotASeatExport(
                f"line {number}: a {tag} line has {len(cells)} cells, not {len(table.columns)}"
            )
        parsed.rows.setdefault(tag, []).append(
            tuple(_cell(column, cells[i], number) for i, column in enumerate(table.columns))
        )


def import_seat(home: MemoryHome, data: bytes) -> EntityKey:
    """Adds an exported seat to `home`: every refusal before the first write, then one transaction."""
    parsed = parse(data)
    if parsed.schema != str(schema.SCHEMA_VERSION):
        raise UnsupportedStoreSchema(home.path, parsed.schema, schema.SCHEMA_VERSION)
    if parsed.world_instance != home.world_instance:
        raise HomeIdentityMismatch("world_instance", home.world_instance, parsed.world_instance)
    if parsed.seat in home.seats():
        raise SeatAlreadyInHome(parsed.seat)
    held = {key for key, *_ in parsed.access}
    records = {event_key(record.event) for record in parsed.records}
    if held != records or not set(parsed.who) <= records:
        raise NotASeatExport("the records and the access rows name different facts")
    identity = StoreIdentity(
        world_instance=home.world_instance, seat=parsed.seat, self_entity=parsed.self_entity
    )
    with home.transaction(retention_pass=True) as db:
        for record in parsed.records:
            rows = parsed.who.get(event_key(record.event), [])
            world.insert_or_verify_record(
                db, record.event, record.at, record.kind, record.place, record.digest, rows.copy
            )
        view = home.seat(identity)
        for key, gist, salience, mine, importance in parsed.access:
            row = db.execute(
                "INSERT INTO access (seat, event_key, gist, salience, mine, importance) "
                "VALUES (?, ?, ?, ?, ?, ?)",
                (view.seat_id, key, gist, salience, mine, importance),
            )
            if home.fts and salience == "notable":
                db.execute(
                    "INSERT INTO access_text (rowid, gist, seat) VALUES (?, ?, ?)",
                    (row.lastrowid, gist, view.seat_id),
                )
        for table in (schema.NAMES, *schema.PRIVATE):
            marks = ", ".join("?" * (len(table.columns) + 1))
            db.executemany(
                f"INSERT INTO {table.name} (seat, {', '.join(table.columns)}) VALUES ({marks})",
                [(view.seat_id, *cells) for cells in parsed.rows.get(table.name, [])],
            )
        for key, value in parsed.meta:
            view.set_meta(key, value)
    return parsed.seat


__all__ = ["FORMAT", "MAGIC", "export_seat", "import_seat", "parse"]
