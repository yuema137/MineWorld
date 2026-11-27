"""The names over the doorways of the observer's place (`ARC-363`).

The naming pack discloses, about the place an observer stands in, a `door-names` record: for each doorway
whose destination the world named, that destination's default name and its forms in other languages. It
reaches exactly the observers who receive movement's `passages` with `to_tags`, so a reader learns a name
over a doorway where it would learn the doorway's tags.

    {"leads_to": [{"to": {"entity": "12", "entity_type": "place"}, "name": "The Copper Kettle",
                   "forms": {"zh_Hans": "铜壶咖啡馆"}}]}

This module only reads: it picks no language and translates nothing. `form` selects among the forms the
world's author wrote, with the clients' fallback (the locale, then its language part, then the default).
"""

from __future__ import annotations

from dataclasses import dataclass, field

from mineworld_sdk.wire.contract import Observation, PerceivedEntity
from mineworld_sdk.wire.ids import EntityId, JsonValue

__all__ = ["DOOR_NAMES", "DoorName", "form", "of", "read"]

DOOR_NAMES = "door-names"
"""The component type the naming pack discloses door names under."""


@dataclass(frozen=True, slots=True)
class DoorName:
    """The name of the place one doorway leads to."""

    to: EntityId
    name: str
    forms: dict[str, str] = field(default_factory=dict[str, str])


def form(name: str, forms: dict[str, str], locale: str | None) -> str:
    """The form of a name for `locale` (a catalog stem such as `zh_Hans`): the form written for it, else the
    one form written for its language part (`zh`), when exactly one is, else the default `name`."""
    if not locale or not forms:
        return name
    if locale in forms:
        return forms[locale]
    language = locale.split("_")[0]
    found = [written for key, written in forms.items() if key.split("_")[0] == language]
    return found[0] if len(found) == 1 else name


def read(payload: JsonValue) -> tuple[DoorName, ...]:
    """The door names of a disclosed `door-names` payload, in the order disclosed. An entry that is not one
    is skipped, so a newer pack's shape is read as far as it is understood."""
    leads = payload.get("leads_to") if isinstance(payload, dict) else None
    found: list[DoorName] = []
    for entry in leads if isinstance(leads, list) else []:
        if not isinstance(entry, dict):
            continue
        to, name, forms = entry.get("to"), entry.get("name"), entry.get("forms")
        entity = to.get("entity") if isinstance(to, dict) else None
        if not isinstance(entity, str) or not isinstance(name, str) or not name:
            continue
        written = (
            {k: v for k, v in forms.items() if isinstance(v, str) and v}
            if isinstance(forms, dict)
            else {}
        )
        found.append(DoorName(to=EntityId(entity), name=name, forms=written))
    return tuple(found)


def of(observation: Observation) -> tuple[DoorName, ...]:
    """The door names disclosed about the observer's place; empty when none are."""
    if observation.self_location is None:
        return ()
    here = observation.self_location.place.entity
    place: PerceivedEntity | None = next(
        (entity for entity in observation.entities if entity.id == here), None
    )
    if place is None:
        return ()
    for record in place.components:
        if record.component_type == DOOR_NAMES:
            return read(record.payload)
    return ()
