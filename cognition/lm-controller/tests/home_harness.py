"""The transparency harness's two arms (pr-s28a §8 AC-a-2 … AC-a-4, AC-a-8; step-28 I-S28-2, I-S28-3).

- **`none`**: every seat alone in a home of its own (P4's one store per seat), its frames fed in order.
- **`home`**: every seat a view of one shared home, the seats' frames interleaved in a seeded order
  (`home_streams.interleave`).

Both arms run the seat's frames through `absorb`, and snapshot each seat right after every one of its retention
passes (the pass `absorb` runs is wrapped, in this process only) and at the end: the sha256 and length of its
`SeatView.dump()`, and the sha256 of the 200 `MemorySection.render()` outputs of `home_streams.queries` at that
point. The oracle of every comparison is the `none` arm, computed by the test; nothing the home reports about
itself is asserted.
"""

from __future__ import annotations

import hashlib
from collections.abc import Callable, Generator, Mapping, Sequence
from contextlib import contextmanager
from dataclasses import dataclass, field
from functools import cache

import pytest
from home_streams import CAST, STRANGERS, interleave, queries, seat_streams, world
from memory_fixtures import INSTANCE
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, WorldInstanceId
from retention_driver import DAY, retention
from support import run

from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention import absorb as absorb_module
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.host import PassCause, PassReport, run_pass
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve

END = 999
"""The point index of a seat's end, after its last frame."""
QUERY_SEED = 7

type Streams = Mapping[EntityKey, Sequence[PerceivedEvent]]
type Policies = Callable[[EntityKey], Retention]


@dataclass(frozen=True)
class Snapshot:
    point: int
    now: int
    dump: str
    """sha256 of the seat view's dump."""
    size: int
    sections: str
    """sha256 of the 200 renders; `""` when sections were not asked for."""


@dataclass(frozen=True)
class Lengths:
    """After one of a seat's frames, in the `none` arm: its one-seat home's dump, its view's dump, and the
    records it holds (AC-a-8)."""

    home: int
    view: int
    held: int


@dataclass
class Arm:
    snapshots: dict[EntityKey, list[Snapshot]] = field(
        default_factory=dict[EntityKey, list[Snapshot]]
    )
    lengths: dict[EntityKey, list[Lengths]] = field(default_factory=dict[EntityKey, list[Lengths]])
    home: str = ""
    """The `home` arm's final `MemoryHome.dump()`, as its sha256."""


@dataclass(frozen=True)
class Cast:
    """Who the seats are: their entities, the world instance, and the people queries may name."""

    me: Mapping[EntityKey, EntityId]
    instance: WorldInstanceId
    people: Sequence[EntityId]

    def identity(self, seat: EntityKey) -> StoreIdentity:
        return StoreIdentity(world_instance=self.instance, seat=seat, self_entity=self.me[seat])

    def index(self, seat: EntityKey) -> int:
        return sorted(self.me).index(seat)


def sections(view: SeatView, seat: int, point: int, now: int, people: Sequence[EntityId]) -> str:
    texts = [
        retrieve(view, RecallQuery(at=at, counterparts=who, words=words)).render()
        for at, who, words in queries(QUERY_SEED, seat, point, now, people)
    ]
    return hashlib.sha256("".join(texts).encode("utf-8")).hexdigest()


class _Recorder:
    def __init__(self, cast: Cast, arm: Arm, asked: frozenset[EntityKey]) -> None:
        self.cast = cast
        self.arm = arm
        self.asked = asked

    def snap(self, view: SeatView, now: int, point: int | None = None) -> None:
        seat = view.identity.seat
        taken = self.arm.snapshots.setdefault(seat, [])
        number = len(taken) if point is None else point
        dump = view.dump()
        digest = (
            sections(view, self.cast.index(seat), number, now, self.cast.people)
            if seat in self.asked
            else ""
        )
        taken.append(Snapshot(number, now, hashlib.sha256(dump).hexdigest(), len(dump), digest))


@contextmanager
def _recording(recorder: _Recorder) -> Generator[None]:
    """Wraps the pass `absorb` runs, so every seat is snapshot right after each of its passes."""

    async def recorded(
        store: SeatView, kept: Retention, now: int, cause: PassCause = "day"
    ) -> PassReport:
        report = await run_pass(store, kept, now, cause)
        recorder.snap(store, now)
        return report

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(absorb_module, "run_pass", recorded)
        yield


def _frames(facts: Sequence[PerceivedEvent], frame: int) -> list[Sequence[PerceivedEvent]]:
    return [facts[start : start + frame] for start in range(0, len(facts), frame)]


def none_arm(
    streams: Streams,
    frame: int,
    cast: Cast,
    policies: Policies,
    *,
    asked: frozenset[EntityKey] | None = None,
    lengths: bool = False,
) -> Arm:
    """Every seat alone in its own in-memory home. With `lengths`, each one-seat home's dump, its view's dump
    and its held records are measured after every frame (AC-a-8)."""
    arm = Arm()
    recorder = _Recorder(cast, arm, frozenset(streams) if asked is None else asked)
    with _recording(recorder):
        for seat in sorted(streams):
            facts = streams[seat]
            kept = policies(seat)
            with MemoryHome.open(":memory:", cast.instance) as home:
                view = home.seat(cast.identity(seat))
                for batch in _frames(facts, frame):
                    run(absorb(view, batch, batch[-1].id, kept), timeout_s=600)
                    if lengths:
                        held = len(view.row_bytes("l0"))
                        measured = Lengths(len(home.dump()), len(view.dump()), held)
                        arm.lengths.setdefault(seat, []).append(measured)
                recorder.snap(view, facts[-1].at, END)
    return arm


def home_arm(
    streams: Streams,
    frame: int,
    seed: int,
    cast: Cast,
    policies: Policies,
    *,
    asked: frozenset[EntityKey] | None = None,
    after_frame: Callable[[MemoryHome, Mapping[EntityKey, int]], None] | None = None,
) -> Arm:
    """Every seat a view of one shared in-memory home, their frames interleaved by `seed`. A seat joins the
    home at its first frame, so the home's internal seat numbers follow the interleaving (and must never reach
    a dump, I-S28-5). `after_frame` sees the home and how many frames each seat has absorbed, after every
    frame (AC-a-8 (a))."""
    arm = Arm()
    recorder = _Recorder(cast, arm, frozenset(streams) if asked is None else asked)
    done = dict.fromkeys(streams, 0)
    with _recording(recorder), MemoryHome.open(":memory:", cast.instance) as home:
        views: dict[EntityKey, SeatView] = {}
        kept = {seat: policies(seat) for seat in streams}
        for seat, batch in interleave({s: list(f) for s, f in streams.items()}, frame, seed):
            view = views.get(seat)
            if view is None:
                view = views[seat] = home.seat(cast.identity(seat))
            run(absorb(view, batch, batch[-1].id, kept[seat]), timeout_s=600)
            done[seat] += 1
            if after_frame is not None:
                after_frame(home, done)
        for seat in sorted(streams):
            recorder.snap(views[seat], streams[seat][-1].at, END)
        arm.home = hashlib.sha256(home.dump()).hexdigest()
    return arm


# ── The synthetic world of AC-a-2 … AC-a-4: seed 7, 10 days, 12 seats ────────────────────────────────────

SEED = 7
DAYS = 10
SYNTHETIC = Cast(CAST, INSTANCE, (*CAST.values(), *STRANGERS))


@cache
def synthetic(frame: int) -> dict[EntityKey, list[PerceivedEvent]]:
    """Each seat's stream; for frames of 1, the first two days only (R-a-2)."""
    streams = seat_streams(world(SEED, DAYS))
    if frame == 1:
        return {seat: [f for f in facts if f.at < 2 * DAY] for seat, facts in streams.items()}
    return streams


def consolidate(_seat: EntityKey) -> Retention:
    return retention("consolidate")


@cache
def alone(frame: int) -> Arm:
    """The `none` arm of the synthetic world under `consolidate`, once per frame size and session file."""
    return none_arm(synthetic(frame), frame, SYNTHETIC, consolidate)
