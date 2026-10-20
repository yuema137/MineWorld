"""contact-scene over synthetic props dumps and a generated library (S36-P5 commit 6).

The dumps have the structure `slice_probe_props.gd` writes (a place's floor and solids in the scene's frame,
the body band, each prop's slug, members, 4×4 and support samples); the library is boxes with hand-computable
answers. Each planted defect must fail exactly where planted and pass below its bound.
"""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

import pytest
from contact_shapes import Node, box, write_gltf

from mwcast.cli import main
from mwcast.scene_check import Row, run
from mwcast.tolerances import Tolerances

# The generated library: name -> (low, high) of one box in the model's own frame, metres.
_MODELS: dict[str, tuple[tuple[float, float, float], tuple[float, float, float]]] = {
    "table": ((-0.5, 0.0, -0.4), (0.5, 0.75, 0.4)),
    "chair": ((-0.2, 0.0, -0.2), (0.2, 0.9, 0.2)),
    "cup": ((-0.04, 0.0, -0.04), (0.04, 0.09, 0.04)),
    "pendant": ((-0.7, -0.3, -0.7), (0.7, 0.0, 0.7)),  # hangs from its top
    "clump": ((-0.15, -0.02, -0.15), (0.15, 0.25, 0.15)),  # rooted 20 mm below its origin
}
_MOUNTS: dict[str, dict[str, object]] = {
    "table": {"mount": "floor"},
    "chair": {"mount": "floor"},
    "cup": {"mount": "surface"},
    "pendant": {"mount": "ceiling"},
    "clump": {
        "mount": "ground",
        "rest_mm": 0,
        "embed_mm": 20,
        "why": "Rooted 20 mm below the soil.",
        "walk_through": True,
        "walk_through_why": "Grass bends under a foot.",
    },
}

# The place: a 10 m × 8 m floor, four shell solids along the walls, then the table's solid as index 4
# and the chair's as 5.
_FLOOR = {"min": [0.0, 0.0], "max": [10.0, 8.0]}
_SHELL: list[dict[str, object]] = [
    {"min": [0.0, 0.0], "max": [10.0, 0.3], "height": 2.5},
    {"min": [0.0, 7.7], "max": [10.0, 8.0], "height": 2.5},
    {"min": [0.0, 0.3], "max": [0.3, 7.7], "height": 2.5},
    {"min": [9.7, 0.3], "max": [10.0, 7.7], "height": 2.5},
]
TABLE_AT = (5.0, 4.0)
CHAIR_AT = (5.0, 4.6)  # the chair's box meets the table's at z = 4.4 (touching, not overlapping)


def _solid(cx: float, cz: float, hx: float, hz: float, h: float) -> dict[str, object]:
    return {"min": [cx - hx, cz - hz], "max": [cx + hx, cz + hz], "height": h}


def _library(root: Path, mounts: Mapping[str, Mapping[str, object]] = _MOUNTS) -> Path:
    for name, (low, high) in _MODELS.items():
        (root / name).mkdir(parents=True)
        vertices, faces = box(low, high)
        write_gltf(root / name / f"{name}.gltf", {name: Node(vertices, faces)})
    (root / "contact.json").write_text(json.dumps({"format": 1, "items": mounts}), encoding="utf-8")
    return root


def _at(x: float, y: float, z: float) -> list[float]:
    return [1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0]


def _prop(slug: str, x: float, y: float, z: float, support_y: float | None) -> dict[str, object]:
    return {
        "slug": slug,
        "name": f"Slice/{slug}",
        "parts": None,
        "nodes": [slug],
        "transform": _at(x, y, z),
        "lowest_y": y,
        "support": [[x, z, support_y]] * 9,
    }


def _shipped_like() -> list[dict[str, object]]:
    """A table with a cup on it, a chair tucked to its edge, a pendant over it above the band, grass."""
    return [
        _prop("table", TABLE_AT[0], 0.0, TABLE_AT[1], 0.0),
        _prop("cup", TABLE_AT[0], 0.75, TABLE_AT[1], 0.75),
        _prop("chair", CHAIR_AT[0], 0.0, CHAIR_AT[1], 0.0),
        _prop("pendant", TABLE_AT[0], 2.1, TABLE_AT[1], None),
        _prop("clump", 2.0, 0.0, 2.0, 0.0),
    ]


def _solids(table_half_x: float = 0.5) -> list[dict[str, object]]:
    return [
        *_SHELL,
        _solid(TABLE_AT[0], TABLE_AT[1], table_half_x, 0.4, 0.75),
        _solid(CHAIR_AT[0], CHAIR_AT[1], 0.2, 0.2, 0.9),
    ]


def _dump(
    directory: Path,
    props: Sequence[Mapping[str, object]],
    solids: Sequence[Mapping[str, object]] | None = None,
    place: str = "room",
) -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    document = {
        "format": 1,
        "place": place,
        "label": "GE-1 café",
        "place_shape": {},
        "scene": {
            "base_y": 0.0,
            "floor": _FLOOR,
            "solids": list(solids if solids is not None else _solids()),
        },
        "band": {"low": 0.05, "high": 1.72},
        "props": list(props),
    }
    (directory / f"props-{place}.json").write_text(json.dumps(document), encoding="utf-8")
    return directory


def _rows(
    tmp_path: Path,
    props: Sequence[Mapping[str, object]],
    solids: Sequence[Mapping[str, object]] | None = None,
) -> list[Row]:
    library = _library(tmp_path / "models")
    return run(_dump(tmp_path / "dump", props, solids), library, Tolerances.load())[1]


def _failing(rows: Sequence[Row]) -> list[Row]:
    return [r for r in rows if r.verdict == "FAIL"]


def test_the_shipped_like_scene_passes_every_rule(tmp_path: Path) -> None:
    rows = _rows(tmp_path, _shipped_like())
    assert _failing(rows) == []
    by = {(r.prop, r.rule): r for r in rows}
    assert by[("table#0", "inside")].note.startswith("linked to solid 4")
    assert by[("cup#1", "inside")].note.startswith("linked to solid 4")  # the cup's link is its table's solid
    assert by[("pendant#3", "support")].verdict == "LISTED"
    assert (by[("clump#4", "inside")].verdict, by[("clump#4", "inside")].note) == ("LISTED", "walk_through")
    # touching along an edge: fcl may report the contact at depth 0, or no contact at all
    assert by.get(("table#0 / chair#2", "apart"), by[("table#0", "inside")]).verdict == "PASS"


@pytest.mark.parametrize(("narrower_mm", "verdict"), [(60, "FAIL"), (35, "PASS")])
def test_a_prop_drawn_wider_than_its_solid_fails_past_the_bound(
    tmp_path: Path, narrower_mm: int, verdict: str
) -> None:
    rows = _rows(tmp_path, _shipped_like(), solids=_solids(0.5 - narrower_mm / 1000.0))
    table = next(r for r in rows if (r.prop, r.rule) == ("table#0", "inside"))
    assert table.verdict == verdict
    if verdict == "FAIL":
        assert table.measured == "overhangs solid 4 by 60.0 mm"
        assert table.bound == "scene.mesh_over_solid 40 mm"


def test_a_prop_with_no_solid_under_it_fails_unless_walk_through(tmp_path: Path) -> None:
    lone = [*_shipped_like(), _prop("chair", 7.0, 0.0, 2.0, 0.0)]
    rows = _rows(tmp_path / "a", lone)
    assert [(r.prop, r.rule) for r in _failing(rows)] == [("chair#5", "inside")]
    assert "drawn where no solid is" in _failing(rows)[0].note
    walk = {**_MOUNTS, "chair": {"mount": "floor", "walk_through": True, "walk_through_why": "A test."}}
    library = _library(tmp_path / "b" / "models", walk)
    rows = run(_dump(tmp_path / "b" / "dump", lone), library, Tolerances.load())[1]
    assert _failing(rows) == []


def test_a_chair_moved_into_its_table_fails_naming_both(tmp_path: Path) -> None:
    moved = _shipped_like()
    moved[2] = _prop("chair", CHAIR_AT[0], 0.0, CHAIR_AT[1] - 0.03, 0.0)
    solids = [*_SHELL, _solid(TABLE_AT[0], TABLE_AT[1], 0.5, 0.4, 0.75), _solid(5.0, 4.57, 0.2, 0.2, 0.9)]
    failing = _failing(_rows(tmp_path, moved, solids=solids))
    assert [(r.prop, r.rule) for r in failing] == [("table#0 / chair#2", "apart")]
    assert failing[0].measured.endswith("mm deep")
    assert float(failing[0].measured.split()[0]) == pytest.approx(30.0, abs=0.5)
    assert failing[0].note == "interpenetrate: table / chair"


def test_a_cup_floating_over_its_table_fails(tmp_path: Path) -> None:
    floating = _shipped_like()
    floating[1] = _prop("cup", TABLE_AT[0], 0.756, TABLE_AT[1], 0.75)
    failing = _failing(_rows(tmp_path, floating))
    assert [(r.prop, r.rule, r.measured) for r in failing] == [
        ("cup#1", "support", "lowest point 6.0 mm above its support")
    ]


def test_a_ground_prop_whose_rest_plane_stands_above_the_ground_fails(tmp_path: Path) -> None:
    raised = _shipped_like()
    raised[4] = _prop("clump", 2.0, 0.005, 2.0, 0.0)
    failing = _failing(_rows(tmp_path, raised))
    assert [(r.prop, r.rule, r.measured) for r in failing] == [
        ("clump#4", "support", "rest plane 5.0 mm above its support")
    ]


def test_a_recorded_scene_finding_is_known_until_it_deepens_or_goes_stale(tmp_path: Path) -> None:
    floating = _shipped_like()
    floating[1] = _prop("cup", TABLE_AT[0], 0.756, TABLE_AT[1], 0.75)

    def with_finding(mm: float) -> dict[str, dict[str, object]]:
        return {**_MOUNTS, "cup": {"mount": "surface", "findings": {"scene": [["room", "support", mm]]}}}

    known = run(
        _dump(tmp_path / "a" / "dump", floating),
        _library(tmp_path / "a" / "m", with_finding(6.0)),
        Tolerances.load(),
    )[1]
    assert _failing(known) == [] and [
        r.verdict for r in known if r.prop == "cup#1" and r.rule == "support"
    ] == ["KNOWN"]
    deeper = run(
        _dump(tmp_path / "b" / "dump", floating),
        _library(tmp_path / "b" / "m", with_finding(5.0)),
        Tolerances.load(),
    )[1]
    assert [(r.prop, r.rule) for r in _failing(deeper)] == [("cup#1", "support")]
    stale = run(
        _dump(tmp_path / "c" / "dump", _shipped_like()),
        _library(tmp_path / "c" / "m", with_finding(6.0)),
        Tolerances.load(),
    )[1]
    assert [r.note for r in _failing(stale)] == ["stale finding: it passes now; remove it from contact.json"]
    other_place = run(
        _dump(tmp_path / "d" / "dump", _shipped_like(), place="elsewhere"),
        _library(tmp_path / "d" / "m", with_finding(6.0)),
        Tolerances.load(),
    )[1]
    assert _failing(other_place) == []  # a finding of a place not dumped is not judged


def test_the_command_judges_a_dump_tree_and_exits_by_the_rows(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    library = _library(tmp_path / "models")
    good = _dump(tmp_path / "dump" / "world-a", _shipped_like())
    assert main(["contact-scene", str(good.parent), "--root", str(library)]) == 0
    assert capsys.readouterr().out.strip().splitlines()[-1] == (
        "contact-scene: 1 places, 5 props; 0 failing; 0 recorded findings still open"
    )
    floating = _shipped_like()
    floating[1] = _prop("cup", TABLE_AT[0], 0.756, TABLE_AT[1], 0.75)
    _dump(tmp_path / "dump" / "world-b", floating, place="other")
    assert main(["contact-scene", str(tmp_path / "dump"), "--root", str(library)]) == 1
    out = capsys.readouterr().out
    assert "other  cup#1  support  lowest point 6.0 mm above its support  item.rest.floor 3 mm  FAIL" in out
    assert main(["contact-scene", str(tmp_path / "empty"), "--root", str(library)]) == 1
