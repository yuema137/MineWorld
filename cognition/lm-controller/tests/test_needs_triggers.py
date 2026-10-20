"""The LM seat notices its needs and may choose to sleep (S35 N-e, AC-Ne-6, Q-Ne-1 ruled yes).

A scripted sequence of observations stands for the backend: the recorded `commute-late` observation (awake), then
edited copies of it. The `feeling` trigger fires once per change of a need word and once on waking, and nowhere
else. A scripted answer `{"act": "sleep", "minutes": 480}` becomes the `sleep { for: 480 }` request the SDK builds.
A replay of the same script gives identical requests. The answers are text in this file, not a recorded provider
cassette: the scripted backend is the only backend the core tests use (ENGINEERING_STANDARDS, LM-independence).
"""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

from memory_fixtures import open_view
from mineworld_sdk import offers
from mineworld_sdk.wire.contract import ActionRequest, Observation
from mineworld_sdk.wire.ids import ActionTypeId, EntityKey, EventId

from mineworld_cognition.backend.model import Completion, Usage
from mineworld_cognition.controller.choice import (
    SOCIAL_CHOICE_SCHEMA,
    ContentFailure,
    SocialChoice,
    choice_schema,
    parse_choice,
)
from mineworld_cognition.controller.intent import build
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger, build_request
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.controller.triggers import Felt, feeling_due, felt_of
from mineworld_cognition.controller.validate import Sleep, validate
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection

NEEDS = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "observations" / "needs"
SEAT = EntityKey("seat")
SLEEP = ActionTypeId("sleep")
ANSWER = (
    '{"act": "sleep", "to": null, "words": "", "offer": null, "recalls": [],'
    ' "place": null, "amount": null, "minutes": 480}'
)


def recorded() -> dict[str, Any]:
    value: dict[str, Any] = json.loads((NEEDS / "commute-late.json").read_text(encoding="utf-8"))
    return value


def mine(raw: dict[str, Any]) -> dict[str, Any]:
    return next(e for e in raw["entities"] if e["id"] == raw["observer"])


def with_word(raw: dict[str, Any], need: str, word: str) -> dict[str, Any]:
    edited = copy.deepcopy(raw)
    record = next(c for c in mine(edited)["components"] if c["component_type"] == "needs")
    record["payload"]["words"][need] = word
    return edited


def asleep(raw: dict[str, Any]) -> dict[str, Any]:
    edited = copy.deepcopy(raw)
    mine(edited)["components"].append(
        {
            "entity": raw["observer"],
            "component_type": "asleep",
            "schema_version": 1,
            "payload": {"asleep": True, "since": 43_200},
        }
    )
    return edited


def awake(raw: dict[str, Any]) -> dict[str, Any]:
    return copy.deepcopy(raw)


def situation_of(raw: dict[str, Any]) -> Situation:
    return Situation.of(Observation.model_validate(raw), NameBook.load(open_view()))


def script() -> list[dict[str, Any]]:
    """Three awake observations (the second repeats the first; the third changes hunger), then a sleep, then waking."""
    base = recorded()
    changed = with_word(base, "hunger", "hungry")
    return [base, copy.deepcopy(base), changed, asleep(changed), awake(changed)]


def triggers_of(frames: list[dict[str, Any]]) -> list[str | None]:
    last: Felt | None = None
    out: list[str | None] = []
    for raw in frames:
        now = felt_of(situation_of(raw))
        out.append(feeling_due(now, last))
        last = now
    return out


def test_the_feeling_trigger_fires_once_per_change_and_once_on_waking() -> None:
    frames = script()
    assert triggers_of(frames) == [
        None,
        None,
        "I notice how I feel: hunger: hungry.",
        None,
        "I notice that I am awake again.",
    ]


def test_a_sleeping_seat_is_not_asked_about_its_needs() -> None:
    changed = with_word(recorded(), "tiredness", "ok")
    frames = [recorded(), asleep(changed), asleep(changed)]
    assert triggers_of(frames) == [None, None, None]


def _completion(text: str) -> Completion:
    return Completion(
        text=text, finish="complete", usage=Usage(input_tokens=1, output_tokens=1, estimated=True)
    )


def _sleep_request(raw: dict[str, Any]) -> dict[str, Any]:
    situation = situation_of(raw)
    observation = Observation.model_validate(raw)
    choice = parse_choice(_completion(ANSWER))
    assert isinstance(choice, SocialChoice)
    proposal = validate(choice, situation, MemorySection(lines=()))
    assert isinstance(proposal, Sleep) and proposal.minutes == 480
    request = build(proposal, observation)
    assert isinstance(request, ActionRequest)
    dumped: dict[str, Any] = request.model_dump(mode="json")
    return dumped


def test_a_scripted_sleep_becomes_the_request_the_sdk_builds_for_the_offer() -> None:
    raw = recorded()
    observation = Observation.model_validate(raw)
    assert situation_of(raw).sleep_offered
    expected = offers.request(observation, SLEEP, target=None, payload={"for": 480})
    assert _sleep_request(raw) == expected.model_dump(mode="json")


def test_a_replay_of_the_same_script_gives_identical_requests() -> None:
    raw = recorded()
    assert _sleep_request(raw) == _sleep_request(copy.deepcopy(raw))


def test_sleep_is_refused_by_the_seat_when_it_is_not_offered() -> None:
    raw = recorded()
    raw["affordances"] = [a for a in raw["affordances"] if a["action_type"] != "sleep"]
    situation = situation_of(raw)
    assert not situation.sleep_offered
    choice = parse_choice(_completion(ANSWER))
    assert isinstance(choice, SocialChoice)
    failure = validate(choice, situation, MemorySection(lines=()))
    assert isinstance(failure, ContentFailure) and failure.reason == "unknown_offer"


def test_the_choice_schema_offers_sleep_only_where_sleep_is_offered() -> None:
    with_sleep = choice_schema(go=False, amount=False, sleep=True)
    properties, required = with_sleep["properties"], with_sleep["required"]
    assert isinstance(properties, dict) and "minutes" in properties
    assert isinstance(required, list) and "minutes" in required
    assert choice_schema(go=False, amount=False, sleep=False) == SOCIAL_CHOICE_SCHEMA


def test_the_prompt_names_sleep_only_where_it_is_offered() -> None:
    raw = recorded()
    trigger = Trigger(event=EventId("9"), kind="feeling", speaker=None, words="x")
    offered = build_request(
        SEAT, Persona(), situation_of(raw), MemorySection(lines=()), trigger, ()
    )
    assert '"sleep" with "minutes"' in offered.messages[0].text
    assert "(say, attempt, sleep or nothing)" in offered.messages[0].text
    plain = copy.deepcopy(raw)
    plain["affordances"] = [a for a in plain["affordances"] if a["action_type"] != "sleep"]
    request = build_request(
        SEAT, Persona(), situation_of(plain), MemorySection(lines=()), trigger, ()
    )
    assert '"sleep"' not in request.messages[0].text
    assert "(say, attempt or nothing)" in request.messages[0].text
