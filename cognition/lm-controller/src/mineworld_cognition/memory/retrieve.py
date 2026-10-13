"""Retrieval: the memory section a decision's context holds, bounded by construction (D-P4-9, D-P4-11).

Sections, in order, each filled in its order of preference until its line count or byte budget is
reached; a line that does not fit is not emitted, even in part:

- **L3** stable facts: the counterparts met most often (ties: the earliest first meeting, then the
  entity id string);
- **L2** chapters, newest first;
- **L1** episodes: first those with a counterpart the query names, newest first; then, for the trigger's
  words, those holding a notable record FTS5 (or `LIKE`) matches, ranked by the number of distinct terms
  matched, then by recency, then by the first Event ID;
- **L0** notable records of the last 86 400 s before the query's time, newest first, except those an
  episode shown above already covers.

Every line is cut to 160 bytes at a character boundary, its citation token included, so the rendered
section never exceeds 6 000 bytes. Ordering is integers and strings only: no clock, no randomness, no
float (P4-3); `bm25()` is never read.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from dataclasses import dataclass

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.compress.chapters import chapters
from mineworld_cognition.compress.episodes import load_episodes
from mineworld_cognition.compress.stable import stable_facts
from mineworld_cognition.compress.summarize import DAY_S, clock, counted, day_number
from mineworld_cognition.memory import fts
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import (
    Citation,
    Episode,
    EventKey,
    EventRange,
    Level,
    StableFact,
    event_key,
    event_of,
)
from mineworld_cognition.memory.store import MemoryStore

LINE_BYTES = 160
SECTION_BYTES = 6000
ELLIPSIS = "…"


@dataclass(frozen=True)
class Budget:
    header: str
    lines: int
    bytes: int


BUDGETS: dict[Level, Budget] = {
    "L3": Budget("People I know:", 16, 1600),
    "L2": Budget("Recent weeks:", 4, 800),
    "L1": Budget("Episodes I remember:", 8, 1600),
    "L0": Budget("The last day:", 48, 2000),
}
"""Lines and bytes per section; a section's bytes include its header and every newline (QP4-7)."""


class RecallQuery(CognitionModel):
    at: int
    """The world time of the decision; L0 shows the 86 400 s before it."""
    counterparts: tuple[EntityId, ...] = ()
    """People present now or named in the trigger."""
    words: str = ""
    """The trigger's words, matched as literal terms (P4-4)."""


class SectionLine(CognitionModel):
    level: Level
    text: str
    citation: Citation

    def render(self) -> str:
        return f"{self.text} {self.citation.token()}"


class MemorySection(CognitionModel):
    lines: tuple[SectionLine, ...]

    def render(self) -> str:
        """One header per non-empty section, one line per entry; at most 6 000 bytes."""
        out: list[str] = []
        for level, budget in BUDGETS.items():
            mine = [line.render() for line in self.lines if line.level == level]
            if mine:
                out.append(budget.header + "\n" + "".join(text + "\n" for text in mine))
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
    def __init__(self, level: Level) -> None:
        self.level: Level = level
        self.budget = BUDGETS[level]
        self.used = len(self.budget.header.encode("utf-8")) + 1
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
    lines: list[SectionLine] = []
    lines += _stable(store, names)
    lines += _chapters(store)
    shown = _episodes(store, query)
    lines += shown.lines
    lines += _recent(store, query, shown.covered)
    return MemorySection(lines=tuple(lines))


def _stable(store: MemoryStore, names: NameBook) -> list[SectionLine]:
    section = _Section("L3")
    for fact in stable_facts(store):
        if not section.offer(_stable_text(store, fact, names), fact.citation()):
            break
    return section.lines


def _stable_text(store: MemoryStore, fact: StableFact, names: NameBook) -> str:
    first, last = _at(store, fact.first), _at(store, fact.last)
    text = f"{names.person(fact.entity)}: met on day {day_number(first)}, "
    text += f"last seen day {day_number(last)}, met {counted(fact.times_met, 'time')}, "
    text += counted(fact.exchanges, "exchange")
    return text + (f", {fact.level}." if fact.level else ".")


def _at(store: MemoryStore, event: EventId) -> int:
    row: tuple[int] = store.connection.execute(
        "SELECT at FROM l0 WHERE event_key = ?", (event_key(event),)
    ).fetchone()
    return row[0]


def _chapters(store: MemoryStore) -> list[SectionLine]:
    section = _Section("L2")
    for chapter in chapters(store):
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
    """Closed episodes holding a notable record that matches a term, most distinct terms first, then
    newest, then by first Event ID."""
    matched: dict[int, int] = {}
    for term in terms:
        for number in _episodes_matching(store, term):
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
