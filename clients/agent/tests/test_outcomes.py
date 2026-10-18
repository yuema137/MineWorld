"""The outcome table (pr-s27c §5.6): one row per SDK outcome, the server's reason carried unchanged
(AC-c-3's unit half), and a refusal only a malformed frame earns treated as a defect of the library, never as
a world answer."""

from __future__ import annotations

import pytest
from mineworld_sdk.reconnect import AnswerLost, NotConnected
from mineworld_sdk.reconnect import SeatLost as SdkSeatLost
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.wire.contract import Accepted as WireAccepted
from mineworld_sdk.wire.contract import ActionResult, Rejected
from mineworld_sdk.wire.ids import ActionId
from pydantic import TypeAdapter

from mineworld_agent.models import EventLine, Outcome
from mineworld_agent.outcomes import LibraryDefect, outcome_of, unanswered

RESULT = TypeAdapter[ActionResult](ActionResult)
ACTION = ActionId("41")
CAUSED = [EventLine(ref="F3", at=12, event_type="spoke", subjects=["E1"], place="E2", words=None)]
CAUSED_JSON = [{"ref": "F3", "at": 12, "event_type": "spoke", "subjects": ["E1"], "place": "E2"}]


def answered(result: str) -> Answered:
    return Answered(ACTION, RESULT.validate_json(result))


@pytest.mark.parametrize(
    ("answer", "expected"),
    [
        (answered('{"accepted": {"events": ["7"]}}'), {"kind": "accepted", "events": CAUSED_JSON}),
        (answered('{"rejected": "too_far_away"}'), {"kind": "rejected", "reason": "too_far_away"}),
        (answered('{"rejected": "busy"}'), {"kind": "rejected", "reason": "busy"}),
        (
            answered('{"rejected": "permission_denied"}'),
            {"kind": "rejected", "reason": "permission_denied"},
        ),
        (
            answered('{"rejected": {"system": {"code": "no-route", "detail": "walled in"}}}'),
            {"kind": "rejected", "reason": "system", "system_code": "no-route"},
        ),
        (answered('"unavailable"'), {"kind": "unavailable"}),
        (RefusedRequest("paused", None), {"kind": "refused", "code": "paused"}),
        (RefusedRequest("dispatch_failed", "x"), {"kind": "refused", "code": "dispatch_failed"}),
        (RefusedRequest("world_stopped", None), {"kind": "refused", "code": "world_stopped"}),
        (
            RefusedRequest("actor_not_observer", None),
            {"kind": "refused", "code": "actor_not_observer"},
        ),
    ],
)
def test_every_answer_maps_to_its_outcome(
    answer: Answered | RefusedRequest, expected: dict[str, object]
) -> None:
    outcome: Outcome = outcome_of(answer, CAUSED)
    assert outcome.model_dump(exclude_none=True) == expected


def test_the_facts_are_given_only_to_an_accepted_outcome() -> None:
    assert outcome_of(answered('{"accepted": {"events": []}}'), []).model_dump() == {
        "kind": "accepted",
        "events": [],
    }
    rejected = outcome_of(answered('{"rejected": "busy"}'), CAUSED)
    assert "events" not in rejected.model_dump()
    assert isinstance(RESULT.validate_json('{"rejected": "busy"}'), Rejected)
    assert isinstance(RESULT.validate_json('{"accepted": {"events": []}}'), WireAccepted)


@pytest.mark.parametrize(
    ("error", "expected"),
    [
        (AnswerLost(), {"kind": "refused", "code": "answer_lost"}),
        (NotConnected(), {"kind": "refused", "code": "not_connected"}),
        (SdkSeatLost("taken_over"), {"kind": "seat_lost", "reason": "taken_over"}),
        (SdkSeatLost("world_stopped"), {"kind": "seat_lost", "reason": "world_stopped"}),
    ],
)
def test_a_submit_without_an_answer_says_why(
    error: AnswerLost | NotConnected | SdkSeatLost, expected: dict[str, str]
) -> None:
    assert unanswered(error).model_dump() == expected


@pytest.mark.parametrize(
    "code", ["malformed_frame", "not_joined", "unknown_frame", "already_joined"]
)
def test_a_refusal_only_a_malformed_frame_earns_is_a_library_defect(code: str) -> None:
    with pytest.raises(LibraryDefect, match=code):
        outcome_of(RefusedRequest(code, None), [])  # type: ignore[arg-type]
