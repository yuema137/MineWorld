# Handoff — S15: PR 12d implementation (the towns get walls), resumed after 12n

A continuation aid, never a design authority. The authority is
[`step-11-bodies.md`](step-11-bodies.md) §19 (DESIGN FROZEN 2026-10-08), with evidence in §19.12
(`E-TD<n>`) and deviations in §19.13 (`TD-D<n>`), and §21.15 for what 12n left 12d. The 12n-2 context
is CLOSED (PR #125 merged, adc46bd); its handoff text is in git history
(`git log -- .structured-coding/plans/mvp0/handoff.md`).

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12d — the towns get walls (S15, fourth of five)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §19; evidence §19.12; deviations §19.13
RELATED / BINDING   §20 (12d-0: §20.5, §20.6.1's instrument); §21 (12n: SD-N11, §21.6, §21.15 M-1 … M-6
                    rulings and CROSS-LANE); §§5, 10.4, 11.1, 17, 18; overall.md ruling 5 and QN-2;
                    step-15 R-12d-1 … R-12d-4; step-13 R-PK-2; DECISIONS ARC-23, 25, 34 … 39, 75, 81,
                    DEP-13, DEP-34; MODULE_SPEC §4.1; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-12d-towns in /Users/yuema137/mineworld-worktrees/impl-12d (sole writer);
                    CARGO_TARGET_DIR=/tmp/impl-12d-target; scratch /tmp/s15-12d (base worktree
                    /tmp/s15-12d/base-adc46bd, binary /tmp/s15-12d/base2-mineworld)
BASE                origin/main @ adc46bd (12n-2 merged), merged into the branch as 3950381
APPROVED SCOPE      §19.1's change set; TD-C1 … TD-C8; SD-D1 … SD-D16 as answered by QD-1 … QD-18;
                    the rulings relayed at resumption (2026-10-12): bodies version 5, TD-12b timed by
                    thread CPU as well as wall clock, geometry-caused stopped strides ≤ 10 % (person-
                    caused printed), the 30-day market-town save ≤ 128 MiB, R 250 with NUDGE_MAX = R and
                    BIAS_BAND = ⌊2R/3⌋, claim 2 restated (M-1)
FROZEN INVARIANTS   §19.9, with the radius amendment (TD-14 re-captured at R 250); TD-12 (social-cafe
                    ≤ 3.96 ×, market-town ≤ 3.30 ×, max resolve ≤ 50 ms) never re-scoped; activity criteria
                    TD-5/TD-6 fixed; no person file changes; both towns' place and object files byte-
                    identical; one re-baseline (QN-2: with 12n-2, one event)
ENDPOINT AUTHORITY  §19.9 and the primary session's kickoff of 2026-10-12
  implementation + local validation   authorized
  semantic commits, branch push       authorized
  PR creation / update                authorized, to READY FOR OPERATOR REVIEW
  CI repair                           authorized; fast, test, test-macos, test-windows, platforms,
                                      python green on the exact head
  merge                               operator only, merge commit; never inherited
TOOL DISCIPLINE     Read/Edit/Write for files; cargo, git, gh (no merge), python3 scripts/*, mkdir -p,
                    sed -n, /usr/bin/time, arch -x86_64; never python3 -c, sed -i, awk, xargs, curl,
                    heredoc writes. Measurements in the FOREGROUND only (N-D27)
VALIDATION BUDGET   TD-12: ≤ 12 TD-12a runs + 1 TD-12b run; six other 300-day town runs; hard cap 19
STOP CONDITIONS     NORMAL: PR 12d READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §19.9's list;
                    any TD-12 failure (INCONCLUSIVE is reported); any digest change other than TD-1; the
                    run cap. Decision ids: ask the primary (next free ARC-84, DEP-47)
POST-MERGE SYNC     planning session: step header, §§1–15, overall, MVP_STATUS Updated/S15 lines;
                    this session: §19 and the evidence rows
```

## Current checkpoint — PAUSED for the operator's milestone acceptance (2026-10-12)

The primary session asked all work to pause while the operator tests. The pause arrived after 03ba860
had been pushed and #171 marked ready, so CI run 38346779547 on the exact head 03ba860 was already
running. Nothing was started after the pause, and the local CI pollers were stopped.

- State at the pause: worktree clean except this handoff note, committed locally and NOT pushed.
- CI on 03ba860 at the pause: fast, platforms (macos-26, windows-2025), python ×3 passed; test,
  test-macos and test-windows pending. A second run, 38346776113 (fired by the ready-for-review event),
  shows two jobs failed in 0 s with unexpanded names ("python (${{ matrix.os }} …)", "platforms (${{
  matrix.os }})"); inspect it on resume. It is probably a matrix that did not expand on that event, not
  a test failure.
- Next on resume:
  1. Push this handoff commit only if the primary asks. It is Markdown only, and pushing it would
     trigger a new CI run, so the exact head would change.
  2. Read run 38346779547's test, test-macos and test-windows results. If they are green, explain
     38346776113's 0-second jobs (or re-run them once). Then record the CI evidence in §19.12 or the
     handoff, and declare READY FOR OPERATOR REVIEW.
  3. If anything is red: diagnose, fix, run `cargo test --workspace --no-fail-fast` and the Python suite
     locally, then push.

## Earlier checkpoint — TD-D14 ruled (a) and applied; full local gate green (E-TD10); PR #171 ready, CI on the exact head

## Earlier checkpoint — MATERIAL STOP TD-D14 (HB-3 at R 250); PR #171 open, CI red on scenarios n2

- CI on c172d21: test and test-macos fail on bodies' scenarios n1/n2 (my TD-D9 run had skipped the
  file). n1 passes once its lengths are multiples of R (§21.6), committed; n2's HB-3 (the walker gets
  past a standing person) fails at R 250 with NUDGE_MAX = R. Recommendation (a): restate HB-3 as M-1
  did claim 2. Waiting for the ruling.

## Earlier checkpoint — TD-C8 done; PR open, CI on the exact head (see the PR)

- TD-D12 ruled (all three as recommended) and applied (34bdaef); E-TD8 (AC-10 test_d passes locally).
- origin/main merged again (e893367: 13c nightly; scripts/CI/docs only).
- E-TD9: TD-12a PASS (1.995 ×, 1.61 ×), TD-12b PASS (max 4.61 ms wall, 1.03 ms CPU), TD-11, TD-10,
  M-TD5, TD-14 re-captured, TD-1 after recorded. Runs: other 4 of 6, TD-12 9 of 13.
- Scratch worktrees /tmp/s15-12d/base-adc46bd and /tmp/s15-12d/timing removed.
- F-12d-1: scripts/baselines.txt stale for the towns (re-record from the nightly after merge).

## Earlier checkpoint — MATERIAL STOP TD-D12 (three items; §19.13, evidence E-TD7)

- Merged origin/main (3950381; TD-D9); straight client walks now `walk-to` (TD-D10); TD-7's nudge
  bound and the M-3 geometry share in town_bodies (TD-D11). TD-5's committed tests pass.
- Stopped on: (1) TD-6 — no throw in 300 days in either town; (2) ASR-2 — market-town's 30-day save
  162.9 MiB > 128 MiB; (3) R-PK-2's required `name` vs entities/modern-goods, worlds/lakeside and three
  fixtures merged after the freeze. Recommendations in TD-D12.
- 300-day town runs: other 4 of 6; TD-12 0 of 13. Scratch: /tmp/s15-12d/nb/{social-cafe,market-town}
  are the SD-D13 copies for TD-12a (validated); /tmp/s15-12d/base2-mineworld is main's binary.

## Next actions (after the rulings)

1. Apply the TD-D12 rulings; re-run town_bodies, market_town, lakeside, milestone_e, packs, third_party.
2. TD-C8: TD-12a (foreground, interleaved, the nb copies), TD-12b (scratch timing build: bodies'
   `resolve` wrapped with `Instant` and `libc::clock_gettime(CLOCK_THREAD_CPUTIME_ID)`), TD-11
   Rosetta, TD-10, TD-14 re-captures at R 250, TD-1 after (2 runs left of 6), docs, PR, CI.
