# PR FE-a — Food-energy engine: kcal reserve in `needs`, valid-genesis-first saves, schema-aware views

## DESIGN FROZEN — 2026-11-27 (operator approved in the primary session: every question answered with its recommendation)

```text
Design revision:        revision 2 (2026-11-27), lane L1, after the red-team pre-freeze review
Approved by / evidence: the operator, 2026-11-27, relayed by the primary coordinator:
                        "同意三个都按推荐冻结" (Q-FE-1 … Q-FE-5, PRs #535/#536/#537 as recommended) and
                        "营养设计那两点、S34-c3 也全都同意你的推荐" (C-FE-2 confirmed; Q-FE-6 defer per-person)
Implementation base:    origin/main @ 24dbc3bf (no mapped source path changed since the 8c971f0f re-audit);
                        C0 records the exact base at dispatch
Execution contract:     .structured-coding/plans/mvp0/pr-fe-a-contract.md
Lifecycle:              FROZEN
```

Frozen: scope, invariants (§3), acceptance (§5) and the commit plan (§6). Progress, evidence, audit findings and
bounded corrections stay writable.

**Parent step:** [`step-fe-food-energy.md`](step-fe-food-energy.md). **Execution contract:**
[`pr-fe-a-contract.md`](pr-fe-a-contract.md). **History/evidence:** [`fe-evidence/`](fe-evidence/) (the
recovered 2026-11-23 drafts and reviews; this document supersedes them where they differ).

**Design session:** lane L1, worktree `/Users/yuema137/mineworld-worktrees/design-nutrition`, branch
`docs/pr-design-nutrition`. **Audit base:** `origin/main @ 8c971f0f` (re-audited 2026-11-27 after the red-team
pre-freeze review; first audit at `7718d952`).

**Design revision:** revision 2 (2026-11-27) — red-team pre-freeze review
(`/Users/yuema137/mineworld-worktrees/_reviews/design-fe.md`, verdict FREEZE-AFTER-FIXES on head `6105eea7`) findings
2, 5–12 applied; findings 3 and 4 returned to the operator (step §1, Q-FE-2 consequence line and Q-FE-6).

---

## 0. Decisions this design depends on (plain language)

All **ruled by the operator, 2026-11-27**: Q-FE-1 … Q-FE-5 ("同意三个都按推荐冻结"); C-FE-2 confirmed and Q-FE-6
deferred ("营养设计那两点、S34-c3 也全都同意你的推荐").
Full wording in the step document §1.

1. **Q-FE-1 — Reserve bar.** One kcal reserve per person, full at one configured day's use (2000 default), draining
   continuously; hunger word derived from it by the server; no medical consequences. USD 10,000 unchanged (it is
   FE-b content).
2. **Q-FE-2 — New saves only.** `needs` becomes version 2; old saves are refused with an explicit error and their
   committed history is logically untouched; no migration. Consequence: **every** existing save of a world that enables
   `needs` (Market Town today, even in unitless mode) stops loading in the new build; the old build still plays it.
6. **Q-FE-6 — Per-person daily use.** Daily use is per world and per actor class; per-person values are not in
   this step.
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

## 2. Source audit (base `8c971f0f`)

- The recovered drafts audited 47 files at `b457a94d`. `git diff b457a94d 7718d952` over every mapped path is empty,
  and all 47 SHA-256 values in [`fe-evidence/20261123-source-manifest.json`](fe-evidence/20261123-source-manifest.json)
  verify at `7718d952` (`shasum -a 256 -c`: 47 OK, 0 failed). Raw searches:
  [`fe-evidence/20261123-source-searches.md`](fe-evidence/20261123-source-searches.md).
- Re-audit at `8c971f0f`: 45/47 unchanged; two changed by S31-b2 (#533). `authoring/src/configuration.rs` gains
  `YieldedFile` and a provided `yields()` on the configuration traits — orthogonal to `validate_authored` and to the
  new `ConfigurationRefusal::Owner`. `worldpack/src/configure.rs::compare` now compares only facts whose
  `caused_by()` is `Causation::WorldGenesis` (ARC-61 note 2026-11-27). `needs-interactions-configured` and
  `NeedsAssigned`/`NourishmentAssigned` are direct genesis facts, so §3.3, §3.5 and §3.7 are unchanged; §3.7's
  precedence statement now relies on that filter.
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
- **Reference values.** The 2000 kcal/day default is the FDA Daily Value reference intake used on US Nutrition
  Facts labels (21 CFR 101.9(c)(9); FDA "Daily Value on the Nutrition and Supplement Facts Labels"); it is the
  operator's game default, not personal advice. The constant drain is a deliberate simplification of expenditure
  (operator-playable-mvp §5: "fixes the example default, not a physiological model"): real sleeping expenditure is
  near basal rate, roughly 60–70 kcal/h for a typical adult against the 83 kcal/h daily average, so night drain is
  modestly overstated. This differs from S35's unitless model, which slows hunger in sleep (`hunger_asleep_per_hour`
  30 vs 120); food-energy worlds therefore wake with a lower reserve than a unitless world wakes hungry. An optional
  asleep fraction is a deferred extension, not in FE-a.
- Unitless mode is behaviourally unchanged (same formulas, words and rates) though stored under schema 2. Keeping
  unitless at schema 1 and only food-energy at 2 was considered and rejected: the persisted version is per System
  (`NeedsSystem::VERSION`) and per component/event type, not per variant, so one pack cannot persist two versions of
  the same `Needs` component; a split would need a second component type, i.e. a second hunger store (§3.1 first
  bullet forbids it).

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
- `restore: { hunger: [] }` in food-energy mode is **allowed**: an explicit no-restoration world in which the reserve
  only drains (same as unitless today; step-fe NE-8 removal cases). No refusal code.
- Daily use is per world (base entry) and per actor class (actor-scoped entry). Per-person values are out of scope
  (Q-FE-6).

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
rejection. Nothing in dispatch enforces "genesis only": if an owner returns `InvalidGenesisContent` while reducing a
non-genesis fact, dispatch treats it exactly like any other reduction `KernelError` (the existing runtime fault
path); C3 adds a test that the needs owner never does so and that such an error is not reported as authored input.
`field` and `detail` are bounded diagnostic text for a human, never matched by code or used as a selector; the
typed identity is `system`, `subject` and `code`. Wrappers (`PackError::Composition`, `PersistError::Kernel`, CLI
`run.rs::described`, and `server/src/host.rs::HostError::build` — server crate, l. 509) keep owner, code, subject and
field visible; C1 audits each `Display` and edits `server/src/host.rs` only if it drops nested text. No food,
currency or needs vocabulary in kernel.

### 3.6 Valid-genesis-first save creation (Q-FE-3b)

```text
PersistentWorld::create_with_backend<F>(world: World, creation: Creation, make_backend: F)
    -> Result<(PersistentWorld, Vec<EventEnvelope>), PersistError>
where F: FnOnce() -> Result<Box<dyn PersistenceBackend>, PersistError>
```

`make_backend` runs exactly once, only after snapshot, the single in-memory genesis, manifest encoding and
genesis-row/snapshot encoding have all succeeded; then `initialize` runs once with the precomputed row. Existing
`create(backend, …)` delegates through the common body and keeps its documented limitation (its backend already
exists). It is kept as a legitimate API, not a compatibility adapter (CLAUDE §4 rule 12): ~25 test and owning-system
callers construct a scratch or in-memory backend themselves and have no save path to protect; forcing a closure on
them adds nothing. `serve.rs::persisted` and `run.rs::Begun::new` new-save branches assemble first and pass
`|| SqliteBackend::create(save, …)`; no directory, no SQLite, no "created" message before success; the server binds
only after the host is built (unchanged). Storage/I/O failures after valid genesis return an error and no world;
artifacts in that case are not promised absent. `SaveExists` refusal and the pre-existing interprocess race are
unchanged. Resume branches keep their order (open → configuration drift check → compose → resume).

### 3.7 Old saves (Q-FE-2)

Scope: **every** save of a world enabling `needs` made before FE-a (including unitless Market Town) is refused by
the new build; the old build keeps playing it. The first applicable explicit incompatibility is reported. For such a
save the expected first error is the existing configuration-drift refusal on the direct-genesis
`needs-interactions-configured` fact (its record changes shape), before `PersistedSystemOutdated`
(`persistence/src/replay.rs`); C3's E4 test pins which error appears and that its text names `needs`. The CHANGELOG
entry (C3) tells players to keep the previous build for old needs saves. On refusal: no initialize, no appended revision, no re-genesis, no new facts, no
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
- **Public agent/MCP tool schema** (outside agents are first-class players, S27). `NeedLine` is unchanged (its
  `level` stays the normalized 0…1000 value). `NeedsLine` gains one optional field
  `food_energy: FoodEnergyLine | None = None`, present only in food-energy mode, with
  `FoodEnergyLine { available_kcal: int ≥ 0, capacity_kcal: int 1…100000, use_kcal_per_world_day: int 1…100000 }`
  (no bar — agents read numbers; no `replaces_need` — the adapter already names needs). The agent adapter exposes no
  per-item kcal/serving today (it carries no nourishment) and FE-a does not add it; that is a recorded non-goal.
  `FoodEnergyLine` lives in a new `clients/agent/src/mineworld_agent/food_energy.py` (or the smallest existing
  module that is not `models.py`, 1680 lines), re-exported where models are collected. The generated reference
  `docs/reference/agent-tools.{json,md}` is regenerated by `clients/agent/src/mineworld_agent/reference.py`, never
  hand-edited. The agent guide chapter (`docs/guide/09-who-decides.md` Step 9.3) uses a unitless world and its text
  does not change.

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
| Needs tests | `systems/needs/tests/{configuration,lazy,perception,reactions,removable,section}.rs`, `systems/needs/tests/support/mod.rs` | Typed API update; unitless variant asserted explicitly; support gains food-energy builders. |
| SDK integration test | `sdk/rust/tests/interactions.rs` (l. 239–244 matches `ConfigurationRefusal` exhaustively) | Edit in C1: add the `Owner` arm. |
| Server host error | `server/src/host.rs::HostError::build` (l. 509) | Audit in C1; edit only if its `Display` drops nested owner/code/subject/field. |
| CLI needs tests | `tools/cli/tests/{needs_sections,needs_town,needs_controllers,needs_fixtures}.rs`, `tools/cli/tests/fixture/needs.rs` | `needs_sections`: edit (v2 forms). `fixture/needs.rs`: add a food-energy copy form beside the existing removal form (§4 Fixtures). `needs_fixtures`: one new ignored recorder. `needs_town`, `needs_controllers`: unchanged-verified (unitless Market Town). |
| Rule controller | `cognition/rule-controller/src/needs.rs` (774 l.) and `needs_tests.rs` (+ `fetch.rs`, `paced.rs`, `offered.rs` read-only) | Schema-aware decode; same normalized values and ranks; no policy change; `needs_tests.rs` gains v2/malformed cases. |
| Motivated | `cognition/motivated/src/considerations.rs`, `needs_tests.rs` | Consumes the same projection; no change expected beyond decode; tests unchanged-verified unless decode changes. |
| LM triggers | `cognition/lm-controller/src/mineworld_cognition/controller/triggers.py` | Unchanged-verified: word-based trigger; test proves a numeric reserve change alone does not fire. |
| Agent reference | `clients/agent/src/mineworld_agent/reference.py` → `docs/reference/agent-tools.{json,md}`; tests `clients/agent/tests/{test_cli,test_guide_chapter,test_http_endpoint}.py` | Regenerate for `FoodEnergyLine` (§3.8); run these tests. Guide chapter text unchanged. |
| Generated docs | `docs/reference/{sections,interaction-list,configure}.md` (checked by `tools/cli/tests/reference.rs`) | Regenerate with the schemas in C3; no hand edits. |
| CHANGELOG | `CHANGELOG.md` | C3: Added food energy; Changed `needs` saves (old needs saves need the previous build); Fixed empty save left by a failed creation (C2's fix, entry written with C2). |
| Shared Godot | `clients/protocol/mineworld/{needs,observation}.gd`; `clients/protocol/checks/needs_check.gd` | Envelope-aware decode; replacement row. |
| 2D | `clients/2d/scripts/hud/{readers,panels}.gd`; `clients/2d/scripts/harness/needs_rows.gd` | Reserve row with owner bar/max; unitless unchanged. |
| 3D | `clients/3d-spike/scripts/slice/{needs_hud,needs_rows_probe}.gd` | Same row semantics. |
| Wording | `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po` | New keys for food energy, rate, serving (existing gettext convention, audited at edit). |
| Agent | `clients/agent/src/mineworld_agent/{disclosed,view}.py` + new `food_energy.py` (models.py touched only to add the optional field); `clients/agent/tests/test_needs_view.py` | `FoodEnergyLine` per §3.8, schema-preserving selector. |
| LM | `cognition/lm-controller/src/mineworld_cognition/controller/{situation,prompt}.py`; tests `test_needs_prompt.py`, `test_needs_triggers.py`, `test_controller_choice.py` | Optional typed summary, one bounded line. |
| Python wire | `sdk/python/src/mineworld_sdk/wire/contract.py` | Generic envelope already carries schema; verify, no change expected. |
| Fixtures | `tests/fixtures/observations/needs/*` | Keep schema-1 recordings byte-identical. The synthetic food-energy world is a test copy of Market Town made by `tools/cli/tests/fixture/needs.rs` (food-energy form: `hunger_model: food-energy`, `energy_kcal_per_day: 2000`, a 2400 actor class, every needs person `food_energy.initial_kcal: 1500`, test kcal/servings on items — synthetic values, labelled so). The new observation `market-town-food-energy.json` is **recorded** from the real binary by a new ignored recorder in `needs_fixtures.rs`, added to `SOURCE.md` with its sha256 and "Hand edits: none"; expected en/zh_Hans rows are written by hand from that recording. The same copy, written by the recorder to a scratch directory under `/Users/yuema137/mineworld-worktrees/`, is the world for the E6 2D/3D launches; it keeps the `market-town` pack id, so the 3D client uses Market Town's depiction (or draws plainly, `registry.gd:71`). |
| Reference | `docs/reference/schema/{configure-needs,section-needs}.schema.json`; CLI `reference` | Regenerate via the existing pipeline; no hand edits. |
| Specs | `docs/MODULE_SPEC.md` needs sections; new ARC record; `docs/decisions/ARC-284.md`, `ARC-285.md`, `DEP-140.md` | C0: new ARC (food-energy reserve model; options relabel / replace all worlds / second store / reserve variant; `ResolvesSubjects` contextual-authoring alternative rejected for the genesis check; kernel error amends S35 I-S35-1); dated notes on ARC-284/285 pointing to it; dated note on DEP-140: no nutrition library or data dependency in FE-a, USDA FoodData Central / manufacturer labels as FE-b content sources. |
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
2. **Scope:** `docs/MODULE_SPEC.md` needs/configuration/disclosure/persistence/error sections; one **new ARC record**
   (next free id per `docs/decisions/INDEX.md`) for the food-energy reserve model and its options (see §4 Specs row),
   amending S35 I-S35-1 ("kernel and contracts unchanged") for one domain-neutral error; dated notes in
   `docs/decisions/ARC-284.md` and `ARC-285.md` pointing to it; dated note on `DEP-140.md` (no nutrition library/data
   dependency in FE-a; USDA FoodData Central and manufacturer labels as FE-b content sources); FDA Daily Value
   citation (21 CFR 101.9(c)(9)) and the constant-drain rationale of §3.1 in the spec text. No code.
3. **Implementation:** [ ] record freeze reference and base; [ ] write §3 contracts into owning specs;
   [ ] new ARC record + INDEX; [ ] ARC-284/285 and DEP-140 notes; [ ] link this PR from the step ledger.
4. **Validation:** [ ] `python3 scripts/check_doc_headings.py`; [ ] `python3 scripts/check_decision_ids.py`;
   [ ] reviewer compares spec text with §3 and the step §1 answers.
5. **Acceptance:** no unresolved contract ambiguity; the 2000 kcal and USD 10,000 requirements are stated as
   retained, the latter as FE-b content.
6. **Failure/edges:** a source change since `8c971f0f` touching a mapped seam → bounded re-audit before C1; any
   contract change beyond §3 → material stop.
7. **Commands/evidence:** the two scripts above; record outputs.
8. **Boundary:** docs-only semantic commit. [ ] Implementation [ ] Validation [ ] Independent review

### C1 — SDK owner versions/validation hook and the generic genesis error

1. **Goal:** add exactly the generic seams needs requires, with every other pack unchanged.
2. **Scope:** `sdk/rust/src/interactions/{mod,lookup,resolve}.rs` + `tests.rs`; `sdk/rust/tests/interactions.rs`
   (exhaustive `ConfigurationRefusal` match gains `Owner`); `authoring/src/configuration.rs`;
   `worldpack/src/{configure,error}.rs` + configuration refusal tests; `kernel/src/error.rs`; `server/src/host.rs`
   only if the `HostError::build` audit shows nested fields are dropped. No dispatch behaviour, no domain type, no
   Person field. Depends on C0.
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
   `cargo test -p mineworld-sdk --test interactions`,
   `cargo test -p mineworld-worldpack --test configuration` and `--test interaction_sections`,
   `cargo check -p mineworld-kernel -p mineworld-sdk -p mineworld-worldpack`, clippy on touched crates, `cargo fmt`.
8. **Boundary:** inert seams + discriminating loader/version/error evidence. [ ] Implementation [ ] Validation
   [ ] Independent review

### C2 — Valid-genesis-first save creation in persistence and both CLI callers

1. **Goal:** no save artifact for a world that fails to assemble or begin; no double genesis, fake preflight or
   deletion rollback.
2. **Scope:** `persistence/src/{world,backend,lib}.rs` + `persistence/tests/save.rs`; new-save branches of
   `tools/cli/src/serve.rs` and `tools/cli/src/run.rs`; one new CLI test target for the creation lifecycle (proposed
   name `save_creation`, not an existing target); `CHANGELOG.md` "Fixed" entry. SQLite format unchanged. Depends on
   C1 only for the error type.
3. **Implementation:** [ ] common internal body; [ ] `create_with_backend`; [ ] `create` delegates; [ ] both callers
   assemble first and pass the factory closure; [ ] success message and providers only after initialize;
   [ ] CHANGELOG Fixed entry.
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
2. **Scope:** `systems/needs/src/*` + new `food_energy.rs`; `systems/needs/tests/*` incl. `support/mod.rs`; the
   food-energy copy form in `tools/cli/tests/fixture/needs.rs` and a second exponent-0 synthetic world (never
   `worlds/*`); CLI tests `needs_sections` plus a synthetic nutrition integration target (proposed `food_energy`);
   `cognition/rule-controller/src/{needs,needs_tests}.rs` and `cognition/motivated` decode only if the v2 record
   requires it; regenerated `docs/reference/schema/*needs*.schema.json` and `docs/reference/{sections,
   interaction-list,configure}.md`; `CHANGELOG.md` Added/Changed entries. `needs_town`/`needs_controllers` are run
   unchanged. Depends on C1, C2.
3. **Implementation:** [ ] typed variants, named units, schema 2 table; [ ] `validate_authored` for needs with the
   two configuration codes; [ ] exclusive person/item forms and serving bounds; [ ] genesis validation before insert
   using `EntityRecord::key()` and `caused_by()` (two genesis codes); [ ] `i128` arithmetic, zero-kcal no-fold, sleep
   mode from tiredness; [ ] normalized projection, owner bar, `replaces_need`, holder-only; [ ] still no
   Process/runtime fact; [ ] regenerate references; [ ] CHANGELOG (Added food energy; Changed: saves of worlds with
   `needs` made by earlier builds need that build).
4. **Validation:** [ ] E2 oracles; [ ] E1 real `run --save`/`server --save` invalid model and invalid capacity: no
   save, no listener, visible diagnostic; [ ] E3 two-currency purchase/consume/rejection/zero drink/actor class 2400;
   [ ] E4 real close/reopen, kill/resume, and old-schema/drift refusal on scratch closed + WAL families with whole
   logical row comparison, then matching-version replay of the copy; the test pins which error an old unitless
   Market-Town-shaped needs save gets first and that it names `needs` (§3.7); [ ] E5 unitless, removal (incl.
   `restore.hunger: []` in food-energy) and witness baselines; [ ] rule/motivated projections give unchanged ranks
   and absent needs stays inert; [ ] an `InvalidGenesisContent` raised while reducing a non-genesis fact surfaces as
   an ordinary reduction fault, not as authored-input diagnostics (§3.5).
5. **Acceptance:** formulas, refusals, atomicity, restart and privacy hold through real owners; unrelated digests
   unchanged; no synthetic price is described as retail.
6. **Failure/edges:** overflow widened first; backward time is an invariant failure; corrupt schema is a fault, not
   authoring input; a fixture never writes an operator save or deletes WAL.
7. **Commands/evidence:** via `with-slot.sh`, one file at a time: `cargo test -p mineworld-needs --test <file>` for
   each touched file, `cargo test -p mineworld-cli --test needs_sections`, `--test food_energy`, `--test reference`,
   `--test needs_town`, `--test needs_controllers`, `cargo test -p mineworld-rule-controller --lib needs` and the
   motivated needs tests (crate names checked at edit); clippy/fmt. No 300-day or long run.
8. **Boundary:** one owning System Pack + its synthetic integration evidence; no content, price or controller policy.
   [ ] Implementation [ ] Validation [ ] Independent review

### C4 — Schema-aware shared views and actual both-client proof

1. **Goal:** every view shows the owner's kcal truthfully and rejects malformed data without inventing a bar.
2. **Scope:** paths in §4 rows Shared Godot, 2D, 3D, Wording, Agent, Agent reference, LM, LM triggers, Fixtures.
   No transport change, no generic SDK needs model, no inference policy change. Depends on C3.
3. **Implementation:** [ ] envelope-aware selection; [ ] strict v2 + explicit v1 branch; [ ] replacement row via
   `replaces_need`; [ ] en/zh_Hans kcal/rate/serving strings; [ ] agent `FoodEnergyLine` in its own module and
   regenerated `docs/reference/agent-tools.{json,md}`; [ ] LM optional summary + one bounded line; [ ] ignored
   recorder → `market-town-food-energy.json` + SOURCE.md hash; [ ] hand-built tests lacking schema state schema 1
   explicitly; existing recorded fixture bytes untouched.
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
   `uv run --locked pytest clients/agent/tests/test_needs_view.py clients/agent/tests/test_cli.py clients/agent/tests/test_guide_chapter.py clients/agent/tests/test_http_endpoint.py`
   (one member) and
   `uv run --locked pytest cognition/lm-controller/tests/test_needs_prompt.py cognition/lm-controller/tests/test_needs_triggers.py`;
   ruff/pyright on touched Python per `.structured-coding/standards.md`.
8. **Boundary:** fact rendering + typed adapters + actual client evidence; no UI redesign or model claim.
   [ ] Implementation [ ] Validation [ ] Independent review

## 7. Terminal evidence

READY FOR OPERATOR REVIEW requires: C0–C4 committed with ledger evidence; E1–E7 recorded; actual both-client E6 and
lifecycle E4 complete; independent review complete with findings disposed; PR open; required CI green on the exact
final head, recorded separately from local results. Review starts while CI runs. No redundant local full suite.

## 8. Review requirements

- **Before freeze:** red-team pre-freeze review `/Users/yuema137/mineworld-worktrees/_reviews/design-fe.md` on head
  `6105eea7` against `8c971f0f`: **FREEZE-AFTER-FIXES**, no BLOCKER, four MAJOR. Disposition: #2 re-audit (§2);
  #3 Q-FE-2 consequence stated (step §1, §3.7) and CHANGELOG in scope — needs the operator's confirmation;
  #4 per-person overrides asked as Q-FE-6; #5 agent tool schema (§3.8, §4, C4); #6 caller map (§4, C1, C3);
  #7 FDA citation and constant-drain rationale (§3.1, C0); #8 new ARC + DEP-140 note (C0); #9 `create` kept, reason
  (§3.6); #10 edge cases (§3.2, §3.5, §3.7, C3); #11 size triggers (below); #12 fixture provenance (§4 Fixtures).
- **Existing size debt in touched files at `8c971f0f`** (triggers, not new debt): `clients/agent/src/mineworld_agent/models.py`
  1680 l., `sdk/rust/src/interactions/section.rs` 1013, `worldpack/src/error.rs` 911, `cognition/rule-controller/src/needs.rs`
  774, `kernel/src/error.rs` 769, `clients/agent/src/mineworld_agent/view.py` 747, `tools/cli/src/run.rs` 686. New code
  goes to new modules (`systems/needs/src/food_energy.rs`, `clients/agent/src/mineworld_agent/food_energy.py`); these
  files receive only the minimal hook/variant/field/call-site lines, and each commit's ledger records their line delta.
- **Each commit and at readiness:** ownership, debt, function/file growth (50/100 lines, 500/800 lines as triggers),
  coupling,
  lifecycle failure paths, error context/precedence, typed malformed-record handling, no client rules, no new
  dependency, no world switch.

## 9. Evidence ledger

- 2026-11-27, lane L1, revision 2: red-team findings applied (§8); re-audit at `8c971f0f`: 45/47 hashes equal, two
  changed files reviewed, no contract change.
- 2026-11-27, lane L1: design only. Read-only audit at `7718d952`; 47-file hash verification PASS; no code, build, test,
  Godot, model or heavy job run. All C0–C4 items `[ ]`.
