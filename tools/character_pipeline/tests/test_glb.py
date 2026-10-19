"""GLB decoding against synthetic files with hand-computed expectations, and the committed protagonist."""

from __future__ import annotations

from pathlib import Path

import pytest
from gltf_fixture import build_glb

from mwcast.glb import (
    GlbError,
    joint_parents,
    joint_world_positions,
    material_names,
    png_size,
    positions,
    read_glb,
    skin_joint_names,
    triangle_count,
    vertex_weights,
)

MESHY_D = Path(__file__).resolve().parents[3] / "clients/3d-spike/assets/characters/meshy_d/meshy_d.glb"


def _write(tmp_path: Path, data: bytes, name: str = "fixture.glb") -> Path:
    path = tmp_path / name
    path.write_bytes(data)
    return path


def test_fixture_decodes_to_the_counts_it_was_built_with(tmp_path: Path) -> None:
    joints = (("Hips", None), ("Spine", "Hips"), ("Head", "Spine"))
    glb = read_glb(_write(tmp_path, build_glb(triangles=4, materials=("MW_A",), joints=joints)))

    assert triangle_count(glb) == 4
    assert material_names(glb) == ["MW_A"]
    assert skin_joint_names(glb) == ["Hips", "Spine", "Head"]
    assert joint_parents(glb) == {"Hips": None, "Spine": "Hips", "Head": "Spine"}
    assert len(positions(glb)) == 12


def test_triangle_count_counts_every_material_primitive(tmp_path: Path) -> None:
    glb = read_glb(_write(tmp_path, build_glb(triangles=3, materials=("MW_A", "MW_B"))))

    assert triangle_count(glb) == 6
    assert material_names(glb) == ["MW_A", "MW_B"]


def test_inverse_bind_matrix_inverts_to_the_hand_computed_position(tmp_path: Path) -> None:
    # World bind transform M = T(1, 2, 3) · Rz(90°): rows [[0,-1,0,1],[1,0,0,2],[0,0,1,3],[0,0,0,1]].
    # Its inverse, the inverse bind matrix, is rows [[0,1,0,-2],[-1,0,0,1],[0,0,1,-3],[0,0,0,1]].
    # glTF stores it column-major, so the flat list is column 0, then column 1, and so on.
    inverse_bind = [0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -2.0, 1.0, -3.0, 1.0]
    glb = read_glb(_write(tmp_path, build_glb(ibm_overrides={"Hips": inverse_bind})))

    position = joint_world_positions(glb)["Hips"]

    assert position == pytest.approx((1.0, 2.0, 3.0), abs=1e-6)


def test_normalised_unsigned_short_weight_decodes_to_one(tmp_path: Path) -> None:
    glb = read_glb(_write(tmp_path, build_glb(weights_u16=True)))

    weights = vertex_weights(glb)

    assert weights[0] == pytest.approx({"Hips": 1.0})


def test_a_truncated_file_is_refused_naming_the_file(tmp_path: Path) -> None:
    data = build_glb()
    path = _write(tmp_path, data[:-40], name="cut.glb")

    with pytest.raises(GlbError) as caught:
        read_glb(path)

    assert str(path) in str(caught.value)
    assert "truncated" in str(caught.value)


def test_a_bad_magic_is_refused(tmp_path: Path) -> None:
    data = b"XXXX" + build_glb()[4:]

    with pytest.raises(GlbError, match="magic"):
        read_glb(_write(tmp_path, data))


def test_png_size_reads_the_ihdr_chunk() -> None:
    header = (
        b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\x0dIHDR" + (1024).to_bytes(4, "big") + (2048).to_bytes(4, "big")
    )
    header += bytes([8, 6])

    assert png_size(header) == (1024, 2048, 8, 6)


def test_committed_protagonist_mesh_decodes_to_the_frozen_counts() -> None:
    # Frozen evidence (design A-B1-14): meshy_d.glb is 95,000 triangles, 65,176 vertices, 52 joints, one
    # material named MW_Meshy. Pinned here from the audit, not from the implementation.
    glb = read_glb(MESHY_D)

    assert triangle_count(glb) == 95000
    assert len(positions(glb)) == 65176
    assert len(skin_joint_names(glb)) == 52
    assert material_names(glb) == ["MW_Meshy"]
