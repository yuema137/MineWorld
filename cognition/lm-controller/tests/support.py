"""Shared test values: literal requests and completions, and a synchronous runner for coroutines."""

from __future__ import annotations

import asyncio
from collections.abc import Coroutine
from typing import Any

from mineworld_cognition.backend.model import (
    Completion,
    CompletionRequest,
    Message,
    Sampling,
    Usage,
)

GOLDEN_KEY = "6bd78b32af7a296775cda4bc397c7803340a7377a9e897195003658a11037a27"
"""AP5-1 (a): computed by hand and `shasum -a 256` before any test existed (ledger §13.3)."""

GOLDEN_CANONICAL = (
    '{"key_scheme":1,"request":{"messages":[{"role":"system","text":"You are Alice, who keeps the '
    'café."},{"role":"user","text":"Bob says: \\"Hello — one coffee?\\""}],"output_schema":'
    '{"properties":{"act":{"enum":["say","wait"]}},"required":["act"],"type":"object"},'
    '"purpose":"decide","sampling":{"max_output_tokens":256,"seed":42,"temperature_milli":700}}}'
).encode()


def golden_request() -> CompletionRequest:
    return CompletionRequest(
        purpose="decide",
        messages=(
            Message(role="system", text="You are Alice, who keeps the café."),
            Message(role="user", text='Bob says: "Hello — one coffee?"'),
        ),
        output_schema={
            "type": "object",
            "properties": {"act": {"enum": ["say", "wait"]}},
            "required": ["act"],
        },
        sampling=Sampling(temperature_milli=700, max_output_tokens=256, seed=42),
    )


def request(text: str, *, max_output_tokens: int = 64) -> CompletionRequest:
    """A small request whose key differs by `text`."""
    return CompletionRequest(
        purpose="decide",
        messages=(Message(role="user", text=text),),
        sampling=Sampling(temperature_milli=0, max_output_tokens=max_output_tokens, seed=1),
    )


def completion(text: str, *, input_tokens: int = 10, output_tokens: int = 5) -> Completion:
    return Completion(
        text=text,
        finish="complete",
        usage=Usage(input_tokens=input_tokens, output_tokens=output_tokens, estimated=False),
    )


def run[T](coroutine: Coroutine[Any, Any, T], *, timeout_s: float = 10) -> T:
    """Runs one coroutine to completion with a bound, on a selector event loop on every platform.

    The selector loop connects with `socket.connect`, which the network guard (pytest-socket) patches.
    Windows' default proactor loop connects with `ConnectEx` instead and is not guarded: CI's first
    Windows run showed a TEST-NET connection taking 10 s rather than being refused (ledger F-P5-4)."""

    async def bounded() -> T:
        async with asyncio.timeout(timeout_s):
            return await coroutine

    return asyncio.run(bounded(), loop_factory=asyncio.SelectorEventLoop)
