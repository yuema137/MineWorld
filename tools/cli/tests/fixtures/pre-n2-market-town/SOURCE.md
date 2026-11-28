# Market Town as the build before Nightlife N2 left it: where these files come from

Both files were written by the `mineworld` binary built from `origin/main @ bdb83457` — the base of PR N2
(`.structured-coding/plans/mvp0/pr-nightlife.md` §12), before `sleep-until` and `morning_minute` existed — on
2026-11-27, macOS arm64, from `worlds/market-town` at that commit. `tools/cli/tests/sleep_until.rs` reads them.

| File | sha256 | What |
| --- | --- | --- |
| `world.sqlite` | `7c0ef090d23f736c1bdfa872a2eae4af89f8c1f74bd685c3d642dcd02fe11140` (999 424 bytes) | A genesis-only Market Town save (220 facts): its `sleep-interactions-configured` fact and its `sleep` v1 composition are the earlier build's own bytes. N2-I14: the N2 binary refuses it by name as configuration drift |
| `run-seed-7-days-2.txt` | `24788467b103fdd6a6606597a86fcab1db54b634fca791aa8b260d006b162f15` | `mineworld run worlds/market-town --headless --seed 7 --days 2`'s summary without its first line (the save path), its `history` line (a fingerprint of every fact, which the changed configuration fact moves) and its `wall` line. N2-I1b: the N2 binary prints the same lines |

How `world.sqlite` was made:

1. `cargo build -p mineworld-cli --bin mineworld` at `bdb83457`.
2. `mineworld server worlds/market-town --listen 127.0.0.1:7974 --invite presave-token --save <dir> --stop-on-stdin-eof < /dev/null`:
   the server created the save, began the world and stopped at once on the closed standard input (no tick ran).
3. `sqlite3 world.sqlite "PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE; VACUUM;"` so one file holds every
   committed row (a physical compaction; no row changed); the `-shm` and `-wal` files removed.

How `run-seed-7-days-2.txt` was made: `mineworld run worlds/market-town --headless --seed 7 --days 2 --save <dir>` at
`bdb83457`; its stdout with the three lines named above removed.

**Hand edits: none.** A re-recording is a reviewed change: update the hashes and say why.
