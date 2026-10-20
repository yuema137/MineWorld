# systems/needs/

**`mineworld-needs`**: a person gets hungry and tired, and eating and sleeping help.

```text
section     needs:         in a person file: starting hunger and tiredness ({} for 300 and 700)
                           in an item file: { nourishment: { hunger } }, what eating one does
configure   configure/needs.yaml   rates, thresholds, which restorers the world uses
emits       needs-assigned, nourishment-assigned   at genesis only
owns        Needs          on a person: each need's level, when it was true, awake or asleep
            Nourishment    on an item kind
discloses   needs          to the person alone: levels, words, rates, and what their food would do
depends on  nothing        it hears inventory's items-consumed and sleep's fell-asleep and woke
```

Each need is a number from 0 (sated, rested) to 1 000 (famished, exhausted). Nothing is written while it
rises. The pack stores the level and the moment it was true, and works out the current level whenever
someone reads it. So a month of hunger costs no facts at all.

By default hunger rises 120 an hour awake and 30 asleep; tiredness rises 60 an hour awake and falls 125 an
hour asleep. A world changes these in `configure/needs.yaml`. A world without `sleep` must say
`restore: { tiredness: [] }`, so a tiredness that never falls is always the author's choice.

```sh
cargo test -p mineworld-needs
```

The decisions are [`ARC-284`](../../docs/decisions/ARC-284.md) and
[`ARC-285`](../../docs/decisions/ARC-285.md); the section and the file are specified in
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §§4.1–4.2. Design and evidence:
[`pr-s35-nb-needs.md`](../../.structured-coding/plans/mvp0/pr-s35-nb-needs.md).
