"""W-4w on a synthetic frame dump (S36-P3 commit 3): two frames of nested sleeves on an arm; in frame 2 a
patch of the inner sleeve is pushed through the outer one. The verdict names part, region, clip and frame."""

from __future__ import annotations

import math
from pathlib import Path

import numpy as np
from mwcf_fixture import BONES, Dump
from numpy.typing import NDArray

from mwcast.contact_gates import OutfitManifest, PartSpec
from mwcast.frames import read
from mwcast.gates import GateId, Verdict
from mwcast.tolerances import Tolerances
from mwcast.wardrobe.w4w import gate_w4w

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]


def _cylinder(radius: float, half: float, rings: int, segments: int, turn: float) -> tuple[Floats, Ints]:
    vertices = [
        (
            radius * math.cos(2 * math.pi * (s + turn) / segments),
            1.0 - half + 2 * half * r / rings,
            radius * math.sin(2 * math.pi * (s + turn) / segments),
        )
        for r in range(rings + 1)
        for s in range(segments)
    ]
    faces: list[tuple[int, int, int]] = []
    for r in range(rings):
        for s in range(segments):
            a, b = r * segments + s, r * segments + (s + 1) % segments
            faces += [(a, a + segments, b), (b, a + segments, b + segments)]
    return np.array(vertices, dtype=np.float64), np.array(faces, dtype=np.int64)


def _dump(path: Path, poke_m: float, uncut_poke_m: float | None = None) -> Path:
    """Two frames of an arm in a tee in a hoodie; in frame 2 a patch of the tee pokes `poke_m` through the
    hoodie. With `uncut_poke_m`, the dump also carries the uncut copies (`BodyCheck`, `PartCheck_<id>`), whose
    tee pokes that far instead."""
    arm = _cylinder(0.100, 0.30, 24, 48, 0.0)
    tee = _cylinder(0.1025, 0.21, 16, 48, 0.5)  # already pushed out: 2.5 mm off the arm
    hoodie = _cylinder(0.1050, 0.19, 16, 40, 0.25)
    parts = [arm, tee, hoodie]
    names: tuple[str, ...] = ("Body", "part_tee", "part_hoodie")
    if uncut_poke_m is not None:
        parts = parts * 2
        names += ("BodyCheck", "PartCheck_tee", "PartCheck_hoodie")
    base = np.cumsum([0] + [v.shape[0] for v, _ in parts])
    positions = np.vstack([v for v, _ in parts])
    faces = np.vstack([f + base[k] for k, (_, f) in enumerate(parts)])
    n = positions.shape[0]
    mesh = np.concatenate([np.full(v.shape[0], k, dtype=np.int64) for k, (v, _) in enumerate(parts)])
    body = (mesh == 0) | (mesh == 3)
    region = np.where(body & (np.abs(positions[:, 1] - 1.0) < 0.25), 0, -1).astype(np.int64)
    dump = Dump(
        faces=faces,
        bone=np.full(n, BONES.index("LeftUpperArm"), dtype=np.int64),
        trunk=np.zeros(n),
        limb=np.ones(n),
        ground=np.arange(4, dtype=np.int64),
        meshes=names,
        mesh=mesh,
        region=region,
    )
    bones = np.zeros((len(BONES), 3), dtype=np.float64)
    dump.add("walk:1", bones, positions)
    moved = positions.copy()
    near = (np.abs(positions[:, 1] - 1.0) < 0.05) & (positions[:, 0] > 0.09)
    # the patch faces +x: out to the hoodie, then past it by the poke
    moved[(mesh == 1) & near, 0] += 0.0025 + poke_m
    if uncut_poke_m is not None:
        moved[(mesh == 4) & near, 0] += 0.0025 + uncut_poke_m
    dump.add("walk:2", bones, moved)
    return dump.write(path)


_MANIFEST = OutfitManifest(
    ("arm",), {"part_tee": PartSpec(3.0, ("arm",), "top"), "part_hoodie": PartSpec(4.0, ("arm",), "outer")}
)


def test_w4w_names_the_frame_where_a_layer_pokes_3_mm_and_passes_at_1_mm(tmp_path: Path) -> None:
    tolerances = Tolerances.load()
    result = gate_w4w(read(_dump(tmp_path / "capture.mwcf", 0.003)), _MANIFEST, tolerances)
    assert result.gate is GateId.W4W
    assert result.verdict is Verdict.FAIL
    assert result.reasons and all(r.endswith("walk frame 2") for r in result.reasons)
    assert any("part_tee pokes" in r and "through part_hoodie in arm" in r for r in result.reasons)
    assert "1 of 2 frames pass" in result.looked_at
    passing = gate_w4w(read(_dump(tmp_path / "capture.mwcf", 0.001)), _MANIFEST, tolerances)
    assert passing.verdict is Verdict.PASS, passing.reasons


def test_w4w_judges_the_uncut_shells_when_the_dump_carries_them(tmp_path: Path) -> None:
    """S36-P3b (Q-S36-P3-4): the drawn tee is cut where the hoodie covers it; its uncut copy `PartCheck_tee`
    keeps the poke, and W-4w judges the copies."""
    drawn = read(_dump(tmp_path / "drawn.mwcf", 0.001, uncut_poke_m=0.003))
    result = gate_w4w(drawn, _MANIFEST, Tolerances.load())
    assert result.verdict is Verdict.FAIL
    assert any("part_tee pokes" in r and r.endswith("walk frame 2") for r in result.reasons)
