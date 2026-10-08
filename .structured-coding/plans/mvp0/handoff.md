# Handoff — S15: PR 12d-0 implementation (bodies' cost)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §20 (DESIGN FROZEN 2026-10-08), with evidence in §20.12
(`E-Z<n>`) and deviations in §20.13. The 12c context is CLOSED; its handoff text is in git history at
`5ef5bff`.

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12d-0 — bodies' cost (S15, precursor to 12d)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §20; evidence §20.12; deviations §20.13
RELATED / BINDING   §19's header (the QD-2 ruling), §17.11 DB-10 and its ruling, §17, §18 (12b, 12c as
                    merged), §19 (12d, AWAITING 12d-0), E-TD0b; DECISIONS ARC-39, DEP-13; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/s15-12d0-plan in /Users/yuema137/mineworld-worktrees/impl-12d0 (sole writer;
                    the coordinator's kickoff names this branch and worktree, replacing §20.11's
                    proposed mvp0/pr-12d0-cost / s15-12d0 — §20.13 Z-D1)
BASE                main @ f842c52 + the frozen design (953ff10, Markdown only)
APPROVED SCOPE      §20.1's change set; ZC-1 … ZC-6; SD-Z1 … SD-Z5; §20.7's ladder only on a failed gate
FROZEN INVARIANTS   no diff outside systems/bodies/ and the named documents; bodies' rules unchanged but
                    SD-Z4 and SD-Z5; Class I byte-identical (ZI-1 … ZI-4); Class R only under ZR; towns'
                    digests unchanged (TZ-1: ad49c723…c64b, 365b50e0…1d1d); QB-11 1.5 × on the
                    prototype, never re-scoped; nothing of Rapier survives a resolution (I-5); no float
                    outside rapier.rs
ENDPOINT AUTHORITY  source: the coordinator's kickoff for 12d-0 (2026-10-08) and §20.11
  implementation + local validation   authorized ("Implement ZC-1…ZC-6")
  semantic commits, branch push       authorized ("commit, push"; "commit and push after each step")
  PR creation / update                authorized; against main, marked READY FOR OPERATOR REVIEW
  scratch builds                      /tmp/s15-12d0; `rustup target list --installed`, `arch -x86_64`
  CI repair                           N/A — no CI workflow
  merge                               operator only, merge commit; never inherited
TOOL DISCIPLINE     Read/Edit/Write for files; allowed cargo, git, gh, python3 scripts/*, mkdir -p,
                    sed -n; never python3 -c, sed -i, awk, xargs, curl, cat >> / heredoc writes. Long
                    runs in the background. No edit of .claude/settings*.json or other worktrees.
VALIDATION BUDGET   §20.11: 300-day prototype runs 8 before (ZC-1), 8 for the gate, ≤ 8 per ladder
                    rung; ZR-3's shadow once; x86_64 build once; one full gate; real-model NOT REQUIRED
STOP CONDITIONS     NORMAL: PR 12d-0 READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: TZ-9 failing
                    after the L1/L2 ladder; a Class-I piece moving a Class-I reference (revert, locate
                    the first differing request, report); ZR failing for a Class-R piece; a town digest
                    moving; a path outside systems/bodies/ and the documents; Rosetta differing
POST-MERGE SYNC     planning session: header, §19 (its base and TD-14 references), overall, MVP_STATUS
```

## Current checkpoint — STOPPED (material): SD-Z3 is not result-preserving (§20.13 Z-D2)

- ZC-1: E-Z-base captured (ZI-1 d7025dbc…, ZI-2 53d017d0…, ZI-3 6e4c4015…, TZ-1 both towns, the
  validate outputs). The gate's "before" (TZ-9's eight base runs) NOT yet run: the machine was never
  quiet (load 23 … 61). The profile pending (Z-D3).
- ZC-2: SD-Z2 (`rapier.rs` `index`) committed, byte-identical (E-Z1). SD-Z3 moved ZI-1 and ZI-2 —
  first differing request 307 of long_run (and 518 with a strict 311 mm margin); reverted. Awaiting
  the primary session: drop SD-Z3, or move it to Class R under ZR.
- Not started: ZC-3 (SD-Z1), ZC-4 (SD-Z4), ZC-5 (SD-Z5), ZC-6.
- Base binary: `/tmp/s15-12d0/base-mineworld` (built on 953ff10).
- Prototype rebuilt from §20.12's recipe: `/tmp/s15-12d0/proto/worlds/{social-cafe,market-town}`;
  the copies without bodies: `/tmp/s15-12d0/nobodies/worlds/{social-cafe,market-town}`. All four
  validate (22 entities / 67 genesis facts; 44 / 143; 22 / 53; 44 / 129).
- Scripts (inputs, not evidence): `/tmp/s15-12d0/capture.sh <label> <binary>` (ZI-1 … ZI-3),
  `/tmp/s15-12d0/towns.sh <label> <binary>` (TZ-1).

## Next actions

1. On the primary session's ruling on Z-D2: apply it (drop SD-Z3, or re-implement it as Class R with
   its strides in ZR-3's shadow comparison).
2. Run TZ-9's "before" (`bash /tmp/s15-12d0/gate.sh before /tmp/s15-12d0/base-mineworld`, ≈ 15 min)
   when the load average is low; then ZC-3 (SD-Z1, reach.rs), ZC-4, ZC-5, ZC-6.
