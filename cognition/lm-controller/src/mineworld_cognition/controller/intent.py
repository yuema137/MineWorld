"""`build`: a proposal becomes an `ActionRequest`, through `mineworld_sdk.offers` only (pr-s10-p6 §5.3, C6).

`Say` is a `talk` request with the words as its payload; `Attempt` is the complete affordance the observation
offered, unchanged except for a chosen `amount`; `Go` is one `walk-to` the place; `Nothing` builds no request.
This module is the only place in `controller/` that builds an `ActionRequest` (I-P6-2). AP6-11 as amended by
`ARC-282` (`pr-le-leave-early.md` §0.4 item 3): no `move`; `walk-to` only from a `go` choice; `walk-step` only from
the seat's pacer. A `NotOffered` from `offers` is a semantic failure (ruling 3f): the affordance is no longer
offered in the newest observation.
"""

from __future__ import annotations

from typing import Literal

from mineworld_sdk import offers
from mineworld_sdk.offers import NotOffered
from mineworld_sdk.wire.contract import ActionRecord, ActionRequest, Location, Observation, PlaceId
from mineworld_sdk.wire.ids import ActionTypeId

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.controller.validate import (
    Attempt,
    Fill,
    Go,
    Nothing,
    Proposal,
    Say,
    Sleep,
)

TALK = ActionTypeId("talk")
WALK_TO = ActionTypeId("walk-to")
SLEEP = ActionTypeId("sleep")


class SemanticFailure(CognitionModel):
    reason: Literal["not_offered", "rejected", "unavailable"]
    detail: str


def build(proposal: Proposal, observation: Observation) -> ActionRequest | SemanticFailure | None:
    """The request for `proposal` against the newest observation, or a typed semantic failure, or `None`
    for a valid decision to do nothing."""
    if isinstance(proposal, Nothing):
        return None
    if isinstance(proposal, Say):
        try:
            return offers.request(
                observation,
                TALK,
                target=proposal.to,
                payload={"utterance": proposal.words},
            )
        except NotOffered:
            return SemanticFailure(reason="not_offered", detail="no talk is offered to that person")
    if isinstance(proposal, Go):
        return _go(proposal, observation)
    if isinstance(proposal, Sleep):
        return _sleep(proposal, observation)
    if isinstance(proposal, Fill):
        return _fill(proposal, observation)
    return _attempt(proposal, observation)


def _fill(proposal: Fill, observation: Observation) -> ActionRequest | SemanticFailure:
    """An incomplete offer filled (S31-d §5.8.3): the filler's checked payload, on the action type and target the
    world offered as available, through `offers.request` like every request here (`ARC-72`, `ARC-183`)."""
    try:
        return offers.request(
            observation, proposal.action_type, target=proposal.target, payload=proposal.payload
        )
    except NotOffered:
        return SemanticFailure(reason="not_offered", detail="that offer is no longer available")


def _sleep(proposal: Sleep, observation: Observation) -> ActionRequest | SemanticFailure:
    """sleep's parametric `sleep { for: minutes }` (S35 N-e): the offer is matched by its type and no target."""
    try:
        return offers.request(observation, SLEEP, target=None, payload={"for": proposal.minutes})
    except NotOffered:
        return SemanticFailure(reason="not_offered", detail="sleeping is not offered now")


def _go(proposal: Go, observation: Observation) -> ActionRequest | SemanticFailure:
    """movement's `walk-to { to: { place: Location } }`: a place without a position means "enter it"."""
    place = Location(place=PlaceId(entity=proposal.place, entity_type="place"))
    try:
        return offers.request(
            observation,
            WALK_TO,
            target=None,
            payload={"to": {"place": place.model_dump(mode="json")}},
        )
    except NotOffered:
        return SemanticFailure(reason="not_offered", detail="walking is not offered now")


def _attempt(proposal: Attempt, observation: Observation) -> ActionRequest | SemanticFailure:
    position = proposal.offer.position
    if position >= len(observation.affordances):
        return SemanticFailure(reason="not_offered", detail="the offer moved")
    affordance = observation.affordances[position]
    if affordance.action_type != proposal.offer.action_type:
        return SemanticFailure(reason="not_offered", detail="the offer moved")
    try:
        request = offers.attempt(observation, affordance)
    except NotOffered:
        return SemanticFailure(reason="not_offered", detail="the offer is no longer available")
    payload = request.payload.payload
    if proposal.amount is None or not isinstance(payload, dict):
        return request
    chosen = ActionRecord(
        action_type=request.action_type, payload={**payload, "amount": proposal.amount}
    )
    return request.model_copy(update={"payload": chosen})
