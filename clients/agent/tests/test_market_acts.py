"""The six market and notice kinds of `act` build the packs' own payloads and send them only as offered
(pr-s31g §5.6, C5; I-g-2).

A scripted seat holds one real post-S31-a observation (`planted.py`) with the incomplete offers economy and the
notices pack make (`list-for-sale`, `set-price`, `withdraw`, `publish-notice`, `subscribe`, `unsubscribe`: target
`None`, no payload). Each operation's request is compared with what `mineworld_sdk.offers.request` builds for that
offer and the payload written here as a literal, in the shape C0 pinned from the merged packs
(`systems/economy/src/action.rs`, `systems/notices/src/action.rs`, `body.rs`).
"""

from __future__ import annotations

import copy
from dataclasses import dataclass, field
from types import SimpleNamespace
from typing import Any, cast

import pytest
from mineworld_sdk import offers
from mineworld_sdk.reconnect import NotConnected
from mineworld_sdk.wire.contract import ActionRequest, Observation
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId, EntityId
from planted import Raw, decoded, me, payload, recorded
from pydantic import ValidationError
from support import run

from mineworld_agent.handle import Handle
from mineworld_agent.models import (
    ActResult,
    ListForSaleArgs,
    PriceArg,
    PricesBody,
    PublishNoticeArgs,
    SetPriceArgs,
    SubscribeArgs,
    TextBody,
    UnsubscribeArgs,
    WithdrawArgs,
)
from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Rendered, render

HANDLE = "0123456789abcdef0123456789abcdef"
ACTIONS = ("list-for-sale", "set-price", "withdraw", "publish-notice", "subscribe", "unsubscribe")


@dataclass(slots=True)
class ScriptedHandle:
    """A seat with one observation: enough of `Handle` for an act to be built and sent. A send records the
    request and answers nothing (the connection is between seats)."""

    rendered_: Rendered
    refs: RefTable
    sent: list[ActionRequest] = field(default_factory=list[ActionRequest])
    lost: None = None
    pacer: None = None
    last_at: int | None = None
    seat: SimpleNamespace = field(
        default_factory=lambda: SimpleNamespace(
            clock=None, welcome=SimpleNamespace(world=SimpleNamespace(time_scale=1, paused=False))
        )
    )

    def touch(self) -> None:
        return None

    def rendered(self) -> Rendered:
        return self.rendered_

    async def send(self, request: ActionRequest) -> None:
        self.sent.append(request)
        raise NotConnected()


def offering(actions: tuple[str, ...] = ACTIONS) -> Raw:
    """The recorded café, offering `actions` (and none of the other five) as the packs do: incomplete, no
    target. The recording itself offers `list-for-sale`: the visitor holds kinds it asks nothing for."""
    value = recorded()
    value["affordances"] = [a for a in value["affordances"] if a["action_type"] not in ACTIONS]
    template = next(a for a in value["affordances"] if a["action_type"] == "buy")
    for action in actions:
        offer = copy.deepcopy(template)
        offer.update(action_type=action, target=None, payload=None, available=True)
        offer["requirement"] = {
            "place": "any",
            "within_range": None,
            "requires_line_of_access": False,
            "requires_target_available": False,
        }
        value["affordances"].append(offer)
    return value


def scripted(value: Raw) -> tuple[Player, ScriptedHandle, Observation]:
    observation = decoded(value)
    refs = RefTable()
    rendered = render(observation, refs, 1, True)
    player = Player(PlayerConfig(server="ws://127.0.0.1:1/ws"), Invite("mwagent-scripted"))
    handle = ScriptedHandle(rendered_=rendered, refs=refs)
    player._handles[HANDLE] = cast(Handle, handle)  # pyright: ignore[reportPrivateUsage]
    return player, handle, observation


def held_items(value: Raw) -> list[str]:
    return [line["item"]["entity"] for line in payload(value, me(value), "holdings")["held"]]


def typed(item: str) -> dict[str, Any]:
    return {"entity": item, "entity_type": "item"}


def sent_as(observation: Observation, action: str, body: Any) -> ActionRequest:  # noqa: ANN401 (JSON)
    return offers.request(observation, ActionTypeId(action), target=None, payload=body)


def test_seller_kinds_send_economys_payloads() -> None:
    value = offering()
    kind = held_items(value)[0]
    player, handle, observation = scripted(value)
    ref = handle.refs.ref(EntityId(kind))
    run(player.list_for_sale(ListForSaleArgs(handle=HANDLE, item=ref, price=120)), timeout_s=30)
    run(
        player.set_price(SetPriceArgs(handle=HANDLE, item=ref, price=150, for_shop=True)),
        timeout_s=30,
    )
    run(player.withdraw(WithdrawArgs(handle=HANDLE, item=ref)), timeout_s=30)
    assert handle.sent == [
        sent_as(observation, "list-for-sale", {"item": typed(kind), "price": 120}),
        sent_as(observation, "set-price", {"item": typed(kind), "price": 150, "for": "shop"}),
        sent_as(observation, "withdraw", {"item": typed(kind)}),
    ]


def test_notice_kinds_send_the_notices_packs_payloads() -> None:
    player, handle, observation = scripted(offering())
    run(
        player.publish_notice(
            PublishNoticeArgs(
                handle=HANDLE,
                topic="news.daily",
                valid_for_s=7200,
                body=TextBody(kind="text", text="Market opens at 8"),
            )
        ),
        timeout_s=30,
    )
    run(player.subscribe(SubscribeArgs(handle=HANDLE, topic="prices.ingredients")), timeout_s=30)
    run(
        player.unsubscribe(UnsubscribeArgs(handle=HANDLE, topic="prices.ingredients")), timeout_s=30
    )
    assert handle.sent == [
        sent_as(
            observation,
            "publish-notice",
            {"topic": "news.daily", "body": {"text": "Market opens at 8"}, "valid_for": 7200},
        ),
        sent_as(observation, "subscribe", {"topic": "prices.ingredients"}),
        sent_as(observation, "unsubscribe", {"topic": "prices.ingredients"}),
    ]


def test_price_lines_are_sent_in_item_order() -> None:
    # The agent sees references, not ids, so it cannot sort; the pack takes lines strictly ascending by item.
    # Ids 9, 10 and 101 sort as integers (the kernel's order), not as text ("10" < "101" < "9").
    player, handle, observation = scripted(offering())
    nine, ten, hundred_one = (handle.refs.ref(EntityId(i)) for i in ("9", "10", "101"))
    lines = [
        PriceArg(item=hundred_one, price=3, basis=1),
        PriceArg(item=nine, price=1, basis=1),
        PriceArg(item=ten, price=2, basis=1),
        PriceArg(item=nine, price=4, basis=2),
    ]
    run(
        player.publish_notice(
            PublishNoticeArgs(
                handle=HANDLE,
                topic="prices.ingredients",
                valid_for_s=3600,
                body=PricesBody(kind="prices", lines=lines),
            )
        ),
        timeout_s=30,
    )
    body = {
        "prices": [
            {"item": "9", "price": 1, "basis": 1},
            {"item": "9", "price": 4, "basis": 2},
            {"item": "10", "price": 2, "basis": 1},
            {"item": "101", "price": 3, "basis": 1},
        ]
    }
    assert handle.sent == [
        sent_as(
            observation,
            "publish-notice",
            {"topic": "prices.ingredients", "body": body, "valid_for": 3600},
        )
    ]


def refused(answer: ActResult) -> str | None:
    return getattr(answer.outcome, "code", None)


def test_an_unknown_reference_is_refused_before_anything_is_sent() -> None:
    player, handle, _ = scripted(offering())
    unknown = "E999"
    answers = [
        run(
            player.list_for_sale(ListForSaleArgs(handle=HANDLE, item=unknown, price=1)),
            timeout_s=30,
        ),
        run(player.withdraw(WithdrawArgs(handle=HANDLE, item=unknown)), timeout_s=30),
        run(
            player.publish_notice(
                PublishNoticeArgs(
                    handle=HANDLE,
                    topic="prices.ingredients",
                    valid_for_s=3600,
                    body=PricesBody(
                        kind="prices", lines=[PriceArg(item=unknown, price=1, basis=1)]
                    ),
                )
            ),
            timeout_s=30,
        ),
    ]
    assert [refused(a) for a in answers] == ["unknown_ref"] * 3
    assert handle.sent == []


def test_an_action_the_world_does_not_offer_is_refused_not_offered() -> None:
    value = offering(actions=())
    kind = held_items(value)[0]
    player, handle, _ = scripted(value)
    ref = handle.refs.ref(EntityId(kind))
    answers = [
        run(
            player.list_for_sale(ListForSaleArgs(handle=HANDLE, item=ref, price=120)), timeout_s=30
        ),
        run(player.subscribe(SubscribeArgs(handle=HANDLE, topic="news.daily")), timeout_s=30),
    ]
    assert [refused(a) for a in answers] == ["not_offered", "not_offered"]
    assert handle.sent == []


@pytest.mark.parametrize("words", ["", "x" * 281, "é" * 141])
def test_a_notice_text_outside_the_packs_1_to_280_bytes_is_an_invalid_argument(words: str) -> None:
    # 141 two-byte characters are 282 bytes: the bound is UTF-8 bytes, as the pack counts them.
    with pytest.raises(ValidationError):
        TextBody(kind="text", text=words)
    assert TextBody(kind="text", text="é" * 140).text == "é" * 140
