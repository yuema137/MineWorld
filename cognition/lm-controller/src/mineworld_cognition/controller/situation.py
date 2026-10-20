"""`Situation`: what the observer can see and do, in labels the model may use (pr-s10-p6 §5.2.2, C5).

The model never sees an entity id. People get labels from the name book (`NameBook.person`); equal names are
suffixed ` (2)`, ` (3)` in ascending `EntityId` order, so the labels are unique and deterministic. Offers are
the observation's *available, complete* affordances (`ARC-34`), numbered from 1 in observation order; each keeps
its index into `observation.affordances`, so `intent.py` can re-locate it in the newest observation. `talk_to`
reads the server's verdict (`available`) and computes nothing. Pure: no clock, no randomness (I-3).

`plans` (`pr-le-leave-early.md` §0.4 item 1, LE-c C3) states the next part of my day as schedule discloses it
(`agenda.next`) and, for a punctual part, about how long the way there takes, from movement's `ways` at the
seat's stride cadence (`mineworld_sdk.ways`; one `walk-step` per wall second is `time_scale` world seconds). It
states facts only and gives no advice about when to leave. `places` are the places a `go` may name: those in
`ways`, then the next part's place if it is not among them, labelled from the name book and made unique.
"""

from __future__ import annotations

from typing import NamedTuple

from mineworld_sdk import ways
from mineworld_sdk.wire.contract import Observation, PlaceId
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, JsonValue
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.controller.attributes import description
from mineworld_cognition.memory.anchors import ANCHOR_KINDS, Anchor
from mineworld_cognition.memory.names import NameBook

DISPLAY_NAME_COMPONENT = "display-name"
AGENDA_COMPONENT = "agenda"
WALKING_COMPONENT = "walking"
NEEDS_COMPONENT = "needs"
ASLEEP_COMPONENT = "asleep"
SLEEP = ActionTypeId("sleep")
TALK = ActionTypeId("talk")
DAY_S = 86_400


class Person(CognitionModel):
    label: str
    entity: EntityId
    asleep: bool = False
    """Whether sleep discloses this person as asleep (`asleep`, S35 N-e): a fact, shown as `(asleep)`."""
    description: str = Field(default="", exclude_if=lambda value: value == "")
    """Who I see (S30-f §5.6): the person's `observed` attributes, composed from their own entity record; empty
    when none are disclosed, and then left out of the log so an older log reads the same."""


class Feeling(CognitionModel):
    """One need of mine and the pack's word for it now (`needs`' `words` map, in the record's order)."""

    need: str
    word: str


class Offer(CognitionModel):
    number: int
    action_type: ActionTypeId
    target_label: str | None
    position: int
    up_to: int | None = None
    """The integer `amount` the complete affordance carries, when it carries one: the most the model may
    choose (an LM boss's `penalize`, `pr-le-leave-early.md` §0.5). Read from the payload; never a rule."""


class Place(CognitionModel):
    """A place a `go` may name: its label in the prompt, and its id (never shown)."""

    label: str
    entity: EntityId


class NextPart(CognitionModel):
    """The next part of my day, as schedule discloses it (`agenda.next`), with the way's estimate."""

    label: str
    place: EntityId
    from_at: int
    punctual: bool
    minutes: int | None
    """About how many world minutes the way takes at the seat's cadence; `None` when it is not in `ways`."""


class Situation(CognitionModel):
    me: str
    place: str
    at: int
    present: tuple[Person, ...]
    talk_to: tuple[str, ...]
    offers: tuple[Offer, ...]
    plans: tuple[str, ...] = ()
    places: tuple[Place, ...] = ()
    next: NextPart | None = None
    walking: bool = False
    feelings: tuple[Feeling, ...] = ()
    """My needs and the pack's words for them, as `needs` discloses them to me (S35 N-e); empty when none."""
    asleep_since: int | None = None
    """When I fell asleep, in world seconds, while sleep discloses me as asleep; `None` when I am awake."""
    alarm: int | None = None
    """When my alarm rings, while I sleep and sleep discloses it (to the sleeper only)."""
    sleep_offered: bool = False
    """Whether sleep offers me `sleep` now (S35 N-e): the act takes `minutes`, so its offer is never complete."""

    @staticmethod
    def of(observation: Observation, names: NameBook, *, stride_s: int = 1) -> Situation:
        """`stride_s` is the world seconds between two strides of this seat's walk: `time_scale` for a seat
        whose walk is paced at one `walk-step` per wall second."""
        _learn(observation, names)
        people = _people(observation)
        labels = _labels(people, names)
        here_id = None if observation.self_location is None else observation.self_location.place
        here = names.place(None if here_id is None else here_id.entity)
        present = sorted(
            (
                Person(
                    label=labels[entity],
                    entity=entity,
                    asleep=_asleep_record(observation, entity) is not None,
                    description=description(observation, entity),
                )
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
                target_label=(
                    _payload_person(affordance.payload, labels, names)
                    if affordance.target is None
                    else labels.get(affordance.target)
                ),
                position=position,
                up_to=_amount(affordance.payload),
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
        my_sleep = _asleep_record(observation, observation.observer)
        known_ways = ways.of(observation)
        next_part = _next_part(observation, known_ways, stride_s)
        places = _places(known_ways, next_part, names)
        return Situation(
            me=labels.get(observation.observer, names.person(observation.observer)),
            place=here,
            at=observation.at,
            present=tuple(present),
            talk_to=tuple(talk_to),
            offers=offers,
            plans=_plans(next_part, places),
            places=places,
            next=next_part,
            walking=_component(observation, WALKING_COMPONENT) is not None,
            feelings=_feelings(observation),
            asleep_since=None if my_sleep is None else my_sleep.since,
            alarm=None if my_sleep is None else my_sleep.alarm,
            sleep_offered=any(
                a.action_type == SLEEP and a.available for a in observation.affordances
            ),
        )


def _payload_person(payload: JsonValue, labels: dict[EntityId, str], names: NameBook) -> str | None:
    """The person an untargeted complete offer names in its payload (employment's `penalize` names the
    employee there, so a manager may act from anywhere): the first top-level person reference."""
    if not isinstance(payload, dict):
        return None
    for value in payload.values():
        if isinstance(value, dict) and value.get("entity_type") == "person":
            entity = value.get("entity")
            if isinstance(entity, str):
                return labels.get(EntityId(entity), names.person(EntityId(entity)))
    return None


def _amount(payload: JsonValue) -> int | None:
    amount = payload.get("amount") if isinstance(payload, dict) else None
    return amount if isinstance(amount, int) and not isinstance(amount, bool) else None


def clock(at: int) -> str:
    """A world instant as the time of day, `HH:MM`."""
    of_day = at % DAY_S
    return f"{of_day // 3600:02d}:{of_day % 3600 // 60:02d}"


def _component(observation: Observation, component: str) -> JsonValue:
    for entity in observation.entities:
        if entity.id != observation.observer:
            continue
        for record in entity.components:
            if record.component_type == component:
                return record.payload
    return None


class Asleep(NamedTuple):
    """sleep's `asleep` record for one entity: since when, and the alarm where it is disclosed (the sleeper's)."""

    since: int
    alarm: int | None


def _asleep_record(observation: Observation, entity: EntityId) -> Asleep | None:
    """`entity`'s `asleep` record, when sleep says the entity sleeps and gives an instant it began."""
    for perceived in observation.entities:
        if perceived.id != entity:
            continue
        for record in perceived.components:
            found = record.payload
            if record.component_type != ASLEEP_COMPONENT or not isinstance(found, dict):
                continue
            since = _whole(found.get("since"))
            if found.get("asleep") is True and since is not None:
                return Asleep(since=since, alarm=_whole(found.get("alarm")))
    return None


def _whole(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None


def _feelings(observation: Observation) -> tuple[Feeling, ...]:
    """My needs, in the order the `needs` record's `words` map gives them. Nothing is named here: the pack's
    own need names and words are passed on as they are."""
    needs = _component(observation, NEEDS_COMPONENT)
    words = needs.get("words") if isinstance(needs, dict) else None
    if not isinstance(words, dict):
        return ()
    return tuple(
        Feeling(need=need, word=word) for need, word in words.items() if isinstance(word, str)
    )


def _next_part(
    observation: Observation, known: tuple[ways.Way, ...], stride_s: int
) -> NextPart | None:
    agenda = _component(observation, AGENDA_COMPONENT)
    following = agenda.get("next") if isinstance(agenda, dict) else None
    if not isinstance(following, dict):
        return None
    place, label, from_at = following.get("place"), following.get("label"), following.get("from")
    entity = place.get("entity") if isinstance(place, dict) else None
    if (
        not isinstance(entity, str)
        or not isinstance(label, str)
        or not isinstance(from_at, int)
        or isinstance(from_at, bool)
    ):
        return None
    way = next((way for way in known if way.place == entity), None)
    return NextPart(
        label=label,
        place=EntityId(entity),
        from_at=from_at,
        punctual=following.get("punctual") is True,
        minutes=None if way is None else ways.minutes(ways.estimate(way, stride_s)),
    )


def _places(
    known: tuple[ways.Way, ...], next_part: NextPart | None, names: NameBook
) -> tuple[Place, ...]:
    entities = [way.place for way in known]
    if next_part is not None and next_part.place not in entities:
        entities.append(next_part.place)
    seen: dict[str, int] = {}
    places: list[Place] = []
    for entity in entities:
        name = names.place(entity)
        seen[name] = seen.get(name, 0) + 1
        label = name if seen[name] == 1 else f"{name} ({seen[name]})"
        places.append(Place(label=label, entity=entity))
    return tuple(places)


def _plans(next_part: NextPart | None, places: tuple[Place, ...]) -> tuple[str, ...]:
    """The plan lines, in my own words (§0.4 item 1). Facts only: no advice about when to leave."""
    if next_part is None:
        return ()
    where = next((place.label for place in places if place.entity == next_part.place), "somewhere")
    if not next_part.punctual:
        return (f"Next: {next_part.label} at {where} from {clock(next_part.from_at)}.",)
    way = (
        "I do not know how far it is."
        if next_part.minutes is None
        else f"Walking there takes about {next_part.minutes} minutes."
    )
    return (
        f"My {next_part.label} at {where} starts at {clock(next_part.from_at)}; I mean to be"
        f" there when it begins. {way}",
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


QUERY_ANCHORS = 16
"""The most anchors a consult's recall query carries (pr-s28d §5.7, R-d-4)."""


def anchors_of(observation: Observation) -> tuple[Anchor, ...]:
    """The places, items and organizations the person is with now, as the world disclosed them (pr-s28d
    §5.7): the observer's place first, then each observed entity of one of those kinds, in the observation's
    (ascending id) order, without repeats, at most `QUERY_ANCHORS`."""
    found: dict[EntityId, Anchor] = {}
    here = observation.self_location
    if here is not None:
        found[here.place.entity] = Anchor(here.place.entity, "place")
    for entity in observation.entities:
        kind = entity.entity_type
        if kind in ANCHOR_KINDS and entity.id not in found:
            found[entity.id] = Anchor(entity.id, kind)
    return tuple(found.values())[:QUERY_ANCHORS]


def _is_at(observation: Observation, entity: EntityId, place: PlaceId | None) -> bool:
    if place is None:
        return False
    for record in observation.entities:
        if record.id == entity:
            return record.location is not None and record.location.place.entity == place.entity
    return False
