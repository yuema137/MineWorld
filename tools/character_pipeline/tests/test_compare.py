"""The parity comparison (design §5.7, AB1-8's instrument): one joint, weight or vertex order at a time.

Fixture GLBs carry the geometry (joint positions through the inverse bind matrices). Weights and vertex order
are edited on the measured quantities, which is where the comparison reads them."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

from gltf_fixture import build_glb

from mwcast.compare import Measured, compare_files, compare_measured, measure
from mwcast.gates import Verdict
from mwcast.standard import ACCEPTED_ASSET

REPO = Path(__file__).resolve().parents[3]
JOINTS = (("Hips", None), ("Spine", "Hips"))


def _write(tmp_path: Path, name: str, data: bytes) -> Path:
    path = tmp_path / name
    path.write_bytes(data)
    return path


def _measured(tmp_path: Path, name: str, *, triangles: int) -> Measured:
    return measure(_write(tmp_path, name, build_glb(triangles=triangles, joints=JOINTS)))


def test_identical_exports_pass_every_row(tmp_path: Path) -> None:
    path = _write(tmp_path, "a.glb", build_glb(joints=JOINTS, joint_positions={"Spine": (0.0, 0.5, 0.0)}))

    report = compare_files(path, path)

    assert report.passed
    assert all(row.verdict is Verdict.PASS for row in report.rows)


def test_a_joint_moved_1_1_mm_fails_and_names_it(tmp_path: Path) -> None:
    accepted = _write(
        tmp_path, "accepted.glb", build_glb(joints=JOINTS, joint_positions={"Spine": (0.0, 0.5, 0.0)})
    )
    candidate = _write(
        tmp_path, "cand.glb", build_glb(joints=JOINTS, joint_positions={"Spine": (0.0011, 0.5, 0.0)})
    )

    report = compare_files(candidate, accepted)

    assert not report.passed
    joint_row = next(row for row in report.rows if row.quantity == "joint positions (m)")
    assert joint_row.verdict is Verdict.FAIL
    assert any("Spine" in reason for reason in joint_row.reasons)


def test_a_joint_moved_0_9_mm_passes(tmp_path: Path) -> None:
    accepted = _write(
        tmp_path, "accepted.glb", build_glb(joints=JOINTS, joint_positions={"Spine": (0.0, 0.5, 0.0)})
    )
    candidate = _write(
        tmp_path, "cand.glb", build_glb(joints=JOINTS, joint_positions={"Spine": (0.0009, 0.5, 0.0)})
    )

    assert compare_files(candidate, accepted).passed


def test_a_weight_moved_0_006_fails_and_0_004_passes(tmp_path: Path) -> None:
    base = _measured(tmp_path, "base.glb", triangles=2)
    weights = list(base.weights)
    weights[3] = {"Hips": 1.0 - 0.006, "Spine": 0.006}
    moved = replace(base, weights=tuple(weights))
    assert not compare_measured(moved, base).passed
    weights[3] = {"Hips": 1.0 - 0.004, "Spine": 0.004}
    nearly = replace(base, weights=tuple(weights))
    assert compare_measured(nearly, base).passed


def test_a_reordered_vertex_pair_fails_with_vertex_order_differs(tmp_path: Path) -> None:
    base = _measured(tmp_path, "base.glb", triangles=2)
    swapped_positions = (base.positions[1], base.positions[0], *base.positions[2:])
    swapped = replace(base, positions=swapped_positions)

    report = compare_measured(swapped, base)

    assert not report.passed
    position_row = next(row for row in report.rows if row.quantity == "vertex positions (m)")
    assert any("vertex order differs" in reason for reason in position_row.reasons)


def test_different_triangle_counts_fail_first_and_stop(tmp_path: Path) -> None:
    three = _write(tmp_path, "three.glb", build_glb(triangles=3, joints=JOINTS))
    two = _write(tmp_path, "two.glb", build_glb(triangles=2, joints=JOINTS))

    report = compare_files(three, two)

    assert [row.quantity for row in report.rows] == ["triangles"]
    assert not report.passed


def test_the_accepted_asset_compares_equal_to_itself() -> None:
    accepted = REPO / ACCEPTED_ASSET

    assert compare_files(accepted, accepted).passed
