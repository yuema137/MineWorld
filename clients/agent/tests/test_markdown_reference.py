"""The Markdown renderer of the tool reference (pr-s27f §5.3, §7 "Unit").

The page's tool headings are the published tool names, in publication order, so the guide's links land; every
model a schema links to has its section; a field without a description shows the empty marker; and the output does
not depend on anything but its input."""

from __future__ import annotations

import json
import re
from typing import Any

from mineworld_agent import reference
from mineworld_agent.mcp_server import OPERATIONS, catalogue_json, errors_json, reference_markdown

_SECOND_LEVEL = re.compile(r"^## (.+)$", re.MULTILINE)
_MODEL_LINK = re.compile(r"\]\(#(model-[a-z0-9-]+)\)")
_MODEL_HEADING = re.compile(r"^### Model (\S+)$", re.MULTILINE)


def test_the_tool_headings_are_the_tool_names_in_publication_order() -> None:
    headings = _SECOND_LEVEL.findall(reference_markdown())
    tools = [operation.name for operation in OPERATIONS]
    assert headings[: len(tools)] == tools
    assert headings[len(tools) :] == ["Errors", "Resources", "Prompt", "Models"]


def test_every_linked_model_has_its_section() -> None:
    page = reference_markdown()
    linked = set(_MODEL_LINK.findall(page))
    sections = {reference.model_anchor(name) for name in _MODEL_HEADING.findall(page)}
    assert linked, "no model is linked"
    assert linked <= sections, sorted(linked - sections)


def test_a_field_without_a_description_shows_the_empty_marker() -> None:
    schema: dict[str, Any] = {
        "properties": {
            "bare": {"type": "integer", "minimum": 1},
            "told": {"type": "string", "description": "Said."},
        },
        "required": ["bare"],
    }
    rows = reference.fields_table(schema)
    assert rows[2] == "| `bare` | integer | yes | ≥ 1 | — |"
    assert rows[3] == "| `told` | string | no |  | Said. |"


def test_the_page_is_a_function_of_its_input() -> None:
    arguments: dict[str, Any] = {
        "version": "9.9.9",
        "catalogue": json.loads(catalogue_json()),
        "errors": json.loads(errors_json()),
        "resources": [],
        "templates": [],
        "prompts": [],
    }
    first = reference.render(**arguments)
    assert first == reference.render(**arguments)
    assert "\r" not in first
    assert first.endswith("\n") and not first.endswith("\n\n")
    assert "mineworld-agent 9.9.9" in first.splitlines()[0]
