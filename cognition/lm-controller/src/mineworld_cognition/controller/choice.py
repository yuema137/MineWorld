"""`SocialChoice`, its JSON Schema, and the parser (pr-s10-p6 §§5.2.5, 5.2.6, C6).

A model's answer is parsed here and nowhere else. The schema is written out by hand as a JSON value and kept
as a golden file, so a change is reviewed. `parse_choice` reads only the completion's text and its finish; it
never looks at a world. Its failures are typed (`ContentFailure`), and the reasons are the table's ten rows
(D-P6b-4: the design says "nine" in C6 and lists ten in §5.2.6; the table is the authority).
"""

from __future__ import annotations

import json
from typing import Annotated, Literal

from mineworld_sdk.wire.ids import JsonValue
from pydantic import Field, ValidationError

from mineworld_cognition.backend.model import CognitionModel, Completion

WORDS_MAX_CHARS = 480
RECALLS_MAX = 8
FENCE = "```"

ContentReason = Literal[
    "not_json",
    "schema",
    "truncated",
    "refused",
    "unknown_person",
    "empty_words",
    "words_too_long",
    "unknown_offer",
    "unknown_place",
    "bad_amount",
    "forged_recall",
    "leak",
]
"""The content failure reasons of §5.2.6, one per row of its table."""


class SocialChoice(CognitionModel):
    """What the model answers. Strict and closed; `words` and `recalls` are bounded for the model too."""

    act: Literal["say", "attempt", "go", "nothing"]
    to: str | None
    words: Annotated[str, Field(max_length=WORDS_MAX_CHARS)]
    offer: int | None
    recalls: Annotated[list[str], Field(max_length=RECALLS_MAX)]
    place: str | None = None
    """`go`: a place listed under "Places I can walk to" (LE-c). Absent from an answer to the base schema."""
    amount: int | None = None
    """`attempt` of an offer listed "(up to N)": how much, 1 … N (LE-c). Absent from the base schema."""


class ContentFailure(CognitionModel):
    reason: ContentReason
    detail: str


SOCIAL_CHOICE_SCHEMA: dict[str, JsonValue] = {
    "type": "object",
    "additionalProperties": False,
    "required": ["act", "to", "words", "offer", "recalls"],
    "properties": {
        "act": {"type": "string", "enum": ["say", "attempt", "nothing"]},
        "to": {"type": ["string", "null"]},
        "words": {"type": "string", "maxLength": WORDS_MAX_CHARS},
        "offer": {"type": ["integer", "null"]},
        "recalls": {"type": "array", "items": {"type": "string"}, "maxItems": RECALLS_MAX},
    },
}
"""The schema every decision request carries (§5.2.5). Generated once, committed as a golden (C6)."""


def choice_schema(*, go: bool, amount: bool) -> dict[str, JsonValue]:
    """The schema of one request: the base schema when the seat can neither walk to a listed place nor
    choose an amount, so such a request is the same as before LE-c; otherwise the base schema with the
    `go` act and the `place` and `amount` keys (`pr-le-leave-early.md` §0.4 and §0.5)."""
    if not go and not amount:
        return SOCIAL_CHOICE_SCHEMA
    acts: list[JsonValue] = ["say", "attempt", *(["go"] if go else []), "nothing"]
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["act", "to", "words", "offer", "recalls", "place", "amount"],
        "properties": {
            "act": {"type": "string", "enum": acts},
            "to": {"type": ["string", "null"]},
            "words": {"type": "string", "maxLength": WORDS_MAX_CHARS},
            "offer": {"type": ["integer", "null"]},
            "recalls": {"type": "array", "items": {"type": "string"}, "maxItems": RECALLS_MAX},
            "place": {"type": ["string", "null"]},
            "amount": {"type": ["integer", "null"]},
        },
    }


def parse_choice(completion: Completion) -> SocialChoice | ContentFailure:
    """Reads the answer. Truncation and refusal come from the finish; otherwise the text must be one JSON
    object (after one optional enclosing code fence), and it must match the choice model strictly."""
    if completion.finish == "length":
        return ContentFailure(reason="truncated", detail="the answer hit the output limit")
    if completion.finish == "refused":
        return ContentFailure(reason="refused", detail="the model declined to answer")
    text = _unfenced(completion.text.strip())
    try:
        loaded: JsonValue = json.loads(text)
    except json.JSONDecodeError:
        return ContentFailure(reason="not_json", detail="the answer is not one JSON object")
    if not isinstance(loaded, dict):
        return ContentFailure(reason="not_json", detail="the answer is not one JSON object")
    try:
        return SocialChoice.model_validate(loaded)
    except ValidationError as error:
        first = error.errors()[0]
        where = ".".join(str(part) for part in first["loc"])
        return ContentFailure(reason="schema", detail=f"{where or 'answer'}: {first['type']}")


def _unfenced(text: str) -> str:
    """Removes at most one enclosing Markdown code fence, with an optional `json` tag (§5.2.6)."""
    if not (text.startswith(FENCE) and text.endswith(FENCE) and len(text) >= 2 * len(FENCE)):
        return text
    inner = text[len(FENCE) : -len(FENCE)]
    if inner.startswith("json"):
        inner = inner[len("json") :]
    return inner.strip()
