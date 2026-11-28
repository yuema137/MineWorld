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
    def of(observation: Observation, locale: str = attributes.ENGLISH) -> SelfAttributes:
        """In the seat's `locale` where the world offers it (S30-k §5.6); the prompt's own frame stays
        English (Q-S30k-6)."""
        disclosed = _disclosed(observation, observation.observer)
        reading = disclosed.reading(locale)
        return SelfAttributes(
            about=attributes.compose(disclosed.entries, ABOUT_FACET, ABOUT_BYTES, reading),
            manner=attributes.compose(disclosed.entries, MANNER_FACET, MANNER_BYTES, reading),
        )


NO_ATTRIBUTES = SelfAttributes()
"""No attribute disclosed: the default wherever an older caller or log has none."""


def description(
    observation: Observation, person: EntityId, locale: str = attributes.ENGLISH
) -> str:
    """Who I see of `person`: composed from that person's own entity record in `observation`, in
    `locale` where the world offers it."""
    disclosed = _disclosed(observation, person)
    return attributes.compose(
        disclosed.entries, DESCRIPTION_FACET, DESCRIPTION_BYTES, disclosed.reading(locale)
    )


def _disclosed(observation: Observation, entity: EntityId) -> attributes.Disclosed:
    for perceived in observation.entities:
        if perceived.id == entity:
            return attributes.read(perceived)
    return attributes.Disclosed()
