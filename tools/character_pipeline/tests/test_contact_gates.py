"""C-1 and C-2 on the committed protagonist and on generated variants with planted depths (S36-P1 commit
3)."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pytest
from conftest import REPO_ROOT
from contact_shapes import Node, box, open_box, write_glb

from mwcast.contact_gates import OutfitManifest, Skinning, gate_c1, gate_c2, limb_in_trunk
from mwcast.gates import GateId, Verdict
from mwcast.glb import Glb, node_meshes, primitive_attribute, primitive_positions, read_glb, triangles
from mwcast.tolerances import Tolerances

PROTAGONIST = REPO_ROOT / "clients/3d-spike/assets/characters/meshy_d/meshy_d.glb"


def test_the_protagonist_passes_c1_at_rest_with_its_margins_printed() -> None:
    result = gate_c1(read_glb(PROTAGONIST), Tolerances.load())
    assert result.gate is GateId.C1
    assert result.verdict is Verdict.PASS, result.reasons
    assert any("in trunk:" in line and "margin" in line for line in result.looked_at)


# --- C-1, generated: an open trunk shell and a hand box sunk into its side ----------------------------------

_JOINTS = {
    "Hips": (0.0, 1.0, 0.0),
    "Spine": (0.0, 1.1, 0.0),
    "Neck": (0.0, 1.5, 0.0),
    **{
        side + name: (sign * x, y, 0.0)
        for side, sign in (("Left", 1.0), ("Right", -1.0))
        for name, x, y in (
            ("UpperArm", 0.2, 1.45),
            ("LowerArm", 0.45, 1.3),
            ("Hand", 0.25, 1.3),
            ("UpperLeg", 0.1, 0.95),
            ("LowerLeg", 0.1, 0.5),
            ("Foot", 0.1, 0.08),
        )
    },
}


def _sunk_hand(depth_m: float) -> tuple[Skinning, np.ndarray]:
    # the trunk: open at the neck and the waist, as a garment-less fused torso patch would be
    trunk_v, trunk_f = open_box((-0.15, 1.0, -0.1), (0.15, 1.5, 0.1), drop=(2, 3))
    face = 0.15 - depth_m
    hand_v, hand_f = box((face, 1.28, -0.02), (face + 0.04, 1.32, 0.02))
    positions = np.vstack([trunk_v, hand_v])
    faces = np.vstack([trunk_f, hand_f + trunk_v.shape[0]])
    weights = [{"mixamorig:Spine": 1.0}] * trunk_v.shape[0] + [{"mixamorig:LeftHand": 1.0}] * hand_v.shape[0]
    return Skinning.of(positions.tolist(), weights), faces


def test_c1_fails_a_hand_sunk_8_mm_into_the_trunk_and_passes_one_sunk_4_mm() -> None:
    tolerances = Tolerances.load()
    skin, faces = _sunk_hand(0.008)
    deep = limb_in_trunk(skin, faces.tolist(), _JOINTS, tolerances)
    assert deep.verdict is Verdict.FAIL
    assert deep.reasons == ("Left hand is 8.0 mm inside the trunk (4 vertices over 5 mm)",)
    skin, faces = _sunk_hand(0.004)
    shallow = limb_in_trunk(skin, faces.tolist(), _JOINTS, tolerances)
    assert shallow.verdict is Verdict.PASS, shallow.reasons
    assert any(
        line.startswith("Left hand in trunk: 8 vertices, 4 inside, worst 4.0 mm")
        for line in shallow.looked_at
    )


# --- C-2, generated: a two-region body and three nested shells ----------------------------------------------

_REGIONS = ["torso", "hips"]


def _uv2(count: int, region: int) -> np.ndarray:
    # u = region + 0.5 (parts: -0.5); v = -0.5 is the assembly's "under an outer layer"
    return np.tile([region + 0.5, -0.5], (count, 1))


def _outfit(tmp_path: Path, mid_top_m: float) -> tuple[Path, Path]:
    torso_v, torso_f = box((-0.15, 1.1, -0.1), (0.15, 1.5, 0.1))
    hips_v, hips_f = box((-0.15, 0.9, -0.1), (0.15, 1.1, 0.1))
    body_v = np.vstack([torso_v, hips_v])
    body_f = np.vstack([torso_f, hips_f + 8])
    body_uv = np.vstack([_uv2(8, 0), _uv2(8, 1)])
    tee_v, tee_f = box((-0.155, 1.095, -0.105), (0.155, 1.505, 0.105))
    mid_v, mid_f = box((-0.16, 0.89, -0.11), (0.16, mid_top_m, 0.11))
    outer_v, outer_f = box((-0.165, 0.885, -0.115), (0.165, 1.515, 0.115))
    nodes = {
        "Body": Node(body_v, body_f, body_uv),
        "part_tee": Node(tee_v, tee_f, _uv2(8, -1)),
        "part_mid": Node(mid_v, mid_f, _uv2(8, -1)),
        "part_outer": Node(outer_v, outer_f, _uv2(8, -1)),
    }
    manifest = {
        "name": "nested",
        "regions": _REGIONS,
        "parts": {
            "part_tee": {"layer": 1, "covers": ["torso"], "class": "top"},
            "part_mid": {"layer": 2, "covers": ["torso", "hips"], "class": "top"},
            "part_outer": {"layer": 3, "covers": ["torso", "hips"], "class": "outer"},
        },
    }
    manifest_path = tmp_path / "outfit_manifest.json"
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
    return write_glb(tmp_path / "outfit_nested.glb", nodes), manifest_path


def test_c2_fails_a_middle_layer_pushed_3_mm_through_the_outer_one_and_passes_at_1_mm(tmp_path: Path) -> None:
    tolerances = Tolerances.load()
    glb_path, manifest_path = _outfit(tmp_path, 1.515 + 0.003)
    result = gate_c2(read_glb(glb_path), OutfitManifest.load(manifest_path), tolerances)
    assert result.verdict is Verdict.FAIL
    assert result.reasons == ("part_mid pokes 3.0 mm through part_outer in torso (garment.poke.layer 2 mm)",)
    glb_path, manifest_path = _outfit(tmp_path, 1.515 + 0.001)
    result = gate_c2(read_glb(glb_path), OutfitManifest.load(manifest_path), tolerances)
    assert result.verdict is Verdict.PASS, result.reasons
    # every pair in layer order was tested: the body in each part, and each lower layer in each higher one
    assert result.looked_at[1] == "shells: drawn (the outfit has no uncut copies)"
    pairs = [line.split(":")[0] for line in result.looked_at[2:]]
    assert pairs == [
        "Body inside part_tee",
        "Body inside part_mid",
        "part_tee inside part_mid",
        "Body inside part_outer",
        "part_tee inside part_outer",
        "part_mid inside part_outer",
    ]


def test_c2_judges_the_uncut_shells_when_the_outfit_carries_them(tmp_path: Path) -> None:
    """S36-P3b (Q-S36-P3-4): the drawn middle layer is cut where the outer one covers it, so its poke is not
    in the drawn GLB; `PartCheck_mid` keeps it, and C-2 reads that copy."""
    tolerances = Tolerances.load()
    glb_path, manifest_path = _outfit(tmp_path, 1.515 + 0.001)  # the drawn shells: within the bound
    (tmp_path / "poked").mkdir()
    poked, _ = _outfit(tmp_path / "poked", 1.515 + 0.003)  # the same outfit before the cut: 3 mm through
    drawn = read_glb(glb_path)
    nodes = {
        name: Node(*_positions_and_faces(drawn, name), _uv_of(drawn, name))
        for name in ("Body", "part_tee", "part_mid", "part_outer")
    }
    copies = read_glb(poked)
    for name, check in (("Body", "BodyCheck"), ("part_tee", "PartCheck_tee"), ("part_mid", "PartCheck_mid"),
                        ("part_outer", "PartCheck_outer")):  # fmt: skip
        nodes[check] = Node(*_positions_and_faces(copies, name), _uv_of(copies, name))
    both = write_glb(tmp_path / "outfit_uncut.glb", nodes)
    manifest = OutfitManifest.load(manifest_path)
    assert gate_c2(drawn, manifest, tolerances).verdict is Verdict.PASS
    result = gate_c2(read_glb(both), manifest, tolerances)
    assert result.looked_at[1] == "shells: uncut (BodyCheck, PartCheck_<id>)"
    assert result.reasons == ("part_mid pokes 3.0 mm through part_outer in torso (garment.poke.layer 2 mm)",)


def _positions_and_faces(glb: Glb, name: str) -> tuple[np.ndarray, np.ndarray]:
    mesh = node_meshes(glb)[name]
    return np.asarray(primitive_positions(glb, mesh, 0)), np.asarray(triangles(glb, mesh, 0))


def _uv_of(glb: Glb, name: str) -> np.ndarray:
    return np.asarray(primitive_attribute(glb, node_meshes(glb)[name], 0, "TEXCOORD_1"))


def test_c2_cannot_look_when_the_manifest_names_a_part_the_outfit_lacks(tmp_path: Path) -> None:
    glb_path, manifest_path = _outfit(tmp_path, 1.5)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["parts"]["part_scarf"] = {"layer": 4, "covers": ["torso"], "class": "outer"}
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
    result = gate_c2(read_glb(glb_path), OutfitManifest.load(manifest_path), Tolerances.load())
    assert result.verdict is Verdict.FAIL
    assert result.reasons == ("cannot look: the outfit has no mesh node ['part_scarf']",)


@pytest.mark.parametrize("missing", ["Body"])
def test_c2_cannot_look_without_the_body_region_channel(tmp_path: Path, missing: str) -> None:
    body_v, body_f = box((0.0, 0.0, 0.0), (1.0, 1.0, 1.0))
    glb_path = write_glb(
        tmp_path / "plain.glb", {missing: Node(body_v, body_f), "part_a": Node(body_v, body_f)}
    )
    manifest = OutfitManifest.load(
        _write(tmp_path, {"regions": ["torso"], "parts": {"part_a": {"layer": 1, "covers": ["torso"]}}})
    )
    result = gate_c2(read_glb(glb_path), manifest, Tolerances.load())
    assert result.verdict is Verdict.FAIL
    assert "has no TEXCOORD_1" in result.reasons[0]


def _write(tmp_path: Path, document: object) -> Path:
    path = tmp_path / "manifest.json"
    path.write_text(json.dumps(document), encoding="utf-8")
    return path
