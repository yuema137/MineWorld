"""I-c-14 (AC-S27-14, the adapter half): the pacer sends at most one `walk-step` per wall second, one in
flight, none while the world is paused or before the newest observation reflects the last step, and it
stops when the walk is gone or a step is answered with an end. A virtual wall clock and sleep replace real
time, so cadence is measured exactly; the scripted `step` stands for the Player's submit."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field

from support import run

from mineworld_sdk.pacer import Pacer, Step, Wait, Walking

SECOND = 1.0
"""The rule, stated here and not read from the module under test (test rules §25)."""


@dataclass
class Wall:
    """A virtual wall clock; `sleep` advances it and yields to the loop."""

    now: float = 0.0
    slept: list[float] = field(default_factory=list[float])

    def time(self) -> float:
        return self.now

    async def sleep(self, seconds: float) -> None:
        self.slept.append(seconds)
        self.now += seconds
        await asyncio.sleep(0)


@dataclass
class World:
    """What the pacer reads, scripted: the walk lasts `steps` accepted steps."""

    wall: Wall
    steps: int
    sent: list[float] = field(default_factory=list[float])
    paused_until: float = -1.0
    answers: dict[int, Step] = field(default_factory=dict[int, Step])
    in_flight: int = 0
    most_in_flight: int = 0
    reflected: bool = True

    def walking(self) -> Walking:
        if not self.reflected:
            return "unseen"
        return "walking" if len(self.sent) < self.steps else "gone"

    def paused(self) -> bool:
        return self.wall.now < self.paused_until

    async def step(self) -> Step:
        self.in_flight += 1
        self.most_in_flight = max(self.most_in_flight, self.in_flight)
        self.sent.append(self.wall.now)
        await self.wall.sleep(0.2)  # the answer takes a fifth of a second
        self.in_flight -= 1
        return self.answers.get(len(self.sent), "stepped")


def pacer(world: World) -> Pacer:
    return Pacer(
        walking=world.walking,
        paused=world.paused,
        step=world.step,
        wall=world.wall.time,
        sleep=world.wall.sleep,
    )


def walk(world: World) -> None:
    async def go() -> None:
        paced = pacer(world)
        await paced.start()
        await paced.wait()

    run(go(), timeout_s=10)


def test_at_most_one_step_per_wall_second() -> None:
    world = World(Wall(), steps=12)
    walk(world)
    assert len(world.sent) == 12
    gaps = [b - a for a, b in zip(world.sent, world.sent[1:], strict=False)]
    assert all(gap >= SECOND - 1e-9 for gap in gaps), gaps
    span = world.sent[-1] - world.sent[0]
    assert (len(world.sent) - 1) / span <= 1.0 + 1e-9, "≤ 1 step per wall second over the walk"


def test_one_step_in_flight() -> None:
    world = World(Wall(), steps=5)
    walk(world)
    assert world.most_in_flight == 1


def test_it_stops_when_the_walk_is_gone_and_sends_nothing_after() -> None:
    world = World(Wall(), steps=3)
    walk(world)
    assert len(world.sent) == 3


def test_an_answer_that_ends_the_walk_stops_it() -> None:
    # TooFarAway, Busy, any rejection, a refusal or a lost seat: the Player answers "stop".
    world = World(Wall(), steps=50, answers={4: "stop"})
    walk(world)
    assert len(world.sent) == 4


def test_nothing_is_sent_while_paused() -> None:
    world = World(Wall(), steps=4, paused_until=30.0)
    walk(world)
    assert world.sent[0] >= 30.0, "the first step waits for the resume"
    assert len(world.sent) == 4


def test_a_courtesy_wait_is_kept_before_the_next_step() -> None:
    world = World(Wall(), steps=3, answers={1: Wait(7.5)})
    walk(world)
    assert world.sent[1] - world.sent[0] >= 7.5


def test_an_observation_that_does_not_reflect_the_walk_is_waited_for_then_given_up() -> None:
    world = World(Wall(), steps=3, reflected=False)
    walk(world)
    assert world.sent == [], "never stepped blind"
    assert world.wall.now >= 5.0, "and waited a few seconds for the observation first"


def test_stop_ends_the_walk_and_a_new_one_keeps_the_interval() -> None:
    async def go() -> World:
        world = World(Wall(), steps=100)
        paced = pacer(world)
        await paced.start()
        while len(world.sent) < 2:
            await asyncio.sleep(0)
        await paced.stop()
        assert not paced.running
        stopped_at = len(world.sent)
        await world.wall.sleep(0.1)
        assert len(world.sent) == stopped_at, "nothing after stop"
        await paced.start()
        while len(world.sent) < stopped_at + 1:
            await asyncio.sleep(0)
        await paced.stop()
        return world

    world = run(go(), timeout_s=10)
    assert world.sent[-1] - world.sent[-2] >= SECOND - 1e-9
