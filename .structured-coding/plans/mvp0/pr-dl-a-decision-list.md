# PR S24 DL-a — The decision list: `decisions.yaml`, the `DecisionModel` trait, `cognition/installed`, host wiring

## DESIGN FROZEN 2026-10-12 (primary session, operator standing instruction: recommendations accepted)

```text
Design revision:        revision 2 (2026-10-12): revision 1 (PR #177, first commit) with §14.1's rulings
                        and the cross-design ruling on QFGb-1 (memory: decoded and passed through as typed
                        JSON MemoryPolicyChoice, SD-DLa-9, SD-DLa-15) applied
Approved by / evidence: the primary session, 2026-10-12, under the operator's standing instruction
                        (recommendations accepted); relayed to this planning session (§14.1)
Implementation base:    origin/main after DOC-24 has merged; C0 records the commit
Start condition:        DOC-24 merged (met: #175, ARC-84 and ARC-85 are in docs/DECISIONS.md) AND the
                        operator's go. Implementation starts only on the operator's go
Execution contract:     §13 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§3.1), invariants (§3.3), decisions SD-DLa-1 … SD-DLa-15, acceptance AC-DLa-1 … AC-DLa-11 and the
commit plan are frozen. Progress, evidence, findings and bounded corrections stay writable (§15). No DL-a
question remains open (§14.1). **Implementation starts only on the operator's go**, in a fresh session
(§13); this planning session does not implement.

*Superseded header:* `DESIGN DRAFT — awaiting the primary session's and the operator's review (not frozen)`,
revision 1.

**Effort:** `mvp0` (filed here under step-24 QS24-14) · **Step:**
[`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md) §§3, 4, 5, 10.2 (AC-DL-1,
AC-DL-3), 11.2 (the DL-a row), 12 (R-S24-8), 13 (the `MODULE_SPEC.md` rows), 14.1 (rulings) · **Parent:**
[`overall.md`](overall.md), 2026-10-12 section ("MVP-0 PRs from step-24") · **Decision ids this PR
implements:** `ARC-84` (the decision list is host-read content, not world state) and `ARC-85` (behaviour
extension points: interface, carrier, selection, parameters, host), both recorded by DOC-24 · **Followed by:**
[`pr-dl-b-motivated.md`](pr-dl-b-motivated.md) (DL-b depends on this PR) · **Consumed later by:** S10 P6
(the `lm` entry and its fallback, AC-DL-5), WE-b (director seats, MVP-1).

**Working name:** DL-a. The PR number is assigned when the implementation PR opens.
**Planning base:** `origin/main @ cda4c13` (#174); frozen on `origin/main @ d14f05e` (DOC-24 #175 merged,
F-DLa-2). Planning worktree
`/Users/yuema137/mineworld-worktrees/design-dl`, branch `docs/dl-design`. This planning session writes
this file and `pr-dl-b-motivated.md` only.

### Binding rulings this design is written under

| Source | Ruling |
| --- | --- |
| step-24 QS24-1 | The decision list is (a): World Pack content read by hosts, never seeded, with operator override. `run` prints its digest; a save does not store it |
| step-24 QS24-3 | `rule` stays the default in MVP-0; every reference world byte-identical |
| step-24 QS24-12 | `mineworld run` hosts no out-of-process model in MVP-0; such a seat runs its fallback there |
| step-24 QS24-14 | MVP-0 PRs are filed under `mvp0/` |
| operator, 2026-10-12 | Every extension point pluggable; users can add their own decision models; LLMs only propose; the world stays valid with models removed; same seed → same history; macOS, Linux and Windows |

---

## 1. Goal, in one paragraph

Today every in-process seat is driven by one `PacedRuleController::new(seed, PACE)`, hard-wired in two
hosts (`tools/cli/src/run.rs:125`, `tools/cli/src/serve.rs:192` via `hosted::PacedSeat`). DL-a makes **how
each seat decides** a selectable, typed, per-seat choice — the *decision list* — without changing any byte
of any world that does not ask for it. It adds a small trait crate (`cognition/decision`) every in-process
decision model implements, a build-level installed set of models (`cognition/installed`, the `ARC-33`
pattern), the `decisions.yaml` format (default, per-class, per-seat, typed parameters, fallback chains),
its decoding with positioned refusals, the operator overrides `--decisions` and `--seat-model`, and the
wiring of both hosts to resolve a model per seat. `rule` — today's paced controller, unchanged — is the
first and only shipped model in this PR; `motivated` follows in DL-b.

---

## 2. Audit — what exists on `main @ cda4c13`

### 2.1 The controller and its two hosts

| Item | Where | What matters for DL-a |
| --- | --- | --- |
| `PacedRuleController { seed, pace }` | `cognition/rule-controller/src/paced.rs` | `decide(&self, &Observation<Value>)` and `step(&self, …)` are pure; `pace` is the answer window (a line heard in `(t − pace, t]` is answered once) — so pace is **host-supplied context**, not a model preference |
| `Draw`, `mix` | `paced.rs` (`pub(crate)`) | `Draw::new` mixes `seed ^ 0x6d69_6e65_776f_726c`, observer, instant with SplitMix64's finalizer; used by `agenda.rs`, `social.rs`, `offered.rs`. Draw indices 0–15 documented in the module header |
| `RuleController` (reactive) | `src/lib.rs` | `&mut self`, stateful by design (F-13); drives `--agent SEAT` only; stays outside the trait (step-24 §5.1) |
| `walks(observation)` | `src/walking.rs`, `pub` | read by `hosted::PacedSeat` to decide whether to schedule step consults |
| crate dependencies | `cognition/rule-controller/Cargo.toml` | contracts plus the System Pack vocabularies it reads (conversation, movement, group-activity, naming, schedule) and packages. **Finding F-DLa-1:** step-24 §5.1 says the rule controller "depends on `mineworld-contracts` only"; it does not — the *trait* crate can, a *model* crate depends on the vocabularies it reads. Recorded, no design change |
| `PACKAGE` | `src/lib.rs:300` | a model crate's package identity (`ARC-53`); `tools/cli/src/packs.rs:31` lists `CONTROLLERS: [Package; 1]` by hand |
| `run` | `tools/cli/src/run.rs` | one `controller` for all seats (line 125); `Consult::run(&PacedRuleController, …)`; header line `[mineworld] run <name> (<id>): seats …; seed N; to day D; pace 900; <how>` |
| `server --town` | `tools/cli/src/serve.rs` `drivers` / `Driver::{Reactive, Paced(k)}`; `tools/cli/src/hosted.rs` `PacedSeat` | `--agent` seats → reactive; with `--town` every other seat → `PacedSeat::new(pacing, epoch, k, &tally)`; without `--town` no in-server controller. `PacedSeat` owns the lattice and walking logic, model-independent apart from `controller.decide/step` |
| CLI | `tools/cli/src/main.rs` `Subcommand::{Server, Run, Validate}` | clap derive; seat parser `seat()` |

### 2.2 The World Pack format and the precedents to reuse

| Item | Where | Use in DL-a |
| --- | --- | --- |
| `WorldManifest`, `deny_unknown_fields` | `worldpack/src/format.rs` | a `decisions:` key must be added or `world.yaml` naming it is refused |
| `packages` framework key: host-read, never seeded, never compared at resume | `worldpack/src/configure.rs` `licence_policy`, `FRAMEWORK` | the precedent for "pack content a host reads that is not world state" |
| seeded decoding with positions | `worldpack/src/content.rs` (`ContentFile: DeserializeSeed`), `read.rs` `parse_with` | the decision list is decoded the same way: one pass through a `DeserializeSeed` that knows the installed models and the pack's seats and classes, so every refusal keeps its line and column (`DEP-10`) |
| `parameters!`, `Parameters`, `Fields::decode_field` (bounds checked while positioned) | `sdk/rust/src/interactions/section.rs` | reused for model parameter blocks (§5.4) |
| `installed!` (monomorphic catalog enum, `DEP-12`) | `sdk/rust/src/installed.rs`, `systems/installed/src/lib.rs` | `cognition/installed` follows the same pattern with its own, much smaller macro |
| `EntityClasses::class_of` — first matching definition, else the implicit class | `authoring/src/classes.rs` | a person is in **exactly one** defined class (or only the implicit `person`), so "two class assignments match one seat" cannot happen; the only class-level ambiguity is a class named twice (§5.3) |
| `pack.seats()`, `pack.people()[key].tags`, `pack.classes()` | `worldpack/src/read.rs` | the roster the decoder checks against |
| `run` output in `ci_parity.py` | `scripts/ci_parity.py` `summary()` | `summary-300` **keeps the header line** (memory mode); `summary-30s` drops lines starting `[mineworld] run `. So any change to the header in the absent case changes `summary-300` and breaks AC-DL-1 (§6) |
| baselines | `scripts/baselines.txt` | `summary-300`, `summary-1000`, `summary-300s`, `facts-300s`, `journal-300s` per world, seed 7. **PR #171 (12d) re-baselines** bodies-yard/market-town; DL-a compares against the file at its own base |
| per-crate `clippy.toml` | workspace `clippy.toml` (HashMap/HashSet disallowed) | clippy reads the nearest `clippy.toml` from the crate's manifest directory upward; a model crate's own file must repeat the workspace entries and add `Cell`, `RefCell`, `Mutex`, `RwLock`, `OnceCell`, atomics (R-S24-8) |
| hashing | `Cargo.lock` has no `sha2`/`blake3` | the decision-list digest reuses `run`'s FNV-1a 64 (for reading, not evidence), no new dependency |

### 2.3 What does not exist

No `cognition/decision`, no `cognition/installed`, no `decisions:` key, no `--decisions`/`--seat-model`
flags, no `ARC-84`/`ARC-85` (DOC-24 not merged at planning base). `cognition/lm-controller` is Python and
is not a Cargo member; the workspace `members` list names `cognition/rule-controller` explicitly, so new
cognition crates are added by name (a `cognition/*` glob would match the Python directory and fail).

---

## 3. Scope

### 3.1 In scope

1. **`cognition/decision`** (`mineworld-decision`, new): the `DecisionModel` trait, `Seed`, `Consult`,
   `ModelId`, the erased `SeatModel`, `Draw` and `mix` (moved verbatim from `paced.rs`), the
   `ModelParameters` trait with the `parameters!` adapter, the `decisions.yaml` authored format, its
   decoder (`DecisionList::read`), resolution (`ResolvedDecisions`), and the digest. Depends on
   `mineworld-contracts`, `serde`, `serde_json`, `serde-saphyr`, `mineworld-sdk` (for the `Parameters`
   traits only; §5.4).
2. **`cognition/installed`** (`mineworld-installed-models`, new): the build's list of in-process models via a
   `models!` macro defined in `mineworld-decision`; `rule` is its only line in DL-a, plus `external` lines
   (none shipped; §5.6).
3. **`cognition/rule-controller`**: `Draw`/`mix` come from `mineworld-decision` (byte-identical); a
   zero-sized `Rule` model implements `DecisionModel` by delegating to `PacedRuleController`; no change to
   `PacedRuleController`'s API or behaviour; its own `clippy.toml`.
4. **`worldpack`**: `WorldManifest.decisions: Option<RelativeFile>` (the file's existence and that it lies
   inside the pack are checked at read; its content is not worldpack's — SD-DLa-3).
5. **`tools/cli`**: `run` and `server` resolve one model per seat; `--decisions FILE` and
   `--seat-model SEAT=MODEL` on both; `validate` decodes the pack's decision list; `run`'s header carries
   the digest **only when a decision list is in effect**; `hosted::PacedSeat` drives a `Box<dyn SeatModel>`;
   `packs.rs` derives its controller list from `cognition/installed`; a new read-only subcommand
   `mineworld decisions <world> [--decisions FILE] [--seat-model SEAT=MODEL]… --json` that prints each seat's
   resolved model and its `MemoryPolicyChoice` as typed JSON for the cognition process (SD-DLa-15, QFGb-1).
5a. **`memory:`** section of `decisions.yaml`: decoded structurally, resolved per seat (seat > class >
   default), passed through uninterpreted as `MemoryPolicyChoice` (SD-DLa-9).
6. **Template determinism test** over every installed model (R-S24-8): a CLI integration test that runs a
   reference world with every seat on each installed model twice and across a kill-and-resume, and compares
   save bytes.
7. **Specifications:** `docs/MODULE_SPEC.md` §4.1 (`decisions:` and the file's shape; never seeded) and §5
   (decision models, `cognition/installed`, how a user installs one) — DOC-24 (#175) already wrote both;
   DL-a's C1 **amends** them where this frozen design differs (F-DLa-2), and adds a dated note on `ARC-84`; `docs/DECISIONS.md` dated notes on
   `ARC-27` (stateless seeded controllers generalised to every in-process model) and `ARC-33` (the installed
   pattern reused for models); READMEs for the two new crates (human-facing, short).

### 3.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The `motivated` model, curves, ATUS data | DL-b |
| Semantic validation of `memory:` (policy ids, their parameters, fallback rules) | FG-b's `resolve` in the cognition process, at cognition start (QFGb-1); DL-a decodes the structure, resolves per seat and passes the choice through (SD-DLa-9) |
| The `lm` model and AC-DL-5 (fallback on a killed cognition process; cassette replay) | S10 P6, which adds an `external lm` line |
| Director seats on Organizations | WE-b (MVP-1, QS24-2) |
| Storing the resolved list in a save | deferred by QS24-1 |
| A change to `RuleController` (`--agent`) | none; it stays the reactive controller |
| Python-side consumption of the resolved list | FG-b (`resolve` of `MemoryPolicyChoice`) and P6 (R-P6-FG-2), reading SD-DLa-15's JSON; no Python YAML parser |

### 3.3 Invariants this PR must hold

- **I-DLa-1 (absent means today).** No `decisions:` key, no `--decisions`, no `--seat-model` → every seat
  `rule`, every byte of every save and every line of `run`'s output as on the base (AC-DL-1).
- **I-DLa-2 (never world state).** Nothing of the decision list is seeded, journaled, compared at resume or
  stored in a save (`INV-1`, `ARC-84`). Resuming a save with a different decision list is allowed.
- **I-DLa-3 (pure models).** Every in-process model decides as a pure function of `(model parameters, seed,
  pace, observation)`; `&self`, no interior mutability, no clock, no RNG but `Draw` (`ARC-27` extended).
- **I-DLa-4 (models cannot grant).** A model returns at most one `ActionRequest`, revalidated at dispatch;
  parameters change preferences, never what is offered (step-24 §4.3).
- **I-DLa-5 (refused by name).** Unknown model, class, seat, non-seat person, parameter, or out-of-bound
  value is refused at load with file, line and column; nothing unknown is ignored.
- **I-DLa-6 (no kernel, contract, System or server change).** `kernel/`, `contracts/`, `systems/`,
  `server/`, `persistence/` are untouched. `mineworld-server` still names no controller crate.
- **I-DLa-7 (determinism across platforms).** Integer-only; `BTreeMap`/`Vec` only; resolution order is the
  roster order.

---

## 4. Reuse

| Need | Option | Decision |
| --- | --- | --- |
| Plug-in registration | dynamic loading, `inventory`/`linkme` linker sections, a declared installed set | **The declared installed set**, as `ARC-33`/`DEP-12` chose for System Packs: monomorphic, no unsafe, one line per model; installing is a `Cargo.toml` line plus a macro line and a rebuild |
| Parameter blocks with bounds | hand-written validation per model; `parameters!` (IL-a) | **Reuse `parameters!`**; a model whose parameters do not fit scalar literal bounds implements `ModelParameters` directly (DL-b does, for curve tables) |
| YAML decoding with positions | `serde-saphyr` (`DEP-10`) | reused; seeded decoding as `worldpack/src/content.rs` does |
| Digest | SHA-256 crate; FNV-1a 64 already in `run.rs` | **FNV-1a 64**, labelled "for reading, not evidence" exactly as `run`'s fingerprint; no new dependency |
| Utility-AI / BT frameworks | `big-brain`, `bonsai-bt` | not relevant to DL-a (DL-b, `DEP-47`) |

---

## 5. Design

### 5.1 The trait (SD-DLa-1)

`cognition/decision/src/model.rs`:

```rust
/// What every consult is given besides the observation: host context, never preferences.
pub struct Consult<'a> {
    pub seed: Seed,                       // the run's seed, the same for every seat (today's behaviour)
    pub pace: SimDuration,                // the answer window: the host's consult period in world seconds
    pub observation: &'a Observation<Value>,
}

/// An in-process decision model: what a seat attempts, as a pure function of what it is given.
pub trait DecisionModel: Default + Send + Sync + 'static {
    /// The id named in decisions.yaml, e.g. "rule". Validated as a ModelId by the installed set's test.
    const ID: &'static str;
    /// Typed, bounded parameters with defaults (§5.4). `()` for none.
    type Parameters: ModelParameters;
    /// The consult: at most one request.
    fn decide(&self, parameters: &Self::Parameters, consult: &Consult<'_>) -> Option<ActionRequest>;
    /// The next stride of a disclosed walk, between consults (as `PacedRuleController::step`).
    fn step(&self, parameters: &Self::Parameters, consult: &Consult<'_>) -> Option<ActionRequest>;
}

/// The object-safe form a host holds per seat: a model with its decoded parameters.
pub trait SeatModel: Send + Sync {
    fn id(&self) -> &ModelId;
    fn decide(&self, consult: &Consult<'_>) -> Option<ActionRequest>;
    fn step(&self, consult: &Consult<'_>) -> Option<ActionRequest>;
}
pub struct Configured<M: DecisionModel> { model: M, parameters: M::Parameters, id: ModelId }
impl<M: DecisionModel> SeatModel for Configured<M> { … }
```

**Refinement of step-24 §5.1, recorded (bounded):** step-24 sketches `decide(&self, p, seed, obs)`. The
answer window `pace` is also host context that a model cannot know otherwise (`PacedRuleController::new`
takes it), so it travels in `Consult`. `const ID` and an associated type make `DecisionModel` not
object-safe; hosts hold `Box<dyn SeatModel>` built by the installed set. Neither changes what step-24
approved.

### 5.2 `rule` as the first model, byte-identical (SD-DLa-2)

`cognition/rule-controller/src/model.rs`: `#[derive(Default)] pub struct Rule;` with `ID = "rule"`,
`Parameters = ()`, `decide` = `PacedRuleController::new(consult.seed.raw(), consult.pace).decide(consult.observation)`,
`step` likewise. `PacedRuleController` is `Copy` and two words; constructing it per consult costs nothing and
keeps the type and its tests unchanged. `Draw` and `mix` move to `mineworld-decision` **verbatim**, including
the salt `0x6d69_6e65_776f_726c`; a model that wants an independent stream calls `Draw::salted(salt, seed,
observation)` (DL-b uses its own salt). `Draw::new(seed, observation)` stays the rule's constructor.

### 5.3 The file, its decoding and resolution (SD-DLa-3 … SD-DLa-8)

`world.yaml`:

```yaml
decisions: decisions.yaml        # optional; a path relative to the pack root, inside it
```

`decisions.yaml` (DL-a's accepted shape; P6/FG-b/WE-b extend it):

```yaml
models:                          # every model this file uses, once each; omitted entries use defaults
  rule: {}
  probe:                         # (test-only model in DL-a's tests)
    parameters: { patience: 3 }
default: rule                    # optional; absent = rule
assign:                          # optional
  - { class: staff, model: probe }     # an ARC-64 defined class (configure/classes.yaml)
  - { seat: alice,  model: rule }
```

- **SD-DLa-3 Ownership.** `worldpack` owns only the `decisions:` key: it checks the path is relative, stays
  inside the pack root after normalisation, and exists (`PackError::DecisionsFileMissing`,
  `PackError::DecisionsOutsidePack`). It never parses the file. `mineworld-decision` owns the format; the
  host (`tools/cli`) is the composition root that hands it the installed models and the pack's roster. This
  keeps `worldpack` ignorant of controllers (its dependency direction is world → systems → kernel) and
  `mineworld-decision` ignorant of World Pack files beyond the one it reads.
- **SD-DLa-4 Roster.** The host builds `Roster { seats: Vec<SeatEntry { seat: EntityKey, class: ClassRef }>,
  classes: BTreeSet<ClassRef>, people: BTreeSet<EntityKey> }` from `pack.seats()` (roster order),
  `pack.classes().class_of(Classed { entity_type: Person, tags })` and `pack.people()`. `ClassRef` is a
  validated identifier in `mineworld-decision` (same spelling rule as `ClassName`, which it does not import:
  `mineworld-authoring` depends on the kernel).
- **SD-DLa-5 One positioned pass.** `DecisionList::read(path, &Catalog, &Roster)` decodes with a
  `DeserializeSeed` carrying the catalog and the roster, so each refusal is raised while the decoder stands on
  the offending node: unknown top-level key; a structural error in `memory:` (SD-DLa-9); unknown model id (lists the installed
  ones); a model named twice; unknown parameter or out-of-bound value (through the model's decoder, §5.4);
  `seat:` naming no person; `seat:` naming a person who is not a seat (step-24 §4.2 rule 2); `class:`
  naming an undefined class; `class: person` (or any implicit class: "use `default:`"); an entry with both
  or neither of `seat`/`class`; two entries for one seat, or for one class; an assignment naming a model not
  listed in `models:` and not installed. The error type is `DecisionsError { path, line, column, detail }`
  and the CLI prints it as worldpack prints `PackError::Malformed`.
- **SD-DLa-6 Precedence.** seat > class > default > `rule`. Because `class_of` puts a person in at most one
  defined class, no class-level ambiguity exists beyond a class named twice (refused).
- **SD-DLa-7 Parameters per model, not per seat.** In DL-a a model has one parameter block per file (in
  `models:`). Per-seat parameter overrides are not in step-24's shape and are not added (QDLa-3).
- **SD-DLa-8 Fallback chains.** An entry under `models:` may name `fallback: <model-id | none>`. Accepted
  only for models the catalog marks `External` (§5.6); on an in-process model it is refused ("an in-process
  model is always present"). An external model **must** name one. Chains are followed at resolution: at most
  4 links, no cycle, ending in an in-process model or `none`; violations are refused at the `fallback:` node.
- **SD-DLa-9 `memory:` decoded structurally and passed through (ruling on QFGb-1, 2026-10-12; supersedes
  revision 1's refusal).** The shape is FG-b's §6 (`pr-fg-b-retention-host.md`, PR #180):

  ```yaml
  memory:
    default: { policy: consolidate }          # optional; absent → no default entry (FG-b's DEFAULT_CHOICE)
    assign:                                   # optional; seat > class > default; one entry per seat or class
      - { class: staff, policy: decay, parameters: { half_life_hours: 336 } }
      - { seat: alice, policy: lm-summarise, fallback: consolidate, parameters: { calls_per_sim_day: 4 } }
  ```

  DL-a's decoder checks, with file, line and column: unknown keys under `memory`, `default` and each entry;
  an entry with both or neither of `seat`/`class`; a non-seat or unknown person; an undefined or implicit
  class; two entries for one seat or one class; `policy` and `fallback` spelled as a `PolicyId`
  (`^[a-z][a-z0-9-]{0,47}$`, FG-b's rule); `parameters` a mapping with string keys, converted to a JSON
  object (YAML scalars to JSON scalars; a non-string key, a tag, or a non-finite number refused). It does
  **not** interpret `policy`, `fallback` or `parameters`: policy ids are Python entry points the Rust side
  cannot see. **Recorded consequence:** an unknown policy id or invalid policy parameters fail when the
  cognition process starts (FG-b's `resolve`, naming the file, line and column carried in `source`), not at
  `run`'s load. Resolution per seat uses the same roster and precedence as models (SD-DLa-4, SD-DLa-6). The
  memory choice is part of the digest (SD-DLa-14) only when a `memory:` section is present, so the absent
  case is unchanged.

**Resolution** produces `ResolvedDecisions { source: Source, per_seat: Vec<(EntityKey, Resolved)> }` in roster
order, where `Resolved` is the in-process model that will actually drive the seat in this host plus, when it
differs, the assigned external model and the chain followed. `Source` is `Implicit` (nothing given),
`Pack`, `File` (`--decisions`), each possibly `+ seat overrides`.

### 5.4 Model parameters (SD-DLa-10)

```rust
pub trait ModelParameters: Default + Clone + PartialEq + Debug + Serialize + Send + Sync + 'static {
    /// Decodes the block at the decoder's position, refusing unknown keys and out-of-bound values there.
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}
impl ModelParameters for () { … }        // refuses any key: "<model> takes no parameters"

/// For a block made by `mineworld_sdk::parameters!`: decode its Partial field by field through
/// `Fields::decode_field` (bounds checked while positioned), refuse unknown keys, then
/// `Default::default().apply(&partial)`.
pub fn bounded<'de, P: mineworld_sdk::interactions::Parameters, D: Deserializer<'de>>(d: D) -> Result<P, D::Error>;
```

A model whose parameters are made by `parameters!` implements `decode` as `bounded(d)`. The canonical form
of a decoded block for the digest is `serde_json::to_vec(&parameters)` of the full (defaulted) value, so
`{}` and an explicit default give one digest. **Dependency note:** reusing the trait means
`mineworld-decision` depends on `mineworld-sdk` (and through it the kernel crate, as every rule-controller
dependency already does transitively). If the reviewer prefers the trait crate on contracts only, the
alternative is to copy the `Fields` shape — rejected here as a second implementation of IL-a's bound check.

### 5.5 Operator overrides (SD-DLa-11)

- `mineworld run … --decisions FILE` / `mineworld server … --decisions FILE`: replaces the pack's file;
  decoded against the same pack's roster. The path is the operator's, anywhere.
- `--seat-model SEAT=MODEL`, repeatable, on both: applied after the file; the seat must be a seat, the model
  installed; parameters come from the file's `models:` block when the file lists that model, else defaults;
  the same seat twice is refused. Refusals name the flag.
- `server`: the resolved models drive seats only where an in-server controller drives a seat today — i.e.
  with `--town` (every seat not in `--agent`). `--decisions` or `--seat-model` **without** `--town` is
  refused ("decisions choose how in-server seats decide; add --town") rather than silently unused (QDLa-1).
  `--agent SEAT` keeps the reactive controller and wins over the list for that seat; a seat named by both
  `--agent` and `--seat-model` is refused.
- `validate`: decodes the pack's `decisions.yaml` when `world.yaml` names one and prints
  `decisions: <n> assignment(s), default <id>`; with no key it prints nothing new (validate's output is a
  parity key, `manifest`/`validate` in `ci_parity.py`).

### 5.6 External models (SD-DLa-12)

`models!` accepts two kinds of line:

```text
mineworld_decision::models! {
    Rule => mineworld_rule_controller::Rule,      // in-process: a DecisionModel type
    external "lm",                                // P6 adds this; none in DL-a
}
```

An external model has no Rust type: it is a Controller Pack that joins a seat over the protocol
(step-24 §5.3). In `run` its seat runs the end of its fallback chain and the header says so
(`; alice: lm → rule (run hosts no out-of-process model)`, QS24-12). In `server --town` the in-server
controller is the fallback model, and the external process takes the seat over through S11-B's existing
take-over path when it joins. DL-a ships **no** external line; the kind, the chain rules and their tests use
a test-only catalog. Recommended over waiting for P6 because the chain rules and the header wording are this
PR's format, and P6 should only add a line (QDLa-2).

### 5.7 Host wiring (SD-DLa-13)

- **`run`**: `let models: Vec<Box<dyn SeatModel>> = resolved.instantiate(&catalog)` in roster order;
  `Consult::run(&*models[k], …)` builds `mineworld_decision::Consult { seed: Seed(request.seed), pace: PACE,
  observation: &seen }`. The loop, lattice, `RUN_STEP`, allocation, tally and fingerprint are unchanged.
- **`server --town`**: `Driver::Paced(k)` becomes `Driver::Modelled { k, model: Arc<dyn SeatModel> }`;
  `PacedSeat` holds `Arc<dyn SeatModel>` and `Seed`, builds the `Consult` with its world-second pace (pace ×
  scale), and keeps its lattice/walking logic verbatim. The factory closure clones the `Arc` per binding.
  `walks()` stays `mineworld_rule_controller::walks` (a movement-disclosure reader, model-independent).
- **`packs.rs`**: `CONTROLLERS` becomes `Model::packages()` from `cognition/installed` plus
  `mineworld_rule_controller::PACKAGE` (which is also the `rule` model's package; deduplicated by name).

### 5.8 The digest in `run`'s header (SD-DLa-14)

When `Source` is `Implicit`, the header is byte-for-byte today's. Otherwise `run` appends one clause to the
header line: `; decisions <016x> (<n> rule, <m> motivated, …)`, where the digest is FNV-1a 64 over the
canonical encoding of `per_seat` (seat key, effective model id, canonical parameter JSON, chain), so two
files that resolve alike print alike. No host path appears in it (the save-mode parity record drops the
header anyway). The server prints the same clause on its `town:` line. Rationale for "only when present":
`ci_parity.py`'s `summary-300` keeps the header, so an always-printed clause would move every world's
baseline (§2.2).

### 5.9 The resolved list as typed JSON for the cognition process (SD-DLa-15; ruling on QFGb-1)

`mineworld decisions <world> [--decisions FILE] [--seat-model SEAT=MODEL]… [--packs …] --json` reads the pack
and the decision list exactly as `run` does (same decoder, same overrides, same refusals), starts no world and
writes one JSON document to stdout:

```json
{
  "format": 1,
  "world": "social-cafe",
  "digest": "00f3…",
  "seats": [
    { "seat": "alice",
      "model": { "id": "rule", "assigned": "lm", "chain": ["lm", "rule"] },
      "memory": { "policy": "lm-summarise", "fallback": "consolidate",
                  "parameters": { "calls_per_sim_day": 4 },
                  "source": { "file": "decisions.yaml", "line": 12, "column": 7 } } },
    { "seat": "bob", "model": { "id": "rule" }, "memory": null }
  ]
}
```

- Typed on the Rust side (`ResolvedDecisionsJson`, `SeatJson`, `MemoryPolicyChoiceJson`, `SourceSpanJson`,
  all `#[derive(Serialize)]` with no maps of free strings except `parameters`, which is the pass-through
  object); seats in roster order; keys in a fixed order; `memory: null` when no `memory:` entry and no default
  applies, which FG-b reads as its `DEFAULT_CHOICE`.
- `memory` matches FG-b's `MemoryPolicyChoice` field for field (`policy`, `fallback`, `parameters`,
  `source`); `source.file` is the path as given relative to the pack root, or as passed by `--decisions`.
- `format: 1` is the document's version; a change to its shape bumps it, and FG-b's reader refuses an
  unknown version.
- Without `--json` the subcommand prints the same content as a human table. The subcommand is read-only and
  never writes a save.
- P6 (R-P6-FG-2) runs it (or links the same library function) at seat start; FG-b's tests use literal JSON of
  this shape until then.

---

## 6. Determinism and replay

| Claim | Why it holds | Owner |
| --- | --- | --- |
| Absent list → identical history and output | the `rule` path constructs the same `PacedRuleController(seed, PACE)` per consult, `Draw` moved verbatim, header unchanged | AC-DLa-1 (baselines check, Gate-2-style real runs) |
| Same pack, seed, list and code → same history | models are pure by signature; resolution is in roster order; integer arithmetic | AC-DLa-6 template test |
| Kill-and-resume equals uninterrupted (`AC-6`) | models hold no state; the decision list is re-read at resume (not stored) — the operator must pass the same list; the header digest shows it | AC-DLa-6 |
| Replay (`mineworld replay`) | replays the journal; never calls a controller (`ARC-25`); the decision list is irrelevant to it | unchanged; no test added |
| Three platforms | no float, no hash order, no time | the template test runs in `test`, `test-windows`, `test-macos` CI jobs |

---

## 7. Test ownership (test rules §26)

| Layer | Owns |
| --- | --- |
| Static | `cargo fmt`, `cargo clippy -D warnings` incl. the model crates' `clippy.toml` disallowing interior mutability (mutation M-DLa-4 shows it bites); type-level object safety of `SeatModel` |
| Unit (`cognition/decision`) | decoder refusals with line/column (each SD-DLa-5 case, one table-driven test); precedence; chain rules (length, cycle, terminal kind); digest equality for equivalent files and inequality for a changed parameter; `bounded` rejects an out-of-range and an unknown key at the right position; `Draw` golden values (frozen from the base before the move — test rules §25) |
| Unit (`rule-controller`) | `Rule` via `SeatModel` equals `PacedRuleController` on the existing paced test observations (existing tests stay; one parity test added) |
| Integration (`tools/cli/tests`) | `run` with no list: header and summary identical to a golden captured on the base; `--decisions`/`--seat-model` refusals; mixed assignment drives the assigned model (a test-only probe model is not installable in the binary, so this uses `rule` vs `none`-fallback external in a test build — see C5); `server --town` with a list starts and its `town:` line carries the digest; `--decisions` without `--town` refused |
| Template (R-S24-8) | for each `Model::AVAILABLE` in-process entry: social-cafe, every seat on that model, 10 days, twice in memory → equal fingerprints; `--save` 5 + resume to 10 vs uninterrupted 10 → identical `facts` and `journal` bytes |
| Real lifecycle | AC-DL-1 over every world with the release binary: `ci_parity.py record --profile long` + `baseline check` (≈ minutes; once, at the final head) |
| Gate 1 | **Not required**: no LLM-facing semantics change |
| CI impact | new workspace members and `tools/cli` change → the selector's full fallback (`scripts/ci_changes.py` fails closed on unmapped paths); one canonical PR CI |

### Mutations (to be run and reverted, recorded in §15)

| Id | Mutation | Must turn red |
| --- | --- | --- |
| M-DLa-1 | change `Draw`'s salt by one bit in `mineworld-decision` | the `Draw` golden test and AC-DLa-1's golden |
| M-DLa-2 | resolve class before seat | the precedence unit test |
| M-DLa-3 | decoder ignores unknown keys under `assign` entries | the refusal table test |
| M-DLa-4 | add a `Cell<u64>` counter to `Rule` | clippy (`disallowed-types`) |
| M-DLa-5 | add a `RefCell` via a type alias the lint misses, mutated per consult | the template test's kill-and-resume equality |
| M-DLa-6 | always print the decisions clause | AC-DLa-1 golden header test |

---

## 8. Acceptance criteria (fixed before anything is measured, `ARC-23`)

| Id | Criterion | Counterfactual |
| --- | --- | --- |
| **AC-DLa-1** (= AC-DL-1) | With no `decisions:` key and no flag, `python3 scripts/ci_parity.py baseline check` on a long record of the final head passes for every world (every column of `scripts/baselines.txt` at the base), and `run`'s header line for social-cafe at seed 7 equals the base's byte for byte | M-DLa-1, M-DLa-6 |
| **AC-DLa-2** (AC-DL-3, refusals) | Each of: uninstalled model, unknown class, implicit class, non-seat person, unknown person, unknown parameter, out-of-bound parameter, duplicate seat entry, duplicate class entry, fallback on an in-process model, external without fallback, chain of 5, cyclic chain, and in `memory:` an unknown key, a malformed `PolicyId`, a non-seat person, an undefined class, a duplicate entry, non-mapping or non-string-keyed `parameters` — is refused at load with the file, line and column of the offending node and a message naming it | M-DLa-3 |
| **AC-DLa-3** (AC-DL-3, removal) | Removing an assignment (or the whole `decisions:` key) from a pack that had one runs the world; with the key removed the run is AC-DLa-1's | — |
| **AC-DLa-4** | Precedence seat > class > default holds on a fixture world with a defined class | M-DLa-2 |
| **AC-DLa-5** | `--decisions` replaces the pack's file; `--seat-model` overrides one seat; both refuse an unknown seat/model naming the flag; `server` refuses them without `--town` | — |
| **AC-DLa-6** (template) | For every installed in-process model: two in-memory runs give equal fingerprints; a 5-day save resumed to 10 days equals an uninterrupted 10-day save in `facts` and `journal` bytes; on Linux, macOS and Windows CI | M-DLa-5 |
| **AC-DLa-7** | `run` with a decision list prints the digest clause; two files resolving alike print the same digest; changing one parameter changes it | — |
| **AC-DLa-8** | No file under `kernel/`, `contracts/`, `systems/`, `server/`, `persistence/` changes (`git diff --stat`) | — |
| **AC-DLa-9** | `mineworld packs` lists controller packages from `cognition/installed`; its output for the base build is unchanged | — |
| **AC-DLa-11** (QFGb-1) | `mineworld decisions <world> --json` on a fixture pack with a `memory:` section prints, per seat in roster order, the resolved model and a `memory` object whose fields equal FG-b §6's `MemoryPolicyChoice` (`policy`, `fallback`, `parameters`, `source` with the winning entry's file, line, column); seat > class > default holds for memory; a pack without `memory:` prints `"memory": null` for every seat; an unknown policy id (e.g. `nonexistent-policy`) is **accepted** by DL-a and passed through (validation is FG-b's at cognition start); a golden JSON file pins the shape and key order | a resolver that drops `source`, or one that validates policy names in Rust |
| **AC-DLa-10** | `MODULE_SPEC.md` §4.1 and §5 describe the key, the file, precedence, fallback, overrides and how to install a model, citing `ARC-84`/`ARC-85`; doc checks pass | — |

---

## 9. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-DLa-1 | `Draw` move changes a draw (e.g. a `const` folded differently) | golden values frozen before the move (C1); M-DLa-1; AC-DLa-1 |
| R-DLa-2 | Host order of model construction differs from roster order | resolution in roster order; template test; AC-DLa-1 |
| R-DLa-3 | A resumed run with a different list silently diverges from the first invocation | allowed by `INV-1`; the digest is in the header of every invocation; documented in `MODULE_SPEC.md` §8.1 note |
| R-DLa-4 | 12d (#171) re-baselines while DL-a is open | DL-a rebases; AC-DLa-1 is checked against the baselines file at DL-a's own base, never edited by DL-a |
| R-DLa-5 | `mineworld-decision` → `mineworld-sdk` dependency judged too heavy | §5.4 alternative; reviewer decides at freeze (QDLa-4) |
| R-DLa-6 | User models break purity in ways clippy cannot see | the template test is automatic for every installed model (AC-DLa-6) |

---

## 10. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.
Targeted validation per commit; the PR's CI is the one full run (test rules §8–9). Commands from the
worktree root.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Start implementation against the frozen design. **Scope.** This file (ledger opened),
  `handoff-dl-a.md` (new).
- [x] Implementation: verify the `DESIGN FROZEN` header and §13; record the base commit; confirm DOC-24 is on
  it (`grep -n "ARC-84" docs/DECISIONS.md`); initialize the handoff. *Evidence E-DLa-0.*
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`. *E-DLa-0.*
- [x] Review: no authority line widened; rulings copied verbatim. *E-DLa-0.*
- **Commit boundary.** Documentation only.

### C1 — Specifications before code

- **Goal.** `CLAUDE.md` §2.2: the format and the extension point are specified before code.
- **Scope.** `docs/MODULE_SPEC.md` §4.1 and §5 as DOC-24 (#175) wrote them, amended per F-DLa-2; §8.1
  (`--decisions`, `--seat-model`, `mineworld decisions --json`); `docs/DECISIONS.md` dated notes on `ARC-84`
  (digest printed only when a list is in effect, QDLa-5; `memory:` decoded and passed through, policy ids
  validated by the cognition process at start, QFGb-1), `ARC-27` and `ARC-33`.
- [x] Implementation: the edits above, citing this design and `ARC-84`/`ARC-85`; every F-DLa-2 item fixed so
  code and specification agree (`CLAUDE.md` §2.1 rule 4). *E-DLa-1.*
- [x] Validation: both doc checks. *E-DLa-1.*
- [x] Review: terminology fixed (`Controller Pack`, `Controller`, decision model is "a Controller Pack's
  model", never a new pack type); no rule stated only in a README. *E-DLa-1.*
- **Commit boundary.** Documentation only.

### C2 — `cognition/decision`: trait, `Draw`, parameters

- **Goal.** The extension point exists and is pure by signature.
- **Scope.** `cognition/decision/{Cargo.toml, README.md, clippy.toml, src/lib.rs, src/model.rs,
  src/draw.rs, src/parameters.rs}`; root `Cargo.toml` members + `[workspace.dependencies]`.
- [x] Implementation: `Seed`, `Consult`, `ModelId`, `DecisionModel`, `SeatModel`, `Configured`; `Draw`/`mix`
  copied verbatim (not yet used by the rule controller); `ModelParameters`, `()` impl, `bounded`. *E-DLa-2.*
- [x] Validation: **first**, on the base, capture `Draw` golden values (seed, observer, instant, n → value)
  for 32 tuples by a test against `paced.rs`'s private `Draw` (temporary `#[cfg(test)]` in rule-controller,
  recorded then removed) and freeze them in `cognition/decision/src/draw_tests.rs`; `cargo test -p
  mineworld-decision`; `cargo clippy -p mineworld-decision --all-targets -- -D warnings`. *E-DLa-0, E-DLa-2.*
- [x] Review: no `Cell`/`RefCell`/`Mutex` reachable in trait signatures; object safety of `SeatModel`;
  golden values not produced by the code under test (test rules §25). *E-DLa-2.*
- **Failure cases.** A `parameters!` block decode error must carry position (unit test asserts line/column).

### C3 — `rule` becomes a model, byte-identical

- **Goal.** The rule controller implements the trait without changing a draw.
- **Scope.** `cognition/rule-controller/{Cargo.toml, clippy.toml, src/paced.rs, src/model.rs, src/lib.rs}`.
- [ ] Implementation: `paced.rs` uses `mineworld_decision::{Draw, mix}`; `agenda.rs`/`social.rs`/`offered.rs`
  import paths updated; `Rule` model (SD-DLa-2).
- [ ] Validation: existing `cargo test -p mineworld-rule-controller` unchanged and green; new parity test
  `Rule` via `SeatModel` vs `PacedRuleController` over the paced test fixtures; `Draw` goldens green;
  M-DLa-1 and M-DLa-4 run and reverted.
- [ ] Review: diff of `paced.rs` is imports and the removed `Draw`/`mix` only.

### C4 — The format: `decisions.yaml` decoding, resolution, digest; `cognition/installed`

- **Goal.** A typed, positioned, refused-by-name decision list and the build's model list.
- **Scope.** `cognition/decision/src/{format.rs, read.rs, resolve.rs, digest.rs, models.rs (the
  models! macro)}`; `cognition/installed/{Cargo.toml, README.md, src/lib.rs, tests/installed.rs}`;
  `worldpack/src/{format.rs, read.rs, error.rs}` (the `decisions:` key, SD-DLa-3); `worldpack/tests/`.
- [ ] Implementation: SD-DLa-4 … SD-DLa-9 (including the `memory:` decoder, per-seat memory resolution and
  the YAML→JSON conversion of `parameters`), SD-DLa-12, SD-DLa-14 (digest function only), and the typed
  JSON structs of SD-DLa-15 (serialization only; the subcommand is C5).
- [ ] Validation: table-driven refusal test (AC-DLa-2's list, each asserting path, line, column, substring);
  precedence (AC-DLa-4, fixture with `configure/classes.yaml`); chain rules with a test-only catalog
  including an external entry; digest equality/inequality; `cognition/installed` guard test (ids valid and
  unique; manifest dependencies equal to listed models, as `systems/installed/tests/installed.rs`);
  worldpack: missing file, path escaping the pack, absent key reads as today; M-DLa-2, M-DLa-3 run and
  reverted; `cargo test -p mineworld-decision -p mineworld-installed-models -p mineworld-worldpack`.
- [ ] Review: worldpack parses nothing inside the file; no `HashMap`; error messages name nodes as
  worldpack's do.

### C5 — Hosts: `run`, `server --town`, `validate`, flags, `packs`

- **Goal.** Both hosts drive each seat by its resolved model; the absent case is identical.
- **Scope.** `tools/cli/{Cargo.toml, src/main.rs, src/run.rs, src/serve.rs, src/hosted.rs, src/packs.rs,
  src/decisions.rs (new: read pack/flag, build roster, resolve; the `decisions` subcommand)}`;
  `tools/cli/tests/decisions.rs` (new); `tools/cli/tests/fixtures/decisions/` (golden JSON).
- [ ] Implementation: SD-DLa-11, SD-DLa-13, SD-DLa-14 (header clause), SD-DLa-15 (`mineworld decisions …
  [--json]`); existing `hosted.rs` unit tests adapted to construct `PacedSeat` with `Arc<dyn SeatModel>` (the
  lattice assertions unchanged).
- [ ] Validation (QFGb-1 part): AC-DLa-11 — golden JSON for a fixture pack with `memory:` (class and seat
  entries, a default, an unknown policy id passed through); `"memory": null` for social-cafe; the JSON parses
  with `serde_json` back into the typed structs (round trip on the Rust side; the Python side is FG-b's test).
- [ ] Validation: golden header + summary test for social-cafe seed 7, 30 days, captured on the base
  **before** this commit (test rules §25) and asserted equal; flag refusals; `--seat-model alice=rule` prints
  the clause; `server --town --decisions` starts (existing real-server test harness) and its `town:` line
  carries the digest; `--decisions` without `--town` refused; `cargo test -p mineworld-cli`; M-DLa-6 run and
  reverted.
- [ ] Review: `serve.rs` still never names a model type (only `SeatModel`); `mineworld-server` untouched
  (AC-DLa-8).

### C6 — Template determinism test over installed models

- **Goal.** Every installed model — today `rule`, tomorrow DL-b's and users' — is held to AC-6 automatically.
- **Scope.** `tools/cli/tests/decision_models.rs` (new).
- [ ] Implementation: iterate `Model::AVAILABLE` in-process ids; build `--seat-model` flags for every seat of
  social-cafe; the two equalities of AC-DLa-6 comparing save bytes, not the printed fingerprint.
- [ ] Validation: test green; M-DLa-5 (a hidden `RefCell` counter in a test-only build of `Rule`) turns it
  red, recorded and reverted; wall time recorded (budget ≤ 60 s debug per model).
- [ ] Review: the test reads the catalog, so adding a model needs no test edit.

### C7 — Real-binary AC-DL-1 and close-out

- **Goal.** AC-DLa-1 evidence on the final head; ledger complete.
- **Scope.** This file (§15), `handoff-dl-a.md`.
- [ ] Implementation: `cargo build --release -p mineworld-cli`; `python3 scripts/ci_parity.py record --binary
  target/release/mineworld --profile long --out <scratch>/dl-a.long`; `python3 scripts/ci_parity.py baseline
  check <scratch>/dl-a.long`.
- [ ] Validation: classify PASS/FAIL/INCONCLUSIVE from the check's verdict lines; record wall time.
- [ ] Review: every §8 criterion has evidence or is explicitly open; deviations recorded.
- **Commit boundary.** Documentation only; then push, open the PR, one CI.

---

## 11. Requirements this PR places on other PRs

- **DL-b** adds one `models!` line and one `Cargo.toml` line in `cognition/installed`; nothing else in DL-a's
  code changes. The template test covers it automatically.
- **P6** adds `external "lm"` and turns AC-DL-5 into its own test; P6's "routine policy" becomes the seat's
  fallback chain (step-24 §8.6).
- **FG-b** (R-DLa-FG-1 answered): DL-a accepts FG-b §6's `memory:` shape unchanged, refuses structural
  errors with line and column, and emits the per-seat `MemoryPolicyChoice` JSON through `mineworld decisions
  --json` (SD-DLa-9, SD-DLa-15). FG-b owns semantic validation (`resolve`) at cognition start and refuses an
  unknown `format` version.
- **P6** (R-P6-FG-2) obtains each LM seat's choice from that JSON.
- **WE-b** extends seat entries to Organization seats under QS24-2.

## 12. Edits to planning documents (primary session's, after merge)

`step-24` §11.2 DL-a row → merged with PR number; `overall.md` 2026-10-12 list → DL-a merged; F-DLa-1 noted
under step-24 §5.1.

---

## 13. Execution contract (filled at freeze, 2026-10-12)

The source of every authority line is the primary session's freeze of 2026-10-12 under the operator's
standing instruction, relayed to this planning session. **Implementation starts only on the operator's go.**

```text
PROJECT / PR:            MineWorld mvp0, S24 PR DL-a — the decision list
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-dl-a-decision-list.md
RELATED / BINDING DOCS:  mvp1/step-24-decision-and-memory.md (§§3, 4, 5, 10.2, 11.2, 12, 13, 14.1);
                         overall.md 2026-10-12; docs/DECISIONS.md ARC-23, ARC-25, ARC-27, ARC-33, ARC-53,
                         ARC-61, ARC-64, ARC-84, ARC-85, DEP-10, DEP-12; docs/MODULE_SPEC.md §§4.1, 5, 8.1;
                         pr-fg-b-retention-host.md §6 (the memory: shape and MemoryPolicyChoice, QFGb-1);
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-dl-a, its own, held by one session
BRANCH:                  mvp0/pr-dl-a, created from origin/main
START CONDITION:         DOC-24 merged AND the operator's explicit go. Before both, no implementation
                         session starts
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §3.1, as frozen (including 5a and the decisions subcommand)
FROZEN INVARIANTS:       §3.3 I-DLa-1 … I-DLa-7; SD-DLa-1 … SD-DLa-15 as ruled (§14.1); the JSON shape of
                         SD-DLa-15 (format 1) as agreed with FG-b
SEQUENCE:                C0 … C7
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *; mkdir -p;
                         sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; edits under kernel/, contracts/,
                         systems/, server/, persistence/, cognition/lm-controller/; editing
                         scripts/baselines.txt
MATERIAL STOPS:          AC-DLa-1 FAIL; any edit outside §3.1's paths; a new external dependency; a change to
                         a §8 criterion after it has run once; a change to PacedRuleController's behaviour;
                         a change to the memory: shape or the MemoryPolicyChoice JSON that FG-b has not
                         agreed
PLATFORMS:               Linux, macOS, Windows
VALIDATION BUDGET:       unit/integration/binary tests: unrestricted locally; one long parity record at C7
                         (expected ≤ 20 min); CI: the PR's runs
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-dl-a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized, in a fresh session, once the start
                         condition holds (operator's go)          (primary session, 2026-10-12)
                         semantic commits: authorized             (primary session; working rules §14)
                         branch push: authorized                  (primary session; working rules §21)
                         PR creation / update: authorized         (primary session; working rules §21)
                         CI repair to review readiness: authorized (primary session; working rules §21)
                         merge: never by the implementation session; only after the primary session's
                         review and with explicit operator authorization (working rules §22)
POST-MERGE SYNC OWNER:   primary session: overall.md, step-24; implementation session: this ledger
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Questions (QDLa-n). **[OPERATOR]** marks operator-material ones.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QDLa-1** [OPERATOR] | `server`: should a pack's `decisions.yaml` drive seats even without `--town`, or only with it? | **Only with `--town`**, and `--decisions`/`--seat-model` without `--town` are refused. Today without `--town` no seat is driven in-server; a pack file silently starting controllers would change what `server` does for every existing invocation |
| **QDLa-2** | Ship the `external` model kind and fallback chains now (tested with a test-only catalog), or wait for P6? | **Now.** The chain rules and header wording are this PR's format; P6 then adds one line |
| **QDLa-3** [OPERATOR] | Per-seat parameter overrides (`{ seat: alice, model: motivated, parameters: {…} }`)? | **Not in DL-a.** One parameter block per model per file, as step-24 §4.2 shows. A world wanting two tunings of one model can be revisited with DL-b evidence |
| **QDLa-4** | `mineworld-decision` depending on `mineworld-sdk` to reuse `parameters!` vs. a contracts-only trait crate | **Depend on the SDK**: reusing IL-a's bound checks beats a second implementation; the rule controller already depends on the kernel transitively |
| **QDLa-5** | Header clause only when a list is in effect (keeps baselines) vs. always | **Only when in effect** (§5.8) |
| **QDLa-6** | `memory:` refused until FG-b, or accepted and ignored? | **Refused by name** (I-DLa-5) — *superseded by the QFGb-1 ruling, §14.1* |

### 14.1 Rulings, 2026-10-12 (primary session, operator standing instruction; binding)

| Question | Ruling |
| --- | --- |
| QDLa-1 … QDLa-5 | **Accepted as recommended.** |
| QDLa-6 | Accepted as recommended, then **superseded the same day** by the cross-design ruling on QFGb-1 below: `memory:` is not refused. |
| **QFGb-1** (FG-b, PR #180; cross-design) | **(a).** DL-a's Rust reader decodes `memory:` (FG-b §6's shape), resolves each seat's block (seat > class > default) and emits typed JSON `MemoryPolicyChoice` for the cognition process; FG-b validates the policy name and parameters at cognition start. Applied as SD-DLa-9 (revised), SD-DLa-15 (new), scope 5/5a, AC-DLa-2 (revised), AC-DLa-11 (new), C4/C5, §11, §13. |

Consequence recorded: an unknown memory policy id fails when the cognition process starts, naming file, line
and column, not at `run`'s load (SD-DLa-9).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] | [x] | [x] |
| C1 | (this commit) | [x] | [x] | [x] |
| C2 | (this commit) | [x] | [x] | [x] |
| C3 … C7 | — | [ ] | [ ] | [ ] |

Implementation session: fresh, started 2026-10-13 on the operator's go; worktree
`/Users/yuema137/mineworld-worktrees/impl-dl-a`, branch `mvp0/pr-dl-a`, **base `origin/main @ 6306977`**
(#177). Handoff: [`handoff-dl-a.md`](handoff-dl-a.md).

### Evidence

- **E-DLa-0 (C0).** `DESIGN FROZEN 2026-10-12` header and §13 filled, verified. `grep -n "^## ARC-84"
  docs/DECISIONS.md` → line 7733 (DOC-24 #175 is on the base). `check_doc_headings.py` → "223 numbered
  sections across 31 documents, none duplicated"; `check_decision_ids.py` → "122 decision ids, all distinct"
  (PASS). Authority lines copied from §13 into the handoff unchanged; the operator's kickoff adds one
  obligation (full local workspace test and Python suites before a push that changes tests), recorded as
  such, widening nothing.
- **Goldens frozen on the base, before any code change (test rules §25).** (1) `Draw`: 32 tuples
  `(seed, observer, instant, n, bound) → (below, offset(1 400))` and four `mix` values, printed by a temporary
  `#[cfg(test)]` module placed inside `paced.rs` at `6306977` (so it could reach the private `Draw`), then
  deleted uncommitted; C2 freezes them in `cognition/decision/src/draw_tests.rs`. (2) The base binary's
  `run worlds/social-cafe --headless --seed 7 --days 30` output minus its `wall` line, frozen as
  `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` (header line
  `[mineworld] run Social Café (social-cafe): seats alice, …, wanderer; seed 7; to day 30; pace 900s; in
  memory`; fingerprint `af1927a1e2182bcd`). (3) The base binary's `packs list` (controller line
  `controller-pack mineworld-rule-controller 0.1.0 MIT Yue Ma bundled`) and `validate` outputs, kept in
  scratch for AC-DLa-9.
- **E-DLa-1 (C1).** `docs/MODULE_SPEC.md` §4.1 rewritten for the implemented format: F-DLa-2 items (1)–(5)
  fixed (the trait crate's dependencies; `Consult`; the digest only when a list is in effect; structural
  `memory:` errors at load, policy ids at cognition start; `atus-a3-2025`), "the loader refuses the
  `decisions:` key" removed, the key's path rules, rules 1–8 (refusals, seats/classes, precedence, chains,
  overrides, absent = today, digest, `memory:` pass-through); §5.1 the trait as implemented, `SeatModel`,
  `ModelParameters`/`bounded`, the model crates' `clippy.toml`, `Draw::salted`, out-of-process hosting in
  `run`/`server --town`, `models!` registration, the guard and template tests; §8.1 the synopsis, `server`
  (`--decisions`/`--seat-model`, refused without `--town`, `--agent` wins, `none` seats undriven, the
  `town:` clause), `validate`'s decisions line, `run` (models, header clauses, list never stored),
  `decisions` (the JSON document, format 1), `packs list` (controllers from `cognition/installed`).
  `docs/DECISIONS.md` dated notes on `ARC-27` (stateless seeded controllers generalised to every in-process
  model), `ARC-33` (the installed pattern reused for models) and `ARC-84` (QDLa-5; QFGb-1). Terminology:
  a decision model is "a Controller Pack's model" throughout; no new pack type. Checks:
  `check_doc_headings.py` → 223 sections, none duplicated; `check_decision_ids.py` → 122 ids, all distinct
  (PASS). D-DLa-1 … D-DLa-4 recorded below.
- **E-DLa-2 (C2).** New crate `cognition/decision` (`mineworld-decision`): `src/model.rs` (`Seed`, `Consult`,
  `ModelId` over the system-id rule, `DecisionModel` with `ID`/`PACKAGE`/`Parameters`/`decide`/`step`,
  object-safe `SeatModel` — pinned by a compile-time `const _: Option<&dyn SeatModel>` —, `Configured<M>`),
  `src/draw.rs` (`Draw::new`, `Draw::salted`, `below`, `offset`, `mix`, salt `0x6d69_6e65_776f_726c`),
  `src/parameters.rs` (`ModelParameters`, `()`, `bounded`), `clippy.toml` (workspace entries plus every
  interior-mutability type and atomic), `README.md`; root `Cargo.toml` member (named, with the reason a
  `cognition/*` glob cannot be used) and `[workspace.dependencies]` entry. No new external dependency.
  `cargo test -p mineworld-decision` → 9 passed (PASS): the 32 `Draw` goldens and 4 `mix` goldens from
  E-DLa-0, a salted stream independent of the rule's, and parameter decoding (defaults; out-of-bound
  refused at its key `(2, 1)`; wrong type keeps the decoder's `(2, 11)`; unknown key at `(3, 1)` listing
  the known keys; twice-written key; `()` refuses any key at `(2, 3)`). `cargo clippy -p mineworld-decision
  --all-targets -- -D warnings` clean (PASS). Review: the trait signatures name `&self`, contract types and
  the model's own `Parameters` only; the goldens are literals printed by the base's code.
  *Finding F-DLa-3 (serde-saphyr positions):* `serde_saphyr::Error::custom` is locationless by design
  (`de/error.rs`: "keep custom errors locationless"); the decoder attaches the innermost value's position to
  a locationless error raised *inside* a value, and serde's own constructors (`invalid_value`,
  `unknown_field`, …) receive the current key's position. The SDK's `parameters!` refuses a bound with
  `custom`, after the value is consumed, so `bounded` restates exactly that refusal (recognised by the SDK's
  "'<field>' is " prefix) as `invalid_value`, which is positioned at the field's key; a type error passes
  unchanged with the decoder's own position. `()` refuses with `unknown_field` for the same reason. The
  decision-list reader (C4) therefore positions its own semantic refusals from `serde_saphyr::Spanned`
  values rather than from `custom` errors.

### Mutations

(none run yet)

### Deviations and findings

- **F-DLa-1** (planning): step-24 §5.1's "the rule controller depends on contracts only" is inaccurate (§2.1);
  no design change.
- **F-DLa-2** (planning, at freeze, after rebasing on `origin/main @ d14f05e`): DOC-24 (#175) wrote
  `MODULE_SPEC.md` §4.1/§5 from step-24's sketch. Five statements differ from this frozen design and are
  amended by C1: (1) `mineworld-decision` "depends on `mineworld-contracts` only" → also `mineworld-sdk`
  (QDLa-4); (2) `decide(&self, parameters, seed, observation)` / `step(&self, parameters, observation)` →
  both take `Consult { seed, pace, observation }` (SD-DLa-1); (3) "`run` prints the digest … in its header"
  → only when a decision list is in effect (QDLa-5, SD-DLa-14); (4) "an unknown … policy … is refused at
  load" → structural `memory:` errors at load, unknown policy ids at cognition start (QFGb-1, SD-DLa-9);
  (5) the example's `curves: atus-2003-2023` → `atus-a3-2025` (QDLb-3; DL-b's C7 owns the `motivated`
  paragraph, DL-a fixes the example). Also: "the loader refuses the `decisions:` key until that PR merges"
  is removed by C1.

Implementation-time deviations (all bounded; none touches a frozen invariant, a public contract under
`contracts/`, an ownership boundary, a dependency or scope):

- **D-DLa-1 (C1, C2) — `DecisionModel` gains `const PACKAGE: Package`.** *Previous assumption:* §5.1's trait has
  `ID`, `Parameters`, `decide`, `step`; §5.7 has `packs.rs` derive its controller list as
  "`Model::packages()` from `cognition/installed`". *Audit:* nothing in §5.1's trait carries a package, and a
  `models!` line names only a type, so the catalog cannot know a model's package (`ARC-53`) unless the type
  states it. *Resolution:* the trait states it, as `SystemPack::PACKAGE` does (`mineworld_sdk::Package`, a
  re-export the trait crate already reaches through the SDK, QDLa-4). *Consequence:* `Rule::PACKAGE` is
  `mineworld_rule_controller::PACKAGE`, so AC-DLa-9's list is unchanged. Specified in `MODULE_SPEC.md` §5.1.
- **D-DLa-2 (C1, C4) — an external model takes no `parameters` in DL-a.** step-24 §5.3 says an external
  entry carries "only `fallback` and model-specific parameters"; the Rust side cannot type an external
  model's parameters, and I-DLa-5 forbids accepting what is not understood. DL-a refuses `parameters` on an
  external entry by name; the PR that installs the first external model (P6) defines how they are carried.
- **D-DLa-3 (C1, C4) — "at most 4 links" counts `fallback:` steps.** A chain `m0 → m1 → … → mk` has `k`
  links; `k ≤ 4` is accepted, `k = 5` refused (AC-DLa-2's "chain of 5" is tested as five links).
- **D-DLa-4 (C1, C4) — `none` is reserved.** `fallback: none` ends a chain, so no installed model may have
  the id `none`; the `cognition/installed` guard test refuses it. In `mineworld decisions --json` a seat
  whose chain ends at `none` has `"model": { "id": null, … }`.
