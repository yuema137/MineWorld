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
from mineworld_sdk.pacer import Pacer
from mineworld_sdk.perceived import CursorCell, PerceivedBatch
from mineworld_sdk.reconnect import SeatLost as SdkSeatLost
from mineworld_sdk.resuming import ResumingSeat, Seen
from mineworld_sdk.session import Outcome as SdkOutcome
from mineworld_sdk.session import RefusedRequest
from mineworld_sdk.wire.contract import ActionRequest
from mineworld_sdk.wire.ids import ActionId, EntityKey, EventId, event_order

from mineworld_agent.inbox import Inbox
from mineworld_agent.models import SeatLossKind
from mineworld_agent.refs import RefTable
from mineworld_agent.state import SeatState, StateDir
from mineworld_agent.view import Rendered, render

__all__ = ["VIEWS_KEPT", "Handle"]

VIEWS_KEPT = 8
"""Rendered views a handle keeps for deltas, pages and `attempt` (step-27 §5.10)."""

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
    on_inbox: Callable[[], None] | None = None
    """Called, synchronously, after this handle's inbox gained at least one fact (S27-d's inbox hook). The
    `Player` binds it to the handle string; it must not raise."""
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
    streamed: bool = True
    """Whether the seat has the reliable `perceived` stream. A world without a save serves none from the
    beginning (`cursor_unavailable`, `PROTOCOL.md` §5.8); the inbox is then fed from each observation's
    own `events`, best effort, as a client without a stream sees them (D-c-19)."""
    fresh: bool = True
    """Joined without a saved cursor: the backfill up to the connection's head is the world's past, not
    news, and is not put in the inbox (D-c-18)."""
    _head: EventId | None = None
    _head_known: bool = False
    _held: list[PerceivedBatch] = field(default_factory=list[PerceivedBatch])
    _added_through: EventId | None = None
    _not_before: float = 0.0
    _saved_connection: int = 0
    _saved_cursor: EventId | None = None

    # ── Life ──────────────────────────────────────────────────────────────────────────────────────

    def start(self) -> None:
        self.last_call = self.wall()
        self._save(force=True)
        self._tasks = [asyncio.create_task(self._watch())]
        if self.streamed:
            self._tasks.append(asyncio.create_task(self._drain()))
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

    # ── Sending ───────────────────────────────────────────────────────────────────────────────────

    async def send(self, request: ActionRequest) -> tuple[int, SdkOutcome]:
        """One request on this seat, one in flight (the lock), never sooner than the server's last
        `rate_limited { retry_after_ms }` allows (R-S27c-a-1; step-27 §6.5). Returns the connection it
        went on and the answer; raises what `ResumingSeat.submit` raises."""
        async with self.lock:
            wait = self.backoff()
            if wait > 0:
                await self.sleep(wait)
            connection = self.seat.connection
            answer = await self.seat.submit(request)
            if isinstance(answer, RefusedRequest) and answer.code == "rate_limited":
                after = answer.retry_after_ms if answer.retry_after_ms is not None else 1000
                self._not_before = self.wall() + after / 1000
            return connection, answer

    def backoff(self) -> float:
        """Wall seconds before this handle may submit again (0 when it may now)."""
        return max(0.0, self._not_before - self.wall())

    async def reflecting(self, patience: float) -> None:
        """Waits, at most `patience` wall seconds, for an observation that reflects the newest answered
        request (`acted_through`), so that the view after an act shows the act (D-c-20)."""
        with contextlib.suppress(TimeoutError):
            async with asyncio.timeout(patience):
                while True:
                    signal = self.signal()
                    if self.lost is not None or self.reflected():
                        return
                    await signal.wait()

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
                    through = self.cursor.cursor() if self.streamed else self._added_through
                    if through is not None and event_order(through) >= event_order(newest):
                        return
                    await self.inbox.changed.wait()

    # ── Background ────────────────────────────────────────────────────────────────────────────────

    async def _drain(self) -> None:
        try:
            async for batch in self.seat.perceived():
                if self.fresh and not self._head_known:
                    self._held.append(batch)
                else:
                    self._ingest(batch)
        except SdkSeatLost as lost:
            self._lose(lost.reason)
        except SessionClosed:
            return

    def _ingest(self, batch: PerceivedBatch) -> None:
        """Into the inbox, then the cursor (never the other way round: M-c-6)."""
        head = self._head
        news = [
            event
            for event in batch.events
            if not (self.fresh and head is not None and event_order(event.id) <= event_order(head))
        ]
        self.inbox.add(news)
        if news:
            self._announce()
        self.cursor.commit(batch)
        self._save(force=False)

    def _announce(self) -> None:
        """Tell the hook that the inbox gained facts (after `Inbox.add`, so the hook sees them)."""
        if self.on_inbox is not None:
            self.on_inbox()

    async def _watch(self) -> None:
        try:
            while True:
                seen = await self.seat.changed(self.newest)
                first, self.newest = self.newest is None, seen
                if first and self.fresh and self.streamed:
                    # The backfill ends at the connection's head, and is sent before the first
                    # observation (`PROTOCOL.md` §5.8): what came through by then is the past.
                    self._head, self._head_known = seen.perceived_through, True
                    held, self._held = self._held, []
                    for batch in held:
                        self._ingest(batch)
                if not self.streamed:
                    self._take(seen)
                if seen.connection != self._saved_connection:
                    self._save(force=True)
                self._bump()
        except SdkSeatLost as lost:
            self._lose(lost.reason)
        except SessionClosed:
            return

    def _take(self, seen: Seen) -> None:
        """Without a stream: the facts this observation says I learned since the previous frame."""
        events = seen.frame.observation.events
        if events:
            self.inbox.add(events)
            self._announce()
            newest = max((event.id for event in events), key=event_order)
            if self._added_through is None or event_order(newest) > event_order(
                self._added_through
            ):
                self._added_through = newest

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
            self._tasks.append(asyncio.create_task(self.pacer.stop()))

    def _bump(self) -> None:
        observed, self._observed = self._observed, asyncio.Event()
        observed.set()

    # ── The state file ────────────────────────────────────────────────────────────────────────────

    def _save(self, *, force: bool) -> None:
        """Rewrites the state file when a new connection's welcome brought a fresh `resume`
        (`force`), or when the committed cursor moved on (D-c-16)."""
        if self.states is None or self.lost is not None:
            return
        cursor = self.cursor.cursor()
        if not force and cursor == self._saved_cursor:
            return
        welcome = self.seat.welcome
        self.states.write(
            self.key,
            SeatState(instance=welcome.world.instance, resume=welcome.resume, cursor=cursor),
        )
        self._saved_connection = self.seat.connection
        self._saved_cursor = cursor

    def __repr__(self) -> str:
        return f"Handle(seat={self.key!r}, views={len(self._renders)}, lost={self.lost!r})"
