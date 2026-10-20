"""`recall_view`: the seat's memory stack recalled, with the PA-11 episode knob applied (pr-s10-p6 §5.2.3, C5;
pr-s28c §5.8).

The recall host runs the stack and checks every line (`memory.scopes.host`); a defective scope is withheld for
the consult and reported, never shown to a model (QS28c-3). This keeps every line but the person scope's `L1`
lines beyond `recall_episodes`, which arrive ranked (counterparts first, then lexical matches): a consult shows
`min(L1 lines, recall_episodes)` episodes (QS28c-5). Fewer lines can only shrink the section, so the 6 000-byte
ceiling (`ARC-59`) still holds, and `citations()` is computed on the kept lines, so `recalls` can only cite
what was shown. Pure: no clock, no randomness (I-3).
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.retention.selection import DEFAULT_CHOICE
from mineworld_cognition.memory.retrieve import MemorySection, RecallQuery, SectionLine
from mineworld_cognition.memory.scopes.host import OnDefect, Recalled, recall
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack, resolve_stack

EPISODES_MAX = 8
"""The most L1 lines a consult may show (`recall_episodes 0 … 8`, §5.13)."""


def recall_view(
    store: SeatView,
    stack: MemoryStack,
    query: RecallQuery,
    recall_episodes: int,
    *,
    on_defect: OnDefect = "withhold",
) -> Recalled:
    if not 0 <= recall_episodes <= EPISODES_MAX:
        raise ValueError(f"recall_episodes must be between 0 and {EPISODES_MAX}")
    recalled = recall(store, stack, query, on_defect=on_defect)
    kept: list[SectionLine] = []
    episodes = 0
    for line in recalled.section.lines:
        if line.level == "L1":
            if episodes == recall_episodes:
                continue
            episodes += 1
        kept.append(line)
    section = recalled.section.model_copy(update={"lines": tuple(kept)})
    return Recalled(section=section, defects=recalled.defects)


def memory_view(store: SeatView, query: RecallQuery, recall_episodes: int) -> MemorySection:
    """`recall_view` under the default stack `[person]`, raising on a defect: what a seat without a written
    stack is shown."""
    stack = resolve_stack(
        DEFAULT_CHOICE, ScopeRegistry.installed(), seat=EntityKey(store.identity.seat)
    )
    return recall_view(store, stack, query, recall_episodes, on_defect="raise").section
