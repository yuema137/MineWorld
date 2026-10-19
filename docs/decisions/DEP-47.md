## DEP-47 — The utility core is our own; big-brain, bonsai-bt and dogoap are declined

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-b (no dependency added) · **Approved by** the operator's step-24 rulings of 2026-10-12 (the design's §6.2 and §7 recommendation, accepted under the standing instruction) · **Relates to** `REUSE_POLICY.md` §12 (a record when a candidate is rejected), `CLAUDE.md` §4 rule 16, `ARC-25`, `ARC-23`, `DEP-46` · **Design** step-24 §§6.2, 6.3, 7

**Problem.** The `motivated` decision model (`MODULE_SPEC.md` §5.1) needs a utility core: options scored by multiplying response curves over observed inputs, with a seeded weighted draw among the best options. Mature Rust crates exist for utility AI, behaviour trees and GOAP. `REUSE_POLICY.md` requires a record both when MineWorld adopts a substantial dependency and when it rejects one.

**Options considered.**

| Candidate | Verdict and reason |
| --- | --- |
| `big-brain` (utility AI) | **Declined.** It is built on Bevy's ECS, so adopting it forces Bevy's execution model on world state, which `CLAUDE.md` §4 rule 16 forbids; it has had 28 breaking releases |
| `bonsai-bt` (behaviour trees) | **Declined.** Standalone, but last active in early 2024, and it solves a behaviour-tree case MineWorld has not chosen: a tree encodes today's bands with more structure, not motivation |
| `dogoap` (GOAP) | **Declined.** Its core is standalone GOAP, which is not chosen for now (no System publishes effect declarations); revisit with DL-d |
| **Our own utility core**: integer response curves as data, following Dave Mark's published method, over our own types | **Chosen** |

**Choice.** Build the core ourselves. It is about 300 lines of integer arithmetic over our types. Scores are integers in 0 … 1 000, with a product of curves rescaled by compensation; there is no float anywhere. Its state is none: each consult is a pure function of `(seed, parameters, observation)`. Adoption would cost more than it saves, because the framework's execution model and its determinism would be someone else's.

**Consequences.** The core lives in `cognition/motivated`. A later GOAP model (DL-d) reopens this record only if it is needed; the candidates above are not re-evaluated from memory.

**Revisit** when Systems publish effect declarations for multi-step goals (DL-d), or when a candidate crate's core is shown to be independent of its engine.

**Note, 2026-10-14 (DL-b, the core as built; `.structured-coding/plans/mvp0/pr-dl-b-motivated.md` §§5, 15).**
- **Where it lives.** `cognition/motivated` (`mineworld-motivated`) holds the core and adds no external
  dependency.
- **Scoring.** Every option is offered only from what the observation discloses. Its score is its base weight
  multiplied, in the listed order, by its considerations. Each consideration is a piecewise-linear integer
  curve (≤ 24 points) over one input, adjusted by Mark's compensation factor `1 000 − 1 000 / k`. All
  arithmetic is floor division in `u64`, and the crate denies `clippy::float_arithmetic`.
- **Choice.** A seeded draw, weighted by score, chooses among the options within `top_band` ‰ of the best,
  using `Draw::salted` with the model's own salt. A door is re-drawn per commitment window.
- **Shared code.** The observation readers and request builders are shared with `rule` through
  `mineworld_rule_controller::{read, act}` (SD-DLb-1).
- **Measured result (social-cafe, 30 days, seed 7, every seat `motivated`).**
  - AC-DLb-3: night speech 0 ‰ (PASS).
  - AC-DLb-4: TV against ATUS 513 ‰ against the limit of ≤ 250, and `rule`'s TV is 382 (**FAIL**).
  - AC-DLb-5: night at home 972 ‰ (PASS).
  - AC-DLb-6: agenda kept 930 ‰ against `rule`'s 937 (PASS).
- **Why AC-DLb-4 fails.** The diagnosis (F-DLb-5) is a limitation of the frozen defaults, not a defect. Away
  from home `stay`'s base equals `greet`'s, considerations only lower a score, and the top band is 10 %. So a
  greeting is drawable only at 16:00–20:59 or at home in the evening, and the run speaks 97 lines in 30 days,
  against `rule`'s ≈ 10 159.
- **Status.** `motivated` therefore ships as an optional, non-default model. `rule` stays the default
  (QS24-3), and the revision of model or criterion is the operator's (R-DLb-6).
