## ARC-59 — A seat's memory is a derivation of what it perceived, compressed with its Event IDs kept

**Date** 2026-10-10 · **Status** accepted; implemented by S10 PR P4 (`mineworld-cognition`, `memory/`
and `compress/`) · **Approved by** the primary session's P4 rulings QP4-1 … QP4-10 and the operator's
QP4-11 (2026-10-10), under QS10-1 (`AC-10` is an MVP-0 gate), QS10-11, QS10-17 and QS10-19 ·
**Relates to** `ARC-56`, `ARC-57`, `ARC-58`, `ARC-43` (the `perceived` stream), `INV-4`, `INV-13`,
`AC-10`, `DEP-37` · **Design** `.structured-coding/plans/mvp0/step-17-cognition.md` §3.8, §3.9;
`.structured-coding/plans/mvp0/pr-s10-p4-memory.md` D-P4-2 … D-P4-10, AP4-10 · **Placeholder**
`ARC-S10-c`

**Problem.** A language-model controller must remember a life of 100 simulated days and more, yet a
model's context is bounded, and `AC-10` requires that "character history does not require feeding all
historical events to a model; biography compression stays bounded while original event provenance is
retained". Memory must also respect `INV-13` (a controller never learns world truth it did not
perceive) and `INV-4` (the save is the world's truth, and only the world's).

**Options considered** (step-17 §4.3, §4.4; P4 §4):

```text
(a) an agent-memory framework (Letta, mem0, LangGraph memory,   none keeps Event-ID provenance by
    Graphiti, generative-agents reflection)                      construction; most put a model or an
                                                                 embedding service on the ingest path
(b) model-written summaries as the index                         provenance and coverage would depend on
                                                                 a model; AC-10 would need one
(c) a deterministic derivation of the perceived facts, with      chosen
    structural compression; a model may only rewrite text
(d) memory kept inside the world's save (save/cognition_cache/)   couples a controller's state to the
                                                                 world's truth (INV-4)
```

**Choice: (c).**

- **Input** is the seat's `perceived` stream only (`mineworld perceived` offline, the same audience
  function live), never `observation.events`, a save, the fact log or another seat's memory.
- **Levels.** L0: one record per perceived fact, rendered by a per-event-type renderer, with a generic
  renderer for every type no plug-in knows, so a new System Pack needs no cognition code to be
  remembered. L1: episodes, maximal runs of records at one place, with gaps of at most 1 800 s of world
  time, within one day (`at // 86 400`). L2: chapters, the closed episodes of one simulated week. L3:
  stable facts per counterpart. Every level carries the Event IDs it stands for, as ranges over the
  seat's perceived sequence; coverage is exact (every perceived id is in the open tail or in exactly one
  closed episode).
- **Bounds by construction.** The memory section a decision's context holds is at most 6 000 bytes,
  with per-section line and byte budgets and a 160-byte line, whatever the store holds. The store grows;
  the context does not. No forgetting in MVP-0 (QP4-11).
- **A model never writes provenance.** `StructuralSummarizer` writes every summary with no model, and is
  the only summarizer ingestion calls. An optional, off-by-default `embellish` pass may replace a
  summary's text with prose through the `summarize` tier of the gateway (`ARC-57`, `ARC-58`); it never
  writes, adds or drops a citation.
- **The store is the operator's, outside the save.** One SQLite file per seat at a path the caller
  gives, keyed by world instance, seat and self entity, refused by name on a mismatch.
- **Determinism is defined on a canonical dump** (every table in key order, JSON Lines, no float, no
  wall-clock value), not on SQLite file bytes, which differ across SQLite versions and platforms.

**Consequences.** `AC-10` holds with no model reachable. Deleting a store loses nothing: re-ingesting
the perceived history rebuilds it byte for byte (as a dump). Retrieval is lexical and structural only;
embeddings, importance scores and beliefs come later behind the retrieval seam (QS10-10).

**Revisit** if a world's perceived history makes the 6 000-byte section too small for a useful decision
context (a retune is a new record), if forgetting is added (MVP-1), or if embedding retrieval is adopted.
