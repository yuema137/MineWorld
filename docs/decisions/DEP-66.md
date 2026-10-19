## DEP-66 — The MatrAIx persona schema: adopted as an optional backend pack (licence check, pinned commit and digest, update procedure, attribution, isolation); earlier texts kept

**Date** 2026-10-15 · **Status** accepted (Re-ruled 2026-10-15, operator, final: RR-S30-5); implemented by S30-h; no dependency is added before S30-h, and none outside its pack ever · **Approved by** the operator, directly in the primary session, 2026-10-15 (RR-S30-1 … RR-S30-5, the last final; Q-S30-8 for the overlay's privacy; Q-S30-9 for `log-linear`) · **Relates to** `ARC-122`, `ARC-123`, `ARC-126`, `DEP-67`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12, [`MODULE_SPEC.md`](MODULE_SPEC.md) §2 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§3.2, 6.6–6.7, 16.2

**Problem.** The operator pointed to MatrAIx (Li, Hao et al., *MatrAIx: Simulating the World with 8.3 Billion Persona Agents*, arXiv 2608.04205v1, 2026) as the model for very many person attributes. Its schema has 1,290 categorical dimensions in five groups, natural-language phrases and six locale label packs, but no body or appearance dimension and no numeric kind, and some dimensions and its coreset carry licences narrower than the repository's.

**Options considered — the rulings in order, every earlier text kept** (step-30 §16.2).

| Ruling | Text | Status |
| --- | --- | --- |
| Step approval (Q-S30-7) | "taxonomy adapted, optional import in MVP-1"; about 200 built-ins | superseded by RR-S30-3 (≤ 100) and RR-S30-2 (everything before the release) |
| RR-S30-1 | "我们不要对matrixl有依赖，我只是让你学习它的方式" — inspiration only, no dependency | superseded by RR-S30-4 and RR-S30-5; its "no hard dependency" survives in RR-S30-5 |
| RR-S30-3 | ≤ 100 attributes, only what affects play | in force for the built-in catalogue and as the default mask's bound |
| RR-S30-4 | import as-is, behind a modular mask of 100, `all` available | in force as refined by RR-S30-5 |
| **RR-S30-5 (final)** | an optional backend with a demo, never a hard dependency | **in force** |

**Choice.** An optional Entity Pack, `entities/attributes-matraix`, not installed by default.

1. **Licence check.** The repository `MatrAIx-ai/MatrAIx-Persona-8B` is MIT. Excluded **on licence grounds only**: the 50 dimensions whose `source_origin.license` is `citation-only` (28) or `research-use-reference` (22), listed in step-30 §6.6, until their owners' terms are confirmed (re-admission is a reviewed pack change citing the confirmation); and the Persona-1M coreset (`matraix-research-only`, derived from real people's records), never imported. 1,240 dimensions remain. Redundancy is the mask's job, never a reason to drop one.
2. **Pinned.** `persona/schema/dimensions.json` (schema 1.0) at upstream commit `3633d8dab149a9482a71b024418a49ae828cc941`, SHA-256 `cbf4db9d887dec057431aff409e1d28d83f345e7a31e29a4275a607212ecfca5`, imported verbatim: ids, categories as groups, value lists, phrases and the six label packs, nothing renamed.
3. **Overlay** inside the pack, keyed by their ids and never editing their file: visibility (default `learnable`; `private` for ethnicity, religion, sexual orientation, disability, citizenship, veteran status, income and every `health_*` dimension, Q-S30-8), mutability (`derived` `age_bracket` from `core.birth`), owner (`economy` for `socioeconomic_band`), facets, and level order.
4. **Masks** `matraix-core-100` (default; ≤ 100 chosen by gameplay, each with its `why`) and `all`; **mapping** `mappings/core.yaml` to the built-ins readers declare (for example `core.chronotype ← lstyle_sleep_schedule`, `economy.wealth-band ← socioeconomic_band`, S30-a F-S30a-2), every target value existing and every source value covered.
5. **Update procedure.** The pack's offline import tool (Python standard library) verifies the digest, re-applies the exclusions and regenerates the pack; CI re-runs it over the pinned inputs and requires byte equality; an upgrade is a reviewed PR with a diff report; a removed dimension that a mask, mapping or profile names fails by name; a removal is a major version.
6. **Attribution.** `pack.yaml` (`license: MIT`, `attribution:`), `LICENSE` (upstream text), `NOTICE` (commit, digest, paper), and the line *"Optional person-generation backend built on the MatrAIx persona schema (Li, Hao et al., 2026, arXiv 2608.04205; MIT), imported unchanged."* in the README's backends row, the demo's guide chapter, and a `CITATION` entry.
7. **Isolation.** Nothing outside the pack names it, except `worlds/matraix-demo/`, its guide page, the README's attribution line, the plans and this file: a CI grep refuses any other mention (X-S30-18), and a CI job builds and tests the workspace with the pack removed (X-S30-19). Their sampler code is not used (`DEP-67`).

**Why not ourselves.** A broad, documented, MIT persona schema with phrases and translations exists; re-authoring 1,240 dimensions would reinvent it. **Why not as the core.** It has no appearance or numeric kind, it is oriented to software-user personas, and a hard dependency would make every world depend on an external release (RR-S30-5).

**Consequences.** The workspace, every reference world and every test run without the pack; a world uses it only by requiring it and naming `backend:` for it. `worlds/matraix-demo` (S30-h) shows its attributes in casting briefs, "who I am" and "who I see", the agent description and both inspect panels, including a zh-Hans panel from its label pack, and a CI-executed guide section walks through switching backend and mask and back. Under it, the appearance group and quantities still come from `core` (`ARC-125`, `ARC-126`).

**Revisit** when a pinned upstream release changes the schema, when an excluded dimension's licence is confirmed, or when the authors' terms for joint promotion are agreed (the operator's).
