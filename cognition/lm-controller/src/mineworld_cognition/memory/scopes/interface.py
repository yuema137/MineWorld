"""The memory scope interface (pr-s28c-memory-scopes.md §5.2; MODULE_SPEC §5.3; `ARC-100`).

A **memory scope** organises and recalls what one person perceived. It declares the sections it may fill
(`sections`), the rows it would add when a seat first holds a fact (`index`, hosted by S28-d), its recall
(`recall`) and its share of the seat view (`account`). It never receives a connection, a path, a clock or a
gateway: it reads one seat's held facts through a `ScopeView`, and the host checks every line it returns.

The carriers a recall returns — `RecallQuery`, `SectionLine` and `SectionKey` — are defined here, because a
scope author needs them and the person scope's retrieval (`memory.retrieve`) re-exports them.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Annotated, ClassVar, Literal, Protocol

from mineworld_sdk.wire.ids import (
    EntityId,
    EntityIdField,
    EntityKey,
    EventId,
    EventIdField,
    EventTypeIdField,
)
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.records import Citation, NonNegative, Salience

ScopeId = Annotated[str, Field(pattern=r"^[a-z][a-z0-9-]{0,47}$")]
"""A scope's id: its entry-point name."""
PERSON_SECTIONS = ("with", "L3", "L2", "L1", "L0")
"""The person scope's section keys, in render order: `with` (pr-p6c-fgd), then P4's four levels. Reserved:
no other scope's id may be one of them."""
SectionKey = Annotated[str, Field(pattern=r"^(L[0-3]|[a-z][a-z0-9-]{0,47})$")]
"""A section's key: one of `PERSON_SECTIONS` for the person scope, a scope's id for any other scope's one
section."""


class RecallQuery(CognitionModel):
    at: int
    """The world time of the decision; L0 shows the window before it (86 400 s by default)."""
    counterparts: tuple[EntityId, ...] = ()
    """People present now or named in the trigger."""
    words: str = ""
    """The trigger's words, matched as literal terms (P4-4)."""
    speaker: EntityId | None = None
    """The person who just spoke to me, if someone did: the `with` section is theirs (pr-p6c-fgd §4.1)."""
    heard: EventId | None = None
    """The line just heard, which the prompt already shows: never repeated under `with`."""


class SectionLine(CognitionModel):
    level: SectionKey
    """The section the line belongs to: `with` or `L3` … `L0` for the person scope, a scope's id otherwise.
    The name `level` is P4's, kept so that every logged section parses unchanged (F-S28c-2)."""
    text: str
    citation: Citation

    def render(self) -> str:
        return f"{self.text} {self.citation.token()}"


class ScopeParameters(CognitionModel):
    """The base of every scope's `Parameters`: strict, frozen, unknown keys refused; every field defaulted."""


class SectionBudget(CognitionModel):
    lines: Annotated[int, Field(ge=0, le=48)]
    bytes: Annotated[int, Field(ge=0, le=6000)]
    """The section's bytes, its header and every newline included (P4's rule, QP4-7)."""


class SectionSpec(CognitionModel):
    key: SectionKey
    header: Annotated[str, Field(min_length=1, max_length=60, pattern=r"^[^\n]*:$")]
    """One line ending with `:`. The person scope's `with` header names the speaker (`{name}`)."""
    budget: SectionBudget


class FactRecord(CognitionModel):
    """A held record's envelope, as the world scope stores it."""

    event: EventIdField
    at: int
    kind: EventTypeIdField
    place: EntityIdField | None
    who: tuple[tuple[EntityIdField, Literal[1, 2, 3]], ...]
    """The record's `fact_who` rows in ordinal order, each with its role (QS28a-3)."""


class Reading(CognitionModel):
    """The seat's own rendering of a held record."""

    gist: str
    salience: Salience
    mine: bool
    importance: NonNegative


class ScopeBytes(CognitionModel):
    tables: tuple[tuple[str, NonNegative], ...]
    """(view table, bytes), in view order."""

    def total(self) -> int:
        return sum(size for _, size in self.tables)


@dataclass(frozen=True)
class TableSpec:
    """A scope's own table, declared now and hosted by S28-d (QS28c-4)."""

    name: str
    """`<scope id>_<name>`."""
    columns: tuple[str, ...]
    keyed_by: Literal["event", "seat", "seat-event"]


class ScopeRow(CognitionModel):
    """One row `index` returns, hosted by S28-d (QS28c-4)."""

    table: str
    cells: tuple[str | int | None, ...]


class ScopeView(Protocol):
    """What a scope reads: one seat's held facts, nothing else. Every method refuses a key the seat does not
    hold with `AccessDenied` (raised by the view, naming the scope, the seat and the key)."""

    @property
    def seat(self) -> EntityKey: ...

    @property
    def me(self) -> EntityId: ...

    def holds(self, event: EventId) -> bool: ...

    def record(self, event: EventId) -> tuple[FactRecord, Reading]: ...

    def held_between(self, first_at: int, last_at: int, limit: int) -> tuple[EventId, ...]:
        """The held events with `first_at <= at <= last_at`, newest first, at most `limit`."""
        ...

    def name(self, entity: EntityId) -> str | None:
        """The seat's own name for `entity` (its name book), or `None`."""
        ...


class MemoryScope[P: ScopeParameters](Protocol):
    id: ClassVar[str]
    Parameters: type[P]
    tables: ClassVar[tuple[TableSpec, ...]]

    def sections(self, params: P) -> tuple[SectionSpec, ...]: ...

    def index(self, fact: FactRecord, reading: Reading, params: P) -> tuple[ScopeRow, ...]: ...

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: P,
    ) -> tuple[SectionLine, ...]: ...

    def account(self, view: ScopeView) -> ScopeBytes: ...
