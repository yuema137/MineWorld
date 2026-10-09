# Handoff — S19: PR TW-a implementation (the `calendar` System Pack)

A continuation aid, never a design authority. The authority is
[`step-19-time-weather.md`](step-19-time-weather.md) §16 (DESIGN FROZEN 2026-10-08), its ledger §16.9.

```text
PROJECT / PR        MVP-0 · Step 19 / PR TW-a — the calendar System Pack · PR #94
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-19-time-weather.md §16 (ledger §16.9)
RELATED / BINDING   step-19 §§1–15; CLAUDE.md §§2–4; DECISIONS ARC-26, ARC-28, ARC-33, ARC-35, ARC-61,
                    ARC-62, ARC-67, DEP-30; IL-a's API on main
BRANCH / WORKTREE   mvp0/pr-tw-a-calendar in /Users/yuema137/mineworld-worktrees/impl-tw-a (sole writer);
                    C4 parked on mvp0/pr-tw-a-c4-proposed (cf91bbb), not in the PR
BASE                origin/main @ 543c80a; origin/main aa74b32 merged in at e7fdee7
APPROVED SCOPE      §16.1/§16.3
FROZEN INVARIANTS   INV-TW-1 … INV-TW-5; IL-a API unchanged; only market-town's digest moves; no edit
                    to kernel, contracts, presence, persistence, server, clients, schedule, other packs;
                    one new dependency only (solar-positioning =0.7.0, libm). Operator 2026-10-08: macOS,
                    Linux and Windows — no Unix-only assumption in new code
ENDPOINT AUTHORITY  source: the coordinator's kickoff for TW-a (2026-10-08) and §16.7
  implementation, commits, push, PR, CI repair   authorized
  merge                                          operator only; never inherited
VALIDATION BUDGET   six 300-day town runs; used 4
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. REACHED INSTEAD: MATERIAL —
                    C4 vs AC-1 check 3 (TWa-F1, E-TWa-4)
POST-MERGE SYNC     planning (primary) session: step-19 header, overall.md
```

## Current checkpoint — CLOSED / AWAITING OPERATOR ACTION (C4 decision)

- C1–C3 committed; the full gate and CI are recorded (E-TWa-6, E-TWa-7). C5 is the ledger.
- C4: needs a decision between (i), (ii) and (iii) in §16.9 "C4 decision requested". If (i) is chosen,
  the AC-1 lane amends ARC-35 and check 3; cf91bbb then lands on this branch, the market-town digest
  `24a95d2a…d270` (E-TWa-5) is recorded as the baseline with that commit, and town runs 5–6 re-confirm
  it.
- Open housekeeping: 4 empty `mineworld-kill-*` directories in the system temp dir, from the
  load-raced kill test (E-TWa-6b).
