# PR S10 P7 — Milestone D: speak to Alice in 2D, meet her in 3D, and she remembers

## EXECUTION DETAIL UNDER A FROZEN STEP (bounded detail, not a new design)

```text
Scope authority:        step-17-cognition.md, STEP DESIGN FROZEN 2026-10-08: §3.14 (the claim, the automated test
                        steps 1–8, the operator's run, the false successes), §3.16 (removability), §9 (the P7 row and
                        its dated note DOC-24), §10 (IC-6, IC-7, IC-8, IC-10), the deliverables table (P7 row)
Binding later rulings:  pr-s10-p6-lm-controller.md (DESIGN FROZEN 2026-10-14) §10: R-P7-P6-1, R-P7-P6-2;
                        pr-fg-b-retention-host.md §11 R-P7-FG-1; pr-fg-b2-size-trigger.md (the 20 MiB default bound,
                        operator 2026-10-14)
This document:          the commit-level plan the step does not give, written by the P7 implementation session as a
                        bounded detail under the frozen scope, on the coordinator's instruction of 2026-10-15
                        ("if the step does not already detail P7 to the commit, write one as a bounded detail under the
                        frozen scope and record it"). It changes no frozen invariant, contract, ownership or scope.
                        Where it must reconcile two binding documents it says so (§3) and follows the later ruling.
Implementation base:    origin/main @ 5cbb4697 (P6b merged as #207; P6a not merged)
Branch / worktree:      mvp0/pr-s10-p7, /Users/yuema137/mineworld-worktrees/impl-p7 (this session alone)
Lifecycle:              READY FOR OPERATOR REVIEW (CI on the exact head: see the PR); Q-P7-1 resolved; Q-P7-2 and Q-P7-3 ruled to follow-ups (P6c, FG-d)
```

**Effort:** `mvp0` · **Step:** [`step-17-cognition.md`](step-17-cognition.md) · **Parent:** [`overall.md`](overall.md)
· **Consumes:** [`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (P6b merged; P6a in progress elsewhere)

---

## 1. Goal

Prove Milestone D against the real binaries: one persisted world, Alice on an `lm` seat driven by the real cognition
process, a `2d`-tagged protocol client speaks to her, both processes are SIGKILLed and restarted, a `3d`-tagged
protocol client meets her, and her reply is decided from a context that holds the Event IDs of the 2D lines and cites
them. Then prove the same world valid with the model unplugged, prove the backend swappable without touching the World
Pack (`AC-4`, IC-6), and prove the LM controller removable by deletion (IC-7). Publish Milestone D's review package.

## 2. Preconditions, checked on 2026-10-15

| Needs | State | Consequence |
| --- | --- | --- |
| P6b (the controller, `python -m mineworld_cognition`, `replay_log`) | merged, #207 | P7 builds on `origin/main` |
| P6a (the `persona` pack) | **not merged** (branch `mvp0/pr-s10-p6a`, resumed in parallel) | **Nothing in P7 needs it.** P6b reads `Persona` when present and works without it (P6 §5.2.1). The recorded cassette is replayed through `replay_log`, which re-runs the **logged** inputs, persona included, so P6a's later merge cannot make the cassette miss. Only the operator's live demonstration reads better with a persona; it runs without one |
| DL-a, FG-b, FG-b2, FG-c | merged | the decision list, the bounded store, the 20 MiB default bound, the summary boundary |
| S12, S14 (the real clients) | present on `main` (`mineworld-2d`, `mineworld-3d`) | the operator's run (§3.14.3) is documented; the automated test uses protocol-level clients |
| A local model for the recording | `ollama` with `qwen3.5:4b` (the admitted local triple, P6 §0.1) on this machine | the cassette is recorded locally, with no key and no paid call (QS10-19) |

No part of P7 is blocked by P6a. §7 lists what is held for an operator ruling.

## 3. Reconciliations (bounded; each follows the later binding ruling)

- **D-P7-1 — the live run is scripted; the cassette is replayed from its log (R-P7-P6-1).** Step-17 §3.14.2 step 1 runs
  the cognition process with `mode = "replay"` against `milestone_d.jsonl`. P6 found (F-P6-5) that a live hosted run is
  not reproducible at the request level (world instants follow wall time, event ids interleave with movement), so a
  strict cassette misses on a fresh live run, and its frozen design places R-P7-P6-1 on P7: *"Milestone D's live
  scenario runs with `ScriptedBackend` or replays a logged run through `replay_log`."* P7 does both, and keeps each
  claim on the half that can carry it:
  - `tools/cli/tests/milestone_d.rs` runs steps 1–8 against the real server and the real cognition process
    (`SeatRunner`, `__main__.serve`), whose `social` tier is a `ScriptedBackend`: a pure function of the request that
    quotes the earliest line it is shown as said to it and cites that line's token (the step's own description of
    `scripted`, §3.11.3). Every identity, memory, restart and provenance assertion is made here.
  - `cognition/lm-controller/tests/cassettes/milestone_d.jsonl` and its decision log
    `cognition/lm-controller/tests/fixtures/milestone_d.decisions.jsonl` are recorded once from the same scenario with
    the local model (`record` mode, `qwen3.5:4b`). `tests/test_milestone_d_replay.py` replays the log through
    `replay_log` and asserts step 6's model-dependent clauses on the **recorded completion**: `recalls` cites a 2D line
    and the words mention the umbrella. IC-6 is the same replay under two backend bindings.
- **D-P7-2 — the 100-day bound is the seat's bound, 20 MiB by default (DOC-24 vs FG-b2).** Step-24 §8.6 (2026-10-12)
  wrote "≤ 1 MiB". FG-b2 (operator ruling 2026-10-14) replaced the fixed 1 MiB with a per-seat `store_bound_mib`,
  20 MiB by default and at least 20. P7 asserts the store is within Alice's resolved bound after 100 days and records
  the measured size beside the historical 1 MiB figure.
- **D-P7-3 — the removal check removes one workspace line (IC-7).** The root `pyproject.toml` names
  `cognition/lm-controller` as a uv workspace member, so deleting the directory alone makes `uv sync` refuse the
  workspace. The member line is the Python twin of a pack's `installed!` line: registration, not a dependency. The
  check deletes the directory **and** that one line, re-resolves the lock, and runs everything else. Any other edit the
  check would need is a failure of IC-7.
- **D-P7-4 — where the tests run.** `milestone_d.rs`'s model-driven tests need the Python workspace, which the `core`
  job does not install (`scripts/ci_layer.py`: "Its tests never join `core`"). They are `#[ignore]`d with a reason and
  run, never skipped, by the `python` layer (`cargo test -p mineworld-cli --test milestone_d -- --ignored`), which has
  both toolchains. Step 8 (Alice unplugged) needs no Python and runs in `core` as an ordinary test.

## 4. Audit (origin/main @ 5cbb4697)

| Item | Where | What matters |
| --- | --- | --- |
| `python -m mineworld_cognition` | `cognition/lm-controller/src/mineworld_cognition/__main__.py` | `main(argv)` loads, reads the assignment, `build_router(config)`; `scripted` mode needs a backend supplied in code, which `main` cannot take today |
| `SeatRunner` | `controller/seat.py` | store `store/<instance>/alice.sqlite`, log `alice.decisions.jsonl`, resume file `alice.resume`; joins with the store's cursor; one consult at a time |
| `DecisionRecord` | `controller/log.py` | `input.memory.lines[*].citation.ranges`, `input.trigger`, `record.attempts[*].key`, `record.outcome`, `record.request`; `replay_log(log, cassette)` builds its own replay gateway |
| `RecordingBackend` | `record.py` | writes `<cassette>.partial`, flushed per entry, renamed on a clean close; a SIGKILL leaves the `.partial` and the next `record` start refuses |
| The memory store | `memory/store.py` | `meta(key, value)`: `cursor`; `l0(event_key, event_id, …)` |
| The save | `persistence/src/sqlite.rs` | `journal(revision, at, action_id, entry)`, `facts(event_id, revision, fact)` |
| Rust test support | `tools/cli/tests/support/mod.rs` | `Server::start`, `Client::{connect, join_as, walk_accepted, submit_accepted, perceived}`, `talk`, `walk`, `tagged`, `run_command`; `rusqlite` is already a dev-dependency |
| CI | `scripts/ci_layer.py` | `python` layer: static checks, `cargo build -p mineworld-cli`, pytest per member; `core`: `cargo test --workspace` |
| References to the LM controller outside it | `git grep` | none in any `*.rs`, `Cargo.toml`, `sdk/python`, `clients/agent` (I-2 holds today) |

## 5. Acceptance (fixed before measuring)

| Id | Criterion | Mutation that must fail it |
| --- | --- | --- |
| AP7-1 (steps 1–3) | the server and the cognition process start; a sink listener named as the backend's `base_url` accepts **zero** connections over the whole run; the `2d` visitor's line is answered by a `spoke` from Alice to the visitor whose request's actor is Alice and whose request was submitted by her seat (the decision log's acted consult) | — |
| AP7-2 (step 4) | after SIGKILL of both and a restart of both: the same world instance in both welcomes; Alice's store file is the same store (its pre-kill `l0` rows unchanged) and its cursor did not go back | — |
| AP7-3 (steps 5–6) | the `3d` visitor's "Hello again." is answered; that consult's logged memory cites the Event IDs of step 2's and step 3's `spoke` facts; its `recalls` cites at least one; the resulting `spoke` fact is in the save, from Alice to the visitor | **IC-8:** the store deleted between steps 4 and 5 with re-ingestion disabled: the test fails naming the absent ids |
| AP7-4 (step 6, recorded) | `replay_log` over the recorded log and cassette reproduces every key and request; the recorded 3D consult cites a 2D line and its words mention the umbrella | — |
| AP7-5 (step 7) | `mineworld inspect` on the save resolves every cause (exit 0) | — |
| AP7-6 (step 8) | the same scenario with no cognition process and the server without `--town`: the visitor's lines are in the save and Alice says nothing | — |
| AP7-7 (IC-6, `AC-4`) | the recorded log replayed under two backend bindings (different name, URL and model): every key and every request identical, no file under `worlds/` touched | the model name put into the key: the second binding misses |
| AP7-8 (IC-7) | with `cognition/lm-controller/` deleted and its workspace line removed in a scratch copy: `cargo test --workspace`, the SDK's and the agent library's tests, and `milestone_d`'s step 8 pass | an SDK module importing `mineworld_cognition`: the scratch run fails |
| AP7-9 (IC-10, live half) | a fake key planted in the cognition process's environment appears in no artefact of the run (save, store, decision log, process output) | — |
| AP7-10 (DOC-24, D-P7-2) | after 100 simulated days of the same save, Alice's store, absorbed with her resolved policy, is within her bound, and the memory a "Hello again" from the visitor retrieves still cites a 2D line | — |

## 6. Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review separately (§9 ledger).

- **C0 — this document and the handoff.** Markdown only.
- **C1 — the scripted cognition entry and Milestone D over the real binaries.**
  `__main__.main(argv, *, scripted=None)` (additive: a backend supplied in code reaches `build_router`);
  `cognition/lm-controller/tests/milestone_d_cognition.py` (the scripted function and the entry the Rust test
  starts); `tools/cli/tests/milestone_d.rs` (steps 1–8, AP7-1…3, 5, 6, 9; the record variant, ignored and
  environment-gated); `scripts/ci_layer.py` `python` layer runs the ignored tests. Validation: the file, then the
  IC-8 mutation red. Review: the test reads only frames, the save, the store and the log; no world rule moves.
- **C2 — the recorded cassette and its replay.** Record with `qwen3.5:4b` locally; commit the cassette and the
  log; `replay_log(..., gateway=None)` additive parameter; `tests/test_milestone_d_replay.py` (AP7-4, AP7-7).
  Validation: the file; the key mutation red. Review: no provider or model name in any key (IC-10).
- **C3 — removability (IC-7).** `scripts/ci_removability.py` and a `removability` layer; CI job. Validation: the
  script locally on the Python half and step 8; the full Rust half on CI. Review: the check edits nothing but the
  directory and the one line.
- **C4 — the 100-day assertion (AP7-10).** `tests/test_milestone_d_100_days.py` (`real_binary`). Validation: the file.
- **C5 — the review package and close-out.** `docs/HUMAN_REVIEW_QUEUE.md` Milestone D package; ledger; evidence.

## 7. Held for the operator (not blocked by P6a)

- **Q-P7-1 (R-P7-P6-2).** *"The 100-day fallback bound is measured over a recorded run's decision log with the
  counters."* A recorded 100-simulated-day run with a model on Alice's seat is ≈ 100 wall days at the hosted pace, or a
  `--time-scale`d run whose triggers go stale at a rate set by the scale rather than by the model. AP6-4 already holds
  the statistical bound (1 000 seeded consults, 30 % garbage, ≤ 5 %). Recommendation: accept the bound as AP6-4 plus
  the counters of the recorded Milestone D run (reported in §9), and run a `--time-scale` 100-day LM run as an
  operator-run or nightly task under its own design. **Ruled 2026-10-16 (§7.1): run from the scripted model; PASS, AP7-11.**
- **Q-P7-2 (AP7-4's model clause, F-P7-5).** With the admitted local model (`qwen3.5:4b`, `structured_output =
  "none"`, `reasoning = "off"`), the recorded 3D reply is decided from a context that holds both 2D lines (asserted),
  but the model's words do not use them: *"Hello Vera. It's good to hear from you."*, `recalls: ["[#75]"]` (the 3D
  line itself). The requests are seeded, so repeated recordings give the same answer (5 further runs on 2026-10-15:
  0 cited a 2D line; the 2D reply did mention the umbrella every time). Options: (a) record the cassette with the
  admitted API model `gpt-4.1-nano` (40/40 in the spike), a paid call that needs your approval and your key, run by
  you; (b) a P6 follow-up that puts what the speaker said before into a section of its own at the top of WHAT I
  REMEMBER (today it is one line among 48 in "The last day", behind name-learning noise); (c) accept the recorded
  sample as the honest result of a 4B local model. Recommendation: (b) as a follow-up PR, then re-record locally; (a)
  if you want the evidence before that. The replay test prints the clause instead of asserting it.
- **Q-P7-3 (AP7-10's citation clause, F-P7-8).** After 100 simulated days of the same save (the visitor's seat driven
  by its rule controller, 2 142 exchanges with Alice), Alice's store is 787 316 B of her 20 MiB bound (the historical
  1 MiB also holds), but no line of the day-100 context cites the 2D line or her reply: `consolidate` keeps episodes
  8 weeks, and the people lines cite first and last facts only. Step-24 §8.6 wrote this assertion as an adversarial
  check on importance, and it fails. Options: (a) a retention change that keeps a line addressed to me by a person
  in a stable, cited form (an FG-b follow-up); (b) restate the check for a visitor who is absent for the 100 days
  (needs a decision model that drives nothing, which DL-a does not install). Recommendation: (a). The clause is a
  strict `xfail` in `test_milestone_d_100_days.py`.

### 7.1 Rulings, 2026-10-16 (primary session, under the operator's merge authority; relayed by the coordinator)

| Question | Ruling | Done here |
| --- | --- | --- |
| Q-P7-1 | Run the 100-day bound from the scripted deterministic model on Alice's seat: free and reproducible | `tests/test_milestone_d_fallback.py` (AP7-11). A hosted `--time-scale`d run was tried first and starves (in-server controllers are paced in wall seconds, QTW-13: 11 consults in 100 days at one day per wall second), so the run replays Alice's real 100-day history through her consult engine. The garbage draw includes how often the request was asked, so a transport retry draws afresh (D-P7-9) |
| Q-P7-2 | No paid recording. Follow-up **P6c** (another agent, stacked on this branch): the current speaker's earlier lines in a section of their own at the top of the memory view, and a prompt rule to use it. Until P6c lands the model clause stays printed, not asserted; then re-record with local qwen and assert it | unchanged: printed |
| Q-P7-3 | A real defect against the operator's memory principle. Follow-up **FG-d** (same follow-up agent): conversational-partner salience in the default retention. Keep the strict xfail | unchanged: strict xfail |
| D-P7-6, D-P7-7 (prompt wording in P6b) | Accepted for now; on the milestone review list | — |

## 8. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-P7-1 | The Python job grows past its ceiling | small scenario, `--hold 3`, one server per test; wall times recorded |
| R-P7-2 | The local model's recorded words do not mention the umbrella | the recording is one honest sample; if the words miss, that is recorded as a finding, not re-rolled silently |
| R-P7-3 | A killed `uv run` leaves its Python child holding the seat | the cognition process is started as the workspace's own interpreter (`.venv/bin/python`, `.venv\Scripts\python.exe`), so `Child::kill` kills the process that holds the socket |

## 9. Ledger (live)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | c599c831 | [x] | [x] doc checks PASS | [x] scope and reconciliations re-read against the step |
| C1 | 653636fa | [x] | [x] milestone_d both tests, IC-8 and F-P7-3 mutations red | [x] reads only frames, save, store, log; no world rule moved; file 843 lines (test, >800 noted) |
| C2 | c1913cf7 | [x] | [x] replay 3 passed, IC-6 mutation red | [x] no provider term in the log (scan), key unchanged by binding |
| C3 | 86df2324, db6389cf | [x] | [x] Python half PASS, mutation red; Rust half: CI `removability` | [x] edits only the directory and the member line |
| C4 | f1dbad32 | [x] | [x] 1 passed, 1 xfailed (strict) | [x] bound from the resolved retention, not a literal |
| C5 | f9c1f39e | [x] | [x] doc checks PASS | [x] review package states both measured gaps |
| C6 | this commit | [x] AP7-11, the 100-day fallback bound (Q-P7-1 ruling) | [x] 1 passed 40.6 s; mutation red | [x] every input seeded; the bound counts the whole consult path, not garbage alone |

Commit mapping: the `removability` layer of `scripts/ci_layer.py` landed in C1's commit beside the `python` layer
line (one file); it belongs to C3. C3's Rust half (`cargo test --workspace` without the directory) is evidenced by
CI's `removability` job only, and stays CI-pending until that job is green on the exact head.

### Evidence

All local runs on macOS (arm64), 2026-10-15, base `5cbb4697` plus this branch's working tree, heavy runs inside
`cast-slots/with-slot.sh` (R-LOAD-1). No workspace-wide run locally; CI on the exact head is the workspace gate.

| Criterion | Result | Evidence |
| --- | --- | --- |
| AP7-1 … AP7-3, AP7-5, AP7-9 | **PASS** | `cargo test -p mineworld-cli --test milestone_d -- --include-ignored --exact she_remembers_across_the_two_views`: ok (17.6 s with the unplugged run). The 2D lines #59 (visitor) and #62 (Alice) are in the context of her 3D reply and cited by its `recalls`; same instance across the restart; the store's pre-kill rows unchanged and its cursor not back; both of Alice's lines caused by requests journaled as Alice and submitted by acted consults; `inspect` exit 0; the backend's sink listener accepted nothing; the planted key in no file and no printed line |
| IC-8 mutation | **red, reverted** | the store file deleted between steps 4 and 5 and the backfill not absorbed (planted in the test and `seat.py`): `IC-8: the context of Alice's 3D reply holds the 2D line #59; it cites {67, 71}` |
| AP7-6 (step 8) | **PASS** | `the_world_runs_with_alice_unplugged`: ok; runs in `core` |
| AP7-4 | **PASS (replay, context) / FAIL (model clause, Q-P7-2)** | `pytest tests/test_milestone_d_replay.py`: 3 passed; 4 consults replayed, 0 mismatches; the 3D context holds #59 and #62; printed: `recalls_cite_a_2d_line: false`, `words_mention_the_umbrella: false` |
| AP7-7 (IC-6) | **PASS**, mutation red | two bindings (`local`/`qwen3.5:4b` at loopback, `elsewhere`/`another-model-entirely` at `models.example.invalid`): 0 mismatches each, `worlds/` untouched. Mutation (the bound model's name appended to the key): `CassetteMiss … was never recorded`, reverted |
| AP7-8 (IC-7) | **PASS (Python half) / CI (Rust half)**, mutation red | `CARGO_TARGET_DIR=target python3 scripts/ci_removability.py --python-only`: 2 450 files copied without the directory, member line removed, `uv sync` 0, SDK 106 passed, agent 98 passed. Mutation (`import mineworld_cognition` in the SDK's `__init__`): `ModuleNotFoundError`, exit 4, reverted. `cargo test --workspace` in the copy: the `removability` CI job |
| AP7-10 | **PASS (bound) / FAIL (citation, Q-P7-3, strict xfail)** | `pytest -s tests/test_milestone_d_100_days.py`: 1 passed, 1 xfailed, 39.9 s; `run` to day 100 24.0 s; 124 066 facts absorbed; 787 316 B ≤ 20 971 520 B; `line_cited: false`, `reply_cited: false` |
| AP7-11 (Q-P7-1, R-P7-P6-2) | **PASS**, mutation red | `pytest -s tests/test_milestone_d_fallback.py`: 1 passed, 40.6 s. Alice's 100-day history (seed 7, 3 303 lines said to her, one in 3), 1 101 consults through the consult engine with `remembers` behind seeded 5 % transport and 30 % garbage per attempt: 1 021 acted, 64 chose nothing, **16 fell back (1.4 %)**, all `content`; 374 invalid answers, 94 content and 9 transport retries. Mutation (content layer `attempts` 3 → 1): 280 of 1 101 fell back (25.4 %), red, reverted |
| F-P7-3 regression | **PASS**, mutation red | `test_a_restarted_process_does_not_answer_again_a_line_its_store_already_held`; with `detect` over every redelivered fact: `['2', '2', '3'] != ['2', '3']`, reverted |
| Touched P6 tests | **PASS** | `test_controller_{choice,seat,log,consult,parts,server,resilience}.py`, `test_provider_scan.py`, `test_milestone_d_replay.py`: 71 passed (69.9 s); ruff check and format, pyright strict: 0 errors; `cargo fmt --check`, `cargo clippy -p mineworld-cli --test milestone_d -D warnings`: clean |

**CI, first head `f9c1f39e` (run 38822679254).** PASS: `fast`, `test`, `test-macos`, `python` on Ubuntu and
Windows (the live Milestone D test and the 100-day test with its strict xfail), `platforms`. FAIL, each classified:
- `removability`: its own defect. `ac1_composability` checks 1 and 2 read git history and the file list, and the
  plain file copy was no repository. Repaired in `db6389cf`: the copy is a detached worktree of HEAD with one
  throwaway commit. Locally, in that worktree: `ac1_composability` 15 passed; the Python half PASS.
- `python (macos-15)`: `clients/agent/tests/test_idle.py::test_a_handle_without_calls_gives_its_seat_back` refused
  `seat_occupied`. Out of scope: `clients/agent` is untouched here, and the same file passed in the scratch copy and
  locally (98 passed). The cognition member passed on that leg: 425 passed, 1 xfailed.
- `test-windows`: cancelled at its 60-minute limit while still running (`social_interactions`), with no test failed.
  This is pre-existing: `main`'s run 38818061551 was cancelled the same way. P7 adds one ordinary test to `core`
  (the unplugged run, ≈ 15 s).

The recording: `MINEWORLD_MILESTONE_D_RECORD=… cargo test -p mineworld-cli --test milestone_d -- --ignored --exact
record_the_milestone_d_cassette` against Ollama's `qwen3.5:4b` on loopback (no key, no paid call). Its own step-6
assertion fails on the model's recalls (Q-P7-2), so the committed cassette and log were assembled from the kept
scratch of the last of five runs (`MINEWORLD_KEEP_SCRATCH=all`): `cassette-1.jsonl` and `cassette-2.jsonl` (header
once) and `store/alice.decisions.jsonl`. Nothing in them was edited.

### Deviations and findings

- **F-P7-1 (bounded fix).** `python -m mineworld_cognition` never closed its router, so a `record` session's cassette
  stayed `.partial` and the ledger was never closed. `__main__._closing` closes it however serving ends. Reviewed;
  exercised only by a clean stop, which no test sends (the recording's processes are SIGKILLed by the scenario).
- **F-P7-2 (finding, known D-P6b-15).** A restarted process presents no cursor at its first join, so the server
  backfills the world's history; the store takes it without change (AP7-2 holds), at the cost of the whole history
  per restart. For P6's owner: open the store before the first join when its file exists.
- **F-P7-3 (bounded fix, the F-13 rule for an lm seat).** That backfill made a restarted process answer the 2D line
  a second time. `SeatRunner._ingest` now detects triggers only on facts newer than the store's cursor before the batch.
- **F-P7-4 (bounded fix).** `examples/local.toml` left `reasoning` unset; `qwen3.5:4b` then thinks, spends the 256
  output tokens and answers empty (`truncated`), and the re-ask times out. `reasoning = "off"` added.
- **F-P7-5.** The recorded model does not use the 2D lines in its 3D reply: Q-P7-2.
- **F-P7-6 (bounded fix).** `replay_log` replayed a consult logged `dropped` (superseded at submit) as `acted`: the
  pre-submit verdict is not in the log. It now replays that verdict from the logged outcome.
- **D-P7-5 (bounded).** `test_milestone_d_100_days.py` imports test helpers from `test_controller_server.py` rather than
  copying them (D-P6b-31 copied); `realserver.Server` gains `town=False`, so the 100-day test starts the plain server as `milestone_d.rs` does. A first failure there was the test's own: a loop on `visitor.changed()` with no `since` never yielded to the in-process seat runner; it now waits for a newer frame each time.
- **D-P7-6 (bounded P6b prompt change).** WHAT JUST HAPPENED said "A person said to me", so a model with two people in
  reach could not know whom to answer (the scripted model answered Bob). It now names the speaker by the situation's
  label ("Someone" if not present); no id reaches the prompt (I-10). Golden `prompt_request.json` updated.
- **D-P7-7 (bounded P6b prompt change, for the operator's review).** With the shipped rules `qwen3.5:4b` answered
  `act: "attempt"` with no offers and wrote recalls unquoted (`[#59]`, `not_json`) on all three attempts. Three rule
  lines were added to the system message: when to use `say` and `attempt`, that a recall is a JSON string token, one
  example object, and "answer them as yourself, using what you remember of them, and cite the memories you used".
  After them every recorded consult parsed. The layout of §5.2.4 is unchanged.
- **D-P7-9 (bounded test design, AP7-11).** The bound's model draws by (seed, request, occurrence). A draw by the
  request alone (`garbage.py`'s, used by AP6-3) fails a transport retry every time, since a retry re-sends the same
  request; measured that way, 15 of 289 sampled consults fell back on transport alone (7.6 % in all). The decision
  log is written as one file (rotation off) so the run is read back whole; with the 8 MiB rotation only the last
  289 of 1 101 records remained in the current file.
- **D-P7-8 (bounded test design).** In the live test the visitor waits 2 s at the counter before speaking, because
  Alice's view can trail the visitor's (D-P6b-30); a reply is waited for in the decision log before a kill, because the
  log line follows the world's acceptance.

### Execution contract

```text
PROJECT / PR:            S10 P7 — Milestone D
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p7-milestone-d.md (this file)
RELATED / BINDING DOCS:  step-17-cognition.md; pr-s10-p6-lm-controller.md; pr-fg-b-retention-host.md;
                         pr-fg-b2-size-trigger.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md
IMPLEMENTATION BASE:     origin/main @ 5cbb4697
APPROVED SCOPE:          step-17 P7 row; nothing in kernel/, contracts/, server/, persistence/, systems/, PROTOCOL.md
FROZEN INVARIANTS:       step-17 I-1 … I-16; P6 I-P6-*; no paid API call; no key read, printed or copied
VALIDATION BUDGET:       targeted tests only (R-LOAD-1); heavy runs inside cast-slots/with-slot.sh; the local model
                         for the one recording; CI on the exact head is the workspace gate
ENDPOINT AUTHORITY:      implementation, commits, push, PR creation, CI repair: authorized (coordinator's
                         instruction of 2026-10-15); merge: explicit operator authorization only
POST-MERGE SYNC OWNER:   the primary session (step-17 §15.2, overall.md); this session owns this document
STOP CONDITION:          PR "S10 P7: Milestone D proof" open, READY FOR OPERATOR REVIEW, CI on the exact head; DO NOT MERGE
```
