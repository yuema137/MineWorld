# Bodies Yard

Two walled rooms — a hall with a table and a court with a pillar — joined by one door, twelve people
who walk, talk and get in each other's way, and sixteen loose boxes and balls. Nobody walks through a
wall, the table, the pillar, an object or anybody else: people stop at furniture, nudge each other
aside a little, and push objects out of their way. Within reach, a person may `kick` or `throw` an
object, or `shove` somebody half a metre.

```sh
mineworld validate worlds/bodies-yard
mineworld run worlds/bodies-yard --headless --seed 7 --days 30
```

It is the integration world of the `bodies` System Pack. The rules are in
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) (`ARC-39`'s note, `DEP-13`); the `body:` section in
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1.
