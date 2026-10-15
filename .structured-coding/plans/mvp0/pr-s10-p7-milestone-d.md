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
Lifecycle:              IN IMPLEMENTATION
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
  operator-run or nightly task under its own design. Until ruled, this criterion is **NOT RUN**.

## 8. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-P7-1 | The Python job grows past its ceiling | small scenario, `--hold 3`, one server per test; wall times recorded |
| R-P7-2 | The local model's recorded words do not mention the umbrella | the recording is one honest sample; if the words miss, that is recorded as a finding, not re-rolled silently |
| R-P7-3 | A killed `uv run` leaves its Python child holding the seat | the cognition process is started as the workspace's own interpreter (`.venv/bin/python`, `.venv\Scripts\python.exe`), so `Child::kill` kills the process that holds the socket |

## 9. Ledger (live)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | — | [ ] | [ ] | [ ] |
| C1 | — | [ ] | [ ] | [ ] |
| C2 | — | [ ] | [ ] | [ ] |
| C3 | — | [ ] | [ ] | [ ] |
| C4 | — | [ ] | [ ] | [ ] |
| C5 | — | [ ] | [ ] | [ ] |

### Evidence

### Deviations and findings

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
