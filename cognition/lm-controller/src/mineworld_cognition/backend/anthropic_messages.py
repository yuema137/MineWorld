"""Claude by API key: the Anthropic Messages API over `httpx2` (DEP-33; pr-s10-p5b §5.1).

Anthropic's OpenAI-compatibility layer ignores `response_format` and `seed`, so Claude has this adapter
of its own. Its rules are the OpenAI-compatible adapter's (D-P5-4):

(a) it is built only from a `BackendConfig` and an already-resolved key; it reads no environment
    variable, and its client is created with `trust_env=False` (F-P5-1);
(b) `base_url` has no default (`https://api.anthropic.com` in the example);
(c) no retries: one call is one request and one ledger entry;
(d) the key travels in `x-api-key`, only when a key was given; there is never an `Authorization` header,
    and nothing here logs a request or a header.

The request's schema is lowered to Anthropic's strict subset on a **copy** (`strict_schema`): the cassette
key, computed by the recorder on the unlowered request, is unchanged. A failure never carries a response
body, a header or the URL: only a typed reason and a status.
"""

from __future__ import annotations

import json
from collections.abc import Mapping

import httpx2
from mineworld_sdk.wire.ids import JsonValue

from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    Finish,
    Usage,
    estimate_tokens,
)
from mineworld_cognition.backend.strict_schema import lower_schema
from mineworld_cognition.config import BackendConfig
from mineworld_cognition.secrets import Secret

API_VERSION = "2023-06-01"
_FINISH: Mapping[str, Finish] = {
    "end_turn": "complete",
    "stop_sequence": "complete",
    "max_tokens": "length",
    "refusal": "refused",
}


def request_body(config: BackendConfig, request: CompletionRequest) -> dict[str, object]:
    """The JSON body of `POST {base_url}/v1/messages` for one request."""
    system = [message.text for message in request.messages if message.role == "system"]
    body: dict[str, object] = {
        "model": config.model,
        "max_tokens": request.sampling.max_output_tokens,
        "messages": [
            {"role": message.role, "content": message.text}
            for message in request.messages
            if message.role != "system"
        ],
    }
    if system:
        body["system"] = "\n".join(system)
    if config.temperature == "send":
        body["temperature"] = request.sampling.temperature_milli / 1000
    if request.output_schema is not None and config.structured_output == "json_schema":
        body["output_config"] = {
            "format": {"type": "json_schema", "schema": lower_schema(request.output_schema)}
        }
    return body


def _count(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else None


def _parse(payload: JsonValue, request: CompletionRequest) -> Completion | BackendFailure:
    malformed = BackendFailure(reason="malformed_response")
    if not isinstance(payload, dict):
        return malformed
    reason = payload.get("stop_reason")
    content = payload.get("content")
    if not isinstance(reason, str) or reason not in _FINISH or not isinstance(content, list):
        return malformed
    texts: list[str] = []
    for block in content:
        if not isinstance(block, dict):
            return malformed
        if block.get("type") == "text":
            text = block.get("text")
            if not isinstance(text, str):
                return malformed
            texts.append(text)
    answer = "".join(texts)
    usage = payload.get("usage")
    given = _count(usage.get("input_tokens")) if isinstance(usage, dict) else None
    produced = _count(usage.get("output_tokens")) if isinstance(usage, dict) else None
    if given is not None and produced is not None:
        counted = Usage(input_tokens=given, output_tokens=produced, estimated=False)
    else:
        asked = "".join(sent.text for sent in request.messages)
        counted = Usage(
            input_tokens=estimate_tokens(asked),
            output_tokens=estimate_tokens(answer),
            estimated=True,
        )
    return Completion(text=answer, finish=_FINISH[reason], usage=counted)


class AnthropicMessagesBackend:
    def __init__(
        self,
        config: BackendConfig,
        key: Secret | None,
        *,
        transport: httpx2.AsyncBaseTransport | None = None,
    ) -> None:
        self._config = config
        self._url = config.base_url.rstrip("/") + "/v1/messages"
        headers = {"Content-Type": "application/json", "anthropic-version": API_VERSION}
        if key is not None:
            headers["x-api-key"] = key.reveal()
        self._client = httpx2.AsyncClient(
            headers=headers,
            timeout=httpx2.Timeout(config.request_timeout_s),
            trust_env=False,
            follow_redirects=False,
            transport=transport,
        )

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        content = json.dumps(request_body(self._config, request), ensure_ascii=False).encode()
        try:
            response = await self._client.post(self._url, content=content)
        except httpx2.TimeoutException:
            return BackendFailure(reason="timeout")
        except httpx2.TransportError:
            return BackendFailure(reason="unreachable")
        if response.status_code in (401, 403):
            return BackendFailure(reason="unauthorized")
        if not 200 <= response.status_code < 300:
            return BackendFailure(reason="http_status", status=response.status_code)
        try:
            payload: JsonValue = response.json()
        except ValueError:
            return BackendFailure(reason="malformed_response")
        return _parse(payload, request)

    async def aclose(self) -> None:
        await self._client.aclose()

    def __repr__(self) -> str:
        return f"AnthropicMessagesBackend(model={self._config.model!r})"
