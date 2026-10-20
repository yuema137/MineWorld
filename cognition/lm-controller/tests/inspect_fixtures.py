"""The homes the inspection tests read (pr-s28e §7): the synthetic 12-seat world (seed 7, 10 days, `home_streams`)
through `absorb` under `consolidate`, in day frames, every seat a view of one file home. Test-only; the oracles
are the streams and the dumps, never a report of the code under test.
"""

from __future__ import annotations

from collections.abc import Set as AbstractSet
from dataclasses import dataclass
from functools import cache
from pathlib import Path

from home_streams import CAST, SEATS, day_frames, seat_streams, world
from memory_fixtures import INSTANCE, rows
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import retention
from support import run

from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb

SEED = 7
DAYS = 10


@cache
def streams() -> dict[EntityKey, list[PerceivedEvent]]:
    return seat_streams(world(SEED, DAYS))


def identity(seat: EntityKey) -> StoreIdentity:
    return StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=CAST[seat])


@dataclass(frozen=True)
class Oracle:
    """pr-s28e §5.3's table, computed from one seat view dump alone (the C-7 oracle)."""

    held: frozenset[int]
    cursor: int | None
    watermark: int | None
    spans: tuple[tuple[int, int], ...]
    """Every episode's and chapter range's ends."""
    forgotten: tuple[tuple[int, int], ...]

    @staticmethod
    def of(dump: bytes) -> Oracle:
        meta = {str(row["key"]): row["value"] for row in rows(dump, "meta")}

        def number(value: object) -> int | None:
            return None if value is None else int(str(value))

        def ends(table: str) -> tuple[tuple[int, int], ...]:
            return tuple(
                (int(str(row["first_key"])), int(str(row["last_key"]))) for row in rows(dump, table)
            )

        return Oracle(
            held=frozenset(int(str(row["event_id"])) for row in rows(dump, "l0")),
            cursor=number(meta["cursor"]),
            watermark=number(meta["forgotten_below"]),
            spans=ends("episodes") + ends("chapter_ranges"),
            forgotten=ends("forgotten"),
        )

    def classify(self, event: str, perceived: AbstractSet[str]) -> str:
        key = int(event)
        covered = any(first <= key <= last for first, last in self.spans)
        if key in self.held:
            return "held"
        if self.cursor is None or key > self.cursor:
            return "not-yet-delivered"
        if event not in perceived:
            return "covered-not-perceived" if covered else "never-perceived"
        if covered:
            return "merged"
        if any(first <= key <= last for first, last in self.forgotten):
            return "forgotten"
        if self.watermark is not None and key <= self.watermark:
            return "forgotten-below-watermark"
        return "incoherent"


def build(path: Path | str) -> None:
    """Every seat's stream absorbed in day frames into the home at `path` (`":memory:"` excluded: a file)."""
    kept = retention("consolidate")
    where = Path(path)
    with MemoryHome.open(where, INSTANCE) as home:
        for seat in SEATS:
            view = home.seat(identity(seat))
            for frame in day_frames(streams()[seat]):
                run(absorb(view, frame, frame[-1].id, kept), timeout_s=600)
