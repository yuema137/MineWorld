"""The prose name check (FU-S28-1; step-28 §9.2 C-4 (d); pr-s28b §5.4): model prose may not name a known person
or place that its group did not involve. Pure: no store, no connection, no clock.

**Known names** are the values of the seat's `NameBook` (persons and places alike); the generic labels
(`someone`, `somewhere`, `others`) are not names. Each name is matched as a whole word, case-sensitively. Names
are tried longest first, and a match that overlaps an earlier, longer one is ignored, so `Bob` inside an allowed
`Bob Achterberg` is no separate match. A name several entities share is allowed when any of them is.

**The allowed set** of a group (QS28b-1, ruled 2026-10-18): the seat's own entity; the counterparts and the place
of the group's container and of every item nested in it, recursively; and every entity whose known name already
occurs, by the same rule, in the text of one of them (a quoted name, a merged chapter's own text).

Only known names are checked: an unknown capitalised word is not a name memory knows, and a partial name (`Bob`
for an outside `Bob Smith`) is not detected. Both are recorded limitations (pr-s28b R-b-4).
"""

from __future__ import annotations

import re
from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from typing import TYPE_CHECKING

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.names import SOMEONE, SOMEWHERE
from mineworld_cognition.memory.records import ItemId

if TYPE_CHECKING:
    from mineworld_cognition.memory.retention.items import Extracted

GENERIC = frozenset({SOMEONE, SOMEWHERE, "others"})
"""Labels memory writes for what it cannot name: never a name, whatever the name book holds."""


@dataclass(frozen=True)
class _Name:
    text: str
    pattern: re.Pattern[str]
    owners: frozenset[EntityId]


class KnownNames:
    """A seat's known names, compiled once per pass, longest first."""

    def __init__(self, names: Mapping[EntityId, str]) -> None:
        owners: dict[str, set[EntityId]] = {}
        for entity, name in names.items():
            if name and name not in GENERIC:
                owners.setdefault(name, set()).add(entity)
        ordered = sorted(owners, key=lambda name: (-len(name), name))
        self._names = tuple(
            _Name(name, re.compile(rf"(?<!\w){re.escape(name)}(?!\w)"), frozenset(owners[name]))
            for name in ordered
        )

    def found(self, text: str) -> list[tuple[str, frozenset[EntityId]]]:
        """The known names `text` uses, in the order they occur, each with the entities that bear it."""
        spans: list[tuple[int, int, _Name]] = []
        for name in self._names:
            for match in name.pattern.finditer(text):
                start, end = match.span()
                if any(
                    start < taken_end and taken_start < end for taken_start, taken_end, _ in spans
                ):
                    continue
                spans.append((start, end, name))
        return [(name.text, name.owners) for _, _, name in sorted(spans, key=lambda span: span[0])]

    def entities_in(self, texts: Iterable[str]) -> set[EntityId]:
        return {entity for text in texts for _, owners in self.found(text) for entity in owners}


def outside_names(
    prose: str, names: KnownNames, allowed: frozenset[EntityId] | set[EntityId]
) -> tuple[str, ...]:
    """The known names the prose uses that belong to no allowed entity, in the order they occur, once each."""
    out: list[str] = []
    for name, owners in names.found(prose):
        if owners.isdisjoint(allowed) and name not in out:
            out.append(name)
    return tuple(out)


def allowed_entities(
    extracted: Extracted, container: ItemId, me: EntityId, names: KnownNames
) -> set[EntityId]:
    """The allowed set of the group keyed by `container` (items 1 … 3 of the module's rule)."""
    entities: list[EntityId] = []
    texts: list[str] = []
    pending = [container]
    while pending:
        item = extracted.by_id[pending.pop()]
        entities.extend(item.counterparts)
        if item.place is not None:
            entities.append(item.place)
        texts.append(item.text)
        pending.extend(extracted.children.get(item.item, ()))
    return allowed(me, entities, texts, names)


def allowed(
    me: EntityId, entities: Iterable[EntityId], texts: Iterable[str], names: KnownNames
) -> set[EntityId]:
    """The seat, the given entities, and every entity whose known name occurs in the given texts."""
    return {me, *entities, *names.entities_in(texts)}
