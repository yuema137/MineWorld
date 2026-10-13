"""The SDK's errors. Every one derives from `MineWorldError`, so a caller can catch the SDK's failures
without catching a library's.

No error message ever contains the invite or a resume secret (D-P3-9, P3b-6): the SDK never formats one
into a message, and the server never sends one back.
"""

from __future__ import annotations

from mineworld_sdk.wire.frames import PROTOCOL_VERSION, ClosingReason, RefusalCode
from mineworld_sdk.wire.ids import EntityId


class MineWorldError(Exception):
    """Something the SDK refused or could not do."""


class MalformedFrame(MineWorldError):
    """A frame from the server is not a revision-2 frame this SDK models: unknown kind, unknown field,
    wrong type, or a value the contract forbids. The server and this SDK disagree, which is a defect on
    one side (`server/PROTOCOL.md` §2's "unknown fields are refused", from the client's side)."""


class UnsupportedFrame(MineWorldError):
    """A frame carries a field revision 2 specifies but this SDK does not model yet (D-P3-6)."""


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
    `welcome`, a binary frame, a refusal that names no request (other than `lagged`), or a `perceived`
    frame out of order (`perceived.ConnectionOrder`)."""


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
