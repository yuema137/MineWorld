"""The generated reference for the operator's cognition configuration (S25 PR 25b; step-25 §5).

Writes `cognition-toml.md` and `schema/cognition-toml.schema.json` into `--out`, from the Pydantic models in
`mineworld_cognition.config`. Nothing is read from a model, a key, a world or a network: the generator only
imports the models and runs its own examples through the same loader an operator's file goes through.

    uv run --locked python cognition/lm-controller/tools/reference.py --out docs/reference

Two refusals keep the page honest. A field with no description fails generation, naming the model and the
field. An example that the loader refuses fails generation, naming the example.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from dataclasses import dataclass
from importlib.metadata import version
from pathlib import Path
from typing import Any, get_args, get_origin

from pydantic import BaseModel

from mineworld_cognition import config
from mineworld_cognition.config import CognitionConfig, ConfigError

PACKAGE = "mineworld-cognition"
PAGE = "cognition-toml.md"
SCHEMA = "schema/cognition-toml.schema.json"
COMMAND = "uv run --locked python cognition/lm-controller/tools/reference.py --out docs/reference"
JSON_SCHEMA_DRAFT = "https://json-schema.org/draft/2020-12/schema"

JsonObject = dict[str, Any]


class GenerationError(Exception):
    """The reference cannot be generated: a field without a description, or an example the loader refuses."""


@dataclass(frozen=True)
class Example:
    """One complete `cognition.toml`, printed on the page and loaded to prove it."""

    title: str
    text: str


EXAMPLES: tuple[Example, ...] = (
    Example(
        "A local model",
        """[recording]
mode = "live"

[tiers]
social = "local"

[backends.local]
base_url = "http://127.0.0.1:11434/v1"
model = "qwen3.5:4b"
reasoning = "off"
""",
    ),
    Example(
        "A hosted provider, a budget and a key file",
        """[recording]
mode = "live"

[tiers]
social = "deepseek"

[budgets]
calls_per_wall_hour = 20
tokens_per_wall_day = 30000
ledger = "cognition-state/budget.sqlite"

[secrets]
env_file = "mineworld.env"

[backends.deepseek]
preset = "deepseek"
model = "your-model-name"
""",
    ),
    Example(
        "Claude, through an API key",
        """[recording]
mode = "live"

[tiers]
social = "claude"

[backends.claude]
kind = "anthropic"
base_url = "https://api.anthropic.com"
model = "your-model-name"
key_env = "ANTHROPIC_API_KEY"
""",
    ),
    Example(
        "Replay a recording, with no network",
        """[recording]
mode = "replay"
cassette = "cassettes/recorded.jsonl"

[tiers]
social = "local"

[backends.local]
base_url = "http://127.0.0.1:11434/v1"
model = "qwen3.5:4b"
""",
    ),
)


def nested_models(annotation: object) -> list[type[BaseModel]]:
    """Every Pydantic model an annotation names, through unions, `Annotated` and generic arguments."""
    if (
        get_origin(annotation) is None
        and isinstance(annotation, type)
        and issubclass(annotation, BaseModel)
    ):
        return [annotation]
    return [model for argument in get_args(annotation) for model in nested_models(argument)]


def undescribed_fields(model: type[BaseModel]) -> list[str]:
    """`Model.field` for every field, of this model and of every model it nests, that has no description."""
    missing: list[str] = []
    seen: set[type[BaseModel]] = set()
    pending: list[type[BaseModel]] = [model]
    while pending:
        current = pending.pop(0)
        if current in seen:
            continue
        seen.add(current)
        for name, field in current.model_fields.items():
            if not field.description:
                missing.append(f"{current.__name__}.{name}")
            pending.extend(nested_models(field.annotation))
    return missing


def check_described(model: type[BaseModel]) -> None:
    missing = undescribed_fields(model)
    if missing:
        raise GenerationError(
            "fields without a description (add Field(description=...) and regenerate): "
            + ", ".join(missing)
        )


def check_examples() -> None:
    """Every example must pass `config.load`, the loader an operator's file goes through."""
    with tempfile.TemporaryDirectory() as directory:
        for index, example in enumerate(EXAMPLES):
            path = Path(directory) / f"example-{index}.toml"
            path.write_text(example.text, encoding="utf-8", newline="\n")
            try:
                config.load(path)
            except ConfigError as error:
                raise GenerationError(
                    f"example {example.title!r} is refused by the loader: {error}"
                ) from None


def schema_document() -> JsonObject:
    check_described(CognitionConfig)
    return {"$schema": JSON_SCHEMA_DRAFT, **CognitionConfig.model_json_schema()}


def resolve(node: JsonObject, defs: JsonObject) -> JsonObject:
    reference = node.get("$ref")
    if isinstance(reference, str):
        return dict(defs[reference.removeprefix("#/$defs/")])
    return node


def cell(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def type_text(node: JsonObject) -> str:
    if "anyOf" in node:
        options = [option for option in node["anyOf"] if option != {"type": "null"}]
        return f"{type_text(options[0])}, or unset" if len(options) == 1 else "see the description"
    if "enum" in node:
        return "one of " + ", ".join(f"`{value}`" for value in node["enum"])
    kinds = {
        "string": "text",
        "integer": "whole number",
        "object": "table",
        "boolean": "true or false",
    }
    return kinds.get(str(node.get("type")), "see the description")


def default_text(node: JsonObject) -> str:
    if "default" not in node:
        return "none"
    if node["default"] is None:
        return "unset"
    return f"`{json.dumps(node['default'])}`"


def bounds_text(node: JsonObject) -> str:
    parts: list[str] = []
    options = [option for option in node.get("anyOf", [node]) if option != {"type": "null"}]
    bounded = options[0] if options else node
    if "minimum" in bounded:
        parts.append(f">= {bounded['minimum']}")
    if "maximum" in bounded:
        parts.append(f"<= {bounded['maximum']}")
    if "minLength" in bounded:
        parts.append(f"length >= {bounded['minLength']}")
    if "maxLength" in bounded:
        parts.append(f"length <= {bounded['maxLength']}")
    return ", ".join(parts) if parts else "none"


def field_table(body: JsonObject, defs: JsonObject) -> list[str]:
    required = set(body.get("required", []))
    lines = [
        "| Key | Type | Required | Default | Bounds | Description |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for key, raw in body["properties"].items():
        node = resolve(raw, defs) if "$ref" in raw else raw
        description = cell(str(raw.get("description", node.get("description", ""))))
        lines.append(
            f"| `{key}` | {cell(type_text(node))} | {'yes' if key in required else 'no'} "
            f"| {default_text(raw)} | {cell(bounds_text(raw))} | {description} |"
        )
    return lines


def mapping_table(node: JsonObject) -> list[str]:
    """A table whose keys are an enumerated set and whose values are strings (`[tiers]`)."""
    keys: list[str] = node["propertyNames"]["enum"]
    lines = [
        "| Key | Type | Required | Default | Bounds | Description |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for key in keys:
        lines.append(
            f"| `{key}` | text: a backend name, or `none` | no | unset | none "
            f"| The backend that answers this tier. |"
        )
    return lines


def section(name: str, node: JsonObject, defs: JsonObject) -> list[str]:
    if "patternProperties" in node:
        pattern, value = next(iter(node["patternProperties"].items()))
        lines = [f"## `[{name}.<name>]`", ""]
        if "description" in node:
            lines += [str(node["description"]), ""]
        lines += [f"`<name>` must match the pattern `{pattern}`. One table per backend:", ""]
        return [*lines, *field_table(resolve(value, defs), defs), ""]
    lines = [f"## `[{name}]`", ""]
    if "description" in node:
        lines += [str(node["description"]), ""]
    if "propertyNames" in node:
        lines += mapping_table(node)
    else:
        lines += field_table(resolve(node, defs), defs)
    return [*lines, ""]


def banner() -> str:
    return (
        f"<!-- GENERATED by `{COMMAND}` from {PACKAGE} {version(PACKAGE)}.\n"
        "     Do not edit. Change the field's description in the model and regenerate. -->"
    )


INTRODUCTION = """# Cognition configuration (`cognition.toml`)

This page describes the operator's cognition configuration as this build reads it. Every key, type, bound
and default comes from the Pydantic models in `cognition/lm-controller/src/mineworld_cognition/config.py`.
Rules a schema cannot express, such as which `kind` accepts which option, are written in the description of
the field they belong to, and the loader enforces them.

The file belongs to you, never to a world. The package README is
[`cognition/lm-controller/README.md`](../../cognition/lm-controller/README.md). The machine-readable schema is
[`schema/cognition-toml.schema.json`](schema/cognition-toml.schema.json).

A key marked *Required* must be present. A key marked *unset* may be left out and has no value. Keys that are
not listed here are refused by the loader."""


def render_page(schema: JsonObject) -> str:
    defs: JsonObject = schema.get("$defs", {})
    lines = [banner(), "", INTRODUCTION, ""]
    for name, node in schema["properties"].items():
        lines += section(name, node, defs)
    lines += [
        "## Examples",
        "",
        "Each example is a whole file. The test suite loads every block below with the loader.",
        "",
    ]
    for example in EXAMPLES:
        lines += [f"### {example.title}", "", "```toml", example.text.rstrip("\n"), "```", ""]
    return "\n".join(lines).rstrip("\n") + "\n"


def generate() -> dict[str, str]:
    """The files the generator owns, by path relative to `--out`. Pure: nothing is written."""
    check_examples()
    schema = schema_document()
    return {
        PAGE: render_page(schema),
        SCHEMA: json.dumps(schema, indent=2, sort_keys=True) + "\n",
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Generate the cognition configuration reference.")
    parser.add_argument("--out", required=True, type=Path, help="the directory to write into")
    arguments = parser.parse_args(argv)
    try:
        files = generate()
    except GenerationError as error:
        print(f"reference: {error}", file=sys.stderr)
        return 1
    for relative, text in files.items():
        target: Path = arguments.out / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {len(files)} files to {arguments.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
