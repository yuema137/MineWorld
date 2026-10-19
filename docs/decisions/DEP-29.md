## DEP-29 — Test scratch: a `std`-only helper of our own, not the `tempfile` crate

**Date** 2026-10-08 · **Status** selected; no dependency added · **Approved by** the primary session at
the freeze of PR test-hygiene (its QTH-3, which asked for this record) · **Relates to**
`ENGINEERING_STANDARDS.md` §22 "Test scratch" · **Design**
`.structured-coding/plans/mvp0/pr-test-hygiene.md` §§2–4

**Problem.** A passing `cargo test --workspace` left 135 entries, about 16 GB, under `target/tmp`:
helpers removed a test's directory *before* the test and never after. Every test must remove its own
scratch. The audit fixed what the mechanism must do:
- the scratch path's last component is exactly the name the test gives, because a World Pack's id is
  its directory's name and refusal messages name it;
- some tests need a path that does **not** exist yet (the CLI creates the save), others an existing
  empty directory;
- two tests of one process must never share a scratch (12c's "database is locked"), and two processes
  on one `target/` must not collide;
- saves an operator wants to keep (`BODIES_YARD_SAVES`) must be keepable, side by side in one
  directory.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) `tempfile` 3.27 (`TempDir`, removed on drop; MIT OR Apache-2.0; mature). Not in Cargo.lock: it
    would add tempfile, fastrand and rustix (+ linux-raw-sys on Linux)
(b) a std-only test-support crate of our own (`mineworld-test-support`, publish = false)
(c) a remove_dir_all at the end of every test
```

**Choice: (b).** No dependency is added.

**Why not the others** (`REUSE_POLICY.md` §12's reasons):

- **(a) `tempfile` — missing required semantics, so a wrapper of the whole helper's size anyway.**
  - Its names carry a random suffix. The leaf name must be exact, so the test's directory would be a
    child of the `TempDir`, which already is (b)'s layout.
  - Random containers scatter kept saves, so `BODIES_YARD_SAVES` could no longer point at one
    directory.
  - A same-name clash between two tests is hidden by random names rather than reported; (b) panics
    and names it, which turns a locked database into a clear failure.
  - Keeping a scratch on failure needs `std::thread::panicking()` in our own `Drop` anyway.
  - What it does well — secure, race-free creation in a shared world-writable directory — is not
    needed for a test's scratch under the build's own `target/`.
- **(c) cleanup in each test — the failure mode that caused the problem.** It does not run on an early
  `return` or a panic, nothing checks it, and about sixty sites would each have to remember it.

**Isolating interface.** `mineworld-test-support`: `Scratch` and the `scratch!` macro. Tests name
only those; if the helper is ever replaced (by `tempfile` or otherwise), only that crate changes.

**Accepted limitations and the revisit trigger.**
- A process killed with `SIGKILL` (or a test with an infinite loop that is killed) leaves its
  `mineworld-scratch-<pid>` directory; it is under `target/`, so `cargo clean` removes it, and
  `check_scratch.py left` reports it.
- Revisit if test scratch must live outside `target/` in a shared, world-writable directory, where
  `tempfile`'s secure creation matters.

---

## DEP-29 note — ending a test's child process on every platform (2026-10-09, S13 PR 13w)

**Approved by** the primary session at 13w's design freeze (step-14 §15, QW-2) · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §15.3–15.4

The decision is unchanged: `mineworld-test-support` stays standard-library-only. It gains a module
`process` so that the default suite compiles and passes on Windows as on Unix (the operator's
all-platforms requirement, step-14 §13.0.1):

- **`kill(&mut Child) -> Killed`** and **`Killed::killed()`**: whether the kill ended the child. On Unix
  the status is signal 9 (`ExitStatusExt`, now used in this crate only). On Windows std's `Child::kill`
  is `TerminateProcess(handle, 1)`, so the status is exit code 1, and the child must have been running
  (`try_wait` is `None`) immediately before the kill. Tests assert the verdict, never a raw status.
- **`interruptible(Command)`, `InterruptibleChild` and `interrupt(&mut InterruptibleChild)`**: an
  operator's stop. On Unix, `SIGINT` through `sh -c "kill -INT <pid>"` (moved unchanged from the CLI's
  test support), then a wait. On Windows the child is spawned with `CREATE_NEW_PROCESS_GROUP` (std's
  safe `CommandExt::creation_flags`), and `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, <pid>)` reaches
  that group and nothing else; Ctrl-C cannot be sent to one group. `interrupt` accepts only an
  `InterruptibleChild`, so a Ctrl-Break can never be broadcast to every process on the console (a
  compile-fail doctest pins it).

**Options considered for the one Windows call** (`REUSE_POLICY.md` §§11–12, both directions):

```text
(a) one `unsafe extern "system"` declaration of GenerateConsoleCtrlEvent from kernel32, which std
    already links on every Windows target
(b) `windows-sys` 0.61.2 (already in Cargo.lock, transitively) as a cfg(windows) dependency
(c) libc / nix on Unix in place of `sh -c kill`
```

**Choice: (a), and `sh -c kill` on Unix.**
- (b) is a real dependency (`REUSE_POLICY.md` §11) for one function, and its functions are `unsafe` to
  call as well; it buys no safety over (a).
- (c) adds a dependency and `unsafe` to replace a command that every Unix CI image and developer machine
  already has, and that 13a's container proves in CI.
- The crate's `#![forbid(unsafe_code)]` becomes `#![deny(unsafe_code)]`, with exactly one
  `#[allow(unsafe_code)]` function, `send_interrupt` under `cfg(windows)`. The call takes two integers by
  value and touches no memory.

**Accepted limitations.**
- **The residual race on Windows.** A child that exits with code 1 by itself in the instant between
  `try_wait` and `TerminateProcess` would read as killed. Every killing test also checks separately that
  the child had not finished (no final summary line, or a revision short of the end), so a false
  "killed" cannot also pass that check. On Unix the verdict is unambiguous.
- **A console is needed on Windows.** `GenerateConsoleCtrlEvent` works only for a caller attached to a
  console. Without one the call fails, and `interrupt` returns a named error ("no console attached;
  Ctrl-Break cannot be delivered"); it never passes silently.
- The server must handle Ctrl-Break itself: `tokio::signal::ctrl_c()` does not catch it on Windows
  (step-14 F-13w-2). `serve.rs`'s `stop_requested` does, since S11-D (step-12 SD-D13, QW-1).

**Revisit** if a test needs to stop a child that is not a console process, or the server gains a stop that
needs no signal (an admin `shutdown` frame); `interrupt` would then wrap that instead.
