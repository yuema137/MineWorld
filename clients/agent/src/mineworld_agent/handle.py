"""One handle of a `Player`: one seat, and everything the library keeps for it (pr-s27c §5.3).

A handle owns one `ResumingSeat` and, for that seat only, its references, its last eight rendered views,
its inbox, its walk pacer and its submit lock (one request in flight, step-27 §6.5). Three background tasks
serve it and nothing else:

- **the drain** reads the reliable `perceived` stream into the inbox and commits each batch's `through`
  to the cursor only after the batch is in the inbox (INV-S27-6: neither the SDK's buffer nor the server's
  queue ever fills; M-c-6), saving the cursor to the state file as it advances;
- **the watch** keeps the newest observation (newest wins: an idle agent loses observations and nothing
  else) and rewrites the state file after every welcome, which carries a fresh `resume`;
- **the idle timer** gives the seat back (`leave`) after `idle_release_s` wall seconds without a call
  (step-27 §5.9; QS27-3).

Nothing here is shared with another handle (I-c-2): no module-level table, cache or inbox.
"""

from __future__ import annotations

import asyncio
import contextlib
from collections import OrderedDict
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, field

from mineworld_sdk.errors import SessionClosed
from mineworld_sdk.perceived import CursorCell
from mineworld_sdk.reconnect import SeatLost as SdkSeatLost
from mineworld_sdk.resuming import ResumingSeat, Seen
from mineworld_sdk.wire.ids import ActionId, EntityKey, EventId, event_order

from mineworld_agent.inbox import Inbox
from mineworld_agent.models import SeatLossKind
from mineworld_agent.pacer import Pacer
from mineworld_agent.refs import RefTable
from mineworld_agent.state import SeatState, StateDir
from mineworld_agent.view import Rendered, render

__all__ = ["VIEWS_KEPT", "Handle"]

VIEWS_KEPT = 8
"""Rendered views a handle keeps for deltas, pages and `attempt` (step-27 §5.10)."""

CURSOR_SAVE_EVERY = 1.0
"""Wall seconds between two saves of an advancing cursor; a welcome is saved at once. A program that dies
between two saves rejoins from a slightly older cursor and is sent those facts again."""

type Wall = Callable[[], float]
type Sleep = Callable[[float], Awaitable[None]]


def _action_order(action: ActionId) -> int:
    return int(action)


@dataclass(eq=False, slots=True)
class Handle:
    """One seat of one Player."""

    seat: ResumingSeat
    key: EntityKey
    refs: RefTable
    inbox: Inbox
    cursor: CursorCell
    wall: Wall
    sleep: Sleep
    idle_release_s: int
    states: StateDir | None
    pacer: Pacer | None = None
    lock: asyncio.Lock = field(default_factory=asyncio.Lock)
    newest: Seen | None = None
    shown: Seen | None = None
    """The observation of the last view returned to the agent (for `wait(view_changed)`)."""
    last_call: float = 0.0
    last_at: int | None = None
    lost: SeatLossKind | None = None
    last_action: tuple[int, ActionId] | None = None
    """The connection and `ActionId` of the newest request this handle submitted that was answered."""
    _renders: OrderedDict[int, Rendered] = field(default_factory=OrderedDict[int, Rendered])
    _rendered_seen: Seen | None = None
    _next_view: int = 1
    _observed: asyncio.Event = field(default_factory=asyncio.Event)
    _tasks: list[asyncio.Task[None]] = field(default_factory=list[asyncio.Task[None]])
    _saved_connection: int = 0
    _saved_at: float | None = None

    # ── Life ──────────────────────────────────────────────────────────────────────────────────────

    def start(self) -> None:
        self.last_call = self.wall()
        self._save(force=True)
        self._tasks = [
            asyncio.create_task(self._drain()),
            asyncio.create_task(self._watch()),
        ]
        if self.idle_release_s > 0:
            self._tasks.append(asyncio.create_task(self._idle()))

    async def first_observation(self, patience: float) -> None:
        """Waits for the seat's first observation (the stream's first frame, `PROTOCOL.md` §5), or for
        the seat to end (`lost`)."""
        async with asyncio.timeout(patience):
            while True:
                signal = self.signal()
                if self.newest is not None or self.lost is not None:
                    return
                await signal.wait()

    def signal(self) -> asyncio.Event:
        """Set when a newer observation arrives or the seat ends. Take it *before* looking at `newest`,
        then wait on it: nothing that happens in between is missed."""
        return self._observed

    async def stop(self, *, give_back: bool) -> None:
        """Ends the handle: stops the pacer and the tasks; with `give_back`, sends `leave` and deletes
        the state file."""
        if self.pacer is not None:
            await self.pacer.stop()
        current = asyncio.current_task()
        for task in self._tasks:
            if task is not current and not task.done():
                task.cancel()
        await asyncio.gather(*(t for t in self._tasks if t is not current), return_exceptions=True)
        if give_back:
            with contextlib.suppress(Exception):
                await self.seat.leave()
            if self.states is not None:
                self.states.delete(self.key)

    def touch(self) -> None:
        self.last_call = self.wall()

    # ── Views ─────────────────────────────────────────────────────────────────────────────────────

    def rendered(self) -> Rendered:
        """The newest observation, rendered once and numbered; the last eight are kept."""
        seen = self.newest
        if seen is None:
            raise RuntimeError("a handle has an observation once joined")
        held = self._renders.get(self._next_view - 1)
        if held is not None and self._rendered_seen is seen:
            return held
        number = self._next_view
        self._next_view += 1
        made = render(seen.frame.observation, self.refs, number, self._acted_through(seen))
        self._renders[number] = made
        self._rendered_seen = seen
        while len(self._renders) > VIEWS_KEPT:
            self._renders.popitem(last=False)
        return made

    def held(self, view: int) -> Rendered | None:
        return self._renders.get(view)

    def _acted_through(self, seen: Seen) -> bool:
        """Whether `seen` reflects every request this handle submitted: its frame's `acted_through`
        reached the newest answered request on the same connection. A request of an earlier connection
        is reflected by any observation of a later one, which is a whole observation of the world after
        it (`PROTOCOL.md` §4.2: the stream restarts with a whole observation)."""
        last = self.last_action
        if last is None or last[0] != seen.connection:
            return True
        through = seen.frame.acted_through
        return through is not None and _action_order(through) >= _action_order(last[1])

    def reflected(self) -> bool:
        """Whether the newest observation reflects the newest answered request."""
        return self.newest is not None and self._acted_through(self.newest)

    # ── Facts an act caused ───────────────────────────────────────────────────────────────────────

    async def delivered_through(self, events: list[EventId], patience: float) -> None:
        """Waits until the drain has delivered past every fact in `events`, or `patience` wall seconds:
        whatever of them this seat perceives is then in the inbox."""
        if not events:
            return
        newest = max(events, key=event_order)
        with contextlib.suppress(TimeoutError):
            async with asyncio.timeout(patience):
                while True:
                    self.inbox.changed.clear()
                    cursor = self.cursor.cursor()
                    if cursor is not None and event_order(cursor) >= event_order(newest):
                        return
                    await self.inbox.changed.wait()

    # ── Background ────────────────────────────────────────────────────────────────────────────────

    async def _drain(self) -> None:
        try:
            async for batch in self.seat.perceived():
                self.inbox.add(batch.events)
                self.cursor.commit(batch)
                self._save(force=False)
        except SdkSeatLost as lost:
            self._lose(lost.reason)
        except SessionClosed:
            return

    async def _watch(self) -> None:
        try:
            while True:
                seen = await self.seat.changed(self.newest)
                self.newest = seen
                if seen.connection != self._saved_connection:
                    self._save(force=True)
                self._bump()
        except SdkSeatLost as lost:
            self._lose(lost.reason)
        except SessionClosed:
            return

    async def _idle(self) -> None:
        while True:
            remaining = self.last_call + self.idle_release_s - self.wall()
            if remaining <= 0:
                self.lost = "idle_released"
                self._bump()
                await self.stop(give_back=True)
                return
            await self.sleep(remaining)

    def _lose(self, reason: SeatLossKind) -> None:
        if self.lost is None:
            self.lost = reason
        self._bump()
        if self.pacer is not None:
            # The pacer's next step meets the same loss and stops; nothing more is sent.
            asyncio.get_running_loop().create_task(self.pacer.stop())

    def _bump(self) -> None:
        observed, self._observed = self._observed, asyncio.Event()
        observed.set()

    # ── The state file ────────────────────────────────────────────────────────────────────────────

    def _save(self, *, force: bool) -> None:
        if self.states is None or self.lost is not None:
            return
        now = self.wall()
        if not force and self._saved_at is not None and now - self._saved_at < CURSOR_SAVE_EVERY:
            return
        welcome = self.seat.welcome
        cursor = self.cursor.cursor()
        self.states.write(
            self.key,
            SeatState(instance=welcome.world.instance, resume=welcome.resume, cursor=cursor),
        )
        self._saved_connection = self.seat.connection
        self._saved_at = now

    def __repr__(self) -> str:
        return f"Handle(seat={self.key!r}, views={len(self._renders)}, lost={self.lost!r})"
