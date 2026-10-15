"""DL-a's decision JSON read as the LM controller's assignment (pr-s10-p6 §5.9, C3, AP6-10).

Every refusal names the file (or the key) and is raised before anything is joined.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from binary import REPOSITORY, mineworld
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.config import ConfigError
from mineworld_cognition.controller.assignment import read_assignment

FIXTURE = Path(__file__).parent / "fixtures" / "decisions_format1.json"


def _write(directory: Path, data: dict[str, object]) -> Path:
    path = directory / "seats.json"
    path.write_text(json.dumps(data), encoding="utf-8")
    return path


def _fixture() -> dict[str, object]:
    loaded: dict[str, object] = json.loads(FIXTURE.read_text(encoding="utf-8"))
    return loaded


def test_the_fixture_selects_alice_with_her_memory_choice() -> None:
    assignment = read_assignment(FIXTURE, None)
    assert assignment.world == "social-cafe"
    assert [seat.key for seat in assignment.seats] == [EntityKey("alice")]
    memory = assignment.seats[0].memory
    assert memory is not None
    assert memory.policy == "consolidate"


def test_narrowing_to_an_lm_seat_keeps_it_and_a_non_lm_seat_is_refused(tmp_path: Path) -> None:
    assert [seat.key for seat in read_assignment(FIXTURE, ["alice"]).seats] == [EntityKey("alice")]
    with pytest.raises(ConfigError, match="not an lm seat"):
        read_assignment(FIXTURE, ["bob"])


def test_an_unknown_format_is_refused_by_name(tmp_path: Path) -> None:
    data = _fixture()
    data["format"] = 2
    with pytest.raises(ConfigError, match="unknown decision JSON format") as refused:
        read_assignment(_write(tmp_path, data), None)
    assert str(tmp_path) in str(refused.value)


def test_a_file_with_no_lm_seat_is_refused(tmp_path: Path) -> None:
    data = _fixture()
    seats = data["seats"]
    assert isinstance(seats, list)
    seats[0]["model"] = {"id": "rule"}  # alice is no longer assigned lm
    with pytest.raises(ConfigError, match="no lm seat in"):
        read_assignment(_write(tmp_path, data), None)


def test_an_unknown_field_is_refused_by_location(tmp_path: Path) -> None:
    data = _fixture()
    data["surprise"] = True
    with pytest.raises(ConfigError, match="surprise"):
        read_assignment(_write(tmp_path, data), None)


def test_a_missing_file_is_refused_by_name(tmp_path: Path) -> None:
    missing = tmp_path / "absent.json"
    with pytest.raises(ConfigError, match=r"absent\.json"):
        read_assignment(missing, None)


@pytest.mark.real_binary
def test_the_real_binary_decisions_output_is_read_as_the_assignment(tmp_path: Path) -> None:
    output = mineworld(
        "decisions",
        REPOSITORY / "worlds" / "social-cafe",
        "--decisions",
        REPOSITORY / "cognition" / "lm-controller" / "examples" / "alice-lm.decisions.yaml",
        "--json",
    )
    path = tmp_path / "seats.json"
    path.write_text(output, encoding="utf-8")
    assignment = read_assignment(path, None)
    assert assignment.world == "social-cafe"
    assert [seat.key for seat in assignment.seats] == [EntityKey("alice")]
