"""Fixture memory scopes for the registry, selection and host tests (pr-s28c §§7, 8).

Well-formed fixture scopes (`Annotations`, `Notes`) and one malformed class per load check. The hostile
scope `Cites` returns whatever lines a test plants in `Cites.lines`.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from importlib.metadata import EntryPoint
from typing import Annotated, ClassVar

from pydantic import BaseModel, Field

from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.scopes import (
    Column,
    FactRecord,
    Reading,
    RecallQuery,
    ScopeBytes,
    ScopeParameters,
    ScopeRow,
    ScopeView,
    SectionBudget,
    SectionLine,
    SectionSpec,
    TableSpec,
)
from mineworld_cognition.memory.scopes.registry import GROUP


class NotesParameters(ScopeParameters):
    section_lines: Annotated[int, Field(ge=0, le=16)] = Field(
        default=0, description="Lines of the notes section."
    )
    section_bytes: Annotated[int, Field(ge=0, le=1600)] = Field(
        default=0, description="Bytes of the notes section."
    )
    secret: str = Field(default="", description="A field a test plants a value in.")


class Notes:
    """A well-formed optional scope with one section, sized by its parameters, and no lines."""

    id: ClassVar[str] = "notes"
    Parameters = NotesParameters
    tables: ClassVar[tuple[TableSpec, ...]] = (
        TableSpec("notes_index", (Column("entity", "text", "key"),), "seat"),
    )

    def sections(self, params: NotesParameters) -> tuple[SectionSpec, ...]:
        return (
            SectionSpec(
                key=self.id,
                header="Notes:",
                budget=SectionBudget(lines=params.section_lines, bytes=params.section_bytes),
            ),
        )

    def index(
        self, fact: FactRecord, reading: Reading, params: NotesParameters
    ) -> tuple[ScopeRow, ...]:
        return ()

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: NotesParameters,
    ) -> tuple[SectionLine, ...]:
        return ()

    def account(self, view: ScopeView) -> ScopeBytes:
        return ScopeBytes(tables=())


class CitesParameters(ScopeParameters):
    section_lines: Annotated[int, Field(ge=0, le=16)] = Field(default=4, description="Lines.")
    section_bytes: Annotated[int, Field(ge=0, le=1600)] = Field(default=400, description="Bytes.")


class Cites:
    """A hostile scope: its recall returns the lines planted in `lines`, or raises `fails`; it records the
    view it was given."""

    id: ClassVar[str] = "cites"
    Parameters = CitesParameters
    tables: ClassVar[tuple[TableSpec, ...]] = ()
    lines: ClassVar[list[SectionLine]] = []
    fails: ClassVar[list[Exception]] = []
    seen: ClassVar[list[ScopeView]] = []
    reads: ClassVar[list[Callable[[ScopeView], object]]] = []

    def sections(self, params: CitesParameters) -> tuple[SectionSpec, ...]:
        return (
            SectionSpec(
                key=self.id,
                header="Cited:",
                budget=SectionBudget(lines=params.section_lines, bytes=params.section_bytes),
            ),
        )

    def index(
        self, fact: FactRecord, reading: Reading, params: CitesParameters
    ) -> tuple[ScopeRow, ...]:
        return ()

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: CitesParameters,
    ) -> tuple[SectionLine, ...]:
        Cites.seen.append(view)
        for read in Cites.reads:
            read(view)
        if Cites.fails:
            raise Cites.fails[0]
        return tuple(Cites.lines)

    def account(self, view: ScopeView) -> ScopeBytes:
        return ScopeBytes(tables=())

    @staticmethod
    def reset() -> None:
        Cites.lines.clear()
        Cites.fails.clear()
        Cites.seen.clear()
        Cites.reads.clear()


# ── Malformed: one load check each ────────────────────────────────────────────────────────────────


def not_a_class() -> None:
    return None


class WrongId(Notes):
    id: ClassVar[str] = "something-else"


class UndefaultedParameters(ScopeParameters):
    needed: int


class Undefaulted(Notes):
    id: ClassVar[str] = "undefaulted"
    Parameters = UndefaultedParameters  # pyright: ignore[reportAssignmentType, reportIncompatibleVariableOverride]


class PlainParameters(BaseModel):
    lines: int = 0


class NotScopeParameters(Notes):
    id: ClassVar[str] = "plain"
    Parameters = PlainParameters  # pyright: ignore[reportAssignmentType, reportIncompatibleVariableOverride]


class ForeignTable(Notes):
    id: ClassVar[str] = "foreign"
    tables: ClassVar[tuple[TableSpec, ...]] = (
        TableSpec("notes_index", (Column("entity", "text", "key"),), "seat"),
    )


class NoRecall:
    id: ClassVar[str] = "norecall"
    Parameters = NotesParameters
    tables: ClassVar[tuple[TableSpec, ...]] = ()

    def sections(self, params: NotesParameters) -> tuple[SectionSpec, ...]:
        return ()

    def index(
        self, fact: FactRecord, reading: Reading, params: NotesParameters
    ) -> tuple[ScopeRow, ...]:
        return ()

    def account(self, view: ScopeView) -> ScopeBytes:
        return ScopeBytes(tables=())


class NeedsArguments(Notes):
    id: ClassVar[str] = "needs-arguments"
    tables: ClassVar[tuple[TableSpec, ...]] = ()

    def __init__(self, required: int) -> None:
        self.required = required


# ── pr-s28d: hosted tables ────────────────────────────────────────────────────────────────────────


class TallyParameters(ScopeParameters):
    most: Annotated[int, Field(ge=1, le=10_000)] = Field(
        default=10_000, description="The most rows a seat keeps."
    )


TALLY_COLUMNS = (
    Column("entity", "text", "key"),
    Column("first_key", "key", "first"),
    Column("last_key", "key", "last"),
    Column("first_at", "int", "first"),
    Column("last_at", "int", "last"),
    Column("times", "int", "count"),
)


class Tally:
    """A well-formed optional scope with one hosted table: one row per person a fact names (envelope `who`),
    merged by first, last and count; capped by `most`, evicting the least recently seen."""

    id: ClassVar[str] = "tally"
    Parameters = TallyParameters
    tables: ClassVar[tuple[TableSpec, ...]] = (
        TableSpec("tally_seen", TALLY_COLUMNS, "seat", cap="most", evict=("last_at", "entity")),
    )

    def sections(self, params: TallyParameters) -> tuple[SectionSpec, ...]:
        return ()

    def index(
        self, fact: FactRecord, reading: Reading, params: TallyParameters
    ) -> tuple[ScopeRow, ...]:
        key = event_key(fact.event)
        entities = dict.fromkeys(entity for entity, _ in fact.who)
        return tuple(
            ScopeRow(table="tally_seen", cells=(entity, key, key, fact.at, fact.at, 1))
            for entity in entities
        )

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: TallyParameters,
    ) -> tuple[SectionLine, ...]:
        return ()

    def account(self, view: ScopeView) -> ScopeBytes:
        return ScopeBytes(tables=())


class Hostile(Tally):
    """Its `index` returns the rows a test plants in `rows`, whatever the fact."""

    id: ClassVar[str] = "hostile"
    tables: ClassVar[tuple[TableSpec, ...]] = (TableSpec("hostile_seen", TALLY_COLUMNS, "seat"),)
    rows: ClassVar[list[object]] = []

    def index(
        self, fact: FactRecord, reading: Reading, params: TallyParameters
    ) -> tuple[ScopeRow, ...]:
        planted = [row(fact) if callable(row) else row for row in Hostile.rows]
        return tuple(planted)  # pyright: ignore[reportReturnType]


def _bad(name: str, table: TableSpec) -> type:
    return type(name.title().replace("-", ""), (Tally,), {"id": name, "tables": (table,)})


def _seen(scope: str) -> str:
    return scope.replace("-", "_") + "_seen"


NoKey = _bad("nokey", TableSpec(_seen("nokey"), (Column("n", "int", "count"),), "seat"))
CountOnText = _bad(
    "counttext",
    TableSpec(
        _seen("counttext"), (Column("e", "text", "key"), Column("n", "text", "count")), "seat"
    ),
)
SeatColumn = _bad("seatcol", TableSpec(_seen("seatcol"), (Column("seat", "text", "key"),), "seat"))
CapMissing = _bad(
    "capmissing",
    TableSpec(_seen("capmissing"), TALLY_COLUMNS, "seat", cap="absent", evict=("entity",)),
)
CapNoEvict = _bad("capnoevict", TableSpec(_seen("capnoevict"), TALLY_COLUMNS, "seat", cap="most"))
EvictMissing = _bad(
    "evictmissing", TableSpec(_seen("evictmissing"), TALLY_COLUMNS, "seat", evict=("absent",))
)
EventKeyed = _bad(
    "eventkeyed",
    TableSpec(_seen("eventkeyed"), TALLY_COLUMNS, "event"),  # pyright: ignore[reportArgumentType]
)


def point(name: str, target: str) -> EntryPoint:
    """An entry point of the scope group, with no distribution (`(unknown distribution)`)."""
    return EntryPoint(name, f"scope_fixtures:{target}", GROUP)
