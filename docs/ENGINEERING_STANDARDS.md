# MineWorld Engineering Standards

**Status:** Project-wide engineering policy
**Applies to:** All production code, plugins, infrastructure, tests, and AI-assisted development in MineWorld.

The machine-readable subset of this policy is declared in
[`.structured-coding/standards.md`](../.structured-coding/standards.md), which the
`structured-coding` skill reads. That file never replaces this one: it carries the rules a
reviewer or a command can decide mechanically, and this document remains authoritative for
everything.

---

# 1. Purpose

MineWorld is intended to become a long-lived, modular infrastructure project rather than a one-off game prototype.

The primary engineering risk is not failure to implement individual features. It is gradual architectural decay:

- modules becoming tightly coupled;
- large files accumulating unrelated responsibilities;
- features requiring modifications across the entire repository;
- tests passing while the actual system does not run;
- LM-generated code adding layers of unnecessary abstraction or duplication;
- implementation details leaking across boundaries;
- renderer, simulation, networking, and cognition becoming inseparable.

The purpose of these standards is to prevent that outcome.

Our engineering priority is:

> **Maintain a modular, composable, understandable system whose components can be independently implemented, replaced, tested, and validated as the repository grows.**

Code quantity, test count, abstraction count, and feature count are not success metrics.

---

# 2. Core Engineering Principles

The following priorities apply across the entire repository:

1. **Modularity over convenience**
2. **Clear ownership over shared mutable state**
3. **Explicit contracts over implicit coupling**
4. **Integration behavior over superficial unit-test coverage**
5. **Real execution over mocked success**
6. **Strong typing over unstructured dictionaries**
7. **Simple architecture over speculative abstraction**
8. **Readable code over clever code**
9. **Replaceable modules over globally embedded implementations**
10. **Long-term maintainability over short-term patching**

A feature is not complete merely because it works once.

It is complete when it works **without damaging the architecture around it**.

---

# 3. Rust-First Policy

MineWorld should use **Rust by default** for long-lived infrastructure and deterministic world logic.

Rust is preferred for:

- simulation kernel;
- entity/component state;
- event processing;
- scheduling;
- world state;
- system interfaces;
- persistence;
- networking;
- server infrastructure;
- deterministic simulation;
- plugin runtime;
- protocol implementations;
- performance-sensitive systems.

This is a preference, not a requirement that every line of MineWorld must be Rust.

Other languages are appropriate where they provide a substantial ecosystem or development advantage.

Examples:

### Python

Appropriate for:

- LM integration;
- experimental cognition policies;
- ML/RL research;
- embeddings;
- model serving adapters;
- rapid research prototypes.

### GDScript / C# / C++

Appropriate inside renderer-specific adapters where required by Godot, Unreal, or other engines.

### TypeScript / JavaScript

Appropriate for:

- browser-based administration;
- world editors;
- dashboards;
- web clients.

---

# 4. Language Boundaries Must Be Explicit

Using multiple languages must not result in multiple implementations of world semantics.

The authoritative simulation remains independent of the language used by external modules.

Cross-language communication must occur through explicit contracts.

Conceptually:

```text
Python Cognition
      │
      │ ActionIntent
      ▼
Rust Kernel
      │
      │ World State Delta
      ▼
Godot Renderer
```

Python must not modify kernel persistence directly.

The renderer must not modify authoritative entity state directly.

External services must not bypass system interfaces.

Language choice is local.

World semantics are global.

---

# 5. Module Independence

Every substantial capability should belong to a clearly defined module.

Examples:

```text
kernel
systems/inventory
systems/employment
systems/conversation
systems/economy
cognition
networking
persistence
renderer/godot
```

A module should define:

- what it owns;
- what public interface it exposes;
- which other modules it depends on;
- which events/actions it consumes;
- which events/actions it produces;
- what invariants it guarantees.

A module should **not** reach inside another module's private implementation.

---

# 6. Dependency Direction

Dependencies should follow deliberate architectural layers.

For example:

```text
World Pack
    ↓
Systems
    ↓
Kernel Contracts
```

not:

```text
Kernel
 ↕
Renderer
 ↕
Employment
 ↕
LM Provider
 ↕
Inventory
```

Circular dependency is an architectural warning.

If two modules need extensive knowledge of each other's internals, the boundary is probably wrong.

---

# 7. Single Ownership of State

Every mutable piece of authoritative state must have a clear owner.

For example:

```text
EmploymentSystem
owns:
    EmploymentComponent
```

```text
EconomySystem
owns:
    AccountBalance
```

EmploymentSystem may emit:

```text
WageDue
```

It must not directly modify another system's bank balance.

The receiving system decides how that event affects its own state.

This principle allows systems to remain independently removable.

---

# 8. The Change Amplification Test

One of the most important architectural tests in MineWorld is:

> **How much existing code must change when a new feature is added?**

Suppose we add:

```text
UniversitySystem
```

A clean implementation should primarily require:

```text
new module
new contracts
registration/configuration
integration tests
```

It should **not** require editing:

```text
Person
InventorySystem
ConversationSystem
EmploymentSystem
Renderer internals
Cognition internals
Kernel scheduler logic
```

unless there is a genuine architectural reason.

If adding one feature requires modifications across many unrelated modules, stop implementation and reconsider the architecture.

This is not merely refactoring debt.

It is evidence that the abstraction boundary is wrong.

---

# 9. Avoid Central God Objects

MineWorld must not accumulate objects such as:

```text
WorldManager
GameManager
AgentManager
EverythingContext
GlobalState
```

that gradually become responsible for most project behavior.

Likewise, avoid classes or modules that know simultaneously about:

```text
rendering
networking
LM calls
inventory
employment
movement
persistence
```

Coordination components are allowed.

Business logic should remain in the module that owns it.

---

# 10. Code Size and Complexity

There should not be rigid rules requiring artificial splitting, but extreme size should trigger review.

Recommended warning thresholds:

### Functions

- ~50 lines: consider whether the function has multiple responsibilities.
- ~100 lines: strong architectural warning.
- hundreds of lines: normally unacceptable.

Exceptions include generated code, declarative mappings, parsers, or similarly justified structures.

### Files

- ~500 lines: review whether responsibilities should be separated.
- ~800 lines: strong warning.
- thousands of lines: unacceptable for normal handwritten production code.

A file should represent a coherent responsibility, not simply be a container for related-looking code.

Do not split code into dozens of meaningless 20-line files merely to satisfy a metric.

The goal is **conceptual locality**, not small files for their own sake.

---

# 11. Function Design

Functions should normally do one conceptual operation.

Prefer:

```text
validate_action
resolve_action
emit_events
apply_state_transition
```

over:

```text
handle_everything()
```

Deeply nested control flow should be avoided.

If understanding one function requires simultaneously understanding several unrelated systems, reconsider the design.

---

# 12. Strong Typing

Important domain concepts should have explicit types.

Avoid passing arbitrary:

```text
dict
map<string, any>
JSON blobs
Any
untyped metadata
```

across core module boundaries.

Prefer concepts such as:

```text
EntityId
PersonId
PlaceId
EventId
ActionId
WorldTime
SystemId
Money
RelationshipValue
```

where appropriate.

Different semantic concepts should not accidentally become interchangeable merely because both happen to be strings or integers.

---

# 13. Typed Contracts

All important boundaries should have explicit schemas.

Especially:

```text
Entity
Component
ActionIntent
Event
Process
Observation
System interface
Network message
Persistence record
Plugin interface
```

Contracts should be versioned where long-term compatibility matters.

Changes to core contracts require deliberate review.

---

# 14. Invalid States Should Be Difficult to Represent

Prefer architecture where impossible states are prevented structurally.

For example, avoid:

```text
employment_status = "working"
employer = None
```

if the type model can represent employment more cleanly.

Use Rust's enums, newtypes, ownership model, and type system where they materially clarify domain invariants.

Do not create complex type gymnastics merely for sophistication.

---

# 15. Automated Code Quality Checks

Automated checks are mandatory.

At minimum, every pull request should run:

```text
format checking
compilation/type checking
linting
core integration tests
critical regression tests
```

For Rust this should initially include:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Tests should then run through the project's normal test command.

The exact commands may evolve, but quality gates must remain automatic.

Code should not rely on reviewers noticing basic formatting, lint, or type problems manually.

---

# 16. CI Must Test the Product, Not Just the Source Tree

CI should eventually include several layers.

## Fast structural checks

Run on every change:

```text
format
lint
type/compile checks
schema validation
```

## Core integration tests

Run on every pull request.

Examples:

```text
create world
start server
load systems
create entities
perform actions
observe events
persist
reload
continue simulation
```

## Scenario tests

Run realistic MineWorld scenarios.

Examples:

```text
Social Café
Market Town
```

## Long-running stability tests

Can run separately from the fastest PR loop.

Examples:

```text
simulate 100 days
simulate many agents
restart server repeatedly
replay event log
```

The exact CI schedule may evolve.

The principle does not change:

> At least some tests must exercise MineWorld as an actual running system.

---

# 17. Integration Tests Are More Important Than Unit-Test Count

MineWorld deliberately does **not** optimize for maximum unit-test count or code coverage.

A repository with 20,000 trivial unit tests can still fail immediately when components are connected.

Testing priorities should be:

1. **Integration tests**
2. **Contract tests**
3. **Regression tests**
4. **End-to-end smoke tests**
5. Unit tests where they provide real information

Unit tests remain valuable for:

- nontrivial algorithms;
- tricky state transitions;
- parsing;
- serialization;
- invariants;
- boundary conditions;
- deterministic transformations.

But do not create tests merely to inflate coverage.

---

# 18. Do Not Test Trivialities

Usually avoid tests whose only purpose is verifying:

```text
a getter returns a field
a constructor assigns its arguments
an enum contains its declared values
Rust's type system behaves correctly
a third-party library performs its documented behavior
```

Such tests create maintenance burden while providing little information.

A test should have a meaningful failure mode.

Ask:

> If this test fails six months from now, will it tell us something important broke?

If not, reconsider whether it is useful.

---

# 19. Test Behavior, Not Implementation Details

Prefer:

> Transferring an item moves ownership from Alice to Bob and emits ItemTransferred.

over:

> internal helper `_set_owner()` was called once.

Tests should survive reasonable internal refactoring.

Public behavior and contracts matter more than implementation structure.

---

# 20. Integration-First Development

A feature is not considered validated until it runs in a realistic integration path.

For example, implementing:

```text
EmploymentSystem
```

should eventually be tested through something resembling:

```text
create world
→ load EmploymentSystem
→ create Person
→ create Organization
→ submit ApplyForJob ActionIntent
→ validate
→ resolve
→ emit EmploymentStarted
→ change schedule
→ persist
→ restart world
→ verify employment remains
```

Testing only:

```text
EmploymentSystem::new()
```

does not validate the feature.

---

# 21. Continuous Real-World Execution

MineWorld should maintain small runnable sample worlds throughout development.

At minimum:

```text
Social Café
Market Town
```

These are not only demos.

They are integration fixtures.

Major architectural changes should be evaluated by actually running these worlds.

Development decisions should be informed by observed behavior rather than solely by static reasoning.

---

# 22. Headless Testing Is First-Class

The authoritative simulation must always be testable without a renderer.

Example validation:

```bash
mineworld run worlds/market-town --headless
```

The framework should be able to:

```text
run simulation
execute actions
produce events
persist state
reload
replay
```

without Godot, Unreal, or any graphical interface.

This is essential for reproducibility and infrastructure testing.

## Test scratch

A headless test that writes files — a save, a copied or generated pack, a scratch repository — owns
them only for its own duration. The rules:

- A test makes its scratch through `mineworld-test-support`'s `Scratch` (the `scratch!` macro), never
  by joining a name onto `CARGO_TARGET_TMPDIR`, `std::env::temp_dir()` or a literal `/tmp` path.
  `scripts/check_scratch.py scan` reports any such line in a test. A test that cannot depend on the
  helper keeps a guard of its own with the same behaviour and is listed, with its reason, in that
  script's `EXEMPT` table (today one: the leaf crate `mineworld-packages`, whose structure test
  refuses any MineWorld dependency).
- A scratch lives at `<CARGO_TARGET_TMPDIR>/mineworld-scratch-<pid>/<name>`. Its last path component is
  exactly the name the test gives, because a World Pack's id is its directory's name.
- A scratch is removed when its guard is dropped, **whether the test passed or failed**.
  `MINEWORLD_KEEP_SCRATCH=failed` keeps the scratch of a test that panicked, `MINEWORLD_KEEP_SCRATCH=all`
  keeps every scratch; a kept scratch prints `[scratch] kept <path>` to stderr. Any other value is
  refused.
- A name is unique among the scratches alive in one process. Creating a name that is already alive
  panics and names it, so two tests can never share one save.
- Only the process that created a scratch removes it. A child process that receives a scratch path
  (a `harness = false` kill test) never owns it.
- A path an operator supplies to a test through an environment variable (`BODIES_YARD_SAVES`) is
  read, never removed.
- After a passing `cargo test --workspace`, `scripts/check_scratch.py left` reports nothing under
  `target/tmp` and no `mineworld-*` entry in the temporary directory.

The reasons are disk and isolation: before this rule a passing suite left about 16 GB of saves under
`target/tmp` (`.structured-coding/plans/mvp0/pr-test-hygiene.md` §2), and two tests sharing one save
failed with a locked database. The helper is our own code rather than the `tempfile` crate: `DEP-29`.

---

# 23. Deterministic Test Mode

Where randomness is used, test scenarios should support deterministic seeds.

Example:

```text
seed = 42
```

The same:

```text
initial state
inputs
system versions
seed
```

should produce the same deterministic simulation result, excluding explicitly external/non-deterministic controller calls.

LM outputs used in regression scenarios should be recordable and replayable.

---

# 24. LM Calls Must Not Make Tests Fragile

Core integration tests should not require live external LM APIs.

Use:

- recorded cognition results;
- deterministic test controllers;
- local fixtures;
- stub controllers where appropriate.

Separately, maintain optional live-model integration tests when useful.

MineWorld correctness must not depend on whether an external provider responds.

---

# 25. Bug Fixes Should Add Regression Coverage

For meaningful defects:

1. reproduce the problem;
2. add a test that fails for the bug;
3. fix the bug;
4. verify the test passes.

Prefer integration-level regression tests when the failure involved multiple components.

---

# 26. Coverage Is Diagnostic, Not a Target

Coverage tools may be used to discover completely untested areas.

There is no requirement to maximize a global percentage.

A high percentage does not prove system correctness.

A lower percentage with strong scenario and integration coverage may be substantially more useful.

---

# 27. Readability

Code should be understandable without reconstructing the author's thought process.

Prefer:

```text
clear names
small conceptual units
explicit contracts
local reasoning
straightforward control flow
```

Avoid:

```text
clever metaprogramming without need
deep generic abstractions
hidden global behavior
magic configuration
unexplained side effects
```

Comments should primarily explain:

> why something exists;

not merely repeat:

> what the next line does.

---

# 28. Avoid Premature Abstraction

MineWorld is intentionally modular, but modularity does not mean creating abstraction layers before a second implementation exists.

Prefer:

```text
simple implementation
→ identify repeated concept
→ define stable abstraction
```

over:

```text
invent hypothetical abstraction
→ build framework
→ discover no implementation needs it
```

The kernel should stay small.

Complexity belongs in optional systems whenever possible.

---

# 29. Avoid Compatibility Debt During Early Development

Before a public stable API exists, do not preserve poor interfaces merely because they already exist in the repository.

If an early contract is wrong:

> change it cleanly.

Do not accumulate adapters around bad design purely to avoid touching code.

Once MineWorld publishes stable public contracts, compatibility policy can become stricter.

---

# 30. Architecture Review Trigger

Stop feature implementation and perform an architecture review when any of the following happens repeatedly:

- a new feature modifies many unrelated modules;
- circular dependencies appear;
- one file becomes the default place for new logic;
- one system directly mutates another system's state;
- tests require extensive mocking of internals;
- renderer-specific concepts leak into world semantics;
- LM-provider-specific concepts leak into cognition contracts;
- local and cloud execution require separate world logic;
- multiple systems duplicate the same state;
- the same concept gains incompatible representations in different modules;
- integration tests become difficult to construct because components cannot run independently.

These are architectural symptoms, not merely code-style problems.

---

# 31. Definition of Done for a Feature

A nontrivial feature is complete when:

### Architecture

- ownership is clear;
- public contract is defined;
- dependencies respect architectural direction;
- unrelated existing modules do not require invasive changes.

### Implementation

- code is readable;
- types are explicit;
- no unnecessary duplication is introduced;
- files/functions remain conceptually scoped.

### Validation

- appropriate integration behavior is tested;
- meaningful edge cases are covered;
- existing sample worlds still run;
- relevant regression tests pass.

### Runtime

- the feature works in actual execution;
- persistence/restart behavior is validated where relevant;
- headless execution remains functional where relevant.

### Documentation

- extension points and important invariants are documented.

---

# 32. Pull Request Review Questions

Every substantial PR should be evaluated with these questions:

### Modularity

Could this capability have been implemented as a new module rather than modifying existing modules?

### Coupling

What other components now need to know this feature exists?

### Ownership

Which system owns every new piece of mutable state?

### Replaceability

Could this module be removed or replaced without rewriting unrelated systems?

### Contracts

Are interactions with other modules explicit and typed?

### Validation

Is the actual behavior exercised through integration testing?

### Execution

Has the relevant sample world or scenario actually been run?

### Complexity

Did this PR introduce abstractions that are not yet needed?

### Maintainability

Will the next similar feature be easier to add, or harder?

The last question is especially important.

> **Good infrastructure should make the next feature easier.**

If each new feature becomes harder to implement, architectural debt is accumulating.

---

# 33. AI-Assisted Coding Policy

Coding agents should follow the same engineering rules as human contributors.

They must not optimize for:

```text
number of files changed
number of tests added
number of abstractions introduced
amount of generated code
```

Before substantial implementation, coding agents should:

1. identify the owning module;
2. identify affected contracts;
3. identify integration points;
4. determine whether existing abstractions already support the feature;
5. flag architectural holes rather than silently patching around them.

If a requested feature requires broad invasive modification, the agent should surface this as an architecture concern.

Do not solve architectural problems with layers of glue code unless explicitly justified.

---

# 34. Repository Health Principle

As MineWorld grows, repository quality should be judged partly by the following practical test:

> Can a new contributor understand one module, modify it, test it, and validate it without understanding the entire repository?

If the answer increasingly becomes no, the architecture is degrading.

The long-term goal is:

```text
large repository
≠
large cognitive burden
```

A healthy MineWorld codebase should become larger primarily by adding **independent capability**, not by increasing global coupling.

---

# 35. Final Engineering Principle

MineWorld should always prefer:

> **a small, well-tested kernel plus independently composable systems**

over:

> **a feature-rich monolith with extensive but low-information test coverage.**

The strongest evidence that the architecture works is not a large test count.

It is this:

> **New worlds, systems, controllers, and renderers can be added without destabilizing or rewriting the infrastructure that already exists.**
