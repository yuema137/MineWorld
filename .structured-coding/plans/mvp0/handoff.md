# Handoff — S15: PR 12b implementation (people: walls and nudging)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §17 (§17.0's freeze record binds and overrides the rest of
§17), with evidence in §17.10 (`E-PB<n>`) and deviations in §17.11. The 12a context is CLOSED; its
handoff text is in git history at `918c869`.

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12b — people: walls and nudging (S15, second of five; the first
                    resolver pack, `bodies`, on Rapier)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §17; evidence §17.10; deviations §17.11
RELATED / BINDING   step-11 header freeze record (QB-1, QB-10, QB-15), §§4.4–4.7, 5, 6.3, 9.6–9.8, 10.1,
                    11.1, 15.1, §16 (12a as merged); overall.md §3 (S15), §7; DECISIONS ARC-15, ARC-23,
                    ARC-25, ARC-26, ARC-27, ARC-30, ARC-31, ARC-33, ARC-35, ARC-39, DEP-12, DEP-13 (this
                    PR); REUSE_POLICY §§11–12, 15, 17; MODULE_SPEC §§3.1, 4.1; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12b-people in /Users/yuema137/mineworld-worktrees/s15-12b (sole writer)
BASE                main @ 918c869 (03f1d7c + the 12a post-merge docs and the 12b planning merge #58,
                    Markdown only)
APPROVED SCOPE      §17.1's change set, with §17.0's override: rapier3d declared in
                    systems/bodies/Cargo.toml, NOT the root manifest (QP-3 overruled); PB-C1 … PB-C8;
                    SD-B1 … SD-B16 as answered by QP-1 … QP-16
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/, server/, cognition/, clients/,
                    authoring/, sdk/, systems/presence/, systems/movement/, worldpack/src/, tools/cli/src/,
                    worlds/social-cafe/, worlds/market-town/; no other existing System Pack. Root
                    Cargo.toml untouched unless QP-9 (PB-14(b) > 1.5×) — then only dev opt-level
                    overrides for rapier3d/parry3d. PB-1: social-cafe sha ad49c723…c64b (365 330 facts)
                    and market-town sha 365b50e0…1d1d (372 755 facts), faults 0. Existing tests unchanged
                    except QP-2's two files. Rapier named only in systems/bodies/src/rapier.rs (and the
                    pack's manifest); nothing of Rapier survives a resolution; no float persisted.
                    I-11 (≤ 310 mm, ≤ 2 generations, ≤ 4 people), I-12 (595 mm on integers, blocked →
                    halved → stay), I-13 (inert where PlaceShape is absent). DEP-13 before any code.
ENDPOINT AUTHORITY  source: the primary session's kickoff message for 12b (2026-10-07) and §17.9
  implementation + local validation   authorized (kickoff: "Implement PB-C0 … PB-C8")
  semantic commits, branch push       authorized (kickoff: "commit and push"; "Commit and push after
                                      every small step")
  PR creation / update                authorized; open against main marked READY FOR OPERATOR REVIEW
  scratch builds                      authorized: base binary before PB-C6, x86_64-apple-darwin build
                                      (PB-12), both under /tmp/s15-12b; `rustup target add/list` allowed
  root-manifest profile override      only if PB-14(b) fails (QP-9)
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit — NOT this session
VALIDATION BUDGET   unit/integration/static unrestricted; 300-day runs ≤ 8 in all (PB-1 ×2 at PB-C6 and
                    ×2 at PB-C8, M-PB1 ×1, PB-14(b) ×4, one re-run each); 30-day bodies-yard runs inside
                    their tests; the x86_64 build once and its Rosetta runs; one full workspace gate on the
                    final head (background); about 90 minutes; real model NOT REQUIRED
STOP CONDITIONS     normal: PR 12b READY FOR OPERATOR REVIEW — DO NOT MERGE. Material: any kernel or
                    contract change; a change to an existing world's facts; a path outside §17.1; a
                    root-manifest edit other than QP-9's; Rapier named outside rapier.rs; a persisted
                    float; HB-1 failing on every rung of PB-13's ladder; PB-14(a) > 100 µs or (b) > 1.5×
                    without QP-9 passing; PB-12 FAIL (not PARTIAL); an existing test failing for a
                    reason other than QP-2
```

## MATERIAL STOP (2026-10-07)

PB-14(b) fails, 2.06× against 1.5×, also with QP-9's override (reverted). Evidence E-PB8, options
DB-10 (§17.11). Reported to the primary session; no PR opened. PB-C1 … PB-C7 committed and pushed
(through db64471); PB-12 PASS, PB-14(a) PASS. Pending the decision: PB-1 and M-PB1's 300-day half on
the final head, the full gate, PB-C8's commit of the close, the PR.

## Checkpoint before the stop

PB-C1 … PB-C5 committed and pushed: 5a5bbd0, 3074db9, b48cecc, c16ab2a, 539ebbf. The pack exists,
is not yet installed. Evidence E-PB1 … E-PB5; deviations DB-1 … DB-7 (§17.11; DB-4, the blocked
walker's stop at first contact, is flagged for the operator). Base binary and its validate outputs:
/tmp/s15-12b/base-mineworld, /tmp/s15-12b/base-validate-*.txt. Budget used: 300-day runs 0 of 8.

## Next actions

- PB-C6: install (systems/installed two lines + manifest), QP-2's two test edits (registration.rs,
  seam_vocabulary.rs — the admission must also cover registration.rs's lines, which name `bodies`),
  `cargo test -p mineworld-cli` (background), PB-1's two 300-day digests and validate cmp.
- PB-C7: worlds/bodies-yard, tools/cli/tests/bodies{,_yard,_yard_restart}.rs, long_run.rs.
- PB-C8: Rosetta (x86_64 build in background), cost, MVP_STATUS, full gate once, PR.
