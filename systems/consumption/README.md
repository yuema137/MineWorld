# systems/consumption/

**`mineworld-consumption`**: eating the food and drinking the drinks a person carries.

```text
provides    eat { item }       a held kind of category `food`
            drink { item }     a held kind of category `drink`
states      items-consumed     inventory's fact, through `consume`
offers      one complete eat or drink per edible or drinkable kind held, no target
owns        nothing
depends on  inventory          (and reads item's category)
```

A person eats what they carry, wherever they are. Goods — a mug, a pen — are never consumed. This
is the interaction, not a need: nobody gets hungry. Removing this pack removes `eat` and `drink`
and nothing else.

```sh
cargo test -p mineworld-consumption
```

The decision is [`ARC-38`](../../docs/DECISIONS.md). Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.5 (E-C5).
