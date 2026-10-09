"""Requests built only from what the newest observation offers.

The server's verdict travels in every observation: each `Affordance` says whether the action is
available to this observer now, and why not if it is not. These functions read that verdict and
compute nothing (step-17 §3.6, layer 2). A request the observation does not offer as available is
refused here, before any frame is sent (`NotOffered`); a request it does offer is still validated by the
server exactly as any other, because the world may have changed since the observation (D-P3-8).

Nothing here names a System Pack's vocabulary (D-P3-2): `talk`, `move` and their payloads belong to the
code that knows those packs.
"""

from __future__ import annotations

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.wire.contract import (
    ActionRecord,
    ActionRequest,
    Affordance,
    Location,
    Observation,
)
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, JsonValue

__all__ = ["NotOffered", "attempt", "request"]


class NotOffered(MineWorldError):
    """The observation does not offer this action, to this target, as available now."""


def attempt(observation: Observation, affordance: Affordance) -> ActionRequest:
    """The request a complete affordance names, unchanged (`ARC-34`, `PROTOCOL.md` §6).

    The affordance must be one of `observation`'s, available, and complete (it carries a `payload`).
    """
    if affordance not in observation.affordances:
        raise NotOffered(f"{affordance.action_type!r} is not among this observation's affordances")
    if not affordance.available:
        raise NotOffered(
            f"{affordance.action_type!r} is offered unavailable: {affordance.unavailable_reason!r}"
        )
    if affordance.payload is None:
        raise NotOffered(
            f"{affordance.action_type!r} is not a complete affordance; it carries no payload"
        )
    return ActionRequest(
        actor=observation.observer,
        action_type=affordance.action_type,
        target=affordance.target,
        payload=ActionRecord(action_type=affordance.action_type, payload=affordance.payload),
    )


def request(
    observation: Observation,
    action_type: ActionTypeId,
    *,
    target: EntityId | None,
    payload: JsonValue,
    actor_location: Location | None = None,
) -> ActionRequest:
    """A request for `action_type` on `target` with the caller's own `payload`, provided some
    affordance of `observation` offers that action type, on that target, as available now."""
    offered = [
        affordance
        for affordance in observation.affordances
        if affordance.action_type == action_type and affordance.target == target
    ]
    if not any(affordance.available for affordance in offered):
        verdicts = [repr(affordance.unavailable_reason) for affordance in offered]
        raise NotOffered(
            f"{action_type!r} on target {target!r} is not offered as available"
            + (f"; offered unavailable: {', '.join(verdicts)}" if verdicts else "")
        )
    return ActionRequest(
        actor=observation.observer,
        action_type=action_type,
        target=target,
        payload=ActionRecord(action_type=action_type, payload=payload),
        actor_location=actor_location,
    )
