"""The winding-number push-out on shapes with known answers (S36-P3 commit 1). Every verdict is C-2's own
rule (`contact_gates.layer_order`) with the tolerance file's bounds: what judges outfits judges this."""

from __future__ import annotations

import math

import numpy as np
from contact_shapes import box, open_box
from numpy.typing import NDArray

from mwcast.contact import TriMesh
from mwcast.contact_gates import LayerArrays, OutfitManifest, PartSpec, layer_order
from mwcast.gates import GateResult, Verdict
from mwcast.tolerances import Tolerances
from mwcast.wardrobe.pushout import GAP_M, push_out

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]


def _cylinder(radius: float, half_height: float, rings: int, segments: int, turn: float = 0.0) -> TriMesh:
    """An open cylinder about the y axis (a sleeve: open at both ends), outward-facing; `turn` rotates its
    vertices by that share of one segment."""
    vertices = [
        (
            radius * math.cos(2 * math.pi * (s + turn) / segments),
            -half_height + 2 * half_height * r / rings,
            radius * math.sin(2 * math.pi * (s + turn) / segments),
        )
        for r in range(rings + 1)
        for s in range(segments)
    ]
    faces: list[tuple[int, int, int]] = []
    for r in range(rings):
        for s in range(segments):
            a, b = r * segments + s, r * segments + (s + 1) % segments
            c, d = a + segments, b + segments
            faces += [(a, c, b), (b, c, d)]
    return TriMesh(np.array(vertices, dtype=np.float64), np.array(faces, dtype=np.int64))


def _cone(radius: float, apex_y: float, segments: int) -> TriMesh:
    """A closed cone with its base disc at y = 0 and its apex (vertex 0) at y = `apex_y`, outward-facing."""
    ring = [
        (radius * math.cos(2 * math.pi * s / segments), 0.0, radius * math.sin(2 * math.pi * s / segments))
        for s in range(segments)
    ]
    vertices = [(0.0, apex_y, 0.0), (0.0, 0.0, 0.0), *ring]
    faces: list[tuple[int, int, int]] = []
    for s in range(segments):
        a, b = 2 + s, 2 + (s + 1) % segments
        faces += [(0, b, a), (1, a, b)]
    return TriMesh(np.array(vertices, dtype=np.float64), np.array(faces, dtype=np.int64))


def _c2(under: TriMesh, flagged: Ints, shell: Floats, shell_faces: Ints) -> GateResult:
    """C-2 on a body `under` (one region; the `flagged` vertices are under the part) and one covering part."""
    manifest = OutfitManifest(("arm",), {"part_outer": PartSpec(1.0, ("arm",), "top")})
    flags = np.zeros(under.vertices.shape[0], dtype=bool)
    flags[flagged] = True
    skin = LayerArrays(under.vertices, under.faces, np.zeros(under.vertices.shape[0], dtype=np.int64), flags)
    outer = LayerArrays(
        shell, shell_faces, np.full(shell.shape[0], -1, dtype=np.int64), np.zeros(shell.shape[0], dtype=bool)
    )
    return layer_order(manifest, Tolerances.load(), "Body", skin, {"part_outer": outer}, [])


def _radius(points: Floats) -> Floats:
    return np.hypot(points[:, 0], points[:, 2])


def test_a_sleeve_inside_the_arm_is_pushed_out_and_passes_c2() -> None:
    arm = _cylinder(0.100, 0.30, 24, 48)
    sleeve = _cylinder(0.098, 0.20, 16, 48, turn=0.5)  # shorter, so its ends lie well inside the arm's span
    covered = np.flatnonzero(np.abs(arm.vertices[:, 1]) < 0.19)  # the arm under the sleeve
    before = _c2(arm, covered, sleeve.vertices, sleeve.faces)
    assert before.verdict is Verdict.FAIL, before.reasons
    pushed, report = push_out(sleeve, arm, probe=covered)
    assert _radius(pushed).min() >= 0.1015
    assert report.passes[0].moved == sleeve.vertices.shape[0]
    assert report.unreached.size == 0
    assert report.still_inside == 0
    after = _c2(arm, covered, pushed, sleeve.faces)
    assert after.verdict is Verdict.PASS, after.reasons


def test_a_vertex_sunk_past_reach_is_reported_and_not_moved() -> None:
    arm = _cylinder(0.100, 0.30, 24, 48)
    sleeve = _cylinder(0.104, 0.20, 16, 48, turn=0.5)  # outside the arm by more than the gap: nothing to do
    vertices = sleeve.vertices.copy()
    dragged = 8 * 48 + 5  # a vertex at mid-height
    vertices[dragged, [0, 2]] *= 0.048 / 0.104  # 52 mm inside the arm: another limb, not this layer's skin
    sunk = TriMesh(vertices, sleeve.faces)
    pushed, report = push_out(sunk, arm)
    assert report.unreached.tolist() == [dragged]
    assert np.array_equal(pushed[dragged], vertices[dragged])
    assert all(p.unreached == 1 for p in report.passes)


def test_a_sharp_tip_between_outer_vertices_moves_the_outer_face() -> None:
    # the inner cone's tip pokes 3 mm through the top of a coarse open box; every box vertex lies outside the
    # cone and further than the gap from it, so rules 1-4 move nothing and only rule 5 can enclose the tip
    cone = _cone(0.05, 0.103, 32)
    shell_v, shell_f = open_box((-0.15, -0.05, -0.15), (0.15, 0.10, 0.15), drop=(2,))
    shell = TriMesh(shell_v, shell_f)
    every = np.arange(cone.vertices.shape[0], dtype=np.int64)
    assert _c2(cone, every, shell_v, shell_f).verdict is Verdict.FAIL
    pushed, report = push_out(shell, cone, probe=every)
    assert report.passes[0].moved == 0
    assert report.passes[0].poke_moved == 3
    assert report.still_poking == 0
    assert math.isclose(report.worst_mm, 3.0 + GAP_M * 1000, abs_tol=0.01)
    after = _c2(cone, every, pushed, shell_f)
    assert after.verdict is Verdict.PASS, after.reasons


def test_the_under_surface_never_moves() -> None:
    body_v, body_f = box((-0.1, -0.1, -0.1), (0.1, 0.1, 0.1))
    body = TriMesh(body_v, body_f)
    shell_v, shell_f = box((-0.099, -0.099, -0.099), (0.099, 0.099, 0.099))
    copy = body.vertices.copy()
    _pushed, report = push_out(TriMesh(shell_v, shell_f), body)
    assert np.array_equal(body.vertices, copy)
    assert report.still_inside == 0
