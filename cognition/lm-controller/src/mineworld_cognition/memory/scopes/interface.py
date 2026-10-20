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
from mineworld_cognition.memory.anchors import Anchor, AnchorKind
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
    anchors: tuple[Anchor, ...] = ()
    """The places, items and organizations the person is with now (pr-s28d §5.7): the anchored scopes' targets.
    The person scope never reads it."""


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
    anchors: tuple[Anchor, ...] = ()
    """The fact's payload anchors (`memory.anchors`, pr-s28d §5.2): the places, items and organizations its
    payload names by typed reference."""


class ScopeBytes(CognitionModel):
    tables: tuple[tuple[str, NonNegative], ...]
    """(view table, bytes), in view order."""

    def total(self) -> int:
        return sum(size for _, size in self.tables)


type MergeRole = Literal["key", "first", "last", "count"]
"""How the home merges a second row with the same key into the stored one (pr-s28d §5.4): `key` columns form
the primary key; `first` keeps the stored value; `last` takes the new one; `count` adds the new one."""
type ColumnType = Literal["text", "key", "int"]
"""`text` and `int` as SQLite stores them; `key`: an `EventKey`, which must be the held fact's, written in a
dump as its number."""


@dataclass(frozen=True)
class Column:
    name: str
    """`^[a-z][a-z0-9_]{0,47}$`, never `seat`."""
    type: ColumnType
    merge: MergeRole


@dataclass(frozen=True)
class TableSpec:
    """A scope's own per-seat table, hosted by the memory home (QS28c-4; pr-s28d §5.4, QS28d-3)."""

    name: str
    """`<scope id, each - written _>_<name>`: a bare SQL name."""
    columns: tuple[Column, ...]
    keyed_by: Literal["seat"]
    """Per-seat tables only: an event-keyed table is not hosted until a scope needs one (F-S28d-4)."""
    cap: str | None = None
    """The name of an `int` field of the scope's `Parameters`: the most rows a seat keeps in this table."""
    evict: tuple[str, ...] = ()
    """Columns, ascending: the order in which the cap and the bound's backstop delete rows."""


class ScopeRow(CognitionModel):
    """One row `index` returns: the table (one of the scope's own) and its cells, in column order."""

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

    def anchored(
        self,
        entity: EntityId,
        *,
        at: bool,
        named: frozenset[AnchorKind],
        until: int,
        limit: int,
        notable: bool,
    ) -> tuple[EventId, ...]:
        """The held facts anchored to `entity` at or before `until`, newest first, at most `limit`: its place
        is `entity` (with `at`), or its payload names `entity` with a kind in `named`; `notable` keeps only
        notable ones (pr-s28d §5.5)."""
        ...

    def own_rows(
        self, table: str, where: tuple[tuple[str, str | int], ...]
    ) -> tuple[ScopeRow, ...]:
        """This seat's rows of one of the calling scope's own tables whose named `key`-role columns equal the
        given values, in key order. Another scope's table, or a column that is not a key column, is
        `AccessDenied`."""
        ...

    def shown(self, event: EventId) -> bool:
        """Whether a line admitted earlier in this recall cites a range that contains `event`."""
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
