# Playable dialogue and persistent contextual memory — PR design

Lifecycle: **DRAFT — FREEZE-READY.** This is not DESIGN FROZEN. No operator approval exists yet.
Design revision: DM-20261127-2 (supersedes the recovered draft DM-20261123-1).
Execution contract: [`pr-playable-dialogue-memory-contract.md`](pr-playable-dialogue-memory-contract.md)
(filled; its pending values are the decisions below).
Parent requirement: [`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §§1–4 (check 2:
"a player speaks, receives a relevant response, and later can verify a remembered prior exchange after
leaving/rejoining and after save/restart"). This is a single-PR step of that requirement; no separate step
document is created, so this file is the one authority for the lane (CLAUDE.md §3.2).
Owners: planning — primary coordinator; drafting — design lane L3; execution — one fresh delegated session per
the contract; independent red-team — the coordinator's assigned reviewer.

## 0. Decisions for the operator (plain language)

Answer each with "yes as recommended" or a change. All five answers together freeze this design.

| # | Question | Recommendation (pending freeze) |
| --- | --- | --- |
| **Q-D1** | **Which Python runtime runs the talking resident?** R1: the launcher uses a Python 3.12+ you already have installed on this Mac, with the project's locked cognition packages, and talks to your local Ollama `qwen3.5:4b`. If any of that is missing, the game still starts, residents follow rules, and a message says why conversation is off. R2: ship a private Python inside the app; needs its own prototype and licence audit. | **R1 now.** R2 later as its own bounded prototype PR. R1 is labelled "requires local runtime", never "self-contained". |
| **Q-D2** | **Who talks, who plays?** Market Town. Alice Moreau (`alice`) is driven by the language model. You play Vera Lindgren (`visitor`). In two-window mode the 2D window is Vera and the 3D window is Wes Calloway (`wanderer`). Everyone else follows rules. | **Yes.** |
| **Q-D3** | **May the final check talk to the local model?** One serial session on this Mac, local Ollama only, no cloud, no downloads, no money: at most **8 model calls in total** (5 scripted player turns plus incidental triggers), **≤180 s total inference time**, **≤20 s per reply**, **≤256 output tokens per reply**, each real-process run ≤10 min, ≤60 min wall for all real-process runs together, one heavy-job slot. Every other test uses recorded/scripted model answers and never needs a model. | **Yes.** |
| **Q-D4** | **Which code does the launcher work start from?** The launcher's town-hosting/living-world correction lives on the lost client integration stack (`d586754a`), which the client-stack recovery lane is rebuilding. The model-deadline fix (C1) touches only cognition and can start on `main` now. | **C1 ships as its own small PR on `main` (7718d952 or newer) right after freeze; C2–C5 ship as the second PR on the recovered integration branch (`recovered/integration-d586754a`, pending recovery) once it exists, C1 is merged, and the launcher seams are re-audited.** Both PRs execute under this one frozen design and contract. |
| **Q-D5** | **How far may the implementer go without asking?** Commit, push its branch, open/update its PR and repair CI up to READY FOR OPERATOR REVIEW. Never merge. | **Yes** (standard Structured Coding defaults; CLAUDE.md §§3.1, 3.4). |

What you get when this PR is done: open the app, walk to Alice, tell her something, leave, come back, change
one detail, quit the app, reopen the same save, and she answers with the *updated* detail and the unchanged
original detail. Other residents keep walking and trading while the model thinks or is switched off. With no
model at all, the world still runs.

## 1. Delivery outcome and scope

The same authoritative town supports headless, 2D and 3D clients. One explicitly selected resident is driven by
the existing LM Controller, and can be taken over by rule or human controllers, keeping Person identity and
authoritative history across release/rejoin. The player has a relevant exchange, corrects an earlier fact,
closes the application, and after reopening the same save receives an answer that uses the corrected fact and
the retained earlier detail. Other residents keep acting during slow/absent inference. The launcher owns
cognition lifetime and uses a stable, per-save-slot cognition directory across restarts and package upgrades.

Reuse `SeatRunner`, `Router`, `SeatHomes`, `mineworld_sdk`, the existing memory/retry/breaker/scope entry points,
rule fallback (`took_over: "hosted"`) and the OpenAI-compatible backend. Not in scope: a new cognition
framework, database, provider SDK, kernel Person field, client game logic, model-service launcher, installer,
paid/cloud provider, full-town LM rollout, arbitrary backup rollback, atomic world+cognition snapshot, hot
memory-policy migration, semantic truth checker, observer privileges for controllers, or any scale claim.
All models may be absent and all renderers detached; rule-only validity is required, but rule template replies
are never accepted as contextual LM conversation.

Not closed by this PR: shopping/nutrition, shop hours, residences, sleep advance and localization lanes, and the
self-contained runtime (R2).

**LM-concept boundary (CLAUDE.md §4.3).** Provider endpoint, model name, tokens and tiers stay inside the
cognition Controller Pack's operator configuration (`CognitionConfig`) and gateway. The launcher sees only
opaque paths (interpreter, config, decisions, manifest) and a versioned nonsecret readiness status. Nothing in
this PR adds an LM concept to a kernel, System, protocol or World Pack contract.

## 2. Source audit (re-audited 2026-11-27)

Audit base: `origin/main` @ `7718d952` (#531). The recovered draft audited `b457a94d`; `git diff --stat
b457a94d 7718d952 -- cognition tools/launch scripts/package.py worlds/market-town sdk` is **empty**, so every seam
below is unchanged since the draft and was re-read on `7718d952` in this lane. The draft's other evidence
(delivered source `703aa8dc`) is preserved in
[`dialogue-memory-evidence/integration-audit-20261123.md`](dialogue-memory-evidence/integration-audit-20261123.md)
(SHA-256 `73b73fef…`, byte-exact) and the full prior draft in
[`dialogue-memory-evidence/recovered-draft-DM-20261123-1.md`](dialogue-memory-evidence/recovered-draft-DM-20261123-1.md)
(SHA-256 `4ec9ae42…`, byte-exact).

| Concern | Verified on 7718d952 | Design consequence |
| --- | --- | --- |
| Launch chain | `tools/launch/src/lib.rs::run` → `children.rs::Server::start` → join → clients → `Server::stop`. `children.rs:59` passes `--agent alice --stop-on-stdin-eof`; no `--town`, no `--decisions`, no cognition child | Cognition joins this ownership chain. **PROVISIONAL**: the integration stack changes `Server::start` to town hosting; re-audit before C2 |
| Launch profile | `config.rs` reads optional `runtime/launch.toml`, a strict string-only TOML subset (`ARC-78`), keys `world`, `seat`, `seat-2d`, `seat-3d`; defaults 2D=`carol`, 3D=`visitor`, both=`visitor`/`wanderer` | Add typed cognition keys to the same grammar; this profile overrides 2D to `visitor` explicitly |
| Save | `user.rs::UserDir::save` → `saves/<world>`; run logs pruned to five | Cognition store lives outside saves and logs |
| Assignment | `mineworld decisions <world> --decisions FILE --json` → `controller/assignment.py::read_assignment` (selects `assigned == "lm"`) | Server and resolver get the exact same decisions file |
| Cognition entry | `__main__.py` (181 lines) → `build_router` + `serve`; one `SeatHomes`, one `SeatRunner` per LM seat; no world handle | Add opt-in supervised mode; standalone CLI unchanged |
| Deadline defect | `triggers.py::Pending.offer` stores `received_at`; `seat.py::_consults` (l.537) only discards stale triggers, then `_consult_once` (l.627) passes a fresh `deadline_s`; `consult.py::consult` measures from its own `started`. `_submit` takes `_submit_lock` (l.487) with no expiry recheck | Real defect: queue, first-Observation and lock waits escape the bound → C1 |
| Gateway defect | `gateway.py::ModelGateway.complete` takes `now` before `async with budget.limiter`, then `asyncio.timeout(limit)` wraps only the backend call; `CancelledError` releases the reservation but charges nothing | Limiter wait escapes the timeout; charge time is pre-wait; a cancelled started call is uncharged → C1 |
| Size debt | `config.py` 732 lines, `seat.py` 765 lines (existing) | No net growth in `seat.py` without disposition; deadline logic goes in a cohesive helper if it improves ownership |
| Memory home | `homes.py::home_path` → `<store>/<instance>/memory.sqlite` (`home`) or `<store>/<instance>/<seat>.sqlite` (`none`); schema `SCHEMA_VERSION = 6` | One top-level store per save slot generation |
| Recall | `memory/retrieve.py::_with` (l.175), `_recent` (l.323); `validate.py` rejects forged recall tokens, not false prose | Citations are necessary, not sufficient; review rendered text |
| Sample config | `examples/local.toml` l.13 says format 1, l.21 says "FG-c is not merged" | Stale; corrected in C2 |
| Spec drift | `docs/MODULE_SPEC.md` l.2243: LM Controller "never chooses movement"; `seat.py` runs `mineworld_sdk.pacer` walk pacing | Stale sentence; corrected in C2 (spec, not behaviour) |
| Package | `scripts/package.py` packages server, launchers, worlds, clients; no Python runtime, no cognition/SDK distributions | R1 ships profile + manifest only; R2 deferred |
| Market Town seats | `worlds/market-town/world.yaml` `seats:` l.228+ lists `visitor`, `wanderer`, `alice`, …; names: Alice Moreau, Vera Lindgren, Wes Calloway | Profile in Q-D2 is real, not invented |
| Existing tests | `tools/cli/tests/milestone_d.rs::she_remembers_across_the_two_views` (ignored; python layer), `record_the_milestone_d_cassette` (operator-run); `speech_both_ways.rs`; cognition `test_budget.py` (`test_a_third_concurrent_call_waits_and_calls_are_admitted_first_in_first_out`, `test_a_caller_deadline_cuts_the_call_and_it_is_charged_as_a_timeout`), `test_controller_{seat,consult,homes}.py`, `test_home_isolation.py`, `test_memory_isolation.py`, `test_milestone_d_{replay,fallback}.py` | Extend these; no parallel harness |
| Integration base | `recovered/client-stack` @ `003d3543` stops at S34-e; vs its merge-base `4cd160bb` it changes nothing in `tools/launch`, `cognition`, `scripts/package.py`. `recovered/integration-d586754a` does **not** exist yet | Q-D4: C1 independent; C2–C5 wait |
| Local model | `ollama list`: `qwen3.5:4b` (`7f3251fa6878`, 4.0 GB). Not run | Size proves nothing about latency or headroom |

Historical Milestone D evidence (scripted earliest-line quotation; one 2026-10 local qwen reply in
`pr-p6c-fgd-memory-coherence.md` §10) is fixture material, not acceptance for corrected recall, current model
compatibility, Godot windows or packaged runtime.

## 3. Runtime route and profile

### 3.1 R1 (recommended, Q-D1)

The launcher is configured with an absolute operator interpreter path and runs it directly (no shell, no
`uv run`) as `<python> -I -m mineworld_cognition`. Before starting the child it runs a preflight in the same
isolated mode: Python ≥ 3.12, the installed SDK/cognition distributions match a reviewed runtime manifest
(source revision, lock digest, installed source-file hashes — version strings alone are insufficient because all
members are `0.1.0`), and the `mineworld.forgetting`, `.retry`, `.breaker`, `.memory_scopes` entry-point groups
resolve. An editable/symlinked install passes only if its imported files match the manifest. A mismatch,
missing interpreter or missing model endpoint yields **degraded rule-only play** with an actionable message;
never an implicit fallback interpreter, never a conversation PASS, never a self-contained claim. No Python
dependency enters the Rust build. The launcher never downloads a model or runtime and never starts or stops an
Ollama service.

R2 (self-contained CPython + locked closure in the package) is deferred: provenance, redistribution terms,
relocatability, native wheels, universal-macOS coverage and size are unaudited. A later prototype PR owns it.

The selected route and the rejection of new brain/memory libraries (existing abstractions already implement the
ownership and protocol) are recorded in one new decision record allocated under `docs/DECISIONS.md` §3 in C2.

### 3.2 Profile (Q-D2)

Market Town for all three modes. LM seat `alice`; human `visitor` in 2D and 3D solo modes; both-mode 2D
`visitor`, 3D `wanderer`; all other seats rule. EntityIds are resolved from the world/welcome, never hardcoded.
Same-player recall across clients: release `visitor` from one client and reacquire from the other. `wanderer`
is a distinct Person and must not inherit `visitor`'s private exchanges. Social Café Alice fixtures remain
deterministic test fixtures, never a silent substitute for Market Town play.

### 3.3 Configuration and data flow (launcher seams PROVISIONAL until the integration base is re-audited)

`runtime/launch.toml` keeps its string-only grammar and gains four optional per-mode keys, all present or all
absent (partial fails by name): `cognition-python` (absolute interpreter), `cognition-config` (absolute
operator-owned `CognitionConfig` TOML), `cognition-decisions` (safe bundle-relative decisions file),
`cognition-manifest` (safe bundle-relative runtime manifest). Default remains absent ⇒ rule-only. Per-user
absolute paths live only in this Mac's configured bundle, never in a distributable profile; the delivery records
a redacted copy.

Flow: `Choice` carries typed paths; `Bundle` resolves bundle-relative resources. `run` resolves save/generation,
runs the owned server binary as a bounded short child to emit format-3 assignment from the same world+decisions
it passes to `Server::start`; the launcher rejects an LM seat that is also a configured human seat; Python
`read_assignment` validates exactly one LM seat. The Python entry point gains typed supervisor-only overrides
(server URL, store directory, decisions JSON) plus `--supervised` (stdin-EOF stop and a versioned status line
on an owned pipe), applied through the same `CognitionConfig` model, never by text concatenation. In supervised
mode provider config cannot override server/store/assignment, and preflight rejects a non-loopback backend URL.
The invite passes as a named child environment variable; the admin token is removed from the child environment;
neither appears in argv, files or logs.

## 4. Invariants (proposed for freeze)

1. Server/System ownership unchanged. Controllers see only permitted Observations and perceived streams; every
   change is an offered `ActionIntent` revalidated by the server. Renderers grant no perception.
2. Rule, human and LM takeover preserve Person EntityId, inventory, holdings, relations and history.
   `take_over=False` never steals a human seat. Cognition stops submitting after seat loss. Re-entry rebuilds only
   what that Person legitimately perceived during absence; no transfer of human thoughts or model internals.
3. One process / Router / SeatHomes owns all LM seats of a run; this profile selects exactly one. Shared storage
   is not shared knowledge. No observer/global channel enters a Controller prompt.
4. **Absolute deadline.** At `Pending.offer` receipt R, expiry E = R + D. Pending wait, first-Observation wait,
   preparation, gateway admission (limiter), HTTP, every retry delay, `_submit_lock` acquisition and dispatch all
   spend this one budget; no stage resets E. No backend request or protocol submit begins at or after E; the
   check is repeated after acquiring `_submit_lock`, immediately before the socket send.
5. A consult ends by E plus a measured scheduler allowance. Expired pending triggers are counted and discarded
   without model work. Existing Pending supersession stays; there is no promise to answer each line of a burst.
6. A dispatch sent before E may be acknowledged after E: that is an uncertain acknowledgement, never
   auto-resubmitted; reconcile via SDK history. Acceptance proves no new dispatch after E and no duplicate action.
7. Gateway timeout includes limiter acquisition. Every exit releases exactly what was acquired. A request that
   never reached the backend costs no call; a started backend call costs exactly one call on success, failure,
   timeout or cancellation (actual usage, else the existing failure-token convention), charged at invocation
   time. The ledger is never reset on restart. `CancelledError` propagates after cleanup; `CassetteMiss` stays
   loud.
8. Each save slot generation owns `cognition/<world>/<generation>/` under the user directory, outside the save and
   run logs; home, resume, decision log, ledger and resolved config all live there. Retained across restart and
   package upgrade; never two writers. The active generation selector is persisted outside logs, bound to the
   welcomed world instance, and a mismatch is refused. `--fresh` allocates a new generation without deleting old
   memory; smoke uses a disposable generation.
9. Restoring an older world backup requires a matching stopped cognition snapshot or a new derivation
   generation. Arbitrary rollback with a newer home is not advertised (it could remember discarded future events)
   and a cursor mismatch is never hidden or silently erased.
10. Supervisor states `disabled`, `starting`, `ready` (seat joined and home opened), `degraded(reason)`,
    `stopped` are distinct and nonsecret. Spawn or endpoint liveness is never `ready`. World and clients never
    wait for inference. Readiness timeout releases the child and leaves hosted rule fallback.
11. The launcher is the single shutdown owner. Final client exit or launch error closes cognition stdin; the
    Stopper cancels work, leaves seats (existing 5 s bound), exits within 15 s; then server stdin closes (existing
    10 s bound). First exit of two clients preserves the other. Launcher death gives both children EOF. Only owned
    child handles are killed/reaped; no process-name search; Ollama is never stopped.
12. Model absence, budget exhaustion, open breaker and malformed or slow answers never block ticks, other
    residents or saving. Rule handback need not answer the timed-out line; no fabricated reply or recall. No
    hidden cloud tier, retry or install. Non-social tiers `none`; retention `consolidate`. Time scale does not
    change wall budgets.

## 5. Ordered semantic commits

Each commit carries its tests and a ledger update. All boxes are open: nothing has been implemented, run or
reviewed. `shuffle`/`file_order`/training settings: N/A (no such interface in this lane). Local validation
follows R-LOAD-1: targeted tests only, no local full-workspace suite; CI on the exact head is the gate.

### C1 — preserve the trigger's absolute deadline end to end (base: `main`)

**Goal.** Close the queue/receipt/submit expiry defect (§2) before cognition joins the packaged lifecycle.
Independent of launcher code, so it can start before the integration base is recovered.

**Scope.** `controller/triggers.py`, `controller/seat.py::_consults/_consult_once/_submit`,
`controller/consult.py::ConsultLimits/consult`, `gateway.py::ModelGateway.complete`, targeted tests in
`test_budget.py`, `test_controller_consult.py`, `test_controller_seat.py`; cognition runtime docs. No retry
policy redesign, no default budget change, no launcher change.

**Implementation.**
- [ ] Thread one absolute expiry from `Pending` receipt through the caller chain; remove the consult-start
  reset; Observation/recall/submit-lock time spends it.
- [ ] Bound gateway admission+backend and consult awaitables by the remaining budget; recheck expiry at backend
  invocation and after acquiring `_submit_lock`; normalize the `deadline` terminal record.
- [ ] Release reservation/limiter on every exit; charge exactly the started backend calls, including
  cancellation, at invocation time; do not swallow `CancelledError` as a retry.

**Validation.**
- [ ] Two seats, `max_in_flight=1`, first backend blocked: second expires in the queue without invoking the
  backend; a third obtains the freed slot.
- [ ] Pending wait and late first Observation consume D; a backend finishing after E cannot submit; retry delay
  and `_submit_lock` wait cannot dispatch late.
- [ ] Cancel before admission vs. during backend: correct call accounting, no held reservation/slot, no duplicate
  action after an uncertain SDK acknowledgement.
- [ ] Existing consult/retry/frame-rejection/pacer tests stay green; capture a failing-before-fix run.

**Acceptance.** Captured timestamps show no backend start or submit at ≥ E; no fresh D after pending/Observation
wait; slot and budget usable after every exit; exactly one charge per started backend call.

**Failure/edges.** Expired trigger → discard and count; gateway timeout → existing fallback/handback; socket loss
→ `disconnected`; caller stop → cancel and clean; malformed text → existing bounded content failure; no reask.

**Commands.** `uv run --locked pytest cognition/lm-controller/tests/test_budget.py
cognition/lm-controller/tests/test_controller_consult.py cognition/lm-controller/tests/test_controller_seat.py`;
ruff format/check and strict pyright on the cognition member. NOT RUN.

**Boundary/review.**
- [ ] Independent logic review of every gateway caller (including the summary gateway), charge paths,
  timeout-vs-sent uncertainty, stop/pacer locks; disposition for `seat.py` 765-line debt and any growth.
- [ ] Staged-diff inspection; commit only this correction and its tests.

### C2 — one validated operator profile and stable cognition generation (base: integration branch)

**Goal.** Give launcher and server one assignment/runtime/store identity, keeping operator endpoint config
separate from World Pack and secrets.

**Scope (PROVISIONAL paths).** `tools/launch/src/{config,bundle,user}.rs`, `tools/launch/LAUNCHER.md`;
`cognition/lm-controller/examples/local.toml` stale comments; `docs/MODULE_SPEC.md` l.2243 movement sentence;
Market Town operator decisions/profile fixture; R1 runtime manifest format and preflight in the cognition entry
point; the new decision record. No new TOML parser, no shell command interface.

**Implementation.**
- [ ] Typed optional cognition keys in the existing grammar; unknown/duplicate/partial entries fail by name.
- [ ] Resolve world, LM seat and human seats from the actual assignment; forbid an LM seat that is a human seat;
  one decisions file for server and resolver; exactly one LM seat in this profile.
- [ ] Allocate/reuse the persistent generation directory and selector; disposable smoke generation; `--fresh`
  keeps old cognition; restore-mismatch refusal.
- [ ] R1 preflight (interpreter, manifest, entry points, loopback backend); supervisor overrides through
  `CognitionConfig`; no invite/admin in files, argv or logs.

**Validation.**
- [ ] Default/disabled profile unchanged; invalid seat, partial keys, stale manifest, wrong world/format, missing
  runtime each give their named outcome.
- [ ] Generation tests: restart reuse, fresh/smoke isolation, package relocation and upgrade keep user state, log
  pruning never touches cognition.

**Acceptance.** Server and resolver agree; resident selection checked; one store per generation with
instance-scoped home; no secret on disk; missing runtime ⇒ actionable degraded rule-only launch.

**Failure/edges.** Busy home or identity/schema mismatch fails cognition by name and leaves the store untouched;
malformed profile fails before any child starts; restore mismatch refused.

**Commands.** `cargo test -p mineworld-launch` (selected config/user/bundle cases, one heavy slot); cognition
config/assignment/home tests; docs link and decision-ID checks. NOT RUN.

**Boundary/review.**
- [ ] Independent review: path escapes, save collision, writer isolation, profile grammar, reuse decision,
  secret flow.
- [ ] Commit configuration, contract text and tests together; no child supervision before C3.

### C3 — supervise one cognition child through the real protocol lifecycle

**Goal.** Make cognition reachable from packaged play and stop it with its world, including launcher death and
two-window lifetime.

**Scope (PROVISIONAL paths).** `tools/launch/src/{lib,children}.rs` (a cohesive cognition-child helper if
needed), `mineworld_cognition/__main__.py`, `shutdown.py`, `controller/seat.py` readiness hook, launcher
real-process fixtures. No server/client domain change; no model-service ownership.

**Implementation.**
- [ ] Start town server, resolve assignment, start the direct-interpreter child with the private invite env and
  without admin env; own its stdin and handle.
- [ ] `--supervised` EOF mode feeds the existing Stopper; versioned status (`ready` after join + home open,
  bounded `degraded`), no provider/secret data; standalone CLI unchanged.
- [ ] Poll child/status during play; unexpected exit ⇒ degraded, world keeps running; bounded reads only.
- [ ] Stop cognition before server on final client exit and on error/drop paths; force-kill/reap only the owned
  child after the bound.

**Validation.**
- [ ] Real child: readiness after join/home; unreachable endpoint; malformed answer; killed child; launch failure
  after server start; shutdown during an in-flight request.
- [ ] Two clients: first closes, second still acts; last closes, server persists, all owned children reaped.
  Killing the supervisor stops server and cognition via EOF.
- [ ] Other residents act during blocked inference; no cloud attempt; no admin token in the child env; a planted
  invite never appears in logs.

**Acceptance.** Child is the direct interpreter; status reflects held seat/home; no orphaned owned child after
bounded shutdown; only owned processes stopped; world responsive under model failure.

**Failure/edges.** Human-held seat never taken; busy store refuses a second writer; readiness expiry ⇒ degraded;
malformed/closed status named. HTTP cancellation may not stop Ollama GPU work: measure and report, never claim it.

**Commands.** Targeted launcher real-process fixture; cognition entry/shutdown tests; `speech_both_ways` and the
Milestone D protocol path where applicable. Scripted backend only; ≤10 min, one heavy slot. NOT RUN.

**Boundary/review.**
- [ ] Red-team: EOF inheritance, kill/reap, status bounds, stdin ownership, drop order, partial-launch exits.
- [ ] Commit lifecycle atomically.

### C4 — corrected memory and controller substitution through a real save

**Goal.** Replace earliest-line quotation with observable corrected recall across restart and non-omniscient
controller substitution.

**Scope.** Extend `tools/cli/tests/milestone_d.rs`, `speech_both_ways.rs`, and in `cognition/lm-controller/tests/`: `milestone_d_cognition.py`,
`test_milestone_d_replay.py`, `test_home_isolation.py`, `test_memory_isolation.py`, `test_controller_seat.py`.
The historical cassette stays untouched; any new recording is separate. Production recall/prompt changes only for
a demonstrated missing relevant perceived line or newer/older confusion, recorded before the change; changing
retention, scopes or bounds is a material deviation. No client game logic.

**Implementation.**
- [ ] Scripted sequence: runtime-generated nonce code + initial location; leave/rejoin question; correction of
  location with code unchanged; unrelated exchange; another resident's recorded action; graceful full stop and
  restart; current-location-and-code question. Questions never contain the answers.
- [ ] Link accepted player/NPC `spoke` Event IDs to shown memory, valid recalls, request/action IDs and the caused
  saved fact; prove retained home marker/cursor reuse on restart.
- [ ] Private undisclosed nonce and a second seat in the same home: no access or recall leak. Rule → human → LM
  release with unchanged identity and holdings.

**Validation.**
- [ ] Scripted backend derives its answer only from shown records; removing the correction or its access makes
  the test fail; a hidden fact never appears in shown memory.
- [ ] Retained-home restart differs from a null-home rebuild; old triggers not replayed; a fresh instance cannot
  recall the old nonce; backup caveat enforced.

**Acceptance.** After restart the caused NPC utterance carries the corrected location and the original code, with
correct source references; no superseded location as current; no unknown private nonce; same Person across
controller swaps; no seat theft or stale concurrent action. A SQLite file alone fails.

**Failure/edges.** Cited but wrong answer = FAIL. Correction absent from shown memory = retrieval defect; present
but ignored = model/prompt failure. Missing runtime or stall before answer = INCONCLUSIVE.

**Commands.** Selected cognition replay/retrieve/isolation/seat tests; `cargo test -p mineworld-cli --test
milestone_d -- --include-ignored --exact she_remembers_across_the_two_views` plus the new correction test (named
in the ledger once it exists). No live model; no 100-day/scale scenario. NOT RUN.

**Boundary/review.**
- [ ] Review exact shown prompt, rendered text, correction chronology, access predicates, restart identity and
  oracle provenance.
- [ ] Commit acceptance and any bounded owner fix together.

### C5 — package and observe the selected profile; bounded live gate

**Goal.** Deliver the playable path with honest prerequisites and package-level evidence.

**Scope.** `scripts/package.py::layout/assemble/licences/check_contents`, `packaging/PLAY.txt`, launcher/package
contract text, smoke/play artifacts. R1 adds the reviewed profile, decisions and manifest — not Python.

**Implementation.**
- [ ] Assemble the R1 profile and notices; preflight validates manifest/hash and entry points from the relocated
  bundle.
- [ ] Keep package/source entry-point parity and user save/cognition paths; default profile stays model-free.
- [ ] Run packaged 2D, 3D and both modes on the same save with `alice` on the LM; record startup/usable frame,
  response, corrected recall after close/reopen, and shutdown.

**Validation.**
- [ ] `--self-test`, standard smoke, R1 missing-prerequisite degraded path, configured-runtime success.
- [ ] Rendered dialogue and walk/interact in both clients; both-window exit semantics; packaged model-absent run;
  no download or remote connection.
- [ ] Checksums, source/export identity, redacted config, lock/manifest, licences, size report, process-cleanup
  evidence.
- [ ] **Live gate (§6)** — the only live-model run in this PR.

**Acceptance.** The relocated package runs the declared route; a player receives contextual corrected recall
across restart; no owned-child leak; R1 labelled prerequisite-dependent.

**Failure/edges.** Missing interpreter/arch/metadata fails preflight by name and never switches to source Python;
read-only package dir and paths with spaces/Unicode work; a new save slot cannot reuse a prior nonce.

**Commands.** `python3 scripts/package.py --self-test`; `assemble`, `budget`, `smoke` on existing artifacts in
one heavy slot; GUI sessions through UI tools. NOT RUN.

**Boundary/review.**
- [ ] Review assembled manifest, prerequisite disclosure, licences, fingerprints, window evidence, performance,
  process lifetime, and maintainability per CLAUDE.md §3.4.
- [ ] Final ledger/diff review; one semantic delivery commit; READY FOR OPERATOR REVIEW. Never merge.

## 6. Bounded local live gate (Q-D3)

Single serial session on this Mac after all deterministic checks pass. Installed `qwen3.5:4b` at
`http://127.0.0.1:11434/v1`; empty `key_env`; no secrets file; no cloud tier; no download; no service start.
`structured_output="none"`, `reasoning="off"` are starting candidates whose compatibility is measured, not
assumed; the response schema is still checked client-side. Social tier local; other tiers `none`; retention
`consolidate`; `max_in_flight=1`; `max_calls=1` per consult; no automatic reask; D = 20 s absolute; ≤256 output
tokens per call; existing ≤6000-byte memory section.

Envelope (hard caps, enforced by the harness, counted across restart and failures, never reset):

| Bound | Value |
| --- | --- |
| Scripted player turns | 5 (statement; rejoin question; correction; unrelated line; post-restart question) |
| Backend calls, total | ≤ 8 (includes arrival/need/upcoming triggers and failures) |
| Inference wall time, total | ≤ 180 s |
| Per reply | ≤ 20 s trigger-to-submit |
| Each real-process run | ≤ 10 min |
| All real-process runs in C3–C5 | ≤ 60 min wall, one heavy slot, no concurrent heavy job |
| Money | 0; local only |

Sequence: nonce code + location → relevant acknowledgement → leave/rejoin ask → correction (new location, same
code) → unrelated short exchange → close all → reopen same save, Person and generation → "what is the current
location and code?". PASS requires the corrected location and the original code in the reply, both source events
in shown context with valid citations, and a caused saved NPC utterance; meaning is reviewed by a reader, not
token presence. At least one other resident's action during inference is recorded. Rule-only baseline vs one-LM
profile: usable frame, other-resident action/tick latency, reply latency, CPU and unified-memory pressure —
reported as observations, not capacity claims.

Stop at any cap: save the world, keep evidence, report INCONCLUSIVE (prerequisite unavailable) or FAIL (wrong
answer). Never raise a cap, substitute a model or retry for a lucky answer without a new operator decision.
Privacy, identity, deadline and cleanup negatives are owned by deterministic tests, not the live gate.

## 7. Verification ownership

| Layer | Unique claim | Status |
| --- | --- | --- |
| Static | types, format/lint, provider/ownership scans, doc links, decision IDs | NOT RUN |
| Deterministic integration | absolute deadline, cleanup/accounting, assignment/store isolation, causality, access, replay, identity | NOT RUN |
| Live gate (Gate 1) | local model gives relevant reply and corrected recall within §6 caps | NOT RUN; pending Q-D3 |
| Real process + package (Gate 2) | EOF/cleanup, save+home restart, exported windows with R1 runtime | NOT RUN |
| Terminal CI | exact final head under existing CI policy | NOT RUN |

Report exact commands, counts, wall time, configuration, HEAD + diff fingerprint, and decisive observations;
classify PASS/FAIL/INCONCLUSIVE only after inspection. Review covers debt, boundaries, readability, robustness,
coupling, extension, size growth and restart behaviour.

## 8. Corrections to the recovered draft DM-20261123-1

1. **Audit base** `b457a94d` → `7718d952`; seams re-read and the seam diff between them is empty (§2).
2. **Implementation base resolved by split**: C1 runs on `main`; C2–C5 wait on `recovered/integration-d586754a`,
   which does not exist yet (`recovered/client-stack` stops at S34-e and changes no launcher/cognition file).
   Launcher seams are marked PROVISIONAL and must be re-audited on that branch before C2.
3. **Endpoint authority**: the draft blocked push/PR/CI in all lanes because its *planning* dispatch did; that
   restriction is not an execution decision. Execution endpoints now follow Structured Coding defaults pending
   Q-D5; merge stays operator-only.
4. **Live gate made concrete**: added turn count (5), total heavy envelope (≤60 min) and money (0); the draft
   left the total heavy envelope "to be approved by coordinator".
5. **Operator decisions moved to the top in plain language**; Q-D4 and Q-D5 added (the draft had three).
6. **Planning location**: the draft lived in a `/tmp` clone; it is now tracked here with both recovered sources
   preserved byte-exact under `dialogue-memory-evidence/`.
7. **Verified test inventory**: added `test_memory_isolation.py`, `test_milestone_d_fallback.py` and the
   cognition helper `milestone_d_cognition.py`; confirmed the Milestone D test names.
8. **LM-concept boundary** stated explicitly (§1): the launcher handles opaque paths and a nonsecret status only.
9. **R-LOAD-1** applied: no local full-workspace suite; CI is the gate.

## 9. Freeze checklist (coordinator)

- [ ] Independent red-team reads this design and the preserved audit; findings and dispositions recorded here.
- [ ] Operator answers Q-D1…Q-D5.
- [ ] Add the `DESIGN FROZEN` header with revision, approval evidence, base and contract link; fill the contract's
  pending values from the answers.
- [ ] Dispatch C1 (its own PR) to a fresh owner on `main`; hold C2–C5 (second PR, on merged C1) until the integration branch exists, then re-audit the
  launcher seams and record the exact base SHA here (bounded, non-material unless the seams moved).
- [ ] Keep the progress page honest: source implemented, package integrated, reviewed and player-accepted are
  separate states.
