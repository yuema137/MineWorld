# systems/inventory/

**`mineworld-inventory`**: what people and organizations hold — and the only pack that writes it.

```text
section     holdings             a person's or organization's `holdings: { coffee: 2, tea: 1 }`
emits       stocked              at genesis, from that section; visible to the holder
            items-transferred    stated by a deciding pack through `transfer`; visible to both sides
owns        Holdings             component `holdings`: [{ item, count }] in item order, never zero
discloses   Holdings             to the holder only
depends on  item                 only a declared kind can be held
```

A pack that decides a give — and later a purchase — does not touch holdings. It asks
`admit_transfer`, states the fact through `transfer`, and inventory checks it again before writing
anything (`ARC-26`). A person carries at most `PERSON_CAPACITY = 6` items, every kind together; an
organization is not bounded. Nothing is used up yet, and without the bound a person nobody drives
would absorb the town's items.

```sh
cargo test -p mineworld-inventory
```

The decision is [`ARC-37`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.4 (D-C3).
