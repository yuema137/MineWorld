# Handoff — S19: PR TW-d implementation (San Diego record data, `tools/weather-fetch`, `source: record`)

A continuation aid, never a design authority. The authority is
[`step-19-time-weather.md`](step-19-time-weather.md) §18 (DESIGN FROZEN 2026-10-09), its ledger §18.11.

```text
PROJECT / PR        MVP-0 · Step 19 / PR TW-d — San Diego record data, tools/weather-fetch, source: record
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-19-time-weather.md §18 (ledger §18.11)
RELATED / BINDING   as §17.10, plus §17 (TW-b, merged #113), step-18 §12 (IL-b: SD-IB-5, QIB-6, D-IB-9),
                    DECISIONS DEP-8, DEP-31 (this PR), ARC-35 (+ notes), ARC-55, ARC-61, ARC-68;
                    REUSE_POLICY; ENGINEERING_STANDARDS; ENGINEERING_RULES; CORE_CONCEPTS
BRANCH / WORKTREE   mvp0/pr-tw-d-data in /Users/yuema137/mineworld-worktrees/impl-tw-d (sole writer)
BASE                origin/main @ 2c6d34c at start; origin/main @ cf18713 merged at the end (70db0b0)
APPROVED SCOPE      §18.1; change set §18.10
FROZEN INVARIANTS   INV-TW-1, -2, -3, -5, -7, -10; SD-TW-d-1 … 11; §18.6 criteria and mutations.
                    Operator (kickoff 2026-10-09): no runtime fetch; downloads only through the tool's
                    fetch path; macOS, Linux, Windows; LF, CRLF and BOM read identically; integers only;
                    worlds without weather data byte-identical
ENDPOINT AUTHORITY  source: the coordinator's kickoff for TW-d (2026-10-09) and §18.10
  implementation, commits, push, PR, CI repair   authorized
  merge                                          operator only; never inherited
SEQUENCE            C1 → C2 → C3 → C4 → C5 → C6 (§18.4) — all done
VALIDATION BUDGET   ≤ 8 town runs (used 8, counting the 200-day leg); ≤ 3 NOAA fetches (used 1);
                    full workspace test ≤ 2 runs (used 1, E-TWd-9)
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §18.10 (1)–(8); none hit
POST-MERGE SYNC     planning (primary) session: step-19 header, overall.md
```

## Current checkpoint — MATERIAL STOP TWd-F4 (deny.toml rejects webpki-roots' CDLA-Permissive-2.0); PR #121 open, NOT READY

- main @ 0ba037f (#99) added `deny.toml`; `fast` fails on the `fetch` feature's `webpki-roots`. `test`
  passed. Options (A) a crate-scoped exception in deny.toml, or (B) drop `fetch`/`ureq` — §18.11 TWd-F4.
  After the ruling: apply it, re-run `cargo deny check licenses sources bans` and the targeted tests,
  push, and wait for `fast` and `test` on the exact head.
- Second merge of origin/main: 376a538 (0ba037f), targeted tests green (acceptance, cli, weather,
  weather-fetch, installed, worldpack; target/tw-d/merge2-tests.log).

## Earlier checkpoint — ready apart from CI

- Commits: 92ad657 C1, c1234d9 C2, 7b3f58c C3, 38ce8c6 C4, 8e4da6e C5, 70db0b0 merge of origin/main
  (cf18713), then the C6 ledger commit. The PR URL, final head and CI runs are in the PR, not here (a
  commit cannot name its own CI).
- Evidence E-TWd-1 … 9, mutations M-TWd-1, -2, -3b, -4, -5, -10, -A1, -A2 (all killed), deviations
  TWd-D1 … D12, findings TWd-F1 … F3 in §18.11.
- New market-town baseline (300 d, seed 7): `d5db8988bb9d8c33ec8e1cf1ba906d58d1fbd49d2a4ad7bc2d2a69b0b0a922ee`,
  375 619 facts. Rules worlds unchanged (E-TWd-5: TW-b's `90479fd8…ae57` reproduced before the switch).
- For the operator: TWd-F2 — the record world's Process ids shift by one more (the TWb-F1 class); the
  record re-check passes in TWb-F1's proposed form, which TW-b merged with but whose ruling is not
  recorded on main.

## Notes

- Large untracked artefacts under target/tw-d/: noaa/USW00023188.dly (the fetched file, 4 300 290 B),
  rules-save, record-save (2.7 GB each), cp/U, cp/R (~3.3 GB each); logs *.out, gate.log, gate-test.log,
  criterion4.txt, reshape-report*.txt.
