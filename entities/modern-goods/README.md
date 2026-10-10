# modern-goods

Everyday shop goods — food, drink and household things — as an Entity Pack any world can use.

```sh
mineworld packs validate entities/modern-goods
```

A world uses it by requiring it and naming the pack root:

```yaml
requires:
  modern-goods: "^0.1"
```

```sh
mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default
```

Every kind carries an `item:` section, so a world that requires this pack must enable `item`.
The rules are in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §2 and
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-71`, `ARC-77`.
