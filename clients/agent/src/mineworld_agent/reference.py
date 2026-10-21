"""The adapter's tool reference as Markdown (pr-s27f §5.3): `mineworld-agent schema --markdown`.

It renders the data the adapter already publishes — the tool list of `catalogue_json()`, the error vocabulary of
`errors_json()`, the resources, the resource templates and the `play` prompt — and nothing else, so the page is a
function of the models (`ARC-90`'s drift rule, kept by `tests/test_reference_pages.py`). It imports no `mcp`
(I-d-1): `mcp_server.reference_markdown()` feeds it plain JSON data.

Deterministic: no timestamps, LF only, tools in publication order, fields in schema order, every model once.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any, Final, cast

type Json = Mapping[str, Any]

_REF_PREFIX: Final = "#/$defs/"
_NO_DESCRIPTION: Final = "—"
"""What a field without a description shows (S27-d's descriptions test counts those fields)."""


def model_anchor(name: str) -> str:
    """The heading slug of a model's section (`### Model WorldText` → `model-worldtext`)."""
    return f"model-{name.lower()}"


def _cell(text: str) -> str:
    """Text fit for one table cell: one line, pipes escaped."""
    return " ".join(text.split()).replace("|", "\\|")


def type_of(schema: Json) -> str:
    """A field's type in words, with models linked to their sections."""
    if "$ref" in schema:
        name = str(schema["$ref"]).removeprefix(_REF_PREFIX)
        return f"[`{name}`](#{model_anchor(name)})"
    for union in ("anyOf", "oneOf"):
        if union in schema:
            members = cast(Sequence[Json], schema[union])
            return " or ".join(type_of(member) for member in members)
    if "const" in schema:
        return f"`{_literal(schema['const'])}`"
    if "enum" in schema:
        return "one of " + ", ".join(f"`{_literal(value)}`" for value in schema["enum"])
    kind = schema.get("type")
    if kind == "array":
        items = cast(Json, schema.get("items", {}))
        return f"array of {type_of(items)}" if items else "array"
    if kind == "object" and isinstance(schema.get("additionalProperties"), Mapping):
        return f"object of {type_of(cast(Json, schema['additionalProperties']))}"
    if isinstance(kind, str):
        return kind
    return "any JSON value"


def _literal(value: object) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


_BOUNDS: Final = (
    ("minimum", "≥ {}"),
    ("maximum", "≤ {}"),
    ("exclusiveMinimum", "> {}"),
    ("exclusiveMaximum", "< {}"),
    ("minLength", "length ≥ {}"),
    ("maxLength", "length ≤ {}"),
    ("minItems", "items ≥ {}"),
    ("maxItems", "items ≤ {}"),
    ("pattern", "matches `{}`"),
)


def bounds_of(schema: Json) -> str:
    """The field's bounds and default, as the schema states them."""
    parts = [template.format(schema[key]) for key, template in _BOUNDS if key in schema]
    for member in cast(Sequence[Json], schema.get("anyOf", ())):
        parts += [template.format(member[key]) for key, template in _BOUNDS if key in member]
    if "default" in schema:
        parts.append(f"default `{_literal(schema['default'])}`")
    return "; ".join(parts)


def fields_table(schema: Json) -> list[str]:
    """One row per property of an object schema: field, type, required, bounds, description."""
    properties = cast(Mapping[str, Json], schema.get("properties", {}))
    if not properties:
        return ["No fields."]
    required = set(cast(Sequence[str], schema.get("required", ())))
    lines = [
        "| Field | Type | Required | Bounds and default | Description |",
        "| --- | --- | --- | --- | --- |",
    ]
    for name, field in properties.items():
        description = str(field.get("description", "")) or _NO_DESCRIPTION
        lines.append(
            f"| `{name}` | {_cell(type_of(field))} | {'yes' if name in required else 'no'} | "
            f"{_cell(bounds_of(field))} | {_cell(description)} |"
        )
    return lines


def _models(tools: Sequence[Json], errors: Json) -> dict[str, Json]:
    """Every model any published schema defines, once, sorted by name."""
    found: dict[str, Json] = {}
    schemas: list[Json] = [cast(Json, errors["schema"])]
    for tool in tools:
        schemas += [cast(Json, tool["inputSchema"]), cast(Json, tool.get("outputSchema", {}))]
    for schema in schemas:
        for name, model in cast(Mapping[str, Json], schema.get("$defs", {})).items():
            found.setdefault(name, model)
    return dict(sorted(found.items()))


def _annotations(tool: Json) -> str:
    hints = cast(Json, tool.get("annotations", {}))
    words = [
        word
        for key, word in (
            ("readOnlyHint", "read-only"),
            ("destructiveHint", "destructive"),
            ("idempotentHint", "idempotent"),
        )
        if hints.get(key) is True
    ]
    if hints.get("idempotentHint") is False:
        words.append("not idempotent")
    return ", ".join(words) or "none"


def _tool(tool: Json) -> list[str]:
    name = str(tool["name"])
    lines = [f"## {name}", "", str(tool.get("description", "")).strip(), ""]
    lines += [f"Annotations: {_annotations(tool)}.", "", f"**Arguments of `{name}`**", ""]
    lines += fields_table(cast(Json, tool["inputSchema"]))
    lines += ["", f"**Result of `{name}`**", ""]
    lines += fields_table(cast(Json, tool.get("outputSchema", {})))
    return [*lines, ""]


def _model(name: str, model: Json) -> list[str]:
    lines = [f"### Model {name}", ""]
    description = str(model.get("description", "")).strip()
    if description:
        lines += [" ".join(description.split()), ""]
    if "properties" in model:
        lines += fields_table(model)
    else:
        lines.append(f"Type: {type_of(model)}.")
    return [*lines, ""]


def render(
    *,
    version: str,
    catalogue: Json,
    errors: Json,
    resources: Sequence[Json],
    templates: Sequence[Json],
    prompts: Sequence[Json],
) -> str:
    """The whole page."""
    tools = cast(Sequence[Json], catalogue["tools"])
    lines = [
        "<!-- GENERATED by `mineworld-agent schema --markdown` from mineworld-agent "
        f"{version}. Do not edit: change the model's docstring and regenerate. -->",
        "",
        "# The agent adapter's tools",
        "",
        f"The MCP tools, errors, resources and prompt of `mineworld-agent` {version}, generated from the adapter's "
        "own models: the same schemas `tools/list` publishes and `mineworld-agent schema` prints as JSON "
        "([`agent-tools.json`](agent-tools.json)). Every type that names a model links to its section under "
        "[Models](#models).",
        "",
        "| Tool | Annotations |",
        "| --- | --- |",
    ]
    lines += [f"| [`{tool['name']}`](#{tool['name']}) | {_annotations(tool)} |" for tool in tools]
    lines.append("")
    for tool in tools:
        lines += _tool(tool)
    lines += [
        "## Errors",
        "",
        "A refused call is an `isError` result whose text is a `ToolError`.",
        "",
    ]
    lines += [f"- {_cell(str(row))}" for row in cast(Sequence[str], errors["mapping"])]
    lines += ["", "**Fields of `ToolError`**", ""]
    lines += fields_table(cast(Json, errors["schema"]))
    lines += [
        "",
        "## Resources",
        "",
        "| URI | Name | Type | Description |",
        "| --- | --- | --- | --- |",
    ]
    for resource in resources:
        lines.append(
            f"| `{resource['uri']}` | {resource['name']} | {resource.get('mimeType', '')} | "
            f"{_cell(str(resource.get('description', '')))} |"
        )
    lines += ["", "Resource templates:", "", "| URI template | Name | Type | Description |"]
    lines.append("| --- | --- | --- | --- |")
    for template in templates:
        lines.append(
            f"| `{template['uriTemplate']}` | {template['name']} | {template.get('mimeType', '')} | "
            f"{_cell(str(template.get('description', '')))} |"
        )
    lines += ["", "## Prompt", ""]
    for prompt in prompts:
        lines += [f"`{prompt['name']}`: {_cell(str(prompt.get('description', '')))}", ""]
        lines += ["```text", str(prompt["text"]).rstrip("\n"), "```", ""]
    lines += ["## Models", ""]
    for name, model in _models(tools, errors).items():
        lines += _model(name, model)
    return "\n".join(lines).rstrip("\n") + "\n"
