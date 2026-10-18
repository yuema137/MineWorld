# systems/relationships/

**`mineworld-relationships`**: who knows whom, and how well. It changes only because something
happened between two people.

```text
declares    knows                  a directed edge, Person → Person
owns        Acquaintances          per counterpart: familiarity, regard, exchanges, activities shared
subscribes  spoke, invitation-accepted, invitation-declined, group-activity-ended
emits       became-acquainted, relationship-changed (only when a level is crossed, up or down),
            acquaintance-forgotten (when the acquaintance cap evicts somebody)
provides    nothing
depends on  nothing
catalog     DecayCurve             the curves familiarity and regard may fade by (ARC-62, ARC-86)
```

It provides no action and runs no process. Other packs state what happened, and this pack, the
owner of relationship state, reacts. A world without conversation or group-activity still installs
it. It simply hears nothing from the missing pack.

How Alice regards Bob is disclosed to Alice and to nobody else.

By default nothing fades and a person keeps at most 150 acquaintances, which no reference world
reaches. A world that wants people to forget each other says so in its configuration (below). Without
it, every pair that keeps meeting reaches its top level within weeks and stays there.

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
`place` where the causing fact happened. The three facts, `became-acquainted`, `relationship-changed`
and `acquaintance-forgotten`, are heard by the two people only; a list may switch their biography
(compiled on). The schema is [`../../docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.2.

## Forgetting: decay and the acquaintance cap

```yaml
parameters:
  - { familiarity_decay: half-life, familiarity_decay_days: 90, familiarity_floor: 50,
      regard_decay: linear, regard_decay_days: 365, known_at_most: 150 }
  - { target: kin, familiarity_decay_days: 365 }     # kin fade slower
  - { actor: hermit, known_at_most: 15 }
```

| Parameter | Type | Default | Bound | Scoped by |
| --- | --- | --- | --- | --- |
| `familiarity_decay` | curve name | `none` | a bundled curve or one in the catalog | actor, target |
| `familiarity_decay_days` | days | 365 | 1 … 3 650 | actor, target |
| `familiarity_floor` | | 0 | 0 … 1 000 | actor, target |
| `regard_decay` | curve name | `none` | as above | actor, target |
| `regard_decay_days` | days | 365 | 1 … 3 650 | actor, target |
| `regard_floor` | | 0 | 0 … 1 000 | actor, target |
| `known_at_most` | people | 150 | 5 … 1 500 | actor |

- **Decay is lazy.** Nothing runs on a timer and no fact is written for it. A value's current reading
  is computed from the stored value, the time since the two last had contact, and the curve, wherever
  it is read: when a contact adds to it, when it is disclosed to its holder, and when the cap ranks
  entries. Familiarity fades toward its floor; regard fades toward zero from either side and never
  below `regard_floor` in magnitude. Forgetting only lowers a value: whatever a curve returns, this
  pack clamps it.
- **Curves:** `none` (the default), `linear` (loses the whole range, 1 000, in one time constant),
  `half-life` (halves every time constant, by an integer table), `power` (`T ÷ (T + t)`). All integer
  arithmetic, so every platform computes the same value. The 365-day default is not calibrated; a
  world that turns decay on states its own.
- **Levels at contact.** A level crossed by fading is stated by the next contact between the pair, as
  one `relationship-changed` from the last stated level to the new one, caused by that contact.
- **The cap.** When a new acquaintance takes a person past `known_at_most`, the entry with the lowest
  current familiarity is evicted (ties: the oldest contact, then the lowest id; never the person just
  met), its `knows` edge is removed, and `acquaintance-forgotten` is stated. Meeting them again starts
  afresh with a new `became-acquainted`.
- **Not by place.** These fields describe a pair or a person, not a place: an entry with `place:`, or a
  region, that sets one is refused at load, as is a `target:` entry that sets `known_at_most`.
- **The reading's instant** is the world's. In `mineworld run` it is the observation's instant; on a
  hosted server a disclosure may trail the wall clock by the time since the world last advanced.
- **Rounding.** Each contact re-bases the values at their current reading, so a pair that meets very
  often fades marginally faster than the closed form, by at most one point per contact.

## Writing your own curve

A curve is code, through the `DecayCurve` extension catalog (`ARC-62`):

1. Write a System Pack crate whose type implements `mineworld_relationships::DecayCurve` and
   `Default`. `remaining(excess, elapsed, time_constant)` returns how much of the value's excess over
   its floor is left; it must be pure and should not increase with `elapsed`.
2. Install it: its line in `systems/installed/Cargo.toml` and in `installed!`, and its type on
   relationships' `extension mineworld_relationships::DecayCurve` line. Rebuild.
3. Enable its pack in the world's `systems:` list, so a save records which curve code it ran with.
4. Name it: `{ familiarity_decay: my-curve, familiarity_decay_days: 120 }`.
5. The loader refuses an unknown curve name at its line and column, and a curve whose pack the world
   does not enable, by name. Replay and resume work as for every reduction.

The decision is [`ARC-86`](../../docs/DECISIONS.md); the design and its arithmetic are
[`pr-fg-a-relationship-forgetting.md`](../../.structured-coding/plans/mvp0/pr-fg-a-relationship-forgetting.md)
§§4–5.
