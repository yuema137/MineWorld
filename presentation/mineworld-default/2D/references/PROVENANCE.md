# Provenance — default 2D reference images

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

C2PA creation time is the `c2pa.created` action's `when`, in UTC. All four were first committed in
`8978cee` on 2026-09-25 (author's local time, UTC−7, hence one day earlier than the C2PA times).

| File | C2PA created (UTC) | SHA-256 |
| --- | --- | --- |
| `01_town_square.png` | 2026-09-26 03:07:57 | `b3972e811123fb820a766f599ef2f02bb4da41b028fd3007eddcf2f53fdb1793` |
| `02_cafe_street.png` | 2026-09-26 03:07:56 | `d3d1386fd930fe361288648c8c09cdc42cf366653d7a4bb6e439e034ee3bdfeb` |
| `03_harbour_cafe.png` | 2026-09-26 03:07:57 | `f1fc216443655e661ab35ef02c85476b525140785aae8094947404e2b6086c0f` |
| `04_lakeside_park.png` | 2026-09-26 03:07:56 | `ad483723363f0c601989db46d140d7033cd3771c0fb69579cb16789de4edcff3` |

## 3. Downstream use

These images are attached as style references to the generations under `../candidates/`; each
candidate's `.provenance.yaml` lists the references it used by path and SHA-256 digest, matching
the table above. A candidate is a new OpenAI Output with its own record, not a copy of a reference.
Crops of these images made as style references on spike branches (`clients/2d-spike/art/style_refs/`
on `vis/2d-generated-assets`) inherit this record.
