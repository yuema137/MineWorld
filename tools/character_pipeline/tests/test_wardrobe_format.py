"""The runtime garment format (§4.4): round-trips, the `.mhclo` formula on a hand-made fixture, W-1, M-p-3."""

from __future__ import annotations

import struct
from pathlib import Path

import pytest

from mwcast.wardrobe import fixtures
from mwcast.wardrobe import garment_format as gf

# A four-vertex template (metres) and a `.mhclo` of three garment vertices, computed by hand below.
TEMPLATE: list[gf.Vec3] = [(0.0, 0.0, 0.0), (0.2, 0.0, 0.0), (0.0, 0.4, 0.0), (0.0, 0.0, 0.5)]
MHCLO = """# Exported from MakeClothes (TM)
# license CC0
basemesh hm08
name fixture
x_scale 0 1 2.0
y_scale 0 2 4.0
z_scale 0 3 5.0
z_depth 50
verts 0
 0 1 2 0.5 0.25 0.25 1.0 0.0 0.0
 1 2 3 0.2 0.3 0.5 0.0 -2.0 1.0
 3
delete_verts
 2 - 3
"""


def test_the_mhclo_formula_matches_the_hand_computed_fixture() -> None:
    mhclo = gf.parse_mhclo(MHCLO, "fixture.mhclo")
    fitted = gf.fit(mhclo.binding, TEMPLATE)

    # scales: x |0.2 - 0| / 2 = 0.1, y |0.4 - 0| / 4 = 0.1, z |0.5 - 0| / 5 = 0.1 (decimetres -> metres)
    # v0: 0.5*(0,0,0) + 0.25*(0.2,0,0) + 0.25*(0,0.4,0) + (1.0*0.1, 0, 0) = (0.15, 0.1, 0)
    # v1: 0.2*(0.2,0,0) + 0.3*(0,0.4,0) + 0.5*(0,0,0.5) + (0, -0.2, 0.1) = (0.04, -0.08, 0.35)
    # v2: rigid on vertex 3 = (0, 0, 0.5)
    expected = [(0.15, 0.1, 0.0), (0.04, -0.08, 0.35), (0.0, 0.0, 0.5)]
    assert gf.max_distance(fitted, expected) < 1e-9
    assert mhclo.delete_verts == (2, 3)


def test_the_scale_follows_the_template_so_a_wider_body_widens_the_offset() -> None:
    mhclo = gf.parse_mhclo(MHCLO, "fixture.mhclo")
    wide = [(0.0, 0.0, 0.0), (0.4, 0.0, 0.0), (0.0, 0.4, 0.0), (0.0, 0.0, 0.5)]

    assert gf.fit(mhclo.binding, wide)[0] == pytest.approx((0.1 + 0.2, 0.1, 0.0))


def test_every_file_round_trips(tmp_path: Path) -> None:
    binding = gf.parse_mhclo(MHCLO, "fixture.mhclo").binding
    weights = gf.TemplateWeights(
        joints=("mixamorig:Hips", "mixamorig:Spine"),
        influences=tuple(((0, 0.75), (1, 0.25), (0, 0.0), (0, 0.0)) for _ in TEMPLATE),
    )
    corr = [gf.Corr(tri=(0, 1, 2), b0=0.25, b1=0.5)]

    gf.write_binding(tmp_path / "binding.bin", binding)
    gf.write_hides(tmp_path / "hides.bin", [3, 1, 3])
    gf.write_corr(tmp_path / "corr.bin", corr)
    gf.write_template(tmp_path / "template.bin", TEMPLATE)
    gf.write_template_weights(tmp_path / "w.bin", weights)

    assert gf.read_binding(tmp_path / "binding.bin").records[0] == binding.records[0]
    assert gf.read_hides(tmp_path / "hides.bin") == (1, 3)
    assert gf.read_corr(tmp_path / "corr.bin") == tuple(corr)
    assert gf.read_template(tmp_path / "template.bin")[1] == pytest.approx((0.2, 0.0, 0.0))
    assert gf.read_template_weights(tmp_path / "w.bin") == weights


def test_w1_passes_the_fixture_and_m_p_3_a_weight_off_by_1e_3_fails(tmp_path: Path) -> None:
    binding = gf.parse_mhclo(MHCLO, "fixture.mhclo").binding
    assert gf.check_binding(binding, len(TEMPLATE)) == []

    path = tmp_path / "binding.bin"
    gf.write_binding(path, binding)
    data = bytearray(path.read_bytes())
    first_weight = 48 + 12  # header 12 + scale refs 36, then three u32 ids
    (w0,) = struct.unpack_from("<f", data, first_weight)
    struct.pack_into("<f", data, first_weight, w0 + 1e-3)
    path.write_bytes(bytes(data))

    failures = gf.check_binding(gf.read_binding(path), len(TEMPLATE))
    assert failures == ["vertex 0: weights sum to 1.001000"]


def test_w1_refuses_an_out_of_range_triangle_and_a_bad_scale_reference() -> None:
    record = gf.BindRecord(verts=(0, 1, 9), weights=(0.2, 0.3, 0.5), offset=(0.0, 0.0, 0.0))
    binding = gf.Binding(
        scale=(gf.ScaleRef(0, 0, 1.0), gf.ScaleRef(0, 2, 1.0), gf.ScaleRef(0, 3, 0.0)), records=(record,)
    )

    failures = gf.check_binding(binding, len(TEMPLATE))

    assert any(f.startswith("scale x") for f in failures)
    assert any(f.startswith("scale z") for f in failures)
    assert "vertex 0: template triangle (0, 1, 9) invalid" in failures


def test_a_truncated_file_is_refused(tmp_path: Path) -> None:
    gf.write_template(tmp_path / "t.bin", TEMPLATE)
    (tmp_path / "t.bin").write_bytes((tmp_path / "t.bin").read_bytes()[:-4])

    with pytest.raises(gf.FormatError, match="bytes for 4 positions"):
        gf.read_template(tmp_path / "t.bin")


def test_a_binding_follows_the_exported_vertex_order_and_seam_copies() -> None:
    """A UV seam makes the exporter repeat a vertex; each exported vertex takes its source's record."""
    binding = gf.parse_mhclo(MHCLO, "fixture.mhclo").binding
    source = gf.fit(binding, TEMPLATE)
    exported = [source[2], source[0], source[0], source[1]]

    expanded = fixtures.expand_binding(binding, source, exported)

    assert [r.verts for r in expanded.records] == [(3, 3, 3), (0, 1, 2), (0, 1, 2), (1, 2, 3)]
    with pytest.raises(fixtures.FixtureError, match="matches no source vertex"):
        fixtures.expand_binding(binding, source, [(9.0, 9.0, 9.0)])


def test_weights_are_remapped_to_the_body_skin_order() -> None:
    weights = gf.TemplateWeights(joints=("a", "b"), influences=(((1, 0.6), (0, 0.4), (0, 0.0), (0, 0.0)),))

    remapped = gf.remap_joints(weights, ["x", "b", "a"])

    assert remapped.joints == ("x", "b", "a")
    assert remapped.influences[0][:2] == ((1, 0.6), (2, 0.4))
    with pytest.raises(gf.FormatError, match="no joints"):
        gf.remap_joints(weights, ["x"])
