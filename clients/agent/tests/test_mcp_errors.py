"""§5.4's table, one row per exception, through an in-process MCP client (pr-s27d §§5.4, 7, F-S27d-2).

A refusal is an `isError` result whose text is the JSON of a `ToolError`, with no structured content; the code is
the adapter's own. An act's refusal is data, not one of these rows (S27-c §5.6)."""

from __future__ import annotations

from typing import Any

import pytest
from mcp.client.client import Client
from mcp.server.lowlevel import Server
from mcp.types import CallToolResult, TextContent
from mcp_double import fake_player
from support import run

from mineworld_agent.mcp_server import Adapter
from mineworld_agent.models import JoinRefused, PlayerRefusal, SeatLostRefusal, ToolError
from mineworld_agent.player import PlayerConfig

HANDLE = "0123456789abcdef0123456789abcdef"
CONFIG = PlayerConfig(server="ws://127.0.0.1:7878/ws")


def failing(name: str, error: BaseException) -> Server[Any]:
    return Adapter().server(fake_player({name: error}), CONFIG)


def call(server: Server[Any], name: str, arguments: dict[str, Any]) -> CallToolResult:
    async def scenario() -> CallToolResult:
        async with Client(server) as client:
            return await client.call_tool(name, arguments)

    return run(scenario(), timeout_s=30)


def text_of(answer: CallToolResult) -> str:
    first = answer.content[0]
    assert isinstance(first, TextContent)
    return first.text


def error_of(answer: CallToolResult) -> ToolError:
    assert answer.is_error is True
    assert answer.structured_content is None
    return ToolError.model_validate_json(text_of(answer))


@pytest.mark.parametrize(
    ("name", "arguments", "error", "expected"),
    [
        (
            "observe",
            {"handle": HANDLE},
            PlayerRefusal("unknown_handle"),
            {"kind": "error", "code": "unknown_handle"},
        ),
        (
            "act",
            {"handle": HANDLE, "act": {"kind": "say", "to": "E1", "words": "hello"}},
            PlayerRefusal("unknown_ref"),
            {"kind": "error", "code": "unknown_ref"},
        ),
        (
            "seats",
            {},
            PlayerRefusal("not_connected", "the server could not be reached"),
            {"kind": "error", "code": "not_connected"},
        ),
        (
            "wait",
            {"handle": HANDLE, "until": {"kind": "any_event"}},
            SeatLostRefusal("taken_over"),
            {"kind": "error", "code": "seat_lost", "seat_lost": "taken_over"},
        ),
        (
            "join",
            {"seat": "visitor", "nickname": "Ada"},
            JoinRefused("not_granted"),
            {"kind": "error", "code": "join_refused", "join": "not_granted"},
        ),
    ],
)
def test_each_refusal_is_a_typed_error_result(
    name: str, arguments: dict[str, Any], error: BaseException, expected: dict[str, str]
) -> None:
    operation = arguments["act"]["kind"] if name == "act" else name
    answer = call(failing(operation, error), name, arguments)
    assert error_of(answer).model_dump(exclude_none=True) == expected


def test_arguments_the_schema_refuses_are_invalid_arguments() -> None:
    answer = call(failing("observe", PlayerRefusal("unknown_handle")), "observe", {})
    assert error_of(answer).code == "invalid_arguments"


def test_an_unexpected_exception_is_internal_and_only_its_type_is_logged(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The message of an unexpected exception may hold anything: it is never sent, and never logged."""
    secret = "SECRET-IN-A-MESSAGE-91c2"
    answer = call(
        failing("stop", RuntimeError(secret)), "act", {"handle": HANDLE, "act": {"kind": "stop"}}
    )
    assert error_of(answer).model_dump(exclude_none=True) == {"kind": "error", "code": "internal"}
    assert secret not in text_of(answer)
    logged = capsys.readouterr().err
    assert "RuntimeError" in logged
    assert secret not in logged
