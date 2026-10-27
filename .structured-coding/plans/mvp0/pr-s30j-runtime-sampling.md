# PR S30-j — Runtime sampling of new people: a newcomer drawn against the people already there

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-26), DRAFT
Author:                 S30-j PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30j,
                        branch docs/pr-design-s30j, planning base origin/main @ 0b2faf2f (#382)
Approved by / evidence: none yet — §15 asks three questions; nothing here is frozen
Implementation base:    main (origin/main at C0); C0 records the commit and the merged prerequisites
Start condition:        S30-d merged (met: #293, merge commit 2af65e6d); this design frozen (NOT met);
                        §15 ruled (NOT met); the primary's dispatch (NOT met)
Execution contract:     §13
Implementer:            Opus-class session (step-30 §13.1, the S30-j row); C5 may go to a Haiku-class sub-agent
                        from this commit plan, reviewed by the session
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15;
amended) §§3.3 (F-S30-10), 8.3–8.6, 11 (persistence and replay), 12 (I-S30-6, I-S30-7, I-S30-8), 13.1 (the S30-j row),
13.3 (S30-j starts once S30-d has merged; release gate), 13.4 (AC-S30-5, AC-S30-7, AC-S30-8, AC-S30-12) · **Specified
by:** `docs/MODULE_SPEC.md` §4.1 "The world's configuration", "Population profiles", "The lock file", "Persistence"
(line 1758); `docs/decisions/ARC-123.md` item 6 ("the sampler runs only at genesis") · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (MERGED, #280),
[`pr-s30d-sampler.md`](pr-s30d-sampler.md) (MERGED, #293; its R-S30j-d-1),
[`pr-s30e-builtin-catalogue.md`](pr-s30e-builtin-catalogue.md) (MERGED, #300) · **Parent:** [`overall.md`](overall.md)
· **Followed by:** whichever System first brings real newcomers (a migration System in S21, a happening's arrival);
it calls this PR's hook unchanged · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 2, 5, 7, 8, 9, 10, 11, 12, 16);
`docs/CORE_CONCEPTS.md` INV-7, INV-12, INV-15; `docs/DECISIONS.md` `ARC-25` (replay re-executes), `ARC-26` (emitting
another pack's vocabulary), `ARC-61` (configuration), `ARC-120` … `ARC-124`; step-18 QPL-11 and `overall.md` line 776
(no runtime entity creation by a System); the lane-lead charter's rule "no world-specific hard-coding".

**Decision ids.** One new record proposed: **`ARC-128`** — runtime sampling of newcomers (§5, Q-S30j-3). A dated
note under `ARC-123` item 6. `ARC-127`, `ARC-129` and `DEP-69` stay unused.

No model is called. No reference world's bytes move. Nothing under `kernel/` or `contracts/` changes.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-j row).** "`sample_newcomer(profile, current population, causing EventId)` in the model
  crate, stratified against the current population's realised counts; an owner hook so the System that creates a
  Person seeds its attributes in the same reduction; a fixture creator (F-S30-10)."
- **Integration checkpoint (same row).** "a newcomer's values reproduce on replay and across kill and resume; 1,000
  synthetic arrivals keep the world within AC-S30-5's tolerance."
- **Adversarial criteria (same row).** "re-sampling on resume is detected; a newcomer created without its seed is
  refused."
- **F-S30-10 (step-30 §3.3).** "`World::create_entity` exists (`kernel/src/world.rs:572`), but no System on `main`
  creates a Person at runtime → S30-j proves runtime sampling with a fixture creator; a real newcomer source (S21
  migration, a happening's arrival) is that System's own PR and uses S30-j's entry."
- **AC-S30-5.** "A synthetic 1,000-person resolution matches every declared marginal within 0.5 pp and every
  conditional within 2 pp for groups of ≥ 50."
- **R-S30j-d-1 (S30-d §11).** "A newcomer is resolved with the same words of randomness (§5.3), its group's realised
  counts taken from the current population."
- **I-S30-6.** The sampler is a pure function of (catalogue, profile, lock, people files without names, seed, salt),
  identical on three platforms. **I-S30-7.** Authored and ruled values are never re-drawn. **I-S30-8.** `kernel/` and
  `contracts/` contain no attribute vocabulary (AC-S30-12).
- **The brief (2026-10-26).** Keep the kernel ignorant of attributes. No world-specific hard-coding: the mechanism is
  generic data, and every acceptance criterion names a second world or a generated variant.
- **R-LOAD-1.** Locally only the targeted test files of a commit and `cargo clippy` on touched crates; never `cargo
  test --workspace`; CI on the exact head is the gate. Heavy local jobs run through
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

---

## 1. Goal

Let a Person who **joins a running world** get attributes drawn from the world's profile, so that:

1. **The draw looks at who is already there.** Within the newcomer's parent group, the value the population lacks
   most is the likeliest one. Over many arrivals the world keeps its profile's shape (AC-S30-5).
2. **The draw is world state, not a tool's guess.** It happens inside the dispatch that admits the newcomer. Replay
   re-executes it and compares it byte for byte. Kill and resume reproduce it. Nothing re-samples.
3. **Only the owner draws.** The System that brings a newcomer asks; `attributes` samples and states the values. The
   creator never writes `traits` and never chooses a value.
4. **The kernel learns nothing.** No kernel or contract change.

---

## 2. Scope

### 2.1 In scope

1. **The model crate** (`authoring/attributes/`):
   - `sample_newcomer` (`sample/newcomer.rs`, new), sharing the genesis sampler's helpers (moved, unchanged in
     behaviour, to `sample/values.rs`);
   - the configuration block `newcomers:` (`config.rs`) and its checks (`context.rs`);
   - the runtime newcomer context carried by `attributes-composed` (`facts.rs`), and the Process that holds it;
   - the fact `newcomer-admitted` and the checked constructor `admit_newcomer` (`newcomer.rs`, new);
   - `uses: { admits: true }` in an `attributes/` file (`declaration.rs`, `catalogue.rs`);
   - `Deserialize` for the frozen profile types, so the runtime context round-trips (`profile/mod.rs`).
2. **The `attributes` pack** (`systems/attributes/`): genesis skips deferred people; the newcomer Process starts from
   `attributes-composed`; `newcomer-admitted` is checked, sampled and answered with `traits-stated` and
   `attributes-handed` in the same dispatch; a proposal about a person who has not arrived is refused.
3. **The CLI** (`tools/cli/src/attributes_resolve.rs`): `resolve` lists deferred people as `deferred`, with no values.
4. **Fixtures and tests:** one fixture world with deferred people; a test-only fixture creator; the 1,000-arrival
   fidelity test on two profiles; the replay and kill-and-resume program; the adversarial plants.
5. **Specification sync:** `docs/MODULE_SPEC.md` §4.1, `docs/decisions/ARC-128.md` (new), a dated note under
   `ARC-123`, the generated reference for `configure/attributes.yaml`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| A System creating a new entity during a dispatch (a kernel change) | QPL-11's own kernel design, if the operator wants it (Q-S30j-1) |
| A real newcomer source: migration, a happening's arrival, births | that System's own PR; it calls `admit_newcomer` |
| Departures: removing a person from the counts | the departure System's PR (R-j-3) |
| Owner-held attributes stratified against the owner's state | not done: an owner's state is the owner's (D-j-4) |
| Any reference world enabling `newcomers:` | S30-g or a later world PR |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, or any System other than `attributes` | never |

### 2.3 Dependencies

- **Hard, met:** S30-c (#280), S30-d (#293), S30-e (#300) merged; S30-f (#373) merged. C0 re-checks.
- **None open.** S30-g, S30-h, S30-i and S30-k touch other files. If S30-g lands first and enables `attributes` in a
  reference world, this PR still moves no byte of it: the `newcomers:` block is absent there (I-j-1).

---

## 3. Audit (`origin/main @ 0b2faf2f`, 2026-10-26)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-j-1 | `World::create_entity(&mut self, …)` and `create_authored_entity` are methods of `World`. A running System gets a `WorldView`, whose writes are components, relations, processes and deferrals — no entity creation | `kernel/src/world.rs:572-590`; `kernel/src/view.rs:253-440` | **no System can create a Person at run time** (F-S30j-1) |
| A-j-2 | The only callers outside tests are the pack loader (`worldpack/src/load.rs:259-288`) and the `bodies` resolver's own test world (`systems/bodies/src/resolve.rs:403-408`) | `grep create_entity` | F-S30-10 still holds: no runtime newcomer source exists |
| A-j-3 | Runtime entity creation by a System is a recorded open question: "a kernel change", "Runtime entity creation is its own kernel design, if wanted" (QPL-11, accepted as "not in this step"). S21 builds a fixed vehicle fleet for the same reason (INV-RT-10) | `step-18-physics-list.md:753-760, 1195`; `overall.md:776`; `mvp1/step-21-regions-travel.md:145, 618` | S30-j does not decide QPL-11 (Q-S30j-1) |
| A-j-4 | Persistence journals three inputs: genesis, dispatch, advance. Replay re-executes each through the same `World` call and compares the answer and every fact **as bytes**; any difference is `ReplayDiverged` naming the revision | `persistence/src/replay.rs:1-6, 47-118` | a draw made inside a dispatch is reproduced and checked for free |
| A-j-5 | `PersistentWorld::resume` restores the newest snapshot, then re-executes every later journal row | `persistence/src/world.rs:175-215` | a newcomer drawn after the last snapshot is re-drawn and compared; one before it is restored from `Traits` |
| A-j-6 | Hosts compare a save's genesis context facts with the re-assembled ones: a change is `ConfigurationDrift` naming the system | `worldpack/src/configure.rs:518-615`; S30-d §5.7 | whatever the runtime sampler reads must be inside a context fact |
| A-j-7 | Reduction runs in **generations**: every subscriber reacts to every fact of one generation, then the facts they emitted form the next | `kernel/src/dispatch.rs:604-660` | two admissions in one emission list are sampled against the same population (D-j-6) |
| A-j-8 | An `Err` out of `react` is "a bug in a system, not a rejected request"; reduction is not transactional. A refusal of a request belongs in `validate` | `kernel/src/dispatch.rs:55-66` | the creator runs a checked constructor in `validate`; `attributes` re-checks in `react` (§5.6) |
| A-j-9 | The genesis sampler: `resolve(catalogue, profile, lock, words, members)`; `Words { s }` with `order`, `tie`, `draw`; `Member { key, tags, authored }`; categorical groups by largest remainder; numeric draws at `(2·slot + 1) / 2u`; roles after their target; defaults; `covers: adults` at `WorldTime::EPOCH` | `authoring/attributes/src/sample/resolve.rs:108-143, 215-335, 419-587, 670-759, 761-799` | `sample_newcomer` reuses `Words`, `Member`, the weights, the numeric shapes and the roles |
| A-j-10 | `uniform_age_within_band` returns `birth = −age · 86 400` seconds: a birth relative to the epoch | `resolve.rs:648-665` | a newcomer's birth is relative to the instant of arrival (D-j-5) |
| A-j-11 | `attributes` resolves every Person before seeding (`resolve_subjects`) and states `traits-stated` only when a person has a plain trait, then one `attributes-handed` per owner | `systems/attributes/src/section.rs:61-161` | deferred people are left out of `members` and get no fact at genesis |
| A-j-12 | `attributes-composed` carries the catalogue's projection and the population's **identity** (profile id, version, digest; seed; salt; lock digest) — not the profile itself. The catalogue runs as the `attributes-catalogue` Process | `authoring/attributes/src/facts.rs:30-48, 147-187`; `systems/attributes/src/data.rs:92-117`; `system.rs:121-131` | a running world has no profile to sample from today (F-S30j-2) |
| A-j-13 | The profile types derive `Serialize` (the digest is FNV-1a 64 of canonical JSON) but not `Deserialize` | `authoring/attributes/src/profile/mod.rs:152-319` | the runtime context needs a round trip (C2) |
| A-j-14 | A proposal is checked by `check_proposal` in the proposer's `validate` ("an ordinary rejection of its own request") and again by `attributes` in `react`; the cause must be an earlier fact | `facts.rs:189-262`; `system.rs:162-205` | the admission follows the same pattern exactly |
| A-j-15 | `Uses { reads, proposes }` is decoded from an `attributes/` file's `uses:` block and kept per System in the catalogue | `catalogue.rs:247-254`; `declaration.rs:485-506` | `admits: true` is one more field of the same block |
| A-j-16 | `configure/attributes.yaml` is `format, profile?, seed?, salt, lock?, backend?, mask?`, `deny_unknown_fields`, with a JSON schema for the generated reference | `authoring/attributes/src/config.rs` | `newcomers:` is one optional block |
| A-j-17 | The fidelity test builds the fixture catalogue and profile from `tools/cli/tests/fixtures/attributes-sampled/` and checks AC-S30-5 and AC-S30-6 on synthetic populations | `authoring/attributes/tests/fidelity.rs:1-30` | the 1,000-arrival test sits beside it |
| A-j-18 | A kernel world composed by hand with the real `attributes` pack and a test-only System is the established acceptance pattern | `tests/acceptance/tests/attributes_seam.rs:1-60` | the fixture creator lives there, in a test, never in a library |
| A-j-19 | The SIGKILL program: control, victim killed at kill points, survivor resumes in a new process, saves compared byte for byte, `verify()` from genesis; `harness = false` | `tests/acceptance/tests/arrival_resolvers_resume.rs:1-30`; `tests/acceptance/Cargo.toml:11-17`; `persistence/tests/kill_and_resume.rs` | S30-j's kill-and-resume program copies it |
| A-j-20 | `WorldPack::assemble` returns the world with systems installed and entities created, genesis facts unapplied; `world` is a public field | `worldpack/src/load.rs:216-306` | the program loads the fixture world by the real loader and installs the fixture creator before genesis |
| A-j-21 | `MODULE_SPEC` §4.1 "Persistence": "the sampler runs only at genesis, so a resumed world never re-samples"; `ARC-123` item 6 says the same | `docs/MODULE_SPEC.md:1758`; `docs/decisions/ARC-123.md:16` | both gain a dated note: the sampler also runs at an admission, inside a dispatch, and still never at resume |
| A-j-22 | No reference world enables `attributes` | `grep attributes worlds/*/world.yaml` (empty) | I-j-1 is cheap to hold |

### 3.2 Findings

- **F-S30j-1. No System can create a Person at run time** (A-j-1 … A-j-3). F-S30-10 read `World::create_entity` as
  available to a fixture creator. It is a `&mut World` method used at assembly only, and making it a System's
  capability is QPL-11's open kernel question. Two honest routes exist (Q-S30j-1):
  - **(a) Deferred people** (recommended). A world declares some people now and marks them to arrive later. They exist
    from assembly, but `attributes` resolves nothing for them at genesis. A System admits one at run time, and
    `attributes` samples it then. No kernel change.
  - **(b) Runtime entity creation first.** A kernel PR lets a System create an entity during a dispatch; S30-j waits
    for it.
  The hook does not depend on the route. It takes an existing `PersonId` that has never been seeded. A System built
  on (b) later calls the same hook unchanged.
- **F-S30j-2. A running world cannot see its profile** (A-j-12, A-j-13). Only the profile's identity is in world state.
  A draw at run time must read the profile from world state, or replay and resume would depend on files read at
  host start. Decided (D-j-2): when `newcomers:` is configured, `attributes-composed` also carries a newcomer context —
  the profile's runtime form, the seed, the salt and each deferred person's fixed values — and `attributes` keeps it in
  a Process. The drift check then covers it (A-j-6). Without the block nothing changes, byte for byte.
- **F-S30j-3. "The creator seeds in the same reduction" cannot mean the creator writes values.** `traits` is
  `attributes`' component (INV-7). Decided (D-j-1): the creator emits `attributes`' own request fact,
  `newcomer-admitted`, built by `attributes`' checked constructor, in its resolve; `attributes` reacts in the same
  dispatch with the drawn values as `traits-stated` and `attributes-handed`. One dispatch, one journal revision, one
  byte-compared replay (A-j-4).
- **F-S30j-4. "Created without its seed is refused" needs a place to refuse.** Under route (a):
  - the creator's `validate` runs `admit_newcomer` — a person not deferred, already arrived, or not drawable is an
    ordinary rejection with a code;
  - `attributes` re-checks in `react`, so a creator that skips the check stops the world with a named fault;
  - a person who has not arrived has no attributes: `attributes` refuses a proposal about them (`not-arrived`), and
    their observation carries no `traits`;
  - a creator that never calls the hook at all is invisible to `attributes`, because nothing tells it. The scenario
    test catches that plant (X-j-5). Only route (b), with a kernel-recorded creation, could make the world itself
    refuse it. Q-S30j-1 states this trade.
- **F-S30j-5. "Current population" must be read without copying owners' state.** Plain traits are `attributes`' own
  `Traits`. An owner's attribute (for example `economy.wealth-band`) is the owner's state, readable only through the
  owner's static projection, which `attributes` cannot call. Decided (D-j-4): plain traits are stratified against the
  current `Traits`; an owner's attribute is drawn independently from the profile row, as `stratify: false` draws. No
  owner state is copied (`ARC-120` "a view, never a copy").
- **F-S30j-6. Realised counts change with time.** A derived parent such as `core.age-band` moves as people age. The
  step asks for "the current population's realised counts", so the newcomer is grouped, and the population counted,
  by values **at the instant of arrival** (Q-S30j-2). Ageing with no departures makes the oldest band grow; newcomers
  cannot pull it down. That is the departure System's concern (R-j-3).
- **F-S30j-7. Numeric attributes need a count to stratify against.** The genesis path spreads quantiles
  `(i + ½) / n` over a known `n`. A newcomer arrives alone. Decided (D-j-3): each numeric stratum is cut into ten
  equal-probability bins at the profile's own deciles; the newcomer's bin is drawn by the same deficit rule as a
  category, then a quantile is drawn inside the bin. Integer arithmetic only.
- **F-S30j-8. `resolve.rs` is 799 lines.** The newcomer path goes in its own file, and the shared helpers move to
  `sample/values.rs` without a change of behaviour, held by S30-d's golden (`attributes-sampled.resolve.json`).

### 3.3 What does not exist

No runtime draw. No newcomer fact, Process or configuration. No fixture with deferred people. No test that resumes a
world after an admission.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Reused in the tree:** S30-d's sampler pieces (`Words`, `largest_remainder`, the integer quantile table, the
  numeric shapes, the role rule), S30-c's proposal pattern (check in `validate`, re-check in `react`), the catalogue
  Process pattern, the persistence kill-and-resume program, the pack loader.
- **External, considered and declined** (`DEP-67` already declined pgmpy, pomegranate, PopulationSim and MatrAIx's
  sampler for genesis; the same reasons hold here — floats, Python, a different problem):
  - *Sequential apportionment* is a known problem with known answers: the chairman assignment problem (Tijdeman 1980)
    and the quota method (Balinski and Young 1975) choose, one seat at a time, the party furthest below its quota.
    The rule here (§5.4) is that idea with a random choice among the parties below quota, weighted by how far below
    they are. It is twenty lines of integer code; no library offers it in Rust with integer arithmetic. Recorded in
    `ARC-128`, not adopted as a dependency.
  - *Low-discrepancy sequences* (van der Corput, Sobol) for numeric arrivals: declined. They need the arrival index
    of a stratum, which a deferred or departing population makes ambiguous; deciles counted from the current
    population need no index (D-j-3).
- **Not forced:** the kernel's pipeline is used as it is; nothing adopts another execution model.

---

## 5. Design

### 5.1 The world's configuration (`configure/attributes.yaml`)

```yaml
format: 1
profile: test-town
seed: fixture-pop-v1
lock: cast
newcomers:                      # optional; absent = no runtime sampling, and every byte as before
  later: [nadia, omar, pia]     # declared people whose attributes are resolved when they arrive
```

- `newcomers.later` lists person keys the world declares (`population:`). Each must be a Person of this world
  (`UnknownPerson`), listed once (`DuplicateNewcomer`).
- A deferred person may have an `attributes:` section and lock entries. Those are fixed values and restrictions,
  applied at arrival exactly as at genesis: authored, then lock (I-S30-7).
- `newcomers:` without `profile:` is allowed: arrivals then get fixed values and defaults only.
- The block is generic. It names no attribute, tag or world.

### 5.2 What the running world holds

- `AttributesComposed` gains `newcomers: Option<NewcomerContext>`, serialized only when present:

```rust
pub struct NewcomerContext {
    pub profile: Option<Profile>,                    // the runtime form; digest unchanged by the round trip
    pub seed: String,
    pub salt: u32,
    pub deferred: BTreeMap<EntityKey, Fixed>,        // per deferred person: authored + locked values, restrictions
}
pub struct Fixed {
    pub values: BTreeMap<QualifiedId, (AttributeValue, Pinned)>,   // Pinned: authored | ruled | locked
    pub restrictions: BTreeMap<QualifiedId, Restriction>,
}
```

- `attributes`, reacting to `attributes-composed` with `newcomers` present, starts a second Process,
  `attributes-newcomers` (uninterruptible, like the catalogue's), whose state is the context plus
  `arrived: BTreeSet<EntityKey>`, empty at genesis.
- The drift check compares `attributes-composed` whole (A-j-6). A changed profile, seed, salt, lock or list between
  kill and resume is `ConfigurationDrift` naming `attributes`.

### 5.3 Genesis

- `resolve_subjects` leaves deferred people out of `members`. Genesis stratification is over the people present.
- A deferred person gets no `traits-stated` and no `attributes-handed` at genesis.
- Every other person is resolved exactly as on `main`. S30-d's golden is unchanged (AC-j-7).

### 5.4 The draw: `sample_newcomer`

```rust
/// One newcomer's values, drawn against the people already resolved.
pub fn sample_newcomer(
    catalogue: &Catalogue,
    profile: Option<&Profile>,
    words: Words,                 // Words::new(seed, salt): the genesis words (R-S30j-d-1)
    newcomer: Member<'_>,         // key, tags, fixed values and restrictions — never a name
    population: &[Resident<'_>],  // every arrived or genesis person: key and stored plain values
    at: WorldTime,                // the instant of arrival
    because: EventId,             // the admission's cause; named in every refusal
) -> Result<Arrival, ProfileError>;

pub struct Resident<'a> { pub key: &'a str, pub values: &'a BTreeMap<QualifiedId, AttributeValue> }
pub struct Arrival { pub values: BTreeMap<QualifiedId, (AttributeValue, Origin)> }
```

The algorithm, per profiled attribute X in the profile's topological order (ties by id, as genesis):

1. **Fixed first.** A value the newcomer's fixed set holds is kept, never drawn (I-S30-7).
2. **Group.** The newcomer's parent values, each resolved or derived at `at`. The population's members whose parent
   values, derived at `at`, are the same form the group. Let `n` be how many of them hold a value of X.
3. **Categorical, `stratify: true`, plain trait.** For each value v of X:
   `deficit(v) = max(0, (n + 1) · w(v) − 1 000 000 · c(v))`, where `w` is the row's integer weight in parts per
   million (masks and adjustments applied, as genesis) and `c(v)` is how many in the group hold v.
   - The deficits of all values sum to exactly 1 000 000 when none is negative; a value with `w(v) = 0` never has a
     deficit, so a mask still holds (AC-S30-6).
   - A restriction `allowed:` zeroes the values outside it.
   - The value is chosen by `ppm(words.draw(X, key, 0))` against the cumulative deficits. If every allowed deficit is
     zero, the choice falls back to the weights within the restriction; if those are zero too,
     `Unsatisfiable { person, attribute }`.
   - Why it holds the shape: a value is chosen only while it is below its share, so no value ever exceeds its share
     by a whole person. Over 1,000 arrivals the marginals stay within AC-S30-5 (AC-j-2 measures it).
4. **Numeric, `stratify: true`, plain trait (D-j-3).** The stratum's deciles `e₁ … e₉` are the values at quantiles
   `k / 10` through S30-d's integer shapes. Each member's value falls in one of ten bins. The bin is chosen by step
   3's rule with `w = 100 000` per bin. The quantile inside it is
   `(bin · 100 000 + ppm(words.draw(X, key, 1)) div 10) / 1 000 000`, mapped through the same shape. A `between:`
   restriction draws uniformly in its bounds, as genesis (D-d-2).
5. **`stratify: false`, or an owner's attribute (D-j-4).** One independent draw from the row's weights with
   `words.draw(X, key, 0)`, exactly as genesis draws it.
6. **`uniform_age_within_band`** gives an age as genesis does; the birth is `at − age · 86 400` (D-j-5).
7. **Roles** for X apply right after it, as genesis, from the newcomer's tags (the entity's registry tags) and earlier
   values.
8. **Defaults** fill what is still absent. Derived attributes are never stored.
9. **`covers: adults`** is checked at `at`: a newcomer under 18 by any age-derived attribute is `NotCovered`.

- **Pure.** The output depends only on the arguments. The words are the genesis words keyed by the person's key
  (R-S30j-d-1); `because` is not mixed in, so a newcomer's values do not move when unrelated earlier history moves.
- **Integer only.** The crate keeps `#![deny(clippy::float_arithmetic)]`.
- **Origins** are `sampled { stratum, draw }` with `draw` = the group's `n`, or `authored`/`ruled`/`locked`/`role`/
  `default`. They are tool output and never stated.

### 5.5 The admission: `newcomer-admitted` and `admit_newcomer`

```rust
/// A System's request that `attributes` resolve a deferred person now (`attributes`' vocabulary).
pub struct NewcomerAdmitted { pub person: PersonId, pub because: EventId }   // event type "newcomer-admitted"

/// The check every admitter runs in `validate` (every code below except `unknown-cause`).
pub fn check_newcomer(
    world: &WorldRead<'_>, admitter: &SystemId, person: PersonId,
) -> Result<(), Rejection>;

/// The same check, then the emission the admitter returns, citing its cause.
pub fn admit_newcomer(
    world: &WorldRead<'_>, admitter: &SystemId, person: PersonId, because: EventId,
) -> Result<Emission, Rejection>;
```

Refusal codes, each with a detail naming the person:

| Code | When |
| --- | --- |
| `attributes-uncomposed` | the world has no catalogue |
| `no-newcomers` | the world configures no `newcomers:` |
| `not-an-admitter` | the admitter's `attributes/` file does not declare `uses: { admits: true }` |
| `not-a-newcomer` | the person is not in `newcomers.later` |
| `already-arrived` | the person is in `arrived` |
| `newcomer-unsatisfiable` | the dry-run draw fails (`Unsatisfiable`, `EmptySupport`, `NotCovered`, …), with its message |

- The emission's audience is `SystemInternal`, like `attributes-handed`.
- An admitter declares that it emits `newcomer-admitted` and depends on `attributes` (`ARC-26`).

### 5.6 The reaction (`systems/attributes`)

On `newcomer-admitted`, in `react`:

1. Re-run every check of §5.5, and `because < this fact's id` (`unknown-cause`). A failure is
   `FactRefusedByOwner` naming the code — a creator that skipped its own check stops the world (A-j-8).
2. Build `population` from every Person whose key is not deferred-and-unarrived and who holds `Traits` (plain stored
   values), in entity-id order.
3. `sample_newcomer(…, at = world.at(), because)`.
4. Add the key to `arrived` (`set_process_state`), so a second admission of the same person in the same generation is
   refused (A-j-7).
5. Emit, in this order: `traits-stated { person, values }` **always**, even with no plain value, so the arrival is on
   the record; then one `attributes-handed` per owner with values. Audiences as genesis. The next generation reduces
   them through the existing paths.

Also:

- A proposal (`attribute-change-proposed`) about a deferred person not yet arrived is refused `not-arrived`, in
  `check_proposal`'s world form (`admit_proposal`) and in the re-check.
- Disclosure needs no change: a person with no `Traits` discloses nothing.

### 5.7 The fixture creator (test-only)

`tests/acceptance/tests/newcomers/creator.rs` (a module of the test programs, never a library; A-j-18):

- System `test-arrivals`; `attributes/arrivals.yaml` shipped beside the fixture world declares
  `uses: { admits: true }`.
- Action `arrive { person }`:
  - `validate` runs `check_newcomer`, so every refusal of §5.5 is an ordinary rejection;
  - `resolve` emits the creator's own fact `arrived { person }`;
  - the creator subscribes to its own `arrived` and, in `react`, emits the `newcomer-admitted` emission that
    `admit_newcomer` returns, with `because` = the `arrived` fact's id.
  The cause must be a recorded fact, and `arrived`'s id is known only once it is recorded; so the request follows one
  generation later, in the same dispatch.
- It places nobody and writes nothing else. Deferred people are present from genesis with no attributes (§5.8).

### 5.8 The fixture world (`tools/cli/tests/fixtures/attributes-newcomers/`)

- S30-d's `attributes-sampled` catalogue and `test-town` profile, copied (that world and its golden stay untouched).
- Eight people present at genesis; four deferred: one with an authored value (`nadia`), one with a lock entry
  (`omar`), one with an `allowed:` restriction (`pia`), one with nothing (`quin`).
- `systems: [presence, naming, attributes]`; `configure/attributes.yaml` as §5.1 with the four keys.
- A second configuration file beside the world, `attributes-newcomers.drift.yaml`, with a different salt, for the
  drift case (as S30-f kept its masked variant beside its world).

### 5.9 The tests

| Test | Kind | What it shows |
| --- | --- | --- |
| `authoring/attributes/tests/newcomers.rs` | crate | §5.4 on hand-built inputs: fixed kept; mask held; restriction respected; `Unsatisfiable`; birth relative to `at`; `NotCovered` at `at`; the input type has no name field, and the same key with the same population gives the same values whatever `because` is |
| `authoring/attributes/tests/fidelity.rs` (new cases) | crate | 1,000 arrivals into the fixture's genesis population, on `test-town` **and** on `entities/populations-us`' San Diego County profile with a generated population (the second world): AC-j-2 |
| `systems/attributes/tests/newcomers.rs` | pack | genesis skips deferred people; an admission states values in the same dispatch; every refusal code of §5.5; `not-arrived` |
| `tests/acceptance/tests/newcomers_resume.rs` (`harness = false`) | real lifecycle | the fixture world loaded by the real loader, the creator installed before genesis, `PersistentWorld` on SQLite; control, victims killed at three points around the admissions, survivors resumed in new processes; saves compared byte for byte; `verify()` from genesis |
| `tools/cli/tests/attributes_resolve.rs` (new case) | binary | `resolve --json` on the fixture lists the four as `deferred` with no values |

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-j-1 (I-S30-10) | A world without `newcomers:` is byte-identical: its `attributes-composed`, genesis facts, observations and the S30-d golden | AC-j-7; existing tests unchanged |
| I-j-2 | Only `attributes` draws and states a newcomer's values; the creator writes no `traits` and chooses no value | review; `OwnedBy<AttributesSystem> for Traits` unchanged; X-j-4 |
| I-j-3 (I-S30-6) | `sample_newcomer` is a pure, integer function of its arguments; no name, no run seed, no wall clock | AC-j-3; X-j-2 |
| I-j-4 (I-S30-7) | A deferred person's authored, ruled and locked values are never drawn | AC-j-1 |
| I-j-5 | Nothing re-samples at resume: a newcomer's values come from the log or the snapshot, and replay checks them as bytes | AC-j-4; X-j-1 |
| I-j-6 (I-S30-8) | `kernel/` and `contracts/` gain nothing | AC-S30-12's grep; `git diff --stat` empty there |
| I-j-7 | No owner state is copied into `attributes` | review of §5.6 step 2 (plain `Traits` only) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-cli`, `mineworld-acceptance`; the crate's `float_arithmetic` deny | per commit; CI `fast` |
| Unit / crate | the draw's rules; the deficit sum; the refusal codes; the configuration's checks; the profile round trip (digest equal) | `authoring/attributes/src/*/tests.rs`; `authoring/attributes/tests/newcomers.rs` |
| Statistical | AC-S30-5 on 1,000 arrivals, two profiles | `fidelity.rs` |
| Pack | genesis skip, same-dispatch seeding, `not-arrived` | `systems/attributes/tests/newcomers.rs` |
| Real lifecycle (Gate 2) | replay and SIGKILL-and-resume reproduce the newcomers byte for byte; drift refused | `newcomers_resume.rs`: run by CI's `acceptance` layer on the PR head, bounded (< 2 min) |
| Golden | S30-d's `attributes-sampled.resolve.json`, unchanged | `tools/cli/tests/attributes_resolve.rs` |
| Gate 1 (real LLM) | NOT REQUIRED: no model-facing text changes | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-j-1 | **Admission in one dispatch.** In the fixture world, `arrive { nadia }` is one journal revision holding `arrived`, `newcomer-admitted` and `traits-stated` for Nadia, in that order; Nadia's authored value is in it unchanged; Omar's locked value survives his admission; Pia's value is inside her `allowed:` set | S30-j row; I-S30-7 |
| AC-j-2 | **1,000 arrivals (checkpoint).** Starting from the fixture's eight genesis people, 1,000 arrivals at one-minute intervals: every declared marginal within 0.5 pp and every conditional within 2 pp for groups of ≥ 50, and every decile of every numeric stratum of ≥ 50 within 2 pp of 10 %. The same on the San Diego County profile from a generated 200-person genesis population (the second world) | AC-S30-5; charter |
| AC-j-3 | **Determinism.** The values of the 1,000 arrivals are identical in two runs and on Linux, macOS and Windows (one digest printed and compared in CI) | AC-S30-7 |
| AC-j-4 | **Replay and resume (checkpoint).** In `newcomers_resume.rs`, for kill points before, between and after the four admissions, each survivor's facts, journal and snapshots equal the control's byte for byte, and `verify()` replays the survivor's save from genesis | S30-j row; I-j-5 |
| AC-j-5 | **Re-sampling detected (adversarial).** Plant X-j-1 makes `verify()` fail with `ReplayDiverged` naming the admission's revision; the drift configuration makes resume fail with `ConfigurationDrift` naming `attributes` | S30-j row |
| AC-j-6 | **No seed, refused (adversarial).** `arrive` of a non-deferred person is rejected `not-a-newcomer`; a second `arrive` of Nadia `already-arrived`; a creator without `admits` `not-an-admitter`; plant X-j-4 (the creator skips `validate`'s check) stops the world with `FactRefusedByOwner` and the code; a proposal about Quin before he arrives is `not-arrived`; plant X-j-5 (the creator emits `arrived` without the request) fails the scenario's check that every arrived person has a `traits-stated` in the same revision | S30-j row; F-S30j-4 |
| AC-j-7 | **Nothing else moves.** Without `newcomers:`, `attributes-composed`'s bytes for `attributes-sampled` equal `main`'s (a hand-copied fact), and S30-d's golden is unchanged | I-j-1 |
| AC-j-8 | **Kernel ignorant.** `git diff --stat origin/main...HEAD -- kernel contracts` is empty, and AC-S30-12's grep passes | AC-S30-12 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s30j`. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the steps in order; nothing of a later commit.
3. Run the commit's validation one `--test` or `--lib` at a time, through `cast-slots/with-slot.sh`; record command,
   count, wall time and PASS / FAIL / INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`;
   `cargo fmt --all --check`. **R-LOAD-1:** never `cargo test --workspace`.
4. Mutations: copy the file, plant, run the named test red (record its name), restore from the copy, `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert or stop and record why.
7. Update §16 and the handoff in the same commit; stage by path; commit with the session's attribution line. No
   heredoc, no `sed -i`, no `python3 -c`, no `curl`.

#### C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30j.md` (new).
- Steps: record the base; confirm S30-c, S30-d, S30-e, S30-f merged; re-read A-j-1 … A-j-22 and record every moved
  line; record §15's rulings; if Q-S30j-1 is ruled (b), stop: this plan does not apply.
- [ ] Implementation. [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  [ ] Review: the base satisfies §2.3; every ruling is applied to §§5, 8 before C1.
- **Commit message:** `docs(plan): S30-j — open the ledger at the base`

#### C1 — Shared helpers moved; the draw

- **Files:** `authoring/attributes/src/sample/{mod.rs, resolve.rs, values.rs (new), newcomer.rs (new)}`;
  `authoring/attributes/tests/newcomers.rs` (new).
- Steps: move `value_at`, `slug_of`, `index_of`, `traits_of`, `current`, `numeric_value`, `apply_roles` to
  `values.rs`, `pub(crate)`, each taking `at: WorldTime` where it derives or births (genesis passes `EPOCH`); no other
  change to `resolve.rs`. Write `Resident`, `Arrival`, `sample_newcomer` as §5.4.
- Validation: `cargo test -p mineworld-attributes-model --lib`; `--test fidelity` (genesis cases unchanged);
  `--test newcomers`; `cargo test -p mineworld-cli --test attributes_resolve` (S30-d's golden, unchanged). Mutations
  **M-j-1** (deficit ignores `c(v)`) → the stratification case red; **M-j-2** (birth relative to `EPOCH` for a
  newcomer) → the birth case red; **M-j-3** (`because` mixed into the word) → the stability case red.
- Edge cases: an empty population (the first newcomer draws by weight); a group of zero (same); an all-zero
  restricted row (`Unsatisfiable`); a numeric stratum with no row (`MissingCondition`).
- [ ] Implementation. [ ] Validation. [ ] Review: genesis output byte-identical (the golden); no float; no name; the
  deficit sum is exactly 1 000 000 in the test's hand-computed case.
- **Commit message:** `feat(attributes-model): sample_newcomer — one person drawn against the people already there (S30-j)`

#### C2 — Configuration, runtime context, request fact, checked constructor

- **Files:** `authoring/attributes/src/{config.rs, context.rs, facts.rs, newcomer.rs (new), declaration.rs,
  catalogue.rs, lib.rs, profile/mod.rs}` and their tests; `docs/reference/` (regenerated for the configuration and the
  `uses:` block, as S30-d did).
- Steps: `newcomers: { later }` with `UnknownPerson`, `DuplicateNewcomer`; `NewcomerContext`, `Fixed`; the optional
  `newcomers` field of `AttributesComposed` (`skip_serializing_if`); `Deserialize` on the profile types with a
  round-trip test that the digest is unchanged; `uses.admits`; `NewcomerAdmitted`; `check_newcomer` and `admit_newcomer` (§5.5); the
  `not-arrived` case in `check_proposal`'s world form.
- Validation: `cargo test -p mineworld-attributes-model --lib`; `--test structure`. Mutation **M-j-4**
  (`skip_serializing_if` dropped) → AC-j-7's byte case red.
- [ ] Implementation. [ ] Validation. [ ] Review: every refusal names its person; the context holds no name and no
  origin beyond `Pinned`; absent block, absent bytes.
- **Commit message:** `feat(attributes-model): the newcomers block, its runtime context and the admission check (ARC-128, S30-j)`

#### C3 — The pack: genesis skip and the reaction

- **Files:** `systems/attributes/src/{section.rs, data.rs, system.rs}`; `systems/attributes/tests/newcomers.rs` (new).
- Steps: §5.3 and §5.6 exactly; the `attributes-newcomers` Process; declaration gains `subscribing_to` and
  `emitting` as needed.
- Validation: `cargo test -p mineworld-attributes --test newcomers`; `--lib`; `-p mineworld-acceptance --test
  attributes_seam` (S30-c's pattern, unchanged). Mutation **M-j-5** (`arrived` not updated) → the same-generation
  double admission case red.
- [ ] Implementation. [ ] Validation. [ ] Review: only `attributes` writes `Traits`; plain values only in the
  population (I-j-7); every fact in one dispatch.
- **Commit message:** `feat(attributes): admit a deferred person and state their drawn values in the same dispatch (S30-j)`

#### C4 — `resolve` shows deferred people

- **Files:** `tools/cli/src/attributes_resolve.rs`; `tools/cli/tests/attributes_resolve.rs` (one case);
  `tools/cli/tests/fixtures/attributes-newcomers/**` (new, §5.8).
- Validation: `cargo test -p mineworld-cli --test attributes_resolve`.
- [ ] Implementation. [ ] Validation. [ ] Review: the S30-d golden unchanged; deferred people carry no values.
- **Commit message:** `feat(cli): attributes resolve lists people who arrive later (S30-j)`

#### C5 — 1,000 arrivals on two profiles

- **Files:** `authoring/attributes/tests/fidelity.rs` (new cases).
- Steps: AC-j-2 and AC-j-3's digest line; the generated San Diego genesis population uses S30-d's `--synthetic`
  member shape (keys `synthetic-NNNNN`, no tags).
- Validation: `cargo test -p mineworld-attributes-model --test fidelity`. Mutation **M-j-6** (draw by weight only, no
  deficit) → the 0.5 pp check red on at least one marginal, recorded with its worst deviation.
- [ ] Implementation. [ ] Validation. [ ] Review: the expectations are the profile's own rows, never the code's output.
- **Commit message:** `test(attributes): 1,000 newcomers keep the world's shape on two profiles (AC-S30-5, S30-j)`

#### C6 — Replay, kill and resume; the adversarial plants

- **Files:** `tests/acceptance/tests/{newcomers_resume.rs (new), newcomers/creator.rs (new)}`;
  `tests/acceptance/Cargo.toml` (`[[test]] newcomers_resume`, `harness = false`).
- Steps: §5.7; the program as `arrival_resolvers_resume.rs` (A-j-19): the fixture world through `assemble`, the
  creator installed before genesis, four `arrive` requests among clock advances; kill points before, between and
  after them; AC-j-4, AC-j-5, AC-j-6.
- Validation: `cargo test -p mineworld-acceptance --test newcomers_resume` (one slot). Plants **X-j-1** (a
  process-local counter mixed into the word) → `verify()` red with `ReplayDiverged`; **X-j-4** (the creator's
  `validate` skips the check, admits a non-deferred person) → `FactRefusedByOwner`; **X-j-5** (the creator emits
  `arrived` only) → the completeness check red, naming the person.
- [ ] Implementation. [ ] Validation. [ ] Review: the survivor really resumed (head short of the control's, as the
  pattern requires); no expectation is computed by the code under test.
- **Commit message:** `test(attributes): newcomers survive replay and SIGKILL; re-sampling and a missing seed are caught (S30-j)`

#### C7 — Specification sync

- **Files:** `docs/MODULE_SPEC.md` §4.1; `docs/decisions/ARC-128.md` (new); `docs/decisions/ARC-123.md` (a dated note);
  the index through `python3 scripts/check_decision_ids.py --write-index`.
- Steps: `MODULE_SPEC` §4.1 "The world's configuration" (`newcomers:`), "Persistence" (line 1758: the sampler also
  runs inside an admission's dispatch; never at resume), the codes of §5.5, the fact `newcomer-admitted`;
  `ARC-128` with Problem, Options (a)/(b) of Q-S30j-1, Choice, Consequences, Revisit (when QPL-11 is decided).
- Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: no specification text and code disagree (`CLAUDE.md` §2.1 rule 4).
- **Commit message:** `docs(spec): runtime sampling of newcomers as implemented (ARC-128, S30-j)`

#### C8 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: AC-j-8's diff and grep; push `mvp0/pr-s30j`; `gh pr create`; CI on the exact head.
- [ ] Implementation. [ ] Validation: CI on the head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-j close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time, through
`cast-slots/with-slot.sh`; `cargo clippy` on the touched crates. Never `cargo test --workspace`, never the nightly
locally. CI on the exact head (Linux, Windows, macOS) is the gate.

---

## 11. Requirements this PR places on other PRs

- **R-QPL11-j-1 (whoever decides QPL-11).** If a System gains runtime entity creation, a Person it creates in a world
  with `newcomers:` must be admitted through `admit_newcomer` in the same dispatch. The `not-a-newcomer` check then
  widens from "listed in `later`" to "never seeded"; `ARC-128`'s Revisit names this.
- **R-S21-j-1 (S21 migration, or any arrival source).** Call `check_newcomer` in `validate`, and emit
  `admit_newcomer`'s emission within the arrival's own dispatch, citing the arrival's fact; declare `uses: { admits: true }`; place the person yourself.
- **R-dep-j-1 (any departure System).** Whether a departed person still counts toward a newcomer's group is that
  System's decision, recorded in `ARC-128`'s Revisit (R-j-3).
- **R-S31-j-1 (S31).** `economy.wealth-band` is drawn independently for newcomers (D-j-4). If S31 needs it stratified,
  it states the counts in a form `attributes` can read, in its own PR.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §3.3 F-S30-10 | a dated note: `create_entity` is not a System's capability (QPL-11); S30-j uses deferred people (Q-S30j-1) |
| `step-30-person-attributes.md` §8.6 | a third entry point: an admission's dispatch (`sample_newcomer`) |
| `step-30-person-attributes.md` §11.1 | "The sampler runs only at genesis" → "at genesis and inside an admission's dispatch; never at resume" |
| `step-30-person-attributes.md` §13.1, S30-j row | "a newcomer created without its seed" → "admitted", per F-S30j-4 |
| `overall.md` | S30-j frozen / merged; QPL-11 unchanged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-j — runtime sampling of new people
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30j-runtime-sampling.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§3.3, 8, 11, 12, 13); pr-s30c-attributes-pack.md;
                         pr-s30d-sampler.md; pr-s30e-builtin-catalogue.md; step-18-physics-list.md QPL-11;
                         docs/MODULE_SPEC.md §4.1; docs/decisions/ARC-120 … ARC-124, ARC-25, ARC-26; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30j, its own, held by one session
BRANCH:                  mvp0/pr-s30j, created from origin/main
START CONDITION:         this design frozen AND §15 ruled (Q-S30j-1 as (a)) AND the primary's dispatch; S30-d merged (met)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–5, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-6, I-S30-7, I-S30-8, I-S30-10; §6 I-j-1 … I-j-7; §8 as written
SEQUENCE:                C0 … C8 (§9)
IMPLEMENTER:             Opus-class session; C5 may be delegated to a Haiku-class sub-agent, reviewed before commit
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace), through with-slot.sh;
                         target/*/mineworld attributes *; git; gh (pr create/view; never merge);
                         python3 scripts/check_doc_headings.py, check_decision_ids.py [--write-index],
                         check_scratch.py, ci_layer.py docs; cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; any edit
                         under kernel/, contracts/, server/, persistence/, worlds/, or any systems/<pack> other than
                         attributes; any model call; a new external dependency
MATERIAL STOPS:          Q-S30j-1 ruled (b); a needed kernel, contracts, persistence or server change; a reference
                         world's bytes moving; S30-d's golden moving; AC-j-2 unreachable with §5.4's rule
PLATFORMS:               Linux, macOS, Windows
VALIDATION BUDGET:       crate and pack tests unrestricted, one file at a time; the resume program < 2 min per run.
                         Gate 1 NOT REQUIRED; Gate 2 = newcomers_resume.rs (§7)
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30j.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once frozen (source: the operator's freeze)
                         semantic commits: authorized (working rules §14)
                         branch push: authorized (working rules §21)
                         PR creation / update: authorized (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: MERGE-READY, reported to the coordinator with the PR number and head SHA
                         merge: never by the implementation session — the coordinator merges (charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-30, overall.md; implementation session: this ledger and the merge identity
STOP CONDITION:          S30-j READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-j-1 | The newcomer Process rewrites its whole state (with the profile) on each admission, growing snapshots | the profile is static and small (a few KB for the fixtures; measured in C3 for San Diego and recorded); if it exceeds 64 KB, the profile moves to its own Process written once (bounded) |
| R-j-2 | Scanning every Person's `Traits` per admission is O(people × attributes) | 1,000 arrivals into 1,000 people is ~10⁶ reads, measured in C5; a count cache is an owner-held Process later, not now (`CLAUDE.md` §4 rule 11) |
| R-j-3 | With ageing and no departures the oldest band grows past its share; newcomers cannot correct an excess | named, not hidden: AC-j-2 arrives within one simulated day; departures are R-dep-j-1 |
| R-j-4 | Deferred people are visible before they arrive (present, named, no attributes) | the fixture accepts it; a real arrival System places people itself (R-S21-j-1); route (b) removes it |
| R-j-5 | Two admissions in one generation are drawn against the same population | correct and deterministic; the second is not skewed by more than one person; recorded (D-j-6) |
| R-j-6 | The deficit rule is less random than independent draws: a small group's next value is predictable | only the shape is fixed; which value among those below share is still drawn; `stratify: false` keeps independent draws per attribute |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30j-1 [how a newcomer comes into being; QPL-11]** | No System can create a Person at run time; that is QPL-11's open kernel question (F-S30j-1). How should S30-j bring newcomers? | **(a) Deferred people.** A world declares people now and lists them under `newcomers.later`; a System admits one at run time and `attributes` draws their values then. No kernel change, and S30-j ships now. The hook takes an existing person, so a later kernel PR (route (b)) reuses it unchanged. The cost: "created without its seed" becomes "admitted without its seed"; the world refuses every checkable case, but a creator that never calls the hook is caught by tests, not by the world (F-S30j-4). Declined for now: (b) a kernel PR for runtime entity creation first — it also serves migration and fragile shards, but it is a public kernel change with its own design and it delays this release-blocking PR |
| **Q-S30j-2 [what "realised counts" counts]** | A newcomer is drawn against the people already there. Should they be counted by their values **now** (ages advanced, evolved traits as they are), or by the values they were drawn with? | **Now, at the instant of arrival** — the step's words ("the current population"), and no extra stored state. Owner-held attributes are drawn independently, because their values are the owner's state (D-j-4). Consequence: an ageing world with no departures drifts old; departures are a later System's (R-j-3) |
| **Q-S30j-3 [the record]** | `ARC-123` says "the sampler runs only at genesis". Record runtime sampling as a dated note there, or as a new record? | **A new record, `ARC-128`**, with a dated note under `ARC-123` pointing to it. It adds a fact, a Process, a configuration block and a refusal set, and it carries QPL-11's revisit clause |

Bounded decisions recorded here rather than asked:

- **D-j-1.** The creator emits `attributes`' request fact; `attributes` draws and states (F-S30j-3).
- **D-j-2.** The runtime context lives in `attributes-composed` and a Process, only with `newcomers:` (F-S30j-2).
- **D-j-3.** Numeric strata use ten decile bins under the same deficit rule (F-S30j-7).
- **D-j-4.** Owner-held attributes are drawn independently (F-S30j-5).
- **D-j-5.** A newcomer's birth is relative to the instant of arrival.
- **D-j-6.** Admissions in one generation are drawn against that generation's population (A-j-7).
- **D-j-7.** `because` names the cause and is checked, but is not mixed into the words (§5.4).

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base and ledger | — | [ ] | [ ] | [ ] |
| C1 the draw | — | [ ] | [ ] | [ ] |
| C2 configuration, context, admission check | — | [ ] | [ ] | [ ] |
| C3 the pack | — | [ ] | [ ] | [ ] |
| C4 `resolve` | — | [ ] | [ ] | [ ] |
| C5 1,000 arrivals | — | [ ] | [ ] | [ ] |
| C6 replay, resume, plants | — | [ ] | [ ] | [ ] |
| C7 specification sync | — | [ ] | [ ] | [ ] |
| C8 close-out | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-j-1 … M-j-6 and X-j-1, X-j-4, X-j-5 as §9 lists them)

### Deviations and findings

F-S30j-1 … F-S30j-8 (§3.2), recorded at design time.
