"""The closed registry of fillers: one per incomplete action type an LM seat may fill (pr-s31d §5.8; `ARC-183`).

A model answers a complete offer by its number; an incomplete offer has no payload to pick, so a typed **filler**
asks for checked fields and builds the payload itself (`base.py`). The registry is closed and typed, as `ARC-153`
kept the MCP surface (QS27-9: no generic free-form act): a further incomplete action is one module and one line
below. `listed` numbers the fillers that list themselves in an observation `F1`, `F2`, … in registry order; when
none does, a consult is exactly what it was before fillers existed (I-d-8).
"""

from __future__ import annotations

from collections.abc import Mapping
from types import MappingProxyType

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import ActionTypeId

from mineworld_cognition.config import FillConfig
from mineworld_cognition.controller.fill.base import (
    Fillable,
    Filler,
    FillPayload,
    FillShown,
)
from mineworld_cognition.controller.fill.publish_notice import PublishNoticeFiller
from mineworld_cognition.memory.names import NameBook

FILLERS: Mapping[ActionTypeId, Filler] = MappingProxyType(
    {
        PublishNoticeFiller.ACTION_TYPE: PublishNoticeFiller(),
    }
)
"""The installed fillers, by the action type each fills, in listing order."""


def listed(observation: Observation, names: NameBook, settings: FillConfig) -> tuple[Fillable, ...]:
    """The fillers that list themselves in `observation`, numbered `F1`, `F2`, … in registry order."""
    found: list[Fillable] = []
    for filler in FILLERS.values():
        shown = filler.listed(observation, names, settings)
        if shown is not None:
            found.append(Fillable(number=f"F{len(found) + 1}", filler=filler, shown=shown))
    return tuple(found)


def due_events(fillables: tuple[Fillable, ...]) -> Mapping[str, str]:
    """The union of the listed fillers' due fact types, each with its line; empty when none is listed."""
    events: dict[str, str] = {}
    for fillable in fillables:
        for event, line in fillable.due_events().items():
            events.setdefault(event, line)
    return MappingProxyType(events)


__all__ = [
    "FILLERS",
    "FillPayload",
    "FillShown",
    "Fillable",
    "Filler",
    "due_events",
    "listed",
]
