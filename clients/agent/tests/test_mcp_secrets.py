"""No secret in any byte the agent or a log sees (INV-S27-11, AC-d-4's in-process half, pr-s27d §§7, I-d-6).

A real `Player` holds a canary invite and is built against a port nothing listens on, so every operation fails
without reaching a world. The canary must appear in no answer, error, resource, prompt, tool list or stderr line.
Each tool is called with arguments the schema accepts, so each call reaches the `Player`."""

from __future__ import annotations

from typing import Any

import pytest
from mcp.client.client import Client
from mcp.shared.exceptions import MCPError
from mcp.types import CallToolResult, ReadResourceResult, TextContent, TextResourceContents
from mineworld_sdk.wire.frames import Invite
from support import run

from mineworld_agent.mcp_server import (
    REFERENCE_ERRORS,
    REFERENCE_TOOLS,
    WORLD_STATUS,
    Adapter,
    inbox_uri,
)
from mineworld_agent.player import Player, PlayerConfig

CANARY = "mwagent-canary-invite-5e0d7a91"
HANDLE = "0123456789abcdef0123456789abcdef"
CONFIG = PlayerConfig(server="ws://127.0.0.1:9/ws")
CALLS: dict[str, dict[str, Any]] = {
    "seats": {},
    "join": {"seat": "visitor", "nickname": "Ada"},
    "observe": {"handle": HANDLE},
    "inspect": {"handle": HANDLE, "ref": "E1"},
    "events": {"handle": HANDLE},
    "wait": {"handle": HANDLE, "until": {"kind": "any_event"}},
    "clock": {"handle": HANDLE},
    "leave": {"handle": HANDLE},
}
ACTS: tuple[dict[str, Any], ...] = (
    {"kind": "say", "to": "E1", "words": "hello"},
    {"kind": "walk_to", "to": {"place": "E1"}},
    {"kind": "move", "to": {"place": "E1", "x_mm": 0, "y_mm": 0, "z_mm": 0}},
    {"kind": "stop"},
    {"kind": "attempt", "view": 1, "offer": 1},
)
"""Every kind of `act`, each with arguments the schema accepts."""


def _answer_text(answer: CallToolResult) -> str:
    first = answer.content[0]
    assert isinstance(first, TextContent)
    return first.text + str(answer.structured_content or "")


def _resource_text(result: ReadResourceResult) -> str:
    content = result.contents[0]
    assert isinstance(content, TextResourceContents)
    return content.text


async def _everything_the_agent_sees() -> list[str]:
    seen: list[str] = []
    adapter = Adapter()
    async with Player(CONFIG, Invite(CANARY)) as player:
        server = adapter.server(player, CONFIG)
        async with Client(server) as client:
            seen.append((await client.list_tools()).model_dump_json(by_alias=True))
            for name, arguments in CALLS.items():
                seen.append(_answer_text(await client.call_tool(name, arguments)))
            for act in ACTS:
                arguments = {"handle": HANDLE, "act": act}
                seen.append(_answer_text(await client.call_tool("act", arguments)))
            for uri in (REFERENCE_TOOLS, REFERENCE_ERRORS, WORLD_STATUS, inbox_uri(HANDLE)):
                try:
                    seen.append(_resource_text(await client.read_resource(uri)))
                except MCPError as error:
                    seen.append(error.error.message)
            prompt = await client.get_prompt("play")
            content = prompt.messages[0].content
            assert isinstance(content, TextContent)
            seen.append(content.text)
    return seen


def test_no_answer_error_resource_prompt_or_tool_list_holds_the_canary(
    capsys: pytest.CaptureFixture[str],
) -> None:
    seen = run(_everything_the_agent_sees(), timeout_s=60)
    assert len(seen) == 1 + len(CALLS) + len(ACTS) + 4 + 1
    for text in seen:
        assert CANARY not in text, text[:200]
    assert CANARY not in capsys.readouterr().err


def test_the_canary_is_not_in_the_schemas_either() -> None:
    """The schemas are the agent's whole view of the surface: a credential has no field in any of them."""

    async def scenario() -> str:
        async with Player(CONFIG, Invite(CANARY)) as player:
            async with Client(Adapter().server(player, CONFIG)) as client:
                return (await client.list_tools()).model_dump_json()

    assert CANARY not in run(scenario(), timeout_s=60)
