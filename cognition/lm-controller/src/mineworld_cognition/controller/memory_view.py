"""`memory_view`: the retrieved section with the PA-11 episode knob applied (pr-s10-p6 §5.2.3, C5).

`retrieve` (P4, made forgetting-aware by FG-b) returns the section. This keeps every L3, L2 and L0 line, and
the first `recall_episodes` L1 lines, which arrive ranked (counterparts first, then lexical matches). Fewer lines
can only shrink the section, so the 6 000-byte ceiling (`ARC-59`) still holds, and `citations()` is computed on
the kept lines, so `recalls` can only cite what was shown. Pure: no clock, no randomness (I-3).
"""

from __future__ import annotations

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.retrieve import MemorySection, RecallQuery, SectionLine, retrieve

EPISODES_MAX = 8
"""The most L1 lines a consult may show (`recall_episodes 0 … 8`, §5.13)."""


def memory_view(store: SeatView, query: RecallQuery, recall_episodes: int) -> MemorySection:
    if not 0 <= recall_episodes <= EPISODES_MAX:
        raise ValueError(f"recall_episodes must be between 0 and {EPISODES_MAX}")
    kept: list[SectionLine] = []
    episodes = 0
    for line in retrieve(store, query).lines:
        if line.level == "L1":
            if episodes == recall_episodes:
                continue
            episodes += 1
        kept.append(line)
    return MemorySection(lines=tuple(kept))
