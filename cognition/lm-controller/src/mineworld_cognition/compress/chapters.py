"""L2: one chapter per simulated week (`day // 7`) of closed episodes (§5.5).

A week's chapter is written once the week is fully past: when a closed episode exists in a later week.
Episodes are consecutive, so every episode of the earlier week is closed by then, and the chapter is a
function of the facts alone.

Its citation is stored with it (pr-fg-b §5.5): one range per run of consecutive episode numbers among the
week's episodes when it is written. Consecutive episodes are adjacent in the perceived sequence (D-P4-7),
so a gap in the numbers is an episode retention forgot before the chapter existed, and the chapter must
not cite it. Its first and last keys and times, its importance (the largest of its episodes') and whether
any episode was mine are stored too, so retention can merge its episodes into it and nothing about it
changes. Every statement names the seat (pr-s28a §5.8).
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.compress.episodes import load_episodes
from mineworld_cognition.compress.summarize import Summarizer
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Chapter, Episode, EventRange, event_key, event_of


def close_weeks(store: SeatView, names: NameBook, summarizer: Summarizer) -> int:
    """Writes the chapter of every fully past week that has none; returns how many it wrote."""
    db = store.connection
    seat = store.seat_id
    newest: int | None = db.execute(
        "SELECT max(day) FROM episodes WHERE seat = ? AND closed = 1", (seat,)
    ).fetchone()[0]
    if newest is None:
        return 0
    weeks: list[tuple[int]] = db.execute(
        "SELECT DISTINCT day / 7 FROM episodes WHERE seat = ? AND closed = 1 AND day / 7 < ? "
        "AND day / 7 NOT IN (SELECT week FROM chapters WHERE seat = ?) ORDER BY 1",
        (seat, newest // 7, seat),
    ).fetchall()
    for (week,) in weeks:
        episodes = load_episodes(store, "closed = 1 AND day / 7 = ? ORDER BY episode", (week,))
        first, last = event_key(episodes[0].first), event_key(episodes[-1].last)
        newcomers: list[tuple[str]] = db.execute(
            "SELECT entity FROM stable WHERE seat = ? AND first_key BETWEEN ? AND ? "
            "ORDER BY first_key, entity",
            (seat, first, last),
        ).fetchall()
        text = summarizer.chapter(
            episodes, names, newcomers=[EntityId(entity) for (entity,) in newcomers]
        )
        numbers = [episode.number for episode in episodes]
        importance, mine = db.execute(
            "SELECT max(importance), max(mine) FROM episodes "
            "WHERE seat = ? AND closed = 1 AND day / 7 = ?",
            (seat, week),
        ).fetchone()
        db.execute(
            "INSERT INTO chapters (seat, week, first_episode, last_episode, text, prose, prose_key, "
            "first_key, last_key, first_at, last_at, importance, mine) "
            "VALUES (?, ?, ?, ?, ?, NULL, NULL, ?, ?, ?, ?, ?, ?)",
            (
                seat,
                week,
                numbers[0],
                numbers[-1],
                text,
                first,
                last,
                episodes[0].opened_at,
                episodes[-1].closed_at,
                importance,
                mine,
            ),
        )
        db.executemany(
            "INSERT INTO chapter_ranges (seat, week, first_key, last_key) VALUES (?, ?, ?, ?)",
            [
                (seat, week, event_key(run.first), event_key(run.last))
                for run in consecutive_runs(episodes)
            ],
        )
    return len(weeks)


def consecutive_runs(episodes: list[Episode]) -> list[EventRange]:
    """One range per run of consecutive episode numbers, in order."""
    runs: list[EventRange] = []
    start = previous = episodes[0]
    for episode in episodes[1:]:
        if episode.number != previous.number + 1:
            runs.append(EventRange(first=start.first, last=previous.last))
            start = episode
        previous = episode
    runs.append(EventRange(first=start.first, last=previous.last))
    return runs


def chapters(store: SeatView, limit: int) -> list[Chapter]:
    """The `limit` newest chapters, newest week first, with their stored ranges: no `episodes` row is
    read (F-FGb-3), and the read is bounded (F-S24-2)."""
    db = store.connection
    rows: list[tuple[int, int, int, str, str | None]] = db.execute(
        "SELECT week, first_episode, last_episode, text, prose FROM chapters WHERE seat = ? "
        "ORDER BY week DESC LIMIT ?",
        (store.seat_id, limit),
    ).fetchall()
    found: list[Chapter] = []
    for week, first_episode, last_episode, text, prose in rows:
        ranges: list[tuple[str, str]] = db.execute(
            "SELECT first_key, last_key FROM chapter_ranges WHERE seat = ? AND week = ? "
            "ORDER BY first_key",
            (store.seat_id, week),
        ).fetchall()
        found.append(
            Chapter(
                week=week,
                first_episode=first_episode,
                last_episode=last_episode,
                ranges=tuple(
                    EventRange(first=event_of(first), last=event_of(last)) for first, last in ranges
                ),
                text=text,
                prose=prose,
            )
        )
    return found
