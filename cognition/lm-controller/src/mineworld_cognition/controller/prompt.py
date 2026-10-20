"""`build_request`: the decision request, assembled from typed parts (pr-s10-p6 §5.2.4, C6).

A pure function. It reads no clock, no randomness and no environment, and it never writes an entity id, a
provider name or a key into the request (I-10). Heard words are data: the delimiter tokens `<<<` and `>>>` are
replaced before quoting, so a heard line cannot close the quote. The sampling seed is derived from the seat and
the trigger's event id, so the same trigger always yields the same request (§5.2.4).

A seat with a next part of its day, or places it may walk to, also gets a `MY DAY` section and the `go` act
(`pr-le-leave-early.md` §0.4, LE-c); a situation without them renders exactly as before, so recorded
requests keep their keys.
"""

from __future__ import annotations

import hashlib
from typing import Literal

from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, JsonValue

from mineworld_cognition.backend.model import CognitionModel, CompletionRequest, Message, Sampling
from mineworld_cognition.controller.choice import choice_schema
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.situation import Situation, clock
from mineworld_cognition.memory.retrieve import MemorySection

REASK_PREVIOUS_BYTES = 2048
TEMPERATURE_MILLI = 700
MAX_OUTPUT_TOKENS = 256
HEARD_OPEN = "<<<heard>>>"
HEARD_CLOSE = "<<<end>>>"
DELIMITER_SAFE = {"<<<": chr(0x2039) * 3, ">>>": chr(0x203A) * 3}


class Trigger(CognitionModel):
    event: EventId
    kind: Literal["spoken_to", "arrived", "upcoming", "complaint"]
    speaker: EntityId | None
    words: str


class Reask(CognitionModel):
    """A content failure: the previous raw answer, and the reason it was not usable (§5.2.4)."""

    kind: Literal["reask"]
    previous: str
    reason: str
    detail: str


class Replan(CognitionModel):
    """A semantic refusal: the world refused the choice, with its typed code (§5.2.4)."""

    kind: Literal["replan"]
    rejection: str


Feedback = tuple[Reask | Replan, ...]


def sampling_seed(seat: EntityKey, event: EventId) -> int:
    """`sha256("<seat>:<event>")`, the first four bytes as an unsigned integer (§5.2.4)."""
    return int.from_bytes(hashlib.sha256(f"{seat}:{event}".encode()).digest()[:4], "big")


def build_request(
    seat: EntityKey,
    persona: Persona,
    situation: Situation,
    memory: MemorySection,
    trigger: Trigger,
    feedback: Feedback,
    output_schema: dict[str, JsonValue] | None = None,
) -> CompletionRequest:
    """The request for one attempt. `feedback` is empty for the first attempt; a re-ask adds the previous
    raw answer and the reason; a re-plan adds the refusal. The schema defaults to `choice_schema` of the
    situation."""
    messages = [
        Message(role="system", text=_system(persona, situation)),
        Message(role="user", text=_user(situation, memory, trigger)),
    ]
    for note in feedback:
        if note.kind == "reask":
            messages.append(
                Message(role="assistant", text=_cut(note.previous, REASK_PREVIOUS_BYTES))
            )
            messages.append(
                Message(
                    role="user",
                    text=(
                        f"That answer was not usable: {note.reason}: {note.detail}. "
                        "Answer again with one JSON object."
                    ),
                )
            )
        else:
            messages.append(
                Message(
                    role="user",
                    text=(
                        f"The world refused your choice: {note.rejection}. "
                        "Choose again from what is listed."
                    ),
                )
            )
    return CompletionRequest(
        purpose="decide",
        messages=tuple(messages),
        output_schema=(
            choice_schema(go=bool(situation.places), amount=any(o.up_to for o in situation.offers))
            if output_schema is None
            else output_schema
        ),
        sampling=Sampling(
            temperature_milli=TEMPERATURE_MILLI,
            max_output_tokens=MAX_OUTPUT_TOKENS,
            seed=sampling_seed(seat, trigger.event),
        ),
    )


def _system(persona: Persona, situation: Situation) -> str:
    lines = [f"You are {situation.me}."]
    if persona.summary:
        lines.append(f"About you: {persona.summary}")
    if persona.traits:
        lines.append("Traits: " + ", ".join(persona.traits) + ".")
    if persona.speech:
        lines.append(f"How you speak: {persona.speech}")
    if situation.places:
        lines.extend(
            [
                'You may also use "go" with "place" set to a place listed under Places I can walk'
                " to; you then walk there. Nobody walks you anywhere unless you choose to go.",
            ]
        )
    if any(offer.up_to is not None for offer in situation.offers):
        lines.append(
            'For an offer listed "(up to N)", you may set "amount" to how much, from 1 to N; with'
            ' "amount" null it is the whole N.'
        )
    lines.extend(
        [
            "Rules: answer with one JSON object and nothing else.",
            'The object has the keys "act" (say, attempt or nothing), "to" (a person from the list,'
            ' or null), "words" (what you say, or ""), "offer" (a number from the list, or null),'
            ' and "recalls" (a list of citation tokens you rely on, or []).',
            'Use "say" to speak to someone you can talk to; use "attempt" only with a number listed'
            ' under Offers. Each recall is a JSON string, a token exactly as shown, such as "[#12]".',
            'For example: {"act": "say", "to": "<someone you can talk to>", "words": "<what you say>",'
            ' "offer": null, "recalls": ["[#12]"]}',
            "Choose only among the people and offers listed. Never invent people, events or numbers.",
            "Cite memory only by the tokens shown.",
            "When someone speaks to you, answer them as yourself, using what you remember of them, and"
            " cite the memories you used.",
            "If WHAT I REMEMBER begins with what you and that person said to each other before, your"
            " answer must take it up: in your words, mention what they told you there, and cite those"
            " tokens in recalls.",
        ]
    )
    return "\n".join(lines)


def _user(situation: Situation, memory: MemorySection, trigger: Trigger) -> str:
    people = ", ".join(person.label for person in situation.present) or "nobody"
    talk = ", ".join(situation.talk_to) or "nobody"
    offers = (
        "\n".join(
            f"{offer.number}. {offer.action_type}"
            + ("" if offer.target_label is None else f" with {offer.target_label}")
            + ("" if offer.up_to is None else f" (up to {offer.up_to})")
            for offer in situation.offers
        )
        or "none"
    )
    heard = _heard(trigger, situation)
    return "\n".join(
        [
            "WHERE I AM",
            f"Place: {situation.place}",
            f"People here: {people}",
            f"I can talk to: {talk}",
            "Offers:",
            offers,
            "",
            *_my_day(situation),
            "WHAT I REMEMBER",
            memory.render() or "nothing",
            "",
            "WHAT JUST HAPPENED",
            heard,
        ]
    )


def _my_day(situation: Situation) -> list[str]:
    """The `MY DAY` section: the time, the plan lines and the places a `go` may name. Nothing when the
    situation has none of them, so an older situation renders as it always did."""
    if not situation.plans and not situation.places:
        return []
    lines = ["MY DAY", f"Now: {clock(situation.at)}", *situation.plans]
    if situation.walking:
        lines.append("I am walking now.")
    if situation.places:
        lines.append("Places I can walk to: " + ", ".join(p.label for p in situation.places))
    return [*lines, ""]


def _heard(trigger: Trigger, situation: Situation) -> str:
    """Names the speaker by the label the situation lists them under, so a model with several people in reach knows
    whom it is answering (S10 P7 D-P7-6); a speaker not present now is "Someone". Never an entity id (I-10)."""
    if trigger.kind == "arrived":
        return "Someone I know came into this place."
    if trigger.kind == "upcoming":
        return "I think about what comes next in my day."
    if trigger.kind == "complaint":
        return trigger.words
    labels = {person.entity: person.label for person in situation.present}
    speaker = labels.get(trigger.speaker, "Someone") if trigger.speaker is not None else "Someone"
    return f"{speaker} said to me: {HEARD_OPEN}{_quoted(trigger.words)}{HEARD_CLOSE}"


def _quoted(words: str) -> str:
    for token, safe in DELIMITER_SAFE.items():
        words = words.replace(token, safe)
    return words


def _cut(text: str, limit: int) -> str:
    """At most `limit` UTF-8 bytes, cut at a character boundary."""
    encoded = text.encode("utf-8")
    if len(encoded) <= limit:
        return text
    return encoded[:limit].decode("utf-8", errors="ignore")
