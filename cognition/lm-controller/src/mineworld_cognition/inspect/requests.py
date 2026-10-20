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


InspectRequest = Annotated[
    AccessRequest | ForgottenRequest | BytesRequest, Field(discriminator="view")
]
