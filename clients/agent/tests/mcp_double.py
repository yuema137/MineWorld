"""A `Player` stand-in for the adapter's in-process tests (S27-d C3): each operation answers what the test set for
it, or raises the exception it was given. The adapter calls the same thirteen names and `inbox`; nothing else of
`Player` is used, so the double is faithful to the adapter's contract and nothing more."""

from __future__ import annotations

from collections.abc import Awaitable, Callable, Mapping
from types import SimpleNamespace
from typing import cast

from mineworld_agent.models import AgentModel
from mineworld_agent.player import Player

OPERATIONS: tuple[str, ...] = (
    "seats",
    "join",
    "observe",
    "inspect",
    "events",
    "wait",
    "say",
    "walk_to",
    "move",
    "stop",
    "attempt",
    "clock",
    "leave",
    "inbox",
)

type Answer = AgentModel | BaseException


def fake_player(answers: Mapping[str, Answer]) -> Player:
    """A `Player` whose operation `name` returns `answers[name]`, or raises it when it is an exception."""

    def responder(name: str) -> Callable[[AgentModel], Awaitable[AgentModel]]:
        async def respond(args: AgentModel) -> AgentModel:
            del args
            answer = answers[name]
            if isinstance(answer, BaseException):
                raise answer
            return answer

        return respond

    return cast(Player, SimpleNamespace(**{name: responder(name) for name in OPERATIONS}))
