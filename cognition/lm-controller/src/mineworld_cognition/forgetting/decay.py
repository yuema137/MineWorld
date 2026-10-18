"""`decay` (pr-fg-b §5.9): an item survives while its importance, weighted by a forgetting curve of its age,
stays at or above a threshold.

Retention `r` (per mille) of `now - last_at`: `half-life` halves once per whole half-life, linear between
(`half_life_permille`); `hyperbolic` is `1000 * s // (s + age)`, the power law of exponent 1 that Murre and
Dros compare (their fitted parameters are not used). An L0, L1 or L2 item survives while
`importance * r // 1000 >= threshold`. A non-survivor nested in a surviving container merges into it; a
top-level one is dropped; one under a removed container follows it. If the survivors' bytes still exceed
`budget.store_bytes`, `consolidate`'s defaults are applied to the survivors. Stable facts are kept.
Integer arithmetic only.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, ClassVar, Literal

from pydantic import BaseModel, ConfigDict, Field

from mineworld_cognition.memory.retention import (
    Drop,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
    half_life_permille,
    hyperbolic_permille,
)

HOUR_S = 3_600
WEEK_S = 604_800
YEAR_S = 31_536_000
DEFAULT_L1_WEEKS = 8
DEFAULT_L2_YEARS = 5
"""`consolidate`'s defaults, applied to the survivors when they do not fit (§5.9)."""

Fate = Literal["keep", "merge", "drop"]


class DecayParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    curve: Literal["half-life", "hyperbolic"] = "half-life"
    half_life_hours: Annotated[int, Field(ge=1, le=87_600)] = 168
    scale_hours: Annotated[int, Field(ge=1, le=87_600)] = 72
    threshold: Annotated[int, Field(ge=0, le=1_000)] = 100


def order(event: str) -> tuple[int, str]:
    """A decimal id's numeric order without converting it."""
    return (len(event), event)


class Decay:
    id: ClassVar[str] = "decay"
    Parameters = DecayParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: DecayParameters
    ) -> Sequence[Keep | Merge | Drop]:
        def retention(item: MemoryItem) -> int:
            age = budget.now - item.last_at
            if params.curve == "half-life":
                return half_life_permille(age, params.half_life_hours * HOUR_S)
            return hyperbolic_permille(age, params.scale_hours * HOUR_S)

        fate: dict[ItemId, Fate] = {}
        for item in items:
            survives = (
                item.level == 3 or item.importance * retention(item) // 1000 >= params.threshold
            )
            fate[item.item] = "keep" if survives else "drop" if item.container is None else "merge"
        by_id = {item.item: item for item in items}
        if _surviving_bytes(items, fate, by_id) > budget.store_bytes:
            _consolidate_survivors(items, fate, by_id, budget.now)
        return decisions(items, fate, by_id)


def _gone(item: MemoryItem, fate: dict[ItemId, Fate], by_id: dict[ItemId, MemoryItem]) -> bool:
    """Whether the item or a container above it leaves."""
    current: MemoryItem | None = item
    while current is not None:
        if fate[current.item] != "keep":
            return True
        current = None if current.container is None else by_id[current.container]
    return False


def _surviving_bytes(
    items: Sequence[MemoryItem], fate: dict[ItemId, Fate], by_id: dict[ItemId, MemoryItem]
) -> int:
    return sum(item.bytes for item in items if not _gone(item, fate, by_id))


def _consolidate_survivors(
    items: Sequence[MemoryItem], fate: dict[ItemId, Fate], by_id: dict[ItemId, MemoryItem], now: int
) -> None:
    established = [
        order(r.first) for item in items if item.level == 3 for r in item.citation.ranges
    ]
    for item in items:
        if _gone(item, fate, by_id):
            continue
        if item.level == 0 and item.last_at < now:
            fate[item.item] = "merge"
        elif item.level == 1 and item.container is not None:
            if item.last_at < now - DEFAULT_L1_WEEKS * WEEK_S:
                fate[item.item] = "merge"
        elif item.level == 2 and item.last_at < now - DEFAULT_L2_YEARS * YEAR_S:
            holds = any(
                order(r.first) <= event <= order(r.last)
                for r in item.citation.ranges
                for event in established
            )
            if not holds:
                fate[item.item] = "drop"


def decisions(
    items: Sequence[MemoryItem], fate: dict[ItemId, Fate], by_id: dict[ItemId, MemoryItem]
) -> list[Keep | Merge | Drop]:
    """Drops of top-level items and merges grouped by container, naming nothing under a removed
    container (it follows its container)."""
    plan: list[Keep | Merge | Drop] = []
    groups: dict[tuple[ItemId, int], list[ItemId]] = {}
    for item in items:
        container = None if item.container is None else by_id[item.container]
        if container is not None and _gone(container, fate, by_id):
            continue
        if fate[item.item] == "drop":
            plan.append(Drop(item=item.item))
        elif fate[item.item] == "merge" and item.container is not None:
            groups.setdefault((item.container, item.level + 1), []).append(item.item)
    for (_, into), members in groups.items():
        plan.append(Merge(items=tuple(members), into=1 if into == 1 else 2))
    return plan
