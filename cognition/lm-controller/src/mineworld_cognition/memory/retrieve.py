"""Retrieval: the memory section a decision's context holds, bounded by construction (D-P4-9, D-P4-11).

Sections, in order, each filled in its order of preference until its line count or byte budget is
reached; a line that does not fit is not emitted, even in part:

- **with** (pr-p6c-fgd §4.1), only when the query names the person speaking to me: what we said to each
  other before — records of an exchange kind (`EXCHANGE_KINDS`) I took part in whose only counterpart is
  them, other than the line just heard: the first `WITH_FIRST` (the first words, which `consolidate` keeps)
  and then the newest, shown oldest first;
- **L3** stable facts: the counterparts met most often (ties: the earliest first meeting, then the
  entity id string);
- **L2** chapters, newest first;
- **L1** episodes: first those with a counterpart the query names, newest first; then, for the trigger's
  words, those holding a notable record FTS5 (or `LIKE`) matches, ranked by the number of distinct terms
  matched, then by recency, then by the first Event ID;
- **L0** notable records of the last 86 400 s before the query's time, newest first, except those an
  episode or the **with** section shown above already covers.

Every line is cut to 160 bytes at a character boundary, its citation token included, so the rendered
section never exceeds 6 000 bytes. Ordering is integers and strings only: no clock, no randomness, no
float (P4-3); `bm25()` is never read.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from typing import Literal

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.compress.chapters import chapters
from mineworld_cognition.compress.episodes import load_episodes
from mineworld_cognition.compress.stable import stable_facts, stable_text
from mineworld_cognition.compress.summarize import DAY_S, clock, day_number
from mineworld_cognition.memory import fts
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import (
    Citation,
    Episode,
    EventKey,
    EventRange,
    event_key,
    event_of,
)
from mineworld_cognition.memory.render.conversation import EXCHANGE_KINDS
from mineworld_cognition.memory.store import MemoryStore

LINE_BYTES = 160
SECTION_BYTES = 6000
ELLIPSIS = "…"


@dataclass(frozen=True)
class Budget:
    header: str
    lines: int
    bytes: int


SectionLevel = Literal["with", "L3", "L2", "L1", "L0"]
"""A section of the memory part: the four levels, and `with`, what the speaker and I said before."""

BUDGETS: dict[SectionLevel, Budget] = {
    "with": Budget("What {name} and I said to each other before:", 6, 1000),
    "L3": Budget("People I know:", 16, 1600),
    "L2": Budget("Recent weeks:", 4, 800),
    "L1": Budget("Episodes I remember:", 8, 1600),
    "L0": Budget("The last day:", 48, 1000),
}
"""Lines and bytes per section; a section's bytes include its header and every newline (QP4-7). `with`'s
1 000 bytes were L0's (2 000 before pr-p6c-fgd), so the sum, and the ceiling, stay 6 000 (`ARC-59`)."""
WITH_FIRST = 2
"""How many of the first exchanges with the speaker the `with` section shows before the newest."""


class RecallQuery(CognitionModel):
    at: int
    """The world time of the decision; L0 shows the 86 400 s before it."""
    counterparts: tuple[EntityId, ...] = ()
    """People present now or named in the trigger."""
    words: str = ""
    """The trigger's words, matched as literal terms (P4-4)."""
    speaker: EntityId | None = None
    """The person who just spoke to me, if someone did: the `with` section is theirs (pr-p6c-fgd §4.1)."""
    heard: EventId | None = None
    """The line just heard, which the prompt already shows: never repeated under `with`."""


class SectionLine(CognitionModel):
    level: SectionLevel
    text: str
    citation: Citation

    def render(self) -> str:
        return f"{self.text} {self.citation.token()}"


def header(level: SectionLevel, speaker: str | None) -> str:
    """A section's header; `with`'s names the speaker by label, never by id (I-10)."""
    return BUDGETS[level].header.replace("{name}", speaker or "someone")


class MemorySection(CognitionModel):
    lines: tuple[SectionLine, ...]
    speaker: str | None = None
    """The label of the person the `with` section is about, when it has lines."""

    def render(self) -> str:
        """One header per non-empty section, one line per entry; at most 6 000 bytes."""
        out: list[str] = []
        for level in BUDGETS:
            mine = [line.render() for line in self.lines if line.level == level]
            if mine:
                out.append(header(level, self.speaker) + "\n" + "".join(t + "\n" for t in mine))
        return "".join(out)

    def citations(self) -> tuple[EventRange, ...]:
        """Every range any line cites, sorted: what a decision's `recalls` may name (P6)."""
        ranges = {(r.first, r.last): r for line in self.lines for r in line.citation.ranges}
        return tuple(sorted(ranges.values(), key=lambda r: (event_key(r.first), event_key(r.last))))


def cut(text: str, citation: Citation) -> str:
    """`text`, shortened at a character boundary with an ellipsis so that the line with its citation
    token is at most `LINE_BYTES` bytes."""
    room = LINE_BYTES - len(citation.token().encode("utf-8")) - 1
    if len(text.encode("utf-8")) <= room:
        return text
    room -= len(ELLIPSIS.encode("utf-8"))
    kept = text.encode("utf-8")[:room].decode("utf-8", errors="ignore")
    return kept.rstrip() + ELLIPSIS


class _Section:
    def __init__(self, level: SectionLevel, speaker: str | None = None) -> None:
        self.level: SectionLevel = level
        self.budget = BUDGETS[level]
        self.used = len(header(level, speaker).encode("utf-8")) + 1
        self.lines: list[SectionLine] = []
        self.full = False

    def offer(self, text: str, citation: Citation) -> bool:
        """Adds the line if it fits the section's bounds; once one does not, the section is full."""
        if self.full:
            return False
        line = SectionLine(level=self.level, text=cut(text, citation), citation=citation)
        size = len(line.render().encode("utf-8")) + 1
        if len(self.lines) == self.budget.lines or self.used + size > self.budget.bytes:
            self.full = True
            return False
        self.lines.append(line)
        self.used += size
        return True


def retrieve(store: MemoryStore, query: RecallQuery) -> MemorySection:
    names = NameBook.load(store)
    speaker = None if query.speaker is None else names.person(query.speaker)
    lines: list[SectionLine] = _with(store, query, speaker)
    lines += _stable(store, names)
    lines += _chapters(store)
    shown = _episodes(store, query)
    lines += shown.lines
    said = [event_key(line.citation.ranges[0].first) for line in lines if line.level == "with"]
    lines += _recent(store, query, shown.covered + [(key, key) for key in said])
    return MemorySection(lines=tuple(lines), speaker=speaker if said else None)


def _with(store: MemoryStore, query: RecallQuery, speaker: str | None) -> list[SectionLine]:
    """The first `WITH_FIRST` and then the newest exchanges with the speaker, shown oldest first. Two
    bounded reads of `l0`/`l0_who` (S28-a ports them as one `SeatView` method with the same order and
    limits)."""
    if query.speaker is None:
        return []
    section = _Section("with", speaker)
    kinds = ",".join("?" * len(EXCHANGE_KINDS))
    sql = (
        "SELECT l0.event_key, l0.at, l0.gist FROM l0 JOIN l0_who w ON w.event_key = l0.event_key "
        f"WHERE w.entity = ? AND l0.involved = 1 AND l0.kind IN ({kinds}) AND l0.at <= ? "
        "AND l0.event_key != ? AND NOT EXISTS (SELECT 1 FROM l0_who o "
        "WHERE o.event_key = l0.event_key AND o.entity != ?) ORDER BY l0.event_key "
    )
    heard = "" if query.heard is None else event_key(query.heard)
    arguments = (query.speaker, *EXCHANGE_KINDS, query.at, heard, query.speaker)
    rows: list[tuple[str, int, str]] = store.connection.execute(
        sql + "ASC LIMIT ?", (*arguments, WITH_FIRST)
    ).fetchall()
    rows += store.connection.execute(
        sql + "DESC LIMIT ?", (*arguments, section.budget.lines - WITH_FIRST)
    ).fetchall()
    for key, at, gist in sorted(set(rows)):
        if not section.offer(
            f"Day {day_number(at)} {clock(at)}: {gist}", Citation.of(event_of(key))
        ):
            break
    return section.lines


def _stable(store: MemoryStore, names: NameBook) -> list[SectionLine]:
    section = _Section("L3")
    for fact in stable_facts(store, BUDGETS["L3"].lines):
        if not section.offer(stable_text(fact, names), fact.citation()):
            break
    return section.lines


def _chapters(store: MemoryStore) -> list[SectionLine]:
    section = _Section("L2")
    for chapter in chapters(store, BUDGETS["L2"].lines):
        if not section.offer(chapter.prose or chapter.text, chapter.citation()):
            break
    return section.lines


@dataclass(frozen=True)
class _Shown:
    lines: list[SectionLine]
    covered: list[tuple[EventKey, EventKey]]


def _episodes(store: MemoryStore, query: RecallQuery) -> _Shown:
    section = _Section("L1")
    seen: set[int] = set()
    covered: list[tuple[EventKey, EventKey]] = []

    def offer(episodes: Iterable[Episode]) -> None:
        for episode in episodes:
            if episode.number in seen:
                continue
            if not section.offer(episode.prose or episode.text, episode.citation()):
                return
            seen.add(episode.number)
            covered.append((event_key(episode.first), event_key(episode.last)))

    if query.counterparts:
        marks = ",".join("?" * len(query.counterparts))
        offer(
            load_episodes(
                store,
                f"closed = 1 AND episode IN (SELECT episode FROM episode_who WHERE entity IN "
                f"({marks})) ORDER BY episode DESC",
                tuple(query.counterparts),
            )
        )
    terms = fts.query_terms(query.words)
    if terms and not section.full:
        offer(_matching(store, terms))
    return _Shown(section.lines, covered)


def _matching(store: MemoryStore, terms: Sequence[str]) -> list[Episode]:
    """Closed episodes holding a notable record that matches a term, or whose own text or prose holds
    it, most distinct terms first, then newest, then by first Event ID. The second set reaches an episode
    whose records retention merged into it (pr-fg-b §5.10)."""
    matched: dict[int, int] = {}
    for term in terms:
        found = set(_episodes_matching(store, term))
        found |= set(_episode_text_matching(store, term))
        for number in found:
            matched[number] = matched.get(number, 0) + 1
    if not matched:
        return []
    marks = ",".join("?" * len(matched))
    found = load_episodes(store, f"episode IN ({marks})", tuple(matched))
    return sorted(found, key=lambda e: (-matched[e.number], -e.closed_at, event_key(e.first)))


def _episodes_matching(store: MemoryStore, term: str) -> list[int]:
    if store.fts:
        sql = (
            "SELECT DISTINCT e.episode FROM l0_text JOIN l0 ON l0.rowid = l0_text.rowid "
            "JOIN episodes e ON l0.event_key BETWEEN e.first_key AND e.last_key "
            "WHERE l0_text MATCH ? AND e.closed = 1"
        )
        argument = fts.fts_string(term)
    else:
        sql = (
            "SELECT DISTINCT e.episode FROM l0 JOIN episodes e "
            "ON l0.event_key BETWEEN e.first_key AND e.last_key "
            "WHERE l0.salience = 'notable' AND l0.gist LIKE ? ESCAPE '\\' AND e.closed = 1"
        )
        argument = fts.like_pattern(term)
    rows: list[tuple[int]] = store.connection.execute(sql, (argument,)).fetchall()
    return [number for (number,) in rows]


def _episode_text_matching(store: MemoryStore, term: str) -> list[int]:
    """Closed episodes whose summary text or prose holds `term` literally (`LIKE … ESCAPE`, P4-4). Over at
    most the store's episodes, which the retention bound limits."""
    pattern = fts.like_pattern(term)
    rows: list[tuple[int]] = store.connection.execute(
        "SELECT episode FROM episodes WHERE closed = 1 AND "
        "(text LIKE ? ESCAPE '\\' OR prose LIKE ? ESCAPE '\\')",
        (pattern, pattern),
    ).fetchall()
    return [number for (number,) in rows]


def _recent(
    store: MemoryStore, query: RecallQuery, covered: list[tuple[EventKey, EventKey]]
) -> list[SectionLine]:
    section = _Section("L0")
    rows: list[tuple[str, int, str]] = store.connection.execute(
        "SELECT event_key, at, gist FROM l0 WHERE salience = 'notable' AND at > ? AND at <= ? "
        "ORDER BY event_key DESC",
        (query.at - DAY_S, query.at),
    ).fetchall()
    for key, at, gist in rows:
        if any(first <= key <= last for first, last in covered):
            continue
        event = event_of(key)
        if not section.offer(f"Day {day_number(at)} {clock(at)}: {gist}", Citation.of(event)):
            break
    return section.lines
