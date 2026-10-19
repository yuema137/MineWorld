## DEP-37 — The memory store: standard-library `sqlite3`, with FTS5 as a match filter and integer ranking

**Date** 2026-10-10 · **Status** adopted by S10 PR P4 (no new dependency: CPython's `sqlite3` and the
SQLite it bundles) · **Approved by** the primary session (QP4-1, QP4-4, QP4-8, 2026-10-10) ·
**Licence** SQLite: public domain; `sqlite3`: the PSF licence · **Supersedes** step-17's placeholder
`DEP-S10-d` and the memory half of `DEP-S10-f` · **Relates to** `ARC-59`, `ARC-57` (`SqliteLedger`
uses the same module) · **Design** `pr-s10-p4-memory.md` §4, D-P4-3, D-P4-8, D-P4-11, D-P4-12, AP4-12

**Problem.** One seat's memory is about 54 000 records per 100 simulated days, written by one process,
read back in order and by range, and searched lexically for the words of a trigger. Records, roll-ups
and the cursor must commit together (the cursor is never ahead of what is durable). Bytes must be
reproducible, and everything must work with no model and no embedding service, on Linux, macOS and
Windows, with no native build step.

**Options considered** (facts checked 2026-10-10):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| **stdlib `sqlite3`** | PSF; SQLite public domain; ships with CPython | **Adopted**: transactions, ordered reads, zero new dependencies |
| plain JSON Lines files per level | n/a | Rejected as the store: we would rebuild indexes, transactions and lexical search; kept as the **canonical dump** format |
| LanceDB | Apache-2.0; 0.30.x | Rejected for P4, reference only: a columnar vector database with native wheels, for a few small ordered tables; re-evaluate with embedding retrieval |
| DuckDB | MIT; mature | Rejected: analytical rather than per-row transactional; its full-text search is an extension that may be downloaded at run time, which a network-guarded test must never do |
| SQLAlchemy or another ORM | MIT | Rejected: an ORM for a handful of tables |
| LangGraph `SqliteStore` / checkpointers | MIT | Rejected: brings the LangGraph runtime and its key-value shape (`REUSE_POLICY.md` §3) |
| **SQLite FTS5** (same database) | public domain | **Adopted as a candidate filter only**: it decides which records match; its `bm25()` score is a float and is never used |
| sqlite-vec | MIT or Apache-2.0; pre-1.0 | Rejected for P4, reference only: needs embeddings and a loadable extension, which some CPython builds disable; re-evaluate with embedding retrieval, at 1.0 or later |
| rank-bm25 | Apache-2.0 | Rejected: float scores, and an in-memory index of the whole history |
| tantivy-py | MIT | Rejected: a second storage engine beside SQLite, for a match FTS5 already provides |
| Chroma and other vector stores | Apache-2.0 | Rejected: embeddings (QS10-10) |
| `LIKE` over notable records | n/a | **Fallback** only, behind the same query function, if a platform's CPython lacks FTS5 |

**Choice.** `MemoryStore` (`memory/store.py`) is a concrete class over `sqlite3`: one file per seat,
`journal_mode` left at `DELETE` (one file to copy or delete), every ingest call one `BEGIN IMMEDIATE`
transaction, ids stored as strings with a 20-character zero-padded ordering key (a u64 id can overflow
SQLite's signed `INTEGER`). `":memory:"` is the in-memory store for tests; there is no second
implementation. FTS5 (`tokenize='unicode61 remove_diacritics 2'`) indexes the gists of notable
records. A trigger's words become quoted FTS5 strings, never query syntax, and matching episodes are
ranked by the number of distinct terms matched (an integer), then recency, then the first Event ID.

**Why not ourselves.** Storage, transactions and lexical indexing are commodity; SQLite provides all
three in the interpreter we already require. What stays ours is the derivation and the ranking rule,
which no candidate provides with Event-ID provenance.

**Limitation accepted.** `unicode61` treats a run of CJK characters as one token, so CJK triggers match
only whole runs (QP4-8); a trigram or ICU tokenizer is a follow-up behind the retrieval seam. FTS5 must
be compiled into the interpreter; P4 checks it on every CI platform (`test_fts5_platform.py`).

**Revisit** if a platform's CPython ships without FTS5, if embedding retrieval is adopted (sqlite-vec or
LanceDB, re-evaluated then), or if CJK worlds become a demo target.
