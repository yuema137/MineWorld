## DEP-67 — The population sampler is our own; pgmpy, pomegranate, PopulationSim and MatrAIx's sampler code declined; MatrAIx's DAG as an optional `log-linear` mode

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15; Q-S30-9 ruled 2026-10-15); implemented by S30-d, the `log-linear` mode by S30-h · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-9) · **Relates to** `ARC-123`, `ARC-126`, `DEP-66`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§8.6, 16.3

**Problem.** Genesis needs a dependency-aware, stratified sampler that is integer, deterministic on Linux, macOS and Windows, shared by the world's genesis and the casting CLI, and that never re-draws authored or ruled values. `CLAUDE.md` §4 rule 16 asks both halves of the reuse question.

**Options considered** (step-30 §8.6).

| Candidate | Verdict |
| --- | --- |
| `pgmpy` (MIT, Python) | Declined: Python while genesis is Rust; float sampling; a dependency larger than the problem (≈ 400 lines of sampler) |
| `pomegranate` (MIT, Python) | Declined, for the same reasons |
| PopulationSim (ActivitySim, BSD-3) | Declined for MVP-0: it solves household synthesis against zone control totals (IPF), in Python with pandas; recorded as the reference if MVP-1's region worlds need joint fitting |
| MatrAIx's sampler code | Not used: float Python. Its formula — prior × likelihood ratios × hard masks — is the one `ARC-123` uses |
| **Our own, in `mineworld-attributes-model`** | **Chosen:** integer, deterministic, one implementation with two entry points |

**Choice.** The sampler is MineWorld's, in the model crate (S30-d). **MatrAIx's `full_dag.json`** (1,308 nodes, 6,999 directed proposal edges with a valid topological order, 524 masks, 54 full CPTs) **is offered as an optional `log-linear` mode of its backend only** (Q-S30-9, ruled 2026-10-15): the import tool converts each term to fixed-point log weights, the engine sums them and exponentiates through a committed integer table, cycles are refused at import, and their energy-scoring repair is not reproduced. Its priors are global, so it is conditioned on adults and is never a place's distribution; it is used by `worlds/matraix-demo`, never by a reference world. S30-h prototypes it first and drops it, reporting to the operator, if determinism, load time (a 24.8 MB file) or the ≤ 2 pp fidelity check against their own sampler on the masked set fails.

**Why not them.** Each library is float and Python, outside genesis's language and determinism; the interim Python casting sampler is removed with no adapter (`CLAUDE.md` §4 rule 12). **Not forced:** no library's execution model owns our genesis.

**Consequences.** No sampling dependency is added. The sampler is about 400 lines of Rust that MineWorld maintains, tested by golden draws on three platforms (AC-S30-7) and by planted mutations (an `f64` draw, a re-drawn ruled value, a read display name; step-30 X-S30-6, X-S30-13, X-S30-14). The interim casting sampler on `mvp0/s26-cast-director` is removed by S30-g.

**Revisit** when MVP-1 region worlds need joint fitting (PopulationSim), or if a deterministic Rust library solving the same problem appears.
