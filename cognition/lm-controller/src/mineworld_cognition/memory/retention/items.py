"""The items a pass shows a policy (pr-fg-b §5.7.0, §5.4): every L0 record of a closed episode, every closed
episode, every chapter and every stable fact, with host-computed importance and their own dump bytes.

Everything is read from the store's dump rows (`SeatView.row_bytes`), so an item's `bytes` is exactly
what removing its rows subtracts from `dump()`, and the values shown are the ones the bound is measured on.
The open episode and its record are never items; their bytes are part of the pass's overhead.
"""

from __future__ import annotations

from dataclasses import dataclass

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.compress.stable import stable_text
from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.home.encoding import Cell
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Citation, EventRange, ItemId, StableFact, event_of
from mineworld_cognition.memory.retention.policy import ItemLevel, MemoryItem

BONUS_CAP = 200
IMPORTANCE_CAP = 1000
OVERHEAD_TABLES = ("meta", "names", "forgotten", "retention_passes")
"""Tables whose bytes no policy can remove: the pass's fixed overhead, with the open episode's rows."""

type Row = dict[str, Cell]


@dataclass(frozen=True)
class Ref:
    """Where an item's rows are."""

    level: ItemLevel
    key: str
    """`event_key` (L0), `first_key` (L1, L2), the entity (L3)."""
    last_key: str
    """`last_key` (L1, L2); equal to `key` otherwise."""
    number: int
    """The episode number (L1) or the week (L2); 0 otherwise."""


@dataclass(frozen=True)
class Extracted:
    items: tuple[MemoryItem, ...]
    refs: dict[ItemId, Ref]
    children: dict[ItemId, tuple[ItemId, ...]]
    """Container → the items nested directly in it, in key order."""
    overhead: int
    by_id: dict[ItemId, MemoryItem]
    total: int
    """The store's whole dump size at extraction: every table's rows, each line with its newline."""


def text(row: Row, name: str) -> str:
    value = row[name]
    assert isinstance(value, str), name
    return value


def number(row: Row, name: str) -> int:
    value = row[name]
    assert isinstance(value, int), name
    return value


def optional(row: Row, name: str) -> str | None:
    value = row[name]
    assert value is None or isinstance(value, str), name
    return value


def l0_item(key: str) -> ItemId:
    return ItemId(f"l0:{key}")


def l1_item(episode: int) -> ItemId:
    return ItemId(f"l1:{episode}")


def l2_item(week: int) -> ItemId:
    return ItemId(f"l2:{week}")


def l3_item(entity: str) -> ItemId:
    return ItemId(f"l3:{entity}")


def _range(first: str, last: str) -> EventRange:
    return EventRange.model_construct(first=event_of(first), last=event_of(last))


def _owned(rows: list[tuple[Row, int]], key: str) -> tuple[dict[Cell, int], dict[Cell, list[str]]]:
    """Participant (or range) rows' bytes per owner, and the entities they name."""
    sizes: dict[Cell, int] = {}
    who: dict[Cell, list[str]] = {}
    for row, size in rows:
        sizes[row[key]] = sizes.get(row[key], 0) + size
        if "entity" in row:
            who.setdefault(row[key], []).append(text(row, "entity"))
    return sizes, who


class _Extraction:
    def __init__(self, store: SeatView) -> None:
        self.store = store
        self.items: list[MemoryItem] = []
        self.refs: dict[ItemId, Ref] = {}
        self.children: dict[ItemId, list[ItemId]] = {}
        self.total = 0
        self.overhead = sum(size for table in OVERHEAD_TABLES for _, size in self.rows(table))
        stable = self.rows("stable")
        self.exchanges = {text(row, "entity"): number(row, "exchanges") for row, _ in stable}
        self.stable = stable

    def rows(self, table: str) -> list[tuple[Row, int]]:
        """A table's dump rows; every table is read exactly once, so `total` ends equal to the dump."""
        found = self.store.row_bytes(table)
        self.total += sum(size for _, size in found)
        return found

    def importance(self, base: int, counterparts: tuple[str, ...]) -> int:
        """The base plus a reinforcement bonus from the stable facts at this moment (§5.4)."""
        best = max((self.exchanges.get(entity, 0) for entity in counterparts), default=0)
        return min(IMPORTANCE_CAP, base + min(BONUS_CAP, 2 * best))

    def add(self, item: MemoryItem, ref: Ref) -> None:
        self.items.append(item)
        self.refs[item.item] = ref
        if item.container is not None:
            self.children.setdefault(item.container, []).append(item.item)

    def episodes(self, chaptered: set[int]) -> list[tuple[str, str, int, bool]]:
        """Closed episodes as items; returns every episode's span, in key order."""
        sizes, who = _owned(self.rows("episode_who"), "episode")
        spans: list[tuple[str, str, int, bool]] = []
        for row, size in self.rows("episodes"):
            episode, first, last = (
                number(row, "episode"),
                text(row, "first_key"),
                text(row, "last_key"),
            )
            closed = number(row, "closed") == 1
            spans.append((first, last, episode, closed))
            own = size + sizes.get(episode, 0)
            if not closed:
                self.overhead += own
                continue
            week = number(row, "day") // 7
            counterparts = tuple(who.get(episode, ()))
            place = optional(row, "place")
            self.add(
                MemoryItem.model_construct(
                    item=l1_item(episode),
                    level=1,
                    container=l2_item(week) if week in chaptered else None,
                    first_at=number(row, "opened_at"),
                    last_at=number(row, "closed_at"),
                    day=number(row, "opened_at") // DAY_S,
                    kind="episode",
                    counterparts=tuple(EntityId(c) for c in counterparts),
                    place=None if place is None else EntityId(place),
                    mine=number(row, "mine") == 1,
                    importance=self.importance(number(row, "importance"), counterparts),
                    citation=Citation.model_construct(ranges=(_range(first, last),)),
                    text=text(row, "text"),
                    bytes=own,
                ),
                Ref(1, first, last, episode),
            )
        return spans

    def records(self, spans: list[tuple[str, str, int, bool]]) -> None:
        """Every record of a closed episode, nested in it. Records and episodes are both in key order,
        so one walk finds each record's episode."""
        sizes, who = _owned(self.rows("l0_who"), "event_key")
        # pr-s28d §5.8: a record's payload anchors belong to it, as its participants do.
        anchors, _ = _owned(self.rows("access_anchor"), "event_key")
        index = 0
        for row, size in self.rows("l0"):
            key = text(row, "event_key")
            while spans[index][1] < key:
                index += 1
            first, _, episode, closed = spans[index]
            assert first <= key, f"record {key} lies in no episode"
            own = size + sizes.get(key, 0) + anchors.get(key, 0)
            if not closed:
                self.overhead += own
                continue
            counterparts = tuple(who.get(key, ()))
            place = optional(row, "place")
            at = number(row, "at")
            event = EventId(text(row, "event_id"))
            self.add(
                MemoryItem.model_construct(
                    item=l0_item(key),
                    level=0,
                    container=l1_item(episode),
                    first_at=at,
                    last_at=at,
                    day=at // DAY_S,
                    kind=text(row, "kind"),
                    counterparts=tuple(EntityId(c) for c in counterparts),
                    place=None if place is None else EntityId(place),
                    mine=number(row, "mine") == 1,
                    importance=self.importance(number(row, "importance"), counterparts),
                    citation=Citation.model_construct(
                        ranges=(EventRange.model_construct(first=event, last=event),)
                    ),
                    text=text(row, "gist"),
                    bytes=own,
                ),
                Ref(0, key, key, 0),
            )

    def chapters(self, chapters: list[tuple[Row, int]]) -> None:
        range_rows = self.rows("chapter_ranges")
        sizes, _ = _owned(range_rows, "week")
        ranges: dict[int, list[EventRange]] = {}
        for row, _ in range_rows:
            ranges.setdefault(number(row, "week"), []).append(
                _range(text(row, "first_key"), text(row, "last_key"))
            )
        for row, size in chapters:
            week = number(row, "week")
            self.add(
                MemoryItem.model_construct(
                    item=l2_item(week),
                    level=2,
                    container=None,
                    first_at=number(row, "first_at"),
                    last_at=number(row, "last_at"),
                    day=number(row, "first_at") // DAY_S,
                    kind="chapter",
                    counterparts=(),
                    place=None,
                    mine=number(row, "mine") == 1,
                    importance=min(IMPORTANCE_CAP, number(row, "importance")),
                    citation=Citation.model_construct(ranges=tuple(ranges[week])),
                    text=text(row, "text"),
                    bytes=size + sizes.get(week, 0),
                ),
                Ref(2, text(row, "first_key"), text(row, "last_key"), week),
            )

    def stable_facts(self) -> None:
        names = NameBook.load(self.store)
        for row, size in self.stable:
            fact = _stable_fact(row)
            self.add(
                MemoryItem.model_construct(
                    item=l3_item(fact.entity),
                    level=3,
                    container=None,
                    first_at=fact.first_at,
                    last_at=fact.last_at,
                    day=fact.first_at // DAY_S,
                    kind="stable",
                    counterparts=(fact.entity,),
                    place=None,
                    mine=False,
                    importance=min(IMPORTANCE_CAP, fact.exchanges + 10 * fact.times_met),
                    citation=fact.citation(),
                    text=stable_text(fact, names),
                    bytes=size,
                ),
                Ref(3, fact.entity, fact.entity, 0),
            )


def extract(store: SeatView) -> Extracted:
    """The pass's items, in order: episodes, their records, chapters, stable facts."""
    extraction = _Extraction(store)
    chapters = extraction.rows("chapters")
    spans = extraction.episodes({number(row, "week") for row, _ in chapters})
    extraction.records(spans)
    extraction.chapters(chapters)
    extraction.stable_facts()
    return Extracted(
        tuple(extraction.items),
        extraction.refs,
        {key: tuple(value) for key, value in extraction.children.items()},
        extraction.overhead,
        {item.item: item for item in extraction.items},
        extraction.total,
    )


def _stable_fact(row: Row) -> StableFact:
    level_key = optional(row, "level_key")
    return StableFact(
        entity=EntityId(text(row, "entity")),
        first=event_of(text(row, "first_key")),
        last=event_of(text(row, "last_key")),
        first_at=number(row, "first_at"),
        last_at=number(row, "last_at"),
        times_met=number(row, "times_met"),
        exchanges=number(row, "exchanges"),
        level=optional(row, "level"),
        level_event=None if level_key is None else event_of(level_key),
    )
