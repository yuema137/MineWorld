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

## Current checkpoint — READY FOR OPERATOR REVIEW (Z-D9 ruled: 3.60 × accepted; Z-D8 accepted)

- origin/main (CI) merged into the branch; PR opened READY FOR OPERATOR REVIEW — DO NOT MERGE.

### Earlier checkpoint — TZ-9a fails on social-cafe (§20.13 Z-D9, E-Z7)

- Z-D7 ruled (c): SD-Z3, SD-Z4 dropped. SD-Z5 done (31e3779, VERSION 3, E-Z6); docs (df94e46).
- New bases: ZI-1 23f7fa76…, ZI-2 c8358f8b…, ZI-3 bd6a1002… (E-Z6); Rosetta equal (E-Z7).
- TZ-1, TZ-8, TZ-10, full gate PASS. TZ-9b PASS (max 10.5 ms, p99 0.33 ms). TZ-9a: market-town
  2.999 × PASS; social-cafe 3.60 × FAIL (re-run clean). The ladder not tried. No PR opened.

### Earlier checkpoint — ZR-3 fails for SD-Z3 + SD-Z4 (§20.13 Z-D7, E-Z5)

- Z-D6 ruled: SD-Z6 added (Class I) and done on the PR branch (18d8e48, E-Z4: ZI-1 … ZI-4 identical);
  TZ-9 re-scoped to TZ-9a (3.0 ×) and TZ-9b (50 ms).
- ZC-4 (SD-Z3 + SD-Z4) written and tested, ZR-3 fails on long_run (9 of 418 > 50 mm = 2.15 %); parked
  on `mvp0/s15-12d0-zc4-wip` @ c1b5749. Options (a)/(b)/(c) in Z-D7. ZC-5, ZC-6, TZ-9a/b not run.
- Binaries: `/tmp/s15-12d0/z6-mineworld` (PR head's code), `/tmp/s15-12d0/z34-mineworld` (the WIP).

### Earlier checkpoint — TZ-9 cannot pass within §20 (§20.13 Z-D6, E-Z3)

- Z-D4 ruled: SD-Z1 dropped (§20.4 amendment). The ZC-1 profile (E-Z3, `/usr/bin/sample`, 30 and 300
  days): 49 % of the with-bodies run is entry E3's `nearest_free` lattice scan, 24 % stride/Rapier,
  25 % the rest. Removing all Rapier work leaves ≈ 5.8 × (bound 1.5 ×); removing E3's scan too ≈ 2.0 ×.
  ZC-4 … ZC-6 not started, as ruled. Awaiting the operator.

### Earlier checkpoint — SD-Z1 not result-preserving (§20.13 Z-D4)

- Z-D2 ruled (option (b): SD-Z3 → Class R); TZ-9's instrument fixed (§20.6.1). Both recorded.
- ZC-1: E-Z-base captured (ZI-1 d7025dbc…, ZI-2 53d017d0…, ZI-3 6e4c4015…, TZ-1 both towns, the
  validate outputs). The gate's "before" under §20.6.1: 7 of 8 runs (E-Z-before; social-cafe 7.98 ×,
  market-town ≥ 6.02 ×). The profile pending (Z-D3).
- ZC-2: SD-Z2 (`rapier.rs` `index`) committed, byte-identical (E-Z1). SD-Z3 reverted (Z-D2), to be
  re-applied as Class R in ZC-4.
- ZC-3: SD-Z1 written, moved ZI-1, ZI-2 and ZI-3 (E-Z2; first differing stride long_run's 419th, 14 mm),
  reverted; patch at `/tmp/s15-12d0/sd-z1-reverted.patch`. Also: SD-Z1 buys no measurable CPU on 30
  prototype days. Awaiting the primary session: drop SD-Z1, or move it to Class R under ZR.
- Not started: ZC-4 (SD-Z3 + SD-Z4), ZC-5 (SD-Z5), ZC-6. No PR opened.
- Base binary: `/tmp/s15-12d0/base-mineworld` (built on 953ff10).
- Prototype rebuilt from §20.12's recipe: `/tmp/s15-12d0/proto/worlds/{social-cafe,market-town}`;
  the copies without bodies: `/tmp/s15-12d0/nobodies/worlds/{social-cafe,market-town}`. All four
  validate (22 entities / 67 genesis facts; 44 / 143; 22 / 53; 44 / 129).
- Scripts (inputs, not evidence): `/tmp/s15-12d0/capture.sh <label> <binary>` (ZI-1 … ZI-3),
  `/tmp/s15-12d0/towns.sh <label> <binary>` (TZ-1).

## Next actions

1. On the primary session's ruling on Z-D4: drop SD-Z1, or re-apply the kept patch as Class R
   (`git apply /tmp/s15-12d0/sd-z1-reverted.patch`; its two wrong cull scenarios rewritten from
   measured values).
2. ZC-4 (SD-Z3 behind `Policy.exact_corridor`, SD-Z4, ZR-3's shadow), ZC-5, ZC-6; TZ-9 with
   `bash /tmp/s15-12d0/gate.sh after <binary>` (CPU instrument, interleaved).
3. Worth taking first (Z-D3, information): the ZC-1 profile — where ≈ 7 s of CPU per 30 prototype
   days goes, since scene size is ruled out.
