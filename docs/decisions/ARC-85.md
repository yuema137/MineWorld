## ARC-85 — Behaviour extension points share one pattern: interface, carrier, selection, parameters, host

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-a (decision models), FG-a and FG-b (forgetting), WE-a (world evolution) · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 rulings 2 and 3) · **Relates to** `ARC-62`, `ARC-64`, `ARC-61`, `ARC-34`, `ARC-53`, `CLAUDE.md` §4 rules 2, 7, 11 · **Design** step-24 §§0, 1.2, 3

**Problem.** Three behaviour extension points need to be pluggable: person decisions, forgetting, and world evolution. Each needs a typed interface, a selection mechanism, validated parameters, and bounds that a plug-in cannot exceed. Left ad hoc, each would invent a loader, a registry and a pack type, and bounds would live in plug-in code.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) One new pack type per extension point | Declined: `MODULE_SPEC.md` §1 already has the carriers, and a new pack type is an operator question |
| (b) One pattern over the existing carriers: five parts, the interface, the carrier pack type, selection by content, typed parameters, and a host that enforces the bounds | **Chosen** |
| (c) A plug-in registry with untyped metadata and no host around the plug-in | Declined: it violates `CLAUDE.md` §4 rule 7 and lets a plug-in exceed a budget |

**Choice.** Each extension point has five parts: (1) a typed interface (Rust trait or Python Protocol, named types only); (2) a carrier that is an existing pack type (System Pack or Controller Pack); (3) selection by content, per world, per class (`ARC-64`), and per person, with precedence person > class > default, refused by name when unknown, and ambiguity refused at load; (4) parameters typed by the implementation, each with a bound and a default; (5) a host, which is code around the plug-in that enforces the hard bounds and invariants whatever the plug-in returns. Each point ships bundled implementations that are ordinary packs, none privileged, and two examples: one without a language model and one with one. No layer may require a model. An LLM never mutates world state.

**Consequences.** Adding a fourth extension point is a design question, not a new mechanism. A world's behaviour is selected by its `systems:`, `configure/` and `decisions.yaml`, and a user's rule is installed like any other pack.

**Revisit** when a proposed extension point cannot be expressed by the carriers above, or when a host bound is found to be enforceable only by a plug-in.
