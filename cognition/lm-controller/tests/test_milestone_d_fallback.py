"""The 100-day fallback bound (S10 P7, R-P7-P6-2; Q-P7-1 as ruled by the primary session on 2026-10-16).

Ruling: *"the 100-day bound run from the scripted deterministic model on Alice's seat — costs nothing and is
reproducible."* A hosted run cannot carry it: the in-server controllers are paced in wall seconds (QTW-13), so a
`--time-scale`d town starves (measured 2026-10-15: at one simulated day per wall second, 11 consults in 100 days).
The run is therefore Alice's real 100-day history, replayed through her consult engine:

```text
mineworld run worlds/social-cafe --headless --seed 7 --days 100 --save DIR       the town's 100 days (seeded)
mineworld perceived … --person alice --json                                      what Alice perceived
for every line said to Alice (one in STRIDE, ~1 000 consults):
    absorb her history up to that line into her store (her resolved retention)
    consult: her situation (the speaker in reach), her memory view, the line as trigger,
             the model `remembers` behind seeded garbage (5 % transport failures, 30 % garbage per attempt),
             the world accepting what she says
    the decision record appended to her decision log
```

Every input is seeded or a pure function of the request, so the log is reproducible. Its counters (P6 §5.12) give the
bound: `fallbacks / consults ≤ 5 %` (ruled, QP6-7), and every consult resolves. What this does not measure is the
live path's own refusals (a person walking away mid-consult): AP6-3 and AP6-16 cover those over the real server.
"""

from __future__ import annotations

import hashlib
import json
import random
import re
import time
from collections import Counter
from pathlib import Path
from typing import Any

import pytest
from binary import mineworld
from garbage import GARBAGE
from milestone_d_cognition import remembers
from mineworld_sdk.session import Answered
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation, PerceivedEvent
from mineworld_sdk.wire.ids import ActionId, EntityId, EntityKey, WorldInstanceId
from support import run

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest, Usage
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.config import BUNDLED_RETRY, Layer
from mineworld_cognition.controller.consult import (
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    LayerPolicies,
    NewestState,
    consult,
)
from mineworld_cognition.controller.log import (
    DecisionInput,
    DecisionLog,
    DecisionRecord,
    counters_from,
    read_decisions,
)
from mineworld_cognition.controller.memory_view import memory_view
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.gateway import Budget, GateOutcome, ModelGateway
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import DEFAULT_CHOICE, resolve
from mineworld_cognition.memory.retrieve import RecallQuery

pytestmark = pytest.mark.real_binary

WORLD = "worlds/social-cafe"
ALICE = EntityKey("alice")
DAYS = 100
CONSULTS = 1_000
"""About this many consults: one line in every `len(lines) // CONSULTS`."""
SEED = 7
FAILURE_PERMILLE = 50
GARBAGE_PERMILLE = 300
BOUND_PERMILLE = 50
"""≤ 5 % of consults fall back (ruled, QP6-7)."""
FRAME = 256


ASKED: Counter[str] = Counter()
"""How often each request was asked: the draw's third input, so a retry draws afresh (below)."""


def deterministic(request: CompletionRequest) -> Completion | BackendFailure:
    """`remembers` behind seeded transport failures and garbage, drawn by (seed, request, how often that request was
    asked before). The run is reproducible, and each attempt is independent, as a real transport failure is: a draw
    by the request alone would fail a transport retry, which re-sends the same request, every time (it would bound
    the fallback share from below by the failure rate, which AP6-3's live scenario accepts and this bound must not)."""
    key = str(cassette_key(request))
    occurrence = ASKED[key]
    ASKED[key] += 1
    digest = hashlib.sha256(f"{SEED}:{key}:{occurrence}".encode()).digest()
    rng = random.Random(int.from_bytes(digest[:8], "big"))
    roll = rng.randrange(1000)
    if roll < FAILURE_PERMILLE:
        return BackendFailure(reason="unreachable")
    if roll < FAILURE_PERMILLE + GARBAGE_PERMILLE:
        text, finish = GARBAGE[rng.randrange(len(GARBAGE))]
        return Completion(
            text=text, finish=finish, usage=Usage(input_tokens=10, output_tokens=10, estimated=True)
        )
    return remembers(request)


class _Still:
    def now(self) -> int:
        return 0


def _policies() -> LayerPolicies:
    def bind(layer: Layer) -> BoundRetry:
        default = BUNDLED_RETRY[layer]
        return RetryRegistry.installed().bind(default.policy, dict(default.parameters), layer)

    return LayerPolicies(
        transport=bind("transport"), content=bind("content"), semantic=bind("semantic")
    )


def _body(fact: PerceivedEvent) -> dict[str, Any]:
    payload = fact.payload.payload
    assert isinstance(payload, dict)
    return dict(payload)


def _person(reference: object) -> EntityId:
    assert isinstance(reference, dict)
    return EntityId(str(reference["entity"]))  # pyright: ignore[reportUnknownArgumentType]


def _observation(me: EntityId, speaker: EntityId, fact: PerceivedEvent) -> Observation:
    """Alice's view at the line: she and the speaker at the line's place, `talk` to the speaker offered."""
    place = None if fact.place is None else {"entity": fact.place.entity, "entity_type": "place"}
    where = None if place is None else {"place": place, "local": None, "facing": None}

    def person(entity: EntityId) -> dict[str, Any]:
        return {
            "id": entity,
            "entity_type": "person",
            "location": where,
            "tags": [],
            "components": [],
        }

    return Observation.model_validate(
        {
            "observer": me,
            "at": fact.at,
            "self_location": where,
            "entities": [person(me), person(speaker)],
            "relations": [],
            "events": [],
            "affordances": [
                {
                    "action_type": "talk",
                    "target": speaker,
                    "available": True,
                    "unavailable_reason": None,
                    "requirement": {
                        "place": "any",
                        "within_range": None,
                        "requires_line_of_access": False,
                        "requires_target_available": True,
                    },
                }
            ],
        }
    )


async def _one(
    number: int,
    fact: PerceivedEvent,
    me: EntityId,
    store: SeatView,
    gateway: ModelGateway,
    policies: LayerPolicies,
) -> DecisionRecord:
    """One consult on one line said to Alice, as her seat runner makes it, against an accepting world."""
    speaker = _person(_body(fact)["speaker"])
    observation = _observation(me, speaker, fact)
    situation = Situation.of(observation, NameBook.load(store))
    trigger = Trigger(
        event=fact.id,
        kind="spoken_to",
        speaker=speaker,
        words=str(_body(fact)["utterance"]),
    )
    memory = memory_view(
        store, RecallQuery(at=fact.at, counterparts=(speaker,), words=trigger.words), 8
    )
    answers: list[str] = []

    async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
        return await gateway.complete(ALICE, request, timeout_s=timeout)

    async def submit(request: ActionRequest) -> Answered:
        answers.append("accepted")
        return Answered(
            action_id=ActionId(str(number)),
            result=Accepted.model_validate({"accepted": {"events": []}}),
        )

    async def no_sleep(seconds: float) -> None:
        return None

    record = await consult(
        ConsultInput(
            seat=ALICE,
            persona=Persona(),
            situation=situation,
            memory=memory,
            trigger=trigger,
        ),
        ConsultDeps(
            ask=ask,
            newest=lambda: NewestState(situation=situation, observation=observation),
            current=lambda: True,
            submit=submit,
            clock=lambda: 0.0,
            sleep=no_sleep,
            policies=policies,
            limits=ConsultLimits(deadline_s=20.0, max_calls=5),
        ),
    )
    return DecisionRecord(
        seat=str(ALICE),
        consult=number,
        input=DecisionInput(
            persona=Persona(),
            situation=situation,
            memory=memory,
            trigger=trigger,
            observation=observation,
        ),
        record=record,
        answers=tuple(answers),
    )


def test_over_100_simulated_days_at_most_5_percent_of_alices_consults_fall_back(
    tmp_path: Path,
) -> None:
    started = time.monotonic()
    save = tmp_path / "save"
    mineworld("run", WORLD, "--headless", "--seed", str(SEED), "--days", str(DAYS), "--save", save)
    found = re.search(r"instance ([0-9a-f]{32})", mineworld("inspect", save))
    assert found is not None
    out = mineworld("perceived", WORLD, "--save", save, "--person", "alice", "--json")
    facts = [PerceivedEvent.model_validate_json(line) for line in out.splitlines() if line]
    named = [f for f in facts if f.event_type == "named" and _body(f).get("name") == "Alice Moreau"]
    me = _person(_body(named[0])["person"])
    lines = [
        index
        for index, fact in enumerate(facts)
        if fact.event_type == "spoke" and _person(_body(fact)["listener"]) == me
    ]
    stride = max(1, len(lines) // CONSULTS)
    chosen = lines[::stride]

    retention = resolve(DEFAULT_CHOICE, PolicyRegistry.installed(), None)
    instance = WorldInstanceId(found.group(1))
    home = MemoryHome.open(tmp_path / "alice.sqlite", instance)
    store = home.seat(StoreIdentity(world_instance=instance, seat=ALICE, self_entity=me))
    log_path = tmp_path / "alice.decisions.jsonl"
    log = DecisionLog(log_path, rotate_bytes=1 << 40)  # one file: the whole run is read back
    budget = Budget.of(
        BudgetPolicy(calls_per_wall_hour=1_000_000, tokens_per_wall_day=1_000_000_000),
        MemoryLedger(),
        _Still(),
    )
    gateway = ModelGateway(ScriptedBackend(deterministic), budget)
    policies = _policies()

    async def replay() -> None:
        absorbed = 0
        for number, index in enumerate(chosen, start=1):
            while absorbed <= index:
                frame = facts[absorbed : min(index + 1, absorbed + FRAME)]
                await absorb(store, frame, frame[-1].id, retention)
                absorbed += len(frame)
            log.append(await _one(number, facts[index], me, store, gateway, policies))

    try:
        run(replay(), timeout_s=900)
    finally:
        home.close()
        run(gateway.aclose())

    records = read_decisions(log_path)
    counters = counters_from(records)
    outcomes = Counter(record.record.outcome for record in records)
    causes = Counter(
        str(record.record.cause) for record in records if record.record.outcome == "fell_back"
    )
    print(
        json.dumps(
            {
                "days": DAYS,
                "wall_s": round(time.monotonic() - started, 1),
                "lines_to_alice": len(lines),
                "stride": stride,
                "consults": counters.consults,
                "acted": counters.acted,
                "chose_nothing": counters.chose_nothing,
                "fallbacks": counters.fallbacks,
                "fallback_permille": 1000 * counters.fallbacks // max(1, counters.consults),
                "fell_back_by_cause": dict(causes),
                "invalid_answers": counters.invalid_answers,
                "retries_content": counters.retries_content,
                "retries_transport": counters.retries_transport,
                "model_calls": counters.model_calls,
            }
        )
    )
    assert counters.consults == len(chosen) >= 500, "the run is not vacuous"
    assert counters.invalid_answers > 0 and counters.retries_transport > 0, (
        "garbage and failures were drawn"
    )
    assert counters.acted > counters.consults // 2, (
        "the model mostly answered, so the bound is not of silence"
    )
    assert sum(outcomes.values()) == counters.consults, "every consult resolved to one outcome"
    assert 1000 * counters.fallbacks <= BOUND_PERMILLE * counters.consults, (
        f"{counters.fallbacks} of {counters.consults} consults fell back: {dict(causes)}"
    )
