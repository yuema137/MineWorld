# PR S10 P6 — The LM controller: an AI character that decides through a language model

## DRAFT — PR DESIGN, NOT FROZEN

```text
Design revision:        revision 1 (2026-10-14), DRAFT
Author:                 S10 P6 design agent, worktree /Users/yuema137/mineworld-worktrees/design-p6,
                        branch docs/p6-design, planning base origin/main @ 171ea06
Approved by / evidence: none yet. This document authorizes no implementation (CLAUDE.md §3.1)
Implementation base:    origin/main with DL-a and FG-b merged (C0 records the commit); FG-c lands
                        alongside (§10, R-P6-FGc-1)
Start condition:        DL-a merged AND FG-b merged AND this design frozen by the primary session with
                        the operator's thresholds (§8, §14) AND the operator's go
Execution contract:     §12 (drafted; every line marked `unresolved` is settled at freeze)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-17-cognition.md`](step-17-cognition.md) §§3.4–3.6, 3.10–3.16, §9
(the P6 row with its dated notes of 2026-10-11, 2026-10-12 and 2026-10-13), §15.2 (the P6 row, carried
findings), §15.8 (platforms) · **Also binding:** [`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md)
§§4.2, 5.3, 8.6, AC-DL-5 · **Parent:** [`overall.md`](overall.md) · **Consumes:**
[`pr-s10-p3b-perceived.md`](pr-s10-p3b-perceived.md) (merged), [`pr-s10-p4-memory.md`](pr-s10-p4-memory.md)
(merged), [`pr-s10-p5-backends.md`](pr-s10-p5-backends.md) and
[`pr-s10-p5b-hosted-subscriptions.md`](pr-s10-p5b-hosted-subscriptions.md) (merged),
[`pr-dl-a-decision-list.md`](pr-dl-a-decision-list.md) (frozen), [`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md)
(frozen), [`pr-fg-c-lm-summarise.md`](pr-fg-c-lm-summarise.md) (frozen, lands alongside) · **Prior art:**
[`docs/references/PRIOR_ART_2026-10.md`](../../../docs/references/PRIOR_ART_2026-10.md) PA-10, PA-11; `DEP-46`
· **Followed by:** P7 (Milestone D).

**Decision numbers.** `overall.md`'s table reserves `ARC-72`, `ARC-73` and `ARC-74` for S10 P4 and P6; P4
took `ARC-59`, so all three are P6's to propose: `ARC-72` (the LM controller's composition and the
proposal path), `ARC-73` (failure containment: layered retry, breaker, deadline, fallback by seat handback),
`ARC-74` (the `persona` System Pack). No new `DEP` number: P6 adds no dependency; its declined candidates
are a dated note under `DEP-44` (§4). The primary session confirms the numbers at freeze.

---

## 0. Binding inputs, verbatim or exactly restated

| Source | Ruling |
| --- | --- |
| step-17 §9 P6, 2026-10-11 (PA-10) | The controller is built from small typed parts: persona, memory view, choice prompt and parser. The model's free text is a **proposal** that the server validates. Concordia stays reference-only |
| step-17 §9 P6, 2026-10-11 (PA-11) | One bounded retrieval-count knob beside the 6 000-byte retrieval budget (`ARC-59`) |
| step-17 §9 P6, 2026-10-12 (DOC-24) | P6 depends on FG-b, which merges first. P6's routine policy becomes the seat's fallback model, named in `decisions.yaml` |
| step-17 §9 P6, 2026-10-13 (1) | **Admission gate (hard):** a model qualifies for a seat only if it produces schema-valid JSON on the first attempt for at least 38 of the 40 fixed spike scenarios (`tools/model_spike.py`), whichever way it gets there |
| 2026-10-13 (2) | Runtime-enforced structured output is preferred, not required. On by default where the backend supports it; a model whose runtime refuses it may qualify with `structured_output = "none"` if it passes the same gate |
| 2026-10-13 (3a) | Every answer is validated against the schema before it becomes an `ActionIntent`; invalid answers never reach the server |
| 2026-10-13 (3b) | Retries are layered by failure class, each layer pluggable: (i) transport/server (429, 5xx, 529, timeouts, resets) with exponential backoff and jitter, honouring `Retry-After`; (ii) content (invalid JSON, schema mismatch, refusal text) re-asked with the validation error, a few times; (iii) semantic (a valid proposal the owning System refuses) re-planned once with the refusal reason. All bounded by the per-consult deadline, the P5 budgets and a total-attempts cap. Layers live in the controller host above the P5a gateway, which never retries; every retry is its own recorded call. `RetryPolicy` is a typed interface with bundled `exponential-backoff`, `fixed`, `none`; selected and parameterised per world, seat and backend through configuration; users may write their own. Seeded jitter, injectable clock and sleep, cassette-replayed calls. Proposed defaults: transport ≤ 3 attempts, 0.5–8 s backoff, ±50 % jitter; content ≤ 2 re-asks; semantic 1 re-plan; ≤ 5 calls per consult |
| 2026-10-13 (3c) | When the layers are exhausted or a bound is hit, the decision for that consult is made by the seat's DL-a fallback model (`motivated` or `rule`); the world clock never waits for a model |
| 2026-10-13 (3d) | A circuit breaker per seat: if the invalid-or-failed share over the last N consults exceeds a threshold (proposal N = 20, 20 %), the seat runs on its fallback for a cooldown (proposal: one simulated hour) |
| 2026-10-13 (3e) | A per-consult wall-clock deadline; timeout, disconnect or budget refusal → fallback at that consult |
| 2026-10-13 (3f) | A valid but unreasonable proposal is refused by the owning System with a typed reason; the controller treats a refusal like a failed answer for (c) and (d) |
| 2026-10-13 (4) | Per-seat counters (invalid answers, retries per class, fallbacks, breaker trips, timeouts, server refusals) go to the controller's own log, never to world facts. Acceptance includes a fallback bound (proposal ≤ 5 % over a 100-day replayed run) and a scripted backend returning 30 % garbage, proving the world's facts stay valid and every consult resolves |
| 2026-10-13 (5) | Thresholds marked "proposal" are fixed at P6's freeze with the operator |
| P3b Q-P3b-4 (operator, 2026-10-10) | P6 persists the `resume` secret beside the memory store, rewritten after every rejoin |
| P3b F-P3b-3/4/5 | `based_on` records `Seen.connection`; ingestion starts before anything else awaits on the seat; `open()` is retried at start-up |
| P5a R-P6-1 | Models only through `Router.for_tier`; every `GateOutcome` handled; `None`, `Refused`, `Failed` are fallbacks; `CassetteMiss` stops the run; freeze needs the spike report |
| P4 R-P6-2 | One store per seat at `store/<instance>/<seat>.sqlite`; names learned from observations; context memory from `retrieve(...).render()`; `recalls` validated against `MemorySection.citations()` |
| FG-b R-P6-FG-1/2 | `await absorb(...)` for every batch; `PassReport`s in the decision log; each LM seat's `MemoryPolicyChoice` from DL-a's JSON, `resolve`d at seat start, refusing to start on a resolution error |
| FG-c R-P6-FGc-1 | `Router.for_tier("summarize")` passed to `resolve` for an assisted policy |
| DL-a §11 | P6 adds `external "lm"` to `cognition/installed` and turns AC-DL-5 into its own test |
| QS10-19 (operator, 2026-10-08) | Local models only for every gate; agents never use the operator's key; cassettes a gate depends on come from a local model |
| QP5-3 (operator) | The operator runs the spike personally; agents never run, pull or download a model |
| Platforms (operator, 2026-10-08) | macOS, Linux and Windows |

### 0.1 Spike results, operator-run 2026-10-13 (recorded here as evidence; E-P6-spike)

Run by the operator with `cognition/lm-controller/tools/model_spike.py` (40 fixed scenarios, `SpikeChoice`
schema, sha256 of the scenario file in each report). Not re-run by any agent.

| Runtime / backend | Model | `structured_output` | First-attempt schema-valid | p50 | p95 | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Ollama 0.40.2 (local) | `qwen3.5:4b` (nvfp4 build) | `json_schema` | — (every call HTTP 501, "structured output is unavailable") | — | — | runtime refuses enforced output |
| Ollama 0.40.2 (local) | `qwen3.5:4b` (nvfp4 build) | `none` | **39 / 40** | 2.6 s | 4.8 s | **admitted; default local model** |
| OpenAI API | `gpt-4.1-nano` | `json_schema` | **40 / 40** | 0.6 s | 0.9 s | **admitted; recommended API option** |
| OpenAI API | `gpt-5-nano` | `json_schema` | 0 / 40 (40 × HTTP 400) | — | — | **INCONCLUSIVE**: the request was refused before the model answered; the claim was not exercised |

Findings from the spike, recorded:

- **F-P6-S1.** An admission is a property of the triple (runtime version, model build, `structured_output`),
  not of a model name: `qwen3.5:4b` fails under `json_schema` on this runtime and passes under `none`. P6's
  README records admitted triples with their report's scenario hash (§5.15).
- **F-P6-S2.** `gpt-5-nano` answered HTTP 400 forty times. A reasoning model rejecting a parameter P5a's
  OpenAI-compatible adapter always sends is the likely cause. Two candidates, **neither verified here**: the
  `temperature` field (the `openai` preset sends it; `temperature = "omit"` already exists as a per-backend
  setting) and the `max_tokens` field (the adapter always sends `max_tokens`; OpenAI's reasoning models are
  documented elsewhere as expecting `max_completion_tokens`). This is a provider-adapter finding for P5a's
  preset table, not a P6 change (QP6-11).
- **F-P6-S3.** `model_spike.py` prints one JSON line (`{"default": …, "verdict": …}`); the per-model numbers
  are only in `report.json`. P6 adds a human-readable summary table (§5.15, commit C14).
- **F-P6-S4.** The gate measures `SpikeChoice` (`act: say | wait | leave`), not P6's `SocialChoice`
  (§5.2.5). The gate stays as ruled (fixed scenarios; changing them after a run is a material stop under P5a
  §11). P6's own schema validity is measured separately: deterministically under scripted garbage (AP6-4)
  and by the operator's live session (AP6-14, Gate 1).

---

## 1. Goal, in one paragraph

Make a seat that a world's decision list assigns to `lm` decide its **social** choices — what to say, to
whom, and which offered affordance to attempt — through a language model, while the world stays exactly as
valid as with every model removed. The controller is a Python Controller Pack that joins its seat over the
public protocol (`ARC-56`), takes the seat over from the in-server fallback model that DL-a resolved
(`took_over: "hosted"`), feeds its perceived facts into the FG-b-bounded memory store, and for each trigger
assembles a deterministic request from small typed parts (persona, situation, memory view, choice prompt),
asks the `social` tier through the P5 gateway, parses and validates the answer into a **proposal**, and
builds an `ActionRequest` only from what the newest observation offers. Every failure class has its own
bounded, pluggable retry layer; a per-consult deadline, the P5 budgets and an attempts cap bound them all; a
per-seat circuit breaker stops a failing model; and whenever the model cannot decide, the seat is handed
back to its DL-a fallback model, which the server drives in-process. Nothing in the kernel, the contracts,
the server or any System changes.

---

## 2. Scope

### 2.1 In scope

P6 is proposed as **two PRs** (QP6-1, recommendation: split), designed together here:

**P6a — the `persona` System Pack (Rust)**

1. `systems/persona/` (`mineworld-persona`): owns a `persona:` section in people files (bounded strings),
   a `Persona` component disclosed to its holder only, no action, no process (step-17 §3.8.5, QS10-7).
2. One line in `systems/installed`; `persona` appended to `worlds/social-cafe/world.yaml`'s `systems:`;
   `persona:` sections for social-cafe's nine non-player seats.
3. `docs/MODULE_SPEC.md` §4.1 (the `persona:` section); `docs/DECISIONS.md` `ARC-74`.

**P6b — the LM controller (Python, plus one Rust line)**

4. `cognition/installed`: `external "lm"` (DL-a SD-DLa-12), and its guard test.
5. `cognition/lm-controller/src/mineworld_cognition/controller/` (new subpackage): the typed parts, the
   proposal path, the failure classifier, `RetryPolicy` and `CircuitBreaker` with bundled implementations
   and entry-point registries, the consult engine, triggers, the seat runner, resume-secret persistence,
   the decision log and counters, the assignment reader (§5.1).
6. `mineworld_cognition/__main__.py`: `python -m mineworld_cognition --config FILE`.
7. `config.py` extended (R-P6-1): `server`, `invite_env`, `nickname`, `store`, `decisions_json`,
   `[controller]`, `[retry.transport|content|semantic]`, `[breaker]`, `[seats.<key>]` overrides (§5.13).
8. Two additive P5a changes (QP6-6): `BackendFailure.retry_after_s` (set by both HTTP adapters from a
   delta-seconds `Retry-After`), and an optional `timeout_s` on `ModelGateway.complete` so a deadline-cut
   call is charged like a timeout (§5.7).
9. `cognition/lm-controller/examples/local.toml` (the default local binding, `qwen3.5:4b`,
   `structured_output = "none"`), `examples/alice-lm.decisions.yaml` (an operator decision list for
   `--decisions`), README sections.
10. `tools/model_spike.py`: a human-readable summary table (F-P6-S3).
11. Specifications before code: `docs/MODULE_SPEC.md` §5 (the LM Controller paragraph), `docs/DECISIONS.md`
    `ARC-72`, `ARC-73`, dated notes on `ARC-57` (gateway `timeout_s`, `retry_after_s`) and `DEP-44`
    (declined retry and breaker libraries).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Milestone D's scenario test, the `AC-4` swap (IC-6), removability (IC-7), the 100-day store assertion, the 100-day replayed fallback bound | P7 |
| Routine (agenda walking, invitations) while the LM process holds the seat | Not in P6 (QP6-3): while it holds the seat, an LM seat acts socially only; routine is the fallback model's, which drives the seat whenever it is handed back. An "idle handback" is a follow-up |
| `OllamaNativeBackend` | not needed: the spike's admitted local triple uses `/v1` with `structured_output = "none"` (QS10-3 stays closed) |
| A fix for `gpt-5-nano`'s HTTP 400 | a P5a preset/adapter follow-up after the operator's discriminating re-run (QP6-11) |
| Per-world `lm` parameters in `decisions.yaml` | later (QP6-5); DL-a passes external models no parameters today |
| Running `mineworld decisions` from the cognition process | not in P6 (QP6-4): the operator writes its JSON once |
| A moderation `Filter` | stays reserved and empty (QS10-13) |
| Embedding retrieval | later (QS10-10) |
| Any change to `kernel/`, `contracts/`, `server/`, `persistence/`, any System except the new `persona`, `PROTOCOL.md` | never in P6 (I-1, I-2) |
| A Concordia adapter | after Milestone D, under its own design (DEP-46) |

### 2.3 Dependencies

| Dependency | State on 2026-10-14 | What P6 needs from it |
| --- | --- | --- |
| P3, P3b (SDK, `ResumingSeat`) | merged | the seat, the perceived stream, `Seen`, typed `SeatLost` |
| P4 (memory) | merged | `MemoryStore`, `retrieve`, `NameBook.learn`, `MemorySection.citations()` |
| P5a, P5b (gateway, adapters, recorder, budgets) | merged | `Router.for_tier`, `GateOutcome`, `ScriptedBackend`, `ReplayBackend`, `SqliteLedger` |
| S11-B (exclusivity, holds, takeover), S11-C (perceived stream), S11-D (admin) | landed (`PROTOCOL.md` §§4.2, 5.8) | `took_over: "hosted"`, `leave` returns a seat to its default at once, `resume` |
| **DL-a** | frozen, **not merged** | `external` kind, fallback chains, `mineworld decisions --json` (format 1), `server --town --decisions` driving the fallback |
| **FG-b** | frozen, **not merged** | `absorb`, `resolve`, `MemoryPolicyChoice`, `DEFAULT_CHOICE`, the 1 MiB bound |
| FG-c | frozen, lands alongside | assisted `resolve(choice, registry, summarize=…)`; whichever of FG-c and P6 merges second wires R-P6-FGc-1 |
| P6a | this design | P6b reads `Persona` when present and works without it (§5.2.1) |

---

## 3. Audit (`origin/main @ 171ea06`)

### 3.1 Source anchors

| Item | Where | What matters for P6 |
| --- | --- | --- |
| `ModelGateway.complete(seat, request) -> GateOutcome` | `cognition/lm-controller/src/mineworld_cognition/gateway.py` | budget pre-check → in-flight slot → backend under `asyncio.timeout(call_timeout_s)` → charge. **No retry.** A call cancelled from outside is never charged (the charge follows the await) — F-P6-1 |
| `BackendFailure(reason, status)` | `backend/model.py` | reasons `unreachable`, `timeout`, `http_status`, `malformed_response`, `unauthorized`; **no `Retry-After`** — F-P6-2 |
| `OpenAICompatibleBackend.complete` | `backend/openai_compatible.py` | maps 401/403 → `unauthorized`, non-2xx → `http_status`; always sends `max_tokens`; sends `temperature` when `temperature = "send"` (F-P6-S2) |
| `ScriptedBackend(script, delay_s)` | `backend/scripted.py` | a pure function of the request; counts calls — the garbage test's backend |
| `Cassette`, `ReplayBackend`, `RecordingBackend` | `record.py` | strict in-order replay by key; a miss raises; failures are never recorded |
| `BudgetPolicy` | `budget.py` | `calls_per_wall_hour 20`, `tokens_per_wall_day 30 000`, `max_in_flight 2`, `call_timeout_s 20`; wall-keyed (QS10-18) |
| `CognitionConfig`, `build_router` | `config.py` | `recording`, `tiers`, `budgets`, `backends`, `secrets`; `extra="forbid"`; refuses a config inside a world directory |
| `PRESETS` | `backend/providers.py` | `openai` sends `temperature` |
| `model_spike.py` | `tools/` | prints one JSON line; `SpikeChoice` schema; thresholds fixed (38/40, p95 ≤ 15 s) |
| `MemoryStore`, `retrieve`, `RecallQuery(at, counterparts, words)`, `MemorySection(lines)`, `citations()`, `BUDGETS` | `memory/` | sections L3 16 lines, L2 4, L1 8, L0 48; 160 B per line; 6 000 B ceiling; L1 lines arrive ranked |
| `NameBook.learn(entity, name, source)` | `memory/names.py` | the call P6 makes from observations |
| `ResumingSeat.open(connector, seat, invite, nickname, cursor, take_over, resume, policy, sleep, clock)` | `sdk/python/src/mineworld_sdk/resuming.py` | `perceived()` once per seat; `changed(after) -> Seen(connection, frame, perceived_through)`; `submit -> Outcome`; `leave()` gives the seat back; `detach()` leaves it held; `open()` makes one attempt (F-P3b-5) |
| `offers.attempt`, `offers.request` | `sdk/python/src/mineworld_sdk/offers.py` | build a request only from an available affordance; raise `NotOffered` otherwise |
| `Talk { utterance }`, `UTTERANCE_MAX_BYTES = 480`, empty refused | `systems/conversation/src/{action,utterance}.rs` | the `talk` payload is `{"utterance": "…"}`; bytes are UTF-8 bytes |
| seat states, `leave`, holds, `took_over` | `server/PROTOCOL.md` §4.2 | `leave` returns the seat to its default **at once**; a returned seat gets a controller **built afresh**; joining a `hosted` seat needs no flag (`took_over: "hosted"`); a dropped socket leaves the seat `held` for `--hold` (default 30 wall s) |
| `PacedSeat`, `--pace` default 5 wall s | `tools/cli/src/hosted.rs`, `main.rs` | the in-server fallback is consulted at most one pace (5 wall s by default) after it is built |
| decision list, `external`, `mineworld decisions --json` | `pr-dl-a-decision-list.md` §§5.3, 5.6, 5.9 | `{"seat": "alice", "model": {"id": "rule", "assigned": "lm", "chain": ["lm", "rule"]}, "memory": …}`; in `server --town` the in-server controller is the fallback; the external process takes over through S11-B |
| `MODULE_SPEC.md` §5.1 | `docs/MODULE_SPEC.md` lines 1040–1083 | "While its process is absent, the seat's host runs the fallback within one consult" |
| reserved decision ids | `overall.md` decision-number table | `ARC-72 … ARC-74` reserved for S10 P4/P6; none used in `DECISIONS.md` |
| `systems/installed` | `systems/installed/src/lib.rs` | `installed!` lines; a pack is one line plus a `Cargo.toml` line |
| self-only disclosure precedent | `systems/relationships` (`ARC-28` point 6), `systems/naming/src/system.rs` `discloses` | the shape `persona` copies |

### 3.2 Findings

| Id | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| F-P6-1 | Cancelling `ModelGateway.complete` from outside (to enforce a consult deadline) releases the reservation but never charges the ledger, so budgets would undercount | `gateway.py`: `ledger.charge` follows the awaited call; cancellation skips it | add an optional `timeout_s` to `complete` (effective `min(call_timeout_s, timeout_s)`); a deadline-cut call is then a `timeout` failure, charged one call and no tokens (D-P6-7) |
| F-P6-2 | No failure carries `Retry-After` | `BackendFailure` has `reason`, `status` only | add `retry_after_s: int | None` (0 … 3 600), set by both HTTP adapters on any non-2xx with a delta-seconds header; an HTTP-date is ignored. Failures are never recorded, so cassettes are unchanged (D-P6-8) |
| F-P6-3 | The in-server fallback only exists in `server --town` (DL-a SD-DLa-11). Without `--town`, a seat's default is `free` and `leave` makes Alice stand | `PROTOCOL.md` §4.2; DL-a §5.5 | handback only when the welcome said `took_over: "hosted"`; otherwise the fallback for a consult is "stand" (§5.8) |
| F-P6-4 | After a SIGKILL the seat is `held` for `--hold` (default 30 wall s) before its default returns; "the fallback within one consult" (`MODULE_SPEC.md` §5.1, AC-DL-5) therefore holds only after the hold | `PROTOCOL.md` §4.2 | AP6-8 measures "within `hold_seconds` + one pace + 2 s"; the operator chooses `--hold` (QP6-9) |
| F-P6-5 | A live hosted run is not reproducible at the request level: world instants follow wall time and client timing, so the observation and the memory text differ between runs, and a strict cassette misses | `runtime.rs` pacing (step-17 §2.3); P4 L1 text carries clock times | "cassette determinism" is defined over **logged decision inputs** replayed offline (§5.11); live integration tests use `ScriptedBackend` |
| F-P6-6 | A `decisions.yaml` inside social-cafe naming `alice: lm` would add a clause to `run`'s header and move `summary-300` | DL-a SD-DLa-14, §2.2 (`ci_parity.py` keeps the header) | P6 ships the assignment as an operator file under `cognition/lm-controller/examples/`, used with `--decisions`; no world file changes for the assignment |
| F-P6-7 | Enabling `persona` in social-cafe changes Alice's own observations (she perceives her component) and may change save bytes; step-17 I-7 says every existing transcript is unchanged by every PR of the step | step-17 §5 I-7 | P6a runs the baseline check; any moved column is a material stop for the operator's ruling (QP6-2) |
| F-P6-8 | P5a's spike schema is not P6's schema | `model_spike.py` `SCHEMA` | F-P6-S4 |
| F-P6-9 | Neither `RetryPolicy` nor `CircuitBreaker` has a second implementation outside P6, but the operator ruled both pluggable with bundled implementations; FG-b already defines an entry-point registry (`mineworld.forgetting`) | FG-b §5.3 | P6's registries copy FG-b's shape (group, id = entry-point name, strict `Parameters`, typed load errors); a shared loader is extracted only if FG-b's code already has one free of `ForgettingPolicy` (bounded decision in C7) |

### 3.3 What does not exist

No `controller/` subpackage, no `__main__.py`, no `context.py`, no decision log, no `persona` pack, no
`external` line, no `examples/local.toml`, no retry or breaker code anywhere in Python.

---

## 4. Reuse, in both directions (`REUSE_POLICY.md`; the operator's 2026-10-08 directive)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Model calls, budgets, recording | P5a's gateway, recorder, ledger (ours, merged) | **REUSE** unchanged except D-P6-7/D-P6-8 |
| Seat lifecycle, reconnect | P3b's `ResumingSeat` (ours, merged, DEP-44) | **REUSE** |
| Memory, retention | P4, FG-b, FG-c (ours) | **REUSE** |
| Retry with backoff | `tenacity` (Apache-2.0), `backoff` (MIT), `stamina` (MIT, wraps tenacity) | **REJECT; build** (~80 lines). DEP-44 already declined `tenacity` and `backoff` for the reconnect loop: they own the loop and the sleep, while our layers need per-class budgets shared under one deadline and attempt cap, seeded jitter, an injected clock, and a record per attempt. Licence facts for `stamina` were **not verified** in this planning session; C1 verifies them before writing the DEP-44 note |
| Circuit breaker | `pybreaker` (BSD-3-Clause), `aiobreaker`, `purgatory` | **REJECT; build** (~60 lines). A breaker over *consult outcomes* (content and semantic failures included, budget refusals excluded) with a cooldown in simulated time is not what these HTTP-call breakers count. Maturity and licences beyond `pybreaker`'s were **not verified**; C1 verifies them |
| Structured-output parsing and re-ask | Instructor | **REFERENCE ONLY** (step-17 §4.2): its re-ask idea, our code, every raw attempt recorded |
| Plug-in discovery | `importlib.metadata.entry_points` (stdlib), FG-b's registry shape | **REUSE** the stdlib and FG-b's conventions |
| Agent frameworks (Concordia, LangGraph, Letta, AI Town) | DEP-46, step-17 §4 | **REFERENCE ONLY / REJECT** as recorded |

No dependency is added (I-P6-9). Declining the retry and breaker libraries is recorded as a dated note on
`DEP-44` (C1), because `REUSE_POLICY.md` §12 requires a record when we build instead.

---

## 5. Design

### 5.1 Module layout (`cognition/lm-controller/src/mineworld_cognition/`)

```text
controller/
  __init__.py        public names only (listed in C5–C11)
  persona.py         Persona: decoded from the observer's own `persona` component; absent → Persona.empty()
  situation.py       Situation: my place, people here by unique display label, talk targets, numbered offers
  memory_view.py     MemoryView: retrieve() + the PA-11 knob `recall_episodes`
  prompt.py          ChoicePrompt: (Persona, Situation, MemorySection, Trigger, Feedback) → CompletionRequest
  choice.py          SocialChoice (Pydantic) and SOCIAL_CHOICE_SCHEMA; parse_choice(text)
  validate.py        validate(choice, situation, memory) → Proposal | ContentFailure
  intent.py          Proposal → ActionRequest | SemanticFailure, through mineworld_sdk.offers only
  failures.py        FailureClass and classify(); the typed failure records
  retry.py           RetryPolicy protocol; exponential-backoff, fixed, none; registry `mineworld.retry`
  breaker.py         CircuitBreaker protocol; failure-rate, never; registry `mineworld.breaker`
  consult.py         consult(ConsultInput, ConsultDeps) → ConsultRecord: the layered engine
  triggers.py        T1 (spoken to), T3 (a remembered person entered my place); one in flight + dirty
  log.py             DecisionLog (JSON Lines, bounded rotation), DecisionInput, Counters, replay_log()
  resume_file.py     read / write (atomic) / delete the seat's resume secret
  assignment.py      read DL-a's `mineworld decisions --json` (format 1): lm seats, memory choices
  seat.py            SeatRunner: the seat's lifecycle and its state machine (§5.8)
__main__.py          python -m mineworld_cognition --config FILE
config.py            extended (§5.13)
```

Dependency direction inside the package: `seat → consult → {prompt, validate, intent, retry, breaker,
failures}`; `seat → {triggers, log, resume_file, assignment, memory, gateway}`. No module under
`controller/` imports a backend adapter, `httpx2`, a preset, or a provider name (I-10, AP6-12). Every file is
expected under ~300 lines; `seat.py` and `consult.py` are the two to watch (the ~500-line review trigger).

### 5.2 The typed parts (PA-10)

Each part is a frozen Pydantic model (`CognitionModel`) or a pure function. None reads a clock, a random
source, the environment, a save or another seat's store (I-3).

#### 5.2.1 `Persona`

```python
class Persona(CognitionModel):
    summary: str = ""            # ≤ 280 bytes
    traits: tuple[str, ...] = () # ≤ 8, each ≤ 32 bytes
    speech: str = ""             # ≤ 160 bytes
    @staticmethod
    def of(observation: Observation) -> Persona   # the observer's own `persona` component, strictly
                                                   # decoded; absent or undecodable → Persona() and a
                                                   # counted `persona_absent` note in the log
```

The component's JSON is known **by name** (as P4's renderers know payloads); a world without `persona`
yields an empty persona and the prompt uses the display name alone. The bounds repeat P6a's so a bad value
cannot reach a prompt even from a foreign server.

#### 5.2.2 `Situation`

```python
class Person(CognitionModel):
    label: str                    # unique within the situation: "Bob", or "Bob (2)" on a name collision
    entity: EntityId              # never rendered to the model
class Offer(CognitionModel):
    number: int                   # 1-based, in observation order among available complete affordances
    action_type: ActionTypeId
    target_label: str | None
    position: int                 # index into observation.affordances (re-located by intent.py)
class Situation(CognitionModel):
    me: str                       # my display label
    place: str                    # place label (NameBook) or "somewhere"
    at: int                       # observation.at (world seconds)
    present: tuple[Person, ...]   # people in my place, sorted by (label, entity)
    talk_to: tuple[str, ...]      # labels with an AVAILABLE talk affordance targeting them
    offers: tuple[Offer, ...]     # available complete affordances only (ARC-34)
    @staticmethod
    def of(observation: Observation, names: NameBook) -> Situation
```

- Names are learned into the store from the observation's `display-name` components first
  (`NameBook.learn`, R-P6-2); labels come from the `NameBook`.
- Collisions: equal display names are suffixed ` (2)`, ` (3)` in ascending `EntityId` order — deterministic,
  and mapped back to ids only through this `Situation`.
- `talk_to` reads the server's verdict (`available`) and computes nothing (step-17 §3.6 layer 2).

#### 5.2.3 `MemoryView` (PA-11)

`MemoryView.of(store, query, recall_episodes) -> MemorySection`: calls P4's `retrieve(store, query)` (which
FG-b makes forgetting-aware) and keeps every L3, L2 and L0 line and **the first `recall_episodes` L1 lines**
(they arrive ranked: counterparts first, then lexical matches). `recall_episodes` is an integer `0 … 8`,
default 8 (no change from P4). It is the one retrieval-count knob PA-11 asks for (AI Town's
`NUM_MEMORIES_TO_SEARCH`), set per seat in `cognition.toml` (§5.13). Fewer lines can only shrink the section,
so the 6 000-byte ceiling (`ARC-59`) still holds, and `citations()` is computed on the kept lines, so
`recalls` can only cite what was shown. `RecallQuery`: `at` = the observation's `at`; `counterparts` = the
trigger's speaker plus everyone present; `words` = the trigger's quoted words.

#### 5.2.4 `ChoicePrompt`

`ChoicePrompt.build(persona, situation, memory, trigger, feedback, sampling) -> CompletionRequest`, a pure
function. Message layout, fixed:

```text
system     who I am (display name, persona summary, traits, speech style); the rules (answer with one
           JSON object; choose only among the people and offers listed; never invent people, events or
           numbers; cite memory only by the tokens shown); the JSON contract written out in words,
           always — so a backend with structured_output = "none" still receives it
user       WHERE I AM: place, people here, who I can talk to, numbered offers
           WHAT I REMEMBER: memory.render() (citation tokens such as [#1890-1907])
           WHAT JUST HAPPENED: the trigger, heard words quoted between <<<heard>>> … <<<end>>>
(re-ask)   assistant: the previous raw answer, cut to 2 048 bytes at a character boundary
           user: "That answer was not usable: <reason code>: <detail>. Answer again with one JSON object."
(re-plan)  user: "The world refused your choice: <rejection code>. Choose again from what is listed."
```

- **Injection hygiene.** Heard text has every occurrence of the delimiter tokens `<<<` and `>>>` replaced
  by `‹‹‹` and `›››` before quoting; heard words are data, never instructions (step-17 §3.13).
- `output_schema` = `SOCIAL_CHOICE_SCHEMA` on every request; whether the backend enforces it is the
  backend's `structured_output` setting, never the prompt's concern (ruling (2)).
- `sampling` = `Sampling(temperature_milli=700, max_output_tokens=256, seed=…)` with
  `seed = int.from_bytes(sha256(f"{seat}:{trigger_event_id}").digest()[:4], "big")` — no RNG, and the same
  trigger always yields the same seed (as the spike's 700/256 settings).
- The request holds no float, no entity id, no provider concept (I-10); its canonical JSON is the cassette
  key (ARC-58).

#### 5.2.5 `SocialChoice` and its schema

```python
class SocialChoice(CognitionModel):           # strict, frozen, extra="forbid"
    act: Literal["say", "attempt", "nothing"]
    to: str | None                            # a label from Situation.talk_to, when act = say
    words: str                                # what I say, when act = say; "" otherwise
    offer: int | None                         # an Offer.number, when act = attempt
    recalls: list[str]                        # citation tokens or ids shown in WHAT I REMEMBER; ≤ 8
```

`SOCIAL_CHOICE_SCHEMA` lists all five properties as required, `additionalProperties: false`, `to` and
`offer` nullable, `recalls` an array of strings with `maxItems: 8`, `words` with `maxLength: 480` (a
character bound for the model; bytes are checked locally). It is generated once from the model and
committed as a golden JSON file so a change is reviewed (C6).

#### 5.2.6 The parser and the validator (`choice.py`, `validate.py`)

`parse_choice(text) -> SocialChoice | ContentFailure`, then `validate(choice, situation, memory) -> Proposal
| ContentFailure`. Every failure has a typed `reason` (the re-ask's feedback and a counter key):

| Reason | Rule |
| --- | --- |
| `not_json` | after stripping surrounding whitespace and **at most one** enclosing Markdown code fence (```` ``` ```` or ```` ```json ````), the text is not one JSON object |
| `schema` | Pydantic strict validation fails (Pydantic owns the field mechanics; no per-field unit tests, test rules §1) |
| `truncated` | the completion's `finish` is `length` |
| `refused` | the completion's `finish` is `refused`, or `act` cannot be read and the text is non-empty prose |
| `unknown_person` | `act = say` and `to` is not in `situation.talk_to` |
| `empty_words` / `words_too_long` | `act = say` and `words` is empty, or more than 480 UTF-8 bytes (never truncated) |
| `unknown_offer` | `act = attempt` and `offer` is not an `Offer.number` |
| `forged_recall` | a `recalls` entry is not a token shown, nor an id inside a range of `memory.citations()` |
| `leak` | `words` contains a delimiter token or a citation token (`[#…]`) |

Fields that do not belong to the chosen `act` are ignored, not refused (a recorded normalisation that keeps
garbage-free answers from failing on noise). A `Proposal` is `Say(to: EntityId, words: str, recalls)`,
`Attempt(offer: Offer, recalls)` or `Nothing(recalls)` — ids, never labels.

### 5.3 The proposal → `ActionIntent` path

```text
trigger ─► ConsultInput (persona, situation, memory, trigger, based_on) ─► ChoicePrompt ─► gateway
   ─► Completion ─► parse_choice ─► validate ─► Proposal
   ─► intent.build(proposal, NEWEST observation)
        Say      → offers.request(obs, "talk", target=to, payload={"utterance": words})
        Attempt  → offers.attempt(obs, obs.affordances[offer.position]) after re-locating the offer in
                   the newest observation by (position, action_type, target); a moved offer → NotOffered
        Nothing  → no request (a valid decision; counted, not a failure)
   ─► pre-submit checks (step-17 §3.6 layer 2): the consult deadline has not passed; no newer line from
      the trigger's speaker arrived after based_on (else: drop and re-decide once, on the newest state)
   ─► seat.submit(request) ─► Answered(Accepted) | Answered(Rejected|Unavailable) | RefusedRequest
```

- The only ways a model's text reaches the world are a `talk` utterance and the unchanged payload of a
  complete affordance the server offered (`ARC-34`). A model that "decides" anything else produces no
  representable proposal (`INV-10`).
- `NotOffered` from `offers` and a server `Rejected`/`Unavailable` are **semantic failures** (ruling (3f)).
- The world revalidates every request at dispatch regardless (step-17 §3.6 layer 1); P6's checks only
  avoid sending what is already known to fail.
- No `move`, `walk-step` or other movement request is ever built by `controller/` (I-15, AP6-11).

### 5.4 Triggers and the consult loop

| Trigger | From | Handling |
| --- | --- | --- |
| T1 spoken to | a perceived `spoke` fact whose listener (`subjects`) is me and whose speaker is not me | a social consult |
| T3 a remembered person entered my place | a perceived `arrived` / `person-entered-place` fact for a person with an L3 stable fact, while I am in that place | a social consult, at most once per counterpart per `greet_cooldown_s` world seconds (default 1 800, proposal) |

- Trigger detection reads the batch **after** `absorb` committed it (memory first, R-P6-FG-1).
- **One consult in flight per seat.** A trigger arriving meanwhile marks the seat dirty; when the consult
  ends, one more consult runs on the newest trigger (step-17 §3.4.5). Triggers older than the consult
  deadline are dropped and counted (`stale_triggers`).
- T2 (invitations), T4 (agenda), T5 (heartbeat) are not P6's (QP6-3). An invitation that arrives while a
  consult runs is visible to the model as a numbered offer if the group-activity pack offers it complete.

### 5.5 Failure classes and the layered retry (`failures.py`, `retry.py`, `consult.py`)

#### 5.5.1 Classification (fixed table, one function, unit-tested as a table)

| Observed | Class | Retryable |
| --- | --- | --- |
| `Failed(timeout)`, `Failed(unreachable)`, `Failed(malformed_response)` | transport | yes |
| `Failed(http_status)` with 408, 409, 425, 429, 500, 502, 503, 504, 529 | transport | yes |
| `Failed(http_status)` any other code; `Failed(unauthorized)` | transport | **no** → fallback now |
| `Refused(BudgetRefusal)` | budget | no → fallback now (ruling (3e)); not a breaker failure |
| `Completed` but `parse_choice`/`validate` fails | content | yes |
| `NotOffered` at intent building; `Answered` with `Rejected` or `Unavailable` | semantic | yes |
| `RefusedRequest` (a frame refusal such as `paused`) | frame | no; not a breaker failure (not the model's fault) |
| `NotConnected`, `AnswerLost`, `SeatLost` | seat | no; the consult ends `disconnected` (ruling (3e)); not a breaker failure |
| `CassetteMiss` | — | never an outcome: it propagates and stops the process (R-P6-1) |

#### 5.5.2 `RetryPolicy`

```python
class RetryDecision(CognitionModel):
    retry: bool
    delay_s: float = 0.0            # ≥ 0; the engine sleeps through the injected `sleep`

class RetryPolicy[P: CognitionModel](Protocol):
    id: ClassVar[str]               # ^[a-z][a-z0-9-]{0,47}$, equal to its entry-point name
    Parameters: ClassVar[type[P]]   # every field has a default
    def decide(self, attempt: int, failure: FailureRecord, params: P,
               jitter: random.Random) -> RetryDecision: ...
```

- `attempt` counts this class's attempts in this consult, from 1. `jitter` is a `random.Random` seeded by
  the engine from `(seat, consult number, class)` through sha256 — reproducible, never global.
- Registry: `RetryRegistry.installed()` loads entry points of group `mineworld.retry`; `bind(id, params,
  source)` validates strictly; unknown id and invalid parameters are typed errors naming the config key
  (FG-b §5.3's conventions). A user's own policy is a distribution exposing one entry point.

Bundled policies (each one class, registered under the package's own entry points):

| Id | Parameters (defaults) | Behaviour |
| --- | --- | --- |
| `exponential-backoff` | `attempts 3`, `first_s 0.5`, `factor 2`, `ceiling_s 8.0`, `jitter 0.5` | retry while `attempt < attempts`; delay = `min(ceiling, first × factor^(attempt−1))` scaled by a factor drawn from `[1 − jitter, 1 + jitter]`; if the failure carries `retry_after_s`, delay = `max(delay, retry_after_s)` |
| `fixed` | `attempts 1`, `delay_s 0.0` | retry while `attempt < attempts`, after `delay_s` |
| `none` | — | never retries |

`attempts` counts tries including the first; the content layer's "2 re-asks" is `fixed` with `attempts 3`,
and the semantic layer's "1 re-plan" is `fixed` with `attempts 2`. Parameter bounds: `attempts 1 … 10`,
delays `0 … 60` s, `jitter 0 … 1`.

#### 5.5.3 The engine (`consult.py`)

```python
@dataclass(frozen=True)
class ConsultDeps:
    ask: Callable[[CompletionRequest, float], Awaitable[GateOutcome]]   # gateway.complete(seat, r, timeout_s=)
    newest: Callable[[], Seen]                                           # the seat's newest observation
    submit: Callable[[ActionRequest], Awaitable[Outcome]]
    clock: Callable[[], float]                                           # monotonic wall seconds
    sleep: Callable[[float], Awaitable[None]]
    policies: LayerPolicies                                              # bound transport/content/semantic
    limits: ConsultLimits                                                # deadline_s, max_calls

async def consult(input: ConsultInput, deps: ConsultDeps) -> ConsultRecord
```

Loop, in prose: build the request; ask with `timeout_s = remaining deadline`; classify; on a retryable
failure ask that class's policy; a retry happens only if the policy says so **and** `calls < max_calls`
**and** `delay_s` fits in the remaining deadline; a content retry rebuilds the request with the re-ask
feedback; a semantic retry rebuilds it from the **newest** observation with the re-plan feedback; anything
else ends the consult. Outcomes (`ConsultRecord.outcome`):

```text
acted        a request was submitted and Accepted
chose        the proposal was Nothing (valid)
fell_back    layers exhausted, a non-retryable failure, the cap, or the deadline → §5.8
budget       the gateway Refused → §5.8
disconnected the seat ended or reconnected mid-consult → §5.8
dropped      a newer line from the same speaker arrived: re-decide once (not a failure)
```

Every attempt is appended to `ConsultRecord.attempts` with its class, cassette key, typed outcome, and —
for a completion — the raw text (the model's own words; no provider concept). Every model call is one
gateway call, one ledger charge and, in record mode, one cassette entry (ruling (3b)).

### 5.6 The circuit breaker (`breaker.py`)

```python
class CircuitBreaker[P: CognitionModel](Protocol):
    id: ClassVar[str]
    Parameters: ClassVar[type[P]]
    def observe(self, outcome: BreakerOutcome, params: P) -> BreakerState: ...   # after each counted consult
    def state(self) -> BreakerState: ...                                          # closed | open(until_s) | half_open
```

- `BreakerOutcome` is `success` (`acted`, `chose`) or `failure` (`fell_back` from content, semantic or
  transport exhaustion, a non-retryable transport failure, or the deadline). `budget`, `disconnected`,
  `dropped` and frame refusals are **not counted** (none is the model's fault).
- Bundled `failure-rate`: `window 20`, `min_consults 10`, `threshold_permille 200`, `cooldown_world_s 3 600`.
  It opens when the window holds at least `min_consults` and failures × 1 000 > `threshold_permille` ×
  window length. While open the seat is handed back (§5.8) for `cooldown_world_s / time_scale` wall seconds
  (`welcome.world.time_scale` read at the trip; recorded). Then **half-open**: the seat rejoins; the next
  consult is a probe — success closes (window cleared), failure re-opens for the same cooldown.
- Bundled `never`: always closed (for tests and for operators who want per-consult fallback only).
- Registry group `mineworld.breaker`, same conventions as §5.5.2.

### 5.7 Deadlines and budgets

| Bound | Default | Where enforced | Status |
| --- | --- | --- | --- |
| consult deadline | 20 wall s from the trigger's receipt to the submit | `consult.py`; it also replaces step-17's `max_decision_age` (one knob, not two) | **proposal** |
| per-call timeout | `min(budgets.call_timeout_s = 20, remaining deadline)` | the gateway's new `timeout_s` (D-P6-7) | P5a default + D-P6-7 |
| calls per consult | 5 | `consult.py` | **proposal** (ruling 3b) |
| transport attempts | 3, backoff 0.5 → 8 s, ±50 % jitter | `exponential-backoff` | **proposal** |
| content attempts | 3 (2 re-asks), no delay | `fixed` | **proposal** |
| semantic attempts | 2 (1 re-plan) | `fixed` | **proposal** |
| breaker | window 20, min 10, > 20 %, cooldown 3 600 world s | `failure-rate` | **proposal** |
| handback | 10 wall s (two default paces) | `seat.py` | **proposal** |
| greet cooldown (T3) | 1 800 world s | `triggers.py` | **proposal** |
| per seat calls / tokens | 20 per wall hour / 30 000 per wall day | P5a ledger, unchanged | P5a (QP5-4) |

Why 20 s: the admitted local model's p95 is 4.8 s, so a deadline of 20 s fits a first attempt plus two
re-asks at p95 with room for one transport backoff; the default `call_timeout_s` is also 20 s. Interplay
recorded as risk R-P6-4: with retries, the P5a default of 20 calls per wall hour admits as few as four
fully-retried consults an hour.

### 5.8 Fallback: handing the seat back to its DL-a fallback model

**The mechanism (D-P6-2).** The in-server controller of an `lm` seat under `server --town` *is* the seat's
DL-a fallback model (DL-a SD-DLa-12). The LM process holds the seat while the model is healthy. When a
consult must fall back (`fell_back`, `budget`, `disconnected`), or the breaker opens, and the seat's welcome
said `took_over: "hosted"`, the runner sends `leave`: the server returns the seat to its default **at once**,
with a fallback controller built afresh, which is consulted within one pace (`PROTOCOL.md` §4.2; `--pace`
default 5 wall s). The runner deletes the resume file (a `leave` kills the secret), waits the handback
period (10 wall s, or the breaker's cooldown), then joins again with its memory store's cursor: it is
welcomed `took_over: "hosted"`, receives every fact it missed from its cursor (P3b), and resumes.

```text
             start: open(take_over=False, resume=<file or None>)
                         │ welcome
                         ▼
   ┌──────────────► HOLDING ── consult ok ──► HOLDING
   │                     │
   │   fell_back / budget / disconnected / breaker open
   │                     ▼
   │     took_over was "hosted"? ── no ──► HOLDING (fallback = stand: no request this consult)
   │                     │ yes
   │                     ▼
   │                 leave; delete resume file
   │                     ▼
   │                 HANDED_BACK (handback_s, or the breaker's cooldown) ── fallback model drives
   │                     │
   └──── rejoin (cursor = store.cursor(); breaker half-open if it was open)
```

- **Why not decide the fallback in Python?** The fallback models are Rust (`rule`, DL-b's `motivated`);
  re-implementing them in Python would be a second implementation of a policy that DL-a just made
  pluggable, and step-24 §8.6 names the server's hosted fallback for exactly this. The handback is host
  state: nothing is journaled and no fact is written (`PROTOCOL.md` §4.2, last paragraph), so the world's
  determinism and history are untouched.
- **Cost.** One `leave`, one reconnect and one backfill per fallback, on loopback typically milliseconds.
  With the content defaults and independent 30 % garbage per attempt, the expected share of consults that
  fall back is 0.3³ = 2.7 % (AP6-4).
- **A seat that is not hosted** (`server` without `--town`, or a chain ending in `none`): `leave` would only
  make the person stand while losing the triggers, so the runner stays, records `fallback: stand` and sends
  nothing for that consult (F-P6-3).
- **A killed process.** No `leave` is sent; the seat is `held` for `hold_seconds`, then returns to its default
  (F-P6-4). A restart inside the hold rejoins with the persisted secret (§5.10).
- **A human takes the seat** (`take_over`): the runner receives `SeatLost("taken_over")`, deletes the resume
  file, and retries joining every 60 wall s (`seat_retry_s`, proposal); `seat_occupied` keeps it waiting
  until the human leaves (R-S11-4, `AC-5`). The world never waits for it.

### 5.9 Seat assignment through DL-a's decision list, and the memory policy through FG-b

1. The operator writes a decision list (or uses `examples/alice-lm.decisions.yaml`):

   ```yaml
   models:
     lm: { fallback: rule }        # DL-a SD-DLa-8: an external model must name a fallback
   assign:
     - { seat: alice, model: lm }
   ```

2. The server: `mineworld server worlds/social-cafe --town --decisions <file> --save <dir>` — Alice is driven
   by `rule` until the LM process joins.
3. The assignment: `mineworld decisions worlds/social-cafe --decisions <file> --json > seats.json` (DL-a
   SD-DLa-15, `format: 1`).
4. The cognition process: `python -m mineworld_cognition --config cognition.toml`, whose `decisions_json`
   names `seats.json`. `assignment.py` reads it with strict Pydantic models (an unknown `format` is refused),
   selects every seat whose `model.assigned == "lm"`, and refuses to start when none is (`no lm seat in
   <file>`). An optional `seats = [...]` in `cognition.toml` narrows the selection and must be a subset
   (else refused by name).
5. Per seat, `memory` (or `DEFAULT_CHOICE` when `null`) goes to FG-b's `resolve` with the policy registry and,
   when FG-c is present, `Router.for_tier("summarize")` (R-P6-FG-2, R-P6-FGc-1). A resolution error refuses
   to start that seat, naming the file, line and column from `source`.
6. At each welcome, the runner checks `welcome.world.id` equals the JSON's `world`; a mismatch ends the seat
   with a named error. The server's resolved list cannot be read over the wire, so a different `--decisions`
   on the server and in step 3 is detectable only by its effect — recorded as R-P6-7.

The Rust side gains one line, `external "lm"` in `cognition/installed` (DL-a §5.6), with no other Rust change
in P6b.

### 5.10 Resume-secret persistence (Q-P3b-4)

- Path: `store/<instance>/<seat>.resume`, beside `<seat>.sqlite`; content: the 32 lowercase hex characters
  of the current `welcome.resume`, nothing else.
- **Written** after every welcome — the first, and whenever `Seen.connection` increases — by writing
  `<seat>.resume.tmp` then `os.replace` (atomic on all three platforms). On POSIX the file is created with
  mode `0o600`; on Windows it inherits the operator's profile ACLs (documented, as P5a's `env_file`).
- **Read** at start: when present and well-formed it is passed as `resume=` to `ResumingSeat.open`; the SDK
  retries without it on `invalid_resume` (row 7), so a stale file costs nothing.
- **Deleted** on `leave` (handback, clean shutdown) and on `SeatLost`, because the secret is dead.
- Never logged, never in a `repr`, never in the decision log (I-16; the SDK's `ResumeSecret` already hides
  itself). AP6-9 plants a known secret and scans every artefact.

### 5.11 Cassette determinism

- **What is deterministic.** `consult` is a pure function of `(ConsultInput, the gateway's answers, the
  submit outcomes, the seeded jitter)`. Given the same inputs, it builds byte-identical `CompletionRequest`s
  (same cassette keys) and identical `ActionRequest`s. Prompt assembly reads no clock and no randomness;
  the sampling seed is derived from `(seat, trigger id)`.
- **What is logged.** Each consult's `DecisionInput` — persona, situation, the kept `MemorySection`, the
  trigger, `based_on (connection, seq, perceived_through, at)` — and, for each attempt, the cassette key, the
  class, the typed outcome, the re-plan's refreshed situation, and the submit outcome.
- **Replay (`log.replay_log(log, cassette) -> ReplayReport`).** Re-runs every logged consult through
  `consult` with a `ReplayBackend` over the cassette, a `submit` that returns the logged outcomes, and a
  clock that never expires; it asserts every key and every `ActionRequest` equals the logged ones. This is
  AC-DL-5's "under cassette replay its requests are identical", tested with no server and no model (AP6-8).
- **What is not deterministic, by design (F-P6-5).** Two live hosted runs differ in timing, so their contexts
  differ; a strict cassette therefore replays a *logged* run, not a fresh live one. Live integration tests use
  `ScriptedBackend` (a pure function of the request), which needs no re-recording when timing moves. P7
  inherits this as R-P7-P6-1 (§10).
- Transport failures are never recorded (P5a); transport-retry paths are tested with `ScriptedBackend`
  returning `BackendFailure`s.

### 5.12 Observability

Per seat, `Counters` (a frozen snapshot type) with these fields, all non-negative integers:

```text
consults, acted, chose_nothing, invalid_answers (content failures, per attempt, by reason),
retries_transport, retries_content, retries_semantic, fallbacks (by cause: transport, content, semantic,
deadline, budget, disconnected, cap, stand), breaker_trips, timeouts (call and deadline), server_refusals
(Rejected/Unavailable by code), frame_refusals, stale_triggers, dropped, handbacks, model_calls
```

- Written to the seat's decision log as a `counters` record every 50 consults and at shutdown, and printed
  as one status line per seat at shutdown (`[cognition] alice: 212 consults, 205 acted, …`), as the server
  prints its seat statistics. Never a world fact, never journaled (ruling (4)).
- `model_calls` equals the ledger's call count for the seat over the run (AP6-5 checks it).
- The decision log: `store/<instance>/<seat>.decisions.jsonl`, typed records (`DecisionRecord`, `Counters`,
  `PassReport` summaries from `absorb`), rotated at 8 MiB to `.1` (two files kept: a hard bound of 16 MiB per
  seat). It is not inside the SQLite memory store, so FG-b's 1 MiB bound is unaffected.

### 5.13 Configuration (`cognition.toml`, extending `CognitionConfig`)

```toml
server         = "ws://127.0.0.1:7878/ws"
invite_env     = "MINEWORLD_INVITE"      # the NAME of the variable holding the server's invite
nickname       = "cognition"
store          = "cognition-state"        # relative to this file; never inside a world directory
decisions_json = "seats.json"             # mineworld decisions … --json
# seats        = ["alice"]                # optional subset of the lm seats in decisions_json

[controller]
consult_deadline_s = 20
max_calls          = 5
handback_s         = 10
seat_retry_s       = 60
recall_episodes    = 8                    # PA-11
greet_cooldown_s   = 1800                 # world seconds

[retry.transport]
policy = "exponential-backoff"
parameters = { attempts = 3, first_s = 0.5, factor = 2, ceiling_s = 8.0, jitter = 0.5 }
[retry.content]
policy = "fixed"
parameters = { attempts = 3 }
[retry.semantic]
policy = "fixed"
parameters = { attempts = 2 }

[breaker]
policy = "failure-rate"
parameters = { window = 20, min_consults = 10, threshold_permille = 200, cooldown_world_s = 3600 }

[seats.alice.controller]                  # optional per-seat overrides of [controller]
recall_episodes = 6
[seats.alice.retry.content]               # optional per-seat override of one layer
policy = "fixed"
parameters = { attempts = 2 }

[backends.local.retry.transport]          # optional per-backend override of one layer
policy = "none"
```

- **Precedence**, most specific first: `[seats.<key>.retry.<class>]` > `[backends.<name>.retry.<class>]` (the
  backend bound to `social`) > `[retry.<class>]` > the bundled default. Per world means per configuration
  file; per seat and per backend are the tables above (ruling (3b)). `decisions.yaml` carries no retry
  setting in P6 (QP6-5).
- Every new key is typed and `extra="forbid"`; `invite_env` follows `key_env`'s rule (a variable name, never a
  value). The invite is read from the environment only (R-S11-5), never from a file this package writes.
- Bounds: `consult_deadline_s 1 … 120`, `max_calls 1 … 10`, `handback_s 1 … 600`, `seat_retry_s 1 … 3 600`,
  `recall_episodes 0 … 8`, `greet_cooldown_s 0 … 86 400`.

### 5.14 P6a — the `persona` System Pack

- Section (people files): `persona: { summary: <str ≤ 280 B, non-empty>, traits: [<str ≤ 32 B>] ≤ 8,
  speech: <str ≤ 160 B> }`; `traits` and `speech` optional; unknown keys refused with line and column (the
  `configure/` and section precedents, `DEP-10`).
- Component `Persona` with the same bounds; disclosed through `PerceptionProvider::discloses` **to its holder
  only** (`ARC-28` point 6's rule, as relationships do). No action, no process, no reaction.
- Whether seeding emits a genesis fact is decided by following `naming`'s precedent exactly (A-P6-1); if it
  does, its `Visibility` is `Entities({holder})`.
- `systems/installed`: `Persona => mineworld_persona::PersonaSystem`. social-cafe: `persona` appended last to
  `systems:` (so no earlier reducer order moves) and sections for `alice`, `bob`, `carol`, `dev`, `erin`,
  `felix`, `grace`, `hana`, `ivan` — authored from each file's existing `note` and tags, inventing no world
  rule (R-S10-9: persona shapes speech only).
- `MODULE_SPEC.md` §4.1: the section, its owner, its bounds, "shapes speech, grants nothing"; `ARC-74`.
- Baselines: `python3 scripts/ci_parity.py baseline check` on a long record of the final head. If any
  social-cafe column moves, that is a **material stop** for QP6-2's ruling; no baseline is edited without it.

### 5.15 The spike tool's summary table

`model_spike.py` gains `summary_table(report) -> str`, a pure function, and `main` prints the table before
the existing JSON line (which stays last, unchanged, for scripts):

```text
model                 output       valid  done  failures         p50 ms  p95 ms  meets
qwen3.5:4b            none          39/40  40    -                 2600    4800  yes
gpt-4.1-nano          json_schema   40/40  40    -                  600     900  yes
gpt-5-nano            json_schema    0/40   0    http_status:400×40    -       -  no
default: qwen3.5:4b   criteria: valid ≥ 38/40, p95 ≤ 15000 ms   scenarios sha256 <first 12 hex>
```

The README gains a table of admitted triples (runtime, model, `structured_output`, report hash, date),
starting with the two admitted on 2026-10-13. No runtime admission check is added (QP6-10).

### 5.16 Platforms

- `__main__` runs `asyncio.run(main(config))` on the platform's **default** loop (the proactor on Windows),
  so a P5b subscription bridge can start a subprocess there (F-P5b-1). Tests keep P5a's selector loop for
  the network guard, except the bridge tests that already run the default loop.
- Stopping: Ctrl-C (`KeyboardInterrupt`) on every platform, and the server closing the socket; no
  `SIGTERM` handler and no `add_signal_handler` (Windows has neither). On Ctrl-C every seat sends `leave`
  within 5 s, deletes its resume file and prints its counters.
- Paths through `pathlib`; the resume file's `0o600` applies on POSIX only; `os.replace` on all three.
- `examples/local.toml` uses relative paths valid everywhere; README lists per-platform locations as P5a's
  does.

---

## 6. Invariants this PR must hold

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-P6-1 | No change under `kernel/`, `contracts/`, `server/`, `persistence/`, `PROTOCOL.md`, or any System Pack other than the new `systems/persona` (step-17 I-1, I-2) | `git diff --stat` in C15 and A3 |
| I-P6-2 | An answer reaches the world only as a validated `talk` utterance or an unchanged complete-affordance payload built through `mineworld_sdk.offers` (ruling 3a) | AP6-1; a source scan that `controller/` builds `ActionRequest` only in `intent.py` |
| I-P6-3 | A model decides only social choices; no movement request is built by `controller/` (step-17 I-15) | AP6-11 |
| I-P6-4 | The world never waits for a model: every consult ends within its deadline plus 1 s, and a held or handed-back seat is driven by the server's fallback | AP6-2, AP6-7, AP6-8 |
| I-P6-5 | Every model call is one gateway call: one ledger charge and, when recording, one cassette entry; P5a's gateway still never retries | AP6-5 |
| I-P6-6 | Memory is ingested before the cursor advances and before triggers are read (R-P6-FG-1, F-P3b-4) | AP6-15; ordering assertion in the runner's test |
| I-P6-7 | The world never reads cognition state; deleting every file under `store/` changes no byte of any save (step-17 I-6) | AP6-15 |
| I-P6-8 | No provider concept in `controller/`, the decision log, the store, a cassette key or a request; no secret (key, invite, resume) in any artefact (step-17 I-10, I-16) | AP6-12 |
| I-P6-9 | No new dependency | `uv.lock` diff empty in C15 |
| I-P6-10 | Consult logic is deterministic given its inputs (seeded jitter, injected clock and sleep, derived sampling seed) | AP6-8 replay half; AP6-5 |

---

## 7. Test ownership (test rules §26)

| Layer | Owns |
| --- | --- |
| **Static** | ruff, ruff-format, pyright strict over the new subpackage (Literal misuse, protocol conformance of bundled policies, `Proposal` exhaustiveness in `match`); cargo fmt/clippy for `systems/persona` and the `cognition/installed` line. No unit test duplicates a Pydantic field check |
| **Unit (pure, no server)** | failure classification table; `exponential-backoff` delay sequence against hand-computed values for a fixed seed; `Retry-After` overrides; deadline cut; breaker transitions; `parse_choice`/`validate` reason table (one table-driven test, every reason, plus valid cases); injection neutralisation; `Situation` from the golden `server/tests/frames/observation.json` (labels, collisions, numbering); `MemoryView` knob and citation consistency; `ChoicePrompt` golden request (canonical JSON hand-reviewed, test rules §25); `intent` building against the golden `submit.json` shape; registries (unknown id, bad parameters, entry-point id mismatch); config precedence and refusals; DL-a JSON reader refusals; resume-file atomic write/read/delete; decision-log rotation; `summary_table` golden string; the consult engine over scripted `ask`/`submit` sequences for every outcome; the seeded 30 % garbage run of 1 000 consults (AP6-4) |
| **Integration (real binary, `real_binary` marker)** | the seat runner against `mineworld server --town --decisions`: takeover, reply, handback and rejoin, kill and fallback (AP6-8), resume within the hold (AP6-9), two processes on one seat (I-14), IC-3 revalidation under delay (AP6-16), the 30 % garbage world-validity scenario (AP6-3), I-5/I-6 (AP6-15); persona disclosure to self only (A2, Rust integration test over `observe`) |
| **Replay** | `replay_log` over a log and cassette written by a scripted integration run (AP6-8 replay half) |
| **Gate 1 (real LLM)** | operator-run only (QP5-3, QS10-19): a bounded live session with the admitted local model (AP6-14). Agents never run it. Recorded NOT RUN until the operator does |
| **Gate 2 (real lifecycle)** | owned by the real-binary integration tests above (they run the real server and real sockets); the 100-day replayed run is P7's |
| **CI impact** | `cognition/lm-controller/**` → the `python` job (all three platforms) and the `fast` static checks; `systems/persona`, `systems/installed`, `cognition/installed` → the selector's full fallback for Rust; one canonical PR CI per PR |

Mutations (planted, seen red, reverted; recorded in §15):

| Id | Mutation | Must turn red |
| --- | --- | --- |
| M-P6-1 | `validate` skips the `forged_recall` check | the reason table test; AP6-1 (a garbage line citing a forged id reaches the log as `acted`) |
| M-P6-2 | the engine ignores `max_calls` | AP6-2's per-consult call bound |
| M-P6-3 | the content layer retries forever (policy `attempts` ignored) | the deadline/cap test and AP6-4's call count |
| M-P6-4 | transport backoff ignores `retry_after_s` | the Retry-After unit test |
| M-P6-5 | the breaker counts `budget` outcomes as failures | the breaker exclusion test |
| M-P6-6 | the runner does not `leave` on fallback (stays and sends nothing) | AP6-8: the fallback model's request never appears for Alice after a forced failure |
| M-P6-7 | the resume file is written once and never rewritten | AP6-9 (restart after a rejoin is refused `seat_occupied`) |
| M-P6-8 | `intent.py` submits a `Say` without `offers.request` (bypasses the verdict) | AP6-16 IC-3's local-drop assertion turns into a server rejection count mismatch, and the scan in I-P6-2 |
| M-P6-9 | the prompt includes an entity id | AP6-12's scan |
| M-P6-10 | triggers are read before `absorb` commits | the runner ordering test |

---

## 8. Acceptance criteria (fixed before anything is measured, `ARC-23`)

Thresholds marked **(proposal)** are fixed by the operator at freeze (ruling 5); the rest follow from the
design.

| Id | Criterion | Counterfactual |
| --- | --- | --- |
| **AP6-1** | In the garbage scenario (AP6-3), no submitted request carries a payload that failed validation: every Alice `spoke` fact's utterance is either a valid scripted line or a line the fallback model produces (its template or greetings); zero utterances contain a garbage marker | M-P6-1 |
| **AP6-2** | Every consult ends with exactly one terminal outcome; no consult makes more than `max_calls` (5) model calls; no consult lasts longer than `consult_deadline_s` + 1 s wall **(proposal: 1 s slack)** | M-P6-2 |
| **AP6-3** | **The world stays valid** under a scripted backend that returns garbage on 30 % of attempts (seeded; garbage drawn from the reason table's kinds) plus 5 % transport failures: `mineworld inspect` on the save resolves every cause; the save resumes; `mineworld replay` reproduces every snapshot; every consult in the decision log resolves (count of terminal records = count of consults) | M-P6-6 (fallback never engaged) |
| **AP6-4** | In the seeded unit run of 1 000 consults with independent 30 % content garbage per attempt and the default layers, the fallback share is ≤ 5 % **(proposal)** (expected 2.7 %), and every consult resolves | M-P6-3 |
| **AP6-5** | Retry layering: each class retries only through its own policy; seeded jitter reproduces the same delay sequence; `retry_after_s` lengthens a delay and a `Retry-After` beyond the remaining deadline ends the consult at once; `Counters.model_calls` equals the ledger's charges for the seat | M-P6-4 |
| **AP6-6** | Breaker: with `failure-rate` defaults it opens on the consult that makes failures exceed 20 % of a window of ≥ 10, hands the seat back for `3 600 / time_scale` wall s, probes once half-open, and never counts `budget`, `disconnected`, `dropped` or frame refusals **(proposal: N 20, 20 %, cooldown one simulated hour)** | M-P6-5 |
| **AP6-7** | Deadline: a scripted backend that delays 30 s makes the consult fall back by `consult_deadline_s` + 1 s, and the visitor's own requests during that time are answered by the server as usual | — |
| **AP6-8** | **AC-DL-5.** (kill) With the cognition process SIGKILLed, Alice's seat is driven by her fallback model within `hold_seconds` + one pace + 2 s (a request from Alice by the in-server controller appears); (replay) `replay_log` over a scripted integration run's log and cassette reproduces every cassette key and every `ActionRequest` | M-P6-6 |
| **AP6-9** | Resume: restarting the cognition process inside the hold rejoins with the persisted secret (`took_over: "held"`), after at least one earlier rejoin (so the rewrite is exercised); the secret, the invite and a planted fake key appear in no log, decision log, store or cassette | M-P6-7 |
| **AP6-10** | Assignment: only seats whose resolved model is assigned `lm` are joined; an unknown `format`, a `seats` entry that is not an `lm` seat, a world id that differs from the welcome, and a FG-b resolution error each refuse to start with a message naming the file (and line and column where FG-b gives them) | — |
| **AP6-11** | No request built under `controller/` has action type `move` or `walk-step` (source scan of `intent.py` plus an integration assertion over the scenario's journal for Alice's LM-driven intervals) | — |
| **AP6-12** | The provider-concept scan (P5a's `test_provider_scan.py`, extended to `controller/`, the decision log and the store schema) finds no provider or model name, URL or key; no entity id in any `CompletionRequest` of the scenario | M-P6-9 |
| **AP6-13** | Observability: the shutdown status line and the last `counters` record agree; their fields equal the counts recomputed from the decision log's consult records | — |
| **AP6-14** | **Gate 1, operator-run (NOT RUN by agents):** with `examples/local.toml` (`qwen3.5:4b`, `structured_output = "none"`), a live session of ≥ 100 consults: P6-schema validity on the first attempt ≥ 95 % **(proposal)**, fallback share ≤ 5 % **(proposal)**, and the operator's judgement that Alice's replies read as natural speech | — |
| **AP6-15** | I-5 and I-6: with the store deleted and the seat re-joined from a null cursor, the structural store dump is byte-identical; deleting every file under `store/` changes no byte of the save; triggers are never read before `absorb` returns | M-P6-10 |
| **AP6-16** | IC-3: a scripted backend answering after 3 s while the visitor walks out of talk range yields no Alice `spoke` after the departure in the save, and the decision log shows a local `NotOffered` drop or a `TooFarAway` refusal; removing P6's pre-submit check alone still passes (the world's check holds) | M-P6-8 |
| **AP6-17** | Two cognition processes configured for Alice: the second is refused `seat_occupied` and retries at `seat_retry_s`; Alice is never driven by both (I-14) | — |
| **AP6-18** | `model_spike.py` prints the summary table before the unchanged JSON line; `summary_table` equals a hand-written golden for a fixture report | — |
| **AP6a-1** | `persona:` refusals (unknown key, over-long summary, nine traits, empty summary) name file, line and column | — |
| **AP6a-2** | Alice's observation discloses her `Persona` component to her and to no other observer (two-observer integration test) | a provider that discloses to everyone |
| **AP6a-3** | `ci_parity.py baseline check` on the final head: every column unchanged, **or** the operator's QP6-2 ruling is recorded with each moved column and its reason line | — |
| **P7-owned** | the 100-day replayed fallback bound (≤ 5 % **(proposal)**), Milestone D, IC-6, IC-7, the 1 MiB store at day 100 | — |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run (test rules §§8–9). Commands run from the worktree
root. Python commands: `uv run --locked ruff check cognition/lm-controller`, `uv run --locked ruff format
--check cognition/lm-controller`, `uv run --locked pyright`, and the cognition suite as `uv run --locked
pytest cognition/lm-controller/tests/<file>` (a single path, so the member's own `addopts` and network guard
apply, P5a D-P5-11). Every commit is small enough for one implementer session; each lists exact files.

### P6a — the `persona` System Pack

#### A0 — Freeze and contract (Markdown only)

- **Goal / scope.** Start P6a against the frozen design: this file's ledger (§15), `handoff-s10-p6a.md`.
- [ ] Implementation: verify the `DESIGN FROZEN` header and §12; record the base commit; initialise the
  handoff with every field the working rules require.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: no authority line widened; QP6-1/QP6-2 rulings copied verbatim.
- **Boundary.** Documentation only.

#### A1 — Specification before code

- **Goal.** `CLAUDE.md` §2.2: the section and its owner are specified first.
- **Scope.** `docs/MODULE_SPEC.md` §4.1 (`persona:`), `docs/DECISIONS.md` `ARC-74` (number confirmed at
  freeze).
- [ ] Implementation: the §4.1 entry (owner, bounds, disclosure to self, "shapes speech, grants nothing",
  R-S10-9) and `ARC-74` (problem, options: a pack / controller configuration / `note`, choice, consequences).
- [ ] Validation: both doc checks.
- [ ] Review: terminology (`System Pack`, `Component`, `Person`); no rule only in a README.

#### A2 — `systems/persona`: section, component, self-only disclosure

- **Goal.** The pack exists and discloses to its holder only.
- **Scope.** `systems/persona/{Cargo.toml, README.md, src/lib.rs, src/section.rs, src/component.rs,
  src/system.rs, tests/persona.rs}`; root `Cargo.toml` members.
- [ ] Implementation: mirror `systems/naming`'s structure for the section and seeding (A-P6-1 decided by
  reading `naming` first and recording what it does); bounds of §5.14; `discloses` returns the component
  only when the observer is the holder.
- [ ] Validation: section refusals table (AP6a-1) with line/column; a two-observer integration test over
  `mineworld_presence::observe` (AP6a-2), mutation "disclose to everyone" seen red and reverted; `cargo test
  -p mineworld-persona`; `cargo clippy -p mineworld-persona --all-targets -- -D warnings`.
- [ ] Review: no action, process or reaction; no other pack's vocabulary; `HashMap` absent.
- **Failure cases.** Over-long values are refused at load, never truncated.

#### A3 — Install and author social-cafe's personas

- **Scope.** `systems/installed/{Cargo.toml, src/lib.rs}`; `worlds/social-cafe/world.yaml` (append
  `persona`); nine people files.
- [ ] Implementation: one installed line; `persona` appended **last**; sections from each file's `note` and
  tags.
- [ ] Validation: `cargo test -p mineworld-installed-systems` (the installed guard test; crate name confirmed
  from its `Cargo.toml`); `target/debug/mineworld validate worlds/social-cafe`; `cargo build --release -p
  mineworld-cli`; `python3 scripts/ci_parity.py record --binary target/release/mineworld --profile long --out
  <scratch>`; `python3 scripts/ci_parity.py baseline check <scratch>` — classify PASS/FAIL from its verdict
  lines (AP6a-3). A moved column is a **material stop** (QP6-2).
- [ ] Review: no rule smuggled into persona text; reducer order of earlier systems unchanged.
- **Boundary.** Then push, open P6a's PR, one CI.

### P6b — the LM controller

#### C0 — Freeze and contract (Markdown only)

- [ ] Implementation: verify the header and §12; confirm DL-a and FG-b are merged (`git log origin/main
  --oneline | grep -i "dl-a\|fg-b"` and the presence of `cognition/decision` and `memory/retention`); record
  the base; initialise `handoff-s10-p6.md`.
- [ ] Validation: both doc checks.
- [ ] Review: every R-P6-* requirement of §0 is mapped to a commit below.

#### C1 — Specifications and decision records

- **Scope.** `docs/MODULE_SPEC.md` §5 (an LM Controller paragraph: protocol client; assignment through
  DL-a's JSON; fallback by handback; retry layers and breaker as configuration; the controller's own log);
  `docs/DECISIONS.md` `ARC-72`, `ARC-73`; dated notes on `ARC-57` (`timeout_s`, `retry_after_s`) and
  `DEP-44` (retry and breaker libraries declined, with verified licences).
- [ ] Implementation: the edits, citing this design.
- [ ] Validation: both doc checks.
- [ ] Review: `Controller`, `Controller Pack`, `ActionIntent` used exactly; the handback is described as host
  state, never as a world fact.

#### C2 — `external "lm"`

- **Scope.** `cognition/installed/src/lib.rs` (one `models!` line), its guard test.
- [ ] Implementation: `external "lm",` as DL-a §5.6 shows.
- [ ] Validation: `cargo test -p mineworld-installed-models`; `target/debug/mineworld decisions
  worlds/social-cafe --decisions cognition/lm-controller/examples/alice-lm.decisions.yaml --json` shows
  `"assigned": "lm"`, `"chain": ["lm", "rule"]` for alice (the example file is added in this commit).
- [ ] Review: no other Rust file changes; `run` without `--decisions` prints today's header (DL-a I-DLa-1).

#### C3 — Configuration and the assignment reader

- **Scope.** `config.py` (new keys of §5.13 and their models); `controller/assignment.py`; tests
  `tests/test_controller_config.py`, `tests/test_assignment.py` (with a literal format-1 JSON fixture and one
  `real_binary` test that parses the binary's own output).
- [ ] Implementation: typed models with `extra="forbid"` and the bounds of §5.13; precedence resolver
  `layer_policy(config, seat, backend, class) -> (id, parameters, source_key)`; `read_assignment(path) ->
  Assignment` (strict; `format == 1`; `lm` seats; memory choices as FG-b's `MemoryPolicyChoice`).
- [ ] Validation: refusals (unknown key, out-of-bound value, `invite_env` holding a value-like string,
  `store` inside a world directory, unknown `format`, `seats` not a subset, no `lm` seat); precedence table;
  the real-binary parse.
- [ ] Review: no YAML parsing in Python; error messages never echo a value (P5a `_describe`).

#### C4 — Two additive P5a changes

- **Scope.** `backend/model.py` (`retry_after_s`), `backend/openai_compatible.py` and
  `backend/anthropic_messages.py` (parse `Retry-After`), `gateway.py` (`timeout_s`); tests in the existing
  `test_openai_compatible.py`, `test_anthropic_messages.py`, `test_budget.py`.
- [ ] Implementation: `retry_after_s: int | None` bounded 0 … 3 600, set only from a delta-seconds header on a
  non-2xx answer; `complete(seat, request, *, timeout_s: float | None = None)` with `min(call_timeout_s,
  timeout_s)`.
- [ ] Validation: an `httpx2.MockTransport` 429 with `Retry-After: 3` → `retry_after_s == 3`; an HTTP-date
  → `None`; a scripted backend delayed past `timeout_s` → `Failed(timeout)` and one call charged, zero
  tokens; existing P5a tests unchanged and green.
- [ ] Review: no header value other than the integer reaches any type (AP5-9); no cassette format change.

#### C5 — Typed parts I: `Persona`, `Situation`, `MemoryView`

- **Scope.** `controller/{__init__.py, persona.py, situation.py, memory_view.py}`; `tests/test_controller_parts.py`.
- [ ] Implementation: §5.2.1–§5.2.3.
- [ ] Validation: `Situation.of` over `server/tests/frames/observation.json` (labels, collision suffixes on
  a doctored copy, offer numbering from available complete affordances only); `Persona.of` with and without
  the component, and with an over-long value (→ empty, counted); `MemoryView` with `recall_episodes` 0, 3, 8
  over a P4 fixture store: section bytes non-increasing, `citations()` equal to the kept lines' ranges.
- [ ] Review: no clock, randomness or environment read; names never ids.

#### C6 — Typed parts II: choice, prompt, validation, intent

- **Scope.** `controller/{choice.py, prompt.py, validate.py, intent.py}`;
  `tests/fixtures/social_choice.schema.json` (golden), `tests/fixtures/prompt_request.json` (golden);
  `tests/test_controller_choice.py`.
- [ ] Implementation: §5.2.4–§5.2.6 and §5.3's `intent.build`.
- [ ] Validation: the reason table (every reason of §5.2.6 and three valid answers) in one table-driven
  test; fence stripping accepts exactly one fence; injection: heard text containing `<<<end>>>` renders
  neutralised; prompt golden (hand-reviewed canonical JSON; test rules §25); the derived sampling seed for a
  fixed `(seat, trigger)` equals a hand-computed value; `intent.build(Say)` equals the golden `submit.json`
  request shape with `{"utterance": …}`; a target not offered → `SemanticFailure(not_offered)`; M-P6-1 and
  M-P6-9 planted and reverted.
- [ ] Review: no movement action type can be produced; the schema golden matches the model.

#### C7 — Failures and `RetryPolicy`

- **Scope.** `controller/{failures.py, retry.py}`; `pyproject.toml` entry points (`[project.entry-points."mineworld.retry"]`
  for the three bundled policies); `tests/test_controller_retry.py`.
- [ ] Implementation: §5.5.1 and §5.5.2; the registry copies FG-b's conventions (F-P6-9: first read FG-b's
  `memory/retention/registry.py`; extract a shared loader only if it is already independent of
  `ForgettingPolicy`, else mirror it; record which).
- [ ] Validation: the classification table; delay sequence for seed 0 against hand-computed values; jitter
  bounds; `Retry-After` override; unknown id and bad parameters refused naming the key; M-P6-4.
- [ ] Review: `attempts` semantics (tries including the first) consistent with §5.7's table.

#### C8 — `CircuitBreaker`

- **Scope.** `controller/breaker.py`; entry points `mineworld.breaker`; `tests/test_controller_breaker.py`.
- [ ] Implementation: §5.6.
- [ ] Validation: closed → open at the exact consult; `min_consults`; cooldown conversion with `time_scale`
  1 and 60; half-open success and failure; exclusions; M-P6-5.
- [ ] Review: the breaker reads no clock itself (the runner supplies time).

#### C9 — The consult engine

- **Scope.** `controller/consult.py`; `tests/test_controller_consult.py`.
- [ ] Implementation: §5.5.3 with `ConsultDeps` injected.
- [ ] Validation: one scripted sequence per outcome (acted, chose, each fell_back cause, budget,
  disconnected, dropped); cap and deadline; a semantic re-plan uses the newest situation; the seeded 1 000-
  consult garbage run (AP6-4: every consult terminal, fallback share ≤ the frozen threshold, calls ≤ 5
  each); M-P6-2, M-P6-3.
- [ ] Review: every model call goes through `deps.ask`; no sleep beyond the remaining deadline.

#### C10 — Decision log, counters, offline replay

- **Scope.** `controller/log.py`; `tests/test_controller_log.py`.
- [ ] Implementation: `DecisionInput`, `DecisionRecord`, `Counters`, JSONL writer with 8 MiB rotation,
  `replay_log(log_path, cassette_path)`.
- [ ] Validation: rotation at the bound with a small configured size; a scripted run's log replays with
  identical keys and requests through `ReplayBackend` (AP6-8 replay half, unit scale); counters recomputed
  from records equal the written counters (AP6-13 unit half); a planted key and resume secret absent.
- [ ] Review: the log holds the model's raw words but no provider concept; replay reaches no backend.

#### C11 — Triggers, resume file, seat runner

- **Scope.** `controller/{triggers.py, resume_file.py, seat.py}`; `tests/test_controller_seat.py` (an
  in-process fake connector built on the SDK's `Connection` protocol, as P3b's tests do).
- [ ] Implementation: §5.4, §5.8, §5.10: start the `absorb` task first (F-P3b-4); retry `open()` with the
  P3b `ReconnectPolicy` delays until `seat_retry_s` (F-P3b-5); `based_on` with `Seen.connection` (F-P3b-3);
  learn names from every new observation; the state machine of §5.8; rewrite the resume file on every new
  connection; delete it on `leave`/`SeatLost`.
- [ ] Validation: triggers T1/T3 from fixture facts; one in flight plus dirty; ingestion-before-trigger
  ordering (M-P6-10); handback when `took_over == "hosted"`, stand otherwise; resume file atomicity
  (`os.replace` observed by reading during write) and rewrite on rejoin (M-P6-7, unit half).
- [ ] Review: no world write path; the runner never constructs a backend.

#### C12 — The entry point and examples; integration over the real binary

- **Scope.** `__main__.py`; `examples/local.toml`, `examples/alice-lm.decisions.yaml` (from C2), README;
  `tests/test_controller_server.py` (`real_binary`).
- [ ] Implementation: `python -m mineworld_cognition --config FILE`; default loop; Ctrl-C handling (§5.16);
  status lines.
- [ ] Validation (each test starts its own `mineworld server worlds/social-cafe --town --decisions
  examples/alice-lm.decisions.yaml --save <scratch> --hold 3 --pace 1`, a protocol client for `visitor`, and
  the runner in `scripted` mode in-process): takeover (`took_over: "hosted"`) and a scripted reply as an
  Alice `spoke` caused by her seat's action; forced failure → handback → the fallback's request for Alice →
  rejoin; SIGKILL-equivalent (the runner's socket aborted without `leave`) → fallback within `hold` + pace +
  2 s (AP6-8 kill half); restart within the hold after one rejoin → `took_over: "held"` (AP6-9); second
  runner → `seat_occupied` (AP6-17); I-5 and I-6 (AP6-15). Wall time recorded; target ≤ 90 s for the file on
  Linux.
- [ ] Review: no test reaches beyond loopback (the guard); no model, no key.

#### C13 — The garbage scenario and IC-3

- **Scope.** `tests/test_controller_resilience.py` (`real_binary`), `tests/garbage.py` (the seeded garbage
  script).
- [ ] Implementation: a `ScriptedBackend` script returning valid answers, 30 % garbage across the reason
  kinds and 5 % `BackendFailure`s, seeded; a 3 s delayed script for IC-3.
- [ ] Validation: AP6-1, AP6-2, AP6-3 (`mineworld inspect`, a resume of the save, `mineworld replay`),
  AP6-11, AP6-12 over the scenario's artefacts, AP6-16; M-P6-6 and M-P6-8 planted and reverted. Target ≤ 90 s.
- [ ] Review: assertions read the save and the decision log, not the runner's memory.

#### C14 — Spike summary table and recorded spike evidence

- **Scope.** `tools/model_spike.py` (`summary_table`, `main`); `tests/test_spike_tool.py` (golden table);
  README's admitted-triples table; this file's §0.1 cross-referenced from P5a's ledger by a one-line note.
- [ ] Implementation: §5.15.
- [ ] Validation: golden string for a fixture report (AP6-18); `test_no_ci_command_runs_the_spike…` still
  green.
- [ ] Review: the JSON line is unchanged and last; no threshold constant moved.

#### C15 — Close-out

- **Scope.** this file (§15), `handoff-s10-p6.md`.
- [ ] Implementation: every §8 criterion with evidence or explicitly open (AP6-14 NOT RUN until the
  operator); I-P6-1 and I-P6-9 diffs recorded.
- [ ] Validation: static checks; `uv run --locked pytest cognition/lm-controller/tests` once locally only if
  a specific reason exists (test rules §8); otherwise the PR's CI.
- [ ] Review: deviations recorded; parent synchronisation marked pending for the primary session.
- **Boundary.** Documentation only; push, open the PR, one CI; READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-P7-P6-1 (on P7).** Milestone D's live scenario runs with `ScriptedBackend` or replays a **logged** run
  through `replay_log` (§5.11); a strict cassette cannot replay a fresh live run (F-P6-5). IC-6 (the `AC-4`
  swap) renames and repoints the binding under `replay_log`.
- **R-P7-P6-2 (on P7).** The 100-day fallback bound is measured over a recorded run's decision log with the
  counters of §5.12.
- **R-FGc-P6-1 (on FG-c or P6, whichever merges second).** Pass `Router.for_tier("summarize")` to `resolve`
  for an assisted policy (R-P6-FGc-1).
- **R-P5a-P6-1 (on a P5a preset follow-up).** Resolve F-P6-S2 after the operator's discriminating re-run
  (QP6-11).

## 11. Edits to parent documents (owner: the primary session, after freeze or merge)

| Document | Edit |
| --- | --- |
| `step-17-cognition.md` §15.2 P6 row | designed in this file; frozen / merged as … |
| `step-17-cognition.md` §3.4.6 | routine while an LM seat holds the seat is not P6's (QP6-3 outcome) |
| `step-17-cognition.md` §3.6 | `max_decision_age` replaced by the consult deadline |
| `overall.md` decision-number table | `ARC-72`, `ARC-73`, `ARC-74` used by P6 |
| `pr-s10-p5-backends.md` ledger | one line pointing to §0.1 for the spike's evidence |

---

## 12. Execution contract (DRAFT; settled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, S10 P6a (persona) and P6b (LM controller)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md
RELATED / BINDING DOCS:  step-17-cognition.md (§§3.4–3.16, §9 P6 row, §15); mvp1/step-24 (§§4.2, 5.3, 8.6,
                         AC-DL-5); pr-s10-p3b-perceived.md; pr-s10-p4-memory.md; pr-s10-p5-backends.md;
                         pr-s10-p5b-hosted-subscriptions.md; pr-dl-a-decision-list.md;
                         pr-fg-b-retention-host.md; pr-fg-c-lm-summarise.md; server/PROTOCOL.md §4.2;
                         docs/MODULE_SPEC.md §§4.1, 5; docs/DECISIONS.md ARC-56…60, ARC-84, ARC-85,
                         ARC-87, DEP-44, DEP-46; docs/references/PRIOR_ART_2026-10.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREES:               P6a: /Users/yuema137/mineworld-worktrees/impl-s10-p6a, branch
                         mvp0/pr-s10-p6a-persona; P6b: /Users/yuema137/mineworld-worktrees/impl-s10-p6,
                         branch mvp0/pr-s10-p6-lm-controller; each its own, one session each
                                                                  (source: unresolved — QP6-1)
START CONDITION:         P6a: freeze + the operator's go. P6b: freeze + DL-a and FG-b merged + the
                         operator's go                            (source: step-17 §9, DL-a §11, FG-b)
IMPLEMENTATION BASE:     origin/main at A0 / C0; recorded there
APPROVED SCOPE:          §2.1                                     (source: unresolved until freeze)
FROZEN INVARIANTS:       §0 rulings; §6 I-P6-1 … I-P6-10; §8 thresholds as the operator fixes them
SEQUENCE:                A0 … A3; C0 … C15
ALLOWED COMMANDS:        cargo *; uv run --locked *; git; gh (never merge); python3 scripts/*;
                         target/*/mineworld *; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; reading any key, invite or .env file;
                         running, pulling or downloading a model; any live or record mode; edits under
                         kernel/, contracts/, server/, persistence/, systems/ (except systems/persona and
                         one line in systems/installed in P6a); editing scripts/baselines.txt without
                         QP6-2's ruling
MATERIAL STOPS:          a moved baseline column (A3); any change to the protocol or the server; a new
                         dependency; a change to a §8 threshold after it has run once; a change to DL-a's
                         JSON or FG-b's MemoryPolicyChoice; a need to retry inside the P5a gateway
PLATFORMS:               Linux, macOS, Windows
VALIDATION BUDGET:       unit/integration/real-binary tests: unrestricted locally; real-binary test files
                         ≤ 90 s each on Linux; one long parity record in A3 (≤ 20 min); Gate 1: operator only
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 handoff-s10-p6a.md; handoff-s10-p6.md
ENDPOINT AUTHORITY:      implementation + local validation:   unresolved (operator's go at freeze)
                         semantic commits:                    unresolved (default authorized; §14)
                         branch push:                         unresolved (default authorized; §21)
                         PR creation / update:                unresolved (default authorized; §21)
                         CI repair to review readiness:       unresolved (default authorized; §21)
                         merge: explicit operator authorization only (working rules §22)
POST-MERGE SYNC OWNER:   primary session: step-17, overall.md; implementation session: this ledger
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-P6-1 | Handback churn makes Alice flicker between the LM's voice and the rule template | fallback share ≤ 5 % (AP6-4, AP6-14); the breaker turns repeated failure into one longer handback; the handback period is tunable |
| R-P6-2 | The fallback model, built afresh at the handback, answers the line the LM failed on with its template | that *is* the ruled behaviour ("the fallback decides that consult"); the template never claims a memory it lacks (QS10-9) |
| R-P6-3 | Live runs cannot be replayed from a cassette (F-P6-5) | logged-input replay (§5.11); scripted live tests; R-P7-P6-1 |
| R-P6-4 | P5a's default 20 calls per wall hour starves a busy LM seat once retries happen | budget refusal falls back cleanly and is counted; the operator raises the ceiling in `cognition.toml`; AP6-14 reports the observed calls per consult |
| R-P6-5 | `structured_output = "none"` models drift with runtime upgrades | admissions are per triple with a report hash (F-P6-S1); the README says to re-run the spike after a runtime upgrade |
| R-P6-6 | Real-binary tests push the `python` job past its ceiling (P4's AC-10 already uses 233–243 s of 300 s on Windows) | ≤ 90 s per file; `--pace 1 --hold 3`; small scenarios; recorded wall times |
| R-P6-7 | The server's `--decisions` and the cognition process's JSON differ | the world id check; the README shows one decision file used for both commands; a digest on the wire would be an S11 change (not proposed) |
| R-P6-8 | Persona in social-cafe moves baselines (F-P6-7) | A3's material stop; QP6-2 |
| R-P6-9 | A model's words claim memories it was not given though `recalls` validates | known limit of free text (R-S10-10); the operator's judgement in AP6-14 |
| R-P6-10 | Entry-point registries load arbitrary user code | as FG-b: loaded once at start, typed errors, operator-installed distributions only; no remote loading |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QP6-1 [primary]** | One PR or two: split the Rust `persona` pack (P6a) from the Python controller (P6b)? | **Two.** P6a is independent, small, can start now (it needs neither DL-a nor FG-b), and carries the only baseline risk; P6b then stays Python plus one Rust line, which suits small sequential implementer commits |
| **QP6-2 [operator — world content and I-7]** | If enabling `persona` in social-cafe moves a baseline column, may P6a re-baseline with a reason line (step-17 I-7 says this step changes no transcript)? | **Yes**, for the columns that move because Alice and the NPCs now perceive their own persona (world content, not a rule change), each with a reason line; any other movement stays a stop |
| **QP6-3 [operator — behaviour]** | While the LM process holds a seat, who does routine (walking to the agenda, invitations)? (a) nobody: the LM seat acts socially only, routine resumes whenever the seat is handed back; (b) a minimal Python routine port (QS10-8); (c) an "idle handback": hand the seat to the fallback whenever the person is away from their agenda's place and no conversation is under way | **(a) in P6**, (c) as a follow-up after Milestone D. (b) duplicates a policy DL-a just made pluggable. (a) is enough for Milestone D, where Alice works the counter |
| **QP6-4 [primary]** | How does the cognition process learn its seats: (a) a JSON file the operator writes with `mineworld decisions --json`; (b) the process runs the binary itself? | **(a).** One mechanism, no subprocess, works when cognition runs on another machine; the README shows the command |
| **QP6-5 [primary]** | Retry, breaker and controller knobs per world in `decisions.yaml` (needs DL-a to pass external parameters through), or in `cognition.toml` only? | **`cognition.toml` only in P6**, with per-seat and per-backend tables. Revisit when a World Pack author needs to ship a tuning |
| **QP6-6 [primary — P5a interface]** | Accept the two additive P5a changes: `BackendFailure.retry_after_s` and `ModelGateway.complete(..., timeout_s=)`? | **Yes.** Without the first, "honouring `Retry-After`" is impossible; without the second, deadline cuts are uncharged (F-P6-1, F-P6-2). Both are optional fields, change no cassette and keep the gateway retry-free |
| **QP6-7 [operator — thresholds]** | Fix the proposals: deadline 20 s; ≤ 5 calls; transport 3 attempts 0.5–8 s ±50 %; content 2 re-asks; semantic 1 re-plan; breaker 20 / min 10 / > 20 % / 3 600 world s; handback 10 s; greet cooldown 1 800 world s; fallback ≤ 5 % (AP6-4, AP6-14, P7); first-attempt P6-schema validity ≥ 95 % (AP6-14); consult slack 1 s | **Accept as listed.** They follow the ruled defaults and the spike (p95 4.8 s locally) |
| **QP6-8 [operator — behaviour]** | When the fallback cannot run in-server (no `--town`, or a chain ending in `none`), should a failed consult make the LM seat stand (send nothing), or speak a fixed deterministic phrase (QS10-9)? | **Stand.** A fixed phrase would be a second fallback model in Python; the operator gets the rule model's speech by running `--town` |
| **QP6-9 [operator — operations]** | `--hold` for LM seats: keep the server default (30 s; a crashed process leaves Alice standing up to 30 s, then the fallback), or recommend lower in the README? | **Keep 30 s** and say so in the README: the hold is what lets a restarted process resume with its persisted secret (Q-P3b-4) |
| **QP6-10 [operator — gate enforcement]** | Enforce the admission gate at runtime (refuse a model without a passing report), or keep it an operator procedure with a README table? | **Procedure plus table.** Runtime enforcement needs a trusted report format and adds a failure mode for users' own models; the gate binds what the project recommends and ships |
| **QP6-11 [operator — paid API]** | For `gpt-5-nano`'s HTTP 400: re-run the spike for that one model with `temperature = "omit"` to tell the two hypotheses apart (≈ 40 nano calls of your own key), then a small P5a preset/adapter follow-up? | **Yes, operator-run**, outside P6. If `omit` alone fixes it, the follow-up is a preset note; if not, an adapter option for the output-token field name. No agent uses the key (QS10-19) |
| **QP6-12 [primary]** | Decision numbers `ARC-72` (composition and proposal path), `ARC-73` (failure containment), `ARC-74` (persona) from S10's reservation, and no new DEP? | **Yes**; record them in `overall.md`'s table before the PRs merge |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| A0 … A3 | — | [ ] | [ ] | [ ] |
| C0 … C15 | — | [ ] | [ ] | [ ] |

### Evidence

- **E-P6-spike** (operator, 2026-10-13): §0.1. Not re-run by any agent.

### Assumptions to verify in implementation

| Id | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-P6-1 | `naming`'s seeding pattern (with or without a genesis fact) applies to `persona` unchanged | A2, by reading `systems/naming` first | follow whatever `naming` does and record it |
| A-P6-2 | DL-a's `server --town --decisions` drives an `lm` seat with its fallback and yields it to a joining connection with `took_over: "hosted"` | C2/C12 against the merged DL-a | a DL-a defect: stop and report to the primary session |
| A-P6-3 | FG-b's `absorb`/`resolve` signatures match FG-b §5.6/§6 as frozen | C0 against the merged FG-b | adapt the call sites; record |
| A-P6-4 | `leave` followed by a fresh join within milliseconds is accepted (`took_over: "hosted"`) with the backfill from the cursor | C12 | a server defect; stop |

### Mutations

(none run yet)

### Deviations and findings

(none yet; planning findings are §3.2 and §0.1)
