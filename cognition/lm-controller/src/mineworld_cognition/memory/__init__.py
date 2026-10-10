"""A seat's subjective memory: a deterministic derivation of what one Person perceived (ARC-59).

Design: `.structured-coding/plans/mvp0/pr-s10-p4-memory.md` (P4). Memory reads only the facts it is
given, the names it is taught and its own store; it never opens a save, runs a process, or reads another
seat's store (I-3). Nothing here names a provider or a model (I-10).

What a cognition client calls (§5.8): `MemoryStore.open(path, StoreIdentity)`, `ingest(store, facts,
through)` for every perceived frame before advancing its cursor, `NameBook.load(store).learn(entity,
name)` for names from observations, and `retrieve(store, RecallQuery(...)).render()` for the memory part
of a decision's context.
"""

from mineworld_cognition.memory.ingest import ConflictingFact, IngestReport, OutOfOrder, ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Citation, EventRange, StoreIdentity
from mineworld_cognition.memory.store import MemoryStore, NotAMemoryStore, StoreIdentityMismatch

__all__ = [
    "Citation",
    "ConflictingFact",
    "EventRange",
    "IngestReport",
    "MemoryStore",
    "NameBook",
    "NotAMemoryStore",
    "OutOfOrder",
    "StoreIdentity",
    "StoreIdentityMismatch",
    "ingest",
]
