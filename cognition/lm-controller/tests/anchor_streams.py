"""Perceived streams for the anchored scope's tests (pr-s28d §6.1). Test-only; imports nothing of the code under
test.

- `payload(rng)` draws one `generated` payload: random keys, typed references of every kind (persons among them)
  nested 0 … 4 levels deep, duplicates, and sometimes more than 16 anchors, or no JSON object at all.
- `generated(seed, me, count)` is one seat's stream of such facts, with random event types, places and times:
  nothing in it is an id, a name, an event type or a payload shape any code knows.
- `lakeside(seed, days)` is a multi-seat stream shaped like `worlds/lakeside`: arrivals at a pier, a bakery and a
  boathouse, `fished` with a typed rod, `items-transferred` with a typed item, `walk-started` with its nested
  destination. Every member of an audience gets the same envelope.
"""

from __future__ import annotations

import random
from collections.abc import Sequence

from memory_fixtures import fact, named, walk_started
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, JsonValue

DAY = 86_400
KINDS = ("place", "item", "organization", "person")


def _id(rng: random.Random) -> str:
    return str(rng.randrange(10**7, 10**19))


def _ref(entity: str, kind: str) -> JsonValue:
    return {"entity": entity, "entity_type": kind}


def _key(rng: random.Random) -> str:
    return "".join(rng.choice("abcdefghijklmnopqrstuvwxyz_") for _ in range(rng.randrange(1, 9)))


def _nest(rng: random.Random, value: JsonValue, depth: int) -> JsonValue:
    for _ in range(depth):
        value = {_key(rng): value} if rng.random() < 0.7 else [rng.randrange(9), value]
    return value


def payload(rng: random.Random, pool: Sequence[tuple[str, str]] = ()) -> JsonValue:
    """One `generated` payload. `pool` is a set of `(entity, kind)` to reuse, so entities recur across facts."""
    shape = rng.random()
    if shape < 0.05:
        return None
    if shape < 0.1:
        return rng.choice([3, "text", True, [], {}])
    count = rng.randrange(20, 30) if rng.random() < 0.05 else rng.randrange(0, 6)
    body: dict[str, JsonValue] = {}
    for index in range(count):
        if pool and rng.random() < 0.6:
            entity, kind = rng.choice(list(pool))
        else:
            entity, kind = _id(rng), rng.choice(KINDS)
        reference: JsonValue = _ref(entity, kind)
        if rng.random() < 0.1:
            reference = {"entity": entity, "entity_type": kind, "extra": 1}  # not a reference
        body[f"{_key(rng)}{index}"] = _nest(rng, reference, rng.randrange(0, 5))
    if rng.random() < 0.3:
        body["note"] = "plain words"
    return body


def generated(
    seed: int, me: EntityId, count: int = 400, others: Sequence[EntityId] = ()
) -> tuple[list[PerceivedEvent], list[tuple[str, str]]]:
    """One seat's `generated` stream, ascending ids, and the pool of anchored entities it draws from."""
    rng = random.Random(seed)
    places = [_id(rng) for _ in range(6)]
    pool = [(entity, rng.choice(KINDS[:3])) for entity in (_id(rng) for _ in range(12))]
    pool += [(place, "place") for place in places] + [(me, "item")]  # never an anchor of mine
    people = list(others) or [EntityId(_id(rng)) for _ in range(3)]
    facts: list[PerceivedEvent] = []
    event = rng.randrange(10**6, 10**7)
    at = 6 * 3600
    for _ in range(count):
        event += rng.randrange(1, 50)
        at += rng.randrange(30, 2400)
        kind = "".join(rng.choice("abcdefghij") for _ in range(6))
        place = rng.choice([*places, None])
        who = [me, *rng.sample(people, rng.randrange(0, len(people) + 1))]
        facts.append(
            fact(
                event,
                at,
                kind,
                payload(rng, pool),
                subjects=[who[0]],
                participants=who,
                place=None if place is None else EntityId(place),
            )
        )
    return facts, pool


# ── lakeside-like ─────────────────────────────────────────────────────────────────────────────


LAKE_PEOPLE = (EntityId("5100000001"), EntityId("5100000002"), EntityId("5100000003"))
LAKE_SEATS = (EntityKey("ada"), EntityKey("bram"), EntityKey("cleo"))
LAKE_NAMES = ("Ada", "Bram", "Cleo")
PIER, BAKERY, BOATHOUSE = EntityId("5200000001"), EntityId("5200000002"), EntityId("5200000003")
LAKE_PLACES = {PIER: "the pier", BAKERY: "the bakery", BOATHOUSE: "the boathouse"}
ROD, BREAD, OAR = EntityId("5300000001"), EntityId("5300000002"), EntityId("5300000003")
LAKE_ITEMS = {ROD: "a fishing rod", BREAD: "a loaf", OAR: "an oar"}

type Heard = tuple[PerceivedEvent, frozenset[EntityKey]]


def _arrived(event: int, at: int, person: EntityId, place: EntityId) -> PerceivedEvent:
    return fact(
        event,
        at,
        "arrived",
        {"person": _ref(person, "person"), "place": _ref(place, "place")},
        subjects=[person],
        participants=[person],
        place=place,
    )


def _fished(event: int, at: int, person: EntityId, rod: EntityId) -> PerceivedEvent:
    return fact(
        event,
        at,
        "fished",
        {"catch": "trout", "rod": _ref(rod, "item")},
        subjects=[person],
        participants=[person],
        place=PIER,
    )


def _transferred(
    event: int, at: int, giver: EntityId, taker: EntityId, item: EntityId, place: EntityId
) -> PerceivedEvent:
    return fact(
        event,
        at,
        "items-transferred",
        {"from": _ref(giver, "person"), "to": _ref(taker, "person"), "item": _ref(item, "item")},
        subjects=[giver, taker],
        participants=[giver, taker],
        place=place,
    )


def lakeside(seed: int, days: int, per_day: int = 40) -> list[Heard]:
    """The `lakeside-like` arm: the people's names first (heard by all), then `per_day` facts a day, each heard
    by its actor and, at random, the others."""
    rng = random.Random(seed)
    everyone = frozenset(LAKE_SEATS)
    heard: list[Heard] = []
    event = 100
    for person, name in zip(LAKE_PEOPLE, LAKE_NAMES, strict=True):
        heard.append((named(event, person, name), everyone))
        event += 1
    places = list(LAKE_PLACES)
    for day in range(days):
        at = day * DAY + 7 * 3600
        for _ in range(per_day):
            event += 1 + rng.randrange(3)
            at += rng.randrange(60, 900)
            person = rng.choice(LAKE_PEOPLE)
            place = rng.choice(places)
            roll = rng.random()
            if roll < 0.3:
                one = _arrived(event, at, person, place)
            elif roll < 0.5:
                one = _fished(event, at, person, rng.choice([ROD, OAR]))
                place = PIER
            elif roll < 0.7:
                other = rng.choice([p for p in LAKE_PEOPLE if p != person])
                one = _transferred(event, at, person, other, rng.choice(list(LAKE_ITEMS)), place)
            else:
                one = walk_started(event, at, person, rng.choice(places))
                # walk_started's envelope place is the destination's; it is heard where the walker is
            audience = {
                seat
                for seat, who in zip(LAKE_SEATS, LAKE_PEOPLE, strict=True)
                if who == person or rng.random() < 0.6
            }
            heard.append((one, frozenset(audience)))
    return heard


THING_NAMES: dict[EntityId, str] = {**LAKE_PLACES, **LAKE_ITEMS}
"""The names an observation discloses for lakeside's places and things (a test teaches them to the name book)."""


def streams(heard: Sequence[Heard]) -> dict[EntityKey, list[PerceivedEvent]]:
    out: dict[EntityKey, list[PerceivedEvent]] = {}
    for one, audience in heard:
        for seat in sorted(audience):
            out.setdefault(seat, []).append(one)
    return out
