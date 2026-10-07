# systems/relationships/

**`mineworld-relationships`**: who knows whom, and how well. It changes only because something
happened between two people.

```text
declares    knows                  a directed edge, Person → Person
owns        Acquaintances          per counterpart: familiarity, regard, exchanges, activities shared
subscribes  spoke, invitation-accepted, invitation-declined, group-activity-ended
emits       became-acquainted, relationship-changed (only when a level is crossed, up or down)
provides    nothing
depends on  nothing
```

It provides no action and runs no process. Other packs state what happened, and this pack, the
owner of relationship state, reacts. A world without conversation or group-activity still installs
it. It simply hears nothing from the missing pack.

How Alice regards Bob is disclosed to Alice and to nobody else.

**Known gap:** values never decay. In a long run every pair that keeps meeting reaches its top level
within weeks, and then nothing changes. The 300-day test prints this per 30-day bucket rather than
hiding it.

```sh
cargo test -p mineworld-relationships
```

The decision is [`ARC-28`](../../docs/DECISIONS.md). Design and evidence:
[`step-09-social.md`](../../.structured-coding/plans/mvp0/step-09-social.md) §4.2 (SD-5…SD-9, C3).
