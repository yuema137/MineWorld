"""L2: one chapter per simulated week (`day // 7`) of closed episodes (§5.5).

A week's chapter is written once the week is fully past: when a closed episode exists in a later week.
Episodes are consecutive, so every episode of the earlier week is closed by then, and the chapter is a
function of the facts alone.
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.compress.episodes import load_episodes
from mineworld_cognition.compress.summarize import Summarizer
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Chapter, event_key, event_of
from mineworld_cognition.memory.store import MemoryStore


def close_weeks(store: MemoryStore, names: NameBook, summarizer: Summarizer) -> int:
    """Writes the chapter of every fully past week that has none; returns how many it wrote."""
    db = store.connection
    newest: int | None = db.execute("SELECT max(day) FROM episodes WHERE closed = 1").fetchone()[0]
    if newest is None:
        return 0
    weeks: list[tuple[int]] = db.execute(
        "SELECT DISTINCT day / 7 FROM episodes WHERE closed = 1 AND day / 7 < ? "
        "AND day / 7 NOT IN (SELECT week FROM chapters) ORDER BY 1",
        (newest // 7,),
    ).fetchall()
    for (week,) in weeks:
        episodes = load_episodes(store, "closed = 1 AND day / 7 = ? ORDER BY episode", (week,))
        first, last = event_key(episodes[0].first), event_key(episodes[-1].last)
        newcomers: list[tuple[str]] = db.execute(
            "SELECT entity FROM stable WHERE first_key BETWEEN ? AND ? ORDER BY first_key, entity",
            (first, last),
        ).fetchall()
        text = summarizer.chapter(
            episodes, names, newcomers=[EntityId(entity) for (entity,) in newcomers]
        )
        db.execute(
            "INSERT INTO chapters (week, first_episode, last_episode, text, prose, prose_key) "
            "VALUES (?, ?, ?, ?, NULL, NULL)",
            (week, episodes[0].number, episodes[-1].number, text),
        )
    return len(weeks)


def chapters(store: MemoryStore) -> list[Chapter]:
    """Every chapter, newest week first."""
    rows: list[tuple[int, int, int, str, str | None, str, str]] = store.connection.execute(
        "SELECT c.week, c.first_episode, c.last_episode, c.text, c.prose, f.first_key, l.last_key "
        "FROM chapters c JOIN episodes f ON f.episode = c.first_episode "
        "JOIN episodes l ON l.episode = c.last_episode ORDER BY c.week DESC"
    ).fetchall()
    return [
        Chapter(
            week=week,
            first_episode=first_episode,
            last_episode=last_episode,
            first=event_of(first),
            last=event_of(last),
            text=text,
            prose=prose,
        )
        for week, first_episode, last_episode, text, prose, first, last in rows
    ]
