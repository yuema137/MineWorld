"""Fixture memory scopes for the registry, selection and host tests (pr-s28c §§7, 8).

Well-formed fixture scopes (`Annotations`, `Notes`) and one malformed class per load check. The hostile
scope `Cites` returns whatever lines a test plants in `Cites.lines`.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from importlib.metadata import EntryPoint
from typing import Annotated, ClassVar

from pydantic import BaseModel, Field

from mineworld_cognition.memory.scopes import (
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
        TableSpec("notes_index", ("event_key", "entity"), "event"),
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
    tables: ClassVar[tuple[TableSpec, ...]] = (TableSpec("notes_index", ("event_key",), "event"),)


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


def point(name: str, target: str) -> EntryPoint:
    """An entry point of the scope group, with no distribution (`(unknown distribution)`)."""
    return EntryPoint(name, f"scope_fixtures:{target}", GROUP)
