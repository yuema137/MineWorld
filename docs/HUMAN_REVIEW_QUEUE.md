# Human review queue

**Only large integrated milestones belong here.** Not individual PRs, structs, components, events,
dependency choices or refactors — the pi-agent owns routine engineering review. A milestone earns
a place here when a coherent capability can be *experienced* as a whole.

Engineering progress is tracked separately in [`MVP_STATUS.md`](MVP_STATUS.md). A milestone parked
here does not block anything else.

**Updated:** 2026-09-27

---

## Framework milestones

| | Milestone | Demonstrates | State |
| --- | --- | --- | --- |
| **A** | Runnable world runtime | load Social Café → server → 2D + 3D clients → cause a change → both observe it → restart → state survives | 🚧 in progress (05c, 05d) |
| **B** | Persistent people and social life | Alice and Bob persist, know each other, share an activity, and survive a restart with their history | ❌ |
| **C** | Objects and everyday economy | Market Town: work → earn → buy → inventory changes → another client sees it → persists | ❌ |
| **D** | LM-native persistent characters | speak to Alice in 2D, meet her in 3D, and she reacts consistently with what happened | ❌ |
| **E** | Package composition | a real world assembled from independently installable packs | ❌ |

## Default-style milestones — taste, and the operator decides

These are candidates until the operator says otherwise (`ARC-11`). An agent may build and
recommend; it may not declare something the default look.

Named per `ARC-20`, because with two 3D tracks running (`ARC-18`) "the 3D character" no longer
identifies one thing.

| | Milestone | State |
| --- | --- | --- |
| **VIS-2D-1** | Playable 2D default scene with an enterable interior | 🚧 candidate in progress |
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | ❌ **failed the agent's own fidelity gate, twice** — rebuilt, still not the same person; see below |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | ❌ |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

`VIS-3D-GODOT-1` first failed on categorical identity mismatch, not polish: the reference is a
young woman in an open burgundy zip hoodie and the candidate was a man in a red quilted puffer
jacket (`ARC-17`).

**It has since been rebuilt and it fails again, and the agent is reporting that rather than
submitting it.** The wardrobe now matches the contract fact by fact — open burgundy zip hoodie
with a hood, cream drawstrings, ribbed cuffs and hem, two panels with zip tape, the cream tee with
its legible mountain-and-slogan print, worn mid-blue denim, the backpack with her hand on the
strap — and the *person* still does not. Five hard-fail categories are wrong: hairstyle category,
face shape, expression, gaze and skin tone. A viewer shown the two side by side would say *same
clothes, different person*, so `VISUAL_FIDELITY.md` §9 forbids requesting review.

The per-category verdict, the evidence and what would close each failure are in
[`presentation/mineworld-default/3D/FIDELITY_REVIEW.md`](../presentation/mineworld-default/3D/FIDELITY_REVIEW.md);
the runtime frames are in `presentation/mineworld-default/3D/candidate/`. Reproduce them with
`./mineworld-3d --portrait`.

**One decision is the operator's and blocks the largest remaining failure.** The CC0 Vitruvian
head is one head, and CharMorph's morphs reshape the body far more than the skull. Reaching this
face means either searching the face morph sliders, or accepting the base mesh cannot get there
and replacing the head — which `ARC-19` §3 explicitly permits, while forbidding only that we build
character-reconstruction technology ourselves. More effort on the current head should wait on
that.

The rig, retarget, animation, cadence, footwear, ground-contact, garment-modelling, hair-modelling
and texture work underneath it is unaffected and is kept.

### What a review package contains (`ARC-20`)

Milestone id · what changed · **the exact launch command** · real runtime screenshots · the
canonical reference · a side-by-side where applicable · known limitations · the specific
subjective questions being asked. The operator must be able to launch, look, walk and judge
quickly. An engineering log is not a review artefact.

### Reaching review does not stop work

On `READY FOR HUMAN VISUAL REVIEW`: preserve the runnable candidate, save the screenshots, record
it here, **stop subjective polishing on that branch**, and move to independent work. Review is a
branch-level checkpoint, never a global barrier. Neither 3D track waits on the other, and none of
the framework milestones wait on any of them.

## Style infrastructure — architecture, and it never waits here

Tracked in [`MVP_STATUS.md`](MVP_STATUS.md), listed only so the split is visible: Presentation and
Asset Pack interfaces, style manifest schema, provenance, generation-pipeline integration,
renderer bindings, style switching, validation and composition. None of it blocks on a taste
decision, and none of it may be tied to whichever style happens to be default.

---

## Open questions for the operator, not blocking anything

- **The project's name.** Microsoft published an unrelated "MineWorld" in 2025 — a video-generative
  world model — with a paper, a repository and a Hugging Face presence. Purely a discoverability
  and package-naming collision (`ARC-12`), and it blocks no engineering. Worth a deliberate
  decision before public launch rather than discovering it in a search result.

## Accepted, not to be re-litigated

- **The 2D and 3D presentation direction**, as of 2026-09-27. Movement, camera work and the
  3D lighting rig are accepted; the visual track now improves fidelity toward the references
  rather than re-deciding the direction.
- **Procedural SVG is retired as the 2D art strategy**, keeping its layout, placement, projection,
  camera and occlusion logic. Visuals get replaced; architecture does not.
- **Primitive humanoids are below baseline** and are being replaced by a reusable rigged pipeline.
- **The default character is an identity reconstruction of `04_character_closeup.png`**
  (`ARC-19`), which supersedes `ARC-4`'s facial-fidelity exclusion for that one image. `ARC-4`'s
  reasoning survives, applied to the right object: the bar is high for **our** default character,
  which we pay once and ship as an asset, and stays low for **what the framework requires**, which
  is the humanoid profile and nothing about appearance.
