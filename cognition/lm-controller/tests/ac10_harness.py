"""The `AC-10` scenario's inputs and oracles (IC-4; pr-s10-p4 AP4-10), computed from the real export.

`History.build` runs the real binary: a 100-day `mineworld run` of social-cafe at seed 7 into a scratch
directory, `mineworld inspect` for the world instance, and `mineworld perceived --json` for Alice's and
Bob's perceived facts. The save is read only through the CLI. Every oracle here is a function of those
exports alone: nothing in this module imports `mineworld_cognition`, and facts are located by rule,
never by literal ids or counts (`ARC-23`), so a change of world behaviour moves the oracles with it.
"""

from __future__ import annotations

import re
import time
from collections.abc import Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path

from binary import REPOSITORY, mineworld
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId, JsonValue, WorldInstanceId

WORLD = "worlds/social-cafe"
SEED = 7
DAYS = 100
TRIGGER_DAYS = (10, 30, 60, 100)
DAY_S = 86400
NOON_S = 43200
QUOTE_PREFIX = 90
"""Characters of an unperceived utterance searched for in the store: fewer than a gist keeps of a quote
(99), and enough to tell apart the long nested utterances social-cafe's people repeat to each other."""


def _payload(fact: PerceivedEvent) -> dict[str, JsonValue]:
    payload = fact.payload.payload
    assert isinstance(payload, dict), fact.id
    return payload


def _entity(reference: JsonValue) -> EntityId:
    assert isinstance(reference, dict)
    entity = reference["entity"]
    assert isinstance(entity, str)
    return EntityId(entity)


def utterance(fact: PerceivedEvent) -> str:
    text = _payload(fact)["utterance"]
    assert isinstance(text, str)
    return text


def listener(fact: PerceivedEvent) -> EntityId:
    return _entity(_payload(fact)["listener"])


def speaker(fact: PerceivedEvent) -> EntityId:
    return _entity(_payload(fact)["speaker"])


def _export(save: Path, person: str) -> list[PerceivedEvent]:
    out = mineworld("perceived", WORLD, "--save", save, "--person", person, "--json")
    return [PerceivedEvent.model_validate_json(line) for line in out.splitlines() if line]


@dataclass(frozen=True)
class History:
    save: Path
    instance: WorldInstanceId
    alice: list[PerceivedEvent]
    bob: list[PerceivedEvent]
    me: EntityId
    names: dict[EntityId, str]
    wall_s: dict[str, float]

    @staticmethod
    def build(scratch: Path) -> History:
        wall: dict[str, float] = {}
        save = scratch / "save"
        started = time.monotonic()
        mineworld(
            "run", WORLD, "--headless", "--seed", str(SEED), "--days", str(DAYS), "--save", save
        )
        wall["run"] = time.monotonic() - started
        found = re.search(r"instance ([0-9a-f]{32})", mineworld("inspect", save))
        assert found is not None, "mineworld inspect printed no world instance"
        started = time.monotonic()
        alice, bob = _export(save, "alice"), _export(save, "bob")
        wall["export"] = time.monotonic() - started
        names = {
            _entity(_payload(f)["person"]): str(_payload(f)["name"])
            for f in alice
            if f.event_type == "named"
        }
        yaml = (REPOSITORY / WORLD / "people" / "alice.yaml").read_text(encoding="utf-8")
        name = re.search(r"^name:\s*(.+?)\s*$", yaml, re.MULTILINE)
        assert name is not None
        (me,) = [entity for entity, known in names.items() if known == name.group(1)]
        return History(save, WorldInstanceId(found.group(1)), alice, bob, me, names, wall)

    def ids(self) -> list[EventId]:
        return [fact.id for fact in self.alice]

    def triggers(self) -> list[PerceivedEvent]:
        """Per trigger day d, the first `spoke` to Alice at or after noon of day d."""
        found: list[PerceivedEvent] = []
        for day in TRIGGER_DAYS:
            after = (day - 1) * DAY_S + NOON_S
            found.append(
                next(
                    f
                    for f in self.alice
                    if f.event_type == "spoke" and f.at >= after and listener(f) == self.me
                )
            )
        return found

    def frames(self, size: int, cuts: Sequence[EventId] = ()) -> Iterator[list[PerceivedEvent]]:
        """The export in frames of `size` facts, a frame also ending at every id in `cuts`."""
        frame: list[PerceivedEvent] = []
        for fact in self.alice:
            frame.append(fact)
            if len(frame) == size or fact.id in cuts:
                yield frame
                frame = []
        if frame:
            yield frame

    def earliest_naming(self) -> dict[EntityId, EventId]:
        """For every participant but Alice, the first of her perceived facts that names them."""
        first: dict[EntityId, EventId] = {}
        for fact in self.alice:
            for entity in fact.participants:
                if entity != self.me:
                    first.setdefault(entity, fact.id)
        return first

    def unperceived(self, count: int = 5) -> list[PerceivedEvent]:
        """`spoke` facts in Bob's export, absent from Alice's, at a place other than hers at that
        moment (from her own arrivals in the export), whose words she never heard said by anyone."""
        mine = set(self.ids())
        heard = "\n".join(utterance(f) for f in self.alice if f.event_type == "spoke")
        where: list[tuple[int, EntityId]] = []
        for fact in self.alice:
            if fact.event_type in ("arrived", "person-entered-place"):
                payload = _payload(fact)
                if _entity(payload["person"]) == self.me and fact.place is not None:
                    where.append((fact.at, fact.place.entity))
        located: list[PerceivedEvent] = []
        for fact in self.bob:
            if fact.event_type != "spoke" or fact.id in mine or fact.place is None:
                continue
            hers = [place for at, place in where if at <= fact.at]
            if not hers or hers[-1] == fact.place.entity:
                continue
            if utterance(fact)[:QUOTE_PREFIX] in heard:
                continue
            located.append(fact)
            if len(located) == count:
                break
        return located


def expand(order: dict[EventId, int], ids: list[EventId], first: str, last: str) -> list[EventId]:
    """The perceived ids a range stands for: those between its ends in Alice's perceived sequence."""
    return ids[order[EventId(first)] : order[EventId(last)] + 1]


LABELLED = [
    r"«[^»]*(?:»|$)",  # quoted world speech, possibly cut at a line's end
    r"\[#[0-9#,-]+\]",  # a citation token
    r"Day \d+, \d\d:\d\d\N{EN DASH}\d\d:\d\d",
    r"Day \d+ \d\d:\d\d",
    r"\d+ notable moments?, \d+ mine",
    r"\d+ others? came and went",
    r"Week \d+: \d+ episodes?",
    r"\(\d+\)",
    r"met on day \d+",
    r"last seen day \d+",
    r"met \d+ times?",
    r"\d+ exchanges?",
]
"""Every place a number may stand in a memory text: the summarizers' labelled fields (pr-s10-p4 §5.5),
written here as literals. AP4-4 (d) removes them and requires that no digit remains, so an entity id
cannot hide in a text however short it is."""


def unlabelled_digits(text: str) -> str | None:
    """`text` with every labelled field removed, if a digit remains; `None` when none does."""
    rest = text
    for pattern in LABELLED:
        rest = re.sub(pattern, "", rest)
    return rest if re.search(r"\d", rest) else None
