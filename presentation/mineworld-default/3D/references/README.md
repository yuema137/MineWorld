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

This matters most for `04_character_closeup.png`, and that image is the exception to the rule
above: **`ARC-19` makes it the identity source of truth for the MineWorld default character.**
Face, hair, freckles, garment structure and accessories are all in scope, superseding `ARC-4`'s
earlier exclusion of facial fidelity, skin rendering and hair simulation for this one image. The
contract derived from its pixels is `../CHARACTER_IDENTITY.md`.

`ARC-4`'s reasoning — that a photoreal face standard would put character production beyond what
community creators can afford — is not overturned. It is applied to the right object. There are
two different bars:

```text
the MineWorld default character   as faithful to 04_character_closeup.png as practical
                                  paid once by this project, shipped as an asset

what the framework requires       the humanoid profile alone: scale, axes, root convention,
                                  skeleton, retarget compatibility, glTF expectations
                                  paid by every creator, and it stays cheap
```

A creator shipping their own world needs a rig that satisfies the profile. They do not need our
face — `same skeleton ≠ same mesh ≠ same face ≠ same clothes`, and one animation library serves
characters who look nothing alike. A high-fidelity default is an example of what the framework
permits, never a threshold it imposes.

Still excluded, now on reuse grounds rather than fidelity grounds: **building character
reconstruction technology from scratch**. Mature tools are evaluated instead, against quality,
automation, licence, redistribution, engine portability and runtime compatibility. `DEP-8` is
unchanged — an asset must be free to *redistribute*, not merely free to use.

## Still uncovered — follow-up, not a blocker

A true interior at room scale (the café interior appears only through a window), and a night or
overcast condition. These are presentation-pack follow-up work and **do not block MVP-0**.
Renderer work stops for them only if it becomes genuinely ambiguous without them.

These are style references, not game content (§14): they fix realism, lighting, material
complexity, density, architectural credibility, character realism, colour and human scale — not
the town MineWorld must build.
