# Step 01 / PR 01 — Entity and Component contracts

**Role:** combined **step and PR** document. Step S1 needs exactly one PR, so this single
document is the authority for both levels; there is no separate step file.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md)
**Lifecycle:** `DRAFT — AWAITING OPERATOR APPROVAL TO FREEZE`
**Implementation base:** branch `main`, commit `75e1d2b` (no prerequisites to merge)
**Blocked by:** prerequisite **P-1** (no Rust toolchain installed — §4.2)

Binding parents: [`overall.md`](overall.md) ·
[`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) ·
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) ·
[`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) ·
[`CLAUDE.md`](../../../CLAUDE.md)

---

# 1. Goal

Establish the contract layer for **identity and state** — the vocabulary every later step
depends on — as a Cargo workspace with one crate, `mineworld-contracts`, containing pure data
types and no behavior beyond validation and serialization.

This is the step where `INV-7` (single writer), `INV-12` (kernel ignorance) and
`ENGINEERING_STANDARDS.md` §12 (strong typing at boundaries) become mechanically enforceable
rather than aspirational. Getting the types wrong here is expensive in every later step, which
is why it is its own PR.

## 1.1 Approved scope

```text
Cargo workspace at the repository root
contracts/  →  crate mineworld-contracts

    identity        EntityId, EntityKey, EntityType, typed entity references
    id family       SystemId, ComponentTypeId, RelationTypeId, EventId,
                    ActionId, ProcessId
    time            WorldTime, SimDuration                (types only, no scheduling)
    entity record   Entity: id, key, type, tags, lifecycle, metadata
    component model Component trait, ComponentDeclaration, ComponentRecord
    relation model  RelationTypeDeclaration, Relation (triple-keyed)
    errors          one typed error enum for contract validation failures

docs/ARCHITECTURE.md   §13 amended to record decision D-4
contracts/README.md    updated to describe what now exists
```

## 1.2 Non-goals for this PR

Explicitly not in this PR, and not to be "just added while we are here":

```text
ActionIntent / ActionResult / Event / Observation payloads      → S2
System trait, registry, dependency resolution, write enforcement → S3
EntityId allocation, component storage, any mutable world state  → S3/S4
scheduler, processes, WorldTime arithmetic policy                → S4
persistence, event log, snapshots, migrations                    → S5
any domain semantics: jobs, money, hunger, sleep, conversation   → S6+
Protobuf, cross-language bindings                                → S10/S11
```

`ComponentRecord` in this PR is the *shape* a persistence layer will store, not a persistence
implementation.

## 1.3 Frozen invariants for this PR

- `INV-12`: nothing in `mineworld-contracts` names a domain concept. A type called
  `Employment`, `Money`, `Hunger`, or `Conversation` appearing in this crate fails review.
- `ENGINEERING_STANDARDS.md` §12: no `HashMap<String, serde_json::Value>`-shaped state and no
  `Any` crosses a contract boundary. Component payload erasure exists in exactly one place
  (`ComponentRecord`), and it is documented as the persistence-facing boundary.
- Every collection type in this crate is order-deterministic (`BTreeMap`, `BTreeSet`, `Vec`).
  `HashMap`/`HashSet` are forbidden in contract types, because iteration order reaches the
  event log and would break `AC-12` (risk `R-6` in the parent).
- No random identity. No UUIDv4, no time-seeded values, nothing that differs between two runs
  of the same seeded world.
- Every contract type is `serde`-serializable by construction, so that `S5` can reconstruct
  state from the event log (risk `R-2`).
- The crate has no dependency on the kernel, on any system, or on any renderer, transport, or
  model provider (`INV-14`, dependency direction in `ENGINEERING_STANDARDS.md` §6).

---

# 2. Integration checkpoint

A PR this early cannot integrate with a running world, so its checkpoint is the strongest one
available: an **executable demonstration that the type system enforces the invariants**, not a
claim that it does.

```text
cargo test -p mineworld-contracts
    ├── round-trip: every contract type survives serialize → deserialize unchanged
    ├── determinism: repeated serialization of the same value is byte-identical
    ├── compile-fail: a PlaceId passed where a PersonId is required does not compile
    ├── compile-fail: constructing a typed reference from a raw EntityId without the
    │                 checked constructor does not compile
    └── validation: malformed EntityKey / SystemId / Tag are rejected with a named error
```

**Adversarial criteria** — the checkpoint is not satisfied if any of these is possible:

1. A component's payload can be written as an untyped map outside `ComponentRecord`.
2. A typed entity reference can be produced from an arbitrary `EntityId` with no type check.
3. Any contract type serializes with nondeterministic ordering.
4. Any contract type carries a domain concept.
5. A test asserts an expected value produced by the implementation under test
   (test rules §25).

---

# 3. Test ownership declaration

Mapping the skill's layer taxonomy onto this project
([`references/adaptation.md`](../../../.claude/skills/structured-coding/references/adaptation.md)):
"Gate 1" is real-model evidence, "Gate 2" is real-lifecycle evidence.

| Layer | Owns in this PR |
| --- | --- |
| **Static** | formatting, compilation, type errors, lints: `cargo fmt --check`, `cargo check`, `cargo clippy -D warnings`. No unit test duplicates a property these own. |
| **Unit** | deterministic local semantics: serde round-trip and byte-stable ordering; `EntityKey` / `SystemId` / `Tag` validation including boundary lengths and illegal characters; `LifecycleState` transition legality; component declaration ↔ owner association; relation endpoint-type validation; typed-reference checked conversion success and failure. |
| **Compile-fail (unit, via `trybuild`)** | type-level guarantees that cannot be asserted at runtime: id-kind confusion, unchecked typed-reference construction. |
| **Gate 1 — real model** | `NOT REQUIRED`. No model exists in this effort before S10, and no LLM-facing semantics are touched. |
| **Gate 2 — real lifecycle** | `NOT APPLICABLE`. There is no runtime, no process, no filesystem lifecycle and no persistence in this PR. First applicable in S5 (persistence restart) and S7 (headless run). |
| **CI** | `NOT AVAILABLE` by decision D-9: no remote exists. Canonical evidence for this PR is the four local commands of §6 run at the exact final HEAD, recorded in §8. |

Per test rules §8–§9 there is no full-suite run to make: the workspace contains one crate and
its own tests are the whole suite.

---

# 4. Source audit

## 4.1 Inspected

| Source | Finding used by this design |
| --- | --- |
| `docs/CORE_CONCEPTS.md` §3 | Entity fields: `EntityID`, `EntityType`, `Tags`, `LifecycleState`, `Metadata`; metadata is authoring provenance and never gameplay semantics. |
| `docs/CORE_CONCEPTS.md` §3.1 | Component carries `ComponentType`, `OwnerSystem`, explicit versioned `Schema`, typed `Data`; reads may be broad, writes are the owner's alone. |
| `docs/CORE_CONCEPTS.md` §4 | Person state classes (identity / traits / mutable / relationships) are *component* concerns, not fields of `Entity`. Confirms `Entity` stays semantics-free. |
| `docs/CORE_CONCEPTS.md` §9 | Relations are typed edges across all entity-type pairs, and a system may attach its own state to a relation, which it then owns. |
| `docs/CORE_CONCEPTS.md` §13 | System declaration fields — consumed in S3, but `SystemId` must exist here because component ownership references it. |
| `docs/ARCHITECTURE.md` §2 | Kernel responsibility list; identity and component storage are kernel, so the contracts crate must not contain storage. |
| `docs/ARCHITECTURE.md` §13–14 | Stack table says "Protobuf + generated language bindings"; §14 fixes the one-way dependency `World Pack → Systems → Kernel Contracts`. The first is amended by D-4 in this PR; the second is satisfied by the workspace layout. |
| `docs/ENGINEERING_STANDARDS.md` §§12–14 | The id-type list to provide, the ban on untyped blobs at boundaries, and the requirement that invalid states be hard to represent. |
| `docs/ENGINEERING_STANDARDS.md` §§17–19, 26–28 | Test posture: no trivial tests, no coverage target, behavior over implementation, no premature abstraction. |
| `docs/MVP.md` §9 `AC-12` | Determinism requirement, which is why ordering and id allocation are design constraints rather than details. |

## 4.2 Repository state and prerequisites

```text
git branch      main
HEAD            75e1d2b  (docs: define MineWorld architecture, MVP, and engineering process)
working tree    clean apart from this planning directory
Cargo.toml      none anywhere in the repository
Rust code       none
directories     kernel/ systems/ cognition/ server/ clients/ sdk/ worlds/ tools/ tests/
                contain only .gitkeep
.gitignore      already excludes /target/ and **/*.rs.bk
```

**P-1 — blocking prerequisite: no Rust toolchain is installed.**

```text
cargo   → command not found
rustc   → command not found
```

Nothing in this PR can be validated until a toolchain exists, which means the PR can be
designed and frozen but not executed. Installing it changes the operator's machine, so it
needs explicit authorization. Recommended: `rustup` with the current stable toolchain, pinned
in `rust-toolchain.toml` (channel, plus the `rustfmt` and `clippy` components) so that every
later session and eventually CI use the same compiler. The pinned channel is recorded in the
ledger when P-1 is satisfied, since the design cannot name a version it has not observed.

**P-2 — non-blocking:** the `cargo` checks declared in `.structured-coding/standards.md`
require one `standards.py approve --project .` before the standards helper will run them. The
same commands can be run directly, which is what §6 does; the approval only affects the
helper's report.

## 4.3 Unresolved assumptions

| ID | Assumption | Resolution path |
| --- | --- | --- |
| **A-1** | A small declarative macro to generate the four typed entity references is clearer than four hand-written copies. | Write both forms for one type during C1; keep the one a reader understands faster and record the comparison. `ENGINEERING_STANDARDS.md` §27 forbids clever metaprogramming *without need*; four identical newtypes plus their checked constructors is the need, and the macro stays under ten lines of expansion per type or it is abandoned. |
| **A-2** | `trybuild` is the right way to make the compile-fail guarantees executable. | Confirmed during C1 by making one compile-fail case actually fail for the expected reason; if the diagnostics prove too brittle to assert on, downgrade to asserting the absence of a public unchecked constructor plus a documented reason, and record the downgrade rather than dropping the claim. |
| **A-3** | `WorldTime` as simulated seconds since a world epoch is sufficient for S4's scheduler. | S4 owns the scheduling model (D-6 still open). This PR provides the type and its ordering only; if S4 needs a different representation, changing it is a contract change in S4, cheap while no data is persisted. |

---

# 5. Design decisions

Recorded before implementation so they can be reviewed as decisions rather than discovered as
code.

| ID | Decision | Rationale |
| --- | --- | --- |
| **DD-1** | `EntityId` is a monotonically allocated opaque integer newtype, never reused. Allocation itself belongs to the kernel (S3), not to this crate. | Random identity (UUIDv4) would make two runs of the same seeded world differ, breaking `AC-12`. Keeping the allocator out keeps the contracts crate pure data. |
| **DD-2** | `EntityKey` is a separate, authoring-stable identifier (the `alice` in `people/alice.yaml`), validated to a restricted character set. World Packs reference entities by key; the runtime resolves keys to `EntityId` at load. | World Pack authors need stable human-readable references, and the runtime needs compact deterministic identity. Conflating them would force either fragile string comparison in hot paths or unstable authoring references. |
| **DD-3** | Typed entity references (`PersonId`, `PlaceId`, `ItemId`, `OrganizationId`) wrap `EntityId`, convert *to* `EntityId` for free, and can only be produced *from* one through a checked constructor that is given the `EntityType`. | `ENGINEERING_STANDARDS.md` §12: distinct semantic concepts must not be interchangeable. This is the compile-time half of that requirement. |
| **DD-4** | Component ownership is part of the component's *type-level* declaration: a `Component` implementation states its `ComponentTypeId`, its owning `SystemId`, and its schema version. | S3 enforces `INV-7` against this declaration. Ownership recorded per instance instead of per type would let two systems claim the same component at runtime. |
| **DD-5** | Exactly one type erases a component payload — `ComponentRecord`, the persistence- and wire-facing shape — and it is documented as that boundary. Typed access goes through `Component`. | Erasure is unavoidable at a storage boundary; the standard forbids it *crossing core module boundaries* casually. One named, documented place is the compromise. |
| **DD-6** | Relations are identified by the triple `(RelationTypeId, from, to)`. There is no `RelationId`, and a second edge of the same type between the same ordered pair is not representable. | Matches `CORE_CONCEPTS.md` §9. Per-relation state lives in a component owned by the declaring system, keyed by the same triple, so `INV-7` applies unchanged. |
| **DD-7** | All collections in contract types are `BTreeMap` / `BTreeSet` / `Vec`. `HashMap` and `HashSet` are denied in this crate, with the reason stated in the crate docs. | Hash iteration order would leak into serialized state and the event log, breaking `AC-12`. |
| **DD-8** | Dependencies are `serde` (derive) and `thiserror`; dev-dependencies are `serde_json` and `trybuild`. Nothing else. | Each earns its place: serialization is a frozen invariant, typed errors are required by §12, JSON is the round-trip vehicle in tests, `trybuild` is the only way to test a compile-time guarantee. Explicitly rejected: `uuid` (DD-1), `chrono` (calendar semantics are a system concern, `INV-12`), any ECS crate (decision D-5 in the parent). |
| **DD-9** | `WorldTime` is a signed integer count of simulated seconds from a world epoch, with `SimDuration` for differences; calendar interpretation (dates, weekdays, "modern" vs other calendars) is a system concern and absent here. | `INV-12`. A `chrono::DateTime` in the kernel contracts would import a calendar the kernel must not know about. |
| **DD-10** | `EventId`, `ActionId`, `ProcessId` newtypes are defined here even though their payload contracts arrive in S2 and S4. | One authoritative id module beats three crates each inventing its own integer wrapper; the cost is a few unused types for one PR. |

---

# 6. Verification commands

```sh
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p mineworld-contracts
```

Test counts and wall time are recorded in §8 after each run. A command that could not be run
is recorded as `NOT RUN` with its reason and never reported as a pass (test rules §16).

---

# 7. Commit plan

Five semantic commits. Each is independently reviewable, carries its own tests, and updates
this ledger before it is made.

Every commit tracks three separate obligations. Completing one never completes another.

---

## C1 — Cargo workspace and the identity module

### Goal
Create the workspace and the identity vocabulary every other type depends on. It is first
because nothing else in the crate can be written without ids, and it is separate because the
workspace layout is the decision reviewers most need to see in isolation.

### Scope
```text
new  Cargo.toml                       [workspace] with members = ["contracts"]
new  rust-toolchain.toml              pinned channel + rustfmt, clippy
new  contracts/Cargo.toml             crate mineworld-contracts
new  contracts/src/lib.rs             crate docs stating the DD-7 / DD-5 rules
new  contracts/src/ids.rs             EntityId, EntityKey, EntityType, typed refs,
                                      SystemId, ComponentTypeId, RelationTypeId,
                                      EventId, ActionId, ProcessId
new  contracts/src/error.rs           ContractError (thiserror)
new  contracts/tests/compile_fail.rs  trybuild harness
new  contracts/tests/compile_fail/*.rs  the failing cases
```
Non-goals: no `Entity`, no components, no relations, no time. Depends on: nothing (P-1 only).

### Implementation
- [ ] Workspace `Cargo.toml` with `resolver = "2"`, shared `[workspace.package]` metadata and `[workspace.dependencies]` for `serde` and `thiserror`.
- [ ] `rust-toolchain.toml` pinning the observed stable channel with `rustfmt` and `clippy`.
- [ ] `mineworld-contracts` crate with `#![forbid(unsafe_code)]` and a crate-level doc comment stating: no domain concepts, no hash-ordered collections, one erasure boundary.
- [ ] `EntityId` newtype: opaque integer, `Copy`, ordered, serde-transparent, no public arithmetic, no `Default`.
- [ ] `EntityKey`: validated constructor (restricted charset, bounded length, no leading or trailing separator), `TryFrom<&str>`, `Display`.
- [ ] `EntityType` enum: `Person`, `Place`, `Item`, `Organization`, exhaustive and serde-stable.
- [ ] Typed references per DD-3, with free conversion to `EntityId`, a checked constructor taking `(EntityId, EntityType)`, and **no** public unchecked constructor.
- [ ] `SystemId`, `ComponentTypeId`, `RelationTypeId`: validated slug newtypes.
- [ ] `EventId`, `ActionId`, `ProcessId`: opaque integer newtypes (DD-10).
- [ ] `ContractError` enum with one variant per validation failure class, each carrying the offending value's description but not a formatted blob.
- [ ] Resolve A-1: write the macro and the hand-written form for one type, keep the clearer one, record the comparison in §8.

### Validation
- [ ] Unit: `EntityKey` accepts the documented legal forms and rejects empty, over-long, illegal-character and separator-edge cases, each with the expected `ContractError` variant.
- [ ] Unit: slug validation for `SystemId`, `ComponentTypeId`, `RelationTypeId` over the same boundary classes.
- [ ] Unit: checked typed-reference construction succeeds for the matching `EntityType` and fails with the expected error for every non-matching one.
- [ ] Unit: serde round-trip for every type introduced, plus byte-stability on repeated serialization.
- [ ] Compile-fail: `PlaceId` where `PersonId` is required (A-2).
- [ ] Compile-fail: constructing a typed reference from a raw `EntityId` without the checked constructor.
- [ ] Static: the four commands of §6.

### Review
- [ ] Inspect every public item for a domain concept that should not be in this crate (`INV-12`).
- [ ] Confirm no `HashMap`/`HashSet` and no `Default` on an id type that would fabricate identity.
- [ ] Confirm no test asserts a value produced by the code under test (test rules §25).
- [ ] Confirm the crate has no dependency pointing at `kernel/`, `systems/`, or anything outside `[workspace.dependencies]`.

### Acceptance criteria
`cargo fmt --all --check`, `cargo check`, `cargo clippy -D warnings` all clean; `cargo test -p
mineworld-contracts` passes with the compile-fail cases failing for the *expected* reason (the
`trybuild` expected-output files match, not merely "some error"); `EntityId` exposes no public
constructor other than the one the kernel will use, and that one is documented as
kernel-allocation-only.

### Failure and edge cases
| Case | Required behavior |
| --- | --- |
| `EntityKey` empty, over-long, illegal character, leading/trailing separator | rejected with the specific `ContractError` variant; never silently normalized |
| Typed reference constructed with a mismatched `EntityType` | rejected, carrying both the expected and the actual type |
| Unknown `EntityType` during deserialization | hard error, never a silent default |
| Integer overflow at the top of the `EntityId` space | documented; allocation is the kernel's problem in S3, and this crate must not wrap silently |
| `trybuild` diagnostics unstable across toolchain versions | fall back per A-2 and record it; do not delete the claim |

### Commit boundary
Independently reviewable: the workspace plus one module plus its tests. No entity record, no
components, no relations, no documentation amendments. Diff summary, staged file list, test
counts and any deviation recorded in §8 before committing.

---

## C2 — Entity record and lifecycle

### Goal
Represent an entity as `CORE_CONCEPTS.md` §3 defines it, and nothing more — the step where the
temptation to add `name`, `position`, or `inventory` is refused in a reviewable way.

### Scope
```text
new  contracts/src/entity.rs   Entity, Tag, Tags, LifecycleState, Metadata
edit contracts/src/lib.rs      module wiring
new  tests in-crate            lifecycle and round-trip
```
Non-goals: no component attachment, no relations. Depends on: C1.

### Implementation
- [ ] `Tag` validated newtype; `Tags` as a `BTreeSet<Tag>` wrapper with set semantics and deterministic iteration.
- [ ] `LifecycleState` enum (`Active`, `Dormant`, `Destroyed`) with an explicit `can_transition_to` relation and no transition out of `Destroyed`.
- [ ] `Metadata`: authoring provenance only — source pack, source path, optional authoring note — documented as never gameplay-relevant.
- [ ] `Entity` record: `id`, `key`, `entity_type`, `tags`, `lifecycle`, `metadata`; constructed through a builder or explicit constructor that cannot produce a half-initialized value.
- [ ] Document on the type that all semantics live in components (`INV-12`, `CORE_CONCEPTS.md` §4).

### Validation
- [ ] Unit: legal transitions permitted, illegal ones rejected, `Destroyed` terminal.
- [ ] Unit: `Tags` deduplicates, orders deterministically, and round-trips byte-identically.
- [ ] Unit: `Entity` serde round-trip preserving every field.
- [ ] Unit: `Metadata` is optional in the sense the specification requires — an entity authored without provenance is representable.
- [ ] Static: §6 commands.

### Review
- [ ] Confirm `Entity` gained no gameplay field, and that adding one would be visibly wrong.
- [ ] Confirm no lifecycle transition is reachable by mutating a public field directly.
- [ ] Confirm no test merely asserts that a constructor assigned its arguments
      (`ENGINEERING_STANDARDS.md` §18).

### Acceptance criteria
An `Entity` cannot be constructed in an inconsistent state through the public API; `Destroyed`
has no outgoing transition; two serializations of an equal `Entity` are byte-identical; §6
clean.

### Failure and edge cases
| Case | Required behavior |
| --- | --- |
| Transition from `Destroyed` | rejected with a named error |
| Duplicate tags supplied | deduplicated, not an error |
| Unknown lifecycle value on deserialization | hard error |
| Metadata absent | valid |

### Commit boundary
One module, its tests, module wiring. No component or relation code.

---

## C3 — Component model

### Goal
Make component ownership a type-level fact, so S3 can enforce `INV-7` mechanically instead of
by convention. This is the highest-value commit in the PR.

### Scope
```text
new  contracts/src/component.rs   Component trait, ComponentTypeId binding,
                                  ComponentSchemaVersion, ComponentDeclaration,
                                  ComponentRecord
edit contracts/src/lib.rs
new  tests in-crate               declaration, ownership, erasure round-trip
```
Non-goals: no storage, no registry, no write enforcement (that is S3 consuming this). Depends
on: C1, C2.

### Implementation
- [ ] `ComponentSchemaVersion` newtype with ordering, so migrations in S5 can compare.
- [ ] `Component` trait with associated `COMPONENT_TYPE: ComponentTypeId`, `OWNER: SystemId`, `SCHEMA_VERSION`, and serde bounds; documented so that implementing it *is* the ownership declaration (DD-4).
- [ ] `ComponentDeclaration`: the runtime-inspectable form of the same facts, derivable from any `Component` implementation, for S3's registry.
- [ ] `ComponentRecord`: `(EntityId, ComponentTypeId, ComponentSchemaVersion, payload)` — the single documented erasure boundary (DD-5), with typed conversion to and from a `Component`.
- [ ] Two test-only component implementations owned by two different stub systems, used by this and later commits' tests.

### Validation
- [ ] Unit: a `Component` implementation's declaration reports the expected type id, owner and version.
- [ ] Unit: typed → `ComponentRecord` → typed round-trip is lossless.
- [ ] Unit: decoding a `ComponentRecord` into the wrong component type fails with a named error rather than producing a default value.
- [ ] Unit: decoding a record whose schema version is newer than the target type fails explicitly (forward-incompatibility is an error, not a silent drop).
- [ ] Unit: two components with the same `ComponentTypeId` but different owners are detectable as a conflict at declaration level — the check S3 will rely on.
- [ ] Static: §6 commands.

### Review
- [ ] Confirm the erasure boundary exists in exactly one type and is documented as such.
- [ ] Confirm nothing outside `ComponentRecord` accepts or returns an untyped payload.
- [ ] Confirm the trait cannot be implemented without stating an owner.
- [ ] Confirm no component defined in this crate carries domain meaning; the two test components are named for their role in tests, not for a gameplay concept.

### Acceptance criteria
Ownership and schema version are inseparable from the component type; wrong-type and
newer-version decodes fail with distinct named errors; the conflict case a registry must reject
is demonstrated by a test at declaration level; §6 clean.

### Failure and edge cases
| Case | Required behavior |
| --- | --- |
| Wrong-type decode | named error, no default value |
| Newer schema version than the type supports | named error naming both versions |
| Older schema version | representable and reported, so S5 can migrate rather than guess |
| Same `ComponentTypeId` declared by two owners | detectable as a conflict |
| Payload that is not valid for the declared type | named error |

### Commit boundary
The component model and its tests. No registry, no enforcement, no storage.

---

## C4 — Relation model

### Goal
Represent typed edges as `CORE_CONCEPTS.md` §9 defines them, across every entity-type pair,
with attached state left to the declaring system.

### Scope
```text
new  contracts/src/relation.rs   RelationTypeDeclaration, Relation, RelationKey
edit contracts/src/lib.rs
new  tests in-crate              endpoint validation, triple identity, round-trip
```
Non-goals: no relationship *values* (friendship, trust) — those are components owned by a
relationships system in S8. Depends on: C1, C2, C3.

### Implementation
- [ ] `RelationTypeDeclaration`: id, owning `SystemId`, directedness, and the permitted endpoint `EntityType` sets.
- [ ] `Relation`: the `(RelationTypeId, from, to)` triple per DD-6, with a validating constructor that checks endpoint types against the declaration.
- [ ] Symmetric handling for an undirected relation type: a canonical ordering so that `(a, b)` and `(b, a)` are the same edge and serialize identically.
- [ ] Document that per-relation state is a component keyed by the triple and owned by the declaring system.

### Validation
- [ ] Unit: a relation whose endpoints violate the declared types is rejected, naming the offending endpoint.
- [ ] Unit: an undirected relation constructed in both argument orders produces one identical canonical value.
- [ ] Unit: a directed relation distinguishes `(a, b)` from `(b, a)`.
- [ ] Unit: every entity-type pair the specification lists (Person↔Person, Person↔Organization, Organization↔Place, Person↔Item, Place↔Place) is representable.
- [ ] Unit: serde round-trip and byte-stability.
- [ ] Static: §6 commands.

### Review
- [ ] Confirm no relationship semantics (friend, employee, owns) are defined in this crate — only the machinery.
- [ ] Confirm canonical ordering cannot be bypassed by direct field construction.
- [ ] Confirm the triple-identity consequence (no parallel edges of one type) is documented where a reader will find it.

### Acceptance criteria
All five specification pairs are representable; endpoint violations are rejected with a named
error; undirected edges are canonical in both construction orders; §6 clean.

### Failure and edge cases
| Case | Required behavior |
| --- | --- |
| Endpoint type not permitted by the declaration | rejected, naming the endpoint and both types |
| Self-edge (`from == to`) | permitted or rejected per the declaration, never left implicit |
| Undirected edge given in reverse order | canonicalized |
| Unknown `RelationTypeId` | the caller's problem in S3; this crate validates against the declaration it is given, and that is documented |

---

## C5 — Specification synchronization

### Goal
Leave no stale specification behind: record decision D-4 where a future session will read it,
and describe what `contracts/` now contains. `CLAUDE.md` §2.1 rule 4 makes a code/spec
disagreement a defect, so this is part of the PR rather than a follow-up.

### Scope
```text
edit docs/ARCHITECTURE.md     §13 contracts row + a short note recording D-4's route
edit contracts/README.md      what exists now, what arrives in S2/S3 (human-facing, short)
edit this document            final ledger, evidence, deviations
```
Non-goals: no other specification edits; D-2 (§16 wording) is **not** bundled here — it is a
separate open decision. Depends on: C1–C4.

### Implementation
- [ ] `docs/ARCHITECTURE.md` §13: change the contracts row to state Rust types as the source of truth with Protobuf introduced at the first cross-language boundary, referencing decision D-4 and the step that will do it.
- [ ] `contracts/README.md`: rewrite for what now exists, staying short and human-facing per the documentation law.
- [ ] Final ledger entries in §8: evidence, decisions, deviations, and the resolution of A-1/A-2/A-3.

### Validation
- [ ] Every relative link in the touched documents resolves.
- [ ] No specification statement contradicts the code that now exists — verified by re-reading the amended sections against the crate's public API.
- [ ] Static: §6 commands once more at the final head (docs-only changes still shift the head; test rules §20).

### Review
- [ ] Confirm `contracts/README.md` stayed human-facing and short, and did not become a specification.
- [ ] Confirm the D-4 amendment records the decision and its reason, not just the new state.
- [ ] Confirm no other document drifted (`MODULE_SPEC.md` §9 versioning claims still hold).

### Acceptance criteria
`docs/ARCHITECTURE.md` no longer states a Protobuf-first route; `contracts/README.md`
describes the real crate; links resolve; §6 clean at the final head.

### Commit boundary
Documentation only. No code change rides along.

---

# 8. Live ledger

Filled during execution. Nothing here is pre-written.

## 8.1 Progress
```text
C1  not started
C2  not started
C3  not started
C4  not started
C5  not started
```

## 8.2 Evidence
```text
(no validation has been run; P-1 blocks all of it)
```

## 8.3 Findings, decisions, deviations
```text
FINDING (planning, 2026-09-25)
    No Rust toolchain on the machine: cargo and rustc are absent.
SOURCE AUDIT
    cargo --version, rustc --version, rustup show — all "command not found";
    find . -name Cargo.toml → no results.
WHY IT MATTERS
    Every acceptance criterion in this PR is a compiler or test observation.
    The PR can be frozen but not executed.
DECISION
    Raised as prerequisite P-1 for operator authorization; rust-toolchain.toml
    pins the observed channel once it exists.
```

## 8.4 Resolution of unresolved assumptions
```text
A-1  open — resolved during C1
A-2  open — resolved during C1
A-3  open — carried to S4 (D-6)
```

---

# 9. Execution contract

Filled from the skill's template. This section is the authorization record; the working rules
it specializes are in
[`prompts/implementation-working-rules.md`](../../../.claude/skills/structured-coding/prompts/implementation-working-rules.md).

```text
PROJECT / PR:
  Step 01 / PR 01 — Entity and Component contracts (effort mvp0)

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/step-01-entity-component-contracts.md

RELATED / BINDING DOCS:
  .structured-coding/plans/mvp0/overall.md
  docs/CORE_CONCEPTS.md, docs/ARCHITECTURE.md, docs/ENGINEERING_STANDARDS.md
  CLAUDE.md

IMPLEMENTATION BASE:
  branch: mvp0/pr-01-entity-component-contracts, created from main @ 75e1d2b
  merged prerequisites: none

APPROVED SCOPE:
  §1.1 of this document. Create the Cargo workspace and the mineworld-contracts
  crate (identity, entity record, component model, relation model), amend
  docs/ARCHITECTURE.md §13 for D-4, update contracts/README.md.

FROZEN INVARIANTS:
  - no domain concept in mineworld-contracts (INV-12)
  - component ownership is type-level and inseparable (INV-7 precondition)
  - one documented payload-erasure boundary: ComponentRecord
  - BTreeMap/BTreeSet/Vec only; HashMap/HashSet forbidden in this crate
  - no random or time-derived identity; deterministic serialization
  - every contract type is serde-serializable
  - the crate depends on nothing in kernel/, systems/, clients/, cognition/
  - forbidden: storage, registry, dispatch, scheduling, persistence, any
    ActionIntent/Event/Observation payload

APPROVED IMPLEMENTATION SEQUENCE:
  C1 workspace + identity  →  C2 entity record  →  C3 component model
  →  C4 relation model  →  C5 specification synchronization

AUTONOMOUS VALIDATION BUDGET:
  - ordinary unit / static / compile-fail tests: unrestricted
  - real LLM calls: none in this PR (Gate 1 NOT REQUIRED)
  - real lifecycle / Gate 2: NOT APPLICABLE in this PR
  - GPU / API / monetary: none
  - toolchain installation: NOT AUTHORIZED until P-1 is granted

REQUIRED LIVE DOCUMENTATION:
  this document, §7 checkboxes and §8 ledger

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff.md   (initialized at execution start)

PR CONTEXT SCOPE:
  This context belongs to PR 01 only. It begins when the operator authorizes
  implementation and ends at READY FOR OPERATOR REVIEW.

ENDPOINT AUTHORITY:
  - implementation + local validation:  authorized once P-1 is granted
      source: operator instruction "我们继续" authorizing the mvp0 effort;
              execution still requires the freeze approval in §10
  - semantic commits:                   authorized
      source: operator's stated commit sequence for this repository
              (commit 2 = Entity/Component contracts)
  - branch push:                        NOT AUTHORIZED
      source: operator decision D-9 — local-only until S13
  - PR creation / update:               N/A — no remote exists
      source: operator decision D-9
  - remote CI:                          N/A — no remote exists
      source: operator decision D-9
  - merge into main:                    explicit operator authorization only

POST-MERGE SYNCHRONIZATION OWNER:
  planning session owns overall.md; the implementation session owns this
  document, its evidence, deviations and remaining issues.

NORMAL STOP CONDITION:
  PR 01 READY FOR OPERATOR REVIEW on branch
  mvp0/pr-01-entity-component-contracts — DO NOT MERGE.

STOP CONDITION:
  implementation complete
  + §6 commands clean at the exact final HEAD
  + this document synchronized
  + handoff current

MERGE AUTHORITY:
  NEVER merge without explicit operator approval.
```

---

# 10. Approval

This document is **not frozen**. Freezing requires the operator to:

1. approve the scope, the commit plan and the frozen invariants above;
2. grant prerequisite **P-1** (install the Rust toolchain), without which the PR cannot be
   executed;
3. confirm the endpoint authority block, in particular that pushing is not authorized.

On approval, replace this section with:

```markdown
## DESIGN FROZEN

Design revision: <revision or fingerprint of §§1–7 and §9 as approved>
Approved by / evidence: <operator approval reference>
Implementation base: mvp0/pr-01-entity-component-contracts @ <commit>
Execution contract: §9
Lifecycle: FROZEN
```

and start implementation in a **fresh session**, which re-reads this document in full before
its first edit.
