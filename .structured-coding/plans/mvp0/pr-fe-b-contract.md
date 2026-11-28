# PR FE-b — execution contract

**Lifecycle:** `DRAFT — FREEZE-READY` (2026-11-27). Becomes `FROZEN` only when the operator answers
[`pr-fe-b-market-town-food-content.md`](pr-fe-b-market-town-food-content.md) §1 and the primary marks the design
`DESIGN FROZEN`. The generic body this specializes is `.claude/skills/structured-coding/prompts/implementation-working-rules.md`
§1 onward (read in full by the executor; not copied here).

```text
PROJECT / PR:
  MineWorld mvp0 — Step FE, PR FE-b: Market Town food-energy content and liquidity/stock proof

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-fe-b-market-town-food-content.md

RELATED / BINDING DOCS:
  .structured-coding/plans/mvp0/step-fe-food-energy.md (§1 rulings, §4 NE-3/5/7/9/10, §6 blockers);
  pr-fe-a-food-energy-engine.md §3 (model, authoring forms, arithmetic);
  operator-playable-mvp-20261123.md; step-35-needs-and-sleep.md; CLAUDE.md; AGENTS.md;
  docs/{VISION,CORE_CONCEPTS,ARCHITECTURE,MODULE_SPEC,ENGINEERING_RULES,ENGINEERING_STANDARDS,REUSE_POLICY}.md;
  .structured-coding/standards.md; skill prompts implementation-working-rules.md and test-ci-gate-rules.md (full).
  History only: fe-evidence/20261123-*.md.

IMPLEMENTATION BASE:
  origin/main at dispatch (design audited at c373b802). C0 records the base and re-audits every path in design §2
  that changed since c373b802 (bounded); a premise P1–P4 (design §5.2) that no longer holds is material: stop.

IMPLEMENTATION WORKTREE / BRANCH / SESSION:
  /Users/yuema137/mineworld-worktrees/fe-b-content, branch feat/fe-b-market-town-food, one fresh Claude Opus 5.5
  session. Never the root checkout, never /tmp. Confirm no other session holds the worktree before editing.

APPROVED SCOPE:
  Design §4 content and §6 commits C0–C4 exactly: Market Town items/people/organizations/configure/needs.yaml,
  new decisions.yaml + world.yaml key, README "Food and money", CHANGELOG entry, MODULE_SPEC decisions-example fix;
  test oracles (needs_town.rs kcal replay, food_liquidity.rs, fast bound test) and literal-fixture updates C0 lists;
  NE-3/NE-5/NE-7 evidence on Market Town. Evidence files under fe-evidence/.

FROZEN INVARIANTS:
  - No production Rust, Python, GDScript or kernel change; the rule controller is configured, not modified.
  - economy sole mover of money; no money created after genesis; no new faucet, grant or reset.
  - One inventory unit = one whole serving; kcal and prices exactly as design §3 after C0 pinning.
  - C0 may change a price only within Q-FEb-4 ceilings (ρ_max ≤ 1.6636 ¢/kcal, croissant ≤ 415, sandwich ≤ 865);
    otherwise stop.
  - chance_spend_permille 0 only through Market Town's decisions.yaml; the global default stays 1.
  - Thresholds of CP-4, S35 and design §5.4 fixed as written; never retuned after a run; a FAIL returns to the
    operator.
  - Social Café, Bodies Yard, Commute Town content unchanged; Commute Town stays unitless.
  - No new dependency; no client rule; holder-only disclosure unchanged.

APPROVED IMPLEMENTATION SEQUENCE:
  C0 re-audit and source pinning → C1 content → C2 oracles → C3 Market Town acceptance → C4 review and ledger.

VALIDATION AUTHORITY:
  Targeted tests only, through /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh (R-LOAD-1); never a local
  full-workspace run. The 300-day × 3-seed runs (design §5.4) are requested from the coordinator's shared slot. CI on
  the exact head is the gate. Godot client captures via the existing E6 harness; missing Godot → INCONCLUSIVE.

SPENDING / EXTERNAL CALLS:
  No paid calls, no model inference. Public web reads for source pinning (C0) only; no personal data sent.

COMMIT / PUSH / PR AUTHORITY (if Q-FEb-7 is approved):
  Semantic commits, push, open the PR, repair CI to READY FOR OPERATOR REVIEW. Trailer
  "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>". No merge.

STOP CONDITIONS:
  A premise P1–P4 false in code; a price outside the ceilings; any §5.4 or S35 threshold FAIL; a needed change
  outside the approved scope; another session holding the worktree.

COMPLETION:
  Every design §6 item [x] with evidence or N/A with reason; §5.4 table filled with PASS/FAIL/INCONCLUSIVE and
  numbers; independent review findings dispositioned; Lifecycle READY FOR OPERATOR REVIEW — DO NOT MERGE.
```
