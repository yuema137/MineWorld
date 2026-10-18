"""`Persona`: the observer's own character, read from its own `persona` component (pr-s10-p6 §5.2.1, C5).

The component is P6a's (`systems/persona`): a person's authored character, disclosed to that person only.
The controller reads the observer's own copy and nothing else. The bounds repeat P6a's, so a bad value from
a foreign server cannot reach a prompt. A world without `persona` yields an empty persona, and the prompt uses
the display name alone. Pure: no clock, no randomness, no environment (I-3).
"""

from __future__ import annotations

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import JsonValue
from pydantic import ValidationError, model_validator

from mineworld_cognition.backend.model import CognitionModel

PERSONA_COMPONENT = "persona"
"""The component type P6a's pack publishes to its holder."""
SUMMARY_BYTES = 280
TRAIT_BYTES = 32
TRAITS_MAX = 8
SPEECH_BYTES = 160


class Persona(CognitionModel):
    """A person's character as the prompt may show it. Every field is bounded in UTF-8 bytes."""

    summary: str = ""
    traits: tuple[str, ...] = ()
    speech: str = ""

    @model_validator(mode="after")
    def _bounded(self) -> Persona:
        if len(self.summary.encode("utf-8")) > SUMMARY_BYTES:
            raise ValueError(f"summary is longer than {SUMMARY_BYTES} bytes")
        if len(self.traits) > TRAITS_MAX:
            raise ValueError(f"more than {TRAITS_MAX} traits")
        if any(len(trait.encode("utf-8")) > TRAIT_BYTES for trait in self.traits):
            raise ValueError(f"a trait is longer than {TRAIT_BYTES} bytes")
        if len(self.speech.encode("utf-8")) > SPEECH_BYTES:
            raise ValueError(f"speech is longer than {SPEECH_BYTES} bytes")
        return self

    @staticmethod
    def of(observation: Observation) -> tuple[Persona, bool]:
        """The observer's own `persona` component, strictly decoded, and whether it was absent or
        undecodable (then the empty persona and `True`; the caller counts a `persona_absent` note)."""
        payload = _own_persona_payload(observation)
        if payload is None:
            return Persona(), True
        try:
            # Lax mode only for JSON's list-for-tuple; `extra="forbid"` and every bound still apply.
            return Persona.model_validate(payload, strict=False), False
        except ValidationError:
            return Persona(), True


def _own_persona_payload(observation: Observation) -> JsonValue | None:
    for entity in observation.entities:
        if entity.id != observation.observer:
            continue
        for record in entity.components:
            if record.component_type == PERSONA_COMPONENT and record.entity == observation.observer:
                return record.payload
    return None
