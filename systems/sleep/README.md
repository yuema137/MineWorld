# systems/sleep/

**`mineworld-sleep`**: a person lies down to sleep for a while, and gets up again.

```text
provides    sleep { for }      minutes; the alarm is now + for (15 … 840 unless the world says otherwise)
            wake  { }          get up
emits       fell-asleep        to the room; it does not carry the alarm
            alarm-set          to the sleeper only
            woke               with the reason: alarm, asked, addressed or acted
owns        Asleep             on a sleeping person, and one sleep Process ending at the alarm
discloses   asleep             to anyone who sees the sleeper; the alarm only to the sleeper
            sleep-terms        one's own bounds for `for`, where one is
depends on  presence
```

Where you may sleep is the world's choice. With no `configure/sleep.yaml`, people sleep at places
tagged `residential`. A world widens or narrows that with rules, for example
`- { action: sleep, place: bench, effect: permit }`.

A sleep ends four ways: the alarm rings, the sleeper asks to `wake`, somebody talks to them (unless
the world sets `woken_by_talk: false`), or they walk or speak. `movement` and `conversation` never
learn that sleep exists. Removing this pack removes `sleep` and `wake` and nothing else.

```sh
cargo test -p mineworld-sleep
```

The decision is [`ARC-283`](../../docs/decisions/ARC-283.md); the section is specified in
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.2. Design and evidence:
[`pr-s35-na-sleep.md`](../../.structured-coding/plans/mvp0/pr-s35-na-sleep.md).
