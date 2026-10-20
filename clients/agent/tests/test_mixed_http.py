"""Two agents and a person through one HTTP adapter, all in the café (pr-s27e §8 AC-e-2, AC-e-3, AC-e-7).

Agent A (bearer: the visitor grant) plays `visitor` and agent B (bearer: the wanderer grant) plays `wanderer`, both
through one adapter process; a plain SDK client with the invite plays `bob`. Each sees the other two in the café;
Bob says a line to A. For 20 wall seconds every fact A is shown is a fact of `mineworld perceived --person visitor`,
and every fact B is shown one of `--person wanderer` (the server's audience rule is the oracle, never the adapter's
report of itself). A's handle used with B's bearer is `unknown_handle`, and works again with A's. A third agent
whose bearer is the `visitor:take-over` grant cannot resume A's seat: its state partition holds no resume file.
"""

from __future__ import annotations

import asyncio
import time
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest
from http_adapter import http_adapter
from mineworld_sdk import offers
from mineworld_sdk.wire.contract import Accepted as WireAccepted
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId
from realserver import Server, market_town, perceived_export, seated
from support import run_default
from test_play_mcp import (
    PATIENCE,
    WALK_PATIENCE,
    Transcript,
    all_events,
    call,
    person_ref,
    refused,
    until,
    walk,
)

from mineworld_agent.pool import partition

pytestmark = pytest.mark.real_server

WINDOW_S = 20.0
"""The oracle's window, in wall seconds (AC-e-2)."""
LINE = "Good morning, Vera."
TALK = ActionTypeId("talk")


@pytest.fixture
def café(tmp_path: Path) -> Iterator[Server]:
    server = Server(
        "market-town",
        *market_town(
            "--save",
            str(tmp_path / "save"),
            "--grant",
            "visitor",
            "--grant",
            "visitor:take-over",
            "--grant",
            "wanderer",
            "--grant",
            "alice",
            "--hold",
            "10",
            "--time-scale",
            "1",
        ),
    )
    try:
        yield server
    finally:
        server.stop()


def people(view: dict[str, Any]) -> set[str]:
    return {str(person["name"]["text"]) for person in view["here"]["people"]}


async def _bob_says(server: Server, words: str) -> None:
    """Bob, a plain client, says `words` to Vera once his observation offers it (once she is close enough)."""
    bob = await seated(server, "bob")
    try:
        since: int | None = None
        names: list[str] = []
        try:
            async with asyncio.timeout(WALK_PATIENCE + PATIENCE):
                while True:
                    frame = await bob.changed(since)  # strictly newer, so the loop waits
                    since, seen = frame.seq, frame.observation
                    names = [
                        str(component.payload)
                        for entity in seen.entities
                        for component in entity.components
                        if component.component_type == "display-name"
                    ] + [
                        f"{offer.target}:{offer.available}:{offer.unavailable_reason}"
                        for offer in seen.affordances
                        if offer.action_type == TALK
                    ]
                    vera = next(
                        (
                            entity.id
                            for entity in seen.entities
                            for component in entity.components
                            if component.component_type == "display-name"
                            and "Vera" in str(component.payload)
                        ),
                        None,
                    )
                    if vera is not None and any(
                        offer.action_type == TALK and offer.target == vera and offer.available
                        for offer in seen.affordances
                    ):
                        break
        except TimeoutError:
            raise AssertionError(f"Bob was never offered talk to Vera; he saw {names}") from None
        outcome = await bob.submit(
            offers.request(seen, TALK, target=vera, payload={"utterance": words})
        )
        assert isinstance(getattr(outcome, "result", None), WireAccepted), outcome
        await bob.leave()
    finally:
        await bob.close()


def test_two_agents_and_a_person_each_see_only_their_own(café: Server, tmp_path: Path) -> None:
    run_default(_mixed(café, tmp_path), timeout_s=300)


async def _mixed(town: Server, tmp_path: Path) -> None:
    grants = town.grants()
    state = tmp_path / "state"
    transcript = Transcript()
    async with http_adapter(town.url, state, tmp_path / "logs") as adapter:
        async with (
            adapter.session(grants["visitor"]) as a,
            adapter.session(grants["wanderer"]) as b,
        ):
            started = time.monotonic()
            joined_a = await call(a, transcript, "join", {"seat": "visitor", "nickname": "A"})
            joined_b = await call(b, transcript, "join", {"seat": "wanderer", "nickname": "B"})
            handle_a, handle_b = str(joined_a["handle"]), str(joined_b["handle"])
            bob = asyncio.create_task(_bob_says(town, LINE))

            async def everyone_here() -> bool:
                seen_a = (await call(a, transcript, "observe", {"handle": handle_a}))["view"]
                seen_b = (await call(b, transcript, "observe", {"handle": handle_b}))["view"]
                names_a, names_b = people(seen_a), people(seen_b)
                return (
                    any(n.startswith("Wes") for n in names_a)
                    and any(n.startswith("Bob") for n in names_a)
                    and any(n.startswith("Vera") for n in names_b)
                    and any(n.startswith("Bob") for n in names_b)
                )

            await until(everyone_here, patience=PATIENCE, what="the three saw each other")
            # Bob may stand beyond talking distance in the café: A walks up to him, then he speaks.
            seen_a = (await call(a, transcript, "observe", {"handle": handle_a}))["view"]
            bob_ref = person_ref(seen_a, "Bob")
            assert bob_ref is not None
            await walk(a, transcript, handle_a, {"person": bob_ref})
            await bob

            heard: list[dict[str, Any]] = []

            async def a_heard() -> bool:
                # The inbox, not `wait`: Bob may have spoken before a `wait` would have started.
                heard[:] = (
                    event
                    for event in await all_events(a, transcript, handle_a)
                    if isinstance(event.get("words"), dict) and event["words"]["text"] == LINE
                )
                return bool(heard)

            await until(a_heard, patience=PATIENCE, what="A heard Bob")

            # AC-e-3: A's handle with B's bearer is unknown there; B's call changed nothing for A.
            crossed = await refused(b, transcript, "observe", {"handle": handle_a})
            assert crossed.code == "unknown_handle", crossed
            crossed = await refused(
                b,
                transcript,
                "act",
                {"handle": handle_a, "act": {"kind": "say", "to": "E1", "words": "x"}},
            )
            assert crossed.code == "unknown_handle", crossed
            await call(a, transcript, "observe", {"handle": handle_a})

            # AC-e-7: C's bearer also admits the visitor seat, but C's partition has no resume file, so its join
            # is a plain join onto a held seat.
            async with adapter.session(grants["visitor take-over"]) as c:
                held = await refused(c, transcript, "join", {"seat": "visitor", "nickname": "C"})
                assert held.join == "seat_occupied", held
            partitions = {path.name for path in state.iterdir() if path.is_dir()}
            mine = partition(Invite(grants["visitor"]))
            assert {mine, partition(Invite(grants["wanderer"]))} <= partitions, partitions
            assert (state / mine / "visitor.json").is_file()
            assert not (
                state / partition(Invite(grants["visitor take-over"])) / "visitor.json"
            ).exists()

            left = WINDOW_S - (time.monotonic() - started)
            if left > 0:
                await asyncio.sleep(left)  # the oracle's window: the town keeps living meanwhile
            facts_a = await all_events(a, transcript, handle_a)
            facts_b = await all_events(b, transcript, handle_b)
            facts_a += heard
            await call(a, transcript, "leave", {"handle": handle_a})
            await call(b, transcript, "leave", {"handle": handle_b})

    save = tmp_path / "save"
    for person, facts in (("visitor", facts_a), ("wanderer", facts_b)):
        exported = {
            (event.at, str(event.event_type))
            for event in perceived_export("market-town", save, person)
        }
        shown = {(fact["at"], fact["event_type"]) for fact in facts}
        assert shown, f"{person} was shown no fact"
        assert shown <= exported, (person, sorted(shown - exported))
