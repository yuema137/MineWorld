# Handoff — S15: PR 12a implementation (the arrival-resolver seam)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §16 (§16.0's freeze record binds and overrides the rest of
§16), with evidence in §16.10 (`E-RS<n>`) and deviations in §16.11. The S9 contexts (11a … 11f) are
CLOSED; their handoff text is in git history at `b8afd4f`.

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12a — the arrival-resolver seam (S15, first of five; a framework
                    precursor that names no physics)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §16; evidence §16.10; deviations §16.11
RELATED / BINDING   step-11 header freeze record (QB-1, QB-10, QB-15 and its three bounds), §§4.3–4.4,
                    4.6–4.7, 8.2, 10.1 (I-1, I-2, I-6, I-7, I-13), 11.1, 15.3; overall.md §3 (S15), §7;
                    DECISIONS ARC-15, ARC-23, ARC-25, ARC-26, ARC-33, ARC-35 and its notes, DEP-12, and
                    ARC-39 (this PR); MODULE_SPEC §3.1; ENGINEERING_RULES §15; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12a-resolver-seam in /Users/yuema137/mineworld-worktrees/s15-12a (sole writer)
BASE                main @ 6d48e03 (b8afd4f + the 12a planning merge #53, Markdown only)
APPROVED SCOPE      §16.1's change set; RS-C1 … RS-C8; SD-R1 … SD-R14 as answered by QR-1 … QR-12
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/, server/, cognition/, clients/,
                    worlds/, authoring/, tools/cli/src/, root Cargo.toml; no System Pack but presence and
                    movement; movement exactly SD-R9. RS-1: social-cafe sha ad49c723…c64b (365 330
                    facts) and market-town sha 365b50e0…1d1d (372 755 facts), faults 0, no digest
                    re-baselined. Existing tests unchanged except QR-2's three `presence v2` literals.
                    Presence's declaration unchanged (QR-3); VERSION 3 (QR-2). QB-15 bounds as §16.9.
                    The seam names no physics (RS-13); synthetic packs in test files only; no market word.
ENDPOINT AUTHORITY  source: the primary session's kickoff message for 12a (2026-10-07) and §16.9
  implementation + local validation   authorized (kickoff: "You may implement RS-C1…RS-C8")
  semantic commits, branch push       authorized (kickoff: "commit, push"; "Commit and push after every
                                      small step")
  PR creation / update                authorized; marked READY FOR OPERATOR REVIEW
  scratch base build (RS-2)           authorized (§16.9); done: /tmp/s15-12a/base-mineworld from 6d48e03
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit — NOT this session
VALIDATION BUDGET   unit/integration/static unrestricted; 300-day runs ≤ 4 (RS-C4, RS-C8, M-RS1, one
                    re-run); RS-2's cross-build runs; SIGKILL harness ≤ 4 runs; one full workspace gate on
                    the final head (background); about one hour; real model NOT REQUIRED
STOP CONDITIONS     normal: PR 12a READY FOR OPERATOR REVIEW — DO NOT MERGE. Material: any kernel or
                    contract change; a path outside §16.1; naming physics in the seam; movement naming a
                    resolver; a change to an existing world's facts (either 300-day digest ≠ E-RS0); an
                    existing test failing for a reason other than QR-2's literals; presence's or
                    movement's structural scan needing an edit; an answer to QR-2/QR-4 other than §16.0's
```

## Current checkpoint

RS-C1 (documents) in progress. Before any edit: the base binary for RS-2 was built from 6d48e03 and
copied to `/tmp/s15-12a/base-mineworld`; the base save `B` (`/tmp/s15-12a/B.mwsave`, pristine copy
`B.orig.mwsave`) and base `validate` outputs (`/tmp/s15-12a/base-validate-*.txt`) were written by it.

## Next actions

1. Commit RS-C1 and push.
2. RS-C2: `systems/presence/src/resolve.rs`, `event.rs`, `system.rs` (VERSION 3), `lib.rs`, README,
   `tests/resolver_catalog.rs`; the three QR-2 literals.
3. RS-C3, RS-C4, RS-C5, RS-C6, RS-C7, RS-C8 in order (§16.5).
