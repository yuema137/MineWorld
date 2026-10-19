## ARC-94 — The per-character standard: literal gates with sources, a scorecard, and a failed-generation policy

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings of 2026-10-14 · **Relates to** `ARC-9`, `DEP-8`, `DEP-51`, `DEP-52`, `DEP-55`, `ARC-23` · **Design** step-26 §§6, 15; `pr-s26b1-pipeline.md` §§5.6–5.8

**Problem.** Every protagonist-quality character is generated from a third-party mesh, and its acceptance has so far rested on one person's judgement recorded in a long note. A second character would be judged by a different eye against a different standard, and the bounds would drift toward whatever the candidate already is.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Per-character judgement only | Declined: no reproducible bar; the next character is judged against the last one's mood |
| (b) Gates without a scorecard | Declined: a pass on the gates does not show the characters are of the same quality tier |
| (c) Literal gates with a source per bound, a scorecard row per character, quality-tier parity with the protagonist as the control, and a failed-generation policy | **Chosen** |

**Choice.** Option (c). The standard is step-26 §6 in substance: gates G-1…G-14 with each bound a fixed literal from the protagonist's accepted record or a named real-world reference (`ARC-23` rule 2); each gate prints what it looked at before its verdict (`ARC-23` rule 1); each batch has two review pages (the static scorecard and the visual review; the runtime page is parked for the large milestone, Q-S26-6); a failed generation is recorded in `generation_log.yaml` with its reason and is never silently replaced. The gates are code in `tools/character_pipeline/mwcast/gates.py`, with their bounds in `standard.py`.

**Consequences.** A bound changes only by amending this record. A gate that cannot look (for example G-4(b) without a run directory) reports FAIL with the reason, not PASS. Runtime gates G-7…G-13 are built in S26-b2; until then they are recorded "by parity" against the accepted asset, never as PASS.

**Revisit** when the first batch of new characters is judged, or if a bound must move for a reason outside the protagonist's record.
