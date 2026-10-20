"""The frame dump and the gait gates (S36-P2): MWCF read back exactly and refused when broken; G-4w fails
exactly where a foot is planted; G-9w fails a hand inside the trunk.

Synthetic dumps only (`mwcf_fixture.py`); the real dumps are Godot's and are checked by the probe's run."""

from __future__ import annotations

from pathlib import Path
from typing import cast

import numpy as np
import pytest
from mwcf_fixture import BONES, WALK_FRAMES, body, build

from mwcast import gait
from mwcast.frames import FramesError, read
from mwcast.gates import Verdict
from mwcast.tolerances import Tolerances

TOL = Tolerances.load()


def test_a_dump_reads_back_exactly(tmp_path: Path) -> None:
    b = body()
    d = b.dump
    d.add("rest:0", b.joints, b.rest)
    d.add("stand:1", b.joints, b.rest + 0.5, full=False)
    frames = read(d.write(tmp_path / "x.mwcf"))

    assert frames.bone_names == BONES and frames.vertex_count == b.rest.shape[0]
    assert np.array_equal(frames.faces, d.faces) and np.array_equal(frames.ground, d.ground)
    assert [f.tag for f in frames.frames] == ["rest:0", "stand:1"]
    assert np.allclose(frames.full_positions(frames.frames[0]), b.rest, atol=1e-6)
    assert np.allclose(frames.ground_positions(frames.frames[1]), b.rest[d.ground] + 0.5, atol=1e-6)
    assert frames.dominant()[int(b.parts["Left foot"][0])] == "LeftFoot"
    with pytest.raises(FramesError, match="ground set only"):
        frames.full_positions(frames.frames[1])


@pytest.mark.parametrize(
    ("damage", "message"),
    [("magic", "bad magic"), ("version", "version 2"), ("truncated", "truncated"), ("index", "out of range")],
)
def test_a_broken_dump_is_refused_by_name(tmp_path: Path, damage: str, message: str) -> None:
    b = body()
    d = b.dump
    d.add("rest:0", b.joints, b.rest)
    if damage == "index":
        d.ground = np.append(d.ground, d.n + 7)
    path = d.write(
        tmp_path / "x.mwcf",
        magic=b"MWCX" if damage == "magic" else b"MWCF",
        version=2 if damage == "version" else 1,
    )
    if damage == "truncated":
        path.write_bytes(path.read_bytes()[:-10])
    with pytest.raises(FramesError, match=message):
        read(path)


def test_a_clean_body_passes_and_its_block_reads_on_the_floor(tmp_path: Path) -> None:
    frames = read(build(tmp_path))
    ground = gait.gate_g4w(frames, TOL)

    assert ground.result.verdict is Verdict.PASS, ground.result.reasons
    block = ground.block
    assert block["rest_mm"] == 0.0 and block["worst_sink_mm"] == 0.0
    feet = cast(dict[str, dict[str, dict[str, float]]], block["feet"])
    walk = feet["left"]["walk"]
    # stance is half the cycle; the frames at the turn are neither (the central difference averages them)
    assert 0.35 <= walk["planted_share"] <= 0.5 and walk["planted_float_mm"] == 0.0
    assert feet["right"]["stand"]["planted_share"] == 1.0


def _planted_frame(tmp_path: Path, side: str) -> int:
    down = gait.planted(read(build(tmp_path, name="probe.mwcf")), "walk", side)
    return int(np.flatnonzero(down)[len(np.flatnonzero(down)) // 2])


def test_a_foot_sinking_6_mm_in_one_planted_frame_fails_naming_side_clip_and_frame(tmp_path: Path) -> None:
    k = _planted_frame(tmp_path, "left")
    frames = read(build(tmp_path, sink={("walk", "left", k): 0.006}))
    result = gait.gate_g4w(frames, TOL).result

    assert result.verdict is Verdict.FAIL
    assert result.reasons == (
        f"G-4w left foot, walk frame {k}, planted, sink 6.0 mm (bound 5 mm, floor.sink.foot)",
    )


def test_a_foot_sinking_4_mm_passes(tmp_path: Path) -> None:
    k = _planted_frame(tmp_path, "left")
    frames = read(build(tmp_path, sink={("walk", "left", k): 0.004}))

    assert gait.gate_g4w(frames, TOL).result.verdict is Verdict.PASS


def test_a_planted_foot_floating_12_mm_for_six_frames_fails_and_the_same_lift_in_swing_passes(
    tmp_path: Path,
) -> None:
    probe = read(build(tmp_path, name="probe.mwcf"))
    down = gait.planted(probe, "walk", "right")
    stance = [int(k) for k in np.flatnonzero(down)][:6]
    swing = [int(k) for k in np.flatnonzero(~down)][:6]

    planted_lift = read(build(tmp_path, lift={("walk", "right", k): 0.012 for k in stance}))
    result = gait.gate_g4w(planted_lift, TOL).result
    assert result.verdict is Verdict.FAIL
    assert any(
        r.startswith("G-4w right foot, walk frames")
        and "planted, float 12.0 mm" in r
        and "floor.float.planted" in r
        for r in result.reasons
    ), result.reasons
    assert not any("left foot" in r for r in result.reasons)

    swing_lift = read(build(tmp_path, lift={("walk", "right", k): 0.012 for k in swing}, name="swing.mwcf"))
    assert gait.gate_g4w(swing_lift, TOL).result.verdict is Verdict.PASS


def test_planted_follows_the_gait_speed(tmp_path: Path) -> None:
    frames = read(build(tmp_path))
    left = gait.planted(frames, "walk", "left")
    right = gait.planted(frames, "walk", "right")

    assert left.shape == (WALK_FRAMES,)
    # the right foot is half a cycle behind the left: planted in the left's swing
    assert np.array_equal(np.roll(left, WALK_FRAMES // 2), right)
    assert gait.planted(frames, "stand", "left").all()


def test_a_dump_without_a_jog_cannot_be_judged(tmp_path: Path) -> None:
    b = body()
    b.dump.add("rest:0", b.joints, b.rest)
    b.dump.add("stand:0", b.joints, b.rest)
    b.dump.add("walk:0", b.joints, b.rest, speed=1.2)
    result = gait.gate_g4w(read(b.dump.write(tmp_path / "x.mwcf")), TOL).result

    assert (
        result.verdict is Verdict.FAIL and "cannot look" in result.reasons[0] and "jog" in result.reasons[0]
    )


def test_g9w_passes_hanging_arms_and_fails_a_hand_8_mm_inside_the_trunk(tmp_path: Path) -> None:
    clean = gait.gate_g9w(read(build(tmp_path, name="clean.mwcf")), TOL)
    assert clean.result.verdict is Verdict.PASS, clean.result.reasons

    # the hand's innermost corner (x 0.27 m) moved to 8 mm inside the trunk's side (x 0.18 m)
    planted = gait.gate_g9w(read(build(tmp_path, arm_in=0.098)), TOL)
    assert planted.result.verdict is Verdict.FAIL
    limbs = planted.block["limb_in_trunk_mm"]
    assert isinstance(limbs, dict)
    assert limbs["hand"] == pytest.approx(8.0, abs=0.1)
    assert any(
        "Left hand, walk frames 0–29" in r and "self.hand_in_trunk" in r for r in planted.result.reasons
    )


def test_a_planted_blip_shorter_than_a_stance_is_swing_and_a_run_may_wrap_the_cycle() -> None:
    raw = np.asarray([1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1], dtype=bool)
    kept = gait._phases(raw)  # pyright: ignore[reportPrivateUsage]

    # frames 11, 0, 1 are one cyclic run of 3; 4–5 is a 2-frame blip; 7–9 a stance
    assert kept.tolist() == [1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1]
