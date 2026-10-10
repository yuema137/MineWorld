"""Lowering a request's JSON Schema to the strict subset some structured-output modes accept.

A strict mode refuses the numeric and length bounds and needs every object closed. `lower_schema`
returns a **copy** with those bounds removed and `additionalProperties: false` set on every object; the
request itself is never changed, so the cassette key, computed on the unlowered request, is unchanged
(ARC-58), and local validation (P6) still enforces every removed bound. Used by the Messages adapter
and the subscription bridge's presets (pr-s10-p5b-hosted-subscriptions.md §5.1, §5.4).

The walk is schema-aware: it lowers subschemas only where JSON Schema puts them, so a property *named*
`minimum`, or an `enum` value that happens to be an object, is data and is left alone.
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import JsonValue

REFUSED = frozenset(
    {
        "minLength",
        "maxLength",
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "multipleOf",
        "maxItems",
    }
)
"""Keywords a strict mode refuses; `minItems` is refused only above 1 (handled separately)."""

_NAMED_SUBSCHEMAS = frozenset(
    {"properties", "patternProperties", "$defs", "definitions", "dependentSchemas"}
)
_ONE_SUBSCHEMA = frozenset(
    {
        "items",
        "additionalItems",
        "contains",
        "not",
        "if",
        "then",
        "else",
        "propertyNames",
        "unevaluatedItems",
        "unevaluatedProperties",
    }
)
_SUBSCHEMA_LISTS = frozenset({"anyOf", "allOf", "oneOf", "prefixItems"})


def _is_object(schema: dict[str, JsonValue]) -> bool:
    kind = schema.get("type")
    return (
        kind == "object" or (isinstance(kind, list) and "object" in kind) or "properties" in schema
    )


def lower_schema(schema: JsonValue) -> JsonValue:
    """The strict copy of `schema` (a boolean schema, or anything not an object, is returned as is)."""
    if not isinstance(schema, dict):
        return schema
    lowered: dict[str, JsonValue] = {}
    for keyword, value in schema.items():
        if keyword in REFUSED:
            continue
        if keyword == "minItems" and isinstance(value, int) and value > 1:
            continue
        if keyword in _NAMED_SUBSCHEMAS and isinstance(value, dict):
            lowered[keyword] = {name: lower_schema(sub) for name, sub in value.items()}
        elif keyword in _ONE_SUBSCHEMA:
            lowered[keyword] = (
                [lower_schema(sub) for sub in value]
                if isinstance(value, list)
                else lower_schema(value)
            )
        elif keyword in _SUBSCHEMA_LISTS and isinstance(value, list):
            lowered[keyword] = [lower_schema(sub) for sub in value]
        else:
            lowered[keyword] = value
    if _is_object(schema):
        lowered["additionalProperties"] = False
    return lowered
