"""The memory homes of one cognition process (pr-s28f-home-wiring.md §5.4; step-28 §6.6; `ARC-98`).

`SeatHomes` is the controller's only opener and closer of memory homes. Under `[memory] sharing = "home"` the seats
of one process share one home per world instance, at `<store>/<instance>/memory.sqlite`; under `"none"` each seat
has its own, at `<store>/<instance>/<seat>.sqlite`. Nothing a seat recalls depends on the choice (I-S28-2).

A home has one connection, and an assisted retention pass awaits its model inside its transaction
(`memory/retention/host.py`). So each home has one `asyncio.Lock` (stdlib; one loop, one connection), and every call
that can write to the home runs under it: a seat's first `home.seat()` here, and every `absorb` in the seat runner.
Reads take no lock: while a pass awaits, nothing is written yet, so a read sees committed rows only.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from pathlib import Path

from mineworld_sdk.wire.ids import EntityKey, WorldInstanceId

from mineworld_cognition.memory.config import MemoryConfig, Sharing
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.records import StoreIdentity

HOME_FILE = "memory.sqlite"


@dataclass(frozen=True)
class HeldSeat:
    view: SeatView
    lock: asyncio.Lock
    """The home's write lock, shared by every seat of the home."""
    home: MemoryHome
    """The home the view lives in (the seat binds its memory stack on it, pr-s28d §5.9)."""


@dataclass
class _Open:
    home: MemoryHome
    lock: asyncio.Lock
    holders: set[EntityKey] = field(default_factory=set[EntityKey])


def home_path(store: Path, instance: WorldInstanceId, seat: EntityKey, sharing: Sharing) -> Path:
    """`<store>/<instance>/memory.sqlite` under `home`; `<store>/<instance>/<seat>.sqlite` under `none`. Both parts
    are validated identifiers (a 32-hex instance, a seat key), so neither names another directory."""
    if sharing == "service":
        raise ValueError(
            "controller.homes: sharing = service is refused when the configuration loads"
        )
    directory = store / str(instance)
    return directory / (HOME_FILE if sharing == "home" else f"{seat}.sqlite")


class SeatHomes:
    """The memory homes of one cognition process (step-28 §6.6, MVP-0): one per world instance under
    `sharing = "home"`, one per seat under `"none"`. The only opener and closer of homes in the controller."""

    def __init__(self, store: Path, memory: MemoryConfig) -> None:
        self._store = store
        self._memory = memory
        self._open: dict[Path, _Open] = {}
        self._held: dict[EntityKey, Path] = {}

    async def hold(self, identity: StoreIdentity) -> HeldSeat:
        """The seat's view in its home, the home opened on its first seat. A home's refusal (`HomeBusy`,
        `HomeIdentityMismatch`, `UnsupportedStoreSchema`, `NotAMemoryHome`) propagates unchanged."""
        if identity.seat in self._held:
            raise ValueError(f"controller.homes: seat {identity.seat} is already held")
        path = home_path(self._store, identity.world_instance, identity.seat, self._memory.sharing)
        opened = self._open.get(path)
        if opened is None:
            path.parent.mkdir(parents=True, exist_ok=True)
            home = MemoryHome.open(
                path,
                identity.world_instance,
                verify=self._memory.verify,
                page_cache_kib=self._memory.page_cache_kib,
            )
            opened = self._open[path] = _Open(home=home, lock=asyncio.Lock())
        # F-S28f-1: a new seat's row is never written into another seat's open transaction.
        try:
            async with opened.lock:
                view = opened.home.seat(identity)
        except BaseException:
            if not opened.holders:
                opened.home.close()
                del self._open[path]
            raise
        opened.holders.add(identity.seat)
        self._held[identity.seat] = path
        return HeldSeat(view=view, lock=opened.lock, home=opened.home)

    def release(self, seat: EntityKey) -> None:
        """Drops the seat from its home's holders; the home closes with its last holder. Twice is a no-op."""
        path = self._held.pop(seat, None)
        if path is None:
            return
        opened = self._open[path]
        opened.holders.discard(seat)
        if not opened.holders:
            opened.home.close()
            del self._open[path]

    def close(self) -> None:
        """Closes every home still open (once, when serving ends)."""
        for opened in self._open.values():
            opened.home.close()
        self._open.clear()
        self._held.clear()

    def open_paths(self) -> tuple[Path, ...]:
        """The paths of the homes open now, in opening order."""
        return tuple(self._open)
