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

## Current checkpoint — TD-C1 in progress

- E-TD-base captured on 77a8717 (`/tmp/s15-12d/base-*`, binary `/tmp/s15-12d/target-base/debug/mineworld`):
  towns' digests = E-TD0; validate outputs = E-Z7's; bodies-yard 30-day = bd6a1002…; long_run bytes
  pending.
- E-TD1 (the slice geometry table) delegated to a read-only research agent; its output goes to
  `/tmp/s15-12d/etd1/table.txt`.
- Discovery: `ARC-39` already has a note 4 (IL-a, #80), so 12d's note is **note 5** (§19.13 TD-D1).

## Next actions

1. Finish TD-C1: DECISIONS notes (ARC-35, ARC-37, ARC-39 note 5), MODULE_SPEC §4.1, E-TD-base and E-TD1
   into §19.12; doc checks; commit, push.
2. TD-C2 … TD-C8 as §19.5.
