"""The reliable `perceived` stream (`server/PROTOCOL.md` §5.8), as a consumer that must not miss a fact
receives it.

`ConnectionOrder` holds one connection to the stream's promises — ascending, never repeated, a cursor
that never goes back — before any frame reaches a consumer (design §4.4's per-connection checks). A
stream that breaks them is the server's defect, and the client fails closed rather than ingest it.

`PerceivedStream` is the seat's side, across connections (design §4.4, D-P3b-2 … D-P3b-4):

- **The caller's store is the cursor's only authority.** Every join presents what `CursorSource.cursor()`
  reports — what the consumer durably ingested — never what merely arrived (P4's R-P3b-1).
- **At most once per process.** The stream remembers `delivered`, the `through` of the last batch the
  consumer took; a rejoin that re-sends facts at or below it has them removed, so the consumer never
  sees one twice even though the store's cursor may lag behind what it was handed.
- **Bounded, and overflowing as the server does.** Batches the consumer has not taken wait in a buffer
  of at most `bound` facts. Past it, every waiting batch is discarded and `LocalLag` ends the
  connection: the seat rejoins from the store's cursor and the facts come again. The price of not
  reading is a reconnect, never a gap (`PROTOCOL.md` §5.8).
"""

from __future__ import annotations

import asyncio
from collections import deque
from dataclasses import dataclass
from itertools import pairwise
from typing import Protocol, Self

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.frames import Perceived
from mineworld_sdk.wire.ids import EventId, event_order

__all__ = [
    "ConnectionOrder",
    "CursorAhead",
    "CursorCell",
    "CursorSource",
    "LocalLag",
    "PerceivedBatch",
    "PerceivedStream",
]


@dataclass(frozen=True, slots=True)
class PerceivedBatch:
    """Facts a seat delivered, in strictly ascending `EventId`, every one after the previous batch's
    `through`. Store `through` as the cursor once `events` are durably ingested — not before."""

    events: tuple[PerceivedEvent, ...]
    through: EventId
    connection: int
    """Which connection of the seat delivered it: 1, then +1 per rejoin."""


class CursorSource(Protocol):
    """Where a seat reads the cursor it presents at every join: the id of the last perceived fact the
    caller durably ingested, or `None` for none. P4's `MemoryStore` is one."""

    def cursor(self) -> EventId | None: ...


class CursorCell:
    """An in-memory `CursorSource` for a caller without a durable store. It moves only on `commit`."""

    __slots__ = ("_cursor",)

    def __init__(self, start: EventId | None = None) -> None:
        self._cursor = start

    def cursor(self) -> EventId | None:
        return self._cursor

    def commit(self, batch: PerceivedBatch) -> None:
        """Records `batch.through` as ingested. A cursor never goes back: a lower one is refused."""
        if self._cursor is not None and event_order(batch.through) < event_order(self._cursor):
            raise ValueError(f"a cursor never goes back: {batch.through} is below {self._cursor}")
        self._cursor = batch.through

    def __repr__(self) -> str:
        return f"CursorCell({self._cursor!r})"


class CursorAhead(MineWorldError):
    """The cursor source reports a fact the seat never delivered: it claims to have ingested what it was
    not given (design D-P3b-3). Presenting it would skip facts, so the seat ends instead."""

    def __init__(self, cursor: EventId, delivered: EventId | None) -> None:
        super().__init__(f"the cursor source is at {cursor}, past the last delivered {delivered}")
        self.cursor = cursor
        self.delivered = delivered


class LocalLag(MineWorldError):
    """The consumer left more facts waiting than the seat's bound (design D-P3b-4): the connection ends
    without `leave`, and the seat rejoins from the store's cursor."""

    def __init__(self, bound: int) -> None:
        super().__init__(f"more than {bound} perceived facts waited for the consumer")
        self.bound = bound


class ConnectionOrder:
    """The order checks of one connection's `perceived` frames (design §4.4, checks 1 to 3)."""

    __slots__ = ("_through",)

    def __init__(self) -> None:
        self._through: EventId | None = None

    @property
    def last_through(self) -> EventId | None:
        """The `through` of the last frame admitted on this connection, or `None` before the first."""
        return self._through

    def admit(self, frame: Perceived) -> str | None:
        """Admits `frame` and returns `None`, or returns what is wrong with it and admits nothing."""
        keys = [event_order(event.id) for event in frame.events]
        if any(later <= earlier for earlier, later in pairwise(keys)):
            return "the facts of a perceived frame are not in strictly ascending order"
        through = event_order(frame.through)
        previous = None if self._through is None else event_order(self._through)
        if keys and previous is not None and keys[0] <= previous:
            return (
                f"a perceived frame repeats a fact at or before the cursor {self._through}: "
                f"{frame.events[0].id}"
            )
        if keys and through < keys[-1]:
            return f"a perceived frame's through {frame.through} is below its own last fact"
        if previous is not None and through < previous:
            return (
                f"a perceived frame's through {frame.through} is below the previous {self._through}"
            )
        self._through = frame.through
        return None


class PerceivedStream:
    """The seat's perceived facts for one consumer: `async for batch in seat.perceived(): …`.

    It yields every batch already accepted, in order, then ends with `StopAsyncIteration` after the seat
    left or detached, or raises the `SeatLost` that ended the seat. The seat drives it with `accept`,
    `discard`, `since` and `finish`; a consumer only iterates.
    """

    def __init__(self, source: CursorSource, bound: int) -> None:
        self._source = source
        self._bound = bound
        self._waiting: deque[PerceivedBatch] = deque()
        self._waiting_facts = 0
        self._delivered: EventId | None = None
        self._high: EventId | None = None
        self._end: BaseException | None = None
        self._finished = False
        self._ready = asyncio.Event()

    def __aiter__(self) -> Self:
        return self

    async def __anext__(self) -> PerceivedBatch:
        while not self._waiting:
            if self._finished:
                if self._end is not None:
                    raise self._end
                raise StopAsyncIteration
            self._ready.clear()
            await self._ready.wait()
        batch = self._waiting.popleft()
        self._waiting_facts -= len(batch.events)
        self._delivered = batch.through
        return batch

    @property
    def delivered(self) -> EventId | None:
        """The `through` of the last batch the consumer took, or `None` before the first."""
        return self._delivered

    def since(self) -> EventId | None:
        """The cursor a join presents: the source's, checked against what was delivered."""
        cursor = self._source.cursor()
        delivered = self._delivered
        if (
            cursor is not None
            and delivered is not None
            and event_order(cursor) > event_order(delivered)
        ):
            raise CursorAhead(cursor, delivered)
        return cursor

    def accept(self, frame: Perceived, connection: int) -> None:
        """Takes one frame a connection admitted. Raises `LocalLag`, having discarded every waiting
        batch, when the consumer has left more than the bound waiting."""
        high = self._high
        keep = tuple(
            event
            for event in frame.events
            if high is None or event_order(event.id) > event_order(high)
        )
        if not keep and high is not None and event_order(frame.through) <= event_order(high):
            return
        self._waiting.append(PerceivedBatch(keep, frame.through, connection))
        self._waiting_facts += len(keep)
        self._high = frame.through
        self._ready.set()
        if self._waiting_facts > self._bound:
            self.discard()
            raise LocalLag(self._bound)

    def discard(self) -> None:
        """Drops every batch the consumer has not taken: a rejoin from the store's cursor sends them
        again, and nothing is lost."""
        self._waiting.clear()
        self._waiting_facts = 0
        self._high = self._delivered

    def finish(self, end: BaseException | None) -> None:
        """Ends the stream after the batches already waiting: `None` for a leave or a detach, else the
        error the consumer receives once it has taken them."""
        if self._finished:
            return
        self._finished = True
        self._end = end
        self._ready.set()
