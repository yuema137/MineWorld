"""Frames to and from JSON text, exactly as the Rust types write them.

The omission rules are declared on the fields that need them (`Refused.token`, `Refused.detail`,
`Closing.detail`, `Affordance.payload`), mirroring each field's `skip_serializing_if`. There is no
blanket `exclude_none`: `Welcome.resume` and every other `Option` the Rust side writes as `null` stay
`null` (R-P3-4).
"""

from __future__ import annotations

import json

from pydantic import TypeAdapter, ValidationError

from mineworld_sdk.errors import MalformedFrame
from mineworld_sdk.wire.contract import WireModel
from mineworld_sdk.wire.frames import Join, Leave, ServerFrame, Submit
from mineworld_sdk.wire.ids import JsonValue

_SERVER_FRAMES = TypeAdapter[ServerFrame](ServerFrame)


def encode(frame: Join | Submit | Leave) -> str:
    """The text of a client frame. Only `join`, `submit` and `leave` can be encoded (`INV-9`)."""
    return json.dumps(to_json(frame), ensure_ascii=False, separators=(",", ":"))


def decode(text: str | bytes) -> ServerFrame:
    """A server frame from its text, or `MalformedFrame` naming what did not fit.

    Raises `UnsupportedFrame` for a frame revision 2 specifies but this SDK does not yet model.
    """
    try:
        return _SERVER_FRAMES.validate_json(text)
    except ValidationError as error:
        raise MalformedFrame(str(error)) from None


def to_json(frame: WireModel) -> JsonValue:
    """A frame as the JSON value it encodes to, for comparison with another JSON value."""
    value: JsonValue = frame.model_dump(mode="json")
    return value
