"""The reliable `perceived` stream (`server/PROTOCOL.md` §5.8), as a consumer that must not miss a fact
receives it.

`ConnectionOrder` holds one connection to the stream's promises — ascending, never repeated, a cursor
that never goes back — before any frame reaches a consumer (design §4.4's per-connection checks). A
stream that breaks them is the server's defect, and the client fails closed rather than ingest it.
"""

from __future__ import annotations

from itertools import pairwise

from mineworld_sdk.wire.frames import Perceived
from mineworld_sdk.wire.ids import EventId, event_order

__all__ = ["ConnectionOrder"]


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
