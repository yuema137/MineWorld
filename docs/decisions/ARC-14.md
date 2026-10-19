## ARC-14 — The default 2D style is `town`

**Date** 2026-09-27

[`ARC-11`](#arc-11--default-style-is-taste-style-infrastructure-is-architecture) separates the
default style, which is taste and the operator's to decide, from the style infrastructure, which
is architecture and proceeds autonomously. This record is that decision taken: **the operator
compared four complete scenes and chose `town`.** It is no longer a candidate.

**What was compared.** Four art variants of the same walkable square, identical in layout,
projection, camera, crowd density and scene logic, differing only in which art each role
resolves to:

| Variant | Art |
| --- | --- |
| `town` | generated cast, plus the shared generated set — buildings, props, vegetation, ground |
| `full` | generated cast, plus shopfronts and planted props generated for the spike |
| `people` | generated cast over the procedural world |
| `procedural` | the earlier all-procedural build, as the baseline |

All four were rendered at one commit and at matched framing, including a still framed to
`presentation/mineworld-default/2D/references/02_cafe_street.png` so the comparison against the
plates is like for like. The sets are in `clients/2d-spike/screenshots/`, and
`clients/2d-spike/README.md` gives the launch commands.

**What distinguishes the choice.** `town` carries a warm overall key measurably close to the
reference plates — whole-frame mean luminance 0.513 against the plates' 0.517, with 4.0% of the
frame below 0.20 luminance against 4.4%. Beyond the numbers: cherry blossom against the greens,
a varied roofline of terracotta, slate and tile, and building silhouettes that differ from one
another rather than repeating one mass. It reads as a place with a history of being built in
rather than a row of one shop.

**Its known weakness, accepted.** The shared set has four distinct tree sprites, so repetition
shows at wide zoom where `full`'s nine procedural species do not. The operator chose `town` with
that visible.

**The other three are kept.** They are not dead alternatives — they are the demonstration that
the presentation layer is swappable, which is the substance of `ARC-11`'s split. `--variant=`
remains the interface, and a change that can only be made to `town` is a change made in the
wrong place.

**This fixes the default, not the style system.** MineWorld must host anime, pixel, voxel,
low-poly, photorealistic, retro, hand-painted and minimal styles
([`ARC-11`](#arc-11--default-style-is-taste-style-infrastructure-is-architecture)). A World Pack
ships its own Presentation Pack and selects its own style without touching this choice and
without modifying the default pack; nothing here privileges `town` in the loader, the contracts
or the renderer bindings. If a later change makes `town` hard to replace, that change is the
defect, not this record.

**Recorded because** "good enough" silently becoming "accepted" is the failure `ARC-11` exists to
prevent, and the converse also needs a record: once the operator has chosen, an agent should not
reopen the question as though it were still open.

**Note, 2026-10-08 — how this record reached `main`.** The text above is carried verbatim from
`vis/2d-generated-assets` @ `af5e236`, the branch it was written on, which is not merged (S12 PR 13a,
`.structured-coding/plans/mvp0/step-13-client-2d.md` §14). It fills the identifier gap `ARC-16`
recorded. Two things changed since, both operator decisions of 2026-10-08 (QS12-1, QS12-2): the
style is now drawn by the connected reference client `clients/2d/`, laid out from a world's disclosed
passages (market-town's street) rather than the spike's invented square; and the art moved into
the Presentation Pack `presentation/mineworld-default/2D/`, where the four variants are binding sets
(`ARC-46`). `clients/2d-spike/` and its screenshots stay on that branch as the visual reference.
Neither change reopens the choice of `town`.
