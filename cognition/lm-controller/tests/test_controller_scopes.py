"""AC-c-9: a seat recalls through its memory stack, and a defective scope never reaches a model (pr-s28c §5.8).

The real `SeatRunner` over the scripted connector and a scripted backend, as `test_controller_seat.py` runs it.
The seat's stack holds the hostile fixture scope `cites` after `person`; its line cites an event no one
perceived. The oracle is the request the same seat sends with `person` alone.
"""

from __future__ import annotations

import asyncio
import json
from pathlib import Path

import pytest
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EventId
from scope_fixtures import Cites
from scripted_seat import (
    INVITE,
    FakeClock,
    Link,
    ScriptedServer,
    observation,
    perceived,
    spoke,
    until,
    welcome,
)
from support import run
from test_controller_seat import ALICE, ScriptedAnswers
from test_controller_seat import _cancel as cancel  # pyright: ignore[reportPrivateUsage]
from test_controller_seat import _config as config  # pyright: ignore[reportPrivateUsage]
from test_controller_seat import _decisions as decisions  # pyright: ignore[reportPrivateUsage]
from test_controller_seat import _logged as logged  # pyright: ignore[reportPrivateUsage]
from test_controller_seat import _nothing as nothing  # pyright: ignore[reportPrivateUsage]
from test_controller_seat import _WallClock as WallClock  # pyright: ignore[reportPrivateUsage]

from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.homes import SeatHomes
from mineworld_cognition.controller.seat import SeatRunner
from mineworld_cognition.gateway import Budget, ModelGateway, Router
from mineworld_cognition.memory.config import MemoryConfig
from mineworld_cognition.memory.records import Citation
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.retrieve import SectionLine
from mineworld_cognition.memory.scopes.errors import ScopeNotSelectable
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice
from mineworld_cognition.memory.scopes.world import WorldScope

PERSON = MemoryScopeChoice(
    id="person", parameters={"sections": {"L0": {"lines": 48, "bytes": 600}}}
)


def _registry() -> ScopeRegistry:
    return ScopeRegistry(
        {"person": PersonScope, "world": WorldScope, "cites": Cites}, base=("person", "world")
    )


def _runner(tmp_path: Path, link: Link, backend: ScriptedAnswers, seat: LmSeat) -> SeatRunner:
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), WallClock())
    clock = FakeClock()
    return SeatRunner(
        config(),
        seat,
        router=Router({"social": ModelGateway(backend, budget)}, budget),
        connector=ScriptedServer(link),
        invite=Invite(INVITE),
        nickname="cognition",
        store_dir=tmp_path / "store",
        homes=SeatHomes(tmp_path / "store", MemoryConfig()),
        clock=clock.time,
        sleep=clock.sleep,
    )


def _requests(tmp_path: Path, *scopes: MemoryScopeChoice) -> list[CompletionRequest]:
    backend = ScriptedAnswers(nothing)
    seat = LmSeat(key=ALICE, memory=MemoryPolicyChoice(policy="consolidate", scopes=scopes))

    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(_runner(tmp_path, link, backend, seat).run())
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5", utterance="hello there")))
        await logged(tmp_path, 1)
        await cancel(task)

    run(scenario())
    return backend.requests


@pytest.fixture(autouse=True)
def with_the_hostile_scope_installed(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(ScopeRegistry, "installed", staticmethod(_registry))
    Cites.reset()


def test_a_hostile_scope_is_withheld_logged_and_never_shown(tmp_path: Path) -> None:
    Cites.lines.append(
        SectionLine(level="cites", text="a secret", citation=Citation.of(EventId("99")))
    )
    hostile = _requests(tmp_path / "hostile", PERSON, MemoryScopeChoice(id="cites"))
    plain = _requests(tmp_path / "plain", PERSON)
    Cites.reset()
    assert hostile and plain
    assert hostile[0].model_dump_json() == plain[0].model_dump_json()
    assert "a secret" not in hostile[0].model_dump_json()
    log = (tmp_path / "hostile" / "store" / "alice.decisions.jsonl").read_text(encoding="utf-8")
    defects = [json.loads(line) for line in log.splitlines() if '"scope-defect"' in line]
    assert [(d["seat"], d["consult"], d["scope"]) for d in defects] == [("alice", 1, "cites")]
    assert defects[0]["error"].startswith("AccessDenied")
    assert len(decisions(tmp_path / "hostile")) == 1


def test_a_seat_whose_stack_lists_world_is_refused_before_it_plays(tmp_path: Path) -> None:
    seat = LmSeat(
        key=ALICE,
        memory=MemoryPolicyChoice(
            policy="consolidate",
            scopes=(MemoryScopeChoice(id="person"), MemoryScopeChoice(id="world")),
        ),
    )
    with pytest.raises(ScopeNotSelectable):
        _runner(tmp_path, Link(welcome(resume=1), observation(1)), ScriptedAnswers(nothing), seat)
