# Handoff — S15: PR 12n-2 implementation (people walk there)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §21 (DESIGN FROZEN 2026-10-09): the medium plan §21.13, its
detail §21.13.1 and the open rulings §21.13.2; evidence, deviations and findings in §21.15 (`E-NW<n>`,
`N-D<n>`). The 12n-1 context is CLOSED (PR #116 merged, ecc8d40); its handoff text is in git history
(`git log -- .structured-coding/plans/mvp0/handoff.md`).

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12n-2 — people walk there (S15)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §21.13, §21.13.1; ledger §21.15
RELATED / BINDING   §21.14's list: §§16–20; step-19 §4 (QTW-13); step-12 §16 (D-SB6); DECISIONS
                    ARC-23, 25, 26, 27, 34, 42, 55, 62, 67, 75; ENGINEERING_RULES §§4–9; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12n2-walk in /Users/yuema137/mineworld-worktrees/impl-12n2 (sole writer; N-D10)
BASE                origin/main @ ecc8d40 (12n-1 merged)
APPROVED SCOPE      §21.14 12n-2: NW-C1 … NW-C4, SD-N14 … SD-N16; paths cognition/rule-controller/src,
                    tools/cli/src/{hosted,run}.rs, tools/cli/tests/, docs (ARC-27, ARC-42 notes;
                    MODULE_SPEC §8.1), step-11 §21, this handoff; plus N-D12 and A-10 if ruled (§21.13.2)
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/src/, server/, systems/, clients/,
                    worlds/; no pack or controller reads the scale or a wall clock; §21.8 as written
ENDPOINT AUTHORITY  §21.14 and the primary session's kickoff (2026-10-09)
  Phase 1           detailed plan committed, pushed, draft PR; STOP for the primary's approval
  after approval    implementation, local validation, semantic commits, push, PR updates, CI repair
                    (fast + test + platforms green on the exact head; F-12n-CI1 re-run once)
  merge             operator only; 12n-2 merges immediately before 12d (QN-2)
TOOL DISCIPLINE     Read/Edit/Write for files; cargo, git, gh (no merge), python3 scripts/*, mkdir -p,
                    sed -n, /usr/bin/time, arch -x86_64; never python3 -c, sed -i, awk, xargs, curl,
                    heredoc writes
VALIDATION BUDGET   §21.14 12n-2: ≤ 19 town 300-day runs in all (8 + 4 + 1 + 2 + 4); NW-9 ≤ two attempts
                    per scale, ≤ 3 wall minutes each; 30-day runs unrestricted
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §21.14's list (NW-1/NW-2/
                    NW-9 failing, a scale read, a path outside scope, cross-arch digests differing, a cap)
POST-MERGE SYNC     planning session: step header, overall, MVP_STATUS S15 lines; this session: §21 ledger
```

## Current checkpoint — NW-C2 committed (ac9055c); MATERIAL STOP M-2 (NW-4 no-bodies cost)

- f241a1b NW-C0 plan · 50685e6 merge of origin/main (#123, #124) · 4416ee8 NW-C1 (hosts pace steps;
  byte-identical, E-NW1) · ac9055c NW-C2 (people walk there; E-NW2; NW-4 FAIL → M-2, E-NW3).
- M-2 (§21.15): 300 d user CPU with walking 52.16 s (social-cafe) / 88.02 s (market-town) vs NW-4's
  27.8 / 23.9 s; 1.88 × / 2.18 × today's base. Options (a) restate the bound, (b) wander stays a
  `move`; recommendation (a). Held: NW-C3, NW-C4.
- Binaries /tmp/s15-12n/12n2/{base,c1,c2}-mineworld; script /tmp/s15-12n/12n2/capture.sh; logs
  cap-*.log, nw4-main.log, c2-nw7.log. Town 300-day runs used: 4 of 19.

## Next actions

1. On the M-2 ruling: apply it (if (b), re-measure NW-4 with the 2 remaining NW-4 runs + debug runs).
2. NW-C3 — scratch merge with origin/mvp0/pr-12d-towns @ 8814aad under /tmp/s15-12n/12n2/merge.
3. NW-C4 — workspace tests once, CI on the exact head, READY FOR OPERATOR REVIEW.
