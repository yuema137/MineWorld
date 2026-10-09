# systems/item/

**`mineworld-item`**: what kinds of things exist.

```text
section     item                 an item file's `item: { category: drink, name: Coffee }`
emits       item-kind-declared   at genesis, from that section; public
owns        ItemKind             component `item-kind`, payload { "category": "drink", "name": "Coffee" }
discloses   item-catalogue       on a place, to whoever perceives it: every kind's item, category, name
depends on  nothing
```

An item file declares a kind, not one object (`ARC-36`). This pack says the kind exists, so that
`inventory`, and later the market's other packs, share one vocabulary. Ask `is_declared(world, item)`;
an item file without an `item:` section is an inert entity nobody trades.

```sh
cargo test -p mineworld-item
```

The decision is [`ARC-37`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.4 (D-C2).
