"""`consolidate`, the default policy (pr-fg-b §5.9): detail fades into summaries by age, and summaries older
than years are forgotten unless they hold what a stable fact rests on.

- An L0 record with `last_at < now - l0_hours * 3600` merges into its episode (default 0 hours: exact
  detail lives only for the current day, QFGb-3).
- An episode in a chapter with `last_at < now - l1_weeks * 604800` merges into the chapter.
- A chapter with `last_at < now - l2_years * 31536000` is dropped, except one whose ranges contain a
  stable fact's first, last or level fact (the moments that established what the seat knows of someone).
- Stable facts are kept.
- **Keepsakes** (pr-p6c-fgd §4.2): per counterpart, the first `keepsakes` exchanges — records of an
  `EXCHANGE_KINDS` event whose only counterpart is that person, a line I said to them or they said to me —
  stay records. Their episode does not merge into its chapter, and a chapter holding one is not dropped.
  `keepsakes = 0` is the rule above exactly.

Something already leaving with its container is not named again. Integer arithmetic only.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, ClassVar

from pydantic import BaseModel, ConfigDict, Field

from mineworld_cognition.memory.retention import (
    EXCHANGE_KINDS,
    Drop,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
)

HOUR_S = 3_600
WEEK_S = 604_800
YEAR_S = 31_536_000


class ConsolidateParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    l0_hours: Annotated[int, Field(ge=0, le=720)] = 0
    l1_weeks: Annotated[int, Field(ge=1, le=104)] = 8
    l2_years: Annotated[int, Field(ge=1, le=50)] = 5
    keepsakes: Annotated[int, Field(ge=0, le=64)] = 4
    """Per counterpart, how many of the first lines exchanged directly with them are kept as records."""


def order(event: str) -> tuple[int, str]:
    """A decimal id's numeric order without converting it: shorter is smaller, then lexical."""
    return (len(event), event)


def keepsakes(items: Sequence[MemoryItem], per_person: int) -> set[ItemId]:
    """Each counterpart's first `per_person` exchange records, in event order: a pure function of the items."""
    if per_person == 0:
        return set()
    by_person: dict[str, list[MemoryItem]] = {}
    for item in items:
        if item.level == 0 and item.kind in EXCHANGE_KINDS and len(item.counterparts) == 1:
            by_person.setdefault(item.counterparts[0], []).append(item)
    kept: set[ItemId] = set()
    for lines in by_person.values():
        lines.sort(key=lambda item: order(item.citation.ranges[0].first))
        kept.update(item.item for item in lines[:per_person])
    return kept


class Consolidate:
    id: ClassVar[str] = "consolidate"
    Parameters = ConsolidateParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: ConsolidateParameters
    ) -> Sequence[Keep | Merge | Drop]:
        now = budget.now
        established = [
            order(r.first) for item in items if item.level == 3 for r in item.citation.ranges
        ]
        by_id = {item.item: item for item in items}
        kept_records = keepsakes(items, params.keepsakes)
        kept_episodes = {by_id[record].container for record in kept_records}
        kept_chapters = {
            by_id[episode].container for episode in kept_episodes if episode is not None
        }

        def holds_stable(item: MemoryItem) -> bool:
            return any(
                order(r.first) <= event <= order(r.last)
                for r in item.citation.ranges
                for event in established
            )

        dropped = {
            item.item
            for item in items
            if item.level == 2
            and item.last_at < now - params.l2_years * YEAR_S
            and not holds_stable(item)
            and item.item not in kept_chapters
        }
        into_chapters: dict[ItemId, list[ItemId]] = {}
        for item in items:
            if (
                item.level == 1
                and item.container is not None
                and item.container not in dropped
                and item.item not in kept_episodes
                and item.last_at < now - params.l1_weeks * WEEK_S
            ):
                into_chapters.setdefault(item.container, []).append(item.item)
        leaving = dropped | {episode for group in into_chapters.values() for episode in group}
        episodes = {item.item: item for item in items if item.level == 1}
        into_episodes: dict[ItemId, list[ItemId]] = {}
        for item in items:
            if item.level != 0 or item.container is None or item.container in leaving:
                continue
            chapter = episodes[item.container].container
            if chapter in dropped or item.item in kept_records:
                continue
            if item.last_at < now - params.l0_hours * HOUR_S:
                into_episodes.setdefault(item.container, []).append(item.item)
        plan: list[Keep | Merge | Drop] = [Drop(item=chapter) for chapter in sorted(dropped)]
        plan += [Merge(items=tuple(group), into=2) for group in into_chapters.values()]
        plan += [Merge(items=tuple(group), into=1) for group in into_episodes.values()]
        return plan
