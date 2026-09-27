# Candidates — default 2D style

**Nothing in this directory is an asset.** Under
[`ARC-9`](../../../../docs/DECISIONS.md) a generated output is a *candidate*; it becomes an asset
when a person accepts it:

```text
generate candidates  →  human selection  →  cleanup and normalization  →  packaged asset
```

These are the generate step. They are kept separate from any approved asset directory on purpose,
and every `*.provenance.yaml` here records `approval.status: candidate` and
`generation.human_curated: false`. The tooling never changes either field — promotion is a human
act.

## What is here

Generated from the four committed style references in [`../references`](../references) with
`tools/asset_generation`, using reference-conditioned generation (the references are sent with
every request; the prompt asks for the *style*, never the content).

| Group | Assets |
| --- | --- |
| `building_*` | café, bakery, bookshop, flower shop, cottage, townhouse, apartment block |
| `tree_*` | four tree types, a shrub cluster, a flower bed |
| `person_*` | four ordinary modern people, each as a 2×2 four-direction sheet |
| `prop_*` | bench, street lamp, café parasol table, planter, signpost, litter bin, bicycle |
| `ground_*` | pavement slabs, stone path, grass, plaza cobble — opaque, intended to tile |

`contact_sheet.png` shows the whole set over a checkerboard, which is the quickest way to judge
the cutouts.

Everything except `ground_*` has a real alpha channel and has been normalized (alpha peak raised
to 255, edge colour bled 8px outwards so scaling does not produce a dark fringe). See
`tools/asset_generation/README.md`.

## What was filtered, and what was not

Automated filtering removed only objectively broken output — failed transparency, corrupted
anatomy, stray lettering, wrong projection, gross style mismatch. **Which style variant is right
was not decided here**; that is the operator's call at the integrated-scene level, so plausible
alternatives were kept rather than pruned.

## Known limitations — measured, not guessed

**Characters are the weak result.** `person_*` sheets are internally consistent — the same
clothing, hair and palette across all four views — but they do **not** match the reference
characters. They are drawn at eye level rather than from the isometric camera, with much heavier
outlines and larger heads than the small, finely-drawn townspeople in
[`../references`](../references). Buildings, props and vegetation match the references closely;
characters need rework before they can share a scene with them. This is the one group that should
not be promoted as-is.

**Nothing is on a shared scale.** Each asset was generated to fill its own 1024×1024 frame, so
the conifer and the apartment block occupy the same pixel height. Relative sizing is a
normalization step still to be done; the generator got it wrong by construction, not by accident.

**Some assets touch the frame edge** and may be clipped: `prop_bicycle` (96% fill, left+bottom),
`building_bakery` (95%, left+bottom), `tree_shrub_cluster` (94%, left). Regenerate with explicit
margin if a full silhouette matters.

**Ground tiles are only approximately seamless.** Measuring opposite-edge difference against the
natural adjacent-pixel baseline for each texture:

| Tile | Horizontal seam | Vertical seam | Baseline | Verdict |
| --- | --- | --- | --- | --- |
| `ground_grass` | 8.7 | 9.5 | 7.2 | effectively seamless |
| `ground_pavement_slabs` | 6.3 | 10.4 | 8.7 | acceptable |
| `ground_stone_path` | 8.2 | 12.2 | 7.7 | mild vertical seam |
| `ground_plaza_cobble` | 8.8 | **30.6** | 6.6 | **visible horizontal band when tiled** |

`ground_plaza_cobble` needs a seam-healing pass or regeneration before it is tiled.

**Projection drifts slightly between assets.** The references sit at a shallower isometric
elevation than some candidates; anything used together in one scene needs checking against a
common angle.

**`building_bakery` used a slightly different prompt** from the rest — it was the single-asset
proof generated before the recipe existed. Its sidecar records what was actually used.

**Character sheets are one image containing four views**, not four separate sprites. Splitting
them into directional frames is still to be done.
