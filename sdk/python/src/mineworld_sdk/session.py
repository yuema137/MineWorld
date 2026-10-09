"""One seat of a running server, driven from Python with no privilege a client lacks.

The session speaks `server/PROTOCOL.md` §4's sequence: `join`, then `welcome`, then the observation
stream, `submit` paired with `result` by the session's own token, and `leave` answered by `closing`.

- **Newest wins** (§8). The session keeps only the newest observation. It never queues them, because a
  controller acts on what the world looks like now, and the server drops frames for a slow reader
  anyway.
- **One observer** (`INV-13`). Every observation must name the observer `welcome` named. A frame for
  anybody else ends the session with `ForeignObserver`; the session never shows it to its caller.
- **No retry and no reconnect.** A dropped socket ends the session with `SessionClosed(None)`.
  Reconnecting belongs with `resume` (P3b).

The session never reads the environment, never logs, and never puts the invite in a message or a
`repr` (D-P3-9).
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from types import TracebackType
from typing import Protocol, Self

from websockets.asyncio.client import connect as websocket_connect
from websockets.exceptions import ConnectionClosed

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.wire import codec
from mineworld_sdk.wire.contract import ActionRequest, ActionResult
from mineworld_sdk.wire.frames import (
    PROTOCOL_VERSION,
    Closing,
    ClosingReason,
    Invite,
    Join,
    Leave,
    ObservationFrame,
    RefusalCode,
    Refused,
    Result,
    ServerFrame,
    Submit,
    Welcome,
)
from mineworld_sdk.wire.ids import ActionId, CorrelationToken, EntityId, EntityKey

__all__ = [
    "Answered",
    "Connection",
    "ForeignObserver",
    "Invite",
    "JoinRefused",
    "Outcome",
    "ProtocolMismatch",
    "ProtocolViolation",
    "RefusedRequest",
    "SeatSession",
    "SessionClosed",
]

JOIN_PATIENCE = 10.0
"""Seconds to wait for the answer to a `join`. A wrong invite is answered after 500 ms (§4.1)."""

LEAVE_PATIENCE = 5.0
"""Seconds to wait for `closing { left }` after a `leave`. The server answers at once (§2)."""

MAX_FRAME_BYTES = 16 * 1024 * 1024
"""The largest frame accepted: well above any observation of the sample worlds."""


class JoinRefused(MineWorldError):
    """The server did not grant the seat. `code` is the refusal code or the closing reason."""

    def __init__(self, code: RefusalCode | ClosingReason) -> None:
        super().__init__(f"the server refused the join: {code}")
        self.code: RefusalCode | ClosingReason = code


class ProtocolMismatch(MineWorldError):
    """The server speaks a protocol revision this SDK does not (`PROTOCOL.md` §10)."""

    def __init__(self, protocol: int) -> None:
        super().__init__(
            f"the server speaks protocol {protocol}; this SDK speaks {PROTOCOL_VERSION}"
        )
        self.protocol = protocol


class ForeignObserver(MineWorldError):
    """An observation named another observer than this connection's (`INV-13`)."""

    def __init__(self, expected: EntityId, received: EntityId) -> None:
        super().__init__(
            f"an observation for observer {received} arrived on the connection of observer {expected}"
        )
        self.expected = expected
        self.received = received


class ProtocolViolation(MineWorldError):
    """The server sent a frame the sequence does not allow: an answer to no request, a second
    `welcome`, a binary frame, or a refusal that names no request."""


class SessionClosed(MineWorldError):
    """The session ended. `reason` is the server's `closing` reason, or `None` when the socket dropped
    without one."""

    def __init__(self, reason: ClosingReason | None) -> None:
        super().__init__(
            f"the session is closed: {reason}"
            if reason
            else "the connection dropped without closing"
        )
        self.reason: ClosingReason | None = reason


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


class SeatSession:
    """One seat of a running server. Use `connect`, as an async context manager:

    ```python
    async with await SeatSession.connect(url, seat=..., invite=..., nickname=...) as session:
        seen = await session.changed()
        outcome = await session.submit(request)
    ```
    """

    def __init__(self, connection: Connection, welcome: Welcome) -> None:
        self._connection = connection
        self._welcome = welcome
        self._newest: ObservationFrame | None = None
        self._arrived = asyncio.Event()
        self._pending: dict[CorrelationToken, asyncio.Future[Outcome]] = {}
        self._issued = 0
        self._sent = 1  # the join
        self._ended: MineWorldError | None = None
        self._left: asyncio.Future[ClosingReason] | None = None
        self._reader: asyncio.Task[None] | None = None

    # ── Joining ───────────────────────────────────────────────────────────────────────────────────

    @classmethod
    async def connect(
        cls, url: str, *, seat: EntityKey, invite: Invite, nickname: str
    ) -> SeatSession:
        """Opens a WebSocket to `url` (`ws://host:port/ws`) and joins `seat`."""
        connection = await websocket_connect(
            url, compression=None, max_size=MAX_FRAME_BYTES, open_timeout=JOIN_PATIENCE
        )
        try:
            return await cls.join(connection, seat=seat, invite=invite, nickname=nickname)
        except BaseException:
            await connection.close()
            raise

    @classmethod
    async def join(
        cls, connection: Connection, *, seat: EntityKey, invite: Invite, nickname: str
    ) -> SeatSession:
        """Joins `seat` over an open connection and starts reading. Raises `JoinRefused`,
        `ProtocolMismatch` or `ProtocolViolation`, after closing the connection."""
        await connection.send(codec.encode(Join(invite=invite, nickname=nickname, seat=seat)))
        answer = await asyncio.wait_for(_receive(connection), JOIN_PATIENCE)
        if isinstance(answer, Welcome):
            if answer.protocol != PROTOCOL_VERSION:
                await connection.close()
                raise ProtocolMismatch(answer.protocol)
            session = cls(connection, answer)
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
            self._end(SessionClosed(None))
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
                    self._arrived.set()
            case Result():
                self._resolve(frame.token, Answered(frame.action_id, frame.result))
            case Refused() if frame.token is not None:
                self._resolve(frame.token, RefusedRequest(frame.code, frame.detail))
            case Refused():
                self._end(ProtocolViolation(f"a refusal naming no request: {frame.code}"))
            case Closing():
                if self._left is not None and not self._left.done():
                    self._left.set_result(frame.reason)
                self._end(SessionClosed(frame.reason))
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
