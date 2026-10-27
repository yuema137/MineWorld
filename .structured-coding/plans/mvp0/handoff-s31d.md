# Handoff — PR S31-d (continuation aid; the design's §15 is the authority)

- PR: S31-d, `.structured-coding/plans/mvp0/pr-s31d-reporter-policies.md` (DESIGN FROZEN 2026-10-20).
- Worktree / branch: `/Users/yuema137/mineworld-worktrees/impl-s31d`, `mvp0/pr-s31d`, base `origin/main`
  `0b2faf2f`. The helper worktree `impl-s31d-lm` (C6, C7) is removed; its commits are cherry-picked.
- Endpoint: MERGE-READY (pushed, `fast` green on the exact head, PR number and head SHA reported). Never merge.
- State: C0 … C8 committed. AC-d-15 scan clean (only `systems/economy/tests/controller_wire.rs` under
  `systems/`). Local real-binary runs: every criterion of `price_reporter.rs` passes except AC-d-8 (c)
  (F-S31d-13, needs an operator ruling; recommended: an economy PR that discloses a shop's adjustment).
- Milestone: run `price_reporter.rs`, `price_reporter_lm.rs` (python layer), the template test, the parity
  record and §8.1's mutations.
- For S31-f: Market Town's composition needs `ARC-35` check 3 to admit `notices` and `price-survey` (F-S31d-10);
  `price-reporter` with the topic S31-b fixes; keepers' `keepers:` lines; a second seller of a kind (F-S31d-9);
  and F-S31d-13's ruling before the 30-day convergence checkpoint (AC-S31-13), since Market Town's market-day
  modifier makes keepers' moves overshoot on Saturdays.
