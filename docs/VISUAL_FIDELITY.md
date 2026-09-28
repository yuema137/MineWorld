# Visual fidelity: how a reference is compared, and when a candidate may be shown

This specification governs every visual deliverable that declares a reference — characters,
buildings, interiors, vegetation, lighting, whole scenes — in both the 2D and the 3D reference
clients. It exists because a candidate was brought to human review described as
`"same wardrobe, recognisably not the same person"` when the reference was a young woman in an
open burgundy zip hoodie and the candidate was a man in a red quilted puffer jacket. Nothing in
that sentence was false. It was still the wrong verdict, and it cost a review round.

Read this with [`ACCEPTANCE.md`](ACCEPTANCE.md), which decides *who owns* a judgement. This
document decides *how a visual comparison is performed and reported* once the agent owns it.

---

## 1. A reference is the source of truth, not an inspiration

When a deliverable declares a reference image, the task is:

```text
recreate this specific visible thing, as faithfully as practical
```

It is **not**:

```text
create a believable thing inspired by the reference
```

A result that is attractive and reads as something else has failed. This is the whole distinction,
and every rule below follows from it.

The reference governs **appearance**. It governs nothing about runtime contracts: the
`SkeletonProfileHumanoid` rig, the spatial frame, the animation contract, scale and axes, and glTF
export expectations are standardised by the contracts and by the humanoid profile specification
(`docs/HUMANOID_PROFILE.md`, arriving with `vis/3d-human-pipeline`), and they are identical across
characters who look nothing alike. Standardising the
skeleton is not standardising the face:

```text
same skeleton  ≠  same mesh  ≠  same face  ≠  same clothes
```

Many different people share one humanoid profile and one animation library while differing
completely in body mesh, face, hair, clothing, textures and accessories. That is the intended
shape, and it is what makes reference fidelity a per-character concern rather than an engine one.

## 2. Requirements flow from the reference to the asset, never back

```text
reference  →  identity-defining constraints  →  choose, generate or customise assets that satisfy them
```

Never:

```text
available asset  →  material and colour edits  →  declare approximate match
```

Choosing a humanoid, a building kit or a tree set because it is conveniently to hand and then
redefining the reference around what it happens to provide is the failure this rule names. An
asset that cannot satisfy a constraint is evidence about the asset, not a reason to relax the
constraint.

Where an existing asset genuinely can satisfy the constraints, use it —
[`REUSE_POLICY.md`](REUSE_POLICY.md) is unaffected, and reuse remains preferred. The rule is about
which direction the specification travels.

## 3. The identity contract is written before the modelling

Before building a referenced deliverable, read the reference **as an image** and write its contract
as a specification document, derived from the pixels and not from any earlier prose description —
including a brief, including one written by a reviewer. A description of a description is how
detail is lost.

For a character the contract covers at least:

| Group | What it must record |
| --- | --- |
| Body | perceived gender presentation, apparent age, proportions, shoulder width, torso/hip balance, overall silhouette |
| Face | face shape, jaw and chin, skin tone, freckles or other markings, eye and brow impression, mouth and expression |
| Hair | colour, tied or loose structure, volume, stray strands, silhouette from front, side and back |
| Clothing | each garment by **category** and construction, not only by colour; graphics and text; footwear type and colour |
| Accessories | anything carried or worn that a viewer would use to recognise this person |
| Identity | what makes this one recognisable rather than generic |

For a place, the same shape applies to massing and silhouette, roofline, materials and weathering,
fenestration, signage, planting, and the lighting condition the reference depicts.

## 4. Identity-defining features are never deferred

A graphic on a shirt, a backpack strap, a hood, a hairstyle, a stepped roofline: on a referenced
deliverable these are not decoration. They are what a viewer matches against.

Deferring them is legitimate when the milestone is *prove the skeleton loads* or *prove a room is
enterable*. It is not legitimate once the milestone is named for the reference. A milestone called
*reference-matched character* is not satisfiable with the recognition cues switched off.

## 5. Hard-fail categories — the agent rejects these itself

A candidate must not reach human review with any of these fundamentally wrong. These are automatic
failures, not partial matches:

```text
perceived gender presentation        outer-garment category
apparent age                         a missing defining garment structure
body silhouette                      grossly wrong face shape
hairstyle category                   missing identity-defining accessories
major hair colour                    massing or silhouette of a referenced building
```

Worked examples, each an automatic failure:

```text
open zip hoodie          → quilted puffer jacket
brown voluminous updo    → black close-cropped hair
young woman              → adult man
stepped varied roofline  → one flat continuous parapet
```

## 6. How a comparison is reported

Per category, with the reference fact and the candidate fact beside each other, and a verdict:

```text
Outer garment FAIL: reference is an open burgundy cotton zip hoodie with hood,
  drawstrings and ribbed cuffs; candidate is a red quilted puffer jacket with a
  collar and no hood.
```

**Prohibited**: `broadly similar`, `roughly matches`, `approximately right`, `close to the
reference`, `same wardrobe`, and every other comparative that names no fact. They permit
self-persuasion, and a reader cannot act on them.

List the **largest** miss first, in the words a viewer would use.

## 7. The loop

```text
reference image
      ↓
implementation
      ↓
runtime screenshot from the real client
      ↓
vision-based side-by-side comparison, at the reference's own framing
      ↓
reject, or iterate
```

Both sides are inspected **as pixels**. A comparison conducted between two written descriptions is
not a comparison.

## 8. Evidence at the resolution of the claim

A comparison must be rendered at a scale where the claim is decidable. This project has produced
the same defect three times: a face judged at 60 px, a character judged in street-distance
screenshots, and a review montage at 620 px per tile that was read as *nothing changed* when both
the footwear and the roofline had in fact changed.

This is the image-specific case of a general rule: `DECISIONS.md` `ARC-23` — **locate before
counting** — which records seven instances of an instrument measuring something adjacent to what
it was named after, twice coming within a step of a wrong architectural conclusion.

So: compare at the reference's own framing. If the reference is a chest-up portrait, the candidate
crop is chest-up. If the claim is about a sole touching the ground, the crop is the foot. A contact
sheet is for orientation, never for a verdict.

## 9. The self-review question

Before requesting human review:

> Shown the reference and the runtime screenshot side by side, would a person who had never seen
> this project reasonably recognise them as the same character — or the same street?

If the answer is clearly no, **do not request human review.** Report the failure and what it would
take to fix, which is more useful than a candidate that will be rejected.

## 10. What a review submission contains

- the canonical reference;
- a front or three-quarter view from actual gameplay;
- a rear view;
- a closer view at the reference's framing;
- normal environmental lighting, not a flattering one-off;
- a genuine side-by-side sheet.

## 11. Ownership

The agent owns objective pipeline correctness and owns rejecting obvious fidelity failures under
§5. **Final visual acceptance belongs to the operator** ([`ACCEPTANCE.md`](ACCEPTANCE.md),
[`DECISIONS.md`](DECISIONS.md) `ARC-11`): nothing becomes an official default until they say so.

A rejection resets the visual candidate. It does not reset the engineering underneath it, and it
never blocks runtime, server, systems, persistence, networking, package interfaces, style
infrastructure or enterable-building work.

## 12. One reference image does not show everything

Visible appearance must match closely. Unseen geometry is plausible reconstruction, and saying so
is required rather than optional. Never invent a visible feature that contradicts the reference.

Generating additional consistent views from the canonical image to aid modelling is an acceptable
intermediate production technique under `ARC-9`. The original image remains the source of truth,
and a generated view never overrides it.
