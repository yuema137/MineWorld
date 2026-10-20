"""The host bound (pr-fg-b §5.7.3; pr-fg-b2 §4.1): the backstop that holds the store under its target
whatever a policy did.

The bound is per person: 20 MiB by default, raisable by a world, never lower (`StoreBounds`; operator
rulings of 2026-10-14). After a plan is executed, while the canonical dump is over the limit, the host applies these steps in
order until it is under: `consolidate` with its defaults; every remaining record of a closed episode
merged into it, oldest episode first; the episodes of weeks with a chapter merged into it, oldest first;
chapters dropped, oldest first; chapterless closed episodes dropped, oldest first; stable facts dropped,
fewest meetings first. Sizes are accounted per item and confirmed by `dump()`. Every removal counts as
forced. A store still over after the last step is a defect or a pathological world, never silently over:
`RetentionBoundUnreachable`, and the pass rolls back.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from functools import cache

from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention.forgotten import HOST, Forgetter, PassScope, normalize
from mineworld_cognition.memory.retention.items import Extracted, extract, l3_item
from mineworld_cognition.memory.retention.plan import execute, nested, remove, validate
from mineworld_cognition.memory.retention.policy import RetentionBudget
from mineworld_cognition.memory.retention.registry import (
    PolicyError,
    PolicyRegistry,
    RegisteredPolicy,
    SourceSpan,
)

MIB = 1_048_576
FLOOR_MIB = 20
"""The default and the smallest bound a store may have (operator ruling of 2026-10-14: default 20 MB, a
world may raise it; pr-fg-b2 §1.1 R-FGb2-2)."""
CEILING_MIB = 1_024
"""An upper sanity bound: `dump()` holds the whole store in memory."""
LOG_RESERVE = 512
"""Bytes kept free under the target for the pass's own log row and meta update, written after the bound
is checked, so that the dump the pass leaves is under the target too."""
FALLBACK = "consolidate"


class StoreBoundOutOfRange(PolicyError):
    """A `store_bound_mib` below the floor or above the ceiling (pr-fg-b2 §4.1)."""

    def __init__(self, given: int, source: SourceSpan | None = None) -> None:
        where = f" ({source})" if source is not None else ""
        super().__init__(
            f"memory store_bound_mib {given}{where} is outside {FLOOR_MIB} … {CEILING_MIB}: a world may "
            f"raise the per-person bound, never set it below {FLOOR_MIB} MiB"
        )
        self.given = given
        self.floor = FLOOR_MIB
        self.ceiling = CEILING_MIB
        self.source = source


@dataclass(frozen=True)
class StoreBounds:
    """One store's bound and the figures derived from it (pr-fg-b2 §4.1). It cannot be built below the
    floor, so no `Retention` has a smaller bound."""

    mib: int = FLOOR_MIB

    def __post_init__(self) -> None:
        if isinstance(self.mib, bool) or not FLOOR_MIB <= self.mib <= CEILING_MIB:
            raise StoreBoundOutOfRange(self.mib)

    @property
    def bound(self) -> int:
        """The hard bound: no committed state `absorb` leaves exceeds it (pr-fg-b2 §4.6)."""
        return self.mib * MIB

    @property
    def target(self) -> int:
        """75 % of the bound: the dump after every pass is at most this."""
        return self.bound * 3 // 4

    @property
    def trigger(self) -> int:
        """A size pass runs when the store's content exceeds this (93.75 % of the bound)."""
        return self.bound - self.bound // 16

    @property
    def limit(self) -> int:
        """What the backstop drives the dump to: the target less the pass log's reserve."""
        return self.target - LOG_RESERVE


DEFAULT_BOUNDS = StoreBounds()


class RetentionBoundUnreachable(RuntimeError):
    def __init__(self, size: int, floor: int) -> None:
        super().__init__(
            f"the memory store is {size} bytes after every backstop step, over the {floor}-byte limit"
        )
        self.size = size
        self.floor = floor


@cache
def fallback() -> RegisteredPolicy:
    """`consolidate` with its defaults, found through the entry-point group as every policy is."""
    return PolicyRegistry.installed().bind(FALLBACK, {})


def consolidated(store: SeatView, now: int, bounds: StoreBounds) -> int:
    """`consolidate`'s defaults applied to the current items, its drops signed `host:consolidate`; the number
    of items removed."""
    extracted = extract(store)
    budget = RetentionBudget.model_construct(
        store_bytes=max(0, bounds.target - extracted.overhead), now=now, calls_left=0
    )
    resolved = validate(fallback().plan(extracted.items, budget), extracted)
    outcome = execute(store, resolved, extracted, Forgetter(now // DAY_S, HOST + FALLBACK))
    return outcome.merged + outcome.dropped


type Step = Callable[[Extracted, SeatView], list[tuple[ItemId, bool]]]


def enforce(store: SeatView, now: int, bounds: StoreBounds, scope: PassScope) -> tuple[int, int]:
    """Applies the backstop until the dump is at most `bounds.limit`; returns the items it removed and
    the dump's size it leaves. Every range it forgets is signed `host:<step>` with the pass's day, and
    normalised within the pass's `scope`."""
    limit = bounds.limit
    size = store.size()
    if size <= limit:
        return 0, size
    forced = consolidated(store, now, bounds)
    normalize(store, scope)
    size = store.size()
    for step, name in STEPS:
        if size <= limit:
            return forced, size
        extracted = extract(store)
        signed = Forgetter(now // DAY_S, HOST + name)
        order = step(extracted, store)
        removed, size = _shed(store, extracted, order, size, limit, scope, signed)
        forced += removed
    if size > limit:
        raise RetentionBoundUnreachable(size, limit)
    return forced, size


def _shed(
    store: SeatView,
    extracted: Extracted,
    order: list[tuple[ItemId, bool]],
    size: int,
    limit: int,
    scope: PassScope,
    signed: Forgetter,
) -> tuple[int, int]:
    """Removes candidates in order (each `(item, forget)`) until the accounted size is under `limit`, then
    confirms with one dump, and goes on if the dump says otherwise. A forgotten item is signed `signed`."""
    removed = 0
    estimate = size
    for item, forget in order:
        if estimate <= limit:
            normalize(store, scope)
            size = store.size()
            if size <= limit:
                return removed, size
            estimate = size
        estimate -= sum(extracted.by_id[each].bytes for each in nested(extracted, item))
        removed += remove(store, extracted, item, forget=signed if forget else None)
    normalize(store, scope)
    return removed, store.size()


def _closed_records(extracted: Extracted, store: SeatView) -> list[tuple[ItemId, bool]]:
    """Every record of a closed episode, oldest episode first (records follow in order)."""
    return [(item.item, False) for item in extracted.items if item.level == 0]


def _chaptered_episodes(extracted: Extracted, store: SeatView) -> list[tuple[ItemId, bool]]:
    episodes = [item for item in extracted.items if item.level == 1 and item.container is not None]
    weeks = sorted(episodes, key=lambda item: (item.day // 7, extracted.refs[item.item].number))
    return [(item.item, False) for item in weeks]


def _chapters(extracted: Extracted, store: SeatView) -> list[tuple[ItemId, bool]]:
    chapters = [item for item in extracted.items if item.level == 2]
    return [
        (item.item, True) for item in sorted(chapters, key=lambda i: extracted.refs[i.item].number)
    ]


def _chapterless_episodes(extracted: Extracted, store: SeatView) -> list[tuple[ItemId, bool]]:
    return [
        (item.item, True) for item in extracted.items if item.level == 1 and item.container is None
    ]


def _stable_facts(extracted: Extracted, store: SeatView) -> list[tuple[ItemId, bool]]:
    rows: list[tuple[str]] = store.connection.execute(
        "SELECT entity FROM stable WHERE seat = ? ORDER BY times_met, first_key, entity",
        (store.seat_id,),
    ).fetchall()
    return [(l3_item(entity), True) for (entity,) in rows]


STEPS: tuple[tuple[Step, str], ...] = (
    (_closed_records, "closed-records"),
    (_chaptered_episodes, "chaptered-episodes"),
    (_chapters, "chapters"),
    (_chapterless_episodes, "chapterless-episodes"),
    (_stable_facts, "stable-facts"),
)
"""Steps 2 … 6 of the backstop, in order, each with the name it signs its forgotten ranges with
(`host:<name>`; pr-s28b F-S28b-4). Only steps 4 and 5 forget a range: 2 and 3 merge, and 6 drops stable
facts, which cover nothing."""
