"""A policy's plan: validated whole (§5.7.1), then executed by the host alone (§5.7.2).

Validation refuses the whole plan by a typed reason; a refused plan is never partly applied. Execution
deletes rows of the seat's own view only, inside the pass's transaction, so every perceived fact stays
represented exactly once: by a surviving unit, or by the forgotten set (FGb-1, FGb-6). A record is removed by
releasing it (`SeatView.release`), which collects the shared record when no seat of the home holds it any more
(pr-s28a §5.6).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Literal

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import ItemId, event_key
from mineworld_cognition.memory.retention.forgotten import Forgetter, add
from mineworld_cognition.memory.retention.items import Extracted
from mineworld_cognition.memory.retention.policy import Drop, Keep, Merge

Reason = Literal[
    "unknown-item", "duplicate", "merge-mismatch", "drop-nested", "keep-under-removed", "malformed"
]


class PlanRejected(Exception):
    """The plan breaks a rule of §5.7.1; nothing of it is applied."""

    def __init__(self, reason: Reason, detail: str) -> None:
        super().__init__(f"{reason}: {detail}")
        self.reason: Reason = reason


@dataclass
class Resolved:
    """What a valid plan removes, directly named (nested items follow their container)."""

    into_episodes: list[ItemId] = field(default_factory=list[ItemId])
    into_chapters: list[ItemId] = field(default_factory=list[ItemId])
    dropped: list[ItemId] = field(default_factory=list[ItemId])


@dataclass(frozen=True)
class Outcome:
    kept: int
    merged: int
    dropped: int


def validate(plan: object, extracted: Extracted) -> Resolved:
    """The plan as removals, or `PlanRejected` with the first rule it breaks."""
    if not isinstance(plan, list | tuple):
        raise PlanRejected(
            "malformed", f"a plan is a sequence of decisions, not {type(plan).__name__}"
        )
    decisions: list[Keep | Merge | Drop] = []
    entries: list[object] = list(plan)  # pyright: ignore[reportUnknownArgumentType] - checked below
    for decision in entries:
        if not isinstance(decision, Keep | Merge | Drop):
            raise PlanRejected("malformed", f"{type(decision).__name__} is not a decision")
        decisions.append(decision)
    items = extracted.by_id
    named: set[ItemId] = set()
    for decision in decisions:
        for item in decision.items if isinstance(decision, Merge) else (decision.item,):
            if item not in items:
                raise PlanRejected("unknown-item", item)
            if item in named:
                raise PlanRejected("duplicate", item)
            named.add(item)
    resolved = Resolved()
    for decision in decisions:
        if isinstance(decision, Merge):
            levels = {items[item].level for item in decision.items}
            containers = {items[item].container for item in decision.items}
            if levels != {decision.into - 1} or len(containers) != 1 or None in containers:
                raise PlanRejected("merge-mismatch", ", ".join(decision.items))
            target = resolved.into_episodes if decision.into == 1 else resolved.into_chapters
            target.extend(decision.items)
        elif isinstance(decision, Drop):
            if items[decision.item].container is not None:
                raise PlanRejected("drop-nested", decision.item)
            resolved.dropped.append(decision.item)
    removed = {*resolved.into_episodes, *resolved.into_chapters, *resolved.dropped}

    def gone(item: ItemId | None) -> bool:
        while item is not None:
            if item in removed:
                return True
            item = items[item].container
        return False

    for decision in decisions:
        if isinstance(decision, Merge) and gone(items[decision.items[0]].container):
            raise PlanRejected("merge-mismatch", f"{decision.items[0]}'s container is removed")
        if isinstance(decision, Keep) and gone(items[decision.item].container):
            raise PlanRejected("keep-under-removed", decision.item)
    return resolved


def execute(
    store: SeatView, resolved: Resolved, extracted: Extracted, forgetter: Forgetter
) -> Outcome:
    """Deletes what the plan removes and forgets what it drops, each range signed by `forgetter` (the pass's
    day and policy, or a backstop step); counts every item kept, merged, dropped."""
    merged = dropped = 0
    for item in resolved.into_episodes:
        merged += remove(store, extracted, item, forget=None)
    for item in resolved.into_chapters:
        merged += remove(store, extracted, item, forget=None)
    for item in resolved.dropped:
        dropped += remove(store, extracted, item, forget=forgetter)
    return Outcome(len(extracted.items) - merged - dropped, merged, dropped)


def nested(extracted: Extracted, item: ItemId) -> list[ItemId]:
    """The item and everything nested in it, container first."""
    found = [item]
    for child in extracted.children.get(item, ()):
        found += nested(extracted, child)
    return found


def remove(store: SeatView, extracted: Extracted, item: ItemId, *, forget: Forgetter | None) -> int:
    """Deletes one item's rows and those of everything nested in it; with a `forget`ter, adds its citation to
    the forgotten set, signed by it (a stable fact cites and covers nothing, so it adds nothing). Returns the
    number of items removed. A record is released: its access row goes, and its shared record with it when no
    seat holds it any more."""
    db = store.connection
    seat = store.seat_id
    removed = nested(extracted, item)
    for each in removed:
        ref = extracted.refs[each]
        if ref.level == 0:
            store.release((ref.key,))
        elif ref.level == 1:
            db.execute("DELETE FROM episode_who WHERE seat = ? AND episode = ?", (seat, ref.number))
            db.execute("DELETE FROM episodes WHERE seat = ? AND episode = ?", (seat, ref.number))
        elif ref.level == 2:
            db.execute("DELETE FROM chapters WHERE seat = ? AND week = ?", (seat, ref.number))
        else:
            db.execute("DELETE FROM stable WHERE seat = ? AND entity = ?", (seat, ref.key))
    level = extracted.refs[item].level
    if forget is not None and level in (1, 2):
        ranges = extracted.by_id[item].citation.ranges
        add(store, [(event_key(r.first), event_key(r.last)) for r in ranges], forget)
    if level == 2:
        db.execute(
            "DELETE FROM chapter_ranges WHERE seat = ? AND week = ?",
            (seat, extracted.refs[item].number),
        )
    return len(removed)
