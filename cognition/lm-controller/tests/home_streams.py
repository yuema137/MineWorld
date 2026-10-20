"""Multi-seat perceived streams for the memory home's tests (pr-s28a §5.9). Test-only; imports nothing of the code
under test.

**Synthetic.** `world(seed, days)` is one ordered list of `(fact, audience)`: every draw is SplitMix64 of `(seed,
index, salt)` as `retention_stream.py` does, so any prefix is reproducible and the same on every platform. Each
seat's stream (`seat_streams`) is the facts whose audience holds it, and it is **the same `PerceivedEvent`
object** in every member's stream: a world gives every observer one envelope (step-28 F-S28-3). An audience holds
1 to 12 of the twelve seats, about 13 % of facts one seat only (step-28 F-S28-2's shape). Every renderer's event
type appears, plus one no code knows (`fished`) and one whose only subject is a place and which has no
participant (`lamp-lit`, as social-cafe's `passage-opened`), so `fact_who` holds subject-only rows.

**Real.** `cafe12(scratch)` runs `mineworld run worlds/social-cafe --headless --seed 7 --days 2` and exports the
twelve people's perceived facts through `mineworld perceived --json`, as `ac10_harness` does for two.

`queries(seed, seat, point, now, cast)` draws the 200 recall queries a test renders at one point: times within the
last day before `now`, 0 … 3 counterparts from the cast, 0 … 3 words of the vocabulary. They are plain tuples,
so this module stays free of the code under test.
"""

from __future__ import annotations

import re
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

from binary import REPOSITORY, mineworld
from memory_fixtures import (
    arrived,
    became_acquainted,
    conversation_started,
    entered,
    fact,
    named,
    relationship_changed,
    spoke,
    walk_ended,
    walk_started,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, JsonValue, WorldInstanceId
from retention_stream import CROWD, LEVELS, NAMES, PEOPLE, PLACES, WORDS, draw, utterance

DAY = 86_400
SEATS: tuple[EntityKey, ...] = tuple(EntityKey(name.split()[0].lower()) for name in NAMES)
"""The twelve seats' keys, in the cast's order (`alice`, `bob`, … `lena`)."""
CAST: dict[EntityKey, EntityId] = dict(zip(SEATS, PEOPLE, strict=True))
STRANGERS = [EntityId(f"73000000{n:02d}") for n in range(8)]
"""People no seat is: counterparts that are not home-mates, and names learned by some seats only."""
STRANGER_NAMES = ["Mara", "Nils", "Oona", "Pavel", "Quinn", "Rosa", "Sami", "Tove"]
PER_DAY = 60
QUERIES = 200

type Heard = tuple[PerceivedEvent, frozenset[EntityKey]]


def _audience(seed: int, index: int) -> list[EntityKey]:
    """1 … 12 seats, in a seeded order: one seat for about 13 % of facts, otherwise 2 … 12 evenly."""
    count = 1 if draw(seed, index, 20) % 100 < 13 else 2 + draw(seed, index, 21) % 11
    order = sorted(SEATS, key=lambda seat: draw(seed, index, 30 + SEATS.index(seat)))
    return order[:count]


def world(seed: int, days: int, per_day: int = PER_DAY) -> list[Heard]:
    """The world's perceived facts, in id order, each with the seats that perceived it."""
    everyone = frozenset(SEATS)
    out: list[Heard] = []
    event = 1000
    for person, name in zip(PEOPLE, NAMES, strict=True):
        event += 1
        out.append((named(event, person, name), everyone))
    index = 0
    for day in range(days):
        if day > 0 and draw(seed, day, 1) % 9 == 0:
            continue  # a quiet day: the next day's pass follows a jump
        for slot in range(per_day):
            index += 1
            event += 1 + draw(seed, index, 2) % 5
            at = day * DAY + 7 * 3600 + slot * (14 * 3600 // per_day)
            place = PLACES[draw(seed, day * 1000 + slot // 12, 3) % len(PLACES)]
            audience = _audience(seed, index)
            out.append((_one(seed, index, event, at, place, audience), frozenset(audience)))
    return out


def _one(
    seed: int, index: int, event: int, at: int, place: EntityId, audience: list[EntityKey]
) -> PerceivedEvent:
    actor = CAST[audience[0]]
    stranger = STRANGERS[draw(seed, index, 5) % len(STRANGERS)]
    other = CAST[audience[1]] if len(audience) > 1 else stranger
    kind = draw(seed, index, 4) % 100
    if kind < 30:
        return spoke(event, at, actor, other, utterance(seed, index), place)
    if kind < 40:
        return arrived(event, at, actor, place)
    if kind < 46:
        return entered(event, at, actor, PLACES[0], place)
    if kind < 56:
        return conversation_started(event, at, actor, other, place)
    if kind < 62:
        return became_acquainted(event, at, actor, other, place)
    if kind < 68:
        level = LEVELS[draw(seed, index, 8) % len(LEVELS)]
        return relationship_changed(event, at, actor, other, "acquaintance", level, place)
    if kind < 74:
        return walk_started(event, at, actor, place)
    if kind < 80:
        outcome = ("arrived", "interrupted")[draw(seed, index, 9) % 2]
        return walk_ended(event, at, actor, outcome, place)
    if kind < 84:
        name = STRANGER_NAMES[STRANGERS.index(stranger)]
        payload: JsonValue = {"person": {"entity": stranger, "entity_type": "person"}, "name": name}
        return fact(
            event, at, "named", payload, subjects=[stranger], participants=[stranger], place=None
        )
    if kind < 92:
        who = [actor, other, stranger]
        return fact(
            event, at, "fished", {"catch": "trout"}, subjects=[other], participants=who, place=place
        )
    return fact(event, at, "lamp-lit", {"lamp": 1}, subjects=[place], participants=[], place=place)


def seat_streams(heard: Sequence[Heard]) -> dict[EntityKey, list[PerceivedEvent]]:
    """Each seat's perceived stream: the same fact objects, in id order."""
    streams: dict[EntityKey, list[PerceivedEvent]] = {seat: [] for seat in SEATS}
    for one, audience in heard:
        for seat in SEATS:
            if seat in audience:
                streams[seat].append(one)
    return streams


HEAVY = (7, 2, 2800, 220)
"""`heavy_world`'s `(seed, days, per_day, gathered)` (pr-s28b §6.4, fixed at C5 from S28-a's
`test_retention_size.BUSY`): ≈ 8.7 KB of positional dump per fact for each seat that holds it, ≈ 24 MB a day, so
every seat crosses its trigger at least twice in the run."""
SOLO_EVERY = 7
"""Every 7th fact of `heavy_world` is perceived by one seat only (drawn), so singly held records exist too."""


def heavy_world(
    seed: int,
    days: int,
    seats: int = 3,
    per_day: int = HEAVY[2],
    gathered: int = HEAVY[3],
) -> dict[EntityKey, list[PerceivedEvent]]:
    """The multi-seat heavy stream (pr-s28b §6.4): `retention_stream.heavy`'s busy crowd, whose participants
    are the first `seats` seats and `gathered` strangers, so every record is held by every seat (the worst case
    for I-a-6 (a)); every `SOLO_EVERY`-th fact is one drawn seat's alone. All seats are named first. Each seat's
    stream holds the same fact objects, in id order."""
    chosen = SEATS[:seats]
    everyone = frozenset(chosen)
    heard: list[Heard] = []
    event = 1000
    for seat in chosen:
        event += 1
        heard.append((named(event, CAST[seat], NAMES[SEATS.index(seat)]), everyone))
    index = 0
    for day in range(days):
        for slot in range(per_day):
            index += 1
            event += 1 + draw(seed, index, 2) % 3
            at = day * DAY + 6 * 3600 + slot * (16 * 3600 // per_day)
            place = PLACES[(slot // 40 + day) % len(PLACES)]
            audience = (
                frozenset({chosen[draw(seed, index, 7) % seats]})
                if index % SOLO_EVERY == 0
                else everyone
            )
            offset = draw(seed, index, 9) % len(CROWD)
            crowd = [CROWD[(offset + n) % len(CROWD)] for n in range(gathered)]
            who = [*(CAST[seat] for seat in chosen if seat in audience), *crowd]
            one = fact(
                event,
                at,
                "crowd-gathered",
                {"size": gathered},
                subjects=[crowd[0]],
                participants=who,
                place=place,
            )
            heard.append((one, audience))
    return {seat: facts for seat, facts in seat_streams(heard).items() if seat in everyone}


def day_frames(facts: Sequence[PerceivedEvent]) -> list[list[PerceivedEvent]]:
    """Frames that each end at a first fact of a new day, as `retention_driver.days`: a day's pass then ends
    the frame that opens the day."""
    frames: list[list[PerceivedEvent]] = []
    frame: list[PerceivedEvent] = []
    day: int | None = None
    for one in facts:
        frame.append(one)
        if day is not None and one.at // DAY > day:
            frames.append(frame)
            frame = []
        day = one.at // DAY
    if frame:
        frames.append(frame)
    return frames


def interleave(
    streams: dict[EntityKey, list[PerceivedEvent]], frame: int, seed: int
) -> list[tuple[EntityKey, list[PerceivedEvent]]]:
    """The seats' frames of `frame` facts, merged in a seeded order: at each step one seat that has frames
    left is drawn (SplitMix64 of `(seed, step)`), and its next frame follows. Each seat's own order is kept;
    the seats' relative order is the draw's alone."""
    pending = {
        seat: [facts[start : start + frame] for start in range(0, len(facts), frame)]
        for seat, facts in sorted(streams.items())
    }
    merged: list[tuple[EntityKey, list[PerceivedEvent]]] = []
    step = 0
    while any(pending.values()):
        ready = [seat for seat, frames in pending.items() if frames]
        seat = ready[draw(seed, step, 90) % len(ready)]
        merged.append((seat, pending[seat].pop(0)))
        step += 1
    return merged


type Query = tuple[int, tuple[EntityId, ...], str]
"""`(at, counterparts, words)`: a `RecallQuery`'s fields."""


def queries(seed: int, seat: int, point: int, now: int, cast: Sequence[EntityId]) -> list[Query]:
    """The 200 recall queries of one seat at one point, drawn from `(seed, seat, point)` alone."""
    out: list[Query] = []
    base = seed * 1_000_003 + seat * 10_007 + point
    for n in range(QUERIES):
        at = max(0, now - draw(base, n, 40) % DAY)
        who = tuple(cast[draw(base, n, 50 + k) % len(cast)] for k in range(draw(base, n, 41) % 4))
        words = " ".join(
            WORDS[draw(base, n, 60 + k) % len(WORDS)] for k in range(draw(base, n, 42) % 4)
        )
        out.append((at, who, words))
    return out


@dataclass(frozen=True)
class Cafe12:
    """The real 12-seat stream: social-cafe, seed 7, 2 days."""

    save: Path
    instance: WorldInstanceId
    streams: dict[EntityKey, list[PerceivedEvent]]
    me: dict[EntityKey, EntityId]


CAFE_WORLD = "worlds/social-cafe"
CAFE_DAYS = 2


def cafe12(scratch: Path) -> Cafe12:
    save = scratch / "save"
    mineworld(
        "run", CAFE_WORLD, "--headless", "--seed", "7", "--days", str(CAFE_DAYS), "--save", save
    )
    found = re.search(r"instance ([0-9a-f]{32})", mineworld("inspect", save))
    assert found is not None, "mineworld inspect printed no world instance"
    people = sorted(path.stem for path in (REPOSITORY / CAFE_WORLD / "people").glob("*.yaml"))
    streams: dict[EntityKey, list[PerceivedEvent]] = {}
    me: dict[EntityKey, EntityId] = {}
    for person in people:
        out = mineworld("perceived", CAFE_WORLD, "--save", save, "--person", person, "--json")
        facts = [PerceivedEvent.model_validate_json(line) for line in out.splitlines() if line]
        yaml = (REPOSITORY / CAFE_WORLD / "people" / f"{person}.yaml").read_text(encoding="utf-8")
        name = re.search(r"^name:\s*(.+?)\s*$", yaml, re.MULTILINE)
        assert name is not None, person
        (entity,) = {
            _named(f) for f in facts if f.event_type == "named" and _name(f) == name.group(1)
        }
        streams[EntityKey(person)] = facts
        me[EntityKey(person)] = entity
    return Cafe12(save, WorldInstanceId(found.group(1)), streams, me)


def _body(one: PerceivedEvent) -> dict[str, JsonValue]:
    body = one.payload.payload
    assert isinstance(body, dict), one.id
    return body


def _named(one: PerceivedEvent) -> EntityId:
    person = _body(one)["person"]
    assert isinstance(person, dict)
    entity = person["entity"]
    assert isinstance(entity, str)
    return EntityId(entity)


def _name(one: PerceivedEvent) -> str:
    return str(_body(one)["name"])
