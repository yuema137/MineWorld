# systems/naming/

**`mineworld-naming`**: what people are called.

```text
section     name                 a person file's `name: Alice Moreau`
emits       named                at genesis, from that section
owns        DisplayName          component `display-name`, payload { "name": "Alice Moreau" }
discloses   DisplayName          to everybody who perceives the person — names are public
depends on  nothing
```

Before this pack, nothing owned a name, so no client could show one and Alice said "Earlier, person
4 said …". Now a client reads the name from the observation, and a controller says "Earlier, Vera
Lindgren said …".

It is called `naming` because "identity" is the kernel's word for an entity's id.

```sh
cargo test -p mineworld-naming
```

The decision is [`ARC-31`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-09-social.md`](../../.structured-coding/plans/mvp0/step-09-social.md) §4.3 (C2–C4).
