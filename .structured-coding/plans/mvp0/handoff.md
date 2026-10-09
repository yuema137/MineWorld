# Handoff — S15: PR 12d implementation (the town gets bodies)

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §19 (DESIGN FROZEN 2026-10-08), with evidence in §19.12
(`E-TD<n>`) and deviations in §19.13. The 12d-0 context is CLOSED (merged as #84); its handoff text is
in git history at `77a8717`.

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12d — the town gets bodies (S15, fourth of five)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §19; evidence §19.12; deviations §19.13
RELATED / BINDING   §20 (12d-0 as merged: §20.5, §20.6.1's instrument, §20.14's bases); §§5, 10.4, 11.1,
                    17, 18; overall.md ruling 5; step-15 R-12d-1 … R-12d-4; step-13 R-PK-2; DECISIONS
                    ARC-23, ARC-25, ARC-34 … ARC-39, DEP-13; MODULE_SPEC §4.1; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12d-towns in /Users/yuema137/mineworld-worktrees/impl-12d (sole writer)
BASE                main @ 77a8717 (#85 merged: the frozen §19; includes 12d-0 #84, IL-a #80, 2D 13a #82,
                    CI #63)
APPROVED SCOPE      §19.1's change set; TD-C1 … TD-C8; SD-D1 … SD-D16 as answered by QD-1 … QD-18
FROZEN INVARIANTS   §19.9: no edit outside §19.1's change set; bodies' rules unchanged (bodies-yard 30-day
                    sha bd6a1002…80e6, long_run 23f7fa76…5125, long_run_objects c8358f8b…c5b4); TD-12
                    (social-cafe ≤ 3.96 ×, market-town ≤ 3.30 ×, max resolve ≤ 50 ms) never re-scoped;
                    activity criteria TD-5/TD-6 fixed, remedies only §19.4's content ladder; no person
                    file changes; both towns' place and object files byte-identical; one re-baseline
ENDPOINT AUTHORITY  source: §19.9 (the freeze) and the primary session's kickoff (2026-10-08)
  implementation + local validation   authorized
  semantic commits, branch push       authorized ("commit and push after each step")
  PR creation / update                authorized, READY FOR OPERATOR REVIEW
  scratch builds                      /tmp/s15-12d (base binary, x86_64, TD-12b timing copy)
  CI repair                           authorized; `fast` and `test` green on the exact head
  merge                               operator only, merge commit; never inherited
TOOL DISCIPLINE     Read/Edit/Write for files; cargo, git, gh (no merge), python3 scripts/*, mkdir -p,
                    sed -n, /usr/bin/time, arch -x86_64, target/*/mineworld, godot; never python3 -c,
                    sed -i, awk, xargs, curl, cat >> / heredoc writes; long jobs in the background
VALIDATION BUDGET   (§19.13 amendment) TD-12: ≤ 12 TD-12a runs + 1 TD-12b run; six other 300-day town
                    runs (base 2 used); hard cap 19 — reaching it is a material stop
STOP CONDITIONS     NORMAL: PR 12d READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §19.9's list;
                    any TD-12 failure (INCONCLUSIVE is reported); any digest change other than TD-1; the
                    run cap
POST-MERGE SYNC     planning session: step header, §§1–15, overall, MVP_STATUS Updated/S15 lines;
                    this session: §19 and the evidence rows
```

## Current checkpoint — PAUSED: 12d waits for 12n (navigation), by the operator's ruling on TD-D7

The ruling (2026-10-08, §19.13): "Add pathfinding first. 12d waits for it." 12n, designed by a separate
planning session, adds server-side route planning. **12d resumes on top of 12n.** No PR is open.

**Where 12d stands (branch `mvp0/pr-12d-towns`, WIP head pushed, tests NOT green):**
- Done: TD-C1 (notes, MODULE_SPEC, E-TD-base, E-TD1), TD-C2 (item names, the catalogue), TD-C3 (the
  doorway refusal, bodies v4), TD-C4 (AC-1 check 3; M-TD2 … M-TD4 run on the real towns, TD-D5), TD-C6
  (FU-12a-1).
- **TD-C5 is complete:** both towns' bodies (62 street solids; table north's south face at y 4 160,
  37 mm inset, guarded by `town_bodies.rs`; the bank tree omitted, FU-12d-1), the four objects, the
  doorways, §19.6's literal edits, `ac15_one_alice.rs` re-routed round table north (6/6).
- The radius ruling: `PERSON_RADIUS` 250, `CLEARANCE` derived; bodies' tests re-derived (E-TD5);
  TD-D8: two claims do not hold at 250 with derived geometry and are left failing for the operator —
  `actions::a_stride_toward_a_person_within_the_offset_is_still_stopped` and
  `scenarios::n3_a_crowd_is_nudged_in_bounded_chains_and_sometimes_blocks`.
- `town_bodies.rs`: validate + refusals, the geometry exception, the wall check from carol's home
  (PASS, E-TD4) and the catalogue pass; the 300-day town tests and the counterfactual are written, not
  yet run.
- Evidence: §19.12 E-TD-base … E-TD6 (TD-5 diagnostics, the ladder table, the cost observation);
  deviations §19.13 TD-D1 … TD-D8. Runs used: 300-day 2 of 6 other, 0 of 13 TD-12.

## Next actions (when 12n has merged)

1. Merge (or rebase onto) main with 12n; re-run `cargo test -p mineworld-bodies` and the CLI tests TD-C5
   touched; re-check the TD-D8 tests (and the operator's answer on them).
2. **Next edit: `tools/cli/tests/milestone_c.rs` `walk_into_the_cafe`** — its straight walk meets a street
   prop (`TooFarAway`); §19.6's waypoint (or 12n's route, if it gives the test one).
3. **Re-run TD-5** on 12n: `routines.rs`, `run.rs`, `run_restart.rs`, `market_town.rs`, `milestone_b.rs`,
   `milestone_c.rs`, `market_composition.rs`; then TD-C7's `town_bodies.rs` 300-day tests (TD-6, TD-7)
   and the counterfactual (TD-8).
4. **Re-run TD-12** on 12n under §20.6.1 (counterfactual copies: rebuild from the final towns; the
   scratch ones in /tmp/s15-12d/nobodies match this WIP head).
5. TD-C8: TD-10, TD-11, TD-14 (re-capture bodies-yard and the long-run bytes at r = 250 as new bases),
   TD-1's re-baseline, the gate, the 2D `client_2d` Godot check (TD-D2), PR.
