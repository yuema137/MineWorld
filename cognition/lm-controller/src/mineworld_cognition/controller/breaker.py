"""The per-seat circuit breaker (pr-s10-p6 §5.6, C8).

A breaker counts consult outcomes, not HTTP calls: `success` for `acted` and `chose`, `failure` for a consult that
fell back for content, semantic or transport reasons, or hit the deadline. The engine decides which outcomes are
counted (C9); a breaker never sees a budget refusal, a disconnect, a dropped trigger or a frame refusal. A breaker
reads no clock: the caller passes `now_wall_s` and the world's `time_scale`, so the cooldown is measured in
world seconds and converted to wall seconds (`cooldown_world_s / time_scale`). Registry group `mineworld.breaker`,
shaped as `retry.py`.
"""

from __future__ import annotations

from collections import deque
from collections.abc import Iterable
from dataclasses import dataclass
from importlib.metadata import EntryPoint, entry_points
from typing import Annotated, ClassVar, Literal, Protocol, cast

from mineworld_sdk.wire.ids import JsonValue
from pydantic import Field, ValidationError

from mineworld_cognition.backend.model import CognitionModel

GROUP = "mineworld.breaker"
BreakerOutcome = Literal["success", "failure"]


class BreakerState(CognitionModel):
    """`closed`, `open` (until a wall second), or `half_open` (the next consult is a probe)."""

    kind: Literal["closed", "open", "half_open"]
    until_wall_s: float | None = None


CLOSED = BreakerState(kind="closed")


class CircuitBreaker(Protocol):
    def observe(
        self, outcome: BreakerOutcome, now_wall_s: float, time_scale: int
    ) -> BreakerState: ...

    def state(self, now_wall_s: float) -> BreakerState: ...


class FailureRateParameters(CognitionModel):
    window: Annotated[int, Field(ge=1, le=200)] = 20
    min_consults: Annotated[int, Field(ge=1, le=200)] = 10
    threshold_permille: Annotated[int, Field(ge=1, le=1000)] = 200
    cooldown_world_s: Annotated[int, Field(ge=0, le=86400)] = 3600


class NeverParameters(CognitionModel):
    """`never` takes no parameters."""


class FailureRate:
    """Opens when the window holds at least `min_consults` outcomes and `failures * 1000 > threshold_permille *
    window length`. Open, it stays closed to the seat for the cooldown; then half-open, where the next outcome
    decides: a success closes it with an empty window, a failure opens it again for the same cooldown."""

    id: ClassVar[str] = "failure-rate"
    Parameters: ClassVar[type[CognitionModel]] = FailureRateParameters

    def __init__(self, params: FailureRateParameters) -> None:
        self._params = params
        self._window: deque[BreakerOutcome] = deque(maxlen=params.window)
        self._open_until: float | None = None

    def state(self, now_wall_s: float) -> BreakerState:
        if self._open_until is None:
            return CLOSED
        if now_wall_s < self._open_until:
            return BreakerState(kind="open", until_wall_s=self._open_until)
        return BreakerState(kind="half_open")

    def observe(self, outcome: BreakerOutcome, now_wall_s: float, time_scale: int) -> BreakerState:
        if self.state(now_wall_s).kind == "half_open":
            # The probe decides: success closes with an empty window, failure opens for a full cooldown.
            self._open_until = None
            if outcome == "success":
                self._window.clear()
                return CLOSED
            return self._trip(now_wall_s, time_scale)
        self._window.append(outcome)
        if self._tripped():
            return self._trip(now_wall_s, time_scale)
        return CLOSED

    def _tripped(self) -> bool:
        if len(self._window) < self._params.min_consults:
            return False
        failures = sum(1 for item in self._window if item == "failure")
        return failures * 1000 > self._params.threshold_permille * len(self._window)

    def _trip(self, now_wall_s: float, time_scale: int) -> BreakerState:
        self._open_until = now_wall_s + self._params.cooldown_world_s / time_scale
        return BreakerState(kind="open", until_wall_s=self._open_until)


class Never:
    """Always closed: per-consult fallback only."""

    id: ClassVar[str] = "never"
    Parameters: ClassVar[type[CognitionModel]] = NeverParameters

    def __init__(self, params: NeverParameters) -> None:
        self._params = params

    def state(self, now_wall_s: float) -> BreakerState:
        return CLOSED

    def observe(self, outcome: BreakerOutcome, now_wall_s: float, time_scale: int) -> BreakerState:
        return CLOSED


class UnknownBreaker(ValueError):
    def __init__(self, policy: str, source: str, known: Iterable[str]) -> None:
        super().__init__(
            f"{source}: unknown breaker; the installed breakers are {', '.join(sorted(known))}"
        )
        self.policy = policy


class InvalidBreakerParameters(ValueError):
    def __init__(self, source: str, policy: str, why: str) -> None:
        super().__init__(f"{source}: the parameters of breaker {policy} are invalid: {why}")


@dataclass(frozen=True)
class BoundBreaker:
    """A breaker kind bound to its validated parameters; `create()` makes one seat's breaker."""

    policy_id: str
    _kind: type[FailureRate] | type[Never]
    _params: CognitionModel

    def create(self) -> CircuitBreaker:
        if self._kind is FailureRate:
            return FailureRate(cast(FailureRateParameters, self._params))
        return Never(cast(NeverParameters, self._params))


class BreakerRegistry:
    """The installed breakers, by entry-point name (group `mineworld.breaker`)."""

    def __init__(self, kinds: dict[str, type[FailureRate] | type[Never]]) -> None:
        self._kinds = kinds

    @staticmethod
    def installed() -> BreakerRegistry:
        return BreakerRegistry.from_entry_points(entry_points(group=GROUP))

    @staticmethod
    def from_entry_points(points: Iterable[EntryPoint]) -> BreakerRegistry:
        loaded: dict[str, type[FailureRate] | type[Never]] = {}
        for point in points:
            loaded[point.name] = cast(type[FailureRate] | type[Never], point.load())
        return BreakerRegistry(loaded)

    def ids(self) -> tuple[str, ...]:
        return tuple(sorted(self._kinds))

    def bind(self, policy: str, parameters: dict[str, JsonValue], source: str) -> BoundBreaker:
        if policy not in self._kinds:
            raise UnknownBreaker(policy, source, self._kinds)
        kind = self._kinds[policy]
        try:
            params = kind.Parameters.model_validate(parameters)
        except ValidationError as error:
            why = "; ".join(
                ".".join(str(part) for part in item["loc"]) + ": " + item["msg"]
                for item in error.errors()
            )
            raise InvalidBreakerParameters(source, policy, why) from None
        return BoundBreaker(policy, kind, params)
