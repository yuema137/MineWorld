## DEP-61 — The home's single-writer lock is SQLite's own exclusive locking; `filelock` is declined

**Date** 2026-10-15 · **Status** accepted (PR design frozen 2026-10-15); implemented by S28-a; no dependency is added · **Approved by** the operator's freeze of `pr-s28a-memory-home.md` (2026-10-15); QS28a-4 ruled by the primary · **Relates to** `DEP-37`, `DEP-58`, `ARC-109`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** `.structured-coding/plans/mvp0/pr-s28a-memory-home.md` §4, §5.5

**Problem.** A home has one writer: a second process writing the same home would interleave its transactions with the first's. The second process must be refused at open, by name (`HomeBusy`), on Linux, macOS and Windows, and a process killed while holding the home must leave nothing that refuses the next open.

**Options considered.**

| Candidate | Verdict |
| --- | --- |
| **SQLite's own locking: `PRAGMA locking_mode = EXCLUSIVE`, then `BEGIN EXCLUSIVE; COMMIT` at open, `timeout = 0`** | **Chosen.** Cross-platform through SQLite's VFS (POSIX advisory locks, Windows `LockFileEx`), no dependency, no side file; the operating system releases the lock when the holder dies |
| `filelock` (Unlicense; mature) | Declined: a second lock beside SQLite's own, which can disagree with it, and a dependency for one call |
| A hand-written lock file (`fcntl` / `msvcrt`) | Declined: two code paths, and a file that goes stale after a SIGKILL |
| A side file recording the holder's pid (`HomeBusy(path, holder_pid)`) | Declined (QS28a-4): it can go stale and disagree with SQLite's lock; `HomeBusy(path)` names the file, and the platform's tools find the process |

**Choice.** SQLite's exclusive locking. `HomeBusy(path)` is raised when the lock cannot be taken at once. `auto_vacuum` is set only on an empty database, because on a database with pages the pragma rewrites the header even when the value is unchanged, and a refused file must be left byte-identical (S28-a ledger, F-S28a-10).

**Consequences.** Network file systems, whose locking SQLite documents as unreliable, are out of scope. The lock is tested across processes on all three platforms in CI (`test_home_open.py`), including a holder killed with `os._exit`.

**Revisit** if the `HomeBusy` test fails on a platform; the recorded alternative is `filelock`, by decision.
