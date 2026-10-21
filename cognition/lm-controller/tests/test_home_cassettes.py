"""AS28-10: cassettes recorded without a shared home replay under one (pr-s28f §§7, 8; AC-f-2, AC-f-3, AC-f-4,
AC-f-7).

(a) The committed FG-c cassette replays for its seat inside a home that two peer seats share, interleaved frame by
frame, to the frozen digest. (b) Two seat runners of one process on the scripted server (not the default town),
recorded under `sharing = "none"`, replay under `"home"` with no `CassetteMiss` and the same decision and pass lines.
No expected value comes from the code under test: (a) uses the committed cassette and digest, (b) compares two runs
of the shipped code that differ only in `sharing`. No model is reached.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Any

from memory_fixtures import INSTANCE, alice_identity
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import frames
from retention_stream import PEOPLE, stream
from scripted_seat import (
    INVITE,
    FakeClock,
    Link,
    ScriptedServer,
    observation,
    perceived,
    spoke,
    until,
    welcome,
)
from support import completion, run
from test_controller_seat import ScriptedAnswers
from test_lm_summarise import GENEROUS, gateway, retention
from test_lm_summarise_replay import CASSETTE, DAYS, DIGEST, SEED

from mineworld_cognition.backend.model import Completion, CompletionRequest, ModelBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.config import CognitionConfig
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.homes import SeatHomes
from mineworld_cognition.controller.seat import SeatRunner
from mineworld_cognition.gateway import Budget, ModelGateway, Router
from mineworld_cognition.memory.config import Sharing
from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, resolve
from mineworld_cognition.record import Cassette, RecordingBackend, ReplayBackend

SEATS = {EntityKey("alice"): "101", EntityKey("wanderer"): "102"}
SPEAKER = "5"
DAY = 86_400
INSTANCE_DIR = str(json.loads(welcome())["world"]["instance"])
"""The scripted server's world instance: the directory its homes are in."""


# ── (a) the committed cassette, its seat sharing a home with two peers ───────────────────────────


def test_the_committed_cassette_replays_to_its_digest_inside_a_shared_home() -> None:
    facts = list(stream(SEED, DAYS))  # the cassette's stream (`test_lm_summarise_replay._facts`)
    alice = retention(
        gateway(ReplayBackend(Cassette.load(CASSETTE)), **GENEROUS), calls_per_sim_day=24
    )
    peers = [
        (StoreIdentity(world_instance=INSTANCE, seat=EntityKey(key), self_entity=PEOPLE[n]), n + 1)
        for n, key in ((1, "bob"), (2, "carol"))
    ]

    async def go() -> bytes:
        with MemoryHome.open(":memory:", INSTANCE) as home:
            views = [home.seat(alice_identity())] + [home.seat(identity) for identity, _ in peers]
            kept = resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed())
            for start, frame in zip(range(0, len(facts), 256), frames(facts, 256), strict=True):
                await absorb(views[0], frame, frame[-1].id, alice)
                for view, (_, every) in zip(views[1:], peers, strict=True):
                    theirs = [fact for index, fact in enumerate(frame, start) if index % every == 0]
                    if theirs:
                        await absorb(view, theirs, theirs[-1].id, kept)
            return views[0].dump()

    dump = run(go(), timeout_s=600)  # a miss raises `CassetteMiss`
    assert hashlib.sha256(dump).hexdigest() == DIGEST


# ── (b) two runners of one process, recorded under `none`, replayed under `home` ────────────────


def _config(sharing: Sharing) -> CognitionConfig:
    return CognitionConfig.model_validate(
        {
            "recording": {"mode": "scripted"},
            "tiers": {"social": "local"},
            "backends": {
                "local": {
                    "kind": "openai-compatible",
                    "base_url": "http://127.0.0.1:11434/v1",
                    "model": "fake-model",
                    "structured_output": "none",
                }
            },
            "controller": {"handback_s": 10, "consult_deadline_s": 20},
            "memory": {"sharing": sharing},
        }
    )


def _welcome(seat: EntityKey) -> str:
    frame: dict[str, Any] = json.loads(welcome(resume=1, observer=SEATS[seat]))
    frame["seat"] = str(seat)
    return json.dumps(frame)


def _observation(seat: EntityKey) -> str:
    frame: dict[str, Any] = json.loads(observation(1))
    frame["observation"]["observer"] = SEATS[seat]
    return json.dumps(frame)


def _shared(fact: dict[str, Any]) -> dict[str, Any]:
    """A line both seats perceive: the same envelope, both observers among its participants."""
    fact["participants"] = [SPEAKER, *SEATS.values()]
    return fact


FRAMES = (
    perceived("2", _shared(spoke("2", speaker=SPEAKER, listener="101", at=40))),
    perceived(
        "3", _shared(spoke("3", speaker=SPEAKER, listener="102", utterance="and you", at=80))
    ),
    perceived(
        "5",
        _shared(spoke("4", speaker=SPEAKER, listener="101", utterance="morning", at=DAY + 40)),
        _shared(spoke("5", speaker=SPEAKER, listener="102", utterance="hello", at=DAY + 50)),
    ),
)
"""Three frames; the third on the next simulated day, so each seat's `absorb` runs a retention pass."""


def _nothing(request: CompletionRequest) -> Completion:
    return completion(
        json.dumps({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
    )


def _lines(store: Path, seat: EntityKey) -> list[dict[str, Any]]:
    path = store / f"{seat}.decisions.jsonl"
    if not path.exists():
        return []
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def _kinds(store: Path, seat: EntityKey, kind: str) -> list[dict[str, Any]]:
    return [line for line in _lines(store, seat) if line["kind"] == kind]


class _Wall:
    def now(self) -> int:
        return 1_700_000_000


def _play(store: Path, sharing: Sharing, backend: ModelBackend) -> None:
    config = _config(sharing)
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), _Wall())
    router = Router({"social": ModelGateway(backend, budget)}, budget)
    homes = SeatHomes(store, config.memory)
    links = {seat: Link(_welcome(seat), _observation(seat)) for seat in SEATS}

    async def scenario() -> None:
        clock = FakeClock()
        tasks = [
            asyncio.create_task(
                SeatRunner(
                    config,
                    LmSeat(key=seat, memory=None),
                    router=router,
                    connector=ScriptedServer(link),
                    invite=Invite(INVITE),
                    nickname="cognition",
                    store_dir=store,
                    homes=homes,
                    clock=clock.time,
                    sleep=clock.sleep,
                ).run()
            )
            for seat, link in links.items()
        ]
        for link in links.values():
            await until(lambda link=link: "join" in link.kinds(), "the join")
        expected = (
            {"alice": 1, "wanderer": 0},
            {"alice": 1, "wanderer": 1},
            {"alice": 2, "wanderer": 2},
        )
        for frame, counts in zip(FRAMES, expected, strict=True):
            for link in links.values():
                link.say(frame)
            for seat, count in counts.items():
                await until(
                    lambda seat=seat, count=count: (
                        len(_kinds(store, EntityKey(seat), "decision")) >= count
                    ),
                    f"{count} consults of {seat}",
                    rounds=20_000,
                )
        for task in tasks:
            task.cancel()
        await asyncio.gather(*tasks, return_exceptions=True)
        homes.close()

    run(scenario(), timeout_s=60)


def test_a_two_seat_run_recorded_under_none_replays_under_home_with_the_same_logs(
    tmp_path: Path,
) -> None:
    cassette = tmp_path / "two-seats.jsonl"
    recorder = RecordingBackend(
        ScriptedAnswers(_nothing), cassette, binding="local", model="fake-model"
    )
    _play(tmp_path / "none", "none", recorder)
    run(recorder.aclose())
    _play(tmp_path / "home", "home", ReplayBackend(Cassette.load(cassette)))  # a miss raises

    for seat in SEATS:
        for kind in ("decision", "pass"):
            recorded = _kinds(tmp_path / "none", seat, kind)
            assert recorded, f"{seat} wrote {kind} lines"
            assert _kinds(tmp_path / "home", seat, kind) == recorded, f"{seat}'s {kind} lines"
        (line,) = _kinds(tmp_path / "home", seat, "pass")
        assert (line["seat"], line["day"]) == (str(seat), 1)

    # AC-f-2: the layout.
    for sharing, memory_files in (
        ("home", ["memory.sqlite"]),
        ("none", ["alice.sqlite", "wanderer.sqlite"]),
    ):
        store = tmp_path / sharing
        assert sorted(path.name for path in (store / INSTANCE_DIR).glob("*.sqlite")) == memory_files
        assert not list(store.glob("*.sqlite")), "no memory file of the earlier layout"
        for seat in SEATS:
            assert (store / f"{seat}.decisions.jsonl").is_file()
    connection = sqlite3.connect(tmp_path / "home" / INSTANCE_DIR / "memory.sqlite")
    try:
        keys = sorted(row[0] for row in connection.execute("SELECT key FROM seats"))
    finally:
        connection.close()
    assert keys == ["alice", "wanderer"]
