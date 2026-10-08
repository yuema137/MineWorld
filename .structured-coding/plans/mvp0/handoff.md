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

## Current checkpoint — STOPPED (material): AO-2 fails on every rung of SD-O18's ladder

- E-PO-base captured on the base (0fd0be3), before any code: `/tmp/s15-12c/base-mineworld`
  (sha-256 08a2affd…48363), `base-validate-{social-cafe,market-town}.txt`, `base-longrun-line.txt`
  (d7025dbc…79eaf), `base-yard30.txt` (summary sha 3a2c3322…aa9d, = E-PB8).
- PO-C1 … PO-C5 committed and pushed (2fb6d6a, 7cb6d0e, 6849ae8 + f6243db, 92dd1be, b18b465), each
  green with its mutations (E-PO1 … E-PO5).
- PO-C6 partial, committed as a WIP commit: the extended world (c1: 16 objects), the scan's QO-16 edits
  (DO-12), rung p1 (SHOVE_OFFER_REACH), the shove-length fix with its regression (DO-14),
  long_run_objects (PO-13 a: PASS, release 52.8 µs / 109.7 µs). `bodies_yard`'s 30-day test is RED by
  design of the stop: AO-2 fails in a later bucket on every rung (E-PO6, DO-13).
- Not run, pending the decision: PO-9's verdict, PO-10, PO-11, PO-12, PO-14, PO-2/PO-16 binary
  refusals, PO-C7 (PO-1, the gate, the PR).

## Update — rung p3 applied by ruling; STOPPED again (E-PO7, DO-16)

- p3 (unaimed kick/throw toward the room's free centre) implemented, tested (hand literals, the
  wall scenario, M-P3). AO-2 fails on seeds 7, 8 and 9 for want of a push in a later bucket (objects
  gather at the centre and jam against the pillar, the table and each other). Per the ruling: no rung
  added; reported with counts. 3 of the 6 granted 30-day runs used.

## Final — READY FOR OPERATOR REVIEW; context CLOSED / AWAITING OPERATOR ACTION

- The AO-2 question closed by the primary session's final ruling: DO-18 and AO-2′ with (b′); the one
  measurement passed on seeds 7, 8, 9 (E-PO10). The finish list done (E-PO11): PO-2/PO-16 binary,
  PO-10, PO-11 + M-PO9, PO-12 Rosetta, PO-13, PO-14 30.7 s, PO-1, the full gate on 145c7f7 (636
  passed, 0 failed, 1 ignored). WIP commits squashed into PO-C6 and PO-C7 (force-with-lease). The PR
  is open, marked READY FOR OPERATOR REVIEW, merge commit only. Do not merge.
- Post-merge: this session owns §18 (merge identity); the planning session owns the step header,
  §§1–15, overall and MVP_STATUS's Updated/S15 lines; FU-12c-1 carried to planning.

## Next actions (historical — before the final AO-2 ruling)

1. Apply the decision (criteria, content or policy), re-run `bodies_yard thirty_days`.
2. Finish PO-C6: PO-2 and PO-16 refusals in bodies_yard.rs, PO-10 (the copy now strips items' body:),
   bodies_yard_restart.rs's activity line (kicks, throws, shoves, pushes > 0), M-PO2 binary, M-PO9.
3. PO-C7: Rosetta (PO-12), PO-14 (≤ 40 s), PO-1 towns, the full gate once, MVP_STATUS, bodies README
   (systems/bodies/README.md does not exist; SD-O21 lists it), the PR marked READY FOR OPERATOR REVIEW.
