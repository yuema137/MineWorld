# Step 04 / PR 04 — World clock, scheduler, and Process

**Role:** combined step and PR document. S4 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) · **Lifecycle:** `DESIGN FROZEN`
**Base:** `main` after PR 02 and PR 03a/03b merge
**Depends on:** S2's `EventEnvelope` and `Causation`, S3's system registry and dispatch

Binding: [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §10 ·
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) §5 ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `DEP-6`

---

# 1. Goal

Make the world advance. A discrete-event queue drives simulated time, processes run over that
time, and interruption is a *request* the owning system answers — not something another system
performs.

## 1.1 Scope

```text
kernel/src/clock.rs      WorldClock: current WorldTime, advancement, no calendar
kernel/src/schedule.rs   the (WorldTime, sequence) queue, cascade limit, deterministic drain
kernel/src/process.rs    ProcessStore: Process state, lifecycle, interruption requests
kernel/tests/            determinism, ordering, interruption, long-run behaviour
```

## 1.2 Non-goals

```text
persistence of the queue                 → S5 (but the queue must be serializable here)
any concrete process (dinner, travel)    → S6+
calendar semantics: dates, weekdays      → a system, never the kernel (INV-12)
parallel execution                       → out of scope for MVP-0 (DEP-6)
```

## 1.3 Frozen invariants

- `AC-12`: the same seed, inputs and system versions reproduce the same event sequence exactly.
- `AC-11`: hundreds of simulated days run cheaply, which requires skipping idle time rather
  than ticking through it.
- `INV-3`: a `Process` is mutable while it runs; an `Event` is an instantaneous immutable fact.
- `INV-7`: only the owning system mutates a process. Others may request interruption.
- `INV-12`: the clock counts simulated seconds. It does not know what a day, a weekday or
  opening hours are.
- The queue is serializable, because S5 must save and resume a world mid-run with scheduled
  work still pending.

---

# 2. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-1** | The queue is a `BinaryHeap` of `(WorldTime, Sequence, Payload)`, `Sequence` a monotonic `u64` assigned on insertion. Ordering is by time, then sequence. | `DEP-6`. Two items scheduled for the same instant must have one defined order, and insertion order is the only tie-break that is both natural and reproducible. |
| **SD-2** | Advancement jumps to the next scheduled time. There is no empty tick. | `AC-11`. A world where nothing happens between 02:00 and 06:00 should cost nothing to simulate. |
| **SD-3** | Everything scheduled at one `WorldTime` forms a **logical instant**. Within it, event reduction is synchronous and ordered by system registration order; anything deferred is re-queued at a strictly later `(time, sequence)`. | Decided with `D-6`: a wage is paid in the instant it falls due, not a tick later, while all deferral flows through one ordered queue so replay is exact. |
| **SD-4** | A cascade depth limit per logical instant, exceeded → a named error naming the systems that were cycling. | A system emitting events that cause it to emit events is a bug. Hanging silently is the worst possible response; failing with the cycle named is the best available one. |
| **SD-5** | `Process` is stored state — id, type, participants, location, start, expected end, state, progress, interruptibility, owning system — never a coroutine or a task. | `CORE_CONCEPTS.md` §10, and the reason `DEP-6` rejected every SimPy-style crate: a suspended call stack cannot be saved to SQLite and resumed in a new process. |
| **SD-6** | Interruption is `request_interrupt(process, reason)`, delivered to the owning system, which decides: end it, suspend it, or refuse. The requester learns the outcome; it never mutates the process. | `INV-7` and `CORE_CONCEPTS.md` §10's dinner example. A phone call does not end a dinner — `DinnerSystem` does, having been told about the call. |
| **SD-7** | The clock exposes `WorldTime` only. Any notion of a day, an hour, a schedule or opening hours belongs to a system that interprets those seconds. | `INV-12`. The moment the kernel knows what 09:00 means, it has a calendar, and the next request is holidays. |

---

# 3. Commit plan

## C1 — Clock and queue
- [ ] Implementation: `WorldClock` over S1's `WorldTime`; the `(time, sequence)` heap; `schedule_at`, `schedule_after`, `next_instant`, `drain_instant`.
- [ ] Validation: items at the same time drain in insertion order; advancement skips empty spans; scheduling into the past is a named error rather than silent reordering; the queue serializes and restores with pending work intact; a 10,000-item queue drains in the right order (determinism at scale, not performance theatre).
- [ ] Review: no calendar concept anywhere; no wall-clock time; nothing non-deterministic in ordering.

## C2 — Logical instants and the cascade limit
- [ ] Implementation: instant execution — drain, deliver to subscribed systems in registration order, collect emitted events, reduce them synchronously within the instant, re-queue deferred work; depth counter with the limit from SD-4.
- [ ] Validation: an event emitted during reduction is reduced in the same instant; a deferred one lands strictly later; a deliberate two-system cycle hits the limit and the error names both; the same scenario replays identically twice.
- [ ] Review: no path lets a reduction reorder already-queued work; the deterministic order is documented where a reader will find it.

## C3 — Process store and interruption
- [ ] Implementation: `ProcessStore` with ownership-gated writes reusing S3's token; lifecycle start/progress/end; `request_interrupt` routed to the owner; the outcome returned to the requester.
- [ ] Validation: an owning system ends its own process; a non-owning system's direct mutation does not compile (or fails per KD-2); a refused interruption leaves the process running and tells the requester; a process surviving a save/restore round trip keeps its scheduled end.
- [ ] Review: no API lets a non-owner mutate; the dinner-and-phone-call scenario from `CORE_CONCEPTS.md` §10 is expressible with the types as built.

## C4 — Long-run and documentation
- [ ] Validation: a seeded scenario runs **hundreds of simulated days** headless, with wall time and event count recorded, and a second run with the same seed produces an identical event sequence — the first real `AC-11` and `AC-12` evidence.
- [ ] `kernel/README.md` updated; ledger closed.

---

# 4. Integration checkpoint

```text
cargo test --workspace
    ├── ordering and tie-break determinism
    ├── idle spans skipped
    ├── synchronous reduction inside an instant, deferral strictly later
    ├── cascade limit names the cycling systems
    ├── interruption decided by the owner, refusal honoured
    ├── queue and processes survive serialization with pending work
    └── hundreds of simulated days, identical across two seeded runs
```

**Adversarial criteria:** no wall-clock or OS time anywhere in the kernel; no iteration whose
order depends on a hash; no way to mutate another system's process; replay divergence is a
failure, not a tolerance.

# 5. Test ownership

Static: formatting, types, lints. Unit: ordering, tie-breaks, cascade limit, interruption
matrix, serialization round-trip. Real-lifecycle: the long headless run in C4 is the first
genuine lifecycle evidence in this project — it runs the real loop, not a fixture. Real-model:
`NOT REQUIRED`.

# 6. Verification

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check && cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

# 7. Review and approval

```text
CHECKED  AC-11 and AC-12 have mechanisms and a test that would fail if either broke
CHECKED  INV-3, INV-7, INV-12 are structural, not documented intentions
CHECKED  DEP-6's reason for rejecting the crates — serializable processes — is honoured by SD-5
CHECKED  the queue is serializable now, so S5 does not have to retrofit it
CHECKED  scope excludes calendars, concrete processes and parallelism
```

APPROVED for implementation once its dependencies merge.
