# Reuse Before Reinvention

**Status:** mandatory policy for dependency and build-versus-adopt decisions
**Audience:** coding agents and contributors.

Companion to [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) (what to read before touching
production code) and [`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) (the complete
engineering policy). Where this document speaks about dependencies, it governs. Decision records
required by §11 and §12 live in [`DECISIONS.md`](DECISIONS.md).

---

MineWorld should not reimplement mature infrastructure unnecessarily.

Before implementing a substantial technical capability from scratch, first investigate whether a mature open-source library, framework, protocol, engine component, or existing implementation already solves the problem sufficiently well.

The default decision order is:

```text
use an existing mature solution
        ↓
adapt it behind a MineWorld-owned interface
        ↓
extend it if the extension remains clean
        ↓
implement our own solution only when necessary
```

However:

> **Do not use a dependency merely because it exists.**

The goal is not maximum dependency reuse.

The goal is:

> **the simplest, fastest, most maintainable solution that satisfies MineWorld's architecture and long-term requirements.**

---

## 1. Research Before Building

Before implementing a nontrivial infrastructure component, briefly search for existing solutions.

Examples include:

```text
ECS
physics
pathfinding
navigation
network transport
serialization
RPC
database access
event sourcing
plugin runtimes
WASM sandboxing
authentication
asset loading
3D character controllers
2D movement
Godot integration
LLM serving
embedding/vector search
job scheduling
configuration
observability
```

Do not spend days designing a custom subsystem before checking whether an established solution already exists.

When useful, inspect:

```text
official documentation
GitHub repositories
package registries
existing game engines
Rust crates
relevant standards/protocols
production usage
maintenance activity
licenses
```

Prefer primary sources and maintained upstream repositories when evaluating dependencies.

---

## 2. Evaluate Fit, Not Popularity

Do not choose a framework solely because:

```text
it has many GitHub stars
it is trendy
another AI project uses it
it supports many features
it looks architecturally sophisticated
```

Evaluate whether it fits **MineWorld's actual requirements**.

For any significant dependency, consider:

### Architectural fit

Does it preserve:

```text
modularity
renderer independence
server authority
headless execution
local/cloud parity
typed contracts
system ownership
controller independence
```

?

### Scope fit

Are we using the library for the problem it was designed to solve?

Or are we forcing MineWorld into the library's architecture?

### Replaceability

Can it sit behind a MineWorld-owned interface?

Could we replace it later without rewriting unrelated systems?

### Maturity

Check:

```text
maintenance activity
release history
documentation
test quality
real-world usage
known limitations
```

### License

The dependency must have a license compatible with MineWorld and its intended open-source distribution.

Record significant licensing constraints.

### Operational burden

Consider:

```text
extra services
databases
runtime dependencies
GPU requirements
deployment complexity
cloud-only assumptions
configuration burden
```

A technically powerful framework may still be the wrong choice if it dramatically complicates local hosting.

### Performance

Use evidence appropriate to the problem.

Do not prematurely optimize, but avoid clearly unsuitable tools for core hot paths.

---

## 3. Prefer Libraries Over Framework Lock-In

When possible, prefer a focused library that solves one problem cleanly over a framework that requires MineWorld to adopt its entire execution model.

Prefer:

```text
MineWorld architecture
        +
focused dependency
```

over:

```text
third-party framework architecture
        ↓
MineWorld forced inside it
```

This is especially important for:

```text
simulation
world state
network authority
plugin contracts
persistence semantics
entity ownership
```

These are central MineWorld concepts and should remain under MineWorld's architectural control.

---

## 4. Differentiate Commodity Infrastructure From MineWorld's Core Value

Do not spend project effort rebuilding commodity technology unless necessary.

Examples that should normally use mature existing implementations:

```text
SQLite
HTTP/WebSocket stacks
TLS
serialization
compression
cryptographic primitives
3D rendering engines
physics engines
basic navigation
containerization
LLM serving
standard database drivers
```

MineWorld's differentiated engineering effort should primarily go into concepts such as:

```text
modular world semantics
System composition
ActionIntent contracts
Process/Event separation
persistent causal simulation
Person/Controller separation
world truth vs memory
cross-renderer semantic interactions
world-pack composition
long-lived living-world behavior
```

Do not waste time recreating infrastructure that does not differentiate MineWorld.

---

## 5. Use Thin Adapters

When adopting an external dependency, avoid leaking it throughout the repository.

Prefer:

```text
MineWorld interface
       │
       ▼
thin adapter
       │
       ▼
external library
```

For example:

```text
PersistenceBackend
    └── SQLite adapter
```

rather than allowing SQLite-specific concepts throughout the simulation.

Likewise:

```text
NavigationBackend
    ├── Godot implementation
    └── future alternative
```

or:

```text
ModelProvider
    ├── Ollama
    ├── vLLM
    └── OpenAI-compatible
```

The adapter should be as thin as practical.

Do not reproduce the entire third-party API inside MineWorld unless MineWorld genuinely needs that abstraction.

---

## 6. Do Not Abstract Everything Prematurely

Replaceability does not mean creating an interface for every dependency immediately.

Create an abstraction when at least one of the following is true:

```text
multiple implementations are already needed;
the dependency crosses a major architecture boundary;
the dependency is likely to vary between local/cloud deployment;
the dependency would otherwise leak implementation-specific concepts into core code;
the component is central enough that future replacement would be very expensive.
```

Otherwise, direct use may be simpler.

Avoid speculative interfaces that add complexity without providing real architectural isolation.

---

## 7. Reuse at the Right Level

Sometimes the correct answer is:

```text
use the entire library
```

Sometimes:

```text
use one low-level component
```

Sometimes:

```text
study the implementation
but implement a smaller MineWorld-specific version
```

Sometimes:

```text
do not use it
```

For example, an existing project may contain useful:

```text
pathfinding
network protocol
agent memory
renderer integration
```

while its overall game architecture is incompatible with MineWorld.

It is acceptable to reuse the useful part without adopting the entire framework.

---

## 8. Existing AI / Agent Projects Are References, Not Foundations by Default

Projects such as AI Town, generative-agent frameworks, agent-society simulators, and game-NPC frameworks should be examined for reusable ideas and components.

Do not assume MineWorld should be built on top of them.

Ask:

```text
Does this component solve a MineWorld problem cleanly?

Can it be isolated behind our contracts?

Does its architecture scale to our modular world model?

Would adopting it reduce work without constraining future systems?
```

If yes, reuse it.

If no, learn from it and implement the required MineWorld abstraction cleanly.

---

## 9. Forking Is More Expensive Than Depending

Prefer, in order:

```text
upstream dependency
>
thin adapter
>
small maintained patch
>
fork
```

Forking a major project creates long-term maintenance responsibility.

Do not fork unless:

```text
upstream cannot support the required behavior;
the change is fundamental;
a small adapter cannot solve it;
and maintaining the divergence is justified.
```

If a fork becomes necessary, document why.

---

## 10. Avoid Copy-Paste Dependencies

Do not silently copy substantial code from external projects into MineWorld.

If external code is reused:

```text
respect its license;
preserve required attribution;
record its origin;
prefer a formal dependency where practical.
```

Do not turn third-party code into anonymous internal code that MineWorld must maintain forever.

---

## 11. Dependency Introduction Requires a Small Decision Record

For substantial dependencies, record a concise decision.

It should answer:

```text
What problem are we solving?

What existing solutions were considered?

Why was this one selected?

Why is implementing it ourselves worse?

What MineWorld interface isolates it?

What important limitations or lock-in risks exist?
```

This does not need to become a long document.

The purpose is to prevent future developers from replacing or duplicating a dependency without understanding why it exists.

---

## 12. Custom Implementation Requires Justification Too

If a mature existing solution was considered but rejected, briefly record why.

Valid reasons may include:

```text
architecture mismatch
license incompatibility
excessive deployment complexity
poor local-hosting support
unacceptable lock-in
missing required semantics
unmaintained project
performance mismatch
dependency larger than the problem
inability to isolate it cleanly
```

"Writing it ourselves feels cleaner" is not sufficient by itself.

---

## 13. Time-to-Working-System Matters

MineWorld is not an exercise in implementing every subsystem from first principles.

When two solutions are architecturally acceptable, prefer the one that gets a reliable integration running sooner.

The project should optimize for:

```text
correctness
maintainability
modularity
development velocity
```

together.

Do not spend several weeks building infrastructure that a proven library could provide in one day unless owning that implementation materially benefits MineWorld.

---

## 14. But Avoid Short-Term Speed That Creates Long-Term Lock-In

The reverse is equally important.

Do not save one day today by adopting a framework that will require rewriting MineWorld six months later.

Before adopting a large dependency, ask:

> **If MineWorld grows by an order of magnitude, is this dependency still an implementation detail, or does MineWorld become trapped inside its assumptions?**

If the latter, reconsider the design.

---

## 15. Prototype Before Committing to Large Dependencies

For important infrastructure decisions, prefer a small spike or integration prototype before deeply integrating the dependency.

Examples:

```text
Can this Rust crate support our event model?

Can Godot consume our server state cleanly?

Can this networking stack support local + public servers?

Can this ECS represent removable System Packs?

Can this plugin runtime enforce our capability boundaries?
```

Validate the risky assumption first.

Do not build months of architecture around an untested dependency assumption.

---

## 16. Pi-Agent Responsibility

The primary pi-agent should actively perform this reuse evaluation during planning.

Implementation agents should not automatically invent new infrastructure.

Before assigning a substantial custom implementation, the pi-agent should ask:

> **Does a mature open-source solution already solve this well enough?**

If the answer may plausibly be yes:

1. investigate existing options;
2. compare them against MineWorld requirements;
3. choose reuse, adaptation, or custom implementation;
4. record the decision;
5. continue autonomously.

Routine dependency decisions do not require human approval during MVP-0 if they are consistent with the frozen architecture.

---

## 17. Review Question

Every substantial design review should include:

> **Are we reinventing a mature wheel here?**

and also:

> **Are we forcing an existing wheel into a place where it does not fit?**

Both failure modes matter.

The correct solution is the one that maximizes:

> **architectural fit × maintainability × implementation efficiency**

not the one with either the fewest dependencies or the most dependencies.

---

## 19. Reference audits

*Added 2026-09-27, generalizing §8 from a warning into a practice.*

§8 says existing agent projects are references rather than foundations. That is a judgement, and a
judgement needs evidence, so MineWorld audits them deliberately rather than absorbing or dismissing
them by reputation.

A reference audit takes a project that has already built something we will need and asks, per area:

```text
what it does · what problem that solves · what is worth borrowing
what must not be copied · whether MineWorld already has a stronger abstraction
what mature component it reveals that we should reuse outright
```

Every idea is classified `REUSE` · `ADAPT` · `REFERENCE ONLY` · `REJECT`, with `REUSE` reserved for
things whose licence *and* architecture permit clean adoption.

Two rules make the audit useful rather than decorative:

1. **Every coupling criticism maps to a MineWorld rule.** A large file is not a finding; a large
   file that knows about both a coffee machine and an HTTP client is, because it violates the
   separation our invariants exist to hold. Style complaints without a rule behind them are noise.
2. **An audit is informational, never authoritative.** It does not change a frozen contract because
   another project chose differently. Where it finds a genuinely better answer to something we have
   *not* settled, that is recorded as a proposal and evaluated against the specs on its own.

Audits live in `docs/references/`. The practice exists to keep us out of both failure modes — *"it
has been done, so copy it"* and *"we are more ambitious, so write everything from nothing"* — and
to leave one honest principle in their place:

> **Do not re-step into pits others have already fallen into; reuse the wheels they have already
> built well; decide MineWorld's own abstraction and long-term direction ourselves.**

Audited so far: Microverse (`docs/references/MICROVERSE_AUDIT.md`, in progress). Worth auditing
next: AI Town, Concordia, AgentSociety, SimWorld.

## 18. Final Principle

MineWorld should build what is unique to MineWorld and reuse what the ecosystem already does well.

> **Reuse mature infrastructure when it fits.  
> Wrap it when isolation matters.  
> Extend it when extension remains clean.  
> Build our own only when MineWorld genuinely requires something different.**
