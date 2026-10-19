## DEP-59 — The world thread's CPU time per tick: `cpu-time`, not our own `unsafe` over `libc` and `windows-sys`

**Date** 2026-10-15 · **Status** accepted; dependency added in the F-tick-1 fix · **Approved by** the operator, 2026-10-15 ("服务器加测'线程真正干活的时间'" — the server also measures the time the thread actually works), amending D-SB12 · **Relates to** `ARC-42` (note of 2026-10-15), `DEP-29` note, `REUSE_POLICY.md` §§11–12 · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §16.11 (D-SB12, F-tick-1 note)

**Problem.** CP-B4's tick bound needs the CPU time the world thread spends inside one tick, on macOS, Linux and Windows. The standard library has no thread CPU clock. Every platform has one behind a C call: `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` on Unix, `GetThreadTimes(GetCurrentThread(), …)` on Windows, both `unsafe` to call from Rust.

**Options considered** (both directions).

| Option | Code we own | Verdict |
| --- | --- | --- |
| (a) **`cpu-time` 1.0.0** (MIT OR Apache-2.0, tailhook/cpu-time): `ThreadTime::try_now()` / `try_elapsed()`, `clock_gettime` over `libc` on Unix, `GetThreadTimes` over `winapi` 0.3 on Windows | One call site, no `unsafe` | **Chosen** |
| (b) `libc` (Unix) and `windows-sys` 0.61 (Windows) as direct `cfg` dependencies — both already in `Cargo.lock` transitively | Two `cfg`-gated `unsafe` blocks (~30 lines), the `FILETIME` arithmetic, a `windows-sys` feature set (`Win32_System_Threading`, `Win32_Foundation`), and a Windows branch no developer machine here compiles | Declined: the same `unsafe` calls as (a), written and kept by us |
| (c) `libc` on Unix and one `extern "system"` declaration of `GetThreadTimes` from kernel32 (the `DEP-29` note's form) | As (b), less one dependency edge, plus a hand-written FFI signature with pointer arguments | Declined: `DEP-29`'s call took two integers and touched no memory; `GetThreadTimes` writes four structures through pointers, which is exactly what a reviewed binding is for |
| (d) Process CPU time read from outside (`ps`, `/proc`, `GetProcessTimes` in the test harness) | A per-platform parser in the tests | Declined: the whole process, not the world thread; one total, not a per-tick tail |
| (e) `sysinfo`, `nix` | — | Declined: `sysinfo` is a whole-system inspector far beyond one clock; `nix` is Unix-only |

**Facts verified at source** (crate sources in the cargo registry, 2026-10-15): `cpu-time` 1.0.0 is 346 lines (`lib.rs`, `clock_gettime.rs`, `windows.rs`). Its `ThreadTime` is a `Duration` with a `PhantomData<Rc<()>>`, so it is neither `Send` nor `Sync` and cannot be read on one thread and compared on another. `try_now` returns `io::Result` (the panicking `now` is not used). Licence field `MIT/Apache-2.0`, with `LICENSE-MIT` and `LICENSE-APACHE` in the package. New crates in `Cargo.lock`: `cpu-time`, and on Windows targets only `winapi` 0.3.9, `winapi-i686-pc-windows-gnu` 0.4.0 and `winapi-x86_64-pc-windows-gnu` 0.4.0, each `MIT/Apache-2.0`. `cargo deny check licenses sources bans` passes with them. `libc` is already compiled on Unix.

**Choice.** (a). It is the least code we own that is correct on all three platforms, and it has no `unsafe` of ours. `cpu-time` is named only in `server/src/runtime/world.rs` (`TickStart`, `TickTimes`), so its types do not cross the module. If the clock cannot be read, a tick's CPU time is recorded as its wall time, an upper bound.

**Accepted limitations.**
- **Maintenance.** `cpu-time` 1.0.0 and `winapi` 0.3 are not actively developed. The calls they wrap are stable OS interfaces, and the crate is small enough to vendor or replace with (b) in an afternoon if either stops building.
- **Windows resolution.** Windows charges thread time at the scheduler's clock interval (commonly 15.6 ms), not per instruction. A sub-millisecond tick therefore reads as 0 or one interval, and a tick has to burn at least four intervals (about 63 ms) to be sure to read over 50 ms there. The bound keeps its meaning — steady-state ticks measured ≤ 0.6 ms of CPU (step-12 E-SB9) — but the planted CPU mutation is evidence on macOS, not Windows.

**Revisit** if `cpu-time` or `winapi` stops building on a supported target, or if the server comes to need more than one OS clock (then (b) behind a MineWorld module).
