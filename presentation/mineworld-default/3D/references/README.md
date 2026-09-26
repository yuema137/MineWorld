# Reference images — default 3D style

These define the 3D look. Prose and the manifest summarise them; when they disagree, the images
win ([`../../../../docs/ART_DIRECTION.md`](../../../../docs/ART_DIRECTION.md) §16).

| File | Covers |
| --- | --- |
| `01_lakeside_promenade.png` | the original "Style C": promenade, café and convenience-store frontage, mountains and sailboats, pedestrians, a dog, benches, planters, late afternoon |
| `02_town_square.png` | plaza with fountain, café and bookshop, wayfinding signpost, lake beyond, a person seated on a bench with a dog |
| `03_cafe_frontage.png` | café exterior at conversational scale: outdoor tables, a seated customer with coffee, sandwich boards, a barista visible through the window |
| `04_character_closeup.png` | a person at close range on a main street, with others seated outside a coffee shop |
| `05_main_street_golden_hour.png` | street corner, shopfronts, golden-hour light, no people — the architecture and materials on their own |
| `06_lake_trail.png` | nature outside the town: lakeside trail, jetty and boats, mountains, ducks, low sun |

What they establish: believable modern small-town architecture in wood, brick, stucco, stone and
painted metal; human-scale streets; moderate environmental detail that stays readable; natural
warm light, mostly late afternoon; muted natural colour with saturated accents only in flowers
and signage.

## Scope of authority — ruled 2026-09-25

A reference is the strongest source of truth **within the dimensions it is designated to
represent**, and carries no authority outside them
([`../../../../docs/DECISIONS.md`](../../../../docs/DECISIONS.md) `ARC-4`). Each entry in
`manifest.yaml` states its `authoritative_for` list.

This matters most for `04_character_closeup.png`. It is a valuable reference for **body
proportions, casual modern clothing, how a character sits in its environment, the camera distance
of a close encounter, and ordinary-person visual identity**. Its **facial photorealism is
explicitly not the fidelity target**.

The default 3D character is:

```text
realistic anatomy and proportions
+ believable ordinary clothing
+ natural animation
+ recognizable facial expression
+ moderate facial geometry and material detail
+ subtle simplification
```

and it explicitly does not depend on photoreal skin, MetaHuman-level facial assets, facial
scanning, cinematic hair simulation, or a custom character pipeline. The reason is architectural
rather than aesthetic: the default presentation has to stay practical for open-source and
community-created worlds, and a photoreal face standard would make community character production
prohibitively expensive.

## Still uncovered — follow-up, not a blocker

A true interior at room scale (the café interior appears only through a window), and a night or
overcast condition. These are presentation-pack follow-up work and **do not block MVP-0**.
Renderer work stops for them only if it becomes genuinely ambiguous without them.

These are style references, not game content (§14): they fix realism, lighting, material
complexity, density, architectural credibility, character realism, colour and human scale — not
the town MineWorld must build.
