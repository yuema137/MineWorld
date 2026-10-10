# Handoff — S15: PR 12n-1 implementation (the walk)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §21 (DESIGN FROZEN 2026-10-09), with evidence, deviations and
findings in §21.15 (`E-NV<n>`, `N-D<n>`). The 12d-0 context is CLOSED; its handoff text is in git
history (`git log -- .structured-coding/plans/mvp0/handoff.md`).

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12n-1 — the walk (S15, navigation framework)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §21; evidence/deviations §21.15
RELATED / BINDING   §21.14's list: §§16–20; step-19 §4 (QTW-13); step-12 §16 (D-SB6); DECISIONS
                    ARC-23, 25, 26, 27, 34, 39, 42, 55, 62, 67; REUSE_POLICY; ENGINEERING_RULES §§4–9;
                    CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12n-navigation in /Users/yuema137/mineworld-worktrees/impl-12n (sole writer)
BASE                origin/main @ 551fb2c (#100 merged: §21 frozen)
APPROVED SCOPE      §21.14: NV-C1 … NV-C7; SD-N1 … SD-N13 (SD-N11 = TD-D8); paths systems/movement,
                    systems/bodies, systems/installed, Cargo.lock (pathfinding + deps only),
                    tools/cli/tests/walking.rs (+ fixtures), clients/protocol/ADOPTION.md,
                    docs/{DECISIONS,MODULE_SPEC,MVP_STATUS}.md, systems/README.md, step-11 §21, handoff
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/src/, server/, systems/presence/,
                    clients/ (but ADOPTION.md), worlds/; no pack or controller reads the scale or a
                    wall clock (NV-11); bodies' resolution rules unchanged (NUDGE_MAX, BIAS_BAND only
                    become expressions of R, equal at R 300); NV-1 byte-identical for every world
ENDPOINT AUTHORITY  source: §21.14 and the coordinator's kickoff for 12n-1 (2026-10-09)
  implementation + local validation   authorized
  semantic commits, branch push       authorized ("commit and push after each step")
  PR creation / update                authorized; READY FOR OPERATOR REVIEW
  scratch builds                      /tmp/s15-12n; no branch pushed from them
  CI repair                           authorized; `fast` and `test` green on the exact head; `python`
                                      not required (red on main for an unrelated reason); F-12n-CI1
                                      re-run once and recorded
  merge                               operator only; never inherited
TOOL DISCIPLINE     Read/Edit/Write for files; cargo, git, gh (no merge), python3 scripts/*, mkdir -p,
                    sed -n, target/*/mineworld, /usr/bin/time; never python3 -c, sed -i, awk, xargs,
                    curl, cat >> / heredoc writes; long jobs in the background
VALIDATION BUDGET   four 300-day town runs (NV-1: base 2 + final 2); bodies-yard 30-day runs and
                    committed tests unrestricted; NV-5's 2 000 scenes; NV-10 timing once; x86_64 build
                    once (+ its Rosetta runs); one full gate on the final head
STOP CONDITIONS     NORMAL: 12n-1 READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §21.14's list
                    (presence/kernel/contracts change, a scale read, a path outside scope, NV-1 not
                    byte-identical, NV-7 failing at R 250, a digest change, cross-arch differing, a cap)
POST-MERGE SYNC     planning session: step header, overall, MVP_STATUS S15 lines; this session: §21 ledger
```

## Current checkpoint — READY FOR OPERATOR REVIEW (M-1 ruled (a) and applied; ARC-73 → ARC-75)

Context CLOSED / AWAITING OPERATOR ACTION. The exact-head CI is in the PR (#116), not here.

### Earlier checkpoint — NV-C7 (close), held at MATERIAL STOP M-1

- Commits: 1200b90 NV-C1 · b8dc78a NV-C2+C3 · 3847ec2 NV-C4 · 1796c29 NV-C5 · c1fab19 NV-7 evidence + M-1
  · 030e67f NV-C6 · MVP_STATUS · 9fbc807 merge of origin/main (fb1d701; DECISIONS conflict: ARC-75/DEP-34
  beside ARC-68, both kept).
- M-1 (§21.15): at R 250 with derived NUDGE_MAX the n3 crowd is never nudged ("the crowd was nudged"
  fails); claim 1 passes at R 250 and everything passes at R 300. Awaiting the operator's ruling — options
  (a)/(b)/(c) in §21.15. The towns do not enable bodies, so no ruling changes their digests.
- Record ids: `ARC-W` → ARC-75, `DEP-P` → DEP-34 (N-D1).
- Binaries: /tmp/s15-12n/base-mineworld (551fb2c), c5-mineworld (1796c29), head-mineworld (9fbc807 + ledger).
  Scripts: /tmp/s15-12n/capture.sh. Scratch R 250 worktree: /tmp/s15-12n/r250 (detached, uncommitted
  edits; target /tmp/s15-12n/target-r250). Town 300-day runs used: 4 of 4 (cap reached; no more without
  authorization).

## Next actions

1. Record E-NV5 (head captures, x86_64), push, open the PR marked "awaiting M-1 ruling".
2. On the ruling: apply it (bodies tests/geometry only), re-run NV-7 at R 300 and R 250, bodies-yard and
   long-run captures (towns unaffected), push; CI on the exact head.
