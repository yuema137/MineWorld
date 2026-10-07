# systems/item-transfer/

**`mineworld-item-transfer`**: giving things to people. It owns nothing.

```text
provides    give { item, count }   to a Person in the same place, within 3 m, who can take it
states      items-transferred      inventory's fact, through inventory's checked constructor
offers      one complete give per kind held, count 1, to each other person present
depends on  inventory, presence
```

`give` decides; `inventory` writes, and checks again (`ARC-26`). Each offer carries the exact request,
so the paced controller gives without ever having been compiled against this pack (`ARC-34`). Remove
the pack from a world and nobody is offered a give, a give is answered `Unavailable`, and nothing else
changes (`AC-2`).

```sh
cargo test -p mineworld-item-transfer   # offers, refusals, the unchanged controller giving, AC-2
```

The decision is [`ARC-37`](../../docs/DECISIONS.md). Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.4 (D-C4).
