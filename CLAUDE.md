# CLAUDE.md — MineWorld operating instructions

This file is binding for every coding agent and every session in this repository. It is a
specification, not an introduction. Read it fully before the first edit of a session.

---

## 1. What this project is

> **MineWorld is an open-source framework for building persistent, modular living game worlds.**
>
> Worlds are composed from independent entities, simulation systems, controllers, and
> presentation layers rather than implemented as monolithic games.

```text
Simulation creates reality.
Controllers propose actions.
Presentation observes reality.
```

MineWorld is infrastructure, not a game and not a demo to fork. The extension model is
`install modules → compose world → configure → run`, never `fork source → edit game code`.

A world with every language model removed must remain a valid MineWorld world. If MineWorld
stops being meaningful once the models are unplugged, it has degenerated into an AI-NPC demo
and the architecture has failed.

Frozen top-level acceptance criterion for the whole project:

> **The framework must demonstrate that materially different games can be constructed by
> composing the same core entities with different independently installable interaction
> systems, without modifying the kernel.**

Authoritative specifications, in reading order:
[`docs/VISION.md`](docs/VISION.md) ·
[`docs/CORE_CONCEPTS.md`](docs/CORE_CONCEPTS.md) ·
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) ·
[`docs/MODULE_SPEC.md`](docs/MODULE_SPEC.md) ·
[`docs/NETWORKING.md`](docs/NETWORKING.md) ·
[`docs/MVP.md`](docs/MVP.md) ·
[`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) ·
[`docs/ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) ·
[`docs/REUSE_POLICY.md`](docs/REUSE_POLICY.md) ·
[`docs/DECISIONS.md`](docs/DECISIONS.md).

**Before modifying production code**, read [`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md)
along with `VISION`, `ARCHITECTURE`, `ENGINEERING_STANDARDS`, and the contracts of the module you
are touching. Those rules are mandatory unless an approved design document explicitly overrides
them.

## 1.1 The worlds are meant to be played

MineWorld builds **playable walking / exploration / travel-oriented worlds**. The player stands
inside the world:

```text
enter world → move through space → explore → enter places → encounter people
→ interact → participate in activities → travel elsewhere → keep living there
```

It is therefore none of these, and a design that drifts toward one has failed:

```text
a social-science simulator      an agent benchmark
an NPC chatbot framework        a backend world-state database
a dashboard for watching agents move
```

Agent simulation exists to make the world alive. It is not the product by itself.

Two reference clients are first-class and permanent — 2D for fast architectural validation, 3D
for the embodied experience the project exists to enable. Neither is a placeholder for the other
([`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) §§2–3, §10;
[`docs/MVP.md`](docs/MVP.md) §7).

Current repository state: **specification-only**. No kernel code exists yet. Do not invent
repository structure that is not there; audit before describing it.

---

## 2. Documentation law

Documentation is not an afterthought in this project. It is the reviewable artifact that
makes the architecture auditable, and it is written *before* the code it governs.

### 2.1 Two kinds of Markdown, and only two

| File | Audience | Requirement |
| --- | --- | --- |
| Any file named `README.md`, at any depth | **humans** | Short, clear, one obvious main line. Low reading burden. It orients a person and links onward; it does not carry the specification. |
| **Every other** `.md` file | **coding agents** | A specification. Complete, precise, unambiguous, terminologically exact, conceptually aligned with every other spec in the repository. Length is not a defect; vagueness is. |

Rules that follow from that split:

1. A `README.md` never becomes the authority for a design. If a reader must know a rule to
   implement correctly, the rule lives in a specification document, and the README links to
   it.
2. A specification document never sacrifices completeness for brevity. Do not trim, do not
   summarize away detail, do not replace a stated rule with a gesture at it.
3. Terminology is fixed across all documents. `Entity`, `Person`, `Place`, `Item`,
   `Organization`, `Relation`, `Process`, `ActionIntent`, `Event`, `System`, `Controller`,
   `World`, `Component`, `Observation`, `World Pack`, `System Pack`, `Entity Pack`,
   `Controller Pack`, `Presentation Pack`, `Kernel` mean exactly what
   [`docs/CORE_CONCEPTS.md`](docs/CORE_CONCEPTS.md) and
   [`docs/MODULE_SPEC.md`](docs/MODULE_SPEC.md) say they mean. Never introduce a synonym for
   a defined term, and never reuse a defined term for a different concept. Extending the
   ontology is a change to those documents, reviewed as such.
4. When code and specification disagree, that is a defect. Decide explicitly whether the
   code is wrong or the specification is stale, record the decision, and fix one of them.
   Never leave the contradiction unrecorded.
5. English is the authoritative language for all documentation and specifications. Chinese
   discussion in conversation is expected and fine; a Chinese file is only ever a mirror of
   an English original, named `*.zh-CN.md`, and a mirror may never introduce, relax, or
   amend a requirement.

### 2.2 Design before implementation

Every design and every plan that affects architecture must exist as a document before the
implementation starts, so that it can be reviewed afterwards. This is a hard requirement of
this project, not a style preference.

That includes: module boundaries and ownership, contract shapes, new systems, new pack
types, networking and persistence decisions, and any deviation from a previously agreed
design. "It is in the commit message" and "it is in the conversation" do not satisfy this.

---

## 3. Development process: Structured Coding

All substantial development in this repository runs through the Structured Coding workflow,
installed as a project skill at
[`.claude/skills/structured-coding/`](.claude/skills/structured-coding/) (upstream:
<https://github.com/yuema137/structured-coding>, installed version in
`.claude/skills/structured-coding/VERSION`).

Invoke the `structured-coding` skill and follow it. `SKILL.md` routes the required reading
for the active phase; read what the phase row lists, not a summary of it.

### 3.1 Phases and human checkpoints

```text
overall planning  →  step planning  →  PR design  →  DESIGN FROZEN
                                                        │
                                                        ▼
                                             fresh execution session
                                                        │
                                      implement · validate · review · commit
                                                        │
                                                        ▼
                                          READY FOR OPERATOR REVIEW
                                                        │
                                                        ▼
                                          operator authorizes merge
                                                        │
                                                        ▼
                                     update PR → step → overall documents
```

Human decisions stay at exactly four points: requirements, design freeze, material
deviation, and merge review. Everything inside an approved contract is autonomous:
investigate, implement, validate, review, commit, iterate, without asking for permission
already granted.

Specifically:

- A planning request does not authorize implementation. Loading the skill does not authorize
  a merge.
- Implementation requires a user-approved `DESIGN FROZEN` header and a filled execution
  contract. Never manufacture that approval, and never mark a design frozen because drafting
  finished.
- Each new PR starts in a fresh implementation session. A resumed session after compaction is
  the *same* PR: recover repository truth, re-read the PR design in full, then continue.
- Frozen means scope, invariants, and acceptance are frozen. Progress, evidence, audit
  findings, and bounded corrections stay writable and must be kept current.
- A bounded discovery is resolved autonomously and recorded. A change to a frozen invariant,
  a public contract, an ownership boundary, a material dependency, or scope stops the
  dependent action and goes back to the operator with evidence and the smallest reviewable
  revision.
- Every planned commit tracks implementation, deterministic validation, and LLM logic review
  as separate checkable items with evidence. One never completes another. `[x]` requires the
  work and the evidence; a genuinely inapplicable item is `N/A` with the audited reason.
- A test not run is not a pass. Classify results as `PASS`, `FAIL`, or `INCONCLUSIVE` from
  actual evidence, never from an exit code alone.

### 3.2 Where planning documents live — and they are tracked

```text
.structured-coding/plans/<effort>/overall.md
.structured-coding/plans/<effort>/step-NN-<name>.md
.structured-coding/plans/<effort>/pr-NNx-<name>.md
.structured-coding/plans/<effort>/pr-NNx-contract.md
.structured-coding/plans/<effort>/handoff.md
```

**Deliberate deviation from the skill's default:** the skill keeps planning documents out of
version control. MineWorld tracks `.structured-coding/plans/` in Git, because every design
and plan in this project must be reviewable later by someone who only has the repository.
`.gitignore` therefore excludes only `.structured-coding/standards.local.md` and session
state.

One location is the authority. Do not create competing tracking documents beside the PR
ledger; the contract may be a section of the PR design or a linked file, and the handoff is a
continuation aid, never a second design authority.

### 3.3 Project standards plugged into the skill

The machine-readable half of MineWorld's engineering policy is declared in
[`.structured-coding/standards.md`](.structured-coding/standards.md), which the skill reads
as the project's standards contract:

- `review.trigger` is `commit`: the review conventions are brought to attention at every
  semantic commit, not only at PR readiness.
- `review.conventions` carries the architectural and testing rules a reviewer or an LLM can
  judge on a diff.
- `checks.tools` declares `cargo fmt`, `cargo check`, `cargo clippy -D warnings`, and
  `cargo test` at repository scope, with `ruff` and `pyright` declared and disabled until the
  first Python module exists under `cognition/`.

That file is tracked in Git, which is what makes it the project's standard.
`.structured-coding/standards.local.md` is personal, untracked, and may only add or tighten.

The standards helper **reports; it never blocks**. Nothing in it prevents a commit or a
merge, and a clean report is not proof that a check ran — read the outcomes. Enforcement that
must actually block is CI's job (see [`docs/ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md)
§§15–16). Tools that execute project code need one deliberate approval:

```sh
python3 .claude/skills/structured-coding/scripts/standards.py inspect --project .
python3 .claude/skills/structured-coding/scripts/standards.py approve --project .
python3 .claude/skills/structured-coding/scripts/standards.py run     --project . --base main
```

No host hooks are registered. Do not register or enable the `continuity`, `checkpoints`, or
`standards` presets on your own initiative; if the operator enables one, read its interface
document and bind the session explicitly. Without hooks these obligations remain procedural,
and missing hooks never block authorized work. Report actual enforcement status accurately.

### 3.4 Scope of the workflow

The full workflow is for substantial work: a change spanning several commits, PRs, or
sessions. Do not impose overall/step/PR documents on a typo fix, a one-line correction, or a
question. The documentation law in §2 still applies to those.

---

## 4. Engineering standards are binding

[`docs/ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) is the complete, authoritative
engineering policy for all production code, plugins, infrastructure, tests, and AI-assisted
development. Read it before implementing. It is not advisory.

The non-negotiable subset, restated here so no session can claim not to have seen it — the
document itself governs in every case:

1. **Single ownership.** Every mutable piece of authoritative state has exactly one owning
   system. A system that needs another system's state to change emits an event; the owner
   decides. `EmploymentSystem` emits `WageDue`; only `EconomySystem` moves money.
2. **Kernel ignorance.** The kernel knows identity, components, time, spatial semantics,
   action dispatch, system registry, process scheduling, event sourcing, relationships,
   persistence, networking, permissions, and plugin lifecycle. It does not know what a job
   is, what money is, what romance is, or that people sleep. Those are System Packs.
3. **No leakage across layers.** Renderer concepts never enter simulation contracts.
   LM-provider concepts never enter cognition contracts. Controllers and clients submit
   `ActionIntent`s and never mutate world state. Single-player and multiplayer share one
   server-authoritative path.
4. **Dependency direction.** World Pack → Systems → kernel contracts, one way. A circular
   dependency is an architectural warning, not a detail to route around.
5. **Change amplification test.** Adding a capability should add a module plus contracts,
   registration, and integration tests — not edits to `Person`, unrelated systems, renderer
   internals, cognition internals, or scheduler logic. If it does, stop implementing and
   raise the architecture question.
6. **No God objects.** No `WorldManager`, `GameManager`, `GlobalState`, or type that knows
   rendering, networking, cognition, and domain logic at once.
7. **Strong typing at boundaries.** Named domain types, not `dict`, `map<string, any>`, JSON
   blobs, `Any`, or untyped metadata. Distinct identifier types do not silently interchange.
   Prefer models in which invalid states are hard to represent.
8. **Integration over unit count.** Integration, contract, scenario, and regression tests
   carry the weight. Never write a test whose only assertion is that a getter returns a
   field, a constructor assigns its arguments, an enum lists its variants, or a library works
   as documented. Test observable behavior and events, not internal helper calls.
9. **Real execution.** A feature is validated when it runs in a realistic integration path:
   create world → load system → act → emit → persist → restart → verify. Sample worlds are
   integration fixtures, not demos; major changes are evaluated by running them.
10. **Headless first, deterministic, LM-independent.** The authoritative simulation runs and
    is tested with no renderer. Randomness is seeded and reproducible. Core tests never
    require a live external model; use deterministic controllers or recorded cognition.
11. **No premature abstraction.** Simple implementation → observe the repeated concept →
    define the abstraction. The kernel stays small; complexity goes into optional systems.
12. **No early compatibility debt.** Before a public stable contract exists, a wrong
    interface is changed cleanly rather than wrapped in adapters.
13. **Agents do not optimize for volume.** Not files changed, not tests added, not
    abstractions introduced, not lines generated. Before substantial implementation:
    identify the owning module, the affected contracts, the integration points, whether an
    existing abstraction already covers this, and surface architectural holes instead of
    patching around them.
14. **Semantic space is not render geometry.** The simulation knows `Cafe.Counter`; a client
    knows `(x, y, z)`, a navmesh point, or an animation anchor. Continuous local movement and
    higher-level travel are two scales of one spatial model — never collapse movement into
    renderer-local physics alone, nor into teleportation between place names. `MoveIntent`,
    the travel `Process`, authoritative spatial state, and rendered movement stay four distinct
    things ([`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) §§5–6).
15. **Spatial requirements belong to System contracts, and clients only report intent.** An
    action declares whether it needs the same `Place`, an interaction radius, line of access, or
    an available target; `send message` and `apply for remote job` declare that they do not. A
    client detects "player pressed interact while targeting Alice" and sends an intent; the
    server answers with the resolved action or with `Unavailable`, `Busy`, `TooFarAway`,
    `PermissionDenied`, `NoSupportedInteraction`. A rule evaluated in a renderer is a defect
    ([`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) §§4, 7–9).

16. **Reuse before reinvention.** Before building any substantial infrastructure component,
    check whether a mature library already solves it, and prefer: adopt → adapt behind a
    MineWorld interface → extend → build our own. Never adopt a dependency merely because it
    exists, and never force MineWorld inside a framework's execution model — world state,
    system ownership, network authority, plugin contracts and persistence semantics stay under
    MineWorld's control. Commodity infrastructure (SQLite, HTTP/WebSocket, TLS, serialization,
    rendering, physics, navigation, containers, model serving) is reused, not rebuilt. Both
    adopting a substantial dependency **and** rejecting one in favour of our own code require a
    short record in [`docs/DECISIONS.md`](docs/DECISIONS.md); "writing it ourselves feels
    cleaner" is not a reason. Prototype before committing to anything large
    ([`docs/REUSE_POLICY.md`](docs/REUSE_POLICY.md)).

Two questions gate every spatial or interaction contract before it merges: can it support a
Minecraft-like embodied 3D client without redesigning the kernel, and can both the 2D and 3D
clients use the capability without duplicating game logic? A "no" to either sends the design
back ([`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) §§11–12, §22).

Every substantial design review also asks both halves of the reuse question: are we reinventing
a mature wheel, and are we forcing an existing wheel where it does not fit?

Size thresholds are review triggers, not mechanical rules: a function past ~50 lines is
questioned and past ~100 is a strong warning; a file past ~500 lines is reviewed and past
~800 is a strong warning. Do not shatter code into meaningless fragments to satisfy a
number — the goal is conceptual locality.

Stop and raise an architecture review when the symptoms in §30 of the standards appear
repeatedly. They are architectural evidence, not style complaints.

---

## 5. Implementation sequence

Commit order for the initial build-out. Each entry is at least one PR under §3.

```text
1  Vision + architecture + MVP specifications        (done)
2  Entity / Component contracts
3  ActionIntent / Event contracts
4  System interface
5  World clock + scheduler
6  Persistence
7  First trivial system
8  Headless demo
```

**No language model is required before commit 8, and none should be attached earlier.** An
LM attached early produces the illusion of progress — a charming NPC line — while the
infrastructure that the project actually exists to build is still missing.

Language choice follows §3 of the engineering standards: Rust by default for kernel,
systems, persistence, networking, and server; Python for cognition and LM integration;
engine-native code only inside presentation adapters; TypeScript only for web clients and
admin tooling.

---

## 6. Session start checklist

1. Read this file.
2. Invoke the `structured-coding` skill and identify the active phase from the request,
   `.structured-coding/plans/`, and repository state.
3. Inspect branch, HEAD, status, and recent history before concluding anything about
   progress.
4. Read the specifications that bind the work at hand — at minimum
   [`docs/CORE_CONCEPTS.md`](docs/CORE_CONCEPTS.md),
   [`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) and
   [`docs/ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) before touching kernel or
   system code.
5. Audit real source before specifying file-level or function-level work. Never invent
   structure.

Commits are semantic and scoped. Do not commit or push unless the active contract authorizes
it; never merge without explicit operator authorization.
