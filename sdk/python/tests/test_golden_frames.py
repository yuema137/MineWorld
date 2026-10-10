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

from mineworld_sdk.errors import MalformedFrame, UnsupportedFrame
from mineworld_sdk.wire import codec
from mineworld_sdk.wire.contract import (
    ActionRecord,
    ActionRequest,
    Affordance,
    Observation,
    SpatialRequirement,
)
from mineworld_sdk.wire.frames import Invite, Join, Leave, ObservationFrame, Submit
from mineworld_sdk.wire.ids import (
    ActionTypeId,
    CorrelationToken,
    EntityId,
    EntityKey,
    JsonValue,
)

FRAMES = Path(__file__).resolve().parents[3] / "server" / "tests" / "frames"


def golden(kind: str) -> tuple[str, JsonValue]:
    text = (FRAMES / f"{kind}.json").read_text(encoding="utf-8")
    value: JsonValue = json.loads(text)
    return text, value


def server_frame(kind: str) -> None:
    """The file decodes, and what it decodes to encodes back to the file."""
    text, expected = golden(kind)
    frame = codec.decode(text)
    assert frame.t == kind
    assert codec.to_json(frame) == expected, f"{kind}.json no longer round-trips"


def join() -> None:
    _, expected = golden("join")
    frame = Join(
        invite=Invite("3f9c0a1b2c3d4e5f60718293a4b5c6d7"),
        nickname="Yue",
        seat=EntityKey("visitor"),
    )
    assert json.loads(codec.encode(frame)) == expected


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
}


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
        '{"t":"observation","seq":3,"revision":null,"observation":{"observer":"101","at":5,'
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


def test_an_observation_carrying_events_is_unsupported_until_s11_c() -> None:
    # D-P3-6: loud, not silently dropped.
    text = observation_text("").replace('"events":[]', '"events":[{"event_type":"spoke"}]')
    with pytest.raises(UnsupportedFrame, match="S11-C"):
        codec.decode(text)
    # The guard is on the model, not only on the decoder.
    with pytest.raises(UnsupportedFrame):
        Observation.model_validate_json(
            '{"observer":"1","at":0,"self_location":null,"entities":[],"relations":[],'
            '"events":[{}],"affordances":[]}'
        )
