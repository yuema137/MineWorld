"""`importance-recency-cap` (pr-fg-b §5.9): keep the best-scoring items per level, within the budget.

`score = (w_importance * importance + w_recency * r) // 1000`, with `r` the half-life retention of the
item's age (Generative Agents' recency x importance, in integers). Greedy by score, top-down: top-level
units (chapters, chapterless episodes) up to their level's `k` and `store_bytes`; then the episodes inside
chosen chapters; then the records inside chosen episodes; then stable facts. Equal scores are ordered by
`tie_key(seed, item)`, the sha256 of the seed and the item's first Event ID: a priority-sampling key
without RNG state, so the selection is a pure function. Unchosen nested items merge into their container;
unchosen top-level items and stable facts are dropped.
"""

from __future__ import annotations

from collections.abc import Callable, Sequence
from typing import Annotated, ClassVar

from pydantic import BaseModel, ConfigDict, Field

from mineworld_cognition.memory.retention import (
    Drop,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
    half_life_permille,
    tie_key,
)

HOUR_S = 3_600


class ImportanceRecencyCapParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    w_importance: Annotated[int, Field(ge=0, le=1_000)] = 700
    w_recency: Annotated[int, Field(ge=0, le=1_000)] = 300
    half_life_hours: Annotated[int, Field(ge=1, le=87_600)] = 168
    k_l0: Annotated[int, Field(ge=0, le=100_000)] = 2_000
    k_l1: Annotated[int, Field(ge=0, le=100_000)] = 400
    k_l2: Annotated[int, Field(ge=0, le=100_000)] = 260
    k_l3: Annotated[int, Field(ge=0, le=10_000)] = 150
    seed: Annotated[int, Field(ge=0, le=4_294_967_295)] = 0


class ImportanceRecencyCap:
    id: ClassVar[str] = "importance-recency-cap"
    Parameters = ImportanceRecencyCapParameters

    def plan(
        self,
        items: Sequence[MemoryItem],
        budget: RetentionBudget,
        params: ImportanceRecencyCapParameters,
    ) -> Sequence[Keep | Merge | Drop]:
        def score(item: MemoryItem) -> int:
            r = half_life_permille(budget.now - item.last_at, params.half_life_hours * HOUR_S)
            return (params.w_importance * item.importance + params.w_recency * r) // 1000

        def rank(item: MemoryItem) -> tuple[int, int, str]:
            return (-score(item), tie_key(params.seed, item), item.item)

        caps = (params.k_l0, params.k_l1, params.k_l2, params.k_l3)
        chosen: set[ItemId] = set()
        counts = [0, 0, 0, 0]
        used = 0

        def choose(candidates: list[MemoryItem]) -> None:
            nonlocal used
            for item in sorted(candidates, key=rank):
                if (
                    counts[item.level] < caps[item.level]
                    and used + item.bytes <= budget.store_bytes
                ):
                    chosen.add(item.item)
                    counts[item.level] += 1
                    used += item.bytes

        def under_chosen(level: int) -> Callable[[MemoryItem], bool]:
            return lambda item: item.level == level and item.container in chosen

        choose([item for item in items if item.level in (1, 2) and item.container is None])
        choose([item for item in items if under_chosen(1)(item)])
        choose([item for item in items if under_chosen(0)(item)])
        choose([item for item in items if item.level == 3])
        plan: list[Keep | Merge | Drop] = []
        groups: dict[tuple[ItemId, int], list[ItemId]] = {}
        for item in items:
            if item.item in chosen:
                continue
            if item.container is None:
                plan.append(Drop(item=item.item))
            elif item.container in chosen:
                groups.setdefault((item.container, item.level + 1), []).append(item.item)
        for (_, into), members in groups.items():
            plan.append(Merge(items=tuple(members), into=1 if into == 1 else 2))
        return plan
