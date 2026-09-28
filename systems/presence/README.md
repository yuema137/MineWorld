# systems/presence/

**`mineworld-presence`** — where people are, what one of them perceives, and what the server says
they may attempt.

```text
owns        Presence            where a person is: a place, and optionally where in it
            present-in          the edge that says which place a person is in
provides    arrive              a person comes to be at a location
emits       arrived             and the fact that records it
subscribes  arrived             so the state above is a projection of the event log
```

Two halves, and they are different kinds of thing:

- **`PresenceSystem`** is installed into a world and asked four things — install, validate,
  resolve, react.
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

## `arrive` is not movement

It records an arrival; it does not walk anybody anywhere. Travel is a `Process` that takes simulated
time and does not exist yet. Without an action, though, location could not exist at all — a
component is written only by its owning system, so nothing else can say where anybody is.

```sh
cargo test -p mineworld-presence
cargo doc -p mineworld-presence --open      # the crate docs are the reference
```

Read [`src/lib.rs`](src/lib.rs) first, then [`src/interaction.rs`](src/interaction.rs) for the seam
and [`src/observe.rs`](src/observe.rs) for what an observer is and is not told.
