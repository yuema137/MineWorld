## ARC-89 — Sealed log blocks: journal and fact rows of completed spans move unchanged into compressed blocks, and history is read as a stream

**Date** 2026-10-12 · **Status** accepted (design, step-24 FG-d; operator QSR-1 of 2026-10-10 and QS24-5 of 2026-10-12, building SR-b in MVP-0); implemented by FG-d (SR-b), `SAVE_FORMAT` 4 · **Approved by** the operator (QSR-1, QS24-5); the concrete design is `.structured-coding/plans/mvp0/pr-sr-b-log-compression.md` (design PR #178; frozen 2026-10-12) · **Relates to** `ARC-25`, `ARC-27`, `ARC-49`, `ARC-81` (amended, rule 1), `DEP-2`, `DEP-43`, `ARC-59`, `AC-9`, `AC-12`, `QS24-4` (save growth bound) · **Design** `pr-sr-b-log-compression.md` §§6.1, 6.2 (D-SRB-1 … D-SRB-11)

**Problem.** After `ARC-81`'s snapshot pruning, a Market Town save is almost entirely its event log. The log grows by about 3.75 MiB per simulated day on walking and 5.59 MiB per day with bodies, against a bound of 0.25 to 0.5 MiB per day (QS24-4). Facts and journal entries compress 18 to 40 times when compressed together in spans, but only 2 to 2.5 times one row at a time. Readers of history (`perceived`, `inspect`, `biography`) also load the whole log, peaking at 1.3 to 1.5 GiB on a 300-day save (`F-S24-1`). `ARC-25` requires that the log stay history, never derived and never rewritten, and that replay compare bytes.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Delete or rewrite old rows, or keep only the newest span | Declined: destroys history; violates `ARC-25` and `AC-9`, whose causes must resolve |
| (b) Compress each row alone | Declined: 2 to 2.5 times, far short of the bound |
| (c) Seal completed spans into blocks, moving rows unchanged; readers stream one block at a time | **Chosen** |
| (d) Archive old segments to separate files (FG-e) | Deferred to MVP-1 (FG-e), after streaming readers exist; it builds on this record |

**Choice.** Let `A = 64 · interval` be `ARC-81`'s anchor interval (4 096 revisions by default). At each anchor commit `a` of that lattice, with `a ≥ 2A`, the live rows of the revisions in (`sealed_through`, `a − A`] move, byte for byte, into one sealed block in the same transaction as the anchor. A block holds two zstd frames (journal and facts) over a length-prefixed framing of the rows, at one fixed level for blocks (level 9), as `DEP-43` requires of every stored kind. Rows keep their key columns and bytes; a row is either live or inside exactly one block, never both. The backend stores opaque blocks and never decodes a fact (`DEP-2`). Replay compares re-executed entries and facts with the logged bytes, decoded from a block. Every reader of history streams blocks. The save format becomes `SAVE_FORMAT` 4; an older save is refused by name, with no migration (QSR-3).

**Consequences.** `ARC-81` rule 1 ("`facts` and `journal` are only ever inserted into") is amended to "inserted into, and moved unchanged into a sealed block". No other row is deleted. Stored bytes remain a function of pack, seed, age, pace and code (`ARC-27`), so `ARC-49`'s parity hashes continue to hold on all three platforms. The per-day save bound is checked nightly (`AC-SAVE-1`), not by refusing facts.

**Revisit** if a block-level read cannot meet the streaming bound, if a sealed span must be rewritten for a correction (which this record forbids), or when FG-e's archive segments need a manifest format, which a successor record decides.

**Note 2026-10-13 (implementation, SR-b; design frozen 2026-10-12, rulings QSRB-1 … QSRB-8).** The record's `a − interval` in the first draft meant `a − A`, as stated above. The decisions the implementation holds to, in full in the design's §6.1:

```text
D-SRB-1   rows move, bytes do not change: a journal or fact row is live or in exactly one block, with the
          same key columns and bytes; ARC-81 rule 1 is amended to say so, and no other deletion of a log
          row exists
D-SRB-2   the sealing rule, a pure function sealing(n, interval, sealed_through) beside ARC-81's retired:
          when n mod A = 0 and n ≥ 2A, seal (sealed_through, n − A]; blocks are (kA, (k+1)A], sealed one
          span late, so resume never reads a block and every block ends at an anchor (QSRB-1)
D-SRB-3   one table: sealed(first_revision PK, last_revision, first_event_id, last_event_id,
          max_action_id, journal BLOB, facts BLOB); spans contiguous, the live tail after the newest
D-SRB-4   the framing (persistence/src/block.rs), little-endian: journal rows (revision, at, action id,
          length, entry) in revision order; fact rows (event id, revision, length, fact) in EventId order;
          decoding checks contiguity, spans, ordering and trailing bytes, and names the block's span
D-SRB-5   each frame is zstd level 9 (QSRB-2), checksum and content size on, single-threaded, no
          dictionary: block bytes are a function of the row bytes on every platform, which AC-8 checks
D-SRB-6   the backend's history is two cursors, revisions_after and facts_after, each inside one read
          transaction, holding one decoded block at a time and paging the live tail; journal_after is
          removed; facts_of, last_facts, head and highest_action_id read both storages
D-SRB-7   replay, verify, verify_from and resume iterate the revision cursor and compare the
          re-executed entry and facts with the cursor's bytes
D-SRB-8   every production whole-log reader streams: the server's backfill under its existing seam,
          perceived, inspect, biography, run's fingerprint and genesis instant (F-S24-1, persistence and
          CLI side; the server's seam itself is routed to the server lane, QSRB-6)
D-SRB-9   SAVE_FORMAT 4; snapshots no longer reference the journal; a format-3 save is refused by name,
          no migration (QSRB-3)
D-SRB-10  mineworld inspect prints one `log` line: blocks, their span, stored bytes and row bytes, the
          live tail
D-SRB-11  a seal is verified before the rows go: the block's frames are decoded and compared with the
          source rows and the deleted counts with the block's, or the commit fails (WorldAheadOfSave)
```

**Ways of building it compared** (design §5): blocks of `A` revisions in a `sealed` table on the anchor lattice (chosen); per-row zstd (2 – 2.5×, misses QS24-4 by 5×); per-row zstd with a trained dictionary (9 – 11×, plus a stored, versioned dictionary; a dictionary buys 2 % at block scale); blocks as files beside `world.sqlite` (two-store atomicity, deferred to FG-e); SQLite page compression (rejected in `ARC-55`); a columnar container (no column structure in opaque rows); a fixed 4 096 independent of the interval (a second lattice and a test-only knob, QSRB-1).

**Bounds** (operator QSRB-4): 30-day Market Town save ≤ 48 MiB; 300-day ≤ 96 MiB with bodies and ≤ 80 MiB walking; growth ≤ 0.25 MiB per simulated day (CI, every PR), and the nightly `AC-SAVE-1` at 0.5 MiB per day until stable; peak memory of a history reader on a 300-day save ≤ 256 MiB; a seal ≤ 100 ms per block.

**Revisit triggers added:** a block-level `AC-8` difference on any leg; FG-e moving blocks out of the file; a world whose span exceeds 64 MiB of raw rows.

**Note 2026-10-26 (operator ruling, "all as recommended"): ASRB-2 is loosened from 0.25 to 0.28 MiB per simulated day.** QSRB-4 allows a PR to tighten the growth bound but never to loosen it without a ruling; this note is that ruling. The CI form is unchanged: (*S*300 − *S*30) ÷ 270 of the same seed, Market Town, asserted in `tools/cli/tests/market_town.rs` (`growth_is_bounded`). The evidence:

```text
Market Town, CI form (S300 − S30) ÷ 270     before N-f   0.241 MiB/day
                                            after N-f    0.268 MiB/day   over 0.25, under 0.28
cause of the 0.027 increase                 N-f's added movement: about 128 000 more `arrived` facts
gave-way facts (also added by N-f)          0.007 MiB/day of the total
alternative considered                      not persisting transient movement facts; refused, because
                                            ARC-39 (operator QB-1) keeps every true arrival in the log
```

The new bound leaves about 4 % headroom over the measured 0.268. The 300-day size bound (ASRB-1, 96 MiB) and the nightly `AC-SAVE-1` bound (0.5 MiB per day) are unchanged. A later PR may still tighten ASRB-2; loosening it again needs another ruling.
