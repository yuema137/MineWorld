"""What a caller asks the inspector (pr-s28e §5.4): one request per view, discriminated by `view`."""

from __future__ import annotations

from typing import Annotated, Literal

from mineworld_sdk.wire.ids import EntityKeyField, EventIdField
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel


class AccessRequest(CognitionModel):
    view: Literal["access"] = "access"
    seat: EntityKeyField
    event: EventIdField | None = None


class ForgottenRequest(CognitionModel):
    view: Literal["forgotten"] = "forgotten"
    seat: EntityKeyField


class BytesRequest(CognitionModel):
    view: Literal["bytes"] = "bytes"
    seat: EntityKeyField | None = None


class RecallRequest(CognitionModel):
    view: Literal["recall"] = "recall"
    seat: EntityKeyField
    at: int
    counterparts: tuple[str, ...] = ()
    """Entity ids, or names the seat's own name book holds exactly, case-sensitive (F-S28e-8)."""
    words: str = ""
    trace: bool = False
    trace_all: bool = False
    """Show every held key inside a cited range, not the first `TRACE_KEYS` (F-S28e-7)."""


class ScopesRequest(CognitionModel):
    view: Literal["scopes"] = "scopes"
    seat: EntityKeyField


InspectRequest = Annotated[
    AccessRequest | ForgottenRequest | BytesRequest | RecallRequest | ScopesRequest,
    Field(discriminator="view"),
]
