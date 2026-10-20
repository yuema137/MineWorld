## DEP-6 — Scheduler: purpose-built discrete-event queue

**Date** 2026-09-25 · **Status** decided; supersedes the provisional recommendation

**Problem.** Advance a world clock over simulated time, wake processes at scheduled moments,
deliver events to subscribed systems, skip idle time so hundreds of simulated days are cheap
(`AC-11`), and reproduce a run exactly from the same seed (`AC-12`) — while the same process
serves real network clients over tokio and can be saved and resumed mid-run (`AC-6`).

**Options considered.** `DesCartes`, `desim`, `simulacra`, `score`, or a purpose-built queue.

**Choice: purpose-built**, a `BinaryHeap` keyed by `(WorldTime, sequence)` inside the kernel.

**Why not the crates — one decisive reason, the same for most of them.** They model a *process*
as a coroutine: SimPy-style, where in-flight state lives on a suspended call stack. MineWorld
models a `Process` as serializable state owned by a system, with participants, progress and
interruptibility as data ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §10). That difference is not
cosmetic: a suspended coroutine cannot be written to SQLite and resumed in a new process, so
adopting one would forfeit "save, shutdown, restart, continue" for any process in flight —
a frozen MVP requirement, not a nicety.

Per crate, beyond that: `DesCartes` supplies its own deterministic async runtime as a
replacement for tokio, which is precisely the framework lock-in
[`REUSE_POLICY.md`](REUSE_POLICY.md) §3 warns against, and our server needs real tokio for real
sockets. `simulacra` models network message flow, a different domain. `desim` and `score` are
both process-as-coroutine.

**Why ours is small.** A binary heap, a monotonic sequence counter for tie-breaking, and a loop.
The hard part of this scheduler is not the data structure — it is the determinism rules, which
are ours to define either way.

**Isolating interface.** The scheduler is internal to the kernel; systems see scheduled wake-ups
and delivered events, never the queue.

**Accepted limitations.** Single-threaded advancement. Parallel system execution would need a
different design, and determinism currently matters more than throughput at MVP scale.

**Licenses.** All candidates are permissive; none was rejected for licensing.

**Implementation note, 2026-09-30 (S4, PR 06).** The queue is a `BTreeMap` keyed by
`(WorldTime, Sequence)` rather than a `BinaryHeap`: the same key and the same order, but a heap's
layout depends on its insertion history, so two equal queues could serialize to different bytes,
and a queue S5 saves and compares across runs needs a canonical form. The choice — purpose-built,
not a crate — and the reason for it are unchanged. Recorded here so that the code and this decision
do not disagree (`CLAUDE.md` §2.1 rule 4); the evidence is
`.structured-coding/plans/mvp0/step-04-clock-scheduler-process.md` §8 F-9.
