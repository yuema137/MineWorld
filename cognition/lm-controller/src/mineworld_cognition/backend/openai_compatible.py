"""The one HTTP adapter: the OpenAI-compatible chat-completions schema over `httpx2` (D-P5-4; DEP-27).

It reaches Ollama, llama.cpp's server, LM Studio, vLLM and any hosted OpenAI-compatible API the operator
configures. Its rules:

(a) it is built only from a `BackendConfig` and an already-resolved key; it reads no environment
    variable at all, and its client is created with `trust_env=False`, so no proxy, `.netrc` or
    certificate variable is read either (F-P5-1);
(b) `base_url` has no default;
(c) no retries: one call is one request and one ledger entry;
(d) `Authorization` is sent only when a key was given, and nothing here logs a request or a header;
(e) this is the only module that imports `httpx2`.

A failure never carries a response body, a header or the URL (AP5-9): only a typed reason and a status.
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
from mineworld_cognition.config import BackendConfig
from mineworld_cognition.secrets import Secret

_FINISH: Mapping[str, Finish] = {
    "stop": "complete",
    "length": "length",
    "content_filter": "refused",
}
_REASONING_EFFORT = {"off": "none", "low": "low", "medium": "medium", "high": "high"}
_SCHEMA_NAME = "answer"


def request_body(config: BackendConfig, request: CompletionRequest) -> dict[str, object]:
    """The JSON body of `POST {base_url}/chat/completions` for one request."""
    body: dict[str, object] = {
        "model": config.model,
        "messages": [
            {"role": message.role, "content": message.text} for message in request.messages
        ],
        "max_tokens": request.sampling.max_output_tokens,
        "stream": False,
    }
    if config.temperature == "send":
        body["temperature"] = request.sampling.temperature_milli / 1000
    if request.sampling.seed is not None:
        body["seed"] = request.sampling.seed
    if request.output_schema is not None:
        if config.structured_output == "json_schema":
            body["response_format"] = {
                "type": "json_schema",
                "json_schema": {
                    "name": _SCHEMA_NAME,
                    "schema": request.output_schema,
                    "strict": True,
                },
            }
        elif config.structured_output == "json_object":
            body["response_format"] = {"type": "json_object"}
    if config.reasoning != "unset":
        body["reasoning_effort"] = _REASONING_EFFORT[config.reasoning]
    return body


def _count(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else None


def _parse(payload: JsonValue, request: CompletionRequest) -> Completion | BackendFailure:
    malformed = BackendFailure(reason="malformed_response")
    if not isinstance(payload, dict):
        return malformed
    choices = payload.get("choices")
    if not isinstance(choices, list) or not choices or not isinstance(choices[0], dict):
        return malformed
    choice = choices[0]
    message = choice.get("message")
    reason = choice.get("finish_reason")
    if not isinstance(message, dict) or not isinstance(reason, str) or reason not in _FINISH:
        return malformed
    content = message.get("content")
    finish = _FINISH[reason]
    if content is None and finish == "refused":
        content = ""
    if not isinstance(content, str):
        return malformed
    usage = payload.get("usage")
    prompt = _count(usage.get("prompt_tokens")) if isinstance(usage, dict) else None
    completion = _count(usage.get("completion_tokens")) if isinstance(usage, dict) else None
    if prompt is not None and completion is not None:
        counted = Usage(input_tokens=prompt, output_tokens=completion, estimated=False)
    else:
        asked = "".join(sent.text for sent in request.messages)
        counted = Usage(
            input_tokens=estimate_tokens(asked),
            output_tokens=estimate_tokens(content),
            estimated=True,
        )
    return Completion(text=content, finish=finish, usage=counted)


class OpenAICompatibleBackend:
    def __init__(
        self,
        config: BackendConfig,
        key: Secret | None,
        *,
        transport: httpx2.AsyncBaseTransport | None = None,
    ) -> None:
        self._config = config
        self._url = config.base_url.rstrip("/") + "/chat/completions"
        headers = {"Content-Type": "application/json"}
        if key is not None:
            headers["Authorization"] = f"Bearer {key.reveal()}"
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
        return f"OpenAICompatibleBackend(model={self._config.model!r})"
