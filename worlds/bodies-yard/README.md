# Bodies Yard

Two walled rooms — a hall with a table and a court with a pillar — joined by one door, and twelve
people who walk, talk and get in each other's way. Nobody walks through a wall, the table, the pillar
or anybody else: people stop at furniture and nudge each other aside a little.

```sh
mineworld validate worlds/bodies-yard
mineworld run worlds/bodies-yard --headless --seed 7 --days 30
```

It is the integration world of the `bodies` System Pack. The rules are in
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) (`ARC-39`'s note, `DEP-13`); the `body:` section in
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1.
