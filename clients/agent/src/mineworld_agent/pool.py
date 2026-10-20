"""A bounded pool of `Player`s, one per credential, for an adapter that serves several agents (S27-e §5.3).

Over Streamable HTTP each request names its credential in its bearer. The pool gives each credential its own
`Player`, so a handle minted under one credential is simply absent from another credential's `Player`, which
answers `unknown_handle` (step-27 §6.2's handle binding, by construction). The pool never validates a credential —
the world server does at `join` — and never logs, persists or prints one.

- **Keys.** A credential is looked up by an HMAC of it under a per-process random salt, so no credential is a dict
  key and none is compared byte by byte.
- **State.** With a `state_dir`, each credential's `Player` keeps its files in `STATE_DIR/<partition>`, a stable,
  irreversible digest of the credential: one credential's `Player` never reads another's resume file (F-S27e-3).
- **Bound.** At most `limit` `Player`s. A miss on a full pool evicts the least recently used `Player` that plays no
  seat; when every `Player` plays one, the pool is `Full` (the HTTP gate answers 503, QS27e-2).

No `mcp` import here (I-d-1).
"""

from __future__ import annotations

import asyncio
import hashlib
import hmac
import itertools
import secrets
from collections.abc import Callable
from dataclasses import dataclass

from mineworld_sdk.wire.frames import Invite

from mineworld_agent.player import Player, PlayerConfig

__all__ = ["Credentials", "Full", "partition"]

type MakePlayer = Callable[[PlayerConfig, Invite, Callable[[str], None]], Player]

_PARTITION_LABEL = b"mineworld-agent state v1\x00"


def partition(credential: Invite) -> str:
    """The state subdirectory name of a credential: 32 hex digits of SHA-256 over a fixed label and the credential.
    Stable across restarts (a restarted adapter resumes the seats a credential held) and not reversible."""
    digest = hashlib.sha256(_PARTITION_LABEL + credential.reveal().encode("utf-8"))
    return digest.hexdigest()[:32]


class Full(Exception):
    """Every slot holds a `Player` that plays a seat; a new credential has to wait."""


def _player(config: PlayerConfig, credential: Invite, on_inbox: Callable[[str], None]) -> Player:
    return Player(config, credential, on_inbox=on_inbox)


@dataclass(slots=True)
class _Slot:
    player: Player
    last_used: int


class Credentials:
    """credential → its own `Player`, at most `limit` of them."""

    def __init__(
        self,
        config: PlayerConfig,
        *,
        limit: int,
        on_inbox: Callable[[str], None],
        make: MakePlayer = _player,
    ) -> None:
        if limit < 1:
            raise ValueError("limit must be at least 1")
        self._config = config
        self._limit = limit
        self._on_inbox = on_inbox
        self._make = make
        self._salt = secrets.token_bytes(32)
        self._slots: dict[str, _Slot] = {}
        self._lock = asyncio.Lock()
        self._clock = itertools.count()

    def _key(self, credential: Invite) -> str:
        return hmac.new(self._salt, credential.reveal().encode("utf-8"), "sha256").hexdigest()

    async def admit(self, credential: Invite) -> bool:
        """Reserves the credential's `Player` (creating it, evicting an idle one if needed). `False` only when every
        slot holds a `Player` that plays a seat."""
        try:
            await self.player_for(credential)
        except Full:
            return False
        return True

    async def player_for(self, credential: Invite) -> Player:
        """The credential's `Player`. Raises `Full` when the pool is full and no `Player` in it is idle."""
        key = self._key(credential)
        async with self._lock:
            slot = self._slots.get(key)
            if slot is None:
                if len(self._slots) >= self._limit:
                    await self._evict()
                slot = _Slot(
                    self._make(self._config_for(credential), credential, self._on_inbox), 0
                )
                self._slots[key] = slot
            slot.last_used = next(self._clock)
            return slot.player

    async def _evict(self) -> None:
        idle = [
            (slot.last_used, key) for key, slot in self._slots.items() if slot.player.active() == 0
        ]
        if not idle:
            raise Full
        _, key = min(idle)
        await self._slots.pop(key).player.close()

    def _config_for(self, credential: Invite) -> PlayerConfig:
        root = self._config.state_dir
        if root is None:
            return self._config
        return self._config.model_copy(update={"state_dir": root / partition(credential)})

    async def close(self) -> None:
        """Closes every `Player`: each handle leaves, so each seat returns to its default at once."""
        async with self._lock:
            slots, self._slots = list(self._slots.values()), {}
        for slot in slots:
            await slot.player.close()

    def __repr__(self) -> str:
        return f"Credentials(players={len(self._slots)}, limit={self._limit})"
