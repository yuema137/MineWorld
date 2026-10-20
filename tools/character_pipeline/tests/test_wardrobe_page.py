"""The review page's rules (I-p-13, I-p-4): an honest lead, frames withheld on a V-1 FAIL, red stays red."""

from __future__ import annotations

from pathlib import Path

from mwcast.wardrobe import page

PNG = b"\x89PNG\r\n\x1a\nnot really a png"


def _frame(tmp: Path, name: str, ok: bool) -> dict[str, object]:
    path = tmp / f"{name}.png"
    path.write_bytes(PNG)
    return {"id": name, "path": str(path), "pass": ok, "checks": {"poke_through": {"pass": ok}}}


def _subject(tmp: Path, v1: str) -> dict[str, object]:
    outfits = [
        {
            "name": n,
            "garments": ["g"],
            "v1": v1,
            "gates": [{"gate": "W-4", "verdict": "PASS"}],
            "frames": [_frame(tmp, f"{n}-front", True), _frame(tmp, f"{n}-walk", False)],
        }
        for n in ("A", "B")
    ]
    return {"key": "vera-lindgren", "name": "Vera", "outfits": outfits}


def test_a_failed_pilot_leads_with_not_adopted_and_the_levers(tmp_path: Path) -> None:
    data = {
        "status": "failed",
        "subjects": [_subject(tmp_path, "PASS")],
        "levers": [{"id": "(a)", "text": "face kept", "cost": "≈ 300 credits"}],
    }

    text = page.render(data, tmp_path / "out").read_text(encoding="utf-8")

    assert text.index(page.NOT_ADOPTED) < text.index("<section>")
    assert "(a)" in text and "accepted" not in text.lower().replace("accept / reject", "")


def test_passed_without_every_sheet_is_shown_as_pending(tmp_path: Path) -> None:
    subject = _subject(tmp_path, "PASS")
    subject["outfits"] = subject["outfits"][:1]  # type: ignore[index]

    text = page.render({"status": "passed", "subjects": [subject]}, tmp_path / "out").read_text(
        encoding="utf-8"
    )

    assert page.STATUS_LINES["pending"] in text
    assert page.STATUS_LINES["passed"] not in text


def test_m_p_6_frames_of_an_outfit_failing_v1_are_withheld(tmp_path: Path) -> None:
    text = page.render(
        {"status": "pending", "subjects": [_subject(tmp_path, "FAIL")]}, tmp_path / "out"
    ).read_text(encoding="utf-8")

    assert "V-1 FAIL" in text
    assert "A-front" not in text and '<img src="img/vera-lindgren-A' not in text


def test_a_flagged_frame_is_shown_red(tmp_path: Path) -> None:
    text = page.render(
        {"status": "pending", "subjects": [_subject(tmp_path, "PASS")]}, tmp_path / "out"
    ).read_text(encoding="utf-8")

    assert 'class="flagged"' in text and "A-walk" in text
