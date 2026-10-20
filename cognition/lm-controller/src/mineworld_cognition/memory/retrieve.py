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

from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from typing import Literal

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.compress.chapters import chapters
from mineworld_cognition.compress.episodes import load_episodes
from mineworld_cognition.compress.stable import stable_facts, stable_text
from mineworld_cognition.compress.summarize import DAY_S, clock, day_number
from mineworld_cognition.memory import fts
from mineworld_cognition.memory.home import SeatView
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
from mineworld_cognition.memory.scopes.interface import RecallQuery as RecallQuery
from mineworld_cognition.memory.scopes.interface import SectionKey
from mineworld_cognition.memory.scopes.interface import SectionLine as SectionLine

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
1 000 bytes were L0's (2 000 before pr-p6c-fgd), so the sum, and the ceiling, stay 6 000 (`ARC-59`). The
person scope's defaults (`memory.scopes.person`), which a world may only shrink."""
LEVELS: tuple[SectionLevel, ...] = tuple(BUDGETS)
"""The person scope's sections in render order; equal to `memory.scopes.interface.PERSON_SECTIONS`."""
WITH_FIRST = 2
"""How many of the first exchanges with the speaker the `with` section shows before the newest."""

type Budgets = Mapping[SectionLevel, Budget]


def header(level: SectionLevel, speaker: str | None) -> str:
    """A section's header; `with`'s names the speaker by label, never by id (I-10)."""
    return BUDGETS[level].header.replace("{name}", speaker or "someone")


class SectionHeading(CognitionModel):
    """A non-person section of the stack (pr-s28c §5.6): its key and its one-line header."""

    key: SectionKey
    header: str


class MemorySection(CognitionModel):
    lines: tuple[SectionLine, ...]
    speaker: str | None = None
    """The label of the person the `with` section is about, when it has lines."""
    sections: tuple[SectionHeading, ...] = ()
    """The stack's other sections, in render order, after the person scope's five (F-S28c-2). Empty under the
    default stack, so a logged section parses and renders as before."""

    def render(self) -> str:
        """One header per non-empty section, one line per entry: the person scope's five in `BUDGETS`'
        order, then each of `sections`; at most 6 000 bytes."""
        out: list[str] = []
        for level in BUDGETS:
            self._render(out, level, header(level, self.speaker))
        for heading in self.sections:
            self._render(out, heading.key, heading.header)
        return "".join(out)

    def _render(self, out: list[str], key: str, heading: str) -> None:
        mine = [line.render() for line in self.lines if line.level == key]
        if mine:
            out.append(heading + "\n" + "".join(t + "\n" for t in mine))

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
    def __init__(self, level: SectionLevel, budgets: Budgets, speaker: str | None = None) -> None:
        self.level: SectionLevel = level
        self.budget = budgets[level]
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


def retrieve(
    store: SeatView, query: RecallQuery, budgets: Budgets = BUDGETS, window_s: int = DAY_S
) -> MemorySection:
    """The person scope's five sections. `budgets` and `window_s` are its parameters (the defaults are
    today's, so every statement runs as before); headers are always `BUDGETS`'s."""
    names = NameBook.load(store)
    speaker = None if query.speaker is None else names.person(query.speaker)
    lines: list[SectionLine] = _with(store, query, speaker, budgets)
    lines += _stable(store, names, budgets)
    lines += _chapters(store, budgets)
    shown = _episodes(store, query, budgets)
    lines += shown.lines
    said = [event_key(line.citation.ranges[0].first) for line in lines if line.level == "with"]
    lines += _recent(store, query, shown.covered + [(key, key) for key in said], budgets, window_s)
    return MemorySection(lines=tuple(lines), speaker=speaker if said else None)


def _with(
    store: SeatView, query: RecallQuery, speaker: str | None, budgets: Budgets
) -> list[SectionLine]:
    """The first `WITH_FIRST` and then the newest exchanges with the speaker, shown oldest first. Two
    bounded reads of the seat's held records (`access ⋈ fact`) whose participants (`fact_who` roles 1
    and 3) are exactly the seat and the speaker: schema 3's `l0`/`l0_who` read, ported to schema 4 with
    the same order and limits (pr-p6c-fgd §9, D-P6c-3); `involved` is derived, as S28-a does (F-S28a-7)."""
    if query.speaker is None:
        return []
    section = _Section("with", budgets, speaker)
    kinds = ",".join("?" * len(EXCHANGE_KINDS))
    sql = (
        "SELECT a.event_key, f.at, a.gist FROM access a JOIN fact f ON f.event_key = a.event_key "
        "JOIN fact_who w ON w.event_key = a.event_key AND w.role IN (1, 3) AND w.entity = ? "
        f"WHERE a.seat = ? AND f.kind IN ({kinds}) AND f.at <= ? AND a.event_key != ? "
        "AND EXISTS (SELECT 1 FROM fact_who m WHERE m.event_key = a.event_key "
        "AND m.role IN (1, 3) AND m.entity = ?) "
        "AND NOT EXISTS (SELECT 1 FROM fact_who o WHERE o.event_key = a.event_key "
        "AND o.role IN (1, 3) AND o.entity != ? AND o.entity != ?) ORDER BY a.event_key "
    )
    heard = "" if query.heard is None else event_key(query.heard)
    me = store.me
    arguments = (
        query.speaker,
        store.seat_id,
        *EXCHANGE_KINDS,
        query.at,
        heard,
        me,
        query.speaker,
        me,
    )
    rows: list[tuple[str, int, str]] = store.connection.execute(
        sql + "ASC LIMIT ?", (*arguments, WITH_FIRST)
    ).fetchall()
    rows += store.connection.execute(
        sql + "DESC LIMIT ?", (*arguments, max(0, section.budget.lines - WITH_FIRST))
    ).fetchall()
    for key, at, gist in sorted(set(rows)):
        if not section.offer(
            f"Day {day_number(at)} {clock(at)}: {gist}", Citation.of(event_of(key))
        ):
            break
    return section.lines


def _stable(store: SeatView, names: NameBook, budgets: Budgets) -> list[SectionLine]:
    section = _Section("L3", budgets)
    for fact in stable_facts(store, section.budget.lines):
        if not section.offer(stable_text(fact, names), fact.citation()):
            break
    return section.lines


def _chapters(store: SeatView, budgets: Budgets) -> list[SectionLine]:
    section = _Section("L2", budgets)
    for chapter in chapters(store, section.budget.lines):
        if not section.offer(chapter.prose or chapter.text, chapter.citation()):
            break
    return section.lines


@dataclass(frozen=True)
class _Shown:
    lines: list[SectionLine]
    covered: list[tuple[EventKey, EventKey]]


def _episodes(store: SeatView, query: RecallQuery, budgets: Budgets) -> _Shown:
    section = _Section("L1", budgets)
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
                f"closed = 1 AND episode IN (SELECT episode FROM episode_who WHERE seat = ? AND "
                f"entity IN ({marks})) ORDER BY episode DESC",
                (store.seat_id, *query.counterparts),
            )
        )
    terms = fts.query_terms(query.words)
    if terms and not section.full:
        offer(_matching(store, terms))
    return _Shown(section.lines, covered)


def _matching(store: SeatView, terms: Sequence[str]) -> list[Episode]:
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


def _episodes_matching(store: SeatView, term: str) -> list[int]:
    """This seat's closed episodes holding a notable record of its own whose gist matches `term`."""
    if store.fts:
        sql = (
            "SELECT DISTINCT e.episode FROM access_text JOIN access a ON a.rowid = access_text.rowid "
            "JOIN episodes e ON e.seat = a.seat AND a.event_key BETWEEN e.first_key AND e.last_key "
            "WHERE access_text MATCH ? AND a.seat = ? AND e.closed = 1"
        )
        argument = fts.fts_string(term)
    else:
        sql = (
            "SELECT DISTINCT e.episode FROM access a JOIN episodes e "
            "ON e.seat = a.seat AND a.event_key BETWEEN e.first_key AND e.last_key "
            "WHERE a.salience = 'notable' AND a.gist LIKE ? ESCAPE '\\' AND a.seat = ? "
            "AND e.closed = 1"
        )
        argument = fts.like_pattern(term)
    rows: list[tuple[int]] = store.connection.execute(sql, (argument, store.seat_id)).fetchall()
    return [number for (number,) in rows]


def _episode_text_matching(store: SeatView, term: str) -> list[int]:
    """Closed episodes whose summary text or prose holds `term` literally (`LIKE … ESCAPE`, P4-4). Over at
    most the seat's episodes, which the retention bound limits."""
    pattern = fts.like_pattern(term)
    rows: list[tuple[int]] = store.connection.execute(
        "SELECT episode FROM episodes WHERE seat = ? AND closed = 1 AND "
        "(text LIKE ? ESCAPE '\\' OR prose LIKE ? ESCAPE '\\')",
        (store.seat_id, pattern, pattern),
    ).fetchall()
    return [number for (number,) in rows]


def _recent(
    store: SeatView,
    query: RecallQuery,
    covered: list[tuple[EventKey, EventKey]],
    budgets: Budgets,
    window_s: int,
) -> list[SectionLine]:
    section = _Section("L0", budgets)
    rows: list[tuple[str, int, str]] = store.connection.execute(
        "SELECT a.event_key, f.at, a.gist FROM access a JOIN fact f ON f.event_key = a.event_key "
        "WHERE a.seat = ? AND a.salience = 'notable' AND f.at > ? AND f.at <= ? "
        "ORDER BY a.event_key DESC",
        (store.seat_id, query.at - window_s, query.at),
    ).fetchall()
    for key, at, gist in rows:
        if any(first <= key <= last for first, last in covered):
            continue
        event = event_of(key)
        if not section.offer(f"Day {day_number(at)} {clock(at)}: {gist}", Citation.of(event)):
            break
    return section.lines
