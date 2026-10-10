"""The provider-neutral interface to a language model (pr-s10-p5-backends.md D-P5-2).

A cognition component asks a question as a `CompletionRequest` and receives a `Completion`, or a
`BackendFailure` when the model could not be asked. Nothing here names a provider, a model, an endpoint,
a tier, a binding or a key (I-10): those belong to the operator's configuration, which selects the
backend before a request is built (D-P5-3). The request is the cassette's key material, so it holds no
float: a temperature is in thousandths (`temperature_milli`, D-P5-6).
"""

from __future__ import annotations

from typing import Annotated, Literal, Protocol

from mineworld_sdk.wire.ids import JsonValue
from pydantic import BaseModel, ConfigDict, Field, model_validator

Purpose = Literal["decide", "summarize"]
"""Why the model is asked: a decision a seat acts on, or a summary memory keeps (step-17 §3.7.1)."""

Role = Literal["system", "user", "assistant"]

Finish = Literal["complete", "length", "refused"]
"""How the model ended: it finished; it hit `max_output_tokens`; it declined (a content filter)."""

FailureReason = Literal[
    "unreachable", "timeout", "http_status", "malformed_response", "unauthorized"
]
"""Why no completion exists. A failure is never something the model said, so it is never recorded."""

_U32_MAX = 2**32 - 1


class CognitionModel(BaseModel):
    """Strict, frozen, closed: the configuration every model of this package shares (as the SDK's)."""

    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


class Message(CognitionModel):
    role: Role
    text: str


class Sampling(CognitionModel):
    temperature_milli: Annotated[int, Field(ge=0, le=2000)]
    """The temperature times 1 000: 700 is 0.7. An integer, so the key never formats a float."""
    max_output_tokens: Annotated[int, Field(ge=1, le=32768)]
    seed: Annotated[int, Field(ge=0, le=_U32_MAX)] | None = None


class CompletionRequest(CognitionModel):
    purpose: Purpose
    messages: Annotated[tuple[Message, ...], Field(min_length=1)]
    output_schema: JsonValue | None = None
    """The JSON Schema the answer must satisfy, or None for free text. It may hold no float."""
    sampling: Sampling


class Usage(CognitionModel):
    input_tokens: Annotated[int, Field(ge=0)]
    output_tokens: Annotated[int, Field(ge=0)]
    estimated: bool
    """True when the backend reported no usage and the numbers are the gateway's estimate."""


class Completion(CognitionModel):
    text: str
    finish: Finish
    usage: Usage


class BackendFailure(CognitionModel):
    """The model could not be asked. It carries a typed reason and, for `http_status`, the code; never a
    response body, a header or a URL, any of which can carry a key (I-16, AP5-9)."""

    reason: FailureReason
    status: Annotated[int, Field(ge=100, le=599)] | None = None

    @model_validator(mode="after")
    def _status_with_http_status_only(self) -> BackendFailure:
        if (self.reason == "http_status") != (self.status is not None):
            raise ValueError("a status code is given exactly when the reason is http_status")
        return self


def estimate_tokens(text: str) -> int:
    """A deterministic token estimate: a quarter of the UTF-8 bytes, rounded up (D-P5-8). Used when a
    backend reports no usage, and by the budget's pre-check."""
    return -(-len(text.encode("utf-8")) // 4)


class ModelBackend(Protocol):
    """Anything that answers a `CompletionRequest`: a scripted function, a cassette, or an adapter."""

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        """Ask once. No retry: one call is one ledger entry (D-P5-4 (c))."""
        ...

    async def aclose(self) -> None:
        """Release what the backend holds (a connection pool; a cassette being written)."""
        ...
