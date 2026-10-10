"""The canonical bytes of a request, and the cassette key over them (D-P5-6, ARC-58).

`canonical_json` is UTF-8 JSON of `{"key_scheme": KEY_SCHEME, "request": …}` with sorted keys, `,` and
`:` separators, non-ASCII characters written as themselves, and no Unicode normalization: the text's
bytes are the request. **A float anywhere is refused**, because float formatting is the classic source
of keys that differ between platforms and versions, and the SDK's `JsonValue` admits floats. A change of
any rule here is a `KEY_SCHEME` bump, never a silent collision.
"""

from __future__ import annotations

import hashlib
import json
from typing import NewType

from mineworld_sdk.wire.ids import JsonValue

from mineworld_cognition.backend.model import CompletionRequest

KEY_SCHEME = 1

CassetteKey = NewType("CassetteKey", str)
"""Lowercase hexadecimal sha256 of `canonical_json(request)`."""


class KeyMaterialError(ValueError):
    """The request holds a value that cannot be keyed: a float. A programming error, never a fallback."""

    def __init__(self, path: str) -> None:
        super().__init__(f"a float cannot be key material, at {path}; use an integer (D-P5-6)")
        self.path = path


def _refuse_floats(value: JsonValue, path: str) -> None:
    if isinstance(value, float):
        raise KeyMaterialError(path)
    if isinstance(value, list):
        for index, item in enumerate(value):
            _refuse_floats(item, f"{path}[{index}]")
    elif isinstance(value, dict):
        for key, item in value.items():
            _refuse_floats(item, f"{path}.{key}")


def canonical_object(request: CompletionRequest) -> dict[str, JsonValue]:
    """The keyed object, float-checked: what `canonical_json` serializes."""
    keyed: dict[str, JsonValue] = {
        "key_scheme": KEY_SCHEME,
        "request": request.model_dump(mode="json"),
    }
    _refuse_floats(keyed, "$")
    return keyed


def canonical_json(request: CompletionRequest) -> bytes:
    return json.dumps(
        canonical_object(request),
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def cassette_key(request: CompletionRequest) -> CassetteKey:
    return CassetteKey(hashlib.sha256(canonical_json(request)).hexdigest())
