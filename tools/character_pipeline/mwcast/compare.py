"""The parity comparison of two exports (design §5.7; PIPELINE.md §7).

`compare` answers one question: does the candidate reproduce the accepted asset within the bounds of
`standard.py`? It compares counts, joint names and world positions, vertex positions and UVs by index, and
weights by vertex and joint name. It never re-matches vertices by search: a re-ordered mesh is a reproduction
failure to diagnose, so the verdict says "vertex order differs". Material names and provenance are reported,
not compared (F-B1-6).
"""

from __future__ import annotations

import math
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path

from mwcast import glb as glb_reader
from mwcast import standard
from mwcast.gates import Verdict

Vec3 = tuple[float, float, float]


@dataclass(frozen=True)
class Measured:
    """The quantities `compare` reads from one export, in index order."""

    triangles: int
    joint_names: frozenset[str]
    joints: Mapping[str, Vec3]
    positions: tuple[Vec3, ...]
    uvs: tuple[tuple[float, float], ...]
    weights: tuple[Mapping[str, float], ...]


@dataclass(frozen=True)
class Row:
    """One compared quantity: the worst offender it found, its bound, and the verdict (before the message)."""

    quantity: str
    worst: str
    bound: str
    verdict: Verdict
    reasons: tuple[str, ...]


@dataclass(frozen=True)
class ParityReport:
    rows: tuple[Row, ...]

    @property
    def passed(self) -> bool:
        return all(row.verdict is Verdict.PASS for row in self.rows)


def measure(path: Path) -> Measured:
    """Read one GLB into the quantities the comparison uses."""
    glb = glb_reader.read_glb(path)
    return Measured(
        triangles=glb_reader.triangle_count(glb),
        joint_names=frozenset(glb_reader.skin_joint_names(glb)),
        joints=glb_reader.joint_world_positions(glb),
        positions=tuple(glb_reader.positions(glb)),
        uvs=tuple(glb_reader.uvs(glb)),
        weights=tuple(glb_reader.vertex_weights(glb)),
    )


def _distance(a: Vec3, b: Vec3) -> float:
    return math.dist(a, b)


def _max_abs(a: tuple[float, ...], b: tuple[float, ...]) -> float:
    return max(abs(x - y) for x, y in zip(a, b, strict=True))


def _verdict(passed: bool) -> Verdict:
    return Verdict.PASS if passed else Verdict.FAIL


def _count_row(name: str, a: int, b: int) -> Row:
    return Row(
        name, f"{a} vs {b}", "equal", _verdict(a == b), () if a == b else (f"{name} differ: {a} vs {b}",)
    )


def compare_measured(candidate: Measured, accepted: Measured) -> ParityReport:
    """The rows of §5.7 for two measured exports.

    Different triangle counts fail first and stop the comparison."""
    triangles = _count_row("triangles", candidate.triangles, accepted.triangles)
    if triangles.verdict is Verdict.FAIL:
        return ParityReport((triangles,))
    vertices = _count_row("vertices", len(candidate.positions), len(accepted.positions))
    rows: list[Row] = [triangles, vertices, _joint_name_row(candidate, accepted)]
    if vertices.verdict is Verdict.FAIL:
        return ParityReport(tuple(rows))
    rows.append(_joint_row(candidate, accepted))
    rows.append(_position_row(candidate, accepted))
    rows.append(_uv_row(candidate, accepted))
    rows.append(_weight_row(candidate, accepted))
    return ParityReport(tuple(rows))


def _joint_name_row(candidate: Measured, accepted: Measured) -> Row:
    missing = sorted(accepted.joint_names - candidate.joint_names)
    unexpected = sorted(candidate.joint_names - accepted.joint_names)
    reasons: list[str] = []
    if missing:
        reasons.append(f"joints missing from the candidate: {', '.join(missing)}")
    if unexpected:
        reasons.append(f"joints unexpected in the candidate: {', '.join(unexpected)}")
    worst = f"{len(candidate.joint_names)} vs {len(accepted.joint_names)} names"
    return Row("joint names", worst, "equal sets", _verdict(not reasons), tuple(reasons))


def _joint_row(candidate: Measured, accepted: Measured) -> Row:
    shared = sorted(set(candidate.joints) & set(accepted.joints))
    if not shared:
        return Row("joint positions (m)", "no shared joints", f"{standard.PARITY_JOINT_M}", Verdict.FAIL, ())
    worst_name = max(shared, key=lambda n: _distance(candidate.joints[n], accepted.joints[n]))
    worst = _distance(candidate.joints[worst_name], accepted.joints[worst_name])
    reasons: tuple[str, ...] = ()
    if worst > standard.PARITY_JOINT_M:
        reasons = (f"joint {worst_name} is {worst:.6f} m from the accepted position",)
    return Row(
        "joint positions (m)",
        f"{worst:.6f} at {worst_name}",
        f"<= {standard.PARITY_JOINT_M}",
        _verdict(worst <= standard.PARITY_JOINT_M),
        reasons,
    )


def _position_row(candidate: Measured, accepted: Measured) -> Row:
    pairs = list(zip(candidate.positions, accepted.positions, strict=True))
    if not pairs:
        return Row(
            "vertex positions (m)", "no vertices", f"<= {standard.PARITY_POSITION_M}", Verdict.FAIL, ()
        )
    diffs = [_max_abs(a, b) for a, b in pairs]
    worst_index = max(range(len(diffs)), key=lambda i: diffs[i])
    worst = diffs[worst_index]
    reasons: list[str] = []
    if diffs[0] > standard.PARITY_POSITION_M:
        reasons.append(f"vertex order differs: vertex 0 differs by {diffs[0]:.3e} m")
    elif worst > standard.PARITY_POSITION_M:
        reasons.append(f"vertex {worst_index} differs by {worst:.3e} m")
    return Row(
        "vertex positions (m)",
        f"{worst:.3e} at vertex {worst_index}",
        f"<= {standard.PARITY_POSITION_M}",
        _verdict(not reasons),
        tuple(reasons),
    )


def _uv_row(candidate: Measured, accepted: Measured) -> Row:
    if len(candidate.uvs) != len(accepted.uvs):
        return Row("uv", "counts differ", f"<= {standard.PARITY_UV}", Verdict.FAIL, ("UV counts differ",))
    diffs = [_max_abs(a, b) for a, b in zip(candidate.uvs, accepted.uvs, strict=True)]
    worst = max(diffs, default=0.0)
    worst_index = diffs.index(worst) if diffs else 0
    reasons: tuple[str, ...] = ()
    if worst > standard.PARITY_UV:
        reasons = (f"UV {worst_index} differs by {worst:.3e}",)
    return Row(
        "uv", f"{worst:.3e} at {worst_index}", f"<= {standard.PARITY_UV}", _verdict(not reasons), reasons
    )


def _weight_row(candidate: Measured, accepted: Measured) -> Row:
    if len(candidate.weights) != len(accepted.weights):
        return Row(
            "weights",
            "counts differ",
            f"<= {standard.PARITY_WEIGHT}",
            Verdict.FAIL,
            ("vertex counts differ",),
        )
    worst = 0.0
    where = (0, "")
    for index, (a, b) in enumerate(zip(candidate.weights, accepted.weights, strict=True)):
        for joint in set(a) | set(b):
            diff = abs(a.get(joint, 0.0) - b.get(joint, 0.0))
            if diff > worst:
                worst = diff
                where = (index, joint)
    reasons: tuple[str, ...] = ()
    if worst > standard.PARITY_WEIGHT:
        reasons = (f"weight of vertex {where[0]} for {where[1]} differs by {worst:.4f}",)
    return Row(
        "weights",
        f"{worst:.4f} at vertex {where[0]} for {where[1] or '-'}",
        f"<= {standard.PARITY_WEIGHT}",
        _verdict(not reasons),
        reasons,
    )


def compare_files(candidate: Path, accepted: Path) -> ParityReport:
    return compare_measured(measure(candidate), measure(accepted))
