"""`Situation`: what the observer can see and do, in labels the model may use (pr-s10-p6 §5.2.2, C5).

The model never sees an entity id. People get labels from the name book (`NameBook.person`); equal names are
suffixed ` (2)`, ` (3)` in ascending `EntityId` order, so the labels are unique and deterministic. Offers are
the observation's *available, complete* affordances (`ARC-34`), numbered from 1 in observation order; each keeps
its index into `observation.affordances`, so `intent.py` can re-locate it in the newest observation. `talk_to`
reads the server's verdict (`available`) and computes nothing. Pure: no clock, no randomness (I-3).
"""

from __future__ import annotations

from mineworld_sdk.wire.contract import Observation, PlaceId
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, JsonValue

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.names import NameBook

DISPLAY_NAME_COMPONENT = "display-name"
TALK = ActionTypeId("talk")


class Person(CognitionModel):
    label: str
    entity: EntityId


class Offer(CognitionModel):
    number: int
    action_type: ActionTypeId
    target_label: str | None
    position: int


class Situation(CognitionModel):
    me: str
    place: str
    at: int
    present: tuple[Person, ...]
    talk_to: tuple[str, ...]
    offers: tuple[Offer, ...]

    @staticmethod
    def of(observation: Observation, names: NameBook) -> Situation:
        _learn(observation, names)
        people = _people(observation)
        labels = _labels(people, names)
        here_id = None if observation.self_location is None else observation.self_location.place
        here = names.place(None if here_id is None else here_id.entity)
        present = sorted(
            (
                Person(label=labels[entity], entity=entity)
                for entity in people
                if entity != observation.observer and _is_at(observation, entity, here_id)
            ),
            key=lambda person: (person.label, person.entity),
        )
        talk_to: list[str] = []
        for affordance in observation.affordances:
            if affordance.action_type != TALK or not affordance.available:
                continue
            label = None if affordance.target is None else labels.get(affordance.target)
            if label is not None and label not in talk_to:
                talk_to.append(label)
        offers = tuple(
            Offer(
                number=number,
                action_type=affordance.action_type,
                target_label=None if affordance.target is None else labels.get(affordance.target),
                position=position,
            )
            for number, (position, affordance) in enumerate(
                (
                    (position, affordance)
                    for position, affordance in enumerate(observation.affordances)
                    if affordance.available and affordance.payload is not None
                ),
                start=1,
            )
        )
        return Situation(
            me=labels.get(observation.observer, names.person(observation.observer)),
            place=here,
            at=observation.at,
            present=tuple(present),
            talk_to=tuple(talk_to),
            offers=offers,
        )


def _learn(observation: Observation, names: NameBook) -> None:
    """Teaches every disclosed display name first (R-P6-2), in observation order."""
    for entity in observation.entities:
        for record in entity.components:
            if record.component_type != DISPLAY_NAME_COMPONENT:
                continue
            name = _display_name(record.payload)
            if name is not None:
                names.learn(entity.id, name)


def _display_name(payload: JsonValue) -> str | None:
    if isinstance(payload, dict):
        value = payload.get("name")
        if isinstance(value, str):
            return value
    return None


def _people(observation: Observation) -> list[EntityId]:
    """The observer and every person the observation lists, in ascending id order."""
    found = {observation.observer}
    found.update(entity.id for entity in observation.entities if entity.entity_type == "person")
    return sorted(found, key=int)


def _labels(people: list[EntityId], names: NameBook) -> dict[EntityId, str]:
    seen: dict[str, int] = {}
    labels: dict[EntityId, str] = {}
    for entity in people:
        name = names.person(entity)
        seen[name] = seen.get(name, 0) + 1
        labels[entity] = name if seen[name] == 1 else f"{name} ({seen[name]})"
    return labels


def _is_at(observation: Observation, entity: EntityId, place: PlaceId | None) -> bool:
    if place is None:
        return False
    for record in observation.entities:
        if record.id == entity:
            return record.location is not None and record.location.place.entity == place.entity
    return False
