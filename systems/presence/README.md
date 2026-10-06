# systems/presence/

**`mineworld-presence`** — where people are, what one of them perceives, and what the server says
they may attempt.

```text
owns        Presence               where a person is: a place, and optionally where in it
            present-in             the edge that says which place a person is in
provides    nothing                who may move a person is another system's decision
emits       arrived                somebody is now somewhere (genesis, or through `arrival`)
            person-entered-place   somebody is now in another place: an occupancy change
subscribes  arrived                so the state above is a projection of the event log
```

Two halves, and they are different kinds of thing:

- **`PresenceSystem`** is installed into a world, declares what it owns, and reduces `arrived`
  into that state.
- **`observe(...)`** is a query. Perception is a read, so it is a function over a composed world,
  and what it returns is an `Observation`: a value listing what one observer was shown, with no
  handle to the world inside it.

## Why this pack does not know what `talk` is

An observation carries **affordances** — what this observer may attempt, each with the server's
verdict — so that a 2D client and a 3D client can offer the same interactions without either
implementing a rule. The answers are not computed from a list of actions kept here. Each pack
implements `PerceptionProvider` and offers its own actions; this pack asks the kernel's route map
whether the action is still provided, and the contract layer's `SpatialRequirement::evaluate`
whether it is possible right now.

The same seam carries **state**: a pack says which of its own components an observer may know about
a given entity, and this pack puts back exactly what it is handed. It never walks the component
stores, because exposing a component on the grounds that it exists is how an observation becomes a
window onto everything.

The consequence is the point: disabling a pack removes its affordances *and* its state from every
observation in the world, and adding a pack requires no edit here. A test scans these sources for
another pack's vocabulary, because the claim is an absence and only a structural test can hold it.

## Presence owns where people are; who may move them is another system's decision

A world places its people by genesis. After that, a system that decides movement — `movement`, in
the worlds this repository ships — depends on this pack and states its `arrived` through the checked
`arrival` constructor; this pack reduces it and still refuses a value its state may not hold
([`DECISIONS.md` `ARC-26`](../../docs/DECISIONS.md)). When a person's place changes, this pack states
`person-entered-place`, because occupancy is its state. The `arrive` action this pack once provided
is retired: a distance rule beside an unrestricted relocation is not a rule.

```sh
cargo test -p mineworld-presence
cargo doc -p mineworld-presence --open      # the crate docs are the reference
```

Read [`src/lib.rs`](src/lib.rs) first, then [`src/interaction.rs`](src/interaction.rs) for the seam
and [`src/observe.rs`](src/observe.rs) for what an observer is and is not told.
