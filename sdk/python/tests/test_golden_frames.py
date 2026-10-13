"""AP-1 and AP-3: the Python models encode exactly what the Rust types write.

The oracle is `server/tests/frames/*.json`, the reviewed files `server/tests/frames.rs` checks the Rust
types against (SD-A12). They are read here as the Rust test reads them: a server frame decodes and
re-encodes to a JSON value equal to the file; a client frame, built from its model, encodes to it. The
values are compared as parsed JSON, never as text, so key order and line endings do not matter
(D-P3-11 (d)).
"""

from __future__ import annotations

import json
from collections.abc import Callable
from pathlib import Path

import pytest

from mineworld_sdk.errors import MalformedFrame
from mineworld_sdk.wire import codec
from mineworld_sdk.wire.contract import (
    ActionRecord,
    ActionRequest,
    Affordance,
    Observation,
    SpatialRequirement,
)
from mineworld_sdk.wire.delta import DeltaMismatch, ObservationDelta, apply_delta
from mineworld_sdk.wire.frames import Invite, Join, Leave, ObservationFrame, PerceivedJoin, Submit
from mineworld_sdk.wire.ids import (
    ActionTypeId,
    CorrelationToken,
    EntityId,
    EntityKey,
    EventId,
    JsonValue,
)

FRAMES = Path(__file__).resolve().parents[3] / "server" / "tests" / "frames"


def golden(kind: str) -> tuple[str, JsonValue]:
    text = (FRAMES / f"{kind}.json").read_text(encoding="utf-8")
    value: JsonValue = json.loads(text)
    return text, value


def server_frame(kind: str, t: str | None = None) -> None:
    """The file decodes, and what it decodes to encodes back to the file. `t` is the frame kind when
    the file is one example of a kind among several (`refused-lagged.json` is a `refused`)."""
    text, expected = golden(kind)
    frame = codec.decode(text)
    assert frame.t == (t or kind)
    assert codec.to_json(frame) == expected, f"{kind}.json no longer round-trips"


def join() -> None:
    _, expected = golden("join")
    frame = Join(
        invite=Invite("3f9c0a1b2c3d4e5f60718293a4b5c6d7"),
        nickname="Yue",
        seat=EntityKey("visitor"),
        perceived=PerceivedJoin(since=EventId("1873")),
    )
    assert json.loads(codec.encode(frame)) == expected
    # Without `perceived`, the frame carries no such field (an older server refuses it, §10).
    plain = Join(invite=Invite("x" * 8), nickname="Yue", seat=EntityKey("visitor"))
    assert "perceived" not in json.loads(codec.encode(plain))


def submit() -> None:
    _, expected = golden("submit")
    talk = ActionTypeId("talk")
    frame = Submit(
        token=CorrelationToken("c1"),
        request=ActionRequest(
            actor=EntityId("101"),
            action_type=talk,
            target=EntityId("9007199254740995"),
            payload=ActionRecord(action_type=talk, payload={"utterance": "hello"}),
        ),
    )
    assert json.loads(codec.encode(frame)) == expected


def leave() -> None:
    _, expected = golden("leave")
    assert json.loads(codec.encode(Leave())) == expected


# One check per golden file. A file without an entry fails `test_every_golden_frame_has_a_model`.
CHECKS: dict[str, Callable[[], None]] = {
    "join": join,
    "submit": submit,
    "leave": leave,
    "welcome": lambda: server_frame("welcome"),
    "observation": lambda: server_frame("observation"),
    "result": lambda: server_frame("result"),
    "refused": lambda: server_frame("refused"),
    "closing": lambda: server_frame("closing"),
    "clock": lambda: server_frame("clock"),
    "perceived": lambda: server_frame("perceived"),
    "delta": lambda: server_frame("delta"),
    "refused-cursor_unavailable": lambda: server_frame("refused-cursor_unavailable", "refused"),
    "refused-lagged": lambda: server_frame("refused-lagged", "refused"),
    "closing-lagged": lambda: server_frame("closing-lagged", "closing"),
}

DELTAS = FRAMES / "deltas"


@pytest.mark.parametrize("case", sorted(path.stem for path in DELTAS.glob("*.json")))
def test_golden_delta_case(case: str) -> None:
    # The reviewed `{ base, delta, next }` cases the Rust and GDScript appliers are checked against
    # (step-12 CA-9): the delta decodes and re-encodes to the file, and applies to `next`.
    value = json.loads((DELTAS / f"{case}.json").read_text(encoding="utf-8"))
    base = Observation.model_validate(value["base"])
    delta = ObservationDelta.model_validate(value["delta"])
    assert codec.to_json(delta) == value["delta"], f"{case}: the delta no longer round-trips"
    assert codec.to_json(apply_delta(base, delta)) == value["next"], case


def test_there_are_golden_delta_cases() -> None:
    assert len(list(DELTAS.glob("*.json"))) >= 7, f"no reviewed delta cases under {DELTAS}"


def test_a_delta_removing_an_entity_not_held_is_a_mismatch() -> None:
    held = Observation.model_validate_json(
        '{"observer":"1","at":0,"self_location":null,"entities":[],"relations":[],"events":[],'
        '"affordances":[]}'
    )
    delta = ObservationDelta.model_validate_json('{"at":1,"entities":{"remove":["7"]},"events":[]}')
    with pytest.raises(DeltaMismatch, match="entity 7"):
        apply_delta(held, delta)


def test_every_golden_frame_has_a_model() -> None:
    files = {path.stem for path in FRAMES.glob("*.json")}
    assert files, f"no golden frames found under {FRAMES}"
    unmodelled = sorted(files - CHECKS.keys())
    vanished = sorted(CHECKS.keys() - files)
    assert not unmodelled, (
        f"golden frames with no Python model: {[f'{k}.json' for k in unmodelled]}"
    )
    assert not vanished, f"checks for golden frames that no longer exist: {vanished}"


@pytest.mark.parametrize("kind", sorted(CHECKS))
def test_golden_frame(kind: str) -> None:
    CHECKS[kind]()


def observation_text(entities: str) -> str:
    return (
        '{"t":"observation","seq":3,"revision":null,"acted_through":null,'
        '"observation":{"observer":"101","at":5,'
        f'"self_location":null,"entities":[{entities}],"relations":[],"events":[],'
        '"affordances":[]}}'
    )


def entity(id_json: str) -> str:
    return f'{{"id":{id_json},"entity_type":"person","location":null,"tags":[],"components":[]}}'


def test_ids_above_two_to_the_fifty_three_stay_distinct() -> None:
    # AP-3: as doubles, both are 9007199254740996 (spike/FINDINGS.md F1).
    text = observation_text(entity('"9007199254740995"') + "," + entity('"9007199254740997"'))
    frame = codec.decode(text)
    assert isinstance(frame, ObservationFrame)
    ids = [perceived.id for perceived in frame.observation.entities]
    assert ids == ["9007199254740995", "9007199254740997"]
    assert json.loads(text) == codec.to_json(frame)


def test_an_id_written_as_a_number_is_refused() -> None:
    with pytest.raises(MalformedFrame, match="entities"):
        codec.decode(observation_text(entity("9007199254740995")))


def requirement() -> SpatialRequirement:
    return SpatialRequirement(
        place="any",
        within_range=None,
        requires_line_of_access=False,
        requires_target_available=False,
    )


def test_an_affordance_available_with_a_reason_or_unavailable_without_one_is_refused() -> None:
    ring = ActionTypeId("ring")
    with pytest.raises(ValueError, match="must not carry an unavailable_reason"):
        Affordance(
            action_type=ring,
            target=None,
            available=True,
            unavailable_reason="busy",
            requirement=requirement(),
        )
    with pytest.raises(ValueError, match="must carry an unavailable_reason"):
        Affordance(
            action_type=ring,
            target=None,
            available=False,
            unavailable_reason=None,
            requirement=requirement(),
        )


def test_a_request_whose_two_action_types_disagree_is_refused() -> None:
    with pytest.raises(ValueError, match="disagrees with its payload"):
        ActionRequest(
            actor=EntityId("1"),
            action_type=ActionTypeId("talk"),
            payload=ActionRecord(action_type=ActionTypeId("move"), payload={}),
        )


def test_a_fact_whose_two_event_types_disagree_is_refused() -> None:
    # EventEnvelopeFields' agreement check, mirrored: the envelope and its record name one type.
    _, value = golden("perceived")
    assert isinstance(value, dict) and isinstance(value["events"], list)
    event = value["events"][0]
    assert isinstance(event, dict)
    tampered = json.dumps({**value, "events": [{**event, "event_type": "tolled"}]})
    with pytest.raises(MalformedFrame, match="disagrees with its payload"):
        codec.decode(tampered)
