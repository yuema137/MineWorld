"""The host bound (pr-fg-b §5.7.3): the backstop that holds the store under `TARGET` whatever a policy did.

After a plan is executed, while the canonical dump is over the limit, the host applies these steps in
order until it is under: `consolidate` with its defaults; every remaining record of a closed episode
merged into it, oldest episode first; the episodes of weeks with a chapter merged into it, oldest first;
chapters dropped, oldest first; chapterless closed episodes dropped, oldest first; stable facts dropped,
fewest meetings first. Sizes are accounted per item and confirmed by `dump()`. Every removal counts as
forced. A store still over after the last step is a defect or a pathological world, never silently over:
`RetentionBoundUnreachable`, and the pass rolls back.
"""

from __future__ import annotations

from collections.abc import Callable
from functools import cache

from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention.forgotten import normalize
from mineworld_cognition.memory.retention.items import Extracted, extract, l3_item
from mineworld_cognition.memory.retention.plan import execute, nested, remove, validate
from mineworld_cognition.memory.retention.policy import RetentionBudget
from mineworld_cognition.memory.retention.registry import PolicyRegistry, RegisteredPolicy
from mineworld_cognition.memory.store import MemoryStore

TARGET = 786_432
"""768 KiB: the dump after every pass is at most this. Not a parameter (step-24 §1.2 ruling 2 (4))."""
STORE_BOUND = 1_048_576
"""1 MiB: the hard bound. `TARGET` leaves room for one simulated day of perception before the next pass."""
LOG_RESERVE = 512
"""Bytes kept free under `TARGET` for the pass's own log row and meta update, written after the bound is
checked, so that the dump the pass leaves is under `TARGET` too."""
LIMIT = TARGET - LOG_RESERVE
FALLBACK = "consolidate"


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


def consolidated(store: MemoryStore, now: int) -> int:
    """`consolidate`'s defaults applied to the current items; the number of items removed."""
    extracted = extract(store)
    budget = RetentionBudget.model_construct(
        store_bytes=max(0, TARGET - extracted.overhead), now=now, calls_left=0
    )
    resolved = validate(fallback().plan(extracted.items, budget), extracted)
    outcome = execute(store.connection, resolved, extracted, fts=store.fts)
    return outcome.merged + outcome.dropped


def enforce(store: MemoryStore, now: int) -> tuple[int, int]:
    """Applies the backstop until the dump is at most `LIMIT`; returns the items it removed and the
    dump's size it leaves."""
    size = store.size()
    if size <= LIMIT:
        return 0, size
    forced = consolidated(store, now)
    normalize(store.connection)
    size = store.size()
    steps: tuple[Callable[[Extracted, MemoryStore], list[tuple[ItemId, bool]]], ...] = (
        _closed_records,
        _chaptered_episodes,
        _chapters,
        _chapterless_episodes,
        _stable_facts,
    )
    for step in steps:
        if size <= LIMIT:
            return forced, size
        extracted = extract(store)
        removed, size = _shed(store, extracted, step(extracted, store), size)
        forced += removed
    if size > LIMIT:
        raise RetentionBoundUnreachable(size, LIMIT)
    return forced, size


def _shed(
    store: MemoryStore, extracted: Extracted, order: list[tuple[ItemId, bool]], size: int
) -> tuple[int, int]:
    """Removes candidates in order (each `(item, forget)`) until the accounted size is under `LIMIT`, then
    confirms with one dump, and goes on if the dump says otherwise."""
    removed = 0
    estimate = size
    for item, forget in order:
        if estimate <= LIMIT:
            normalize(store.connection)
            size = store.size()
            if size <= LIMIT:
                return removed, size
            estimate = size
        estimate -= sum(extracted.by_id[each].bytes for each in nested(extracted, item))
        removed += remove(store.connection, extracted, item, fts=store.fts, forget=forget)
    normalize(store.connection)
    return removed, store.size()


def _closed_records(extracted: Extracted, store: MemoryStore) -> list[tuple[ItemId, bool]]:
    """Every record of a closed episode, oldest episode first (records follow in order)."""
    return [(item.item, False) for item in extracted.items if item.level == 0]


def _chaptered_episodes(extracted: Extracted, store: MemoryStore) -> list[tuple[ItemId, bool]]:
    episodes = [item for item in extracted.items if item.level == 1 and item.container is not None]
    weeks = sorted(episodes, key=lambda item: (item.day // 7, extracted.refs[item.item].number))
    return [(item.item, False) for item in weeks]


def _chapters(extracted: Extracted, store: MemoryStore) -> list[tuple[ItemId, bool]]:
    chapters = [item for item in extracted.items if item.level == 2]
    return [
        (item.item, True) for item in sorted(chapters, key=lambda i: extracted.refs[i.item].number)
    ]


def _chapterless_episodes(extracted: Extracted, store: MemoryStore) -> list[tuple[ItemId, bool]]:
    return [
        (item.item, True) for item in extracted.items if item.level == 1 and item.container is None
    ]


def _stable_facts(extracted: Extracted, store: MemoryStore) -> list[tuple[ItemId, bool]]:
    rows: list[tuple[str]] = store.connection.execute(
        "SELECT entity FROM stable ORDER BY times_met, first_key, entity"
    ).fetchall()
    return [(l3_item(entity), True) for (entity,) in rows]
