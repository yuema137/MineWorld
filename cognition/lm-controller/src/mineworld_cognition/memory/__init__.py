"""A seat's subjective memory: a deterministic derivation of what one Person perceived (ARC-59).

Design: `.structured-coding/plans/mvp0/pr-s10-p4-memory.md` (P4), stored in a memory home since S28-a
(`pr-s28a-memory-home.md`, `ARC-109`). Memory reads only the facts it is given, the names it is taught and its
own seat view; it never opens a save, runs a process, or reads another seat's rows (I-3, I-S28-1). Nothing here
names a provider or a model (I-10).

What a cognition client calls: `MemoryHome.open(path, world_instance)` and `home.seat(StoreIdentity)` for a
`SeatView`; `ingest(view, facts, through)` for every perceived frame before advancing its cursor;
`NameBook.load(view).learn(entity, name)` for names from observations; and `retrieve(view,
RecallQuery(...)).render()` for the memory part of a decision's context.
"""

from mineworld_cognition.memory.home import (
    HomeBusy,
    HomeIdentityMismatch,
    MemoryHome,
    NotAMemoryHome,
    SeatNotInHome,
    SeatView,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.ingest import IngestReport, OutOfOrder, ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Citation, EventRange, StoreIdentity
from mineworld_cognition.memory.retrieve import MemorySection, RecallQuery, SectionLine, retrieve
from mineworld_cognition.memory.scopes.world import ConflictingFact

__all__ = [
    "Citation",
    "ConflictingFact",
    "EventRange",
    "HomeBusy",
    "HomeIdentityMismatch",
    "IngestReport",
    "MemoryHome",
    "MemorySection",
    "NameBook",
    "NotAMemoryHome",
    "OutOfOrder",
    "RecallQuery",
    "SeatNotInHome",
    "SeatView",
    "SectionLine",
    "StoreIdentity",
    "UnsupportedStoreSchema",
    "ingest",
    "retrieve",
]
