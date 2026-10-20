# Golden renders of PR S28-a: where `s28a_renders.json` comes from

`s28a_renders.json` is the oracle of `test_home_golden.py` (pr-s28a-memory-home.md §8 AC-a-10, AC-a-13 and C-2 of
step-28). It records what memory rendered **before** the memory home existed, so that the schema-4 code can be
held to it.

## Captured from

- Code: the schema-3 `MemoryStore` (one SQLite store per seat), at commit `817d202b` (C0 of S28-a, on
  `origin/main @ b37733b9`), with C1's test modules (`home_streams.py`, `capture_golden_renders.py`) added and
  nothing under `src/` changed.
- Command, from the repository root, with the `mineworld` binary built by `cargo build -p mineworld-cli`:

  ```sh
  uv run --locked python cognition/lm-controller/tests/capture_golden_renders.py --cafe
  ```

- Date and machine: 2026-10-15, macOS (Darwin 25.2.0), CPython 3.14 through `uv`. Wall time 21.9 s (load
  average 2.8).

## What it holds

| Key | Content |
| --- | --- |
| `seed`, `days`, `people` | the synthetic world's parameters: `home_streams.world(7, 10)`, its twelve seats' entities |
| `ingest` | per seat, one point per day frame of plain `ingest`: the frame's last id and the sha256 of the 200 renders |
| `absorb` | per seat, one point per day frame that ran a pass under the default `consolidate` (its trigger key, its passes and their bound-forced removals), and the end (point 999) |
| `fgc` | Alice on the FG-c cassette's stream (`retention_stream.stream(11, 12)`, `tests/cassettes/lm-summarise.jsonl` replayed), points as `absorb` |
| `samples` | three full renders (Alice, end of `ingest`, queries 0 … 2), read by hand at C1: names, counterparts, places, citations and the four section headers render as P4 specifies |
| `cafe12_schema3` | AC-a-9's baseline: the sum of the twelve plain-`ingest` schema-3 stores' `dump()` lengths on social-cafe, seed 7, 2 days, every person (`home_streams.cafe12`): **11 712 772 bytes** |

A point's 200 queries are `home_streams.queries(7, seat, point, now, cast)`: SplitMix64 of `(seed, seat, point)`, no
clock and no `random`.

## When it may change

Never by re-running the module after C1. At C4 the schema-3 store was deleted, and the module's writer and its
`--cafe` measurement went with it: since then the module only computes what `test_home_golden.py` compares with
this file, on the schema-4 code. A deliberate change to what memory renders is a design change; it re-captures
from the code before that change, and records the commit here.

## Re-captures

- **P6c + FG-d (pr-p6c-fgd-memory-coherence.md §9, D-P6c-3), 2026-10-18.** P6c changes what retrieval renders (the
  `with` section, L0's budget 2 000 → 1 000 bytes) and FG-d what `consolidate` keeps (keepsakes), both written on
  the schema-3 store before S28-a merged. Re-captured from that schema-3 code, PR #249's head `0d57aa09`, with this
  directory's C1 modules from `e2e5dab9` (`capture_golden_renders.py`, `home_streams.py`) added and nothing under
  `src/` changed, by `uv run --locked python cognition/lm-controller/tests/capture_golden_renders.py` (no `--cafe`:
  `cafe12_schema3` is plain `ingest`'s store bytes, which neither change moves, and the writer keeps it) on scratch
  CI, ubuntu-24.04, CPython 3.12 (run 39125660610). sha256 of the file:
  `af651392247ad7a47f0ff3e9b5e56187884601c34e49bbe05b34710e16578c77`. Only `sections` hashes and the samples'
  texts moved; the seeds, points, triggers, passes and forced removals are unchanged.
- **F-FGd-2's host protection (pr-p6c-fgd §9, D-P6c-5), 2026-10-18.** The retention host overrides an assisted
  answer that would drop a direct conversation, which changes what `lm-summarise` keeps on the `fgc` arm's stream.
  Re-captured as above from `0d57aa09` with the same `host.py` guard applied (diffed line for line against the
  merged code's) on scratch CI run 39186636448, ubuntu-24.04, CPython 3.12. sha256:
  `ebc96872efa01da81c560233117e5ab6eebf4d96fe4f3fcc49db74b489443a27`. Seven `fgc` points' `sections` moved;
  `ingest`, `absorb` (plain `consolidate`), the samples and `cafe12_schema3` did not.

# Parity evidence of PR S28-b: where `s28b_parity.json` comes from

`s28b_parity.json` is the oracle of AC-b-2 (`test_retention_forgotten.py::test_attribution_never_changes_what_is_forgotten`;
`pr-s28b-retention-on-seat-views.md` §8.2, I-b-2): what each seat view held at the end of a shared-home run
**before** S28-b added forgotten-range attribution, so that schema 5 can be held to it with its new columns removed.

## Captured from

- Code: S28-a's merged schema-4 home, `origin/main @ b85f2804` (S28-a merged as `9884c64c`), with only
  `tests/capture_s28b_parity.py` added: `git diff --stat -- cognition/lm-controller/src` was empty at the capture.
- Command, from the repository root, under the machine's slot wrapper (R-LOAD-1):

  ```sh
  uv run --locked python cognition/lm-controller/tests/capture_s28b_parity.py --write
  ```

- Date and machine: 2026-10-18, macOS (Darwin 25.2.0), CPython 3.14 through `uv`; 11.3 s of CPU (the wall time
  was spent waiting for a slot; load average 18 at the end).

- **Second capture, at C2 (ledger D-b-2).** The pass log's `bytes_before` and `bytes_after` measure the dump, and
  attribution lengthens every `forgotten` line, so a schema-5 dump cannot equal a schema-4 one with only the new
  columns removed. `views` was therefore captured from the same unchanged code, in a detached worktree of commit
  `65936053` (C0: `src/` is `b85f2804`'s) with the C2 version of `capture_s28b_parity.py` (which adds
  `parity_view`, test-only) copied in, by the same command; 10.9 s CPU, 13.1 s wall, not under the slot wrapper.
  Before it, every one of the 36 seat views of both codes was compared line by line with the parity view's rule
  applied to both (a scratch comparison): identical, so nothing but those cells differs.

## What it holds

| Key | Content |
| --- | --- |
| `seed`, `days`, `frame`, `interleaving` | `home_streams.world(7, 30)`, every seat in one shared home, frames of 256 interleaved by seed 1 |
| `policies` | AC-b-1's three: `consolidate` (defaults), `decay` (`threshold` 1 000), `importance-recency-cap` (every `k` 0) |
| `dumps` | C0's capture: per policy and seat, the sha256 of the whole seat-view `dump()` at the end of the run |
| `views` | C2's capture, AC-b-2's oracle: the same, of `capture_s28b_parity.parity_view(dump)` |

## When it may change

Never by running the module's `--write` again after C0: from C2 on the tests use it only to compute. A deliberate change to what retention
keeps is a design change; it re-captures from the code before that change and records the commit here.

- **Third capture, after the merge of main with FG-d (ledger D-b-8), 2026-10-19.** FG-d's keepsakes (F-FGd-1) and
  the host's protection (F-FGd-2) change what `consolidate` keeps, so its 12 `views` were re-captured from main's
  unchanged schema-4 code (`5b576413`, the main side of merge `2c13d9c9`) in the detached scratch worktree, with
  this `capture_s28b_parity.py` and the JSON copied in, by the same command: 14.0 s CPU, 34 s wall. `decay` and
  `importance-recency-cap` came out unchanged. The new `consolidate` digests equal what the schema-5 code gives.
