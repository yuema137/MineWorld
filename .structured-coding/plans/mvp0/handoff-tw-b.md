# Handoff — S19: PR TW-b implementation (the `weather` System Pack, seeded rules)

A continuation aid, never a design authority. The authority is
[`step-19-time-weather.md`](step-19-time-weather.md) §17 (DESIGN FROZEN 2026-10-09), its ledger §17.11.

```text
PROJECT / PR        MVP-0 · Step 19 / PR TW-b — the weather System Pack (seeded rules)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-19-time-weather.md §17 (ledger §17.11)
RELATED / BINDING   CLAUDE.md; step-19 §§4–6, §10 (INV-TW-1 … 10), §11, §14.1, §16 (TW-a, merged);
                    DECISIONS ARC-26, ARC-28, ARC-33, ARC-35 (+ 2026-10-09 note), ARC-61, ARC-67, DEP-30;
                    ENGINEERING_STANDARDS; ENGINEERING_RULES; CORE_CONCEPTS
BRANCH / WORKTREE   mvp0/pr-tw-b-weather in /Users/yuema137/mineworld-worktrees/impl-tw-b (sole writer)
BASE                origin/main @ a454e37 (#107 merged: §17 frozen; TW-a on main). IL-b (#102) OPEN at start
APPROVED SCOPE      §17.1; change set §17.10
FROZEN INVARIANTS   INV-TW-1, -2, -3, -5, -10; SD-TW-b-1 … 13; §17.6 criteria and mutations. Operator:
                    macOS, Linux, Windows; realistic defaults with sources; integers only (no f32/f64 in
                    the pack); the time scale never reaches a rule; worlds without weather byte-identical
ENDPOINT AUTHORITY  source: the coordinator's kickoff for TW-b (2026-10-09) and §17.10
  implementation, commits, push, PR, CI repair   authorized
  merge                                          operator only; never inherited
SEQUENCE            C1 → C2 → C3 → C4 → C5 (§17.5); merge origin/main at the end (adapt to IL-b's
                    three-argument `seed` if it has merged, §17.9.1)
VALIDATION BUDGET   ≤ 8 town runs of 300 days; full workspace test ≤ 2 runs; unit statistics unrestricted
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §17.10 (1)–(7)
POST-MERGE SYNC     planning (primary) session: step-19 header, overall.md
```

## Current checkpoint — MATERIAL STOP (6) open; PR open, not READY

- C1–C4 are committed and pushed: 7b975eb, fa9b0e8, da31cd0 (TWb-D9, accepted), 4b922da, 0728fa9; merge
  52a3c1c (origin/main 551fb2c: IL-b, 13b, P3); 0599b2b (IL-b `seed`); 5928db2 (opt-in save checks,
  ledger). Evidence E-TWb-0 … 6 and mutations are in §17.11.
- INV-TW-1 PASS. The new market-town baseline is `90479fd8…ae57` (375 527 facts). CP-TW-b PASS. AC-1
  14/14. Town runs 6 of 8.
- **TWb-F1, material stop (6):** criterion 5 fails as frozen. The only differences are Process ids
  shifted by +1 (the climate Process takes id 2 at genesis). The proposed revision is in TWb-F1. Awaiting
  the operator's ruling; the PR is not marked READY until then.
- TWb-D11: the twin-session reconciliation; every change kept.
- Saves (large, untracked): target/tw-b/{base,head}/market-save (about 2.8 GB each), target/tw-b/cp/{A,B,R}.
  Scripts: target/tw-b/capture.sh and gate.sh. Logs: target/tw-b/*.log.
- Next after a ruling: if the revision is accepted, record the ruling and mark the PR READY on a green
  exact head. Do not merge.
