# PR SR-b (step-24 FG-d) — Sealed-block compression of the world event log (facts and journal)

## DESIGN FROZEN 2026-10-12 (primary session; operator QSRB-3, QSRB-4)

```text
Design revision:        revision 2 (2026-10-12): revision 1 (PR #178, first commit) with §13.1's rulings
                        recorded, ARC-SRB replaced by ARC-89, and §12 filled
Approved by / evidence: the operator's rulings on QSRB-3 and QSRB-4 and the primary session's on QSRB-1,
                        QSRB-2, QSRB-5 … QSRB-8, all of 2026-10-12, relayed by the coordinator (§13.1)
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §12 (filled at freeze)
Lifecycle:              FROZEN — implementation starts only on the operator's go
```

Scope (§2.1), invariants (§2.3), decisions D-SRB-1 … D-SRB-11, acceptance and adversarial criteria
ASRB-1 … ASRB-14 and the commit plan are frozen. Progress, evidence, findings and bounded corrections stay
writable (§14). No SR-b question remains open. **Implementation starts only on the operator's go**, in a
fresh session (§12); this planning session does not implement.

*Superseded header:* `DESIGN DRAFT — awaiting the primary session's review and the operator's rulings
(not frozen)`, revision 1.

**Effort:** `mvp0` (filed under `mvp0/` per QS24-14) · **Step:** step-24 FG-d,
[`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md) §8.1 item 2, §10.1
(save-growth bound), §10.2 `AC-SAVE-1`, §11 (FG-d row), finding `F-S24-1` (§2.5) · **Outline:**
[`pr-s6-save-retention.md`](pr-s6-save-retention.md) §6.9 (SR-b), §4.4 (log compressibility), §5 option
5.8 · **Parent:** [`overall.md`](overall.md) "Saves grow without bound (finding F-SAVE-1)", the MVP-0 PR
list from step-24, the 12d rulings (30-day market-town save ≤ 192 MiB for towns with bodies,
2026-10-12).
**Working name:** SR-b. The PR number is assigned when the PR opens.
**Planning worktree:** `/Users/yuema137/mineworld-worktrees/design-sr-b`, branch `docs/sr-b-design`.

**Decision numbers (QSRB-8, ruled 2026-10-12 by the primary session).** **`ARC-89`** is SR-b's
architecture record (sealed log blocks and the streaming read path; amends `ARC-81` rule 1 and extends
`ARC-25`'s accepted limitation). Revision 1 wrote it as the placeholder `ARC-SRB`. No dependency record:
SR-b uses `zstd` exactly as `DEP-43` admitted it. The primary session records the number in `overall.md`'s
table. An implementation session never picks a number.

### Binding rules this design is written under

| Source | Rule | Where this design meets it |
| --- | --- | --- |
| `ARC-25` | The fact log is history: append-only, never derived, never rewritten. State is the journal re-executed; snapshots are checkpoints. Replay compares re-executed journal entries and facts with the logged ones **as bytes**. | I-SRB-1, I-SRB-2; D-SRB-1, D-SRB-6 |
| `ARC-27` / `AC-12` | The stored bytes of a run's facts, journal and snapshots are a function of pack, seed, age, pace and code; a re-run of a killed command finishes the same world. | D-SRB-2 (sealing is a function of the revision), D-SRB-5 (block bytes a function of the row bytes); ASRB-5, ASRB-6 |
| `ARC-49` / `AC-8` | `scripts/ci_parity.py` hashes every stored byte of a 30-day save on macOS, Linux and Windows. | D-SRB-5; §6.10; ASRB-7 |
| `AC-9` (`docs/MVP.md`) | Every fact's cause resolves (`mineworld inspect` checks it over the whole log). | D-SRB-8 (inspect streams and still checks every fact); ASRB-3 |
| `ARC-81` | Snapshots retained by a pure rule (genesis, an anchor every `64 × interval`, newest two), stored as zstd frames; **rule 1: `facts` and `journal` are only ever inserted into.** | Amended by `ARC-89` (D-SRB-1): rows move, unchanged, into sealed blocks; nothing else in `ARC-81` changes |
| `DEP-2` | `rusqlite` behind `PersistenceBackend`; the backend stores opaque encoded rows and knows nothing about meaning. | D-SRB-4: a block frames opaque rows; the backend still never decodes a fact or an entry |
| `DEP-43` | `zstd` 0.13, default features off, bundled libzstd, one fixed level per stored kind, no dictionary, single-threaded. | D-SRB-5 (level 9 for blocks, QSRB-2); no new dependency |
| Operator QSR-3 (precedent) | A save of an older format is refused by name; no migration. | D-SRB-9: `SAVE_FORMAT` 4; ASRB-8 |
| Operator QSR-1, QS24-5 | Build SR-b, in MVP-0, as its own PR. | this document |
| Operator QS24-4 | Save growth ≤ 0.25 MiB per simulated day (baseline profile), ≤ 0.5 with walking, Market Town, days 100–300; a PR may tighten, never loosen without a ruling. | ASRB-2; `AC-SAVE-1` (§6.11) |
| Operator 12d ruling (2026-10-12) | 30-day market-town save ≤ 192 MiB for towns with bodies. | ASRB-3 replaces it with a tighter bound (QSRB-4) |
| Operator, all platforms | macOS, Linux and Windows alike. | §6.12; ASRB-7 |
| `CLAUDE.md` §4.5 | Change amplification: no kernel, contracts, systems or packages edit. | I-SRB-6; ASRB-12 |

---

## 1. Goal, in one paragraph

After SR (#153) a Market Town save is almost entirely its log: on today's `main` (people walk) a
300-day save measures 1 123.8 MiB and grows by **3.75 MiB per simulated day**; with bodies (12d, PR #171)
1 668.1 MiB and **5.59 MiB per day** (§4). Facts and journal entries compress 18–40× when compressed
together in spans of 4 096 revisions, but only 2–2.5× one row at a time (§4.3, SR §4.4). This PR moves the
journal rows and fact rows of each completed 4 096-revision span, **unchanged byte for byte**, out of the
live tables into one sealed block per span: two zstd frames (journal and facts) of a length-prefixed
framing of the rows, written in the same SQLite transaction as the anchor commit that seals them. Replay
still compares re-executed entries and facts with the logged bytes, now decoded from a block. Every
reader of history becomes a stream that holds one block at a time, which also closes the persistence
side of `F-S24-1`: today `mineworld perceived`, `inspect` and `biography` each peak at 1.3–1.5 GiB of
memory on a 300-day save because they load the whole log (§4.5). Projected result: a 300-day Market Town
save of ≈ 45 MiB (walking) or ≈ 61 MiB (bodies), growing by ≈ 0.12–0.17 MiB per simulated day (§4.6).

## 2. Scope

### 2.1 In scope

- **The sealing rule** (D-SRB-2): at each anchor commit *a* of `ARC-81`'s lattice with *a* ≥ 2·*A*
  (*A* = 64 × interval = 4 096 by default), the live rows of revisions (*sealed_through*, *a* − *A*] are
  sealed into one block, in *a*'s transaction.
- **The block format** (D-SRB-3, D-SRB-4, D-SRB-5): a new table `sealed`, a framing of opaque rows, zstd
  level 9 frames with checksum and content size; a round-trip check before the live rows are deleted.
- **The streaming read path** (D-SRB-6, D-SRB-7): `PersistenceBackend` gains two cursors,
  `revisions_after` (journal row with its facts, revision order) and `facts_after` (fact rows, `EventId`
  order), both holding one decoded block at a time inside one read transaction; `journal_after` is removed;
  `facts_of`, `last_facts`, `head`, `highest_action_id` read sealed and live storage.
- **Replay on the stream** (`replay_after`, `verify`, `verify_from`, `resume`): one pass, facts taken from
  the cursor's revision, not from a `facts_of` query per revision.
- **Every production whole-log reader converted to the stream** (`F-S24-1`, persistence and CLI side):
  `tools/cli/src/history.rs` (`saved_facts`, `SavedHistory::perceived` — the server's backfill — and
  `mineworld perceived`), `inspect.rs` (journal summary and the `AC-9` check), `biography.rs`, `run.rs`
  (`Fingerprint::of_save`, `genesis_instant`).
- **`SAVE_FORMAT` 3 → 4**: schema change (`sealed` table; the `snapshots → journal` foreign key dropped,
  §6.5); a format-3 save refused by name.
- **`mineworld inspect`** prints one `log` line: blocks sealed, their revision range and stored bytes,
  the live tail (D-SRB-10).
- **CI**: `market_town.rs`'s 30-day bound tightened, a 300-day bound and a per-day growth assertion added
  (ASRB-1 … ASRB-3); `ci_parity.py` requires the `sealed` table; `scripts/baselines.txt`'s stored-table
  digests re-recorded once with a reason line; the nightly `AC-SAVE-1` (§6.11).
- `ARC-89` and the spec edits it requires (§6.13).

### 2.2 Not in scope, and where it goes

```text
cold archive segments outside world.sqlite (FG-e)                 MVP-1, after this PR (step-24 §8.1 item 3)
changing PerceivedHistory / the server backfill into a paged or   server lane, the remainder of F-S24-1
  streamed seam (the backfill still returns the perceived subset    (§6.8, QSRB-6); FG-e depends on it
  as one Vec and renders every frame before sending)
dropping, truncating or summarising any fact or journal row      never (ARC-25, INV-11, step-24 §8.1)
compressing the live tail row by row, or a trained dictionary     measured and declined (§4.4, §5)
a migration of format-3 saves                                     refused by name (QSR-3 precedent; QSRB-3)
a user-facing span or level setting                               not now (QSR-4 precedent)
fewer walk or bodies facts at the source                          movement / bodies lanes (step-24 §8.1 item 4)
F-SAVE-2 (Process state as a JSON number array)                   kernel lane; snapshot-only, unaffected
test helpers that load a whole 30-day log (Tables::read)          kept: test-scale, they now read the cursor
```

### 2.3 Invariants this PR must hold

- **I-SRB-1 The log is whole and unchanged.** Every journal row (revision, instant, `ActionId`, entry
  bytes) and every fact row (`EventId`, revision, envelope bytes) a format-3 build would write for a run
  is present in the format-4 save with identical bytes, either live or inside exactly one block. No path
  deletes a row without the same transaction holding it in a block that has been decoded and compared.
- **I-SRB-2 Replay compares the logged bytes.** Replay, `verify` and `verify_from` compare re-executed
  encodings with the row bytes a block or the live table yields, exactly as today. A block's compressed
  bytes are a storage representation; no comparison of world state is argued from them.
- **I-SRB-3 Sealing is a function of the revision.** Which rows are sealed after the commit of revision
  *n* depends only on *n*, the snapshot interval and the revision the previous block ends at — never on
  wall time, process lifetime or restarts. With the default interval the block boundaries are the fixed
  multiples of 4 096. A killed-and-resumed headless run holds the same rows and blocks as an uninterrupted
  one (`ARC-27`).
- **I-SRB-4 Resume reads no block.** The snapshot a resume restores is always newer than every sealed
  revision, so resuming a world decodes no block (it is what lets FG-e leave old blocks cold).
- **I-SRB-5 A reader sees one consistent history.** A cursor reads inside one SQLite read transaction:
  a seal committed by the writer while a reader iterates cannot make the reader miss or repeat a row.
- **I-SRB-6 Kernel, contracts, systems and packages unchanged.** No file under `kernel/`, `contracts/`,
  `systems/`, `packages/` changes.

## 3. Audit anchors (`origin/main @ cda4c13`)

| Where | What it says today | Consequence for this design |
| --- | --- | --- |
| `persistence/src/sqlite.rs` `SCHEMA` | four tables; `facts(event_id PK, revision REFERENCES journal, fact)`, index `facts_by_revision`; `snapshots(revision PK REFERENCES journal, snapshot)`; `PRAGMA foreign_keys = ON` | deleting a sealed journal row that a retained anchor or genesis snapshot references would violate the FK: format 4 drops that FK (§6.5) |
| `sqlite.rs` `write_revision` | inserts journal, facts, snapshot; deletes `retire` rows (each must delete exactly 1 row) — one transaction | sealing attaches after the retirement, same transaction (D-SRB-2) |
| `sqlite.rs` `journal_after`, `last_facts`, `facts_of` | `Vec` of every row after a revision; `ORDER BY event_id DESC LIMIT n`; per-revision `SELECT` | replaced or re-implemented over blocks + tail (D-SRB-6) |
| `sqlite.rs` `head`, `highest_action_id` | `MAX(revision)`, `MAX(action_id)` over `journal` | also over `sealed.last_revision`, `sealed.max_action_id` |
| `persistence/src/backend.rs` | `RevisionRow { revision, at, action_id, entry, facts, snapshot, retire }`; trait of 11 methods | gains `seal: Option<RevisionSpan>`; two cursor methods; `journal_after` removed |
| `persistence/src/world.rs` `ANCHOR_EVERY = 64`, `retired`, `commit` | the anchor lattice and the pure retention rule; `commit` fills `retire` on scheduled revisions | the sealing rule sits beside `retired`, on the same lattice (D-SRB-2) |
| `persistence/src/replay.rs` `replay_after`, `reproduce` | `backend.journal_after(from)?` (whole tail in a `Vec`), then `backend.facts_of(revision)` per revision | one cursor pass; a per-revision `facts_of` over sealed storage would decode a block 4 096 times |
| `persistence/src/format.rs` | `SAVE_FORMAT = 3`; `SNAPSHOT_LEVEL = 3`; `encode_snapshot` uses `zstd::bulk::Compressor` with checksum and content size | 4; a block codec beside it (D-SRB-4) |
| `tools/cli/src/history.rs` `saved_facts` | `last_facts(i64::MAX)` decoded into one `Vec<EventEnvelope>`; used by `SavedHistory::perceived` (server backfill) and `mineworld perceived` | `F-S24-1`: becomes a stream into `perceived_by` |
| `systems/presence/src/audience.rs` `perceived_by` | takes any `IntoIterator<Item: Borrow<EventEnvelope>>`, applies whereabouts to every fact from the first | already streamable; must still see every fact from genesis (no skipping blocks below `since`) |
| `server/src/protocol/fact.rs` `read_backfill` | `history.perceived(…)` on a blocking task, then renders every frame into a `Vec<ServerFrame>` | the seam stays (§2.2); memory falls to the perceived subset |
| `tools/cli/src/inspect.rs` | `Journal::read` = `journal_after(0)`; `facts` = all facts decoded; `Causes::check(&facts, …)` keeps a `BTreeSet` of seen ids | streamed: one pass each, a ring of the last *n* facts, the id set kept (8–40 B per fact) |
| `tools/cli/src/biography.rs` | all facts decoded; `names_in(&facts)`; filter `selected` | one streamed pass collecting `named` facts and the person's entries |
| `tools/cli/src/run.rs` `Fingerprint::of_save`, `genesis_instant` | all facts in memory for an FNV fingerprint; `journal_after(0)` to read the first row | streamed; `facts_of(GENESIS)` / first cursor row only |
| `server/src/host.rs` `persisted` | `recent_facts(recent_events = 64)` | `last_facts(64)` reads the live tail first |
| `tools/cli/src/serve.rs:325`, `biography.rs:195`, `inspect.rs:57`, `run.rs:335` | `facts_of(GENESIS)` | genesis is in block 1 after the first seal: one block decoded (≈ 1 ms) |
| `persistence/tests/kill_and_resume.rs:571-595` | compares survivor and control by raw SQL over `journal`, `facts`, `snapshots`; intervals 4 (cafe, 301 revisions) and 8 (clock, 634) | no seal happens at those sizes (first seal at 2·*A* = 512 / 1 024): intervals change, `sealed` compared (C6) |
| `persistence/tests/save.rs:464-500`, `retention.rs:275`, `tools/cli/tests/save_retention.rs:91` | raw SQL tampering of `facts` / `snapshots`, format marking | adjusted; a block-tamper test added |
| `tools/cli/tests/headless/mod.rs` `Tables::read`, `assert_same_history` | `last_facts(all)`, `journal_after(0)`, snapshots; byte equality (AC-12) | reads the cursors and the raw `sealed` rows; equality over all four |
| `tests/acceptance/tests/{arrival_resolvers_resume,configuration_seam}.rs` | `journal_after(0)` + `facts_of` per revision | move to `revisions_after` |
| `tools/cli/tests/market_town.rs` `save_is_bounded` | 30-day save ≤ 128 MiB on `main` (≤ 192 MiB on the 12d branch); prints the 300-day size unbounded | tightened and extended (ASRB-1 … ASRB-3) |
| `scripts/ci_parity.py` | `REQUIRED_TABLES = ("facts", "journal", "snapshots")`; hashes every table but `manifest`, generically, in key order, chunked per 1 000 rows; baselines `facts-300s` = `table-300s.facts.rows` | `sealed` hashed automatically; added to `REQUIRED_TABLES`; `facts-300s` / `journal-300s` change meaning to the live tail (§6.10) |
| `scripts/ci_stability.py replay` (nightly) | every world saved 300 days and replayed; CA-13 hosts market-town's save | the host of `AC-SAVE-1` (§6.11) |
| `Cargo.lock` | `zstd 0.13.3`, `zstd-sys 2.1.1+zstd.1.5.7`; no SHA-2 crate | no new crate; the one-time logical-equality proof uses `sqlite3` + `shasum` on an unsealed copy (C7) |

## 4. Measurements (planning, scratch; not evidence for any acceptance criterion)

All on the operator's macOS 26.2 arm64 laptop, release profile, seed 7,
`mineworld run worlds/market-town --headless --seed 7 --days N --packs entities --packs
presentation/mineworld-default --save DIR`. A 300-day save was made by running to day 100 and then to
day 300 on the same save (a resume; `ARC-27` makes it the same history). Table sizes are SQLite `dbstat`
page totals; row sizes `length()` of stored blobs. Scratch saves lived under the worktree's `target/` and
were deleted after measuring.

### 4.1 Today's saves (`main @ cda4c13`, people walk)

| | 30 days | 100 days | 300 days |
| --- | ---: | ---: | ---: |
| `world.sqlite` | 117 772 288 B (112.3 MiB) | 391 278 592 B (373.2 MiB) | 1 178 353 664 B (1 123.8 MiB) |
| head revision | 60 331 | 200 926 | 603 633 |
| fact rows | 82 298 (raw 76.4 MB, mean 928 B) | 272 687 | 817 834 (raw 763.2 MB) |
| journal raw | 25.6 MB (max row 298 362 B: genesis) | — | 257.3 MB |
| `dbstat` facts / journal / index / snapshots | 87.0 / 28.8 / 1.0 / 0.9 MB | — | 868.5 / 290.7 / 11.2 / 8.0 MB |
| snapshot rows | 17 | — | 150 (53 KB each) |
| wall, peak RSS of the run | 7.1 s, 113 MB | 23.0 s, 319 MB | 46.0 s (resumed), 911 MB |

Growth over days 100–300: (1 178 353 664 − 391 278 592) ÷ 200 = **3 935 375 B ≈ 3.75 MiB per simulated
day**; 2 013.5 revisions and 2 725.7 facts per day. The 30-day size equals CI's (run 38091503186,
112.3 MiB).

### 4.2 Towns with bodies (12d branch `origin/mvp0/pr-12d-towns @ 4e4ce10`, PR #171, not merged)

Built in a scratch detached worktree, removed afterwards.

| | 100 days | 300 days |
| --- | ---: | ---: |
| `world.sqlite` | 577 343 488 B | 1 749 176 320 B (1 668.1 MiB; 12d's record says the same) |
| head revision | ≈ 276 735 | 833 121 |
| fact rows | — | 1 219 332 (raw 1 170.8 MB) |
| journal raw | — | 322.2 MB |
| `dbstat` facts / journal / index / snapshots | — | 1 358.3 / 362.5 / 16.7 / 11.7 MB |
| snapshot rows | — | 206 (55 KB each) |
| wall, peak RSS | 38.2 s, 484 MB | 78.5 s (resumed), 1 376 MB |

Growth over days 100–300: **5 859 164 B ≈ 5.59 MiB per day**, ≈ 2 782 revisions per day.

### 4.3 One block of 4 096 revisions

"Raw" is the concatenation of the rows' stored bytes in key order, without the framing D-SRB-4 adds
(≈ 20–30 B per row before compression, highly repetitive). zstd 1.5.7 CLI, `-T1`.

| Block | facts raw → L3 / L9 / L19 | journal raw → L3 / L9 / L19 |
| --- | --- | --- |
| walking, 30-day save, revisions 1–4 096 (holds genesis) | 5 491 141 → 252 119 / 158 384 / 135 262 | 1 986 196 → 129 987 / 97 377 / 76 986 |
| walking, 30-day, 28 673–32 768 | 5 156 287 → 215 811 (23.9×) / 129 050 (40.0×) / 112 074 | 1 733 093 → 95 134 (18.2×) / 65 508 (26.5×) / 53 414 |
| walking, 300-day, blocks 50 / 100 / 140 | ≈ 5.17 MB → 214–217 K / 127–128 K | ≈ 1.76 MB → 92 K / 64.5 K |
| bodies, 300-day, blocks 100 / 150 / 200 | 5.6–6.0 MB → 227–234 K / 137–143 K | ≈ 1.59 MB → 84–85 K / 59–60 K |

The ratio is flat from block 4 to block 140: block size, not age, decides it.

Levels on one combined block (walking, 28 673–32 768, 6 889 380 B): L3 310 966 · L6 227 621 · **L9
194 596 (35.4×)** · L12 183 634 · L19 162 194. Wall per block, CLI including process start: L3 < 10 ms,
L9 ≈ 10–20 ms (`zstd -b9` ≈ 430 MB/s → ≈ 16 ms), L12 ≈ 30 ms, L19 1.35 s; decompression ≈ 12 GB/s
(≈ 0.6 ms per block). Two frames (facts, journal) cost nothing over one: 129 050 + 65 508 = 194 558 vs
194 596 at L9.

### 4.4 A trained dictionary

A 64 KB dictionary trained (`zstd --train`) on 3 274 rows of revisions 1–8 192, applied to the block
above: L3 310 218 B vs 310 966 (−0.2 %); L9 190 679 vs 194 596 (−2.0 %). At block scale a dictionary buys
nothing worth a second stored, versioned artefact. (Per row, a dictionary gives 9–11×, SR §4.4 — still
half of what a block gives without one.)

### 4.5 Readers of a 300-day save (`F-S24-1`, walking save of §4.1)

`/usr/bin/time -l`, release binary at `cda4c13`.

| Command | Peak RSS | Wall | Why |
| --- | ---: | ---: | --- |
| `mineworld perceived worlds/market-town --save D --person alice` | 1 408 401 408 B (1.31 GiB) | 2.70 s | `saved_facts`: every fact decoded into one `Vec` (346 899 lines printed) |
| `mineworld inspect D --last 0` | 1 598 537 728 B (1.49 GiB) | 2.68 s | every journal row and every fact in memory |
| `mineworld biography worlds/market-town --save D --person alice` | 1 413 988 352 B (1.32 GiB) | 1.98 s | every fact decoded (6 260 entries printed) |
| `mineworld replay worlds/market-town --save D` | 330 743 808 B (315 MiB) | 6.65 s | `journal_after(0)`: every journal row in one `Vec` |
| `mineworld run … --days 300` resumed from day 100 | 911 032 320 B (869 MiB) | 46.0 s | `Fingerprint::of_save` reads every fact at the end |

The server's backfill (`SavedHistory::perceived`) is the same `saved_facts` as the first row, on a
blocking task of a hosted world (CA-13, `tools/cli/tests/perceived.rs`).

### 4.6 Projections (computed from §4.1–4.3; labelled; to be measured at C7)

```text
per simulated day, Market Town, days 100–300
  walking   blocks/day 2 013.5 / 4 096 = 0.492
            L9  0.492 × 192.6 KB (block) + 0.492 × 53.2 KB (anchor)  = 120.8 KB ≈ 0.115 MiB/day
            L3  0.492 × 306.5 KB         + 26.1 KB                    = 176.8 KB ≈ 0.169 MiB/day
  bodies    blocks/day 2 782 / 4 096 = 0.679
            L9  0.679 × 199.5 KB + 0.679 × 55.1 KB                   = 172.9 KB ≈ 0.165 MiB/day
            L3  0.679 × 313.5 KB + 37.4 KB                            = 250.3 KB ≈ 0.239 MiB/day

300-day save (sealed blocks + live tail ≤ 2·A revisions at ≈ 1.15 page overhead + snapshots)
  walking   L9  146 blocks 28.1 MB + tail 5 617 rev ≈ 10.9 MB + snapshots 8.0 MB   ≈ 47 MB ≈ 45 MiB
  bodies    L9  202 blocks 40.3 MB + tail 5 729 rev ≈ 11.9 MB + snapshots 11.7 MB  ≈ 64 MB ≈ 61 MiB
            L3  ≈ 84 MiB
30-day save
  walking   L9  13 blocks 2.5 MB + tail 7 083 rev ≈ 13.7 MB + snapshots 0.9 MB      ≈ 17 MB
  bodies    L9  ≈ 18 blocks 3.6 MB + tail ≤ 8 191 rev ≈ 16.7 MB + snapshots 1.2 MB ≈ 22 MB
```

Both profiles fall under QS24-4's 0.5 MiB/day with walking, at either level; only level 9 leaves margin
under 0.25 MiB/day with bodies.

## 5. Reuse comparison (`REUSE_POLICY.md` §§1–4, 11–12)

SR §5 compared storage options for the whole save and chose 5.8 (this PR) as the remaining lever; the
operator approved it (QSR-1, QS24-5). The comparison here is between ways of building 5.8.

| # | Option | 300-day size (walking, est.) | Readers | Determinism / platforms | Complexity | Verdict |
| --- | --- | ---: | --- | --- | --- | --- |
| B1 | **Blocks of 4 096 revisions in a `sealed` table, two zstd frames each, on the anchor lattice** | ≈ 45 MiB (L9) | one block in memory; streaming cursors | `zstd` bundled, fixed level, no dictionary — as snapshots already are under `AC-8` | medium: one table, one codec, two cursors | **adopted** |
| B2 | Per-row zstd, no dictionary | ≈ 600 MiB (2–2.5×, SR §4.4) | unchanged row access | as B1 | low | declined: misses QS24-4 by 5× |
| B3 | Per-row zstd with a trained dictionary | ≈ 120 MiB (9–11×) | unchanged row access | a stored, versioned dictionary; retraining changes bytes | medium | declined: half B1's ratio, plus a dictionary lifecycle |
| B4 | Blocks as files beside `world.sqlite` (segments) | ≈ B1 | as B1 | two-store atomicity: a block file and the row deletion cannot share one SQLite transaction | high | deferred to FG-e, which moves already-sealed blocks out (step-24 §8.1 item 3) |
| B5 | SQLite page compression (ZIPVFS, `sqlite-zstd`) | ≈ 10× | transparent | proprietary; LGPL-3.0 and asynchronous | — | rejected in SR §5.5 (`ARC-55`) |
| B6 | Columnar container (Parquet / Arrow IPC) for blocks | ≈ B1 | needs a reader library | a large dependency for opaque byte rows | high | declined: no column structure to exploit — rows are opaque JSON |
| B7 | B1 with blocks keyed to a fixed constant 4 096, independent of the snapshot interval | ≈ B1 | as B1 | as B1 | a second lattice and a test-only knob to reach it | declined for D-SRB-2's lattice (QSRB-1) |

Reusing, not reinventing: zstd (already `DEP-43`), SQLite transactions for atomicity, WAL read
transactions for consistent readers. Written here: a 30-line framing of opaque rows (a length-prefixed
record stream; no general format fits "opaque byte rows, keys, exact round trip" more simply) and the
two cursors. Not forced: no framework's storage model; `world.sqlite` stays one file.

## 6. Design

### 6.1 Decisions (D-SRB-1 … D-SRB-11)

| ID | Decision |
| --- | --- |
| D-SRB-1 | **Rows move; bytes do not change.** A journal row or fact row is either live (in `journal` / `facts`) or inside exactly one sealed block, with the same key columns and the same bytes. Amends `ARC-81` rule 1 ("only ever inserted into") to "inserted into, and moved unchanged into a sealed block". No other deletion of a log row exists. |
| D-SRB-2 | **The sealing rule.** Let *A* = `ANCHOR_EVERY × interval` (4 096 at the default interval 64). When revision *n* is committed with *n* mod *A* = 0 and *n* ≥ 2·*A*, the rows of revisions (*sealed_through*, *n* − *A*] are sealed into one block in *n*'s transaction, where *sealed_through* is the last revision of the newest block (0 if none). With a fixed interval the blocks are exactly (*kA*, (*k*+1)*A*], *k* ≥ 0, sealed one span late. A pure function `sealing(n, interval, sealed_through) -> Option<RevisionSpan>` in `world.rs`, beside `retired`. |
| D-SRB-3 | **The `sealed` table.** `sealed(first_revision INTEGER PRIMARY KEY, last_revision INTEGER NOT NULL, first_event_id INTEGER, last_event_id INTEGER, max_action_id INTEGER, journal BLOB NOT NULL, facts BLOB NOT NULL, CHECK (first_revision <= last_revision))`. Event-id and action-id columns are `NULL` when the span recorded none. Spans are contiguous and non-overlapping; the live tail starts at `last_revision + 1` of the newest block. |
| D-SRB-4 | **The framing**, in a new module `persistence/src/block.rs` (storage representation of opaque rows; it decodes no entry and no fact): journal frame = for each row in revision order `revision u64 LE · at i64 LE · action_id (u8 0, or u8 1 + u64 LE) · length u32 LE · entry bytes`; facts frame = for each row in `EventId` order `event_id u64 LE · revision u64 LE · length u32 LE · fact bytes`. Decoding checks: revisions contiguous and equal to the block's span, every fact's revision inside it, event ids strictly increasing and equal to the block's first/last columns, no trailing bytes. Any failure is `PersistError::Damaged` naming the block's span. |
| D-SRB-5 | **The codec.** Each frame is `zstd(level 9, checksum on, content size in header, single-threaded, no dictionary)` through `zstd::bulk::Compressor`, as `encode_snapshot` already does at level 3. One fixed level for blocks (`BLOCK_LEVEL = 9`, QSRB-2): stored bytes are then a pure function of the row bytes on every platform, which `AC-8` checks rather than assumes. |
| D-SRB-6 | **The cursors.** `PersistenceBackend` gains `revisions_after(after) -> Result<Box<dyn Iterator<Item = Result<StoredRevision, PersistError>> + '_>, PersistError>` (`StoredRevision { revision, at, action_id, entry, facts: Vec<FactRow> }`, revision order) and `facts_after(after: Option<u64>) -> … Iterator<Item = Result<FactRow, PersistError>>` (`EventId` order). Each opens one read transaction (`Connection::unchecked_transaction`, deferred) for its whole life (I-SRB-5), walks the blocks overlapping the request in order decoding one at a time (`facts_after` decodes only the facts frame), then pages the live tail by key range (1 024 rows per query). `journal_after` is removed; `facts_of(r)` decodes one block when *r* is sealed; `last_facts(n)` reads the tail newest-first and decodes blocks backwards only if the tail holds fewer than *n*; `head` and `highest_action_id` take the maximum of live and sealed columns. |
| D-SRB-7 | **Replay on the cursor.** `replay_after` iterates `revisions_after(from)`; `reproduce` compares the re-executed entry with `StoredRevision::entry` and the re-executed facts with `StoredRevision::facts` — the same byte comparisons, no per-revision `facts_of`. `reexecute` (verify, `verify_from`) and `resume` use it unchanged in meaning. Memory: one block plus the world. |
| D-SRB-8 | **Every production whole-log reader streams** (`F-S24-1`, persistence and CLI side): `saved_facts` becomes `for_each_saved_fact` / a fallible iterator over `facts_after(None)` decoding one envelope at a time; `SavedHistory::perceived` feeds it to `perceived_by(…).take_while(id ≤ through)` and stops at `through`; `mineworld perceived` prints as it goes; `inspect` streams the journal summary and the facts once (type counts, `AC-9` with the seen-id set, a ring of the last *n*); `biography` streams once (collecting `named` facts for `names_in` and the person's selected entries, then renders); `Fingerprint::of_save` streams; `genesis_instant` reads `facts_of(GENESIS)` or the first cursor row. A fallible iterator meets `perceived_by`'s infallible `IntoIterator` through a small adapter that stops at the first error and returns it after the pass. |
| D-SRB-9 | **`SAVE_FORMAT` 4.** Schema: `sealed` added; `snapshots.revision` loses `REFERENCES journal (revision)` (a retained anchor or genesis snapshot outlives its sealed journal row); `facts.revision → journal` kept (both move together; facts are deleted first). A format-3 save is refused with `SaveFormatOutdated { saved: 3, supported: 4 }` by `run`, `serve`/`server`, `replay`, `inspect`, `perceived`, `biography`; no migration (QSR-3 precedent, QSRB-3). |
| D-SRB-10 | **`inspect`** prints after the snapshots line: `log        146 blocks sealed r1–r598016 (28.1 MB stored, 1 012.4 MB of rows); live r598017–r603633` (the numbers are illustrative). `MODULE_SPEC.md` §8.1 lists it. |
| D-SRB-11 | **Sealing is verified before rows are deleted.** In the sealing transaction the backend reads the live rows of the span, frames and compresses them, decompresses and re-parses its own frames and compares every row with the source, inserts the block, deletes `facts` then `journal` rows of the span, and requires the deleted counts to equal the block's row counts. Any mismatch fails the commit (`Damaged`), so the world becomes `WorldAheadOfSave`, as for any failed commit. |

### 6.2 Why the anchor lattice, one span late (D-SRB-2)

- **Resume reads no block (I-SRB-4).** The newest stored snapshot is at or after the newest anchor *a*
  (`ARC-81` keeps every anchor and the newest two), and the newest block ends at *a* − *A* < *a*. So the
  tail a resume re-executes is live — for any interval, including a clean shutdown's off-lattice
  checkpoint. Sealing the span ending at *n* itself would also satisfy this at the default interval, but
  not after an interval change or an off-lattice checkpoint below *n*.
- **Recent reads stay cheap.** The server's `recent_facts(64)`, `head_input`, and a backfill from a
  recent cursor touch only the live tail, which holds between *A* and 2·*A* − 1 revisions (≈ 7–15 MB raw).
  The tail is a constant, not growth: it does not enter the per-day bound.
- **FG-e alignment.** Every block starts one past an anchor revision and ends at the next anchor, so a
  cold segment built from blocks begins at an anchor and carries its own restart point
  (step-24 §8.1 item 3).
- **One lattice, no new knob.** Tests reach sealing with small intervals exactly as they reach anchors
  today (interval 2 → *A* = 128, first seal at revision 256). The rule reads *sealed_through* rather than
  assuming *n* − 2·*A*, so a save resumed under another interval stays well-formed: its next block is
  (*sealed_through*, *n* − *A*] on the new lattice. Production paths never change the interval (64), so a
  production save's blocks are the fixed multiples of 4 096 — a function of the revision alone.

### 6.3 Why level 9 (D-SRB-5, QSRB-2)

A block is written once per 4 096 revisions — every 2.0 simulated days (walking) or 1.5 (bodies) — not
every 64 like a snapshot. Level 9 stores 37 % less than level 3 (194.6 KB vs 311.0 KB per block, §4.3)
for ≈ 10 ms more per block, and is the difference between ≈ 0.165 and ≈ 0.239 MiB/day with bodies (§4.6).
Level 19 saves another 17 % for > 1 s per block on the world thread: declined. The level is a constant;
a level change changes no decoded byte and no replay verdict (I-SRB-2), which the adversarial run of
§8 shows.

### 6.4 The trait and row changes

```rust
pub struct RevisionSpan { pub first: WorldRevision, pub last: WorldRevision }

pub struct RevisionRow {
    // … unchanged fields, `retire` included …
    /// The span whose live rows this revision's transaction seals into one block, after the row,
    /// its snapshot and its retirements are written (`ARC-89`). Always entirely below `revision`.
    pub seal: Option<RevisionSpan>,
}

pub struct StoredRevision {
    pub revision: WorldRevision,
    pub at: i64,
    pub action_id: Option<u64>,
    pub entry: Vec<u8>,
    pub facts: Vec<FactRow>,
}
```

`PersistentWorld` keeps `sealed_through` (read once at `create`/`resume` with one query, updated only after
the backend reports the commit, as `stored` is). The guard that refuses a `retire` entry ≥ *n* gains its
twin: a `seal` span whose `last` ≥ *n* − *A* + 1 is refused before the transaction (debug assertion and
`Damaged`).

### 6.5 Crash safety

| Moment of a kill or a power cut | What the save holds afterwards |
| --- | --- |
| before the sealing commit's transaction commits | the previous revision as head; the span still live; the next run re-executes revision *n* and seals then — the same block bytes (D-SRB-2, D-SRB-5) |
| after it commits | revision *n*, its snapshot, its retirements and the block; the span's live rows gone |
| power loss with `Durability::ProcessCrash` (headless) | SQLite WAL semantics as today: the newest commits may be lost whole, never in part |

There is no state in which a row is in neither place or in both: the insert of the block and the deletes
are one transaction. No `VACUUM` runs; the pages freed by the deletes are reused by the next 4 096
revisions' inserts, so the file grows by the block and the anchor only (≈ 250 KB per span), not by the
tail. The foreign-key change (D-SRB-9) is what makes the deletes legal while genesis and anchor snapshots
inside the span are retained.

### 6.6 Error behaviour

| Case | Behaviour |
| --- | --- |
| a block frame is not zstd, fails its checksum, or its content size disagrees | `Damaged { detail: "the sealed block r…–r… does not decompress: …" }`; replay, verify and readers stop; resume is unaffected unless it needs that block (it never does, I-SRB-4) |
| a frame decompresses but its framing is wrong (non-contiguous revisions, a fact outside the span, event ids out of order, trailing bytes, columns disagree) | `Damaged` naming the span and the first inconsistency |
| a block decodes but a row differs from history | `ReplayDiverged { revision, … }` from replay/verify, as for a live row |
| spans overlap, leave a gap, or the tail does not start after the newest block | `Damaged` from the cursor ("the log skips from r… to r…"), as `replay_after` reports a skip today |
| the seal's round-trip check or deleted counts disagree (D-SRB-11) | the commit fails; `WorldAheadOfSave` |
| a format-3 save | refused before any read, naming 3 and 4 (D-SRB-9) |

### 6.7 The reading path, reader by reader

| Reader | Today | After |
| --- | --- | --- |
| `resume` | `latest_snapshot`, `journal_after(snapshot)`, `facts_of` per revision | `latest_snapshot`, `revisions_after(snapshot)` — live rows only (I-SRB-4) |
| `verify`, `mineworld replay` | whole journal in a `Vec`, `facts_of` per revision | one cursor pass from genesis, one block at a time |
| `verify_from(anchor)` | as above from the anchor | cursor from the anchor; the first block decoded may start below it (rows ≤ anchor skipped) |
| server backfill `SavedHistory::perceived` | every fact decoded into a `Vec`, then filtered | `facts_after(None)` → decode → `perceived_by` → `take_while(≤ through)`; stops at `through`; returns the perceived subset |
| `mineworld perceived` | same `Vec` | same stream, printed as it goes |
| `mineworld inspect` | journal `Vec` + facts `Vec` | two streamed passes; id set and a ring of `--last` facts |
| `mineworld biography` | facts `Vec`, `names_in`, filter | one streamed pass |
| `run` end-of-run fingerprint | facts `Vec` | streamed |
| `run`'s `genesis_instant` | `journal_after(0)` (the whole journal, to read one row) | `facts_of(GENESIS)`, else the first cursor row |
| `host.rs` `recent_facts(64)`, `run`'s `head_input` | `last_facts`, `journal_after(head − 1)` | live tail |

`perceived_by` applies whereabouts from the world's first fact, so a backfill from a late cursor still
decodes every block's facts frame once; it no longer holds them. What stays unbounded is the perceived
subset the seam returns and the frames `read_backfill` renders (§2.2, QSRB-6).

### 6.8 What `F-S24-1` asked, and what this PR answers

`F-S24-1` (step-24 §2.5) names backfill and "several CLI commands" loading the whole fact log, as the first
memory wall and a blocker of archival. This PR removes every whole-log load in `persistence/` and
`tools/cli/src/` (§6.7) and makes the backend's history a cursor, which FG-e's segment files can sit
behind. The remaining part is the server's seam (`PerceivedHistory::perceived` returns a `Vec`;
`read_backfill` renders all frames before sending): a protocol-adjacent change owned by the server lane,
recorded as a request (§11 R-server) and asked as QSRB-6.

### 6.9 `SAVE_FORMAT` 4 and the format-3 fixture

`format.rs`: `SAVE_FORMAT = 4`, history line "4 — SR-b (`ARC-89`): journal and fact rows of each
completed span live in a sealed block; `snapshots` no longer references `journal`." A committed fixture
`persistence/tests/fixtures/format-3/world.sqlite` is written by a build of the implementation base (the
SR precedent, §14.1 of `pr-s6-save-retention.md`): the persistence test world, small, format 3 in its
manifest. The format-2 fixture stays and is refused as outdated (2 < 4).

### 6.10 `AC-8`, `AC-12` and the baselines

- **`AC-12`** (`Tables::read`, `assert_same_history`): compares, byte for byte, the logical journal and
  facts (through the cursors), the snapshots, **and the raw `sealed` rows**. Two runs of one build give
  equal blocks because D-SRB-2 and D-SRB-5 make them functions of the rows.
- **`AC-8`** (`ci_parity.py`): `table_digests` already hashes every table but `manifest`, generically; the
  `sealed` table is hashed with no code change, and `REQUIRED_TABLES` gains `"sealed"` so a save without it
  is refused by name. The walking 30-day save seals 13 blocks (§4.6), so the three-platform comparison
  covers block bytes on every PR.
- **`scripts/baselines.txt`**: `facts-300s` and `journal-300s` are digests of the stored `facts` and
  `journal` tables; after SR-b those tables hold the live tail only, so both values change once, with a
  reason line ("SR-b: the log's storage changed, not the log"). `sealed-300s` is added
  (`table-300s.sealed.rows`). **`summary-300s` must not change** for any world: it digests `run`'s printed
  lines, which include the `history … fingerprint` line computed from every logged fact's bytes — the
  continuous evidence that the facts are unchanged. The journal's logical equality is proved once, at C7,
  on an unsealed copy (ASRB-11). Whether `facts-300s`/`journal-300s` should instead become logical digests
  is QSRB-5.

### 6.11 `AC-SAVE-1` (nightly) and the CI assertions

- **CI, every PR** (`tools/cli/tests/market_town.rs`, which already makes a 30-day and a 300-day saved run
  in the default suite): ASRB-3's 30-day bound; ASRB-1's 300-day bound; the per-day growth
  (*S*300 − *S*30) ÷ 270 ≤ ASRB-2's bound, from the two saves of the same seed. Each message prints the
  measured value and the bound.
- **Nightly `AC-SAVE-1`** (`scripts/ci_stability.py replay`, market-town only): run to day 100, record
  `world.sqlite` + `-wal`, run on to day 300 (a resume, the same history by `ARC-27`), record again,
  assert (*S*300 − *S*100) ÷ 200 ≤ the bound, then replay and CA-13 as today. The sizes go to the
  timings artefact. The self-test gains the verdict rule. Bound: **0.5 MiB/day** (QS24-4's step bound,
  ruled for the nightly by QSRB-4 "until stable"); the measured value is recorded each night, and a later
  ruling tightens it towards this PR's 0.25.

### 6.12 Platforms

No new native code: `zstd-sys` already builds on the three CI platforms (SR C1). Block bytes are a pure
function of the row bytes under a pinned libzstd, single-threaded, fixed level — the property `AC-8`
already verifies for snapshot frames — and the framing is explicit little-endian. SQLite transaction and
WAL read-transaction semantics are identical on all three.

### 6.13 Spec edits (C1)

- `docs/DECISIONS.md`: `ARC-89` (D-SRB-1 … D-SRB-11, §5's comparison, revisit triggers: "a block-level
  `AC-8` difference on any leg", "FG-e moves blocks out of the file", "a world whose span exceeds 64 MiB
  raw"); a dated note under `ARC-81` (rule 1 amended) and under `ARC-25`'s accepted limitation (the log is
  compressed, still whole, still replayed byte for byte).
- `docs/ARCHITECTURE.md` persistence section: the log is stored as a live tail and sealed blocks; readers
  stream.
- `docs/MODULE_SPEC.md` §8.1: `inspect`'s `log` line.
- `persistence/README.md`, `persistence/src/lib.rs` and `sqlite.rs` module docs.
- `step-06-persistence.md` §9 L-6: dated note that log compression landed in SR-b; the log still whole.

## 7. Test ownership

| Failure class | Owner | File |
| --- | --- | --- |
| framing round trip and every decode check of D-SRB-4 (gap, fact outside span, unordered ids, trailing bytes, truncated frame, wrong checksum) | Unit | `persistence/src/block.rs` tests |
| the sealing rule over intervals, heads and `sealed_through` (table-driven, incl. changed interval and *n* < 2·*A*) | Unit | `persistence/src/world.rs` tests |
| after every commit of a long sweep the sealed spans equal the rule's oracle, the tail starts after them, every row is in exactly one place, resume decodes no block | Integration | `persistence/tests/retention.rs` (extended sweep) |
| replay / verify / `verify_from` from every anchor across sealed and live storage | Integration | `persistence/tests/retention.rs`, `save.rs` |
| kill at, before and after a sealing commit: survivor = control in logical rows, snapshots and raw blocks | Integration (real child process) | `persistence/tests/kill_and_resume.rs` |
| a reader iterating while the writer seals sees every row once (I-SRB-5) | Integration (two connections) | `persistence/tests/save.rs` |
| a tampered block → `Damaged` naming the span; a valid frame of other rows → `ReplayDiverged` | Integration | `persistence/tests/save.rs` |
| format 3 refused (base-built fixture: library; binary: run, replay, inspect, server, perceived, biography) | Integration | `persistence/tests/save.rs`, `tools/cli/tests/save_retention.rs` |
| AC-12 byte equality incl. `sealed` | existing, extended | `tools/cli/tests/headless/mod.rs` |
| AC-8 three platforms incl. `sealed` | CI | `scripts/ci_parity.py` |
| ASRB-1/2/3 sizes and growth | CI assertion | `tools/cli/tests/market_town.rs` |
| AC-SAVE-1 | nightly | `scripts/ci_stability.py` (+ self-test) |
| reader memory bounds (ASRB-4) and CA-13 backfill | recorded measurement (release, `/usr/bin/time`), CA-13 ignored test run once | C7 |
| static: fmt, clippy `-D warnings`, doc checks | static | CI `fast` |
| real LLM | NOT REQUIRED: no model-facing boundary changes | — |

## 8. Acceptance and adversarial criteria (fixed now, before anything is measured)

Measured at C7 on Market Town, seed 7, release profile, on the implementation base's `main`. If 12d has
merged by then, the "bodies" rows are measured on `main`; otherwise on a scratch merge of the 12d branch
with this PR's head, recorded as such, and the walking rows are the gate.

| ID | Criterion |
| --- | --- |
| ASRB-1 | 300-day save (`world.sqlite` + `-wal` after exit): ≤ **96 MiB** with bodies; ≤ **80 MiB** walking without bodies. Asserted in CI (`market_town.rs`, on whichever profile `main` runs). |
| ASRB-2 | Growth per simulated day, Market Town: ≤ **0.25 MiB/day** over days 100–300 (measured), for walking and for bodies; and (*S*300 − *S*30) ÷ 270 ≤ 0.25 MiB/day asserted in CI. |
| ASRB-3 | 30-day save ≤ **48 MiB** (replacing 128 MiB / 192 MiB), and its sealed spans exactly the rule's for its head, asserted in CI (`market_town.rs`). |
| ASRB-4 | Peak RSS on the 300-day walking save: `mineworld perceived … --person alice`, `inspect --last 0`, `biography … --person alice`, `replay` each ≤ **256 MiB** (today 1.31 / 1.49 / 1.32 GiB / 315 MiB, §4.5); output identical to the base build's on the same logical history (perceived lines, inspect lines other than the new `log` line and the format number, biography lines). CA-13 run once on the 300-day save: no `lagged`, p99 tick within CP-B4's 50 ms. |
| ASRB-5 | `kill_and_resume`: kills at the sealing commit, one before and one after, plus the existing points: survivor = control in logical journal and facts, snapshot rows and raw `sealed` rows; `verify` passes. |
| ASRB-6 | `AC-12` (`run.rs`, `market_town.rs`, headless): two runs of one seed equal in all four, including raw blocks. |
| ASRB-7 | `AC-8`: `ci_parity.py compare` passes for macOS arm64, Linux x86_64, Windows x86_64 on the PR head, including `table.sealed.*`. |
| ASRB-8 | A format-3 save (base-built fixture) is refused by the library; `run`, `replay`, `inspect`, `server`, `perceived` and `biography` refuse a format-3-marked save naming 3 and 4. |
| ASRB-9 | A block's facts frame replaced by (a) non-zstd bytes → `Damaged` naming the span, from `verify` and from `perceived`; (b) a valid frame of the same rows with one fact byte changed → `ReplayDiverged` at that fact's revision; resume of either save still succeeds (I-SRB-4). |
| ASRB-10 | Sweep at interval 2 to ≥ 3 000 revisions: after every commit the spans equal `{(kA, (k+1)A] : (k+2)A ≤ n}`, every revision 1 … *n* appears exactly once across blocks and tail, every fact once, and a resume at that head decodes no block (counted by a test backend wrapper). |
| ASRB-11 | The log is unchanged: on the 30- and 300-day saves, the SHA-256 of logical facts and journal rows (an unsealed copy queried with SR §14.1's `sqlite3 … | shasum -a 256` commands) equals the base build's; row counts equal; `scripts/baselines.txt` `summary-300s` unchanged for every world. |
| ASRB-12 | No file under `kernel/`, `contracts/`, `systems/`, `packages/` changes. |
| ASRB-13 | `mineworld replay` of the 300-day save reproduces every revision, fact and retained snapshot (the line names 603 633-scale counts) and its wall time is within 1.25 × the base build's on the same history. |
| ASRB-14 | The sealing commit's added wall time ≤ **100 ms** per block in release (measured over the 300-day run; max and mean recorded). |

**Adversarial (each must make a named criterion fail, run once, recorded):**

- never seal → ASRB-1, ASRB-3 fail;
- seal the span ending at *n* − *A* + 1 (one revision too far) → ASRB-10 fails and the guard trips;
- delete live rows without inserting the block (or skip the round-trip check and corrupt one byte in
  framing) → ASRB-11 and replay fail;
- `facts_after` decodes every block up front into one `Vec` → ASRB-4 fails;
- seal on a wall-clock condition (odd microseconds) → ASRB-5 or ASRB-6 fails;
- write blocks at level 3 instead of 9 → ASRB-11, ASRB-13 and replay still pass (a level never changes a
  verdict, I-SRB-2) and a save mixing both levels replays;
- a cursor without its read transaction (page queries outside one snapshot) → the concurrent-seal test
  fails.

## 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-SRB-1 | zstd level-9 output differs across platforms | same pinned bundled libzstd as snapshots; `AC-8` hashes `sealed`; fallback: level 3 (already proven under `AC-8`) is a constant change |
| R-SRB-2 | Sealing on the world thread stalls a hosted world | ≈ 20 ms per block every ≈ 1.5–2 simulated days; ASRB-14 bounds it; CA-13 measures p99; if it fails, seal on a second connection after the commit (a design revision, not a silent change) |
| R-SRB-3 | A long read transaction (a backfill) keeps the WAL from checkpointing | backfill ≈ seconds; the WAL is reset at the next checkpoint after it ends; recorded with CA-13 |
| R-SRB-4 | A raw-SQL reader of `facts`/`journal` silently sees only the tail | every raw reader is listed in §3 and changed at C6; `REQUIRED_TABLES` refuses a save without `sealed` |
| R-SRB-5 | Players lose format-3 saves | no release has shipped (S23 R-a/R-c open); QSRB-3 |
| R-SRB-6 | 12d lands with more facts per revision than measured | ASRB-1/2 are measured on `main` at C7; bounds have ≥ 30 % margin over the projection |
| R-SRB-7 | The server backfill's perceived subset still grows without bound | recorded, routed (§6.8, QSRB-6); FG-e waits for it |

## 10. Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its
evidence; `N/A` needs an audited reason. Commands assume the worktree root with `cargo` on `PATH`.
Targeted validation per commit; the PR's CI is the one full run.

### C0 — Freeze verification and handoff (Markdown only)

- **Goal.** Start the implementation context against the frozen design.
- **Scope.** This document (§14 opened); new `handoff-sr-b.md`.
- [x] Implementation: verify the `DESIGN FROZEN` header, §12's authority lines and sources, the
  assigned decision number; record the base commit; initialize the handoff with the contract's fields.
  (§14.2 C0; base 6306977; `handoff-sr-b.md`.)
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  (Both clean, §14.2.)
- [x] Review: every authority line has a source; no scope widened. (§12 lines each carry "freeze
  rulings", a working-rules default or "mvp0 convention"; ARC-89 pre-exists and is completed, not
  duplicated.)

### C1 — Decision record and spec edits

- **Goal.** The decision exists before the code (`CLAUDE.md` §2.2).
- **Scope.** §6.13's files. No code.
- [x] Implementation: `ARC-89` (assigned, QSRB-8); notes under `ARC-25` and `ARC-81`;
  `ARCHITECTURE.md`, `MODULE_SPEC.md` §8.1, `persistence/README.md`, step-06 L-6. (ARC-89 pre-existed
  from DOC-24: its `a − interval` corrected to `a − A`, its level made explicit (9), and a dated
  implementation note added with D-SRB-1 … D-SRB-11, §5's comparison, QSRB-4's bounds and §6.13's three
  revisit triggers. `persistence/src/lib.rs` and `sqlite.rs` module docs change with the code at C3/C4.)
- [x] Validation: both doc checks. ("223 numbered sections across 31 documents, none duplicated";
  "122 decision ids, all distinct".)
- [x] Review: `ARC-25`'s guarantees restated, not relaxed; `ARC-81` rule 1's amendment is the only one.
  (ARC-25 note repeats each guarantee — no row dropped, resume reads live rows, replay compares bytes;
  ARC-81 note amends rule 1 only, records rule 7's format number and the FK removal as consequences.)
- **Commit boundary.** Documentation only.

### C2 — The block codec (`block.rs`) and the sealing rule (pure)

- **Goal.** D-SRB-2, D-SRB-4, D-SRB-5 as pure, tested functions; nothing wired.
- **Scope.** New `persistence/src/block.rs` (`encode_block(span, journal_rows, fact_rows) ->
  SealedBlock`, `decode_journal`, `decode_facts`, `BLOCK_LEVEL = 9`); `world.rs` `sealing(n, interval,
  sealed_through)`; `lib.rs` module line.
- [x] Implementation. `persistence/src/block.rs` (`BLOCK_LEVEL = 9`, `JournalRow`, `SealedFact`,
  `BlockKeys`, `SealedBlock`, `encode_block`, `decode_journal`, `decode_facts`; the encoder runs the same
  checks as the decoder, so it refuses rows that do not fill the span); `RevisionSpan` in `backend.rs`
  (needed by the codec one commit earlier than planned — bounded); `world.rs` `sealing(n, interval,
  sealed_through)` (marked `expect(dead_code)` outside tests until C3 wires it).
- [x] Validation: `cargo test -p mineworld-persistence --lib` (block round trip incl. genesis-sized row,
  empty facts span; every D-SRB-4 decode check; the rule's table: first seal at 2·*A*, steady state, changed
  interval, never ≥ *n* − *A* + 1); clippy; fmt. (6 passed — 4 block tests: round trip with keys and
  compression; a 300 000-byte row and a span without facts; the encoder's refusals; 12 decoder damage
  cases each naming "the sealed block r1–r10": gap, fact outside the span, unordered ids, id columns, action
  column, trailing byte, truncated row, bad tag, not zstd, flipped byte, cut checksum — and the rule's
  11-case table. `cargo clippy -p mineworld-persistence --all-targets -- -D warnings` clean after one
  `type_complexity` fix; `cargo fmt --all` clean.)
- [x] Review: the codec never decodes an entry or a fact; the rule reads only its three arguments.
  (`block.rs` imports no contract type and calls no `serde`; `sealing` reads `n`, `interval`,
  `sealed_through` and the constant `ANCHOR_EVERY`.)
- **Failure cases.** Truncated / extended / reordered frames → `Damaged` naming the span, never a panic.

### C3 — Format 4: the `sealed` table, sealing in the commit, readers over both storages

- **Goal.** Saves seal; every existing trait method returns the same rows as before (still `Vec`-shaped
  in this commit), so the whole suite stays green on unchanged consumers.
- **Scope.** `format.rs` (`SAVE_FORMAT = 4`, history); `sqlite.rs` (schema D-SRB-3, FK change, seal step
  D-SRB-11 in `write_revision`, `head`, `highest_action_id`, `facts_of`, `last_facts`, `journal_after`
  over blocks + tail); `backend.rs` (`RevisionSpan`, `RevisionRow::seal`); `world.rs` (`sealed_through`,
  `seal` filled on anchor commits, guard); test doubles and `RevisionRow` literals (`save.rs`,
  `tools/cli/tests/inspect.rs`); format-3 fixture from the base build + refusal tests (library and CLI).
- [x] Implementation. `format.rs` (`SAVE_FORMAT = 4`, history line); `sqlite.rs` (schema with `sealed`, the
  `snapshots → journal` FK dropped; `seal` step after retirement in `write_revision`: read the span's live
  rows, `encode_block`, decode and compare, insert, delete facts then journal, compare counts; `head` and
  `highest_action_id` over both tables; `facts_of` decodes the one block holding the revision;
  `last_facts` reads the tail newest-first and decodes blocks backwards only for what it lacks;
  `journal_after` over blocks then tail (removed at C4); multi-query reads in one read transaction unless
  already inside one; a `blocks_decoded()` counter); `backend.rs` (`RevisionRow::seal`, `SealedSpan`, trait
  method `sealed()`); `world.rs` (`sealed_through`, read at resume from `sealed()`; `schedule_storage`
  fills `retire` and `seal` with both guards); `save.rs` (`Failing::sealed`, formats 4/5 and 1–3, both
  fixtures refused); `tools/cli/tests/inspect.rs` (`seal: None`); `save_retention.rs` (format 3 marked;
  `perceived` and `biography` added to the commands); `.gitignore` admits the format-3 fixture.
- [x] Validation: `cargo test -p mineworld-persistence` → lib 6, kill_and_resume cafe + clock PASS,
  retention 4, save 11, all passed (the retention sweep at interval 8 now seals 22 blocks and verifies
  from genesis through them; `verify_from` at interval 2 reads anchors 128 and 256 out of blocks).
  `cargo clippy --workspace --all-targets -- -D warnings` clean. Mutation "seal one revision too far"
  (`last = n − A + 1`): RED — unit table fails; `retention.rs` sweep and verify_from panic "the rule sealed
  r1–r513 at r1024" / "r1–r129 at r256" (the guard); reverted. Mutation "never seal": deferred to C6/C7,
  where the size criteria it must turn red (ASRB-1, ASRB-3) first exist. CLI targets and
  `mineworld-acceptance`: deferred to C4 — see finding F-SRB-1.
- [x] Review: no path deletes a log row except the seal step; the seal runs after insert and retirement
  in the same transaction; deleted counts checked. (`grep -n "DELETE FROM" persistence/src/sqlite.rs`: the
  snapshot retirement and the two deletes inside `seal`, both `BETWEEN` the span, after the `sealed` insert,
  their counts compared.)

**Finding F-SRB-1 (C3, bounded).** *Previous assumption:* C3 keeps every consumer green with
`Vec`-shaped methods, so the CLI targets can be validated at C3. *Evidence:* `cargo test -p mineworld-cli
--test run` ran a `mineworld replay` of a social-cafe save for > 15 min CPU before it was stopped:
`replay::reproduce` calls `facts_of(revision)` once per revision, and over sealed storage each call decodes
the whole 4 096-revision block — the quadratic cost §3's audit row for `replay.rs` predicted.
*Consequence:* the CLI and acceptance validation listed for C3 moves to C4, where replay reads the cursor
(D-SRB-7). No design change.

### C4 — Streaming cursors and replay on them

- **Goal.** D-SRB-6, D-SRB-7, I-SRB-5.
- **Scope.** `backend.rs` (`StoredRevision`, `revisions_after`, `facts_after`; `journal_after` removed);
  `sqlite.rs` (cursor structs with a deferred read transaction, block-at-a-time, paged tail);
  `replay.rs` (`replay_after`, `reproduce`, `reexecute` on the cursor); test call sites of
  `journal_after` (`headless/mod.rs` `Tables::read` + raw `sealed` rows in `assert_same_history`,
  `configuration_seam.rs`, `arrival_resolvers_resume.rs`, `tools/cli/tests/inspect.rs`); concurrent-seal
  test.
- [x] Implementation. `backend.rs` (`StoredRevision`, type aliases `Revisions<'_>` / `Facts<'_>` for the
  design's boxed iterators, trait methods `revisions_after`, `facts_after`; `journal_after` removed); new
  `persistence/src/sqlite/cursor.rs` (a submodule of the one SQL file, keeping `sqlite.rs` under 800 lines:
  `RevisionCursor`, `FactCursor`, one read transaction each via `SqliteBackend::reading`, the block found by
  key (`last_revision >= next`, `first_revision > last AND last_event_id > floor`), one decoded block at a
  time, the tail paged 1 024 rows per query, a skip or a block that goes back reported as `Damaged`, the
  cursor ending after its first error); `replay.rs` (`replay_after` iterates `revisions_after`; `reproduce`
  takes the `StoredRevision` — no `facts_of`); call sites forced by the removal, converted one commit early
  (bounded): `run.rs` `genesis_instant`, `head_input`; `inspect.rs` `Journal::read`; test helpers
  `headless/mod.rs` `Tables::read` (cursors + raw `sealed` rows, compared in `assert_same_history`),
  `arrival_resolvers_resume.rs`, `configuration_seam.rs`; `save.rs` `Failing`; the concurrent-seal test
  `a_cursor_reads_one_history_while_another_connection_seals_the_rows_ahead_of_it` (interval 16: the
  reader opens at r2 040, the writer seals 1–1 024 and 1 025–2 048 under it).
- [x] Validation: `cargo test -p mineworld-persistence`; `cargo test -p mineworld-acceptance`;
  `cargo test -p mineworld-cli --test run --test inspect --test save_retention` (the targets that use
  `Tables::read` and the cursors; the rest run in the PR's CI); mutation "cursor without read
  transaction" red on the concurrent-seal test. (Persistence: lib 6, cafe + clock PASS, retention 4 (2.73 s
  at C3 → 0.57 s), save 12 — all passed. CLI: save_retention 1 + 1 ignored, inspect 3, run 4 (102.8 s);
  acceptance: every target passed (15 result lines, exit 0; log `/tmp/impl-sr-b-target/c4-tests.log`).
  ASRB-8's six-command refusal passed inside `save_retention`. `cargo clippy --workspace --all-targets
  -- -D warnings` clean. Mutation: `Reading::open` with no transaction → RED, "the cursor reads on:
  Damaged { detail: "the log skips from r1024 to r2049" }"; reverted, green.)
- [x] Review: no per-revision `facts_of` left in replay; one comparison path for resume, verify and
  `verify_from`. (`grep facts_of persistence/src/replay.rs` → none; `resume` and `reexecute` both call
  `replay_after`.)

**Finding F-SRB-2 (C6, bounded; fixed in its own commit).** *Previous assumption:* holding one decoded
block at a time bounds a reader's memory. *Evidence:* release head build, 300-day walking save:
`perceived --person alice` peaked at 371 785 728 B and `biography` at 376 291 328 B — over ASRB-4's
256 MiB — while `inspect --last 0` peaked at 74 MB and the cursor alone (a scratch tool streaming every
revision) at 43 MB. `heap` on a running `perceived` showed 6.9 MB of live allocations against a 139.6 MB
footprint; `vmmap --summary` 2.2 s in showed `MALLOC_LARGE (empty)` 508.4 MB virtual / 290.8 MB in the
footprint: freed large buffers the allocator keeps. `zstd::stream::decode_all` grows each block's output
(≈ 5–7 MB) by doubling, so every block leaves freed regions of many sizes. *Decision:* decompress into
one allocation of exactly the frame's declared content size (`zstd::bulk::decompress` with
`zstd_safe::get_frame_content_size`, both already part of `DEP-43`'s crate; a header claiming more than
1 GiB is refused as damage; checksum still verified; a size mismatch is damage). *Result:* `perceived`
peak 371.8 MB → 28.6 MB on the same save (C7 records the four readers). No design change: D-SRB-5's codec
and D-SRB-6's one-block-at-a-time hold; only the decoder's allocation pattern changed.

*Correction to C3's evidence:* "the retention sweep … seals 22 blocks" was a miscount written without a
measurement; the rule gives 23 blocks for head 12 293 at interval 8 (anchors 1 024 … 12 288). C6's sweep
asserts the count against the oracle.

### C5 — Production readers on the stream (`F-S24-1`)

- **Goal.** D-SRB-8, D-SRB-10.
- **Scope.** `tools/cli/src/history.rs`, `perceived.rs`, `inspect.rs` (+ `log` line), `biography.rs`,
  `run.rs` (`Fingerprint::of_save`, `genesis_instant`); `tools/cli/tests/perceived.rs`, `inspect.rs`,
  biography and lakeside history tests as touched.
- [x] Implementation. `history.rs`: `saved_facts` replaced by `with_saved_facts(save, read)` — a
  `Decoded` adapter over `facts_after(None)` that decodes one envelope at a time, ends at the first error
  and returns it after `read` (D-SRB-8's adapter); `SavedHistory::perceived` = `perceived_by(facts
  .take_while(id ≤ through), …)` (the raw stream stops at `through`, so a backfill reads no later block;
  same output since later facts are never returned); `perceived.rs` checks the save, then prints as the
  stream goes; `inspect.rs` streams the journal once (kinds, actions, sealed row bytes) and the facts once
  (type counts, `Causes::add` with the seen-id set, a ring of the newest `--last`), and prints the `log`
  line; `biography.rs` one pass (selected entries kept; names through `names_in` one fact at a time, so it
  still names no event type); `run.rs` `Fingerprint::of_save` streams. `tools/cli/tests/perceived.rs`
  CA-6's per-revision `facts_of` loop (quadratic over blocks) moved to the revision cursor.
- [x] Validation: `cargo test -p mineworld-cli --test perceived --test inspect --test run --test
  milestone_e`; outputs compared with the base build on a 30-day save (identical but for the `log` line
  and the format number); mutation "collect all then filter" shows ASRB-4's measurement fail at C7.
  (Tests, with `--test biography` added: biography 2, inspect 3, milestone_e 7, perceived 3 + 1 ignored
  (re-run after the CA-6 edit: 3 passed), run 4 — all passed, log `c5-tests.log`. Outputs, release
  builds of base 6306977 (`scratch-srb-base` worktree, own target dir) and head, Market Town seed 7,
  30 days: `run`'s printed lines identical but for the header path and `wall` (incl. `history 82298
  facts, fingerprint 346dcaa6291381a0`); `perceived --person alice` (34 698 lines), `perceived --json
  --since 5000` (32 665 lines) and `biography --person alice` (736 lines) byte-identical (`cmp`);
  `inspect --last 50` differs only in the format number, the random instance and the added line
  `log        13 block(s) sealed r1–r53248 (3.6 MB stored, 90.1 MB of rows); live r53249–r60331`. The
  "collect all then filter" mutation is run at C7 with the memory measurement.)
- [x] Review: no `last_facts(i64::MAX)` or whole-journal read remains in `tools/cli/src` or `server/src`
  (`grep`); error from the stream surfaces, never a truncated success. (`grep -rn "last_facts\|
  journal_after" tools/cli/src server/src` → none; `last_facts(64)` stays in `PersistentWorld::
  recent_facts`, live tail first. Every streaming reader returns the stream's error: `with_saved_facts`
  checks the adapter's error after `read`; inspect, biography and the fingerprint `?` each row.)

### C6 — Kill-and-resume, sweeps, raw-SQL tests, CI assertions, parity and nightly

- **Goal.** ASRB-3, ASRB-5, ASRB-9, ASRB-10 in tests; CI and nightly wiring.
- **Scope.** `kill_and_resume.rs` (intervals 2 and 4 so both runs seal; kill points at/around the
  sealing commit; raw compare of `sealed`); `retention.rs` (sweep oracle, resume-decodes-no-block counter);
  `save.rs` (block tamper; raw SQL fixed); `save_retention.rs`; `market_town.rs` (ASRB-1/2/3 assertions);
  `scripts/ci_parity.py` (`REQUIRED_TABLES`, self-test row); `scripts/baselines.txt` (re-record
  `facts-300s`, `journal-300s`, add `sealed-300s`, reason line; `summary-300s` unchanged);
  `scripts/ci_stability.py` (`AC-SAVE-1`, self-test).
- [x] Implementation. `kill_and_resume.rs`: intervals cafe 2 / clock 4 (anchors 128 / 256, first seal at
  256 / 512), kill points "before sealing" (2A − 1), "sealing" (2A), "after sealing" (2A + 1) added,
  `Rows` gains every column of `sealed`, the control's blocks checked against the rule computed in the
  test, facts counted through the stream. `retention.rs`: the ASRB-10 sealing sweep (interval 2 to
  r3 000; blocks = rule after every commit; the live journal by raw SQL is exactly the revisions after the
  blocks; every revision and every fact once through both cursors at and after every anchor; a resume at
  every head through a `Shared(Rc<SqliteBackend>)` wrapper with `blocks_decoded()` unchanged).
  `save.rs`: ASRB-9 (a) non-zstd facts frame → `Damaged "the sealed block r1–r1024 does not decompress"`
  from `verify` and from the fact stream, resume succeeds; (b) a re-sealed valid frame with one fact byte
  changed → `ReplayDiverged` at that fact's revision, resume succeeds. `market_town.rs`: ASRB-3
  (≤ 48 MiB + blocks = rule), ASRB-1 (≤ 80 MiB walking), ASRB-2 ((S300 − S30) ÷ 270 ≤ 0.25 MiB/day).
  `ci_parity.py`: `REQUIRED_TABLES` + `sealed`, `BASELINE_KEYS` + `sealed-300s`, self-test cases (a
  differing sealed chunk; a record without `table.sealed.rows`), and `baseline write` reads the old file's
  reason lines without requiring keys added since (bounded tooling fix: adding a key otherwise made
  `write` refuse its own input). `ci_stability.py`: `AC-SAVE-1` (market-town to day 100, measured, on to
  300; `judge_growth` ≤ 0.5 MiB/day; sizes and rate in the timings; three self-test cases).
  `scripts/baselines.txt`: re-recorded once from a long record of the head build, with a reason line per
  world (see F-SRB-3).
- [x] Validation: `cargo test -p mineworld-persistence`; `cargo test -p mineworld-cli --test
  market_town` (bound mutated to 1 MiB once → red with the measured value); `python3 scripts/ci_parity.py
  --self-test`; `python3 scripts/ci_stability.py --self-test`. (Persistence: lib 6; kill_and_resume cafe
  (301 revisions, 1 block) and clock (634, 1 block) PASS at all eight kill points — e.g. clock "sealing:
  kill sent at revision 512 … 513 committed; survivor restored snapshot 512"; retention 5; save 13.
  market_town: PASS in 134.4 s — "ASRB-3: the 30-day save is 20324352 B (19.4 MiB), 17 snapshots
  (882181 B), 13 sealed blocks, head 60331"; "ASRB-1: the 300-day save is 63086592 B (60.2 MiB); ASRB-2:
  it grew 0.151 MiB per simulated day from day 30"; AC-12 incl. raw `sealed` rows held for twin and
  killed-and-re-run. Mutation BOUND = 1 MiB → RED "ASRB-3: the 30-day save is 20324352 B (19.4 MiB),
  over the 1 MiB bound"; reverted. Mutation never seal (`row.seal = … .filter(|_| false)`) → RED in
  kill_and_resume ("the control holds exactly the sealing rule's blocks"), the sealing sweep, the
  concurrent-seal and block-tamper tests; with a release build of that mutation the 30-day save is
  117 776 384 B (112.3 MiB) > 48 MiB, ASRB-3's bound; reverted. Both self-tests "passed" (python3.12;
  the system python3 3.9 lacks `tomllib`). `baseline check` of the long record against the new file →
  "baselines PASS". `cargo clippy --workspace --all-targets -- -D warnings` clean after one
  `suspicious_map` fix.)
- [x] Review: every raw reader in §3 handled; the baselines' reason line states storage, not behaviour.
  (Raw SQL over `facts`/`journal` remaining: `save.rs` tamper of the newest fact (40 steps, nothing
  sealed), `retention.rs` `live_journal` (deliberately the live table), `kill_and_resume.rs` `rows`
  (live + sealed, complete); `tools/cli/tests/perceived.rs` CA-6 moved to the cursor at C5; tests that
  read `last_facts(all)` (third_party, weather, lakeside history) still get every fact.)

**Finding F-SRB-3 (C6, baselines).** *Previous assumption (§6.10):* only `facts-300s`/`journal-300s` change
and `summary-300s` must not. *Evidence:* the head's long record (`ci_parity.py record --profile long`,
2 104.9 s) differs from the committed file in `summary-300`, `summary-1000` and `summary-300s` of every
world as well. The committed file was stale on main already: nightly issue #170 (2026-10-14) shows main's
own record with exactly these 12 summary values (compared one by one: 12/12 equal), drift owned by the
primary as F-12d-1; and the base build 6306977 prints byte-identical lines to the head for social-cafe
300 days (memory) and market-town 300 days (saved). So SR-b changes no summary. *Decision:* re-record the
whole file once (QSRB-5's one re-recording) with a reason line that says both things — storage keys
changed by SR-b, summaries re-recorded as main already had them — so the nightly's baseline check passes
on this head. If 12d merges first, the second to merge re-records again (QSRB-7).

### C7 — Integration checkpoint: real runs, measurements, close-out

- **Goal.** ASRB-1 … ASRB-14 on the real binary; READY FOR OPERATOR REVIEW.
- **Scope.** §14 evidence; handoff; requests to the primary session (§11).
- [ ] Implementation: N/A beyond the ledger unless a criterion fails.
- [ ] Validation: release 30/100/300-day saves (walking; bodies per §8's rule); sizes and `dbstat`;
  `inspect`; `replay` with wall time vs base; `verify_from` first/middle/last anchor; `/usr/bin/time -l`
  for the four readers; CA-13 once; ASRB-11's unsealed-copy SHA-256 (a scratch tool under `target/`, never
  committed) vs the base build; seal timing; the adversarial runs; the PR's CI incl. `AC-8`.
- [ ] Review: every number in §14 comes from a command recorded beside it; scratch saves deleted.

## 11. Requirements this PR places on other documents and lanes

- **R-overall.** The primary session records `ARC-89` (assigned, QSRB-8) in `overall.md`'s table (next
  free then `ARC-90`) and, after merge, marks
  `F-SAVE-1` resolved for the log (FG-e open) and records the 30-day bound change (ASRB-3) beside the
  12d ruling it replaces.
- **R-step-24.** FG-d's row and §8.1 item 2 point at this document; `F-S24-1` is marked "persistence and
  CLI side closed by SR-b; server seam open".
- **R-server (F-S24-1 remainder).** `PerceivedHistory::perceived` and `read_backfill` stream frames
  instead of materialising the perceived subset (QSRB-6). FG-e depends on it.
- **R-12d.** Whichever of 12d and SR-b merges second re-runs `market_town.rs` and re-records
  `scripts/baselines.txt` (both change it); SR-b's ASRB-3 bound replaces 12d's 192 MiB.
- **R-S23.** SR-b lands before S23's first release, so no released save is format 3 (QSRB-3).

## 12. Execution contract (filled at freeze, 2026-10-12)

Source of every line marked "freeze rulings": the primary session's SR-b rulings of 2026-10-12, which
carry the operator's QSRB-3 and QSRB-4, relayed by the coordinator to this planning session ("record
rulings … fill the execution contract, note 'implementation starts only on the operator's go' … Do not
merge"). Endpoints the rulings do not name individually (commits, push, PR, CI repair) take the working
rules' shipped defaults (§§14, 21) once the operator gives the go. Lines marked "mvp0 convention" follow
SR's contract (`pr-s6-save-retention.md` §12) and were not separately ruled.

```text
PROJECT / PR:            MineWorld mvp0, step-24 FG-d = SR-b — sealed-block compression of the world
                         event log
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-sr-b-log-compression.md
RELATED / BINDING DOCS:  pr-s6-save-retention.md (§6.9, ARC-81); mvp1/step-24-decision-and-memory.md
                         (§2.5 F-S24-1, §8.1, §10, §11 FG-d); step-06-persistence.md; docs/DECISIONS.md
                         ARC-25, ARC-27, ARC-49, ARC-55, ARC-81, DEP-2, DEP-5, DEP-43; docs/ARCHITECTURE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; overall.md; CLAUDE.md
WORKTREE:                its own, held by one session (CLAUDE.md §3.1):
                         /Users/yuema137/mineworld-worktrees/impl-sr-b, created from main at the go
                                                         (mvp0 convention; SR's impl-save-retention)
BRANCH:                  mvp0/pr-sr-b, created from main        (coordinator's brief, 2026-10-12)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records it
START:                   only on the operator's go, in a fresh implementation session (freeze rulings)
APPROVED SCOPE:          §2.1, as frozen                                            (freeze rulings)
FROZEN INVARIANTS:       §2.3; D-SRB-1 … D-SRB-11; §13.1's rulings: anchor lattice (QSRB-1); BLOCK_LEVEL 9
                         (QSRB-2); format 3 refused (QSRB-3); the bounds of ASRB-1 … ASRB-4 and ASRB-14 as
                         ruled — 30-day ≤ 48 MiB, 300-day ≤ 96 MiB with bodies and ≤ 80 MiB walking, growth
                         ≤ 0.25 MiB/day, nightly AC-SAVE-1 at 0.5 MiB/day until stable, reader peak ≤ 256 MiB,
                         seal ≤ 100 ms per block (QSRB-4); baselines re-recorded once with a reason line
                         (QSRB-5); no change to the server's PerceivedHistory seam (QSRB-6); ARC-89 (QSRB-8)
                                                                                    (freeze rulings)
SEQUENCE:                C0 … C7
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *;
                         sqlite3 (read-only queries on scratch saves); zstd (scratch measurement);
                         /usr/bin/time; mkdir -p; sed -n                          (mvp0 convention)
NEVER:                   alter a fact or journal row's bytes; delete a log row outside the seal step; VACUUM
                         or rewrite a user's save; python3 -c; sed -i; curl; heredoc writes; leave a scratch
                         save outside the test scratch helper (DEP-29) or target/; add a dependency
                                                                                   (mvp0 convention)
MATERIAL STOPS:          any kernel/contracts/systems/packages change; a new dependency; a change to the
                         span, the level or a bound after it has been measured once; ASRB-7 failing on any
                         leg; ASRB-14 or CA-13 failing (R-SRB-2); any change to the server's PerceivedHistory
                         seam (QSRB-6); FG-e work
PLATFORMS:               macOS, Linux, Windows (ASRB-7)
VALIDATION BUDGET:       unit, static and local integration, incl. 30/100/300-day release runs and one
                         12d scratch merge: unrestricted. CI: the PR's runs; no manual dispatch
                                                                                   (mvp0 convention)
LIVE DOCUMENTATION:      this document (§14)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-sr-b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized on the operator's go, in a fresh
                         session                                     (freeze rulings, 2026-10-12)
                         semantic commits: authorized after the go   (working rules §14 default)
                         branch push: authorized after the go        (working rules §21 default)
                         PR creation / update: authorized after the go
                                                                     (working rules §21 default)
                         CI repair to review readiness: authorized after the go
                                                                     (working rules §21 default)
                         merge: never by the implementation session; only after the primary session's
                         review and with explicit operator authorization
                                                     (freeze rulings "Do not merge"; working rules §22)
POST-MERGE SYNC OWNER:   the primary session owns overall.md and step-24; the implementation session owns
                         this document's ledger, evidence, deviations, remaining issues, the step-06 L-6
                         note                                                       (mvp0 convention)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 13. Questions

**[operator]** marks a question only the operator can answer; **[primary]** one the primary session
decides.

### 13.1 Rulings, 2026-10-12

Relayed by the coordinator to the SR-b planning session on 2026-10-12.

| ID | Ruled by | Ruling | Where it lands |
| --- | --- | --- | --- |
| QSRB-1 | primary | **Anchor lattice**: blocks on `ARC-81`'s lattice, sealed one span late. | D-SRB-2, §6.2; §12 FROZEN INVARIANTS |
| QSRB-2 | primary | **zstd level 9** for blocks. | D-SRB-5, §6.3; §12 |
| QSRB-3 | operator | **Refuse** format-3 saves by name; no upgrade command. | D-SRB-9, §6.9; ASRB-8; §11 R-S23 |
| QSRB-4 | operator | **Bounds accepted**, Market Town with bodies: 30-day ≤ 48 MiB; 300-day ≤ 96 MiB (≤ 80 MiB walking); growth ≤ 0.25 MiB/day; the nightly `AC-SAVE-1` asserts 0.5 MiB/day until stable; reader peak RSS ≤ 256 MiB; seal ≤ 100 ms per block. | ASRB-1 … ASRB-4, ASRB-14; §6.11; §12 |
| QSRB-5 | primary | **Re-record once** `facts-300s` / `journal-300s` (+ `sealed-300s`) with a reason line. | §6.10; C6 |
| QSRB-6 | primary | **Route** the backfill interface change to the server lane, recorded as the remainder of `F-S24-1`. | §2.2, §6.8; §11 R-server; §12 MATERIAL STOPS |
| QSRB-7 | primary | **Parallel** with 12d; the second to merge re-runs `market_town.rs` and re-records the baselines. | §11 R-12d |
| QSRB-8 | primary | **`ARC-89`**. | header; C1; §6.13; §11 R-overall |

### 13.2 The questions as asked (kept for review)

| ID | Question | Recommendation |
| --- | --- | --- |
| QSRB-1 [primary] | Block boundaries on `ARC-81`'s anchor lattice (*A* = 64 × interval, sealed one span late), rather than a separate fixed 4 096 with a test-only knob (B7)? In production both give the multiples of 4 096. | **Anchor lattice** (D-SRB-2, §6.2): resume never reads a block, FG-e segments start at anchors, tests need no new knob. |
| QSRB-2 [primary] | zstd level for blocks: 9 (≈ 194.6 KB per walking block, ≈ 16 ms) or 3 (≈ 311 KB, < 10 ms, the snapshot level)? | **9**: 37 % smaller; the margin under 0.25 MiB/day with bodies depends on it (§4.6). |
| QSRB-3 [operator] | Format-3 saves: refuse by name (QSR-3's precedent), or ship a one-shot `mineworld save upgrade`? | **Refuse.** No release has shipped; land SR-b before S23's first release. |
| QSRB-4 [operator] | Bounds: 300-day ≤ 96 MiB (bodies) / ≤ 80 MiB (walking); growth ≤ 0.25 MiB/day for walking **and** bodies (tighter than QS24-4's 0.5 with walking); 30-day ≤ 48 MiB, replacing 12d's 192 MiB. And should the nightly `AC-SAVE-1` assert 0.25 too, or keep the step's 0.5? | **Yes** to the PR bounds (≥ 30 % margin over the projections). Nightly asserts **0.5** (the ruled step bound) and records the value; tighten after a month of nights. |
| QSRB-5 [primary] | `scripts/baselines.txt`: re-record `facts-300s`/`journal-300s` as stored-table digests (now the live tail) with a reason line, plus `sealed-300s` — or teach `ci_parity` logical digests (needs zstd in the standard-library-only script, or a new binary subcommand)? | **Re-record once** (§6.10). `summary-300s` (the fact fingerprint) stays the continuous logical evidence; ASRB-11 proves the journal once. |
| QSRB-6 [primary → server lane] | The server's backfill seam still returns the perceived subset as one `Vec` and renders every frame before sending. Change it here, or route it? | **Route** it as the remainder of `F-S24-1` (server lane, before FG-e). SR-b removes the whole-log load under the existing seam. |
| QSRB-7 [primary] | Placement against 12d (PR #171, open): SR-b after 12d, or in parallel with the second to merge re-running `market_town.rs` and the baselines? | **Parallel**, disjoint crates (`persistence/`, `tools/cli/`); R-12d names the reconciliation. |
| QSRB-8 [primary] | Decision number for `ARC-SRB`. | **`ARC-89`** (next free per `overall.md`), verified unused at freeze. |

## 14. Ledger (live during implementation)

**Status: IN IMPLEMENTATION** (operator's go 2026-10-13, relayed by the coordinator). Opened by C0 in
the implementation session (`/Users/yuema137/mineworld-worktrees/impl-sr-b`, branch `mvp0/pr-sr-b`).

### 14.1 Planning-session record (2026-10-12)

```text
Audit base:          origin/main @ cda4c13
Measured:            §4 (walking main; 12d branch @ 4e4ce10 in a scratch detached worktree, removed);
                     scratch saves under target/srb-scratch, deleted after measuring
Not measured:        the framing overhead of D-SRB-4 (estimated ≈ 20–30 B per row before compression);
                     seal timing inside a commit (CLI timings only); bodies 30-day head (estimated)
```

### 14.2 Implementation record

```text
Implementation base:  origin/main @ 6306977 (Merge pull request #177), fetched 2026-10-13
Worktree / branch:    /Users/yuema137/mineworld-worktrees/impl-sr-b, mvp0/pr-sr-b (one session)
Handoff:              handoff-sr-b.md
Environment:          macOS 26.2 arm64 (operator's laptop), rustc per rust-toolchain.toml,
                      CARGO_TARGET_DIR=/tmp/impl-sr-b-target
```

**C0 — freeze verification (done).** Header `DESIGN FROZEN 2026-10-12`, revision 2; §12 filled with a
source on every endpoint line; decision number `ARC-89` (QSRB-8) — found already present in
`docs/DECISIONS.md` (added by DOC-24, commit 203095b, as the design-level record); C1 completes it rather
than adding a second record. `python3 scripts/check_doc_headings.py` → "223 numbered sections across 31
documents, none duplicated"; `python3 scripts/check_decision_ids.py` → "122 decision ids, all distinct".
No scope widened.

**Format-3 fixture (written at C0, before any code edit, committed at C3).** The worktree at the base
commit 6306977, with one uncommitted, ignored test (`persistence/tests/make_format_3.rs`, deleted after
use) that creates a save of the persistence test world (`support::assembled`, interval 8, 12 scripted
steps; head 20, snapshot rows at 1, 8, 16), written under `$CARGO_TARGET_DIR` and copied to
`persistence/tests/fixtures/format-3/world.sqlite` (40 960 B, SHA-256
`6c8242af66ac191725f9a6a866ef37a466554349cd7d9d6df63802db02fa2f17`; `format 3` in its manifest; every row
a format-3 build's own bytes). The scratch copy was deleted.
