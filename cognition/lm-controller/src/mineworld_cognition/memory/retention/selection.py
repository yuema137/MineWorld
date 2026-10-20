"""A seat's memory policy choice, as the decision list's reader emits it (pr-fg-b §6, QFGb-1 (a)).

The Rust reader of `decisions.yaml` resolves `memory:` (seat > class > default) and emits one JSON object
per seat; this module owns that object's Python model and its semantic validation. Nothing here reads
YAML. `resolve` runs when the cognition process starts, so an unknown policy or invalid parameters stop the
seat before it plays, naming the file, line and column of the winning entry.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Annotated

from mineworld_sdk.wire.ids import EntityKey, JsonValue
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.gateway import ModelGateway
from mineworld_cognition.memory.retention.bound import (
    DEFAULT_BOUNDS,
    FLOOR_MIB,
    StoreBoundOutOfRange,
    StoreBounds,
)
from mineworld_cognition.memory.retention.registry import (
    JsonObject,
    PolicyError,
    PolicyRegistry,
    RegisteredPolicy,
    SourceSpan,
)
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice

PolicyId = Annotated[str, Field(pattern=r"^[a-z][a-z0-9-]{0,47}$")]


class MemoryPolicyChoice(CognitionModel):
    policy: PolicyId
    fallback: PolicyId | None = None
    """Only an assisted policy (FG-c) takes one."""
    parameters: JsonObject = Field(default_factory=dict[str, JsonValue])
    store_bound_mib: int | None = None
    """The per-person store bound in MiB; `None` is the 20 MiB default. A world may raise it, never lower
    it: `resolve` refuses a value outside 20 … 1 024 (pr-fg-b2 §4.1). The host's bound, not a policy's
    parameter: no policy sees it."""
    scopes: tuple[MemoryScopeChoice, ...] | None = None
    """The memory stack as written (pr-s28c §5.5); `None` is `[person]` with its defaults. Resolved by
    `memory.scopes.stack.resolve_stack`, not here."""
    source: SourceSpan | None = None
    """Where the winning entry was written."""


DEFAULT_CHOICE = MemoryPolicyChoice(policy="consolidate")
"""A world with no `memory:`."""


class BoundBelowDefault(StoreBoundOutOfRange):
    """A `store_bound_mib` under the 20 MiB default: a world may raise the bound, never lower it (step-28 §5.5;
    AS28-16 (c)). A `StoreBoundOutOfRange`, as FG-b2 named it, now naming the seat."""

    def __init__(self, seat: EntityKey | None, given: int, source: SourceSpan | None) -> None:
        super().__init__(given, source)
        where = f" ({source})" if source is not None else ""
        whose = f" for seat {seat}" if seat is not None else ""
        PolicyError.__init__(
            self,
            f"memory.retention: store_bound_mib {given}{whose}{where} is below the {FLOOR_MIB} MiB "
            f"default: a world may raise the per-person bound, never lower it",
        )
        self.seat = seat
        self.given_mib = given


class FallbackNotAllowed(PolicyError):
    def __init__(self, policy: str, source: SourceSpan | None) -> None:
        where = f" ({source})" if source is not None else ""
        super().__init__(
            f"retention policy {policy!r}{where} takes no fallback: only an assisted policy does"
        )
        self.policy = policy
        self.source = source


class FallbackNotPlain(PolicyError):
    def __init__(self, policy: str, fallback: str, source: SourceSpan | None) -> None:
        where = f" ({source})" if source is not None else ""
        super().__init__(
            f"retention policy {policy!r}{where} names fallback {fallback!r}, which is assisted: a "
            "fallback must be a no-model policy"
        )
        self.policy = policy
        self.fallback = fallback
        self.source = source


@dataclass(frozen=True)
class Retention:
    """What `absorb` runs at every pass (pr-fg-b §5.6; FG-c's assisted alternative, R-FGc-FG-1).

    `fallback` is set exactly for an assisted policy: the plain policy, bound with its defaults, that
    decides every group the model does not (QFGc-6). `gateway` is the seat's `summarize` gateway, or
    `None` when that tier is unbound (the pass then records `fallback:unbound`)."""

    policy: RegisteredPolicy
    fallback: RegisteredPolicy | None = None
    gateway: ModelGateway | None = None
    bounds: StoreBounds = DEFAULT_BOUNDS
    """The store's bound, target and size trigger (pr-fg-b2 §4.1)."""


def resolve(
    choice: MemoryPolicyChoice,
    registry: PolicyRegistry,
    gateway: ModelGateway | None = None,
    *,
    seat: EntityKey | None = None,
) -> Retention:
    """The choice bound to an installed policy, or a refusal by name (`UnknownPolicy`,
    `InvalidParameters`, `FallbackNotAllowed`, `FallbackNotPlain`, `BoundBelowDefault` naming `seat`,
    `StoreBoundOutOfRange`). An assisted
    policy's fallback is the choice's, else the policy's own, bound with its defaults; a plain policy takes
    none."""
    bound = registry.bind(choice.policy, choice.parameters, source=choice.source)
    if not bound.assisted:
        if choice.fallback is not None:
            raise FallbackNotAllowed(choice.policy, choice.source)
        return Retention(bound, bounds=_bounds(choice, seat))
    named = choice.fallback if choice.fallback is not None else bound.fallback_id
    if named is None:  # an assisted policy always declares one (registry, _assisted_fallback)
        raise PolicyError(f"retention policy {choice.policy!r} declares no fallback")
    fallback = registry.bind(named, {})
    if fallback.assisted:
        raise FallbackNotPlain(choice.policy, named, choice.source)
    return Retention(bound, fallback=fallback, gateway=gateway, bounds=_bounds(choice, seat))


def _bounds(choice: MemoryPolicyChoice, seat: EntityKey | None) -> StoreBounds:
    mib = FLOOR_MIB if choice.store_bound_mib is None else choice.store_bound_mib
    if mib < FLOOR_MIB:
        raise BoundBelowDefault(seat, mib, choice.source)
    try:
        return StoreBounds(mib)
    except StoreBoundOutOfRange:
        raise StoreBoundOutOfRange(mib, choice.source) from None
