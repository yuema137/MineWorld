"""The inspector's views (pr-s28e §§5.4 to 5.6), assembled from `memory/explain.py`, `ForgottenSet`, the seat view and
the home. Nothing here writes: every read goes through the seat view's and the home's own functions, or a `SELECT`
that names the seat.
"""

from __future__ import annotations

from fractions import Fraction

from mineworld_sdk.wire.ids import EntityKey, EventId

from mineworld_cognition.inspect.inputs import Inputs, check_perceived
from mineworld_cognition.inspect.report import (
    AccessReport,
    Bound,
    BytesReport,
    ForgottenReport,
    HomeBytes,
    PassRow,
    RowBytes,
    SeatBytes,
    SeatSection,
    Source,
    TableBytes,
)
from mineworld_cognition.memory.explain import Forgotten, explain, held_counts
from mineworld_cognition.memory.home import MemoryHome, SeatView, schema
from mineworld_cognition.memory.home.encoding import Cell, as_number, encoded, line_bytes
from mineworld_cognition.memory.home.view import Row
from mineworld_cognition.memory.records import event_of
from mineworld_cognition.memory.retention.bound import StoreBounds
from mineworld_cognition.memory.retention.forgotten import ForgottenSet

PERSON = "person"
"""The scope of every seat-view table until S28-c's `account` says otherwise (QS28e-4)."""
LARGEST = 5


def held_ids(view: SeatView) -> frozenset[EventId]:
    rows: list[tuple[str]] = view.connection.execute(
        "SELECT event_key FROM access WHERE seat = ?", (view.seat_id,)
    ).fetchall()
    return frozenset(event_of(key) for (key,) in rows)


def perceived_for(view: SeatView, inputs: Inputs) -> Inputs:
    """`inputs`, its perceived set checked against the seat (D-e-4)."""
    if inputs.perceived is not None:
        check_perceived(inputs.perceived, held_ids(view), view.identity.seat)
    return inputs


def show_fact(seat: str, event: EventId) -> str:
    since = int(event) - 1
    return f"mineworld perceived <world> --save <save> --person {seat} --since {since} --json"


def access(
    home: MemoryHome, seat: EntityKey, event: EventId | None, inputs: Inputs
) -> AccessReport:
    view = home.view_of(seat)
    perceived = perceived_for(view, inputs).perceived
    source: Source = "default" if perceived is None else "given"
    if event is None:
        return AccessReport(seat=seat, counts=held_counts(view), perceived=source)
    return AccessReport(
        seat=seat,
        explanation=explain(view, event, perceived),
        perceived=source,
        show_fact=show_fact(seat, event),
    )


def forgotten(home: MemoryHome, seat: EntityKey) -> ForgottenReport:
    view = home.view_of(seat)
    found = ForgottenSet.load(view)
    table = schema.BY_NAME["retention_passes"]
    logged = sorted(
        (cells for _, cells in view.private_rows(table)),
        key=lambda cells: str(cells["trigger"]),
        reverse=True,
    )
    passes = [
        PassRow.model_validate({**cells, "trigger": event_of(str(cells["trigger"]))})
        for cells in logged
    ]
    return ForgottenReport(
        seat=seat,
        ranges=tuple(Forgotten.of(each) for each in found.attributed()),
        watermark=found.watermark(),
        passes=tuple(passes),
    )


def _row_key(table: schema.ViewTable, cells: Row) -> str:
    """A row's ordering cells, comma-joined; keys written as ids, as the dump writes them."""
    columns = [column.strip() for column in table.order.split(",")]
    columns = [column for column in columns if column in cells]
    if table.name == "l0_who":
        columns.append("entity")
    values: list[str] = []
    for column in columns:
        value = cells[column]
        if column in schema.KEY_COLUMNS and isinstance(value, str) and value:
            value = event_of(value)
        values.append("-" if value is None else str(value))
    return ",".join(values)


def seat_bytes(home: MemoryHome, seat: EntityKey, inputs: Inputs) -> SeatBytes:
    view = home.view_of(seat)
    tables: list[TableBytes] = []
    rows: list[tuple[int, int, str, str]] = []
    for position, table in enumerate(schema.VIEW_TABLES):
        sized = view.row_bytes(table.name)
        tables.append(
            TableBytes(
                table=table.name,
                scope=PERSON,
                rows=len(sized),
                bytes=sum(size for _, size in sized),
            )
        )
        rows += [(size, position, table.name, _row_key(table, cells)) for cells, size in sized]
    rows.sort(key=lambda row: (-row[0], row[1], row[3]))
    largest = tuple(
        RowBytes(table=name, scope=PERSON, key=key, bytes=size)
        for size, _, name, key in rows[:LARGEST]
    )
    bounds = StoreBounds(inputs.bound_mib)
    given = inputs.choice is not None and inputs.choice.store_bound_mib is not None
    bound = Bound(
        mib=bounds.mib,
        bound=bounds.bound,
        target=bounds.target,
        trigger=bounds.trigger,
        source="given" if given else "default",
    )
    content = view.content_size()
    return SeatBytes(
        seat=seat,
        size=view.size(),
        content_size=content,
        tables=tuple(tables),
        largest=largest,
        bound=bound,
        fill_per_mille=content * 1000 // bounds.bound,
        physical_share=_physical_share(view),
    )


def _physical_share(view: SeatView) -> int:
    """Σ over the seat's held records of (the record's `fact` and `fact_who` home-dump bytes / its holders)."""
    db = view.connection
    share = Fraction(0)
    fact_codecs = tuple(schema.codec(column) for column in schema.FACT_COLUMNS)
    for key, event_id, at, kind, place, digest, holders in db.execute(
        "SELECT f.event_key, f.event_id, f.at, f.kind, f.place, f.digest, "
        "(SELECT count(*) FROM access h WHERE h.event_key = f.event_key) "
        "FROM access a JOIN fact f ON f.event_key = a.event_key WHERE a.seat = ? ORDER BY f.event_key",
        (view.seat_id,),
    ):
        size = line_bytes("fact", (event_id, at, kind, place, digest), fact_codecs)
        who: list[tuple[str, int]] = db.execute(
            "SELECT entity, role FROM fact_who WHERE event_key = ? ORDER BY ordinal", (key,)
        ).fetchall()
        if who:
            cells: list[Cell] = [as_number(key)]
            for entity, role in who:
                cells += [role, entity]
            size += len(encoded("fact_who", cells).encode("utf-8")) + 1
        share += Fraction(size, holders)
    return int(share)


def home_bytes(home: MemoryHome) -> HomeBytes:
    totals = home.totals()
    return HomeBytes(
        home_meta=totals.home_meta,
        fact=totals.fact,
        fact_who=totals.fact_who,
        seats=tuple(SeatSection(seat=key, bytes=size) for key, size in totals.seats),
        dump=totals.dump,
        file=totals.file,
        free=totals.free,
    )


def bytes_view(home: MemoryHome, seat: EntityKey | None, inputs: Inputs) -> BytesReport:
    return BytesReport(
        seat=None if seat is None else seat_bytes(home, seat, inputs), home=home_bytes(home)
    )
