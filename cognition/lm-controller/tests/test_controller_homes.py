"""The `[memory]` table, the page cache, and the process's homes with their write lock (pr-s28f §§5.2-5.4;
AC-f-1, AC-f-5, AC-f-6).

The concurrency case is F-S28f-1: an assisted pass awaits its model inside its transaction on the home's one
connection, so a second seat's write in that window must wait for the home's lock. Its oracle is the same two calls
run one after the other in a fresh home: no expected byte comes from the code under test.
"""

from __future__ import annotations

import asyncio
import re
import sqlite3
from pathlib import Path

import pytest
from memory_fixtures import INSTANCE, alice_identity
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from retention_stream import PEOPLE, stream
from support import run
from test_controller_config import BASE, write
from test_lm_summarise import GENEROUS, gateway, retention, scripted

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.config import ConfigError, load
from mineworld_cognition.controller.homes import SeatHomes, home_path
from mineworld_cognition.memory.config import MemoryConfig, SharingModeUnavailable
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve

SOURCE = Path(__file__).resolve().parents[1] / "src" / "mineworld_cognition"
BOB = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=PEOPLE[1])


# ── AC-f-1: the [memory] table ────────────────────────────────────────────────────────────────────


def test_a_file_without_memory_gets_the_defaults(tmp_path: Path) -> None:
    memory = load(write(tmp_path, BASE)).memory
    assert (memory.sharing, memory.page_cache_kib, memory.verify) == ("home", 8192, "off")


@pytest.mark.parametrize(
    ("table", "names"),
    [
        ("page_cache_kib = 255\n", "memory.page_cache_kib"),
        ("page_cache_kib = 1048577\n", "memory.page_cache_kib"),
        ('verify = "always"\n', "memory.verify"),
        ('sharing = "cloud"\n', "memory.sharing"),
        ("cache_records = 10\n", "memory.cache_records"),
        ('home = "x/memory.sqlite"\n', "memory.home"),
    ],
)
def test_a_wrong_memory_value_is_refused_by_name(tmp_path: Path, table: str, names: str) -> None:
    with pytest.raises(ConfigError, match=re.escape(names)):
        load(write(tmp_path, BASE + "[memory]\n" + table))


def test_service_is_refused_with_its_own_type_as_the_cause(tmp_path: Path) -> None:
    with pytest.raises(ConfigError) as raised:
        load(write(tmp_path, BASE + '[memory]\nsharing = "service"\n'))
    cause = raised.value.__cause__
    assert isinstance(cause, SharingModeUnavailable)
    assert cause.mode == "service"
    assert str(cause).startswith("memory.config:")


def test_the_page_cache_reaches_sqlite_and_a_direct_caller_is_bounded(tmp_path: Path) -> None:
    with MemoryHome.open(tmp_path / "m.sqlite", INSTANCE, page_cache_kib=1024) as home:
        assert home.connection.execute("PRAGMA cache_size").fetchone()[0] == -1024
    with MemoryHome.open(tmp_path / "d.sqlite", INSTANCE) as home:
        assert home.connection.execute("PRAGMA cache_size").fetchone()[0] == -8192
    with pytest.raises(ValueError, match="memory.home: page_cache_kib"):
        MemoryHome.open(tmp_path / "x.sqlite", INSTANCE, page_cache_kib=255)
    assert not (tmp_path / "x.sqlite").exists()


# ── AC-f-6: the homes of one process ──────────────────────────────────────────────────────────────


def test_the_layout_is_fixed_by_sharing(tmp_path: Path) -> None:
    alice = EntityKey("alice")
    assert home_path(tmp_path, INSTANCE, alice, "home") == tmp_path / INSTANCE / "memory.sqlite"
    assert home_path(tmp_path, INSTANCE, alice, "none") == tmp_path / INSTANCE / "alice.sqlite"


def test_seats_of_one_instance_share_one_home_and_one_lock(tmp_path: Path) -> None:
    async def go() -> None:
        homes = SeatHomes(tmp_path, MemoryConfig())
        alice = await homes.hold(alice_identity())
        bob = await homes.hold(BOB)
        assert alice.view.home is bob.view.home
        assert alice.lock is bob.lock
        assert homes.open_paths() == (tmp_path / INSTANCE / "memory.sqlite",)
        with pytest.raises(ValueError, match="seat alice is already held"):
            await homes.hold(alice_identity())
        homes.release(EntityKey("alice"))
        homes.release(EntityKey("alice"))  # twice: a no-op
        assert homes.open_paths() == (tmp_path / INSTANCE / "memory.sqlite",), "bob still holds it"
        homes.release(EntityKey("bob"))
        assert homes.open_paths() == ()
        with pytest.raises(ValueError, match="closed"):
            _ = bob.view.home.connection

    run(go())


def test_under_none_each_seat_has_its_own_home_and_close_closes_all(tmp_path: Path) -> None:
    async def go() -> None:
        homes = SeatHomes(tmp_path, MemoryConfig(sharing="none"))
        alice = await homes.hold(alice_identity())
        bob = await homes.hold(BOB)
        assert alice.view.home is not bob.view.home
        assert alice.lock is not bob.lock
        assert set(homes.open_paths()) == {
            tmp_path / INSTANCE / "alice.sqlite",
            tmp_path / INSTANCE / "bob.sqlite",
        }
        homes.close()
        assert homes.open_paths() == ()
        with pytest.raises(ValueError, match="closed"):
            _ = alice.view.home.connection

    run(go())


def test_the_controller_opens_homes_only_through_seat_homes() -> None:
    controller = SOURCE / "controller"
    openers = [
        path.name
        for path in sorted(controller.rglob("*.py"))
        if "MemoryHome.open(" in path.read_text(encoding="utf-8") and path.name != "homes.py"
    ]
    assert openers == []
    homes = (controller / "homes.py").read_text(encoding="utf-8")
    for named in ("alice", "cafe", "social-cafe", "barista"):
        assert named not in homes.lower(), f"the layout names {named}"


# ── AC-f-5: the write lock ────────────────────────────────────────────────────────────────────────


class _Gated:
    """A backend whose first call waits for `gate`; every answer is `inner`'s."""

    def __init__(self, gate: asyncio.Event) -> None:
        self._inner = scripted()
        self._gate: asyncio.Event | None = gate
        self.calls = 0

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.calls += 1
        if self._gate is not None:
            gate, self._gate = self._gate, None
            await gate.wait()
        return await self._inner.complete(request)

    async def aclose(self) -> None:
        return None


def _two_days() -> list[PerceivedEvent]:
    return list(stream(11, 2))


def _consolidate() -> Retention:
    return resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed())


async def _absorb_all(
    view: SeatView, facts: list[PerceivedEvent], kept: Retention, lock: asyncio.Lock
) -> None:
    async with lock:
        await absorb(view, facts, facts[-1].id, kept)


def test_a_second_seats_write_waits_for_an_assisted_pass_parked_at_its_model(
    tmp_path: Path,
) -> None:
    facts = _two_days()
    bobs = facts[::2]

    async def concurrent() -> tuple[bytes, bytes]:
        gate = asyncio.Event()
        backend = _Gated(gate)
        assisted = retention(gateway(backend, **GENEROUS), calls_per_sim_day=24)
        homes = SeatHomes(tmp_path / "concurrent", MemoryConfig())
        alice = await homes.hold(alice_identity())
        bob = await homes.hold(BOB)
        first = asyncio.create_task(_absorb_all(alice.view, facts, assisted, alice.lock))
        while backend.calls == 0:
            await asyncio.sleep(0)
        assert not first.done(), "alice's pass is parked at its model call"
        second = asyncio.create_task(_absorb_all(bob.view, bobs, _consolidate(), bob.lock))
        for _ in range(20):
            await asyncio.sleep(0)
        assert not second.done(), "bob's write waits for alice's pass to commit"
        gate.set()
        await asyncio.gather(first, second)
        dumps = alice.view.dump(), bob.view.dump()
        homes.close()
        return dumps

    async def one_after_the_other() -> tuple[bytes, bytes]:
        assisted = retention(gateway(scripted(), **GENEROUS), calls_per_sim_day=24)
        homes = SeatHomes(tmp_path / "sequential", MemoryConfig())
        alice = await homes.hold(alice_identity())
        bob = await homes.hold(BOB)
        await _absorb_all(alice.view, facts, assisted, alice.lock)
        await _absorb_all(bob.view, bobs, _consolidate(), bob.lock)
        dumps = alice.view.dump(), bob.view.dump()
        homes.close()
        return dumps

    assert run(concurrent(), timeout_s=120) == run(one_after_the_other(), timeout_s=120)


def test_without_the_lock_a_second_seats_write_fails_inside_the_open_pass(tmp_path: Path) -> None:
    """Why the lock exists (F-S28f-1): the same window with no lock is SQLite's nested-transaction error."""
    facts = _two_days()

    async def go() -> None:
        gate = asyncio.Event()
        backend = _Gated(gate)
        assisted = retention(gateway(backend, **GENEROUS), calls_per_sim_day=24)
        homes = SeatHomes(tmp_path, MemoryConfig())
        alice = await homes.hold(alice_identity())
        bob = await homes.hold(BOB)
        first = asyncio.create_task(absorb(alice.view, facts, facts[-1].id, assisted))
        while backend.calls == 0:
            await asyncio.sleep(0)
        with pytest.raises(sqlite3.OperationalError, match="within a transaction"):
            await absorb(bob.view, facts[::2], facts[::2][-1].id, _consolidate())
        gate.set()
        await first
        homes.close()

    run(go(), timeout_s=120)
