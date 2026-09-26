# kernel/

MineWorld's authoritative world state: **`mineworld-kernel`**, the Rust crate that holds who
exists, what state they carry, how they are connected, and which systems are allowed to change any
of it.

What exists today:

```text
entities     EntityRegistry     identity, allocated from a counter and never reused
components   ComponentStore     one table per component type, writes gated on ownership
relations    RelationStore      typed edges, written by the system that declared the type
access       SystemIdentity, OwnedBy<S>, WriteToken<S>, WriteAccess, owned_component!
system       System: what a system declares, and the four things it is asked to do
view         what a running system is handed — reads open, writes gated on ownership
registry     which systems a world is composed of, and which of them are enabled
dispatch     ActionIntent → route → validate → resolve → Event(s) → reduce
world        World: the composed whole, and the only issuer of write capability in it
errors       KernelError
```

Still to come, in this order: the world clock and scheduler, then persistence and the event log.

## The two ideas to understand first

**A component is written only by the system that owns it**, and that is the compiler's rule here,
not a review convention:

```rust
world.insert(entity, MyComponent { .. })?;   // fine, inside my own system
world.insert(entity, YourComponent { .. })?; // does not compile
```

A write needs the owning system's `WriteToken`; tokens are never constructed, only granted, and the
one act that grants one is installing a system into a `World`. A running system is handed a
`WorldView`, never the store itself. Reads are open on purpose: any system may read any component,
and a component whose owning system is not installed reads as absent rather than as an error.

**A world is its systems, and configuration decides which of them act.** Disabling a system takes
its actions out of dispatch and its reductions out of the pipeline, with no edit to anything else —
an action no enabled system provides is answered `Unavailable`, which is an ordinary answer and not
a failure. That is the whole point of the project, so it has a test of its own:
[`tests/two_systems.rs`](tests/two_systems.rs), where two worlds differ by one boolean.

Some of these guarantees are checked by code that must **not** compile — a system writing state it
does not own, a forged token, an external crate building its own issuer of write capability — in
[`kernel/tests/compile_fail/`](tests/compile_fail/). The single-writer rule's edges, including the
attempts that do succeed and what closes them, are written up in
[`.structured-coding/plans/mvp0/step-03-kernel-and-systems.md`](../.structured-coding/plans/mvp0/step-03-kernel-and-systems.md)
§2.10.2 and §4.8.

```sh
cargo test -p mineworld-kernel
cargo doc -p mineworld-kernel --open      # the crate docs are the reference
```

Four rules govern anything added here, and the crate documentation states them in full: no domain
concept (the kernel does not know what a job is, or what weather is), nothing ordered by when it
happened, identity allocated and never reused, and a refusal that changes nothing — with one
documented narrowing, in [`src/dispatch.rs`](src/dispatch.rs), for the case where a system breaks
its own contract half way through reducing.

Why the storage is purpose-built rather than an ECS is recorded as `DEP-1` in
[`../docs/DECISIONS.md`](../docs/DECISIONS.md). What belongs in the kernel at all is specified in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) §2 and
[`../docs/CORE_CONCEPTS.md`](../docs/CORE_CONCEPTS.md).
