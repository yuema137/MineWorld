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
from planted import Raw, decoded, item, me, payload, person, plant, recorded, shown
from pydantic import ValidationError
from support import run

from mineworld_agent.handle import Handle
from mineworld_agent.models import (
    AcceptProposalArgs,
    ActResult,
    DeclineProposalArgs,
    InspectArgs,
    ListForSaleArgs,
    PriceArg,
    PricesBody,
    ProposePriceArgs,
    PublishNoticeArgs,
    SetPriceArgs,
    SubscribeArgs,
    TextBody,
    UnsubscribeArgs,
    WithdrawArgs,
    WithdrawProposalArgs,
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


# ── Haggling (pr-s31h §5.14, C7; AC-h-9) ─────────────────────────────────────────────────────────

BOB, GRACE, CAROL = "10", "11", "20"
"""The recording's visitor (Bob, the buyer), and two people in the café: Grace (the seller) and Carol."""
HAGGLING = ("propose-price", "accept-proposal", "decline-proposal", "withdraw-proposal")


def proposal_line(kind: str, number: int = 7, buyer: str = BOB) -> dict[str, Any]:
    """One line of economy's `proposals` record, in `ProposalLine`'s shape (`systems/economy/src/proposal.rs`):
    Grace's ask of 4.50 for `kind`, Bob's proposal of 3.50."""
    return {
        "proposal": number,
        "seller": GRACE,
        "buyer": person(buyer),
        "item": item(kind),
        "price": 350,
        "ask": 450,
        "valid_until": 1000,
        "status": "open",
        "for_shop": False,
    }


def seen_by(value: Raw, observer: str) -> Raw:
    """The recording as `observer` would receive it: the same café, observed by another person in it, told
    the world's currency about itself (economy discloses `currency` to every living person)."""
    copied = copy.deepcopy(value)
    copied["observer"] = observer
    plant(copied, observer, "currency", payload(value, me(value), "currency"))
    return copied


def haggling_offered(value: Raw, targets: dict[str, tuple[str | None, ...]]) -> Raw:
    """`value` offering each haggling action on each of its targets, incomplete, as economy offers them."""
    template = next(a for a in value["affordances"] if a["action_type"] == "buy")
    for action, on in targets.items():
        for target in on:
            offer = copy.deepcopy(template)
            offer.update(action_type=action, target=target, payload=None, available=True)
            value["affordances"].append(offer)
    return value


def test_a_proposal_reaches_its_buyer_and_its_seller_and_nobody_else() -> None:
    # AC-h-9: Bob's outgoing and Grace's incoming show the proposal with `display`; Carol's view and her
    # inspect of Grace show neither. Each observation is as the server discloses it: the record is about
    # the observer itself (§5.9).
    base = recorded()
    kind = held_items(base)[0]
    line = proposal_line(kind)

    bobs = seen_by(base, BOB)
    plant(bobs, BOB, "proposals", {"incoming": [], "outgoing": [line]})
    view, refs = shown(bobs)
    assert view.me.proposals is not None and view.me.proposals.incoming == []
    (outgoing,) = view.me.proposals.outgoing
    assert (outgoing.proposal, outgoing.seller, outgoing.buyer) == (
        7,
        refs.ref(EntityId(GRACE)),
        "E1",
    )
    assert (outgoing.price.minor, outgoing.price.display) == (350, "3.50")
    assert outgoing.ask is not None and outgoing.ask.display == "4.50"
    assert (outgoing.status, outgoing.valid_until, outgoing.for_shop) == ("open", 1000, False)

    graces = seen_by(base, GRACE)
    plant(graces, GRACE, "proposals", {"incoming": [line], "outgoing": []})
    view, refs = shown(graces)
    assert view.me.proposals is not None and view.me.proposals.outgoing == []
    (incoming,) = view.me.proposals.incoming
    assert (incoming.proposal, incoming.buyer) == (7, refs.ref(EntityId(BOB)))
    assert incoming.price.display == "3.50"

    carols = seen_by(base, CAROL)
    view, refs = shown(carols)
    assert view.me.proposals is None
    assert "proposals" not in view.model_dump_json()
    player, _, _ = scripted(carols)
    seen = run(
        player.inspect(InspectArgs(handle=HANDLE, ref=refs.ref(EntityId(GRACE)))), timeout_s=30
    )
    assert "proposals" not in [c.component for c in seen.components]
    assert "proposals" not in seen.model_dump_json()


def test_a_proposals_record_on_someone_else_never_reaches_my_view() -> None:
    # I-h-5 on the adapter's side: the record is read from my own entity only, so even a record on another
    # person (which the server never sends) shows nowhere in the view.
    carols = seen_by(recorded(), CAROL)
    kind = held_items(recorded())[0]
    plant(carols, GRACE, "proposals", {"incoming": [proposal_line(kind)], "outgoing": []})
    view, _ = shown(carols)
    assert view.me.proposals is None
    assert view.here.people, "Grace is in Carol's view"
    assert "proposals" not in view.model_dump_json()


def test_the_four_haggling_kinds_send_economys_payloads() -> None:
    base = recorded()
    kind = held_items(base)[0]
    value = haggling_offered(
        base,
        {
            "propose-price": (None, GRACE),
            "accept-proposal": (CAROL,),
            "decline-proposal": (None,),
            "withdraw-proposal": (None,),
        },
    )
    plant(
        value,
        BOB,
        "proposals",
        {"incoming": [proposal_line(kind, number=9, buyer=CAROL)], "outgoing": []},
    )
    player, handle, observation = scripted(value)
    ref, grace = handle.refs.ref(EntityId(kind)), handle.refs.ref(EntityId(GRACE))
    run(
        player.propose_price(ProposePriceArgs(handle=HANDLE, to=grace, item=ref, price=350)),
        timeout_s=30,
    )
    run(player.propose_price(ProposePriceArgs(handle=HANDLE, item=ref, price=90)), timeout_s=30)
    run(player.accept_proposal(AcceptProposalArgs(handle=HANDLE, proposal=9)), timeout_s=30)
    run(player.decline_proposal(DeclineProposalArgs(handle=HANDLE, proposal=9)), timeout_s=30)
    run(player.withdraw_proposal(WithdrawProposalArgs(handle=HANDLE, proposal=4)), timeout_s=30)

    def sent(action: str, target: str | None, body: Any) -> ActionRequest:  # noqa: ANN401 (JSON)
        return offers.request(
            observation,
            ActionTypeId(action),
            target=None if target is None else EntityId(target),
            payload=body,
        )

    assert handle.sent == [
        sent("propose-price", GRACE, {"item": typed(kind), "price": 350}),
        sent("propose-price", None, {"item": typed(kind), "price": 90}),
        sent("accept-proposal", CAROL, {"proposal": 9}),
        sent("decline-proposal", None, {"proposal": 9}),
        sent("withdraw-proposal", None, {"proposal": 4}),
    ]


def test_accepting_a_proposal_not_mine_to_answer_sends_nothing() -> None:
    # The target of `accept-proposal` is the proposal's buyer, read from my newest record; a number my record
    # does not list, or an outgoing one, is not offered to me.
    base = recorded()
    kind = held_items(base)[0]
    value = haggling_offered(base, {action: (None, CAROL) for action in HAGGLING})
    plant(value, BOB, "proposals", {"incoming": [], "outgoing": [proposal_line(kind, number=9)]})
    player, handle, _ = scripted(value)
    answers = [
        run(player.accept_proposal(AcceptProposalArgs(handle=HANDLE, proposal=9)), timeout_s=30),
        run(player.accept_proposal(AcceptProposalArgs(handle=HANDLE, proposal=3)), timeout_s=30),
        run(
            player.propose_price(ProposePriceArgs(handle=HANDLE, to="E999", item="E998", price=1)),
            timeout_s=30,
        ),
    ]
    assert [refused(a) for a in answers] == ["not_offered", "not_offered", "unknown_ref"]
    assert handle.sent == []


@pytest.mark.parametrize("price", [0, 2**64])
def test_a_proposed_price_is_one_minor_unit_or_more_within_the_wire_bound(price: int) -> None:
    with pytest.raises(ValidationError):
        ProposePriceArgs(handle=HANDLE, item="E1", price=price)
    with pytest.raises(ValidationError):
        AcceptProposalArgs(handle=HANDLE, proposal=0)
