"""The one way to a model: budget gate → recorder → backend, for one tier binding (§5.3; ARC-57).

`ModelGateway.complete(seat, request)`:

1. key material: `canonical_json(request)` — a float raises `KeyMaterialError` (a bug, raised);
2. budget pre-check against the seat's wall-time windows — over a ceiling, `Refused`, and nothing below
   runs (P5-2);
3. the timeout starts: the smaller of `call_timeout_s` and the caller's `timeout_s`, around both the slot
   and the backend, so a call still queued when it ends never reaches the backend and costs nothing;
4. an in-flight slot (process-wide, first in first out); the bound is checked again once the slot is held;
5. the backend (a replay, a recorder around a live backend, a live backend, or a scripted one); a cassette
   miss raises `CassetteMiss`, which is never an outcome;
6. the charge, dated when the call reached the backend: one call, and the reported usage (or the estimate,
   flagged); a failure, a timeout or a cancellation charges the call and no tokens. A cancellation is
   charged, then raised.

Every exit releases the reservation and the slot it took (ARC-57 dated note 2026-11-27).

`Router.for_tier(tier)` gives the gateway bound to a tier, or None: "no model for this purpose", and
the controller falls back (P6).
"""

from __future__ import annotations

import asyncio
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Literal

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.backend.canonical import canonical_json
from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    ModelBackend,
)
from mineworld_cognition.budget import (
    BudgetPolicy,
    BudgetRefusal,
    Clock,
    InFlightLimiter,
    Ledger,
    Reservations,
    precheck,
    reservation,
)

Tier = Literal["ordinary_decision", "social", "major_decision", "summarize"]
"""What a model is asked for (`ARCHITECTURE.md` §9.1). `routine` is never a tier: it is deterministic
policy (I-15)."""

TIERS: tuple[Tier, ...] = ("ordinary_decision", "social", "major_decision", "summarize")


@dataclass(frozen=True)
class Completed:
    completion: Completion


@dataclass(frozen=True)
class Refused:
    refusal: BudgetRefusal


@dataclass(frozen=True)
class Failed:
    failure: BackendFailure


GateOutcome = Completed | Refused | Failed


@dataclass(frozen=True)
class Budget:
    """What every gateway of one process shares: the ceilings, the ledger, the in-flight limit, the
    admitted-but-uncharged reservations, and the wall clock."""

    policy: BudgetPolicy
    ledger: Ledger
    clock: Clock
    limiter: InFlightLimiter
    reservations: Reservations

    @staticmethod
    def of(policy: BudgetPolicy, ledger: Ledger, clock: Clock) -> Budget:
        return Budget(policy, ledger, clock, InFlightLimiter(policy.max_in_flight), Reservations())


class ModelGateway:
    def __init__(self, backend: ModelBackend, budget: Budget) -> None:
        self._backend = backend
        self._budget = budget

    async def complete(
        self, seat: EntityKey, request: CompletionRequest, *, timeout_s: float | None = None
    ) -> GateOutcome:
        """One call. `timeout_s`, when given, is what is left of the caller's own bound (a consult's absolute
        deadline); the bound used is the smaller of it and `call_timeout_s`, and it covers the wait for the
        slot as well as the backend. A call cut in the backend is charged like a timeout (F-P6-1); a call cut
        before it reached the backend is `Failed` with reason `timeout` and costs nothing."""
        canonical_json(request)
        budget = self._budget
        held = reservation(request)
        refusal = precheck(
            budget.policy, budget.ledger, budget.reservations, seat, budget.clock.now(), held
        )
        if refusal is not None:
            return Refused(refusal)
        limit = budget.policy.call_timeout_s
        if timeout_s is not None:
            limit = min(limit, timeout_s)
        budget.reservations.hold(seat, held)
        started: int | None = None
        try:
            async with asyncio.timeout(limit) as bound:
                async with budget.limiter:
                    _still_open(bound)
                    started = budget.clock.now()
                    answer = await self._backend.complete(request)
        except TimeoutError:
            answer = BackendFailure(reason="timeout")
        except asyncio.CancelledError:
            if started is not None:
                budget.ledger.charge(seat, started, calls=1, tokens=0, estimated=False)
            raise
        finally:
            budget.reservations.release(seat, held)
        if started is None:
            return Failed(BackendFailure(reason="timeout"))
        if isinstance(answer, BackendFailure):
            budget.ledger.charge(seat, started, calls=1, tokens=0, estimated=False)
            return Failed(answer)
        usage = answer.usage
        budget.ledger.charge(
            seat,
            started,
            calls=1,
            tokens=usage.input_tokens + usage.output_tokens,
            estimated=usage.estimated,
        )
        return Completed(answer)

    def sharing[G: ModelGateway](self, kind: type[G]) -> G:
        """A gateway of another kind over this one's backend and budget: one ledger, one recording, one
        in-flight limit. How a seat's assisted memory policy sees the router's `summarize` gateway through
        FG-c's `SummaryBoundaryGateway` (D-P6b-36). The router still owns, and closes, the backend."""
        return kind(self._backend, self._budget)

    async def aclose(self) -> None:
        await self._backend.aclose()

    def __repr__(self) -> str:
        return f"ModelGateway({self._backend!r})"


def _still_open(bound: asyncio.Timeout) -> None:
    """Raises `TimeoutError` when the bound has passed although the slot was granted: no backend call begins
    at or after it (the slot's waiter can be woken in the same loop turn the bound ends)."""
    when = bound.when()
    if when is not None and asyncio.get_running_loop().time() >= when:
        raise TimeoutError


class Router:
    """Tier → gateway, or None. Built from the operator's configuration (`config.build_router`)."""

    def __init__(self, gateways: Mapping[Tier, ModelGateway | None], budget: Budget) -> None:
        self._gateways = dict(gateways)
        self._budget = budget

    def for_tier(self, tier: Tier) -> ModelGateway | None:
        return self._gateways.get(tier)

    async def aclose(self) -> None:
        """Closes every gateway once (several tiers may share one), then the ledger."""
        closed: set[int] = set()
        for gateway in self._gateways.values():
            if gateway is not None and id(gateway) not in closed:
                closed.add(id(gateway))
                await gateway.aclose()
        self._budget.ledger.close()

    def __repr__(self) -> str:
        bound = {tier: gateway is not None for tier, gateway in self._gateways.items()}
        return f"Router({bound})"
