# Playable dialogue and memory integration — PR design and execution contract

Lifecycle: DRAFT FOR OPERATOR DESIGN REVIEW. This is not DESIGN FROZEN.
Design revision: DM-20261123-1. Approval evidence: absent; must be supplied by the
operator. Planning owner: coordinator; drafting owner: memory_plan. Execution owner:
one fresh delegated session, to be assigned by coordinator; independent red-team:
coordinator's separately assigned reviewer. No source implementation is authorized
by this draft. This document is the single PR ledger for this lane.

Parent: `overall.md` and coordinator-owned
`operator-playable-mvp-20261123.md` §§1–4, 8. The parent is currently untracked in
the coordinator checkout and must be imported by its owner before freeze. This
isolated clone does not manufacture a second parent. Binding policies:
`CLAUDE.md` §§1.1, 2, 3.1–3.4, 4; `docs/ARCHITECTURE.md` §§8–9;
`docs/MODULE_SPEC.md` §§5–5.3; `tools/launch/LAUNCHER.md`;
`step-23-release.md`; `step-17-cognition.md`; `step-28-shared-memory.md`.
The operator's November 23 dual-use clarification applies even where this clone's
committed policy predates the coordinator's dirty clarification.

## 1. Delivery outcome and scope

The same authoritative town supports headless, 2D and 3D clients. One explicitly
selected resident can be driven by rule, human or the existing LM Controller,
retaining Person identity and authoritative history across release/rejoin. The
player has a relevant exchange, changes an earlier fact, closes the application,
and receives an answer using the corrected fact and the retained earlier detail
after reopening the same save. Other residents keep acting during slow/absent
inference. The delivered launcher owns cognition lifetime and uses a stable,
isolated cognition slot across restarts and package upgrades.

Reuse `SeatRunner`, `Router`, `SeatHomes`, SDK, existing memory/retry/breaker/scope
entry points, rule fallback and OpenAI-compatible backend. No new cognition
framework, database, provider SDK, kernel Person field, client game logic, model
service launcher, installer or paid/cloud provider. No full-town LM rollout.
No arbitrary backup rollback, atomic world+cognition snapshot, hot memory-policy
migration, semantic truth checker, research observer privileges or scale claim.
All models may be absent; all renderers may be detached. Rule-only validity is
required, but rule templates are not acceptance for contextual LM conversation.

The PR covers the correctness prerequisite, supervisor integration and one
explicitly selected delivery profile. It does not close shopping/nutrition,
shop-hours, residence, sleep advance or localization lanes. Existing startup and
town/admin supervisor corrections remain separate predecessor owners.

## 2. Source audit and reproducible planning identity

Planning clone: `/tmp/mineworld-dialogue-memory-plan-20261123`.
Branch: `plan/playable-dialogue-memory-20261123`.
Audited committed base: `b457a94d9a1ace6a39005de683cc393886e19288`.
Implementation base: pending exact reviewed town/admin launcher predecessor HEAD;
coordinator must resolve this before freeze. A source diff audit, not assumption,
is required when rebasing this design onto that predecessor.
The dirty root checkout is read-only; it is not an execution tree.

Full preserved audit: [integration-audit-20261123.md](dialogue-memory-evidence/integration-audit-20261123.md).
It audited delivered source `703aa8dce66c3fa150942315e74bffd346fcfa8a` and compared
actual cognition/launcher trees with the root. Export identity
`e3f207186ea76d562800dbd6c53cb40daf9dacc1` is operator-amendment evidence, not
independently reconstructed in this lane.

Re-inspected actual caller paths in this planning session:

| Concern | Existing implementation and design consequence |
| --- | --- |
| Launch | `tools/launch/src/lib.rs::run` -> `Server::start` -> `Server::join` -> `Clients::run` -> `Server::stop`; add cognition to this ownership chain after reviewed town/admin predecessor, never a second independent server |
| Config | `config.rs::Choice/choice/parse` accepts only three string sections today; `Bundle` resolves package paths; use explicit typed cognition selection, not shell command strings |
| Save | `user.rs::UserDir::save` is `saves/<world>`; logs prune five runs. Stable cognition belongs outside saves/logs and smoke scratch is disposable |
| Server | `children.rs::Server::start` currently passes `--agent alice`; predecessor owns changing that to town/admin. This lane adds matching `--decisions` only, preserving its admin secret isolation |
| Assignment | `mineworld decisions <world> --decisions FILE --json` format 3 -> `controller/assignment.py::read_assignment`; both server and resolver must use the exact same file |
| Cognition | `__main__.py::main` -> `build_router` + `serve` -> one `SeatHomes`, one `SeatRunner` per selected seat; no world handle |
| Deadline | `controller/triggers.py::Pending` already stores `received_at`; `seat.py::_consults` discards that time after a stale check, then `_consult_once`/`consult.py::consult` starts a new relative deadline. First Observation, submission lock and model queue waiting escape the advertised bound |
| Gateway | `gateway.py::ModelGateway.complete` holds reservation, waits limiter outside timeout, and charges at pre-wait `now`. Cancellation currently releases reservation but may bypass charging an already invoked backend |
| Memory | `homes.py::home_path/SeatHomes.hold` opens `<store>/<instance>/memory.sqlite`; resume/log names are seat-only at top-level store. Use separate top-level store per save slot and world generation |
| Recall | `memory/retrieve.py::_with` selects first two plus newest exchanges, six lines/1000 bytes, then other bounded sections; `_recent` is newest first. Citation validity does not prove truth. No new semantic memory engine is justified before contextual acceptance identifies a defect |
| Package | `scripts/package.py::layout/assemble/licences/check_contents` includes no Python runtime or installed cognition/SDK distributions; package policy forbids .md globally, including dependency metadata licence files unless explicitly revised |
| Runtime | Cognition and SDK require Python >=3.12; runtime deps include pydantic, httpx2, python-dotenv, websockets and their locked transitive packages. Root uv workspace also has unrelated dev/mesh tools: never bundle the entire `.venv` |
| Metadata | cognition pyproject publishes `mineworld.forgetting`, `.retry`, `.breaker`, `.memory_scopes` entry points. Installed distribution metadata is required; copying `src/` alone loses contract-critical discovery |

Historical Milestone D evidence: `tools/cli/tests/milestone_d.rs` runs real server
and cognition processes with protocol clients tagged 2d/3d; its scripted backend
quotes the earliest shown line. `pr-p6c-fgd-memory-coherence.md` §10 records an
actual historical local qwen3.5:4b umbrella reply and replay/citation checks.
These are useful fixtures and historical evidence, not current live compatibility,
corrected-memory acceptance, actual Godot-window evidence or exported runtime
completeness. No runtime tests, build, inference or downloads ran in this design lane.

## 3. Runtime and delivery decision requiring operator selection

This material dependency/delivery decision must be settled before freeze; neither
alternative is approved by drafting it. Existing installed model: qwen3.5:4b,
reported by coordinator's read-only `ollama list`. Model file size proves no
performance or machine headroom. Launcher must never download a model/runtime or
start/kill an operator's Ollama service.

**R1 — first local Mac delivery with declared installed-runtime prerequisite
(recommended for the immediate playable checkpoint).** The launcher is explicitly
configured with an absolute operator interpreter path. Before any cognition child,
run that interpreter directly, without shell/`uv run`, in isolated Python mode;
preflight Python >=3.12, package versions/entry-point groups and importability.
Use already installed locked SDK/cognition distributions from a reviewed source
revision. Store a nonsecret runtime manifest with source revision and lock digest;
reject a stale/unreviewed package rather than silently using another checkout.
The supervisor invokes that interpreter directly with `-I -m mineworld_cognition`.
No Python dependency is added to the Rust/kernel build. Package documentation and
launcher status must say this package requires the separately provisioned local
cognition runtime and local model service. Missing preflight produces rule-only
degraded play with an actionable message; it cannot be reported as conversation
PASS or as a clean-machine self-contained release. This option reaches the user's
current Mac through an actual package, with explicit prerequisites, and keeps
self-contained distribution as the subsequent dependency checkpoint.

**R2 — self-contained cognition runtime in the exported package.** Provision a
target-specific relocatable CPython >=3.12 runtime at package-build time, install
only the locked SDK/cognition runtime closure as real distributions with entry
points, and include runtime/dependency licences and hashes. No app-launch install.
Reuse the project's Python/uv toolchain; do not select PyInstaller/PyOxidizer or
copy a development venv without evidence. The exact CPython artifact provenance,
redistribution terms, relocatability, native wheels, architectures, compressed
size and clean-environment prototype are NOT yet audited. macos-universal requires
both supported CPU paths or an explicit operator-approved narrower target; the
existing Rust universal binary does not make Python native extensions universal.
R2 needs a separately bounded artifact prototype/reuse decision before this PR
can freeze it. No invented runtime/download authority or size-budget increase.

Concrete operator decisions: choose R1 or commission the R2 artifact prototype;
confirm the proposed first contextual resident/save profile: Market Town, LM
`alice` (Alice Moreau), solo human `visitor` (Vera Lindgren), and in both-client
mode 2D `visitor` plus 3D `wanderer` (Wes Calloway), all remaining seats rule.
These keys are explicitly declared in `worlds/market-town/world.yaml` lines
228–239 and their `people/` definitions; this is an audited proposal, not invented
seat availability. Authorize the bounded local live gate
in §7 or leave live acceptance pending. Current Social Café Alice fixture is reused
for deterministic localization, never silently substituted for Market Town play.
Runtime EntityIds are resolved from the selected world/welcome, never hardcoded.
For same-player recall across clients, release `visitor` from the first client
and reacquire `visitor` from the second. Concurrent `wanderer` is a distinct Person
and must not inherit private visitor exchanges. Reconfirm mapping after predecessor
rebase; do not silently retain launcher default `carol` for this profile.

No additional choice is needed about adopting a new brain/memory library: rejected
as unnecessary because existing installed abstractions implement the required
ownership and protocol. Record the selected runtime route and rejection rationale
in one new decision record allocated through `docs/DECISIONS.md` rules, rather than
inventing a reserved decision ID in this draft.

### 3.1 Proposed R1 configuration and actual launch data flow

Keep `runtime/launch.toml`'s existing string-only grammar. Add four optional
per-mode keys: `cognition-python` (absolute operator interpreter),
`cognition-config` (absolute operator-owned existing CognitionConfig TOML),
`cognition-decisions` (safe bundle-relative selected operator decision file), and
`cognition-manifest` (safe bundle-relative reviewed runtime manifest). All four
are present together or all absent; partial configuration fails. Defaults remain
absent/rule-only. This Mac's configured package uses Market Town for all modes;
solo modes select visitor, both selects visitor/wanderer. Per-user paths must not
be checked into a distributable public profile. The delivery operator fills the
explicit paths in this local configured artifact and records a redacted profile.
No command argument string, executable search, shell interpolation or fallback
interpreter selection is permitted.

`Choice` carries typed profile paths/selection; `Bundle` resolves only its safe
relative resources. `run` resolves save/generation then asks the owned server
binary to produce format-3 assignment from the same world+decisions it passes to
`Server::start`. The resolver is an owned bounded short child; no invitation or
admin capability goes to that command. Python assignment reader validates selected
LM seat, while the launcher prevents conflicts with configured human seats.

Extend Python entry point with typed supervisor-only overrides for server URL,
store directory and decisions JSON plus `--supervised` EOF/status mode. Existing
`--config` still supplies provider, tier, budgets, memory and policy selections.
Overrides are parsed as data and applied through the same CognitionConfig model,
not raw concatenation into TOML. The ephemeral resolved JSON is nonsecret in this
run's generation directory; server URL is loopback. Invite remains a named child
environment value, admin is removed. Provider config can never override the owned
server/store/assignment in supervised mode. The operator may not point this MVP
profile at a cloud endpoint: preflight rejects nonloopback backend bindings.

A versioned preflight/status boundary verifies interpreter version, exact installed
runtime closure against the reviewed manifest (including installed source-file
hashes, not merely all packages' unchanged `0.1.0` version strings), required entry
points, selected seat and home-open readiness. A symlinked/editable installation
is acceptable only if its actual imported files match that manifest; dirty root
or unrelated source cannot satisfy it by version coincidence. This may be an
opt-in preflight path of the existing cognition entry point; no daemon or new
framework is introduced. C2 implements and tests the schema/overrides/manifest;
C3 uses it for child supervision. R2 may replace interpreter resource resolution
with an audited bundled path, not widen provider or world authority.

## 4. Frozen-intent candidates and exact semantics

1. Server/System ownership is unchanged. Controllers receive permitted Observation
   and perceived streams; all changes go through offered ActionIntents and server
   revalidation. The renderer cannot grant perception or change decisions.
2. Rule, human and LM takeover preserve Person EntityId, authoritative inventory,
   holdings, relations and history. `take_over=False` never steals a human seat.
   Cognition stops stale submissions after seat loss. Re-entry may rebuild only
   the Person's legitimately perceived events during controller absence.
3. One process/Router/SeatHomes owns all selected LM seats in one run; initial
   shipped selection is exactly one LM resident. Shared storage is not shared
   knowledge. No observer/global event channel enters the Controller prompt.
4. Absolute deadline: at Pending.offer's monotonic receipt R, expiry E=R+D.
   Pending wait, first-Observation wait, synchronous preparation, gateway admission,
   HTTP, every retry delay, submit-lock acquisition and request dispatch all spend
   this same budget. No stage resets E. No new backend request or protocol submit
   begins at/after E. Recheck immediately before actual socket submission after
   acquiring `_submit_lock`, not only before entering `_submit`.
5. The consult ends by E plus a small measured scheduler allowance. Bound awaitable
   asks/submits with cancellation; check monotonic time after synchronous work.
   Expired pending triggers are counted and discarded without model work. A
   newer serially admitted line supersedes according to existing Pending policy;
   no promise to answer each line in a burst.
6. Dispatch before E can be accepted by the server or acknowledged after E.
   A timeout after send is an uncertain acknowledgement, not proof that no action
   happened. Never automatically resubmit such an intent; use SDK action/history
   reconciliation. Acceptance proves no *new dispatch* after expiry and no duplicate
   action, not rollback of an already accepted world action. Keep replay causality.
7. A gateway timeout includes limiter acquisition. Every exit releases exactly the
   reservation/limiter actually acquired. A request that never enters the backend
   costs no call; once backend invocation begins, success/failure/timeout/cancel
   incurs exactly one call, with actual usage when available and conservative
   existing failure-token convention otherwise. Record charge at invocation wall
   time, never reset/erase ledger on restart. Caller cancellation propagates after
   cleanup; CassetteMiss remains loud. Tests must establish this distinction.
8. Each save slot/generation owns `cognition/<world>/<generation>/` outside world
   save and run logs. Home paths still include world instance. Resume, decisions,
   ledger and configuration use this same generation directory. Retain it across
   normal restart and package upgrade; never run simultaneous writers. Persist
   the active generation selector outside logs, associate it with the welcomed
   world instance, and reject mismatch rather than choosing a new directory on
   each run. `--fresh`
   allocates a new generation without deleting historical memory; smoke uses its
   disposable generation. No cross-world/nonmatching-seat reuse.
9. Restoring an old world backup requires a matching stopped cognition snapshot
   or a new derivation generation. This release does not advertise arbitrary
   rollback with a newer home, because that can remember discarded future events.
   Do not merely hide a cursor mismatch or silently erase the old store.
10. Readiness: disabled, starting, ready(seat joined + home opened), degraded(reason),
    stopped are distinct nonsecret supervisor states. Process spawn/endpoint
    liveness alone never means ready. Clients/world need not wait for inference.
    Bounded readiness failure releases owned cognition and leaves hosted rule
    fallback; malformed selection/config fails by name before launch. Avoid
    parsing human logs: define a narrow versioned machine status on an owned pipe.
11. Shutdown owner is launcher. Final client exit/error closes cognition control
    stdin; child Stopper cancels work, leaves seats within existing 5s bound and
    exits within 15s, then launcher closes server stdin and waits its existing 10s
    bound. First exit of two clients preserves the other. Launcher death yields
    cognition EOF as well as server EOF. No PID-by-name cleanup; never stop Ollama.
    Python CLI's standalone mode remains supported; supervisor EOF mode is opt-in.
12. Model absence, budget exhaustion, open breaker and malformed/slow answers cannot
    block server ticks, other residents or saving. Rule handback need not answer
    the original timed-out line; degraded state must not fabricate a reply/recall.
    No hidden cloud tier, retry or automatic install. Non-social tiers stay none;
    retention stays consolidate. Time-scale changes do not alter wall budgets.

## 5. Ordered semantic commits

Each commit carries its own tests and this live ledger update. All checkboxes below
are intentionally incomplete. No implementation/test/independent review has run.
`shuffle/file_order` and training/scorer settings are N/A: no such interface exists
in this lane. Source scope below is audited, not permission to modify other owners.

### C1 — preserve the trigger's absolute deadline end to end

**1 Goal.** Close a real queue/receipt/submit expiration defect before attaching
cognition to the packaged lifecycle. This is independently routable as a bounded
correctness repair if coordinator assigns its own frozen contract/owner; this draft
does not itself authorize repair.

**2 Scope.** `controller/triggers.py`, `controller/seat.py::_consults/_consult_once/_submit`,
`controller/consult.py::ConsultLimits/consult`, `gateway.py::complete`, and targeted
existing gateway/consult/seat/budget tests; cognition runtime docs. Dependencies:
none on launcher code, but one owner must hold cognition files for the whole PR.
No retry policy redesign or changed default budget ceilings.

**3 Implementation.**
- [ ] Thread one absolute expiry derived from Pending receipt through actual caller
  chain, removing consult-start reset; spend Observation/recall/submit-lock time.
- [ ] Bound gateway queue+backend and consult awaitables; check expiry at backend
  invocation and actual submit after lock; normalize deadline terminal record.
- [ ] Release reservation/limiter on timeout/cancel, charge exactly attempted
  backend calls including cancellation, and preserve CassetteMiss/SDK uncertain
  acknowledgement semantics. Do not swallow CancelledError as a retry.

**4 Validation.**
- [ ] Deterministic regressions: two seats, max_in_flight=1, first backend blocked,
  second expires in queue without invoking backend; third obtains freed slot.
- [ ] Pending waiting/late first Observation consumes D; backend finishing after
  expiry cannot submit; retry delay and `_submit_lock` wait cannot dispatch late.
- [ ] Cancel before admission versus during backend gives correct call accounting,
  no held reservation/slot, no duplicate action after uncertain SDK acknowledgement.
- [ ] Existing ordinary successful consult/retry/frame rejection and pacer behavior
  remains green; no live model is required for this correctness commit.

**5 Acceptance.** No newly begun backend/submit at >= E in captured timestamps;
no fresh D after pending/Observation wait; budget/slot usable after every exit;
exactly one charge for a started backend. Existing final action revalidation holds.

**6 Failure/edges.** Expired triggers discard; gateway timeout fails/handbacks under
existing policy; socket loss records disconnected; caller stop cancels/cleans;
malformed model text follows existing bounded content failure; no hidden reask.

**7 Commands/evidence.** After prerequisite installed environment is verified,
`uv run --locked pytest cognition/lm-controller/tests/test_budget.py
cognition/lm-controller/tests/test_controller_consult.py
cognition/lm-controller/tests/test_controller_seat.py` as one cognition-only
invocation. Gateway concurrency/timeout tests actually live in `test_budget.py`;
there is no `test_gateway.py`. Existing tests include
`test_a_third_concurrent_call_waits_and_calls_are_admitted_first_in_first_out`
and `test_a_caller_deadline_cuts_the_call_and_it_is_charged_as_a_timeout`. Ruff/format and strict Pyright for affected cognition member. New real
queue/lock timing test records wall timestamps and tolerance. NOT RUN.

**8 Boundary/review.**
- [ ] Independent logic review checks every gateway/consult caller (including
  summary gateway), charge paths, timeout-vs-sent uncertainty and stop/pacer locks;
  record existing 732/765-line config/seat debt and any new growth disposition.
- [ ] Inspect exact staged files/diff, preserve failing-before-fix evidence, update
  ledger and commit only this correction and tests. No inference/package work.

### C2 — define one validated operator profile and stable cognition generation

**1 Goal.** Give launcher and server one assignment/runtime/store identity; keep
operator endpoint/config separate from World Pack and secrets.

**2 Scope.** `tools/launch/src/config.rs`, `bundle.rs`, `user.rs`, launcher contract
`LAUNCHER.md`; `examples/local.toml` stale format/summarize comments;
`MODULE_SPEC.md` §5.1 stale movement sentence; selected operator decision/profile
fixture; selected runtime decision record. Depends on reviewed town/admin base
and R1/R2 choice. No third TOML parser or arbitrary shell command interface.

**3 Implementation.**
- [ ] Add typed optional local cognition selection to launch profile using the
  existing narrow parser where sufficient; unknown/duplicate/invalid entries fail.
- [ ] Resolve selected world, LM resident and human seats from actual assignment;
  forbid selecting human entry seats as LM. Use one decision file for server
  start and format-3 resolver; validate exactly one assigned LM seat in this profile.
- [ ] Allocate/reuse persistent generation directory and separate smoke generation;
  preserve old cognition on fresh world; distinguish world backup restore policy.
- [ ] Apply R1 interpreter/metadata/source-lock preflight or the separately reviewed
  R2 artifact route. Build nonsecret controller config from validated settings;
  never interpolate invitation/admin tokens into files, argv or logs.

**4 Validation.**
- [ ] Existing config tests cover disabled/default behavior; invalid seat/profile,
  stale runtime manifest, wrong world/format and no runtime are explicit outcomes.
- [ ] Save-slot/generation selection tests prove restart reuse, fresh/smoke isolation,
  package relocation and upgrade preserve user state, logs cannot prune memory.

**5 Acceptance.** Server/resolver agree; resident selection is checked, not guessed;
one stable top-level store contains instance-scoped home and seat-only runtime
files; no invite/admin secret on disk; missing runtime means actionable degraded
rule-only launch, never implicit interpreter fallback.

**6 Failure/edges.** Busy home, identity/schema mismatch fail cognition by name;
leave original store untouched. Missing runtime/model yields degraded world;
malformed assignment/profile fails configuration before process ownership starts.
Restore mismatch is refused or requires a deliberately new derivation generation.

**7 Commands/evidence.** `cargo test -p mineworld-launch` selected config/user/bundle
cases after single shared heavy slot is granted; package self-test targeted profile
fixtures; cognition config/assignment/home tests. Static docs heading/decision-ID
checks and diff review. These are intended commands, NOT RUN in this lane.

**8 Boundary/review.**
- [ ] Independent review checks path escapes, save collision, same-store writer
  isolation, untrusted profile grammar, dependency/reuse decision and secret flow.
- [ ] Record staged paths/diff and commit configuration + contract + tests together;
  no child supervision starts until C3.

### C3 — supervise one cognition child through real protocol lifecycle

**1 Goal.** Make contextual cognition reachable from packaged play and stop with
its owned world, including launcher death and two-window lifetime.

**2 Scope.** `tools/launch/src/lib.rs::run`, `children.rs`, a cohesive cognition
child helper if needed, `mineworld_cognition/__main__.py`, `shutdown.py`,
`controller/seat.py` readiness boundary, launcher process integration fixtures.
Depends on C1/C2. No unrelated client/server domain change, no model-service owner.

**3 Implementation.**
- [ ] Start town server, resolve matching assignment, then direct cognition child
  with private invite environment and admin env removed. Own its stdin/handle.
- [ ] Add opt-in supervisor EOF mode to Python entry point feeding existing Stopper;
  standalone CLI retains current signals. Define versioned status for joined+home
  ready and bounded degraded failure, with no provider/secret data.
- [ ] Poll child/status during client lifetime; unexpected cognition exit reports
  degraded without stopping valid rule world. Avoid unbounded readiness/log reads.
- [ ] Shutdown cognition before server on final client exit and error/drop paths;
  force-kill/reap only owned child after bound. EOF after launcher kill stops it.

**4 Validation.**
- [ ] Real child fixture: readiness after join/home, unavailable endpoint, malformed
  answer, killed child, launch failure after server start, shutdown during request.
- [ ] Both-client run: first closes, second still acts; last closes, server persists
  and all owned children are reaped. Kill supervisor: EOF stops server+cognition.
- [ ] No-model world and other residents continue during blocked inference; no
  cloud attempt, no admin in cognition environment, planted invite absent logs.

**5 Acceptance.** Actual child is direct interpreter rather than uv wrapper; status
reflects held seat/home; no orphan owned child survives bounded shutdown; only
launcher-owned processes stop; world remains responsive under model failure.

**6 Failure/edges.** Lost/human seat never forcibly taken. Busy store cannot create
a second writer. Readiness expires into degraded fallback. Status malformed/closed
is named. EOF cancellation need not stop already running Ollama GPU work; measure
and report that limit, without claiming killing an HTTP client unloads a model.

**7 Commands/evidence.** Targeted launcher real-process fixture plus cognition
entrypoint/shutdown tests; existing CLI `speech_both_ways` and selected Milestone D
protocol path where applicable. Real-process gate <=10 minutes, one shared heavy
slot; no live inference in this commit's deterministic lifecycle gate. NOT RUN.

**8 Boundary/review.**
- [ ] Independent red-team checks EOF inheritance, direct-child kill/reap, status
  bounds, stdin ownership, reverse drop order and all error/partial-launch exits.
- [ ] Inspect staged source/tests/contracts and commit lifecycle atomically.

### C4 — establish corrected memory and controller substitution through real save

**1 Goal.** Replace earliest-quotation sufficiency with observable contextual,
corrected recall across restart and non-omniscient controller substitution.

**2 Scope.** Extend `tools/cli/tests/milestone_d.rs`, `speech_both_ways.rs`, existing
cognition Milestone D helper/replay tests and home isolation/seat tests. Preserve
historical cassette. New bounded recording is separate, not an edited golden.
Potential production recall/prompt changes are limited to a demonstrated missing
relevant perceived line or confusion between newer/older disclosed evidence;
record before changing. Altering retention/scopes/bounds is material deviation.
Depends C1–C3; no client game logic.

**3 Implementation.**
- [ ] Add nonce/code + initial location, leave/rejoin question, newer correction,
  unrelated exchange, graceful full stop/restart and correction question sequence.
- [ ] Link accepted player/NPC spoke Event IDs to model shown memory, valid recalls,
  request/action IDs and saved caused facts; prove reopened home marker/cursor reuse.
- [ ] Include private-undisclosed nonce and another seat in same home; assert no
  access/recall leak. Exercise rule->human->LM release without changing identity.
- [ ] Keep seeded replay across renderer tags and headless same authoritative
  outcomes, then actual Godot package trials in C5. Historical protocol tags alone
  do not claim window acceptance.

**4 Validation.**
- [ ] Deterministic backend derives answer from shown corrected + initial-code
  records, never expected constants hidden in prompt. Removing correction/source
  access makes relevant test fail; unrelated hidden fact cannot appear in memory.
- [ ] Restart retained home differs from null-home rebuild evidence; old triggers
  do not replay, fresh instance cannot recall old nonce; backup caveat enforced.
- [ ] Gate 1 in §7 owns real semantic answer quality. Scripted tests own causal
  plumbing, not a claim that Qwen understands corrections.

**5 Acceptance.** Initial relevant reply, later answer containing library + original
nonce code after restart, correct source references and caused NPC fact; no bakery
as current location, no unknown private nonce; same Person/holdings/relations across
controller swaps, no human theft/concurrent stale action. A SQLite file alone fails.

**6 Failure/edges.** Cited but semantically incorrect answer is FAIL. Missing
correction in shown memory is retrieval defect; present correction ignored is
model/prompt failure. Missing runtime/stalled run before answer is INCONCLUSIVE,
not a fake PASS. Do not continually raise gate cap to get a lucky answer.

**7 Commands/evidence.** Selected cognition replay/retrieve/isolation/seat tests;
`cargo test -p mineworld-cli --test milestone_d -- --include-ignored --exact
she_remembers_across_the_two_views` retains old plumbing coverage; name the new
correction test in execution ledger after it exists. The single packaged live gate in §7 jointly owns real answer semantics and
package reachability, with distinct assertions; do not spend a duplicate source
live sequence before it. No 100-day/scale/nightly scenario in initial checkpoint. NOT RUN.

**8 Boundary/review.**
- [ ] Independent review inspects exact shown prompt, rendered NPC text, correction
  Event chronology, access predicates, restart identity and test oracle provenance.
- [ ] Commit causal acceptance and bounded owner fix together; preserve unsuccessful
  model observations and request a material amendment if chosen model cannot meet
  contract inside budget. No silent promotion of template answer to real evidence.

### C5 — package and observe the selected delivery profile

**1 Goal.** Deliver the actual playable path with honest runtime prerequisites
and package-level evidence; source success cannot close exported delivery.

**2 Scope.** `scripts/package.py::layout/assemble/licences/check_contents`,
`packaging/PLAY.txt`, launcher/package contract, bounded smoke/play artifacts.
Depends R1/R2 and C1–C4. R1 copies reviewed profile/decision resources and source-lock
runtime manifest, not Python itself. R2 adds only approved artifact/closure and
explicit policy exceptions for required distribution metadata/licence names.
No asset redesign, blanket bundle size exception or whole-dev-environment copy.

**3 Implementation.**
- [ ] Assemble chosen runtime route, profile and notices; validate manifest/hash,
  installed entry-point groups and dependency closure from relocated package.
- [ ] Preserve one-shot package entry point/source parity and user save/cognition
  paths; disabled/default rule profile remains model-independent.
- [ ] Run real packaged 2D/3D/both windows with same semantic world/save and selected
  LM resident. Record startup/usable frame, meaningful response, correction recall
  after close/reopen and shutdown; keep event/request/citation evidence alongside.

**4 Validation.**
- [ ] Package self-tests and standard smoke; R1 explicitly tests missing prerequisite
  degraded path and configured installed runtime success; R2 clean environment
  excludes source checkout, development venv, user site and build-tool PATH.
- [ ] Actual rendered dialogue and walk/interact in both clients; both-window exit
  semantics; packaged rule-only model-absent run; no hidden downloading/connect.
- [ ] Retain checksums, exact source/export identity, config/lock/runtime manifest,
  licences, size report and process cleanup evidence. No passing source run reused
  as proof of package contents or current exported GUI behavior.

**5 Acceptance.** Declared route runs through a relocated package; actual players
can speak/receive contextual corrected recall across restart; no-owned-child
leak. R1 is labelled prerequisite-dependent, never self-contained. R2 meets its
clean-runtime gate, target architectures and existing compressed budgets or stops
for explicit revision. Each client receives server facts via shared rules.

**6 Failure/edges.** Missing native wheel/interpreter arch/metadata fails named
preflight, never silently switches source Python. Read-only package directory and
paths with spaces/Unicode work. New world/save slot cannot reuse prior nonce.

**7 Commands/evidence.** `python3 scripts/package.py --self-test`; selected
`assemble`, `budget`, `smoke` using existing artifacts only after heavy-slot/resource
approval and exact exported args recorded. Actual GUI sessions via UI tools.
§7 budgets one final packaged live sequence; source/plumbing checks use
scripted/replay backends. If diagnostics consume live calls, all attempts count
and any unmet package proof remains pending until a reviewed extension. NOT RUN.

**8 Boundary/review.**
- [ ] Independent review of assembled file manifest, prerequisite disclosure,
  licences/runtime closure, source/export fingerprints, actual window evidence,
  startup/performance and process lifetime; maintainability review per CLAUDE §3.4.
- [ ] Final ledger/diff/staged review, one semantic delivery commit, then normal
  authorized review/CI endpoint. Never merge or publish from this planning lane.

## 6. Verification ownership and result ledger

| Layer | Unique claim | Status |
| --- | --- | --- |
| Static | Rust/Python types, format/lint, provider/ownership guard scans, doc links/decision IDs | NOT RUN |
| Deterministic integration | Absolute admission/dispatch bound, cleanup/accounting, assignment/store isolation, causality, access, replay and identity | NOT RUN |
| Gate 1 | Local installed model produces relevant acknowledgement and latest corrected recall with cited earlier code, within cap | NOT RUN; proposed budget needs approval |
| Gate 2 | Real subprocess EOF/cleanup, actual save+home restart and actual exported windows/runtime closure | NOT RUN; no existing historical evidence substituted |
| Terminal CI | Exact final candidate repository checks under existing CI policy | NOT RUN; public endpoint unresolved here |

Development uses targeted tests. Full local suite default zero; no manual remote
full CI. Repository-required checks and existing selector policy are preserved.
Report exact commands, counts, wall time, configuration, HEAD+diff fingerprint,
artifacts and decisive observations; PASS/FAIL/INCONCLUSIVE only after inspection.
Do not optimize test count or duplicate a real lifecycle with an elaborate fake.
Review covers technical debt, readability, owner boundaries, robustness, coupling,
extension, size growth and restart behavior. Material new debt is fixed/escalated.

## 7. Proposed bounded local live gate (not execution authority)

One serial heavy slot, one resident, already installed qwen3.5:4b,
`http://127.0.0.1:11434/v1`, no key/secret env file, no cloud, download or service
start. Direct configured interpreter. `structured_output=none`, `reasoning=off`
are historical adapter candidates, not assumed currently compatible.
Social tier local only; ordinary/major/summarize none; consolidate retention;
`max_in_flight=1`, `max_calls=1`, no automatic reasks, D=20s absolute,
<=256 output tokens/call, existing <=6000-byte full person memory ceiling.
Durable per-seat ledger retained; harness imposes <=8 actual backend calls TOTAL
across the packaged sequence/restart/failed attempts, not per-session or per-seat reset.
All incidental arrival/feeling/due triggers count. An 8-call cap is a harness
admission bound, not inferred from calls/hour settings. Inference wall <=180s
TOTAL, each real lifecycle gate <=10 minutes; total heavy execution envelope to
be approved by coordinator after runtime/profile choice. No CPU/GPU concurrency.

Run one final packaged live sequence after deterministic source checks; its
Gate-1 assertions concern answer semantics and its Gate-2 assertions concern
actual package/runtime/process reachability. This is one run, not duplicated
source and package semantic campaigns. Use runtime-generated short nonce instead
of MAPLE-47 constants in production prompt. Sequence: initial code/location -> relevant answer -> leave/rejoin ask
-> correction to library with original code unchanged -> unrelated short exchange
-> close all -> same save/Person/home reopen -> current location/code question.
Question does not contain either answer. Require newer correction and original
code in shown context, valid citations and actual caused saved NPC utterance;
review answer meaning, not token existence alone. At least one unrelated resident
action during stall is recorded. No need to call live model for every privacy,
identity, deadline or cleanup negative test: deterministic source tests own those.

Compare rule-only baseline and one-LM profile: first feedback/usable frame,
other-resident action/tick latency, answer latency, CPU and unified-memory pressure.
Report observations, not invented acceptable capacity. Stop admission at cap or
machine-pressure failure; save world and retain evidence. No automatic increased
cap/model download/provider substitution. A failed meaningful answer is FAIL;
an unavailable prerequisite preventing the path is INCONCLUSIVE.

## 8. Filled execution contract — pending approval fields explicit

PROJECT / PR: MineWorld playable contextual dialogue and persistent memory.
PRIMARY DESIGN: this file, revision DM-20261123-1.
RELATED / BINDING: §1 parents and policies; full Structured Coding
`prompts/implementation-working-rules.md` and `prompts/test-ci-gate-rules.md` must
be read by the fresh execution owner in full; their bodies remain in place and
are not replaced by this contract.
IMPLEMENTATION BASE: unresolved exact reviewed town/admin predecessor; audit base
b457a94d as §2. R1/R2 selection and profile approval unresolved; actual available seat mapping
is audited in §3 and must be rechecked after predecessor rebase.
APPROVED SCOPE: none yet; proposed §1, C1–C5.
FROZEN INVARIANTS: none manufactured; proposed §4.
APPROVED IMPLEMENTATION SEQUENCE: none yet; proposed C1–C5.
AUTONOMOUS VALIDATION BUDGET: current design dispatch permits read-only audit and
planning writes only; no build, inference, download, paid/public writes or children.
Proposed execution: targeted ordinary local checks under shared heavy-job slot;
Gate 1/2 bounded by §7 only when operator explicitly accepts this execution contract.
REQUIRED LIVE DOCUMENTATION: this file; preserve failure/deviation evidence.
HANDOFF: `handoff-playable-dialogue-memory.md`, created by fresh execution owner;
not a second design authority. Record base/head/tree fingerprint, exact next action,
active processes, budgets, authority sources and unresolved blockers.
PR CONTEXT: THIS PR only; start fresh, inspect git/process truth, read full design,
parents and execution/test rules, verify freeze/base/prerequisites and initialize
handoff before any implementation edit. Compaction resumes same PR and rereads
full design and contract; never invent a new freeze.

ENDPOINT AUTHORITY (each source explicit):

| Endpoint | Current authority | Source |
| --- | --- | --- |
| This isolated planning document/evidence | Authorized | Coordinator design-only dispatch |
| Source implementation/local tests | Not started; requires approved freeze | CLAUDE §3.1; coordinator no-source-implementation instruction |
| Semantic commits | No planning-lane commit performed; execution value unresolved | Coordinator planning-only/no-public-write dispatch; must reconcile final execution endpoint before freeze |
| Push / PR create / PR update / public CI action | Not authorized in this planning lane; execution value unresolved | Coordinator no-public-write dispatch; no inherited execution approval asserted |
| Live inference / downloads / paid endpoints | Not authorized here; §7 is proposal only | Coordinator dispatch and operator amendment §5 |
| Merge | Explicit operator approval for exact merge only | CLAUDE §3.1 |

NORMAL STOP: current lane stops after independently reviewable design+contract and
remaining decisions delivered to coordinator. Fresh execution endpoint must be
filled from actual operator authority; do not silently inherit this planning-only
endpoint into a future approved run. Standard proposed execution endpoint is
READY FOR OPERATOR REVIEW after implementation, deterministic/local gates,
independent review, committed final content and required exact-head CI; coordinator
must reconcile publication/CI authority before freeze rather than narrow by habit.
POST-MERGE SYNCHRONIZATION: execution owner updates PR evidence/result; coordinator
owns parent/operator amendment/overall/progress page; reconcile their dirty state,
never overwrite. This is not merge authority.
STOP/REVIEW: material change to scope/invariants/public ownership, runtime dependency,
retention semantics, packaging target/size or resource budget returns dependent
work to operator with smallest concrete revision. Routine failures inside an
approved contract are fixed autonomously. All pending tests remain NOT RUN.

## 9. Coordinator freeze checklist and dispatch payload

- [ ] Independent red-team reads this design and full audit; findings/dispositions
  recorded here. This drafting worker did not review itself as independent.
- [ ] Select R1 or commission bounded R2 artifact prototype; record exact runtime
  manifest/source-lock and selected world/resident/human seat mapping.
- [ ] Resolve exact town/admin predecessor SHA and compare its actual caller chain;
  do not steal launcher owner files or root dirty checkout.
- [ ] Record explicit operator design approval, live budget and final execution
  endpoint sources. Only then add DESIGN FROZEN header/revision/evidence.
- [ ] Delegate fresh execution owner with unique isolated tree, base SHA, paths
  C1–C5, this contract, acceptance §4/§5/§7, shared heavy limits and reviewer capacity.
  Coordinator may route C1 as its own bounded approved repair first; if so remove
  its execution checkbox ownership here and consume its exact reviewed commit.
- [ ] Keep progress page honest: source implemented, package integrated,
  independently reviewed and player accepted remain separate states.

Current conclusion: this is a concrete audited implementation plan, with known
runtime/base/approval decisions exposed. It does not claim a frozen design,
working contextual package, clean-machine runtime, current live result or a test
pass. Source implementation has not begun.
