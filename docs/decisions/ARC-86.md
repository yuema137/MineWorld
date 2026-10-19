## ARC-86 — System-owned forgetting is lazy, pure and in the owning System; no model touches it

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by FG-a · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 ruling 2(6); QS24-7 accepted as recommended) · **Relates to** `INV-7`, `INV-11`, `INV-12`, `ARC-25`, `ARC-62`, `ARC-28`, `ARC-65`, `CORE_CONCEPTS.md` §5.3 · **Design** step-24 §§3.2, 8.2

**Problem.** System-owned person state grows: relationship familiarity and regard, the acquaintance list, and the conversation's remembered lines. Forgetting it could be a timer-driven Process that writes a fact per day, a controller policy that forgets world state, or a pure rule inside the owning System.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A Process that decays values on a timer and writes a fact per step | Declined: decay would add facts, journal rows and save growth every day, and it would break "no per-tick fact" |
| (b) A controller forgets world state for the world | Declined: a controller may not own a Component (`INV-7`); it can stop reading it, not forget it |
| (c) A lazy, pure rule in the owning System: each value is computed from `(value, last_contact, now)` where it is read, and stored state changes only inside reductions that already happen | **Chosen** |

**Choice.** Decay is a `DecayCurve` catalog entry (`ARC-62`) named in the owning System's `configure/<pack>.yaml`. The owning System clamps every curve result to `[floor, value]`, so forgetting only lowers a value toward its floor. The cap (`known_at_most`, default 150) evicts the lowest current familiarity inside the reduction that adds the acquaintance, emits `acquaintance-forgotten`, and unrelates `knows`. No model is involved. A level change caused by decay is observed lazily, and a biographical fact is emitted only by a reduction that touches the pair. Defaults change no reference world's bytes.

**Consequences.** Decay adds zero facts, zero journal rows and zero save growth, and replay reproduces it exactly. `last_contact` is already stored (`RelationshipValues::last_contact`), so no component encoding changes. The configuration payload grows, which bumps the System's version and refuses older saves by name (PA-3), and the new fact `acquaintance-forgotten` is declared. Read-time decay uses the operator-approved read-only clock `WorldRead::at()` (2026-10-12), a kernel addition available only during disclosure.

**Revisit** when a biographical decay must be visible without a reduction touching the pair, or when a System needs per-person state beyond `(value, last_contact)`.

**Note, 2026-10-13 (S24, PR FG-a; F-FGA-8, QFGA-4, QFGA-1) — two precisions.** (1) "Defaults change no reference world's bytes" means every fact and every run summary: a save's genesis journal row records the composition (each installed system's declaration and version), so raising `relationships` to 3 and `conversation` to 4 and declaring `acquaintance-forgotten` moves that one row, and `scripts/baselines.txt`'s `journal-300s` is re-recorded with a reason line while every later journal row, every fact and every summary stay byte-identical (`pr-fg-a-relationship-forgetting.md` FA-1). (2) `WorldRead::at()` is the world-time instant of whatever read holds it — a reaction's event instant, a validation's dispatch instant, `World::read()`'s `now()` — not something available only to disclosure; disclosure is its first user. It is never a wall clock (`INV-12`).
