"""The assisted branch of the retention host (pr-fg-c §§5.1-5.3, C2), with test-only assisted policies.

No `lm-summarise` here: these policies are scripted in the test, and every model answer comes from a
`ScriptedBackend`. Each test runs the production path (`resolve`, `ModelGateway`, `SummaryGateway`, the
host through `absorb`). Oracles read the dump; the pass log is compared without its assisted columns, so a
fallback is shown to leave the plain policy's store behind (AFGc-5).
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import ClassVar, Literal, assert_never

import pytest
from memory_fixtures import alice_identity, all_rows, seat_view
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from pydantic import BaseModel, ConfigDict, Field
from retention_driver import built, days
from retention_stream import stream
from support import completion, run

from mineworld_cognition.backend.model import BackendFailure, CompletionRequest, Message, Sampling
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.forgetting.consolidate import Consolidate
from mineworld_cognition.gateway import Budget, Completed, Failed, ModelGateway, Refused
from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention import Drop, Keep, MemoryItem, Merge, RetentionBudget
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.assisted import (
    AssistedPlan,
    Exhausted,
    GroupAnswer,
    RequestTooLarge,
    StopReason,
    SummaryGateway,
    WrongPurpose,
)
from mineworld_cognition.memory.retention.policy import RetentionDecision
from mineworld_cognition.memory.retention.registry import PolicyRegistry, UnknownPolicy
from mineworld_cognition.memory.retention.selection import (
    FallbackNotAllowed,
    FallbackNotPlain,
    MemoryPolicyChoice,
    Retention,
    resolve,
)

SEED = 11
DAYS = 12
SAMPLING = Sampling(temperature_milli=0, max_output_tokens=96, seed=0)
ANSWER = '{"summary": "a short line", "importance": 9}'


class FakeClock:
    def now(self) -> int:
        return 1_760_000_000


class Parameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    calls: int = Field(default=3, ge=0, le=30)


class Asks:
    """Test-only: asks one summary per Merge group of its fallback's plan and keeps each group answered.
    It stops at the first refusal, failure or exhausted allowance. `HOSTILE` names an item outside the
    group instead; `SIZE` sets the request's size."""

    id: ClassVar[str] = "asks"
    fallback: ClassVar[str] = "consolidate"
    Parameters = Parameters
    SIZE: ClassVar[int] = 40
    HOSTILE: ClassVar[bool] = False

    def calls_per_pass(self, params: Parameters) -> int:
        return params.calls

    async def plan(
        self,
        items: Sequence[MemoryItem],
        budget: RetentionBudget,
        params: Parameters,
        gateway: SummaryGateway,
        fallback_plan: Sequence[RetentionDecision],
    ) -> AssistedPlan:
        by_id = {item.item: item for item in items}
        answers: list[GroupAnswer] = []
        stopped: StopReason | None = None
        for decision in fallback_plan:
            if not isinstance(decision, Merge):
                continue
            container = by_id[decision.items[0]].container
            assert container is not None
            request = CompletionRequest(
                purpose="summarize",
                messages=(Message(role="user", text="x" * self.SIZE),),
                sampling=SAMPLING,
            )
            outcome = await gateway.complete(request)
            match outcome:
                case Completed():
                    answers.append(
                        GroupAnswer(
                            container=container,
                            decisions=self._decisions(items, container, decision),
                            prose=None,
                            prose_key=None,
                        )
                    )
                case Refused():
                    stopped = "budget"
                case Failed():
                    stopped = "failed"
                case Exhausted(calls=calls):
                    stopped = "no-calls" if calls == 0 else "exhausted"
                case _:
                    assert_never(outcome)
            if stopped is not None:
                break
        return AssistedPlan(answers=tuple(answers), stopped=stopped, invalid=0)

    def _decisions(
        self, items: Sequence[MemoryItem], container: ItemId, merge: Merge
    ) -> tuple[RetentionDecision, ...]:
        if self.HOSTILE:
            outside = next(i.item for i in items if i.container is None and i.item != container)
            return (Drop(item=outside),)
        return (Keep(item=container), *(Keep(item=each) for each in merge.items))


class TooBig(Asks):
    id: ClassVar[str] = "too-big"
    SIZE: ClassVar[int] = 6_001


class Hostile(Asks):
    id: ClassVar[str] = "hostile"
    HOSTILE: ClassVar[bool] = True


class Forgets(Asks):
    """Test-only: the model "chooses" to forget. Every group whose container is top-level is answered with
    `Drop(container)` (the drop cell of pr-fg-c §5.4), whatever it holds; other groups are kept."""

    id: ClassVar[str] = "forgets"

    def _decisions(
        self, items: Sequence[MemoryItem], container: ItemId, merge: Merge
    ) -> tuple[RetentionDecision, ...]:
        if next(i for i in items if i.item == container).container is None:
            return (Drop(item=container),)
        return super()._decisions(items, container, merge)


REGISTRY = PolicyRegistry(
    {
        "asks": Asks,
        "too-big": TooBig,
        "hostile": Hostile,
        "forgets": Forgets,
        "consolidate": Consolidate,
    }
)


def _gateway(backend: ScriptedBackend, **policy: int) -> ModelGateway:
    return ModelGateway(backend, Budget.of(BudgetPolicy(**policy), MemoryLedger(), FakeClock()))


def _answering() -> ScriptedBackend:
    return ScriptedBackend(lambda _request: completion(ANSWER))


def _retention(policy: str, gateway: ModelGateway | None, **parameters: int) -> Retention:
    choice = MemoryPolicyChoice.model_validate({"policy": policy, "parameters": parameters})
    return resolve(choice, REGISTRY, gateway)


def _facts() -> list[PerceivedEvent]:
    return list(stream(SEED, DAYS))


def _rows(dump: bytes) -> list[dict[str, object]]:
    return all_rows(dump)


def _plain_rows(dump: bytes) -> list[dict[str, object]]:
    """Every row of the store except the pass log. The log is compared by `_passes`: its byte counts
    include its own rows, so it differs by the two assisted columns alone (AFGc-5)."""
    return [row for row in _rows(dump) if row["table"] != "retention_passes"]


def _passes(dump: bytes) -> list[tuple[str, int]]:
    """(outcome, model_decisions) of every pass, in day order."""
    return [
        (str(row["outcome"]), int(str(row["model_decisions"])))
        for row in _rows(dump)
        if row["table"] == "retention_passes"
    ]


def _consolidated(facts: Sequence[PerceivedEvent]) -> bytes:
    return built(facts, resolve(MemoryPolicyChoice(policy="consolidate"), REGISTRY), 256)


def _seat() -> EntityKey:
    return alice_identity().seat


def test_a_request_over_the_bound_never_reaches_the_backend() -> None:
    # AFGc-1 (the 6 001-byte case). Its mutation removes the byte check from SummaryGateway: the backend
    # is called, and the plain-rows oracle still holds only if the answer is refused.
    facts = _facts()
    backend = _answering()
    dump = built(facts, _retention("too-big", _gateway(backend)), 256)
    assert backend.calls == 0
    outcomes = [outcome for outcome, _ in _passes(dump)]
    assert "fallback:raised: RequestTooLarge" in outcomes
    assert _plain_rows(dump) == _plain_rows(_consolidated(facts))


def test_the_gateway_checks_each_request_before_the_backend() -> None:
    # The bound is 6 000 bytes of message text: at the bound it is sent; over it, or for another purpose,
    # it is refused with the backend untouched (FGc-2).
    backend = _answering()
    summary = SummaryGateway(_gateway(backend), _seat(), calls_left=3)

    def request(
        size: int, purpose: Literal["summarize", "decide"] = "summarize"
    ) -> CompletionRequest:
        return CompletionRequest(
            purpose=purpose,
            messages=(Message(role="user", text="a" * size),),
            sampling=SAMPLING,
        )

    assert isinstance(run(summary.complete(request(6_000))), Completed)
    with pytest.raises(RequestTooLarge):
        run(summary.complete(request(6_001)))
    with pytest.raises(WrongPurpose):
        run(summary.complete(request(10, purpose="decide")))
    assert backend.calls == 1
    assert summary.calls_made == 1


def test_the_allowance_is_spent_by_the_gateway_and_returned_not_raised() -> None:
    backend = _answering()
    summary = SummaryGateway(_gateway(backend), _seat(), calls_left=2)
    request = CompletionRequest(
        purpose="summarize", messages=(Message(role="user", text="a"),), sampling=SAMPLING
    )
    outcomes = [run(summary.complete(request)) for _ in range(3)]
    assert [type(each).__name__ for each in outcomes] == ["Completed", "Completed", "Exhausted"]
    assert backend.calls == 2


def test_the_registry_refuses_a_bad_fallback_by_name() -> None:
    # AFGc-2 (the selection half). Its mutation accepts an assisted fallback: the first case passes.
    with pytest.raises(FallbackNotPlain, match="too-big"):
        resolve(
            MemoryPolicyChoice.model_validate({"policy": "asks", "fallback": "too-big"}),
            REGISTRY,
            None,
        )
    with pytest.raises(UnknownPolicy, match="nope"):
        resolve(
            MemoryPolicyChoice.model_validate({"policy": "asks", "fallback": "nope"}),
            REGISTRY,
            None,
        )
    with pytest.raises(FallbackNotAllowed):
        resolve(
            MemoryPolicyChoice.model_validate({"policy": "consolidate", "fallback": "consolidate"}),
            REGISTRY,
            None,
        )
    assert _retention("asks", None).fallback is not None


def test_a_fallback_stands_whole_when_no_answer_is_used() -> None:
    # AFGc-5: no allowance; no bound tier; a backend that fails every call. Each pass's log says why, and
    # the store is the plain policy's store.
    facts = _facts()
    expected = _plain_rows(_consolidated(facts))
    no_calls = _retention("asks", _gateway(_answering()), calls=0)
    unbound = _retention("asks", None)
    failing = _retention(
        "asks", _gateway(ScriptedBackend(lambda _r: BackendFailure(reason="unreachable")))
    )
    no_calls_dump = built(facts, no_calls, 256)
    assert _plain_rows(no_calls_dump) == expected
    assert {(o, m) for o, m in _passes(no_calls_dump)} == {("fallback:no-calls", 0)}
    unbound_dump = built(facts, unbound, 256)
    assert _plain_rows(unbound_dump) == expected
    assert {(o, m) for o, m in _passes(unbound_dump)} == {("fallback:unbound", 0)}
    failing_dump = built(facts, failing, 256)
    assert _plain_rows(failing_dump) == expected
    outcomes = {o for o, _ in _passes(failing_dump)}
    assert "fallback:failed" in outcomes
    assert outcomes <= {"fallback:failed", "assisted"}
    assert all(model == 0 for _, model in _passes(failing_dump))


def test_a_refused_call_stops_the_pass_and_is_recorded() -> None:
    # AFGc-5 (the budget case): one call an hour, so the second is refused. The first answer stands and
    # is the only one counted as a model decision.
    facts = _facts()
    backend = _answering()
    dump = built(facts, _retention("asks", _gateway(backend, calls_per_wall_hour=1)), 256)
    assert backend.calls == 1
    answered = [each for each in _passes(dump) if each[1] > 0]
    assert [outcome for outcome, _ in answered] == ["fallback:budget"]


def test_the_pass_never_exceeds_its_declared_allowance() -> None:
    # AFGc-6 (the bound half): every pass's backend calls are at most the declared allowance.
    facts = _facts()
    backend = _answering()
    kept = _retention(
        "asks",
        _gateway(backend, calls_per_wall_hour=10_000_000, tokens_per_wall_day=1_000_000_000),
        calls=3,
    )
    per_pass: list[int] = []
    with seat_view() as store:
        for frame in days(facts):
            before = backend.calls
            run(absorb(store, frame, frame[-1].id, kept), timeout_s=120)
            per_pass.append(backend.calls - before)
    assert max(per_pass) <= 3
    assert max(per_pass) > 0


def test_a_hostile_answer_is_discarded_and_its_group_falls_back() -> None:
    # AFGc-3's mutation: an answer naming an item outside its group is discarded, so the store is the
    # plain policy's, and the pass records `invalid`. Without the host's group check the outside item is
    # dropped and the store differs.
    facts = _facts()
    generous = {"calls_per_wall_hour": 10_000_000, "tokens_per_wall_day": 1_000_000_000}
    dump = built(facts, _retention("hostile", _gateway(_answering(), **generous), calls=24), 256)
    assert _plain_rows(dump) == _plain_rows(_consolidated(facts))
    assert "fallback:invalid" in {outcome for outcome, _ in _passes(dump)}
    assert all(model == 0 for _, model in _passes(dump))


def test_frames_do_not_change_the_store_of_an_assisted_policy() -> None:
    # The host reads facts, not frames (FGb-5 with an assisted policy): frames of 1 and of 64 agree.
    # Each build gets its own gateway: a ledger is state, and a shared one would refuse the second build.
    facts = _facts()[:600]
    one = built(facts, _retention("asks", _gateway(_answering()), calls=2), 1)
    many = built(facts, _retention("asks", _gateway(_answering()), calls=2), 64)
    assert one == many


def _direct_exchanges(dump: bytes) -> set[str]:
    """The `spoke` records a store holds whose only counterpart is one person: lines exchanged directly."""
    rows = _rows(dump)
    counterparts: dict[str, int] = {}
    for row in rows:
        if row["table"] == "l0_who":
            event = str(row["event_id"])
            counterparts[event] = counterparts.get(event, 0) + 1
    return {
        str(row["event_id"])
        for row in rows
        if row["table"] == "l0"
        and row["kind"] == "spoke"
        and counterparts.get(str(row["event_id"])) == 1
    }


def test_a_model_that_drops_a_direct_conversation_is_overridden_by_the_host() -> None:
    # F-FGd-2, ruled host-enforced (operator, 2026-10-18; pr-p6c-fgd §9): under an assisted strategy the
    # memories of the people the seat talked to directly are the host's to protect. The scripted model drops
    # every top-level group; the host overrides each answer that would drop a direct exchange, so every
    # keepsake `consolidate` keeps is still a record, while answers that drop nothing protected stand. Its
    # mutation (the host's `_forgets` filter removed) loses keepsakes.
    facts = _facts()
    generous = {"calls_per_wall_hour": 10_000_000, "tokens_per_wall_day": 1_000_000_000}
    dump = built(facts, _retention("forgets", _gateway(_answering(), **generous), calls=24), 256)
    kept = _direct_exchanges(_consolidated(facts))
    assert kept, "the stream holds direct exchanges that consolidate keeps"
    assert kept <= _direct_exchanges(dump)
    assert all(outcome != "fallback:invalid" for outcome, _ in _passes(dump))
