# systems/appearance/

**`mineworld-appearance`**: what people look like.

```text
section     appearance           a person file's `appearance: { look: alice-moreau }`
emits       appeared             at genesis, from that section; public
owns        Look                 `look`, payload { "look": "alice-moreau" }, reduced from `appeared` by this pack alone
discloses   Look                 to every observer who perceives the person — what someone looks like is visible
depends on  nothing
```

A look is a key, not art. The world says that Alice has the appearance called `alice-moreau`; the
Presentation Pack a client uses says what that looks like (`characters/cast.yaml` in 3D, a
`look:alice-moreau` role in 2D). A world without this pack, or a person without the section, has no
look, and every client draws them with its fallback.

```sh
cargo test -p mineworld-appearance
```

The decision is [`ARC-91`](../../docs/DECISIONS.md); the binding in clients is
[`ARC-92`](../../docs/DECISIONS.md). The section format is [`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md)
§4.1. Design and evidence: [`pr-s26a-appearance.md`](../../.structured-coding/plans/mvp0/pr-s26a-appearance.md).
