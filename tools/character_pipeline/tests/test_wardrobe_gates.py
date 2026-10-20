"""The static per-outfit gates on small hand-built outfit GLBs: G-1, G-2, W-2, G-14, V-1 and the planted
mutations M-p-4 (a part over its budget), M-p-5 (provenance without a Meshy task) and M-p-6 (a stock garment
drawn)."""

from __future__ import annotations

import json
import struct
from collections.abc import Mapping, Sequence
from pathlib import Path

from mwcast.gates import Verdict
from mwcast.glb import read_glb
from mwcast.wardrobe import wgates

TASK = "01a1155c-dafe-70bc-9279-9cef9a35856b"


def _outfit(tmp: Path, nodes: Sequence[tuple[str, int, str]], provenance: Mapping[str, object]) -> Path:
    """A GLB with one mesh node per (name, triangles, material); each mesh one indexed primitive."""
    binary = bytearray()
    views: list[dict[str, object]] = []
    accessors: list[dict[str, object]] = []
    meshes: list[dict[str, object]] = []
    gl_nodes: list[dict[str, object]] = []
    materials: list[dict[str, object]] = []
    for name, tris, material in nodes:
        positions = b"".join(struct.pack("<3f", float(k), 0.0, 0.0) for k in range(3 * tris))
        indices = struct.pack(f"<{3 * tris}I", *range(3 * tris))
        for payload, kind, comp, count in (
            (positions, "VEC3", 5126, 3 * tris),
            (indices, "SCALAR", 5125, 3 * tris),
        ):
            while len(binary) % 4:
                binary.append(0)
            views.append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(payload)})
            binary.extend(payload)
            accessors.append(
                {"bufferView": len(views) - 1, "componentType": comp, "count": count, "type": kind}
            )
        materials.append({"name": material})
        meshes.append(
            {
                "primitives": [
                    {
                        "attributes": {"POSITION": len(accessors) - 2},
                        "indices": len(accessors) - 1,
                        "material": len(materials) - 1,
                    }
                ]
            }
        )
        gl_nodes.append({"name": name, "mesh": len(meshes) - 1})
    document = {
        "asset": {"version": "2.0"},
        "scenes": [
            {"nodes": list(range(len(gl_nodes))), "extras": {"mineworld_provenance": json.dumps(provenance)}}
        ],
        "nodes": gl_nodes,
        "meshes": meshes,
        "materials": materials,
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": len(binary)}],
    }
    chunk = json.dumps(document).encode("utf-8")
    chunk += b" " * (-len(chunk) % 4)
    while len(binary) % 4:
        binary.append(0)
    data = struct.pack("<III", 0x46546C67, 2, 28 + len(chunk) + len(binary))
    data += (
        struct.pack("<II", len(chunk), 0x4E4F534A)
        + chunk
        + struct.pack("<II", len(binary), 0x004E4942)
        + binary
    )
    path = tmp / "outfit.glb"
    path.write_bytes(data)
    return path


PILOT_PART = {"source_type": "generated", "meshy_task": TASK, "pipeline": {"version": "mwcast-1"}}
NODES = [("Body", 50, "MW_Cast"), ("part_vera-tee", 20, "MW_Cast_vera-tee")]
CLASSES = {"vera-tee": "top"}


def test_a_clean_outfit_passes_g1_g2_w2_g14_and_v1(tmp_path: Path) -> None:
    glb = read_glb(_outfit(tmp_path, NODES, {"vera-tee": PILOT_PART}))

    for result in (
        wgates.gate_g1(glb),
        wgates.gate_g2(glb),
        wgates.gate_w2(glb, CLASSES),
        wgates.gate_g14(glb),
        wgates.gate_v1(glb),
    ):
        assert result.verdict is Verdict.PASS, result


def test_m_p_4_a_part_over_its_budget_fails_w2(tmp_path: Path) -> None:
    glb = read_glb(
        _outfit(
            tmp_path,
            [("Body", 50, "MW_Cast"), ("part_vera-tee", 20, "MW_Cast_vera-tee")],
            {"vera-tee": PILOT_PART},
        )
    )

    result = wgates.gate_w2(glb, {"vera-tee": "socks"})  # 20 triangles fit any real budget: lower it by class
    assert result.verdict is Verdict.PASS
    big = read_glb(
        _outfit(
            tmp_path,
            [("Body", 50, "MW_Cast"), ("part_vera-tee", 1_501, "MW_Cast_vera-tee")],
            {"vera-tee": PILOT_PART},
        )
    )
    failed = wgates.gate_w2(big, {"vera-tee": "socks"})
    assert failed.verdict is Verdict.FAIL and "1501 triangles > 1500" in failed.reasons[0]


def test_m_p_5_provenance_without_a_meshy_task_fails_v1(tmp_path: Path) -> None:
    glb = read_glb(_outfit(tmp_path, NODES, {"vera-tee": {"source_type": "generated", "pipeline": {}}}))

    assert wgates.gate_v1(glb).verdict is Verdict.FAIL
    assert wgates.gate_g14(glb).verdict is Verdict.PASS  # keys present; V-1 owns the Meshy-task rule


def test_m_p_6_a_stock_or_fixture_garment_fails_v1(tmp_path: Path) -> None:
    stock = {"source_type": "stock", "role": "fixture", "asset": "MakeHuman toigo_basic_tucked_t-shirt (CC0)"}
    glb = read_glb(_outfit(tmp_path, NODES, {"vera-tee": stock}))

    result = wgates.gate_v1(glb)
    assert result.verdict is Verdict.FAIL
    assert any("never drawn" in r for r in result.reasons) and any("stock" in r for r in result.reasons)


def test_a_missing_provenance_fails_g14_and_v1(tmp_path: Path) -> None:
    glb = read_glb(_outfit(tmp_path, NODES, {}))

    assert wgates.gate_g14(glb).verdict is Verdict.FAIL
    assert wgates.gate_v1(glb).verdict is Verdict.FAIL


def test_g2_refuses_a_part_named_hoodie_and_a_wrong_material(tmp_path: Path) -> None:
    """M-p-10's static half: a part named `Hoodie` is the name human.gd hides (F-S32p-4)."""
    glb = read_glb(_outfit(tmp_path, [("Body", 5, "MW_Cast"), ("Hoodie", 5, "MW_Hoodie")], {}))

    reasons = wgates.gate_g2(glb).reasons
    assert any("human.gd hides" in r for r in reasons)
    assert any("expected exactly [MW_Cast_body]" in r or "MW_Hoodie" in r for r in reasons)


def test_g1_counts_the_whole_outfit(tmp_path: Path) -> None:
    glb = read_glb(_outfit(tmp_path, [("Body", 60_000, "MW_Cast"), ("part_a", 40_001, "MW_Cast_a")], {}))

    result = wgates.gate_g1(glb)
    assert result.verdict is Verdict.FAIL and result.reasons == ("100001 triangles > 100000",)
