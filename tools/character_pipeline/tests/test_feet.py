"""The foot-grounding judge (S36-P4 commit 3): a planted foot is held to the tolerance file's bounds, a
swinging foot and an unreached step are not, and the CLI's verdict follows the judge."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from mwcast import cli
from mwcast.feet import FeetError, judge
from mwcast.tolerances import Tolerances


def _frame(
    n: int, segment: str, left: tuple[float, bool], right: tuple[float, bool], **extra: bool
) -> dict[str, object]:
    def foot(row: tuple[float, bool]) -> dict[str, object]:
        return {"height_mm": row[0], "planted": row[1], "influence": 1.0, **extra}

    return {
        "frame": n,
        "segment": segment,
        "lift_mm": 0.0,
        "feet": {"left": foot(left), "right": foot(right)},
    }


def _report(*frames: dict[str, object], body: str = "bob") -> dict[str, object]:
    frames = tuple(f for frame in frames for f in (frame, frame, frame))
    return {
        "format": 1,
        "seed": 1,
        "ik": True,
        "scene": "course",
        "bodies": [{"id": body, "frames": list(frames)}],
    }


def test_a_planted_foot_6_mm_under_fails_naming_body_foot_segment_and_frame() -> None:
    verdict = judge(_report(_frame(7, "kerb_up", (-6.0, True), (0.0, True))), Tolerances.load())
    assert not verdict.passed
    assert sorted({f.line() for f in verdict.failures}) == [
        "FAIL bob left foot, segment kerb_up, frame 7: -6.0 mm (sinks past 5.0 mm)"
    ]


def test_a_planted_foot_4_mm_under_and_a_swing_foot_40_mm_up_pass() -> None:
    verdict = judge(_report(_frame(1, "ramp", (-4.0, True), (40.0, False))), Tolerances.load())
    assert verdict.passed
    assert verdict.extents[("bob", "left", "ramp")].sink_mm == -4.0
    assert verdict.extents[("bob", "right", "ramp")].planted == 0


def test_a_planted_float_past_the_bound_fails_and_an_unreached_one_is_listed() -> None:
    floating = judge(_report(_frame(2, "stairs", (0.0, True), (11.0, True))), Tolerances.load())
    assert {f.why for f in floating.failures} == {"floats past 10.0 mm"}
    unreached = judge(
        _report(_frame(3, "stairs", (0.0, True), (-140.0, True), unreached=True)), Tolerances.load()
    )
    assert unreached.passed
    assert sorted(set(unreached.unreached)) == [
        "bob frame 3 left, segment stairs: unreached (+0.0 mm)",
        "bob frame 3 right, segment stairs: unreached (-140.0 mm)",
    ]


def test_the_bounds_come_from_the_tolerance_file(tmp_path: Path) -> None:
    pack = tmp_path / "pack.json"
    pack.write_text(
        json.dumps(
            {
                "format": 1,
                "regions": {
                    "floor.sink.foot": {
                        "bound_mm": 3,
                        "rationale": "a pack that tightens",
                        "source": "test",
                        "status": "inconclusive",
                    }
                },
            }
        ),
        encoding="utf-8",
    )
    report = _report(_frame(1, "flat", (-4.0, True), (0.0, True)))
    assert judge(report, Tolerances.load()).passed
    assert not judge(report, Tolerances.load().tightened_by(pack)).passed


def test_a_malformed_report_is_refused_by_field() -> None:
    empty: dict[str, object] = {}
    with pytest.raises(FeetError, match="height_mm"):
        judge(
            {
                "format": 1,
                "bodies": [{"id": "a", "frames": [{"frame": 0, "feet": {"left": empty, "right": empty}}]}],
            },
            Tolerances.load(),
        )
    with pytest.raises(FeetError, match="format"):
        judge({"format": 2, "bodies": []}, Tolerances.load())


def test_the_cli_judges_a_file_and_fails_on_a_sunk_planted_foot(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    good = tmp_path / "good.json"
    good.write_text(json.dumps(_report(_frame(0, "flat", (0.5, True), (30.0, False)))), encoding="utf-8")
    bad = tmp_path / "bad.json"
    bad.write_text(json.dumps(_report(_frame(4, "kerb_down", (0.0, True), (-140.0, True)))), encoding="utf-8")
    assert cli.main(["feet", str(good)]) == 0
    assert cli.main(["feet", str(bad)]) == 1
    out = capsys.readouterr().out
    assert "FAIL bob right foot, segment kerb_down, frame 4: -140.0 mm" in out
    assert "feet: PASS" in out and "feet: FAIL" in out


CLIENT = Path(__file__).resolve().parents[3] / "clients/3d-spike"


def test_foot_grounding_sends_nothing_and_writes_no_authoritative_state() -> None:
    """AC-5 (I-S36-6): the grounding names no link, no intent and no observation writer."""
    source = (CLIENT / "scripts/foot_grounding.gd").read_text(encoding="utf-8")
    code = "\n".join(line.split("##")[0].split("#")[0] for line in source.splitlines())
    for word in ("SliceLink", "Intent", "intent", "Observation", "observation", "rpc", "send", "WebSocket"):
        assert word not in code, word


def test_every_tunable_has_a_value_a_reason_and_a_status() -> None:
    """I-P4-2: the tunables are data with their reasons; the file and the script's key list agree."""
    document = json.loads((CLIENT / "assets/characters/foot_grounding.json").read_text(encoding="utf-8"))
    assert document["format"] == 1
    source = (CLIENT / "scripts/foot_grounding.gd").read_text(encoding="utf-8")
    for key, row in document["tunables"].items():
        assert f'"{key}"' in source, key
        assert isinstance(row["value"], int | float) and row["reason"].strip(), key
        assert row["status"] in {"cited", "inherited", "inconclusive"}, key


def test_a_planted_frame_outside_a_run_of_three_is_not_judged() -> None:
    """S36-P2's planted rule: a foot passing the drift rule for one frame in mid-swing is not planted."""
    sunk = _frame(0, "flat", (-40.0, True), (0.0, True))
    swing = _frame(1, "flat", (-40.0, False), (0.0, True))
    report = {"format": 1, "bodies": [{"id": "bob", "frames": [swing, sunk, swing, swing]}]}
    assert judge(report, Tolerances.load()).passed
