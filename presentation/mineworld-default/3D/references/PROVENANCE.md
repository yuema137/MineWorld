# Provenance — default 3D reference images

`ARC-9` provenance record for every image in this directory. Recorded 2026-10-08 from two sources
only: the C2PA manifest embedded in each PNG (read with `exiftool -a -G1`) and the repository
history. Nothing here is reconstructed from memory; the prompts were not recorded at generation
time and are therefore not given.

## 1. Origin and terms

| Field | Value |
| --- | --- |
| Source type | AI-generated (`trainedAlgorithmicMedia`), operator-produced in ChatGPT, then selected by the operator as style references (human-curated) |
| Generator, per C2PA | software agent `ChatGPT`, version `gpt-image`; claim generator `OpenAI Media Service API` (c2pa-rs 0.79.2, C2PA spec 2.2.0); actions `c2pa.created`, `c2pa.converted`, `c2pa.watermarked.unbound` |
| ChatGPT account tier | not recorded in the metadata; immaterial, because the consumer Terms of Use and the Services Agreement both assign Output to the user |
| Terms relied on | OpenAI Terms of Use, effective 2026-01-01, "Ownership of content" (assignment of Output), read first-hand 2026-10-08 — [`docs/DECISIONS.md`](../../../../docs/DECISIONS.md) `DEP-8`, "Generated images (OpenAI), recorded 2026-10-08" |
| Licence in this repository | owned output, distributed under the repository's MIT licence |
| Obligations kept | disclosed as AI-generated here and in the top-level `NOTICE`; not used to train any model; the signed C2PA manifests are kept in the committed PNGs — do not strip them when re-saving |

## 2. Per image

C2PA creation time is the `c2pa.created` action's `when`, in UTC. The commit is the first one
that added the bytes on any branch.

| File | C2PA created (UTC) | First committed | SHA-256 |
| --- | --- | --- | --- |
| `01_lakeside_promenade.png` | 2026-09-26 03:00:38 | `8074962`, 2026-09-25 (as `presentation/mineworld-default-realistic/references/lakeside_01_style_c.png`; renamed unchanged here in `8978cee`) | `1f1669e865162fe34b3d6cc9ad48737b169b6f568b880730d22388233baafd04` |
| `02_town_square.png` | 2026-09-26 03:13:10 | `8978cee`, 2026-09-25 | `d4d8271b62d669d64b467c91acd21a7726058303398cc105603aa409d332ddb1` |
| `03_cafe_frontage.png` | 2026-09-26 03:12:53 | `8978cee`, 2026-09-25 | `d4b307b10c8a20e2b0e4bc88996a3835b156a955e67f9d7cc5ca1de418d7ff62` |
| `04_character_closeup.png` | 2026-09-26 03:12:52 | `8978cee`, 2026-09-25 | `5b0c8f0b818a9ca7555515e7374f2ec40770a09bc990f8c568b6dc913e41a3a1` |
| `05_main_street_golden_hour.png` | 2026-09-26 03:12:58 | `8978cee`, 2026-09-25 | `a6eb7af50805bec8f54d0ea2b93ce8b4a30a9eb7ed761146f7916b6c1e893d6f` |
| `06_lake_trail.png` | 2026-09-26 03:12:52 | `8978cee`, 2026-09-25 | `509d1b827a72f342518197dc53bd7bbd2fa3ad72c015ac75f2222f561075b497` |

Commit dates are the author's local time (UTC−7), which is why they read one day earlier than the
C2PA times.

## 3. `04_character_closeup.png` is upstream of a shipped asset

Besides being the identity source of truth for the default character (`ARC-19`), a masked crop of
this image is the **input** to the Meshy image-to-3D generation that produced
`clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` and `meshy_d_base_color.png`
([`../../LICENSES/MESHY_ROUTE_D_CHARACTER.txt`](../../LICENSES/MESHY_ROUTE_D_CHARACTER.txt)). That
asset's chain is therefore: this record (OpenAI Output, owned) → Meshy paid-plan Output (owned,
`DEP-8` "Generated meshes") → MineWorld post-processing.

## 4. Derived images

These are crops or composites that contain pixels of a reference image. They are re-encoded JPEGs
without the C2PA manifest, and this record is their provenance:

| File | Derived from | First committed |
| --- | --- | --- |
| `../candidate/00_reference_chest.jpg` | crop of `04_character_closeup.png` | `4121d86`, 2026-09-27 |
| `../candidate/side_by_side_chest.jpg` | reference crop beside a render | `4121d86`, 2026-09-27 |
| `../candidate/side_by_side_head.jpg` | reference crop beside a render | `4121d86`, 2026-09-27 |
| `../candidate/side_by_side_rear.jpg` | reference crop beside a render | `eb62dc3`, 2026-10-06 |
| `../candidate/route_d/d3_side_by_side.jpg` | reference crop beside a render | `d1fcbc1`, 2026-10-07 |

Any other evidence image that embeds a crop of a reference inherits this record in the same way.
