"""AC-25f-1: the scorer passes the ON-1 fixture on every check this build can pass, and each check bites on a
fixture changed in exactly the way that check exists to catch (PROTOCOL.md §5)."""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

import score
from conftest import FIXTURES

pytestmark = pytest.mark.real_binary

BASE = FIXTURES / "on1" / "harbour-village"


def world(tmp_path: Path) -> Path:
    target = tmp_path / BASE.name
    shutil.copytree(BASE, target)
    return target


def verdicts(path: Path, binary: Path) -> dict[str, score.Check]:
    result = score.score(path, score.load_rubric("ON-1"), score.Binary(binary, []))
    return {check.id: check for check in result.checks}


def replace(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    assert old in text, f"{old!r} not in {path}"
    path.write_text(text.replace(old, new), encoding="utf-8")


def test_the_fixture_passes_s1_to_s4_fails_s5_as_a_product_gap_and_s6_on_its_own_merits(
    tmp_path: Path, binary: Path
) -> None:
    checks = verdicts(world(tmp_path), binary)
    assert [checks[c].verdict for c in ("S1", "S2", "S3", "S4", "S8")] == ["PASS"] * 5, checks
    # Happenings are not in this build, so no world can pass S5, and the reason says so (a product gap).
    assert checks["S5"].verdict == "FAIL" and "product gap" in checks["S5"].evidence, checks["S5"]
    # The decision model lm is (P6b): S6 is judged, and the fixture puts no seat on it.
    assert checks["S6"].verdict == "FAIL" and "product gap" not in checks["S6"].evidence, checks[
        "S6"
    ]
    assert checks["S6"].evidence.startswith("on lm: none"), checks["S6"]
    assert checks["S7"].verdict == "SKIPPED"


def test_s1_bites_on_an_unknown_key(tmp_path: Path, binary: Path) -> None:
    path = world(tmp_path)
    replace(path / "world.yaml", "mineworld: ", "weather_today: sunny\nmineworld: ")
    checks = verdicts(path, binary)
    assert checks["S1"].verdict == "FAIL", checks["S1"]
    assert "exited 1" in checks["S1"].evidence


def test_s2_bites_on_a_world_that_cannot_run(tmp_path: Path, binary: Path) -> None:
    # A world with no seat validates but does not run; S4 fails with it, because its people are not seats.
    path = world(tmp_path)
    replace(path / "world.yaml", "seats:\n  - mara\n  - tom\n  - lin\n", "")
    checks = verdicts(path, binary)
    assert checks["S1"].verdict == "PASS", checks["S1"]
    assert checks["S2"].verdict == "FAIL", checks["S2"]
    assert checks["S3"].verdict == "PASS", checks["S3"]


def test_s3_bites_on_a_village_with_no_bakery(tmp_path: Path, binary: Path) -> None:
    path = world(tmp_path)
    for name in ("world.yaml", "places/quay.yaml", "people/lin.yaml"):
        replace(path / name, "bakery", "shop")
    (path / "places" / "bakery.yaml").rename(path / "places" / "shop.yaml")
    replace(path / "places" / "shop.yaml", "bakery", "store")
    checks = verdicts(path, binary)
    assert checks["S1"].verdict == "PASS" and checks["S2"].verdict == "PASS", checks
    assert checks["S3"].verdict == "FAIL" and "no place says bakery" in checks["S3"].evidence, (
        checks["S3"]
    )
    assert checks["S4"].verdict == "PASS"


def test_s3_bites_on_a_bakery_that_is_not_by_the_sea(tmp_path: Path, binary: Path) -> None:
    path = world(tmp_path)
    replace(path / "places" / "quay.yaml", "  - seaside\n  - harbour\n", "  - square\n")
    checks = verdicts(path, binary)
    assert checks["S3"].verdict == "FAIL" and "seaside" in checks["S3"].evidence, checks["S3"]
    assert checks["S1"].verdict == checks["S2"].verdict == checks["S4"].verdict == "PASS"


def test_s4_bites_on_a_resident_who_is_not_a_seat(tmp_path: Path, binary: Path) -> None:
    path = world(tmp_path)
    replace(
        path / "world.yaml", "seats:\n  - mara\n  - tom\n  - lin\n", "seats:\n  - mara\n  - tom\n"
    )
    checks = verdicts(path, binary)
    assert [checks[c].verdict for c in ("S1", "S2", "S3")] == ["PASS"] * 3, checks
    assert checks["S4"].verdict == "FAIL", checks["S4"]


def test_s6_bites_on_a_provider_address_in_the_pack(tmp_path: Path, binary: Path) -> None:
    path = world(tmp_path)
    (path / "notes.yaml").write_text("model: http://127.0.0.1:11434/v1\n", encoding="utf-8")
    checks = verdicts(path, binary)
    assert checks["S6"].verdict == "FAIL" and "names" in checks["S6"].evidence, checks["S6"]
    assert "product gap" not in checks["S6"].evidence


def test_s8_bites_on_a_run_whose_walks_are_rejected(tmp_path: Path, binary: Path) -> None:
    # Round 1's defect: no door to the bakery, and a routine that sends Lin there. The run exits 0 with 0 faults.
    path = world(tmp_path)
    quay = path / "places" / "quay.yaml"
    text = quay.read_text(encoding="utf-8")
    quay.write_text(text[: text.index("passages:")], encoding="utf-8")
    replace(path / "world.yaml", "  - movement\n", "  - movement\n  - schedule\n")
    lin = path / "people" / "lin.yaml"
    lin.write_text(
        lin.read_text(encoding="utf-8").replace("place: bakery", "place: quay")
        + 'routine:\n  - from: "09:00"\n    place: bakery\n    label: work\n'
        + '  - from: "17:00"\n    place: quay\n    label: home\n',
        encoding="utf-8",
    )
    checks = verdicts(path, binary)
    assert checks["S2"].verdict == "PASS", checks["S2"]
    assert checks["S8"].verdict == "FAIL" and "walk-to" in checks["S8"].evidence, checks["S8"]
