"""AC-d-6 (pr-s27d §8; step-27 AC-S27-3): on one world, the adapter answers what a plain client is answered.

The world is a scratch copy of `worlds/social-cafe` with IE-4's talk rule, built as `tools/cli/tests/social_interactions.rs`
builds it (IE-4): the copy's `world.yaml` gains a `configure:` list of `classes`, then `conversation`, and the two
sections are written as `configure/classes.yaml` and `configure/conversation.yaml`, with the same text. Residents may
not talk to commuters, and commuters may not talk to residents.

- The adapter on the commuter seat (`grace`) sees the resident Carol's `talk` offer unavailable, for `permission_denied`.
- A plain client on Carol's seat sees the same reason on its own affordance for `grace`, and a `talk` it sends anyway is
  `Rejected(permission_denied)`, the server's own answer.
- The adapter's `say` to Carol is `refused` with `not_offered` (F-S27d-5): a refusal is data in the act's outcome.

`TooFarAway` through the adapter (AC-c-3's case) is covered by `test_play_mcp.py`'s walk to the street, and the
`Unavailable` arm is unit-owned (F-S27d-4); neither is repeated here."""

from __future__ import annotations

import asyncio
import shutil
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest
from mcp_relay import Relay
from mineworld_sdk import SeatSession
from mineworld_sdk.wire.contract import ActionRecord, ActionRequest, Observation, Rejected
from mineworld_sdk.wire.ids import ActionTypeId, EntityId
from realserver import REPOSITORY, Server, seated
from support import run_default
from test_play_mcp import PATIENCE, Transcript, adapter, call, person_ref, walk

TALK = ActionTypeId("talk")
CLASSES = (
    "- { class: resident, of: person, tag: resident }\n"
    "- { class: commuter, of: person, tag: commuter }\n"
)
RULES = (
    "rules:\n"
    "  - { action: talk, actor: resident, target: commuter, effect: forbid }\n"
    "  - { action: talk, actor: commuter, target: resident, effect: forbid }\n"
)


def scratch_cafe(root: Path) -> Path:
    """A copy of social-cafe configured with IE-4's talk rule, as the Rust test writes it."""
    copy = root / "social-cafe"
    shutil.copytree(REPOSITORY / "worlds" / "social-cafe", copy)
    manifest = copy / "world.yaml"
    manifest.write_text(
        manifest.read_text(encoding="utf-8") + "\nconfigure:\n  - classes\n  - conversation\n",
        encoding="utf-8",
    )
    (copy / "configure").mkdir()
    (copy / "configure" / "classes.yaml").write_text(CLASSES, encoding="utf-8")
    (copy / "configure" / "conversation.yaml").write_text(RULES, encoding="utf-8")
    return copy


@pytest.fixture
def cafe(tmp_path: Path) -> Iterator[Server]:
    server = Server(str(scratch_cafe(tmp_path)), "--grant", "grace")
    try:
        yield server
    finally:
        server.stop()


def test_the_adapter_answers_what_a_plain_client_is_answered(cafe: Server, tmp_path: Path) -> None:
    run_default(_same_answers(cafe, tmp_path), timeout_s=120)


def _exit_to(view: dict[str, Any], label: str) -> str:
    """The reference of the place an exit of this view leads to, by one of its labels."""
    for exit_ in view["here"]["exits"]:
        if any(each["text"] == label for each in exit_["leads_to"]["labels"]):
            return str(exit_["leads_to"]["ref"])
    raise AssertionError(f"no exit leads to {label}")


async def _observation_showing(session: SeatSession, name: str) -> tuple[Observation, EntityId]:
    """The plain client's next observation that shows an entity named `name`, and that entity's id."""
    async with asyncio.timeout(PATIENCE):
        while True:
            seen = (await session.changed()).observation
            for entity in seen.entities:
                for component in entity.components:
                    if component.component_type == "display-name" and name in str(
                        component.payload
                    ):
                        return seen, entity.id


async def _same_answers(server: Server, tmp_path: Path) -> None:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    transcript = Transcript()
    carol: SeatSession = await seated(server, "carol")
    try:
        async with adapter(
            relay_url, tmp_path / "state", server.grants()["grace"], tmp_path / "same.log"
        ) as session:
            joined = await call(session, transcript, "join", {"seat": "grace", "nickname": "Grace"})
            handle = str(joined["handle"])
            # Grace starts at her office; Carol, a resident, is at the apartments. A talk needs both in one place,
            # so the commuter walks there: the office, the street, the apartments.
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            await walk(session, transcript, handle, {"place": _exit_to(view, "street")})
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            await walk(session, transcript, handle, {"place": _exit_to(view, "apartments")})
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            carol_ref = person_ref(view, "Carol")
            assert carol_ref is not None, "Carol is not at the apartments"

            # The adapter: the resident's talk, offered to the commuter, is unavailable for the rule.
            talk = next(
                offer
                for offer in view["offers"]
                if offer["action_type"] == "talk" and offer["target"] == carol_ref
            )
            assert talk["available"] is False
            assert talk["unavailable"]["reason"] == "permission_denied", talk

            # The plain client: its own affordance for grace carries the same reason. Its newest observation must
            # show the commuter, who has just arrived.
            seen, grace_id = await _observation_showing(carol, "Grace")
            plain = next(
                affordance
                for affordance in seen.affordances
                if affordance.action_type == TALK and affordance.target == grace_id
            )
            assert plain.available is False
            assert plain.unavailable_reason == talk["unavailable"]["reason"], (
                plain.unavailable_reason
            )

            # The plain client sends the talk anyway: the server's own answer, a rejection for the same reason.
            sent = await carol.submit(
                ActionRequest(
                    actor=seen.observer,
                    action_type=TALK,
                    target=grace_id,
                    payload=ActionRecord(action_type=TALK, payload={"utterance": "hello"}),
                )
            )
            result = getattr(sent, "result", None)
            assert isinstance(result, Rejected), sent
            assert result.rejected == "permission_denied", result

            # The adapter's say to the resident: a refusal in the outcome, `not_offered`.
            said = await call(
                session, transcript, "say", {"handle": handle, "to": carol_ref, "words": "hello"}
            )
            assert said["outcome"] == {"kind": "refused", "code": "not_offered"}, said["outcome"]
    finally:
        await carol.close()
        await relay.close()
