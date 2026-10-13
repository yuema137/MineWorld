"""A Python client of the MineWorld protocol, revision 2 (`server/PROTOCOL.md`).

A program joins one seat of a running server, reads typed frames, and submits typed requests, with
no more privilege than any other client has. `SeatSession` is one connection; `ResumingSeat` keeps a
seat across dropped sockets, `lagged` and server restarts, and delivers the reliable `perceived` stream
from the caller's own cursor.
"""

from mineworld_sdk import offers
from mineworld_sdk.errors import MalformedFrame, MineWorldError, UnsupportedFrame
from mineworld_sdk.offers import NotOffered
from mineworld_sdk.perceived import (
    CursorAhead,
    CursorCell,
    CursorSource,
    LocalLag,
    PerceivedBatch,
    PerceivedStream,
)
from mineworld_sdk.resuming import (
    AnswerLost,
    Connector,
    NotConnected,
    ReconnectPolicy,
    ResumingSeat,
    SeatLossReason,
    SeatLost,
    Seen,
)
from mineworld_sdk.session import (
    Answered,
    ForeignObserver,
    Invite,
    JoinRefused,
    Outcome,
    Perceiving,
    ProtocolMismatch,
    ProtocolViolation,
    RefusedRequest,
    SeatSession,
    SessionClosed,
)

__all__ = [
    "AnswerLost",
    "Answered",
    "Connector",
    "CursorAhead",
    "CursorCell",
    "CursorSource",
    "ForeignObserver",
    "Invite",
    "JoinRefused",
    "LocalLag",
    "MalformedFrame",
    "MineWorldError",
    "NotConnected",
    "NotOffered",
    "Outcome",
    "PerceivedBatch",
    "PerceivedStream",
    "Perceiving",
    "ProtocolMismatch",
    "ProtocolViolation",
    "ReconnectPolicy",
    "RefusedRequest",
    "ResumingSeat",
    "SeatLossReason",
    "SeatLost",
    "SeatSession",
    "Seen",
    "SessionClosed",
    "UnsupportedFrame",
    "offers",
]
