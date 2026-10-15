"""The thirteen tools as an MCP client sees them, in process (pr-s27d §§5.2, 8 I-d-2 … I-d-4, I-d-9).

The published schemas are the `Player`'s models, exactly; every field of every result reaches the agent through
some tool's `outputSchema`; the tool list is closed; a call returns the model as structured content and its text;
bad arguments answer `invalid_arguments` without echoing a value; a result over 8 KiB is a defect, not a reply."""

from __future__ import annotations

from typing import Any, cast

from mcp.client.client import Client
from mcp.server.lowlevel import Server
from mcp.types import CallToolResult, TextContent
from mcp_double import fake_player
from support import run

from mineworld_agent.mcp_server import OPERATIONS, Adapter, describe, reachable_models
from mineworld_agent.models import (
    EventLine,
    EventsResult,
    LeaveResult,
    SeatsResult,
    ToolError,
    WorldLine,
    WorldText,
    render_text,
)
from mineworld_agent.player import PlayerConfig

HANDLE = "0123456789abcdef0123456789abcdef"
CANARY = "CANARY-7f3a9c"
CONFIG = PlayerConfig(server="ws://127.0.0.1:7878/ws")
EXPECTED_ORDER = [
    "seats",
    "join",
    "observe",
    "inspect",
    "events",
    "wait",
    "say",
    "walk_to",
    "move",
    "stop",
    "attempt",
    "clock",
    "leave",
]


def serve(answers: dict[str, Any]) -> tuple[Adapter, Server[Any]]:
    adapter = Adapter()
    return adapter, adapter.server(fake_player(answers), CONFIG)


def test_the_tool_list_is_closed() -> None:
    """I-d-4 (mutation 5): exactly the thirteen names of §5.2, in that order, and no other tool."""

    async def scenario() -> list[str]:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return [tool.name for tool in listed.tools]

    assert run(scenario(), timeout_s=30) == EXPECTED_ORDER
    assert [operation.name for operation in OPERATIONS] == EXPECTED_ORDER


def test_the_published_schemas_are_the_models() -> None:
    """I-d-2 (R-S27d-c-1, F-S27d-1): each tool's inputSchema and outputSchema equal its models' schemas, unnested."""

    async def scenario() -> dict[str, tuple[dict[str, Any], dict[str, Any] | None]]:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return {tool.name: (tool.input_schema, tool.output_schema) for tool in listed.tools}

    published = run(scenario(), timeout_s=30)
    for operation in OPERATIONS:
        input_schema, output_schema = published[operation.name]
        assert input_schema == operation.args.model_json_schema(), operation.name
        assert output_schema == operation.result.model_json_schema(), operation.name


def _pairs(schema: dict[str, Any]) -> set[tuple[str, str]]:
    """(model name, field name) for the root and every definition of an output schema."""
    found: set[tuple[str, str]] = set()
    root = schema.get("title")
    found |= {(str(root), field) for field in schema.get("properties", {})}
    for name, definition in schema.get("$defs", {}).items():
        found |= {(name, field) for field in definition.get("properties", {})}
    return found


def test_every_player_field_reaches_the_agent() -> None:
    """I-d-3 (the operator's principle): every field of every model the results reach is in some outputSchema."""

    async def scenario() -> list[dict[str, Any]]:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return [tool.output_schema or {} for tool in listed.tools]

    published: set[tuple[str, str]] = set()
    for schema in run(scenario(), timeout_s=30):
        published |= _pairs(schema)
    missing: list[str] = []
    for operation in OPERATIONS:
        for model in reachable_models(operation.result):
            for field in model.model_fields:
                if (model.__name__, field) not in published:
                    missing.append(f"{model.__name__}.{field}")
    assert sorted(set(missing)) == [], "fields no tool's outputSchema reaches"


def test_descriptions_name_the_errors_and_the_error_schema() -> None:
    """Each description lists the codes its tool can raise, and says where the error schema is (§5.2, §5.5)."""

    async def scenario() -> list[tuple[str, str]]:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return [(tool.name, tool.description or "") for tool in listed.tools]

    for name, description in run(scenario(), timeout_s=30):
        operation = next(op for op in OPERATIONS if op.name == name)
        assert "Errors: invalid_arguments" in description, name
        for code in operation.errors:
            assert code in description, (name, code)
        assert "mineworld://reference/agent-errors" in description, name
        assert description == describe(operation), name


def test_a_call_returns_the_model_and_its_text() -> None:
    """A success is the `Player`'s model as structured content, and its `render_text` as the text."""
    result = LeaveResult()

    async def scenario() -> CallToolResult:
        _, server = serve({"leave": result})
        async with Client(server) as client:
            return await client.call_tool("leave", {"handle": HANDLE})

    answer = run(scenario(), timeout_s=30)
    assert answer.is_error is not True
    assert answer.structured_content == result.model_dump(mode="json")
    assert isinstance(answer.content[0], TextContent)
    assert answer.content[0].text == render_text(result)


def test_bad_arguments_answer_invalid_arguments_without_echoing_a_value() -> None:
    """A wrong type, a bad pattern and an unknown key all answer `invalid_arguments`; no value is echoed, and an
    unknown key's name is not echoed either (an agent may send a secret as a key)."""

    async def scenario(arguments: dict[str, Any]) -> CallToolResult:
        _, server = serve({})
        async with Client(server) as client:
            return await client.call_tool("observe", arguments)

    for arguments in (
        {"handle": HANDLE, "since_view": CANARY},
        {"handle": CANARY},
        {"handle": HANDLE, CANARY: 1},
    ):
        answer = run(scenario(arguments), timeout_s=30)
        assert answer.is_error is True
        assert answer.structured_content is None
        text = answer.content[0]
        assert isinstance(text, TextContent)
        error = ToolError.model_validate_json(text.text)
        assert error.code == "invalid_arguments"
        assert error.detail is not None
        assert CANARY not in text.text, arguments


def test_a_result_over_the_byte_cap_is_an_internal_defect() -> None:
    """§5.2's check: a result over `max_result_bytes` is never sent; the answer is `internal`."""
    words = WorldText(text="a" * 480, by="E1", kind="speech")
    events = [
        EventLine(
            ref=f"F{number}",
            at=number,
            event_type="spoke",
            subjects=["E1"],
            place="E2",
            words=words,
        )
        for number in range(1, 21)
    ]
    oversized = EventsResult(
        events=events,
        next=None,
        unread=0,
        dropped=0,
        at=1,
        time_scale=1,
        paused=False,
        world_seconds_since_last_call=0,
    )

    async def scenario() -> CallToolResult:
        _, server = serve({"events": oversized})
        async with Client(server) as client:
            return await client.call_tool("events", {"handle": HANDLE})

    answer = run(scenario(), timeout_s=30)
    assert answer.is_error is True
    assert answer.structured_content is None
    text = answer.content[0]
    assert isinstance(text, TextContent)
    assert ToolError.model_validate_json(text.text).code == "internal"


def test_the_seats_answer_is_a_structured_model() -> None:
    """A second success shape: the world line and the seat list, as JSON the schema types."""
    seats = SeatsResult(
        world=WorldLine(
            protocol=1,
            at=0,
            time_scale=1,
            paused=False,
            entities=3,
            systems=["conversation"],
            clients=1,
        ),
        seats=["visitor"],
    )

    async def scenario() -> CallToolResult:
        _, server = serve({"seats": seats})
        async with Client(server) as client:
            return await client.call_tool("seats", {})

    answer = run(scenario(), timeout_s=30)
    assert answer.is_error is not True
    structured = cast(dict[str, Any], answer.structured_content)
    assert structured["seats"] == ["visitor"]
    assert structured == seats.model_dump(mode="json")
