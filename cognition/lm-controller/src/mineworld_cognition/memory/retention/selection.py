"""A seat's memory policy choice, as the decision list's reader emits it (pr-fg-b §6, QFGb-1 (a)).

The Rust reader of `decisions.yaml` resolves `memory:` (seat > class > default) and emits one JSON object
per seat; this module owns that object's Python model and its semantic validation. Nothing here reads
YAML. `resolve` runs when the cognition process starts, so an unknown policy or invalid parameters stop the
seat before it plays, naming the file, line and column of the winning entry.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Annotated

from mineworld_sdk.wire.ids import JsonValue
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.gateway import ModelGateway
from mineworld_cognition.memory.retention.registry import (
    JsonObject,
    PolicyError,
    PolicyRegistry,
    RegisteredPolicy,
    SourceSpan,
)

PolicyId = Annotated[str, Field(pattern=r"^[a-z][a-z0-9-]{0,47}$")]


class MemoryPolicyChoice(CognitionModel):
    policy: PolicyId
    fallback: PolicyId | None = None
    """Only an assisted policy (FG-c) takes one."""
    parameters: JsonObject = Field(default_factory=dict[str, JsonValue])
    source: SourceSpan | None = None
    """Where the winning entry was written."""


DEFAULT_CHOICE = MemoryPolicyChoice(policy="consolidate")
"""A world with no `memory:`."""


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


def resolve(
    choice: MemoryPolicyChoice, registry: PolicyRegistry, gateway: ModelGateway | None = None
) -> Retention:
    """The choice bound to an installed policy, or a refusal by name (`UnknownPolicy`,
    `InvalidParameters`, `FallbackNotAllowed`, `FallbackNotPlain`). An assisted policy's fallback is the
    choice's, else the policy's own, bound with its defaults; a plain policy takes none."""
    bound = registry.bind(choice.policy, choice.parameters, source=choice.source)
    if not bound.assisted:
        if choice.fallback is not None:
            raise FallbackNotAllowed(choice.policy, choice.source)
        return Retention(bound)
    named = choice.fallback if choice.fallback is not None else bound.fallback_id
    if named is None:  # an assisted policy always declares one (registry, _assisted_fallback)
        raise PolicyError(f"retention policy {choice.policy!r} declares no fallback")
    fallback = registry.bind(named, {})
    if fallback.assisted:
        raise FallbackNotPlain(choice.policy, named, choice.source)
    return Retention(bound, fallback=fallback, gateway=gateway)
