"""Leaving early, the LM seat's half (`pr-le-leave-early.md` §0.4 and §0.5, LE-c C3 to C5), with scripted models only.

The world discloses facts — schedule's `agenda.next`, movement's `ways`, employment's `late-for-shift` — and the seat
turns them into prompt lines, an `upcoming` or `complaint` trigger, and the `go` and `amount` choices. Every value
here is a literal; the walking minutes are worked by hand from the shared formula (`ceil(length / 1340) + doors + 1`
strides at the seat's time scale), never read from the module under test.
"""

from __future__ import annotations

import json
from typing import Any, Literal

from memory_fixtures import ALICE, BOB, CAFE, PARK, open_view
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, JsonValue

from mineworld_cognition.backend.model import Completion, Usage
from mineworld_cognition.controller.choice import (
    SOCIAL_CHOICE_SCHEMA,
    ContentFailure,
    parse_choice,
)
from mineworld_cognition.controller.intent import build
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger, build_request
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.controller.triggers import Asked, complaint_line, upcoming_due
from mineworld_cognition.controller.validate import Attempt, Go, validate
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection

HOME = EntityId("5000000009")
EMPLOYER = EntityId("6000000001")
SEAT = EntityKey("alice")
FROM = 5 * 3600 + 30 * 60
"""05:30 on day 0: the punctual part begins."""
SCALE = 12
"""A hosted seat at 12x: one stride per wall second is 12 world seconds."""
# 52 000 mm and three doorways: ceil(52 000 / 1 340) = 39, + 3 + 1 = 43 strides; at 12 s = 516 s → 9 minutes.
CAFE_WAY = {"place": {"entity": CAFE, "entity_type": "place"}, "doors": 3, "length": 52_000}
CAFE_MINUTES = 9
PARK_WAY = {"place": {"entity": PARK, "entity_type": "place"}, "doors": 1, "length": 1_340}


def _named(entity: EntityId, kind: str, name: str, extra: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "id": entity,
        "entity_type": kind,
        "location": None,
        "tags": [],
        "components": [
            {
                "entity": entity,
                "component_type": "display-name",
                "schema_version": 1,
                "payload": {"name": name},
            },
            *extra,
        ],
    }


def _record(component: str, payload: object) -> dict[str, Any]:
    return {"entity": ALICE, "component_type": component, "schema_version": 1, "payload": payload}


def _observation(
    at: int,
    *,
    punctual: bool = True,
    ways: list[dict[str, Any]] | None = None,
    walking: bool = False,
    affordances: list[dict[str, Any]] | None = None,
) -> Observation:
    agenda = {
        "place": {"entity": HOME, "entity_type": "place"},
        "label": "home",
        "since": 0,
        "until": FROM,
        "next": {
            "place": {"entity": CAFE, "entity_type": "place"},
            "label": "work",
            "from": FROM,
            "punctual": punctual,
        },
    }
    mine = [
        _record("agenda", agenda),
        _record("ways", [CAFE_WAY, PARK_WAY] if ways is None else ways),
    ]
    if walking:
        mine.append(_record("walking", {}))
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": at,
            "self_location": {
                "place": {"entity": HOME, "entity_type": "place"},
                "local": None,
                "facing": None,
            },
            "entities": [
                _named(ALICE, "person", "Alice", mine),
                _named(CAFE, "place", "the Café", []),
                _named(PARK, "place", "the Park", []),
            ],
            "relations": [],
            "events": [],
            "affordances": [] if affordances is None else affordances,
        }
    )


def _offer(action: str, target: EntityId | None, payload: JsonValue) -> dict[str, Any]:
    wire: dict[str, Any] = {
        "action_type": action,
        "target": target,
        "available": True,
        "unavailable_reason": None,
        "requirement": {
            "place": "any",
            "within_range": None,
            "requires_line_of_access": False,
            "requires_target_available": False,
        },
    }
    if payload is not None:
        wire["payload"] = payload
    return wire


def _walkable(at: int) -> Observation:
    """Walking is offered (an incomplete affordance: the destination is the client's)."""
    return _observation(at, affordances=[_offer("walk-to", None, None)])


def _situation(observation: Observation) -> Situation:
    return Situation.of(observation, NameBook.load(open_view()), stride_s=SCALE)


def _answer(text: str) -> Completion:
    return Completion(
        text=text, finish="complete", usage=Usage(input_tokens=1, output_tokens=1, estimated=True)
    )


def _prompt(
    situation: Situation,
    kind: Literal["upcoming", "complaint"] = "upcoming",
    words: str = "",
) -> tuple[str, str]:
    request = build_request(
        SEAT,
        Persona(),
        situation,
        MemorySection(lines=()),
        Trigger(event=EventId("9"), kind=kind, speaker=None, words=words),
        (),
    )
    return request.messages[0].text, request.messages[1].text


# ── Facts in the prompt (§0.4 item 1) ─────────────────────────────────────────────────────────────


def test_a_punctual_next_part_is_one_line_with_the_walking_minutes_at_the_seat_cadence() -> None:
    situation = _situation(_observation(FROM - 3600))
    assert situation.plans == (
        "My work at the Café starts at 05:30; I mean to be there when it begins."
        f" Walking there takes about {CAFE_MINUTES} minutes.",
    )
    assert [place.label for place in situation.places] == ["the Café", "the Park"]
    _, user = _prompt(situation)
    assert "MY DAY\nNow: 04:30\n" in user
    assert situation.plans[0] in user
    assert "Places I can walk to: the Café, the Park" in user
    assert "leave" not in user.lower(), "the prompt gives no advice about when to leave"


def test_an_unknown_way_and_a_non_punctual_part_are_stated_as_they_are() -> None:
    unknown = _situation(_observation(FROM - 3600, ways=[PARK_WAY]))
    assert unknown.plans[0].endswith("I do not know how far it is.")
    assert [place.label for place in unknown.places] == ["the Park", "the Café"]
    loose = _situation(_observation(FROM - 3600, punctual=False))
    assert loose.plans == ("Next: work at the Café from 05:30.",)


def test_a_seat_with_no_day_renders_exactly_as_before() -> None:
    bare = Observation.model_validate(
        {
            "observer": ALICE,
            "at": 0,
            "self_location": None,
            "entities": [],
            "relations": [],
            "events": [],
            "affordances": [],
        }
    )
    situation = _situation(bare)
    assert situation.plans == () and situation.places == ()
    request = build_request(
        SEAT,
        Persona(),
        situation,
        MemorySection(lines=()),
        Trigger(event=EventId("9"), kind="arrived", speaker=None, words=""),
        (),
    )
    assert request.output_schema == SOCIAL_CHOICE_SCHEMA
    assert "MY DAY" not in request.messages[1].text
    assert '"go"' not in request.messages[0].text


# ── The upcoming trigger (§0.4 item 2) ────────────────────────────────────────────────────────────


def test_upcoming_asks_once_inside_the_horizon_then_every_reask_until_set_out_arrived_or_begun() -> (
    None
):
    horizon, reask = 3600, 900
    part = _situation(_observation(0)).next
    assert part is not None

    def due(at: int, last: Asked | None, *, walking: bool = False, here: EntityId = HOME) -> bool:
        return upcoming_due(part, here, walking, at, horizon, reask, last)

    assert not due(FROM - horizon - 1, None), "before the horizon"
    assert due(FROM - horizon, None)
    asked = Asked(from_at=FROM, at=FROM - horizon)
    assert not due(FROM - horizon + reask - 1, asked)
    assert due(FROM - horizon + reask, asked)
    assert not due(FROM - 60, None, walking=True), "set out"
    assert not due(FROM - 60, None, here=CAFE), "arrived"
    assert not due(FROM, None), "the part has begun"
    loose = part.model_copy(update={"punctual": False})
    assert not upcoming_due(loose, HOME, False, FROM - 60, horizon, reask, None)


# ── The go choice (§0.4 item 3) ───────────────────────────────────────────────────────────────────


def test_go_names_a_listed_place_and_builds_one_walk_to_it() -> None:
    observation = _walkable(FROM - 1800)
    situation = _situation(observation)
    system, _ = _prompt(situation)
    assert '"go"' in system
    schema = build_request(
        SEAT,
        Persona(),
        situation,
        MemorySection(lines=()),
        Trigger(event=EventId("9"), kind="upcoming", speaker=None, words=""),
        (),
    ).output_schema
    assert schema is not None and "go" in json.dumps(schema)
    choice = parse_choice(
        _answer(
            '{"act": "go", "to": null, "words": "", "offer": null, "recalls": [],'
            ' "place": "the Café", "amount": null}'
        )
    )
    assert not isinstance(choice, ContentFailure)
    proposal = validate(choice, situation, MemorySection(lines=()))
    assert proposal == Go(kind="go", place=CAFE, recalls=())
    assert not isinstance(proposal, ContentFailure)
    request = build(proposal, observation)
    assert request is not None and not hasattr(request, "reason")
    assert request.model_dump(mode="json")["payload"] == {
        "action_type": "walk-to",
        "payload": {
            "to": {
                "place": {
                    "place": {"entity": CAFE, "entity_type": "place"},
                    "local": None,
                    "facing": None,
                }
            }
        },
    }


def test_go_to_an_unlisted_place_is_a_content_failure() -> None:
    situation = _situation(_walkable(FROM - 1800))
    choice = parse_choice(
        _answer(
            '{"act": "go", "to": null, "words": "", "offer": null, "recalls": [],'
            ' "place": "the Moon", "amount": null}'
        )
    )
    assert not isinstance(choice, ContentFailure)
    failure = validate(choice, situation, MemorySection(lines=()))
    assert isinstance(failure, ContentFailure) and failure.reason == "unknown_place"


def test_a_script_that_goes_when_the_prompt_says_it_is_time_goes_before_it_would_be_late() -> None:
    """AC-LE2-5's script, offline: the seat is asked every reask from the horizon; the script reads only the
    prompt ("takes about N minutes", "Now", "starts at") and says `go` once now >= start - 1.5 N - 5 min."""
    horizon, reask = 3600, 900
    asked: Asked | None = None
    went: int | None = None
    for at in range(FROM - 2 * horizon, FROM, 60):
        situation = _situation(_walkable(at))
        if not upcoming_due(situation.next, HOME, False, at, horizon, reask, asked):
            continue
        assert situation.next is not None
        asked = Asked(from_at=situation.next.from_at, at=at)
        _, user = _prompt(situation)
        minutes = int(user.split("takes about ")[1].split(" ")[0])
        now = user.split("Now: ")[1][:5]
        now_s = int(now[:2]) * 3600 + int(now[3:]) * 60
        if now_s >= FROM - (minutes * 60 * 3) // 2 - 300:
            went = at
            break
    assert went is not None, "the script went"
    assert went + CAFE_MINUTES * 60 <= FROM, "and arrives on time if the walk takes its estimate"


# ── Complaint and penalize (§0.5, §0.6) ───────────────────────────────────────────────────────────


def _late(**fields: JsonValue) -> dict[str, JsonValue]:
    body: dict[str, JsonValue] = {
        "employee": {"entity": ALICE, "entity_type": "person"},
        "employer": {"entity": EMPLOYER, "entity_type": "organization"},
        "manager": {"entity": BOB, "entity_type": "person"},
        "shift": FROM,
        "late_by": 700,
        "count": 3,
        "level": "final",
        "absent": False,
    }
    body.update(fields)
    return body


def test_the_complaint_line_in_the_manager_s_and_the_employee_s_words() -> None:
    store = open_view()
    names = NameBook.load(store)
    names.learn(ALICE, "Alice")
    names.learn(EMPLOYER, "the Café")
    assert (
        complaint_line(_late(window_days=7), BOB, names)
        == "Alice was 12 minutes late for the shift at the Café — the third time in 7 days."
        " It is a final warning."
    )
    assert (
        complaint_line(_late(level="notice", count=1), ALICE, names)
        == "I was 12 minutes late for my shift at the Café — the first time."
    )
    assert (
        complaint_line(_late(absent=True, level="warning", count=2), ALICE, names)
        == "I did not come to my shift at the Café — the second time. It is a warning."
    )
    assert complaint_line(_late(), EntityId("7000000099"), names) is None, "not my fact"
    _, user = _prompt(_situation(_observation(FROM)), "complaint", "I was late.")
    assert user.endswith("WHAT JUST HAPPENED\nI was late.")


def test_a_bounded_offer_lists_its_bound_and_an_amount_inside_it_is_sent() -> None:
    # employment's complete offer (LE-b): untargeted, the employee named in the payload.
    employee = {"entity": ALICE, "entity_type": "person"}
    penalize = _offer("penalize", None, {"employee": employee, "amount": 600})
    observation = _observation(FROM + 900, affordances=[penalize])
    situation = Situation.of(observation, NameBook.load(open_view()), stride_s=SCALE)
    assert situation.offers[0].up_to == 600
    system, user = _prompt(situation, "complaint", "Alice was late.")
    assert "1. penalize with Alice (up to 600)" in user
    assert '"amount"' in system

    def answer(amount: str) -> Attempt | ContentFailure:
        choice = parse_choice(
            _answer(
                '{"act": "attempt", "to": null, "words": "", "offer": 1, "recalls": [],'
                f' "place": null, "amount": {amount}}}'
            )
        )
        assert not isinstance(choice, ContentFailure)
        proposal = validate(choice, situation, MemorySection(lines=()))
        assert isinstance(proposal, Attempt | ContentFailure)
        return proposal

    chosen = answer("250")
    assert isinstance(chosen, Attempt) and chosen.amount == 250
    request = build(chosen, observation)
    assert request is not None and not hasattr(request, "reason")
    assert request.model_dump(mode="json")["payload"]["payload"] == {
        "employee": employee,
        "amount": 250,
    }
    whole = answer("null")
    assert isinstance(whole, Attempt)
    whole_request = build(whole, observation)
    assert whole_request is not None and not hasattr(whole_request, "reason")
    assert whole_request.model_dump(mode="json")["payload"]["payload"] == {
        "employee": employee,
        "amount": 600,
    }
    for wrong in ("0", "601"):
        failure = answer(wrong)
        assert isinstance(failure, ContentFailure) and failure.reason == "bad_amount"


# ── A disclosed shift is an appointment (S35 N-h R6) ─────────────────────────────────────────────


def test_a_shift_employment_discloses_makes_the_unflagged_part_at_the_workplace_punctual() -> None:
    observation = _observation(FROM - 3600, punctual=False)
    job: dict[str, Any] = {
        "job": {
            "employer": {"entity": EMPLOYER, "entity_type": "organization"},
            "workplace": {"entity": CAFE, "entity_type": "place"},
            "from": "05:30",
            "until": "10:00",
            "wage": 100,
            "produces": list[Any](),
        }
    }
    observation.entities[0].components.append(
        observation.entities[0].components[0].model_validate(_record("employment", job))
    )
    situation = _situation(observation)
    assert situation.next is not None and situation.next.punctual
    assert situation.plans[0].startswith("My work at the Café starts at 05:30; I mean to be there")
    # Without the job, the same unflagged part is only stated.
    plain = _situation(_observation(FROM - 3600, punctual=False))
    assert plain.next is not None and not plain.next.punctual
