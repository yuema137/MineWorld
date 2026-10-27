"""The `publish-notice` filler, on two clerks (S31-d §5.8; AC-d-13; `ARC-183`).

Each content failure is named; the payload is the notices pack's own shape with lines in item order; a world
refusal re-plans; an offer gone by the intent is `not_offered`. The scripted model is a pure function of the
request, so nothing is reached.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from memory_fixtures import seat_view
from mineworld_sdk.session import Answered
from mineworld_sdk.wire.contract import ActionRequest, Rejected
from mineworld_sdk.wire.ids import ActionId, JsonValue
from reporter_fixtures import APPLE, BREAD, JUICE, crown, dollar, settings
from reporter_harness import accepted, run_due
from scripted_reporter import reporter

from mineworld_cognition.backend.model import Completion, CompletionRequest, Usage
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.controller import fill
from mineworld_cognition.controller.choice import ContentFailure
from mineworld_cognition.controller.fill.publish_notice import BookShown, PublishNoticeFiller
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.memory.names import NameBook

GOLDEN = Path(__file__).resolve().parent / "fixtures" / "fill_choice.schema.json"
FILLER = PublishNoticeFiller()


def _shown(observation_name: str = "dollar") -> BookShown:
    observation = dollar() if observation_name == "dollar" else crown()
    topic = "prices.ingredients" if observation_name == "dollar" else "market.daily"
    with seat_view() as store:
        names = NameBook.load(store)
        Situation.of(observation, names)
        shown = FILLER.listed(observation, names, settings(topic))
    assert isinstance(shown, BookShown)
    return shown


def _lines(*lines: tuple[str, int, int]) -> JsonValue:
    return {
        "lines": [{"item": item, "price": price, "basis": basis} for item, price, basis in lines]
    }


def test_the_book_is_shown_in_labels_only() -> None:
    lines = FILLER.section("F1", _shown())
    assert lines[2:] == (
        "MY PRICE BOOK (amounts in minor units of the US dollar: 300 = $3.00)",
        "Bread (5 notes): sales 300, 320, 300; asks store shop 300 (stock 1), Grace Okafor 260",
        "Apple (1 notes): sales 100; asks none",
        "Juice (7 notes): sales 250, 250, 350, 350, 340; asks store shop 250 (stock 4),"
        " cafe shop 350 (stock 9)",
    )
    assert lines[0].startswith(
        'F1. publish-notice: post suggested prices on prices.ingredients. Answer "act"'
    )
    text = "\n".join(lines)
    for entity in ("41", "42", "43", "6000000001", "7000000012", "5000000011"):
        assert entity not in text


def test_a_kind_without_a_catalogue_name_is_numbered_and_a_crown_has_no_decimals() -> None:
    lines = FILLER.section("F1", _shown("crown"))
    assert lines[2:] == (
        "MY PRICE BOOK (amounts in minor units of the crown: 3 = 3 cr)",
        "kind 1 (3 notes): sales 3, 3, 4; asks none",
        "kind 2 (2 notes): sales 6; asks market shop 5 (stock 2)",
    )


def test_a_checked_answer_is_the_notices_packs_payload_in_item_order() -> None:
    checked = FILLER.check(_lines(("Juice", 340, 7), ("Bread", 330, 3)), _shown())
    assert not isinstance(checked, ContentFailure)
    assert checked.payload == {
        "topic": "prices.ingredients",
        "body": {
            "prices": [
                {"item": str(BREAD), "price": 330, "basis": 3},
                {"item": str(JUICE), "price": 340, "basis": 7},
            ]
        },
        "valid_for": 21600,
    }
    assert checked.target is None


@pytest.mark.parametrize("fixture", ["dollar", "crown"])
@pytest.mark.parametrize(
    ("content", "reason"),
    [
        (None, "no_lines"),
        ({"lines": []}, "no_lines"),
        (_lines(*[("Bread", 1, 1)] * 33), "no_lines"),
        (_lines(("Croissant", 300, 1)), "unknown_item"),
        (_lines(("FIRST", 300, 1), ("FIRST", 310, 1)), "duplicate_item"),
        (_lines(("FIRST", 0, 1)), "price_out_of_range"),
        (_lines(("FIRST", 10**12 + 1, 1)), "price_out_of_range"),
        (_lines(("FIRST", 300, 0)), "basis_out_of_range"),
        (_lines(("FIRST", 300, -1)), "basis_out_of_range"),
        ({"lines": [{"item": "FIRST", "price": "300", "basis": 1}]}, "schema"),
    ],
)
def test_each_content_failure_is_named(fixture: str, content: JsonValue, reason: str) -> None:
    shown = _shown(fixture)
    first = shown.kinds[0].label
    text = json.dumps(content).replace("FIRST", first)
    checked = FILLER.check(json.loads(text), shown)
    assert isinstance(checked, ContentFailure)
    assert checked.reason == reason


@pytest.mark.parametrize("fixture", ["dollar", "crown"])
def test_a_basis_one_more_than_the_notes_shown_is_refused(fixture: str) -> None:
    shown = _shown(fixture)
    kind = shown.kinds[0]
    over = FILLER.check(_lines((kind.label, 5, kind.notes + 1)), shown)
    assert isinstance(over, ContentFailure) and over.reason == "basis_out_of_range"
    exact = FILLER.check(_lines((kind.label, 5, kind.notes)), shown)
    assert not isinstance(exact, ContentFailure)


def test_the_listed_schema_is_the_golden() -> None:
    with seat_view() as store:
        names = NameBook.load(store)
        Situation.of(dollar(), names)
        fillables = fill.listed(dollar(), names, settings())
    requests: list[CompletionRequest] = []
    run_due(dollar(), settings(), ScriptedBackend(reporter), requests=requests)
    assert [item.number for item in fillables] == ["F1"]
    assert requests[0].output_schema == json.loads(GOLDEN.read_text(encoding="utf-8"))


def _answering(text: str) -> ScriptedBackend:
    def script(request: CompletionRequest) -> Completion:
        del request
        return Completion(
            text=text,
            finish="complete",
            usage=Usage(input_tokens=0, output_tokens=1, estimated=True),
        )

    return ScriptedBackend(script)


def test_an_f_number_not_listed_is_unknown_fill() -> None:
    answer = json.dumps(
        {
            "act": "fill",
            "to": None,
            "words": "",
            "offer": None,
            "recalls": [],
            "fill": {"offer": "F2", "notice": _lines(("Bread", 300, 3))},
        }
    )
    record = run_due(dollar(), settings(), _answering(answer))
    assert record.outcome == "fell_back"
    assert {attempt.failure.reason for attempt in record.attempts if attempt.failure} == {
        "unknown_fill"
    }


def test_the_scripted_clerk_posts_and_the_request_goes_through_the_offer() -> None:
    record = run_due(dollar(), settings(), ScriptedBackend(reporter))
    assert record.outcome == "acted"
    request = record.request
    assert isinstance(request, ActionRequest)
    assert request.action_type == "publish-notice"
    assert request.target is None
    assert request.payload.payload == {
        "topic": "prices.ingredients",
        "body": {
            "prices": [
                {"item": str(BREAD), "price": 300, "basis": 5},
                {"item": str(APPLE), "price": 100, "basis": 1},
                {"item": str(JUICE), "price": 340, "basis": 7},
            ]
        },
        "valid_for": 21600,
    }


def test_an_offer_gone_by_the_intent_is_not_offered() -> None:
    record = run_due(dollar(), settings(), ScriptedBackend(reporter), newest=dollar(offered=False))
    assert record.outcome == "fell_back"
    assert record.attempts[0].failure is not None
    assert record.attempts[0].failure.reason == "not_offered"


def test_a_world_refusal_re_plans() -> None:
    answers = iter(["rejected", "accepted"])

    async def submit(request: ActionRequest) -> Answered:
        del request
        if next(answers) == "rejected":
            return Answered(
                action_id=ActionId("1"),
                result=Rejected.model_validate({"rejected": "busy"}),
            )
        return accepted()

    requests: list[CompletionRequest] = []
    record = run_due(
        dollar(), settings(), ScriptedBackend(reporter), submit=submit, requests=requests
    )
    assert record.outcome == "acted"
    assert [attempt.layer for attempt in record.attempts] == ["semantic", "none"]
    assert "The world refused your choice: rejected." in requests[1].messages[-1].text
