"""`SelfAttributes` and a person's description: what the world discloses as attributes, composed for the prompt
(pr-s30f-attribute-consumers §5.6).

Every phrase is the server's: this module selects by facet and cuts by bytes through `mineworld_sdk.attributes`
(§5.4), and names no attribute. Who I am and how I speak come from the observer's own entity; who I see comes from
the other person's own entity record in the same observation, never from the observer's blocks (M-f-2). An
observation with no attribute blocks yields empty texts, and the prompt then renders exactly as before (I-f-2).
Pure: no clock, no randomness, no environment.
"""

from __future__ import annotations

from mineworld_sdk import attributes
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.backend.model import CognitionModel

ABOUT_BYTES = 600
"""Who I am: the facet `prompt`, at most this many UTF-8 bytes."""
MANNER_BYTES = 160
"""How I speak: the facet `speech`."""
DESCRIPTION_BYTES = 160
"""Who I see: the facet `observed`, for each person present."""
ABOUT_FACET = "prompt"
MANNER_FACET = "speech"
DESCRIPTION_FACET = "observed"


class SelfAttributes(CognitionModel):
    """The observer's own attributes as the system message shows them; empty when none are disclosed."""

    about: str = ""
    manner: str = ""

    @staticmethod
    def of(observation: Observation) -> SelfAttributes:
        entries = _entries(observation, observation.observer)
        return SelfAttributes(
            about=attributes.compose(entries, ABOUT_FACET, ABOUT_BYTES),
            manner=attributes.compose(entries, MANNER_FACET, MANNER_BYTES),
        )


NO_ATTRIBUTES = SelfAttributes()
"""No attribute disclosed: the default wherever an older caller or log has none."""


def description(observation: Observation, person: EntityId) -> str:
    """Who I see of `person`: composed from that person's own entity record in `observation`."""
    return attributes.compose(_entries(observation, person), DESCRIPTION_FACET, DESCRIPTION_BYTES)


def _entries(observation: Observation, entity: EntityId) -> tuple[attributes.AttributeEntry, ...]:
    for perceived in observation.entities:
        if perceived.id == entity:
            return attributes.blocks(perceived)
    return ()
