"""The LM price clerk's cassette (S31-d AC-d-12, AC-S31-11 "cassette replay"; `ARC-58`).

`tests/cassettes/price-reporter.jsonl` was recorded once from the scripted clerk (`scripted_reporter.reporter`, a
pure function of the request) through `RecordingBackend(binding="local", model="scripted")`: no model was reached
and nothing was paid. It is replayed strictly for two `due` consults: a US-dollar clerk on `prices.ingredients`,
and a crown clerk of exponent 0 on `market.daily` whose kinds have no catalogue names. Each replay must hit the
recorded key and submit the committed request; a changed prompt line is a `CassetteMiss`, never a fallback.
Re-recording is a deliberate change: run this module with `PRICE_REPORTER_RECORD=1` and review the diff.
"""

from __future__ import annotations

import os
from pathlib import Path

import pytest
from mineworld_sdk.wire.contract import ActionRequest
from mineworld_sdk.wire.ids import EventId, JsonValue
from reporter_fixtures import APPLE, BREAD, FLOUR, JUICE, SALT, crown, dollar, settings
from reporter_harness import due, run_due
from scripted_reporter import reporter
from support import run

from mineworld_cognition.backend.model import ModelBackend
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import FillConfig
from mineworld_cognition.controller.consult import ConsultRecord
from mineworld_cognition.record import Cassette, CassetteMiss, RecordingBackend, ReplayBackend

CASSETTE = Path(__file__).resolve().parent / "cassettes" / "price-reporter.jsonl"

DOLLAR: JsonValue = {
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
CROWN: JsonValue = {
    "topic": "market.daily",
    "body": {
        "prices": [
            {"item": str(FLOUR), "price": 3, "basis": 3},
            {"item": str(SALT), "price": 6, "basis": 2},
        ]
    },
    "valid_for": 50400,
}


def _crown_settings() -> FillConfig:
    return settings("market.daily", valid_minutes=840)


def _both(backend_for: ModelBackend) -> tuple[ConsultRecord, ConsultRecord]:
    first = run_due(dollar(), settings(), backend_for)
    second = run_due(crown(), _crown_settings(), backend_for, trigger=due(EventId("9002")))
    return first, second


@pytest.mark.skipif(
    os.environ.get("PRICE_REPORTER_RECORD") != "1", reason="records the committed cassette"
)
def test_record_the_committed_cassette(tmp_path: Path) -> None:
    path = tmp_path / "price-reporter.jsonl"
    backend = RecordingBackend(ScriptedBackend(reporter), path, binding="local", model="scripted")
    _both(backend)
    run(backend.aclose())
    CASSETTE.write_bytes(path.read_bytes())


def _payload(record: ConsultRecord) -> JsonValue:
    assert record.outcome == "acted", record
    assert isinstance(record.request, ActionRequest)
    assert record.request.action_type == "publish-notice"
    assert record.request.target is None
    return record.request.payload.payload


def test_the_committed_cassette_replays_both_clerks_to_the_committed_requests() -> None:
    cassette = Cassette.load(CASSETTE)
    first, second = _both(ReplayBackend(cassette))
    assert _payload(first) == DOLLAR
    assert _payload(second) == CROWN
    assert [attempt.layer for attempt in first.attempts] == ["none"]
    assert [attempt.layer for attempt in second.attempts] == ["none"]


def test_the_scripted_clerk_and_its_recording_agree() -> None:
    scripted = ScriptedBackend(reporter)
    first, second = _both(scripted)
    assert (_payload(first), _payload(second)) == (DOLLAR, CROWN)


def test_a_changed_prompt_line_is_a_cassette_miss_not_a_fallback() -> None:
    replay = ReplayBackend(Cassette.load(CASSETTE))
    with pytest.raises(CassetteMiss):
        run_due(dollar(), settings(), replay, trigger=due(words="Another day began."))
