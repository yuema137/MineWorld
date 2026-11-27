# PR FE-a — Food-energy engine: kcal reserve in `needs`, valid-genesis-first saves, schema-aware views

**Lifecycle:** `DRAFT — FREEZE-READY` (2026-11-27). **Not DESIGN FROZEN; no implementation authorized.** Freeze
requires the operator's answers to Q-FE-1 … Q-FE-5 ([`step-fe-food-energy.md`](step-fe-food-energy.md) §1) and an
independent re-review of this text (§8).

**Parent step:** [`step-fe-food-energy.md`](step-fe-food-energy.md). **Execution contract:**
[`pr-fe-a-contract.md`](pr-fe-a-contract.md). **History/evidence:** [`fe-evidence/`](fe-evidence/) (the
recovered 2026-11-23 drafts and reviews; this document supersedes them where they differ).

**Design session:** lane L1, worktree `/Users/yuema137/mineworld-worktrees/design-nutrition`, branch
`docs/pr-design-nutrition`. **Audit base:** `origin/main @ 7718d952`.

---

## 0. Decisions this design depends on (plain language)

All **recommended, pending freeze**; full wording in the step document §1.

1. **Q-FE-1 — Reserve bar.** One kcal reserve per person, full at one configured day's use (2000 default), draining
   continuously; hunger word derived from it by the server; no medical consequences. USD 10,000 unchanged (it is
   FE-b content).
2. **Q-FE-2 — New saves only.** `needs` becomes version 2; old saves are refused with an explicit error and their
   committed history is logically untouched; no migration.
3. **Q-FE-3 — Minimal framework changes.** `KernelError::InvalidGenesisContent` (domain-neutral), save backend
   created only after valid genesis (both CLI callers), SDK per-owner schema version + `validate_authored` hook.
   Kernel stays domain-ignorant.
4. **Q-FE-4 — Engine first, content after.** This PR uses synthetic worlds only; no file under `worlds/` changes.
5. **Q-FE-5 — Execution endpoint.** Implement → READY FOR OPERATOR REVIEW autonomously; merge by the operator.

## 1. Observable outcome

A synthetic authored world chooses `hunger_model: unitless` (today's behaviour, default) or `food-energy` inside
the optional `needs` pack. In food-energy mode the server consumes the existing `ItemsConsumed` fact, applies
configured kcal/day use lazily in integer arithmetic, and discloses truthful kcal, capacity, daily use, serving and
an owner-computed bar to the holder only. Headless, agent, LM, 2D and 3D views render those same values. A
scripted market run buys, eats, saves, restarts and verifies the exact authoritative result with no model and no
renderer. Invalid authored content fails with an actionable diagnostic **before** any save file or listening socket
exists.

Not delivered here: Market Town content, retail-price claims, NPC affordability, `worlds/*` edits, migration,
hydration/illness, shop hours, sleep advance, controller policy changes, real-LLM claims.

## 2. Source audit (base `7718d952`)

- The recovered drafts audited 47 files at `b457a94d`. `git diff b457a94d 7718d952` over every mapped path is empty,
  and all 47 SHA-256 values in [`fe-evidence/20261123-source-manifest.json`](fe-evidence/20261123-source-manifest.json)
  verify at `7718d952` (`shasum -a 256 -c`: 47 OK, 0 failed). Raw searches:
  [`fe-evidence/20261123-source-searches.md`](fe-evidence/20261123-source-searches.md).
- Re-read in this session: `persistence/src/world.rs::PersistentWorld::create` (l. 128–171: snapshot → one
  `genesis` → manifest/row/snapshot encode → `backend.initialize`); `persistence/src/sqlite.rs::{create,exists}`
  (create makes the directory and schema immediately; `exists` checks only the file);
  `tools/cli/src/serve.rs::persisted` (l. 491–535) and `tools/cli/src/run.rs::Begun::new` (l. 318–350): both call
  `SqliteBackend::create` **before** `pack.assemble()`; the only production `PersistentWorld::create` callers
  (the third, `server/src/runtime/tests.rs`, is a test).
- `sdk/rust/src/interactions/lookup.rs` l. 38 and 55 hard-code `SectionConfigured<S>` event schema 1 and
  `Interactions<S>` component schema 1. `InteractionSection` (mod.rs l. 55) has no version constants and no owner
  validation hook; `HeldSection::{check,seed}` (mod.rs l. 164/168) route through `resolve` (resolve.rs l. 240).
  `Fields::set_names` (section.rs l. 60) and `Section::chain` (l. 413) are public.
- `authoring/src/configuration.rs::ConfigurationRefusal` has exactly `ClassUndefined` and `Ambiguous`.
- `kernel/src/error.rs::FactRefusedByOwner` (l. 343) is documented as a checked-constructor violation, not
  authored input. `contracts/src/entity.rs` l. 236 `key()`; `contracts/src/event.rs` `Causation::WorldGenesis`.
- `systems/needs`: `NeedsSystem::VERSION` 1 (system.rs l. 40); events schema 1 (event.rs l. 30, 60); components
  schema 1 (component.rs l. 25, 93); `Needs::mode()` reads `self.hunger.mode()` (component.rs l. 48);
  disclosed record (`disclose.rs`) carries `hunger`, `tiredness`, `words`, `asleep`, `rates{hunger_per_hour,
  hunger_asleep_per_hour, tiredness_per_hour, rest_per_hour}`, `restore`, `nourishment[]`, `kinds[]`.
  Parameters (interactions.rs l. 168–188): `hunger_per_hour`, `hunger_asleep_per_hour`, `tiredness_per_hour`,
  `rest_per_hour`, `thresholds`, `restore`.
- Clients: `clients/protocol/mineworld/observation.gd::component_value` (l. 137) returns the payload without its
  schema envelope; `needs.gd::own_needs` (l. 25). `tools/cli/tests/needs_rows_parity.rs` is `#[ignore]` (needs
  Godot 4.7).
- Rule controller: `fetch.rs::best` (l. 215), `beyond_chance_budget` (l. 405), `paced.rs::DECIDED_BY_NAME` (l. 121).

## 3. Contracts and invariants (frozen at freeze)

### 3.1 Model and arithmetic

- `needs` keeps **one** hunger state per person with a closed variant: `Unitless(Level)` (today) or
  `FoodEnergy { numerator, at }`. Tiredness `Level` is unchanged and is the **only** carrier of sleep mode;
  `Needs::mode()` reads tiredness. No second ticking store.
- Units: `D` = effective daily use (kcal per 86 400 world seconds); capacity `C = D`; numerator in kcal × 86 400,
  held in `u64`, all intermediate products and signed time differences in `i128` before subtraction or
  multiplication; clamp, then narrow. Reading at an instant earlier than `at` is an invariant failure, not a
  saturated duration.
  - `N(t) = max(0, N(at) − D·(t − at))`; consuming `count` of `K` kcal: `N' = min(C·86400, N(t) + K·count·86400)`.
  - `available_kcal = ⌊N/86400⌋`; `bar = ⌊1000·N/(C·86400)⌋` of maximum 1000;
    normalized `hunger = ⌊1000·(C·86400 − N)/(C·86400)⌋`; hunger words use the existing thresholds, *at or above*.
  - Item motivation projection for controllers: `ceil(1000·min(K, C)/C)`, 0 for `K = 0`. It ranks food; it never
    changes the reserve.
- Same rate asleep and awake; no midnight reset; no wall-time depletion; no Process, no runtime fact, no random
  draw. Two successful consumption facts at one instant both count. A zero-kcal consumption writes no energy fold.
- Unitless mode is behaviourally unchanged (same formulas, words and rates) though stored under schema 2.

### 3.2 Authoring

- `configure/needs.yaml`: base-only `hunger_model: unitless | food-energy` (default `unitless`); actor-scoped
  `energy_kcal_per_day` (1 … 100 000), legal only in food-energy and **required** as a base entry there;
  `hunger_per_hour` / `hunger_asleep_per_hour` legal only in unitless (rejected anywhere, even shadowed, in
  food-energy). Tiredness, thresholds and restore unchanged.
- Person forms (exclusive): unitless `{ hunger: 0..1000, tiredness }`; food-energy
  `{ food_energy: { initial_kcal: 0..effective D }, tiredness }`. In food-energy mode `needs: {}` is rejected
  (no silent reading of old initial hunger 300).
- Item forms (exclusive): `{ nourishment: { hunger } }` or
  `{ nourishment: { energy_kcal: 0..100000, serving: { label, quantity: { unit: g | ml, amount: 1..100000 } } } }`.
  `label` 1–128 UTF-8 bytes, no control characters, plain authored display text (as item names are today), never
  parsed. **One inventory unit = one whole serving.** No fractional inventory, unpacking or recipes.
- Errors and codes: configuration-only conflicts → `ConfigurationRefusal::Owner` with codes
  `needs-model-parameter-conflict`, `needs-energy-daily-use-required`, naming the real `EntryAt` and field.
  Cross-model or capacity contradictions found when reducing genesis → `InvalidGenesisContent` with codes
  `needs-model-mismatch`, `needs-initial-energy-out-of-range`, subject from `EntityRecord::key()`, field
  `food_energy.initial_kcal` / `nourishment.energy_kcal` / `hunger`. Decode errors keep line/column; genesis
  errors claim no line.

### 3.3 SDK seam

- `InteractionSection` gains `const CONFIGURED_SCHEMA_VERSION: EventSchemaVersion` and
  `const COMPONENT_SCHEMA_VERSION: ComponentSchemaVersion`, both defaulting to 1, used by `lookup.rs` in place of
  the two literals and by the declaration/reference generator. Every other pack inherits 1 unchanged.
- `InteractionSection::validate_authored(section, resolved) -> Result<(), ConfigurationRefusal>`, default no-op,
  invoked exactly once by `resolve` after the existing class/ambiguity resolution, so `check` and `seed` cannot
  disagree. It sees authored presence (`set_names`, reference-list `chain`, regions) before defaults erase it.
- `ConfigurationRefusal::Owner { entry: Option<EntryAt>, code: RejectionCode, detail: String }`, mapped
  exhaustively in `worldpack/src/configure.rs` with file path and entry index.

### 3.4 Version table

| Contract | Version |
| --- | --- |
| `NeedsSystem` system version | 2 |
| `Needs` component and the holder-disclosed `needs` record | 2 |
| `Nourishment` component | 2 |
| `NeedsAssigned`, `NourishmentAssigned` events | 2 |
| `needs-interactions-configured` event, `needs-interactions` component | 2 |
| Every other System Pack; economy/inventory/consumption facts; save container format | unchanged |

### 3.5 Kernel error (Q-FE-3a)

```text
KernelError::InvalidGenesisContent { system: SystemId, subject: EntityKey, code: RejectionCode,
                                     field: String, detail: String }
```

Returned only by an owner reducing a fact whose `caused_by()` is `Causation::WorldGenesis`, validated before it
inserts the component. Construction stops; the whole candidate world is discarded (dispatch stays
non-transactional; no rollback promise). During replay the same failure means the history cannot be rebuilt and
resume fails closed. A non-genesis contradiction keeps `FactRefusedByOwner`. Never used for ordinary action
rejection. Wrappers (`PackError::Composition`, `PersistError::Kernel`, CLI `described` / `HostError::build`) keep
owner, code, subject and field visible. No food, currency or needs vocabulary in kernel.

### 3.6 Valid-genesis-first save creation (Q-FE-3b)

```text
PersistentWorld::create_with_backend<F>(world: World, creation: Creation, make_backend: F)
    -> Result<(PersistentWorld, Vec<EventEnvelope>), PersistError>
where F: FnOnce() -> Result<Box<dyn PersistenceBackend>, PersistError>
```

`make_backend` runs exactly once, only after snapshot, the single in-memory genesis, manifest encoding and
genesis-row/snapshot encoding have all succeeded; then `initialize` runs once with the precomputed row. Existing
`create(backend, …)` delegates through the common body and keeps its documented limitation (its backend already
exists). `serve.rs::persisted` and `run.rs::Begun::new` new-save branches assemble first and pass
`|| SqliteBackend::create(save, …)`; no directory, no SQLite, no "created" message before success; the server binds
only after the host is built (unchanged). Storage/I/O failures after valid genesis return an error and no world;
artifacts in that case are not promised absent. `SaveExists` refusal and the pre-existing interprocess race are
unchanged. Resume branches keep their order (open → configuration drift check → compose → resume).

### 3.7 Old saves (Q-FE-2)

The first applicable explicit incompatibility is reported (configuration drift may precede
`PersistedSystemOutdated`). On refusal: no initialize, no appended revision, no re-genesis, no new facts, no
deletion or truncation by application code, and the manifest, head, committed revisions/facts/snapshots/blocks and
host metadata are **logically** unchanged. SQLite's own WAL/checkpoint representation may change; physical bytes
are not promised. Tests use scratch copies of complete `world.sqlite` + WAL + SHM families; user saves are never
touched.

### 3.8 Disclosure wire and consumers

- The v2 `needs` record adds `hunger_model: "unitless" | "food-energy"`. `rates` keeps the tiredness fields; its
  hunger rate fields appear only in unitless. Food-energy adds
  `food_energy: { unit: "kcal", available_kcal, capacity_kcal, use_kcal_per_world_day,
  bar: { value, maximum }, replaces_need: "<need identifier>" }`. `nourishment[]` / `kinds[]` entries keep the
  normalized `hunger` motivation score and add `energy_kcal` and `serving` in food-energy mode.
- `replaces_need` is owner-disclosed identity so a generic reader can swap the deficit row for the reserve row
  without naming "hunger" (keeps `scripts/check_client_rules.py` R7 intact and unmodified).
- Readers inspect the component envelope's schema before decoding. Schema 1 recordings use the existing legacy
  branch; schema 2 accepts exactly the two variants; unknown schema, malformed or incomplete food-energy records
  render no energy row and **never** fall back to a deficit bar (valid tiredness may still render). Integers are
  checked finite, whole, non-negative, within bounds by equality/type checks only (no `<`/`>` in `needs.gd`).
- No client, agent or LM view computes kcal, thresholds or rates. LM gets one bounded owner-facts line; the word-based
  feeling trigger is unchanged (numeric reserve changes alone never trigger inference). No-needs prompt golden
  unchanged.

## 4. Caller and consumer map

| Boundary | Path / symbol | Disposition |
| --- | --- | --- |
| SDK version/validation | `sdk/rust/src/interactions/{mod,lookup,resolve}.rs` | §3.3; unrelated owners byte-identical declarations. |
| Config refusal | `authoring/src/configuration.rs`; `worldpack/src/{configure,error}.rs` | `Owner` variant, exhaustive mapping. |
| Kernel error | `kernel/src/error.rs` | New variant + docs only; `dispatch.rs` behaviour unchanged; `contracts` unchanged. |
| Wrappers | `worldpack/src/error.rs::PackError::Composition`, `persistence/src/error.rs::PersistError::Kernel`, CLI formatters | Pin identifying fields in output. |
| Save creation | `persistence/src/{world,backend,lib}.rs` | §3.6. |
| CLI callers | `tools/cli/src/serve.rs::persisted`, `tools/cli/src/run.rs::Begun::new` | New-save branch uses the factory. `main.rs::validate` already uses `pack.load` without persistence: unchanged. |
| Old `create` callers | `persistence/tests/{save,retention,host_journal,kill_and_resume}.rs`, `server/src/runtime/tests.rs`, owning-system persisted tests | API kept; no bulk migration. |
| Needs owner | `systems/needs/src/{component,event,section,interactions,level,system,disclose,codec,lib}.rs` + new `food_energy.rs` | §3.1–3.4, 3.8. |
| Needs tests | `systems/needs/tests/{configuration,lazy,perception,reactions,removable,section}.rs` | Typed API update; unitless variant asserted explicitly. |
| Rule controller | `cognition/rule-controller/src/needs.rs` (+ `fetch.rs`, `paced.rs`, `offered.rs` read-only) | Schema-aware decode; same normalized values and ranks; no policy change. |
| Motivated | `cognition/motivated/src/considerations.rs` | Consumes the same projection; no change expected beyond decode. |
| Shared Godot | `clients/protocol/mineworld/{needs,observation}.gd`; `clients/protocol/checks/needs_check.gd` | Envelope-aware decode; replacement row. |
| 2D | `clients/2d/scripts/hud/{readers,panels}.gd`; `clients/2d/scripts/harness/needs_rows.gd` | Reserve row with owner bar/max; unitless unchanged. |
| 3D | `clients/3d-spike/scripts/slice/{needs_hud,needs_rows_probe}.gd` | Same row semantics. |
| Wording | `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po` | New keys for food energy, rate, serving (existing gettext convention, audited at edit). |
| Agent | `clients/agent/src/mineworld_agent/{models,disclosed,view}.py`; `clients/agent/tests/test_needs_view.py` | Typed `FoodEnergy`/`Serving` records, schema-preserving selector. |
| LM | `cognition/lm-controller/src/mineworld_cognition/controller/{situation,prompt}.py`; tests `test_needs_prompt.py`, `test_needs_triggers.py`, `test_controller_choice.py` | Optional typed summary, one bounded line. |
| Python wire | `sdk/python/src/mineworld_sdk/wire/contract.py` | Generic envelope already carries schema; verify, no change expected. |
| Fixtures | `tests/fixtures/observations/needs/*` | Keep schema-1 recordings; add synthetic food-energy fixture + hand-written en/zh_Hans rows. |
| Reference | `docs/reference/schema/{configure-needs,section-needs}.schema.json`; CLI `reference` | Regenerate via the existing pipeline; no hand edits. |
| Specs | `docs/MODULE_SPEC.md` needs sections; `docs/decisions/ARC-284.md`, `ARC-285.md` | Dated notes (C0). |
| Registration | `systems/installed` | No new pack or registration. |

## 5. Acceptance (engine subset E1–E7 of NE-1 … NE-10)

All **NOT RUN**.

- **E1 (NE-1).** Each authoring error in §3.2 is refused with the stated code and location. Real `server --save` and
  `run --save` with an invalid model and an out-of-range initial reserve exit non-zero, show owner/code/subject/field,
  print no "created", leave an absent save path absent (no `world.sqlite`/WAL/SHM), and bind no port.
- **E2 (NE-2, NE-6).** Literal oracle; one-second remainder; split = unsplit; cap, zero, zero-kcal no-fold;
  same-instant double meal; `i128` max-count and signed-endpoint cases; equal reserve under 1/2/4-hour presets;
  no Process/fact/draw.
- **E3 (NE-3, NE-4).** Two synthetic worlds (currency exponent 2 and 0, D 2000 and 2400, actor-class override):
  buy → hold → eat/drink through real `economy`/`inventory`/`consumption`/`needs`; rejection atomicity; zero-kcal
  drink.
- **E4 (NE-5).** Real file-backed close/reopen and kill/resume equal uninterrupted state; refusal of an old schema-1
  save and of configuration drift preserves logical contents for a closed and a crash-tail WAL family.
- **E5 (NE-8).** Unrelated packs keep schema 1 and identical declarations; unitless world unchanged; absent `needs`
  inert; Social Cafe/Bodies Yard baseline facts unchanged; malformed/unknown v2 never downgraded.
- **E6 (NE-7).** Same values in shared decoder, agent and LM views; actual 2D and 3D screenshots/logs in en and
  zh_Hans showing the reserve refilling and tiredness intact; R7 guard unchanged; no extra LM trigger.
- **E7 (NE-10).** Independent review per §8.

## 6. Semantic commit plan

Each commit carries separate `[ ]` Implementation, `[ ]` Validation and `[ ]` Independent LLM review items, each
ticked only with recorded evidence. Before each commit the executor records diff summary, staged paths, tested head,
commands, counts, wall time, outcomes, deviations. Heavy cargo jobs run through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one test file at a time (R-LOAD-1); never a local
full-workspace run.

### C0 — Contracts first

1. **Goal:** make the frozen contracts authoritative in the specs before code.
2. **Scope:** `docs/MODULE_SPEC.md` needs/configuration/disclosure/persistence/error sections; dated notes in
   `docs/decisions/ARC-284.md` and `ARC-285.md` (amend S35 "kernel unchanged" for one domain-neutral error); a new
   decision id only if `scripts/check_decision_ids.py` policy requires one. No code.
3. **Implementation:** [ ] record freeze reference and base; [ ] write §3 contracts into owning specs;
   [ ] ARC notes; [ ] link this PR from the step ledger.
4. **Validation:** [ ] `python3 scripts/check_doc_headings.py`; [ ] `python3 scripts/check_decision_ids.py`;
   [ ] reviewer compares spec text with §3 and the step §1 answers.
5. **Acceptance:** no unresolved contract ambiguity; the 2000 kcal and USD 10,000 requirements are stated as
   retained, the latter as FE-b content.
6. **Failure/edges:** a source change since `7718d952` touching a mapped seam → bounded re-audit before C1; any
   contract change beyond §3 → material stop.
7. **Commands/evidence:** the two scripts above; record outputs.
8. **Boundary:** docs-only semantic commit. [ ] Implementation [ ] Validation [ ] Independent review

### C1 — SDK owner versions/validation hook and the generic genesis error

1. **Goal:** add exactly the generic seams needs requires, with every other pack unchanged.
2. **Scope:** `sdk/rust/src/interactions/{mod,lookup,resolve}.rs` + `tests.rs`; `authoring/src/configuration.rs`;
   `worldpack/src/{configure,error}.rs` + configuration refusal tests; `kernel/src/error.rs`. No dispatch behaviour,
   no domain type, no Person field. Depends on C0.
3. **Implementation:** [ ] two defaulted version constants used in `lookup.rs` and the declaration/reference path;
   [ ] `validate_authored` default no-op called once in `resolve`; [ ] `ConfigurationRefusal::Owner` + exhaustive
   mapping; [ ] `KernelError::InvalidGenesisContent`; [ ] wrapper display keeps identifying fields.
4. **Validation:** [ ] stub owner at schema 2 shows declarations/configured fact/component at 2 while a real
   unchanged owner stays 1; [ ] stub hook refusal propagates through the real loader with correct `EntryAt` for
   base, region, default-list and inherited sources; existing class/ambiguity errors still win first;
   [ ] full wrapped error text contains owner/code/subject/field (no unstable prefix assertion).
5. **Acceptance:** E1 configuration path and E5 version defaults; no existing pack's declaration or payload changes.
6. **Failure/edges:** hook runs exactly once per resolve; missing configuration uses compiled unitless defaults;
   unexpected change in an unrelated pack is a failure, never a baseline update.
7. **Commands/evidence:** via `with-slot.sh`: `cargo test -p mineworld-sdk --lib interactions`,
   `cargo test -p mineworld-worldpack --test configuration` and `--test interaction_sections`,
   `cargo check -p mineworld-kernel -p mineworld-sdk -p mineworld-worldpack`, clippy on touched crates, `cargo fmt`.
8. **Boundary:** inert seams + discriminating loader/version/error evidence. [ ] Implementation [ ] Validation
   [ ] Independent review

### C2 — Valid-genesis-first save creation in persistence and both CLI callers

1. **Goal:** no save artifact for a world that fails to assemble or begin; no double genesis, fake preflight or
   deletion rollback.
2. **Scope:** `persistence/src/{world,backend,lib}.rs` + `persistence/tests/save.rs`; new-save branches of
   `tools/cli/src/serve.rs` and `tools/cli/src/run.rs`; one new CLI test target for the creation lifecycle (proposed
   name `save_creation`, not an existing target). SQLite format unchanged. Depends on C1 only for the error type.
3. **Implementation:** [ ] common internal body; [ ] `create_with_backend`; [ ] `create` delegates; [ ] both callers
   assemble first and pass the factory closure; [ ] success message and providers only after initialize.
4. **Validation:** [ ] stub-owner genesis failure and encode failure → factory called 0 times; success → exactly 1 and
   replay shows one genesis; [ ] **real CLI** `run --save` and `server --save` on a synthetic pack whose assemble
   fails (undecodable section) → non-zero exit, absent directory stays absent, no listener (today this leaves an
   empty `world.sqlite` and the next launch fails on resume); [ ] real successful creation + reopen;
   [ ] existing-save branch never reset or re-genesised; [ ] old `create` API save/replay unchanged.
5. **Acceptance:** E1 no-artifact property for assemble failures and stubbed genesis failures; E4 creation/replay
   invariant. The real-CLI *invalid genesis* proof is owned by C3 (review R3) and is listed there; C2's ledger marks
   it "deferred to C3", not passed.
6. **Failure/edges:** absent vs caller-created empty directory; existing save → `SaveExists`; factory or initialize
   I/O failure → error, no world, artifacts not promised absent; interprocess race out of scope.
7. **Commands/evidence:** via `with-slot.sh`: `cargo test -p mineworld-persistence --test save`,
   `cargo test -p mineworld-cli --test save_creation`, one existing run/server lifecycle regression target chosen at
   edit; check/clippy touched crates; fmt. Processes bounded, loopback only; record stderr, exit code, a complete
   listing of the save path and the port check.
8. **Boundary:** generic lifecycle fix, no domain rule. [ ] Implementation [ ] Validation [ ] Independent review

### C3 — `needs` v2: food-energy model, forms, reactions, disclosure; synthetic persisted commerce

1. **Goal:** make §3.1–3.4 and §3.8 (server side) real in the one owning System Pack.
2. **Scope:** `systems/needs/src/*` + new `food_energy.rs`; `systems/needs/tests/*`; synthetic test packs under the
   owning test fixtures (never `worlds/*`); CLI tests `needs_sections` plus a synthetic nutrition integration target
   (proposed `food_energy`); `cognition/rule-controller/src/needs.rs` and `cognition/motivated` decode only if the
   v2 record requires it; regenerated `docs/reference/schema/*needs*.schema.json`. Depends on C1, C2.
3. **Implementation:** [ ] typed variants, named units, schema 2 table; [ ] `validate_authored` for needs with the
   four codes; [ ] exclusive person/item forms and serving bounds; [ ] genesis validation before insert using
   `EntityRecord::key()` and `caused_by()`; [ ] `i128` arithmetic, zero-kcal no-fold, sleep mode from tiredness;
   [ ] normalized projection, owner bar, `replaces_need`, holder-only; [ ] still no Process/runtime fact;
   [ ] regenerate reference.
4. **Validation:** [ ] E2 oracles; [ ] E1 real `run --save`/`server --save` invalid model and invalid capacity: no
   save, no listener, visible diagnostic; [ ] E3 two-currency purchase/consume/rejection/zero drink/actor class 2400;
   [ ] E4 real close/reopen, kill/resume, and old-schema/drift refusal on scratch closed + WAL families with whole
   logical row comparison, then matching-version replay of the copy; [ ] E5 unitless, removal and witness baselines;
   [ ] rule/motivated projections give unchanged ranks and absent needs stays inert.
5. **Acceptance:** formulas, refusals, atomicity, restart and privacy hold through real owners; unrelated digests
   unchanged; no synthetic price is described as retail.
6. **Failure/edges:** overflow widened first; backward time is an invariant failure; corrupt schema is a fault, not
   authoring input; a fixture never writes an operator save or deletes WAL.
7. **Commands/evidence:** via `with-slot.sh`, one file at a time: `cargo test -p mineworld-needs --test <file>` for
   each touched file, `cargo test -p mineworld-cli --test needs_sections`, `--test food_energy`, `--test reference`,
   rule/motivated needs test files; clippy/fmt. No 300-day or long run.
8. **Boundary:** one owning System Pack + its synthetic integration evidence; no content, price or controller policy.
   [ ] Implementation [ ] Validation [ ] Independent review

### C4 — Schema-aware shared views and actual both-client proof

1. **Goal:** every view shows the owner's kcal truthfully and rejects malformed data without inventing a bar.
2. **Scope:** paths in §4 rows Shared Godot, 2D, 3D, Wording, Agent, LM, Fixtures. No transport change, no generic
   SDK needs model, no inference policy change. Depends on C3.
3. **Implementation:** [ ] envelope-aware selection; [ ] strict v2 + explicit v1 branch; [ ] replacement row via
   `replaces_need`; [ ] en/zh_Hans kcal/rate/serving strings; [ ] agent typed records; [ ] LM optional summary + one
   bounded line; [ ] hand-built tests lacking schema state schema 1 explicitly; recorded fixture bytes untouched.
4. **Validation:** [ ] E6 actual 2D and 3D scenes (initial, after caloric food, after zero-kcal drink, after restart)
   in en and zh_Hans; [ ] row probes equal hand-written literals; [ ] malformed/non-integer/negative/bool/unknown/
   missing v2 → no energy row, no fallback; [ ] schema-1 rows and no-needs prompt golden unchanged; [ ] reserve
   change alone triggers no LM consult; [ ] `python3 scripts/check_client_rules.py` passes unmodified.
5. **Acceptance:** both clients, agent and LM views show the same disclosed kcal/day/serving; normalized hunger never
   printed as kcal; a missing Godot makes the client claim INCONCLUSIVE, never passed.
6. **Failure/edges:** Godot JSON floats must be integral where integers are expected; booleans rejected; overlong
   serving/prompt line is omitted whole, never truncated to a misleading value.
7. **Commands/evidence:** `python3 scripts/check_client_rules.py`;
   `cargo test -p mineworld-cli --test needs_rows_parity -- --ignored --test-threads=1` (via `with-slot.sh`);
   `godot --headless --path clients/protocol --script res://checks/needs_check.gd`; bounded windowed 2D/3D launch
   with the synthetic world and screenshots; **separate** pytest invocations from the repository root (review R1):
   `uv run --locked pytest clients/agent/tests/test_needs_view.py` and
   `uv run --locked pytest cognition/lm-controller/tests/test_needs_prompt.py cognition/lm-controller/tests/test_needs_triggers.py`;
   ruff/pyright on touched Python per `.structured-coding/standards.md`.
8. **Boundary:** fact rendering + typed adapters + actual client evidence; no UI redesign or model claim.
   [ ] Implementation [ ] Validation [ ] Independent review

## 7. Terminal evidence

READY FOR OPERATOR REVIEW requires: C0–C4 committed with ledger evidence; E1–E7 recorded; actual both-client E6 and
lifecycle E4 complete; independent review complete with findings disposed; PR open; required CI green on the exact
final head, recorded separately from local results. Review starts while CI runs. No redundant local full suite.

## 8. Review requirements

- **Before freeze:** an independent reviewer (not this lane) reads this document, the contract and the step §5–§7 and
  checks them against `7718d952` source. Record the verdict and SHA of the reviewed text here.
- **Each commit and at readiness:** ownership, debt, function/file growth (50/100 lines, 500/800 lines as triggers;
  `sdk/rust/src/interactions/section.rs` is already 1013 lines — add nothing there unless reviewed), coupling,
  lifecycle failure paths, error context/precedence, typed malformed-record handling, no client rules, no new
  dependency, no world switch.

## 9. Evidence ledger

- 2026-11-27, lane L1: design only. Read-only audit at `7718d952`; 47-file hash verification PASS; no code, build, test,
  Godot, model or heavy job run. All C0–C4 items `[ ]`.
