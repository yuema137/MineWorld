"""Schema 5: the memory home's tables, the seat view's tables, and the codecs of the canonical dump
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

SCHEMA_VERSION = 5
"""2 since FG-b, 3 since FG-b2 (P4's per-seat stores); 4 since S28-a: one home of seat views over shared fact
records, dumped positionally (`ARC-101`, `ARC-109`); 5 since S28-b: attributed forgotten ranges and the pass
log's `prose_contradictions` (`ARC-109`'s note of 2026-10-18). A store or home of another version is refused by
name; memory is rebuilt from the perceived stream, never migrated (QS28-8)."""

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
    # Each range says who forgot it (pr-s28b §5.1): the first and last pass's day and policy (or the host
    # bound's `host:<step>`), and how many passes added to it (an upper bound, F-S28b-2).
    "CREATE TABLE forgotten (seat INTEGER NOT NULL, first_key TEXT NOT NULL, last_key TEXT NOT NULL, "
    "first_day INTEGER NOT NULL, first_policy TEXT NOT NULL, last_day INTEGER NOT NULL, "
    "last_policy TEXT NOT NULL, passes INTEGER NOT NULL CHECK (passes >= 1), "
    "PRIMARY KEY (seat, first_key))",
    "CREATE TABLE retention_passes (seat INTEGER NOT NULL, trigger TEXT NOT NULL, day INTEGER NOT NULL, "
    "cause TEXT NOT NULL CHECK (cause IN ('day', 'size')), policy TEXT NOT NULL, "
    "outcome TEXT NOT NULL, kept INTEGER NOT NULL, merged INTEGER NOT NULL, dropped INTEGER NOT NULL, "
    "forced INTEGER NOT NULL, bytes_before INTEGER NOT NULL, bytes_after INTEGER NOT NULL, "
    "forgotten_ranges INTEGER NOT NULL, model_decisions INTEGER NOT NULL, "
    "fallback_decisions INTEGER NOT NULL, prose_contradictions INTEGER NOT NULL, "
    "PRIMARY KEY (seat, trigger))",
)
FTS_DDL = (
    f"CREATE VIRTUAL TABLE access_text USING fts5(gist, seat UNINDEXED, tokenize='{fts.TOKENIZER}')"
)
"""Notable gists, `rowid` equal to their `access` row's: an index, rebuilt from `access`, never dumped."""

CREATED_KEY = "created"
"""The home's creation marker (`ARC-152`; pr-fu-s28a-1-home-marker.md): a `home_meta` row of 128 random bits in
hex, written once, in the transaction that creates the home (or, for a schema-4 home made before the row
existed, at its first open), and never rewritten. It tells a resumed home from one rebuilt by a replay, whose
rows a replay recreates identically (I-S28-5). It is never dumped: no dump, digest or world state contains it."""
DUMPED_HOME_META = ("schema", "world_instance")
"""The `home_meta` rows `MemoryHome.dump()` writes, in this (key) order; `created` is not among them."""

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
    ViewTable(
        "forgotten",
        (
            "first_key",
            "last_key",
            "first_day",
            "first_policy",
            "last_day",
            "last_policy",
            "passes",
        ),
        "first_key",
    ),
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
            "prose_contradictions",
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

ID_DIGITS = 20
"""The longest `EventId` and `EntityId`: u64's 20 decimal digits."""
K_ACCESS_OVERHEAD_BYTES = (len('["fact",') + len('["access",') - len('["l0",') + ID_DIGITS + 2) + (
    len('["fact_who",') + ID_DIGITS + len("]\n") + len(',1,""') + ID_DIGITS
)
"""`K` (QS28a-1; I-a-6 (b)): the most bytes a one-seat home's dump spends on one held record beyond that record's
lines in the seat view, derived from the grammar of §5.4 (`ARC-101`'s note) alone. 93.

- `fact` + `access` against `l0`: the two tags less one (`["fact",` 8 + `["access",` 10 less `["l0",` 6), the id
  written twice where the view writes it once (≤ 20 digits), and one more `]` and newline (2); every other cell
  appears once in each, with as many commas: 12 + 20 + 2 = 34.
- `fact_who` against `l0_who`: the record's one `fact_who` line costs `["fact_who",` 12, the id (≤ 20), `]` and
  newline (2), and `,role,"entity"` (5 + the entity's digits) per row; the view costs `["l0_who",` 10, the id,
  `,"entity"]` and newline (15 + the id's and entity's digits) per counterpart. So every counterpart costs the home
  ≥ 10 bytes less than the view, and the line's worst case is a record with no counterpart and one row the view
  does not show (the seat's own, or one subject that is not a participant): 12 + 20 + 2 + 5 + 20 = 59.

The seat's identity costs the home 20 bytes less than the view (`["seat", key, self]` and two `home_meta` lines
against four `meta` identity lines, whatever the seat key's length), so a seat with no record is within `K * 0`.
A record whose envelope names more than one entity the view does not show costs 5 + its digits more per further
such entity (ledger F-S28a-9)."""
