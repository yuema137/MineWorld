"""AP5-1 and AP5-2: one stable, provider-free key per request, and no float in key material."""

from __future__ import annotations

import json

import pytest
from mineworld_sdk.wire.ids import JsonValue
from pydantic import ValidationError
from support import GOLDEN_CANONICAL, GOLDEN_KEY, golden_request

from mineworld_cognition.backend.canonical import KeyMaterialError, canonical_json, cassette_key
from mineworld_cognition.backend.model import CompletionRequest, Message, Sampling


def test_golden_key_is_the_hand_computed_literal() -> None:
    # AP5-1 (a): the bytes and their hash were written out by hand from D-P5-6's rules (ledger §13.3).
    # Runs on all three CI legs (AP5-12).
    request = golden_request()
    assert canonical_json(request) == GOLDEN_CANONICAL
    assert cassette_key(request) == GOLDEN_KEY


def test_a_provider_field_cannot_enter_the_request() -> None:
    # AP5-1 (c) and I-10: no field can carry a model, a URL or a binding; the closed model refuses one.
    data = golden_request().model_dump(mode="json")
    data["model"] = "some-model"
    with pytest.raises(ValidationError, match="model"):
        CompletionRequest.model_validate_json(json.dumps(data))


@pytest.mark.parametrize(
    ("schema", "path"),
    [
        ({"type": "number", "minimum": 0.5}, "$.request.output_schema.minimum"),
        (
            {"type": "array", "prefixItems": [{"const": 1}, {"const": 2.5}]},
            "$.request.output_schema.prefixItems[1].const",
        ),
    ],
)
def test_a_float_in_the_output_schema_is_refused_by_path(schema: JsonValue, path: str) -> None:
    # AP5-2: the SDK's JsonValue admits floats, so the request is valid and the refusal must come from
    # the key itself, naming where the float is.
    request = CompletionRequest(
        purpose="decide",
        messages=(Message(role="user", text="hi"),),
        output_schema=schema,
        sampling=Sampling(temperature_milli=0, max_output_tokens=8),
    )
    with pytest.raises(KeyMaterialError) as raised:
        cassette_key(request)
    assert raised.value.path == path


def test_a_float_at_the_top_level_or_in_messages_cannot_be_built() -> None:
    # AP5-2, the other two places: the request's own fields are integers and strings, and the strict
    # models refuse a float there before any key is computed.
    with pytest.raises(ValidationError):
        Sampling.model_validate({"temperature_milli": 0.7, "max_output_tokens": 1})
    with pytest.raises(ValidationError):
        Message.model_validate({"role": "user", "text": 0.5})
