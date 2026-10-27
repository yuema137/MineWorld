"""The closed filler registry, the `due` trigger, and nothing else moving (S31-d §§5.8.2-5.8.4; AC-d-13, AC-d-14).

A filler lists itself only when the world offers its action available and the seat has what it needs; a seat no
filler lists gets no `due` trigger, no `fill` act and no `fill` schema, and its request is byte for byte the request
built before fillers existed (I-d-8).
"""

from __future__ import annotations

import json

from memory_fixtures import ALICE, CAFE, fact, seat_view
from mineworld_sdk.wire.ids import ActionTypeId, EntityKey, EventId
from reporter_fixtures import crown, dollar, settings

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import Completion, Usage
from mineworld_cognition.config import FillConfig
from mineworld_cognition.controller import fill
from mineworld_cognition.controller.choice import (
    SOCIAL_CHOICE_SCHEMA,
    ContentFailure,
    SocialChoice,
    parse_choice,
)
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger, build_request
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.controller.triggers import detect
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection


def _listed(observation_name: str, config: FillConfig) -> tuple[fill.Fillable, ...]:
    observation = {
        "dollar": dollar(),
        "unavailable": dollar(available=False),
        "absent": dollar(offered=False),
        "crown": crown(),
    }[observation_name]
    with seat_view() as store:
        names = NameBook.load(store)
        Situation.of(observation, names)
        return fill.listed(observation, names, config)


def test_the_registry_is_closed_and_holds_the_publish_notice_filler() -> None:
    assert list(fill.FILLERS) == ["publish-notice"]
    filler = fill.FILLERS[ActionTypeId("publish-notice")]
    assert filler.key == "notice"
    assert dict(filler.due_events) == {"day-began": "A new day began."}


def test_a_filler_lists_itself_only_when_offered_available_with_a_book_and_a_topic() -> None:
    assert [item.number for item in _listed("dollar", settings())] == ["F1"]
    assert [item.number for item in _listed("crown", settings("market.daily"))] == ["F1"]
    assert _listed("unavailable", settings()) == ()
    assert _listed("absent", settings()) == ()
    assert _listed("dollar", FillConfig()) == ()


def test_due_events_are_the_listed_fillers_only() -> None:
    assert dict(fill.due_events(_listed("dollar", settings()))) == {"day-began": "A new day began."}
    assert dict(fill.due_events(())) == {}


def test_a_day_began_fact_is_a_due_trigger_only_for_a_listed_seat() -> None:
    day = fact(501, 86_400, "day-began", {"day": 1}, subjects=[], participants=[], place=None)
    with seat_view() as store:
        unlisted = detect([day], ALICE, store, None, 86_400, {}, 1800)
        listed = detect(
            [day], ALICE, store, None, 86_400, {}, 1800, {"day-began": "A new day began."}
        )
    assert unlisted == []
    assert listed == [
        Trigger(event=EventId("501"), kind="due", speaker=None, words="A new day began.")
    ]


def _social() -> Situation:
    return Situation(me="Alice", place="Cafe", at=4112, present=(), talk_to=(), offers=())


def test_an_unlisted_request_is_byte_identical_and_carries_no_fill() -> None:
    trigger = Trigger(event=EventId("77"), kind="arrived", speaker=CAFE, words="")
    memory = MemorySection(lines=())
    before = build_request(EntityKey("alice"), Persona(), _social(), memory, trigger, ())
    after = build_request(
        EntityKey("alice"), Persona(), _social(), memory, trigger, (), fillables=()
    )
    assert cassette_key(before) == cassette_key(after)
    assert before.output_schema == SOCIAL_CHOICE_SCHEMA
    assert "fill" not in json.dumps(before.output_schema)
    text = "\n".join(message.text for message in before.messages)
    assert "TO FILL" not in text and '"fill"' not in text


def test_a_listed_request_adds_the_rule_the_section_and_the_schema() -> None:
    fillables = _listed("dollar", settings())
    trigger = Trigger(event=EventId("9001"), kind="due", speaker=None, words="A new day began.")
    request = build_request(
        EntityKey("carol"),
        Persona(),
        _social(),
        MemorySection(lines=()),
        trigger,
        (),
        fillables=fillables,
    )
    system, user = request.messages[0].text, request.messages[1].text
    assert 'Use "fill" only with an F-number listed under TO FILL' in system
    assert '"act" (say, attempt, fill or nothing)' in system
    assert user.index("TO FILL") < user.index("WHAT I REMEMBER")
    assert user.endswith("WHAT JUST HAPPENED\nA new day began.")
    assert SOCIAL_CHOICE_SCHEMA["required"] == ["act", "to", "words", "offer", "recalls"]


def _completion(answer: dict[str, object]) -> Completion:
    return Completion(
        text=json.dumps(answer),
        finish="complete",
        usage=Usage(input_tokens=0, output_tokens=1, estimated=True),
    )


def test_without_a_listed_filler_the_fill_act_and_key_are_refused_as_before() -> None:
    base: dict[str, object] = {"to": None, "words": "", "offer": None, "recalls": []}
    act = parse_choice(_completion({**base, "act": "fill"}))
    key = parse_choice(_completion({**base, "act": "nothing", "fill": None}))
    assert act == ContentFailure(reason="schema", detail="act: literal_error")
    assert key == ContentFailure(reason="schema", detail="fill: extra_forbidden")
    listed = parse_choice(
        _completion({**base, "act": "fill", "fill": {"offer": "F1"}}), fillers=True
    )
    assert isinstance(listed, SocialChoice) and listed.act == "fill"
