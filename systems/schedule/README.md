# systems/schedule/

**`mineworld-schedule`**: a person's day — where they mean to be, and when. An agenda people may
follow, never a mover.

```text
section     routine              a person file's day: [{ from: "05:30", place: cafe, label: work }, …]
emits       routine-assigned     at genesis, from that section
            agenda-changed       at each boundary of the day
owns        Routine, Agenda      on the person, and one `routine` Process per person
discloses   Agenda               to its holder only
depends on  nothing
```

Each person's day is a Process. It wakes at every boundary, states the new agenda and sleeps until
the next one. It never ends.

The pack moves nobody: it states no other pack's facts. `mineworld run`'s controller walks people to
their agenda's place. A player may ignore it, and a person nobody drives stays where they are while
their day goes by.

```sh
cargo test -p mineworld-schedule
cargo test -p mineworld-cli --test routines     # a 30-day town: every day kept, every seat following
```

The decision is [`ARC-32`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-09-social.md`](../../.structured-coding/plans/mvp0/step-09-social.md) §4.3 (C5–C8).
