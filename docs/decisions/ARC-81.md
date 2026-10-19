## ARC-81 — Save retention: snapshots pruned by a pure rule and stored compressed; the log kept whole

**Date** 2026-10-10 · **Amends** `ARC-25`'s accepted limitation ("compaction and snapshot pruning are
later work"), for snapshots only · **Relates to** `INV-11`, `ARC-23`, `ARC-25`, `ARC-27`, `ARC-49`,
`DEP-2`, `DEP-5`, `DEP-43`, [`MVP.md`](MVP.md) §9 `AC-8`, `AC-12` · **Approved by** the operator's
rulings on QSR-1, QSR-3, QSR-4 and the primary session's on QSR-2, QSR-5, QSR-6 (2026-10-10) ·
**Design** `.structured-coding/plans/mvp0/pr-s6-save-retention.md` (finding `F-SAVE-1`)

**Problem.** A save wrote a full world snapshot every 64 revisions and never dropped one. Measured at
`bb62edf`: a 30-day Market Town save is 357 MiB and a 300-day one 3.53 GiB, of which 80–84 % is
snapshots of ≈ 0.7 MB each. Snapshot size had plateaued; the growth was purely their count.
Snapshots are checkpoints, never an authority (`ARC-25`), so almost all of them carry no information
the journal cannot regenerate.

**Options considered** (design §5; sizes for the 300-day save).

```text
5.1  keep genesis + an anchor every 4 096 revisions + the newest 2     ≈ 636 MiB   adopted
5.2  zstd-compressed snapshot rows                                    ≈ 809 MiB   adopted
5.1 + 5.2                                                             ≈ 587 MiB   the choice
5.3  delta snapshots (zstd --patch-from a base)                       chained restore, one damaged
                                                                      delta damages every later one
5.4  SQLite VACUUM / auto_vacuum                                      no effect alone; nothing deleted
5.5  page compression: ZIPVFS, sqlite-zstd                            proprietary; LGPL-3.0, layout
                                                                      changed outside our transaction
5.6  content-addressed dedup of snapshot sections                     a second storage model for ≈ 3 MiB
5.7  drop journal and facts before a checkpoint                       contradicts ARC-25 and INV-11
5.8  compress the log in sealed blocks (SR-b)                         ≈ 40 MiB; changes where the
                                                                      authority's bytes live: its own PR
```

**Choice.**

1. **Only snapshot rows are ever deleted** (D-SR-1). No row of `facts` or `journal` is deleted,
   updated or re-encoded; their bytes are exactly what a format-2 build wrote for the same run.
2. **Anchors** (D-SR-2). Scheduled snapshots stay every `interval` revisions (default 64). An anchor is
   a scheduled snapshot at a multiple of `64 · interval` (default 4 096). Genesis (revision 1) is
   always kept.
3. **The retention rule** (D-SR-3). When the scheduled snapshot at revision *n* is committed, the
   retained set is `K(n) = {1} ∪ {a ≤ n : a mod (64·interval) = 0} ∪ {n, n − interval}`; every stored
   snapshot *s < n* outside `K(n)` is deleted in the same transaction. An off-lattice checkpoint (a
   clean shutdown's) is kept until the next scheduled snapshot and then falls under the rule. The rule
   reads *n*, the interval and the set of stored revisions, nothing else, so a killed-and-resumed
   headless run holds the same rows as an uninterrupted one (`ARC-27`).
4. **The codec** (D-SR-4, D-SR-5). A stored snapshot is a zstd frame — level 3, checksum on, content
   size in the header, single-threaded, no dictionary — of the exact bytes `serde_json` gives the
   `WorldSnapshot`. It lives in `persistence/src/format.rs` (`encode_snapshot`, `decode_snapshot`);
   the backend stays opaque (`DEP-2`). Journal entries and facts are not compressed.
5. **Comparison is of decoded bytes** (D-SR-6). `verify` decompresses each stored snapshot and compares
   its JSON bytes with the state the history produces. `verify_from(backend, composed, anchor)`
   restores a retained snapshot and re-executes to the head with the same checks.
6. **Where pruning runs** (D-SR-7). `PersistentWorld` tracks the stored snapshot revisions, computes
   the revisions to retire and hands them to the backend in `RevisionRow::retire`; the SQLite backend
   deletes them inside the revision's transaction, after inserting the new snapshot. A failed delete
   fails the whole revision.
7. **Format** (D-SR-8). `SAVE_FORMAT` is 3. A format-2 save is refused by name
   (`SaveFormatOutdated { saved: 2, supported: 3 }`); no migration (QSR-3: no release has shipped).
8. **Visible** (D-SR-9). `mineworld inspect` prints the retained snapshot revisions and their stored
   sizes (`ARC-23`).

**Why it does not weaken `ARC-25`.** A restart still restores the newest snapshot at or below the head,
which the rule never deletes (`n` and `n − interval` are always kept). `verify` still re-executes every
revision from genesis and compares every fact and every snapshot the save holds. Equality of state is
argued only from decoded JSON, so no codec setting can change a replay verdict. Free pages left by a
delete are reused by the next insert; `VACUUM` is never run on a user's save.

**Accepted limitations.** The log alone still grows ≈ 1.9 MiB per simulated day in Market Town; SR-b
(log compression, design §6.9) is its remedy, approved as a separate PR (QSR-1). Retention is a constant,
not a player or World Pack setting (QSR-4). A hosted world may hold one extra off-lattice checkpoint
between scheduled snapshots, which `AC-12` does not cover (`ARC-27`).

**Revisit** if a feature needs denser history (a "rewind to any hour" would revisit 5.3 / 5.6 and the
anchor spacing), if `AC-8` parity ever differs in the `snapshots` table (see `DEP-43`), or when SR-b is
designed.

**Note, 2026-10-10 (S15, PR 12n-2; step-11 §21.15 M-6, the operator's ruling) — ASR-2 for walking
towns.** Since people walk (`ARC-75`), a walk's strides are requests and facts of their own. Market Town
records about 2.2 × the facts, and its 30-day save is 112.3 MiB, against 59.2 MiB before. ASR-2's bound
becomes ≤ 128 MiB for walking towns; the retention rule and the snapshots it keeps are unchanged. The
300-day size is printed by `tools/cli/tests/market_town.rs` as a measurement, not a bound. SR-b (log
compression) is the expected remedy for the size, as above.

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 2, the operator's ruling) — ASR-2 for towns
with bodies.** With `bodies` installed in the towns (`ARC-39` note 7), every walk also meets walls,
furniture and people: stopped-short, displaced arrivals and shoves join the log, and Market Town's
30-day save measured 170 811 392 B (162.9 MiB), about 45 % over the walking town's 112.3 MiB. ASR-2's
bound becomes **≤ 192 MiB for towns with bodies** (the 30-day market-town save); the retention rule and
the snapshots it keeps are unchanged. SR-b's log compression is still expected to bring the size back
down.

**Note, 2026-10-13 (SR-b, `ARC-89`) — rule 1 amended.** Rule 1 now reads: no row of `facts` or `journal`
is updated or re-encoded, and none is deleted except when it is **moved unchanged into a sealed block**,
in the transaction of the anchor commit that seals its span, after the block has been decoded and
compared with the rows it replaces. This is the only amendment: rules 2 – 8 (anchors, the retention rule,
the snapshot codec, comparison of decoded bytes, where pruning runs, visibility) stand, except that
`SAVE_FORMAT` is now 4 (rule 7) and a format-3 save is refused by name like a format-2 one. The snapshot
table no longer references the journal, since a retained genesis or anchor snapshot outlives its sealed
journal row. ASR-2's 30-day bound is replaced by `ARC-89`'s tighter ones (≤ 48 MiB at 30 days).
