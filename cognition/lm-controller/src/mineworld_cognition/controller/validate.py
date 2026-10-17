"""`validate`: a parsed choice becomes a proposal, or a typed content failure (pr-s10-p6 §5.2.6, C6).

Every rule is one row of the table in §5.2.6, applied in order for the chosen `act`. Fields that do not belong to
the chosen act are ignored, not refused. A proposal names people and offers by id and position, never by label.
A recall is valid only if it is a citation token shown in the memory section, or an event id inside a range the
section cites (`forged_recall`).
"""

from __future__ import annotations

import re
from typing import Literal

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.controller.choice import WORDS_MAX_CHARS, ContentFailure, SocialChoice
from mineworld_cognition.controller.situation import Offer, Situation
from mineworld_cognition.memory.records import EventRange, event_key
from mineworld_cognition.memory.retrieve import MemorySection

CITATION = re.compile(r"\[#[^\]]*\]")
DELIMITERS = ("<<<", ">>>")
WORDS_MAX_BYTES = 480


class Say(CognitionModel):
    kind: Literal["say"]
    to: EntityId
    words: str
    recalls: tuple[str, ...]


class Attempt(CognitionModel):
    kind: Literal["attempt"]
    offer: Offer
    recalls: tuple[str, ...]


class Nothing(CognitionModel):
    kind: Literal["nothing"]
    recalls: tuple[str, ...]


Proposal = Say | Attempt | Nothing
"""What the model proposed, validated. The only shapes a request may be built from (`ARC-72`, `INV-10`)."""


def validate(
    choice: SocialChoice, situation: Situation, memory: MemorySection
) -> Proposal | ContentFailure:
    """Applies §5.2.6's rules to one parsed answer, in table order for the chosen act."""
    if choice.act == "say":
        proposal = _say(choice, situation)
        if isinstance(proposal, ContentFailure):
            return proposal
    elif choice.act == "attempt":
        proposal = _attempt(choice, situation)
        if isinstance(proposal, ContentFailure):
            return proposal
    else:
        proposal = Nothing(kind="nothing", recalls=tuple(choice.recalls))
    forged = _forged_recall(choice.recalls, memory)
    if forged is not None:
        return forged
    return _with_recalls(proposal, tuple(choice.recalls))


def _say(choice: SocialChoice, situation: Situation) -> Say | ContentFailure:
    label = choice.to
    if label is None or label not in situation.talk_to:
        return ContentFailure(reason="unknown_person", detail="the person is not one I can talk to")
    entity = next((person.entity for person in situation.present if person.label == label), None)
    if entity is None:
        return ContentFailure(reason="unknown_person", detail="the person is not here")
    if choice.words == "":
        return ContentFailure(reason="empty_words", detail="there is nothing to say")
    if len(choice.words.encode("utf-8")) > WORDS_MAX_BYTES or len(choice.words) > WORDS_MAX_CHARS:
        return ContentFailure(reason="words_too_long", detail="the words exceed 480 bytes")
    if any(token in choice.words for token in DELIMITERS) or CITATION.search(choice.words):
        return ContentFailure(reason="leak", detail="the words contain a control token")
    return Say(kind="say", to=entity, words=choice.words, recalls=())


def _attempt(choice: SocialChoice, situation: Situation) -> Attempt | ContentFailure:
    for offer in situation.offers:
        if offer.number == choice.offer:
            return Attempt(kind="attempt", offer=offer, recalls=())
    return ContentFailure(reason="unknown_offer", detail="that offer is not listed")


def _forged_recall(recalls: list[str], memory: MemorySection) -> ContentFailure | None:
    shown = {line.citation.token() for line in memory.lines}
    ranges = memory.citations()
    for recall in recalls:
        if recall in shown:
            continue
        if _inside_a_cited_range(recall, ranges):
            continue
        return ContentFailure(
            reason="forged_recall", detail="a recall names nothing that was shown"
        )
    return None


def _inside_a_cited_range(recall: str, ranges: tuple[EventRange, ...]) -> bool:
    if not recall.isdigit():
        return False
    key = event_key(EventId(recall))
    return any(event_key(item.first) <= key <= event_key(item.last) for item in ranges)


def _with_recalls(proposal: Say | Attempt | Nothing, recalls: tuple[str, ...]) -> Proposal:
    if isinstance(proposal, Say):
        return Say(kind="say", to=proposal.to, words=proposal.words, recalls=recalls)
    if isinstance(proposal, Attempt):
        return Attempt(kind="attempt", offer=proposal.offer, recalls=recalls)
    return Nothing(kind="nothing", recalls=recalls)
