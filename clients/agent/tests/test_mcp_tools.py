"""The nine tools as an MCP client sees them, in process (pr-s27d §§5.2, 8 I-d-2 … I-d-4, I-d-9).

The published schemas are the `Player`'s models, exactly; every field of every result reaches the agent through
some tool's `outputSchema`; the tool list is closed; a call returns the model as structured content and its text;
bad arguments answer `invalid_arguments` without echoing a value; a result over 8 KiB is a defect, not a reply."""

from __future__ import annotations

from typing import Any, cast

import pytest
from mcp import types
from mcp.client.client import Client
from mcp.server.context import ServerRequestContext
from mcp.server.lowlevel import Server
from mcp.shared.exceptions import MCPError
from mcp.types import CallToolResult, TextContent
from mcp_double import fake_player
from recorded import NAMES, observation
from support import run

from mineworld_agent.mcp_server import (
    ACTS,
    OPERATIONS,
    Adapter,
    act_kinds,
    describe,
    published_schema,
    reachable_models,
)
from mineworld_agent.models import (
    Accepted,
    ActResult,
    AgentModel,
    AttemptArgs,
    EventLine,
    EventsResult,
    InboxCounts,
    LeaveResult,
    MoveArgs,
    ObserveResult,
    Point,
    SayArgs,
    SeatsResult,
    SleepArgs,
    StopArgs,
    ToolError,
    ToPlace,
    View,
    WalkToArgs,
    WorldLine,
    WorldText,
    render_text,
)
from mineworld_agent.player import PlayerConfig
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, Moment, render, view_of


def whole(name: str) -> View:
    """A recorded market-town observation as the `Player` renders it (as `test_view.py` builds it)."""
    moment = Moment(at=0, time_scale=1, paused=False, since=None)
    seen = render(observation(name), RefTable(), 1, True)
    return view_of(seen, moment, InboxCounts(unread=0, dropped=0), Caps())


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
    "act",
    "clock",
    "leave",
]


def serve(
    answers: dict[str, Any], calls: list[tuple[str, AgentModel]] | None = None
) -> tuple[Adapter, Server[Any]]:
    adapter = Adapter()
    return adapter, adapter.server(fake_player(answers, calls), CONFIG)


def test_the_tool_list_is_closed() -> None:
    """I-d-4 (mutation 5), as pr-s27d1 R-1 amends it: exactly nine names, in that order, and no other tool."""

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
        assert output_schema is not None, operation.name
        assert _only_titles_dropped(operation.args.model_json_schema(), input_schema), (
            operation.name
        )
        assert _only_titles_dropped(operation.result.model_json_schema(), output_schema), (
            operation.name
        )


def _only_titles_dropped(model: object, published: object, under: str = "") -> bool:
    """True when `published` is `model` with `title` keywords removed and nothing else changed (R-d-4): no other
    key is missing, none is added, and every other value is equal. `under` is the keyword `model` sits in, so a
    property *named* `title` (a key of `properties`) is not mistaken for the keyword."""
    if isinstance(model, list) and isinstance(published, list):
        pairs = zip(cast(list[object], model), cast(list[object], published), strict=False)
        same_length = len(cast(list[object], model)) == len(cast(list[object], published))
        return same_length and all(_only_titles_dropped(a, b) for a, b in pairs)
    if isinstance(model, dict) and isinstance(published, dict):
        left, right = cast(dict[str, object], model), cast(dict[str, object], published)
        names = under in ("properties", "$defs")
        if "title" in right and not names:
            return False
        if set(right) - set(left):
            return False
        for key, value in left.items():
            if key not in right:
                if names or key != "title":
                    return False
                continue
            if key in ("default", "const", "enum", "examples"):
                if value != right[key]:
                    return False
            elif not _only_titles_dropped(value, right[key], "" if names else key):
                return False
        return True
    return model == published


def _pairs(schema: dict[str, Any], root: str) -> set[tuple[str, str]]:
    """(model name, field name) for the root and every definition of an output schema."""
    found: set[tuple[str, str]] = set()
    found |= {(root, field) for field in schema.get("properties", {})}
    for name, definition in schema.get("$defs", {}).items():
        found |= {(name, field) for field in definition.get("properties", {})}
    return found


def test_every_player_field_reaches_the_agent() -> None:
    """I-d-3 (the operator's principle): every field of every model the results reach is in some outputSchema."""

    async def scenario() -> dict[str, dict[str, Any]]:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return {tool.name: tool.output_schema or {} for tool in listed.tools}

    published: set[tuple[str, str]] = set()
    schemas = run(scenario(), timeout_s=30)
    for operation in OPERATIONS:
        published |= _pairs(schemas[operation.name], operation.result.__name__)
    missing: list[str] = []
    for operation in OPERATIONS:
        for model in reachable_models(operation.result):
            for field in model.model_fields:
                if (model.__name__, field) not in published:
                    missing.append(f"{model.__name__}.{field}")
    assert sorted(set(missing)) == [], "fields no tool's outputSchema reaches"


def test_the_official_client_validates_recorded_views_against_the_published_schema() -> None:
    """R-d-4: the untitled schemas still validate. The official client checks every structured result against the
    tool's `outputSchema`: a recorded market-town view passes, and the same schema refuses a result without its
    `view`, so the check is live (the client raises rather than delivering it)."""
    observe = next(operation for operation in OPERATIONS if operation.name == "observe")
    results = [ObserveResult(view=whole(name)) for name in NAMES]

    async def accepted(result: ObserveResult) -> CallToolResult:
        _, server = serve({"observe": result})
        async with Client(server) as client:
            return await client.call_tool("observe", {"handle": HANDLE})

    for result in results:
        answer = run(accepted(result), timeout_s=30)
        assert answer.is_error is not True
        assert answer.structured_content == result.model_dump(mode="json")

    async def listed(
        _ctx: ServerRequestContext[Any], _params: types.PaginatedRequestParams | None
    ) -> types.ListToolsResult:
        return types.ListToolsResult(
            tools=[
                types.Tool(
                    name="observe",
                    input_schema=published_schema(observe.args),
                    output_schema=published_schema(observe.result),
                )
            ]
        )

    async def broken(
        _ctx: ServerRequestContext[Any], _params: types.CallToolRequestParams
    ) -> types.CallToolResult:
        return types.CallToolResult(content=[], structured_content={"not_a_view": 1})

    async def refused() -> None:
        server: Server[Any] = Server("broken", on_list_tools=listed, on_call_tool=broken)
        async with Client(server) as client:
            await client.call_tool("observe", {"handle": HANDLE})

    with pytest.raises(ExceptionGroup) as raised:
        run(refused(), timeout_s=30)
    leaves = _leaves(raised.value)
    assert any(
        isinstance(leaf, RuntimeError) and "Invalid structured content" in str(leaf)
        for leaf in leaves
    ), leaves


def _leaves(error: BaseException) -> list[BaseException]:
    """The exceptions inside nested exception groups (the in-process client raises through its task groups)."""
    if isinstance(error, BaseExceptionGroup):
        group = cast(BaseExceptionGroup[BaseException], error)
        return [leaf for inner in group.exceptions for leaf in _leaves(inner)]
    return [error]


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

    async def scenario(tool: str, arguments: dict[str, Any]) -> CallToolResult:
        _, server = serve({})
        async with Client(server) as client:
            return await client.call_tool(tool, arguments)

    for tool, arguments in (
        ("observe", {"handle": HANDLE, "since_view": CANARY}),
        ("observe", {"handle": CANARY}),
        ("observe", {"handle": HANDLE, CANARY: 1}),
        # a discriminated union: pydantic's own message would name the tag that was sent
        ("wait", {"handle": HANDLE, "until": {"kind": CANARY}}),
        ("act", {"handle": HANDLE, "act": {"kind": CANARY, "to": "E1", "words": "hi"}}),
        ("act", {"handle": HANDLE, "act": {"to": CANARY}}),
        ("act", {"handle": HANDLE, "act": {"kind": "say", "to": CANARY, "words": "hi"}}),
        ("act", {"handle": HANDLE, "act": {"kind": "attempt", "view": CANARY, "offer": 1}}),
        ("act", {"handle": HANDLE, "act": CANARY}),
    ):
        answer = run(scenario(tool, arguments), timeout_s=30)
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


def _act_result(at: int, view: View | None = None) -> ActResult:
    return ActResult(
        outcome=Accepted(events=[]),
        view=view,
        at=at,
        time_scale=1,
        paused=False,
        world_seconds_since_last_call=None,
    )


ACT_CALLS: dict[str, tuple[dict[str, Any], AgentModel]] = {
    "say": (
        {"kind": "say", "to": "E2", "words": "hello"},
        SayArgs(handle=HANDLE, to="E2", words="hello"),
    ),
    "walk_to": (
        {"kind": "walk_to", "to": {"place": "E9"}},
        WalkToArgs(handle=HANDLE, to=ToPlace(place="E9")),
    ),
    "move": (
        {"kind": "move", "to": {"place": "E9", "x_mm": 250, "y_mm": -40}, "facing_deg": 90},
        MoveArgs(handle=HANDLE, to=Point(place="E9", x_mm=250, y_mm=-40), facing_deg=90),
    ),
    "stop": ({"kind": "stop"}, StopArgs(handle=HANDLE)),
    "attempt": (
        {"kind": "attempt", "view": 3, "offer": 2},
        AttemptArgs(handle=HANDLE, view=3, offer=2),
    ),
    "sleep": ({"kind": "sleep", "minutes": 480}, SleepArgs(handle=HANDLE, minutes=480)),
}
"""Each `act` kind, the arguments an agent sends, and the library call it must become."""


def test_the_act_kinds_are_the_six_operations() -> None:
    """The dispatch table and the published union name the same six kinds, in the same order."""
    assert act_kinds() == tuple(ACTS) == tuple(ACT_CALLS)


def test_each_kind_reaches_its_own_operation() -> None:
    """pr-s27d1 AC-4 (R-3, mutation M-1): each kind calls its own `Player` operation, with the library's argument
    model built from the handle and that kind's fields, and answers that operation's result. Each operation
    answers a distinct result, so a kind wired to another operation is seen."""
    answers: dict[str, Any] = {
        kind: _act_result(at=number) for number, kind in enumerate(ACT_CALLS, start=1)
    }

    async def scenario(act: dict[str, Any]) -> tuple[CallToolResult, list[tuple[str, AgentModel]]]:
        calls: list[tuple[str, AgentModel]] = []
        _, server = serve(answers, calls)
        async with Client(server) as client:
            answer = await client.call_tool("act", {"handle": HANDLE, "act": act})
        return answer, calls

    for kind, (act, expected) in ACT_CALLS.items():
        answer, calls = run(scenario(act), timeout_s=30)
        assert answer.is_error is not True, (kind, answer.content)
        assert calls == [(kind, expected)], kind
        assert type(calls[0][1]) is type(expected), kind
        assert answer.structured_content == answers[kind].model_dump(mode="json"), kind


def test_the_official_client_validates_an_act_result() -> None:
    """pr-s27d1 AC-5: an `act` result holding a fresh recorded view passes the official client's validation
    against the one published `ActResult` schema."""
    result = _act_result(at=7, view=whole(NAMES[0]))

    async def scenario() -> CallToolResult:
        _, server = serve({"attempt": result})
        async with Client(server) as client:
            return await client.call_tool(
                "act", {"handle": HANDLE, "act": {"kind": "attempt", "view": 1, "offer": 1}}
            )

    answer = run(scenario(), timeout_s=30)
    assert answer.is_error is not True
    assert answer.structured_content == result.model_dump(mode="json")


def test_an_unknown_kind_is_named_from_the_schema_alone() -> None:
    """pr-s27d1 §4: an unknown `kind` answers `invalid_arguments` at `act`, listing the six kinds."""

    async def scenario() -> CallToolResult:
        _, server = serve({})
        async with Client(server) as client:
            return await client.call_tool("act", {"handle": HANDLE, "act": {"kind": CANARY}})

    answer = run(scenario(), timeout_s=30)
    text = answer.content[0]
    assert isinstance(text, TextContent)
    error = ToolError.model_validate_json(text.text)
    assert error.code == "invalid_arguments"
    assert error.detail is not None
    assert error.detail.startswith("act: unknown kind; expected one of:"), error.detail
    for kind in ACTS:
        assert f"'{kind}'" in error.detail, kind
    assert CANARY not in text.text


def test_unknown_names_are_not_echoed() -> None:
    """pr-s27d1 R-4: an unknown tool, prompt or resource is refused without repeating what the agent sent."""

    async def scenario() -> list[str]:
        _, server = serve({})
        messages: list[str] = []
        async with Client(server) as client:
            for attempt in (
                client.call_tool(CANARY, {}),
                client.get_prompt(CANARY),
                client.read_resource(f"mineworld://{CANARY}"),
                client.read_resource(f"mineworld://seat/{'0' * 31}{CANARY}/inbox"),
            ):
                try:
                    await attempt
                except MCPError as error:
                    messages.append(error.error.message)
        return messages

    messages = run(scenario(), timeout_s=30)
    assert messages == ["unknown tool", "unknown prompt", "unknown resource", "unknown resource"]


def test_leave_is_destructive() -> None:
    """pr-s27d1 R-5: `leave` gives the seat back, so it is published as destructive and not idempotent."""

    async def scenario() -> types.Tool:
        _, server = serve({})
        async with Client(server) as client:
            listed = await client.list_tools()
        return next(tool for tool in listed.tools if tool.name == "leave")

    annotations = run(scenario(), timeout_s=30).annotations
    assert annotations is not None
    assert annotations.destructive_hint is True
    assert annotations.idempotent_hint is False
