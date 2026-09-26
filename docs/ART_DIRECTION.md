# MineWorld Default 3D Art Direction

**Status:** default presentation direction, and the definition of the Presentation Style Pack
**Audience:** coding agents, contributors, artists, and asset selection.

Scope note. This document does two things: §§1–11 and §§20–22 define the *default* MineWorld
visual direction, and §§12–19 define the *Presentation Style Pack* concept that any style —
default or not — is expressed in. The default direction binds the default pack only; §11 is
explicit that no world is required to use it.

It is not a sixth pack type. A Presentation Style Pack is the specified internal structure of the
Presentation Pack already defined in [`MODULE_SPEC.md`](MODULE_SPEC.md) §6 — see
[`DECISIONS.md`](DECISIONS.md) `ARC-1`.

**The four canonical reference images are not yet in the repository.** `§13` makes them the
primary art-direction source, so until they are committed to
`presentation/mineworld-default-realistic/references/`, the prose below is the only available
statement of the style and is weaker than intended. This gap is recorded in that directory's
`README.md`.

---

## 1. Default Visual Direction

The four approved MineWorld reference images define the default 3D visual direction.

The target style can be described as:

> **Grounded, semi-realistic cozy realism for an ordinary modern living world.**

The world should look believable enough that the player feels they are walking through a real town, while avoiding the production cost and visual fragility of full photorealism.

The desired result is closer to:

```text
realistic proportions
+ believable architecture
+ natural materials
+ human-scale environments
+ warm natural lighting
+ moderate geometric/detail complexity
+ subtle artistic simplification
```

than either:

```text
cartoon / anime / low-poly stylization
```

or:

```text
hyper-photorealistic cinematic rendering
```

The reference images should be treated as the primary visual target.

---

# 2. Core Feeling

MineWorld should feel like a place where ordinary people actually live.

The visual experience should communicate:

- calmness;
- everyday life;
- walkability;
- warmth;
- safety;
- curiosity;
- tourism and exploration;
- human-scale spaces;
- environmental credibility;
- places where the player naturally wants to stop, look around, and interact.

A typical scene might contain:

```text
a café
a convenience store
apartments
trees
benches
street lamps
bicycles
small restaurants
parks
a waterfront
mountains
ordinary pedestrians
dogs
small signs
outdoor seating
```

The environment should feel attractive without looking artificially luxurious or fantastical.

The ideal reaction is:

> "This feels like somewhere I could actually visit and spend an afternoon."

---

# 3. Realism Level

The default should be **realistic, but intentionally not maximally realistic**.

We do not need:

```text
film-quality assets
extreme texture resolution
perfect skin simulation
cinematic facial rendering
dense microgeometry
AAA-level environmental clutter
```

Instead we want:

```text
believable scale
believable lighting
believable materials
recognizable everyday architecture
realistic human proportions
clean environment design
moderate asset detail
consistent visual quality
```

The world should still look good when built from reusable/modular asset packs.

This is important because MineWorld is infrastructure for community-created worlds, not a single handcrafted AAA game.

---

# 4. Environment Style

## Architecture

Default architecture should resemble believable modern small towns and walkable neighborhoods.

Examples:

- lakeside towns;
- mountain towns;
- suburban downtown areas;
- pedestrian commercial streets;
- cafés and local shops;
- small apartment buildings;
- parks;
- waterfront promenades.

Buildings should use realistic proportions and recognizable construction materials:

```text
wood
brick
stucco
stone
glass
painted metal
concrete
```

Avoid overly exaggerated fantasy proportions.

---

## Environmental Detail

Use enough detail to make the world feel inhabited:

```text
benches
signs
flower planters
trash bins
street lamps
bicycles
outdoor tables
shop windows
small decorations
trees and vegetation
```

but do not overwhelm every scene with visual clutter.

The player should be able to quickly understand:

```text
where they can walk
where entrances are
which objects appear interactive
where NPCs are
what kind of place they are in
```

Gameplay readability remains more important than decorative density.

---

# 5. Lighting

Default lighting should favor natural, pleasant conditions.

Preferred examples:

```text
soft daylight
late afternoon
golden hour
mild overcast
warm shop interiors
natural shadows
subtle atmospheric depth
```

Avoid excessive cinematic contrast.

Avoid making every scene look like:

```text
a movie poster
a cyberpunk scene
a dramatic fantasy landscape
```

Lighting should make ordinary life attractive rather than dramatic.

---

# 6. Color Direction

The default palette should be natural and moderately warm.

Use:

```text
natural greens
wood tones
stone
muted blues
warm interior light
soft neutral clothing
moderately saturated flowers/signage
```

Avoid:

```text
extreme saturation
neon-heavy palettes
heavy teal-orange grading
washed-out gray realism
```

Color should support a peaceful exploration atmosphere.

---

# 7. Human Characters

Characters should have:

```text
realistic human proportions
normal everyday clothing
recognizable age variation
ordinary hairstyles
believable posture
subtle facial stylization if needed
```

Characters do not need photorealistic faces.

In fact, slight simplification is desirable because:

- it reduces uncanny-valley risk;
- it lowers asset-production requirements;
- it makes community-created characters easier;
- it keeps visual consistency across different asset sources.

Target:

> realistic enough to feel like normal people, simplified enough to remain practical.

Avoid highly stylized anime proportions or cartoon anatomy in the default pack.

---

# 8. Animation Direction

Animation should prioritize:

```text
readability
natural timing
smooth locomotion
clear interaction states
```

over cinematic fidelity.

Important actions include:

```text
walking
standing
sitting
talking
looking
holding objects
giving objects
using doors
eating/drinking
working
simple group activities
```

Minor imperfections are acceptable.

The key requirement is that characters do not visually feel disconnected from the semantic world state.

---

# 9. Camera and Embodiment

The default 3D interaction model should remain Minecraft-like in its directness:

```text
move through world
look around
approach entity
target entity
press interact
```

However:

> **Minecraft is only an interaction reference. It is not a visual reference.**

MineWorld should not inherit:

```text
block geometry
voxel aesthetics
pixel textures
Minecraft proportions
```

The visual target is the approved semi-realistic MineWorld reference imagery.

---

# 10. Production Philosophy

The default visual style should be achievable using a mixture of:

```text
high-quality open-source assets
commercially compatible asset packs
procedural/environment tools
community-created assets
small amounts of custom art
```

Do not require every world creator to employ a professional 3D art team.

Whenever possible:

> reuse mature asset ecosystems instead of building generic environmental assets from scratch.

The default presentation pack should demonstrate how good visual coherence can be achieved through:

```text
careful asset selection
material normalization
lighting
scale consistency
vegetation
color grading
```

rather than expensive custom modeling.

---

# 11. Art Style Must Be Modular

This visual direction is the **default MineWorld presentation style**, not a requirement imposed on all MineWorld worlds.

A creator may instead build:

```text
pixel 2D
anime 3D
low-poly
voxel
photorealistic
retro PS1
hand-painted
science-fiction
medieval
cartoon
minimalist
```

without changing semantic world logic.

Presentation remains independent from simulation.

Therefore:

```text
Lakewood World
+
Default Realistic Presentation
```

and

```text
Lakewood World
+
Anime Presentation
```

may represent the same semantic world.

---

# 12. Presentation Style Pack

MineWorld should introduce a first-class concept:

> **Presentation Style Pack**

A Presentation Style Pack defines the intended art direction and the concrete presentation configuration used by a renderer.

It should not consist solely of a prompt.

Recommended structure:

```text
presentation/
  mineworld-default-realistic/
    manifest.yaml

    references/
      environment_01.png
      environment_02.png
      environment_03.png
      environment_04.png

    ART_DIRECTION.md

    assets/
      asset_bindings.yaml

    materials/
      material_rules.yaml

    characters/
      character_style.yaml

    lighting/
      lighting_profile.yaml

    renderer/
      godot.yaml
      unreal.yaml

    generation/
      optional_prompt_guidelines.md
```

Not every style pack must contain every directory.

---

# 13. Reference Images Are the Primary Art-Direction Source

The approved reference images should be stored with the default style pack.

For example:

```text
presentation/
  mineworld-default-realistic/
    references/
      lakeside_01.png
      lakeside_02.png
      lakeside_03.png
      lakeside_04.png
```

They act as the primary visual reference for:

- human contributors;
- coding agents;
- artists;
- asset selection;
- future generative asset tools.

When reviewing a visual change, contributors should be able to ask:

> Does this still look like it belongs in the same world as these references?

This is significantly more reliable than relying on prose alone.

---

# 14. Reference Images Are Not Runtime Assets by Default

Reference images describe **style**, not necessarily game content.

They should not imply that MineWorld must reproduce:

```text
the exact town
the exact café
the exact mountains
the exact characters
the exact signs
```

They define:

```text
realism level
lighting
material complexity
environment density
architectural credibility
character realism
color atmosphere
human scale
```

The actual world may be completely different.

---

# 15. Style Manifest

In addition to images, each style pack should contain a concise machine-readable manifest.

Example:

```yaml
id: mineworld-default-realistic
name: MineWorld Default Realistic

dimension:
  - 3d

direction:
  realism: semi_realistic
  stylization: low
  detail_level: medium
  atmosphere: warm_calm
  environment_density: medium

characters:
  proportions: realistic
  facial_detail: medium
  stylization: subtle

materials:
  physically_based: preferred
  texture_detail: medium

lighting:
  default: natural
  contrast: moderate
  preferred_conditions:
    - daylight
    - golden_hour
    - soft_overcast

environment:
  human_scale: true
  readability_priority: high

reference_images:
  - references/lakeside_01.png
  - references/lakeside_02.png
  - references/lakeside_03.png
  - references/lakeside_04.png
```

The exact schema should remain small and evolve only when real use cases require it.

Do not overengineer an exhaustive universal art ontology.

---

# 16. Why Images + Manifest Are Better Than Prompts Alone

A text prompt is useful during content generation, but it should **not** define the style contract.

Prompts have several problems:

```text
model-dependent
version-dependent
non-deterministic
ambiguous
hard to compare visually
difficult for human contributors to interpret consistently
```

Therefore the priority should be:

```text
1. Reference Images
2. Human-readable Art Direction
3. Small Style Manifest
4. Optional Generation Prompts
```

Prompts are an implementation aid.

They are not the source of truth.

---

# 17. Optional Generative Prompt Guidelines

If creators use generative tools, the style pack may provide recommended prompt language.

For the default MineWorld style, a suitable conceptual description is:

> Grounded semi-realistic 3D game environment, believable modern everyday architecture, realistic proportions, moderate geometric detail, physically plausible but simplified materials, natural warm daylight, soft atmospheric depth, ordinary people wearing casual modern clothing, human-scale walkable streets, calm lakeside/small-town atmosphere, attractive but not cinematic or luxurious, polished indie-game realism, designed for exploration and life simulation rather than action.

Negative guidance:

> Avoid anime, cartoon proportions, voxel/block visuals, exaggerated fantasy architecture, hyper-photorealistic cinematic rendering, extreme texture detail, neon cyberpunk lighting, heavy stylization, excessive clutter, dramatic action-game composition.

However, this prompt is only guidance.

The approved reference images remain the stronger style definition.

---

# 18. Asset Selection Rule

When selecting assets for the default style, prioritize consistency over individual asset fidelity.

Five individually beautiful assets with incompatible style may produce a worse world than five moderately detailed assets with coherent:

```text
scale
materials
lighting response
geometry density
color palette
character proportions
```

Therefore visual integration should include:

```text
scale normalization
material normalization
lighting checks
LOD consistency
collision validation
interaction anchor validation
```

---

# 19. Style Packs and Semantic Interactions Must Remain Separate

Presentation packs may describe how an interaction looks.

They may not define whether the interaction is valid.

Example:

Semantic action:

```text
Sit(actor, bench)
```

Default realistic 3D presentation:

```text
walk to seat anchor
play sit animation
```

Pixel presentation:

```text
move sprite to seat tile
switch seated sprite
```

The `Sit` interaction still belongs to the relevant simulation System.

The Presentation Style Pack only determines its visual representation.

---

# 20. 2D Art Direction

The first 2D demo does not need to imitate the 3D references literally.

Its purpose is fast architecture validation.

However, the default 2D presentation should preferably preserve the same high-level mood:

```text
calm
ordinary
warm
walkable
human-scale
environmentally readable
```

A simplified semi-realistic or clean illustrated top-down style is appropriate.

The 2D and 3D presentations do not need identical art.

They must represent identical semantic interactions.

---

# 21. Default Style Acceptance Criteria

The default 3D presentation should visually satisfy the following:

### Environment

A screenshot should plausibly resemble an attractive but believable real-world neighborhood or travel destination.

### Characters

Characters should resemble ordinary people rather than fantasy heroes or heavily stylized avatars.

### Detail

The world should look intentionally designed rather than primitive, while still being practical for an open-source project.

### Mood

The player should feel encouraged to:

```text
walk
look around
enter places
sit
talk
explore
linger
```

rather than feel pushed toward combat or objective completion.

### Technical practicality

The style must remain achievable using modular reusable assets.

---

# 22. Default Style Summary

The MineWorld default 3D visual style is:

> **A warm, grounded, semi-realistic indie 3D aesthetic depicting believable ordinary life. Environments use realistic architecture, natural materials, moderate detail, soft natural lighting, human-scale streets and interiors, and restrained artistic simplification. Characters use realistic proportions with slightly simplified facial and material detail. The goal is not photorealism, but visual credibility: the player should feel that they are walking through a pleasant place that could plausibly exist in the real world.**

The four approved lakeside-town reference images are the canonical visual references for this default presentation direction.

MineWorld must nevertheless remain presentation-independent, and community creators must be free to provide entirely different Presentation Style Packs without changing world simulation semantics.

---

# 23. Authoring a Presentation Style

*Added 2026-09-25, after §§1–22, as the creator-facing flow this format exists to serve.*

Creating a style must not require writing a manifest by hand. The intended flow is:

```text
New Presentation Style
   → upload 4–10 reference images you like
   → write two or three sentences
   → choose 2D / 3D
   → the system analyses them and generates an initial style manifest
   → the creator corrects anything the analysis got wrong
```

The generated manifest is a **starting point the creator owns**, not an authority the analysis
imposes. References remain the source of truth (§16), and the manifest remains the compact
machine-readable summary of them.

## 23.1 What this requires of the schema

If a manifest is generated from images, every field must be answerable from images plus a short
description. That splits the schema into three kinds of field, and the split is a design
constraint, not a documentation detail:

| Kind | Examples | Where it comes from |
| --- | --- | --- |
| **Derived** | `realism`, `stylization`, `detail_level`, `atmosphere`, `environment_density`, `characters.proportions`, `characters.facial_detail`, `materials.texture_detail`, `lighting.default`, `lighting.contrast`, `lighting.preferred_conditions` | inferred from the reference images, then confirmed by the creator |
| **Chosen** | `id`, `name`, `dimension` | stated by the creator; `dimension` is the 2D/3D selection in the flow |
| **Policy** | `environment.readability_priority`, `environment.human_scale` | **not** style at all. Gameplay readability outranks decorative density for every MineWorld style (§4, `ENGINEERING_RULES.md` §4), so these are project invariants that a style restates rather than chooses |

A field that cannot be inferred, chosen, or fixed by policy does not belong in the manifest.
This is the concrete form of §15's warning against an exhaustive universal art ontology: the
schema stays small because generation keeps it honest.

## 23.2 What the analysis must not do

It may not infer anything about simulation. A reference image showing a café with people sitting
outside says nothing about whether `Sit` exists in a world — that is a System question (§19). The
analysis reads style and only style.
