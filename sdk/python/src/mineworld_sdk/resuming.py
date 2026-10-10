"""A seat that survives what a real network does to it: dropped sockets, `lagged`, server restarts.

`ResumingSeat` composes one `SeatSession` per connection (design D-P3b-1) and owns what outlives one:
the world instance, the observer, the caller's cursor source and the perceived stream, the connection
counter, and the backoff. When a connection ends it decides, by the table of the design's §4.5
(`reconnect.py`) and nothing else, whether to rejoin — with the welcome's `resume` while the server
still holds the seat, afresh after the hold or a restart — or to end with a typed `SeatLost`.

- Every join presents as `since` what the caller's `CursorSource` reports, never a cursor the SDK
  advanced on receipt (P3b-2). `cursor_unavailable` is final; no other cursor is tried (P3b-3).
- A submit is sent at most once. One whose connection ends before its answer fails `AnswerLost`; one
  made between connections raises `NotConnected` and sends nothing (P3b-4).
- Observations across connections are ordered by `(connection, seq)`: `seq` restarts on each (`Seen`).
- The backoff is seeded and injectable, and nothing reads the environment (DEP-44). Only asyncio
  primitives that the selector and the proactor loop both implement are used (P3b-5).
- No invite and no resume secret appears in any `repr`, `str` or message (P3b-6); the SDK never logs.
"""

from __future__ import annotations

import asyncio
import random
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, field
from types import TracebackType
from typing import Self

from mineworld_sdk.errors import ForeignObserver, MineWorldError, SessionClosed
from mineworld_sdk.perceived import CursorAhead, CursorSource, LocalLag, PerceivedStream
from mineworld_sdk.reconnect import (
    AnswerLost,
    NotConnected,
    ReconnectPolicy,
    Retry,
    SeatLossReason,
    SeatLost,
    delays,
    end_outcome,
    join_outcome,
)
from mineworld_sdk.session import (
    TRANSPORT_FAILURES,
    Connection,
    Outcome,
    Perceiving,
    SeatSession,
    open_socket,
)
from mineworld_sdk.wire.contract import ActionRequest
from mineworld_sdk.wire.frames import Invite, ObservationFrame, Perceived, Welcome
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, ResumeSecret, WorldInstanceId

__all__ = [
    "AnswerLost",
    "Connector",
    "NotConnected",
    "ReconnectPolicy",
    "ResumingSeat",
    "SeatLossReason",
    "SeatLost",
    "Seen",
]

type Connector = Callable[[], Awaitable[Connection]]
"""Opens one socket to the server. `ResumingSeat.connect(url)` builds the default; a test gives one that
follows a restarted server to its new port, or aborts a transport."""

type Sleep = Callable[[float], Awaitable[None]]
type WallClock = Callable[[], float]

_DEFAULT_POLICY = ReconnectPolicy()


@dataclass(frozen=True, slots=True)
class Seen:
    """One observation, placed among every connection of the seat (D-P3b-7, step-17 §3.6)."""

    connection: int
    frame: ObservationFrame
    perceived_through: EventId | None
    """The last `through` received on this connection before `frame`, or `None`."""

    def after(self, other: Seen | None) -> bool:
        """Whether this observation comes strictly after `other` in `(connection, seq)` order."""
        if other is None:
            return True
        return (self.connection, self.frame.seq) > (other.connection, other.frame.seq)


@dataclass(slots=True)
class _Intake:
    """Holds a connection's perceived frames until its welcome is accepted, so that a world that is not
    this seat's (row 10) or a foreign observer (row 11) never reaches the consumer."""

    accept: Callable[[Perceived], None]
    held: list[Perceived] = field(default_factory=list[Perceived])
    open: bool = False

    def __call__(self, frame: Perceived) -> None:
        if self.open:
            self.accept(frame)
        else:
            self.held.append(frame)

    def admit(self) -> None:
        self.open = True
        held, self.held = self.held, []
        for frame in held:
            self.accept(frame)


class ResumingSeat:
    """One seat, across as many connections as the network needs. Use `connect` (or `open` with a
    `Connector`) as an async context manager; `perceived()` once for the reliable stream."""

    def __init__(
        self,
        connector: Connector,
        *,
        seat: EntityKey,
        invite: Invite,
        nickname: str,
        cursor: CursorSource | None,
        policy: ReconnectPolicy,
        sleep: Sleep,
        clock: WallClock,
    ) -> None:
        self._connector = connector
        self._seat = seat
        self._invite = invite
        self._nickname = nickname
        self._policy = policy
        self._sleep = sleep
        self._clock = clock
        self._rng = random.Random(policy.seed)
        self._stream = None if cursor is None else PerceivedStream(cursor, policy.perceived_buffer)
        self._stream_taken = False
        self._session: SeatSession | None = None
        self._candidate: SeatSession | None = None
        self._welcome: Welcome | None = None
        self._connection = 0
        self._verdict: asyncio.Future[MineWorldError] | None = None
        self._end: BaseException | None = None
        self._stopping = False
        self._progress = asyncio.Event()
        self._supervisor: asyncio.Task[None] | None = None

    # ── Joining ───────────────────────────────────────────────────────────────────────────────────

    @classmethod
    async def connect(
        cls,
        url: str,
        *,
        seat: EntityKey,
        invite: Invite,
        nickname: str,
        cursor: CursorSource | None,
        take_over: bool = False,
        resume: ResumeSecret | None = None,
        policy: ReconnectPolicy = _DEFAULT_POLICY,
    ) -> ResumingSeat:
        """Joins `seat` of the server at `url` (`ws://host:port/ws`) and keeps it joined."""
        return await cls.open(
            lambda: open_socket(url),
            seat=seat,
            invite=invite,
            nickname=nickname,
            cursor=cursor,
            take_over=take_over,
            resume=resume,
            policy=policy,
        )

    @classmethod
    async def open(
        cls,
        connector: Connector,
        *,
        seat: EntityKey,
        invite: Invite,
        nickname: str,
        cursor: CursorSource | None,
        take_over: bool = False,
        resume: ResumeSecret | None = None,
        policy: ReconnectPolicy = _DEFAULT_POLICY,
        sleep: Sleep | None = None,
        clock: WallClock | None = None,
    ) -> ResumingSeat:
        """Returns after the first welcome. `take_over` and `resume` apply to this first join only. A
        terminal answer raises `SeatLost`; a socket that cannot be opened raises what opening raised; a
        refused `resume` is retried at once without it (row 7). `sleep` and `clock` default to
        `asyncio.sleep` and the running loop's `time` (D-P3b-9)."""
        made = cls(
            connector,
            seat=seat,
            invite=invite,
            nickname=nickname,
            cursor=cursor,
            policy=policy,
            sleep=sleep or asyncio.sleep,
            clock=clock or asyncio.get_running_loop().time,
        )
        presented = resume
        for _ in range(3):
            try:
                await made._attempt(resume=presented, take_over=take_over)
                break
            except SeatLost as lost:
                made._finish(lost)
                raise
            except Retry as retry:
                if retry.resume_refused:
                    presented = None
                elif retry.__cause__ is not None:
                    raise retry.__cause__ from None
        else:
            lost = SeatLost("gave_up")
            made._finish(lost)
            raise lost
        made._supervisor = asyncio.create_task(made._supervise())
        return made

    # ── What the seat knows ───────────────────────────────────────────────────────────────────────

    @property
    def welcome(self) -> Welcome:
        """The current connection's `welcome` (the last one, between connections)."""
        if self._welcome is None:
            raise RuntimeError("a seat has a welcome once open() has returned")
        return self._welcome

    @property
    def instance(self) -> WorldInstanceId:
        """Which world this seat is in, fixed by the first welcome."""
        return self.welcome.world.instance

    @property
    def observer(self) -> EntityId:
        return self.welcome.observer

    @property
    def connection(self) -> int:
        """1 for the first connection, +1 per rejoin."""
        return self._connection

    async def changed(self, after: Seen | None = None) -> Seen:
        """The newest observation strictly after `after` in `(connection, seq)` order, waiting for one.
        Raises what ended the seat: `SeatLost`, or `SessionClosed` after `leave` / `detach`."""
        while True:
            progress = self._progress
            if self._end is not None:
                raise self._end
            session = self._session
            if session is None or session.ended is not None:
                await progress.wait()
                continue
            number = self._connection
            since = after.frame.seq if after is not None and after.connection == number else None
            if await _newer_on(session, since, progress) and session.newest is not None:
                seen = Seen(number, session.newest, session.newest_through)
                if seen.after(after):
                    return seen

    def perceived(self) -> PerceivedStream:
        """The reliable stream of this seat's facts, for its single consumer. Once only."""
        if self._stream is None:
            raise RuntimeError("opened with cursor=None: this seat asked for no perceived stream")
        if self._stream_taken:
            raise RuntimeError("the perceived stream has one consumer, and it was already taken")
        self._stream_taken = True
        return self._stream

    # ── Acting ────────────────────────────────────────────────────────────────────────────────────

    async def submit(self, request: ActionRequest) -> Outcome:
        """Sends `request` on the current connection, once. Raises `NotConnected` between connections,
        `AnswerLost` if its connection ends before the answer, or what ended the seat."""
        if self._end is not None:
            raise self._end
        session, verdict = self._session, self._verdict
        if session is None or verdict is None or session.ended is not None:
            raise NotConnected()
        try:
            return await session.submit(request)
        except (MineWorldError, *TRANSPORT_FAILURES):
            raise await asyncio.shield(verdict) from None

    async def leave(self) -> None:
        """Gives the seat back (`closing { left }`) and ends the seat; the stream ends after what it
        accepted. Between connections there is no socket to say it on: the hold simply runs out."""
        await self._stop(SessionClosed("left"), give_back=True)

    async def detach(self) -> None:
        """Closes without `leave`: the server holds the seat, and `welcome.resume` re-takes it."""
        await self._stop(SessionClosed(None), give_back=False)

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(
        self,
        kind: type[BaseException] | None,
        error: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        await self.leave()

    def __repr__(self) -> str:
        state = "open" if self._end is None else type(self._end).__name__
        return f"ResumingSeat(seat={self._seat!r}, connection={self._connection}, {state})"

    # ── Connections ───────────────────────────────────────────────────────────────────────────────

    async def _attempt(self, *, resume: ResumeSecret | None, take_over: bool) -> None:
        """One join, answered by rows 6 to 11; on a welcome, the connection is installed."""
        perceiving, intake = None, None
        if self._stream is not None:
            stream, number = self._stream, self._connection + 1
            intake = _Intake(lambda frame: stream.accept(frame, number))
            try:
                perceiving = Perceiving(stream.since(), intake)
            except CursorAhead as ahead:
                raise SeatLost("protocol_violation", ahead) from None
        try:
            connection = await self._connector()
        except TRANSPORT_FAILURES as failure:
            raise Retry() from failure
        try:
            session = await SeatSession.join(
                connection,
                seat=self._seat,
                invite=self._invite,
                nickname=self._nickname,
                take_over=take_over,
                resume=resume,
                perceiving=perceiving,
            )
        except BaseException as failure:
            await _close_quietly(connection)
            mapped = join_outcome(failure)
            if mapped is failure:
                raise
            raise mapped from failure
        self._candidate = session
        try:
            await self._accept_welcome(session)
        finally:
            self._candidate = None
        self._install(session)
        if intake is not None:
            try:
                intake.admit()
            except LocalLag:
                # Row 5 at the join itself: the connection ends without `leave`, and the supervisor
                # rejoins with this welcome's `resume`, as for any other local lag.
                await session.close()

    async def _accept_welcome(self, session: SeatSession) -> None:
        """Rows 10 and 11 on a rejoin: another world instance, or another observer, ends the seat."""
        first = self._welcome
        if first is None:
            return
        if session.welcome.world.instance != first.world.instance:
            try:
                await session.leave()
            except (MineWorldError, *TRANSPORT_FAILURES):
                pass
            raise SeatLost("world_changed")
        if session.observer != first.observer:
            await session.close()
            raise SeatLost("protocol_violation", ForeignObserver(first.observer, session.observer))

    def _install(self, session: SeatSession) -> None:
        self._connection += 1
        self._session = session
        self._welcome = session.welcome
        self._verdict = asyncio.get_running_loop().create_future()
        self._bump()

    async def _supervise(self) -> None:
        try:
            while (session := self._session) is not None:
                ended = await session.wait_closed()
                if self._stopping:
                    return
                lost = end_outcome(ended)
                welcome, loss_at = self.welcome, self._clock()
                self._session = None
                # Rows 4-7 and 11 fail a pending submit AnswerLost; row 3 with the SeatLost itself.
                self._settle(AnswerLost() if lost is None or lost.cause else lost)
                if lost is not None:
                    self._finish(lost)
                    return
                if self._stream is not None:
                    self._stream.discard()
                self._bump()
                await self._rejoin(loss_at, welcome)
        except SeatLost as lost:
            self._finish(lost)
        except asyncio.CancelledError:
            raise
        except Exception as unexpected:  # D-P3b-5: unknown errors reach the caller unchanged
            self._finish(unexpected)

    async def _rejoin(self, loss_at: float, welcome: Welcome) -> None:
        """Rows 4 to 7 and 12: wait the next delay, join with `resume` while the hold lasts (D-P3b-6)."""
        waits = delays(self._policy, self._rng)
        resume = welcome.resume if welcome.hold_seconds > 0 else None
        hold_ends = loss_at + welcome.hold_seconds
        at_once = False
        while True:
            if not at_once:
                delay = next(waits)
                if self._clock() - loss_at + delay > self._policy.give_up_after:
                    raise SeatLost("gave_up")
                await self._sleep(delay)
            at_once = False
            presented = resume if resume is not None and self._clock() < hold_ends else None
            try:
                await self._attempt(resume=presented, take_over=False)
                return
            except Retry as retry:
                if retry.resume_refused:
                    resume, at_once = None, True

    # ── Ending ────────────────────────────────────────────────────────────────────────────────────

    async def _stop(self, end: SessionClosed, *, give_back: bool) -> None:
        if self._end is not None:
            return
        self._stopping = True
        supervisor = self._supervisor
        if supervisor is not None and not supervisor.done():
            supervisor.cancel()
            await asyncio.gather(supervisor, return_exceptions=True)
        for session in (self._candidate, self._session):
            if session is None or session.ended is not None:
                continue
            if give_back:
                try:
                    await session.leave()
                except (MineWorldError, *TRANSPORT_FAILURES):
                    pass
            await session.close()
        self._finish(end)

    def _settle(self, error: MineWorldError) -> None:
        """Gives every submit pending on the connection that just ended the error the table names."""
        verdict, self._verdict = self._verdict, None
        if verdict is not None and not verdict.done():
            verdict.set_result(error)

    def _finish(self, end: BaseException) -> None:
        if self._end is not None:
            return
        self._end = end
        self._session = None
        self._settle(end if isinstance(end, MineWorldError) else AnswerLost())
        if self._stream is not None:
            self._stream.finish(None if isinstance(end, SessionClosed) else end)
        self._bump()

    def _bump(self) -> None:
        progress, self._progress = self._progress, asyncio.Event()
        progress.set()


async def _newer_on(session: SeatSession, since: int | None, progress: asyncio.Event) -> bool:
    """Waits for an observation newer than `since` on `session`, or for the seat to move on. Returns
    whether the session has one. Both waiters are always finished before returning: no task leaks."""
    arrival = asyncio.ensure_future(session.changed(since))
    moved = asyncio.ensure_future(progress.wait())
    try:
        await asyncio.wait((arrival, moved), return_when=asyncio.FIRST_COMPLETED)
    finally:
        for waiter in (arrival, moved):
            if not waiter.done():
                waiter.cancel()
        await asyncio.gather(arrival, moved, return_exceptions=True)
    return arrival.done() and not arrival.cancelled() and arrival.exception() is None


async def _close_quietly(connection: Connection) -> None:
    try:
        await connection.close()
    except (MineWorldError, *TRANSPORT_FAILURES):
        pass
