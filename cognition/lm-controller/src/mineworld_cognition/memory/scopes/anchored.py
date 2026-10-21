"""The anchored scope: what a person knows of places and things (pr-s28d-anchored-scope.md §5.6; MODULE_SPEC §5.3).

One implementation, `AnchoredScope`, configured per anchor kind by its parameters; the bundled entry points are
`place` (`roles: [at]`, `kinds: [place]`) and `item` (`roles: [named]`, `kinds: [item]`) (QS28d-1). A fact anchors an
entity by `at` (the envelope's place) or `named` (a typed reference in its payload, `memory.anchors`).

- **`index`** returns one row per anchor of the fact, never the seat's own entity; the home merges it into the
  anchor's known row (`first`, `last`, `count`), so `visits` counts the perceived facts anchored to the entity
  since the first the seat held (F-S28d-9). The table is capped at `anchors_max`, the least recently seen anchor
  evicted first (QS28d-4).
- **`recall`** is off by default (`section_bytes: 0`). On, for the anchors the person is with now
  (`RecallQuery.anchors`), it offers one known line per anchor, citing the newest held fact anchored to it (no
  such fact: no line, F-S28d-7), then the notable gists anchored to them that no earlier section shows.

It names no event type, entity id or world (I-d-8): every input is the contract's typed reference or the envelope.
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Annotated, ClassVar, Literal, Self

from mineworld_sdk.wire.ids import EntityId, EventId
from pydantic import Field, model_validator

from mineworld_cognition.compress.summarize import clock, day_number
from mineworld_cognition.memory.anchors import AnchorKind
from mineworld_cognition.memory.records import Citation, event_key
from mineworld_cognition.memory.retrieve import LINE_BYTES, cut
from mineworld_cognition.memory.scopes.interface import (
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

type AnchorRole = Literal["at", "named"]
GENERIC: dict[AnchorKind, str] = {
    "place": "somewhere",
    "item": "something",
    "organization": "some group",
}
"""The label of an anchor the seat has no name for: a line never shows an id (I-10)."""
CITE_LOOKBACK = 64
"""How many of an anchor's newest held facts a known line looks through for one no earlier line cites."""


class AnchoredParameters(ScopeParameters):
    roles: list[AnchorRole] = Field(
        min_length=1,
        description="How a fact anchors an entity: `at` (the fact happened there, the envelope's place) and "
        "`named` (the fact's payload names it by typed reference).",
    )
    kinds: list[AnchorKind] = Field(
        min_length=1,
        description="The kinds of entity this scope remembers: `place`, `item`, `organization`.",
    )
    anchors_max: Annotated[int, Field(ge=1, le=10_000)] = Field(
        default=256,
        description="The most anchors a person keeps a known row for; beyond it the one seen least "
        "recently is forgotten first.",
    )
    section_lines: Annotated[int, Field(ge=0, le=16)] = Field(
        default=4, description="Lines of the recall section."
    )
    section_bytes: Annotated[int, Field(ge=0, le=1_600)] = Field(
        default=0,
        description="Bytes of the recall section, its header and newlines included; 0 (the default) turns "
        "the section off. It counts against the 6 000-byte ceiling with every other section.",
    )

    @model_validator(mode="after")
    def _consistent(self) -> Self:
        if len(set(self.roles)) != len(self.roles) or len(set(self.kinds)) != len(self.kinds):
            raise ValueError("roles and kinds are each listed once")
        if "at" in self.roles and "place" not in self.kinds:
            raise ValueError("the role `at` anchors places: `kinds` must include `place`")
        return self


class PlaceParameters(AnchoredParameters):
    roles: list[AnchorRole] = Field(
        default=["at"],
        min_length=1,
        description="How a fact anchors a place: `at` (the fact happened there) and `named` (its payload "
        "names it).",
    )
    kinds: list[AnchorKind] = Field(
        default=["place"],
        min_length=1,
        description="The kinds of entity this scope remembers: `place`, `item`, `organization`.",
    )


class ItemParameters(AnchoredParameters):
    roles: list[AnchorRole] = Field(
        default=["named"],
        min_length=1,
        description="How a fact anchors a thing: `named` (its payload names it by typed reference).",
    )
    kinds: list[AnchorKind] = Field(
        default=["item"],
        min_length=1,
        description="The kinds of entity this scope remembers: `place`, `item`, `organization`.",
    )


def known_table(scope: str) -> TableSpec:
    return TableSpec(
        name=f"{scope}_known",
        keyed_by="seat",
        cap="anchors_max",
        evict=("last_at", "entity"),
        columns=(
            Column("entity", "text", "key"),
            Column("first_key", "key", "first"),
            Column("last_key", "key", "last"),
            Column("first_at", "int", "first"),
            Column("last_at", "int", "last"),
            Column("visits", "int", "count"),
        ),
    )


class AnchoredScope:
    """The shared implementation; `PlaceScope` and `ItemScope` name it and set its defaults."""

    id: ClassVar[str] = "anchored"
    header: ClassVar[str] = "Things I know:"
    Parameters: type[AnchoredParameters] = AnchoredParameters
    tables: ClassVar[tuple[TableSpec, ...]] = ()

    def sections(self, params: AnchoredParameters) -> tuple[SectionSpec, ...]:
        budget = SectionBudget(lines=params.section_lines, bytes=params.section_bytes)
        return (SectionSpec(key=self.id, header=self.header, budget=budget),)

    def index(
        self, fact: FactRecord, reading: Reading, params: AnchoredParameters
    ) -> tuple[ScopeRow, ...]:
        key = event_key(fact.event)
        table = f"{self.id}_known"
        return tuple(
            ScopeRow(table=table, cells=(entity, key, key, fact.at, fact.at, 1))
            for entity in _anchors(fact, reading, params)
        )

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: AnchoredParameters,
    ) -> tuple[SectionLine, ...]:
        budget = budgets[self.id]
        if budget.lines == 0 or budget.bytes == 0:
            return ()
        targets = list(dict.fromkeys(a.entity for a in query.anchors if a.kind in params.kinds))
        kinds: dict[EntityId, AnchorKind] = {a.entity: a.kind for a in query.anchors}
        section = _Section(self.id, self.header, budget)
        cited: set[EventId] = set()
        reads = _Reads(view, params, query.at)
        for entity in targets:
            rows = view.own_rows(f"{self.id}_known", (("entity", entity),))
            # The newest held fact anchored to it that no line before shows (C-3 check 4).
            newest = [
                event
                for event in reads.anchored(entity, limit=CITE_LOOKBACK, notable=False)
                if event not in cited and not view.shown(event)
            ][:1]
            if not rows or not newest:
                continue
            _, _, _, first_at, last_at, visits = rows[0].cells
            name = _name(view, entity, kinds[entity])
            text = (
                f"{name}: {visits} moments I remember, Day {day_number(_int(first_at))} to "
                f"Day {day_number(_int(last_at))}"
            )
            if not section.offer(text, newest[0]):
                return section.lines()
            cited.add(newest[0])
        gists: dict[EventId, EntityId] = {}
        for entity in targets:
            for event in reads.anchored(entity, limit=budget.lines, notable=True):
                gists.setdefault(event, entity)
        for event in sorted(gists, key=event_key, reverse=True):
            if event in cited or view.shown(event):
                continue
            fact, reading = view.record(event)
            name = _name(view, gists[event], kinds[gists[event]])
            text = f"{name}, Day {day_number(fact.at)} {clock(fact.at)}: {reading.gist}"
            if not section.offer(text, event):
                break
            cited.add(event)
        return section.lines()

    def account(self, view: ScopeView) -> ScopeBytes:
        """The known table's bytes: the scope's share of the seat view (its rows are dumped by the home)."""
        rows = view.own_rows(f"{self.id}_known", ())
        size = sum(_row_bytes(f"{self.id}_known", row.cells) for row in rows)
        return ScopeBytes(tables=((f"{self.id}_known", size),))


class PlaceScope(AnchoredScope):
    """The places a person knows: where the facts they perceived happened, how often, from when to when."""

    id: ClassVar[str] = "place"
    header: ClassVar[str] = "Places I know:"
    Parameters = PlaceParameters
    tables: ClassVar[tuple[TableSpec, ...]] = (known_table("place"),)


class ItemScope(AnchoredScope):
    """The things a person knows: the items the facts they perceived name, how often, from when to when."""

    id: ClassVar[str] = "item"
    header: ClassVar[str] = "Things I know:"
    Parameters = ItemParameters
    tables: ClassVar[tuple[TableSpec, ...]] = (known_table("item"),)


# ── Helpers ───────────────────────────────────────────────────────────────────────────────────


def _anchors(fact: FactRecord, reading: Reading, params: AnchoredParameters) -> list[EntityId]:
    """The fact's anchors for this scope, in order, deduplicated: its place (`at`), then the payload's
    anchors of the scope's kinds (`named`). The seat's own entity is never among the reading's anchors
    (`render` leaves it out), and a place is never a person."""
    found: list[EntityId] = []
    if "at" in params.roles and fact.place is not None:
        found.append(fact.place)
    if "named" in params.roles:
        found += [a.entity for a in reading.anchors if a.kind in params.kinds]
    return list(dict.fromkeys(found))


class _Reads:
    def __init__(self, view: ScopeView, params: AnchoredParameters, until: int) -> None:
        self.view = view
        self.at = "at" in params.roles
        self.named: frozenset[AnchorKind] = (
            frozenset(params.kinds) if "named" in params.roles else frozenset()
        )
        self.until = until

    def anchored(self, entity: EntityId, *, limit: int, notable: bool) -> tuple[EventId, ...]:
        return self.view.anchored(
            entity, at=self.at, named=self.named, until=self.until, limit=limit, notable=notable
        )


class _Section:
    """P4's offering rule: each line cut to 160 bytes with its citation; stop at the first line over the
    section's lines or bytes, header and newlines counted."""

    def __init__(self, key: str, header: str, budget: SectionBudget) -> None:
        self.key = key
        self.budget = budget
        self.used = len(header.encode("utf-8")) + 1
        self._lines: list[SectionLine] = []

    def offer(self, text: str, event: EventId) -> bool:
        citation = Citation.of(event)
        line = SectionLine(level=self.key, text=cut(text, citation), citation=citation)
        size = len(line.render().encode("utf-8")) + 1
        if len(self._lines) >= self.budget.lines or self.used + size > self.budget.bytes:
            return False
        assert size <= LINE_BYTES + 1
        self._lines.append(line)
        self.used += size
        return True

    def lines(self) -> tuple[SectionLine, ...]:
        return tuple(self._lines)


def _name(view: ScopeView, entity: EntityId, kind: AnchorKind) -> str:
    return view.name(entity) or GENERIC[kind]


def _int(value: object) -> int:
    assert isinstance(value, int)
    return value


def _row_bytes(table: str, cells: tuple[str | int | None, ...]) -> int:
    """The bytes of a known row's dump line: `["<table>","<entity>",<first>,<last>,<first_at>,<last_at>,
    <visits>]` and its newline, keys written as their numbers."""
    entity, first, last, *numbers = cells
    written = [f'"{entity}"', *(_number(v) for v in (first, last)), *(str(n) for n in numbers)]
    return len(f'["{table}",{",".join(written)}]') + 1


def _number(key: object) -> str:
    """An `EventKey` (decimal digits, zero-padded) as the JSON number a dump writes."""
    assert isinstance(key, str) and key.isdigit()
    return key.lstrip("0") or "0"
