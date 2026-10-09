"""The SDK's errors. Every one derives from `MineWorldError`, so a caller can catch the SDK's failures
without catching a library's.

No error message ever contains the invite (D-P3-9): the session never formats one into a message, and
the server never sends one back.
"""

from __future__ import annotations


class MineWorldError(Exception):
    """Something the SDK refused or could not do."""


class MalformedFrame(MineWorldError):
    """A frame from the server is not a revision-2 frame this SDK models: unknown kind, unknown field,
    wrong type, or a value the contract forbids. The server and this SDK disagree, which is a defect on
    one side (`server/PROTOCOL.md` §2's "unknown fields are refused", from the client's side)."""


class UnsupportedFrame(MineWorldError):
    """A frame carries a field revision 2 specifies but this SDK does not model yet (D-P3-6)."""
