"""Milestone D after 100 simulated days (S10 P7, AP7-10; step-24 §8.6, R-P7-FG-1, D-P7-2).

Step-24 §8.6 gave Milestone D one more assertion: *after 100 simulated days Alice's store is bounded and her reply still
cites the 2D lines* — which forces those lines to be important enough to survive forgetting, or merged with their ids.
FG-b2 (operator, 2026-10-14) replaced the fixed 1 MiB of that sentence by the seat's `store_bound_mib`, 20 MiB by
default, so the bound asserted is the one Alice's retention resolves to, and the measured size is printed beside the
historical 1 MiB.

The run, against the real binary:

```text
mineworld server worlds/social-cafe --save DIR                                   the 2D exchange, live:
    Alice's seat runner (in-process, the scripted model of `milestone_d_cognition`) and a visitor client
mineworld run worlds/social-cafe --headless --seed 7 --days 100 --save DIR      the same world, on to day 100
mineworld perceived … --person alice --since <her store's cursor> --json        what she perceived since
absorb(her store, those facts, her resolved retention)                          R-P7-FG-1
```

Then the context a "Hello again." from the visitor would be decided from at day 100 is retrieved. Step-24 asks that
it still cite the 2D line or Alice's reply. Measured on 2026-10-15 it did not (F-P7-8, Q-P7-3): `consolidate` had
merged the day-1 records into their episode and the episode into its chapter. FG-d (pr-p6c-fgd-memory-coherence.md
§4.2) keeps each person's first exchanges as records, and P6c's `with` section (§4.1) shows the speaker's first words,
so the clause is now an ordinary assertion. The query is the one the seat builds for that consult: the speaker named.
"""

from __future__ import annotations

import asyncio
import json
import re
import time
from dataclasses import dataclass
from pathlib import Path

import pytest
from binary import mineworld
from milestone_d_cognition import remembers
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, WorldInstanceId
from realserver import INVITE, Server
from support import run
from test_controller_server import (  # pyright: ignore[reportPrivateUsage]
    _alice_id,  # pyright: ignore[reportPrivateUsage]
    _config,  # pyright: ignore[reportPrivateUsage]
    _heard,  # pyright: ignore[reportPrivateUsage]
    _say,  # pyright: ignore[reportPrivateUsage]
    _stop,  # pyright: ignore[reportPrivateUsage]
    _talk_available,  # pyright: ignore[reportPrivateUsage]
    _visitor,  # pyright: ignore[reportPrivateUsage]
    _wait_for,  # pyright: ignore[reportPrivateUsage]
    _walk_into_reach,  # pyright: ignore[reportPrivateUsage]
)

from mineworld_cognition.__main__ import serve
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import build_router
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.log import read_decisions
from mineworld_cognition.controller.memory_view import memory_view
from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import DEFAULT_CHOICE, resolve
from mineworld_cognition.memory.retrieve import RecallQuery

pytestmark = pytest.mark.real_binary

WORLD = "worlds/social-cafe"
ALICE = EntityKey("alice")
FROM_THE_2D_WINDOW = "Hi, I'm new in town. I left a red umbrella here yesterday."
FROM_THE_3D_WINDOW = "Hello again."
DAYS = 100
FRAME = 256
SETTLE_S = 2.0
"""How long the visitor stands at the counter before speaking, as in `milestone_d.rs`."""
HISTORICAL_BOUND = 1_048_576
"""Step-24 §8.6's figure, superseded by FG-b2's per-seat bound; printed for comparison only."""


def _export(save: Path, since: EventId | None) -> list[PerceivedEvent]:
    after = () if since is None else ("--since", str(since))
    out = mineworld("perceived", WORLD, "--save", save, "--person", "alice", *after, "--json")
    return [PerceivedEvent.model_validate_json(line) for line in out.splitlines() if line]


def _body(fact: PerceivedEvent) -> dict[str, object]:
    payload = fact.payload.payload
    assert isinstance(payload, dict)
    return dict(payload)


def _entity(reference: object) -> str:
    assert isinstance(reference, dict)
    return str(reference["entity"])  # pyright: ignore[reportUnknownArgumentType]


def _day_100(tmp_path: Path) -> Day100:
    save, store_dir = tmp_path / "save", tmp_path / "store"
    server = Server(save, town=False)
    try:

        async def the_2d_exchange() -> EntityId:
            config = _config()
            router = build_router(config, scripted=ScriptedBackend(remembers))
            task = asyncio.create_task(
                serve(
                    config,
                    [LmSeat(key=ALICE, memory=None)],
                    router=router,
                    invite=Invite(INVITE),
                    url=server.url,
                    store_dir=store_dir,
                )
            )
            try:
                await _wait_for(
                    lambda: (store_dir / "alice.resume").exists(), "the runner joined Alice"
                )
                visitor = await _visitor(server.url)
                async with visitor:
                    await _walk_into_reach(visitor)
                    observation = (await visitor.changed()).observation
                    alice, me = _alice_id(observation), observation.observer
                    await _talk_available(visitor, alice)
                    await asyncio.sleep(
                        SETTLE_S
                    )  # Alice's view of the walk can trail the visitor's (D-P6b-30)
                    await _say(visitor, alice, FROM_THE_2D_WINDOW)

                    # Each wait is for a frame newer than the last: `changed()` with no `since` answers at
                    # once, and a loop on it would never yield to the seat runner on this event loop.
                    deadline = time.monotonic() + 60
                    since = None
                    while True:
                        frame = await asyncio.wait_for(visitor.changed(since), 30)
                        since = frame.seq
                        if any(
                            speaker == str(alice) for speaker, _ in _heard(frame.observation, me)
                        ):
                            break
                        assert time.monotonic() < deadline, (
                            "Alice answered the 2D line; her decision log: "
                            + _log_summary(store_dir)
                        )
                    return me
            finally:
                await _stop(task)
                await router.aclose()

        visitor = run(the_2d_exchange(), timeout_s=150)
    finally:
        server.stop()

    early = _export(save, None)
    alice = next(
        _entity(_body(fact)["listener"])
        for fact in early
        if fact.event_type == "spoke" and _body(fact)["utterance"] == FROM_THE_2D_WINDOW
    )
    line = next(
        f for f in early if f.event_type == "spoke" and _body(f)["utterance"] == FROM_THE_2D_WINDOW
    )
    reply = next(
        f
        for f in early
        if f.event_type == "spoke"
        and _entity(_body(f)["speaker"]) == alice
        and _entity(_body(f)["listener"]) == str(visitor)
    )

    started = time.monotonic()
    mineworld("run", WORLD, "--headless", "--seed", "7", "--days", str(DAYS), "--save", save)
    ran_s = time.monotonic() - started
    found = re.search(r"instance ([0-9a-f]{32})", mineworld("inspect", save))
    assert found is not None

    retention = resolve(DEFAULT_CHOICE, PolicyRegistry.installed(), None)
    # The seat runner's memory: the process's home of this world instance (pr-s28f §5.4, `sharing = "home"`).
    instance = WorldInstanceId(found.group(1))
    home = MemoryHome.open(store_dir / instance / "memory.sqlite", instance)
    store = home.seat(
        StoreIdentity(world_instance=instance, seat=ALICE, self_entity=EntityId(alice))
    )
    try:
        later = _export(save, store.cursor())
        assert later, "Alice perceived something in 100 days"
        for start in range(0, len(later), FRAME):
            frame = later[start : start + FRAME]
            run(absorb(store, frame, frame[-1].id, retention), timeout_s=300)
        size = store.content_size()
        section = memory_view(
            store,
            RecallQuery(
                at=later[-1].at,
                counterparts=(visitor,),
                words=FROM_THE_3D_WINDOW,
                speaker=visitor,
            ),
            8,
        )
        cited = {
            int(event)
            for span in section.citations()
            for event in range(int(span.first), int(span.last) + 1)
        }
    finally:
        home.close()

    print(
        json.dumps(
            {
                "days": DAYS,
                "run_wall_s": round(ran_s, 1),
                "facts_absorbed": len(later),
                "store_content_bytes": size,
                "bound_bytes": retention.bounds.bound,
                "historical_1_MiB_held": size <= HISTORICAL_BOUND,
                "line_2d": str(line.id),
                "reply_2d": str(reply.id),
                "line_cited": int(line.id) in cited,
                "reply_cited": int(reply.id) in cited,
            }
        )
    )
    return Day100(
        size, retention.bounds.bound, int(line.id), int(reply.id), cited, section.render()
    )


@dataclass(frozen=True)
class Day100:
    size: int
    bound: int
    line: int
    reply: int
    cited: set[int]
    rendered: str


@pytest.fixture(scope="module")
def day_100(tmp_path_factory: pytest.TempPathFactory) -> Day100:
    return _day_100(tmp_path_factory.mktemp("milestone-d-100-days"))


def test_after_100_days_alices_store_is_within_her_bound(day_100: Day100) -> None:
    assert day_100.size <= day_100.bound, f"Alice's store is within her bound: {day_100.size} B"


def test_after_100_days_her_context_still_cites_the_2d_lines(day_100: Day100) -> None:
    # AFGd-3 (pr-p6c-fgd §6), formerly F-P7-8's strict xfail. Its mutation sets consolidate's `keepsakes`
    # default to 0: the day-1 records are merged away and the context cites neither line.
    assert day_100.line in day_100.cited or day_100.reply in day_100.cited, (
        f"at day {DAYS} the context for {FROM_THE_3D_WINDOW!r} still cites the 2D line #{day_100.line} or "
        f"Alice's reply #{day_100.reply}; it cites {len(day_100.cited)} ids: {day_100.rendered}"
    )


def _log_summary(store_dir: Path) -> str:
    """Each consult's trigger, talk targets and outcome, for a failure message."""
    path = store_dir / "alice.decisions.jsonl"
    if not path.exists():
        return "(none)"
    return "; ".join(
        f"{record.input.trigger.kind} #{record.input.trigger.event} to={record.input.situation.talk_to} "
        f"-> {record.record.outcome}"
        for record in read_decisions(path)
    )
