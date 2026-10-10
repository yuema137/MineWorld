"""Identity on the wire: distinct types for distinct identifiers, validated where they enter a model.

Every identifier is a `NewType` over `str`, so pyright refuses an `EventId` where an `EntityId` is
expected (`.structured-coding/standards.md`: distinct identifiers are distinct types). The `*Field`
aliases add the validation the Rust type performs on decode, and are what the models' fields use.

**Opaque identities are decimal strings, and only strings** (`server/PROTOCOL.md` §7, D-P3-3). The
server writes `EntityId`, `EventId`, `ActionId` and `SessionId` as `"9007199254740995"`. A JSON number
in an id position is refused here, although the Rust decoder would accept one: a client must never be
the place where an id passed through a double (`spike/FINDINGS.md` F1, F2). Nothing here converts an id
to `int`, so two ids above 2^53 stay two ids.
"""

from __future__ import annotations

import re
import unicodedata
from typing import Annotated, NewType

from pydantic import AfterValidator

EntityId = NewType("EntityId", str)
EventId = NewType("EventId", str)
ActionId = NewType("ActionId", str)
SessionId = NewType("SessionId", str)
ProcessId = NewType("ProcessId", str)

EntityKey = NewType("EntityKey", str)
ActionTypeId = NewType("ActionTypeId", str)
EventTypeId = NewType("EventTypeId", str)
SystemId = NewType("SystemId", str)
ComponentTypeId = NewType("ComponentTypeId", str)
RelationTypeId = NewType("RelationTypeId", str)
RejectionCode = NewType("RejectionCode", str)
Tag = NewType("Tag", str)

WorldInstanceId = NewType("WorldInstanceId", str)
ResumeSecret = NewType("ResumeSecret", str)
"""`welcome.resume`: 128 secret bits as 32 lowercase hexadecimal characters (`PROTOCOL.md` §5.1)."""
CorrelationToken = NewType("CorrelationToken", str)

type JsonValue = bool | int | float | str | list[JsonValue] | dict[str, JsonValue] | None
"""Any JSON value: the type of every payload the SDK carries without interpreting (D-P3-7).

A payload belongs to the System Pack that owns it (`docs/CORE_CONCEPTS.md` §15), exactly as the server
carries it as `serde_json::Value`. Code that knows the pack interprets it; the SDK never does.
"""

_DECIMAL = re.compile(r"(?:0|[1-9][0-9]*)")
_U64_MAX = 2**64 - 1
_IDENTIFIER = re.compile(r"[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?")
_MAX_IDENTIFIER_LENGTH = 64
_INSTANCE = re.compile(r"[0-9a-f]{32}")
MAX_TOKEN_LENGTH = 64
"""The longest correlation token, in UTF-8 bytes (`server/src/protocol/request.rs`)."""


def _decimal(value: str) -> str:
    if not _DECIMAL.fullmatch(value) or int(value) > _U64_MAX:
        raise ValueError(
            f"an id is an unsigned 64-bit integer written as a decimal string: {value!r}"
        )
    return value


def _identifier(value: str) -> str:
    # contracts/src/ids.rs `check_identifier`: 1 to 64 bytes of [a-z0-9_-], no separator at an edge.
    if len(value) > _MAX_IDENTIFIER_LENGTH or not _IDENTIFIER.fullmatch(value):
        raise ValueError(
            "an identifier is 1 to 64 lowercase ASCII letters, digits, '-' and '_', not beginning or "
            f"ending with a separator: {value!r}"
        )
    return value


def _instance(value: str) -> str:
    if not _INSTANCE.fullmatch(value):
        raise ValueError(f"a world instance is 32 lowercase hexadecimal characters: {value!r}")
    return value


def _resume(value: str) -> str:
    if not _INSTANCE.fullmatch(value):
        raise ValueError("a resume secret is 32 lowercase hexadecimal characters")
    return value


def _token(value: str) -> str:
    length = len(value.encode("utf-8"))
    if not 1 <= length <= MAX_TOKEN_LENGTH:
        raise ValueError(
            f"a correlation token is 1 to {MAX_TOKEN_LENGTH} bytes, and this is {length}"
        )
    if any(unicodedata.category(character) == "Cc" for character in value):
        raise ValueError("a correlation token must not contain control characters")
    return value


EntityIdField = Annotated[EntityId, AfterValidator(_decimal)]
EventIdField = Annotated[EventId, AfterValidator(_decimal)]
ActionIdField = Annotated[ActionId, AfterValidator(_decimal)]
SessionIdField = Annotated[SessionId, AfterValidator(_decimal)]
ProcessIdField = Annotated[ProcessId, AfterValidator(_decimal)]

EntityKeyField = Annotated[EntityKey, AfterValidator(_identifier)]
ActionTypeIdField = Annotated[ActionTypeId, AfterValidator(_identifier)]
EventTypeIdField = Annotated[EventTypeId, AfterValidator(_identifier)]
SystemIdField = Annotated[SystemId, AfterValidator(_identifier)]
ComponentTypeIdField = Annotated[ComponentTypeId, AfterValidator(_identifier)]
RelationTypeIdField = Annotated[RelationTypeId, AfterValidator(_identifier)]
RejectionCodeField = Annotated[RejectionCode, AfterValidator(_identifier)]
TagField = Annotated[Tag, AfterValidator(_identifier)]

WorldInstanceIdField = Annotated[WorldInstanceId, AfterValidator(_instance)]
ResumeSecretField = Annotated[ResumeSecret, AfterValidator(_resume)]
CorrelationTokenField = Annotated[CorrelationToken, AfterValidator(_token)]
