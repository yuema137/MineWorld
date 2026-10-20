"""Records: a valid fit and generation log load; unknown keys and wrong types are refused with the key
path; the override rule; and the scratch refusal."""

from __future__ import annotations

from pathlib import Path
from typing import cast

import pytest

from mwcast.records import (
    RecordError,
    load_record,
    overrides,
    parse_fit,
    parse_generation_log,
)
from mwcast.scratch import ScratchError, resolve_scratch

SOURCE = Path("fit.yaml")


def _vera_fit() -> dict[str, object]:
    """Vera's values, as PIPELINE.md §4.1 and design §5.4 give them, with flecks on (the default)."""
    return {
        "format": 1,
        "character": "vera-lindgren",
        "stature_m": 1.767,
        "accessories": ["backpack"],
        "source": {
            "meshy_task": "01a1155c-dafe-70bc-9279-9cef9a35856b",
            "download_file": "model_glb.glb",
            "download_sha256": "3de801cdb7dcf34d21ee45758982319096bd62f3a111ab354c46fb2eeb586ef7",
            "input_sha256": "fc749a474dcb5b7c1ee1b45e601b2bb0aa8c38e6016811d67c5813d0a68b05fd",
            "reference": {
                "path": "presentation/mineworld-default/3D/references/04_character_closeup.png",
                "sha256": "5b0c8f0b818a9ca7555515e7374f2ec40770a09bc990f8c568b6dc913e41a3a1",
                "generator": "ChatGPT gpt-image (C2PA, references/PROVENANCE.md §1)",
            },
        },
        "mask": {
            "session": "birefnet-portrait",
            "crop_box": [230, 40, 860, 1086],
            "fg_ratio": 0.85,
            "size_px": 1024,
            "alpha_threshold": 16,
        },
        "gameready": {
            "target_triangles": 95000,
            "eye_z_raw": 0.779,
            "centre_band_z_raw": 0.05,
            "centre_band_half_raw": 0.03,
            "sole_z_m": -0.0177,
            "eye_from_sole_m": 1.655,
            "weld_m": 1e-06,
            "bake": {
                "size_px": 2048,
                "samples": 1,
                "margin_px": 8,
                "extrusion_m": 0.015,
                "ray_m": 0.04,
                "device": "auto",
            },
        },
        "fit": {
            "rig_source": "clients/3d-spike/assets/characters/vitruvian/vitruvian.glb",
            "charmorph_body": "tools/character_pipeline/rig_source/body_g5.glb",
            "charmorph_landmarks_z": [-0.0177, 0.083, 0.51, 0.947, 1.401, 1.531, 1.637],
            "picks": {
                "frame": "dplus-v1",
                "view_px": 1024,
                "px_per_raw_unit": 512,
                "r_shoulder": [418, 228],
                "l_shoulder": [606, 228],
                "r_elbow": [325, 355],
                "l_elbow": [699, 355],
                "r_wrist": [248, 480],
                "l_wrist": [776, 480],
                "r_handtip": [200, 530],
                "l_handtip": [828, 530],
                "r_hip": [467, 490],
                "l_hip": [557, 490],
                "r_knee": [470, 730],
                "l_knee": [553, 730],
                "r_ankle": [458, 935],
                "l_ankle": [565, 935],
                "chin": [512, 155],
                "eye": [512, 113],
            },
        },
        "weights": {
            "head_rigid": {"above_chin_m": 0.01, "half_width_m": 0.16},
            "pack_core": {
                "behind_spine1_m": 0.075,
                "above_hips_m": 0.02,
                "above_spine2_m": 0.10,
                "half_width_m": 0.12,
            },
            "smooth": {"factor": 0.5, "repeat": 4, "limit": 4},
            "fingers": {"hand_share_min": 0.95, "power": 6, "keep": 2},
            "pack_panels": {
                "x_in": 0.12,
                "x_out": 0.21,
                "sleeve_share_max": 0.5,
                "arm_zero": True,
                "smooth": True,
                "smooth_repeat": 8,
            },
        },
        "flecks": {"enabled": True, "z_min_m": 1.65, "dark": 0.30, "lighter": 0.12},
        "export": {"material": "MW_Cast"},
    }


def vera_fit_object() -> dict[str, object]:
    """Vera's record as a fresh object, for other test modules (gates, pipeline)."""
    return _vera_fit()


def _child(parent: dict[str, object], key: str) -> dict[str, object]:
    value = parent[key]
    assert isinstance(value, dict)
    return cast(dict[str, object], value)


def test_a_valid_record_parses_and_its_only_override_is_counted() -> None:
    raw = _vera_fit()
    raw["flecks"] = {"enabled": False, "z_min_m": 1.65, "dark": 0.30, "lighter": 0.12}

    fit = parse_fit(raw, SOURCE)

    assert fit.character == "vera-lindgren"
    assert fit.fit.picks["eye"] == (512, 113)
    assert overrides(fit) == ["flecks.enabled"]


def test_the_spine_depth_defaults_to_hips_relative_and_takes_only_its_two_values() -> None:
    assert parse_fit(_vera_fit(), SOURCE).fit.spine_depth == "hips-relative"  # #211's rule, unchanged
    raw = _vera_fit()
    _child(raw, "fit")["spine_depth"] = "body-centre"  # S32-p: a bare base-layer body (Ivan's G-6)
    assert parse_fit(raw, SOURCE).fit.spine_depth == "body-centre"
    _child(raw, "fit")["spine_depth"] = "sideways"

    with pytest.raises(RecordError, match="spine_depth"):
        parse_fit(raw, SOURCE)


def test_the_defaults_themselves_are_not_overrides() -> None:
    assert overrides(parse_fit(_vera_fit(), SOURCE)) == []


def test_an_unknown_top_level_key_is_refused_with_its_name() -> None:
    raw = _vera_fit()
    raw["bogus"] = 1

    with pytest.raises(RecordError, match="bogus: unknown key"):
        parse_fit(raw, SOURCE)


def test_an_unknown_nested_key_is_refused_with_its_key_path() -> None:
    raw = _vera_fit()
    _child(raw, "mask")["extra"] = 2

    with pytest.raises(RecordError, match=r"mask\.extra: unknown key"):
        parse_fit(raw, SOURCE)


def test_a_string_where_a_number_belongs_is_refused_with_its_key_path() -> None:
    raw = _vera_fit()
    _child(_child(raw, "weights"), "pack_panels")["x_out"] = "0.2"

    with pytest.raises(RecordError, match=r"weights\.pack_panels\.x_out: expected a number"):
        parse_fit(raw, SOURCE)


def test_a_bad_json_line_is_refused_with_its_line(tmp_path: Path) -> None:
    path = tmp_path / "fit.yaml"
    path.write_text('{\n  "format": 1,\n  "character": \n}\n', encoding="utf-8")

    with pytest.raises(RecordError, match="line 4"):
        load_record(path)


def test_a_pick_outside_the_named_set_is_refused() -> None:
    raw = _vera_fit()
    del _child(_child(raw, "fit"), "picks")["eye"]

    with pytest.raises(RecordError, match=r"fit\.picks\.eye: missing"):
        parse_fit(raw, SOURCE)


def test_the_generation_log_totals_match_vera() -> None:
    raw = {
        "runs": [
            {
                "run": "apose_v1",
                "task_id": "01a11555",
                "task_id_complete": False,
                "params": {"mode": "remesh", "target_polycount": 30000},
                "credits": 30,
                "verdict": "rejected",
                "reason": "identity lost",
                "date": "2026-10-07",
            },
            {
                "run": "apose_v2",
                "task_id": "01a11559",
                "task_id_complete": False,
                "params": {"mode": "raw"},
                "credits": 30,
                "verdict": "rejected",
                "reason": "no backpack",
                "date": "2026-10-07",
            },
            {
                "run": "apose_v3",
                "task_id": "01a1155c-dafe-70bc-9279-9cef9a35856b",
                "task_id_complete": True,
                "params": {"mode": "raw"},
                "credits": 30,
                "verdict": "kept",
                "reason": "chosen",
                "date": "2026-10-07",
            },
        ]
    }

    log = parse_generation_log(raw, Path("generation_log.yaml"))

    assert len(log.runs) == 3
    assert log.total_credits == 90
    assert log.kept == 1


def test_scratch_refuses_a_path_inside_the_repository(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()

    with pytest.raises(ScratchError, match="inside the repository"):
        resolve_scratch(str(repo / "scratch"), {}, repo)


def test_scratch_refuses_when_nothing_is_given(tmp_path: Path) -> None:
    with pytest.raises(ScratchError, match="MINEWORLD_CAST_SCRATCH"):
        resolve_scratch(None, {}, tmp_path)


def test_scratch_accepts_a_path_outside_the_repository(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()
    outside = tmp_path / "scratch"

    assert resolve_scratch(None, {"MINEWORLD_CAST_SCRATCH": str(outside)}, repo) == outside.resolve()


REPO = Path(__file__).resolve().parents[3]
VERA_DIR = REPO / "presentation/mineworld-default/3D/characters/cast/vera-lindgren"


def test_vera_committed_records_load_with_one_override() -> None:
    fit = parse_fit(load_record(VERA_DIR / "fit.yaml"), VERA_DIR / "fit.yaml")

    assert overrides(fit) == ["flecks.enabled"]


def test_vera_committed_generation_log_totals_three_runs_ninety_credits_one_kept() -> None:
    log = parse_generation_log(
        load_record(VERA_DIR / "generation_log.yaml"), VERA_DIR / "generation_log.yaml"
    )

    assert len(log.runs) == 3
    assert log.total_credits == 90
    assert log.kept == 1


def test_check_characters_on_the_repository_prints_records_only_and_exits_zero(
    capsys: pytest.CaptureFixture[str],
) -> None:
    from mwcast.cli import main

    assert main(["check-characters"]) == 0
    out = capsys.readouterr().out
    assert "vera-lindgren: records only (no character.yaml)" in out
    assert "alba-ferreira: brief only (no fit.yaml yet)" in out


ALBA_BRIEF = REPO / "presentation/mineworld-default/3D/characters/cast/alba-ferreira/brief.yaml"


def _check(root: Path, capsys: pytest.CaptureFixture[str]) -> tuple[int, str]:
    from mwcast.cli import main

    code = main(["check-characters", "--root", str(root)])
    return code, capsys.readouterr().out


def _cast_dir(root: Path, name: str) -> Path:
    directory = root / "presentation/pack/3D/characters/cast" / name
    directory.mkdir(parents=True)
    return directory


def test_check_characters_passes_a_valid_brief_only_directory(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    directory = _cast_dir(tmp_path, "alba-ferreira")
    (directory / "brief.yaml").write_text(ALBA_BRIEF.read_text(encoding="utf-8"), encoding="utf-8")

    code, out = _check(tmp_path, capsys)

    assert code == 0, out
    assert "alba-ferreira: brief only (no fit.yaml yet)" in out


def test_check_characters_refuses_a_brief_that_is_not_a_casting_brief(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    (_cast_dir(tmp_path, "briefed") / "brief.yaml").write_text("person: briefed\n", encoding="utf-8")

    code, out = _check(tmp_path, capsys)

    assert code == 1
    assert "briefed: FAIL:" in out and "missing key `from`" in out


def test_check_characters_refuses_a_brief_for_another_person(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    directory = _cast_dir(tmp_path, "someone-else")
    (directory / "brief.yaml").write_text(ALBA_BRIEF.read_text(encoding="utf-8"), encoding="utf-8")

    code, out = _check(tmp_path, capsys)

    assert code == 1
    assert "`person` is `alba-ferreira`, the directory is `someone-else`" in out


def test_check_characters_fails_generation_records_without_fit_yaml(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    directory = _cast_dir(tmp_path, "alba-ferreira")
    (directory / "brief.yaml").write_text(ALBA_BRIEF.read_text(encoding="utf-8"), encoding="utf-8")
    (directory / "generation_log.yaml").write_text("{}\n", encoding="utf-8")

    code, out = _check(tmp_path, capsys)

    assert code == 1
    assert "alba-ferreira: FAIL: generation_log.yaml without fit.yaml" in out


def test_check_characters_fails_a_directory_with_neither_fit_nor_brief(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    _cast_dir(tmp_path, "empty")

    code, out = _check(tmp_path, capsys)

    assert code == 1
    assert "empty: FAIL: neither fit.yaml nor brief.yaml" in out
    assert "1 character directories checked; 1 failing" in out
