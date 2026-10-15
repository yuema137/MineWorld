"""The golden renders of pr-s28a C-2: what memory shows each seat, captured once from the schema-3 code.

Run by hand from the repository root (the command and the base commit are in `tests/golden/SOURCE.md`):

    uv run --locked python cognition/lm-controller/tests/capture_golden_renders.py [--cafe]

It writes `tests/golden/s28a_renders.json`. `test_home_golden.py` recomputes the same renders with the code under
test and compares them with that file, so the file is the oracle and this module only computes. Three arms, each
over seats opened one per store (P4's layout, the `none` arm of step-28):

- `ingest`: the synthetic 12-seat world (seed 7, 10 days, `home_streams.world`) through plain `ingest`, in day
  frames, rendered after every frame;
- `absorb`: the same world through `absorb` with the default `consolidate`, in day frames, rendered after every
  frame that ran a pass (with the bound-forced removals it made) and at the end;
- `fgc`: the FG-c cassette's stream (`retention_stream.stream(11, 12)`, Alice) replayed through `lm-summarise`,
  in day frames, rendered after every pass and at the end (AC-a-13).

At every point each seat renders the 200 queries `home_streams.queries` draws; the point keeps the sha256 of the
concatenated `MemorySection.render()` outputs. Three full texts are kept for a human to read. `--cafe` also
records the sum of the twelve plain-`ingest` stores' dump lengths on the real 12-seat stream (AC-a-9's baseline).
"""

from __future__ import annotations

import hashlib
import json
import sys
import tempfile
from collections.abc import Generator, Sequence
from contextlib import contextmanager
from pathlib import Path

from home_streams import CAST, SEATS, STRANGERS, cafe12, day_frames, queries, seat_streams, world
from memory_fixtures import INSTANCE, alice_identity
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, WorldInstanceId
from retention_stream import PEOPLE
from retention_stream import stream as single_stream
from support import run
from test_lm_summarise import GENEROUS, gateway, retention

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve
from mineworld_cognition.memory.store import MemoryStore
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
def opened(who: StoreIdentity) -> Generator[MemoryStore]:
    """One seat's memory, alone (the `none` arm)."""
    with MemoryStore.open(":memory:", who) as store:
        yield store


def sections(store: MemoryStore, seat: int, point: int, now: int) -> tuple[str, list[str]]:
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
    store: MemoryStore, facts: Sequence[PerceivedEvent], kept: Retention, seat: int
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


def fgc_arm() -> list[Point]:
    """The FG-c cassette's stream replayed (no model is reached: a `ReplayBackend`)."""
    replay = ReplayBackend(Cassette.load(CASSETTE))
    kept = retention(gateway(replay, **GENEROUS), calls_per_sim_day=24)
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


def cafe_bytes(scratch: Path) -> dict[str, int]:
    """AC-a-9's baseline: Σ of the twelve plain-`ingest` stores' dump lengths on the real stream."""
    real = cafe12(scratch)
    total = 0
    for seat, facts in real.streams.items():
        with opened(identity(seat, real.me[seat], real.instance)) as store:
            ingest(store, facts, facts[-1].id)
            total += len(store.dump())
    return {"seats": len(real.streams), "days": 2, "dump_bytes": total}


def compute() -> dict[str, object]:
    streams = seat_streams(world(SEED, DAYS))
    return {
        "ingest": ingest_arm(streams),
        "absorb": absorb_arm(streams),
        "fgc": fgc_arm(),
        "samples": samples(streams),
    }


def main(arguments: Sequence[str]) -> None:
    golden: dict[str, object] = {"seed": SEED, "days": DAYS, "people": [str(p) for p in PEOPLE]}
    golden.update(compute())
    if "--cafe" in arguments:
        with tempfile.TemporaryDirectory() as scratch:
            golden["cafe12_schema3"] = cafe_bytes(Path(scratch))
    elif GOLDEN.is_file():
        kept: object = json.loads(GOLDEN.read_text(encoding="utf-8")).get("cafe12_schema3")
        if kept is not None:
            golden["cafe12_schema3"] = kept
    GOLDEN.parent.mkdir(parents=True, exist_ok=True)
    GOLDEN.write_text(json.dumps(golden, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(sys.argv[1:])
