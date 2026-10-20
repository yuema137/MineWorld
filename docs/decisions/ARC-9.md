## ARC-9 — Generated assets are first-class candidates, never automatic content

**Date** 2026-09-26

MineWorld does not have to source its look from existing asset libraries, and for the default
style it largely cannot: the research behind `DEP-8` found no open-licensed semi-realistic
building kit, no CC0 clothed and rigged modern people, and nothing illustrated that matches the
2D references. Bending the art direction to fit what happens to exist would be the wrong
trade — the direction is the thing being protected.

So generation is accepted and encouraged where it reaches the intended style faster. The rule
that makes it safe is the pipeline, not the source:

```text
generate candidates  →  human selection  →  cleanup and normalization  →  packaged asset
```

**A generated output is a candidate. It becomes an asset when a person accepts it.** That
selection step is not ceremony: `DEP-8`'s coherence procedure — one scale, one material
authority, one lighting rig — is exactly what a pile of individually plausible generated objects
will otherwise fail.

### Where generation fits, by medium

2D takes it well: scenes, buildings, ground and wall textures, vegetation, background character
looks, UI and icons. 3D takes it unevenly: style references, PBR textures, props and building
concepts are productive; meshes usually need cleanup; **rigged characters and reusable animation
are where one-shot generation does not yet land**, which is why `DEP-8` records a character
pipeline rather than a shopping list.

### Provenance is the price

Anything entering the repository, a default pack, or anything publishable carries its origin:

```yaml
source_type: generated
generation:
  tool: <tool>
  model: <model and version>
  date: 2026-09-26
  prompt_summary: "semi-realistic modern outdoor bench…"
  human_curated: true
postprocess: { cleaned: true, retopology: false, texture_adjusted: true }
license: { redistribution_allowed: true }
```

The question is never *"was this AI-generated?"* It is **"do this asset's terms permit
commercial use, modification, redistribution, and bundling into an open-source project?"** —
which is the same question `DEP-8` asks of a downloaded asset, and the same ambiguity the Mixamo
entry above resolves conservatively.

### Two layers, deliberately different standards

| Layer | Examples | Standard |
| --- | --- | --- |
| **Official / shareable** | this repository, default packs, published DLC, a future registry | redistributable, modifiable, bundleable, provenance recorded |
| **Private / local** | a player's own world, a private server, an unpublished mod | the player's own business |

MineWorld should not police what someone generates for their own machine. It must be strict
about what it hands to someone else.

### Current practical constraint, stated so it is not mistaken for policy

No image or model generation tool is wired into the build agents, so today generated assets enter
through the operator, who produced the existing reference images that way. The pipeline above is
the standing policy; the operator is currently its generation step.

### Roadmap

```text
MVP-0   generated reference images and textures; placeholders elsewhere
MVP-1   provenance fields and asset validation in the pack format
MVP-2   reference-image-first authoring: upload 4-8 images, get a style manifest
        and candidate assets back as a pack draft
```

MVP-2 is the same flow `ART_DIRECTION.md` §23 already describes for style manifests, extended
from describing a style to producing the assets that embody it.

## ARC-9 note — `source_type: rendered` (2026-10-19, S26-c1)

A third origin beside `generated` and downloaded: an image **rendered by the project from one of its own
assets**. The first are the default 2D pack's cast sprites (`presentation/mineworld-default/2D/art/cast/`),
rendered in Blender from the same rigged 3D characters the 3D client draws (operator ruling 2026-10-19:
2D uses these realistic renders; this replaces step-26 §9's image-generation method for the cast, see
`.structured-coding/plans/mvp0/pr-s26c1-cast-client.md` §3 D-5). Its sidecar carries, instead of a
`generation` block:

```yaml
source_type: rendered
render:
  source: <what was rendered>
  character: <character id>
  source_file: <file name>
  source_sha256: <sha256 of the rendered file>      # the exact asset version, so a re-export is detectable
  source_bin_sha256: <sha256 of its buffer>
  tool: <renderer and script>
  date: <date>
  camera: { projection, elevation_deg, yaw_from_walk_deg, view, faces }
  pose: { stride_deg, frame, lead_leg }
  lighting: <summary>
postprocess: { supersample, downsample, outline_px, normalized, height_m, pixel_height, anchor, size }
human_curated: false
license: { redistribution_allowed: true, note: inherits the source asset's provenance }
```

The terms are the source asset's: a render adds no third-party input. The source's own provenance (for a
cast character, `scenes[0].extras.mineworld_provenance` in its glTF, `source_type: generated`) stays the
authority for how the subject was made; the sidecar's hashes tie each image to that exact source.
