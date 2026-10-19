## ARC-12 — Intent compliance is testable here, and the name is a launch question

**Date** 2026-09-27 · **Source** a reference audit of Microsoft's MineWorld (arXiv 2025), an
unrelated project that shares our name

Microsoft's MineWorld is a **video-generative world model**: given Minecraft frames and an action,
it generates the next frames — `p(x_{i+1} | x_{<i}, a_i)`. Ours is a world *runtime*. The two use
"world" to mean different things: theirs is visual state latent in model parameters, ours is
explicit structured fact. Their action vocabulary is Minecraft's and frozen; ours is composed at
run time by whichever System Packs are installed, so `Talk` exists because `ConversationSystem`
does and `Attack` may not exist at all. Nothing about their architecture transfers.

Three ideas do.

**1. An action is a first-class structured object, not prose.** They discretize Minecraft's input —
exclusive key groups, camera movement quantized into bins — into 11 tokens interleaved with visual
tokens. Utterly different mechanism, same conviction, and it is the one we already froze in
`ActionIntent`. Worth recording as independent corroboration rather than as something to copy.

**2. Controllability must be evaluated separately from plausibility.** Their sharpest idea: a
generated frame looking right does not mean it followed the action, so they run an inverse dynamics
model over the output to recover which action *actually* happened and compare it to the one
requested. They are explicit that visual quality alone is insufficient.

**This is the part MineWorld should adopt, and we can do it far more strictly than they can**,
because we hold structured state and need no second model to guess what happened:

```text
ActionIntent  GiveItem(Alice, Bob, Coffee)
expect        an ItemTransferred event, and owner(Coffee) == Bob
```

So `intent compliance`, `action validity`, `state transition correctness` and
`renderer consistency` become a class of integration evaluation we can assert directly. Recorded
here as a testing principle for `AC-13` and for every System Pack: **a system that accepts an
action must be shown to have caused the consequence it claims**, not merely to have returned
`Accepted`.

**3. Real-time is a first-class requirement, and that argues for our shape.** They treat latency as
a headline metric and optimize hard — 2 FPS to 5.91 FPS through diagonal decoding, on 32×A100 for
200k steps. The lesson is not the algorithm, which we will never need. It is that generating every
frame is ruinously expensive, and therefore:

> **LM-native does not mean an LM produces every frame.**

MineWorld's layering is the cheap one and should stay: language models make low-frequency semantic
decisions, the runtime holds authoritative state, and Godot renders at 60 FPS. `VISION.md` §1.1
already says authoring is LM-native while the runtime is not LM-dependent; this is the performance
argument for the same split.

**A generative renderer is a plausible future Presentation Pack, and is neither default nor MVP.**
Someone may one day want a video model as the presentation layer instead of a rasterizer. Our
renderer-independent contracts should carry that without change, which is a nice confirmation that
`INV-5` and `INV-14` are worth what they cost. Their project is imaginable as an exotic
*presentation backend*, never as our kernel.

**The name.** Microsoft's MineWorld has a 2025 paper, a GitHub repository and a Hugging Face
presence. That is a discoverability and package-naming collision, not a technical one, and it
blocks nothing today. **Re-evaluate the name before any public launch** — recorded so the decision
is deliberate rather than discovered in a search result.
