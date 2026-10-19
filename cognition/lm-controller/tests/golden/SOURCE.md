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
