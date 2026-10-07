# Step 08 / PR 09 — Headless demo: World Pack loading, rule controller and the CLI (S7)

**Role:** combined step and PR document. S7 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S7, artefact note `ARC-6`, §7 ·
**Lifecycle:** `DESIGN FROZEN` (2026-10-06, primary session; review and answers in §10.1)
**Base:** `main @ ef53484` (S6 merged as `6f61582`; `ef53484` is the docs-only post-merge update)
**Branch / worktree:** `mvp0/pr-09-headless` in `/Users/yuema137/mineworld-worktrees/s7-headless`
(held by this session only; `vis-character` and `vis-environment` belong to other agents)
**Depends on:** S4's clock and schedule, S5's `PersistentWorld` and `ARC-25`, S6's `MovementSystem` and
`ARC-26`, S5V's World Pack loader (05c), `RuleController` (05d), `mineworld server` (05b)

Binding: [`CLAUDE.md`](../../../CLAUDE.md) §4 rules 2, 3, 9, 10, 11, 13, 16 ·
[`docs/MVP.md`](../../../docs/MVP.md) §7 (artefacts), §9 `AC-6`, `AC-9`, `AC-11`, `AC-12` ·
[`docs/MODULE_SPEC.md`](../../../docs/MODULE_SPEC.md) §§5, 8 ·
[`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) §§21–24 ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `ARC-6`, `ARC-23`, `ARC-25`, `ARC-26`, `DEP-6` ·
[`step-04-clock-scheduler-process.md`](step-04-clock-scheduler-process.md) ·
[`step-06-persistence.md`](step-06-persistence.md) §9.1 L-3 (`F-13`), §12 ·
[`step-07-movement.md`](step-07-movement.md) §10.1 Q4

## Why this is PR 09, and why the file is `step-08`

The overall plan calls this step S7. PR 08 was S6, so this ships as **PR 09**. The step-document number
follows the file sequence (`step-07-movement.md` was S6), so this is `step-08-headless.md`.

---

# 1. Goal

`mineworld run worlds/social-cafe --headless --seed 7 --days 300` runs the café for three hundred
simulated days with no renderer, no model and no server, every seated Person driven by a seeded rule,
and prints a fingerprint of the history it produced. Run it again with the same seed and the
fingerprint is the same; with another seed it is not. Kill it halfway with `--save`, run the same
command again, and it finishes the same world, byte for byte. `mineworld inspect` then says what the
save holds and that every fact in it has a cause; `mineworld create` gives a world author a pack that
`validate` and `run` accept.

The S7 checkpoint from `overall.md` §3 is the floor of this design:

```text
CP-1  AC-11  a seeded rule-based world runs hundreds of simulated days with no renderer and no model,
             with the REAL systems (presence, movement, conversation) — not S4's test-only ones
CP-2  AC-12  the same seed reproduces the run exactly; a different seed does not
CP-3  AC-6   a mid-run restart (a killed process, the same command re-run) continues correctly:
             identical to the uninterrupted run, byte for byte
CP-4  AC-9   every state change in a sampled window traces to an intent, a process, or an event
CP-5  ARC-6  create / validate / run / inspect exist as commands a person types
```

## 1.1 Scope

```text
cognition/rule-controller/   PacedRuleController: stateless, seeded, consulted on a fixed pace;
                             answers within a window, and takes initiative (greet, approach, wander,
                             use a doorway). RuleController unchanged. Depends on mineworld-movement.
systems/movement/            discloses a place's Passages to whoever perceives the place (Q4)
tools/cli/src/run.rs         the headless driver: pace schedule, perception, dispatch, resume point,
                             summary, history fingerprint; in memory or --save (ProcessCrash)
tools/cli/src/inspect.rs     what a save holds; the AC-9 cause check
tools/cli/src/create.rs      a new pack from an embedded template, validated before success
tools/cli/templates/         the embedded template pack (world.yaml, places/, people/)
tools/cli/src/main.rs        argument parsing for run / inspect / create; USAGE
tools/cli/tests/             run.rs (AC-11, AC-12), run_restart.rs (AC-6), inspect.rs (AC-9),
                             create.rs; commands.rs's "does not exist" test re-pointed
docs                         MODULE_SPEC §8 (the CLI as implemented), DECISIONS ARC-27, ARC-26 dated
                             note (Q5), MVP_STATUS, READMEs (tools/cli, cognition/rule-controller,
                             systems/movement)
```

## 1.2 Non-goals

```text
any kernel, contracts, persistence or server change     none is needed (§2.3); I-1
the server's --agent path and F-13 there                 recorded, stays S10 (§2.5, Q2)
world configuration (MAX_STRIDE as a pack setting)      deferred to the first pack that needs it (Q5)
new System Packs, schedules, processes, travel           S8 and later
an LM, an LM controller, recorded cognition             S10
tools/inspector beyond `inspect`'s summary              S8 (overall §3 cross-cutting)
performance benchmarking, CI workflows                  S13
save migration                                          S5 L-1, unchanged
~~`clap` — hand parsing stays (Q12)~~                    void: Q12 overruled, clap adopted (DEP-11, C1b)
```

## 1.3 Frozen invariants (frozen 2026-10-06; I-9 added from §10.1)

- **I-1 No layer below the composition root changes, except one disclosure.** No edit to `kernel/`,
  `contracts/`, `persistence/` or `server/`. The only System Pack change is Q4's `Passages` disclosure
  in `systems/movement`. S7 is a tool and a controller over merged contracts (`INV-12`).
- **I-2 A run's history is a pure function of its inputs.** Facts, journal and snapshots of
  `run --seed S --days D` depend only on the pack, `S`, `D`, the pace and the code. No wall clock, no
  hash-ordered iteration, no thread or task, no process id reaches them. Wall time is printed, on its
  own line, and never stored. The world instance identity is allocated per created world and is
  *excluded by definition* from `AC-12` (§2.6).
- **I-3 The paced controller is stateless.** `PacedRuleController::decide(&self, &Observation)` — the
  compiler, not a convention, says a decision is a function of the seed and the observation. That is
  what makes CP-3 hold with controllers in the loop (§2.5).
- **I-4 One request path.** The driver turns a controller's `ActionRequest` into an `ActionIntent`
  with `ActionIntent::allocate` and a monotonic `ActionId`, exactly as `server/src/runtime.rs` does;
  a resumed run's allocator starts past the journal's highest. A controller consumes observations
  only (`INV-13`) and never touches a world.
- **I-5 No floating point** in any new or changed code; geometry is integer millimetres.
- **I-6** The 330 existing tests stay green in substance. `RuleController` and `--agent` behave exactly
  as today (`AC-15`). The one test whose subject changes (`commands.rs`'s "a command that does not
  exist" used `inspect`) is re-pointed at a command that still does not exist, with the same claim.
- **I-7 The same command finishes the same world.** `--days D` names the world's age since genesis,
  so re-running a killed `run --save` completes the identical history; it never appends a second run.
- **I-8 `create` never overwrites** and never reports success for a pack `WorldPack::read` + `load`
  refuse.
- **I-9 Activity before determinism** (§10.1). Every seat has an accepted `move` and an accepted
  `talk` in every 30-day bucket, checked before any `AC-11`/`AC-12` comparison is made.

---

# 2. The question the brief asked first: what is actually left of S7

## 2.1 What exists, from source (`main @ ef53484`)

| Overall S7 item | State | Where |
| --- | --- | --- |
| World Pack loader with schema validation (`world.yaml`, `people/`, `places/`) | **done** (05c), passages added in S6 | `worldpack/src/{read,load,format,catalog}.rs`; refuses unknown fields by name (`DEP-10`) |
| `RuleController` | **done** (05d) — **reactive only** | `cognition/rule-controller/src/lib.rs`: answers the newest line per speaker, once, when `talk` is available; never initiates |
| `mineworld server <world>` | **done** (05b), `--save` (S5) | `tools/cli/src/main.rs` `serve`, `persisted` |
| `--agent SEAT` | **done** (05d) | `tools/cli/src/agent.rs` `drive` — over `WorldHost`, wall-paced |
| `mineworld validate <world>` | **done** (05c) | `main.rs` `validate`: read + load, prints ids and genesis count |
| `mineworld replay <world> --save DIR` | **done** (S5), not in the original S7 list | `main.rs` `replay` → `persistence::verify` |
| `tools/world-validator` (overall §3 cross-cutting, R-3) | **done in substance** | it *is* `mineworld validate`; validation derives from the Rust types via `serde`, so R-3's single validation path holds |
| `mineworld run <world> --headless --seed --days` | **missing** | `main.rs:203` answers `run` with "does not exist yet — it is S7's" |
| `mineworld create`, `mineworld inspect` | **missing** | same line |
| `AC-11` / `AC-12` with real systems | **missing** | only `kernel/tests/long_run.rs`, with test-only `Routine`/`Pager` systems |
| `AC-6` end to end with controllers | **missing** | `persistence/tests/kill_and_resume.rs` drives *scripted* requests, not controllers |
| `AC-9` sampled-window check | **missing** | causation exists on every `EventEnvelope`; nothing checks a run's log |

## 2.2 The finding that shapes the step: a headless café today does nothing at all

Three facts from source, taken together:

1. **No real system acts on its own.** `presence`, `movement` and `conversation` implement only
   `react`; none starts a process or defers a fact (`rg "start_process|\.defer\(|fn wake" systems/` →
   no match). Time passing changes nothing in social-cafe.
2. **The only controller is reactive.** `RuleController::decide` acts only when somebody has spoken
   to its Person (`lib.rs:109–127`). It never speaks first and never moves.
3. **Nothing else submits requests** in a run with no clients.

So `run --days 300` with today's controller would advance an idle clock and produce a history of the
five genesis facts — and that history would be *perfectly* stable and *perfectly* reproducible.
That is `ARC-23`'s failure exactly: an instrument that cannot see the thing it measures reports a
clean result. `AC-11` and `AC-12` "with the real systems" are only meaningful if people in the world
**do** things. S7's real work is therefore a controller that takes initiative, and a test that locates
the activity before it counts determinism (§5 adversarial criteria).

`AC-11`'s own wording supplies the shape: *"a seeded **rule-based configuration**"*. The seed belongs
to the rule. Nothing else in the workspace consumes randomness (`rg "rand::|thread_rng"` → none), so
the seed is exactly the rule controllers' seed.

## 2.3 Why the server path cannot be reused for `run`

`--agent` drives a seat through `WorldHost`. That path is correct for a hosted world and is
non-deterministic by design:

```text
server/src/runtime.rs:64–81    HostClock: world time = epoch + wall seconds elapsed (Instant::now)
server/src/runtime.rs:382–399  sweep: observations sent with try_send; a full channel drops frames
tools/cli/src/agent.rs:41      the agent is a tokio task; when it sees which observation is scheduling
```

A hundred days would take a hundred days, and which observation a controller sees depends on wall
time and task scheduling. `run` therefore needs its own driver — synchronous, on one thread, advancing
the kernel directly (`runtime.rs:62–63` already says *"a headless run … advances the kernel directly
and has no use for this"*). It calls the same three things the server calls and nothing else:
`presence::observe`, `ActionIntent::allocate`, and `World::dispatch` / `PersistentWorld::dispatch`
(+ `advance_to`). No kernel, server or persistence change is needed (I-1).

## 2.4 What S7 should and should not now include

**Include** — what is genuinely missing, and the least that makes each checkpoint honest:

```text
PacedRuleController        initiative, or CP-1/CP-2 measure an idle clock (§2.2)
movement discloses Passages   without it no headless Person can find the door, so the street is never
                           entered and S6's PersonEnteredPlace never fires in the AC-11 run (Q4)
mineworld run              the driver, in memory or --save; summary and fingerprint
mineworld inspect          a save's summary and the AC-9 cause check — the CP-4 artefact
mineworld create           the ARC-6 command; small, from an embedded template
ARC-27                     the decision that defines what a deterministic controller-driven run is
```

**Do not rebuild** the loader, `validate`, `replay`, the server, `--agent` or `RuleController`.

**Honest size.** One controller type (~250 lines with its rule), one driver (~250), two small
commands (~150 each), one disclosure (~20), tests and documents. It is a smaller step than S5 or S6.

## 2.5 `F-13`: handled for `run`, recorded for `--agent`

`F-13` (step-06 L-3): a restarted `--agent` answers its last line again, because *which line it has
answered* lives in `RuleController::answered` (`lib.rs:82–84`), controller memory rather than world
state.

**`run` must handle it**, because CP-3 compares a killed-and-resumed run with an uninterrupted one
byte for byte, and a controller that re-answers after the restart diverges at the first such line.
Recording it would make CP-3 either fail or be weakened to "the resumed run did not crash", which
is not `AC-6`.

**How, without adding world state:** remove the memory rather than persist it.

```text
pace P (default 600 s)   seat k (in the pack's seat order) is consulted at epoch + k + m·P, m = 0, 1, …
                         so no two seats are ever consulted at one instant (requires seats < P)
reply window             a line heard at h is answered by the consult at t only if t − P < h ≤ t
```

Every line is heard at the instant a *different* seat was consulted (only consults dispatch, and
no real system speaks on its own), so it falls inside exactly one consult window of its listener.
"Answer once" then holds with no record of having answered, and `decide(&self, …)` is a function of
the seed and the observation. A controller rebuilt after a restart decides exactly as the one that
died (I-3). A line whose window passes while `talk` is unavailable is not answered later — the
listener missed the moment, which is a rule, not a defect.

**`--agent` keeps `F-13`.** A hosted world has no fixed consult schedule: observations arrive many
times per world-second at the same instant, so it needs the dedup map, and the map is memory. Its
remedy is a perception or cognition change (S10, as step-06 recorded). S7 does not touch
`RuleController` (I-6).

## 2.6 Non-determinism audit, before claiming `AC-12`

Every source found, and its disposition:

| Source | Where | In a `run`'s history? |
| --- | --- | --- |
| `HashMap` / `HashSet` | banned workspace-wide by `clippy.toml` `disallowed-types`; `cargo clippy -D warnings` is a gate | no — new code uses `BTreeMap`/`Vec` |
| `Instant::now` | `server/src/runtime.rs:73` (`HostClock`) | no — `run` never constructs a host |
| `SystemTime::now` + process id | `server/src/protocol.rs:399–403` (`WorldInstanceId::allocate`) | **manifest only** — allocated when `run --save` creates a save. Not in facts, journal or snapshots (the kernel does not know it). Excluded from `AC-12` by name: two runs are two worlds. The test compares facts, journal and snapshots byte for byte and states that the manifest's instance differs |
| tokio tasks, channels, `try_send` | `server`, `agent.rs` | no — `run` is synchronous on the main thread |
| wall-time reporting | the new `run` summary | printed on a separate `wall` line; tests compare everything else |
| SQLite row order | `persistence/src/sqlite.rs` — every read has `ORDER BY` | deterministic |
| serde_json map order | `BTreeMap` (no `preserve_order` in the workspace) or insertion order — both deterministic | deterministic |
| entity / event / action ids | pack load order is `BTreeMap` (`worldpack/src/lib.rs` doc); event ids by the kernel; action ids by the driver's monotonic allocator | deterministic |
| seat order | `WorldPack::seats()` is a `BTreeSet<EntityKey>` | deterministic (alice, visitor, wanderer) |
| controller randomness | new: SplitMix64 over `(seed, observer, instant)`, a pure function | deterministic by construction |
| floating point | none in the workspace (I-5) | — |

---

# 3. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **HD-1** | `run` has its own synchronous driver in `tools/cli/src/run.rs`; it does not use `WorldHost`. | §2.3. The composition root is the one crate allowed to know packs, persistence and the controller together (`main.rs` doc). |
| **HD-2** | The pace schedule of §2.5: seat *k* at `epoch + k + m·P`; default `P` = 600 s. Before each consult the driver `advance_to`s the consult instant (fires nothing today; correct the day a system schedules work). | One seat per instant makes in-instant order moot and makes the reply window exact. 600 s gives 144 consults per seat per day — about 130 000 consults over 300 days for three seats, cheap in memory and affordable in SQLite at `ProcessCrash`. |
| **HD-3** | `PacedRuleController { seed, pace }` in `cognition/rule-controller`, `decide(&self)`. Rule, in priority order: (1) answer the newest in-window line from the lowest-id speaker `talk` is available against; (2) seeded initiative — greet someone `talk` is available against; approach a perceived person (a stride toward them, stopping about 1 m short); wander (a stride in a seeded direction); or step toward / through a disclosed doorway; (3) nothing. All choices are `mix(seed, observer, instant)`; replies reuse `RuleController`'s wording functions. | `AC-11`'s "seeded rule-based configuration". Initiative is what makes the run observable (§2.2). Statelessness is I-3. One crate, two types: the reactive one stays exactly as `AC-15` needs it. |
| **HD-4** | Every distance the controller computes is a *proposal*; the server decides. A stride is computed with integer `isqrt` and floor division, so its length never exceeds `MAX_STRIDE`, and `TooFarAway` refusals are counted, not avoided. | `ENGINEERING_RULES.md` §8: the controller measures nothing it acts on as a rule; it proposes a destination and reads the verdict. |
| **HD-5** | `--days D` is the world's age: run until `epoch + D·86 400 s`. A day is the CLI's unit, not the kernel's. | I-7: re-running a killed command completes the same world. `INV-12`: the kernel still knows only seconds. |
| **HD-6** | Without `--save` the world is in memory; with `--save DIR` it is a `PersistentWorld` at `Durability::ProcessCrash` (step-06 §12's note for S7). A save that exists is resumed. | The quick path stays quick; persistence is opt-in. `PowerLoss` (an fsync per revision) is for hosted worlds (step-06 Q5). |
| **HD-7** | Resume point: the first consult instant strictly after the head revision's instant — or *at* it, when the head input is an `Advance` (read from the journal's last entry), because then the consult at that instant had not run. The `ActionId` allocator resumes at `highest_action_id + 1`. | A consult that dispatched is journaled; a consult that dispatched nothing is re-run harmlessly because the controller is stateless. |
| **HD-8** | The run prints a **history fingerprint**: FNV-1a 64 over every fact's stored bytes in `EventId` order, with the fact count beside it. Purpose-built, ~10 lines, not a dependency. | A person can compare two runs by eye (`ARC-6`: something a person can run). It is a fingerprint for accidental divergence, not a security property — tests compare the logs byte for byte as well. Q9. |
| **HD-9** | `inspect <save-dir> [--last N]` reads a save only: manifest, head revision and instant, journal by input kind and outcome, facts by event type and by cause kind, the `AC-9` check, the last *N* facts. Read-only: it never resumes or writes. | CP-4's artefact. Generic: it names event types and causes, never a domain concept. |
| **HD-10** | The `AC-9` check: every `Action(id)` cause names a journaled dispatch; every `Event(id)` cause names a fact with a smaller `EventId`; `WorldGenesis` appears only in revision 1; `Process` and `SystemTick` causes are counted (a process id is itself a trace; none occur in social-cafe). | `AC-9`'s three kinds — intent, process, event — each checked as far as the log alone can check it, and the limit stated. |
| **HD-11** | `create <directory>` writes an embedded minimal template (one place, two people, both seats; presence, movement, conversation), substituting the world id after checking it is a valid key; refuses an existing directory; reads and loads the result before reporting success. | `MVP.md` §7: how a world author first touches MineWorld. Embedded, so the binary works away from the repository. Reading it back is the fail-closed check that the template and the format have not drifted (I-8). |
| **HD-12** | `run` drives every seat the pack offers, and only seats. | A seat is what a controller may occupy (`world.yaml`); a non-seat person (bob) is not driven by anyone, exactly as on a server. |

---

# 4. Commit plan

Each commit lists implementation, validation and review separately. Evidence is recorded in §9 as each
item completes. Line counts are estimates from the audit, not targets.

### Frozen-answer amendments (recorded at the start of execution, from §10.1)

```text
Q12 overruled   C1 also records DEP-11 (clap, derive). A new commit C1b migrates server, validate and
                replay to clap BEFORE any new command, with every existing CLI test unchanged and
                green across it; main.rs's hand-parsing note becomes a pointer to DEP-11. §1.2's
                "clap — hand parsing stays" non-goal is void. C4, C6, C7 add their commands as clap
                subcommands.
Q4 condition    C2 gains a far-side check: the real Godot protocol module handles an observation
                carrying the passages record, through a real clients/protocol/run.sh run; evidence
                recorded in §9.
Q9 condition    The fingerprint is display only. Every equality claim in a test compares the bytes
                of facts, journal and snapshots (and stdout text), never fingerprints.
I-9 (frozen)    The activity precondition: before any determinism comparison, every seat has an
                accepted move AND an accepted talk in every 30-day bucket of the run. A comparison
                made without it is not evidence.
```

## C0 — Design (this document) — docs only

- [x] Implementation: §§1–11 written from the source audit of §8.
- [x] Validation: `python3 scripts/check_decision_ids.py` (35 ids, all distinct), `python3 scripts/check_doc_headings.py` (141 sections, none duplicated) — §9 E-0.
- [x] Review: every claim in §2 cites a file and line or a command; no material finding is routed around (§8.3).

## C1 — Specification amendments, before code

**Goal.** The CLI and the meaning of a deterministic controller-driven run are specified before they
exist (`CLAUDE.md` §2.2). **Scope.** `docs/MODULE_SPEC.md` §8, `docs/DECISIONS.md`. **Depends on:** the freeze.

- [x] Implementation (§9 E-1):
  - [x] `MODULE_SPEC.md` §8 (new §8.1): add the implemented command surface — `server`, `validate`, `replay`, `run`, `inspect`, `create`, with their options and the meaning of `--days` (HD-5) and `--seed` — keeping §8's install/add-system lines marked as intended, not implemented.
  - [x] `DECISIONS.md` **ARC-27** — *a headless run is a pace schedule over stateless seeded controllers*: problem (§2.2, §2.3), options (reuse the server path; scripted inputs; a stateful controller with persisted memory; the paced stateless controller), choice HD-2/HD-3/I-3, what `AC-12` covers and excludes (instance id, wall time), `F-13` resolved for `run` and kept for `--agent` (§2.5).
  - [x] `DECISIONS.md` `ARC-26`: dated note that `MAX_STRIDE` stays a constant past S7 (Q5).
  - [x] `DECISIONS.md` **DEP-11** — the CLI's argument parsing: `clap` with derive; the trigger that was met (six subcommands, `run` with four options), alternatives (hand parsing, `argh`, `pico-args`, `lexopt`), isolating interface (`main.rs` only), accepted limitation (build time).
- [x] Validation: both check scripts pass; `ARC-27` is unique on every remote branch (`git grep` over `refs/remotes/origin`). — §9 E-1.
- [x] Review: the CLI text in §8 matches §3 exactly; no synonym for a defined term (`Controller`, `Observation`, `World Pack`); `ARC-27` says what it excludes.

**Acceptance.** A reader of MODULE_SPEC §8 and ARC-27 alone can implement `run` and predict its output's determinism.
**Failure cases.** None executable. **Commit boundary.** Docs only.

## C1b — The existing commands move to `clap` (DEP-11), and nothing else changes

**Goal.** Adopt the parser before the new commands, so the migration is reviewable on its own and the
existing CLI tests are its regression net. **Scope.** `Cargo.toml` (workspace dependency),
`tools/cli/Cargo.toml`, `tools/cli/src/main.rs`. **Depends on:** C1.

- [x] Implementation: `clap` (derive) parses `server`, `validate`, `replay`; `create`/`inspect`/`run` still answered "does not exist yet" (hidden subcommands taking any trailing arguments); the `Why the arguments are parsed by hand` doc becomes a pointer to `DEP-11`. — §9 E-2.
- [x] Validation: every existing `tools/cli/tests/*` test passes **unchanged** (no test file in the diff); complaint texts the tests assert still appear. — §9 E-2.
- [x] Review: no behaviour change beyond help formatting. One bounded deviation: a parse error now exits with clap's status 2 rather than 1 — still non-zero, still a message naming the argument, never a panic (no test or document asserts 1). — §9 E-2.

## C2 — Movement discloses a place's doorways (Q4)

**Goal.** A Person perceiving a place can know where its doorways are, so a controller can walk out.
**Scope.** `systems/movement/src/system.rs` (`PerceptionProvider::discloses`), its test file, README.
No change to `move`'s rule, to `Passages`, or to any other pack. **Depends on:** C1.

- [x] Implementation: `MovementSystem::discloses(world, observer, subject)` returns the `Passages` record when `subject` is a place entity that has one, to any observer (perception already lists only the observer's own place, `observe.rs` `perceived`); nothing for people. `codec::to_value` added. — §9 E-3.
- [x] Validation: `systems/movement/tests/disclosure.rs` (3 tests) over the movement test town rather than social-cafe (the movement crate cannot depend on the pack loader; the town has the same doorway shape with literal coordinates) — the café's record names the street and both doorway positions; the street's side from the street; no record on persons; none for an unjoined place; none after `disable` while the state is still present. Mutation (invert the place check) fails all 3. — §9 E-3.
- [x] Validation (far side, Q4 condition): PASS — `clients/protocol/run.sh evidence`, real Godot 4.7.2 headless, five transcripts; see §9 E-3.
- [x] Review: no rule moved into disclosure (it states where a door is, never whether one may pass); the Godot client reads components by type (`clients/protocol/mineworld/observation.gd:102–108`); `AC-13`/`AC-15` tests unaffected.

**Acceptance.** The record appears exactly for place subjects and disappears with the system.
**Failure cases.** A place with no passages discloses nothing (not an empty record).

## C3 — `PacedRuleController`

**Goal.** A controller that takes initiative, deterministically, with no memory (HD-3, I-3).
**Scope.** `cognition/rule-controller/src/{lib.rs, paced.rs (new), tests.rs}`, `Cargo.toml` (+ `mineworld-movement`), README. `RuleController` unchanged. **Depends on:** C2 (doorways are read from the observation).

- [x] Implementation (§9 E-4):
  - [x] `paced.rs`: `PacedRuleController::new(seed: u64, pace: SimDuration)` (`contracts/src/time.rs`); `decide(&self, &Observation<Value>) -> Option<ActionRequest>` with the HD-3 priority; `mix` (SplitMix64) private; integer stride geometry (HD-4) against `MAX_STRIDE`. Bounded addition to HD-3: an in-window line is answered with probability 75/100 (seeded), so conversations end and people walk — otherwise two paced controllers in range answer each other at every consult forever and never move, which would starve I-9's move count.
  - [x] `lib.rs`: the four helpers stay where they are; `paced` is a child module and uses them through `crate::` (no move needed, no change to them); re-export the new type.
- [x] Validation (unit — deterministic local semantics this crate owns) — `cognition/rule-controller/src/paced_tests.rs`, 7 tests:
  - [x] window: answered (some of 64 seeds) when heard 10 s before; heard at `t` inside, at `t+P` never again (no seed), `t−P` outside (no seed), `t−P+1` inside. Mutation `opened <= at` → the window test fails, reverted.
  - [x] fresh controllers with one seed decide identically over 50 consults × 8 seeds; seeds 7 and 8 differ over 40 consults, and seed 7 acts at > 10 of 40 consults with nobody speaking to it (initiative located).
  - [x] stride sweep (axes, 1 mm, diagonals, 2 000 km) ≤ `MAX_STRIDE` and always closer; stop-short never overshot; every walk proposed from the café over 64 seeds × 16 consults is ≤ 2 000 mm; by the door, the crossing lands exactly on the street side of the doorway.
  - [x] greets only Bob (Sue unavailable); with no `move` offered and nobody to talk to, nothing from any seed.
  - [x] `RuleController`'s six existing tests unchanged and green (13/13 in the crate).
- [x] Review: `&self` makes I-3 structural; no `HashMap`, no float (`isqrt` on `u64`), no clock — the instant is the observation's; verdicts read through `may_talk_to` and the `move` affordance; submits only `talk` and `move`. clippy `-D warnings` clean.

**Acceptance.** All of the above, and `cargo clippy -D warnings` clean for the crate.
**Failure cases.** An observation without the observer's location: no move, may still answer. A reply that would exceed `UTTERANCE_MAX_BYTES`: silent, as `RuleController`.

## C4 — `mineworld run`, in memory and with `--save`; `AC-11` and `AC-12`

**Goal.** The command, and the evidence for CP-1 and CP-2 through the real binary.
**Scope.** `tools/cli/src/{main.rs, run.rs (new)}`, `tools/cli/Cargo.toml` (none expected), `tools/cli/tests/run.rs` (new). **Depends on:** C3.

- [x] Implementation (§9 E-5):
  - [x] `main.rs`: `run <world> --headless --seed N --days N [--save DIR]` as a clap subcommand (`--headless` required, `--days` ≥ 1); `run` removed from the "does not exist" list.
  - [x] `run.rs`: as planned. Bounded deviations: (1) an advance fault stops the run (nothing asked for it, so there is no request to refuse) while a dispatch fault is counted and survived; (2) the fingerprint of a `--save` run is read back from the whole save, so a resumed run's fingerprint covers the world's history, not one invocation; (3) the activity table prints per 30-day bucket so I-9 is checkable off stdout; (4) `last_facts(usize::MAX)` overflowed the backend's signed limit ("does not fit a stored integer", found by the first 300-day save) — "all" is now `i64::MAX`; (5) `tools/cli` gains `mineworld-kernel` as a direct dependency (`Dispatched`, `World`), within the composition root.
- [x] Validation (real binary, `CARGO_BIN_EXE_mineworld`) — `tools/cli/tests/run.rs`, 3 tests, shared helpers in `tools/cli/tests/headless/mod.rs`:
  - [x] **AC-11**: seed 7, 300 days, saved twice and in memory once, in parallel: I-9 holds in all ten buckets for all three seats; 300 day lines; `faults 0`; `person-entered-place` > 0; > 100 000 facts.
  - [x] **AC-12**: the two saves' facts, journal and snapshots equal **row by row as bytes**; manifests equal but for `instance` (asserted to differ); stdout equal but for header and `wall`. Seed 8 vs 7 (30 days, saved): genesis fact equal, fact logs differ. The in-memory run prints the same requests / facts / activity / history / consults / faults lines as the saved run. Q9 condition: no assertion compares fingerprints.
  - [x] `mineworld replay` verifies a 300-day save from genesis.
  - [x] Negative: missing `--headless`, `--seed`, `--days`; `--days 0`; `--seed one`; `--fast`; a missing pack — each refused naming the argument, no panic.
  - [x] Mutation: seeding the controller with `seed ^ process id` → the 300-day test fails at the byte comparison (`headless/mod.rs:160`, "facts differ first at row …"), reverted.
- [x] Review: the loop calls only `advance_to`, `observe`, `decide`, `ActionIntent::allocate`, `dispatch`; synchronous, no tokio; `Instant::now` used only for the `wall` line; seats in the pack's `BTreeSet` order; per-request tallies in `BTreeMap`s.

**Acceptance.** As validation; the 300-day test's wall time in the debug profile is recorded, and if a single run exceeds ~60 s the pace rises (not the days fall) — Q10.
**Failure cases.** A system fault during dispatch: counted, reported, the run continues (as the server does); a commit failure with `--save`: the run stops with the cause and a non-zero exit.

## C5 — A killed `run --save` finishes the same world (`AC-6` end to end, `F-13`)

**Goal.** CP-3 through real processes. **Scope.** `tools/cli/tests/run_restart.rs` (new); `run.rs` only if the test finds a defect. **Depends on:** C4.

- [x] Implementation: `tools/cli/tests/run_restart.rs` — control `--days 30 --save A`; kill points day 5 / 15 / 25 (`Child::kill`, SIGKILL, after reading the `day k` line); the same command re-run. `run.rs` unchanged by this commit. — §9 E-6.
- [x] Validation:
  - [x] each victim: signal 9, no `history` line, head on disk < control's; each survivor reports resuming at exactly that head; at least one re-executed a tail after its snapshot;
  - [x] each survivor's facts, journal and snapshots equal the control's row by row as bytes (Q9: fingerprints not compared — the plan's "and its fingerprint" is dropped as redundant with the byte equality and forbidden as evidence);
  - [x] **stop and continue**. Bounded deviation: seed 7 has *no* line straddling day 10 (located: one spoke fact within 20 minutes of it), so the test does what this item said to do instead of failing — it finds, in the control's log, the first day boundary with a straddling line (day 6, one line), stops a run there and continues it to day 30: byte-identical to the control. It then asserts that *no* line in the 30 days is quoted back more than once before its speaker says something new to the same listener, and that the straddling line is quoted back exactly once.
  - [x] Mutations, each reverted: resume point made inclusive after a request (HD-7) → both tests FAIL; the paced answer window widened to two paces (a controller that "forgets" it answered) → the stop-and-continue test FAILS with dozens of lines answered twice. A first version of the once-check counted replies within one pace only and did **not** fail under that mutation — an instrument that could not see a second answer (`ARC-23`); replaced by the quote-and-next-line check above.
- [x] Review: never killed / ignored the file / empty tail / two empty logs — each excluded as listed in the file's header.

**Acceptance.** All three kill points identical to the control. **Failure cases.** A kill landing inside a commit: the head is the previous revision (SQLite WAL; S5 L-2).

## C6 — `mineworld inspect` and the `AC-9` check

**Goal.** CP-4. **Scope.** `tools/cli/src/{main.rs, inspect.rs (new)}`, `tools/cli/tests/inspect.rs` (new), `tools/cli/tests/commands.rs` (re-point I-6). **Depends on:** C4.

- [x] Implementation: `tools/cli/src/inspect.rs` (HD-9 / HD-10) over `SqliteBackend` reads only (`manifest` + `check_format`, `head`, `journal_after`, `facts_of(GENESIS)`, `last_facts`), decoded with `persistence::format::decode`; `inspect <save> [--last N]` as a clap subcommand; a failed check exits non-zero naming up to ten facts. — §9 E-7.
- [x] Validation — `tools/cli/tests/inspect.rs`, 3 tests:
  - [x] on a 30-day `run --save`: the report's head equals the journal length on disk, its fact count equals the save's, systems with versions, all three cause kinds, 200 window lines, "every cause resolves: N fact(s) checked". Bounded deviation: the check covers the **whole** log, not a sampled window — reading it all is cheap, and a sample would be the weaker claim; the printed window is the newest N, not a mid-run slice.
  - [x] negative: a forged revision committed through `PersistenceBackend::commit` (a copy of the newest fact, renumbered, caused by action 999999) — the unforged save passes first, the forged one fails naming `fact #… caused by action 999999`.
  - [x] a directory with no save is refused by name. "Leaves the file's bytes unchanged" is checked as the logical tables (facts, journal, snapshots, manifest) before and after: the SQLite file's raw bytes may legitimately change when a WAL is checkpointed on close, which is not a write of world data.
  - [x] `commands.rs`'s missing-command test re-pointed from `inspect` to `create` (still missing until C7), claim and assertions unchanged (I-6).
- [x] Review: `inspect.rs` names event types, causes and journal kinds as stored; no domain concept; the output states that process and system-tick causes are counted, not resolved.

**Acceptance / failure cases.** As validation.

## C7 — `mineworld create`

**Goal.** CP-5's last command. **Scope.** `tools/cli/src/{main.rs, create.rs (new)}`, `tools/cli/templates/**` (new), `tools/cli/tests/create.rs` (new). **Depends on:** C4 (the created world is run).

- [x] Implementation: HD-11 — `tools/cli/src/create.rs`, template `tools/cli/templates/new-world/` embedded with `include_str!` (world.yaml, places/home.yaml, people/first.yaml, people/second.yaml); the id is checked with `EntityKey::new` before substitution. Bounded addition: with `create` and `inspect` both real, the "not yet" refusal now answers `install` and `add-system` — MODULE_SPEC §8's intended-but-unimplemented commands — and `commands.rs`'s missing-command test is re-pointed to `install employment`, claim and assertions unchanged (I-6). main.rs crate doc updated to the six commands. — §9 E-8.
- [x] Validation — `tools/cli/tests/create.rs`, 2 tests: create → validate (systems, seats first, second) → run 2 days with both seats moving and talking; an existing directory refused with its contents untouched and nothing written; `My Town!` refused by name and nothing created.
- [x] Review: the template's comments point at MODULE_SPEC §4.1 and the commands rather than restating rules; it uses only fields the loader implements (the read-back proves it).

## C8 — Documentation and ledger close

- [x] `tools/cli/README.md`, `cognition/rule-controller/README.md`, `systems/movement/README.md` (in C2), `docs/MVP_STATUS.md` (world boot, CLI, S7 and four evidence rows), `main.rs` crate doc (in C1b and C7). — `e5bba45`.
- [x] Full gates once on the final executable head (§6) — §9 E-final; §12 closeout; `handoff.md` closed.
- [x] Review: the READMEs describe and link; the rules they mention live in MODULE_SPEC §8.1, ARC-27 and ARC-26.

---

# 5. Integration checkpoint

```text
mineworld run worlds/social-cafe --headless --seed 7 --days 300           AC-11
    ├── no fault; every seat moved and talked in every 30-day bucket
    ├── PersonEnteredPlace occurred (the street is used)
    └── a fingerprint and a fact count printed
same command again                         → identical output but the wall line          AC-12
--seed 8                                   → a different history
--save A, killed at day k, same command    → facts/journal/snapshots == uninterrupted   AC-6
--days 10 then --days 30 on one save       → == uninterrupted; a straddling reply once   F-13
mineworld replay worlds/social-cafe --save A                → verifies from genesis
mineworld inspect A                        → every cause resolves; window printed        AC-9
mineworld create T/x && validate T/x && run T/x --headless …  → succeeds                 ARC-6
```

**Adversarial criteria (`ARC-23`):** two idle histories agreeing is excluded by locating activity per
seat and per bucket *before* comparing; a seed that is never read is excluded by the different-seed
run; a resume that read nothing is excluded by the reported tail; a fingerprint that hashes nothing is
excluded by printing the count of facts hashed and by comparing the logs themselves in the tests; a
bound is never derived from the quantity under test (`MAX_STRIDE` is the published constant, not a
value read back from the controller).

# 6. Test ownership and verification

```text
STATIC     fmt, clippy -D warnings (owns: no HashMap/HashSet, unused code), check
UNIT       rule-controller: window boundary, restart equivalence (fresh instance), seed sensitivity,
           stride bound; movement: disclosure presence/absence
REAL-LIFECYCLE (Gate 2 analogue)
           the real binary over the real pack: 300-day run (AC-11), repeat/seed runs (AC-12), SIGKILL
           and re-run (AC-6), inspect over a real save (AC-9), create→validate→run
REAL-LLM   NOT REQUIRED — no model anywhere (overall §2 item 4)
CI         no workflow exists (S13); the local gates below are terminal evidence, run once on the
           final head
```

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test -p mineworld-persistence --test kill_and_resume
python3 scripts/check_decision_ids.py
python3 scripts/check_doc_headings.py
```

Plus, recorded by hand once in §9: `cargo run --release -p mineworld-cli -- run worlds/social-cafe
--headless --seed 7 --days 300` twice, its output and wall time — the artefact a person runs.

# 7. Self-review against the frozen specifications

```text
CHECKED  INV-12 / I-1: no kernel, contracts, persistence or server change; "day" lives in the CLI
CHECKED  INV-13: the paced controller takes one argument and it is an Observation
CHECKED  INV-1: run's seats are driven over allocate→dispatch, the same authority path as a client;
         the driver is not a privileged mutator (it has no handle that writes)
CHECKED  INV-7 / ARC-26: no system's state is written outside its reducer; the disclosure in C2 is a
         read
CHECKED  ENGINEERING_RULES §8: the controller proposes strides and reads verdicts; TooFarAway is
         counted, never pre-empted by a duplicated rule
CHECKED  CLAUDE.md §4 rule 10: seeded, reproducible, headless, LM-free
CHECKED  CLAUDE.md §4 rule 11: no abstraction introduced for one use — no Controller trait, no
         driver framework; the mixer is private to its one consumer
CHECKED  CLAUDE.md §4 rule 16: no dependency added; the fingerprint and argument parsing are small
         enough that adopting a crate would be the decision made for its own sake (Q9, Q12)
CHECKED  the two gate questions (overall §2 item 8): no spatial or interaction contract changes; the
         doorway disclosure is usable by 2D and 3D clients identically
FLAGGED  C2 changes a System Pack's observable output (one extra component record) — Q4
FLAGGED  F-13 remains for --agent by decision — Q2
```

---

# 8. Source audit (`main @ ef53484`, 2026-10-06)

## 8.1 What was inspected

```text
tools/cli/src/{main,agent,perceive}.rs, tools/cli/Cargo.toml, tools/cli/tests/commands.rs
cognition/rule-controller/{Cargo.toml, src/lib.rs, src/tests.rs (test list)}
worldpack/src/{lib,load,read,catalog}.rs (public surface)
worlds/social-cafe/{world.yaml, places/*.yaml, people/alice.yaml, people/wanderer.yaml}
systems/presence/src/observe.rs; systems/movement/src/{system.rs (offers), component.rs};
systems/conversation/src/{component.rs, action.rs (INTERACTION_RANGE)}
server/src/runtime.rs (HostClock, ActionIds, submit, sweep), server/src/protocol.rs (WorldInstanceId)
persistence/src/{lib,world,backend,format,input}.rs, persistence/src/sqlite.rs (journal_after,
last_facts), persistence/tests/kill_and_resume.rs (header)
kernel/tests/long_run.rs (what S4 proved, and with what)
clippy.toml; contracts/src/event.rs (Causation); clients/protocol/mineworld/observation.gd:102–119
docs: MVP §§7, 9; MODULE_SPEC §8; VISION §4.3; ENGINEERING_STANDARDS §22; DECISIONS ARC-6, ARC-23,
ARC-25, ARC-26, DEP-10; overall §§2, 3, 7; step-04 §§1–7; step-06 §§9.1, 10, 12; step-07 §§1, 10, 11
searches: rand/seed/HashMap/SystemTime/Instant across *.rs; start_process/defer/wake across systems/;
"mineworld run|create|inspect" across docs and tools
```

## 8.2 Findings

```text
F-1  The real systems are time-inert and the only controller is reactive, so a headless café is idle
     (§2.2). Bounded: answered by HD-3 inside S7's own scope.
F-2  The server's agent path is wall-paced and drops observations under load (§2.3). Bounded: HD-1.
F-3  F-13 breaks CP-3 for any stateful controller (§2.5). Bounded for run: I-3; kept for --agent (Q2).
F-4  WorldInstanceId::allocate reads the wall clock; it reaches the manifest only (§2.6). Bounded:
     excluded from AC-12 by name, verified by comparing every other table.
F-5  Nothing in an observation says where a doorway is (movement offers `move` and discloses nothing),
     so no controller can use the street. Bounded if Q4 is accepted (C2).
F-6  tools/cli/tests/commands.rs's last test uses `inspect` as its example of a missing command; C6
     makes it exist. Re-pointed, same claim (I-6).
F-7  ARC-26 says MAX_STRIDE is a constant "until world configuration exists (S7)". S7 has no pack that
     needs another value. Q5.
F-8  main.rs's crate doc says clap is right "the day create and inspect arrive with real option
     surfaces". They arrive with two options between them. Q12.
F-9  `tools/world-validator` (overall §3 cross-cutting, "with S7") is `mineworld validate` in
     substance; no separate tool is proposed. Recorded so the cross-cutting line can be closed.
```

## 8.3 Material findings

None that changes a frozen invariant of an earlier step, a public contract or an ownership boundary.
C2 adds observable output to a System Pack (Q4) and is raised rather than assumed.

---

# 9. Ledger and evidence

```text
E-0  C0 design, 2026-10-06, on main @ ef53484 + this file. check_decision_ids: 35 ids, all distinct;
     check_doc_headings: 141 numbered sections across 22 documents, none duplicated. No cargo gate run
     (docs only). Test count 330 taken from overall §7, not re-counted in Phase 1.
E-1  C1 specs. MODULE_SPEC §8.1 (the implemented CLI), DECISIONS ARC-26 dated note, ARC-27, DEP-11.
     check_decision_ids: 37 ids, all distinct; check_doc_headings: 142 sections, none duplicated.
     `git grep "## ARC-27" / "## DEP-11"` over every origin branch except this one: no match.
     Review: §8.1 matches HD-5, HD-8, HD-9, HD-10, HD-11 and Q3; ARC-27 names its exclusions
     (instance id, wall time) and states the fingerprint is never evidence (Q9 condition).
E-2  C1b clap. clap 4.6.7 resolved (workspace pin 4.6.6, features derive). `cargo clippy -p
     mineworld-cli --all-targets -D warnings` clean. `cargo test -p mineworld-cli`: 17 passed, 0 failed
     (ac13 2, ac15 6, commands 4, restart 2, server_command 3), no test file changed. By hand:
     `--listen nope` → "invalid value 'nope' for '--listen'", exit 2; `--agent "Bad Seat"` → names
     the key rule, exit 2; `--help` lists server, validate, replay (the not-yet commands hidden).
E-3  C2 disclosure. `cargo test -p mineworld-movement`: disclosure 3/3, movement 7/7, persisted 1/1;
     mutation `if is_place { return Vec::new() }` → 0/3, reverted. Workspace `cargo test --no-fail-fast`
     after the change: no failure anywhere (counted at the final gates).
     Far side (Q4 condition), CLAIM: the Godot protocol module handles an observation carrying the
     passages record. OWNER: real lifecycle. COMMAND: `clients/protocol/run.sh evidence` (22.6 s wall;
     starts `mineworld server worlds/social-cafe --agent alice` per run; Godot 4.7.2 headless). The
     demo's report now lists each perceived entity's component types and reads `passages` through
     `MineWorldObservation.component` (clients/protocol/demo/demo.gd, reporting only).
     OBSERVED in clients/protocol/evidence/transcript-2d.log:17–19, transcript-3d.log:17–19,
     simultaneous-3d.log:18–20: `1  tags ["cafe","public"]` / `components ["passages"]` /
     `doorways 1 (read through MineWorldObservation.component)`; every transcript then completed its
     scripted run (accepted moves and talks, Alice's recall line, "the scripted run is over"), and
     request-2d.json / request-3d.json are byte-unchanged (git diff shows no change); `cargo test -p
     mineworld-cli --test ac13_semantic_parity` 2/2 over them. RESULT: PASS. Evidence files
     re-recorded by the run, never edited.
E-4  C3 PacedRuleController. `cargo test -p mineworld-rule-controller`: 13 passed (6 RuleController,
     7 paced). Mutation `opened <= at` (window boundary) → a_line_is_answered_in_one_window_only_and_
     never_again FAILED, reverted. `cargo clippy -p mineworld-rule-controller --all-targets -D warnings`
     clean. New dependency edges: rule-controller → movement (in-workspace), serde (workspace).
E-3a Coordinator sharpening of Q4 (2026-10-06, from the 3D environment slice, which walks café →
     street → café with `move` and had to be told place identities by a `--places=` launcher flag):
     C2 must disclose the *observer's own place's* passages. Already covered by C2 as built — the
     only place an observation lists is the observer's own, so that is the only place movement can
     disclose on. Pinned by `systems/movement/tests/disclosure.rs`
     `a_person_in_the_cafe_is_told_where_its_door_is_and_where_it_leads` (observer inside the café sees
     the café's passage to the street, both doorway positions) and, from the street,
     `the_street_is_not_described_…` (the street's own side). Far side: E-3's Godot transcripts show
     it in the observer's own place (`1 tags ["cafe","public"] … components ["passages"]`). No change
     to C2; the slice's `--places=` stopgap can go once this merges (its owners' call).
E-5  C4 run. By hand (debug build, this machine): seed 7, 300 days in memory 19.1 s wall — 129 600
     consults; move accepted 63 286, rejected TooFarAway 2 871; talk accepted 43 141; facts arrived
     63 290, conversation-started 28 560, person-entered-place 6 051, spoke 43 141, passage-opened 1;
     141 043 facts; faults 0. The same with --save: 37.9 s, 109 299 revisions; `mineworld replay`:
     109 299 revisions re-executed, 141 043 facts and 1 708 snapshots reproduced byte for byte.
     Q10: no single run exceeded 60 s, so the pace stays 600 s. TooFarAway refusals are proposals
     the server declined (a crossing proposed at an isqrt-rounded 2 000 mm), counted per HD-4.
     `cargo test -p mineworld-cli --test run`: 3 passed, 55.2 s (three 300-day runs in parallel
     dominate). clippy -D warnings clean. Mutation (process id in the seed) → FAILED at the byte
     comparison, reverted (44.6 s).
E-6  C5 restart. `cargo test -p mineworld-cli --test run_restart`: 2 passed, 15.1 s. Kill points 5,
     15, 25: all identical to the control byte for byte. Stop-and-continue located at day 6 (one
     straddling line), identical; no line answered twice in 30 days. Mutations: HD-7 inclusive
     resume → 2 FAILED (8.1 s); two-pace window → FAILED ("lines answered more than once, said at:
     [2400, 3002, 4202, …]"); the earlier one-pace-only once-check missed that mutation and was
     replaced. F-13 for `run`: closed by I-3 and shown across a real restart.
E-7  C6 inspect. `cargo test -p mineworld-cli --test inspect --test commands`: 3 + 4 passed (4.0 s).
     By hand on a 2-day save: head revision 747 at t172201 (day 2, 23:50:01); journal genesis 1,
     move accepted 425, move rejected TooFarAway 13, talk accepted 308; 991 facts; causes action 933,
     event 53, world genesis 5; "every cause resolves: 991 fact(s) checked". clippy clean.
E-8  C7 create. `cargo test -p mineworld-cli --test create --test commands`: 2 + 4 passed. clippy
     clean.
E-final  Gates on e5bba45 (the final executable head; later commits are planning documents only),
     clean tree, 2026-10-06:
       cargo fmt --all --check                                         PASS
       cargo check --workspace --all-targets                           PASS
       cargo clippy --workspace --all-targets --all-features -D warnings   PASS
       cargo test --workspace --no-fail-fast                           PASS — 350 tests ok, 0 failed
                                                                       (330 before + 20 new: disclosure
                                                                       3, paced 7, run 3, run_restart 2,
                                                                       inspect 3, create 2); 89 s wall
       cargo test -p mineworld-persistence --test kill_and_resume      PASS — cafe and clock, every
                                                                       kill point identical
       python3 scripts/check_decision_ids.py                           PASS — 37 ids, all distinct
       python3 scripts/check_doc_headings.py                           PASS — 142 sections
     Godot far-side evidence (E-3) was recorded on the C2 head; the protocol and server are unchanged
     since, so it stands. CI: N/A, no workflow in the repository (S13).
```

## 9.1 Limitations (expected)

```text
L-1  --agent still re-answers after a restart (F-13), by decision (Q2).
L-2  The paced controller's conversation is formulaic: replies quote, greetings come from a fixed set.
     That is the point of a rule (AC-11 is about stability, not charm); interpretation is S10.
L-3  Places have no extent, so a wandering Person is bounded only by the pull of the people near them;
     positions may drift. No rule exists to violate (walls are S6 L-2).
L-4  The AC-9 check resolves Action and Event causes from the log; Process and SystemTick causes are
     counted, not resolved (none occur in social-cafe).
L-5  The fingerprint is FNV-1a 64: detects accidental divergence, proves nothing against an adversary.
L-6  Protocol revision 1 names facts to a client only by id, so a client cannot see a fact's type: the
     3D slice inferred PersonEnteredPlace from a fact count. Recorded at the coordinator's request, not
     fixed here — it belongs to a later protocol revision (server/PROTOCOL.md), with step-06 L-7's removal of `deferrals_unscheduled`.
```

---

# 10. Questions for the primary session / operator

```text
Q1   Accept HD-3: initiative lives in a new PacedRuleController in cognition/rule-controller,
     RuleController untouched. Alternative: drive `run` from a scripted input file — reproducible but
     not a rule-based configuration, and it would make AC-11 a replay of a script. Recommended: HD-3.
Q2   F-13: handled for `run` by statelessness (pace window, §2.5), recorded and left for `--agent`
     (S10). Alternative: persist controller memory as world state — puts controller bookkeeping into
     the world, against INV-1's separation. Recommended as stated.
Q3   `--headless` is required by `run` (the documented shape in MODULE_SPEC §8 and ENGINEERING_STANDARDS
     §22; a future non-headless `run` stays possible). Alternative: accept it as an optional no-op.
     Recommended: required.
Q4   Movement discloses a place's Passages (C2), so headless people can use the door and S6's
     PersonEnteredPlace occurs in the AC-11 run. Cost: one extra component record in observations of a
     place; clients look components up by type. Alternative: leave it out; the run then never leaves
     the café. Recommended: include.
Q5   MAX_STRIDE stays a constant; world configuration waits for the first pack that needs another
     value (CLAUDE.md §4 rule 11), and ARC-26 gains a dated note saying so. Recommended.
Q6   `create` writes an embedded minimal template (HD-11). Alternative: copy an existing pack
     directory and rewrite its id — depends on the repository being present at run time, and needs a
     YAML-preserving edit. Recommended: embedded template.
Q7   `inspect` reads a save only, and its AC-9 check resolves Action and Event causes (HD-10, L-4).
     Recommended.
Q8   The CLI contract is recorded in MODULE_SPEC §8 (no new specification file); ARC-27 records the
     run's determinism. Recommended.
Q9   The fingerprint is a purpose-built FNV-1a 64 (~10 lines), not a hashing crate, and needs no DEP
     record because it is not infrastructure. Alternative: adopt `sha2` with a DEP record.
     Recommended: FNV.
Q10  Budget: the 300-day test runs in the debug profile through the binary; if one run exceeds ~60 s,
     the pace rises (fewer consults per day) rather than the days fall, because AC-11 names "hundreds
     of days". Recommended.
Q11  `--days D` is the world's age since genesis, so re-running a killed command completes it (HD-5).
     Alternative: D more days from wherever the save is. Recommended: age.
Q12  Argument parsing stays by hand (each command has at most four options); main.rs's clap note is
     updated to say why the trigger is not yet met. Alternative: adopt clap now with a DEP record.
     Recommended: by hand.
```

## 10.1 Answers — primary session review, 2026-10-06

Decided under the operator's autonomous authorization (overall §7) and their standing rule that
objective architectural correctness belongs to the agent. Ten answers are accepted as recommended,
two carry conditions, and **Q12 is overruled**.

**The audit finding is the most valuable thing in this design.** A headless social-cafe today does
nothing: no system acts on its own, and `RuleController` only ever answers. `run --days 300` would
therefore produce five genesis facts, perfectly reproducible and proving nothing. That is exactly
the `ARC-23` trap, a clean number from an instrument that cannot see. The activity precondition
(every seat moved and talked in every 30-day bucket, checked *before* determinism is compared) is
what makes `AC-11`/`AC-12` mean something, and it is frozen as an invariant.

**Q1 — ACCEPTED.** `PacedRuleController`, with `RuleController` untouched so `AC-15` is undisturbed.
It is a rule controller, not a model, so the "no LM before commit 8" rule (`CLAUDE.md` §5) holds.

**Q2 — ACCEPTED.** The stateless pace-window design is the right shape: `decide(&self)` makes "no
controller memory" something the compiler enforces rather than a convention. F-13 stays open for
`--agent` and belongs to S10.

**Q3 — ACCEPTED.**

**Q4 — ACCEPTED, with a far-side condition.** Movement discloses a place's passages. The claim that
"clients look components up by type, so an extra record is ignored" must be **verified from the far
side**, not asserted from Rust: risk R-9 records a client-side defect no Rust test could see. Show
the Godot protocol module (`clients/protocol/mineworld/`) handling an observation that carries the
passages record, through a real `run.sh` run, and record the evidence.

**Q5 — ACCEPTED.**

**Q6 — ACCEPTED.**

**Q7 — ACCEPTED.**

**Q8 — ACCEPTED.**

**Q9 — ACCEPTED, with a condition.** A purpose-built FNV-1a fingerprint is fine **for display only**.
Every equality claim in a test (facts, journal, snapshots) compares the bytes themselves, never
fingerprints. A fingerprint match must never be the evidence that two runs agree.

**Q10 — ACCEPTED.**

**Q11 — ACCEPTED.**

**Q12 — OVERRULED: adopt `clap` now, with a DEP record.** `tools/cli/src/main.rs` already records
the decision this question would reverse: *"`clap` is the right answer the day `create` and `inspect`
arrive with real option surfaces."* S7 is that day. It adds `create`, `inspect` and a `run` with
four options, on top of `server`, `validate` and `replay`. Rewriting the note to say the trigger is
not yet met would move the goalposts after reaching them, which is the self-persuasion `REUSE_POLICY`
and `CLAUDE.md` §4 rule 16 exist to stop. Argument parsing is commodity infrastructure.

Record **`DEP-11`** (the CLI's argument parsing: `clap` with derive), naming the option counts as
the trigger that was met. Migrate the existing subcommands in **their own commit, before** the new
ones, with every existing CLI test passing unchanged across it. Replace the main.rs note with a
pointer to `DEP-11`.

---

# 11. Execution contract (confirmed at freeze, 2026-10-06)

```text
PROJECT / PR        MVP-0 · Step 08 / PR 09 — Headless demo: World Pack loading, rule controller and
                    the CLI (S7)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-08-headless.md (this file)
RELATED / BINDING   overall.md §§2, 3 (S7), 7; MVP §§7, 9 (AC-6, AC-9, AC-11, AC-12); MODULE_SPEC §§5, 8;
                    ENGINEERING_STANDARDS §§21–24; DECISIONS ARC-6, ARC-23, ARC-25, ARC-26, DEP-6
                    (+ ARC-27 once C1 lands); step-04; step-06 §§9.1, 12; step-07 §10.1
IMPLEMENTATION BASE main @ ef53484; branch mvp0/pr-09-headless; worktree
                    /Users/yuema137/mineworld-worktrees/s7-headless (held by this session only)
APPROVED SCOPE      §1.1, as answered in §10
FROZEN INVARIANTS   §1.3 I-1 … I-8
SEQUENCE            C0 → C1 → C1b → C2 → C3 → C4 → C5 → C6 → C7 → C8, each committed and pushed when
                    coherent (C1b added from §10.1 Q12)
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED; long runs and
                    kill tests bounded to a few minutes of wall time in total (Q10)
LIVE DOCUMENTATION  this file (§4 checkboxes, §9 ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 09 at C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized after DESIGN FROZEN — source: the brief ("Phase 2
                                      (only after you are told the design is frozen)")
  semantic commits                    authorized — source: the brief ("Commit and push after every
                                      small step")
  branch push                         authorized — source: D-12; the brief
  PR creation / update                authorized — source: the brief ("Open a PR with gh pr create")
  CI repair                           N/A — no CI workflow in the repository (S13)
  merge                               explicit operator authorization only; the brief: "do not merge it"
POST-MERGE SYNC     the planning session owns step/overall updates; this session owns this document
NORMAL STOP         PR 09 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to an existing public contract beyond §1.1 as answered, to
                    ownership, or to scope — stop and report with evidence
```
