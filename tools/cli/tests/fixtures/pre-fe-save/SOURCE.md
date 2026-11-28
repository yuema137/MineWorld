# A save written by the build before FE-a: where it comes from

`world.sqlite` (sha256 `f240111a6f57960724a620690308f3322ac0392980bac4093adef37d98480440`, 110 592 bytes) is the save of a
copy of `tools/cli/tests/worlds/commute-town` with `needs` configured, written by the `mineworld` binary built from
`origin/main @ d252338e` — the last build before PR FE-a (`ARC-370`). It is the real pre-FE-a save that
`tools/cli/tests/food_energy.rs` refuses (design §3.7, C-FE-2): its `needs-interactions-configured` fact and its
`needs` v1 composition are an earlier build's own bytes, not rewritten.

How it was made (2026-11-27, macOS):

1. `git worktree add --detach <dir> d252338e`; `cargo build -p mineworld-cli` there.
2. The world: a copy of `tools/cli/tests/worlds/commute-town` at `d252338e`, its `world.yaml` `configure:` list gaining
   `  - needs` after `  - employment`, and `configure/needs.yaml` written as
   `parameters:\n  - { thresholds: [400, 600, 900] }\n` (`food_energy.rs::pre_fe_world` makes the same copy).
3. `mineworld server <copy> --listen 127.0.0.1:7973 --invite presave-token --save <dir>`, stopped with Ctrl-C once
   it listened: a genesis-only save (39 facts).
4. `sqlite3 world.sqlite "PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE; VACUUM;"` so that one file
   holds every committed row (a physical compaction; no row changed); the `-shm` file removed.

**Hand edits: none.** A re-recording is a reviewed change: update the hash and say why.
