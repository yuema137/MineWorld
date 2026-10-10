"""One seat of a running server, driven from Python with no privilege a client lacks.

The session speaks `server/PROTOCOL.md` §4's sequence: `join`, then `welcome`, then the observation
stream, `submit` paired with `result` by the session's own token, and `leave` answered by `closing`.

- **Newest wins** (§8). The session keeps only the newest observation. It never queues them, because a
  controller acts on what the world looks like now, and the server drops frames for a slow reader
  anyway. A `delta` frame (§5.3) is applied to the observation held, so `newest` is always whole; one
  that does not apply ends the session with `ProtocolViolation` (reconnecting is `ResumingSeat`'s).
- **Facts handed on, in order** (§5.8). With `perceiving` on `connect`, each frame of the reliable
  stream is checked against the connection's order (`perceived.ConnectionOrder`) and handed to the
  caller's sink, on the reader task, in frame order. The session keeps no fact and advances no cursor
  of the caller's: what was received is not what was ingested (P4's R-P3b-1).
- **One observer** (`INV-13`). Every observation must name the observer `welcome` named. A frame for
  anybody else ends the session with `ForeignObserver`; the session never shows it to its caller.
- **No retry and no reconnect.** A dropped socket ends the session with `SessionClosed(None)`; the
  server's `lagged` sequence (`refused { lagged }`, which names no request, then `closing { lagged }`)
  ends it with `SessionClosed("lagged")`. Reconnecting is `resuming.ResumingSeat`'s, which composes one
  session per connection.

The session never reads the environment, never logs, and never puts the invite in a message or a
`repr` (D-P3-9).
"""

from __future__ import annotations

import asyncio
from collections.abc import Callable
from dataclasses import dataclass
from types import TracebackType
from typing import Protocol, Self

from websockets.asyncio.client import connect as websocket_connect
from websockets.exceptions import ConnectionClosed, InvalidHandshake

from mineworld_sdk.errors import (
    ForeignObserver,
    JoinRefused,
    MineWorldError,
    ProtocolMismatch,
    ProtocolViolation,
    SessionClosed,
)
from mineworld_sdk.perceived import ConnectionOrder
from mineworld_sdk.wire import codec
from mineworld_sdk.wire.contract import ActionRequest, ActionResult
from mineworld_sdk.wire.delta import DeltaMismatch, apply_delta
from mineworld_sdk.wire.frames import (
    PROTOCOL_VERSION,
    Clock,
    Closing,
    ClosingReason,
    Delta,
    Invite,
    Join,
    Leave,
    ObservationFrame,
    Perceived,
    PerceivedJoin,
    RefusalCode,
    Refused,
    Result,
    ServerFrame,
    Submit,
    Welcome,
)
from mineworld_sdk.wire.ids import (
    ActionId,
    CorrelationToken,
    EntityId,
    EntityKey,
    EventId,
    ResumeSecret,
)

__all__ = [
    "TRANSPORT_FAILURES",
    "Answered",
    "Connection",
    "ForeignObserver",
    "Invite",
    "JoinRefused",
    "Outcome",
    "Perceiving",
    "ProtocolMismatch",
    "ProtocolViolation",
    "RefusedRequest",
    "SeatSession",
    "SessionClosed",
    "open_socket",
]

JOIN_PATIENCE = 10.0
"""Seconds to wait for the answer to a `join`. A wrong invite is answered after 500 ms (§4.1)."""

LEAVE_PATIENCE = 5.0
"""Seconds to wait for `closing { left }` after a `leave`. The server answers at once (§2)."""

MAX_FRAME_BYTES = 16 * 1024 * 1024
"""The largest frame accepted: well above any observation of the sample worlds."""

TRANSPORT_FAILURES = (OSError, TimeoutError, InvalidHandshake, ConnectionClosed)
"""What a socket that could not be opened, or that ended under a join, raises. A caller that reconnects
retries these and nothing else (D-P3b-5); named here so that only this module imports `websockets`."""


@dataclass(frozen=True, slots=True)
class Answered:
    """The world answered the request: `action_id` is the identity the server allocated for it, and
    `result` is accepted, rejected or unavailable."""

    action_id: ActionId
    result: ActionResult


@dataclass(frozen=True, slots=True)
class RefusedRequest:
    """The frame carrying the request was refused (`PROTOCOL.md` §5.5): it was not a request the world
    considered at all, as distinct from a rejection inside an answer."""

    code: RefusalCode
    detail: str | None


type Outcome = Answered | RefusedRequest
"""What a submitted request came to."""


class Connection(Protocol):
    """The text-frame socket a session runs over. `websockets`' client connection is one."""

    async def send(self, message: str, /) -> None: ...

    async def recv(self) -> str | bytes: ...

    async def close(self) -> None: ...


@dataclass(frozen=True, slots=True)
class Perceiving:
    """A `join`'s request for the reliable `perceived` stream, and where its frames go.

    `since` is the cursor to serve from (`PROTOCOL.md` §5.8): the `through` the caller last *ingested*,
    or `None` for this world's first fact. `deliver` receives every frame that passed the connection's
    order checks, on the session's reader task, one at a time, in frame order. An error it raises that is
    a `MineWorldError` ends the session with that error, and the socket is closed without `leave`."""

    since: EventId | None
    deliver: Callable[[Perceived], None]


async def open_socket(url: str) -> Connection:
    """Opens a WebSocket to `url` (`ws://host:port/ws`) as every session of this SDK does: no
    compression, `MAX_FRAME_BYTES`, `JOIN_PATIENCE` to open, `websockets`' keepalive left on."""
    return await websocket_connect(
        url, compression=None, max_size=MAX_FRAME_BYTES, open_timeout=JOIN_PATIENCE
    )


class SeatSession:
    """One seat of a running server. Use `connect`, as an async context manager:

    ```python
    async with await SeatSession.connect(url, seat=..., invite=..., nickname=...) as session:
        seen = await session.changed()
        outcome = await session.submit(request)
    ```
    """

    def __init__(
        self,
        connection: Connection,
        welcome: Welcome,
        deliver: Callable[[Perceived], None] | None = None,
    ) -> None:
        self._connection = connection
        self._welcome = welcome
        self._newest: ObservationFrame | None = None
        self._newest_through: EventId | None = None
        self._clock: Clock | None = None
        self._deliver = deliver
        self._order = ConnectionOrder()
        self._lagging = False
        self._arrived = asyncio.Event()
        self._finished = asyncio.Event()
        self._pending: dict[CorrelationToken, asyncio.Future[Outcome]] = {}
        self._issued = 0
        self._sent = 1  # the join
        self._ended: MineWorldError | None = None
        self._left: asyncio.Future[ClosingReason] | None = None
        self._reader: asyncio.Task[None] | None = None

    # ── Joining ───────────────────────────────────────────────────────────────────────────────────

    @classmethod
    async def connect(
        cls,
        url: str,
        *,
        seat: EntityKey,
        invite: Invite,
        nickname: str,
        take_over: bool = False,
        resume: ResumeSecret | None = None,
        perceiving: Perceiving | None = None,
    ) -> SeatSession:
        """Opens a WebSocket to `url` (`ws://host:port/ws`) and joins `seat`; with `perceiving`, also
        asks for the reliable `perceived` stream from its cursor (`PROTOCOL.md` §5.8)."""
        connection = await open_socket(url)
        try:
            return await cls.join(
                connection,
                seat=seat,
                invite=invite,
                nickname=nickname,
                take_over=take_over,
                resume=resume,
                perceiving=perceiving,
            )
        except BaseException:
            await connection.close()
            raise

    @classmethod
    async def join(
        cls,
        connection: Connection,
        *,
        seat: EntityKey,
        invite: Invite,
        nickname: str,
        take_over: bool = False,
        resume: ResumeSecret | None = None,
        perceiving: Perceiving | None = None,
    ) -> SeatSession:
        """Joins `seat` over an open connection and starts reading. Raises `JoinRefused` (for an occupied
        seat, `seat_occupied`, unless `resume` re-takes it or `take_over` takes it, `PROTOCOL.md` §4.2;
        for a cursor the world cannot serve, `cursor_unavailable`, §5.8), `ProtocolMismatch` or
        `ProtocolViolation`, after closing the connection."""
        frame = Join(
            invite=invite,
            nickname=nickname,
            seat=seat,
            resume=resume,
            take_over=take_over,
            perceived=None if perceiving is None else PerceivedJoin(since=perceiving.since),
        )
        await connection.send(codec.encode(frame))
        answer = await asyncio.wait_for(_receive(connection), JOIN_PATIENCE)
        if isinstance(answer, Welcome):
            if answer.protocol != PROTOCOL_VERSION:
                await connection.close()
                raise ProtocolMismatch(answer.protocol)
            session = cls(connection, answer, None if perceiving is None else perceiving.deliver)
            session._reader = asyncio.create_task(session._read())
            return session
        if isinstance(answer, Refused):
            if answer.code in ("protocol_mismatch", "unauthorized"):
                # The server follows these two with `closing`; read it, so the socket is closed by the
                # client after the server's last word (§5.6).
                try:
                    await asyncio.wait_for(_receive(connection), LEAVE_PATIENCE)
                except (MineWorldError, ConnectionClosed, TimeoutError):
                    pass
            await connection.close()
            raise JoinRefused(answer.code)
        await connection.close()
        if isinstance(answer, Closing):
            raise JoinRefused(answer.reason)
        raise ProtocolViolation(f"the answer to a join was a {answer.t} frame")

    # ── What the session knows ────────────────────────────────────────────────────────────────────

    @property
    def welcome(self) -> Welcome:
        """The server's `welcome`: the seat, the observer, and what the world is."""
        return self._welcome

    @property
    def observer(self) -> EntityId:
        """The only identity this connection perceives the world as."""
        return self._welcome.observer

    @property
    def newest(self) -> ObservationFrame | None:
        """The newest observation received, or `None` before the first."""
        return self._newest

    @property
    def clock(self) -> Clock | None:
        """The newest `clock` frame: whether the host has paused the world, and at what scale it runs.
        `None` before the first, which the server sends right after `welcome` (`PROTOCOL.md` §5.9)."""
        return self._clock

    @property
    def newest_through(self) -> EventId | None:
        """The `through` of the last `perceived` frame received on this connection before `newest`
        arrived, or `None` (step-17 §3.6's `based_on`). Read with `newest`: the two change together."""
        return self._newest_through

    @property
    def ended(self) -> MineWorldError | None:
        """What ended the session, or `None` while it is open."""
        return self._ended

    @property
    def sent_frames(self) -> int:
        """How many frames this session has sent, the `join` included."""
        return self._sent

    async def changed(self, since: int | None = None) -> ObservationFrame:
        """The newest observation whose `seq` is greater than `since` (any, when `since` is `None`),
        waiting for one if there is none yet. Raises the error that ended the session, if it ended."""
        while True:
            newest = self._newest
            if newest is not None and (since is None or newest.seq > since):
                return newest
            if self._ended is not None:
                raise self._ended
            self._arrived.clear()
            await self._arrived.wait()

    # ── Acting ────────────────────────────────────────────────────────────────────────────────────

    async def submit(self, request: ActionRequest) -> Outcome:
        """Sends `request` and waits for the answer paired with it by token."""
        if self._ended is not None:
            raise self._ended
        self._issued += 1
        token = CorrelationToken(f"c{self._issued}")
        answer: asyncio.Future[Outcome] = asyncio.get_running_loop().create_future()
        self._pending[token] = answer
        await self._send(Submit(token=token, request=request))
        return await answer

    async def leave(self) -> ClosingReason:
        """Gives the seat up: sends `leave`, waits for `closing`, and closes the socket."""
        if self._ended is not None:
            raise self._ended
        self._left = asyncio.get_running_loop().create_future()
        await self._send(Leave())
        try:
            return await asyncio.wait_for(self._left, LEAVE_PATIENCE)
        finally:
            await self._stop()

    async def close(self) -> None:
        """Closes the socket **without** `leave`: the server holds the seat for `hold_seconds`, and the
        welcome's `resume` re-takes it (`PROTOCOL.md` §4.2). The session ends `SessionClosed(None)`."""
        await self._stop()

    async def wait_closed(self) -> MineWorldError:
        """Waits until the session has ended, and returns what ended it."""
        await self._finished.wait()
        return self._ended or SessionClosed(None)

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(
        self,
        kind: type[BaseException] | None,
        error: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        if self._ended is None:
            try:
                await self.leave()
            except (MineWorldError, ConnectionClosed, TimeoutError):
                pass
        await self._stop()

    def __repr__(self) -> str:
        state = "open" if self._ended is None else repr(self._ended)
        return f"SeatSession(seat={self._welcome.seat!r}, observer={self.observer!r}, {state})"

    # ── Internals ─────────────────────────────────────────────────────────────────────────────────

    async def _send(self, frame: Join | Submit | Leave) -> None:
        await self._connection.send(codec.encode(frame))
        self._sent += 1

    async def _read(self) -> None:
        try:
            while self._ended is None:
                self.route(await _receive(self._connection))
        except ConnectionClosed:
            # A socket that drops between `refused { lagged }` and its `closing` ended for the same
            # reason (design C2's failure cases).
            self._end(SessionClosed("lagged" if self._lagging else None))
        except MineWorldError as error:
            self._end(error)
        finally:
            if self._ended is not None:
                await self._connection.close()

    def route(self, frame: ServerFrame) -> None:
        """Applies one server frame to the session's state. The reader calls this for every frame
        after `welcome`; a test may call it with frames it builds (AP-4)."""
        match frame:
            case ObservationFrame():
                if frame.observation.observer != self.observer:
                    self._end(ForeignObserver(self.observer, frame.observation.observer))
                    return
                if self._newest is None or frame.seq > self._newest.seq:
                    self._newest = frame
                    self._newest_through = self._order.last_through
                    self._arrived.set()
            case Delta():
                held = self._newest
                if held is None or frame.base != held.seq:
                    self._end(
                        ProtocolViolation(f"a delta on base {frame.base}, not the frame held")
                    )
                    return
                try:
                    observation = apply_delta(held.observation, frame.delta)
                except DeltaMismatch as mismatch:
                    self._end(ProtocolViolation(str(mismatch)))
                    return
                self.route(
                    ObservationFrame(
                        t="observation",
                        seq=frame.seq,
                        revision=frame.revision,
                        acted_through=frame.acted_through,
                        observation=observation,
                    )
                )
            case Perceived():
                if self._deliver is None:
                    self._end(ProtocolViolation("a perceived frame on a join that asked for none"))
                    return
                wrong = self._order.admit(frame)
                if wrong is not None:
                    self._end(ProtocolViolation(wrong))
                    return
                self._deliver(frame)
            case Result():
                self._resolve(frame.token, Answered(frame.action_id, frame.result))
            case Refused() if frame.token is not None:
                self._resolve(frame.token, RefusedRequest(frame.code, frame.detail))
            case Refused() if frame.code == "lagged":
                # PROTOCOL.md §5.5, §5.8: names no request, and `closing { lagged }` follows. The client
                # rejoins and loses nothing, so this is no violation (F-P3b-1).
                self._lagging = True
            case Refused():
                self._end(ProtocolViolation(f"a refusal naming no request: {frame.code}"))
            case Closing():
                if self._left is not None and not self._left.done():
                    self._left.set_result(frame.reason)
                self._end(SessionClosed(frame.reason))
            case Clock():
                # The host's pacing (PROTOCOL.md §5.9): kept, newest wins, like an observation.
                self._clock = frame
            case Welcome():
                self._end(ProtocolViolation("a second welcome on one connection"))

    def _resolve(self, token: CorrelationToken, outcome: Outcome) -> None:
        answer = self._pending.pop(token, None)
        if answer is None:
            self._end(ProtocolViolation(f"an answer for token {token!r}, which no request carried"))
        elif not answer.done():
            answer.set_result(outcome)

    def _end(self, error: MineWorldError) -> None:
        if self._ended is not None:
            return
        self._ended = error
        for answer in self._pending.values():
            if not answer.done():
                answer.set_exception(error)
        self._pending.clear()
        if self._left is not None and not self._left.done():
            self._left.set_exception(error)
        self._arrived.set()
        self._finished.set()

    async def _stop(self) -> None:
        self._end(SessionClosed(None))
        reader = self._reader
        if reader is not None and not reader.done() and reader is not asyncio.current_task():
            reader.cancel()
            try:
                await reader
            except asyncio.CancelledError:
                pass
        await self._connection.close()


async def _receive(connection: Connection) -> ServerFrame:
    message = await connection.recv()
    if isinstance(message, bytes):
        raise ProtocolViolation("a binary frame; this protocol sends JSON text only")
    return codec.decode(message)
