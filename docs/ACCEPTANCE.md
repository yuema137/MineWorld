# Acceptance standard

**Status:** how agent autonomy and operator judgement divide
**Audience:** coding agents, and whoever reviews their work.

One test settles most cases:

> **Can failure be demonstrated objectively? The agent owns it.**
>
> **Does the answer depend on taste, fidelity, comfort, atmosphere or experience? The operator
> owns it.**

And a third question worth asking every time:

> **Does a pending operator decision prevent unrelated work? Almost never. Continue.**

Human review is a checkpoint on one branch, never a global barrier
([`DECISIONS.md`](DECISIONS.md) `ARC-11`).

---

# 1. What the agent owns

Design, implement, review, test, fix, merge and continue, wherever acceptance rests on evidence:

```text
schemas · API contracts · serialization · persistence · deterministic transitions
networking · identity consistency · event causality · package loading
dependency and licence validation · file formats · renderer bindings
collision correctness · navigation reachability · asset dimensions · scale normalization
skeleton compatibility · animation retargeting · performance · absence of crashes
regression tests · clean-checkout reproducibility · headless execution · export correctness
```

Also, and this list exists because interrupting over these wastes the operator's attention:

```text
PR approval · struct definitions · event schemas · database choice · refactoring
dependency upgrades · test organization · protocol details · component implementation
CI · linting · formatting · adapters · conversion scripts · generation plumbing
licence metadata schemas · WASM interfaces · Godot import mechanics · GLB conversion
animation retargeting mechanics
```

**Vision capability is agent-owned infrastructure.** If the environment cannot inspect image
pixels, that is a defect the agent fixes — not a reason to guess, and not a reason to interrupt.

# 2. What the operator owns

```text
Does this look like the intended default style?
Does the scene feel alive rather than synthetic?
Does the 3D character actually look like the reference person?
Does entering the building feel natural? Does the interior feel believable?
Does the scale feel pleasant to walk through? Does the lighting feel right?
Is there enough visual density? Would I want to spend time here?
Is this fidelity sufficient to ship as MineWorld's default?
```

The agent analyses and recommends. It **never self-approves** these, and before approval such work
is a *default-style candidate*, never the accepted style.

Some of these have measurable proxies and still belong here: walking feels too fast, the camera is
uncomfortable, the room feels cramped, the building feels toy-sized, the character is uncanny, the
street feels lifeless. An agent may *notice* these. It may not *settle* them.

---

# 3. The workflow

```text
inspect the canonical references (actually, in pixels)
        ↓
implement · generate · validate objectively · integrate
        ↓
agent self-review · runtime screenshots · reference-vs-runtime comparison
        ↓
fix every objective defect
        ↓
READY FOR HUMAN VISUAL REVIEW
        ↓
accepted · accepted with minor follow-up · revision required
```

**The operator is not a substitute for automated testing.** They see a candidate only after the
objective defects are already gone. A review that spends its attention on a missing texture has
wasted the thing it exists to collect.

## 3.1 Vision is a hard prerequisite

Before claiming that anything matches a reference, the agent must have inspected that reference's
**pixels**. A filename, a manifest, `ART_DIRECTION.md` or a prior text description does not count.

The proof is describing what only the image shows: the hairstyle, the specific garment, the camera
angle, the material, the vegetation density, the lighting direction, the foreground relationships.
If the environment cannot do this, the agent records the gap, establishes a vision path, continues
unrelated work, and **claims no fidelity in the meantime**.

## 3.2 Response states

| | Meaning |
| --- | --- |
| **Accepted** | the direction is approved and becomes the current default baseline |
| **Accepted with minor follow-up** | the agent completes the named changes without another review, unless they materially alter appearance |
| **Revision required** | the operator names the discrepancy; the agent returns with another *integrated* candidate, not a stream of small ones |

---

# 4. Objective gates before a visual review

These are the agent's, and they are checked before the operator is asked anything.

## 4.1 Any demo

Launches from a **clean checkout** with no hand-preserved engine cache, imports its assets
automatically, has no missing textures or broken references, no fatal errors, and relaunches
reproducibly.

## 4.2 Movement and interiors

The player moves correctly, cannot pass through blocking geometry, reaches the entrance, **enters**,
moves around inside, and leaves. Entry and exit preserve position, spawn no duplicate player, keep
world identity, change semantic location where a `Place` exists, and leave the camera working.

An interior is **not** satisfied by a popup, a static image, a façade, a billboard, or a decorative
room you cannot reach. It needs navigable floor, walls, furniture, working collision, and a legible
layout.

## 4.3 Generated assets

Auto-reject: broken alpha, halos, wrong crop or dimensions, perspective mismatch, generation text
artifacts, severe anatomy errors, wrong scale, corrupt files. For directional characters, front,
back, left and right must be **the same person** — hair, clothing, accessories, proportions,
palette — verified before a batch is generated rather than after.

## 4.4 3D scale

```text
human 1.7–1.8 m · door taller than the human · furniture human-scaled
plausible ceiling height · movement speed consistent with the scale
```

Whether that scale *feels* good remains the operator's.

## 4.5 3D character

GLB imports, mesh intact, materials assigned, textures resolve, skeleton present with the expected
bones, scale normalized, axes correct. Idle and walk work without foot sliding, inverted limbs, an
exploded skeleton or wrong facing. Every asset — mesh, textures, rig, animation, shaders — passes
the redistribution test in `DEP-8`, and that check is entirely the agent's.

---

# 5. Scope: small and complete beats broad and fake

For an evaluation demo, depth wins:

```text
one exterior street or plaza · one social building with a real interior
one more building · several characters · vegetation and street furniture
indoor and outdoor movement that works
```

is worth more than a large map of façades. Cut content to buy a real interior.

---

# 6. The review package

```text
milestone id · what changed · exact launch command · what to inspect
runtime screenshots (exterior, interior, character) · the references used
reference-vs-result comparison where it applies
known limitations · the specific subjective questions being asked
```

Two to three coherent complete candidates where several directions are viable — and never a request
to choose between individual trees, benches or textures. **Do not bury a review request inside an
engineering log.**

---

# 7. The current queue

| | Milestone |
| --- | --- |
| **VIS-2D-1** | a complete playable 2D scene materially matching the references, with at least one real enterable interior |
| **VIS-3D-1** | a rigged humanoid intentionally matching the designated character reference, shown clearly in the runtime |
| **VIS-3D-2** | a small complete 3D environment: that character, outdoor space, enterable building, real interior, lighting, materials, movement, camera modes |

Everything needed to make those runnable and technically correct is handled autonomously **before**
the operator is asked.

---

# 8. Approving a default does not constrain the framework

An accepted default presentation is what MineWorld *ships and demonstrates*. It is never a
restriction on what a MineWorld world may look like. The architecture keeps

```text
semantic world → presentation interface → any Presentation, Style or Asset Pack
```

so that pixel, anime, photorealistic, voxel, low-poly, hand-painted and minimal all remain
possible. Infrastructure that assumed the default style would fail the project's own premise
(`ARC-11`).
