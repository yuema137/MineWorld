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


@dataclass(frozen=True)
class Retention:
    """What `absorb` runs at every pass. FG-c adds the assisted alternative (R-FGc-FG-1)."""

    policy: RegisteredPolicy


def resolve(choice: MemoryPolicyChoice, registry: PolicyRegistry) -> Retention:
    """The choice bound to an installed policy, or a refusal by name (`UnknownPolicy`,
    `InvalidParameters`, `FallbackNotAllowed`)."""
    bound = registry.bind(choice.policy, choice.parameters, source=choice.source)
    if choice.fallback is not None:
        raise FallbackNotAllowed(choice.policy, choice.source)
    return Retention(bound)
