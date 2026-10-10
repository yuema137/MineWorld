# PR S6-SR — Save retention: pruned snapshots, compressed snapshot rows, the log kept whole (`F-SAVE-1`)

## DESIGN FROZEN 2026-10-10 (primary session)

```text
Design revision:        revision 2 (2026-10-10): revision 1 (PR #145, first commit) with §13.1's
                        rulings filled in and the decision numbers assigned
Approved by / evidence: the operator's rulings on QSR-1, QSR-3, QSR-4 and the primary session's rulings
                        on QSR-2, QSR-5, QSR-6, all of 2026-10-10, relayed by the coordinator to the
                        persistence-lane design session (§13.1)
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §12 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§2.1, with QSR-5's CI assertion), invariants (§2.3), decisions D-SR-1 … D-SR-9, the acceptance
and adversarial criteria ASR-1 … ASR-12 and the commit plan are frozen. Progress, evidence, findings and
bounded corrections stay writable (§14). No SR question remains open. Implementation starts in a fresh
session (§12); this planning session does not implement.

*Superseded header:* `DESIGN DRAFT — awaiting the primary session's review and the operator's rulings
(not frozen)`, revision 1.

**Effort:** `mvp0` · **Step:** S5/S6 persistence follow-up, [`step-06-persistence.md`](step-06-persistence.md)
(§1.2 non-goal "log compaction, snapshot pruning", §9 L-6) · **Parent:** [`overall.md`](overall.md) §5
"Saves grow without bound (finding F-SAVE-1, 2026-10-09)"; the decision-number table.
**Raised by:** S19 TW-b/TW-d, [`step-19-time-weather.md`](step-19-time-weather.md) F-TWbd-1 and QTWd-2
(routed to the persistence lane as `F-SAVE-1`). **Carried as a risk by:** S23,
[`step-23-release.md`](step-23-release.md) §"Risk carried" and R-R5.
**Working name:** SR (save retention). The PR number is assigned when the PR opens.
**Planning worktree:** `/Users/yuema137/mineworld-worktrees/design-save-retention`, branch
`docs/save-retention-design`.

**Decision numbers (QSR-6, ruled 2026-10-10 by the primary session).** **`ARC-81`** is SR's
architecture record (the retention rule and the snapshot codec, amending `ARC-25`'s accepted
limitation). **`DEP-43`** is the `zstd` dependency record. Revision 1 wrote them as the placeholders
`ARC-SR` and `DEP-SR`. The primary session records both in `overall.md`'s decision-number table. An
implementation session never picks a number.

### Binding rules this design is written under

| Source | Rule | Where this design meets it |
| --- | --- | --- |
| `ARC-25` | The fact log is HISTORY: append-only, never derived, never rewritten. State is the journal re-executed; snapshots are checkpoints, *never an authority on its own*. Restart = newest snapshot ≤ head + journal tail, byte-compared. `verify` re-executes from genesis and compares every stored snapshot. | §2.3 I-SR-1 … I-SR-4; D-SR-1, D-SR-6 |
| `ARC-25` accepted limitation | "The log is kept whole; compaction and snapshot pruning are later work." | This PR is that later work for snapshots only; the log stays whole (D-SR-1). |
| `ARC-27` | `AC-12` covers the facts, the journal **and the snapshots** of a run, byte for byte, as a function of pack, seed, age, pace and code; a re-run of a killed command finishes the same world. | D-SR-3 (retention is a function of the stored set and the head, nothing else); ASR-5, ASR-6 |
| `ARC-49` / `AC-8` | `scripts/ci_parity.py` hashes every stored byte of a 30-day save on macOS, Linux and Windows. | D-SR-5 (deterministic codec, bundled and pinned); ASR-7 |
| `DEP-2` | `rusqlite` behind `PersistenceBackend`; the backend stores opaque encoded rows and knows nothing about meaning. | D-SR-4, D-SR-7: the codec lives in `format.rs`; the backend only deletes the revisions it is told to |
| `step-06` I-1 (`INV-11`) | No fact is ever rewritten, deleted, re-numbered or repaired. | I-SR-1; ASR-11 |
| `step-06` I-5 / `ARC-25` | A save of another format is refused, never decoded on a guess. | D-SR-8: `SAVE_FORMAT` 3; ASR-8 |
| `CLAUDE.md` §4.16, `REUSE_POLICY.md` | Reuse before reinvention; both adoption and rejection recorded. | §5; `DEP-43` |
| Operator, all platforms | macOS, Linux and Windows alike. | §7; ASR-7 |
| `ARC-55` / `deny.toml` | A dependency's licence is one of MIT, Apache-2.0, BSD-2/3, ISC, Zlib, CC0, Unlicense, Unicode-3.0. | §5 (rejects LGPL and proprietary options); `DEP-43` |

---

## 1. Goal, in one paragraph

A MineWorld save grows by a full world snapshot every 64 revisions and never drops one: a 30-day Market
Town save measures 357 MiB and a 300-day one 3.53 GiB, of which 80–84 % is snapshots (§4). Snapshots
are checkpoints, not authority (`ARC-25`), so almost all of them can go. This PR keeps, in one
deterministic rule, the genesis snapshot, one **anchor** every 4 096 revisions and the **two newest**
scheduled snapshots, deletes every other snapshot in the same SQLite transaction that writes the next
one, and stores each snapshot row as a zstd frame of its unchanged JSON encoding. The fact log and the
journal are not touched: no fact or journal row is deleted, rewritten or re-encoded. The measured
projection for a 300-day Market Town save is ≈ 587 MiB (−84 %), bounded thereafter by the authoritative
log's own growth (≈ 1.9 MiB per simulated day). Compressing the log itself — which would bring the same
save to ≈ 40 MiB — is designed in §6.9 as a separate follow-up (SR-b) because it changes how the
authority is stored and therefore needs its own ruling (QSR-1).

## 2. Scope

### 2.1 In scope

- A retention rule for snapshot rows (D-SR-2, D-SR-3), applied in `PersistentWorld::commit` and executed
  by the backend inside the revision's transaction.
- A snapshot codec: zstd level 3 frames of the existing `serde_json` encoding (D-SR-4, D-SR-5), with
  `verify` comparing decoded JSON bytes.
- `SAVE_FORMAT` 2 → 3 (D-SR-8).
- `verify_from`: re-execute from a retained anchor to the head, comparing every later retained snapshot
  (D-SR-6) — the instrument for "replay from any retained anchor".
- `mineworld inspect` reports the retained snapshot revisions (D-SR-9).
- The 30-day size bound (ASR-2) as an assertion in CI (QSR-5): in `tools/cli/tests/market_town.rs`,
  which already runs `--days 30 --save C` (and `--days 300 --save A`) in the default suite. The 300-day
  bound (ASR-1) is a recorded measurement, not an assertion.
- The dependency `zstd` (`DEP-43`), the decision `ARC-81`, and the spec edits they require.

### 2.2 Not in scope, and where it goes

```text
compressing or segmenting the fact log and the journal       SR-b, §6.9, after QSR-1
dropping journal or facts before an anchor ("checkpoint       rejected: contradicts ARC-25 and INV-11 (§5.7)
  compaction")
a migration of format-2 saves                                 refused by name, as ARC-25 requires; QSR-3
a user-facing retention setting (CLI flag, World Pack field)  QSR-4; the rule is a constant in this PR
the encoding of a Process's state as a JSON array of numbers  F-SAVE-2 (§4.4), kernel lane; recorded here
                                                               only
the cost of encoding a WorldSnapshot every 64 revisions        measured (§4.5), not changed
cognition stores, client caches                               not world saves (ARC-59)
a Postgres backend                                            ARCHITECTURE §8 "later"; the trait change
                                                               here keeps it implementable
```

### 2.3 Invariants this PR must hold

- **I-SR-1 The log is whole.** No row of `facts` or `journal` is deleted, updated or re-encoded by any
  path this PR adds. Their bytes in a format-3 save are exactly the bytes a format-2 build would have
  written for the same run.
- **I-SR-2 Resume never loses its snapshot.** The snapshot `latest_snapshot(head)` would return is never
  deleted; every pruning happens in the same transaction as the commit of a newer scheduled snapshot.
- **I-SR-3 Retention is a function, not a history.** Which snapshots a save holds after committing a
  scheduled snapshot at revision *n* depends only on *n*, the snapshot interval and the set of stored
  revisions — never on wall time, process lifetime or how many restarts happened. A killed-and-resumed
  headless run therefore holds the same rows as an uninterrupted one (`ARC-27`).
- **I-SR-4 Byte comparison is of the canonical encoding.** Replay and `verify` compare the JSON bytes of
  `WorldSnapshot`, `JournalEntry` and `EventEnvelope`, as today. A snapshot's compressed bytes are a
  storage representation; equality of world state is never argued from them.
- **I-SR-5 Kernel and systems unchanged.** No file under `kernel/`, `contracts/`, `systems/` or
  `packages/` changes (change-amplification test, `CLAUDE.md` §4.5).

## 3. Audit anchors (`origin/main @ bb62edf`)

| Where | What it says today | Consequence for this design |
| --- | --- | --- |
| `persistence/src/world.rs:13` | `DEFAULT_SNAPSHOT_INTERVAL: u64 = 64` | the lattice the anchors sit on |
| `persistence/src/world.rs` `commit` | a snapshot when `revision.is_multiple_of(snapshot_interval)`, encoded with `encode` and committed in the revision's row | pruning attaches here (D-SR-7) |
| `persistence/src/world.rs` `checkpoint` | an off-lattice snapshot at the head, written at a clean shutdown (`server/src/runtime.rs:200`) when the clock has not idled | off-lattice rows exist only for hosted worlds; D-SR-3 removes them at the next scheduled snapshot |
| `persistence/src/world.rs` `resume` | `latest_snapshot(head)`, then `decode::<WorldSnapshot>` | the one restore decode site (D-SR-4) |
| `persistence/src/replay.rs` `verify` | for every revision, if `snapshot_at(revision)` exists, `encode(world.snapshot()) != stored` → `SnapshotDisagreesWithHistory` | becomes a comparison of decoded bytes (D-SR-6) |
| `persistence/src/sqlite.rs` | four tables; `snapshots(revision PK REFERENCES journal, snapshot BLOB)`; WAL; `auto_vacuum` never set (0, NONE); page size 4 096 | free pages left by a `DELETE` are reused by the next insert; no `VACUUM` needed in steady state (§5.4) |
| `persistence/src/sqlite.rs` `checkpoint` | `INSERT OR IGNORE` | unchanged |
| `persistence/src/backend.rs` | `RevisionRow { revision, at, action_id, entry, facts, snapshot }`; the trait stores encoded rows only | gains one typed field, `retire` (D-SR-7) |
| `persistence/src/format.rs` | `SAVE_FORMAT = 2`; `check_format` refuses older and newer | 3 (D-SR-8) |
| `persistence/tests/save.rs:467–510` | tamper test writes `'{}'` into a snapshot row | rewritten for compressed rows (C2) |
| `persistence/tests/kill_and_resume.rs` | compares facts, journal and snapshot rows of a killed survivor and a control by raw SQL; asserts the restored snapshot is genesis or a multiple of the interval | still true; extended with kill points at an anchor and at a pruning commit (C5) |
| `tests/acceptance/tests/arrival_resolvers_resume.rs`, `configuration_seam.rs` | compare snapshot rows of survivor and control through the backend | unchanged semantics; stored bytes must still be equal (I-SR-3) |
| `tools/cli/tests/headless/mod.rs:162–224` | `AC-12`: byte equality of facts, journal and snapshots of two runs | unchanged; holds by I-SR-3 and D-SR-5 |
| `tools/cli/tests/market/mod.rs:720–756` | decodes the newest snapshot with `format::decode(…, "snapshot")` | switches to the snapshot decoder (C2) |
| `scripts/ci_parity.py:59, 557` | `AC-8` reads `facts`, `journal`, `snapshots` with Python `sqlite3` and hashes stored bytes | unchanged; the codec must be byte-deterministic across the three platforms (D-SR-5, ASR-7) |
| `tools/cli/src/main.rs` `Run`, `Serve`, `Replay`, `Inspect` | `run --save`, `serve --save`, `replay --save`, `inspect` | only `inspect` gains a line (D-SR-9) |
| `Cargo.toml` (workspace) | `rusqlite = { version = "0.40.2", features = ["bundled"] }`: a C compiler is already required on every platform | `zstd-sys` adds C code, not a toolchain |

## 4. Measurements (planning, scratch; not evidence for any acceptance criterion)

All on the operator's macOS 26.2 arm64 laptop, the repository's `dev` profile (opt-level 1, `ARC-30`),
`origin/main @ bb62edf`, `mineworld run worlds/market-town --headless --seed 7 --days N --save DIR`.
Table sizes are SQLite's `dbstat` page totals; row sizes are `length()` of the stored blob.

### 4.1 Current saves

| | 30 days | 300 days |
| --- | ---: | ---: |
| `world.sqlite` | 373 882 880 B (356.6 MiB) | 3 795 013 632 B (3 619 MiB) |
| `snapshots` table | 297.9 MiB (83.6 %) | 3 036.1 MiB (83.9 %) |
| `facts` table | 39.3 MiB | 389.0 MiB |
| `journal` table | 18.9 MiB | 189.3 MiB |
| `facts_by_revision` index | 0.4 MiB | 4.9 MiB |
| head revision | 29 193 | 290 995 |
| snapshot rows | 457 | 4 547 |
| snapshot row size | mean 682 KB; 311 KB at genesis → 704 KB at day 30 | max 736 KB |
| fact rows | 38 286, mean 926 B | 375 619 |
| journal rows | 29 193, mean 601 B (genesis entry 298 KB, holding the pre-genesis `WorldSnapshot`) | 290 995 |
| wall time | 8.9 s | 178.1 s (the same 300 days with no save: 54.7 s) |

Revisions accrue at ≈ 973 per simulated day in Market Town; the fact fingerprint of the saved and the
unsaved 300-day runs is the same (`27693f9e0c72bc9f`, for reading only). The 266 MB reported for
`F-SAVE-1` on 2026-10-09 predates later growth of the per-snapshot state (weather, walking).

### 4.2 What a snapshot is made of (day 30, 703 730 B)

```text
components   433 375 B   conversation-history 290 866 (12 people), acquaintances 93 102, routine 12 082, …
time         247 630 B   processes 246 599 (19 processes; one Process's state is a byte vector)
relations      7 501 B
entities       9 189 B
composition    5 934 B
```

At day 300 the same sections measure conversation-history 288 242 B and processes 246 615 B: the state
has **plateaued**; snapshot size is bounded and the growth is purely the count of snapshots.

### 4.3 Compression and deltas of one snapshot (day 30, 703 730 B)

| Codec | Bytes | Ratio | Speed (compress, this laptop) |
| --- | ---: | ---: | --- |
| zstd 1 | 53 725 | 13.1× | ≈ 440 MB/s |
| **zstd 3** | **52 045** | **13.5×** | **≈ 600 MB/s (≈ 1.2 ms); decompress ≈ 1 GB/s** |
| zstd 9 | 41 550 | 16.9× | ≈ 120 MB/s |
| zstd 19 | 33 021 | 21.3× | slow (seconds per 50 snapshots) |
| gzip/deflate 6 | 45 672 | 15.4× | ≈ 16 ms per snapshot |
| lz4 (default) | 96 989 | 7.3× | fastest |
| zstd 3 `--patch-from` the previous scheduled snapshot (64 revisions earlier) | 1 708 | 412× | — |
| zstd 3 `--patch-from` a snapshot 14 592 revisions earlier | 11 466 | 61× | — |

### 4.4 The log's own compressibility (30-day save)

| Representation | facts | journal |
| --- | ---: | ---: |
| raw, every 10th row (sample) | 3 509 978 B | 1 731 251 B |
| per-row zstd 3, no dictionary | 1 392 488 B (2.5×) | 837 602 B (2.1×) |
| per-row zstd 3, 64 KB dictionary trained on the sample | 315 447 B (11.1×) | 184 131 B (9.4×) |
| one block of revisions 8 193–12 288, zstd 3 | 4 865 643 → 211 964 B (23×) | 2 381 052 → 118 068 B (20×) |
| same block, zstd 9 | 127 416 B (38×) | 83 797 B (28×) |

**Companion finding `F-SAVE-2` (kernel lane; not this PR).** A `Process`'s `state` is serialized as a
JSON array of decimal numbers, one per byte (`"state":[123,34,99,…]`). The weather process's 66 KB packed
series (step-19 SD-TW-d-5) therefore costs ≈ 245 KB in every snapshot, ≈ 3.7× its size. Base64 or
`serde_bytes` would cut each snapshot by about a third. It is a change to a kernel type's encoding and a
save-format change, so it is routed, not done here. After D-SR-4 its cost per stored snapshot is small.

### 4.5 Pruning simulated on a copy of the 30-day save

Deleting every snapshot except genesis, multiples of 4 096 and the newest two scheduled ones left 10
rows; `VACUUM` (0.26 s) shrank the file from 373 882 880 B to 68 218 880 B (65.1 MiB), with `snapshots`
at 6.3 MiB uncompressed. Writing the snapshots is also the main I/O cost of a saved run: a 300-day run
writes ≈ 3.0 GiB of snapshot pages through the WAL, and pruning alone does not reduce that write volume;
compression does (to ≈ 226 MiB).

## 5. Reuse comparison (`REUSE_POLICY.md` §§1–4, 11–12)

Sizes are for the 300-day Market Town save, total file, computed from §4 unless marked measured.
"Log" = facts + journal + index = 583 MiB, kept in every option but 5.7.

| # | Option | 300-day size | Replay / resume | Cost per save | Complexity | Platforms | New dependency |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| 5.1 | **Keep genesis + anchors every 4 096 + newest 2**, uncompressed | ≈ 636 MiB (log + 74 × 0.72 MB) | unchanged: resume from newest; verify from genesis; anchors verifiable | one `DELETE` per 64 revisions | low: one pure rule, one trait field | SQLite only | none |
| 5.2 | zstd-compressed snapshot rows, no pruning | ≈ 809 MiB, still +2.7 MiB/day | unchanged (decode then compare JSON) | ≈ 1.2 ms compress per snapshot; 14× fewer bytes written | low | bundled C, already needed | `zstd` (MIT; MIT/Apache-2.0 `-safe`, `-sys`; libzstd BSD-3 bundled) |
| **5.1 + 5.2** | **Recommended** | **≈ 587 MiB** (log + 74 × 52 KB) | unchanged | both of the above | low | as above | `zstd` |
| 5.3 | Delta snapshots (`--patch-from` the previous, periodic bases) | ≈ 590 MiB with no pruning | restore needs a base and a chain; one damaged delta damages every later snapshot until the next base | patch per snapshot, base management | high: chains, base policy, chain-aware verify | as 5.2 | `zstd` (`patch-from`/ref-prefix API, `experimental` feature) |
| 5.4 | SQLite `VACUUM` / `auto_vacuum = INCREMENTAL` / `FULL` | no effect alone (nothing is deleted) | unchanged | `VACUUM` rewrites the whole file | trivial | SQLite | none |
| 5.5 | Page-level compression: SQLite ZIPVFS; `sqlite-zstd` extension | ≈ 10× on everything | transparent to rows | background maintenance | medium | ZIPVFS: proprietary licence; `sqlite-zstd`: LGPL-3.0 per its upstream README, outside `ARC-55`; its dictionary chunking is asynchronous | rejected on licence and determinism |
| 5.6 | Content-defined dedup of snapshot sections (hash-addressed components) | ≈ 590 MiB with no pruning (similar to 5.3) | restore reassembles from a content table | hashing per component per snapshot | high: a second storage model inside `format.rs` | — | none or `blake3` |
| 5.7 | Journal compaction into a checkpoint: drop journal (and facts) before an anchor | ≈ 40 MiB + recent log | **breaks `ARC-25`**: genesis no longer re-run, `verify` from genesis impossible, facts are `INV-11` history | — | — | — | rejected |
| 5.8 | Log compression, SR-b: seal journal and facts of each 4 096-revision span into zstd blocks at each anchor | ≈ 40 MiB with 5.1+5.2 | replay decodes blocks; byte comparison on decoded rows unchanged | ≈ 10 ms per anchor | medium: block table, readers, `ci_parity` | as 5.2 | `zstd` |

**Per-option notes.**

- **5.1 (adopted).** Snapshots are checkpoints; deleting one loses no information the journal cannot
  regenerate. Anchors bound the cost of re-checking a stretch of history (`verify_from`, D-SR-6) and of
  diagnosing a divergence. Spacing 4 096 = 64 scheduled snapshots ≈ 4.2 Market Town days; 300 days hold
  71 anchors. Uses SQLite's own free-page reuse; no new mechanism.
- **5.2 (adopted).** Plateaued 700 KB JSON is 13.5× smaller at zstd 3 with ≈ 1 ms cost. zstd 3 is chosen
  over 9 (+25 % ratio, 5× slower) because a snapshot is encoded on the world thread every 64 revisions
  of a hosted world. deflate 6 (pure-Rust `miniz_oxide` possible) compresses 12 % better here but is
  ≈ 13× slower; lz4 (`lz4_flex`, pure Rust) is 1.9× larger. `ruzstd` (MIT, pure Rust) has an encoder
  whose own documentation says it does not yet reach the reference library's speed or ratio; it is the
  named fallback if the C build ever becomes a problem (re-evaluation trigger in `DEP-43`).
- **5.3, 5.6 (declined).** After 5.1 the snapshots of a 300-day save total ≈ 3.8 MiB; deltas or dedup
  would save ≈ 3 MiB of a 587 MiB file at the price of chained restore. Revisit only if anchors become
  dense (e.g. a "rewind to any hour" feature).
- **5.4 (used only as SQLite already behaves).** With pruning in the writing transaction, freed pages
  are reused by the next snapshot's insert; the file plateaus at the high-water mark plus at most one
  snapshot's worth of free pages. No `auto_vacuum` change (it requires a rebuild of existing files and
  would add page-move work to every commit). `VACUUM` is not run automatically.
- **5.5 (rejected).** Licence (ZIPVFS proprietary; `sqlite-zstd` LGPL-3.0) and, for `sqlite-zstd`,
  storage layout that changes outside our transaction.
- **5.7 (rejected).** It is exactly what `ARC-25` forbids; it would need the operator to reverse a
  frozen architecture decision, and nothing measured here justifies that.
- **5.8 (designed, deferred to SR-b).** The only remaining large lever (§6.9). Deferred because it moves
  where the authority's bytes live, which deserves its own ruling (QSR-1).

## 6. Design

### 6.1 Decisions (D-SR-1 … D-SR-9)

| ID | Decision |
| --- | --- |
| D-SR-1 | Only snapshot rows are ever deleted. `facts` and `journal` are untouched (I-SR-1). |
| D-SR-2 | Scheduled snapshots stay every `interval` revisions (default 64). An **anchor** is a scheduled snapshot whose revision is a multiple of `interval × 64` (default 4 096). Genesis (revision 1) is always kept. |
| D-SR-3 | **The retention rule.** When the scheduled snapshot at revision *n* is committed, the retained set is `K(n) = {1} ∪ {a ≤ n : a mod (64·interval) = 0} ∪ {n, n − interval}`; every stored snapshot revision *s < n* with *s ∉ K(n)* is deleted in the same transaction. Off-lattice checkpoints (a clean shutdown's) are kept until the next scheduled snapshot and then fall under the rule. |
| D-SR-4 | **The snapshot codec.** A stored snapshot is `zstd(level 3, checksum on, content size in header, single-threaded)` of the exact bytes `encode(&WorldSnapshot)` produces today. The codec lives in `format.rs` as `encode_snapshot` / `decode_snapshot`; the backend stays opaque. Journal entries and facts are not compressed in this PR. |
| D-SR-5 | **Determinism of the codec.** `zstd` with default features off, bundled libzstd (never `pkg-config`), pinned in `Cargo.lock`; one compression level, no dictionary, no multithreading. Stored snapshot bytes are then a pure function of the JSON bytes on every platform, which `AC-8` checks rather than assumes (ASR-7). |
| D-SR-6 | **Comparison is of decoded bytes.** `verify` decodes each stored snapshot and compares its JSON bytes with `encode(world.snapshot())`. New `verify_from(backend, composed, anchor)` restores the retained snapshot at `anchor`, re-executes to the head with the same byte checks as resume, and compares every later retained snapshot. |
| D-SR-7 | **Where pruning runs.** `PersistentWorld` keeps the set of stored snapshot revisions (read once at `create`/`resume` via `snapshot_revisions`, ≤ ~80 entries), computes `retire` from D-SR-3 and passes it in `RevisionRow::retire: Vec<WorldRevision>`; `SqliteBackend::write_revision` deletes those rows inside the revision's transaction, after inserting the new snapshot. The policy is typed and testable in Rust; the backend executes a list. |
| D-SR-8 | `SAVE_FORMAT` becomes 3. A format-2 save is refused with `SaveFormatOutdated { saved: 2, supported: 3 }`, as `ARC-25` requires; no migration in this PR (QSR-3). |
| D-SR-9 | `mineworld inspect` prints the retained snapshot revisions and their stored sizes on one line, so the effect is visible without SQL (`ARC-23`). |

### 6.2 Why the rule is a function of the stored set (I-SR-3)

A headless run commits scheduled snapshots only; its stored set after any commit at *n* is exactly
`K(n)`, whether or not the process was killed and resumed in between, because each pruning is atomic
with the snapshot that triggers it and the rule reads nothing else. A kill between two scheduled
snapshots leaves the set at `K(n_prev)`, which is also what the uninterrupted run held at that head.
Hosted worlds can hold one extra off-lattice checkpoint between scheduled snapshots; `AC-12` does not
cover hosted worlds (`ARC-27`), and the next scheduled snapshot restores `K(n)`.

### 6.3 Why "newest two"

Resume needs one. The second costs ≈ 52 KB and keeps a near fallback for diagnosis and for
`verify_from` when the newest stored snapshot is the one under suspicion; it also makes the off-lattice
interplay simple (`n − interval` is retained, so a checkpoint just below `n` is never the only recent
snapshot). QSR-2 asks whether one is enough.

### 6.4 Custom intervals and changed intervals

`snapshot_every(i)` (tests use 8 and others) keeps working: anchors are at multiples of `64·i`. A save
resumed under a different interval simply has its old off-lattice rows retired at the next scheduled
snapshot unless they happen to be anchors of the new lattice. Nothing assumes the interval is 64.

### 6.5 The trait change

```rust
pub struct RevisionRow {
    // … unchanged fields …
    /// Snapshot revisions to delete in this revision's transaction, after `snapshot` is written.
    /// Never the revision itself; never a revision above it.
    pub retire: Vec<WorldRevision>,
}
```

`SqliteBackend` executes `DELETE FROM snapshots WHERE revision = ?1` per entry (≤ 2 rows per commit in
steady state). A backend that does not support deletion would be a defect, not an option: no
`Option`-shaped capability is introduced (`CLAUDE.md` §4.11).

### 6.6 Error behaviour

| Case | Behaviour |
| --- | --- |
| a stored snapshot is not a valid zstd frame, or its checksum fails | `PersistError::Damaged { detail: "the snapshot at rN does not decompress: …" }`; resume refuses; never a panic |
| it decompresses to JSON that does not decode | `Damaged` naming "snapshot" (existing `decode`) |
| it decodes but disagrees with history | `SnapshotDisagreesWithHistory { revision }` (existing) |
| `verify_from` given a revision that holds no snapshot | `PersistError::NoSnapshotAt { revision }` (new variant), naming the retained revisions |
| a `retire` entry ≥ the committed revision | refused by `PersistentWorld` before the transaction (a debug assertion and an error); never reaches SQL |
| the delete fails | the whole revision's transaction fails; the world is `WorldAheadOfSave`, as for any failed commit |

### 6.7 Spec edits (C1)

- `docs/DECISIONS.md`: `ARC-81` (D-SR-1 … D-SR-9, the §5 comparison summary, revisit triggers); `DEP-43`
  (`zstd`, with `lz4_flex`, `miniz_oxide`/`flate2`, `ruzstd`, `sqlite-zstd`, ZIPVFS declined, and the
  re-evaluation trigger "a platform build of `zstd-sys` fails, or `AC-8` parity differs in the
  `snapshots` table"); a dated note under `ARC-25` pointing its "pruning is later work" limitation at
  `ARC-81`.
- `docs/ARCHITECTURE.md` persistence section: snapshots are retained by rule; the log is kept whole.
- `persistence/README.md`: one line on retention and compression; `persistence/src/lib.rs` and
  `sqlite.rs` module docs.
- `step-06-persistence.md` §9 L-6: note that snapshot pruning landed in SR.

### 6.8 Platforms

`zstd-sys` compiles bundled C with the same `cc` path `libsqlite3-sys` already uses (MSVC on Windows,
clang on macOS, gcc in the Linux container). Every target is 64-bit little-endian. Determinism across
them is asserted by `AC-8`'s existing three-platform comparison of the 30-day save's stored bytes,
which after this PR includes compressed snapshot rows. File deletion semantics are SQLite's, identical
on all three.

### 6.9 SR-b (designed here, built only after QSR-1)

At each anchor commit *a*, the journal rows and fact rows of revisions `(a − 4096·k, a − 4096·(k−1)]`
that are older than the newest anchor are moved, in the same transaction, into one table
`sealed(first_revision, last_revision, journal_block, facts_block)` whose blocks are zstd 3 frames of
the rows' unchanged bytes with their keys; the live tables keep only the unsealed tail. Readers
(`journal_after`, `facts_of`, `last_facts`, `highest_action_id`, biography/history/perceived) read
sealed blocks then the tail; replay compares decoded row bytes exactly as today. Measured ratio 20–23×
(§4.4) → a 300-day save of ≈ 40 MiB. It changes `ci_parity.py`'s table list and every raw-SQL reader
in tests, and it means a fact's bytes are no longer individually addressable by SQL — the reason it is
a separate ruling.

## 7. Test ownership

| Test | File | Owner |
| --- | --- | --- |
| retention rule over intervals, heads and stored sets (table-driven, including off-lattice) | `persistence/src/world.rs` unit tests or `persistence/tests/retention.rs` | this PR |
| codec round-trip and damaged frame | `persistence/tests/save.rs` | this PR (rewrites the tamper test) |
| resume after pruning; off-lattice checkpoint then restart | `persistence/tests/save.rs` | this PR |
| kill-and-resume across an anchor and a pruning commit | `persistence/tests/kill_and_resume.rs` | this PR extends |
| `verify_from` from every retained anchor | `persistence/tests/save.rs` and the 300-day measurement | this PR |
| `AC-12` byte equality | `tools/cli/tests/headless/mod.rs` | existing, must stay green unchanged |
| `AC-8` parity | `scripts/ci_parity.py` in CI | existing, must stay green unchanged |
| size criterion ASR-2 (30 days) | `tools/cli/tests/market_town.rs`: save C's `world.sqlite` (+ `-wal`) ≤ 64 MiB and its snapshot revisions exactly `K(n)`, asserted | this PR (CI assertion, QSR-5) |
| size criterion ASR-1 (300 days) | measured by `mineworld run … --save` and `mineworld inspect`; recorded in §14 | this PR (recorded measurement, QSR-5) |

## 8. Acceptance and adversarial criteria (fixed now, before anything is measured)

| ID | Criterion |
| --- | --- |
| ASR-1 | `mineworld run worlds/market-town --headless --seed 7 --days 300 --save D`: `D/world.sqlite` plus any `-wal` ≤ **640 MiB** after exit; `snapshots` table ≤ **8 MiB** (`dbstat`); stored snapshot revisions exactly `K(n)` for the final scheduled *n* (D-SR-3). |
| ASR-2 | Same at `--days 30`: file ≤ **64 MiB**; snapshot rows = 1 + ⌊n/4096⌋ + 2 (deduplicated) for the final *n*. Asserted in CI by `tools/cli/tests/market_town.rs` on save C (QSR-5). |
| ASR-3 | `mineworld replay worlds/market-town --save D` on the 300-day save passes: every revision re-executed from genesis, every fact and every retained snapshot reproduced byte for byte (decoded JSON). |
| ASR-4 | `verify_from` from **every** retained anchor of the 30-day save, and from at least 3 anchors of the 300-day save (first, middle, last), reaches the head with every fact, entry and later snapshot byte-identical. |
| ASR-5 | `kill_and_resume`: survivor and control are equal in journal, facts and snapshot rows (stored bytes) and in the set of snapshot revisions, for kills including the revision of an anchor commit and of a commit that retires rows. |
| ASR-6 | `AC-12` (`tools/cli/tests/headless`): two runs, same seed, stored bytes of all three tables equal — unchanged test, green. |
| ASR-7 | `AC-8`: `ci_parity.py compare` passes for macOS arm64, Linux x86_64 (container) and Windows x86_64 on the PR head, including the `snapshots` table chunks. |
| ASR-8 | A format-2 save (fixture made by a build of the base commit) is refused by `run`, `serve`, `replay` and `inspect` with the outdated-format message naming 2 and 3. |
| ASR-9 | A snapshot row replaced by (a) bytes that are not a zstd frame → `Damaged` naming the revision; (b) a valid frame of different JSON → `SnapshotDisagreesWithHistory` from `verify`. Resume never panics. |
| ASR-10 | Resume never loses its snapshot: for every head in a 1 … 3·4096+5 sweep with interval 8, after each commit `latest_snapshot(head)` exists and is ≥ head − 8 (or 1); a clean-shutdown checkpoint followed by restart resumes with 0 rows replayed. |
| ASR-11 | The log is whole: the 300-day save's `facts` and `journal` row counts are 375 619 and 290 995 (as at the base), and their SHA-256 over stored bytes equals the base commit's for the 30-day save. |
| ASR-12 | No file under `kernel/`, `contracts/`, `systems/`, `packages/` changes (`git diff --stat`). |

**Adversarial (each must make a named criterion fail, run once and recorded):**

- retire nothing → ASR-1 fails;
- also retire `n − interval` and `n` → ASR-10 fails;
- make the rule read `SystemTime::now()` parity (e.g. retire odd anchors on odd seconds) → ASR-5 or
  ASR-6 fails;
- write snapshots at zstd level 9 instead of 3 → ASR-3 and ASR-4 still pass (they must: they compare
  decoded state, I-SR-4), and a save mixing both levels still resumes; this shows that a codec setting
  can never change a replay verdict;
- delete one fact row → ASR-11 and replay fail.

Measured, not gated: wall time of the 300-day saved run before and after (expected to fall from 178 s,
since ≈ 2.8 GiB fewer bytes are written).

## 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-SR-1 | zstd output differs across platforms or toolchains | bundled libzstd 1.5.x pinned by `Cargo.lock`, default features off, single-threaded; `AC-8` detects; fallback: `ci_parity` hashes decoded snapshot JSON (a reviewed change to `ARC-49`) |
| R-SR-2 | A future zstd upgrade changes stored bytes for new snapshots | it changes no decoded state and no replay result; `AC-12` compares two runs of one build, so it is unaffected; noted in `DEP-43` |
| R-SR-3 | A bug in the rule deletes the resume snapshot | I-SR-2 enforced by construction (never retire ≥ n − interval) and by ASR-10's sweep |
| R-SR-4 | Players with format-2 saves lose them | no public release has shipped (S23 is in design); QSR-3 |
| R-SR-5 | The log alone still grows ≈ 1.9 MiB per simulated day | stated; SR-b (§6.9) is the remedy, approved as a separate PR after SR merges (QSR-1) |

## 10. Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its
evidence; `N/A` needs an audited reason. Commands assume the worktree root with `cargo` on `PATH`.
Targeted validation per commit; the PR's CI is the one full run.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Start the implementation context against the frozen design. The freeze header, the
  contract's authority lines and the decision numbers (`ARC-81`, `DEP-43`) were filled by the planning
  session at freeze (§12, §13.1); C0 verifies them rather than writing them.
- **Scope.** This document (§14 opened); a new `handoff-sr.md`.
- [x] Implementation: verify the `DESIGN FROZEN` header, §12's authority lines and their sources, and
  that `overall.md`'s table lists `ARC-81` and `DEP-43` for SR; record the implementation base commit;
  initialize the handoff with the contract's required fields. *Evidence: §14.1 (base a5f5357; the
  table does not yet list SR — F-SR-0, a request to the primary session); `handoff-sr.md`.*
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  *Evidence: both exit 0 at C0 (§14.1).*
- [x] Review: every authority line has a source; no scope widened. *Evidence: §12 re-read; C0 edits
  only this ledger and the new handoff.*
- **Commit boundary.** Documentation only.

### C1 — Decisions, spec edits, the dependency

- **Goal.** The decisions exist before the code (`CLAUDE.md` §2.2); the dependency passes the licence
  gate on every platform before anything uses it.
- **Scope.** `docs/DECISIONS.md` (`ARC-81`, `DEP-43`, note under `ARC-25`); `docs/ARCHITECTURE.md`
  persistence section; `persistence/README.md`; `step-06-persistence.md` L-6 note; workspace
  `Cargo.toml` `zstd = { version = "0.13", default-features = false }`; `persistence/Cargo.toml`.
- [x] Implementation: the files above. *Evidence: §14.2.*
- [x] Validation: `cargo deny check`; `cargo check --workspace`; both doc checks; push and confirm the
  three-platform build is green before C2 depends on it. *Evidence: §14.2 (deny, check, doc checks
  local; the three-platform build is the draft PR's first CI run, recorded there).*
- [x] Review: each decision names alternatives and revisit triggers; `ARC-25`'s wording is extended,
  not relaxed. *Evidence: §14.2.*
- **Failure cases.** `zstd-sys` fails on a CI leg: stop, record, propose `ruzstd` (material: a
  dependency change returns to the operator).

### C2 — The snapshot codec and format 3

- **Goal.** Every stored snapshot is a zstd frame; every comparison is of decoded JSON.
- **Scope.** `persistence/src/format.rs` (`SAVE_FORMAT = 3`, history line, `encode_snapshot`,
  `decode_snapshot`); `world.rs` (`create`, `commit`, `checkpoint`, `resume`); `replay.rs` (`verify`);
  `error.rs` if a variant is needed; `persistence/tests/save.rs` (tamper test for ASR-9; format-2
  refusal ASR-8 with a fixture produced by the base build and committed under `persistence/tests/`);
  `tools/cli/tests/market/mod.rs` (decoder).
- [x] Implementation. *Evidence: §14.3.*
- [x] Validation: `cargo test -p mineworld-persistence`; `cargo test -p mineworld-cli --test market`;
  `cargo test -p mineworld-cli --test headless` (AC-12); clippy `-D warnings`; fmt. *Evidence: §14.3
  (`market` and `headless` are modules, not test targets: their users `run`, `inspect` ran here and
  `market_town` runs at C5).*
- [x] Review: no call site decodes a snapshot with the plain `decode`; journal and facts encoding
  unchanged (diff shows no change to their paths). *Evidence: §14.3.*

### C3 — The retention rule

- **Goal.** D-SR-2, D-SR-3, D-SR-7: snapshots retired atomically by a pure rule.
- **Scope.** `persistence/src/world.rs` (the rule as a private pure function; stored-set tracking;
  `retire` computation), `backend.rs` (`RevisionRow::retire`), `sqlite.rs` (`write_revision` deletes);
  the test double in `persistence/tests/save.rs`; new `persistence/tests/retention.rs` (ASR-10 sweep,
  off-lattice checkpoint then restart, changed interval).
- [ ] Implementation.
- [ ] Validation: `cargo test -p mineworld-persistence`; `cargo test -p mineworld-acceptance` (the two
  resume comparisons); headless AC-12; the adversarial mutations "retire nothing" and "retire n"
  run once each and recorded.
- [ ] Review: the rule reads nothing but *n*, the interval and the stored set; deletion only in the
  revision's transaction; `retire` never contains ≥ n − interval.

### C4 — `verify_from` and `inspect`

- **Goal.** D-SR-6, D-SR-9.
- **Scope.** `persistence/src/replay.rs` (`verify_from`), `lib.rs` export, `error.rs`
  (`NoSnapshotAt`); `tools/cli/src/inspect.rs` (one line); tests for ASR-4 (30-day scale via a fixture
  world) and the error case.
- [ ] Implementation.
- [ ] Validation: `cargo test -p mineworld-persistence`; `cargo test -p mineworld-cli`.
- [ ] Review: `verify_from` shares `replay_after`; no second comparison path.

### C5 — Kill-and-resume and the real runs (integration checkpoint)

- **Goal.** ASR-1 … ASR-7, ASR-11 on the real binary.
- **Scope.** `persistence/tests/kill_and_resume.rs` (kill points at an anchor and at a retiring commit);
  `tools/cli/tests/market_town.rs` (QSR-5: save C's file ≤ 64 MiB and snapshot revisions exactly
  `K(n)`, asserted after its activity checks, with the measured size in the failure message); §14
  evidence.
- [ ] Implementation: the kill points; the ASR-2 assertion.
- [ ] Validation: `cargo test -p mineworld-persistence --test kill_and_resume`;
  `cargo test -p mineworld-cli --test market_town` (the assertion passes; mutated once to 1 MiB to see
  it fail, recorded); release-profile runs of
  30 and 300 days with `--save`; `mineworld inspect`; `sqlite3 … dbstat` sizes; `mineworld replay` on
  the 300-day save; `verify_from` on three anchors; SHA-256 of facts/journal vs the base build
  (ASR-11); wall times; the PR's CI (`AC-8` three legs).
- [ ] Review: every number in §14 comes from a command recorded beside it.

### C6 — Close-out

- **Goal.** READY FOR OPERATOR REVIEW.
- **Scope.** §14 final; handoff; `overall.md` F-SAVE-1 entry marked "SR in review" (primary session's
  file — a request, §11).
- [ ] Implementation / [ ] Validation (exact-head CI) / [ ] Review.

## 11. Requirements this PR places on other documents and lanes

- **R-overall.** The primary session records `ARC-81`/`DEP-43` (assigned, QSR-6) in `overall.md`'s
  table, and records `F-SAVE-2` as a kernel-lane finding (ruled 2026-10-10); after merge, `F-SAVE-1` is
  marked resolved for snapshots, with SR-b (QSR-1: approved as a separate PR after SR merges) open.
- **R-S23.** SR lands before S23's first release (QSR-3), so no released save is format 2.
  `step-23-release.md` R-R5 can cite the bounded size (≈ 2 MiB per simulated day) once SR merges.
- **R-SR-b.** The SR-b design (§6.9) is written as its own PR document after SR merges, in a fresh
  planning context; it inherits D-SR-4/D-SR-5's codec and `DEP-43`.
- **R-kernel (`F-SAVE-2`).** Process state encoded as a JSON number array; routed to the kernel lane.
- **R-ARC-49.** None unless R-SR-1 materializes.

## 12. Execution contract (filled at freeze, 2026-10-10)

Source of every line marked "primary session": its freeze rulings of 2026-10-10, relayed by the
coordinator to the persistence-lane design session ("FROZEN … merge only after primary review", with the
worktree and branch named). Endpoints the rulings do not name individually (push, PR, CI repair) take
the working rules' shipped defaults (§21). Lines marked "mvp0 convention" follow the contracts of
P4 and P5a (`pr-s10-p4-memory.md` §12) and were not separately ruled.

```text
PROJECT / PR:            MineWorld mvp0, persistence PR SR — snapshot retention and compression
                         (F-SAVE-1)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s6-save-retention.md
RELATED / BINDING DOCS:  step-06-persistence.md; docs/DECISIONS.md ARC-25, ARC-27, ARC-49, ARC-55, DEP-2,
                         DEP-5; docs/ARCHITECTURE.md (persistence); docs/ENGINEERING_STANDARDS.md;
                         docs/REUSE_POLICY.md; overall.md (F-SAVE-1; decision table); CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-save-retention, its own, held by one
                         session (CLAUDE.md §3.1)                               (primary session)
BRANCH:                  mvp0/pr-s6-save-retention, created from main           (primary session)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records it
APPROVED SCOPE:          §2.1, as frozen (including QSR-5's 30-day CI assertion)
FROZEN INVARIANTS:       §2.3; D-SR-1 … D-SR-9; QSR-2's constants (anchor every 64 scheduled snapshots =
                         4 096 revisions at the default interval; newest two kept); QSR-3 (format 2
                         refused); QSR-4 (no player or pack configuration); every platform
SEQUENCE:                C0 … C6
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *;
                         sqlite3 (read-only queries on scratch saves); mkdir -p; sed -n   (mvp0 convention)
NEVER:                   delete, update or re-encode a facts or journal row; VACUUM or otherwise rewrite a
                         user's save; python3 -c; sed -i; curl; heredoc writes; leave a scratch save
                         outside the test scratch helper (DEP-29) or target/; add a dependency other than
                         zstd                                                    (mvp0 convention)
MATERIAL STOPS:          any kernel/contracts/systems/packages change; a dependency other than zstd, or
                         zstd failing to build on a CI leg; a change to D-SR-3's constants or ASR-1/ASR-2's
                         bounds after either has been measured once; ASR-7 failing on any leg; any part of
                         SR-b (QSR-1: a separate PR)
PLATFORMS:               macOS, Linux, Windows (ASR-7)
VALIDATION BUDGET:       unit, static and local integration, including 30- and 300-day release runs:
                         unrestricted. CI: the PR's runs; no manual dispatch    (mvp0 convention)
LIVE DOCUMENTATION:      this document (§14)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-sr.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized, in a fresh session
                                                                 (primary session, 2026-10-10)
                         semantic commits: authorized            (primary session; working rules §14)
                         branch push: authorized                 (primary session; working rules §21)
                         PR creation / update: authorized        (primary session; working rules §21)
                         CI repair to review readiness: authorized
                                                                 (primary session; working rules §21)
                         merge: never by the implementation session. Only after the primary session's
                         review, and only with explicit operator authorization (working rules §22)
POST-MERGE SYNC OWNER:   the primary session owns overall.md (F-SAVE-1, F-SAVE-2, the decision table);
                         the implementation session owns this document's ledger, evidence, deviations
                         and remaining issues, and the step-06 L-6 note       (mvp0 convention)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 13. Questions

**[operator]** marks a question only the operator can answer; **[primary]** one the primary session
decides.

### 13.1 Rulings, 2026-10-10

Relayed by the coordinator to the persistence-lane design session on 2026-10-10.

| ID | Ruled by | Ruling | Where it lands |
| --- | --- | --- | --- |
| QSR-1 | operator | **Yes.** SR-b (log compression, §6.9) is built as a separate PR after SR merges. | §2.2; §11 R-SR-b; §12 MATERIAL STOPS (no SR-b work in SR) |
| QSR-2 | primary | **Accepted.** Anchor every 4 096 revisions (64 scheduled snapshots at the default interval); keep the newest two. | D-SR-2, D-SR-3; §12 FROZEN INVARIANTS |
| QSR-3 | operator | **Refuse** format-2 saves by name; SR lands before S23's first release. | D-SR-8; ASR-8; §11 R-S23 |
| QSR-4 | operator | **No** player or World Pack configuration of retention yet. | §2.2; §12 FROZEN INVARIANTS |
| QSR-5 | primary | **Yes** for 30 days: the size bound is a CI assertion. 300 days stays a recorded measurement. | §2.1; §7; ASR-2; C5 |
| QSR-6 | primary | **`ARC-81`** (retention and codec) and **`DEP-43`** (`zstd`). | header; C1; §11 R-overall |
| — | primary | Record **`F-SAVE-2`** (Process state as a JSON number array) as a kernel-lane finding. | §4.4; §11 R-kernel, R-overall |

### 13.2 The questions as asked (kept for review)

| ID | Question | Recommendation |
| --- | --- | --- |
| QSR-1 [operator] | Build SR-b (§6.9: sealed, compressed blocks of journal and facts; 300-day save ≈ 40 MiB instead of ≈ 587 MiB)? It keeps every fact's bytes but no longer stores each as its own SQL row. | Yes, as a separate PR after SR merges; it is the only remaining large lever. |
| QSR-2 [primary] | Retention constants: anchor every 64 scheduled snapshots (4 096 revisions), newest two kept. | As drafted. |
| QSR-3 [operator] | Format-2 saves: refuse by name (policy) or provide a one-shot `mineworld save upgrade`? | Refuse — no release has shipped; land SR before S23's first release. |
| QSR-4 [operator] | Should retention be configurable by players or World Packs (e.g. keep more anchors)? | Not now; the constant is a function of revisions, and a setting can be added when a feature (rewind) needs it. |
| QSR-5 [primary] | Should ASR-1/ASR-2's size bounds become a CI check (a 30-day saved run's size in `ci_parity` or a scenario job), or stay a recorded measurement? | Add the 30-day bound (≤ 64 MiB) as an assertion in the existing saved-run CI path; 300 days stays a recorded measurement. |
| QSR-6 [primary] | Decision numbers. | Assign `ARC-81` and `DEP-43` (next free per `overall.md`, verified unused). |

## 14. Ledger (live during implementation)

Opened by C0 in the implementation session (2026-10-10), worktree
`/Users/yuema137/mineworld-worktrees/impl-save-retention`, branch `mvp0/pr-s6-save-retention`.

### 14.1 C0 — freeze verification

```text
Implementation base:   origin/main @ a5f535735e31ac14ff2413df014e82bf48fb6043 (#145 merged: this design
                       frozen). The audit anchors of §3 (bb62edf) were re-read at this base: every
                       file and line named there is unchanged in substance.
Freeze header:         present (revision 2, 2026-10-10), lifecycle FROZEN; §12 filled, every
                       ENDPOINT AUTHORITY line carries its source.
Decision numbers:      ARC-81 and DEP-43 are named by this document (QSR-6) and unused in
                       docs/DECISIONS.md at the base.
```

**Finding F-SR-0 (bounded; a request to the primary session, not an edit).** C0 asks to verify that
`overall.md`'s decision-number table lists `ARC-81` and `DEP-43` for SR. At the base it does not: the
table ends with S23's row and the sentence "The next free numbers are ARC-81 and DEP-43". The numbers
are assigned by the primary session's QSR-6 ruling (§13.1), which is this PR's authority for them, so
implementation proceeds; `overall.md` is the primary session's file (§12 POST-MERGE SYNC OWNER), so this
session does not edit it. Request R-overall (§11) stands: the primary session adds the row
`S6 SR | ARC-81 | DEP-43 (zstd)` before merge.

**Base measurements (for ASR-8 and ASR-11; release profile, `/tmp/impl-sr/base-mineworld` built from
a5f5357, macOS 26.2 arm64).**

```text
mineworld run worlds/market-town --headless --seed 7 --days 30  --save …/base-30    wall 5.17 s
  world.sqlite 373 882 880 B; journal 29 193 rows (head 29 193); facts 38 286; snapshots 457
mineworld run worlds/market-town --headless --seed 7 --days 300 --save …/base-300   wall 57.30 s
  world.sqlite 3 795 013 632 B; journal 290 995 rows; facts 375 619; snapshots 4 547
SHA-256 over the stored rows, computed identically for base and head:
  sqlite3 DB "SELECT event_id, revision, hex(fact) FROM facts ORDER BY event_id" | shasum -a 256
  sqlite3 DB "SELECT revision, at, action_id, hex(entry) FROM journal ORDER BY revision" | shasum -a 256
  base-30   facts   a1fc10a8a991e5b73a5b38f0c66e3425c39684b196fdac40296c14042f53ddfa
            journal ae11e5fccc856a91c360ae27218b7b0164f0c5710bc5e7d0fa01754c9008ae33
  base-300  facts   5b1dd246e35f45f81334e40c5b9c4a591f275c5b84c3a0710ed27832e86e9707
            journal 60d8e807360e3b8545a201a6ca3b4febdea296bd9a4fda92c59d3c077aaeebd2
```

The base counts equal §4.1's (measured at bb62edf), so ASR-11's numbers stand at this base.

**ASR-8 fixture.** `persistence/tests/fixtures/format-2/world.sqlite` (53 248 B, SHA-256
`346f61f8f1e6195dcc240d4fe9782e416e0ffa69e2b0aad79057c13991118344`) was written by the base build:
a scratch detached worktree at a5f5357 (`/tmp/impl-sr/base-tree`, never pushed) with one uncommitted
ignored test that creates a save of the persistence test world (`support::assembled`, interval 8,
12 scripted steps; head 20, snapshot rows at 1, 8, 16, uncompressed). It is `format 2` in its manifest
and every row is a format-2 build's own bytes.

*Deviation D-1 (bounded).* ASR-8 names `run`, `serve`, `replay` and `inspect`. Those commands open a
save of a real World Pack, and the smallest base-built pack save (social-cafe, one day) is 4.8 MB —
too large to commit. So the committed fixture proves a genuine format-2 save is refused by the library
(`PersistentWorld::resume`, `verify`, `verify_from`); the four commands are shown to route that refusal
by a CLI test that writes a social-cafe save with the head build and sets its manifest format to 2 with
SQL (the same technique `save.rs` already uses for formats 1 and 3). Both halves are recorded under C2.

### 14.2 C1 — decisions, spec edits, the dependency

```text
docs/DECISIONS.md            ARC-81 and DEP-43 appended; a dated note under ARC-25's limitations
docs/ARCHITECTURE.md §8      snapshots retained by rule, compressed; the log kept whole
persistence/README.md        one paragraph on retention and compression
step-06-persistence.md L-6   dated note: snapshot pruning landed in SR; the log still whole
Cargo.toml                   zstd = { version = "0.13", default-features = false } (workspace)
persistence/Cargo.toml       zstd = { workspace = true }
Cargo.lock                   + zstd 0.13.3, zstd-safe 7.3.0, zstd-sys 2.1.1+zstd.1.5.7,
                             jobserver 0.1.35, getrandom 0.4.3, r-efi 6.0.0 (cc's `parallel`)
```

Discovery (recorded in DEP-43): `zstd-safe` and `zstd-sys` declare BSD-3-Clause, not MIT/Apache-2.0 as
the design's §5 table said; both are on `ARC-55`'s list. `zstd-sys` turns on `cc`'s `parallel`
feature, which brings three build-time crates (above), all admitted. `zstd-sys`'s build switches to a
system libzstd only with its `pkg-config` feature or the `ZSTD_SYS_USE_PKG_CONFIG` variable; neither is
set (`cargo tree -e features -i zstd-sys` shows only `std`).

```text
cargo deny check licenses sources bans     bans ok, licenses ok, sources ok           PASS
cargo check --workspace --all-targets      Finished, no warnings                      PASS
python3 scripts/check_doc_headings.py      193 numbered sections … none duplicated    PASS
python3 scripts/check_decision_ids.py      106 decision ids, all distinct             PASS
```

Review: ARC-81 lists options 5.1–5.8 with reasons and revisit triggers; DEP-43 lists zstd levels,
deflate, lz4, ruzstd, sqlite-zstd and ZIPVFS and its re-evaluation trigger. ARC-25's text is untouched;
the note under it extends the limitation for snapshots and restates that the log is kept whole.

Three-platform build of C1 (zstd-sys's C build): CI run 38074891512 on the scratch branch
`scratch/sr-c1-build` at 5e202e8 (draft PRs skip the build jobs) — `fast`, `test`, `test-windows`,
`test-macos`, `platforms (macos-26)`, `platforms (windows-2025)`, `python` × 3: all success. **PASS**: the
C2 dependency on zstd is safe on every leg.

### 14.3 C2 — the snapshot codec and format 3

```text
persistence/src/format.rs   SAVE_FORMAT 3 (+ history line); SNAPSHOT_LEVEL 3; encode_snapshot (bulk
                            Compressor, include_checksum, include_contentsize); snapshot_json
                            (stream::decode_all; error "the snapshot at rN does not decompress: …");
                            decode_snapshot = decode(snapshot_json(..), "snapshot")
persistence/src/world.rs    create/commit (revision_row), checkpoint → encode_snapshot;
                            resume → decode_snapshot(bytes, revision)
persistence/src/replay.rs   verify compares encode(world.snapshot()) with snapshot_json(stored)
persistence/tests/save.rs   tamper test rewritten: (a) '{}' over the newest snapshot → Damaged naming the
                            revision, from resume and from verify; (b) genesis' frame copied over the
                            newest → SnapshotDisagreesWithHistory at that revision. Format test: 4 is
                            too new, 1 and 2 outdated, against 3. New: the committed base-built
                            format-2 fixture is refused by resume and verify (copied to scratch first)
tools/cli/tests/market/mod.rs   format::decode_snapshot for the newest snapshot
tools/cli/tests/save_retention.rs  ASR-8 through the binary: run, replay, inspect and server refuse a
                            format-2 save with "the save is format 2; this code reads format 3 …"
tools/cli/Cargo.toml        rusqlite as a dev-dependency (workspace's own; marks the manifest format)
.gitignore, .gitattributes  the fixture is committed (negated ignore) and binary
```

`ARC-81` check (I-SR-1, review): `git diff a5f5357 -- persistence/src` touches no journal or fact
encoding path — `revision_row`'s `entry` and `facts` lines and `reproduce` are unchanged; the only
`decode(.., "snapshot")` left is inside `decode_snapshot` (`grep '"snapshot"'` over `*.rs`).

```text
cargo test -p mineworld-persistence                     11 passed (save) + kill_and_resume cafe, clock
                                                        PASS                                  PASS
cargo test -p mineworld-cli --test save_retention --test inspect --test run
                                                        1 + 3 + 3 passed (rc 0; run includes the AC-12
                                                        300-day same-seed comparison)          PASS
cargo fmt --all --check                                 rc 0                                   PASS
cargo clippy -p mineworld-persistence -p mineworld-cli --all-targets -- -D warnings   rc 0    PASS
```

*Note.* The host is shared with other sessions' builds and 30-day runs; one earlier attempt at the CLI
tests was stopped by this session's own 10-minute tool limit while still compiling (no result; re-run
in the background above).
