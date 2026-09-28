# PR 01a — the `VIS-3D-GODOT-2` environment slice

## DESIGN FROZEN

```text
Design revision       vis-3d-godot-2/pr-01a, sections 1-6
Approved by           operator brief of 2026-09-27 delivered to this session, which states the
                      scope, the two declarable states, the review package, the concurrency
                      boundary with vis/3d-human-pipeline, and the standing instruction to
                      "commit and push after each meaningful completed step"
Implementation base   main @ 5b99840, branch vis/3d-godot-2-environment (own worktree)
Execution contract    pr-01a-contract.md
Lifecycle             IN EXECUTION
```

**Combined step/PR document.** Steps 02–09 of [`overall.md`](overall.md) are one PR executed as a
sequence of semantic commits. Step 01 (`docs/VISUAL_SLICE.md`) is the first commit of it.

---

## 1. Goal and scope

Build the slice specified in [`docs/VISUAL_SLICE.md`](../../../docs/VISUAL_SLICE.md) inside the
existing `clients/3d-spike` Godot project, as a new scene with its own launcher, and reach
`VIS-3D-GODOT-2 ENVIRONMENT: TECHNICALLY READY` with recorded evidence.

**In scope:** the slice scene and its construction code; a material and lighting pass; the café
interior; the protocol binding and semantic `Place` identity; the character slot; the capture and
measurement harness; asset acquisition with `DEP-8` records; the review package.

**Out of scope:** everything in [`overall.md`](overall.md) §3.

## 2. Invariants

1. **No file owned by `vis/3d-human-pipeline` is edited.** Named in `overall.md` §3.
2. **The existing promenade scene keeps working.** `./mineworld-3d`, `--drive` and `--shots`
   behave as they do on `main`.
3. **One world unit is one metre**, and no node carries a non-uniform import scale
   (`DEP-8`, coherence procedure step 1).
4. **No world rule is evaluated in the client** (`ENGINEERING_RULES.md` §§4, 7–9).
5. **Identities stay strings** (`clients/protocol/ADOPTION.md` §3.1).
6. **No asset enters the repository without a recorded relicensing answer** (`DEP-8` as amended).
7. **No fidelity claim without a pixel comparison at the reference's own framing** (`ARC-17` §6),
   and none of the phrases prohibited by `VISUAL_FIDELITY.md` §6.
8. **`ACCEPTED` is never written by this branch** (`ARC-11`).

## 3. Source audit

Read in this session before any edit:

| Path | Finding |
| --- | --- |
| `clients/3d-spike/project.godot` | Forward+, 1600×900, 2048 directional shadows, SSAA; no SDFGI, no lightmapper settings, no reflection probes |
| `clients/3d-spike/scripts/main.gd` | the `ARC-13` rig: one HDRI sky at 0.80, one sun at 17°/2.9 energy, a cool fill, AgX at 1.06, SSAO on, **SSIL off**, glow, depth fog, saturation 1.16 |
| `clients/3d-spike/scripts/mats.gd` | world-triplanar PBR from Poly Haven `diff`/`nor_gl`/`arm` triples; `pbr()`, `paint()`, `glass()`, `clear_glass()`, `emissive()`, `card()`; 11 named materials |
| `clients/3d-spike/scripts/build.gd` | `box` `slab` `cyl` `prism` `sphere` `ground` `card` `label` `blocker` `box_blocker`; collision is opt-in per call |
| `clients/3d-spike/scripts/interior.gd` | an enterable shell already exists: pieced front wall, cafe and shop fit-outs, local omni lights at energy 6.5. **Ceiling is a blank plane; the back wall is bare; there is no display case, no glassware, no hand-scale object.** This is the largest single gap |
| `clients/3d-spike/scripts/player.gd`, `camera_rig.gd` | one controller, three cameras, boom with one collision ray; mode switch preserves state |
| `clients/3d-spike/scripts/shots.gd` | 24 scripted viewpoints and a headless `--drive`; capture writes `res://shots` |
| `clients/3d-spike/screenshots/21_street_golden.jpg`, `17_cafe_counter.jpg` | the measured baseline. See §4 |
| `clients/protocol/mineworld/` + `ADOPTION.md` | `MineWorldClient`, `MineWorldObservation`, `MineWorldSpace`; copy or symlink into the project; four rules a client must not break |
| `mineworld-3d` | the launcher; rebuilds the class cache when a script is newer |

### 3.1 The measured baseline, reference fact against candidate fact

Largest miss first, from `21_street_golden.jpg` against `05_main_street_golden_hour.png` and
`17_cafe_counter.jpg` against the interior visible in `03_cafe_frontage.png`:

```text
INTERIOR CONTENT  reference: a back wall of full-height timber shelving carrying jars, bottles,
  ceramics and plants; a glass pastry case with two loaded shelves; a framed chalkboard menu of
  handwritten lines; six pendants at three heights; potted plants on the counter.
  candidate: a bare brick wall, one green box counter, one grey box, a 12-character label
  reading TODAY, three cone pendants, and no object smaller than a chair.

OPENINGS          reference: every window has a reveal, a sill and a head, and the shopfront has
  pilasters, mullions, a transom, a moulded cornice and a dentil course.
  candidate: flat rectangles cut in a tiled brick plane, a flat green fascia with no cornice and
  no lettering at all.

GROUND            reference: pale grey-buff setts roughly 0.30 x 0.20 m in stretcher courses with
  a granite kerb, per-unit tonal variation and joint shadow.
  candidate: one flat orange plane with a 1.8 m concrete tile and no kerb read.

CEILING           reference: exposed warm timber planking running front to back.
  candidate: an untextured pale grey plane.

VEGETATION        reference: street trees with layered foliage, window boxes, hanging baskets,
  trailing ivy, planters at three sizes.
  candidate: four card-blob trees and no frontage planting.

BACKDROP          reference: a forested ridge and a church spire in warm haze.
  candidate: pale angular polygonal shards.
```

## 4. Acceptance

The nine objective gates of the operator brief, each with evidence:

```text
G1  the slice launches reliably, from a clean checkout, with no missing resource
G2  the cafe is physically enterable on foot
G3  the interior is fully navigable; VISUAL_SLICE.md §6.3 passes
G4  collision is correct: no pass-through, no fall-through, no escape but the doorway
G5  real materials: PBR maps, correct texel density, no untextured architectural surface
G6  the lighting pipeline is active, and the GI choice is recorded with measurements
G7  reflections and environment lighting work indoors and out
G8  scale is objectively checked against VISUAL_SLICE.md §3
G9  the three camera modes work indoors and out and preserve state across a switch
G10 performance is reasonable, measured
G11 asset provenance and licences recorded, with the DEP-8 relicensing question answered
G12 MineWorld integration works: a real intent round-trip against the real server
G13 the character slot is stable
```

## 5. Commit plan

Each commit: implementation, deterministic validation, LLM logic review — separately checkable.
Each is pushed on completion.

### C1 — the slice specification
- [x] Implementation: `docs/VISUAL_SLICE.md`, engine-neutral; `overall.md`; this document; contract.
- [x] Validation: grep the document for engine vocabulary. Evidence in §7.
- [x] Review: every operator-named scope element appears as a requirement; nothing Godot-specific.

### C2 — scene skeleton, launcher, capture and measurement harness
- [ ] Implementation: `scripts/slice/*.gd`, `scenes/slice.tscn`, root launcher, capture views,
      a scale probe that prints measured dimensions, a luminance probe.
- [ ] Validation: launch from a clean checkout; walk the street end to end under `--drive`.
- [ ] Review: no shared file edited beyond the launcher; the promenade scene still builds.

### C3 — café exterior architecture
- [ ] Implementation: profiled shopfront, signage, masonry with real openings, awning, brackets.
- [ ] Validation: measured against `VISUAL_SLICE.md` §3.2; captured and compared to
      `03_cafe_frontage.png` at that reference's framing.
- [ ] Review: massing and silhouette checked against `VISUAL_FIDELITY.md` §5 hard-fails.

### C4 — street, secondary façades, ground, furniture, planting
- [ ] Implementation: `VISUAL_SLICE.md` §4 in full.
- [ ] Validation: no repeated façade unit; frontage length and widths measured.
- [ ] Review: the out-of-scope list in §4 has not been crossed.

### C5 — material authority and the lighting pipeline
- [ ] Implementation: material re-authority pass, texel density, sky, exposure, probes, GI.
- [ ] Validation: the GI comparison, with frames and numbers for each candidate technique.
- [ ] Review: no technique used merely because it exists; the decision is recorded.

### C6 — the café interior
- [ ] Implementation: `VISUAL_SLICE.md` §6 in full, with collision.
- [ ] Validation: the walk-in loop, clearances, three object sizes, counted.
- [ ] Review: nothing in the room is a billboard or a projected image.

### C7 — threshold verification
- [ ] Implementation: the four-sample measurement path and its report.
- [ ] Validation: `VISUAL_SLICE.md` §7.1, as numbers.
- [ ] Review: thresholds stated before the run, not chosen to fit the result.

### C8 — runtime integration and the character slot
- [ ] Implementation: protocol module adopted, `Place` identity, intent path, character slot.
- [ ] Validation: an intent round-trip against the running server.
- [ ] Review: `ADOPTION.md` §3's four rules, checked one by one.

### C9 — the review package
- [ ] Implementation: the §12 views, `ASSETS`/licence records, limitations, queue entry.
- [ ] Validation: every view captured from the running client at its normal settings.
- [ ] Review: it is a review package and not an engineering log (`ARC-20` §3).

## 6. Test ownership

| Failure class | Owner |
| --- | --- |
| a script that does not parse, a missing resource, a broken path | launch of the slice; the class cache build |
| collision, navigability, camera state across a switch, walking speed | the scripted drive run |
| scale | the scale probe, printed and recorded |
| exposure across the threshold, interior legibility, clipped exterior | the luminance probe, printed and recorded |
| visual fidelity to a reference | pixel comparison at the reference's framing; never a test |
| taste | **the operator.** Not testable and not the agent's (`ARC-11`) |

Rust `cargo` layers are **N/A** for this PR until C8 touches a Rust path; C8 runs the server and
therefore uses the existing workspace build.

## 7. Evidence log

### C1 — 2026-09-27

```text
$ grep -niE 'godot|unreal|forward\+|lumen|nanite|sdfgi|lightmapgi|ssao|ssil|reflectionprobe|
    volumetric|shader|tonemap|node|gdscript|blueprint|\.tscn|\.uasset' docs/VISUAL_SLICE.md
```

Result recorded with the commit. Occurrences of `Godot` and `Unreal` are permitted only where the
document names the two **tracks** (`VIS-3D-GODOT-2`, `VIS-3D-UE5-1`) rather than a technique; any
other hit is a defect in the document.

## 8. Deviations and discoveries

None yet.
