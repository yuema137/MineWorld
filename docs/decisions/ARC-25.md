## ARC-25 — A world's state is its journal re-executed; its history is the fact log

**Date** 2026-09-30 · **Implements** [`ARCHITECTURE.md`](ARCHITECTURE.md) §§7–8 · **Relates to**
`INV-11`, `INV-15`, [`MVP.md`](MVP.md) §9 `AC-6`, `AC-12`, `AC-15` §9.1, `DEP-1`, `DEP-2`, `ARC-15`,
`ARC-23` · **Design** `.structured-coding/plans/mvp0/step-06-persistence.md` (S5, PR 07)

**Problem.** A persisted world must be rebuilt after its process dies. `ARCHITECTURE.md` §7 described
that as `snapshot + events after it`. Read literally — re-apply the logged facts through the reducers —
it cannot be done with the System contract as merged in S3 and S4, for four independent reasons found
in source:

```text
resolve writes      System::resolve is handed a writable view; a write made there is caused by a
                    request and recorded in no fact
processes           start, end, suspend, reschedule and set-state are writes in any hook, with or
                    without a fact
time                a process wake and a deferred fact fire because the clock reached an instant;
                    nothing in the log says the world advanced
react emits         react, wake and interrupt both write state and return further emissions, so
                    re-applying a fact would emit its logged consequences a second time
```

**Options considered.** (1) Redesign the System contract for pure fact replay: `resolve`, `wake` and
`interrupt` read-only, process lifecycle and time expressed as facts, `react` split into an apply half
and an emit half. (2) Record the *inputs* that drive the deterministic pipeline, re-execute them, and
check the result against the fact log.

**Choice: (2).** A persisted world is three append-only things in one SQLite file, committed together
one revision at a time:

```text
facts       every EventEnvelope in EventId order      HISTORY: what happened (INV-11)
journal     every input that moved the world          what the world was asked, and when
snapshots   the whole of a world's state at some      a checkpoint of the function below,
            revisions                                 never an authority on its own
```

and the two questions have two answers:

```text
What happened?              the fact log. Append-only, never derived, never rewritten. Biographies and
                            projections derive from it (INV-4).
What is the world's state?  the kernel's one pipeline applied to the journal from genesis. A restart
                            loads the newest snapshot at or below the head and re-executes the journal
                            after it; every re-executed input must regenerate its logged answer and its
                            logged facts byte for byte, or the load is refused (ReplayDiverged).
                            Verification re-executes from genesis and requires every stored snapshot to
                            equal the state the history produces at its revision.
```

**Journaled inputs.** Genesis, always — revision 1, recording the assembled world before genesis, the
instant and the genesis facts (`ARC-15`), so genesis is re-run rather than trusted. Every
`dispatch(intent, at)`, whatever the answer — accepted, rejected, unavailable, or a system fault —
because a dispatch moves the clock even when refused and because journaling every request is what keeps
an `ActionId` unique across a restart. An `advance_to(until)` only when it fired at least one instant or
faulted: idle advances only move the clock, every later input carries its own instant, and the kernel's
checks pass identically without them.

**Revision.** A `WorldRevision` is the position of an input in the journal; genesis is 1. It is
committed before anyone is told it, and it is what a client is told a persisted world is at — the
referent of `MVP.md` §9.1's *same persisted state revision*.

**Why (2), and the axis is the pipeline.** Option 1 changes two merged contracts and introduces a
second way to reduce state — exactly the drift `kernel/src/dispatch.rs` was written to prevent, where
two recording paths would be two accounts of one world. Option 2 adds one table and keeps one pipeline.
It is sound only because the pipeline is deterministic, which is not assumed: S4 replays 300 simulated
days byte for byte from one seed. And it is the form recorded cognition needs anyway: an LM controller's
decision reaches a world only as an `ActionIntent`, so journaling intents is what makes an LM-driven run
replayable without the model (`ARCHITECTURE.md` §7).

**Why this is still event sourcing.** The fact log remains the one historical truth and the check on
every reconstruction; no state is accepted that would have produced a different history. Every state
change is still caused by an `ActionIntent`, a `Process` or an `Event` (`INV-15`) — the journal records
the first and the clock that drives the second. State that existed only in memory — a system breaking
`INV-7` through interior mutability, a global or a wall clock — makes re-execution differ, and the first
differing fact refuses the load.

**Versions are refused, never guessed.** A save format, system version, component schema or
composition (the installed systems in registration order, which is the reduction order) that differs
from the running code is refused by name. An event schema changed without a system version change is
still caught, as a byte divergence on replay. Migration is a later step, taken when a pack first needs
one (`MODULE_SPEC.md` §9 rule 2).

**Accepted limitations.** Reconstruction costs re-execution of the journal tail since the last
snapshot, not a load of facts. A restored world's clock is the instant of its last revision: idle
seconds after it are not persisted, and world time does not pass while no process hosts the world.
The log is kept whole; compaction and snapshot pruning are later work.

**Note 2026-10-10 (`ARC-81`).** Snapshot pruning is that later work, done for snapshots only:
`ARC-81` retains genesis, an anchor every 64 scheduled snapshots and the newest two, and stores each
snapshot as a zstd frame of its JSON encoding. Everything above still holds unchanged — the fact log
and the journal are kept whole, a restart restores the newest snapshot at or below the head, and
`verify` re-executes from genesis and compares every snapshot the save still holds. Compaction of the
log remains later work (SR-b), and dropping journal or facts before a checkpoint stays excluded.

**Note 2026-10-13 (`ARC-89`, SR-b, `SAVE_FORMAT` 4).** The log is now compressed, and still whole: the
journal and fact rows of each completed span move, byte for byte, into one sealed block of two zstd
frames, in the transaction of the anchor commit that seals them. Every guarantee above holds unchanged:
no fact or journal row is dropped, rewritten or re-encoded; a restart restores the newest snapshot and
re-executes the journal after it (which is always live, never sealed); replay and `verify` compare every
re-executed entry and fact with the logged bytes, decoded from a block where the row is sealed; and a
block's compressed bytes are a storage representation from which no comparison is argued. The
accepted limitation "the log is kept whole" now reads "kept whole, compressed in sealed blocks".
