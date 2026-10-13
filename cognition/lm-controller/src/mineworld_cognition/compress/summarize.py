"""Summary text: the `Summarizer` interface and `StructuralSummarizer`, its model-free default (§5.7).

`StructuralSummarizer` is a pure function of its inputs: counts, names, times, and at most one gist
quoted from the records. It is the only summarizer ingestion calls, so `AC-10` holds with no model.
Its text never holds an id; a summary's provenance is its citation, kept beside the text, never in it.
"""

from __future__ import annotations

from collections import Counter
from collections.abc import Sequence
from typing import Protocol

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Episode, L0Record

DAY_S = 86400


def day_number(at: int) -> int:
    """The day a world time falls on, counted from 1 as `mineworld inspect` prints it."""
    return at // DAY_S + 1


def clock(at: int) -> str:
    seconds = at % DAY_S
    return f"{seconds // 3600:02d}:{seconds % 3600 // 60:02d}"


def counted(number: int, noun: str) -> str:
    return f"{number} {noun}" if number == 1 else f"{number} {noun}s"


class Summarizer(Protocol):
    def episode(
        self, records: Sequence[L0Record], names: NameBook, *, place: EntityId | None
    ) -> str: ...

    def chapter(
        self, episodes: Sequence[Episode], names: NameBook, *, newcomers: Sequence[EntityId]
    ) -> str: ...


class StructuralSummarizer:
    def episode(
        self, records: Sequence[L0Record], names: NameBook, *, place: EntityId | None
    ) -> str:
        """`Day 12, 09:10\N{EN DASH}09:40, somewhere: with Bob Achterberg; 6 notable moments, 3 mine;
        4 others came and went; Bob Achterberg said to me: «…»`."""
        first, last = records[0], records[-1]
        notable = [record for record in records if record.salience == "notable"]
        company = list(dict.fromkeys(c for record in notable for c in record.counterparts))
        everyone = dict.fromkeys(c for record in records for c in record.counterparts)
        others = [entity for entity in everyone if entity not in company]
        head = f"Day {day_number(first.at)}, {clock(first.at)}\N{EN DASH}{clock(last.at)}, "
        head += f"{names.place(place)}: "
        head += f"with {names.people(company)}" if company else "alone"
        parts = [head]
        if notable:
            mine = sum(1 for record in notable if record.mine)
            parts.append(f"{counted(len(notable), 'notable moment')}, {mine} mine")
        if others:
            parts.append(f"{len(others)} {'other' if len(others) == 1 else 'others'} came and went")
        if notable:
            theirs = [record for record in notable if not record.mine]
            parts.append((theirs or notable)[-1].gist)
        return "; ".join(parts)

    def chapter(
        self, episodes: Sequence[Episode], names: NameBook, *, newcomers: Sequence[EntityId]
    ) -> str:
        """`Week 3: 61 episodes, mostly at somewhere; most often with Bob Achterberg (14), Carol Mensah
        (9); first met Dev Raman.`"""
        places = Counter(episode.place for episode in episodes)
        # Counter keeps first-seen order, and `most_common` is stable, so ties go to the earliest.
        place = places.most_common(1)[0][0]
        people = Counter(c for episode in episodes for c in episode.counterparts)
        week = episodes[0].day // 7 + 1
        parts = [
            f"Week {week}: {counted(len(episodes), 'episode')}, mostly at {names.place(place)}"
        ]
        if people:
            often = ", ".join(f"{names.person(c)} ({n})" for c, n in people.most_common(3))
            parts.append(f"most often with {often}")
        if newcomers:
            parts.append(f"first met {names.people(newcomers)}")
        return "; ".join(parts) + "."
