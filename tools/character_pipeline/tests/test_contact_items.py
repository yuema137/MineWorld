"""C-3 and C-4 over a generated prop library and the real one (S36-P1 commit 4)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from contact_shapes import Node, box, write_gltf

from mwcast.cli import main
from mwcast.contact_gates import KNOWN, load_mounts, run_item_gates
from mwcast.gates import Verdict
from mwcast.tolerances import Tolerances


def _prop(root: Path, slug: str, nodes: dict[str, Node]) -> None:
    (root / slug).mkdir(parents=True)
    write_gltf(root / slug / f"{slug}.gltf", nodes)


def _node(y: float = 0.0, x: float = 0.0) -> Node:
    vertices, faces = box((0.0, 0.0, 0.0), (0.5, 0.5, 0.5))
    return Node(vertices, faces, translation=(x, y, 0.0))


def _library(tmp_path: Path, items: dict[str, object]) -> Path:
    root = tmp_path / "models"
    _prop(root, "raised_6", {"block": _node(y=0.006)})
    _prop(root, "raised_2", {"block": _node(y=0.002)})
    _prop(root, "overlapping", {"left": _node(), "right": _node(x=0.496)})
    _prop(root, "resting", {"table": _node(), "cup": _node(y=0.5, x=0.1)})
    (root / "contact.json").write_text(json.dumps({"format": 1, "items": items}), encoding="utf-8")
    return root


_ITEMS: dict[str, object] = {
    "raised_6": {"mount": "floor"},
    "raised_2": {"mount": "floor"},
    "overlapping": {"mount": "surface"},
    "resting": {"mount": "floor", "resting": [["cup", "table"]]},
}


def test_the_generated_library_fails_exactly_where_planted(tmp_path: Path) -> None:
    results = run_item_gates(_library(tmp_path, _ITEMS), Tolerances.load())
    verdicts = {slug: [r.verdict for r in rs] for slug, rs in results.items()}
    assert verdicts == {
        "overlapping": [Verdict.PASS, Verdict.FAIL],
        "raised_2": [Verdict.PASS, Verdict.PASS],
        "raised_6": [Verdict.FAIL, Verdict.PASS],
        "resting": [Verdict.PASS, Verdict.PASS],
    }
    assert results["raised_6"][0].reasons == ("raised_6 floats 6.0 mm",)
    assert results["overlapping"][1].reasons == ("overlapping: left and right interpenetrate 4.0 mm",)
    assert results["overlapping"][0].looked_at == (
        "overlapping: mount surface, not floor-standing (not judged here)",
    )
    assert any("cup / table resting, depth 0.0 mm" in line for line in results["resting"][1].looked_at)


def test_a_prop_without_a_mount_and_a_mount_without_a_prop_both_fail(tmp_path: Path) -> None:
    items = dict(_ITEMS)
    del items["raised_2"]
    items["vanished"] = {"mount": "floor"}
    results = run_item_gates(_library(tmp_path, items), Tolerances.load())
    assert results["raised_2"][0].reasons == ("cannot look: no mount declared in contact.json",)
    assert results["vanished"][0].reasons == ("cannot look: declared in contact.json, but no such directory",)


def test_an_unknown_mount_word_is_refused(tmp_path: Path) -> None:
    root = _library(tmp_path, {**_ITEMS, "raised_2": {"mount": "shelf"}})
    with pytest.raises(ValueError, match="raised_2 mount 'shelf' is not one of"):
        load_mounts(root / "contact.json")


def test_a_resting_pair_naming_a_missing_node_cannot_look(tmp_path: Path) -> None:
    root = _library(tmp_path, {**_ITEMS, "resting": {"mount": "floor", "resting": [["cup", "saucer"]]}})
    results = run_item_gates(root, Tolerances.load())
    assert results["resting"][1].reasons == ("cannot look: a resting pair names a missing node ['saucer']",)


def test_a_recorded_finding_is_known_until_it_deepens_or_is_fixed(tmp_path: Path) -> None:
    recorded = {**_ITEMS, "raised_6": {"mount": "floor", "findings": {"C-3": 6.0, "lane": "assets"}}}
    results = run_item_gates(_library(tmp_path, recorded), Tolerances.load())
    assert results["raised_6"][0].reasons == (
        f"{KNOWN}raised_6 floats 6.0 mm (recorded 6.0 mm in contact.json)",
    )
    shallower = {**_ITEMS, "raised_6": {"mount": "floor", "findings": {"C-3": 5.0}}}
    results = run_item_gates(_library(tmp_path / "b", shallower), Tolerances.load())
    assert results["raised_6"][0].reasons == ("raised_6 floats 6.0 mm",)
    stale = {**_ITEMS, "raised_2": {"mount": "floor", "findings": {"C-3": 2.0}}}
    results = run_item_gates(_library(tmp_path / "c", stale), Tolerances.load())
    assert results["raised_2"][0].reasons[0].startswith("stale finding: raised_2")


def test_the_command_checks_a_second_library_by_root_and_names_the_failing_props(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    root = _library(tmp_path, _ITEMS)
    assert main(["contact-items", "--root", str(root)]) == 1
    summary = capsys.readouterr().out.strip().splitlines()[-1]
    assert summary == (
        "contact-items: 4 props; failing ['overlapping', 'raised_6']; recorded findings still open none"
    )


def test_the_shipped_prop_library_passes_or_fails_only_by_recorded_findings(
    capsys: pytest.CaptureFixture[str],
) -> None:
    assert main(["contact-items"]) == 0
    assert "failing none" in capsys.readouterr().out
