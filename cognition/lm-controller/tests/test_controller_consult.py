"""The layered consult engine over scripted answers (pr-s10-p6 §§5.5.3, 5.6, C9; AP6-2, AP6-4, M-P6-2, M-P6-3, M-P6-5).

Every outcome has a test. The clock and the sleep are fakes: a sleep advances the fake clock, and nothing waits.
The 1 000-consult run uses the seeded garbage generator with 30 % garbage and no transport failures, and asserts the
fallback share bound (AP6-4): the expected share is 0.3 cubed, 2.7 %, so the bound of 5 % has room.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable

from garbage import as_gate, garbage_script
from memory_fixtures import ALICE, BOB
from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation, Rejected
from mineworld_sdk.wire.ids import ActionId, ActionTypeId, EntityKey, EventId
from support import run

from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    Finish,
    Usage,
)
from mineworld_cognition.budget import BudgetRefusal
from mineworld_cognition.config import BUNDLED_RETRY
from mineworld_cognition.controller.breaker import BreakerOutcome
from mineworld_cognition.controller.consult import (
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    ConsultRecord,
    LayerPolicies,
    NewestState,
    breaker_outcome,
    consult,
)
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Offer, Person, Situation
from mineworld_cognition.gateway import Failed, GateOutcome, Refused
from mineworld_cognition.memory.retrieve import MemorySection

ANSWER_NOTHING = '{"act": "nothing", "to": null, "words": "", "offer": null, "recalls": []}'
ANSWER_SAY = '{"act": "say", "to": "Bob", "words": "hello", "offer": null, "recalls": []}'

Ask = Callable[[CompletionRequest, float], Awaitable[GateOutcome]]
Submit = Callable[[ActionRequest], Awaitable[Answered | RefusedRequest]]


class FakeClock:
    """Monotonic fake time: `sleep` advances it and records each sleep, so no test waits."""

    def __init__(self) -> None:
        self.now = 0.0
        self.sleeps: list[float] = []

    def clock(self) -> float:
        return self.now

    async def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds


def _bound(layer: str, bundled: str) -> BoundRetry:
    default = BUNDLED_RETRY[layer]  # type: ignore[index]
    return RetryRegistry.installed().bind(default.policy, dict(default.parameters), bundled)


def _policies() -> LayerPolicies:
    return LayerPolicies(
        transport=_bound("transport", "transport"),
        content=_bound("content", "content"),
        semantic=_bound("semantic", "semantic"),
    )


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


def _input(event: str = "1907") -> ConsultInput:
    return ConsultInput(
        seat=EntityKey("alice"),
        persona=Persona(),
        situation=_situation(),
        memory=MemorySection(lines=()),
        trigger=Trigger(event=EventId(event), kind="spoken_to", speaker=BOB, words="hello"),
    )


def _accepted() -> Answered:
    return Answered(
        action_id=ActionId("1"), result=Accepted.model_validate({"accepted": {"events": []}})
    )


def _rejected() -> Answered:
    return Answered(action_id=ActionId("1"), result=Rejected.model_validate({"rejected": "busy"}))


def _completion(text: str) -> Completion:
    finish: Finish = "complete"
    return Completion(
        text=text, finish=finish, usage=Usage(input_tokens=1, output_tokens=1, estimated=False)
    )


def _answer_ask(answer: str | BackendFailure, seen: list[CompletionRequest] | None = None) -> Ask:
    async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
        if seen is not None:
            seen.append(request)
        if isinstance(answer, BackendFailure):
            return Failed(answer)
        return as_gate(_completion(answer))

    return ask


def _accepting(log: list[ActionRequest] | None = None) -> Submit:
    async def submit(request: ActionRequest) -> Answered | RefusedRequest:
        if log is not None:
            log.append(request)
        return _accepted()

    return submit


def _deps(
    clock: FakeClock,
    ask: Ask,
    *,
    submit: Submit | None = None,
    limits: ConsultLimits | None = None,
    current: bool = True,
) -> ConsultDeps:
    state = NewestState(situation=_situation(), observation=_observation())
    return ConsultDeps(
        ask=ask,
        newest=lambda: state,
        current=lambda: current,
        submit=submit or _accepting(),
        clock=clock.clock,
        sleep=clock.sleep,
        policies=_policies(),
        limits=limits or ConsultLimits(deadline_s=20.0, max_calls=5),
    )


def _go(deps: ConsultDeps, event: str = "1907") -> ConsultRecord:
    return run(consult(_input(event), deps))


def test_a_valid_say_is_submitted_and_acted() -> None:
    record = _go(_deps(FakeClock(), _answer_ask(ANSWER_SAY)))
    assert record.outcome == "acted"
    assert record.request is not None and record.request.action_type == "talk"


def test_a_valid_nothing_is_chose_and_submits_nothing() -> None:
    submitted: list[ActionRequest] = []
    record = _go(_deps(FakeClock(), _answer_ask(ANSWER_NOTHING), submit=_accepting(submitted)))
    assert record.outcome == "chose"
    assert submitted == []


def test_transport_failures_exhaust_three_attempts_then_fall_back() -> None:
    clock = FakeClock()
    record = _go(_deps(clock, _answer_ask(BackendFailure(reason="unreachable"))))
    assert (record.outcome, record.cause) == ("fell_back", "transport")
    assert len(record.attempts) == 3
    assert len(clock.sleeps) == 2  # two retries, each slept through the injected sleep


def test_an_unauthorized_answer_is_not_retried() -> None:
    clock = FakeClock()
    record = _go(_deps(clock, _answer_ask(BackendFailure(reason="unauthorized"))))
    assert (record.outcome, record.cause) == ("fell_back", "transport")
    assert len(record.attempts) == 1
    assert clock.sleeps == []


def test_content_failures_exhaust_two_reasks_then_fall_back() -> None:
    record = _go(_deps(FakeClock(), _answer_ask("hello there")))
    assert (record.outcome, record.cause) == ("fell_back", "content")
    assert len(record.attempts) == 3  # the first answer and two re-asks


def test_semantic_refusals_exhaust_one_replan_then_fall_back() -> None:
    submits: list[ActionRequest] = []

    async def refuse(request: ActionRequest) -> Answered | RefusedRequest:
        submits.append(request)
        return _rejected()

    record = _go(_deps(FakeClock(), _answer_ask(ANSWER_SAY), submit=refuse))
    assert (record.outcome, record.cause) == ("fell_back", "semantic")
    assert len(submits) == 2  # the first submit and one re-plan


def test_the_call_cap_ends_the_consult_at_max_calls() -> None:
    seen: list[CompletionRequest] = []
    deps = _deps(
        FakeClock(),
        _answer_ask(BackendFailure(reason="unreachable"), seen),
        limits=ConsultLimits(deadline_s=20.0, max_calls=2),
    )
    record = _go(deps)
    assert (record.outcome, record.cause) == ("fell_back", "cap")
    assert len(seen) == 2


def test_a_slow_model_is_cut_by_the_deadline_and_no_sleep_passes_it() -> None:
    clock = FakeClock()

    async def slow(request: CompletionRequest, timeout: float) -> GateOutcome:
        clock.now += 30.0  # the call takes longer than the whole deadline
        return Failed(BackendFailure(reason="timeout"))

    record = _go(_deps(clock, slow))
    assert (record.outcome, record.cause) == ("fell_back", "deadline")
    assert all(seconds <= 20.0 for seconds in clock.sleeps)


def test_a_budget_refusal_is_budget_and_is_not_retried() -> None:
    refusal = BudgetRefusal(limit="calls_per_wall_hour", spent=20, asked=1, ceiling=20)

    async def refused(request: CompletionRequest, timeout: float) -> GateOutcome:
        return Refused(refusal)

    record = _go(_deps(FakeClock(), refused))
    assert record.outcome == "budget"
    assert len(record.attempts) == 1


def test_a_lost_seat_is_disconnected() -> None:
    async def lost(request: ActionRequest) -> Answered | RefusedRequest:
        raise MineWorldError("the seat is lost")

    record = _go(_deps(FakeClock(), _answer_ask(ANSWER_SAY), submit=lost))
    assert record.outcome == "disconnected"


def test_a_newer_line_before_the_submit_drops_the_proposal() -> None:
    submitted: list[ActionRequest] = []
    record = _go(
        _deps(FakeClock(), _answer_ask(ANSWER_SAY), submit=_accepting(submitted), current=False)
    )
    assert record.outcome == "dropped"
    assert submitted == []


def test_a_frame_refusal_falls_back_with_cause_frame_and_is_not_retried() -> None:
    async def paused(request: ActionRequest) -> Answered | RefusedRequest:
        return RefusedRequest(code="paused", detail=None)

    record = _go(_deps(FakeClock(), _answer_ask(ANSWER_SAY), submit=paused))
    assert (record.outcome, record.cause) == ("fell_back", "frame")
    assert len(record.attempts) == 1


def test_a_replan_is_built_with_the_refusal_as_feedback() -> None:
    seen: list[CompletionRequest] = []
    answers = iter([_rejected(), _accepted()])

    async def refuse_then_accept(request: ActionRequest) -> Answered | RefusedRequest:
        return next(answers)

    _go(_deps(FakeClock(), _answer_ask(ANSWER_SAY, seen), submit=refuse_then_accept))
    assert len(seen) == 2
    assert seen[1].messages[-1].text.startswith("The world refused your choice: rejected.")


def test_breaker_counts_success_and_failure_and_nothing_else() -> None:
    def record(outcome: str, cause: str | None) -> ConsultRecord:
        return ConsultRecord(
            outcome=outcome,  # type: ignore[arg-type]
            cause=cause,  # type: ignore[arg-type]
            attempts=(),
            proposal=None,
            request=None,
        )

    expected: list[tuple[ConsultRecord, BreakerOutcome | None]] = [
        (record("acted", None), "success"),
        (record("chose", None), "success"),
        (record("fell_back", "transport"), "failure"),
        (record("fell_back", "content"), "failure"),
        (record("fell_back", "semantic"), "failure"),
        (record("fell_back", "deadline"), "failure"),
        (record("budget", "budget"), None),
        (record("disconnected", "disconnected"), None),
        (record("dropped", None), None),
        (record("fell_back", "frame"), None),
    ]
    for sample, counted in expected:
        assert breaker_outcome(sample) == counted, (sample.outcome, sample.cause)


def test_seeded_thirty_percent_garbage_resolves_every_consult_within_bounds() -> None:
    """AP6-4: 1 000 consults; every one terminal; at most five calls each; fallback share at most 5 %."""
    script = garbage_script(
        seed=7, garbage_permille=300, failure_permille=0, valid_answer=ANSWER_NOTHING
    )

    async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
        return as_gate(script(request))

    fell_back = 0
    for index in range(1000):
        record = _go(_deps(FakeClock(), ask), event=str(10_000 + index))
        assert record.outcome in (
            "acted",
            "chose",
            "fell_back",
            "budget",
            "disconnected",
            "dropped",
        )
        assert len(record.attempts) <= 5
        if record.outcome == "fell_back":
            fell_back += 1
    assert fell_back * 100 <= 5 * 1000  # at most 5 %
