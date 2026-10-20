"""A seat's memory stack: chosen in `decisions.yaml` `memory: scopes`, resolved when the seat starts
(pr-s28c-memory-scopes.md §5.5; MODULE_SPEC §5.3).

The Rust reader spells each id and passes the list through with positions (decision JSON format 3); this module
judges the rest. `resolve_stack` is a pure function of the choice and the installed registry: it binds every scope,
or refuses the stack by name with the entry's source position, never echoing a value.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from mineworld_sdk.wire.ids import EntityKey, JsonValue
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.retention.registry import JsonObject, SourceSpan
from mineworld_cognition.memory.scopes.errors import (
    PersonScopeRequired,
    ScopeListedTwice,
    ScopeNotSelectable,
    TotalSectionsOverCeiling,
)
from mineworld_cognition.memory.scopes.interface import ScopeId
from mineworld_cognition.memory.scopes.registry import BoundScope, ScopeRegistry

if TYPE_CHECKING:
    from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice

SECTION_BYTES = 6000
"""The memory part's ceiling (D-P4-9, `ARC-59`); equal to `memory.retrieve.SECTION_BYTES`."""
PERSON = "person"
NOT_SELECTABLE = ("world",)
"""Base scopes no stack lists: their placement is `cognition.toml`'s (QS28c-1)."""


class MemoryScopeChoice(CognitionModel):
    id: ScopeId
    parameters: JsonObject = Field(default_factory=dict[str, JsonValue])
    source: SourceSpan | None = None
    """Where the scope's entry was written."""


DEFAULT_STACK: tuple[MemoryScopeChoice, ...] = (MemoryScopeChoice(id=PERSON),)
"""The stack of an entry that writes no `scopes`."""


@dataclass(frozen=True)
class MemoryStack:
    """The bound scopes of one seat: `person` first, then the world's order."""

    seat: EntityKey
    scopes: tuple[BoundScope, ...]

    def section_bytes(self) -> int:
        return sum(spec.budget.bytes for scope in self.scopes for spec in scope.sections)


def resolve_stack(
    choice: MemoryPolicyChoice, registry: ScopeRegistry, *, seat: EntityKey
) -> MemoryStack:
    """The choice's stack (`scopes` `None`: `[person]`) bound to installed scopes, or a refusal by name, in
    this order: `ScopeNotSelectable`, `PersonScopeRequired`, `ScopeListedTwice`, `UnknownScope`,
    `ScopeParameterError`, `TotalSectionsOverCeiling`. A scope without its own position is named by the
    entry's."""
    return _resolve(choice.scopes, registry, seat=seat, source=choice.source)


def _resolve(
    scopes: tuple[MemoryScopeChoice, ...] | None,
    registry: ScopeRegistry,
    *,
    seat: EntityKey,
    source: SourceSpan | None,
) -> MemoryStack:
    chosen = DEFAULT_STACK if scopes is None else scopes
    for choice in chosen:
        if choice.id in NOT_SELECTABLE:
            raise ScopeNotSelectable(choice.id, choice.source or source)
    if not chosen or chosen[0].id != PERSON:
        raise PersonScopeRequired(chosen[0].source if chosen else source)
    seen: set[str] = set()
    for choice in chosen:
        if choice.id in seen:
            raise ScopeListedTwice(choice.id, choice.source or source)
        seen.add(choice.id)
    bound = tuple(
        registry.bind(choice.id, choice.parameters, source=choice.source or source)
        for choice in chosen
    )
    stack = MemoryStack(seat=seat, scopes=bound)
    total = stack.section_bytes()
    if total > SECTION_BYTES:
        raise TotalSectionsOverCeiling(total, SECTION_BYTES, source)
    return stack
