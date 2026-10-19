## DEP-57 — The per-seat request bucket: `governor`, driven from the seat table's injected `Instant`

**Date** 2026-10-14 · **Status** accepted; dependency added in S27-a C5 · **Approved by** the step-27 frozen rule (§8, `DEP-S27-b`): adopt `governor` if C0's prototype shows its clock can be driven from the seat table's injected `Instant`, otherwise our own bucket · **Relates to** `ARC-97`, `DEP-14` (option (n)), `REUSE_POLICY.md` §§11–12, 15 · **Design** `pr-s27a-seat-grants-budget.md` §§4, 5.3, 9 C0

**Problem.** `ARC-97`'s per-seat budget needs a token bucket (GCRA or equivalent) whose time is the seat table's injected wall `Instant`, so every transition is testable with literal instants and no sleep (the seat table's existing model for holds), and whose state is keyed by seat inside the table, which iterates deterministically.

**Options considered** (both directions).

| Option | Verdict |
| --- | --- |
| (a) **`governor` 0.10.4 (MIT), one direct limiter per seat** (`RateLimiter::direct_with_clock`), with a clock the seat table sets to its injected `now` before each check | **Chosen** — the prototype passed (below) |
| (b) `governor`'s keyed limiter (`hashmap_with_clock`) | Declined: its keys live in a `HashMap` the seat table cannot iterate deterministically, duplicating the table's own key set |
| (c) Our own integer bucket (~40 lines in `seats.rs`) | Declined since (a) works: GCRA's arithmetic, its rounding and its burst semantics are what a mature crate has already had reviewed; "writing it ourselves feels cleaner" is not a reason |
| (d) `tower_governor` / `tower::limit` | Declined: they limit HTTP requests in the transport, where no per-seat key exists, not WebSocket frames per seat |

**Prototype (C0, 2026-10-14), outside the repository build** (a scratch crate with `governor = { version = "=0.10.4", default-features = false, features = ["std"] }`). Facts verified from the crate source and a test with no sleep:

```text
governor::clock::Reference is implemented for std::time::Instant   src/clock/with_std.rs:23 (feature "std")
Reference's bounds   Sized + Add<Nanos> + Eq + Ord + Copy + Send + Sync + Debug   src/clock.rs:61-62
Clock's bounds       none beyond `type Instant: Reference; fn now(&self)` (and Clone for the limiter);
                     a clock over Rc<Cell<Instant>> — neither Send nor Sync — compiles and works, so a
                     clock owned by the world thread is fine
test                 Quota::per_second(10).allow_burst(20): 20 checks pass at t0; the 21st fails with
                     NotUntil::wait_time_from(t0) == 100 ms exactly; at t0 + 100 ms − 1 ns still
                     refused; at t0 + 100 ms one passes and only one                    PASS
licence              crate licence field "MIT"; repository boinkor-net/governor, MIT License
```

New crates in `Cargo.lock` (normal edges, `default-features = false, features = ["std"]`): `governor`, `futures-timer`, `lock_api`, `nonzero_ext` (Apache-2.0), `parking_lot`, `parking_lot_core`, `portable-atomic`, `spinning_top`, `web-time` — every one MIT, Apache-2.0, or a dual of them, all in `deny.toml`'s list. (C5 note: adding the dependency to the workspace also put `redox_syscall` (MIT) in `Cargo.lock`, a Redox-only target edge of `parking_lot_core` that no supported platform compiles; the prototype's `cargo tree` for the host target did not show it.) The rest (`futures-util`, `hashbrown` 0.16, `smallvec`, `cfg-if`, `libc`, …) are already compiled.

**Choice.** (a). `governor` is named only in `server/src/seats.rs` (the bucket, behind the seat table's own `spend`), so its types do not cross the crate (`REUSE_POLICY.md`: a thin MineWorld-owned interface).

**Revisit** if `governor`'s clock model stops accepting an injected `Instant`, or if per-address join limiting for public hosting (`DEP-14` option (n)) is adopted, which would reuse this dependency.
