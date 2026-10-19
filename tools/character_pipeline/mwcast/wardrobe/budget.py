"""The pilot's Meshy credit budget (design §4.6; I-p-3; F-S32p-1). Pure functions over ledger and recipe.

The rules, each refused **before** the POST with its name:

1. **cap** — `spent + projection > cap`. The projection is `max(credits_per_run, the largest measured
   charge so far)`, so a run that measured more than 30 raises every later projection.
2. **ceiling** — the run would take a phase ceiling over (V-body 90; phase V 390).
3. **retries** — the asset already used its first run plus its retry budget (2 per body, garment, hair).
4. **retry pool** — the credits charged to retries so far plus this retry's projection exceed the pool (600).
5. **phase gate** — the first run of a gated phase (IH) is admitted only when `cap - spent >= min_remaining`
   (450): a subject is never started that the budget cannot finish.

After the task, `reconcile` (the `meshy.run_live` charge hook) replaces the provisional charge with the
measured one: the larger of the task's `consumed_credits` and the balance delta; a disagreement is a finding.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, replace
from typing import cast

from mwcast.meshy import Ledger, LedgerEntry
from mwcast.wardrobe.recipe import Budget


class BudgetRefusal(Exception):
    """A run was refused before its POST. The message names the rule and the numbers."""


@dataclass(frozen=True)
class Admission:
    phase: str
    asset: str
    kind: str
    attempt: int  # 1 for the first run of the asset
    projection: int

    @property
    def is_retry(self) -> bool:
        return self.attempt > 1


def asset_name(kind: str, ident: str) -> str:
    """The ledger's name of an asset: `body:<subject>`, `garment:<id>` or `hair:<id>`."""
    return f"{kind}:{ident}"


def projection(ledger: Ledger, budget: Budget) -> int:
    """§4.6: `max(30, the largest consumed_credits measured so far)`. A run's charge can exceed its measured cost
    when another consumer shares the account (the balance delta is then the larger; F-impl-5); that is charged to
    the ledger but does not predict this run's cost, so it is used only when no measurement exists."""
    measured = [
        e.measured if e.measured is not None else e.charged
        for e in ledger.entries
        if e.measured is not None or e.balance_delta is not None
    ]
    return max([budget.credits_per_run, *measured])


def charged_in(ledger: Ledger, phases: tuple[str, ...]) -> int:
    return sum(e.charged for e in ledger.entries if e.phase in phases)


def runs_of(ledger: Ledger, asset: str) -> list[LedgerEntry]:
    return [e for e in ledger.entries if e.asset == asset]


def retry_spend(ledger: Ledger) -> int:
    """Credits charged to every run that was not its asset's first."""
    seen: set[str] = set()
    total = 0
    for entry in ledger.entries:
        if entry.asset in seen:
            total += entry.charged
        seen.add(entry.asset)
    return total


def admit(ledger: Ledger, budget: Budget, *, phase: str, asset: str, kind: str) -> Admission:
    """Admit one run or raise `BudgetRefusal` naming the rule. Nothing is sent by this function."""
    if phase not in budget.phases:
        raise BudgetRefusal(f"refused (phase): {phase!r} is not a budget phase")
    if kind not in budget.retries:
        raise BudgetRefusal(f"refused (kind): {kind!r} is not body, garment or hair")
    p = projection(ledger, budget)
    if ledger.spent + p > budget.cap:
        raise BudgetRefusal(
            f"refused (cap): spent {ledger.spent} + projection {p} = {ledger.spent + p} exceeds the cap "
            f"{budget.cap}; no request was made"
        )
    for ceiling in budget.ceilings:
        if phase in ceiling.phases:
            used = charged_in(ledger, ceiling.phases)
            if used + p > ceiling.credits:
                raise BudgetRefusal(
                    f"refused (ceiling {ceiling.name}): charged {used} + projection {p} "
                    f"exceeds {ceiling.credits}"
                )
    previous = runs_of(ledger, asset)
    allowed = 1 + budget.retries[kind]
    if len(previous) >= allowed:
        raise BudgetRefusal(
            f"refused (retries): {asset} already has {len(previous)} runs; "
            f"a {kind} gets 1 + {budget.retries[kind]}"
        )
    if previous:
        spent_on_retries = retry_spend(ledger)
        if spent_on_retries + p > budget.retry_pool:
            raise BudgetRefusal(
                f"refused (retry pool): retries charged {spent_on_retries} + projection {p} exceed the pool "
                f"{budget.retry_pool}"
            )
    if phase in budget.gate_phases and not any(e.phase in budget.gate_phases for e in ledger.entries):
        remaining = budget.cap - ledger.spent
        if remaining < budget.gate_min_remaining:
            raise BudgetRefusal(
                f"refused (phase gate): {remaining} credits remain before the first {phase} run; "
                f"{budget.gate_min_remaining} are needed to finish the gated subjects (rule 4)"
            )
    return Admission(phase=phase, asset=asset, kind=kind, attempt=len(previous) + 1, projection=p)


def _balance(response: object) -> int | None:
    """The credit balance of a `/balance` response (`balance`, as Meshy documents it; `credits` also read)."""
    if not isinstance(response, dict):
        return None
    data = cast(dict[str, object], response)
    for key in ("balance", "credits"):
        value = data.get(key)
        if isinstance(value, int) and not isinstance(value, bool):
            return value
    return None


def reconcile(ledger: Ledger, task_id: str, consumed: int | None, balances: Mapping[str, object]) -> Ledger:
    """The charge hook: the run's provisional charge replaced by the measured one (rule 3 of §4.6)."""
    index = max((i for i, e in enumerate(ledger.entries) if e.task_id == task_id), default=-1)
    if index < 0:
        raise BudgetRefusal(f"reconcile: task {task_id} has no ledger entry")
    entry = ledger.entries[index]
    before = _balance(balances.get("balance_before"))
    after = _balance(balances.get("balance_after"))
    delta = before - after if before is not None and after is not None else None
    readings = [v for v in (consumed, delta) if v is not None]
    findings: list[str] = []
    if readings:
        charged = max(readings)
        if consumed is not None and delta is not None and consumed != delta:
            findings.append(
                f"consumed_credits {consumed} differs from the balance delta {delta}; the larger charged"
            )
    else:
        charged = entry.provisional
        findings.append(
            "no measured charge (no consumed_credits, no balance delta): the provisional charge stands"
        )
    new_entry = replace(
        entry, measured=consumed, balance_delta=delta, charged=charged, finding="; ".join(findings)
    )
    entries = (*ledger.entries[:index], new_entry, *ledger.entries[index + 1 :])
    return Ledger(spent=ledger.spent - entry.charged + charged, runs=ledger.runs, entries=entries)


def reports_crossed(before: int, after: int, budget: Budget) -> list[int]:
    """The report thresholds (§4.6 rule 5) that a change of `spent` from `before` to `after` crossed."""
    return [t for t in budget.report_at if before < t <= after]


def summary(ledger: Ledger, budget: Budget, kept: Mapping[str, bool]) -> list[str]:
    """The report lines: totals, per phase, the measured cost per kept asset, and S26's running total."""
    lines = [f"pilot spent {ledger.spent} of {budget.cap} credits ({len(ledger.entries)} itemised runs)"]
    for phase in budget.phases:
        lines.append(f"  {phase}: {charged_in(ledger, (phase,))}")
    for kind in ("body", "garment", "hair"):  # ledger assets are named `<kind>:<id>` (asset_name)
        assets = sorted({e.asset for e in ledger.entries if e.asset.startswith(f"{kind}:")})
        kept_assets = [a for a in assets if kept.get(a, False)]
        total = sum(e.charged for e in ledger.entries if e.asset in assets)
        per = f"{total / len(kept_assets):.1f}" if kept_assets else "n/a (none kept)"
        lines.append(
            f"  {kind}: {total} credits over {len(assets)} assets, {len(kept_assets)} kept; per kept {per}"
        )
    prior = sum(p.credits for p in budget.s26_prior if p.inside_approval)
    outside = sum(p.credits for p in budget.s26_prior if not p.inside_approval)
    lines.append(
        f"  S26 running total {prior + ledger.spent} of {budget.s26_approval} "
        f"(prior inside the approval {prior}; listed separately, before it: {outside})"
    )
    return lines
