# MineWorld default presentation style

One style family, two dimension-scoped packs. Both describe the same kind of place — a calm,
ordinary, warm, walkable lakeside town — and both drive the *same* semantic world.

| Pack | Look | Covers |
| --- | --- | --- |
| [`3D/`](3D/) | grounded semi-realistic cozy realism | promenade, square, café frontage, a character close-up, a golden-hour street, a lake trail |
| [`2D/`](2D/) | clean illustrated isometric | square, café street, harbour, park and residences |

They are two packs rather than one because a single manifest cannot honestly describe both: the
3D references use realistic human proportions, the 2D references are stylized. A manifest that
claimed one value for `characters.proportions` would be false for half its own references.
Recorded as [`../../docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-3`.

What the two packs must share is not art. It is meaning: a `Talk` is the same `ActionIntent`
whether the player clicked a sprite or walked up to someone and pressed a key, and neither pack
decides whether that `Talk` is allowed
([`../../docs/ART_DIRECTION.md`](../../docs/ART_DIRECTION.md) §§19–20).

```text
mineworld-default/
├── 2D/  manifest.yaml · ART_DIRECTION.md · references/
└── 3D/  manifest.yaml · ART_DIRECTION.md · references/
```

The directory is `mineworld-default`, not `-realistic`: "realistic" describes the 3D pack and
would be false for the 2D one.
