"""Schema 4: the memory home's tables, the seat view's tables, and the codecs of the canonical dump
(pr-s28a-memory-home.md §§5.2, 5.3, 5.4; `ARC-101`, `ARC-109`).

The home holds every seat one cognition process hosts in one world instance. The `world` scope's tables (`fact`,
`fact_who`) hold each perceived fact's perspective-free envelope once, keyed by its `EventKey`; `access` holds
one row per (seat, held fact) with that seat's rendering; every private table of P4 and FG-b gains a leading
`seat` column and is otherwise schema 3's. `seats.seat` is an internal integer: it is never dumped, so the order
in which seats were opened cannot reach a dump (I-S28-5).

A **seat view** is the projection of the home onto one seat, in schema 3's tables and order: `meta` (the home's
identity, the seat's identity and its own rows), `names`, `l0` (`access ⋈ fact`), `l0_who` (the held facts'
participants other than the seat, envelope order), then the private tables. `VIEW_TABLES` fixes each view
table's columns and order; `CODECS` fixes how each column is written in the canonical line (`encoding.line`).
"""

from __future__ import annotations

from dataclasses import dataclass, field

from mineworld_cognition.memory import fts
from mineworld_cognition.memory.home.encoding import Codec, as_number, raw, short_digest

SCHEMA_VERSION = 4
"""2 since FG-b, 3 since FG-b2 (P4's per-seat stores); 4 since S28-a: one home of seat views over shared fact
records, dumped positionally (`ARC-101`, `ARC-109`). A store of another version is refused by name; memory is
rebuilt from the perceived stream, never migrated (QS28-8)."""

HOME_DDL: tuple[str, ...] = (
    "CREATE TABLE home_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    "CREATE TABLE seats (seat INTEGER PRIMARY KEY, key TEXT NOT NULL UNIQUE, self_entity TEXT NOT NULL)",
    "CREATE TABLE fact (event_key TEXT PRIMARY KEY, event_id TEXT NOT NULL, at INTEGER NOT NULL, "
    "kind TEXT NOT NULL, place TEXT, digest TEXT NOT NULL)",
    "CREATE TABLE fact_who (event_key TEXT NOT NULL, ordinal INTEGER NOT NULL, entity TEXT NOT NULL, "
    "role INTEGER NOT NULL CHECK (role IN (1, 2, 3)), PRIMARY KEY (event_key, ordinal))",
    "CREATE TABLE access (seat INTEGER NOT NULL, event_key TEXT NOT NULL, gist TEXT NOT NULL, "
    "salience TEXT NOT NULL CHECK (salience IN ('notable', 'ambient')), "
    "mine INTEGER NOT NULL CHECK (mine IN (0, 1)), importance INTEGER NOT NULL, "
    "PRIMARY KEY (seat, event_key))",
    "CREATE INDEX access_by_key ON access (event_key)",
    "CREATE TABLE meta (seat INTEGER NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, "
    "PRIMARY KEY (seat, key))",
    # `importance` and `mine` are written when the episode closes (0 while open), so they survive the
    # removal of its records (pr-fg-b §5.4).
    "CREATE TABLE episodes (seat INTEGER NOT NULL, episode INTEGER NOT NULL, day INTEGER NOT NULL, "
    "place TEXT, first_key TEXT NOT NULL, last_key TEXT NOT NULL, opened_at INTEGER NOT NULL, "
    "closed_at INTEGER NOT NULL, closed INTEGER NOT NULL CHECK (closed IN (0, 1)), "
    "records INTEGER NOT NULL, text TEXT NOT NULL, prose TEXT, prose_key TEXT, "
    "importance INTEGER NOT NULL, mine INTEGER NOT NULL CHECK (mine IN (0, 1)), "
    "PRIMARY KEY (seat, episode))",
    "CREATE INDEX episodes_by_last_key ON episodes (seat, last_key)",
    "CREATE TABLE episode_who (seat INTEGER NOT NULL, episode INTEGER NOT NULL, entity TEXT NOT NULL, "
    "PRIMARY KEY (seat, episode, entity))",
    "CREATE TABLE chapters (seat INTEGER NOT NULL, week INTEGER NOT NULL, "
    "first_episode INTEGER NOT NULL, last_episode INTEGER NOT NULL, text TEXT NOT NULL, prose TEXT, "
    "prose_key TEXT, first_key TEXT NOT NULL, last_key TEXT NOT NULL, first_at INTEGER NOT NULL, "
    "last_at INTEGER NOT NULL, importance INTEGER NOT NULL, "
    "mine INTEGER NOT NULL CHECK (mine IN (0, 1)), PRIMARY KEY (seat, week))",
    "CREATE TABLE chapter_ranges (seat INTEGER NOT NULL, week INTEGER NOT NULL, "
    "first_key TEXT NOT NULL, last_key TEXT NOT NULL, PRIMARY KEY (seat, week, first_key))",
    "CREATE TABLE stable (seat INTEGER NOT NULL, entity TEXT NOT NULL, first_key TEXT NOT NULL, "
    "last_key TEXT NOT NULL, times_met INTEGER NOT NULL, exchanges INTEGER NOT NULL, level TEXT, "
    "level_key TEXT, first_at INTEGER NOT NULL, last_at INTEGER NOT NULL, PRIMARY KEY (seat, entity))",
    "CREATE TABLE names (seat INTEGER NOT NULL, entity TEXT NOT NULL, name TEXT NOT NULL, "
    "source_key TEXT, PRIMARY KEY (seat, entity))",
    "CREATE TABLE forgotten (seat INTEGER NOT NULL, first_key TEXT NOT NULL, last_key TEXT NOT NULL, "
    "PRIMARY KEY (seat, first_key))",
    "CREATE TABLE retention_passes (seat INTEGER NOT NULL, trigger TEXT NOT NULL, day INTEGER NOT NULL, "
    "cause TEXT NOT NULL CHECK (cause IN ('day', 'size')), policy TEXT NOT NULL, "
    "outcome TEXT NOT NULL, kept INTEGER NOT NULL, merged INTEGER NOT NULL, dropped INTEGER NOT NULL, "
    "forced INTEGER NOT NULL, bytes_before INTEGER NOT NULL, bytes_after INTEGER NOT NULL, "
    "forgotten_ranges INTEGER NOT NULL, model_decisions INTEGER NOT NULL, "
    "fallback_decisions INTEGER NOT NULL, PRIMARY KEY (seat, trigger))",
)
FTS_DDL = (
    f"CREATE VIRTUAL TABLE access_text USING fts5(gist, seat UNINDEXED, tokenize='{fts.TOKENIZER}')"
)
"""Notable gists, `rowid` equal to their `access` row's: an index, rebuilt from `access`, never dumped."""

SEAT_META = (("cursor", ""), ("newest", ""), ("retention_day", ""), ("forgotten_below", ""))
"""A seat's own state: the perceived cursor and newest key (P4-2), the day of the last retention pass and the
forgotten set's watermark (pr-fg-b §5.5, §5.8); `""` until set."""
IDENTITY_KEYS = ("world_instance", "schema", "seat", "self_entity")
"""The `meta` rows of a seat view that the home supplies: its own identity and the seat's."""

KEY_COLUMNS = frozenset(
    {"event_key", "first_key", "last_key", "level_key", "source_key", "trigger"}
)
"""Columns holding an `EventKey` (a padded `EventId`): written as the unpadded decimal number."""
ID_COLUMNS = frozenset({"event_id"})
DIGEST_COLUMNS = frozenset({"digest"})
NUMBER_META = frozenset({"cursor", "newest", "forgotten_below"})
"""`meta` values holding an `EventId` or `EventKey`: written as numbers too (`null` while unset)."""


def codec(column: str) -> Codec:
    """How one column's value is written in a canonical line (§5.4)."""
    if column in KEY_COLUMNS or column in ID_COLUMNS:
        return as_number
    if column in DIGEST_COLUMNS:
        return short_digest
    return raw


def meta_codec(key: str) -> Codec:
    return as_number if key in NUMBER_META else raw


@dataclass(frozen=True)
class ViewTable:
    """One table of a seat view: its dumped columns (schema 3's names, without `seat`), in order, the
    `ORDER BY` of its rows, and each column's codec (derived once: the meter encodes a line per write)."""

    name: str
    columns: tuple[str, ...]
    order: str
    codecs: tuple[Codec, ...] = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        object.__setattr__(self, "codecs", tuple(codec(column) for column in self.columns))


META = ViewTable("meta", ("key", "value"), "key")
NAMES = ViewTable("names", ("entity", "name", "source_key"), "entity")
L0 = ViewTable(
    "l0",
    ("event_id", "at", "kind", "place", "salience", "mine", "gist", "digest", "importance"),
    "event_key",
)
"""`access ⋈ fact` for the seat; `event_key` orders the rows and is not dumped (the id is)."""
L0_WHO = ViewTable("l0_who", ("event_id", "entity"), "event_key, ordinal")
"""The held facts' `fact_who` rows of role 1 or 3 whose entity is not the seat's, in ordinal order."""
PRIVATE: tuple[ViewTable, ...] = (
    ViewTable(
        "episodes",
        (
            "episode",
            "day",
            "place",
            "first_key",
            "last_key",
            "opened_at",
            "closed_at",
            "closed",
            "records",
            "text",
            "prose",
            "prose_key",
            "importance",
            "mine",
        ),
        "episode",
    ),
    ViewTable("episode_who", ("episode", "entity"), "episode, entity"),
    ViewTable(
        "chapters",
        (
            "week",
            "first_episode",
            "last_episode",
            "text",
            "prose",
            "prose_key",
            "first_key",
            "last_key",
            "first_at",
            "last_at",
            "importance",
            "mine",
        ),
        "week",
    ),
    ViewTable("chapter_ranges", ("week", "first_key", "last_key"), "week, first_key"),
    ViewTable(
        "stable",
        (
            "entity",
            "first_key",
            "last_key",
            "times_met",
            "exchanges",
            "level",
            "level_key",
            "first_at",
            "last_at",
        ),
        "entity",
    ),
    ViewTable("forgotten", ("first_key", "last_key"), "first_key"),
    ViewTable(
        "retention_passes",
        (
            "trigger",
            "day",
            "cause",
            "policy",
            "outcome",
            "kept",
            "merged",
            "dropped",
            "forced",
            "bytes_before",
            "bytes_after",
            "forgotten_ranges",
            "model_decisions",
            "fallback_decisions",
        ),
        "trigger",
    ),
)
"""The private tables after `l0_who`, in dump order. `meta` and `names` are private too, and come first."""
SEAT_TABLES: tuple[ViewTable, ...] = (META, NAMES, *PRIVATE)
"""Every table with a `seat` column whose rows a seat view shows as they are (its `meta` rows besides the
identity rows): what the home's meter keeps by trigger."""
VIEW_TABLES: tuple[ViewTable, ...] = (META, NAMES, L0, L0_WHO, *PRIVATE)
"""A seat view's tables, in dump order (P4's `_DUMPED` order)."""
BY_NAME: dict[str, ViewTable] = {table.name: table for table in VIEW_TABLES}

FACT_COLUMNS = ("event_id", "at", "kind", "place", "digest")
"""A `fact` line of the home dump: the id once (the key's number is the id's), then the envelope fields."""
ACCESS_COLUMNS = ("event_id", "gist", "salience", "mine", "importance")
"""An `access` line of the home dump, inside its seat's section: the seat is the section's header."""
