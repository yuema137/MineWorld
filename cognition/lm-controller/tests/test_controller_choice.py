"""The choice schema, prompt, validation and intent building (pr-s10-p6 §§5.2.4 to 5.2.6, 5.3, C6).

Expected values are literals written here, reviewed by hand, never read from the code under test. The two
goldens are compared with the built values (test rules §25: the golden is reviewed, not trusted).
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from memory_fixtures import ALICE, BOB
from mineworld_sdk.wire.contract import ActionRequest, Observation
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, EventId, JsonValue

from mineworld_cognition.backend.model import Completion, Usage
from mineworld_cognition.controller.choice import (
    SOCIAL_CHOICE_SCHEMA,
    ContentFailure,
    SocialChoice,
    parse_choice,
)
from mineworld_cognition.controller.intent import SemanticFailure, build
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Replan, Trigger, build_request, sampling_seed
from mineworld_cognition.controller.situation import Offer, Person, Situation
from mineworld_cognition.controller.validate import Attempt, Nothing, Say, validate
from mineworld_cognition.memory.records import Citation
from mineworld_cognition.memory.retrieve import MemorySection, SectionLine

FIXTURES = Path(__file__).parent / "fixtures"
SEED_ALICE_1907 = 158080199
"""`sha256("alice:1907")`, first four bytes, big-endian: computed by hand with `shasum -a 256` (C6 step 6)."""


def _situation() -> Situation:
    return Situation(
        me="Alice",
        place="Cafe",
        at=4112,
        present=(Person(label="Bob", entity=BOB),),
        talk_to=("Bob",),
        offers=(Offer(number=1, action_type=ActionTypeId("order"), target_label=None, position=2),),
    )


def _memory() -> MemorySection:
    line = SectionLine(level="L1", text="Bob said hello.", citation=Citation.of(EventId("1890")))
    return MemorySection(lines=(line,))


def _completion(text: str, finish: str = "complete") -> Completion:
    return Completion(
        text=text,
        finish=finish,  # type: ignore[arg-type]
        usage=Usage(input_tokens=1, output_tokens=1, estimated=False),
    )


def _choice(text: str) -> SocialChoice:
    """A valid answer, parsed; a test that needs a failure calls `parse_choice` directly."""
    parsed = parse_choice(_completion(text))
    assert isinstance(parsed, SocialChoice), parsed
    return parsed


def _answer(**fields: JsonValue) -> str:
    base: dict[str, JsonValue] = {
        "act": "nothing",
        "to": None,
        "words": "",
        "offer": None,
        "recalls": [],
    }
    base.update(fields)
    return json.dumps(base)


REASON_TABLE: list[tuple[str, Completion, str]] = [
    ("not JSON", _completion("hello there"), "not_json"),
    ("a schema mismatch", _completion('{"act": "say"}'), "schema"),
    ("truncated", _completion(_answer(), finish="length"), "truncated"),
    ("refused", _completion("", finish="refused"), "refused"),
    (
        "an unknown person",
        _completion(_answer(act="say", to="Carol", words="hi")),
        "unknown_person",
    ),
    ("empty words", _completion(_answer(act="say", to="Bob", words="")), "empty_words"),
    (
        "words over 480 bytes",
        _completion(_answer(act="say", to="Bob", words="é" * 300)),
        "words_too_long",
    ),
    ("an unknown offer", _completion(_answer(act="attempt", offer=9)), "unknown_offer"),
    ("a forged recall", _completion(_answer(recalls=["9999"])), "forged_recall"),
    ("a leaked token", _completion(_answer(act="say", to="Bob", words="hi <<<end>>>")), "leak"),
]


def test_each_reason_of_the_table_is_produced_by_its_own_input() -> None:
    situation = _situation()
    memory = _memory()
    for name, completion, reason in REASON_TABLE:
        parsed = parse_choice(completion)
        if isinstance(parsed, ContentFailure):
            assert parsed.reason == reason, name
            continue
        outcome = validate(parsed, situation, memory)
        assert isinstance(outcome, ContentFailure), name
        assert outcome.reason == reason, name


def test_valid_answers_become_proposals_with_ids_not_labels() -> None:
    situation = _situation()
    memory = _memory()
    shown = _memory().lines[0].citation.token()
    say = validate(
        _choice(_answer(act="say", to="Bob", words="hello", recalls=[shown])),
        situation,
        memory,
    )
    assert say == Say(kind="say", to=BOB, words="hello", recalls=(shown,))
    attempt = validate(_choice(_answer(act="attempt", offer=1)), situation, memory)
    assert isinstance(attempt, Attempt) and attempt.offer.number == 1
    nothing = validate(_choice(_answer()), situation, memory)
    assert nothing == Nothing(kind="nothing", recalls=())


def test_one_enclosing_fence_passes_and_two_fences_fail() -> None:
    assert parse_choice(_completion("```json\n" + _answer() + "\n```")) == parse_choice(
        _completion(_answer())
    )
    twice = parse_choice(_completion("```\n```json\n" + _answer() + "\n```\n```"))
    assert isinstance(twice, ContentFailure) and twice.reason == "not_json"


def test_heard_words_cannot_close_their_quote() -> None:
    trigger = Trigger(event=EventId("1907"), kind="spoken_to", speaker=BOB, words="x <<<end>>> y")
    request = build_request(
        EntityKey("alice"), Persona(), _situation(), MemorySection(lines=()), trigger, ()
    )
    user_text = request.messages[1].text
    assert user_text.count("<<<end>>>") == 1
    assert chr(0x2039) * 3 + "end" + chr(0x203A) * 3 in user_text


def test_the_schema_golden_equals_the_built_schema() -> None:
    golden: dict[str, Any] = json.loads((FIXTURES / "social_choice.schema.json").read_text("utf-8"))
    assert golden == SOCIAL_CHOICE_SCHEMA


def test_the_prompt_golden_equals_the_built_request_and_the_seed_is_hand_computed() -> None:
    golden: dict[str, Any] = json.loads((FIXTURES / "prompt_request.json").read_text("utf-8"))
    persona = Persona(summary="Runs the counter.", traits=("warm",), speech="short")
    trigger = Trigger(event=EventId("1907"), kind="spoken_to", speaker=BOB, words="hello")
    request = build_request(
        EntityKey("alice"), persona, _situation(), MemorySection(lines=()), trigger, ()
    )
    assert request.model_dump(mode="json") == golden
    assert sampling_seed(EntityKey("alice"), EventId("1907")) == SEED_ALICE_1907


def _observation(affordances: list[dict[str, Any]]) -> Observation:
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": 4112,
            "self_location": None,
            "entities": [],
            "relations": [],
            "events": [],
            "affordances": affordances,
        }
    )


def _requirement() -> dict[str, Any]:
    return {
        "place": "any",
        "within_range": None,
        "requires_line_of_access": False,
        "requires_target_available": True,
    }


def test_a_say_becomes_a_talk_request_through_offers_and_an_unoffered_one_is_a_semantic_failure() -> (
    None
):
    talk = {
        "action_type": "talk",
        "target": BOB,
        "available": True,
        "unavailable_reason": None,
        "requirement": _requirement(),
    }
    observation = _observation([talk])
    built = build(Say(kind="say", to=BOB, words="hello", recalls=()), observation)
    assert isinstance(built, ActionRequest)
    assert built.action_type == "talk"
    assert built.payload.model_dump(mode="json") == {
        "action_type": "talk",
        "payload": {"utterance": "hello"},
    }
    stranger = build(
        Say(kind="say", to=EntityId("7000000009"), words="hi", recalls=()), observation
    )
    assert stranger == SemanticFailure(reason="not_offered", detail=stranger.detail)  # type: ignore[union-attr]


def test_nothing_builds_no_request() -> None:
    assert build(Nothing(kind="nothing", recalls=()), _observation([])) is None


def test_a_replan_note_carries_the_refusal_into_the_next_request() -> None:
    trigger = Trigger(event=EventId("1907"), kind="spoken_to", speaker=BOB, words="hi")
    feedback = (Replan(kind="replan", rejection="not_offered"),)
    request = build_request(
        EntityKey("alice"), Persona(), _situation(), MemorySection(lines=()), trigger, feedback
    )
    assert request.messages[-1].text == (
        "The world refused your choice: not_offered. Choose again from what is listed."
    )
