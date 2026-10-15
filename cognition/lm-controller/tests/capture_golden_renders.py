"""The golden renders of pr-s28a C-2: what memory shows each seat, captured once from the schema-3 code.

The file `tests/golden/s28a_renders.json` was written by this module at C1, on the schema-3 `MemoryStore`, with
the command and the base commit `tests/golden/SOURCE.md` records (`--cafe` then also measured AC-a-9's schema-3
baseline, which the file keeps). Since C4 this module runs the schema-4 code: `test_home_golden.py` recomputes
every render through it and compares them with the file, so the file is the oracle and this module only
computes. Three arms, each over seats with a home of their own (the `none` arm of step-28):

- `ingest`: the synthetic 12-seat world (seed 7, 10 days, `home_streams.world`) through plain `ingest`, in day
  frames, rendered after every frame;
- `absorb`: the same world through `absorb` with the default `consolidate`, in day frames, rendered after every
  frame that ran a pass (with the bound-forced removals it made) and at the end;
- `fgc`: the FG-c cassette's stream (`retention_stream.stream(11, 12)`, Alice) replayed through `lm-summarise`,
  in day frames, rendered after every pass and at the end (AC-a-13).

At every point each seat renders the 200 queries `home_streams.queries` draws; the point keeps the sha256 of the
concatenated `MemorySection.render()` outputs. Three full texts are kept for a human to read.
"""

from __future__ import annotations

import hashlib
from collections.abc import Generator, Sequence
from contextlib import contextmanager
from pathlib import Path

from home_streams import CAST, SEATS, STRANGERS, day_frames, queries, seat_streams, world
from memory_fixtures import INSTANCE, alice_identity, seat_view
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, WorldInstanceId
from retention_stream import stream as single_stream
from support import run

from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve
from mineworld_cognition.record import Cassette, ReplayBackend

GOLDEN = Path(__file__).resolve().parent / "golden" / "s28a_renders.json"
CASSETTE = Path(__file__).resolve().parent / "cassettes" / "lm-summarise.jsonl"
SEED = 7
DAYS = 10
FGC_SEED = 11
FGC_DAYS = 12
END = 999
"""The point index of the end of a run, after every pass."""
CAST_IDS: tuple[EntityId, ...] = (*CAST.values(), *STRANGERS)

type Point = dict[str, int | str]
type Arm = dict[str, list[Point]]


def identity(seat: EntityKey, me: EntityId, instance: WorldInstanceId = INSTANCE) -> StoreIdentity:
    return StoreIdentity(world_instance=instance, seat=seat, self_entity=me)


@contextmanager
def opened(who: StoreIdentity) -> Generator[SeatView]:
    """One seat's memory, alone in its home (the `none` arm)."""
    with seat_view(who) as store:
        yield store


def sections(store: SeatView, seat: int, point: int, now: int) -> tuple[str, list[str]]:
    """The sha256 of the seat's 200 renders at a point, and the renders themselves."""
    texts = [
        retrieve(store, RecallQuery(at=at, counterparts=who, words=words)).render()
        for at, who, words in queries(SEED, seat, point, now, CAST_IDS)
    ]
    return hashlib.sha256("".join(texts).encode("utf-8")).hexdigest(), texts


def consolidate() -> Retention:
    return resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed())


def ingest_arm(streams: dict[EntityKey, list[PerceivedEvent]]) -> Arm:
    arm: Arm = {}
    for index, seat in enumerate(SEATS):
        points: list[Point] = []
        with opened(identity(seat, CAST[seat])) as store:
            for number, frame in enumerate(day_frames(streams[seat])):
                ingest(store, frame, frame[-1].id)
                digest, _ = sections(store, index, number, frame[-1].at)
                points.append({"point": number, "through": frame[-1].id, "sections": digest})
        arm[seat] = points
    return arm


def absorbed(
    store: SeatView, facts: Sequence[PerceivedEvent], kept: Retention, seat: int
) -> list[Point]:
    """`facts` through `absorb` in day frames; a point after every frame that ran a pass, and the end."""
    points: list[Point] = []
    passes = 0
    for frame in day_frames(facts):
        report = run(absorb(store, frame, frame[-1].id, kept), timeout_s=600)
        if not report.passes:
            continue
        digest, _ = sections(store, seat, passes, frame[-1].at)
        points.append(
            {
                "point": passes,
                "trigger": report.passes[-1].trigger,
                "passes": len(report.passes),
                "forced": sum(each.forced for each in report.passes),
                "sections": digest,
            }
        )
        passes += 1
    digest, _ = sections(store, seat, END, facts[-1].at)
    points.append({"point": END, "trigger": "end", "passes": 0, "forced": 0, "sections": digest})
    return points


def absorb_arm(streams: dict[EntityKey, list[PerceivedEvent]]) -> Arm:
    arm: Arm = {}
    for index, seat in enumerate(SEATS):
        with opened(identity(seat, CAST[seat])) as store:
            arm[seat] = absorbed(store, streams[seat], consolidate(), index)
    return arm


class _Clock:
    def now(self) -> int:
        return 1_760_000_000


def fgc_arm() -> list[Point]:
    """The FG-c cassette's stream replayed (no model is reached: a `ReplayBackend`), with the gateway and the
    `lm-summarise` parameters `test_lm_summarise_replay.py` replays it with."""
    replay = ReplayBackend(Cassette.load(CASSETTE))
    policy = BudgetPolicy(calls_per_wall_hour=10_000_000, tokens_per_wall_day=1_000_000_000)
    gateway = ModelGateway(replay, Budget.of(policy, MemoryLedger(), _Clock()))
    choice = MemoryPolicyChoice.model_validate(
        {"policy": "lm-summarise", "parameters": {"calls_per_sim_day": 24}}
    )
    kept = resolve(choice, PolicyRegistry.installed(), gateway)
    with opened(alice_identity()) as store:
        return absorbed(store, list(single_stream(FGC_SEED, FGC_DAYS)), kept, 0)


def samples(streams: dict[EntityKey, list[PerceivedEvent]]) -> list[dict[str, str]]:
    """Three full renders for a human to read: Alice at the end of the `ingest` arm, queries 0 … 2."""
    seat = SEATS[0]
    frames = day_frames(streams[seat])
    with opened(identity(seat, CAST[seat])) as store:
        for frame in frames:
            ingest(store, frame, frame[-1].id)
        _, texts = sections(store, 0, len(frames) - 1, frames[-1][-1].at)
    return [{"seat": seat, "query": str(n), "text": texts[n]} for n in range(3)]


def compute() -> dict[str, object]:
    streams = seat_streams(world(SEED, DAYS))
    return {
        "ingest": ingest_arm(streams),
        "absorb": absorb_arm(streams),
        "fgc": fgc_arm(),
        "samples": samples(streams),
    }
