"""The inspector's views (pr-s28e §§5.4 to 5.6), assembled from `memory/explain.py`, `ForgottenSet`, the seat view and
the home. Nothing here writes: every read goes through the seat view's and the home's own functions, or a `SELECT`
that names the seat.
"""

from __future__ import annotations

import json
from collections.abc import Set as AbstractSet
from fractions import Fraction

from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId

from mineworld_cognition.controller.memory_view import recall_view
from mineworld_cognition.inspect.inputs import Inputs, InspectRefusal, check_perceived
from mineworld_cognition.inspect.report import (
    AccessReport,
    Bound,
    BytesReport,
    ForgottenReport,
    HeldTable,
    HomeBytes,
    InstalledScope,
    PassRow,
    RecallReport,
    RowBytes,
    ScopesReport,
    SeatBytes,
    SeatSection,
    Source,
    StackedScope,
    TableBytes,
    TraceLine,
    TraceRange,
)
from mineworld_cognition.inspect.requests import RecallRequest
from mineworld_cognition.memory.explain import Forgotten, explain, held_counts
from mineworld_cognition.memory.home import MemoryHome, SeatView, schema
from mineworld_cognition.memory.home.encoding import Cell, as_number, encoded, line_bytes
from mineworld_cognition.memory.home.view import Row
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import EventRange, event_key, event_of
from mineworld_cognition.memory.retention.bound import StoreBounds
from mineworld_cognition.memory.retention.forgotten import ForgottenSet
from mineworld_cognition.memory.retention.selection import DEFAULT_CHOICE
from mineworld_cognition.memory.retrieve import RecallQuery, SectionLine
from mineworld_cognition.memory.scopes.interface import PERSON_SECTIONS
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack, resolve_stack

PERSON = "person"
"""The scope of every table of the seat view but the optional scopes' own (pr-s28d §5.4)."""
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
    owners = _owners(home)
    for position, table in enumerate((*schema.VIEW_TABLES, *home.hosted())):
        sized = view.row_bytes(table.name)
        tables.append(
            TableBytes(
                table=table.name,
                scope=owners.get(table.name, PERSON),
                rows=len(sized),
                bytes=sum(size for _, size in sized),
            )
        )
        rows += [(size, position, table.name, _row_key(table, cells)) for cells, size in sized]
    rows.sort(key=lambda row: (-row[0], row[1], row[3]))
    largest = tuple(
        RowBytes(table=name, scope=owners.get(name, PERSON), key=key, bytes=size)
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


def _owners(home: MemoryHome) -> dict[str, str]:
    """Each registered scope table's scope (pr-s28d §5.4); every other table is the person scope's."""
    rows: list[tuple[str, str]] = home.connection.execute(
        "SELECT name, scope FROM scope_tables"
    ).fetchall()
    return dict(rows)


def seat_stack(view: SeatView, inputs: Inputs) -> MemoryStack:
    """The seat's stack, as the seat would resolve it (`resolve_stack`): from `--decisions`, else `[person]`."""
    choice = DEFAULT_CHOICE if inputs.choice is None else inputs.choice
    return resolve_stack(choice, ScopeRegistry.installed(), seat=EntityKey(view.identity.seat))


def _stack_source(inputs: Inputs) -> Source:
    return "given" if inputs.choice is not None and inputs.choice.scopes is not None else "default"


def scopes(home: MemoryHome, seat: EntityKey, inputs: Inputs) -> ScopesReport:
    view = home.view_of(seat)
    registry = ScopeRegistry.installed()
    stack = seat_stack(view, inputs)
    chosen = (
        {}
        if inputs.choice is None or inputs.choice.scopes is None
        else {choice.id: choice.source or inputs.choice.source for choice in inputs.choice.scopes}
    )
    named = {bound.id for bound in stack.scopes}
    owners = _owners(home)
    held: list[HeldTable] = []
    for table in home.hosted():
        count = len(view.private_rows(table))
        if count:
            scope = owners[table.name]
            held.append(HeldTable(table=table.name, scope=scope, rows=count, named=scope in named))
    return ScopesReport(
        seat=seat,
        installed=tuple(
            InstalledScope(id=scope, distribution=registry.distribution(scope))
            for scope in registry.ids()
        ),
        stack=tuple(
            StackedScope(
                id=bound.id,
                parameters=json.dumps(
                    bound.parameters.model_dump(mode="json"), sort_keys=True, separators=(",", ":")
                ),
                source=None if chosen.get(bound.id) is None else str(chosen[bound.id]),
            )
            for bound in stack.scopes
        ),
        stack_source=_stack_source(inputs),
        tables=tuple(held),
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


# ── recall (§5.6) ──────────────────────────────────────────────────────────────────────────────────

TRACE_KEYS = 8
"""Held keys shown per cited range unless `--trace-all` (F-S28e-7)."""


def resolve_counterparts(view: SeatView, given: tuple[str, ...]) -> tuple[EntityId, ...]:
    """Each `--with` value: an entity id, or a name the seat's own name book holds exactly (F-S28e-8)."""
    by_name: dict[str, EntityId] = {}
    for entity, name in sorted(NameBook.load(view).known().items()):
        by_name.setdefault(name, entity)
    out: list[EntityId] = []
    for value in given:
        if value.isascii() and value.isdigit():
            out.append(EntityId(value))
        elif value in by_name:
            out.append(by_name[value])
        else:
            known = ", ".join(sorted(by_name)) or "none"
            raise InspectRefusal(
                f"--with: seat {view.identity.seat!r} knows no one by that name; its names: {known}"
            )
    return tuple(out)


def _range(
    view: SeatView,
    cited: EventRange,
    perceived: AbstractSet[EventId] | None,
    show_all: bool,
) -> TraceRange:
    first, last = event_key(cited.first), event_key(cited.last)
    db = view.connection
    inside: int = db.execute(
        "SELECT count(*) FROM access WHERE seat = ? AND event_key BETWEEN ? AND ?",
        (view.seat_id, first, last),
    ).fetchone()[0]
    limit = -1 if show_all else TRACE_KEYS
    keys: list[tuple[str]] = db.execute(
        "SELECT event_key FROM access WHERE seat = ? AND event_key BETWEEN ? AND ? "
        "ORDER BY event_key LIMIT ?",
        (view.seat_id, first, last, limit),
    ).fetchall()
    return TraceRange(
        first=explain(view, cited.first, perceived),
        last=None if cited.first == cited.last else explain(view, cited.last, perceived),
        held_inside=inside,
        shown=tuple(explain(view, event_of(key), perceived) for (key,) in keys),
    )


def _trace(
    view: SeatView, line: SectionLine, perceived: AbstractSet[EventId] | None, show_all: bool
) -> TraceLine:
    return TraceLine(
        level=line.level,
        # A line's section is its scope's: the person's five keys, else the scope's own id (pr-s28c §5.2).
        scope=PERSON if line.level in PERSON_SECTIONS else line.level,
        text=line.render(),
        citation=line.citation.token(),
        ranges=tuple(_range(view, cited, perceived, show_all) for cited in line.citation.ranges),
    )


def recall(home: MemoryHome, request: RecallRequest, inputs: Inputs) -> RecallReport:
    """The section a consult with these inputs receives, through the consult's own function (I-e-3)."""
    view = home.view_of(request.seat)
    perceived = perceived_for(view, inputs).perceived
    counterparts = resolve_counterparts(view, request.counterparts)
    query = RecallQuery(at=request.at, counterparts=counterparts, words=request.words)
    stack = seat_stack(view, inputs)
    section = recall_view(view, stack, query, inputs.episodes).section
    trace = (
        tuple(_trace(view, line, perceived, request.trace_all) for line in section.lines)
        if request.trace or request.trace_all
        else ()
    )
    return RecallReport(
        seat=request.seat,
        at=request.at,
        counterparts=counterparts,
        words=request.words,
        recall_episodes=inputs.episodes,
        recall_source="default" if inputs.recall_episodes is None else "given",
        stack=tuple(bound.id for bound in stack.scopes),
        stack_source=_stack_source(inputs),
        section=section.render(),
        trace=trace,
    )
