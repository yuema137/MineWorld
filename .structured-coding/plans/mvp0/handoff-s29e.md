# Handoff — S29-e (the depot's interior, the platform, the shared station state, the station pieces)

A continuation aid; the authority is [`pr-s29e-depot-interior.md`](pr-s29e-depot-interior.md) (§15, the ledger).

- **Worktree / branch:** `/Users/yuema137/mineworld-worktrees/impl-s29-e`, `mvp0/pr-s29e`, stacked on S29-d
  (`origin/mvp0/pr-s29d`, PR #403, head `6a099d7c`, not merged: a merge freeze holds main). When #403 merges,
  merge `origin/main` into this branch.
- **Shared module:** S29-f's `clients/shared/station/station_state.gd` and its check are adopted byte for byte
  from `origin/mvp0/pr-s29f` (PR #366, open, held for the operator); S29-e adds `station_clock.gd` beside it.
- **State:** see the ledger. The endpoint is MERGE-READY: pushed, `fast` green on the exact head; the
  coordinator merges.
- **Milestone rows (M):** AC-e-3 (connected walk), AC-e-4 (12e geometry), AC-e-5 (X-5 planted copy), AC-e-6
  (perf), AC-e-7 (threshold) — their commands are in the ledger.
