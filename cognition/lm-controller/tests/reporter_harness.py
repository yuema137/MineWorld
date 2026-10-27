"""One `due` consult of a price clerk through the shipped consult engine, with a fake clock and a fake world.

The model is whatever backend the test passes (scripted, recording or replay); the world accepts every submit
unless the test passes another answer. The fillers are listed from the observation exactly as the seat runner
lists them (`fill.listed`), and the situation is `Situation.of` the same observation.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable

from memory_fixtures import seat_view
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation
from mineworld_sdk.wire.ids import ActionId, EntityKey, EventId
from support import run

from mineworld_cognition.backend.model import BackendFailure, CompletionRequest, ModelBackend
from mineworld_cognition.config import BUNDLED_RETRY, FillConfig
from mineworld_cognition.controller import fill
from mineworld_cognition.controller.consult import (
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    ConsultRecord,
    LayerPolicies,
    NewestState,
    consult,
)
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.gateway import Completed, Failed, GateOutcome
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection

Submit = Callable[[ActionRequest], Awaitable[Answered | RefusedRequest]]
DUE_EVENT = EventId("9001")


def due(event: EventId = DUE_EVENT, words: str = "A new day began.") -> Trigger:
    return Trigger(event=event, kind="due", speaker=None, words=words)


def accepted() -> Answered:
    return Answered(
        action_id=ActionId("1"), result=Accepted.model_validate({"accepted": {"events": []}})
    )


async def accept(request: ActionRequest) -> Answered:
    del request
    return accepted()


def _policies() -> LayerPolicies:
    def bound(layer: str) -> BoundRetry:
        default = BUNDLED_RETRY[layer]  # type: ignore[index]
        return RetryRegistry.installed().bind(default.policy, dict(default.parameters), layer)

    return LayerPolicies(
        transport=bound("transport"), content=bound("content"), semantic=bound("semantic")
    )


class Clock:
    def __init__(self) -> None:
        self.now = 0.0

    def clock(self) -> float:
        return self.now

    async def sleep(self, seconds: float) -> None:
        self.now += seconds


def run_due(
    observation: Observation,
    settings: FillConfig,
    backend: ModelBackend,
    *,
    submit: Submit = accept,
    newest: Observation | None = None,
    trigger: Trigger | None = None,
    requests: list[CompletionRequest] | None = None,
) -> ConsultRecord:
    """One consult of the clerk on a `due` trigger; `newest` is the observation the intent is built against."""
    with seat_view() as store:
        names = NameBook.load(store)
        situation = Situation.of(observation, names)
        fillables = fill.listed(observation, names, settings)
        later = observation if newest is None else newest
        clock = Clock()

        async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
            del timeout
            if requests is not None:
                requests.append(request)
            completion = await backend.complete(request)
            if isinstance(completion, BackendFailure):
                return Failed(failure=completion)
            return Completed(completion=completion)

        deps = ConsultDeps(
            ask=ask,
            newest=lambda: NewestState(situation=Situation.of(later, names), observation=later),
            current=lambda: True,
            submit=submit,
            clock=clock.clock,
            sleep=clock.sleep,
            policies=_policies(),
            limits=ConsultLimits(deadline_s=20.0, max_calls=5),
        )
        return run(
            consult(
                ConsultInput(
                    seat=EntityKey("carol"),
                    persona=Persona(),
                    situation=situation,
                    memory=MemorySection(lines=()),
                    trigger=due() if trigger is None else trigger,
                    fillables=fillables,
                ),
                deps,
            )
        )
