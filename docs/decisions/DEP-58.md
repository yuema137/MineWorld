## DEP-58 — Reuse verdicts for shared memory: build on SQLite and P4/FG-b, adopt the methods of content addressing and tiered memory, decline Letta, Zep, Graphiti and mem0 as runtime dependencies

**Date** 2026-10-14 · **Status** accepted (considered 2026-10-14, step-28 §16); no new dependency is added by this record. Each licence marked **verify** in step-28 §16 is confirmed by the PR that first relies on it · **Approved by** the step-28 frozen design · **Relates to** `DEP-37`, `ARC-98`, `ARC-100`, `ARC-101`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12, 15 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §16

**Problem.** Shared memory needs storage, content addressing, tiered loading and pluggable scopes. `CLAUDE.md` §4 (rule 16) requires each candidate to be checked in both directions: whether a mature wheel is being reinvented, and whether an existing wheel is being forced where it does not fit.

**Options considered** (verdicts from step-28 §16).

| Candidate | Verdict |
| --- | --- |
| SQLite (`sqlite3`, `DEP-37`) | **Keep.** One file per home, FTS5, incremental vacuum |
| Content-addressed stores (Git objects, IPFS / `multiformats`, Perkeep, restic, borg) | **Adopt the method, decline the code.** Objects already have a stable world id (`EventId`); a hash key would add a second identity |
| Ref-counted blob stores (`diskcache`, LMDB-backed stores) | **Decline.** A stored refcount can drift from its references; the anti-join over access rows is exact |
| Generative Agents memory stream (Park et al., 2023) | **Reference only**; each agent's stream is its own |
| MemGPT / Letta tiered memory | **Adopt the method, decline the dependency.** Hot and cold tiers and load-when-needed are the method. Shared blocks are agent-writable text, a knowledge channel outside the world, and model-managed paging is not deterministic |
| Zep / Graphiti | **Decline.** Model-extracted triples are not a deterministic derivation, it needs a graph server, and its sharing is by tenant, not by perception |
| mem0 | **Decline.** Its scopes are application tenancy and its writes are model-decided. Scopes as first-class configuration are adopted (`ARC-100`) |
| Forcing an existing wheel | Not done. No framework's execution model owns our transactions, access rule or determinism |

**Choice.** Build on SQLite and P4/FG-b; adopt the methods of content addressing and tiered memory; decline Letta, Zep, Graphiti and mem0 as runtime dependencies.

**Why not them.** Each declined candidate either makes model output the source of stored knowledge, or gives access by tenant rather than by perception. Both conflict with INV-13 and `ARC-59`.

**Revisit** when the MVP-1 cold tier (S28-h) or an embedding-based scope is proposed (step-28 §18). Either needs a `DEP` record of its own.
