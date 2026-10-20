"""Gates G-1…G-6 and G-14: one PASS and one FAIL on fixtures (X-5, X-6, X-7 and the cases of design §9 B3).

The frozen evidence of the accepted protagonist mesh (A-B1-14, F-B1-8) is pinned here as well."""

from __future__ import annotations

import inspect
import json
import math
from dataclasses import replace
from pathlib import Path

import pytest
from gltf_fixture import build_glb
from test_records import vera_fit_object

from mwcast import gates
from mwcast.gates import GateId, GateResult, Verdict
from mwcast.glb import Glb, joint_parents, read_glb, skin_joint_names
from mwcast.records import FitParams, parse_fit
from mwcast.standard import REFERENCE_SKELETON

REPO = Path(__file__).resolve().parents[3]
MESHY_D = REPO / "clients/3d-spike/assets/characters/meshy_d/meshy_d.glb"
VERA_STATURE = 1.767
VERA_EYE_Z_RAW = 0.779


def _vera() -> FitParams:
    return parse_fit(vera_fit_object(), Path("fit.yaml"))


def _png(width: int, height: int, depth: int = 8, colour: int = 2) -> bytes:
    """A PNG signature and IHDR chunk: all `png_size` reads."""
    ihdr = width.to_bytes(4, "big") + height.to_bytes(4, "big") + bytes([depth, colour, 0, 0, 0])
    return b"\x89PNG\r\n\x1a\n" + (13).to_bytes(4, "big") + b"IHDR" + ihdr + b"\x00\x00\x00\x00"


def _skeleton(
    renames: dict[str, str] | None = None,
    reparent: dict[str, str | None] | None = None,
) -> list[tuple[str, str | None]]:
    """The 52 joints of the reference skeleton, in skin order, with optional renames and re-parenting."""
    reference = read_glb(REPO / REFERENCE_SKELETON)
    names = skin_joint_names(reference)
    parents = joint_parents(reference)
    renamed = renames or {}
    out: list[tuple[str, str | None]] = []
    for name in names:
        parent = parents[name]
        if reparent and name in reparent:
            parent = reparent[name]
        new_parent = renamed.get(parent, parent) if parent is not None else None
        out.append((renamed.get(name, name), new_parent))
    return out


def _verdict(result: GateResult) -> Verdict:
    return result.verdict


def _meshy() -> Glb:
    return read_glb(MESHY_D)


def test_x5_a_100001_triangle_mesh_fails_g1_and_names_the_count() -> None:
    result = gates.gate_g1(read_glb_bytes(build_glb(triangles=100_001)))

    assert result.gate is GateId.G1
    assert _verdict(result) is Verdict.FAIL
    assert any("100001" in reason for reason in result.reasons)


def test_x5_a_100000_triangle_mesh_passes_g1() -> None:
    result = gates.gate_g1(read_glb_bytes(build_glb(triangles=100_000)))

    assert _verdict(result) is Verdict.PASS


def test_x6_a_second_material_fails_g2_naming_both() -> None:
    result = gates.gate_g2(read_glb_bytes(build_glb(materials=("MW_A", "MW_B"), image_png=_png(2048, 2048))))

    assert _verdict(result) is Verdict.FAIL
    joined = " ".join(result.reasons)
    assert "MW_A" in joined
    assert "MW_B" in joined


def test_g2_one_material_named_mw_cast_with_a_base_colour_passes() -> None:
    result = gates.gate_g2(read_glb_bytes(build_glb(materials=("MW_Cast",), image_png=_png(2048, 2048))))

    assert _verdict(result) is Verdict.PASS


def test_g2_the_accepted_mesh_fails_naming_mw_meshy() -> None:
    result = gates.gate_g2(_meshy())

    assert _verdict(result) is Verdict.FAIL
    assert any("MW_Meshy" in reason for reason in result.reasons)


def test_g3_a_1024_square_base_colour_fails_and_2048_passes() -> None:
    small = gates.gate_g3(read_glb_bytes(build_glb(image_png=_png(1024, 1024))))
    full = gates.gate_g3(read_glb_bytes(build_glb(image_png=_png(2048, 2048))))

    assert _verdict(small) is Verdict.FAIL
    assert _verdict(full) is Verdict.PASS


def test_g4_passes_when_the_sole_and_the_eye_line_are_on_the_bounds(tmp_path: Path) -> None:
    target = VERA_STATURE * 0.9366
    run = _gameready_json(tmp_path, scale=target / VERA_EYE_Z_RAW, sole_raw=0.0)
    glb = read_glb_bytes(build_glb(base_y=-0.0177))

    result = gates.gate_g4(glb, _vera(), run)

    assert _verdict(result) is Verdict.PASS


def test_g4_fails_when_the_soles_are_6_mm_low(tmp_path: Path) -> None:
    target = VERA_STATURE * 0.9366
    run = _gameready_json(tmp_path, scale=target / VERA_EYE_Z_RAW, sole_raw=0.0)
    glb = read_glb_bytes(build_glb(base_y=-0.0237))

    result = gates.gate_g4(glb, _vera(), run)

    assert _verdict(result) is Verdict.FAIL
    assert any("soles at y" in reason for reason in result.reasons)


def test_g4_fails_when_the_eye_line_is_2_percent_high(tmp_path: Path) -> None:
    target = VERA_STATURE * 0.9366
    run = _gameready_json(tmp_path, scale=target / VERA_EYE_Z_RAW * 1.02, sole_raw=0.0)
    glb = read_glb_bytes(build_glb(base_y=-0.0177))

    result = gates.gate_g4(glb, _vera(), run)

    assert _verdict(result) is Verdict.FAIL
    assert any("sole-to-eye" in reason for reason in result.reasons)


def test_g4_without_a_run_directory_cannot_pass(tmp_path: Path) -> None:
    result = gates.gate_g4(read_glb_bytes(build_glb(base_y=-0.0177)), _vera(), None)

    assert _verdict(result) is Verdict.FAIL
    assert any("needs a run directory" in reason for reason in result.reasons)


def test_g5_the_accepted_mesh_passes_on_names_and_parents() -> None:
    result = gates.gate_g5(_meshy(), REPO, None)

    assert _verdict(result) is Verdict.PASS
    assert any("rolls: not checked" in line for line in result.looked_at)


def test_x7_a_renamed_bone_fails_g5_naming_missing_and_unexpected() -> None:
    renamed = {"mixamorig:LeftForeArm": "mixamorig:LeftForearm"}
    result = gates.gate_g5(read_glb_bytes(build_glb(joints=_skeleton(renames=renamed))), REPO, None)

    assert _verdict(result) is Verdict.FAIL
    joined = " ".join(result.reasons)
    assert "mixamorig:LeftForeArm" in joined
    assert "mixamorig:LeftForearm" in joined


def test_g5_a_re_parented_bone_fails_even_with_the_same_names() -> None:
    reparent: dict[str, str | None] = {"mixamorig:LeftHand": "mixamorig:Spine2"}
    result = gates.gate_g5(read_glb_bytes(build_glb(joints=_skeleton(reparent=reparent))), REPO, None)

    assert _verdict(result) is Verdict.FAIL
    assert any("parent differs" in reason and "LeftHand" in reason for reason in result.reasons)


def test_g6_the_accepted_mesh_passes_the_limb_ratios_at_vera_stature() -> None:
    result = gates.gate_g6(_meshy(), _vera())

    assert not [r for r in result.reasons if "ratio" in r]


def test_g6_a_stature_25_percent_off_fails_the_limb_ratios() -> None:
    fit = replace(_vera(), stature_m=2.5)

    result = gates.gate_g6(_meshy(), fit)

    assert _verdict(result) is Verdict.FAIL
    assert any("ratio" in reason for reason in result.reasons)


def test_g6_a_hips_joint_0_3_m_behind_the_body_fails_and_prints_the_joint() -> None:
    glb = read_glb_bytes(build_glb(joints=_skeleton(), joint_positions={"mixamorig:Hips": (0.0, 0.0, 0.3)}))

    result = gates.gate_g6(glb, _vera())

    assert _verdict(result) is Verdict.FAIL
    assert any("mixamorig:Hips" in reason for reason in result.reasons)


def _fused_shell(y: float) -> list[tuple[float, float, float]]:
    """A body fused with a backpack, as Meshy makes it (F-B1-B3-1): one closed elliptical section at height
    `y`, centre z -0.13, half-width 0.17 m, half-depth 0.205 m (front z +0.075, the pack's back z -0.335),
    sampled every 30 degrees like a decimated mesh. There is no inner back surface between pack and chest."""
    return [
        (0.17 * math.sin(math.radians(a)), y, -0.13 + 0.205 * math.cos(math.radians(a)))
        for a in range(0, 360, 30)
    ]


def _clipped_extent_margin(
    points: list[tuple[float, float, float]], joint: tuple[float, float, float]
) -> float:
    """The B3 rule R-B1-G6 replaced: the x/z extent of the slab's vertices within 0.15 m horizontally."""
    jx, jy, jz = joint
    near = [(x, z) for x, y, z in points if abs(y - jy) <= 0.010 and math.hypot(x - jx, z - jz) <= 0.15]
    return min(
        jx - min(x for x, _ in near),
        max(x for x, _ in near) - jx,
        jz - min(z for _, z in near),
        max(z for _, z in near) - jz,
    )


def test_g6_a_spine_inside_a_fused_body_and_pack_shell_passes_where_the_clipped_rule_failed() -> None:
    # Vera's Spine: 126 mm behind the front of a 411 mm section; nothing behind it within 150 mm.
    shell = _fused_shell(1.09)
    spine = (0.0, 1.09, -0.053)

    assert _clipped_extent_margin(shell, spine) < 0.0
    span = gates.depth_span(shell, spine)
    assert span is not None
    assert span.back_z == pytest.approx(-0.335)
    assert span.front_z == pytest.approx(0.075)
    assert span.margin(spine[2]) == pytest.approx(0.128)


def test_g6_a_joint_in_front_of_the_shell_or_beside_it_is_outside_the_body() -> None:
    shell = _fused_shell(1.09)

    in_front = gates.depth_span(shell, (0.0, 1.09, 0.10))
    assert in_front is not None and in_front.margin(0.10) == pytest.approx(-0.025)
    behind = gates.depth_span(shell, (0.0, 1.09, -0.36))
    assert behind is not None and behind.margin(-0.36) == pytest.approx(-0.025)
    assert gates.depth_span(shell, (0.25, 1.09, -0.13)) is None
    assert gates.depth_span(shell, (0.0, 1.20, -0.13)) is None


def test_g14_the_accepted_mesh_fails_naming_exactly_the_three_missing_keys() -> None:
    result = gates.gate_g14(_meshy(), REPO)

    assert result.reasons == ("missing: references, reference_generator, pipeline",)


def test_g14_a_key_shaped_string_in_the_provenance_fails() -> None:
    extras: dict[str, object] = {
        "mineworld_provenance": {
            "source_type": "generated",
            "ai_generated": True,
            "generator": "msy_" + "A" * 24,
            "meshy_task": "01a1155c-dafe-70bc-9279-9cef9a35856b",
            "references": [],
            "reference_generator": "x",
            "postprocess": "x",
            "pipeline": {"version": "mwcast-1", "fit_sha256": "0" * 64, "blender": "Blender 5.2.2 LTS"},
        }
    }
    result = gates.gate_g14(read_glb_bytes(build_glb(extras=extras)), REPO)

    assert _verdict(result) is Verdict.FAIL
    assert any("key pattern" in reason for reason in result.reasons)


def test_g14_a_missing_meshy_task_fails() -> None:
    extras: dict[str, object] = {
        "mineworld_provenance": {
            "source_type": "generated",
            "ai_generated": True,
            "generator": "Meshy",
            "references": [],
            "reference_generator": "x",
            "postprocess": "x",
            "pipeline": {"version": "mwcast-1", "fit_sha256": "0" * 64, "blender": "Blender 5.2.2 LTS"},
        }
    }
    result = gates.gate_g14(read_glb_bytes(build_glb(extras=extras)), REPO)

    assert _verdict(result) is Verdict.FAIL
    assert "missing: meshy_task" in result.reasons


def test_the_accepted_mesh_counts_are_the_frozen_evidence() -> None:
    result = gates.gate_g1(_meshy())

    assert _verdict(result) is Verdict.PASS
    assert "triangles 95000" in result.looked_at


def test_no_gate_takes_a_bound_as_an_argument() -> None:
    # X-9, the structural half: a gate's parameters are its inputs, never a bound (ARC-23 rule 2).
    forbidden = ("max_", "tol", "bound", "limit")
    for name, function in inspect.getmembers(gates, inspect.isfunction):
        if not name.startswith("gate_g"):
            continue
        for parameter in inspect.signature(function).parameters:
            assert not parameter.startswith(forbidden), f"{name} takes a bound-like parameter {parameter}"


def read_glb_bytes(data: bytes) -> Glb:
    """Write fixture bytes to a file and read them back through the reader."""
    import tempfile

    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "fixture.glb"
        path.write_bytes(data)
        return read_glb(path)


def _gameready_json(folder: Path, *, scale: float, sole_raw: float) -> Path:
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "gameready.json").write_text(
        json.dumps({"scale": scale, "sole_raw": sole_raw}), encoding="utf-8"
    )
    return folder


def test_g6_a_joint_behind_sparse_back_vertices_is_inside_by_the_triangles() -> None:
    # S26-c, F-S26c-1: a decimated fused body has big flat triangles on the back, so the vertex window at the
    # Hips (Alice r1) held only the apron's front vertices. The depth line still crosses the back triangle.
    points = [
        (-0.02, 0.94, 0.10),
        (0.02, 0.94, 0.10),
        (0.0, 0.945, 0.098),  # dense front (apron tie)
        (-0.30, 0.70, -0.14),
        (0.30, 0.70, -0.14),
        (0.0, 1.20, -0.14),  # one big back triangle
    ]
    tris = [(0, 1, 2), (3, 4, 5)]
    hips = (0.0, 0.94, 0.004)

    vertex_only = gates.depth_span(points, hips)
    assert vertex_only is not None and vertex_only.margin(hips[2]) < 0.0
    span = gates.depth_span(points, hips, tris)
    assert span is not None
    assert span.back_z == pytest.approx(-0.14)
    assert span.margin(hips[2]) == pytest.approx(0.096)


def test_g6_a_joint_behind_the_back_triangle_stays_outside() -> None:
    points = [
        (-0.02, 0.94, 0.10),
        (0.02, 0.94, 0.10),
        (0.0, 0.945, 0.098),
        (-0.30, 0.70, -0.14),
        (0.30, 0.70, -0.14),
        (0.0, 1.20, -0.14),
    ]
    span = gates.depth_span(points, (0.0, 0.94, -0.20), [(0, 1, 2), (3, 4, 5)])
    assert span is not None and span.margin(-0.20) == pytest.approx(-0.06)
