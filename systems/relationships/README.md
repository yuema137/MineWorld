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

## Its section of the World's Interaction List

A world may say whose contacts form relationships, how much a contact moves familiarity and regard,
and whether these facts enter a biography, in `configure/relationships.yaml` (listed in
`world.yaml`'s `configure:`):

```yaml
rules:
  - { action: acquaint, actor: noble, target: commoner, effect: forbid }
parameters:
  - { spoke_familiarity: 10, accepted_regard: 50, declined_regard: -30,
      activity_familiarity: 50, activity_regard: 20 }
  - { actor: grump, declined_regard: -100 }
  - { place: cafe, spoke_familiarity: 20 }
consequences:
  - { fact: became-acquainted, actor: servant, biography: off }
```

`acquaint` is not an action anybody sends. It is a rule name for the decision this pack makes
while reducing another pack's fact: whether `actor` (the person who would come to know) forms a
relationship with `target` (the counterpart). It is directed, like `knows`: forbidding
`noble → commoner` leaves `commoner → noble` alone. Forbidden, that direction gets no entry, no edge
and no fact. It cannot be put in a region.

| Parameter | Default | Bound |
| --- | --- | --- |
| `spoke_familiarity` | 10 | 0 … 1 000 |
| `accepted_regard` | 50 | −1 000 … 1 000 |
| `declined_regard` | −30 | −1 000 … 1 000 |
| `activity_familiarity` | 50 | 0 … 1 000 |
| `activity_regard` | 20 | −1 000 … 1 000 |

Parameters are looked up with `actor` the person whose values change, `target` the counterpart and
`place` where the causing fact happened. Both facts, `became-acquainted` and `relationship-changed`,
are heard by the two people only; a list may switch their biography (compiled on). The schema is
[`../../docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.2.
