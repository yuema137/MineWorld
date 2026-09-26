# Step 01 / PR 01 — Entity and Component contracts

**Role:** combined **step and PR** document. Step S1 needs exactly one PR, so this single
document is the authority for both levels; there is no separate step file.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md)
## DESIGN FROZEN

```text
Design revision:         §§1–7 and §9 as committed in 39abfb3
Approved by / evidence:  operator approval 2026-09-25 — "Freeze，开始实现",
                         with prerequisite P-1 authorized in the same exchange
Implementation base:     branch mvp0/pr-01-entity-component-contracts,
                         created from main @ 39abfb3
Execution contract:      §9
Lifecycle:               READY FOR OPERATOR REVIEW — DO NOT MERGE
Implementation context:  CLOSED / AWAITING OPERATOR ACTION
Commits on the branch:   e040818  C1  workspace + identity
                         80fdf2c  C2  entity record and lifecycle
                         40e448e  C3  component model
                         868cdc3  C4  relation model
                         90bb755  C4b world time value types (added — §8.3)
                         C5       the documentation commit that carries this line
Final evidence:          §8.2 — §6's four commands clean at the final head, 32 tests passing
Outstanding for the operator:
                         review and merge authorization; the three stale statements listed as
                         follow-ups in §8.3 are updated at merge, not here
```

Frozen: scope (§1.1), non-goals (§1.2), invariants (§1.3), the integration checkpoint (§2),
test ownership (§3), the commit sequence (§7) and the execution contract (§9). Live and
writable: the §7 checkboxes, the §8 ledger, and bounded design corrections recorded there.

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

**P-1 — RESOLVED, and the original finding was wrong.**

The first audit concluded no toolchain existed, because `cargo` and `rustc` are not on the
`PATH` of a non-interactive shell. A direct inspection of `~/.cargo/bin` corrected it: a full
`rustup` installation is present and nothing needed to be installed.

```text
~/.cargo/bin/cargo   cargo 1.97.1 (c980f4866 2026-06-30)
~/.cargo/bin/rustc   rustc 1.97.1 (8bab26f4f 2026-07-14)
rustup show          stable-aarch64-apple-darwin (default)
components           rustc, cargo, rust-std, rustfmt, clippy, rust-docs
cargo fmt --version  rustfmt 1.9.0-stable (8bab26f4f6 2026-07-14)
cargo clippy         clippy 0.1.97 (8bab26f4f6 2026-07-14)
```

**Consequence for every implementation session:** `~/.cargo/bin` is not on the default
non-interactive `PATH` in this environment. Prefix each verification command with
`export PATH="$HOME/.cargo/bin:$PATH"`, or invoke the absolute path. A "command not found"
here means the `PATH` was not exported — it does not mean the toolchain is missing, and it is
never recorded as a failed check.

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
| **DD-11** | `rust-toolchain.toml` pins the exact observed version `1.97.1` with components `rustfmt` and `clippy`, not the floating `stable` channel. | A floating channel makes `clippy -D warnings` and the `trybuild` expected-output files (A-2) change under the project without a commit. An exact pin is what lets S13's CI reproduce a local result. Bumping it is then a visible, deliberate commit. |

---

# 6. Verification commands

```sh
export PATH="$HOME/.cargo/bin:$PATH"     # required: see P-1 in §4.2
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
- [x] Workspace `Cargo.toml` with `resolver = "2"`, shared `[workspace.package]` metadata and `[workspace.dependencies]` for `serde` and `thiserror`. Edition 2024, `rust-version = "1.97.1"`; `serde_json` and `trybuild` are declared in the same table so the crate's dev-dependencies also resolve through it.
- [x] `rust-toolchain.toml` pinning the observed stable channel with `rustfmt` and `clippy`. Pinned to exact `1.97.1` with `profile = "minimal"` (DD-11); rustup installed that named toolchain on the first `cargo` invocation in the repository.
- [x] `mineworld-contracts` crate with `#![forbid(unsafe_code)]` and a crate-level doc comment stating: no domain concepts, no hash-ordered collections, one erasure boundary. `contracts/src/lib.rs` states four rules (the third — one erasure boundary — is worded for the type that arrives in C3); `#![warn(missing_docs)]` was added so `-D warnings` makes an undocumented public contract item a hard error.
- [x] `EntityId` newtype: opaque integer, `Copy`, ordered, serde-transparent, no public arithmetic, no `Default`. `EntityId::from_raw` / `raw` only, documented as the kernel-allocation and persistence boundary.
- [x] `EntityKey`: validated constructor (restricted charset, bounded length, no leading or trailing separator), `TryFrom<&str>`, `Display`. Also `TryFrom<String>`, and `#[serde(try_from = "String")]` so deserialization runs the same validation.
- [x] `EntityType` enum: `Person`, `Place`, `Item`, `Organization`, exhaustive and serde-stable (`snake_case`, no `#[serde(other)]`, no `Default`).
- [x] Typed references per DD-3, with free conversion to `EntityId`, a checked constructor taking `(EntityId, EntityType)`, and **no** public unchecked constructor. `PersonId` / `PlaceId` / `ItemId` / `OrganizationId`, each with `ENTITY_TYPE`, `new`, `entity_id`, `From<_> for EntityId`; the wrapped field is private, which the compile-fail case pins.
- [x] `SystemId`, `ComponentTypeId`, `RelationTypeId`: validated slug newtypes sharing one documented character rule with `EntityKey`.
- [x] `EventId`, `ActionId`, `ProcessId`: opaque integer newtypes (DD-10).
- [x] `ContractError` enum with one variant per validation failure class, each carrying the offending value's description but not a formatted blob. Five variants; the identifier variants carry an `IdentifierKind` so one rule serves four identifier types without losing which one was rejected.
- [x] Resolve A-1: write the macro and the hand-written form for one type, keep the clearer one, record the comparison in §8. Resolved in favour of the hand-written form — see §8.4.

### Validation
- [x] Unit: `EntityKey` accepts the documented legal forms and rejects empty, over-long, illegal-character and separator-edge cases, each with the expected `ContractError` variant. `contracts/tests/identity.rs::entity_key_accepts_authored_names_and_the_length_boundary` (7 legal forms incl. the 64-byte boundary) and `::entity_key_rejects_malformed_names_with_a_named_error` (8 rejected forms incl. uppercase, space, `/`, non-ASCII, both separator edges, 65 bytes).
- [x] Unit: slug validation for `SystemId`, `ComponentTypeId`, `RelationTypeId` over the same boundary classes. `::declaration_names_share_the_rule_and_report_their_own_kind`, which also pins that each type reports *its own* `IdentifierKind` — the copy-paste failure class the hand-written form introduces.
- [x] Unit: checked typed-reference construction succeeds for the matching `EntityType` and fails with the expected error for every non-matching one. `::a_typed_reference_is_built_only_for_its_own_entity_type` — the full 4 × 4 table (4 accepted, 12 rejected).
- [x] Unit: serde round-trip for every type introduced, plus byte-stability on repeated serialization. `::opaque_identities_are_written_as_bare_integers`, `::authored_names_round_trip_as_validated_strings`, `::entity_type_has_stable_names_and_no_silent_fallback`, `::a_typed_reference_is_still_checked_when_it_is_read_back`, `::repeated_serialization_is_byte_identical`. Every expected value is a hand-written literal, including the wire shapes `7`, `"alice"`, `"person"` and `{"entity":3,"entity_type":"person"}`.
- [x] Compile-fail: `PlaceId` where `PersonId` is required (A-2). `contracts/tests/compile_fail/place_id_where_person_id_is_required.rs`; the `.stderr` pins `error[E0308] … expected 'PersonId', found 'PlaceId'`.
- [x] Compile-fail: constructing a typed reference from a raw `EntityId` without the checked constructor. `contracts/tests/compile_fail/typed_reference_without_the_check.rs`; the `.stderr` pins `error[E0423]: cannot initialize a tuple struct which contains private fields`.
- [x] Static: the four commands of §6. All four clean — evidence in §8.2.

### Review
- [x] Inspect every public item for a domain concept that should not be in this crate (`INV-12`). Enumerated the 20 public items (`grep -n '^pub ' contracts/src/*.rs`): four opaque id newtypes, four validated name newtypes, `EntityType` and its four variants, four typed references, `MAX_IDENTIFIER_LENGTH`, `ContractError`, `IdentifierKind`. `Person`/`Place`/`Item`/`Organization` are the kernel taxonomy of `CORE_CONCEPTS.md` §3, not domain semantics. A case-insensitive grep for `job|employ|money|wage|hunger|sleep|romance|conversation|health|mood|combat|skill` over `contracts/src/` hits exactly one line: the word `conversation` inside the prose listing the specification's own example identifier strings. No type, field or variant names a domain concept.
- [x] Confirm no `HashMap`/`HashSet` and no `Default` on an id type that would fabricate identity. `grep -rn 'HashMap\|HashSet' contracts/` matches only the crate-doc sentence that forbids them; `grep -rn 'Default' contracts/src/` matches only the doc sentence explaining why `EntityId` has none. No `#[derive(Default)]` anywhere. The prohibition is additionally mechanical — see MUTATION 1 in §8.2.
- [x] Confirm no test asserts a value produced by the code under test (test rules §25). Read `contracts/tests/identity.rs` line by line: every expectation is a literal (`ContractError` variants constructed in the test, JSON strings, integers, the input string itself). The only comparisons against constructor output are of the form "the value the constructor accepted equals the literal it was given", which is the no-silent-normalization contract, not a self-derived expectation.
- [x] Confirm the crate has no dependency pointing at `kernel/`, `systems/`, or anything outside `[workspace.dependencies]`. `contracts/Cargo.toml` declares exactly four dependencies, all `{ workspace = true }`: `serde`, `thiserror` and dev-only `serde_json`, `trybuild`. No `path` dependency exists anywhere in the workspace.

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
- [x] `Tag` validated newtype; `Tags` as a `BTreeSet<Tag>` wrapper with set semantics and deterministic iteration. `Tag` reuses the C1 identifier rule through the new `IdentifierKind::Tag`; `Tags` is `#[serde(transparent)]` over the set, so it is stored as a sorted array.
- [x] `LifecycleState` enum (`Active`, `Dormant`, `Destroyed`) with an explicit `can_transition_to` relation and no transition out of `Destroyed`. Self-transitions are refused too — a no-op reported as success would let a caller believe it moved an entity that never moved.
- [x] `Metadata`: authoring provenance only — source pack, source path, optional authoring note — documented as never gameplay-relevant. A plain record with public fields: it has no invariant to protect, and a struct literal already makes a missing field a compile error.
- [x] `Entity` record: `id`, `key`, `entity_type`, `tags`, `lifecycle`, `metadata`; constructed through a builder or explicit constructor that cannot produce a half-initialized value. `Entity::new(id, key, entity_type)` takes the three facts an entity cannot exist without and starts `Active`, untagged, unauthored; `with_tags` / `with_metadata` add the optional parts. Fields are private, with accessors and `transition_to` as the only mutation.
- [x] Document on the type that all semantics live in components (`INV-12`, `CORE_CONCEPTS.md` §4). Module documentation and the `Entity` doc comment both state it, and name what is deliberately absent: name, position, owner, inventory, health.

### Validation
- [x] Unit: legal transitions permitted, illegal ones rejected, `Destroyed` terminal. `contracts/tests/entity.rs::the_lifecycle_permits_exactly_the_documented_transitions` asserts the full 4-legal / 5-illegal table, and `::an_entity_moves_through_its_lifecycle_only_when_the_move_is_legal` adds that a refused transition leaves the entity unchanged.
- [x] Unit: `Tags` deduplicates, orders deterministically, and round-trips byte-identically. `::tags_are_a_set_with_one_fixed_order` builds the same set in two authoring orders, with a duplicate, and asserts equal values, the literal order `["cafe","furniture"]`, identical bytes, and that a duplicate in stored data is absorbed rather than rejected.
- [x] Unit: `Entity` serde round-trip preserving every field. `::an_entity_is_stored_as_its_documented_shape` pins the exact stored JSON of a plain and an authored entity; `::a_restored_entity_keeps_every_field_and_rejects_an_unknown_lifecycle` reads every field back and checks that `"archived"` and an invalid authored key are both hard errors.
- [x] Unit: `Metadata` is optional in the sense the specification requires — an entity authored without provenance is representable. Same two tests: the plain entity's stored form has no `metadata` key at all (`skip_serializing_if`), and it deserializes from a record that omits it.
- [x] Static: §6 commands. All four clean — evidence in §8.2.
- [x] *Added:* Compile-fail: `entity.lifecycle = LifecycleState::Destroyed` does not compile (`E0616`, private field), which makes the review item below executable instead of a reading.

### Review
- [x] Confirm `Entity` gained no gameplay field, and that adding one would be visibly wrong. The record has exactly the six fields `CORE_CONCEPTS.md` §3 lists — `id`, `key`, `entity_type`, `tags`, `lifecycle`, `metadata` — and the module documentation names the fields a reviewer should expect to be refused (name, position, owner, inventory, health), so a future diff adding one reads as a contradiction of the file it is in.
- [x] Confirm no lifecycle transition is reachable by mutating a public field directly. All six fields are private; the only mutating method on `Entity` is `transition_to`, which consults `can_transition_to` first. Pinned by the added compile-fail case. One path does bypass the check by design: `Deserialize` restores a stored record, including a `destroyed` one. That is restoration of a fact, not a transition, and it is the mechanism S5 needs; it cannot invent an illegal *state*, only re-read a state that was reached legally.
- [x] Confirm no test merely asserts that a constructor assigned its arguments (`ENGINEERING_STANDARDS.md` §18). The per-field assertions in `::a_restored_entity_keeps_every_field_and_rejects_an_unknown_lifecycle` read the fields *after a serialization round trip*, so what they pin is storage fidelity — a renamed or dropped field in the stored shape — not `new()` assigning its parameters. No test constructs a value and immediately reads the same value back out.

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
- [x] `ComponentSchemaVersion` newtype with ordering, so migrations in S5 can compare. `const fn new`, so it can be an associated constant.
- [x] `Component` trait with associated `COMPONENT_TYPE: ComponentTypeId`, `OWNER: SystemId`, `SCHEMA_VERSION`, and serde bounds; documented so that implementing it *is* the ownership declaration (DD-4). The constants are exactly those types, which required `ComponentTypeId::from_static` / `SystemId::from_static` — see the deviation in §8.3. The trait carries a doc example that is compiled as a doc-test.
- [x] `ComponentDeclaration`: the runtime-inspectable form of the same facts, derivable from any `Component` implementation, for S3's registry. `ComponentDeclaration::of::<C>()` is infallible, because the identifiers were checked when the declaring crate compiled, and `conflicts_with` is the ownership check S3 needs.
- [x] `ComponentRecord`: `(EntityId, ComponentTypeId, ComponentSchemaVersion, payload)` — the single documented erasure boundary (DD-5), with typed conversion to and from a `Component`. Built as `ComponentRecord<P = Vec<u8>>`: the label comes from the component type, the encoding of the payload is the persistence layer's choice. The "conversion" is split — see the deviation in §8.3 — into labelling (`new::<C>`) and the checked read (`payload_for::<C>`), with the codec left to the caller.
- [x] Two test-only component implementations owned by two different stub systems, used by this and later commits' tests. `Measured` (owner `first-stub`) and `Flagged` (owner `second-stub`) in `contracts/tests/component.rs`, plus `MeasuredV2` (same type, next schema version) and `MeasuredByTheWrongSystem` (same type, different owner) so that the version and conflict cases have something real to compare.

### Validation
- [x] Unit: a `Component` implementation's declaration reports the expected type id, owner and version. `contracts/tests/component.rs::a_declaration_reports_the_component_types_own_facts`, which also pins the declaration's stored shape, since a registry will persist it.
- [x] Unit: typed → `ComponentRecord` → typed round-trip is lossless. `::a_component_survives_erasure_and_comes_back_whole`.
- [x] Unit: decoding a `ComponentRecord` into the wrong component type fails with a named error rather than producing a default value. `::a_record_is_not_readable_as_another_component_type` — the check runs before the payload is ever handed over, so no decode can produce a plausible wrong value.
- [x] Unit: decoding a record whose schema version is newer than the target type fails explicitly (forward-incompatibility is an error, not a silent drop). `::a_schema_difference_is_reported_in_the_direction_it_points`, which covers both directions with distinct errors and shows that an outdated record's payload and version are still reachable for a migration.
- [x] Unit: two components with the same `ComponentTypeId` but different owners are detectable as a conflict at declaration level — the check S3 will rely on. `::two_owners_of_one_component_type_are_a_conflict`, including the three non-conflicts (same owner's next version, an unrelated type, a declaration against itself).
- [x] Static: §6 commands. All four clean — evidence in §8.2.
- [x] *Added:* Compile-fail: a `Component` implementation that omits `OWNER` does not compile (`E0046`, missing `OWNER`), and an illegal identifier literal fails while it compiles (`E0080`, the const-evaluated rule). The first makes the review item below executable; the second is the guarantee that made `from_static` worth adding.
- [x] *Added:* `::a_record_is_stored_as_its_documented_shape` pins the record's stored JSON, the shape a persistence layer will write to a row.

### Review
- [x] Confirm the erasure boundary exists in exactly one type and is documented as such. `grep -rn payload contracts/src/` outside `component.rs` and the crate docs returns one line, and it is a doc sentence about event payload *contracts* arriving in S2. `ComponentRecord`'s own documentation and rule 4 of the crate documentation both name it as the boundary and say why one is unavoidable.
- [x] Confirm nothing outside `ComponentRecord` accepts or returns an untyped payload. `grep -rnE '\bValue\b|Box<dyn|dyn Any|serde_json' contracts/src/` is empty: there is no dynamic value type in the crate at all, and `serde_json` is a dev-dependency, so no production path can even name one.
- [x] Confirm the trait cannot be implemented without stating an owner. Pinned by the added compile-fail case: the associated constant has no default, so `E0046` is the only possible outcome. Nothing in the trait can be defaulted into existence later without that case going red.
- [x] Confirm no component defined in this crate carries domain meaning; the two test components are named for their role in tests, not for a gameplay concept. `grep -rn 'impl Component' contracts/src/` finds no implementation in the crate at all — the only one in `src/` is inside a doc example. The test components are `Measured`, `Flagged`, `MeasuredV2` and `MeasuredByTheWrongSystem`, named for what they do in the test; their owners are `first-stub` and `second-stub`, which name no system a world would have.

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
- [x] `RelationTypeDeclaration`: id, owning `SystemId`, directedness, and the permitted endpoint `EntityType` sets. Built through `directed(..)` or `undirected(..)`, so directedness and the endpoint sets cannot disagree; the permitted sets are `EntityTypeSet`, which cannot be empty. Self-edges are part of the declaration (`SelfEdges`, `Forbidden` unless `permitting_self_edges()` says otherwise), which is how the failure table's "never left implicit" is satisfied.
- [x] `Relation`: the `(RelationTypeId, from, to)` triple per DD-6, with a validating constructor that checks endpoint types against the declaration. `Relation::between(&declaration, &from_entity, &to_entity)` takes whole entities, because reading the endpoint's type off the entity record is the only way the caller cannot misstate it.
- [x] Symmetric handling for an undirected relation type: a canonical ordering so that `(a, b)` and `(b, a)` are the same edge and serialize identically. Ordered by `EntityId`; an undirected declaration therefore carries one endpoint set, so canonicalization can never turn a permitted edge into a refused one.
- [x] Document that per-relation state is a component keyed by the triple and owned by the declaring system. Module documentation, first section, together with the consequence that there are no parallel edges of one type and what to do instead (model the connections as entities).

### Validation
- [x] Unit: a relation whose endpoints violate the declared types is rejected, naming the offending endpoint. `contracts/tests/relation.rs::an_endpoint_of_the_wrong_type_is_refused_by_the_end_that_refused_it` — both ends, each naming the end, the entity, its type and the permitted set.
- [x] Unit: an undirected relation constructed in both argument orders produces one identical canonical value. `::an_undirected_edge_is_canonical_in_both_construction_orders`, asserting equal values *and* equal bytes.
- [x] Unit: a directed relation distinguishes `(a, b)` from `(b, a)`. `::a_directed_edge_distinguishes_its_two_orders`, plus the reversed-pair rejection in the endpoint test.
- [x] Unit: every entity-type pair the specification lists (Person↔Person, Person↔Organization, Organization↔Place, Person↔Item, Place↔Place) is representable. `::every_entity_type_pair_the_specification_lists_is_representable`, one table over all five.
- [x] Unit: serde round-trip and byte-stability. The stored shapes of both `Relation` and `RelationTypeDeclaration` are asserted as literals (`::an_undirected_edge_is_canonical_in_both_construction_orders`, `::a_declaration_is_stored_as_its_documented_shape_and_cannot_permit_nothing`), the declaration round-trips, and an empty endpoint set is refused on the way in from storage too.
- [x] Static: §6 commands. All four clean — evidence in §8.2.
- [x] *Added:* `::a_self_edge_follows_the_declaration_and_nothing_else` — the failure table's self-edge case, in both declared directions.
- [x] *Added:* Compile-fail: a `Relation` cannot be built as a struct literal (`E0451`, private fields), which is what makes the canonical ordering unbypassable.

### Review
- [x] Confirm no relationship semantics (friend, employee, owns) are defined in this crate — only the machinery. `grep -rniE '(struct|enum|const|fn) +[a-z_]*(friend|employee|owns|trust|romance|parent)' contracts/src/` is empty. The words appear only in prose: the module documentation saying that friendship strength, trust and employment are *not* here, and two doc lines using `employee_of` to explain what "directed" means. The relation names in the tests are identifier strings, not types.
- [x] Confirm canonical ordering cannot be bypassed by direct field construction. All three fields of `Relation` are private and `between` is the only constructor; pinned by the added compile-fail case. One honest gap: `Deserialize` restores a stored triple without a declaration to canonicalize against, because the value does not know its own directedness. A relation was canonical when it was written, and re-validating stored relations against the registry's declarations is S3's job at load — recorded in §8.3 rather than papered over.
- [x] Confirm the triple-identity consequence (no parallel edges of one type) is documented where a reader will find it. The module documentation's second section is titled "One edge per type per ordered pair", states that the triple *is* the identity, and says what to do instead when two connections between the same entities must be distinguished.

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

## C4b — World time value types

> **Added during execution.** This block was not part of the frozen §7 sequence. It exists
> because §1.1's approved scope lists `WorldTime` and `SimDuration` and none of the five frozen
> commits implements them; the audit, the alternatives and the decision are recorded in §8.3
> under "the approved scope contains a line the frozen commit plan never implements". C1–C5 keep
> exactly the content §7 froze.

### Goal
Deliver the two time value types §1.1 approves, with nothing that belongs to the scheduler.

### Scope
```text
new  contracts/src/time.rs   WorldTime, SimDuration
edit contracts/src/lib.rs    module wiring
new  contracts/tests/time.rs ordering, signed difference, stored shape
```
Non-goals: no clock, no tick, no advancement, no calendar, no process scheduling — S4 owns all
of it (§1.2, D-6). Depends on: nothing; the types stand alone.

### Implementation
- [x] `WorldTime`: signed simulated seconds from a world epoch (DD-9), ordered, serde-transparent, with `EPOCH`.
- [x] `SimDuration`: a signed length of simulated seconds, ordered, serde-transparent, with `ZERO`.
- [x] `WorldTime::duration_since`, the one relation DD-9 names ("`SimDuration` for differences"), returning `None` rather than wrapping when the difference is inexpressible.
- [x] Documented on the module: no calendar (`INV-12` — a `chrono::DateTime` here would import the calendar the kernel must not know) and no scheduling.

### Validation
- [x] Unit: moments order chronologically, including before the epoch — the assertion an unsigned clock would fail. `contracts/tests/time.rs::moments_order_chronologically_including_before_the_epoch`.
- [x] Unit: the difference between two moments is signed in both directions and refuses an inexpressible one. `::the_difference_between_two_moments_is_signed`.
- [x] Unit: both types are stored as bare seconds. `::simulated_time_is_stored_as_bare_seconds`.
- [x] Static: §6 commands. All four clean — evidence in §8.2.

### Review
- [x] Confirm no calendar or clock concept entered the crate: `time.rs` has no notion of a day, a date, a weekday or a season, no dependency beyond `serde`, and nothing that advances a value.
- [x] Confirm the scheduler's decisions are still open: the module states that tick selection, process waking and simultaneity are the scheduler's contracts, so A-3 remains an S4 question rather than something this PR settled by accident.
- [x] Confirm no trivial test: each of the three pins a property that a plausible wrong implementation would break (unsigned clock, unsigned or wrapping difference, wrapped wire shape).

### Acceptance criteria
Both types exist, order correctly across the epoch, difference correctly in both directions, and
carry no calendar or scheduling semantics; §6 clean.

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
- [x] `docs/ARCHITECTURE.md` §13: change the contracts row to state Rust types as the source of truth with Protobuf introduced at the first cross-language boundary, referencing decision D-4 and the step that will do it. The row now names `mineworld-contracts`, and a new §13.1 records D-4 in full: what the previous row said, the route now taken, three reasons a later session must not reverse by accident, and the two binding consequences.
- [x] `contracts/README.md`: rewrite for what now exists, staying short and human-facing per the documentation law. One screen: what the crate is, the types that exist, what is still to come, the five rules in one sentence each pointing at the crate documentation, the two commands, and the compile-fail directory.
- [x] Final ledger entries in §8: evidence, decisions, deviations, and the resolution of A-1/A-2/A-3.

### Validation
- [x] Every relative link in the touched documents resolves. Checked mechanically: every relative Markdown link in `contracts/README.md` and `docs/ARCHITECTURE.md` was resolved against the filesystem — 12 links, all present, none missing.
- [x] No specification statement contradicts the code that now exists — verified by re-reading the amended sections against the crate's public API. §13.1's three claims about what Rust expresses and Protobuf does not are each realized in the crate (typed references with checked constructors; ownership as an associated constant on `Component`; one erasure boundary in `ComponentRecord`), and its statement that the payload encoding is left to the persistence layer matches `ComponentRecord<P = Vec<u8>>`. The README's type list was checked item by item against `lib.rs`'s re-exports.
- [x] Static: §6 commands once more at the final head (docs-only changes still shift the head; test rules §20). See "final verification" in §8.2.

### Review
- [x] Confirm `contracts/README.md` stayed human-facing and short, and did not become a specification. 38 lines, no normative statement that exists only there: the five rules are a one-line summary of the crate documentation, which is the authority, and the specification links point onward to `ARCHITECTURE.md` and `CORE_CONCEPTS.md`.
- [x] Confirm the D-4 amendment records the decision and its reason, not just the new state. §13.1 states what the row used to say, the decision, three reasons, and what follows from it — including that a later `.proto` is a mirror whose upkeep belongs to the change that adds it.
- [x] Confirm no other document drifted (`MODULE_SPEC.md` §9 versioning claims still hold). `MODULE_SPEC.md` §9 requires core contracts to be versioned where long-term compatibility matters and a pack that changes an owned component's schema to ship a migration: `ComponentSchemaVersion` plus the two directional schema errors are exactly the mechanism that claim needs, so §9 is satisfied rather than contradicted. Three *stale* statements were found elsewhere and deliberately not touched, because they describe the state of `main` and become false only on merge — listed as follow-ups in §8.3.

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
C1  DONE     workspace, pinned toolchain, identity module, errors, compile-fail harness
C2  DONE     entity record: Tag/Tags, LifecycleState, Metadata, Entity
C3  DONE     component model: Component trait, declaration, record, const-checked ids
C4  DONE     relation model: declaration, typed edge, canonical undirected ordering
C4b DONE     world time value types (added commit — see §8.3)
C5  DONE     ARCHITECTURE.md §13 + new §13.1 for D-4, contracts/README.md, this ledger
```

## 8.2 Evidence

Environment for every run below: macOS aarch64, `rustc`/`cargo` 1.97.1 from the toolchain
pinned in `rust-toolchain.toml`, `PATH` exported per P-1. No network dependency after the
initial dependency fetch. No background jobs.

```text
C1 — workspace and identity module           (evidence for commit "feat(contracts): …")
  cargo fmt --all --check                       PASS   <1s
      first run reported a diff in contracts/tests/identity.rs; `cargo fmt --all` applied it
      and the check is clean. Recorded because the check did fail once.
  cargo check --workspace --all-targets         PASS   <1s   (cold build of the workspace
      including dependencies: 4.8s)
  cargo clippy --workspace --all-targets
      --all-features -- -D warnings             PASS   <1s   0 warnings
  cargo test -p mineworld-contracts             PASS    2s
      lib unit tests                 0 (no in-crate tests; the contract tests are integration
                                        tests against the public API)
      tests/compile_fail.rs          1 passed   — harness covering 2 compile-fail cases
      tests/identity.rs              9 passed
      doc-tests                      0
      total                         10 passed, 0 failed, 0 ignored
```

```text
C2 — entity record and lifecycle          (evidence for commit "feat(contracts): … entity …")
  cargo fmt --all --check                       PASS   <1s
      first run reported a diff in contracts/src/ids.rs (the signature of
      `validate_identifier` after it became `pub(crate)`); `cargo fmt --all` applied it.
  cargo check --workspace --all-targets         PASS   <1s
  cargo clippy --workspace --all-targets
      --all-features -- -D warnings             PASS   <1s   0 warnings
  cargo test -p mineworld-contracts             PASS    2s
      tests/compile_fail.rs          1 passed   — harness now covering 3 compile-fail cases
      tests/entity.rs                6 passed
      tests/identity.rs              9 passed
      lib unit tests / doc-tests     0 / 0
      total                         16 passed, 0 failed, 0 ignored
```

```text
C3 — component model                      (evidence for commit "feat(contracts): … component …")
  cargo fmt --all --check                       PASS   <1s
  cargo check --workspace --all-targets         PASS    1s
  cargo clippy --workspace --all-targets
      --all-features -- -D warnings             PASS   <1s   0 warnings
  cargo test -p mineworld-contracts             PASS    4s
      tests/compile_fail.rs          1 passed   — harness covering 5 compile-fail cases, each
                                                  listed individually as ok with --nocapture
      tests/component.rs             6 passed
      tests/entity.rs                6 passed
      tests/identity.rs              9 passed
      doc-tests                      1 passed   — the `Component` example in the trait docs
      total                         23 passed, 0 failed, 0 ignored
  regression note
      the identifier-rule refactor (one `const fn` feeding both the const and the runtime path)
      left all 9 identity tests green without an edit, which is the evidence that the two paths
      judge the same rule.
```

```text
C4 — relation model                       (evidence for commit "feat(contracts): … relation …")
  cargo fmt --all --check                       PASS   <1s
  cargo check --workspace --all-targets         PASS   <1s
  cargo clippy --workspace --all-targets
      --all-features -- -D warnings             PASS   <1s   0 warnings
  cargo test -p mineworld-contracts             PASS    3s
      tests/compile_fail.rs          1 passed   — harness covering 6 compile-fail cases
      tests/component.rs             6 passed
      tests/entity.rs                6 passed
      tests/identity.rs              9 passed
      tests/relation.rs              6 passed
      doc-tests                      1 passed
      total                         29 passed, 0 failed, 0 ignored
```

```text
C4b — world time value types               (evidence for commit "feat(contracts): … time …")
  cargo fmt --all --check                       PASS   <1s
  cargo check --workspace --all-targets         PASS   <1s
  cargo clippy --workspace --all-targets
      --all-features -- -D warnings             PASS    1s   0 warnings
  cargo test -p mineworld-contracts             PASS    3s
      tests/compile_fail.rs          1 passed   — harness covering 6 compile-fail cases
      tests/component.rs             6 passed
      tests/entity.rs                6 passed
      tests/identity.rs              9 passed
      tests/relation.rs              6 passed
      tests/time.rs                  3 passed
      doc-tests                      1 passed
      total                         32 passed, 0 failed, 0 ignored
```

```text
C5 — specification synchronization        (documentation only; no code changed)
  link resolution                               PASS   12 relative links in the two touched
                                                       documents resolve on the filesystem
  final verification at the final head
      cargo fmt --all --check                   PASS
      cargo check --workspace --all-targets     PASS
      cargo clippy --workspace --all-targets
          --all-features -- -D warnings         PASS   0 warnings
      cargo test -p mineworld-contracts         PASS   32 passed, 0 failed, 0 ignored
                                                       (9 identity, 6 entity, 6 component,
                                                        6 relation, 3 time, 1 compile-fail
                                                        harness over 6 cases, 1 doc-test)
      total wall time for the four              ~4s
  head identity
      the four commands were run on the tree that became the C5 commit — the working tree was
      clean and identical to that commit's content — and re-run at the resulting HEAD after
      committing, with the same results. The exact final SHA is in the operator handoff and in
      `git log`, and no code, test or configuration file changed after the run.
```

```text
MUTATION 5 — is the canonical ordering of an undirected edge actually applied?
  mutation     replaced the direction-dependent ordering in `Relation::between` with
               `(from.id(), to.id())`
  expected     the undirected test fails; the directed test does not
  observed     `an_undirected_edge_is_canonical_in_both_construction_orders` FAILED at the
               equality of the two construction orders; the other 5 relation tests stayed green
  verdict      behavior-changing, and correctly localized — the directed behavior is unaffected
               by the mutation and its test says so
  cleanup      reverted; `cargo test` green again before the commit
```

```text
MUTATION 4 — does the ownership conflict check actually look at the owner?
  mutation     `conflicts_with` reduced to `self.component_type == other.component_type`
  expected     the conflict test fails on the non-conflict it asserts, not on the conflict
  observed     `two_owners_of_one_component_type_are_a_conflict` FAILED at
               `assertion failed: !owner.conflicts_with(&same_owner_next_version)`;
               5 passed, 1 failed
  verdict      behavior-changing — the test pins both halves of the check, so a version bump by
               the owning system cannot be mistaken for a second writer
  cleanup      reverted; `cargo test` green again before the commit
```

```text
MUTATION 3 — is `Destroyed` terminal because of a check, or because no test looks?
  mutation     added `(Self::Destroyed, Self::Active)` to `LifecycleState::can_transition_to`
  expected     the transition table test and the entity lifecycle test both fail
  observed     `the_lifecycle_permits_exactly_the_documented_transitions` FAILED at the
               assertion for `destroyed -> active`, and
               `an_entity_moves_through_its_lifecycle_only_when_the_move_is_legal` FAILED at
               the expected `IllegalLifecycleTransition`; 4 passed, 2 failed
  verdict      behavior-changing — the terminal state is pinned at both the state-machine and
               the entity level, and the other four tests correctly stayed green
  cleanup      reverted; `cargo test` green again before the commit
```

```text
MUTATION 1 — is the HashMap prohibition real, or only prose?
  mutation     added `pub fn mutation_probe() -> std::collections::HashMap<String, u8>` to
               contracts/src/lib.rs
  expected     clippy rejects it
  observed     2 × `error: use of a disallowed type 'std::collections::HashMap'`, each
               carrying the configured reason, and "build failed"; exit non-zero
  verdict      behavior-changing — DD-7 is enforced by clippy.toml, not by convention
  cleanup      reverted; `cargo clippy` clean again before the commit

MUTATION 2 — is the compile-fail claim load-bearing, or does it pass for an unrelated reason?
  mutation     `pub struct PersonId(EntityId)` → `pub struct PersonId(pub EntityId)`
  expected     the unchecked-construction case now compiles, so trybuild fails
  observed     `tests/compile_fail/typed_reference_without_the_check.rs ... error`,
               "Expected test case to fail to compile, but it succeeded.", test FAILED
  verdict      behavior-changing — the case pins the private field, which is the actual
               guarantee, and the sibling id-confusion case stayed green (so the two cases
               are independent)
  cleanup      reverted; `cargo test` green again before the commit
```

## 8.3 Findings, decisions, deviations
```text
FINDING (planning, 2026-09-25) — later corrected, kept as evidence
    First audit concluded no Rust toolchain existed.
SOURCE AUDIT
    cargo --version, rustc --version, rustup show → "command not found";
    find . -name Cargo.toml → no results.
WHY IT MATTERS
    Every acceptance criterion in this PR is a compiler or test observation.
DECISION AT THE TIME
    Raised as blocking prerequisite P-1 requiring operator authorization to
    install rustup.

CORRECTION (planning, 2026-09-25)
    The finding was wrong. `ls ~/.cargo/bin` shows a complete rustup
    installation: cargo/rustc 1.97.1, rustfmt 1.9.0, clippy 0.1.97, toolchain
    stable-aarch64-apple-darwin. Nothing was installed.
ROOT CAUSE
    ~/.cargo/bin is absent from the PATH of a non-interactive shell, and the
    audit inferred absence from an unresolved command name.
IMPLICATION
    Verification commands must export PATH="$HOME/.cargo/bin:$PATH" first.
    A bare "command not found" is a PATH defect in the session, never a
    failed check and never INCONCLUSIVE evidence about the code.
DECISION
    P-1 closed as satisfied without installation. DD-11 pins the observed
    exact version 1.97.1 in rust-toolchain.toml rather than floating stable.
```

```text
FINDING (C1) — the pinned toolchain was not the installed toolchain
    DD-11 pins the exact version 1.97.1, but only the `stable` alias was installed
    (`rustup toolchain list` → `stable-aarch64-apple-darwin` alone), so the first cargo
    invocation in the repository made rustup install the named toolchain
    `1.97.1-aarch64-apple-darwin`.
SOURCE AUDIT
    rustup toolchain list; rustup show; the first `cargo check` printed
    "syncing channel updates for 1.97.1-aarch64-apple-darwin … downloading 5 components".
WHY IT MATTERS
    A frozen decision caused an unplanned environment change on the operator's machine.
DECISION
    Kept DD-11: `profile = "minimal"` limits the download to the components the checks need,
    the install completed within the first check (total 11s including the cold build), and the
    reproducibility DD-11 buys is the reason it exists. Recorded so the cost is visible.

DEVIATION (C1) — `clippy.toml` added at the workspace root, which C1's file list did not name
    Reason: the frozen invariant "no HashMap/HashSet in this crate" was otherwise only prose,
    and §1.3 is a review-enforced rule that a lint can enforce mechanically.
    Source evidence: `clippy::disallowed_types` is configuration-driven and is a warn-level
    lint, so with `-D warnings` (already in §6) a violation fails the build.
    Impact: one new 8-line file; no change to any contract type or public API.
    Validation: MUTATION 1 in §8.2 — the guard actually fires, with the configured reason.

DEVIATION (C1) — typed entity references serialize as a tagged object, not as a bare integer
    Previous assumption:
        DD-3 describes the checked constructor as the only way to build a typed reference, and
        says nothing about the serialized shape.
    Audit evidence:
        A typed reference whose serialized form is a bare integer can be deserialized from any
        integer, because `Deserialize` has no type information to check. Adversarial criterion
        2 in §2 ("a typed entity reference can be produced from an arbitrary EntityId with no
        type check") would then hold in code but fail at the first round trip through the
        event log, which is exactly the boundary S5 reconstructs state from.
    Corrected understanding:
        The reference must carry the `EntityType` it was checked against into its serialized
        form, so that deserializing is the same checked construction as building one.
    Implementation consequence:
        A private `TypedEntityRef { entity, entity_type }` is the serde representation of all
        four references (`#[serde(into = …, try_from = …)]`); the wire shape is
        `{"entity":3,"entity_type":"person"}`. Cost: one redundant tag per reference.
    Validation consequence:
        `::a_typed_reference_is_still_checked_when_it_is_read_back` asserts the exact wire
        shape and that a `place`-tagged value is rejected as an entity-type mismatch.

FINDING (C1) — `contracts/src/ids.rs` is 688 lines, past the §10 review threshold
    SOURCE AUDIT
        wc -l contracts/src/ids.rs → 688; the three sections are opaque identity (≈100),
        authoring identity (≈200), entity type and typed references (≈290). Roughly 330 of
        those lines are eight near-identical newtype blocks.
    WHY IT MATTERS
        ENGINEERING_STANDARDS.md §10 makes ~500 lines a review trigger (~800 a strong
        warning), and CLAUDE.md §4 forbids shattering code to satisfy a number.
    DECISION
        Reviewed and kept as one file. It has one responsibility — the identity vocabulary —
        and the frozen C1 scope names `contracts/src/ids.rs` as its home; length comes from
        mandated per-item documentation and from A-1's hand-written form, not from mixed
        responsibilities. Revisit if a later step pushes it toward 800 lines.

DECISION (C1) — `ContractError` is deliberately not `serde`-serializable
    The §1.3 invariant "every contract type is serde-serializable" exists so that S5 can
    reconstruct state from the event log. A validation error is not state: it is never stored,
    never replayed and never crosses the persistence boundary. Adding serde to it would invite
    exactly that. Recorded so the omission reads as a decision rather than an oversight.

FINDING (C1) — the approved scope contains a line the frozen commit plan never implements
    SOURCE AUDIT
        §1.1 "Approved scope" lists, between the id family and the entity record:
            time            WorldTime, SimDuration                (types only, no scheduling)
        §7's five commits are workspace+identity (C1, whose non-goals say "no time"), entity
        record (C2), component model (C3), relation model (C4) and documentation (C5). None of
        them mentions `WorldTime` or `SimDuration`. §9's APPROVED SCOPE paraphrases the crate
        as "(identity, entity record, component model, relation model)", also without time.
        §1.2's non-goal for time is narrower than the scope line: it excludes "scheduler,
        processes, WorldTime arithmetic policy → S4", which presupposes that the type exists.
    WHY IT MATTERS
        Shipping the PR without the two types leaves the PR short of its own approved scope,
        and S2 (events, which are timestamped) would have to invent them. Adding them to an
        existing commit would contradict that commit's stated non-goals (C1) or mix two
        concepts (C2–C4). Leaving them out silently would make §1.1 stale on the day the PR
        merges, which `CLAUDE.md` §2.1 rule 4 calls a defect.
    DECISION
        Deliver them in one additional, independently reviewable commit — **C4b — world time
        value types** — placed after C4 and before the documentation commit, so that C1…C4 and
        C5 keep exactly the content §7 froze. Nothing in §1.3's invariants, in any public type
        already committed, or in the ownership boundaries changes; C4b adds two value types
        with ordering, serde and no policy, as §1.1 words it. The alternative — stopping the
        PR for an operator ruling on a two-type gap that §1.1 already approves — was rejected
        as disproportionate, but the deviation is recorded here and in the review handoff so
        the operator can reverse it in review.
    VALIDATION
        C4b's own checklist and evidence, recorded in §7 and §8.2 like every other commit.

FOLLOW-UP (C5) — three statements elsewhere become stale on merge, and were left alone
    Each says the repository contains no Rust code. All three are true of `main` and false the
    moment this branch merges, so they belong to the merge/post-merge synchronization rather
    than to this PR, whose C5 non-goals forbid other specification edits:
        README.md:35                  "Specification only — no kernel code yet. The next three
                                       PRs define the contracts."
        CLAUDE.md:43                  "Current repository state: specification-only. No kernel
                                       code exists yet."
        .structured-coding/standards.md:15
                                      "No Rust crate and no Python package exists yet …"
                                      and, below it, that the cargo checks "become meaningful
                                      the moment kernel/ gains a Cargo.toml" and until then
                                      "report INCONCLUSIVE, never PASS" — now satisfied by
                                      contracts/Cargo.toml instead of kernel/.
    Recommended at merge: update all three in one commit, and re-word the standards note to say
    the cargo checks are live.

FOLLOW-UP (C5) — P-2 is still open and still non-blocking
    The standards helper will not run the declared cargo checks until
    `standards.py approve --project .` is run once. The same four commands were run directly
    throughout this PR, which is what §6 specifies, so nothing here depended on the helper. The
    approval is an operator action whenever the helper's report is wanted.

DEVIATION (C4) — `RelationKey` was not created; `Relation` is the key
    C4's scope line names three types: `RelationTypeDeclaration`, `Relation`, `RelationKey`.
    The implementation checklist names only the first two and the canonical ordering. DD-6 says
    the triple *is* the identity and there is no relation id, so a separate key type would be
    either a duplicate of `Relation` or a second identity for one edge — the thing DD-6 rules
    out. `Relation` is therefore documented as being what per-relation component state is keyed
    by, and no `RelationKey` exists. If the intent was a two-field `(from, to)` pair, that is
    not a relation's identity either, because the relation type is part of it.

FINDING (C4) — a stored relation cannot re-canonicalize itself on load
    A `Relation` carries no directedness: that lives in the declaration, which the value has no
    reference to. `Deserialize` therefore restores the triple as written. Every relation written
    by `Relation::between` is already canonical, so this matters only for data that was edited
    by hand or produced by a different version. The honest boundary is that validating stored
    relations against the registry's declarations belongs to the load path in S3, alongside the
    same check for component declarations; the alternative — making `Relation` deserialization
    require a declaration — would mean no plain `Deserialize`, and the persistence layer could
    not read a relation without first resolving its type.

DEVIATION (C3) — the component payload is a type parameter, and the codec is not in this crate
    Previous assumption:
        C3's checklist says `ComponentRecord` has "typed conversion to and from a `Component`",
        and its edge-case table expects a named error for "payload that is not valid for the
        declared type". Both presuppose that this crate can encode and decode a payload.
    Audit evidence:
        Encoding requires a format. DD-8 freezes the dependency list at `serde` + `thiserror`,
        with `serde_json` dev-only, so no encoder exists in the crate; `serde` alone provides
        traits, not a format. Promoting `serde_json` to a runtime dependency would (a) be a new
        material dependency, which §7 of the working rules says to stop for, and (b) choose JSON
        as the encoding of every component payload in the kernel's vocabulary — pre-empting
        decision D-4, which says the cross-language wire format is introduced at the first real
        cross-language boundary (S10/S11) and mirrored from these types.
    Corrected understanding:
        Erasure is unavoidable at a storage boundary, but *choosing the encoding* is not this
        layer's job. The record therefore carries `P`, the encoded form, as a type parameter
        defaulted to `Vec<u8>`, and the contract layer owns the part that is genuinely its own:
        the label cannot lie, and the payload is not handed to the wrong reader.
    Implementation consequence:
        `ComponentRecord<P = Vec<u8>>`. `ComponentRecord::new::<C>(entity, payload)` takes the
        component type and schema version from `C`, so a record cannot be mislabelled;
        `payload_for::<C>()` returns the payload only for the right type at the right version,
        with three distinct errors; `payload()` is the unchecked read a store uses to move a
        record it does not interpret. The edge case "payload not valid for the declared type"
        belongs to whoever decodes, which is S5 — recorded here rather than dropped.
    Validation consequence:
        `contracts/tests/component.rs` stands in for that caller: it encodes with `serde_json`
        (dev-dependency), and the checks the contract layer owns are tested directly, including
        both schema directions and the wrong-type read. MUTATION 4 in §8.2 shows the ownership
        check is load-bearing.

DEVIATION (C3) — `contracts/src/ids.rs` was changed, although C3's scope names only
`component.rs` and `lib.rs`
    Reason: DD-4 says a `Component` implementation "states its `ComponentTypeId`, its owning
    `SystemId`, and its schema version". Those are associated constants, and Rust cannot build a
    validated `String` in a constant, so with C1's `String`-backed identifiers the trait could
    only have carried `&'static str` — which is not what DD-4 says, and would have moved
    validation to load time.
    Source evidence: `const` initializers may call a `const fn`; a `const fn` that panics is a
    compile error at the site that evaluates it (const panic, stable since 1.57). `Cow::Borrowed`
    is const-constructible, and `Cow<str>`'s `PartialEq`, `Ord` and `Hash` all compare by
    content, so a borrowed and an owned identifier of the same text are the same value.
    Impact: `SystemId` and `ComponentTypeId` are now `Cow<'static, str>`-backed and gained
    `const fn from_static`. The identifier rule moved into one `const fn check_identifier`, with
    the runtime path converting its verdict into the rich `ContractError` — one rule, two
    entry points, no second implementation to drift. `EntityKey`, `RelationTypeId` and `Tag`
    were deliberately left alone: they name authored data, not code, so nothing declares them as
    a literal.
    Validation: all 9 pre-existing identity tests passed unchanged after the refactor, and the
    added compile-fail case proves an illegal literal is rejected at compile time
    (`error[E0080]: evaluation panicked: a system id literal must be 1 to 64 bytes …`).

CORRECTION (C3) — a C1 checklist note was inaccurate
    C1's third implementation item claims the crate documentation states "one erasure boundary".
    It did not: the C1 crate documentation had four rules and none of them was that one, and the
    note I wrote against the item said the rule was "worded for the type that arrives in C3",
    which overstated what was there. C3 adds it as rule 4 of five, now that `ComponentRecord`
    exists to point at. Kept as a correction rather than an edit of the C1 note, because a
    reviewer reading the C1 commit should know the rule was not yet in it.

DECISION (C2) — `Metadata`'s provenance fields are `String`, not a new identity type
    QUESTION
        `source_pack` names a pack, and packs have declared ids (`docs/MODULE_SPEC.md` §9), so
        ENGINEERING_STANDARDS.md §12 argues for a `PackId` newtype.
    AUDIT
        §1.1's id family is closed: EntityId, EntityKey, EntityType, typed references,
        SystemId, ComponentTypeId, RelationTypeId, EventId, ActionId, ProcessId. `PackId`
        appears nowhere in this PR's scope, and nothing else in the crate would use it.
    DECISION
        Keep `String` for this PR. Introducing a public identity type outside the approved
        scope is the kind of change §7 of the working rules says to stop for, and inventing it
        before pack loading is a contract would be premature abstraction
        (ENGINEERING_STANDARDS.md §28). The type documents that these are never interpreted:
        nothing in the simulation may branch on them, and a system that needs to know something
        about an entity reads a component or a tag. When pack loading becomes a contract,
        `source_pack` becomes that type — a clean change while nothing is persisted
        (§29 of the standards).
    NOT A BLOB
        This is a named record with three named text fields, not `map<string, any>`: the thing
        §12 forbids is an untyped bag, and `Metadata` cannot grow one.

DECISION (C2) — self-transitions in the lifecycle are refused
    `CORE_CONCEPTS.md` §3 lists the states but not the transition table, so the table is this
    PR's to define. `Active -> Active` is refused along with everything out of `Destroyed`: a
    lifecycle change that changes nothing is a caller's mistake, and returning `Ok` for it
    would hide the mistake at exactly the place S3 will decide whether to stop simulating an
    entity. Recorded because it is a contract a later step will rely on, not an implementation
    detail.

DECISION (C1) — `Cargo.lock` is committed
    MineWorld's deliverable is a server and a set of binaries, not a published library, and
    DD-11 pins the toolchain for exactly the reproducibility reason that argues for pinning
    the dependency graph too. `.gitignore` does not exclude it.

DECISION (C1) — `#![warn(missing_docs)]` on the crate
    With §6's `-D warnings`, an undocumented public item in the contract layer now fails the
    build. A contract crate whose audience is other coding agents cannot afford an
    undocumented public type.
```

## 8.4 Resolution of unresolved assumptions
```text
A-1  RESOLVED in C1 — hand-written newtypes; the macro was written, compiled and abandoned.

     Evidence. Both forms exist and both compile. The hand-written form is what is
     committed (contracts/src/ids.rs, 244 lines for the four references). The macro form was
     built as a standalone copy of the crate in the session scratchpad
     (a1-experiment, `cargo check` clean): a `typed_entity_reference!($name => $variant)`
     rule plus four one-line invocations, 73 lines in total, of which the macro arm — the
     code expanded per type — is 60 lines.

     Decision. A-1's own guard is "the macro stays under ten lines of expansion per type or
     it is abandoned". Measured expansion is ~55 code lines per type, five times the
     threshold, so the guard decides it: abandoned. The secondary criterion agrees for a
     weaker reason — the macro can only give the four references one templated doc comment
     built with `concat!`, whereas the hand-written form documents each reference and its
     constructor in place, which is what a reader of a contract crate is looking for.

     Cost accepted. ~170 more lines and eight near-identical blocks in one module. The
     failure class this introduces — a copy-pasted constant or `IdentifierKind` in one of the
     eight — is owned by two tests that check all four of each exhaustively
     (`::a_typed_reference_is_built_only_for_its_own_entity_type`,
     `::declaration_names_share_the_rule_and_report_their_own_kind`).

A-2  RESOLVED in C1 — trybuild is viable, and no downgrade was needed.

     Both compile-fail cases fail for the intended reason, and the generated `.stderr` files
     pin that reason rather than "some error":
         place_id_where_person_id_is_required.stderr
             error[E0308]: mismatched types … expected `PersonId`, found `PlaceId`
         typed_reference_without_the_check.stderr
             error[E0423]: cannot initialize a tuple struct which contains private fields
             note: constructor is not visible here due to private fields
     Load-bearing: MUTATION 2 in §8.2 makes the second case compile by publishing the field,
     and trybuild then fails. Brittleness across toolchains is bounded by DD-11's exact pin;
     the fallback described in A-2 was not needed and is not in effect.

A-3  open — carried to S4 (D-6). Untouched by C1, whose non-goals exclude time. The types
     themselves are delivered by C4b (see the scope-reconciliation finding in §8.3) as values
     with an ordering and no scheduling policy, which is all §1.1 asks for.
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
  branch: mvp0/pr-01-entity-component-contracts, created from main @ 39abfb3
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
  - toolchain installation: not needed — P-1 satisfied, rustup 1.97.1 present

REQUIRED LIVE DOCUMENTATION:
  this document, §7 checkboxes and §8 ledger

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff.md   (initialized at execution start)

PR CONTEXT SCOPE:
  This context belongs to PR 01 only. It begins when the operator authorizes
  implementation and ends at READY FOR OPERATOR REVIEW.

ENDPOINT AUTHORITY:
  - implementation + local validation:  authorized
      source: operator approval 2026-09-25 — "Freeze，开始实现"
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

# 10. Approval history

```text
2026-09-25  design presented for freeze, with P-1 raised as a blocking prerequisite
2026-09-25  operator approved the freeze and authorized P-1
2026-09-25  P-1 closed as already satisfied (see §8.3); design frozen at 39abfb3
2026-09-25  implementation session executed C1 → C2 → C3 → C4 → C4b → C5 on branch
            mvp0/pr-01-entity-component-contracts; §7 checkboxes, §8 ledger and the handoff
            synchronized; lifecycle set to READY FOR OPERATOR REVIEW. Not merged, not pushed
            (decision D-9: local-only until S13).
```

The operator's decisions still outstanding: whether to accept the two execution-time deviations
that touch the shape of this PR — the added C4b commit and the component payload being a type
parameter rather than an encoded value this crate can decode (both in §8.3) — and whether to
merge.

Implementation runs in a session separate from the planning session that wrote this document,
and that session re-reads this document in full, plus the working rules and test rules, before
its first edit. Merge into `main` remains gated on explicit operator authorization.
