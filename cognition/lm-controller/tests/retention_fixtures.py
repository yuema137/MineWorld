"""Test-only retention policies (pr-fg-b §8.1): one hostile plan per validation rule of §5.7.1, a policy
that raises, one that returns a `dict`, the most aggressive legal plan, one that tries to change what it
was shown, and classes the registry must refuse.

Each is registered through `PolicyRegistry({...})`, the way a user's policy is exercised without installing
a distribution. They import only `mineworld_cognition.memory.retention`'s public names, as a user's would.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, ClassVar

from pydantic import BaseModel, ConfigDict, Field

from mineworld_cognition.memory.retention import (
    Drop,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
)


class NoParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


def _top(items: Sequence[MemoryItem]) -> list[MemoryItem]:
    return [item for item in items if item.container is None]


def _records(items: Sequence[MemoryItem]) -> list[MemoryItem]:
    return [item for item in items if item.level == 0]


class UnknownItem:
    id: ClassVar[str] = "unknown-item"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return [Keep(item=ItemId("l0:99999999999999999999"))]


class Duplicate:
    id: ClassVar[str] = "duplicate"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        first = _top(items)[0].item
        return [Keep(item=first), Keep(item=first)]


class MergeMismatch:
    """L0 records of two different episodes merged as if they shared one."""

    id: ClassVar[str] = "merge-mismatch"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        records = _records(items)
        other = next(r for r in records if r.container != records[0].container)
        return [Merge(items=(records[0].item, other.item), into=1)]


class DropNested:
    id: ClassVar[str] = "drop-nested"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return [Drop(item=_records(items)[0].item)]


class KeepUnderRemoved:
    """Drops an episode and explicitly keeps one of its records."""

    id: ClassVar[str] = "keep-under-removed"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        record = _records(items)[0]
        assert record.container is not None
        return [Drop(item=record.container), Keep(item=record.item)]


class Raises:
    id: ClassVar[str] = "raises"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        raise RuntimeError("a policy failed")


class ReturnsDict:
    id: ClassVar[str] = "returns-dict"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> object:
        return {"keep": [item.item for item in items]}


class DropEverything:
    """The most aggressive legal plan: every top-level item dropped, stable facts included."""

    id: ClassVar[str] = "drop-everything"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return [Drop(item=item.item) for item in _top(items)]


class DropDayOne:
    """Forgets every chapterless episode of day 1 and keeps the rest, so that week 0's chapter is written
    later over a week with a hole in it."""

    id: ClassVar[str] = "drop-day-one"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return [
            Drop(item=item.item)
            for item in items
            if item.level == 1 and item.container is None and item.day == 1
        ]


class Mutates:
    """Keeps everything, after trying to lower what the host computed: the item is frozen."""

    id: ClassVar[str] = "mutates"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        items[0].importance = 0  # pyright: ignore[reportAttributeAccessIssue]
        return []


class HoursParameters(BaseModel):
    """Written as a user might, without `extra="forbid"`: Pydantic alone would ignore an unknown key, so
    the registry's own refusal of one is what a test of it observes."""

    model_config = ConfigDict(strict=True, frozen=True)
    hours: Annotated[int, Field(ge=0, le=720)] = 0


class KeepsNothingNamed:
    """A user's policy with one bounded parameter, which names no item: everything is kept."""

    id: ClassVar[str] = "user-policy"
    Parameters = HoursParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: HoursParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return [Keep(item=item.item) for item in items if item.level == 3]


class WrongId:
    id: ClassVar[str] = "something-else"
    Parameters = NoParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: NoParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return []


class RequiredParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")
    hours: int


class NoDefault:
    id: ClassVar[str] = "no-default"
    Parameters = RequiredParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: RequiredParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return []


HOSTILE = {
    "unknown-item": UnknownItem,
    "duplicate": Duplicate,
    "merge-mismatch": MergeMismatch,
    "drop-nested": DropNested,
    "keep-under-removed": KeepUnderRemoved,
    "raises": Raises,
    "returns-dict": ReturnsDict,
    "mutates": Mutates,
}
"""Policy id → class, for the plans the host must refuse whole."""
