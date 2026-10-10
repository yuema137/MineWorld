# PR S10-P4 — Subjective memory, compression L0–L3 with retained Event IDs, retrieval, `AC-10`

## DESIGN FROZEN 2026-10-10 (primary session)

```text
Design revision:        revision 2 (2026-10-10): revision 1 (PR #138, first commit) with §13.1's
                        rulings filled in, as committed on docs/s10-p4-design with this header
Approved by / evidence: the primary session's rulings on QP4-1 … QP4-10 of 2026-10-10, and the
                        operator's ruling on QP4-11 of the same day, relayed by the coordinator to
                        the S10 planning session (§13.1)
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §12 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§2.1), invariants (§2.3), decisions D-P4-1 … D-P4-13, the acceptance and adversarial criteria
AP4-1 … AP4-13 and the commit plan are frozen. Progress, evidence, findings and bounded corrections stay
writable (§14). No P4 question remains open. Implementation starts in a fresh session (§12); this
planning session does not implement.

*Superseded header:* `DESIGN DRAFT — awaiting the primary session's review (not frozen)`, revision 1.

**Effort:** `mvp0` · **Step:** S10, [`step-17-cognition.md`](step-17-cognition.md) (§3.8 subjective
memory, §3.9 compression, §3.16.3 seams, §4.3, §4.4, §4.7, §5, §6 A-5, §9 P4, §10 IC-4, §15 audit and
rulings, §15.8 platforms) · **Parent:** [`overall.md`](overall.md) §3 (S10), §4 (`AC-10` → S10), the
decision-number table ("Decision numbers assigned since the parallel build-out table").
**Predecessors (merged):**

- P3, the Python SDK, #98: [`pr-s10-p3-python-sdk.md`](pr-s10-p3-python-sdk.md);
- P5a, backends, recorder, budgets, #120: [`pr-s10-p5-backends.md`](pr-s10-p5-backends.md);
- P5b, the Anthropic adapter and the subscription route, #126:
  [`pr-s10-p5b-hosted-subscriptions.md`](pr-s10-p5b-hosted-subscriptions.md);
- S11-C, perception, the `perceived` stream and `mineworld perceived`, #95: [`step-12-server.md`](step-12-server.md)
  §17 (`ARC-43`).

**Working name:** P4. The PR number is assigned when the PR opens.
**Planning base:** `origin/main @ 5ccc402` (#135). Planning worktree
`/Users/yuema137/mineworld-worktrees/plan-s10-p4`, branch `docs/s10-p4-design`.
**Prerequisite now met:** step-17 §15.3 gated P4 on S11-C's offline export. `mineworld perceived` is
on `main` (`tools/cli/src/perceived.rs`, S11-C C-C3b), so P4 can be detailed and, once frozen, built.

**Decision numbers (QP4-1, ruled 2026-10-10 by the primary session).** **`ARC-59`** is P4's
architecture record (memory, compression with retained ids, the store outside the save). **`DEP-37`**
is the store and retrieval record (replacing step-17's placeholders `DEP-S10-d` and the memory half of
`DEP-S10-f`). The primary session records both in `overall.md`'s decision-number table. `ARC-72 …
ARC-74` stay with P6. An implementation session never picks a number.

### Binding rulings this design is written under

| Source | Ruling | Where this design meets it |
| --- | --- | --- |
| `AC-10` (`MVP.md` §9), verbatim | "After 100 simulated days, character history does not require feeding all historical events to a model; biography compression stays bounded while original event provenance is retained." | §5.5, §5.6, AP4-10 (IC-4) |
| QS10-1 (a) (operator, 2026-10-08) | `AC-4`, `AC-10` and Milestone D are MVP-0 gates. P4 is brought forward. | the whole PR |
| QS10-11 (step-17) | Context ceilings as stated; P4 may tune them on the 100-day export **before** IC-4 is run, then freeze them, never after measuring. | §6 (planning measurement), D-P4-9, QP4-7 |
| QS10-17 (step-17) | The cognition store lives in the operator's cognition directory, keyed by world instance and seat, never inside the world's save. | D-P4-3, the `ARCHITECTURE.md` §8 edit (G-6) |
| QS10-10 (step-17) | Embedding retrieval and beliefs later, after Milestone D, behind `Retriever`. | §4.2, D-P4-11 |
| QS10-19 (operator, 2026-10-08) | Local models only for any gate. Agents never use a key, never call a hosted API, never run, pull or download a model. | §7 (no Gate 1), §12 NEVER |
| QS10-18 (primary, 2026-10-08) | Cost ceilings on wall time (P5a's budgets); the **context bound** on simulated time. | D-P4-9: the bound is a byte ceiling, independent of the world's age |
| step-17 §15.3 | Memory ingests only the reliable `perceived` stream, never the lossy `observation.events`. | D-P4-5 |
| Platforms (operator, 2026-10-08), verbatim: "我们要保证支持全平台，mac linux windows都可以" | macOS, Linux and Windows. FTS5 must exist in all three CPython builds, verified in P4's first commit; otherwise lexical retrieval without FTS5 behind `Retriever` (step-17 §15.8). | D-P4-12, AP4-12 |
| `CLAUDE.md` §4 rules 3, 7, 10, 11, 16 | No provider concept in cognition contracts; typed boundaries; LM-independent deterministic tests; no premature abstraction; reuse before reinvention, recorded both ways. | §4, D-P4-2, D-P4-4, AP4-11 |
| P5a R-P4-1 | P4 starts from P5's `cognition/lm-controller` tree and adds `memory/` and `compress/`. It does not recreate the package, the workspace member, the guard or the CI commands. | D-P4-1, §5.1 |

---

## 1. Goal, in one paragraph

A seat's memory is a deterministic derivation of what that Person perceived, and of nothing else. P4
adds to `mineworld-cognition` a per-seat **memory store** (one SQLite file, standard-library `sqlite3`),
**ingestion** of perceived facts into L0 records rendered by small per-event-type renderers (a generic
renderer covers every event type no plug-in knows), deterministic **compression** into L1 episodes, L2
chapters and L3 stable facts, every one of which carries the Event IDs it stands for, and a bounded,
deterministic **retrieval** that assembles the memory section a decision's context will hold. A
`StructuralSummarizer` writes every summary's text with no model. An optional `ModelSummarizer` may
rewrite that text into prose through P5a's gateway (tier `summarize`), never touching the citations. P4
then proves `AC-10` on a real 100-day `mineworld run` save, exported with `mineworld perceived`: the
assembled memory section stays under a fixed byte ceiling that does not grow with the world's age, every
citation resolves to a fact the Person perceived, every perceived fact is covered exactly once, nothing
the Person did not perceive appears anywhere, and the store is byte-reproducible from the facts. No
model is reachable in any test. P4 joins no seat, builds no decision context, and calls no model in its
default path: those are P6.

## 2. Scope

### 2.1 In scope

- `cognition/lm-controller/src/mineworld_cognition/memory/`: the record types, the store, the name book,
  the renderers and their registry, ingestion, retrieval and the memory section.
- `cognition/lm-controller/src/mineworld_cognition/compress/`: L1 episodes, L2 chapters, L3 stable
  facts, the `Summarizer` protocol, `StructuralSummarizer`, `ModelSummarizer` and the `embellish` pass.
- The `AC-10` scenario test (IC-4) over the real `mineworld` binary, plus its small harness.
- `docs/DECISIONS.md`: `ARC-59` and `DEP-37` (QP4-1).
- `docs/ARCHITECTURE.md` §8 (G-6) and `docs/CORE_CONCEPTS.md` §5.4 (G-2), the two edits step-17 §12
  assigns to P4.
- `cognition/lm-controller/pyproject.toml`: one pytest marker, `real_binary`; `scripts/ci_layer.py`:
  `python-smoke` deselects it by name (§5.10).
- `cognition/lm-controller/README.md`: a few lines on memory (it stays under its human-README length).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Joining a seat, the `perceived` stream in the SDK (cursor, resume, `lagged`, `cursor_unavailable`) | P3b |
| The decision context, `SocialChoice`, `recalls` validation, triggers, routine policy, the decision log | P6 |
| `CognitionConfig.store` and the `python -m mineworld_cognition` entry point | P6 (P5a R-P6-1) |
| Names learned from observations (`display-name` components, place labels) | P6 calls `NameBook.learn` (§5.4); P4 provides the call |
| Impressions from observed relationship values (step-17 §3.8.3) | P6, from observations; P4 keeps the L3 level that perceived facts state |
| Embedding retrieval, importance scores, beliefs and contradictions | later, behind `Retriever` (QS10-10) |
| Forgetting: pruning old L0 rows from the store | MVP-1 (QP4-11) |
| CJK-aware lexical matching | follow-up behind `Retriever` (QP4-8) |
| The `persona` pack | P6 |
| Any Rust, server, protocol, SDK or World Pack change | none: a material stop (§12) |

### 2.3 Invariants this PR must hold

Numbered as step-17 §5 where they come from it; `P4-n` are this PR's own.

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-1 | No change to `kernel/` or `contracts/`. | AP4-13 (diff gate) |
| I-2 | No Rust crate and nothing under `sdk/python` depends on `mineworld_cognition`. P4 adds no import of `mineworld_cognition` from the SDK. | AP4-13; the existing `test_structural_isolation.py` |
| I-3 | Memory reads only what it is given: perceived facts, names it is taught, and its own store. No module of `memory/` or `compress/` opens a save, runs a subprocess, reads the fact log or another seat's store. | AP4-11 (source scan); AP4-10 (d) |
| I-5 | Memory is a derivation: the canonical dump of a store rebuilt from the same perceived facts is byte-identical, whatever the batch boundaries. | AP4-10 (e), (g) |
| I-9 | A new System Pack needs no cognition code to be remembered: an unknown event type is rendered, covered and citable. | AP4-4 (c) |
| I-10 | No provider concept in the memory store, its schema, its dump, or any module of `memory/` or `compress/`. | AP4-11; P5a's scan extended to the store dump |
| I-11 | Every test runs with no model reachable and outbound network blocked except loopback. | the member's existing `addopts` guard |
| I-12 | Every summary carries Event IDs, and every one resolves to a fact the seat perceived. | AP4-10 (b) |
| I-13 | The memory section's size is bounded independently of the world's age. | AP4-10 (a) |
| P4-1 | A model never writes, adds or drops a citation. Prose replaces a summary's text only. | AP4-9 (b) |
| P4-2 | The cursor a store reports is never ahead of what it has durably ingested. | AP4-3 |
| P4-3 | Retrieval is deterministic: no wall clock, no randomness, no floating-point ordering. | AP4-7 (e) |
| P4-4 | Player words never become query syntax: a trigger's words are quoted terms, never an FTS5 expression. | AP4-7 (d) |
| P4-5 | No entity id appears in any text a model could be shown; names, or a generic label, only. | AP4-4 (d) |

---

## 3. Audit anchors (`origin/main @ 5ccc402`)

| File / symbol | Finding | Consequence for P4 |
| --- | --- | --- |
| `cognition/lm-controller/` | P5a/P5b's package: `backend/` (model, canonical key, scripted, adapters, registry, presets), `record.py`, `budget.py` (incl. `SqliteLedger` over stdlib `sqlite3`), `gateway.py` (`ModelGateway.complete(seat, request) -> Completed \| Refused \| Failed`; `Router.for_tier`), `config.py`, `secrets.py`, `errors.py`. No `memory/`, no `compress/`. | P4 adds two subpackages beside them (R-P4-1). It reaches a model only through a `ModelGateway` passed in (D-P4-10). |
| `gateway.py` `Tier` | `"summarize"` is already a tier; `Purpose` in `backend/model.py` already has `"summarize"`. | `ModelSummarizer` uses both unchanged; P4 adds no tier and no purpose. |
| `backend/model.py` `CompletionRequest` | Strict, frozen, no floats (`temperature_milli`); `estimate_tokens`. | A summarize request is built from these types only. |
| `backend/scripted.py` `ScriptedBackend` | A function of the request; counts calls; optional delay. | P4's summarizer tests use it behind a real `ModelGateway` (no model). |
| `record.py` `RecordingBackend`, `ReplayBackend`, `CassetteMiss` | Strict ordered replay; a miss is an exception. | A record → replay round trip of an `embellish` pass, written into `tmp_path`; no committed cassette (D-P4-10). |
| `tests/test_provider_scan.py` `TERMS`, `PATTERN`, `ALLOWED` | Scans `src/mineworld_cognition` except the adapter files and `config.py`. | New modules are scanned automatically. P4 adds the store's dump to the scanned artefacts (AP4-11). |
| `tests/support.py` `run` | Runs a coroutine on a selector loop on every platform (pytest-socket guards it; F-P5-4). | P4's async tests use it. |
| `pyproject.toml` (member) `addopts` | `--allow-hosts=127.0.0.1,::1`, `-m "not live_model"`, `-v`, `--strict-markers`. | P4 adds the `real_binary` marker; `--strict-markers` makes it mandatory to declare. |
| `scripts/ci_layer.py` `python`, `python-smoke` | `python` builds `mineworld-cli` (dev profile) and runs the cognition suite whole; `python-smoke` runs it with no binary. `.github/workflows` runs `python` on `ubuntu-24.04`, `windows-2025`, `macos-15`, with `python-version: "3.12"`. | The AC-10 test needs the binary: it carries `real_binary`, which `python-smoke` deselects by name, as it does `real_server` for the SDK (§5.10). |
| `sdk/python/src/mineworld_sdk/wire/contract.py` `PerceivedEvent` | Strict model of `PerceivedEvent<serde_json::Value>`: `id` (decimal `EventId` string), `at` (i64), `event_type`, `subjects`, `participants`, `place: PlaceId \| None`, `caused_by`, `payload: EventRecord` (`payload: JsonValue`), `visibility`, `provenance`. | Ingestion's input type, unchanged. P4 never converts an id to `int` (D-P4-4). |
| `sdk/python/src/mineworld_sdk/wire/ids.py` | `EventId`, `EntityId`, `EntityKey`, `WorldInstanceId` are distinct `NewType`s; ids are decimal strings up to u64. | The store's identity and records use them. |
| `tools/cli/src/perceived.rs` | `mineworld perceived <world> --save DIR --person KEY [--since ID] [--json]`; `--json` prints one `PerceivedEvent` per line in the server's wire form, ascending ids, from `audience::perceived_by`. Reads the save and the pack only. | The AC-10 harness's input, and the same function the live stream uses (step-17 I-4). |
| `tools/cli/src/inspect.rs` l. 69 | `mineworld inspect SAVE` prints `world <pack>, instance <32 hex>`. | The harness reads the world instance from it (the store's identity), never from the save directly. |
| `docs/ARCHITECTURE.md` §8 l. 273 | `save/cognition_cache/   later (S10 / MVP-1)`. | G-6: removed; the store lives in the operator's cognition directory (C1). |
| `docs/CORE_CONCEPTS.md` §5.4 l. 246 | "A controller normally reads the summary." | G-2: "A controller normally reads the compression of its own perceived history; the objective biography's compression is a tool's view" (C1). |
| `systems/*/src` event types (grep of `const EVENT_TYPE`) | `spoke {speaker, listener, utterance}`; `arrived {person, location}`; `person-entered-place {person, from, place}`; `named {person, name}`; `became-acquainted {person, counterpart}`; `relationship-changed {person, counterpart, from, to}`; group-activity, invitations, agenda, and others. | The plug-in renderers P4 ships (D-P4-6), each decoding the payload with its own small strict model; everything else renders generically. |
| `worlds/social-cafe/people/alice.yaml` | `name: Alice Moreau`; places carry tags and no `name`. | Places have no `named` fact: offline, a place is rendered generically until P6 teaches its label (F-P4-3, QP4-9). |

### 3.1 Planning measurement (scratch, not evidence for any acceptance criterion)

Run once in this planning session to size the design (QS10-11 allows tuning before IC-4 runs; IC-4's
thresholds below are literals from step-17 and are **not** measured here). Release build of
`main @ 5ccc402` on the planning machine (macOS arm64); scratch outside the repository.

```text
mineworld run worlds/social-cafe --headless --seed 7 --days 100 --save S
  wall 8.0 s (release profile); 10.9 s (dev profile, which is [optimized + debuginfo], as CI builds it)
  122 345 facts in the save; head t8639110 (day 100, 23:45:10)
mineworld perceived worlds/social-cafe --save S --person alice --json
  0.6 s; 54 044 facts, ascending ids, 29.2 MB of JSON Lines; 464–652 facts per simulated day
  arrived 28 187 · spoke 14 019 (to Alice 2 120, by Alice 2 039, the rest overheard) ·
  person-entered-place 3 071 · conversation-started 2 649 · group-activity facts ~4 600 ·
  invitations ~1 100 · agenda-changed 301 · relationship-changed 72 · became-acquainted 22 · named 12 ·
  passage-opened 5 · routine-assigned 1
  places: 6 distinct; 314 facts with no place
SDK validation of the whole export (PerceivedEvent.model_validate_json per line): 0.62 s
FTS5 in the workspace interpreter (uv-managed CPython 3.14.5, SQLite 3.50.4): present; bm25() returns
  a float
```

Findings drawn from it are §6.

---

## 4. Reuse comparison (the operator's standing rule; `REUSE_POLICY.md` §§1–4, 11, 12, 19)

Step-17 §4.3 (agent-memory frameworks), §4.4 (summarization) and §4.7 (the store row) already compared
Letta, mem0, LangChain/LangGraph memory, Graphiti, generative-agents reflection, extractive summarizers,
SQLAlchemy, DuckDB and LangGraph's `SqliteSaver`, and chose our own memory over SQLite. Those verdicts
stand and are not repeated. P4 re-opens the question for the two pieces whose fit depends on facts only
now measurable (§3.1): **the store engine** and **lexical retrieval**, adding the candidates the brief
names (sqlite-vec, plain files, LanceDB) and the ones a fair comparison needs. Facts were checked on
2026-10-10 where marked; everything else is marked *verify in C1*.

**Fit** is judged on the step's properties: **D** deterministic and replayable bytes on every platform;
**V** valid with no model or embedding service; **P** three platforms with no native build step;
**I** isolatable behind `MemoryStore` / `Retriever`; **S** the size of what it adds against what P4
needs (one writer per seat, ~54 000 rows per 100 days, lexical match over a few thousand utterances a
week).

### 4.1 The store

| Candidate | Licence | Maturity | D | V | P | I | S | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **stdlib `sqlite3`** (CPython's bundled SQLite) | Python's PSF licence; SQLite public domain | ships with every CPython; SQLite 3.50.4 in the workspace interpreter (measured) | ✓ transactions; ordered reads; content is deterministic given ordered writes (the **file** bytes are not a contract, D-P4-8) | ✓ | ✓ no wheel, no build | ✓ behind `MemoryStore` | ✓ zero new dependencies; already used by P5a's `SqliteLedger` | **REUSE** (`DEP-37`). |
| **Plain files** (JSON Lines per level, appended) | n/a | n/a | ✓ | ✓ | ✓ (Windows replace semantics need care, as P5a's cassettes) | ✓ | ✗ we would build indexes, transactions (cursor atomic with records, P4-2), range queries and lexical search ourselves | **REJECT.** Atomicity of "records + roll-up + cursor" is the property P4-2 needs, and a database gives it for free. Kept as the **canonical dump** format for determinism and review (D-P4-8). |
| **LanceDB** (Python package over a Rust core, Lance columnar format) | Apache-2.0 | active; 0.30.2 on PyPI 2026-03-31 seen on three mirrors, a later 0.34.0 reported but not confirmed (*verify*) | ~ | ✓ for plain tables | ~ native wheels per platform (pyarrow, the Rust core); *verify Windows arm64* | ✓ | ✗ a columnar vector database, built for embeddings and analytics; a large dependency tree for a few small ordered tables | **REJECT for P4; REFERENCE ONLY.** Its strength is vector search over large tables, which QS10-10 defers. Re-evaluate together with embedding retrieval (D-P4-11). |
| **DuckDB** | MIT | mature | ✓ | ✓ | ✓ wheels | ✓ | ~ analytical engine; its full-text search is an extension installed at run time (*verify*: autoload may download), which a network-guarded test must never do | **REJECT** (as step-17 §4.7): analytical, not transactional per-row ingest; the FTS extension's download path conflicts with I-11. |
| **SQLAlchemy / an ORM over SQLite** | MIT | mature | ✓ | ✓ | ✓ | ✓ | ✗ an ORM for four tables | **REJECT** (as step-17 §4.7). |
| **LangGraph `SqliteStore` / checkpointers** | MIT | maintained | ~ | ✓ | ✓ | ✗ brings the LangGraph runtime and its key-value store shape | | **REJECT** (as step-17 §4.3, §4.7): framework lock-in (`REUSE_POLICY.md` §3). |

### 4.2 Retrieval

| Candidate | Licence | Maturity | D | V | P | I | S | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **SQLite FTS5** (in the same database) | public domain | part of SQLite; present in the workspace interpreter (measured); present in python-build-standalone builds (its SQLite flags list `SQLITE_ENABLE_FTS5`, PR #800, checked 2026-10-10); **CI's** interpreters are `actions/setup-python` 3.12 builds, *verify in C1 on all three legs* | ✓ for the match **set**; ✗ for `bm25()` ordering, which is a float (measured) whose value may differ across SQLite versions | ✓ | ✓ if compiled in (the C1 check) | ✓ behind `Retriever` | ✓ nothing new | **REUSE, as a candidate filter only** (`DEP-37`, D-P4-11): FTS5 decides which records match; ranking is our integer rule. |
| **sqlite-vec** (vector search extension) | MIT or Apache-2.0 | pre-1.0 ("expect breaking changes"); 0.1.9 2026-03-31, 0.1.10 alpha in May 2026 (registry mirrors, checked 2026-10-10) | ~ | ✗ needs embeddings, so an embedding model on the memory path | ✗ a loadable extension; `enable_load_extension` is missing from some CPython builds (macOS system Python), *verify* | ✓ | ~ | **REJECT for P4; REFERENCE ONLY.** It is the natural engine for QS10-10's embedding retrieval later, inside the same SQLite file. Re-evaluate then, at 1.0 or later. |
| **rank-bm25** (pure Python BM25) | Apache-2.0 (*verify*) | small, slow-moving | ✗ float scores | ✓ | ✓ | ✓ | ~ in-memory corpus rebuilt per query | **REJECT.** Float scores, and an in-memory index of the whole history, when SQLite already holds an index on disk. |
| **tantivy-py** (Rust full-text engine) | MIT | maintained (*verify version*) | ~ float scores | ✓ | ~ native wheels per platform | ✓ | ✗ a second storage engine beside SQLite | **REJECT.** A second index with its own files and lifecycle, for a lexical match FTS5 already provides. Revisit only if FTS5 is absent on a platform and the LIKE fallback proves too slow. |
| **Chroma / other vector stores** | Apache-2.0 | active | ✗ | ✗ embeddings | ~ | ✓ | ✗ | **REJECT** (QS10-10). |
| **`LIKE` over notable records** (no FTS) | n/a | n/a | ✓ | ✓ | ✓ | ✓ | ✓ | **FALLBACK** (step-17 §15.8): used only if C1 finds FTS5 absent on a CI leg, recorded as a deviation; same integer ranking. |
| **Our own integer ranking** over FTS5's match set | n/a | n/a | ✓ | ✓ | ✓ | ✓ | ✓ a few dozen lines | **CHOSEN** (D-P4-11). |

### 4.3 What remains ours, and why (`REUSE_POLICY.md` §12)

Memory derived from a world's perceived event log, with Event-ID provenance kept by construction and no
model on the ingest path, is MineWorld's core value (`REUSE_POLICY.md` §4); no candidate in step-17 §4.3
or here provides it ("missing required semantics"). The commodity parts (storage, transactions, lexical
indexing) are reused. `DEP-37` records the adopted pieces, the declined ones above with their
re-evaluation triggers, and supersedes step-17's `DEP-S10-d` placeholder.

---

## 5. Design

### 5.1 Package layout (additions to P5a's tree)

```text
cognition/lm-controller/
  pyproject.toml                     + marker real_binary (and nothing else)
  README.md                          + a short "Memory" paragraph
  src/mineworld_cognition/
    memory/
      __init__.py                    the public names of memory
      records.py                     EventKey, Citation, Salience, L0Record, Episode, Chapter, StableFact,
                                     StoreIdentity (frozen, strict models)
      store.py                       MemoryStore: open/create, identity check, schema v1, transactions,
                                     cursor, reads, canonical dump
      names.py                       NameBook: entity → display name, learned from perceived `named`
                                     facts and from P6's observations; generic labels otherwise
      render/
        __init__.py                  Renderer protocol; RENDERERS registry (EventTypeId → Renderer);
                                     render(fact, me, names) -> Rendered
        generic.py                   the envelope-only renderer every unknown type uses (I-9)
        conversation.py              spoke, conversation-started
        presence.py                  arrived, person-entered-place (ambient unless they concern me)
        relationships.py             became-acquainted, relationship-changed (also feed L3's level)
        naming.py                    named (also feeds NameBook)
      ingest.py                      ingest(store, facts, through) — idempotent, ordered, one transaction
      retrieve.py                    RecallQuery, MemorySection, SectionLine, retrieve(store, query)
      fts.py                         FTS5 availability check, the quoted-term query builder, the LIKE
                                     fallback (selected once at store open)
    compress/
      __init__.py
      episodes.py                    the L1 roll-up rule (D-P4-7); called by ingest inside its transaction
      chapters.py                    L2: one simulated week of closed episodes
      stable.py                      L3: per-counterpart stable facts with their establishing ids
      summarize.py                   Summarizer protocol; StructuralSummarizer (default, model-free)
      model_summarizer.py            ModelSummarizer (gateway, tier "summarize"); embellish(store, ...)
  tests/
    memory_fixtures.py               hand-built PerceivedEvent literals (no implementation call)
    binary.py                        locates the mineworld binary (MINEWORLD_BIN or target/debug); fails,
                                     never skips (as sdk/python/tests/realserver.py)
    test_memory_store.py             AP4-1, AP4-2, AP4-3
    test_memory_render.py            AP4-4
    test_memory_compress.py          AP4-5, AP4-6
    test_memory_retrieve.py          AP4-7
    test_memory_summarizers.py       AP4-8, AP4-9
    test_memory_isolation.py         AP4-11 (source scan; store-dump provider scan)
    test_fts5_platform.py            AP4-12 (FTS5 present, or the fallback recorded)
    test_ac10.py                     AP4-10 (IC-4), marked real_binary
    ac10_harness.py                  runs mineworld run / perceived / inspect into tmp_path; the oracles
```

No module is expected past about 300 lines; `store.py` and `retrieve.py` are the largest.

### 5.2 Decisions (D-P4-1 … D-P4-13)

| ID | Decision | Reason |
| --- | --- | --- |
| **D-P4-1** | Memory lives in `mineworld-cognition` as `memory/` and `compress/`, not in a new workspace member. No new dependency: `sqlite3`, `json`, `hashlib`, `re`, `unicodedata` (stdlib), Pydantic (DEP-24) and the SDK. | P5a R-P4-1. A third member would split one Controller Pack across two packages. |
| **D-P4-2** | **Typed records at every boundary.** `L0Record`, `Episode`, `Chapter`, `StableFact`, `Citation`, `StoreIdentity`, `MemorySection` are strict, frozen Pydantic models (`CognitionModel`). `Citation` is a sorted tuple of disjoint, non-adjacent `EventRange(first: EventId, last: EventId)` over **this seat's perceived sequence** (a range covers the perceived ids between its ends, not every world id). | `CLAUDE.md` §4 rule 7. A range over the perceived sequence is exact: an episode is a consecutive run of perceived records (D-P4-7), so one range covers it exactly, and a 291-record episode cites in one token. |
| **D-P4-3** | **One SQLite file per seat, at a path the caller gives.** `MemoryStore.open(path, identity)` creates the file with schema v1, or opens it and refuses, naming both, an identity that differs: `world_instance` (`WorldInstanceId`), `seat` (`EntityKey`), `self_entity` (`EntityId`), or `schema`. `":memory:"` is accepted (tests). The store never chooses a directory and never sits in a save (QS10-17; G-6). | `INV-4`: the save is the world's truth; memory is a seat's. P6 supplies the path from configuration and the identity from `welcome`. |
| **D-P4-4** | **Ids stay strings.** The store orders facts by `event_key`: the decimal id left-padded with `0` to 20 characters (u64's width), which sorts lexicographically exactly as the integers do. No id is converted to `int` or `float` anywhere. | The SDK's rule (ids above 2^53 stay distinct, `ids.py`); SQLite's `INTEGER` is a signed 64-bit, which a u64 id can overflow. |
| **D-P4-5** | **Input is perceived facts only**, as `Sequence[PerceivedEvent]` with the frame's `through`. `ingest` never accepts `Observation.events`. Facts must arrive in ascending id order; a fact whose id is at or below the cursor is a re-delivery: identical content is a no-op, differing content raises `ConflictingFact` naming the id. A fact below the newest ingested id but above the cursor (out of order inside a batch) raises `OutOfOrder`. | step-17 §15.3 (memory must not depend on a reader's speed); §3.4.3 (re-delivery after a crash between receipt and ingestion is idempotent by `EventId`). |
| **D-P4-6** | **Renderers by event type, with a generic default and a salience class.** A `Renderer` maps `(fact, me, names)` to `Rendered(gist: str, counterparts: tuple[EntityId, ...], salience: Salience, mine: bool, level: LevelChange \| None)`. `Salience` is `notable` or `ambient`. Plug-ins exist for the types in §5.1; each decodes `payload.payload` with its own strict model and, on a decode failure, returns the generic rendering (and a counted warning), never an exception. The generic renderer reads only the envelope (`event_type`, `place`, `subjects`, `participants`) and is `notable`. `arrived` and `person-entered-place` are `ambient` unless the person is me. `mine` is true when I am `provenance.controller_decision`'s actor — the store cannot know action ids before P6, so P4 decides `mine` from the payload's actor where a plug-in knows it (`spoke.speaker == me`) and from "I am the only subject" otherwise; P6 may refine it with its decision log. | step-17 §3.8.2 (renderers as plug-ins; I-9). Salience is the measured necessity of §6 F-P4-2: 28 187 of 54 044 perceived facts are other people's strides; shown one by one they would fill every L0 line. Ambient records are still ingested, covered and citable. |
| **D-P4-7** | **The L1 roll-up rule (refined from step-17 §3.9.1; QP4-2).** An episode is a maximal run of consecutive L0 records (in perceived order) such that: the record's **place** (the fact's place, else the place of the previous record) is the episode's place; the gap from the previous record is ≤ 1 800 s of world time; and the record's day (`at // 86 400`) is the episode's day. An `ambient` record never opens or splits an episode: it joins the open one if within the gap, else opens a new one only if no episode is open. The episode's counterpart set is the **union** of its records' counterparts. An episode closes when the next record does not fit, or when ingestion is told the world's time has passed the gap (P6, on an observation). Closing is decided by facts only, never by batch boundaries. | §6 F-P4-1: step-17's "sharing a place **and a counterpart set**" yields 44 137 episodes in 100 days (median size 1) on the real export, which is no compression; same place, gap and day yields 930 (9.3 a day, median 42 records). |
| **D-P4-8** | **Determinism is defined on a canonical dump, not on SQLite file bytes.** `MemoryStore.dump() -> bytes` writes every table in key order as UTF-8 JSON Lines with sorted keys (no floats, no wall-clock values). I-5 compares dumps. | SQLite's file layout (page allocation, freelist, header counters, WAL) is not a contract and differs across SQLite versions, so "byte-identical stores" (step-17 IC-4 (e)) cannot hold across CI's three platforms. The dump is the reviewable content. |
| **D-P4-9** | **Bounds by construction: per-section byte budgets under one ceiling.** The memory section is at most **6 000 bytes** (UTF-8, rendered). Sections and budgets, in order: L3 ≤ 16 lines and 1 600 B; L2 ≤ 4 lines and 800 B; L1 ≤ 8 lines and 1 600 B; L0 ≤ 48 lines and 2 000 B. Every line is cut to ≤ 160 bytes at a character boundary (an ellipsis marks a cut), citation token included. A section fills in its order of preference until either bound; a line that does not fit is not partly emitted. The rendered section therefore never exceeds 6 000 B, whatever the store holds. | step-17 §3.9.1 and QS10-11: tuned **before** IC-4 runs. 16 + 4 + 8 + 48 lines at full length would be 12 160 B, over the ceiling; the budgets keep the counts as caps and make the ceiling hold by construction. |
| **D-P4-10** | **`ModelSummarizer` is in P4, off by default, and touches text only.** It takes a `ModelGateway` (tier `summarize`, chosen by P6's caller through `Router.for_tier`) and the seat's `EntityKey`. `embellish(store, summarizer, *, limit)` is an async pass over closed episodes and chapters without prose: per summary, one `CompletionRequest(purpose="summarize", …)` whose messages hold the structural text (names only, no ids); `Completed` → the prose is validated (≤ 160 bytes; no digit run that looks like an id; no delimiter token) and stored with its cassette key; `Refused`, `Failed`, or `None` (tier unbound) → nothing stored, the structural text stands; `CassetteMiss` propagates. Ingestion and retrieval never await a model. | step-17 §3.9.2, §3.9.4 (two summarizers behind one interface from the start; prose is an embellishment, never the index). The brief: summarizer tests use scripted or recorded backends via P5a's gateway. |
| **D-P4-11** | **Retrieval: structured selection, FTS5 as a filter, integer ranking.** `retrieve(store, RecallQuery(at, counterparts, words))` returns a `MemorySection`: (1) L3, top counterparts by times met (ties: earliest first meeting, then entity id string); (2) L2 chapters, newest first; (3) L1 episodes, in two passes: those whose counterparts include one in the query, newest first; then, when `words` is non-empty, episodes containing a `notable` L0 record that FTS5 matches, ranked by the number of **distinct query terms** matched (an integer), ties by recency, then by the episode's first `event_key`; (4) L0 `notable` records within the last 86 400 s before `at`, newest first. A record already shown at a higher section is not repeated. `bm25()` is never used. | step-17 §3.8.6, kept, with the float-ordering hazard of §3.1 removed (P4-3). Matching on the episode's records, not on its structural text, finds the umbrella line even when the summary does not quote it (Milestone D). |
| **D-P4-12** | **FTS5, checked once per process, with a recorded fallback.** `fts.available()` creates an FTS5 table in `:memory:`; the store creates `l0_text USING fts5(gist, tokenize='unicode61 remove_diacritics 2')` when available. Query terms: the trigger's words, NFKC-normalized, lower-cased, split on non-alphanumerics, length ≥ 3, the first 8 distinct, each written as a quoted FTS5 string with `"` doubled, joined by `OR`. Without FTS5, the same terms drive `LIKE` over `notable` gists with `ESCAPE`; same ranking. | step-17 §15.8; P4-4 (player words are data, never `NEAR`, `*`, column filters or boolean syntax). |
| **D-P4-13** | **No second `MemoryStore` implementation.** SQLite `":memory:"` is the in-memory store for tests. `MemoryStore` is a concrete class, not a protocol. | `CLAUDE.md` §4 rule 11. Step-17 §3.16.3 planned an in-memory implementation "because the tests need it"; `":memory:"` meets that need with the production code path (QP4-4). |

### 5.3 The store, schema v1

```text
meta(key TEXT PRIMARY KEY, value TEXT)              schema=1, world_instance, seat, self_entity,
                                                    cursor (EventId or ""), newest (event_key or "")
l0(event_key TEXT PRIMARY KEY, event_id TEXT, at INTEGER, kind TEXT, place TEXT,
   salience TEXT CHECK (salience IN ('notable','ambient')), mine INTEGER CHECK (mine IN (0,1)),
   gist TEXT, digest TEXT)                          digest: sha256 of the fact's canonical JSON, for
                                                    ConflictingFact (D-P4-5)
l0_who(event_key TEXT, entity TEXT, PRIMARY KEY (event_key, entity))
l0_text (FTS5, notable gists; rowid aligned with an l0_rowid side table) — or absent (fallback)
episodes(episode INTEGER PRIMARY KEY, day INTEGER, place TEXT, first_key TEXT, last_key TEXT,
         opened_at INTEGER, closed_at INTEGER, closed INTEGER, records INTEGER, text TEXT,
         prose TEXT, prose_key TEXT)                prose_key: P5a's CassetteKey; never a model or
                                                    binding name (I-10)
episode_who(episode INTEGER, entity TEXT, PRIMARY KEY (episode, entity))
chapters(week INTEGER PRIMARY KEY, first_episode INTEGER, last_episode INTEGER, text TEXT,
         prose TEXT, prose_key TEXT)
stable(entity TEXT PRIMARY KEY, first_key TEXT, last_key TEXT, times_met INTEGER, exchanges INTEGER,
       level TEXT, level_key TEXT)
names(entity TEXT PRIMARY KEY, name TEXT, source_key TEXT)
```

- Episode numbers are assigned in closing order, so they are a function of the facts (D-P4-8).
- Every write of one `ingest` call, the roll-up it triggers and the cursor update are one transaction
  (`BEGIN IMMEDIATE … COMMIT`); an exception rolls everything back (P4-2).
- Connections are opened and closed by the store's owner (`close()`, and a context manager), so no
  handle outlives use; on Windows an open handle blocks deletion of `tmp_path` (P5a D-P5-13's lesson).
- `journal_mode` stays the default (`DELETE`), so a store is one file to copy or delete.

### 5.4 Ingestion, one call

```text
ingest(store, facts: Sequence[PerceivedEvent], through: EventId) -> IngestReport
  BEGIN IMMEDIATE
  for fact in facts (ascending ids, checked):
    re-delivery → compare digest → no-op | ConflictingFact
    rendered = RENDERERS.get(fact.event_type, generic)(fact, me, names)
    names.learn_from(fact)                 # `named` only, through its plug-in
    insert l0, l0_who, l0_text (notable)
    episodes.feed(record)                  # may close the open episode (D-P4-7)
    stable.feed(record, rendered.level)    # first/last/exchanges/level for each counterpart
  chapters.close_weeks(up_to=newest closed episode's day)
  meta.cursor = through ; meta.newest = last event_key
  COMMIT
```

`IngestReport` counts records, re-deliveries, decode fallbacks and closed episodes, for P6's log.
`NameBook.learn(entity, name)` is the call P6 makes from observations; the offline path learns person
names from `named` facts alone (F-P4-3).

### 5.5 The levels, exactly

| Level | Unit | Rule | Citation | Text (structural) |
| --- | --- | --- | --- | --- |
| L0 | one perceived fact | every fact, as rendered | its own id | the gist: `Bob said to me: «…»`, `I said to Bob: «…»`, `Bob said to Carol: «…»`, `Bob arrived`; unknown types: `<event type> at <place label>, with <names>` |
| L1 | episode | D-P4-7 | one range over the perceived sequence | `Day 12, 09:10–09:40, <place label>: talked with Bob (6 lines: 3 his, 3 mine; he said «…»); 4 others came and went.` — counts and at most one quote, chosen as the newest notable utterance addressed to me, else the newest notable |
| L2 | chapter | the closed episodes of one simulated week (`day // 7`) | the ranges of its episodes, merged | `Week 3: 61 episodes, mostly at <place>; most often with Bob (14), Carol (9); first met Dev.` |
| L3 | stable fact per counterpart | from every record naming the counterpart | `first_key` (first perceived fact naming them), `level_key` (the newest level change), `last_key` | `Bob: met on day 1, last seen day 99, met 212 times, 1 380 exchanges, friendly.` |

Quotes are delimited by `«` and `»`; a delimiter inside an utterance is replaced by `"`, so quoted words
cannot close the quote (step-17 §3.13, prompt injection). Place labels come from `NameBook`; an unnamed
place renders as `somewhere`, an unnamed person as `someone` (P4-5).

### 5.6 The memory section

`MemorySection` holds `lines: tuple[SectionLine, ...]` where `SectionLine = (level, text, citation)`,
and `render() -> str`: one header per non-empty section and one line per entry, each line ending with
its citation token (`[#1890-1907]` or `[#1890]`; several ranges comma-separated). P6 embeds the rendered
text in the decision context and validates `recalls` against the union of the section's citations.
`MemorySection.citations()` returns that union. The byte bound is D-P4-9's.

### 5.7 The summarizers

```python
class Summarizer(Protocol):
    def episode(self, records: Sequence[L0Record], names: NameBook) -> str: ...
    def chapter(self, episodes: Sequence[Episode], names: NameBook) -> str: ...
```

- `StructuralSummarizer` implements §5.5's templates. Pure: the same records and names give the same
  bytes. It is the only summarizer ingestion calls.
- `ModelSummarizer` (D-P4-10) is not a `Summarizer` that ingestion calls; it is the `embellish` pass,
  which reads the structural text back from the store. It therefore cannot be on the `AC-10` path.

### 5.8 What P6 will call (the seam, typed)

```text
MemoryStore.open(path, StoreIdentity) / close() / dump() / cursor() -> EventId | None
ingest(store, facts, through) -> IngestReport
NameBook.learn(entity, name)              (through the store)
retrieve(store, RecallQuery) -> MemorySection
embellish(store, ModelSummarizer, limit) -> EmbellishReport      (async; optional)
```

Nothing here names a seat session, a frame, a provider or a decision. P6 owns when each is called.

### 5.9 Platforms

- `sqlite3` and FTS5 on CPython 3.12 (CI) and 3.14 (the workspace default), on Linux, macOS and Windows:
  checked by `test_fts5_platform.py` on all three legs (AP4-12). If a leg lacks FTS5, C1 records it as a
  bounded deviation and the `LIKE` fallback is what that leg tests.
- Paths are `pathlib.Path`, given by the caller; no default directory (step-17 §15.8).
- The harness runs `mineworld` with the platform's executable suffix (`binary.py`), as the SDK's
  `realserver.py` does.

### 5.10 The AC-10 scenario and CI

- `test_ac10.py` is marked `real_binary`. In the `python` layer, which builds `mineworld-cli` first, it
  runs; `python-smoke` adds `-m "not real_binary"` to the cognition command, visibly, as it already
  deselects `real_server` for the SDK. A missing binary fails the test, never skips it (D-P3-10).
- Cost: the 100-day run took 10.9 s with the dev profile CI builds (§3.1), the export 0.6 s, validation
  0.6 s; ingestion is measured in C4 (A-5). The whole test is expected under 60 s per leg; over 120 s on
  any leg is a stop with the measurement (§12).

---

## 6. Findings from the planning measurement

| ID | Finding | Evidence | Decision |
| --- | --- | --- | --- |
| F-P4-1 | Step-17's L1 rule ("sharing a place **and a counterpart set**") does not compress the real history. | Scratch prototype over the 100-day export: 44 137 episodes (441 a day, median 1 record). Same place + gap ≤ 30 min + day boundary, counterparts as union: 930 episodes (9.3 a day, max 17; median 42 records, max 291; at most 11 counterparts). | D-P4-7; QP4-2 |
| F-P4-2 | Other people's strides dominate what a Person perceives. | `arrived` 28 187 of 54 044; `person-entered-place` 3 071. | Salience classes (D-P4-6) |
| F-P4-3 | Places carry no `named` fact; only the 12 people do. Offline, places have no label. | `named` 12, all `{person, name}`; `places/*.yaml` have tags and no `name`. | Generic place label offline; P6 teaches labels from observations (QP4-9) |
| F-P4-4 | FTS5's `bm25()` is a float. | Probe: `bm25 = -1e-06` for a single-row match. | Integer ranking (D-P4-11) |
| F-P4-5 | A 100-day run is cheap enough to generate the AC-10 fixture inside the test. | 10.9 s dev profile; 8.0 s release. | No committed fixture; the test runs the binary (§5.10, QP4-6) |
| F-P4-6 | "Byte-identical stores" cannot hold across SQLite versions. | SQLite file format: page layout and header counters are not a content contract. | Canonical dump (D-P4-8; QP4-3) |

---

## 7. Test ownership (test rules §26)

```text
STATIC     ruff; pyright strict: NewType ids never mixed (EventId vs EntityId vs EventKey), Salience a
           Literal, GateOutcome handled with match/assert_never in ModelSummarizer, no Any. No unit test
           re-proves these.
UNIT       store identity and schema refusal; event_key order at u64 extremes; idempotent and ordered
           ingest; atomicity; renderer literals (heard, said, overheard, generic, decode fallback);
           the L1 rule on hand-built sequences (gap 1 800 vs 1 801 s, place change, day boundary,
           ambient joining); L2 weeks; L3 statistics against hand-computed values; retrieval bounds,
           ranking and ties, FTS query quoting; StructuralSummarizer golden lines; ModelSummarizer over
           ScriptedBackend behind a real ModelGateway (every GateOutcome; prose validation; citations
           unchanged); record → replay round trip in tmp_path; CassetteMiss propagation.
INTEGRATION
           AC-10 (IC-4) over the real binary: mineworld run (100 days) → mineworld perceived → ingest in
           live-sized frames → retrieve at days 10, 30, 60, 100 → assertions (a)–(g). The store-dump
           provider scan after an embellish pass.
GATE 1     NOT REQUIRED: no LLM-facing behaviour is accepted in P4. ModelSummarizer's real-model prose
           quality is not a P4 claim; it is off by default. No agent runs a model (QS10-19).
GATE 2     The AC-10 scenario is the real-lifecycle evidence (a real world run, the real export, the real
           store on disk), run in CI's python job on three platforms.
CI IMPACT  the cognition pytest command gains the new tests (unit ~seconds; AC-10 ~30–60 s per leg,
           measured in C7); python-smoke deselects real_binary; Rust layers unchanged; core unchanged.
```

---

## 8. Acceptance and adversarial criteria (fixed now, before anything is measured)

Bounds are literals from the requirement or from step-17, never from the implementation (`ARC-23`).
Each criterion names the mutation that must turn it red; mutations are planted, seen red, reverted, and
recorded in §14. Oracles never call the code under test.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AP4-1** Identity | (a) Opening a store with a different `world_instance`, `seat`, `self_entity` or `schema` raises `StoreIdentityMismatch` naming the field and both values; the file is unchanged (dump before = after). (b) A new store records all four. | Skip the instance comparison: (a) opens silently. |
| **AP4-2** Ids and order | (a) `event_key` of `"9"` sorts before `"10"`, and `"18446744073709551615"` (u64 max) sorts last; ingesting facts with those ids in ascending order succeeds and `cursor()` returns the exact string. (b) A re-delivered identical fact is a no-op (dump unchanged); the same id with a changed utterance raises `ConflictingFact`; a batch `[5, 3]` raises `OutOfOrder`. (c) No `int(` or `float(` is applied to an id in `memory/` or `compress/` (scan). | Order by `CAST(event_id AS INTEGER)`: u64 max overflows and (a) fails. Make re-delivery overwrite: (b)'s conflict passes silently. |
| **AP4-3** Durable before the cursor | An exception raised from inside ingestion on the 3rd of 5 facts (a renderer the test registers that raises) leaves the dump byte-identical to before and `cursor()` unchanged. | Update the cursor before the records, outside the transaction: the cursor moves. |
| **AP4-4** Rendering | (a) Literals: a `spoke` from Bob to me renders `Bob said to me: «Hello.»`; mine to Bob `I said to Bob: «…»`; Bob to Carol `Bob said to Carol: «…»`; names from `named` facts. (b) An utterance containing `»` cannot close the quote. (c) **I-9:** a synthetic event type `fished` from a pack the code has never seen is ingested, rendered generically, covered by an episode, and citable; no exception, no warning. (d) **P4-5:** across every rendered gist, episode, chapter and stable text in the AC-10 store, no entity id occurs (the oracle: every entity id string in the export, matched as a whole token). (e) A `spoke` whose payload lacks `utterance` renders generically and counts one decode fallback. | (c) Raise on an unknown type: (c) fails. (d) Render an unnamed person as `entity-<id>`: (d) names the line. |
| **AP4-5** The L1 rule | On hand-built sequences: gap 1 800 s joins and 1 801 s splits; a place change splits; crossing `at // 86 400` splits; a `null` place joins the open episode; an ambient record of another person never opens or splits an episode; counterparts are the union. | Require an equal counterpart set (step-17's wording): the union case splits. |
| **AP4-6** L2 and L3 | Hand-computed over a 3-week fixture: chapter weeks and their merged citations; for one counterpart, `first_key`, `last_key`, `times_met`, `exchanges` and `level` (from `relationship-changed.to`) equal the literals written in the test. | Compute `first_key` from the newest chapter only: the literal differs. |
| **AP4-7** Retrieval | (a) Every section respects its line and byte budget; the rendered section is ≤ 6 000 B on an adversarial store of maximum-length (480-byte, multibyte) utterances. (b) A counterpart in the query brings their episodes first. (c) Words match a `notable` record inside an older episode, which is returned with its citation; ties break by recency, then by first `event_key`. (d) **P4-4:** words `umbrella" OR "x`, `NEAR(a b)`, `gist:foo`, `*`, `-x` raise nothing and match only literal terms. (e) **P4-3:** two retrievals of one store give identical sections; `retrieve.py` imports no clock, no `random`, and never selects `bm25`. | (a) Drop the byte budget: the adversarial store exceeds 6 000. (d) Pass words unquoted to `MATCH`: `OperationalError`. |
| **AP4-8** StructuralSummarizer | Golden: one hand-built episode and one chapter render to literal strings written in the test; the same inputs twice give identical bytes. | Include a wall-clock timestamp: the second call differs. |
| **AP4-9** ModelSummarizer | Behind a real `ModelGateway` over `ScriptedBackend`, with a `FakeClock` and a `MemoryLedger`: (a) every request has `purpose="summarize"` and contains no event id and no entity id; (b) **P4-1:** after `embellish`, every summary's citation is byte-identical to before (compare dumps with prose columns removed), and only `prose`/`prose_key` changed; (c) `Refused` (budget exhausted), `Failed(timeout)` and an unbound tier (`None`) store no prose and raise nothing; (d) prose over 160 bytes, or containing a digit run of 3 or more, is discarded and counted; (e) record through `RecordingBackend` into `tmp_path`, then replay through `ReplayBackend`: identical prose and `prose_key`s; a changed structural text raises `CassetteMiss`, which `embellish` propagates; (f) the backend's call count equals the number of summaries embellished (one call each, no retry). | (b) Let prose carry a citation it parses from the completion: citations change. (e) Catch `CassetteMiss` as a fallback: the replay passes silently. |
| **AP4-10** **`AC-10` (IC-4)** over the real binary | Fixed: `mineworld run worlds/social-cafe --headless --seed 7 --days 100 --save <tmp>`; `mineworld perceived … --person alice --json`; `mineworld inspect` for the instance; Alice's `EntityId` from the `named` fact whose name equals `alice.yaml`'s `name`. Ingest in frames of 256 facts. For each day d ∈ {10, 30, 60, 100}, the trigger is the first `spoke` whose listener is Alice with `at ≥ (d−1)·86 400 + 43 200`; ingest every fact up to it, then `retrieve(at=trigger.at, counterparts={speaker}, words=utterance)`. **(a) Bounded:** every section ≤ 6 000 B, and day 100's ≤ 1.10 × day 30's. **(b) Provenance resolves:** every id in every citation in the store (not only the sections) is in Alice's export, which `mineworld perceived` read from the save. **(c) Coverage:** Alice's exported ids equal, exactly and disjointly, the ids of the open (unclosed) L0 tail ∪ the closed episodes' ranges. **(d) No omniscience:** five `spoke` facts located in Bob's export and absent from Alice's (place ≠ Alice's whereabouts at that time, from the export) appear in no row and no text of the dump. **(e) Determinism:** two stores built from the same export give byte-identical dumps. **(f) Reach-back:** at day 100, the L3 line of each listed counterpart cites, as `first_key`, the earliest fact in Alice's export naming that counterpart (computed by the test from the export). **(g) Batch independence:** ingesting in frames of 1, 256 and the whole export gives byte-identical dumps. | Feed the union of every person's export: (b) and (d) fail. Disable the L1 roll-up (episodes never close): (c) fails. Drop one closed episode's range: (c) fails. Remove D-P4-9's budgets: (a) fails. Compute L3 over the last 7 days only: (f) fails. Close the open episode at the end of each batch: (g) fails. |
| **AP4-11** Isolation and provider-freedom | (a) A source scan of `memory/` and `compress/` finds no `subprocess`, no `world.sqlite`, no `open(` other than through `MemoryStore`, and no import of `config`, `record`, `secrets` or any `backend` adapter module (`ModelSummarizer` imports `gateway` and `backend.model` only). (b) P5a's provider scan (`TERMS`) over the canonical dump of a store embellished through a gateway whose binding is `local` and whose model name is `qwen-test`: no match. (c) `memory/` and `compress/` are inside the scan's scope (not in `ALLOWED`). | (b) Store the cassette entry's `meta.model` beside the prose: `qwen` is found. |
| **AP4-12** Every platform | The whole cognition suite, AP4-10 included, passes on `ubuntu-24.04`, `windows-2025` and `macos-15` in the PR's CI; `test_fts5_platform.py` reports FTS5 present (or C1's recorded deviation names the leg and the fallback under test). | Leave a store connection open past the test: Windows fails to remove `tmp_path`. |
| **AP4-13** Scope | `git diff --stat <base>..HEAD` touches only `cognition/lm-controller/**`, `docs/DECISIONS.md`, `docs/ARCHITECTURE.md` (§8), `docs/CORE_CONCEPTS.md` (§5.4), `scripts/ci_layer.py` (`python-smoke` only), `.structured-coding/**`, and `uv.lock` only if the member's metadata change requires it. No `*.rs`, nothing under `sdk/python/`, `worlds/`, `systems/`, `.github/`. | — (a diff gate) |

---

## 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-P4-1 | CI's setup-python 3.12 builds lack FTS5 on one platform. | AP4-12 in C1, before anything depends on it; the `LIKE` fallback behind the same query function (D-P4-12), recorded. |
| R-P4-2 | The 100-day test is slower on Windows. | Measured in C7 on all legs; the stop at 120 s (§12). The test is one module-scoped fixture: one run, one export, every assertion. |
| R-P4-3 | World behaviour changes (S15 re-baselines, new packs) change the export. | No committed fixture and no literal counts: every oracle is computed from the export of the current `main` (`ARC-23`'s "located, not counted"). |
| R-P4-4 | The store grows without bound (~54 000 L0 rows per seat per 100 days; size measured in C7). | `AC-10` bounds the context, not the store (step-17 §3.9.1). Forgetting is MVP-1 (QP4-11). |
| R-P4-5 | CJK utterances are not matched word by word: `unicode61` treats a run of CJK characters as one token. | Recorded limitation (QP4-8); lexical retrieval is the fourth and last selection pass; a trigram or ICU tokenizer is a `Retriever` follow-up. |
| R-P4-6 | Renderer plug-ins drift from pack payloads. | Decode failures fall back to the generic rendering and are counted (D-P4-6); AP4-10 runs on the real packs, so a drift shows as a fallback count, which the test asserts is 0 on social-cafe. |
| R-P4-7 | The refined L1 rule (D-P4-7) or the IC-4 refinements (D-P4-8, (c), (f), (g)) are read as moving acceptance after measuring. | They are argued from §3.1's prototype counts, which measure **compression ratio**, not IC-4's criteria; IC-4's literals (6 000 B, 1.10, 100 days, seed 7) are step-17's, unchanged. QP4-2 and QP4-3 put them before the primary session before freeze. |
| R-P4-8 | `mine` is approximate before P6's decision log exists. | Recorded in D-P4-6; P6 may refine it. No P4 criterion depends on `mine` beyond the `spoke.speaker` literal. |

---

## 10. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately. `[x]` needs the work
and its evidence. An item that turns out not to apply is `N/A`, with the audited reason. Commands
assume the worktree root, with `uv` and `cargo` on `PATH`. Targeted validation per commit; no local
full suite (test rules §8); the PR's CI is the one full run.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Start the implementation context against the frozen design. The freeze header, the
  contract's authority lines and the decision numbers (`ARC-59`, `DEP-37`) were filled by the planning
  session at freeze (§12, §13.1); C0 verifies them rather than writing them.
- **Scope.** This document (§14 opened); a new `handoff-s10-p4.md`.
- [x] Implementation: verify the `DESIGN FROZEN` header, §12's authority lines and their sources, and
  that `overall.md`'s table lists `ARC-59` and `DEP-37` for P4; record the implementation base commit;
  initialize the handoff with the contract's required fields. Evidence: §14.1 C0.
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  Evidence: §14.1 C0 (193 sections, none duplicated; 104 ids, all distinct).
- [x] Review: every authority line has a source, and none was widened. Evidence: §14.1 C0.
- **Commit boundary.** Documentation only.

### C1 — Decisions, spec edits, the platform check, the marker

- **Goal.** The decisions exist before the code (`CLAUDE.md` §2.2); FTS5 is known present (or the
  fallback chosen) on all three legs before anything depends on it.
- **Scope.**
  - `docs/DECISIONS.md`: `ARC-59` — memory is a derivation of perceived facts; compression
    L0–L3 with citations kept by construction; the model may rewrite text, never citations; the store
    is the operator's, outside the save; determinism on the canonical dump. `DEP-37` — stdlib
    `sqlite3` and FTS5 as a filter with integer ranking; §4's declines (LanceDB, DuckDB, SQLAlchemy,
    LangGraph stores, plain files, sqlite-vec, rank-bm25, tantivy-py, Chroma) with re-evaluation
    triggers; supersedes step-17's `DEP-S10-d`.
  - `docs/ARCHITECTURE.md` §8: remove `cognition_cache/`; one sentence that the cognition store lives in
    the operator's cognition directory because it is not world state (G-6).
  - `docs/CORE_CONCEPTS.md` §5.4: G-2's sentence.
  - `cognition/lm-controller/pyproject.toml`: marker `real_binary`.
  - `scripts/ci_layer.py`: `python-smoke`'s cognition command gains `-m "not real_binary"`.
  - `tests/test_fts5_platform.py` and `memory/fts.py` (`available()`, the query builder).
- [x] Implementation: the files above (evidence §14.1 C1; one extra file, §14.3 X-1).
- [ ] Validation (all local items done, §14.1 C1; the per-leg FTS5 line waits on the first CI run):
  - ruff, ruff format, pyright strict: zero findings;
  - `uv run --locked pytest cognition/lm-controller -k fts5` locally;
  - `python3 scripts/ci_layer.py --list python-smoke` shows the deselection; `--list core` and
    `--list python` byte-identical to the base's except as intended;
  - both doc checks;
  - **the PR's first CI run** reports FTS5 on all three legs (record per leg; this is the one place a
    pushed head is needed early — the branch is pushed after C1, and the draft PR opened).
- [x] Review:
  - each DECISIONS entry names its alternatives and a revisit trigger (`REUSE_POLICY.md` §§11–12);
  - G-2 and G-6 match step-17 §12's wording;
  - the query builder quotes every term (P4-4).
  Evidence: §14.1 C1.
- **Failure cases.** FTS5 absent on a leg: record a bounded deviation, keep `LIKE` as that leg's path,
  continue. `--strict-markers` refuses an undeclared marker: the marker is declared in this commit.

### C2 — Records and the store

- **Goal.** A seat's store opens, refuses a foreign identity, orders ids correctly, and dumps
  canonically.
- **Scope.** `memory/records.py`, `memory/store.py`, `memory/__init__.py`, `tests/memory_fixtures.py`,
  `tests/test_memory_store.py` (AP4-1, AP4-2 (a), (c)).
- [x] Implementation: D-P4-2, D-P4-3, D-P4-4, D-P4-8, D-P4-13; schema v1 (§5.3); context-managed close.
  Landed with C3 and C4 in one commit (`8db4626`; §14.1 C2–C4).
- [x] Validation: AP4-1; AP4-2 (a), (c); a store created, closed and reopened in `tmp_path` keeps its
  identity and cursor. Evidence: §14.1 C2–C4; mutations M1, M2 (§14.2).
- [x] Review: no id becomes an `int`; no connection outlives `close()`; the dump has no float and no
  wall-clock value. Evidence: §14.1 C2–C4 (review).

### C3 — Renderers, names and ingestion

- **Goal.** Perceived facts become L0 records, idempotently, in order, atomically with the cursor.
- **Scope.** `memory/render/**`, `memory/names.py`, `memory/ingest.py`; `tests/test_memory_render.py`
  (AP4-4 (a)–(c), (e)); `tests/test_memory_store.py` (AP4-2 (b), AP4-3). Ingestion calls the roll-up in
  `compress/episodes.py`. If C3 alone would need a temporary stand-in for it, C3 and C4 land as one
  commit instead; the implementing session decides by reviewability and records the mapping.
- [x] Implementation: D-P4-5, D-P4-6; `ingest` per §5.4; `IngestReport`. Commit `8db4626` (C2–C4).
- [x] Validation: AP4-2 (b); AP4-3; AP4-4 (a)–(c), (e); ingestion time for the 100-day export measured
  once locally and recorded (A-5). Evidence: §14.1 C2–C4; mutations M3, M4, M5.
- [x] Review: every plug-in falls back on a decode failure; quotes are injection-safe; `names.learn`
  is the only write path for names. Evidence: §14.1 C2–C4 (review).

### C4 — Compression: L1, L2, L3 and the structural summarizer

- **Goal.** Episodes, chapters and stable facts, each with exact citations, decided by facts only.
- **Scope.** `compress/episodes.py`, `compress/chapters.py`, `compress/stable.py`,
  `compress/summarize.py`; `tests/test_memory_compress.py` (AP4-5, AP4-6, AP4-8).
- [x] Implementation: D-P4-7; §5.5; `StructuralSummarizer`. Commit `8db4626` (C2–C4).
- [x] Validation: AP4-5; AP4-6; AP4-8; batch independence on a fixture (frames of 1 and of all).
  Evidence: §14.1 C2–C4; mutations M6, M7, M11.
- [x] Review: no closing decision depends on a batch boundary; every closed episode's range is
  consecutive in the perceived sequence; chapters close only on fully past weeks. Evidence: §14.1
  C2–C4 (review).

### C5 — Retrieval and the memory section

- **Goal.** A bounded, deterministic section with citations, from structure first and words last.
- **Scope.** `memory/retrieve.py`, `memory/fts.py` (the match side); `tests/test_memory_retrieve.py`
  (AP4-7).
- [x] Implementation: D-P4-9, D-P4-11, D-P4-12; `MemorySection.render()` and `citations()`. Commit
  `a03b0fb`; the (d) fixture strengthened in C8 (§14.2 M9).
- [x] Validation: AP4-7 (a)–(e). Evidence: §14.1 C5; mutations M8, M9, M10.
- [x] Review: no float reaches an ordering; the budgets are applied after line cutting; a line is never
  partly emitted. Evidence: §14.1 C5 (review).

### C6 — `ModelSummarizer` and `embellish`, through P5a's gateway

- **Goal.** Optional prose that can never alter provenance, tested with no model.
- **Scope.** `compress/model_summarizer.py`; `tests/test_memory_summarizers.py` (AP4-9);
  `tests/test_memory_isolation.py` (AP4-11).
- [x] Implementation: D-P4-10; the summarize request template (names only); prose validation. Commit
  `590f30a`.
- [x] Validation: AP4-9 (a)–(f); AP4-11 (a)–(c). Evidence: §14.1 C6; mutations M12, M13, M14.
- [x] Review: every `GateOutcome` case handled (`assert_never`); `CassetteMiss` not caught; no
  provider concept or binding name reaches the store. Evidence: §14.1 C6 (review).

### C7 — `AC-10` over the real binary (IC-4)

- **Goal.** The acceptance criterion, shown on a real 100-day history, with every mutation red.
- **Scope.** `tests/ac10_harness.py`, `tests/binary.py`, `tests/test_ac10.py` (AP4-10, AP4-4 (d)).
- [x] Implementation: the harness (subprocess calls to the binary, into `tmp_path`); a module-scoped
  fixture; assertions (a)–(g); the decode-fallback count asserted 0. Commit `d421b18`.
- [ ] Validation (local items done, §14.1 C7 and §14.2 A1–A7; the per-leg CI wall times wait on the
  PR's first full run, §14.4):
  - `cargo build -p mineworld-cli`, then `uv run --locked pytest cognition/lm-controller -k ac10`;
    wall time recorded;
  - the six mutations of AP4-10 each planted, run, seen red, reverted (§14.2);
  - the store's size on disk after 100 days, recorded (R-P4-4);
  - the PR's CI: all three legs, wall time of `test_ac10.py` per leg.
- [x] Review: no oracle calls `mineworld_cognition`; located facts are chosen by rule, not by literal
  ids; the harness reads the save only through the CLI. Evidence: §14.1 C7 (review).

### C8 — Close-out

- **Goal.** Review readiness.
- **Scope.** This document's ledger (§14), the README paragraph, `handoff-s10-p4.md`; parent
  synchronization marked pending for the S10 planning session (step-17 §15, overall `AC-10` row).
- [x] Implementation: ledger complete; deviations recorded; README. Evidence: §14.1 C8.
- [ ] Validation: both doc checks; the final head's CI green on every job.
- [ ] Review: AP4-13's diff gate; every `[x]` carries evidence.

---

## 11. Requirements this PR places on other PRs

- **R-P6-2 (on P6).** P6 opens one `MemoryStore` per seat at `CognitionConfig.store / <instance> /
  <seat>.sqlite` (or the layout P6 freezes), with the identity from `welcome`; calls `ingest` for every
  `perceived` frame before advancing the SDK cursor; teaches names from observations
  (`NameBook.learn`); builds the decision context's memory part from `retrieve(...).render()`; validates
  `recalls` against `MemorySection.citations()`; and calls `embellish` only when `summarize` is bound.
- **R-P3b-1 (on P3b).** The SDK's perceived cursor is advanced by the caller after a successful
  `ingest`, never by the session on receipt (step-17 §3.4.3).
- **R-P7-1' (on P7).** Milestone D's "same memory" check reads the cited ranges through
  `MemorySection.citations()`; IC-8's mutation (store deleted, re-ingestion disabled) uses P4's store.

## 12. Execution contract (filled at freeze, 2026-10-10)

Source of every line marked "primary session": its freeze rulings of 2026-10-10, relayed by the
coordinator to the S10 planning session: "implementation authorized for a fresh session; merge only
after primary review". Endpoints the rulings do not name individually (push, PR, CI repair) take the
working rules' shipped defaults (§21), which that authorization does not narrow. Lines marked "S10
convention" follow P5a's contract (`pr-s10-p5-backends.md` §11) and were not separately ruled.

```text
PROJECT / PR:            MineWorld mvp0, S10 PR P4 — subjective memory, compression L0–L3, retrieval,
                         AC-10
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p4-memory.md
RELATED / BINDING DOCS:  step-17-cognition.md (§§3.8, 3.9, 3.16.3, 4.3, 4.4, 4.7, 5, 6, 10, 15);
                         pr-s10-p5-backends.md (gateway, recorder, R-P4-1); pr-s10-p3-python-sdk.md;
                         step-12-server.md §17 (perceived; ARC-43); overall.md (S10; decision table);
                         docs/ARCHITECTURE.md §8; docs/CORE_CONCEPTS.md §5; docs/ENGINEERING_STANDARDS.md
                         (§§22–24); docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s10-p4, its own, held by one
                         session (CLAUDE.md §3.1)                                  (S10 convention)
BRANCH:                  mvp0/pr-s10-p4-memory, created from main                   (S10 convention)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records it
APPROVED SCOPE:          §2.1, as frozen
FROZEN INVARIANTS:       §2.3; D-P4-1 … D-P4-13 as ruled; QS10-11; QS10-17; QS10-18; QS10-19; every platform
SEQUENCE:                C0 … C8
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv *; python3 scripts/*; target/*/mineworld *;
                         mkdir -p; sed -n                                          (S10 convention)
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading
                         ~/.config/mineworld/secrets.env or any key file; using any API key; calling any
                         hosted API; running, pulling or downloading any model; running
                         tools/model_spike.py
MATERIAL STOPS:          any Rust, sdk/python, world or system change; any new dependency (P4 adds none);
                         a change to IC-4's literals (6 000 B, 1.10, 100 days, seed 7, days 10/30/60/100);
                         a change to D-P4-7 or D-P4-9 after AP4-10 has run once; test_ac10.py over 120 s
                         on any CI leg; FTS5 absent on every leg
PLATFORMS:               Linux, macOS, Windows (AP4-12)
VALIDATION BUDGET:       unit, static, local integration (incl. 100-day runs): unrestricted. Real model
                         calls: none by agents. CI: the PR's runs; no manual dispatch
LIVE DOCUMENTATION:      this document (§14)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s10-p4.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized, in a fresh session
                                                                 (primary session, 2026-10-10)
                         semantic commits: authorized            (primary session; working rules §14)
                         branch push: authorized                 (primary session; working rules §21)
                         PR creation / update: authorized        (primary session; working rules §21)
                         CI repair to review readiness: authorized
                                                                 (primary session; working rules §21)
                         merge: never by the implementation session. Only after the primary session's
                         review, and only with explicit operator authorization (primary session,
                         2026-10-10; working rules §22)
POST-MERGE SYNC OWNER:   the S10 planning session owns step-17 §15 and overall.md; the implementation
                         session owns this document's ledger, evidence, deviations and remaining issues
                                                                 (S10 convention, as P5a)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 13. Questions

**[operator]** marks a question only the operator can answer (scope, cost, a paid API, a reversal of an
operator ruling). **[primary]** marks one the primary session decides. No question here needs a model,
a key or money.

### 13.1 Rulings, 2026-10-10

| ID | Ruling |
| --- | --- |
| QP4-1 | **Primary:** `ARC-59` for P4 and `DEP-37` for the store and retrieval record. The primary session records both in `overall.md`'s decision-number table. `ARC-72 … ARC-74` stay with P6. |
| QP4-2 | **Primary:** the refined episode rule is accepted (same place, gap ≤ 30 min, same day, people merged; D-P4-7). |
| QP4-3 | **Primary:** the IC-4 changes are accepted. Each is stricter or makes a criterion hold across platforms; no threshold is relaxed. |
| QP4-4 … QP4-10 | **Primary:** as recommended. |
| QP4-11 | **Operator, 2026-10-10:** no pruning in P4; forgetting goes to MVP-1. C7 measures and records store growth per seat per 100 days (R-P4-4). |
| Freeze | **Primary session, 2026-10-10:** P4 DESIGN FROZEN; implementation authorized for a fresh session; merge only after the primary session's review (§12). |

### 13.2 The questions as asked (kept for review)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QP4-1 [primary]** | Decision numbers: `ARC-59` for P4's architecture record (leaving `ARC-72 … ARC-74` to P6), and which DEP number for the store and retrieval record (next free: `DEP-37`)? | **`ARC-59` and `DEP-37`**, recorded in `overall.md`'s table before P4 merges, so the reservation does not live only in this document (the 2026-10-09 collision lesson). |
| **QP4-2 [primary]** | Accept D-P4-7's L1 rule (same place, gap ≤ 30 min, day; counterparts as union) in place of step-17 §3.9.1's "sharing a place and a counterpart set"? | **Yes.** F-P4-1: the step-17 rule yields 441 episodes a day (median size 1) on the real export; the refined rule 9.3 a day. It changes no IC-4 literal. |
| **QP4-3 [primary]** | Accept the IC-4 refinements: determinism on the canonical dump, not SQLite file bytes (D-P4-8); coverage as "open L0 tail ∪ closed episodes, exact and disjoint"; the mutation "disable the L1 roll-up" asserted through (c), not (a), because (a) holds by construction (D-P4-9) and the budget's own mutation is added; two added criteria, (f) reach-back and (g) batch independence? | **Yes.** Each makes IC-4 stricter or makes a step-17 criterion satisfiable across platforms; none relaxes a literal. |
| **QP4-4 [primary]** | Drop step-17 §3.16.3's in-memory `MemoryStore` implementation, and use SQLite `":memory:"` for tests (D-P4-13)? | **Yes.** One implementation, exercised by every test; no protocol for a single implementation (`CLAUDE.md` §4 rule 11). |
| **QP4-5 [primary]** | `ModelSummarizer` and `embellish` in P4 (off by default, scripted and replayed in tests), or deferred to P6? | **In P4.** The step's "two summarizers behind one interface from the start" and the brief's "summarizer tests via P5a's gateway" place it here; it touches no seat. |
| **QP4-6 [primary]** | The AC-10 test runs the real binary inside the cognition suite (`real_binary` marker, deselected by `python-smoke`), generating its 100-day save each run (10.9 s measured with CI's profile), rather than committing a 29 MB fixture? | **Yes.** No fixture drift, no large file, and the same audience function as live (step-17 I-4). Stop at 120 s on any leg. |
| **QP4-7 [primary]** | The section budgets of D-P4-9 (6 000 B total; L3 1 600, L2 800, L1 1 600, L0 2 000; 160 B per line), tuned before IC-4 runs (QS10-11)? | **Accept**, then freeze with the design; never retuned after AP4-10's first run (a material stop). |
| **QP4-8 [primary]** | CJK utterances: accept `unicode61`'s limitation for P4 (a run of CJK characters is one token), with a trigram or ICU tokenizer as a `Retriever` follow-up? | **Yes, for P4.** Lexical match is the last of four selection passes; structure (counterparts, recency, L3) carries recall. Revisit before any Chinese-language world is a demo target. |
| **QP4-9 [primary]** | Places have no `named` fact (F-P4-3). Accept generic place labels (`somewhere`) offline, with P6 teaching labels from observations through `NameBook.learn`, rather than adding place names to worlds? | **Yes.** Naming places is world content and a naming-pack question, outside P4; the bound and provenance criteria do not depend on labels. |
| **QP4-10 [primary]** | Day boundary = `WorldTime` seconds `// 86 400`, as `mineworld inspect` counts days, without consulting a calendar pack? | **Yes.** Memory reads no calendar state; a calendar's dates are a later refinement through perceived `day-began` facts if a world needs it. |
| **QP4-11 [operator — scope]** | Forgetting: the store keeps every L0 row (estimated tens of MB per seat per 100 days; measured in C7). Should P4 prune old rows? | **No pruning in P4.** `AC-10` bounds context, not storage; forgetting is part of MVP-1's memory evolution (`MVP.md` §9.2). |

## 14. Ledger (live during implementation)

Opened at C0 by the implementation session (2026-10-10), worktree
`/Users/yuema137/mineworld-worktrees/impl-s10-p4`, branch `mvp0/pr-s10-p4-memory`.

### 14.1 Per-commit evidence

**C0 — freeze and contract.**

- Implementation base: `origin/main @ 573c205` (#139). The branch was first cut from `f9626c4` (#138)
  and fast-forwarded to `573c205` before any edit, so that `overall.md`'s decision table is the one P4
  builds on.
- Verified: the `DESIGN FROZEN 2026-10-10` header (revision 2, approved by the primary session's rulings
  QP4-1 … QP4-10 and the operator's QP4-11, lifecycle FROZEN); §12's endpoint lines, each with its source
  (primary session 2026-10-10, or the S10 convention, or working rules §§14, 21, 22), none widened;
  `overall.md` l. 918 lists `S10 P4 | ARC-59 (memory and compression) | DEP-37 (SQLite store with FTS5
  matching)`.
- Handoff initialized: [`handoff-s10-p4.md`](handoff-s10-p4.md).
- Validation: `python3 scripts/check_doc_headings.py` → "193 numbered sections across 26 documents, none
  duplicated"; `python3 scripts/check_decision_ids.py` → "104 decision ids, all distinct". PASS.
- Session recovery (2026-10-10): the first implementation session stopped right after writing this
  ledger start and the handoff, before committing. The replacement session (sole writer of the worktree)
  audited both diffs against §12 and the repository, found them correct, and committed them as C0.
  `origin/main` has since moved to `bb62edf` (#140, `step-23-release.md` only); the base stays `573c205`,
  since nothing P4 depends on changed.

**C1 — decisions, spec edits, the platform check, the marker.**

- Files: `docs/DECISIONS.md` (`ARC-59`, `DEP-37`, appended after `ARC-60`); `docs/ARCHITECTURE.md` §8
  (`cognition_cache/` removed; one paragraph: the store is the operator's, not world state, G-6);
  `docs/CORE_CONCEPTS.md` §5.4 (G-2's sentence); `cognition/lm-controller/pyproject.toml` (marker
  `real_binary`); `scripts/ci_layer.py` (`python-smoke`'s cognition command); `memory/__init__.py`,
  `memory/fts.py` (`available()`, `query_terms`, `fts_string`, `like_pattern`);
  `tests/test_fts5_platform.py`; `tests/test_structural_isolation.py` (X-1).
- Static: `ruff check` "All checks passed!"; `ruff format --check` clean after `ruff format`; `pyright`
  "0 errors, 0 warnings, 0 informations". PASS.
- `uv run --locked pytest cognition/lm-controller -k "fts5 or ci_runs" -s`: 4 passed; the platform line
  `[fts5] darwin arm64 CPython 3.14.5 SQLite 3.50.4 FTS5=True`. PASS (local leg only).
- `python3 scripts/ci_layer.py --list` for `core`, `python` and `fast`: byte-identical to the base's
  (`573c205`); `python-smoke` differs only in its cognition line, now
  `uv run --locked pytest cognition/lm-controller -m 'not live_model and not real_binary'`. PASS.
- Doc checks: 193 sections, none duplicated; 106 decision ids, all distinct (`ARC-59`, `DEP-37` added).
- Review: `ARC-59` lists options (a)–(d) and its revisit triggers; `DEP-37` lists §4's eleven candidates
  with verdicts, the limitation (CJK, FTS5 must be compiled in) and its triggers. G-2's sentence is step-17
  §12's verbatim plus the `ARC-59` link; G-6 follows step-17 §12 (the store lives in the operator's
  cognition directory because it is not world state). `fts_string` quotes and doubles; `query_terms`
  already strips `"` (split on non-alphanumerics), so the doubling is a second guard.
- FTS5 per CI leg: recorded in §14.4 with the PR's first full run (X-2).

**C2–C4 — store, renderers, ingestion, compression (one commit, `8db4626`).** C3's ingestion calls C4's
roll-up, so they land together, as §10 C3 allows; C2's tests use `ingest`, so it joins them.

- Files: `memory/{__init__,records,store,names,ingest}.py`, `memory/render/{__init__,conversation,
  presence,relationships,naming}.py`, `compress/{__init__,episodes,chapters,stable,summarize}.py`;
  `tests/memory_fixtures.py`, `tests/test_memory_{store,render,compress}.py`. Largest module `store.py`
  (264 lines); the longest function `ingest` (≈45 lines after `_write` was split out).
- Validation (targeted): the three test files plus `test_fts5_platform`, `test_provider_scan`,
  `test_structural_isolation`, run on the staged tree alone (later files stashed): 40 passed; ruff and
  pyright clean. PASS.
- A-5, measured once on the planning export regenerated at this base (`mineworld run … --seed 7 --days
  100`, dev profile, 9.7 s; Alice's export 54 044 facts): SDK validation 0.9–1.2 s, ingestion in frames
  of 256 into a file store **1.45–2.1 s**, 0 decode fallbacks, 923 closed episodes, 14 chapters, dump
  19.6 MB. The design's scratch prototype counted 930 episodes (§6 F-P4-1); its handling of null places
  and ambient records was not recorded, so the difference of 7 is noted, not explained. It is a
  compression ratio, which no criterion bounds.
- Review: ids are strings end to end (`event_key` pads, `event_of` strips; the AP4-2 (c) scan finds only
  three `int(` conversions, all of booleans); `MemoryStore.close` and the context manager close the one
  connection, and `test_a_store_releases_its_file_when_closed` deletes the file after closing; the dump
  holds integers and strings only (no `at` is a wall clock: `at` is world time). Every plug-in returns
  `fallback(...)` when `decode` fails; `quote` replaces `«`/`»` and folds whitespace, so speech cannot
  close a quote or break a line; `NameBook.learn` is the only `INSERT INTO names`. Closing is decided in
  `Episodes.feed` from the record and the stored open episode only; an episode is a run of consecutive
  keys by construction; `close_weeks` writes a week only when a closed episode exists in a later week.

**C5 — retrieval (`a03b0fb`).** `memory/retrieve.py`; `tests/test_memory_retrieve.py`.

- Validation: 10 passed (AP4-7 (a)–(e)); the adversarial store (sixty days of 480-byte, three-byte-per-
  character utterances, twenty long names) renders every section within its line and byte budget, every
  line ≤ 160 bytes, the whole ≤ 6 000 bytes, all four sections used. PASS.
- Review: every ordering key is an integer or a string (`times_met`, `first_key`, `episode`, `closed_at`,
  `event_key`); `cut` runs before a line is measured, and `_Section.offer` adds a line whole or marks the
  section full; `bm25` appears only in the module's docstring.

**C6 — `ModelSummarizer` and `embellish` (`590f30a`).** `compress/model_summarizer.py`;
`tests/test_memory_{summarizers,isolation}.py`.

- Validation: 7 + 3 passed (AP4-9 (a)–(f), AP4-11 (a)–(c)), behind a real `ModelGateway` over
  `ScriptedBackend`, `RecordingBackend` and `ReplayBackend`, a fake clock and a `MemoryLedger`; no
  network, no model. PASS.
- Review: `match outcome` covers `Completed`, `Refused`, `Failed`, then `assert_never`; nothing catches
  `CassetteMiss` (M13); the dump of a store embellished through a recording whose model is `qwen-test`
  has no provider term (AP4-11 (b)); only `prose` and `prose_key` are written (M12).

**C7 — `AC-10` over the real binary (`d421b18`).** `tests/{binary,ac10_harness,test_ac10}.py`.

- Gate 2 specification (written before the run): claim — on a real 100-day history, Alice's memory
  section stays ≤ 6 000 B and does not grow with age, every citation resolves into her perceived set,
  coverage is exact, nothing unperceived appears, the store is deterministic and batch-independent;
  owner — Gate 2 (real binary, real export, real store on disk); command — `uv run --locked pytest
  cognition/lm-controller/tests/test_ac10.py -s` after `cargo build -p mineworld-cli`; PASS evidence —
  8 tests green with the printed section sizes; counterfactuals — §14.2 A1–A7.
- Result, local (macOS arm64, dev profile, head `d421b18`): **PASS**, 8 passed in 29.9 s. Wall: run
  11.2 s, two exports 4.0 s, ingestion with four retrievals 2.1 s; the remainder is the three extra
  dumps of (e) and (g). Section bytes: day 10 4 424, day 30 4 909, day 60 4 905, day 100 4 915 (day 100 /
  day 30 = 1.001 ≤ 1.10). Decode fallbacks 0. Five unperceived `spoke` facts located in Bob's export.
- R-P4-4: the store on disk after 100 days is **19 521 536 bytes** (19.5 MB) for one seat; the canonical
  dump 19.6 MB.
- Review: `ac10_harness.py` imports `binary` and the SDK only; it locates triggers, Alice's whereabouts,
  the unperceived facts and each counterpart's first naming by rule over the exports; the save is read
  only through `mineworld run`, `inspect` and `perceived`.

**C8 — close-out.** README "Memory" paragraph; this ledger; the handoff; `test_memory_retrieve.py`'s
hostile-words fixture strengthened after M9 survived in part (§14.2). AP4-13's diff gate,
`git diff --name-only 573c205` filtered by the allowed list: no file outside it (no `*.rs`, nothing
under `sdk/python/`, `worlds/`, `systems/`, `.github/`; `uv.lock` unchanged). Both doc checks pass.
Parent synchronization (step-17
§15, `overall.md`'s `AC-10` row) is pending for the S10 planning session, which owns it (§12).

### 14.2 Mutations

(Each planted, run, observed red, reverted with `git checkout -- <file>`; local, macOS arm64.)

| ID | Criterion | Mutation planted | Observed |
| --- | --- | --- | --- |
| M1 | AP4-1 | `_check_identity` skips `world_instance` | RED: `test_a_foreign_identity…[world_instance]` |
| M2 | AP4-2 (a) | `event_key` stops padding (the code never orders by `CAST`; an unpadded key is the same defect: `"9" > "10"`) | RED: `test_ids_order_as_integers…`, and every test that ingests ids of mixed widths |
| M3 | AP4-2 (b) | a re-delivery with a different digest overwrites it | RED: `test_a_redelivery_is_a_no_op…` |
| M4 | AP4-3 | the cursor written before the transaction, outside it (first form also fed the new cursor into the order check, which turned ten tests red for the wrong reason; replanted so only the cursor moves) | RED: `test_a_failure_inside_ingestion…` (dump differs: cursor 24) |
| M5 | AP4-4 (c) | `render` raises on an unknown type (`RENDERERS[...]`) | RED: `test_an_event_type_no_code_knows…` |
| M6 | AP4-5 | a notable record must have the episode's counterpart set | RED: union, ambient and chapter tests |
| M7 | AP4-6; AP4-10 (f) | `stable.first_key` follows every newer fact (L3 over a recent window) | RED: `test_stable_facts…`, and on the real history `test_f_each_counterpart_reaches_back…` |
| M8 | AP4-7 (a); AP4-10 (a) | `_Section.offer` without the byte budget; then without any budget | RED: adversarial store 6 158 B > 6 000; on the real history `test_a_bounded…` |
| M9 | AP4-7 (d) | the trigger's raw words passed to `MATCH` unquoted | first run: RED for `umbrella" OR "x` (`fts5: syntax error`) and the ranking test, but **survived** for `NEAR(a b)` and `gist:foo`, which parse as valid FTS5 and matched nothing in the fixture either way. Fixture strengthened (C8: the words `near`, `gist`, `foo` now occur), rerun: RED for those two as well. `*` and `-x` yield no term, so no query runs in either form: they assert "raises nothing" only |
| M9' | P4-4 | terms quoted → bare (`fts_string` skipped, terms still split) | equivalent: `query_terms` yields lower-case alphanumeric runs, which FTS5 reads as barewords; the quoting is the second guard, recorded, not testable apart from M9 |
| M10 | AP4-7 (e) | `import random` in `retrieve.py` | RED: `test_retrieval_is_deterministic…` |
| M11 | AP4-8 | a clock reading appended to the chapter text | RED: golden test, chapters, batch independence |
| M12 | AP4-9 (b) | `embellish` also rewrites an episode's `first_key` | RED: `test_prose_replaces_text_only…` |
| M13 | AP4-9 (e) | `CassetteMiss` caught and skipped | RED: `test_recorded_prose_replays…` and the isolation import scan |
| M14 | AP4-11 (b) | `prose_key` carries `:qwen-test` | RED: `test_an_embellished_store_holds_no_provider_concept` |
| A1 | AP4-10 (b), (d) | the store fed the union of Alice's and Bob's exports | RED: `test_b…`, `test_d…` |
| A2 | AP4-10 (c) | the roll-up disabled (`_fits` always true: episodes never close) | RED: `test_c…` (the open tail spans 100 days) |
| A3 | AP4-10 (c) | closed episode 100's row deleted | RED: `test_c…` (coverage differs) |
| A4 | AP4-10 (a) | D-P4-9's budgets removed | M8's second form: RED |
| A5 | AP4-10 (f) | L3 over a recent window | M7: RED |
| A6 | AP4-10 (g) | the open episode closed at the end of every batch | RED: `test_g…` (and six fixture tests); 161 s under the mutation |
| A7 | AP4-4 (d) | an unnamed person rendered `entity-<id>` | RED: `'entity-8 arrived'` named |
| — | AP4-12 | a store connection left open | not observable on macOS (POSIX deletes an open file); the Windows leg runs `test_a_store_releases_its_file_when_closed`, but a red Windows run would need a CI run beyond the PR's own (§12 budget), so it was not obtained (X-13) |

### 14.3 Deviations and discoveries

**X-1 (bounded; C1) — `python-smoke` must restate `not live_model`.**

```text
Previous assumption: python-smoke "adds -m \"not real_binary\" to the cognition command" (§5.10).
Audit evidence:      the member's addopts end in `-m "not live_model"` (pyproject.toml); pytest parses
                     addopts before the command line and `-m` is a single-valued option, so a second
                     `-m` replaces the first rather than combining with it. tests/test_structural_
                     isolation.py::test_ci_runs_pytest_once_per_member pinned the cognition line of both
                     layers to the bare command.
Corrected:           the python-smoke cognition command is
                     `-m "not live_model and not real_binary"`; the isolation test pins each layer's
                     exact line (python: bare; python-smoke: with that expression) and still refuses any
                     line that selects live_model.
Impact:              none on scope: the same deselection the design asked for, without silently
                     re-selecting live_model tests. One extra file touched, inside
                     cognition/lm-controller/** (AP4-13).
Validation:          `ci_layer.py --list python-smoke`; the isolation test passes.
```

All further entries are bounded: none changes a frozen invariant, an IC-4 literal, D-P4-7 or D-P4-9, a
public contract, ownership, a dependency, or scope.

**X-2 (C1) — a draft PR runs only `fast`.** `.github/workflows/ci.yml` runs `python` (and `test`) on
non-draft pull requests only. The per-leg FTS5 evidence C1 planned "on the PR's first CI run" therefore
comes from the first non-draft run (the PR is marked ready at C8). The LIKE fallback exists behind the
same function, so nothing depended on FTS5 meanwhile. No scratch branch was pushed (the contract's CI
budget is the PR's runs).

**X-3 (C6) — `model_summarizer.py` also imports `backend.canonical`.** AP4-11 (a) says `ModelSummarizer`
imports `gateway` and `backend.model` only. It stores each prose's cassette key (D-P4-10), which is
`backend.canonical.cassette_key`; recomputing the hash here would duplicate the key scheme. `canonical`
is neither an adapter, `config`, `record` nor `secrets`; the isolation test pins the exact set of
outside imports (`gateway`, `backend.model`, `backend.canonical`).

**X-4 (C7) — AP4-4 (d)'s oracle made stricter than a token match.** The literal oracle ("every entity id
string in the export, matched as a whole token") cannot be met on social-cafe: its entity ids are `1` …
`18`, which are also days, hours and counts in §5.5's frozen templates (`Day 12`, `met 212 times`). The
oracle used removes every labelled template field (written in `ac10_harness.LABELLED` as literals) and
every `«…»` quote, then requires that **no digit at all** remains. An id can hide nowhere, and the
`entity-<id>` mutation is red (A7). Section lines cut at 160 bytes drop their last `; ` part before the
check (a half label no longer reads as one); the whole text of every stored line is checked uncut.

**X-5 (C4) — the `Summarizer` protocol takes two keyword inputs.** `episode(records, names, *, place)`
and `chapter(episodes, names, *, newcomers)`: the episode's place is the roll-up's (D-P4-7), which its
records alone do not determine when an ambient record from elsewhere opened it; "first met" (§5.5) needs
the counterparts whose first naming falls in the week, which `chapters.py` reads from L3.

**X-6 (C4) — the L1 template is pack-neutral.** `compress/` knows no event type, so the episode line
counts notable records ("6 notable moments, 3 mine") and ambient passers ("4 others came and went"),
and quotes one **gist**: the newest notable record that is not mine, else the newest notable. §5.5's
"newest utterance addressed to me" would need a per-record "I took part" flag that schema v1 does not
hold. Golden literal: AP4-8.

**X-7 (C3, C4) — who a record concerns.** A record's counterparts are the fact's **participants** other
than me, for every renderer (the envelope's own statement; `passage-opened` lists places as subjects and
no participants). L3: `first_key` is the first perceived fact naming them as a participant (the oracle
of AP4-10 (f) uses the same definition, computed from the export); `times_met` counts closed episodes
whose counterparts include them (ambient presence included, as D-P4-7's union does); `exchanges` counts
notable facts with them in which I am a participant; `level` is the `to` of the newest
`relationship-changed` whose `person` is me. `became-acquainted` sets no level (the payload states none).

**X-8 (C7) — AP4-10 (c) also bounds the open tail.** With episodes that never close, "open tail ∪
closed ranges" would still cover everything. The test therefore takes the open tail to be the one open
episode and requires its records to share one day, so the "roll-up disabled" mutation is red (A2).

**X-9 (C3) — closing on elapsed time is P6's.** D-P4-7 mentions closing "when ingestion is told the
world's time has passed the gap (P6, on an observation)". P4 closes on facts only; an open episode's
records stay in the open L0 tail, covered and citable. P6 adds the call when it has observations.

**X-10 (C2) — `Citation` enforces sorted and disjoint, not non-adjacent.** Adjacency is defined over the
seat's perceived sequence, which a citation value cannot see. Episodes are consecutive, so a chapter
cites one span; L3 cites up to three single facts.

**X-11 (C2) — the FTS5 index uses `l0`'s own rowid.** `l0` is a rowid table, so `l0_text.rowid =
l0.rowid` aligns them without the `l0_rowid` side table §5.3 sketches. The index is never dumped.

**X-12 (C2) — `NotAMemoryStore`.** Opening a SQLite file that has tables but no `meta` (a world's save,
for instance) is refused rather than turned into a store.

**X-13 (C2) — AP4-12's mutation is not observable off Windows.** See §14.2: the test runs on the
Windows leg; its red form was not obtained.

**X-14 (C4) — day numbers.** The split rule uses `at // 86 400` (QP4-10); texts print that plus one,
as `mineworld inspect` counts days (`t8639110 (day 100, …)`).

**X-15 (C5) — section headers.** `People I know:`, `Recent weeks:`, `Episodes I remember:`, `The last
day:`; each section's bytes include its header and newlines, so the 6 000-byte bound covers the whole
rendered text.

**X-16 (C6) — readings of two criteria.** AP4-9 (d)'s "no delimiter token" is read as the citation
token's mark `#` (prose may never look like provenance); a completion whose `finish` is not `complete`
is also discarded. AP4-11 (a)'s source scan allows `def open(` (the store's constructor) and the FTS5
probe's `sqlite3.connect(":memory:")`; every other `open(`, `os.`, `Path(`, `subprocess` or
`world.sqlite` is a finding.

### 14.4 CI runs per head

| Head | Run | Event | Result |
| --- | --- | --- | --- |
| `3de3664` (C1) | 38035409127 | `pull_request`, draft | `changes`, `fast` pass; every other job skipped (draft, X-2) |
| `84caaf1` (C8) | 38037138795 | `pull_request`; the push and `gh pr ready` landed two seconds apart, the `ready_for_review` run (38037136527) was cancelled by the workflow's concurrency group and the surviving run still read the PR as a draft | `fast` only; `python`, `test` skipped. Re-triggered by the next push (this ledger entry) |
