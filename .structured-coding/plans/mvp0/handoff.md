# Handoff — S15: PR 12c implementation (objects: walking pushes them; kick, throw and shove)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §18 (§18.0's freeze record binds and overrides the rest of
§18), with evidence in §18.10 (`E-PO<n>`) and deviations in §18.11. The 12b context is CLOSED; its
handoff text is in git history at `0fd0be3`.

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12c — objects: walking pushes them; kick, throw and shove (S15,
                    third of five)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §18; evidence §18.10; deviations §18.11
RELATED / BINDING   step-11 header freeze record (QB-1, QB-3, QB-6, QB-8, QB-10, QB-15), §§4.4–4.7, 5,
                    6.3, 7.2, 8.1, 9.5, 10.1, 11.1, §17 (12b as merged, §17.12), §18; overall.md §3
                    (S15), §7; DECISIONS ARC-23, ARC-25, ARC-26, ARC-27, ARC-31, ARC-33, ARC-34, ARC-36,
                    ARC-37, ARC-38, ARC-39 and its notes, DEP-13; MODULE_SPEC §4.1; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12c-objects in /Users/yuema137/mineworld-worktrees/s15-12c (sole writer)
BASE                main @ 0fd0be3 (9c617ed + the 12b post-merge docs #60 and the 12c design #61,
                    Markdown only)
APPROVED SCOPE      §18.1's change set; PO-C1 … PO-C7; SD-O1 … SD-O21 as answered by QO-1 … QO-20
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/, server/, cognition/, clients/,
                    authoring/, sdk/, systems/{presence,movement,item,inventory,installed}/,
                    worldpack/src/, tools/cli/src/, tests/acceptance/, worlds/{social-cafe,market-town}/,
                    nor the root Cargo.toml; no other existing System Pack. PO-1: social-cafe sha
                    ad49c723…c64b (365 330 facts), market-town sha 365b50e0…1d1d (372 755 facts),
                    faults 0. PO-13 b: 12b's long run byte-identical to E-PO-base (sha d7025dbc…79eaf).
                    Existing tests unchanged except QO-16's. Controller unchanged; activity fixed in
                    content or bodies' offers only (AO-1 … AO-3 fixed). bodies → mineworld-item is a
                    crate dependency used ONLY for `is_declared` (ARC-39 note, guard test with a
                    mutation, no system dependency; a world with bodies and without item validates and
                    runs). Rapier named only in rapier.rs; no float outside it; no float persisted.
                    Cost bounds fixed: release ≤ 100 µs per swept move, ≤ 2 ms per kick/throw;
                    300-day dev run ≤ 40 s. Gates unedited and passing: AC-1 13/13, I-2 scan 4/4,
                    seam_vocabulary; no diff under cognition/.
ENDPOINT AUTHORITY  source: the primary session's kickoff message for 12c (2026-10-08) and §18.9
  implementation + local validation   authorized (kickoff: "You may implement PO-C1 to PO-C7")
  semantic commits, branch push       authorized (kickoff: "commit, push"; "Commit and push after
                                      every small step")
  PR creation / update                authorized; open against main marked READY FOR OPERATOR REVIEW
  scratch builds                      authorized: base binary, x86_64-apple-darwin build, under
                                      /tmp/s15-12c; `rustup target list --installed`, `arch -x86_64`
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited
TOOL DISCIPLINE     Read/Edit/Write for file changes; never python3 -c, sed -i, awk, xargs, curl,
                    cat >> / heredoc writes. Long runs (> ~2 min) in the background.
VALIDATION BUDGET   §18.9: 300-day runs ≤ 6; 30-day yard runs inside tests, ≤ 3 re-runs for SD-O18's
                    ladder; x86_64 build once + 3 Rosetta runs; one full gate on the final head.
STOP CONDITIONS     NORMAL: PR 12c READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §18.9's list, and
                    the kickoff's (kernel/contract/controller change; town digests or 12b long-run base
                    changing; a path outside §18.1; anything from mineworld_item beyond is_declared; a
                    cost or activity bound failing; Rapier outside rapier.rs; a persisted float).
```

## Current checkpoint

- E-PO-base captured on the base (0fd0be3), before any code: `/tmp/s15-12c/base-mineworld`
  (sha-256 08a2affd…48363), `base-validate-{social-cafe,market-town}.txt`, `base-longrun-line.txt`
  (d7025dbc…79eaf), `base-yard30.txt` (summary sha 3a2c3322…aa9d, = E-PB8).
- PO-C1 (docs) in progress.

## Next actions

1. PO-C1: ARC-36, ARC-39 (note 2), DEP-13 notes; MODULE_SPEC §4.1; ledger E-PO1; commit, push.
2. PO-C2 … PO-C7 in order, each committed and pushed.
