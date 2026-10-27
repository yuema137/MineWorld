"""The layered consult engine (pr-s10-p6 §§5.5.3, 5.6, C9).

`consult` is a pure function of its input and its dependencies: every model call goes through `deps.ask`, every
submit through `deps.submit`, every sleep through `deps.sleep`, and jitter is seeded per (seat, trigger, layer). It
never submits an unvalidated proposal (`ARC-72`). It ends in exactly one terminal outcome:

    acted        a request was submitted and accepted
    chose        the proposal was "nothing" (valid)
    fell_back    a layer ran out, a failure was not retryable, the cap or the deadline was hit (cause given)
    budget       the gateway refused the call (not retried, not counted by the breaker)
    disconnected the seat ended or was lost mid-consult (not counted by the breaker)
    dropped      a newer line arrived before the submit, so the proposal is not sent (not counted)

A frame refusal (`paused`) is `fell_back` with cause `frame`, which the breaker does not count. A `rate_limited`
refusal is the one exception: the same request is sent again after the transport layer's wait (D-P6b-35).
"""

from __future__ import annotations

import hashlib
import random
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Literal

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import CognitionModel, CompletionRequest
from mineworld_cognition.controller.attributes import NO_ATTRIBUTES, SelfAttributes
from mineworld_cognition.controller.breaker import BreakerOutcome
from mineworld_cognition.controller.choice import ContentFailure, SocialChoice, parse_choice
from mineworld_cognition.controller.failures import (
    RATE_LIMITED,
    FailureRecord,
    from_content,
    from_gate,
    from_refused,
    from_semantic,
)
from mineworld_cognition.controller.fill.base import Fillable
from mineworld_cognition.controller.intent import SemanticFailure, build
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Feedback, Reask, Replan, Trigger, build_request
from mineworld_cognition.controller.retry import BoundRetry
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.controller.validate import Proposal, validate
from mineworld_cognition.gateway import Completed, Failed, GateOutcome, Refused
from mineworld_cognition.memory.retrieve import MemorySection

LAYERS = ("transport", "content", "semantic")
Layer = Literal["transport", "content", "semantic"]
ConsultOutcome = Literal["acted", "chose", "fell_back", "budget", "disconnected", "dropped"]
Cause = Literal[
    "transport", "content", "semantic", "deadline", "cap", "budget", "disconnected", "frame"
]


@dataclass(frozen=True)
class NewestState:
    """The newest situation and observation, read at the moment of a rebuild or of the intent (step-17 §3.6)."""

    situation: Situation
    observation: Observation


@dataclass(frozen=True)
class ConsultLimits:
    deadline_s: float
    max_calls: int


@dataclass(frozen=True)
class LayerPolicies:
    transport: BoundRetry
    content: BoundRetry
    semantic: BoundRetry


@dataclass(frozen=True)
class ConsultDeps:
    ask: Callable[[CompletionRequest, float], Awaitable[GateOutcome]]
    """The gateway call, with the remaining deadline as its timeout."""
    newest: Callable[[], NewestState]
    current: Callable[[], bool]
    """False when a newer line from the trigger's speaker arrived after the consult's basis (dropped)."""
    submit: Callable[[ActionRequest], Awaitable[Answered | RefusedRequest]]
    clock: Callable[[], float]
    """Monotonic wall seconds."""
    sleep: Callable[[float], Awaitable[None]]
    policies: LayerPolicies
    limits: ConsultLimits


@dataclass(frozen=True)
class ConsultInput:
    seat: EntityKey
    persona: Persona
    situation: Situation
    memory: MemorySection
    trigger: Trigger
    attributes: SelfAttributes = NO_ATTRIBUTES
    fillables: tuple[Fillable, ...] = ()
    """The fillers listed for this consult's observation (S31-d §5.8); none, and the consult is as before."""


class AttemptRecord(CognitionModel):
    layer: Literal["transport", "content", "semantic", "none"]
    key: str
    failure: FailureRecord | None
    text: str | None


class ConsultRecord(CognitionModel):
    outcome: ConsultOutcome
    cause: Cause | None
    attempts: tuple[AttemptRecord, ...]
    proposal: Proposal | None
    request: ActionRequest | None


def _jitter(seat: EntityKey, trigger: Trigger, layer: Layer) -> random.Random:
    """One stream per (seat, trigger, layer), reproducible and never global (§5.5.2)."""
    digest = hashlib.sha256(f"{seat}:{trigger.event}:{layer}".encode()).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def _end(
    outcome: ConsultOutcome,
    cause: Cause | None,
    attempts: list[AttemptRecord],
    proposal: Proposal | None = None,
    request: ActionRequest | None = None,
) -> ConsultRecord:
    """The one terminal record of a consult. The engine never leaves a consult without one (AP6-2)."""
    return ConsultRecord(
        outcome=outcome,
        cause=cause,
        attempts=tuple(attempts),
        proposal=proposal,
        request=request,
    )


async def consult(input: ConsultInput, deps: ConsultDeps) -> ConsultRecord:
    started = deps.clock()
    limits = deps.limits
    jitter = {layer: _jitter(input.seat, input.trigger, layer) for layer in LAYERS}
    counts: dict[Layer, int] = {"transport": 0, "content": 0, "semantic": 0}
    attempts: list[AttemptRecord] = []
    calls = 0
    situation = input.situation
    feedback: Feedback = ()
    bound: dict[Layer, BoundRetry] = {
        "transport": deps.policies.transport,
        "content": deps.policies.content,
        "semantic": deps.policies.semantic,
    }

    def remaining() -> float:
        return limits.deadline_s - (deps.clock() - started)

    async def retry_or_end(layer: Layer, failure: FailureRecord) -> ConsultRecord | None:
        """The layer's decision for a failed attempt: `None` to go on, or the terminal record."""
        counts[layer] += 1
        decision = bound[layer].decide(counts[layer], failure, jitter[layer])
        if not (failure.retryable and decision.retry):
            return _end("fell_back", layer, attempts)
        if calls >= limits.max_calls:
            return _end("fell_back", "cap", attempts)
        if decision.delay_s > remaining():
            return _end("fell_back", "deadline", attempts)
        await deps.sleep(decision.delay_s)
        return None

    while True:
        if remaining() <= 0:
            return _end("fell_back", "deadline", attempts)
        if calls >= limits.max_calls:
            return _end("fell_back", "cap", attempts)
        calls += 1
        request = build_request(
            input.seat,
            input.persona,
            situation,
            input.memory,
            input.trigger,
            feedback,
            attributes=input.attributes,
            fillables=input.fillables,
        )
        key = str(cassette_key(request))
        gate = await deps.ask(request, remaining())

        if isinstance(gate, Refused):
            failure = from_gate(gate)
            attempts.append(AttemptRecord(layer="none", key=key, failure=failure, text=None))
            return _end("budget", "budget", attempts)

        if isinstance(gate, Failed):
            failure = from_gate(gate)
            attempts.append(AttemptRecord(layer="transport", key=key, failure=failure, text=None))
            ended = await retry_or_end("transport", failure)
            if ended is not None:
                return ended
            continue

        assert isinstance(gate, Completed)
        text = gate.completion.text
        parsed = parse_choice(gate.completion, fillers=bool(input.fillables))
        proposal: Proposal | ContentFailure
        if isinstance(parsed, SocialChoice):
            proposal = validate(parsed, situation, input.memory, input.fillables)
        else:
            proposal = parsed
        if isinstance(proposal, ContentFailure):
            failure = from_content(proposal)
            attempts.append(AttemptRecord(layer="content", key=key, failure=failure, text=text))
            feedback = (
                *feedback,
                Reask(kind="reask", previous=text, reason=proposal.reason, detail=proposal.detail),
            )
            ended = await retry_or_end("content", failure)
            if ended is not None:
                return ended
            continue

        newest = deps.newest()
        built = build(proposal, newest.observation)
        if built is None:
            attempts.append(AttemptRecord(layer="none", key=key, failure=None, text=text))
            return _end("chose", None, attempts, proposal=proposal)
        if isinstance(built, SemanticFailure):
            failure = from_semantic(built)
            attempts.append(AttemptRecord(layer="semantic", key=key, failure=failure, text=text))
            situation = deps.newest().situation
            feedback = (*feedback, Replan(kind="replan", rejection=built.reason))
            ended = await retry_or_end("semantic", failure)
            if ended is not None:
                return ended
            continue

        while True:
            if not deps.current():
                attempts.append(AttemptRecord(layer="none", key=key, failure=None, text=text))
                return _end("dropped", None, attempts, proposal=proposal)
            try:
                answer = await deps.submit(built)
            except MineWorldError:
                return _end("disconnected", "disconnected", attempts, proposal=proposal)
            if not (isinstance(answer, RefusedRequest) and answer.code == RATE_LIMITED):
                break
            # The seat's request budget is spent (S27-a): wait as the transport layer says, at least
            # `retry_after_ms`, and send the same request again. No model call, so the call cap does not
            # apply; running out ends like any frame refusal, which the breaker does not count (D-P6b-35).
            failure = from_refused(answer)
            attempts.append(AttemptRecord(layer="transport", key=key, failure=failure, text=text))
            counts["transport"] += 1
            decision = bound["transport"].decide(counts["transport"], failure, jitter["transport"])
            if not decision.retry or decision.delay_s > remaining():
                return _end("fell_back", "frame", attempts, proposal=proposal)
            await deps.sleep(decision.delay_s)
        if isinstance(answer, RefusedRequest):
            attempts.append(
                AttemptRecord(layer="none", key=key, failure=from_refused(answer), text=text)
            )
            return _end("fell_back", "frame", attempts, proposal=proposal)
        if isinstance(answer.result, Accepted):
            attempts.append(AttemptRecord(layer="none", key=key, failure=None, text=text))
            return _end("acted", None, attempts, proposal=proposal, request=built)
        reason = "unavailable" if answer.result == "unavailable" else "rejected"
        failure = from_semantic(
            SemanticFailure(reason=reason, detail="the world refused the request")
        )
        attempts.append(AttemptRecord(layer="semantic", key=key, failure=failure, text=text))
        situation = deps.newest().situation
        feedback = (*feedback, Replan(kind="replan", rejection=reason))
        ended = await retry_or_end("semantic", failure)
        if ended is not None:
            return ended


def breaker_outcome(record: ConsultRecord) -> BreakerOutcome | None:
    """What the breaker counts from a consult (§5.6): `None` for the outcomes the model is not at fault for."""
    if record.outcome in ("acted", "chose"):
        return "success"
    if record.outcome in ("budget", "disconnected", "dropped"):
        return None
    if record.cause == "frame":
        return None
    return "failure"
