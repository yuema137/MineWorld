"""The pool of players, one per credential (pr-s27e §5.3, C2), with a `Player` double: no socket."""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import cast

import pytest
from mineworld_sdk.wire.frames import Invite
from support import run

from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.pool import Credentials, Full, partition

ONE = Invite("credential-one-0001")
TWO = Invite("credential-two-0002")
THREE = Invite("credential-three-0003")


class FakePlayer:
    """What the pool uses of a `Player`: `active()` and `close()`; it keeps the config and credential it was given."""

    def __init__(self, config: PlayerConfig, credential: Invite) -> None:
        self.config = config
        self.credential = credential
        self.playing = 0
        self.closed = False

    def active(self) -> int:
        return self.playing

    async def close(self) -> None:
        self.closed = True


def _pool(limit: int, state_dir: Path | None = None) -> tuple[Credentials, list[FakePlayer]]:
    made: list[FakePlayer] = []

    def make(config: PlayerConfig, credential: Invite, _on_inbox: Callable[[str], None]) -> Player:
        player = FakePlayer(config, credential)
        made.append(player)
        return cast(Player, player)

    config = PlayerConfig(server="ws://127.0.0.1:1/ws", state_dir=state_dir)
    return Credentials(config, limit=limit, on_inbox=lambda _h: None, make=make), made


def _fake(player: Player) -> FakePlayer:
    return cast(FakePlayer, player)


def test_one_credential_one_player_two_credentials_two() -> None:
    async def scenario() -> None:
        pool, made = _pool(4)
        first = await pool.player_for(ONE)
        assert await pool.player_for(Invite("credential-one-0001")) is first
        second = await pool.player_for(TWO)
        assert second is not first
        assert len(made) == 2
        assert _fake(first).credential == ONE and _fake(second).credential == TWO

    run(scenario(), timeout_s=10)


def test_two_credentials_two_partitions(tmp_path: Path) -> None:
    """Each credential's `Player` keeps its files under its own partition; the partition is stable across pools
    (each pool has a new salt) and differs between credentials."""

    async def scenario() -> tuple[Path | None, Path | None]:
        pool, _ = _pool(4, tmp_path)
        one = _fake(await pool.player_for(ONE)).config.state_dir
        two = _fake(await pool.player_for(TWO)).config.state_dir
        return one, two

    one, two = run(scenario(), timeout_s=10)
    assert one == tmp_path / partition(ONE)
    assert two == tmp_path / partition(TWO)
    assert one != two
    again, _ = run(scenario(), timeout_s=10)
    assert again == one
    assert len(partition(ONE)) == 32
    assert ONE.reveal() not in partition(ONE)


def test_without_a_state_dir_no_partition() -> None:
    async def scenario() -> Path | None:
        pool, _ = _pool(2)
        return _fake(await pool.player_for(ONE)).config.state_dir

    assert run(scenario(), timeout_s=10) is None


def test_a_full_pool_evicts_the_least_recently_used_idle_player() -> None:
    async def scenario() -> None:
        pool, made = _pool(2)
        await pool.player_for(ONE)
        await pool.player_for(TWO)
        await pool.player_for(ONE)  # TWO is now the least recently used
        await pool.player_for(THREE)
        assert made[1].closed and not made[0].closed
        assert len(made) == 3

    run(scenario(), timeout_s=10)


def test_an_active_player_is_never_evicted() -> None:
    async def scenario() -> None:
        pool, made = _pool(2)
        await pool.player_for(ONE)
        await pool.player_for(TWO)
        made[0].playing = 1  # ONE is the least recently used, but plays a seat
        await pool.player_for(THREE)
        assert made[1].closed and not made[0].closed

    run(scenario(), timeout_s=10)


def test_full_when_every_slot_is_active() -> None:
    async def scenario() -> None:
        pool, made = _pool(2)
        await pool.player_for(ONE)
        await pool.player_for(TWO)
        made[0].playing = made[1].playing = 1
        assert await pool.admit(THREE) is False
        with pytest.raises(Full):
            await pool.player_for(THREE)
        assert not made[0].closed and not made[1].closed
        made[1].playing = 0
        assert await pool.admit(THREE) is True
        assert made[1].closed

    run(scenario(), timeout_s=10)


def test_admit_then_player_for_is_a_hit() -> None:
    async def scenario() -> None:
        pool, made = _pool(1)
        assert await pool.admit(ONE) is True
        await pool.player_for(ONE)
        assert len(made) == 1

    run(scenario(), timeout_s=10)


def test_close_closes_every_player() -> None:
    async def scenario() -> None:
        pool, made = _pool(3)
        await pool.player_for(ONE)
        await pool.player_for(TWO)
        await pool.close()
        assert all(player.closed for player in made)
        assert "players=0" in repr(pool)

    run(scenario(), timeout_s=10)


def test_the_repr_names_no_credential_and_no_key() -> None:
    async def scenario() -> str:
        pool, _ = _pool(3)
        await pool.player_for(ONE)
        return repr(pool)

    shown = run(scenario(), timeout_s=10)
    assert shown == "Credentials(players=1, limit=3)"
    assert ONE.reveal() not in shown and partition(ONE) not in shown
