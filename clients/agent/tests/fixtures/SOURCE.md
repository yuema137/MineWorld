# Recorded observations: where they come from

Frozen real evidence for the view tests (pr-s27c-player-library.md F-S27c-3; test rules §25). Each file is one
`Observation` exactly as the real server sent it to a seat, written by `record.py`. Expected values in the
tests are read from these files, never from the code under test.

| File | What it is | sha256 |
| --- | --- | --- |
| `market-town-start.json` | the visitor's first observation, in the café at genesis: 4 people, 21 offers | `ae66089886aba9487fd2cc1bf16ee461ec07d24357f487ffccd137bc9babd6c9` |
| `market-town-walking.json` | the visitor's, after `walk-to` the street and one `walk-step`: its own `walking` disclosed, 22 offers | `2205e581b04a9d58ef687c854f162a3d558fc9b61a8ecac82d20514c017735ae` |
| `market-town-street.json` | the wanderer's, joined at 13:46 on day 1 while the town walked it: the street, with passages to all five places, 7 offers | `d753865a1d3422267c6d541572fea88616186a1b996fe0bf986e4043cc1746f6` |

**Source.** Recorded 2026-10-14 from the `mineworld` binary built (`cargo build -p mineworld-cli`) on branch
`mvp0/pr-s27c` at `f8e30c6`, whose `server/`, `systems/`, `worlds/` and `tools/` are identical to
`origin/main @ f772a18`. Command, from the repository root:

```sh
MINEWORLD_BIN=<target>/debug/mineworld uv run --locked python clients/agent/tests/fixtures/record.py
```

It runs `mineworld server worlds/market-town --town --agent alice --listen 127.0.0.1:0` for the first two,
then the same with `--time-scale 300 --pace 1` until `/status.at` passes 13:40 for the third.

**The café "at its busiest seeded hour" (step-27 R-S27-3) was not reached.** At `--time-scale 300 --pace 1`
the town's controllers had walked the wanderer only as far as the street by 13:46, and no recorded hour put
more people in the café than genesis does (Alice, Bob, the visitor and the wanderer). The genesis café is
therefore the densest recorded market-town observation (≈ 25 KiB as the server sends it); the size cap is
also exercised by a synthetic crowd built from it (`recorded.py` `crowd`: 200 people, things and offers).

A re-recording that changes these files is a reviewed change: update the hashes and say why.
