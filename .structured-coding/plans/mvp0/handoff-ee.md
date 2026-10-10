# Handoff — S16 PR E-e (Lakeside and Milestone E)

Continuation aid only; the authority is `step-16-packages.md` §18 (DESIGN FROZEN 2026-10-10, FQ-e1 …
FQ-e8 ruled in §18.0, execution contract §18.11). Nothing here overrides it.

```text
PROJECT / PR        MVP-0 · S16 / PR E-e — Lakeside and Milestone E (draft PR #137)
PRIMARY DESIGN      .structured-coding/plans/mvp0/step-16-packages.md §18
BINDING             step-16 §§1–9, §§14–17, §16.12, §17.12; overall.md "Parallel build-out, 2026-10-08" and
                    later rulings; CLAUDE.md; docs/ENGINEERING_STANDARDS.md, MODULE_SPEC.md,
                    PACKAGE_FORMAT.md, DECISIONS.md ARC-23, ARC-33, ARC-53, ARC-54, ARC-66, ARC-71, ARC-77
BRANCH / WORKTREE   mvp0/pr-ee-lakeside · /Users/yuema137/mineworld-worktrees/impl-ee (this session only)
BASE                main @ a926fbf (merged as 5192fb6); origin/main @ bb62edf merged as 3e4a638 (docs only)
DECISION            ARC-77 (assigned by the primary, FQ-e3); no DEP
SCOPE               §18.1–18.6 with FQ-e1 (a): Ee-C7 in this PR
STOPS               §18.11 MATERIAL STOPS (no product code, no Cargo.toml/lock, no controller edit, PD-50's
                    ceiling, PD-47 by content only, S13's files beyond FQ-e1's roots)
ENDPOINTS           implementation, commits, pushes (branch and scratch/ee-*), PR update and ready, CI
                    repair; merge NOT authorized
SEQUENCE            Ee-C1 14d778a → C2 535bf8d → merge 3e4a638 → C3 1b483d4 → C4…C6 3e5c95c (one commit,
                    F-Ee7) → C7 36d0338 → C8 (docs, ledger, this file)
```

## Current checkpoint

Every commit through Ee-C7 is implemented and validated locally (§18.8 E-Ee3, E-Ee4). The milestone
test passes 7/7 locally (34.6 s, ≈ 2.2 GB scratch). Ee-C7's scratch pushes (`scratch/ee-36d0338-scenario`
and `-image`) are the evidence for EE-14; the PR's own CI (fast, test, platforms, test-windows,
test-macos) needs the PR marked ready.

**BLOCKED on two material findings (§18.8 F-Ee6, F-Ee8), the primary's to rule.** EE-14 FAILS:
the native AC-8 legs lack the presentation manifests (F-Ee8, fix proven on scratch `7f4fb6f`), and with
that fixed `ac8` fails G-5 on Lakeside's `validate` key on Windows alone, because a required pack's
directory prints with `\` there (F-Ee6; run 38037163670). Everything else is green locally (E-Ee8).

## Next actions

1. On the ruling: apply F-Ee8's two lines per leg to `ci.yml` and F-Ee6's chosen fix; push a new
   `scratch/ee-<sha>-scenario`; `ac8` must be green with four records.
2. PR #137 CI (fast, test, platforms, test-windows, test-macos) green on the exact final head.
3. READY FOR OPERATOR REVIEW with §8.3's checklist (in `docs/HUMAN_REVIEW_QUEUE.md`, Milestone E).
   Do not merge.
4. After the operator's merge: record the merge identity in §18; delete `scratch/ee-*` branches. The
   primary owns §9.6's status, the step header, overall.md and the progress page.
