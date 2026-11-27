# Playable MVP clarification and integration amendment — 2026-11-23

Status: operator requirements recorded; implementation amendments below are DRAFT,
not a manufactured design freeze. Parent: `overall.md`. Existing frozen module
contracts remain authoritative until explicitly amended. This document owns the
new requirement reconciliation, not a replacement ledger for existing PRs.
Coordinator: primary. Independent review: `delivery_review`.

## 1. Required player experience

The operator's actual trial found long Godot-only startup, stationary residents,
unanswered speech, inaccessible expected clock controls, and a night-time town
without the expected home/sleep/shop rhythm. A package that starts and connects
but fails that experience does not satisfy the requested first MVP.

The operator now explicitly requires an actually playable demonstration of the
framework: save and resume; explore; converse with residents who respond and
remember; buy goods; eat and drink to replenish a visible need; configure these
mechanisms and enable or disable them. The example world uses USD, a player
starting endowment of USD 10,000, realistic food/drink/snack prices, and daily
food-energy requirements expressed in kilocalories. Other authored worlds must
be able to choose different currencies and mechanisms.

The example must consist of configured entities, items, places and systems.
Dollar symbols, player endowment, item prices, nutrition quantities, shop schedules
and personal home assignments are content or owning-system configuration, never
kernel/client special cases. This does not require giant classes: retain existing
Entity/Component/System ownership and typed configuration. Kernel ignorance,
authoritative simulation, removable packs and source/export parity remain binding.

This expands or clarifies acceptance, never reduces the remaining original MVP
scope. The November 24 target remains a target, not evidence of feasibility.

## 2. Audited present state and disposition

Audit base: delivered source `703aa8dce66c3fa150942315e74bffd346fcfa8a`;
export `e3f207186ea76d562800dbd6c53cb40daf9dacc1`. Evidence and source paths in
`/tmp/mineworld-living-world-audit-20261123.md`.

| Requirement | Established state | Disposition |
| --- | --- | --- |
| Save/resume | Bounded packaged replay passed; no completed full gameplay-loop replay | Preserve prior evidence; repeat with meaningful purchases, consumption and remembered interaction |
| Visible startup | Runtime had no loading UI; staged correction in progress | Existing presentation correction, separate owner and review |
| Residents act/reply | Packaged launcher supplies only reactive Alice, no town controller; source played launcher already supports town | Bounded launch integration correction; validate non-Alice actions and actual replies |
| Meaningful dialogue/memory | Offline rule replies are probabilistic fixed English templates quoting observed speech; LM memory modules exist but are not wired/proven in the delivered trial | Do not claim template replies satisfy contextual dialogue or durable memory; audit controller/memory integration and execution route |
| Time controls | Existing presets 4/2/1 real hours per world day, default 2; package lacks local admin capability | Restore existing UI through private per-run host token; no new UI or altered movement speed |
| Language | General language selection and Apply already exist; S30-k attribute localisation unfinished | Document exact boundary, test exported switch; do not claim all content translated |
| Shopping/consumption | Market Town enables economy, inventory, food, sleep and needs; delivered default Social Cafe does not | Validate explicit richer-world selection with matching 3D depiction and separate save; do not silently repurpose a save |
| Currency | Market Town already declares USD name/symbol/exponent in economy configuration; amounts are integer minor units | Reuse configuration; test a second currency without client/kernel edits |
| Initial USD 10,000 | Market Town visitor currently has 200000 cents, i.e. USD 2,000 | Proposed demo content amendment: 1000000 cents for new player genesis; existing balances must not be silently reset |
| Food energy | Current needs use 0–1000 hunger and authored restoration values, not kcal | New typed nutrition/energy design required; relabelling old hunger as kcal is incorrect |
| Realistic prices | Current content has authored US-sized prices; this audit establishes no current retail provenance | Collect dated retailer/menu references and serving sizes before claiming realism |
| Homes/sleep | Routines name shared apartments; Market Town sleep exists, not private homes or beds for every resident | Verify night/morning behavior; design personal residence assignment separately if required |
| Shop hours | Current economy purchase resolution has no time/staff-presence check | Genuine owning-system contract gap; lights/doors alone cannot implement business hours |
| Sleep skip | S35 Q-S35-9 excludes shared-world skipping and defers single-player; host allows high scales only when solo | New single-player proposal required; do not equate clock rescaling with simulated overnight life |

S35 Q-S35-7 explicitly keeps Social Cafe and Bodies Yard as removal witnesses.
Do not add sleep/needs there merely to make a screenshot pass. Market Town has
an existing matching 3D depiction. Its real packaged play path remains to be
validated; previously testing Social Cafe does not prove it.

## 3. Already authorized bounded execution

Startup correction continues in its isolated owner tree. Launcher living-world
correction uses a separate owner tree on the reviewed source, touching launcher
code/contracts/tests rather than the startup owner's files.

The operator explicitly answered **“同意复用现有 getrandom 库”** to the concrete
C2 proposal in the audit above. This authorizes the narrow ARC-78 standard-library
exception to reuse workspace-pinned getrandom for a per-run OS-random admin secret.
No new paid service, secret logging, argv token or saved token is authorized.
Generate independently from the invitation; fail closed on entropy failure;
pass only to owned local children via environment. Preserve supervisor shutdown
and both-client lifetime semantics. Do not introduce `--hosted-here`/keep-running
behavior or blanket solo authority for both-client mode through this repair.

## 4. Concrete proposed acceptance for the playable integration

Each row needs exact-source/export identity and independent review. These are
proposed end-to-end checks to reconcile with existing module acceptance, not
claims that checks ran or a newly approved design freeze.

1. Start the selected living world with usable loading feedback; move between
   places and interact using the actual packaged client.
2. An unoccupied non-player resident travels under a controller. A player speaks,
   receives a relevant response, and later can verify a remembered prior exchange
   after leaving/rejoining and after save/restart. Event-log existence alone is
   insufficient: recall must influence an observable response without fabricated
   knowledge of undisclosed events.
3. A new demo player starts with USD 10,000. Buy a specified serving: stock,
   inventory and wallet change by the authoritative configured amount exactly
   once; rejected purchases change none of them.
4. Eat/drink that serving: inventory decreases and the owning need changes by
   its configured nutritional contribution exactly once. The displayed energy
   reflects authoritative disclosed values; zero-calorie drinks must not create
   food energy. A daily target and serving units must be explicit.
5. Save, close and reopen: retain wallet, inventory, need state, time and the
   supported memory state; do not regrant starting money or reapply consumption.
6. Change existing 1/2/4-hour day setting and language through the exported UI;
   verify the server clock and persistence. Walking/physics/dialogue pacing must
   not scale with day length.
7. Observe a night-to-morning routine in the living world: configured home
   destination, sleeping and waking, and business availability where supported.
   Do not require empty streets by fiat: night workers are a content choice.
8. A second configured world changes currency/endowment/nutrition or removes the
   relevant optional system. No renderer/kernel code edits, orphan settings,
   silently ignored keys or fabricated substitute behavior are allowed. Invalid
   dependency combinations fail with an actionable configuration error.

## 5. Single-player sleep advance proposal and unresolved decisions

Proposed behavior: a permitted sleep action in a solo world presents a sleeping
screen while the authoritative host processes the intervening night, stopping at
the requested morning or an earlier wake/interruption. Save the resulting world
normally and resume the prior day-length setting. Shared worlds do not jump time
because one person sleeps.

`kernel/src/advance.rs` already skips idle instants while firing due facts/process
wakes in order. That alone is insufficient: hosted controller decisions are
scheduled separately in `server/src/runtime.rs::consult`; merely calling
advance_to(morning), or increasing time scale while retaining wall-paced agents,
can omit their overnight actions. The design must explicitly interleave controller
decisions, movement completion and scheduled events without inventing simulation
rules in the kernel. Cancellation, earlier wake, failure and restart must have
deterministic, persisted results. Do not implement a direct timestamp jump.

The operator approved **2000 kcal/day, configurable** on November 23. This
fixes the example default, not a physiological model.

Food energy, serving units, overrides, the bar and old saves are designed in
[`step-fe-food-energy.md`](step-fe-food-energy.md) (engine PR FE-a, Market Town
content PR FE-b); its §1 records the 2026-11-27 rulings.

Still to resolve before dependent implementation:

- Explicit nutritional serving units and optional per-person overrides.
- How energy intake/expenditure maps to the visible bar; what happens below
  target. Existing S35 explicitly forbids forced collapse/illness consequences;
  those remain excluded unless the operator revises them.
- Configuration at world creation versus live system removal in an existing
  save. Do not promise destructive hot-removal or automatic migration.
- Contextual conversation/memory integration. Read-only `ollama list` on this
  Mac confirms an existing `qwen3.5:4b` model (reported 4.0 GB); no model was
  downloaded or run. The existing local provider path is the first candidate,
  subject to real response/memory and shared-machine performance checks. No
  paid model calls are authorized by this requirement alone.
- Scope of personal residences, staffed opening hours and sleep advance in this
  first delivery, with concrete implementation designs and review checkpoints.

## 6. Delivery reporting

Report implemented, integrated into the supplied package, independently reviewed,
and manually accepted as separate states. Never substitute PR count, static
component existence or smoke-test success for the player loop. Existing successful
tests and original failures are retained. The visible progress page must list
actual trial failures and new acceptance gaps until each is resolved.


## 7. Scale and hosting roadmap requested by the operator

Status: advance planning, not authorization to deploy infrastructure or run new
large benchmarks. Operator explicitly asks to preserve code cleanliness and
performance as worlds grow from tens to hundreds or thousands of residents;
local Mac play remains today's priority. No capacity claim is inferred from
small-town tests or design documents.

### 7.1 Preserve independently measurable costs

| Domain | Ownership and existing foundation | Measurement before optimization |
| --- | --- | --- |
| Authoritative world | Server/Kernel plus owning System Packs; scheduled semantic instants, not render frames | Tick CPU/wall p50/p95/p99/max, due-entry backlog, action latency, facts/sec, save commit latency |
| Resident decisions | Controllers propose intents; world revalidates; deterministic daily behavior independent of models | Consults/sec, cost per consult, queue wait, timeout/fallback rate; distinguish rules from LM calls |
| Model inference and recall | Separate cognition/provider/memory interfaces; wall-time budgets already in ARCHITECTURE §9.1 | Concurrent inference, input/output tokens, first-token and full-response latency, recall latency, per-world costs |
| Client presentation | Render only disclosed state; geometry/assets/lighting remain client-owned | First feedback/usable world, frame p95/p99, visible entities, draw calls, texture/mesh memory |
| Persistence and replication | Authoritative event history, snapshots, bounded perception/deltas | Bytes per simulated day, recovery duration, retention, bytes/sec/client and observation construction cost |

Existing architecture explicitly separates render, simulation, network and
cognition rates. Preserve it. Idle Process time already skips work; this does not
prove that perception, controllers or memory avoid scanning every entity.
Profile those paths rather than assuming overall complexity from one subsystem.

### 7.2 Initial scale experiments and interpretation

Use generated, schema-valid worlds and a fixed reference machine/configuration.
When their prerequisites are ready, measure 20, 100, 300 and 1000 residents in
headless rule-only mode; then repeat selected sizes with a client and separately
with a controlled subset of model-driven residents. Include dense shared places,
not only sparse neighborhoods. Report wall-time versus simulated-time rates,
actual controllers active, visible residents, configuration, seed and source head.
Do not conflate a fast headless 300-day run with playable latency at 1000 NPCs.

Reuse the existing S28-g scale design for its memory-specific 12/100-person,
300-day contract; do not claim it has executed without exact-head evidence, and
do not turn its memory results into full-server capacity claims. Larger interactive
capacity testing is a distinct future checkpoint, after the present player loop.
No heavy run is scheduled by this roadmap.

Illustrative capacity math, not a benchmark: 1000 residents consulted every five
seconds means 200 rule decisions per wall second. If only 10% need one model
call every five minutes, that is about 0.33 calls/second, versus 3.33 if all 1000
call that often. Input/output tokens, model speed and burstiness still determine
cost and hardware. Model work must therefore have explicit admission limits,
bounded queues, event-driven activation and fallback; adding residents cannot
silently multiply paid calls without an operator budget.

Optimization candidates only after measured bottlenecks: spatial/interest indexes,
subscription-filtered events, bounded memory retrieval and caches, asset reuse,
instancing and level-of-detail, batching independent inference, and avoiding
per-frame whole-world scans. Off-screen rendering may be reduced; authoritative
life and scheduled events may not be silently discarded. Any simulation-level
approximation requires its own semantic contract and parity evidence.

### 7.3 Deployment progression and upgrade triggers

1. **Today: one local Mac.** Server, client and optional local cognition may share
   the machine but remain separate modules/processes. Offline rule-only behavior
   must keep the world valid. Record CPU, RAM, GPU/unified-memory contention,
   disk growth and usable-start latency before choosing hardware.
2. **First hosted world: one dedicated authoritative server process per world.**
   Run the renderer on players' devices. Place inference behind the existing
   provider interface, locally or on a separately provisioned service. CPU/RAM/
   storage primarily serve simulation and persistence; a GPU is justified by a
   measured local model requirement, not merely by resident count. Deployment
   needs authenticated access, bounded requests, backups and tested restore;
   publishing a loopback prototype directly is not a deployment design.
3. **More independent worlds: distribute whole worlds across hosts.** Each world
   retains one authoritative owner and its own budgets and persistence. Add
   placement and admission control when actual utilization requires them. Model
   workers may serve multiple worlds with explicit per-world limits.
4. **One world exceeds one host: separate design checkpoint.** First profile and
   optimize the existing owner. Sharding a single world by region is not an
   automatic next step: cross-region movement, transactions, time ordering,
   recovery and ownership transfer need explicit contracts and tests. Do not
   insert a distributed framework, microservice mesh or new database preemptively.

Upgrade only when measured action/tick/response latency or memory/storage bounds
miss the accepted playable target at the desired load. Choose host size from
measured peak memory plus headroom, CPU tick/decision demand, disk latency and
retention, network traffic and model concurrency. Numerical machine sizes,
provider selection and monthly cost remain unestimated until these inputs exist.
No external hosting purchase or paid inference is authorized here.

### 7.4 Maintainability and performance review on current work

Continue CLAUDE §3.4 independent review for each execution lane: ownership,
readability, robustness, extensibility, coupling and actual size growth. Existing
~50/~100-line function and ~500/~800-line file thresholds trigger review, not
mechanical splitting. Avoid duplicated business logic in clients, giant manager
classes, speculative extension layers, per-item special branches and hidden
unbounded caches. Keep startup and launcher repairs scoped and composable.

Every optimization states a measured bottleneck, a comparable before/after path,
correctness invariants and memory/latency tradeoffs. Preserve failed measurements
and uncertainty; an animated loader is a feedback improvement, not proof of
faster loading. Tests that merely duplicate implementation do not establish
scalability or correctness.


## 8. Dual-use framework clarification and expert-review disposition

The operator explicitly clarified on November 23 that games and agent/social
simulation must both work on the same separated backend. Not every resident
needs an LLM; each decision implementation plugs into the Controller contract.
This supersedes the coordinator's earlier interpretation excluding social
simulation as a use. CLAUDE §1.1, VISION §1.2/§3/§7.3 and ENGINEERING_RULES §1
record the clarification. It preserves playable MVP delivery and both clients.
It does not launch a separate benchmark, paid experiment or institution simulator.

The supplied expert review's architectural distinctions are useful: Person is
independent of Controller, ActionIntent is not authoritative Event, and simulation
is independent of cognition. Its strongest proposed verification is behavioral
composition, not merely modules in different directories. Accept that verification
need without claiming that new institutions, banking or arbitrary novel actions
already emerge. Software plugin authority remains different from a resident's
in-world permissions. No assertion that another platform is categorically less
capable is established by this review.

Concrete compatibility checks to incorporate into the owning integration designs:

1. Run the same configured world headlessly, with 2D and with 3D. For the same
   seeded scripted intents at specified semantic instants, compare authoritative
   outcomes and persisted history; rendering alone must not change them. Actual
   human/model decisions may differ and then histories should diverge.
2. Drive one Person through rule → human → model or scripted-controller takeover
   and release. Keep identity, holdings, relationships and world history; preserve
   each memory implementation's documented access/rebuild rules rather than
   asserting automatic transfer of arbitrary internal model state.
3. Exercise composed shopping/consumption/needs/persistence through each relevant
   interface. No duplicated business logic in clients, no client-written balances
   or simulation flags, no per-demo action added merely to manufacture success.
4. Run with all models absent and with renderers detached. Existing deterministic
   controllers and Systems continue. Model failure fallback and budget limits
   remain explicit, not hidden network retries.
5. Research instrumentation must not give participant Controllers omniscience.
   Aggregate/event analysis is a separately authorized observer or offline tool,
   never an unrestricted state channel in ordinary Observation.

Use typed interfaces, data-defined instances and declared System dependencies.
Do not interpret the operator's word “overload” as requiring deep class hierarchies
or separate game/simulation engines. A new interaction belongs to an owning
System; a new decision strategy belongs to a Controller; a new view belongs to
Presentation. A cross-System contract failure is reviewed rather than patched
with scenario-specific branches.
