"""AB-1 and AB-2: the Messages adapter's exact mapping, its key-neutral schema lowering, its error
mapping and no retry; and the refusal of the OpenAI-compatibility layer for Claude (DEP-33).

Every case runs over httpx2's mock transport: no socket is opened and no hosted API is reached.
"""

from __future__ import annotations

import json
from pathlib import Path

import httpx2
import pytest
from mineworld_sdk.wire.ids import JsonValue
from support import GOLDEN_KEY, golden_request, run

from mineworld_cognition.backend.anthropic_messages import AnthropicMessagesBackend
from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    Message,
    Sampling,
)
from mineworld_cognition.backend.strict_schema import lower_schema
from mineworld_cognition.config import BackendConfig, ConfigError, load
from mineworld_cognition.record import Cassette, RecordingBackend
from mineworld_cognition.secrets import Secret

KEY = "MWTEST-anthropic-key"


def _answer(stop_reason: str = "end_turn", *, usage: bool = True) -> bytes:
    body: dict[str, object] = {
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": '{"act":'},
            {"type": "text", "text": '"say"}'},
        ],
        "stop_reason": stop_reason,
    }
    if usage:
        body["usage"] = {"input_tokens": 31, "output_tokens": 7}
    return json.dumps(body).encode()


def _config(**overrides: object) -> BackendConfig:
    fields: dict[str, object] = {
        "kind": "anthropic",
        "base_url": "https://api.example.com",
        "model": "claude-test",
        "key_env": "MWTEST_ANTHROPIC_KEY",
    }
    fields.update(overrides)
    return BackendConfig.model_validate(fields)


class Recorder:
    def __init__(self, status: int = 200, body: bytes | None = None) -> None:
        self.status = status
        self.body = _answer() if body is None else body
        self.seen: list[httpx2.Request] = []

    def __call__(self, sent: httpx2.Request) -> httpx2.Response:
        self.seen.append(sent)
        return httpx2.Response(self.status, content=self.body)


def _bounded_request() -> CompletionRequest:
    return CompletionRequest(
        purpose="decide",
        messages=(
            Message(role="system", text="You are Alice."),
            Message(role="system", text="You keep the café."),
            Message(role="user", text="Bob: one coffee?"),
            Message(role="assistant", text="Coming up."),
            Message(role="user", text="Thanks."),
        ),
        output_schema={
            "type": "object",
            "properties": {
                "act": {"enum": ["say", "wait"]},
                "words": {"type": "string", "maxLength": 480},
            },
            "required": ["act", "words"],
        },
        sampling=Sampling(temperature_milli=700, max_output_tokens=256, seed=42),
    )


def _ask(
    config: BackendConfig,
    handler: Recorder,
    asked: CompletionRequest | None = None,
    *,
    key: Secret | None = None,
) -> Completion | BackendFailure:
    backend = AnthropicMessagesBackend(config, key, transport=httpx2.MockTransport(handler))

    async def session() -> Completion | BackendFailure:
        try:
            return await backend.complete(asked or golden_request())
        finally:
            await backend.aclose()

    return run(session())


def test_the_request_maps_to_the_exact_messages_body() -> None:
    # AB-1: system hoisted and joined, no temperature by default, no seed, the lowered schema.
    handler = Recorder()
    answer = _ask(_config(), handler, _bounded_request(), key=Secret(KEY))
    sent = handler.seen[0]
    assert sent.method == "POST"
    assert str(sent.url) == "https://api.example.com/v1/messages"
    assert json.loads(sent.content) == {
        "model": "claude-test",
        "max_tokens": 256,
        "system": "You are Alice.\nYou keep the café.",
        "messages": [
            {"role": "user", "content": "Bob: one coffee?"},
            {"role": "assistant", "content": "Coming up."},
            {"role": "user", "content": "Thanks."},
        ],
        "output_config": {
            "format": {
                "type": "json_schema",
                "schema": {
                    "type": "object",
                    "properties": {
                        "act": {"enum": ["say", "wait"]},
                        "words": {"type": "string"},
                    },
                    "required": ["act", "words"],
                    "additionalProperties": False,
                },
            }
        },
    }
    assert sent.headers["x-api-key"] == KEY
    assert sent.headers["anthropic-version"] == "2023-06-01"
    assert "authorization" not in sent.headers
    assert answer == Completion.model_validate(
        {
            "text": '{"act":"say"}',
            "finish": "complete",
            "usage": {"input_tokens": 31, "output_tokens": 7, "estimated": False},
        }
    )


def test_temperature_is_sent_only_when_asked_and_no_schema_with_none() -> None:
    handler = Recorder()
    _ask(_config(temperature="send", structured_output="none"), handler, _bounded_request())
    body = json.loads(handler.seen[0].content)
    assert body["temperature"] == 0.7
    assert "output_config" not in body
    assert "seed" not in body
    assert "x-api-key" not in handler.seen[0].headers  # no key resolved, none sent


def test_the_cassette_key_is_the_unlowered_requests(tmp_path: Path) -> None:
    # AB-1's key half: recorded through the adapter, the entry's key is AP5-1's literal and its request
    # still carries the schema as asked (no additionalProperties), so lowering happened on a copy.
    handler = Recorder()
    cassette = tmp_path / "claude.jsonl"
    inner = AnthropicMessagesBackend(_config(), None, transport=httpx2.MockTransport(handler))
    recorder = RecordingBackend(inner, cassette, binding="claude", model="claude-test")

    async def session() -> None:
        await recorder.complete(golden_request())
        await recorder.aclose()

    run(session())
    sent_schema = json.loads(handler.seen[0].content)["output_config"]["format"]["schema"]
    assert sent_schema["additionalProperties"] is False
    (entry,) = Cassette.load(cassette).entries
    assert entry.key == GOLDEN_KEY
    assert entry.request == golden_request()


@pytest.mark.parametrize(
    ("stop_reason", "finish"),
    [("end_turn", "complete"), ("stop_sequence", "complete"), ("max_tokens", "length"),
     ("refusal", "refused")],
)  # fmt: skip
def test_stop_reasons_map_to_finishes(stop_reason: str, finish: str) -> None:
    answer = _ask(_config(), Recorder(body=_answer(stop_reason)))
    assert isinstance(answer, Completion)
    assert answer.finish == finish


def test_an_unknown_stop_reason_or_a_bad_body_is_malformed() -> None:
    assert _ask(_config(), Recorder(body=_answer("tool_use"))) == BackendFailure(
        reason="malformed_response"
    )
    assert _ask(_config(), Recorder(body=b"<html>")) == BackendFailure(reason="malformed_response")


def test_missing_usage_is_estimated_and_flagged() -> None:
    answer = _ask(_config(), Recorder(body=_answer(usage=False)))
    assert isinstance(answer, Completion)
    assert answer.usage.estimated is True
    assert answer.usage.output_tokens == 4  # ceil(len('{"act":"say"}') / 4), by hand


@pytest.mark.parametrize(
    ("status", "failure"),
    [
        (401, BackendFailure(reason="unauthorized")),
        (403, BackendFailure(reason="unauthorized")),
        (429, BackendFailure(reason="http_status", status=429)),
        (500, BackendFailure(reason="http_status", status=500)),
        (529, BackendFailure(reason="http_status", status=529)),
    ],
)
def test_error_statuses_map_to_typed_failures_with_one_request(
    status: int, failure: BackendFailure
) -> None:
    handler = Recorder(status=status, body=b'{"type":"error"}')
    assert _ask(_config(), handler) == failure
    assert len(handler.seen) == 1  # no retry


def test_lowering_is_schema_aware_and_pure() -> None:
    # Expected values written by hand from §5.1's rules.
    schema: JsonValue = {
        "type": "object",
        "properties": {
            "minimum": {"type": "integer", "minimum": 0, "maximum": 9, "multipleOf": 3},
            "tags": {"type": "array", "items": {"type": "string", "minLength": 1}, "minItems": 2,
                     "maxItems": 5},
            "one": {"type": "array", "minItems": 1},
            "choice": {"anyOf": [{"properties": {"x": {"exclusiveMinimum": 0}}}, {"type": "null"}]},
            "fixed": {"enum": [{"maxLength": 3}]},
        },
        "additionalProperties": True,
    }  # fmt: skip
    before = json.dumps(schema, sort_keys=True)
    assert lower_schema(schema) == {
        "type": "object",
        "properties": {
            "minimum": {"type": "integer"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "one": {"type": "array", "minItems": 1},
            "choice": {
                "anyOf": [
                    {"properties": {"x": {}}, "additionalProperties": False},
                    {"type": "null"},
                ]
            },
            "fixed": {"enum": [{"maxLength": 3}]},
        },
        "additionalProperties": False,
    }
    assert json.dumps(schema, sort_keys=True) == before  # the input is untouched


def _write(tmp_path: Path, backend_table: str) -> Path:
    path = tmp_path / "cognition.toml"
    path.write_text(
        '[recording]\nmode = "replay"\ncassette = "c.jsonl"\n[tiers]\nsocial = "x"\n'
        f"[backends.x]\n{backend_table}",
        encoding="utf-8",
        newline="\n",
    )
    return path


def test_the_compatibility_layer_is_refused_for_claude(tmp_path: Path) -> None:
    # AB-2.
    path = _write(
        tmp_path,
        'kind = "openai-compatible"\nbase_url = "https://api.anthropic.com/v1/"\nmodel = "m"\n',
    )
    with pytest.raises(ConfigError) as refused:
        load(path)
    assert 'kind = "anthropic"' in str(refused.value)
    assert "response_format" in str(refused.value)


def test_the_anthropic_kind_loads_with_temperature_omitted_and_refuses_foreign_options(
    tmp_path: Path,
) -> None:
    base = 'kind = "anthropic"\nbase_url = "https://api.anthropic.com"\nmodel = "m"\n'
    config = load(_write(tmp_path, base))
    assert config.backends["x"].temperature == "omit"
    for extra, named in (
        ('preset = "openai"\n', "preset"),
        ('structured_output = "json_object"\n', "structured_output"),
        ('reasoning = "low"\n', "reasoning"),
    ):
        with pytest.raises(ConfigError) as refused:
            load(_write(tmp_path, base + extra))
        assert named in str(refused.value)


def test_a_claude_subscription_kind_does_not_exist(tmp_path: Path) -> None:
    # QP5b-1 (AB-7's "not ruled in" half): the dropped route is an unknown kind.
    path = _write(
        tmp_path,
        'kind = "claude-code-subscription"\nbase_url = "https://api.example.com"\nmodel = "m"\n',
    )
    with pytest.raises(ConfigError) as refused:
        load(path)
    assert "backends.x.kind" in str(refused.value)
