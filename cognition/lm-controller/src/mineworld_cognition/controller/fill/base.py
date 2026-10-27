"""The parts of a filler: what one lists, shows, asks for, checks and builds (pr-s31d §5.8.2; `ARC-183`).

A **filler** knows one incomplete action type. The world says "you may do this" (an available affordance with no
payload), and the content is the actor's. A filler lists itself when the observation offers its action and the
seat has what the content is made from; it shows the model what it needs in labels only; it asks for typed fields
under one key; it checks the answer field by field; and it builds the payload from the checked fields. The model
never writes a payload. Every member is pure: no clock, no randomness, no environment (`ARC-72`).
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Protocol

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, JsonValue

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.config import FillConfig
from mineworld_cognition.controller.choice import ContentFailure
from mineworld_cognition.memory.names import NameBook


class FillShown(CognitionModel):
    """What one listed filler showed the model: the filler's own typed view, checked against later."""


class FillPayload(CognitionModel):
    """A checked answer, as the request it becomes: the action type, the target, and the payload the filler
    built from typed fields."""

    action_type: ActionTypeId
    target: EntityId | None
    payload: JsonValue


class Filler(Protocol):
    """One incomplete action type, filled from typed fields."""

    @property
    def action_type(self) -> ActionTypeId:
        """The incomplete action this filler fills."""
        ...

    @property
    def key(self) -> str:
        """The key the model answers this filler's fields under, inside `fill`."""
        ...

    @property
    def due_events(self) -> Mapping[str, str]:
        """The perceived fact types that raise a `due` consult while this filler lists itself, each with the line
        `WHAT JUST HAPPENED` reads for it."""
        ...

    def listed(
        self, observation: Observation, names: NameBook, settings: FillConfig
    ) -> FillShown | None:
        """What this filler shows, when it lists itself in `observation`; else `None`."""
        ...

    def section(self, number: str, shown: FillShown) -> tuple[str, ...]:
        """The `TO FILL` line for `number` (`F1`), then the lines of the filler's own block."""
        ...

    def schema(self) -> dict[str, JsonValue]:
        """The JSON Schema of the value under `key`."""
        ...

    def check(self, content: JsonValue, shown: FillShown) -> FillPayload | ContentFailure:
        """The answer under `key`, checked against what was shown, as the request it becomes."""
        ...


@dataclass(frozen=True)
class Fillable:
    """One listed filler in one consult: its number (`F1`, `F2`, … in registry order), the filler, and what it
    showed."""

    number: str
    filler: Filler
    shown: FillShown

    def due_events(self) -> Mapping[str, str]:
        return self.filler.due_events
