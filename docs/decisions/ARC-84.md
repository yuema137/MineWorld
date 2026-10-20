## ARC-84 — The decision list is host-read content, not world state

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-a · **Approved by** the operator's step-24 rulings of 2026-10-12 (QS24-1, accepted as recommended) · **Relates to** `INV-1`, `ARC-61`, `ARC-64`, `ARC-33`, `ARC-53`, `AC-4`, `AC-5` · **Design** `.structured-coding/plans/mvp1/step-24-decision-and-memory.md` §4.1, §4.2

**Problem.** A world must choose which decision model drives each seat (rule, motivated, LM, director), with typed parameters, and may mix models per person. The choice could be seeded into world state as a `configure/` framework key, or kept outside the world. Seeding makes control a fact about a Person, which `INV-1` forbids, and it would trip the configure drift check at resume (`ARC-61`).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A World Pack file, `decisions.yaml`, read by hosts and never seeded; the operator may override it | **Chosen** |
| (b) A `configure/` framework key, seeded, with the drift check | Declined: makes control a world fact, and a controller change would alter world state at resume |
| (c) Operator-only configuration, with no file in the World Pack | Declined: the author's default casting of who decides how would be lost, and a world could not be run as authored |

**Choice.** The decision list is `decisions.yaml`, named by `world.yaml`'s `decisions:` key (`MODULE_SPEC.md` §4.1). It names models, never providers, model names, URLs or keys; the backend binding stays in the operator's `cognition.toml`. Hosts (`run`, `server --town`, the cognition process) read it; `--decisions FILE` and `--seat-model SEAT=MODEL` override it. Absent, every seat is `rule`, and every byte is as today. `run` prints the digest of the resolved list in its header. A save does not store the list; storing it as non-authoritative host metadata is deferred until a resume needs it (QS24-1).

**Consequences.** A controller choice never changes a Person or a world fact. The decision list adds one loader and no new pack type. Its assignments are validated against the same world with line and column, as `configure/` is.

**Revisit** when a resume needs the resolved list recorded in the save, or when a world needs a decision list that must travel with the save as authored content.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-9, SD-DLa-14, SD-DLa-15; QDLa-5, QFGb-1).** Two refinements, implemented by DL-a, neither changing the choice. (1) `run` prints the digest of the resolved list **only when a decision list is in effect** (a `decisions:` key, `--decisions` or `--seat-model`), as `; decisions <digest> (<counts>)` at the end of its header line; with none, the header is byte for byte what it was, because the parity record keeps that line and an always-printed clause would move every reference world's baseline. (2) `decisions.yaml`'s `memory:` section is decoded structurally by the Rust reader (unknown keys, entry shape, seats and classes, policy-id spelling, `parameters` as a string-keyed mapping, each refused with line and column), resolved per seat seat > class > default, and passed through uninterpreted as a typed `MemoryPolicyChoice` by `mineworld decisions <world> --json` (format 1, `MODULE_SPEC.md` §8.1). The cognition process validates the policy id and its parameters when it starts (FG-b's `resolve`), so an unknown policy id fails then, naming the file, line and column it was written at, not at `run`'s load. One YAML parser reads the file; the cognition process reads JSON.
