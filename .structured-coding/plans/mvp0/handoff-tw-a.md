# Handoff — S19: PR TW-a implementation (the `calendar` System Pack)

A continuation aid, never a design authority. The authority is
[`step-19-time-weather.md`](step-19-time-weather.md) §16 (DESIGN FROZEN 2026-10-08), its ledger §16.9.

```text
PROJECT / PR        MVP-0 · Step 19 / PR TW-a — the calendar System Pack
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-19-time-weather.md §16 (ledger §16.9)
RELATED / BINDING   step-19 §§1–15 (§5 the calendar design, §10 INV-TW-*, §11.2 TW-a row); CLAUDE.md §§2–4;
                    DECISIONS ARC-26, ARC-28, ARC-33, ARC-35, ARC-61, ARC-62; IL-a's API on main
BRANCH / WORKTREE   mvp0/pr-tw-a-calendar in /Users/yuema137/mineworld-worktrees/impl-tw-a (sole writer)
BASE                origin/main @ 543c80a (IL-a #80 merged)
APPROVED SCOPE      §16.1/§16.3: systems/calendar (new), its registry line, market-town opt-in (C4),
                    DECISIONS ARC-67 + DEP-30, CORE_CONCEPTS time terms
FROZEN INVARIANTS   INV-TW-1 … INV-TW-5; IL-a API unchanged; only market-town's digest moves; no edit
                    to kernel, contracts, presence, persistence, server, clients, schedule, other packs;
                    one new dependency only: solar-positioning =0.7.0, no default features, libm
ENDPOINT AUTHORITY  source: the coordinator's kickoff for TW-a (2026-10-08) and §16.7
  implementation + local validation   authorized
  semantic commits, branch push       authorized (commit and push after each commit-plan step)
  PR creation / update                authorized; READY FOR OPERATOR REVIEW
  CI repair                           authorized (`fast` and `test` required on main)
  merge                               operator only; never inherited
TOOL DISCIPLINE     Read/Edit/Write for files; cargo, git, gh (no merge), python3 scripts/*, mkdir -p,
                    sed -n, target/debug/mineworld, /usr/bin/time; never python3 -c, sed -i, awk, xargs,
                    curl, cat >> / heredoc writes. No .claude/settings*, no other worktree.
VALIDATION BUDGET   six 300-day town runs (used: 2 — E-TWa-0 base capture); one full gate
STOP CONDITIONS     NORMAL: TW-a READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §16.7's five;
                    and (found at start) AC-1 check 3 vs C4 — see the ledger
POST-MERGE SYNC     planning (primary) session: step-19 header, overall.md
```

## Current checkpoint

- E-TWa-0 captured on 543c80a (target/tw-a/base-summary.txt): equals main's recorded values.
- Next: C1 docs; then C2 civil/sun; C3 the pack; C4 blocked pending the AC-1 check-3 question.
