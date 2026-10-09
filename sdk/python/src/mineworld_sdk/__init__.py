"""A Python client of the MineWorld protocol, revision 2 (`server/PROTOCOL.md`).

A program joins one seat of a running server, reads typed frames, and submits typed requests, with
no more privilege than any other client has.
"""

from mineworld_sdk import offers
from mineworld_sdk.errors import MalformedFrame, MineWorldError, UnsupportedFrame
from mineworld_sdk.offers import NotOffered
from mineworld_sdk.session import (
    Answered,
    ForeignObserver,
    Invite,
    JoinRefused,
    Outcome,
    ProtocolMismatch,
    ProtocolViolation,
    RefusedRequest,
    SeatSession,
    SessionClosed,
)

__all__ = [
    "Answered",
    "ForeignObserver",
    "Invite",
    "JoinRefused",
    "MalformedFrame",
    "MineWorldError",
    "NotOffered",
    "Outcome",
    "ProtocolMismatch",
    "ProtocolViolation",
    "RefusedRequest",
    "SeatSession",
    "SessionClosed",
    "UnsupportedFrame",
    "offers",
]
