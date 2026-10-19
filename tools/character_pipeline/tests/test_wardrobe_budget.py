"""The pilot's credit budget (§4.6): every rule refuses before a POST; the measured charge is what counts."""

from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path

import pytest

from mwcast import meshy
from mwcast.meshy import Ledger, LedgerEntry, load_ledger, run_live
from mwcast.wardrobe.budget import BudgetRefusal, admit, projection, reconcile, reports_crossed
from mwcast.wardrobe.recipe import load_recipe

RECIPE = load_recipe(Path(__file__).resolve().parents[1] / "recipes" / "s32p-pilot.yaml")
BUDGET = RECIPE.budget
FAKE_KEY = "msy_" + "R" * 24


def _entry(task: str, phase: str, asset: str, charged: int, measured: int | None = None) -> LedgerEntry:
    return LedgerEntry(
        task_id=task,
        phase=phase,
        asset=asset,
        provisional=30,
        measured=measured,
        balance_delta=None,
        charged=charged,
    )


def _ledger(*entries: LedgerEntry) -> Ledger:
    return Ledger(
        spent=sum(e.charged for e in entries), runs=tuple(e.task_id for e in entries), entries=entries
    )


def test_the_first_body_run_is_admitted_at_30() -> None:
    admission = admit(_ledger(), BUDGET, phase="V-body", asset="body:vera-lindgren", kind="body")

    assert (admission.attempt, admission.projection, admission.is_retry) == (1, 30, False)


def test_a_garment_gets_two_retries_then_is_refused() -> None:
    ledger = _ledger(*(_entry(f"t{i}", "V-garments", "garment:vera-tee", 30, 30) for i in range(3)))

    with pytest.raises(BudgetRefusal, match=r"refused \(retries\)"):
        admit(ledger, BUDGET, phase="V-garments", asset="garment:vera-tee", kind="garment")
    assert admit(ledger, BUDGET, phase="V-garments", asset="garment:vera-jeans", kind="garment").attempt == 1


def test_vera_s_body_ceiling_is_90() -> None:
    ledger = _ledger(*(_entry(f"t{i}", "V-body", "body:vera-lindgren", 30, 30) for i in range(3)))

    with pytest.raises(BudgetRefusal, match=r"refused \(ceiling V-body\)"):
        admit(ledger, BUDGET, phase="V-body", asset="body:vera-lindgren", kind="body")


def test_the_phase_v_ceiling_refuses_at_390() -> None:
    entries = [_entry(f"t{i}", "V-garments", f"garment:g{i}", 30, 30) for i in range(12)]
    ledger = _ledger(_entry("b", "V-body", "body:vera-lindgren", 30, 30), *entries)  # 390

    with pytest.raises(BudgetRefusal, match=r"refused \(ceiling phase V\)"):
        admit(ledger, BUDGET, phase="V-garments", asset="garment:vera-tee", kind="garment")


def test_a_larger_measured_charge_raises_the_projection() -> None:
    ledger = _ledger(_entry("t1", "V-body", "body:vera-lindgren", 40, 40))

    assert projection(ledger, BUDGET) == 40
    assert (
        admit(ledger, BUDGET, phase="V-garments", asset="garment:vera-tee", kind="garment").projection == 40
    )


def test_the_ih_gate_needs_450_remaining_before_the_first_ih_run() -> None:
    with pytest.raises(BudgetRefusal, match=r"refused \(phase gate\)"):
        admit(
            Ledger(spent=760, runs=(), entries=()),
            BUDGET,
            phase="IH-body",
            asset="body:ivan-petrov",
            kind="body",
        )
    admit(
        Ledger(spent=750, runs=(), entries=()), BUDGET, phase="IH-body", asset="body:ivan-petrov", kind="body"
    )
    started = _ledger(_entry("i1", "IH-body", "body:ivan-petrov", 30, 30))
    late = Ledger(
        spent=800, runs=started.runs, entries=started.entries
    )  # the gate applies to the first run only
    admit(late, BUDGET, phase="IH-body", asset="body:hana-sato", kind="body")


def test_m_p_7_a_ledger_at_1190_refuses_the_next_run_before_any_request(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """M-p-7: the ledger set to 1,190 -> refused; the fake transport records zero requests."""
    monkeypatch.setenv(meshy.KEY_ENV, FAKE_KEY)
    ledger = Ledger(spent=1190, runs=(), entries=())
    calls: list[str] = []

    class Recording:
        def request(
            self, method: str, url: str, headers: Mapping[str, str], body: bytes | None, timeout_s: float
        ) -> tuple[int, bytes]:
            calls.append(url)
            return 500, b"{}"

    with pytest.raises(BudgetRefusal, match=r"refused \(cap\)"):
        admit(ledger, BUDGET, phase="IH-garments", asset="garment:hana-skirt", kind="garment")
    meshy.write_ledger(tmp_path / "credits.json", ledger)
    image = tmp_path / "in.png"
    image.write_bytes(b"\x89PNG fake")
    with pytest.raises(meshy.MeshyError, match="cap"):
        run_live(
            image=image,
            overrides={},
            run_dir=tmp_path / "run",
            ledger_path=tmp_path / "credits.json",
            cap=BUDGET.cap,
            transport=Recording(),
            sleep=lambda _s: None,
            charge=reconcile,
        )
    assert calls == []


def test_reconcile_charges_the_larger_of_consumed_and_balance_delta() -> None:
    ledger = _ledger(_entry("t1", "V-body", "body:vera-lindgren", 30))

    agreed = reconcile(
        ledger, "t1", 20, {"balance_before": {"balance": 500}, "balance_after": {"balance": 480}}
    )
    larger = reconcile(
        ledger, "t1", 20, {"balance_before": {"balance": 500}, "balance_after": {"balance": 465}}
    )
    unmeasured = reconcile(ledger, "t1", None, {"balance_before": {}, "balance_after": {}})

    assert (agreed.spent, agreed.entries[0].charged, agreed.entries[0].finding) == (20, 20, "")
    assert larger.spent == 35 and "differs from the balance delta 35" in larger.entries[0].finding
    assert unmeasured.spent == 30 and "provisional charge stands" in unmeasured.entries[0].finding


def test_a_live_run_with_the_hook_writes_the_measured_charge(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv(meshy.KEY_ENV, FAKE_KEY)
    balances = iter([b'{"balance": 1000}', b'{"balance": 975}'])

    class Fake:
        def request(
            self, method: str, url: str, headers: Mapping[str, str], body: bytes | None, timeout_s: float
        ) -> tuple[int, bytes]:
            if url.endswith("/balance"):
                return 200, next(balances)
            if method == "POST":
                return 200, b'{"result": "task-9"}'
            return 200, json.dumps({"status": "SUCCEEDED", "consumed_credits": 25}).encode("utf-8")

    image = tmp_path / "in.png"
    image.write_bytes(b"\x89PNG fake")
    run_live(
        image=image,
        overrides={},
        run_dir=tmp_path / "run",
        ledger_path=tmp_path / "credits.json",
        cap=1200,
        transport=Fake(),
        sleep=lambda _s: None,
        charge=reconcile,
        phase="V-body",
        asset="body:vera-lindgren",
    )

    ledger = load_ledger(tmp_path / "credits.json")
    assert ledger.spent == 25
    assert ledger.entries[0].measured == 25 and ledger.entries[0].balance_delta == 25
    assert ledger.entries[0].phase == "V-body"


def test_the_600_report_is_due_when_crossed() -> None:
    assert reports_crossed(580, 610, BUDGET) == [600]
    assert reports_crossed(600, 630, BUDGET) == []
