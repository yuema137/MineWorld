## ARC-23 — Locate before counting: an instrument must be shown to see what it measures

**Date** 2026-09-27 · **Relates to** [`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) §8,
[`ACCEPTANCE.md`](ACCEPTANCE.md), `ARC-17` · **Applies to** every verification in this
repository, not only visual ones

This project has produced the same defect seven times, in code, in tests and in review. A number
was trusted because it was plausible, and it turned out to be measuring something adjacent to the
thing it was named after.

| | The instrument | What it actually measured |
| --- | --- | --- |
| 1 | A face-morph sweep taking `max abs(x)` across the head at a height | the neck under the chin, the ears beside the cheeks — **0.00% for every chin and cheek morph**, indistinguishable from the dead end that would have justified replacing the head |
| 2 | An inverted-face count, to explain holes in a garment | the armpit at `z 1.31–1.38`, a legitimate fold; the visible holes are the outer shoulder at `z 1.42–1.47`. Four optimisation sweeps ran against it |
| 3 | The same count, compared against the *previous* step | zero for a surface already inside out — the reassuring number was an artefact of the comparison |
| 4 | A review contact sheet at 620 px per tile | read as "nothing changed" when both the footwear and the roofline had changed |
| 5 | A third-person frame with a 60 px head | a face, judged at a scale where a face cannot be judged |
| 6 | A clamp bound computed as `PLAYER_SPEED * MAX_SIM_DT` | itself — raising the constant under test raised the bound, and the assertion passed while the player crossed 2.4 m in one frame |
| 7 | A per-frame step check as a multiple of the median | unusualness, not displacement — it failed on a *correctly* clamped hitch |

**The decisive observation is (1): a measurement that cannot see the thing it measures reports no
effect**, and "no effect" is exactly what a genuine dead end looks like. A broken instrument does
not announce itself; it returns a clean, confident, wrong number.

**Decided.**

1. **Locate before counting.** Before an aggregate is trusted, show it is reading the right
   region: print where the extrema are, which elements contributed, the bounding box of what was
   selected. A count with no location behind it is not evidence.
2. **A bound is never derived from the quantity under test.** Fixed literals, or a bound derived
   from the requirement rather than the implementation. Otherwise the test moves with the defect.
3. **Measure the property the claim names**, not a proxy that correlates with it. "No frame
   displaces the player more than this" is not "no frame is unusual".
4. **Evidence at the resolution of the claim** — `VISUAL_FIDELITY.md` §8, which is this rule for
   images specifically.
5. **A sub-item `PASS` is a claim and needs its own evidence.** An honest overall verdict does not
   license unverified per-item claims beneath it, and two were asserted here against frames that
   contradicted them.

**The tell worth teaching.** In (1) and (2) the defect surfaced only because the numbers refused
to behave — a sweep reporting exactly 0.00% everywhere, an optimisation bouncing 82–127 with no
trend. The agent that found both named the lesson better than the rule does: *"That is luck, not
method."* **Non-convergence and implausible uniformity are evidence about the instrument, not
noise to tune through.** When a number will not settle, stop optimising against it and go and look
at what it is reading.

**Why this is a decision and not a style note.** Six of the seven were caught, but every one was
caught late and two came within a step of a wrong architectural conclusion — replacing a character
head that did not need replacing, and shipping a movement clamp that did not clamp. The cost of a
broken instrument is not a wasted hour; it is a confident decision made on a clean number.
