"""Drives a store through `absorb` for the retention tests: by frames, or one simulated day at a time with
the store measured right after each pass. The bound and coverage oracles read `dump()` here, never a
value the host reports about itself (pr-fg-b §8.2)."""

from __future__ import annotations

from collections.abc import Callable, Iterable, Iterator, Sequence
from dataclasses import dataclass

from memory_fixtures import seat_view
from mineworld_sdk.wire.contract import PerceivedEvent
from support import run

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.retention.absorb import AbsorbReport, absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve

DAY = 86_400


def retention(policy: str, registry: PolicyRegistry | None = None, **parameters: int) -> Retention:
    """A choice in the JSON form DL-a emits (QFGb-1), resolved against the installed policies."""
    choice = MemoryPolicyChoice.model_validate({"policy": policy, "parameters": parameters})
    return resolve(choice, registry or PolicyRegistry.installed())


def frames(facts: Sequence[PerceivedEvent], size: int) -> Iterator[Sequence[PerceivedEvent]]:
    for start in range(0, len(facts), size):
        yield facts[start : start + size]


def days(facts: Iterable[PerceivedEvent]) -> Iterator[list[PerceivedEvent]]:
    """Frames that each end at a first fact of a new day: `absorb` then ends each frame with a pass."""
    frame: list[PerceivedEvent] = []
    day: int | None = None
    for fact in facts:
        frame.append(fact)
        if day is not None and fact.at // DAY > day:
            yield frame
            frame = []
        day = fact.at // DAY
    if frame:
        yield frame


def feed(store: SeatView, batches: Iterable[Sequence[PerceivedEvent]], kept: Retention) -> None:
    """Every batch through `absorb`, in one event loop."""

    async def go() -> None:
        for batch in batches:
            await absorb(store, batch, batch[-1].id, kept)

    run(go(), timeout_s=600)


def built(facts: Sequence[PerceivedEvent], kept: Retention, size: int) -> bytes:
    with seat_view() as store:
        feed(store, frames(facts, size), kept)
        return store.dump()


@dataclass(frozen=True)
class Pass:
    day: int
    report: AbsorbReport
    dump: bytes
    cursor: str | None
    newest: str | None


def by_day(store: SeatView, facts: Iterable[PerceivedEvent], kept: Retention) -> Iterator[Pass]:
    """One frame per day, each ending at the day's first fact; yields the store right after the pass."""
    for frame in days(facts):
        report = run(absorb(store, frame, frame[-1].id, kept), timeout_s=120)
        if report.passes:
            yield Pass(frame[-1].at // DAY, report, store.dump(), store.cursor(), store.newest())


def at_days(
    store: SeatView,
    facts: Iterable[PerceivedEvent],
    kept: Retention,
    wanted: Callable[[int], bool],
) -> dict[int, Pass]:
    return {each.day: each for each in by_day(store, facts, kept) if wanted(each.day)}
