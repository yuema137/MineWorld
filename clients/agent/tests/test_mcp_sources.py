"""One server for both transports: a `Player` per request (pr-s27e §5.2, C3), in process.

`Adapter.serve_players` answers each tool call and resource read with the `Player` its source chose for that request;
the list endpoints and the prompt never ask the source. Over stdio `Adapter.server` passes a constant source, and
S27-d's own files (`test_mcp_tools.py`, `test_mcp_errors.py`, `test_mcp_resources.py`) run against it unchanged."""

from __future__ import annotations

from typing import Any, cast

from mcp.client.client import Client
from mcp.server.context import ServerRequestContext
from support import run

from mineworld_agent.mcp_server import Adapter
from mineworld_agent.models import LeaveArgs, LeaveResult, PlayerRefusal, ToolError
from mineworld_agent.player import Player, PlayerConfig

CONFIG = PlayerConfig(server="ws://127.0.0.1:7878/ws")
HANDLE = "0123456789abcdef0123456789abcdef"


class Recorder:
    """A `Player` double that records the handles it was asked to leave."""

    def __init__(self) -> None:
        self.left: list[str] = []

    async def leave(self, args: LeaveArgs) -> LeaveResult:
        self.left.append(args.handle)
        return LeaveResult()

    def active(self) -> int:
        return 0

    async def close(self) -> None:
        return None


def test_each_call_is_answered_by_the_player_the_source_chose() -> None:
    first, second = Recorder(), Recorder()
    order = [first, second, second, first]
    asked: list[int] = []

    async def source(_ctx: ServerRequestContext[Any]) -> Player:
        asked.append(1)
        return cast(Player, order[len(asked) - 1])

    async def scenario() -> None:
        server = Adapter().serve_players(source, CONFIG)
        async with Client(server) as client:
            for _ in order:
                answer = await client.call_tool("leave", {"handle": HANDLE})
                assert answer.is_error is not True

    run(scenario(), timeout_s=30)
    assert len(first.left) == 2 and len(second.left) == 2


def test_the_list_endpoints_and_the_prompt_never_ask_the_source() -> None:
    asked: list[int] = []

    async def source(_ctx: ServerRequestContext[Any]) -> Player:
        asked.append(1)
        return cast(Player, Recorder())

    async def scenario() -> None:
        server = Adapter().serve_players(source, CONFIG)
        async with Client(server) as client:
            await client.list_tools()
            await client.list_resources()
            await client.list_resource_templates()
            await client.list_prompts()
            await client.get_prompt("play")
            await client.read_resource("mineworld://reference/agent-tools")

    run(scenario(), timeout_s=30)
    assert asked == []


def test_a_refusing_source_is_a_typed_tool_error() -> None:
    """A source that raises `PlayerRefusal` is mapped by `failure_of`, like any `Player` refusal."""

    async def source(_ctx: ServerRequestContext[Any]) -> Player:
        raise PlayerRefusal("not_connected", "no credential")

    async def scenario() -> str:
        server = Adapter().serve_players(source, CONFIG)
        async with Client(server) as client:
            answer = await client.call_tool("leave", {"handle": HANDLE})
        assert answer.is_error is True
        return str(getattr(answer.content[0], "text", ""))

    error = ToolError.model_validate_json(run(scenario(), timeout_s=30))
    assert error.code == "not_connected"
