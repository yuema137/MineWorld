"""The one way to a model: budget gate → recorder → backend, for one tier binding (§5.3; ARC-57).

`ModelGateway.complete(seat, request)`:

1. key material: `canonical_json(request)` — a float raises `KeyMaterialError` (a bug, raised);
2. budget pre-check against the seat's wall-time windows — over a ceiling, `Refused`, and nothing below
   runs (P5-2);
3. an in-flight slot (process-wide, first in first out);
4. the backend (a replay, a recorder around a live backend, a live backend, or a scripted one) within
   `call_timeout_s`; a cassette miss raises `CassetteMiss`, which is never an outcome;
5. the charge: one call, and the reported usage (or the estimate, flagged); a failure or a timeout
   charges the call and no tokens.

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

    async def complete(self, seat: EntityKey, request: CompletionRequest) -> GateOutcome:
        canonical_json(request)
        budget = self._budget
        now = budget.clock.now()
        held = reservation(request)
        refusal = precheck(budget.policy, budget.ledger, budget.reservations, seat, now, held)
        if refusal is not None:
            return Refused(refusal)
        budget.reservations.hold(seat, held)
        try:
            async with budget.limiter:
                try:
                    async with asyncio.timeout(budget.policy.call_timeout_s):
                        answer = await self._backend.complete(request)
                except TimeoutError:
                    answer = BackendFailure(reason="timeout")
        finally:
            budget.reservations.release(seat, held)
        if isinstance(answer, BackendFailure):
            budget.ledger.charge(seat, now, calls=1, tokens=0, estimated=False)
            return Failed(answer)
        usage = answer.usage
        budget.ledger.charge(
            seat,
            now,
            calls=1,
            tokens=usage.input_tokens + usage.output_tokens,
            estimated=usage.estimated,
        )
        return Completed(answer)

    async def aclose(self) -> None:
        await self._backend.aclose()

    def __repr__(self) -> str:
        return f"ModelGateway({self._backend!r})"


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
