"""The `person` scope: the person's private layer and its recall (pr-s28c-memory-scopes.md §5.4, F-S28c-6).

This module owns the scope: its class, its parameters and its recall entry. Its implementation stays where
P4 put it, and is named here rather than moved: `memory.ingest` (holding what is perceived), `compress/`
(episodes, chapters, stable facts) and `memory.retrieve` (the five sections). A base scope (QS28c-1): the host
passes it a `BaseView`, and it declares no table of its own.

Its parameters can only shrink today's sections (step-28 §5.4); the defaults are `memory.retrieve.BUDGETS`, so
the default stack renders exactly today's section (I-c-1).
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Annotated, ClassVar

from pydantic import Field, model_validator

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.retrieve import BUDGETS, LEVELS, Budget, SectionLevel, retrieve
from mineworld_cognition.memory.scopes.errors import ScopeError
from mineworld_cognition.memory.scopes.interface import (
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
from mineworld_cognition.memory.scopes.view import BaseView


class PersonSection(CognitionModel):
    """One of the person scope's sections: at most its default lines and bytes."""

    lines: Annotated[int, Field(ge=0, le=48, description="The most lines the section shows.")]
    bytes: Annotated[
        int,
        Field(
            ge=0,
            le=6000,
            description="The section's most bytes, its header and every newline included.",
        ),
    ]


def _default(level: SectionLevel) -> PersonSection:
    return PersonSection(lines=BUDGETS[level].lines, bytes=BUDGETS[level].bytes)


class PersonSections(CognitionModel):
    """The five sections, each defaulting to today's budget; each may only shrink."""

    with_: PersonSection = Field(
        default_factory=lambda: _default("with"),
        alias="with",
        description="What the speaker and I said to each other before: 6 lines, 1 000 bytes.",
    )
    L3: PersonSection = Field(
        default_factory=lambda: _default("L3"),
        description="People I know (stable facts): 16 lines, 1 600 bytes.",
    )
    L2: PersonSection = Field(
        default_factory=lambda: _default("L2"),
        description="Recent weeks (chapters): 4 lines, 800 bytes.",
    )
    L1: PersonSection = Field(
        default_factory=lambda: _default("L1"),
        description="Episodes I remember: 8 lines, 1 600 bytes. A consult also caps the lines at "
        "[controller] recall_episodes.",
    )
    L0: PersonSection = Field(
        default_factory=lambda: _default("L0"),
        description="The last day (notable records): 48 lines, 1 000 bytes.",
    )

    def of(self, level: SectionLevel) -> PersonSection:
        sections: dict[SectionLevel, PersonSection] = {
            "with": self.with_,
            "L3": self.L3,
            "L2": self.L2,
            "L1": self.L1,
            "L0": self.L0,
        }
        return sections[level]

    @model_validator(mode="after")
    def _only_shrink(self) -> PersonSections:
        for level in LEVELS:
            given, default = self.of(level), BUDGETS[level]
            if given.lines > default.lines:
                raise ValueError(f"{level}.lines may not exceed its default {default.lines}")
            if given.bytes > default.bytes:
                raise ValueError(f"{level}.bytes may not exceed its default {default.bytes}")
        return self


class PersonParameters(ScopeParameters):
    sections: PersonSections = Field(
        default_factory=PersonSections,
        description="Each section's lines and bytes; each may only shrink from its default.",
    )
    l0_window_s: Annotated[int, Field(ge=3600, le=86400)] = Field(
        default=DAY_S,
        description="How far back, in world seconds, the last-day section looks (D-P4-11).",
    )


class PersonScopeMisused(ScopeError):
    """The person scope was given a view that is not a base view: a host defect."""


class PersonScope:
    """The person's private layer and its recall: what the speaker and I said before, the people I know, recent
    weeks, the episodes I remember and the last day. Required, and first in every stack."""

    id: ClassVar[str] = "person"
    Parameters = PersonParameters
    tables: ClassVar[tuple[TableSpec, ...]] = ()

    def sections(self, params: PersonParameters) -> tuple[SectionSpec, ...]:
        return tuple(
            SectionSpec(
                key=level,
                header=BUDGETS[level].header,
                budget=SectionBudget(
                    lines=params.sections.of(level).lines, bytes=params.sections.of(level).bytes
                ),
            )
            for level in LEVELS
        )

    def index(
        self, fact: FactRecord, reading: Reading, params: PersonParameters
    ) -> tuple[ScopeRow, ...]:
        return ()

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: PersonParameters,
    ) -> tuple[SectionLine, ...]:
        chosen: dict[SectionLevel, Budget] = {
            level: Budget(BUDGETS[level].header, budgets[level].lines, budgets[level].bytes)
            for level in LEVELS
        }
        return retrieve(_base(view).seat_view, query, chosen, params.l0_window_s).lines

    def account(self, view: ScopeView) -> ScopeBytes:
        """The seat view's tables but `meta`, each with its bytes, in view order: their sum is the seat's
        meter (`content_size()`, F-S28c-16)."""
        seat = _base(view).seat_view
        return ScopeBytes(
            tables=tuple(
                (table.name, sum(size for _, size in seat.row_bytes(table.name)))
                for table in schema.VIEW_TABLES
                if table.name != "meta"
            )
        )


def _base(view: ScopeView) -> BaseView:
    if not isinstance(view, BaseView):
        raise PersonScopeMisused("memory.scopes: the person scope reads only a base view")
    return view
