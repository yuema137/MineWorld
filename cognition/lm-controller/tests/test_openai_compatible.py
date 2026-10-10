"""AP5-6 (b) and (c): the adapter's exact mapping, its error mapping, and no retry (D-P5-4, D-P5-10).

Most cases run over httpx2's mock transport. Three use real loopback sockets the test owns: a stub
server with a canned answer (the real transport path), a closed port, and a server that never answers.
"""

from __future__ import annotations

import asyncio
import json
import socket
import time

import httpx2
import pytest
from pydantic import ValidationError
from support import golden_request, request, run

from mineworld_cognition.backend.model import BackendFailure, Completion
from mineworld_cognition.backend.openai_compatible import OpenAICompatibleBackend
from mineworld_cognition.config import BackendConfig
from mineworld_cognition.secrets import Secret

ANSWER = {
    "choices": [
        {"message": {"role": "assistant", "content": '{"act":"say"}'}, "finish_reason": "stop"}
    ],
    "usage": {"prompt_tokens": 31, "completion_tokens": 7},
}


def _config(**overrides: object) -> BackendConfig:
    fields: dict[str, object] = {
        "kind": "openai-compatible",
        "base_url": "http://127.0.0.1:9/v1",
        "model": "test-model",
    }
    fields.update(overrides)
    return BackendConfig.model_validate(fields)


class Recorder:
    """A mock-transport handler: answers with `status` and `body`, and keeps every request it saw."""

    def __init__(self, status: int = 200, body: bytes | None = None) -> None:
        self.status = status
        self.body = json.dumps(ANSWER).encode() if body is None else body
        self.seen: list[httpx2.Request] = []

    def __call__(self, sent: httpx2.Request) -> httpx2.Response:
        self.seen.append(sent)
        return httpx2.Response(self.status, content=self.body)


def _ask(
    config: BackendConfig, handler: Recorder, key: Secret | None = None, *, golden: bool = True
) -> Completion | BackendFailure:
    backend = OpenAICompatibleBackend(config, key, transport=httpx2.MockTransport(handler))

    async def session() -> Completion | BackendFailure:
        try:
            return await backend.complete(golden_request() if golden else request("hi"))
        finally:
            await backend.aclose()

    return run(session())


def test_the_request_maps_to_the_exact_body_with_no_authorization_without_a_key() -> None:
    handler = Recorder()
    answer = _ask(_config(), handler)
    sent = handler.seen[0]
    assert sent.method == "POST"
    assert str(sent.url) == "http://127.0.0.1:9/v1/chat/completions"
    assert "authorization" not in sent.headers
    assert json.loads(sent.content) == {
        "model": "test-model",
        "messages": [
            {"role": "system", "content": "You are Alice, who keeps the café."},
            {"role": "user", "content": 'Bob says: "Hello — one coffee?"'},
        ],
        "max_tokens": 256,
        "stream": False,
        "temperature": 0.7,
        "seed": 42,
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "answer",
                "schema": {
                    "type": "object",
                    "properties": {"act": {"enum": ["say", "wait"]}},
                    "required": ["act"],
                },
                "strict": True,
            },
        },
    }
    assert answer == Completion.model_validate(
        {
            "text": '{"act":"say"}',
            "finish": "complete",
            "usage": {"input_tokens": 31, "output_tokens": 7, "estimated": False},
        }
    )


def test_the_typed_options_change_only_their_own_fields() -> None:
    handler = Recorder()
    _ask(_config(structured_output="json_object", temperature="omit", reasoning="off"), handler)
    body = json.loads(handler.seen[0].content)
    assert body["response_format"] == {"type": "json_object"}
    assert "temperature" not in body
    assert body["reasoning_effort"] == "none"
    handler = Recorder()
    _ask(_config(structured_output="none", reasoning="high"), handler)
    body = json.loads(handler.seen[0].content)
    assert "response_format" not in body
    assert body["reasoning_effort"] == "high"


def test_a_key_is_sent_as_a_bearer_token() -> None:
    handler = Recorder()
    _ask(_config(key_env="MWTEST_KEY"), handler, Secret("MWTEST-abc"))
    assert handler.seen[0].headers["authorization"] == "Bearer MWTEST-abc"


@pytest.mark.parametrize(
    ("status", "body", "failure"),
    [
        (401, b'{"error":"bad key"}', BackendFailure(reason="unauthorized")),
        (500, b"oops", BackendFailure(reason="http_status", status=500)),
        (200, b"<html>not json</html>", BackendFailure(reason="malformed_response")),
        (
            200,
            json.dumps(
                {"choices": [{"message": {"content": "x"}, "finish_reason": "odd"}]}
            ).encode(),
            BackendFailure(reason="malformed_response"),
        ),
    ],
)
def test_server_answers_map_to_typed_failures(
    status: int, body: bytes, failure: BackendFailure
) -> None:
    assert _ask(_config(), Recorder(status, body)) == failure


def test_a_failure_is_never_retried() -> None:
    # AP5-6 (c): one complete() is one request, even for a 503.
    handler = Recorder(503, b"busy")
    assert _ask(_config(), handler) == BackendFailure(reason="http_status", status=503)
    assert len(handler.seen) == 1


def test_missing_usage_is_estimated_and_flagged() -> None:
    body = json.dumps(
        {"choices": [{"message": {"content": "abcdefgh"}, "finish_reason": "length"}]}
    )
    answer = _ask(_config(), Recorder(200, body.encode()), golden=False)
    assert isinstance(answer, Completion)
    assert answer.finish == "length"
    assert (answer.usage.input_tokens, answer.usage.output_tokens, answer.usage.estimated) == (
        1,
        2,
        True,
    )


def _real(config: BackendConfig) -> Completion | BackendFailure:
    backend = OpenAICompatibleBackend(config, None)

    async def session() -> Completion | BackendFailure:
        try:
            return await backend.complete(request("hi"))
        finally:
            await backend.aclose()

    return run(session())


def test_a_loopback_stub_server_is_reached_over_real_sockets() -> None:
    # The real transport path: httpx2's connection pool, h11, a socket on 127.0.0.1.
    seen: list[bytes] = []

    async def session() -> Completion | BackendFailure:
        async def serve(reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
            head = await reader.readuntil(b"\r\n\r\n")
            length = next(
                int(line.split(b":")[1])
                for line in head.split(b"\r\n")
                if line.lower().startswith(b"content-length:")
            )
            seen.append(head + await reader.readexactly(length))
            body = json.dumps(ANSWER).encode()
            writer.write(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: "
                + str(len(body)).encode()
                + b"\r\nConnection: close\r\n\r\n"
                + body
            )
            await writer.drain()
            writer.close()

        server = await asyncio.start_server(serve, "127.0.0.1", 0)
        port = server.sockets[0].getsockname()[1]
        backend = OpenAICompatibleBackend(_config(base_url=f"http://127.0.0.1:{port}/v1"), None)
        try:
            return await backend.complete(request("hi"))
        finally:
            await backend.aclose()
            server.close()
            await server.wait_closed()

    answer = run(session())
    assert isinstance(answer, Completion)
    assert answer.text == '{"act":"say"}'
    assert seen[0].startswith(b"POST /v1/chat/completions HTTP/1.1\r\n")


def test_a_closed_port_is_unreachable() -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    assert _real(_config(base_url=f"http://127.0.0.1:{port}/v1")) == BackendFailure(
        reason="unreachable"
    )


def test_a_stalled_server_times_out_within_the_request_timeout() -> None:
    async def session() -> tuple[Completion | BackendFailure, float]:
        held: list[asyncio.StreamWriter] = []

        async def stall(_: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
            held.append(writer)  # accept, read nothing, answer nothing

        server = await asyncio.start_server(stall, "127.0.0.1", 0)
        port = server.sockets[0].getsockname()[1]
        config = _config(base_url=f"http://127.0.0.1:{port}/v1", request_timeout_s=1)
        backend = OpenAICompatibleBackend(config, None)
        started = time.monotonic()
        try:
            answer = await backend.complete(request("hi"))
        finally:
            await backend.aclose()
            for writer in held:
                writer.close()
            server.close()
            await server.wait_closed()
        return answer, time.monotonic() - started

    answer, elapsed = run(session())
    assert answer == BackendFailure(reason="timeout")
    assert elapsed < 1 + 1


@pytest.mark.parametrize(
    ("base_url", "why"),
    [
        ("https://user:pass@api.example.com/v1", "credentials"),
        ("http://api.example.com/v1", "https"),
        ("https://api.example.com/v1?key=abc", "query"),
        ("ftp://127.0.0.1/v1", "http or https"),
    ],
)
def test_an_unsafe_base_url_is_refused(base_url: str, why: str) -> None:
    with pytest.raises(ValidationError, match=why):
        _config(base_url=base_url)


def test_base_url_has_no_default() -> None:
    with pytest.raises(ValidationError, match="base_url"):
        BackendConfig.model_validate({"kind": "openai-compatible", "model": "m"})
