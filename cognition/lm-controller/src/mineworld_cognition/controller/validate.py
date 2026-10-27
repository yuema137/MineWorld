"""`validate`: a parsed choice becomes a proposal, or a typed content failure (pr-s10-p6 §5.2.6, C6).

Every rule is one row of the table in §5.2.6, applied in order for the chosen `act`. Fields that do not belong to
the chosen act are ignored, not refused. A proposal names people and offers by id and position, never by label.
A recall is valid only if it is a citation token shown in the memory section, or an event id inside a range the
section cites (`forged_recall`).
"""

from __future__ import annotations

import re
from typing import Literal

from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EventId, JsonValue

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.controller.choice import WORDS_MAX_CHARS, ContentFailure, SocialChoice
from mineworld_cognition.controller.fill.base import Fillable
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
    amount: int | None = None
    """The amount chosen for an offer with an `up_to`; `None` takes the offer as it stands."""


class Go(CognitionModel):
    """Set out for a listed place (`pr-le-leave-early.md` §0.4 item 3): one `walk-to`, then the pacer."""

    kind: Literal["go"]
    place: EntityId
    recalls: tuple[str, ...]


class Sleep(CognitionModel):
    """Lie down to sleep here, for `minutes` (S35 N-e). The world decides whether that length is allowed here."""

    kind: Literal["sleep"]
    minutes: int
    recalls: tuple[str, ...]


class Nothing(CognitionModel):
    kind: Literal["nothing"]
    recalls: tuple[str, ...]


class Fill(CognitionModel):
    """An incomplete offer filled (S31-d §5.8; `ARC-183`): the payload a listed filler built from checked, typed
    fields. Never text the model wrote as a payload."""

    kind: Literal["fill"]
    action_type: ActionTypeId
    target: EntityId | None
    payload: JsonValue
    recalls: tuple[str, ...]


Proposal = Say | Attempt | Go | Sleep | Fill | Nothing
"""What the model proposed, validated. The only shapes a request may be built from (`ARC-72`, `INV-10`)."""


def validate(
    choice: SocialChoice,
    situation: Situation,
    memory: MemorySection,
    fillables: tuple[Fillable, ...] = (),
) -> Proposal | ContentFailure:
    """Applies §5.2.6's rules to one parsed answer, in table order for the chosen act. `fill` is checked by the
    listed filler it names (`fillables`); with none listed the parser has already refused it."""
    if choice.act == "fill":
        proposal = _fill(choice, fillables)
        if isinstance(proposal, ContentFailure):
            return proposal
    elif choice.act == "say":
        proposal = _say(choice, situation)
        if isinstance(proposal, ContentFailure):
            return proposal
    elif choice.act == "attempt":
        proposal = _attempt(choice, situation)
        if isinstance(proposal, ContentFailure):
            return proposal
    elif choice.act == "go":
        proposal = _go(choice, situation)
        if isinstance(proposal, ContentFailure):
            return proposal
    elif choice.act == "sleep":
        proposal = _sleep(choice, situation)
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
        if offer.number != choice.offer:
            continue
        if offer.up_to is None or choice.amount is None:
            return Attempt(kind="attempt", offer=offer, recalls=())
        if not 1 <= choice.amount <= offer.up_to:
            return ContentFailure(
                reason="bad_amount", detail=f"the amount is from 1 to {offer.up_to}"
            )
        return Attempt(kind="attempt", offer=offer, recalls=(), amount=choice.amount)
    return ContentFailure(reason="unknown_offer", detail="that offer is not listed")


def _go(choice: SocialChoice, situation: Situation) -> Go | ContentFailure:
    for place in situation.places:
        if place.label == choice.place:
            return Go(kind="go", place=place.entity, recalls=())
    return ContentFailure(reason="unknown_place", detail="that place is not listed")


def _sleep(choice: SocialChoice, situation: Situation) -> Sleep | ContentFailure:
    """Only a number of minutes is checked here; the bound and the place are the world's to decide (`ARC-72`)."""
    if not situation.sleep_offered:
        return ContentFailure(reason="unknown_offer", detail="sleeping is not offered now")
    if choice.minutes is None or choice.minutes < 1:
        return ContentFailure(
            reason="bad_amount", detail="sleep needs a number of minutes, 1 or more"
        )
    return Sleep(kind="sleep", minutes=choice.minutes, recalls=())


def _fill(choice: SocialChoice, fillables: tuple[Fillable, ...]) -> Fill | ContentFailure:
    """The listed filler the answer names checks its own fields; an F-number not listed is `unknown_fill`."""
    answer = choice.fill
    number = None if answer is None else answer.get("offer")
    listed = next((item for item in fillables if item.number == number), None)
    if answer is None or listed is None:
        return ContentFailure(reason="unknown_fill", detail="that F-number is not listed")
    checked = listed.filler.check(answer.get(listed.filler.key), listed.shown)
    if isinstance(checked, ContentFailure):
        return checked
    return Fill(
        kind="fill",
        action_type=checked.action_type,
        target=checked.target,
        payload=checked.payload,
        recalls=(),
    )


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


def _with_recalls(proposal: Proposal, recalls: tuple[str, ...]) -> Proposal:
    return proposal.model_copy(update={"recalls": recalls})
