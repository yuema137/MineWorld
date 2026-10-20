# Recorded observations of needs, sleep and lateness: where they come from

Frozen evidence for N-e's readers, panels and agent view (`.structured-coding/plans/mvp0/pr-s35-ne-presentation.md`
§5). Each file is one `Observation` as a seat received it from the real `mineworld server` built from
`mvp0/pr-s35-ne-2d` at `c1f781ea` (the base is `mvp0/pr-s35-nc @ 036f7fbb`, with N-a, N-b, LE-a/b/c and N-c). The
JSON is the observation the client holds, serialised from the wire type with its keys in the order the
protocol's Rust struct gives them. The expected values in the tests are written by hand from these files, never
read from the code under test.

| File | What it is | sha256 |
| --- | --- | --- |
| `market-town-night.json` | Market Town with the packs (`tools/cli/tests/fixtures/needs.rs` copy), `--town`, at 02:32 on day 1; seat 8 is asleep since 02:25 with its alarm at 09:11, hungry at `peckish` and tired (`tired`), and its agenda's next part is coffee at 07:00 | `739439a10bda84b1d4cf8dfb63445fae7707316ca7784adade0b3b19aad60075` |
| `commute-late.json` | commute-town with the packs; seat 8 (Ben) is held by the recorder from genesis and never moved; at 12:00 on day 1 his shift at the bakery, which began at 07:30, is over and he never came: one `late-for-shift` with `absent: true`, `late_by: 16200`, `count: 1`, `level: notice`, told to him by employer 12; he is `famished` and `exhausted`; his agenda's next part is work at the bakery at 07:30 on day 2, punctual | `7422037a8d16c4a7df9c255a595c2b36dfbae79c8cdc1d1355a7a8f555107b9b` |
| `commute-sleeper.json` | commute-town with the packs and sleep permitted at the class `bakery-floor` (the bakery's tag `bakery`), written into the recorder's copy; Ana (seat 7) walked home to the bakery and slept there from 00:05 on day 1 for 60 minutes; Ben (seat 8) walked in and his first observation with Ana in the room shows her asleep since 315 s with no alarm (the alarm is only the sleeper's), and his own needs `ok` and `tired` | `42ac37c034f1b43ce467af47c940d5f9a19deff40b98449e6e813fc9482510e3` |

**Recorder.** `tools/cli/tests/needs_fixtures.rs`, three `#[ignore]` tests, run by hand, once:

```sh
MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test needs_fixtures -- --ignored --test-threads=1
```

Each recorder starts the binary as `mineworld server <copy> --solo --time-scale 300 --cadence 1 --hold 0`
(`--town` for the first only; `--solo` is the flag that lets the clock run above 24 times the day, and it listens
on loopback only). The recorded run reached 02:30 on day 1 in about 30 wall seconds for Market Town and about
150 for the commute recording. The first recorder picks the first seat whose own `asleep` (with an alarm),
`needs` and `agenda.next` are in view at 02:30; the others are disconnected and released.

**Deviations from the plan's §5 (recorded, not hand-edited):**

- `commute-late.json`: the plan had Ben walk to the bakery late. The recorder does nothing and he never comes,
  so the fact is the absent form of `late-for-shift` (its own `late_by` is the whole shift). Both forms are the
  same event; the absent one reaches the observer at the shift's end without a walk.
- `market-town-night.json`: no seat asleep at 02:30 in the run was a worker whose next part was punctual, so the
  recorder asks for the next part only. The punctual next part is in `commute-late.json` (Ben's work at 07:30).

**Hand edits: none.** A recording that cannot reach a must-contain item is not replaced by an edited file; each
item above was found in the recorded observation.

A re-recording that changes these files is a reviewed change: update the hashes and say why.
