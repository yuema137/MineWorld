# Step 04 / PR 06 — World clock, scheduler, and Process

**Role:** combined step and PR document. S4 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) · **Lifecycle:** `READY FOR OPERATOR REVIEW` —
GitHub [#20](https://github.com/yuema137/MineWorld/pull/20), not merged · implementation context
`CLOSED / AWAITING OPERATOR ACTION` (§12)
**Base:** `main @ 7cf8844` (re-audited 2026-09-30; originally "main after PR 02 and PR 03a/03b merge")
**Branch / worktree:** `mvp0/pr-06-scheduler` in `/Users/yuema137/mineworld-worktrees/s4-scheduler`
**Depends on:** S2's `EventEnvelope` and `Causation`, S3's system registry and dispatch (`BD-7` seam),
S5V's server, which is now a caller of dispatch

Binding: [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §10 ·
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) §5 ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `DEP-6`, `ARC-15`, `ARC-23`

## DESIGN FROZEN

```text
Design revision   §§1–7 as frozen before S5V (scope, invariants, SD-1…SD-7, commit plan, checkpoint,
                  test ownership); §8 re-audit and §9 implementation design added 2026-09-30 as
                  bounded resolutions within that frozen intent
Approved by       the primary session under the operator's autonomous-execution authorization
                  (overall.md §7, 2026-09-25); §7 below is the recorded review
Implementation    main @ 7cf8844, branch mvp0/pr-06-scheduler
base
Execution         §10 of this document
contract
Lifecycle         FROZEN (scope, invariants, acceptance); ledger sections live
```

## Why this is PR 06 and not PR 04

This document was written as "PR 04" before the S5V vertical slice was inserted ahead of it. The
contract-fix PR from the renderer-integration spike took that number on GitHub (GitHub #2, recorded
in `overall.md` §7 as `"PR 04"`), and S5V then used 05a–05d. Two PRs called 04 would make every
cross-reference ambiguous — `pr-04-contract-fixes.md` already exists in this directory — so S4 ships
as **PR 06**, the next unused number. Nothing else about the step changed with its number.

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

Scope as re-audited (§8) adds the integration points that did not exist when this was frozen and
that the change must keep correct: `kernel/src/{dispatch,view,system,world,error,lib}.rs` (the
`BD-7` seam this step drives), one identifier in `contracts/src/ids.rs` (§8 finding F-6), and the
server's call site in `server/src/runtime.rs` (F-4).

## 1.2 Non-goals

```text
persistence of the queue                 → S5 (but the queue must be serializable here)
any concrete process (dinner, travel)    → S6+
calendar semantics: dates, weekdays      → a system, never the kernel (INV-12)
parallel execution                       → out of scope for MVP-0 (DEP-6)
```

Added by the re-audit: replacing the server's wall-time pacing (`HostClock`) — how fast a hosted
world's time passes is a host decision and stays the server's; removing the now-redundant
`deferrals_unscheduled` protocol field — a wire change, left to the next protocol revision (F-4).

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

How each of these is realized against the code that exists today is §9; where the realization
narrows or reinterprets one, §8 records it as a finding.

---

# 3. Commit plan

The four commits below are the frozen plan. Each carries the frozen line items and, under them,
the file-level checklist the re-audit (§8) and the implementation design (§9) produced. Evidence is
recorded in §11 as each item completes.

## C0 — Re-audit, PR rename, execution contract (docs only)
- [x] Implementation: §8 re-audit, §9 implementation design, §10 contract, PR 06 rename; `handoff.md` reinitialized for PR 06.
- [x] Validation: `python3 scripts/check_decision_ids.py`, `python3 scripts/check_doc_headings.py` — see §11 E-0.
- [x] Review: every finding classified bounded or material with source evidence; no material finding (§8.3).

## C1 — Clock and queue
- [x] Implementation: `WorldClock` over S1's `WorldTime`; the `(time, sequence)` heap; `schedule_at`, `schedule_after`, `next_instant`, `drain_instant`.
  - [x] `kernel/src/clock.rs`: `WorldClock { now, started }` — `now()`, `has_started()`, crate-private `check`/`advance_to` refusing to move backwards (`ClockWouldMoveBackwards`) once started, `restore`. Serializable. The clock starts at the first instant a world is given (genesis, restore, dispatch or advance) rather than at a fixed epoch, so a world may begin wherever its assembler states (F-5).
  - [x] `kernel/src/schedule.rs`: `Schedule` — `BTreeMap<(WorldTime, Sequence), Scheduled>` (§9 ID-2, F-9), `insert`, `next_instant`, `pop_at`, `len`, `entries`, validated `restore`; `Sequence`, `Scheduled::Fact(Deferral)` (the `Wake` variant arrives with processes in C3), `ScheduledEntry`, `ScheduleSnapshot { now, next_sequence, entries, next_event }`. Design-name mapping: `schedule_at` = `insert` (reached only through `WorldView::defer` and, in C3, process starts); `schedule_after` is not a separate method — a system computes `at` from `WorldView::at()`; `next_instant` = `next_instant`; `drain_instant` = `pop_at` looped by `World::fire_instant`.
  - [x] `kernel/src/system.rs`: `Deferral { at, emission, emitter, caused_by, decision }` with accessors and crate-private `Cause`; `Emission` and `Deferral` serialize; `Emission::record()` accessor added.
  - [x] `kernel/src/view.rs`: `WorldParts` carries the call's `Cause` and a `pending` list instead of a deferral vector; `defer` captures writer and cause.
  - [x] `kernel/src/world.rs`: `World { clock, schedule, ran }`; `now`, `next_instant`, `scheduled`, `schedule_snapshot`, `restore_schedule`; crate-private `WorldSplit`, `check_dispatch_instant`, `run_at`, `begin_at`, `has_run`.
  - [x] `kernel/src/dispatch.rs`: `dispatch` checks the instant (backwards / `ScheduledWorkDue`), moves the clock, and files pending entries after each system call (`file_pending`); `genesis` sets the clock; `Dispatcher` is shared by request, genesis and scheduled work; `EventIds::{next, restore}`.
  - [x] `kernel/src/advance.rs` (pulled forward from C2, because "drain" belongs with the queue): `World::advance_to`, `World::step`, `Advanced { events, instants, skipped }`; each entry fired through `Dispatcher::fire_fact` → the existing `record` + `reduce`.
  - [x] `kernel/src/error.rs`: `ClockWouldMoveBackwards`, `ScheduledWorkDue`, `ScheduleSequenceExhausted`, `RestoreAfterTheWorldHasRun`, `PersistedEntryBeforeNow`, `PersistedSequenceOutsideCounter`, `PersistedSequenceRepeated`, `PersistedSequenceCounterTooLow`, `PersistedEventCounterTooLow`, `PersistedEntryNamesUninstalledSystem`.
- [x] Validation: items at the same time drain in insertion order; advancement skips empty spans; scheduling into the past is a named error rather than silent reordering; the queue serializes and restores with pending work intact; a 10,000-item queue drains in the right order (determinism at scale, not performance theatre). — §11 E-1.
- [x] Review: no calendar concept anywhere; no wall-clock time; nothing non-deterministic in ordering. — §11 E-1 review.

## C2 — Logical instants and the cascade limit
- [x] Implementation: instant execution — drain, deliver to subscribed systems in registration order, collect emitted events, reduce them synchronously within the instant, re-queue deferred work; depth counter with the limit from SD-4.
  - [x] `kernel/src/dispatch.rs`: cascade error names the systems still emitting in the second half of the budget (§8 F-13); `kernel/src/error.rs` doc updated.
  - [x] `World::advance_to(until) → Advanced` and `World::step()`; each due entry fired through the **existing** `Dispatcher::record` + `reduce` (no second cascade implementation, §8 F-1); a fired fact carries the causation captured at deferral; a deferral whose emitter is disabled at its instant is skipped and counted. — landed in C1 (`kernel/src/advance.rs`); see C1.
  - [x] `server/src/runtime.rs`: advance the world to the host's instant before dispatching and on each sweep, so queued work fires and reaches observers (§8 F-4); `deferrals_unscheduled` stays on the wire, documented as always zero since S4. — `WorldRuntime::{advance, tick}`; `submit` advances first and refuses with `DispatchFailed` on a fault; `Command::Sweep` → `tick`; the `unscheduled` counter removed; `HostClock` re-documented as pacing; `server/src/{host,protocol}.rs` and `server/PROTOCOL.md` docs updated.
  - [x] `server/tests/support/mod.rs`: `Chatter` provides `remind`, which defers a public `spoke` fact; `remind_request` helper.
- [x] Validation: an event emitted during reduction is reduced in the same instant; a deferred one lands strictly later; a deliberate two-system cycle hits the limit and the error names both; the same scenario replays identically twice. — §11 E-2.
- [x] Review: no path lets a reduction reorder already-queued work; the deterministic order is documented where a reader will find it. — §11 E-2 review.

## C3 — Process store and interruption
- [x] Implementation: `ProcessStore` with ownership-gated writes reusing S3's token; lifecycle start/progress/end; `request_interrupt` routed to the owner; the outcome returned to the requester.
  - [x] `contracts/src/ids.rs` + `error.rs` + `lib.rs`: `ProcessTypeId` and `IdentifierKind::ProcessTypeId` (§8 F-6); `contracts/tests/identity.rs` extended so the kind-reporting test covers it.
  - [x] `kernel/src/process.rs`: `ProcessKind`, `Process`, `ProcessPhase`, `Interruptibility`, `ProcessStart<P>`, `ProcessStore` (open reads; crate-private writes, validated `restore`), `InterruptRequest`, `InterruptOutcome { Ended, Suspended, Refused, Uninterruptible, NotRunning, OwnerDisabled }`.
  - [x] `kernel/src/view.rs`: `WorldRead::{process, processes}`; `WorldView<S>::{start_process, end_process, suspend_process, reschedule_process, set_process_state, request_interrupt}`. Mapping: the planned `resume_process` is `reschedule_process`, which sets a new end and resumes — resuming without stating when the process now ends would leave it unwoken. `WorldParts` gains the process store, the registry (read-only), a foreign-emission buffer and a nesting depth; `nested()` reborrows it for an owner's decision.
  - [x] `kernel/src/system.rs`: `System::wake` (default refuses with `ProcessNotWokenBySystem`) and `System::interrupt` (default leaves the process running); `DynSystem` counterparts.
  - [x] `kernel/src/dispatch.rs`: `Dispatcher::call` (one system call → file pending → record foreign facts then own), `fire_wake`, `Firing { Fired, Stale, Skipped }`; `kernel/src/advance.rs` fires `Scheduled::Wake`; `kernel/src/schedule.rs` `Scheduled::Fact(Box<Deferral>) | Wake(ProcessId)` (boxed per clippy's variant-size lint); `ScheduleSnapshot` gains `next_process`, `processes`.
  - [x] `kernel/src/world.rs`: `processes` field, `World::processes()`, snapshot/restore include processes and check their owners are installed.
  - [x] `kernel/src/error.rs`: `ProcessEndNotInTheFuture`, `ProcessIdSpaceExhausted`, `ProcessNotRunning`, `ProcessNotOwned`, `ProcessKindMismatch`, `ProcessNotWokenBySystem`, `InterruptionsTooDeep`, `PersistedProcessCounterTooLow`, `PersistedProcessOutsideCounter`, `PersistedProcessRepeated`.
- [x] Validation: an owning system ends its own process; a non-owning system's direct mutation does not compile (or fails per KD-2); a refused interruption leaves the process running and tells the requester; a process surviving a save/restore round trip keeps its scheduled end. — §11 E-3.
- [x] Review: no API lets a non-owner mutate; the dinner-and-phone-call scenario from `CORE_CONCEPTS.md` §10 is expressible with the types as built. — §11 E-3 review.

## C4 — Long-run and documentation
- [x] Validation: a seeded scenario runs **hundreds of simulated days** headless, with wall time and event count recorded, and a second run with the same seed produces an identical event sequence — the first real `AC-11` and `AC-12` evidence. — `kernel/tests/long_run.rs`, §11 E-4.
- [x] `kernel/README.md` updated; ledger closed. — README module table and a paragraph on time; `kernel/src/lib.rs` crate docs gain a "time" table; ledger §11 E-4.
  - [x] `docs/DECISIONS.md` `DEP-6`: dated implementation note for §9 ID-2 (ordered map rather than a heap), so code and decision do not disagree (`CLAUDE.md` §2.1 rule 4).
  - [x] `server/PROTOCOL.md`: `deferrals_unscheduled` documented as zero since S4. — done with the server change in C2.

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

Re-audit addition: the server path. A client's request to a hosted world whose system defers a
fact must see that fact arrive in its observation stream after the deferral's instant, through the
real server loop and a real socket — the end-to-end proof that the `deferrals_unscheduled` gap S5V
recorded is closed (§8 F-4).

**Adversarial criteria:** no wall-clock or OS time anywhere in the kernel; no iteration whose
order depends on a hash; no way to mutate another system's process; replay divergence is a
failure, not a tolerance.

# 5. Test ownership

Static: formatting, types, lints. Unit: ordering, tie-breaks, cascade limit, interruption
matrix, serialization round-trip. Real-lifecycle: the long headless run in C4 is the first
genuine lifecycle evidence in this project — it runs the real loop, not a fixture. Real-model:
`NOT REQUIRED`.

Re-audit addition: the server deferral test (§4) is real-lifecycle evidence of the host loop — a
real socket, the real world thread and the real sweep cadence. CI: no workflow exists in this
repository yet (S13), so the local gates of §6 are the terminal evidence and are run once on the
final head.

# 6. Verification

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check && cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/check_decision_ids.py
python3 scripts/check_doc_headings.py
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

Re-audit review, 2026-09-30:

```text
CHECKED  every divergence between this design and main @ 7cf8844 is listed in §8 with evidence
CHECKED  no finding changes a frozen invariant, an ownership boundary or the step's scope
FLAGGED  F-6 adds one identifier type to contracts/ — additive, implementing a field CORE_CONCEPTS
         §10 already specifies; raised in the PR body for the primary session to confirm
CHECKED  the server change (F-4) is the minimum that keeps an existing caller correct
```

---

# 8. Re-audit against `main @ 7cf8844` (2026-09-30)

The design above was frozen before S5V merged. This section is the audit of that design against the
source as it now stands, done before any code was written for this PR. Each finding is classified
**bounded** (resolved here, within the frozen intent) or **material** (would change a frozen
invariant, a public contract, an ownership boundary or scope — stop and report).

## 8.1 What was inspected

```text
kernel/src/{lib,world,dispatch,system,view,registry,components,access,error}.rs   the BD-7 seam
kernel/tests/{dispatch,genesis,two_systems}.rs                                      dispatch callers
contracts/src/{time,event,ids,error,lib}.rs                  WorldTime, Causation::Process, ProcessId
server/src/{runtime,host,protocol}.rs, server/PROTOCOL.md    the new caller, its stand-in clock
server/tests/support/mod.rs                                  how the server's test world is assembled
worldpack/src/load.rs, tools/cli/src/main.rs                 genesis at the configured epoch
systems/{conversation,presence}                              whether any shipped system defers
step-03-kernel-and-systems.md  BD-7, KD-1/KD-2, BI-13        what 03b already delivered for S4
docs/DECISIONS.md  DEP-6, ARC-15, ARC-23                     the binding decisions
baseline: cargo test --workspace --no-fail-fast  → 259 passed, 0 failed (main @ 7cf8844, 32 s)
```

## 8.2 Findings

**F-1 — C2's instant semantics already exist (bounded).** `kernel/src/dispatch.rs` implements
synchronous reduction in registration order, `CASCADE_DEPTH_LIMIT = 16`,
`KernelError::ReductionCascadeTooDeep { limit, systems }` naming the systems that emitted while
reducing, and `Deferral` / `WorldView::defer` refusing an instant not strictly later
(`DeferralNotInTheFuture`). PR 03b built them as `BD-7`, "the seam that keeps S4 from re-plumbing
the pipeline", and `kernel/tests/dispatch.rs` already pins the ping/pong cycle. *Resolution:* C2
does not write a second cascade; firing a queued entry goes through the same `Dispatcher::record`
and `reduce` a request does, so dispatch, genesis and scheduled work cannot reduce differently. C2's
validation re-proves the cycle on the scheduled path, because that is the path this PR adds.

**F-2 — a deferred fact has no recorded cause (bounded).** `Deferral` holds `at` and `emission`
only. When the queue fires it the kernel must supply `CausedBy` (`INV-15`, `AC-9`), and the emitter
is not recoverable: `Emission::owner` is the event type's vocabulary owner, which `record`
deliberately does not use as the emitter. *Resolution:* `WorldView::defer` captures the writing
system, the causation of the call it is made in (`Action(id)` in resolve, `Event(id)` in react,
`Process(id)` in wake) and the controller decision, into crate-private fields. The fired fact is
then the deferring system's, caused by what it was handling when it deferred — the truthful link.
`Deferral`'s public shape gains accessors only.

**F-3 — where the clock lives (bounded).** The design names `WorldClock` and the queue but not
their owner. Dispatch is "told the instant" and returns deferrals because S4 did not exist; DEP-6
says the scheduler is internal to the kernel and systems never see the queue. A queue outside
`World` would need a public "schedule this deferral" entry point — and `Deferral` is `Clone`, so a
caller could fire one twice. *Resolution:* `World` owns the clock, the schedule and the process
store, as it owns its other state. `World::dispatch(&intent, at)` keeps its signature and gains two
refusals that change nothing — an instant earlier than the clock (`ClockWouldMoveBackwards`) and an
instant at or after which scheduled work is still due (`ScheduledWorkDue`) — and then queues its own
deferrals. `Dispatched::deferred` still reports them, now as *queued*. All existing callers dispatch
at non-decreasing instants (audited: `kernel/tests`, `systems/*/tests`, `server/src/runtime.rs`).

**F-4 — the server is a caller with a stand-in clock (bounded).** `server/src/runtime.rs` keeps a
`HostClock` (wall time from a configured epoch) explicitly as "a provisional stand-in" for S4, and
counts deferrals in `deferrals_unscheduled` because it had nowhere to put them. Under F-3, a server
that dispatched without advancing would hit `ScheduledWorkDue` as soon as any system deferred.
*Resolution:* the minimum that keeps it correct — `submit` advances the world to the host's instant
before dispatching, and each sweep advances it too, so queued work fires in host time and its facts
reach observers through the same `remember` + `sweep` path. Pacing stays the host's (`HostClock`
remains: how fast a hosted world's seconds pass is a deployment decision, not a kernel one).
`deferrals_unscheduled` stays on the wire, reads zero, and is documented as such; removing it is a
protocol change and is left to the next protocol revision. No shipped system defers today
(`grep defer systems/`), so no current behaviour changes.

**F-5 — genesis and "has run" (bounded).** `World::genesis` is refused once the world has
dispatched (`ARC-15`). With a clock, advancing is also running. *Resolution:* genesis sets the
clock to its instant (the world begins then), and the one bit becomes "has run", set by dispatch and
by advancing. Pack loading (`worldpack/src/load.rs`) and the server both genesis at the configured
epoch and dispatch later, so nothing they do changes.

**F-6 — a process's `type` needs an identifier (bounded; flagged).** `CORE_CONCEPTS.md` §10 lists
`type` among a Process's fields, and SD-5 restates it. `contracts/` already carries `ProcessId` and
`Causation::Process` but no name type for a kind of process; the identifier rule and its validation
(`validate_identifier`, `check_identifier`) are crate-private to `contracts/`. *Resolution:* add
`ProcessTypeId` to `contracts/src/ids.rs`, built exactly as `ComponentTypeId` / `EventTypeId` are
(`new`, `from_static`, serde through validation) with an `IdentifierKind::ProcessTypeId` variant. No
existing contract type changes shape. *Why bounded, not material:* it implements a field the
specification already defines, in the crate that owns identifier vocabulary, following an
established pattern; the alternatives were an unvalidated string in the kernel (weaker typing, and a
duplicate of a rule the contract layer owns) or dropping `type` (contradicting §10). Flagged in the
PR body so the primary session can overrule it.

**F-7 — `progress` is not kernel vocabulary (bounded).** §10 lists `state` and `progress`; what
progress *means* — a course of a meal, a distance travelled — is the owning system's (`INV-12`).
*Resolution:* the kernel record carries `state` as owner-encoded bytes labelled with the process
type (the same erasure rule as `EventRecord`: the kernel never interprets or chooses the encoding,
`BI-1`), and progress is part of that state. The record's generic fields are id, type, owner,
participants, place, started, expected end, phase and interruptibility.

**F-8 — KD-2 and process ownership (bounded).** The design allows "does not compile (or fails per
KD-2)". Starting a process names its kind, so `start_process::<P>` requires
`P: ProcessKind<Owner = S>` and a non-owner's attempt does not compile. Acting on an existing
process names an id, which is a value; the typed methods (`end_process::<P>(id)` …) keep the
compile-time gate on the kind and add a run-time check that the stored process is that kind and
belongs to the writer — the same split `RelationStore` has, and for the same reason.

**F-9 — SD-1's container (bounded).** A `BinaryHeap` iterates in an order that depends on its
insertion history, so serializing it directly would make two equal queues produce different bytes,
and a snapshot would have to be sorted anyway. *Resolution:* an ordered map keyed by
`(WorldTime, Sequence)` — the same key, the same order, canonical serialization, and the kernel's
own rule (`BTreeMap`/`BTreeSet`/`Vec` only, `kernel/src/lib.rs` rule 2). `DEP-6` text names a heap,
so it gets a dated implementation note rather than a silent disagreement.

**F-10 — SD-4's budget (bounded).** The existing limit is per reduction chain (per dispatch), not
per instant. *Resolution:* keep it per chain — each request and each fired queue entry gets the
same budget. Termination of an instant is still guaranteed: the set of entries due at an instant is
finite, and nothing can be added *at* the current instant (deferral and process ends are refused
unless strictly later), so an instant consists of finitely many bounded chains.

**F-11 — interruption needs a delivery mechanism (bounded).** SD-6 says the owner decides and the
requester learns the outcome. A system that wants to interrupt is itself running (reacting to the
phone call), holding the world's state mutably through its view. *Resolution:*
`WorldView::request_interrupt` delivers the request synchronously to the owner's `System::interrupt`
through a view built from the same parts and the **owner's** token, then reads the outcome off the
store — `Ended` (gone), `Suspended`, or `Refused` (still running) — so the outcome cannot be
misreported. `Uninterruptible`, `NotRunning` and `OwnerDisabled` are answered without consulting the
owner. Facts the owner emits while deciding are recorded as the owner's, with the requester's
causation, ahead of the requester's own. Nested requests are bounded by the cascade limit.

**F-12 — the integration checkpoint's "load a world" (bounded).** The checkpoint must be a real
path. The only shipped World Pack (`worlds/social-cafe`) composes conversation and presence, and
neither schedules anything, so loading it exercises nothing of S4. *Resolution:* two real paths —
the C4 long-run world assembled the way a pack loader assembles one (install systems, create
entities, genesis, then run), and the server path of §4, through a real socket. Both run the real
loop; neither substitutes a fixture for the scheduler.

**F-13 — the cascade error named a system that was not cycling (bounded; found during C2).** The
first scheduled-path cycle test set the ping/pong loop off with a ring that the chime also answers
once. The error read `[chime, ping, pong]`: 03b's implementation names *every* system that emitted
while reducing, anywhere in the chain. SD-4 says the error names "the systems that were cycling".
*Previous assumption:* naming every reducing emitter equals naming the cycle (true in 03b's test,
where no other system emitted). *Corrected understanding:* it over-names whenever an honest reaction
shares the chain. A first correction — name only the last generation's emitters — was tried and
failed both cycle tests, because a two-system cycle alternates one emitter per generation (recorded
as a failed attempt). *Resolution:* name the systems whose last emission while reducing falls in the
second half of the budget (generation > `CASCADE_DEPTH_LIMIT / 2`). Every member of a cycle of
period ≤ 8 has emitted there; a one-off answer at generation 1 has not. A cycle of period > 8 would
be named partially; recorded as a limitation, since an honest chain is two or three generations
(`BI-13`). The existing dispatch test (`[ping, pong]`, bystander not named) passes unchanged.

## 8.3 Material findings

None. No frozen invariant (§1.3), ownership boundary or scope line changes. F-6 is the only change
outside `kernel/` and the server call site, and is flagged rather than assumed.

---

# 9. Implementation design

**ID-1 — World owns time.** `World { …, clock: WorldClock, schedule: Schedule, processes:
ProcessStore, ran: bool }`. Public: `now()`, `next_instant()`, `scheduled()`, `processes()`,
`advance_to(until)`, `step()`, `schedule_snapshot()`, `restore_schedule(snapshot)`.

**ID-2 — the queue.** `BTreeMap<(WorldTime, Sequence), Scheduled>` and a monotonic `next_sequence`.
`Scheduled` is `Fact(Deferral)` or `Wake(ProcessId)`. Everything a call schedules (deferrals and
process boundaries alike) is collected in one ordered list during the call and inserted after it, so
sequence numbers follow the order things were asked for.

**ID-3 — advancing.** `advance_to(until)`: while the earliest entry is at or before `until`, move
the clock to it and fire every entry at that instant in sequence order, each fully reduced before
the next; then move the clock to `until`. `step()` fires exactly the next instant. Both return
`Advanced { events, instants, skipped }`: the facts in recording order, how many instants ran
(the observable for SD-2), and how many entries were skipped because their system is disabled.

**ID-4 — firing.** A `Fact` records the deferral's emission as its deferring system with the
captured causation and decision, then reduces. A `Wake` is fired only if the process still exists,
is running and still ends at this instant (otherwise it is stale and silently dropped — a rescheduled
or ended process leaves its old wake behind by design); the owner's `wake` runs with
`Causation::Process(id)`.

**ID-5 — processes.** `ProcessKind { const PROCESS_TYPE: ProcessTypeId; type Owner:
SystemIdentity; }`. `Process` is the stored record (F-7). Ending removes the record; history is
the facts the owner emitted. A process is never zero-length: its expected end, if any, must be
strictly later than now (`INV-3` — something that starts and ends in one instant is an event).

**ID-6 — snapshots.** `ScheduleSnapshot { now, next_sequence, entries, next_process, processes,
next_event }`: everything this PR adds to a world, plus the event counter, so that a restored world
continues identity where the saved one stopped. Restore is assembly (refused once a world has run)
and validates: entries not before `now`, sequences strictly increasing and below `next_sequence`,
process ids below `next_process`, every process owner and deferring system installed, event counter
at least 1. Component state is S5's, not this snapshot's.

---

# 10. Execution contract

```text
PROJECT / PR        MVP-0 · Step 04 / PR 06 — World clock, scheduler, and Process
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-04-clock-scheduler-process.md (this file)
RELATED / BINDING   overall.md §§2, 7; CORE_CONCEPTS §§2, 10, 11, 13; ARCHITECTURE §5;
                    DECISIONS DEP-6, ARC-15, ARC-23; ENGINEERING_RULES; ENGINEERING_STANDARDS;
                    step-03 BD-7, KD-1/KD-2
IMPLEMENTATION BASE main @ 7cf8844; branch mvp0/pr-06-scheduler; worktree
                    /Users/yuema137/mineworld-worktrees/s4-scheduler (held by this session only)
APPROVED SCOPE      §1.1 as re-audited; §8 findings F-1…F-12
FROZEN INVARIANTS   §1.3; INV-7 (only owners mutate processes); INV-12 (no calendar);
                    no wall-clock or OS time in the kernel; no hash-ordered iteration;
                    the 259 pre-existing tests stay green, AC-15's in particular
SEQUENCE            C0 → C1 → C2 → C3 → C4, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED;
                    the long run is bounded to well under a minute of wall time
LIVE DOCUMENTATION  this file (§3 checkboxes, §11 ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 06
ENDPOINT AUTHORITY
  implementation + local validation   authorized — operator autonomous authorization
                                      (overall §7, 2026-09-25) and the primary session's brief
                                      of 2026-09-30
  semantic commits                    authorized — same sources; "commit and push after every
                                      coherent step" (brief)
  branch push                         authorized — D-12; brief
  PR creation / update                authorized — D-12; brief ("open a PR with gh pr create")
  CI repair                           N/A — no CI workflow exists in the repository (S13)
  merge                               explicit operator authorization only; the brief repeats
                                      "do not merge"
POST-MERGE SYNC     the brief assigns this session the PR/step document and overall.md §7 updates
                    at review readiness; the merge record itself is the merging session's
NORMAL STOP         PR 06 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to an existing public contract's shape, to ownership, or to
                    scope beyond §8 — stop and report with evidence
```

---

# 11. Ledger and evidence

**E-0 (C0).** Baseline `cargo test --workspace --no-fail-fast` on `main @ 7cf8844`: 259 passed,
0 failed, 32 s wall. `check_decision_ids.py`: 33 ids, all distinct; `check_doc_headings.py`: 134
numbered sections across 21 documents, none duplicated. PASS.

**E-1 (C1).** Working tree on `mvp0/pr-06-scheduler` after C0, before the C1 commit.

```text
cargo fmt --all --check                                         clean
cargo clippy --workspace --all-targets --all-features -D warnings   clean
cargo test --workspace --no-fail-fast                           275 passed, 0 failed (259 + 16)
  kernel/src/schedule/tests.rs   5 unit   same-instant insertion order; earlier instant first;
                                          next_instant skips the span 10 → 1,000,000;
                                          10,000 scattered entries vs an independent stable-sort
                                          oracle; restore round trip and four refusals
  kernel/src/system/tests.rs     1 changed   the deferral now carries the writer's id, the
                                          call's causation and the controller decision
  kernel/tests/schedule.rs      10 integ  deferred fact fires at its instant, caused by the
                                          request, reduced in the same instant (ids continue);
                                          a reaction's deferral lands strictly later naming its
                                          parent; 2 instants for rings 10^4 and 10^7 s apart;
                                          step fires one instant; backwards dispatch / advance
                                          refused unchanged; ScheduledWorkDue at and after the
                                          due instant, then accepted after advancing; genesis
                                          sets the clock; disabled emitter skipped and counted;
                                          save → JSON → restore fires identical envelopes;
                                          restore refused after running / for an uninstalled
                                          system
```

`ARC-23` in the 10,000-entry test: the expected order is a stable sort computed without the
schedule; the test also asserts *where* the data lies — between 401 and 500 distinct instants, with
at least ten entries per instant on average — so the tie-break is exercised rather than vacuous.

Mutation evidence: **NOT OBTAINED.** Two mutations were prepared (`due <= at` → `due < at` in
`check_dispatch_instant`; removing the disabled-emitter check in `fire_fact`) and their run was
denied by the session's permission classifier as test-weakening. Both edits were reverted
immediately and the revert verified by `grep`; no mutated code was committed. The two behaviours are
pinned directly by `a_request_may_not_overtake_work_that_is_due` (dispatch *at* the due instant) and
`a_disabled_system_does_not_fire_what_it_deferred`, but that the tests would fail under those
mutations is argued from reading them, not observed.

Review (C1): `grep` over `kernel/src` for `SystemTime`, `Instant::`, `std::time`, `HashMap`,
`HashSet`, `BinaryHeap`, `rand` — none; no day/hour/weekday/calendar vocabulary in `clock.rs`,
`schedule.rs`, `advance.rs`. Ordering is the `BTreeMap` key `(WorldTime, Sequence)` only; the
sequence counter is the one source of tie-break and is persisted with the schedule. All existing
callers dispatch at non-decreasing instants (audit in §8 F-3), and the 259 pre-existing tests pass
unchanged except the one in-crate test whose bench had to carry the new `Cause`.

**E-2 (C2).** Working tree after the C1 commit `615785e`, before the C2 commit.

```text
cargo fmt --all --check; cargo clippy … -D warnings            clean
cargo test --workspace --no-fail-fast                           279 passed, 0 failed (275 + 4)
  kernel/tests/schedule.rs  +3   a reaction's deferral to t100 fires after the t100 work queued
                                 before it ([10, 8]); a ping/pong cycle set off by a *fired*
                                 deferral errors with limit 16 naming [ping, pong] — not the
                                 bystander, not the chime that answered once; twelve requests with
                                 rings and echoes, run twice, serialize to identical bytes
  server/tests/two_clients.rs +1 a `remind` over a real WebSocket is accepted with no events; the
                                 deferred `spoke` reaches the *other* client within the 4 s bound,
                                 caused by `Action(<the remind's id>)`, at an instant later than
                                 /status reported before the request; afterwards /status reads
                                 deferrals_unscheduled 0, faults 0 (real-lifecycle: real socket,
                                 world thread, sweep cadence; ~1.0 s wall)
  kernel/tests/dispatch.rs   unchanged and passing: [ping, pong] with the bystander not named
  tools/cli/tests/ac15_one_alice.rs  unchanged and passing (AC-15)
```

Failed attempt, kept as evidence (F-13): naming only the last generation's emitters produced
`[ping]` / `[pong]` for the two cycle tests; replaced by the second-half window.

`ARC-23` in the replay test: before comparing bytes the test counts what the history holds — 12
`timer-set`, 18 `rang` (12 requests + 6 echoes for the odd labels), 18 `chimed` — derived from the
inputs the test states, not from the implementation, so equal bytes are not two empty histories.
The cascade test states the limit as the literal 16 rather than trusting the constant.

Review (C2): the only writers of the schedule are `Dispatcher::file_pending` (after each system
call, in asking order) and `World::restore_schedule`; firing removes the earliest entry only
(`pop_at`), and nothing can be filed at the instant being fired (deferral and, in C3, process ends
are refused unless strictly later), so a reduction cannot reorder queued work — pinned by
`a_reaction_cannot_jump_ahead_of_work_already_queued_for_the_same_instant`. The order is
documented in `kernel/src/schedule.rs` (module docs, "the whole of the ordering rule") and
`kernel/src/advance.rs` ("A logical instant"). Server: `advance` precedes every dispatch, so the
kernel's `ScheduledWorkDue` refusal is unreachable from the host in normal operation; a fault in
either is counted in `faults`, never swallowed.

**E-3 (C3).** Working tree after the C2 commit `661aabf`, before the C3 commit.

```text
cargo fmt --all --check; cargo clippy … -D warnings            clean (after boxing the deferral
                                                                variant: clippy::large_enum_variant)
cargo test --workspace --no-fail-fast                           291 passed, 0 failed (279 + 12)
  kernel/tests/process.rs          11 integ  the dinner and the phone call (CORE_CONCEPTS §10):
     owner ends its own dinner at t3600, the fact caused by Process(id), no controller decision;
     dessert → Refused, dinner unchanged (record equal before/after) and still ends at t3600;
     starters → Ended, the dining fact recorded as dining's, caused by Event(phone-rang), BEFORE the
       phone's own fact, and the t3600 wake is visited and ignored (stale, not counted as skipped);
     main → Suspended, not woken at t3600; asked again while suspended → Refused;
     uninterruptible → Uninterruptible although the owner would have ended it (owner not consulted);
     no dinner → NotRunning; dining disabled → OwnerDisabled, and its wake is skipped and counted;
     the phone using its OWN kind on the dinner → Err(ProcessNotOwned{dinner, dining, phone}),
       dinner unchanged;
     zero-length dinner → Err(ProcessEndNotInTheFuture{t50, t50}), nothing stored;
     save mid-main-course → JSON → restore: same record, course Main, end t3600, and the restored
       world's dinner-ended envelope equals the original's;
     a system with no `wake` → Err(ProcessNotWokenBySystem{forgetful, 1});
     two owners requesting each other's interruption forever → Err(InterruptionsTooDeep{limit 16})
  kernel/tests/compile_fail/a_system_cannot_end_or_start_another_systems_process.rs
     both `end_process::<Dinner>` and `start_process(ProcessStart::<Dinner>…)` written in the phone's
     own `react` fail with E0271 "type mismatch resolving <Dinner as ProcessKind>::Owner == Phone";
     the .stderr pins that reason for each call
  contracts/tests/identity.rs      1 extended  ProcessTypeId reports its own IdentifierKind and
                                   deserializes through the rule
  kernel doc-test                  1 new       the ProcessKind example compiles
```

Review (C3), against "no API lets a non-owner mutate":

```text
WorldView<S> process writes   every one requires P: ProcessKind<Owner = S> (compile time) and
                              ProcessStore::owned_mut checks the stored owner and kind (run time)
ProcessStore                  start / owned_mut / end / restore are pub(crate); public: get, iter,
                              len, is_empty
Process                       setters pub(crate); public accessors read-only; state_for::<P>
                              checks the kind before handing bytes back
World                         processes() is &ProcessStore; restore_schedule replaces processes
                              only before the world has run (assembly), checking owners installed
interrupt delivery            the owner's hook gets a WorldView built with the OWNER's token by
                              InstalledSystem — the requester never holds it; the outcome is read
                              off the store, so neither side can misreport it
Deserialize for Process       lets a caller build a record; the only sink for one is
                              restore_schedule, which is assembly
```

The dinner-and-phone-call scenario is not merely expressible — it is `kernel/tests/process.rs`,
with the phone deciding nothing and the dining system deciding everything. File sizes:
`kernel/src/dispatch.rs` 748 lines and `kernel/src/system.rs` 697 are past the 500-line review
trigger and below the 800 warning; both are mostly the module documentation the crate keeps for its
contracts, and splitting the pipeline across files would separate the four entry points that the
module exists to keep on one code path. Recorded rather than split.

**E-4 (C4) — the long run, and the terminal gates.** Working tree after the C3 commit `b48afea`,
with the C4 content; this is the final executable content of the PR (later commits change only
planning documents).

The long run (`kernel/tests/long_run.rs`, real-lifecycle evidence: the real clock, queue, processes,
deferrals and interruptions, driven only by `World::advance_to`). Gate specification, written before
the run: *claim* — 300 simulated days run headless and cheaply (`AC-11`) and replay identically from
one seed (`AC-12`); *evidence* — equal serialized histories for two seed-42 runs, a different history
for seed 43, coverage of every person on every day, and a mid-run save/restore that continues
byte-identically; *counterfactual* — a hash-ordered iteration, a clock read, or a sequence not
persisted would make one of the three comparisons differ.

```text
world      6 people; routine (process owner: one-to-eight-hour activities, refuses interruption
           while asleep) + pager (defers seeded pages minutes later, then requests interruptions);
           assembled as a pack loader would: entities, install, genesis; seed 42
result     300 days, 16,372 facts: 10,858 activity-ended, 2,754 paged, 2,754 page-answered
           (912 refused while asleep, 1,842 ended early); 13,608 instants visited for
           25,920,000 simulated seconds
wall       ~0.10 s first run, ~0.07 s second (debug profile, Apple Silicon host)
replay     seed 42 twice → identical bytes, identical instant counts                   PASS
seed       seed 42 vs 43 → different bytes (the seed is read)                          PASS
coverage   every one of 300 days, every one of 6 people has an ended activity; last
           fact on day 300 (locate-before-counting, ARC-23)                            PASS
bound      10,858 ≥ 300 × (86,400 / 28,800) × 6 = 5,400, the floor the 8-hour maximum
           guarantees — a bound from the stated requirement, not from the code         PASS
idle       13,608 × 100 < 25,920,000: instants are under 1% of simulated seconds       PASS
restore    saved at day 137 with processes and entries in flight, restored into a
           freshly assembled world, continued to day 300: byte-identical to the
           uninterrupted run's facts after day 137 (> 1,000 facts asserted)            PASS
```

Terminal gates on the final content:

```text
cargo fmt --all --check                                          PASS
cargo check --workspace --all-targets                            PASS (rc 0)
cargo clippy --workspace --all-targets --all-features -D warnings PASS (rc 0)
cargo test --workspace --no-fail-fast                            PASS — 294 passed, 0 failed
                                                                 (259 pre-existing + 35), 17 s
  of which tools/cli/tests/ac15_one_alice.rs                     6 passed (AC-15 unchanged)
python3 scripts/check_decision_ids.py                            PASS — 33 ids, all distinct
python3 scripts/check_doc_headings.py                            PASS — 134 sections, none duplicated
cargo doc -p mineworld-kernel                                    9 pre-existing private-link
                                                                 warnings (access, entities,
                                                                 registry, system, world); none in
                                                                 the modules this PR adds
CI                                                               N/A — the repository has no
                                                                 workflow (S13); the local gates
                                                                 above are the terminal evidence
```

Test count by commit: 259 → 275 (C1, +16) → 279 (C2, +4) → 291 (C3, +12) → 294 (C4, +3).

## 11.1 Limitations and follow-ups

```text
L-1  mutation evidence was not obtained (E-1): the session's permission classifier denied running
     deliberately weakened code. The behaviours are pinned by direct tests.
L-2  deferrals_unscheduled remains on the wire at zero (F-4) — remove in the next protocol revision.
L-3  the cascade error names a cycle of period > 8 only partially (F-13).
L-4  a process whose owner is disabled at its expected end is not woken and stays past its end;
     re-enabling the owner does not re-wake it (the wake was skipped and counted). A future
     "re-enable resumes due wakes" rule would be S6+ design.
L-5  ProcessTypeId was added to contracts (F-6) — flagged for the primary session's confirmation.
L-6  the server paces world time at one simulated second per wall second (HostClock); a different
     rate, or pausing, is a host decision for S11/S13.
L-7  no shipped System Pack schedules anything yet; the first is S6 (movement/travel).
```

---

# 12. Closeout — READY FOR OPERATOR REVIEW

```text
PR                    GitHub #20 — https://github.com/yuema137/MineWorld/pull/20 (base main)
base                  main @ 7cf8844 (origin/main unchanged at PR creation)
final executable HEAD ecc5921 — the last commit that changes code or tests; every gate in E-4 ran
                      on this content
final PR HEAD         the commit carrying this section (planning documents only); `git log` on the
                      branch is authoritative — a commit cannot name its own hash
semantic commits      38ab650 docs(plan) re-audit · 615785e C1 clock and schedule ·
                      661aabf C2 instants and server · b48afea C3 processes · ecc5921 C4 long run
CI                    N/A — no workflow in the repository (S13)
working tree          clean after the closeout commit
merge                 NOT authorized; the operator merges
```

**Post-merge, owned by whoever merges (per the brief, this session already updated `overall.md` §7 to
"PR 06 open, awaiting review"):** set this document's lifecycle to `MERGED` with the merge commit,
move S4 to "Done" in `overall.md` §7, and detail S5 (persistence) against the merged kernel —
`World::schedule_snapshot` is the S4 half of what S5 must store, and component state is the other.
