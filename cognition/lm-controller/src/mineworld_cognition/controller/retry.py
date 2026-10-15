"""Retry policies: a typed protocol, three bundled classes, and a registry of entry points (pr-s10-p6 §5.5.2, C7).

A policy decides, for one failed attempt of one class, whether to retry and how long to wait. It reads no clock:
the engine sleeps through an injected `sleep`, and jitter comes from a `random.Random` the engine seeds per
(seat, consult, class). The registry mirrors FG-b's registry in shape (group `mineworld.retry`, the entry-point
name is the id, strict parameters, typed load errors). It does not reuse FG-b's loader, because that loader is
coupled to `ForgettingPolicy` (F-P6-9, decided in C7; FG-b's file is unchanged).
"""

from __future__ import annotations

import random
from collections.abc import Iterable
from importlib.metadata import EntryPoint, entry_points
from typing import Annotated, ClassVar, Protocol, cast

from mineworld_sdk.wire.ids import JsonValue
from pydantic import Field, ValidationError

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.controller.failures import FailureRecord

GROUP = "mineworld.retry"
ATTEMPTS_MIN, ATTEMPTS_MAX = 1, 10
DELAY_MAX_S = 60.0


class RetryDecision(CognitionModel):
    retry: bool
    delay_s: Annotated[float, Field(ge=0.0)] = 0.0


class ExponentialBackoffParameters(CognitionModel):
    attempts: Annotated[int, Field(ge=ATTEMPTS_MIN, le=ATTEMPTS_MAX)] = 3
    first_s: Annotated[float, Field(ge=0.0, le=DELAY_MAX_S)] = 0.5
    factor: Annotated[float, Field(ge=1.0, le=10.0)] = 2.0
    ceiling_s: Annotated[float, Field(ge=0.0, le=DELAY_MAX_S)] = 8.0
    jitter: Annotated[float, Field(ge=0.0, le=1.0)] = 0.5


class FixedParameters(CognitionModel):
    attempts: Annotated[int, Field(ge=ATTEMPTS_MIN, le=ATTEMPTS_MAX)] = 1
    delay_s: Annotated[float, Field(ge=0.0, le=DELAY_MAX_S)] = 0.0


class NoParameters(CognitionModel):
    """`none` takes no parameters."""


class RetryPolicy(Protocol):
    """One retry layer's rule. `attempt` counts this class's attempts in this consult, from 1. `params` is
    the class's own `Parameters` model, validated by the registry."""

    id: ClassVar[str]
    Parameters: ClassVar[type[CognitionModel]]

    def decide(
        self, attempt: int, failure: FailureRecord, params: CognitionModel, jitter: random.Random
    ) -> RetryDecision: ...


class ExponentialBackoff:
    """Retry while `attempt < attempts`; delay `min(ceiling, first * factor^(attempt-1))`, scaled by a factor
    drawn from `[1 - jitter, 1 + jitter]`; `Retry-After` lengthens the delay, never shortens it."""

    id: ClassVar[str] = "exponential-backoff"
    Parameters: ClassVar[type[CognitionModel]] = ExponentialBackoffParameters

    def decide(
        self, attempt: int, failure: FailureRecord, params: CognitionModel, jitter: random.Random
    ) -> RetryDecision:
        settings = cast(ExponentialBackoffParameters, params)
        if attempt >= settings.attempts:
            return RetryDecision(retry=False)
        base = min(settings.ceiling_s, settings.first_s * settings.factor ** (attempt - 1))
        scale = 1.0 - settings.jitter + 2.0 * settings.jitter * jitter.random()
        delay = base * scale
        if failure.retry_after_s is not None:
            delay = max(delay, float(failure.retry_after_s))
        return RetryDecision(retry=True, delay_s=delay)


class Fixed:
    """Retry while `attempt < attempts`, after `delay_s`."""

    id: ClassVar[str] = "fixed"
    Parameters: ClassVar[type[CognitionModel]] = FixedParameters

    def decide(
        self, attempt: int, failure: FailureRecord, params: CognitionModel, jitter: random.Random
    ) -> RetryDecision:
        settings = cast(FixedParameters, params)
        if attempt >= settings.attempts:
            return RetryDecision(retry=False)
        return RetryDecision(retry=True, delay_s=settings.delay_s)


class NoRetry:
    """Never retries."""

    id: ClassVar[str] = "none"
    Parameters: ClassVar[type[CognitionModel]] = NoParameters

    def decide(
        self, attempt: int, failure: FailureRecord, params: CognitionModel, jitter: random.Random
    ) -> RetryDecision:
        return RetryDecision(retry=False)


class UnknownRetryPolicy(ValueError):
    def __init__(self, policy: str, source: str, known: Iterable[str]) -> None:
        super().__init__(
            f"{source}: unknown retry policy; the installed policies are {', '.join(sorted(known))}"
        )
        self.policy = policy


class InvalidRetryParameters(ValueError):
    def __init__(self, source: str, policy: str, why: str) -> None:
        super().__init__(f"{source}: the parameters of retry policy {policy} are invalid: {why}")


class BoundRetry:
    """A policy bound to its validated parameters for one layer (`LayerPolicies`, C9)."""

    def __init__(self, policy_id: str, policy: RetryPolicy, params: CognitionModel) -> None:
        self.policy_id = policy_id
        self._policy = policy
        self._params = params

    @property
    def attempts(self) -> int:
        """The attempts this layer allows, counting the first try (`fixed` and `exponential-backoff`)."""
        if isinstance(self._params, (ExponentialBackoffParameters, FixedParameters)):
            return self._params.attempts
        return 1

    def decide(self, attempt: int, failure: FailureRecord, jitter: random.Random) -> RetryDecision:
        return self._policy.decide(attempt, failure, self._params, jitter)


class RetryRegistry:
    """The installed retry policies, by entry-point name (group `mineworld.retry`)."""

    def __init__(self, policies: dict[str, RetryPolicy]) -> None:
        self._policies = policies

    @staticmethod
    def installed() -> RetryRegistry:
        return RetryRegistry.from_entry_points(entry_points(group=GROUP))

    @staticmethod
    def from_entry_points(points: Iterable[EntryPoint]) -> RetryRegistry:
        loaded: dict[str, RetryPolicy] = {}
        for point in points:
            policy = cast(RetryPolicy, point.load()())
            loaded[point.name] = policy
        return RetryRegistry(loaded)

    def ids(self) -> tuple[str, ...]:
        return tuple(sorted(self._policies))

    def bind(self, policy: str, parameters: dict[str, JsonValue], source: str) -> BoundRetry:
        if policy not in self._policies:
            raise UnknownRetryPolicy(policy, source, self._policies)
        chosen = self._policies[policy]
        try:
            params = chosen.Parameters.model_validate(parameters)
        except ValidationError as error:
            why = "; ".join(
                ".".join(str(part) for part in item["loc"]) + ": " + item["msg"]
                for item in error.errors()
            )
            raise InvalidRetryParameters(source, policy, why) from None
        return BoundRetry(policy, chosen, params)
