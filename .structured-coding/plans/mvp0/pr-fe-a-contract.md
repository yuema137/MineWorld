# PR FE-a — execution contract (filled)

**Lifecycle:** `FROZEN` (2026-11-27), with [`pr-fe-a-food-energy-engine.md`](pr-fe-a-food-energy-engine.md)
DESIGN FROZEN; every question in [`step-fe-food-energy.md`](step-fe-food-energy.md) §1 ruled by the operator. The generic body that this block specializes is
`.claude/skills/structured-coding/prompts/implementation-working-rules.md` §1 onward (read in full by the executor;
not copied here).

```text
PROJECT / PR:
  MineWorld mvp0 — Step FE, PR FE-a: food-energy engine

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-fe-a-food-energy-engine.md

RELATED / BINDING DOCS:
  .structured-coding/plans/mvp0/step-fe-food-energy.md (operator answers §1, acceptance §4);
  .structured-coding/plans/mvp0/operator-playable-mvp-20261123.md;
  .structured-coding/plans/mvp0/step-35-needs-and-sleep.md; pr-s35-nb-needs.md;
  CLAUDE.md; AGENTS.md; docs/{VISION,CORE_CONCEPTS,ARCHITECTURE,MODULE_SPEC,ENGINEERING_RULES,
  ENGINEERING_STANDARDS}.md; docs/decisions/ARC-284.md, ARC-285.md; .structured-coding/standards.md;
  .claude/skills/structured-coding/prompts/{implementation-working-rules,test-ci-gate-rules}.md (full).
  History only: .structured-coding/plans/mvp0/fe-evidence/.

IMPLEMENTATION BASE:
  origin/main @ 24dbc3bf at freeze (mapped source unchanged since the 8c971f0f re-audit) or later at
  dispatch; C0 records the exact base. If any mapped path (design §4) changed since 24dbc3bf, re-audit those seams before C1 (bounded), or stop if a
  contract in design §3 no longer fits (material).

IMPLEMENTATION WORKTREE / BRANCH / SESSION:
  /Users/yuema137/mineworld-worktrees/fe-a-engine, branch feat/fe-a-food-energy-engine, one fresh
  Claude Opus 5.5 session (CLAUDE.md §3.4 ruling 2026-11-27). Never the root checkout, never /tmp.
  Confirm no other session holds the worktree before editing.

APPROVED SCOPE:
  Design §3 contracts and §4 paths, commits C0–C4 exactly: needs v2 food-energy model; SDK per-owner
  schema constants and validate_authored hook; ConfigurationRefusal::Owner; KernelError::
  InvalidGenesisContent (domain-neutral); PersistentWorld::create_with_backend and the two CLI new-save
  callers; schema-aware shared/2D/3D/agent/LM views; synthetic test worlds and fixtures; owning specs and
  generated reference. No file under worlds/ changes.

FROZEN INVARIANTS:
  - One hunger state per person (closed variant); sleep mode only from tiredness Level.
  - Integer, world-time-only, lazy reserve; i128 intermediates; cap and zero discard; no Process,
    runtime fact or random draw added to needs.
  - economy sole money owner; inventory sole holdings owner; consumption unchanged; one inventory unit =
    one whole serving.
  - Kernel gains one domain-neutral error only; dispatch behaviour unchanged; contracts crate unchanged.
  - Version table design §3.4 exactly; every other pack stays at schema 1.
  - No semantic-invalid world allocates a save backend; old create() API kept.
  - Old saves refused explicitly; logically committed contents preserved; no migration, reset, re-genesis.
  - Holder-only disclosure; no client/agent/LM kcal, threshold or rate arithmetic; malformed v2 never
    falls back to a deficit bar; scripts/check_client_rules.py unmodified.
  - Unitless default and absent-needs behaviour unchanged; Social Cafe / Bodies Yard untouched.
  - Daily use per world and per actor class only; no per-person field (Q-FE-6).
  - Public agent tool schema changes only by the optional NeedsLine.food_energy of design §3.8, via the
    regenerated reference.
  - No new dependency, God object, generic resource framework, controller policy or optimizer change.

APPROVED IMPLEMENTATION SEQUENCE:
  C0 contracts/docs → C1 SDK + error seams → C2 valid-genesis-first creation → C3 needs v2 + synthetic
  persisted commerce → C4 schema-aware views + actual both-client proof → independent review → PR/CI.
  Each commit: separate Implementation / Validation / Independent review items with evidence.

AUTONOMOUS VALIDATION BUDGET:
  - Static, unit, integration, CLI lifecycle, Godot headless checks, bounded windowed 2D/3D synthetic
    launches: unrestricted within R-LOAD-1.
  - R-LOAD-1: one test file at a time locally; heavy cargo builds/tests through
    /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh; never a local full-workspace run;
    CI on the exact head is the full gate.
  - Real LLM calls, paid APIs, inference, 30/300-day or long runs, benchmarks: none (not needed;
    LM behaviour is asserted deterministically).
  - Expected aggregate local wall time <= ~1 h including retries; beyond that, stop with a projection.
  - Missing Godot/display → that claim is INCONCLUSIVE, never a pass.

REQUIRED LIVE DOCUMENTATION:
  PRIMARY DESIGN DOC: commit ledger, commands, counts, wall time, outcomes, tested head, deviations,
  review findings and dispositions.

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-fe-a.md (created by the executor at C0; continuation aid only).

PR CONTEXT SCOPE:
  FE-a only. FE-b (Market Town content) is a separate, later design and session.

PR CONTEXT INITIALIZATION / AUTHORITY ORDER / CONTINUITY / COMPACTION / RESUME / CLOSEOUT:
  As in the generic working rules. Authority order: repository/git/process truth → primary design →
  binding docs → handoff → emergency snapshot → conversation. No hooks enabled.

ENDPOINT AUTHORITY:
  - implementation + local validation:  authorized (effective at DESIGN FROZEN)
      source: operator, 2026-11-27, Q-FE-5 approved as recommended: "同意三个都按推荐冻结"
  - semantic commits:                   authorized, no approval per commit
      source: same
  - branch push:                        authorized
      source: same
  - PR creation / update:               authorized
      source: same
  - CI repair to review readiness:      authorized
      source: same
  - merge:                              explicit operator authorization only; section 22.
  If Q-FE-5 is answered differently, record the actual answer here verbatim before execution.

POST-MERGE SYNCHRONIZATION OWNER:
  Implementation session: this PR document, merge identity, evidence, deviations, remaining issues.
  Primary coordinator: step-fe-food-energy.md progress, operator-playable-mvp-20261123.md, overall.md,
  progress page, and dispatch of FE-b design.

NORMAL STOP CONDITION:
  PR FE-a READY FOR OPERATOR REVIEW — DO NOT MERGE.

STOP CONDITION:
  implementation C0–C4 complete + E1–E7 evidence recorded + independent review dispositions recorded +
  PR opened/updated + required CI green on the exact final head.

MATERIAL STOPS (return to the operator with evidence and the smallest revision):
  A new domain owner or any domain rule in kernel/persistence/server/clients; a version or wire shape
  beyond design §3.4/§3.8; migration, reset or hot-removal; a new dependency or spend; weaker
  persistence/history semantics; any controller-policy or worlds/ content change; bypassing or editing
  the R7 client-rule guard; budget expansion.

NEVER:
  python3 -c; sed -i; perl -pi; curl/wget; /tmp worktrees or artifacts; writing or deleting a user save;
  deleting WAL as a repair; a local full-workspace test run; editing worlds/*; merging.

ALLOWED DATA / RESOURCES:
  Synthetic authored worlds and scratch save families (mineworld-test-support scratch guards); copies of
  complete SQLite + WAL + SHM families only.

PLATFORMS:
  macOS, Linux, Windows (CI); no platform-specific code expected.

MERGE AUTHORITY:
  NEVER merge without explicit operator approval of the concrete candidate.
```
