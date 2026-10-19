## DEP-28 — The Interaction List: our own narrow schema, on Cedar's semantics; no policy engine

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List", decision
numbers) and the primary session at PR IL-b's freeze · **Relates to** `REUSE_POLICY.md` §§2, 11–12, 17,
`ARC-63`, `QPL-1` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §3

**Problem.** The list decides three things, deterministically, on the hottest paths (`validate`,
offers, emission): is a (role classes, action, place) tuple allowed; which value of a typed parameter
applies; which audience and biographical flag a fact gets.

**Options considered** (step-18 §3.2, primary sources):

```text
flecs relationships        storage and queries, not rules; an ECS is DEP-1's rejection    reference
RimWorld / Factorio        data names code; ordered composition; untyped, game-bound      reference
Cedar                      exactly the rule semantics (scope triple, forbid overrides     adopt the semantics,
                           permit, schema validation); nothing for parameters or          not the engine
                           consequences; conditions are an expression language; default
                           deny; an entity set per request; "skip on error"
OPA / Rego                 general language; time, randomness and network builtins        reject
Casbin                     scripted matchers (rhai), async runtime, untyped model files   reject
interaction matrices       the presentation authors expect                                reference
our own SDK schema         all three columns, integers, sorted data, pure lookups         build, narrowly
```

**Choice: build, narrowly, with Cedar's semantics.** Our rule grammar is Cedar's scope triple without
`when`; our tie rule is forbid-overrides-permit; our sections are validated by their owner's type before
anything runs. Unlike Cedar, a world's default is `permit` (today's behaviour) and an erroring entry is
refused at load, never skipped. No dependency is added; YAML is `serde-saphyr` (`DEP-10`), as before.

**Revisit trigger.** Attribute conditions becoming a requirement (QIL-11): Cedar's engine is then the
candidate, not a language of our own.
