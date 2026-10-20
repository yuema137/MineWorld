"""Memory scopes (step-28 §5; pr-s28c-memory-scopes.md; MODULE_SPEC §5.3; `ARC-100`).

This package's top level is the scope author's surface, and nothing else: the protocol, its carriers and P4's
citation types. A bundled scope imports exactly these names, as a user's does.

The host itself is imported from its modules: `registry` (`ScopeRegistry`, the entry-point group
`mineworld.memory_scopes`), `stack` (`MemoryScopeChoice`, `resolve_stack`), `host` (recall through a stack),
`view` (`GuardedView`, `BaseView`), `errors` (the named errors). The bundled scopes are `person` and `world`
(the shared fact records, S28-a).
"""

from mineworld_cognition.memory.records import Citation, EventRange
from mineworld_cognition.memory.scopes.interface import (
    Column,
    FactRecord,
    MemoryScope,
    Reading,
    RecallQuery,
    ScopeBytes,
    ScopeParameters,
    ScopeRow,
    ScopeView,
    SectionBudget,
    SectionLine,
    SectionSpec,
    TableSpec,
)

__all__ = [
    "Citation",
    "Column",
    "EventRange",
    "FactRecord",
    "MemoryScope",
    "Reading",
    "RecallQuery",
    "ScopeBytes",
    "ScopeParameters",
    "ScopeRow",
    "ScopeView",
    "SectionBudget",
    "SectionLine",
    "SectionSpec",
    "TableSpec",
]
