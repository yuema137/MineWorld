"""A seeded synthetic perceived stream for retention tests (pr-fg-b §8.3). Test-only; imports nothing of the
code under test.

`stream(seed, days, per_day)` is a pure function: every draw is SplitMix64 of `(seed, index, salt)`, with
no RNG state, so any prefix is reproducible and the same on every platform. A fixed cast of twelve people
(Alice, whose memory it is, and eleven others) and six places; the event types memory's renderers know,
plus one no code knows (`fished`, I-9). Ids ascend strictly with gaps, as a real world's do; some days are
silent, so a pass can follow a jump over several days. The perceived set a test's oracle uses is the
generator's own output.
"""

from __future__ import annotations

from collections.abc import Iterator

from memory_fixtures import (
    ALICE,
    arrived,
    became_acquainted,
    conversation_started,
    entered,
    fact,
    named,
    relationship_changed,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

DAY = 86_400
MASK = (1 << 64) - 1
PEOPLE = [ALICE, *(EntityId(f"71000000{n:02d}") for n in range(1, 12))]
PLACES = [EntityId(f"52000000{n:02d}") for n in range(6)]
NAMES = ["Alice Moreau", "Bob", "Carol", "Dev", "Erin", "Felix", "Grace", "Hana", "Ivan", "Jun",
         "Kofi", "Lena"]  # fmt: skip
WORDS = ["rain", "coffee", "umbrella", "bread", "market", "café", "été", "雨", "tomorrow", "music",
         "letter", "garden", "river", "lamp", "zeppelin", "noon"]  # fmt: skip
LEVELS = ["acquaintance", "friendly", "friend", "close friend"]


def splitmix64(value: int) -> int:
    value = (value + 0x9E3779B97F4A7C15) & MASK
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK
    return value ^ (value >> 31)


def draw(seed: int, index: int, salt: int) -> int:
    return splitmix64(splitmix64(seed) ^ (index << 8) ^ salt)


def utterance(seed: int, index: int) -> str:
    """1 … 480 UTF-8 bytes of words, multibyte ones included."""
    budget = draw(seed, index, 11) % 480 + 1
    words: list[str] = []
    used = 0
    salt = 0
    while True:
        word = WORDS[draw(seed, index, 100 + salt) % len(WORDS)]
        size = len(word.encode("utf-8")) + (1 if words else 0)
        if used + size > budget:
            break
        words.append(word)
        used += size
        salt += 1
    return " ".join(words) or "a"


def stream(seed: int, days: int, per_day: int = 200) -> Iterator[PerceivedEvent]:
    event = 1000
    index = 0
    for person, name in zip(PEOPLE, NAMES, strict=True):
        event += 1
        yield named(event, person, name)
    for day in range(days):
        if day > 0 and draw(seed, day, 1) % 40 == 0:
            continue  # a silent day
        for slot in range(per_day):
            index += 1
            event += 1 + draw(seed, index, 2) % 7
            at = day * DAY + 7 * 3600 + slot * (14 * 3600 // per_day)
            place = PLACES[draw(seed, day * 1000 + slot // 20, 3) % len(PLACES)]
            yield _one(seed, index, event, at, place)


def _one(seed: int, index: int, event: int, at: int, place: EntityId) -> PerceivedEvent:
    kind = draw(seed, index, 4) % 100
    other = PEOPLE[1 + draw(seed, index, 5) % 11]
    third = PEOPLE[1 + draw(seed, index, 6) % 11]
    mine = draw(seed, index, 7) % 3
    if kind < 45:
        speaker, listener = ((other, ALICE), (ALICE, other), (other, third))[mine]
        return spoke(event, at, speaker, listener, utterance(seed, index), place)
    if kind < 60:
        return arrived(event, at, ALICE if mine == 0 else other, place)
    if kind < 70:
        return entered(event, at, ALICE if mine == 0 else other, PLACES[0], place)
    if kind < 80:
        speaker, listener = ((other, ALICE), (ALICE, other), (other, third))[mine]
        return conversation_started(event, at, speaker, listener, place)
    if kind < 85:
        return became_acquainted(event, at, ALICE if mine == 0 else other, third, place)
    if kind < 90:
        level = LEVELS[draw(seed, index, 8) % len(LEVELS)]
        person = ALICE if mine == 0 else other
        return relationship_changed(event, at, person, third, "acquaintance", level, place)
    who = [other, third] if mine else [other, ALICE, third]
    return fact(
        event, at, "fished", {"catch": "trout"}, subjects=[other], participants=who, place=place
    )
