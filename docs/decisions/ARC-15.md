## ARC-15 — A world's initial state is a recorded genesis fact

**Date** 2026-09-27 · **Implements** `MODULE_SPEC.md` §4 · **Relates to** `MVP.md` §9 `AC-9`,
`AC-12`

**Problem.** A World Pack states what is true of a world before anything happens: Alice is behind the
counter. That is component state, and in MineWorld a component is written **only** by the system that
owns it, **only** while it resolves an action or reacts to a fact. World assembly has no action to
resolve — so PR 05b, writing the server's first test world, found that placing four people meant
dispatching four intents, and `ActionIntent::new` needs an `ActionId` that at assembly time no
allocator has issued. Its test world worked around it by allocating from `9_000_000`, far above the
server's own allocator, so that assembly could not collide with a client's request.

**Options considered.** (1) A seeded allocator handed to the loader through configuration — the
server's allocator starts above whatever assembly spent. (2) An assembly-time allocator owned by the
pack loader. (3) A kernel path for authored state.

**Choice: (3), in the form that records events.** `World::genesis(at, facts)` records what a world
begins with and reduces it through the systems that own that state, using the same recorder and the
same reducer a request goes through. Two differences and no third: causation is
`Causation::WorldGenesis`, and there is no controller decision to name.

**Why, and the axis is the event log.** Options 1 and 2 both produce `Causation::Action(9_000_000)`
in a world's history — a request no controller made, no client sent and no allocator issued.
Replaying that log asks *what was action 9000000?* and the honest answer is *nothing; it was a number
chosen to avoid a collision*. That is inventing history, and moving the fabrication from a test into
the loader would tidy the workaround rather than answer it. Option 1 additionally leaks
world-assembly detail into the transport's configuration, so a loader and a server would have to
agree on a number to stay out of each other's way — a coupling with no owner.

A kernel path that wrote components directly would fail the other way: state with no cause in the
log, which breaks `AC-9` for the whole of a world's initial state and makes it unreconstructible from
its own history. `kernel/src/system.rs` already refuses exactly that for `System::install` — "a
system that wrote state while being installed would write facts no event explains" — and
`contracts/src/event.rs` already named the answer: `WorldGenesis` exists "so that a world's initial
state is *explained* rather than uncaused", and "a loaded World Pack's initial facts are caused by
this and by nothing else". The decision was therefore less a choice than a reading of two contracts
that had already made it.

**What it buys.**

```text
no ActionId is invented      Provenance::controller_decision stays None, which is what it means:
                             this fact came from no request. The server's allocator still starts at
                             1 and can never collide with assembly, because assembly allocates no
                             request identities at all.
state has a causal origin    "where did Alice's initial position come from" answers with an event
                             id, a genesis causation, and the system that reduced it — by the same
                             path a walked arrival takes, so state and log cannot disagree.
AC-12 holds through seeding  event ids come from the world's own counter in the order the loader
                             states the facts, and that order is the pack's (places in key order,
                             then people in key order).
```

**Scope, stated exactly.** Three public kernel items (`World::genesis`, `Emission::owner`, two
`KernelError` variants) and no domain knowledge: `genesis` cannot name a component, an action or a
system, and reads each fact's emitting system off `Event::OWNER` in the contract rather than from an
argument. `contracts/` is unchanged — `WorldGenesis` becoming reachable is that layer's own stated
intent.

**Accepted limitations.** Genesis is refused once a world has dispatched anything, so *the world
coming into existence* stays true of the log that claims it; a person who arrives while a world runs
arrives by acting. Reduction of genesis facts is not transactional, exactly as dispatch's is not: a
system that breaks its own contract while reducing leaves what it wrote, and a failed assembly is
discarded rather than repaired (`kernel/src/world.rs`).
