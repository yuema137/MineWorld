## DEP-46 — Prior-art audit, October 2026: no shared-world or AI-NPC framework is adopted

**Date** 2026-10-11 · **Status** adopted (a decision not to depend; no dependency added) · **Approved by** the
primary session on the operator's instruction of 2026-10-11 ("learn from existing frameworks; do not add
dependencies on immature tools") · **Relates to** `REUSE_POLICY.md` §§8, 12, 17, 19; `ARC-25`, `ARC-58`, INV-9 ·
**Evidence** `docs/references/PRIOR_ART_2026-10.md` and the four detailed audits it links

**Problem.** A persistent shared world with composable behaviour, multiplayer and AI characters has been
built in part by several projects. `REUSE_POLICY.md` §12 requires a record when MineWorld keeps its own
implementation instead of adopting one.

**Options considered** (facts as read 2026-10-11; details and sources in the audits):

| Candidate | Licence | Verdict and the MineWorld rule it would break |
| --- | --- | --- |
| Croquet Microverse | Apache-2.0 | Reference only: authority runs on every client (INV-9); beta, no release |
| Decentraland SDK7 | Apache-2.0 | Reference only: no single-writer rule, per-client sync, publishing tied to its platform |
| Vircadia (native core / vircadia-world) | Apache-2.0 | Reject / reference only: deprecated lineage; pre-1.0 successor with state in Postgres rows and no event log (`ARC-25`) |
| Concordia | Apache-2.0 | Reference only now: a model referee resolves actions (VISION: models removable); no replay (`ARC-58`). Candidate for an optional controller adapter after Milestone D, under its own design |
| AI Town, Stanford Generative Agents | MIT; Apache-2.0 (unverified) | Reference only: the framework owns state; OpenAI-only or unseeded |
| Inworld, Convai | proprietary, hosted | Reject as controllers: hosted, not offline, no replay |

**Choice.** MineWorld keeps its own semantic layer (contracts, kernel, Systems, packs, persistence,
server, perception, seats, cognition contracts) and continues to reuse commodity infrastructure through
its existing `DEP-` records. Design ideas PA-1 … PA-11 are proposals, each entering through its own PR
design.

**Revisit** when any audited project reaches a stable release with a server-authoritative mode and an
event log, or when an optional controller adapter (Concordia) is designed after Milestone D.
