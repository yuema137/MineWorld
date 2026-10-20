"""The decision log, counters and logged-input replay (pr-s10-p6 §§5.11, 5.12, C10; AP6-8 replay half).

The replay test records twenty consults through a real `RecordingBackend` over the seeded garbage script, writes their
decision log, and replays it with `replay_log`: no server, no model, no network. A planted fake key and a planted
resume secret must appear in no file the controller writes.
"""

from __future__ import annotations

import dataclasses
import json
from pathlib import Path

import pytest
from garbage import garbage_script
from memory_fixtures import ALICE, BOB
from mineworld_sdk.session import Answered
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation
from mineworld_sdk.wire.ids import ActionId, ActionTypeId, EntityKey, EventId
from support import run

from mineworld_cognition.__main__ import _status  # pyright: ignore[reportPrivateUsage]
from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.config import BUNDLED_RETRY
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.consult import (
    AttemptRecord,
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    ConsultRecord,
    LayerPolicies,
    NewestState,
    consult,
)
from mineworld_cognition.controller.failures import FailureRecord
from mineworld_cognition.controller.log import (
    Counters,
    DecisionInput,
    DecisionLog,
    DecisionRecord,
    counters_from,
    read_decisions,
    replay_log,
)
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Offer, Person, Situation
from mineworld_cognition.gateway import Budget, GateOutcome, ModelGateway
from mineworld_cognition.memory.retention.host import PassReport
from mineworld_cognition.memory.retrieve import MemorySection
from mineworld_cognition.record import RecordingBackend

ANSWER_SAY = '{"act": "say", "to": "Bob", "words": "hello", "offer": null, "recalls": []}'
PLANTED_KEY = "sk-test-PLANTED-value"
PLANTED_ENV = "MINEWORLD_TEST_PLANTED_KEY"


class Clock:
    def now(self) -> int:
        return 0


def _situation() -> Situation:
    return Situation(
        me="Alice",
        place="Cafe",
        at=4112,
        present=(Person(label="Bob", entity=BOB),),
        talk_to=("Bob",),
        offers=(Offer(number=1, action_type=ActionTypeId("order"), target_label=None, position=2),),
    )


def _observation() -> Observation:
    talk = {
        "action_type": "talk",
        "target": BOB,
        "available": True,
        "unavailable_reason": None,
        "requirement": {
            "place": "any",
            "within_range": None,
            "requires_line_of_access": False,
            "requires_target_available": True,
        },
    }
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": 4112,
            "self_location": None,
            "entities": [],
            "relations": [],
            "events": [],
            "affordances": [talk],
        }
    )


def _bound(layer: str) -> BoundRetry:
    default = BUNDLED_RETRY[layer]  # type: ignore[index]
    return RetryRegistry.installed().bind(default.policy, dict(default.parameters), layer)


def _policies() -> LayerPolicies:
    return LayerPolicies(
        transport=_bound("transport"), content=_bound("content"), semantic=_bound("semantic")
    )


def _live_gateway(recorder: RecordingBackend) -> ModelGateway:
    budget = Budget.of(
        BudgetPolicy(calls_per_wall_hour=1_000_000, tokens_per_wall_day=1_000_000_000),
        MemoryLedger(),
        Clock(),
    )
    return ModelGateway(recorder, budget)


def _record_one(index: int, gateway: ModelGateway, submits: list[str]) -> DecisionRecord:
    situation = _situation()
    observation = _observation()
    trigger = Trigger(
        event=EventId(str(20_000 + index)), kind="spoken_to", speaker=BOB, words="hello"
    )
    memory = MemorySection(lines=())

    async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
        return await gateway.complete(EntityKey("alice"), request, timeout_s=timeout)

    async def submit(request: ActionRequest) -> Answered:
        submits.append("accepted")
        return Answered(
            action_id=ActionId("1"), result=Accepted.model_validate({"accepted": {"events": []}})
        )

    deps = ConsultDeps(
        ask=ask,
        newest=lambda: NewestState(situation=situation, observation=observation),
        current=lambda: True,
        submit=submit,
        clock=lambda: 0.0,
        sleep=_no_sleep,
        policies=_policies(),
        limits=ConsultLimits(deadline_s=20.0, max_calls=5),
    )
    result: ConsultRecord = run(
        consult(
            ConsultInput(
                seat=EntityKey("alice"),
                persona=Persona(),
                situation=situation,
                memory=memory,
                trigger=trigger,
            ),
            deps,
        )
    )
    return DecisionRecord(
        seat="alice",
        consult=index + 1,
        input=DecisionInput(
            persona=Persona(),
            situation=situation,
            memory=memory,
            trigger=trigger,
            observation=observation,
        ),
        record=result,
        answers=tuple(submits),
    )


async def _no_sleep(seconds: float) -> None:
    return None


def test_rotation_keeps_one_older_file_and_no_more(tmp_path: Path) -> None:
    """Rotation starts a new file once the next line would pass the bound; one older file is kept, never two."""
    path = tmp_path / "alice.decisions.jsonl"
    older = Path(f"{path}.1")
    log = DecisionLog(path, rotate_bytes=2048)
    submits: list[str] = []
    gateway = _live_gateway(
        RecordingBackend(
            ScriptedBackend(
                garbage_script(
                    seed=1, garbage_permille=0, failure_permille=0, valid_answer=ANSWER_SAY
                )
            ),
            tmp_path / "unused.jsonl",
            binding="local",
            model="test-model",
        )
    )
    for index in range(12):
        submits.clear()
        log.append(_record_one(index, gateway, submits))
    longest_line = max(
        len(line.encode("utf-8")) + 1
        for written in (path, older)
        for line in written.read_text(encoding="utf-8").splitlines()
    )
    assert older.exists()
    assert not Path(f"{path}.2").exists()
    # The active file never passes the bound unless one line alone does.
    assert path.stat().st_size <= max(2048, longest_line)
    # Two files hold fewer than all twelve records: the oldest were dropped with the rotation.
    assert len(read_decisions(older)) + len(read_decisions(path)) < 12


def test_counters_count_calls_refusals_and_fallbacks_as_the_engine_ended_them() -> None:
    """model_calls: a budget refusal made no call; a disconnect did (its submit failed after the call)."""
    timeout = FailureRecord(cls="transport", reason="timeout", retryable=True)
    budget = FailureRecord(cls="budget", reason="budget", retryable=False)

    def decision(index: int, record: ConsultRecord, answers: tuple[str, ...]) -> DecisionRecord:
        return DecisionRecord(
            seat="alice",
            consult=index,
            input=DecisionInput(
                persona=Persona(),
                situation=_situation(),
                memory=MemorySection(lines=()),
                trigger=Trigger(event=EventId(str(index)), kind="arrived", speaker=None, words=""),
                observation=_observation(),
            ),
            record=record,
            answers=answers,
        )

    timed_out = ConsultRecord(
        outcome="fell_back",
        cause="transport",
        attempts=(AttemptRecord(layer="transport", key="k1", failure=timeout, text=None),),
        proposal=None,
        request=None,
    )
    refused = ConsultRecord(
        outcome="budget",
        cause="budget",
        attempts=(AttemptRecord(layer="none", key="k2", failure=budget, text=None),),
        proposal=None,
        request=None,
    )
    rejected = ConsultRecord(
        outcome="fell_back",
        cause="semantic",
        attempts=(
            AttemptRecord(
                layer="semantic",
                key="k3",
                failure=FailureRecord(cls="semantic", reason="rejected", retryable=True),
                text="x",
            ),
        ),
        proposal=None,
        request=None,
    )
    disconnected = ConsultRecord(
        outcome="disconnected", cause="disconnected", attempts=(), proposal=None, request=None
    )
    counted = counters_from(
        [
            decision(1, timed_out, ()),
            decision(2, refused, ()),
            decision(3, rejected, ("rejected",)),
            decision(4, disconnected, ()),
        ]
    )
    assert counted == Counters(
        consults=4, fallbacks=4, timeouts=1, server_refusals=1, model_calls=3
    )


def test_counters_recomputed_from_the_records_equal_the_counts() -> None:
    record = ConsultRecord(outcome="chose", cause=None, attempts=(), proposal=None, request=None)
    records = [
        DecisionRecord(
            seat="alice",
            consult=index + 1,
            input=DecisionInput(
                persona=Persona(),
                situation=_situation(),
                memory=MemorySection(lines=()),
                trigger=Trigger(
                    event=EventId(str(index + 1)), kind="arrived", speaker=None, words=""
                ),
                observation=_observation(),
            ),
            record=record,
            answers=(),
        )
        for index in range(3)
    ]
    counted = counters_from(records)
    assert counted == Counters(consults=3, chose_nothing=3, model_calls=0)


def test_a_recorded_run_replays_with_no_mismatch_and_plants_leak_nothing(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """AP6-8, replay half. The planted fake key sits in the environment while the run writes its files; it must
    appear in none of them. (The resume secret is planted and scanned from C11, the first commit that writes one.)"""
    monkeypatch.setenv(PLANTED_ENV, PLANTED_KEY)
    cassette_path = tmp_path / "cassette.jsonl"
    log_path = tmp_path / "decisions.jsonl"
    script = garbage_script(
        seed=3, garbage_permille=300, failure_permille=0, valid_answer=ANSWER_SAY
    )
    recorder = RecordingBackend(
        ScriptedBackend(script), cassette_path, binding="local", model="test-model"
    )
    gateway = _live_gateway(recorder)
    log = DecisionLog(log_path)
    submits: list[str] = []
    for index in range(20):
        submits.clear()
        log.append(_record_one(index, gateway, submits))
    run(gateway.aclose())
    report = run(replay_log(log_path, cassette_path))
    assert report.checked == 20
    assert report.mismatches == ()
    for written in (log_path, cassette_path):
        assert PLANTED_KEY not in written.read_text(encoding="utf-8")


def test_the_decision_lines_read_back_as_written(tmp_path: Path) -> None:
    log_path = tmp_path / "decisions.jsonl"
    DecisionLog(log_path).append(
        DecisionRecord(
            seat="alice",
            consult=1,
            input=DecisionInput(
                persona=Persona(),
                situation=_situation(),
                memory=MemorySection(lines=()),
                trigger=Trigger(event=EventId("7"), kind="spoken_to", speaker=BOB, words="hi"),
                observation=_observation(),
            ),
            record=ConsultRecord(
                outcome="chose", cause=None, attempts=(), proposal=None, request=None
            ),
            answers=(),
        )
    )
    [read] = read_decisions(log_path)
    assert read.consult == 1 and read.record.outcome == "chose"


def test_pass_lines_carry_every_report_field_and_leave_readers_unchanged(tmp_path: Path) -> None:
    # pr-s28f §5.7, AC-f-7: one `pass` line per report, in order; `read_decisions` and the status line read
    # only their own kinds.
    reports = [
        PassReport(
            day=day,
            policy="lm-summarise",
            outcome="fallback:budget" if day == 2 else "assisted",
            kept=10 + day,
            merged=day,
            dropped=1,
            forced=0,
            bytes_before=4096,
            bytes_after=2048,
            forgotten_ranges=3,
            model_decisions=5,
            fallback_decisions=day,
            cause="day",
            trigger=f"{day}00",
            prose_contradictions=0,
        )
        for day in (1, 2)
    ]
    log_path = tmp_path / "alice.decisions.jsonl"
    log = DecisionLog(log_path)
    log.passes("alice", reports)
    log.counters(Counters(consults=4))
    loaded = [json.loads(line) for line in log_path.read_text(encoding="utf-8").splitlines()]
    passes = [line for line in loaded if line["kind"] == "pass"]
    assert passes == [
        {"kind": "pass", "seat": "alice", **dataclasses.asdict(report)} for report in reports
    ]
    assert read_decisions(log_path) == []
    assert _status(tmp_path, LmSeat(key=EntityKey("alice"), memory=None)).startswith(
        "[cognition] alice: consults 4,"
    )
