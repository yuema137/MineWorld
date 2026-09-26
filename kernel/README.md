# kernel/

MineWorld's authoritative world state: **`mineworld-kernel`**, the Rust crate that holds who
exists, what state they carry, and how they are connected — and lets only the owning system
change any of it.

What exists today:

```text
entities     EntityRegistry     identity, allocated from a counter and never reused
components   ComponentStore     one table per component type, writes gated on ownership
relations    RelationStore      typed edges, written by the system that declared the type
access       SystemIdentity, OwnedBy<S>, WriteToken<S>, WriteAccess, owned_component!
errors       KernelError
```

Still to come, in this order: the `System` interface with registration and dispatch, then the
world clock and scheduler, then persistence and the event log.

## The one idea to understand first

A component is written only by the system that owns it, and that is the compiler's rule here, not
a review convention:

```rust
store.insert(&my_token, entity, MyComponent { .. })?;   // fine
store.insert(&my_token, entity, YourComponent { .. })?; // does not compile
```

A write needs the owning system's `WriteToken`, tokens are granted rather than constructed, and
`owned_component!` declares a component and its owner together so the two cannot drift apart.
Reads are open on purpose: any system may read any component, and a component whose owning system
is not installed reads as absent rather than as an error, which is what lets systems be added and
removed without breaking the ones that were watching.

Some of these guarantees are checked by code that must **not** compile — a system writing state it
does not own, a forged token, a pack claiming another crate's type — in
[`kernel/tests/compile_fail/`](tests/compile_fail/). The single-writer rule's edges, including two
attempts that do succeed and what closes them, are written up in
[`.structured-coding/plans/mvp0/step-03-kernel-and-systems.md`](../.structured-coding/plans/mvp0/step-03-kernel-and-systems.md)
§2.10.2.

```sh
cargo test -p mineworld-kernel
cargo doc -p mineworld-kernel --open      # the crate docs are the reference
```

Four rules govern anything added here, and the crate documentation states them in full: no domain
concept (the kernel does not know what a job is), nothing ordered by when it happened, identity
allocated and never reused, and a refusal that changes nothing.

Why the storage is purpose-built rather than an ECS is recorded as `DEP-1` in
[`../docs/DECISIONS.md`](../docs/DECISIONS.md). What belongs in the kernel at all is specified in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) §2 and
[`../docs/CORE_CONCEPTS.md`](../docs/CORE_CONCEPTS.md).
