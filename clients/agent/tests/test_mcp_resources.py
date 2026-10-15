"""The resources, the `play` prompt and the inbox subscription (pr-s27d sections 5.5 to 5.7, 7; golden `play_prompt.txt`).

Resources are produced in process (F-S27d-3); the reference resources are the tool list and the error schema; an
unknown handle is a resource-not-found error. The subscription publishes a cue for the handle whose inbox gained
facts, and only that one, with no content (X-7)."""

from __future__ import annotations

import asyncio
import json
from collections.abc import Callable, Coroutine
from pathlib import Path
from typing import Any, cast

from mcp.client.client import Client
from mcp.shared.exceptions import MCPError
from mcp.shared.subscriptions import ResourceUpdated
from mcp.types import ReadResourceResult, TextResourceContents
from mcp_double import fake_player
from support import run

from mineworld_agent.mcp_server import (
    INBOX_TEMPLATE,
    PLAY_PROMPT,
    REFERENCE_ERRORS,
    REFERENCE_TOOLS,
    RESOURCE_NOT_FOUND,
    WORLD_STATUS,
    Adapter,
    catalogue_json,
    inbox_uri,
)
from mineworld_agent.models import (
    InboxStatus,
    PlayerRefusal,
    SeatsResult,
    ToolError,
    WorldLine,
)
from mineworld_agent.player import PlayerConfig

HANDLE = "0123456789abcdef0123456789abcdef"
OTHER = "fedcba9876543210fedcba9876543210"
CONFIG = PlayerConfig(server="ws://127.0.0.1:7878/ws")
GOLDEN = Path(__file__).resolve().parent / "golden" / "play_prompt.txt"


def seats_answer() -> SeatsResult:
    world = WorldLine(
        protocol=1,
        at=0,
        time_scale=1,
        paused=False,
        entities=3,
        systems=["conversation"],
        clients=1,
    )
    return SeatsResult(world=world, seats=["visitor"])


def test_the_resources_and_the_template_are_listed() -> None:
    async def scenario() -> tuple[list[str], list[str]]:
        _, server = _serve({})
        async with Client(server) as client:
            resources = await client.list_resources()
            templates = await client.list_resource_templates()
        return (
            sorted(str(resource.uri) for resource in resources.resources),
            [template.uri_template for template in templates.resource_templates],
        )

    listed, templates = run(scenario(), timeout_s=30)
    assert listed == sorted([REFERENCE_TOOLS, REFERENCE_ERRORS, WORLD_STATUS])
    assert templates == [INBOX_TEMPLATE]


def test_the_reference_resources_are_the_tool_list_and_the_error_schema() -> None:
    async def scenario() -> tuple[str, str]:
        _, server = _serve({})
        async with Client(server) as client:
            tools = await client.read_resource(REFERENCE_TOOLS)
            errors = await client.read_resource(REFERENCE_ERRORS)
        return _text(tools), _text(errors)

    tools, errors = run(scenario(), timeout_s=30)
    assert tools == catalogue_json()
    assert json.loads(errors)["schema"] == ToolError.model_json_schema()


def test_the_world_status_resource_is_a_fresh_seats_call() -> None:
    async def scenario() -> str:
        _, server = _serve({"seats": seats_answer()})
        async with Client(server) as client:
            return _text(await client.read_resource(WORLD_STATUS))

    assert run(scenario(), timeout_s=30) == seats_answer().model_dump_json()


def test_the_inbox_resource_is_the_status_of_its_handle() -> None:
    status = InboxStatus(unread=2, dropped=0, newest="F2")

    async def scenario() -> str:
        _, server = _serve({"inbox": status})
        async with Client(server) as client:
            return _text(await client.read_resource(inbox_uri(HANDLE)))

    assert run(scenario(), timeout_s=30) == status.model_dump_json()


def test_an_unknown_handle_is_a_resource_not_found_error() -> None:
    async def scenario() -> None:
        _, server = _serve({"inbox": PlayerRefusal("unknown_handle")})
        async with Client(server) as client:
            await client.read_resource(inbox_uri(HANDLE))

    assert _mcp_error_of(scenario).error.code == RESOURCE_NOT_FOUND


def test_a_malformed_inbox_uri_is_a_resource_not_found_error() -> None:
    async def scenario() -> None:
        _, server = _serve({})
        async with Client(server) as client:
            await client.read_resource("mineworld://seat/not-a-handle/inbox")

    assert _mcp_error_of(scenario).error.code == RESOURCE_NOT_FOUND


def test_the_play_prompt_equals_its_golden_and_fits_1200_bytes() -> None:
    golden = GOLDEN.read_text(encoding="utf-8")
    assert PLAY_PROMPT == golden
    assert len(PLAY_PROMPT.encode("utf-8")) <= 1200

    async def scenario() -> str:
        _, server = _serve({})
        async with Client(server) as client:
            prompt = await client.get_prompt("play")
        content = prompt.messages[0].content
        assert getattr(content, "type", None) == "text"
        return str(getattr(content, "text", ""))

    assert run(scenario(), timeout_s=30) == golden


def test_the_inbox_cue_reaches_the_listener_of_that_handle_only() -> None:
    """A cue for `HANDLE` reaches a listener on its URI, carries no content, and never reaches another handle."""

    async def scenario() -> tuple[ResourceUpdated, bool]:
        adapter, server = _serve({})
        async with Client(server) as client:
            async with client.listen(resource_subscriptions=[inbox_uri(HANDLE)]) as subscription:
                adapter.on_inbox(HANDLE)
                event = await asyncio.wait_for(subscription.__anext__(), timeout=10)
            async with client.listen(resource_subscriptions=[inbox_uri(OTHER)]) as other:
                adapter.on_inbox(HANDLE)
                try:
                    await asyncio.wait_for(other.__anext__(), timeout=0.3)
                    heard = True
                except TimeoutError:
                    heard = False
        assert isinstance(event, ResourceUpdated)
        return event, heard

    event, heard = run(scenario(), timeout_s=30)
    assert event.uri == inbox_uri(HANDLE)
    assert heard is False


def _serve(answers: dict[str, Any]) -> tuple[Adapter, Any]:
    adapter = Adapter()
    return adapter, adapter.server(fake_player(answers), CONFIG)


def _find_mcp_error(error: BaseException) -> MCPError | None:
    """The first `MCPError` in an exception, through nested exception groups."""
    if isinstance(error, MCPError):
        return error
    if isinstance(error, BaseExceptionGroup):
        for inner in cast("BaseExceptionGroup[BaseException]", error).exceptions:
            found = _find_mcp_error(inner)
            if found is not None:
                return found
    return None


def _mcp_error_of(scenario: Callable[[], Coroutine[Any, Any, None]]) -> MCPError:
    """The MCP error a scenario ends with. The in-process client re-raises a handler's `MCPError` inside its task
    groups; on the wire the same error is a JSON-RPC error response."""
    try:
        run(scenario(), timeout_s=30)
    except BaseException as error:
        found = _find_mcp_error(error)
        if found is None:
            raise
        return found
    raise AssertionError("the scenario returned: no error was raised")


def _text(result: ReadResourceResult) -> str:
    """The text of a read resource's first content (the resources here are all text)."""
    content = result.contents[0]
    assert isinstance(content, TextResourceContents)
    return content.text
